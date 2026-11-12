//! The engine classes for `[Decorate` (`Decorate.Construct()`): the Haven `Cannon`,
//! `MaabusCoffin` and every `BaseSpawner`. `muo_items::decorate_glue` has the item classes.

use muo_commands::hooks::{CommandHooks, enum_parse, parse_error};
use muo_primitives::parse::{to_bool, to_i32};
use muo_primitives::{Point3D, TimeSpan};
use muo_server::World;
use muo_server::ids::ItemId;
use muo_server::item::ops;
use muo_server::type_registry::{Phase, TypeRegistry};

use crate::quests::dark_tides_mobs::CoffinData;
use crate::quests::uzeraan_mobs::new_cannon;
use crate::spawners::{self, core};

fn val<'a>(p: &'a str, key: &str) -> Option<&'a str> {
    if p.starts_with(key) {
        p.find('=').map(|i| &p[i + 1..])
    } else {
        None
    }
}

/// `TimeSpan.Parse(s)`: throws on bad text.
fn timespan(s: &str) -> Result<TimeSpan, String> {
    TimeSpan::try_parse(s).ok_or_else(|| parse_error(s))
}

fn construct(
    w: &mut World,
    full: &str,
    _item_id: i32,
    ps: &[String],
) -> Option<Result<ItemId, String>> {
    if full != "Server.Engines.Quests.Haven.Cannon" && full != "Server.Items.Cannon" {
        return None;
    }
    let mut direction = 0; // CannonDirection.North
    for p in ps {
        if let Some(v) = val(p, "CannonDirection") {
            // `(CannonDirection)Enum.Parse(.., true)`: North, East, South, West.
            direction = match enum_parse(&["North", "East", "South", "West"], v) {
                Ok(d) => d,
                Err(e) => return Some(Err(e)),
            };
        }
    }
    Some(Ok(new_cannon(w, direction)))
}

fn apply(
    w: &mut World,
    item: ItemId,
    _item_id: i32,
    ps: &[String],
    mag: bool,
) -> Result<bool, String> {
    if crate::quests::dark_tides_mobs::coffin_is(w, item) {
        for p in ps {
            if let Some(v) = val(p, "SpawnLocation") {
                // `Point3D.Parse` throws on bad text
                let t = Point3D::try_parse(v).ok_or_else(|| parse_error(v))?;
                if let Some(i) = w.item_mut(item)
                    && let Some(d) = i.ext.get_mut::<CoffinData>()
                {
                    d.spawn_location = t;
                }
            }
        }
        return Ok(true);
    }
    if spawners::data(w, item).is_none() {
        return Ok(false);
    }
    core::set_next_spawn(w, item, TimeSpan::ZERO);
    for p in ps {
        if let Some(v) = val(p, "Spawn") {
            core::add_entry(w, item, Some(v), 100, 1, true, None, None);
        } else if let Some(v) = val(p, "MinDelay") {
            if let Some(d) = spawners::data_mut(w, item) {
                d.min_delay = timespan(v)?;
            }
        } else if let Some(v) = val(p, "MaxDelay") {
            if let Some(d) = spawners::data_mut(w, item) {
                d.max_delay = timespan(v)?;
            }
        } else if let Some(v) = val(p, "NextSpawn") {
            core::set_next_spawn(w, item, timespan(v)?);
        } else if let Some(v) = val(p, "Count") {
            let n = to_i32(v);
            core::set_count(w, item, n);
            if let Some(d) = spawners::data_mut(w, item) {
                for e in &mut d.entries {
                    e.max_count = n;
                }
            }
        } else if let Some(v) = val(p, "Team") {
            if let Some(d) = spawners::data_mut(w, item) {
                d.team = to_i32(v);
            }
        } else if let Some(v) = val(p, "HomeRange") {
            core::set_home_range(w, item, to_i32(v));
        } else if !mag && let Some(v) = val(p, "WalkingRange") {
            core::set_walking_range(w, item, to_i32(v));
        } else if let Some(v) = val(p, "Running") {
            core::set_running(w, item, to_bool(v));
        } else if let Some(v) = val(p, "Group")
            && let Some(d) = spawners::data_mut(w, item)
        {
            d.group = to_bool(v);
        }
    }
    ops::invalidate_properties(w, item);
    Ok(true)
}

fn configure(w: &mut World) {
    let h = w.resources.get_mut::<CommandHooks>();
    h.decorate_construct.push(construct);
    h.decorate_apply.push(apply);
}

pub fn register(types: &mut TypeRegistry) {
    types.register_phase_default(
        Phase::Configure,
        "Server.Engines.DecorateGlue.Configure",
        configure,
    );
}
