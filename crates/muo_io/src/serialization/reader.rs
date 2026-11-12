//! `GenericReader`: the save file reader.
//!
//! Everything is little endian, as in the save files. `DateTime`/`TimeSpan` travel as 100 ns tick counts
//! (`i64`); `muo_primitives` wrappers convert at the edge.
//!
//! What the server adds (it owns the entity tables, so `muo_io` cannot):
//!
//! ```ignore
//! pub trait ReadEntityExt: GenericReader {
//!     fn read_item(&mut self, w: &World) -> Result<Option<ItemId>> {
//!         let s = self.read_serial()?;           // IGenericReader.ReadEntity<T>: serial, then look it up
//!         Ok(w.items.contains_key(&ItemId(s)).then_some(ItemId(s)))
//!     }
//!     fn read_map(&mut self, w: &World) -> Result<Option<MapId>> { /* MapId from `read_map_index()` */ }
//!     fn read_race(&mut self, w: &World) -> Result<Option<RaceId>> { /* from `read_race_index()` */ }
//!     fn read_type_name(&mut self, w: &World) -> Result<Option<&'static ItemType>> { /* `read_type()` + registry */ }
//! }
//! impl<R: GenericReader + ?Sized> ReadEntityExt for R {}
//! ```

use crate::error::{IoError, Result, SeekOrigin};
use crate::text::TextDefinition;
use muo_primitives::{Point2D, Point3D, Rectangle2D, Rectangle3D, Serial};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

/// `DateTime.MinValue.Ticks` / `DateTime.MaxValue.Ticks`.
pub const DATE_TIME_MIN_TICKS: i64 = 0;
pub const DATE_TIME_MAX_TICKS: i64 = 3_155_378_975_999_999_999;

/// How a serialized `Type` is named: by full name (flag 1, legacy) or by the XXH3-64 hash of its full name
/// (flag 2, seed 9609125370673258709 over the UTF-16LE bytes; see `HashUtility`). The server resolves both.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TypeRef {
    Name(String),
    Hash(u64),
}

/// A `Guid`, kept in canonical (RFC 4122, `ToString`) byte order.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash)]
pub struct Guid(pub [u8; 16]);

impl Guid {
    /// Bytes as `Guid.ToByteArray` / `TryWriteBytes` lay them out (first three fields little endian).
    pub fn to_net_bytes(self) -> [u8; 16] {
        let b = self.0;
        [
            b[3], b[2], b[1], b[0], b[5], b[4], b[7], b[6], b[8], b[9], b[10], b[11], b[12], b[13],
            b[14], b[15],
        ]
    }

    pub fn from_net_bytes(b: [u8; 16]) -> Self {
        Self([
            b[3], b[2], b[1], b[0], b[5], b[4], b[7], b[6], b[8], b[9], b[10], b[11], b[12], b[13],
            b[14], b[15],
        ])
    }
}

impl std::fmt::Display for Guid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let b = &self.0;
        write!(
            f,
            "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
            b[0],
            b[1],
            b[2],
            b[3],
            b[4],
            b[5],
            b[6],
            b[7],
            b[8],
            b[9],
            b[10],
            b[11],
            b[12],
            b[13],
            b[14],
            b[15]
        )
    }
}

pub trait GenericReader {
    /// Fills `buf` or fails with `EndOfStream` for a short source.
    fn read_exact(&mut self, buf: &mut [u8]) -> Result<()>;

    fn position(&self) -> u64;

    fn seek(&mut self, offset: i64, origin: SeekOrigin) -> Result<u64>;

    /// `ReadString(intern)`: a presence flag then the raw string, for readers made with prefixed strings;
    /// otherwise the raw string. `None` is a null string.
    fn read_string(&mut self) -> Result<Option<String>>;

    /// `AnchoredTimeShift` in ticks: elapsed time between the loaded save starting and this load.
    fn anchored_time_shift_ticks(&self) -> i64 {
        0
    }

    fn read_array<const N: usize>(&mut self) -> Result<[u8; N]>
    where
        Self: Sized,
    {
        let mut a = [0u8; N];
        self.read_exact(&mut a)?;
        Ok(a)
    }

    /// `Read(Span<byte>)`.
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.read_exact(buf)?;
        Ok(buf.len())
    }

    /// Reads `n` bytes without trusting `n` for the allocation.
    fn read_vec(&mut self, n: usize) -> Result<Vec<u8>> {
        const CHUNK: usize = 1 << 16;
        let mut v = Vec::new();
        let mut left = n;
        while left > 0 {
            let c = left.min(CHUNK);
            let start = v.len();
            v.resize(start + c, 0);
            self.read_exact(&mut v[start..])?;
            left -= c;
        }
        Ok(v)
    }

    fn read_u8(&mut self) -> Result<u8> {
        let mut a = [0u8; 1];
        self.read_exact(&mut a)?;
        Ok(a[0])
    }

    fn read_i8(&mut self) -> Result<i8> {
        Ok(self.read_u8()? as i8)
    }

    fn read_bool(&mut self) -> Result<bool> {
        Ok(self.read_u8()? != 0)
    }

    fn read_i16(&mut self) -> Result<i16> {
        let mut a = [0u8; 2];
        self.read_exact(&mut a)?;
        Ok(i16::from_le_bytes(a))
    }

    fn read_u16(&mut self) -> Result<u16> {
        let mut a = [0u8; 2];
        self.read_exact(&mut a)?;
        Ok(u16::from_le_bytes(a))
    }

    fn read_i32(&mut self) -> Result<i32> {
        let mut a = [0u8; 4];
        self.read_exact(&mut a)?;
        Ok(i32::from_le_bytes(a))
    }

    fn read_u32(&mut self) -> Result<u32> {
        let mut a = [0u8; 4];
        self.read_exact(&mut a)?;
        Ok(u32::from_le_bytes(a))
    }

    fn read_i64(&mut self) -> Result<i64> {
        let mut a = [0u8; 8];
        self.read_exact(&mut a)?;
        Ok(i64::from_le_bytes(a))
    }

    fn read_u64(&mut self) -> Result<u64> {
        let mut a = [0u8; 8];
        self.read_exact(&mut a)?;
        Ok(u64::from_le_bytes(a))
    }

    fn read_f32(&mut self) -> Result<f32> {
        Ok(f32::from_bits(self.read_u32()?))
    }

    fn read_f64(&mut self) -> Result<f64> {
        Ok(f64::from_bits(self.read_u64()?))
    }

    fn read_serial(&mut self) -> Result<Serial> {
        Ok(Serial::new(self.read_u32()?))
    }

    /// 7-bit encoded int, least significant group first. It accepts
    /// non-minimal encodings, which `BufferWriter.WriteRaw` produces for string lengths.
    fn read_encoded_int(&mut self) -> Result<i32> {
        let mut v: i32 = 0;
        let mut shift: u32 = 0;
        loop {
            let b = self.read_u8()?;
            // the shift count is masked to 5 bits
            v |= i32::from(b & 0x7F).wrapping_shl(shift);
            shift += 7;
            if b < 0x80 {
                return Ok(v);
            }
        }
    }

    /// `ReadStringRaw`: encoded byte length (<= 0 is the empty string) then UTF-8.
    fn read_string_raw(&mut self) -> Result<String> {
        let length = self.read_encoded_int()?;
        if length <= 0 {
            return Ok(String::new());
        }
        let bytes = self.read_vec(length as usize)?;
        Ok(String::from_utf8_lossy(&bytes).into_owned())
    }

    /// `ReadType`: flag 1 is a full name, 2 a name hash, anything else null.
    fn read_type(&mut self) -> Result<Option<TypeRef>> {
        match self.read_u8()? {
            0 => Ok(None),
            1 => Ok(Some(TypeRef::Name(self.read_string_raw()?))),
            2 => Ok(Some(TypeRef::Hash(self.read_u64()?))),
            _ => Ok(None), // an unknown flag reads as null
        }
    }

    /// `ReadDateTime`: UTC ticks, validated against `DateTime`'s range.
    fn read_date_time_ticks(&mut self) -> Result<i64> {
        let t = self.read_i64()?;
        if !(DATE_TIME_MIN_TICKS..=DATE_TIME_MAX_TICKS).contains(&t) {
            return Err(IoError::InvalidData(format!(
                "DateTime ticks {t} out of range"
            )));
        }
        Ok(t)
    }

    fn read_time_span_ticks(&mut self) -> Result<i64> {
        self.read_i64()
    }

    /// `ReadDeltaTime`, for old-version payloads only: a delta from now (`utc_now_ticks`), with the two
    /// sentinels for `DateTime.MinValue`/`MaxValue`.
    fn read_delta_time_ticks(&mut self, utc_now_ticks: i64) -> Result<i64> {
        match self.read_i64()? {
            i64::MIN => Ok(DATE_TIME_MIN_TICKS),
            i64::MAX => Ok(DATE_TIME_MAX_TICKS),
            delta => {
                let t = delta.wrapping_add(utc_now_ticks);
                if !(DATE_TIME_MIN_TICKS..=DATE_TIME_MAX_TICKS).contains(&t) {
                    return Err(IoError::InvalidData(format!(
                        "delta time ticks {t} out of range"
                    )));
                }
                Ok(t)
            }
        }
    }

    /// `ReadAnchoredTime`: the stored value moved forward by the load's `anchored_time_shift_ticks`, the
    /// sentinels untouched, saturating at both ends.
    fn read_anchored_time_ticks(&mut self) -> Result<i64> {
        let value = self.read_date_time_ticks()?;
        if value == DATE_TIME_MIN_TICKS || value == DATE_TIME_MAX_TICKS {
            return Ok(value);
        }
        let shift = self.anchored_time_shift_ticks();
        if shift == 0 {
            return Ok(value);
        }
        let ticks = value.saturating_add(shift);
        Ok(if ticks >= DATE_TIME_MAX_TICKS {
            DATE_TIME_MAX_TICKS
        } else if ticks <= 0 {
            DATE_TIME_MIN_TICKS
        } else {
            ticks
        })
    }

    /// `ReadDecimal`: the four `decimal.GetBits` ints (lo, mid, hi, flags).
    fn read_decimal_bits(&mut self) -> Result<[i32; 4]> {
        Ok([
            self.read_i32()?,
            self.read_i32()?,
            self.read_i32()?,
            self.read_i32()?,
        ])
    }

    /// `ReadIPAddress`: length byte (4 or 16) then the address bytes.
    fn read_ip_address(&mut self) -> Result<IpAddr> {
        match self.read_u8()? {
            4 => {
                let mut a = [0u8; 4];
                self.read_exact(&mut a)?;
                Ok(IpAddr::V4(Ipv4Addr::from(a)))
            }
            16 => {
                let mut a = [0u8; 16];
                self.read_exact(&mut a)?;
                Ok(IpAddr::V6(Ipv6Addr::from(a)))
            }
            n => Err(IoError::InvalidData(format!(
                "invalid IP address length {n}"
            ))),
        }
    }

    fn read_point3d(&mut self) -> Result<Point3D> {
        let (x, y, z) = (self.read_i32()?, self.read_i32()?, self.read_i32()?);
        Ok(Point3D::new(x, y, z))
    }

    fn read_point2d(&mut self) -> Result<Point2D> {
        let (x, y) = (self.read_i32()?, self.read_i32()?);
        Ok(Point2D::new(x, y))
    }

    fn read_rect2d(&mut self) -> Result<Rectangle2D> {
        let (start, end) = (self.read_point2d()?, self.read_point2d()?);
        Ok(Rectangle2D::from_points(start, end))
    }

    fn read_rect3d(&mut self) -> Result<Rectangle3D> {
        let (start, end) = (self.read_point3d()?, self.read_point3d()?);
        Ok(Rectangle3D::from_points(start, end))
    }

    /// `ReadMap`'s byte; 0xFF is no map. The server turns it into its map id.
    fn read_map_index(&mut self) -> Result<Option<u8>> {
        Ok(Some(self.read_u8()?).filter(|&i| i != 0xFF))
    }

    /// `ReadRace`'s byte; 0xFF is no race.
    fn read_race_index(&mut self) -> Result<Option<u8>> {
        self.read_map_index()
    }

    /// `ReadEnum<T>` for an enum of `size` bytes: 1 byte, 2 bytes, an encoded int (4) or a long (8).
    fn read_enum(&mut self, size: usize) -> Result<i64> {
        match size {
            1 => Ok(i64::from(self.read_u8()?)),
            2 => Ok(i64::from(self.read_i16()?)),
            4 => Ok(i64::from(self.read_encoded_int()?)),
            8 => self.read_i64(),
            _ => Err(IoError::InvalidData(format!(
                "unsupported enum size {size}"
            ))),
        }
    }

    fn read_guid(&mut self) -> Result<Guid> {
        let mut a = [0u8; 16];
        self.read_exact(&mut a)?;
        Ok(Guid::from_net_bytes(a))
    }

    /// `ReadBitArray`: encoded bit count then `ceil(n / 8)` bytes, least significant bit first.
    fn read_bit_array(&mut self) -> Result<Vec<bool>> {
        let bit_length = self.read_encoded_int()?;
        if bit_length < 0 {
            return Err(IoError::InvalidData("negative BitArray length".into()));
        }
        let bytes = self.read_vec((bit_length as usize).div_ceil(8))?;
        Ok((0..bit_length as usize)
            .map(|i| bytes[i / 8] & (1 << (i % 8)) != 0)
            .collect())
    }

    /// `ReadTextDefinition`: 0 empty, 1 number, 2 text, anything else null.
    fn read_text_definition(&mut self) -> Result<Option<TextDefinition>> {
        Ok(match self.read_encoded_int()? {
            0 => Some(TextDefinition::EMPTY),
            1 => Some(TextDefinition::of_number(self.read_encoded_int()?)),
            2 => Some(match self.read_string()? {
                Some(s) => TextDefinition::of_text(s),
                None => TextDefinition::EMPTY,
            }),
            _ => None,
        })
    }
}
