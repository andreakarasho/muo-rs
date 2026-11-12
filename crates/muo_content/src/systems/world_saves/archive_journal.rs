//! Archive journal: the `.archive-journal.json` recovery log of archive operations.
//!
//! The statics become an [`ArchiveJournal`] value; `AutoArchive` shares it between the loop and its worker thread
//! behind a mutex.

use muo_primitives::dotnet::InvariantCase;
use std::fs;
use std::path::PathBuf;

use indexmap::IndexMap;
use muo_primitives::DateTime;
use serde::{Deserialize, Serialize};

use super::archive::ArchivePeriod;

const MAX_JOURNAL_ENTRIES: usize = 100;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum ArchiveOperationState {
    Started,
    Archived,
    Distributed,
    Completed,
    Failed,
}

/// System.Text.Json's round-trip form: `Z` after a UTC value and the fraction trimmed of trailing zeros.
mod dt_json {
    use muo_primitives::DateTime;
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(d: &DateTime, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&d.to_json_string())
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<DateTime, D::Error> {
        let s = String::deserialize(d)?;
        DateTime::parse_json_with_local(&s, crate::systems::world_saves::local_time::offset_at)
            .ok_or_else(|| serde::de::Error::custom(format!("invalid DateTime '{s}'")))
    }
}

fn max_date() -> DateTime {
    DateTime::MAX
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ArchiveJournalEntry {
    #[serde(default)]
    pub id: String,
    pub period: ArchivePeriod,
    #[serde(with = "dt_json")]
    pub range_start: DateTime,
    pub state: ArchiveOperationState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub archive_file: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temp_file: Option<String>,
    #[serde(default)]
    pub source_directories: Vec<String>,
    #[serde(default)]
    pub entry_count: i32,
    #[serde(default)]
    pub total_source_bytes: i64,
    #[serde(default)]
    pub archive_bytes: i64,
    #[serde(with = "dt_json")]
    pub started_at: DateTime,
    #[serde(with = "dt_json", default = "max_date")]
    pub completed_at: DateTime,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub failure_reason: Option<String>,
    #[serde(default)]
    pub destination_results: IndexMap<String, bool>,
}

#[derive(Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct ArchiveJournalData {
    #[serde(default)]
    operations: Vec<ArchiveJournalEntry>,
}

#[derive(Default)]
pub struct ArchiveJournal {
    journal_path: PathBuf,
    data: ArchiveJournalData,
}

impl ArchiveJournal {
    /// `Configure(archivePath)`: loads `<archivePath>/.archive-journal.json` if it reads, else starts fresh.
    pub fn configure(archive_path: &std::path::Path) -> Self {
        let mut j = ArchiveJournal {
            journal_path: archive_path.join(".archive-journal.json"),
            data: Default::default(),
        };
        j.load();
        j
    }

    pub fn operations(&self) -> &[ArchiveJournalEntry] {
        &self.data.operations
    }

    fn load(&mut self) {
        if !self.journal_path.exists() {
            return;
        }
        match muo_data::json::deserialize_file::<ArchiveJournalData>(&self.journal_path) {
            Ok(d) => self.data = d.unwrap_or_default(),
            Err(e) => {
                tracing::error!("Failed to load archive journal, starting fresh: {e}");
                self.data = Default::default();
            }
        }
    }

    fn save(&self) {
        let run = || -> Result<(), String> {
            let mut temp = self.journal_path.clone().into_os_string();
            temp.push(".tmp");
            let temp = PathBuf::from(temp);
            muo_data::json::serialize_file(&temp, &self.data).map_err(|e| e.to_string())?;
            if self.journal_path.exists() {
                fs::remove_file(&self.journal_path).map_err(|e| e.to_string())?;
            }
            fs::rename(&temp, &self.journal_path).map_err(|e| e.to_string())
        };
        if let Err(e) = run() {
            tracing::error!("Failed to save archive journal: {e}");
        }
    }

    fn prune(&mut self) {
        let ops = &mut self.data.operations;
        if ops.len() > MAX_JOURNAL_ENTRIES {
            // Remove oldest completed/failed entries beyond the limit
            let mut to_remove = ops.len() - MAX_JOURNAL_ENTRIES;
            let mut i = ops.len();
            while i > 0 && to_remove > 0 {
                i -= 1;
                if matches!(
                    ops[i].state,
                    ArchiveOperationState::Completed | ArchiveOperationState::Failed
                ) {
                    ops.remove(i);
                    to_remove -= 1;
                }
            }
        }
    }

    /// `BeginOperation`: the index of the new entry (an entry is addressed by id after a `prune`).
    pub fn begin_operation(
        &mut self,
        now: DateTime,
        period: ArchivePeriod,
        range_start: DateTime,
        temp_file: String,
        archive_file: String,
        source_directories: Vec<String>,
    ) -> String {
        let id = format!(
            "{}-{}",
            range_start.to_iso_string(),
            period.name().lower_inv()
        );
        self.data.operations.push(ArchiveJournalEntry {
            id: id.clone(),
            period,
            range_start,
            state: ArchiveOperationState::Started,
            temp_file: Some(temp_file),
            archive_file: Some(archive_file),
            source_directories,
            entry_count: 0,
            total_source_bytes: 0,
            archive_bytes: 0,
            started_at: now,
            completed_at: DateTime::MAX,
            failure_reason: None,
            destination_results: IndexMap::new(),
        });
        self.prune();
        self.save();
        id
    }

    fn entry(&mut self, id: &str) -> Option<&mut ArchiveJournalEntry> {
        self.data.operations.iter_mut().rev().find(|e| e.id == id)
    }

    pub fn record_archived(&mut self, id: &str, entry_count: i32, archive_bytes: i64) {
        if let Some(e) = self.entry(id) {
            e.state = ArchiveOperationState::Archived;
            e.entry_count = entry_count;
            e.archive_bytes = archive_bytes;
            e.temp_file = None; // Temp file has been renamed to final
        }
        self.save();
    }

    pub fn record_distributed(&mut self, id: &str, results: IndexMap<String, bool>) {
        if let Some(e) = self.entry(id) {
            e.state = ArchiveOperationState::Distributed;
            e.destination_results = results;
        }
        self.save();
    }

    pub fn record_completed(&mut self, now: DateTime, id: &str) {
        if let Some(e) = self.entry(id) {
            e.state = ArchiveOperationState::Completed;
            e.completed_at = now;
        }
        self.save();
    }

    pub fn record_failure(&mut self, now: DateTime, id: &str, reason: &str) {
        if let Some(e) = self.entry(id) {
            e.state = ArchiveOperationState::Failed;
            e.failure_reason = Some(reason.to_string());
            e.completed_at = now;
        }
        self.save();
    }

    /// `RecoverInterrupted`: handles missing files and directories (an admin cleaned up the archives folder).
    pub fn recover_interrupted(&mut self, now: DateTime) {
        let mut recovered = false;
        for entry in &mut self.data.operations {
            match entry.state {
                ArchiveOperationState::Started => {
                    // Archive creation was interrupted. Clean up temp file if it exists.
                    if let Some(temp) = &entry.temp_file
                        && std::path::Path::new(temp).exists()
                        && let Err(e) = fs::remove_file(temp)
                    {
                        tracing::warn!("Failed to clean up temp file {temp}: {e}");
                    }
                    tracing::warn!(
                        "Recovered interrupted archive operation {} (was in Started state)",
                        entry.id
                    );
                    entry.state = ArchiveOperationState::Failed;
                    entry.failure_reason = Some("Interrupted during archive creation".into());
                    entry.completed_at = now;
                    recovered = true;
                }
                ArchiveOperationState::Archived => {
                    // Created but not distributed. If the file was deleted (admin cleanup), mark as failed.
                    if entry
                        .archive_file
                        .as_ref()
                        .is_some_and(|f| !std::path::Path::new(f).exists())
                    {
                        tracing::warn!(
                            "Archive file for operation {} no longer exists (manually deleted?). Marking failed.",
                            entry.id
                        );
                        entry.state = ArchiveOperationState::Failed;
                        entry.failure_reason = Some("Archive file missing on recovery".into());
                        entry.completed_at = now;
                        recovered = true;
                    } else {
                        tracing::warn!(
                            "Found unfinished archive operation {} (Archived but not distributed). Will re-attempt distribution on next cycle.",
                            entry.id
                        );
                    }
                }
                ArchiveOperationState::Distributed => {
                    // Sources should have been pruned but weren't.
                    tracing::info!(
                        "Completing interrupted operation {} (was Distributed, pruning sources)",
                        entry.id
                    );
                    entry.state = ArchiveOperationState::Completed;
                    entry.completed_at = now;
                    recovered = true;
                }
                _ => {}
            }
        }
        if recovered {
            self.save();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Dir(PathBuf);

    impl Dir {
        fn new() -> Dir {
            static N: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
            let d = std::env::temp_dir().join(format!(
                "muo_journal_{}_{}",
                std::process::id(),
                N.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
            ));
            let _ = fs::remove_dir_all(&d);
            fs::create_dir_all(&d).unwrap();
            Dir(d)
        }
    }

    impl Drop for Dir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn at(h: u32) -> DateTime {
        DateTime::from_ymd_hms(2026, 3, 22, h, 0, 0).unwrap()
    }

    fn begin(j: &mut ArchiveJournal, period: ArchivePeriod, temp: &str, archive: &str) -> String {
        j.begin_operation(
            at(14),
            period,
            at(14),
            temp.into(),
            archive.into(),
            vec!["backup1".into()],
        )
    }

    fn state(j: &ArchiveJournal) -> &ArchiveJournalEntry {
        j.operations().last().unwrap()
    }

    #[test]
    fn begin_operation_creates_started_entry() {
        let t = Dir::new();
        let mut j = ArchiveJournal::configure(&t.0);
        j.begin_operation(
            at(14),
            ArchivePeriod::Hourly,
            at(14),
            "/tmp/archive.tar.zst.tmp".into(),
            "/archives/hourly/2026-03-22-14.tar.zst".into(),
            vec!["backup1".into(), "backup2".into()],
        );
        let e = state(&j);
        assert_eq!(e.state, ArchiveOperationState::Started);
        assert_eq!(e.period, ArchivePeriod::Hourly);
        assert_eq!(e.temp_file.as_deref(), Some("/tmp/archive.tar.zst.tmp"));
        assert_eq!(e.source_directories.len(), 2);
    }

    #[test]
    fn record_archived_transitions_state() {
        let t = Dir::new();
        let mut j = ArchiveJournal::configure(&t.0);
        let id = begin(
            &mut j,
            ArchivePeriod::Daily,
            "/tmp/test.tmp",
            "/archives/daily/2026-03-22.tar.zst",
        );
        j.record_archived(&id, 42, 1_024_000);
        let e = state(&j);
        assert_eq!(e.state, ArchiveOperationState::Archived);
        assert_eq!(e.entry_count, 42);
        assert_eq!(e.archive_bytes, 1_024_000);
        assert!(e.temp_file.is_none()); // Temp file cleared after rename
    }

    #[test]
    fn record_distributed_stores_results() {
        let t = Dir::new();
        let mut j = ArchiveJournal::configure(&t.0);
        let id = begin(
            &mut j,
            ArchivePeriod::Monthly,
            "/tmp/test.tmp",
            "/archives/monthly/2026-03.tar.zst",
        );
        j.record_archived(&id, 100, 5_000_000);
        let mut results = IndexMap::new();
        results.insert("Local Filesystem".to_string(), true);
        results.insert("S3 us-east-1".to_string(), false);
        j.record_distributed(&id, results);
        let e = state(&j);
        assert_eq!(e.state, ArchiveOperationState::Distributed);
        assert!(e.destination_results["Local Filesystem"]);
        assert!(!e.destination_results["S3 us-east-1"]);
    }

    #[test]
    fn record_completed_is_terminal() {
        let t = Dir::new();
        let mut j = ArchiveJournal::configure(&t.0);
        let id = begin(
            &mut j,
            ArchivePeriod::Hourly,
            "/tmp/test.tmp",
            "/archives/hourly/test.tar.zst",
        );
        j.record_archived(&id, 10, 500);
        j.record_distributed(&id, IndexMap::new());
        j.record_completed(at(15), &id);
        let e = state(&j);
        assert_eq!(e.state, ArchiveOperationState::Completed);
        assert_ne!(e.completed_at, DateTime::MAX);
    }

    #[test]
    fn record_failure_records_reason() {
        let t = Dir::new();
        let mut j = ArchiveJournal::configure(&t.0);
        let id = begin(
            &mut j,
            ArchivePeriod::Hourly,
            "/tmp/test.tmp",
            "/archives/hourly/test.tar.zst",
        );
        j.record_failure(at(15), &id, "Disk full");
        let e = state(&j);
        assert_eq!(e.state, ArchiveOperationState::Failed);
        assert_eq!(e.failure_reason.as_deref(), Some("Disk full"));
        assert_ne!(e.completed_at, DateTime::MAX);
    }

    #[test]
    fn journal_persists_to_json_file_in_the_expected_shape() {
        let t = Dir::new();
        let mut j = ArchiveJournal::configure(&t.0);
        begin(
            &mut j,
            ArchivePeriod::Hourly,
            "/tmp/test.tmp",
            "/archives/hourly/test.tar.zst",
        );
        let text = fs::read_to_string(t.0.join(".archive-journal.json")).unwrap();
        assert!(text.contains("Hourly") && text.contains("Started"));
        assert!(
            text.contains("\"State\": \"Started\"") && text.contains("\"Operations\""),
            "{text}"
        );
        assert!(
            text.contains("\"RangeStart\": \"2026-03-22T14:00:00Z\""),
            "{text}"
        );
        // A fresh journal reads the file back.
        assert_eq!(ArchiveJournal::configure(&t.0).operations().len(), 1);
    }

    #[test]
    fn recover_interrupted_cleans_up_started_operations() {
        let t = Dir::new();
        let mut j = ArchiveJournal::configure(&t.0);
        let temp = t.0.join("interrupted.tar.zst.tmp");
        fs::write(&temp, "partial archive data").unwrap();
        begin(
            &mut j,
            ArchivePeriod::Hourly,
            &temp.to_string_lossy(),
            &t.0.join("final.tar.zst").to_string_lossy(),
        );
        j.recover_interrupted(at(15));
        let e = state(&j);
        assert_eq!(e.state, ArchiveOperationState::Failed);
        assert_eq!(
            e.failure_reason.as_deref(),
            Some("Interrupted during archive creation")
        );
        assert!(!temp.exists()); // Temp file should be cleaned up
    }

    #[test]
    fn recover_interrupted_completes_distributed_operations() {
        let t = Dir::new();
        let mut j = ArchiveJournal::configure(&t.0);
        let id = begin(
            &mut j,
            ArchivePeriod::Daily,
            "/tmp/test.tmp",
            "/archives/daily/test.tar.zst",
        );
        j.record_archived(&id, 50, 2000);
        j.record_distributed(&id, IndexMap::from([("Local".to_string(), true)]));
        j.recover_interrupted(at(15));
        assert_eq!(state(&j).state, ArchiveOperationState::Completed);
    }

    #[test]
    fn prune_drops_old_finished_entries_only() {
        let t = Dir::new();
        let mut j = ArchiveJournal::configure(&t.0);
        for _ in 0..MAX_JOURNAL_ENTRIES + 3 {
            let id = begin(&mut j, ArchivePeriod::Daily, "t", "a");
            j.record_completed(at(15), &id);
        }
        assert_eq!(j.operations().len(), MAX_JOURNAL_ENTRIES);
    }
}
