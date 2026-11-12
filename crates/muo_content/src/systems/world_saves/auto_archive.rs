//! AutoArchive: rolls the automatic backups up into hourly, daily and monthly
//! `.tar.zst` archives, prunes old ones, and restores a save from a backup or archive when none exists.
//!
//! The statics are one [`AutoArchive`] resource (`w.resources`) over an `Arc` that the `ThreadPool` work item (a
//! `std::thread`) shares. Times are compared in UTC (`Core.Now`).
//! shortcut: console colors (`Utility.PushColor`) are not printed, the menu text is the same.
//! shortcut: when the restore menu cannot read the console (headless), the server would stop with a
//! `HeadlessConsoleInputException`; `Configure` has no error channel here, so it logs the error and starts fresh.

use muo_primitives::dotnet::InvariantCase;
use std::cmp::Reverse;
use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, Instant};

use indexmap::IndexMap;
use muo_primitives::DateTime;
use muo_primitives::path_utility;
use muo_server::World;
use muo_server::events::WorldSavePostSnapshotEventArgs;
use muo_server::persistence::{LoadPrompt, PersistError};

use super::archive::{
    ArchiveCompletedEventArgs, ArchiveDestination, ArchiveDestinationRegistry,
    ArchiveFailedEventArgs, ArchivePeriod, LocalArchiveDestination,
};
use super::archive_journal::ArchiveJournal;
use crate::compression::managed_archive;

const BACKUP_COMPLETE_MARKER: &str = ".backup-complete";

fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

/// What `Configure` read; fixed afterwards.
struct Config {
    temp_archive_path: PathBuf,
    backup_path: PathBuf,
    automatic_backup_path: PathBuf,
    archive_path: PathBuf,
    save_path: PathBuf,
    compression_level: i32,
    verify_archives: bool,
    retry_count: i32,
    retry_delay_ms: i32,
    backup_max_age_days: i32,
}

#[derive(Copy, Clone)]
struct Schedule {
    next_hourly: DateTime,
    next_daily: DateTime,
    next_monthly: DateTime,
}

type Completed = Box<dyn Fn(&ArchiveCompletedEventArgs) + Send + Sync>;
type Failed = Box<dyn Fn(&ArchiveFailedEventArgs) + Send + Sync>;

struct Shared {
    cfg: Config,
    /// `Archive` is set (archiving locally) and `Prune` is set (`enableArchivePruning`).
    archive_enabled: bool,
    prune_enabled: bool,
    schedule: Mutex<Schedule>,
    is_archiving: AtomicBool,
    journal: Mutex<ArchiveJournal>,
    destinations: Mutex<ArchiveDestinationRegistry>,
    on_completed: Mutex<Vec<Completed>>,
    on_failed: Mutex<Vec<Failed>>,
}

/// The `AutoArchive` statics. `Default` is not configured: every call does nothing until `configure` ran.
#[derive(Default, Clone)]
pub struct AutoArchive(Option<Arc<Shared>>);

fn now() -> DateTime {
    DateTime::utc_now()
}

impl AutoArchive {
    fn s(&self) -> Option<&Arc<Shared>> {
        self.0.as_ref()
    }

    pub fn archive_path(&self) -> Option<&Path> {
        self.s().map(|s| s.cfg.archive_path.as_path())
    }

    pub fn backup_path(&self) -> Option<&Path> {
        self.s().map(|s| s.cfg.backup_path.as_path())
    }

    pub fn automatic_backup_path(&self) -> Option<&Path> {
        self.s().map(|s| s.cfg.automatic_backup_path.as_path())
    }

    pub fn next_hourly_archive(&self) -> DateTime {
        self.s()
            .map_or(DateTime::MIN, |s| lock(&s.schedule).next_hourly)
    }

    pub fn next_daily_archive(&self) -> DateTime {
        self.s()
            .map_or(DateTime::MIN, |s| lock(&s.schedule).next_daily)
    }

    pub fn next_monthly_archive(&self) -> DateTime {
        self.s()
            .map_or(DateTime::MIN, |s| lock(&s.schedule).next_monthly)
    }

    /// `ArchiveDestinationRegistry.Register`.
    pub fn register_destination(&self, d: Arc<dyn ArchiveDestination>) {
        if let Some(s) = self.s() {
            lock(&s.destinations).register(d);
        }
    }

    /// `ArchiveDestinationRegistry.Destinations`, copied.
    pub fn destinations(&self) -> Vec<Arc<dyn ArchiveDestination>> {
        self.s()
            .map(|s| lock(&s.destinations).destinations().to_vec())
            .unwrap_or_default()
    }

    /// `ArchiveJournal.Operations`, copied.
    pub fn journal_operations(&self) -> Vec<super::archive_journal::ArchiveJournalEntry> {
        self.s()
            .map(|s| lock(&s.journal).operations().to_vec())
            .unwrap_or_default()
    }

    /// `ArchiveCompleted +=`.
    pub fn on_archive_completed(
        &self,
        f: impl Fn(&ArchiveCompletedEventArgs) + Send + Sync + 'static,
    ) {
        if let Some(s) = self.s() {
            lock(&s.on_completed).push(Box::new(f));
        }
    }

    /// `ArchiveFailed +=`.
    pub fn on_archive_failed(&self, f: impl Fn(&ArchiveFailedEventArgs) + Send + Sync + 'static) {
        if let Some(s) = self.s() {
            lock(&s.on_failed).push(Box::new(f));
        }
    }
}

/// `AutoArchive.Configure()`.
pub fn configure(w: &mut World) {
    let mut prompt = muo_server::core_loop::default_prompt();
    configure_with(w, &mut prompt);
}

/// `Configure` with the console given, for tests.
pub fn configure_with(w: &mut World, prompt: &mut dyn LoadPrompt) {
    let base = w.core.base_directory.clone();
    let get = |w: &World, key: &str, default: &str| {
        path_utility::get_full_path(&w.config.get_setting(key, default.to_string()), &base)
    };
    let temp_archive_path = get(w, "autoArchive.tempArchivePath", "temp");

    let mut upd =
        |key: &str, d: String| w.config.get_or_update_setting(key, d.clone()).unwrap_or(d);
    let backup_path =
        path_utility::get_full_path(&upd("autoArchive.backupPath", "Backups".into()), &base);
    let automatic_backup_path = backup_path.join("Automatic");
    let archive_path =
        path_utility::get_full_path(&upd("autoArchive.archivePath", "Archives".into()), &base);

    let mut upd_b = |key: &str, d: bool| w.config.get_or_update_setting(key, d).unwrap_or(d);
    let use_local_archives = upd_b("autoArchive.archiveLocally", true);
    let enable_pruning = upd_b("autoArchive.enableArchivePruning", true);
    let verify_archives = upd_b("autoArchive.verifyArchives", true);

    let mut upd_i = |key: &str, d: i32| w.config.get_or_update_setting(key, d).unwrap_or(d);
    let compression_level = upd_i("autoArchive.compressionLevel", 3);
    let retry_count = upd_i("autoArchive.retryCount", 3);
    let retry_delay_ms = upd_i("autoArchive.retryDelayMs", 500);
    let backup_max_age_days = upd_i("autoArchive.backupMaxAge", 30);
    let hourly = upd_i("autoArchive.hourlyRetention", 24);
    let daily = upd_i("autoArchive.dailyRetention", 30);
    let monthly = upd_i("autoArchive.monthlyRetention", 12);

    let save_path = base.join(w.config.get_setting("world.savePath", "Saves".to_string()));

    let mut registry = ArchiveDestinationRegistry::default();
    if use_local_archives {
        registry.register(Arc::new(LocalArchiveDestination::new(
            hourly, daily, monthly,
        )));
    }

    // Initialize journal and recover interrupted operations
    let mut journal = ArchiveJournal::configure(&archive_path);
    journal.recover_interrupted(w.now);

    let n = w.now;
    let date = n.date();
    let shared = Arc::new(Shared {
        cfg: Config {
            temp_archive_path,
            backup_path,
            automatic_backup_path,
            archive_path,
            save_path,
            compression_level,
            verify_archives,
            retry_count,
            retry_delay_ms,
            backup_max_age_days,
        },
        archive_enabled: use_local_archives,
        prune_enabled: enable_pruning,
        schedule: Mutex::new(Schedule {
            next_hourly: date.add_hours(f64::from(n.hour())),
            next_daily: date,
            next_monthly: date.add_days(f64::from(1 - n.day() as i32)),
        }),
        is_archiving: AtomicBool::new(false),
        journal: Mutex::new(journal),
        destinations: Mutex::new(registry),
        on_completed: Mutex::new(Vec::new()),
        on_failed: Mutex::new(Vec::new()),
    });

    // Restores an archive file placed in the Saves folder
    restore_from_archive(&shared);

    // If no valid save data exists, offer to restore from backups/archives
    if !has_valid_save_data(&shared)
        && let Err(e) = prompt_restore_from_backup(&shared, prompt)
    {
        tracing::error!("{e}");
    }

    *w.resources.get_mut::<AutoArchive>() = AutoArchive(Some(shared));

    // Do a local archive rollup & prune
    auto_archive_locally(w);
}

/// `AutoArchive.Initialize()`: moves the replaced save to the backups after every world save.
pub fn initialize(w: &mut World) {
    w.events
        .world_save_post_snapshot
        .subscribe(|w, args| backup(w, args));
}

fn backup(w: &mut World, args: &WorldSavePostSnapshotEventArgs) {
    let Some(s) = w.resources.get_mut::<AutoArchive>().0.clone() else {
        return;
    };
    if !args.old_save_path.is_dir() {
        return;
    }
    let run = || -> io::Result<PathBuf> {
        fs::create_dir_all(&s.cfg.automatic_backup_path)?;
        let backup_path = s.cfg.automatic_backup_path.join(w.now.to_timestamp());
        path_utility::move_directory_contents(&args.old_save_path, &backup_path, true)?;
        // Write a marker file so Rollup knows this backup is complete and not mid-write.
        // This prevents archiving a partially-moved directory if Rollup runs concurrently.
        fs::write(backup_path.join(BACKUP_COMPLETE_MARKER), [])?;
        Ok(backup_path)
    };
    match run() {
        Ok(p) => tracing::info!("Created backup at {}", p.display()),
        Err(e) => {
            tracing::error!("Backup failed: {e}");
            return;
        }
    }
    if s.archive_enabled {
        auto_archive_locally(w);
    }
}

fn restore_from_archive(s: &Shared) {
    if !s.cfg.save_path.is_dir() {
        return;
    }
    for file in paths_by_timestamp_name(&s.cfg.save_path, true) {
        if restore_from_file(s, &file) {
            let _ = fs::remove_file(&file);
            break;
        }
    }
}

fn file_name(p: &Path) -> String {
    p.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default()
}

fn restore_from_file(s: &Shared, file: &Path) -> bool {
    let name = file_name(file);
    let stem = name.split('.').next().unwrap_or("");
    if try_get_date(stem).is_none() {
        return false;
    }
    tracing::info!("Restoring world save from archive {name}");
    let Ok(temp) = ensure_random_path(&s.cfg.temp_archive_path) else {
        return false;
    };
    let mut prompt = muo_server::core_loop::default_prompt();
    let ok = extract_archive(file, &temp)
        && prompt_and_restore_from_extracted_archive(&temp, &s.cfg.save_path, &mut prompt);
    cleanup_temp_directory(&temp);
    ok
}

fn ensure_random_path(base: &Path) -> io::Result<PathBuf> {
    let mut rng = muo_primitives::random::BuiltInRng::default();
    path_utility::ensure_random_path(base, base, &mut rng)
}

/// True when the Saves folder has `SerializedTypes.db` or a `.bin`/`.idx` file in a subfolder.
fn has_valid_save_data(s: &Shared) -> bool {
    let save_path = &s.cfg.save_path;
    if !save_path.is_dir() {
        return false;
    }
    if save_path.join("SerializedTypes.db").exists() {
        return true;
    }
    let Ok(dirs) = fs::read_dir(save_path) else {
        return false;
    };
    for dir in dirs.flatten() {
        if !dir.path().is_dir() {
            continue;
        }
        let Ok(files) = fs::read_dir(dir.path()) else {
            continue;
        };
        for f in files.flatten() {
            let n = file_name(&f.path());
            if n.ends_with(".bin") || n.ends_with(".idx") {
                return true;
            }
        }
    }
    false
}

enum RestorePoint {
    Backup(PathBuf),
    Archive(PathBuf),
}

fn prompt_restore_from_backup(s: &Shared, prompt: &mut dyn LoadPrompt) -> Result<(), PersistError> {
    // Collect available restore points: backup directories and archive files, newest first.
    let mut points: BTreeMap<Reverse<DateTime>, (RestorePoint, String)> = BTreeMap::new();

    if let Ok(dirs) = fs::read_dir(&s.cfg.automatic_backup_path) {
        for dir in dirs.flatten().map(|d| d.path()).filter(|d| d.is_dir()) {
            if !dir.join(BACKUP_COMPLETE_MARKER).exists() {
                continue;
            }
            let name = file_name(&dir);
            if let Some(date) = try_get_date(&name) {
                points.insert(
                    Reverse(date),
                    (RestorePoint::Backup(dir), format!("Backup: {name}")),
                );
            }
        }
    }

    if let Ok(period_dirs) = fs::read_dir(&s.cfg.archive_path) {
        for period_dir in period_dirs
            .flatten()
            .map(|d| d.path())
            .filter(|d| d.is_dir())
        {
            let period_name = file_name(&period_dir);
            let Ok(files) = fs::read_dir(&period_dir) else {
                continue;
            };
            for file in files.flatten().map(|f| f.path()) {
                let name = file_name(&file);
                let stem = match name.find('.') {
                    Some(i) if i > 0 => &name[..i],
                    _ => &name,
                };
                if let Some(date) = try_get_date(stem) {
                    points.insert(
                        Reverse(date),
                        (
                            RestorePoint::Archive(file.clone()),
                            format!("{period_name} archive: {name}"),
                        ),
                    );
                }
            }
        }
    }

    if points.is_empty() {
        return Ok(());
    }

    let entries: Vec<(DateTime, RestorePoint, String)> =
        points.into_iter().map(|(d, (p, l))| (d.0, p, l)).collect();

    // Interactive paginated menu
    const PAGE_SIZE: usize = 20;
    let mut page = 0;
    let total_pages = entries.len().div_ceil(PAGE_SIZE);
    loop {
        prompt.write_line("");
        prompt.write_line("No world save data found.");
        let mut head = format!("Available backups and archives ({} total", entries.len());
        if total_pages > 1 {
            head.push_str(&format!(", page {}/{}", page + 1, total_pages));
        }
        head.push_str("):");
        prompt.write_line(&head);
        prompt.write_line("");

        let start = page * PAGE_SIZE;
        let end = (start + PAGE_SIZE).min(entries.len());
        for (i, (date, _, label)) in entries.iter().enumerate().take(end).skip(start) {
            prompt.write_line(&format!("  {:>3}) {}  {label}", i + 1, format_date(*date)));
        }

        prompt.write_line("");
        let mut ask = String::from("Enter number to restore");
        if page + 1 < total_pages {
            ask.push_str(", [n]ext page");
        }
        if page > 0 {
            ask.push_str(", [p]rev page");
        }
        ask.push_str(", or Enter to start fresh: ");
        prompt.write(&ask);

        let input = prompt.read_line()?;
        if input.trim().is_empty() {
            prompt.write_line("Starting with no world save data.");
            return Ok(());
        }
        if input.eq_ignore_ascii_case("n") && page + 1 < total_pages {
            page += 1;
            continue;
        }
        if input.eq_ignore_ascii_case("p") && page > 0 {
            page -= 1;
            continue;
        }
        if let Some(choice) = input
            .trim()
            .parse::<usize>()
            .ok()
            .filter(|c| (1..=entries.len()).contains(c))
        {
            let (_, point, label) = &entries[choice - 1];
            prompt.write(&format!("Restoring {label}"));
            prompt.write_line("...");
            match point {
                RestorePoint::Archive(f) => restore_from_archive_file(s, f, prompt),
                RestorePoint::Backup(d) => {
                    if let Err(e) = restore_from_backup_directory(d, &s.cfg.save_path) {
                        tracing::error!("Restore failed: {e}");
                    }
                }
            }
            return Ok(());
        }
        prompt.write_line("Invalid selection. Try again.");
    }
}

fn format_date(d: DateTime) -> String {
    let (y, mo, da) = d.ymd();
    format!(
        "{y:04}-{mo:02}-{da:02} {:02}:{:02}:{:02}",
        d.hour(),
        d.minute(),
        d.second()
    )
}

fn restore_from_archive_file(s: &Shared, archive: &Path, prompt: &mut dyn LoadPrompt) {
    let Ok(temp) = ensure_random_path(&s.cfg.temp_archive_path) else {
        return;
    };
    if extract_archive(archive, &temp) {
        prompt_and_restore_from_extracted_archive(&temp, &s.cfg.save_path, prompt);
    } else {
        prompt.write_line(&format!("Failed to extract {}.", file_name(archive)));
    }
    cleanup_temp_directory(&temp);
}

fn reset_dir(save_path: &Path) -> io::Result<()> {
    if save_path.exists() {
        fs::remove_dir_all(save_path)?;
    }
    fs::create_dir_all(save_path)
}

fn restore_from_backup_directory(backup_dir: &Path, save_path: &Path) -> io::Result<()> {
    reset_dir(save_path)?;
    path_utility::copy_directory_contents(backup_dir, save_path, true)?;
    tracing::info!("Restored world save from backup {}", file_name(backup_dir));
    Ok(())
}

/// `.tar.zst`, `.tar.gz` or `.tar`.
fn extract_archive(archive: &Path, out: &Path) -> bool {
    let name = file_name(archive);
    if name.ends_with(".tar.zst") {
        managed_archive::extract_tar_zstd(archive, out)
    } else if name.ends_with(".tar.gz") {
        managed_archive::extract_tar_gz(archive, out)
    } else if name.ends_with(".tar") {
        managed_archive::extract_tar(archive, out)
    } else {
        tracing::warn!("Unsupported archive format: {name}");
        false
    }
}

/// Lists the backup directories inside an extracted archive and lets the user choose; one backup is restored
/// directly.
fn prompt_and_restore_from_extracted_archive(
    extracted: &Path,
    save_path: &Path,
    prompt: &mut dyn LoadPrompt,
) -> bool {
    let mut backups: Vec<(DateTime, PathBuf, String)> = Vec::new();
    if let Ok(dirs) = fs::read_dir(extracted) {
        for dir in dirs.flatten().map(|d| d.path()).filter(|d| d.is_dir()) {
            let name = file_name(&dir);
            if let Some(date) = try_get_date(&name) {
                backups.push((date, dir, name));
            }
        }
    }
    if backups.is_empty() {
        prompt.write_line("No valid backup directories found inside the archive.");
        return false;
    }
    // Sort newest first
    backups.sort_by_key(|b| Reverse(b.0));

    let chosen = if backups.len() == 1 {
        0
    } else {
        // Multiple backups, let the user choose
        prompt.write_line("");
        prompt.write_line(&format!("Archive contains {} backups:", backups.len()));
        prompt.write_line("");
        for (i, b) in backups.iter().enumerate() {
            prompt.write_line(&format!("  {:>3}) {}  {}", i + 1, format_date(b.0), b.2));
        }
        prompt.write_line("");
        prompt.write("[1]> ");
        let input = prompt.read_line().unwrap_or_default();
        if input.trim().is_empty() {
            0 // Default to most recent
        } else {
            match input.trim().parse::<usize>() {
                Ok(c) if (1..=backups.len()).contains(&c) => c - 1,
                _ => {
                    prompt.write_line("Invalid selection, using most recent.");
                    0
                }
            }
        }
    };
    tracing::info!("Restoring backup {}", backups[chosen].2);
    let r = reset_dir(save_path)
        .and_then(|()| path_utility::move_directory_contents(&backups[chosen].1, save_path, true));
    if let Err(e) = r {
        tracing::error!("Restore failed: {e}");
        return false;
    }
    true
}

fn cleanup_temp_directory(temp: &Path) {
    if temp.is_dir()
        && let Err(e) = fs::remove_dir_all(temp)
    {
        tracing::warn!("Failed to clean up temp directory {}: {e}", temp.display());
    }
}

/// `AutoArchive.PruneBackups()`.
pub fn prune_backups(w: &mut World) -> io::Result<()> {
    let Some(s) = w.resources.get_mut::<AutoArchive>().0.clone() else {
        return Ok(());
    };
    prune(&s, w.now)
}

fn prune(s: &Shared, now: DateTime) -> io::Result<()> {
    if s.cfg.archive_path.is_dir() {
        // Get retention counts from registered destinations
        for p in [
            ArchivePeriod::Hourly,
            ArchivePeriod::Daily,
            ArchivePeriod::Monthly,
        ] {
            prune_local_archives(s, p, max_retention(s, p))?;
        }
    }
    if !s.cfg.automatic_backup_path.is_dir() {
        return Ok(());
    }
    let threshold = now.add_days(-f64::from(s.cfg.backup_max_age_days));
    for folder in fs::read_dir(&s.cfg.automatic_backup_path)?
        .flatten()
        .map(|d| d.path())
    {
        if !folder.is_dir() {
            continue;
        }
        // Skip backup directories that are still being written
        if !folder.join(BACKUP_COMPLETE_MARKER).exists() {
            continue;
        }
        let Some(date) = try_get_date(&file_name(&folder)) else {
            continue;
        };
        if date < threshold {
            tracing::info!("Pruning old backup {}", folder.display());
            retry_file_operation(s, || fs::remove_dir_all(&folder))?;
        }
    }
    Ok(())
}

fn max_retention(s: &Shared, period: ArchivePeriod) -> i32 {
    lock(&s.destinations)
        .destinations()
        .iter()
        .map(|d| d.get_retention_count(period))
        .fold(0, i32::max)
}

fn prune_local_archives(
    s: &Shared,
    period: ArchivePeriod,
    mut min_retained: i32,
) -> io::Result<()> {
    let dir = s.cfg.archive_path.join(period.name());
    if !dir.is_dir() {
        return Ok(());
    }
    for archive in paths_by_timestamp_name(&dir, true) {
        if min_retained > 0 {
            min_retained -= 1;
            continue;
        }
        tracing::info!(
            "Pruning {} archive {}",
            period.name().lower_inv(),
            file_name(&archive)
        );
        retry_file_operation(s, || fs::remove_file(&archive))?;
    }
    Ok(())
}

/// `AutoArchiveLocally()`: starts the rollup on a worker thread when a period elapsed and none is running.
pub fn auto_archive_locally(w: &mut World) {
    let Some(s) = w.resources.get_mut::<AutoArchive>().0.clone() else {
        return;
    };
    let date = w.now;
    {
        let sc = lock(&s.schedule);
        if date < sc.next_hourly && date < sc.next_daily && date < sc.next_monthly {
            return;
        }
    }
    if s.is_archiving.swap(true, Ordering::SeqCst) {
        tracing::debug!("Archive operation already in progress, skipping this cycle");
        return;
    }
    std::thread::spawn(move || {
        // Use current time for schedule checks (not the captured time from the caller)
        // to avoid re-processing periods that elapsed during a long archive operation.
        let n = now();
        let r = (|| -> io::Result<()> {
            let (mut h, mut d, mut m) = {
                let sc = lock(&s.schedule);
                (sc.next_hourly, sc.next_daily, sc.next_monthly)
            };
            if n >= h {
                rollup(&s, ArchivePeriod::Hourly)?;
                advance_schedule(&mut h, n, |dt| dt.add_hours(1.0));
                lock(&s.schedule).next_hourly = h;
            }
            if n >= d {
                rollup(&s, ArchivePeriod::Daily)?;
                advance_schedule(&mut d, n, |dt| dt.add_days(1.0));
                lock(&s.schedule).next_daily = d;
            }
            if n >= m {
                rollup(&s, ArchivePeriod::Monthly)?;
                advance_schedule(&mut m, n, |dt| {
                    dt.checked_add_months(1).unwrap_or(DateTime::MAX)
                });
                lock(&s.schedule).next_monthly = m;
            }
            if s.prune_enabled {
                prune(&s, now())?;
            }
            Ok(())
        })();
        if let Err(e) = r {
            tracing::error!("Archive operation failed: {e}");
        }
        s.is_archiving.store(false, Ordering::SeqCst);
    });
}

/// `ForceRollup()`: an immediate rollup regardless of schedule.
pub fn force_rollup(w: &mut World) {
    let Some(s) = w.resources.get_mut::<AutoArchive>().0.clone() else {
        return;
    };
    if s.is_archiving.swap(true, Ordering::SeqCst) {
        tracing::warn!("Cannot force rollup - an archive operation is already in progress");
        return;
    }
    std::thread::spawn(move || {
        let r = (|| -> io::Result<()> {
            rollup(&s, ArchivePeriod::Hourly)?;
            rollup(&s, ArchivePeriod::Daily)?;
            rollup(&s, ArchivePeriod::Monthly)?;
            if s.prune_enabled {
                prune(&s, now())?;
            }
            Ok(())
        })();
        if let Err(e) = r {
            tracing::error!("Forced archive operation failed: {e}");
        }
        s.is_archiving.store(false, Ordering::SeqCst);
    });
}

/// Moves a schedule marker past `now` so a long operation doesn't cause redundant catch-up cycles.
fn advance_schedule(
    schedule: &mut DateTime,
    now: DateTime,
    advance: impl Fn(DateTime) -> DateTime,
) {
    while *schedule <= now {
        let next = advance(*schedule);
        if next <= *schedule {
            break;
        }
        *schedule = next;
    }
}

fn rollup(s: &Shared, period: ArchivePeriod) -> io::Result<()> {
    if !s.cfg.automatic_backup_path.is_dir() {
        return Ok(());
    }
    let current_range_start = archive_period_start(now(), period);

    let mut items: IndexMap<DateTime, BTreeMap<Reverse<DateTime>, PathBuf>> = IndexMap::new();
    for path in fs::read_dir(&s.cfg.automatic_backup_path)?
        .flatten()
        .map(|d| d.path())
    {
        if !path.is_dir() {
            continue;
        }
        // Skip backup directories that are still being written (no completion marker)
        if !path.join(BACKUP_COMPLETE_MARKER).exists() {
            continue;
        }
        let Some(date) = try_get_date(&file_name(&path)) else {
            continue;
        };
        let range_start = archive_period_start(date, period);
        // Only archive the past
        if range_start >= current_range_start {
            continue;
        }
        items
            .entry(range_start)
            .or_default()
            .insert(Reverse(date), path);
    }

    let period_str = period.name();
    let archive_dir = s.cfg.archive_path.join(period_str);
    fs::create_dir_all(&archive_dir)?;
    let period_lower = period_str.lower_inv();

    // Leave behind 1 hourly for daily, 1 daily for monthly.
    let minimum = usize::from(period != ArchivePeriod::Monthly);

    for (range_start, sorted_backups) in items {
        let backups: Vec<PathBuf> = sorted_backups.into_values().collect();
        if backups.len() <= minimum {
            continue;
        }
        let started = Instant::now();
        let file_name = format!("{}.tar.zst", to_time_stamp(range_start, period));
        let archive_file = archive_dir.join(&file_name);
        let temp_file = format!("{}.tmp", archive_file.display());

        // Journal: record start
        let id = lock(&s.journal).begin_operation(
            now(),
            period,
            range_start,
            temp_file,
            archive_file.to_string_lossy().into_owned(),
            backups
                .iter()
                .map(|b| b.to_string_lossy().into_owned())
                .collect(),
        );

        // Create archive using managed streaming compression
        let entry_count = managed_archive::create_tar_zstd(
            &backups,
            &archive_file,
            &s.cfg.automatic_backup_path,
            s.cfg.compression_level,
        );

        if entry_count >= 0 {
            // Verify if enabled
            if s.cfg.verify_archives {
                let verified = managed_archive::count_entries(&archive_file);
                if verified != entry_count {
                    tracing::warn!(
                        "Archive verification failed for {}: expected {entry_count} entries, got {verified}",
                        archive_file.display()
                    );
                    lock(&s.journal).record_failure(
                        now(),
                        &id,
                        &format!("Verification failed: expected {entry_count}, got {verified}"),
                    );
                    fire_failed(s, period, range_start, "Archive verification failed");
                    continue;
                }
            }

            let archive_size = fs::metadata(&archive_file).map(|m| m.len()).unwrap_or(0);
            lock(&s.journal).record_archived(&id, entry_count, archive_size as i64);
            let elapsed = started.elapsed().as_secs_f64();
            tracing::info!(
                "Created {period_lower} archive at {} ({entry_count} entries, {:.1} MB, {elapsed:.2}s)",
                archive_file.display(),
                archive_size as f64 / 1_048_576.0
            );

            // Distribute to registered destinations
            let all_succeeded =
                distribute_to_destinations(s, &archive_file, period, range_start, &id);

            // Only delete source backups if all destinations with retention succeeded
            if all_succeeded {
                // Keep the latest one, but delete the rest.
                for backup in backups.iter().skip(minimum) {
                    retry_file_operation(s, || fs::remove_dir_all(backup))?;
                }
                lock(&s.journal).record_completed(now(), &id);
            } else {
                tracing::warn!(
                    "Not pruning source backups for {period_lower} archive - some destinations failed"
                );
            }

            let args = ArchiveCompletedEventArgs {
                archive_file_path: archive_file.to_string_lossy().into_owned(),
                period,
                range_start,
                file_size_bytes: archive_size,
                elapsed_seconds: started.elapsed().as_secs_f64(),
            };
            for f in lock(&s.on_completed).iter() {
                f(&args);
            }
        } else {
            tracing::warn!("Failed to create {period_lower} archive");
            lock(&s.journal).record_failure(now(), &id, "Archive creation failed");
            fire_failed(s, period, range_start, "Archive creation failed");
        }
    }
    Ok(())
}

fn fire_failed(s: &Shared, period: ArchivePeriod, range_start: DateTime, why: &str) {
    let args = ArchiveFailedEventArgs {
        period,
        range_start,
        exception: why.to_string(),
    };
    for f in lock(&s.on_failed).iter() {
        f(&args);
    }
}

fn distribute_to_destinations(
    s: &Shared,
    archive_file: &Path,
    period: ArchivePeriod,
    range_start: DateTime,
    id: &str,
) -> bool {
    let destinations = lock(&s.destinations).destinations().to_vec();
    if destinations.is_empty() {
        lock(&s.journal).record_distributed(id, IndexMap::new());
        return true;
    }
    let mut results = IndexMap::new();
    let mut all_succeeded = true;
    for dest in destinations {
        let sent = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            dest.send_archive(archive_file, period, range_start)
        }));
        match sent {
            Ok(true) => {
                results.insert(dest.name().to_string(), true);
                tracing::debug!("Archive sent to destination {}", dest.name());
            }
            Ok(false) => {
                results.insert(dest.name().to_string(), false);
                tracing::warn!("Destination {} rejected archive", dest.name());
                if dest.get_retention_count(period) > 0 {
                    all_succeeded = false;
                }
            }
            Err(_) => {
                tracing::error!("Failed to send archive to destination {}", dest.name());
                results.insert(dest.name().to_string(), false);
                if dest.get_retention_count(period) > 0 {
                    all_succeeded = false;
                }
            }
        }
    }
    lock(&s.journal).record_distributed(id, results);
    all_succeeded
}

/// `RetryFileOperation`: an IO error is retried (waiting `retryDelayMs * attempt`) until the last attempt, whose
/// error is returned.
fn retry_file_operation(s: &Shared, mut op: impl FnMut() -> io::Result<()>) -> io::Result<()> {
    let count = s.cfg.retry_count;
    for attempt in 0..count {
        match op() {
            Ok(()) => return Ok(()),
            Err(e) if attempt < count - 1 => {
                tracing::debug!(
                    "File operation failed on attempt {}/{count}, retrying: {e}",
                    attempt + 1
                );
                std::thread::sleep(Duration::from_millis(
                    (s.cfg.retry_delay_ms.max(0) * (attempt + 1)) as u64,
                ));
            }
            Err(e) => return Err(e),
        }
    }
    Ok(())
}

/// `ToTimeStamp(date, period)`.
pub fn to_time_stamp(d: DateTime, period: ArchivePeriod) -> String {
    let (y, mo, da) = d.ymd();
    match period {
        ArchivePeriod::Monthly => format!("{y:04}-{mo:02}"),
        ArchivePeriod::Daily => format!("{y:04}-{mo:02}-{da:02}"),
        ArchivePeriod::Hourly => format!("{y:04}-{mo:02}-{da:02}-{:02}", d.hour()),
    }
}

/// `ArchivePeriodStart(date, period)`.
pub fn archive_period_start(d: DateTime, period: ArchivePeriod) -> DateTime {
    match period {
        ArchivePeriod::Monthly => d.date().add_days(-f64::from(d.day() - 1)),
        ArchivePeriod::Daily => d.date(),
        ArchivePeriod::Hourly => d.date().add_hours(f64::from(d.hour())),
    }
}

/// Files or directories of `path` named by a timestamp, newest first. (Might give wrong results if there is a
/// file that matches: 2021-09-01, and 2021-09-01-00.)
pub fn paths_by_timestamp_name(path: &Path, files: bool) -> Vec<PathBuf> {
    let mut items: BTreeMap<Reverse<DateTime>, PathBuf> = BTreeMap::new();
    let Ok(rd) = fs::read_dir(path) else {
        return Vec::new();
    };
    for item in rd.flatten().map(|e| e.path()) {
        if item.is_dir() == files {
            continue;
        }
        let name = file_name(&item);
        let name = if files {
            name.split('.').next().unwrap_or("").to_string()
        } else {
            name
        };
        if let Some(date) = try_get_date(&name) {
            items.insert(Reverse(date), item);
        }
    }
    items.into_values().collect()
}

/// `TryGetDate`: `yyyy[-MM[-dd[-HH[-mm[-ss]]]]]`, taking the leading numeric parts; any invalid date is `None`.
pub fn try_get_date(value: &str) -> Option<DateTime> {
    let mut parts = [0u32, 1, 1, 0, 0, 0];
    let mut i = 0;
    for part in value.split('-').filter(|p| !p.is_empty()) {
        let Ok(v) = part.parse::<i32>() else { break };
        // A seventh part is out of range, so not a date.
        let slot = parts.get_mut(i)?;
        *slot = u32::try_from(v).ok()?;
        i += 1;
    }
    if i == 0 {
        return None;
    }
    DateTime::from_ymd_hms(
        i32::try_from(parts[0]).ok()?,
        parts[1],
        parts[2],
        parts[3],
        parts[4],
        parts[5],
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dates_parse_like_try_get_date() {
        assert_eq!(try_get_date("2024-05"), DateTime::from_ymd(2024, 5, 1));
        assert_eq!(
            try_get_date("2024-05-06-07"),
            DateTime::from_ymd_hms(2024, 5, 6, 7, 0, 0)
        );
        assert_eq!(
            try_get_date("2024-05-06-07-08-09"),
            DateTime::from_ymd_hms(2024, 5, 6, 7, 8, 9)
        );
        assert_eq!(try_get_date("2024-13"), None);
        assert_eq!(try_get_date("nope"), None);
        assert_eq!(try_get_date("2024-05-06-07-08-09-10"), None);
        assert_eq!(try_get_date(""), None);
    }

    #[test]
    fn period_starts_and_stamps() {
        let d = DateTime::from_ymd_hms(2024, 5, 6, 7, 8, 9).unwrap();
        assert_eq!(
            archive_period_start(d, ArchivePeriod::Hourly),
            DateTime::from_ymd_hms(2024, 5, 6, 7, 0, 0).unwrap()
        );
        assert_eq!(
            archive_period_start(d, ArchivePeriod::Daily),
            DateTime::from_ymd(2024, 5, 6).unwrap()
        );
        assert_eq!(
            archive_period_start(d, ArchivePeriod::Monthly),
            DateTime::from_ymd(2024, 5, 1).unwrap()
        );
        assert_eq!(to_time_stamp(d, ArchivePeriod::Hourly), "2024-05-06-07");
        assert_eq!(to_time_stamp(d, ArchivePeriod::Monthly), "2024-05");
    }

    #[test]
    fn newest_timestamp_first() {
        let root = std::env::temp_dir().join(format!("muo_autoarch_{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        for n in [
            "2024-01-01.tar.zst",
            "2024-03-01.tar.zst",
            "junk.txt",
            "2024-02-01.tar.zst",
        ] {
            fs::write(root.join(n), b"").unwrap();
        }
        let names: Vec<_> = paths_by_timestamp_name(&root, true)
            .iter()
            .map(|p| file_name(p))
            .collect();
        assert_eq!(
            names,
            [
                "2024-03-01.tar.zst",
                "2024-02-01.tar.zst",
                "2024-01-01.tar.zst"
            ]
        );
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn schedule_advances_past_now() {
        let now = DateTime::from_ymd_hms(2024, 5, 6, 7, 30, 0).unwrap();
        let mut s = DateTime::from_ymd_hms(2024, 5, 6, 3, 0, 0).unwrap();
        advance_schedule(&mut s, now, |d| d.add_hours(1.0));
        assert_eq!(s, DateTime::from_ymd_hms(2024, 5, 6, 8, 0, 0).unwrap());
    }
}
