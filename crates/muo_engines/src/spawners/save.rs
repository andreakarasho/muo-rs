//! The generated serializers of BaseSpawner (version 13; 10 to 12 and older load through migrations),
//! Spawner (2), RegionSpawner (0), ProximitySpawner (0) and SpawnerEntry (2; 1 and older), byte for byte.
//! Older versions are read by the `Deserialize(reader, version)` migrations of `BaseSpawner` and `SpawnerEntry`.

use muo_io::serialization::Guid;
use muo_io::{GenericReader, GenericWriter, Result};
use muo_primitives::{DateTime, Rectangle3D, TimeSpan};
use muo_server::ids::{EntityId, ItemId};
use muo_server::item::{ITEM_BASE, ops, serialize as iser};
use muo_server::world::World;

use super::core::{self, SPAWNER, kind_of_type};
use super::{
    DEFAULT_MAX_DELAY, DEFAULT_MAX_SPAWN_ATTEMPTS, DEFAULT_MIN_DELAY, Entry, Kind,
    SpawnPositionMode, SpawnerData, data, data_mut,
};

const BASE_VERSION: i32 = 13;
const SPAWNER_VERSION: i32 = 2;
const ENTRY_VERSION: i32 = 2;

// BaseSpawner.SaveFlag
const F_RETURN_ON_DEACTIVATE: i32 = 0x1;
const F_WAY_POINT: i32 = 0x2;
const F_GROUP: i32 = 0x4;
const F_MIN_DELAY: i32 = 0x8;
const F_MAX_DELAY: i32 = 0x10;
const F_TEAM: i32 = 0x20;
const F_SPAWN_LOCATION_IS_HOME: i32 = 0x40;
const F_END: i32 = 0x80;
const F_SPAWN_POSITION_MODE: i32 = 0x100;
const F_MAX_SPAWN_ATTEMPTS: i32 = 0x200;

fn write_entry(w: &World, out: &mut dyn GenericWriter, e: &Entry) {
    out.write_i32(ENTRY_VERSION);
    out.write_enum(4, i64::from(i32::from(e.disabled)));
    out.write_opt_string(e.name.as_deref());
    out.write_i32(e.probability);
    out.write_i32(e.max_count);
    out.write_opt_string(e.properties.as_deref());
    out.write_opt_string(e.parameters.as_deref());
    // `_spawned.Tidy()`: deleted entities are dropped
    let live: Vec<EntityId> = e
        .spawned
        .iter()
        .copied()
        .filter(|s| match s {
            EntityId::Item(i) => !ops::is_deleted(w, *i),
            EntityId::Mobile(m) => !muo_content::mx::deleted(w, *m),
        })
        .collect();
    out.write_encoded_int(live.len() as i32);
    for s in live {
        iser::write_entity(w, out, Some(s));
    }
}

fn read_entity_ref(w: &World, r: &mut dyn GenericReader) -> Result<Option<EntityId>> {
    iser::read_entity(w, r)
}

fn read_entry(w: &World, r: &mut dyn GenericReader) -> Result<Entry> {
    let version = r.read_i32()?;
    let mut e = Entry::default();
    if version >= ENTRY_VERSION {
        let flags = r.read_enum(4)? as i32;
        e.name = r.read_string()?;
        e.probability = r.read_i32()?;
        e.max_count = r.read_i32()?;
        e.properties = r.read_string()?;
        e.parameters = r.read_string()?;
        let n = r.read_encoded_int()?;
        for _ in 0..n.max(0) {
            if let Some(s) = read_entity_ref(w, r)? {
                e.spawned.push(s);
            }
        }
        e.disabled = flags & 1 != 0;
    } else {
        e.name = r.read_string()?;
        e.probability = r.read_i32()?;
        e.max_count = r.read_i32()?;
        e.properties = r.read_string()?;
        e.parameters = r.read_string()?;
        // version 1 (generated migration) counts with an encoded int, older saves with an int
        let n = if version == 1 {
            r.read_encoded_int()?
        } else {
            r.read_i32()?
        };
        for _ in 0..n.max(0) {
            if let Some(s) = read_entity_ref(w, r)? {
                e.spawned.push(s);
            }
        }
    }
    Ok(e)
}

pub fn serialize(w: &World, id: ItemId, out: &mut dyn GenericWriter) {
    (ITEM_BASE.serialize)(w, id, out);
    let Some(d) = data(w, id) else { return };

    out.write_i32(BASE_VERSION);
    let mut flags = 0;
    let set = |cond: bool, f: i32| if cond { f } else { 0 };
    flags |= set(d.return_on_deactivate, F_RETURN_ON_DEACTIVATE);
    flags |= set(d.way_point.is_some(), F_WAY_POINT);
    flags |= set(d.group, F_GROUP);
    flags |= set(d.min_delay != DEFAULT_MIN_DELAY, F_MIN_DELAY);
    flags |= set(d.max_delay != DEFAULT_MAX_DELAY, F_MAX_DELAY);
    flags |= set(d.team != 0, F_TEAM);
    flags |= set(d.spawn_location_is_home, F_SPAWN_LOCATION_IS_HOME);
    flags |= set(d.end != DateTime::MIN, F_END);
    flags |= set(
        !matches!(
            d.spawn_position_mode,
            SpawnPositionMode::Automatic | SpawnPositionMode::Abandoned
        ),
        F_SPAWN_POSITION_MODE,
    );
    flags |= set(
        d.max_spawn_attempts != DEFAULT_MAX_SPAWN_ATTEMPTS,
        F_MAX_SPAWN_ATTEMPTS,
    );
    out.write_enum(4, i64::from(flags));
    out.write_guid(d.guid);
    out.write_i32(d.walking_range);
    if let Some(wp) = d.way_point.filter(|_| flags & F_WAY_POINT != 0) {
        iser::write_entity(w, out, Some(EntityId::Item(wp)));
    }
    if flags & F_MIN_DELAY != 0 {
        out.write_time_span_ticks(d.min_delay.ticks());
    }
    if flags & F_MAX_DELAY != 0 {
        out.write_time_span_ticks(d.max_delay.ticks());
    }
    out.write_i32(d.count);
    if flags & F_TEAM != 0 {
        out.write_i32(d.team);
    }
    out.write_bool(d.running);
    if flags & F_END != 0 {
        out.write_date_time_ticks(d.end.ticks());
    }
    if flags & F_SPAWN_POSITION_MODE != 0 {
        out.write_enum(1, d.spawn_position_mode as i64);
    }
    if flags & F_MAX_SPAWN_ATTEMPTS != 0 {
        out.write_i32(d.max_spawn_attempts);
    }

    // Spawner
    out.write_encoded_int(SPAWNER_VERSION);
    let has_bounds = d.spawn_bounds != Rectangle3D::default();
    out.write_enum(
        4,
        i64::from(i32::from(d.use_spiral_scan) | (i32::from(has_bounds) << 1)),
    );
    if has_bounds {
        out.write_rect3d(d.spawn_bounds);
    }
    out.write_encoded_int(d.entries.len() as i32);
    for e in &d.entries {
        write_entry(w, out, e);
    }
}

/// Reads the entries of a pre-13 BaseSpawner section: `count` then each entry.
fn read_entries(w: &World, r: &mut dyn GenericReader, encoded_count: bool) -> Result<Vec<Entry>> {
    let n = if encoded_count {
        r.read_encoded_int()?
    } else {
        r.read_i32()?
    };
    let mut v = Vec::with_capacity(n.clamp(0, 4096) as usize);
    for _ in 0..n.max(0) {
        v.push(read_entry(w, r)?);
    }
    Ok(v)
}

fn read_waypoint(w: &World, r: &mut dyn GenericReader) -> Result<Option<ItemId>> {
    iser::read_item(w, r)
}

fn read_date_time(r: &mut dyn GenericReader) -> Result<DateTime> {
    Ok(DateTime::from_ticks(r.read_date_time_ticks()?))
}

pub fn deserialize(w: &mut World, id: ItemId, r: &mut dyn GenericReader) -> Result<()> {
    (ITEM_BASE.deserialize)(w, id, r)?;
    let Some(ty) = w.item(id).map(|i| i.ty) else {
        return Ok(());
    };
    let mut d = SpawnerData::new(kind_of_type(ty));
    let now = w.now;

    let version = r.read_i32()?;
    match version {
        10 => {
            d.guid = r.read_guid()?;
            d.return_on_deactivate = r.read_bool()?;
            d.entries = read_entries(w, r, true)?;
            d.walking_range = r.read_i32()?;
            d.way_point = read_waypoint(w, r)?;
            d.group = r.read_bool()?;
            d.min_delay = TimeSpan::from_ticks(r.read_time_span_ticks()?);
            d.max_delay = TimeSpan::from_ticks(r.read_time_span_ticks()?);
            d.count = r.read_i32()?;
            d.team = r.read_i32()?;
            let home_range = r.read_i32()?;
            d.running = r.read_bool()?;
            let end = read_date_time(r)?;
            d.end = if d.running { end } else { now };
            if home_range > 0 {
                d.pending_home_range = Some(home_range);
            }
            d.spawn_location_is_home = false;
            d.spawn_position_mode = SpawnPositionMode::Automatic;
            d.max_spawn_attempts = DEFAULT_MAX_SPAWN_ATTEMPTS;
        }
        11 => {
            d.guid = r.read_guid()?;
            d.return_on_deactivate = r.read_bool()?;
            d.entries = read_entries(w, r, true)?;
            d.walking_range = r.read_i32()?;
            d.way_point = read_waypoint(w, r)?;
            d.group = r.read_bool()?;
            d.min_delay = TimeSpan::from_ticks(r.read_time_span_ticks()?);
            d.max_delay = TimeSpan::from_ticks(r.read_time_span_ticks()?);
            d.count = r.read_i32()?;
            d.team = r.read_i32()?;
            d.spawn_bounds = r.read_rect3d()?;
            d.running = r.read_bool()?;
            d.spawn_location_is_home = r.read_bool()?;
            let end = read_date_time(r)?;
            d.end = if d.running { end } else { now };
            d.spawn_position_mode = SpawnPositionMode::Automatic;
            d.max_spawn_attempts = DEFAULT_MAX_SPAWN_ATTEMPTS;
        }
        12 => {
            let flags = r.read_enum(4)? as i32;
            d.guid = r.read_guid()?;
            d.return_on_deactivate = flags & F_RETURN_ON_DEACTIVATE != 0;
            d.entries = read_entries(w, r, true)?;
            d.walking_range = r.read_i32()?;
            if flags & F_WAY_POINT != 0 {
                d.way_point = read_waypoint(w, r)?;
            }
            d.group = flags & F_GROUP != 0;
            d.min_delay = if flags & F_MIN_DELAY != 0 {
                TimeSpan::from_ticks(r.read_time_span_ticks()?)
            } else {
                DEFAULT_MIN_DELAY
            };
            d.max_delay = if flags & F_MAX_DELAY != 0 {
                TimeSpan::from_ticks(r.read_time_span_ticks()?)
            } else {
                DEFAULT_MAX_DELAY
            };
            d.count = r.read_i32()?;
            d.team = if flags & F_TEAM != 0 {
                r.read_i32()?
            } else {
                0
            };
            d.running = r.read_bool()?;
            d.spawn_location_is_home = flags & F_SPAWN_LOCATION_IS_HOME != 0;
            let end = if flags & F_END != 0 {
                Some(read_date_time(r)?)
            } else {
                None
            };
            d.end = if d.running { end.unwrap_or(now) } else { now };
            d.spawn_position_mode = if flags & F_SPAWN_POSITION_MODE != 0 {
                SpawnPositionMode::from_u8(r.read_u8()?)
            } else {
                SpawnPositionMode::Automatic
            };
            d.max_spawn_attempts = if flags & F_MAX_SPAWN_ATTEMPTS != 0 {
                r.read_i32()?
            } else {
                DEFAULT_MAX_SPAWN_ATTEMPTS
            };
        }
        v if v < BASE_VERSION => {
            // the pre-codegen layout (`BaseSpawner` migration)
            d.guid = r.read_guid()?;
            d.return_on_deactivate = r.read_bool()?;
            d.entries = read_entries(w, r, false)?;
            d.walking_range = r.read_i32()?;
            d.way_point = read_waypoint(w, r)?;
            d.group = r.read_bool()?;
            d.min_delay = TimeSpan::from_ticks(r.read_time_span_ticks()?);
            d.max_delay = TimeSpan::from_ticks(r.read_time_span_ticks()?);
            d.count = r.read_i32()?;
            d.team = r.read_i32()?;
            let home_range = r.read_i32()?;
            if home_range > 0 {
                d.pending_home_range = Some(home_range);
            }
            d.running = r.read_bool()?;
            d.end = if d.running {
                DateTime::from_ticks(r.read_delta_time_ticks(now.ticks())?)
            } else {
                now
            };
            d.spawn_position_mode = SpawnPositionMode::Automatic;
            d.max_spawn_attempts = DEFAULT_MAX_SPAWN_ATTEMPTS;
        }
        _ => {
            let flags = r.read_enum(4)? as i32;
            d.guid = r.read_guid()?;
            d.return_on_deactivate = flags & F_RETURN_ON_DEACTIVATE != 0;
            d.walking_range = r.read_i32()?;
            if flags & F_WAY_POINT != 0 {
                d.way_point = read_waypoint(w, r)?;
            }
            d.group = flags & F_GROUP != 0;
            d.min_delay = if flags & F_MIN_DELAY != 0 {
                TimeSpan::from_ticks(r.read_time_span_ticks()?)
            } else {
                DEFAULT_MIN_DELAY
            };
            d.max_delay = if flags & F_MAX_DELAY != 0 {
                TimeSpan::from_ticks(r.read_time_span_ticks()?)
            } else {
                DEFAULT_MAX_DELAY
            };
            d.count = r.read_i32()?;
            if flags & F_TEAM != 0 {
                d.team = r.read_i32()?;
            }
            d.running = r.read_bool()?;
            d.spawn_location_is_home = flags & F_SPAWN_LOCATION_IS_HOME != 0;
            if flags & F_END != 0 {
                d.end = read_date_time(r)?;
            }
            if flags & F_SPAWN_POSITION_MODE != 0 {
                d.spawn_position_mode = SpawnPositionMode::from_u8(r.read_u8()?);
            }
            d.max_spawn_attempts = if flags & F_MAX_SPAWN_ATTEMPTS != 0 {
                r.read_i32()?
            } else {
                DEFAULT_MAX_SPAWN_ATTEMPTS
            };
        }
    }

    // BaseSpawner.AfterDeserialization: a pre-11 HomeRange becomes square bounds around the location
    if let Some(home_range) = d.pending_home_range.take() {
        let loc = w.item(id).map_or_else(Default::default, |i| i.location);
        d.spawn_bounds = Rectangle3D::new(
            loc.x - home_range,
            loc.y - home_range,
            -128,
            home_range * 2 + 1,
            home_range * 2 + 1,
            256,
        );
    }

    // Spawner section
    let sv = r.read_encoded_int()?;
    match sv {
        0 => {}
        1 => {
            let flags = r.read_enum(4)? as i32;
            d.use_spiral_scan = flags & 1 != 0;
            d.spawn_bounds = if flags & 2 != 0 {
                r.read_rect3d()?
            } else {
                Rectangle3D::default()
            };
        }
        _ => {
            let flags = r.read_enum(4)? as i32;
            d.use_spiral_scan = flags & 1 != 0;
            if flags & 2 != 0 {
                d.spawn_bounds = r.read_rect3d()?;
            }
            d.entries = read_entries(w, r, true)?;
        }
    }

    if let Some(i) = w.item_mut(id) {
        i.ext.insert(d);
    }
    after_deserialization(w, id);
    Ok(())
}

/// Entry `AfterDeserialization` (the spawn's `Spawner` is this spawner) and `RebuildSpawned()`.
fn after_deserialization(w: &mut World, id: ItemId) {
    let sid = super::spawner_id(id);
    let spawned: Vec<EntityId> = data(w, id)
        .map(|d| {
            d.entries
                .iter()
                .flat_map(|e| e.spawned.iter().copied())
                .collect()
        })
        .unwrap_or_default();
    for e in spawned {
        match e {
            EntityId::Item(i) => {
                if let Some(x) = w.item_mut(i) {
                    x.spawner = Some(sid);
                }
            }
            EntityId::Mobile(m) => {
                if let Some(x) = w.mobile_mut(m) {
                    x.spawner = Some(sid);
                }
            }
        }
    }
    core::rebuild_spawned(w, id);
}

// ---- RegionSpawner (0) and ProximitySpawner (0) ----

pub fn serialize_region(w: &World, id: ItemId, out: &mut dyn GenericWriter) {
    (SPAWNER.serialize)(w, id, out);
    out.write_encoded_int(0);
    out.write_opt_string(data(w, id).and_then(|d| d.spawn_region_name.as_deref()));
}

pub fn deserialize_region(w: &mut World, id: ItemId, r: &mut dyn GenericReader) -> Result<()> {
    (SPAWNER.deserialize)(w, id, r)?;
    let _version = r.read_encoded_int()?;
    let name = r.read_string()?;
    let map = w.item(id).and_then(|i| i.map);
    let region = match (&name, map) {
        (Some(n), Some(m)) => muo_server::region::find_by_name(w, n, m, true),
        _ => None,
    };
    if let Some(rid) = region {
        muo_content::regions::base_region::init_rectangles(w, rid);
    }
    if let Some(d) = data_mut(w, id) {
        d.spawn_region_name = name;
        d.spawn_region = region;
    }
    Ok(())
}

pub fn serialize_proximity(w: &World, id: ItemId, out: &mut dyn GenericWriter) {
    (SPAWNER.serialize)(w, id, out);
    out.write_encoded_int(0);
    if let Some(d) = data(w, id) {
        out.write_i32(d.trigger_range);
        out.write_text_definition(d.spawn_message.as_ref());
        out.write_bool(d.instant_flag);
    }
}

pub fn deserialize_proximity(w: &mut World, id: ItemId, r: &mut dyn GenericReader) -> Result<()> {
    (SPAWNER.deserialize)(w, id, r)?;
    let _version = r.read_encoded_int()?;
    let range = r.read_i32()?;
    let msg = r.read_text_definition()?;
    let instant = r.read_bool()?;
    if let Some(d) = data_mut(w, id) {
        d.trigger_range = range;
        d.spawn_message = msg;
        d.instant_flag = instant;
    }
    Ok(())
}

/// Used by tests and the importer: a [`Guid`] from text.
pub fn parse_guid(s: &str) -> Option<Guid> {
    let hex: String = s
        .chars()
        .filter(|c| *c != '-' && *c != '{' && *c != '}')
        .collect();
    if hex.len() != 32 {
        return None;
    }
    let mut b = [0u8; 16];
    for (i, byte) in b.iter_mut().enumerate() {
        *byte = u8::from_str_radix(hex.get(i * 2..i * 2 + 2)?, 16).ok()?;
    }
    Some(Guid(b))
}

#[allow(dead_code)]
fn _kind(_: Kind) {}
