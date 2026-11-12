//! The NPC speed table and the speed members of BaseCreature (GetSpeeds, SetSpeed, CurrentMoveSpeed...).

use muo_primitives::dotnet::InvariantCase;
use std::collections::HashMap;

use muo_data::expansion::Expansion;
use muo_primitives::utility;
use muo_server::ids::MobileId;
use muo_server::world::World;
use serde::Deserialize;

use super::data::{cd, cd_mut, read};
use super::enums::*;
use super::vtable::ctype;

/// `NPCSpeeds.SpeedClassEntry`.
#[derive(Clone, Debug, Default, Deserialize)]
pub struct SpeedClassEntry {
    pub level: String,
    pub active: f64,
    pub passive: f64,
    #[serde(rename = "activeMove", default)]
    pub active_move: f64,
    #[serde(rename = "passiveMove", default)]
    pub passive_move: f64,
    #[serde(default)]
    pub types: Vec<String>,
}

/// `NPCSpeeds`' statics.
#[derive(Default)]
pub struct NpcSpeeds {
    /// By `Type.Name` of the creature class, lower-cased (the JSON names classes by short name).
    by_type: HashMap<String, SpeedClassEntry>,
    by_level: HashMap<i32, SpeedClassEntry>,
    pub min_idle_seconds: i32,
    pub max_idle_seconds: i32,
}

fn level_of(name: &str) -> Option<SpeedLevel> {
    Some(match name {
        "None" => SpeedLevel::None,
        "VerySlow" => SpeedLevel::VerySlow,
        "Slow" => SpeedLevel::Slow,
        "Medium" => SpeedLevel::Medium,
        "Fast" => SpeedLevel::Fast,
        "VeryFast" => SpeedLevel::VeryFast,
        _ => return None,
    })
}

impl NpcSpeeds {
    /// `RegisterSpeed`.
    pub fn register(&mut self, entry: SpeedClassEntry) {
        if let Some(l) = level_of(&entry.level) {
            self.by_level.insert(l as i32, entry.clone());
        }
        for t in &entry.types {
            self.by_type.insert(t.lower_inv(), entry.clone());
        }
    }
}

/// `NPCSpeeds.Configure` (default priority): the idle seconds and `Data/npc-speeds.json`.
pub fn configure(w: &mut World) {
    let min = w.config.get_setting("movement.delay.npcMinIdle", 15);
    let max = w.config.get_setting("movement.delay.npcMaxIdle", 25);
    let path = w.core.base_directory.join("Data/npc-speeds.json");
    let table = w.resources.get_mut::<NpcSpeeds>();
    table.min_idle_seconds = min;
    table.max_idle_seconds = max;
    let Ok(text) = std::fs::read_to_string(&path) else {
        return;
    };
    match muo_data::json::from_str::<Vec<SpeedClassEntry>>(text.trim_start_matches('\u{feff}')) {
        Ok(entries) => {
            for e in entries {
                table.register(e);
            }
        }
        // `JsonConfig.Deserialize` throws out of `Configure`: startup stops
        Err(e) => {
            w.core.startup_error = Some(format!("Failed to deserialize Data/npc-speeds.json: {e}."))
        }
    }
}

/// `NPCSpeeds.FindEntry`: the SpeedClass bucket, else the class's own entry, else Medium.
pub fn find_entry(w: &World, id: MobileId) -> Option<SpeedClassEntry> {
    let t = w.resources.get::<NpcSpeeds>()?;
    let class = ctype(w, id).map_or(SpeedLevel::None, |c| (c.speed_class)(w, id));
    if class != SpeedLevel::None
        && let Some(e) = t.by_level.get(&(class as i32))
    {
        return Some(e.clone());
    }
    let name = super::type_name(w, id).lower_inv();
    t.by_type
        .get(&name)
        .or_else(|| t.by_level.get(&(SpeedLevel::Medium as i32)))
        .cloned()
}

/// GetSpeeds: (active, passive). Without a table entry the serialized speeds stand.
pub fn get_speeds(w: &World, id: MobileId) -> (f64, f64) {
    match find_entry(w, id) {
        Some(e) => (e.active, e.passive),
        None => read(w, id, (0.0, 0.0), |d| (d.active_speed, d.passive_speed)),
    }
}

/// GetMoveSpeeds: (active, passive); 0 inherits.
pub fn get_move_speeds(w: &World, id: MobileId) -> (f64, f64) {
    find_entry(w, id).map_or((0.0, 0.0), |e| (e.active_move, e.passive_move))
}

/// `GetSpeeds` through the class's virtual.
pub fn speeds_of(w: &World, id: MobileId) -> (f64, f64) {
    ctype(w, id).map_or((0.0, 0.0), |c| (c.get_speeds)(w, id))
}

pub fn move_speeds_of(w: &World, id: MobileId) -> (f64, f64) {
    ctype(w, id).map_or((0.0, 0.0), |c| (c.get_move_speeds)(w, id))
}

/// `CurrentSpeed = value` (`fieldChanged: OnCurrentSpeedChange` tells the AI).
pub fn set_current_speed(w: &mut World, id: MobileId, value: f64) {
    let Some(d) = cd_mut(w, id) else { return };
    let changed = d.current_speed != value;
    d.current_speed = value;
    if changed {
        crate::ai::on_current_speed_changed(w, id);
    }
}

/// `ActiveMoveSpeed`/`PassiveMoveSpeed` setters coerce to `Max(0, value)`.
pub fn set_active_move_speed(w: &mut World, id: MobileId, value: f64) {
    if let Some(d) = cd_mut(w, id) {
        d.active_move_speed = value.max(0.0);
    }
}

pub fn set_passive_move_speed(w: &mut World, id: MobileId, value: f64) {
    if let Some(d) = cd_mut(w, id) {
        d.passive_move_speed = value.max(0.0);
    }
}

/// SetSpeed(active, passive, isPassive): sets the think clock and clears the move overrides.
pub fn set_speed(w: &mut World, id: MobileId, active: f64, passive: f64, is_passive: bool) {
    if let Some(d) = cd_mut(w, id) {
        d.active_speed = active;
        d.passive_speed = passive;
        d.active_move_speed = 0.0;
        d.passive_move_speed = 0.0;
    }
    set_current_speed(w, id, if is_passive { passive } else { active });
}

/// SetMoveSpeed
pub fn set_move_speed(w: &mut World, id: MobileId, active: f64, passive: f64) {
    set_active_move_speed(w, id, active);
    set_passive_move_speed(w, id, passive);
}

/// ClearMoveSpeed
pub fn clear_move_speed(w: &mut World, id: MobileId) {
    if let Some(d) = cd_mut(w, id) {
        d.active_move_speed = 0.0;
        d.passive_move_speed = 0.0;
    }
}

/// ScaleMoveSpeed: inheriting values stay inheriting.
pub fn scale_move_speed(w: &mut World, id: MobileId, scalar: f64) {
    if let Some(d) = cd_mut(w, id) {
        if d.active_move_speed > 0.0 {
            d.active_move_speed *= scalar;
        }
        if d.passive_move_speed > 0.0 {
            d.passive_move_speed *= scalar;
        }
    }
}

/// SnapSpeedsToTable
pub fn snap_speeds_to_table(w: &mut World, id: MobileId) {
    let (a, p) = speeds_of(w, id);
    let (am, pm) = move_speeds_of(w, id);
    let Some(d) = cd_mut(w, id) else { return };
    if (d.active_speed - a).abs() < 0.0001 && (d.passive_speed - p).abs() < 0.0001 {
        d.active_speed = a;
        d.passive_speed = p;
    }
    if am > 0.0 && (d.active_move_speed - am).abs() < 0.0001 {
        d.active_move_speed = am;
    }
    if pm > 0.0 && (d.passive_move_speed - pm).abs() < 0.0001 {
        d.passive_move_speed = pm;
    }
}

/// SetCurrentSpeedToActive
pub fn set_current_speed_to_active(w: &mut World, id: MobileId) {
    if let Some(v) = cd(w, id).map(|d| d.active_speed) {
        set_current_speed(w, id, v);
    }
}

/// SetCurrentSpeedToPassive
pub fn set_current_speed_to_passive(w: &mut World, id: MobileId) {
    if let Some(v) = cd(w, id).map(|d| d.passive_speed) {
        set_current_speed(w, id, v);
    }
}

/// MigrateMoveSpeeds: pre-v22 saves have no movement clock; untuned creatures adopt the table's values.
pub fn migrate_move_speeds(w: &mut World, id: MobileId) {
    let (a, p) = speeds_of(w, id);
    let (am, pm) = move_speeds_of(w, id);
    if let Some(d) = cd_mut(w, id)
        && d.active_speed == a
        && d.passive_speed == p
    {
        d.active_move_speed = am;
        d.passive_move_speed = pm;
    }
}

/// CurrentMoveSpeed: resolved seconds per step.
pub fn current_move_speed(w: &World, id: MobileId) -> f64 {
    let Some(d) = cd(w, id) else { return 0.0 };
    if d.target_location.is_some() {
        return HERDING_MOVE_SPEED;
    }
    let speed = if d.current_speed == d.active_speed {
        if d.active_move_speed > 0.0 {
            d.active_move_speed
        } else {
            d.active_speed
        }
    } else if d.current_speed == d.passive_speed {
        if d.passive_move_speed > 0.0 {
            d.passive_move_speed
        } else {
            d.passive_speed
        }
    } else {
        d.current_speed
    };
    let follow = ctype(w, id).map_or(0.0, |c| (c.follow_move_speed)(w, id));
    if follow > 0.0 && crate::ai::is_pacing_to_master(w, id) {
        follow.min(speed)
    } else {
        speed
    }
}

/// `Utility.RandomMinMax(NPCSpeeds.MinIdleSeconds, MaxIdleSeconds)`.
pub fn random_idle_seconds(w: &mut World) -> i32 {
    let (min, max) = w
        .resources
        .get::<NpcSpeeds>()
        .map_or((0, 0), |t| (t.min_idle_seconds, t.max_idle_seconds));
    utility::random_min_max(&mut w.rng, min, max)
}

/// True from AOS on, for `FollowMoveSpeed`'s era check callers outside this module.
pub fn aos(w: &World) -> bool {
    w.expansion() >= Expansion::AOS
}
