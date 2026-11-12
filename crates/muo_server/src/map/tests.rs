//! Map tests (CanFitItem, CanSpawnMobile, CanSpawnMobileTileData, the enumerators,
//! StaticTileEnumerator), terrain tests (the parts that test map behavior), and tests of the
//! rest of the map: sectors, line of sight, parsing.
//!
//! Dropped: `*_ThrowsOnVersionChange` (a list changed under an enumerator cannot compile here),
//! `*_MapNullYieldsEmpty` for the type-level `Empty` (a `None` map is covered once), `MapSelectionTests` and
//! `TileMatrixInjectionTests` (muo_data owns both), `TestTerrainTests` scope/prefab cases (a test world
//! is its own scope), `SyntheticTileDataTests` (muo_data's `TileData::from_bytes`).

use muo_data::TileFlag;
use muo_primitives::{Point2D, Point3D, Rectangle2D};

use super::enumerate::multis_in_sector;
use super::enumerate::*;
use super::fit::*;
use super::surface::*;
use super::test_support::*;
use super::tiles::*;
use super::{MapId, SECTOR_SIZE, SpawnFailureReason};
use crate::net_state::NetStateId;

const FEL: MapId = MapId::FELUCCA;

// ---- Terrain ------------------------------------------------------------------------------------------

#[test]
fn stamping_one_tile_keeps_the_rest_of_the_block() {
    let mut w = world();
    set_land(&mut w, FEL, 1400, 1400, GRASS, 3);
    set_land(&mut w, FEL, 1401, 1400, DIRT, 4);
    assert_eq!(w.maps.land_tile(FEL, 1400, 1400).id, GRASS);
    assert_eq!(w.maps.land_tile(FEL, 1401, 1400).id, DIRT);
}

#[test]
fn stamping_an_uninjected_block_does_not_touch_other_blocks() {
    let mut w = world();
    set_land(&mut w, FEL, 1400, 1400, GRASS, 3);
    assert_eq!(w.maps.land_tile(FEL, 1464, 1464).id, 0);
    assert_eq!(w.maps.land_tile(FEL, 1401, 1400).id, 0);
}

#[test]
fn static_stamp_reaches_the_tile_matrix() {
    let mut w = world();
    add_static(&mut w, FEL, 1400, 1400, WALL, 3);
    let cell = statics_at(&mut w, FEL, 1400, 1400);
    assert_eq!(cell.len(), 1);
    assert_eq!((cell[0].id, cell[0].z), (WALL, 3));
}

#[test]
fn test_multis_register_a_footprint() {
    let mut w = world();
    let id = place_multi(
        &mut w,
        FEL,
        p(300, 300, 0),
        &[entry(FLOOR, 0, 0, 0, TileFlag::empty())],
    );
    let mcl = (w.maps.multi_components)(&w, id);
    assert!(!mcl.list.is_empty());
}

// ---- GetAverageZ ------------------------------------------------------------------------------------

#[test]
fn average_z_of_a_flat_and_a_sloped_tile() {
    let mut w = world();
    assert_eq!(w.maps.get_average_z_full(FEL, 500, 500), (0, 0, 0));

    // top 10, left 0, right 4, bottom 0: |top - bottom| > |left - right| picks (left + right) / 2
    set_land(&mut w, FEL, 600, 600, GRASS, 10);
    set_land(&mut w, FEL, 600, 601, GRASS, 0);
    set_land(&mut w, FEL, 601, 600, GRASS, 4);
    set_land(&mut w, FEL, 601, 601, GRASS, 0);
    assert_eq!(w.maps.get_average_z_full(FEL, 600, 600), (0, 2, 10));

    // |top - bottom| = 1 > |left - right| = 0 -> FloorAverage(0, 0) = 0
    set_land(&mut w, FEL, 700, 700, GRASS, -1);
    set_land(&mut w, FEL, 701, 701, GRASS, -2);
    assert_eq!(w.maps.get_average_z(FEL, 700, 700), 0);
}

// ---- CanFitItem ---------------------------------------------------------------------------------------

#[test]
fn can_fit_item_internal_map_and_bounds() {
    let mut w = world();
    assert!(!can_fit_item(&mut w, MapId::INTERNAL, 100, 100, 0, 1));
    let (width, height) = {
        let m = w.maps.get(FEL).unwrap();
        (m.width, m.height)
    };
    assert!(!can_fit_item(&mut w, FEL, -1, 100, 0, 1));
    assert!(!can_fit_item(&mut w, FEL, 100, -1, 0, 1));
    assert!(!can_fit_item(&mut w, FEL, width + 1, 100, 0, 1));
    assert!(!can_fit_item(&mut w, FEL, 100, height + 1, 0, 1));
}

#[test]
fn can_fit_item_land_surface() {
    let mut w = world();
    let avg = w.maps.get_average_z(FEL, 1500, 1600);
    assert!(can_fit_item(&mut w, FEL, 1500, 1600, avg, 1));
    assert!(
        !can_fit_item(&mut w, FEL, 1500, 1600, avg + 50, 1),
        "no surface 50 units above land"
    );
}

#[test]
fn can_fit_item_surface_impassable_multi_is_a_surface() {
    let mut w = world();
    let (x, y) = (1700, 1700);
    let top = 10
        + i32::from(
            w.maps
                .tile_data
                .item(usize::from(SURFACE_IMPASSABLE_TILE))
                .height,
        );
    place_multi(
        &mut w,
        FEL,
        p(x, y, 0),
        &[entry(
            SURFACE_IMPASSABLE_TILE,
            0,
            0,
            10,
            TileFlag::SURFACE | TileFlag::IMPASSABLE,
        )],
    );
    assert!(can_fit_item(&mut w, FEL, x, y, top, 1));
}

#[test]
fn can_fit_does_not_allow_surface_impassable() {
    let mut w = world();
    let (x, y) = (1750, 1750);
    let top = 10
        + i32::from(
            w.maps
                .tile_data
                .item(usize::from(SURFACE_IMPASSABLE_TILE))
                .height,
        );
    place_multi(
        &mut w,
        FEL,
        p(x, y, 0),
        &[entry(
            SURFACE_IMPASSABLE_TILE,
            0,
            0,
            10,
            TileFlag::SURFACE | TileFlag::IMPASSABLE,
        )],
    );
    assert!(!can_fit(&mut w, FEL, x, y, top, 1, false, true, true));
}

#[test]
fn can_fit_item_non_movable_world_item_surface_works() {
    let mut w = world();
    let (x, y) = (1800, 1800);
    let id = place_item(&mut w, FEL, p(x, y, 10), i32::from(SURFACE_TILE));
    w.item_mut(id)
        .unwrap()
        .flags
        .remove(crate::item::ImplFlag::MOVABLE);
    let top = 10
        + w.maps
            .tile_data
            .item(usize::from(SURFACE_TILE))
            .calc_height();
    assert!(can_fit_item(&mut w, FEL, x, y, top, 1));
}

#[test]
fn can_fit_item_movable_world_item_is_not_a_surface() {
    let mut w = world();
    let (x, y) = (1850, 1850);
    let z = w.maps.get_average_z(FEL, x, y) + 20;
    place_item(&mut w, FEL, p(x, y, z), i32::from(SURFACE_TILE));
    let top = z + w
        .maps
        .tile_data
        .item(usize::from(SURFACE_TILE))
        .calc_height();
    assert!(!can_fit_item(&mut w, FEL, x, y, top, 1));
}

#[test]
fn can_fit_item_impassable_tile_blocks_placement() {
    let mut w = world();
    let (x, y) = (1900, 1900);
    place_multi(
        &mut w,
        FEL,
        p(x, y, 0),
        &[entry(IMPASSABLE_TILE, 0, 0, 10, TileFlag::IMPASSABLE)],
    );
    assert!(!can_fit_item(&mut w, FEL, x, y, 10, 5));
}

// ---- CanSpawnMobile -----------------------------------------------------------------------------------

#[test]
fn can_spawn_mobile_internal_map_and_bounds() {
    let mut w = world();
    assert_eq!(
        can_spawn_mobile(&mut w, MapId::INTERNAL, 100, 100, -128, 127, false, false),
        Err(SpawnFailureReason::INVALID_MAP)
    );
    let width = w.maps.get(FEL).unwrap().width;
    assert!(can_spawn_mobile(&mut w, FEL, -1, 100, -128, 127, false, false).is_err());
    assert!(can_spawn_mobile(&mut w, FEL, 100, -1, -128, 127, false, false).is_err());
    assert!(can_spawn_mobile(&mut w, FEL, width + 1, 100, -128, 127, false, false).is_err());
}

#[test]
fn can_spawn_mobile_on_the_default_ground() {
    let mut w = world();
    let (x, y) = (1500, 1600);
    assert!(can_spawn_mobile(&mut w, FEL, x, y, -128, 127, false, false).is_ok());

    let avg = w.maps.get_average_z(FEL, x, y);
    assert!(
        can_spawn_mobile(&mut w, FEL, x, y, avg + 50, avg + 100, false, false).is_err(),
        "Z range excludes the land"
    );
    assert!(
        can_spawn_mobile(&mut w, FEL, x, y, -128, 127, false, true).is_err(),
        "cantWalk skips land"
    );
}

#[test]
fn can_spawn_mobile_a_mobile_blocks_the_spawn() {
    let mut w = world();
    let (x, y) = (1500, 1600);
    let land_z = w.maps.get_average_z(FEL, x, y);
    place_mobile(&mut w, FEL, p(x, y, land_z));

    let r = can_spawn_mobile(&mut w, FEL, x, y, land_z, land_z + 10, false, false);
    assert_eq!(r, Err(SpawnFailureReason::TRANSIENT_BLOCKER));
}

#[test]
fn can_spawn_mobile_hidden_staff_do_not_block() {
    let mut w = world();
    let (x, y) = (1520, 1600);
    let m = place_mobile(&mut w, FEL, p(x, y, 0));
    {
        let m = w.mobile_mut(m).unwrap();
        m.access_level = crate::mobile::AccessLevel::GameMaster;
        m.hidden = true;
    }
    assert!(can_spawn_mobile(&mut w, FEL, x, y, 0, 10, false, false).is_ok());
}

fn spawn_z(
    w: &mut crate::World,
    x: i32,
    y: i32,
    min: i32,
    max: i32,
    swim: bool,
    cant_walk: bool,
) -> Option<i32> {
    can_spawn_mobile(w, FEL, x, y, min, max, swim, cant_walk).ok()
}

#[test]
fn can_spawn_mobile_finds_the_lowest_valid_surface() {
    let mut w = world();
    let (x, y) = (1200, 1200);
    place_multi(
        &mut w,
        FEL,
        p(x, y, 0),
        &[
            entry(SURFACE_TILE, 0, 0, 10, TileFlag::SURFACE),
            entry(SURFACE_TILE, 0, 0, 50, TileFlag::SURFACE),
        ],
    );
    let z = spawn_z(&mut w, x, y, 5, 60, false, false).expect("a spawn");
    assert!(z <= 20, "expected the lowest floor, got {z}");
}

#[test]
fn can_spawn_mobile_z_range_selects_the_floor() {
    let mut w = world();
    let (x, y) = (1300, 1300);
    place_multi(
        &mut w,
        FEL,
        p(x, y, 0),
        &[
            entry(SURFACE_TILE, 0, 0, 10, TileFlag::SURFACE),
            entry(SURFACE_TILE, 0, 0, 50, TileFlag::SURFACE),
        ],
    );
    let z = spawn_z(&mut w, x, y, 45, 60, false, false).expect("a spawn");
    assert!((45..=60).contains(&z), "expected the second floor, got {z}");
}

#[test]
fn can_spawn_mobile_low_ceiling_blocks_high_ceiling_allows() {
    let mut w = world();
    place_multi(
        &mut w,
        FEL,
        p(1400, 1400, 0),
        &[
            entry(SURFACE_TILE, 0, 0, 20, TileFlag::SURFACE),
            entry(IMPASSABLE_TILE, 0, 0, 28, TileFlag::IMPASSABLE),
        ],
    );
    assert!(spawn_z(&mut w, 1400, 1400, 15, 35, false, false).is_none());

    for (x, ceiling) in [(1450, 50), (1475, 100)] {
        place_multi(
            &mut w,
            FEL,
            p(x, x, 0),
            &[
                entry(SURFACE_TILE, 0, 0, 20, TileFlag::SURFACE),
                entry(IMPASSABLE_TILE, 0, 0, ceiling, TileFlag::IMPASSABLE),
            ],
        );
        assert!(
            spawn_z(&mut w, x, x, 15, 35, false, false).is_some(),
            "ceiling at {ceiling}"
        );
    }
}

#[test]
fn can_spawn_mobile_water_for_a_swimmer() {
    let mut w = world();
    place_multi(
        &mut w,
        FEL,
        p(1550, 1550, 0),
        &[entry(WET_TILE, 0, 0, 0, TileFlag::WET)],
    );
    assert!(spawn_z(&mut w, 1550, 1550, -10, 10, true, false).is_some());
}

#[test]
fn can_spawn_mobile_world_item_surface() {
    let mut w = world();
    let id = place_item(&mut w, FEL, p(1650, 1650, 10), i32::from(SURFACE_TILE));
    w.item_mut(id)
        .unwrap()
        .flags
        .remove(crate::item::ImplFlag::MOVABLE);
    assert!(spawn_z(&mut w, 1650, 1650, 5, 30, false, false).is_some());
}

#[test]
fn can_spawn_mobile_failure_reasons() {
    let mut w = world();
    // an impassable static on the only ground
    add_static(&mut w, FEL, 2000, 2000, ROCK, 0);
    let r = can_spawn_mobile(&mut w, FEL, 2000, 2000, 0, 5, false, false);
    assert_eq!(r, Err(SpawnFailureReason::NON_TRANSIENT_BLOCKER));
    // no surface in range at all is also a map-geometry problem
    let r = can_spawn_mobile(&mut w, FEL, 2100, 2000, 40, 50, false, false);
    assert_eq!(r, Err(SpawnFailureReason::NON_TRANSIENT_BLOCKER));
}

#[test]
fn can_spawn_item_treats_tables_as_surfaces() {
    let mut w = world();
    let (x, y) = (2200, 2200);
    add_static(&mut w, FEL, x, y, COUNTER, 5);
    // counter: Surface | Impassable, height 4: the top at 9 is a surface for an item, never for a mobile
    assert_eq!(can_spawn_item(&mut w, FEL, x, y, 6, 20), Some(9));
    assert!(can_spawn_mobile(&mut w, FEL, x, y, 6, 20, false, false).is_err());
}

#[test]
fn can_fit_with_mobiles_and_surface_options() {
    let mut w = world();
    let (x, y) = (2300, 2300);
    assert!(can_fit(&mut w, FEL, x, y, 0, 16, false, true, true));
    assert!(
        !can_fit(&mut w, FEL, x, y, 3, 16, false, true, true),
        "no surface at 3"
    );
    assert!(can_fit(&mut w, FEL, x, y, 3, 16, false, true, false));
    place_mobile(&mut w, FEL, p(x, y, 0));
    assert!(!can_fit(&mut w, FEL, x, y, 0, 16, false, true, true));
    assert!(
        can_fit(&mut w, FEL, x, y, 0, 16, false, false, true),
        "checkMobiles false ignores the mobile"
    );
}

#[test]
fn random_nearby_location_finds_a_fitting_tile() {
    let mut w = world();
    let loc = get_random_nearby_location(&mut w, FEL, p(2400, 2400, 0), 2, 0, 10, 16, false, false);
    assert!((2400..2402).contains(&loc.x) && (2400..2402).contains(&loc.y) && loc.z == 0);
}

// ---- StaticTileEnumerator -----------------------------------------------------------------------------

fn default_components() -> Vec<muo_data::client::multi_data::MultiTileEntry> {
    vec![
        entry(0x1, 0, 0, 0, TileFlag::empty()),
        entry(0x2, 1, 0, 0, TileFlag::empty()),
    ]
}

#[test]
fn static_tiles_statics_only_ignores_multis() {
    let mut w = world();
    let before = static_tiles(&mut w, FEL, 200, 200);
    place_multi(&mut w, FEL, p(200, 200, 0), &default_components());
    assert_eq!(static_tiles(&mut w, FEL, 200, 200), before);
}

#[test]
fn static_tiles_multis_only_and_both() {
    let mut w = world();
    add_static(&mut w, FEL, 400, 400, WALL, 0);
    place_multi(&mut w, FEL, p(400, 400, 0), &default_components());
    let statics = static_tiles(&mut w, FEL, 400, 400);
    let multi = multi_tiles(&w, FEL, 400, 400);
    let both = static_and_multi_tiles(&mut w, FEL, 400, 400);
    assert!(!multi.is_empty());
    assert_eq!(both.len(), statics.len() + multi.len());
    assert!(statics.iter().chain(&multi).all(|t| both.contains(t)));
    assert_eq!(both[..statics.len()], statics[..], "statics come first");
}

#[test]
fn static_tiles_multi_z_offset_applied() {
    let mut w = world();
    place_multi(&mut w, FEL, p(500, 500, 10), &default_components());
    let tiles = multi_tiles(&w, FEL, 500, 500);
    assert!(!tiles.is_empty());
    assert!(tiles.iter().all(|t| t.z >= 10));
}

#[test]
fn static_tiles_multiple_and_deleted_multis() {
    let mut w = world();
    let a = place_multi(&mut w, FEL, p(900, 900, 0), &default_components());
    place_multi(&mut w, FEL, p(900, 900, 5), &default_components());
    assert!(multi_tiles(&w, FEL, 900, 900).len() >= 2);

    // flagged deleted while still listed (mid-delete): skipped
    flag_deleted(&mut w, a);
    let tiles = multi_tiles(&w, FEL, 900, 900);
    assert!(!tiles.is_empty());
    assert!(tiles.iter().all(|t| t.z >= 5));
}

#[test]
fn static_tiles_a_multi_with_an_empty_cell_does_not_hide_later_multis() {
    let mut w = world();
    // outer's bounds cover the probe cell (1101, 1100) but it has no tile there
    place_multi(
        &mut w,
        FEL,
        p(1100, 1100, 0),
        &[
            entry(0x1, 0, 0, 0, TileFlag::BACKGROUND),
            entry(0x2, 2, 0, 0, TileFlag::BACKGROUND),
        ],
    );
    place_multi(
        &mut w,
        FEL,
        p(1101, 1100, 20),
        &[entry(0x3, 0, 0, 0, TileFlag::BACKGROUND)],
    );
    let tiles = multi_tiles(&w, FEL, 1101, 1100);
    assert_eq!(tiles.len(), 1);
    assert_eq!(tiles[0].z, 20);
}

// ---- ItemEnumerator -----------------------------------------------------------------------------------

fn items_of(w: &crate::World, rect: Rectangle2D) -> Vec<crate::ids::ItemId> {
    items_in_bounds(w, Some(FEL), rect, false).collect()
}

#[test]
fn item_enumerator_filters_by_bounds_and_order() {
    let mut w = world();
    let rect = Rectangle2D::new(100, 100, 32, 32);
    let a = place_item(&mut w, FEL, p(105, 105, 0), 1);
    let b = place_item(&mut w, FEL, p(130, 130, 0), 1);
    place_item(&mut w, FEL, p(90, 90, 0), 1);
    assert_eq!(items_of(&w, rect), vec![a, b]);
}

#[test]
fn item_enumerator_skips_deleted_and_parented_items() {
    let mut w = world();
    let rect = Rectangle2D::new(200, 200, 16, 16);
    let ids: Vec<_> = (205..208)
        .map(|x| place_item(&mut w, FEL, p(x, 205, 0), 1))
        .collect();
    delete_item(&mut w, ids[1]);
    assert_eq!(items_of(&w, rect), vec![ids[0], ids[2]]);

    let rect = Rectangle2D::new(250, 250, 16, 16);
    let a = place_item(&mut w, FEL, p(255, 255, 0), 1);
    let b = place_item(&mut w, FEL, p(256, 255, 0), 1);
    let container = place_item(&mut w, FEL, p(255, 255, 0), 0xE75);
    set_parent(&mut w, b, container);
    assert_eq!(items_of(&w, rect), vec![a, container]);
    // a flagged-deleted item still listed in its sector
    flag_deleted(&mut w, a);
    assert_eq!(items_of(&w, rect), vec![container]);
}

#[test]
fn item_enumerator_make_bounds_inclusive() {
    let mut w = world();
    let rect = Rectangle2D::new(300, 300, 1, 1);
    let a = place_item(&mut w, FEL, p(301, 301, 0), 1);
    assert_eq!(items_in_bounds(&w, Some(FEL), rect, false).count(), 0);
    assert_eq!(
        items_in_bounds(&w, Some(FEL), rect, true).collect::<Vec<_>>(),
        vec![a]
    );
}

#[test]
fn item_enumerator_null_map_yields_empty() {
    let w = world();
    assert_eq!(
        items_in_bounds(&w, None, Rectangle2D::EMPTY, false).count(),
        0
    );
    assert_eq!(items_at(&w, None, 0, 0).count(), 0);
    assert_eq!(
        items_in_bounds_by_distance(&w, None, Rectangle2D::EMPTY, Point2D::ZERO, false).count(),
        0
    );
}

#[test]
fn item_enumerator_steps_across_sectors() {
    let mut w = world();
    let rect = Rectangle2D::new(500, 500, SECTOR_SIZE * 2, SECTOR_SIZE * 2);
    let ids = [
        place_item(&mut w, FEL, p(rect.x() + 1, rect.y() + 1, 0), 1),
        place_item(
            &mut w,
            FEL,
            p(rect.x() + SECTOR_SIZE + 1, rect.y() + 1, 0),
            1,
        ),
        place_item(
            &mut w,
            FEL,
            p(rect.x() + SECTOR_SIZE + 1, rect.y() + SECTOR_SIZE + 1, 0),
            1,
        ),
    ];
    assert_eq!(items_of(&w, rect), ids);
}

#[test]
fn item_enumerator_map_bounds_are_clamped() {
    let mut w = world();
    let (width, height) = {
        let m = w.maps.get(FEL).unwrap();
        (m.width, m.height)
    };
    let rect = Rectangle2D::new(
        width - SECTOR_SIZE - 2,
        height - SECTOR_SIZE - 2,
        SECTOR_SIZE * 2,
        SECTOR_SIZE * 2,
    );
    let a = place_item(&mut w, FEL, p(width - 2, height - 2, 0), 1);
    assert_eq!(items_of(&w, rect), vec![a]);
}

#[test]
fn item_at_filters_exact_location() {
    let mut w = world();
    let a = place_item(&mut w, FEL, p(600, 600, 0), 1);
    let b = place_item(&mut w, FEL, p(600, 600, 0), 1);
    place_item(&mut w, FEL, p(601, 600, 0), 1);
    assert_eq!(
        items_at(&w, Some(FEL), 600, 600).collect::<Vec<_>>(),
        vec![a, b]
    );

    let container = place_item(&mut w, FEL, p(650, 650, 0), 0xE75);
    let inside = place_item(&mut w, FEL, p(650, 650, 0), 1);
    set_parent(&mut w, inside, container);
    assert_eq!(
        items_at(&w, Some(FEL), 650, 650).collect::<Vec<_>>(),
        vec![container]
    );
}

#[test]
fn item_enumerator_zero_and_negative_range_are_the_center_only() {
    let mut w = world();
    let c = place_item(&mut w, FEL, p(700, 700, 0), 1);
    place_item(&mut w, FEL, p(701, 700, 0), 1);
    assert_eq!(
        items_in_range(&w, Some(FEL), 700, 700, 0).collect::<Vec<_>>(),
        vec![c]
    );
    assert_eq!(
        items_in_range(&w, Some(FEL), 700, 700, -5).collect::<Vec<_>>(),
        vec![c]
    );
}

// ---- ItemByDistanceEnumerator -------------------------------------------------------------------------

#[test]
fn item_by_distance_ring_traversal() {
    let mut w = world();
    let c = p(900, 900, 0);
    let ids = [
        place_item(&mut w, FEL, c, 1),
        place_item(&mut w, FEL, p(c.x + SECTOR_SIZE, c.y, 0), 1),
        place_item(&mut w, FEL, p(c.x, c.y + SECTOR_SIZE, 0), 1),
        place_item(&mut w, FEL, p(c.x + SECTOR_SIZE * 2 - 1, c.y, 0), 1),
    ];
    let found: Vec<_> =
        items_in_range_by_distance(&w, Some(FEL), c.x, c.y, SECTOR_SIZE * 2).collect();
    assert_eq!(found.len(), 4);
    for id in ids {
        assert!(found.iter().any(|f| f.0 == id));
    }
    assert_eq!(
        found.iter().find(|f| f.0 == ids[0]).unwrap().1,
        0,
        "the center item is distance 0"
    );
    // ring order: the center sector first
    assert_eq!(found[0].0, ids[0]);
}

#[test]
fn item_by_distance_edge_cases() {
    let mut w = world();
    let (width, height) = {
        let m = w.maps.get(FEL).unwrap();
        (m.width, m.height)
    };
    let a = place_item(&mut w, FEL, p(width - 2, height - 2, 0), 1);
    let found: Vec<_> =
        items_in_range_by_distance(&w, Some(FEL), width - 2, height - 2, SECTOR_SIZE * 2)
            .map(|f| f.0)
            .collect();
    assert_eq!(found, vec![a]);

    let c = place_item(&mut w, FEL, p(1000, 1000, 0), 1);
    place_item(&mut w, FEL, p(1001, 1000, 0), 1);
    let found: Vec<_> = items_in_range_by_distance(&w, Some(FEL), 1000, 1000, -5)
        .map(|f| f.0)
        .collect();
    assert_eq!(found, vec![c]);
}

#[test]
fn item_by_distance_matches_bounds_enumerator_at_the_map_edge() {
    let mut w = world();
    // off-map items live in the shared invalid sector, which neither walk visits
    place_item(&mut w, FEL, p(3, -2, 0), 1);
    let by_distance: Vec<_> = items_in_range_by_distance(&w, Some(FEL), 5, 5, 18)
        .map(|f| f.0)
        .collect();
    let in_range: Vec<_> = items_in_range(&w, Some(FEL), 5, 5, 18).collect();
    assert_eq!(by_distance, in_range);
}

#[test]
fn item_by_distance_bounds_orders_by_proximity() {
    let mut w = world();
    let bounds = Rectangle2D::new(500, 500, 64, 64);
    let ids = [
        place_item(&mut w, FEL, p(532, 532, 0), 1),
        place_item(&mut w, FEL, p(548, 532, 0), 1),
        place_item(&mut w, FEL, p(563, 563, 0), 1),
    ];
    let found: Vec<_> =
        items_in_bounds_by_distance(&w, Some(FEL), bounds, bounds_center(bounds), false).collect();
    assert_eq!(found.len(), 3);
    let d = |id| found.iter().find(|f| f.0 == id).unwrap().1;
    assert!(d(ids[0]) <= d(ids[1]) && d(ids[1]) <= d(ids[2]));

    // makeBoundsInclusive
    let rect = Rectangle2D::new(300, 300, 8, 8);
    let edge = place_item(&mut w, FEL, p(308, 308, 0), 1);
    assert!(
        items_in_bounds_by_distance(&w, Some(FEL), rect, Point2D::new(304, 304), false)
            .all(|f| f.0 != edge)
    );
    assert!(
        items_in_bounds_by_distance(&w, Some(FEL), rect, Point2D::new(304, 304), true)
            .any(|f| f.0 == edge)
    );
}

// ---- Mobile enumerators -------------------------------------------------------------------------------

#[test]
fn mobile_enumerators() {
    let mut w = world();
    let rect = Rectangle2D::new(100, 100, 32, 32);
    let a = place_mobile(&mut w, FEL, p(105, 105, 0));
    let b = place_mobile(&mut w, FEL, p(130, 130, 0));
    place_mobile(&mut w, FEL, p(90, 90, 0));
    assert_eq!(
        mobiles_in_bounds(&w, Some(FEL), rect, false).collect::<Vec<_>>(),
        vec![a, b]
    );

    w.mobile_mut(a).unwrap().deleted = true;
    assert_eq!(
        mobiles_in_bounds(&w, Some(FEL), rect, false).collect::<Vec<_>>(),
        vec![b]
    );

    let c = place_mobile(&mut w, FEL, p(300, 300, 0));
    place_mobile(&mut w, FEL, p(301, 300, 0));
    assert_eq!(
        mobiles_in_range(&w, Some(FEL), 300, 300, 0).collect::<Vec<_>>(),
        vec![c]
    );
    assert_eq!(
        mobiles_in_range(&w, Some(FEL), 300, 300, -5).collect::<Vec<_>>(),
        vec![c]
    );
    assert_eq!(
        mobiles_at(&w, Some(FEL), 300, 300).collect::<Vec<_>>(),
        vec![c]
    );

    // across sectors, and clamped at the map edge
    let rect = Rectangle2D::new(500, 500, SECTOR_SIZE * 2, SECTOR_SIZE * 2);
    let ids = [
        place_mobile(&mut w, FEL, p(501, 501, 0)),
        place_mobile(&mut w, FEL, p(500 + SECTOR_SIZE + 1, 501, 0)),
        place_mobile(
            &mut w,
            FEL,
            p(500 + SECTOR_SIZE + 1, 500 + SECTOR_SIZE + 1, 0),
        ),
    ];
    assert_eq!(
        mobiles_in_bounds(&w, Some(FEL), rect, false).collect::<Vec<_>>(),
        ids
    );
    let (width, height) = {
        let m = w.maps.get(FEL).unwrap();
        (m.width, m.height)
    };
    let edge = place_mobile(&mut w, FEL, p(width - 2, height - 2, 0));
    let rect = Rectangle2D::new(
        width - SECTOR_SIZE - 2,
        height - SECTOR_SIZE - 2,
        SECTOR_SIZE * 2,
        SECTOR_SIZE * 2,
    );
    assert_eq!(
        mobiles_in_bounds(&w, Some(FEL), rect, false).collect::<Vec<_>>(),
        vec![edge]
    );
}

#[test]
fn mobile_by_distance() {
    let mut w = world();
    let c = p(900, 900, 0);
    let ids = [
        place_mobile(&mut w, FEL, c),
        place_mobile(&mut w, FEL, p(c.x + SECTOR_SIZE, c.y, 0)),
        place_mobile(&mut w, FEL, p(c.x, c.y + SECTOR_SIZE, 0)),
        place_mobile(&mut w, FEL, p(c.x + SECTOR_SIZE * 2 - 1, c.y, 0)),
    ];
    let found: Vec<_> =
        mobiles_in_range_by_distance(&w, Some(FEL), c.x, c.y, SECTOR_SIZE * 2).collect();
    assert_eq!(found.len(), 4);
    assert_eq!(found[0], (ids[0], 0));

    w.mobile_mut(ids[1]).unwrap().deleted = true;
    let found: Vec<_> = mobiles_in_range_by_distance(&w, Some(FEL), c.x, c.y, SECTOR_SIZE * 2)
        .map(|f| f.0)
        .collect();
    assert!(!found.contains(&ids[1]));
    assert_eq!(
        mobiles_in_range_by_distance(&w, Some(FEL), c.x, c.y, 0).count(),
        1
    );
}

// ---- Client enumerators -------------------------------------------------------------------------------

fn client(w: &mut crate::World, p: Point3D, n: u32) -> (NetStateId, crate::ids::MobileId) {
    let ns = NetStateId(n);
    let id = w.new_mobile(&crate::mobile::MOBILE_BASE);
    {
        let m = w.mobile_mut(id).unwrap();
        m.location = p;
        m.map = Some(FEL);
        m.net_state = Some(ns);
    }
    super::sector::on_enter_mobile(w, FEL, p, id);
    (ns, id)
}

#[test]
fn client_enumerators() {
    let mut w = world();
    let rect = Rectangle2D::new(100, 100, 32, 32);
    let a = client(&mut w, p(105, 105, 0), 1);
    let b = client(&mut w, p(130, 130, 0), 2);
    client(&mut w, p(90, 90, 0), 3);
    assert_eq!(
        clients_in_bounds(&w, Some(FEL), rect, false).collect::<Vec<_>>(),
        vec![a.0, b.0]
    );

    // a client whose mobile is gone is skipped
    w.mobile_mut(a.1).unwrap().deleted = true;
    assert_eq!(
        clients_in_bounds(&w, Some(FEL), rect, false).collect::<Vec<_>>(),
        vec![b.0]
    );

    let c1 = client(&mut w, p(600, 600, 0), 10);
    let c2 = client(&mut w, p(600, 600, 0), 11);
    client(&mut w, p(601, 600, 0), 12);
    assert_eq!(
        clients_at(&w, Some(FEL), 600, 600).collect::<Vec<_>>(),
        vec![c1.0, c2.0]
    );

    let n = client(&mut w, p(902, 902, 0), 20);
    let m = client(&mut w, p(898, 898, 0), 21);
    client(&mut w, p(910, 910, 0), 22);
    assert_eq!(
        clients_in_range(&w, Some(FEL), 900, 900, 5).collect::<Vec<_>>(),
        vec![n.0, m.0],
        "898 and 902 share sector 56: entry order"
    );
    let z = client(&mut w, p(950, 950, 0), 30);
    client(&mut w, p(951, 950, 0), 31);
    assert_eq!(
        clients_in_range(&w, Some(FEL), 950, 950, 0).collect::<Vec<_>>(),
        vec![z.0]
    );
    assert_eq!(
        clients_in_range(&w, Some(FEL), 950, 950, -5).collect::<Vec<_>>(),
        vec![z.0]
    );
    assert_eq!(
        clients_in_range_by_distance(&w, Some(FEL), 950, 950, 3).collect::<Vec<_>>(),
        vec![(z.0, 0), (NetStateId(31), 0)],
        "the distance is the sector ring"
    );
    assert_eq!(clients_in_range(&w, None, 950, 950, 3).count(), 0);
}

// ---- Multi enumerators --------------------------------------------------------------------------------

fn multi_at(w: &mut crate::World, x: i32, y: i32) -> crate::ids::ItemId {
    place_multi(w, FEL, p(x, y, 0), &[])
}

#[test]
fn multi_enumerators() {
    let mut w = world();
    let rect = Rectangle2D::new(100, 100, 32, 32);
    let a = multi_at(&mut w, 105, 105);
    let b = multi_at(&mut w, 130, 130);
    multi_at(&mut w, 90, 90);
    assert_eq!(
        multis_in_bounds(&w, Some(FEL), rect, false).collect::<Vec<_>>(),
        vec![a, b]
    );

    flag_deleted(&mut w, a);
    assert_eq!(
        multis_in_bounds(&w, Some(FEL), rect, false).collect::<Vec<_>>(),
        vec![b]
    );

    let inclusive = multi_at(&mut w, 301, 301);
    let rect = Rectangle2D::new(300, 300, 1, 1);
    assert_eq!(multis_in_bounds(&w, Some(FEL), rect, false).count(), 0);
    assert_eq!(
        multis_in_bounds(&w, Some(FEL), rect, true).collect::<Vec<_>>(),
        vec![inclusive]
    );

    let near = multi_at(&mut w, 602, 602);
    let near2 = multi_at(&mut w, 598, 598);
    multi_at(&mut w, 610, 610);
    let found: Vec<_> = multis_in_range(&w, Some(FEL), 600, 600, 5).collect();
    assert_eq!(found.len(), 2);
    assert!(found.contains(&near) && found.contains(&near2));

    let s1 = multi_at(&mut w, 700, 700);
    let s2 = multi_at(&mut w, 701, 700);
    assert_eq!(
        multis_in_sector(&w, Some(FEL), 700, 700).collect::<Vec<_>>(),
        vec![s1, s2]
    );
    assert_eq!(multis_in_sector(&w, None, 700, 700).count(), 0);

    let c = multi_at(&mut w, 800, 800);
    multi_at(&mut w, 801, 800);
    assert_eq!(
        multis_in_range(&w, Some(FEL), 800, 800, 0).collect::<Vec<_>>(),
        vec![c]
    );
    assert_eq!(
        multis_in_range(&w, Some(FEL), 800, 800, -5).collect::<Vec<_>>(),
        vec![c]
    );
}

#[test]
fn multi_spanning_two_sectors_is_yielded_once() {
    let mut w = world();
    // components reach two tiles either side, so at x = 1024 the multi spans sectors 63 and 64
    let spanning = place_multi(
        &mut w,
        FEL,
        p(1024, 1000, 0),
        &[
            entry(0x1, -2, 0, 0, TileFlag::BACKGROUND),
            entry(0x1, 2, 0, 0, TileFlag::BACKGROUND),
        ],
    );
    let m = w.maps.get(FEL).unwrap();
    assert!(
        m.sector(63, 62).multis.contains(&spanning) && m.sector(64, 62).multis.contains(&spanning)
    );

    let found: Vec<_> =
        multis_in_bounds(&w, Some(FEL), Rectangle2D::new(1000, 990, 48, 20), false).collect();
    assert_eq!(found, vec![spanning]);
    // a nested query keeps its own bookkeeping
    let mut outer = Vec::new();
    for id in multis_in_bounds(&w, Some(FEL), Rectangle2D::new(1000, 990, 48, 20), false) {
        outer.push(id);
        assert_eq!(
            multis_in_bounds(&w, Some(FEL), Rectangle2D::new(3000, 3000, 4, 4), false).count(),
            0
        );
    }
    assert_eq!(outer, vec![spanning]);
}

#[test]
fn moving_a_multi_re_registers_its_sectors() {
    let mut w = world();
    let id = place_multi(
        &mut w,
        FEL,
        p(1024, 1000, 0),
        &[
            entry(0x1, -2, 0, 0, TileFlag::BACKGROUND),
            entry(0x1, 2, 0, 0, TileFlag::BACKGROUND),
        ],
    );
    let old = w.item(id).unwrap().location;
    w.item_mut(id).unwrap().location = p(1104, 1000, 0);
    super::sector::on_move_item(&mut w, FEL, old, id);
    let m = w.maps.get(FEL).unwrap();
    assert!(!m.sector(63, 62).multis.contains(&id) && !m.sector(64, 62).multis.contains(&id));
    assert!(m.sector(68, 62).multis.contains(&id) && m.sector(69, 62).multis.contains(&id));
    assert!(m.sector(64, 62).items.is_empty() && m.sector(69, 62).items.contains(&id)); // 1104 is in sector 69
}

// ---- Surfaces, LOS, sectors ---------------------------------------------------------------------------

#[test]
fn top_surface() {
    let mut w = world();
    let (x, y) = (2500, 2500);
    // land at 0, a floor static whose top is 10
    add_static(&mut w, FEL, x, y, FLOOR, 10);
    assert!(matches!(
        get_top_surface(&mut w, FEL, p(x, y, 5)),
        Some(Surface::Land(_))
    ));
    assert!(matches!(
        get_top_surface(&mut w, FEL, p(x, y, 10)),
        Some(Surface::Static(_))
    ));
    assert_eq!(get_top_surface_z(&mut w, FEL, p(x, y, 30)), 10);
    assert_eq!(
        get_top_surface_z(&mut w, FEL, p(x, y, -50)),
        -50,
        "nothing below: the point's own Z"
    );
    assert_eq!(get_top_surface_z(&mut w, MapId::INTERNAL, p(x, y, 7)), 7);
    assert!(get_top_surface(&mut w, MapId::INTERNAL, p(x, y, 7)).is_none());

    let item = place_item(&mut w, FEL, p(x + 1, y, 20), i32::from(FLOOR));
    w.item_mut(item)
        .unwrap()
        .flags
        .remove(crate::item::ImplFlag::MOVABLE);
    assert_eq!(
        get_top_surface(&mut w, FEL, p(x + 1, y, 25)),
        Some(Surface::Item(item))
    );
}

#[test]
fn fix_column_drops_a_hovering_item_to_the_ground() {
    let mut w = world();
    let (x, y) = (2600, 2600);
    let hover = place_item(&mut w, FEL, p(x, y, 8), i32::from(PLAIN));
    fix_column(&mut w, FEL, x, y);
    assert_eq!(w.item(hover).unwrap().location.z, 0);
}

#[test]
fn line_of_sight_open_ground_and_walls() {
    let mut w = world();
    let los = |w: &mut crate::World, a, b| super::los::line_of_sight(w, FEL, a, b);
    assert!(los(&mut w, p(1000, 1000, 14), p(1005, 1000, 14)));
    assert!(los(&mut w, p(1000, 1000, 0), p(1000, 1000, 0)));
    assert!(
        !los(&mut w, p(1000, 1000, 0), p(1030, 1000, 0)),
        "beyond MaxLOSDistance"
    );
    assert!(!super::los::line_of_sight(
        &mut w,
        MapId::INTERNAL,
        p(1, 1, 0),
        p(2, 1, 0)
    ));

    add_static(&mut w, FEL, 1003, 1000, WALL, 0);
    assert!(
        !los(&mut w, p(1000, 1000, 10), p(1005, 1000, 10)),
        "a wall in the way"
    );
    assert!(
        los(&mut w, p(1000, 1000, 10), p(1003, 1000, 10)),
        "the wall is the target's own tile"
    );
    assert!(los(&mut w, p(1005, 1000, 10), p(1006, 1000, 10)));
    assert!(
        !los(&mut w, p(1005, 1000, 10), p(1000, 1000, 10)),
        "the same, from the other side"
    );

    // a window item blocks like a wall
    let win = place_item(&mut w, FEL, p(1103, 1000, 0), i32::from(WINDOW));
    assert!(!los(&mut w, p(1100, 1000, 10), p(1105, 1000, 10)));
    delete_item(&mut w, win);
    assert!(los(&mut w, p(1100, 1000, 10), p(1105, 1000, 10)));
}

#[test]
fn line_of_sight_mobiles_and_objects() {
    use super::los::*;
    let mut w = world();
    let a = place_mobile(&mut w, FEL, p(1200, 1200, 0));
    let b = place_mobile(&mut w, FEL, p(1204, 1200, 0));
    assert!(line_of_sight_mobiles(&mut w, FEL, a, a));
    assert!(line_of_sight_mobiles(&mut w, FEL, a, b));
    add_static(&mut w, FEL, 1202, 1200, WALL, 0);
    assert!(!line_of_sight_mobiles(&mut w, FEL, a, b));
    w.mobile_mut(a).unwrap().access_level = crate::mobile::AccessLevel::GameMaster;
    assert!(
        line_of_sight_mobiles(&mut w, FEL, a, b),
        "staff see through"
    );
    assert!(line_of_sight_objects(
        &mut w,
        FEL,
        LosObject::Mobile(a),
        LosObject::Mobile(b)
    ));

    w.mobile_mut(a).unwrap().access_level = crate::mobile::AccessLevel::Player;
    assert!(!line_of_sight_objects(
        &mut w,
        FEL,
        LosObject::Mobile(a),
        LosObject::Mobile(b)
    ));
    assert_eq!(
        get_point(&mut w, FEL, LosObject::Mobile(a), true),
        p(1200, 1200, 14)
    );
    assert_eq!(
        get_point(&mut w, FEL, LosObject::Land(p(7, 7, 99)), false),
        p(7, 7, 1)
    );
    assert_eq!(
        get_point(
            &mut w,
            FEL,
            LosObject::Static {
                x: 5,
                y: 6,
                z: 20,
                item_id: i32::from(WALL)
            },
            false
        ),
        p(5, 6, 10 + 1)
    );
    // an item in a backpack is seen where the carrier is
    let pack = place_item(&mut w, FEL, p(1, 1, 0), 0xE75);
    let gem = place_item(&mut w, FEL, p(1, 1, 0), 0xF21);
    w.item_mut(gem).unwrap().parent = Some(crate::ids::EntityId::Item(pack));
    w.item_mut(pack).unwrap().parent = Some(crate::ids::EntityId::Mobile(b));
    assert_eq!(
        get_point(&mut w, FEL, LosObject::Item(gem), false),
        p(1204, 1200, 14)
    );
    assert!(
        line_of_sight_objects(&mut w, FEL, LosObject::Mobile(b), LosObject::Item(gem)),
        "the root parent itself"
    );
}

#[test]
fn sectors_hold_entities_and_move_them() {
    let mut w = world();
    let m = place_mobile(&mut w, FEL, p(20, 20, 0));
    let it = place_item(&mut w, FEL, p(20, 20, 0), 1);
    {
        let map = w.maps.get(FEL).unwrap();
        assert_eq!(map.sector_at(20, 20).mobiles, vec![m]);
        assert_eq!(map.sector_at(20, 20).items, vec![it]);
    }

    let old = w.mobile(m).unwrap().location;
    w.mobile_mut(m).unwrap().location = p(40, 20, 0);
    super::sector::on_move_mobile(&mut w, FEL, old, m);
    let map = w.maps.get(FEL).unwrap();
    assert!(map.sector_at(20, 20).mobiles.is_empty());
    assert_eq!(map.sector_at(40, 20).mobiles, vec![m]);

    // out of the map: the shared invalid sector
    let far = place_item(&mut w, FEL, p(-5, -5, 0), 1);
    assert_eq!(w.maps.get(FEL).unwrap().sector_at(-5, -5).items, vec![far]);
    assert_eq!(
        w.maps.get(FEL).unwrap().sector_at(-100, -9).items,
        vec![far],
        "one shared invalid sector"
    );
}

#[test]
fn sectors_wake_around_clients_and_sleep_without() {
    let mut w = world();
    let (ns, id) = client(&mut w, p(160, 160, 0), 1);
    {
        let map = w.maps.get(FEL).unwrap();
        assert!(map.sector(10, 10).active());
        assert!(map.sector(12, 12).active() && map.sector(8, 8).active());
        assert!(!map.sector(13, 10).active());
        assert_eq!(map.sector(10, 10).clients, vec![(ns, id)]);
    }
    super::sector::on_leave_mobile(&mut w, FEL, p(160, 160, 0), id);
    let map = w.maps.get(FEL).unwrap();
    assert!(!map.sector(10, 10).active() && !map.sector(12, 12).active());
    assert!(map.sector(10, 10).clients.is_empty());

    // the internal map never reports active
    assert!(!w.maps.get(MapId::INTERNAL).unwrap().sector(0, 0).active());
}

#[test]
fn on_client_change_swaps_the_connection() {
    let mut w = world();
    let m = place_mobile(&mut w, FEL, p(160, 160, 0));
    super::sector::on_client_change(&mut w, FEL, None, Some(NetStateId(5)), m);
    assert!(w.maps.get(FEL).unwrap().sector(10, 10).active());
    assert_eq!(
        w.maps.get(FEL).unwrap().sector(10, 10).clients,
        vec![(NetStateId(5), m)]
    );
    super::sector::on_client_change(&mut w, FEL, Some(NetStateId(5)), None, m);
    assert!(!w.maps.get(FEL).unwrap().sector(10, 10).active());
}

// ---- Map: names, parse, bound, registration -----------------------------------------------------------

#[test]
fn parse_names_and_indexes() {
    let w = world();
    let m = &w.maps;
    assert_eq!(m.parse("Felucca"), Some(MapId(0)));
    assert_eq!(m.parse("  trammel "), Some(MapId(1)));
    assert_eq!(m.parse("1"), Some(MapId(1)));
    assert_eq!(m.parse("127"), Some(MapId::INTERNAL));
    assert_eq!(m.parse("internal"), Some(MapId::INTERNAL));
    assert_eq!(m.parse("Malas"), None);
    assert_eq!(m.parse("5"), None);
    assert_eq!(m.parse(""), None);
    assert_eq!(m.map_names(), vec!["Felucca", "Trammel", "Internal"]);
    assert_eq!(m.map_values(), vec![MapId(0), MapId(1), MapId::INTERNAL]);
}

#[test]
fn bound_clamps_and_internal_keeps_its_name() {
    let mut w = world();
    let fel = w.maps.get(FEL).unwrap();
    assert_eq!(fel.bound(-5, 99999), (0, fel.height - 1));
    let int = w.maps.get_mut(MapId::INTERNAL).unwrap();
    assert_eq!(
        (int.width, int.height),
        (SECTOR_SIZE, SECTOR_SIZE),
        "at least a sector"
    );
    int.set_name("Elsewhere");
    assert_eq!(int.name(), "Internal");
    let fel = w.maps.get_mut(FEL).unwrap();
    fel.set_name("Britannia");
    assert_eq!(fel.name(), "Britannia");
}

#[test]
fn floor_average_rounds_toward_zero() {
    // top/bottom pair wins when the diagonal differences tie
    let mut w = world();
    set_land(&mut w, FEL, 800, 800, GRASS, -3);
    set_land(&mut w, FEL, 801, 801, GRASS, 0);
    // |top - bottom| = 3 > |left - right| = 0 -> average of left and right = 0
    assert_eq!(w.maps.get_average_z(FEL, 800, 800), 0);
    set_land(&mut w, FEL, 900, 900, GRASS, -3);
    set_land(&mut w, FEL, 901, 901, GRASS, -2);
    set_land(&mut w, FEL, 900, 901, GRASS, -2);
    set_land(&mut w, FEL, 901, 900, GRASS, -2);
    // |top - bottom| = 1 > |left - right| = 0 -> FloorAverage(-2, -2): -4 nudged to -5, / 2 = -2 (truncating)
    assert_eq!(w.maps.get_average_z(FEL, 900, 900), -2);
}

#[test]
fn tile_data_masks_ids() {
    let w = world();
    assert_eq!(
        item_data(
            &w,
            &crate::Item::new(
                muo_primitives::Serial::new(0x4000_0001),
                &crate::item::ITEM_BASE
            )
        )
        .height,
        0
    );
}

// ---- CanSpawnMobileTests, the ones the tests above only approximate ----

#[test]
fn can_spawn_mobile_invalid_coordinates_returns_false() {
    let mut w = world();
    let (width, height) = {
        let m = w.maps.get(FEL).unwrap();
        (m.width, m.height)
    };
    assert!(can_spawn_mobile(&mut w, FEL, -1, 100, -128, 127, false, false).is_err());
    assert!(can_spawn_mobile(&mut w, FEL, 100, -1, -128, 127, false, false).is_err());
    assert!(can_spawn_mobile(&mut w, FEL, width + 1, 100, -128, 127, false, false).is_err());
    assert!(can_spawn_mobile(&mut w, FEL, 100, height + 1, -128, 127, false, false).is_err());
}

#[test]
fn can_spawn_mobile_mobile_blocks_spawn() {
    let mut w = world();
    let (x, y) = (1500, 1600);
    let land_z = w.maps.get_average_z(FEL, x, y);
    place_mobile(&mut w, FEL, p(x, y, land_z));

    // Land surface is blocked by mobile at the same Z
    assert!(can_spawn_mobile(&mut w, FEL, x, y, land_z - 16, land_z + 16, false, false).is_err());
}

#[test]
fn can_spawn_mobile_mobile_at_different_z_does_not_block() {
    let mut w = world();
    let (x, y) = (1520, 1620);
    let land_z = w.maps.get_average_z(FEL, x, y);
    // a mobile at Z=landZ+50 (far above the land)
    place_mobile(&mut w, FEL, p(x, y, land_z + 50));

    // Land should still be valid (mobile above doesn't block ground level)
    assert!(can_spawn_mobile(&mut w, FEL, x, y, land_z - 16, land_z + 16, false, false).is_ok());
}
