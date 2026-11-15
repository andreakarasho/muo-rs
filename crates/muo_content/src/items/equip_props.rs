//! Constant class values of weapons, armor, clothing and jewelry that moved from the Rust class tables to the
//! template `props` (`Data/Templates/Items`, the field in camelCase). Each reader returns the prop of the item's
//! class chain, else `fallback` (the class table's value, which a class without the prop inherits from its parent).
//!
//! Enum values are stored by name (`"defSkill": "Swords"`); a name resolves through a table built once per enum,
//! so a read neither allocates nor parses. An explicit `null` is `None` for the `Option` fields.

use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::OnceLock;

use serde_json::Value;

use muo_server::ids::ItemId;
use muo_server::world::World;

use crate::templates::props;

/// The name -> value table of an enum (names are the `Debug` texts), built on first use.
pub struct Names<T: 'static> {
    map: OnceLock<HashMap<String, T>>,
    all: fn() -> Vec<T>,
}

impl<T: Copy + Debug> Names<T> {
    pub const fn new(all: fn() -> Vec<T>) -> Self {
        Self {
            map: OnceLock::new(),
            all,
        }
    }

    pub fn get(&self, name: &str) -> Option<T> {
        self.map
            .get_or_init(|| {
                (self.all)()
                    .into_iter()
                    .map(|v| (format!("{v:?}"), v))
                    .collect()
            })
            .get(name)
            .copied()
    }
}

/// Every value `from` accepts in `0..n`.
pub fn all_of<T>(n: i32, from: fn(i32) -> Option<T>) -> Vec<T> {
    (0..n).filter_map(from).collect()
}

/// The raw prop.
pub fn raw<'a>(w: &'a World, id: ItemId, key: &str) -> Option<&'a Value> {
    props::item(w, id, key)
}

/// An enum prop by name, else `fallback`.
pub fn named<T: Copy + Debug>(
    w: &World,
    id: ItemId,
    key: &str,
    names: &Names<T>,
    fallback: T,
) -> T {
    raw(w, id, key)
        .and_then(Value::as_str)
        .and_then(|s| names.get(s))
        .unwrap_or(fallback)
}

/// An optional enum prop: a name, or `null` for none; else `fallback`.
pub fn named_opt<T: Copy + Debug>(
    w: &World,
    id: ItemId,
    key: &str,
    names: &Names<T>,
    fallback: Option<T>,
) -> Option<T> {
    match raw(w, id, key) {
        Some(Value::Null) => None,
        Some(Value::String(s)) => names.get(s).or(fallback),
        _ => fallback,
    }
}

/// An optional number prop (`null` for none).
pub fn float_opt(w: &World, id: ItemId, key: &str, fallback: Option<f64>) -> Option<f64> {
    match raw(w, id, key) {
        Some(Value::Null) => None,
        Some(v) => v.as_f64().or(fallback),
        None => fallback,
    }
}

/// An optional string prop (`null` for none), interned when the table loaded.
pub fn str_opt(
    w: &World,
    id: ItemId,
    key: &str,
    fallback: Option<&'static str>,
) -> Option<&'static str> {
    props::as_static_str(w, raw(w, id, key)).unwrap_or(fallback)
}

/// A bool prop.
pub fn boolean(w: &World, id: ItemId, key: &str, fallback: bool) -> bool {
    props::as_bool(raw(w, id, key)).unwrap_or(fallback)
}

/// `Race.Allow*` by name (`"requiredRaces": "AllowElvesOnly"`).
pub fn races(s: &str) -> Option<i32> {
    use muo_server::race::*;
    Some(match s {
        "AllowAllRaces" => ALLOW_ALL_RACES,
        "AllowHumanOrElves" => ALLOW_HUMAN_OR_ELVES,
        "AllowElvesOnly" => ALLOW_ELVES_ONLY,
        "AllowGargoylesOnly" => ALLOW_GARGOYLES_ONLY,
        _ => return None,
    })
}

/// `RequiredRaces` from the `requiredRaces` prop, else `fallback`.
pub fn required_races(w: &World, id: ItemId, fallback: Option<i32>) -> Option<i32> {
    match raw(w, id, "requiredRaces") {
        Some(Value::Null) => None,
        Some(Value::String(s)) => races(s).or(fallback),
        _ => fallback,
    }
}

pub static SKILLS: Names<muo_server::skills::SkillName> = Names::new(skills);
pub static RESOURCES: Names<crate::craft::CraftResource> = Names::new(resources);

fn skills() -> Vec<muo_server::skills::SkillName> {
    all_of(64, muo_server::skills::SkillName::from_i32)
}

fn resources() -> Vec<crate::craft::CraftResource> {
    all_of(512, crate::craft::CraftResource::from_i32)
}
