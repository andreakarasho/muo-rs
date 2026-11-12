//! Spawner tests: construct, round-trip the save bytes, load the older versions, read the JSON data format.

use muo_io::{GenericWriter, SaveReader, SaveWriter};
use muo_primitives::{Point3D, Rectangle3D, TimeSpan};
use muo_server::World;
use muo_server::ids::ItemId;
use muo_server::item::{Item, ItemType, ops};

use super::core::{self, PROXIMITY_SPAWNER, REGION_SPAWNER, SPAWNER};
use super::{Kind, SpawnPositionMode, data};

fn save(w: &World, id: ItemId) -> Vec<u8> {
    let ty = w.item(id).unwrap().ty;
    let mut out = SaveWriter::new(true);
    (ty.serialize)(w, id, &mut out);
    out.into_vec()
}

fn load(w: &mut World, ty: &'static ItemType, bytes: &[u8]) -> ItemId {
    let serial = w.next_item_serial();
    let id = w.add_item(Item::new_for_load(serial, ty));
    let mut r = SaveReader::new(bytes);
    (ty.deserialize)(w, id, &mut r).expect("loads");
    id
}

#[test]
fn names_are_unique_and_register() {
    let mut types = muo_server::type_registry::TypeRegistry::default();
    super::register(&mut types);
    assert!(
        types
            .find_type_by_name("ProximitySpawner", false, true)
            .is_some()
    );
    assert_eq!(SPAWNER.full_name, "Server.Engines.Spawners.Spawner");
}

#[test]
fn a_new_spawner_has_the_constructor_state() {
    let mut w = World::new();
    let id = (SPAWNER.construct.unwrap())(&mut w);
    let d = data(&w, id).unwrap();
    assert!(d.running && !d.group);
    assert_eq!(d.count, 1);
    assert_eq!(d.walking_range, -1);
    assert_eq!(d.max_spawn_attempts, 0);
    // HomeRange 4 around the origin it was built at
    assert_eq!(d.spawn_bounds, Rectangle3D::new(-4, -4, -128, 9, 9, 256));
    assert!(!w.item(id).unwrap().visible() && !w.item(id).unwrap().movable());
    assert_eq!(core::home_range(&w, id), 4);
}

#[test]
fn every_class_round_trips() {
    for (ty, kind) in [
        (&SPAWNER, Kind::Spawner),
        (&REGION_SPAWNER, Kind::Region),
        (&PROXIMITY_SPAWNER, Kind::Proximity),
    ] {
        let mut w = World::new();
        let id = (ty.construct.unwrap())(&mut w);
        core::add_entry(&mut w, id, Some("Orc"), 50, 3, false, None, None);
        core::add_entry(
            &mut w,
            id,
            Some("Troll"),
            100,
            1,
            false,
            Some("Hue=5"),
            None,
        );
        if let Some(e) = super::data_mut(&mut w, id).and_then(|d| d.entries.get_mut(1)) {
            e.disabled = true;
        }
        core::set_walking_range(&mut w, id, 7);
        if let Some(d) = super::data_mut(&mut w, id) {
            d.team = 2;
            d.group = true;
            d.min_delay = TimeSpan::from_mins(1);
            d.spawn_position_mode = SpawnPositionMode::Enabled;
            d.trigger_range = 3;
            d.instant_flag = true;
        }
        let bytes = save(&w, id);

        let mut w2 = World::new();
        let id2 = load(&mut w2, ty, &bytes);
        let (a, b) = (data(&w, id).unwrap(), data(&w2, id2).unwrap());
        assert_eq!(b.kind, kind);
        assert_eq!(a.guid, b.guid);
        assert_eq!(
            (a.walking_range, a.team, a.group, a.count),
            (b.walking_range, b.team, b.group, b.count)
        );
        assert_eq!(
            (a.min_delay, a.max_delay, a.spawn_position_mode),
            (b.min_delay, b.max_delay, b.spawn_position_mode)
        );
        assert_eq!(a.spawn_bounds, b.spawn_bounds);
        assert_eq!(b.entries.len(), 2);
        assert_eq!(b.entries[0].name.as_deref(), Some("Orc"));
        assert!(b.entries[1].disabled);
        assert_eq!(b.entries[1].properties.as_deref(), Some("Hue=5"));
        if kind == Kind::Proximity {
            assert_eq!((b.trigger_range, b.instant_flag), (3, true));
        }
    }
}

/// A version 12 BaseSpawner section with a version 1 Spawner section (entries inside the base part).
#[test]
fn a_version_12_save_loads() {
    let mut w = World::new();
    let proto = (SPAWNER.construct.unwrap())(&mut w);
    // Item part from a real spawner, then the spawner part by hand.
    let mut out = SaveWriter::new(true);
    (muo_server::item::ITEM_BASE.serialize)(&w, proto, &mut out);
    out.write_i32(12);
    out.write_enum(4, 0x8 | 0x20); // MinDelay, Team
    out.write_guid(muo_io::serialization::Guid([7; 16]));
    out.write_encoded_int(1); // entries
    out.write_i32(1); // entry version 1
    out.write_opt_string(Some("Mongbat"));
    out.write_i32(100);
    out.write_i32(4);
    out.write_opt_string(None);
    out.write_opt_string(None);
    out.write_encoded_int(0);
    out.write_i32(-1); // walking range
    out.write_time_span_ticks(TimeSpan::from_mins(2).ticks());
    out.write_i32(3); // count
    out.write_i32(9); // team
    out.write_bool(false); // running
    out.write_encoded_int(1); // Spawner v1
    out.write_enum(4, 0x2);
    out.write_rect3d(Rectangle3D::new(10, 20, -128, 5, 5, 256));
    let bytes = out.into_vec();

    let id = load(&mut w, &SPAWNER, &bytes);
    let d = data(&w, id).unwrap();
    assert_eq!(d.guid, muo_io::serialization::Guid([7; 16]));
    assert_eq!((d.count, d.team, d.running), (3, 9, false));
    assert_eq!(d.min_delay, TimeSpan::from_mins(2));
    assert_eq!(d.max_delay, super::DEFAULT_MAX_DELAY);
    assert_eq!(d.entries.len(), 1);
    assert_eq!(
        (d.entries[0].name.as_deref(), d.entries[0].max_count),
        (Some("Mongbat"), 4)
    );
    assert_eq!(d.spawn_bounds, Rectangle3D::new(10, 20, -128, 5, 5, 256));
    assert_eq!(d.max_spawn_attempts, super::DEFAULT_MAX_SPAWN_ATTEMPTS);
}

#[test]
fn the_json_data_format_reads_and_builds() {
    let w = World::new();
    let v: serde_json::Value = serde_json::from_str(
        r#"{ "$type": "Spawner", "guid": "3df0543a-373c-4673-a98b-8191686f4ab3", "name": "Spawner (225)",
             "location": [4571, 2303, -2], "map": "Felucca", "count": 2, "minDelay": "00:05:00",
             "maxDelay": "00:10:00", "team": 0, "homeRange": 10, "walkingRange": -1,
             "entries": [ { "name": "Fisherman", "maxCount": 1, "probability": 100 }, { "name": "Bard" } ] }"#,
    )
    .unwrap();
    let dto = super::json::read_dto(&w, &v).unwrap();
    assert_eq!(dto.kind, Kind::Spawner);
    assert_eq!(dto.location, Point3D::new(4571, 2303, -2));
    assert_eq!(dto.count, 2);
    assert_eq!(dto.home_range, 10);
    assert_eq!(dto.min_delay, TimeSpan::from_mins(5));
    assert_eq!(dto.entries[1].probability, 100);
    assert_eq!(dto.entries[1].max_count, 1);
    assert_eq!(dto.guid.to_string(), "3df0543a-373c-4673-a98b-8191686f4ab3");
    assert!(super::json::read_dto(&w, &serde_json::json!({ "$type": "Nope" })).is_err());

    let mut w = World::new();
    let id = super::json::to_spawner(&mut w, &dto);
    let d = data(&w, id).unwrap();
    assert_eq!(
        d.spawn_bounds,
        core::bounds_from_home_range(dto.location, 10)
    );
    assert_eq!(d.entries.len(), 2);
    assert_eq!(ops::name(&w, id).as_deref(), Some("Spawner (225)"));
}

#[test]
fn the_spiral_offsets_walk_the_ring() {
    assert_eq!(super::cache::spiral_offset(1, 0), (-1, -1));
    assert_eq!(super::cache::spiral_offset(1, 2), (1, -1));
    assert_eq!(super::cache::spiral_offset(1, 4), (1, 1));
    assert_eq!(super::cache::spiral_offset(1, 6), (-1, 1));
}

#[test]
fn glob_matches_nested_json() {
    let dir = std::env::temp_dir().join(format!("muo_glob_{}", std::process::id()));
    std::fs::create_dir_all(dir.join("Data/Spawns/a")).unwrap();
    std::fs::write(dir.join("Data/Spawns/a/x.json"), "[]").unwrap();
    std::fs::write(dir.join("Data/Spawns/y.json"), "[]").unwrap();
    std::fs::write(dir.join("Data/Spawns/z.txt"), "").unwrap();
    assert_eq!(
        super::json::glob_files(&dir, "Data/Spawns/**/*.json").len(),
        2
    );
    assert_eq!(super::json::glob_files(&dir, "Data/Spawns/*.json").len(), 1);
    std::fs::remove_dir_all(&dir).ok();
}

// ---- the distribution data, export and the argument parsers ----------------------------------------------------------

/// `Data/Spawns` of the distribution in `MUO_DISTRIBUTION` (read only); the tests below skip when it is not next door.
fn distribution_spawns() -> Option<std::path::PathBuf> {
    let p = std::path::PathBuf::from(std::env::var_os("MUO_DISTRIBUTION")?).join("Data/Spawns");
    p.is_dir().then_some(p)
}

/// A world with the six standard maps (`Data/map-definitions.json`).
fn world_with_maps() -> World {
    let mut w = World::new();
    for (i, name, width, height) in [
        (0, "Felucca", 7168, 4096),
        (1, "Trammel", 7168, 4096),
        (2, "Ilshenar", 2304, 1600),
        (3, "Malas", 2560, 2048),
        (4, "Tokuno", 1448, 1448),
        (5, "TerMur", 1280, 4096),
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

#[test]
fn every_distribution_spawn_file_reads() {
    let Some(dir) = distribution_spawns() else {
        return;
    };
    let w = world_with_maps();
    let files = super::json::glob_files(&dir, "**/*.json");
    assert!(files.len() >= 109, "found {} files", files.len());
    let mut spawners = 0;
    for f in &files {
        let text = std::fs::read_to_string(f).unwrap();
        let v: serde_json::Value = serde_json::from_str(text.trim_start_matches('\u{feff}'))
            .unwrap_or_else(|e| panic!("{}: {e}", f.display()));
        let serde_json::Value::Array(a) = v else {
            panic!("{} is not an array", f.display())
        };
        for (i, e) in a.iter().enumerate() {
            let dto = super::json::read_dto(&w, e)
                .unwrap_or_else(|e2| panic!("{} #{i}: {e2}", f.display()));
            assert!(
                dto.map
                    .is_some_and(|m| m != muo_server::map::MapId::INTERNAL),
                "{} #{i}: no map",
                f.display()
            );
            spawners += 1;
        }
    }
    assert!(spawners > 5000, "only {spawners} spawners");
}

#[test]
fn every_distribution_spawn_file_imports() {
    let Some(dir) = distribution_spawns() else {
        return;
    };
    let mut w = world_with_maps();
    let mut all = indexmap::IndexMap::new();
    let (mut generated, mut failures) = (0, 0);
    for f in super::json::glob_files(&dir, "**/*.json") {
        let (g, x) = super::json::import_json(&mut w, None, &f, &mut all);
        generated += g;
        failures += x;
    }
    assert_eq!(failures, 0);
    assert!(generated > 5000, "only {generated} spawners");
}

#[test]
fn export_writes_what_the_importer_reads() {
    let mut w = world_with_maps();
    let id = (SPAWNER.construct.unwrap())(&mut w);
    ops::set_name(&mut w, id, Some("Spawner (225)".to_owned()));
    ops::move_to_world_map(
        &mut w,
        id,
        Point3D::new(100, 200, 5),
        Some(muo_server::map::MapId::FELUCCA),
    );
    core::add_entry(&mut w, id, Some("Orc"), 100, 2, false, Some("Hue 5"), None);
    let dto = super::export::to_dto(&w, id).unwrap();
    assert_eq!(dto.home_range, 4);
    assert_eq!(dto.spawn_bounds, Rectangle3D::default());
    let text = super::export::serialize_compact(&w, std::slice::from_ref(&dto));
    assert!(
        text.starts_with("[\n  {\n    \"$type\": \"Spawner\",\n    \"guid\": \""),
        "{text}"
    );
    assert!(
        text.contains("\"location\": [100, 200, 5],")
            && text.contains("\"minDelay\": \"00:05:00\","),
        "{text}"
    );
    assert!(text.contains("{ \"name\": \"Orc\", \"probability\": 100, \"maxCount\": 2, \"properties\": \"Hue 5\" }"), "{text}");
    let v: serde_json::Value = serde_json::from_str(&text).unwrap();
    let back = super::json::read_dto(&w, &v[0]).unwrap();
    assert_eq!(
        (back.guid, back.count, back.home_range, back.entries.len()),
        (dto.guid, dto.count, 4, 1)
    );
    assert_eq!(back.entries[0].properties.as_deref(), Some("Hue 5"));
}

#[test]
fn the_xml_importer_reads_runuo_spawners() {
    let mut w = world_with_maps();
    let dir = std::env::temp_dir().join(format!("muo_spawn_xml_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let p = dir.join("s.xml");
    std::fs::write(
        &p,
        "<spawners><spawner><name>Test</name><location>(10, 20, 0)</location><map>Felucca</map><count>3</count>\
         <homerange>2</homerange><creaturesname><creaturename>Orc</creaturename><creaturename>Ogre</creaturename></creaturesname></spawner></spawners>",
    )
    .unwrap();
    let mut all = indexmap::IndexMap::new();
    let (g, f) = super::export::import_xml(&mut w, None, &p, &mut all);
    assert_eq!((g, f), (1, 0));
    let id = *all.values().next().unwrap();
    assert_eq!(data(&w, id).unwrap().entries.len(), 2);
    assert_eq!(data(&w, id).unwrap().count, 3);
    assert_eq!(core::home_range(&w, id), 2);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn entry_properties_and_parameters_pick_a_constructor_and_set_properties() {
    use super::spawn_args::{apply, construct, resolve_props};
    let mut w = World::new();
    muo_content::props::register_core(&mut w);
    let mut types = muo_server::type_registry::TypeRegistry::default();
    super::register(&mut types);
    w.types = types;
    let ty = w.types.find_type_by_name("Spawner", false, true).unwrap();
    // an unknown or read-only property names no property
    assert!(resolve_props(&w, ty, Some("NoSuchThing 1")).is_none());
    // `name value` pairs
    let assignments =
        resolve_props(&w, ty, Some("Name Hello")).expect("Name is a CommandProperty of Item");
    let e = construct(&mut w, ty, None).expect("parameterless");
    assert!(apply(&mut w, e, &assignments));
    let muo_server::ids::EntityId::Item(i) = e else {
        panic!()
    };
    assert_eq!(ops::name(&w, i).as_deref(), Some("Hello"));
    // Spawner has no constructor with one parameter
    assert!(construct(&mut w, ty, Some("5")).is_none());
}

// ---- SectorSpawnCache -------------------------------------------------------------------------------------

mod sector_spawn_cache {
    use super::super::cache::{self, SectorSpawnCaches};
    use super::super::{PosState, SpawnPositionMode};
    use muo_primitives::{BitMask256, Point3D, Rectangle2D, Rectangle3D};
    use muo_server::World;
    use muo_server::map::MapId;
    const FELUCCA: MapId = MapId::FELUCCA;
    const INTERNAL: MapId = MapId::INTERNAL;
    use std::collections::HashSet;

    #[test]
    fn bit_mask_256_set_bit_sets_correct_bit() {
        for bit in [0, 63, 64, 127, 128, 191, 192, 255] {
            let mut mask = BitMask256::ALL_CLEAR;
            mask.set_bit(bit);
            assert!(mask.get_bit(bit));
            assert_eq!(mask.pop_count(), 1);
        }
    }

    #[test]
    fn bit_mask_256_set_multiple_bits_counts_correctly() {
        let mut mask = BitMask256::ALL_CLEAR;
        for b in [0, 64, 128, 192] {
            mask.set_bit(b);
        }
        assert_eq!(mask.pop_count(), 4);
        assert!(mask.get_bit(0) && mask.get_bit(64) && mask.get_bit(128) && mask.get_bit(192));
        assert!(!mask.get_bit(1));
    }

    #[test]
    fn bit_mask_256_clear_bit_clears_correct_bit() {
        let mut mask = BitMask256::ALL_CLEAR;
        mask.set_bit(50);
        mask.set_bit(100);
        assert_eq!(mask.pop_count(), 2);
        mask.clear_bit(50);
        assert!(!mask.get_bit(50));
        assert!(mask.get_bit(100));
        assert_eq!(mask.pop_count(), 1);
    }

    #[test]
    fn bit_mask_256_get_nth_set_bit_first_bit_in_each_ulong() {
        let mut mask = BitMask256::ALL_CLEAR;
        for b in [0, 64, 128, 192] {
            mask.set_bit(b);
        }
        for (n, expected) in [(0, 0), (1, 64), (2, 128), (3, 192)] {
            assert_eq!(mask.nth_set_bit(n), expected);
        }
    }

    #[test]
    fn bit_mask_256_get_nth_set_bit_within_single_ulong() {
        let mut mask = BitMask256::ALL_CLEAR;
        for b in [5, 10, 20] {
            mask.set_bit(b);
        }
        assert_eq!(mask.nth_set_bit(0), 5);
        assert_eq!(mask.nth_set_bit(1), 10);
        assert_eq!(mask.nth_set_bit(2), 20);
    }

    fn counts(w: &mut World) -> (usize, usize, usize) {
        let c = w.resources.get_mut::<SectorSpawnCaches>();
        (
            c.cached_sector_count(),
            c.land_cache_count(),
            c.water_cache_count(),
        )
    }

    fn pos(w: &mut World, x: i32, y: i32, water: bool) {
        cache::set_valid(w, FELUCCA, Point3D::new(x, y, 0), water);
    }

    #[test]
    fn set_valid_land_and_water_separate_caches() {
        let mut w = World::new();
        pos(&mut w, 100, 100, false);
        pos(&mut w, 105, 105, true);
        assert_eq!(counts(&mut w), (2, 1, 1));
    }

    #[test]
    fn set_valid_multiple_same_sector_only_one_cache_entry() {
        let mut w = World::new();
        for p in [96, 100, 111] {
            pos(&mut w, p, p, false);
        }
        assert_eq!(counts(&mut w).0, 1);
    }

    #[test]
    fn set_valid_different_sectors_multiple_cache_entries() {
        let mut w = World::new();
        for (x, y) in [(0, 0), (16, 0), (0, 16)] {
            pos(&mut w, x, y, false);
        }
        assert_eq!(counts(&mut w).0, 3);
    }

    fn random(w: &mut World, bounds: Rectangle3D, water: bool) -> Option<(i32, i32)> {
        cache::try_get_random_position(w, FELUCCA, &[bounds], water).map(|(p, _)| (p.x, p.y))
    }

    #[test]
    fn try_get_random_position_empty_cache_returns_false() {
        let mut w = World::new();
        assert!(random(&mut w, Rectangle3D::new(0, 0, 0, 100, 100, 20), false).is_none());
    }

    #[test]
    fn try_get_random_position_with_cached_position_returns_true() {
        let mut w = World::new();
        pos(&mut w, 50, 50, false);
        assert_eq!(
            random(&mut w, Rectangle3D::new(0, 0, -128, 100, 100, 256), false),
            Some((50, 50))
        );
    }

    #[test]
    fn try_get_random_position_water_vs_land_separated() {
        let mut w = World::new();
        let bounds = Rectangle3D::new(0, 0, -128, 100, 100, 256);
        pos(&mut w, 50, 50, false);
        pos(&mut w, 60, 60, true);
        assert_eq!(random(&mut w, bounds, false), Some((50, 50)));
        assert_eq!(random(&mut w, bounds, true), Some((60, 60)));
    }

    #[test]
    fn try_get_random_position_outside_bounds_returns_false() {
        let mut w = World::new();
        pos(&mut w, 200, 200, false);
        // Only covers 0-99
        assert!(random(&mut w, Rectangle3D::new(0, 0, -128, 100, 100, 256), false).is_none());
    }

    #[test]
    fn invalidate_sectors_removes_both_land_and_water() {
        let mut w = World::new();
        pos(&mut w, 50, 50, false);
        pos(&mut w, 55, 55, true);
        assert_eq!(counts(&mut w).0, 2);
        cache::invalidate_sectors(&mut w, FELUCCA, Rectangle2D::new(0, 0, 100, 100));
        assert_eq!(counts(&mut w), (0, 0, 0));
    }

    #[test]
    fn invalidate_sectors_only_affects_specified_area() {
        let mut w = World::new();
        pos(&mut w, 50, 50, false);
        pos(&mut w, 500, 500, false);
        assert_eq!(counts(&mut w).0, 2);
        cache::invalidate_sectors(&mut w, FELUCCA, Rectangle2D::new(0, 0, 100, 100));
        assert_eq!(counts(&mut w).0, 1);
    }

    #[test]
    fn invalidate_sectors_different_maps_independent() {
        let mut w = World::new();
        cache::set_valid(&mut w, FELUCCA, Point3D::new(50, 50, 0), false);
        cache::set_valid(&mut w, INTERNAL, Point3D::new(50, 50, 0), false);
        assert_eq!(counts(&mut w).0, 2);
        cache::invalidate_sectors(&mut w, FELUCCA, Rectangle2D::new(0, 0, 100, 100));
        assert_eq!(counts(&mut w).0, 1);
    }

    #[test]
    fn get_spiral_offset_ring1_correct_offsets() {
        for (position, dx, dy) in [
            (0, -1, -1),
            (1, 0, -1),
            (2, 1, -1),
            (3, 1, 0),
            (4, 1, 1),
            (5, 0, 1),
            (6, -1, 1),
            (7, -1, 0),
        ] {
            assert_eq!(
                cache::spiral_offset(1, position),
                (dx, dy),
                "position {position}"
            );
        }
    }

    #[test]
    fn spiral_pattern_ring2_covers_all_expected_tiles() {
        let offsets: HashSet<(i32, i32)> = (0..16).map(|p| cache::spiral_offset(2, p)).collect();
        assert_eq!(offsets.len(), 16);
        for (dx, dy) in offsets {
            assert_eq!(dx.abs().max(dy.abs()), 2);
        }
    }

    #[test]
    fn reset_clears_all_state() {
        let mut state = PosState {
            spiral_ring: 5,
            spiral_ring_position: 10,
            spiral_complete: true,
            ..Default::default()
        };
        state.non_transient_failures += 1;
        state.reset();
        assert!(!state.spiral_complete);
        assert_eq!((state.spiral_ring, state.spiral_ring_position), (0, 0));
    }

    #[test]
    fn should_cache_positions_enabled_always_true() {
        assert!(PosState::default().should_cache_positions(SpawnPositionMode::Enabled));
    }

    #[test]
    fn should_cache_positions_disabled_always_false() {
        let state = PosState {
            non_transient_failures: 1,
            ..Default::default()
        };
        assert!(!state.should_cache_positions(SpawnPositionMode::Disabled));
    }

    #[test]
    fn should_cache_positions_automatic_true_after_threshold_failures() {
        // FailureThreshold is 5, so need more than 5 failures
        let state = PosState {
            non_transient_failures: 6,
            ..Default::default()
        };
        assert!(state.should_cache_positions(SpawnPositionMode::Automatic));
    }

    #[test]
    fn should_cache_positions_automatic_false_with_fewer_than_threshold_failures() {
        let state = PosState {
            non_transient_failures: 5,
            ..Default::default()
        };
        assert!(!state.should_cache_positions(SpawnPositionMode::Automatic));
    }

    #[test]
    fn should_abandon_false_when_spiral_not_complete() {
        let state = PosState {
            consecutive_useless: 25,
            ..Default::default()
        };
        assert!(!state.should_abandon());
    }

    #[test]
    fn should_abandon_true_after_threshold_useless_results() {
        let state = PosState {
            spiral_complete: true,
            consecutive_useless: 25,
            ..Default::default()
        };
        assert!(state.should_abandon());
    }

    #[test]
    fn record_useful_cache_hit_resets_useless_counter() {
        let mut state = PosState {
            spiral_complete: true,
            consecutive_useless: 20,
            ..Default::default()
        };
        state.consecutive_useless = 0; // RecordUsefulCacheHit
        state.consecutive_useless += 24;
        assert!(!state.should_abandon()); // Only 24 after reset
        state.consecutive_useless += 1; // 25th
        assert!(state.should_abandon());
    }
}

// ---- SpawnerEntryOwnershipTests, SpawnerSaveMigrationTests, SpawnerTickAndGroupDtoTests, Json/* -----------------------

mod spawner_behavior_tests {
    use super::super::export;
    use super::super::json::{self, SpawnerDto};
    use super::super::{Kind, data, data_mut};
    use super::*;
    use muo_primitives::Rectangle3D;
    use muo_server::map::MapId;

    const MIN: TimeSpan = TimeSpan::from_mins(1);

    fn spawner_with(w: &mut World, names: &[&str]) -> ItemId {
        core::create(
            w,
            &SPAWNER,
            Kind::Spawner,
            1,
            MIN,
            TimeSpan::from_mins(2),
            0,
            Rectangle3D::default(),
            names,
        )
    }

    fn place(w: &mut World, id: ItemId, x: i32, y: i32) {
        ops::move_to_world_map(w, id, Point3D::new(x, y, 0), Some(MapId::FELUCCA));
    }

    fn fixture(name: &str) -> Option<Vec<u8>> {
        std::fs::read(
            std::path::PathBuf::from(std::env::var_os("MUO_SPAWNER_FIXTURES")?).join(name),
        )
        .ok()
    }

    #[test]
    fn disabled_defaults_false_and_round_trips_binary() {
        let mut w = World::new();
        let spawner = spawner_with(&mut w, &["Rabbit", "Bird"]);
        assert!(!data(&w, spawner).unwrap().entries[0].disabled);
        data_mut(&mut w, spawner).unwrap().entries[1].disabled = true;

        let bytes = save(&w, spawner);
        let loaded = load(&mut w, &SPAWNER, &bytes);
        let d = data(&w, loaded).unwrap();
        assert_eq!(d.entries.len(), 2);
        assert!(!d.entries[0].disabled);
        assert!(d.entries[1].disabled);
    }

    #[test]
    fn disabled_is_omitted_from_json_when_false_and_written_when_true() {
        let mut w = world_with_maps();
        let spawner = spawner_with(&mut w, &["Rabbit"]);
        let json_of =
            |w: &World| export::serialize_compact(w, &[export::to_dto(w, spawner).unwrap()]);
        assert!(!json_of(&w).contains("disabled"));
        data_mut(&mut w, spawner).unwrap().entries[0].disabled = true;
        assert!(json_of(&w).contains("\"disabled\": true"));
    }

    #[test]
    fn add_remove_clear_operate_on_owner_list() {
        let mut w = World::new();
        let spawner = (SPAWNER.construct.unwrap())(&mut w);
        // `new Spawner()` starts with the constructor's parameterless entries list
        core::remove_all_entries(&mut w, spawner);
        assert!(data(&w, spawner).unwrap().entries.is_empty());

        core::add_entry(&mut w, spawner, Some("Rabbit"), 100, 2, false, None, None);
        core::add_entry(
            &mut w,
            spawner,
            Some("Bird"),
            50,
            1,
            false,
            Some("Hue 33"),
            None,
        );
        let d = data(&w, spawner).unwrap();
        assert_eq!(d.entries.len(), 2);
        assert_eq!(d.entries[0].name.as_deref(), Some("Rabbit"));
        assert_eq!(d.entries[1].properties.as_deref(), Some("Hue 33"));

        core::remove_entry(&mut w, spawner, 0);
        let d = data(&w, spawner).unwrap();
        assert_eq!(d.entries.len(), 1);
        assert_eq!(d.entries[0].name.as_deref(), Some("Bird"));

        core::remove_all_entries(&mut w, spawner);
        assert!(data(&w, spawner).unwrap().entries.is_empty());
    }

    #[test]
    fn dupe_clones_entries_into_independent_list() {
        let mut w = World::new();
        let source = spawner_with(&mut w, &["Rabbit", "Bird"]);
        data_mut(&mut w, source).unwrap().entries[1].disabled = true;

        let copy = (SPAWNER.construct.unwrap())(&mut w);
        muo_server::item::base::dupe(&mut w, source, copy);

        let c = data(&w, copy).unwrap();
        assert_eq!(c.entries.len(), 2);
        assert_eq!(c.entries[1].name.as_deref(), Some("Bird"));
        assert!(c.entries[1].disabled);
        assert_ne!(data(&w, source).unwrap().guid, c.guid);

        core::add_entry(&mut w, source, Some("Orc"), 100, 1, false, None, None);
        assert_eq!(data(&w, copy).unwrap().entries.len(), 2);
    }

    #[test]
    fn copy_entries_to_replaces_target_entries() {
        let mut w = World::new();
        let source = spawner_with(&mut w, &["Rabbit"]);
        let target = spawner_with(&mut w, &["Orc", "Troll"]);
        core::copy_entries_to(&mut w, source, target);
        let t = data(&w, target).unwrap();
        assert_eq!(t.entries.len(), 1);
        assert_eq!(t.entries[0].name.as_deref(), Some("Rabbit"));
    }

    #[test]
    fn copy_entries_to_self_is_no_op() {
        let mut w = World::new();
        let spawner = spawner_with(&mut w, &["Rabbit", "Bird"]);
        core::copy_entries_to(&mut w, spawner, spawner);
        let d = data(&w, spawner).unwrap();
        assert_eq!(d.entries.len(), 2);
        assert_eq!(d.entries[0].name.as_deref(), Some("Rabbit"));
        assert_eq!(d.entries[1].name.as_deref(), Some("Bird"));
    }

    #[test]
    fn v12_spawner_entries_are_adopted_by_owner() {
        let Some(bytes) = fixture("spawner.v12-v1.bin") else {
            return;
        };
        let mut w = World::new();
        let id = load(&mut w, &SPAWNER, &bytes);
        assert_eq!(ops::name(&w, id).as_deref(), Some("FixtureSpawner"));
        let d = data(&w, id).unwrap();
        assert_eq!(d.entries.len(), 2);
        assert_eq!(d.entries[0].name.as_deref(), Some("Rabbit"));
        assert_eq!(d.entries[1].properties.as_deref(), Some("Hue 33"));
        assert!(!d.entries[0].disabled);
        assert!(d.spawned.is_empty());
    }

    #[test]
    fn v12_proximity_and_region_entries_are_adopted_through_subclasses() {
        let (Some(p), Some(r)) = (
            fixture("proximity.v12-v1-v0.bin"),
            fixture("region.v12-v1-v0.bin"),
        ) else {
            return;
        };
        let mut w = World::new();
        let prox = load(&mut w, &PROXIMITY_SPAWNER, &p);
        let d = data(&w, prox).unwrap();
        assert_eq!(d.entries.len(), 2);
        assert_eq!(d.entries[0].name.as_deref(), Some("Rat"));
        assert_eq!(d.entries[1].name.as_deref(), Some("Bird"));
        assert_eq!(d.trigger_range, 5);
        assert!(d.instant_flag);

        let region = load(&mut w, &REGION_SPAWNER, &r);
        let d = data(&w, region).unwrap();
        assert_eq!(d.entries.len(), 2);
        assert_eq!(d.entries[0].name.as_deref(), Some("Orc"));
        assert_eq!(d.entries[1].name.as_deref(), Some("Troll"));
    }

    #[test]
    fn next_spawn_on_stopped_spawner_restarts_it() {
        let mut w = world_with_maps();
        let spawner = spawner_with(&mut w, &["Rabbit"]);
        core::stop(&mut w, spawner);
        assert!(!data(&w, spawner).unwrap().running);
        core::set_next_spawn(&mut w, spawner, TimeSpan::from_secs(5));
        assert!(data(&w, spawner).unwrap().running);
    }

    fn full_world() -> World {
        let mut w = world_with_maps();
        muo_content::register(&mut w.types);
        muo_items::register(&mut w.types);
        muo_mobiles::register(&mut w.types);
        super::super::register(&mut w.types);
        w
    }

    #[test]
    fn new_format_round_trips_byte_identical_with_live_spawn_references() {
        let mut w = full_world();
        let spawner = core::create(
            &mut w,
            &SPAWNER,
            Kind::Spawner,
            1,
            TimeSpan::from_mins(5),
            TimeSpan::from_mins(10),
            0,
            Rectangle3D::default(),
            &["Rabbit"],
        );
        place(&mut w, spawner, 1500, 1500);
        core::spawn(&mut w, spawner);
        assert_eq!(data(&w, spawner).unwrap().spawned.len(), 1);

        let first = save(&w, spawner);
        let loaded = load(&mut w, &SPAWNER, &first);
        let d = data(&w, loaded).unwrap();
        assert_eq!(d.entries.len(), 1);
        assert_eq!(d.entries[0].spawned.len(), 1);
        core::rebuild_spawned(&mut w, loaded);
        assert_eq!(data(&w, loaded).unwrap().spawned.len(), 1);

        assert_eq!(first, save(&w, loaded));
    }

    #[test]
    fn dto_round_trip_carries_group() {
        let mut w = world_with_maps();
        let spawner = core::create(
            &mut w,
            &SPAWNER,
            Kind::Spawner,
            1,
            TimeSpan::from_mins(5),
            TimeSpan::from_mins(10),
            0,
            Rectangle3D::default(),
            &["Rabbit"],
        );
        place(&mut w, spawner, 1500, 1500);
        data_mut(&mut w, spawner).unwrap().group = true;
        let text = export::serialize_compact(&w, &[export::to_dto(&w, spawner).unwrap()]);
        assert!(text.contains("\"group\""));

        let v: serde_json::Value = serde_json::from_str(&text).unwrap();
        let dto = json::read_dto(&w, &v[0]).unwrap();
        let loaded = json::to_spawner(&mut w, &dto);
        assert!(data(&w, loaded).unwrap().group);
    }

    // ---- Json/* ----

    fn dtos(w: &World, text: &str) -> Vec<SpawnerDto> {
        let v: serde_json::Value = serde_json::from_str(text).unwrap();
        v.as_array()
            .unwrap()
            .iter()
            .map(|e| json::read_dto(w, e).unwrap())
            .collect()
    }

    fn build(w: &mut World, text: &str) -> ItemId {
        let d = dtos(w, text);
        assert_eq!(d.len(), 1);
        json::to_spawner(w, &d[0])
    }

    fn bounds_of(w: &World, id: ItemId) -> Rectangle3D {
        data(w, id).unwrap().spawn_bounds
    }

    #[test]
    fn spawn_bounds_json_equals_home_range_path() {
        let (x, y, z) = (100, 200, 5);
        for hr in [0, 1, 3, 7] {
            let mut w = world_with_maps();
            // Path A: legacy homeRange field
            let a = format!(
                r#"[{{"$type":"Spawner","location":[{x},{y},{z}],"map":"Felucca","count":1,"homeRange":{hr},"entries":[]}}]"#
            );
            let from_home_range = build(&mut w, &a);
            let expected = bounds_of(&w, from_home_range);

            // Path B: the migrated spawnBounds field
            let (x1, y1, z1, x2, y2, z2) = if hr == 0 {
                (x, y, z, x + 1, y + 1, z)
            } else {
                (x - hr, y - hr, -128, x + hr + 1, y + hr + 1, 128)
            };
            let b = format!(
                r#"[{{"$type":"Spawner","location":[{x},{y},{z}],"map":"Felucca","count":1,"spawnBounds":{{"x1":{x1},"y1":{y1},"z1":{z1},"x2":{x2},"y2":{y2},"z2":{z2}}},"entries":[]}}]"#
            );
            let from_spawn_bounds = build(&mut w, &b);
            assert_eq!(expected, bounds_of(&w, from_spawn_bounds), "hr {hr}");
        }
    }

    #[test]
    fn home_range_no_spawn_bounds_produces_centered_bounds() {
        let mut w = world_with_maps();
        let legacy = r#"[{"$type":"Spawner","guid":"3df0543a-373c-4673-a98b-8191686f4ab3","location":[100,200,5],"map":"Felucca","count":1,
            "homeRange":3,"entries":[{"name":"Fisherman","maxCount":1,"probability":100}]}]"#;
        let s = build(&mut w, legacy);
        // homeRange 3 -> Rectangle3D(100-3, 200-3, -128, 7, 7, 256)
        assert_eq!(bounds_of(&w, s), Rectangle3D::new(97, 197, -128, 7, 7, 256));
    }

    #[test]
    fn home_range_zero_produces_single_tile_bounds() {
        let mut w = world_with_maps();
        let legacy = r#"[{"$type":"Spawner","location":[100,200,5],"map":"Felucca","count":1,"homeRange":0,
            "entries":[{"name":"Fisherman","maxCount":1,"probability":100}]}]"#;
        let s = build(&mut w, legacy);
        // homeRange 0 -> Rectangle3D(100, 200, 5, 1, 1, 0)
        assert_eq!(bounds_of(&w, s), Rectangle3D::new(100, 200, 5, 1, 1, 0));
    }

    #[test]
    fn serialize_then_deserialize_file_preserves_spawner() {
        let mut w = world_with_maps();
        let original = core::create(
            &mut w,
            &SPAWNER,
            Kind::Spawner,
            3,
            TimeSpan::from_mins(4),
            TimeSpan::from_mins(8),
            1,
            Rectangle3D::new(200, 200, 0, 9, 9, 0),
            &["Tanner"],
        );
        place(&mut w, original, 204, 204);
        let path = std::env::temp_dir().join(format!("muo_spawner_rt_{}.json", std::process::id()));
        std::fs::write(
            &path,
            export::serialize_compact(&w, &[export::to_dto(&w, original).unwrap()]),
        )
        .unwrap();

        let text = std::fs::read_to_string(&path).unwrap();
        let rebuilt = build(&mut w, &text);
        let d = data(&w, rebuilt).unwrap();
        assert_eq!((d.count, d.team), (3, 1));
        assert_eq!(d.spawn_bounds, Rectangle3D::new(200, 200, 0, 9, 9, 0));
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn serialize_compact_produces_compact_loadable_format() {
        let mut w = world_with_maps();
        let original = core::create(
            &mut w,
            &SPAWNER,
            Kind::Spawner,
            1,
            TimeSpan::from_mins(5),
            TimeSpan::from_mins(10),
            0,
            Rectangle3D::default(),
            &["Fisherman"],
        );
        place(&mut w, original, 200, 200);
        core::set_spawn_bounds(
            &mut w,
            original,
            Rectangle3D::new(195, 195, -128, 11, 11, 256),
        ); // == homeRange 5

        let json = export::serialize_compact(&w, &[export::to_dto(&w, original).unwrap()]);
        // No BOM, LF, trailing newline.
        assert!(json.starts_with('[') && !json.contains('\r') && json.ends_with("]\n"));
        // $type first; scalar containers inline; entries (array of objects) expanded.
        assert!(json.contains("\"$type\": \"Spawner\""));
        assert!(json.contains("\"location\": [200, 200, 0]"));
        assert!(json.contains("\"homeRange\": 5"));
        assert!(!json.contains("spawnBounds"));
        assert!(json.contains("\"entries\": [\n"));
        assert!(json.contains("{ \"name\": \"Fisherman\","));

        let rebuilt = build(&mut w, &json);
        assert_eq!(
            bounds_of(&w, rebuilt),
            Rectangle3D::new(195, 195, -128, 11, 11, 256)
        );
    }

    #[test]
    fn export_writes_entries_at_same_json_position_with_disabled_only_when_set() {
        let mut w = world_with_maps();
        let spawner = core::create(
            &mut w,
            &SPAWNER,
            Kind::Spawner,
            1,
            TimeSpan::from_mins(5),
            TimeSpan::from_mins(10),
            0,
            Rectangle3D::default(),
            &["Rabbit", "Bird"],
        );
        data_mut(&mut w, spawner).unwrap().entries[1].disabled = true;
        let json = export::serialize_compact(&w, &[export::to_dto(&w, spawner).unwrap()]);
        assert!(
            json.contains("\"entries\": [\n      { \"name\": \"Rabbit\""),
            "{json}"
        );
        assert!(json.contains("{ \"name\": \"Bird\""));
        assert_eq!(json.matches("\"disabled\": true").count(), 1);
    }

    #[test]
    fn import_adopts_deserialized_entry_objects() {
        let mut w = world_with_maps();
        let text = r#"[{"$type":"Spawner","guid":"11111111-1111-1111-1111-111111111111","location":[1500,1500,0],"map":"Felucca","count":1,"minDelay":"00:05:00","maxDelay":"00:10:00","homeRange":4,"entries":[{"name":"Rabbit","probability":100,"maxCount":1,"disabled":true}]}]"#;
        let s = build(&mut w, text);
        let d = data(&w, s).unwrap();
        assert_eq!(d.entries.len(), 1);
        assert!(d.entries[0].disabled);
    }

    fn round_trip_text(w: &World, id: ItemId) -> String {
        export::serialize_compact(w, &[export::to_dto(w, id).unwrap()])
    }

    #[test]
    fn spawner_non_square_bounds_writes_spawn_bounds_round_trips() {
        let mut w = world_with_maps();
        // Bounds centered on (102,102), placed at (105,105) -> not a homeRange square.
        let original = core::create(
            &mut w,
            &SPAWNER,
            Kind::Spawner,
            2,
            TimeSpan::from_mins(3),
            TimeSpan::from_mins(7),
            0,
            Rectangle3D::new(100, 100, 0, 5, 5, 0),
            &["Fisherman"],
        );
        place(&mut w, original, 105, 105);
        let json = round_trip_text(&w, original);
        assert!(json.contains("\"$type\": \"Spawner\"") && json.contains("\"count\": 2"));
        assert!(json.contains("spawnBounds") && !json.contains("homeRange"));

        let s = build(&mut w, &json);
        let d = data(&w, s).unwrap();
        assert_eq!(d.count, 2);
        assert_eq!(d.min_delay, TimeSpan::from_mins(3));
        assert_eq!(d.spawn_bounds, Rectangle3D::new(100, 100, 0, 5, 5, 0));
        assert_eq!(d.entries.len(), 1);
        assert_eq!(d.entries[0].name.as_deref(), Some("Fisherman"));
    }

    #[test]
    fn spawner_writes_mandatory_omits_optional_defaults() {
        let mut w = world_with_maps();
        // Default delays (5/10), team 0, default maxSpawnAttempts.
        let original = core::create(
            &mut w,
            &SPAWNER,
            Kind::Spawner,
            1,
            TimeSpan::from_mins(5),
            TimeSpan::from_mins(10),
            0,
            Rectangle3D::default(),
            &["Fisherman"],
        );
        place(&mut w, original, 110, 110);
        let json = round_trip_text(&w, original);
        assert!(
            json.contains("\"guid\":") && json.contains("minDelay") && json.contains("maxDelay")
        );
        assert!(
            !json.contains("\"team\"")
                && !json.contains("maxSpawnAttempts")
                && !json.contains("spawnLocationIsHome")
        );
    }

    #[test]
    fn proximity_spawner_round_trips_fields() {
        let mut w = world_with_maps();
        let original = core::create(
            &mut w,
            &PROXIMITY_SPAWNER,
            Kind::Proximity,
            1,
            TimeSpan::from_mins(5),
            TimeSpan::from_mins(10),
            0,
            Rectangle3D::default(),
            &["Fisherman"],
        );
        {
            let d = data_mut(&mut w, original).unwrap();
            d.trigger_range = 4;
            d.instant_flag = true;
            d.spawn_message = Some(muo_io::TextDefinition::of_number(500000));
        }
        place(&mut w, original, 120, 120);
        let json = round_trip_text(&w, original);
        assert!(json.contains("\"$type\": \"ProximitySpawner\""));
        assert!(json.contains("\"triggerRange\": 4"));
        assert!(json.contains("\"instant\": true"));

        let p = build(&mut w, &json);
        let d = data(&w, p).unwrap();
        assert_eq!(
            (d.kind, d.trigger_range, d.instant_flag),
            (Kind::Proximity, 4, true)
        );
        assert_eq!(d.spawn_message.as_ref().map(|m| m.number), Some(500000));
    }

    #[test]
    fn region_spawner_round_trips_region_by_name() {
        let mut w = world_with_maps();
        let region = muo_server::region::new_base_region(
            &mut w,
            Some("DtoTestRegion"),
            MapId::FELUCCA,
            50,
            &[muo_primitives::Rectangle2D::new(1400, 1670, 40, 40)],
        );
        muo_server::region::register(&mut w, region);
        let original = core::create(
            &mut w,
            &REGION_SPAWNER,
            Kind::Region,
            1,
            TimeSpan::from_mins(5),
            TimeSpan::from_mins(10),
            0,
            Rectangle3D::default(),
            &["Fisherman"],
        );
        core::set_spawn_region(&mut w, original, Some(region));
        place(&mut w, original, 1416, 1683);
        let json = round_trip_text(&w, original);
        assert!(json.contains("\"$type\": \"RegionSpawner\""));
        assert!(json.contains("DtoTestRegion"));
        assert!(!json.contains("homeRange") && !json.contains("spawnBounds"));

        let rebuilt = build(&mut w, &json);
        let r = core::region(&w, rebuilt).expect("region by name");
        assert_eq!(
            muo_server::region::find_by_name(&w, "DtoTestRegion", MapId::FELUCCA, false),
            Some(r)
        );
    }

    // ---- ImportCleanupTests ----

    fn import(
        w: &mut World,
        name: &str,
        text: &str,
        all: &mut indexmap::IndexMap<muo_io::serialization::Guid, ItemId>,
    ) {
        static N: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0); // tests run in parallel
        let n = N.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let dir =
            std::env::temp_dir().join(format!("muo_spawner_import_{}_{n}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join(name);
        std::fs::write(&path, text).unwrap();
        json::import_json(w, None, &path, all);
        std::fs::remove_dir_all(&dir).ok();
    }

    fn guid(s: &str) -> muo_io::serialization::Guid {
        super::super::save::parse_guid(s).unwrap()
    }

    fn spawners_at(w: &World, map: MapId, x: i32, y: i32) -> Vec<ItemId> {
        w.items
            .iter()
            .filter(|(_, i)| {
                i.map == Some(map)
                    && i.location.x == x
                    && i.location.y == y
                    && i.ext.get::<super::super::SpawnerData>().is_some()
            })
            .map(|(id, _)| *id)
            .collect()
    }

    #[test]
    fn import_valid_file_places_spawner() {
        let mut w = world_with_maps();
        let text = r#"[{"$type":"Spawner","guid":"11111111-1111-1111-1111-111111111111","location":[305,305,0],"map":"Felucca","count":1,
            "spawnBounds":{"x1":300,"y1":300,"x2":310,"y2":310},"entries":[]}]"#;
        let mut all = indexmap::IndexMap::new();
        import(&mut w, "test.json", text, &mut all);
        let g = guid("11111111-1111-1111-1111-111111111111");
        let placed = spawners_at(&w, MapId::FELUCCA, 305, 305)
            .into_iter()
            .find(|id| data(&w, *id).unwrap().guid == g);
        assert!(placed.is_some());
        assert!(all.contains_key(&g));
    }

    #[test]
    fn import_no_map_skips_the_spawner() {
        let mut w = world_with_maps();
        let text = r#"[{"$type":"Spawner","guid":"22222222-2222-2222-2222-222222222222","location":[400,400,0],"count":1,
            "spawnBounds":{"x1":395,"y1":395,"x2":405,"y2":405},"entries":[]}]"#;
        let g = guid("22222222-2222-2222-2222-222222222222");
        let mut all = indexmap::IndexMap::new();
        import(&mut w, "test-nomap.json", text, &mut all);
        assert!(!all.contains_key(&g));
        for (_, i) in w.items.iter() {
            if let Some(d) = i.ext.get::<super::super::SpawnerData>() {
                assert_ne!(d.guid, g);
            }
        }
    }

    #[test]
    fn import_duplicate_location_replaces_existing_spawner() {
        let mut w = world_with_maps();
        let new_guid = guid("33333333-3333-3333-3333-333333333333");
        let text = r#"[{"$type":"Spawner","guid":"33333333-3333-3333-3333-333333333333","location":[310,310,0],"map":"Felucca","count":1,
            "spawnBounds":{"x1":305,"y1":305,"x2":315,"y2":315},"entries":[]}]"#;
        let existing = core::create(
            &mut w,
            &SPAWNER,
            Kind::Spawner,
            1,
            TimeSpan::from_mins(5),
            TimeSpan::from_mins(10),
            0,
            Rectangle3D::default(),
            &[],
        );
        place(&mut w, existing, 310, 310);
        let existing_guid = data(&w, existing).unwrap().guid;
        let mut all = indexmap::IndexMap::new();
        all.insert(existing_guid, existing);
        import(&mut w, "test-dup.json", text, &mut all);

        // existing should have been deleted and replaced.
        assert!(w.item(existing).is_none_or(|i| i.deleted()));
        assert!(!all.contains_key(&existing_guid));
        let placed = spawners_at(&w, MapId::FELUCCA, 310, 310)
            .into_iter()
            .find(|id| data(&w, *id).unwrap().guid == new_guid);
        assert!(placed.is_some());
    }

    #[test]
    fn every_spawn_file_deserializes_and_builds_spawners() {
        let Some(dir) = distribution_spawns() else {
            return;
        };
        let mut w = world_with_maps();
        let (mut files, mut built) = (0, 0);
        for f in json::glob_files(&dir, "**/*.json") {
            files += 1;
            let text = std::fs::read_to_string(&f).unwrap();
            let v: serde_json::Value = serde_json::from_str(text.trim_start_matches('\u{feff}'))
                .unwrap_or_else(|e| panic!("{}: {e}", f.display()));
            for e in v.as_array().into_iter().flatten() {
                let dto =
                    json::read_dto(&w, e).unwrap_or_else(|e2| panic!("{}: {e2}", f.display()));
                json::to_spawner(&mut w, &dto);
                built += 1;
            }
        }
        assert_ne!(files, 0);
        assert!(built > 5000);
    }
}
