//! The load and save flow of the world (`Configure`, `Load`, `Save`, `Snapshot`, the staged
//! save publishing, `FinishWorldSave`, `Broadcast`) and of persistence (`Load`,
//! `SerializeAll`, `PostWorldSaveAll`, `PostDeserializeAll`).
//!
//! Save flow, without worker heaps:
//! 1. `save`: `Running` -> `PendingSave`, picks a temp folder and asks the loop for a snapshot.
//! 2. `snapshot` (on the loop thread, from `LoopHooks::world_snapshot`): `Saving`; every persistence composes
//!    its files in memory, biggest estimated payload first; the `WorldSave` event fires; `WritingSave`.
//! 3. the `SaveWorker` writes the files to the temp folder and stages them at `<Saves>.next`; then, back on the
//!    loop, `WorldSavePostSnapshot` fires (archivers), the previous save is set aside, the staged one renamed to
//!    `Saves`, and `finish_world_save` returns to `Running` and applies the pending add/delete queues.
//!
//! shortcut: the publish step runs on the loop thread because event handlers are not `Send`; it is renames, so it
//! is short unless a `WorldSavePostSnapshot` handler (an archiver) is slow. Upgrade by running the handlers on
//! the worker if that proves a stall.
//! shortcut: no `File.SetLastWriteTimeUtc` on the published folder (std cannot set a directory's time).

use std::cmp::Reverse;
use std::path::{Path, PathBuf};
use std::time::Instant;

use muo_io::serialization::persistence::load_types_db;
use muo_primitives::{DateTime, Serial};

use crate::core_loop::EventLoopContext;
use crate::events::{self, WorldSavePostSnapshotEventArgs};
use crate::mobile::AccessLevel;
use crate::packets::message::{MessageType, create_message};
use crate::persistence::{LoadContext, LoadPrompt, PersistError, Snapshot};
use crate::world::{World, WorldState};

impl World {
    /// `World.Configure()`: `world.tempSavePath` (default `temp`) and `world.savePath` (default `Saves`, written
    /// back), made absolute against the base directory.
    pub fn configure_paths(&mut self) {
        let base = self.core.base_directory.clone();
        let temp = self
            .config
            .get_setting("world.tempSavePath", "temp".to_string());
        self.temp_save_path = full_path(&temp, &base);
        let save = self
            .config
            .get_or_update_setting("world.savePath", "Saves".to_string())
            .unwrap_or_else(|_| "Saves".to_string());
        self.save_path = full_path(&save, &base);
        // `UseMultiThreadedSaves`: written back; saves here always use the worker thread
        let _ = self
            .config
            .get_or_update_setting("world.useMultithreadedSaves", true);
    }

    /// `World.WaitForWriteCompletion()`.
    pub fn wait_for_write_completion(&self) {
        self.save_worker.wait_idle();
    }

    /// `World.StagedSavePath`: a complete snapshot is staged here before the previous save is touched.
    pub fn staged_save_path(&self) -> PathBuf {
        let mut s = self.save_path.clone().into_os_string();
        s.push(".next");
        PathBuf::from(s)
    }

    /// `World.Load()`. `Err` is a fatal load error (a bad type answered "n", "Deserialization failed.").
    pub fn load(&mut self, prompt: &mut dyn LoadPrompt) -> Result<(), PersistError> {
        if self.state != WorldState::Initial {
            return Ok(());
        }
        self.set_state(WorldState::Loading);

        tracing::info!("Loading world");
        let watch = Instant::now();

        self.recover_staged_save()?;

        self.persistence_load(prompt)?;
        events::invoke_world_load(self, &mut ());

        // Set the world to running before we process our queues
        self.set_state(WorldState::Running);

        self.post_deserialize_all(); // Process safety queues

        tracing::info!(
            "Loading world done ({} items, {} mobiles) ({:.2} seconds)",
            self.items.len(),
            self.mobiles.len(),
            watch.elapsed().as_secs_f64()
        );
        Ok(())
    }

    /// `Persistence.Load(path)`: the types db, every entity persistence's indexes, then every payload.
    fn persistence_load(&mut self, prompt: &mut dyn LoadPrompt) -> Result<(), PersistError> {
        let save_path = self.save_path.clone();
        let types_db = load_types_db(&save_path)?;
        let mut registry = std::mem::take(&mut self.persistence);
        let mut ctx = LoadContext {
            save_path: &save_path,
            types_db: &types_db,
            prompt,
        };
        let mut result = Ok(());
        for p in registry.iter_mut() {
            result = p.deserialize_indexes(self, &mut ctx);
            if result.is_err() {
                break;
            }
        }
        // This should probably not be parallel since Mobiles must be loaded before Items
        if result.is_ok() {
            for p in registry.iter_mut() {
                result = p.deserialize(self, &mut ctx);
                if result.is_err() {
                    break;
                }
            }
        }
        self.persistence = registry;
        result
    }

    /// `Persistence.PostDeserializeAll()`.
    pub fn post_deserialize_all(&mut self) {
        let mut registry = std::mem::take(&mut self.persistence);
        for p in registry.iter_mut() {
            p.post_deserialize(self);
        }
        self.persistence = registry;
    }

    /// `Persistence.SerializeAll()`: composes every persistence's files now. Largest known payloads first (LPT
    /// scheduling; for one thread only the order of the result changes).
    pub fn serialize_all(&mut self) -> Result<Vec<Snapshot>, PersistError> {
        let mut registry = std::mem::take(&mut self.persistence);
        let mut order: Vec<_> = registry.iter_mut().collect();
        order.sort_by_key(|p| Reverse(p.estimated_size()));
        let mut out = Vec::with_capacity(order.len());
        let mut result = Ok(());
        for p in order {
            match p.serialize(self) {
                Ok(s) => out.push(s),
                Err(e) => {
                    result = Err(e);
                    break;
                }
            }
        }
        self.persistence = registry;
        result.map(|()| out)
    }

    /// `Persistence.PostWorldSaveAll()`.
    pub fn post_world_save_all(&mut self) {
        let mut registry = std::mem::take(&mut self.persistence);
        for p in registry.iter_mut() {
            p.post_world_save(self);
        }
        self.persistence = registry;
    }

    /// Composes the whole world and writes it to `dir` (no staging, no events, no state change). What a save's
    /// middle does, for tools and the golden test.
    pub fn write_snapshot_to(&mut self, dir: &Path) -> Result<(), PersistError> {
        self.save_start_time = self.now;
        for s in self.serialize_all()? {
            s.write(dir)?;
        }
        Ok(())
    }

    /// `World.Save()`: starts a save; the snapshot itself happens when the loop gets to it.
    pub fn save(&mut self) {
        if self.state != WorldState::Running {
            return;
        }

        self.wait_for_write_completion(); // Blocks Save until current disk flush is done.
        self.set_state(WorldState::PendingSave);

        // Preserialize
        let prepared = self
            .recover_staged_save()
            .map_err(|e| e.to_string())
            .and_then(|()| ensure_random_path(&self.temp_save_path).map_err(|e| e.to_string()));
        match prepared {
            Ok(path) => self.core.request_snapshot(path),
            Err(e) => {
                tracing::error!("Preparing to save world failed: {e}");
                trace_exception(self.now, &e);
                self.broadcast_staff(
                    0x35,
                    true,
                    "Preparing for a world save failed! Check the logs!",
                );
            }
        }
    }

    /// `World.Snapshot(snapshotPath)`.
    pub fn snapshot(&mut self, snapshot_path: &Path) {
        if self.state != WorldState::PendingSave {
            return;
        }

        self.flush_net();

        self.set_state(WorldState::Saving);

        // The world is frozen from here: one anchor for the whole save. Written into save indexes so anchored
        // timestamps can be re-based by the downtime at load.
        self.save_start_time = self.now;

        self.broadcast(0x35, true, "The world is saving, please wait.");

        tracing::info!("Saving world");
        let watch = Instant::now();

        let result = if snapshot_path.as_os_str().is_empty() {
            Err(PersistError::Failed(
                "Snapshot path cannot be null or empty (Parameter 'snapshotPath')".into(),
            ))
        } else {
            self.serialize_all()
        };

        let snapshots = match result {
            Ok(s) => {
                // A WorldSave handler failing is logged and the save goes on.
                let ran = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    events::invoke_world_save(self, &mut ());
                }));
                if ran.is_err() {
                    tracing::error!("A WorldSave handler failed");
                }
                Some(s)
            }
            Err(e) => {
                tracing::error!("Saving world failed: {e}");
                trace_exception(self.now, &e.to_string());
                None
            }
        };

        let Some(snapshots) = snapshots else {
            self.broadcast_staff(0x35, true, "World save failed! Check the logs!");
            finish_world_save(self);
            return;
        };

        self.set_state(WorldState::WritingSave);
        let secs = watch.elapsed().as_secs_f64();
        let ctx = self.core.loop_context.clone();
        let (dir, staging) = (snapshot_path.to_path_buf(), self.staged_save_path());
        self.save_worker.submit(move || {
            write_files(&ctx, snapshots, &dir, &staging);
            Ok(())
        });

        tracing::info!("Saving world done ({secs:.2} seconds)");
        self.broadcast(
            0x35,
            true,
            &format!("World save completed in {secs:.2} seconds."),
        );
    }

    /// `PublishStagedSave(archive)`: lets subscribers archive the previous save, sets it aside, renames the
    /// staged save into place.
    fn publish_staged_save(&mut self, archive: bool) -> std::io::Result<()> {
        let staging = self.staged_save_path();

        if archive {
            let mut args = WorldSavePostSnapshotEventArgs {
                old_save_path: self.save_path.clone(),
                new_save_path: staging.clone(),
            };
            events::invoke_world_save_post_snapshot(self, &mut args);
        }

        if self.save_path.exists() {
            set_aside(&self.save_path, "previous", self.now)?;
        }

        move_directory(&staging, &self.save_path)
    }

    /// `World.RecoverStagedSave()`: finishes an interrupted publish. Runs at boot (before load) and before every
    /// save; whatever is at `Saves/` is set aside, never deleted.
    pub fn recover_staged_save(&mut self) -> Result<(), PersistError> {
        let staging = self.staged_save_path();
        if !staging.exists() {
            return Ok(());
        }
        tracing::warn!(
            "A complete world save was staged at {} but never published; publishing it now.",
            staging.display()
        );
        self.publish_staged_save(false)
            .map_err(|e| PersistError::Io(e.into()))
    }

    /// `NetState.FlushAll()`.
    fn flush_net(&mut self) {
        // shortcut: the transport flushes when `net_glue::pump` runs (which `w.send` queues for), so a save that
        // forces the flush first sends the same bytes in the same order; there is no earlier flush to do.
    }

    /// `World.Broadcast(hue, ascii, text)`: to every connection that has a mobile.
    pub fn broadcast(&mut self, hue: i32, ascii: bool, text: &str) {
        self.broadcast_where(hue, ascii, text, |_, _| true);
    }

    /// `World.BroadcastStaff(hue, ascii, text)`: to game masters and above.
    pub fn broadcast_staff(&mut self, hue: i32, ascii: bool, text: &str) {
        self.broadcast_where(hue, ascii, text, |w, m| {
            w.mobile(m)
                .is_some_and(|m| m.access_level >= AccessLevel::GameMaster)
        });
    }

    fn broadcast_where(
        &mut self,
        hue: i32,
        ascii: bool,
        text: &str,
        allow: impl Fn(&World, crate::ids::MobileId) -> bool,
    ) {
        let bytes = create_message(
            Serial::MINUS_ONE,
            -1,
            MessageType::REGULAR,
            hue,
            3,
            ascii,
            Some("ENU"),
            "System",
            text,
        );
        let targets: Vec<_> = self
            .net_states
            .iter()
            .filter_map(|(ns, d)| d.mobile.filter(|m| allow(self, *m)).map(|_| *ns))
            .collect();
        for ns in targets {
            self.send(ns, &bytes);
        }
    }
}

/// `World.FinishWorldSave()`.
pub fn finish_world_save(w: &mut World) {
    w.set_state(WorldState::Running);
    w.post_world_save_all(); // Process decay and safety queues
    // MovementThrottle.ResetAllMovementTiming(): prevents post-save movement rejection bursts
    let now = w.tick_count;
    let alive: Vec<_> = w
        .net_states
        .iter()
        .filter(|(_, d)| {
            d.mobile
                .is_some_and(|m| w.mobile(m).is_some_and(|m| !m.deleted))
        })
        .map(|(ns, _)| *ns)
        .collect();
    let throttle = std::mem::take(&mut w.movement_throttle);
    throttle.reset_all_movement_timing(
        w.net_states
            .iter_mut()
            .filter(|(ns, _)| alive.contains(ns))
            .map(|(_, d)| &mut d.movement),
        now,
    );
    w.movement_throttle = throttle;
}

/// `WriteFiles` on the worker: writes the snapshot, stages it, then has the loop publish it.
fn write_files(ctx: &EventLoopContext, snapshots: Vec<Snapshot>, dir: &Path, staging: &Path) {
    let watch = Instant::now();
    tracing::info!("Writing world save snapshot");
    let staged = (|| -> std::io::Result<()> {
        std::fs::create_dir_all(dir)?;
        for s in &snapshots {
            s.write(dir).map_err(std::io::Error::other)?;
        }
        if staging.exists() {
            set_aside(staging, "unpublished", DateTime::utc_now())?;
        }
        move_directory(dir, staging)
    })();

    match staged {
        Ok(()) => {
            tracing::info!(
                "Writing world save snapshot done ({:.2} seconds)",
                watch.elapsed().as_secs_f64()
            );
            ctx.post(|w| {
                if let Err(e) = w.publish_staged_save(true) {
                    publish_failed(w, &e.to_string());
                }
                finish_world_save(w);
            });
        }
        Err(e) => {
            let message = e.to_string();
            ctx.post(move |w| {
                publish_failed(w, &message);
                finish_world_save(w);
            });
        }
    }
}

fn publish_failed(w: &mut World, message: &str) {
    tracing::error!("Writing world save snapshot failed: {message}");
    trace_exception(w.now, message);
    w.broadcast_staff(
        0x35,
        true,
        "Writing world save snapshot failed! Check the logs!",
    );
}

/// `Persistence.TraceException`: appends to `save-errors.log`.
fn trace_exception(now: DateTime, message: &str) {
    use std::io::Write;
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("save-errors.log")
    {
        let _ = writeln!(f, "# {}\n{message}\n\n", format_date(now));
    }
    eprintln!("{message}");
}

/// `yyyy-MM-dd-HH-mm-ss-fff`.
fn format_date(t: DateTime) -> String {
    let (y, mo, d) = t.ymd();
    format!(
        "{y:04}-{mo:02}-{d:02}-{:02}-{:02}-{:02}-{:03}",
        t.hour(),
        t.minute(),
        t.second(),
        t.millisecond()
    )
}

/// `SetAside`: moves `path` to `<path>.<reason>-<timestamp>`.
fn set_aside(path: &Path, reason: &str, now: DateTime) -> std::io::Result<()> {
    let mut aside = path.to_path_buf().into_os_string();
    aside.push(format!(".{reason}-{}", format_date(now)));
    let aside = PathBuf::from(aside);
    move_directory(path, &aside)?;
    tracing::warn!(
        "Set aside {} as {}; delete or archive it by hand.",
        path.display(),
        aside.display()
    );
    Ok(())
}

/// `MoveDirectory`: atomic rename on one volume, file-by-file move otherwise.
fn move_directory(source: &Path, destination: &Path) -> std::io::Result<()> {
    match std::fs::rename(source, destination) {
        Ok(()) => Ok(()),
        Err(e) => {
            if destination.exists() {
                return Err(e);
            }
            move_directory_contents(source, destination)
        }
    }
}

/// `PathUtility.MoveDirectoryContents`.
fn move_directory_contents(source: &Path, dest: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dest)?;
    for entry in std::fs::read_dir(source)? {
        let entry = entry?;
        let target = dest.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            move_directory_contents(&entry.path(), &target)?;
        } else if std::fs::rename(entry.path(), &target).is_err() {
            std::fs::copy(entry.path(), &target)?;
            std::fs::remove_file(entry.path())?;
        }
    }
    let _ = std::fs::remove_dir_all(source);
    Ok(())
}

/// `PathUtility.GetFullPath(path, basePath)`.
pub fn full_path(path: &str, base: &Path) -> PathBuf {
    if path.is_empty() {
        base.to_path_buf()
    } else if Path::new(path).is_absolute() {
        PathBuf::from(path)
    } else {
        base.join(path)
    }
}

/// `PathUtility.EnsureRandomPath`: `<base>/<16 hex digits>`, created.
fn ensure_random_path(base: &Path) -> std::io::Result<PathBuf> {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos() as u64);
    let n = nanos.rotate_left(17).wrapping_mul(0x9E37_79B9_7F4A_7C15)
        ^ COUNTER.fetch_add(1, Ordering::Relaxed)
        ^ u64::from(std::process::id());
    let path = base.join(format!("{n:016x}"));
    std::fs::create_dir_all(&path)?;
    Ok(path)
}

#[cfg(test)]
mod staged_publish_tests {
    //! Staged save publish tests: a complete snapshot is staged next
    //! to Saves/ before the previous save is touched, and an interrupted publish is finished at the next boot.
    use super::*;
    use std::fs;

    struct Root(PathBuf);
    impl Drop for Root {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn setup(name: &str) -> (World, Root) {
        let root = std::env::temp_dir().join(format!("muo-staged-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let mut w = World::new();
        w.save_path = root.join("Saves");
        (w, Root(root))
    }

    fn write_marker(dir: &Path, name: &str) {
        fs::create_dir_all(dir).unwrap();
        fs::write(dir.join("marker.txt"), name).unwrap();
    }

    fn read_marker(dir: &Path) -> String {
        fs::read_to_string(dir.join("marker.txt")).unwrap()
    }

    fn dirs(root: &Path) -> Vec<PathBuf> {
        fs::read_dir(root)
            .unwrap()
            .map(|e| e.unwrap().path())
            .filter(|p| p.is_dir())
            .collect()
    }

    #[test]
    fn nothing_staged_recovery_is_a_no_op() {
        let (mut w, root) = setup("noop");
        write_marker(&w.save_path, "current");

        w.recover_staged_save().unwrap();

        assert_eq!(read_marker(&w.save_path), "current");
        assert_eq!(dirs(&root.0).len(), 1);
    }

    #[test]
    fn staged_save_replaces_saves_and_keeps_the_previous_one() {
        let (mut w, root) = setup("replace");
        write_marker(&w.save_path, "old");
        write_marker(&w.staged_save_path(), "new");

        w.recover_staged_save().unwrap();

        assert_eq!(read_marker(&w.save_path), "new");
        assert!(!w.staged_save_path().exists());

        let aside: Vec<_> = dirs(&root.0)
            .into_iter()
            .filter(|d| d.to_string_lossy().contains(".previous-"))
            .collect();
        assert_eq!(aside.len(), 1);
        assert_eq!(read_marker(&aside[0]), "old");
    }

    #[test]
    fn staged_save_with_no_saves_is_published() {
        let (mut w, root) = setup("fresh");
        write_marker(&w.staged_save_path(), "new");

        w.recover_staged_save().unwrap();

        assert_eq!(read_marker(&w.save_path), "new");
        assert_eq!(dirs(&root.0).len(), 1);
    }
}
