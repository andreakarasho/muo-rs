//! The `Skills` of a mobile as `[PropertyObject]`s: `[get Skills.Magery.Base`, `[set Skills.Magery.Cap 120`.
//! `Skill.Name`/`Lock`/`Value`/`NonRacialValue` are read only (the lock is set through its own call).

use muo_server::ids::EntityId;
use muo_server::mobile::AccessLevel;
use muo_server::skills::{SkillName, SkillTable};

use crate::props::{Kind, PropDef, Value};
use crate::utilities::types::PropType;

const CNS: AccessLevel = AccessLevel::Counselor;
const GM: AccessLevel = AccessLevel::GameMaster;
const SKILL_LOCKS: &[(&str, i64)] = &[("Up", 0), ("Down", 1), ("Locked", 2)];

fn skill_of(e: EntityId, index: i32) -> Option<(muo_server::ids::MobileId, SkillName)> {
    match e {
        EntityId::Mobile(m) => SkillName::from_i32(index).map(|s| (m, s)),
        EntityId::Item(_) => None,
    }
}

macro_rules! skill_prop {
    ($name:literal, $idx:expr) => {
        PropDef {
            name: $name,
            kind: Kind::Object(
                "Skill",
                &[
                    PropDef {
                        name: "Name",
                        kind: Kind::Plain(PropType::Str),
                        read: CNS,
                        write: CNS,
                        get: |w, e| match skill_of(e, $idx) {
                            Some(_) => w
                                .resources
                                .get::<SkillTable>()
                                .and_then(|t| t.info.get($idx as usize))
                                .map_or(Value::Null, |i| Value::Str(i.name.to_string())),
                            None => Value::Null,
                        },
                        set: None,
                    },
                    PropDef {
                        name: "Lock",
                        kind: Kind::Plain(PropType::Enum(SKILL_LOCKS)),
                        read: CNS,
                        write: CNS,
                        get: |w, e| match skill_of(e, $idx) {
                            Some((m, s)) => Value::Enum(
                                w.mobile(m)
                                    .and_then(|m| m.skills.as_ref())
                                    .and_then(|k| k.peek(s as usize))
                                    .map_or(0, |k| k.lock as i64),
                            ),
                            None => Value::Null,
                        },
                        set: None,
                    },
                    PropDef {
                        name: "Base",
                        kind: Kind::Plain(PropType::Double),
                        read: CNS,
                        write: GM,
                        get: |w, e| match skill_of(e, $idx) {
                            Some((m, s)) => Value::Float(
                                w.mobile(m)
                                    .and_then(|m| m.skills.as_ref())
                                    .and_then(|k| k.peek(s as usize))
                                    .map_or(0.0, |k| f64::from(k.base) / 10.0),
                            ),
                            None => Value::Null,
                        },
                        set: Some(|w, e, v| {
                            if let Some((m, s)) = skill_of(e, $idx) {
                                muo_server::skills::set_base(w, m, s, v.f64());
                            }
                            Ok(())
                        }),
                    },
                    PropDef {
                        name: "Cap",
                        kind: Kind::Plain(PropType::Double),
                        read: CNS,
                        write: GM,
                        get: |w, e| match skill_of(e, $idx) {
                            Some((m, s)) => Value::Float(
                                w.mobile(m)
                                    .and_then(|m| m.skills.as_ref())
                                    .and_then(|k| k.peek(s as usize))
                                    .map_or(100.0, |k| f64::from(k.cap) / 10.0),
                            ),
                            None => Value::Null,
                        },
                        set: Some(|w, e, v| {
                            if let Some((m, s)) = skill_of(e, $idx) {
                                muo_server::skills::set_cap(w, m, s, v.f64());
                            }
                            Ok(())
                        }),
                    },
                    PropDef {
                        name: "Value",
                        kind: Kind::Plain(PropType::Double),
                        read: CNS,
                        write: CNS,
                        get: |w, e| match skill_of(e, $idx) {
                            Some((m, s)) => Value::Float(muo_server::skills::value_ref(w, m, s)),
                            None => Value::Null,
                        },
                        set: None,
                    },
                    PropDef {
                        name: "NonRacialValue",
                        kind: Kind::Plain(PropType::Double),
                        read: CNS,
                        write: CNS,
                        get: |w, e| match skill_of(e, $idx) {
                            Some((m, s)) => {
                                Value::Float(muo_server::skills::non_racial_ref(w, m, s))
                            }
                            None => Value::Null,
                        },
                        set: None,
                    },
                ],
            ),
            read: CNS,
            write: CNS,
            get: |w, e| match skill_of(e, $idx) {
                Some((m, _)) if w.mobile(m).is_some_and(|m| m.skills.is_some()) => {
                    Value::Text("...".into())
                }
                _ => Value::Null,
            },
            set: None,
        }
    };
}

/// `Skills`' `[CommandProperty]` members: `Cap` and one `Skill` per `SkillName`.
pub static SKILLS: &[PropDef] = &[
    PropDef {
        name: "Cap",
        kind: Kind::Plain(PropType::Numeric(crate::utilities::types::Numeric::Int)),
        read: CNS,
        write: GM,
        get: |w, e| match e {
            EntityId::Mobile(m) => w
                .mobile(m)
                .and_then(|m| m.skills.as_ref())
                .map_or(Value::Null, |k| Value::Int(i64::from(k.cap))),
            EntityId::Item(_) => Value::Null,
        },
        set: Some(|w, e, v| {
            if let EntityId::Mobile(m) = e
                && let Some(k) = w.mobile_mut(m).and_then(|m| m.skills.as_mut())
            {
                k.cap = v.i32();
            }
            Ok(())
        }),
    },
    skill_prop!("Alchemy", 0),
    skill_prop!("Anatomy", 1),
    skill_prop!("AnimalLore", 2),
    skill_prop!("ItemID", 3),
    skill_prop!("ArmsLore", 4),
    skill_prop!("Parry", 5),
    skill_prop!("Begging", 6),
    skill_prop!("Blacksmith", 7),
    skill_prop!("Fletching", 8),
    skill_prop!("Peacemaking", 9),
    skill_prop!("Camping", 10),
    skill_prop!("Carpentry", 11),
    skill_prop!("Cartography", 12),
    skill_prop!("Cooking", 13),
    skill_prop!("DetectHidden", 14),
    skill_prop!("Discordance", 15),
    skill_prop!("EvalInt", 16),
    skill_prop!("Healing", 17),
    skill_prop!("Fishing", 18),
    skill_prop!("Forensics", 19),
    skill_prop!("Herding", 20),
    skill_prop!("Hiding", 21),
    skill_prop!("Provocation", 22),
    skill_prop!("Inscribe", 23),
    skill_prop!("Lockpicking", 24),
    skill_prop!("Magery", 25),
    skill_prop!("MagicResist", 26),
    skill_prop!("Tactics", 27),
    skill_prop!("Snooping", 28),
    skill_prop!("Musicianship", 29),
    skill_prop!("Poisoning", 30),
    skill_prop!("Archery", 31),
    skill_prop!("SpiritSpeak", 32),
    skill_prop!("Stealing", 33),
    skill_prop!("Tailoring", 34),
    skill_prop!("AnimalTaming", 35),
    skill_prop!("TasteID", 36),
    skill_prop!("Tinkering", 37),
    skill_prop!("Tracking", 38),
    skill_prop!("Veterinary", 39),
    skill_prop!("Swords", 40),
    skill_prop!("Macing", 41),
    skill_prop!("Fencing", 42),
    skill_prop!("Wrestling", 43),
    skill_prop!("Lumberjacking", 44),
    skill_prop!("Mining", 45),
    skill_prop!("Meditation", 46),
    skill_prop!("Stealth", 47),
    skill_prop!("RemoveTrap", 48),
    skill_prop!("Necromancy", 49),
    skill_prop!("Focus", 50),
    skill_prop!("Chivalry", 51),
    skill_prop!("Bushido", 52),
    skill_prop!("Ninjitsu", 53),
    skill_prop!("Spellweaving", 54),
    skill_prop!("Mysticism", 55),
    skill_prop!("Imbuing", 56),
    skill_prop!("Throwing", 57),
];
