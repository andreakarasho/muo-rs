//! The firewall (Firewall, IFirewallEntry, BaseFirewallEntry, CidrFirewallEntry, SingleIpFirewallEntry,
//! FirewallSettings, FirewallConnectionFilter): the operator's list of blocked addresses
//! and ranges, persisted to `Configuration/firewall.json`, some with an expiry.
//!
//! The entries live in the world (the game loop adds, removes and expires them); the accept path reads a derived
//! [`SortedRangeIndex`] behind an `Arc`, rebuilt whenever the entries change.

use std::net::IpAddr;
use std::sync::{Arc, RwLock};

use muo_net::connection_filters::ConnectionFilter;
use muo_primitives::collections::SortedRangeIndex;
use muo_primitives::dotnet::parse_ip_address;
use muo_primitives::net::{
    create_cidr_address, to_ip_addr, to_u128, try_parse_cidr_range, write_mapped_ipv6,
};
use muo_primitives::{DateTime, TimeSpan};
use muo_server::World;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use super::NetworkRegistrations;

const PATH: &str = "Configuration/firewall.json";
const LEGACY_PATH: &str = "firewall.cfg";

/// `IFirewallEntry`: an inclusive range of addresses in IPv6 form.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct FirewallEntry {
    pub min: u128,
    pub max: u128,
}

impl FirewallEntry {
    /// `new SingleIpFirewallEntry(address)`.
    pub fn single(address: IpAddr) -> Self {
        let v = to_u128(address);
        Self { min: v, max: v }
    }

    /// `new CidrFirewallEntry(min, max)`.
    pub fn range(min: IpAddr, max: IpAddr) -> Self {
        Self {
            min: to_u128(min),
            max: to_u128(max),
        }
    }

    /// `new CidrFirewallEntry(ipAddress, prefixLength)`.
    pub fn cidr(address: IpAddr, mut prefix_length: i32) -> Self {
        if address.is_ipv4() {
            prefix_length += 96; // 32 -> 128
        }
        let bytes = write_mapped_ipv6(address);
        Self {
            min: create_cidr_address(bytes, prefix_length, false),
            max: create_cidr_address(bytes, prefix_length, true),
        }
    }

    /// `new CidrFirewallEntry("a.b.c.d/n")`: `None` when it is not a valid address or CIDR.
    pub fn parse_cidr(text: &str) -> Option<Self> {
        try_parse_cidr_range(text).map(|(min, max)| Self { min, max })
    }

    /// `MinIpAddress.ToIpAddress()` of a single-address entry (`MinIpAddress == MaxIpAddress`), else `None`.
    pub fn single_address(&self) -> Option<IpAddr> {
        (self.min == self.max).then(|| to_ip_addr(self.min, false))
    }

    /// `IsBlocked(address)`.
    pub fn is_blocked(&self, address: IpAddr) -> bool {
        let a = to_u128(address);
        a >= self.min && a <= self.max
    }

    /// `IComparable.CompareTo`: by minimum, then the wider range first; 0 is the same range.
    pub fn compare_to(&self, other: &Self) -> std::cmp::Ordering {
        self.min
            .cmp(&other.min)
            .then_with(|| other.max.cmp(&self.max))
    }

    /// `ToFirewallEntry(string)`: `a-b` is a range, `a/n` a CIDR, anything else a single address.
    pub fn parse(entry: &str) -> Option<Self> {
        if let Some((a, b)) = entry.split_once('-') {
            return Some(Self::range(parse_ip_address(a)?, parse_ip_address(b)?));
        }
        if entry.contains('/') {
            return Self::parse_cidr(entry);
        }
        Some(Self::single(parse_ip_address(entry)?))
    }
}

impl std::fmt::Display for FirewallEntry {
    /// `BaseFirewallEntry.ToString()`.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.min == self.max {
            write!(f, "{}", to_ip_addr(self.min, false))
        } else {
            write!(
                f,
                "{}-{}",
                to_ip_addr(self.min, false),
                to_ip_addr(self.max, false)
            )
        }
    }
}

/// `DateTime?` through System.Text.Json: a string that is not an ISO 8601 date-time fails the whole file.
fn strict_expires<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Option<String>, D::Error> {
    let s = Option::<String>::deserialize(d)?;
    if let Some(v) = &s
        && DateTime::parse_json_with_local(v, crate::systems::world_saves::local_time::offset_at)
            .is_none()
    {
        return Err(serde::de::Error::custom(format!(
            "The JSON value could not be converted to System.DateTime: '{v}'"
        )));
    }
    Ok(s)
}

/// `FirewallEntryRecord`.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct FirewallEntryRecord {
    pub value: String,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "strict_expires"
    )]
    pub expires: Option<String>,
}

/// `FirewallSettings`.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct FirewallSettings {
    pub entries: Vec<FirewallEntryRecord>,
}

struct Stored {
    entry: FirewallEntry,
    /// Absolute expiry tick (`Core.TickCount`); permanent entries have none.
    expires_at: Option<i64>,
}

/// The firewall's state, held in the world's resources.
#[derive(Default)]
pub struct Firewall {
    entries: Vec<Stored>,
    index: Arc<RwLock<SortedRangeIndex<u128>>>,
    dirty: bool,
    configured: bool,
}

impl Firewall {
    /// `FirewallSetCount`.
    pub fn count(&self) -> usize {
        self.entries.len()
    }

    /// `ReadFirewallSet`: the entries in insertion order.
    pub fn entries(&self) -> impl Iterator<Item = FirewallEntry> + '_ {
        self.entries.iter().map(|s| s.entry)
    }

    /// `IsBlocked(address)`.
    pub fn is_blocked(&self, address: IpAddr) -> bool {
        !self.entries.is_empty()
            && self
                .index
                .read()
                .is_ok_and(|i| i.contains(to_u128(address)))
    }

    fn rebuild_index(&self) {
        let ranges: Vec<(u128, u128)> = self
            .entries
            .iter()
            .map(|s| (s.entry.min, s.entry.max))
            .collect();
        if let Ok(mut i) = self.index.write() {
            *i = SortedRangeIndex::from_ranges(ranges);
        }
    }

    fn index_of(&self, entry: &FirewallEntry) -> Option<usize> {
        self.entries
            .iter()
            .position(|s| s.entry.compare_to(entry).is_eq())
    }

    /// `Add(entry, ttl, persist)`: false when the same range is already there.
    pub fn add(
        &mut self,
        entry: FirewallEntry,
        ttl: TimeSpan,
        persist: bool,
        tick_count: i64,
    ) -> bool {
        if self.index_of(&entry).is_some() {
            return false;
        }
        let expires_at = (ttl > TimeSpan::ZERO)
            .then(|| tick_count.wrapping_add(ttl.total_milliseconds() as i64));
        self.entries.push(Stored { entry, expires_at });
        self.rebuild_index();
        if persist {
            self.dirty = true;
        }
        true
    }

    /// `Remove(entry)`.
    pub fn remove(&mut self, entry: &FirewallEntry) -> bool {
        let Some(i) = self.index_of(entry) else {
            return false;
        };
        self.entries.remove(i);
        self.rebuild_index();
        self.dirty = true;
        true
    }

    /// `ExpireEntries(nowTicks)`.
    pub fn expire_entries(&mut self, now_ticks: i64) {
        let before = self.entries.len();
        self.entries
            .retain(|s| s.expires_at.is_none_or(|e| e.wrapping_sub(now_ticks) > 0));
        if self.entries.len() != before {
            self.rebuild_index();
            self.dirty = true;
        }
    }

    /// `LoadFrom(settings)`.
    pub fn load_from(&mut self, settings: &FirewallSettings, now: DateTime, tick_count: i64) {
        for record in &settings.entries {
            let Some(entry) = FirewallEntry::parse(&record.value) else {
                warn!("Ignoring unparseable firewall entry \"{}\"", record.value);
                continue;
            };
            let mut ttl = TimeSpan::ZERO;
            if let Some(expires) = record.expires.as_deref().and_then(|v| {
                DateTime::parse_json_with_local(
                    v,
                    crate::systems::world_saves::local_time::offset_at,
                )
            }) {
                ttl = expires.since(now);
                if ttl <= TimeSpan::ZERO {
                    continue; // already expired
                }
            }
            self.add(entry, ttl, false, tick_count);
        }
    }

    /// `ToSettings()`: `expires` is now plus the ticks left, both read at the same instant.
    pub fn to_settings(&self, now: DateTime, now_ticks: i64) -> FirewallSettings {
        FirewallSettings {
            entries: self
                .entries
                .iter()
                .map(|s| FirewallEntryRecord {
                    value: s.entry.to_string(),
                    expires: s.expires_at.map(|e| {
                        now.add_milliseconds(e.wrapping_sub(now_ticks) as f64)
                            .to_json_string()
                    }),
                })
                .collect(),
        }
    }
}

/// `FirewallConnectionFilter`: asks the shared index.
struct FirewallConnectionFilter {
    index: Arc<RwLock<SortedRangeIndex<u128>>>,
}

impl ConnectionFilter for FirewallConnectionFilter {
    fn name(&self) -> &str {
        "firewall"
    }

    // Firewall.Configure() owns loading and registration.
    fn register(&mut self) {}

    // Nothing to hydrate: the set loads at Configure and is maintained by a main-loop timer.
    fn start(&mut self, _token: &muo_net::CancelToken) {}

    // Firewall.Save() runs from the shutdown event.
    fn stop(&mut self) {}

    fn should_deny(&mut self, address: IpAddr) -> Result<bool, String> {
        Ok(self
            .index
            .read()
            .map_err(|e| e.to_string())?
            .contains(to_u128(address)))
    }
}

/// `Firewall.Configure()`.
pub fn configure(w: &mut World) {
    if w.resources.get_mut::<Firewall>().configured {
        return;
    }
    w.resources.get_mut::<Firewall>().configured = true;

    let path = w.core.base_directory.join(PATH);
    let now = w.now;
    let ticks = w.tick_count;
    if path.exists() {
        match muo_data::json::deserialize_file::<FirewallSettings>(&path) {
            Ok(Some(settings)) => w
                .resources
                .get_mut::<Firewall>()
                .load_from(&settings, now, ticks),
            Ok(None) => {}
            Err(e) => warn!("{PATH}: {e}"),
        }
    } else if let Some(legacy) = resolve_legacy_cfg_path(w) {
        migrate_legacy_cfg(w, &legacy);
        save(w); // materialize firewall.json; the .cfg is no longer read after this
        try_mark_legacy_cfg_migrated(&legacy);
    }

    // Main-thread maintenance: expire TTLs and flush pending writes. No background thread.
    let every = TimeSpan::from_secs(30);
    muo_server::timer::start(w, every, every, 0, maintenance);
    w.events.shutdown.subscribe(|w, _| save(w));

    // Expose the set to the accept path. Everything else (gump, commands, persistence) keeps using the Firewall API
    // directly; only the per-connection question goes through the filter registry.
    let index = w.resources.get_mut::<Firewall>().index.clone();
    w.resources
        .get_mut::<NetworkRegistrations>()
        .filters
        .push(Box::new(FirewallConnectionFilter { index }));
}

fn maintenance(w: &mut World) {
    let ticks = w.tick_count;
    let f = w.resources.get_mut::<Firewall>();
    f.expire_entries(ticks);
    if f.dirty {
        save(w);
    }
}

/// `Firewall.Save()`: a temporary file, then an atomic swap.
pub fn save(w: &mut World) {
    let (now, ticks) = (w.now, w.tick_count);
    let f = w.resources.get_mut::<Firewall>();
    f.dirty = false;
    let settings = f.to_settings(now, ticks);
    let path = w.core.base_directory.join(PATH);
    let mut tmp = path.as_os_str().to_owned();
    tmp.push(".tmp");
    let tmp = std::path::PathBuf::from(tmp);
    let result = muo_data::json::serialize_file(&tmp, &settings)
        .map_err(|e| e.to_string())
        .and_then(|()| std::fs::rename(&tmp, &path).map_err(|e| e.to_string()));
    if let Err(e) = result {
        warn!("Could not save {PATH}: {e}");
    }
}

fn resolve_legacy_cfg_path(w: &World) -> Option<std::path::PathBuf> {
    let under_base = w.core.base_directory.join(LEGACY_PATH);
    if under_base.is_file() {
        return Some(under_base);
    }
    std::path::Path::new(LEGACY_PATH)
        .is_file()
        .then(|| LEGACY_PATH.into())
}

fn migrate_legacy_cfg(w: &mut World, legacy: &std::path::Path) {
    let Ok(text) = std::fs::read_to_string(legacy) else {
        return;
    };
    let ticks = w.tick_count;
    let f = w.resources.get_mut::<Firewall>();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if line.contains(['*', 'X', 'x', '?']) {
            warn!("Legacy firewall entry \"{line}\" ignored during migration");
            continue;
        }
        if let Some(entry) = FirewallEntry::parse(line) {
            f.add(entry, TimeSpan::ZERO, false, ticks);
        }
    }
    info!(
        "Migrated {} entr(ies) from legacy firewall.cfg to firewall.json",
        f.count()
    );
}

fn try_mark_legacy_cfg_migrated(legacy: &std::path::Path) {
    let mut to = legacy.as_os_str().to_owned();
    to.push(".migrated");
    if let Err(e) = std::fs::rename(legacy, &to) {
        warn!(
            "Could not rename migrated legacy firewall file \"{}\": {e}",
            legacy.display()
        );
    }
}

/// `Firewall.Add(entry, ttl, persist)`.
pub fn add(w: &mut World, entry: FirewallEntry, ttl: TimeSpan, persist: bool) -> bool {
    let ticks = w.tick_count;
    w.resources
        .get_mut::<Firewall>()
        .add(entry, ttl, persist, ticks)
}

/// `Firewall.Remove(entry)`.
pub fn remove(w: &mut World, entry: &FirewallEntry) -> bool {
    w.resources.get_mut::<Firewall>().remove(entry)
}

/// `Firewall.IsBlocked(address)`.
pub fn is_blocked(w: &mut World, address: IpAddr) -> bool {
    w.resources.get_mut::<Firewall>().is_blocked(address)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ip(s: &str) -> IpAddr {
        s.parse().unwrap()
    }

    #[test]
    fn single_ip_entries() {
        for (text, back) in [
            ("192.168.1.1", "192.168.1.1"),
            ("::ffff:192.168.1.1", "192.168.1.1"),
            (
                "ae45:c5c7:9372:2d3a:413c:6490:017d:2c18",
                "ae45:c5c7:9372:2d3a:413c:6490:17d:2c18",
            ),
        ] {
            let e = FirewallEntry::single(ip(text));
            assert_eq!(e.min, e.max);
            assert_eq!(to_ip_addr(e.min, false), ip(back));
        }
    }

    #[test]
    fn cidr_entries() {
        for (cidr, start, end) in [
            ("192.168.1.1/24", "192.168.1.0", "192.168.1.255"),
            ("::ffff:10.25.3.250/112", "10.25.0.0", "10.25.255.255"),
            ("::ffff:10.25.5.250/124", "10.25.5.240", "10.25.5.255"),
            ("::ffff:192.168.1.1/120", "192.168.1.0", "192.168.1.255"),
            (
                "d15e:d490:03cd:f9e1:95d8:8413:e6b8:e226/88",
                "D15E:D490:03CD:F9E1:95D8:8400::",
                "D15E:D490:03CD:F9E1:95D8:84FF:FFFF:FFFF",
            ),
            (
                "2001:4860:4860::8888/32",
                "2001:4860:0000:0000:0000:0000:0000:0000",
                "2001:4860:FFFF:FFFF:FFFF:FFFF:FFFF:FFFF",
            ),
        ] {
            let e = FirewallEntry::parse_cidr(cidr).unwrap();
            assert_eq!(to_ip_addr(e.min, false), ip(start), "{cidr}");
            assert_eq!(to_ip_addr(e.max, false), ip(end), "{cidr}");
        }
    }

    #[test]
    fn legacy_ip_forms_and_strict_expires() {
        // IPAddress.Parse: "127.1" is 127.0.0.1, "010.0.0.1" is 8.0.0.1 (octal)
        assert_eq!(
            FirewallEntry::parse("127.1").unwrap().to_string(),
            "127.0.0.1"
        );
        assert_eq!(
            FirewallEntry::parse("010.0.0.1-0x0a.0.0.9")
                .unwrap()
                .to_string(),
            "8.0.0.1-10.0.0.9"
        );
        assert!(
            FirewallEntry::parse("127.1")
                .unwrap()
                .is_blocked(ip("127.0.0.1"))
        );
        let ok: FirewallSettings =
            serde_json::from_str(r#"{"entries":[{"value":"1.2.3.4","expires":"2026-11-05T09:30:15.25Z"},{"value":"5.6.7.8","expires":null}]}"#).unwrap();
        assert_eq!(ok.entries.len(), 2);
        // System.Text.Json refuses the whole file: no whitespace, no date-time without the T form, no free text
        for bad in [
            "tomorrow",
            " 2026-11-05T09:30:15Z",
            "2026-11-05T09:30:15+0200",
            "11/05/2026",
        ] {
            let json = format!(r#"{{"entries":[{{"value":"1.2.3.4","expires":"{bad}"}}]}}"#);
            assert!(
                serde_json::from_str::<FirewallSettings>(&json).is_err(),
                "{bad}"
            );
        }
    }

    #[test]
    fn entries_parse_and_print() {
        let r = FirewallEntry::parse("10.0.0.1-10.0.0.9").unwrap();
        assert_eq!(r.to_string(), "10.0.0.1-10.0.0.9");
        assert!(r.is_blocked(ip("10.0.0.5")) && !r.is_blocked(ip("10.0.0.10")));
        assert_eq!(
            FirewallEntry::parse("10.0.0.7").unwrap().to_string(),
            "10.0.0.7"
        );
        assert!(FirewallEntry::parse("garbage").is_none());
    }

    #[test]
    fn add_remove_expire_and_index() {
        let mut f = Firewall::default();
        let a = FirewallEntry::parse("10.0.0.0/24").unwrap();
        assert!(f.add(a, TimeSpan::ZERO, true, 0));
        assert!(!f.add(a, TimeSpan::ZERO, true, 0), "same range twice");
        assert!(f.add(
            FirewallEntry::single(ip("9.9.9.9")),
            TimeSpan::from_secs(10),
            true,
            1000
        ));
        assert!(f.is_blocked(ip("10.0.0.77")) && f.is_blocked(ip("9.9.9.9")));
        f.expire_entries(10_999);
        assert!(f.is_blocked(ip("9.9.9.9")));
        f.expire_entries(11_000);
        assert!(!f.is_blocked(ip("9.9.9.9")) && f.is_blocked(ip("10.0.0.1")));
        assert!(f.remove(&a));
        assert!(!f.remove(&a));
        assert!(!f.is_blocked(ip("10.0.0.1")));
    }

    #[test]
    fn settings_round_trip_with_expiry() {
        let now = DateTime::from_unix_seconds(1_700_000_000).unwrap();
        let mut f = Firewall::default();
        f.add(
            FirewallEntry::single(ip("1.2.3.4")),
            TimeSpan::ZERO,
            false,
            0,
        );
        f.add(
            FirewallEntry::single(ip("5.6.7.8")),
            TimeSpan::from_secs(60),
            false,
            5000,
        );
        let s = f.to_settings(now, 5000);
        assert_eq!(s.entries[0].expires, None);
        let json = muo_data::json::to_string(&s).unwrap();
        assert!(!json.contains("null"), "{json}");

        let mut g = Firewall::default();
        g.load_from(&s, now, 9000);
        assert_eq!(g.count(), 2);
        // an expiry already passed is skipped on load
        let mut stale = s.clone();
        stale.entries[1].expires = Some(now.add_seconds(-1.0).to_json_string());
        let mut h = Firewall::default();
        h.load_from(&stale, now, 0);
        assert_eq!(h.count(), 1);
    }

    // Firewall behavior and persistence tests.
    mod behavior {
        use super::*;

        #[test]
        fn add_then_is_blocked_single_ip() {
            let mut f = Firewall::default();
            assert!(f.add(
                FirewallEntry::single(ip("1.2.3.4")),
                TimeSpan::ZERO,
                true,
                0
            ));
            assert!(f.is_blocked(ip("1.2.3.4")));
            assert!(!f.is_blocked(ip("1.2.3.5")));
        }

        #[test]
        fn add_range_blocks_inside_not_outside() {
            let mut f = Firewall::default();
            f.add(
                FirewallEntry::range(ip("10.0.0.0"), ip("10.0.0.255")),
                TimeSpan::ZERO,
                true,
                0,
            );
            assert!(f.is_blocked(ip("10.0.0.7")));
            assert!(!f.is_blocked(ip("10.0.1.0")));
        }

        #[test]
        fn remove_unblocks() {
            let mut f = Firewall::default();
            let entry = FirewallEntry::single(ip("1.2.3.4"));
            f.add(entry, TimeSpan::ZERO, true, 0);
            assert!(f.remove(&entry));
            assert!(!f.is_blocked(ip("1.2.3.4")));
        }

        #[test]
        fn expire_entries_removes_expired_keeps_permanent() {
            let mut f = Firewall::default();
            let tick = 1_000;
            f.add(
                FirewallEntry::single(ip("1.1.1.1")),
                TimeSpan::ZERO,
                true,
                tick,
            ); // no ttl
            f.add(
                FirewallEntry::single(ip("2.2.2.2")),
                TimeSpan::from_millis(50),
                true,
                tick,
            ); // ttl

            f.expire_entries(tick + 100); // past the ttl

            assert!(f.is_blocked(ip("1.1.1.1")));
            assert!(!f.is_blocked(ip("2.2.2.2")));
        }

        #[test]
        fn to_firewall_entry_parses_forms() {
            let single = FirewallEntry::parse("1.2.3.4").unwrap();
            assert_eq!(single.min, single.max);
            let cidr = FirewallEntry::parse("10.0.0.0/24").unwrap();
            assert_ne!(cidr.min, cidr.max);
            let range = FirewallEntry::parse("10.0.0.0-10.0.0.255").unwrap();
            assert_eq!((range.min, range.max), (cidr.min, cidr.max));
            assert!(FirewallEntry::parse("not-an-ip").is_none());
        }

        #[test]
        fn to_settings_round_trips_permanent_and_ttl() {
            let now = DateTime::from_unix_seconds(1_700_000_000).unwrap();
            let mut f = Firewall::default();
            f.add(
                FirewallEntry::single(ip("1.2.3.4")),
                TimeSpan::ZERO,
                true,
                0,
            );
            f.add(
                FirewallEntry::single(ip("2.2.2.2")),
                TimeSpan::from_hours(1),
                true,
                0,
            );

            let settings = f.to_settings(now, 0);

            let mut g = Firewall::default();
            g.load_from(&settings, now, 0);

            assert!(g.is_blocked(ip("1.2.3.4")));
            assert!(g.is_blocked(ip("2.2.2.2")));
        }

        #[test]
        fn load_from_skips_already_expired() {
            let now = DateTime::from_unix_seconds(1_700_000_000).unwrap();
            let settings = FirewallSettings {
                entries: vec![FirewallEntryRecord {
                    value: "9.9.9.9".to_string(),
                    expires: Some(now.add_seconds(-3600.0).to_json_string()),
                }],
            };
            let mut f = Firewall::default();
            f.load_from(&settings, now, 0);
            assert!(!f.is_blocked(ip("9.9.9.9")));
        }

        #[test]
        fn to_settings_omits_expiry_for_permanent() {
            let now = DateTime::from_unix_seconds(1_700_000_000).unwrap();
            let mut f = Firewall::default();
            f.add(
                FirewallEntry::single(ip("1.2.3.4")),
                TimeSpan::ZERO,
                true,
                0,
            );

            let settings = f.to_settings(now, 0);

            assert_eq!(settings.entries.len(), 1);
            assert_eq!(settings.entries[0].expires, None);
            assert_eq!(settings.entries[0].value, "1.2.3.4");
        }
    }
}
