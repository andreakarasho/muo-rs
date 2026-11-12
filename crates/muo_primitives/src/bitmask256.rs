//! A 256-bit mask in four words: Z levels (-128..127), sector positions (16x16 tiles), etc.
//! There is no BMI2 `pdep` fast path: the portable select below gives the same answer.

#[derive(Copy, Clone, Eq, PartialEq, Hash, Default, Debug)]
pub struct BitMask256 {
    pub bits: [u64; 4],
}

/// The bits of `[start, end]` that fall inside the 64-bit word starting at `segment_offset`.
fn segment_mask(start: i32, end: i32, segment_offset: i32) -> u64 {
    let local_start = (start - segment_offset).max(0);
    let local_end = (end - segment_offset).min(63);
    if local_start > 63 || local_end < 0 || local_start > local_end {
        return 0;
    }
    let count = local_end - local_start + 1;
    let mask = if count >= 64 {
        u64::MAX
    } else {
        (1u64 << count) - 1
    };
    mask << local_start
}

impl BitMask256 {
    pub const ALL_SET: BitMask256 = BitMask256 {
        bits: [u64::MAX; 4],
    };
    pub const ALL_CLEAR: BitMask256 = BitMask256 { bits: [0; 4] };

    /// Sets bit `index` (0-255); out of range is ignored.
    pub fn set_bit(&mut self, index: i32) {
        if let Ok(i @ 0..=255) = usize::try_from(index) {
            self.bits[i >> 6] |= 1u64 << (i & 63);
        }
    }

    pub fn clear_bit(&mut self, index: i32) {
        if let Ok(i @ 0..=255) = usize::try_from(index) {
            self.bits[i >> 6] &= !(1u64 << (i & 63));
        }
    }

    pub fn get_bit(&self, index: i32) -> bool {
        match usize::try_from(index) {
            Ok(i @ 0..=255) => self.bits[i >> 6] & (1u64 << (i & 63)) != 0,
            _ => false,
        }
    }

    fn clamp_range(start: i32, end: i32) -> Option<(i32, i32)> {
        if end < 0 || start > 255 || start > end {
            None
        } else {
            Some((start.max(0), end.min(255)))
        }
    }

    /// Sets every bit of the inclusive range.
    pub fn set_range(&mut self, start: i32, end: i32) {
        if let Some((s, e)) = Self::clamp_range(start, end) {
            for (i, w) in self.bits.iter_mut().enumerate() {
                *w |= segment_mask(s, e, i as i32 * 64);
            }
        }
    }

    pub fn clear_range(&mut self, start: i32, end: i32) {
        if let Some((s, e)) = Self::clamp_range(start, end) {
            for (i, w) in self.bits.iter_mut().enumerate() {
                *w &= !segment_mask(s, e, i as i32 * 64);
            }
        }
    }

    fn zip(&self, o: &BitMask256, f: impl Fn(u64, u64) -> u64) -> BitMask256 {
        BitMask256 {
            bits: std::array::from_fn(|i| f(self.bits[i], o.bits[i])),
        }
    }

    pub fn and(&self, o: &BitMask256) -> BitMask256 {
        self.zip(o, |a, b| a & b)
    }

    pub fn or(&self, o: &BitMask256) -> BitMask256 {
        self.zip(o, |a, b| a | b)
    }

    pub fn xor(&self, o: &BitMask256) -> BitMask256 {
        self.zip(o, |a, b| a ^ b)
    }

    /// `this & ~other`
    pub fn and_not(&self, o: &BitMask256) -> BitMask256 {
        self.zip(o, |a, b| a & !b)
    }

    pub fn not(&self) -> BitMask256 {
        BitMask256 {
            bits: self.bits.map(|w| !w),
        }
    }

    pub fn pop_count(&self) -> u32 {
        self.bits.iter().map(|w| w.count_ones()).sum()
    }

    pub fn is_empty(&self) -> bool {
        self.bits.iter().all(|w| *w == 0)
    }

    pub fn is_full(&self) -> bool {
        self.bits.iter().all(|w| *w == u64::MAX)
    }

    /// Index of the lowest set bit, or -1.
    pub fn lowest_set_bit(&self) -> i32 {
        for (i, w) in self.bits.iter().enumerate() {
            if *w != 0 {
                return i as i32 * 64 + w.trailing_zeros() as i32;
            }
        }
        -1
    }

    /// Index of the highest set bit, or -1.
    pub fn highest_set_bit(&self) -> i32 {
        for (i, w) in self.bits.iter().enumerate().rev() {
            if *w != 0 {
                return i as i32 * 64 + 63 - w.leading_zeros() as i32;
            }
        }
        -1
    }

    /// Index of the `n`th set bit (0-indexed), or -1 when fewer than `n + 1` bits are set.
    pub fn nth_set_bit(&self, n: i32) -> i32 {
        let Ok(mut n) = u32::try_from(n) else {
            return -1;
        };
        for (i, w) in self.bits.iter().enumerate() {
            let count = w.count_ones();
            if n < count {
                let mut bits = *w;
                for _ in 0..n {
                    bits &= bits - 1; // clear the lowest set bit
                }
                return i as i32 * 64 + bits.trailing_zeros() as i32;
            }
            n -= count;
        }
        -1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bits_and_ranges() {
        let mut m = BitMask256::ALL_CLEAR;
        assert!(m.is_empty() && m.lowest_set_bit() == -1 && m.highest_set_bit() == -1);
        m.set_bit(0);
        m.set_bit(63);
        m.set_bit(64);
        m.set_bit(255);
        m.set_bit(256);
        m.set_bit(-1);
        assert_eq!(m.pop_count(), 4);
        assert!(m.get_bit(63) && m.get_bit(64) && !m.get_bit(65) && !m.get_bit(256));
        assert_eq!((m.lowest_set_bit(), m.highest_set_bit()), (0, 255));
        m.clear_bit(0);
        assert_eq!(m.lowest_set_bit(), 63);

        let mut r = BitMask256::default();
        r.set_range(60, 130);
        assert_eq!(r.pop_count(), 71);
        assert!(r.get_bit(60) && r.get_bit(130) && !r.get_bit(59) && !r.get_bit(131));
        r.clear_range(-10, 63);
        assert_eq!(r.lowest_set_bit(), 64);
        r.set_range(-5, 1000);
        assert!(r.is_full());
        r.set_range(10, 5);
        r.clear_range(300, 400);
        assert!(r.is_full());
    }

    #[test]
    fn logic_and_select() {
        let mut a = BitMask256::default();
        let mut b = BitMask256::default();
        a.set_range(0, 99);
        b.set_range(50, 149);
        assert_eq!(a.and(&b).pop_count(), 50);
        assert_eq!(a.or(&b).pop_count(), 150);
        assert_eq!(a.xor(&b).pop_count(), 100);
        assert_eq!(a.and_not(&b).highest_set_bit(), 49);
        assert_eq!(a.not().pop_count(), 156);

        let mut s = BitMask256::default();
        for i in [3, 70, 71, 200] {
            s.set_bit(i);
        }
        let got: Vec<i32> = (0..5).map(|n| s.nth_set_bit(n)).collect();
        assert_eq!(got, [3, 70, 71, 200, -1]);
        assert_eq!(s.nth_set_bit(-1), -1);
    }
}
