//! `SaveWriter`: the buffer and file writers as one in-memory writer.
//!
//! shortcut: `FileBufferWriter` drained a staging block to disk while composing. The whole world was held in
//! worker heaps before writing anyway, so this composes in memory and `write_to_file` writes the result;
//! upgrade to a streaming writer only if a single save outgrows memory.

use super::writer::GenericWriter;
use crate::error::{IoError, Result, SeekOrigin};
use std::path::Path;

#[derive(Clone, Debug)]
pub struct SaveWriter {
    buffer: Vec<u8>,
    position: usize,
    prefix_strings: bool,
}

impl SaveWriter {
    /// `new BufferWriter(prefixStr)`: `prefix_strings` puts a presence flag before every string (all current saves).
    pub fn new(prefix_strings: bool) -> Self {
        Self {
            buffer: Vec::new(),
            position: 0,
            prefix_strings,
        }
    }

    pub fn with_capacity(capacity: usize, prefix_strings: bool) -> Self {
        Self {
            buffer: Vec::with_capacity(capacity),
            position: 0,
            prefix_strings,
        }
    }

    /// Everything written so far (the high-water mark, wherever the cursor is).
    pub fn as_slice(&self) -> &[u8] {
        &self.buffer
    }

    pub fn into_vec(self) -> Vec<u8> {
        self.buffer
    }

    /// `FileBufferWriter` on `Dispose`: creates or truncates `path` and writes the composed bytes.
    pub fn write_to_file(&self, path: &Path) -> Result<()> {
        std::fs::write(path, &self.buffer).map_err(IoError::from)
    }
}

impl GenericWriter for SaveWriter {
    fn position(&self) -> u64 {
        self.position as u64
    }

    fn write_bytes(&mut self, bytes: &[u8]) {
        let end = self.position + bytes.len();
        let overlap = self
            .buffer
            .len()
            .saturating_sub(self.position)
            .min(bytes.len());
        self.buffer[self.position..self.position + overlap].copy_from_slice(&bytes[..overlap]);
        self.buffer.extend_from_slice(&bytes[overlap..]);
        self.position = end;
    }

    fn seek(&mut self, offset: i64, origin: SeekOrigin) -> Result<u64> {
        let base = match origin {
            SeekOrigin::Begin => 0,
            SeekOrigin::Current => self.position as i64,
            SeekOrigin::End => self.buffer.len() as i64,
        };
        let target = base.saturating_add(offset).max(0);
        if target > self.buffer.len() as i64 {
            return Err(IoError::SeekOutOfRange(target));
        }
        self.position = target as usize;
        Ok(self.position as u64)
    }

    fn prefix_strings(&self) -> bool {
        self.prefix_strings
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::serialization::reader::GenericReader;
    use crate::serialization::save_reader::SaveReader;
    use muo_primitives::{Point3D, Serial};

    fn le<const N: usize>(b: &[u8], at: usize) -> [u8; N] {
        b[at..at + N].try_into().unwrap()
    }

    #[test]
    fn primitives_are_little_endian_at_expected_offsets() {
        let mut w = SaveWriter::new(true);
        w.write_u8(0xAB);
        w.write_i8(-5);
        w.write_bool(true);
        w.write_bool(false);
        w.write_i16(-12345);
        w.write_u16(54321);
        w.write_i32(-123456789);
        w.write_u32(3123456789);
        w.write_i64(-1234567890123456789);
        w.write_u64(12345678901234567890);
        w.write_f64(1234.5678);
        w.write_f32(56.75);
        w.write_serial(Serial::new(0x40000001));
        assert_eq!(
            w.position(),
            1 + 1 + 1 + 1 + 2 + 2 + 4 + 4 + 8 + 8 + 8 + 4 + 4
        );
        let b = w.as_slice();
        assert_eq!(b[0], 0xAB);
        assert_eq!(b[1], (-5i8) as u8);
        assert_eq!(b[2], 1);
        assert_eq!(b[3], 0);
        assert_eq!(i16::from_le_bytes(le(b, 4)), -12345);
        assert_eq!(u16::from_le_bytes(le(b, 6)), 54321);
        assert_eq!(i32::from_le_bytes(le(b, 8)), -123456789);
        assert_eq!(u32::from_le_bytes(le(b, 12)), 3123456789);
        assert_eq!(i64::from_le_bytes(le(b, 16)), -1234567890123456789);
        assert_eq!(u64::from_le_bytes(le(b, 24)), 12345678901234567890);
        assert_eq!(f64::from_le_bytes(le(b, 32)), 1234.5678);
        assert_eq!(f32::from_le_bytes(le(b, 40)), 56.75);
        assert_eq!(u32::from_le_bytes(le(b, 44)), 0x40000001);
    }

    #[test]
    fn seek_end_uses_high_water_mark_not_current_position() {
        let mut w = SaveWriter::new(true);
        w.write_i64(1);
        w.write_i64(2);
        w.write_i64(3); // high water = 24
        w.seek(4, SeekOrigin::Begin).unwrap();
        w.write_i32(99); // position now 8, high water still 24
        assert_eq!(w.seek(0, SeekOrigin::End).unwrap(), 24);
        assert_eq!(w.position(), 24);
    }

    #[test]
    fn seek_current_and_begin_behave() {
        let mut w = SaveWriter::new(true);
        w.write_u32(0xDEADBEEF);
        assert_eq!(w.seek(2, SeekOrigin::Begin).unwrap(), 2);
        assert_eq!(w.seek(1, SeekOrigin::Current).unwrap(), 3);
        w.write_u8(0x77);
        assert_eq!(w.as_slice()[3], 0x77);
    }

    #[test]
    fn growth_preserves_content_and_position() {
        let mut w = SaveWriter::with_capacity(16, true);
        for i in 0..100i64 {
            w.write_i64(i);
        }
        assert_eq!(w.position(), 800);
        assert!(w.as_slice().len() >= 800);
        for i in 0..100usize {
            assert_eq!(i64::from_le_bytes(le(w.as_slice(), i * 8)), i as i64);
        }
    }

    #[test]
    fn span_write_crosses_growth_boundary() {
        let mut w = SaveWriter::with_capacity(8, true);
        let payload: Vec<u8> = (0..64).map(|i| i as u8 + 1).collect();
        w.write_u16(7);
        w.write_bytes(&payload);
        assert_eq!(w.position(), 66);
        assert_eq!(&w.as_slice()[2..66], &payload[..]);
    }

    #[test]
    fn encoded_int_matches_format() {
        let cases: [(i32, &[u8]); 11] = [
            (0, &[0x00]),
            (127, &[0x7F]),
            (128, &[0x80, 0x01]),
            (0x3FFF, &[0xFF, 0x7F]),
            (0x4000, &[0x80, 0x80, 0x01]),
            (0x1F_FFFF, &[0xFF, 0xFF, 0x7F]),
            (0x20_0000, &[0x80, 0x80, 0x80, 0x01]),
            (0xFFF_FFFF, &[0xFF, 0xFF, 0xFF, 0x7F]),
            (0x1000_0000, &[0x80, 0x80, 0x80, 0x80, 0x01]),
            (i32::MAX, &[0xFF, 0xFF, 0xFF, 0xFF, 0x07]),
            (-1, &[0xFF, 0xFF, 0xFF, 0xFF, 0x0F]),
        ];
        for (value, expected) in cases {
            let mut w = SaveWriter::new(true);
            w.write_encoded_int(value);
            assert_eq!(w.position() as usize, expected.len(), "{value}");
            assert_eq!(&w.as_slice()[..expected.len()], expected, "{value}");
        }
    }

    #[test]
    fn prefixed_strings_write_flag_length_and_utf8() {
        let mut w = SaveWriter::new(true);
        w.write_string("héllo Ωorld");
        let utf8 = "héllo Ωorld".as_bytes();
        let b = w.as_slice();
        assert_eq!(b[0], 1); // not-null flag
        assert_eq!(b[1] as usize, utf8.len()); // small string: 1 byte length
        assert_eq!(&b[2..2 + utf8.len()], utf8);
        assert_eq!(w.position() as usize, 2 + utf8.len());
        w.write_opt_string(None);
        assert_eq!(w.as_slice()[2 + utf8.len()], 0); // null flag
    }

    /// Text of `chars` UTF-16 units: ASCII, 2-byte, 3-byte and surrogate-pair content.
    fn mixed(chars: usize) -> String {
        let mut s = String::new();
        let mut units = 0;
        let mut i = 0;
        while units < chars {
            match i % 4 {
                0 => {
                    s.push('a');
                    units += 1;
                }
                1 => {
                    s.push('é');
                    units += 1;
                }
                2 => {
                    s.push('Ω');
                    units += 1;
                }
                _ if units + 2 <= chars => {
                    s.push('𝔘');
                    units += 2;
                }
                _ => {
                    s.push('z');
                    units += 1;
                }
            }
            i += 1;
        }
        s
    }

    #[test]
    fn string_paths_agree_across_the_scratch_boundary() {
        for chars in [84usize, 85, 86, 300] {
            let value = mixed(chars);
            assert_eq!(value.encode_utf16().count(), chars);
            let mut w = SaveWriter::with_capacity(16, true);
            w.write_string(&value);
            let b = w.as_slice();
            assert_eq!(b[0], 1);
            let (mut offset, mut length, mut shift) = (1usize, 0usize, 0);
            loop {
                let cur = b[offset];
                offset += 1;
                length |= ((cur & 0x7F) as usize) << shift;
                shift += 7;
                if cur & 0x80 == 0 {
                    break;
                }
            }
            assert_eq!(length, value.len(), "{chars}");
            assert_eq!(&b[offset..offset + length], value.as_bytes());
            assert_eq!(w.position() as usize, offset + length);
        }
    }

    #[test]
    fn date_time_writes_ticks_little_endian() {
        // DateTime is a UTC tick count on disk (any local-to-UTC conversion happens before the writer)
        let ticks = 639_189_949_230_000_000i64;
        let mut w = SaveWriter::new(true);
        w.write_date_time_ticks(ticks);
        w.write_date_time_ticks(ticks);
        assert_eq!(i64::from_le_bytes(le(w.as_slice(), 0)), ticks);
        assert_eq!(i64::from_le_bytes(le(w.as_slice(), 8)), ticks);
    }

    #[test]
    fn point3d_writes_three_ints() {
        let mut w = SaveWriter::new(true);
        w.write_point3d(Point3D::new(100, -200, 30));
        let b = w.as_slice();
        assert_eq!(w.position(), 12);
        assert_eq!(i32::from_le_bytes(le(b, 0)), 100);
        assert_eq!(i32::from_le_bytes(le(b, 4)), -200);
        assert_eq!(i32::from_le_bytes(le(b, 8)), 30);
    }

    #[test]
    fn decimal_round_trips_through_reader() {
        // 1234567.89012m = 123456789012 with scale 5
        let bits = [0xBE99_1A14u32 as i32, 0x1C, 0, 5 << 16];
        let mut w = SaveWriter::new(true);
        w.write_decimal_bits(bits);
        let mut r = SaveReader::new(w.as_slice());
        assert_eq!(r.read_decimal_bits().unwrap(), bits);
    }

    #[test]
    fn long_string_prefix_is_zero_padded_to_worst_case_width() {
        let value = "a".repeat(100);
        let mut w = SaveWriter::new(true);
        w.write_string_raw(&value);
        let b = w.as_slice();
        assert_eq!(b[0], 100 | 0x80);
        assert_eq!(b[1], 0);
        assert_eq!(w.position(), 2 + 100);
        let mut r = SaveReader::new(w.as_slice());
        assert_eq!(r.read_string_raw().unwrap(), value);
    }

    #[test]
    fn three_byte_per_char_content_round_trips() {
        for chars in [42usize, 100, 10_000] {
            let value = "二".repeat(chars);
            assert_eq!(value.len(), 3 * chars);
            let mut w = SaveWriter::with_capacity(16, true);
            w.write_string(&value);
            let mut r = SaveReader::new(w.as_slice());
            assert_eq!(r.read_string().unwrap().as_deref(), Some(value.as_str()));
        }
    }

    #[test]
    fn surrogate_pair_content_round_trips_through_raw_path() {
        let value = "😀".repeat(60); // 120 units, 240 bytes
        let mut w = SaveWriter::with_capacity(16, true);
        w.write_string_raw(&value);
        let mut r = SaveReader::new(w.as_slice());
        assert_eq!(r.read_string_raw().unwrap(), value);
    }
}
