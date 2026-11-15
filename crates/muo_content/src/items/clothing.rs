//! `BaseClothing` (save version 8) and the clothing base classes: `BaseCloak`,
//! `BaseHat`, `BaseMiddleTorso`, `BaseOuterLegs`, `BaseOuterTorso`, `BasePants`, `BaseShirt`, `BaseShoes`,
//! `BaseWaist`.
//!
//! State is [`ClothingData`] in `Item::ext` (the shipwreck flag of `BaseHat` sits on it too), the constants and
//! virtuals of a class are [`ClothingClass`]. A leaf is one `clothing_leaf!` call in `items/leaves.rs`.
//!
//! Saves: version 8 is the current layout. Versions 6 and 7 (`V6Content`, `V7Content`) and the pre-codegen layout
//! of everything older are read with their quirks, kept for save compatibility (see [`Layout`]).

use muo_io::{GenericReader, GenericWriter, Result};
use muo_server::World;
use muo_server::ext::Ext;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::base;
use muo_server::item::deps;
use muo_server::item::ops;
use muo_server::item::{ITEM_BASE, ItemType, LootType};
use muo_server::layer::Layer;
use muo_server::mobile::{AccessLevel, StatType};
use muo_server::packets::equipment::{EquipInfoAttribute, send_display_equipment_info};
use muo_server::property_list::PropertyList;
use muo_server::serialization_ext::ReadEntityExt;

use super::armor::{append_space_with_article, ore_type_number};
use super::equip_hooks::{CraftArgs, hooks, lower_text};
use super::equip_props as ep;
use super::weapon::{self, WeaponType};
use super::{MlInt, Wear, core_uotd, parent_mobile, random_min_max, read_enum, same};
use crate::aos::{
    AosArmorAttribute, AosAttribute, AosElementAttribute, AttrKind, Attrs,
    add_armor_attribute_properties, add_attribute_properties, add_skill_bonus_properties,
};
use crate::core;
use crate::craft::{self, CraftResource};
use crate::hooks::rnd;
use crate::misc::aos_helpers;
use crate::templates::item_props as ip;

int_enum! {
    /// `ClothingQuality`.
    pub enum ClothingQuality { Low = 0, Regular = 1, Exceptional = 2 }
}

/// The `override`s of a clothing class: its constants, and the virtuals that are code. A subclass is
/// `ClothingClass { .. , ..PARENT }`.
#[derive(Copy, Clone)]
pub struct ClothingClass {
    /// The `layer` the abstract class passes to `BaseClothing(itemID, layer, hue)`.
    pub layer: Layer,
    pub default_weight: Option<f64>,
    /// `LabelNumber` when overridden (0 keeps the tiledata name).
    pub label_number: i32,
    /// `RequiredRaces` when overridden (`Race.AllowElvesOnly`, ..).
    pub required_races: Option<i32>,
    /// `RequiredRace` (a single race by its name: "Elf"), when overridden.
    pub required_race: Option<&'static str>,
    pub artifact_rarity: i32,
    pub base_physical_resistance: i32,
    pub base_fire_resistance: i32,
    pub base_cold_resistance: i32,
    pub base_poison_resistance: i32,
    pub base_energy_resistance: i32,
    pub init_min_hits: MlInt,
    pub init_max_hits: MlInt,
    pub base_str_bonus: i32,
    pub base_dex_bonus: i32,
    pub base_int_bonus: i32,
    pub aos_str_req: i32,
    pub old_str_req: i32,
    pub allow_male_wearer: bool,
    pub allow_female_wearer: bool,
    pub can_fortify: bool,
    pub default_resource: CraftResource,
    /// A bare `[Flippable]`: the class's own `Flip()` method (the `Flip` command calls it).
    pub flip: Option<fn(&mut World, ItemId)>,
    /// `OnCraft(..)`.
    pub on_craft: fn(&mut World, ItemId, &CraftArgs) -> i32,
    /// `Scissor(from, scissors)`.
    pub scissor: fn(&mut World, ItemId, MobileId) -> bool,
    /// `Dye(from, sender)`: `(world, item, from, sender)`, the tub (see [`tub_dyed_hue`], [`tub_fail_message`]).
    pub dye: fn(&mut World, ItemId, MobileId, ItemId) -> bool,
    /// `OnHit(weapon, damageTaken)`: `(world, clothing, weapon, damage)`.
    pub on_hit: fn(&mut World, ItemId, ItemId, i32) -> i32,
    /// `AddEquipInfoAttributes(from, attrs)`.
    pub add_equip_info_attributes: fn(&World, ItemId, MobileId, &mut Vec<EquipInfoAttribute>),
}

/// `BaseClothing`'s defaults.
pub static BASE_CLOTHING_CLASS: ClothingClass = ClothingClass {
    layer: Layer::Invalid,
    default_weight: None,
    label_number: 0,
    required_races: None,
    required_race: None,
    artifact_rarity: 0,
    base_physical_resistance: 0,
    base_fire_resistance: 0,
    base_cold_resistance: 0,
    base_poison_resistance: 0,
    base_energy_resistance: 0,
    init_min_hits: same(0),
    init_max_hits: same(0),
    base_str_bonus: 0,
    base_dex_bonus: 0,
    base_int_bonus: 0,
    aos_str_req: 10,
    old_str_req: 0,
    allow_male_wearer: true,
    allow_female_wearer: true,
    can_fortify: true,
    default_resource: CraftResource::None,
    flip: None,
    on_craft: on_craft_virtual,
    scissor: scissor_virtual,
    dye: dye_virtual,
    on_hit: on_hit_virtual,
    add_equip_info_attributes: add_equip_info_attributes_virtual,
};

/// Everything `BaseClothing` (and `BaseHat`) adds to `Item`.
#[derive(Clone, Debug)]
pub struct ClothingData {
    pub attributes: Attrs,
    pub clothing_attributes: Attrs,
    pub skill_bonuses: Attrs,
    pub resistances: Attrs,
    pub hit_points: i32,
    pub max_hit_points: i32,
    pub crafter: Option<String>,
    pub quality: ClothingQuality,
    pub resource: CraftResource,
    /// -1 means the class's value.
    pub str_req: i32,
    /// `BaseHat._isShipwreckedItem`.
    pub is_shipwrecked: bool,
    /// `IArcaneEquip` of `Cloak`, `Robe` and `ThighBoots`: `(CurArcaneCharges, MaxArcaneCharges)`.
    pub arcane_charges: (i32, i32),
    /// `RewardCloak/RewardRobe/RewardDress._number` (the label) and `_isRewardItem`.
    pub number: i32,
    pub is_reward_item: bool,
    /// `DeathRobe._decayTimer`.
    pub decay_timer: muo_server::timer::TimerHandle,
}

impl ClothingData {
    /// The field initializers of `BaseClothing` as `new BaseClothing(Serial)` leaves them (the loader
    /// constructor): `_resource` is not set, so it is `None` unless the record has it.
    fn for_load() -> ClothingData {
        ClothingData {
            attributes: Attrs::default(),
            clothing_attributes: Attrs::default(),
            skill_bonuses: Attrs::default(),
            resistances: Attrs::default(),
            hit_points: 0,
            max_hit_points: 0,
            crafter: None,
            quality: ClothingQuality::Regular,
            resource: CraftResource::None,
            str_req: -1,
            is_shipwrecked: false,
            arcane_charges: (0, 0),
            number: 0,
            is_reward_item: false,
            decay_timer: muo_server::timer::TimerHandle::default(),
        }
    }
}

/// The clothing state of an item.
pub fn data(w: &World, id: ItemId) -> Option<&ClothingData> {
    w.item(id)?.ext.get()
}

/// The clothing state of an item, mutable.
pub fn data_mut(w: &mut World, id: ItemId) -> Option<&mut ClothingData> {
    w.item_mut(id)?.ext.get_mut()
}

/// The constants of the item's class.
pub fn class(w: &World, id: ItemId) -> &'static ClothingClass {
    w.item(id)
        .and_then(|i| i.ty.family::<ClothingClass>())
        .unwrap_or(&BASE_CLOTHING_CLASS)
}

/// `LabelNumber` when overridden: the class's template prop `labelNumber`, else its table field (0 keeps the
/// tiledata name).
pub fn class_label_number(w: &World, id: ItemId) -> i32 {
    crate::templates::item_props::int(w, id, "labelNumber", class(w, id).label_number)
}

/// `ArtifactRarity`: the class's template prop `artifactRarity`, else its table field.
pub fn artifact_rarity(w: &World, id: ItemId) -> i32 {
    crate::templates::item_props::int(w, id, "artifactRarity", class(w, id).artifact_rarity)
}

/// `InitMinHits` and `InitMaxHits` of the era.
pub fn init_hits(w: &World, id: ItemId) -> (i32, i32) {
    let (c, ml) = (class(w, id), core::ml(w));
    (
        ip::ml_int(w, id, "initMinHits", c.init_min_hits).get(ml),
        ip::ml_int(w, id, "initMaxHits", c.init_max_hits).get(ml),
    )
}

/// `DefaultResource`.
pub fn default_resource(w: &World, id: ItemId) -> CraftResource {
    ep::named(
        w,
        id,
        "defaultResource",
        &ep::RESOURCES,
        class(w, id).default_resource,
    )
}

/// `AllowMaleWearer` and `AllowFemaleWearer`.
pub fn allow_wearer(w: &World, id: ItemId) -> (bool, bool) {
    let c = class(w, id);
    (
        ep::boolean(w, id, "allowMaleWearer", c.allow_male_wearer),
        c.allow_female_wearer,
    )
}

/// `CanFortify`.
pub fn can_fortify(w: &World, id: ItemId) -> bool {
    ep::boolean(w, id, "canFortify", class(w, id).can_fortify)
}

/// `item is BaseClothing`.
pub fn is_clothing(w: &World, id: ItemId) -> bool {
    w.item(id).is_some_and(|i| i.ty.is(&BASE_CLOTHING))
}

/// `new BaseClothing(itemID, layer, hue)` for class `ty` (the layer is its class's); every leaf constructor starts
/// here.
pub fn new_clothing(w: &mut World, ty: &'static ItemType, item_id: i32, hue: i32) -> ItemId {
    let id = ops::new_item_with_id(w, ty, item_id);
    let c = ty.family::<ClothingClass>().unwrap_or(&BASE_CLOTHING_CLASS);
    (ty.set_layer)(w, id, c.layer);
    (ty.set_hue)(w, id, hue);
    crate::templates::table(w); // the class props below read the loaded table
    let (min, max) = init_hits(w, id);
    let hits = random_min_max(w, min, max);
    let mut d = ClothingData::for_load();
    d.resource = default_resource(w, id);
    d.hit_points = hits;
    d.max_hit_points = hits;
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(d);
    }
    id
}

// ---- getters ----

/// `StrRequirement`.
pub fn str_requirement(w: &World, id: ItemId) -> i32 {
    let set = data(w, id).map_or(-1, |d| d.str_req);
    if set != -1 {
        set
    } else if core::aos(w) {
        class(w, id).aos_str_req
    } else {
        class(w, id).old_str_req
    }
}

/// `ClothingAttributes[attribute]`.
fn clothing_attr(w: &World, id: ItemId, a: AosArmorAttribute) -> i32 {
    super::attr(w, id, AttrKind::Armor, a as u32)
}

/// `GetLowerStatReq()`.
pub fn lower_stat_req(w: &World, id: ItemId) -> i32 {
    if core::aos(w) {
        clothing_attr(w, id, AosArmorAttribute::LowerStatReq)
    } else {
        0
    }
}

/// `ComputeStatReq(type)` (clothing only has a Str requirement, whatever the stat).
pub fn compute_stat_req(w: &World, id: ItemId, _stat: StatType) -> i32 {
    aos_helpers::scale(str_requirement(w, id), 100 - lower_stat_req(w, id))
}

/// `ComputeStatBonus(type)`.
pub fn compute_stat_bonus(w: &World, id: ItemId, stat: StatType) -> i32 {
    let c = class(w, id);
    let a = |x: AosAttribute| super::attr(w, id, AttrKind::Attributes, x as u32);
    match stat {
        StatType::STR => {
            ip::int(w, id, "baseStrBonus", c.base_str_bonus) + a(AosAttribute::BonusStr)
        }
        StatType::DEX => {
            ip::int(w, id, "baseDexBonus", c.base_dex_bonus) + a(AosAttribute::BonusDex)
        }
        _ => ip::int(w, id, "baseIntBonus", c.base_int_bonus) + a(AosAttribute::BonusInt),
    }
}

/// `Resistances.Physical` and the others.
fn resist(w: &World, id: ItemId, e: AosElementAttribute) -> i32 {
    super::attr(w, id, AttrKind::Element, e as u32)
}

/// `UnscaleDurability()`.
pub fn unscale_durability(w: &mut World, id: ItemId) {
    let scale = 100 + clothing_attr(w, id, AosArmorAttribute::DurabilityBonus);
    if let Some(d) = data_mut(w, id) {
        d.hit_points = (d.hit_points * 100 + (scale - 1)) / scale;
        d.max_hit_points = (d.max_hit_points * 100 + (scale - 1)) / scale;
    }
    ops::invalidate_properties(w, id);
}

/// `ScaleDurability()`.
pub fn scale_durability(w: &mut World, id: ItemId) {
    let scale = 100 + clothing_attr(w, id, AosArmorAttribute::DurabilityBonus);
    if let Some(d) = data_mut(w, id) {
        d.hit_points = (d.hit_points * scale + 99) / 100;
        d.max_hit_points = (d.max_hit_points * scale + 99) / 100;
    }
    ops::invalidate_properties(w, id);
}

// ---- setters with side effects ----

/// `Quality` setter.
pub fn set_quality(w: &mut World, id: ItemId, value: ClothingQuality) {
    if let Some(d) = data_mut(w, id)
        && d.quality != value
    {
        d.quality = value;
        ops::invalidate_properties(w, id);
    }
}

/// `Crafter` setter.
pub fn set_crafter(w: &mut World, id: ItemId, value: Option<String>) {
    if let Some(d) = data_mut(w, id)
        && d.crafter != value
    {
        d.crafter = value;
        ops::invalidate_properties(w, id);
    }
}

/// `Resource` setter (`OnResourceChanged`: the hue follows).
pub fn set_resource(w: &mut World, id: ItemId, value: CraftResource) {
    let Some(d) = data_mut(w, id) else { return };
    if d.resource == value {
        return;
    }
    d.resource = value;
    ops::invalidate_properties(w, id);
    if let Some(ty) = w.item(id).map(|i| i.ty) {
        let hue = craft::hue(w, value);
        (ty.set_hue)(w, id, hue);
    }
}

/// `StrRequirement` setter.
pub fn set_str_requirement(w: &mut World, id: ItemId, value: i32) {
    if let Some(d) = data_mut(w, id) {
        d.str_req = value;
    }
    ops::invalidate_properties(w, id);
}

/// `HitPoints` setter: ignored without `MaxHitPoints`; below zero the clothing is destroyed (and, unlike armor,
/// the setter goes on).
pub fn set_hit_points(w: &mut World, id: ItemId, value: i32) {
    let Some(d) = data_mut(w, id) else { return };
    if value == d.hit_points || d.max_hit_points <= 0 {
        return;
    }
    d.hit_points = value;
    if value < 0 {
        if let Some(ty) = w.item(id).map(|i| i.ty) {
            (ty.delete)(w, id);
        }
    } else if let Some(d) = data_mut(w, id)
        && d.hit_points > d.max_hit_points
    {
        d.hit_points = d.max_hit_points;
    }
    ops::invalidate_properties(w, id);
}

/// `MaxHitPoints` setter.
pub fn set_max_hit_points(w: &mut World, id: ItemId, value: i32) {
    if let Some(d) = data_mut(w, id)
        && d.max_hit_points != value
    {
        d.max_hit_points = value;
        ops::invalidate_properties(w, id);
    }
}

/// `IsShipwreckedItem` of a `BaseHat`.
pub fn is_shipwrecked(w: &World, id: ItemId) -> bool {
    data(w, id).is_some_and(|d| d.is_shipwrecked)
}

/// `DistributeBonuses(amount)`: `++Resistances.Physical` and the other four, picked at random.
pub fn distribute_bonuses(w: &mut World, id: ItemId, amount: i32) {
    for _ in 0..amount {
        let e = match rnd::below(w, 5) {
            0 => AosElementAttribute::Physical,
            1 => AosElementAttribute::Fire,
            2 => AosElementAttribute::Cold,
            3 => AosElementAttribute::Poison,
            _ => AosElementAttribute::Energy,
        };
        let v = resist(w, id, e) + 1;
        super::set_attr(w, id, AttrKind::Element, e as u32, v);
    }
    ops::invalidate_properties(w, id);
}

// ---- crafting, dyeing, scissors, combat ----

fn on_craft_virtual(w: &mut World, id: ItemId, a: &CraftArgs) -> i32 {
    // shortcut: a quality number the craft system never passes (not 0..=2) reads as Regular.
    set_quality(
        w,
        id,
        ClothingQuality::from_i32(a.quality).unwrap_or(ClothingQuality::Regular),
    );
    if a.makers_mark {
        let name = muo_server::mobile::part1::props_b::raw_name(w, a.from);
        set_crafter(w, id, name);
    }
    let Some(ty) = w.item(id).map(|i| i.ty) else {
        return a.quality;
    };
    if default_resource(w, id) != CraftResource::None {
        set_resource(w, id, a.resource);
    } else {
        (ty.set_hue)(w, id, a.res_hue);
    }
    if a.do_not_color {
        (ty.set_hue)(w, id, 0);
    }
    a.quality
}

/// `OnCraft(..)`.
pub fn on_craft(w: &mut World, id: ItemId, a: &CraftArgs) -> i32 {
    (class(w, id).on_craft)(w, id, a)
}

fn hat_on_craft(w: &mut World, id: ItemId, a: &CraftArgs) -> i32 {
    set_quality(
        w,
        id,
        ClothingQuality::from_i32(a.quality).unwrap_or(ClothingQuality::Regular),
    );
    if data(w, id).is_some_and(|d| d.quality == ClothingQuality::Exceptional) {
        // BLAME OSI. (We can't confirm it's an OSI bug yet.)
        let amount = if a.runic_tool.is_some() {
            6
        } else if core::se(w) {
            15
        } else {
            14
        };
        distribute_bonuses(w, id, amount);
    }
    on_craft_virtual(w, id, a)
}

/// `sender.DyedHue`.
pub fn tub_dyed_hue(w: &World, tub: ItemId) -> i32 {
    match hooks(w).and_then(|h| h.dye_tub) {
        Some(f) => f(w, tub).0,
        None => w.item(tub).map_or(0, |i| i.hue),
    }
}

/// `sender.FailMessage`.
pub fn tub_fail_message(w: &World, tub: ItemId) -> i32 {
    hooks(w)
        .and_then(|h| h.dye_tub)
        .map_or(1042083, |f| f(w, tub).1)
}

fn dye_virtual(w: &mut World, id: ItemId, from: MobileId, tub: ItemId) -> bool {
    let dyed_hue = tub_dyed_hue(w, tub);
    let Some(ty) = w.item(id).filter(|i| !i.deleted()).map(|i| i.ty) else {
        return false;
    };
    if let Some(EntityId::Mobile(root)) = ops::root_parent(w, id)
        && root != from
    {
        return false;
    }
    (ty.set_hue)(w, id, dyed_hue);
    true
}

/// `Dye(from, sender)`.
pub fn dye(w: &mut World, id: ItemId, from: MobileId, tub: ItemId) -> bool {
    (class(w, id).dye)(w, id, from, tub)
}

fn scissor_with(w: &mut World, id: ItemId, from: MobileId) -> bool {
    let pack = muo_server::mobile::part1::props_b::backpack(w, from);
    if !ops::is_child_of(w, id, pack.map(EntityId::Item)) {
        deps::send_localized_message(w, from, 502437); // Items you wish to cut must be in your backpack.
        return false;
    }
    if hooks(w)
        .and_then(|h| h.ethic_is_imbued)
        .is_some_and(|f| f(w, id))
    {
        deps::send_localized_message(w, from, 502440); // Scissors can not be used on that to produce anything.
        return false;
    }
    let ty = w.item(id).map(|i| i.ty);
    let resource = data(w, id).map_or(CraftResource::None, |d| d.resource);
    if let Some(ty) = ty
        && let Some(f) = hooks(w).and_then(|h| h.tailoring_scissor)
        && let Some((res, amount)) = f(w, ty, resource, true)
    {
        let per = if w.item(id).is_some_and(|i| i.player_constructed) {
            amount / 2
        } else {
            1
        };
        (ty.scissor_helper)(w, id, from, res, per, true);
        ops::set_loot_type(w, res, LootType::Regular);
        return true;
    }
    deps::send_localized_message(w, from, 502440); // Scissors can not be used on that to produce anything.
    false
}

fn scissor_virtual(w: &mut World, id: ItemId, from: MobileId) -> bool {
    scissor_with(w, id, from)
}

/// `BaseShoes.Scissor`: only cloth shoes can be cut.
fn shoes_scissor(w: &mut World, id: ItemId, from: MobileId) -> bool {
    if default_resource(w, id) == CraftResource::None {
        return scissor_with(w, id, from);
    }
    deps::send_localized_message(w, from, 502440); // Scissors can not be used on that to produce anything.
    false
}

/// `Scissor(from, scissors)`.
pub fn scissor(w: &mut World, id: ItemId, from: MobileId) -> bool {
    (class(w, id).scissor)(w, id, from)
}

const WEAR: Wear = Wear {
    hit_points: |w, id| data(w, id).map_or(0, |d| d.hit_points),
    max_hit_points: |w, id| data(w, id).map_or(0, |d| d.max_hit_points),
    set_hit_points,
    set_max_hit_points,
};

fn on_hit_virtual(w: &mut World, id: ItemId, weapon_id: ItemId, damage: i32) -> i32 {
    let absorbed = rnd::min_max(w, 1, 4);
    // Don't go below zero
    let damage = absorbed.min(damage);

    if rnd::below(w, 100) < 25 {
        // 25% chance to lower durability
        if core::aos(w) && clothing_attr(w, id, AosArmorAttribute::SelfRepair) > rnd::below(w, 10) {
            let hp = data(w, id).map_or(0, |d| d.hit_points);
            set_hit_points(w, id, hp + 2);
        } else {
            let wear = if weapon::weapon_type(w, weapon_id) == WeaponType::Bashing {
                absorbed / 2
            } else {
                rnd::below(w, 2)
            };
            WEAR.apply(w, id, wear);
        }
    }
    damage
}

/// `OnHit(weapon, damageTaken)`.
pub fn on_hit(w: &mut World, id: ItemId, weapon_id: ItemId, damage: i32) -> i32 {
    (class(w, id).on_hit)(w, id, weapon_id, damage)
}

// ---- wearing ----

/// The race `RequiredRace` names, as the world's race (`Race.Elf`, ..).
fn required_race(w: &World, id: ItemId) -> Option<(muo_server::race::RaceId, String)> {
    let name = class(w, id).required_race?;
    let races = w.resources.get::<muo_server::race::Races>()?;
    races
        .races
        .iter()
        .flatten()
        .find(|r| r.name == name)
        .map(|r| (r.id(), r.plural_name.clone()))
}

/// `RequiredRace != null && from.Race != RequiredRace`, with the message the wearer gets.
fn wrong_race(w: &mut World, id: ItemId, m: MobileId) -> bool {
    let Some(name) = class(w, id).required_race else {
        return false;
    };
    let Some((race, plural)) = required_race(w, id) else {
        return true;
    };
    if w.mobile(m).and_then(|m| m.race) == Some(race) {
        return false;
    }
    if name == "Elf" {
        deps::send_localized_message(w, m, 1072203); // Only Elves may use this.
    } else {
        deps::send_message(w, m, &format!("Only {plural} may use this."));
    }
    true
}

/// `AddStatBonuses(parent)`.
pub fn add_stat_bonuses(w: &mut World, id: ItemId, parent: MobileId) {
    let (s, d, i) = (
        compute_stat_bonus(w, id, StatType::STR),
        compute_stat_bonus(w, id, StatType::DEX),
        compute_stat_bonus(w, id, StatType::INT),
    );
    if s != 0 || d != 0 || i != 0 {
        super::add_stat_mods(w, id, parent, s, d, i);
    }
}

/// `ValidateMobile(m)`: sends what the wearer may no longer wear to their pack.
pub fn validate_mobile(w: &mut World, m: MobileId) {
    let Some(items) = w.mobile(m).map(|m| m.items.clone()) else {
        return;
    };
    for &item in items.iter().rev() {
        if !is_clothing(w, item) {
            continue;
        }
        let gm = w
            .mobile(m)
            .is_none_or(|m| m.access_level >= AccessLevel::GameMaster);
        if wrong_race(w, item, m)
            || (!gm && {
                let (male, female) = allow_wearer(w, item);
                super::wrong_gender(w, m, male, female)
            })
        {
            deps::add_to_backpack(w, m, item);
        }
    }
}

fn on_added(w: &mut World, id: ItemId, parent: EntityId) {
    if let EntityId::Mobile(m) = parent {
        if core::aos(w) {
            super::skill_bonuses_add_to(w, id, m);
        }
        add_stat_bonuses(w, id, m);
        if let Some(ty) = w.mobile(m).map(|m| m.ty) {
            (ty.check_stat_timers)(w, m);
        }
    }
    base::on_added_base(w, id, parent);
}

fn on_removed(w: &mut World, id: ItemId, parent: EntityId) {
    if let EntityId::Mobile(m) = parent {
        if core::aos(w) {
            super::skill_bonuses_remove(w, id);
        }
        super::remove_stat_mods(w, id, m);
        if let Some(ty) = w.mobile(m).map(|m| m.ty) {
            (ty.check_stat_timers)(w, m);
        }
    }
    base::on_removed_base(w, id, parent);
}

fn allow_secure_trade(
    w: &mut World,
    id: ItemId,
    from: MobileId,
    to: MobileId,
    new_owner: MobileId,
    accepted: bool,
) -> bool {
    if let Some(f) = hooks(w).and_then(|h| h.ethic_check_trade)
        && !f(w, from, to, new_owner, id)
    {
        return false;
    }
    base::allow_secure_trade_base(w, id, from, to, new_owner, accepted)
}

fn can_equip(w: &mut World, id: ItemId, from: MobileId) -> bool {
    let Some(m) = w.mobile(from) else {
        return base::can_equip_base(w, id, from);
    };
    if !m.player || m.access_level >= AccessLevel::GameMaster {
        return base::can_equip_base(w, id, from);
    }
    if let Some(f) = hooks(w).and_then(|h| h.ethic_check_equip)
        && !f(w, from, id)
    {
        return false;
    }
    if wrong_race(w, id, from) {
        return false;
    }
    let (male, female) = allow_wearer(w, id);
    if super::wrong_gender(w, from, male, female) {
        return false;
    }
    let str_bonus = compute_stat_bonus(w, id, StatType::STR);
    let str_req = compute_stat_req(w, id, StatType::STR);
    let str = muo_server::mobile::base::str(w, from);
    if str < str_req || str + str_bonus < 1 {
        deps::send_localized_message(w, from, 500213); // You are not strong enough to equip that.
        return false;
    }
    base::can_equip_base(w, id, from)
}

fn on_after_duped(w: &mut World, id: ItemId, new_item: ItemId) {
    let Some(src) = data(w, id).cloned() else {
        return;
    };
    let hue = w.item(id).map_or(0, |i| i.hue);
    let Some(ty) = w.item(new_item).map(|i| i.ty) else {
        return;
    };
    let Some(d) = data_mut(w, new_item) else {
        return;
    };
    d.attributes = src.attributes.clone();
    d.resistances = src.resistances.clone();
    d.skill_bonuses = src.skill_bonuses.clone();
    d.clothing_attributes = src.clothing_attributes.clone();
    // Set hue again because of resource
    (ty.set_hue)(w, new_item, hue);
    // Set HP/Max again because of durability
    set_hit_points(w, new_item, src.hit_points);
    set_max_hit_points(w, new_item, src.max_hit_points);
}

fn allow_equipped_cast(w: &World, id: ItemId, from: MobileId) -> bool {
    base::allow_equipped_cast_base(w, id, from)
        || super::attr(
            w,
            id,
            AttrKind::Attributes,
            AosAttribute::SpellChanneling as u32,
        ) != 0
}

fn check_property_conflict(w: &World, id: ItemId, m: MobileId) -> bool {
    base::check_property_conflict_base(w, id, m) || super::property_conflict_layer(w, id, m)
}

fn add_name_property(w: &World, id: ItemId, list: &mut PropertyList) {
    let Some(d) = data(w, id) else { return };
    let Some(ty) = w.item(id).map(|i| i.ty) else {
        return;
    };
    let ore = ore_type_number(d.resource);
    let name = ops::name(w, id);
    let label = (ty.label_number)(w, id);
    if ore != 0 {
        match name {
            // ~1_oretype~ ~2_armortype~
            Some(n) => list.add_number_fmt(1053099, format_args!("#{ore}\t{n}")),
            None => list.add_number_fmt(1053099, format_args!("#{ore}\t#{label}")),
        }
    } else {
        match name {
            None => list.add(label),
            Some(n) => list.add_text(&n),
        }
    }
}

fn get_properties(w: &World, id: ItemId, list: &mut PropertyList) {
    base::get_properties_base(w, id, list);
    let (Some(d), Some(ty)) = (data(w, id), w.item(id).map(|i| i.ty)) else {
        return;
    };
    let aos = core::aos(w);
    if let Some(c) = &d.crafter {
        list.add_str(1050043, c); // crafted by ~1_NAME~
    }
    if hooks(w)
        .and_then(|h| h.faction_item)
        .is_some_and(|f| f(w, id))
    {
        list.add(1041350); // faction item
    }
    if d.quality == ClothingQuality::Exceptional {
        list.add(1060636); // exceptional
    }
    let races = (ty.required_races)(w, id);
    if races == muo_server::race::ALLOW_ELVES_ONLY {
        list.add(1075086); // Elves Only
    }
    if races == muo_server::race::ALLOW_GARGOYLES_ONLY {
        list.add(1111709); // Gargoyles Only
    }
    add_skill_bonus_properties(&d.skill_bonuses, list, aos);

    let rarity = artifact_rarity(w, id);
    if rarity > 0 {
        list.add_int(1061078, rarity); // artifact rarity ~1_val~
    }
    add_armor_attribute_properties(&d.clothing_attributes, list, aos);
    add_attribute_properties(&d.attributes, list, aos, core::ml(w), 0, 0, 0);
    (ty.add_resistance_properties)(w, id, list);

    let prop = clothing_attr(w, id, AosArmorAttribute::DurabilityBonus);
    if prop > 0 {
        list.add_int(1060410, prop); // durability ~1_val~%
    }
    let lower = clothing_attr(w, id, AosArmorAttribute::LowerStatReq);
    if lower != 0 {
        list.add_int(1060435, lower); // lower requirements ~1_val~%
    }
    let prop = compute_stat_req(w, id, StatType::STR);
    if prop > 0 {
        list.add_int(1061170, prop); // strength requirement ~1_val~
    }
    if d.hit_points >= 0 && d.max_hit_points > 0 {
        // durability ~1_val~ / ~2_val~
        list.add_number_fmt(
            1060639,
            format_args!("{}\t{}", d.hit_points, d.max_hit_points),
        );
    }
}

fn add_equip_info_attributes_virtual(
    w: &World,
    id: ItemId,
    _from: MobileId,
    attrs: &mut Vec<EquipInfoAttribute>,
) {
    let Some((d, ty)) = data(w, id).zip(w.item(id).map(|i| i.ty)) else {
        return;
    };
    if (ty.display_loot_type)(w, id) {
        match w.item(id).map(|i| i.loot_type) {
            Some(LootType::Blessed) => attrs.push(EquipInfoAttribute::new(1038021, -1)), // blessed
            Some(LootType::Cursed) => attrs.push(EquipInfoAttribute::new(1049643, -1)),  // cursed
            _ => {}
        }
    }
    if hooks(w)
        .and_then(|h| h.faction_item)
        .is_some_and(|f| f(w, id))
    {
        attrs.push(EquipInfoAttribute::new(1041350, -1)); // faction item
    }
    if d.quality == ClothingQuality::Exceptional {
        attrs.push(EquipInfoAttribute::new(1018305 - d.quality as i32, -1));
    }
}

fn hat_add_equip_info_attributes(
    w: &World,
    id: ItemId,
    from: MobileId,
    attrs: &mut Vec<EquipInfoAttribute>,
) {
    add_equip_info_attributes_virtual(w, id, from, attrs);
    if is_shipwrecked(w, id) {
        attrs.push(EquipInfoAttribute::new(1041645, -1)); // recovered from a shipwreck
    }
}

fn on_single_click(w: &mut World, id: ItemId, from: MobileId) {
    if !core_uotd(w) {
        on_single_click_pre_uotd(w, id, from);
        return;
    }
    let Some((crafter, ty)) = data(w, id)
        .map(|d| d.crafter.clone())
        .zip(w.item(id).map(|i| i.ty))
    else {
        return;
    };
    let mut attrs = Vec::new();
    (class(w, id).add_equip_info_attributes)(w, id, from, &mut attrs);

    let name = ops::name(w, id);
    let number = match &name {
        None => (ty.label_number)(w, id),
        Some(n) => {
            ops::label_to(w, id, from, n);
            1041000
        }
    };
    if attrs.is_empty() && crafter.is_none() && name.is_some() {
        return;
    }
    let serial = w.item(id).map(|i| i.serial);
    if let (Some(ns), Some(serial)) = (deps::net_state_of(w, from), serial) {
        send_display_equipment_info(w, ns, serial, number, crafter.as_deref(), false, &attrs);
    }
}

/// `OnSingleClickPreUOTD(from)`.
pub fn on_single_click_pre_uotd(w: &mut World, id: ItemId, from: MobileId) {
    let Some((d, ty)) = data(w, id).cloned().zip(w.item(id).map(|i| i.ty)) else {
        return;
    };
    let name = ops::name(w, id).unwrap_or_else(|| {
        let article_an = ops::item_data(w, id)
            .flags
            .contains(muo_data::tile_data::TileFlag::ARTICLE_AN);
        let mut b = String::new();
        append_space_with_article(&mut b, &lower_text(w, (ty.label_number)(w, id)), article_an);
        b
    });
    let exceptional = d.quality == ClothingQuality::Exceptional;
    let text = match (&d.crafter, exceptional) {
        (None, true) => format!("{name} of exceptional quality"),
        (None, false) => name,
        (Some(c), true) => format!("{name} crafted with exceptional quality by {c}"),
        (Some(c), false) => format!("{name} crafted by {c}"),
    };
    ops::label_to(w, id, from, &text);
}

// ---- save layout (BaseClothing version 8) ----

const SERIALIZATION_VERSION: i32 = 8;

mod flag {
    pub const RESOURCE: u32 = 0x1;
    pub const ATTRIBUTES: u32 = 0x2;
    pub const CLOTHING_ATTRIBUTES: u32 = 0x4;
    pub const SKILL_BONUSES: u32 = 0x8;
    pub const RESISTANCES: u32 = 0x10;
    pub const MAX_HIT_POINTS: u32 = 0x20;
    pub const HIT_POINTS: u32 = 0x40;
    pub const CRAFTER: u32 = 0x80;
    pub const QUALITY: u32 = 0x100;
    pub const STR_REQUIREMENT: u32 = 0x200;

    // Versions 6 and 7 and the pre-codegen layout number the same fields with `PlayerConstructed` at 0x80.
    pub const OLD_PLAYER_CONSTRUCTED: u32 = 0x80;
    pub const OLD_CRAFTER: u32 = 0x100;
    pub const OLD_QUALITY: u32 = 0x200;
    pub const OLD_STR_REQUIREMENT: u32 = 0x400;
}

fn serialize(w: &World, id: ItemId, writer: &mut dyn GenericWriter) {
    (ITEM_BASE.serialize)(w, id, writer);
    let Some(d) = data(w, id) else { return };
    writer.write_i32(SERIALIZATION_VERSION);

    let default_resource = default_resource(w, id);
    let mut f = 0_u32;
    let mut set = |on: bool, bit: u32| {
        if on {
            f |= bit;
        }
    };
    set(d.resource != default_resource, flag::RESOURCE);
    set(!d.attributes.is_empty(), flag::ATTRIBUTES);
    set(!d.clothing_attributes.is_empty(), flag::CLOTHING_ATTRIBUTES);
    set(!d.skill_bonuses.is_empty(), flag::SKILL_BONUSES);
    set(!d.resistances.is_empty(), flag::RESISTANCES);
    set(d.max_hit_points != 0, flag::MAX_HIT_POINTS);
    set(d.hit_points != 0, flag::HIT_POINTS);
    set(
        d.crafter.as_deref().is_some_and(|s| !s.is_empty()),
        flag::CRAFTER,
    );
    set(d.quality != ClothingQuality::Regular, flag::QUALITY);
    set(d.str_req != -1, flag::STR_REQUIREMENT);
    writer.write_enum(4, i64::from(f));

    if f & flag::RESOURCE != 0 {
        writer.write_enum(4, d.resource as i64);
    }
    if f & flag::ATTRIBUTES != 0 {
        d.attributes.serialize(writer);
    }
    if f & flag::CLOTHING_ATTRIBUTES != 0 {
        d.clothing_attributes.serialize(writer);
    }
    if f & flag::SKILL_BONUSES != 0 {
        d.skill_bonuses.serialize(writer);
    }
    if f & flag::RESISTANCES != 0 {
        d.resistances.serialize(writer);
    }
    if f & flag::MAX_HIT_POINTS != 0 {
        writer.write_encoded_int(d.max_hit_points);
    }
    if f & flag::HIT_POINTS != 0 {
        writer.write_encoded_int(d.hit_points);
    }
    if f & flag::CRAFTER != 0 {
        writer.write_opt_string(d.crafter.as_deref());
    }
    if f & flag::QUALITY != 0 {
        writer.write_enum(4, d.quality as i64);
    }
    if f & flag::STR_REQUIREMENT != 0 {
        writer.write_i32(d.str_req);
    }
}

/// The three older layouts, which share their flag bits (`PlayerConstructed` at 0x80, then crafter, quality and
/// strength requirement).
struct Layout {
    /// Version 6 and the pre-codegen layout keep the crafter as a mobile, named on the next tick.
    crafter_is_mobile: bool,
    /// The pre-codegen layout writes the strength requirement as an encoded int, the others as an int.
    str_req_encoded: bool,
    /// `MigrateFrom(V6Content)` forgets `HitPoints`: the bytes are read, the value is dropped.
    drop_hit_points: bool,
}

const V6: Layout = Layout {
    crafter_is_mobile: true,
    str_req_encoded: false,
    drop_hit_points: true,
};
const V7: Layout = Layout {
    crafter_is_mobile: false,
    str_req_encoded: false,
    drop_hit_points: false,
};
const PRE_CODEGEN: Layout = Layout {
    crafter_is_mobile: true,
    str_req_encoded: true,
    drop_hit_points: false,
};

fn deserialize(w: &mut World, id: ItemId, reader: &mut dyn GenericReader) -> Result<()> {
    crate::templates::table(w); // `DefaultResource` reads the class props
    (ITEM_BASE.deserialize)(w, id, reader)?;
    let version = reader.read_i32()?;
    let layout = match version {
        6 => Some(&V6),
        7 => Some(&V7),
        v if v < SERIALIZATION_VERSION => Some(&PRE_CODEGEN),
        _ => None,
    };
    let mut d = ClothingData::for_load();
    let f = reader.read_enum(4)? as u32;
    let mut crafter_mobile = None;

    if let Some(layout) = layout {
        // Old layouts: a resource flagged or `DefaultResource`.
        d.resource = default_resource(w, id);
        if f & flag::RESOURCE != 0 {
            d.resource = read_enum(reader, CraftResource::from_i32, "CraftResource")?;
        }
        if f & flag::ATTRIBUTES != 0 {
            d.attributes = Attrs::deserialize(reader)?;
        }
        if f & flag::CLOTHING_ATTRIBUTES != 0 {
            d.clothing_attributes = Attrs::deserialize(reader)?;
        }
        if f & flag::SKILL_BONUSES != 0 {
            d.skill_bonuses = Attrs::deserialize(reader)?;
        }
        if f & flag::RESISTANCES != 0 {
            d.resistances = Attrs::deserialize(reader)?;
        }
        if f & flag::MAX_HIT_POINTS != 0 {
            d.max_hit_points = reader.read_encoded_int()?;
        }
        if f & flag::HIT_POINTS != 0 {
            let hp = reader.read_encoded_int()?;
            if !layout.drop_hit_points {
                d.hit_points = hp;
            }
        }
        if f & flag::OLD_CRAFTER != 0 {
            if layout.crafter_is_mobile {
                crafter_mobile = reader.read_mobile(w)?;
            } else {
                d.crafter = reader.read_string()?;
            }
        }
        if f & flag::OLD_QUALITY != 0 {
            d.quality = read_enum(reader, ClothingQuality::from_i32, "ClothingQuality")?;
        }
        if f & flag::OLD_STR_REQUIREMENT != 0 {
            d.str_req = if layout.str_req_encoded {
                reader.read_encoded_int()?
            } else {
                reader.read_i32()?
            };
        }
        let player_constructed = f & flag::OLD_PLAYER_CONSTRUCTED != 0;
        if let Some(i) = w.item_mut(id) {
            i.player_constructed = player_constructed;
        }
    } else {
        // Current layout. No `else` for the resource in the generated reader: a record without it keeps `None`.
        if f & flag::RESOURCE != 0 {
            d.resource = read_enum(reader, CraftResource::from_i32, "CraftResource")?;
        }
        if f & flag::ATTRIBUTES != 0 {
            d.attributes = Attrs::deserialize(reader)?;
        }
        if f & flag::CLOTHING_ATTRIBUTES != 0 {
            d.clothing_attributes = Attrs::deserialize(reader)?;
        }
        if f & flag::SKILL_BONUSES != 0 {
            d.skill_bonuses = Attrs::deserialize(reader)?;
        }
        if f & flag::RESISTANCES != 0 {
            d.resistances = Attrs::deserialize(reader)?;
        }
        if f & flag::MAX_HIT_POINTS != 0 {
            d.max_hit_points = reader.read_encoded_int()?;
        }
        if f & flag::HIT_POINTS != 0 {
            d.hit_points = reader.read_encoded_int()?;
        }
        if f & flag::CRAFTER != 0 {
            d.crafter = reader.read_string()?;
        }
        if f & flag::QUALITY != 0 {
            d.quality = read_enum(reader, ClothingQuality::from_i32, "ClothingQuality")?;
        }
        if f & flag::STR_REQUIREMENT != 0 {
            d.str_req = reader.read_i32()?;
        }
    }

    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(d);
    }
    if layout.is_some_and(|l| l.crafter_is_mobile) {
        // Timer.DelayCall((item, crafter) => item._crafter = crafter?.RawName, this, crafter)
        super::defer_crafter_name(w, id, crafter_mobile, |w, id, name| {
            if let Some(d) = data_mut(w, id) {
                d.crafter = name;
            }
        });
    }
    after_deserialization(w, id);
    Ok(())
}

/// `[AfterDeserialization]`.
fn after_deserialization(w: &mut World, id: ItemId) {
    if data(w, id).is_some_and(|d| d.max_hit_points == 0 && d.hit_points == 0) {
        let (min, max) = init_hits(w, id);
        let hits = random_min_max(w, min, max);
        if let Some(d) = data_mut(w, id) {
            d.hit_points = hits;
            d.max_hit_points = hits;
        }
    }
    if let Some(parent) = parent_mobile(w, id) {
        if core::aos(w) {
            super::skill_bonuses_add_to(w, id, parent);
        }
        add_stat_bonuses(w, id, parent);
        if let Some(ty) = w.mobile(parent).map(|m| m.ty) {
            (ty.check_stat_timers)(w, parent);
        }
    }
}

// ---- class chain ----

fn resistance(
    w: &World,
    id: ItemId,
    (key, base): (&str, fn(&ClothingClass) -> i32),
    e: AosElementAttribute,
) -> i32 {
    ip::int(w, id, key, base(class(w, id))) + resist(w, id, e)
}

/// `Server.Items.BaseClothing`.
pub static BASE_CLOTHING: ItemType = ItemType {
    full_name: "Server.Items.BaseClothing",
    parent: Some(&ITEM_BASE),
    construct: None,
    family: Some(&BASE_CLOTHING_CLASS),
    default_weight: |w, id| {
        ep::float_opt(w, id, "defaultWeight", class(w, id).default_weight)
            .unwrap_or_else(|| base::default_weight_base(w, id))
    },
    label_number: |w, id| match class_label_number(w, id) {
        0 => base::label_number_base(w, id),
        n => n,
    },
    required_races: |w, id| {
        ep::required_races(w, id, class(w, id).required_races)
            .unwrap_or_else(|| base::required_races_base(w, id))
    },
    physical_resistance: |w, id| {
        resistance(
            w,
            id,
            ("basePhysicalResistance", |c| c.base_physical_resistance),
            AosElementAttribute::Physical,
        )
    },
    fire_resistance: |w, id| {
        resistance(
            w,
            id,
            ("baseFireResistance", |c| c.base_fire_resistance),
            AosElementAttribute::Fire,
        )
    },
    cold_resistance: |w, id| {
        resistance(
            w,
            id,
            ("baseColdResistance", |c| c.base_cold_resistance),
            AosElementAttribute::Cold,
        )
    },
    poison_resistance: |w, id| {
        resistance(
            w,
            id,
            ("basePoisonResistance", |c| c.base_poison_resistance),
            AosElementAttribute::Poison,
        )
    },
    energy_resistance: |w, id| {
        resistance(
            w,
            id,
            ("baseEnergyResistance", |c| c.base_energy_resistance),
            AosElementAttribute::Energy,
        )
    },
    on_after_duped,
    on_added,
    on_removed,
    allow_secure_trade,
    can_equip,
    check_property_conflict,
    allow_equipped_cast,
    add_name_property,
    get_properties,
    on_single_click,
    serialize,
    deserialize,
    ..ITEM_BASE
};

/// Declares an abstract clothing class that only fixes its layer: `[SerializationGenerator(0, false)]`, no fields
/// of its own.
macro_rules! clothing_base {
    ($(#[$m:meta])* $name:ident, $class:ident = $full:literal, $layer:expr $(, $f:ident : $v:expr)*) => {
        /// The class table.
        pub static $class: ClothingClass = ClothingClass { layer: $layer, $($f: $v,)* ..BASE_CLOTHING_CLASS };
        $(#[$m])*
        pub static $name: ItemType = ItemType {
            full_name: $full,
            parent: Some(&BASE_CLOTHING),
            family: Some(&$class),
            serialize: |w, id, out| {
                (BASE_CLOTHING.serialize)(w, id, out);
                out.write_i32(0);
            },
            deserialize: |w, id, r| {
                (BASE_CLOTHING.deserialize)(w, id, r)?;
                let _version = r.read_i32()?;
                Ok(())
            },
            ..BASE_CLOTHING
        };
    };
}

clothing_base!(
    /// `Server.Items.BaseCloak`.
    BASE_CLOAK, BASE_CLOAK_CLASS = "Server.Items.BaseCloak", Layer::Cloak
);
clothing_base!(
    /// `Server.Items.BaseMiddleTorso`.
    BASE_MIDDLE_TORSO, BASE_MIDDLE_TORSO_CLASS = "Server.Items.BaseMiddleTorso", Layer::MiddleTorso
);
clothing_base!(
    /// `Server.Items.BaseOuterLegs`.
    BASE_OUTER_LEGS, BASE_OUTER_LEGS_CLASS = "Server.Items.BaseOuterLegs", Layer::OuterLegs
);
clothing_base!(
    /// `Server.Items.BaseOuterTorso`.
    BASE_OUTER_TORSO, BASE_OUTER_TORSO_CLASS = "Server.Items.BaseOuterTorso", Layer::OuterTorso
);
clothing_base!(
    /// `Server.Items.BasePants`.
    BASE_PANTS, BASE_PANTS_CLASS = "Server.Items.BasePants", Layer::Pants
);
clothing_base!(
    /// `Server.Items.BaseShirt`.
    BASE_SHIRT, BASE_SHIRT_CLASS = "Server.Items.BaseShirt", Layer::Shirt
);
clothing_base!(
    /// `Server.Items.BaseShoes` (only cloth shoes can be cut with scissors).
    BASE_SHOES, BASE_SHOES_CLASS = "Server.Items.BaseShoes", Layer::Shoes, scissor: shoes_scissor
);
clothing_base!(
    /// `Server.Items.BaseWaist`.
    BASE_WAIST, BASE_WAIST_CLASS = "Server.Items.BaseWaist", Layer::Waist
);

/// `BaseHat`: a flag for hats saved from a shipwreck, an extra equip info line and name property, and an
/// `OnCraft` that hands out the exceptional resistances first.
pub static BASE_HAT_CLASS: ClothingClass = ClothingClass {
    layer: Layer::Helm,
    on_craft: hat_on_craft,
    add_equip_info_attributes: hat_add_equip_info_attributes,
    ..BASE_CLOTHING_CLASS
};

/// `Server.Items.BaseHat`.
pub static BASE_HAT: ItemType = ItemType {
    full_name: "Server.Items.BaseHat",
    parent: Some(&BASE_CLOTHING),
    family: Some(&BASE_HAT_CLASS),
    add_name_properties: |w, id, list| {
        (BASE_CLOTHING.add_name_properties)(w, id, list);
        if is_shipwrecked(w, id) {
            list.add(1041645); // recovered from a shipwreck
        }
    },
    serialize: |w, id, out| {
        (BASE_CLOTHING.serialize)(w, id, out);
        out.write_i32(0);
        out.write_bool(is_shipwrecked(w, id));
    },
    deserialize: |w, id, r| {
        (BASE_CLOTHING.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        let shipwrecked = r.read_bool()?;
        if let Some(d) = data_mut(w, id) {
            d.is_shipwrecked = shipwrecked;
        }
        Ok(())
    },
    ..BASE_CLOTHING
};
