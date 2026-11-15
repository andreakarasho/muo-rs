//! `CreatureType` members whose value is a template prop: a class whose override was a constant points the member
//! here and keeps the constant in its template's `"props"`. Without the prop (no class in the chain sets it, or no
//! table is loaded) a member answers what `BASE_CREATURE_C` answers.

use serde_json::Value;

use muo_server::ids::MobileId;
use muo_server::poison::PoisonId;
use muo_server::world::World;

use super::props::{self, as_bool, as_i32, as_static_str};
use crate::creature::BASE_CREATURE_C;
use crate::creature::setters::add_loot_n;
use crate::loot_pack::{self, LootPack};

macro_rules! str_prop {
    ($($f:ident = $key:literal;)*) => {$(
        pub fn $f(w: &World, id: MobileId) -> Option<&'static str> {
            as_static_str(w, props::mobile(w, id, $key)).unwrap_or_else(|| (BASE_CREATURE_C.$f)(w, id))
        }
    )*};
}

macro_rules! int_prop {
    ($($f:ident = $key:literal;)*) => {$(
        pub fn $f(w: &World, id: MobileId) -> i32 {
            as_i32(props::mobile(w, id, $key)).unwrap_or_else(|| (BASE_CREATURE_C.$f)(w, id))
        }
    )*};
}

macro_rules! bool_prop {
    ($($f:ident = $key:literal;)*) => {$(
        pub fn $f(w: &World, id: MobileId) -> bool {
            as_bool(props::mobile(w, id, $key)).unwrap_or_else(|| (BASE_CREATURE_C.$f)(w, id))
        }
    )*};
}

str_prop! {
    corpse_name = "corpseName";
    default_name = "defaultName";
}

int_prop! {
    meat = "meat";
    hides = "hides";
    feathers = "feathers";
    scales = "scales";
    steps_max = "stepsMax";
}

bool_prop! {
    bleed_immune = "bleedImmune";
    can_rummage_corpses = "canRummageCorpses";
    always_murderer = "alwaysMurderer";
    can_fly = "canFly";
    gives_ml_minor_artifact = "givesMlMinorArtifact";
}

pub fn treasure_map_level(w: &mut World, id: MobileId) -> i32 {
    match as_i32(props::mobile(w, id, "treasureMapLevel")) {
        Some(v) => v,
        None => (BASE_CREATURE_C.treasure_map_level)(w, id),
    }
}

/// `"poisonImmune": <level>` or `null`.
pub fn poison_immune(w: &World, id: MobileId) -> Option<PoisonId> {
    match props::mobile(w, id, "poisonImmune") {
        Some(Value::Null) => None,
        Some(v) => v.as_u64().and_then(|n| u8::try_from(n).ok()).map(PoisonId),
        None => (BASE_CREATURE_C.poison_immune)(w, id),
    }
}

/// A `"loot"` pack name: the era selectors and the fixed packs.
fn loot_pack(w: &World, name: &str) -> Option<&'static LootPack> {
    Some(match name {
        "Poor" => loot_pack::poor(w),
        "Meager" => loot_pack::meager(w),
        "Average" => loot_pack::average(w),
        "Rich" => loot_pack::rich(w),
        "FilthyRich" => loot_pack::filthy_rich(w),
        "UltraRich" => loot_pack::ultra_rich(w),
        "SuperBoss" => loot_pack::super_boss(w),
        "Gems" => &loot_pack::GEMS,
        "LowScrolls" => &loot_pack::LOW_SCROLLS,
        "MedScrolls" => &loot_pack::MED_SCROLLS,
        "HighScrolls" => &loot_pack::HIGH_SCROLLS,
        "Potions" => &loot_pack::POTIONS,
        _ => return None,
    })
}

/// `"loot": [["Rich", 1], ["Average", 2], ...]`: `AddLoot(pack, amount)` in order.
pub fn generate_loot(w: &mut World, id: MobileId) {
    let Some(list) = props::mobile(w, id, "loot").and_then(Value::as_array) else {
        return (BASE_CREATURE_C.generate_loot)(w, id);
    };
    let mut packs = Vec::with_capacity(list.len());
    for e in list {
        let (name, n) = match e.as_array().map(Vec::as_slice) {
            Some([name, n]) => (name.as_str(), as_i32(Some(n))),
            _ => (None, None),
        };
        match (name.and_then(|s| loot_pack(w, s)), n) {
            (Some(p), Some(n)) => packs.push((p, n)),
            _ => tracing::warn!("Templates: a bad \"loot\" entry {e}"),
        }
    }
    for (p, n) in packs {
        add_loot_n(w, id, p, n);
    }
}
