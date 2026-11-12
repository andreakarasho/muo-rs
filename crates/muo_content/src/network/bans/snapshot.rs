//! Blocklist snapshot, file and promoted guard: the immutable
//! address-range set a list file builds, the file's header line, and the guard that stops one address being
//! reported again within a window.

use std::collections::HashMap;
use std::net::IpAddr;
use std::path::Path;

use muo_primitives::collections::SortedRangeIndex;
use muo_primitives::net::{to_u128, try_mapped_v4, try_parse_v4_bytes, try_v4};

/// `BlocklistSnapshot`.
#[derive(Clone, Debug, Default)]
pub struct BlocklistSnapshot {
    v4: SortedRangeIndex<u32>,
    v6: SortedRangeIndex<u128>,
}

fn trim_ascii(mut s: &[u8]) -> &[u8] {
    while let [first, rest @ ..] = s
        && first.is_ascii_whitespace()
    {
        s = rest;
    }
    while let [rest @ .., last] = s
        && last.is_ascii_whitespace()
    {
        s = rest;
    }
    s
}

/// `Utf8Parser.TryParse(int)` that must consume the whole token.
fn parse_bits(token: &[u8]) -> Option<i32> {
    std::str::from_utf8(token).ok()?.parse().ok()
}

fn decode_v6(addr: &[u8]) -> Option<u128> {
    if addr.len() > 45 {
        return None;
    }
    match muo_primitives::dotnet::parse_ip_address(std::str::from_utf8(addr).ok()?)? {
        std::net::IpAddr::V6(a) => Some(u128::from(a)),
        std::net::IpAddr::V4(_) => None,
    }
}

impl BlocklistSnapshot {
    pub fn empty() -> Self {
        Self::default()
    }

    pub fn count(&self) -> usize {
        self.v4.len() + self.v6.len()
    }

    /// `Build(data, out parsed, out skipped)`: one address or CIDR per line; blank lines and `#`/`;` comments are
    /// ignored, anything else that does not parse counts as skipped.
    pub fn build(data: &[u8]) -> (Self, usize, usize) {
        let (mut parsed, mut skipped) = (0, 0);
        let mut v4: Vec<(u32, u32)> = Vec::new();
        let mut v6: Vec<(u128, u128)> = Vec::new();

        for line in data.split(|&b| b == b'\n') {
            let line = trim_ascii(line);
            if line.is_empty() || line[0] == b'#' || line[0] == b';' {
                continue;
            }
            let slash = line.iter().position(|&b| b == b'/');
            let addr = slash.map_or(line, |i| &line[..i]);
            let bits_token = slash.map_or(&line[..0], |i| &line[i + 1..]);

            if !addr.contains(&b':') {
                // IPv4 single or CIDR
                let Some(ip) = try_parse_v4_bytes(addr) else {
                    skipped += 1;
                    continue;
                };
                if slash.is_some() {
                    match parse_bits(bits_token) {
                        Some(bits) if (0..=32).contains(&bits) => {
                            let size = if bits == 0 {
                                u32::MAX
                            } else {
                                (1u32 << (32 - bits)) - 1
                            };
                            let b = ip & !size;
                            v4.push((b, b.wrapping_add(size)));
                            parsed += 1;
                        }
                        _ => skipped += 1,
                    }
                } else {
                    v4.push((ip, ip));
                    parsed += 1;
                }
            } else if let Some(v) = decode_v6(addr) {
                if slash.is_some() {
                    match parse_bits(bits_token) {
                        Some(bits) if (0..=128).contains(&bits) => {
                            let mask = if bits == 0 {
                                0
                            } else {
                                !((1u128 << (128 - bits)) - 1)
                            };
                            let b = v & mask;
                            v6.push((b, b | !mask));
                            parsed += 1;
                        }
                        _ => skipped += 1,
                    }
                } else {
                    v6.push((v, v));
                    parsed += 1;
                }
            } else {
                skipped += 1;
            }
        }
        (
            Self {
                v4: SortedRangeIndex::from_ranges(v4),
                v6: SortedRangeIndex::from_ranges(v6),
            },
            parsed,
            skipped,
        )
    }

    /// `IsBanned(ip)` / `Contains(ip)`.
    pub fn is_banned(&self, ip: IpAddr) -> bool {
        if let IpAddr::V6(v6) = ip {
            if v6.to_ipv4_mapped().is_some() {
                // v6-encoded v4 must not dodge the v4 set
                return try_mapped_v4(ip).is_some_and(|v| self.v4.contains(v));
            }
            return self.v6.contains(to_u128(ip));
        }
        try_v4(ip).is_some_and(|v| self.v4.contains(v))
    }
}

/// `BlocklistHeader`.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct BlocklistHeader {
    pub generated: Option<String>,
    pub count: i32,
    pub present: bool,
}

/// `BlocklistFile.TryReadHeader(path)`: the `# ... generated=X count=N` first line.
pub fn try_read_header(path: &Path) -> Option<BlocklistHeader> {
    if !path.exists() {
        return None;
    }
    let text = std::fs::read(path).ok()?;
    let first_line = text
        .split(|&b| b == b'\n')
        .next()
        .filter(|_| !text.is_empty());
    let Some(first) = first_line else {
        return Some(BlocklistHeader {
            present: true,
            ..Default::default()
        });
    };
    let first = String::from_utf8_lossy(first);
    let first = first.trim_end_matches('\r');
    let (mut generated, mut count) = (None, 0);
    if first.starts_with('#') {
        for tok in first.split(' ').filter(|t| !t.is_empty()) {
            if let Some(g) = tok.strip_prefix("generated=") {
                generated = Some(g.to_owned());
            } else if let Some(c) = tok.strip_prefix("count=") {
                count = c.parse().unwrap_or(0);
            }
        }
    }
    Some(BlocklistHeader {
        generated,
        count,
        present: true,
    })
}

/// `BlocklistFile.Load(path)`: the snapshot, or an empty one when the file is absent or unreadable.
pub fn load_file(path: &Path) -> (BlocklistSnapshot, usize, usize) {
    match std::fs::read(path) {
        Ok(bytes) => BlocklistSnapshot::build(&bytes),
        Err(_) => (BlocklistSnapshot::empty(), 0, 0),
    }
}

/// `PromotedGuard`: remembers an address for a while so it is reported once per window.
#[derive(Default)]
pub struct PromotedGuard {
    expiry: HashMap<u128, i64>,
}

impl PromotedGuard {
    /// `TryMark(ip, nowTicks, ttlMs)`: false while an earlier mark of the address is live.
    pub fn try_mark(&mut self, ip: u128, now_ticks: i64, ttl_ms: i64) -> bool {
        if self
            .expiry
            .get(&ip)
            .is_some_and(|exp| exp.wrapping_sub(now_ticks) > 0)
        {
            return false;
        }
        self.expiry.insert(ip, now_ticks.wrapping_add(ttl_ms));
        true
    }

    /// `Sweep(nowTicks)`.
    pub fn sweep(&mut self, now_ticks: i64) {
        self.expiry.retain(|_, exp| exp.wrapping_sub(now_ticks) > 0);
    }

    pub fn len(&self) -> usize {
        self.expiry.len()
    }

    pub fn is_empty(&self) -> bool {
        self.expiry.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build(lines: &[&str]) -> BlocklistSnapshot {
        BlocklistSnapshot::build(lines.join("\n").as_bytes()).0
    }

    fn ip(s: &str) -> IpAddr {
        s.parse().unwrap()
    }

    #[test]
    fn single_ip_is_matched() {
        let s = build(&["1.2.3.4"]);
        assert!(s.is_banned(ip("1.2.3.4")));
        assert!(!s.is_banned(ip("1.2.3.5")));
    }

    #[test]
    fn cidr_contains_and_excludes_boundaries() {
        let s = build(&["10.0.0.0/24"]);
        assert!(s.is_banned(ip("10.0.0.0")) && s.is_banned(ip("10.0.0.255")));
        assert!(!s.is_banned(ip("10.0.1.0")) && !s.is_banned(ip("9.255.255.255")));
    }

    #[test]
    fn comments_blanks_and_garbage_are_skipped() {
        let (s, parsed, skipped) = BlocklistSnapshot::build(
            [
                "# header generated=x",
                "",
                "not-an-ip",
                "1.2.3.4",
                "::1",
                "5.6.7.0/24",
            ]
            .join("\n")
            .as_bytes(),
        );
        assert_eq!(parsed, 3);
        assert!(skipped >= 1);
        assert!(s.is_banned(ip("5.6.7.200")));
    }

    #[test]
    fn ipv6_single_and_cidr_are_matched() {
        let s = build(&["2001:db8::1", "2001:db8:1::/48"]);
        assert!(s.is_banned(ip("2001:db8::1")));
        assert!(s.is_banned(ip("2001:db8:1::abcd")));
        assert!(!s.is_banned(ip("2001:db8:2::1")));
    }

    #[test]
    fn ipv4_mapped_ipv6_is_normalized_to_v4() {
        assert!(build(&["1.2.3.4"]).is_banned(ip("::ffff:1.2.3.4")));
    }

    #[test]
    fn nested_and_overlapping_intervals_are_coalesced() {
        let s = build(&["10.0.0.0/24", "10.0.0.5/32"]);
        assert!(s.is_banned(ip("10.0.0.100")) && s.is_banned(ip("10.0.0.5")));
        assert!(!s.is_banned(ip("10.0.1.0")));
        let s = build(&["10.0.0.0/25", "10.0.0.64/25"]);
        assert!(s.is_banned(ip("10.0.0.100")) && !s.is_banned(ip("10.0.0.200")));
    }

    #[test]
    fn promoted_guard_marks_once_per_window() {
        let mut g = PromotedGuard::default();
        assert!(g.try_mark(1, 1000, 500));
        assert!(!g.try_mark(1, 1499, 500));
        assert!(g.try_mark(1, 1500, 500));
        g.sweep(5000);
        assert!(g.is_empty());
    }

    fn temp_file(name: &str, content: Option<&str>) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("muo-bl-{}-{name}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let p = dir.join("blocklist.txt");
        match content {
            Some(c) => std::fs::write(&p, c).unwrap(),
            None => {
                let _ = std::fs::remove_file(&p);
            }
        }
        p
    }

    #[test]
    fn reads_header_generated_and_count() {
        let p = temp_file(
            "header",
            Some(
                "# muo-blocklist v1 generated=2026-07-21T09:24:23Z count=2\n1.2.3.4\n5.6.7.0/24\n",
            ),
        );
        let h = try_read_header(&p).expect("header");
        assert!(h.present);
        assert_eq!(h.generated.as_deref(), Some("2026-07-21T09:24:23Z"));
        assert_eq!(h.count, 2);
    }

    #[test]
    fn missing_file_reports_absent_and_loads_empty() {
        let p = temp_file("missing", None);
        assert!(try_read_header(&p).is_none());
        let (snap, _, _) = load_file(&p);
        assert!(!snap.is_banned(ip("1.2.3.4")));
    }

    // Pins the exact header line the blocklist exporter emits: the producer adds informational tokens the
    // reader does not know about, and the reload detector breaks silently if generated= stops being read.
    #[test]
    fn reads_generator_header_with_extra_tokens() {
        let p = temp_file(
            "generator",
            Some(
                "# muo-blocklist generated=2026-07-25T18:03:11Z count=12442 ipv4=7868 cidr=4574 feeds=2\n2.181.183.77\n223.169.0.0/16\n",
            ),
        );
        let h = try_read_header(&p).expect("header");
        assert_eq!(h.generated.as_deref(), Some("2026-07-25T18:03:11Z"));
        assert_eq!(h.count, 12442);

        let (snap, parsed, skipped) = load_file(&p);
        assert_eq!(parsed, 2);
        assert_eq!(skipped, 0);
        assert!(snap.is_banned(ip("2.181.183.77")));
        assert!(snap.is_banned(ip("223.169.4.9")));
    }

    #[test]
    fn first_mark_true_then_suppressed_until_ttl() {
        let mut g = PromotedGuard::default();
        assert!(g.try_mark(42, 1000, 5000));
        assert!(!g.try_mark(42, 2000, 5000)); // within TTL
        assert!(g.try_mark(42, 6001, 5000)); // expired -> re-mark
    }

    #[test]
    fn sweep_removes_expired_entries_allowing_remark() {
        let mut g = PromotedGuard::default();
        assert!(g.try_mark(7, 0, 1000));
        assert!(!g.try_mark(7, 500, 1000)); // still within TTL

        g.sweep(500); // not yet expired, sweep should not remove it
        assert!(!g.try_mark(7, 999, 1000));

        g.sweep(1001); // now expired, sweep removes it
        assert!(g.try_mark(7, 1002, 1000)); // fresh mark, not "still marked"
    }
}
