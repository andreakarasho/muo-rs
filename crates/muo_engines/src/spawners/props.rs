//! The `[CommandProperty]` members of `BaseSpawner`, `Spawner`, `ProximitySpawner` and `RegionSpawner` for `[props`,
//! `[get`, `[set` and `where`. The Rust `Spawner` is `BaseSpawner` and `Spawner` in one class, so one table serves
//! both. Every member needs Developer access.
//!
//! `RegionSpawner.SpawnRegion` is read and written as the region's name.

use muo_content::ext_field;
use muo_content::props::{self, Kind, PropDef, Value};
use muo_content::text_def_prop;
use muo_content::utilities::types::{Numeric, PropType};
use muo_server::World;
use muo_server::ids::EntityId;
use muo_server::mobile::AccessLevel;

use super::{SpawnPositionMode, data, data_mut};

const DEV: AccessLevel = AccessLevel::Developer;
const INT: Kind = Kind::Plain(PropType::Numeric(Numeric::Int));
const BOOL: Kind = Kind::Plain(PropType::Bool);

const POSITION_MODES: &[(&str, i64)] = &[
    ("Automatic", 0),
    ("Enabled", 1),
    ("Disabled", 2),
    ("Abandoned", 3),
];

fn item(e: EntityId) -> Option<muo_server::ids::ItemId> {
    match e {
        EntityId::Item(i) => Some(i),
        EntityId::Mobile(_) => None,
    }
}

/// `data(w, id)` as the `ext_field!` getter wants it.
fn sd(w: &World, i: muo_server::ids::ItemId) -> Option<&super::SpawnerData> {
    data(w, i)
}

fn sd_mut(w: &mut World, i: muo_server::ids::ItemId) -> Option<&mut super::SpawnerData> {
    data_mut(w, i)
}

/// `Guid.Parse` for the formats a person types: `D` (8-4-4-4-12), `N` (32 digits), with or without `{}` or `()`.
fn parse_guid(text: &str) -> Option<muo_io::serialization::Guid> {
    let t = text.trim();
    let t = t
        .strip_prefix('{')
        .and_then(|x| x.strip_suffix('}'))
        .or_else(|| t.strip_prefix('(').and_then(|x| x.strip_suffix(')')))
        .unwrap_or(t);
    let digits: String = t.chars().filter(|c| *c != '-').collect();
    let dashes = t.len() - digits.len();
    if digits.len() != 32
        || !(dashes == 0
            || (dashes == 4
                && [8, 13, 18, 23]
                    .iter()
                    .all(|i| t.as_bytes().get(*i) == Some(&b'-'))))
    {
        return None;
    }
    let mut b = [0u8; 16];
    for (i, o) in b.iter_mut().enumerate() {
        *o = u8::from_str_radix(digits.get(2 * i..2 * i + 2)?, 16).ok()?;
    }
    Some(muo_io::serialization::Guid(b))
}

/// What `BaseSpawner` declares (the Rust `Spawner` is both classes; the gump lists the two groups apart, see `props::class_chain`).
pub static BASE_SPAWNER: &[PropDef] = &[
    PropDef {
        name: "Guid",
        kind: Kind::Plain(PropType::Str),
        read: DEV,
        write: DEV,
        get: |w, e| {
            item(e)
                .and_then(|i| data(w, i))
                .map_or(Value::Null, |d| Value::Text(d.guid.to_string()))
        },
        set: Some(|w, e, v| {
            let i = item(e).ok_or_else(|| "That is not an item.".to_string())?;
            let g = v
                .string()
                .and_then(|s| parse_guid(&s))
                .ok_or_else(|| "That is not properly formatted.".to_string())?;
            if let Some(d) = data_mut(w, i) {
                d.guid = g;
            }
            Ok(())
        }),
    },
    ext_field!(
        sd,
        sd_mut,
        "ReturnOnDeactivate",
        BOOL,
        DEV,
        DEV,
        |d| Value::Bool(d.return_on_deactivate),
        |d, v| d.return_on_deactivate = v.bool(),
        false
    ),
    PropDef {
        name: "WayPoint",
        kind: Kind::Plain(PropType::Entity),
        read: DEV,
        write: DEV,
        get: |w, e| {
            Value::Entity(
                item(e)
                    .and_then(|i| data(w, i))
                    .and_then(|d| d.way_point)
                    .map(EntityId::Item),
            )
        },
        set: Some(|w, e, v| {
            let Some(i) = item(e) else {
                return Err("That is not an item.".to_string());
            };
            // a `WayPoint` only; anything else is a type mismatch
            let wp = match v.entity() {
                Some(EntityId::Item(p))
                    if w.item(p)
                        .is_some_and(|x| x.ty.full_name == "Server.Items.WayPoint") =>
                {
                    Some(p)
                }
                None if v.is_null() => None,
                _ => return Err("not a WayPoint".to_string()),
            };
            if let Some(d) = data_mut(w, i) {
                d.way_point = wp;
            }
            Ok(())
        }),
    },
    ext_field!(
        sd,
        sd_mut,
        "Group",
        BOOL,
        DEV,
        DEV,
        |d| Value::Bool(d.group),
        |d, v| d.group = v.bool(),
        true
    ),
    ext_field!(
        sd,
        sd_mut,
        "MinDelay",
        Kind::TimeSpan,
        DEV,
        DEV,
        |d| Value::TimeSpan(d.min_delay),
        |d, v| if let Value::TimeSpan(t) = v {
            d.min_delay = *t
        },
        true
    ),
    ext_field!(
        sd,
        sd_mut,
        "MaxDelay",
        Kind::TimeSpan,
        DEV,
        DEV,
        |d| Value::TimeSpan(d.max_delay),
        |d, v| if let Value::TimeSpan(t) = v {
            d.max_delay = *t
        },
        true
    ),
    ext_field!(
        sd,
        sd_mut,
        "Team",
        INT,
        DEV,
        DEV,
        |d| Value::Int(i64::from(d.team)),
        |d, v| d.team = v.i32(),
        true
    ),
    PropDef {
        name: "SpawnBounds",
        kind: Kind::Rect3D,
        read: DEV,
        write: DEV,
        get: |w, e| {
            item(e)
                .and_then(|i| data(w, i))
                .map_or(Value::Null, |d| Value::Rect3(d.spawn_bounds))
        },
        set: Some(|w, e, v| {
            if let (Some(i), Value::Rect3(r)) = (item(e), v) {
                super::core::set_spawn_bounds(w, i, r);
            }
            Ok(())
        }),
    },
    ext_field!(
        sd,
        sd_mut,
        "SpawnLocationIsHome",
        BOOL,
        DEV,
        DEV,
        |d| Value::Bool(d.spawn_location_is_home),
        |d, v| d.spawn_location_is_home = v.bool(),
        true
    ),
    ext_field!(
        sd,
        sd_mut,
        "End",
        Kind::DateTime,
        DEV,
        DEV,
        |d| Value::DateTime(d.end),
        |d, v| if let Value::DateTime(t) = v {
            d.end = *t
        },
        false
    ),
    ext_field!(
        sd,
        sd_mut,
        "SpawnPositionMode",
        Kind::Plain(PropType::Enum(POSITION_MODES)),
        DEV,
        DEV,
        |d| Value::Enum(d.spawn_position_mode as i64),
        |d, v| d.spawn_position_mode = SpawnPositionMode::from_u8(v.i32() as u8),
        false
    ),
    ext_field!(
        sd,
        sd_mut,
        "MaxSpawnAttempts",
        INT,
        DEV,
        DEV,
        |d| Value::Int(i64::from(d.max_spawn_attempts)),
        |d, v| d.max_spawn_attempts = v.i32(),
        false
    ),
    PropDef {
        name: "HomeRange",
        kind: INT,
        read: DEV,
        write: DEV,
        get: |w, e| {
            item(e)
                .filter(|i| data(w, *i).is_some())
                .map_or(Value::Null, |i| {
                    Value::Int(i64::from(super::core::home_range(w, i)))
                })
        },
        set: Some(|w, e, v| {
            if let Some(i) = item(e) {
                super::core::set_home_range(w, i, v.i32());
            }
            Ok(())
        }),
    },
    PropDef {
        name: "WalkingRange",
        kind: INT,
        read: DEV,
        write: DEV,
        get: |w, e| {
            item(e)
                .filter(|i| data(w, *i).is_some())
                .map_or(Value::Null, |i| {
                    Value::Int(i64::from(super::core::walking_range(w, i)))
                })
        },
        set: Some(|w, e, v| {
            if let Some(i) = item(e) {
                super::core::set_walking_range(w, i, v.i32());
            }
            Ok(())
        }),
    },
    PropDef {
        name: "Count",
        kind: INT,
        read: DEV,
        write: DEV,
        get: |w, e| {
            item(e)
                .and_then(|i| data(w, i))
                .map_or(Value::Null, |d| Value::Int(i64::from(d.count)))
        },
        set: Some(|w, e, v| {
            if let Some(i) = item(e) {
                super::core::set_count(w, i, v.i32());
            }
            Ok(())
        }),
    },
    PropDef {
        name: "Running",
        kind: BOOL,
        read: DEV,
        write: DEV,
        get: |w, e| {
            item(e)
                .and_then(|i| data(w, i))
                .map_or(Value::Null, |d| Value::Bool(d.running))
        },
        set: Some(|w, e, v| {
            if let Some(i) = item(e) {
                super::core::set_running(w, i, v.bool());
            }
            Ok(())
        }),
    },
    PropDef {
        name: "NextSpawn",
        kind: Kind::TimeSpan,
        read: DEV,
        write: DEV,
        get: |w, e| {
            let Some(d) = item(e).and_then(|i| data(w, i)) else {
                return Value::Null;
            };
            let running = d.running && d.timer.is_some_and(|t| t.running(w));
            Value::TimeSpan(if running {
                d.end - w.now
            } else {
                muo_primitives::TimeSpan::ZERO
            })
        },
        set: Some(|w, e, v| {
            if let (Some(i), Value::TimeSpan(t)) = (item(e), v) {
                super::core::set_next_spawn(w, i, t);
            }
            Ok(())
        }),
    },
];

/// What `Spawner` itself declares.
pub static SPAWNER: &[PropDef] = &[ext_field!(
    sd,
    sd_mut,
    "UseSpiralScan",
    BOOL,
    DEV,
    DEV,
    |d| Value::Bool(d.use_spiral_scan),
    |d, v| d.use_spiral_scan = v.bool(),
    false
)];

pub static PROXIMITY: &[PropDef] = &[
    ext_field!(
        sd,
        sd_mut,
        "TriggerRange",
        INT,
        DEV,
        DEV,
        |d| Value::Int(i64::from(d.trigger_range)),
        |d, v| d.trigger_range = v.i32(),
        false
    ),
    text_def_prop!(
        "SpawnMessage",
        DEV,
        DEV,
        |w: &World, e: EntityId| item(e)
            .and_then(|i| data(w, i))
            .and_then(|d| d.spawn_message.clone()),
        Some(|w: &mut World, e: EntityId, v: Value| {
            if let Some(i) = item(e)
                && let Some(d) = data_mut(w, i)
            {
                d.spawn_message = match v {
                    Value::TextDef(t) => Some(t),
                    _ => None,
                };
            }
            Ok(())
        })
    ),
    ext_field!(
        sd,
        sd_mut,
        "InstantFlag",
        BOOL,
        DEV,
        DEV,
        |d| Value::Bool(d.instant_flag),
        |d, v| d.instant_flag = v.bool(),
        false
    ),
];

/// `RegionSpawner.SpawnRegion`: the region by its name. There is no parser for a `Region`, so the text is the name
/// (`Region.Find(name, map, true)` on the spawner's map, as the JSON loader does); the setter is
/// `set_spawn_region` (`SpawnRegionName`, `InitRectangles()`, `InvalidateProperties()`); `null` clears it.
pub static REGION: &[PropDef] = &[PropDef {
    name: "SpawnRegion",
    kind: Kind::Plain(PropType::Str),
    read: DEV,
    write: DEV,
    get: |w, e| match item(e).and_then(|i| data(w, i)) {
        Some(d) => match d.spawn_region.and_then(|r| w.region(r)) {
            Some(r) => Value::Str(r.name.clone().unwrap_or_default()),
            None => Value::Null,
        },
        None => Value::Null,
    },
    set: Some(|w, e, v| {
        let i = item(e).ok_or_else(|| "That is not an item.".to_string())?;
        let region = match v.string().filter(|s| s != "null") {
            None => None,
            Some(name) => {
                let map = w
                    .item(i)
                    .and_then(|x| x.map)
                    .ok_or_else(|| "The spawner is not on a map.".to_string())?;
                Some(
                    muo_server::region::find_by_name(w, &name, map, true)
                        .ok_or_else(|| format!("No region named {name} on that map."))?,
                )
            }
        };
        super::core::set_spawn_region(w, i, region);
        Ok(())
    }),
}];

/// Adds the tables (the `Configure` phase).
pub fn configure(w: &mut World) {
    props::add(w, "Server.Engines.Spawners.BaseSpawner", BASE_SPAWNER);
    props::add(w, "Server.Engines.Spawners.Spawner", SPAWNER);
    props::add(w, "Server.Engines.Spawners.ProximitySpawner", PROXIMITY);
    props::add(w, "Server.Engines.Spawners.RegionSpawner", REGION);
}
