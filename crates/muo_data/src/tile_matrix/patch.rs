//! Map patches: `mapdifN`/`stadifN` diff files applied over a matrix.

use super::{TileMatrix, TileMatrixOptions, group_statics, land_block_from_bytes};
use crate::data_directories::DataDirectories;
use crate::error::{DataError, Result};
use crate::le;
use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::Path;

/// `TileMatrixPatch.LandBlocks` / `StaticBlocks`.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub struct PatchCounts {
    pub land_blocks: i32,
    pub static_blocks: i32,
}

/// The constructor of `TileMatrixPatch`: applies the enabled patches for map file `index` to `matrix`.
pub fn apply_patches(
    matrix: &mut TileMatrix,
    dirs: &DataDirectories,
    index: i32,
    options: &TileMatrixOptions,
) -> Result<PatchCounts> {
    let mut counts = PatchCounts::default();
    if options.patch_land
        && let Some(data) = dirs.find(&format!("mapdif{index}.mul"))
        && let Some(idx) = dirs.find(&format!("mapdifl{index}.mul"))
    {
        counts.land_blocks = patch_land(matrix, &data, &idx)?;
    }
    if options.patch_statics
        && let Some(data) = dirs.find(&format!("stadif{index}.mul"))
        && let Some(idx) = dirs.find(&format!("stadifl{index}.mul"))
        && let Some(lookup) = dirs.find(&format!("stadifi{index}.mul"))
    {
        counts.static_blocks = patch_statics(matrix, &data, &idx, &lookup)?;
    }
    Ok(counts)
}

fn open(path: &Path) -> Result<File> {
    File::open(path).map_err(|e| DataError::io(path, e))
}

fn truncated(path: &Path) -> impl Fn(std::io::Error) -> DataError + '_ {
    move |e| {
        DataError::invalid(
            path.to_string_lossy(),
            format!("Unexpected end of file ({e})."),
        )
    }
}

/// `Math.DivRem(blockID, BlockHeight)`, `(0, 0)`-safe for a matrix with no blocks.
fn div_rem(block_id: i32, block_height: i32) -> (i32, i32) {
    if block_height == 0 {
        (-1, -1)
    } else {
        (block_id / block_height, block_id % block_height)
    }
}

fn patch_land(matrix: &mut TileMatrix, data_path: &Path, index_path: &Path) -> Result<i32> {
    let mut data = BufReader::new(open(data_path)?);
    let mut index = BufReader::new(open(index_path)?);
    let count = (std::fs::metadata(index_path)
        .map_err(|e| DataError::io(index_path, e))?
        .len()
        / 4) as i32;

    for _ in 0..count {
        let block_id = le::i32(&mut index).map_err(truncated(index_path))?;
        let (x, y) = div_rem(block_id, matrix.block_height());
        data.seek_relative(4).map_err(truncated(data_path))?;
        let mut bytes = [0u8; 192];
        data.read_exact(&mut bytes).map_err(truncated(data_path))?;
        matrix.set_land_block(x, y, land_block_from_bytes(&bytes));
    }
    Ok(count)
}

fn patch_statics(
    matrix: &mut TileMatrix,
    data_path: &Path,
    index_path: &Path,
    lookup_path: &Path,
) -> Result<i32> {
    let mut data = open(data_path)?;
    let mut index = BufReader::new(open(index_path)?);
    let mut lookup_r = BufReader::new(open(lookup_path)?);
    let count = (std::fs::metadata(index_path)
        .map_err(|e| DataError::io(index_path, e))?
        .len()
        / 4) as i32;

    for _ in 0..count {
        let block_id = le::i32(&mut index).map_err(truncated(index_path))?;
        let (block_x, block_y) = div_rem(block_id, matrix.block_height());
        let offset = le::i32(&mut lookup_r).map_err(truncated(lookup_path))?;
        let length = le::i32(&mut lookup_r).map_err(truncated(lookup_path))?;
        le::i32(&mut lookup_r).map_err(truncated(lookup_path))?; // Extra

        if offset < 0 || length <= 0 {
            let empty = matrix.empty_static_block().clone();
            matrix.set_static_block(block_x, block_y, empty);
            continue;
        }
        if length > super::MAX_BLOCK_BYTES {
            return Err(DataError::invalid(
                lookup_path.to_string_lossy(),
                "Static block length is out of range.",
            ));
        }
        data.seek(SeekFrom::Start(offset as u64))?;
        let mut bytes = vec![0u8; length as usize];
        data.read_exact(&mut bytes).map_err(truncated(data_path))?;
        matrix.set_static_block(block_x, block_y, group_statics(&bytes));
    }
    Ok(count)
}
