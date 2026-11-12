//! The save commands: `[Save`, `[SetSaves`/`[AutoSave` and `[ArchiveNow`.

use std::rc::Rc;

use muo_server::World;
use muo_server::commands::{CommandEventArgs, CommandSystem};
use muo_server::mobile::AccessLevel;
use muo_server::mobile::part2::messages::send_message;

use super::archive::ArchivePeriod;
use super::archive_journal::ArchiveOperationState;
use super::auto_archive::{self, AutoArchive};
use super::auto_save::{self, AutoSave};

pub fn configure(w: &mut World) {
    let admin = AccessLevel::Administrator;
    CommandSystem::register(w, "Save", admin, &[], Rc::new(save_on_command));
    #[allow(clippy::type_complexity)] // the handler type, spelled out
    let set_saves: Rc<dyn Fn(&mut World, &mut CommandEventArgs)> =
        Rc::new(set_auto_saves_on_command);
    CommandSystem::register(w, "SetSaves", admin, &[], set_saves.clone());
    CommandSystem::register(w, "AutoSave", admin, &[], set_saves);
    CommandSystem::register(
        w,
        "SaveFrequency",
        admin,
        &[],
        Rc::new(set_save_frequency_on_command),
    );
    CommandSystem::register(
        w,
        "PruneArchives",
        admin,
        &[],
        Rc::new(prune_archives_on_command),
    );
    CommandSystem::register(
        w,
        "ArchiveStatus",
        admin,
        &[],
        Rc::new(archive_status_on_command),
    );
    CommandSystem::register(w, "ArchiveNow", admin, &[], Rc::new(archive_now_on_command));
}

/// `[Usage("PruneArchives"), Description("Prunes archives folder.")]`
fn prune_archives_on_command(w: &mut World, _e: &mut CommandEventArgs) {
    if let Err(e) = auto_archive::prune_backups(w) {
        tracing::error!("PruneArchives failed: {e}");
    }
}

/// `[Usage("Save"), Description("Saves the world.")]`
fn save_on_command(w: &mut World, _e: &mut CommandEventArgs) {
    auto_save::save(w);
}

/// `[Usage("AutoSave <on | off>"), Description("Enables or disables automatic shard saving.")]`
fn set_auto_saves_on_command(w: &mut World, e: &mut CommandEventArgs) {
    if e.length() == 1 {
        let enabled = e.get_boolean(0);
        auto_save::set_saves_enabled(w, enabled);
        send_message(
            w,
            e.mobile,
            if enabled {
                "Saves have been enabled."
            } else {
                "Saves have been disabled."
            },
        );
    } else {
        send_message(w, e.mobile, "Format: AutoSave <on | off>");
    }
}

/// `[Usage("SaveFrequency <duration> [warning duration]"), Description("Sets the save frequency starting at midnight local to the server.")]`
fn set_save_frequency_on_command(w: &mut World, e: &mut CommandEventArgs) {
    if e.length() < 1 {
        send_message(
            w,
            e.mobile,
            "Format: SaveFrequency <duration> [warning duration]",
        );
        return;
    }
    let save_delay = e.get_time_span(0);
    let warning_delay = if e.length() >= 2 {
        e.get_time_span(1)
    } else {
        AutoSave::warning(w)
    };
    auto_save::reset_auto_save(w, save_delay, warning_delay);
    send_message(w, e.mobile, "Save frequency has been updated.");
}

fn format_ymd_hm(d: muo_primitives::DateTime) -> String {
    let (y, mo, da) = d.ymd();
    format!("{y:04}-{mo:02}-{da:02} {:02}:{:02}", d.hour(), d.minute())
}

/// `[Usage("ArchiveStatus"), Description("Shows archive system status, journal state, and registered destinations.")]`
fn archive_status_on_command(w: &mut World, e: &mut CommandEventArgs) {
    let m = e.mobile;
    let aa = w.resources.get_mut::<AutoArchive>().clone();
    send_message(w, m, "--- Archive Status ---");
    let (h, d, mo) = (
        aa.next_hourly_archive(),
        aa.next_daily_archive(),
        aa.next_monthly_archive(),
    );
    send_message(w, m, &format!("Next hourly: {}", format_ymd_hm(h)));
    send_message(w, m, &format!("Next daily: {}", &format_ymd_hm(d)[..10]));
    send_message(w, m, &format!("Next monthly: {}", &format_ymd_hm(mo)[..7]));

    let destinations = aa.destinations();
    send_message(
        w,
        m,
        &format!("Registered destinations: {}", destinations.len()),
    );
    for dest in &destinations {
        let r = |p| dest.get_retention_count(p);
        send_message(
            w,
            m,
            &format!(
                "  - {} (retention: {}h/{}d/{}m)",
                dest.name(),
                r(ArchivePeriod::Hourly),
                r(ArchivePeriod::Daily),
                r(ArchivePeriod::Monthly)
            ),
        );
    }

    let operations = aa.journal_operations();
    let mut pending = 0;
    let mut failed = 0;
    for op in &operations {
        match op.state {
            ArchiveOperationState::Started
            | ArchiveOperationState::Archived
            | ArchiveOperationState::Distributed => pending += 1,
            ArchiveOperationState::Failed => failed += 1,
            _ => {}
        }
    }
    send_message(
        w,
        m,
        &format!(
            "Journal entries: {} ({pending} pending, {failed} failed)",
            operations.len()
        ),
    );
}

/// `[Usage("ArchiveNow"), Description("Forces an immediate archive rollup regardless of schedule.")]`
fn archive_now_on_command(w: &mut World, e: &mut CommandEventArgs) {
    send_message(w, e.mobile, "Forcing immediate archive rollup...");
    auto_archive::force_rollup(w);
}
