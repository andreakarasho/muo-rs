//! `BaseMulti`: the item every house, boat and other multi-tile structure derives from.
//!
//! Save layout (`[SerializationGenerator(0, false)]`, no fields): `Item`'s part, then the version `int` 0.
//! [`register`] tells the map which class is the multi and how to ask an item for its footprint.

use muo_data::client::multi_data::MultiComponentList;
use muo_io::{GenericReader, GenericWriter, Result};
use muo_primitives::{Point2D, Point3D, Rectangle2D};

use crate::ids::ItemId;
use crate::item::ItemType;
use crate::item::item_type::ITEM_BASE;
use crate::item::ops;
use crate::world::World;

/// `Server.Items.BaseMulti` (abstract).
pub static BASE_MULTI: ItemType = ItemType {
    full_name: "Server.Items.BaseMulti",
    parent: Some(&ITEM_BASE),
    construct: None,
    set_item_id,
    label_number,
    get_max_update_range: |_, _| 22,
    get_update_range: |_, _, _| 22,
    on_location_change: |w, id, old| {
        (ITEM_BASE.on_location_change)(w, id, old);
        reset_cache(w, id);
    },
    on_map_change: |w, id| {
        (ITEM_BASE.on_map_change)(w, id);
        reset_cache(w, id);
    },
    serialize,
    deserialize,
    ..ITEM_BASE
};

/// `MultiInteriorCacheState`: whether a multi's footprint is eligible for the pathfinding interior-mask cache.
/// Recomputed when it is `Unknown` (after a move resets it).
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash)]
#[repr(u8)]
pub enum MultiInteriorCacheState {
    /// Not yet determined; recompute on next use.
    #[default]
    Unknown = 0,
    /// Whole footprint terrain is below the floor: interior cells may serve from the cache.
    Clean = 1,
    /// Terrain intrudes into the footprint: fall back to live synthesis.
    Dirty = 2,
}

/// Makes `BaseMulti` the map's multi class and `Components` its footprint lookup (what the map finds by `is
/// BaseMulti` and by calling the virtual).
pub fn register(w: &mut World) {
    w.maps.base_multi = Some(&BASE_MULTI);
    w.maps.multi_components = components;
}

/// `Components` through the vtable.
pub fn components(w: &World, id: ItemId) -> &MultiComponentList {
    match w.item(id) {
        Some(i) => (i.ty.components)(w, id),
        None => MultiComponentList::empty(),
    }
}

/// `Components` of `BaseMulti`: `MultiData.GetComponents(ItemID)`.
pub fn components_default(w: &World, id: ItemId) -> &MultiComponentList {
    let item_id = w.item(id).map_or(0, |i| i.item_id);
    w.maps.multi_data.get_components(item_id)
}

fn reset_cache(w: &mut World, id: ItemId) {
    if let Some(i) = w.item_mut(id) {
        i.path_interior_cache_state = MultiInteriorCacheState::Unknown;
    }
}

/// `new BaseMulti(itemID)` of class `ty`: not movable.
pub fn new_base_multi(w: &mut World, ty: &'static ItemType, item_id: i32) -> ItemId {
    let id = ops::new_item_with_id(w, ty, item_id);
    ops::set_movable(w, id, false);
    id
}

/// `ItemID` setter: the map's sectors know a multi by its footprint, so it leaves and re-enters.
fn set_item_id(w: &mut World, id: ItemId, value: i32) {
    let Some(i) = w.item(id) else { return };
    if i.item_id == value {
        return;
    }
    let map = i.map;
    if let Some(m) = map {
        crate::item::deps::map_on_leave(w, m, id);
    }
    crate::item::base::set_item_id_base(w, id, value);
    if let Some(m) = w.item(id).and_then(|i| i.map) {
        crate::item::deps::map_on_enter(w, m, id);
    }

    // The footprint shape changes with ItemID (e.g. a boat's heading swaps the MCL), so the pathfinding
    // interior-cache clean/dirty status must be recomputed.
    reset_cache(w, id);
}

/// The first component's graphic.
fn label_number(w: &World, id: ItemId) -> i32 {
    let mcl = components(w, id);
    if let Some(first) = mcl.list.first() {
        let id = i32::from(first.item_id);
        return if id < 0x4000 {
            1_020_000 + id
        } else {
            1_078_872 + id
        };
    }
    crate::item::base::label_number_base(w, id)
}

/// `Contains(x, y)` (virtual) through the vtable.
pub fn contains(w: &World, id: ItemId, x: i32, y: i32) -> bool {
    w.item(id)
        .is_some_and(|i| (i.ty.multi_contains)(w, id, x, y))
}

/// `Contains(Point2D)`.
pub fn contains_point2d(w: &World, id: ItemId, p: Point2D) -> bool {
    contains(w, id, p.x, p.y)
}

/// `Contains(Point3D)`.
pub fn contains_point3d(w: &World, id: ItemId, p: Point3D) -> bool {
    contains(w, id, p.x, p.y)
}

pub fn contains_default(w: &World, id: ItemId, x: i32, y: i32) -> bool {
    let Some(i) = w.item(id) else { return false };
    let mcl = components(w, id);
    let x = x - (i.location.x + mcl.min().x);
    let y = y - (i.location.y + mcl.min().y);

    x >= 0
        && x < mcl.width()
        && y >= 0
        && y < mcl.height()
        && mcl
            .tiles
            .get(x as usize)
            .and_then(|col| col.get(y as usize))
            .is_some_and(|t| !t.is_empty())
}

/// `Contains(Mobile m)`: on the same map and inside the footprint.
pub fn contains_mobile(w: &World, id: ItemId, m: crate::ids::MobileId) -> bool {
    let (Some(item), Some(mob)) = (w.item(id), w.mobile(m)) else {
        return false;
    };
    mob.map == item.map && contains(w, id, mob.location.x, mob.location.y)
}

/// `Contains(Item item)`.
pub fn contains_item(w: &World, id: ItemId, other: ItemId) -> bool {
    let (Some(item), Some(o)) = (w.item(id), w.item(other)) else {
        return false;
    };
    o.map == item.map && contains(w, id, o.location.x, o.location.y)
}

/// `Intersects(bounds)`.
pub fn intersects(w: &World, id: ItemId, bounds: Rectangle2D) -> bool {
    let Some(i) = w.item(id) else { return false };
    let loc = i.location;
    let c = components(w, id);
    let (min, max) = (c.min(), c.max());

    if bounds.x() > loc.x + max.x || bounds.x() + bounds.width() < loc.x + min.x {
        return false;
    }
    if bounds.y() > loc.y + max.y || bounds.y() + bounds.height() < loc.y + min.y {
        return false;
    }

    let min_x = bounds.x().max(loc.x + min.x);
    let max_x = (bounds.x() + bounds.width()).min(loc.x + max.x);
    let min_y = bounds.y().max(loc.y + min.y);
    let max_y = (bounds.y() + bounds.height()).min(loc.y + max.y);

    for x in min_x..=max_x {
        for y in min_y..=max_y {
            let offset_x = x - loc.x - min.x;
            let offset_y = y - loc.y - min.y;

            if offset_x < 0 || offset_y < 0 || offset_x >= c.width() || offset_y >= c.height() {
                continue;
            }

            if c.tiles
                .get(offset_x as usize)
                .and_then(|col| col.get(offset_y as usize))
                .is_some_and(|t| !t.is_empty())
            {
                return true;
            }
        }
    }

    false
}

fn serialize(w: &World, id: ItemId, writer: &mut dyn GenericWriter) {
    (ITEM_BASE.serialize)(w, id, writer);
    writer.write_i32(0); // version
}

fn deserialize(w: &mut World, id: ItemId, reader: &mut dyn GenericReader) -> Result<()> {
    (ITEM_BASE.deserialize)(w, id, reader)?;
    let _version = reader.read_i32()?;
    Ok(())
}
