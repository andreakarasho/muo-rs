//! Template keys of weapons, armor, clothing, jewelry and the artifact folders (`muo_content::templates::keys`).

use serde_json::Value;

use muo_content::craft::CraftResource;
use muo_content::items::slayer::SlayerName;
use muo_content::items::{armor, clothing, weapon, weapon_combat};
use muo_content::templates::keys::{check_bool, check_int, int, item_key};
use muo_server::World;
use muo_server::ids::ItemId;
use muo_server::type_registry::TypeRegistry;

use crate::weapons::ctor;

/// The value of `from` whose `Debug` name is `s` (indexes `0..n`).
fn by_name<T: std::fmt::Debug>(s: &str, n: i32, from: fn(i32) -> Option<T>) -> Option<T> {
    (0..n).filter_map(from).find(|v| format!("{v:?}") == s)
}

fn slayer(v: &Value) -> Option<SlayerName> {
    by_name(v.as_str()?, 64, SlayerName::from_i32)
}

fn resource(v: &Value) -> Option<CraftResource> {
    by_name(v.as_str()?, 512, CraftResource::from_i32)
}

fn named<T>(v: &Value, f: fn(&Value) -> Option<T>, what: &str) -> Result<(), String> {
    f(v).map(|_| ())
        .ok_or_else(|| format!("expected a {what} name, got {v}"))
}

fn is_true(v: &Value) -> Result<(), String> {
    match v {
        Value::Bool(true) => Ok(()),
        _ => Err(format!("expected true, got {v}")),
    }
}

pub(crate) fn register(types: &mut TypeRegistry) {
    // A test world registers the keys before the crate's whole registration runs on it.
    let keys = types
        .extensions
        .get::<muo_content::templates::keys::ItemKeys>();
    if keys.is_some_and(|k| k.0.contains_key("slayer")) {
        return;
    }
    // Weapons.
    item_key(
        types,
        "slayer",
        |v| named(v, slayer, "SlayerName"),
        |w, id, v| ctor::set_slayer(w, id, slayer(v).unwrap_or(SlayerName::None)),
    );
    item_key(
        types,
        "slayer2",
        |v| named(v, slayer, "SlayerName"),
        |w, id, v| ctor::set_slayer2(w, id, slayer(v).unwrap_or(SlayerName::None) as i32),
    );
    item_key(
        types,
        "resource",
        |v| named(v, resource, "CraftResource"),
        apply_resource,
    );
    item_key(types, "balanced", check_bool, |w, id, v| {
        ctor::set_balanced(w, id, v.as_bool().unwrap_or(false))
    });
    item_key(types, "velocity", check_int, |w, id, v| {
        ctor::set_velocity(w, id, int(v))
    });
    item_key(types, "usesRemaining", check_int, |w, id, v| {
        ctor::set_uses_remaining(w, id, int(v))
    });
    // `Charges = n` of a weapon (a wand's or FireworksWand's `WeaponData::charges`).
    item_key(types, "weaponCharges", check_int, |w, id, v| {
        crate::weapons::special::set_charges(w, id, int(v))
    });
    item_key(types, "weaponShowUsesRemaining", check_bool, |w, id, v| {
        ctor::set_show_uses_remaining(w, id, v.as_bool().unwrap_or(false))
    });
    // `Hue = ElementalDamageHue` (the hue of the class's damage types).
    item_key(types, "elementalDamageHue", is_true, |w, id, _| {
        let h = weapon_combat::elemental_damage_hue(w, id);
        ctor::set_hue(w, id, h);
    });
    // Clothing.
    item_key(types, "strRequirement", check_int, |w, id, v| {
        clothing::set_str_requirement(w, id, int(v))
    });
    // Quivers.
    item_key(types, "quiverWeightReduction", check_int, |w, id, v| {
        crate::quivers::set_weight_reduction(w, id, int(v))
    });
    item_key(types, "quiverLowerAmmoCost", check_int, |w, id, v| {
        crate::quivers::set_lower_ammo_cost(w, id, int(v))
    });
    item_key(types, "quiverDamageIncrease", check_int, |w, id, v| {
        crate::quivers::set_damage_increase(w, id, int(v))
    });
    // Armor: `HitPoints = MaxHitPoints = value`.
    item_key(types, "hitPoints", check_int, |w, id, v| {
        crate::armor::max_and_hit_points(w, id, int(v))
    });
}

/// `Resource = value` of a weapon, armor or clothing.
fn apply_resource(w: &mut World, id: ItemId, v: &Value) {
    let r = resource(v).unwrap_or_default();
    if weapon::is_weapon(w, id) {
        weapon::set_resource(w, id, r);
    } else if armor::data(w, id).is_some() {
        armor::set_resource(w, id, r);
    } else {
        clothing::set_resource(w, id, r);
    }
}
