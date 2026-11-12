//! Multi (house, boat) footprints from `multi.idx`/`multi.mul` or
//! `MultiCollection.uop`.
//!
//! A `MultiData` value owns the footprints; there is no global table. Edits that look at item
//! heights take the `TileData`.

use super::uop::{UopEntry, hash_little2, read_uop_indexes};
use crate::client::ClientVersion;
use crate::configuration::ServerConfiguration;
use crate::data_directories::DataDirectories;
use crate::error::{DataError, Result};
use crate::le;
use crate::tile_data::{TileData, TileFlag};
use crate::tile_matrix::StaticTile;
use muo_io::compression::STANDARD;
use muo_io::{GenericReader, GenericWriter, SeekOrigin, SpanReader};
use muo_primitives::Point2D;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

/// Footprints larger than this are corrupt data, not houses.
const MAX_AREA: i64 = 1 << 22;

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub struct MultiTileEntry {
    pub item_id: u16,
    pub offset_x: i16,
    pub offset_y: i16,
    pub offset_z: i16,
    pub flags: TileFlag,
}

impl MultiTileEntry {
    pub fn new(item_id: u16, x: i16, y: i16, z: i16, flags: TileFlag) -> Self {
        MultiTileEntry {
            item_id,
            offset_x: x,
            offset_y: y,
            offset_z: z,
            flags,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct MultiComponentList {
    min: Point2D,
    max: Point2D,
    center: Point2D,
    width: i32,
    height: i32,
    /// `[x][y]` of the footprint, each cell bottom to top.
    pub tiles: Vec<Vec<Vec<StaticTile>>>,
    pub list: Vec<MultiTileEntry>,
}

static EMPTY: MultiComponentList = MultiComponentList {
    min: Point2D::ZERO,
    max: Point2D::ZERO,
    center: Point2D::ZERO,
    width: 0,
    height: 0,
    tiles: Vec::new(),
    list: Vec::new(),
};

impl MultiComponentList {
    pub fn empty() -> &'static MultiComponentList {
        &EMPTY
    }

    /// The shared constructor body: bounds over the entries that count (the first, and any with flags), then the
    /// tile grid.
    fn from_list(list: Vec<MultiTileEntry>) -> Result<Self> {
        let (mut min, mut max) = (Point2D::ZERO, Point2D::ZERO);
        for (i, e) in list.iter().enumerate() {
            if i == 0 || !e.flags.is_empty() {
                min.x = min.x.min(i32::from(e.offset_x));
                min.y = min.y.min(i32::from(e.offset_y));
                max.x = max.x.max(i32::from(e.offset_x));
                max.y = max.y.max(i32::from(e.offset_y));
            }
        }
        let center = Point2D::new(-min.x, -min.y);
        let (width, height) = (max.x - min.x + 1, max.y - min.y + 1);
        Self::with_bounds(list, min, max, center, width, height)
    }

    fn with_bounds(
        list: Vec<MultiTileEntry>,
        min: Point2D,
        max: Point2D,
        center: Point2D,
        width: i32,
        height: i32,
    ) -> Result<Self> {
        if width < 0 || height < 0 || i64::from(width) * i64::from(height) > MAX_AREA {
            return Err(DataError::invalid(
                "multi",
                "footprint size is out of range",
            ));
        }
        let mut tiles = vec![vec![Vec::new(); height as usize]; width as usize];
        for (i, e) in list.iter().enumerate() {
            if i == 0 || !e.flags.is_empty() {
                let cell = usize::try_from(i32::from(e.offset_x) + center.x)
                    .ok()
                    .and_then(|x| tiles.get_mut(x))
                    .and_then(|col| {
                        usize::try_from(i32::from(e.offset_y) + center.y)
                            .ok()
                            .and_then(|y| col.get_mut(y))
                    })
                    .ok_or_else(|| {
                        DataError::invalid("multi", "tile lies outside the footprint")
                    })?;
                cell.push(StaticTile::new(e.item_id, e.offset_z as i8));
            }
        }
        Ok(MultiComponentList {
            min,
            max,
            center,
            width,
            height,
            tiles,
            list,
        })
    }

    /// `MultiComponentList(List<MultiTileEntry>)`.
    pub fn from_entries(list: &[MultiTileEntry]) -> Self {
        // Cannot fail for i16 offsets: the bounds are computed from the very entries placed.
        Self::from_list(list.to_vec()).unwrap_or_else(|_| EMPTY.clone())
    }

    /// `MultiComponentList(BinaryReader, length, postHSFormat)`: `multi.mul` records of 16 (post-HS) or 12 bytes.
    pub fn from_mul<R: Read>(reader: &mut R, length: i32, post_hs_format: bool) -> Result<Self> {
        let count = length.max(0) / if post_hs_format { 16 } else { 12 };
        let mut list = Vec::with_capacity(count as usize);
        for _ in 0..count {
            let item_id = le::u16(reader)?;
            let (x, y, z) = (le::i16(reader)?, le::i16(reader)?, le::i16(reader)?);
            let flags = if post_hs_format {
                le::u64(reader)?
            } else {
                u64::from(le::u32(reader)?)
            };
            list.push(MultiTileEntry::new(
                item_id,
                x,
                y,
                z,
                TileFlag::from_bits_retain(flags),
            ));
        }
        Self::from_list(list)
    }

    /// `MultiComponentList(IGenericReader)`: the saved form (version 0 masked the id with `0x3FFF`).
    pub fn deserialize(reader: &mut dyn GenericReader) -> Result<Self> {
        let version = reader.read_i32()?;
        let min = reader.read_point2d()?;
        let max = reader.read_point2d()?;
        let center = reader.read_point2d()?;
        let width = reader.read_i32()?;
        let height = reader.read_i32()?;
        let length = reader.read_i32()?;
        if !(0..=(1 << 20)).contains(&length) {
            return Err(DataError::invalid("multi", "entry count is out of range"));
        }
        let mut list = Vec::with_capacity(length as usize);
        for _ in 0..length {
            let item_id = if version == 0 {
                let mut id = i32::from(reader.read_i16()?);
                if id >= 0x4000 {
                    id -= 0x4000;
                }
                id as u16
            } else {
                reader.read_u16()?
            };
            let (x, y, z) = (reader.read_i16()?, reader.read_i16()?, reader.read_i16()?);
            let flags = TileFlag::from_bits_retain(u64::from(reader.read_i32()? as u32));
            list.push(MultiTileEntry::new(item_id, x, y, z, flags));
        }
        Self::with_bounds(list, min, max, center, width, height)
    }

    /// `Serialize`: version 1.
    pub fn serialize(&self, writer: &mut dyn GenericWriter) {
        writer.write_i32(1);
        writer.write_point2d(self.min);
        writer.write_point2d(self.max);
        writer.write_point2d(self.center);
        writer.write_i32(self.width);
        writer.write_i32(self.height);
        writer.write_i32(self.list.len() as i32);
        for e in &self.list {
            writer.write_u16(e.item_id);
            writer.write_i16(e.offset_x);
            writer.write_i16(e.offset_y);
            writer.write_i16(e.offset_z);
            writer.write_i32(e.flags.bits() as i32);
        }
    }

    pub fn min(&self) -> Point2D {
        self.min
    }

    pub fn max(&self) -> Point2D {
        self.max
    }

    pub fn center(&self) -> Point2D {
        self.center
    }

    pub fn width(&self) -> i32 {
        self.width
    }

    pub fn height(&self) -> i32 {
        self.height
    }

    fn cell(&self, x: i32, y: i32) -> Option<(usize, usize)> {
        let (vx, vy) = (x + self.center.x, y + self.center.y);
        (vx >= 0 && vx < self.width && vy >= 0 && vy < self.height)
            .then_some((vx as usize, vy as usize))
    }

    /// `Add`: places an item, first removing a tile of the same height class and roof-ness at the same z.
    pub fn add(&mut self, item_id: i32, x: i32, y: i32, z: i32, tile_data: &TileData) {
        let Some((vx, vy)) = self.cell(x, y) else {
            return;
        };
        let data = tile_data.item(item_id as usize);
        for i in (0..self.tiles[vx][vy].len()).rev() {
            // the cell shrinks as tiles are removed, so index from the end
            let Some(old) = self.tiles[vx][vy].get(i).copied() else {
                continue;
            };
            if i32::from(old.z) == z && (old.height(tile_data) > 0) == (data.height > 0) {
                let new_is_roof = data.roof();
                let old_is_roof = tile_data.item(usize::from(old.id)).roof();
                if new_is_roof == old_is_roof {
                    self.remove(i32::from(old.id), x, y, z);
                }
            }
        }
        self.tiles[vx][vy].push(StaticTile::new(item_id as u16, z as i8));
        self.list.push(MultiTileEntry::new(
            item_id as u16,
            x as i16,
            y as i16,
            z as i16,
            TileFlag::BACKGROUND,
        ));
        self.min.x = self.min.x.min(x);
        self.min.y = self.min.y.min(y);
        self.max.x = self.max.x.max(x);
        self.max.y = self.max.y.max(y);
    }

    /// `RemoveXYZH`: removes the first tile at `z` that is at least `min_height` tall, and its list entry.
    pub fn remove_xyzh(&mut self, x: i32, y: i32, z: i32, min_height: i32, tile_data: &TileData) {
        let Some((vx, vy)) = self.cell(x, y) else {
            return;
        };
        if let Some(i) = self.tiles[vx][vy]
            .iter()
            .position(|t| i32::from(t.z) == z && t.height(tile_data) >= min_height)
        {
            self.tiles[vx][vy].remove(i);
        }
        if let Some(i) = self.list.iter().position(|t| {
            t.offset_x == x as i16
                && t.offset_y == y as i16
                && t.offset_z == z as i16
                && i32::from(tile_data.item(usize::from(t.item_id)).height) >= min_height
        }) {
            self.list.remove(i);
        }
    }

    /// `Remove`: removes the first tile and list entry of `item_id` at `(x, y, z)`.
    pub fn remove(&mut self, item_id: i32, x: i32, y: i32, z: i32) {
        let Some((vx, vy)) = self.cell(x, y) else {
            return;
        };
        if let Some(i) = self.tiles[vx][vy]
            .iter()
            .position(|t| i32::from(t.id) == item_id && i32::from(t.z) == z)
        {
            self.tiles[vx][vy].remove(i);
        }
        if let Some(i) = self.list.iter().position(|t| {
            i32::from(t.item_id) == item_id
                && t.offset_x == x as i16
                && t.offset_y == y as i16
                && t.offset_z == z as i16
        }) {
            self.list.remove(i);
        }
    }

    /// `Resize`: rebuilds the grid at a new size, dropping what falls outside, and the list from the tiles.
    pub fn resize(&mut self, new_width: i32, new_height: i32) {
        let (new_width, new_height) = (new_width.max(0), new_height.max(0));
        if i64::from(new_width) * i64::from(new_height) > MAX_AREA {
            return;
        }
        let mut new_tiles = vec![vec![Vec::new(); new_height as usize]; new_width as usize];
        for (x, col) in new_tiles.iter_mut().enumerate() {
            for (y, cell) in col.iter_mut().enumerate() {
                if let Some(old) = self.tiles.get(x).and_then(|c| c.get(y)) {
                    cell.clone_from(old);
                }
            }
        }
        self.width = new_width;
        self.height = new_height;
        self.min = Point2D::ZERO;
        self.max = Point2D::ZERO;
        self.list.clear();
        for (x, col) in new_tiles.iter().enumerate() {
            for (y, cell) in col.iter().enumerate() {
                for tile in cell {
                    let (vx, vy) = (x as i32 - self.center.x, y as i32 - self.center.y);
                    self.min.x = self.min.x.min(vx);
                    self.min.y = self.min.y.min(vy);
                    self.max.x = self.max.x.max(vx);
                    self.max.y = self.max.y.max(vy);
                    self.list.push(MultiTileEntry::new(
                        tile.id,
                        vx as i16,
                        vy as i16,
                        i16::from(tile.z),
                        TileFlag::BACKGROUND,
                    ));
                }
            }
        }
        self.tiles = new_tiles;
    }
}

/// All multi footprints by multi id.
#[derive(Debug, Default)]
pub struct MultiData {
    components: HashMap<i32, MultiComponentList>,
    pub housing_uop_path: Option<PathBuf>,
    pub housing_entry: Option<UopEntry>,
}

impl MultiData {
    /// `MultiData.Configure`: `MultiCollection.uop` if present, else `multi.idx`/`multi.mul`. The `.mul` layout
    /// follows `maps.enablePostHSMultiComponentFormat` (default: client 7.0.9.0 or later, or unknown).
    pub fn load(
        dirs: &DataDirectories,
        config: &ServerConfiguration,
        server_client_version: Option<&ClientVersion>,
    ) -> Result<MultiData> {
        let mut data = MultiData::default();
        if let Some(path) = dirs.find("MultiCollection.uop") {
            data.load_uop(&path)?;
            return Ok(data);
        }
        let post_hs = config.get_setting(
            "maps.enablePostHSMultiComponentFormat",
            server_client_version.is_none_or(|v| v.ge(&ClientVersion::v7090())),
        );
        data.load_mul(
            &dirs.require("multi.idx")?,
            &dirs.require("multi.mul")?,
            post_hs,
        )?;
        Ok(data)
    }

    /// `GetComponents`: the footprint of `multi_id & 0x3FFF`, empty if there is none.
    pub fn get_components(&self, multi_id: i32) -> &MultiComponentList {
        self.components.get(&(multi_id & 0x3FFF)).unwrap_or(&EMPTY)
    }

    /// Registers a footprint without multi files, for hosts that generate their own (tests).
    pub fn set_components(&mut self, multi_id: i32, components: MultiComponentList) {
        self.components.insert(multi_id & 0x3FFF, components);
    }

    fn load_uop(&mut self, path: &Path) -> Result<()> {
        let name = path.to_string_lossy();
        let mut file = File::open(path).map_err(|e| DataError::io(path, e))?;
        let housing_hash = hash_little2("build/multicollection/housing.bin");
        let mut additional = HashMap::from([(housing_hash, UopEntry::default())]);
        let entries = read_uop_indexes(
            &mut file,
            &name,
            ".bin",
            0x10000,
            Some(4),
            6,
            Some(&mut additional),
        )?;

        for (i, entry) in entries {
            let corrupt = |why: &str| DataError::invalid(name.to_string(), why);
            file.seek(SeekFrom::Start(
                u64::try_from(entry.offset).map_err(|_| corrupt("negative offset"))?,
            ))?;
            let size = entry.size as usize;
            let data = if entry.compressed {
                let mut packed = vec![0u8; entry.compressed_size as usize];
                file.read_exact(&mut packed)
                    .map_err(|_| corrupt("Unexpected end of file."))?;
                let mut out = vec![0u8; size];
                match STANDARD.unpack(&mut out, &packed) {
                    Ok(n) if n == size => out,
                    _ => return Err(corrupt(&format!("Failed to unpack entry {i}."))),
                }
            } else {
                // this branch's bytes are read too, not left to the previous entry's buffer.
                let mut raw = vec![0u8; size];
                file.read_exact(&mut raw)
                    .map_err(|_| corrupt("Unexpected end of file."))?;
                raw
            };
            self.components.insert(
                i,
                parse_uop_entry(&data).map_err(|e| corrupt(&format!("entry {i}: {e}")))?,
            );
        }

        let housing = additional[&housing_hash];
        if housing.size > 0 {
            self.housing_uop_path = Some(path.to_path_buf());
            self.housing_entry = Some(housing);
        }
        Ok(())
    }

    fn load_mul(&mut self, idx_path: &Path, mul_path: &Path, post_hs: bool) -> Result<()> {
        let count = std::fs::metadata(idx_path)
            .map_err(|e| DataError::io(idx_path, e))?
            .len()
            / 12;
        let mut idx = BufReader::new(File::open(idx_path).map_err(|e| DataError::io(idx_path, e))?);
        let mut mul = BufReader::new(File::open(mul_path).map_err(|e| DataError::io(mul_path, e))?);
        for i in 0..count as i32 {
            let lookup = le::i32(&mut idx)?;
            let length = le::i32(&mut idx)?;
            le::i32(&mut idx)?; // Extra
            if lookup < 0 || length <= 0 {
                continue;
            }
            mul.seek(SeekFrom::Start(lookup as u64))?;
            let list = MultiComponentList::from_mul(&mut mul, length, post_hs).map_err(|e| {
                DataError::invalid(mul_path.to_string_lossy(), format!("multi {i}: {e}"))
            })?;
            self.components.insert(i, list);
        }
        Ok(())
    }
}

/// One `MultiCollection.uop` entry: a 4-byte skip, a count, then per tile id/x/y/z/flag and a list of cliloc
/// numbers that are skipped.
fn parse_uop_entry(data: &[u8]) -> muo_io::Result<MultiComponentList> {
    let mut r = SpanReader::new(data);
    r.seek(4, SeekOrigin::Begin)?;
    let count = r.read_u32_le()?;
    let mut list = Vec::new();
    for _ in 0..count {
        let item_id = r.read_u16_le()?;
        let (x, y, z) = (r.read_i16_le()?, r.read_i16_le()?, r.read_i16_le()?);
        let flags = match r.read_u16_le()? {
            1 => TileFlag::empty(),
            257 => TileFlag::GENERIC,
            _ => TileFlag::BACKGROUND, // 0
        };
        let cliloc_count = r.read_u32_le()?;
        r.seek(i64::from(cliloc_count) * 4, SeekOrigin::Current)?; // bypass binary block
        list.push(MultiTileEntry::new(item_id, x, y, z, flags));
    }
    MultiComponentList::from_list(list).map_err(|e| muo_io::IoError::InvalidData(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use muo_io::{SaveReader, SaveWriter};

    const WALL: u16 = 0x3000;
    const FLOOR: u16 = 0x3001;
    const VISIBLE: TileFlag = TileFlag::BACKGROUND;

    /// `WalledHouse`: a test house of walls around a square, with an optional doorway.
    fn walled_house(radius: i16, doorway_on_south: bool) -> MultiComponentList {
        let mut entries = Vec::new();
        for x in -radius..=radius {
            for y in -radius..=radius {
                let edge = x == -radius || x == radius || y == -radius || y == radius;
                let doorway = if doorway_on_south {
                    x == 0 && y == radius
                } else {
                    x == radius && y == 0
                };
                let id = if edge && !doorway { WALL } else { FLOOR };
                entries.push(MultiTileEntry::new(id, x, y, 1, VISIBLE));
            }
        }
        MultiComponentList::from_entries(&entries)
    }

    fn boat() -> MultiComponentList {
        let mut entries = Vec::new();
        for x in -2..=2 {
            for y in -3..=3 {
                entries.push(MultiTileEntry::new(
                    if x == -2 || x == 2 { WALL } else { FLOOR },
                    x,
                    y,
                    0,
                    VISIBLE,
                ));
            }
        }
        MultiComponentList::from_entries(&entries)
    }

    fn seeded() -> MultiData {
        let mut m = MultiData::default();
        m.set_components(0x74, walled_house(3, true));
        m.set_components(0x7A, walled_house(4, false));
        m.set_components(0, boat());
        m
    }

    #[test]
    fn seed_registers_footprint() {
        let m = seeded();
        for id in [0x74, 0x7A, 0] {
            assert!(!m.get_components(id).list.is_empty());
        }
        assert!(m.get_components(0x1234).list.is_empty());
    }

    #[test]
    fn footprints_differ_between_guild_house_and_tower() {
        let m = seeded();
        assert_ne!(
            m.get_components(0x74).width(),
            m.get_components(0x7A).width()
        );
    }

    #[test]
    fn guild_house_has_an_open_doorway() {
        let m = seeded();
        let house = m.get_components(0x74);
        let doorway = &house.tiles[house.center().x as usize][house.height() as usize - 1];
        assert_eq!(doorway.len(), 1);
        assert_eq!(doorway[0].id, FLOOR);
    }

    #[test]
    fn add_remove_resize() {
        let mut td = TileData::empty();
        td.items[usize::from(WALL)].height = 20;
        let mut house = walled_house(1, true);
        let n = house.list.len();
        house.add(i32::from(WALL), 0, 0, 1, &td); // replaces nothing of the same class (floor has height 0)
        assert_eq!(house.list.len(), n + 1);
        house.remove(i32::from(WALL), 0, 0, 1);
        assert_eq!(house.list.len(), n);
        house.add(5, 99, 99, 0, &td); // outside the footprint: ignored
        assert_eq!(house.list.len(), n);
        house.resize(2, 2);
        assert_eq!((house.width(), house.height()), (2, 2));
        assert_eq!(
            house.list.len(),
            house.tiles.iter().flatten().map(Vec::len).sum::<usize>()
        );
    }

    #[test]
    fn save_round_trip() {
        let house = walled_house(2, true);
        let mut w = SaveWriter::new(false);
        house.serialize(&mut w);
        let bytes = w.into_vec();
        let mut r = SaveReader::new(&bytes);
        let back = MultiComponentList::deserialize(&mut r).unwrap();
        assert_eq!(back.list, house.list);
        assert_eq!(back.tiles, house.tiles);
        assert_eq!(back.center(), house.center());
        // a tile outside the stored footprint is corrupt, not a panic
        assert!(MultiComponentList::deserialize(&mut SaveReader::new(&bytes[..40])).is_err());
    }

    #[test]
    fn mul_records() {
        // two tiles: (id 1 at 0,0) and (id 2 at 1,-1) in the 12 byte pre-HS layout
        let mut b = Vec::new();
        for (id, x, y, z, f) in [(1u16, 0i16, 0i16, 0i16, 1u32), (2, 1, -1, 5, 1)] {
            b.extend_from_slice(&id.to_le_bytes());
            for v in [x, y, z] {
                b.extend_from_slice(&v.to_le_bytes());
            }
            b.extend_from_slice(&f.to_le_bytes());
        }
        let m = MultiComponentList::from_mul(&mut b.as_slice(), 24, false).unwrap();
        assert_eq!((m.width(), m.height()), (2, 2));
        assert_eq!(m.center(), Point2D::new(0, 1));
        assert_eq!(m.tiles[1][0][0].id, 2);
        assert!(MultiComponentList::from_mul(&mut b[..10].as_ref(), 24, false).is_err());
    }
}
