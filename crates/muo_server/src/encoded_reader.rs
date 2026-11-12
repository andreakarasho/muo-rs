//! The reader an `EncodedPacketHandler` (0xD7) gets after the target
//! entity serial and sub-packet id. Truncated input is an `Err`, never a panic.

use muo_io::{Result, SpanReader};
use muo_primitives::Point3D;

pub struct EncodedReader<'a> {
    reader: SpanReader<'a>,
}

impl<'a> EncodedReader<'a> {
    pub fn new(body: &'a [u8]) -> Self {
        Self {
            reader: SpanReader::new(body),
        }
    }

    /// The whole packet body, for `Trace`.
    pub fn buffer(&self) -> &'a [u8] {
        self.reader.buffer()
    }

    /// A type byte that isn't 0 reads as 0 without consuming the value.
    pub fn read_i32(&mut self) -> Result<i32> {
        if self.reader.read_byte()? != 0 {
            Ok(0)
        } else {
            self.reader.read_i32()
        }
    }

    pub fn read_point3d(&mut self) -> Result<Point3D> {
        if self.reader.read_byte()? != 3 {
            return Ok(Point3D::ZERO);
        }
        let x = self.reader.read_i16()?;
        let y = self.reader.read_i16()?;
        let z = self.reader.read_byte()?;
        Ok(Point3D::new(x as i32, y as i32, z as i32))
    }

    pub fn read_unicode_string_safe(&mut self) -> Result<String> {
        if self.reader.read_byte()? != 2 {
            return Ok(String::new());
        }
        let len = self.reader.read_u16()? as usize;
        self.reader.read_big_uni_safe(Some(len))
    }

    pub fn read_unicode_string(&mut self) -> Result<String> {
        if self.reader.read_byte()? != 2 {
            return Ok(String::new());
        }
        let len = self.reader.read_u16()? as usize;
        self.reader.read_big_uni(Some(len))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn typed_values() {
        let mut r =
            EncodedReader::new(&[0, 0, 0, 1, 2, 3, 0, 5, 0, 7, 9, 2, 0, 2, 0, b'h', 0, b'i']);
        assert_eq!(r.read_i32().unwrap(), 0x102);
        assert_eq!(r.read_i32().unwrap(), 0); // tag 3: no value read
        assert_eq!(r.read_point3d().unwrap(), Point3D::ZERO);
        assert!(r.read_unicode_string_safe().is_ok());
        assert!(EncodedReader::new(&[0, 1]).read_i32().is_err());
    }

    #[test]
    fn point_and_string() {
        let mut r = EncodedReader::new(&[3, 0, 5, 0, 7, 9, 2, 0, 2, 0, b'h', 0, b'i']);
        assert_eq!(r.read_point3d().unwrap(), Point3D::new(5, 7, 9));
        assert_eq!(r.read_unicode_string().unwrap(), "hi");
    }
}
