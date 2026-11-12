//! CrowdSec reporting (configuration, alert shapes, alert client, reporter): contributes the bans this shard decides
//! on to a CrowdSec local API (LAPI) as alerts, and retracts them on an unban. Delivery runs on its own thread;
//! `report` only enqueues (bounded, drop on overflow).
//!
//! The HTTP client is `ureq` with rustls and the pure Rust RustCrypto provider (`crate::http_tls`).

use std::net::IpAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc::{Receiver, RecvTimeoutError, SyncSender, sync_channel};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use indexmap::IndexMap;
use muo_net::CancelToken;
use muo_net::bans::{BanReporter, ban_reasons};
use muo_primitives::{DateTime, TimeSpan};
use muo_server::World;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use super::NetworkRegistrations;

const PATH: &str = "Configuration/crowdsec.json";
const USER_AGENT: &str = "crowdsec/muo-watcher-1.0";

/// `CrowdSecSettings`.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct CrowdSecSettings {
    #[serde(rename = "lapiUrl")]
    pub lapi_url: String,
    #[serde(rename = "machineId")]
    pub machine_id: String,
    pub password: String,
    pub origin: String,
    #[serde(
        rename = "manualBanDuration",
        with = "muo_data::json::converters::timespan"
    )]
    pub manual_ban_duration: TimeSpan,
    #[serde(
        rename = "flushInterval",
        with = "muo_data::json::converters::timespan"
    )]
    pub flush_interval: TimeSpan,
    #[serde(rename = "maxQueue")]
    pub max_queue: i32,
}

impl Default for CrowdSecSettings {
    fn default() -> Self {
        Self {
            lapi_url: "http://127.0.0.1:8080".to_owned(),
            machine_id: String::new(),
            password: String::new(),
            origin: "muo".to_owned(),
            manual_ban_duration: TimeSpan::from_hours(168),
            flush_interval: TimeSpan::from_secs(1),
            max_queue: 10000,
        }
    }
}

impl CrowdSecSettings {
    /// `ReportingEnabled`.
    pub fn reporting_enabled(&self) -> bool {
        !self.machine_id.trim().is_empty() && !self.password.trim().is_empty()
    }
}

// ---- CrowdSecAlert: the wire shapes ----

#[derive(Clone, Debug, Serialize)]
pub struct CrowdSecAlert {
    pub scenario: String,
    pub scenario_hash: String,
    pub scenario_version: String,
    pub message: String,
    pub events_count: i32,
    pub start_at: String,
    pub stop_at: String,
    pub capacity: i32,
    pub leakspeed: String,
    pub simulated: bool,
    pub events: Vec<serde_json::Value>,
    pub remediation: bool,
    pub source: CrowdSecSource,
    pub decisions: Vec<CrowdSecDecisionDto>,
}

#[derive(Clone, Debug, Serialize)]
pub struct CrowdSecSource {
    pub scope: String,
    pub value: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct CrowdSecDecisionDto {
    pub origin: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub scope: String,
    pub value: String,
    pub duration: String,
    pub scenario: String,
}

#[derive(Serialize)]
struct CrowdSecLoginRequest<'a> {
    machine_id: &'a str,
    password: &'a str,
}

#[derive(Deserialize)]
struct CrowdSecLoginResponse {
    #[serde(default)]
    token: Option<String>,
    #[serde(default)]
    expire: Option<String>,
}

// ---- CrowdSecAlertClient ----

/// `ICrowdSecAlertClient`.
pub trait CrowdSecAlertClient: Send {
    fn post_alerts(
        &mut self,
        alerts: &[CrowdSecAlert],
        deadline: Option<Instant>,
    ) -> Result<(), String>;
    fn delete_decisions(
        &mut self,
        origin: &str,
        ip: IpAddr,
        deadline: Option<Instant>,
    ) -> Result<(), String>;
}

/// `CrowdSecAlertClient`: logs in to the LAPI as a watcher and sends with the bearer token.
pub struct HttpAlertClient {
    agent: ureq::Agent,
    base: String,
    machine_id: String,
    password: String,
    token: Option<String>,
    token_expires: DateTime,
}

/// `BuildDeleteQuery(origin, ip)`.
pub fn build_delete_query(origin: &str, ip: IpAddr) -> String {
    format!(
        "/v1/decisions?origin={}&ip={}",
        escape(origin),
        escape(&ip.to_string())
    )
}

/// `Uri.EscapeDataString`: everything but the unreserved characters.
fn escape(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.' | b'~') {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

/// The scheme and authority of the LAPI url: the requests are host-root-relative (`/v1/alerts`).
fn base_of(url: &str) -> Result<String, String> {
    let (scheme, rest) = url
        .split_once("://")
        .ok_or("LAPI url is not an absolute uri")?;
    if scheme.is_empty() || rest.is_empty() {
        return Err("LAPI url is not an absolute uri".into());
    }
    let authority = rest.split(['/', '?', '#']).next().unwrap_or(rest);
    Ok(format!("{scheme}://{authority}"))
}

impl HttpAlertClient {
    pub fn new(settings: &CrowdSecSettings) -> Result<Self, String> {
        let base = base_of(&settings.lapi_url)?; // fails loud on malformed url
        let agent: ureq::Agent = ureq::Agent::config_builder()
            .tls_config(crate::http_tls::tls_config())
            .timeout_global(Some(Duration::from_secs(30)))
            .http_status_as_error(false)
            .user_agent(USER_AGENT)
            .build()
            .into();
        Ok(Self {
            agent,
            base,
            machine_id: settings.machine_id.clone(),
            password: settings.password.clone(),
            token: None,
            token_expires: DateTime::from_ticks(0),
        })
    }

    fn ensure_auth(&mut self) -> Result<(), String> {
        let now = DateTime::utc_now();
        if self.token.is_some() && now < self.token_expires - TimeSpan::from_mins(1) {
            return Ok(());
        }
        let req = CrowdSecLoginRequest {
            machine_id: &self.machine_id,
            password: &self.password,
        };
        let mut resp = self
            .agent
            .post(format!("{}/v1/watchers/login", self.base))
            .send_json(&req)
            .map_err(|e| e.to_string())?;
        if !resp.status().is_success() {
            return Err(format!("login returned {}", resp.status()));
        }
        let login: CrowdSecLoginResponse =
            resp.body_mut().read_json().map_err(|e| e.to_string())?;
        self.token = Some(login.token.ok_or("CrowdSec login returned no token.")?);
        // LAPI returns an RFC3339 expiry.
        self.token_expires = login
            .expire
            .as_deref()
            .and_then(DateTime::parse_iso)
            .unwrap_or_else(|| now + TimeSpan::from_hours(1));
        Ok(())
    }

    /// `SendWithRetryAsync`: one re-login and retry when the token is rejected mid-flight.
    fn send_with_retry(
        &mut self,
        deadline: Option<Instant>,
        build: impl Fn(
            &ureq::Agent,
            &str,
            &str,
        ) -> Result<ureq::http::Response<ureq::Body>, ureq::Error>,
    ) -> Result<(), String> {
        let _ = deadline;
        self.ensure_auth()?;
        for attempt in 0..2 {
            let token = self.token.clone().unwrap_or_default();
            let resp = build(&self.agent, &self.base, &token).map_err(|e| e.to_string())?;
            let status = resp.status();
            if status == 401 && attempt == 0 {
                self.token = None;
                self.ensure_auth()?;
                continue;
            }
            return if status.is_success() {
                Ok(())
            } else {
                Err(format!("LAPI returned {status}"))
            };
        }
        Err("unreachable".into())
    }
}

impl CrowdSecAlertClient for HttpAlertClient {
    fn post_alerts(
        &mut self,
        alerts: &[CrowdSecAlert],
        deadline: Option<Instant>,
    ) -> Result<(), String> {
        if alerts.is_empty() {
            return Ok(());
        }
        self.send_with_retry(deadline, |agent, base, token| {
            agent
                .post(format!("{base}/v1/alerts"))
                .header("Authorization", format!("Bearer {token}"))
                .send_json(alerts)
        })
    }

    fn delete_decisions(
        &mut self,
        origin: &str,
        ip: IpAddr,
        deadline: Option<Instant>,
    ) -> Result<(), String> {
        let query = build_delete_query(origin, ip);
        self.send_with_retry(deadline, |agent, base, token| {
            agent
                .delete(format!("{base}{query}"))
                .header("Authorization", format!("Bearer {token}"))
                .call()
        })
    }
}

// ---- CrowdSecReporter ----

/// `ReportItem`.
#[derive(Clone, Debug)]
pub struct ReportItem {
    pub ip: IpAddr,
    pub ttl: Duration,
    pub reason: String,
    pub retract: bool,
}

/// `FormatTimestamp`: `yyyy-MM-ddTHH:mm:ss.fffZ`, invariant.
pub fn format_timestamp(t: DateTime) -> String {
    let (y, mo, d) = t.ymd();
    format!(
        "{y:04}-{mo:02}-{d:02}T{:02}:{:02}:{:02}.{:03}Z",
        t.hour(),
        t.minute(),
        t.second(),
        t.millisecond()
    )
}

/// `FormatDuration`: whole seconds, at least one.
pub fn format_duration(ttl: Duration) -> String {
    format!("{}s", (ttl.as_secs() as i64).max(1))
}

/// `BuildAlerts(items, settings, nowUtc)`: one alert per address (the last item for an address wins).
pub fn build_alerts(
    items: &[ReportItem],
    settings: &CrowdSecSettings,
    now_utc: DateTime,
) -> Vec<CrowdSecAlert> {
    let mut by_ip: IndexMap<String, &ReportItem> = IndexMap::new();
    for item in items {
        by_ip.insert(item.ip.to_string(), item);
    }
    let timestamp = format_timestamp(now_utc);
    by_ip
        .into_iter()
        .map(|(value, item)| {
            let ttl = if item.reason == ban_reasons::MANUAL || item.ttl.is_zero() {
                Duration::from_secs_f64(settings.manual_ban_duration.total_seconds().max(0.0))
            } else {
                item.ttl
            };
            let scenario = format!("{}/{}", settings.origin, item.reason);
            CrowdSecAlert {
                scenario: scenario.clone(),
                scenario_hash: String::new(),
                scenario_version: "1.0".into(),
                message: format!("muo {} ban for {value}", item.reason),
                events_count: 1,
                start_at: timestamp.clone(),
                stop_at: timestamp.clone(),
                capacity: 1,
                leakspeed: "0s".into(),
                simulated: false,
                events: Vec::new(),
                remediation: true,
                source: CrowdSecSource {
                    scope: "Ip".into(),
                    value: value.clone(),
                },
                decisions: vec![CrowdSecDecisionDto {
                    origin: settings.origin.clone(),
                    kind: "ban".into(),
                    scope: "Ip".into(),
                    value,
                    duration: format_duration(ttl),
                    scenario,
                }],
            }
        })
        .collect()
}

/// Bounded retry for transient LAPI failures during a drain send; distinct from the single 401 re-login.
const RETRY_DELAYS: [Duration; 2] = [Duration::from_secs(1), Duration::from_secs(2)];

/// Counters the owner can read.
#[derive(Default)]
pub struct Counters {
    pub dropped: AtomicUsize,
    pub send_failures: AtomicUsize,
}

/// What the drain thread hands back when it exits: the queue (for the flush on stop) and the client.
type Drained = (Receiver<ReportItem>, Box<dyn CrowdSecAlertClient>);

/// `CrowdSecReporter`.
pub struct CrowdSecReporter {
    settings: CrowdSecSettings,
    /// Settings were injected: `Register` does not read the file.
    injected: bool,
    base_directory: std::path::PathBuf,
    sender: Option<SyncSender<ReportItem>>,
    receiver: Option<Receiver<ReportItem>>,
    client: Option<Box<dyn CrowdSecAlertClient>>,
    counters: Arc<Counters>,
    cancel: Arc<AtomicBool>,
    drain: Option<JoinHandle<Drained>>,
}

impl CrowdSecReporter {
    pub fn new(base_directory: std::path::PathBuf) -> Self {
        Self {
            settings: CrowdSecSettings::default(),
            injected: false,
            base_directory,
            sender: None,
            receiver: None,
            client: None,
            counters: Arc::default(),
            cancel: Arc::default(),
            drain: None,
        }
    }

    /// The embedding constructor: an injected client and settings, with the queue already made.
    pub fn with_client(settings: CrowdSecSettings, client: Box<dyn CrowdSecAlertClient>) -> Self {
        let (tx, rx) = sync_channel(settings.max_queue.max(1) as usize);
        Self {
            settings,
            injected: true,
            base_directory: std::path::PathBuf::new(),
            sender: Some(tx),
            receiver: Some(rx),
            client: Some(client),
            counters: Arc::default(),
            cancel: Arc::default(),
            drain: None,
        }
    }

    pub fn counters(&self) -> Arc<Counters> {
        self.counters.clone()
    }

    fn enqueue(&self, item: ReportItem) {
        let ok = self
            .sender
            .as_ref()
            .is_some_and(|s| s.try_send(item).is_ok());
        if !ok {
            self.counters.dropped.fetch_add(1, Ordering::SeqCst);
        }
    }

    /// `FlushRemainingOnStop`: what is still queued, sent within a few seconds.
    fn flush_remaining(
        &mut self,
        rx: Receiver<ReportItem>,
        mut client: Box<dyn CrowdSecAlertClient>,
    ) {
        self.sender = None; // TryComplete
        let (mut reports, mut retracts) = (Vec::new(), Vec::new());
        while let Ok(item) = rx.try_recv() {
            if item.retract {
                retracts.push(item);
            } else {
                reports.push(item);
            }
        }
        if reports.is_empty() && retracts.is_empty() {
            return;
        }
        let deadline = Instant::now() + Duration::from_secs(3);
        if !reports.is_empty() {
            let alerts = build_alerts(&reports, &self.settings, DateTime::utc_now());
            if let Err(e) = client.post_alerts(&alerts, Some(deadline)) {
                warn!("CrowdSec flush-on-stop reports failed: {e}");
                record_send_failure(&self.counters, alerts.len());
            }
        }
        let mut seen = std::collections::HashSet::new();
        for r in &retracts {
            if Instant::now() >= deadline {
                break; // out of budget; the rest self-heal via ManualBanDuration
            }
            if !seen.insert(r.ip.to_string()) {
                continue;
            }
            if let Err(e) = client.delete_decisions(&self.settings.origin, r.ip, Some(deadline)) {
                warn!("CrowdSec flush-on-stop retract failed for {}: {e}", r.ip);
                record_send_failure(&self.counters, 1);
            }
        }
    }
}

fn record_send_failure(counters: &Counters, items: usize) {
    let total = counters.send_failures.fetch_add(1, Ordering::SeqCst) + 1;
    warn!(
        "CrowdSec contribution batch dropped after retries ({items} item(s)); total dropped batches: {total}"
    );
}

/// `SendWithBoundedRetryAsync`.
fn send_with_bounded_retry(
    cancel: &AtomicBool,
    mut send: impl FnMut() -> Result<(), String>,
) -> bool {
    let mut attempt = 0;
    loop {
        let Err(e) = send() else { return true };
        let delay = RETRY_DELAYS
            .get(attempt)
            .filter(|_| !cancel.load(Ordering::SeqCst));
        let Some(delay) = delay else {
            warn!(
                "CrowdSec send failed after {} attempt(s); giving up: {e}",
                attempt + 1
            );
            return false;
        };
        warn!(
            "CrowdSec send failed (attempt {}); retrying in {delay:?}: {e}",
            attempt + 1
        );
        std::thread::sleep(*delay);
        attempt += 1;
    }
}

/// `DrainLoop`: waits for an item, lets a burst coalesce, then sends reports and retractions.
fn drain_loop(
    rx: Receiver<ReportItem>,
    mut client: Box<dyn CrowdSecAlertClient>,
    settings: CrowdSecSettings,
    counters: Arc<Counters>,
    cancel: Arc<AtomicBool>,
) -> Drained {
    let flush = Duration::from_secs_f64(settings.flush_interval.total_seconds().max(0.0));
    while !cancel.load(Ordering::SeqCst) {
        let first = match rx.recv_timeout(Duration::from_millis(100)) {
            Ok(i) => i,
            Err(RecvTimeoutError::Timeout) => continue,
            Err(RecvTimeoutError::Disconnected) => break,
        };
        // Coalesce a burst before flushing.
        let end = Instant::now() + flush;
        while Instant::now() < end && !cancel.load(Ordering::SeqCst) {
            std::thread::sleep(Duration::from_millis(20));
        }
        let (mut reports, mut retracts) = (Vec::new(), Vec::new());
        let mut next = Some(first);
        while let Some(item) = next.take().or_else(|| rx.try_recv().ok()) {
            if item.retract {
                retracts.push(item);
            } else {
                reports.push(item);
            }
        }
        if !reports.is_empty() {
            let alerts = build_alerts(&reports, &settings, DateTime::utc_now());
            if !send_with_bounded_retry(&cancel, || client.post_alerts(&alerts, None)) {
                record_send_failure(&counters, alerts.len());
            }
        }
        for r in &retracts {
            if !send_with_bounded_retry(&cancel, || {
                client.delete_decisions(&settings.origin, r.ip, None)
            }) {
                record_send_failure(&counters, 1);
            }
        }
    }
    (rx, client)
}

impl BanReporter for CrowdSecReporter {
    fn name(&self) -> &str {
        "crowdsec"
    }

    fn register(&mut self) {
        if self.injected {
            return;
        }
        let path = self.base_directory.join(PATH);
        self.settings = match muo_data::json::deserialize_file::<CrowdSecSettings>(&path) {
            Ok(Some(s)) => s,
            Ok(None) => {
                let s = CrowdSecSettings::default();
                if let Err(e) = muo_data::json::serialize_file(&path, &s) {
                    warn!("Could not write {PATH}: {e}");
                }
                s
            }
            Err(e) => {
                warn!("{PATH}: {e}; using defaults");
                CrowdSecSettings::default()
            }
        };
    }

    fn start(&mut self, _token: &CancelToken) {
        if !self.settings.reporting_enabled() {
            info!("CrowdSec reporter disabled (machineId/password empty in crowdsec.json)");
            return;
        }
        if self.client.is_none() {
            match HttpAlertClient::new(&self.settings) {
                Ok(c) => self.client = Some(Box::new(c)),
                Err(e) => {
                    warn!("CrowdSec reporter failed to start: {e}");
                    return;
                }
            }
        }
        if self.receiver.is_none() {
            let (tx, rx) = sync_channel(self.settings.max_queue.max(1) as usize);
            self.sender = Some(tx);
            self.receiver = Some(rx);
        }
        let (Some(rx), Some(client)) = (self.receiver.take(), self.client.take()) else {
            return;
        };
        let (settings, counters, cancel) = (
            self.settings.clone(),
            self.counters.clone(),
            self.cancel.clone(),
        );
        self.drain = Some(std::thread::spawn(move || {
            drain_loop(rx, client, settings, counters, cancel)
        }));
    }

    fn stop(&mut self) {
        self.cancel.store(true, Ordering::SeqCst);
        // The flush below reads the queue, so wait for the drain to actually exit first. Without a drain
        // (never started) the queue and client are still here and are flushed directly.
        let drained = match self.drain.take() {
            None => self.receiver.take().zip(self.client.take()),
            Some(handle) => {
                let end = Instant::now() + Duration::from_secs(2);
                while !handle.is_finished() && Instant::now() < end {
                    std::thread::sleep(Duration::from_millis(20));
                }
                if handle.is_finished() {
                    handle.join().ok()
                } else {
                    None
                }
            }
        };
        if let Some((rx, client)) = drained {
            self.flush_remaining(rx, client);
        }
    }

    fn report(&mut self, address: IpAddr, ttl: Duration, reason: &str) {
        self.enqueue(ReportItem {
            ip: address,
            ttl,
            reason: reason.to_owned(),
            retract: false,
        });
    }

    fn can_retract(&self) -> bool {
        true
    }

    fn retract(&mut self, address: IpAddr) {
        self.enqueue(ReportItem {
            ip: address,
            ttl: Duration::ZERO,
            reason: "retract".to_owned(),
            retract: true,
        });
    }
}

/// `CrowdSecReporter.Configure`: `BanChannel.Register(new CrowdSecReporter())`.
pub fn configure(w: &mut World) {
    let base = w.core.base_directory.clone();
    w.resources
        .get_mut::<NetworkRegistrations>()
        .reporters
        .push(Box::new(CrowdSecReporter::new(base)));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn settings() -> CrowdSecSettings {
        CrowdSecSettings {
            max_queue: 4,
            ..CrowdSecSettings::default()
        }
    }

    fn item(ip: &str, reason: &str) -> ReportItem {
        ReportItem {
            ip: ip.parse().unwrap(),
            ttl: Duration::from_secs(3600),
            reason: reason.to_owned(),
            retract: false,
        }
    }

    #[test]
    fn build_alerts_dedups_by_ip() {
        let epoch = DateTime::from_unix_seconds(0).unwrap();
        let alerts = build_alerts(
            &[
                item("1.1.1.1", "rate-limit"),
                item("1.1.1.1", "rate-limit"),
                item("2.2.2.2", "rate-limit"),
            ],
            &settings(),
            epoch,
        );
        assert_eq!(alerts.len(), 2);
        assert!(alerts.iter().all(|a| a.decisions.len() == 1));
        assert_eq!(alerts[0].source.value, "1.1.1.1");
        assert_eq!(alerts[1].source.value, "2.2.2.2");
    }

    #[test]
    fn scenario_comes_from_the_reason_and_origin_from_settings() {
        let epoch = DateTime::from_unix_seconds(0).unwrap();
        let alerts = build_alerts(&[item("3.3.3.3", "manual")], &settings(), epoch);
        let d = &alerts[0].decisions[0];
        assert_eq!(alerts[0].scenario, "muo/manual");
        assert_eq!(
            (
                d.origin.as_str(),
                d.kind.as_str(),
                d.scope.as_str(),
                d.value.as_str()
            ),
            ("muo", "ban", "Ip", "3.3.3.3")
        );
        // a manual ban uses the configured manual duration, not the item's
        assert_eq!(d.duration, "604800s");
        let b = build_alerts(&[item("5.5.5.5", "blocklist")], &settings(), epoch);
        assert_eq!(b[0].scenario, "muo/blocklist");
        assert_eq!(b[0].decisions[0].duration, "3600s");
    }

    #[test]
    fn the_payload_carries_the_scenario_fields_lapi_dereferences() {
        let epoch = DateTime::from_unix_seconds(0).unwrap();
        let alerts = build_alerts(&[item("9.9.9.9", "rate-limit")], &settings(), epoch);
        let v = serde_json::to_value(&alerts).unwrap();
        let o = &v[0];
        assert!(o["scenario_hash"].is_string() && o["scenario_version"].is_string());
        assert_eq!(o["capacity"], 1);
        assert_eq!(o["start_at"], "1970-01-01T00:00:00.000Z");
        assert_eq!(o["decisions"][0]["type"], "ban");
    }

    #[test]
    fn durations_are_at_least_one_second() {
        assert_eq!(format_duration(Duration::from_millis(10)), "1s");
        assert_eq!(format_duration(Duration::from_secs(90)), "90s");
    }

    #[test]
    fn delete_query_escapes_the_values() {
        assert_eq!(
            build_delete_query("model rn", "1.2.3.4".parse().unwrap()),
            "/v1/decisions?origin=model%20rn&ip=1.2.3.4"
        );
        assert_eq!(
            build_delete_query("o", "::1".parse().unwrap()),
            "/v1/decisions?origin=o&ip=%3A%3A1"
        );
    }

    #[test]
    fn a_full_queue_drops_and_counts() {
        struct Nop;
        impl CrowdSecAlertClient for Nop {
            fn post_alerts(
                &mut self,
                _: &[CrowdSecAlert],
                _: Option<Instant>,
            ) -> Result<(), String> {
                Ok(())
            }
            fn delete_decisions(
                &mut self,
                _: &str,
                _: IpAddr,
                _: Option<Instant>,
            ) -> Result<(), String> {
                Ok(())
            }
        }
        let mut r = CrowdSecReporter::with_client(settings(), Box::new(Nop));
        for i in 0..6 {
            r.report(
                format!("1.1.1.{i}").parse().unwrap(),
                Duration::ZERO,
                "rate-limit",
            );
        }
        assert_eq!(r.counters().dropped.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn the_lapi_base_is_scheme_and_authority() {
        assert_eq!(
            base_of("http://127.0.0.1:8080/some/path").unwrap(),
            "http://127.0.0.1:8080"
        );
        assert_eq!(
            base_of("https://lapi.example").unwrap(),
            "https://lapi.example"
        );
        assert!(base_of("not a url").is_err());
    }

    // Reporter (stop and format), configuration and alert client tests.
    use std::sync::Mutex as StdMutex;

    fn reporter_settings() -> CrowdSecSettings {
        CrowdSecSettings {
            machine_id: "shard".to_owned(),
            password: "secret".to_owned(),
            origin: "muo".to_owned(),
            manual_ban_duration: TimeSpan::from_hours(168),
            ..CrowdSecSettings::default()
        }
    }

    #[derive(Default)]
    struct Recorded {
        posted: Vec<Vec<CrowdSecAlert>>,
        deleted: Vec<IpAddr>,
    }

    struct Recording(Arc<StdMutex<Recorded>>);

    impl CrowdSecAlertClient for Recording {
        fn post_alerts(
            &mut self,
            alerts: &[CrowdSecAlert],
            _: Option<Instant>,
        ) -> Result<(), String> {
            self.0.lock().unwrap().posted.push(alerts.to_vec());
            Ok(())
        }
        fn delete_decisions(
            &mut self,
            _: &str,
            ip: IpAddr,
            _: Option<Instant>,
        ) -> Result<(), String> {
            self.0.lock().unwrap().deleted.push(ip);
            Ok(())
        }
    }

    struct Throwing;

    impl CrowdSecAlertClient for Throwing {
        fn post_alerts(&mut self, _: &[CrowdSecAlert], _: Option<Instant>) -> Result<(), String> {
            Err("simulated LAPI outage".to_owned())
        }
        fn delete_decisions(
            &mut self,
            _: &str,
            _: IpAddr,
            _: Option<Instant>,
        ) -> Result<(), String> {
            Ok(())
        }
    }

    #[test]
    fn build_alerts_scenario_from_reason_blocklist() {
        let alerts = build_alerts(
            &[item("5.5.5.5", "blocklist")],
            &reporter_settings(),
            DateTime::from_unix_seconds(0).unwrap(),
        );
        assert_eq!(alerts.len(), 1);
        assert_eq!(alerts[0].scenario, "muo/blocklist");
        assert_eq!(alerts[0].decisions[0].scenario, "muo/blocklist");
    }

    #[test]
    fn format_timestamp_is_iso8601() {
        // the Rust formatter has no culture: the epoch prints the same under fi-FI, th-TH and ar-SA
        assert_eq!(
            format_timestamp(DateTime::from_unix_seconds(0).unwrap()),
            "1970-01-01T00:00:00.000Z"
        );
    }

    #[test]
    fn format_duration_uses_seconds_floors_at_one() {
        assert_eq!(format_duration(Duration::from_secs(3600)), "3600s");
        assert_eq!(format_duration(Duration::ZERO), "1s");
        assert_eq!(format_duration(Duration::from_millis(10)), "1s");
    }

    #[test]
    fn stop_flushes_queued_reports_via_client() {
        let rec = Arc::new(StdMutex::new(Recorded::default()));
        let mut r =
            CrowdSecReporter::with_client(reporter_settings(), Box::new(Recording(rec.clone())));

        r.report(
            "6.6.6.6".parse().unwrap(),
            Duration::from_secs(3600),
            "rate-limit",
        );
        r.stop();

        let rec = rec.lock().unwrap();
        assert_eq!(rec.posted.len(), 1);
        assert_eq!(rec.posted[0].len(), 1);
        assert_eq!(rec.posted[0][0].source.value, "6.6.6.6");
        assert_eq!(r.counters().send_failures.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn stop_flushes_queued_retracts_via_client() {
        let rec = Arc::new(StdMutex::new(Recorded::default()));
        let mut r =
            CrowdSecReporter::with_client(reporter_settings(), Box::new(Recording(rec.clone())));

        r.retract("8.8.8.8".parse().unwrap());
        r.stop();

        assert_eq!(
            rec.lock().unwrap().deleted,
            vec!["8.8.8.8".parse::<IpAddr>().unwrap()]
        );
        assert_eq!(r.counters().send_failures.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn stop_when_flush_send_fails_counts_send_failure() {
        let mut r = CrowdSecReporter::with_client(reporter_settings(), Box::new(Throwing));

        r.report(
            "7.7.7.7".parse().unwrap(),
            Duration::from_secs(3600),
            "rate-limit",
        );
        r.stop();

        assert_eq!(r.counters().send_failures.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn stop_with_empty_queue_does_not_invoke_client_or_fail() {
        let rec = Arc::new(StdMutex::new(Recorded::default()));
        let mut r =
            CrowdSecReporter::with_client(reporter_settings(), Box::new(Recording(rec.clone())));

        r.stop();

        assert!(rec.lock().unwrap().posted.is_empty());
        assert_eq!(r.counters().send_failures.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn crowdsec_settings_round_trip_through_json() {
        let original = CrowdSecSettings {
            lapi_url: "http://10.0.0.5:9090".to_owned(),
            machine_id: "shard".to_owned(),
            password: "secret".to_owned(),
            origin: "muo".to_owned(),
            manual_ban_duration: TimeSpan::from_hours(168),
            flush_interval: TimeSpan::from_secs(1),
            max_queue: 10000,
        };
        let json = muo_data::json::to_string(&original).unwrap();

        // camelCase names must be present or the case-sensitive reader binds nothing
        for key in [
            "lapiUrl",
            "machineId",
            "password",
            "origin",
            "manualBanDuration",
            "flushInterval",
            "maxQueue",
        ] {
            assert!(json.contains(&format!("\"{key}\"")), "{key} in {json}");
        }

        let restored: CrowdSecSettings = muo_data::json::from_str(&json).unwrap();
        assert_eq!(restored.lapi_url, original.lapi_url);
        assert_eq!(restored.machine_id, original.machine_id);
        assert_eq!(restored.password, original.password);
        assert_eq!(restored.origin, original.origin);
        assert_eq!(restored.manual_ban_duration, original.manual_ban_duration);
        assert_eq!(restored.flush_interval, original.flush_interval);
        assert_eq!(restored.max_queue, original.max_queue);
        assert!(restored.reporting_enabled());
    }

    #[test]
    fn crowdsec_settings_defaults_are_reporting_disabled_local_loopback() {
        let s = CrowdSecSettings::default();
        assert_eq!(s.lapi_url, "http://127.0.0.1:8080");
        assert_eq!(s.machine_id, "");
        assert_eq!(s.password, "");
        assert_eq!(s.origin, "muo");
        assert_eq!(s.manual_ban_duration, TimeSpan::from_hours(168));
        assert_eq!(s.flush_interval, TimeSpan::from_secs(1));
        assert_eq!(s.max_queue, 10000);
        assert!(!s.reporting_enabled()); // empty machineId/password => inert
    }

    #[test]
    fn build_delete_query_escapes_origin() {
        // origin is operator-controlled config, so it is untrusted input going into the URL
        assert_eq!(
            build_delete_query("modern uo/test&x=1", "1.2.3.4".parse().unwrap()),
            "/v1/decisions?origin=modern%20uo%2Ftest%26x%3D1&ip=1.2.3.4"
        );
    }

    #[test]
    fn build_delete_query_ipv6_is_escaped() {
        assert_eq!(
            build_delete_query("muo", "2001:db8::1".parse().unwrap()),
            "/v1/decisions?origin=muo&ip=2001%3Adb8%3A%3A1"
        );
    }

    #[test]
    fn user_agent_is_prefixed_for_the_watcher_profile() {
        // LAPI watcher profile matches the crowdsec/ prefix and answers 401 without it
        assert!(USER_AGENT.starts_with("crowdsec/"));
    }
}
