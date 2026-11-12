//! UOP files and their entries.

use crate::error::{DataError, Result};
use crate::le;
use muo_primitives::dotnet::InvariantCase;
use std::collections::HashMap;
use std::io::{Read, Seek, SeekFrom};

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub struct UopEntry {
    pub offset: i64,
    pub size: i32,
    pub compressed: bool,
    pub compressed_size: i32,
    pub extra: i32,
}

impl UopEntry {
    pub fn new(offset: i64, length: i32) -> Self {
        UopEntry {
            offset,
            size: length,
            ..Default::default()
        }
    }
}

/// `UOPFiles.ReadUOPIndexes`. `name` is the file's path (its stem builds the entry names and it prefixes errors);
/// `expected_version` is `None` for any. Entries whose hash is in `additional` fill that map instead.
pub fn read_uop_indexes<R: Read + Seek>(
    stream: &mut R,
    name: &str,
    file_ext: &str,
    entry_count: usize,
    expected_version: Option<i32>,
    precision: usize,
    mut additional: Option<&mut HashMap<u64, UopEntry>>,
) -> Result<HashMap<i32, UopEntry>> {
    let truncated =
        |e: std::io::Error| DataError::invalid(name, format!("Unexpected end of file ({e})."));
    let length = stream.seek(SeekFrom::End(0))? as i64;
    stream.seek(SeekFrom::Start(0))?;

    // MYP\0
    if le::u32(stream).map_err(truncated)? != 0x50594D {
        return Err(DataError::invalid(name, "The file is not a UOP file."));
    }
    let version = le::i32(stream).map_err(truncated)?;
    if let Some(expected) = expected_version
        && version != expected
    {
        return Err(DataError::invalid(
            name,
            format!("Expected version {expected}."),
        ));
    }
    // Signature
    if le::u32(stream).map_err(truncated)? != 0xFD23EC43 {
        return Err(DataError::invalid(name, "Invalid signature."));
    }

    let hashes = generate_hashes(name, file_ext, entry_count, precision);
    let mut next_block = le::i64(stream).map_err(truncated)?;
    let mut entries = HashMap::with_capacity(0x10000);

    loop {
        if next_block < 0 {
            return Err(DataError::invalid(name, "Invalid block offset."));
        }
        stream.seek(SeekFrom::Start(next_block as u64))?;
        let file_count = le::i32(stream).map_err(truncated)?;
        next_block = le::i64(stream).map_err(truncated)?;

        for _ in 0..file_count.max(0) {
            let offset = le::i64(stream).map_err(truncated)?;
            let header_length = le::i32(stream).map_err(truncated)?;
            let compressed_length = le::i32(stream).map_err(truncated)?;
            let decompressed_length = le::i32(stream).map_err(truncated)?;
            let file_name_hash = le::u64(stream).map_err(truncated)?;
            le::u32(stream).map_err(truncated)?; // Adler32
            let compressed = le::i16(stream).map_err(truncated)? == 1;

            if offset == 0 || compressed_length <= 0 || decompressed_length <= 0 {
                continue;
            }
            let entry = UopEntry {
                compressed,
                compressed_size: compressed_length,
                ..UopEntry::new(offset + i64::from(header_length), decompressed_length)
            };
            if let Some(&index) = hashes.get(&file_name_hash) {
                entries.insert(index, entry);
            } else if let Some(add) = additional.as_deref_mut()
                && let Some(slot) = add.get_mut(&file_name_hash)
            {
                *slot = entry;
            }
        }

        if next_block == 0 || next_block >= length {
            break;
        }
    }
    entries.shrink_to_fit();
    Ok(entries)
}

fn generate_hashes(
    file_path: &str,
    ext: &str,
    entry_count: usize,
    precision: usize,
) -> HashMap<u64, i32> {
    let stem = std::path::Path::new(file_path)
        .file_stem()
        .map(|s| s.to_string_lossy().lower_inv())
        .unwrap_or_default();
    let root = format!("build/{stem}");
    (0..entry_count)
        .map(|i| {
            (
                hash_little2(&format!("{root}/{i:0precision$}{ext}")),
                i as i32,
            )
        })
        .collect()
}

/// Bob Jenkins' `hashlittle2` over the UTF-16 code units, `b << 32 | c`.
pub fn hash_little2(s: &str) -> u64 {
    let s: Vec<u32> = s.encode_utf16().map(u32::from).collect();
    let mut length = s.len();
    let mut a = 0xDEADBEEFu32.wrapping_add(length as u32);
    let mut b = a;
    let mut c = a;
    let mut k = 0;

    while length > 12 {
        a = a
            .wrapping_add(s[k])
            .wrapping_add(s[k + 1] << 8)
            .wrapping_add(s[k + 2] << 16)
            .wrapping_add(s[k + 3] << 24);
        b = b
            .wrapping_add(s[k + 4])
            .wrapping_add(s[k + 5] << 8)
            .wrapping_add(s[k + 6] << 16)
            .wrapping_add(s[k + 7] << 24);
        c = c
            .wrapping_add(s[k + 8])
            .wrapping_add(s[k + 9] << 8)
            .wrapping_add(s[k + 10] << 16)
            .wrapping_add(s[k + 11] << 24);

        a = a.wrapping_sub(c);
        a ^= c.rotate_left(4);
        c = c.wrapping_add(b);
        b = b.wrapping_sub(a);
        b ^= a.rotate_left(6);
        a = a.wrapping_add(c);
        c = c.wrapping_sub(b);
        c ^= b.rotate_left(8);
        b = b.wrapping_add(a);
        a = a.wrapping_sub(c);
        a ^= c.rotate_left(16);
        c = c.wrapping_add(b);
        b = b.wrapping_sub(a);
        b ^= a.rotate_left(19);
        a = a.wrapping_add(c);
        c = c.wrapping_sub(b);
        c ^= b.rotate_left(4);
        b = b.wrapping_add(a);

        length -= 12;
        k += 12;
    }

    if length != 0 {
        // Falls through from `length` down to 1.
        if length >= 12 {
            c = c.wrapping_add(s[k + 11] << 24);
        }
        if length >= 11 {
            c = c.wrapping_add(s[k + 10] << 16);
        }
        if length >= 10 {
            c = c.wrapping_add(s[k + 9] << 8);
        }
        if length >= 9 {
            c = c.wrapping_add(s[k + 8]);
        }
        if length >= 8 {
            b = b.wrapping_add(s[k + 7] << 24);
        }
        if length >= 7 {
            b = b.wrapping_add(s[k + 6] << 16);
        }
        if length >= 6 {
            b = b.wrapping_add(s[k + 5] << 8);
        }
        if length >= 5 {
            b = b.wrapping_add(s[k + 4]);
        }
        if length >= 4 {
            a = a.wrapping_add(s[k + 3] << 24);
        }
        if length >= 3 {
            a = a.wrapping_add(s[k + 2] << 16);
        }
        if length >= 2 {
            a = a.wrapping_add(s[k + 1] << 8);
        }
        a = a.wrapping_add(s[k]);

        c ^= b;
        c = c.wrapping_sub(b.rotate_left(14));
        a ^= c;
        a = a.wrapping_sub(c.rotate_left(11));
        b ^= a;
        b = b.wrapping_sub(a.rotate_left(25));
        c ^= b;
        c = c.wrapping_sub(b.rotate_left(16));
        a ^= c;
        a = a.wrapping_sub(c.rotate_left(4));
        b ^= a;
        b = b.wrapping_sub(a.rotate_left(14));
        c ^= b;
        c = c.wrapping_sub(b.rotate_left(24));
    }
    (u64::from(b) << 32) | u64::from(c)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    /// A one-block UOP holding `names` (as `build/<stem>/<index>.<ext>` hashes) with the given entries.
    pub fn build_uop(
        stem: &str,
        ext: &str,
        precision: usize,
        version: i32,
        items: &[(i32, &[u8])],
    ) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(&0x50594Du32.to_le_bytes());
        out.extend_from_slice(&version.to_le_bytes());
        out.extend_from_slice(&0xFD23EC43u32.to_le_bytes());
        out.extend_from_slice(&24i64.to_le_bytes()); // next block right after the header
        out.extend_from_slice(&[0u8; 4]); // pad to 28
        out.extend_from_slice(&(items.len() as i32).to_le_bytes());
        out.extend_from_slice(&0i64.to_le_bytes());
        let table_len = items.len() * 34;
        let mut data_at = (out.len() + table_len) as i64;
        let mut blobs = Vec::new();
        for (index, data) in items {
            let hash = hash_little2(&format!("build/{stem}/{index:0precision$}{ext}"));
            out.extend_from_slice(&data_at.to_le_bytes());
            out.extend_from_slice(&0i32.to_le_bytes()); // header length
            out.extend_from_slice(&(data.len() as i32).to_le_bytes());
            out.extend_from_slice(&(data.len() as i32).to_le_bytes());
            out.extend_from_slice(&hash.to_le_bytes());
            out.extend_from_slice(&0u32.to_le_bytes());
            out.extend_from_slice(&0i16.to_le_bytes());
            data_at += data.len() as i64;
            blobs.extend_from_slice(data);
        }
        out.extend_from_slice(&blobs);
        out
    }

    #[test]
    fn hash_known_values() {
        // hashlittle2 of the empty string: no mixing, the three initial words are all 0xDEADBEEF
        assert_eq!(hash_little2(""), 0xDEADBEEF_DEADBEEF);
        // deterministic and sensitive to every char, including past the 12-char block loop
        assert_ne!(
            hash_little2("build/map0legacymul/00000000.dat"),
            hash_little2("build/map0legacymul/00000001.dat")
        );
        assert_eq!(hash_little2("a"), hash_little2("a"));
    }

    #[test]
    fn reads_entries_and_rejects_corruption() {
        let bytes = build_uop("map0legacymul", ".dat", 8, 5, &[(0, b"abc"), (7, b"hello")]);
        let mut e = read_uop_indexes(
            &mut Cursor::new(&bytes),
            "x/map0LegacyMUL.uop",
            ".dat",
            16,
            Some(5),
            8,
            None,
        )
        .unwrap();
        assert_eq!(e.len(), 2);
        assert_eq!(e[&7].size, 5);
        assert_eq!(e.remove(&0).unwrap().size, 3);

        let wrong_version = read_uop_indexes(
            &mut Cursor::new(&bytes),
            "map0legacymul.uop",
            ".dat",
            16,
            Some(4),
            8,
            None,
        );
        assert!(wrong_version.is_err());
        let mut bad = bytes.clone();
        bad[0] = 0;
        assert!(
            read_uop_indexes(
                &mut Cursor::new(&bad),
                "map0legacymul.uop",
                ".dat",
                16,
                None,
                8,
                None
            )
            .is_err()
        );
        assert!(
            read_uop_indexes(
                &mut Cursor::new(&bytes[..20]),
                "map0legacymul.uop",
                ".dat",
                16,
                None,
                8,
                None
            )
            .is_err()
        );
    }

    #[test]
    fn additional_hashes_are_filled() {
        let bytes = build_uop("multicollection", ".bin", 6, 4, &[(3, b"xy")]);
        // no indexed names are wanted (entry count 0), so the entry lands in the additional map instead
        let extra = hash_little2("build/multicollection/000003.bin");
        let mut add = HashMap::from([(extra, UopEntry::default())]);
        let e = read_uop_indexes(
            &mut Cursor::new(&bytes),
            "MultiCollection.uop",
            ".bin",
            0,
            Some(4),
            6,
            Some(&mut add),
        )
        .unwrap();
        assert!(e.is_empty());
        assert_eq!(add[&extra].size, 2);
    }
}
