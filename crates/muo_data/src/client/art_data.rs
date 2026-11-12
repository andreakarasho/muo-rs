//! Art data: the opaque bounds of item art (`art.mul`/`artidx.mul` or `artLegacyMUL.uop`).

use super::uop::{UopEntry, read_uop_indexes};
use crate::data_directories::DataDirectories;
use crate::error::{DataError, Result};
use crate::le;
use muo_primitives::Rectangle2D;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Seek, SeekFrom};

pub struct ArtData {
    data: Option<File>,
    ranges: HashMap<i32, UopEntry>,
}

impl ArtData {
    /// The art files are optional: with none found `is_initialized` is false and every bound is empty.
    pub fn open(dirs: &DataDirectories) -> Result<ArtData> {
        if let Some(art) = dirs.find("art.mul")
            && let Some(idx) = dirs.find("artidx.mul")
        {
            let ranges = load_mul_ranges(&idx)?;
            let data = File::open(&art).map_err(|e| DataError::io(&art, e))?;
            return Ok(ArtData {
                data: Some(data),
                ranges,
            });
        }
        if let Some(uop) = dirs.find("artLegacyMUL.uop") {
            let mut data = File::open(&uop).map_err(|e| DataError::io(&uop, e))?;
            let ranges = read_uop_indexes(
                &mut data,
                &uop.to_string_lossy(),
                ".tga",
                0x14000,
                Some(5),
                8,
                None,
            )?;
            return Ok(ArtData {
                data: Some(data),
                ranges,
            });
        }
        Ok(ArtData {
            data: None,
            ranges: HashMap::new(),
        })
    }

    pub fn is_initialized(&self) -> bool {
        !self.ranges.is_empty() && self.data.is_some()
    }

    /// `(width, height, opaque bounds)` of static art `index`; all empty for a missing or blank image.
    pub fn get_static_bounds(&mut self, index: i32) -> Result<(u16, u16, Rectangle2D)> {
        const NONE: (u16, u16, Rectangle2D) = (0, 0, Rectangle2D::EMPTY);
        if !(0..=0x10000).contains(&index) {
            return Ok(NONE);
        }
        let Some(entry) = self.ranges.get(&(index + 16384)).copied() else {
            return Ok(NONE);
        };
        let Some(file) = self.data.as_mut() else {
            return Ok(NONE);
        };

        file.seek(SeekFrom::Start(
            u64::try_from(entry.offset)
                .map_err(|_| DataError::invalid("art", "negative offset"))?,
        ))?;
        let mut bytes = vec![0u8; entry.size.max(0) as usize];
        let got = le::read_fully(file, &mut bytes)?;
        bytes.truncate(got);
        let buffer: Vec<u16> = bytes
            .as_chunks::<2>()
            .0
            .iter()
            .map(|c| u16::from_le_bytes(*c))
            .collect();
        if buffer.len() < 4 {
            return Err(DataError::invalid(
                "art",
                format!("art entry {index} is truncated"),
            ));
        }
        let (width, height) = (buffer[2], buffer[3]);
        if width == 0 || height == 0 {
            return Ok(NONE);
        }
        let bounds = bounds_from_rgba1555_bitmap(i32::from(width), i32::from(height), &buffer[4..])
            .ok_or_else(|| DataError::invalid("art", format!("art entry {index} is corrupt")))?;
        Ok((width, height, bounds))
    }
}

fn load_mul_ranges(idx_path: &std::path::Path) -> Result<HashMap<i32, UopEntry>> {
    let mut f = BufReader::new(File::open(idx_path).map_err(|e| DataError::io(idx_path, e))?);
    let count = std::fs::metadata(idx_path)
        .map_err(|e| DataError::io(idx_path, e))?
        .len()
        / 12;
    let mut ranges = HashMap::with_capacity(count as usize);
    for i in 0..count as i32 {
        let offset = le::i32(&mut f)?;
        let length = le::i32(&mut f)?;
        le::i32(&mut f)?; // Skip the Data field since we don't use it
        if length > 0 {
            ranges.insert(i, UopEntry::new(i64::from(offset), length));
        }
    }
    Ok(ranges)
}

/// `GetBoundsFromRGBA1555Bitmap`: the box of non-black pixels of run-length art, `EMPTY` if there are none.
/// `None` when the run data points outside the buffer (a corrupt file).
pub fn bounds_from_rgba1555_bitmap(width: i32, height: i32, data: &[u16]) -> Option<Rectangle2D> {
    let h = usize::try_from(height).ok()?;
    let lookups = data.get(..h)?;
    let data = &data[h..];
    let (mut x_min, mut y_min, mut x_max, mut y_max) = (width, height, -1, -1);

    for (y, &lookup) in lookups.iter().enumerate() {
        let mut i = usize::from(lookup);
        let mut x = 0i32;
        loop {
            let start_x = i32::from(*data.get(i)?);
            let pixel_count = i32::from(*data.get(i + 1)?);
            i += 2;
            if start_x + pixel_count == 0 || start_x > width {
                break;
            }
            x += start_x;
            let end = x + pixel_count;
            while x < end {
                let pixel = *data.get(i)?;
                i += 1;
                if pixel != 0 {
                    x_min = x_min.min(x);
                    x_max = x_max.max(x);
                    y_min = y_min.min(y as i32);
                    y_max = y_max.max(y as i32);
                }
                x += 1;
            }
        }
    }
    Some(if x_max == -1 {
        Rectangle2D::EMPTY
    } else {
        Rectangle2D::new(x_min, y_min, x_max - x_min, y_max - y_min)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bounds_of_a_small_image() {
        // 4x2: lookups [0, 6] index the runs after the table; row 0 is (start 1, count 2, two pixels) and a
        // (0, 0) terminator, row 1 is only a terminator
        let data: Vec<u16> = vec![0, 6, 1, 2, 0x7FFF, 0x7FFF, 0, 0, 0, 0];
        let r = bounds_from_rgba1555_bitmap(4, 2, &data).unwrap();
        assert_eq!((r.x(), r.y(), r.width(), r.height()), (1, 0, 1, 0));
    }

    #[test]
    fn blank_and_corrupt() {
        assert_eq!(
            bounds_from_rgba1555_bitmap(2, 1, &[0, 0, 0]).unwrap(),
            Rectangle2D::EMPTY
        );
        assert!(bounds_from_rgba1555_bitmap(2, 4, &[0]).is_none());
        // a run pointing past the end
        assert!(bounds_from_rgba1555_bitmap(2, 1, &[0, 0, 1]).is_none());
    }
}
