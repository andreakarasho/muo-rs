//! `SectorSpawnCache`: the global per-sector cache of valid spawn positions
//! (separate land and water bitmasks), the house check and the spiral scan.

use std::collections::HashMap;

use muo_content::regions::house::{self, HouseRegionHooks};
use muo_primitives::{BitMask256, Point2D, Point3D, Rectangle2D, Rectangle3D};
use muo_server::map::{MapId, SECTOR_SHIFT, SECTOR_SIZE, fit};
use muo_server::world::World;

use muo_content::hooks::rnd;

/// `_landCaches` and `_waterCaches`.
#[derive(Default)]
pub struct SectorSpawnCaches {
    land: HashMap<(MapId, i32, i32), BitMask256>,
    water: HashMap<(MapId, i32, i32), BitMask256>,
}

impl SectorSpawnCaches {
    fn caches(&mut self, water: bool) -> &mut HashMap<(MapId, i32, i32), BitMask256> {
        if water {
            &mut self.water
        } else {
            &mut self.land
        }
    }

    /// `CachedSectorCount`.
    pub fn cached_sector_count(&self) -> usize {
        self.land.len() + self.water.len()
    }

    /// `LandCacheCount`.
    pub fn land_cache_count(&self) -> usize {
        self.land.len()
    }

    /// `WaterCacheCount`.
    pub fn water_cache_count(&self) -> usize {
        self.water.len()
    }

    /// `ClearAll()`.
    pub fn clear_all(&mut self) {
        self.land.clear();
        self.water.clear();
    }
}

/// `SetValid(map, pos, isWater)`.
pub fn set_valid(w: &mut World, map: MapId, pos: Point3D, is_water: bool) {
    let (sx, sy) = (pos.x >> SECTOR_SHIFT, pos.y >> SECTOR_SHIFT);
    let bit = (pos.x & (SECTOR_SIZE - 1)) + ((pos.y & (SECTOR_SIZE - 1)) << SECTOR_SHIFT);
    w.resources
        .get_mut::<SectorSpawnCaches>()
        .caches(is_water)
        .entry((map, sx, sy))
        .or_default()
        .set_bit(bit);
}

/// `TryGetRandomPosition(map, allBounds, isWater, out pos, out containingBounds, maxRetries)`.
pub fn try_get_random_position(
    w: &mut World,
    map: MapId,
    all_bounds: &[Rectangle3D],
    is_water: bool,
) -> Option<(Point2D, Rectangle3D)> {
    const MAX_RETRIES: i32 = 5;
    if all_bounds.is_empty() {
        return None;
    }

    // Unique sectors and their counts
    let mut sectors: Vec<(i32, i32, i32)> = Vec::new();
    let mut total = 0;
    {
        let caches = w.resources.get_mut::<SectorSpawnCaches>().caches(is_water);
        for b in all_bounds {
            let (start_sx, start_sy) = (b.start.x >> SECTOR_SHIFT, b.start.y >> SECTOR_SHIFT);
            let (end_sx, end_sy) = ((b.end.x - 1) >> SECTOR_SHIFT, (b.end.y - 1) >> SECTOR_SHIFT);
            for sx in start_sx..=end_sx {
                for sy in start_sy..=end_sy {
                    if sectors.iter().any(|&(x, y, _)| x == sx && y == sy) {
                        continue;
                    }
                    if let Some(cache) = caches.get(&(map, sx, sy)) {
                        let count = cache.pop_count() as i32;
                        if count > 0 {
                            sectors.push((sx, sy, count));
                            total += count;
                        }
                    }
                }
            }
        }
    }
    if total == 0 {
        return None;
    }

    for _ in 0..=MAX_RETRIES {
        let target = rnd::below(w, total);
        let mut current = 0;
        for &(sx, sy, sector_count) in &sectors {
            if target < current + sector_count {
                let Some(cache) = w
                    .resources
                    .get_mut::<SectorSpawnCaches>()
                    .caches(is_water)
                    .get(&(map, sx, sy))
                    .copied()
                else {
                    break;
                };
                let bit = cache.nth_set_bit(target - current);
                let (lx, ly) = (bit & (SECTOR_SIZE - 1), bit >> SECTOR_SHIFT);
                let pos = Point2D::new((sx << SECTOR_SHIFT) + lx, (sy << SECTOR_SHIFT) + ly);
                for b in all_bounds {
                    if pos.x >= b.start.x
                        && pos.x < b.end.x
                        && pos.y >= b.start.y
                        && pos.y < b.end.y
                    {
                        return Some((pos, *b));
                    }
                }
                break;
            }
            current += sector_count;
        }
    }
    None
}

/// `IsBlockedByHouse(map, x, y, z)`: a private house; a public AoS house lets mobiles spawn.
pub fn is_blocked_by_house(w: &World, map: MapId, x: i32, y: i32, z: i32) -> bool {
    let region = muo_server::region::find(w, Point3D::new(x, y, z), Some(map));
    let Some(house) = house::house_of(w, region) else {
        return false;
    };
    match w.resources.get::<HouseRegionHooks>().and_then(|h| h.ops) {
        Some(ops) => !(ops.is_aos_rules(w, house) && ops.public(w, house)),
        // no house code wired (tests): every house region is private
        None => true,
    }
}

/// `InvalidateSectors(map, bounds)`: houses placed or demolished.
pub fn invalidate_sectors(w: &mut World, map: MapId, bounds: Rectangle2D) {
    let (start_sx, start_sy) = (
        bounds.start.x >> SECTOR_SHIFT,
        bounds.start.y >> SECTOR_SHIFT,
    );
    let (end_sx, end_sy) = (bounds.end.x >> SECTOR_SHIFT, bounds.end.y >> SECTOR_SHIFT);
    let caches = w.resources.get_mut::<SectorSpawnCaches>();
    for sx in start_sx..=end_sx {
        for sy in start_sy..=end_sy {
            caches.land.remove(&(map, sx, sy));
            caches.water.remove(&(map, sx, sy));
        }
    }
}

/// `GetSpiralOffset(ring, position)`.
pub fn spiral_offset(ring: i32, position: i32) -> (i32, i32) {
    let side_length = ring * 2;
    let (side, side_pos) = (position / side_length, position % side_length);
    match side {
        0 => (-ring + side_pos, -ring),
        1 => (ring, -ring + side_pos),
        2 => (ring - side_pos, ring),
        3 => (-ring, ring - side_pos),
        _ => (0, 0),
    }
}

#[allow(clippy::too_many_arguments)]
fn check_and_cache_position(
    w: &mut World,
    map: MapId,
    x: i32,
    y: i32,
    min_z: i32,
    max_z: i32,
    bounds: Rectangle3D,
    can_swim: bool,
    cant_walk: bool,
) {
    if x < bounds.start.x || x >= bounds.end.x || y < bounds.start.y || y >= bounds.end.y {
        return;
    }
    if let Ok(spawn_z) = fit::can_spawn_mobile(w, map, x, y, min_z, max_z, can_swim, cant_walk) {
        if is_blocked_by_house(w, map, x, y, spawn_z) {
            return;
        }
        set_valid(w, map, Point3D::new(x, y, spawn_z), can_swim && cant_walk);
    }
}

/// `ContinueSpiralScan(...)`: scans `rings_per_tick` more rings; true when the bounds are exhausted.
#[allow(clippy::too_many_arguments)]
pub fn continue_spiral_scan(
    w: &mut World,
    map: MapId,
    center: Point3D,
    bounds: Rectangle3D,
    min_z: i32,
    max_z: i32,
    can_swim: bool,
    cant_walk: bool,
    current_ring: &mut i32,
    ring_position: &mut i32,
    rings_per_tick: i32,
) -> bool {
    let max_ring = bounds.width().max(bounds.height()) / 2 + 1;
    let rings_to_scan = (*current_ring + rings_per_tick).min(max_ring);
    while *current_ring < rings_to_scan {
        if *current_ring == 0 {
            check_and_cache_position(
                w, map, center.x, center.y, min_z, max_z, bounds, can_swim, cant_walk,
            );
        } else {
            for p in 0..*current_ring * 8 {
                let (dx, dy) = spiral_offset(*current_ring, p);
                check_and_cache_position(
                    w,
                    map,
                    center.x + dx,
                    center.y + dy,
                    min_z,
                    max_z,
                    bounds,
                    can_swim,
                    cant_walk,
                );
            }
        }
        *current_ring += 1;
    }
    *ring_position = 0;
    *current_ring >= max_ring
}
