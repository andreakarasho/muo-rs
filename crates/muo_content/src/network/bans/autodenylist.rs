//! The auto-denylist (`AutoDenylist`, `AutoDenylistConfiguration`): a short, capped
//! hold on addresses the shard itself caught misbehaving (a flooder, a silent or foreign connection), so the next
//! attempt is refused at the accept path without another detection.
//!
//! The expiry sits beside the key in an insertion-ordered ring, so the drain stops at the first live record.

use std::collections::{HashSet, VecDeque};
use std::net::IpAddr;
use std::sync::{Arc, Mutex, MutexGuard};

use muo_net::CancelToken;
use muo_net::bans::{BanReporter, ban_reasons};
use muo_net::connection_filters::ConnectionFilter;
use muo_primitives::TimeSpan;
use muo_primitives::net::to_u128;
use muo_server::World;
use serde::{Deserialize, Serialize};
use tracing::warn;

use super::{Clock, NetworkRegistrations};

const PATH: &str = "Configuration/auto-denylist.json";

/// `AutoDenylistSettings`.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct AutoDenylistSettings {
    pub enabled: bool,
    #[serde(with = "muo_data::json::converters::timespan")]
    pub duration: TimeSpan,
    #[serde(rename = "maxEntries")]
    pub max_entries: i32,
}

impl Default for AutoDenylistSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            duration: TimeSpan::from_mins(15),
            max_entries: 324_449,
        }
    }
}

#[derive(Default)]
struct State {
    /// Membership only (normalized v6 bits).
    held: HashSet<u128>,
    /// The same keys in expiry order, with the expiry beside each.
    ring: VecDeque<(u128, i64)>,
    enabled: bool,
    duration_ms: i64,
    max_entries: usize,
    warned_full: bool,
}

/// The denylist the filter and the reporter share.
#[derive(Clone, Default)]
pub struct AutoDenylist(Arc<Mutex<State>>);

impl AutoDenylist {
    fn lock(&self) -> MutexGuard<'_, State> {
        self.0.lock().unwrap_or_else(|p| p.into_inner())
    }

    /// `Count`.
    pub fn count(&self) -> usize {
        self.lock().held.len()
    }

    /// `RingCount`.
    pub fn ring_count(&self) -> usize {
        self.lock().ring.len()
    }

    /// `Hold(address, reason, nowTicks)`: true when the address is held (now or already).
    pub fn hold(&self, address: IpAddr, reason: &str, now_ticks: i64) -> bool {
        let mut s = self.lock();
        if !s.enabled || !ban_reasons::is_behavioral(reason) {
            return false;
        }
        Self::drain_locked(&mut s, now_ticks);
        let key = to_u128(address);

        // Deliberately not refreshed: the first detection sets the expiry and later ones leave it alone. That keeps
        // insertion order equal to expiry order, which is why the drain can stop at the first live record.
        if !s.held.insert(key) {
            return true;
        }
        // Drain already reclaimed everything reclaimable, so being over now means genuinely full.
        if s.held.len() > s.max_entries {
            s.held.remove(&key);
            if !s.warned_full {
                s.warned_full = true;
                warn!(
                    "Auto-denylist is full at {} addresses; further detections are disconnected but not held",
                    s.max_entries
                );
            }
            return false;
        }
        let expiry = now_ticks.wrapping_add(s.duration_ms);
        s.ring.push_back((key, expiry));
        true
    }

    /// `IsDenied(address, nowTicks)`.
    pub fn is_denied(&self, address: IpAddr, now_ticks: i64) -> bool {
        let mut s = self.lock();
        if !s.enabled {
            return false;
        }
        Self::drain_locked(&mut s, now_ticks);
        s.held.contains(&to_u128(address))
    }

    /// `Release(address)`.
    pub fn release(&self, address: IpAddr) {
        let mut s = self.lock();
        if !s.enabled {
            return;
        }
        let key = to_u128(address);
        if s.held.remove(&key) {
            // The ring record has to go too: if the address were detected again before the old record lapsed, that
            // record would retire the new hold early. O(n), but this is an operator retraction.
            if let Some(i) = s.ring.iter().position(|(k, _)| *k == key) {
                s.ring.remove(i);
            }
        }
    }

    /// `Drain(nowTicks)`.
    pub fn drain(&self, now_ticks: i64) {
        Self::drain_locked(&mut self.lock(), now_ticks);
    }

    fn drain_locked(s: &mut State, now_ticks: i64) {
        let before = s.ring.len();
        // Subtraction, never a direct compare: tick counts wrap.
        while let Some(&(key, expiry)) = s.ring.front() {
            if expiry.wrapping_sub(now_ticks) > 0 {
                break;
            }
            s.held.remove(&key);
            s.ring.pop_front();
        }
        if s.ring.len() != before {
            s.warned_full = false;
        }
    }

    /// `LoadForTesting(enabled, durationMs, maxEntries)`.
    pub fn load_for_testing(&self, enabled: bool, duration_ms: i64, max_entries: usize) {
        *self.lock() = State {
            enabled,
            duration_ms,
            max_entries,
            ..State::default()
        };
    }
}

/// `AutoDenylistFilter`.
struct AutoDenylistFilter {
    list: AutoDenylist,
    clock: Clock,
    last_sweep: i64,
}

impl ConnectionFilter for AutoDenylistFilter {
    fn name(&self) -> &str {
        "auto-denylist"
    }

    fn register(&mut self) {}

    fn start(&mut self, _token: &CancelToken) {}

    fn stop(&mut self) {}

    fn should_deny(&mut self, address: IpAddr) -> Result<bool, String> {
        let now = self.clock.tick_count();
        // Only reclaims memory (Hold and IsDenied both drain): matters on a shard gone quiet after a flood.
        if now.wrapping_sub(self.last_sweep) >= 60_000 {
            self.last_sweep = now;
            self.list.drain(now);
        }
        Ok(self.list.is_denied(address, now))
    }
}

/// `AutoDenylistReporter`.
struct AutoDenylistReporter {
    list: AutoDenylist,
    clock: Clock,
}

impl BanReporter for AutoDenylistReporter {
    fn name(&self) -> &str {
        "auto-denylist"
    }

    fn register(&mut self) {}

    fn start(&mut self, _token: &CancelToken) {}

    fn stop(&mut self) {}

    fn report(&mut self, address: IpAddr, _ttl: std::time::Duration, reason: &str) {
        self.list.hold(address, reason, self.clock.tick_count());
    }

    fn can_retract(&self) -> bool {
        true
    }

    fn retract(&mut self, address: IpAddr) {
        self.list.release(address);
    }
}

/// `AutoDenylist.Configure`.
pub fn configure(w: &mut World) {
    let s: AutoDenylistSettings = super::load_settings(w, PATH, AutoDenylistSettings::default());
    let list = w.resources.get_mut::<AutoDenylist>().clone();
    let enabled = s.enabled && s.duration > TimeSpan::ZERO && s.max_entries > 0;
    {
        let mut st = list.lock();
        st.enabled = enabled;
        if !enabled {
            return;
        }
        st.duration_ms = s.duration.total_milliseconds() as i64;
        st.max_entries = s.max_entries as usize;
    }
    let clock = Clock::default();
    let r = w.resources.get_mut::<NetworkRegistrations>();
    r.filters.push(Box::new(AutoDenylistFilter {
        list: list.clone(),
        clock: clock.clone(),
        last_sweep: 0,
    }));
    r.reporters
        .push(Box::new(AutoDenylistReporter { list, clock }));
}

#[cfg(test)]
mod tests {
    use super::*;

    const DURATION: i64 = 900_000;
    const NOW: i64 = 1_000_000;

    fn list(max: usize) -> AutoDenylist {
        let l = AutoDenylist::default();
        l.load_for_testing(true, DURATION, max);
        l
    }

    fn ip(n: u32) -> IpAddr {
        format!("198.51.100.{n}").parse().unwrap()
    }

    #[test]
    fn behavioral_detection_is_held_then_lapses() {
        let l = list(1024);
        assert!(l.hold(ip(10), ban_reasons::INVALID_SEED, NOW));
        assert!(l.is_denied(ip(10), NOW));
        assert!(l.is_denied(ip(10), NOW + DURATION - 1));
        assert!(!l.is_denied(ip(10), NOW + DURATION));
    }

    #[test]
    fn manual_and_list_verdicts_are_not_held() {
        let l = list(1024);
        assert!(!l.hold(ip(11), ban_reasons::MANUAL, NOW));
        assert!(!l.hold(ip(12), ban_reasons::BLOCKLIST, NOW));
        assert!(!l.is_denied(ip(11), NOW) && !l.is_denied(ip(12), NOW));
        assert_eq!(l.count(), 0);
    }

    #[test]
    fn repeat_detection_does_not_extend_the_hold() {
        let l = list(1024);
        l.hold(ip(13), ban_reasons::SILENT_CONNECT, NOW);
        l.hold(ip(13), ban_reasons::SILENT_CONNECT, NOW + DURATION - 1);
        assert_eq!(l.count(), 1);
        assert!(l.is_denied(ip(13), NOW + DURATION - 1));
        assert!(!l.is_denied(ip(13), NOW + DURATION + 1));
    }

    #[test]
    fn ring_and_set_stay_in_step() {
        let l = list(4);
        for i in 0..8 {
            l.hold(ip(70 + i), ban_reasons::INVALID_SEED, NOW);
        }
        assert_eq!(l.count(), 4);
        assert_eq!(l.count(), l.ring_count());
        l.release(ip(71));
        assert_eq!(l.count(), 3);
        assert_eq!(l.count(), l.ring_count());
        l.drain(NOW + DURATION + 1);
        assert_eq!((l.count(), l.ring_count()), (0, 0));
    }

    #[test]
    fn fills_exactly_to_a_non_power_of_two_cap() {
        let l = list(100);
        for i in 0..120 {
            l.hold(ip(i), ban_reasons::INVALID_SEED, NOW);
        }
        assert_eq!((l.count(), l.ring_count()), (100, 100));
        l.drain(NOW + DURATION + 1);
        assert_eq!((l.count(), l.ring_count()), (0, 0));
    }

    #[test]
    fn release_then_re_hold_is_not_retired_by_the_stale_record() {
        let l = list(1024);
        l.hold(ip(15), ban_reasons::RATE_LIMIT, NOW);
        l.release(ip(15));
        l.hold(ip(15), ban_reasons::RATE_LIMIT, NOW + DURATION - 1);
        assert!(l.is_denied(ip(15), NOW + DURATION + 1));
        assert_eq!(l.ring_count(), 1);
    }

    #[test]
    fn release_drops_the_hold_immediately() {
        let l = list(1024);
        l.hold(ip(14), ban_reasons::RATE_LIMIT, NOW);
        l.release(ip(14));
        assert!(!l.is_denied(ip(14), NOW));
    }

    #[test]
    fn cap_is_enforced_and_lapsed_entries_are_reclaimed_first() {
        let l = list(4);
        for i in 0..10 {
            l.hold(ip(100 + i), ban_reasons::INVALID_SEED, NOW);
        }
        assert_eq!(l.count(), 4);
        assert!(l.is_denied(ip(100), NOW) && !l.is_denied(ip(109), NOW));

        let l = list(2);
        l.hold(ip(20), ban_reasons::INVALID_SEED, NOW);
        l.hold(ip(21), ban_reasons::INVALID_SEED, NOW);
        let later = NOW + DURATION + 1;
        assert!(l.hold(ip(22), ban_reasons::INVALID_SEED, later));
        assert!(l.is_denied(ip(22), later));
    }

    #[test]
    fn disabled_holds_nothing() {
        let l = AutoDenylist::default();
        l.load_for_testing(false, DURATION, 10);
        assert!(!l.hold(ip(1), ban_reasons::RATE_LIMIT, NOW));
        assert!(!l.is_denied(ip(1), NOW));
    }
}
