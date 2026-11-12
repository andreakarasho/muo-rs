//! The blocklist (`BlocklistFilter`, `BlocklistConfiguration`): a reputation list
//! (`Configuration/ip-blocklist.txt`, one address or CIDR per line) the accept path denies, with the manual and login
//! allowlists carving addresses out. A hit is reported to the ban channel once per suppression window.

use std::net::IpAddr;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use muo_net::CancelToken;
use muo_net::bans::ban_reasons;
use muo_net::connection_filters::ConnectionFilter;
use muo_primitives::TimeSpan;
use muo_primitives::net::to_u128;
use muo_server::World;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use super::login_allowlist::LoginAllowlist;
use super::manual_allowlist::{self, ManualAllowlist};
use super::snapshot::{BlocklistSnapshot, PromotedGuard, load_file, try_read_header};
use super::{BanReportQueue, Clock, NetworkRegistrations, StopFlag, stop_flag, unix_now};

const PATH: &str = "Configuration/blocklist.json";

/// `BlocklistSettings`.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct BlocklistSettings {
    pub enabled: bool,
    pub file: String,
    #[serde(
        rename = "reloadInterval",
        with = "muo_data::json::converters::timespan"
    )]
    pub reload_interval: TimeSpan,
    #[serde(rename = "reportHits")]
    pub report_hits: bool,
    #[serde(rename = "banDuration", with = "muo_data::json::converters::timespan")]
    pub ban_duration: TimeSpan,
    #[serde(
        rename = "promoteSuppression",
        with = "muo_data::json::converters::timespan"
    )]
    pub promote_suppression: TimeSpan,
}

impl Default for BlocklistSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            file: "Configuration/ip-blocklist.txt".to_owned(),
            reload_interval: TimeSpan::from_secs(60),
            report_hits: true,
            ban_duration: TimeSpan::from_hours(6),
            promote_suppression: TimeSpan::from_secs(60),
        }
    }
}

/// `BlocklistFilter`.
pub struct BlocklistFilter {
    snapshot: Arc<RwLock<BlocklistSnapshot>>,
    guard: PromotedGuard,
    clock: Clock,
    last_sweep: i64,
    enabled: bool,
    path: Option<PathBuf>,
    interval: Duration,
    report_hits: bool,
    ban_duration: Duration,
    suppression_ms: i64,
    manual: ManualAllowlist,
    login: LoginAllowlist,
    reports: BanReportQueue,
    stop: StopFlag,
    /// `World.WorldState`, for the poller.
    world_state: Arc<std::sync::atomic::AtomicU8>,
    settings: BlocklistSettings,
    base_directory: PathBuf,
}

impl BlocklistFilter {
    /// `Count`.
    pub fn count(&self) -> usize {
        self.snapshot.read().map_or(0, |s| s.count())
    }

    /// `Evaluate(address, nowTicks, out shouldReport)`.
    pub fn evaluate(&mut self, address: IpAddr, now_ticks: i64, now_unix: i64) -> (bool, bool) {
        if !self.snapshot.read().is_ok_and(|s| s.is_banned(address)) {
            return (false, false);
        }
        // Both are asked only once the list has matched, so they cost the common accept nothing. The file list is
        // usually redundant because the generator subtracts it, except right after an operator adds an entry
        // without regenerating, which is when someone is waiting to get back in.
        if self.manual.contains(address) {
            return (false, false);
        }
        // A feed listing an address a real player logged in from recently is far more often a false positive than
        // a compromise.
        if self.login.is_allowed(Some(address), now_unix) {
            return (false, false);
        }
        let report = self.report_hits
            && self
                .guard
                .try_mark(to_u128(address), now_ticks, self.suppression_ms);
        (true, report)
    }

    /// `LoadForTesting(snapshot, reportHits, suppressionMs)`.
    pub fn load_for_testing(
        &mut self,
        snapshot: BlocklistSnapshot,
        report_hits: bool,
        suppression_ms: i64,
    ) {
        if let Ok(mut s) = self.snapshot.write() {
            *s = snapshot;
        }
        self.report_hits = report_hits;
        self.suppression_ms = suppression_ms;
    }

    fn log_why_disabled(&self) {
        match &self.path {
            None => info!("Blocklist disabled (\"file\" empty in blocklist.json)"),
            Some(p) if p.exists() => warn!(
                "Blocklist is off (\"enabled\" false in blocklist.json) but a list is present at \"{}\"; no addresses will be denied",
                p.display()
            ),
            Some(_) => info!("Blocklist disabled (\"enabled\" false in blocklist.json)"),
        }
    }
}

/// `Reload()`: capture the header before loading, so the markers describe the version parsed.
fn reload(
    shared: &RwLock<BlocklistSnapshot>,
    path: &std::path::Path,
) -> (Option<String>, Option<std::time::SystemTime>) {
    let write_time = std::fs::metadata(path).and_then(|m| m.modified()).ok();
    let header = try_read_header(path).unwrap_or_default();
    let (next, parsed, skipped) = load_file(path);
    let count = next.count();
    if let Ok(mut s) = shared.write() {
        *s = next;
    }
    info!(
        "Blocklist loaded {parsed} entr(ies) ({count} ranges, {skipped} skipped) gen={}",
        header.generated.as_deref().unwrap_or("")
    );
    (header.generated, write_time)
}

fn changed_since_last_load(
    path: &std::path::Path,
    last_write: Option<std::time::SystemTime>,
    last_generated: &Option<String>,
) -> bool {
    let Ok(meta) = std::fs::metadata(path) else {
        return false;
    };
    if meta.modified().ok() == last_write {
        return false; // cheapest guard
    }
    match try_read_header(path) {
        Some(h) => &h.generated != last_generated,
        None => true,
    }
}

impl ConnectionFilter for BlocklistFilter {
    fn name(&self) -> &str {
        "blocklist"
    }

    fn register(&mut self) {
        let s = &self.settings;
        self.path = (!s.file.trim().is_empty()).then(|| {
            let p = PathBuf::from(&s.file);
            if p.is_absolute() {
                p
            } else {
                self.base_directory.join(p)
            }
        });
        self.enabled = s.enabled && self.path.is_some();
        self.interval = if s.reload_interval <= TimeSpan::ZERO {
            Duration::from_secs(60)
        } else {
            Duration::from_secs_f64(s.reload_interval.total_seconds())
        };
        self.report_hits = s.report_hits;
        self.ban_duration = Duration::from_secs_f64(s.ban_duration.total_seconds().max(0.0));
        self.suppression_ms = s.promote_suppression.total_milliseconds() as i64;
    }

    fn start(&mut self, _token: &CancelToken) {
        if !self.enabled {
            self.log_why_disabled();
            return;
        }
        let Some(path) = self.path.clone() else {
            return;
        };

        // The operator's override on this gate, opted into separately. Without it only the generator's subtraction
        // covers carve-outs, and that does not cover ban contributions.
        if !self.manual.enabled() {
            warn!(
                "Blocklist is on but the manual allowlist is not; set \"enabled\" in ip-allowlist.json so a carve-out also suppresses ban contributions"
            );
        }

        // A missing file is not an error: the gate stays inert until the poll picks up whatever the generator
        // first writes. No restart needed.
        let (mut last_generated, mut last_write) = if path.exists() {
            reload(&self.snapshot, &path) // synchronous prime; empty on failure (fail-open)
        } else {
            info!(
                "Blocklist inert: no list at \"{}\"; polling every {:?}",
                path.display(),
                self.interval
            );
            (None, None)
        };

        let (shared, interval, stop) = (self.snapshot.clone(), self.interval, self.stop.clone());
        let world_state = self.world_state.clone();
        std::thread::spawn(move || {
            while !stop.sleep(interval) {
                if changed_since_last_load(&path, last_write, &last_generated) {
                    // Parsing millions of lines competes with a save for CPU and page cache: wait it out.
                    if stop.wait_while_saving(&world_state) {
                        return;
                    }
                    (last_generated, last_write) = reload(&shared, &path);
                }
            }
        });
    }

    fn stop(&mut self) {}

    fn should_deny(&mut self, address: IpAddr) -> Result<bool, String> {
        let now = self.clock.tick_count();
        // Sweep the promote-guard so a distinct-IP flood cannot grow it unbounded.
        if self.report_hits && now.wrapping_sub(self.last_sweep) >= 60_000 {
            self.last_sweep = now;
            self.guard.sweep(now);
        }
        let (deny, report) = self.evaluate(address, now, unix_now());
        if report {
            // Demand-page this address up to the OS-level bouncer; the loop hands it to the ban channel.
            self.reports
                .push(address, self.ban_duration, ban_reasons::BLOCKLIST);
        }
        Ok(deny)
    }
}

/// `BlocklistFilter.Configure`: `ConnectionFilters.Register(new BlocklistFilter())`.
pub fn configure(w: &mut World) {
    let settings: BlocklistSettings = super::load_settings(w, PATH, BlocklistSettings::default());
    let manual = manual_allowlist::handle(w);
    let login = w.resources.get_mut::<LoginAllowlist>().clone();
    let reports = w.resources.get_mut::<BanReportQueue>().clone();
    let stop = stop_flag(w);
    let base_directory = w.core.base_directory.clone();
    let filter = BlocklistFilter {
        snapshot: Arc::default(),
        guard: PromotedGuard::default(),
        clock: Clock::default(),
        last_sweep: 0,
        enabled: false,
        path: None,
        interval: Duration::from_secs(60),
        report_hits: true,
        ban_duration: Duration::ZERO,
        suppression_ms: 0,
        manual,
        login,
        reports,
        stop,
        world_state: w.state_mirror.clone(),
        settings,
        base_directory,
    };
    w.resources
        .get_mut::<NetworkRegistrations>()
        .filters
        .push(Box::new(filter));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn filter(report_hits: bool) -> BlocklistFilter {
        BlocklistFilter {
            snapshot: Arc::new(RwLock::new(BlocklistSnapshot::build(b"1.2.3.0/24").0)),
            guard: PromotedGuard::default(),
            clock: Clock::default(),
            last_sweep: 0,
            enabled: true,
            path: None,
            interval: Duration::from_secs(60),
            report_hits,
            ban_duration: Duration::from_secs(3600),
            suppression_ms: 60_000,
            manual: ManualAllowlist::default(),
            login: LoginAllowlist::default(),
            reports: BanReportQueue::default(),
            stop: StopFlag::default(),
            world_state: Arc::default(),
            settings: BlocklistSettings::default(),
            base_directory: PathBuf::new(),
        }
    }

    fn ip(s: &str) -> IpAddr {
        s.parse().unwrap()
    }

    #[test]
    fn a_listed_address_is_denied_and_reported_once_per_window() {
        let mut f = filter(true);
        assert_eq!(f.evaluate(ip("1.2.3.4"), 1000, 0), (true, true));
        assert_eq!(f.evaluate(ip("1.2.3.4"), 2000, 0), (true, false));
        assert_eq!(f.evaluate(ip("1.2.3.4"), 61_000, 0), (true, true));
        assert_eq!(f.evaluate(ip("9.9.9.9"), 1000, 0), (false, false));
    }

    #[test]
    fn report_hits_off_never_reports() {
        let mut f = filter(false);
        assert_eq!(f.evaluate(ip("1.2.3.4"), 1000, 0), (true, false));
    }

    #[test]
    fn the_manual_allowlist_carves_an_address_out() {
        let mut f = filter(true);
        f.manual
            .load_for_testing(BlocklistSnapshot::build(b"1.2.3.4").0);
        assert_eq!(f.evaluate(ip("1.2.3.4"), 1000, 0), (false, false));
        assert_eq!(f.evaluate(ip("1.2.3.5"), 1000, 0), (true, true));
    }

    #[test]
    fn a_recent_login_carves_an_address_out() {
        let mut f = filter(true);
        f.login.load_for_testing(true, 1000, 0, 3600);
        f.login.record_login(Some(ip("1.2.3.4")), 500);
        assert_eq!(f.evaluate(ip("1.2.3.4"), 1000, 600), (false, false));
        assert_eq!(f.evaluate(ip("1.2.3.4"), 1000, 2000), (true, true));
    }

    #[test]
    fn settings_defaults_match_the_shipped_file() {
        let s = BlocklistSettings::default();
        assert!(!s.enabled && s.report_hits);
        let json = muo_data::json::to_string(&s).unwrap();
        assert!(json.contains("\"banDuration\": \"06:00:00\""), "{json}");
        assert!(
            json.contains("\"promoteSuppression\": \"00:01:00\""),
            "{json}"
        );
    }

    // Filter tests over an in-memory list (WithList / LoadForTesting).
    fn with_list(list: &str, report_hits: bool, suppression_ms: i64) -> BlocklistFilter {
        let mut f = filter(report_hits);
        f.load_for_testing(
            BlocklistSnapshot::build(list.as_bytes()).0,
            report_hits,
            suppression_ms,
        );
        f
    }

    #[test]
    fn listed_address_is_denied_and_reported_once_per_window() {
        let mut f = with_list("1.2.3.4", true, 5000);
        let a = ip("1.2.3.4");
        assert_eq!(f.evaluate(a, 1000, 0), (true, true));
        // denied again, but promotion suppressed inside the window
        assert_eq!(f.evaluate(a, 1500, 0), (true, false));
        // window elapsed, promotion may be retried
        assert_eq!(f.evaluate(a, 1000 + 5001, 0), (true, true));
    }

    #[test]
    fn unlisted_address_passes() {
        let mut f = with_list("1.2.3.4", true, 5000);
        assert_eq!(f.evaluate(ip("9.9.9.9"), 1, 0), (false, false));
    }

    #[test]
    fn cidr_membership_is_honored() {
        let mut f = with_list("10.20.30.0/24", true, 5000);
        assert!(f.evaluate(ip("10.20.30.255"), 1, 0).0);
        assert!(!f.evaluate(ip("10.20.31.0"), 1, 0).0);
    }

    #[test]
    fn report_hits_disabled_still_denies_but_never_promotes() {
        let mut f = with_list("1.2.3.4", false, 5000);
        assert_eq!(f.evaluate(ip("1.2.3.4"), 1, 0), (true, false));
    }

    #[test]
    fn unconfigured_filter_denies_nothing() {
        let mut f = with_list("", true, 5000);
        assert_eq!(f.count(), 0);
        assert_eq!(f.should_deny(ip("8.8.8.8")), Ok(false));
    }
}
