//! The address-policy half of the network layer: Firewall, ManualAllowlist, LoginAllowlist,
//! Blocklist, AutoDenylist, CrowdSec and BanExemptions.
//!
//! The accept path (`muo_net`'s `ConnectionFilters` and `BanChannel`) runs inside the transport and owns its
//! filters, so a filter cannot reach the `World`. What both sides read lives behind an `Arc`: the firewall's range
//! index, the allowlists, the blocklist snapshot. Content registers its filters and reporters in
//! [`NetworkRegistrations`] during `Configure`; `muo::start_network` hands them to the transport. A ban a filter
//! decides on (a blocklist hit) goes through [`BanReportQueue`], which the loop drains into `BanChannel.Report` after
//! every network slice.
//!
//! The pollers wait out a save (`while (World.Saving || WorldState == PendingSave)`), reading the
//! world's state mirror ([`StopFlag::wait_while_saving`]).

pub mod autodenylist;
pub mod blocklist;
pub mod crowdsec;
pub mod exemptions;
pub mod firewall;
pub mod login_allowlist;
pub mod manual_allowlist;
pub mod snapshot;

use std::net::IpAddr;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use muo_net::bans::{BanReporter, ExemptFn};
use muo_net::connection_filters::ConnectionFilter;
use muo_server::World;

/// Filters, reporters and the exemption `muo::start_network` registers on the transport.
#[derive(Default)]
pub struct NetworkRegistrations {
    pub filters: Vec<Box<dyn ConnectionFilter>>,
    pub reporters: Vec<Box<dyn BanReporter>>,
    pub is_exempt: Option<ExemptFn>,
}

/// `(address, ttl, reason)` bans decided on the accept path, for `BanChannel.Report`.
#[derive(Clone, Default)]
pub struct BanReportQueue(pub Arc<Mutex<Vec<(IpAddr, Duration, String)>>>);

impl BanReportQueue {
    pub fn push(&self, ip: IpAddr, ttl: Duration, reason: &str) {
        if let Ok(mut q) = self.0.lock() {
            q.push((ip, ttl, reason.to_owned()));
        }
    }

    pub fn take(&self) -> Vec<(IpAddr, Duration, String)> {
        self.0
            .lock()
            .map(|mut q| std::mem::take(&mut *q))
            .unwrap_or_default()
    }
}

/// `Core.ClosingTokenSource`, for the pollers: set when the server shuts down.
#[derive(Clone, Default)]
pub struct StopFlag(pub Arc<AtomicBool>);

impl StopFlag {
    pub fn is_set(&self) -> bool {
        self.0.load(Ordering::SeqCst)
    }

    /// Waits `total`, or until the flag is set (checked every 100 ms). True when the flag ended the wait.
    pub fn sleep(&self, total: Duration) -> bool {
        let end = Instant::now() + total;
        while Instant::now() < end {
            if self.is_set() {
                return true;
            }
            std::thread::sleep(Duration::from_millis(100).min(end - Instant::now()));
        }
        self.is_set()
    }
}

impl StopFlag {
    /// `while (World.Saving || World.WorldState == WorldState.PendingSave) await Task.Delay(1s)`: true when the flag
    /// ended the wait.
    pub fn wait_while_saving(&self, state: &std::sync::atomic::AtomicU8) -> bool {
        use muo_server::world::WorldState;
        let saving = || {
            let s = state.load(Ordering::SeqCst);
            s == WorldState::Saving as u8 || s == WorldState::PendingSave as u8
        };
        while saving() {
            if self.sleep(Duration::from_secs(1)) {
                return true;
            }
        }
        self.is_set()
    }
}

/// `Core.TickCount` for the accept-path state: milliseconds since the first call.
#[derive(Clone)]
pub struct Clock(Instant);

impl Default for Clock {
    fn default() -> Self {
        Self(Instant::now())
    }
}

impl Clock {
    pub fn tick_count(&self) -> i64 {
        self.0.elapsed().as_millis() as i64
    }
}

pub fn unix_now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs() as i64)
}

/// A configured path: relative resolves against the base directory, never the working directory.
pub fn resolve_path(w: &World, configured: &str) -> PathBuf {
    let p = PathBuf::from(configured);
    if p.is_absolute() {
        p
    } else {
        w.core.base_directory.join(p)
    }
}

/// The shared shutdown flag; the first call subscribes to the shutdown event.
pub fn stop_flag(w: &mut World) -> StopFlag {
    if let Some(f) = w.resources.get::<StopFlag>() {
        return f.clone();
    }
    let flag = StopFlag::default();
    w.resources.insert(flag.clone());
    let f = flag.clone();
    w.events
        .shutdown
        .subscribe(move |_, _| f.0.store(true, Ordering::SeqCst));
    flag
}

/// The `Configure` of the address-policy files, in the order the filters are consulted.
pub fn configure(w: &mut World) {
    stop_flag(w);
    firewall::configure(w);
    blocklist::configure(w);
    autodenylist::configure(w);
    crowdsec::configure(w);
    login_allowlist::configure(w);
    exemptions::configure(w);
}

/// The `Initialize` of the address-policy files.
pub fn initialize(w: &mut World) {
    manual_allowlist::initialize(w);
    login_allowlist::initialize(w);
}

/// `JsonConfig.Deserialize<T>(path)` or, when the file is missing, the default written back.
pub fn load_settings<T>(w: &World, relative: &str, default: T) -> T
where
    T: serde::Serialize + serde::de::DeserializeOwned,
{
    let path = w.core.base_directory.join(relative);
    match muo_data::json::deserialize_file::<T>(&path) {
        Ok(Some(s)) => s,
        Ok(None) => {
            if let Err(e) = muo_data::json::serialize_file(&path, &default) {
                tracing::warn!("Could not write {relative}: {e}");
            }
            default
        }
        Err(e) => {
            tracing::warn!("{relative}: {e}; using defaults");
            default
        }
    }
}

#[cfg(test)]
mod wait_tests {
    use super::*;
    use muo_server::world::WorldState;
    use std::sync::atomic::AtomicU8;

    #[test]
    fn a_poller_waits_out_a_save_and_leaves_at_shutdown() {
        let stop = StopFlag::default();
        let running = AtomicU8::new(WorldState::Running as u8);
        assert!(!stop.wait_while_saving(&running)); // no save: no wait
        let pending = AtomicU8::new(WorldState::PendingSave as u8);
        stop.0.store(true, Ordering::SeqCst);
        assert!(stop.wait_while_saving(&pending)); // a save, then shutdown ends the wait
    }
}
