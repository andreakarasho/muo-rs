//! `SpanReader`: a bounds-checked cursor over packet bytes (big endian by default).

use crate::error::{IoError, Result, SeekOrigin};
use crate::text::TextEncoding;

#[derive(Clone, Debug)]
pub struct SpanReader<'a> {
    buffer: &'a [u8],
    position: usize,
}

macro_rules! read_int {
    ($name:ident, $ty:ty, $from:ident) => {
        pub fn $name(&mut self) -> Result<$ty> {
            const N: usize = std::mem::size_of::<$ty>();
            let bytes = self.take(N)?;
            Ok(<$ty>::$from(
                bytes.try_into().map_err(|_| IoError::EndOfStream)?,
            ))
        }
    };
}

impl<'a> SpanReader<'a> {
    pub fn new(buffer: &'a [u8]) -> Self {
        Self {
            buffer,
            position: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    pub fn position(&self) -> usize {
        self.position
    }

    pub fn remaining(&self) -> usize {
        self.buffer.len() - self.position
    }

    pub fn buffer(&self) -> &'a [u8] {
        self.buffer
    }

    fn take(&mut self, n: usize) -> Result<&'a [u8]> {
        if n > self.remaining() {
            return Err(IoError::EndOfStream);
        }
        let s = &self.buffer[self.position..self.position + n];
        self.position += n;
        Ok(s)
    }

    pub fn read_byte(&mut self) -> Result<u8> {
        Ok(self.take(1)?[0])
    }

    pub fn read_boolean(&mut self) -> Result<bool> {
        Ok(self.read_byte()? > 0)
    }

    pub fn read_sbyte(&mut self) -> Result<i8> {
        Ok(self.read_byte()? as i8)
    }

    read_int!(read_i16, i16, from_be_bytes);
    read_int!(read_i16_le, i16, from_le_bytes);
    read_int!(read_u16, u16, from_be_bytes);
    read_int!(read_u16_le, u16, from_le_bytes);
    read_int!(read_i32, i32, from_be_bytes);
    read_int!(read_u32, u32, from_be_bytes);
    read_int!(read_u32_le, u32, from_le_bytes);
    read_int!(read_i64, i64, from_be_bytes);
    read_int!(read_u64, u64, from_be_bytes);

    /// `ReadString(encoding, safeString, fixedLength)`; `fixed_length` counts characters (UTF-16 units for the
    /// Unicode encodings), `None` reads up to the terminator or the end of the buffer.
    ///
    /// A terminator ends the string; for fixed lengths the whole field is consumed regardless. Safe mode drops
    /// control codes and non-characters.
    pub fn read_string(
        &mut self,
        encoding: TextEncoding,
        safe: bool,
        fixed_length: Option<usize>,
    ) -> Result<String> {
        if fixed_length == Some(0) {
            return Ok(String::new());
        }
        let unit = encoding.unit_size();
        let remaining = self.remaining();
        let size = match fixed_length {
            Some(n) => {
                let size = n.checked_mul(unit).ok_or(IoError::EndOfStream)?;
                if size > remaining {
                    return Err(IoError::EndOfStream);
                }
                size
            }
            // an odd trailing byte of a UTF-16 string is left unread
            None => remaining - remaining % unit,
        };

        let span = &self.buffer[self.position..self.position + size];
        let terminator = if unit == 2 {
            span.as_chunks::<2>()
                .0
                .iter()
                .position(|c| *c == [0, 0])
                .map(|i| i * 2)
        } else {
            span.iter().position(|&b| b == 0)
        };

        let text = &span[..terminator.unwrap_or(size)];
        self.position += match terminator {
            Some(i) if fixed_length.is_none() => i + unit,
            _ => size,
        };

        Ok(encoding.decode(text, safe))
    }

    pub fn read_little_uni_safe(&mut self, fixed_length: Option<usize>) -> Result<String> {
        self.read_string(TextEncoding::LittleUni, true, fixed_length)
    }

    pub fn read_little_uni(&mut self, fixed_length: Option<usize>) -> Result<String> {
        self.read_string(TextEncoding::LittleUni, false, fixed_length)
    }

    pub fn read_big_uni_safe(&mut self, fixed_length: Option<usize>) -> Result<String> {
        self.read_string(TextEncoding::BigUni, true, fixed_length)
    }

    pub fn read_big_uni(&mut self, fixed_length: Option<usize>) -> Result<String> {
        self.read_string(TextEncoding::BigUni, false, fixed_length)
    }

    pub fn read_utf8_safe(&mut self, fixed_length: Option<usize>) -> Result<String> {
        self.read_string(TextEncoding::Utf8, true, fixed_length)
    }

    pub fn read_utf8(&mut self, fixed_length: Option<usize>) -> Result<String> {
        self.read_string(TextEncoding::Utf8, false, fixed_length)
    }

    pub fn read_ascii_safe(&mut self, fixed_length: Option<usize>) -> Result<String> {
        self.read_string(TextEncoding::Ascii, true, fixed_length)
    }

    pub fn read_ascii(&mut self, fixed_length: Option<usize>) -> Result<String> {
        self.read_string(TextEncoding::Ascii, false, fixed_length)
    }

    pub fn read_latin1_safe(&mut self, fixed_length: Option<usize>) -> Result<String> {
        self.read_string(TextEncoding::Latin1, true, fixed_length)
    }

    pub fn read_latin1(&mut self, fixed_length: Option<usize>) -> Result<String> {
        self.read_string(TextEncoding::Latin1, false, fixed_length)
    }

    /// `Seek`: errors (never clamps) when the target is before the start or past the end.
    pub fn seek(&mut self, offset: i64, origin: SeekOrigin) -> Result<usize> {
        let base = match origin {
            SeekOrigin::Begin => 0,
            SeekOrigin::Current => self.position as i64,
            SeekOrigin::End => self.buffer.len() as i64,
        };
        let target = base.saturating_add(offset);
        if target < 0 || target > self.buffer.len() as i64 {
            return Err(IoError::SeekOutOfRange(target));
        }
        self.position = target as usize;
        Ok(self.position)
    }

    /// `Read(Span<byte>)`: copies what is left, up to `bytes.len()`, and returns the count.
    pub fn read(&mut self, bytes: &mut [u8]) -> usize {
        let n = bytes.len().min(self.remaining());
        bytes[..n].copy_from_slice(&self.buffer[self.position..self.position + n]);
        self.position += n;
        n
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn eos<T: std::fmt::Debug>(r: Result<T>) {
        assert!(matches!(r, Err(IoError::EndOfStream)), "{r:?}");
    }

    #[test]
    fn bytes_and_bool() {
        let mut r = SpanReader::new(&[0x12, 0x34]);
        assert_eq!(r.read_byte().unwrap(), 0x12);
        assert_eq!(r.read_byte().unwrap(), 0x34);
        assert_eq!(r.position(), 2);
        eos(r.read_byte());

        let mut r = SpanReader::new(&[0, 1, 2, 255]);
        assert!(!r.read_boolean().unwrap());
        assert!(
            r.read_boolean().unwrap() && r.read_boolean().unwrap() && r.read_boolean().unwrap()
        );

        let mut r = SpanReader::new(&[0xFF, 0x7F]);
        assert_eq!(
            (r.read_sbyte().unwrap(), r.read_sbyte().unwrap()),
            (-1, 127)
        );
    }

    #[test]
    fn integers_golden() {
        assert_eq!(SpanReader::new(&[0x12, 0x34]).read_i16().unwrap(), 0x1234);
        assert_eq!(
            SpanReader::new(&[0x34, 0x12]).read_i16_le().unwrap(),
            0x1234
        );
        assert_eq!(SpanReader::new(&[0x12, 0x34]).read_u16().unwrap(), 0x1234);
        assert_eq!(
            SpanReader::new(&[0x34, 0x12]).read_u16_le().unwrap(),
            0x1234
        );
        assert_eq!(
            SpanReader::new(&[0x12, 0x34, 0x56, 0x78])
                .read_i32()
                .unwrap(),
            0x12345678
        );
        assert_eq!(
            SpanReader::new(&[0x12, 0x34, 0x56, 0x78])
                .read_u32()
                .unwrap(),
            0x12345678
        );
        assert_eq!(
            SpanReader::new(&[0x78, 0x56, 0x34, 0x12])
                .read_u32_le()
                .unwrap(),
            0x12345678
        );
        let eight = [0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0];
        assert_eq!(
            SpanReader::new(&eight).read_i64().unwrap(),
            0x123456789ABCDEF0
        );
        assert_eq!(
            SpanReader::new(&eight).read_u64().unwrap(),
            0x123456789ABCDEF0
        );
        eos(SpanReader::new(&[0x12]).read_i16());
        eos(SpanReader::new(&[0x12, 0x34, 0x56]).read_i32());
        eos(SpanReader::new(&eight[..7]).read_i64());
    }

    #[test]
    fn ascii() {
        let mut r = SpanReader::new(b"Hello");
        assert_eq!(r.read_ascii(None).unwrap(), "Hello");
        assert_eq!(r.position(), 5);

        let mut r = SpanReader::new(b"Hel\0o");
        assert_eq!(r.read_ascii(None).unwrap(), "Hel");
        assert_eq!(r.position(), 4);

        let mut r = SpanReader::new(b"Hello\0\0\0\0\0");
        assert_eq!(r.read_ascii(Some(10)).unwrap(), "Hello");
        assert_eq!(r.position(), 10);

        eos(SpanReader::new(b"Hel").read_ascii(Some(10)));

        let mut r = SpanReader::new(&[b'H', b'e', 0xFF, b'l', b'o']);
        assert_eq!(r.read_ascii_safe(None).unwrap(), "He?lo");
        assert_eq!(r.position(), 5);
    }

    #[test]
    fn utf8() {
        let mut r = SpanReader::new(b"Hello");
        assert_eq!(r.read_utf8(None).unwrap(), "Hello");
        let mut r = SpanReader::new(b"Hi\0Bye");
        assert_eq!(r.read_utf8(None).unwrap(), "Hi");
        assert_eq!(r.position(), 3);
    }

    #[test]
    fn little_uni() {
        let mut r = SpanReader::new(b"H\0i\0");
        assert_eq!(r.read_little_uni(None).unwrap(), "Hi");
        assert_eq!(r.position(), 4);

        let mut r = SpanReader::new(b"H\0i\0\0\0X\0");
        assert_eq!(r.read_little_uni(None).unwrap(), "Hi");
        assert_eq!(r.position(), 6);

        let mut r = SpanReader::new(b"H\0i\0\0\0\0\0");
        assert_eq!(r.read_little_uni(Some(4)).unwrap(), "Hi");
        assert_eq!(r.position(), 8);

        // C1 control (0x0085 = NEL) is filtered
        let mut r = SpanReader::new(&[b'H', 0, 0x85, 0, b'i', 0]);
        assert_eq!(r.read_little_uni_safe(None).unwrap(), "Hi");
        assert_eq!(r.position(), 6);

        // odd byte count: the last byte is ignored
        let mut r = SpanReader::new(&[b'H', 0, b'i', 0, 0xFF]);
        assert_eq!(r.read_little_uni(None).unwrap(), "Hi");
        assert_eq!(r.position(), 4);
    }

    #[test]
    fn big_uni() {
        let mut r = SpanReader::new(b"\0H\0i");
        assert_eq!(r.read_big_uni(None).unwrap(), "Hi");
        assert_eq!(r.position(), 4);

        let mut r = SpanReader::new(b"\0H\0i\0\0\0X");
        assert_eq!(r.read_big_uni(None).unwrap(), "Hi");
        assert_eq!(r.position(), 6);

        let mut r = SpanReader::new(b"\0H\0i\0\0\0\0");
        assert_eq!(r.read_big_uni(Some(4)).unwrap(), "Hi");
        assert_eq!(r.position(), 8);

        let mut r = SpanReader::new(&[0, b'H', 0, 0x85, 0, b'i']);
        assert_eq!(r.read_big_uni_safe(None).unwrap(), "Hi");
        assert_eq!(r.position(), 6);

        let mut r = SpanReader::new(&[0, b'H', 0, b'i', 0xFF]);
        assert_eq!(r.read_big_uni(None).unwrap(), "Hi");
        assert_eq!(r.position(), 4);
    }

    #[test]
    fn seek() {
        let data = [1, 2, 3, 4, 5];
        let mut r = SpanReader::new(&data);
        r.read_byte().unwrap();
        r.read_byte().unwrap();
        assert_eq!(r.seek(0, SeekOrigin::Begin).unwrap(), 0);
        assert_eq!(r.read_byte().unwrap(), 1);

        let mut r = SpanReader::new(&data);
        r.read_byte().unwrap();
        assert_eq!(r.seek(2, SeekOrigin::Current).unwrap(), 3);
        assert_eq!(r.read_byte().unwrap(), 4);

        let mut r = SpanReader::new(&data);
        assert_eq!(r.seek(-2, SeekOrigin::End).unwrap(), 3);
        assert_eq!(r.read_byte().unwrap(), 4);

        assert!(matches!(
            r.seek(-1, SeekOrigin::Begin),
            Err(IoError::SeekOutOfRange(_))
        ));
        assert!(matches!(
            r.seek(10, SeekOrigin::Begin),
            Err(IoError::SeekOutOfRange(_))
        ));
    }

    #[test]
    fn read_into() {
        let data = [1, 2, 3, 4, 5];
        let mut r = SpanReader::new(&data);
        let mut dest = [0u8; 3];
        assert_eq!(r.read(&mut dest), 3);
        assert_eq!((dest, r.position()), ([1, 2, 3], 3));

        let mut r = SpanReader::new(&data[..3]);
        let mut dest = [0u8; 5];
        assert_eq!(r.read(&mut dest), 3);
        assert_eq!(&dest[..3], &[1, 2, 3]);

        assert_eq!(r.read(&mut []), 0);
        r.seek(3, SeekOrigin::Begin).unwrap();
        assert_eq!(r.read(&mut dest), 0);
        assert_eq!(r.position(), 3);
    }

    #[test]
    fn strings_edge_cases() {
        let mut r = SpanReader::new(&[1, 2, 3]);
        assert_eq!(r.read_ascii(Some(0)).unwrap(), "");
        assert_eq!(r.position(), 0);

        let mut r = SpanReader::new(b"AB\0CD\0EF");
        assert_eq!(r.read_ascii(None).unwrap(), "AB");
        assert_eq!(r.read_ascii(None).unwrap(), "CD");
        assert_eq!(r.read_ascii(None).unwrap(), "EF");
        assert_eq!(r.position(), 8);
    }
}
