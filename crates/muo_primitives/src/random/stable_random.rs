//! Seeded, reproducible random values.

use super::{Rng, seed_state, xoshiro_next};

struct Stable([u64; 4]);

impl Rng for Stable {
    fn next_u64(&mut self) -> u64 {
        xoshiro_next(&mut self.0)
    }
}

/// The first long of a new Rng: a stable value for a specific set of inputs. `max_value` is
/// reinterpreted as `ulong`, so it must be positive.
pub fn first(seed: u64, max_value: i64) -> i64 {
    let max = max_value as u64;
    let mut rng = Stable(seed_state(seed));
    // Lemire's bounded draw over the 64-bit stream (same as Rng::next_i64_below without the <=1 shortcut)
    let mut product = u128::from(max) * u128::from(rng.next_u64());
    let mut low = product as u64;
    if low < max {
        let remainder = max.wrapping_neg() % max;
        while low < remainder {
            product = u128::from(max) * u128::from(rng.next_u64());
            low = product as u64;
        }
    }
    (product >> 64) as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stable_for_same_input_and_in_range() {
        for seed in 0..200u64 {
            let v = first(seed, 1000);
            assert!((0..1000).contains(&v));
            assert_eq!(v, first(seed, 1000));
        }
        assert_ne!(first(1, i64::MAX), first(2, i64::MAX));
    }

    #[test]
    fn splitmix_golden() {
        // First SplitMix64 outputs for seed 0 (published reference values).
        let mut x = 0u64;
        assert_eq!(super::super::split_mix(&mut x), 0xE220_A839_7B1D_CDAF);
        assert_eq!(super::super::split_mix(&mut x), 0x6E78_9E6A_A1B9_65F4);
    }

    #[test]
    fn xoshiro_golden() {
        // xoshiro256** from state {1, 2, 3, 4}: reference outputs 11520, 0, 1509978240, 1215971899390074240
        let mut s = [1u64, 2, 3, 4];
        let got: Vec<u64> = (0..4).map(|_| xoshiro_next(&mut s)).collect();
        assert_eq!(got, [11520, 0, 1_509_978_240, 1_215_971_899_390_074_240]);
    }
}
