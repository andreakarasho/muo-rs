//! `PathfindRecorder` and the `[PathRecord` command: appends one
//! JSONL record per pathfind (start, goal, map, capability flags) to `pathfinding.recorder.path`, for replaying it in
//! a benchmark. There is no step cache (the A* is uncached), so the cache commands (`PathBake`, `PathCacheStats`, ..)
//! and `PathDiag` do not exist.

use std::fs::OpenOptions;
use std::io::{BufWriter, Write};

use muo_primitives::Point3D;
use muo_server::commands::{CommandEventArgs, CommandSystem};
use muo_server::ids::MobileId;
use muo_server::map::MapId;
use muo_server::mobile::AccessLevel;
use muo_server::mobile::part2::messages::send_message;
use muo_server::world::World;

/// The recorder's statics.
#[derive(Default)]
pub struct PathfindRecorder {
    writer: Option<BufWriter<std::fs::File>>,
    pub enabled: bool,
    pub output_path: String,
    pub records_written: i64,
}

/// `PathfindRecorder.Configure` and `PathCacheCommands.Configure`'s `[PathRecord`.
pub fn configure(w: &mut World) {
    let default = w
        .core
        .base_directory
        .join("Data")
        .join("Pathfinding")
        .join("recordings")
        .join("pathfinds.jsonl")
        .to_string_lossy()
        .into_owned();
    let path = w
        .config
        .get_or_update_setting("pathfinding.recorder.path", default.clone())
        .unwrap_or(default);
    w.resources.get_mut::<PathfindRecorder>().output_path = path;
    if w.config
        .get_or_update_setting("pathfinding.recorder.enable", false)
        .unwrap_or(false)
    {
        set_enabled(w, true);
    }
    CommandSystem::register(
        w,
        "PathRecord",
        AccessLevel::Administrator,
        &[],
        std::rc::Rc::new(on_path_record),
    );
}

/// `SetEnabled(enabled)`: opens the file for append on enable, flushes and closes it on disable.
pub fn set_enabled(w: &mut World, enabled: bool) {
    let r = w.resources.get_mut::<PathfindRecorder>();
    if enabled == r.enabled {
        return;
    }
    if enabled {
        let opened = std::path::Path::new(&r.output_path)
            .parent()
            .map_or(Ok(()), std::fs::create_dir_all)
            .and_then(|()| {
                OpenOptions::new()
                    .append(true)
                    .create(true)
                    .open(&r.output_path)
            });
        match opened {
            Ok(f) => {
                r.writer = Some(BufWriter::new(f));
                r.enabled = true;
                tracing::info!("PathfindRecorder enabled, writing to {}", r.output_path);
            }
            Err(e) => {
                tracing::warn!(
                    "PathfindRecorder: failed to open {} for write: {e}",
                    r.output_path
                );
                r.writer = None;
                r.enabled = false;
            }
        }
    } else {
        r.enabled = false;
        if let Some(mut wr) = r.writer.take()
            && let Err(e) = wr.flush()
        {
            tracing::warn!("PathfindRecorder: error closing {}: {e}", r.output_path);
        }
        tracing::info!(
            "PathfindRecorder disabled ({} records this session)",
            r.records_written
        );
    }
}

fn flush(w: &mut World) {
    let r = w.resources.get_mut::<PathfindRecorder>();
    if let Some(wr) = r.writer.as_mut()
        && let Err(e) = wr.flush()
    {
        tracing::warn!("PathfindRecorder: flush failed for {}: {e}", r.output_path);
    }
}

/// `RecordIfEnabled(m, map, start, goal)`: one bool check when disabled.
pub fn record_if_enabled(w: &mut World, m: MobileId, map: MapId, start: Point3D, goal: Point3D) {
    if !w
        .resources
        .get::<PathfindRecorder>()
        .is_some_and(|r| r.enabled)
    {
        return;
    }
    let (mut swim, mut fly, mut doors, mut over) = (false, false, false, false);
    if crate::creature::is_creature(w, m)
        && let Some(c) = crate::creature::vtable::ctype(w, m)
    {
        swim = w.mobile(m).is_some_and(|x| x.can_swim);
        fly = (c.can_fly)(w, m);
        doors = (c.can_open_doors)(w, m);
        over = (c.can_move_over_obstacles)(w, m);
    }
    let line = format!(
        "{{\"Name\":\"recorded\",\"MapId\":{},\"StartX\":{},\"StartY\":{},\"StartZ\":{},\"GoalX\":{},\"GoalY\":{},\"GoalZ\":{},\"CanSwim\":{swim},\"CanFly\":{fly},\"CanOpenDoors\":{doors},\"CanMoveOverObstacles\":{over}}}\n",
        map.0, start.x, start.y, start.z, goal.x, goal.y, goal.z
    );
    let r = w.resources.get_mut::<PathfindRecorder>();
    let Some(wr) = r.writer.as_mut() else { return };
    match wr.write_all(line.as_bytes()) {
        Ok(()) => r.records_written += 1,
        Err(e) => {
            tracing::warn!("PathfindRecorder: write failed, disabling: {e}");
            set_enabled(w, false);
        }
    }
}

/// `[Usage("PathRecord [on|off|flush|status]")]`
fn on_path_record(w: &mut World, e: &mut CommandEventArgs) {
    let arg = e
        .arguments
        .first()
        .map_or_else(|| "status".to_owned(), |a| a.to_lowercase());
    let from = e.mobile;
    match arg.as_str() {
        "on" => {
            set_enabled(w, true);
            let r = w.resources.get_mut::<PathfindRecorder>();
            let text = if r.enabled {
                format!("PathRecord: ON, writing to {}", r.output_path)
            } else {
                "PathRecord: enable failed (see server log)".to_owned()
            };
            send_message(w, from, &text);
        }
        "off" => {
            set_enabled(w, false);
            send_message(w, from, "PathRecord: OFF");
        }
        "flush" => {
            flush(w);
            let n = w.resources.get_mut::<PathfindRecorder>().records_written;
            send_message(
                w,
                from,
                &format!("PathRecord: flushed ({n} records this session)"),
            );
        }
        _ => {
            let r = w.resources.get_mut::<PathfindRecorder>();
            let text = format!(
                "PathRecord: {}, path={}, records={}",
                if r.enabled { "ON" } else { "OFF" },
                r.output_path,
                r.records_written
            );
            send_message(w, from, &text);
        }
    }
}
