//! `GenericWriter`: the save file writer.
//!
//! Little endian like the reader. Writes cannot fail: the implementation composes in memory, and files are
//! written from the finished bytes.
//!
//! The server adds entity writes the same way as the reader (see `reader.rs`): an entity reference is
//! `write_serial(entity serial)` or `Serial::MINUS_ONE` for none/deleted; a map or race is `write_map_index`.

use super::reader::Guid;
use crate::error::{Result, SeekOrigin};
use crate::text::TextDefinition;
use muo_primitives::{Point2D, Point3D, Rectangle2D, Rectangle3D, Serial};
use std::net::IpAddr;

/// `EncodedIntWidth`: bytes a 7-bit encoding of a non-negative `value` needs.
fn encoded_int_width(value: u32) -> usize {
    match value {
        0..0x80 => 1,
        0x80..0x4000 => 2,
        0x4000..0x20_0000 => 3,
        0x20_0000..0x1000_0000 => 4,
        _ => 5,
    }
}

/// Appends the minimal 7-bit encoding of `value` (negative numbers take 5 bytes).
pub fn push_encoded_int(out: &mut Vec<u8>, value: i32) {
    let mut v = value as u32;
    while v >= 0x80 {
        out.push((v | 0x80) as u8);
        v >>= 7;
    }
    out.push(v as u8);
}

pub trait GenericWriter {
    /// Where the next byte goes.
    fn position(&self) -> u64;

    /// `Write(ReadOnlySpan<byte>)`.
    fn write_bytes(&mut self, bytes: &[u8]);

    /// Moves the cursor (to patch earlier bytes). `End` is relative to the bytes written; seeking past them is
    /// an error.
    fn seek(&mut self, offset: i64, origin: SeekOrigin) -> Result<u64>;

    /// Whether `write_string` puts a presence flag before the text (`prefixStrings`).
    fn prefix_strings(&self) -> bool;

    fn write_u8(&mut self, v: u8) {
        self.write_bytes(&[v]);
    }

    fn write_i8(&mut self, v: i8) {
        self.write_u8(v as u8);
    }

    fn write_bool(&mut self, v: bool) {
        self.write_u8(u8::from(v));
    }

    fn write_i16(&mut self, v: i16) {
        self.write_bytes(&v.to_le_bytes());
    }

    fn write_u16(&mut self, v: u16) {
        self.write_bytes(&v.to_le_bytes());
    }

    fn write_i32(&mut self, v: i32) {
        self.write_bytes(&v.to_le_bytes());
    }

    fn write_u32(&mut self, v: u32) {
        self.write_bytes(&v.to_le_bytes());
    }

    fn write_i64(&mut self, v: i64) {
        self.write_bytes(&v.to_le_bytes());
    }

    fn write_u64(&mut self, v: u64) {
        self.write_bytes(&v.to_le_bytes());
    }

    fn write_f32(&mut self, v: f32) {
        self.write_bytes(&v.to_le_bytes());
    }

    fn write_f64(&mut self, v: f64) {
        self.write_bytes(&v.to_le_bytes());
    }

    fn write_serial(&mut self, v: Serial) {
        self.write_u32(v.value());
    }

    fn write_encoded_int(&mut self, v: i32) {
        let mut buf = Vec::with_capacity(5);
        push_encoded_int(&mut buf, v);
        self.write_bytes(&buf);
    }

    /// `WriteRaw`: encoded length then UTF-8, byte-identical with existing save files. The writer reserves the length prefix for
    /// the worst case (3 bytes per UTF-16 unit) and pads it with continuation bytes, so a 50 character string has a
    /// two byte length. Readers accept either form.
    fn write_string_raw(&mut self, v: &str) {
        let worst = v.encode_utf16().count() as u64 * 3;
        let width = encoded_int_width(worst.min(u64::from(u32::MAX)) as u32);
        let mut n = v.len() as u32;
        let mut out = Vec::with_capacity(width + v.len());
        for _ in 1..width {
            out.push((n | 0x80) as u8);
            n >>= 7;
        }
        out.push(n as u8);
        out.extend_from_slice(v.as_bytes());
        self.write_bytes(&out);
    }

    /// `Write(string)` of a non-null string.
    fn write_string(&mut self, v: &str) {
        if self.prefix_strings() {
            self.write_bool(true);
        }
        self.write_string_raw(v);
    }

    /// `Write(string)` of a possibly-null string. Without prefixes a null is written as the empty string.
    fn write_opt_string(&mut self, v: Option<&str>) {
        match v {
            Some(s) => self.write_string(s),
            None if self.prefix_strings() => self.write_bool(false),
            None => self.write_string_raw(""),
        }
    }

    /// `Write(Type)`: 0 for null, else flag 2 and the name hash.
    fn write_type_hash(&mut self, hash: Option<u64>) {
        match hash {
            None => self.write_u8(0),
            Some(h) => {
                self.write_u8(2);
                self.write_u64(h);
            }
        }
    }

    /// `Write(DateTime)`: UTC ticks (convert a local time before calling).
    fn write_date_time_ticks(&mut self, ticks: i64) {
        self.write_i64(ticks);
    }

    fn write_time_span_ticks(&mut self, ticks: i64) {
        self.write_i64(ticks);
    }

    /// `WriteDeltaTime`, obsolete: no current-version format may write delta time (it rewrites its bytes on every
    /// save). Kept for byte-level tests of the old layout.
    fn write_delta_time_ticks(&mut self, ticks: i64, utc_now_ticks: i64) {
        use super::reader::{DATE_TIME_MAX_TICKS, DATE_TIME_MIN_TICKS};
        if ticks == DATE_TIME_MIN_TICKS {
            self.write_i64(i64::MIN);
        } else if ticks == DATE_TIME_MAX_TICKS {
            self.write_i64(i64::MAX);
        } else {
            self.write_i64(ticks.wrapping_sub(utc_now_ticks));
        }
    }

    /// `WriteAnchoredTime`: the absolute value; the reader re-bases it by the elapsed time since the save started.
    fn write_anchored_time_ticks(&mut self, ticks: i64) {
        self.write_i64(ticks);
    }

    fn write_decimal_bits(&mut self, bits: [i32; 4]) {
        for b in bits {
            self.write_i32(b);
        }
    }

    fn write_ip_address(&mut self, v: IpAddr) {
        match v {
            IpAddr::V4(a) => {
                self.write_u8(4);
                self.write_bytes(&a.octets());
            }
            IpAddr::V6(a) => {
                self.write_u8(16);
                self.write_bytes(&a.octets());
            }
        }
    }

    fn write_point3d(&mut self, v: Point3D) {
        self.write_i32(v.x);
        self.write_i32(v.y);
        self.write_i32(v.z);
    }

    fn write_point2d(&mut self, v: Point2D) {
        self.write_i32(v.x);
        self.write_i32(v.y);
    }

    fn write_rect2d(&mut self, v: Rectangle2D) {
        self.write_point2d(v.start);
        self.write_point2d(v.end);
    }

    fn write_rect3d(&mut self, v: Rectangle3D) {
        self.write_point3d(v.start);
        self.write_point3d(v.end);
    }

    /// `Write(Map)`: the map index, 0xFF for none.
    fn write_map_index(&mut self, index: Option<u8>) {
        self.write_u8(index.unwrap_or(0xFF));
    }

    fn write_race_index(&mut self, index: Option<u8>) {
        self.write_map_index(index);
    }

    /// `WriteEnum<T>` for an enum of `size` bytes (same layouts as `read_enum`).
    fn write_enum(&mut self, size: usize, value: i64) {
        match size {
            1 => self.write_u8(value as u8),
            2 => self.write_u16(value as u16),
            4 => self.write_encoded_int(value as i32),
            _ => self.write_u64(value as u64),
        }
    }

    fn write_guid(&mut self, v: Guid) {
        self.write_bytes(&v.to_net_bytes());
    }

    fn write_bit_array(&mut self, bits: &[bool]) {
        self.write_encoded_int(bits.len() as i32);
        let mut bytes = vec![0u8; bits.len().div_ceil(8)];
        for (i, _) in bits.iter().enumerate().filter(|(_, b)| **b) {
            bytes[i / 8] |= 1 << (i % 8);
        }
        self.write_bytes(&bytes);
    }

    /// `Write(TextDefinition)`: 3 for null, 1 number, 2 text, 0 empty.
    fn write_text_definition(&mut self, def: Option<&TextDefinition>) {
        match def {
            None => self.write_encoded_int(3),
            Some(d) if d.number > 0 => {
                self.write_encoded_int(1);
                self.write_encoded_int(d.number);
            }
            Some(d) => match &d.string {
                Some(s) => {
                    self.write_encoded_int(2);
                    self.write_string(s);
                }
                None => self.write_encoded_int(0),
            },
        }
    }
}

/// `IGenericSerializable`.
pub trait GenericSerializable {
    fn serialize(&self, writer: &mut dyn GenericWriter);
}
