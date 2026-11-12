//! The entity-independent parts of persistence:
//! the registry order, the save file paths, and the whole-save load order.
//!
//! The server owns the `Persistence` objects themselves (they hold entities); it stores them in a
//! `PersistenceRegistry<Box<dyn Persistence>>` and drives the phases below.
//!
//! Save (game thread composes, `SaveWorker` writes):
//! 1. `registry.serialization_order(..)`: biggest payloads first (`SerializeAll`'s LPT scheduling).
//! 2. each persistence serializes its self payload and its entities into byte buffers, then
//!    `EntityFileWriter` (entities) or `write_generic_file` (no entities) builds the files.
//! 3. `write_entity_files` on the worker, then `PostWorldSave` on every persistence in registry order.
//!
//! Load (`Persistence.Load`):
//! 1. `load_types_db(save)`.
//! 2. in registry order, every entity persistence reads its idx (`read_idx`) and creates empty entities.
//! 3. in registry order, every persistence reads its bin: mobiles before items is a priority, not a rule here.
//!    Each record goes through `deserialize_record`; a failure is the server's prompt-or-abort decision, since
//!    console questions ("Delete the object and continue? (y/n/a)") do not belong in a library.
//! 4. `PostDeserialize` on every persistence in registry order.

use super::index::{EntityFileWriter, Idx, read_idx, read_types_db};
use super::save_reader::{TypesDb, read_file_if_exists};
use crate::error::Result;
use std::path::{Path, PathBuf};

/// Handle returned by `register`, used to `unregister`.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct PersistenceId(u64);

struct Entry<T> {
    id: PersistenceId,
    priority: i32,
    value: T,
}

/// The `Persistence._registry` sorted set: ordered by priority, ties by registration order.
pub struct PersistenceRegistry<T> {
    entries: Vec<Entry<T>>,
    next_id: u64,
}

impl<T> Default for PersistenceRegistry<T> {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            next_id: 0,
        }
    }
}

impl<T> PersistenceRegistry<T> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, priority: i32, value: T) -> PersistenceId {
        let id = PersistenceId(self.next_id);
        self.next_id += 1;
        let at = self.entries.partition_point(|e| e.priority <= priority);
        self.entries.insert(
            at,
            Entry {
                id,
                priority,
                value,
            },
        );
        id
    }

    pub fn unregister(&mut self, id: PersistenceId) -> Option<T> {
        let at = self.entries.iter().position(|e| e.id == id)?;
        Some(self.entries.remove(at).value)
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Registry (priority) order.
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.entries.iter().map(|e| &e.value)
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.entries.iter_mut().map(|e| &mut e.value)
    }

    /// `SerializeAll`'s order: largest estimated payload first (an indivisible multi-megabyte system serialized
    /// last would extend the freeze by its full duration), registry order among equals.
    pub fn serialization_order(&self, estimated_size: impl Fn(&T) -> u64) -> Vec<&T> {
        let mut v: Vec<&T> = self.iter().collect();
        v.sort_by_key(|p| std::cmp::Reverse(estimated_size(p)));
        v
    }
}

/// `<save>/SerializedTypes.db`.
pub fn types_db_path(save: &Path) -> PathBuf {
    save.join("SerializedTypes.db")
}

/// `<save>/<name>/<name>.idx`.
pub fn entity_idx_path(save: &Path, name: &str) -> PathBuf {
    save.join(name).join(format!("{name}.idx"))
}

/// `<save>/<name>/<name>.bin`: the entities' (and, for non-entity persistences, the only) data file.
pub fn bin_path(save: &Path, name: &str) -> PathBuf {
    save.join(name).join(format!("{name}.bin"))
}

/// Legacy split files, `<name>_<i>.idx`.
pub fn split_idx_path(save: &Path, name: &str, i: usize) -> PathBuf {
    save.join(name).join(format!("{name}_{i}.idx"))
}

/// Legacy split files, `<name>_<i>.bin`.
pub fn split_bin_path(save: &Path, name: &str, i: usize) -> PathBuf {
    save.join(name).join(format!("{name}_{i}.bin"))
}

/// Legacy `<name>.tdb` beside the idx (idx versions 0 and 1).
pub fn legacy_tdb_path(save: &Path, name: &str) -> PathBuf {
    save.join(name).join(format!("{name}.tdb"))
}

/// `Persistence.LoadTypes`: an absent file is an empty table.
pub fn load_types_db(save: &Path) -> Result<TypesDb> {
    Ok(match read_file_if_exists(&types_db_path(save))? {
        Some(bytes) => read_types_db(&bytes)?,
        None => TypesDb::new(),
    })
}

/// `DeserializeIndexes` file discovery: the idx of `name`, or the legacy split idx files in order
/// (`<name>_0.idx`, `<name>_1.idx`, ... stopping at the first missing one; empty files are skipped).
/// Returns `(index into the split list, idx)`; a single idx is index 0.
pub fn load_indexes(save: &Path, name: &str) -> Result<Vec<(usize, Idx)>> {
    if let Some(bytes) = read_file_if_exists(&entity_idx_path(save, name))? {
        return Ok(vec![(0, read_idx(&bytes)?)]);
    }

    let mut out = Vec::new();
    let mut i = 0;
    while let Some(bytes) = read_file_if_exists(&split_idx_path(save, name, i))? {
        if !bytes.is_empty() {
            out.push((i, read_idx(&bytes)?));
        }
        i += 1;
    }
    Ok(out)
}

/// `WriteSnapshot` of an entity persistence: creates `<save>/<name>/` and writes the idx and bin.
pub fn write_entity_files(save: &Path, name: &str, idx: &[u8], bin: &[u8]) -> Result<()> {
    std::fs::create_dir_all(save.join(name))?;
    std::fs::write(bin_path(save, name), bin)?;
    std::fs::write(entity_idx_path(save, name), idx)?;
    Ok(())
}

/// `GenericPersistence.WriteSnapshot`: a persistence with no payload writes nothing.
pub fn write_generic_file(save: &Path, name: &str, payload: &[u8]) -> Result<()> {
    if payload.is_empty() {
        return Ok(());
    }
    std::fs::create_dir_all(save.join(name))?;
    std::fs::write(bin_path(save, name), payload)?;
    Ok(())
}

/// Convenience over `EntityFileWriter` + `write_entity_files`.
pub fn write_entity_snapshot(save: &Path, name: &str, writer: EntityFileWriter) -> Result<()> {
    let (idx, bin) = writer.finish();
    write_entity_files(save, name, &idx, &bin)
}

#[cfg(test)]
mod tests {
    use super::*;
    use muo_primitives::Serial;

    #[test]
    fn registry_orders_by_priority_then_registration() {
        let mut r = PersistenceRegistry::new();
        r.register(100, "items");
        let mobiles = r.register(10, "mobiles");
        r.register(100, "guilds");
        r.register(1, "accounts");
        assert_eq!(
            r.iter().copied().collect::<Vec<_>>(),
            ["accounts", "mobiles", "items", "guilds"]
        );
        assert_eq!(r.unregister(mobiles), Some("mobiles"));
        assert_eq!(r.len(), 3);

        let sizes = |p: &&str| match *p {
            "guilds" => 50,
            "items" => 50,
            _ => 1,
        };
        assert_eq!(
            r.serialization_order(|p| sizes(p)),
            [&"items", &"guilds", &"accounts"]
        );
    }

    #[test]
    fn files_round_trip_on_disk() {
        let dir = std::env::temp_dir().join(format!("muo_io_persist_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);

        let mut w = EntityFileWriter::new(5, &["T".to_string()], &[]);
        w.add_record(0, Serial::new(9), 1, &[1, 2]).unwrap();
        write_entity_snapshot(&dir, "Items", w).unwrap();

        let loaded = load_indexes(&dir, "Items").unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].1.records[0].serial, Serial::new(9));
        assert_eq!(std::fs::read(bin_path(&dir, "Items")).unwrap(), [1, 2]);
        assert!(load_indexes(&dir, "Mobiles").unwrap().is_empty());
        assert!(load_types_db(&dir).unwrap().is_empty());

        write_generic_file(&dir, "Empty", &[]).unwrap();
        assert!(!dir.join("Empty").exists());
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
