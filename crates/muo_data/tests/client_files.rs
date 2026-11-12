//! Reads against a real UO client directory. Set `MUO_CLIENT_DIR` (the `dataDirectories` entry of
//! Configuration/muo.json) and run with `--ignored`.

use muo_data::client::{ArtData, MultiData};
use muo_data::configuration::{ServerConfiguration, ServerSettings};
use muo_data::localization::Localization;
use muo_data::{ClientVersion, DataDirectories, TileData, TileMatrix, TileMatrixOptions};

fn dirs() -> DataDirectories {
    DataDirectories::new([std::env::var("MUO_CLIENT_DIR").expect("MUO_CLIENT_DIR")])
}

#[test]
#[ignore = "needs the UO client files (set MUO_CLIENT_DIR)"]
fn tiledata_loads() {
    let td = TileData::load(&dirs()).unwrap();
    assert!(td.items.iter().any(|i| !i.name.is_empty() && i.height > 0));
    assert!(td.land.iter().any(|l| !l.name.is_empty()));
}

#[test]
#[ignore = "needs the UO client files (set MUO_CLIENT_DIR)"]
fn felucca_reads_land_and_statics() {
    let dirs = dirs();
    let mut m = TileMatrix::open(&dirs, 0, 0, 7168, 4096, &TileMatrixOptions::default()).unwrap();
    assert_ne!(m.map_files_fingerprint(), 0);
    assert_eq!((m.block_width(), m.block_height()), (896, 512));
    assert_eq!(m.get_land_block(175, 175).len(), 64);
    let mut any_static = false;
    for bx in 160..200 {
        for by in 160..200 {
            any_static |= m
                .get_static_block(bx, by)
                .iter()
                .flatten()
                .any(|c| !c.is_empty());
        }
    }
    assert!(any_static, "Britain's surroundings have statics");
}

#[test]
#[ignore = "needs the UO client files (set MUO_CLIENT_DIR)"]
fn multis_load() {
    let cfg = ServerConfiguration::in_memory(ServerSettings::default());
    let m = MultiData::load(&dirs(), &cfg, Some(&ClientVersion::v7090())).unwrap();
    assert!(!m.get_components(0x74).list.is_empty());
}

#[test]
#[ignore = "needs the UO client files (set MUO_CLIENT_DIR)"]
fn art_bounds() {
    let mut art = ArtData::open(&dirs()).unwrap();
    assert!(art.is_initialized());
    let (w, h, bounds) = art.get_static_bounds(0xEED).unwrap();
    assert!(w > 0 && h > 0 && bounds.width() > 0);
}

#[test]
#[ignore = "needs the UO client files (set MUO_CLIENT_DIR)"]
fn cliloc_loads() {
    let mut l = Localization::new(dirs());
    let t = l.get_text(1005000, None).unwrap();
    assert!(t.is_some_and(|t| !t.is_empty()));
}
