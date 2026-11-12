//! The server's random generator.

use super::{Rng, seed_state, xoshiro_next};

/// The server's generator: xoshiro256**. It is normally seeded from the OS, so only
/// [`BuiltInRng::from_seed`] is reproducible.
#[derive(Clone, Debug)]
pub struct BuiltInRng {
    state: [u64; 4],
    /// Tests: `PredictableRandom(value)` swapped in for the generator (see [`BuiltInRng::set_predictable`]).
    fixed: Option<FixedRng>,
}

impl BuiltInRng {
    /// Seeds from the OS (`BuiltInRng.Reset()`).
    pub fn new() -> BuiltInRng {
        use std::hash::{BuildHasher, Hasher};
        // std's RandomState is seeded from the OS, which saves a dependency for one seed.
        let mut h = std::collections::hash_map::RandomState::new().build_hasher();
        h.write_u128(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_or(0, |d| d.as_nanos()),
        );
        BuiltInRng::from_seed(h.finish())
    }

    /// Deterministic stream for a seed (SplitMix64 fills the state, as in `StableRandom`).
    pub fn from_seed(seed: u64) -> BuiltInRng {
        BuiltInRng {
            state: seed_state(seed),
            fixed: None,
        }
    }

    /// `using var rng = new PredictableRandom(value)`: every call returns `value` clamped into its range, until
    /// `set_predictable(None)`.
    pub fn set_predictable(&mut self, value: Option<i32>) {
        self.fixed = value.map(FixedRng);
    }
}

impl Default for BuiltInRng {
    fn default() -> BuiltInRng {
        BuiltInRng::new()
    }
}

impl Rng for BuiltInRng {
    fn next_u64(&mut self) -> u64 {
        match &mut self.fixed {
            Some(f) => f.next_u64(),
            None => xoshiro_next(&mut self.state),
        }
    }

    // The overrides below only differ from the trait's defaults while `PredictableRandom` is in effect.
    fn next(&mut self) -> i32 {
        if let Some(f) = &mut self.fixed {
            return f.next();
        }
        loop {
            let r = self.next_u64() >> 32;
            if r != i32::MAX as u64 {
                return r as i32;
            }
        }
    }

    fn next_below(&mut self, max: i32) -> i32 {
        if let Some(f) = &mut self.fixed {
            return f.next_below(max);
        }
        if max > 1 {
            super::bounded_u32(self, max as u32) as i32
        } else {
            0
        }
    }

    fn next_from(&mut self, min: i32, count: i32) -> i32 {
        if let Some(f) = &mut self.fixed {
            return f.next_from(min, count);
        }
        min.wrapping_add(self.next_below(count))
    }

    fn next_i64(&mut self) -> i64 {
        if let Some(f) = &mut self.fixed {
            return f.next_i64();
        }
        loop {
            let r = self.next_u64() >> 1;
            if r != i64::MAX as u64 {
                return r as i64;
            }
        }
    }

    fn next_i64_below(&mut self, max: i64) -> i64 {
        if let Some(f) = &mut self.fixed {
            return f.next_i64_below(max);
        }
        if max > 1 {
            super::bounded_u64(self, max as u64) as i64
        } else {
            0
        }
    }

    fn next_i64_from(&mut self, min: i64, count: i64) -> i64 {
        if let Some(f) = &mut self.fixed {
            return f.next_i64_from(min, count);
        }
        min.wrapping_add(self.next_i64_below(count))
    }

    fn next_f64(&mut self) -> f64 {
        if let Some(f) = &mut self.fixed {
            return f.next_f64();
        }
        (self.next_u64() >> 11) as f64 * (1.0 / (1u64 << 53) as f64)
    }

    fn fill_bytes(&mut self, buf: &mut [u8]) {
        if let Some(f) = &mut self.fixed {
            return f.fill_bytes(buf);
        }
        for chunk in buf.chunks_mut(8) {
            let word = self.next_u64().to_le_bytes();
            chunk.copy_from_slice(&word[..chunk.len()]);
        }
    }
}

/// Test double for `PredictableRandom`: every call returns `value` clamped into its range.
#[derive(Clone, Copy, Debug)]
pub struct FixedRng(pub i32);

impl Rng for FixedRng {
    fn next_u64(&mut self) -> u64 {
        self.0 as u64
    }
    fn next(&mut self) -> i32 {
        self.0
    }
    fn next_below(&mut self, max: i32) -> i32 {
        self.0.clamp(0, (max - 1).max(0))
    }
    fn next_from(&mut self, min: i32, count: i32) -> i32 {
        // the fixture overrides Next(min, max) with max = min + count
        self.0
            .wrapping_add(min)
            .clamp(min, (min.wrapping_add(count) - 1).max(min))
    }
    fn next_i64(&mut self) -> i64 {
        i64::from(self.0)
    }
    fn next_i64_below(&mut self, max: i64) -> i64 {
        i64::from(self.0).clamp(0, (max - 1).max(0))
    }
    fn next_i64_from(&mut self, min: i64, count: i64) -> i64 {
        (i64::from(self.0) + min).clamp(min, (min + count - 1).max(min))
    }
    fn next_f64(&mut self) -> f64 {
        (f64::from(self.0) / 20.0).clamp(0.0, 1.0)
    }
    fn fill_bytes(&mut self, buf: &mut [u8]) {
        buf.fill(self.0.clamp(0, 255) as u8);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seeded_is_repeatable_and_in_range() {
        let mut a = BuiltInRng::from_seed(42);
        let mut b = BuiltInRng::from_seed(42);
        for _ in 0..1000 {
            let x = a.next_below(37);
            assert_eq!(x, b.next_below(37));
            assert!((0..37).contains(&x));
            assert!((0.0..1.0).contains(&a.next_f64()));
            b.next_f64();
        }
        assert_ne!(
            BuiltInRng::from_seed(1).next_u64(),
            BuiltInRng::from_seed(2).next_u64()
        );
    }

    #[test]
    fn small_bounds_consume_nothing() {
        let mut a = BuiltInRng::from_seed(7);
        let first = a.clone().next_u64();
        assert_eq!(a.next_below(1), 0);
        assert_eq!(a.next_below(0), 0);
        assert_eq!(a.next_u64(), first);
    }

    #[test]
    fn shuffle_keeps_elements() {
        let mut v: Vec<u32> = (0..50).collect();
        BuiltInRng::from_seed(3).shuffle(&mut v);
        assert_ne!(v, (0..50).collect::<Vec<_>>());
        v.sort_unstable();
        assert_eq!(v, (0..50).collect::<Vec<_>>());
    }

    #[test]
    fn fixed_rng_matches_predictable_random() {
        let mut r = FixedRng(10);
        assert_eq!(r.next_below(21), 10);
        assert_eq!(r.next_below(5), 4);
        assert_eq!(r.next_f64(), 0.5);
        let mut b = [0u8; 3];
        r.fill_bytes(&mut b);
        assert_eq!(b, [10; 3]);
    }

    #[test]
    fn fill_bytes_uses_whole_words() {
        let mut a = BuiltInRng::from_seed(9);
        let mut b = a.clone();
        let mut buf = [0u8; 11];
        a.fill_bytes(&mut buf);
        let w0 = b.next_u64().to_le_bytes();
        let w1 = b.next_u64().to_le_bytes();
        assert_eq!(&buf[..8], &w0);
        assert_eq!(&buf[8..], &w1[..3]);
    }
}
