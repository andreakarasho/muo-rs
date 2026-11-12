//! The entity-persistence file layout, as functions over bytes.
//!
//! `<Save>/<Name>/<Name>.idx` (version 5):
//!
//! ```text
//! i32 version = 5
//! i64 save start ticks        anchor for anchored times (v5)
//! i32 type count
//!     raw string full name    x type count (7-bit length, UTF-8)
//! i32 record count
//!     u16 type index | u32 serial | i64 created ticks | i64 bin position | i32 length     x record count (26 bytes)
//! ```
//!
//! `<Name>.bin` is an optional self payload (non-entity data) followed by the records' bytes back to back;
//! an index record's position is absolute in the bin. Older indexes (0..=3) have a hash or index per record
//! instead of a type table; they are read for migration and never written.
//!
//! What stays in `muo_server`: turning type names into constructors, building entities, and calling each
//! entity's `deserialize` (use `deserialize_record` for the length check).

use super::reader::GenericReader;
use super::save_reader::{SaveReader, TypesDb};
use super::save_writer::SaveWriter;
use super::writer::GenericWriter;
use crate::error::{IoError, Result, SeekOrigin};
use muo_primitives::Serial;

pub const IDX_VERSION: i32 = 5;
/// Bytes per v4/v5 index record.
pub const IDX_RECORD_SIZE: usize = 26;

/// How a record names its type.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum IdxTypeRef {
    /// Index into the idx's own type table (v4+).
    Table(u16),
    /// XXH3-64 hash of the full name, resolved through `SerializedTypes.db` (v2, v3).
    Hash(u64),
    /// Index into the legacy `<Name>.tdb` name list (v0, v1).
    Legacy(u32),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IdxRecord {
    pub ty: IdxTypeRef,
    pub serial: Serial,
    /// `None` for a v0 index, which stored no creation time (the loader uses "now").
    pub created_ticks: Option<i64>,
    pub position: i64,
    pub length: i32,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Idx {
    pub version: i32,
    /// Save start (v5); 0 before.
    pub anchor_ticks: i64,
    /// Full type names, v4+.
    pub type_table: Vec<String>,
    pub records: Vec<IdxRecord>,
}

/// Elapsed ticks between the save starting (`anchor_ticks`) and now; zero when there is no anchor or it is in the
/// future. This is the `AnchoredTimeShift` every payload of the save is read with.
pub fn anchored_time_shift(anchor_ticks: i64, now_ticks: i64) -> i64 {
    let shift = now_ticks.saturating_sub(anchor_ticks);
    if anchor_ticks > 0 && shift > 0 {
        shift
    } else {
        0
    }
}

/// `GenericEntityPersistence._typeTable`: every entity type added since boot, insertion ordered. Idx records name
/// a type by its slot here, so entries are never removed (a type whose entities were all deleted keeps its slot
/// until restart).
#[derive(Clone, Debug, Default)]
pub struct TypeTable {
    names: Vec<String>,
    indexes: std::collections::HashMap<String, u16>,
}

impl TypeTable {
    pub fn new() -> Self {
        Self::default()
    }

    /// `RegisterType`: the slot of `full_name`, adding it first when new.
    pub fn register(&mut self, full_name: &str) -> Result<u16> {
        if let Some(&i) = self.indexes.get(full_name) {
            return Ok(i);
        }
        let index = u16::try_from(self.names.len()).map_err(|_| {
            IoError::InvalidData(format!(
                "exceeded {} distinct entity types",
                u32::from(u16::MAX) + 1
            ))
        })?;
        self.names.push(full_name.to_owned());
        self.indexes.insert(full_name.to_owned(), index);
        Ok(index)
    }

    /// `TryGetTypeIndex`.
    pub fn index_of(&self, full_name: &str) -> Option<u16> {
        self.indexes.get(full_name).copied()
    }

    pub fn names(&self) -> &[String] {
        &self.names
    }
}

/// Composes an idx and bin like `WriteSnapshot` does.
pub struct EntityFileWriter {
    bin: Vec<u8>,
    idx: SaveWriter,
    count_position: u64,
    count: i32,
    type_count: usize,
}

impl EntityFileWriter {
    /// `self_payload` is the non-entity data (`GenericPersistence.Serialize`), written first to the bin.
    pub fn new(save_start_ticks: i64, type_table: &[String], self_payload: &[u8]) -> Self {
        let mut idx = SaveWriter::with_capacity(20 + type_table.len() * 64, false);
        idx.write_i32(IDX_VERSION);
        idx.write_i64(save_start_ticks);
        idx.write_i32(type_table.len() as i32);
        for name in type_table {
            idx.write_string_raw(name);
        }
        let count_position = idx.position();
        idx.write_i32(0);
        Self {
            bin: self_payload.to_vec(),
            idx,
            count_position,
            count: 0,
            type_count: type_table.len(),
        }
    }

    /// Adds one entity's serialized bytes. Entities that must not be saved (`SkipSerialization`) are simply not added.
    pub fn add_record(
        &mut self,
        type_index: u16,
        serial: Serial,
        created_ticks: i64,
        data: &[u8],
    ) -> Result<()> {
        if usize::from(type_index) >= self.type_count {
            return Err(IoError::InvalidData(format!(
                "type index {type_index} is not in the type table"
            )));
        }
        let length = i32::try_from(data.len())
            .map_err(|_| IoError::InvalidData("record longer than 2 GiB".into()))?;
        self.idx.write_u16(type_index);
        self.idx.write_serial(serial);
        self.idx.write_i64(created_ticks);
        self.idx.write_i64(self.bin.len() as i64);
        self.idx.write_i32(length);
        self.bin.extend_from_slice(data);
        self.count += 1;
        Ok(())
    }

    /// Patches the record count and returns `(idx bytes, bin bytes)`.
    pub fn finish(mut self) -> (Vec<u8>, Vec<u8>) {
        let end = self.idx.position();
        // Both seeks stay inside the written bytes
        let _ = self.idx.seek(self.count_position as i64, SeekOrigin::Begin);
        self.idx.write_i32(self.count);
        let _ = self.idx.seek(end as i64, SeekOrigin::Begin);
        (self.idx.into_vec(), self.bin)
    }
}

/// Reads an idx of any version. Counts are checked against the bytes left, so a corrupt count is an error
/// rather than a huge allocation.
pub fn read_idx(bytes: &[u8]) -> Result<Idx> {
    let mut r = SaveReader::new(bytes);
    let version = r.read_i32()?;
    if !(0..=IDX_VERSION).contains(&version) {
        return Err(IoError::InvalidData(format!(
            "unknown idx version {version}"
        )));
    }

    let mut idx = Idx {
        version,
        ..Idx::default()
    };
    if version >= 5 {
        idx.anchor_ticks = r.read_date_time_ticks()?;
    }

    if version >= 4 {
        let type_count = read_count(&mut r, 1)?;
        for _ in 0..type_count {
            idx.type_table.push(r.read_string_raw()?);
        }
    }

    let count = read_count(&mut r, if version >= 4 { IDX_RECORD_SIZE } else { 20 })?;
    idx.records.reserve(count);
    for _ in 0..count {
        let ty = if version >= 4 {
            let i = r.read_u16()?;
            if usize::from(i) >= idx.type_table.len() {
                return Err(IoError::InvalidData(format!(
                    "type index {i} is not in the type table"
                )));
            }
            IdxTypeRef::Table(i)
        } else if version >= 2 {
            let flag = r.read_u8()?;
            if flag != 2 {
                return Err(IoError::InvalidData(format!(
                    "Invalid type flag, expected 2 but received {flag}."
                )));
            }
            IdxTypeRef::Hash(r.read_u64()?)
        } else {
            IdxTypeRef::Legacy(r.read_u32()?)
        };
        let serial = r.read_serial()?;
        let created_ticks = if version == 0 {
            None
        } else {
            Some(r.read_date_time_ticks()?)
        };
        if (1..3).contains(&version) {
            r.read_i64()?; // LastSerialized
        }
        let position = r.read_i64()?;
        let length = r.read_i32()?;
        idx.records.push(IdxRecord {
            ty,
            serial,
            created_ticks,
            position,
            length,
        });
    }

    Ok(idx)
}

fn read_count(r: &mut SaveReader<'_>, min_item_size: usize) -> Result<usize> {
    let count = r.read_i32()?;
    let n = usize::try_from(count)
        .map_err(|_| IoError::InvalidData(format!("negative count {count}")))?;
    if n.saturating_mul(min_item_size) > r.remaining() {
        return Err(IoError::EndOfStream);
    }
    Ok(n)
}

/// `<Name>.tdb` of old (v0, v1) saves: `i32 count` then raw names; a record's legacy index is its position here.
pub fn read_legacy_tdb(bytes: &[u8]) -> Result<Vec<String>> {
    let mut r = SaveReader::new(bytes);
    let count = read_count(&mut r, 1)?;
    (0..count).map(|_| r.read_string_raw()).collect()
}

/// `SerializedTypes.db`: `i32 version`, `i32 count`, then `u64 hash` + raw name per type.
pub fn read_types_db(bytes: &[u8]) -> Result<TypesDb> {
    let mut r = SaveReader::new(bytes);
    let _version = r.read_i32()?;
    let count = read_count(&mut r, 9)?;
    let mut db = TypesDb::with_capacity(count);
    for _ in 0..count {
        let hash = r.read_u64()?;
        db.insert(hash, r.read_string_raw()?);
    }
    Ok(db)
}

/// Writes the `SerializedTypes.db` layout (the server no longer writes it; for fixtures and migration tools).
pub fn write_types_db(entries: &[(u64, &str)]) -> Vec<u8> {
    let mut w = SaveWriter::new(false);
    w.write_i32(2);
    w.write_i32(entries.len() as i32);
    for (hash, name) in entries {
        w.write_u64(*hash);
        w.write_string_raw(name);
    }
    w.into_vec()
}

/// The bytes of one record, checked against the bin.
pub fn record_bytes<'a>(bin: &'a [u8], record: &IdxRecord) -> Result<&'a [u8]> {
    let start =
        usize::try_from(record.position).map_err(|_| IoError::SeekOutOfRange(record.position))?;
    let len = usize::try_from(record.length)
        .map_err(|_| IoError::InvalidData("negative record length".into()))?;
    start
        .checked_add(len)
        .and_then(|end| bin.get(start..end))
        .ok_or(IoError::SeekOutOfRange(record.position))
}

/// Runs an entity's `deserialize` at its record position and verifies it consumed exactly the record's bytes
/// (`InternalDeserialize`'s "Serialized object was N bytes, but M bytes deserialized"). A zero-length record means
/// the entity was deleted; the caller handles that before calling.
pub fn deserialize_record<R>(
    reader: &mut SaveReader<'_>,
    record: &IdxRecord,
    deserialize: impl FnOnce(&mut SaveReader<'_>) -> Result<R>,
) -> Result<R> {
    reader.seek(record.position, SeekOrigin::Begin)?;
    let start = reader.position();
    let value = deserialize(reader)?;
    let used = reader.position() - start;
    if used != record.length as u64 {
        return Err(IoError::InvalidData(format!(
            "Serialized object was {} bytes, but {used} bytes deserialized",
            record.length
        )));
    }
    Ok(value)
}

/// Whole-file payloads (`GenericPersistence.Deserialize`, `AdhocPersistence.Deserialize`): runs `deserialize` and
/// requires it to have consumed every byte ("Serialized N bytes, but M bytes deserialized").
pub fn deserialize_exact<R>(
    bytes: &[u8],
    types_db: Option<&TypesDb>,
    shift_ticks: i64,
    deserialize: impl FnOnce(&mut SaveReader<'_>) -> Result<R>,
) -> Result<R> {
    let mut r = SaveReader::new(bytes).with_anchored_time_shift_ticks(shift_ticks);
    if let Some(db) = types_db {
        r = r.with_types_db(db);
    }
    let value = deserialize(&mut r)?;
    if r.position() != bytes.len() as u64 {
        return Err(IoError::InvalidData(format!(
            "Serialized {} bytes, but {} bytes deserialized",
            bytes.len(),
            r.position()
        )));
    }
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Literal v5 index: one type, two records.
    #[test]
    fn idx_v5_golden() {
        let types = ["Server.Items.Longsword".to_string()];
        let mut w = EntityFileWriter::new(0x0102, &types, b"SELF");
        w.add_record(0, Serial::new(0x4000_0001), 7, &[0xAA, 0xBB, 0xCC])
            .unwrap();
        w.add_record(0, Serial::new(0x4000_0002), 8, &[0xDD])
            .unwrap();
        let (idx, bin) = w.finish();

        let mut expected = vec![
            5, 0, 0, 0, // version
            0x02, 0x01, 0, 0, 0, 0, 0, 0, // anchor
            1, 0, 0, 0,  // type count
            22, // name length (22 chars: worst case 66 < 128, one byte)
        ];
        expected.extend_from_slice(b"Server.Items.Longsword");
        expected.extend_from_slice(&[2, 0, 0, 0]); // record count
        expected.extend_from_slice(&[
            0, 0, 1, 0, 0, 0x40, 7, 0, 0, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0,
        ]);
        expected.extend_from_slice(&[
            0, 0, 2, 0, 0, 0x40, 8, 0, 0, 0, 0, 0, 0, 0, 7, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0,
        ]);
        assert_eq!(idx, expected);
        assert_eq!(bin, b"SELF\xAA\xBB\xCC\xDD");

        let parsed = read_idx(&idx).unwrap();
        assert_eq!(parsed.version, 5);
        assert_eq!(parsed.anchor_ticks, 0x0102);
        assert_eq!(parsed.type_table, types);
        assert_eq!(parsed.records.len(), 2);
        assert_eq!(parsed.records[1].position, 7);
        assert_eq!(
            record_bytes(&bin, &parsed.records[0]).unwrap(),
            [0xAA, 0xBB, 0xCC]
        );
    }

    #[test]
    fn rejects_bad_idx() {
        assert!(read_idx(&[]).is_err());
        assert!(read_idx(&[9, 0, 0, 0]).is_err());
        // v4 claiming 2 billion records
        let mut b = vec![4, 0, 0, 0, 0, 0, 0, 0, 0xFF, 0xFF, 0xFF, 0x7F];
        assert!(read_idx(&b).is_err());
        // v4 record naming a type outside the table
        b = vec![4, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0];
        b.extend_from_slice(&[
            3, 0, 1, 0, 0, 0x40, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ]);
        assert!(read_idx(&b).is_err());
        let rec = IdxRecord {
            ty: IdxTypeRef::Table(0),
            serial: Serial::new(1),
            created_ticks: None,
            position: 5,
            length: 10,
        };
        assert!(record_bytes(&[0; 8], &rec).is_err());
    }

    #[test]
    fn idx_v3_hash_records() {
        let mut w = SaveWriter::new(false);
        w.write_i32(3);
        w.write_i32(1);
        w.write_u8(2);
        w.write_u64(0xDEAD_BEEF);
        w.write_u32(0x1234);
        w.write_i64(99);
        w.write_i64(0);
        w.write_i32(12);
        let idx = read_idx(w.as_slice()).unwrap();
        assert_eq!(idx.records[0].ty, IdxTypeRef::Hash(0xDEAD_BEEF));
        assert_eq!(idx.records[0].created_ticks, Some(99));
        assert!(idx.type_table.is_empty());
    }

    #[test]
    fn idx_v1_legacy_records() {
        let mut w = SaveWriter::new(false);
        w.write_i32(1);
        w.write_i32(1);
        w.write_i32(5); // tdb index
        w.write_u32(0x77);
        w.write_i64(10); // created
        w.write_i64(11); // last serialized
        w.write_i64(3);
        w.write_i32(4);
        let idx = read_idx(w.as_slice()).unwrap();
        assert_eq!(idx.records[0].ty, IdxTypeRef::Legacy(5));
        assert_eq!((idx.records[0].position, idx.records[0].length), (3, 4));
    }

    #[test]
    fn types_db_round_trip() {
        let bytes = write_types_db(&[(1, "A.B"), (2, "C")]);
        let db = read_types_db(&bytes).unwrap();
        assert_eq!(db[&1], "A.B");
        assert_eq!(db[&2], "C");
        let tdb = {
            let mut w = SaveWriter::new(false);
            w.write_i32(2);
            w.write_string_raw("X");
            w.write_string_raw("Y");
            w.into_vec()
        };
        assert_eq!(read_legacy_tdb(&tdb).unwrap(), ["X", "Y"]);
    }

    #[test]
    fn anchor_shift() {
        assert_eq!(anchored_time_shift(100, 150), 50);
        assert_eq!(anchored_time_shift(0, 150), 0);
        assert_eq!(anchored_time_shift(200, 150), 0);
    }

    #[test]
    fn record_length_is_checked() {
        let bin = [1u8, 2, 3, 4];
        let rec = IdxRecord {
            ty: IdxTypeRef::Table(0),
            serial: Serial::new(1),
            created_ticks: None,
            position: 1,
            length: 2,
        };
        let mut r = SaveReader::new(&bin);
        assert_eq!(
            deserialize_record(&mut r, &rec, |r| r.read_u16()).unwrap(),
            0x0302
        );
        let err = deserialize_record(&mut r, &rec, |r| r.read_u8()).unwrap_err();
        assert!(
            err.to_string()
                .contains("Serialized object was 2 bytes, but 1 bytes deserialized")
        );
    }

    #[test]
    fn exact_payload_is_checked() {
        assert!(deserialize_exact(&[1, 0], None, 0, |r| r.read_u16()).is_ok());
        let err = deserialize_exact(&[1, 0, 0], None, 0, |r| r.read_u16()).unwrap_err();
        assert!(
            err.to_string()
                .contains("Serialized 3 bytes, but 2 bytes deserialized")
        );
    }
}
