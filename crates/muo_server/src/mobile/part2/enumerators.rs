//! The map enumerators seen from a mobile. Each is the map's
//! enumerator at the mobile's location, empty when the mobile is on no map. The `<T>` forms are the same
//! iterator filtered by a predicate (`ty.is(&X)` in the caller). Collect before mutating the world.
//!
//! // shortcut: `GetItemsAt<T>` etc. have no generic form; callers filter by `ItemType::is`, the same items in the same order.

use muo_primitives::Point3D;

use crate::ids::{ItemId, MobileId};
use crate::map::enumerate;
use crate::net_state::NetStateId;
use crate::world::World;

fn at(w: &World, id: MobileId) -> Option<(crate::map::MapId, Point3D)> {
    let m = w.mobile(id)?;
    Some((m.map?, m.location))
}

/// `GetItemsAt()`.
pub fn get_items_at(w: &World, id: MobileId) -> Vec<ItemId> {
    match at(w, id) {
        Some((map, p)) => enumerate::items_at(w, Some(map), p.x, p.y).collect(),
        None => Vec::new(),
    }
}

/// `GetItemsInRange(range)`.
pub fn get_items_in_range(w: &World, id: MobileId, range: i32) -> Vec<ItemId> {
    match at(w, id) {
        Some((map, p)) => enumerate::items_in_range(w, Some(map), p.x, p.y, range).collect(),
        None => Vec::new(),
    }
}

/// `GetMobilesInRange()`: the no-argument overload is `GetMobilesAt` (the tile, not a range).
pub fn get_mobiles_in_range_at(w: &World, id: MobileId) -> Vec<MobileId> {
    match at(w, id) {
        Some((map, p)) => enumerate::mobiles_at(w, Some(map), p.x, p.y).collect(),
        None => Vec::new(),
    }
}

/// `GetMobilesInRange(range)`.
pub fn get_mobiles_in_range(w: &World, id: MobileId, range: i32) -> Vec<MobileId> {
    match at(w, id) {
        Some((map, p)) => enumerate::mobiles_in_range(w, Some(map), p.x, p.y, range).collect(),
        None => Vec::new(),
    }
}

/// `GetClientsAt()`. On no map this is empty.
pub fn get_clients_at(w: &World, id: MobileId) -> Vec<NetStateId> {
    match at(w, id) {
        Some((map, p)) => enumerate::clients_at(w, Some(map), p.x, p.y).collect(),
        None => Vec::new(),
    }
}

/// `GetClientsInRange(range)`.
pub fn get_clients_in_range(w: &World, id: MobileId, range: i32) -> Vec<NetStateId> {
    match at(w, id) {
        Some((map, p)) => enumerate::clients_in_range(w, Some(map), p.x, p.y, range).collect(),
        None => Vec::new(),
    }
}

/// `GetItemsInRangeByDistance(range)`: nearest sectors first, with each sector's minimum distance.
pub fn get_items_in_range_by_distance(w: &World, id: MobileId, range: i32) -> Vec<(ItemId, i32)> {
    match at(w, id) {
        Some((map, p)) => {
            enumerate::items_in_range_by_distance(w, Some(map), p.x, p.y, range).collect()
        }
        None => Vec::new(),
    }
}

/// `GetMobilesInRangeByDistance(range)`.
pub fn get_mobiles_in_range_by_distance(
    w: &World,
    id: MobileId,
    range: i32,
) -> Vec<(MobileId, i32)> {
    match at(w, id) {
        Some((map, p)) => {
            enumerate::mobiles_in_range_by_distance(w, Some(map), p.x, p.y, range).collect()
        }
        None => Vec::new(),
    }
}

/// `GetClientsInRangeByDistance(range)`.
pub fn get_clients_in_range_by_distance(
    w: &World,
    id: MobileId,
    range: i32,
) -> Vec<(NetStateId, i32)> {
    match at(w, id) {
        Some((map, p)) => {
            enumerate::clients_in_range_by_distance(w, Some(map), p.x, p.y, range).collect()
        }
        None => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mobile::MOBILE_BASE;

    #[test]
    fn a_mobile_on_no_map_enumerates_nothing() {
        let mut w = World::new();
        let id = w.new_mobile(&MOBILE_BASE);
        w.mobile_mut(id).unwrap().map = None;
        assert!(get_items_at(&w, id).is_empty());
        assert!(get_items_in_range(&w, id, 5).is_empty());
        assert!(get_mobiles_in_range(&w, id, 5).is_empty());
        assert!(get_mobiles_in_range_at(&w, id).is_empty());
        assert!(get_clients_at(&w, id).is_empty());
        assert!(get_clients_in_range_by_distance(&w, id, 5).is_empty());
    }
}
