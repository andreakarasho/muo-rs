//! Bulk orders: the bulk order deeds (`BaseBOD`, `SmallBOD`, `LargeBOD` and the
//! smith and tailor classes), their entry tables (`Data/Bulk Orders/*/*.cfg`), the rewards (`rewards`), the deed
//! gumps (`gumps`) and the bulk order book with its entries (`book`).
//!
//! The deed state is one [`BodData`] in `Item::ext`: a small deed uses `amount_cur`, `type_name`, `number` and
//! `graphic`, a large one `entries`. A `Type` is kept as the full name of its class (a type the registry does not
//! know reads back as null, as `ReadType` does).
//!
//! `{number:#}` of the property lines is the custom number format `#`, which prints plain digits (and nothing for 0).

use std::collections::HashMap;
use std::rc::Rc;

use muo_content::craft::CraftResource;
use muo_content::hooks::rnd;
use muo_io::{GenericReader, GenericWriter, Result};
use muo_server::World;
use muo_server::ext::Ext;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType, LootType, deps, ops};
use muo_server::layer::Layer;
use muo_server::property_list::PropertyList;
use muo_server::serialization_ext::{ReadEntityExt, WriteEntityExt};
use muo_server::skills::{self, SkillName};
use muo_server::target::TargetFlags;
use muo_server::type_registry::TypeRegistry;

use crate::craft;

pub mod book;
mod gold_tables;
pub mod gumps;
pub mod rewards;

#[cfg(test)]
mod tests;

/// `BulkMaterialType`.
pub mod mat {
    pub const NONE: i32 = 0;
    pub const DULL_COPPER: i32 = 1;
    pub const SHADOW_IRON: i32 = 2;
    pub const COPPER: i32 = 3;
    pub const BRONZE: i32 = 4;
    pub const GOLD: i32 = 5;
    pub const AGAPITE: i32 = 6;
    pub const VERITE: i32 = 7;
    pub const VALORITE: i32 = 8;
    pub const SPINED: i32 = 9;
    pub const HORNED: i32 = 10;
    pub const BARBED: i32 = 11;
}

/// `BODType`.
pub const BOD_SMITH: i32 = 0;
pub const BOD_TAILOR: i32 = 1;

/// `BulkGenericType`.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum BulkGenericType {
    Iron,
    Cloth,
    Leather,
}

/// `BGTClassifier.Classify(deedType, itemType)`: `itemType` null is leather.
pub fn classify(w: &World, deed_type: i32, item_type: Option<&str>) -> BulkGenericType {
    if deed_type != BOD_TAILOR {
        return BulkGenericType::Iron;
    }
    let Some(t) = item_type else {
        return BulkGenericType::Leather;
    };
    let sub = |base: &str| {
        craft::item::resolve(w, t)
            .is_some_and(|ty| ty.full_name != base && muo_content::items::is_named(ty, base))
    };
    if sub("Server.Items.BaseArmor") || sub("Server.Items.BaseShoes") {
        BulkGenericType::Leather
    } else {
        BulkGenericType::Cloth
    }
}

/// `SmallBulkEntry`.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SmallBulkEntry {
    pub type_name: Option<&'static str>,
    pub number: i32,
    pub graphic: i32,
}

/// `LargeBulkEntry`.
#[derive(Clone, Debug)]
pub struct LargeBulkEntry {
    pub amount: i32,
    pub details: SmallBulkEntry,
}

/// What a deed adds to an `Item`.
#[derive(Clone, Debug, Default)]
pub struct BodData {
    pub amount_max: i32,
    pub require_exceptional: bool,
    pub material: i32,
    pub amount_cur: i32,
    pub type_name: Option<&'static str>,
    pub number: i32,
    pub graphic: i32,
    pub entries: Vec<LargeBulkEntry>,
}

pub fn data(w: &World, id: ItemId) -> Option<&BodData> {
    w.item(id)?.ext.get::<BodData>()
}

pub fn data_mut(w: &mut World, id: ItemId) -> Option<&mut BodData> {
    w.item_mut(id)?.ext.get_mut::<BodData>()
}

pub fn is_bod(w: &World, id: ItemId) -> bool {
    w.item(id).is_some_and(|i| i.ty.is(&BASE_BOD))
}

pub fn is_small(w: &World, id: ItemId) -> bool {
    w.item(id).is_some_and(|i| i.ty.is(&SMALL_BOD))
}

pub fn is_large(w: &World, id: ItemId) -> bool {
    w.item(id).is_some_and(|i| i.ty.is(&LARGE_BOD))
}

/// `this is SmallTailorBOD or LargeTailorBOD`.
pub fn is_tailor(w: &World, id: ItemId) -> bool {
    w.item(id)
        .is_some_and(|i| i.ty.is(&SMALL_TAILOR_BOD) || i.ty.is(&LARGE_TAILOR_BOD))
}

/// `Complete`.
pub fn complete(w: &World, id: ItemId) -> bool {
    let Some(d) = data(w, id) else { return false };
    if is_small(w, id) {
        d.amount_cur == d.amount_max
    } else {
        d.entries.iter().all(|e| e.amount >= d.amount_max)
    }
}

// ---- the entry files ----

#[derive(Default)]
struct EntryCache(HashMap<(String, String), Rc<Vec<SmallBulkEntry>>>);

/// `Utility.ToInt32(text)`: `0x` is hex; a bad number is 0.
fn to_int32(s: &str) -> i32 {
    muo_primitives::parse::to_i32(s)
}

/// `SmallBulkEntry.GetEntries(type, name)` / `LargeBulkEntry.GetEntries`: `Data/Bulk Orders/{type}/{name}.cfg`, cached.
pub fn get_entries(w: &mut World, kind: &str, name: &str) -> Rc<Vec<SmallBulkEntry>> {
    let key = (kind.to_owned(), name.to_owned());
    if let Some(e) = w.resources.get_mut::<EntryCache>().0.get(&key) {
        return e.clone();
    }
    let list = Rc::new(load_entries(w, kind, name));
    w.resources
        .get_mut::<EntryCache>()
        .0
        .insert(key, list.clone());
    list
}

fn load_entries(w: &World, kind: &str, name: &str) -> Vec<SmallBulkEntry> {
    let path = w
        .core
        .base_directory
        .join("Data/Bulk Orders")
        .join(kind)
        .join(format!("{name}.cfg"));
    let mut list = Vec::new();
    let Ok(text) = std::fs::read_to_string(path) else {
        return list;
    };
    for line in text.lines() {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let split: Vec<&str> = line.split('\t').collect();
        if split.len() >= 2 {
            let ty = w.types.find_type_by_name(split[0], false, true);
            let graphic = to_int32(split[split.len() - 1]);
            if let Some(ty) = ty
                && graphic > 0
            {
                list.push(SmallBulkEntry {
                    type_name: Some(ty.full_name()),
                    number: if graphic < 0x4000 {
                        1020000 + graphic
                    } else {
                        1078872 + graphic
                    },
                    graphic,
                });
            }
        }
    }
    list
}

// ---- classes ----

fn new_bod(w: &mut World, ty: &'static ItemType) -> ItemId {
    let item_id = if muo_content::core::aos(w) {
        0x2258
    } else {
        0x14EF
    };
    let id = ops::new_item_with_id(w, ty, item_id);
    ops::set_loot_type(w, id, LootType::Blessed);
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(BodData::default());
    }
    id
}

/// `BaseBOD(hue, amountMax, requireExceptional, material)`.
fn init_base(w: &mut World, id: ItemId, hue: i32, amount_max: i32, req: bool, material: i32) {
    if let Some(ty) = w.item(id).map(|i| i.ty) {
        (ty.set_hue)(w, id, hue);
    }
    if let Some(d) = data_mut(w, id) {
        d.amount_max = amount_max;
        d.require_exceptional = req;
        d.material = material;
    }
}

/// `BaseBOD.GetRandomMaterial(start, chances)`.
pub fn get_random_material(w: &mut World, start: i32, chances: &[f64]) -> i32 {
    let mut random = rnd::double(w);
    for (i, c) in chances.iter().enumerate() {
        if random < *c {
            return if i == 0 {
                mat::NONE
            } else {
                start + (i as i32 - 1)
            };
        }
        random -= c;
    }
    mat::NONE
}

const BLACKSMITH_CHANCES: [f64; 9] = [
    0.501953125,
    0.25,
    0.125,
    0.0625,
    0.03125,
    0.015625,
    0.0078125,
    0.00390625,
    0.001953125,
];
const TAILORING_CHANCES: [f64; 4] = [0.857421875, 0.125, 0.015625, 0.001953125];

/// `Utility.RandomList(params int[])`.
fn random_list(w: &mut World, list: &[i32]) -> i32 {
    list[rnd::below(w, list.len() as i32) as usize]
}

fn random_entry(w: &mut World, list: &[SmallBulkEntry]) -> SmallBulkEntry {
    list[rnd::below(w, list.len() as i32) as usize].clone()
}

/// `SmallSmithBOD(amountCur, amountMax, type, number, graphic, reqExceptional, mat)` (and the tailor one, `hue`).
#[allow(clippy::too_many_arguments)]
pub fn new_small_with(
    w: &mut World,
    ty: &'static ItemType,
    hue: i32,
    amount_cur: i32,
    amount_max: i32,
    type_name: Option<&'static str>,
    number: i32,
    graphic: i32,
    req: bool,
    material: i32,
) -> ItemId {
    let id = new_bod(w, ty);
    init_base(w, id, hue, amount_max, req, material);
    if let Some(d) = data_mut(w, id) {
        d.type_name = type_name;
        d.graphic = graphic;
        d.amount_cur = amount_cur;
        d.number = number;
    }
    id
}

/// The `[Constructible]` `SmallSmithBOD()` / `SmallTailorBOD()`.
fn small_random(w: &mut World, ty: &'static ItemType, smith: bool) -> ItemId {
    let id = new_bod(w, ty);
    let use_materials = rnd::bool(w);
    let (kind, a, b) = if smith {
        ("Blacksmith", "armor", "weapons")
    } else {
        ("Tailoring", "leather", "cloth")
    };
    let entries = get_entries(w, kind, if use_materials { a } else { b });
    if entries.is_empty() {
        return id;
    }
    let hue = if smith { 0x44E } else { 0x483 };
    let amount_max = random_list(w, &[10, 15, 20]);
    let material = if use_materials {
        if smith {
            get_random_material(w, mat::DULL_COPPER, &BLACKSMITH_CHANCES)
        } else {
            get_random_material(w, mat::SPINED, &TAILORING_CHANCES)
        }
    } else {
        mat::NONE
    };
    let req = rnd::bool(w) || material == mat::NONE;
    let entry = random_entry(w, &entries);
    init_base(w, id, hue, amount_max, req, material);
    if let Some(d) = data_mut(w, id) {
        d.type_name = entry.type_name;
        d.number = entry.number;
        d.graphic = entry.graphic;
    }
    id
}

/// `skillReq` of `CreateRandomFor`'s material roll.
fn skill_req(material: i32) -> f64 {
    match material {
        mat::DULL_COPPER => 65.0,
        mat::SHADOW_IRON => 70.0,
        mat::COPPER => 75.0,
        mat::BRONZE => 80.0,
        mat::GOLD => 85.0,
        mat::AGAPITE => 90.0,
        mat::VERITE => 95.0,
        mat::VALORITE => 100.0,
        mat::SPINED => 65.0,
        mat::HORNED => 80.0,
        mat::BARBED => 99.0,
        _ => 0.0,
    }
}

/// `SmallSmithBOD.CreateRandomFor(m)` / `SmallTailorBOD.CreateRandomFor(m)`.
pub fn create_small_for(w: &mut World, m: MobileId, smith: bool) -> Option<ItemId> {
    let use_materials = rnd::bool(w);
    let skill = if smith {
        SkillName::Blacksmith
    } else {
        SkillName::Tailoring
    };
    let their_skill = skills::base(w, m, skill);
    let entries = if smith {
        get_entries(
            w,
            "Blacksmith",
            if use_materials { "armor" } else { "weapons" },
        )
    } else if use_materials && their_skill >= 6.2 {
        // Ugly, but the easiest leather BOD is Leather Cap which requires at least 6.2 skill.
        get_entries(w, "Tailoring", "leather")
    } else {
        get_entries(w, "Tailoring", "cloth")
    };
    if entries.is_empty() {
        return None;
    }
    let amount_max = if their_skill >= 70.1 {
        random_list(w, &[10, 15, 20, 20])
    } else if their_skill >= 50.1 {
        random_list(w, &[10, 15, 15, 20])
    } else {
        random_list(w, &[10, 10, 15, 20])
    };
    let mut material = mat::NONE;
    if use_materials && their_skill >= 70.1 {
        for _ in 0..20 {
            let check = if smith {
                get_random_material(w, mat::DULL_COPPER, &BLACKSMITH_CHANCES)
            } else {
                get_random_material(w, mat::SPINED, &TAILORING_CHANCES)
            };
            // The tailor table has no ShadowIron/Copper rows, but those rolls cannot come from its chances.
            if their_skill >= skill_req(check) {
                material = check;
                break;
            }
        }
    }
    let exc_chance = if their_skill >= 70.1 {
        (their_skill + 80.0) / 200.0
    } else {
        0.0
    };
    let req = exc_chance > rnd::double(w);

    let sys = craft::system(w, if smith { "Blacksmithy" } else { "Tailoring" })?;
    let mut valid = Vec::new();
    for e in entries.iter() {
        let Some(idx) = e.type_name.and_then(|t| sys.search_for(t)) else {
            continue;
        };
        let item = sys.items[idx].clone();
        let mut all = true;
        let mut chance = craft::item::success_chance(w, &sys, &item, m, false, &mut all);
        if all && chance >= 0.0 {
            if req {
                chance = craft::item::exceptional_chance(w, &sys, &item, chance, m);
            }
            if chance > 0.0 {
                valid.push(e.clone());
            }
        }
    }
    if valid.is_empty() {
        return None;
    }
    let entry = random_entry(w, &valid);
    let (ty, hue) = if smith {
        (&SMALL_SMITH_BOD, 0x44E)
    } else {
        (&SMALL_TAILOR_BOD, 0x483)
    };
    Some(new_small_with(
        w,
        ty,
        hue,
        0,
        amount_max,
        entry.type_name,
        entry.number,
        entry.graphic,
        req,
        material,
    ))
}

/// `LargeSmithBOD(amountMax, reqExceptional, mat, entries)` / `LargeTailorBOD(..)`.
pub fn new_large_with(
    w: &mut World,
    ty: &'static ItemType,
    hue: i32,
    amount_max: i32,
    req: bool,
    material: i32,
    entries: Vec<LargeBulkEntry>,
) -> ItemId {
    let id = new_bod(w, ty);
    init_base(w, id, hue, amount_max, req, material);
    if let Some(d) = data_mut(w, id) {
        d.entries = entries;
    }
    id
}

fn convert(list: &[SmallBulkEntry]) -> Vec<LargeBulkEntry> {
    list.iter()
        .map(|d| LargeBulkEntry {
            amount: 0,
            details: d.clone(),
        })
        .collect()
}

/// The `[Constructible]` `LargeSmithBOD()`.
fn large_smith_random(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = new_bod(w, ty);
    let rand = rnd::below(w, 8);
    let name = match rand {
        1 => "largeplate",
        2 => "largechain",
        3 => "largeaxes",
        4 => "largefencing",
        5 => "largemaces",
        6 => "largepolearms",
        7 => "largeswords",
        _ => "largering",
    };
    let entries = convert(&get_entries(w, "Blacksmith", name));
    let use_materials = !(rand > 2 && rand < 8);
    let amount_max = random_list(w, &[10, 15, 20, 20]);
    let req = rnd::double(w) < 0.825;
    let material = if use_materials {
        get_random_material(w, mat::DULL_COPPER, &BLACKSMITH_CHANCES)
    } else {
        mat::NONE
    };
    init_base(w, id, 0x44E, amount_max, req, material);
    if let Some(d) = data_mut(w, id) {
        d.entries = entries;
    }
    id
}

/// The `[Constructible]` `LargeTailorBOD()`.
fn large_tailor_random(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = new_bod(w, ty);
    let mut use_materials = false;
    let name = match rnd::below(w, 14) {
        1 => {
            use_materials = true;
            "femaleleatherset"
        }
        2 => "fishergirl",
        3 => "gypsy",
        4 => "hatset",
        5 => "jester",
        6 => "lady",
        7 => {
            use_materials = true;
            "maleleatherset"
        }
        8 => "pirate",
        9 => {
            use_materials = muo_content::core::ml(w);
            "shoeset"
        }
        10 => {
            use_materials = true;
            "studdedset"
        }
        11 => "towncrier",
        12 => "wizard",
        13 => {
            use_materials = true;
            "boneset"
        }
        _ => "farmer",
    };
    let entries = convert(&get_entries(w, "Tailoring", name));
    let amount_max = random_list(w, &[10, 15, 20, 20]);
    let req = rnd::double(w) < 0.825;
    let material = if use_materials {
        get_random_material(w, mat::SPINED, &TAILORING_CHANCES)
    } else {
        mat::NONE
    };
    init_base(w, id, 0x483, amount_max, req, material);
    if let Some(d) = data_mut(w, id) {
        d.entries = entries;
    }
    id
}

// ---- properties and use ----

fn fmt_number(n: i32) -> String {
    if n == 0 { String::new() } else { n.to_string() }
}

/// `GetMaterialNumberFor(material)`.
pub fn material_number(material: i32) -> i32 {
    if (mat::DULL_COPPER..=mat::VALORITE).contains(&material) {
        return 1045142 + (material - mat::DULL_COPPER);
    }
    if (mat::SPINED..=mat::BARBED).contains(&material) {
        return 1049348 + (material - mat::SPINED);
    }
    0
}

fn small_get_properties(w: &World, id: ItemId, list: &mut PropertyList) {
    (ITEM_BASE.get_properties)(w, id, list);
    let d = data(w, id).cloned().unwrap_or_default();
    list.add(1060654); // small bulk order
    if d.require_exceptional {
        list.add(1045141); // All items must be exceptional.
    }
    if d.material != mat::NONE {
        list.add(material_number(d.material)); // All items must be made with x material.
    }
    list.add_int(1060656, d.amount_max); // amount to make: ~1_val~
    list.add_str(
        1060658,
        &format!("{}\t{}", fmt_number(d.number), d.amount_cur),
    ); // ~1_val~: ~2_val~
}

fn large_get_properties(w: &World, id: ItemId, list: &mut PropertyList) {
    (ITEM_BASE.get_properties)(w, id, list);
    let d = data(w, id).cloned().unwrap_or_default();
    list.add(1060655); // large bulk order
    if d.require_exceptional {
        list.add(1045141); // All items must be exceptional.
    }
    if d.material != mat::NONE {
        list.add(material_number(d.material)); // All items must be made with x material.
    }
    list.add_int(1060656, d.amount_max); // amount to make: ~1_val~
    for (i, e) in d.entries.iter().enumerate() {
        list.add_str(
            1060658 + i as i32,
            &format!("{}\t{}", fmt_number(e.details.number), e.amount),
        );
    }
}

pub(crate) fn in_pack(w: &World, id: ItemId, from: MobileId) -> bool {
    let pack = deps::find_item_on_layer(w, from, Layer::Backpack);
    pack.is_some_and(|p| ops::is_child_of(w, id, Some(EntityId::Item(p))))
}

fn bulk_orders_enabled(w: &World) -> bool {
    w.resources
        .get::<crate::feature_flags::ContentFeatureFlags>()
        .is_none_or(|c| c.bulk_orders)
}

/// The use check both deeds share; true when the gump may open.
fn may_use(w: &mut World, id: ItemId, from: MobileId) -> bool {
    let vendor = muo_content::items::common::hooks(w)
        .and_then(|h| h.root_is_player_vendor)
        .is_some_and(|f| f(w, id));
    if !(in_pack(w, id, from) || ops::in_secure_trade(w, id) || vendor) {
        deps::send_localized_message(w, from, 1045156); // You must have the deed in your backpack to use it.
        return false;
    }
    if !bulk_orders_enabled(w)
        && muo_content::mx::access_level(w, from) < muo_server::mobile::AccessLevel::Administrator
    {
        deps::send_message_hue(w, from, 0x22, "Bulk orders are temporarily disabled.");
        return false;
    }
    true
}

fn on_double_click(w: &mut World, id: ItemId, from: MobileId) {
    if may_use(w, id, from) {
        gumps::send_bod_gump(w, from, id);
    }
}

/// `BeginCombine(from)`.
pub fn begin_combine(w: &mut World, id: ItemId, from: MobileId) {
    if complete(w, id) {
        deps::send_localized_message(w, from, 1045166); // The maximum amount of requested items have already been combined to this deed.
        return;
    }
    // `BODTarget`
    muo_server::target::begin_target(
        w,
        from,
        18,
        false,
        TargetFlags::NONE,
        move |w, from, targeted| {
            if ops::is_deleted(w, id) || !in_pack(w, id, from) {
                return;
            }
            let item = match targeted {
                muo_server::target::Targeted::Entity(EntityId::Item(i)) if in_pack(w, i, from) => i,
                _ => {
                    deps::send_localized_message(w, from, 1045158); // You must have the item in your backpack to target it.
                    return;
                }
            };
            end_combine(w, id, from, item);
        },
    );
}

/// `SmallBOD.GetMaterial(resource)`.
pub fn get_material(resource: CraftResource) -> i32 {
    match resource {
        CraftResource::DullCopper => mat::DULL_COPPER,
        CraftResource::ShadowIron => mat::SHADOW_IRON,
        CraftResource::Copper => mat::COPPER,
        CraftResource::Bronze => mat::BRONZE,
        CraftResource::Gold => mat::GOLD,
        CraftResource::Agapite => mat::AGAPITE,
        CraftResource::Verite => mat::VERITE,
        CraftResource::Valorite => mat::VALORITE,
        CraftResource::SpinedLeather => mat::SPINED,
        CraftResource::HornedLeather => mat::HORNED,
        CraftResource::BarbedLeather => mat::BARBED,
        _ => mat::NONE,
    }
}

/// `EndCombine(from, item)`.
pub fn end_combine(w: &mut World, id: ItemId, from: MobileId, item: ItemId) {
    if is_small(w, id) {
        end_combine_small(w, id, from, item);
    } else {
        end_combine_large(w, id, from, item);
    }
}

fn end_combine_small(w: &mut World, id: ItemId, from: MobileId, item: ItemId) {
    use muo_content::items::{armor, clothing, weapon};
    let d = data(w, id).cloned().unwrap_or_default();
    if d.amount_cur >= d.amount_max {
        deps::send_localized_message(w, from, 1045166); // The maximum amount of requested items have already been combined to this deed.
        return;
    }
    let Some(ity) = w.item(item).map(|i| i.ty) else {
        return;
    };
    let (is_armor, is_clothing, is_weapon) = (
        ity.is(&armor::BASE_ARMOR),
        ity.is(&clothing::BASE_CLOTHING),
        ity.is(&weapon::BASE_WEAPON),
    );
    let of_type = d
        .type_name
        .is_some_and(|t| muo_content::items::is_named(ity, t));
    if !of_type || !(is_armor || is_clothing || is_weapon) {
        deps::send_localized_message(w, from, 1045169); // The item is not in the request.
        return;
    }
    let resource = if is_armor {
        armor::data(w, item).map(|a| a.resource)
    } else if is_clothing {
        clothing::data(w, item).map(|c| c.resource)
    } else {
        None
    };
    let material = get_material(resource.unwrap_or(CraftResource::None));
    let exceptional = (is_armor
        && armor::data(w, item).is_some_and(|a| a.quality == armor::ArmorQuality::Exceptional))
        || (is_clothing
            && clothing::data(w, item)
                .is_some_and(|c| c.quality == clothing::ClothingQuality::Exceptional))
        || (is_weapon
            && weapon::data(w, item)
                .is_some_and(|x| x.quality == weapon::WeaponQuality::Exceptional));
    if (mat::DULL_COPPER..=mat::VALORITE).contains(&d.material) && material != d.material {
        deps::send_localized_message(w, from, 1045168); // The item is not made from the requested ore.
    } else if (mat::SPINED..=mat::BARBED).contains(&d.material) && material != d.material {
        deps::send_localized_message(w, from, 1049352); // The item is not made from the requested leather type.
    } else if d.require_exceptional && !exceptional {
        deps::send_localized_message(w, from, 1045167); // The item must be exceptional.
    } else {
        (ity.delete)(w, item);
        if let Some(x) = data_mut(w, id) {
            x.amount_cur += 1;
        }
        ops::invalidate_properties(w, id);
        deps::send_localized_message(w, from, 1045170); // The item has been combined with the deed.
        gumps::send_bod_gump(w, from, id);
        if d.amount_cur + 1 < d.amount_max {
            begin_combine(w, id, from);
        }
    }
}

fn end_combine_large(w: &mut World, id: ItemId, from: MobileId, item: ItemId) {
    let d = data(w, id).cloned().unwrap_or_default();
    if !is_small(w, item) {
        deps::send_localized_message(w, from, 1045159); // That is not a bulk order.
        return;
    }
    let small = data(w, item).cloned().unwrap_or_default();
    let found = d
        .entries
        .iter()
        .position(|e| e.details.type_name == small.type_name);
    let Some(entry) = found else {
        deps::send_localized_message(w, from, 1045160); // That is not a bulk order for this large request.
        return;
    };
    if d.require_exceptional && !small.require_exceptional {
        deps::send_localized_message(w, from, 1045161); // Both orders must be of exceptional quality.
    } else if (mat::DULL_COPPER..=mat::VALORITE).contains(&d.material)
        && small.material != d.material
    {
        deps::send_localized_message(w, from, 1045162); // Both orders must use the same ore type.
    } else if (mat::SPINED..=mat::BARBED).contains(&d.material) && small.material != d.material {
        deps::send_localized_message(w, from, 1049351); // Both orders must use the same leather type.
    } else if d.amount_max != small.amount_max {
        deps::send_localized_message(w, from, 1045163); // The two orders have different requested amounts and cannot be combined.
    } else if small.amount_cur < small.amount_max {
        deps::send_localized_message(w, from, 1045164); // The order to combine with is not completed.
    } else if d.entries[entry].amount >= d.amount_max {
        deps::send_localized_message(w, from, 1045166); // The maximum amount of requested items have already been combined to this deed.
    } else {
        if let Some(x) = data_mut(w, id) {
            x.entries[entry].amount += small.amount_cur;
        }
        ops::invalidate_properties(w, id);
        if let Some(t) = w.item(item).map(|i| i.ty) {
            (t.delete)(w, item);
        }
        deps::send_localized_message(w, from, 1045165); // The orders have been combined.
        gumps::send_bod_gump(w, from, id);
        if !complete(w, id) {
            begin_combine(w, id, from);
        }
    }
}

// ---- rewards ----

/// `GetRewardGroup` / `ComputeGold` / `ComputeFame` through the deed's calculator.
pub fn compute_gold(w: &mut World, id: ItemId) -> i32 {
    rewards::compute_gold(w, id)
}

/// `BaseBOD.GetRewards(out reward, out gold, out fame)`: the reward item (if any), the gold and the fame.
pub fn get_rewards(w: &mut World, id: ItemId) -> (Option<ItemId>, i32, i32) {
    let gold = rewards::compute_gold(w, id);
    let fame = rewards::compute_fame(w, id);
    let rewards = rewards::compute_rewards(w, id, false);
    let pick = if rewards.is_empty() {
        None
    } else {
        Some(rnd::below(w, rewards.len() as i32) as usize)
    };
    let reward = pick.and_then(|i| rewards::construct(w, &rewards[i]));
    (reward, gold, fame)
}

// ---- serialization ----

fn write_base(out: &mut dyn GenericWriter, d: &BodData) {
    out.write_encoded_int(1);
    out.write_i32(d.amount_max);
    out.write_bool(d.require_exceptional);
    out.write_enum(4, i64::from(d.material));
}

fn serialize_base(w: &World, id: ItemId, out: &mut dyn GenericWriter) {
    (ITEM_BASE.serialize)(w, id, out);
    write_base(out, &data(w, id).cloned().unwrap_or_default());
}

fn deserialize_base(w: &mut World, id: ItemId, r: &mut dyn GenericReader) -> Result<BodData> {
    (ITEM_BASE.deserialize)(w, id, r)?;
    let _version = r.read_encoded_int()?; // both versions have the same fields
    let mut d = BodData {
        amount_max: r.read_i32()?,
        require_exceptional: r.read_bool()?,
        ..Default::default()
    };
    d.material = r.read_enum(4)? as i32;
    Ok(d)
}

/// `BaseBOD.AfterDeserialization`: a deed nowhere (no parent, internal map, location zero) is deleted.
fn after_deserialization(w: &mut World, id: ItemId) {
    muo_server::timer::start(
        w,
        muo_primitives::TimeSpan::ZERO,
        muo_primitives::TimeSpan::ZERO,
        1,
        move |w| {
            let nowhere = w.item(id).is_some_and(|i| {
                i.parent.is_none()
                    && i.map == Some(muo_server::map::MapId::INTERNAL)
                    && i.location == muo_primitives::Point3D::ZERO
            });
            if nowhere && let Some(ty) = w.item(id).map(|i| i.ty) {
                (ty.delete)(w, id);
            }
        },
    );
}

fn small_serialize(w: &World, id: ItemId, out: &mut dyn GenericWriter) {
    serialize_base(w, id, out);
    let d = data(w, id).cloned().unwrap_or_default();
    out.write_i32(0);
    out.write_i32(d.amount_cur);
    out.write_type(d.type_name);
    out.write_i32(d.number);
    out.write_i32(d.graphic);
}

fn small_deserialize(w: &mut World, id: ItemId, r: &mut dyn GenericReader) -> Result<()> {
    let mut d = deserialize_base(w, id, r)?;
    let _version = r.read_i32()?;
    d.amount_cur = r.read_i32()?;
    d.type_name = ReadEntityExt::read_type(r, w)?.map(|t| t.full_name());
    d.number = r.read_i32()?;
    d.graphic = r.read_i32()?;
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(d);
    }
    after_deserialization(w, id);
    Ok(())
}

fn large_serialize(w: &World, id: ItemId, out: &mut dyn GenericWriter) {
    serialize_base(w, id, out);
    let d = data(w, id).cloned().unwrap_or_default();
    out.write_encoded_int(1);
    out.write_encoded_int(d.entries.len() as i32);
    for e in &d.entries {
        out.write_i32(e.amount);
        out.write_opt_string(e.details.type_name);
        out.write_i32(e.details.number);
        out.write_i32(e.details.graphic);
    }
}

fn large_deserialize(w: &mut World, id: ItemId, r: &mut dyn GenericReader) -> Result<()> {
    let mut d = deserialize_base(w, id, r)?;
    let version = r.read_encoded_int()?;
    let count = if version < 1 {
        r.read_i32()?
    } else {
        r.read_encoded_int()?
    };
    for _ in 0..count.max(0) {
        let amount = r.read_i32()?;
        let type_name = r
            .read_string()?
            .and_then(|n| w.types.find_type_by_full_name(&n, true))
            .map(|t| t.full_name());
        let number = r.read_i32()?;
        let graphic = r.read_i32()?;
        d.entries.push(LargeBulkEntry {
            amount,
            details: SmallBulkEntry {
                type_name,
                number,
                graphic,
            },
        });
    }
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(d);
    }
    after_deserialization(w, id);
    Ok(())
}

// ---- the item types ----

/// `Server.Engines.BulkOrders.BaseBOD` (abstract).
pub static BASE_BOD: ItemType = ItemType {
    full_name: "Server.Engines.BulkOrders.BaseBOD",
    parent: Some(&ITEM_BASE),
    construct: None,
    default_weight: |w, id| {
        crate::template_keys::item_float(w, id, "defaultWeight", ITEM_BASE.default_weight)
    },
    ..ITEM_BASE
};

/// `Server.Engines.BulkOrders.SmallBOD` (abstract).
pub static SMALL_BOD: ItemType = ItemType {
    full_name: "Server.Engines.BulkOrders.SmallBOD",
    parent: Some(&BASE_BOD),
    label_number: |w, id| {
        crate::template_keys::item_int(w, id, "labelNumber", BASE_BOD.label_number)
    },
    get_properties: small_get_properties,
    on_double_click,
    on_double_click_not_accessible: on_double_click,
    on_double_click_secure_trade: on_double_click,
    serialize: small_serialize,
    deserialize: small_deserialize,
    ..BASE_BOD
};

/// `Server.Engines.BulkOrders.LargeBOD` (abstract).
pub static LARGE_BOD: ItemType = ItemType {
    full_name: "Server.Engines.BulkOrders.LargeBOD",
    parent: Some(&BASE_BOD),
    label_number: |w, id| {
        crate::template_keys::item_int(w, id, "labelNumber", BASE_BOD.label_number)
    },
    get_properties: large_get_properties,
    on_double_click,
    on_double_click_not_accessible: on_double_click,
    on_double_click_secure_trade: on_double_click,
    serialize: large_serialize,
    deserialize: large_deserialize,
    ..BASE_BOD
};

macro_rules! concrete {
    ($(#[$m:meta])* $name:ident = $full:literal : $parent:ident, $construct:expr) => {
        $(#[$m])*
        pub static $name: ItemType = ItemType {
            full_name: $full,
            parent: Some(&$parent),
            construct: Some($construct),
            serialize: |w, id, out| {
                ($parent.serialize)(w, id, out);
                out.write_i32(0);
            },
            deserialize: |w, id, r| {
                ($parent.deserialize)(w, id, r)?;
                let _version = r.read_i32()?;
                Ok(())
            },
            ..$parent
        };
    };
}

concrete!(/// `SmallSmithBOD`.
    SMALL_SMITH_BOD = "Server.Engines.BulkOrders.SmallSmithBOD" : SMALL_BOD, |w| small_random(w, &SMALL_SMITH_BOD, true));
concrete!(/// `SmallTailorBOD`.
    SMALL_TAILOR_BOD = "Server.Engines.BulkOrders.SmallTailorBOD" : SMALL_BOD, |w| small_random(w, &SMALL_TAILOR_BOD, false));
concrete!(/// `LargeSmithBOD`.
    LARGE_SMITH_BOD = "Server.Engines.BulkOrders.LargeSmithBOD" : LARGE_BOD, |w| large_smith_random(w, &LARGE_SMITH_BOD));
concrete!(/// `LargeTailorBOD`.
    LARGE_TAILOR_BOD = "Server.Engines.BulkOrders.LargeTailorBOD" : LARGE_BOD, |w| large_tailor_random(w, &LARGE_TAILOR_BOD));

/// `SmallBOD.CreateRandomFor(from)` of the vendors (kind 0 smith, 1 tailor).
fn vendor_create_small(w: &mut World, kind: i32, from: MobileId) -> Option<ItemId> {
    create_small_for(w, from, kind == BOD_SMITH)
}

/// Sets what the vendors wait for: the random deed and the accept gump.
pub fn configure(w: &mut World) {
    let h = w.resources.get_mut::<muo_content::vendor::VendorHooks>();
    h.create_small_bod = Some(vendor_create_small);
    h.send_bod_accept_gump = Some(gumps::send_accept_gump);
    h.bod_turn_in = Some(rewards::vendor_turn_in);
    book::configure(w);
}

pub fn register(types: &mut TypeRegistry) {
    for t in [
        &SMALL_SMITH_BOD,
        &SMALL_TAILOR_BOD,
        &LARGE_SMITH_BOD,
        &LARGE_TAILOR_BOD,
    ] {
        types.register_item(t, &[]);
    }
    book::register(types);
    types.register_phase_default(
        muo_server::type_registry::Phase::Configure,
        "Server.Engines.BulkOrders.Configure",
        configure,
    );
}
