//! The login allowlist: an allowlist
//! addresses earn by logging in successfully, so a reputation feed cannot get a known player blocked. Saved to
//! `Configuration/login-allowlist.txt` (`address unixSeconds` per line) on a timer and at shutdown.

use indexmap::IndexMap;
use std::fmt::Write as _;
use std::net::IpAddr;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, MutexGuard};

use muo_net::bans::ban_reasons;
use muo_primitives::net::{is_private_network, to_ip_addr, to_u128};
use muo_primitives::{DateTime, TimeSpan};
use muo_server::World;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use super::{resolve_path, unix_now};

const PATH: &str = "Configuration/login-allowlist.json";

/// `LoginAllowlistSettings`.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct LoginAllowlistSettings {
    pub enabled: bool,
    pub file: String,
    #[serde(with = "muo_data::json::converters::timespan")]
    pub ttl: TimeSpan,
    #[serde(
        rename = "flushInterval",
        with = "muo_data::json::converters::timespan"
    )]
    pub flush_interval: TimeSpan,
    #[serde(rename = "escalateAfterStrikes")]
    pub escalate_after_strikes: i32,
    #[serde(rename = "strikeWindow", with = "muo_data::json::converters::timespan")]
    pub strike_window: TimeSpan,
}

impl Default for LoginAllowlistSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            file: "Configuration/login-allowlist.txt".to_owned(),
            ttl: TimeSpan::from_days(90),
            flush_interval: TimeSpan::from_hours(1),
            escalate_after_strikes: 10,
            strike_window: TimeSpan::from_hours(1),
        }
    }
}

#[derive(Copy, Clone, Default)]
struct Strike {
    count: i32,
    /// Unix seconds; 0 means no window open.
    window_start: i64,
}

#[derive(Default)]
struct State {
    enabled: bool,
    path: Option<PathBuf>,
    ttl_seconds: i64,
    escalate_after_strikes: i32,
    strike_window_seconds: i64,
    /// Address (normalized v6 bits) -> unix seconds of its last successful login.
    allowed: IndexMap<u128, i64>,
    /// Suppressed contributions in the current window. Only holds allowlisted addresses.
    strikes: IndexMap<u128, Strike>,
    dirty: bool,
    /// A write is in flight: a flush landing mid-write waits.
    writing: bool,
}

/// The allowlist both the loop (`RecordLogin`, the flush) and the accept path (`IsAllowed`, the exemption) use.
#[derive(Clone, Default)]
pub struct LoginAllowlist(Arc<Mutex<State>>);

impl LoginAllowlist {
    fn lock(&self) -> MutexGuard<'_, State> {
        self.0.lock().unwrap_or_else(|p| p.into_inner())
    }

    /// `Count`.
    pub fn count(&self) -> usize {
        self.lock().allowed.len()
    }

    /// `Enabled` (`_enabled`).
    pub fn enabled(&self) -> bool {
        self.lock().enabled
    }

    /// `RecordLogin(address, nowUnix)`.
    pub fn record_login(&self, address: Option<IpAddr>, now_unix: i64) {
        let mut s = self.lock();
        let Some(address) = address else { return };
        if !s.enabled || is_private_network(address) {
            return;
        }
        let key = to_u128(address);
        s.allowed.insert(key, now_unix);
        // A fresh login clears the tally: someone just proved they hold an account.
        s.strikes.shift_remove(&key);
        s.dirty = true;
    }

    /// `IsAllowed(address, nowUnix)`.
    pub fn is_allowed(&self, address: Option<IpAddr>, now_unix: i64) -> bool {
        let s = self.lock();
        Self::allowed(&s, address, now_unix)
    }

    fn allowed(s: &State, address: Option<IpAddr>, now_unix: i64) -> bool {
        let Some(address) = address else { return false };
        s.enabled
            && s.allowed
                .get(&to_u128(address))
                .is_some_and(|stamp| now_unix - stamp <= s.ttl_seconds)
    }

    /// `IsExemptFromEscalation(address, reason, nowUnix)`. Stateful: a true answer spends a strike.
    pub fn is_exempt_from_escalation(&self, address: IpAddr, reason: &str, now_unix: i64) -> bool {
        let mut s = self.lock();
        // An operator's explicit ban, or a reason nobody opted in, escalates untouched.
        if !ban_reasons::is_behavioral(reason) || !Self::allowed(&s, Some(address), now_unix) {
            return false;
        }
        if s.escalate_after_strikes <= 0 {
            return true; // revocation disabled: an entry is unconditional
        }
        let key = to_u128(address);
        let mut strike = s.strikes.get(&key).copied().unwrap_or_default();
        if strike.window_start == 0 || now_unix - strike.window_start > s.strike_window_seconds {
            strike = Strike {
                count: 0,
                window_start: now_unix,
            };
        }
        strike.count += 1;
        if strike.count < s.escalate_after_strikes {
            s.strikes.insert(key, strike);
            return true;
        }
        // Allowance spent: drop the entry so this and all after it escalate. Earned back by logging in.
        s.allowed.shift_remove(&key);
        s.strikes.shift_remove(&key);
        s.dirty = true;
        info!(
            "{address} revoked from the login allowlist after {} suppressed contribution(s); last was '{reason}'",
            strike.count
        );
        false
    }

    /// `LoadForTesting(enabled, ttlSeconds, escalateAfterStrikes, strikeWindowSeconds)`.
    pub fn load_for_testing(
        &self,
        enabled: bool,
        ttl_seconds: i64,
        escalate_after_strikes: i32,
        strike_window_seconds: i64,
    ) {
        *self.lock() = State {
            enabled,
            ttl_seconds,
            escalate_after_strikes,
            strike_window_seconds,
            ..State::default()
        };
    }

    /// `Snapshot`: drops what lapsed and returns the entries to write, and how many were dropped.
    fn snapshot(s: &mut State, now_unix: i64) -> (Vec<(u128, i64)>, usize) {
        let cutoff = now_unix - s.ttl_seconds;
        let before = s.allowed.len();
        s.allowed.retain(|_, stamp| *stamp >= cutoff);
        let dropped = before - s.allowed.len();
        let State {
            allowed, strikes, ..
        } = s;
        strikes.retain(|k, _| allowed.contains_key(k));
        // PruneStaleStrikes
        let window = s.strike_window_seconds;
        s.strikes
            .retain(|_, st| now_unix - st.window_start <= window);
        (s.allowed.iter().map(|(k, v)| (*k, *v)).collect(), dropped)
    }
}

/// The header and lines of the file.
fn file_text(entries: &[(u128, i64)], now: DateTime) -> String {
    let mut out = format!(
        "# muo-login-allowlist generated={} count={}\n",
        now.to_iso_seconds_string(),
        entries.len()
    );
    for (addr, stamp) in entries {
        let _ = writeln!(out, "{} {stamp}", to_ip_addr(*addr, false));
    }
    out
}

/// `Write(path, ...)`: sibling file then swap, so a reader never sees a half-written list.
fn write(path: &std::path::Path, entries: &[(u128, i64)], dropped: usize) -> bool {
    let result = (|| -> std::io::Result<()> {
        if let Some(dir) = path.parent().filter(|d| !d.as_os_str().is_empty()) {
            std::fs::create_dir_all(dir)?;
        }
        let mut tmp = path.as_os_str().to_owned();
        tmp.push(".tmp");
        let tmp = PathBuf::from(tmp);
        std::fs::write(&tmp, file_text(entries, DateTime::utc_now()))?;
        std::fs::rename(&tmp, path)
    })();
    match result {
        Ok(()) => {
            if dropped > 0 {
                info!(
                    "Login allowlist wrote {} entr(ies), dropped {dropped} past TTL",
                    entries.len()
                );
            }
            true
        }
        Err(e) => {
            // Recoverable: entries are still in memory and the next flush retries.
            warn!(
                "Could not write the login allowlist to \"{}\": {e}",
                path.display()
            );
            false
        }
    }
}

/// `LoginAllowlist.Configure`.
pub fn configure(w: &mut World) {
    let settings: LoginAllowlistSettings =
        super::load_settings(w, PATH, LoginAllowlistSettings::default());
    let handle = w.resources.get_mut::<LoginAllowlist>().clone();
    let mut s = handle.lock();
    s.enabled =
        settings.enabled && !settings.file.trim().is_empty() && settings.ttl > TimeSpan::ZERO;
    if !s.enabled {
        return;
    }
    s.path = Some(resolve_path(w, &settings.file));
    s.ttl_seconds = settings.ttl.total_seconds() as i64;
    s.escalate_after_strikes = settings.escalate_after_strikes;
    s.strike_window_seconds = settings.strike_window.total_seconds() as i64;
    drop(s);
    w.resources.insert(settings);
}

/// `LoginAllowlist.Initialize`.
pub fn initialize(w: &mut World) {
    let handle = w.resources.get_mut::<LoginAllowlist>().clone();
    if !handle.enabled() {
        info!("Login allowlist disabled");
        return;
    }
    load(&handle);

    let mut interval = w
        .resources
        .get::<LoginAllowlistSettings>()
        .map_or(TimeSpan::from_hours(1), |s| s.flush_interval);
    if interval <= TimeSpan::ZERO {
        interval = TimeSpan::from_hours(1);
    }
    let h = handle.clone();
    muo_server::timer::start(w, interval, interval, 0, move |w| flush(w, &h));

    // HandleClosed skips InvokeShutdown when the server crashed, so the crash path needs its own.
    let h = handle.clone();
    w.events.shutdown.subscribe(move |_, _| on_shutdown(&h));
    let h = handle;
    w.events
        .server_crashed
        .subscribe(move |_, _| on_shutdown(&h));
}

/// `Load()`.
fn load(handle: &LoginAllowlist) {
    let mut s = handle.lock();
    let Some(path) = s.path.clone() else { return };
    if !path.exists() {
        info!("Login allowlist empty: no file at \"{}\"", path.display());
        return;
    }
    let cutoff = unix_now() - s.ttl_seconds;
    let (mut loaded, mut skipped) = (0, 0);
    let text = match std::fs::read(&path) {
        Ok(b) => String::from_utf8_lossy(&b).into_owned(),
        Err(e) => {
            // Fail open: an unreadable list allows nobody, which beats refusing to boot.
            warn!(
                "Could not read the login allowlist at \"{}\"; continuing with {}: {e}",
                path.display(),
                s.allowed.len()
            );
            return;
        }
    };
    for line in text.lines() {
        let span = line.trim();
        if span.is_empty() || span.starts_with('#') || span.starts_with(';') {
            continue;
        }
        let parsed = span.split_once(' ').and_then(|(a, rest)| {
            Some((
                muo_primitives::dotnet::parse_ip_address(a)?,
                rest.trim().parse::<i64>().ok()?,
            ))
        });
        let Some((address, stamp)) = parsed else {
            skipped += 1;
            continue;
        };
        // Expired on disk: do not carry a stranger into memory.
        if stamp < cutoff {
            skipped += 1;
            s.dirty = true; // the file is now out of date; the next flush rewrites it
            continue;
        }
        s.allowed.insert(to_u128(address), stamp);
        loaded += 1;
    }
    info!("Login allowlist loaded {loaded} entr(ies) ({skipped} expired or malformed)");
}

/// `Flush()`: writes on a worker thread; the dirty flag and the write guard are released by it.
fn flush(w: &mut World, handle: &LoginAllowlist) {
    let saving = w.saving() || w.state == muo_server::world::WorldState::PendingSave;
    let mut s = handle.lock();
    // A save owns the disk and nothing here is urgent. dirty stays set, so skipping loses nothing.
    if !s.enabled || !s.dirty || s.writing || saving {
        return;
    }
    let (entries, dropped) = LoginAllowlist::snapshot(&mut s, w.now.to_unix_seconds());
    let Some(path) = s.path.clone() else { return };
    s.dirty = false;
    s.writing = true;
    drop(s);
    let h = handle.clone();
    std::thread::spawn(move || {
        let written = write(&path, &entries, dropped);
        let mut s = h.lock();
        s.writing = false;
        if !written {
            s.dirty = true; // nothing reached disk; the next flush retries
        }
    });
}

/// `OnShutdown()` / `OnCrashed`.
fn on_shutdown(handle: &LoginAllowlist) {
    let mut s = handle.lock();
    // A write already in flight holds the data and has all but the last moments of the list.
    if !s.enabled || !s.dirty || s.writing {
        return;
    }
    let (entries, dropped) = LoginAllowlist::snapshot(&mut s, unix_now());
    s.dirty = false;
    if let Some(path) = s.path.clone() {
        write(&path, &entries, dropped);
    }
}

/// `LoginAllowlist.RecordLogin(address)`.
pub fn record_login(w: &mut World, address: Option<IpAddr>) {
    let now = w.now.to_unix_seconds();
    w.resources
        .get_mut::<LoginAllowlist>()
        .record_login(address, now);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ip(s: &str) -> IpAddr {
        s.parse().unwrap()
    }

    #[test]
    fn records_public_addresses_only_when_enabled() {
        let l = LoginAllowlist::default();
        l.record_login(Some(ip("8.8.8.8")), 100);
        assert!(!l.is_allowed(Some(ip("8.8.8.8")), 100));
        l.load_for_testing(true, 50, 0, 3600);
        l.record_login(Some(ip("192.168.1.2")), 100);
        assert_eq!(l.count(), 0);
        l.record_login(Some(ip("8.8.8.8")), 100);
        assert!(l.is_allowed(Some(ip("8.8.8.8")), 150));
        assert!(!l.is_allowed(Some(ip("8.8.8.8")), 151));
    }

    #[test]
    fn only_behavioral_reasons_are_exempt() {
        let l = LoginAllowlist::default();
        l.load_for_testing(true, 1000, 0, 3600);
        l.record_login(Some(ip("8.8.8.8")), 100);
        assert!(l.is_exempt_from_escalation(ip("8.8.8.8"), ban_reasons::RATE_LIMIT, 100));
        assert!(!l.is_exempt_from_escalation(ip("8.8.8.8"), ban_reasons::MANUAL, 100));
        assert!(!l.is_exempt_from_escalation(ip("8.8.4.4"), ban_reasons::RATE_LIMIT, 100));
    }

    #[test]
    fn strikes_revoke_the_entry_and_a_login_earns_it_back() {
        let l = LoginAllowlist::default();
        l.load_for_testing(true, 10_000, 3, 3600);
        let a = ip("8.8.8.8");
        l.record_login(Some(a), 100);
        assert!(l.is_exempt_from_escalation(a, ban_reasons::RATE_LIMIT, 100));
        assert!(l.is_exempt_from_escalation(a, ban_reasons::RATE_LIMIT, 101));
        // the third strike spends the allowance
        assert!(!l.is_exempt_from_escalation(a, ban_reasons::RATE_LIMIT, 102));
        assert!(!l.is_allowed(Some(a), 102));
        l.record_login(Some(a), 103);
        assert!(l.is_exempt_from_escalation(a, ban_reasons::RATE_LIMIT, 104));
    }

    #[test]
    fn strikes_outside_the_window_start_over() {
        let l = LoginAllowlist::default();
        l.load_for_testing(true, 100_000, 2, 10);
        let a = ip("8.8.8.8");
        l.record_login(Some(a), 100);
        assert!(l.is_exempt_from_escalation(a, ban_reasons::SILENT_CONNECT, 100));
        // the window lapsed: the count starts again at one
        assert!(l.is_exempt_from_escalation(a, ban_reasons::SILENT_CONNECT, 200));
        assert!(!l.is_exempt_from_escalation(a, ban_reasons::SILENT_CONNECT, 201));
    }

    #[test]
    fn file_text_has_the_header_and_one_line_per_entry() {
        let t = file_text(
            &[(to_u128(ip("8.8.8.8")), 123)],
            DateTime::from_unix_seconds(0).unwrap(),
        );
        let mut lines = t.lines();
        assert!(
            lines
                .next()
                .unwrap()
                .starts_with("# muo-login-allowlist generated=1970-01-01")
        );
        assert_eq!(lines.next(), Some("8.8.8.8 123"));
    }

    #[test]
    fn snapshot_drops_lapsed_entries() {
        let l = LoginAllowlist::default();
        l.load_for_testing(true, 100, 0, 3600);
        l.record_login(Some(ip("8.8.8.8")), 0);
        l.record_login(Some(ip("8.8.4.4")), 500);
        let mut s = l.lock();
        let (entries, dropped) = LoginAllowlist::snapshot(&mut s, 550);
        assert_eq!((entries.len(), dropped), (1, 1));
    }

    // Behavior tests of the allowlist. Addresses come from
    // TEST-NET-3 (203.0.113.0/24).
    mod behavior {
        use super::*;
        use std::net::{IpAddr, Ipv4Addr};

        const TTL: i64 = 90 * 24 * 60 * 60; // 90 days, the shipped default
        const WINDOW: i64 = 3600;
        const NOW: i64 = 1_800_000_000;

        fn reset(enabled: bool, strikes: i32) -> LoginAllowlist {
            let l = LoginAllowlist::default();
            l.load_for_testing(enabled, TTL, strikes, WINDOW);
            l
        }

        fn ip(s: &str) -> IpAddr {
            s.parse().unwrap()
        }

        #[test]
        fn recent_login_allows_its_address() {
            let l = reset(true, 0);
            let a = ip("203.0.113.10");
            l.record_login(Some(a), NOW);
            assert!(l.is_allowed(Some(a), NOW));
            assert!(l.is_allowed(Some(a), NOW + TTL / 2));
        }

        #[test]
        fn entry_lapses_once_past_the_ttl() {
            let l = reset(true, 0);
            let a = ip("203.0.113.11");
            l.record_login(Some(a), NOW);
            assert!(l.is_allowed(Some(a), NOW + TTL)); // the boundary still counts
            assert!(!l.is_allowed(Some(a), NOW + TTL + 1)); // a second later it does not
        }

        #[test]
        fn logging_in_again_renews_the_window() {
            let l = reset(true, 0);
            let a = ip("203.0.113.12");
            l.record_login(Some(a), NOW);
            l.record_login(Some(a), NOW + TTL); // still allowed here, so the entry is refreshed
            assert!(l.is_allowed(Some(a), NOW + TTL + TTL));
        }

        #[test]
        fn unknown_address_is_not_allowed() {
            let l = reset(true, 0);
            l.record_login(Some(ip("203.0.113.13")), NOW);
            assert!(!l.is_allowed(Some(ip("203.0.113.14")), NOW));
        }

        #[test]
        fn private_addresses_are_never_recorded() {
            let l = reset(true, 0);
            // A LAN or loopback login says nothing about the public internet.
            l.record_login(Some(ip("10.0.0.5")), NOW);
            l.record_login(Some(IpAddr::V4(Ipv4Addr::LOCALHOST)), NOW);
            assert!(!l.is_allowed(Some(ip("10.0.0.5")), NOW));
            assert!(!l.is_allowed(Some(IpAddr::V4(Ipv4Addr::LOCALHOST)), NOW));
            assert_eq!(l.count(), 0);
        }

        #[test]
        fn disabled_list_allows_nobody() {
            let l = reset(false, 0);
            let a = ip("203.0.113.15");
            l.record_login(Some(a), NOW);
            assert!(!l.is_allowed(Some(a), NOW));
            assert_eq!(l.count(), 0);
        }

        #[test]
        fn null_address_is_handled() {
            let l = reset(true, 0);
            l.record_login(None, NOW);
            assert!(!l.is_allowed(None, NOW));
        }

        #[test]
        fn manual_bans_are_never_exempt() {
            let l = reset(true, 0);
            let a = ip("203.0.113.20");
            l.record_login(Some(a), NOW);
            // An explicit decision must reach the reporters even for an allowlisted address.
            assert!(!l.is_exempt_from_escalation(a, ban_reasons::MANUAL, NOW));
        }

        #[test]
        fn unopted_reasons_are_never_exempt() {
            let l = reset(true, 0);
            let a = ip("203.0.113.21");
            l.record_login(Some(a), NOW);
            // A reason nobody opted into IsBehavioral escalates normally.
            assert!(!l.is_exempt_from_escalation(a, "some-future-reason", NOW));
            assert!(!l.is_exempt_from_escalation(a, ban_reasons::BLOCKLIST, NOW));
        }

        #[test]
        fn behavioral_reasons_are_exempt_while_allowed() {
            let l = reset(true, 0);
            let a = ip("203.0.113.22");
            l.record_login(Some(a), NOW);
            assert!(l.is_exempt_from_escalation(a, ban_reasons::RATE_LIMIT, NOW));
            assert!(l.is_exempt_from_escalation(a, ban_reasons::SILENT_CONNECT, NOW));
            assert!(l.is_exempt_from_escalation(a, ban_reasons::INVALID_SEED, NOW));
        }

        #[test]
        fn entry_is_revoked_once_the_strikes_run_out() {
            let l = reset(true, 3);
            let a = ip("203.0.113.23");
            l.record_login(Some(a), NOW);

            assert!(l.is_exempt_from_escalation(a, ban_reasons::RATE_LIMIT, NOW));
            assert!(l.is_exempt_from_escalation(a, ban_reasons::RATE_LIMIT, NOW));

            // The third strike is the one that escalates, and it takes the entry with it.
            assert!(!l.is_exempt_from_escalation(a, ban_reasons::RATE_LIMIT, NOW));
            assert!(!l.is_allowed(Some(a), NOW));

            // Still revoked afterwards, so everything from here escalates too.
            assert!(!l.is_exempt_from_escalation(a, ban_reasons::RATE_LIMIT, NOW));
        }

        #[test]
        fn quiet_window_clears_the_tally() {
            let l = reset(true, 3);
            let a = ip("203.0.113.24");
            l.record_login(Some(a), NOW);

            assert!(l.is_exempt_from_escalation(a, ban_reasons::RATE_LIMIT, NOW));
            assert!(l.is_exempt_from_escalation(a, ban_reasons::RATE_LIMIT, NOW));

            // Past the window the count restarts, so an occasional tripper never reaches revocation.
            let later = NOW + WINDOW + 1;
            assert!(l.is_exempt_from_escalation(a, ban_reasons::RATE_LIMIT, later));
            assert!(l.is_exempt_from_escalation(a, ban_reasons::RATE_LIMIT, later));
            assert!(l.is_allowed(Some(a), later));
        }

        #[test]
        fn logging_in_again_forgives_accumulated_strikes() {
            let l = reset(true, 3);
            let a = ip("203.0.113.25");
            l.record_login(Some(a), NOW);

            assert!(l.is_exempt_from_escalation(a, ban_reasons::RATE_LIMIT, NOW));
            assert!(l.is_exempt_from_escalation(a, ban_reasons::RATE_LIMIT, NOW));

            l.record_login(Some(a), NOW); // someone proved they hold an account again

            assert!(l.is_exempt_from_escalation(a, ban_reasons::RATE_LIMIT, NOW));
            assert!(l.is_exempt_from_escalation(a, ban_reasons::RATE_LIMIT, NOW));
            assert!(l.is_allowed(Some(a), NOW));
        }

        #[test]
        fn zero_threshold_disables_revocation() {
            let l = reset(true, 0);
            let a = ip("203.0.113.26");
            l.record_login(Some(a), NOW);
            for _ in 0..50 {
                assert!(l.is_exempt_from_escalation(a, ban_reasons::RATE_LIMIT, NOW));
            }
            assert!(l.is_allowed(Some(a), NOW));
        }
    }
}
