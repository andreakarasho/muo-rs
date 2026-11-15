//! Item family constants read through [`super::props`]: weapon and armor stats (`"aosStrReq"`, `"initMinHits"`,
//! `"basePhysicalResistance"`, ..) moved from the Rust class tables to `Data/Templates/Items`.
//!
//! Each reader returns the prop of the item's class chain, else `fallback`: the Rust class table's value, which for
//! a class without the prop is its parent's (`..PARENT`). The key is the table field in camelCase.

use muo_server::ids::ItemId;
use muo_server::world::World;
use serde_json::Value;

use super::props;
use crate::items::{MlInt, ml, same};

/// An integer prop.
pub fn int(w: &World, id: ItemId, key: &str, fallback: i32) -> i32 {
    props::as_i32(props::item(w, id, key)).unwrap_or(fallback)
}

/// A number prop.
pub fn float(w: &World, id: ItemId, key: &str, fallback: f64) -> f64 {
    props::as_f64(props::item(w, id, key)).unwrap_or(fallback)
}

/// An `MlInt` prop: `n` (the same before and from ML) or `[pre, ml]`.
pub fn ml_int(w: &World, id: ItemId, key: &str, fallback: MlInt) -> MlInt {
    let n = |v: &Value| v.as_i64().and_then(|n| i32::try_from(n).ok());
    match props::item(w, id, key) {
        Some(Value::Array(a)) if a.len() == 2 => match (n(&a[0]), n(&a[1])) {
            (Some(pre), Some(m)) => ml(pre, m),
            _ => fallback,
        },
        Some(v) => n(v).map_or(fallback, same),
        None => fallback,
    }
}
