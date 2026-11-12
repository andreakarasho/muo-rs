//! A `TileMatrix` is the land and static tiles of one map file set, usable without any entity. The `Map` that
//! holds it hands out `Map.StaticTileEnumerable`s, which merge in multis and items; that part is muo_server's. The static-tile `Height` needs `TileData`, so `StaticTile::height` takes it.
//!
//! // shortcut: matrices sharing a map file (Felucca and Trammel both on map0) could share loaded blocks to save
//! memory. Each matrix reads its own, which gives the same lookup results; share through a registry above if memory
//! of two 7168x4096 maps ever matters.

mod patch;

pub use patch::{PatchCounts, apply_patches};

use crate::client::ClientVersion;
use crate::client::uop::{UopEntry, read_uop_indexes};
use crate::configuration::ServerConfiguration;
use crate::data_directories::DataDirectories;
use crate::error::{DataError, Result};
use crate::le;
use crate::tile_data::TileData;
use std::fs::File;
use std::hash::Hasher;
use std::io::{Read, Seek, SeekFrom};
use std::time::{Duration, Instant};

pub const SECTOR_SHIFT: i32 = 3;

/// `LandTile`: 3 bytes in `mapX.mul`.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub struct LandTile {
    pub id: i16,
    pub z: i8,
}

impl LandTile {
    pub fn new(id: i16, z: i8) -> LandTile {
        LandTile { id, z }
    }

    pub fn height(&self) -> i32 {
        0
    }

    /// Tiles the client draws as nothing: void, and the "no draw" tiles around cave walls.
    pub fn ignored(&self) -> bool {
        matches!(self.id, 2 | 0x1DB | 0x1AE..=0x1B5)
    }
}

/// `StaticTile`: 7 bytes in `staticsX.mul`.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub struct StaticTile {
    pub id: u16,
    pub x: u8,
    pub y: u8,
    pub z: i8,
    pub hue: i16,
}

impl StaticTile {
    pub const SIZE: usize = 7;

    pub fn new(id: u16, z: i8) -> StaticTile {
        StaticTile {
            id,
            z,
            ..Default::default()
        }
    }

    pub fn with_position(id: u16, x: u8, y: u8, z: i8, hue: i16) -> StaticTile {
        StaticTile { id, x, y, z, hue }
    }

    /// `Height`: the item's tiledata height.
    pub fn height(&self, tile_data: &TileData) -> i32 {
        i32::from(tile_data.item(usize::from(self.id)).height)
    }

    pub(crate) fn from_bytes(b: &[u8]) -> StaticTile {
        StaticTile {
            id: u16::from_le_bytes([b[0], b[1]]),
            x: b[2],
            y: b[3],
            z: b[4] as i8,
            hue: i16::from_le_bytes([b[5], b[6]]),
        }
    }
}

/// The statics of one 8x8 block, indexed `[x][y]` of the tile within the block, each in file order.
pub type StaticBlock = [[Vec<StaticTile>; 8]; 8];

/// Groups a block's raw 7-byte records by `(x & 7, y & 7)` (a trailing partial record is ignored, as `length / 7`).
pub(crate) fn group_statics(bytes: &[u8]) -> StaticBlock {
    let mut block = StaticBlock::default();
    for rec in bytes.as_chunks::<{ StaticTile::SIZE }>().0 {
        let t = StaticTile::from_bytes(rec);
        block[usize::from(t.x & 7)][usize::from(t.y & 7)].push(t);
    }
    block
}

pub(crate) fn land_block_from_bytes(b: &[u8]) -> Vec<LandTile> {
    b.as_chunks::<3>()
        .0
        .iter()
        .take(64)
        .map(|c| LandTile {
            id: i16::from_le_bytes([c[0], c[1]]),
            z: c[2] as i8,
        })
        .collect()
}

/// The `maps.enablePre6000Trammel`, `maps.enableMapDiffPatches` and `maps.enableStaticsDiffPatches` settings
/// (`TileMatrix.Configure`, `TileMatrixPatch.Configure`).
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub struct TileMatrixOptions {
    /// Pre-6.0.0 clients: map0 is both Felucca and Trammel.
    pub pre6000_trammel: bool,
    pub patch_land: bool,
    pub patch_statics: bool,
}

impl TileMatrixOptions {
    /// `server_client_version` is `UOClient.ServerClientVersion`; the patches stay off when it is unknown.
    pub fn from_config(
        config: &ServerConfiguration,
        server_client_version: Option<&ClientVersion>,
    ) -> Self {
        let before =
            |v: ClientVersion| server_client_version.is_some_and(|c| c.compare_to(&v).is_lt());
        TileMatrixOptions {
            pre6000_trammel: config
                .get_setting("maps.enablePre6000Trammel", before(ClientVersion::v6000())),
            patch_land: server_client_version.is_some()
                && config.get_setting("maps.enableMapDiffPatches", before(ClientVersion::v6000())),
            // TODO: Should this be enabled for 6.0 through <7.0.9?
            patch_statics: server_client_version.is_some()
                && config.get_setting(
                    "maps.enableStaticsDiffPatches",
                    before(ClientVersion::v7090()),
                ),
        }
    }
}

/// Largest plausible static block (a corrupt index must not allocate gigabytes).
pub(crate) const MAX_BLOCK_BYTES: i32 = 1 << 20;

const WARNING_INTERVAL: Duration = Duration::from_secs(60);

pub struct TileMatrix {
    file_index: i32,
    block_width: i32,
    block_height: i32,
    map_file: Option<File>,
    index_file: Option<File>,
    data_file: Option<File>,
    uop_map_entries: Option<Vec<UopEntry>>,
    fingerprint: u64,
    land_tiles: Vec<Option<Box<[LandTile]>>>,
    static_tiles: Vec<Option<Box<StaticBlock>>>,
    invalid_land_block: Vec<LandTile>,
    empty_static_block: StaticBlock,
    patches: PatchCounts,
    next_static_warning: Option<Instant>,
    next_land_warning: Option<Instant>,
}

impl TileMatrix {
    /// A matrix with no files: serves only what `set_land_block`/`set_static_block` inject (tests, generated maps,
    /// the internal map).
    pub fn new_empty(width: i32, height: i32) -> TileMatrix {
        let block_width = (width >> SECTOR_SHIFT).max(0);
        let block_height = (height >> SECTOR_SHIFT).max(0);
        let blocks = block_width as usize * block_height as usize;
        TileMatrix {
            file_index: 0x7F,
            block_width,
            block_height,
            map_file: None,
            index_file: None,
            data_file: None,
            uop_map_entries: None,
            fingerprint: 0,
            land_tiles: vec![None; blocks],
            static_tiles: (0..blocks).map(|_| None).collect(),
            invalid_land_block: vec![LandTile::default(); 196],
            empty_static_block: StaticBlock::default(),
            patches: PatchCounts::default(),
            next_static_warning: None,
            next_land_warning: None,
        }
    }

    /// The constructor: opens `mapN.mul` (or `mapNLegacyMUL.uop`), `staidxN.mul` and `staticsN.mul` from the data
    /// directories and applies the diff patches. Files that are not found only log a warning (that block type is then
    /// served from injected blocks alone); files that are found but corrupt are an error. File index `0x7F` (the
    /// internal map) opens nothing.
    pub fn open(
        dirs: &DataDirectories,
        file_index: i32,
        map_id: i32,
        width: i32,
        height: i32,
        options: &TileMatrixOptions,
    ) -> Result<TileMatrix> {
        let mut m = TileMatrix::new_empty(width, height);
        m.file_index = file_index;

        if file_index != 0x7F {
            let map_file_index = if options.pre6000_trammel && map_id == 1 {
                0
            } else {
                file_index
            };
            m.open_map(dirs, map_file_index)?;

            m.index_file = open_optional(dirs, &format!("staidx{map_file_index}.mul"))?;
            m.data_file = open_optional(dirs, &format!("statics{map_file_index}.mul"))?;
            m.fingerprint = m.compute_map_files_fingerprint()?;
        }

        m.patches = apply_patches(&mut m, dirs, file_index, options)?;
        Ok(m)
    }

    fn open_map(&mut self, dirs: &DataDirectories, map_file_index: i32) -> Result<()> {
        if let Some(f) = open_optional(dirs, &format!("map{map_file_index}.mul"))? {
            self.map_file = Some(f);
            return Ok(());
        }
        let uop_name = format!("map{map_file_index}LegacyMUL.uop");
        let Some(path) = dirs.find(&uop_name) else {
            tracing::warn!("map{map_file_index}.mul was not found.");
            return Ok(());
        };
        let mut file = File::open(&path).map_err(|e| DataError::io(&path, e))?;
        let entries = read_uop_indexes(
            &mut file,
            &path.to_string_lossy(),
            ".dat",
            0x14000,
            Some(5),
            8,
            None,
        )?;
        let mut entries: Vec<UopEntry> = entries.into_values().collect();
        self.uop_map_entries = Some(convert_to_map_entries(
            &mut file,
            &mut entries,
            &path.to_string_lossy(),
        )?);
        self.map_file = Some(file);
        Ok(())
    }

    /// XxHash3 over the bytes of the three on-disk files (map, staidx, statics), in that order. The .mul format has
    /// no CRC and CentrEd / UOFiddler rewrite these files in place, so this is the only way to see that a map was
    /// patched or saved. Computed once at open.
    fn compute_map_files_fingerprint(&mut self) -> Result<u64> {
        let mut hasher = muo_primitives::hash::create_xxhash3();
        let mut buf = [0u8; 64 * 1024];
        for file in [
            &mut self.map_file,
            &mut self.index_file,
            &mut self.data_file,
        ]
        .into_iter()
        .flatten()
        {
            file.seek(SeekFrom::Start(0))?;
            loop {
                let n = file.read(&mut buf)?;
                if n == 0 {
                    break;
                }
                hasher.write(&buf[..n]);
            }
        }
        Ok(hasher.finish())
    }

    pub fn file_index(&self) -> i32 {
        self.file_index
    }

    pub fn block_width(&self) -> i32 {
        self.block_width
    }

    pub fn block_height(&self) -> i32 {
        self.block_height
    }

    /// `MapFilesFingerprint`: 0 when no files are open.
    pub fn map_files_fingerprint(&self) -> u64 {
        self.fingerprint
    }

    /// The `LandBlocks`/`StaticBlocks` counts of the applied `TileMatrixPatch`.
    pub fn patches(&self) -> PatchCounts {
        self.patches
    }

    pub fn empty_static_block(&self) -> &StaticBlock {
        &self.empty_static_block
    }

    fn block_index(&self, x: i32, y: i32) -> Option<usize> {
        if x < 0 || y < 0 || x >= self.block_width || y >= self.block_height {
            None
        } else {
            Some(x as usize * self.block_height as usize + y as usize)
        }
    }

    pub fn set_static_block(&mut self, x: i32, y: i32, value: StaticBlock) {
        if let Some(i) = self.block_index(x, y) {
            self.static_tiles[i] = Some(Box::new(value));
        }
    }

    /// The statics of block `(x, y)` (in 8x8 blocks), read on first use. An unreadable block is empty.
    pub fn get_static_block(&mut self, x: i32, y: i32) -> &StaticBlock {
        let Some(i) = self.block_index(x, y) else {
            return &self.empty_static_block;
        };
        // A map without static files (tests, generated maps) serves only what was injected.
        if self.static_tiles[i].is_none() && self.data_file.is_some() && self.index_file.is_some() {
            let block = self
                .read_static_block(x, y)
                .unwrap_or_else(|| self.empty_static_block.clone());
            self.static_tiles[i] = Some(Box::new(block));
        }
        match &self.static_tiles[i] {
            Some(b) => b,
            None => &self.empty_static_block,
        }
    }

    pub fn set_land_block(&mut self, x: i32, y: i32, value: Vec<LandTile>) {
        if let Some(i) = self.block_index(x, y) {
            self.land_tiles[i] = Some(value.into_boxed_slice());
        }
    }

    /// The 64 land tiles of block `(x, y)`, read on first use. An out of range or unreadable block is 196 zero tiles.
    pub fn get_land_block(&mut self, x: i32, y: i32) -> &[LandTile] {
        let Some(i) = self.block_index(x, y) else {
            return &self.invalid_land_block;
        };
        // A map without a map file (tests, generated maps) serves only what was injected.
        if self.land_tiles[i].is_none() && self.map_file.is_some() {
            let block = self
                .read_land_block(x, y)
                .unwrap_or_else(|| self.invalid_land_block.clone());
            self.land_tiles[i] = Some(block.into_boxed_slice());
        }
        match &self.land_tiles[i] {
            Some(b) => b,
            None => &self.invalid_land_block,
        }
    }

    pub fn get_land_tile(&mut self, x: i32, y: i32) -> LandTile {
        let tiles = self.get_land_block(x >> SECTOR_SHIFT, y >> SECTOR_SHIFT);
        tiles[(((y & 7) << 3) + (x & 7)) as usize]
    }

    fn warn_throttled(slot: &mut Option<Instant>, kind: &str, x: i32, y: i32) {
        let now = Instant::now();
        if slot.is_none_or(|t| now >= t) {
            tracing::warn!("Warning: {kind} EOS for map file {x}, {y}");
            *slot = Some(now + WARNING_INTERVAL);
        }
    }

    fn read_static_block(&mut self, x: i32, y: i32) -> Option<StaticBlock> {
        let result = (|| -> std::io::Result<StaticBlock> {
            let index = self
                .index_file
                .as_mut()
                .ok_or(std::io::ErrorKind::NotFound)?;
            index.seek(SeekFrom::Start(
                (i64::from(x) * i64::from(self.block_height) + i64::from(y)) as u64 * 12,
            ))?;
            let lookup = le::i32(index)?;
            let length = le::i32(index)?;
            if lookup < 0 || length <= 0 {
                return Ok(StaticBlock::default());
            }
            if length > MAX_BLOCK_BYTES {
                return Err(std::io::ErrorKind::InvalidData.into());
            }
            let data = self
                .data_file
                .as_mut()
                .ok_or(std::io::ErrorKind::NotFound)?;
            data.seek(SeekFrom::Start(lookup as u64))?;
            let mut bytes = vec![0u8; length as usize];
            data.read_exact(&mut bytes)?;
            Ok(group_statics(&bytes))
        })();
        match result {
            Ok(block) => Some(block),
            Err(_) => {
                Self::warn_throttled(&mut self.next_static_warning, "Static", x, y);
                None
            }
        }
    }

    fn read_land_block(&mut self, x: i32, y: i32) -> Option<Vec<LandTile>> {
        let mut offset = (i64::from(x) * i64::from(self.block_height) + i64::from(y)) * 196 + 4;
        if let Some(entries) = &self.uop_map_entries {
            offset = find_offset(entries, offset);
        }
        let result = (|| -> std::io::Result<Vec<LandTile>> {
            let file = self.map_file.as_mut().ok_or(std::io::ErrorKind::NotFound)?;
            file.seek(SeekFrom::Start(
                u64::try_from(offset).map_err(|_| std::io::ErrorKind::InvalidInput)?,
            ))?;
            let mut bytes = [0u8; 192];
            file.read_exact(&mut bytes)?;
            Ok(land_block_from_bytes(&bytes))
        })();
        match result {
            Ok(b) => Some(b),
            Err(_) => {
                Self::warn_throttled(&mut self.next_land_warning, "Land", x, y);
                None
            }
        }
    }

    /// `FindOffset`: maps a virtual offset of the concatenated UOP chunks to a file offset, -1 past the end.
    pub fn find_offset(&self, offset: i64) -> i64 {
        self.uop_map_entries
            .as_ref()
            .map_or(offset, |e| find_offset(e, offset))
    }
}

fn find_offset(entries: &[UopEntry], offset: i64) -> i64 {
    let mut total = 0i64;
    for entry in entries {
        let new_total = total + i64::from(entry.size);
        if offset < new_total {
            return entry.offset + (offset - total);
        }
        total = new_total;
    }
    -1
}

/// `ConvertToMapEntries`: chunks are stored out of order; each starts with its sequence number.
fn convert_to_map_entries(
    file: &mut File,
    entries: &mut [UopEntry],
    name: &str,
) -> Result<Vec<UopEntry>> {
    // Sorting by offset to make seeking faster
    entries.sort_by_key(|e| e.offset);
    for e in entries.iter_mut() {
        file.seek(SeekFrom::Start(
            u64::try_from(e.offset)
                .map_err(|_| DataError::invalid(name, "negative chunk offset"))?,
        ))?;
        e.extra = le::i32(file).map_err(|_| DataError::invalid(name, "Unexpected end of file."))?; // order
    }
    entries.sort_by_key(|e| e.extra);
    Ok(entries.to_vec())
}

fn open_optional(dirs: &DataDirectories, name: &str) -> Result<Option<File>> {
    match dirs.find(name) {
        Some(p) => Ok(Some(File::open(&p).map_err(|e| DataError::io(&p, e))?)),
        None => {
            if !name.starts_with("map") {
                tracing::warn!("{name} was not found.");
            }
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests;
