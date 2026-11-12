//! The map enumerators: items, mobiles, clients and multis in range or bounds, and the by-distance forms.
//!
//! Each `GetXxx` is a function returning an iterator over the world; items come out in a fixed order
//! (sectors left to right, then top to bottom; inside a sector, the order they entered). A `None` map is
//! `Empty`. `T`-typed variants (`GetItemsInRange<Item subclass>`) are the same iterator filtered with
//! `item.ty.is(..)` by the caller. The iterators borrow the world; `collect()` to mutate while walking.
//!
//! The by-distance iterators walk the sectors in rings around the center sector and yield each id with the
//! smallest possible distance (in tiles, truncated) from the center to its sector (`MinDistance`).

use std::collections::HashSet;

use muo_primitives::{Point2D, Rectangle2D};

use super::{Map, MapId, SECTOR_SIZE};
use crate::World;
use crate::ids::{ItemId, MobileId};
use crate::net_state::NetStateId;

/// `GetXxxInRange(x, y, range)`: the square of side `2 * max(0, range) + 1` around `(x, y)`.
pub fn range_bounds(x: i32, y: i32, range: i32) -> Rectangle2D {
    let clamped = range.max(0);
    let edge = clamped * 2 + 1;
    Rectangle2D::new(x - clamped, y - clamped, edge, edge)
}

/// `makeBoundsInclusive`: one more tile each way.
fn inclusive(bounds: Rectangle2D, make_inclusive: bool) -> Rectangle2D {
    if make_inclusive {
        Rectangle2D::new(
            bounds.x(),
            bounds.y(),
            bounds.width() + 1,
            bounds.height() + 1,
        )
    } else {
        bounds
    }
}

fn map_of(w: &World, map: Option<MapId>) -> Option<&Map> {
    map.and_then(|id| w.maps.get(id))
}

/// The sectors `bounds` touches, in enumeration order (x fastest). Empty without a map.
fn sector_cells(m: Option<&Map>, bounds: Rectangle2D) -> impl Iterator<Item = (i32, i32)> {
    let (sx0, sy0, sx1, sy1) = match m {
        Some(m) => m.calculate_sectors(bounds),
        None => (1, 1, 0, 0),
    };
    (sy0..=sy1).flat_map(move |sy| (sx0..=sx1).map(move |sx| (sx, sy)))
}

// ---- items --------------------------------------------------------------------------------------------

/// `GetItemsAt(x, y)`: items on the ground at exactly this tile.
pub fn items_at(
    w: &World,
    map: Option<MapId>,
    x: i32,
    y: i32,
) -> impl Iterator<Item = ItemId> + '_ {
    let list: &[ItemId] = map_of(w, map).map_or(&[], |m| &m.sector_at(x, y).items);
    list.iter().copied().filter(move |&id| {
        w.item(id).is_some_and(|o| {
            !o.deleted() && o.parent.is_none() && o.location.x == x && o.location.y == y
        })
    })
}

/// `GetItemsInBounds(bounds, makeBoundsInclusive)`.
pub fn items_in_bounds(
    w: &World,
    map: Option<MapId>,
    bounds: Rectangle2D,
    make_inclusive: bool,
) -> impl Iterator<Item = ItemId> + '_ {
    let bounds = inclusive(bounds, make_inclusive);
    let m = map_of(w, map);
    sector_cells(m, bounds)
        .flat_map(move |(sx, sy)| {
            m.map_or(&[][..], |m| &m.sector(sx, sy).items[..])
                .iter()
                .copied()
        })
        .filter(move |&id| {
            w.item(id).is_some_and(|o| {
                !o.deleted() && o.parent.is_none() && bounds.contains_3d(o.location)
            })
        })
}

/// `GetItemsInRange(x, y, range)`.
pub fn items_in_range(
    w: &World,
    map: Option<MapId>,
    x: i32,
    y: i32,
    range: i32,
) -> impl Iterator<Item = ItemId> + '_ {
    items_in_bounds(w, map, range_bounds(x, y, range), false)
}

// ---- mobiles ------------------------------------------------------------------------------------------

/// `GetMobilesAt(x, y)`.
pub fn mobiles_at(
    w: &World,
    map: Option<MapId>,
    x: i32,
    y: i32,
) -> impl Iterator<Item = MobileId> + '_ {
    let list: &[MobileId] = map_of(w, map).map_or(&[], |m| &m.sector_at(x, y).mobiles);
    list.iter().copied().filter(move |&id| {
        w.mobile(id)
            .is_some_and(|o| !o.deleted && o.location.x == x && o.location.y == y)
    })
}

/// `GetMobilesInBounds(bounds, makeBoundsInclusive)`.
pub fn mobiles_in_bounds(
    w: &World,
    map: Option<MapId>,
    bounds: Rectangle2D,
    make_inclusive: bool,
) -> impl Iterator<Item = MobileId> + '_ {
    let bounds = inclusive(bounds, make_inclusive);
    let m = map_of(w, map);
    sector_cells(m, bounds)
        .flat_map(move |(sx, sy)| {
            m.map_or(&[][..], |m| &m.sector(sx, sy).mobiles[..])
                .iter()
                .copied()
        })
        .filter(move |&id| {
            w.mobile(id)
                .is_some_and(|o| !o.deleted && bounds.contains_3d(o.location))
        })
}

/// `GetMobilesInRange(x, y, range)`.
pub fn mobiles_in_range(
    w: &World,
    map: Option<MapId>,
    x: i32,
    y: i32,
    range: i32,
) -> impl Iterator<Item = MobileId> + '_ {
    mobiles_in_bounds(w, map, range_bounds(x, y, range), false)
}

// ---- clients ------------------------------------------------------------------------------------------

/// `GetClientsAt(x, y)`: the connections of mobiles standing on this tile.
pub fn clients_at(
    w: &World,
    map: Option<MapId>,
    x: i32,
    y: i32,
) -> impl Iterator<Item = NetStateId> + '_ {
    let list: &[(NetStateId, MobileId)] =
        map_of(w, map).map_or(&[], |m| &m.sector_at(x, y).clients);
    list.iter().filter_map(move |&(ns, id)| {
        w.mobile(id)
            .is_some_and(|m| !m.deleted && m.location.x == x && m.location.y == y)
            .then_some(ns)
    })
}

/// `GetClientsInBounds(bounds, makeBoundsInclusive)`.
pub fn clients_in_bounds(
    w: &World,
    map: Option<MapId>,
    bounds: Rectangle2D,
    make_inclusive: bool,
) -> impl Iterator<Item = NetStateId> + '_ {
    let bounds = inclusive(bounds, make_inclusive);
    let m = map_of(w, map);
    sector_cells(m, bounds)
        .flat_map(move |(sx, sy)| {
            m.map_or(&[][..], |m| &m.sector(sx, sy).clients[..])
                .iter()
                .copied()
        })
        .filter_map(move |(ns, id)| {
            w.mobile(id)
                .is_some_and(|m| !m.deleted && bounds.contains_3d(m.location))
                .then_some(ns)
        })
}

/// `GetClientsInRange(x, y, range)`.
pub fn clients_in_range(
    w: &World,
    map: Option<MapId>,
    x: i32,
    y: i32,
    range: i32,
) -> impl Iterator<Item = NetStateId> + '_ {
    clients_in_bounds(w, map, range_bounds(x, y, range), false)
}

// ---- by distance --------------------------------------------------------------------------------------

/// The sectors of `(sx0, sy0, sx1, sy1)` in ring order around `center`'s sector, each with its `MinDistance`.
fn ring_sectors(
    center: Point2D,
    (sx0, sy0, sx1, sy1): (i32, i32, i32, i32),
) -> Vec<(i32, i32, i32)> {
    let cx = center.x / SECTOR_SIZE;
    let cy = center.y / SECTOR_SIZE;
    let in_bounds = |x: i32, y: i32| x >= sx0 && x <= sx1 && y >= sy0 && y <= sy1;

    let dx = (cx - sx0).max(sx1 - cx);
    let dy = (cy - sy0).max(sy1 - cy);
    let max_ring = dx.max(dy);

    let mut cells = Vec::new();
    if in_bounds(cx, cy) {
        cells.push((cx, cy));
    }
    for ring in 1..=max_ring {
        let ring_size = ring * 2;
        let (start_x, start_y) = (cx - ring, cy - ring);
        for index in 0..ring * 8 {
            let (x, y) = if index <= ring_size {
                // Top edge
                (start_x + index, start_y)
            } else if index <= ring_size * 2 {
                // Right edge
                (start_x + ring_size, start_y + (index - ring_size))
            } else if index <= ring_size * 3 {
                // Bottom edge
                (
                    start_x + ring_size - (index - ring_size * 2),
                    start_y + ring_size,
                )
            } else {
                // Left edge
                (start_x, start_y + ring_size - (index - ring_size * 3))
            };
            if in_bounds(x, y) {
                cells.push((x, y));
            }
        }
    }
    cells
        .into_iter()
        .map(|(x, y)| (x, y, min_dist_to_sector(center.x, center.y, x, y)))
        .collect()
}

/// `MinDistToSectorSqrt`.
fn min_dist_to_sector(cx: i32, cy: i32, sector_x: i32, sector_y: i32) -> i32 {
    let x0 = sector_x * SECTOR_SIZE;
    let y0 = sector_y * SECTOR_SIZE;
    let x1 = x0 + (SECTOR_SIZE - 1);
    let y1 = y0 + (SECTOR_SIZE - 1);

    let dx = if cx < x0 {
        x0 - cx
    } else if cx > x1 {
        cx - x1
    } else {
        0
    };
    let dy = if cy < y0 {
        y0 - cy
    } else if cy > y1 {
        cy - y1
    } else {
        0
    };

    f64::from(dx * dx + dy * dy).sqrt() as i32
}

fn ring_cells(
    w: &World,
    map: Option<MapId>,
    bounds: Rectangle2D,
    center: Point2D,
) -> (Option<&Map>, Vec<(i32, i32, i32)>) {
    match map_of(w, map) {
        Some(m) => (Some(m), ring_sectors(center, m.calculate_sectors(bounds))),
        None => (None, Vec::new()),
    }
}

/// `GetItemsInBoundsByDistance(bounds, center, makeBoundsInclusive)`: each item with its `MinDistance`.
pub fn items_in_bounds_by_distance(
    w: &World,
    map: Option<MapId>,
    bounds: Rectangle2D,
    center: Point2D,
    make_inclusive: bool,
) -> impl Iterator<Item = (ItemId, i32)> + '_ {
    let bounds = inclusive(bounds, make_inclusive);
    let (m, cells) = ring_cells(w, map, bounds, center);
    cells
        .into_iter()
        .flat_map(move |(sx, sy, d)| {
            m.map_or(&[][..], |m| &m.sector(sx, sy).items[..])
                .iter()
                .map(move |&id| (id, d))
        })
        .filter(move |&(id, _)| {
            w.item(id)
                .is_some_and(|o| !o.deleted() && bounds.contains_3d(o.location))
        })
}

/// The default center of `GetItemsInBoundsByDistance(bounds)` without one: the middle of the bounds.
pub fn bounds_center(bounds: Rectangle2D) -> Point2D {
    Point2D::new(
        bounds.x() + bounds.width() / 2,
        bounds.y() + bounds.height() / 2,
    )
}

/// `GetItemsInRangeByDistance(x, y, range)`.
pub fn items_in_range_by_distance(
    w: &World,
    map: Option<MapId>,
    x: i32,
    y: i32,
    range: i32,
) -> impl Iterator<Item = (ItemId, i32)> + '_ {
    items_in_bounds_by_distance(w, map, range_bounds(x, y, range), Point2D::new(x, y), false)
}

/// `GetMobilesInBoundsByDistance(bounds, center, makeBoundsInclusive)`.
pub fn mobiles_in_bounds_by_distance(
    w: &World,
    map: Option<MapId>,
    bounds: Rectangle2D,
    center: Point2D,
    make_inclusive: bool,
) -> impl Iterator<Item = (MobileId, i32)> + '_ {
    let bounds = inclusive(bounds, make_inclusive);
    let (m, cells) = ring_cells(w, map, bounds, center);
    cells
        .into_iter()
        .flat_map(move |(sx, sy, d)| {
            m.map_or(&[][..], |m| &m.sector(sx, sy).mobiles[..])
                .iter()
                .map(move |&id| (id, d))
        })
        .filter(move |&(id, _)| {
            w.mobile(id)
                .is_some_and(|o| !o.deleted && bounds.contains_3d(o.location))
        })
}

/// `GetMobilesInRangeByDistance(x, y, range)`.
pub fn mobiles_in_range_by_distance(
    w: &World,
    map: Option<MapId>,
    x: i32,
    y: i32,
    range: i32,
) -> impl Iterator<Item = (MobileId, i32)> + '_ {
    mobiles_in_bounds_by_distance(w, map, range_bounds(x, y, range), Point2D::new(x, y), false)
}

/// `GetClientsInBoundsByDistance(bounds, center, makeBoundsInclusive)`.
pub fn clients_in_bounds_by_distance(
    w: &World,
    map: Option<MapId>,
    bounds: Rectangle2D,
    center: Point2D,
    make_inclusive: bool,
) -> impl Iterator<Item = (NetStateId, i32)> + '_ {
    let bounds = inclusive(bounds, make_inclusive);
    let (m, cells) = ring_cells(w, map, bounds, center);
    cells
        .into_iter()
        .flat_map(move |(sx, sy, d)| {
            m.map_or(&[][..], |m| &m.sector(sx, sy).clients[..])
                .iter()
                .map(move |&c| (c, d))
        })
        .filter_map(move |((ns, id), d)| {
            w.mobile(id)
                .is_some_and(|m| !m.deleted && bounds.contains_3d(m.location))
                .then_some((ns, d))
        })
}

/// `GetClientsInRangeByDistance(x, y, range)`.
pub fn clients_in_range_by_distance(
    w: &World,
    map: Option<MapId>,
    x: i32,
    y: i32,
    range: i32,
) -> impl Iterator<Item = (NetStateId, i32)> + '_ {
    clients_in_bounds_by_distance(w, map, range_bounds(x, y, range), Point2D::new(x, y), false)
}

// ---- multis -------------------------------------------------------------------------------------------

/// `GetMultisInSector(x, y)`: the live multis registered in the sector holding this tile.
pub fn multis_in_sector(
    w: &World,
    map: Option<MapId>,
    x: i32,
    y: i32,
) -> impl Iterator<Item = ItemId> + '_ {
    let list: &[ItemId] = map_of(w, map).map_or(&[], |m| &m.sector_at(x, y).multis);
    list.iter()
        .copied()
        .filter(move |&id| w.item(id).is_some_and(|i| !i.deleted()))
}

/// `GetMultisInBounds(bounds, makeBoundsInclusive)`. A multi is listed in every sector it spans, but its
/// location is in exactly one, which the bounds filter guarantees is visited: it is yielded from that sector
/// only. A multi missing from its location's sector (components changed after it was placed) is yielded once,
/// from the first sector that lists it.
pub fn multis_in_bounds(
    w: &World,
    map: Option<MapId>,
    bounds: Rectangle2D,
    make_inclusive: bool,
) -> impl Iterator<Item = ItemId> + '_ {
    let bounds = inclusive(bounds, make_inclusive);
    let m = map_of(w, map);
    let mut unanchored_seen: HashSet<ItemId> = HashSet::new();
    sector_cells(m, bounds)
        .flat_map(move |(sx, sy)| {
            m.map_or(&[][..], |m| &m.sector(sx, sy).multis[..])
                .iter()
                .map(move |&id| (sx, sy, id))
        })
        .filter(move |&(sx, sy, id)| {
            let Some(o) = w.item(id) else { return false };
            if o.deleted() || !bounds.contains_3d(o.location) {
                return false;
            }
            let Some(m) = m else { return false };
            let (bx, by) = m.bound(o.location.x, o.location.y);
            let (hx, hy) = (bx >> super::SECTOR_SHIFT, by >> super::SECTOR_SHIFT);
            if m.sector_key(hx, hy) == m.sector_key(sx, sy) {
                return true;
            }
            let listed_at_home = m.sector(hx, hy).multis.contains(&id);
            if listed_at_home {
                return false;
            }
            unanchored_seen.insert(id)
        })
        .map(|(_, _, id)| id)
}

/// `GetMultisInRange(x, y, range)`.
pub fn multis_in_range(
    w: &World,
    map: Option<MapId>,
    x: i32,
    y: i32,
    range: i32,
) -> impl Iterator<Item = ItemId> + '_ {
    multis_in_bounds(w, map, range_bounds(x, y, range), false)
}
