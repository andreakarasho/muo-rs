//! IP address helpers: parsing, CIDR ranges and private network checks.
//!
//! Addresses are normalized to a `u128` in IPv6 form (an IPv4 address is `::ffff:a.b.c.d`) so v4 and v6
//! ranges compare directly.
// shortcut: `IpAddr::from_str` is stricter than `IPAddress.TryParse` (no `1.2` shorthand, no
// scope ids); config entries in ban/allow lists are full literals. Revisit if a list breaks.

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::sync::OnceLock;

use crate::collections::SortedRangeIndex;

/// `ToUInt128`: the IPv6-form bits of an address.
pub fn to_u128(ip: IpAddr) -> u128 {
    match ip {
        IpAddr::V4(v4) => 0xFFFF_0000_0000u128 | u128::from(u32::from(v4)),
        IpAddr::V6(v6) => u128::from(v6),
    }
}

/// `ToIpAddress`: a v4-mapped value gives the IPv4 address unless `map_to_ipv6` is set.
pub fn to_ip_addr(value: u128, map_to_ipv6: bool) -> IpAddr {
    if !map_to_ipv6 && (0xFFFF_0000_0000..=0xFFFF_FFFF_FFFF).contains(&value) {
        return IpAddr::V4(Ipv4Addr::from(value as u32));
    }
    IpAddr::V6(Ipv6Addr::from(value))
}

/// `WriteMappedIPv6To`: sixteen bytes, an IPv4 address written as `::ffff:a.b.c.d`.
pub fn write_mapped_ipv6(ip: IpAddr) -> [u8; 16] {
    match ip {
        IpAddr::V6(v6) => v6.octets(),
        IpAddr::V4(v4) => Ipv6Addr::from(0xFFFF_0000_0000u128 | u128::from(u32::from(v4))).octets(),
    }
}

/// `Utility.ApplyCidrMask`. Kept as written, including what looks like a bug: for a prefix under 64 (IPv6 only) the
/// masks use `bitsToFlip` the wrong way round, so e.g. `fc00::/7` covers `fc00::` to `fc00:0:0:7f:ffff:..`
/// only. Shift counts wrap at 64. Existing ban and allow lists rely on these ranges, so change it only on purpose.
pub fn apply_cidr_mask(high: &mut u64, low: &mut u64, prefix_length: i32, is_max: bool) {
    if prefix_length == 0 {
        *high = u64::MAX;
        *low = u64::MAX;
        return;
    }
    if prefix_length == 128 {
        return;
    }
    if prefix_length == 64 {
        *low = 0;
        return;
    }
    if prefix_length < 64 {
        let bits_to_flip = (64 - prefix_length) as u32;
        let high_mask = if is_max {
            u64::MAX.wrapping_shr(bits_to_flip)
        } else {
            u64::MAX.wrapping_shl(bits_to_flip + 1)
        };
        *high = if is_max {
            *high | high_mask
        } else {
            *high & high_mask
        };
        *low = if is_max { u64::MAX } else { 0 };
    } else {
        let bits_to_flip = (128 - prefix_length) as u32;
        let low_mask = if is_max {
            u64::MAX.wrapping_shr(64u32.wrapping_sub(bits_to_flip))
        } else {
            u64::MAX.wrapping_shl(bits_to_flip)
        };
        *low = if is_max {
            *low | low_mask
        } else {
            *low & low_mask
        };
    }
}

/// `Utility.CreateCidrAddress`: the lowest (`is_max` false) or highest address of the range.
pub fn create_cidr_address(bytes: [u8; 16], prefix_length: i32, is_max: bool) -> u128 {
    let mut high = u64::from_be_bytes(bytes[..8].try_into().unwrap_or_default());
    let mut low = u64::from_be_bytes(bytes[8..].try_into().unwrap_or_default());
    if prefix_length < 128 {
        apply_cidr_mask(&mut high, &mut low, prefix_length, is_max);
    }
    (u128::from(high) << 64) | u128::from(low)
}

/// `MatchCidr`: is `address` inside `cidr_address/prefix_length`? An IPv4 prefix is 0..=32.
pub fn match_cidr(cidr_address: IpAddr, mut prefix_length: i32, address: IpAddr) -> bool {
    if cidr_address.is_ipv4() {
        prefix_length += 96;
    }
    let bytes = write_mapped_ipv6(cidr_address);
    let min = create_cidr_address(bytes, prefix_length, false);
    let max = create_cidr_address(bytes, prefix_length, true);
    let ip = to_u128(address);
    ip >= min && ip <= max
}

/// `MatchClassC`: same /24.
pub fn match_class_c(a: IpAddr, b: IpAddr) -> bool {
    match_cidr(a, 24, b)
}

/// `TryParseCidrRange`: `a.b.c.d/n`, `::/n` or a bare address (one host) to an inclusive range.
pub fn try_parse_cidr_range(cidr: &str) -> Option<(u128, u128)> {
    let (addr, prefix) = match cidr.find('/') {
        Some(i) => (&cidr[..i], Some(&cidr[i + 1..])),
        None => (cidr, None),
    };
    let ip = crate::dotnet::parse_ip_address(addr)?;
    let max_prefix = if ip.is_ipv6() { 128 } else { 32 };
    let mut prefix_length = match prefix {
        None => max_prefix,
        Some(p) => p
            .trim_matches([' ', '\t', '\n', '\x0b', '\x0c', '\r'])
            .parse::<i32>()
            .ok()
            .filter(|n| (0..=max_prefix).contains(n))?,
    };
    if ip.is_ipv4() {
        prefix_length += 96;
    }
    let bytes = write_mapped_ipv6(ip);
    Some((
        create_cidr_address(bytes, prefix_length, false),
        create_cidr_address(bytes, prefix_length, true),
    ))
}

/// `TryV4`: the big-endian value of an IPv4 address.
pub fn try_v4(ip: IpAddr) -> Option<u32> {
    match ip {
        IpAddr::V4(v4) => Some(u32::from(v4)),
        IpAddr::V6(_) => None,
    }
}

/// `TryMappedV4`: the last four bytes of an IPv6 address (the embedded IPv4 of a mapped one).
pub fn try_mapped_v4(ip: IpAddr) -> Option<u32> {
    match ip {
        IpAddr::V6(v6) => Some(u128::from(v6) as u32),
        IpAddr::V4(_) => None,
    }
}

/// `TryParseV4`: a strict dotted quad into a big-endian value (digits only, at most three per octet).
pub fn try_parse_v4(s: &str) -> Option<u32> {
    try_parse_v4_bytes(s.as_bytes())
}

/// `TryParseV4(ReadOnlySpan<byte>)`: the same, straight from file bytes.
pub fn try_parse_v4_bytes(s: &[u8]) -> Option<u32> {
    let (mut acc, mut octet, mut digits, mut dots) = (0u32, 0u32, 0, 0);
    for &c in s {
        match c {
            b'.' => {
                if digits == 0 || octet > 255 {
                    return None;
                }
                acc = (acc << 8) | octet;
                dots += 1;
                octet = 0;
                digits = 0;
            }
            b'0'..=b'9' => {
                octet = octet * 10 + u32::from(c - b'0');
                digits += 1;
                if digits > 3 {
                    return None;
                }
            }
            _ => return None,
        }
    }
    if dots != 3 || digits == 0 || octet > 255 {
        return None;
    }
    Some((acc << 8) | octet)
}

// ---- NetworkUtilities --------------------------------------------------------------------------

fn build_index(cidrs: &[&str]) -> SortedRangeIndex<u128> {
    let ranges = cidrs
        .iter()
        .map(|c| {
            try_parse_cidr_range(c)
                .unwrap_or_else(|| unreachable!("invalid reserved-network CIDR {c}"))
        })
        .collect();
    SortedRangeIndex::from_ranges(ranges)
}

fn private_v4() -> &'static SortedRangeIndex<u128> {
    static INDEX: OnceLock<SortedRangeIndex<u128>> = OnceLock::new();
    INDEX.get_or_init(|| {
        build_index(&[
            "127.0.0.1/8",
            "192.168.0.0/16",
            "10.0.0.0/8",
            "172.16.0.0/12",
            "169.254.0.0/16",
            "100.64.0.0/10",
        ])
    })
}

fn private_v6() -> &'static SortedRangeIndex<u128> {
    static INDEX: OnceLock<SortedRangeIndex<u128>> = OnceLock::new();
    INDEX.get_or_init(|| build_index(&["fc00::/7", "fe80::/10"]))
}

pub fn is_private_network_v4(ip: IpAddr) -> bool {
    private_v4().contains(to_u128(ip))
}

pub fn is_private_network_v6(ip: IpAddr) -> bool {
    private_v6().contains(to_u128(ip))
}

/// `IsPrivateNetwork`: loopback, RFC 1918, link-local, CGNAT and unique-local ranges.
pub fn is_private_network(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(_) => is_private_network_v4(ip),
        IpAddr::V6(_) => is_private_network_v6(ip),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ip(s: &str) -> IpAddr {
        s.parse().unwrap()
    }

    #[test]
    fn cidr_matching() {
        // (cidr, address, length, should match) from IPAddressTests.TestIPvCIDR
        let cases = [
            ("192.168.100.254", "192.168.100.1", 16, true),
            ("192.168.100.254", "192.168.50.1", 24, false),
            ("192.168.100.1", "192.168.50.1", 32, false),
            ("192.168.50.1", "192.168.50.1", 32, true),
            ("192.168.50.4", "192.168.50.7", 30, true),
            ("192.168.50.4", "192.168.50.9", 30, false),
            ("::ffff:192.168.100.254", "192.168.100.1", 112, true),
            ("::ffff:192.168.100.254", "192.168.50.1", 104, true),
            ("::ffff:192.168.100.254", "192.168.50.1", 120, false),
            (
                "1234:5678:9ABC:1234:5678:9ABC:1234:5678",
                "1234:5677:0:0:0:0:0:0",
                16,
                true,
            ),
            (
                "1234:5678:9ABC:1234:5678:9ABC:1234:5678",
                "1234:5677:0:0:0:0:0:0",
                64,
                false,
            ),
            ("::1234:5678", "1234:5677::", 64, false),
            ("::1234:5678", "::9ABC:5677", 112, false),
            ("::1234:5678", "::1234:66AA", 112, true),
            ("::1234:5678", "::1235:FFFF", 109, true),
            ("::1234:5678", "::1238:ABAC", 109, false),
        ];
        for (cidr, addr, len, want) in cases {
            assert_eq!(
                match_cidr(ip(cidr), len, ip(addr)),
                want,
                "{cidr} {addr} /{len}"
            );
        }
        assert!(
            match_class_c(ip("10.1.2.3"), ip("10.1.2.200"))
                && !match_class_c(ip("10.1.2.3"), ip("10.1.3.3"))
        );
    }

    #[test]
    fn to_u128_round_trip() {
        // IPAddressTests.TestIPAddressToUInt128 (a mapped v6 is mapped to v4 first, as in the test)
        let v = to_u128(ip("192.168.1.1"));
        assert_eq!(v, 0xffff_c0a8_0101);
        assert_eq!(to_ip_addr(v, false), ip("192.168.1.1"));
        assert_eq!(to_ip_addr(v, true), ip("::ffff:192.168.1.1"));
        assert_eq!(to_u128(ip("::ffff:192.168.1.1")), 0xffff_c0a8_0101);
        let w = to_u128(ip("4cce:1490:4577:d72f:693e:b42e:3465:f3db"));
        assert_eq!(w, (0x4cce_1490_4577_d72fu128 << 64) | 0x693e_b42e_3465_f3db);
        assert_eq!(
            to_ip_addr(w, false),
            ip("4cce:1490:4577:d72f:693e:b42e:3465:f3db")
        );
    }

    #[test]
    fn cidr_ranges() {
        assert_eq!(
            try_parse_cidr_range("10.0.0.0/8"),
            Some((to_u128(ip("10.0.0.0")), to_u128(ip("10.255.255.255"))))
        );
        let one = to_u128(ip("1.2.3.4"));
        assert_eq!(try_parse_cidr_range("1.2.3.4"), Some((one, one)));
        let (min, max) = try_parse_cidr_range("172.16.0.0/12").unwrap();
        assert_eq!(
            (min, max),
            (to_u128(ip("172.16.0.0")), to_u128(ip("172.31.255.255")))
        );
        for bad in ["1.2.3.4/33", "1.2.3.4/-1", "nope", "1.2.3.4/x", "::1/129"] {
            assert_eq!(try_parse_cidr_range(bad), None, "{bad}");
        }
    }

    #[test]
    fn v4_parsing() {
        assert_eq!(try_parse_v4("192.168.1.1"), Some(0xC0A8_0101));
        assert_eq!(try_parse_v4_bytes(b"0.0.0.0"), Some(0));
        for bad in [
            "1.2.3",
            "1.2.3.4.5",
            "256.1.1.1",
            "1..2.3",
            "1.2.3.",
            "a.b.c.d",
            "1234.1.1.1",
            "",
        ] {
            assert_eq!(try_parse_v4(bad), None, "{bad}");
        }
        assert_eq!(try_v4(ip("1.2.3.4")), Some(0x0102_0304));
        assert_eq!(try_mapped_v4(ip("::ffff:1.2.3.4")), Some(0x0102_0304));
    }

    #[test]
    fn private_networks() {
        for a in [
            "127.0.0.1",
            "10.2.3.4",
            "192.168.0.9",
            "172.20.1.1",
            "169.254.1.1",
            "100.100.0.1",
            "fc00::1",
            "fe80::2",
        ] {
            assert!(is_private_network(ip(a)), "{a}");
        }
        // fd00::1 is inside fc00::/7 but not in what the mask code computes for a prefix under 64 (see apply_cidr_mask)
        for a in [
            "8.8.8.8",
            "172.32.0.1",
            "2001:db8::1",
            "100.128.0.1",
            "fd00::1",
        ] {
            assert!(!is_private_network(ip(a)), "{a}");
        }
    }
}
