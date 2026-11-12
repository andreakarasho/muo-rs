//! The static tile enumerator, and the small `Item` reads the map code shares
//! (`ItemData`, `AtWorldPoint`, `RootParent`, `GetWorldLocation`).

use muo_data::ItemData;
use muo_data::tile_matrix::StaticTile;
use muo_primitives::Point3D;

use super::MapId;
use crate::World;
use crate::ids::{EntityId, ItemId};
use crate::item::Item;

/// `Item.ItemData`: the tiledata row of the item's graphic.
pub fn item_data(w: &World, item: &Item) -> ItemData {
    w.maps
        .tile_data
        .item((item.item_id & 0xFFFF) as usize)
        .clone()
}

/// `Item.AtWorldPoint(x, y)`.
pub fn at_world_point(item: &Item, x: i32, y: i32) -> bool {
    item.parent.is_none() && item.location.x == x && item.location.y == y
}

/// `Item.RootParent`: the outermost container (an item with no parent is its own root, so `None`), or the
/// mobile that carries the chain.
pub fn root_parent(w: &World, id: ItemId) -> Option<EntityId> {
    let mut p = w.item(id)?.parent;
    while let Some(EntityId::Item(i)) = p {
        match w.item(i) {
            Some(it) if it.parent.is_some() => p = it.parent,
            _ => break,
        }
    }
    p
}

/// `Item.GetWorldLocation()`.
pub fn world_location(w: &World, id: ItemId) -> Option<Point3D> {
    let item = w.item(id)?;
    Some(match root_parent(w, id) {
        None => item.location,
        Some(EntityId::Item(r)) => w.item(r).map_or(item.location, |r| r.location),
        Some(EntityId::Mobile(r)) => w.mobile(r).map_or(item.location, |r| r.location),
    })
}

/// `Tiles.GetStaticBlock(..)[x & 7][y & 7]`: the statics file's tiles of one cell.
pub fn statics_at(w: &mut World, map: MapId, x: i32, y: i32) -> Vec<StaticTile> {
    match w.maps.tiles_mut(map) {
        Some(t) => t.get_static_block(x >> 3, y >> 3)[(x & 7) as usize][(y & 7) as usize].clone(),
        None => Vec::new(),
    }
}

/// The multis of the sector holding `(x, y)`, still alive: `GetMultisInSector`.
pub fn multis_in_sector(
    w: &World,
    map: MapId,
    x: i32,
    y: i32,
) -> impl Iterator<Item = ItemId> + '_ {
    let list: &[ItemId] = w.maps.get(map).map_or(&[], |m| &m.sector_at(x, y).multis);
    list.iter()
        .copied()
        .filter(move |&id| w.item(id).is_some_and(|i| !i.deleted()))
}

/// What the multis standing over `(x, y)` contribute, with `Z` raised by the multi's `Z`.
pub fn multi_tiles_at(w: &World, map: MapId, x: i32, y: i32) -> Vec<StaticTile> {
    let mut out = Vec::new();
    for id in multis_in_sector(w, map, x, y) {
        let Some(item) = w.item(id) else { continue };
        let components = (w.maps.multi_components)(w, id);
        let location = item.location;

        let offset_x = x - location.x - components.min().x;
        let offset_y = y - location.y - components.min().y;

        if offset_x >= 0
            && offset_y >= 0
            && offset_x < components.width()
            && offset_y < components.height()
        {
            for t in &components.tiles[offset_x as usize][offset_y as usize] {
                out.push(StaticTile {
                    id: t.id,
                    x: t.x,
                    y: t.y,
                    z: (i32::from(t.z) + location.z) as i8,
                    hue: t.hue,
                });
            }
        }
    }
    out
}

/// `TileMatrix.GetStaticAndMultiTiles(x, y)`: the file's statics, then each multi's.
pub fn static_and_multi_tiles(w: &mut World, map: MapId, x: i32, y: i32) -> Vec<StaticTile> {
    let mut v = statics_at(w, map, x, y);
    v.extend(multi_tiles_at(w, map, x, y));
    v
}

/// `TileMatrix.GetStaticTiles(x, y)`.
pub fn static_tiles(w: &mut World, map: MapId, x: i32, y: i32) -> Vec<StaticTile> {
    statics_at(w, map, x, y)
}

/// `TileMatrix.GetMultiTiles(x, y)`.
pub fn multi_tiles(w: &World, map: MapId, x: i32, y: i32) -> Vec<StaticTile> {
    multi_tiles_at(w, map, x, y)
}
