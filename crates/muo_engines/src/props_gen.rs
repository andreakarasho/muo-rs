//! Generated property tables: do not edit by hand.
//! `[CommandProperty]` tables of classes whose members are fields of a data struct in `ext`.
#![allow(
    unused_imports,
    unused_variables,
    dead_code,
    redundant_semicolons,
    clippy::all
)]
use muo_content::aos::AosElementAttribute;
use muo_content::props::{self, Kind, PropDef, Value};
use muo_content::utilities::types::{Numeric, PropType};
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

const E_BULKMATERIALTYPE: &[(&str, i64)] = &[
    ("None", 0),
    ("DullCopper", 1),
    ("ShadowIron", 2),
    ("Copper", 3),
    ("Bronze", 4),
    ("Gold", 5),
    ("Agapite", 6),
    ("Verite", 7),
    ("Valorite", 8),
    ("Spined", 9),
    ("Horned", 10),
    ("Barbed", 11),
];
const E_CANNONDIRECTION: &[(&str, i64)] = &[("North", 0), ("East", 1), ("South", 2), ("West", 3)];
const E_IMAGETYPE: &[(&str, i64)] = &[
    ("Betrayer", 0),
    ("Bogling", 1),
    ("BogThing", 2),
    ("Gazer", 3),
    ("Beetle", 4),
    ("GiantBlackWidow", 5),
    ("Scorpion", 6),
    ("JukaMage", 7),
    ("JukaWarrior", 8),
    ("Lich", 9),
    ("MeerMage", 10),
    ("MeerWarrior", 11),
    ("Mongbat", 12),
    ("Mummy", 13),
    ("Pixie", 14),
    ("PlagueBeast", 15),
    ("SandVortex", 16),
    ("StoneGargoyle", 17),
    ("SwampDragon", 18),
    ("Wisp", 19),
    ("Juggernaut", 20),
];
const E_REPAIRSKILLTYPE: &[(&str, i64)] = &[
    ("Smithing", 0),
    ("Tailoring", 1),
    ("Tinkering", 2),
    ("Carpentry", 3),
    ("Fletching", 4),
];
const E_SECURELEVEL: &[(&str, i64)] = &[
    ("Owner", 0),
    ("CoOwners", 1),
    ("Friends", 2),
    ("Anyone", 3),
    ("Guild", 4),
];
const E_TROPHYRANK: &[(&str, i64)] = &[("Bronze", 0), ("Silver", 1), ("Gold", 2)];
static SERVER_FACTIONS_BASEMONOLITH: &[PropDef] = &[
    PropDef {
        name: "Sigil",
        kind: Kind::Plain(PropType::EntityOf("Sigil")),
        read: COUNSELOR,
        write: ADMIN,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::factions::items::MonolithData>())
                .map_or(Value::Null, |d| {
                    let x = &d.sigil;
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
            crate::factions::items::monolith_set_sigil(w, i, n);
            Ok(())
        }),
    },
    PropDef {
        name: "Town",
        kind: TOWN,
        read: COUNSELOR,
        write: ADMIN,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::factions::items::MonolithData>())
                .map_or(Value::Null, |d| {
                    let x = &d.town;
                    x.map_or(Value::Null, |f| Value::Enum(f as i64))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = match v {
                Value::Enum(n) | Value::Int(n) => Some(n as usize),
                _ => None,
            };
            crate::factions::items::monolith_set_town(w, i, n);
            Ok(())
        }),
    },
    PropDef {
        name: "Faction",
        kind: FACTION,
        read: COUNSELOR,
        write: ADMIN,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::factions::items::MonolithData>())
                .map_or(Value::Null, |d| {
                    let x = &d.faction;
                    x.map_or(Value::Null, |f| Value::Enum(f as i64))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = match v {
                Value::Enum(n) | Value::Int(n) => Some(n as usize),
                _ => None,
            };
            crate::factions::items::monolith_set_faction(w, i, n);
            Ok(())
        }),
    },
];

static SERVER_FACTIONS_FACTIONSTONE: &[PropDef] = &[PropDef {
    name: "Faction",
    kind: FACTION,
    read: COUNSELOR,
    write: ADMIN,
    get: |w, e| match e {
        EntityId::Item(i) => w
            .item(i)
            .and_then(|o| o.ext.get::<crate::factions::items::StoneData>())
            .map_or(Value::Null, |d| {
                let x = &d.faction;
                x.map_or(Value::Null, |f| Value::Enum(f as i64))
            }),
        _ => Value::Null,
    },
    set: Some(|w, e, v| {
        let EntityId::Item(i) = e else {
            return Err(NOT_HERE.to_string());
        };
        let n = match v {
            Value::Enum(n) | Value::Int(n) => Some(n as usize),
            _ => None,
        };
        crate::factions::items::faction_stone_set_faction(w, i, n);
        Ok(())
    }),
}];

static SERVER_FACTIONS_JOINSTONE: &[PropDef] = &[PropDef {
    name: "Faction",
    kind: FACTION,
    read: COUNSELOR,
    write: ADMIN,
    get: |w, e| match e {
        EntityId::Item(i) => w
            .item(i)
            .and_then(|o| o.ext.get::<crate::factions::items::StoneData>())
            .map_or(Value::Null, |d| {
                let x = &d.faction;
                x.map_or(Value::Null, |f| Value::Enum(f as i64))
            }),
        _ => Value::Null,
    },
    set: Some(|w, e, v| {
        let EntityId::Item(i) = e else {
            return Err(NOT_HERE.to_string());
        };
        let n = match v {
            Value::Enum(n) | Value::Int(n) => Some(n as usize),
            _ => None,
        };
        crate::factions::items::join_stone_set_faction(w, i, n);
        Ok(())
    }),
}];

static SERVER_FACTIONS_TOWNSTONE: &[PropDef] = &[PropDef {
    name: "Town",
    kind: TOWN,
    read: COUNSELOR,
    write: ADMIN,
    get: |w, e| match e {
        EntityId::Item(i) => w
            .item(i)
            .and_then(|o| o.ext.get::<crate::factions::items::TownStoneData>())
            .map_or(Value::Null, |d| {
                let x = &d.town;
                x.map_or(Value::Null, |f| Value::Enum(f as i64))
            }),
        _ => Value::Null,
    },
    set: Some(|w, e, v| {
        let EntityId::Item(i) = e else {
            return Err(NOT_HERE.to_string());
        };
        let n = match v {
            Value::Enum(n) | Value::Int(n) => Some(n as usize),
            _ => None,
        };
        crate::factions::items::town_stone_set_town(w, i, n);
        Ok(())
    }),
}];

static SERVER_FACTIONS_SIGIL: &[PropDef] = &[
    PropDef {
        name: "LastStolen",
        kind: Kind::DateTime,
        read: COUNSELOR,
        write: ADMIN,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::factions::items::SigilData>())
                .map_or(Value::Null, |d| {
                    let x = &d.last_stolen;
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
                .and_then(|o| o.ext.get_mut::<crate::factions::items::SigilData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.last_stolen == n {
                return Ok(());
            }
            d.last_stolen = n;
            Ok(())
        }),
    },
    PropDef {
        name: "GraceStart",
        kind: Kind::DateTime,
        read: COUNSELOR,
        write: ADMIN,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::factions::items::SigilData>())
                .map_or(Value::Null, |d| {
                    let x = &d.grace_start;
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
                .and_then(|o| o.ext.get_mut::<crate::factions::items::SigilData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.grace_start == n {
                return Ok(());
            }
            d.grace_start = n;
            Ok(())
        }),
    },
    PropDef {
        name: "CorruptionStart",
        kind: Kind::DateTime,
        read: COUNSELOR,
        write: ADMIN,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::factions::items::SigilData>())
                .map_or(Value::Null, |d| {
                    let x = &d.corruption_start;
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
                .and_then(|o| o.ext.get_mut::<crate::factions::items::SigilData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.corruption_start == n {
                return Ok(());
            }
            d.corruption_start = n;
            Ok(())
        }),
    },
    PropDef {
        name: "PurificationStart",
        kind: Kind::DateTime,
        read: COUNSELOR,
        write: ADMIN,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::factions::items::SigilData>())
                .map_or(Value::Null, |d| {
                    let x = &d.purification_start;
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
                .and_then(|o| o.ext.get_mut::<crate::factions::items::SigilData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.purification_start == n {
                return Ok(());
            }
            d.purification_start = n;
            Ok(())
        }),
    },
    PropDef {
        name: "LastMonolith",
        kind: Kind::Plain(PropType::EntityOf("BaseMonolith")),
        read: COUNSELOR,
        write: ADMIN,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::factions::items::SigilData>())
                .map_or(Value::Null, |d| {
                    let x = &d.last_monolith;
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
                .and_then(|o| o.ext.get_mut::<crate::factions::items::SigilData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.last_monolith == n {
                return Ok(());
            }
            d.last_monolith = n;
            Ok(())
        }),
    },
    PropDef {
        name: "Town",
        kind: TOWN,
        read: COUNSELOR,
        write: ADMIN,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::factions::items::SigilData>())
                .map_or(Value::Null, |d| {
                    let x = &d.town;
                    x.map_or(Value::Null, |f| Value::Enum(f as i64))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = match v {
                Value::Enum(n) | Value::Int(n) => Some(n as usize),
                _ => None,
            };
            crate::factions::items::sigil_set_town(w, i, n);
            Ok(())
        }),
    },
    PropDef {
        name: "Corrupted",
        kind: FACTION,
        read: COUNSELOR,
        write: ADMIN,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::factions::items::SigilData>())
                .map_or(Value::Null, |d| {
                    let x = &d.corrupted;
                    x.map_or(Value::Null, |f| Value::Enum(f as i64))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = match v {
                Value::Enum(n) | Value::Int(n) => Some(n as usize),
                _ => None,
            };
            crate::factions::items::sigil_set_corrupted(w, i, n);
            Ok(())
        }),
    },
    PropDef {
        name: "Corrupting",
        kind: FACTION,
        read: COUNSELOR,
        write: ADMIN,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::factions::items::SigilData>())
                .map_or(Value::Null, |d| {
                    let x = &d.corrupting;
                    x.map_or(Value::Null, |f| Value::Enum(f as i64))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = match v {
                Value::Enum(n) | Value::Int(n) => Some(n as usize),
                _ => None,
            };
            crate::factions::items::sigil_set_corrupting(w, i, n);
            Ok(())
        }),
    },
    PropDef {
        name: "IsBeingCorrupted",
        kind: BOOL,
        read: COUNSELOR,
        write: COUNSELOR,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::factions::items::SigilData>())
                .map_or(Value::Null, |d| {
                    Value::Bool(crate::factions::items::is_being_corrupted(w, i))
                }),
            _ => Value::Null,
        },
        set: None,
    },
    PropDef {
        name: "IsCorrupted",
        kind: BOOL,
        read: COUNSELOR,
        write: COUNSELOR,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::factions::items::SigilData>())
                .map_or(Value::Null, |d| {
                    Value::Bool(crate::factions::items::is_corrupted(w, i))
                }),
            _ => Value::Null,
        },
        set: None,
    },
    PropDef {
        name: "IsPurifying",
        kind: BOOL,
        read: COUNSELOR,
        write: COUNSELOR,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::factions::items::SigilData>())
                .map_or(Value::Null, |d| {
                    Value::Bool(crate::factions::items::is_purifying(w, i))
                }),
            _ => Value::Null,
        },
        set: None,
    },
    PropDef {
        name: "IsCorrupting",
        kind: BOOL,
        read: COUNSELOR,
        write: COUNSELOR,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::factions::items::SigilData>())
                .map_or(Value::Null, |d| {
                    Value::Bool(crate::factions::items::is_corrupting(w, i))
                }),
            _ => Value::Null,
        },
        set: None,
    },
    PropDef {
        name: "TimeUntilCorruption",
        kind: Kind::TimeSpan,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::factions::items::SigilData>())
                .map_or(Value::Null, |d| {
                    if !crate::factions::items::is_being_corrupted(w, i) {
                        Value::TimeSpan(TimeSpan::ZERO)
                    } else {
                        let t = d.corruption_start
                            + crate::factions::items::sigil_corruption_period(w)
                            - w.now;
                        Value::TimeSpan(if t < TimeSpan::ZERO {
                            TimeSpan::ZERO
                        } else {
                            t
                        })
                    }
                }),
            _ => Value::Null,
        },
        set: None,
    },
];

static SERVER_FACTIONS_FACTIONEXPLOSIONTRAP: &[PropDef] = &[
    PropDef {
        name: "Faction",
        kind: FACTION,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::factions::traps::TrapData>())
                .map_or(Value::Null, |d| {
                    let x = &d.faction;
                    x.map_or(Value::Null, |f| Value::Enum(f as i64))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = match v {
                Value::Enum(n) | Value::Int(n) => Some(n as usize),
                _ => None,
            };
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::factions::traps::TrapData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.faction == n {
                return Ok(());
            }
            d.faction = n;
            Ok(())
        }),
    },
    PropDef {
        name: "Placer",
        kind: Kind::Plain(PropType::EntityOf("Mobile")),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::factions::traps::TrapData>())
                .map_or(Value::Null, |d| {
                    let x = &d.placer;
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
                .and_then(|o| o.ext.get_mut::<crate::factions::traps::TrapData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.placer == n {
                return Ok(());
            }
            d.placer = n;
            Ok(())
        }),
    },
    PropDef {
        name: "TimeOfPlacement",
        kind: Kind::DateTime,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::factions::traps::TrapData>())
                .map_or(Value::Null, |d| {
                    let x = &d.time_of_placement;
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
                .and_then(|o| o.ext.get_mut::<crate::factions::traps::TrapData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.time_of_placement == n {
                return Ok(());
            }
            d.time_of_placement = n;
            Ok(())
        }),
    },
];

static SERVER_FACTIONS_FACTIONEXPLOSIONTRAPDEED: &[PropDef] = &[PropDef {
    name: "Faction",
    kind: FACTION,
    read: GM,
    write: GM,
    get: |w, e| match e {
        EntityId::Item(i) => w
            .item(i)
            .and_then(|o| o.ext.get::<crate::factions::traps::DeedData>())
            .map_or(Value::Null, |d| {
                let x = &d.faction;
                x.map_or(Value::Null, |f| Value::Enum(f as i64))
            }),
        _ => Value::Null,
    },
    set: Some(|w, e, v| {
        let EntityId::Item(i) = e else {
            return Err(NOT_HERE.to_string());
        };
        let n = match v {
            Value::Enum(n) | Value::Int(n) => Some(n as usize),
            _ => None,
        };
        crate::factions::traps::deed_set_faction(w, i, n);
        Ok(())
    }),
}];

static SERVER_FACTIONS_FACTIONWARHORSE: &[PropDef] = &[PropDef {
    name: "Faction",
    kind: FACTION,
    read: GM,
    write: ADMIN,
    get: |w, e| match e {
        EntityId::Mobile(m) => w
            .mobile(m)
            .and_then(|o| o.ext.get::<crate::factions::mobiles::WarHorseData>())
            .map_or(Value::Null, |d| {
                let x = &d.faction;
                x.map_or(Value::Null, |f| Value::Enum(f as i64))
            }),
        _ => Value::Null,
    },
    set: Some(|w, e, v| {
        let EntityId::Mobile(m) = e else {
            return Err(NOT_HERE.to_string());
        };
        let n = match v {
            Value::Enum(n) | Value::Int(n) => Some(n as usize),
            _ => None,
        };
        crate::factions::mobiles::war_horse_set_faction(w, m, n);
        Ok(())
    }),
}];

static SERVER_FACTIONS_FACTIONBERSERKER: &[PropDef] = &[
    PropDef {
        name: "Faction",
        kind: FACTION,
        read: GM,
        write: ADMIN,
        get: |w, e| match e {
            EntityId::Mobile(m) => w
                .mobile(m)
                .and_then(|o| o.ext.get::<crate::factions::mobiles::GuardData>())
                .map_or(Value::Null, |d| {
                    let x = &d.faction;
                    x.map_or(Value::Null, |f| Value::Enum(f as i64))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Mobile(m) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = match v {
                Value::Enum(n) | Value::Int(n) => Some(n as usize),
                _ => None,
            };
            crate::factions::mobiles::set_guard_faction(w, m, n);
            Ok(())
        }),
    },
    PropDef {
        name: "Town",
        kind: TOWN,
        read: GM,
        write: ADMIN,
        get: |w, e| match e {
            EntityId::Mobile(m) => w
                .mobile(m)
                .and_then(|o| o.ext.get::<crate::factions::mobiles::GuardData>())
                .map_or(Value::Null, |d| {
                    let x = &d.town;
                    x.map_or(Value::Null, |f| Value::Enum(f as i64))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Mobile(m) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = match v {
                Value::Enum(n) | Value::Int(n) => Some(n as usize),
                _ => None,
            };
            crate::factions::mobiles::set_guard_town(w, m, n);
            Ok(())
        }),
    },
];

static SERVER_FACTIONS_FACTIONBOARDVENDOR: &[PropDef] = &[
    PropDef {
        name: "Town",
        kind: TOWN,
        read: COUNSELOR,
        write: ADMIN,
        get: |w, e| match e {
            EntityId::Mobile(m) => w
                .mobile(m)
                .and_then(|o| o.ext.get::<crate::factions::mobiles::FactionVendorData>())
                .map_or(Value::Null, |d| {
                    let x = &d.town;
                    x.map_or(Value::Null, |f| Value::Enum(f as i64))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Mobile(m) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = match v {
                Value::Enum(n) | Value::Int(n) => Some(n as usize),
                _ => None,
            };
            {
                let f = crate::factions::mobiles::vendor_faction(w, m);
                crate::factions::mobiles::set_vendor_town_faction(w, m, n, f)
            };
            Ok(())
        }),
    },
    PropDef {
        name: "Faction",
        kind: FACTION,
        read: COUNSELOR,
        write: ADMIN,
        get: |w, e| match e {
            EntityId::Mobile(m) => w
                .mobile(m)
                .and_then(|o| o.ext.get::<crate::factions::mobiles::FactionVendorData>())
                .map_or(Value::Null, |d| {
                    let x = &d.faction;
                    x.map_or(Value::Null, |f| Value::Enum(f as i64))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Mobile(m) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = match v {
                Value::Enum(n) | Value::Int(n) => Some(n as usize),
                _ => None,
            };
            {
                let t = crate::factions::mobiles::vendor_town(w, m);
                crate::factions::mobiles::set_vendor_town_faction(w, m, t, n)
            };
            Ok(())
        }),
    },
];

static SERVER_ENGINES_BULKORDERS_BASEBOD: &[PropDef] = &[
    PropDef {
        name: "AmountMax",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::bulk_orders::BodData>())
                .map_or(Value::Null, |d| {
                    let x = &d.amount_max;
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
                .and_then(|o| o.ext.get_mut::<crate::bulk_orders::BodData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.amount_max == n {
                return Ok(());
            }
            d.amount_max = n;
            muo_server::item::ops::invalidate_properties(w, i);
            Ok(())
        }),
    },
    PropDef {
        name: "RequireExceptional",
        kind: BOOL,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::bulk_orders::BodData>())
                .map_or(Value::Null, |d| {
                    let x = &d.require_exceptional;
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
                .and_then(|o| o.ext.get_mut::<crate::bulk_orders::BodData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.require_exceptional == n {
                return Ok(());
            }
            d.require_exceptional = n;
            muo_server::item::ops::invalidate_properties(w, i);
            Ok(())
        }),
    },
    PropDef {
        name: "Material",
        kind: Kind::Plain(PropType::Enum(E_BULKMATERIALTYPE)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::bulk_orders::BodData>())
                .map_or(Value::Null, |d| {
                    let x = &d.material;
                    Value::Enum(i64::from(*x))
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
                .and_then(|o| o.ext.get_mut::<crate::bulk_orders::BodData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.material == n {
                return Ok(());
            }
            d.material = n;
            muo_server::item::ops::invalidate_properties(w, i);
            Ok(())
        }),
    },
];

static SERVER_ENGINES_BULKORDERS_SMALLBOD: &[PropDef] = &[
    PropDef {
        name: "AmountCur",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::bulk_orders::BodData>())
                .map_or(Value::Null, |d| {
                    let x = &d.amount_cur;
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
                .and_then(|o| o.ext.get_mut::<crate::bulk_orders::BodData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.amount_cur == n {
                return Ok(());
            }
            d.amount_cur = n;
            muo_server::item::ops::invalidate_properties(w, i);
            Ok(())
        }),
    },
    PropDef {
        name: "Number",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::bulk_orders::BodData>())
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
                .and_then(|o| o.ext.get_mut::<crate::bulk_orders::BodData>())
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
    },
    PropDef {
        name: "Graphic",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::bulk_orders::BodData>())
                .map_or(Value::Null, |d| {
                    let x = &d.graphic;
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
                .and_then(|o| o.ext.get_mut::<crate::bulk_orders::BodData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.graphic == n {
                return Ok(());
            }
            d.graphic = n;
            Ok(())
        }),
    },
];

static SERVER_ENGINES_BULKORDERS_BULKORDERBOOK: &[PropDef] = &[
    PropDef {
        name: "Level",
        kind: Kind::Plain(PropType::Enum(E_SECURELEVEL)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::bulk_orders::book::BookData>())
                .map_or(Value::Null, |d| {
                    let x = &d.level;
                    Value::Enum(i64::from(*x))
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
                .and_then(|o| o.ext.get_mut::<crate::bulk_orders::book::BookData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.level == n {
                return Ok(());
            }
            d.level = n;
            muo_server::item::ops::invalidate_properties(w, i);
            Ok(())
        }),
    },
    PropDef {
        name: "BookName",
        kind: STR,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::bulk_orders::book::BookData>())
                .map_or(Value::Null, |d| {
                    let x = &d.book_name;
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
                .and_then(|o| o.ext.get_mut::<crate::bulk_orders::book::BookData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.book_name == n {
                return Ok(());
            }
            d.book_name = n;
            muo_server::item::ops::invalidate_properties(w, i);
            Ok(())
        }),
    },
    PropDef {
        name: "Entries",
        kind: Kind::Text,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::bulk_orders::book::BookData>())
                .map_or(Value::Null, |d| {
                    let x = &d.entries;
                    Value::Text(
                        "System.Collections.Generic.List`1[Server.Engines.BulkOrders.IBOBEntry]"
                            .to_string(),
                    )
                }),
            _ => Value::Null,
        },
        set: None,
    },
];

static SERVER_ENGINES_CONPVP_TOURNAMENTBRACKETITEM: &[PropDef] = &[PropDef {
    name: "Tournament",
    kind: Kind::Plain(PropType::EntityOf("TournamentController")),
    read: GM,
    write: GM,
    get: |w, e| match e {
        EntityId::Item(i) => w
            .item(i)
            .and_then(|o| o.ext.get::<crate::conpvp::tournament::LinkData>())
            .map_or(Value::Null, |d| {
                let x = &d.tournament;
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
            .and_then(|o| o.ext.get_mut::<crate::conpvp::tournament::LinkData>())
        else {
            return Err(NOT_HERE.to_string());
        };
        if d.tournament == n {
            return Ok(());
        }
        d.tournament = n;
        Ok(())
    }),
}];

static SERVER_ENGINES_CONPVP_TOURNAMENTREGISTRAR: &[PropDef] = &[PropDef {
    name: "Tournament",
    kind: Kind::Plain(PropType::EntityOf("TournamentController")),
    read: GM,
    write: GM,
    get: |w, e| match e {
        EntityId::Item(i) => w
            .item(i)
            .and_then(|o| o.ext.get::<crate::conpvp::tournament::LinkData>())
            .map_or(Value::Null, |d| {
                let x = &d.tournament;
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
            .and_then(|o| o.ext.get_mut::<crate::conpvp::tournament::LinkData>())
        else {
            return Err(NOT_HERE.to_string());
        };
        if d.tournament == n {
            return Ok(());
        }
        d.tournament = n;
        Ok(())
    }),
}];

static SERVER_ENGINES_CONPVP_TOURNAMENTSIGNUPITEM: &[PropDef] = &[
    PropDef {
        name: "Tournament",
        kind: Kind::Plain(PropType::EntityOf("TournamentController")),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::conpvp::tournament::LinkData>())
                .map_or(Value::Null, |d| {
                    let x = &d.tournament;
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
                .and_then(|o| o.ext.get_mut::<crate::conpvp::tournament::LinkData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.tournament == n {
                return Ok(());
            }
            d.tournament = n;
            Ok(())
        }),
    },
    PropDef {
        name: "Registrar",
        kind: Kind::Plain(PropType::EntityOf("Mobile")),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::conpvp::tournament::LinkData>())
                .map_or(Value::Null, |d| {
                    let x = &d.registrar;
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
                .and_then(|o| o.ext.get_mut::<crate::conpvp::tournament::LinkData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.registrar == n {
                return Ok(());
            }
            d.registrar = n;
            Ok(())
        }),
    },
];

static SERVER_ITEMS_TROPHY: &[PropDef] = &[
    PropDef {
        name: "Title",
        kind: STR,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::conpvp::trophy::TrophyData>())
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
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::conpvp::trophy::TrophyData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.title == n {
                return Ok(());
            }
            d.title = n;
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
                .and_then(|o| o.ext.get::<crate::conpvp::trophy::TrophyData>())
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
                .and_then(|o| o.ext.get_mut::<crate::conpvp::trophy::TrophyData>())
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
        name: "Date",
        kind: Kind::DateTime,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::conpvp::trophy::TrophyData>())
                .map_or(Value::Null, |d| {
                    let x = &d.date;
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
                .and_then(|o| o.ext.get_mut::<crate::conpvp::trophy::TrophyData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.date == n {
                return Ok(());
            }
            d.date = n;
            Ok(())
        }),
    },
];

static SERVER_FACTIONS_FACTIONTRAPREMOVALKIT: &[PropDef] = &[PropDef {
    name: "Charges",
    kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
    read: GM,
    write: GM,
    get: |w, e| match e {
        EntityId::Item(i) => w
            .item(i)
            .and_then(|o| o.ext.get::<crate::factions::traps::KitData>())
            .map_or(Value::Null, |d| {
                let x = &d.charges;
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
            .and_then(|o| o.ext.get_mut::<crate::factions::traps::KitData>())
        else {
            return Err(NOT_HERE.to_string());
        };
        if d.charges == n {
            return Ok(());
        }
        d.charges = n;
        Ok(())
    }),
}];

static SERVER_ENGINES_QUESTS_COLLECTOR_OBSIDIAN: &[PropDef] = &[PropDef {
    name: "StatueName",
    kind: STR,
    read: GM,
    write: GM,
    get: |w, e| match e {
        EntityId::Item(i) => w
            .item(i)
            .and_then(|o| o.ext.get::<crate::quests::collector_mobs::ObsidianData>())
            .map_or(Value::Null, |d| {
                let x = &d.statue_name;
                Value::Str(x.clone())
            }),
        _ => Value::Null,
    },
    set: Some(|w, e, v| {
        let EntityId::Item(i) = e else {
            return Err(NOT_HERE.to_string());
        };
        let n = v.string().unwrap_or_default();
        let Some(d) = w.item_mut(i).and_then(|o| {
            o.ext
                .get_mut::<crate::quests::collector_mobs::ObsidianData>()
        }) else {
            return Err(NOT_HERE.to_string());
        };
        if d.statue_name == n {
            return Ok(());
        }
        d.statue_name = n;
        muo_server::item::ops::invalidate_properties(w, i);
        Ok(())
    }),
}];

static SERVER_ENGINES_QUESTS_COLLECTOR_PAINTEDIMAGE: &[PropDef] = &[PropDef {
    name: "Image",
    kind: Kind::Plain(PropType::Enum(E_IMAGETYPE)),
    read: GM,
    write: GM,
    get: |w, e| match e {
        EntityId::Item(i) => w
            .item(i)
            .and_then(|o| {
                o.ext
                    .get::<crate::quests::collector_mobs::PaintedImageData>()
            })
            .map_or(Value::Null, |d| {
                let x = &d.image;
                Value::Enum(i64::from(*x))
            }),
        _ => Value::Null,
    },
    set: Some(|w, e, v| {
        let EntityId::Item(i) = e else {
            return Err(NOT_HERE.to_string());
        };
        let n = v.i64() as i32;
        let Some(d) = w.item_mut(i).and_then(|o| {
            o.ext
                .get_mut::<crate::quests::collector_mobs::PaintedImageData>()
        }) else {
            return Err(NOT_HERE.to_string());
        };
        if d.image == n {
            return Ok(());
        }
        d.image = n;
        muo_server::item::ops::invalidate_properties(w, i);
        Ok(())
    }),
}];

static SERVER_ENGINES_QUESTS_HORNOFRETREAT: &[PropDef] = &[
    PropDef {
        name: "DestLoc",
        kind: Kind::Plain(PropType::IPoint3D),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::quests::items::HornData>())
                .map_or(Value::Null, |d| {
                    let x = &d.dest_loc;
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
                .and_then(|o| o.ext.get_mut::<crate::quests::items::HornData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.dest_loc == n {
                return Ok(());
            }
            d.dest_loc = n;
            Ok(())
        }),
    },
    PropDef {
        name: "DestMap",
        kind: Kind::Map,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::quests::items::HornData>())
                .map_or(Value::Null, |d| {
                    let x = &d.dest_map;
                    Value::Map(*x)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.map();
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::quests::items::HornData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.dest_map == n {
                return Ok(());
            }
            d.dest_map = n;
            Ok(())
        }),
    },
    PropDef {
        name: "Charges",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::quests::items::HornData>())
                .map_or(Value::Null, |d| {
                    let x = &d.charges;
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
                .and_then(|o| o.ext.get_mut::<crate::quests::items::HornData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.charges == n {
                return Ok(());
            }
            d.charges = n;
            muo_server::item::ops::invalidate_properties(w, i);
            Ok(())
        }),
    },
];

static SERVER_ENGINES_QUESTS_NECRO_MAABUSCOFFIN: &[PropDef] = &[
    PropDef {
        name: "Maabus",
        kind: Kind::Plain(PropType::EntityOf("Maabus")),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::quests::dark_tides_mobs::CoffinData>())
                .map_or(Value::Null, |d| {
                    let x = &d.maabus;
                    Value::Entity(x.map(EntityId::Mobile))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.mobile();
            let Some(d) = w.item_mut(i).and_then(|o| {
                o.ext
                    .get_mut::<crate::quests::dark_tides_mobs::CoffinData>()
            }) else {
                return Err(NOT_HERE.to_string());
            };
            if d.maabus == n {
                return Ok(());
            }
            d.maabus = n;
            Ok(())
        }),
    },
    PropDef {
        name: "SpawnLocation",
        kind: Kind::Plain(PropType::IPoint3D),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::quests::dark_tides_mobs::CoffinData>())
                .map_or(Value::Null, |d| {
                    let x = &d.spawn_location;
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
                    .get_mut::<crate::quests::dark_tides_mobs::CoffinData>()
            }) else {
                return Err(NOT_HERE.to_string());
            };
            if d.spawn_location == n {
                return Ok(());
            }
            d.spawn_location = n;
            Ok(())
        }),
    },
];

static SERVER_ENGINES_QUESTS_DOOM_BELLOFTHEDEAD: &[PropDef] = &[
    PropDef {
        name: "Chyloth",
        kind: Kind::Plain(PropType::EntityOf("Chyloth")),
        read: GM,
        write: ADMIN,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::quests::summoning_mobs::BellData>())
                .map_or(Value::Null, |d| {
                    let x = &d.chyloth;
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
                .and_then(|o| o.ext.get_mut::<crate::quests::summoning_mobs::BellData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.chyloth == n {
                return Ok(());
            }
            d.chyloth = n;
            Ok(())
        }),
    },
    PropDef {
        name: "Dragon",
        kind: Kind::Plain(PropType::EntityOf("SkeletalDragon")),
        read: GM,
        write: ADMIN,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::quests::summoning_mobs::BellData>())
                .map_or(Value::Null, |d| {
                    let x = &d.dragon;
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
                .and_then(|o| o.ext.get_mut::<crate::quests::summoning_mobs::BellData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.dragon == n {
                return Ok(());
            }
            d.dragon = n;
            Ok(())
        }),
    },
];

static SERVER_ENGINES_QUESTS_HAVEN_CANNON: &[PropDef] = &[PropDef {
    name: "Canoneer",
    kind: Kind::Plain(PropType::EntityOf("MilitiaCanoneer")),
    read: GM,
    write: GM,
    get: |w, e| match e {
        EntityId::Item(i) => w
            .item(i)
            .and_then(|o| o.ext.get::<crate::quests::uzeraan_mobs::CannonData>())
            .map_or(Value::Null, |d| {
                let x = &d.canoneer;
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
            .and_then(|o| o.ext.get_mut::<crate::quests::uzeraan_mobs::CannonData>())
        else {
            return Err(NOT_HERE.to_string());
        };
        if d.canoneer == n {
            return Ok(());
        }
        d.canoneer = n;
        Ok(())
    }),
}];

static SERVER_MOBILES_CHARACTERSTATUEDEED: &[PropDef] = &[
    PropDef {
        name: "Statue",
        kind: Kind::Plain(PropType::EntityOf("CharacterStatue")),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::rewards::statue::DeedData>())
                .map_or(Value::Null, |d| {
                    let x = &d.statue;
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
                .and_then(|o| o.ext.get_mut::<crate::rewards::statue::DeedData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.statue == n {
                return Ok(());
            }
            d.statue = n;
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
                .and_then(|o| o.ext.get::<crate::rewards::statue::DeedData>())
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
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::rewards::statue::DeedData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.is_reward_item == n {
                return Ok(());
            }
            d.is_reward_item = n;
            muo_server::item::ops::invalidate_properties(w, i);
            Ok(())
        }),
    },
];

static SERVER_ITEMS_REPAIRDEED: &[PropDef] = &[
    PropDef {
        name: "Skill",
        kind: Kind::Plain(PropType::Enum(E_REPAIRSKILLTYPE)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::craft::repair_deed::RepairDeedData>())
                .map_or(Value::Null, |d| {
                    let x = &d.skill;
                    Value::Enum(i64::from(*x))
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
                .and_then(|o| o.ext.get_mut::<crate::craft::repair_deed::RepairDeedData>())
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
        name: "Crafter",
        kind: STR,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::craft::repair_deed::RepairDeedData>())
                .map_or(Value::Null, |d| {
                    let x = &d.crafter;
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
                .and_then(|o| o.ext.get_mut::<crate::craft::repair_deed::RepairDeedData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.crafter == n {
                return Ok(());
            }
            d.crafter = n;
            muo_server::item::ops::invalidate_properties(w, i);
            Ok(())
        }),
    },
];

static SERVER_ENGINES_QUESTS_COLLECTOR_OBSIDIAN_26: &[PropDef] = &[PropDef {
    name: "Quantity",
    kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
    read: GM,
    write: GM,
    get: |w, e| match e {
        EntityId::Item(i) => w
            .item(i)
            .and_then(|o| o.ext.get::<crate::quests::collector_mobs::ObsidianData>())
            .map_or(Value::Null, |d| {
                let x = &d.quantity;
                Value::Int(i64::from(*x))
            }),
        _ => Value::Null,
    },
    set: Some(|w, e, v| {
        let EntityId::Item(i) = e else {
            return Err(NOT_HERE.to_string());
        };
        let n = v.i64() as i32;
        crate::quests::collector_mobs::set_quantity(w, i, n);
        Ok(())
    }),
}];

static SERVER_ENGINES_QUESTS_DOOM_BELLOFTHEDEAD_27: &[PropDef] = &[PropDef {
    name: "Summoning",
    kind: BOOL,
    read: GM,
    write: ADMIN,
    get: |w, e| match e {
        EntityId::Item(i) => w
            .item(i)
            .and_then(|o| o.ext.get::<crate::quests::summoning_mobs::BellData>())
            .map_or(Value::Null, |d| {
                let x = &d.summoning;
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
            .and_then(|o| o.ext.get_mut::<crate::quests::summoning_mobs::BellData>())
        {
            d.summoning = n;
        };
        Ok(())
    }),
}];

static SERVER_ENGINES_QUESTS_NECRO_MAABUSCOFFINCOMPONENT: &[PropDef] = &[PropDef {
    name: "SpawnLocation",
    kind: Kind::Plain(PropType::IPoint3D),
    read: GM,
    write: GM,
    get: |w, e| match e {
        EntityId::Item(i) => Value::Point3D(
            muo_content::items::addon::component_addon(w, i)
                .and_then(|a| w.item(a))
                .and_then(|o| o.ext.get::<crate::quests::dark_tides_mobs::CoffinData>())
                .map_or(Point3D::default(), |d| d.spawn_location),
        ),
        _ => Value::Null,
    },
    set: Some(|w, e, v| {
        let EntityId::Item(i) = e else {
            return Err(NOT_HERE.to_string());
        };
        let n = v.point();
        if let Some(a) = muo_content::items::addon::component_addon(w, i) {
            if let Some(d) = w.item_mut(a).and_then(|o| {
                o.ext
                    .get_mut::<crate::quests::dark_tides_mobs::CoffinData>()
            }) {
                d.spawn_location = n;
            }
        };
        Ok(())
    }),
}];

static SERVER_ENGINES_QUESTS_HAVEN_CANNONCOMPONENT: &[PropDef] = &[PropDef {
    name: "Canoneer",
    kind: Kind::Plain(PropType::EntityOf("MilitiaCanoneer")),
    read: GM,
    write: GM,
    get: |w, e| match e {
        EntityId::Item(i) => Value::Entity(
            muo_content::items::addon::component_addon(w, i)
                .and_then(|a| w.item(a))
                .and_then(|o| o.ext.get::<crate::quests::uzeraan_mobs::CannonData>())
                .and_then(|d| d.canoneer)
                .map(EntityId::Mobile),
        ),
        _ => Value::Null,
    },
    set: Some(|w, e, v| {
        let EntityId::Item(i) = e else {
            return Err(NOT_HERE.to_string());
        };
        let n = v.mobile();
        if let Some(a) = muo_content::items::addon::component_addon(w, i) {
            if let Some(d) = w
                .item_mut(a)
                .and_then(|o| o.ext.get_mut::<crate::quests::uzeraan_mobs::CannonData>())
            {
                d.canoneer = n;
            }
        };
        Ok(())
    }),
}];

static SERVER_ENGINES_QUESTS_HAVEN_CANNON_30: &[PropDef] = &[PropDef {
    name: "CannonDirection",
    kind: Kind::Plain(PropType::Enum(E_CANNONDIRECTION)),
    read: GM,
    write: GM,
    get: |w, e| match e {
        EntityId::Item(i) => w
            .item(i)
            .and_then(|o| o.ext.get::<crate::quests::uzeraan_mobs::CannonData>())
            .map_or(Value::Null, |d| Value::Enum(i64::from(d.direction))),
        _ => Value::Null,
    },
    set: Some(|w, e, v| {
        let EntityId::Item(i) = e else {
            return Err(NOT_HERE.to_string());
        };
        let n = v.i64() as i32;
        if let Some(d) = w
            .item_mut(i)
            .and_then(|o| o.ext.get_mut::<crate::quests::uzeraan_mobs::CannonData>())
        {
            d.direction = n;
        };
        Ok(())
    }),
}];

static SERVER_ENGINES_QUESTS_HAVEN_MILITIACANONEER: &[PropDef] = &[PropDef {
    name: "Active",
    kind: BOOL,
    read: GM,
    write: GM,
    get: |w, e| match e {
        EntityId::Mobile(m) => Value::Bool(
            w.mobile(m)
                .and_then(|o| o.ext.get::<bool>())
                .copied()
                .unwrap_or(false),
        ),
        _ => Value::Null,
    },
    set: Some(|w, e, v| {
        let EntityId::Mobile(m) = e else {
            return Err(NOT_HERE.to_string());
        };
        let n = v.bool();
        if let Some(o) = w.mobile_mut(m) {
            o.ext.insert(n);
        };
        Ok(())
    }),
}];

static SERVER_ENGINES_QUESTS_HAG_HANGOVERCURE: &[PropDef] = &[PropDef {
    name: "Uses",
    kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
    read: GM,
    write: GM,
    get: |w, e| match e {
        EntityId::Item(i) => Value::Int(
            w.item(i)
                .and_then(|o| o.ext.get::<i32>())
                .copied()
                .map_or(0, i64::from),
        ),
        _ => Value::Null,
    },
    set: Some(|w, e, v| {
        let EntityId::Item(i) = e else {
            return Err(NOT_HERE.to_string());
        };
        let n = v.i64() as i32;
        if let Some(o) = w.item_mut(i) {
            o.ext.insert(n);
        };
        Ok(())
    }),
}];

static SERVER_ENGINES_QUESTS_NINJA_HIDDENFIGURE: &[PropDef] = &[PropDef {
    name: "Message",
    kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
    read: GM,
    write: GM,
    get: |w, e| match e {
        EntityId::Mobile(m) => Value::Int(
            w.mobile(m)
                .and_then(|o| o.ext.get::<i32>())
                .copied()
                .map_or(0, i64::from),
        ),
        _ => Value::Null,
    },
    set: Some(|w, e, v| {
        let EntityId::Mobile(m) = e else {
            return Err(NOT_HERE.to_string());
        };
        let n = v.i64() as i32;
        if let Some(o) = w.mobile_mut(m) {
            o.ext.insert(n);
        };
        Ok(())
    }),
}];

static SERVER_ENGINES_BULKORDERS_SMALLBOD_34: &[PropDef] = &[
    PropDef {
        name: "Type",
        kind: Kind::Plain(PropType::Type),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => crate::bulk_orders::data(w, i)
                .and_then(|d| d.type_name)
                .map_or(Value::Null, |t| Value::Text(t.to_string())),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = match v.string() {
                Some(s) => Some(
                    muo_content::props::class_by_name(w, &s)
                        .ok_or("No type with that name was found.")?,
                ),
                None => None,
            };
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::bulk_orders::BodData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            d.type_name = n;
            Ok(())
        }),
    },
    PropDef {
        name: "Complete",
        kind: BOOL,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => Value::Bool(crate::bulk_orders::complete(w, i)),
            _ => Value::Null,
        },
        set: None,
    },
];

static SERVER_ENGINES_BULKORDERS_LARGEBOD: &[PropDef] = &[PropDef {
    name: "Complete",
    kind: BOOL,
    read: GM,
    write: GM,
    get: |w, e| match e {
        EntityId::Item(i) => Value::Bool(crate::bulk_orders::complete(w, i)),
        _ => Value::Null,
    },
    set: None,
}];

static SERVER_ENGINES_CONPVP_KHBOARD: &[PropDef] = &[PropDef {
    name: "Controller",
    kind: Kind::Plain(PropType::EntityOf("KHController")),
    read: GM,
    write: GM,
    get: |w, e| match e {
        EntityId::Item(i) => w
            .item(i)
            .and_then(|o| o.ext.get::<crate::conpvp::koth::KhBoardData>())
            .map_or(Value::Null, |d| {
                let x = &d.controller;
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
        crate::conpvp::koth::set_board_controller(w, i, n);
        Ok(())
    }),
}];

static SERVER_ITEMS_TROPHY_37: &[PropDef] = &[PropDef {
    name: "Rank",
    kind: Kind::Plain(PropType::Enum(E_TROPHYRANK)),
    read: GM,
    write: GM,
    get: |w, e| match e {
        EntityId::Item(i) => w
            .item(i)
            .and_then(|o| o.ext.get::<crate::conpvp::trophy::TrophyData>())
            .map_or(Value::Null, |d| Value::Enum(d.rank as i64)),
        _ => Value::Null,
    },
    set: Some(|w, e, v| {
        let EntityId::Item(i) = e else {
            return Err(NOT_HERE.to_string());
        };
        let n = crate::conpvp::trophy::TrophyRank::from_i32(v.i64() as i32);
        {
            let changed = w
                .item(i)
                .and_then(|o| o.ext.get::<crate::conpvp::trophy::TrophyData>())
                .is_some_and(|d| d.rank != n);
            if changed {
                if let Some(d) = w
                    .item_mut(i)
                    .and_then(|o| o.ext.get_mut::<crate::conpvp::trophy::TrophyData>())
                {
                    d.rank = n;
                }
                (crate::conpvp::trophy::TROPHY.set_hue)(w, i, n.hue());
            }
        };
        Ok(())
    }),
}];

static SERVER_ENGINES_CONPVP_LADDERITEM: &[PropDef] = &[PropDef {
    name: "Ladder",
    kind: Kind::Plain(PropType::EntityOf("LadderController")),
    read: GM,
    write: GM,
    get: |w, e| match e {
        EntityId::Item(i) => w
            .item(i)
            .and_then(|o| o.ext.get::<crate::conpvp::ladder_gump::LadderItemData>())
            .map_or(Value::Null, |d| {
                let x = &d.ladder;
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
        if let Some(d) = w.item_mut(i).and_then(|o| {
            o.ext
                .get_mut::<crate::conpvp::ladder_gump::LadderItemData>()
        }) {
            d.ladder = n;
        };
        Ok(())
    }),
}];

pub fn configure(w: &mut World) {
    props::add(
        w,
        "Server.Factions.BaseMonolith",
        SERVER_FACTIONS_BASEMONOLITH,
    );
    props::add(
        w,
        "Server.Factions.FactionStone",
        SERVER_FACTIONS_FACTIONSTONE,
    );
    props::add(w, "Server.Factions.JoinStone", SERVER_FACTIONS_JOINSTONE);
    props::add(w, "Server.Factions.TownStone", SERVER_FACTIONS_TOWNSTONE);
    props::add(w, "Server.Factions.Sigil", SERVER_FACTIONS_SIGIL);
    props::add(
        w,
        "Server.Factions.FactionExplosionTrap",
        SERVER_FACTIONS_FACTIONEXPLOSIONTRAP,
    );
    props::add(
        w,
        "Server.Factions.FactionGasTrap",
        SERVER_FACTIONS_FACTIONEXPLOSIONTRAP,
    );
    props::add(
        w,
        "Server.Factions.FactionSawTrap",
        SERVER_FACTIONS_FACTIONEXPLOSIONTRAP,
    );
    props::add(
        w,
        "Server.Factions.FactionSpikeTrap",
        SERVER_FACTIONS_FACTIONEXPLOSIONTRAP,
    );
    props::add(
        w,
        "Server.Factions.FactionExplosionTrapDeed",
        SERVER_FACTIONS_FACTIONEXPLOSIONTRAPDEED,
    );
    props::add(
        w,
        "Server.Factions.FactionGasTrapDeed",
        SERVER_FACTIONS_FACTIONEXPLOSIONTRAPDEED,
    );
    props::add(
        w,
        "Server.Factions.FactionSawTrapDeed",
        SERVER_FACTIONS_FACTIONEXPLOSIONTRAPDEED,
    );
    props::add(
        w,
        "Server.Factions.FactionSpikeTrapDeed",
        SERVER_FACTIONS_FACTIONEXPLOSIONTRAPDEED,
    );
    props::add(
        w,
        "Server.Factions.FactionWarHorse",
        SERVER_FACTIONS_FACTIONWARHORSE,
    );
    props::add(
        w,
        "Server.Factions.FactionBerserker",
        SERVER_FACTIONS_FACTIONBERSERKER,
    );
    props::add(
        w,
        "Server.Factions.FactionDeathKnight",
        SERVER_FACTIONS_FACTIONBERSERKER,
    );
    props::add(
        w,
        "Server.Factions.FactionDragoon",
        SERVER_FACTIONS_FACTIONBERSERKER,
    );
    props::add(
        w,
        "Server.Factions.FactionHenchman",
        SERVER_FACTIONS_FACTIONBERSERKER,
    );
    props::add(
        w,
        "Server.Factions.FactionKnight",
        SERVER_FACTIONS_FACTIONBERSERKER,
    );
    props::add(
        w,
        "Server.Factions.FactionMercenary",
        SERVER_FACTIONS_FACTIONBERSERKER,
    );
    props::add(
        w,
        "Server.Factions.FactionNecromancer",
        SERVER_FACTIONS_FACTIONBERSERKER,
    );
    props::add(
        w,
        "Server.Factions.FactionPaladin",
        SERVER_FACTIONS_FACTIONBERSERKER,
    );
    props::add(
        w,
        "Server.Factions.FactionSorceress",
        SERVER_FACTIONS_FACTIONBERSERKER,
    );
    props::add(
        w,
        "Server.Factions.FactionWizard",
        SERVER_FACTIONS_FACTIONBERSERKER,
    );
    props::add(
        w,
        "Server.Factions.FactionBoardVendor",
        SERVER_FACTIONS_FACTIONBOARDVENDOR,
    );
    props::add(
        w,
        "Server.Factions.FactionBottleVendor",
        SERVER_FACTIONS_FACTIONBOARDVENDOR,
    );
    props::add(
        w,
        "Server.Factions.FactionHorseVendor",
        SERVER_FACTIONS_FACTIONBOARDVENDOR,
    );
    props::add(
        w,
        "Server.Factions.FactionOreVendor",
        SERVER_FACTIONS_FACTIONBOARDVENDOR,
    );
    props::add(
        w,
        "Server.Factions.FactionReagentVendor",
        SERVER_FACTIONS_FACTIONBOARDVENDOR,
    );
    props::add(
        w,
        "Server.Engines.BulkOrders.BaseBOD",
        SERVER_ENGINES_BULKORDERS_BASEBOD,
    );
    props::add(
        w,
        "Server.Engines.BulkOrders.SmallBOD",
        SERVER_ENGINES_BULKORDERS_SMALLBOD,
    );
    props::add(
        w,
        "Server.Engines.BulkOrders.BulkOrderBook",
        SERVER_ENGINES_BULKORDERS_BULKORDERBOOK,
    );
    props::add(
        w,
        "Server.Engines.ConPVP.TournamentBracketItem",
        SERVER_ENGINES_CONPVP_TOURNAMENTBRACKETITEM,
    );
    props::add(
        w,
        "Server.Engines.ConPVP.TournamentRegistrar",
        SERVER_ENGINES_CONPVP_TOURNAMENTREGISTRAR,
    );
    props::add(
        w,
        "Server.Engines.ConPVP.TournamentSignupItem",
        SERVER_ENGINES_CONPVP_TOURNAMENTSIGNUPITEM,
    );
    props::add(w, "Server.Items.Trophy", SERVER_ITEMS_TROPHY);
    props::add(
        w,
        "Server.Factions.FactionTrapRemovalKit",
        SERVER_FACTIONS_FACTIONTRAPREMOVALKIT,
    );
    props::add(
        w,
        "Server.Engines.Quests.Collector.Obsidian",
        SERVER_ENGINES_QUESTS_COLLECTOR_OBSIDIAN,
    );
    props::add(
        w,
        "Server.Engines.Quests.Collector.PaintedImage",
        SERVER_ENGINES_QUESTS_COLLECTOR_PAINTEDIMAGE,
    );
    props::add(
        w,
        "Server.Engines.Quests.HornOfRetreat",
        SERVER_ENGINES_QUESTS_HORNOFRETREAT,
    );
    props::add(
        w,
        "Server.Engines.Quests.Necro.MaabusCoffin",
        SERVER_ENGINES_QUESTS_NECRO_MAABUSCOFFIN,
    );
    props::add(
        w,
        "Server.Engines.Quests.Doom.BellOfTheDead",
        SERVER_ENGINES_QUESTS_DOOM_BELLOFTHEDEAD,
    );
    props::add(
        w,
        "Server.Engines.Quests.Haven.Cannon",
        SERVER_ENGINES_QUESTS_HAVEN_CANNON,
    );
    props::add(
        w,
        "Server.Mobiles.CharacterStatueDeed",
        SERVER_MOBILES_CHARACTERSTATUEDEED,
    );
    props::add(w, "Server.Items.RepairDeed", SERVER_ITEMS_REPAIRDEED);
    props::add(
        w,
        "Server.Engines.Quests.Collector.Obsidian",
        SERVER_ENGINES_QUESTS_COLLECTOR_OBSIDIAN_26,
    );
    props::add(
        w,
        "Server.Engines.Quests.Doom.BellOfTheDead",
        SERVER_ENGINES_QUESTS_DOOM_BELLOFTHEDEAD_27,
    );
    props::add(
        w,
        "Server.Engines.Quests.Necro.MaabusCoffinComponent",
        SERVER_ENGINES_QUESTS_NECRO_MAABUSCOFFINCOMPONENT,
    );
    props::add(
        w,
        "Server.Engines.Quests.Haven.CannonComponent",
        SERVER_ENGINES_QUESTS_HAVEN_CANNONCOMPONENT,
    );
    props::add(
        w,
        "Server.Engines.Quests.Haven.Cannon",
        SERVER_ENGINES_QUESTS_HAVEN_CANNON_30,
    );
    props::add(
        w,
        "Server.Engines.Quests.Haven.MilitiaCanoneer",
        SERVER_ENGINES_QUESTS_HAVEN_MILITIACANONEER,
    );
    props::add(
        w,
        "Server.Engines.Quests.Hag.HangoverCure",
        SERVER_ENGINES_QUESTS_HAG_HANGOVERCURE,
    );
    props::add(
        w,
        "Server.Engines.Quests.Ninja.HiddenFigure",
        SERVER_ENGINES_QUESTS_NINJA_HIDDENFIGURE,
    );
    props::add(
        w,
        "Server.Engines.BulkOrders.SmallBOD",
        SERVER_ENGINES_BULKORDERS_SMALLBOD_34,
    );
    props::add(
        w,
        "Server.Engines.BulkOrders.LargeBOD",
        SERVER_ENGINES_BULKORDERS_LARGEBOD,
    );
    props::add(
        w,
        "Server.Engines.ConPVP.KHBoard",
        SERVER_ENGINES_CONPVP_KHBOARD,
    );
    props::add(w, "Server.Items.Trophy", SERVER_ITEMS_TROPHY_37);
    props::add(
        w,
        "Server.Engines.ConPVP.LadderItem",
        SERVER_ENGINES_CONPVP_LADDERITEM,
    );
}
