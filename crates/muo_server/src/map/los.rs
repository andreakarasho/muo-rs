//! Line of sight: `GetPoint`, `LineOfSight`.

use muo_data::TileFlag;
use muo_primitives::utility::{fix_points, in_range_3d, number_between};
use muo_primitives::{Point3D, Point3DList, Rectangle2D};

use super::MapId;
use super::enumerate::{items_at, items_in_bounds};
use super::tiles::{item_data, root_parent, static_and_multi_tiles, world_location};
use crate::World;
use crate::ids::{EntityId, ItemId, MobileId};
use crate::mobile::AccessLevel;

/// The `object` `GetPoint`/`LineOfSight` take.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum LosObject {
    Mobile(MobileId),
    Item(ItemId),
    /// A boxed `Point3D`.
    Point(Point3D),
    /// A `LandTarget` at this location.
    Land(Point3D),
    /// A `StaticTarget`.
    Static {
        x: i32,
        y: i32,
        z: i32,
        item_id: i32,
    },
    /// Any other `IPoint3D`.
    IPoint(Point3D),
}

impl LosObject {
    fn entity(self) -> Option<EntityId> {
        match self {
            LosObject::Mobile(m) => Some(EntityId::Mobile(m)),
            LosObject::Item(i) => Some(EntityId::Item(i)),
            _ => None,
        }
    }
}

/// `Map.GetPoint(o, eye)`.
// `eye` keeps the usual signature; it is only forwarded.
#[allow(clippy::only_used_in_recursion)]
pub fn get_point(w: &mut World, map: MapId, o: LosObject, eye: bool) -> Point3D {
    match o {
        LosObject::Mobile(m) => {
            let mut p = w.mobile(m).map_or(Point3D::ZERO, |m| m.location);
            p.z += 14; // eye ? 15 : 10;
            p
        }
        LosObject::Item(id) => {
            if let Some(root) = root_parent(w, id) {
                let root = match root {
                    EntityId::Item(i) => LosObject::Item(i),
                    EntityId::Mobile(m) => LosObject::Mobile(m),
                };
                get_point(w, map, root, eye)
            } else {
                let mut p = world_location(w, id).unwrap_or(Point3D::ZERO);
                let height = w.item(id).map_or(0, |i| i32::from(item_data(w, i).height));
                p.z += height / 2 + 1;
                p
            }
        }
        LosObject::Point(p) | LosObject::IPoint(p) => p,
        LosObject::Land(mut p) => {
            let (_, _, top) = w.maps.get_average_z_full(map, p.x, p.y);
            p.z = top + 1;
            p
        }
        LosObject::Static { x, y, z, item_id } => {
            let id = w.maps.tile_data.item((item_id & 0xFFFF) as usize);
            Point3D::new(x, y, z - id.calc_height() + i32::from(id.height) / 2 + 1)
        }
    }
}

/// `Map.LineOfSight(Point3D origin, Point3D destination)`.
pub fn line_of_sight(
    w: &mut World,
    map: MapId,
    mut origin: Point3D,
    mut destination: Point3D,
) -> bool {
    if map == MapId::INTERNAL {
        return false;
    }

    if !in_range_3d(origin, destination, w.maps.max_los_distance) {
        return false;
    }

    if origin == destination {
        return true;
    }

    let end = destination;

    if origin.x > destination.x
        || origin.x == destination.x && origin.y > destination.y
        || origin.x == destination.x && origin.y == destination.y && origin.z > destination.z
    {
        std::mem::swap(&mut origin, &mut destination);
    }

    let mut path = Point3DList::new();

    let xd = destination.x - origin.x;
    let yd = destination.y - origin.y;
    let zd = destination.z - origin.z;
    let mut zslp = f64::from(xd * xd + yd * yd).sqrt();
    let sq3d = if zd != 0 {
        (zslp * zslp + f64::from(zd * zd)).sqrt()
    } else {
        zslp
    };

    let rise = f64::from(yd) / sq3d;
    let run = f64::from(xd) / sq3d;
    zslp = f64::from(zd) / sq3d;

    let mut y = f64::from(origin.y);
    let mut z = f64::from(origin.z);
    let mut x = f64::from(origin.x);
    while number_between(x, destination.x, origin.x, 0.5)
        && number_between(y, destination.y, origin.y, 0.5)
        && number_between(z, destination.z, origin.z, 0.5)
    {
        // Math.Round: to even
        let ix = x.round_ties_even() as i32;
        let iy = y.round_ties_even() as i32;
        let iz = z.round_ties_even() as i32;

        match path.last() {
            Some(p) => {
                if p.x != ix || p.y != iy || p.z != iz {
                    path.add(ix, iy, iz);
                }
            }
            None => path.add(ix, iy, iz),
        }

        x += run;
        y += rise;
        z += zslp;
    }

    if path.is_empty() {
        return true; // <--should never happen, but to be safe.
    }

    if path.last() != Some(destination) {
        path.add_point(destination);
    }

    let path_count = path.len();
    let end_top = end.z + 1;

    for i in 0..path_count {
        let Some(point) = path.get(i) else { break };
        let point_top = point.z + 1;

        let land_tile = w.maps.land_tile(map, point.x, point.y);
        let (land_z, _, land_top) = w.maps.get_average_z_full(map, point.x, point.y);

        if land_z <= point_top
            && land_top >= point.z
            && (point.x != end.x || point.y != end.y || land_z > end_top || land_top < end.z)
            && !land_tile.ignored()
        {
            return false;
        }

        let lt_id = i32::from(land_tile.id);
        let mut contains = w.maps.invalid_land_tiles.contains(&lt_id);

        let mut found_static = false;

        for t in static_and_multi_tiles(w, map, point.x, point.y) {
            found_static = true;

            let id = w.maps.tile_data.item(usize::from(t.id));
            let tz = i32::from(t.z);

            if tz <= point_top
                && tz + id.calc_height() >= point.z
                && id.flags.intersects(TileFlag::WINDOW | TileFlag::NO_SHOOT)
                && (point.x != end.x
                    || point.y != end.y
                    || tz > end_top
                    || tz + id.calc_height() < end.z)
            {
                return false;
            }
        }

        if contains && !found_static {
            if items_at(w, Some(map), point.x, point.y)
                .any(|id| w.item(id).is_some_and(|i| i.visible()))
            {
                contains = false;
            }

            if contains {
                return false;
            }
        }
    }

    let mut p_top = origin;
    let mut p_bottom = destination;
    fix_points(&mut p_top, &mut p_bottom);

    let rect = Rectangle2D::new(
        p_top.x,
        p_top.y,
        p_bottom.x - p_top.x + 1,
        p_bottom.y - p_top.y + 1,
    );

    let candidates: Vec<ItemId> = items_in_bounds(w, Some(map), rect, false).collect();
    for id in candidates {
        let Some(item) = w.item(id) else { continue };
        if !item.visible() {
            continue;
        }

        if super::sector::is_multi(w, id) || item.item_id > 0xFFFF {
            continue;
        }

        let data = item_data(w, item);
        let flags = data.flags;

        if !flags.intersects(TileFlag::WINDOW | TileFlag::NO_SHOOT) {
            continue;
        }

        let item_location = item.location;

        for i in 0..path.len() {
            let Some(path_point) = path.get(i) else { break };
            let point_top = path_point.z + 1;

            if
            // Item is on same tile as this point along the LOS path
            item_location.x == path_point.x
                && item_location.y == path_point.y
                && item_location.z <= point_top

                // Item rests on the same level as the path
                && item_location.z + data.calc_height() >= path_point.z

                // Fix door bugging monsters when door is at the START or END of the LOS path by allowing LOS
                && !(flags.contains(TileFlag::DOOR) && item_location.x == origin.x && item_location.y == origin.y
                    || item_location.x == destination.x && item_location.y == destination.y)

                // Item is at some point along the path BEFORE the target
                && (item_location.x != end.x
                    || item_location.y != end.y

                    // Item is diagonally looking DOWN at the target
                    || item_location.z > end_top

                    // Item is diagonally looking UP at the target
                    || item_location.z + data.calc_height() < end.z)
            {
                return false;
            }
        }
    }

    true
}

/// `Map.LineOfSight(object from, object dest)`.
pub fn line_of_sight_objects(w: &mut World, map: MapId, from: LosObject, dest: LosObject) -> bool {
    if from.entity().is_some() && from == dest {
        return true;
    }
    if let LosObject::Mobile(m) = from
        && w.mobile(m)
            .is_some_and(|m| m.access_level > AccessLevel::Player)
    {
        return true;
    }
    if let LosObject::Item(i) = dest
        && from.entity().is_some()
        && root_parent(w, i) == from.entity()
    {
        return true;
    }
    let a = get_point(w, map, from, true);
    let b = get_point(w, map, dest, false);
    line_of_sight(w, map, a, b)
}

/// `Map.LineOfSight(Mobile from, Point3D target)`.
pub fn line_of_sight_mobile_point(
    w: &mut World,
    map: MapId,
    from: MobileId,
    target: Point3D,
) -> bool {
    let Some(m) = w.mobile(from) else {
        return false;
    };
    if m.access_level > AccessLevel::Player {
        return true;
    }

    let mut eye = m.location;
    eye.z += 14;

    line_of_sight(w, map, eye, target)
}

/// `Map.LineOfSight(Mobile from, Mobile to)`.
pub fn line_of_sight_mobiles(w: &mut World, map: MapId, from: MobileId, to: MobileId) -> bool {
    let Some(f) = w.mobile(from) else {
        return false;
    };
    if from == to || f.access_level > AccessLevel::Player {
        return true;
    }

    let Some(t) = w.mobile(to) else { return false };
    let mut eye = f.location;
    let mut target = t.location;

    eye.z += 14;
    target.z += 14; // 10;

    line_of_sight(w, map, eye, target)
}
