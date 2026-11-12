//! The `[CommandProperty]` members `[props` / `[set` / `where` show for `BaseWeapon`, `BaseArmor`, `BaseClothing`
//! and `BaseJewel`: the `[SerializableField]` + `[SerializedCommandProperty]` fields
//! and the auto-properties next to them.
//!
//! A setter writes the field and calls `InvalidateProperties()` where the field carries `[InvalidateProperties]`
//! (every weapon field, `BaseArmor`'s `Identified`/`Crafter`/`MaxHitPoints`.., see the notes on each table).
//! The `Aos*` attribute objects (`canModify: true`) are `Kind::Object`s over the owner's bag: `[set Attributes.Luck 100`.
//!
//! The members whose property runs code and has no field of its own (`BaseWeapon.AccuracyLevel` / `Quality` /
//! `Resource` / `Hue`.., `BaseArmor.BaseArmorRating`.., `BaseClothing.HitPoints`, `BaseJewel.HitPoints`) are in
//! `props_gen.rs`, next to the family modules' scaling and durability code.
//!

use muo_server::ids::{EntityId, ItemId};
use muo_server::item::ops as iops;
use muo_server::mobile::AccessLevel;
use muo_server::world::World;

use crate::props::{self, Kind, PropDef, Value};
use crate::systems::utilities::types::{Numeric, PropType};

const GM: AccessLevel = AccessLevel::GameMaster;
const INT: Kind = Kind::Plain(PropType::Numeric(Numeric::Int));
const BOOL: Kind = Kind::Plain(PropType::Bool);
const STR: Kind = Kind::Plain(PropType::Str);

const SLAYERS: &[(&str, i64)] = &[
    ("None", 0),
    ("Silver", 1),
    ("OrcSlaying", 2),
    ("TrollSlaughter", 3),
    ("OgreTrashing", 4),
    ("Repond", 5),
    ("DragonSlaying", 6),
    ("Terathan", 7),
    ("SnakesBane", 8),
    ("LizardmanSlaughter", 9),
    ("ReptilianDeath", 10),
    ("DaemonDismissal", 11),
    ("GargoylesFoe", 12),
    ("BalronDamnation", 13),
    ("Exorcism", 14),
    ("Ophidian", 15),
    ("SpidersDeath", 16),
    ("ScorpionsBane", 17),
    ("ArachnidDoom", 18),
    ("FlameDousing", 19),
    ("WaterDissipation", 20),
    ("Vacuum", 21),
    ("ElementalHealth", 22),
    ("EarthShatter", 23),
    ("BloodDrinking", 24),
    ("SummerWind", 25),
    ("ElementalBan", 26),
    ("Fey", 27),
];
const WEAPON_DAMAGE: &[(&str, i64)] = &[
    ("Regular", 0),
    ("Ruin", 1),
    ("Might", 2),
    ("Force", 3),
    ("Power", 4),
    ("Vanq", 5),
];
const ARMOR_PROTECTION: &[(&str, i64)] = &[
    ("Regular", 0),
    ("Defense", 1),
    ("Guarding", 2),
    ("Hardening", 3),
    ("Fortification", 4),
    ("Invulnerability", 5),
];
const QUALITY: &[(&str, i64)] = &[("Low", 0), ("Regular", 1), ("Exceptional", 2)];
const GEMS: &[(&str, i64)] = &[
    ("None", 0),
    ("StarSapphire", 1),
    ("Emerald", 2),
    ("Sapphire", 3),
    ("Ruby", 4),
    ("Citrine", 5),
    ("Amethyst", 6),
    ("Tourmaline", 7),
    ("Amber", 8),
    ("Diamond", 9),
];
/// `CraftResource`'s members.
pub const RESOURCES: &[(&str, i64)] = &[
    ("None", 0),
    ("Iron", 1),
    ("DullCopper", 2),
    ("ShadowIron", 3),
    ("Copper", 4),
    ("Bronze", 5),
    ("Gold", 6),
    ("Agapite", 7),
    ("Verite", 8),
    ("Valorite", 9),
    ("RegularLeather", 101),
    ("SpinedLeather", 102),
    ("HornedLeather", 103),
    ("BarbedLeather", 104),
    ("RedScales", 201),
    ("YellowScales", 202),
    ("BlackScales", 203),
    ("GreenScales", 204),
    ("WhiteScales", 205),
    ("BlueScales", 206),
    ("RegularWood", 301),
    ("OakWood", 302),
    ("AshWood", 303),
    ("YewWood", 304),
    ("Heartwood", 305),
    ("Bloodwood", 306),
    ("Frostwood", 307),
];

fn item_of(e: EntityId) -> Option<ItemId> {
    match e {
        EntityId::Item(i) => Some(i),
        _ => None,
    }
}

fn opt_str(s: Option<&String>) -> Value {
    s.map_or(Value::Null, |s| Value::Str(s.clone()))
}

/// Empty text sets a null string, as `[set Crafter ""` does through the string parser.
fn new_str(v: &Value) -> Option<String> {
    v.string().filter(|s| !s.is_empty())
}

/// A property over the data struct `$d` of the family (`$data` / `$data_mut` read and write it).
macro_rules! field {
    ($data:path, $data_mut:path, $name:literal, $kind:expr, |$gd:ident| $get:expr,
     |$sd:ident, $sv:ident| $set:expr, $inval:expr) => {
        PropDef {
            name: $name,
            kind: $kind,
            read: GM,
            write: GM,
            get: |w: &mut World, e: EntityId| {
                item_of(e)
                    .and_then(|i| $data(w, i))
                    .map_or(Value::Null, |$gd| $get)
            },
            set: Some(|w: &mut World, e: EntityId, v: Value| {
                let Some(i) = item_of(e) else {
                    return Err("That is not an item.".to_string());
                };
                if let Some($sd) = $data_mut(w, i) {
                    let $sv = &v;
                    $set;
                }
                if $inval {
                    iops::invalidate_properties(w, i);
                }
                Ok(())
            }),
        }
    };
}

/// The `Aos*` objects with `canModify: true`: a property whose value is the bag (`...`, or null for an item that has
/// none) and whose sub-properties are the bag's members.
macro_rules! aos_object {
    ($name:literal, $cname:literal, $kind:expr, $subs:expr) => {
        PropDef {
            name: $name,
            kind: Kind::Object($cname, $subs),
            read: GM,
            write: GM,
            get: |w, e| match e {
                EntityId::Item(i)
                    if w.item(i)
                        .and_then(|x| crate::items::attrs(x, $kind))
                        .is_some() =>
                {
                    Value::Text("...".into())
                }
                _ => Value::Null,
            },
            set: None,
        }
    };
}

/// `bag[attribute]`: an `int` member of a `BaseAttributes` subclass (`GetValue` / `SetValue` of the owner's bag).
macro_rules! bag_int {
    ($name:literal, $kind:expr, $mask:expr) => {
        PropDef {
            name: $name,
            kind: INT,
            read: GM,
            write: GM,
            get: |w, e| match e {
                EntityId::Item(i) => {
                    Value::Int(i64::from(crate::items::attr(w, i, $kind, $mask as u32)))
                }
                _ => Value::Null,
            },
            set: Some(|w, e, v| {
                if let EntityId::Item(i) = e {
                    crate::items::set_attr(w, i, $kind, $mask as u32, v.i32());
                }
                Ok(())
            }),
        }
    };
}

/// The skill and bonus held by slot `index` of an `AosSkillBonuses` (`GetValues`, the skill even when the bonus is 0).
fn skill_slot(w: &World, i: ItemId, index: u32) -> (i32, f64) {
    let mut v = crate::items::attr(w, i, AttrKind::SkillBonuses, 1 << index);
    let (mut skill, mut bonus) = (0_i32, 0_i32);
    for _ in 0..16 {
        skill = (skill << 1) | (v & 1);
        v >>= 1;
        bonus = (bonus << 1) | (v & 1);
        v >>= 1;
    }
    (skill, f64::from(bonus) / 10.0)
}

/// `SetValues(index, skill, bonus)`: the packed value `Attrs::skill_bonus_value` makes.
fn set_skill_slot(w: &mut World, i: ItemId, index: u32, skill: i32, bonus: f64) {
    let (mut v, mut v_skill, mut v_bonus) = (0_i32, skill, (bonus * 10.0) as i32);
    for _ in 0..16 {
        v = (v << 1) | (v_bonus & 1);
        v_bonus >>= 1;
        v = (v << 1) | (v_skill & 1);
        v_skill >>= 1;
    }
    crate::items::set_attr(w, i, AttrKind::SkillBonuses, 1 << index, v);
}

macro_rules! skill_slot_props {
    ($i:expr, $value:literal, $name:literal) => {
        [
            PropDef {
                name: $value,
                kind: Kind::Plain(PropType::Double),
                read: GM,
                write: GM,
                get: |w, e| match e {
                    EntityId::Item(i) => Value::Float(skill_slot(w, i, $i).1),
                    _ => Value::Null,
                },
                set: Some(|w, e, v| {
                    if let EntityId::Item(i) = e {
                        let skill = skill_slot(w, i, $i).0;
                        set_skill_slot(w, i, $i, skill, v.f64());
                    }
                    Ok(())
                }),
            },
            PropDef {
                name: $name,
                kind: Kind::Plain(PropType::Enum(props::SKILL_NAMES)),
                read: GM,
                write: GM,
                get: |w, e| match e {
                    EntityId::Item(i) => Value::Enum(i64::from(skill_slot(w, i, $i).0)),
                    _ => Value::Null,
                },
                set: Some(|w, e, v| {
                    if let EntityId::Item(i) = e {
                        let bonus = skill_slot(w, i, $i).1;
                        set_skill_slot(w, i, $i, v.i32(), bonus);
                    }
                    Ok(())
                }),
            },
        ]
    };
}

use crate::aos::{
    AosArmorAttribute as AAA, AosAttribute as AA, AosElementAttribute as AEA,
    AosWeaponAttribute as AWA, AttrKind,
};

pub static AOS_ATTRIBUTES: &[PropDef] = &[
    bag_int!("RegenHits", AttrKind::Attributes, AA::RegenHits),
    bag_int!("RegenStam", AttrKind::Attributes, AA::RegenStam),
    bag_int!("RegenMana", AttrKind::Attributes, AA::RegenMana),
    bag_int!("DefendChance", AttrKind::Attributes, AA::DefendChance),
    bag_int!("AttackChance", AttrKind::Attributes, AA::AttackChance),
    bag_int!("BonusStr", AttrKind::Attributes, AA::BonusStr),
    bag_int!("BonusDex", AttrKind::Attributes, AA::BonusDex),
    bag_int!("BonusInt", AttrKind::Attributes, AA::BonusInt),
    bag_int!("BonusHits", AttrKind::Attributes, AA::BonusHits),
    bag_int!("BonusStam", AttrKind::Attributes, AA::BonusStam),
    bag_int!("BonusMana", AttrKind::Attributes, AA::BonusMana),
    bag_int!("WeaponDamage", AttrKind::Attributes, AA::WeaponDamage),
    bag_int!("WeaponSpeed", AttrKind::Attributes, AA::WeaponSpeed),
    bag_int!("SpellDamage", AttrKind::Attributes, AA::SpellDamage),
    bag_int!("CastRecovery", AttrKind::Attributes, AA::CastRecovery),
    bag_int!("CastSpeed", AttrKind::Attributes, AA::CastSpeed),
    bag_int!("LowerManaCost", AttrKind::Attributes, AA::LowerManaCost),
    bag_int!("LowerRegCost", AttrKind::Attributes, AA::LowerRegCost),
    bag_int!("ReflectPhysical", AttrKind::Attributes, AA::ReflectPhysical),
    bag_int!("EnhancePotions", AttrKind::Attributes, AA::EnhancePotions),
    bag_int!("Luck", AttrKind::Attributes, AA::Luck),
    bag_int!("SpellChanneling", AttrKind::Attributes, AA::SpellChanneling),
    bag_int!("NightSight", AttrKind::Attributes, AA::NightSight),
    bag_int!(
        "IncreasedKarmaLoss",
        AttrKind::Attributes,
        AA::IncreasedKarmaLoss
    ),
];

pub static AOS_WEAPON_ATTRIBUTES: &[PropDef] = &[
    bag_int!("LowerStatReq", AttrKind::Weapon, AWA::LowerStatReq),
    bag_int!("SelfRepair", AttrKind::Weapon, AWA::SelfRepair),
    bag_int!("HitLeechHits", AttrKind::Weapon, AWA::HitLeechHits),
    bag_int!("HitLeechStam", AttrKind::Weapon, AWA::HitLeechStam),
    bag_int!("HitLeechMana", AttrKind::Weapon, AWA::HitLeechMana),
    bag_int!("HitLowerAttack", AttrKind::Weapon, AWA::HitLowerAttack),
    bag_int!("HitLowerDefend", AttrKind::Weapon, AWA::HitLowerDefend),
    bag_int!("HitMagicArrow", AttrKind::Weapon, AWA::HitMagicArrow),
    bag_int!("HitHarm", AttrKind::Weapon, AWA::HitHarm),
    bag_int!("HitFireball", AttrKind::Weapon, AWA::HitFireball),
    bag_int!("HitLightning", AttrKind::Weapon, AWA::HitLightning),
    bag_int!("HitDispel", AttrKind::Weapon, AWA::HitDispel),
    bag_int!("HitColdArea", AttrKind::Weapon, AWA::HitColdArea),
    bag_int!("HitFireArea", AttrKind::Weapon, AWA::HitFireArea),
    bag_int!("HitPoisonArea", AttrKind::Weapon, AWA::HitPoisonArea),
    bag_int!("HitEnergyArea", AttrKind::Weapon, AWA::HitEnergyArea),
    bag_int!("HitPhysicalArea", AttrKind::Weapon, AWA::HitPhysicalArea),
    bag_int!(
        "ResistPhysicalBonus",
        AttrKind::Weapon,
        AWA::ResistPhysicalBonus
    ),
    bag_int!("ResistFireBonus", AttrKind::Weapon, AWA::ResistFireBonus),
    bag_int!("ResistColdBonus", AttrKind::Weapon, AWA::ResistColdBonus),
    bag_int!(
        "ResistPoisonBonus",
        AttrKind::Weapon,
        AWA::ResistPoisonBonus
    ),
    bag_int!(
        "ResistEnergyBonus",
        AttrKind::Weapon,
        AWA::ResistEnergyBonus
    ),
    bag_int!("UseBestSkill", AttrKind::Weapon, AWA::UseBestSkill),
    bag_int!("MageWeapon", AttrKind::Weapon, AWA::MageWeapon),
    bag_int!("DurabilityBonus", AttrKind::Weapon, AWA::DurabilityBonus),
];

pub static AOS_ARMOR_ATTRIBUTES: &[PropDef] = &[
    bag_int!("LowerStatReq", AttrKind::Armor, AAA::LowerStatReq),
    bag_int!("SelfRepair", AttrKind::Armor, AAA::SelfRepair),
    bag_int!("MageArmor", AttrKind::Armor, AAA::MageArmor),
    bag_int!("DurabilityBonus", AttrKind::Armor, AAA::DurabilityBonus),
];

pub static AOS_ELEMENT_ATTRIBUTES: &[PropDef] = &[
    bag_int!("Physical", AttrKind::Element, AEA::Physical),
    bag_int!("Fire", AttrKind::Element, AEA::Fire),
    bag_int!("Cold", AttrKind::Element, AEA::Cold),
    bag_int!("Poison", AttrKind::Element, AEA::Poison),
    bag_int!("Energy", AttrKind::Element, AEA::Energy),
    bag_int!("Chaos", AttrKind::Element, AEA::Chaos),
    bag_int!("Direct", AttrKind::Element, AEA::Direct),
];

pub static AOS_SKILL_BONUSES: &[PropDef] = &{
    let [a0, a1] = skill_slot_props!(0, "Skill_1_Value", "Skill_1_Name");
    let [b0, b1] = skill_slot_props!(1, "Skill_2_Value", "Skill_2_Name");
    let [c0, c1] = skill_slot_props!(2, "Skill_3_Value", "Skill_3_Name");
    let [d0, d1] = skill_slot_props!(3, "Skill_4_Value", "Skill_4_Name");
    let [e0, e1] = skill_slot_props!(4, "Skill_5_Value", "Skill_5_Name");
    [a0, a1, b0, b1, c0, c1, d0, d1, e0, e1]
};

mod weapon_table {
    use super::*;
    use crate::items::weapon::{data, data_mut};

    pub static TABLE: &[PropDef] = &[
        field!(
            data,
            data_mut,
            "DamageLevel",
            Kind::Plain(PropType::Enum(WEAPON_DAMAGE)),
            |d| Value::Enum(d.damage_level as i64),
            |d, v| if let Some(x) = crate::items::weapon::WeaponDamageLevel::from_i32(v.i32()) {
                d.damage_level = x
            },
            true
        ),
        field!(
            data,
            data_mut,
            "MaxHitPoints",
            INT,
            |d| Value::Int(i64::from(d.max_hit_points)),
            |d, v| d.max_hit_points = v.i32(),
            true
        ),
        field!(
            data,
            data_mut,
            "Slayer",
            Kind::Plain(PropType::Enum(SLAYERS)),
            |d| Value::Enum(i64::from(d.slayer)),
            |d, v| d.slayer = v.i32(),
            true
        ),
        field!(
            data,
            data_mut,
            "Poison",
            Kind::Poison,
            |d| Value::Poison(d.poison),
            |d, v| if let Value::Poison(p) = v {
                d.poison = *p
            } else if v.is_null() {
                d.poison = None
            },
            true
        ),
        field!(
            data,
            data_mut,
            "PoisonCharges",
            INT,
            |d| Value::Int(i64::from(d.poison_charges)),
            |d, v| d.poison_charges = v.i32(),
            true
        ),
        field!(
            data,
            data_mut,
            "Crafter",
            STR,
            |d| opt_str(d.crafter.as_ref()),
            |d, v| d.crafter = new_str(v),
            true
        ),
        field!(
            data,
            data_mut,
            "Identified",
            BOOL,
            |d| Value::Bool(d.identified),
            |d, v| d.identified = v.bool(),
            true
        ),
        aos_object!(
            "Attributes",
            "AosAttributes",
            AttrKind::Attributes,
            AOS_ATTRIBUTES
        ),
        aos_object!(
            "WeaponAttributes",
            "AosWeaponAttributes",
            AttrKind::Weapon,
            AOS_WEAPON_ATTRIBUTES
        ),
        aos_object!(
            "SkillBonuses",
            "AosSkillBonuses",
            AttrKind::SkillBonuses,
            AOS_SKILL_BONUSES
        ),
        field!(
            data,
            data_mut,
            "Slayer2",
            Kind::Plain(PropType::Enum(SLAYERS)),
            |d| Value::Enum(i64::from(d.slayer2)),
            |d, v| d.slayer2 = v.i32(),
            true
        ),
        aos_object!(
            "AosElementDamages",
            "AosElementAttributes",
            AttrKind::Element,
            AOS_ELEMENT_ATTRIBUTES
        ),
        field!(
            data,
            data_mut,
            "EngravedText",
            STR,
            |d| opt_str(d.engraved_text.as_ref()),
            |d, v| d.engraved_text = new_str(v),
            true
        ),
        PropDef {
            name: "Resource",
            kind: Kind::Plain(PropType::Enum(RESOURCES)),
            read: GM,
            write: GM,
            get: |w, e| match item_of(e).and_then(|i| data(w, i)) {
                Some(d) => Value::Enum(d.resource as i64),
                None => Value::Null,
            },
            set: Some(|w, e, v| {
                if let (Some(i), Some(r)) =
                    (item_of(e), crate::craft::CraftResource::from_i32(v.i32()))
                {
                    crate::items::weapon::set_resource(w, i, r);
                }
                Ok(())
            }),
        },
        field!(
            data,
            data_mut,
            "Cursed",
            BOOL,
            |d| Value::Bool(d.cursed),
            |d, v| d.cursed = v.bool(),
            false
        ),
        field!(
            data,
            data_mut,
            "Consecrated",
            BOOL,
            |d| Value::Bool(d.consecrated),
            |d, v| d.consecrated = v.bool(),
            false
        ),
    ];
}

mod armor_table {
    use super::*;
    use crate::items::armor::{data, data_mut};

    macro_rules! bonus {
        ($name:literal, $f:ident) => {
            field!(
                data,
                data_mut,
                $name,
                INT,
                |d| Value::Int(i64::from(d.$f)),
                |d, v| d.$f = v.i32(),
                true
            )
        };
    }

    pub static TABLE: &[PropDef] = &[
        aos_object!(
            "Attributes",
            "AosAttributes",
            AttrKind::Attributes,
            AOS_ATTRIBUTES
        ),
        aos_object!(
            "ArmorAttributes",
            "AosArmorAttributes",
            AttrKind::Armor,
            AOS_ARMOR_ATTRIBUTES
        ),
        bonus!("PhysicalBonus", physical_bonus),
        bonus!("FireBonus", fire_bonus),
        bonus!("ColdBonus", cold_bonus),
        bonus!("PoisonBonus", poison_bonus),
        bonus!("EnergyBonus", energy_bonus),
        PropDef {
            name: "Resource",
            kind: Kind::Plain(PropType::Enum(RESOURCES)),
            read: GM,
            write: GM,
            get: |w, e| match item_of(e).and_then(|i| data(w, i)) {
                Some(d) => Value::Enum(d.resource as i64),
                None => Value::Null,
            },
            set: Some(|w, e, v| {
                if let (Some(i), Some(r)) =
                    (item_of(e), crate::craft::CraftResource::from_i32(v.i32()))
                {
                    crate::items::armor::set_resource(w, i, r);
                }
                Ok(())
            }),
        },
        field!(
            data,
            data_mut,
            "Identified",
            BOOL,
            |d| Value::Bool(d.identified),
            |d, v| d.identified = v.bool(),
            true
        ),
        bonus!("MaxHitPoints", max_hit_points),
        field!(
            data,
            data_mut,
            "Crafter",
            STR,
            |d| opt_str(d.crafter.as_ref()),
            |d, v| d.crafter = new_str(v),
            true
        ),
        aos_object!(
            "SkillBonuses",
            "AosSkillBonuses",
            AttrKind::SkillBonuses,
            AOS_SKILL_BONUSES
        ),
        field!(
            data,
            data_mut,
            "ProtectionLevel",
            Kind::Plain(PropType::Enum(ARMOR_PROTECTION)),
            |d| Value::Enum(d.protection_level as i64),
            |d, v| if let Some(x) = crate::items::armor::ArmorProtectionLevel::from_i32(v.i32()) {
                d.protection_level = x
            },
            true
        ),
    ];
}

mod clothing_table {
    use super::*;
    use crate::items::clothing::{data, data_mut};

    pub static TABLE: &[PropDef] = &[
        aos_object!(
            "Attributes",
            "AosAttributes",
            AttrKind::Attributes,
            AOS_ATTRIBUTES
        ),
        aos_object!(
            "ClothingAttributes",
            "AosArmorAttributes",
            AttrKind::Armor,
            AOS_ARMOR_ATTRIBUTES
        ),
        aos_object!(
            "SkillBonuses",
            "AosSkillBonuses",
            AttrKind::SkillBonuses,
            AOS_SKILL_BONUSES
        ),
        aos_object!(
            "Resistances",
            "AosElementAttributes",
            AttrKind::Element,
            AOS_ELEMENT_ATTRIBUTES
        ),
        field!(
            data,
            data_mut,
            "MaxHitPoints",
            INT,
            |d| Value::Int(i64::from(d.max_hit_points)),
            |d, v| d.max_hit_points = v.i32(),
            true
        ),
        field!(
            data,
            data_mut,
            "Crafter",
            STR,
            |d| opt_str(d.crafter.as_ref()),
            |d, v| d.crafter = new_str(v),
            true
        ),
        field!(
            data,
            data_mut,
            "Quality",
            Kind::Plain(PropType::Enum(QUALITY)),
            |d| Value::Enum(d.quality as i64),
            |d, v| if let Some(x) = crate::items::clothing::ClothingQuality::from_i32(v.i32()) {
                d.quality = x
            },
            true
        ),
        field!(
            data,
            data_mut,
            "Resource",
            Kind::Plain(PropType::Enum(RESOURCES)),
            |d| Value::Enum(d.resource as i64),
            |d, v| if let Some(x) = crate::craft::CraftResource::from_i32(v.i32()) {
                d.resource = x
            },
            true
        ),
    ];
}

mod jewel_table {
    use super::*;
    use crate::items::jewel::{data, data_mut};

    pub static TABLE: &[PropDef] = &[
        field!(
            data,
            data_mut,
            "MaxHitPoints",
            INT,
            |d| Value::Int(i64::from(d.max_hit_points)),
            |d, v| d.max_hit_points = v.i32(),
            true
        ),
        field!(
            data,
            data_mut,
            "GemType",
            Kind::Plain(PropType::Enum(GEMS)),
            |d| Value::Enum(d.gem_type as i64),
            |d, v| if let Some(x) = crate::items::jewel::GemType::from_i32(v.i32()) {
                d.gem_type = x
            },
            true
        ),
        aos_object!(
            "Attributes",
            "AosAttributes",
            AttrKind::Attributes,
            AOS_ATTRIBUTES
        ),
        aos_object!(
            "Resistances",
            "AosElementAttributes",
            AttrKind::Element,
            AOS_ELEMENT_ATTRIBUTES
        ),
        aos_object!(
            "SkillBonuses",
            "AosSkillBonuses",
            AttrKind::SkillBonuses,
            AOS_SKILL_BONUSES
        ),
        field!(
            data,
            data_mut,
            "GemCount",
            INT,
            |d| Value::Int(i64::from(d.gem_count)),
            |d, v| d.gem_count = v.i32(),
            true
        ),
        field!(
            data,
            data_mut,
            "Resource",
            Kind::Plain(PropType::Enum(RESOURCES)),
            |d| Value::Enum(d.resource as i64),
            |d, v| if let Some(x) = crate::craft::CraftResource::from_i32(v.i32()) {
                d.resource = x
            },
            true
        ),
    ];
}

/// `BaseTool` and `BaseHarvestTool`: `Crafter`, `UsesRemaining` and `Quality` (`ToolData`).
mod tool_table {
    use super::*;
    use crate::items::tool::{self, ToolQuality};

    const TOOL_QUALITY: &[(&str, i64)] = &[("Low", 0), ("Regular", 1), ("Exceptional", 2)];

    pub static TABLE: &[PropDef] = &[
        PropDef {
            name: "Crafter",
            kind: STR,
            read: GM,
            write: GM,
            get: |w, e| match item_of(e).and_then(|i| tool::data(w, i)) {
                Some(d) => opt_str(d.crafter.as_ref()),
                None => Value::Null,
            },
            set: Some(|w, e, v| {
                if let Some(i) = item_of(e) {
                    tool::set_crafter(w, i, new_str(&v));
                }
                Ok(())
            }),
        },
        PropDef {
            name: "UsesRemaining",
            kind: INT,
            read: GM,
            write: GM,
            get: |w, e| match item_of(e).and_then(|i| tool::data(w, i)) {
                Some(d) => Value::Int(i64::from(d.uses_remaining)),
                None => Value::Null,
            },
            set: Some(|w, e, v| {
                if let Some(i) = item_of(e) {
                    tool::set_uses_remaining(w, i, v.i32());
                }
                Ok(())
            }),
        },
        PropDef {
            name: "Quality",
            kind: Kind::Plain(PropType::Enum(TOOL_QUALITY)),
            read: GM,
            write: GM,
            get: |w, e| match item_of(e).and_then(|i| tool::data(w, i)) {
                Some(d) => Value::Enum(d.quality as i64),
                None => Value::Null,
            },
            set: Some(|w, e, v| {
                if let (Some(i), Some(q)) = (item_of(e), ToolQuality::from_i32(v.i32())) {
                    tool::set_quality(w, i, q);
                }
                Ok(())
            }),
        },
    ];
}

/// Adds the tables; called from [`props::register_core`].
pub fn register(w: &mut World) {
    crate::props_gen::configure(w);
    crate::props_hosts::register(w);
    props::add(w, "Server.Items.BaseWeapon", weapon_table::TABLE);
    props::add(w, "Server.Items.BaseArmor", armor_table::TABLE);
    props::add(w, "Server.Items.BaseClothing", clothing_table::TABLE);
    props::add(w, "Server.Items.BaseJewel", jewel_table::TABLE);
    props::add(w, "Server.Items.BaseTool", tool_table::TABLE);
    props::add(w, "Server.Items.BaseHarvestTool", tool_table::TABLE);
    super::prop_bags::register(w);
}
