//! Ad hoc persistence: save and load a one-off file from a closure.

use super::index::deserialize_exact;
use super::reader::GenericReader;
use super::save_reader::{SaveReader, TypesDb, read_file_if_exists};
use super::save_writer::SaveWriter;
use super::worker::SaveWorker;
use super::writer::GenericWriter;
use crate::error::Result;
use std::path::Path;

/// `SerializeToBuffer`: composes with prefixed strings into memory.
pub fn serialize_to_buffer(serializer: impl FnOnce(&mut dyn GenericWriter)) -> SaveWriter {
    let mut w = SaveWriter::new(true);
    serializer(&mut w);
    w
}

/// `DeserializeFromBuffer`.
pub fn deserialize_from_buffer<R>(
    buffer: &[u8],
    types_db: Option<&TypesDb>,
    deserializer: impl FnOnce(&mut dyn GenericReader) -> Result<R>,
) -> Result<R> {
    let mut r = SaveReader::new(buffer);
    if let Some(db) = types_db {
        r = r.with_types_db(db);
    }
    deserializer(&mut r)
}

/// `SerializeAndSnapshot`: composes now, on the caller, and has `worker` write the file (creating its folder).
/// Check `worker.take_errors()` after `worker.wait_idle()` for write failures.
pub fn serialize_and_snapshot(
    file_path: &Path,
    worker: &SaveWorker,
    serializer: impl FnOnce(&mut dyn GenericWriter),
) {
    let writer = serialize_to_buffer(serializer);
    let path = file_path.to_path_buf();
    worker.submit(move || {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        writer.write_to_file(&path)
    });
}

/// `Deserialize`: `Ok(None)` when the file is missing or empty, else the closure's result. The closure must consume
/// the whole file; a "Skip this file and continue? (y/n)" prompt is the caller's to make on `Err`.
pub fn deserialize<R>(
    file_path: &Path,
    deserializer: impl FnOnce(&mut dyn GenericReader) -> Result<R>,
) -> Result<Option<R>> {
    match read_file_if_exists(file_path)? {
        Some(bytes) if !bytes.is_empty() => {
            deserialize_exact(&bytes, None, 0, |r| deserializer(r)).map(Some)
        }
        _ => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn buffer_round_trip() {
        let w = serialize_to_buffer(|w| {
            w.write_i32(7);
            w.write_opt_string(Some("hi"));
            w.write_opt_string(None);
        });
        assert_eq!(w.as_slice(), [7, 0, 0, 0, 1, 2, b'h', b'i', 0]);
        let (a, b, c) = deserialize_from_buffer(w.as_slice(), None, |r| {
            Ok((r.read_i32()?, r.read_string()?, r.read_string()?))
        })
        .unwrap();
        assert_eq!((a, b.as_deref(), c), (7, Some("hi"), None));
    }

    #[test]
    fn snapshot_through_worker() {
        let dir = std::env::temp_dir().join(format!("muo_io_adhoc_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let file = dir.join("sub").join("data.bin");

        let worker = SaveWorker::new();
        serialize_and_snapshot(&file, &worker, |w| w.write_u16(0xBEEF));
        worker.wait_idle();
        assert!(worker.take_errors().is_empty());

        assert_eq!(deserialize(&file, |r| r.read_u16()).unwrap(), Some(0xBEEF));
        assert!(
            deserialize(&dir.join("missing.bin"), |r| r.read_u16())
                .unwrap()
                .is_none()
        );

        std::fs::write(&file, [1, 2, 3]).unwrap();
        assert!(deserialize(&file, |r| r.read_u16()).is_err());
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
