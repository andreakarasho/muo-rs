//! `[Decorate` / `[DecorateMag`: every data file of the distribution reads, and the world fills from it.

use std::path::{Path, PathBuf};

use muo_commands::decorate::{generate_folder, read_all};
use muo_server::World;
use muo_server::type_registry::{Phase, invoke_phase};

fn distribution() -> Option<PathBuf> {
    let p = PathBuf::from(std::env::var_os("MUO_DISTRIBUTION")?).join("Data/Decoration");
    p.is_dir().then_some(p)
}

fn world() -> World {
    let mut w = World::new();
    muo::register_content(&mut w.types);
    invoke_phase(&mut w, Phase::Configure);
    for (i, name, width, height) in [
        (0, "Felucca", 7168, 4096),
        (1, "Trammel", 7168, 4096),
        (2, "Ilshenar", 2304, 1600),
        (3, "Malas", 2560, 2048),
        (4, "Tokuno", 1448, 1448),
    ] {
        w.maps.add(muo_server::map::Map::new(
            i,
            i,
            i,
            width,
            height,
            0,
            name,
            Default::default(),
        ));
    }
    w
}

fn cfg_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for e in std::fs::read_dir(dir).unwrap().flatten() {
        let p = e.path();
        if p.is_dir() {
            cfg_files(&p, out);
        } else if p.extension().is_some_and(|x| x == "cfg") {
            out.push(p);
        }
    }
}

/// Every header names a registered class and every file parses; the classes that don't are listed.
#[test]
fn decorate_reads_every_data_file_without_an_unknown_type() {
    let Some(dir) = distribution() else { return };
    let w = world();
    let mut files = Vec::new();
    cfg_files(&dir, &mut files);
    assert!(files.len() >= 100, "{} files", files.len());
    let mut unknown = Vec::new();
    let mut blocks = 0;
    for f in &files {
        let text = std::fs::read_to_string(f).unwrap();
        match read_all(&w, &text) {
            Ok(lists) => blocks += lists.len(),
            Err(e) => unknown.push(format!("{}: {e}", f.display())),
        }
    }
    assert!(blocks > 7000, "{blocks} blocks");
    assert!(
        unknown.is_empty(),
        "{} files with an unknown type:\n{}",
        unknown.len(),
        unknown.join("\n")
    );
}

/// Placing every block: the constructor errors (`Generate() failed to load type`) are the classes `[Decorate`
/// cannot build yet. Writes target/decorate-failures.txt.
#[test]
fn decorate_generates_and_lists_what_it_cannot_build() {
    let Some(dir) = distribution() else { return };
    let mut w = world();
    let mut log = Vec::new();
    let mut count = 0;
    for (folder, maps) in muo_commands::decorate::folders(&w, false) {
        let d = dir
            .parent()
            .unwrap()
            .join(folder.trim_start_matches("Data/"));
        count += generate_folder(&mut w, &d, &maps, false, &mut log).unwrap();
    }
    log.sort();
    log.dedup();
    let out = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/decorate-failures.txt");
    std::fs::write(out, log.join("\n")).unwrap();
    assert!(
        count > 7000,
        "{count} items placed, {} types failed",
        log.len()
    );
}

/// `item.ItemID = m_ItemID` is the container's overriding setter: its cached `ContainerData` is dropped.
#[test]
fn decorate_item_id_goes_through_the_container_setter() {
    let mut w = world();
    let lists = read_all(&w, "Server.Items.Bag 0x9AB\n1 1 0\n").unwrap();
    let id = lists[0]
        .construct(&mut w, false)
        .ok()
        .flatten()
        .expect("a bag");
    assert_eq!(w.item(id).unwrap().item_id, 0x9AB);
    let ty = w.item(id).unwrap().ty;
    (ty.update_container_data)(&mut w, id);
    assert!(
        w.item(id)
            .unwrap()
            .cont
            .as_ref()
            .is_some_and(|c| c.data.is_some())
    );
    muo_server::item::ops::set_item_id(&mut w, id, 0xE76);
    assert!(
        w.item(id)
            .unwrap()
            .cont
            .as_ref()
            .is_none_or(|c| c.data.is_none()),
        "the cached ContainerData was kept"
    );
}

/// `Point3D.Parse`, `TimeSpan.Parse`, `Map.Parse` and `Enum.Parse` of a parameter throw in `Construct()`, inside its
/// `try`: the entry fails to load (`Generate() failed to load type`).
#[test]
fn decorate_bad_parameters_fail_to_load() {
    let mut w = world();
    for text in [
        "Server.Items.Teleporter 0x1FD (PointDest=nonsense)\n1 1 0\n",
        "Server.Items.Teleporter 0x1FD (MapDest=Nowhere)\n1 1 0\n",
        "Server.Items.Teleporter 0x1FD (Delay=soon)\n1 1 0\n",
        "Server.Items.SkillTeleporter 0x1FD (Skill=Cooking2)\n1 1 0\n",
        "Server.Items.RecallRune 0x1FD (Target=1;2)\n1 1 0\n",
        "Cannon 0x1FD (CannonDirection=Up)\n1 1 0\n",
    ] {
        let lists = read_all(&w, text).unwrap();
        assert!(
            matches!(
                lists[0].construct(&mut w, false),
                Err(muo_commands::decorate::ConstructError::Type(_))
            ),
            "{text}"
        );
    }
    let ok = read_all(
        &w,
        "Server.Items.Teleporter 0x1FD (PointDest=(1, 2, 3);MapDest=Felucca)\n1 1 0\n",
    )
    .unwrap();
    assert!(ok[0].construct(&mut w, false).is_ok());
}
