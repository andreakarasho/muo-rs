//! Loads a copy of a reference save (set its folder with `MUO_GOLDEN_SAVE`),
//! writes it back, and compares per-entity record bytes for the entity types the registry knows.
//!
//! The original is only read: it is copied under `target/golden/`. Unknown types are answered "y" (delete) at the
//! load prompt, so only registered classes survive the round trip. The report lists, per type name, how many
//! records the save has, whether the class is registered, and how many records came back byte-identical.
//! Set `GOLDEN_STRICT=1` to fail on any registered type whose bytes differ (off by default while the item and
//! mobile serializers are being written).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use muo_io::serialization::index::{read_idx, record_bytes};
use muo_server::World;
use muo_server::persistence::{LoadPrompt, PersistError};

struct AlwaysYes(String);

impl LoadPrompt for AlwaysYes {
    fn write(&mut self, text: &str) {
        self.0.push_str(text);
    }
    fn read_line(&mut self) -> Result<String, PersistError> {
        Ok("y".into())
    }
}

fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for e in std::fs::read_dir(from).unwrap() {
        let e = e.unwrap();
        if e.file_type().unwrap().is_dir() {
            copy_dir(&e.path(), &to.join(e.file_name()));
        } else {
            std::fs::copy(e.path(), to.join(e.file_name())).unwrap();
        }
    }
}

/// serial -> (type name, record bytes) of an entity persistence in a save folder.
fn records(save: &Path, name: &str) -> BTreeMap<u32, (String, Vec<u8>)> {
    let dir = save.join(name);
    let (Ok(idx), Ok(bin)) = (
        std::fs::read(dir.join(format!("{name}.idx"))),
        std::fs::read(dir.join(format!("{name}.bin"))),
    ) else {
        return BTreeMap::new();
    };
    let idx = read_idx(&idx).unwrap();
    idx.records
        .iter()
        .map(|r| {
            let ty = match r.ty {
                muo_io::serialization::index::IdxTypeRef::Table(i) => {
                    idx.type_table[usize::from(i)].clone()
                }
                _ => "?".into(),
            };
            (
                r.serial.value(),
                (ty, record_bytes(&bin, r).unwrap().to_vec()),
            )
        })
        .collect()
}

#[test]
fn load_and_resave_original_save() {
    let source = PathBuf::from(std::env::var("MUO_GOLDEN_SAVE").unwrap_or_default());
    if !source.exists() {
        eprintln!("golden: {} not found, skipped", source.display());
        return;
    }
    let work = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/golden");
    let _ = std::fs::remove_dir_all(&work);
    let (input, output) = (work.join("in"), work.join("out"));
    copy_dir(&source, &input);

    let mut w = World::new_unloaded();
    w.save_path = input.clone();
    w.now = muo_primitives::DateTime::utc_now();
    let mut prompt = AlwaysYes(String::new());
    w.load(&mut prompt).expect("load");
    w.write_snapshot_to(&output).expect("save");

    let strict = std::env::var("GOLDEN_STRICT").is_ok_and(|v| v == "1");
    let mut failures = Vec::new();
    for name in ["Mobiles", "Items"] {
        let before = records(&input, name);
        let after = records(&output, name);
        let mut per_type: BTreeMap<&str, (usize, usize, usize)> = BTreeMap::new();
        for (serial, (ty, bytes)) in &before {
            let e = per_type.entry(ty).or_default();
            e.0 += 1;
            if let Some((_, new)) = after.get(serial) {
                e.1 += 1;
                if new == bytes {
                    e.2 += 1;
                } else {
                    failures.push(format!("{name} {serial:#010x} {ty}"));
                }
            }
        }
        println!(
            "{name}: {} records in, {} written back",
            before.len(),
            after.len()
        );
        for (ty, (total, kept, same)) in per_type {
            println!("  {ty}: {total} in, {kept} kept, {same} identical");
        }
    }
    println!("prompts answered: {}", prompt.0.matches("(y/n").count());
    if strict {
        assert!(failures.is_empty(), "records differ: {failures:?}");
    }
}
