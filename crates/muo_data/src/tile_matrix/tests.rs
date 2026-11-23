//! Terrain tests (the injection cases that need no `Map`), plus file
//! backed cases over generated map files.

use super::*;
use crate::client::uop::hash_little2;
use std::path::PathBuf;

const X: i32 = 1400;
const Y: i32 = 1400;
const GRASS: i16 = 0x3000;
const DIRT: i16 = 0x3002;
const WALL: u16 = 0x3000;

/// A fresh scratch directory, removed on drop.
struct Scratch(PathBuf);

impl Scratch {
    fn new(name: &str) -> Scratch {
        let dir = std::env::temp_dir().join(format!("muo_data_{name}_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        Scratch(dir)
    }

    fn write(&self, name: &str, bytes: &[u8]) {
        std::fs::write(self.0.join(name), bytes).unwrap();
    }

    fn dirs(&self) -> DataDirectories {
        DataDirectories::new([self.0.to_string_lossy().to_string()])
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn block_with(first: LandTile) -> Vec<LandTile> {
    let mut b = vec![LandTile::default(); 64];
    b[0] = first;
    b
}

#[test]
fn stamping_one_tile_keeps_the_rest_of_the_block() {
    let mut m = TileMatrix::new_empty(7168, 4096);
    let mut block = m.get_land_block(X >> 3, Y >> 3).to_vec();
    block.truncate(64);
    block[(((Y & 7) << 3) + (X & 7)) as usize] = LandTile::new(GRASS, 3);
    block[(((Y & 7) << 3) + ((X + 1) & 7)) as usize] = LandTile::new(DIRT, 4);
    m.set_land_block(X >> 3, Y >> 3, block);

    assert_eq!(m.get_land_tile(X, Y).id, GRASS);
    assert_eq!(m.get_land_tile(X + 1, Y).id, DIRT);
    assert_eq!(m.get_land_tile(X, Y).z, 3);
}

#[test]
fn stamping_an_uninjected_block_does_not_touch_other_blocks() {
    let mut m = TileMatrix::new_empty(7168, 4096);
    m.set_land_block(X >> 3, Y >> 3, block_with(LandTile::new(GRASS, 3)));
    assert_eq!(m.get_land_tile(X + 64, Y + 64).id, 0);
    // another tile of the injected block is still zero
    assert_eq!(m.get_land_tile(X + 1, Y).id, 0);
}

#[test]
fn static_blocks_serve_what_was_injected() {
    let mut m = TileMatrix::new_empty(7168, 4096);
    assert!(m.get_static_block(X >> 3, Y >> 3)[(X & 7) as usize][(Y & 7) as usize].is_empty());

    let mut block = StaticBlock::default();
    block[(X & 7) as usize][(Y & 7) as usize].push(StaticTile::new(WALL, 3));
    m.set_static_block(X >> 3, Y >> 3, block);

    let statics = &m.get_static_block(X >> 3, Y >> 3)[(X & 7) as usize][(Y & 7) as usize];
    assert_eq!(statics.len(), 1);
    assert_eq!(statics[0].id, WALL);
}

#[test]
fn out_of_range_blocks_are_invalid_and_empty() {
    let mut m = TileMatrix::new_empty(64, 64);
    assert_eq!(m.block_width(), 8);
    assert_eq!(m.get_land_block(-1, 0).len(), 196);
    assert_eq!(m.get_land_block(8, 0).len(), 196);
    assert!(m.get_static_block(0, 8)[0][0].is_empty());
    m.set_land_block(99, 99, block_with(LandTile::new(1, 1))); // ignored
    assert_eq!(m.get_land_tile(-1, -1), LandTile::default());
}

#[test]
fn land_tile_ignored_ids() {
    assert!(LandTile::new(2, 0).ignored());
    assert!(LandTile::new(0x1DB, 0).ignored());
    assert!(LandTile::new(0x1B0, 0).ignored());
    assert!(!LandTile::new(0x1B6, 0).ignored());
}

#[test]
fn static_tile_height_comes_from_tiledata() {
    let mut td = TileData::empty();
    td.items[usize::from(WALL)].height = 20;
    assert_eq!(StaticTile::new(WALL, 0).height(&td), 20);
}

/// map0.mul with 2x2 blocks (16x16 tiles): block (bx, by) is at (bx * 2 + by) * 196, tile id = block * 100 + i.
fn map_bytes(blocks: usize) -> Vec<u8> {
    let mut out = Vec::new();
    for b in 0..blocks {
        out.extend_from_slice(&0i32.to_le_bytes()); // header
        for i in 0..64 {
            out.extend_from_slice(&((b * 100 + i) as i16).to_le_bytes());
            out.push(i as u8);
        }
    }
    out
}

fn static_rec(id: u16, x: u8, y: u8, z: i8, hue: i16) -> [u8; 7] {
    let mut r = [0u8; 7];
    r[..2].copy_from_slice(&id.to_le_bytes());
    r[2] = x;
    r[3] = y;
    r[4] = z as u8;
    r[5..].copy_from_slice(&hue.to_le_bytes());
    r
}

#[test]
fn reads_land_and_statics_from_files() {
    let s = Scratch::new("files");
    s.write("map0.mul", &map_bytes(4));
    // statics: block 3 (x1,y1) holds two tiles at (1, 2) and one at (9 & 7 = 1, 2): same cell, file order kept
    let mut statics = Vec::new();
    statics.extend_from_slice(&static_rec(0x100, 1, 2, 5, 7));
    statics.extend_from_slice(&static_rec(0x200, 9, 2, 6, 0));
    statics.extend_from_slice(&static_rec(0x300, 0, 0, 0, 0));
    s.write("statics0.mul", &statics);
    let mut idx = Vec::new();
    for b in 0..4 {
        if b == 3 {
            idx.extend_from_slice(&0i32.to_le_bytes());
            idx.extend_from_slice(&21i32.to_le_bytes());
        } else {
            idx.extend_from_slice(&(-1i32).to_le_bytes());
            idx.extend_from_slice(&(-1i32).to_le_bytes());
        }
        idx.extend_from_slice(&0i32.to_le_bytes());
    }
    s.write("staidx0.mul", &idx);

    let mut m = TileMatrix::open(&s.dirs(), 0, 0, 16, 16, &TileMatrixOptions::default()).unwrap();
    assert_ne!(m.map_files_fingerprint(), 0);
    // tile (3, 2) of block (1, 0) = block index 1*2+0 = 2
    assert_eq!(
        m.get_land_tile(8 + 3, 2),
        LandTile::new(200 + (2 * 8 + 3), (2 * 8 + 3) as i8)
    );
    assert_eq!(m.get_land_tile(0, 0), LandTile::new(0, 0));

    let block = m.get_static_block(1, 1);
    assert_eq!(
        block[1][2].iter().map(|t| t.id).collect::<Vec<_>>(),
        vec![0x100, 0x200]
    );
    assert_eq!(block[1][2][0], StaticTile::with_position(0x100, 1, 2, 5, 7));
    assert_eq!(block[0][0].len(), 1);
    assert!(m.get_static_block(0, 0)[1][2].is_empty());

    // the fingerprint changes when a file does
    let before = m.map_files_fingerprint();
    s.write("statics0.mul", &statics[..14]);
    let m2 = TileMatrix::open(&s.dirs(), 0, 0, 16, 16, &TileMatrixOptions::default()).unwrap();
    assert_ne!(before, m2.map_files_fingerprint());
}

#[test]
fn a_truncated_map_file_gives_invalid_blocks_not_a_panic() {
    let s = Scratch::new("trunc");
    s.write("map0.mul", &map_bytes(1)[..100]);
    let mut m = TileMatrix::open(&s.dirs(), 0, 0, 16, 16, &TileMatrixOptions::default()).unwrap();
    assert_eq!(m.get_land_block(0, 0).len(), 196);
    assert_eq!(m.get_land_block(1, 1).len(), 196);
}

#[test]
fn missing_files_leave_an_injection_only_matrix() {
    let s = Scratch::new("missing");
    let mut m = TileMatrix::open(&s.dirs(), 3, 3, 16, 16, &TileMatrixOptions::default()).unwrap();
    // an empty file set still has a fingerprint (the hash of nothing); the internal map has none
    assert_eq!(
        m.map_files_fingerprint(),
        TileMatrix::open(&s.dirs(), 4, 4, 16, 16, &TileMatrixOptions::default())
            .unwrap()
            .map_files_fingerprint()
    );
    assert_eq!(
        TileMatrix::open(&s.dirs(), 0x7F, 127, 16, 16, &TileMatrixOptions::default())
            .unwrap()
            .map_files_fingerprint(),
        0
    );
    assert_eq!(m.get_land_tile(0, 0), LandTile::default());
}

#[test]
fn diff_patches_replace_blocks_when_enabled() {
    let s = Scratch::new("patch");
    s.write("map0.mul", &map_bytes(4));
    // mapdif: replace block id 2 (x=1, y=0) with tile ids 9000+i
    let mut dif = Vec::new();
    dif.extend_from_slice(&0i32.to_le_bytes());
    for i in 0..64 {
        dif.extend_from_slice(&(9000 + i as i16).to_le_bytes());
        dif.push(0);
    }
    s.write("mapdif0.mul", &dif);
    s.write("mapdifl0.mul", &2i32.to_le_bytes());
    // stadif: block 1 gets one wall at (0,0)
    s.write("stadif0.mul", &static_rec(0x555, 0, 0, 1, 0));
    s.write("stadifl0.mul", &1i32.to_le_bytes());
    let mut lookup = Vec::new();
    for v in [0i32, 7, 0] {
        lookup.extend_from_slice(&v.to_le_bytes());
    }
    s.write("stadifi0.mul", &lookup);

    let off = TileMatrixOptions::default();
    let mut m = TileMatrix::open(&s.dirs(), 0, 0, 16, 16, &off).unwrap();
    assert_eq!(m.get_land_tile(8, 0).id, 200);
    assert_eq!(m.patches(), PatchCounts::default());

    let on = TileMatrixOptions {
        patch_land: true,
        patch_statics: true,
        ..Default::default()
    };
    let mut m = TileMatrix::open(&s.dirs(), 0, 0, 16, 16, &on).unwrap();
    assert_eq!(m.get_land_tile(8, 0).id, 9000);
    assert_eq!(m.get_land_tile(8 + 1, 0).id, 9001);
    assert_eq!(m.get_land_tile(0, 0).id, 0);
    assert_eq!(m.get_static_block(0, 1)[0][0][0].id, 0x555);
    assert_eq!(
        m.patches(),
        PatchCounts {
            land_blocks: 1,
            static_blocks: 1
        }
    );

    // a patch file shorter than its index says is corrupt
    s.write("mapdifl0.mul", &[2, 0, 0, 0, 3, 0, 0, 0]);
    assert!(TileMatrix::open(&s.dirs(), 0, 0, 16, 16, &on).is_err());
}

/// A `mapNLegacyMUL.uop` whose two chunks are stored in the wrong order; each chunk starts with its sequence number.
#[test]
fn legacy_uop_map_chunks_are_reordered_by_sequence() {
    let s = Scratch::new("uop");
    // chunk 0 (sequence 0): the first 196 bytes of blocks; here one block each so the order is visible
    let chunk = |seq: i32, block: usize| {
        let mut c = Vec::new();
        c.extend_from_slice(&seq.to_le_bytes());
        c.extend_from_slice(&map_bytes(block + 1)[block * 196..(block + 1) * 196][4..]);
        c // 4 (sequence) + 192 land bytes: the sequence number stands in for the 4-byte block header
    };
    let (first, second) = (chunk(0, 0), chunk(1, 1));
    let entries: [(usize, &Vec<u8>); 2] = [(1, &second), (0, &first)]; // stored second-chunk first

    let mut out = Vec::new();
    out.extend_from_slice(&0x50594Du32.to_le_bytes());
    out.extend_from_slice(&5i32.to_le_bytes());
    out.extend_from_slice(&0xFD23EC43u32.to_le_bytes());
    out.extend_from_slice(&24i64.to_le_bytes());
    out.extend_from_slice(&[0u8; 4]);
    out.extend_from_slice(&2i32.to_le_bytes());
    out.extend_from_slice(&0i64.to_le_bytes());
    let mut at = (out.len() + 2 * 34) as i64;
    for (index, data) in entries {
        out.extend_from_slice(&at.to_le_bytes());
        out.extend_from_slice(&0i32.to_le_bytes());
        out.extend_from_slice(&(data.len() as i32).to_le_bytes());
        out.extend_from_slice(&(data.len() as i32).to_le_bytes());
        out.extend_from_slice(
            &hash_little2(&format!("build/map0legacymul/{index:08}.dat")).to_le_bytes(),
        );
        out.extend_from_slice(&0u32.to_le_bytes());
        out.extend_from_slice(&0i16.to_le_bytes());
        at += data.len() as i64;
    }
    out.extend_from_slice(&second);
    out.extend_from_slice(&first);
    s.write("map0LegacyMUL.uop", &out);

    let mut m = TileMatrix::open(&s.dirs(), 0, 0, 16, 16, &TileMatrixOptions::default()).unwrap();
    // the virtual file is chunk0 then chunk1: offset 4 of the virtual file is the first land byte of block (0, 0)
    assert_eq!(m.find_offset(4), s_offset(&out, &first) + 4);
    assert_eq!(m.find_offset(1_000_000), -1);
    assert_eq!(m.get_land_block(0, 0).len(), 64);
    assert_eq!(m.get_land_tile(0, 0).id, 0);
    assert_eq!(m.get_land_tile(0, 8).id, 100); // block (0, 1) comes from the second chunk

    // a stale map0.mul beside it (some client installs keep one full of void tiles) is not what loads: the UOP is
    s.write("map0.mul", &[0xAA; 4 * 196]);
    let mut m = TileMatrix::open(&s.dirs(), 0, 0, 16, 16, &TileMatrixOptions::default()).unwrap();
    assert_eq!(m.get_land_tile(0, 8).id, 100);
}

fn s_offset(file: &[u8], chunk: &[u8]) -> i64 {
    file.windows(chunk.len()).position(|w| w == chunk).unwrap() as i64
}
