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

const E_BOATORDER: &[(&str, i64)] = &[("Move", 0), ("Course", 1), ("Single", 2)];
const E_DECAYLEVEL: &[(&str, i64)] = &[
    ("Ageless", 0),
    ("LikeNew", 1),
    ("Slightly", 2),
    ("Somewhat", 3),
    ("Fairly", 4),
    ("Greatly", 5),
    ("IDOC", 6),
    ("Collapsed", 7),
    ("DemolitionPending", 8),
];
const E_DIRECTION: &[(&str, i64)] = &[
    ("North", 0),
    ("Right", 1),
    ("East", 2),
    ("Down", 3),
    ("South", 4),
    ("Left", 5),
    ("West", 6),
    ("Up", 7),
    ("Mask", 7),
    ("Running", 128),
    ("ValueMask", 135),
];
const E_PLANKSIDE: &[(&str, i64)] = &[("Port", 0), ("Starboard", 1)];
const E_SECURELEVEL: &[(&str, i64)] = &[
    ("Owner", 0),
    ("CoOwners", 1),
    ("Friends", 2),
    ("Anyone", 3),
    ("Guild", 4),
];
static SERVER_MULTIS_BASEBOAT: &[PropDef] = &[
    PropDef {
        name: "MapItem",
        kind: Kind::Plain(PropType::EntityOf("MapItem")),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::boats::BoatData>())
                .map_or(Value::Null, |d| {
                    let x = &d.map_item;
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
                .and_then(|o| o.ext.get_mut::<crate::boats::BoatData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.map_item == n {
                return Ok(());
            }
            d.map_item = n;
            Ok(())
        }),
    },
    PropDef {
        name: "NextNavPoint",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::boats::BoatData>())
                .map_or(Value::Null, |d| {
                    let x = &d.next_nav_point;
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
                .and_then(|o| o.ext.get_mut::<crate::boats::BoatData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.next_nav_point == n {
                return Ok(());
            }
            d.next_nav_point = n;
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
                .and_then(|o| o.ext.get::<crate::boats::BoatData>())
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
            crate::boats::set_owner(w, i, n);
            Ok(())
        }),
    },
    PropDef {
        name: "PPlank",
        kind: Kind::Plain(PropType::EntityOf("Plank")),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::boats::BoatData>())
                .map_or(Value::Null, |d| {
                    let x = &d.p_plank;
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
                .and_then(|o| o.ext.get_mut::<crate::boats::BoatData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.p_plank == n {
                return Ok(());
            }
            d.p_plank = n;
            Ok(())
        }),
    },
    PropDef {
        name: "SPlank",
        kind: Kind::Plain(PropType::EntityOf("Plank")),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::boats::BoatData>())
                .map_or(Value::Null, |d| {
                    let x = &d.s_plank;
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
                .and_then(|o| o.ext.get_mut::<crate::boats::BoatData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.s_plank == n {
                return Ok(());
            }
            d.s_plank = n;
            Ok(())
        }),
    },
    PropDef {
        name: "TillerMan",
        kind: Kind::Plain(PropType::EntityOf("TillerMan")),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::boats::BoatData>())
                .map_or(Value::Null, |d| {
                    let x = &d.tiller_man;
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
                .and_then(|o| o.ext.get_mut::<crate::boats::BoatData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.tiller_man == n {
                return Ok(());
            }
            d.tiller_man = n;
            Ok(())
        }),
    },
    PropDef {
        name: "Hold",
        kind: Kind::Plain(PropType::EntityOf("Hold")),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::boats::BoatData>())
                .map_or(Value::Null, |d| {
                    let x = &d.hold;
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
                .and_then(|o| o.ext.get_mut::<crate::boats::BoatData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.hold == n {
                return Ok(());
            }
            d.hold = n;
            Ok(())
        }),
    },
    PropDef {
        name: "Anchored",
        kind: BOOL,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::boats::BoatData>())
                .map_or(Value::Null, |d| {
                    let x = &d.anchored;
                    Value::Bool(*x)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.bool();
            crate::boats::set_anchored(w, i, n);
            Ok(())
        }),
    },
    PropDef {
        name: "TimeOfDecay",
        kind: Kind::DateTime,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::boats::BoatData>())
                .map_or(Value::Null, |d| {
                    let x = &d.time_of_decay;
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
            crate::boats::set_time_of_decay(w, i, n);
            Ok(())
        }),
    },
    PropDef {
        name: "ShipName",
        kind: STR,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::boats::BoatData>())
                .map_or(Value::Null, |d| {
                    let x = &d.ship_name;
                    x.clone().map_or(Value::Null, Value::Str)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.string();
            crate::boats::set_ship_name(w, i, n);
            Ok(())
        }),
    },
    PropDef {
        name: "Facing",
        kind: Kind::Plain(PropType::Enum(E_DIRECTION)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::boats::BoatData>())
                .map_or(Value::Null, |d| Value::Enum(i64::from(d.facing))),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            {
                crate::boats::set_facing(w, i, n);
            };
            Ok(())
        }),
    },
    PropDef {
        name: "Moving",
        kind: Kind::Plain(PropType::Enum(E_DIRECTION)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::boats::BoatData>())
                .map_or(Value::Null, |d| Value::Enum(i64::from(d.moving))),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::boats::BoatData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.moving == n {
                return Ok(());
            }
            d.moving = n;
            Ok(())
        }),
    },
    PropDef {
        name: "IsMoving",
        kind: BOOL,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::boats::BoatData>())
                .map_or(Value::Null, |d| Value::Bool(crate::boats::is_moving(w, i))),
            _ => Value::Null,
        },
        set: None,
    },
    PropDef {
        name: "Speed",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::boats::BoatData>())
                .map_or(Value::Null, |d| {
                    let x = &d.speed;
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
                .and_then(|o| o.ext.get_mut::<crate::boats::BoatData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.speed == n {
                return Ok(());
            }
            d.speed = n;
            Ok(())
        }),
    },
    PropDef {
        name: "Order",
        kind: Kind::Plain(PropType::Enum(E_BOATORDER)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::boats::BoatData>())
                .map_or(Value::Null, |d| Value::Enum(d.order as i64)),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = match v.i64() {
                1 => crate::boats::BoatOrder::Course,
                2 => crate::boats::BoatOrder::Single,
                _ => crate::boats::BoatOrder::Move,
            };
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::boats::BoatData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.order == n {
                return Ok(());
            }
            d.order = n;
            Ok(())
        }),
    },
];

static SERVER_ITEMS_PLANK: &[PropDef] = &[
    PropDef {
        name: "Boat",
        kind: Kind::Plain(PropType::EntityOf("BaseBoat")),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::boats::parts::PlankData>())
                .map_or(Value::Null, |d| {
                    let x = &d.boat;
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
                .and_then(|o| o.ext.get_mut::<crate::boats::parts::PlankData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.boat == n {
                return Ok(());
            }
            d.boat = n;
            Ok(())
        }),
    },
    PropDef {
        name: "Locked",
        kind: BOOL,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::boats::parts::PlankData>())
                .map_or(Value::Null, |d| {
                    let x = &d.locked;
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
                .and_then(|o| o.ext.get_mut::<crate::boats::parts::PlankData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.locked == n {
                return Ok(());
            }
            d.locked = n;
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
                .and_then(|o| o.ext.get::<crate::boats::parts::PlankData>())
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
                .and_then(|o| o.ext.get_mut::<crate::boats::parts::PlankData>())
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
        name: "Side",
        kind: Kind::Plain(PropType::Enum(E_PLANKSIDE)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::boats::parts::PlankData>())
                .map_or(Value::Null, |d| Value::Enum(d.side as i64)),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = if v.i64() == 1 {
                crate::boats::parts::PlankSide::Starboard
            } else {
                crate::boats::parts::PlankSide::Port
            };
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::boats::parts::PlankData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.side == n {
                return Ok(());
            }
            d.side = n;
            Ok(())
        }),
    },
    PropDef {
        name: "IsOpen",
        kind: BOOL,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => {
                w.item(i)
                    .and_then(|o| o.ext.get::<crate::boats::parts::PlankData>())
                    .map_or(Value::Null, |d| {
                        Value::Bool(w.item(i).is_some_and(|x| {
                            matches!(x.item_id, 0x3ED5 | 0x3ED4 | 0x3E84 | 0x3E89)
                        }))
                    })
            }
            _ => Value::Null,
        },
        set: None,
    },
    PropDef {
        name: "Starboard",
        kind: BOOL,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::boats::parts::PlankData>())
                .map_or(Value::Null, |d| {
                    Value::Bool(d.side == crate::boats::parts::PlankSide::Starboard)
                }),
            _ => Value::Null,
        },
        set: None,
    },
];

static SERVER_MULTIS_LARGEBOATDEED: &[PropDef] = &[
    PropDef {
        name: "MultiId",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::boats::items::BoatDeedData>())
                .map_or(Value::Null, |d| {
                    let x = &d.multi_id;
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
                .and_then(|o| o.ext.get_mut::<crate::boats::items::BoatDeedData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.multi_id == n {
                return Ok(());
            }
            d.multi_id = n;
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
                .and_then(|o| o.ext.get::<crate::boats::items::BoatDeedData>())
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
                .and_then(|o| o.ext.get_mut::<crate::boats::items::BoatDeedData>())
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

static SERVER_MULTIS_LARGEDOCKEDBOAT: &[PropDef] = &[
    PropDef {
        name: "MultiId",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::boats::items::DockedData>())
                .map_or(Value::Null, |d| {
                    let x = &d.multi_id;
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
                .and_then(|o| o.ext.get_mut::<crate::boats::items::DockedData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.multi_id == n {
                return Ok(());
            }
            d.multi_id = n;
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
                .and_then(|o| o.ext.get::<crate::boats::items::DockedData>())
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
                .and_then(|o| o.ext.get_mut::<crate::boats::items::DockedData>())
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
    PropDef {
        name: "ShipName",
        kind: STR,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::boats::items::DockedData>())
                .map_or(Value::Null, |d| {
                    let x = &d.ship_name;
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
                .and_then(|o| o.ext.get_mut::<crate::boats::items::DockedData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.ship_name == n {
                return Ok(());
            }
            d.ship_name = n;
            muo_server::item::ops::invalidate_properties(w, i);
            Ok(())
        }),
    },
];

static SERVER_MULTIS_BASEHOUSE: &[PropDef] = &[
    PropDef {
        name: "LastRefreshed",
        kind: Kind::DateTime,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::house::HouseData>())
                .map_or(Value::Null, |d| {
                    let x = &d.last_refreshed;
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
                .and_then(|o| o.ext.get_mut::<crate::house::HouseData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.last_refreshed == n {
                return Ok(());
            }
            d.last_refreshed = n;
            Ok(())
        }),
    },
    PropDef {
        name: "RestrictDecay",
        kind: BOOL,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::house::HouseData>())
                .map_or(Value::Null, |d| {
                    let x = &d.restrict_decay;
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
                .and_then(|o| o.ext.get_mut::<crate::house::HouseData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.restrict_decay == n {
                return Ok(());
            }
            d.restrict_decay = n;
            Ok(())
        }),
    },
    PropDef {
        name: "Visits",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::house::HouseData>())
                .map_or(Value::Null, |d| {
                    let x = &d.visits;
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
                .and_then(|o| o.ext.get_mut::<crate::house::HouseData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.visits == n {
                return Ok(());
            }
            d.visits = n;
            Ok(())
        }),
    },
    PropDef {
        name: "MaxSecures",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::house::HouseData>())
                .map_or(Value::Null, |d| {
                    let x = &d.max_secures;
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
                .and_then(|o| o.ext.get_mut::<crate::house::HouseData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.max_secures == n {
                return Ok(());
            }
            d.max_secures = n;
            Ok(())
        }),
    },
    PropDef {
        name: "MaxLockDowns",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::house::HouseData>())
                .map_or(Value::Null, |d| {
                    let x = &d.max_lock_downs;
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
                .and_then(|o| o.ext.get_mut::<crate::house::HouseData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.max_lock_downs == n {
                return Ok(());
            }
            d.max_lock_downs = n;
            Ok(())
        }),
    },
    PropDef {
        name: "Price",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::house::HouseData>())
                .map_or(Value::Null, |d| {
                    let x = &d.price;
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
                .and_then(|o| o.ext.get_mut::<crate::house::HouseData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.price == n {
                return Ok(());
            }
            d.price = n;
            Ok(())
        }),
    },
    PropDef {
        name: "NextDecayStage",
        kind: Kind::DateTime,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::house::HouseData>())
                .map_or(Value::Null, |d| {
                    let x = &d.next_decay_stage;
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
                .and_then(|o| o.ext.get_mut::<crate::house::HouseData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.next_decay_stage == n {
                return Ok(());
            }
            d.next_decay_stage = n;
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
                .and_then(|o| o.ext.get::<crate::house::HouseData>())
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
            crate::house::access::set_owner(w, i, n);
            Ok(())
        }),
    },
    PropDef {
        name: "Public",
        kind: BOOL,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::house::HouseData>())
                .map_or(Value::Null, |d| {
                    let x = &d.public;
                    Value::Bool(*x)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.bool();
            crate::house::access::set_public(w, i, n);
            Ok(())
        }),
    },
    PropDef {
        name: "RelativeBanLocation",
        kind: Kind::Plain(PropType::IPoint3D),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::house::HouseData>())
                .map_or(Value::Null, |d| {
                    let x = &d.relative_ban_location;
                    Value::Point3D(*x)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.point();
            crate::house::set_relative_ban_location(w, i, n);
            Ok(())
        }),
    },
    PropDef {
        name: "BanLocation",
        kind: Kind::Plain(PropType::IPoint3D),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::house::HouseData>())
                .map_or(Value::Null, |d| {
                    Value::Point3D(crate::house::ban_location(w, i))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.point();
            crate::house::set_ban_location(w, i, n);
            Ok(())
        }),
    },
    PropDef {
        name: "DecayLevel",
        kind: Kind::Plain(PropType::Enum(E_DECAYLEVEL)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::house::HouseData>())
                .map_or(Value::Null, |d| {
                    Value::Enum(crate::house::decay::decay_level(w, i) as i64)
                }),
            _ => Value::Null,
        },
        set: None,
    },
    PropDef {
        name: "BonusStorageScalar",
        kind: Kind::Plain(PropType::Double),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::house::HouseData>())
                .map_or(Value::Null, |d| {
                    Value::Float(crate::house::bonus_storage_scalar(w))
                }),
            _ => Value::Null,
        },
        set: None,
    },
];

static SERVER_MULTIS_HOUSESIGN: &[PropDef] = &[
    PropDef {
        name: "OriginalOwner",
        kind: Kind::Plain(PropType::EntityOf("Mobile")),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::sign::HouseSignData>())
                .map_or(Value::Null, |d| {
                    let x = &d.original_owner;
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
                .and_then(|o| o.ext.get_mut::<crate::sign::HouseSignData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.original_owner == n {
                return Ok(());
            }
            d.original_owner = n;
            Ok(())
        }),
    },
    PropDef {
        name: "RestrictDecay",
        kind: BOOL,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::sign::HouseSignData>())
                .map_or(Value::Null, |d| {
                    Value::Bool(crate::sign::house_of(w, i).is_some_and(|h| {
                        w.item(h)
                            .and_then(|x| x.ext.get::<crate::house::HouseData>())
                            .is_some_and(|d| d.restrict_decay)
                    }))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.bool();
            if let Some(h) = crate::sign::house_of(w, i) {
                if let Some(d) = w
                    .item_mut(h)
                    .and_then(|x| x.ext.get_mut::<crate::house::HouseData>())
                {
                    d.restrict_decay = n;
                }
            };
            Ok(())
        }),
    },
];

static SERVER_MULTIS_DEEDS_HOUSEDEED: &[PropDef] = &[
    PropDef {
        name: "Offset",
        kind: Kind::Plain(PropType::IPoint3D),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::deeds::DeedData>())
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
                .and_then(|o| o.ext.get_mut::<crate::deeds::DeedData>())
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
    PropDef {
        name: "MultiID",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::deeds::DeedData>())
                .map_or(Value::Null, |d| {
                    let x = &d.multi_id;
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
                .and_then(|o| o.ext.get_mut::<crate::deeds::DeedData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.multi_id == n {
                return Ok(());
            }
            d.multi_id = n;
            Ok(())
        }),
    },
];

static SERVER_ITEMS_HOUSETELEPORTER: &[PropDef] = &[
    PropDef {
        name: "Level",
        kind: Kind::Plain(PropType::Enum(E_SECURELEVEL)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::teleporter::TeleporterData>())
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
            let n = crate::house::SecureLevel::from_u8(v.i64() as u8);
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::teleporter::TeleporterData>())
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
        name: "Target",
        kind: Kind::Plain(PropType::EntityOf("Item")),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::teleporter::TeleporterData>())
                .map_or(Value::Null, |d| {
                    let x = &d.target;
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
            crate::teleporter::set_target(w, i, n);
            Ok(())
        }),
    },
];

pub fn configure(w: &mut World) {
    props::add(w, "Server.Multis.BaseBoat", SERVER_MULTIS_BASEBOAT);
    props::add(w, "Server.Items.Plank", SERVER_ITEMS_PLANK);
    props::add(
        w,
        "Server.Multis.LargeBoatDeed",
        SERVER_MULTIS_LARGEBOATDEED,
    );
    props::add(
        w,
        "Server.Multis.LargeDragonBoatDeed",
        SERVER_MULTIS_LARGEBOATDEED,
    );
    props::add(
        w,
        "Server.Multis.MediumBoatDeed",
        SERVER_MULTIS_LARGEBOATDEED,
    );
    props::add(
        w,
        "Server.Multis.MediumDragonBoatDeed",
        SERVER_MULTIS_LARGEBOATDEED,
    );
    props::add(
        w,
        "Server.Multis.SmallBoatDeed",
        SERVER_MULTIS_LARGEBOATDEED,
    );
    props::add(
        w,
        "Server.Multis.SmallDragonBoatDeed",
        SERVER_MULTIS_LARGEBOATDEED,
    );
    props::add(
        w,
        "Server.Multis.LargeDockedBoat",
        SERVER_MULTIS_LARGEDOCKEDBOAT,
    );
    props::add(
        w,
        "Server.Multis.LargeDockedDragonBoat",
        SERVER_MULTIS_LARGEDOCKEDBOAT,
    );
    props::add(
        w,
        "Server.Multis.MediumDockedBoat",
        SERVER_MULTIS_LARGEDOCKEDBOAT,
    );
    props::add(
        w,
        "Server.Multis.MediumDockedDragonBoat",
        SERVER_MULTIS_LARGEDOCKEDBOAT,
    );
    props::add(
        w,
        "Server.Multis.SmallDockedBoat",
        SERVER_MULTIS_LARGEDOCKEDBOAT,
    );
    props::add(
        w,
        "Server.Multis.SmallDockedDragonBoat",
        SERVER_MULTIS_LARGEDOCKEDBOAT,
    );
    props::add(w, "Server.Multis.BaseHouse", SERVER_MULTIS_BASEHOUSE);
    props::add(w, "Server.Multis.HouseSign", SERVER_MULTIS_HOUSESIGN);
    props::add(
        w,
        "Server.Multis.Deeds.HouseDeed",
        SERVER_MULTIS_DEEDS_HOUSEDEED,
    );
    props::add(
        w,
        "Server.Items.HouseTeleporter",
        SERVER_ITEMS_HOUSETELEPORTER,
    );
}
