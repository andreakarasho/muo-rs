//! The `[CommandProperty]` tables of classes whose members are fields of a data struct in `ext`.
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

static SERVER_ITEMS_ARCANEFOCUS: &[PropDef] = &[PropDef {
    name: "StrengthBonus",
    kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
    read: GM,
    write: GM,
    get: |w, e| match e {
        EntityId::Item(i) => w
            .item(i)
            .and_then(|o| o.ext.get::<crate::spellweaving::items::FocusData>())
            .map_or(Value::Null, |d| {
                let x = &d.strength_bonus;
                Value::Int(i64::from(*x))
            }),
        _ => Value::Null,
    },
    set: Some(|w, e, v| {
        let EntityId::Item(i) = e else {
            return Err(NOT_HERE.to_string());
        };
        let n = v.i64() as i32;
        crate::spellweaving::items::set_strength_bonus(w, i, n);
        Ok(())
    }),
}];

static SERVER_ITEMS_TRANSIENTITEM: &[PropDef] = &[PropDef {
    name: "Expiration",
    kind: Kind::DateTime,
    read: GM,
    write: GM,
    get: |w, e| match e {
        EntityId::Item(i) => w
            .item(i)
            .and_then(|o| o.ext.get::<crate::spellweaving::items::TransientData>())
            .map_or(Value::Null, |d| {
                let x = &d.expiration;
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
            .and_then(|o| o.ext.get_mut::<crate::spellweaving::items::TransientData>())
        else {
            return Err(NOT_HERE.to_string());
        };
        if d.expiration == n {
            return Ok(());
        }
        d.expiration = n;
        Ok(())
    }),
}];

pub fn configure(w: &mut World) {
    props::add(w, "Server.Items.ArcaneFocus", SERVER_ITEMS_ARCANEFOCUS);
    props::add(w, "Server.Items.TransientItem", SERVER_ITEMS_TRANSIENTITEM);
}
