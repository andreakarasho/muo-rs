//! The per-IP connection rate limiter and `is_private_network`.
//!
//! Single-threaded: entries are dropped once both their attempt window and any block have lapsed, which
//! cannot change a verdict.

use std::collections::HashMap;
use std::net::IpAddr;

#[derive(Default)]
struct IpStats {
    attempt_count: i32,
    last_attempt_ticks: i64,
    block_until_ticks: i64,
}

pub struct IpRateLimiter {
    attempts: HashMap<IpAddr, IpStats>,
    max_attempts: i32,
    time_window: i64,
    initial_backoff: i64,
    backoff_multiplier: f64,
    max_backoff: i64,
    next_cleanup: i64,
}

impl IpRateLimiter {
    /// `NetState.ConfigureNetwork` creates it as (10, 10000, 1000, 2.0, 3_600_000): at most 10 attempts per
    /// 10 s, then a doubling backoff from 1 s up to an hour. All times in milliseconds.
    pub fn new(
        max_attempts: i32,
        time_window: i64,
        initial_backoff: i64,
        backoff_multiplier: f64,
        max_backoff: i64,
    ) -> Self {
        Self {
            attempts: HashMap::new(),
            max_attempts,
            time_window,
            initial_backoff,
            backoff_multiplier,
            max_backoff,
            next_cleanup: 0,
        }
    }

    pub fn default_for_server() -> Self {
        Self::new(10, 10000, 1000, 2.0, 3_600_000)
    }

    /// Returns (allowed, total_attempts). Private-network addresses are always allowed.
    pub fn verify(&mut self, ip: IpAddr, now_ticks: i64) -> (bool, i32) {
        if is_private_network(ip) {
            return (true, 0);
        }

        if now_ticks - self.next_cleanup >= 0 {
            self.next_cleanup = now_ticks + 60_000;
            let window = self.time_window;
            self.attempts.retain(|_, s| {
                now_ticks - s.last_attempt_ticks < window || s.block_until_ticks - now_ticks > 0
            });
        }

        let s = self.attempts.entry(ip).or_default();

        if now_ticks - s.last_attempt_ticks > self.time_window {
            s.attempt_count = 1; // Reset
        } else {
            s.attempt_count += 1;
        }

        let total = s.attempt_count;
        s.last_attempt_ticks = now_ticks;

        if s.block_until_ticks - now_ticks > 0 {
            return (false, total);
        }

        if s.attempt_count > self.max_attempts {
            let backoff = (self.initial_backoff as f64
                * self
                    .backoff_multiplier
                    .powi(s.attempt_count - self.max_attempts)) as i64;
            s.block_until_ticks = now_ticks + backoff.min(self.max_backoff);
            return (false, total);
        }

        (true, total)
    }

    /// Addresses currently tracked (for tests and diagnostics).
    pub fn tracked(&self) -> usize {
        self.attempts.len()
    }
}

/// `NetworkUtilities.IsPrivateNetwork`: loopback, RFC 1918, link-local and CGNAT for IPv4; fc00::/7 and
/// fe80::/10 for IPv6. A v4-mapped IPv6 address is judged as IPv6.
pub fn is_private_network(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(a) => {
            let v = u32::from(a);
            [
                (0x7F00_0000u32, 8u32), // 127.0.0.1/8
                (0xC0A8_0000, 16),      // 192.168.0.0/16
                (0x0A00_0000, 8),       // 10.0.0.0/8
                (0xAC10_0000, 12),      // 172.16.0.0/12
                (0xA9FE_0000, 16),      // 169.254.0.0/16
                (0x6440_0000, 10),      // 100.64.0.0/10
            ]
            .iter()
            .any(|&(net, bits)| v >> (32 - bits) == net >> (32 - bits))
        }
        IpAddr::V6(a) => {
            let v = u128::from(a);
            // fc00::/7 and fe80::/10
            v >> 121 == 0x7E || v >> 118 == 0x3FA
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn public() -> IpAddr {
        "203.0.113.5".parse().unwrap()
    }

    #[test]
    fn private_networks() {
        for s in [
            "127.0.0.1",
            "10.1.2.3",
            "192.168.1.1",
            "172.16.0.1",
            "172.31.255.255",
            "169.254.1.1",
            "100.64.0.1",
            "fc00::1",
            "fd12::1",
            "fe80::1",
        ] {
            assert!(is_private_network(s.parse().unwrap()), "{s}");
        }
        for s in [
            "8.8.8.8",
            "172.32.0.1",
            "100.128.0.1",
            "2001:db8::1",
            "::ffff:127.0.0.1",
        ] {
            assert!(!is_private_network(s.parse().unwrap()), "{s}");
        }
    }

    #[test]
    fn private_addresses_are_never_limited() {
        let mut l = IpRateLimiter::default_for_server();
        for _ in 0..100 {
            assert_eq!(l.verify("127.0.0.1".parse().unwrap(), 50_000), (true, 0));
        }
        assert_eq!(l.tracked(), 0);
    }

    #[test]
    fn eleventh_attempt_in_window_is_blocked_with_backoff() {
        let mut l = IpRateLimiter::default_for_server();
        let t = 100_000;
        for i in 1..=10 {
            assert_eq!(l.verify(public(), t), (true, i));
        }
        // 11th: count 11 > 10, backoff = 1000 * 2^1 = 2000 ms
        assert_eq!(l.verify(public(), t), (false, 11));
        // Still blocked a moment later (the attempt still counts)
        assert_eq!(l.verify(public(), t + 1999), (false, 12));
        // Block lapses; the window (10 s) has not, so attempts keep climbing and are limited again
        let (ok, n) = l.verify(public(), t + 5000);
        assert_eq!(n, 13);
        assert!(!ok);
    }

    #[test]
    fn window_expiry_resets_the_count() {
        let mut l = IpRateLimiter::default_for_server();
        for _ in 0..5 {
            l.verify(public(), 100_000);
        }
        assert_eq!(l.verify(public(), 111_000), (true, 1));
    }

    #[test]
    fn backoff_is_capped() {
        let mut l = IpRateLimiter::new(1, 1_000_000_000, 1000, 2.0, 5000);
        let _ = l.verify(public(), 10);
        let _ = l.verify(public(), 10);
        // count 30: 1000 * 2^29 capped to 5000; at t=10 + 5000 it is unblocked only if count permits, still limited
        for _ in 0..28 {
            l.verify(public(), 20_000);
        }
        assert!(!l.verify(public(), 20_000).0);
    }

    #[test]
    fn idle_entries_are_dropped_but_blocked_ones_kept() {
        let mut l = IpRateLimiter::default_for_server();
        let other: IpAddr = "203.0.113.9".parse().unwrap();
        for _ in 0..11 {
            l.verify(public(), 100_000); // blocked until 102_000
        }
        l.verify(other, 100_000);
        assert_eq!(l.tracked(), 2);
        // Next cleanup is due at >= 60_000 after the first; `other` is idle (window passed), the blocked
        // one is not blocked anymore by then either.
        l.verify("198.51.100.1".parse().unwrap(), 200_000);
        assert_eq!(l.tracked(), 1);
    }
}
