//! The surface queries of the map: `FixColumn`, `GetTopSurface`, `GetTopSurfaceZ`.

use muo_data::tile_matrix::{LandTile, StaticTile};
use muo_primitives::Point3D;

use super::MapId;
use super::tiles::{at_world_point, item_data, static_and_multi_tiles};
use crate::World;
use crate::ids::ItemId;

/// What `GetTopSurface` returns: a land tile, a static tile or an item.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Surface {
    Land(LandTile),
    Static(StaticTile),
    Item(ItemId),
}

/// `Map.AcquireFixItems`: the (at most 128) non-multi items standing at `(x, y)`, sorted by Z.
fn acquire_fix_items(w: &World, map: MapId, x: i32, y: i32) -> Vec<ItemId> {
    let Some(m) = w.maps.get(map) else {
        return Vec::new();
    };
    if map == MapId::INTERNAL || x < 0 || x > m.width || y < 0 || y > m.height {
        return Vec::new();
    }
    let mut pool: Vec<ItemId> = Vec::new();
    for id in super::enumerate::items_at(w, Some(map), x, y) {
        if !super::sector::is_multi(w, id) && w.item(id).is_some_and(|i| i.item_id <= 0xFFFF) {
            if pool.len() == 128 {
                break;
            }
            pool.push(id);
        }
    }
    // Array.Sort(ZComparer): an unstable introsort, so equal z come out in its fixed order.
    muo_primitives::dotnet::net_sort(&mut pool, |&a, &b| {
        let z = |id| w.item(id).map_or(0, |i| i.location.z);
        z(a).cmp(&z(b))
    });
    pool
}

/// `Map.FixColumn(x, y)`: drops movable items that hover over the ground onto what is below them.
pub fn fix_column(w: &mut World, map: MapId, x: i32, y: i32) {
    let land_tile = w.maps.land_tile(map, x, y);
    let (_, land_avg, _) = w.maps.get_average_z_full(map, x, y);

    let items = acquire_fix_items(w, map, x, y);

    for i in 0..items.len() {
        let to_fix = items[i];
        let Some(it) = w.item(to_fix) else { continue };
        if !it.movable() {
            continue;
        }

        let mut z = i32::MIN;
        let current_z = it.location.z;

        if !land_tile.ignored() && land_avg <= current_z {
            z = land_avg;
        }

        for tile in static_and_multi_tiles(w, map, x, y) {
            let id = w.maps.tile_data.item(usize::from(tile.id)).clone();

            let check_z = i32::from(tile.z);
            let mut check_top = check_z + id.calc_height();

            if check_top == check_z && !id.surface() {
                check_top += 1;
            }

            if check_top > z && check_top <= current_z {
                z = check_top;
            }
        }

        for (j, &other) in items.iter().enumerate() {
            if j == i {
                continue;
            }
            let Some(item) = w.item(other) else { continue };
            let id = item_data(w, item);

            let check_z = item.location.z;
            let mut check_top = check_z + id.calc_height();

            if check_top == check_z && !id.surface() {
                check_top += 1;
            }

            if check_top > z && check_top <= current_z {
                z = check_top;
            }
        }

        if z != i32::MIN
            && let Some(l) = w.item(to_fix).map(|it| it.location)
        {
            crate::item::ops::set_location(w, to_fix, Point3D::new(l.x, l.y, z));
        }
    }
}

/// Shared walk of `GetTopSurface`/`GetTopSurfaceZ`: `(surface, surface_z)`, stopping at an exact hit.
fn top_surface(w: &mut World, map: MapId, p: Point3D) -> (Option<Surface>, i32) {
    let mut surface = None;
    let mut surface_z = i32::MIN;

    let lt = w.maps.land_tile(map, p.x, p.y);

    if !lt.ignored() {
        let avg_z = w.maps.get_average_z(map, p.x, p.y);

        if avg_z <= p.z {
            surface = Some(Surface::Land(lt));
            surface_z = avg_z;

            if surface_z == p.z {
                return (surface, surface_z);
            }
        }
    }

    for tile in static_and_multi_tiles(w, map, p.x, p.y) {
        let id = w.maps.tile_data.item(usize::from(tile.id));

        if id.surface() || id.wet() {
            let tile_z = i32::from(tile.z) + id.calc_height();

            if tile_z > surface_z && tile_z <= p.z {
                surface = Some(Surface::Static(tile));
                surface_z = tile_z;

                if surface_z == p.z {
                    return (surface, surface_z);
                }
            }
        }
    }

    let sector_items: Vec<ItemId> = w
        .maps
        .get(map)
        .map_or_else(Vec::new, |m| m.sector_at(p.x, p.y).items.clone());

    for id in sector_items {
        let Some(item) = w.item(id) else { continue };
        if super::sector::is_multi(w, id)
            || item.item_id > 0xFFFF
            || !at_world_point(item, p.x, p.y)
            || item.movable()
        {
            continue;
        }

        let data = item_data(w, item);

        if data.surface() || data.wet() {
            let item_z = item.location.z + data.calc_height();

            if item_z > surface_z && item_z <= p.z {
                surface = Some(Surface::Item(id));
                surface_z = item_z;

                if surface_z == p.z {
                    return (surface, surface_z);
                }
            }
        }
    }

    (surface, surface_z)
}

/// `Map.GetTopSurface(p)`: the highest surface that is not above `p`.
pub fn get_top_surface(w: &mut World, map: MapId, p: Point3D) -> Option<Surface> {
    if map == MapId::INTERNAL {
        return None;
    }
    top_surface(w, map, p).0
}

/// `Map.GetTopSurfaceZ(p)`: the Z of that surface, or `p.z` if there is none.
pub fn get_top_surface_z(w: &mut World, map: MapId, p: Point3D) -> i32 {
    if map == MapId::INTERNAL {
        return p.z;
    }
    let (_, z) = top_surface(w, map, p);
    if z == i32::MIN { p.z } else { z }
}
