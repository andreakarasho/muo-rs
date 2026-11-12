//! `TileData`: `tiledata.mul`, the land and item flag tables.
//!
//! The tables (`TileData.LandTable`, `ItemTable`) are not global: a `TileData` is a value the host owns and
//! passes to whatever reads heights and flags.

use crate::data_directories::DataDirectories;
use crate::error::{DataError, Result};
use bitflags::bitflags;

bitflags! {
    #[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash)]
    pub struct TileFlag: u64 {
        const BACKGROUND = 0x1;
        const WEAPON = 0x2;
        const TRANSPARENT = 0x4;
        const TRANSLUCENT = 0x8;
        const WALL = 0x10;
        const DAMAGING = 0x20;
        const IMPASSABLE = 0x40;
        const WET = 0x80;
        const UNKNOWN1 = 0x100;
        const SURFACE = 0x200;
        const BRIDGE = 0x400;
        const GENERIC = 0x800;
        const WINDOW = 0x1000;
        const NO_SHOOT = 0x2000;
        const ARTICLE_A = 0x4000;
        const ARTICLE_AN = 0x8000;
        const INTERNAL = 0x10000;
        const FOLIAGE = 0x20000;
        const PARTIAL_HUE = 0x40000;
        const UNKNOWN2 = 0x80000;
        const MAP = 0x100000;
        const CONTAINER = 0x200000;
        const WEARABLE = 0x400000;
        const LIGHT_SOURCE = 0x800000;
        const ANIMATION = 0x1000000;
        const NO_DIAGONAL = 0x2000000;
        const UNKNOWN3 = 0x4000000;
        const ARMOR = 0x8000000;
        const ROOF = 0x10000000;
        const DOOR = 0x20000000;
        const STAIR_BACK = 0x40000000;
        const STAIR_RIGHT = 0x80000000;
        const HS33 = 1 << 32;
        const HS34 = 1 << 33;
        const HS35 = 1 << 34;
        const HS36 = 1 << 35;
        const HS37 = 1 << 36;
        const HS38 = 1 << 37;
        const HS39 = 1 << 38;
        const HS40 = 1 << 39;
        const HS41 = 1 << 40;
        const HS42 = 1 << 41;
        const HS43 = 1 << 42;
        const HS44 = 1 << 43;
        const HS45 = 1 << 44;
        const HS46 = 1 << 45;
        const HS47 = 1 << 46;
        const HS48 = 1 << 47;
        const HS49 = 1 << 48;
        const HS50 = 1 << 49;
        const HS51 = 1 << 50;
        const HS52 = 1 << 51;
        const HS53 = 1 << 52;
        const HS54 = 1 << 53;
        const HS55 = 1 << 54;
        const HS56 = 1 << 55;
        const HS57 = 1 << 56;
        const HS58 = 1 << 57;
        const HS59 = 1 << 58;
        const HS60 = 1 << 59;
        const HS61 = 1 << 60;
        const HS62 = 1 << 61;
        const HS63 = 1 << 62;
        const HS64 = 1 << 63;
    }
}

pub const LAND_LENGTH: usize = 0x4000;
pub const ITEM_TABLE_LENGTH: usize = 0x10000;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct LandData {
    pub name: String,
    pub flags: TileFlag,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ItemData {
    pub name: String,
    pub flags: TileFlag,
    pub weight: u8,
    pub quality: u8,
    pub animation: u16,
    pub quantity: u8,
    pub value: u8,
    pub height: u8,
}

macro_rules! flag_props {
    ($($get:ident, $set:ident = $flag:ident;)*) => {
        $(
            pub fn $get(&self) -> bool {
                self.flags.contains(TileFlag::$flag)
            }

            pub fn $set(&mut self, value: bool) {
                self.flags.set(TileFlag::$flag, value);
            }
        )*
    };
}

impl ItemData {
    /// Half height for a bridge (you stand on the middle of it).
    pub fn calc_height(&self) -> i32 {
        if self.bridge() {
            i32::from(self.height) / 2
        } else {
            i32::from(self.height)
        }
    }

    /// `this[flag]`: any bit of `flag` set (`(Flags & flag) != 0`, so `ImpassableSurface` reads true when
    /// either bit is set).
    pub fn has(&self, flag: TileFlag) -> bool {
        self.flags.intersects(flag)
    }

    pub fn impassable_surface(&self) -> bool {
        self.has(TileFlag::IMPASSABLE | TileFlag::SURFACE)
    }

    pub fn set_impassable_surface(&mut self, value: bool) {
        self.flags
            .set(TileFlag::IMPASSABLE | TileFlag::SURFACE, value);
    }

    flag_props! {
        door, set_door = DOOR;
        background, set_background = BACKGROUND;
        bridge, set_bridge = BRIDGE;
        wall, set_wall = WALL;
        window, set_window = WINDOW;
        impassable, set_impassable = IMPASSABLE;
        surface, set_surface = SURFACE;
        roof, set_roof = ROOF;
        light_source, set_light_source = LIGHT_SOURCE;
        wet, set_wet = WET;
    }
}

/// The two tables. Always `0x4000` land and `0x10000` item entries; entries past what the file holds are default.
#[derive(Clone, Debug)]
pub struct TileData {
    pub land: Vec<LandData>,
    pub items: Vec<ItemData>,
}

impl Default for TileData {
    fn default() -> Self {
        TileData::empty()
    }
}

impl TileData {
    pub const MAX_LAND_VALUE: usize = LAND_LENGTH - 1;
    pub const MAX_ITEM_VALUE: usize = ITEM_TABLE_LENGTH - 1;

    /// All-zero tables (what a host without client files, such as the tests, starts from).
    pub fn empty() -> TileData {
        TileData {
            land: vec![LandData::default(); LAND_LENGTH],
            items: vec![ItemData::default(); ITEM_TABLE_LENGTH],
        }
    }

    /// `TileData.Load`: finds `tiledata.mul` in the data directories.
    pub fn load(dirs: &DataDirectories) -> Result<TileData> {
        let path = dirs.require("tiledata.mul")?;
        let bytes = std::fs::read(&path).map_err(|e| DataError::io(&path, e))?;
        TileData::from_bytes(&bytes).map_err(|e| match e {
            DataError::InvalidData { reason, .. } => {
                DataError::invalid(path.to_string_lossy(), reason)
            }
            other => other,
        })
    }

    /// Parses a `tiledata.mul` image. The file size picks the layout: 7.0.9.0 and later (`>= 3188736`)
    /// has 64-bit flags and `0x10000` items, 7.0.0.0 (`>= 1644544`) `0x8000` items, older `0x4000`.
    pub fn from_bytes(bytes: &[u8]) -> Result<TileData> {
        let (is64, item_length) = if bytes.len() >= 3_188_736 {
            (true, 0x10000)
        } else if bytes.len() >= 1_644_544 {
            (false, 0x8000)
        } else {
            (false, 0x4000)
        };

        let mut r = Cursor { bytes, pos: 0 };
        let mut data = TileData::empty();

        for i in 0..LAND_LENGTH {
            let header = if is64 {
                i == 1 || (i > 0 && (i & 0x1F) == 0)
            } else {
                (i & 0x1F) == 0
            };
            if header {
                r.skip(4)?;
            }
            let flags = r.flags(is64)?;
            r.skip(2)?; // textureID
            data.land[i] = LandData {
                name: r.name()?,
                flags,
            };
        }

        for i in 0..item_length {
            if (i & 0x1F) == 0 {
                r.skip(4)?;
            }
            let flags = r.flags(is64)?;
            let weight = r.u8()?;
            let quality = r.u8()?;
            let animation = u16::from_le_bytes([r.u8()?, r.u8()?]);
            r.skip(1)?;
            let quantity = r.u8()?;
            r.skip(4)?;
            r.skip(1)?;
            let value = r.u8()?;
            let height = r.u8()?;
            data.items[i] = ItemData {
                name: r.name()?,
                flags,
                weight,
                quality,
                animation,
                quantity,
                value,
                height,
            };
        }
        Ok(data)
    }

    /// `ItemTable[id & MaxItemValue]`.
    pub fn item(&self, id: usize) -> &ItemData {
        &self.items[id & Self::MAX_ITEM_VALUE]
    }

    /// `LandTable[id & MaxLandValue]`.
    pub fn land(&self, id: usize) -> &LandData {
        &self.land[id & Self::MAX_LAND_VALUE]
    }
}

struct Cursor<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl Cursor<'_> {
    fn take(&mut self, n: usize) -> Result<&[u8]> {
        let end = self.pos + n;
        let s = self
            .bytes
            .get(self.pos..end)
            .ok_or_else(|| DataError::invalid("tiledata.mul", "Unexpected end of file."))?;
        self.pos = end;
        Ok(s)
    }

    fn skip(&mut self, n: usize) -> Result<()> {
        self.take(n).map(|_| ())
    }

    fn u8(&mut self) -> Result<u8> {
        Ok(self.take(1)?[0])
    }

    fn flags(&mut self, is64: bool) -> Result<TileFlag> {
        Ok(TileFlag::from_bits_retain(if is64 {
            u64::from_le_bytes(self.take(8)?.try_into().unwrap_or_default())
        } else {
            u64::from(u32::from_le_bytes(
                self.take(4)?.try_into().unwrap_or_default(),
            ))
        }))
    }

    /// 20 bytes, ASCII up to the first NUL (`Encoding.ASCII` turns bytes above 0x7F into `?`).
    fn name(&mut self) -> Result<String> {
        let raw = self.take(20)?;
        let end = raw.iter().position(|&b| b == 0).unwrap_or(raw.len());
        Ok(raw[..end]
            .iter()
            .map(|&b| if b < 0x80 { b as char } else { '?' })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Synthetic tile data: a pre-HS tiledata.mul of 0x4000 land records of 26 bytes
    /// and 0x4000 item records of 37 bytes, a 4-byte header before every 32.
    pub(crate) fn build_synthetic(
        land: &[(usize, &str, TileFlag)],
        items: &[(usize, &str, TileFlag, u8)],
    ) -> Vec<u8> {
        fn name(out: &mut Vec<u8>, n: &str) {
            let mut b = [0u8; 20];
            b[..n.len()].copy_from_slice(n.as_bytes());
            out.extend_from_slice(&b);
        }
        let mut out = Vec::new();
        for i in 0..LAND_LENGTH {
            if i & 0x1F == 0 {
                out.extend_from_slice(&0i32.to_le_bytes());
            }
            let (n, f) = land
                .iter()
                .find(|l| l.0 == i)
                .map_or(("", TileFlag::empty()), |l| (l.1, l.2));
            out.extend_from_slice(&(f.bits() as u32).to_le_bytes());
            out.extend_from_slice(&0i16.to_le_bytes());
            name(&mut out, n);
        }
        for i in 0..0x4000 {
            if i & 0x1F == 0 {
                out.extend_from_slice(&0i32.to_le_bytes());
            }
            let (n, f, h) = items
                .iter()
                .find(|l| l.0 == i)
                .map_or(("", TileFlag::empty(), 0), |l| (l.1, l.2, l.3));
            out.extend_from_slice(&(f.bits() as u32).to_le_bytes());
            out.extend_from_slice(&[0u8; 11]); // weight, quality, animation, 1 skipped, quantity, 4 skipped, 1 skipped
            out.extend_from_slice(&[0, h]); // value, height
            name(&mut out, n);
        }
        out
    }

    #[test]
    fn generated_file_is_pre_hs_size() {
        assert_eq!(build_synthetic(&[], &[]).len(), 1_036_288);
    }

    #[test]
    fn generated_file_round_trips_through_load() {
        let wall = TileFlag::WALL | TileFlag::IMPASSABLE | TileFlag::NO_SHOOT;
        let bytes = build_synthetic(
            &[
                (0x3000, "test grass", TileFlag::empty()),
                (0xA8, "test water", TileFlag::WET | TileFlag::IMPASSABLE),
            ],
            &[
                (0x3000, "test wall", wall, 20),
                (
                    0x3004,
                    "test bridge",
                    TileFlag::BRIDGE | TileFlag::SURFACE,
                    2,
                ),
            ],
        );
        let td = TileData::from_bytes(&bytes).unwrap();
        assert_eq!(td.items[0x3000].flags, wall);
        assert_eq!(td.items[0x3000].height, 20);
        assert_eq!(td.items[0x3000].name, "test wall");
        assert_eq!(td.items[0x3004].calc_height(), 1);
        assert!(td.items[0x3004].bridge());
        assert_eq!(td.land[0xA8].flags, TileFlag::WET | TileFlag::IMPASSABLE);
        assert_eq!(td.land[0xA8].name, "test water");
        assert_eq!(td.items[0x2A0].flags, TileFlag::empty());
        assert_eq!(td.land[3].flags, TileFlag::empty());
        assert_eq!(td.items.len(), 0x10000);
    }

    #[test]
    fn truncated_file_is_an_error() {
        let bytes = build_synthetic(&[], &[]);
        assert!(TileData::from_bytes(&bytes[..bytes.len() - 1]).is_err());
        assert!(TileData::from_bytes(&[]).is_err());
    }

    #[test]
    fn flag_accessors() {
        let mut d = ItemData::default();
        d.set_door(true);
        assert!(d.door() && d.flags == TileFlag::DOOR);
        d.set_impassable_surface(true);
        assert!(d.impassable() && d.surface());
        d.set_door(false);
        assert!(!d.door());
    }
}
