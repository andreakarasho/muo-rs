//! Template keys added by content crates: a family-specific statement (`"slayer": "Silver"`,
//! `"usesRemaining": 50`) that the template core cannot call because its setter lives in a higher crate.
//!
//! A crate registers its keys in its `register(types)`, so every world built from the registry (the server, and
//! the test worlds of the whole program) knows them before any template loads. A key is checked when the file
//! loads (`check`) and replayed with the raw JSON value (`apply`).

use std::collections::HashMap;

use serde_json::Value;

use muo_server::ids::{ItemId, MobileId};
use muo_server::type_registry::TypeRegistry;
use muo_server::world::World;

#[derive(Copy, Clone)]
pub struct KeyDef<Id> {
    /// Validates the value at load; the message names what was expected.
    pub check: fn(&Value) -> Result<(), String>,
    /// Applies the (checked) value to a new entity.
    pub apply: fn(&mut World, Id, &Value),
}

impl<Id> std::fmt::Debug for KeyDef<Id> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("KeyDef")
    }
}

impl<Id> PartialEq for KeyDef<Id> {
    fn eq(&self, o: &Self) -> bool {
        std::ptr::fn_addr_eq(self.apply, o.apply)
    }
}

#[derive(Default)]
pub struct ItemKeys(pub HashMap<&'static str, KeyDef<ItemId>>);

#[derive(Default)]
pub struct MobileKeys(pub HashMap<&'static str, KeyDef<MobileId>>);

/// Adds an item template key. A name the core already uses, or one registered twice, is a programming error.
pub fn item_key(
    types: &mut TypeRegistry,
    name: &'static str,
    check: fn(&Value) -> Result<(), String>,
    apply: fn(&mut World, ItemId, &Value),
) {
    let old = types
        .extensions
        .get_mut::<ItemKeys>()
        .0
        .insert(name, KeyDef { check, apply });
    assert!(old.is_none(), "item template key {name} registered twice");
}

/// Adds a mobile template key.
pub fn mobile_key(
    types: &mut TypeRegistry,
    name: &'static str,
    check: fn(&Value) -> Result<(), String>,
    apply: fn(&mut World, MobileId, &Value),
) {
    let old = types
        .extensions
        .get_mut::<MobileKeys>()
        .0
        .insert(name, KeyDef { check, apply });
    assert!(old.is_none(), "mobile template key {name} registered twice");
}

/// The keys a load can use (empty for a bare registry).
#[derive(Default)]
pub struct Keys<'a> {
    pub items: Option<&'a ItemKeys>,
    pub mobiles: Option<&'a MobileKeys>,
}

impl<'a> Keys<'a> {
    pub fn of(types: &'a TypeRegistry) -> Self {
        Self {
            items: types.extensions.get::<ItemKeys>(),
            mobiles: types.extensions.get::<MobileKeys>(),
        }
    }
}

// Checks shared by key definitions.

pub fn check_int(v: &Value) -> Result<(), String> {
    v.as_i64()
        .and_then(|n| i32::try_from(n).ok())
        .map(|_| ())
        .ok_or_else(|| format!("expected an integer, got {v}"))
}

pub fn check_bool(v: &Value) -> Result<(), String> {
    v.as_bool()
        .map(|_| ())
        .ok_or_else(|| format!("expected true or false, got {v}"))
}

pub fn check_number(v: &Value) -> Result<(), String> {
    v.as_f64()
        .map(|_| ())
        .ok_or_else(|| format!("expected a number, got {v}"))
}

/// An int value of a checked key (0 if the check was skipped).
pub fn int(v: &Value) -> i32 {
    v.as_i64().and_then(|n| i32::try_from(n).ok()).unwrap_or(0)
}

/// A check for a string naming one of `names`.
pub fn check_name(v: &Value, names: &[&str]) -> Result<(), String> {
    let s = v
        .as_str()
        .ok_or_else(|| format!("expected a string, got {v}"))?;
    if names.contains(&s) {
        Ok(())
    } else {
        Err(format!("unknown value {s}"))
    }
}
