//! The placement queries of the map: `CanFit`, `CanFitItem`, `CanSpawnMobile`,
//! `CanSpawnItem`, `GetRandomNearbyLocation`.

use muo_data::TileFlag;
use muo_primitives::{BitMask256, Point3D};

use super::tiles::{at_world_point, item_data, static_and_multi_tiles};
use super::{MapId, SpawnFailureReason};
use crate::World;
use crate::ids::ItemId;
use crate::mobile::AccessLevel;

fn out_of_map(w: &World, map: MapId, x: i32, y: i32) -> bool {
    match w.maps.get(map) {
        Some(m) => x < 0 || y < 0 || x >= m.width || y >= m.height,
        None => true,
    }
}

fn sector_items(w: &World, map: MapId, x: i32, y: i32) -> Vec<ItemId> {
    w.maps
        .get(map)
        .map_or_else(Vec::new, |m| m.sector_at(x, y).items.clone())
}

/// `Map.CanFit(x, y, z, height, checkBlocksFit, checkMobiles, requireSurface)`.
#[allow(clippy::too_many_arguments)]
pub fn can_fit(
    w: &mut World,
    map: MapId,
    x: i32,
    y: i32,
    z: i32,
    height: i32,
    check_blocks_fit: bool,
    check_mobiles: bool,
    require_surface: bool,
) -> bool {
    if map == MapId::INTERNAL {
        return false;
    }
    if out_of_map(w, map, x, y) {
        return false;
    }

    let mut has_surface = false;

    let lt = w.maps.land_tile(map, x, y);
    let (low_z, avg_z, _) = w.maps.get_average_z_full(map, x, y);
    let land_flags = w
        .maps
        .tile_data
        .land(usize::from(lt.id as u16 & 0x3FFF))
        .flags;

    if land_flags.contains(TileFlag::IMPASSABLE) && avg_z > z && z + height > low_z {
        return false;
    }

    if !land_flags.contains(TileFlag::IMPASSABLE) && z == avg_z && !lt.ignored() {
        has_surface = true;
    }

    for tile in static_and_multi_tiles(w, map, x, y) {
        let id = w.maps.tile_data.item(usize::from(tile.id));
        let surface = id.surface();
        let impassable = id.impassable();

        if (surface || impassable)
            && i32::from(tile.z) + id.calc_height() > z
            && z + height > i32::from(tile.z)
        {
            return false;
        }

        if surface && !impassable && z == i32::from(tile.z) + id.calc_height() {
            has_surface = true;
        }
    }

    for id in sector_items(w, map, x, y) {
        let Some(item) = w.item(id) else { continue };
        if super::sector::is_multi(w, id) || item.item_id > 0xFFFF || !at_world_point(item, x, y) {
            continue;
        }

        let data = item_data(w, item);
        let surface = data.surface();
        let impassable = data.impassable();
        let (iz, ty, movable) = (item.location.z, item.ty, item.movable());

        if (surface || impassable || check_blocks_fit && (ty.blocks_fit)(w, id))
            && iz + data.calc_height() > z
            && z + height > iz
        {
            return false;
        }

        if surface && !impassable && !movable && z == iz + data.calc_height() {
            has_surface = true;
        }
    }

    if check_mobiles {
        let mobiles = w
            .maps
            .get(map)
            .map_or_else(Vec::new, |m| m.sector_at(x, y).mobiles.clone());
        for id in mobiles {
            let Some(m) = w.mobile(id) else { continue };
            if m.location.x == x
                && m.location.y == y
                && (m.access_level == AccessLevel::Player || !m.hidden)
                && m.location.z + 16 > z
                && z + height > m.location.z
            {
                return false;
            }
        }
    }

    !require_surface || has_surface
}

/// `CanFit(p, height, ..)` with the defaults (`checkBlocksFit` false, `checkMobiles` true, `requireSurface` true).
pub fn can_fit_point(w: &mut World, map: MapId, p: Point3D, height: i32) -> bool {
    can_fit(w, map, p.x, p.y, p.z, height, false, true, true)
}

/// `Map.CanFitItem(x, y, z, height)`: like `CanFit`, but Surface+Impassable tiles (tables) are valid surfaces.
pub fn can_fit_item(w: &mut World, map: MapId, x: i32, y: i32, z: i32, height: i32) -> bool {
    if map == MapId::INTERNAL || out_of_map(w, map, x, y) {
        return false;
    }

    let mut has_surface = false;

    let lt = w.maps.land_tile(map, x, y);
    let (low_z, avg_z, _) = w.maps.get_average_z_full(map, x, y);
    let land_flags = w
        .maps
        .tile_data
        .land(usize::from(lt.id as u16 & 0x3FFF))
        .flags;

    // Impassable land still blocks items
    if land_flags.contains(TileFlag::IMPASSABLE) && avg_z > z && z + height > low_z {
        return false;
    }

    // Passable land is a valid surface
    if !land_flags.contains(TileFlag::IMPASSABLE) && z == avg_z && !lt.ignored() {
        has_surface = true;
    }

    for tile in static_and_multi_tiles(w, map, x, y) {
        let id = w.maps.tile_data.item(usize::from(tile.id));
        let surface = id.surface();
        let impassable = id.impassable();

        if (surface || impassable)
            && i32::from(tile.z) + id.calc_height() > z
            && z + height > i32::from(tile.z)
        {
            return false;
        }

        // Surface tiles (including Surface+Impassable like tables) are valid surfaces for items
        if surface && z == i32::from(tile.z) + id.calc_height() {
            has_surface = true;
        }
    }

    for id in sector_items(w, map, x, y) {
        let Some(item) = w.item(id) else { continue };
        if super::sector::is_multi(w, id) || item.item_id > 0xFFFF || !at_world_point(item, x, y) {
            continue;
        }

        let data = item_data(w, item);
        let surface = data.surface();
        let impassable = data.impassable();
        let iz = item.location.z;

        if (surface || impassable) && iz + data.calc_height() > z && z + height > iz {
            return false;
        }

        // Must be non-movable to be a stable surface
        if surface && !item.movable() && z == iz + data.calc_height() {
            has_surface = true;
        }
    }

    has_surface
}

/// `Map.CanSpawnMobile(x, y, z)`.
pub fn can_spawn_mobile_at(w: &mut World, map: MapId, x: i32, y: i32, z: i32) -> bool {
    let r = crate::region::find(w, Point3D::new(x, y, z), Some(map));
    crate::region::allow_spawn_of(w, r) && can_fit(w, map, x, y, z, 16, false, true, true)
}

/// `ApplyBlockerMask`: clears the Z levels in the exclusive world range `(block_low, block_high)`.
fn apply_blocker_mask(mask: &mut BitMask256, block_low: i32, block_high: i32, min_z: i32) {
    let start_bit = block_low - min_z + 1;
    let end_bit = block_high - min_z - 1;

    if end_bit < 0 || start_bit > 255 || start_bit > end_bit {
        return;
    }

    mask.clear_range(start_bit.max(0), end_bit.min(255));
}

/// The spawn Z `CanSpawnMobile` found, or why not.
pub type SpawnResult = Result<i32, SpawnFailureReason>;

/// `Map.CanSpawnMobile(x, y, minZ, maxZ, canSwim, cantWalk, out spawnZ, out failureReason)`: the lowest valid
/// surface in `[min_z, max_z]` that nothing blocks.
#[allow(clippy::too_many_arguments)]
pub fn can_spawn_mobile(
    w: &mut World,
    map: MapId,
    x: i32,
    y: i32,
    min_z: i32,
    max_z: i32,
    can_swim: bool,
    cant_walk: bool,
) -> SpawnResult {
    if map == MapId::INTERNAL || out_of_map(w, map, x, y) {
        return Err(SpawnFailureReason::INVALID_MAP);
    }

    let region = crate::region::find(w, Point3D::new(x, y, min_z), Some(map));
    if !crate::region::allow_spawn_of(w, region) {
        return Err(SpawnFailureReason::REGION_BLOCKED);
    }

    // Each bit is a Z level relative to minZ: open_slots set = not blocked, surfaces set = a valid surface.
    let mut open_slots = BitMask256::ALL_SET;
    let mut surfaces = BitMask256::ALL_CLEAR;

    let mut has_non_transient_blocker = false;
    let mut has_transient_blocker = false;

    // 1. Land tile
    let land_tile = w.maps.land_tile(map, x, y);
    let (low_z, avg_z, _) = w.maps.get_average_z_full(map, x, y);

    if !land_tile.ignored() {
        let land_flags = w
            .maps
            .tile_data
            .land(usize::from(land_tile.id as u16 & 0x3FFF))
            .flags;
        let is_impassable = land_flags.contains(TileFlag::IMPASSABLE);
        let is_wet = land_flags.contains(TileFlag::WET);

        // Impassable land blocks, except water tiles don't block swimming mobs
        if is_impassable && !(can_swim && is_wet) {
            apply_blocker_mask(&mut open_slots, low_z - 16, avg_z, min_z);
            has_non_transient_blocker = true;
        }

        if avg_z >= min_z && avg_z <= max_z && (can_swim && is_wet || !cant_walk && !is_impassable)
        {
            surfaces.set_bit(avg_z - min_z);
        }
    }

    // 2. Static and multi tiles (always non-transient)
    for tile in static_and_multi_tiles(w, map, x, y) {
        let id = w.maps.tile_data.item(usize::from(tile.id));
        let tile_z = i32::from(tile.z);
        let tile_top = tile_z + id.calc_height();
        let is_surface = id.surface();
        let is_impassable = id.impassable();
        let is_wet = id.wet();

        if (is_surface || is_impassable) && !(can_swim && is_wet) {
            apply_blocker_mask(&mut open_slots, tile_z - 16, tile_top, min_z);
            has_non_transient_blocker = true;
        }

        if tile_top >= min_z
            && tile_top <= max_z
            && (can_swim && is_wet || !cant_walk && is_surface && !is_impassable)
        {
            surfaces.set_bit(tile_top - min_z);
        }
    }

    // 3. World items
    for id in sector_items(w, map, x, y) {
        let Some(item) = w.item(id) else { continue };
        if super::sector::is_multi(w, id) || item.item_id > 0xFFFF || !at_world_point(item, x, y) {
            continue;
        }

        let data = item_data(w, item);
        let iz = item.location.z;
        let item_top = iz + data.calc_height();
        let is_surface = data.surface();
        let is_impassable = data.impassable();
        let is_wet = data.wet();
        let (ty, movable) = (item.ty, item.movable());

        if (is_surface || is_impassable) && !(can_swim && is_wet) {
            apply_blocker_mask(&mut open_slots, iz - 16, item_top, min_z);

            // Movable items are transient, non-movable are permanent
            if movable || (ty.can_decay)(w, id) {
                has_transient_blocker = true;
            } else {
                has_non_transient_blocker = true;
            }
        }

        if !movable
            && item_top >= min_z
            && item_top <= max_z
            && (can_swim && is_wet || !cant_walk && is_surface && !is_impassable)
        {
            surfaces.set_bit(item_top - min_z);
        }
    }

    // 4. Mobiles (always transient blockers)
    let mobiles = w
        .maps
        .get(map)
        .map_or_else(Vec::new, |m| m.sector_at(x, y).mobiles.clone());
    for id in mobiles {
        let Some(m) = w.mobile(id) else { continue };
        if m.location.x == x
            && m.location.y == y
            && (m.access_level == AccessLevel::Player || !m.hidden)
        {
            apply_blocker_mask(&mut open_slots, m.location.z - 16, m.location.z + 16, min_z);
            has_transient_blocker = true;
        }
    }

    let lowest = surfaces.and(&open_slots).lowest_set_bit();

    if lowest < 0 {
        let mut reason = SpawnFailureReason::empty();
        // No surface at all is also a non-transient issue (map geometry)
        if has_non_transient_blocker || surfaces.is_empty() {
            reason |= SpawnFailureReason::NON_TRANSIENT_BLOCKER;
        }
        if has_transient_blocker {
            reason |= SpawnFailureReason::TRANSIENT_BLOCKER;
        }
        return Err(reason);
    }

    Ok(min_z + lowest)
}

/// `Map.CanSpawnItem(x, y, minZ, maxZ, out spawnZ)`: as `can_spawn_mobile`, but Surface+Impassable tiles
/// (tables) are valid surfaces.
pub fn can_spawn_item(
    w: &mut World,
    map: MapId,
    x: i32,
    y: i32,
    min_z: i32,
    max_z: i32,
) -> Option<i32> {
    if map == MapId::INTERNAL || out_of_map(w, map, x, y) {
        return None;
    }

    let region = crate::region::find(w, Point3D::new(x, y, min_z), Some(map));
    if !crate::region::allow_spawn_of(w, region) {
        return None;
    }

    let mut open_slots = BitMask256::ALL_SET;
    let mut surfaces = BitMask256::ALL_CLEAR;

    let land_tile = w.maps.land_tile(map, x, y);
    let (low_z, avg_z, _) = w.maps.get_average_z_full(map, x, y);

    if !land_tile.ignored() {
        let land_flags = w
            .maps
            .tile_data
            .land(usize::from(land_tile.id as u16 & 0x3FFF))
            .flags;
        let is_impassable = land_flags.contains(TileFlag::IMPASSABLE);

        if is_impassable {
            apply_blocker_mask(&mut open_slots, low_z - 16, avg_z, min_z);
        }

        if !is_impassable && avg_z >= min_z && avg_z <= max_z {
            surfaces.set_bit(avg_z - min_z);
        }
    }

    for tile in static_and_multi_tiles(w, map, x, y) {
        let id = w.maps.tile_data.item(usize::from(tile.id));
        let tile_z = i32::from(tile.z);
        let tile_top = tile_z + id.calc_height();
        let is_surface = id.surface();
        let is_impassable = id.impassable();

        if is_surface || is_impassable {
            apply_blocker_mask(&mut open_slots, tile_z - 16, tile_top, min_z);
        }

        if is_surface && tile_top >= min_z && tile_top <= max_z {
            surfaces.set_bit(tile_top - min_z);
        }
    }

    for id in sector_items(w, map, x, y) {
        let Some(item) = w.item(id) else { continue };
        if super::sector::is_multi(w, id) || item.item_id > 0xFFFF || !at_world_point(item, x, y) {
            continue;
        }

        let data = item_data(w, item);
        let iz = item.location.z;
        let item_top = iz + data.calc_height();
        let is_surface = data.surface();
        let is_impassable = data.impassable();

        if is_surface || is_impassable {
            apply_blocker_mask(&mut open_slots, iz - 16, item_top, min_z);
        }

        if !item.movable() && is_surface && item_top >= min_z && item_top <= max_z {
            surfaces.set_bit(item_top - min_z);
        }
    }

    let lowest = surfaces.and(&open_slots).lowest_set_bit();
    (lowest >= 0).then_some(min_z + lowest)
}

/// `Map.GetRandomNearbyLocation(loc, maxRange, minRange, retryCount, height, checkBlocksFit, checkMobiles)`.
/// Usual arguments: 2, 0, 10, 16, false, false.
#[allow(clippy::too_many_arguments)]
pub fn get_random_nearby_location(
    w: &mut World,
    map: MapId,
    mut loc: Point3D,
    max_range: i32,
    min_range: i32,
    retry_count: i32,
    height: i32,
    check_blocks_fit: bool,
    check_mobiles: bool,
) -> Point3D {
    let range = max_range - min_range;
    if range < 0 {
        // Undefined (negative array size / negative index); callers never pass this.
        return loc;
    }
    let mut j = 0;
    // `new bool[range + 1, range + 1]` for small ranges: which cells were already tried
    let mut locs = (range <= 10).then(|| vec![false; ((range + 1) * (range + 1)) as usize]);

    loop {
        let x_rand = muo_primitives::utility::random(&mut w.rng, range);
        let y_rand = muo_primitives::utility::random(&mut w.rng, range);

        let cell = (x_rand * (range + 1) + y_rand) as usize;
        if locs.as_ref().is_none_or(|l| !l[cell]) {
            let x = loc.x + x_rand + min_range;
            let y = loc.y + y_rand + min_range;

            if can_fit(
                w,
                map,
                x,
                y,
                loc.z,
                height,
                check_blocks_fit,
                check_mobiles,
                true,
            ) {
                loc = Point3D::new(x, y, loc.z);
                break;
            }

            let z = w.maps.get_average_z(map, x, y);

            if can_fit(
                w,
                map,
                x,
                y,
                z,
                height,
                check_blocks_fit,
                check_mobiles,
                true,
            ) {
                loc = Point3D::new(x, y, z);
                break;
            }

            if let Some(l) = locs.as_mut() {
                l[cell] = true;
            }
        }

        j += 1;
        if j >= retry_count {
            break;
        }
    }

    loc
}
