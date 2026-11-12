//! `CraftItem`: what making one item costs and does (`ConsumeRes`, the success and
//! exceptional chances, `Craft`, `CompleteCraft`, the craft timer), over the data of [`super::model`].
//!
//! `FactionImbueGump` after a faction item is made is `factions::gumps::send_imbue_gump`; the `CustomCraft`
//! classes (the tinkering traps) are `super::defs::traps` and the T2A packet menus `super::t2a`.

use std::rc::Rc;

use muo_content::craft::{self as rc, CraftResource};
use muo_content::items::equip_hooks::CraftArgs;
use muo_content::items::jewel::GemType;
use muo_content::items::{is_named, tool as tool_mod};
use muo_content::mx;
use muo_primitives::{TimeSpan, utility};
use muo_server::World;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::{ItemType, deps, ops};
use muo_server::layer::Layer;
use muo_server::skills::{self, SkillName};
use muo_server::type_registry::EntityType;

use super::model::*;
use super::state::{state, state_mut, t2a_enabled};
use super::{context_of, set_context, system};

/// `ConsumeType`.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ConsumeType {
    All,
    Half,
    None,
}

// ---- tables ----

const HEAT_SOURCES: &[i32] = &[
    0x461, 0x48E, // Sandstone oven/fireplace
    0x92B, 0x96C, // Stone oven/fireplace
    0xDE3, 0xDE9, // Campfire
    0xFAC, 0xFAC, // Firepit
    0x184A, 0x184C, // Heating stand (left)
    0x184E, 0x1850, // Heating stand (right)
    0x398C, 0x399F, // Fire field
    0x2DDB, 0x2DDC, // Elven stove
    0x19AA, 0x19BB, // Veteran Reward Brazier
    0x197A, 0x19A9, // Large Forge
    0x0FB1, 0x0FB1, // Small Forge
    0x2DD8, 0x2DD8, // Elven Forge
];

const OVENS: &[i32] = &[
    0x461, 0x46F, // Sandstone oven
    0x92B, 0x93F, // Stone oven
    0x2DDB, 0x2DDC, // Elven stove
];

const MILLS: &[i32] = &[
    0x1920, 0x1921, 0x1922, 0x1923, 0x1924, 0x1295, 0x1926, 0x1928, 0x192C, 0x192D, 0x192E, 0x129F,
    0x1930, 0x1931, 0x1932, 0x1934,
];

/// `m_TypesTable` (without the BlankMap/BlankScroll row, which depends on T2A).
const TYPES_TABLE: &[[&str; 2]] = &[
    ["Server.Items.Log", "Server.Items.Board"],
    ["Server.Items.HeartwoodLog", "Server.Items.HeartwoodBoard"],
    ["Server.Items.BloodwoodLog", "Server.Items.BloodwoodBoard"],
    ["Server.Items.FrostwoodLog", "Server.Items.FrostwoodBoard"],
    ["Server.Items.OakLog", "Server.Items.OakBoard"],
    ["Server.Items.AshLog", "Server.Items.AshBoard"],
    ["Server.Items.YewLog", "Server.Items.YewBoard"],
    ["Server.Items.Leather", "Server.Items.Hides"],
    ["Server.Items.SpinedLeather", "Server.Items.SpinedHides"],
    ["Server.Items.HornedLeather", "Server.Items.HornedHides"],
    ["Server.Items.BarbedLeather", "Server.Items.BarbedHides"],
    ["Server.Items.Cloth", "Server.Items.UncutCloth"],
    ["Server.Items.CheeseWheel", "Server.Items.CheeseWedge"],
    ["Server.Items.Pumpkin", "Server.Items.SmallPumpkin"],
    [
        "Server.Items.WoodenBowlOfPeas",
        "Server.Items.PewterBowlOfPeas",
    ],
];

const COLORED_ITEMS: &[&str] = &[
    "Server.Items.BaseWeapon",
    "Server.Items.BaseArmor",
    "Server.Items.BaseClothing",
    "Server.Items.BaseJewel",
    "Server.Items.DragonBardingDeed",
];

const COLORED_RESOURCES: &[&str] = &[
    "Server.Items.BaseIngot",
    "Server.Items.BaseOre",
    "Server.Items.BaseLeather",
    "Server.Items.BaseHides",
    "Server.Items.UncutCloth",
    "Server.Items.Cloth",
    "Server.Items.BaseGranite",
    "Server.Items.BaseScales",
];

const MARKABLE: &[&str] = &[
    "Server.Items.BaseArmor",
    "Server.Items.BaseWeapon",
    "Server.Items.BaseClothing",
    "Server.Items.BaseInstrument",
    "Server.Items.DragonBardingDeed",
    "Server.Items.BaseTool",
    "Server.Items.BaseHarvestTool",
    "Server.Items.FukiyaDarts",
    "Server.Items.Shuriken",
    "Server.Items.Spellbook",
    "Server.Items.Runebook",
    "Server.Items.BaseQuiver",
];

const NEVER_COLOR: &[&str] = &["Server.Items.OrcHelm"];

// ---- type helpers ----

/// The item class named `full_name`.
pub fn resolve(w: &World, full_name: &str) -> Option<&'static ItemType> {
    match w.types.find_type_by_full_name(full_name, false)? {
        EntityType::Item(t) => Some(t),
        EntityType::Mobile(_) => None,
    }
}

/// `type == base || type.IsSubclassOf(base)` for the class named `ty`.
pub fn derives(w: &World, ty: &str, base: &str) -> bool {
    resolve(w, ty).is_some_and(|t| is_named(t, base))
}

fn derives_any(w: &World, ty: &str, bases: &[&str]) -> bool {
    bases.iter().any(|b| derives(w, ty, b))
}

fn item_type_name(w: &World, id: ItemId) -> &'static str {
    w.item(id).map_or("", |i| i.ty.full_name)
}

fn pack(w: &World, m: MobileId) -> Option<ItemId> {
    deps::find_item_on_layer(w, m, Layer::Backpack)
}

/// `tool?.Deleted == false && tool.UsesRemaining > 0`.
pub fn tool_usable(w: &World, tool: Option<ItemId>) -> bool {
    tool.is_some_and(|t| {
        w.item(t).is_some_and(|i| !i.deleted()) && tool_mod::uses_remaining(w, t) > 0
    })
}

fn tool_deleted(w: &World, t: ItemId) -> bool {
    w.item(t).is_none_or(|i| i.deleted())
}

/// `ItemIDOf(type)`.
pub fn item_id_of(w: &mut World, ty: &'static str) -> i32 {
    if let Some(&id) = state(w).and_then(|s| s.item_ids.get(ty)) {
        return id;
    }
    let mut item_id = match ty {
        "Server.Factions.FactionExplosionTrap" => 14034,
        "Server.Factions.FactionGasTrap" => 4523,
        "Server.Factions.FactionSawTrap" => 4359,
        "Server.Factions.FactionSpikeTrap" => 4517,
        // [CraftItemID(..)] of the tinkering trap crafts
        "Server.Engines.Craft.DartTrapCraft" => 0x1BFC,
        "Server.Engines.Craft.PoisonTrapCraft" => 0x113E,
        "Server.Engines.Craft.ExplosionTrapCraft" => 0x370C,
        _ => 0,
    };
    if item_id == 0
        && let Some(id) = muo_content::factory::make_item(w, ty)
    {
        item_id = w.item(id).map_or(0, |i| i.item_id);
        if let Some(t) = w.item(id).map(|i| i.ty) {
            (t.delete)(w, id);
        }
    }
    state_mut(w).item_ids.insert(ty, item_id);
    item_id
}

/// `CraftItem.ItemId`.
pub fn craft_item_id(w: &mut World, item: &CraftItem) -> i32 {
    if item.item_id == -1 {
        item_id_of(w, item.item_type)
    } else {
        item.item_id
    }
}

/// `CraftItem.LabelNumber(type)`.
pub fn label_number(w: &mut World, ty: &'static str) -> i32 {
    let number = item_id_of(w, ty);
    number + if number >= 0x4000 { 1078872 } else { 1020000 }
}

/// `CraftRes.Name`: the given name, else the label of the item type.
pub fn res_name(w: &mut World, r: &CraftRes) -> Td {
    if r.name.is_empty() {
        Td::of_number(label_number(w, r.item_type))
    } else {
        r.name.clone()
    }
}

/// `CraftRes.SendMessage(from)`.
pub fn send_res_message(w: &mut World, from: MobileId, r: &CraftRes) {
    send_td_or(w, from, &r.message, 502925); // You don't have the resources required to make that item.
}

/// A message number, else its string, else `fallback`.
fn send_td_or(w: &mut World, from: MobileId, td: &Td, fallback: i32) {
    if td.number > 0 {
        deps::send_localized_message(w, from, td.number);
    } else if td.string.as_deref().is_some_and(|s| !s.is_empty()) {
        deps::send_message(w, from, td.string.as_deref().unwrap_or(""));
    } else {
        deps::send_localized_message(w, from, fallback);
    }
}

/// `SendMessage(TextDefinition)` for a message that may be absent.
fn send_td(w: &mut World, from: MobileId, td: &Td) {
    if td.number > 0 {
        deps::send_localized_message(w, from, td.number);
    } else if let Some(s) = td.string.as_deref().filter(|s| !s.is_empty()) {
        deps::send_message(w, from, s);
    }
}

impl CraftItem {
    /// `IsMarkable(type)`.
    pub fn is_markable(&self, w: &World, ty: &str) -> bool {
        if self.force_non_exceptional {
            return false;
        }
        derives_any(w, ty, MARKABLE)
    }
}

/// `CraftItem.RetainsColor(type)`.
pub fn retains_color(w: &World, ty: &str) -> bool {
    if derives_any(w, ty, NEVER_COLOR) {
        return false;
    }
    derives_any(w, ty, COLORED_ITEMS)
}

/// `RetainsColorFrom(system, type)`.
pub fn retains_color_from(w: &World, sys: &CraftSystem, item: &CraftItem, ty: &str) -> bool {
    if (sys.vt.retains_color_from)(w, item, ty) {
        return true;
    }
    if !retains_color(w, item.item_type) {
        return false;
    }
    derives_any(w, ty, COLORED_RESOURCES)
}

/// `Find(from, itemIDs)`: an item or a static/multi tile of one of the id ranges within 2 tiles.
pub fn find_near(w: &mut World, from: MobileId, ids: &[i32]) -> bool {
    let Some(map) = mx::map(w, from) else {
        return false;
    };
    let (loc, fz) = {
        let l = mx::location(w, from);
        (l, l.z)
    };
    for it in muo_server::mobile::part2::enumerators::get_items_in_range(w, from, 2) {
        let Some(item) = w.item(it) else { continue };
        // The height test (`item.Z + 16 > item.Z`) is always true.
        if find_id(item.item_id, ids) {
            return true;
        }
    }
    for x in -2..=2 {
        for y in -2..=2 {
            for tile in muo_server::map::tiles::static_and_multi_tiles(w, map, loc.x + x, loc.y + y)
            {
                let (z, id) = (i32::from(tile.z), i32::from(tile.id));
                if z + 16 > fz && fz + 16 > z && find_id(id, ids) {
                    return true;
                }
            }
        }
    }
    false
}

/// `Find(itemID, itemIDs)`: ranges in pairs.
pub fn find_id(item_id: i32, ids: &[i32]) -> bool {
    ids.chunks(2)
        .any(|p| p.len() == 2 && item_id >= p[0] && item_id <= p[1])
}

/// `GetEquivalentTypes(type)`.
pub fn equivalent_types(w: &World, base: &str) -> Option<Vec<String>> {
    if let Some(row) = TYPES_TABLE.iter().find(|r| r[0] == base) {
        return Some(row.iter().map(|s| (*s).to_string()).collect());
    }
    if !t2a_enabled(w) && base == "Server.Items.BlankMap" {
        return Some(vec![
            "Server.Items.BlankMap".into(),
            "Server.Items.BlankScroll".into(),
        ]);
    }
    None
}

fn types_of(w: &World, names: &[String]) -> Vec<&'static ItemType> {
    names.iter().filter_map(|n| resolve(w, n)).collect()
}

fn is_quantity_type(w: &World, types: &[Vec<&'static ItemType>]) -> bool {
    let Some(q) = state(w).and_then(|s| s.quantity) else {
        return false;
    };
    types
        .iter()
        .any(|row| row.iter().any(|t| (q.type_is_quantity)(t)))
}

/// `ConsumeAttributes(from, ref message, consume)`.
pub fn consume_attributes(
    w: &mut World,
    from: MobileId,
    item: &CraftItem,
    message: &mut Option<Td>,
    consume: bool,
) -> bool {
    let (hits, mana, stam) = match w.mobile(from) {
        Some(m) => (m.hits, m.mana, m.stam),
        None => return false,
    };
    if item.hits > 0 && hits < item.hits {
        *message = Some(item.need_hits_label.clone());
        return false;
    }
    if item.mana > 0 && mana < item.mana {
        *message = Some(item.need_mana_label.clone());
        return false;
    }
    if item.stam > 0 && stam < item.stam {
        *message = Some(item.need_stam_label.clone());
        return false;
    }
    if consume {
        use muo_server::mobile::part1::props_b;
        props_b::set_mana(w, from, mana - item.mana);
        props_b::set_hits(w, from, hits - item.hits);
        props_b::set_stam(w, from, stam - item.stam);
    }
    true
}

/// `ConsumeRes(from, typeRes, craftSystem, ref resHue, ref maxAmount, consumeType, ref message, isFailure)`.
#[allow(clippy::too_many_arguments)]
pub fn consume_res(
    w: &mut World,
    from: MobileId,
    sys: &Rc<CraftSystem>,
    item: &CraftItem,
    type_res: Option<&'static str>,
    res_hue: &mut i32,
    max_amount: &mut i32,
    consume_type: ConsumeType,
    message: &mut Option<Td>,
    is_failure: bool,
) -> bool {
    let Some(our_pack) = pack(w, from) else {
        return false;
    };

    if item.need_heat && !find_near(w, from, HEAT_SOURCES) {
        *message = Some(Td::of_number(1044487)); // You must be near a fire source to cook.
        return false;
    }
    if item.need_oven && !find_near(w, from, OVENS) {
        *message = Some(Td::of_number(1044493)); // You must be near an oven to bake that.
        return false;
    }
    if item.need_mill && !find_near(w, from, MILLS) {
        *message = Some(Td::of_number(1044491)); // You must be near a flour mill to do that.
        return false;
    }

    let n = item.resources.len();
    let mut type_names: Vec<Vec<String>> = Vec::with_capacity(n);
    let mut amounts = vec![0i32; n];
    *max_amount = i32::MAX;

    let res_col = if item.use_sub_res2 {
        &sys.sub_res2
    } else {
        &sys.sub_res
    };

    #[allow(clippy::needless_range_loop)] // `i` also indexes type_names and the type lists
    for i in 0..n {
        let craft_res = &item.resources[i];
        let mut base_type = craft_res.item_type;

        // Resource Mutation
        if base_type == res_col.res_type
            && let Some(tr) = type_res
        {
            base_type = tr;
            if let Some(sub) = res_col.search_for(base_type)
                && skills::base(w, from, sys.main_skill) < sub.required_skill
            {
                *message = Some(sub.message.clone());
                return false;
            }
        }

        let names = equivalent_types(w, base_type).unwrap_or_else(|| vec![base_type.to_string()]);
        let types = types_of(w, &names);
        amounts[i] = craft_res.amount;

        // For stackable items that can be crafted more than one at a time
        if item.use_all_res {
            let temp = muo_server::item::container::get_amount_types(w, our_pack, &types, true)
                / amounts[i];
            if temp < *max_amount {
                *max_amount = temp;
                if *max_amount == 0 {
                    let r = &item.resources[i];
                    *message = Some(if r.message.number > 0 {
                        Td::of_number(r.message.number)
                    } else if r.message.string.as_deref().is_some_and(|s| !s.is_empty()) {
                        r.message.clone()
                    } else {
                        Td::of_number(502925) // You don't have the resources required to make that item.
                    });
                    return false;
                }
            }
        }

        let first = names.first().map(String::as_str).unwrap_or("");
        if is_failure && !(sys.vt.consume_on_failure)(w, from, first, item) {
            amounts[i] = 0;
        } else if is_failure && !muo_content::items::core_uotd(w) {
            amounts[i] -= amounts[i] / 2;
        }
        type_names.push(names);
    }

    // We adjust the amount of each resource to consume the max possible
    if item.use_all_res {
        for a in amounts.iter_mut() {
            *a *= *max_amount;
        }
    } else {
        *max_amount = -1;
    }

    let mut consume_extra: Option<ItemId> = None;
    if item.name.number == 1041267 {
        // Runebooks are a special case, they need a blank recall rune
        let runes = muo_server::item::enumerable::find_items(w, our_pack, true, |w, i| {
            muo_content::factory::item_is(w, i, "Server.Items.RecallRune")
        });
        for rune in runes {
            if !rune_marked(w, rune) {
                consume_extra = Some(rune);
                break;
            }
        }
        if consume_extra.is_none() {
            *message = Some(Td::of_number(1044253)); // You don't have the components needed to make that.
            return false;
        }
    }

    let type_lists: Vec<Vec<&'static ItemType>> =
        type_names.iter().map(|n| types_of(w, n)).collect();
    let quantity = is_quantity_type(w, &type_lists);
    let index: i32;

    if consume_type == ConsumeType::None {
        let mut idx = -1;
        for i in 0..type_lists.len() {
            let q = if quantity {
                get_quantity(w, our_pack, &type_lists[i], item.required_beverage)
            } else {
                let grouper = |w: &World, a: ItemId, b: ItemId| check_hue_grouping(w, a, b);
                muo_server::item::container::get_best_group_amount(
                    w,
                    our_pack,
                    &type_lists[i],
                    true,
                    &grouper,
                )
            };
            if q < amounts[i] {
                idx = i as i32;
                break;
            }
        }
        index = idx;
    } else {
        if consume_type == ConsumeType::Half {
            for a in amounts.iter_mut() {
                *a = std::cmp::max(1, *a / 2);
            }
        }
        let mut hue = 0;
        let mut hue_amount = 0;
        index = if quantity {
            consume_quantity(w, our_pack, &type_lists, &amounts, item.required_beverage)
        } else {
            let slices: Vec<&[&'static ItemType]> = type_lists.iter().map(Vec::as_slice).collect();
            let grouper = |w: &World, a: ItemId, b: ItemId| check_hue_grouping(w, a, b);
            let sys2 = sys.clone();
            let item2 = item.clone();
            let mut cb = |w: &mut World, it: ItemId, amount: i32| {
                let Some(name) = w.item(it).map(|i| i.ty.full_name) else {
                    return;
                };
                if !retains_color_from(w, &sys2, &item2, name) {
                    return;
                }
                if amount >= hue_amount {
                    hue = w.item(it).map_or(0, |i| i.hue);
                    hue_amount = amount;
                }
            };
            muo_server::item::container::consume_total_grouped_any(
                w,
                our_pack,
                &slices,
                &amounts,
                true,
                Some(&mut cb),
                &grouper,
            )
        };
        *res_hue = hue;
    }

    if index == -1 {
        if consume_type != ConsumeType::None
            && let Some(extra) = consume_extra
            && let Some(t) = w.item(extra).map(|i| i.ty)
        {
            (t.delete)(w, extra);
        }
        return true;
    }

    let res = &item.resources[index as usize];
    *message = Some(if res.message.number > 0 {
        Td::of_number(res.message.number)
    } else if res.message.string.as_deref().is_some_and(|s| !s.is_empty()) {
        res.message.clone()
    } else {
        Td::of_number(502925) // You don't have the resources required to make that item.
    });
    false
}

fn rune_marked(w: &World, rune: ItemId) -> bool {
    w.resources
        .get::<super::state::RuneApi>()
        .is_some_and(|api| (api.marked)(w, rune))
}

/// `CheckHueGrouping(a, b)`: `b.Hue.CompareTo(a.Hue)`.
fn check_hue_grouping(w: &World, a: ItemId, b: ItemId) -> i32 {
    let (ha, hb) = (
        w.item(a).map_or(0, |i| i.hue),
        w.item(b).map_or(0, |i| i.hue),
    );
    match hb.cmp(&ha) {
        std::cmp::Ordering::Less => -1,
        std::cmp::Ordering::Equal => 0,
        std::cmp::Ordering::Greater => 1,
    }
}

fn get_quantity(
    w: &World,
    cont: ItemId,
    types: &[&'static ItemType],
    beverage: BeverageType,
) -> i32 {
    let Some(q) = state(w).and_then(|s| s.quantity) else {
        return 0;
    };
    let mut amount = 0;
    for it in muo_server::item::enumerable::find_items(w, cont, true, |_, _| true) {
        if !w.item(it).is_some_and(|i| types.iter().any(|t| i.ty.is(t))) {
            continue;
        }
        match (q.quantity)(w, it) {
            None => amount += w.item(it).map_or(0, |i| i.amount),
            Some((qty, content)) => {
                if content.is_none_or(|c| c == beverage) {
                    amount += qty;
                }
            }
        }
    }
    amount
}

/// `ConsumeQuantity(cont, types, amounts)`: the index of the first type short of its amount, else -1.
fn consume_quantity(
    w: &mut World,
    cont: ItemId,
    types: &[Vec<&'static ItemType>],
    amounts: &[i32],
    beverage: BeverageType,
) -> i32 {
    let Some(q) = state(w).and_then(|s| s.quantity) else {
        return 0;
    };
    let mut lists: Vec<Vec<ItemId>> = Vec::new();
    for (i, row) in types.iter().enumerate() {
        let mut total = 0;
        let mut list = Vec::new();
        for it in muo_server::item::enumerable::find_items(w, cont, true, |_, _| true) {
            if !w.item(it).is_some_and(|x| row.iter().any(|t| x.ty.is(t))) {
                continue;
            }
            match (q.quantity)(w, it) {
                None => {
                    total += w.item(it).map_or(0, |x| x.amount);
                    list.push(it);
                }
                Some((qty, content)) => {
                    if content.is_none_or(|c| c == beverage) {
                        total += qty;
                        list.push(it);
                    }
                }
            }
        }
        if total < amounts[i] {
            return i as i32;
        }
        lists.push(list);
    }
    for (i, list) in lists.iter().enumerate() {
        let mut need = amounts[i];
        for &it in list {
            match (q.quantity)(w, it) {
                None => {
                    let theirs = w.item(it).map_or(0, |x| x.amount);
                    if theirs < need {
                        muo_server::item::base::consume(w, it, theirs);
                        need -= theirs;
                    } else {
                        muo_server::item::base::consume(w, it, need);
                        break;
                    }
                }
                Some((qty, _)) => {
                    if qty < need {
                        (q.set_quantity)(w, it, 0);
                        need -= qty;
                    } else {
                        (q.set_quantity)(w, it, qty - need);
                        break;
                    }
                }
            }
        }
    }
    -1
}

// ---- chances ----

/// `GetExceptionalChance(system, chance, from)`.
pub fn exceptional_chance(
    w: &mut World,
    sys: &CraftSystem,
    item: &CraftItem,
    mut chance: f64,
    from: MobileId,
) -> f64 {
    if item.force_non_exceptional {
        return 0.0;
    }
    let mut bonus = 0.0;
    if let Some((skill, success, exceptional)) = super::state::talisman(w, from)
        && skill == sys.main_skill
    {
        chance -= f64::from(success) / 100.0;
        bonus = f64::from(exceptional) / 100.0;
    }
    chance = match sys.eca {
        CraftEca::FiftyPercentChanceMinusTenPercent => chance * 0.5 - 0.1,
        CraftEca::ChanceMinusSixtyToFortyFive => {
            let v = skills::value(w, from, sys.main_skill);
            chance - (0.60 - (v - 95.0) * 0.03).clamp(0.45, 0.60)
        }
        CraftEca::ChanceMinusSixty => chance - 0.6,
    };
    if chance > 0.0 { chance + bonus } else { chance }
}

/// `GetSuccessChance(from, typeRes, craftSystem, gainSkills, out allRequiredSkills)`.
pub fn success_chance(
    w: &mut World,
    sys: &CraftSystem,
    item: &CraftItem,
    from: MobileId,
    gain_skills: bool,
    all_required: &mut bool,
) -> f64 {
    let (mut min_main, mut max_main, mut val_main) = (0.0, 0.0, 0.0);
    *all_required = true;
    for cs in &item.skills {
        let val = skills::value(w, from, cs.skill);
        if val < cs.min_skill {
            *all_required = false;
        }
        if cs.skill == sys.main_skill {
            min_main = cs.min_skill;
            max_main = cs.max_skill;
            val_main = val;
        }
        if gain_skills {
            // This is a passive check. Success chance is entirely dependant on the main skill
            mx::check_skill(w, from, cs.skill, cs.min_skill, cs.max_skill);
        }
    }
    if !*all_required {
        return 0.0;
    }
    let min_chance = (sys.vt.chance_at_min)(item);
    let mut chance =
        min_chance + (val_main - min_main) / (max_main - min_main) * (1.0 - min_chance);
    if let Some((skill, success, _)) = super::state::talisman(w, from)
        && skill == sys.main_skill
    {
        chance += f64::from(success) / 100.0;
    }
    chance
}

/// `CheckSkills(from, typeRes, craftSystem, ref quality, out allRequiredSkills, gainSkills)`.
pub fn check_skills(
    w: &mut World,
    sys: &CraftSystem,
    item: &CraftItem,
    from: MobileId,
    quality: &mut i32,
    all_required: &mut bool,
    gain_skills: bool,
) -> bool {
    let chance = success_chance(w, sys, item, from, gain_skills, all_required);
    let ex = exceptional_chance(w, sys, item, chance, from);
    if ex > utility::random_double(&mut w.rng) {
        *quality = 2;
    }
    chance > utility::random_double(&mut w.rng)
}

// ---- Craft ----

fn required_expansion_message(w: &World, ex: Expansion) -> Td {
    match ex {
        Expansion::SE => Td::of_number(1063307), // The "Samurai Empire" expansion is required to attempt this item.
        Expansion::ML => Td::of_number(1072650), // The "Mondain's Legacy" expansion is required to attempt this item.
        _ => {
            let name = w
                .expansion_table()
                .map(|t| t.get_info(ex as i32).name.clone())
                .unwrap_or_else(|| format!("{ex:?}"));
            Td::of_text(format!(
                "The \"{name}\" expansion is required to attempt this item."
            ))
        }
    }
}

/// `from.NetState?.SupportsExpansion(ex) == true`.
pub fn supports_expansion(w: &World, from: MobileId, ex: Expansion) -> bool {
    let Some(ns) = w.mobile(from).and_then(|m| m.net_state) else {
        return false;
    };
    let (Some(n), Some(table)) = (w.net_state(ns), w.expansion_table()) else {
        return false;
    };
    n.caps
        .supports_expansion(table, table.get_info(ex as i32), w.expansion() as i32, true)
}

fn has_recipe_failure(w: &World, from: MobileId, recipe: Option<i32>) -> bool {
    // `(from as PlayerMobile)?.HasRecipe(Recipe) == false`
    match recipe {
        Some(r)
            if w.mobile(from)
                .is_some_and(|m| m.ty.is(&muo_content::player_mobile::PLAYER_MOBILE)) =>
        {
            !muo_content::player_mobile::has_recipe(w, from, r)
        }
        _ => false,
    }
}

/// `CraftItem.Craft(from, craftSystem, typeRes, tool[, resHue])`: the checks, then the craft timer.
pub fn craft(
    w: &mut World,
    from: MobileId,
    sys: &Rc<CraftSystem>,
    index: usize,
    type_res: Option<&'static str>,
    tool: Option<ItemId>,
    hue: Option<i32>,
) {
    let item = sys.items[index].clone();
    let hued = hue.is_some();
    if !muo_server::mobile::part1::combat::begin_action(w, from, "CraftSystem") {
        deps::send_localized_message(w, from, 500119); // You must wait to perform another action
        return;
    }

    if item.required_expansion != Expansion::None
        && !supports_expansion(w, from, item.required_expansion)
    {
        muo_server::mobile::part1::combat::end_action(w, from, "CraftSystem");
        let msg = required_expansion_message(w, item.required_expansion);
        super::gumps::send_craft_gump(w, from, sys, tool, Some(msg), super::gumps::CraftPage::None);
        return;
    }

    let mut all_required = true;
    let chance = success_chance(w, sys, &item, from, false, &mut all_required);
    if !all_required || chance <= 0.0 {
        muo_server::mobile::part1::combat::end_action(w, from, "CraftSystem");
        if t2a_enabled(w) || hued {
            muo_server::mobile::part2::messages::send_ascii_message(
                w,
                from,
                "You lack the required skill to craft this item.",
            );
        } else {
            let msg = Some(Td::of_number(1044153)); // You don't have the required skills to attempt this item.
            super::gumps::send_craft_gump(w, from, sys, tool, msg, super::gumps::CraftPage::None);
        }
        return;
    }

    if has_recipe_failure(w, from, item.recipe) {
        muo_server::mobile::part1::combat::end_action(w, from, "CraftSystem");
        let msg = Some(Td::of_number(1072847)); // You must learn that recipe from a scroll.
        super::gumps::send_craft_gump(w, from, sys, tool, msg, super::gumps::CraftPage::None);
        return;
    }

    let bad = (sys.vt.can_craft)(w, from, tool, Some(item.item_type));
    if bad > 0 {
        muo_server::mobile::part1::combat::end_action(w, from, "CraftSystem");
        show_craft_menu(w, from, sys, tool, Some(Td::of_number(bad)));
        return;
    }

    let (mut res_hue, mut max_amount) = (0, 0);
    let mut message = None;
    if let Some(target_hue) = hue {
        // Dry run: check hued resources are available
        if !check_hued_res(w, from, sys, &item, type_res, target_hue) {
            muo_server::mobile::part1::combat::end_action(w, from, "CraftSystem");
            deps::send_localized_message(w, from, 502925); // You don't have the resources required to make that item.
            return;
        }
    } else if !consume_res(
        w,
        from,
        sys,
        &item,
        type_res,
        &mut res_hue,
        &mut max_amount,
        ConsumeType::None,
        &mut message,
        false,
    ) {
        muo_server::mobile::part1::combat::end_action(w, from, "CraftSystem");
        show_craft_menu(w, from, sys, tool, message);
        return;
    }

    let mut message = None;
    if !consume_attributes(w, from, &item, &mut message, false) {
        muo_server::mobile::part1::combat::end_action(w, from, "CraftSystem");
        show_craft_menu(w, from, sys, tool, message);
        return;
    }

    if let Some(c) = context_of(w, sys.name, from) {
        let mut c = c;
        c.on_made(index);
        set_context(w, sys.name, from, c);
    }

    let i_min = sys.min_craft_effect;
    let i_max = sys.max_craft_effect - i_min + 1;
    let mut i_random = utility::random(&mut w.rng, i_max);
    i_random += i_min + 1;

    start_timer(w, from, sys.clone(), index, type_res, tool, i_random, hue);
}

#[allow(clippy::too_many_arguments)]
fn start_timer(
    w: &mut World,
    from: MobileId,
    sys: Rc<CraftSystem>,
    index: usize,
    type_res: Option<&'static str>,
    tool: Option<ItemId>,
    count_max: i32,
    res_hue: Option<i32>,
) {
    let mut count = 0;
    muo_server::timer::start(
        w,
        TimeSpan::ZERO,
        TimeSpan::from_secs_f64(sys.delay),
        count_max,
        move |w| {
            count += 1;
            muo_server::mobile::base::disruptive_action(w, from);
            if count < count_max {
                (sys.vt.play_craft_effect)(w, from);
                return;
            }
            timer_done(w, from, &sys, index, type_res, tool, res_hue);
        },
    );
}

fn timer_done(
    w: &mut World,
    from: MobileId,
    sys: &Rc<CraftSystem>,
    index: usize,
    type_res: Option<&'static str>,
    tool: Option<ItemId>,
    res_hue: Option<i32>,
) {
    muo_server::mobile::part1::combat::end_action(w, from, "CraftSystem");
    let item = sys.items[index].clone();

    let bad = (sys.vt.can_craft)(w, from, tool, Some(item.item_type));
    if bad > 0 {
        if tool_usable(w, tool) {
            show_craft_menu(w, from, sys, tool, Some(Td::of_number(bad)));
        } else {
            deps::send_localized_message(w, from, bad);
        }
        return;
    }

    let mut quality = 1;
    let mut ignored = true;
    check_skills(w, sys, &item, from, &mut quality, &mut ignored, false);

    let Some(context) = context_of(w, sys.name, from) else {
        return;
    };

    if let Some(trap) = super::defs::traps::custom_craft(item.item_type) {
        super::defs::traps::end_craft_action(w, from, sys, index, trap, type_res, tool, quality);
        return;
    }

    let mut makers_mark = false;
    if quality == 2 && skills::base(w, from, sys.main_skill) >= 100.0 {
        makers_mark = item.is_markable(w, item.item_type);
    }

    // T2A menus always prompt for maker's mark (no auto-mark/don't-mark options)
    if makers_mark && (t2a_enabled(w) || context.mark_option == CraftMarkOption::PromptForMark) {
        super::gumps::send_query_makers_mark(w, from, sys, index, quality, type_res, tool, res_hue);
    } else {
        if context.mark_option == CraftMarkOption::DoNotMark {
            makers_mark = false;
        }
        complete_craft(
            w,
            from,
            sys,
            index,
            quality,
            makers_mark,
            type_res,
            tool,
            res_hue,
            None,
        );
    }
}

// ---- hued crafting (tailoring cloth) ----

fn is_hued_match(
    w: &World,
    it: ItemId,
    type_name: &str,
    equivalents: &Option<Vec<String>>,
    hue: i32,
) -> bool {
    let Some(i) = w.item(it) else { return false };
    if i.hue != hue {
        return false;
    }
    match equivalents {
        None => is_named(i.ty, type_name),
        Some(list) => list.iter().any(|e| is_named(i.ty, e)),
    }
}

/// `GetHuedAmount(pack, type, hue)`.
pub fn hued_amount(w: &World, pack: ItemId, type_name: &str, hue: i32) -> i32 {
    let eq = equivalent_types(w, type_name);
    muo_server::item::enumerable::find_items(w, pack, true, |w, i| {
        is_hued_match(w, i, type_name, &eq, hue)
    })
    .into_iter()
    .map(|i| w.item(i).map_or(0, |x| x.amount))
    .sum()
}

fn consume_hued_amount(
    w: &mut World,
    pack: ItemId,
    type_name: &str,
    hue: i32,
    amount: i32,
) -> bool {
    let eq = equivalent_types(w, type_name);
    let mut remaining = amount;
    let mut to_delete = Vec::new();
    let mut reduce = None;
    for it in muo_server::item::enumerable::find_items(w, pack, true, |w, i| {
        is_hued_match(w, i, type_name, &eq, hue)
    }) {
        if remaining <= 0 {
            break;
        }
        let amt = w.item(it).map_or(0, |x| x.amount);
        if amt <= remaining {
            remaining -= amt;
            to_delete.push(it);
        } else {
            reduce = Some((it, amt - remaining));
            remaining = 0;
        }
    }
    if remaining > 0 {
        return false;
    }
    if let Some((it, left)) = reduce {
        ops::set_amount(w, it, left);
    }
    for it in to_delete {
        if let Some(t) = w.item(it).map(|i| i.ty) {
            (t.delete)(w, it);
        }
    }
    true
}

fn check_hued_res(
    w: &mut World,
    from: MobileId,
    sys: &CraftSystem,
    item: &CraftItem,
    type_res: Option<&'static str>,
    target_hue: i32,
) -> bool {
    let Some(our_pack) = pack(w, from) else {
        return false;
    };
    let res_col = if item.use_sub_res2 {
        &sys.sub_res2
    } else {
        &sys.sub_res
    };
    for (i, craft_res) in item.resources.iter().enumerate() {
        let mut base_type = craft_res.item_type;
        if base_type == res_col.res_type
            && let Some(tr) = type_res
        {
            base_type = tr;
        }
        if target_hue >= 0 && i == 0 {
            if hued_amount(w, our_pack, base_type, target_hue) < craft_res.amount {
                return false;
            }
        } else {
            let amount = resolve(w, base_type).map_or(0, |t| {
                muo_server::item::container::get_amount(w, our_pack, t, true)
            });
            if amount < craft_res.amount {
                return false;
            }
        }
    }
    true
}

/// `ConsumeHuedRes(from, typeRes, craftSystem, targetHue, ref resHue, consumeType)`.
#[allow(clippy::too_many_arguments)]
fn consume_hued_res(
    w: &mut World,
    from: MobileId,
    sys: &CraftSystem,
    item: &CraftItem,
    type_res: Option<&'static str>,
    target_hue: i32,
    res_hue: &mut i32,
    consume_type: ConsumeType,
) -> bool {
    let Some(our_pack) = pack(w, from) else {
        return false;
    };
    if item.need_heat && !find_near(w, from, HEAT_SOURCES) {
        return false;
    }
    if item.need_oven && !find_near(w, from, OVENS) {
        return false;
    }
    if item.need_mill && !find_near(w, from, MILLS) {
        return false;
    }
    let res_col = if item.use_sub_res2 {
        &sys.sub_res2
    } else {
        &sys.sub_res
    };
    for (i, craft_res) in item.resources.iter().enumerate() {
        let mut base_type = craft_res.item_type;
        let mut amount = craft_res.amount;
        if base_type == res_col.res_type
            && let Some(tr) = type_res
        {
            base_type = tr;
            if let Some(sub) = res_col.search_for(base_type)
                && skills::base(w, from, sys.main_skill) < sub.required_skill
            {
                return false;
            }
        }
        if consume_type == ConsumeType::Half {
            amount = std::cmp::max(1, amount / 2);
        }
        let ty = resolve(w, base_type);
        if target_hue >= 0 && i == 0 {
            if consume_type == ConsumeType::None {
                if hued_amount(w, our_pack, base_type, target_hue) < amount {
                    return false;
                }
            } else if !consume_hued_amount(w, our_pack, base_type, target_hue, amount) {
                return false;
            }
        } else if consume_type == ConsumeType::None {
            if ty.map_or(0, |t| {
                muo_server::item::container::get_amount(w, our_pack, t, true)
            }) < amount
            {
                return false;
            }
        } else if !ty.is_some_and(|t| {
            muo_server::item::container::consume_total(w, our_pack, t, amount, true, None)
        }) {
            return false;
        }
    }
    *res_hue = target_hue;
    true
}

// ---- CompleteCraft ----

/// `ShowCraftMenu(from, system, tool, message)`: the gump, or (T2A) just the message.
pub fn show_craft_menu(
    w: &mut World,
    from: MobileId,
    sys: &Rc<CraftSystem>,
    tool: Option<ItemId>,
    message: Option<Td>,
) {
    if t2a_enabled(w) {
        // T2A: Don't reopen menu. Player double-clicks tool to restart.
        if let Some(m) = &message {
            send_td(w, from, m);
        }
        return;
    }
    super::gumps::send_craft_gump(w, from, sys, tool, message, super::gumps::CraftPage::None);
}

/// Spends one use of the tool; deletes it when it ran out and breaks on depletion. Returns "tool broken".
fn use_tool(w: &mut World, tool: Option<ItemId>, smithy: bool, from: MobileId) -> bool {
    let Some(t) = tool else { return false };
    let uses = tool_mod::uses_remaining(w, t) - 1;
    tool_mod::set_uses_remaining(w, t, uses);

    if smithy
        && let Some(hammer) = deps::find_item_on_layer(w, from, Layer::OneHanded)
        && hammer != t
        && muo_content::factory::item_is(w, hammer, "Server.Items.AncientSmithyHammer")
    {
        let left = tool_mod::uses_remaining(w, hammer) - 1;
        tool_mod::set_uses_remaining(w, hammer, left);
        if left < 1
            && let Some(ty) = w.item(hammer).map(|i| i.ty)
        {
            (ty.delete)(w, hammer);
        }
    }

    let broken = tool_mod::uses_remaining(w, t) < 1 && tool_mod::class(w, t).break_on_depletion;
    if broken && let Some(ty) = w.item(t).map(|i| i.ty) {
        (ty.delete)(w, t);
    }
    broken
}

/// What `CompleteCraft` needs to say after the item was made or failed.
/// The `queryFactionImbue` block of `CompleteCraft`: `(faction, available silver, silver cost)` when `from` is in a
/// faction that owns the town they stand in, has the silver, and a vendor of the item's kind is within 12.
fn faction_imbue_offer(w: &mut World, from: MobileId, item: ItemId) -> Option<(usize, i32, i32)> {
    use crate::factions::{self, town};
    let def = factions::identify_item(w, item)?;
    let f = factions::find(w, from)?;
    let t = town::from_region(w, muo_server::mobile::part1::props_b::region(w, from))?;
    if town::owner(w, t) != Some(f) {
        return None;
    }
    let pack = deps::find_item_on_layer(w, from, Layer::Backpack)?;
    let silver =
        muo_server::item::container::get_amount(w, pack, &crate::faction_items::SILVER, true);
    if silver < def.silver_cost {
        return None;
    }
    let near = muo_server::mobile::part2::enumerators::get_mobiles_in_range(w, from, 12)
        .into_iter()
        .any(|m| {
            w.mobile(m)
                .is_some_and(|x| is_named_mobile(x.ty, def.vendor_type))
        });
    near.then_some((f, silver, def.silver_cost))
}

fn is_named_mobile(ty: &'static muo_server::mobile::MobileType, name: &str) -> bool {
    let mut t = Some(ty);
    while let Some(c) = t {
        if c.full_name == name {
            return true;
        }
        t = c.parent;
    }
    false
}

fn after_message(
    w: &mut World,
    from: MobileId,
    sys: &Rc<CraftSystem>,
    tool: Option<ItemId>,
    num: i32,
) {
    if tool_usable(w, tool) {
        if t2a_enabled(w) {
            if num > 0 {
                deps::send_localized_message(w, from, num);
            }
            show_craft_menu(w, from, sys, tool, None);
        } else {
            show_craft_menu(w, from, sys, tool, Some(Td::of_number(num)));
        }
    } else if num > 0 {
        deps::send_localized_message(w, from, num);
    }
}

fn send_failed_td(
    w: &mut World,
    from: MobileId,
    sys: &Rc<CraftSystem>,
    tool: Option<ItemId>,
    message: Option<Td>,
) {
    if tool_usable(w, tool) {
        show_craft_menu(w, from, sys, tool, message);
    } else if let Some(m) = message {
        send_td(w, from, &m);
    }
}

/// `CompleteCraft(quality, makersMark, from, craftSystem, typeRes, tool, customCraft[, targetHue])`.
#[allow(clippy::too_many_arguments)]
pub fn complete_craft(
    w: &mut World,
    from: MobileId,
    sys: &Rc<CraftSystem>,
    index: usize,
    quality: i32,
    makers_mark: bool,
    type_res: Option<&'static str>,
    tool: Option<ItemId>,
    target_hue: Option<i32>,
    custom: Option<&super::defs::traps::TrapCraft>,
) {
    let item = sys.items[index].clone();
    let bad = (sys.vt.can_craft)(w, from, tool, Some(item.item_type));
    if bad > 0 {
        if tool_usable(w, tool) {
            show_craft_menu(w, from, sys, tool, Some(Td::of_number(bad)));
        } else {
            deps::send_localized_message(w, from, bad);
        }
        return;
    }

    // The timer's resHue is -1 for an unhued craft: only a non-negative one takes the hued path.
    if let Some(th) = target_hue.filter(|h| *h >= 0) {
        complete_craft_hued(
            w,
            from,
            sys,
            &item,
            index,
            quality,
            makers_mark,
            type_res,
            tool,
            th,
        );
        return;
    }

    let (mut check_hue, mut check_max) = (0, 0);
    let mut check_message = None;
    let consume_ok = consume_res(
        w,
        from,
        sys,
        &item,
        type_res,
        &mut check_hue,
        &mut check_max,
        ConsumeType::None,
        &mut check_message,
        false,
    );
    // Not enough resource to craft it
    if !(consume_ok && consume_attributes(w, from, &item, &mut check_message, false)) {
        if tool_usable(w, tool) {
            show_craft_menu(w, from, sys, tool, check_message);
        } else if let Some(m) = check_message {
            send_td(w, from, &m);
        }
        return;
    }

    let mut tool_broken = false;
    let mut ignored = 1;
    let mut endquality = 1;
    let (mut res_hue, mut max_amount) = (0, 0);
    let mut message = None;
    let mut num = 0;
    let mut all_required = true;

    if check_skills(w, sys, &item, from, &mut ignored, &mut all_required, true) {
        let consumed = consume_res(
            w,
            from,
            sys,
            &item,
            type_res,
            &mut res_hue,
            &mut max_amount,
            ConsumeType::All,
            &mut message,
            false,
        );
        // Not enough resource to craft it
        if !(consumed && consume_attributes(w, from, &item, &mut message, true)) {
            if tool_usable(w, tool) {
                show_craft_menu(w, from, sys, tool, message);
            } else if let Some(m) = message {
                send_td(w, from, &m);
            }
            return;
        }

        tool_broken = use_tool(w, tool, sys.name == "Blacksmithy", from);

        // the item
        let created = match custom {
            Some(c) => super::defs::traps::complete(w, from, c, &mut num),
            None => make_item(w, from, &item, &mut num),
        };
        if let Some(it) = created {
            // Stamped here, not in OnCraft: most craftables do not implement ICraftable.
            if let Some(i) = w.item_mut(it) {
                i.player_constructed = true;
            }
            let args = craft_args(
                w,
                sys,
                &item,
                from,
                quality,
                makers_mark,
                type_res,
                tool,
                res_hue,
            );
            match super::on_craft(w, it, &args) {
                Some(q) => endquality = q,
                None => {
                    if w.item(it).is_some_and(|i| i.hue == 0)
                        && let Some(ty) = w.item(it).map(|i| i.ty)
                    {
                        (ty.set_hue)(w, it, res_hue);
                    }
                }
            }

            if max_amount > 0 {
                let stackable = w.item(it).is_some_and(|i| i.stackable());
                if !stackable && super::uses_remaining_api(w, it).is_some() {
                    let cur = super::uses_remaining_api(w, it).unwrap_or(0);
                    super::set_uses_remaining_api(w, it, cur * max_amount);
                } else {
                    ops::set_amount(w, it, max_amount);
                }
            }

            muo_server::mobile::part2::items::add_to_backpack(w, from, it);

            if mx::access_level(w, from) > muo_server::mobile::AccessLevel::Player {
                let text = format!(
                    "Crafting {} with craft system {}",
                    muo_content::targets::format_entity(w, EntityId::Item(it)),
                    sys.name
                );
                muo_content::targets::command_log(w, from, &text);
            }
        }

        if num == 0 {
            num = (sys.vt.play_ending_effect)(
                w,
                from,
                Ending {
                    failed: false,
                    lost_material: true,
                    tool_broken,
                    quality: endquality,
                    makers_mark,
                },
                &item,
            );
        }

        // FactionItemDefinition.Identify / FactionImbueGump for an IFactionItem
        if let Some(it) = created
            && let Some((f, silver, cost)) = faction_imbue_offer(w, from, it)
        {
            let sys2 = sys.clone();
            crate::factions::gumps::send_imbue_gump(
                w,
                from,
                f,
                it,
                quality,
                silver,
                cost,
                Box::new(move |w| after_message(w, from, &sys2, tool, num)),
            );
            return;
        }
        after_message(w, from, sys, tool, num);
        return;
    }

    if !all_required {
        if tool_usable(w, tool) {
            muo_server::mobile::part2::messages::send_ascii_message(
                w,
                from,
                "You lack the required skill to craft this item.",
            );
        } else {
            deps::send_localized_message(w, from, 1044153); // You don't have the required skills to attempt this item.
        }
        return;
    }

    let consume_type = if item.use_all_res {
        ConsumeType::Half
    } else {
        ConsumeType::All
    };
    // Not enough resource to craft it
    if !consume_res(
        w,
        from,
        sys,
        &item,
        type_res,
        &mut res_hue,
        &mut max_amount,
        consume_type,
        &mut message,
        true,
    ) {
        send_failed_td(w, from, sys, tool, message);
        return;
    }

    if let Some(t) = tool {
        let uses = tool_mod::uses_remaining(w, t) - 1;
        tool_mod::set_uses_remaining(w, t, uses);
        if uses < 1 && tool_mod::class(w, t).break_on_depletion {
            tool_broken = true;
            if let Some(ty) = w.item(t).map(|i| i.ty) {
                (ty.delete)(w, t);
            }
        }
    }

    // SkillCheck failed.
    num = (sys.vt.play_ending_effect)(
        w,
        from,
        Ending {
            failed: true,
            lost_material: true,
            tool_broken,
            quality: endquality,
            makers_mark: false,
        },
        &item,
    );
    if tool_usable(w, tool) {
        show_craft_menu(w, from, sys, tool, Some(Td::of_number(num)));
    } else if num > 0 {
        deps::send_localized_message(w, from, num);
    }
}

#[allow(clippy::too_many_arguments)]
fn complete_craft_hued(
    w: &mut World,
    from: MobileId,
    sys: &Rc<CraftSystem>,
    item: &CraftItem,
    _index: usize,
    mut quality: i32,
    makers_mark: bool,
    type_res: Option<&'static str>,
    tool: Option<ItemId>,
    target_hue: i32,
) {
    // Dry-run check with hue filtering
    let mut check_hue = 0;
    if !consume_hued_res(
        w,
        from,
        sys,
        item,
        type_res,
        target_hue,
        &mut check_hue,
        ConsumeType::None,
    ) {
        if tool_usable(w, tool) {
            show_craft_menu(w, from, sys, tool, Some(Td::of_number(502925)));
        } else {
            deps::send_localized_message(w, from, 502925);
        }
        return;
    }
    let mut check_message = None;
    if !consume_attributes(w, from, item, &mut check_message, false) {
        return;
    }

    let mut tool_broken = false;
    let mut endquality = 1;
    let mut res_hue = 0;
    let num;
    let mut all_required = true;

    if check_skills(w, sys, item, from, &mut quality, &mut all_required, true) {
        let consume_type = if item.use_all_res {
            ConsumeType::Half
        } else {
            ConsumeType::All
        };
        if !consume_hued_res(
            w,
            from,
            sys,
            item,
            type_res,
            target_hue,
            &mut res_hue,
            consume_type,
        ) {
            if tool_usable(w, tool) {
                show_craft_menu(w, from, sys, tool, Some(Td::of_number(502925)));
            } else {
                deps::send_localized_message(w, from, 502925);
            }
            return;
        }

        tool_broken = use_tool(w, tool, sys.name == "Blacksmithy", from);

        let mut dummy = 0;
        if let Some(it) = make_item_plain(w, item, &mut dummy) {
            if let Some(i) = w.item_mut(it) {
                i.player_constructed = true;
            }
            let args = craft_args(
                w,
                sys,
                item,
                from,
                quality,
                makers_mark,
                type_res,
                tool,
                res_hue,
            );
            match super::on_craft(w, it, &args) {
                Some(q) => endquality = q,
                None => {
                    if w.item(it).is_some_and(|i| i.hue == 0)
                        && let Some(ty) = w.item(it).map(|i| i.ty)
                    {
                        (ty.set_hue)(w, it, res_hue);
                    }
                }
            }
            muo_server::mobile::part2::items::add_to_backpack(w, from, it);

            num = (sys.vt.play_ending_effect)(
                w,
                from,
                Ending {
                    failed: false,
                    lost_material: true,
                    tool_broken,
                    quality: endquality,
                    makers_mark: false,
                },
                item,
            );
            after_message(w, from, sys, tool, num);
            return;
        }
    }

    if !all_required {
        if tool_usable(w, tool) {
            muo_server::mobile::part2::messages::send_ascii_message(
                w,
                from,
                "You lack the required skill to craft this item.",
            );
        } else {
            deps::send_localized_message(w, from, 1044153);
        }
        return;
    }

    // Failure: consume resources (half on failure)
    let fail_type = if item.use_all_res {
        ConsumeType::Half
    } else {
        ConsumeType::All
    };
    consume_hued_res(
        w,
        from,
        sys,
        item,
        type_res,
        target_hue,
        &mut res_hue,
        fail_type,
    );

    if let Some(t) = tool {
        let uses = tool_mod::uses_remaining(w, t) - 1;
        tool_mod::set_uses_remaining(w, t, uses);
        if uses < 1 && tool_mod::class(w, t).break_on_depletion {
            tool_broken = true;
            if let Some(ty) = w.item(t).map(|i| i.ty) {
                (ty.delete)(w, t);
            }
        }
    }

    num = (sys.vt.play_ending_effect)(
        w,
        from,
        Ending {
            failed: true,
            lost_material: true,
            tool_broken,
            quality: endquality,
            makers_mark: false,
        },
        item,
    );
    if tool_usable(w, tool) {
        show_craft_menu(w, from, sys, tool, Some(Td::of_number(num)));
    } else if num > 0 {
        deps::send_localized_message(w, from, num);
    }
}

/// The new item (a map outside Trammel and Felucca becomes an IndecipherableMap).
fn make_item(w: &mut World, from: MobileId, item: &CraftItem, num: &mut i32) -> Option<ItemId> {
    if derives(w, item.item_type, "Server.Items.MapItem") {
        let map = mx::map(w, from);
        let trammel_or_felucca =
            map.is_some_and(|m| m == muo_server::map::MapId(0) || m == muo_server::map::MapId(1));
        if !trammel_or_felucca {
            let it = muo_content::factory::make_item(w, "Server.Items.IndecipherableMap");
            deps::send_localized_message(w, from, 1070800); // The map you create becomes mysteriously indecipherable.
            return it;
        }
    }
    make_item_plain(w, item, num)
}

fn make_item_plain(w: &mut World, item: &CraftItem, _num: &mut i32) -> Option<ItemId> {
    muo_content::factory::make_item(w, item.item_type)
}

/// The arguments `ICraftable.OnCraft` gets.
#[allow(clippy::too_many_arguments)]
fn craft_args(
    w: &mut World,
    sys: &CraftSystem,
    item: &CraftItem,
    from: MobileId,
    quality: i32,
    makers_mark: bool,
    type_res: Option<&'static str>,
    tool: Option<ItemId>,
    res_hue: i32,
) -> CraftArgs {
    let resource_type =
        type_res.unwrap_or_else(|| item.resources.first().map_or("", |r| r.item_type));
    let ctx = context_of(w, sys.name, from);
    let mut a = CraftArgs::new(quality, makers_mark, from, rc::from_type(resource_type));
    a.res_hue = res_hue;
    a.craft_system = sys.name;
    a.do_not_color = ctx.as_ref().is_some_and(|c| c.do_not_color);
    a.runic_tool = tool.filter(|t| derives(w, item_type_name(w, *t), "Server.Items.BaseRunicTool"));
    if item.resources.len() > 1 {
        a.second_resource = gem_of_type(item.resources[1].item_type);
    }
    a.pending_gem = ctx.and_then(|c| c.pending_gem);
    a
}

/// The `GemType` of a gem item class.
fn gem_of_type(ty: &str) -> Option<GemType> {
    Some(match ty {
        "Server.Items.StarSapphire" => GemType::StarSapphire,
        "Server.Items.Emerald" => GemType::Emerald,
        "Server.Items.Sapphire" => GemType::Sapphire,
        "Server.Items.Ruby" => GemType::Ruby,
        "Server.Items.Citrine" => GemType::Citrine,
        "Server.Items.Amethyst" => GemType::Amethyst,
        "Server.Items.Tourmaline" => GemType::Tourmaline,
        "Server.Items.Amber" => GemType::Amber,
        "Server.Items.Diamond" => GemType::Diamond,
        _ => return None,
    })
}

/// `CraftResources.GetFromType` for the first resource type of a recipe.
pub fn resource_of(item: &CraftItem, type_res: Option<&str>) -> CraftResource {
    rc::from_type(type_res.unwrap_or_else(|| item.resources.first().map_or("", |r| r.item_type)))
}

/// `from.Skills[skill].Base` (for the gumps).
pub fn skill_base(w: &mut World, from: MobileId, skill: SkillName) -> f64 {
    skills::base(w, from, skill)
}

#[allow(dead_code)]
fn _unused(_: &mut World) {
    let _ = (system, tool_deleted);
}
