//! Real saves: loads a copy of a real save folder with all content registered, saves it again, and compares
//! per-entity record bytes. Prints, by type, records / identical / differing, and the first differing byte
//! (`MUO_GOLDEN_SAVE` overrides the source; `GOLDEN_STRICT=1` fails on any difference or unknown type).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use muo_io::serialization::index::{IdxTypeRef, read_idx, record_bytes};
use muo_server::World;
use muo_server::persistence::{LoadPrompt, PersistError};

/// Answers "y" and remembers each type the loader couldn't find.
struct Collect(Vec<String>, Vec<String>);

impl LoadPrompt for Collect {
    fn write(&mut self, text: &str) {
        if !text.starts_with("Delete the object") {
            self.1.push(text.trim_end().to_string()); // every other message, so a failed record shows why
        }
        if let Some(t) = text.strip_prefix("Error: Type '") {
            self.0.push(t.split('\'').next().unwrap_or("").to_string());
        }
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
                IdxTypeRef::Table(i) => idx.type_table[usize::from(i)].clone(),
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
fn load_and_resave_real_save() {
    let source = PathBuf::from(std::env::var("MUO_GOLDEN_SAVE").unwrap_or_default());
    if !source.exists() {
        eprintln!("golden_saves: {} not found, skipped", source.display());
        return;
    }
    // `MUO_GOLDEN_WORK` names the scratch folder (default target/golden-saves): runs with different sources don't clash.
    let work = std::env::var("MUO_GOLDEN_WORK")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .unwrap()
                .parent()
                .unwrap()
                .join("target")
                .join("golden-saves")
        });
    let _ = std::fs::remove_dir_all(&work);
    let (input, output) = (work.join("in"), work.join("out"));
    copy_dir(&source, &input);

    // A base folder like a run folder: the config, the real Data, and the save copy.
    let base = work.join("base");
    std::fs::create_dir_all(&base).unwrap();
    copy_dir(
        &PathBuf::from(std::env::var_os("MUO_DISTRIBUTION").unwrap_or_default())
            .join("Configuration"),
        &base.join("Configuration"),
    );
    std::fs::rename(&input, base.join("Saves")).unwrap();
    let input = base.join("Saves");
    let data = PathBuf::from(std::env::var_os("MUO_DISTRIBUTION").unwrap_or_default()).join("Data");
    let status = std::process::Command::new("cmd")
        .args(["/C", "mklink", "/J"])
        .arg(base.join("Data"))
        .arg(&data)
        .status()
        .unwrap();
    assert!(status.success());

    let mut w = World::new_unloaded();
    // Anchored times shift by the downtime since the save started: start the clock at the save's own anchor.
    let anchor = read_idx(&std::fs::read(input.join("Items/Items.idx")).unwrap())
        .unwrap()
        .anchor_ticks;
    w.core.fixed_start_now = Some(muo_primitives::DateTime::from_ticks(anchor));
    let mut prompt = Collect(vec![], vec![]);
    muo_server::core_loop::setup_with(
        &mut w,
        base.clone(),
        muo::register_content,
        &mut prompt,
        |_| {},
    )
    .expect("setup");
    w.write_snapshot_to(&output).expect("save");

    let (mut total, mut same, mut diff, mut missing) = (0usize, 0usize, 0usize, 0usize);
    for name in ["Mobiles", "Items"] {
        let before = records(&input, name);
        let after = records(&output, name);
        let mut per: BTreeMap<&str, (usize, usize, usize)> = BTreeMap::new();
        let mut firsts: BTreeMap<&str, String> = BTreeMap::new();
        for (serial, (ty, bytes)) in &before {
            let e = per.entry(ty).or_default();
            e.0 += 1;
            total += 1;
            match after.get(serial) {
                Some((_, new)) if new == bytes => {
                    e.1 += 1;
                    same += 1;
                }
                Some((_, new)) => {
                    e.2 += 1;
                    diff += 1;
                    let at = bytes
                        .iter()
                        .zip(new)
                        .position(|(a, b)| a != b)
                        .unwrap_or(bytes.len().min(new.len()));
                    firsts.entry(ty).or_insert_with(|| {
                        format!(
                            "{serial:#010x} first diff at byte {at} (len {} vs {}): before {:02x?} after {:02x?}",
                            bytes.len(),
                            new.len(),
                            &bytes[at.saturating_sub(4)..(at + 8).min(bytes.len())],
                            &new[at.saturating_sub(4)..(at + 8).min(new.len())]
                        )
                    });
                }
                None => {
                    missing += 1;
                    println!("  NOT WRITTEN BACK {serial:#010x} {ty}");
                }
            }
        }
        println!(
            "{name}: {} records in, {} written back",
            before.len(),
            after.len()
        );
        for (ty, (n, s, d)) in per {
            println!("  {ty}: {n} records, {s} identical, {d} differing");
            if let Some(f) = firsts.get(ty) {
                println!("      {f}");
            }
        }
    }
    // The other persistences: .bin byte for byte (their idx files hold save times).
    let mut other_diff = 0;
    for dir in [
        "Accounts",
        "BOBEntries",
        "ChampionTitles",
        "Disguises",
        "Ethics",
        "Guilds",
        "Jail",
        "PlayerMurders",
        "StealableArtifacts",
        "Virtues",
    ] {
        let (a, b) = (
            input.join(dir).join(format!("{dir}.bin")),
            output.join(dir).join(format!("{dir}.bin")),
        );
        let same = std::fs::read(&a).ok() == std::fs::read(&b).ok();
        other_diff += usize::from(!same);
        println!("{dir}.bin: {}", if same { "identical" } else { "DIFFERS" });
    }
    // `MUO_GOLDEN_REFERENCE`: the same save loaded and saved again by a reference server build. Some records change on
    // a reload (BaseClothing's resource, respawn timers), so a record equal to that one counts as identical too.
    if let Ok(reference) = std::env::var("MUO_GOLDEN_REFERENCE") {
        let mut also = 0;
        for name in ["Mobiles", "Items"] {
            let (before, after, reloaded) = (
                records(&input, name),
                records(&output, name),
                records(Path::new(&reference), name),
            );
            for (serial, (ty, bytes)) in &before {
                if let (Some((_, new)), Some((_, c2))) = (after.get(serial), reloaded.get(serial))
                    && new != bytes
                    && same_but_times(name, new, c2)
                {
                    also += 1;
                    println!("  equal to the reference reload only: {serial:#010x} {ty}");
                }
            }
        }
        println!("{also} differing records equal the reference reload of the save");
        diff = diff.saturating_sub(also);
    }
    println!(
        "TOTAL {total} records, {same} identical, {diff} differing, {missing} not written back"
    );
    println!("unknown types asked about: {:?}", prompt.0);
    for m in prompt
        .1
        .iter()
        .filter(|m| m.contains("Bad deserialize") || m.contains("bytes"))
    {
        println!("LOADER {m}");
    }
    if std::env::var("GOLDEN_STRICT").is_ok_and(|v| v == "1") {
        assert!(other_diff == 0 && diff == 0 && missing == 0 && prompt.0.is_empty());
    }
}

/// Item records hold `LastMoved` (anchored, so it shifts with the run's clock) in bytes 8..16: ignored.
fn same_but_times(name: &str, x: &[u8], y: &[u8]) -> bool {
    if name == "Items" && x.len() == y.len() && x.len() > 16 {
        x[..8] == y[..8] && x[16..] == y[16..]
    } else {
        x == y
    }
}

/// `MUO_SAVE_A`/`MUO_SAVE_B`: compares two save folders (say from two server builds after the same run) by serial:
/// record type, and the first differing byte of the records that are not equal. No server code runs.
#[test]
fn compare_two_saves() {
    let (Ok(a), Ok(b)) = (std::env::var("MUO_SAVE_A"), std::env::var("MUO_SAVE_B")) else {
        return;
    };
    for name in ["Mobiles", "Items"] {
        let (ra, rb) = (records(Path::new(&a), name), records(Path::new(&b), name));
        println!("{name}: {} vs {} records", ra.len(), rb.len());
        if let Some(d) = std::env::var("MUO_DUMP")
            .ok()
            .and_then(|v| u32::from_str_radix(v.trim_start_matches("0x"), 16).ok())
        {
            for (tag, r) in [("A", &ra), ("B", &rb)] {
                if let Some((t, bytes)) = r.get(&d) {
                    println!(
                        "{tag} {t}: {}",
                        bytes
                            .iter()
                            .map(|b| format!("{b:02x}"))
                            .collect::<Vec<_>>()
                            .join(" ")
                    );
                }
            }
        }
        for s in ra
            .keys()
            .chain(rb.keys())
            .collect::<std::collections::BTreeSet<_>>()
        {
            match (ra.get(s), rb.get(s)) {
                (Some((t, x)), Some((u, y))) if t == u && same_but_times(name, x, y) => {}
                (Some((t, x)), Some((u, y))) if t == u => {
                    let skip = if name == "Items" { 8..16 } else { 0..0 };
                    let at = x
                        .iter()
                        .zip(y)
                        .enumerate()
                        .position(|(i, (p, q))| p != q && !skip.contains(&i))
                        .unwrap_or(x.len().min(y.len()));
                    println!(
                        "  {s:#010x} {t}: differs at byte {at} (len {} vs {}): {:02x?} vs {:02x?}",
                        x.len(),
                        y.len(),
                        &x[at.saturating_sub(2)..(at + 6).min(x.len())],
                        &y[at.saturating_sub(2)..(at + 6).min(y.len())]
                    );
                }
                (Some((t, _)), Some((u, _))) => println!("  {s:#010x} type {t} vs {u}"),
                (Some((t, _)), None) => println!("  {s:#010x} only in A: {t}"),
                (None, Some((t, _))) => println!("  {s:#010x} only in B: {t}"),
                (None, None) => {}
            }
        }
    }
}
