//! Random numbers: BuiltInRng, StableRandom, WeightedValue.
//!
//! The generator is a value the `World` owns and passes as `&mut impl Rng`; there is no global one. Tests swap in
//! [`FixedRng`].

mod built_in_rng;
mod stable_random;
mod weighted_value;

pub use built_in_rng::{BuiltInRng, FixedRng};
pub use stable_random::first as stable_first;
pub use weighted_value::WeightedValue;

/// The `System.Random` surface that the server uses. The default methods are the standard algorithms
/// (Lemire's bounded integer, 53-bit doubles) on top of `next_u64`, so any generator with the same
/// `next_u64` stream gives the same values as any other `Xoshiro256**`-backed `Random`.
pub trait Rng {
    fn next_u64(&mut self) -> u64;

    /// `Random.Next()`: [0, i32::MAX)
    fn next(&mut self) -> i32 {
        loop {
            let r = self.next_u64() >> 32;
            if r != i32::MAX as u64 {
                return r as i32;
            }
        }
    }

    /// `Random.Next(maxValue)`: [0, max). `max <= 1` gives 0 without consuming randomness.
    fn next_below(&mut self, max: i32) -> i32 {
        if max > 1 {
            bounded_u32(self, max as u32) as i32
        } else {
            0
        }
    }

    /// `BuiltInRng.Next(minValue, count)`: `min + Next(count)`.
    fn next_from(&mut self, min: i32, count: i32) -> i32 {
        min.wrapping_add(self.next_below(count))
    }

    /// `Random.NextInt64()`: [0, i64::MAX)
    fn next_i64(&mut self) -> i64 {
        loop {
            let r = self.next_u64() >> 1;
            if r != i64::MAX as u64 {
                return r as i64;
            }
        }
    }

    /// `Random.NextInt64(maxValue)`: [0, max)
    fn next_i64_below(&mut self, max: i64) -> i64 {
        if max > 1 {
            bounded_u64(self, max as u64) as i64
        } else {
            0
        }
    }

    /// `BuiltInRng.Next(minValue, count)` for longs.
    fn next_i64_from(&mut self, min: i64, count: i64) -> i64 {
        min.wrapping_add(self.next_i64_below(count))
    }

    /// `Random.NextDouble()`: [0, 1)
    fn next_f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 * (1.0 / (1u64 << 53) as f64)
    }

    /// `Random.NextBytes`
    fn fill_bytes(&mut self, buf: &mut [u8]) {
        // whole words little endian, the tail from one more word
        for chunk in buf.chunks_mut(8) {
            let word = self.next_u64().to_le_bytes();
            chunk.copy_from_slice(&word[..chunk.len()]);
        }
    }

    /// `Random.Shuffle(span)`: forward Fisher-Yates, `j = Next(i, n)`.
    fn shuffle<T>(&mut self, items: &mut [T]) {
        let n = items.len();
        for i in 0..n.saturating_sub(1) {
            let j = i + self.next_below((n - i) as i32) as usize;
            if j != i {
                items.swap(i, j);
            }
        }
    }
}

fn bounded_u32<R: Rng + ?Sized>(r: &mut R, max: u32) -> u32 {
    let mut product = u64::from(max) * (r.next_u64() >> 32);
    let mut low = product as u32;
    if low < max {
        let remainder = max.wrapping_neg() % max;
        while low < remainder {
            product = u64::from(max) * (r.next_u64() >> 32);
            low = product as u32;
        }
    }
    (product >> 32) as u32
}

fn bounded_u64<R: Rng + ?Sized>(r: &mut R, max: u64) -> u64 {
    let mut product = u128::from(max) * u128::from(r.next_u64());
    let mut low = product as u64;
    if low < max {
        let remainder = max.wrapping_neg() % max;
        while low < remainder {
            product = u128::from(max) * u128::from(r.next_u64());
            low = product as u64;
        }
    }
    (product >> 64) as u64
}

/// xoshiro256** step shared by `BuiltInRng` and `StableRandom`.
fn xoshiro_next(s: &mut [u64; 4]) -> u64 {
    let result = s[1].wrapping_mul(5).rotate_left(7).wrapping_mul(9);
    let t = s[1] << 17;
    s[2] ^= s[0];
    s[3] ^= s[1];
    s[1] ^= s[2];
    s[0] ^= s[3];
    s[2] ^= t;
    s[3] = s[3].rotate_left(45);
    result
}

fn split_mix(x: &mut u64) -> u64 {
    *x = x.wrapping_add(0x9e37_79b9_7f4a_7c15);
    let mut z = *x;
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    z ^ (z >> 31)
}

fn seed_state(seed: u64) -> [u64; 4] {
    let mut x = seed;
    [
        split_mix(&mut x),
        split_mix(&mut x),
        split_mix(&mut x),
        split_mix(&mut x),
    ]
}
