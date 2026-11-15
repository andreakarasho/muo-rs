//! `BaseWeapon` (current save layout), `BaseMeleeWeapon` and the abstract classes below it (BaseSword, BaseAxe,
//! BaseBashing, BaseKnife, BasePoleArm, BaseSpear, BaseStaff, BaseRanged, BaseThrown), plus the weapon enums.
//!
//! State is [`WeaponData`] in `Item::ext` (the fields of every class in the chain). What a leaf's `override`s
//! returned is [`WeaponClass`], one `static` per class.
//!
//! The behavior is in sibling modules: `weapon_combat` (OnSwing, CheckHit, OnHit/OnMiss, damage and delay formulas,
//! equip, mods), `weapon_text` (properties, name, single click), `weapon_bases` (the abstract classes' code and
//! OnCraft), `weapon_ability` (WeaponAbility and HitLower), `weapon_migrate` (saves below version 11), `slayer`
//! (SlayerGroup). Systems outside the weapons are hooks in `weapon_hooks`.
//!
//! The `virtual` members a class overrides are fn pointers of [`WeaponClass`] (`on_hit`, `on_swing`, ..): a leaf sets
//! its own and calls the `weapon_combat::*_base` function for `base.X(..)`.

use muo_io::{GenericReader, GenericWriter, Result};
use muo_primitives::TimeSpan;
use muo_server::World;
use muo_server::ext::Ext;
use muo_server::ids::{ItemId, MobileId};
use muo_server::item::base;
use muo_server::item::deps;
use muo_server::item::ops;
use muo_server::item::{ITEM_BASE, ItemType};
use muo_server::layer::Layer;
use muo_server::mods::ModId;
use muo_server::poison::PoisonId;
use muo_server::serialization_ext::{ReadEntityExt, WriteEntityExt};
use muo_server::skills::SkillName;

use super::equip_hooks::CraftArgs;
use super::equip_props::{self as ep, Names};
use super::weapon_bases as wb;
use super::weapon_combat as wc;
use super::weapon_hooks as hk;
use super::weapon_text as wt;
use super::{MlInt, random_min_max, read_enum, same};
use crate::aos::{AosWeaponAttribute, AttrKind, Attrs};
use crate::core;
use crate::craft::{self, CraftResource};
use crate::templates::{self, item_props as ip};

int_enum! {
    /// `WeaponQuality`.
    pub enum WeaponQuality { Low = 0, Regular = 1, Exceptional = 2 }
}

int_enum! {
    /// `WeaponType`.
    pub enum WeaponType {
        Axe = 0, Slashing = 1, Staff = 2, Bashing = 3, Piercing = 4, Polearm = 5, Ranged = 6, Fists = 7,
    }
}

int_enum! {
    /// `WeaponDamageLevel`.
    pub enum WeaponDamageLevel { Regular = 0, Ruin = 1, Might = 2, Force = 3, Power = 4, Vanq = 5 }
}

int_enum! {
    /// `WeaponAccuracyLevel`.
    pub enum WeaponAccuracyLevel {
        Regular = 0, Accurate = 1, Surpassingly = 2, Eminently = 3, Exceedingly = 4, Supremely = 5,
    }
}

int_enum! {
    /// `WeaponDurabilityLevel`.
    pub enum WeaponDurabilityLevel {
        Regular = 0, Durable = 1, Substantial = 2, Massive = 3, Fortified = 4, Indestructible = 5,
    }
}

int_enum! {
    /// `WeaponAnimation`.
    pub enum WeaponAnimation {
        Slash1H = 9, Pierce1H = 10, Bash1H = 11, Bash2H = 12, Slash2H = 13, Pierce2H = 14, ShootBow = 18,
        ShootXBow = 19, Wrestle = 31, Throwing = 32,
    }
}

/// A `WeaponAbility`, by its index in `WeaponAbility.Abilities` (0 is the null slot). The behavior of each is in
/// `weapon_ability`.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
#[repr(u8)]
pub enum Ability {
    ArmorIgnore = 1,
    BleedAttack,
    ConcussionBlow,
    CrushingBlow,
    Disarm,
    Dismount,
    DoubleStrike,
    InfectiousStrike,
    MortalStrike,
    MovingShot,
    ParalyzingBlow,
    ShadowStrike,
    WhirlwindAttack,
    RidingSwipe,
    FrenziedWhirlwind,
    Block,
    DefenseMastery,
    NerveStrike,
    TalonStrike,
    Feint,
    DualWield,
    DoubleShot,
    ArmorPierce,
    Bladeweave,
    ForceArrow,
    LightningArrow,
    PsychicAttack,
    SerpentArrow,
    ForceOfNature,
    InfusedThrow,
    MysticArc,
}

impl Ability {
    pub const ALL: [Ability; 31] = {
        use Ability::*;
        [
            ArmorIgnore,
            BleedAttack,
            ConcussionBlow,
            CrushingBlow,
            Disarm,
            Dismount,
            DoubleStrike,
            InfectiousStrike,
            MortalStrike,
            MovingShot,
            ParalyzingBlow,
            ShadowStrike,
            WhirlwindAttack,
            RidingSwipe,
            FrenziedWhirlwind,
            Block,
            DefenseMastery,
            NerveStrike,
            TalonStrike,
            Feint,
            DualWield,
            DoubleShot,
            ArmorPierce,
            Bladeweave,
            ForceArrow,
            LightningArrow,
            PsychicAttack,
            SerpentArrow,
            ForceOfNature,
            InfusedThrow,
            MysticArc,
        ]
    };
}

pub static ABILITIES: Names<Ability> = Names::new(|| Ability::ALL.to_vec());
pub static TYPES: Names<WeaponType> = Names::new(|| ep::all_of(16, WeaponType::from_i32));
pub static ANIMATIONS: Names<WeaponAnimation> =
    Names::new(|| ep::all_of(64, WeaponAnimation::from_i32));

/// The `override`s of a weapon class that are constants: one `static` per class, a subclass is
/// `WeaponClass { .. , ..PARENT_CLASS }`. The stat fields (requirements, damage, speed, hits, sounds, range) are
/// fallbacks: a class's template `props` (`Data/Templates/Items`, the field in camelCase) override them.
#[derive(Copy, Clone, Debug)]
pub struct WeaponClass {
    /// `DefaultWeight` when the class overrides it; else the tiledata weight.
    pub default_weight: Option<f64>,
    /// `LabelNumber` when overridden (0 keeps the tiledata name); the `labelNumber` prop overrides it.
    pub label_number: i32,
    pub primary_ability: Option<Ability>,
    pub secondary_ability: Option<Ability>,
    /// Read through [`artifact_rarity`] (the `artifactRarity` prop overrides it).
    pub artifact_rarity: i32,
    pub def_max_range: i32,
    pub def_hit_sound: i32,
    pub def_miss_sound: i32,
    pub def_skill: SkillName,
    pub def_type: WeaponType,
    pub def_animation: WeaponAnimation,
    /// `AccuracySkill`.
    pub accuracy_skill: SkillName,
    pub aos_str_req: i32,
    pub aos_dex_req: i32,
    pub aos_int_req: i32,
    pub aos_min_damage: MlInt,
    pub aos_max_damage: MlInt,
    pub aos_speed: i32,
    pub ml_speed: f32,
    pub old_str_req: i32,
    pub old_dex_req: i32,
    pub old_int_req: i32,
    pub old_min_damage: i32,
    pub old_max_damage: i32,
    pub old_speed: i32,
    pub init_min_hits: i32,
    pub init_max_hits: i32,
    /// `CanFortify`.
    pub can_fortify: bool,
    /// BaseAxe's `UnscaleDurability`/`ScaleDurability` also scale `UsesRemaining` by `GetUsesScalar()`.
    pub scales_uses: bool,
    /// BaseThrown's `MinThrowRange` (0 for a class that is not a thrown weapon).
    pub min_throw_range: i32,
    /// BaseRanged's `EffectID` (the moving-effect graphic of a shot).
    pub effect_id: i32,
    /// BaseRanged's `AmmoType`, by full type name (`Ammo` makes one with `make_item`).
    pub ammo_type: Option<&'static str>,
    /// `VirtualDamageBonus`.
    pub virtual_damage_bonus: i32,
    /// BaseAxe/BasePoleArm's `HarvestSystem`, by name (`Lumberjacking`, `Mining`; `None` is no harvest system).
    pub harvest_system: Option<&'static str>,
    // The virtuals of BaseWeapon a class may override, called by `weapon_combat` as virtual dispatch. A leaf sets
    // the field to its own function and calls the `*_base` function for `base.X(..)`.
    /// `OnSwing(attacker, defender, damageBonus)`.
    pub on_swing: fn(&mut World, ItemId, MobileId, MobileId, f64) -> TimeSpan,
    /// `CheckHit(attacker, defender)`.
    pub check_hit: fn(&mut World, ItemId, MobileId, MobileId) -> bool,
    /// `OnHit(attacker, defender, damageBonus)`.
    pub on_hit: fn(&mut World, ItemId, MobileId, MobileId, f64),
    /// `OnMiss(attacker, defender)`.
    pub on_miss: fn(&mut World, ItemId, MobileId, MobileId),
    /// `OnBeforeSwing(attacker, defender)`.
    pub on_before_swing: fn(&mut World, ItemId, MobileId, MobileId),
    /// `GetDelay(m)`.
    pub get_delay: fn(&mut World, ItemId, MobileId) -> TimeSpan,
    /// `AbsorbDamage(attacker, defender, damage)`.
    pub absorb_damage: fn(&mut World, ItemId, MobileId, MobileId, i32) -> i32,
    /// `GetDamageTypes(wielder, ..)`; `None` is no wielder (property list, hue).
    pub get_damage_types: fn(&World, ItemId, Option<MobileId>) -> wc::DamageTypes,
    /// `GetBaseDamage(attacker)`.
    pub get_base_damage: fn(&mut World, ItemId, MobileId) -> f64,
    /// `GetDefendSkillValue(attacker, defender)`.
    pub get_defend_skill_value: fn(&mut World, ItemId, MobileId, MobileId) -> f64,
    /// `ModifyHitChance(attacker, defender, chance)`.
    pub modify_hit_chance: fn(&mut World, ItemId, MobileId, MobileId, f64) -> f64,
    /// `ModifyDamage(attacker, defender, damage)`.
    pub modify_damage: fn(&mut World, ItemId, MobileId, MobileId, i32) -> i32,
    /// `OnCraft(..)` with the crafting arguments resolved.
    pub on_craft: fn(&mut World, ItemId, &CraftArgs) -> i32,
    /// BaseRanged's `OnFired(attacker, defender)`.
    pub on_fired: fn(&mut World, ItemId, MobileId, MobileId) -> bool,
}

/// `BaseWeapon`'s defaults (the values its `virtual` properties return).
pub static BASE_WEAPON_CLASS: WeaponClass = WeaponClass {
    default_weight: None,
    label_number: 0,
    primary_ability: None,
    secondary_ability: None,
    artifact_rarity: 0,
    def_max_range: 1,
    def_hit_sound: 0,
    def_miss_sound: 0,
    def_skill: SkillName::Swords,
    def_type: WeaponType::Slashing,
    def_animation: WeaponAnimation::Slash1H,
    accuracy_skill: SkillName::Tactics,
    aos_str_req: 0,
    aos_dex_req: 0,
    aos_int_req: 0,
    aos_min_damage: same(0),
    aos_max_damage: same(0),
    aos_speed: 0,
    ml_speed: 0.0,
    old_str_req: 0,
    old_dex_req: 0,
    old_int_req: 0,
    old_min_damage: 0,
    old_max_damage: 0,
    old_speed: 0,
    init_min_hits: 0,
    init_max_hits: 0,
    can_fortify: true,
    scales_uses: false,
    min_throw_range: 0,
    effect_id: 0,
    ammo_type: None,
    virtual_damage_bonus: 0,
    harvest_system: None,
    on_swing: wc::swing_base,
    check_hit: wc::check_hit_base,
    on_hit: wc::on_hit_base,
    on_miss: wc::on_miss_base,
    on_before_swing: wc::on_before_swing_base,
    get_delay: wc::get_delay_base,
    absorb_damage: wc::absorb_damage_base,
    get_damage_types: wc::get_damage_types_base,
    get_base_damage: wc::get_base_damage_base,
    get_defend_skill_value: wc::get_defend_skill_value_base,
    modify_hit_chance: wc::modify_hit_chance_base,
    modify_damage: wc::modify_damage_base,
    on_craft: wb::on_craft_base,
    on_fired: |_, _, _, _| true,
};

/// Everything the weapon classes add to `Item`; `Ext::new(WeaponData)` on every weapon.
#[derive(Clone, Debug)]
pub struct WeaponData {
    pub damage_level: WeaponDamageLevel,
    pub accuracy_level: WeaponAccuracyLevel,
    pub durability_level: WeaponDurabilityLevel,
    pub quality: WeaponQuality,
    pub hit_points: i32,
    pub max_hit_points: i32,
    /// `SlayerName`, by value (0 is None).
    pub slayer: i32,
    pub slayer2: i32,
    pub poison: Option<PoisonId>,
    pub poison_charges: i32,
    pub crafter: Option<String>,
    pub identified: bool,
    /// -1 means "use the class's `Aos`/`Old` value".
    pub str_requirement: i32,
    pub dex_requirement: i32,
    pub int_requirement: i32,
    pub min_damage: i32,
    pub max_damage: i32,
    pub hit_sound: i32,
    pub miss_sound: i32,
    /// -1 means "use the class's value".
    pub speed: f32,
    pub max_range: i32,
    /// -1 means the class's `DefSkill`.
    pub skill: i32,
    pub weapon_type: i32,
    pub animation: i32,
    pub resource: CraftResource,
    pub attributes: Attrs,
    pub weapon_attributes: Attrs,
    pub skill_bonuses: Attrs,
    pub element_damages: Attrs,
    pub engraved_text: Option<String>,
    pub cursed: bool,
    pub consecrated: bool,
    // BaseAxe / BasePoleArm
    pub show_uses_remaining: bool,
    pub uses_remaining: i32,
    // BaseRanged
    pub balanced: bool,
    pub velocity: i32,
    // Not serialized: the skill mods this weapon put on its wearer (`m_SkillMod`, `m_MageMod`) and its owner.
    pub skill_mod: Option<(ModId, MobileId)>,
    pub mage_mod: Option<(ModId, MobileId)>,
    /// `FireworksWand._charges` (a `[SerializableField]` of that leaf, saved by its own serializer).
    pub charges: i32,
    /// `BaseWand._wandEffect` (a `WandEffect`, by value).
    pub wand_effect: i32,
}

impl WeaponData {
    /// What deserialization starts from: each field as the generated `Deserialize` leaves it when its flag is
    /// absent.
    pub(super) fn for_load() -> WeaponData {
        WeaponData {
            damage_level: WeaponDamageLevel::Regular,
            accuracy_level: WeaponAccuracyLevel::Regular,
            durability_level: WeaponDurabilityLevel::Regular,
            quality: WeaponQuality::Regular,
            hit_points: 0,
            max_hit_points: 0,
            slayer: 0,
            slayer2: 0,
            poison: None,
            poison_charges: 0,
            crafter: None,
            identified: false,
            str_requirement: -1,
            dex_requirement: -1,
            int_requirement: -1,
            min_damage: -1,
            max_damage: -1,
            hit_sound: -1,
            miss_sound: -1,
            speed: -1.0,
            max_range: -1,
            skill: -1,
            weapon_type: -1,
            animation: -1,
            resource: CraftResource::Iron,
            attributes: Attrs::default(),
            weapon_attributes: Attrs::default(),
            skill_bonuses: Attrs::default(),
            element_damages: Attrs::default(),
            engraved_text: None,
            cursed: false,
            consecrated: false,
            show_uses_remaining: false,
            uses_remaining: 0,
            balanced: false,
            velocity: 0,
            skill_mod: None,
            mage_mod: None,
            charges: 0,
            wand_effect: 0,
        }
    }
}

/// The weapon state of an item.
pub fn data(w: &World, id: ItemId) -> Option<&WeaponData> {
    w.item(id)?.ext.get()
}

/// The weapon state of an item, mutable.
pub fn data_mut(w: &mut World, id: ItemId) -> Option<&mut WeaponData> {
    w.item_mut(id)?.ext.get_mut()
}

/// The constants of the item's class.
pub fn class(w: &World, id: ItemId) -> &'static WeaponClass {
    w.item(id)
        .and_then(|i| i.ty.family::<WeaponClass>())
        .unwrap_or(&BASE_WEAPON_CLASS)
}

/// `item is BaseWeapon`.
pub fn is_weapon(w: &World, id: ItemId) -> bool {
    w.item(id).is_some_and(|i| i.ty.is(&BASE_WEAPON))
}

// ---- constructor ----

/// `new BaseWeapon(itemID)` for class `ty`; every leaf constructor starts here. The returned item is in the
/// world on `Map.Internal` like any new item.
pub fn new_weapon(w: &mut World, ty: &'static ItemType, item_id: i32) -> ItemId {
    let id = ops::new_item_with_id(w, ty, item_id);
    // Layer = (Layer)ItemData.Quality
    let quality = ops::item_data(w, id).quality;
    (ty.set_layer)(w, id, Layer::from_raw(quality));
    let c = ty.family::<WeaponClass>().unwrap_or(&BASE_WEAPON_CLASS);
    templates::table(w); // the stat props below and in combat read the loaded table
    let (min, max) = (
        ip::int(w, id, "initMinHits", c.init_min_hits),
        ip::int(w, id, "initMaxHits", c.init_max_hits),
    );
    let hits = random_min_max(w, min, max);
    let mut d = WeaponData::for_load();
    d.quality = WeaponQuality::Regular;
    d.hit_points = hits;
    d.max_hit_points = hits;
    d.resource = CraftResource::Iron;
    // BaseAxe / BasePoleArm constructors
    if ty.is(&BASE_AXE) || ty.is(&BASE_POLE_ARM) {
        d.uses_remaining = 150;
    }
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(d);
    }
    id
}

// ---- stat getters (the virtual Aos/Old properties and their overrides by an instance value) ----

/// The instance value, else the class's `Aos*` prop/field (`Core.ML` picks an `MlInt`'s side) or its `Old*` one.
fn aos_or_old(
    w: &World,
    id: ItemId,
    set: i32,
    (aos_key, aos): (&str, fn(&WeaponClass) -> MlInt),
    (old_key, old): (&str, fn(&WeaponClass) -> i32),
) -> i32 {
    if set != -1 {
        return set;
    }
    let c = class(w, id);
    if core::aos(w) {
        ip::ml_int(w, id, aos_key, aos(c)).get(core::ml(w))
    } else {
        ip::int(w, id, old_key, old(c))
    }
}

/// `StrRequirement`.
pub fn str_requirement(w: &World, id: ItemId) -> i32 {
    let set = data(w, id).map_or(-1, |d| d.str_requirement);
    aos_or_old(
        w,
        id,
        set,
        ("aosStrReq", |c| same(c.aos_str_req)),
        ("oldStrReq", |c| c.old_str_req),
    )
}

/// `DexRequirement`.
pub fn dex_requirement(w: &World, id: ItemId) -> i32 {
    let set = data(w, id).map_or(-1, |d| d.dex_requirement);
    aos_or_old(
        w,
        id,
        set,
        ("aosDexReq", |c| same(c.aos_dex_req)),
        ("oldDexReq", |c| c.old_dex_req),
    )
}

/// `IntRequirement`.
pub fn int_requirement(w: &World, id: ItemId) -> i32 {
    let set = data(w, id).map_or(-1, |d| d.int_requirement);
    aos_or_old(
        w,
        id,
        set,
        ("aosIntReq", |c| same(c.aos_int_req)),
        ("oldIntReq", |c| c.old_int_req),
    )
}

/// `MinDamage`.
pub fn min_damage(w: &World, id: ItemId) -> i32 {
    let set = data(w, id).map_or(-1, |d| d.min_damage);
    aos_or_old(
        w,
        id,
        set,
        ("aosMinDamage", |c| c.aos_min_damage),
        ("oldMinDamage", |c| c.old_min_damage),
    )
}

/// `MaxDamage`.
pub fn max_damage(w: &World, id: ItemId) -> i32 {
    let set = data(w, id).map_or(-1, |d| d.max_damage);
    aos_or_old(
        w,
        id,
        set,
        ("aosMaxDamage", |c| c.aos_max_damage),
        ("oldMaxDamage", |c| c.old_max_damage),
    )
}

/// `HitSound` (`AosHitSound`/`OldHitSound` are `DefHitSound` in every class).
pub fn hit_sound(w: &World, id: ItemId) -> i32 {
    let set = data(w, id).map_or(-1, |d| d.hit_sound);
    if set == -1 {
        ip::int(w, id, "defHitSound", class(w, id).def_hit_sound)
    } else {
        set
    }
}

/// `MissSound`.
pub fn miss_sound(w: &World, id: ItemId) -> i32 {
    let set = data(w, id).map_or(-1, |d| d.miss_sound);
    if set == -1 {
        ip::int(w, id, "defMissSound", class(w, id).def_miss_sound)
    } else {
        set
    }
}

/// `Speed`: the instance value, else `MlSpeed` (ML), `AosSpeed` (AOS) or `OldSpeed`.
pub fn speed(w: &World, id: ItemId) -> f32 {
    let set = data(w, id).map_or(-1.0, |d| d.speed);
    if set != -1.0 {
        return set;
    }
    let c = class(w, id);
    if core::ml(w) {
        ip::float(w, id, "mlSpeed", f64::from(c.ml_speed)) as f32
    } else if core::aos(w) {
        ip::int(w, id, "aosSpeed", c.aos_speed) as f32
    } else {
        ip::int(w, id, "oldSpeed", c.old_speed) as f32
    }
}

/// `MaxRange` (`DefMaxRange`; BaseThrown scales it with the wielder's Str).
pub fn max_range(w: &World, id: ItemId) -> i32 {
    let set = data(w, id).map_or(-1, |d| d.max_range);
    if set != -1 {
        return set;
    }
    let c = class(w, id);
    let mtr = min_throw_range(w, id);
    if mtr > 0 {
        let (min, max) = (mtr, mtr + 3);
        let Some(attacker) = super::parent_mobile(w, id) else {
            return max;
        };
        let str_req = ip::int(w, id, "aosStrReq", c.aos_str_req);
        let divisor = (140 - str_req) / 3;
        if divisor <= 0 {
            return max;
        }
        let scaled = min + (muo_server::mobile::base::str(w, attacker) - str_req) / divisor;
        return scaled.clamp(min, max);
    }
    ip::int(w, id, "defMaxRange", c.def_max_range)
}

/// BaseRanged `EffectID` (BaseThrown: the item's own graphic).
pub fn effect_id(w: &World, id: ItemId) -> i32 {
    if min_throw_range(w, id) > 0 {
        w.item(id).map_or(0, |i| i.item_id)
    } else {
        ip::int(w, id, "effectId", class(w, id).effect_id)
    }
}

/// `PoisonCharges` setter.
pub fn set_poison_charges(w: &mut World, id: ItemId, value: i32) {
    if let Some(d) = data_mut(w, id) {
        if d.poison_charges == value {
            return;
        }
        d.poison_charges = value;
    }
    ops::invalidate_properties(w, id);
}

/// `Skill`.
pub fn skill(w: &World, id: ItemId) -> SkillName {
    let set = data(w, id).map_or(-1, |d| d.skill);
    SkillName::from_i32(set)
        .unwrap_or_else(|| ep::named(w, id, "defSkill", &ep::SKILLS, class(w, id).def_skill))
}

/// `Type`.
pub fn weapon_type(w: &World, id: ItemId) -> WeaponType {
    let set = data(w, id).map_or(-1, |d| d.weapon_type);
    WeaponType::from_i32(set)
        .unwrap_or_else(|| ep::named(w, id, "defType", &TYPES, class(w, id).def_type))
}

/// `Animation`.
pub fn animation(w: &World, id: ItemId) -> WeaponAnimation {
    let set = data(w, id).map_or(-1, |d| d.animation);
    WeaponAnimation::from_i32(set).unwrap_or_else(|| {
        ep::named(
            w,
            id,
            "defAnimation",
            &ANIMATIONS,
            class(w, id).def_animation,
        )
    })
}

/// `PrimaryAbility` and `SecondaryAbility`.
pub fn abilities(w: &World, id: ItemId) -> (Option<Ability>, Option<Ability>) {
    let c = class(w, id);
    (
        ep::named_opt(w, id, "primaryAbility", &ABILITIES, c.primary_ability),
        ep::named_opt(w, id, "secondaryAbility", &ABILITIES, c.secondary_ability),
    )
}

/// BaseRanged's `AmmoType`, by full type name.
pub fn ammo_type(w: &World, id: ItemId) -> Option<&'static str> {
    ep::str_opt(w, id, "ammoType", class(w, id).ammo_type)
}

/// BaseThrown's `MinThrowRange` (0 for a weapon that is not thrown).
pub fn min_throw_range(w: &World, id: ItemId) -> i32 {
    ip::int(w, id, "minThrowRange", class(w, id).min_throw_range)
}

/// BaseAxe/BasePoleArm's `HarvestSystem` by name.
pub fn harvest_system(w: &World, id: ItemId) -> Option<&'static str> {
    ep::str_opt(w, id, "harvestSystem", class(w, id).harvest_system)
}

/// `VirtualDamageBonus`.
pub fn virtual_damage_bonus(w: &World, id: ItemId) -> i32 {
    ip::int(
        w,
        id,
        "virtualDamageBonus",
        class(w, id).virtual_damage_bonus,
    )
}

/// `CanFortify`.
pub fn can_fortify(w: &World, id: ItemId) -> bool {
    ep::boolean(w, id, "canFortify", class(w, id).can_fortify)
}

// ---- properties with side effects ----

/// `Quality` setter.
pub fn set_quality(w: &mut World, id: ItemId, value: WeaponQuality) {
    unscale_durability(w, id);
    if let Some(d) = data_mut(w, id) {
        d.quality = value;
    }
    scale_durability(w, id);
    ops::invalidate_properties(w, id);
}

/// `DurabilityLevel` setter.
pub fn set_durability_level(w: &mut World, id: ItemId, value: WeaponDurabilityLevel) {
    unscale_durability(w, id);
    if let Some(d) = data_mut(w, id) {
        d.durability_level = value;
    }
    ops::invalidate_properties(w, id);
    scale_durability(w, id);
}

/// `DamageLevel` setter.
pub fn set_damage_level(w: &mut World, id: ItemId, value: WeaponDamageLevel) {
    if let Some(d) = data_mut(w, id)
        && d.damage_level != value
    {
        d.damage_level = value;
        ops::invalidate_properties(w, id);
    }
}

/// `AccuracyLevel` setter: before AOS (`UseSkillMod`) it adds or updates the wielder's "WeaponAccuracy" skill mod.
pub fn set_accuracy_level(w: &mut World, id: ItemId, value: WeaponAccuracyLevel) {
    let changed = data_mut(w, id).is_some_and(|d| {
        let c = d.accuracy_level != value;
        if c {
            d.accuracy_level = value;
        }
        c
    });
    if !changed {
        return;
    }
    if wc::use_skill_mod(w) {
        if value == WeaponAccuracyLevel::Regular {
            if let Some((m, owner)) = data_mut(w, id).and_then(|d| d.skill_mod.take())
                && let Some(ty) = w.mobile(owner).map(|m| m.ty)
            {
                (ty.remove_skill_mod)(w, owner, m);
            }
        } else if let Some((m, owner)) = data(w, id).and_then(|d| d.skill_mod) {
            muo_server::mods::set_skill_mod_value(w, owner, m, f64::from(value as i32 * 5));
        } else if let Some(mobile) = super::parent_mobile(w, id) {
            wc::add_accuracy_mod(w, id, mobile);
        }
    }
    ops::invalidate_properties(w, id);
}

/// `Resource` setter: rescales durability around the change and takes the resource's hue.
pub fn set_resource(w: &mut World, id: ItemId, value: CraftResource) {
    unscale_durability(w, id);
    if let Some(d) = data_mut(w, id) {
        d.resource = value;
    }
    let ty = w.item(id).map(|i| i.ty);
    if let Some(ty) = ty {
        let hue = craft::hue(w, value);
        (ty.set_hue)(w, id, hue);
    }
    ops::invalidate_properties(w, id);
    scale_durability(w, id);
}

/// `HitPoints` setter.
pub fn set_hit_points(w: &mut World, id: ItemId, value: i32) {
    if let Some(d) = data_mut(w, id) {
        if d.hit_points == value {
            return;
        }
        d.hit_points = value.min(d.max_hit_points);
    }
    ops::invalidate_properties(w, id);
}

/// `MaxHitPoints` setter.
pub fn set_max_hit_points(w: &mut World, id: ItemId, value: i32) {
    if let Some(d) = data_mut(w, id) {
        if d.max_hit_points == value {
            return;
        }
        d.max_hit_points = value;
    }
    ops::invalidate_properties(w, id);
}

// ---- durability ----

/// `GetDurabilityBonus()`.
pub fn durability_bonus(w: &World, id: ItemId) -> i32 {
    let Some(d) = data(w, id) else { return 0 };
    if !super::core_uor(w) {
        return d.durability_level as i32 * 5 + (d.quality as i32 - 1) * 10;
    }
    let mut bonus = match d.durability_level {
        WeaponDurabilityLevel::Durable => 20,
        WeaponDurabilityLevel::Substantial => 50,
        WeaponDurabilityLevel::Massive => 70,
        WeaponDurabilityLevel::Fortified => 100,
        WeaponDurabilityLevel::Indestructible => 120,
        WeaponDurabilityLevel::Regular => 0,
    };
    if core::aos(w) {
        bonus += d
            .weapon_attributes
            .get(true, AosWeaponAttribute::DurabilityBonus as u32)
            + craft::attribute_info(d.resource).map_or(0, |a| a.weapon_durability);
    }
    if d.quality == WeaponQuality::Exceptional {
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
        .weapon_attributes
        .get(true, AosWeaponAttribute::LowerStatReq as u32);
    if let Some(a) = craft::attribute_info(d.resource) {
        v += a.weapon_lower_requirements;
    }
    v.min(100)
}

/// `ScalesUses` (BaseAxe's `UsesRemaining` follows durability): the `scalesUses` prop, else the class table.
pub fn scales_uses(w: &World, id: ItemId) -> bool {
    ep::boolean(w, id, "scalesUses", class(w, id).scales_uses)
}

/// `AccuracySkill`: the `accuracySkill` prop, else the class table.
pub fn accuracy_skill(w: &World, id: ItemId) -> SkillName {
    ep::named(
        w,
        id,
        "accuracySkill",
        &ep::SKILLS,
        class(w, id).accuracy_skill,
    )
}

/// `GetUsesScalar()` of BaseAxe.
fn uses_scalar(d: &WeaponData) -> i32 {
    if d.quality == WeaponQuality::Exceptional {
        200
    } else {
        100
    }
}

/// `UnscaleDurability()` (BaseAxe's override also unscales `UsesRemaining`).
pub fn unscale_durability(w: &mut World, id: ItemId) {
    let scale = 100 + durability_bonus(w, id);
    let scales_uses = scales_uses(w, id);
    if let Some(d) = data_mut(w, id) {
        d.hit_points = (d.hit_points * 100 + (scale - 1)) / scale;
        d.max_hit_points = (d.max_hit_points * 100 + (scale - 1)) / scale;
        if scales_uses {
            let s = uses_scalar(d);
            d.uses_remaining = (d.uses_remaining * 100 + (s - 1)) / s;
        }
    }
    ops::invalidate_properties(w, id);
}

/// `ScaleDurability()` (BaseAxe's override also scales `UsesRemaining`).
pub fn scale_durability(w: &mut World, id: ItemId) {
    let scale = 100 + durability_bonus(w, id);
    let scales_uses = scales_uses(w, id);
    if let Some(d) = data_mut(w, id) {
        d.hit_points = (d.hit_points * scale + 99) / 100;
        d.max_hit_points = (d.max_hit_points * scale + 99) / 100;
        if scales_uses {
            let s = uses_scalar(d);
            d.uses_remaining = (d.uses_remaining * s + 99) / 100;
        }
    }
    ops::invalidate_properties(w, id);
}

// ---- vtable entries ----

fn weapon_attr(w: &World, id: ItemId, a: AosWeaponAttribute) -> i32 {
    super::attr(w, id, AttrKind::Weapon, a as u32)
}

fn default_weight(w: &World, id: ItemId) -> f64 {
    ep::float_opt(w, id, "defaultWeight", class(w, id).default_weight)
        .unwrap_or_else(|| base::default_weight_base(w, id))
}

fn label_number(w: &World, id: ItemId) -> i32 {
    match ip::int(w, id, "labelNumber", class(w, id).label_number) {
        0 => base::label_number_base(w, id),
        n => n,
    }
}

/// `ArtifactRarity`: the class's `artifactRarity` prop, else the class table's.
pub fn artifact_rarity(w: &World, id: ItemId) -> i32 {
    ip::int(w, id, "artifactRarity", class(w, id).artifact_rarity)
}

fn set_hue(w: &mut World, id: ItemId, value: i32) {
    base::set_hue_base(w, id, value);
    ops::invalidate_properties(w, id);
}

/// `CheckConflictingLayer`.
fn check_conflicting_layer(
    w: &mut World,
    id: ItemId,
    m: MobileId,
    item: ItemId,
    layer: Layer,
) -> bool {
    if base::check_conflicting_layer_base(w, id, m, item, layer) {
        return true;
    }
    let own = w.item(id).map_or(Layer::Invalid, |i| i.layer);
    let shield_or_light = w.item(item).is_some_and(|i| {
        i.ty.is(&super::armor::BASE_SHIELD)
            || super::is_named(i.ty, "Server.Items.BaseEquipableLight")
    });
    if own == Layer::TwoHanded && layer == Layer::OneHanded {
        deps::send_localized_message(w, m, 500214); // You already have something in both hands.
        return true;
    }
    if own == Layer::OneHanded && layer == Layer::TwoHanded && !shield_or_light {
        deps::send_localized_message(w, m, 500215); // You can only wield one weapon at a time.
        return true;
    }
    false
}

/// `OnAfterDuped`: the copy gets its own attribute bags, the hue (the resource set it) and the hit points
/// (durability scaled them).
fn on_after_duped(w: &mut World, id: ItemId, new_item: ItemId) {
    let Some(src) = data(w, id).cloned() else {
        return;
    };
    let hue = w.item(id).map_or(0, |i| i.hue);
    let Some(d) = data_mut(w, new_item) else {
        return;
    };
    d.attributes = src.attributes.clone();
    d.element_damages = src.element_damages.clone();
    d.skill_bonuses = src.skill_bonuses.clone();
    d.weapon_attributes = src.weapon_attributes.clone();
    d.hit_points = src.hit_points;
    d.max_hit_points = src.max_hit_points;
    if let Some(ty) = w.item(new_item).map(|i| i.ty) {
        (ty.set_hue)(w, new_item, hue);
    }
}

// ---- save layout (BaseWeapon version 11) ----

const SERIALIZATION_VERSION: i32 = 11;

mod flag {
    pub const DAMAGE_LEVEL: u32 = 0x1;
    pub const ACCURACY_LEVEL: u32 = 0x2;
    pub const DURABILITY_LEVEL: u32 = 0x4;
    pub const QUALITY: u32 = 0x8;
    pub const HIT_POINTS: u32 = 0x10;
    pub const MAX_HIT_POINTS: u32 = 0x20;
    pub const SLAYER: u32 = 0x40;
    pub const POISON: u32 = 0x80;
    pub const POISON_CHARGES: u32 = 0x100;
    pub const CRAFTER: u32 = 0x200;
    pub const IDENTIFIED: u32 = 0x400;
    pub const STR_REQUIREMENT: u32 = 0x800;
    pub const DEX_REQUIREMENT: u32 = 0x1000;
    pub const INT_REQUIREMENT: u32 = 0x2000;
    pub const MIN_DAMAGE: u32 = 0x4000;
    pub const MAX_DAMAGE: u32 = 0x8000;
    pub const HIT_SOUND: u32 = 0x1_0000;
    pub const MISS_SOUND: u32 = 0x2_0000;
    pub const SPEED: u32 = 0x4_0000;
    pub const MAX_RANGE: u32 = 0x8_0000;
    pub const SKILL: u32 = 0x10_0000;
    pub const TYPE: u32 = 0x20_0000;
    pub const ANIMATION: u32 = 0x40_0000;
    pub const RESOURCE: u32 = 0x80_0000;
    pub const ATTRIBUTES: u32 = 0x100_0000;
    pub const WEAPON_ATTRIBUTES: u32 = 0x200_0000;
    pub const SKILL_BONUSES: u32 = 0x400_0000;
    pub const SLAYER2: u32 = 0x800_0000;
    pub const ELEMENT_DAMAGES: u32 = 0x1000_0000;
    pub const ENGRAVED_TEXT: u32 = 0x2000_0000;
}

fn write_enum(writer: &mut dyn GenericWriter, v: i32) {
    writer.write_enum(4, i64::from(v));
}

fn serialize(w: &World, id: ItemId, writer: &mut dyn GenericWriter) {
    (ITEM_BASE.serialize)(w, id, writer);
    let Some(d) = data(w, id) else { return };
    writer.write_i32(SERIALIZATION_VERSION);

    // The ShouldSerialize* tests, in field order of the generated SaveFlag.
    let mut f = 0_u32;
    let mut set = |on: bool, bit: u32| {
        if on {
            f |= bit;
        }
    };
    set(
        d.damage_level != WeaponDamageLevel::Regular,
        flag::DAMAGE_LEVEL,
    );
    set(
        d.accuracy_level != WeaponAccuracyLevel::Regular,
        flag::ACCURACY_LEVEL,
    );
    set(
        d.durability_level != WeaponDurabilityLevel::Regular,
        flag::DURABILITY_LEVEL,
    );
    set(d.quality != WeaponQuality::Regular, flag::QUALITY);
    set(d.hit_points > 0, flag::HIT_POINTS);
    set(d.max_hit_points != 0, flag::MAX_HIT_POINTS);
    set(d.slayer != 0, flag::SLAYER);
    set(d.poison.is_some(), flag::POISON);
    set(d.poison_charges > 0, flag::POISON_CHARGES);
    set(
        d.crafter.as_deref().is_some_and(|s| !s.is_empty()),
        flag::CRAFTER,
    );
    set(d.identified, flag::IDENTIFIED);
    set(d.str_requirement != -1, flag::STR_REQUIREMENT);
    set(d.dex_requirement != -1, flag::DEX_REQUIREMENT);
    set(d.int_requirement != -1, flag::INT_REQUIREMENT);
    set(d.min_damage != -1, flag::MIN_DAMAGE);
    set(d.max_damage != -1, flag::MAX_DAMAGE);
    set(d.hit_sound != -1, flag::HIT_SOUND);
    set(d.miss_sound != -1, flag::MISS_SOUND);
    set(d.speed != -1.0, flag::SPEED);
    set(d.max_range != -1, flag::MAX_RANGE);
    set(d.skill != -1, flag::SKILL);
    set(d.weapon_type != -1, flag::TYPE);
    set(d.animation != -1, flag::ANIMATION);
    set(d.resource != CraftResource::Iron, flag::RESOURCE);
    set(!d.attributes.is_empty(), flag::ATTRIBUTES);
    set(!d.weapon_attributes.is_empty(), flag::WEAPON_ATTRIBUTES);
    set(!d.skill_bonuses.is_empty(), flag::SKILL_BONUSES);
    set(d.slayer2 != 0, flag::SLAYER2);
    set(!d.element_damages.is_empty(), flag::ELEMENT_DAMAGES);
    set(
        d.engraved_text.as_deref().is_some_and(|s| !s.is_empty()),
        flag::ENGRAVED_TEXT,
    );
    write_enum(writer, f as i32);

    if f & flag::DAMAGE_LEVEL != 0 {
        write_enum(writer, d.damage_level as i32);
    }
    if f & flag::ACCURACY_LEVEL != 0 {
        write_enum(writer, d.accuracy_level as i32);
    }
    if f & flag::DURABILITY_LEVEL != 0 {
        write_enum(writer, d.durability_level as i32);
    }
    if f & flag::QUALITY != 0 {
        write_enum(writer, d.quality as i32);
    }
    if f & flag::HIT_POINTS != 0 {
        writer.write_i32(d.hit_points);
    }
    if f & flag::MAX_HIT_POINTS != 0 {
        writer.write_i32(d.max_hit_points);
    }
    if f & flag::SLAYER != 0 {
        write_enum(writer, d.slayer);
    }
    if f & flag::POISON != 0 {
        writer.write_poison(d.poison);
    }
    if f & flag::POISON_CHARGES != 0 {
        writer.write_i32(d.poison_charges);
    }
    if f & flag::CRAFTER != 0 {
        writer.write_opt_string(d.crafter.as_deref());
    }
    if f & flag::STR_REQUIREMENT != 0 {
        writer.write_i32(d.str_requirement);
    }
    if f & flag::DEX_REQUIREMENT != 0 {
        writer.write_i32(d.dex_requirement);
    }
    if f & flag::INT_REQUIREMENT != 0 {
        writer.write_i32(d.int_requirement);
    }
    if f & flag::MIN_DAMAGE != 0 {
        writer.write_i32(d.min_damage);
    }
    if f & flag::MAX_DAMAGE != 0 {
        writer.write_i32(d.max_damage);
    }
    if f & flag::HIT_SOUND != 0 {
        writer.write_i32(d.hit_sound);
    }
    if f & flag::MISS_SOUND != 0 {
        writer.write_i32(d.miss_sound);
    }
    if f & flag::SPEED != 0 {
        writer.write_f32(d.speed);
    }
    if f & flag::MAX_RANGE != 0 {
        writer.write_i32(d.max_range);
    }
    if f & flag::SKILL != 0 {
        write_enum(writer, d.skill);
    }
    if f & flag::TYPE != 0 {
        write_enum(writer, d.weapon_type);
    }
    if f & flag::ANIMATION != 0 {
        write_enum(writer, d.animation);
    }
    if f & flag::RESOURCE != 0 {
        write_enum(writer, d.resource as i32);
    }
    if f & flag::ATTRIBUTES != 0 {
        d.attributes.serialize(writer);
    }
    if f & flag::WEAPON_ATTRIBUTES != 0 {
        d.weapon_attributes.serialize(writer);
    }
    if f & flag::SKILL_BONUSES != 0 {
        d.skill_bonuses.serialize(writer);
    }
    if f & flag::SLAYER2 != 0 {
        write_enum(writer, d.slayer2);
    }
    if f & flag::ELEMENT_DAMAGES != 0 {
        d.element_damages.serialize(writer);
    }
    if f & flag::ENGRAVED_TEXT != 0 {
        writer.write_opt_string(d.engraved_text.as_deref());
    }
}

fn deserialize(w: &mut World, id: ItemId, reader: &mut dyn GenericReader) -> Result<()> {
    templates::table(w); // as in `new_weapon`
    (ITEM_BASE.deserialize)(w, id, reader)?;
    let version = reader.read_i32()?;
    if version == 10 {
        let m = super::weapon_migrate::read_v10(w, reader)?;
        super::weapon_migrate::apply(w, id, m);
        after_deserialization(w, id);
        return Ok(());
    }
    if version < SERIALIZATION_VERSION {
        let m = super::weapon_migrate::read_legacy(w, reader, version)?;
        super::weapon_migrate::apply(w, id, m);
        after_deserialization(w, id);
        return Ok(());
    }
    let f = reader.read_enum(4)? as u32;
    let mut d = WeaponData::for_load();

    if f & flag::DAMAGE_LEVEL != 0 {
        d.damage_level = read_enum(reader, WeaponDamageLevel::from_i32, "WeaponDamageLevel")?;
    }
    if f & flag::ACCURACY_LEVEL != 0 {
        d.accuracy_level = read_enum(reader, WeaponAccuracyLevel::from_i32, "WeaponAccuracyLevel")?;
    }
    if f & flag::DURABILITY_LEVEL != 0 {
        d.durability_level = read_enum(
            reader,
            WeaponDurabilityLevel::from_i32,
            "WeaponDurabilityLevel",
        )?;
    }
    if f & flag::QUALITY != 0 {
        d.quality = read_enum(reader, WeaponQuality::from_i32, "WeaponQuality")?;
    }
    if f & flag::HIT_POINTS != 0 {
        d.hit_points = reader.read_i32()?;
    }
    if f & flag::MAX_HIT_POINTS != 0 {
        d.max_hit_points = reader.read_i32()?;
    }
    if f & flag::SLAYER != 0 {
        d.slayer = reader.read_enum(4)? as i32;
    }
    if f & flag::POISON != 0 {
        d.poison = reader.read_poison()?;
    }
    if f & flag::POISON_CHARGES != 0 {
        d.poison_charges = reader.read_i32()?;
    }
    if f & flag::CRAFTER != 0 {
        d.crafter = reader.read_string()?;
    }
    d.identified = f & flag::IDENTIFIED != 0;
    if f & flag::STR_REQUIREMENT != 0 {
        d.str_requirement = reader.read_i32()?;
    }
    if f & flag::DEX_REQUIREMENT != 0 {
        d.dex_requirement = reader.read_i32()?;
    }
    if f & flag::INT_REQUIREMENT != 0 {
        d.int_requirement = reader.read_i32()?;
    }
    if f & flag::MIN_DAMAGE != 0 {
        d.min_damage = reader.read_i32()?;
    }
    if f & flag::MAX_DAMAGE != 0 {
        d.max_damage = reader.read_i32()?;
    }
    if f & flag::HIT_SOUND != 0 {
        d.hit_sound = reader.read_i32()?;
    }
    if f & flag::MISS_SOUND != 0 {
        d.miss_sound = reader.read_i32()?;
    }
    if f & flag::SPEED != 0 {
        d.speed = reader.read_f32()?;
    }
    if f & flag::MAX_RANGE != 0 {
        d.max_range = reader.read_i32()?;
    }
    if f & flag::SKILL != 0 {
        d.skill = reader.read_enum(4)? as i32;
    }
    if f & flag::TYPE != 0 {
        d.weapon_type = reader.read_enum(4)? as i32;
    }
    if f & flag::ANIMATION != 0 {
        d.animation = reader.read_enum(4)? as i32;
    }
    if f & flag::RESOURCE != 0 {
        d.resource = read_enum(reader, CraftResource::from_i32, "CraftResource")?;
    }
    if f & flag::ATTRIBUTES != 0 {
        d.attributes = Attrs::deserialize(reader)?;
    }
    if f & flag::WEAPON_ATTRIBUTES != 0 {
        d.weapon_attributes = Attrs::deserialize(reader)?;
    }
    if f & flag::SKILL_BONUSES != 0 {
        d.skill_bonuses = Attrs::deserialize(reader)?;
    }
    if f & flag::SLAYER2 != 0 {
        d.slayer2 = reader.read_enum(4)? as i32;
    }
    if f & flag::ELEMENT_DAMAGES != 0 {
        d.element_damages = Attrs::deserialize(reader)?;
    }
    if f & flag::ENGRAVED_TEXT != 0 {
        d.engraved_text = reader.read_string()?;
    }

    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(d);
    }
    after_deserialization(w, id);
    Ok(())
}

/// `[AfterDeserialization] AfterDeserialization()`: the mods of a worn weapon, and hit points for a record without.
fn after_deserialization(w: &mut World, id: ItemId) {
    let parent = super::parent_mobile(w, id);
    let (level, mage) = (
        data(w, id).map_or(WeaponAccuracyLevel::Regular, |d| d.accuracy_level),
        super::attr(
            w,
            id,
            AttrKind::Weapon,
            AosWeaponAttribute::MageWeapon as u32,
        ),
    );

    if wc::use_skill_mod(w)
        && level != WeaponAccuracyLevel::Regular
        && let Some(p) = parent
    {
        wc::add_accuracy_mod(w, id, p);
    }

    if core::aos(w)
        && mage != 0
        && mage != 30
        && let Some(p) = parent
    {
        wc::add_mage_mod(w, id, p);
    }

    if core::aos(w)
        && let Some(p) = parent
    {
        super::skill_bonuses_add_to(w, id, p);
    }

    let (s, d, i) = (
        super::attr(
            w,
            id,
            AttrKind::Attributes,
            crate::aos::AosAttribute::BonusStr as u32,
        ),
        super::attr(
            w,
            id,
            AttrKind::Attributes,
            crate::aos::AosAttribute::BonusDex as u32,
        ),
        super::attr(
            w,
            id,
            AttrKind::Attributes,
            crate::aos::AosAttribute::BonusInt as u32,
        ),
    );
    if let Some(p) = parent
        && (s != 0 || d != 0 || i != 0)
    {
        super::add_stat_mods(w, id, p, s, d, i);
    }

    if let Some(p) = parent
        && let Some(ty) = w.mobile(p).map(|m| m.ty)
    {
        (ty.check_stat_timers)(w, p);
    }

    let (hp, max) = data(w, id).map_or((1, 1), |d| (d.hit_points, d.max_hit_points));
    if hp <= 0 && max <= 0 {
        let c = class(w, id);
        let (min, max) = (
            ip::int(w, id, "initMinHits", c.init_min_hits),
            ip::int(w, id, "initMaxHits", c.init_max_hits),
        );
        let hits = random_min_max(w, min, max);
        if let Some(d) = data_mut(w, id) {
            d.hit_points = hits;
            d.max_hit_points = hits;
        }
    }
}

// ---- class chain ----

/// `Server.Items.BaseWeapon`.
pub static BASE_WEAPON: ItemType = ItemType {
    full_name: "Server.Items.BaseWeapon",
    parent: Some(&ITEM_BASE),
    construct: None,
    family: Some(&BASE_WEAPON_CLASS),
    default_weight,
    label_number,
    required_races: |w, id| {
        ep::required_races(w, id, None).unwrap_or_else(|| base::required_races_base(w, id))
    },
    set_hue,
    physical_resistance: |w, id| weapon_attr(w, id, AosWeaponAttribute::ResistPhysicalBonus),
    fire_resistance: |w, id| weapon_attr(w, id, AosWeaponAttribute::ResistFireBonus),
    cold_resistance: |w, id| weapon_attr(w, id, AosWeaponAttribute::ResistColdBonus),
    poison_resistance: |w, id| weapon_attr(w, id, AosWeaponAttribute::ResistPoisonBonus),
    energy_resistance: |w, id| weapon_attr(w, id, AosWeaponAttribute::ResistEnergyBonus),
    check_conflicting_layer,
    on_after_duped,
    can_equip: wc::can_equip,
    on_equip: wc::on_equip,
    on_added: wc::on_added,
    on_removed: wc::on_removed,
    allow_secure_trade: wc::allow_secure_trade,
    allow_equipped_cast: wc::allow_equipped_cast,
    add_name_property: wt::add_name_property,
    get_properties: wt::get_properties,
    on_single_click: wt::on_single_click,
    serialize,
    deserialize,
    ..ITEM_BASE
};

/// `BaseMeleeWeapon`: `AbsorbDamage` also lets Attune Weapon absorb and (pre-AOS) Reactive Armor reflect.
pub static BASE_MELEE_WEAPON_CLASS: WeaponClass = WeaponClass {
    absorb_damage: wb::melee_absorb_damage,
    ..BASE_WEAPON_CLASS
};

/// `Server.Items.BaseMeleeWeapon`: no state and no generated serializer, so no version of its own (its
/// Serialize/Deserialize only call the base).
pub static BASE_MELEE_WEAPON: ItemType = ItemType {
    full_name: "Server.Items.BaseMeleeWeapon",
    parent: Some(&BASE_WEAPON),
    family: Some(&BASE_MELEE_WEAPON_CLASS),
    ..BASE_WEAPON
};

macro_rules! plain_class {
    ($(#[$m:meta])* $name:ident, $full:literal, $parent:ident, $class:ident $(, $f:ident : $v:expr)* $(,)?) => {
        $(#[$m])*
        pub static $name: ItemType = ItemType {
            full_name: $full,
            parent: Some(&$parent),
            family: Some(&$class),
            serialize: |w, id, out| {
                ($parent.serialize)(w, id, out);
                out.write_i32(0);
            },
            deserialize: |w, id, r| {
                ($parent.deserialize)(w, id, r)?;
                let _version = r.read_i32()?;
                Ok(())
            },
            $($f: $v,)*
            ..$parent
        };
    };
}

/// `BaseSword`.
pub static BASE_SWORD_CLASS: WeaponClass = WeaponClass {
    on_hit: wb::blade_on_hit,
    def_skill: SkillName::Swords,
    def_type: WeaponType::Slashing,
    def_animation: WeaponAnimation::Slash1H,
    ..BASE_MELEE_WEAPON_CLASS
};
plain_class!(
    /// `Server.Items.BaseSword`.
    BASE_SWORD, "Server.Items.BaseSword", BASE_MELEE_WEAPON, BASE_SWORD_CLASS,
    on_double_click: wb::blade_on_double_click
);

/// `BaseAxe` (`UsesRemaining` is 150 for a new one, scaled with durability).
pub static BASE_AXE_CLASS: WeaponClass = WeaponClass {
    on_hit: wb::axe_on_hit,
    def_hit_sound: 0x232,
    def_miss_sound: 0x23A,
    def_skill: SkillName::Swords,
    def_type: WeaponType::Axe,
    def_animation: WeaponAnimation::Slash2H,
    scales_uses: true,
    harvest_system: Some("Lumberjacking"),
    ..BASE_MELEE_WEAPON_CLASS
};

/// `Server.Items.BaseAxe`: `ShowUsesRemaining` and `UsesRemaining` after the version.
pub static BASE_AXE: ItemType = ItemType {
    full_name: "Server.Items.BaseAxe",
    parent: Some(&BASE_MELEE_WEAPON),
    family: Some(&BASE_AXE_CLASS),
    serialize: serialize_uses_remaining,
    deserialize: deserialize_uses_remaining,
    on_double_click: wb::axe_on_double_click,
    get_context_menu_entries: harvest_menu_entries,
    ..BASE_MELEE_WEAPON
};

/// `GetContextMenuEntries`: the base entries, then the harvest tool's.
fn harvest_menu_entries(
    w: &mut World,
    id: ItemId,
    from: MobileId,
    list: &mut Vec<muo_server::context_menu_entry::ContextMenuEntry>,
) {
    base::get_context_menu_entries_base(w, id, from, list);
    hk::harvest_context_entries(w, from, id, list);
}

fn serialize_uses_remaining(w: &World, id: ItemId, out: &mut dyn GenericWriter) {
    (BASE_MELEE_WEAPON.serialize)(w, id, out);
    out.write_i32(0);
    let d = data(w, id);
    out.write_bool(d.is_some_and(|d| d.show_uses_remaining));
    out.write_i32(d.map_or(0, |d| d.uses_remaining));
}

fn deserialize_uses_remaining(w: &mut World, id: ItemId, r: &mut dyn GenericReader) -> Result<()> {
    (BASE_MELEE_WEAPON.deserialize)(w, id, r)?;
    let _version = r.read_i32()?;
    let (show, uses) = (r.read_bool()?, r.read_i32()?);
    if let Some(d) = data_mut(w, id) {
        d.show_uses_remaining = show;
        d.uses_remaining = uses;
    }
    Ok(())
}

/// `BaseBashing`.
pub static BASE_BASHING_CLASS: WeaponClass = WeaponClass {
    on_hit: wb::bashing_on_hit,
    get_base_damage: wb::bashing_base_damage,
    def_hit_sound: 0x233,
    def_miss_sound: 0x239,
    def_skill: SkillName::Macing,
    def_type: WeaponType::Bashing,
    def_animation: WeaponAnimation::Bash1H,
    ..BASE_MELEE_WEAPON_CLASS
};
plain_class!(
    /// `Server.Items.BaseBashing`.
    BASE_BASHING, "Server.Items.BaseBashing", BASE_MELEE_WEAPON, BASE_BASHING_CLASS
);

/// `BaseKnife`.
pub static BASE_KNIFE_CLASS: WeaponClass = WeaponClass {
    on_hit: wb::blade_on_hit,
    def_hit_sound: 0x23B,
    def_miss_sound: 0x238,
    def_skill: SkillName::Swords,
    def_type: WeaponType::Slashing,
    def_animation: WeaponAnimation::Slash1H,
    ..BASE_MELEE_WEAPON_CLASS
};
plain_class!(
    /// `Server.Items.BaseKnife`.
    BASE_KNIFE, "Server.Items.BaseKnife", BASE_MELEE_WEAPON, BASE_KNIFE_CLASS,
    on_double_click: wb::blade_on_double_click
);

/// `BasePoleArm`.
pub static BASE_POLE_ARM_CLASS: WeaponClass = WeaponClass {
    on_hit: wb::pole_arm_on_hit,
    def_hit_sound: 0x237,
    def_miss_sound: 0x238,
    def_skill: SkillName::Swords,
    def_type: WeaponType::Polearm,
    def_animation: WeaponAnimation::Slash2H,
    harvest_system: Some("Lumberjacking"),
    ..BASE_MELEE_WEAPON_CLASS
};

/// `Server.Items.BasePoleArm`: same two fields as BaseAxe, but no durability scaling of the uses.
pub static BASE_POLE_ARM: ItemType = ItemType {
    full_name: "Server.Items.BasePoleArm",
    parent: Some(&BASE_MELEE_WEAPON),
    family: Some(&BASE_POLE_ARM_CLASS),
    serialize: serialize_uses_remaining,
    deserialize: deserialize_uses_remaining,
    on_double_click: wb::pole_arm_on_double_click,
    get_context_menu_entries: harvest_menu_entries,
    ..BASE_MELEE_WEAPON
};

/// `BaseSpear`.
pub static BASE_SPEAR_CLASS: WeaponClass = WeaponClass {
    on_hit: wb::spear_on_hit,
    def_hit_sound: 0x23C,
    def_miss_sound: 0x238,
    def_skill: SkillName::Fencing,
    def_type: WeaponType::Piercing,
    def_animation: WeaponAnimation::Pierce2H,
    ..BASE_MELEE_WEAPON_CLASS
};
plain_class!(
    /// `Server.Items.BaseSpear`.
    BASE_SPEAR, "Server.Items.BaseSpear", BASE_MELEE_WEAPON, BASE_SPEAR_CLASS
);

/// `BaseStaff`.
pub static BASE_STAFF_CLASS: WeaponClass = WeaponClass {
    on_hit: wb::bashing_on_hit,
    def_hit_sound: 0x233,
    def_miss_sound: 0x239,
    def_skill: SkillName::Macing,
    def_type: WeaponType::Staff,
    def_animation: WeaponAnimation::Bash2H,
    ..BASE_MELEE_WEAPON_CLASS
};
plain_class!(
    /// `Server.Items.BaseStaff`.
    BASE_STAFF, "Server.Items.BaseStaff", BASE_MELEE_WEAPON, BASE_STAFF_CLASS
);

/// `BaseRanged` (`EffectID`, `AmmoType`, `Ammo` are per leaf: [`RangedClass`]).
pub static BASE_RANGED_CLASS: WeaponClass = WeaponClass {
    on_swing: wb::ranged_on_swing,
    on_hit: wb::ranged_on_hit,
    on_miss: wb::ranged_on_miss,
    on_fired: wb::ranged_on_fired,
    on_craft: wb::ranged_on_craft,
    def_hit_sound: 0x234,
    def_miss_sound: 0x238,
    def_skill: SkillName::Archery,
    def_type: WeaponType::Ranged,
    def_animation: WeaponAnimation::ShootXBow,
    accuracy_skill: SkillName::Archery,
    ..BASE_MELEE_WEAPON_CLASS
};

/// `Server.Items.BaseRanged`: `Balanced` and `Velocity` after the version.
pub static BASE_RANGED: ItemType = ItemType {
    full_name: "Server.Items.BaseRanged",
    parent: Some(&BASE_MELEE_WEAPON),
    family: Some(&BASE_RANGED_CLASS),
    serialize: |w, id, out| {
        (BASE_MELEE_WEAPON.serialize)(w, id, out);
        out.write_i32(0);
        let d = data(w, id);
        out.write_bool(d.is_some_and(|d| d.balanced));
        out.write_i32(d.map_or(0, |d| d.velocity));
    },
    deserialize: |w, id, r| {
        (BASE_MELEE_WEAPON.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        let (balanced, velocity) = (r.read_bool()?, r.read_i32()?);
        if let Some(d) = data_mut(w, id) {
            d.balanced = balanced;
            d.velocity = velocity;
        }
        Ok(())
    },
    ..BASE_MELEE_WEAPON
};

/// `BaseThrown`.
pub static BASE_THROWN_CLASS: WeaponClass = WeaponClass {
    modify_hit_chance: wb::thrown_modify_hit_chance,
    modify_damage: wb::thrown_modify_damage,
    on_fired: wb::thrown_on_fired,
    on_hit: wb::thrown_on_hit,
    on_miss: wb::thrown_on_miss,
    def_hit_sound: 0x5D3,
    def_miss_sound: 0x5D4,
    def_skill: SkillName::Throwing,
    def_animation: WeaponAnimation::Throwing,
    accuracy_skill: SkillName::Throwing,
    ..BASE_RANGED_CLASS
};

/// `Server.Items.BaseThrown`: `[SerializationGenerator(0)]` encodes its version as an encoded int.
pub static BASE_THROWN: ItemType = ItemType {
    full_name: "Server.Items.BaseThrown",
    parent: Some(&BASE_RANGED),
    family: Some(&BASE_THROWN_CLASS),
    serialize: |w, id, out| {
        (BASE_RANGED.serialize)(w, id, out);
        out.write_encoded_int(0);
    },
    deserialize: |w, id, r| {
        (BASE_RANGED.deserialize)(w, id, r)?;
        let _version = r.read_encoded_int()?;
        Ok(())
    },
    get_properties: wb::thrown_get_properties,
    ..BASE_RANGED
};

/// The `Configure` phases of the weapon code: `BaseWeapon.Configure` and `WeaponAbilityPackets.Configure`.
pub fn register(types: &mut muo_server::type_registry::TypeRegistry) {
    use muo_server::type_registry::Phase;
    types.register_phase_default(
        Phase::Configure,
        "Server.Items.BaseWeapon.Configure",
        wc::configure,
    );
    types.register_phase_default(
        Phase::Configure,
        "Server.Items.WeaponAbilityPackets.Configure",
        super::weapon_ability::configure_packets,
    );
}
