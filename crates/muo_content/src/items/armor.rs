//! `BaseArmor` (current layout, version 10) and its migrations, the armor enums and `BaseShield`.
//!
//! State is [`ArmorData`] in `Item::ext`; what a leaf's `override`s returned is [`ArmorClass`].
//! The virtuals of `BaseArmor` that a subclass can override (`ArmorRating`, `ArmorScalar`,
//! `OnHit`, `OnCraft`, `Scissor`, `GetLuckBonus`) are `fn` fields of [`ArmorClass`]: that table is the family
//! vtable, and a shield replaces two of them.
//!
//! Systems in other crates are hooks ([`super::equip_hooks`]): ethics (`CanEquip`, trading, scissors), faction
//! items, the tailoring recipes `Scissor` reads, runic tools, `CraftItem.RetainsColor`.
//!
//! Saves: version 10 is the current layout; versions 8 and 9 (`V8Content`, `V9Content`) and everything older (the
//! pre-codegen `Deserialize(reader, version)`) are read byte-compatibly, including the quirks of those old writers (see
//! [`Layout`]).

use muo_io::{GenericReader, GenericWriter, Result};
use muo_server::World;
use muo_server::ext::Ext;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::base;
use muo_server::item::deps;
use muo_server::item::ops;
use muo_server::item::{ITEM_BASE, ItemType};
use muo_server::layer::Layer;
use muo_server::mobile::{AccessLevel, MobileDelta, StatType};
use muo_server::packets::equipment::{EquipInfoAttribute, send_display_equipment_info};
use muo_server::property_list::PropertyList;
use muo_server::serialization_ext::ReadEntityExt;
use muo_server::skills::SkillName;

use super::equip_hooks::{CraftArgs, hooks, lower_text};
use super::weapon::{self, WeaponType};
use super::{Wear, core_uor, core_uotd, parent_mobile, random_min_max, read_enum};
use crate::aos::{
    AosArmorAttribute, AosAttribute, AttrKind, Attrs, add_armor_attribute_properties,
    add_attribute_properties, add_skill_bonus_properties,
};
use crate::core;
use crate::craft::{self, CraftResource};
use crate::hooks::rnd;
use crate::misc::aos_helpers;
use crate::templates::{self, item_props as ip};

int_enum! {
    /// `ArmorQuality`.
    pub enum ArmorQuality { Low = 0, Regular = 1, Exceptional = 2 }
}

int_enum! {
    /// `ArmorDurabilityLevel`.
    pub enum ArmorDurabilityLevel {
        Regular = 0, Durable = 1, Substantial = 2, Massive = 3, Fortified = 4, Indestructible = 5,
    }
}

int_enum! {
    /// `ArmorProtectionLevel`.
    pub enum ArmorProtectionLevel {
        Regular = 0, Defense = 1, Guarding = 2, Hardening = 3, Fortification = 4, Invulnerability = 5,
    }
}

int_enum! {
    /// `ArmorBodyType`.
    pub enum ArmorBodyType { Gorget = 0, Gloves = 1, Helmet = 2, Arms = 3, Legs = 4, Chest = 5, Shield = 6 }
}

int_enum! {
    /// `ArmorMaterialType`.
    pub enum ArmorMaterialType {
        Cloth = 0, Leather = 1, Studded = 2, Bone = 3, Spined = 4, Horned = 5, Barbed = 6, Ringmail = 7,
        Chainmail = 8, Plate = 9, Dragon = 10, Wood = 11, Stone = 12,
    }
}

int_enum! {
    /// `ArmorMeditationAllowance`.
    pub enum ArmorMeditationAllowance { All = 0, Half = 1, None = 2 }
}

/// `BaseArmor.ArmorScalars`.
// shortcut: the scalars were once settable; nothing sets them, so it is a constant until a config needs it.
pub const ARMOR_SCALARS: [f64; 6] = [0.07, 0.07, 0.14, 0.15, 0.22, 0.35];

/// The `override`s of an armor class: its constants, and the virtuals that are code. The stat fields (resistances,
/// hits, requirements, bonuses, armor base) are fallbacks for the class's template `props`. A subclass is
/// `ArmorClass { .. , ..PARENT }`.
#[derive(Copy, Clone)]
pub struct ArmorClass {
    pub default_weight: Option<f64>,
    /// `LabelNumber` when overridden (0 keeps the tiledata name).
    pub label_number: i32,
    /// `RequiredRaces` when overridden (`Race.AllowElvesOnly`, ..).
    pub required_races: Option<i32>,
    pub artifact_rarity: i32,
    pub base_physical_resistance: i32,
    pub base_fire_resistance: i32,
    pub base_cold_resistance: i32,
    pub base_poison_resistance: i32,
    pub base_energy_resistance: i32,
    pub init_min_hits: i32,
    pub init_max_hits: i32,
    pub aos_str_req: i32,
    pub aos_dex_req: i32,
    pub aos_int_req: i32,
    pub old_str_req: i32,
    pub old_dex_req: i32,
    pub old_int_req: i32,
    pub aos_str_bonus: i32,
    pub aos_dex_bonus: i32,
    pub aos_int_bonus: i32,
    pub old_str_bonus: i32,
    pub old_dex_bonus: i32,
    pub old_int_bonus: i32,
    pub armor_base: i32,
    /// `RevertArmorBase` when overridden; else `armor_base`.
    pub revert_armor_base: Option<i32>,
    pub material_type: ArmorMaterialType,
    pub default_resource: CraftResource,
    /// `DefMedAllowance` (`AosMedAllowance` and `OldMedAllowance` are the same in every class).
    pub def_med_allowance: ArmorMeditationAllowance,
    pub allow_male_wearer: bool,
    pub allow_female_wearer: bool,
    pub can_fortify: bool,
    /// `[Flippable(a, b)]`.
    pub flippable: Option<(i32, i32)>,
    /// `ArmorRating`.
    pub armor_rating: fn(&World, ItemId) -> f64,
    /// `ArmorScalar`.
    pub armor_scalar: fn(&World, ItemId) -> f64,
    /// `GetLuckBonus()`.
    pub get_luck_bonus: fn(&World, ItemId) -> i32,
    /// `OnHit(weapon, damage)`: `(world, armor, weapon, damage)`, returns the damage that gets through.
    pub on_hit: fn(&mut World, ItemId, ItemId, i32) -> i32,
    /// `OnCraft(..)`.
    pub on_craft: fn(&mut World, ItemId, &CraftArgs) -> i32,
    /// `Scissor(from, scissors)`.
    pub scissor: fn(&mut World, ItemId, MobileId) -> bool,
}

/// `BaseArmor`'s defaults. (`MaterialType` has no default; every leaf sets it.)
pub static BASE_ARMOR_CLASS: ArmorClass = ArmorClass {
    default_weight: None,
    label_number: 0,
    required_races: None,
    artifact_rarity: 0,
    base_physical_resistance: 0,
    base_fire_resistance: 0,
    base_cold_resistance: 0,
    base_poison_resistance: 0,
    base_energy_resistance: 0,
    init_min_hits: 0,
    init_max_hits: 0,
    aos_str_req: 0,
    aos_dex_req: 0,
    aos_int_req: 0,
    old_str_req: 0,
    old_dex_req: 0,
    old_int_req: 0,
    aos_str_bonus: 0,
    aos_dex_bonus: 0,
    aos_int_bonus: 0,
    old_str_bonus: 0,
    old_dex_bonus: 0,
    old_int_bonus: 0,
    armor_base: 0,
    revert_armor_base: None,
    material_type: ArmorMaterialType::Plate,
    default_resource: CraftResource::Iron,
    def_med_allowance: ArmorMeditationAllowance::None,
    allow_male_wearer: true,
    allow_female_wearer: true,
    can_fortify: true,
    flippable: None,
    armor_rating: base_armor_rating_virtual,
    armor_scalar: armor_scalar_virtual,
    get_luck_bonus: get_luck_bonus_virtual,
    on_hit: on_hit_virtual,
    on_craft: on_craft_virtual,
    scissor: scissor_virtual,
};

/// Everything `BaseArmor` adds to `Item`.
#[derive(Clone, Debug)]
pub struct ArmorData {
    pub attributes: Attrs,
    pub armor_attributes: Attrs,
    pub skill_bonuses: Attrs,
    pub physical_bonus: i32,
    pub fire_bonus: i32,
    pub cold_bonus: i32,
    pub poison_bonus: i32,
    pub energy_bonus: i32,
    pub identified: bool,
    pub hit_points: i32,
    pub max_hit_points: i32,
    pub crafter: Option<String>,
    pub quality: ArmorQuality,
    pub durability: ArmorDurabilityLevel,
    pub protection_level: ArmorProtectionLevel,
    pub resource: CraftResource,
    /// -1 means the class's value.
    pub armor_base: i32,
    pub str_bonus: i32,
    pub dex_bonus: i32,
    pub int_bonus: i32,
    pub str_req: i32,
    pub dex_req: i32,
    pub int_req: i32,
    /// `_meditate`: -1 (below `All`) means the class's `DefMedAllowance`.
    pub meditate: i32,
    /// `ElvenGlasses._weaponAttributes` (only that family has it).
    pub weapon_attributes: Attrs,
    /// `_curArcaneCharges`, `_maxArcaneCharges` of the arcane gloves.
    pub arcane_charges: (i32, i32),
}

impl ArmorData {
    /// The field initializers of `BaseArmor` as `new BaseArmor(Serial)` leaves them (the loader constructor).
    fn for_load(default_resource: CraftResource) -> ArmorData {
        ArmorData {
            attributes: Attrs::default(),
            armor_attributes: Attrs::default(),
            skill_bonuses: Attrs::default(),
            physical_bonus: 0,
            fire_bonus: 0,
            cold_bonus: 0,
            poison_bonus: 0,
            energy_bonus: 0,
            identified: false,
            hit_points: 0,
            max_hit_points: 0,
            crafter: None,
            quality: ArmorQuality::Regular,
            durability: ArmorDurabilityLevel::Regular,
            protection_level: ArmorProtectionLevel::Regular,
            resource: default_resource,
            armor_base: -1,
            str_bonus: -1,
            dex_bonus: -1,
            int_bonus: -1,
            str_req: -1,
            dex_req: -1,
            int_req: -1,
            meditate: -1,
            weapon_attributes: Attrs::default(),
            arcane_charges: (0, 0),
        }
    }
}

/// The armor state of an item.
pub fn data(w: &World, id: ItemId) -> Option<&ArmorData> {
    w.item(id)?.ext.get()
}

/// The armor state of an item, mutable.
pub fn data_mut(w: &mut World, id: ItemId) -> Option<&mut ArmorData> {
    w.item_mut(id)?.ext.get_mut()
}

/// The constants of the item's class.
pub fn class(w: &World, id: ItemId) -> &'static ArmorClass {
    w.item(id)
        .and_then(|i| i.ty.family::<ArmorClass>())
        .unwrap_or(&BASE_ARMOR_CLASS)
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

/// `item is BaseArmor`.
pub fn is_armor(w: &World, id: ItemId) -> bool {
    w.item(id).is_some_and(|i| i.ty.is(&BASE_ARMOR))
}

/// `item is BaseShield`.
pub fn is_shield(w: &World, id: ItemId) -> bool {
    w.item(id).is_some_and(|i| i.ty.is(&BASE_SHIELD))
}

/// `new BaseArmor(itemID)` for class `ty`; every leaf constructor starts here.
pub fn new_armor(w: &mut World, ty: &'static ItemType, item_id: i32) -> ItemId {
    let id = ops::new_item_with_id(w, ty, item_id);
    let c = ty.family::<ArmorClass>().unwrap_or(&BASE_ARMOR_CLASS);
    templates::table(w); // the stat props below and in combat read the loaded table
    let (min, max) = (
        ip::int(w, id, "initMinHits", c.init_min_hits),
        ip::int(w, id, "initMaxHits", c.init_max_hits),
    );
    let hits = random_min_max(w, min, max);
    let mut d = ArmorData::for_load(c.default_resource);
    d.hit_points = hits;
    d.max_hit_points = hits;
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(d);
    }
    // Hue = CraftResources.GetHue(_resource)
    let hue = craft::hue(w, c.default_resource);
    (ty.set_hue)(w, id, hue);
    // Layer = (Layer)ItemData.Quality
    let quality = ops::item_data(w, id).quality;
    (ty.set_layer)(w, id, Layer::from_raw(quality));
    id
}

// ---- getters ----

fn aos_or_old(w: &World, set: i32, aos: i32, old: i32) -> i32 {
    if set != -1 {
        set
    } else if core::aos(w) {
        aos
    } else {
        old
    }
}

/// `BaseArmorRating`.
pub fn base_armor_rating(w: &World, id: ItemId) -> i32 {
    let set = data(w, id).map_or(-1, |d| d.armor_base);
    if set == -1 {
        ip::int(w, id, "armorBase", class(w, id).armor_base)
    } else {
        set
    }
}

/// `StrBonus`.
pub fn str_bonus(w: &World, id: ItemId) -> i32 {
    let c = class(w, id);
    aos_or_old(
        w,
        data(w, id).map_or(-1, |d| d.str_bonus),
        ip::int(w, id, "aosStrBonus", c.aos_str_bonus),
        ip::int(w, id, "oldStrBonus", c.old_str_bonus),
    )
}

/// `DexBonus`.
pub fn dex_bonus(w: &World, id: ItemId) -> i32 {
    let c = class(w, id);
    aos_or_old(
        w,
        data(w, id).map_or(-1, |d| d.dex_bonus),
        ip::int(w, id, "aosDexBonus", c.aos_dex_bonus),
        ip::int(w, id, "oldDexBonus", c.old_dex_bonus),
    )
}

/// `IntBonus`.
pub fn int_bonus(w: &World, id: ItemId) -> i32 {
    let c = class(w, id);
    aos_or_old(
        w,
        data(w, id).map_or(-1, |d| d.int_bonus),
        ip::int(w, id, "aosIntBonus", c.aos_int_bonus),
        ip::int(w, id, "oldIntBonus", c.old_int_bonus),
    )
}

/// `StrRequirement`.
pub fn str_requirement(w: &World, id: ItemId) -> i32 {
    let c = class(w, id);
    aos_or_old(
        w,
        data(w, id).map_or(-1, |d| d.str_req),
        ip::int(w, id, "aosStrReq", c.aos_str_req),
        ip::int(w, id, "oldStrReq", c.old_str_req),
    )
}

/// `DexRequirement`.
pub fn dex_requirement(w: &World, id: ItemId) -> i32 {
    let c = class(w, id);
    aos_or_old(
        w,
        data(w, id).map_or(-1, |d| d.dex_req),
        ip::int(w, id, "aosDexReq", c.aos_dex_req),
        ip::int(w, id, "oldDexReq", c.old_dex_req),
    )
}

/// `IntRequirement`.
pub fn int_requirement(w: &World, id: ItemId) -> i32 {
    let c = class(w, id);
    aos_or_old(
        w,
        data(w, id).map_or(-1, |d| d.int_req),
        ip::int(w, id, "aosIntReq", c.aos_int_req),
        ip::int(w, id, "oldIntReq", c.old_int_req),
    )
}

/// `MeditationAllowance`.
pub fn meditation_allowance(w: &World, id: ItemId) -> ArmorMeditationAllowance {
    let set = data(w, id).map_or(-1, |d| d.meditate);
    ArmorMeditationAllowance::from_i32(set).unwrap_or(class(w, id).def_med_allowance)
}

/// `BodyPosition`, from the layer.
pub fn body_position(w: &World, id: ItemId) -> ArmorBodyType {
    match w.item(id).map_or(Layer::Invalid, |i| i.layer) {
        Layer::Neck => ArmorBodyType::Gorget,
        Layer::TwoHanded => ArmorBodyType::Shield,
        Layer::Gloves => ArmorBodyType::Gloves,
        Layer::Helm => ArmorBodyType::Helmet,
        Layer::Arms => ArmorBodyType::Arms,
        Layer::InnerLegs | Layer::OuterLegs | Layer::Pants => ArmorBodyType::Legs,
        Layer::InnerTorso | Layer::OuterTorso | Layer::Shirt => ArmorBodyType::Chest,
        _ => ArmorBodyType::Gorget,
    }
}

fn armor_scalar_virtual(w: &World, id: ItemId) -> f64 {
    ARMOR_SCALARS
        .get(body_position(w, id) as usize)
        .copied()
        .unwrap_or(1.0)
}

/// `ArmorScalar`.
pub fn armor_scalar(w: &World, id: ItemId) -> f64 {
    (class(w, id).armor_scalar)(w, id)
}

/// `ScaleArmorByDurability(armor)`.
pub fn scale_armor_by_durability(w: &World, id: ItemId, armor: f64) -> f64 {
    let mut scale = 100;
    if let Some(d) = data(w, id)
        && d.max_hit_points > 0
        && d.hit_points < d.max_hit_points
    {
        scale = 50 + 50 * d.hit_points / d.max_hit_points;
    }
    armor * f64::from(scale) / 100.0
}

/// `BaseArmor.ArmorRating` (what `base.ArmorRating` is for a shield).
fn base_armor_rating_virtual(w: &World, id: ItemId) -> f64 {
    let Some(d) = data(w, id) else { return 0.0 };
    let mut ar = base_armor_rating(w, id);
    if d.protection_level != ArmorProtectionLevel::Regular {
        if core_uor(w) {
            ar += 10;
        }
        ar += 5 * d.protection_level as i32;
    }
    // Colored armor does not give a bonus until UOR+
    if core_uor(w) {
        ar += match d.resource {
            CraftResource::DullCopper => 2,
            CraftResource::ShadowIron => 4,
            CraftResource::Copper => 6,
            CraftResource::Bronze => 8,
            CraftResource::Gold => 10,
            CraftResource::Agapite => 12,
            CraftResource::Verite => 14,
            CraftResource::Valorite => 16,
            CraftResource::SpinedLeather => 10,
            CraftResource::HornedLeather => 13,
            CraftResource::BarbedLeather => 16,
            _ => 0,
        };
    }
    ar += 8 * (d.quality as i32 - 1);
    scale_armor_by_durability(w, id, f64::from(ar))
}

/// `ArmorRating` (virtual: a shield adds the wearer's Parry).
pub fn armor_rating(w: &World, id: ItemId) -> f64 {
    (class(w, id).armor_rating)(w, id)
}

/// `ArmorRatingScaled`.
pub fn armor_rating_scaled(w: &World, id: ItemId) -> f64 {
    armor_rating(w, id) * armor_scalar(w, id)
}

/// `BaseArmorRatingScaled`.
pub fn base_armor_rating_scaled(w: &World, id: ItemId) -> f64 {
    f64::from(base_armor_rating(w, id)) * armor_scalar(w, id)
}

/// `GetProtOffset()`.
pub fn prot_offset(w: &World, id: ItemId) -> i32 {
    match data(w, id).map(|d| d.protection_level) {
        Some(ArmorProtectionLevel::Guarding) => 1,
        Some(ArmorProtectionLevel::Hardening) => 2,
        Some(ArmorProtectionLevel::Fortification) => 3,
        Some(ArmorProtectionLevel::Invulnerability) => 4,
        _ => 0,
    }
}

/// `GetResourceAttrs()`.
pub fn resource_attrs(w: &World, id: ItemId) -> craft::CraftAttr {
    data(w, id)
        .and_then(|d| craft::attribute_info(d.resource))
        .unwrap_or_default()
}

fn resistance(
    w: &World,
    id: ItemId,
    key: &str,
    base: fn(&ArmorClass) -> i32,
    res: fn(&craft::CraftAttr) -> i32,
    bonus: fn(&ArmorData) -> i32,
) -> i32 {
    ip::int(w, id, key, base(class(w, id)))
        + prot_offset(w, id)
        + res(&resource_attrs(w, id))
        + data(w, id).map_or(0, bonus)
}

fn get_luck_bonus_virtual(w: &World, id: ItemId) -> i32 {
    data(w, id)
        .and_then(|d| craft::attribute_info(d.resource))
        .map_or(0, |a| a.armor_luck)
}

/// `GetLuckBonus()`.
pub fn get_luck_bonus(w: &World, id: ItemId) -> i32 {
    (class(w, id).get_luck_bonus)(w, id)
}

/// `GetDurabilityBonus()`.
pub fn durability_bonus(w: &World, id: ItemId) -> i32 {
    let Some(d) = data(w, id) else { return 0 };
    if !core_uor(w) {
        return d.durability as i32 * 5 + (d.quality as i32 - 1) * 10;
    }
    let mut bonus = match d.durability {
        ArmorDurabilityLevel::Durable => 20,
        ArmorDurabilityLevel::Substantial => 50,
        ArmorDurabilityLevel::Massive => 70,
        ArmorDurabilityLevel::Fortified => 100,
        ArmorDurabilityLevel::Indestructible => 120,
        ArmorDurabilityLevel::Regular => 0,
    };
    if core::aos(w) {
        bonus += d
            .armor_attributes
            .get(true, AosArmorAttribute::DurabilityBonus as u32)
            + craft::attribute_info(d.resource).map_or(0, |a| a.armor_durability);
    }
    if d.quality == ArmorQuality::Exceptional {
        bonus + 20
    } else {
        bonus
    }
}

/// `GetLowerStatReq()`.
pub fn lower_stat_req(w: &World, id: ItemId) -> i32 {
    if !core::aos(w) {
        return 0;
    }
    let Some(d) = data(w, id) else { return 0 };
    let mut v = d
        .armor_attributes
        .get(true, AosArmorAttribute::LowerStatReq as u32);
    if let Some(a) = craft::attribute_info(d.resource) {
        v += a.armor_lower_requirements;
    }
    v.min(100)
}

/// `ComputeStatReq(type)`.
pub fn compute_stat_req(w: &World, id: ItemId, stat: StatType) -> i32 {
    let v = match stat {
        StatType::STR => str_requirement(w, id),
        StatType::DEX => dex_requirement(w, id),
        _ => int_requirement(w, id),
    };
    aos_helpers::scale(v, 100 - lower_stat_req(w, id))
}

/// `ComputeStatBonus(type)`.
pub fn compute_stat_bonus(w: &World, id: ItemId, stat: StatType) -> i32 {
    let a = |x: AosAttribute| super::attr(w, id, AttrKind::Attributes, x as u32);
    match stat {
        StatType::STR => str_bonus(w, id) + a(AosAttribute::BonusStr),
        StatType::DEX => dex_bonus(w, id) + a(AosAttribute::BonusDex),
        _ => int_bonus(w, id) + a(AosAttribute::BonusInt),
    }
}

/// `UnscaleDurability()`.
pub fn unscale_durability(w: &mut World, id: ItemId) {
    let scale = 100 + durability_bonus(w, id);
    if let Some(d) = data_mut(w, id) {
        d.max_hit_points = (d.max_hit_points * 100 + (scale - 1)) / scale;
        d.hit_points = (d.hit_points * 100 + (scale - 1)) / scale;
    }
    ops::invalidate_properties(w, id);
}

/// `ScaleDurability()`.
pub fn scale_durability(w: &mut World, id: ItemId) {
    let scale = 100 + durability_bonus(w, id);
    if let Some(d) = data_mut(w, id) {
        d.max_hit_points = (d.max_hit_points * scale + 99) / 100;
        d.hit_points = (d.hit_points * scale + 99) / 100;
    }
    ops::invalidate_properties(w, id);
}

// ---- setters with side effects ----

/// `Invalidate()`: tells the wearer their armor rating changed.
fn invalidate(w: &mut World, id: ItemId) {
    if let Some(m) = parent_mobile(w, id)
        && let Some(ty) = w.mobile(m).map(|m| m.ty)
    {
        (ty.delta)(w, m, MobileDelta::ARMOR);
    }
}

/// `(Parent as Mobile)?.UpdateResistances()`.
fn update_wearer_resistances(w: &mut World, id: ItemId) {
    if let Some(m) = parent_mobile(w, id)
        && let Some(ty) = w.mobile(m).map(|m| m.ty)
    {
        (ty.update_resistances)(w, m);
    }
}

/// `Quality` setter.
pub fn set_quality(w: &mut World, id: ItemId, value: ArmorQuality) {
    unscale_durability(w, id);
    if let Some(d) = data_mut(w, id) {
        d.quality = value;
    }
    scale_durability(w, id);
}

/// `Durability` setter.
pub fn set_durability_level(w: &mut World, id: ItemId, value: ArmorDurabilityLevel) {
    unscale_durability(w, id);
    if let Some(d) = data_mut(w, id) {
        d.durability = value;
    }
    scale_durability(w, id);
}

/// `ProtectionLevel` setter (`OnProtectionLevelChanged`).
pub fn set_protection_level(w: &mut World, id: ItemId, value: ArmorProtectionLevel) {
    let Some(d) = data_mut(w, id) else { return };
    if d.protection_level == value {
        return;
    }
    d.protection_level = value;
    ops::invalidate_properties(w, id);
    invalidate(w, id);
    update_wearer_resistances(w, id);
}

/// `BaseArmorRating` setter.
pub fn set_base_armor_rating(w: &mut World, id: ItemId, value: i32) {
    if let Some(d) = data_mut(w, id) {
        d.armor_base = value;
    }
    invalidate(w, id);
}

/// `Resource` setter.
pub fn set_resource(w: &mut World, id: ItemId, value: CraftResource) {
    if data(w, id).is_none_or(|d| d.resource == value) {
        return;
    }
    unscale_durability(w, id);
    if let Some(d) = data_mut(w, id) {
        d.resource = value;
    }
    if let Some(ty) = w.item(id).map(|i| i.ty) {
        // CraftItem.RetainsColor(GetType())
        let retains = hooks(w).and_then(|h| h.retains_color).is_none_or(|f| f(ty));
        if retains {
            let hue = craft::hue(w, value);
            (ty.set_hue)(w, id, hue);
        }
    }
    invalidate(w, id);
    update_wearer_resistances(w, id);
    scale_durability(w, id);
}

/// `HitPoints` setter: ignored without `MaxHitPoints`; below zero the armor is destroyed.
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
        return;
    }
    d.hit_points = value.min(d.max_hit_points);
    ops::invalidate_properties(w, id);
}

/// `MaxHitPoints` setter.
pub fn set_max_hit_points(w: &mut World, id: ItemId, value: i32) {
    if let Some(d) = data_mut(w, id) {
        d.max_hit_points = value;
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

/// `Identified` setter.
pub fn set_identified(w: &mut World, id: ItemId, value: bool) {
    if let Some(d) = data_mut(w, id) {
        d.identified = value;
    }
}

/// `++PhysicalBonus`, `++FireBonus`, ..: element 0..5 in the order of the `switch (Utility.Random(5))` of
/// `DistributeBonuses`: Physical, Fire, Cold, Poison, Energy.
fn bump_bonus(w: &mut World, id: ItemId, element: i32) {
    if let Some(d) = data_mut(w, id) {
        match element {
            0 => d.physical_bonus += 1,
            1 => d.fire_bonus += 1,
            2 => d.cold_bonus += 1,
            3 => d.poison_bonus += 1,
            _ => d.energy_bonus += 1,
        }
    }
    ops::invalidate_properties(w, id);
}

/// `DistributeBonuses(amount)`.
pub fn distribute_bonuses(w: &mut World, id: ItemId, amount: i32) {
    for _ in 0..amount {
        let element = rnd::below(w, 5);
        bump_bonus(w, id, element);
    }
    ops::invalidate_properties(w, id);
}

// ---- crafting, scissors, combat ----

fn on_craft_virtual(w: &mut World, id: ItemId, a: &CraftArgs) -> i32 {
    // shortcut: a quality number the craft system never passes (not 0..=2) reads as Regular.
    set_quality(
        w,
        id,
        ArmorQuality::from_i32(a.quality).unwrap_or(ArmorQuality::Regular),
    );
    if a.makers_mark {
        let name = muo_server::mobile::part1::props_b::raw_name(w, a.from);
        set_crafter(w, id, name);
    }
    set_resource(w, id, a.resource);
    set_identified(w, id, true);

    let Some(ty) = w.item(id).map(|i| i.ty) else {
        return a.quality;
    };
    if a.do_not_color {
        (ty.set_hue)(w, id, 0);
    }

    let exceptional = data(w, id).is_some_and(|d| d.quality == ArmorQuality::Exceptional);
    let shield = is_shield(w, id);
    if exceptional {
        // Guessed Core.ML removed exceptional resist bonuses from crafted shields
        if !(core::ml(w) && shield) {
            // Not sure since when, but right now 15 points are added, not 14.
            let amount = if a.runic_tool.is_some() {
                6
            } else if core::se(w) {
                15
            } else {
                14
            };
            distribute_bonuses(w, id, amount);
        }
        if core::ml(w) && !shield {
            let bonus = (muo_server::skills::value(w, a.from, SkillName::ArmsLore) / 20.0) as i32;
            for _ in 0..bonus {
                // case 3 is EnergyBonus and case 4 PoisonBonus here (the reverse of DistributeBonuses)
                let element = match rnd::below(w, 5) {
                    3 => 4,
                    4 => 3,
                    n => n,
                };
                bump_bonus(w, id, element);
            }
            muo_server::mobile::part2::stats::check_skill(
                w,
                a.from,
                SkillName::ArmsLore,
                0.0,
                100.0,
            );
        }
    }

    if core::aos(w)
        && let Some(tool) = a.runic_tool
        && let Some(f) = hooks(w).and_then(|h| h.runic_apply_attributes)
    {
        f(w, tool, id);
    }
    a.quality
}

/// `OnCraft(..)`.
pub fn on_craft(w: &mut World, id: ItemId, a: &CraftArgs) -> i32 {
    (class(w, id).on_craft)(w, id, a)
}

fn scissor_virtual(w: &mut World, id: ItemId, from: MobileId) -> bool {
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
        && let Some((res, amount)) = f(w, ty, resource, false)
    {
        let per = if w.item(id).is_some_and(|i| i.player_constructed) {
            amount / 2
        } else {
            1
        };
        (ty.scissor_helper)(w, id, from, res, per, true);
        return true;
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

fn self_repair(w: &World, id: ItemId) -> i32 {
    super::attr(w, id, AttrKind::Armor, AosArmorAttribute::SelfRepair as u32)
}

fn on_hit_virtual(w: &mut World, id: ItemId, weapon_id: ItemId, damage: i32) -> i32 {
    let half = armor_rating(w, id) / 2.0;
    let absorbed = (half + half * rnd::double(w)) as i32;
    let damage = (damage - absorbed).max(0);

    // 25% chance to lower durability
    if rnd::below(w, 4) == 0 {
        if core::aos(w) && self_repair(w, id) > rnd::below(w, 10) {
            let hp = data(w, id).map_or(0, |d| d.hit_points);
            set_hit_points(w, id, hp + 2);
        } else {
            let wear = if weapon::weapon_type(w, weapon_id) == WeaponType::Bashing {
                (absorbed / 2).max(1)
            } else {
                rnd::below(w, 2)
            };
            WEAR.apply(w, id, wear);
        }
    }
    damage
}

/// `OnHit(weapon, damageTaken)`: the damage left after the armor (and the shield, for one).
pub fn on_hit(w: &mut World, id: ItemId, weapon_id: ItemId, damage: i32) -> i32 {
    (class(w, id).on_hit)(w, id, weapon_id, damage)
}

// ---- vtable entries ----

fn set_hue(w: &mut World, id: ItemId, value: i32) {
    base::set_hue_base(w, id, value);
    ops::invalidate_properties(w, id);
}

/// `OnAfterDuped`.
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
    d.armor_attributes = src.armor_attributes.clone();
    d.skill_bonuses = src.skill_bonuses.clone();
    // Set hue again because of resource
    (ty.set_hue)(w, new_item, hue);
    // Set HP/Max again because of durability
    set_hit_points(w, new_item, src.hit_points);
    set_max_hit_points(w, new_item, src.max_hit_points);
}

/// `OnAdded(parent)`.
fn on_added(w: &mut World, id: ItemId, parent: EntityId) {
    if let EntityId::Mobile(m) = parent {
        if core::aos(w) {
            super::skill_bonuses_add_to(w, id, m);
        }
        if let Some(ty) = w.mobile(m).map(|m| m.ty) {
            (ty.delta)(w, m, MobileDelta::ARMOR); // Tell them armor rating has changed
        }
    }
}

/// `OnRemoved(parent)`.
fn on_removed(w: &mut World, id: ItemId, parent: EntityId) {
    if let EntityId::Mobile(m) = parent {
        super::remove_stat_mods(w, id, m);
        if core::aos(w) {
            super::skill_bonuses_remove(w, id);
        }
        if let Some(ty) = w.mobile(m).map(|m| m.ty) {
            (ty.delta)(w, m, MobileDelta::ARMOR); // Tell them armor rating has changed
            (ty.check_stat_timers)(w, m);
        }
    }
    base::on_removed_base(w, id, parent);
}

/// `OnEquip(from)`.
fn on_equip(w: &mut World, id: ItemId, from: MobileId) -> bool {
    if let Some(ty) = w.mobile(from).map(|m| m.ty) {
        (ty.check_stat_timers)(w, from);
    }
    let (s, d, i) = (
        compute_stat_bonus(w, id, StatType::STR),
        compute_stat_bonus(w, id, StatType::DEX),
        compute_stat_bonus(w, id, StatType::INT),
    );
    super::add_stat_mods(w, id, from, s, d, i);
    base::on_equip_base(w, id, from)
}

/// `AllowSecureTrade(from, to, newOwner, accepted)`.
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

/// `CanEquip(from)`.
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
    let Some(ty) = w.item(id).map(|i| i.ty) else {
        return false;
    };
    if !(ty.check_race)(w, id, from, true) {
        return false;
    }
    let c = class(w, id);
    if super::wrong_gender(w, from, c.allow_male_wearer, c.allow_female_wearer) {
        return false;
    }

    let (str_bonus, str_req) = (
        compute_stat_bonus(w, id, StatType::STR),
        compute_stat_req(w, id, StatType::STR),
    );
    let (dex_bonus, dex_req) = (
        compute_stat_bonus(w, id, StatType::DEX),
        compute_stat_req(w, id, StatType::DEX),
    );
    let (int_bonus, int_req) = (
        compute_stat_bonus(w, id, StatType::INT),
        compute_stat_req(w, id, StatType::INT),
    );
    use muo_server::mobile::base as mb;
    let (dex, str, int) = (mb::dex(w, from), mb::str(w, from), mb::int(w, from));
    if dex < dex_req || dex + dex_bonus < 1 {
        deps::send_localized_message(w, from, 502077); // You do not have enough dexterity to equip this item.
        return false;
    }
    if str < str_req || str + str_bonus < 1 {
        deps::send_localized_message(w, from, 500213); // You are not strong enough to equip that.
        return false;
    }
    if int < int_req || int + int_bonus < 1 {
        deps::send_message(w, from, "You are not smart enough to equip that.");
        return false;
    }
    base::can_equip_base(w, id, from)
}

/// `CheckPropertyConflict(m)`.
fn check_property_conflict(w: &World, id: ItemId, m: MobileId) -> bool {
    base::check_property_conflict_base(w, id, m) || super::property_conflict_layer(w, id, m)
}

/// `AllowEquippedCast(from)`.
fn allow_equipped_cast(w: &World, id: ItemId, from: MobileId) -> bool {
    base::allow_equipped_cast_base(w, id, from)
        || super::attr(
            w,
            id,
            AttrKind::Attributes,
            AosAttribute::SpellChanneling as u32,
        ) != 0
}

/// The cliloc number of the ore, leather or scale type in `CraftResource`'s item name (0 for none).
pub(crate) fn ore_type_number(r: CraftResource) -> i32 {
    match r {
        CraftResource::DullCopper => 1053108,
        CraftResource::ShadowIron => 1053107,
        CraftResource::Copper => 1053106,
        CraftResource::Bronze => 1053105,
        CraftResource::Gold => 1053104,
        CraftResource::Agapite => 1053103,
        CraftResource::Verite => 1053102,
        CraftResource::Valorite => 1053101,
        CraftResource::SpinedLeather => 1061118,
        CraftResource::HornedLeather => 1061117,
        CraftResource::BarbedLeather => 1061116,
        CraftResource::RedScales => 1060814,
        CraftResource::YellowScales => 1060818,
        CraftResource::BlackScales => 1060820,
        CraftResource::GreenScales => 1060819,
        CraftResource::WhiteScales => 1060821,
        CraftResource::BlueScales => 1060815,
        _ => 0,
    }
}

/// `AddNameProperty(list)`.
fn add_name_property(w: &World, id: ItemId, list: &mut PropertyList) {
    let Some(d) = data(w, id) else { return };
    let Some(ty) = w.item(id).map(|i| i.ty) else {
        return;
    };
    let ore = ore_type_number(d.resource);
    let name = ops::name(w, id);
    let label = (ty.label_number)(w, id);
    let exceptional = d.quality == ArmorQuality::Exceptional;
    if ore != 0 {
        let quality_number = if exceptional { 1053100 } else { 1053099 };
        match name {
            Some(n) => list.add_number_fmt(quality_number, format_args!("#{ore}\t{n}")),
            None => list.add_number_fmt(quality_number, format_args!("#{ore}\t#{label}")),
        }
    } else if exceptional {
        match name {
            Some(n) => list.add_str(1050040, &n), // exceptional ~1_ITEMNAME~
            None => list.add_localized_number(1050040, label), // exceptional ~1_ITEMNAME~
        }
    } else {
        match name {
            None => list.add(label),
            Some(n) => list.add_text(&n),
        }
    }
}

/// `GetProperties(list)`.
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
    add_armor_attribute_properties(&d.armor_attributes, list, aos);
    add_attribute_properties(
        &d.attributes,
        list,
        aos,
        core::ml(w),
        0,
        0,
        get_luck_bonus(w, id),
    );
    (ty.add_resistance_properties)(w, id, list);

    let prop = durability_bonus(w, id);
    if prop > 0 {
        list.add_int(1060410, prop); // durability ~1_val~%
    }
    let lower = lower_stat_req(w, id);
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

fn durability_text(d: ArmorDurabilityLevel) -> Option<&'static str> {
    match d {
        ArmorDurabilityLevel::Durable => Some("durable"),
        ArmorDurabilityLevel::Substantial => Some("substantial"),
        ArmorDurabilityLevel::Massive => Some("massive"),
        ArmorDurabilityLevel::Fortified => Some("fortified"),
        ArmorDurabilityLevel::Indestructible => Some("indestructible"),
        ArmorDurabilityLevel::Regular => None,
    }
}

fn protection_text(p: ArmorProtectionLevel) -> Option<&'static str> {
    match p {
        ArmorProtectionLevel::Defense => Some("defense"),
        ArmorProtectionLevel::Guarding => Some("guarding"),
        ArmorProtectionLevel::Hardening => Some("hardening"),
        ArmorProtectionLevel::Fortification => Some("fortification"),
        ArmorProtectionLevel::Invulnerability => Some("invulnerability"),
        ArmorProtectionLevel::Regular => None,
    }
}

/// `OnSingleClick(from)`.
fn on_single_click(w: &mut World, id: ItemId, from: MobileId) {
    if !core_uotd(w) {
        on_single_click_pre_uotd(w, id, from);
        return;
    }
    let Some((d, ty)) = data(w, id).cloned().zip(w.item(id).map(|i| i.ty)) else {
        return;
    };
    let mut attrs = Vec::new();
    if (ty.display_loot_type)(w, id) {
        match w.item(id).map(|i| i.loot_type) {
            Some(muo_server::item::LootType::Blessed) => {
                attrs.push(EquipInfoAttribute::new(1038021, -1)) // blessed
            }
            Some(muo_server::item::LootType::Cursed) => {
                attrs.push(EquipInfoAttribute::new(1049643, -1)) // cursed
            }
            _ => {}
        }
    }
    if hooks(w)
        .and_then(|h| h.faction_item)
        .is_some_and(|f| f(w, id))
    {
        attrs.push(EquipInfoAttribute::new(1041350, -1)); // faction item
    }
    if d.quality == ArmorQuality::Exceptional {
        attrs.push(EquipInfoAttribute::new(1018305 - d.quality as i32, -1));
    }
    let gm = w
        .mobile(from)
        .is_some_and(|m| m.access_level >= AccessLevel::GameMaster);
    if d.identified || gm {
        if d.durability != ArmorDurabilityLevel::Regular {
            attrs.push(EquipInfoAttribute::new(1038000 + d.durability as i32, -1));
        }
        if d.protection_level != ArmorProtectionLevel::Regular {
            attrs.push(EquipInfoAttribute::new(
                1038005 + d.protection_level as i32,
                -1,
            ));
        }
    } else if d.durability != ArmorDurabilityLevel::Regular
        || d.protection_level != ArmorProtectionLevel::Regular
    {
        attrs.push(EquipInfoAttribute::new(1038000, -1)); // Unidentified
    }

    let name = ops::name(w, id);
    let number = match &name {
        None => (ty.label_number)(w, id),
        Some(n) => {
            ops::label_to(w, id, from, n);
            1041000
        }
    };
    if attrs.is_empty() && d.crafter.is_none() && name.is_some() {
        return;
    }
    let serial = w.item(id).map(|i| i.serial);
    if let (Some(ns), Some(serial)) = (deps::net_state_of(w, from), serial) {
        send_display_equipment_info(w, ns, serial, number, d.crafter.as_deref(), false, &attrs);
    }
}

/// `AppendSpaceWithArticle(text, articleAn)`.
pub(crate) fn append_space_with_article(b: &mut String, text: &str, article_an: bool) {
    if b.is_empty() {
        b.push_str(if article_an { "an " } else { "a " });
    } else {
        b.push(' ');
    }
    b.push_str(text);
}

/// `OnSingleClickPreUOTD(from)`.
pub fn on_single_click_pre_uotd(w: &mut World, id: ItemId, from: MobileId) {
    let Some((d, ty)) = data(w, id).cloned().zip(w.item(id).map(|i| i.ty)) else {
        return;
    };
    let magic = d.durability != ArmorDurabilityLevel::Regular
        || d.protection_level != ArmorProtectionLevel::Regular;
    let name = ops::name(w, id);
    let label = (ty.label_number)(w, id);
    if magic && !d.identified {
        let n = name.unwrap_or_else(|| lower_text(w, label));
        ops::label_to(w, id, from, &format!("an unidentified {n}"));
        return;
    }
    let article_an = ops::item_data(w, id)
        .flags
        .contains(muo_data::tile_data::TileFlag::ARTICLE_AN);
    if magic {
        let mut b = String::new();
        if let Some(t) = durability_text(d.durability) {
            append_space_with_article(
                &mut b,
                t,
                d.durability == ArmorDurabilityLevel::Indestructible,
            );
        }
        match &name {
            None => append_space_with_article(&mut b, &lower_text(w, label), article_an),
            Some(n) if !b.is_empty() => {
                b.push(' ');
                b.push_str(n);
            }
            Some(n) => b.push_str(n),
        }
        if let Some(t) = protection_text(d.protection_level) {
            b.push_str(" of ");
            b.push_str(t);
        }
        ops::label_to(w, id, from, &b);
        return;
    }
    let name = name.unwrap_or_else(|| {
        format!(
            "{} {}",
            if article_an { "an" } else { "a" },
            lower_text(w, label)
        )
    });
    let exceptional = d.quality == ArmorQuality::Exceptional;
    let text = match (&d.crafter, exceptional) {
        (None, true) => format!("{name} of exceptional quality"),
        (None, false) => name,
        (Some(c), true) => format!("{name} crafted with exceptional quality by {c}"),
        (Some(c), false) => format!("{name} crafted by {c}"),
    };
    ops::label_to(w, id, from, &text);
}

/// `ValidateMobile(m)`: sends what the wearer may no longer wear to their pack.
pub fn validate_mobile(w: &mut World, m: MobileId) {
    let Some(items) = w.mobile(m).map(|m| m.items.clone()) else {
        return;
    };
    for &item in items.iter().rev() {
        if !is_armor(w, item) {
            continue;
        }
        let Some(ty) = w.item(item).map(|i| i.ty) else {
            continue;
        };
        let c = class(w, item);
        let gm = w
            .mobile(m)
            .is_none_or(|m| m.access_level >= AccessLevel::GameMaster);
        if !(ty.check_race)(w, item, m, true)
            || (!gm && super::wrong_gender(w, m, c.allow_male_wearer, c.allow_female_wearer))
        {
            deps::add_to_backpack(w, m, item);
        }
    }
}

// ---- save layout (BaseArmor version 10) ----

const SERIALIZATION_VERSION: i32 = 10;

mod flag {
    pub const ATTRIBUTES: u32 = 0x1;
    pub const ARMOR_ATTRIBUTES: u32 = 0x2;
    pub const PHYSICAL_BONUS: u32 = 0x4;
    pub const FIRE_BONUS: u32 = 0x8;
    pub const COLD_BONUS: u32 = 0x10;
    pub const POISON_BONUS: u32 = 0x20;
    pub const ENERGY_BONUS: u32 = 0x40;
    pub const IDENTIFIED: u32 = 0x80;
    pub const MAX_HIT_POINTS: u32 = 0x100;
    pub const HIT_POINTS: u32 = 0x200;
    pub const CRAFTER: u32 = 0x400;
    pub const QUALITY: u32 = 0x800;
    pub const DURABILITY: u32 = 0x1000;
    pub const PROTECTION_LEVEL: u32 = 0x2000;
    pub const RESOURCE: u32 = 0x4000;
    pub const BASE_ARMOR_RATING: u32 = 0x8000;
    pub const STR_BONUS: u32 = 0x1_0000;
    pub const DEX_BONUS: u32 = 0x2_0000;
    pub const INT_BONUS: u32 = 0x4_0000;
    pub const STR_REQUIREMENT: u32 = 0x8_0000;
    pub const DEX_REQUIREMENT: u32 = 0x10_0000;
    pub const INT_REQUIREMENT: u32 = 0x20_0000;
    pub const MEDITATION_ALLOWANCE: u32 = 0x40_0000;
    pub const SKILL_BONUSES: u32 = 0x80_0000;
    /// Versions 8 and 9 and the pre-codegen layout also keep `PlayerConstructed` in the flags.
    pub const PLAYER_CONSTRUCTED: u32 = 0x100_0000;
}

fn serialize(w: &World, id: ItemId, writer: &mut dyn GenericWriter) {
    (ITEM_BASE.serialize)(w, id, writer);
    let Some(d) = data(w, id) else { return };
    writer.write_i32(SERIALIZATION_VERSION);

    let default_resource = class(w, id).default_resource;
    let mut f = 0_u32;
    let mut set = |on: bool, bit: u32| {
        if on {
            f |= bit;
        }
    };
    set(!d.attributes.is_empty(), flag::ATTRIBUTES);
    set(!d.armor_attributes.is_empty(), flag::ARMOR_ATTRIBUTES);
    set(d.physical_bonus != 0, flag::PHYSICAL_BONUS);
    set(d.fire_bonus != 0, flag::FIRE_BONUS);
    set(d.cold_bonus != 0, flag::COLD_BONUS);
    set(d.poison_bonus != 0, flag::POISON_BONUS);
    set(d.energy_bonus != 0, flag::ENERGY_BONUS);
    set(d.identified, flag::IDENTIFIED);
    set(d.max_hit_points != 0, flag::MAX_HIT_POINTS);
    set(d.hit_points != 0, flag::HIT_POINTS);
    set(
        d.crafter.as_deref().is_some_and(|s| !s.is_empty()),
        flag::CRAFTER,
    );
    set(d.quality != ArmorQuality::Regular, flag::QUALITY);
    set(
        d.durability != ArmorDurabilityLevel::Regular,
        flag::DURABILITY,
    );
    set(
        d.protection_level != ArmorProtectionLevel::Regular,
        flag::PROTECTION_LEVEL,
    );
    set(d.resource != default_resource, flag::RESOURCE);
    set(d.armor_base != -1, flag::BASE_ARMOR_RATING);
    set(d.str_bonus != -1, flag::STR_BONUS);
    set(d.dex_bonus != -1, flag::DEX_BONUS);
    set(d.int_bonus != -1, flag::INT_BONUS);
    set(d.str_req != -1, flag::STR_REQUIREMENT);
    set(d.dex_req != -1, flag::DEX_REQUIREMENT);
    set(d.int_req != -1, flag::INT_REQUIREMENT);
    set(
        d.meditate >= ArmorMeditationAllowance::All as i32,
        flag::MEDITATION_ALLOWANCE,
    );
    set(!d.skill_bonuses.is_empty(), flag::SKILL_BONUSES);
    writer.write_enum(4, i64::from(f));

    if f & flag::ATTRIBUTES != 0 {
        d.attributes.serialize(writer);
    }
    if f & flag::ARMOR_ATTRIBUTES != 0 {
        d.armor_attributes.serialize(writer);
    }
    if f & flag::PHYSICAL_BONUS != 0 {
        writer.write_encoded_int(d.physical_bonus);
    }
    if f & flag::FIRE_BONUS != 0 {
        writer.write_encoded_int(d.fire_bonus);
    }
    if f & flag::COLD_BONUS != 0 {
        writer.write_encoded_int(d.cold_bonus);
    }
    if f & flag::POISON_BONUS != 0 {
        writer.write_encoded_int(d.poison_bonus);
    }
    if f & flag::ENERGY_BONUS != 0 {
        writer.write_encoded_int(d.energy_bonus);
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
    if f & flag::DURABILITY != 0 {
        writer.write_enum(4, d.durability as i64);
    }
    if f & flag::PROTECTION_LEVEL != 0 {
        writer.write_enum(4, d.protection_level as i64);
    }
    if f & flag::RESOURCE != 0 {
        writer.write_enum(4, d.resource as i64);
    }
    if f & flag::BASE_ARMOR_RATING != 0 {
        writer.write_encoded_int(d.armor_base);
    }
    if f & flag::STR_BONUS != 0 {
        writer.write_encoded_int(d.str_bonus);
    }
    if f & flag::DEX_BONUS != 0 {
        writer.write_encoded_int(d.dex_bonus);
    }
    if f & flag::INT_BONUS != 0 {
        writer.write_encoded_int(d.int_bonus);
    }
    if f & flag::STR_REQUIREMENT != 0 {
        writer.write_encoded_int(d.str_req);
    }
    if f & flag::DEX_REQUIREMENT != 0 {
        writer.write_encoded_int(d.dex_req);
    }
    if f & flag::INT_REQUIREMENT != 0 {
        writer.write_encoded_int(d.int_req);
    }
    if f & flag::MEDITATION_ALLOWANCE != 0 {
        writer.write_enum(4, i64::from(d.meditate));
    }
    if f & flag::SKILL_BONUSES != 0 {
        d.skill_bonuses.serialize(writer);
    }
}

/// How an older layout differs from version 10 (all of them read the same flag bits in the same order).
struct Layout {
    /// Versions 8 and below store the crafter as a mobile, whose name is taken on the next tick.
    crafter_is_mobile: bool,
    /// Versions 8, 9 and below keep `PlayerConstructed` in the flags.
    player_constructed_flag: bool,
    /// `MigrateFrom(V8Content)` forgets to copy `ProtectionLevel`: the bytes are read, the value is dropped.
    drop_protection_level: bool,
    /// What a record without the quality flag means: Regular, except for the pre-codegen reader, which never sets
    /// `_quality` there and so leaves the enum's zero, `Low`.
    quality_without_flag: ArmorQuality,
}

const V10: Layout = Layout {
    crafter_is_mobile: false,
    player_constructed_flag: false,
    drop_protection_level: false,
    quality_without_flag: ArmorQuality::Regular,
};
const V9: Layout = Layout {
    crafter_is_mobile: false,
    player_constructed_flag: true,
    drop_protection_level: false,
    quality_without_flag: ArmorQuality::Regular,
};
const V8: Layout = Layout {
    crafter_is_mobile: true,
    player_constructed_flag: true,
    drop_protection_level: true,
    quality_without_flag: ArmorQuality::Regular,
};
const PRE_CODEGEN: Layout = Layout {
    crafter_is_mobile: true,
    player_constructed_flag: true,
    drop_protection_level: false,
    quality_without_flag: ArmorQuality::Low,
};

fn deserialize(w: &mut World, id: ItemId, reader: &mut dyn GenericReader) -> Result<()> {
    templates::table(w); // as in `new_armor`
    (ITEM_BASE.deserialize)(w, id, reader)?;
    let version = reader.read_i32()?;
    let layout = match version {
        8 => &V8,
        9 => &V9,
        v if v < SERIALIZATION_VERSION => &PRE_CODEGEN,
        _ => &V10,
    };
    let f = reader.read_enum(4)? as u32;
    let default_resource = class(w, id).default_resource;
    let mut d = ArmorData::for_load(default_resource);
    d.quality = layout.quality_without_flag;
    let mut crafter_mobile = None;

    if f & flag::ATTRIBUTES != 0 {
        d.attributes = Attrs::deserialize(reader)?;
    }
    if f & flag::ARMOR_ATTRIBUTES != 0 {
        d.armor_attributes = Attrs::deserialize(reader)?;
    }
    if f & flag::PHYSICAL_BONUS != 0 {
        d.physical_bonus = reader.read_encoded_int()?;
    }
    if f & flag::FIRE_BONUS != 0 {
        d.fire_bonus = reader.read_encoded_int()?;
    }
    if f & flag::COLD_BONUS != 0 {
        d.cold_bonus = reader.read_encoded_int()?;
    }
    if f & flag::POISON_BONUS != 0 {
        d.poison_bonus = reader.read_encoded_int()?;
    }
    if f & flag::ENERGY_BONUS != 0 {
        d.energy_bonus = reader.read_encoded_int()?;
    }
    d.identified = f & flag::IDENTIFIED != 0;
    if f & flag::MAX_HIT_POINTS != 0 {
        d.max_hit_points = reader.read_encoded_int()?;
    }
    if f & flag::HIT_POINTS != 0 {
        d.hit_points = reader.read_encoded_int()?;
    }
    if f & flag::CRAFTER != 0 {
        if layout.crafter_is_mobile {
            crafter_mobile = reader.read_mobile(w)?;
        } else {
            d.crafter = reader.read_string()?;
        }
    }
    if f & flag::QUALITY != 0 {
        d.quality = read_enum(reader, ArmorQuality::from_i32, "ArmorQuality")?;
    }
    if f & flag::DURABILITY != 0 {
        d.durability = read_enum(
            reader,
            ArmorDurabilityLevel::from_i32,
            "ArmorDurabilityLevel",
        )?;
    }
    if f & flag::PROTECTION_LEVEL != 0 {
        let p = read_enum(
            reader,
            ArmorProtectionLevel::from_i32,
            "ArmorProtectionLevel",
        )?;
        if !layout.drop_protection_level {
            d.protection_level = p;
        }
    }
    if f & flag::RESOURCE != 0 {
        d.resource = read_enum(reader, CraftResource::from_i32, "CraftResource")?;
    }
    if f & flag::BASE_ARMOR_RATING != 0 {
        d.armor_base = reader.read_encoded_int()?;
    }
    if f & flag::STR_BONUS != 0 {
        d.str_bonus = reader.read_encoded_int()?;
    }
    if f & flag::DEX_BONUS != 0 {
        d.dex_bonus = reader.read_encoded_int()?;
    }
    if f & flag::INT_BONUS != 0 {
        d.int_bonus = reader.read_encoded_int()?;
    }
    if f & flag::STR_REQUIREMENT != 0 {
        d.str_req = reader.read_encoded_int()?;
    }
    if f & flag::DEX_REQUIREMENT != 0 {
        d.dex_req = reader.read_encoded_int()?;
    }
    if f & flag::INT_REQUIREMENT != 0 {
        d.int_req = reader.read_encoded_int()?;
    }
    if f & flag::MEDITATION_ALLOWANCE != 0 {
        d.meditate = reader.read_enum(4)? as i32;
    }
    if f & flag::SKILL_BONUSES != 0 {
        d.skill_bonuses = Attrs::deserialize(reader)?;
    }
    if layout.player_constructed_flag
        && let Some(i) = w.item_mut(id)
    {
        i.player_constructed = f & flag::PLAYER_CONSTRUCTED != 0;
    }

    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(d);
    }
    if layout.crafter_is_mobile {
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

/// `[AfterDeserialization]`: a worn armor puts its skill bonuses and stat bonuses on its wearer.
fn after_deserialization(w: &mut World, id: ItemId) {
    let m = parent_mobile(w, id);
    if core::aos(w)
        && let Some(m) = m
    {
        super::skill_bonuses_add_to(w, id, m);
    }
    let default_resource = class(w, id).default_resource;
    if let Some(d) = data_mut(w, id)
        && d.resource == CraftResource::None
    {
        d.resource = default_resource;
    }
    let (s, d, i) = (
        compute_stat_bonus(w, id, StatType::STR),
        compute_stat_bonus(w, id, StatType::DEX),
        compute_stat_bonus(w, id, StatType::INT),
    );
    if let Some(m) = m {
        if s != 0 || d != 0 || i != 0 {
            super::add_stat_mods(w, id, m, s, d, i);
        }
        if let Some(ty) = w.mobile(m).map(|m| m.ty) {
            (ty.check_stat_timers)(w, m);
        }
    }
}

// ---- class chain ----

/// `Server.Items.BaseArmor`.
pub static BASE_ARMOR: ItemType = ItemType {
    full_name: "Server.Items.BaseArmor",
    parent: Some(&ITEM_BASE),
    construct: None,
    family: Some(&BASE_ARMOR_CLASS),
    default_weight: |w, id| {
        class(w, id)
            .default_weight
            .unwrap_or_else(|| base::default_weight_base(w, id))
    },
    label_number: |w, id| match class_label_number(w, id) {
        0 => base::label_number_base(w, id),
        n => n,
    },
    required_races: |w, id| {
        class(w, id)
            .required_races
            .unwrap_or_else(|| base::required_races_base(w, id))
    },
    set_hue,
    physical_resistance: |w, id| {
        resistance(
            w,
            id,
            "basePhysicalResistance",
            |c| c.base_physical_resistance,
            |a| a.armor_physical_resist,
            |d| d.physical_bonus,
        )
    },
    fire_resistance: |w, id| {
        resistance(
            w,
            id,
            "baseFireResistance",
            |c| c.base_fire_resistance,
            |a| a.armor_fire_resist,
            |d| d.fire_bonus,
        )
    },
    cold_resistance: |w, id| {
        resistance(
            w,
            id,
            "baseColdResistance",
            |c| c.base_cold_resistance,
            |a| a.armor_cold_resist,
            |d| d.cold_bonus,
        )
    },
    poison_resistance: |w, id| {
        resistance(
            w,
            id,
            "basePoisonResistance",
            |c| c.base_poison_resistance,
            |a| a.armor_poison_resist,
            |d| d.poison_bonus,
        )
    },
    energy_resistance: |w, id| {
        resistance(
            w,
            id,
            "baseEnergyResistance",
            |c| c.base_energy_resistance,
            |a| a.armor_energy_resist,
            |d| d.energy_bonus,
        )
    },
    on_after_duped,
    on_added,
    on_removed,
    on_equip,
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

// ---- BaseShield ----

/// `BaseShield`: nothing but `MaterialType = Plate` (the default here) in its constants, and two virtuals.
pub static BASE_SHIELD_CLASS: ArmorClass = ArmorClass {
    material_type: ArmorMaterialType::Plate,
    armor_rating: shield_armor_rating,
    on_hit: shield_on_hit,
    ..BASE_ARMOR_CLASS
};

/// `BaseShield.ArmorRating`: worn, the wearer's Parry skill scales it.
fn shield_armor_rating(w: &World, id: ItemId) -> f64 {
    let ar = base_armor_rating_virtual(w, id);
    match parent_mobile(w, id) {
        Some(m) => muo_server::skills::value_ref(w, m, SkillName::Parry) * ar / 200.0 + 1.0,
        None => ar,
    }
}

/// `BaseShield.OnHit`.
fn shield_on_hit(w: &mut World, id: ItemId, weapon_id: ItemId, damage: i32) -> i32 {
    if core::aos(w) {
        if self_repair(w, id) > rnd::below(w, 10) {
            let hp = data(w, id).map_or(0, |d| d.hit_points);
            set_hit_points(w, id, hp + 2);
        } else {
            let half = armor_rating(w, id) / 2.0;
            let absorbed = ((half + half * rnd::double(w)) as i32).max(2);
            let wear = if weapon::weapon_type(w, weapon_id) == WeaponType::Bashing {
                absorbed / 2
            } else {
                rnd::below(w, 2)
            };
            WEAR.apply(w, id, wear);
        }
        return 0;
    }

    let Some(owner) = parent_mobile(w, id) else {
        return damage;
    };
    let ar = armor_rating(w, id);
    let parry = muo_server::skills::value_ref(w, owner, SkillName::Parry);
    let chance = ((parry - ar * 2.0) / 100.0).max(0.01);

    let mut damage = damage;
    if muo_server::mobile::part2::stats::check_skill_chance(w, owner, SkillName::Parry, chance) {
        let taken = if weapon::skill(w, weapon_id) == SkillName::Archery {
            ar as i32
        } else {
            (ar / 2.0) as i32
        };
        damage -= damage.min(taken);
        muo_server::mobile::part2::effects::fixed_effect(w, owner, 0x37B9, 10, 16);
        if rnd::below(w, 100) < 25 {
            // 25% chance to lower durability
            let wear = rnd::below(w, 2);
            WEAR.apply(w, id, wear);
        }
    }
    damage
}

/// `Server.Items.BaseShield`.
pub static BASE_SHIELD: ItemType = ItemType {
    full_name: "Server.Items.BaseShield",
    parent: Some(&BASE_ARMOR),
    family: Some(&BASE_SHIELD_CLASS),
    serialize: |w, id, out| {
        (BASE_ARMOR.serialize)(w, id, out);
        out.write_i32(0);
    },
    deserialize: |w, id, r| {
        (BASE_ARMOR.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        Ok(())
    },
    ..BASE_ARMOR
};
