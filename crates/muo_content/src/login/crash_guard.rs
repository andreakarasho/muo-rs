//! Crash guard: on a crash, writes a report, copies the save and restarts.

use std::fs;
use std::io::Write;
use std::path::Path;

use muo_primitives::DateTime;
use muo_server::World;
use muo_server::events::ServerCrashedEventArgs;
use tracing::{error, info};

#[derive(Default)]
pub struct CrashGuardState {
    enabled: bool,
    save_backup: bool,
    restart_server: bool,
    generate_report: bool,
}

fn state(w: &mut World) -> &mut CrashGuardState {
    w.resources.get_mut::<CrashGuardState>()
}

/// `CrashGuard.Configure`.
pub fn configure(w: &mut World) {
    let get = |w: &mut World, key: &str| w.config.get_or_update_setting(key, true).unwrap_or(true);
    let (enabled, save_backup, restart_server, generate_report) = (
        get(w, "crashGuard.enabled"),
        get(w, "crashGuard.saveBackup"),
        get(w, "crashGuard.restartServer"), // Under a daemon/service, set server.restartExitCode rather than disabling this
        get(w, "crashGuard.generateReport"),
    );
    *state(w) = CrashGuardState {
        enabled,
        save_backup,
        restart_server,
        generate_report,
    };
}

/// `CrashGuard.Initialize`.
pub fn initialize(w: &mut World) {
    if state(w).enabled {
        w.events.server_crashed.subscribe(on_crash);
    }
}

/// `CrashGuard_OnCrash`.
pub fn on_crash(w: &mut World, e: &mut ServerCrashedEventArgs) {
    if state(w).generate_report {
        generate_crash_report(w, e);
    }
    if state(w).save_backup {
        backup(w);
    }
    if state(w).restart_server {
        e.close = true;
        w.core.kill(true);
    } else {
        w.wait_for_write_completion();
    }
}

/// `DirectoryCopy`: it never creates a subdirectory, so the first one under the save path fails the
/// copy (`DirectoryNotFoundException`, "Backup failed"), and `file.CopyTo(path, false)` fails on an existing file.
fn directory_copy(source: &Path, dest: &Path) -> std::io::Result<()> {
    let entries: Vec<_> = fs::read_dir(source)?.collect::<Result<_, _>>()?;
    for entry in entries
        .iter()
        .filter(|e| e.file_type().is_ok_and(|t| !t.is_dir()))
    {
        let target = dest.join(entry.file_name());
        if target.exists() {
            return Err(std::io::ErrorKind::AlreadyExists.into());
        }
        fs::copy(entry.path(), &target)?;
    }
    for entry in entries
        .iter()
        .filter(|e| e.file_type().is_ok_and(|t| t.is_dir()))
    {
        directory_copy(&entry.path(), &dest.join(entry.file_name()))?;
    }
    Ok(())
}

fn backup(w: &mut World) {
    info!("Backing up");
    let result = (|| {
        let stamp = DateTime::utc_now().to_timestamp();
        let backup_root = w
            .resources
            .get::<crate::systems::world_saves::auto_archive::AutoArchive>()
            .and_then(|a| a.backup_path().map(Path::to_path_buf))
            .unwrap_or_else(|| w.core.base_directory.join("Backups"));
        let backup_path = backup_root.join("Crashed").join(stamp);
        fs::create_dir_all(&backup_path)?;
        if w.save_path.is_dir() {
            directory_copy(&w.save_path, &backup_path)?;
        }
        Ok::<(), std::io::Error>(())
    })();
    match result {
        Ok(()) => info!("Backup done"),
        Err(_) => error!("Backup failed"),
    }
}

fn generate_crash_report(w: &mut World, e: &ServerCrashedEventArgs) {
    info!("Generating crash report");
    let stamp = DateTime::utc_now().to_timestamp();
    let path = w.core.base_directory.join(format!("Crash {stamp}.log"));
    let result = (|| {
        let mut op = fs::File::create(&path)?;
        writeln!(op, "Server Crash Report")?;
        writeln!(op, "===================")?;
        writeln!(op)?;
        let mut ver = env!("CARGO_PKG_VERSION")
            .split('.')
            .chain(std::iter::repeat("0"));
        let (major, minor, build) = (
            ver.next().unwrap_or("0"),
            ver.next().unwrap_or("0"),
            ver.next().unwrap_or("0"),
        );
        writeln!(op, "muo Version {major}.{minor}, Build {build}.0")?;
        writeln!(op, "Operating System: {}", std::env::consts::OS)?;
        writeln!(op, "Time: {stamp}")?;
        writeln!(op, "Accounts: {}", crate::accounting::accounts(w).count())?;
        writeln!(op, "Mobiles: {}", w.mobiles.len())?;
        writeln!(op, "Items: {}", w.items.len())?;
        writeln!(op, "Exception: {}", e.exception)?;
        writeln!(op)?;
        writeln!(op, "Clients:")?;
        writeln!(op, "- Count: {}", w.net_states.len())?;
        for d in w.net_states.values() {
            write!(op, "+ {}:", d.description)?;
            if let Some(a) = &d.account {
                write!(op, " (account = {a})")?;
            }
            if let Some((id, m)) = d.mobile.and_then(|m| Some((m, w.mobile(m)?))) {
                write!(
                    op,
                    " (mobile = {} '{}')",
                    m.serial,
                    (m.ty.name)(w, id).unwrap_or_default()
                )?;
            }
            writeln!(op)?;
        }
        Ok::<(), std::io::Error>(())
    })();
    match result {
        Ok(()) => {
            info!("Crash report generated");
            info!("Sending crash email");
            crate::misc::email::send_crash_email(w, &path);
        }
        Err(_) => error!("Crash report generation failed"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `DirectoryCopy` creates no subdirectory, so a save path with one fails (logged as "Backup failed").
    #[test]
    fn directory_copy_fails_on_a_subdirectory_and_on_an_existing_file() {
        let root = std::env::temp_dir().join(format!("muo_crashguard_{}", std::process::id()));
        let (src, dst) = (root.join("src"), root.join("dst"));
        fs::create_dir_all(src.join("Items")).unwrap();
        fs::create_dir_all(&dst).unwrap();
        fs::write(src.join("a.bin"), b"a").unwrap();
        fs::write(src.join("Items").join("b.bin"), b"b").unwrap();
        assert!(directory_copy(&src, &dst).is_err());
        assert!(
            dst.join("a.bin").exists(),
            "the top-level files were copied first"
        );
        assert!(directory_copy(&src, &dst).is_err());
        fs::remove_dir_all(&root).unwrap();
    }
}
