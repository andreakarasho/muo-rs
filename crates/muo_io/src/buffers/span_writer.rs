//! `SpanWriter`: a packet byte cursor (big endian by default).
//!
//! shortcut: the writer owns a `Vec<u8>` (it does not wrap a caller's buffer or rent from a pool);
//! take the bytes with `into_vec`/`span`. Upgrade to a borrowed variant if a profile shows packet allocation.

use crate::error::{IoError, Result, SeekOrigin};
use crate::text::TextEncoding;
use crate::text::encoding::{truncate_utf16, utf16_len};
use muo_primitives::Serial;

#[derive(Clone, Debug)]
pub struct SpanWriter {
    resize: bool,
    buffer: Vec<u8>,
    position: usize,
    bytes_written: usize,
}

macro_rules! write_int {
    ($name:ident, $name_le:ident, $ty:ty) => {
        pub fn $name(&mut self, value: $ty) -> Result<()> {
            self.write_bytes(&value.to_be_bytes())
        }
        pub fn $name_le(&mut self, value: $ty) -> Result<()> {
            self.write_bytes(&value.to_le_bytes())
        }
    };
}

impl SpanWriter {
    /// A zero-filled buffer of `capacity` bytes. With `resize` false a write past the end is
    /// `IoError::BufferFull`.
    pub fn new(capacity: usize, resize: bool) -> Self {
        Self {
            resize,
            buffer: vec![0; capacity],
            position: 0,
            bytes_written: 0,
        }
    }

    /// Bytes written so far (the high-water mark, not the cursor).
    pub fn bytes_written(&self) -> usize {
        self.bytes_written
    }

    pub fn position(&self) -> usize {
        self.position
    }

    pub fn capacity(&self) -> usize {
        self.buffer.len()
    }

    /// Everything up to the cursor.
    pub fn span(&self) -> &[u8] {
        &self.buffer[..self.position]
    }

    pub fn raw_buffer(&self) -> &[u8] {
        &self.buffer
    }

    pub fn raw_buffer_mut(&mut self) -> &mut [u8] {
        &mut self.buffer
    }

    /// `ToSpan()`: the bytes up to the cursor, consuming the writer.
    pub fn into_vec(mut self) -> Vec<u8> {
        self.buffer.truncate(self.position);
        self.buffer
    }

    fn set_position(&mut self, value: usize) {
        self.position = value;
        self.bytes_written = self.bytes_written.max(value);
    }

    fn grow(&mut self, additional: usize) {
        let new_size = (self.bytes_written + additional).max(self.buffer.len() * 2);
        self.buffer.resize(new_size, 0);
    }

    fn grow_if_needed(&mut self, count: usize) -> Result<()> {
        if self.position + count > self.buffer.len() {
            if !self.resize {
                return Err(IoError::BufferFull);
            }
            self.grow(count);
        }
        Ok(())
    }

    pub fn ensure_capacity(&mut self, capacity: usize) -> Result<()> {
        if capacity > self.buffer.len() {
            if !self.resize {
                return Err(IoError::BufferFull);
            }
            self.grow(capacity - self.bytes_written);
        }
        Ok(())
    }

    /// `Write(ReadOnlySpan<byte>)`.
    pub fn write_bytes(&mut self, bytes: &[u8]) -> Result<()> {
        self.grow_if_needed(bytes.len())?;
        self.buffer[self.position..self.position + bytes.len()].copy_from_slice(bytes);
        self.set_position(self.position + bytes.len());
        Ok(())
    }

    pub fn write_bool(&mut self, value: bool) -> Result<()> {
        self.write_u8(u8::from(value))
    }

    pub fn write_u8(&mut self, value: u8) -> Result<()> {
        self.write_bytes(&[value])
    }

    pub fn write_i8(&mut self, value: i8) -> Result<()> {
        self.write_u8(value as u8)
    }

    write_int!(write_i16, write_i16_le, i16);
    write_int!(write_u16, write_u16_le, u16);
    write_int!(write_i32, write_i32_le, i32);
    write_int!(write_u32, write_u32_le, u32);
    write_int!(write_i64, write_i64_le, i64);
    write_int!(write_u64, write_u64_le, u64);

    pub fn write_serial(&mut self, serial: Serial) -> Result<()> {
        self.write_u32(serial.value())
    }

    /// `Write(ReadOnlySpan<char>, encoding, fixedLength)`. `fixed_length` counts characters (UTF-16 units): the text
    /// is cut to it and zero padded up to `fixed_length * unit` bytes. A UTF-8 string whose encoding is longer than
    /// the field is written whole.
    pub fn write_string(
        &mut self,
        value: &str,
        encoding: TextEncoding,
        fixed_length: Option<usize>,
    ) -> Result<()> {
        let char_length = fixed_length.map_or(utf16_len(value), |n| n.min(utf16_len(value)));
        let src = truncate_utf16(value, char_length);
        let bytes = encoding.encode(&src);

        let unit = encoding.unit_size();
        let src_len = utf16_len(&src);
        let mut byte_count = bytes.len();
        if let Some(n) = fixed_length.filter(|&n| n > src_len) {
            byte_count += (n - src_len) * unit;
        }
        if byte_count == 0 {
            return Ok(());
        }

        self.write_bytes(&bytes)?;
        if let Some(n) = fixed_length {
            let extra = (n * unit).saturating_sub(bytes.len());
            if extra > 0 {
                self.clear(extra)?;
            }
        }
        Ok(())
    }

    pub fn write_little_uni(&mut self, value: &str) -> Result<()> {
        self.write_string(value, TextEncoding::LittleUni, None)
    }

    pub fn write_little_uni_null(&mut self, value: &str) -> Result<()> {
        self.write_little_uni(value)?;
        self.write_u16(0)
    }

    pub fn write_little_uni_fixed(&mut self, value: &str, fixed_length: usize) -> Result<()> {
        self.write_string(value, TextEncoding::LittleUni, Some(fixed_length))
    }

    pub fn write_big_uni(&mut self, value: &str) -> Result<()> {
        self.write_string(value, TextEncoding::BigUni, None)
    }

    pub fn write_big_uni_null(&mut self, value: &str) -> Result<()> {
        self.write_big_uni(value)?;
        self.write_u16(0)
    }

    pub fn write_big_uni_fixed(&mut self, value: &str, fixed_length: usize) -> Result<()> {
        self.write_string(value, TextEncoding::BigUni, Some(fixed_length))
    }

    pub fn write_utf8(&mut self, value: &str) -> Result<()> {
        self.write_string(value, TextEncoding::Utf8, None)
    }

    pub fn write_utf8_null(&mut self, value: &str) -> Result<()> {
        self.write_utf8(value)?;
        self.write_u8(0)
    }

    pub fn write_ascii(&mut self, value: &str) -> Result<()> {
        self.write_string(value, TextEncoding::Ascii, None)
    }

    /// `WriteAscii(char)`: one byte, the low 8 bits of the char.
    pub fn write_ascii_char(&mut self, chr: char) -> Result<()> {
        self.write_u8(u32::from(chr) as u8)
    }

    pub fn write_ascii_null(&mut self, value: &str) -> Result<()> {
        self.write_ascii(value)?;
        self.write_u8(0)
    }

    pub fn write_ascii_fixed(&mut self, value: &str, fixed_length: usize) -> Result<()> {
        self.write_string(value, TextEncoding::Ascii, Some(fixed_length))
    }

    pub fn write_latin1(&mut self, value: &str) -> Result<()> {
        self.write_string(value, TextEncoding::Latin1, None)
    }

    pub fn write_latin1_fixed(&mut self, value: &str, fixed_length: usize) -> Result<()> {
        self.write_string(value, TextEncoding::Latin1, Some(fixed_length))
    }

    pub fn write_latin1_null(&mut self, value: &str) -> Result<()> {
        self.write_latin1(value)?;
        self.write_u8(0)
    }

    /// Writes `count` zero bytes.
    pub fn clear(&mut self, count: usize) -> Result<()> {
        self.grow_if_needed(count)?;
        self.buffer[self.position..self.position + count].fill(0);
        self.set_position(self.position + count);
        Ok(())
    }

    /// `Seek`: `End` is relative to the bytes written; a target past the capacity grows a resizable writer.
    pub fn seek(&mut self, offset: i64, origin: SeekOrigin) -> Result<usize> {
        let base = match origin {
            SeekOrigin::Begin => 0,
            SeekOrigin::Current => self.position as i64,
            SeekOrigin::End => self.bytes_written as i64,
        };
        let target = base.saturating_add(offset);
        if target < 0 {
            return Err(IoError::SeekOutOfRange(target));
        }
        let target = target as usize;
        if target > self.buffer.len() {
            if !self.resize {
                return Err(IoError::BufferFull);
            }
            self.grow(target - self.buffer.len() + 1);
        }
        self.set_position(target);
        Ok(target)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixed(n: usize) -> SpanWriter {
        SpanWriter::new(n, false)
    }

    #[test]
    fn resizes_and_overflows() {
        let mut w = SpanWriter::new(8, true);
        w.write_i64(0x1024).unwrap();
        w.write_i64(0x1024).unwrap();
        assert!(w.capacity() > 8);

        let mut w = fixed(8);
        w.write_i64(0x1024).unwrap();
        assert_eq!(w.span(), [0, 0, 0, 0, 0, 0, 0x10, 0x24]);
        assert!(matches!(w.write_i64(0x1024), Err(IoError::BufferFull)));
    }

    #[test]
    fn integers_golden() {
        let mut w = fixed(2);
        w.write_bool(true).unwrap();
        w.write_bool(false).unwrap();
        assert_eq!(w.span(), [1, 0]);

        let mut w = fixed(2);
        w.write_i8(-1).unwrap();
        w.write_i8(127).unwrap();
        assert_eq!(w.span(), [0xFF, 0x7F]);

        let mut w = fixed(8);
        w.write_i16(0x1234).unwrap();
        w.write_i16_le(0x1234).unwrap();
        w.write_u16(0x1234).unwrap();
        w.write_u16_le(0x1234).unwrap();
        assert_eq!(w.span(), [0x12, 0x34, 0x34, 0x12, 0x12, 0x34, 0x34, 0x12]);

        let mut w = fixed(16);
        w.write_i32(0x12345678).unwrap();
        w.write_i32_le(0x12345678).unwrap();
        w.write_u32(0x12345678).unwrap();
        w.write_u32_le(0x12345678).unwrap();
        assert_eq!(
            w.span(),
            [
                0x12, 0x34, 0x56, 0x78, 0x78, 0x56, 0x34, 0x12, 0x12, 0x34, 0x56, 0x78, 0x78, 0x56,
                0x34, 0x12
            ]
        );

        let mut w = fixed(16);
        w.write_i64(0x123456789ABCDEF0).unwrap();
        w.write_u64(0x123456789ABCDEF0).unwrap();
        assert_eq!(&w.span()[..8], &w.span()[8..]);
        assert_eq!(
            &w.span()[..8],
            [0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0]
        );

        let mut w = fixed(4);
        w.write_serial(Serial::new(0x4000_0001)).unwrap();
        assert_eq!(w.span(), [0x40, 0, 0, 1]);
    }

    #[test]
    fn write_span() {
        let mut w = fixed(6);
        w.write_bytes(&[1, 2, 3]).unwrap();
        w.write_bytes(&[1, 2, 3]).unwrap();
        assert_eq!(w.span(), [1, 2, 3, 1, 2, 3]);
    }

    #[test]
    fn strings() {
        let mut w = fixed(5);
        w.write_ascii("Hello").unwrap();
        assert_eq!(w.span(), b"Hello");

        let mut w = fixed(6);
        w.write_ascii_null("Hello").unwrap();
        assert_eq!(w.span(), b"Hello\0");

        let mut w = fixed(10);
        w.write_ascii_fixed("Hello", 10).unwrap();
        assert_eq!(w.span(), b"Hello\0\0\0\0\0");

        let mut w = fixed(6);
        w.write_utf8_null("Hello").unwrap();
        assert_eq!(w.span(), b"Hello\0");

        let mut w = fixed(12);
        w.write_little_uni_null("Hello").unwrap();
        assert_eq!(w.span(), b"H\0e\0l\0l\0o\0\0\0");

        let mut w = fixed(12);
        w.write_big_uni_null("Hello").unwrap();
        assert_eq!(w.span(), b"\0H\0e\0l\0l\0o\0\0");

        // fixed length cuts longer text and pads shorter text, per UTF-16 unit
        let mut w = fixed(8);
        w.write_big_uni_fixed("Hello", 3).unwrap();
        w.write_big_uni_fixed("A", 1).unwrap();
        assert_eq!(w.span(), b"\0H\0e\0l\0A");
        let mut w = fixed(6);
        w.write_little_uni_fixed("A", 3).unwrap();
        assert_eq!(w.span(), b"A\0\0\0\0\0");

        let mut w = fixed(4);
        w.write_latin1("a\u{E9}\u{263A}").unwrap();
        assert_eq!(w.span(), [b'a', 0xE9, b'?']);
        w.write_ascii_char('z').unwrap();
        assert_eq!(w.span()[3], b'z');
    }

    #[test]
    fn clear_and_seek() {
        let mut w = fixed(10);
        w.write_u8(1).unwrap();
        w.clear(5).unwrap();
        w.write_u8(2).unwrap();
        assert_eq!(w.span(), [1, 0, 0, 0, 0, 0, 2]);

        let mut w = fixed(10);
        w.write_bytes(&[1, 2, 3]).unwrap();
        assert_eq!(w.seek(1, SeekOrigin::Begin).unwrap(), 1);
        assert_eq!(w.bytes_written(), 3);

        let mut w = fixed(10);
        w.write_bytes(&[1, 2]).unwrap();
        assert_eq!(w.seek(2, SeekOrigin::Current).unwrap(), 4);

        let mut w = fixed(10);
        w.write_bytes(&[1, 2, 3]).unwrap();
        assert_eq!(w.seek(-1, SeekOrigin::End).unwrap(), 2);

        assert!(matches!(
            w.seek(-1, SeekOrigin::Begin),
            Err(IoError::SeekOutOfRange(_))
        ));
        assert!(matches!(
            w.seek(11, SeekOrigin::Begin),
            Err(IoError::BufferFull)
        ));

        let mut w = SpanWriter::new(10, true);
        assert_eq!(w.seek(20, SeekOrigin::Begin).unwrap(), 20);
        assert!(w.capacity() >= 20);
    }

    #[test]
    fn capacity_and_owner() {
        let mut w = fixed(10);
        assert!(matches!(w.ensure_capacity(50), Err(IoError::BufferFull)));
        let mut w = SpanWriter::new(10, true);
        w.ensure_capacity(50).unwrap();
        assert!(w.capacity() >= 50);

        let mut w = fixed(10);
        w.write_bytes(&[1, 2, 3]).unwrap();
        assert_eq!(w.into_vec(), [1, 2, 3]);
        assert!(SpanWriter::new(10, false).into_vec().is_empty());
    }

    #[test]
    fn bytes_written_is_a_high_water_mark() {
        let mut w = fixed(10);
        assert_eq!(w.bytes_written(), 0);
        w.write_u8(1).unwrap();
        w.write_u16(0x0203).unwrap();
        assert_eq!(w.bytes_written(), 3);
        w.seek(0, SeekOrigin::Begin).unwrap();
        w.write_u8(0xFF).unwrap();
        assert_eq!(w.bytes_written(), 3);
    }

    #[test]
    fn plain_strings_without_terminator() {
        let mut w = fixed(5);
        w.write_utf8("Hello").unwrap();
        assert_eq!(w.span(), b"Hello");
        let mut w = fixed(10);
        w.write_little_uni("Hello").unwrap();
        assert_eq!(w.span(), b"H e l l o ");
        let mut w = fixed(10);
        w.write_big_uni("Hello").unwrap();
        assert_eq!(w.span(), b" H e l l o");
        assert_eq!(w.position(), 10);
    }
}
