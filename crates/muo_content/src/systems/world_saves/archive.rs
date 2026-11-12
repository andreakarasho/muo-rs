//! World save archiving: the archive enums and event args, `IArchiveDestination`, `LocalArchiveDestination` and
//! `ArchiveDestinationRegistry`.

use std::path::Path;
use std::sync::Arc;

use muo_primitives::DateTime;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, serde::Serialize, serde::Deserialize)]
pub enum ArchivePeriod {
    Hourly,
    Daily,
    Monthly,
}

impl ArchivePeriod {
    /// `period.ToString()`.
    pub fn name(self) -> &'static str {
        match self {
            Self::Hourly => "Hourly",
            Self::Daily => "Daily",
            Self::Monthly => "Monthly",
        }
    }
}

/// `ArchiveCompletedEventArgs`.
#[derive(Clone, Debug)]
pub struct ArchiveCompletedEventArgs {
    pub archive_file_path: String,
    pub period: ArchivePeriod,
    pub range_start: DateTime,
    pub file_size_bytes: u64,
    pub elapsed_seconds: f64,
}

/// `ArchiveFailedEventArgs`; the exception is its message.
#[derive(Clone, Debug)]
pub struct ArchiveFailedEventArgs {
    pub period: ArchivePeriod,
    pub range_start: DateTime,
    pub exception: String,
}

/// `IArchiveDestination`: runs on a background thread, so it is `Send + Sync`.
pub trait ArchiveDestination: Send + Sync {
    /// A human-readable name for logging.
    fn name(&self) -> &str;
    /// Called after a local archive file is created and verified; true if the destination received it.
    fn send_archive(
        &self,
        archive_file_path: &Path,
        period: ArchivePeriod,
        range_start: DateTime,
    ) -> bool;
    /// The number of local archives to retain for this period; destinations keeping their own copies may say 0.
    fn get_retention_count(&self, period: ArchivePeriod) -> i32;
}

/// `LocalArchiveDestination`: the archive is already on the local disk, so sending is a no-op.
pub struct LocalArchiveDestination {
    hourly_retention: i32,
    daily_retention: i32,
    monthly_retention: i32,
}

impl LocalArchiveDestination {
    pub fn new(hourly_retention: i32, daily_retention: i32, monthly_retention: i32) -> Self {
        Self {
            hourly_retention,
            daily_retention,
            monthly_retention,
        }
    }
}

impl ArchiveDestination for LocalArchiveDestination {
    fn name(&self) -> &str {
        "Local Filesystem"
    }

    fn send_archive(&self, _: &Path, _: ArchivePeriod, _: DateTime) -> bool {
        true
    }

    fn get_retention_count(&self, period: ArchivePeriod) -> i32 {
        match period {
            ArchivePeriod::Hourly => self.hourly_retention,
            ArchivePeriod::Daily => self.daily_retention,
            ArchivePeriod::Monthly => self.monthly_retention,
        }
    }
}

/// `ArchiveDestinationRegistry`: register during `Configure`.
#[derive(Default, Clone)]
pub struct ArchiveDestinationRegistry {
    destinations: Vec<Arc<dyn ArchiveDestination>>,
}

impl ArchiveDestinationRegistry {
    pub fn destinations(&self) -> &[Arc<dyn ArchiveDestination>] {
        &self.destinations
    }

    pub fn register(&mut self, destination: Arc<dyn ArchiveDestination>) {
        self.destinations.push(destination);
    }

    pub fn unregister(&mut self, destination: &Arc<dyn ArchiveDestination>) {
        if let Some(i) = self
            .destinations
            .iter()
            .position(|d| Arc::ptr_eq(d, destination))
        {
            self.destinations.remove(i);
        }
    }
}
