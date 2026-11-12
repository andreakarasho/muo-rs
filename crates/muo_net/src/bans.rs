//! The ban channel, its configuration, ban reasons and the reporter interface.
//!
//! The channel only fans locally-decided bans out to external reporters (CrowdSec); enforcement is the
//! accept path's [`crate::connection_filters::ConnectionFilters`]. The channel state is owned by the
//! transport.

use crate::CancelToken;
use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::Path;
use std::time::Duration;
use tracing::{debug, info, warn};

/// The `reason` slugs contributed through the channel. Policy keys off these.
pub mod ban_reasons {
    /// An operator banned this address explicitly. Never exempt, never auto-denied.
    pub const MANUAL: &str = "manual";
    pub const RATE_LIMIT: &str = "rate-limit";
    /// Matched the reputation blocklist. Enforced by its own filter, not by behaviour.
    pub const BLOCKLIST: &str = "blocklist";
    /// Reaped without ever sending a byte.
    pub const SILENT_CONNECT: &str = "silent-connect";
    /// Opened with a zero seed, which no real client sends.
    pub const INVALID_SEED: &str = "invalid-seed";
    /// Positively identified as another protocol entirely.
    pub const FOREIGN_PROTOCOL: &str = "foreign-protocol";

    /// Verdicts the shard reached by watching the connection. Only these may be exempted, and only these
    /// feed the local denylist. Opt-in rather than "everything except manual", so a reason added later
    /// escalates normally instead of silently inheriting an exemption.
    pub fn is_behavioral(reason: &str) -> bool {
        matches!(
            reason,
            RATE_LIMIT | SILENT_CONNECT | INVALID_SEED | FOREIGN_PROTOCOL
        )
    }
}

/// A contribution sink behind [`BanChannel`] (`IBanReporter`).
pub trait BanReporter {
    /// Stable id for logging/config (e.g. `crowdsec`).
    fn name(&self) -> &str;
    /// Reads configuration. No network or file I/O here.
    fn register(&mut self);
    /// Starts background delivery. The token is set on shutdown.
    fn start(&mut self, token: &CancelToken);
    /// Flushes and tears down background delivery.
    fn stop(&mut self);
    /// Enqueues a ban contribution. MUST be non-blocking: it may only enqueue (bounded, drop-on-overflow).
    /// A zero `ttl` means the reporter's default duration.
    fn report(&mut self, address: IpAddr, ttl: Duration, reason: &str);
    /// True if this reporter can retract a previously-reported ban.
    fn can_retract(&self) -> bool;
    /// Enqueues a retraction (e.g. a manual unban). No-op if unsupported.
    fn retract(&mut self, address: IpAddr);
}

/// Optional content-supplied exemption; true drops the contribution before any reporter sees it. This
/// withholds escalation only. An implementation that ignores the reason would silently swallow manual bans.
pub type ExemptFn = Box<dyn Fn(IpAddr, &str) -> bool>;

#[derive(Default)]
pub struct BanChannel {
    reporters: Vec<Box<dyn BanReporter>>,
    pub is_exempt: Option<ExemptFn>,
}

impl BanChannel {
    pub fn reporters(&self) -> &[Box<dyn BanReporter>] {
        &self.reporters
    }

    /// Registers a contribution sink from content. Idempotent by name; configures the reporter immediately
    /// so it is ready before [`BanChannel::start`].
    pub fn register(&mut self, mut reporter: Box<dyn BanReporter>) {
        if self.reporters.iter().any(|r| r.name() == reporter.name()) {
            return;
        }

        reporter.register();
        info!("Ban channel registered reporter '{}'", reporter.name());
        self.reporters.push(reporter);
    }

    /// `ConfigureForTesting`.
    pub fn configure_for_testing(&mut self, reporters: Vec<Box<dyn BanReporter>>) {
        self.reporters = reporters;
    }

    pub fn start(&mut self, token: &CancelToken) {
        for r in &mut self.reporters {
            // A broken contribution path must not crash boot; enforcement is local and unaffected.
            if catch_unwind(AssertUnwindSafe(|| r.start(token))).is_err() {
                tracing::error!(
                    "Ban reporter '{}' failed to start; continuing without it",
                    r.name()
                );
            }
        }
    }

    pub fn stop(&mut self) {
        for r in &mut self.reporters {
            if catch_unwind(AssertUnwindSafe(|| r.stop())).is_err() {
                warn!("Ban reporter '{}' threw while stopping", r.name());
            }
        }
    }

    /// Fans a locally-decided ban out to every reporter. Never throws.
    pub fn report(&mut self, ip: IpAddr, ttl: Duration, reason: &str) {
        if let Some(exempt) = &self.is_exempt
            && exempt(ip, reason)
        {
            debug!("{ip} not contributed ('{reason}'): exempt");
            return;
        }

        for r in &mut self.reporters {
            if catch_unwind(AssertUnwindSafe(|| r.report(ip, ttl, reason))).is_err() {
                warn!("Ban reporter '{}' threw during Report", r.name());
            }
        }
    }

    /// Fans a retraction (manual unban) out to every retract-capable reporter.
    pub fn retract(&mut self, ip: IpAddr) {
        for r in &mut self.reporters {
            if !r.can_retract() {
                continue;
            }
            if catch_unwind(AssertUnwindSafe(|| r.retract(ip))).is_err() {
                warn!("Ban reporter '{}' threw during Retract", r.name());
            }
        }
    }
}

// --- BanConfiguration --------------------------------------------------------------------------------

/// `System.TimeSpan` as `System.Text.Json` writes it: the invariant "c" format `[-][d.]hh:mm:ss[.fffffff]`.
mod timespan {
    use serde::{Deserialize, Deserializer, Serializer};
    use std::time::Duration;

    pub fn format(d: Duration) -> String {
        let total = d.as_secs();
        let (days, h, m, s) = (
            total / 86400,
            total / 3600 % 24,
            total / 60 % 60,
            total % 60,
        );
        let mut out = if days > 0 {
            format!("{days}.{h:02}:{m:02}:{s:02}")
        } else {
            format!("{h:02}:{m:02}:{s:02}")
        };
        let ticks = d.subsec_nanos() / 100;
        if ticks > 0 {
            out.push_str(&format!(".{ticks:07}"));
        }
        out
    }

    /// `TimeSpan.Parse` for the shapes a config file uses: `d`, `[d.]hh:mm[:ss[.f]]`. Negative gives zero.
    pub fn parse(s: &str) -> Option<Duration> {
        let s = s.trim();
        if s.starts_with('-') {
            return Some(Duration::ZERO);
        }

        let (head, frac) = match s.rsplit_once('.') {
            Some((h, f))
                if h.contains(':') && f.chars().all(|c| c.is_ascii_digit()) && !f.is_empty() =>
            {
                (h, Some(f))
            }
            _ => (s, None),
        };

        let (days, clock) = match head.split_once('.') {
            Some((d, c)) if c.contains(':') => (d.parse::<u64>().ok()?, c),
            _ if !head.contains(':') => {
                return Some(Duration::from_secs(head.parse::<u64>().ok()? * 86400));
            }
            _ => (0, head),
        };

        let parts: Vec<u64> = clock
            .split(':')
            .map(|p| p.parse().ok())
            .collect::<Option<_>>()?;
        let (h, m, sec) = match parts[..] {
            [h, m] => (h, m, 0),
            [h, m, s] => (h, m, s),
            _ => return None,
        };

        let mut nanos = 0u32;
        if let Some(f) = frac {
            let digits: String = f.chars().chain(std::iter::repeat('0')).take(7).collect();
            nanos = digits.parse::<u32>().ok()? * 100;
        }
        Some(Duration::new(days * 86400 + h * 3600 + m * 60 + sec, nanos))
    }

    pub fn serialize<S: Serializer>(d: &Duration, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&format(*d))
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Duration, D::Error> {
        let text = String::deserialize(d)?;
        parse(&text).ok_or_else(|| serde::de::Error::custom(format!("invalid TimeSpan '{text}'")))
    }
}

/// Ban-channel policy (`BanSettings`): which reporters receive contributions, and how auto-detections are
/// handled. Field order is the order the file is written in; keep it for byte-identical bans.json files.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct BanSettings {
    /// Whether IP rate-limiter trips are contributed to reporters. They never enter the local firewall set.
    #[serde(rename = "reportRateLimitTrips")]
    pub report_rate_limit_trips: bool,

    /// Duration reported for an auto-detected (rate-limit) ban.
    #[serde(rename = "autoBanDuration", with = "timespan")]
    pub auto_ban_duration: Duration,

    /// Whether behavioural detections are contributed to reporters. Those connections are disconnected
    /// either way; this only controls escalation. Keyed on bytes-received, never elapsed time.
    #[serde(rename = "reportBadConnects")]
    pub report_bad_connects: bool,

    #[serde(rename = "badConnectDuration", with = "timespan")]
    pub bad_connect_duration: Duration,
}

impl Default for BanSettings {
    fn default() -> Self {
        Self {
            report_rate_limit_trips: true,
            auto_ban_duration: Duration::from_secs(4 * 3600),
            report_bad_connects: true,
            bad_connect_duration: Duration::from_secs(4 * 3600),
        }
    }
}

const BAN_CONFIG_PATH: &str = "Configuration/bans.json";

impl BanSettings {
    /// `BanConfiguration.Configure`: reads `Configuration/bans.json` under `base_dir`; a missing file writes
    /// the local-only, fail-open template so operators have something to edit.
    pub fn configure(base_dir: &Path) -> Result<BanSettings, String> {
        let path = base_dir.join(BAN_CONFIG_PATH);

        if path.exists() {
            // `JsonConfig.Deserialize` throws on a bad file, which stops startup
            return std::fs::read_to_string(&path)
                .map_err(|e| e.to_string())
                .and_then(|t| Self::from_json(t.trim_start_matches('\u{feff}')))
                .map_err(|e| format!("Failed to deserialize {}: {e}.", path.display()));
        }

        let settings = BanSettings {
            report_rate_limit_trips: true,
            auto_ban_duration: Duration::from_secs(4 * 3600),
            ..Default::default()
        };
        if let Err(e) = settings.save(&path) {
            warn!("{}: could not write template: {e}", path.display());
        }
        Ok(settings)
    }

    /// Reads JSON the way `JsonConfig` does: comments and trailing commas allowed, keys case-sensitive here
    /// as written by [`BanSettings::to_json`]. (shortcut: comment/trailing-comma tolerance is only
    /// line comments and `,}` / `,]`, enough for hand-edited files)
    pub fn from_json(text: &str) -> Result<BanSettings, String> {
        // Two passes: the second removes commas that only became trailing once comments were gone
        let cleaned = strip_json_extras(&strip_json_extras(text));
        serde_json::from_str(&cleaned).map_err(|e| e.to_string())
    }

    /// Indented with two spaces; `Environment.NewLine` line endings (CRLF on Windows), no trailing newline.
    pub fn to_json(&self) -> String {
        let text = serde_json::to_string_pretty(self).unwrap_or_default();
        if cfg!(windows) {
            text.replace('\n', "\r\n")
        } else {
            text
        }
    }

    fn save(&self, path: &Path) -> std::io::Result<()> {
        if path.exists() {
            std::fs::remove_file(path)?;
        }
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        std::fs::write(path, self.to_json())
    }
}

/// Removes `//` and `/* */` comments and trailing commas outside strings.
fn strip_json_extras(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len());
    let (mut i, mut in_str) = (0, false);
    while i < chars.len() {
        let c = chars[i];
        if in_str {
            out.push(c);
            if c == '\\' && i + 1 < chars.len() {
                out.push(chars[i + 1]);
                i += 1;
            } else if c == '"' {
                in_str = false;
            }
        } else if c == '"' {
            in_str = true;
            out.push(c);
        } else if c == '/' && chars.get(i + 1) == Some(&'/') {
            while i < chars.len() && chars[i] != '\n' {
                i += 1;
            }
            continue;
        } else if c == '/' && chars.get(i + 1) == Some(&'*') {
            i += 2;
            while i + 1 < chars.len() && !(chars[i] == '*' && chars[i + 1] == '/') {
                i += 1;
            }
            i += 2;
            continue;
        } else if c == ',' {
            let next = chars[i + 1..].iter().find(|ch| !ch.is_whitespace());
            if !matches!(next, Some('}') | Some(']')) {
                out.push(c);
            }
        } else {
            out.push(c);
        }
        i += 1;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    #[derive(Default)]
    struct Log {
        reports: Vec<(IpAddr, Duration, String)>,
        retractions: Vec<IpAddr>,
    }

    struct Fake {
        log: Rc<RefCell<Log>>,
        panic_on_report: bool,
        name: &'static str,
    }

    impl Fake {
        fn new(name: &'static str) -> (Box<Self>, Rc<RefCell<Log>>) {
            let log = Rc::new(RefCell::new(Log::default()));
            (
                Box::new(Fake {
                    log: log.clone(),
                    panic_on_report: false,
                    name,
                }),
                log,
            )
        }
    }

    impl BanReporter for Fake {
        fn name(&self) -> &str {
            self.name
        }
        fn register(&mut self) {}
        fn start(&mut self, _t: &CancelToken) {}
        fn stop(&mut self) {}
        fn report(&mut self, address: IpAddr, ttl: Duration, reason: &str) {
            if self.panic_on_report {
                panic!("boom");
            }
            self.log
                .borrow_mut()
                .reports
                .push((address, ttl, reason.to_string()));
        }
        fn can_retract(&self) -> bool {
            true
        }
        fn retract(&mut self, address: IpAddr) {
            self.log.borrow_mut().retractions.push(address);
        }
    }

    fn ip(s: &str) -> IpAddr {
        s.parse().unwrap()
    }

    #[test]
    fn report_fans_out_to_all_reporters() {
        let (a, la) = Fake::new("a");
        let (b, lb) = Fake::new("b");
        let mut ch = BanChannel::default();
        ch.configure_for_testing(vec![a, b]);
        ch.report(ip("1.2.3.4"), Duration::from_secs(3600), "rate-limit");
        assert_eq!(la.borrow().reports.len(), 1);
        assert_eq!(lb.borrow().reports.len(), 1);
        assert_eq!(la.borrow().reports[0].2, "rate-limit");
    }

    #[test]
    fn report_swallows_reporter_failure() {
        let (mut bad, _) = Fake::new("bad");
        bad.panic_on_report = true;
        let (good, lg) = Fake::new("good");
        let mut ch = BanChannel::default();
        ch.configure_for_testing(vec![bad, good]);
        ch.report(ip("1.2.3.4"), Duration::from_secs(3600), "manual");
        assert_eq!(lg.borrow().reports.len(), 1);
    }

    #[test]
    fn report_suppressed_for_exempt_address() {
        let (r, l) = Fake::new("fake");
        let mut ch = BanChannel::default();
        ch.configure_for_testing(vec![r]);
        let exempt = ip("203.0.113.7");
        ch.is_exempt = Some(Box::new(move |a, _| a == exempt));

        ch.report(exempt, Duration::from_secs(3600), "rate-limit");
        assert!(l.borrow().reports.is_empty());

        ch.report(ip("203.0.113.8"), Duration::from_secs(3600), "rate-limit");
        assert_eq!(l.borrow().reports.len(), 1);
    }

    #[test]
    fn retract_reaches_retract_capable_reporters() {
        let (r, l) = Fake::new("fake");
        let mut ch = BanChannel::default();
        ch.configure_for_testing(vec![r]);
        ch.retract(ip("9.9.9.9"));
        assert_eq!(l.borrow().retractions.len(), 1);
    }

    #[test]
    fn register_is_idempotent_by_name() {
        let (a, _) = Fake::new("same");
        let (b, _) = Fake::new("same");
        let mut ch = BanChannel::default();
        ch.register(a);
        ch.register(b);
        assert_eq!(ch.reporters().len(), 1);
    }

    #[test]
    fn behavioral_reasons() {
        assert!(ban_reasons::is_behavioral("rate-limit"));
        assert!(ban_reasons::is_behavioral("foreign-protocol"));
        assert!(!ban_reasons::is_behavioral(ban_reasons::MANUAL));
        assert!(!ban_reasons::is_behavioral(ban_reasons::BLOCKLIST));
    }

    #[test]
    fn settings_round_trip_through_json() {
        let original = BanSettings {
            report_rate_limit_trips: false,
            auto_ban_duration: Duration::from_secs(2 * 3600),
            ..Default::default()
        };
        let json = original.to_json();
        assert!(json.contains("\"reportRateLimitTrips\""));
        assert!(json.contains("\"autoBanDuration\""));
        assert_eq!(BanSettings::from_json(&json).unwrap(), original);
    }

    #[test]
    fn default_file_matches_distribution_bans_json_byte_for_byte() {
        // Distribution/Configuration/bans.json as the server writes it (CRLF on Windows).
        let nl = if cfg!(windows) { "\r\n" } else { "\n" };
        let expected = [
            "{",
            "  \"reportRateLimitTrips\": true,",
            "  \"autoBanDuration\": \"04:00:00\",",
            "  \"reportBadConnects\": true,",
            "  \"badConnectDuration\": \"04:00:00\"",
            "}",
        ]
        .join(nl);
        assert_eq!(BanSettings::default().to_json(), expected);
    }

    #[test]
    fn timespan_forms() {
        assert_eq!(
            timespan::parse("04:00:00"),
            Some(Duration::from_secs(14400))
        );
        assert_eq!(
            timespan::parse("1.02:03:04"),
            Some(Duration::from_secs(86400 + 7384))
        );
        assert_eq!(
            timespan::parse("00:00:01.5"),
            Some(Duration::from_millis(1500))
        );
        assert_eq!(timespan::parse("garbage"), None);
        assert_eq!(
            timespan::format(Duration::from_secs(86400 + 7384)),
            "1.02:03:04"
        );
        assert_eq!(
            timespan::format(Duration::from_millis(1500)),
            "00:00:01.5000000"
        );
    }

    #[test]
    fn json_tolerates_comments_and_trailing_commas() {
        let s =
            BanSettings::from_json("{ // note\n \"reportBadConnects\": false, /* x */ }").unwrap();
        assert!(!s.report_bad_connects);
    }

    #[test]
    fn missing_file_writes_template_and_existing_is_read() {
        let dir = std::env::temp_dir().join(format!("muo_net_bans_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let s = BanSettings::configure(&dir).unwrap();
        assert_eq!(s, BanSettings::default());
        let written = std::fs::read_to_string(dir.join("Configuration/bans.json")).unwrap();
        assert_eq!(written, s.to_json());
        std::fs::write(
            dir.join("Configuration/bans.json"),
            "{\"reportBadConnects\": false}",
        )
        .unwrap();
        assert!(!BanSettings::configure(&dir).unwrap().report_bad_connects);
        std::fs::write(dir.join("Configuration/bans.json"), "{ nope").unwrap();
        assert!(BanSettings::configure(&dir).is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
