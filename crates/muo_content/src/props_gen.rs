//! `[CommandProperty]` tables of classes whose members are fields of a data struct in `ext`.
#![allow(
    unused_imports,
    unused_variables,
    dead_code,
    redundant_semicolons,
    clippy::all
)]
use crate::aos::AosElementAttribute;
use crate::props::{self, Kind, PropDef, Value};
use crate::utilities::types::{Numeric, PropType};
use muo_primitives::{DateTime, Point2D, Point3D, Rectangle2D, TimeSpan};
use muo_server::ids::EntityId;
use muo_server::mobile::AccessLevel;
use muo_server::world::World;

const PLAYER: AccessLevel = AccessLevel::Player;
const COUNSELOR: AccessLevel = AccessLevel::Counselor;
const GM: AccessLevel = AccessLevel::GameMaster;
const SEER: AccessLevel = AccessLevel::Seer;
const ADMIN: AccessLevel = AccessLevel::Administrator;
const DEV: AccessLevel = AccessLevel::Developer;
const OWNER: AccessLevel = AccessLevel::Owner;
const BOOL: Kind = Kind::Plain(PropType::Bool);
const STR: Kind = Kind::Plain(PropType::Str);
const ENTITY: Kind = Kind::Plain(PropType::Entity);
const NOT_HERE: &str = "That object has no such state.";
const FACTION: Kind = Kind::Nullable(&Kind::CustomEnum(
    &[
        "Minax",
        "Council of Mages",
        "True Britannians",
        "Shadowlords",
    ],
    &[
        ("Council of Mages", 0),
        ("Minax", 1),
        ("Shadowlords", 2),
        ("True Britannians", 3),
    ],
));
const TOWN: Kind = Kind::Nullable(&Kind::CustomEnum(
    &[
        "Britain",
        "Magincia",
        "Minoc",
        "Moonglow",
        "Skara Brae",
        "Trinsic",
        "Vesper",
        "Yew",
    ],
    &[
        ("Britain", 0),
        ("Magincia", 1),
        ("Minoc", 2),
        ("Moonglow", 3),
        ("Skara Brae", 4),
        ("Trinsic", 5),
        ("Vesper", 6),
        ("Yew", 7),
    ],
));

const E_ACCESSLEVEL: &[(&str, i64)] = &[
    ("Player", 0),
    ("Counselor", 1),
    ("GameMaster", 2),
    ("Seer", 3),
    ("Administrator", 4),
    ("Developer", 5),
    ("Owner", 6),
];
const E_ARMORBODYTYPE: &[(&str, i64)] = &[
    ("Gorget", 0),
    ("Gloves", 1),
    ("Helmet", 2),
    ("Arms", 3),
    ("Legs", 4),
    ("Chest", 5),
    ("Shield", 6),
];
const E_ARMORDURABILITYLEVEL: &[(&str, i64)] = &[
    ("Regular", 0),
    ("Durable", 1),
    ("Substantial", 2),
    ("Massive", 3),
    ("Fortified", 4),
    ("Indestructible", 5),
];
const E_ARMORQUALITY: &[(&str, i64)] = &[("Low", 0), ("Regular", 1), ("Exceptional", 2)];
const E_CRAFTRESOURCE: &[(&str, i64)] = &[
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
const E_SKILLNAME: &[(&str, i64)] = &[
    ("Alchemy", 0),
    ("Anatomy", 1),
    ("AnimalLore", 2),
    ("ItemID", 3),
    ("ArmsLore", 4),
    ("Parry", 5),
    ("Begging", 6),
    ("Blacksmith", 7),
    ("Fletching", 8),
    ("Peacemaking", 9),
    ("Camping", 10),
    ("Carpentry", 11),
    ("Cartography", 12),
    ("Cooking", 13),
    ("DetectHidden", 14),
    ("Discordance", 15),
    ("EvalInt", 16),
    ("Healing", 17),
    ("Fishing", 18),
    ("Forensics", 19),
    ("Herding", 20),
    ("Hiding", 21),
    ("Provocation", 22),
    ("Inscribe", 23),
    ("Lockpicking", 24),
    ("Magery", 25),
    ("MagicResist", 26),
    ("Tactics", 27),
    ("Snooping", 28),
    ("Musicianship", 29),
    ("Poisoning", 30),
    ("Archery", 31),
    ("SpiritSpeak", 32),
    ("Stealing", 33),
    ("Tailoring", 34),
    ("AnimalTaming", 35),
    ("TasteID", 36),
    ("Tinkering", 37),
    ("Tracking", 38),
    ("Veterinary", 39),
    ("Swords", 40),
    ("Macing", 41),
    ("Fencing", 42),
    ("Wrestling", 43),
    ("Lumberjacking", 44),
    ("Mining", 45),
    ("Meditation", 46),
    ("Stealth", 47),
    ("RemoveTrap", 48),
    ("Necromancy", 49),
    ("Focus", 50),
    ("Chivalry", 51),
    ("Bushido", 52),
    ("Ninjitsu", 53),
    ("Spellweaving", 54),
    ("Mysticism", 55),
    ("Imbuing", 56),
    ("Throwing", 57),
];
const E_TRAPTYPE: &[(&str, i64)] = &[
    ("None", 0),
    ("MagicTrap", 1),
    ("ExplosionTrap", 2),
    ("DartTrap", 3),
    ("PoisonTrap", 4),
];
const E_TREASURELEVEL: &[(&str, i64)] = &[
    ("Level1", 0),
    ("Level2", 1),
    ("Level3", 2),
    ("Level4", 3),
    ("Level5", 4),
    ("Level6", 5),
];
const E_WEAPONACCURACYLEVEL: &[(&str, i64)] = &[
    ("Regular", 0),
    ("Accurate", 1),
    ("Surpassingly", 2),
    ("Eminently", 3),
    ("Exceedingly", 4),
    ("Supremely", 5),
];
const E_WEAPONANIMATION: &[(&str, i64)] = &[
    ("Slash1H", 9),
    ("Pierce1H", 10),
    ("Bash1H", 11),
    ("Bash2H", 12),
    ("Slash2H", 13),
    ("Pierce2H", 14),
    ("ShootBow", 18),
    ("ShootXBow", 19),
    ("Wrestle", 31),
    ("Throwing", 32),
];
const E_WEAPONDURABILITYLEVEL: &[(&str, i64)] = &[
    ("Regular", 0),
    ("Durable", 1),
    ("Substantial", 2),
    ("Massive", 3),
    ("Fortified", 4),
    ("Indestructible", 5),
];
const E_WEAPONQUALITY: &[(&str, i64)] = &[("Low", 0), ("Regular", 1), ("Exceptional", 2)];
const E_WEAPONTYPE: &[(&str, i64)] = &[
    ("Axe", 0),
    ("Slashing", 1),
    ("Staff", 2),
    ("Bashing", 3),
    ("Piercing", 4),
    ("Polearm", 5),
    ("Ranged", 6),
    ("Fists", 7),
];
static SERVER_ITEMS_CORPSE: &[PropDef] = &[
    PropDef {
        name: "TimeOfDeath",
        kind: Kind::DateTime,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::items::corpse::CorpseData>())
                .map_or(Value::Null, |d| {
                    let x = &d.time_of_death;
                    Value::DateTime(*x)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = match v {
                Value::DateTime(t) => t,
                _ => DateTime::MIN,
            };
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::items::corpse::CorpseData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.time_of_death == n {
                return Ok(());
            }
            d.time_of_death = n;
            Ok(())
        }),
    },
    PropDef {
        name: "Killer",
        kind: Kind::Plain(PropType::EntityOf("Mobile")),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::items::corpse::CorpseData>())
                .map_or(Value::Null, |d| {
                    let x = &d.killer;
                    Value::Entity(x.map(EntityId::Mobile))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.mobile();
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::items::corpse::CorpseData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.killer == n {
                return Ok(());
            }
            d.killer = n;
            Ok(())
        }),
    },
    PropDef {
        name: "Owner",
        kind: Kind::Plain(PropType::EntityOf("Mobile")),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::items::corpse::CorpseData>())
                .map_or(Value::Null, |d| {
                    let x = &d.owner;
                    Value::Entity(x.map(EntityId::Mobile))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.mobile();
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::items::corpse::CorpseData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.owner == n {
                return Ok(());
            }
            d.owner = n;
            Ok(())
        }),
    },
    PropDef {
        name: "AccessLevel",
        kind: Kind::Plain(PropType::Enum(E_ACCESSLEVEL)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::items::corpse::CorpseData>())
                .map_or(Value::Null, |d| {
                    let x = &d.access_level;
                    Value::Enum(*x as i64)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = match muo_server::mobile::AccessLevel::from_i32(v.i64() as i32) {
                Some(e) => e,
                None => return Err("That is not a member of the enum.".to_string()),
            };
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::items::corpse::CorpseData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.access_level == n {
                return Ok(());
            }
            d.access_level = n;
            Ok(())
        }),
    },
    PropDef {
        name: "EquipItems",
        kind: Kind::Text,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::items::corpse::CorpseData>())
                .map_or(Value::Null, |d| {
                    let x = &d.equip_items;
                    Value::Text("System.Collections.Generic.List`1[Server.Item]".to_string())
                }),
            _ => Value::Null,
        },
        set: None,
    },
    PropDef {
        name: "HairItemId",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::items::corpse::CorpseData>())
                .map_or(Value::Null, |d| {
                    let x = &d.hair_item_id;
                    Value::Int(i64::from(*x))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::items::corpse::CorpseData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.hair_item_id == n {
                return Ok(());
            }
            d.hair_item_id = n;
            Ok(())
        }),
    },
    PropDef {
        name: "HairHue",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::items::corpse::CorpseData>())
                .map_or(Value::Null, |d| {
                    let x = &d.hair_hue;
                    Value::Int(i64::from(*x))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::items::corpse::CorpseData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.hair_hue == n {
                return Ok(());
            }
            d.hair_hue = n;
            Ok(())
        }),
    },
    PropDef {
        name: "FacialHairItemId",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::items::corpse::CorpseData>())
                .map_or(Value::Null, |d| {
                    let x = &d.facial_hair_item_id;
                    Value::Int(i64::from(*x))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::items::corpse::CorpseData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.facial_hair_item_id == n {
                return Ok(());
            }
            d.facial_hair_item_id = n;
            Ok(())
        }),
    },
    PropDef {
        name: "FacialHairHue",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::items::corpse::CorpseData>())
                .map_or(Value::Null, |d| {
                    let x = &d.facial_hair_hue;
                    Value::Int(i64::from(*x))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::items::corpse::CorpseData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.facial_hair_hue == n {
                return Ok(());
            }
            d.facial_hair_hue = n;
            Ok(())
        }),
    },
    PropDef {
        name: "Forensicist",
        kind: STR,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::items::corpse::CorpseData>())
                .map_or(Value::Null, |d| {
                    let x = &d.forensicist;
                    x.clone().map_or(Value::Null, Value::Str)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.string();
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::items::corpse::CorpseData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.forensicist == n {
                return Ok(());
            }
            d.forensicist = n;
            Ok(())
        }),
    },
    PropDef {
        name: "InstancedCorpse",
        kind: BOOL,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::items::corpse::CorpseData>())
                .map_or(Value::Null, |d| {
                    Value::Bool(crate::items::corpse::instanced_corpse(w, i))
                }),
            _ => Value::Null,
        },
        set: None,
    },
    PropDef {
        name: "IsBones",
        kind: BOOL,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::items::corpse::CorpseData>())
                .map_or(Value::Null, |d| {
                    Value::Bool(crate::items::corpse::get_flag(
                        w,
                        i,
                        crate::items::corpse::flag::IS_BONES,
                    ))
                }),
            _ => Value::Null,
        },
        set: None,
    },
    PropDef {
        name: "Devoured",
        kind: BOOL,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::items::corpse::CorpseData>())
                .map_or(Value::Null, |d| {
                    Value::Bool(crate::items::corpse::devoured(w, i))
                }),
            _ => Value::Null,
        },
        set: None,
    },
    PropDef {
        name: "Carved",
        kind: BOOL,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::items::corpse::CorpseData>())
                .map_or(Value::Null, |d| {
                    Value::Bool(crate::items::corpse::get_flag(
                        w,
                        i,
                        crate::items::corpse::flag::CARVED,
                    ))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.bool();
            crate::items::corpse::set_flag(w, i, crate::items::corpse::flag::CARVED, n);
            Ok(())
        }),
    },
    PropDef {
        name: "VisitedByTaxidermist",
        kind: BOOL,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::items::corpse::CorpseData>())
                .map_or(Value::Null, |d| {
                    Value::Bool(crate::items::corpse::get_flag(
                        w,
                        i,
                        crate::items::corpse::flag::VISITED_BY_TAXIDERMIST,
                    ))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.bool();
            crate::items::corpse::set_flag(
                w,
                i,
                crate::items::corpse::flag::VISITED_BY_TAXIDERMIST,
                n,
            );
            Ok(())
        }),
    },
    PropDef {
        name: "Channeled",
        kind: BOOL,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::items::corpse::CorpseData>())
                .map_or(Value::Null, |d| {
                    Value::Bool(crate::items::corpse::get_flag(
                        w,
                        i,
                        crate::items::corpse::flag::CHANNELED,
                    ))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.bool();
            crate::items::corpse::set_flag(w, i, crate::items::corpse::flag::CHANNELED, n);
            Ok(())
        }),
    },
    PropDef {
        name: "Animated",
        kind: BOOL,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::items::corpse::CorpseData>())
                .map_or(Value::Null, |d| {
                    Value::Bool(crate::items::corpse::get_flag(
                        w,
                        i,
                        crate::items::corpse::flag::ANIMATED,
                    ))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.bool();
            crate::items::corpse::set_flag(w, i, crate::items::corpse::flag::ANIMATED, n);
            Ok(())
        }),
    },
    PropDef {
        name: "SelfLooted",
        kind: BOOL,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::items::corpse::CorpseData>())
                .map_or(Value::Null, |d| {
                    Value::Bool(crate::items::corpse::get_flag(
                        w,
                        i,
                        crate::items::corpse::flag::SELF_LOOTED,
                    ))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.bool();
            crate::items::corpse::set_flag(w, i, crate::items::corpse::flag::SELF_LOOTED, n);
            Ok(())
        }),
    },
    PropDef {
        name: "Criminal",
        kind: BOOL,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::items::corpse::CorpseData>())
                .map_or(Value::Null, |d| {
                    Value::Bool(crate::items::corpse::get_flag(
                        w,
                        i,
                        crate::items::corpse::flag::CRIMINAL,
                    ))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.bool();
            crate::items::corpse::set_flag(w, i, crate::items::corpse::flag::CRIMINAL, n);
            Ok(())
        }),
    },
    PropDef {
        name: "Murderer",
        kind: BOOL,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::items::corpse::CorpseData>())
                .map_or(Value::Null, |d| {
                    Value::Bool(crate::items::corpse::get_flag(
                        w,
                        i,
                        crate::items::corpse::flag::MURDERER,
                    ))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.bool();
            crate::items::corpse::set_flag(w, i, crate::items::corpse::flag::MURDERER, n);
            Ok(())
        }),
    },
    PropDef {
        name: "OwnerWasBaseCreature",
        kind: BOOL,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::items::corpse::CorpseData>())
                .map_or(Value::Null, |d| {
                    Value::Bool(crate::items::corpse::get_flag(
                        w,
                        i,
                        crate::items::corpse::flag::OWNER_WAS_BASE_CREATURE,
                    ))
                }),
            _ => Value::Null,
        },
        set: None,
    },
    PropDef {
        name: "OwnerWasSummoned",
        kind: BOOL,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::items::corpse::CorpseData>())
                .map_or(Value::Null, |d| {
                    Value::Bool(crate::items::corpse::get_flag(
                        w,
                        i,
                        crate::items::corpse::flag::OWNER_WAS_SUMMONED,
                    ))
                }),
            _ => Value::Null,
        },
        set: None,
    },
    PropDef {
        name: "OwnerWasAnimatedDead",
        kind: BOOL,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::items::corpse::CorpseData>())
                .map_or(Value::Null, |d| {
                    Value::Bool(crate::items::corpse::get_flag(
                        w,
                        i,
                        crate::items::corpse::flag::OWNER_WAS_ANIMATED_DEAD,
                    ))
                }),
            _ => Value::Null,
        },
        set: None,
    },
];

static SERVER_ITEMS_BASEWEAPON: &[PropDef] = &[
    PropDef {
        name: "AccuracyLevel",
        kind: Kind::Plain(PropType::Enum(E_WEAPONACCURACYLEVEL)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::items::weapon::WeaponData>())
                .map_or(Value::Null, |d| {
                    let x = &d.accuracy_level;
                    Value::Enum(*x as i64)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = match crate::items::weapon::WeaponAccuracyLevel::from_i32(v.i64() as i32) {
                Some(e) => e,
                None => return Err("That is not a member of the enum.".to_string()),
            };
            crate::items::weapon::set_accuracy_level(w, i, n);
            Ok(())
        }),
    },
    PropDef {
        name: "Quality",
        kind: Kind::Plain(PropType::Enum(E_WEAPONQUALITY)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::items::weapon::WeaponData>())
                .map_or(Value::Null, |d| {
                    let x = &d.quality;
                    Value::Enum(*x as i64)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = match crate::items::weapon::WeaponQuality::from_i32(v.i64() as i32) {
                Some(e) => e,
                None => return Err("That is not a member of the enum.".to_string()),
            };
            crate::items::weapon::set_quality(w, i, n);
            Ok(())
        }),
    },
    PropDef {
        name: "HitPoints",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::items::weapon::WeaponData>())
                .map_or(Value::Null, |d| {
                    let x = &d.hit_points;
                    Value::Int(i64::from(*x))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            crate::items::weapon::set_hit_points(w, i, n);
            Ok(())
        }),
    },
    PropDef {
        name: "DurabilityLevel",
        kind: Kind::Plain(PropType::Enum(E_WEAPONDURABILITYLEVEL)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::items::weapon::WeaponData>())
                .map_or(Value::Null, |d| {
                    let x = &d.durability_level;
                    Value::Enum(*x as i64)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = match crate::items::weapon::WeaponDurabilityLevel::from_i32(v.i64() as i32) {
                Some(e) => e,
                None => return Err("That is not a member of the enum.".to_string()),
            };
            crate::items::weapon::set_durability_level(w, i, n);
            Ok(())
        }),
    },
    PropDef {
        name: "Hue",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::items::weapon::WeaponData>())
                .map_or(Value::Null, |d| {
                    Value::Int(i64::from(w.item(i).map_or(0, |x| x.hue)))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i32();
            {
                muo_server::item::base::set_hue(w, i, n);
                muo_server::item::ops::invalidate_properties(w, i);
            };
            Ok(())
        }),
    },
    PropDef {
        name: "StrRequirement",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::items::weapon::WeaponData>())
                .map_or(Value::Null, |d| {
                    Value::Int(i64::from(crate::items::weapon::str_requirement(w, i)))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i32();
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::items::weapon::WeaponData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.str_requirement == n {
                return Ok(());
            }
            d.str_requirement = n;
            muo_server::item::ops::invalidate_properties(w, i);
            Ok(())
        }),
    },
    PropDef {
        name: "DexRequirement",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::items::weapon::WeaponData>())
                .map_or(Value::Null, |d| {
                    Value::Int(i64::from(crate::items::weapon::dex_requirement(w, i)))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i32();
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::items::weapon::WeaponData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.dex_requirement == n {
                return Ok(());
            }
            d.dex_requirement = n;
            muo_server::item::ops::invalidate_properties(w, i);
            Ok(())
        }),
    },
    PropDef {
        name: "IntRequirement",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::items::weapon::WeaponData>())
                .map_or(Value::Null, |d| {
                    Value::Int(i64::from(crate::items::weapon::int_requirement(w, i)))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i32();
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::items::weapon::WeaponData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.int_requirement == n {
                return Ok(());
            }
            d.int_requirement = n;
            muo_server::item::ops::invalidate_properties(w, i);
            Ok(())
        }),
    },
    PropDef {
        name: "MinDamage",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::items::weapon::WeaponData>())
                .map_or(Value::Null, |d| {
                    Value::Int(i64::from(crate::items::weapon::min_damage(w, i)))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i32();
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::items::weapon::WeaponData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.min_damage == n {
                return Ok(());
            }
            d.min_damage = n;
            muo_server::item::ops::invalidate_properties(w, i);
            Ok(())
        }),
    },
    PropDef {
        name: "MaxDamage",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::items::weapon::WeaponData>())
                .map_or(Value::Null, |d| {
                    Value::Int(i64::from(crate::items::weapon::max_damage(w, i)))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i32();
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::items::weapon::WeaponData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.max_damage == n {
                return Ok(());
            }
            d.max_damage = n;
            muo_server::item::ops::invalidate_properties(w, i);
            Ok(())
        }),
    },
    PropDef {
        name: "HitSound",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::items::weapon::WeaponData>())
                .map_or(Value::Null, |d| {
                    Value::Int(i64::from(crate::items::weapon::hit_sound(w, i)))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i32();
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::items::weapon::WeaponData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.hit_sound == n {
                return Ok(());
            }
            d.hit_sound = n;
            Ok(())
        }),
    },
    PropDef {
        name: "MissSound",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::items::weapon::WeaponData>())
                .map_or(Value::Null, |d| {
                    Value::Int(i64::from(crate::items::weapon::miss_sound(w, i)))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i32();
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::items::weapon::WeaponData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.miss_sound == n {
                return Ok(());
            }
            d.miss_sound = n;
            Ok(())
        }),
    },
    PropDef {
        name: "MaxRange",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::items::weapon::WeaponData>())
                .map_or(Value::Null, |d| {
                    Value::Int(i64::from(crate::items::weapon::max_range(w, i)))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i32();
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::items::weapon::WeaponData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.max_range == n {
                return Ok(());
            }
            d.max_range = n;
            muo_server::item::ops::invalidate_properties(w, i);
            Ok(())
        }),
    },
    PropDef {
        name: "Speed",
        kind: Kind::Plain(PropType::Double),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::items::weapon::WeaponData>())
                .map_or(Value::Null, |d| {
                    Value::Float(f64::from(crate::items::weapon::speed(w, i)))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.f64() as f32;
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::items::weapon::WeaponData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.speed == n {
                return Ok(());
            }
            d.speed = n;
            muo_server::item::ops::invalidate_properties(w, i);
            Ok(())
        }),
    },
    PropDef {
        name: "Skill",
        kind: Kind::Plain(PropType::Enum(E_SKILLNAME)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::items::weapon::WeaponData>())
                .map_or(Value::Null, |d| {
                    Value::Enum(crate::items::weapon::skill(w, i) as i64)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::items::weapon::WeaponData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.skill == n {
                return Ok(());
            }
            d.skill = n;
            muo_server::item::ops::invalidate_properties(w, i);
            Ok(())
        }),
    },
    PropDef {
        name: "Type",
        kind: Kind::Plain(PropType::Enum(E_WEAPONTYPE)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::items::weapon::WeaponData>())
                .map_or(Value::Null, |d| {
                    Value::Enum(crate::items::weapon::weapon_type(w, i) as i64)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::items::weapon::WeaponData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.weapon_type == n {
                return Ok(());
            }
            d.weapon_type = n;
            Ok(())
        }),
    },
    PropDef {
        name: "Animation",
        kind: Kind::Plain(PropType::Enum(E_WEAPONANIMATION)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::items::weapon::WeaponData>())
                .map_or(Value::Null, |d| {
                    Value::Enum(crate::items::weapon::animation(w, i) as i64)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::items::weapon::WeaponData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.animation == n {
                return Ok(());
            }
            d.animation = n;
            Ok(())
        }),
    },
];

static SERVER_ITEMS_BASEARMOR: &[PropDef] = &[
    PropDef {
        name: "Quality",
        kind: Kind::Plain(PropType::Enum(E_ARMORQUALITY)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::items::armor::ArmorData>())
                .map_or(Value::Null, |d| {
                    let x = &d.quality;
                    Value::Enum(*x as i64)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = match crate::items::armor::ArmorQuality::from_i32(v.i64() as i32) {
                Some(e) => e,
                None => return Err("That is not a member of the enum.".to_string()),
            };
            crate::items::armor::set_quality(w, i, n);
            Ok(())
        }),
    },
    PropDef {
        name: "HitPoints",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::items::armor::ArmorData>())
                .map_or(Value::Null, |d| {
                    let x = &d.hit_points;
                    Value::Int(i64::from(*x))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            crate::items::armor::set_hit_points(w, i, n);
            Ok(())
        }),
    },
    PropDef {
        name: "Durability",
        kind: Kind::Plain(PropType::Enum(E_ARMORDURABILITYLEVEL)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::items::armor::ArmorData>())
                .map_or(Value::Null, |d| {
                    let x = &d.durability;
                    Value::Enum(*x as i64)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = match crate::items::armor::ArmorDurabilityLevel::from_i32(v.i64() as i32) {
                Some(e) => e,
                None => return Err("That is not a member of the enum.".to_string()),
            };
            crate::items::armor::set_durability_level(w, i, n);
            Ok(())
        }),
    },
    PropDef {
        name: "BaseArmorRating",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::items::armor::ArmorData>())
                .map_or(Value::Null, |d| {
                    Value::Int(i64::from(crate::items::armor::base_armor_rating(w, i)))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i32();
            crate::items::armor::set_base_armor_rating(w, i, n);
            Ok(())
        }),
    },
    PropDef {
        name: "StrBonus",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::items::armor::ArmorData>())
                .map_or(Value::Null, |d| {
                    Value::Int(i64::from(crate::items::armor::str_bonus(w, i)))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i32();
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::items::armor::ArmorData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.str_bonus == n {
                return Ok(());
            }
            d.str_bonus = n;
            muo_server::item::ops::invalidate_properties(w, i);
            Ok(())
        }),
    },
    PropDef {
        name: "DexBonus",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::items::armor::ArmorData>())
                .map_or(Value::Null, |d| {
                    Value::Int(i64::from(crate::items::armor::dex_bonus(w, i)))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i32();
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::items::armor::ArmorData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.dex_bonus == n {
                return Ok(());
            }
            d.dex_bonus = n;
            muo_server::item::ops::invalidate_properties(w, i);
            Ok(())
        }),
    },
    PropDef {
        name: "IntBonus",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::items::armor::ArmorData>())
                .map_or(Value::Null, |d| {
                    Value::Int(i64::from(crate::items::armor::int_bonus(w, i)))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i32();
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::items::armor::ArmorData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.int_bonus == n {
                return Ok(());
            }
            d.int_bonus = n;
            muo_server::item::ops::invalidate_properties(w, i);
            Ok(())
        }),
    },
    PropDef {
        name: "StrRequirement",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::items::armor::ArmorData>())
                .map_or(Value::Null, |d| {
                    Value::Int(i64::from(crate::items::armor::str_requirement(w, i)))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i32();
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::items::armor::ArmorData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.str_req == n {
                return Ok(());
            }
            d.str_req = n;
            muo_server::item::ops::invalidate_properties(w, i);
            Ok(())
        }),
    },
    PropDef {
        name: "DexRequirement",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::items::armor::ArmorData>())
                .map_or(Value::Null, |d| {
                    Value::Int(i64::from(crate::items::armor::dex_requirement(w, i)))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i32();
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::items::armor::ArmorData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.dex_req == n {
                return Ok(());
            }
            d.dex_req = n;
            muo_server::item::ops::invalidate_properties(w, i);
            Ok(())
        }),
    },
    PropDef {
        name: "IntRequirement",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::items::armor::ArmorData>())
                .map_or(Value::Null, |d| {
                    Value::Int(i64::from(crate::items::armor::int_requirement(w, i)))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i32();
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::items::armor::ArmorData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.int_req == n {
                return Ok(());
            }
            d.int_req = n;
            muo_server::item::ops::invalidate_properties(w, i);
            Ok(())
        }),
    },
    PropDef {
        name: "MeditationAllowance",
        kind: Kind::Plain(PropType::EntityOf("AMA")),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::items::armor::ArmorData>())
                .map_or(Value::Null, |d| {
                    Value::Enum(crate::items::armor::meditation_allowance(w, i) as i64)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::items::armor::ArmorData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.meditate == n {
                return Ok(());
            }
            d.meditate = n;
            muo_server::item::ops::invalidate_properties(w, i);
            Ok(())
        }),
    },
    PropDef {
        name: "BodyPosition",
        kind: Kind::Plain(PropType::Enum(E_ARMORBODYTYPE)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::items::armor::ArmorData>())
                .map_or(Value::Null, |d| {
                    Value::Enum(crate::items::armor::body_position(w, i) as i64)
                }),
            _ => Value::Null,
        },
        set: None,
    },
    PropDef {
        name: "Hue",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::items::armor::ArmorData>())
                .map_or(Value::Null, |d| {
                    Value::Int(i64::from(w.item(i).map_or(0, |x| x.hue)))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i32();
            {
                muo_server::item::base::set_hue(w, i, n);
                muo_server::item::ops::invalidate_properties(w, i);
            };
            Ok(())
        }),
    },
];

static SERVER_ITEMS_BASECLOTHING: &[PropDef] = &[
    PropDef {
        name: "HitPoints",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::items::clothing::ClothingData>())
                .map_or(Value::Null, |d| {
                    let x = &d.hit_points;
                    Value::Int(i64::from(*x))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            crate::items::clothing::set_hit_points(w, i, n);
            Ok(())
        }),
    },
    PropDef {
        name: "StrRequirement",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::items::clothing::ClothingData>())
                .map_or(Value::Null, |d| {
                    Value::Int(i64::from(crate::items::clothing::str_requirement(w, i)))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i32();
            crate::items::clothing::set_str_requirement(w, i, n);
            Ok(())
        }),
    },
];

static SERVER_ITEMS_BASEJEWEL: &[PropDef] = &[PropDef {
    name: "HitPoints",
    kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
    read: GM,
    write: GM,
    get: |w, e| match e {
        EntityId::Item(i) => w
            .item(i)
            .and_then(|o| o.ext.get::<crate::items::jewel::JewelData>())
            .map_or(Value::Null, |d| {
                let x = &d.hit_points;
                Value::Int(i64::from(*x))
            }),
        _ => Value::Null,
    },
    set: Some(|w, e, v| {
        let EntityId::Item(i) = e else {
            return Err(NOT_HERE.to_string());
        };
        let n = v.i64() as i32;
        crate::items::jewel::set_hit_points(w, i, n);
        Ok(())
    }),
}];

static SERVER_ITEMS_CONTAINER: &[PropDef] = &[
    PropDef {
        name: "LiftOverride",
        kind: BOOL,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w.item(i).and_then(|o| Some(o)).map_or(Value::Null, |d| {
                Value::Bool(muo_server::item::container::lift_override(w, i))
            }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.bool();
            let Some(d) = w.item_mut(i).and_then(|o| Some(o.container_mut())) else {
                return Err(NOT_HERE.to_string());
            };
            if d.lift_override == n {
                return Ok(());
            }
            d.lift_override = n;
            Ok(())
        }),
    },
    PropDef {
        name: "MaxItems",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w.item(i).and_then(|o| Some(o)).map_or(Value::Null, |d| {
                Value::Int(i64::from(muo_server::item::container::max_items(w, i)))
            }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i32();
            let Some(d) = w.item_mut(i).and_then(|o| Some(o.container_mut())) else {
                return Err(NOT_HERE.to_string());
            };
            if d.max_items == n {
                return Ok(());
            }
            d.max_items = n;
            muo_server::item::ops::invalidate_properties(w, i);
            Ok(())
        }),
    },
    PropDef {
        name: "GumpID",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w.item(i).and_then(|o| Some(o)).map_or(Value::Null, |d| {
                Value::Int(i64::from(muo_server::item::container::gump_id(w, i)))
            }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i32();
            let Some(d) = w.item_mut(i).and_then(|o| Some(o.container_mut())) else {
                return Err(NOT_HERE.to_string());
            };
            if d.gump_id == n {
                return Ok(());
            }
            d.gump_id = n;
            Ok(())
        }),
    },
    PropDef {
        name: "DropSound",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w.item(i).and_then(|o| Some(o)).map_or(Value::Null, |d| {
                Value::Int(i64::from(muo_server::item::container::drop_sound(w, i)))
            }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i32();
            let Some(d) = w.item_mut(i).and_then(|o| Some(o.container_mut())) else {
                return Err(NOT_HERE.to_string());
            };
            if d.drop_sound == n {
                return Ok(());
            }
            d.drop_sound = n;
            Ok(())
        }),
    },
    PropDef {
        name: "MaxWeight",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w.item(i).and_then(|o| Some(o)).map_or(Value::Null, |d| {
                Value::Int(i64::from(muo_server::item::container::max_weight(w, i)))
            }),
            _ => Value::Null,
        },
        set: None,
    },
    PropDef {
        name: "DefaultGumpID",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w.item(i).and_then(|o| Some(o)).map_or(Value::Null, |d| {
                w.item(i).map_or(Value::Null, |x| {
                    Value::Int(i64::from((x.ty.default_gump_id)(w, i)))
                })
            }),
            _ => Value::Null,
        },
        set: None,
    },
    PropDef {
        name: "DefaultDropSound",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w.item(i).and_then(|o| Some(o)).map_or(Value::Null, |d| {
                w.item(i).map_or(Value::Null, |x| {
                    Value::Int(i64::from((x.ty.default_drop_sound)(w, i)))
                })
            }),
            _ => Value::Null,
        },
        set: None,
    },
];

static SERVER_ITEMS_VIRTUALCHECK: &[PropDef] = &[
    PropDef {
        name: "Plat",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: ADMIN,
        write: ADMIN,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| {
                    o.ext
                        .get::<muo_server::item::virtual_check::VirtualCheckData>()
                })
                .map_or(Value::Null, |d| {
                    let x = &d.plat;
                    Value::Int(i64::from(*x))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            muo_server::item::virtual_check::set_plat(w, i, n);
            Ok(())
        }),
    },
    PropDef {
        name: "Gold",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: ADMIN,
        write: ADMIN,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| {
                    o.ext
                        .get::<muo_server::item::virtual_check::VirtualCheckData>()
                })
                .map_or(Value::Null, |d| {
                    let x = &d.gold;
                    Value::Int(i64::from(*x))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            muo_server::item::virtual_check::set_gold(w, i, n);
            Ok(())
        }),
    },
];

static SERVER_ITEMS_LOCALIZEDADDONCOMPONENT: &[PropDef] = &[PropDef {
    name: "Number",
    kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
    read: GM,
    write: GM,
    get: |w, e| match e {
        EntityId::Item(i) => w
            .item(i)
            .and_then(|o| o.ext.get::<crate::items::addon::LocalizedData>())
            .map_or(Value::Null, |d| {
                let x = &d.number;
                Value::Int(i64::from(*x))
            }),
        _ => Value::Null,
    },
    set: Some(|w, e, v| {
        let EntityId::Item(i) = e else {
            return Err(NOT_HERE.to_string());
        };
        let n = v.i64() as i32;
        let Some(d) = w
            .item_mut(i)
            .and_then(|o| o.ext.get_mut::<crate::items::addon::LocalizedData>())
        else {
            return Err(NOT_HERE.to_string());
        };
        if d.number == n {
            return Ok(());
        }
        d.number = n;
        muo_server::item::ops::invalidate_properties(w, i);
        Ok(())
    }),
}];

static SERVER_ITEMS_ADDONCOMPONENT: &[PropDef] = &[
    PropDef {
        name: "Addon",
        kind: Kind::Plain(PropType::EntityOf("BaseAddon")),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::items::addon::ComponentData>())
                .map_or(Value::Null, |d| {
                    let x = &d.addon;
                    Value::Entity(x.map(EntityId::Item))
                }),
            _ => Value::Null,
        },
        set: None,
    },
    PropDef {
        name: "Offset",
        kind: Kind::Plain(PropType::IPoint3D),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::items::addon::ComponentData>())
                .map_or(Value::Null, |d| {
                    let x = &d.offset;
                    Value::Point3D(*x)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.point();
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::items::addon::ComponentData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.offset == n {
                return Ok(());
            }
            d.offset = n;
            Ok(())
        }),
    },
];

static SERVER_ITEMS_ADDONCONTAINERCOMPONENT: &[PropDef] = &[
    PropDef {
        name: "Addon",
        kind: Kind::Plain(PropType::EntityOf("BaseAddonContainer")),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| {
                    o.ext
                        .get::<crate::items::addon_container::ContainerComponentData>()
                })
                .map_or(Value::Null, |d| {
                    let x = &d.addon;
                    Value::Entity(x.map(EntityId::Item))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = match v.entity() {
                Some(EntityId::Item(i)) => Some(i),
                _ => None,
            };
            let Some(d) = w.item_mut(i).and_then(|o| {
                o.ext
                    .get_mut::<crate::items::addon_container::ContainerComponentData>()
            }) else {
                return Err(NOT_HERE.to_string());
            };
            if d.addon == n {
                return Ok(());
            }
            d.addon = n;
            Ok(())
        }),
    },
    PropDef {
        name: "Offset",
        kind: Kind::Plain(PropType::IPoint3D),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| {
                    o.ext
                        .get::<crate::items::addon_container::ContainerComponentData>()
                })
                .map_or(Value::Null, |d| {
                    let x = &d.offset;
                    Value::Point3D(*x)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.point();
            let Some(d) = w.item_mut(i).and_then(|o| {
                o.ext
                    .get_mut::<crate::items::addon_container::ContainerComponentData>()
            }) else {
                return Err(NOT_HERE.to_string());
            };
            if d.offset == n {
                return Ok(());
            }
            d.offset = n;
            Ok(())
        }),
    },
];

static SERVER_ITEMS_BASEADDONCONTAINER: &[PropDef] = &[PropDef {
    name: "Resource",
    kind: Kind::Plain(PropType::Enum(E_CRAFTRESOURCE)),
    read: GM,
    write: GM,
    get: |w, e| match e {
        EntityId::Item(i) => w
            .item(i)
            .and_then(|o| {
                o.ext
                    .get::<crate::items::addon_container::ContainerAddonData>()
            })
            .map_or(Value::Null, |d| {
                let x = &d.resource;
                Value::Enum(*x as i64)
            }),
        _ => Value::Null,
    },
    set: Some(|w, e, v| {
        let EntityId::Item(i) = e else {
            return Err(NOT_HERE.to_string());
        };
        let n = match crate::craft::CraftResource::from_i32(v.i64() as i32) {
            Some(e) => e,
            None => return Err("That is not a member of the enum.".to_string()),
        };
        crate::items::addon_container::set_resource(w, i, n);
        Ok(())
    }),
}];

static SERVER_ITEMS_BASEADDONCONTAINERDEED: &[PropDef] = &[PropDef {
    name: "Resource",
    kind: Kind::Plain(PropType::Enum(E_CRAFTRESOURCE)),
    read: GM,
    write: GM,
    get: |w, e| match e {
        EntityId::Item(i) => w
            .item(i)
            .and_then(|o| {
                o.ext
                    .get::<crate::items::addon_container::ContainerAddonData>()
            })
            .map_or(Value::Null, |d| {
                let x = &d.resource;
                Value::Enum(*x as i64)
            }),
        _ => Value::Null,
    },
    set: Some(|w, e, v| {
        let EntityId::Item(i) = e else {
            return Err(NOT_HERE.to_string());
        };
        let n = match crate::craft::CraftResource::from_i32(v.i64() as i32) {
            Some(e) => e,
            None => return Err("That is not a member of the enum.".to_string()),
        };
        crate::items::addon_container::set_resource(w, i, n);
        Ok(())
    }),
}];

static SERVER_ITEMS_BASETREASURECHEST: &[PropDef] = &[
    PropDef {
        name: "Level",
        kind: Kind::Plain(PropType::Enum(E_TREASURELEVEL)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::items::cont::ContExt>())
                .map_or(Value::Null, |d| {
                    let x = &d.level;
                    Value::Enum(*x as i64)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = match crate::items::cont::TreasureLevel::from_i32(v.i64() as i32) {
                Some(e) => e,
                None => return Err("That is not a member of the enum.".to_string()),
            };
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::items::cont::ContExt>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.level == n {
                return Ok(());
            }
            d.level = n;
            Ok(())
        }),
    },
    PropDef {
        name: "MinSpawnTime",
        kind: Kind::TimeSpan,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::items::cont::ContExt>())
                .map_or(Value::Null, |d| {
                    let x = &d.min_spawn_time;
                    Value::TimeSpan(*x)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = match v {
                Value::TimeSpan(t) => t,
                _ => TimeSpan::ZERO,
            };
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::items::cont::ContExt>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.min_spawn_time == n {
                return Ok(());
            }
            d.min_spawn_time = n;
            Ok(())
        }),
    },
    PropDef {
        name: "MaxSpawnTime",
        kind: Kind::TimeSpan,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::items::cont::ContExt>())
                .map_or(Value::Null, |d| {
                    let x = &d.max_spawn_time;
                    Value::TimeSpan(*x)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = match v {
                Value::TimeSpan(t) => t,
                _ => TimeSpan::ZERO,
            };
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::items::cont::ContExt>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.max_spawn_time == n {
                return Ok(());
            }
            d.max_spawn_time = n;
            Ok(())
        }),
    },
];

static SERVER_ITEMS_LOCKABLECONTAINER: &[PropDef] = &[
    PropDef {
        name: "IsShipwreckedItem",
        kind: BOOL,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::items::cont::ContExt>())
                .map_or(Value::Null, |d| {
                    let x = &d.is_shipwrecked_item;
                    Value::Bool(*x)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.bool();
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::items::cont::ContExt>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.is_shipwrecked_item == n {
                return Ok(());
            }
            d.is_shipwrecked_item = n;
            Ok(())
        }),
    },
    PropDef {
        name: "TrapOnLockpick",
        kind: BOOL,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::items::cont::ContExt>())
                .map_or(Value::Null, |d| {
                    let x = &d.trap_on_lockpick;
                    Value::Bool(*x)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.bool();
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::items::cont::ContExt>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.trap_on_lockpick == n {
                return Ok(());
            }
            d.trap_on_lockpick = n;
            Ok(())
        }),
    },
    PropDef {
        name: "RequiredSkill",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::items::cont::ContExt>())
                .map_or(Value::Null, |d| {
                    let x = &d.required_skill;
                    Value::Int(i64::from(*x))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::items::cont::ContExt>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.required_skill == n {
                return Ok(());
            }
            d.required_skill = n;
            Ok(())
        }),
    },
    PropDef {
        name: "MaxLockLevel",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::items::cont::ContExt>())
                .map_or(Value::Null, |d| {
                    let x = &d.max_lock_level;
                    Value::Int(i64::from(*x))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::items::cont::ContExt>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.max_lock_level == n {
                return Ok(());
            }
            d.max_lock_level = n;
            Ok(())
        }),
    },
    PropDef {
        name: "KeyValue",
        kind: Kind::Plain(PropType::Numeric(Numeric::UInt)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::items::cont::ContExt>())
                .map_or(Value::Null, |d| {
                    let x = &d.key_value;
                    Value::Int(i64::from(*x))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as u32;
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::items::cont::ContExt>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.key_value == n {
                return Ok(());
            }
            d.key_value = n;
            Ok(())
        }),
    },
    PropDef {
        name: "LockLevel",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::items::cont::ContExt>())
                .map_or(Value::Null, |d| {
                    let x = &d.lock_level;
                    Value::Int(i64::from(*x))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::items::cont::ContExt>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.lock_level == n {
                return Ok(());
            }
            d.lock_level = n;
            Ok(())
        }),
    },
];

static SERVER_ITEMS_TRAPPABLECONTAINER: &[PropDef] = &[
    PropDef {
        name: "TrapLevel",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::items::cont::ContExt>())
                .map_or(Value::Null, |d| {
                    let x = &d.trap_level;
                    Value::Int(i64::from(*x))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::items::cont::ContExt>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.trap_level == n {
                return Ok(());
            }
            d.trap_level = n;
            Ok(())
        }),
    },
    PropDef {
        name: "TrapPower",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::items::cont::ContExt>())
                .map_or(Value::Null, |d| {
                    let x = &d.trap_power;
                    Value::Int(i64::from(*x))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::items::cont::ContExt>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.trap_power == n {
                return Ok(());
            }
            d.trap_power = n;
            Ok(())
        }),
    },
    PropDef {
        name: "TrapType",
        kind: Kind::Plain(PropType::Enum(E_TRAPTYPE)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::items::cont::ContExt>())
                .map_or(Value::Null, |d| {
                    let x = &d.trap_type;
                    Value::Enum(*x as i64)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = match crate::items::cont::TrapType::from_i32(v.i64() as i32) {
                Some(e) => e,
                None => return Err("That is not a member of the enum.".to_string()),
            };
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::items::cont::ContExt>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.trap_type == n {
                return Ok(());
            }
            d.trap_type = n;
            Ok(())
        }),
    },
];

static SERVER_ITEMS_FOOD: &[PropDef] = &[
    PropDef {
        name: "Poisoner",
        kind: Kind::Plain(PropType::EntityOf("Mobile")),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::items::food::FoodData>())
                .map_or(Value::Null, |d| {
                    let x = &d.poisoner;
                    Value::Entity(x.map(EntityId::Mobile))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.mobile();
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::items::food::FoodData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.poisoner == n {
                return Ok(());
            }
            d.poisoner = n;
            Ok(())
        }),
    },
    PropDef {
        name: "Poison",
        kind: Kind::Poison,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::items::food::FoodData>())
                .map_or(Value::Null, |d| {
                    let x = &d.poison;
                    Value::Poison(*x)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = match v {
                Value::Poison(p) => p,
                _ => None,
            };
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::items::food::FoodData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.poison == n {
                return Ok(());
            }
            d.poison = n;
            Ok(())
        }),
    },
    PropDef {
        name: "FillFactor",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::items::food::FoodData>())
                .map_or(Value::Null, |d| {
                    let x = &d.fill_factor;
                    Value::Int(i64::from(*x))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            crate::items::food::set_fill_factor(w, i, n);
            Ok(())
        }),
    },
];

static SERVER_ITEMS_BANKCHECK: &[PropDef] = &[PropDef {
    name: "Worth",
    kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
    read: GM,
    write: GM,
    get: |w, e| match e {
        EntityId::Item(i) => w
            .item(i)
            .and_then(|o| o.ext.get::<crate::items::gold::BankCheckData>())
            .map_or(Value::Null, |d| {
                let x = &d.worth;
                Value::Int(i64::from(*x))
            }),
        _ => Value::Null,
    },
    set: Some(|w, e, v| {
        let EntityId::Item(i) = e else {
            return Err(NOT_HERE.to_string());
        };
        let n = v.i64() as i32;
        crate::items::gold::set_worth(w, i, n);
        Ok(())
    }),
}];

static SERVER_ITEMS_KEY: &[PropDef] = &[
    PropDef {
        name: "MaxRange",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::items::gold::KeyData>())
                .map_or(Value::Null, |d| {
                    let x = &d.max_range;
                    Value::Int(i64::from(*x))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::items::gold::KeyData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.max_range == n {
                return Ok(());
            }
            d.max_range = n;
            Ok(())
        }),
    },
    PropDef {
        name: "Link",
        kind: Kind::Plain(PropType::EntityOf("Item")),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::items::gold::KeyData>())
                .map_or(Value::Null, |d| {
                    let x = &d.link;
                    Value::Entity(x.map(EntityId::Item))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = match v.entity() {
                Some(EntityId::Item(i)) => Some(i),
                _ => None,
            };
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::items::gold::KeyData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.link == n {
                return Ok(());
            }
            d.link = n;
            Ok(())
        }),
    },
    PropDef {
        name: "Description",
        kind: STR,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::items::gold::KeyData>())
                .map_or(Value::Null, |d| {
                    let x = &d.description;
                    x.clone().map_or(Value::Null, Value::Str)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.string();
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::items::gold::KeyData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.description == n {
                return Ok(());
            }
            d.description = n;
            muo_server::item::ops::invalidate_properties(w, i);
            Ok(())
        }),
    },
    PropDef {
        name: "KeyValue",
        kind: Kind::Plain(PropType::Numeric(Numeric::UInt)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::items::gold::KeyData>())
                .map_or(Value::Null, |d| {
                    let x = &d.key_value;
                    Value::Int(i64::from(*x))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as u32;
            crate::items::gold::set_key_value(w, i, n);
            Ok(())
        }),
    },
];

static SERVER_ITEMS_BASEAXE: &[PropDef] = &[
    PropDef {
        name: "ShowUsesRemaining",
        kind: BOOL,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::items::weapon::WeaponData>())
                .map_or(Value::Null, |d| {
                    let x = &d.show_uses_remaining;
                    Value::Bool(*x)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.bool();
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::items::weapon::WeaponData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.show_uses_remaining == n {
                return Ok(());
            }
            d.show_uses_remaining = n;
            muo_server::item::ops::invalidate_properties(w, i);
            Ok(())
        }),
    },
    PropDef {
        name: "UsesRemaining",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::items::weapon::WeaponData>())
                .map_or(Value::Null, |d| {
                    let x = &d.uses_remaining;
                    Value::Int(i64::from(*x))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::items::weapon::WeaponData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.uses_remaining == n {
                return Ok(());
            }
            d.uses_remaining = n;
            muo_server::item::ops::invalidate_properties(w, i);
            Ok(())
        }),
    },
];

static SERVER_ITEMS_BASEPOLEARM: &[PropDef] = &[
    PropDef {
        name: "ShowUsesRemaining",
        kind: BOOL,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::items::weapon::WeaponData>())
                .map_or(Value::Null, |d| {
                    let x = &d.show_uses_remaining;
                    Value::Bool(*x)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.bool();
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::items::weapon::WeaponData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.show_uses_remaining == n {
                return Ok(());
            }
            d.show_uses_remaining = n;
            muo_server::item::ops::invalidate_properties(w, i);
            Ok(())
        }),
    },
    PropDef {
        name: "UsesRemaining",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::items::weapon::WeaponData>())
                .map_or(Value::Null, |d| {
                    let x = &d.uses_remaining;
                    Value::Int(i64::from(*x))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::items::weapon::WeaponData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.uses_remaining == n {
                return Ok(());
            }
            d.uses_remaining = n;
            muo_server::item::ops::invalidate_properties(w, i);
            Ok(())
        }),
    },
];

static SERVER_ITEMS_BASERANGED: &[PropDef] = &[
    PropDef {
        name: "Balanced",
        kind: BOOL,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::items::weapon::WeaponData>())
                .map_or(Value::Null, |d| {
                    let x = &d.balanced;
                    Value::Bool(*x)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.bool();
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::items::weapon::WeaponData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.balanced == n {
                return Ok(());
            }
            d.balanced = n;
            muo_server::item::ops::invalidate_properties(w, i);
            Ok(())
        }),
    },
    PropDef {
        name: "Velocity",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::items::weapon::WeaponData>())
                .map_or(Value::Null, |d| {
                    let x = &d.velocity;
                    Value::Int(i64::from(*x))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::items::weapon::WeaponData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.velocity == n {
                return Ok(());
            }
            d.velocity = n;
            muo_server::item::ops::invalidate_properties(w, i);
            Ok(())
        }),
    },
];

static SERVER_MISC_SHARDPOLLER: &[PropDef] = &[
    PropDef {
        name: "Duration",
        kind: Kind::TimeSpan,
        read: GM,
        write: ADMIN,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::login::shard_poller::ShardPollerData>())
                .map_or(Value::Null, |d| {
                    let x = &d.duration;
                    Value::TimeSpan(*x)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = match v {
                Value::TimeSpan(t) => t,
                _ => TimeSpan::ZERO,
            };
            let Some(d) = w.item_mut(i).and_then(|o| {
                o.ext
                    .get_mut::<crate::login::shard_poller::ShardPollerData>()
            }) else {
                return Err(NOT_HERE.to_string());
            };
            if d.duration == n {
                return Ok(());
            }
            d.duration = n;
            Ok(())
        }),
    },
    PropDef {
        name: "StartTime",
        kind: Kind::DateTime,
        read: GM,
        write: ADMIN,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::login::shard_poller::ShardPollerData>())
                .map_or(Value::Null, |d| {
                    let x = &d.start_time;
                    Value::DateTime(*x)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = match v {
                Value::DateTime(t) => t,
                _ => DateTime::MIN,
            };
            let Some(d) = w.item_mut(i).and_then(|o| {
                o.ext
                    .get_mut::<crate::login::shard_poller::ShardPollerData>()
            }) else {
                return Err(NOT_HERE.to_string());
            };
            if d.start_time == n {
                return Ok(());
            }
            d.start_time = n;
            Ok(())
        }),
    },
    PropDef {
        name: "Active",
        kind: BOOL,
        read: GM,
        write: ADMIN,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::login::shard_poller::ShardPollerData>())
                .map_or(Value::Null, |d| {
                    let x = &d.active;
                    Value::Bool(*x)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.bool();
            crate::login::shard_poller::set_active(w, i, n);
            Ok(())
        }),
    },
];

static SERVER_MOBILES_BASEMOUNT: &[PropDef] = &[
    PropDef {
        name: "NextMountAbility",
        kind: Kind::DateTime,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Mobile(m) => w
                .mobile(m)
                .and_then(|o| o.ext.get::<crate::creature::mount::MountState>())
                .map_or(Value::Null, |d| {
                    let x = &d.next_mount_ability;
                    Value::DateTime(*x)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Mobile(m) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = match v {
                Value::DateTime(t) => t,
                _ => DateTime::MIN,
            };
            let Some(d) = w
                .mobile_mut(m)
                .and_then(|o| o.ext.get_mut::<crate::creature::mount::MountState>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.next_mount_ability == n {
                return Ok(());
            }
            d.next_mount_ability = n;
            Ok(())
        }),
    },
    PropDef {
        name: "InternalItem",
        kind: Kind::Plain(PropType::EntityOf("Item")),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Mobile(m) => w
                .mobile(m)
                .and_then(|o| o.ext.get::<crate::creature::mount::MountState>())
                .map_or(Value::Null, |d| {
                    let x = &d.internal_item;
                    Value::Entity(x.map(EntityId::Item))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Mobile(m) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = match v.entity() {
                Some(EntityId::Item(i)) => Some(i),
                _ => None,
            };
            let Some(d) = w
                .mobile_mut(m)
                .and_then(|o| o.ext.get_mut::<crate::creature::mount::MountState>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.internal_item == n {
                return Ok(());
            }
            d.internal_item = n;
            Ok(())
        }),
    },
];

static SERVER_ITEMS_GAMBLINGSTONE: &[PropDef] = &[PropDef {
    name: "GamblePot",
    kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
    read: GM,
    write: GM,
    get: |w, e| match e {
        EntityId::Item(i) => w
            .item(i)
            .and_then(|o| {
                o.ext
                    .get::<crate::systems::special_systems::stones::GamblingStoneData>()
            })
            .map_or(Value::Null, |d| {
                let x = &d.gamble_pot;
                Value::Int(i64::from(*x))
            }),
        _ => Value::Null,
    },
    set: Some(|w, e, v| {
        let EntityId::Item(i) = e else {
            return Err(NOT_HERE.to_string());
        };
        let n = v.i64() as i32;
        crate::systems::special_systems::stones::set_gamble_pot(w, i, n);
        Ok(())
    }),
}];

static SERVER_ITEMS_REWARDCLOAK: &[PropDef] = &[
    PropDef {
        name: "Number",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::items::clothing::ClothingData>())
                .map_or(Value::Null, |d| {
                    let x = &d.number;
                    Value::Int(i64::from(*x))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            if let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::items::clothing::ClothingData>())
            {
                d.number = n;
            }
            muo_server::item::ops::invalidate_properties(w, i);
            Ok(())
        }),
    },
    PropDef {
        name: "IsRewardItem",
        kind: BOOL,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::items::clothing::ClothingData>())
                .map_or(Value::Null, |d| {
                    let x = &d.is_reward_item;
                    Value::Bool(*x)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.bool();
            if let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::items::clothing::ClothingData>())
            {
                d.is_reward_item = n;
            };
            Ok(())
        }),
    },
];

static SERVER_ITEMS_BASEHAT: &[PropDef] = &[PropDef {
    name: "IsShipwreckedItem",
    kind: BOOL,
    read: GM,
    write: GM,
    get: |w, e| match e {
        EntityId::Item(i) => w
            .item(i)
            .and_then(|o| o.ext.get::<crate::items::clothing::ClothingData>())
            .map_or(Value::Null, |d| {
                let x = &d.is_shipwrecked;
                Value::Bool(*x)
            }),
        _ => Value::Null,
    },
    set: Some(|w, e, v| {
        let EntityId::Item(i) = e else {
            return Err(NOT_HERE.to_string());
        };
        let n = v.bool();
        if let Some(d) = w
            .item_mut(i)
            .and_then(|o| o.ext.get_mut::<crate::items::clothing::ClothingData>())
        {
            d.is_shipwrecked = n;
        };
        Ok(())
    }),
}];

static SERVER_ITEMS_LOCKABLECONTAINER_26: &[PropDef] = &[PropDef {
    name: "Locked",
    kind: BOOL,
    read: GM,
    write: GM,
    get: |w, e| match e {
        EntityId::Item(i) => Value::Bool(crate::items::cont::locked(w, i)),
        _ => Value::Null,
    },
    set: Some(|w, e, v| {
        let EntityId::Item(i) = e else {
            return Err(NOT_HERE.to_string());
        };
        let n = v.bool();
        crate::items::cont::set_locked(w, i, n);
        Ok(())
    }),
}];

static SERVER_ITEMS_LOCKABLECONTAINER_27: &[PropDef] = &[PropDef {
    name: "Picker",
    kind: Kind::Plain(PropType::EntityOf("Mobile")),
    read: GM,
    write: GM,
    get: |w, e| match e {
        EntityId::Item(i) => w
            .item(i)
            .and_then(|o| o.ext.get::<crate::items::cont::ContExt>())
            .map_or(Value::Null, |d| {
                let x = &d.picker;
                Value::Entity(x.map(EntityId::Mobile))
            }),
        _ => Value::Null,
    },
    set: Some(|w, e, v| {
        let EntityId::Item(i) = e else {
            return Err(NOT_HERE.to_string());
        };
        let n = v.mobile();
        if let Some(d) = w
            .item_mut(i)
            .and_then(|o| o.ext.get_mut::<crate::items::cont::ContExt>())
        {
            d.picker = n;
        };
        Ok(())
    }),
}];

static SERVER_ITEMS_BASEADDON: &[PropDef] = &[PropDef {
    name: "Resource",
    kind: Kind::Plain(PropType::Enum(E_CRAFTRESOURCE)),
    read: GM,
    write: GM,
    get: |w, e| match e {
        EntityId::Item(i) => w
            .item(i)
            .and_then(|o| o.ext.get::<crate::items::addon::AddonData>())
            .map_or(Value::Null, |d| Value::Enum(d.resource as i64)),
        _ => Value::Null,
    },
    set: Some(|w, e, v| {
        let EntityId::Item(i) = e else {
            return Err(NOT_HERE.to_string());
        };
        let n = v.i64() as i32;
        match crate::craft::CraftResource::from_i32(n) {
            Some(r) => crate::items::addon::set_addon_resource(w, i, r),
            None => return Err("That is not a member of the enum.".to_string()),
        };
        Ok(())
    }),
}];

static SERVER_ITEMS_BASEADDONDEED: &[PropDef] = &[PropDef {
    name: "Resource",
    kind: Kind::Plain(PropType::Enum(E_CRAFTRESOURCE)),
    read: GM,
    write: GM,
    get: |w, e| match e {
        EntityId::Item(i) => w
            .item(i)
            .and_then(|o| o.ext.get::<crate::items::addon::AddonData>())
            .map_or(Value::Null, |d| Value::Enum(d.resource as i64)),
        _ => Value::Null,
    },
    set: Some(|w, e, v| {
        let EntityId::Item(i) = e else {
            return Err(NOT_HERE.to_string());
        };
        let n = v.i64() as i32;
        match crate::craft::CraftResource::from_i32(n) {
            Some(r) => crate::items::addon::set_deed_resource(w, i, r),
            None => return Err("That is not a member of the enum.".to_string()),
        };
        Ok(())
    }),
}];

static SERVER_MOBILES_BASEMOUNT_30: &[PropDef] = &[
    PropDef {
        name: "ItemID",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Mobile(m) => Value::Int(
                crate::creature::mount::internal_item(w, m)
                    .and_then(|x| w.item(x))
                    .map_or(0, |x| i64::from(x.item_id)),
            ),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Mobile(m) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            crate::creature::mount::set_item_id(w, m, n);
            Ok(())
        }),
    },
    PropDef {
        name: "Rider",
        kind: Kind::Plain(PropType::EntityOf("Mobile")),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Mobile(m) => {
                Value::Entity(crate::creature::mount::rider(w, m).map(EntityId::Mobile))
            }
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Mobile(m) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.mobile();
            crate::creature::mount::set_rider(w, m, n);
            Ok(())
        }),
    },
];

static SERVER_ITEMS_PROSPECTORSTOOL: &[PropDef] = &[PropDef {
    name: "UsesRemaining",
    kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
    read: GM,
    write: GM,
    get: |w, e| match e {
        EntityId::Item(i) => w
            .item(i)
            .and_then(|o| o.ext.get::<crate::items::tool::ToolData>())
            .map_or(Value::Null, |d| {
                let x = &d.uses_remaining;
                Value::Int(i64::from(*x))
            }),
        _ => Value::Null,
    },
    set: Some(|w, e, v| {
        let EntityId::Item(i) = e else {
            return Err(NOT_HERE.to_string());
        };
        let n = v.i64() as i32;
        if let Some(d) = w
            .item_mut(i)
            .and_then(|o| o.ext.get_mut::<crate::items::tool::ToolData>())
        {
            d.uses_remaining = n;
        }
        muo_server::item::ops::invalidate_properties(w, i);
        Ok(())
    }),
}];

static SERVER_MISC_SHARDPOLLER_32: &[PropDef] = &[
    PropDef {
        name: "Title",
        kind: STR,
        read: GM,
        write: ADMIN,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::login::shard_poller::ShardPollerData>())
                .map_or(Value::Null, |d| {
                    let x = &d.title;
                    x.clone().map_or(Value::Null, Value::Str)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.string();
            crate::login::shard_poller::set_title(w, i, n.as_deref());
            Ok(())
        }),
    },
    PropDef {
        name: "TimeRemaining",
        kind: Kind::TimeSpan,
        read: GM,
        write: ADMIN,
        get: |w, e| match e {
            EntityId::Item(i) => Value::TimeSpan(crate::login::shard_poller::time_remaining(w, i)),
            _ => Value::Null,
        },
        set: None,
    },
];

static SERVER_MOBILES_PLAYERMOBILE: &[PropDef] = &[PropDef {
    name: "LastMoved",
    kind: Kind::Plain(PropType::Numeric(Numeric::Long)),
    read: GM,
    write: GM,
    get: |w, e| match e {
        EntityId::Mobile(m) => Value::Int(w.mobile(m).map_or(0, |o| o.last_move_time)),
        _ => Value::Null,
    },
    set: None,
}];

static SERVER_ITEMS_BASEADDONCONTAINER_34: &[PropDef] = &[PropDef {
    name: "Hue",
    kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
    read: GM,
    write: GM,
    get: |w, e| match e {
        EntityId::Item(i) => Value::Int(w.item(i).map_or(0, |o| i64::from(o.hue))),
        _ => Value::Null,
    },
    set: Some(|w, e, v| {
        let EntityId::Item(i) = e else {
            return Err(NOT_HERE.to_string());
        };
        let n = v.i64() as i32;
        if let Some(ty) = w.item(i).map(|o| o.ty) {
            (ty.set_hue)(w, i, n);
        };
        Ok(())
    }),
}];

pub fn configure(w: &mut World) {
    props::add(w, "Server.Items.Corpse", SERVER_ITEMS_CORPSE);
    props::add(w, "Server.Items.BaseWeapon", SERVER_ITEMS_BASEWEAPON);
    props::add(w, "Server.Items.BaseArmor", SERVER_ITEMS_BASEARMOR);
    props::add(w, "Server.Items.BaseClothing", SERVER_ITEMS_BASECLOTHING);
    props::add(w, "Server.Items.BaseJewel", SERVER_ITEMS_BASEJEWEL);
    props::add(w, "Server.Items.Container", SERVER_ITEMS_CONTAINER);
    props::add(w, "Server.Items.VirtualCheck", SERVER_ITEMS_VIRTUALCHECK);
    props::add(
        w,
        "Server.Items.LocalizedAddonComponent",
        SERVER_ITEMS_LOCALIZEDADDONCOMPONENT,
    );
    props::add(
        w,
        "Server.Items.AddonComponent",
        SERVER_ITEMS_ADDONCOMPONENT,
    );
    props::add(
        w,
        "Server.Items.AddonContainerComponent",
        SERVER_ITEMS_ADDONCONTAINERCOMPONENT,
    );
    props::add(
        w,
        "Server.Items.BaseAddonContainer",
        SERVER_ITEMS_BASEADDONCONTAINER,
    );
    props::add(
        w,
        "Server.Items.BaseAddonContainerDeed",
        SERVER_ITEMS_BASEADDONCONTAINERDEED,
    );
    props::add(
        w,
        "Server.Items.BaseTreasureChest",
        SERVER_ITEMS_BASETREASURECHEST,
    );
    props::add(
        w,
        "Server.Items.LockableContainer",
        SERVER_ITEMS_LOCKABLECONTAINER,
    );
    props::add(
        w,
        "Server.Items.TrappableContainer",
        SERVER_ITEMS_TRAPPABLECONTAINER,
    );
    props::add(w, "Server.Items.Food", SERVER_ITEMS_FOOD);
    props::add(w, "Server.Items.BankCheck", SERVER_ITEMS_BANKCHECK);
    props::add(w, "Server.Items.Key", SERVER_ITEMS_KEY);
    props::add(w, "Server.Items.BaseAxe", SERVER_ITEMS_BASEAXE);
    props::add(w, "Server.Items.BasePoleArm", SERVER_ITEMS_BASEPOLEARM);
    props::add(w, "Server.Items.BaseRanged", SERVER_ITEMS_BASERANGED);
    props::add(w, "Server.Misc.ShardPoller", SERVER_MISC_SHARDPOLLER);
    props::add(w, "Server.Mobiles.BaseMount", SERVER_MOBILES_BASEMOUNT);
    props::add(w, "Server.Items.GamblingStone", SERVER_ITEMS_GAMBLINGSTONE);
    props::add(w, "Server.Items.RewardCloak", SERVER_ITEMS_REWARDCLOAK);
    props::add(w, "Server.Items.RewardDress", SERVER_ITEMS_REWARDCLOAK);
    props::add(w, "Server.Items.RewardRobe", SERVER_ITEMS_REWARDCLOAK);
    props::add(w, "Server.Items.BaseHat", SERVER_ITEMS_BASEHAT);
    props::add(
        w,
        "Server.Items.LockableContainer",
        SERVER_ITEMS_LOCKABLECONTAINER_26,
    );
    props::add(
        w,
        "Server.Items.BaseTreasureChest",
        SERVER_ITEMS_LOCKABLECONTAINER_26,
    );
    props::add(
        w,
        "Server.Items.MarkContainer",
        SERVER_ITEMS_LOCKABLECONTAINER_26,
    );
    props::add(
        w,
        "Server.Items.LockableContainer",
        SERVER_ITEMS_LOCKABLECONTAINER_27,
    );
    props::add(w, "Server.Items.BaseAddon", SERVER_ITEMS_BASEADDON);
    props::add(w, "Server.Items.BaseAddonDeed", SERVER_ITEMS_BASEADDONDEED);
    props::add(w, "Server.Mobiles.BaseMount", SERVER_MOBILES_BASEMOUNT_30);
    props::add(
        w,
        "Server.Items.ProspectorsTool",
        SERVER_ITEMS_PROSPECTORSTOOL,
    );
    props::add(w, "Server.Misc.ShardPoller", SERVER_MISC_SHARDPOLLER_32);
    props::add(
        w,
        "Server.Mobiles.PlayerMobile",
        SERVER_MOBILES_PLAYERMOBILE,
    );
    props::add(
        w,
        "Server.Items.BaseAddonContainer",
        SERVER_ITEMS_BASEADDONCONTAINER_34,
    );
}
