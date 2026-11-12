//! House placement: whether a house may stand at a place.

use muo_content::items::house_deps::deps;
use muo_content::regions::{NO_HOUSING_GUARDED_REGION, NO_HOUSING_REGION};
use muo_data::TileData;
use muo_data::tile_data::TileFlag;
use muo_primitives::{Direction, Point2D, Point3D};
use muo_server::World;
use muo_server::ids::{EntityId, MobileId};
use muo_server::item::multi;
use muo_server::map::{MapId, enumerate, tiles};
use muo_server::mobile::AccessLevel;
use muo_server::region;

use crate::house::{self, decay::TEMP_NO_HOUSING_REGION};

/// `HousePlacementResult`.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum HousePlacementResult {
    Valid,
    BadRegion,
    BadLand,
    BadStatic,
    BadItem,
    NoSurface,
    BadRegionHidden,
    BadRegionTemp,
    InvalidCastleKeep,
    BadRegionRaffle,
}

const YARD_SIZE: i32 = 5;

/// `region.AllowHousing(from, p)` (virtual).
pub fn allow_housing(w: &World, reg: region::RegionId, from: MobileId, p: Point3D) -> bool {
    w.region(reg)
        .is_none_or(|r| (r.ty.allow_housing)(w, reg, from, p))
}

/// Any land tile which matches one of these ID numbers is considered a road and cannot be placed over.
const ROAD_IDS: [i32; 18] = [
    0x0071, 0x0078, 0x00E8, 0x00EB, 0x07AE, 0x07B1, 0x3FF4, 0x3FF4, 0x3FF8, 0x3FFB, 0x0442,
    0x0479, // Sand stones
    0x0501, 0x0510, // Sand stones
    0x0009, 0x0015, // Furrows
    0x0150, 0x015C, // Furrows
];

fn is_road(land_id: i32) -> bool {
    ROAD_IDS
        .chunks(2)
        .any(|r| land_id >= r[0] && land_id <= r[1])
}

/// `HouseFoundation.AddStairsTo(ref mcl)`: a copy with a row of stairs added to the south.
pub fn components_with_stairs(
    w: &World,
    multi_id: i32,
) -> muo_data::client::multi_data::MultiComponentList {
    let mut mcl = w.maps.multi_data.get_components(multi_id).clone();
    mcl.resize(mcl.width(), mcl.height() + 1);
    let x_center = mcl.center().x;
    let y_center = mcl.center().y;
    let y = mcl.height() - 1;
    for x in 0..mcl.width() {
        mcl.add(0x63, x - x_center, y - y_center, 0, &w.maps.tile_data);
    }
    mcl
}

/// `HousePlacement.Check(from, multiID, center, out toMove, houseFacing)`.
pub fn check(
    w: &mut World,
    from: MobileId,
    multi_id: i32,
    center: Point3D,
    house_facing: Direction,
) -> (HousePlacementResult, Vec<EntityId>) {
    let mut to_move: Vec<EntityId> = Vec::new();
    let Some(map) = w
        .mobile(from)
        .and_then(|m| m.map)
        .filter(|&m| m != MapId::INTERNAL)
    else {
        return (HousePlacementResult::BadLand, to_move); // A house cannot go here
    };
    let access = w
        .mobile(from)
        .map_or(AccessLevel::Player, |m| m.access_level);

    if deps(w)
        .and_then(|d| d.house_placement_blocked)
        .is_some_and(|f| f(w, from))
    {
        return (HousePlacementResult::BadRegionTemp, to_move);
    }
    if access >= AccessLevel::GameMaster {
        return (HousePlacementResult::Valid, to_move); // Staff can place anywhere
    }
    if map == MapId::ILSHENAR || muo_content::spells::helper::is_felucca_t2a(map, center) {
        return (HousePlacementResult::BadRegion, to_move); // No houses in Ilshenar/T2A
    }
    if map == MapId::MALAS && (multi_id == 0x007C || multi_id == 0x007E) {
        return (HousePlacementResult::InvalidCastleKeep, to_move);
    }
    let r = region::find(w, center, Some(map));
    if region::is_part_of_either(w, r, &NO_HOUSING_REGION, &NO_HOUSING_GUARDED_REGION) {
        return (HousePlacementResult::BadRegion, to_move);
    }

    // This holds data describing the internal structure of the house
    let mcl = if (0x13EC..0x1D00).contains(&multi_id) {
        components_with_stairs(w, multi_id) // this is a AOS house, add the stairs
    } else {
        w.maps.multi_data.get_components(multi_id).clone()
    };

    // Location of the northwest-most corner of the house
    let start = Point3D::new(center.x + mcl.min().x, center.y + mcl.min().y, center.z);
    let mut borders: Vec<Point2D> = Vec::new();

    for x in 0..mcl.width() {
        for y in 0..mcl.height() {
            let tile_x = start.x + x;
            let tile_y = start.y + y;
            let Some(add_tiles) = mcl.tiles.get(x as usize).and_then(|c| c.get(y as usize)) else {
                continue;
            };
            if add_tiles.is_empty() {
                continue; // There are no tiles here, continue checking somewhere else
            }
            let test_point = Point3D::new(tile_x, tile_y, center.z);
            let reg = region::find(w, test_point, Some(map));
            // Cannot place houses in dungeons, towns, treasure map areas etc
            if !allow_housing(w, reg, from, test_point) {
                if region::is_part_of_type(w, reg, &TEMP_NO_HOUSING_REGION) {
                    return (HousePlacementResult::BadRegionTemp, to_move);
                }
                if region::is_part_of_type_name(w, reg, "TreasureRegion")
                    || region::is_part_of_type(w, reg, &muo_content::regions::house::HOUSE_REGION)
                {
                    return (HousePlacementResult::BadRegionHidden, to_move);
                }
                if region::is_part_of_type_name(w, reg, "HouseRaffleRegion") {
                    return (HousePlacementResult::BadRegionRaffle, to_move);
                }
                return (HousePlacementResult::BadRegion, to_move);
            }

            let land_tile = w.maps.land_tile(map, tile_x, tile_y);
            let land_id = i32::from(land_tile.id) & 0x3FFF;

            let items: Vec<_> = enumerate::items_at(w, Some(map), tile_x, tile_y)
                .filter(|&i| {
                    w.item(i).is_some_and(|it| {
                        it.visible() && it.location.x == tile_x && it.location.y == tile_y
                    })
                })
                .collect();
            let mobiles: Vec<_> = enumerate::mobiles_at(w, Some(map), tile_x, tile_y)
                .filter(|&m| {
                    w.mobile(m)
                        .is_some_and(|mo| mo.location.x == tile_x && mo.location.y == tile_y)
                })
                .collect();

            let (land_start_z, land_avg_z, _) = w.maps.get_average_z_full(map, tile_x, tile_y);
            let mut has_foundation = false;

            for add_tile in add_tiles {
                if add_tile.id == 0x1 {
                    continue; // Nodraw
                }
                let add_data = w
                    .maps
                    .tile_data
                    .item(usize::from(add_tile.id) & TileData::MAX_ITEM_VALUE)
                    .clone();
                let is_foundation = add_tile.z == 0 && add_data.wall();
                let mut has_surface = false;
                if is_foundation {
                    has_foundation = true;
                }
                let add_tile_z = center.z + i32::from(add_tile.z);
                let mut add_tile_top = add_tile_z + add_tile.height(&w.maps.tile_data);
                if add_data.surface() {
                    add_tile_top += 16;
                }
                if add_tile_top > land_start_z && land_avg_z > add_tile_z {
                    return (HousePlacementResult::BadLand, to_move); // Broke rule #2
                }
                if is_foundation
                    && !w
                        .maps
                        .tile_data
                        .land(usize::from(land_tile.id as u16 & 0x3FFF))
                        .flags
                        .contains(TileFlag::IMPASSABLE)
                    && land_avg_z == center.z
                {
                    has_surface = true;
                }

                for tile in tiles::static_and_multi_tiles(w, map, tile_x, tile_y) {
                    let id = w
                        .maps
                        .tile_data
                        .item(usize::from(tile.id) & TileData::MAX_ITEM_VALUE);
                    if (id.impassable() || (id.surface() && !id.background()))
                        && add_tile_top > i32::from(tile.z)
                        && i32::from(tile.z) + id.calc_height() > add_tile_z
                    {
                        return (HousePlacementResult::BadStatic, to_move); // Broke rule #2
                    }
                }

                for &item in &items {
                    let Some(it) = w.item(item) else { continue };
                    let (iz, movable) = (it.location.z, it.movable());
                    let id = crate::placement::item_data(w, item);
                    if add_tile_top <= iz || iz + id.calc_height() <= add_tile_z {
                        continue;
                    }
                    if movable {
                        to_move.push(EntityId::Item(item));
                    } else if id.impassable() || (id.surface() && !id.background()) {
                        return (HousePlacementResult::BadItem, to_move); // Broke rule #2
                    }
                }

                if is_foundation && !has_surface {
                    return (HousePlacementResult::NoSurface, to_move); // Broke rule #4
                }

                for &m in &mobiles {
                    let mz = w.mobile(m).map_or(0, |mo| mo.location.z);
                    if add_tile_top > mz && mz + 16 > add_tile_z {
                        to_move.push(EntityId::Mobile(m));
                    }
                }
            }

            if is_road(land_id) {
                return (HousePlacementResult::BadLand, to_move); // Broke rule #5
            }

            if has_foundation {
                if !check_yard(w, map, tile_x, tile_y, YARD_SIZE, house_facing) {
                    return (HousePlacementResult::BadStatic, to_move); // Broke rule #3
                }
                for x_offset in -1..=1 {
                    for y_offset in -1..=1 {
                        if x_offset == 0 && y_offset == 0 {
                            continue;
                        }
                        // To ease this rule, we will not add to the border list if the tile here is under a base
                        // floor (z<=8)
                        let (vx, vy) = (x + x_offset, y + y_offset);
                        if vx >= 0 && vx < mcl.width() && vy >= 0 && vy < mcl.height() {
                            let break_tiles = &mcl.tiles[vx as usize][vy as usize];
                            let should_break = break_tiles.iter().any(|bt| {
                                bt.height(&w.maps.tile_data) == 0
                                    && bt.z <= 8
                                    && w.maps
                                        .tile_data
                                        .item(usize::from(bt.id) & TileData::MAX_ITEM_VALUE)
                                        .surface()
                            });
                            if should_break {
                                continue;
                            }
                        }
                        let border = Point2D::new(tile_x + x_offset, tile_y + y_offset);
                        if !borders.contains(&border) {
                            borders.push(border);
                        }
                    }
                }
            }
        }
    }

    for border in borders {
        let land_tile = w.maps.land_tile(map, border.x, border.y);
        let land_id = usize::from(land_tile.id as u16 & 0x3FFF);
        if w.maps
            .tile_data
            .land(land_id)
            .flags
            .contains(TileFlag::IMPASSABLE)
        {
            return (HousePlacementResult::BadLand, to_move);
        }
        if is_road(land_id as i32) {
            return (HousePlacementResult::BadLand, to_move); // Broke rule #5
        }
        for tile in tiles::static_and_multi_tiles(w, map, border.x, border.y) {
            let id = w
                .maps
                .tile_data
                .item(usize::from(tile.id) & TileData::MAX_ITEM_VALUE);
            if id.impassable()
                || (id.surface()
                    && !id.background()
                    && i32::from(tile.z) + id.calc_height() > center.z + 2)
            {
                return (HousePlacementResult::BadStatic, to_move); // Broke rule #1
            }
        }
        let at: Vec<_> = enumerate::items_at(w, Some(map), border.x, border.y).collect();
        for item in at {
            if w.item(item).is_none_or(|i| i.movable()) {
                continue;
            }
            let z = w.item(item).map_or(0, |i| i.location.z);
            let id = item_data(w, item);
            if id.impassable()
                || (id.surface() && !id.background() && z + id.calc_height() > center.z + 2)
            {
                return (HousePlacementResult::BadItem, to_move); // Broke rule #1
            }
        }
    }

    (HousePlacementResult::Valid, to_move)
}

fn item_data(w: &World, item: muo_server::ids::ItemId) -> muo_data::tile_data::ItemData {
    muo_server::item::ops::item_data(w, item).clone()
}

fn check_yard(
    w: &World,
    map: MapId,
    tile_x: i32,
    tile_y: i32,
    yard_size: i32,
    house_facing: Direction,
) -> bool {
    let is_south_facing = house_facing.contains(Direction::SOUTH);
    let is_east_facing = house_facing.contains(Direction::EAST);

    for x_offset in -yard_size..=yard_size {
        let abs_x = x_offset.abs();
        for y_offset in -yard_size..=yard_size {
            let abs_y = y_offset.abs();
            let yard_point = Point2D::new(tile_x + x_offset, tile_y + y_offset);

            let in_south_yard = y_offset > 0 && y_offset <= yard_size && abs_x <= 1;
            let in_east_yard = x_offset > 0 && x_offset <= yard_size && abs_y <= 1;
            let in_north_yard = y_offset < 0 && y_offset >= -yard_size && abs_x <= 1;
            let in_west_yard = x_offset < 0 && x_offset >= -yard_size && abs_y <= 1;

            // Check each house at this point
            for h in enumerate::multis_in_sector(w, Some(map), yard_point.x, yard_point.y) {
                if !house::is_house(w, h) || !multi::contains_point2d(w, h, yard_point) {
                    continue;
                }
                let existing = house::class(w, h).house_direction;
                let existing_south = existing.contains(Direction::SOUTH);
                let existing_east = existing.contains(Direction::EAST);

                // Sub-Rule 1: No houses within immediate proximity (1 tile radius)
                if abs_x <= 1 && abs_y <= 1 {
                    return false;
                }
                // Sub-Rule 2: If we're south facing, protect our south yard
                if is_south_facing && in_south_yard {
                    return false;
                }
                // Sub-Rule 3: If we're east facing, protect our east yard
                if is_east_facing && in_east_yard {
                    return false;
                }
                // Sub-Rule 4: If there's a south-facing house to our north, respect its yard
                if in_north_yard && existing_south {
                    return false;
                }
                // Sub-Rule 5: If there's an east-facing house to our west, respect its yard
                if in_west_yard && existing_east {
                    return false;
                }
            }
        }
    }
    true
}
