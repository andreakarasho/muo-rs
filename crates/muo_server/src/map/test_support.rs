//! A small world for the map tests: the maps of `Data/map-definitions.json` with empty tile matrices (every tile
//! is land id 0 at Z 0, no flags), an invented tile kit, and helpers to
//! put items, mobiles and multis on the ground.

use muo_data::client::multi_data::{MultiComponentList, MultiTileEntry};
use muo_data::map_definitions::MapDefinition;
use muo_data::tile_matrix::{LandTile, StaticTile};
use muo_data::{ItemData, LandData, TileFlag};
use muo_primitives::Point3D;

use super::{MapId, MapRules};
use crate::Ext;
use crate::World;
use crate::ids::{EntityId, ItemId, MobileId};
use crate::item::{ITEM_BASE, ItemType};
use crate::mobile::MOBILE_BASE;

pub const WALL: u16 = 0x3000;
pub const FLOOR: u16 = 0x3001;
pub const COUNTER: u16 = 0x3002;
pub const STAIR: u16 = 0x3003;
pub const BRIDGE: u16 = 0x3004;
pub const ROOF: u16 = 0x3005;
pub const DOOR: u16 = 0x3006;
pub const WINDOW: u16 = 0x3007;
pub const WATER: u16 = 0x3008;
pub const ROCK: u16 = 0x3009;
pub const PLAIN: u16 = 0x300A;

pub const GRASS: i16 = 0x3000;
pub const DIRT: i16 = 0x3002;

/// `ServerFixture.SurfaceTileId`, `ImpassableTileId`, `WetTileId`, `SurfaceImpassableTileId`.
pub const SURFACE_TILE: u16 = FLOOR;
pub const IMPASSABLE_TILE: u16 = ROCK;
pub const WET_TILE: u16 = WATER;
pub const SURFACE_IMPASSABLE_TILE: u16 = COUNTER;

/// `TestMulti`: a `BaseMulti` with caller-supplied components (in the item's `ext`).
pub static TEST_MULTI: ItemType = ItemType {
    full_name: "Server.Tests.TestMulti",
    parent: Some(&ITEM_BASE),
    ..ITEM_BASE
};

fn test_components(w: &World, id: ItemId) -> &MultiComponentList {
    w.item(id)
        .and_then(|i| i.ext.get::<MultiComponentList>())
        .unwrap_or_else(|| MultiComponentList::empty())
}

fn kit_item(w: &mut World, id: u16, name: &str, flags: TileFlag, height: u8) {
    w.maps.tile_data.items[usize::from(id)] = ItemData {
        name: name.to_string(),
        flags,
        height,
        ..Default::default()
    };
}

/// The test world: Felucca, Trammel and Internal registered, the tile kit loaded.
pub fn world() -> World {
    let mut w = World::new();

    kit_item(
        &mut w,
        WALL,
        "test wall",
        TileFlag::WALL | TileFlag::IMPASSABLE | TileFlag::NO_SHOOT,
        20,
    );
    kit_item(
        &mut w,
        FLOOR,
        "test floor",
        TileFlag::SURFACE | TileFlag::BACKGROUND,
        0,
    );
    kit_item(
        &mut w,
        COUNTER,
        "test counter",
        TileFlag::SURFACE | TileFlag::IMPASSABLE,
        4,
    );
    kit_item(
        &mut w,
        STAIR,
        "test stair",
        TileFlag::BRIDGE | TileFlag::SURFACE | TileFlag::STAIR_BACK,
        5,
    );
    kit_item(
        &mut w,
        BRIDGE,
        "test bridge",
        TileFlag::BRIDGE | TileFlag::SURFACE,
        2,
    );
    kit_item(
        &mut w,
        ROOF,
        "test roof",
        TileFlag::ROOF | TileFlag::IMPASSABLE,
        0,
    );
    kit_item(
        &mut w,
        DOOR,
        "test door",
        TileFlag::DOOR | TileFlag::IMPASSABLE | TileFlag::NO_SHOOT,
        20,
    );
    kit_item(
        &mut w,
        WINDOW,
        "test window",
        TileFlag::WINDOW | TileFlag::WALL | TileFlag::IMPASSABLE,
        20,
    );
    kit_item(
        &mut w,
        WATER,
        "test water",
        TileFlag::WET | TileFlag::IMPASSABLE | TileFlag::SURFACE,
        0,
    );
    kit_item(&mut w, ROCK, "test rock", TileFlag::IMPASSABLE, 20);
    kit_item(&mut w, PLAIN, "test plain", TileFlag::empty(), 1);

    let land = |w: &mut World, id: usize, name: &str, flags: TileFlag| {
        w.maps.tile_data.land[id] = LandData {
            name: name.to_string(),
            flags,
        };
    };
    land(&mut w, 0x3000, "test grass", TileFlag::empty());
    land(&mut w, 0x3001, "test rock", TileFlag::IMPASSABLE);
    land(&mut w, 0x3002, "test dirt", TileFlag::empty());
    for id in [0xA8, 0xA9, 0xAA, 0xAB, 0x136, 0x137] {
        land(
            &mut w,
            id,
            "test water",
            TileFlag::WET | TileFlag::IMPASSABLE,
        );
    }

    for (index, name, rules, width, height) in [
        (0, "Felucca", MapRules::empty(), 7168, 4096),
        (1, "Trammel", MapRules::TRAMMEL_RULES, 7168, 4096),
        (0x7F, "Internal", MapRules::INTERNAL, -1, -1),
    ] {
        let def = MapDefinition {
            index,
            id: index,
            file_index: index,
            name: name.to_string(),
            width,
            height,
            season: 0,
            rules,
        };
        super::loader::register_map(&mut w, &def).expect("test map registers");
    }

    w.maps.base_multi = Some(&TEST_MULTI);
    w.maps.multi_components = test_components;
    w
}

/// `new Item(itemId)` then `MoveToWorld(p, map)`.
pub fn place_item(w: &mut World, map: MapId, p: Point3D, item_id: i32) -> ItemId {
    let id = w.new_item(&ITEM_BASE);
    place_existing_item(w, id, map, p, item_id);
    id
}

/// `MoveToWorld` of an item that exists (any class).
pub fn place_existing_item(w: &mut World, id: ItemId, map: MapId, p: Point3D, item_id: i32) {
    if let Some(it) = w.item_mut(id) {
        it.item_id = item_id;
        it.location = p;
        it.map = Some(map);
    }
    super::sector::on_enter_item(w, map, p, id);
}

/// A `TestMulti` with these components, on the ground.
pub fn place_multi(w: &mut World, map: MapId, p: Point3D, entries: &[MultiTileEntry]) -> ItemId {
    let id = w.new_item(&TEST_MULTI);
    if let Some(it) = w.item_mut(id) {
        it.ext = Ext::new(MultiComponentList::from_entries(entries));
    }
    place_existing_item(w, id, map, p, 1);
    id
}

pub fn entry(item_id: u16, x: i16, y: i16, z: i16, flags: TileFlag) -> MultiTileEntry {
    MultiTileEntry::new(item_id, x, y, z, flags)
}

/// `new Mobile()` then `MoveToWorld(p, map)`.
pub fn place_mobile(w: &mut World, map: MapId, p: Point3D) -> MobileId {
    let id = w.new_mobile(&MOBILE_BASE);
    if let Some(m) = w.mobile_mut(id) {
        m.location = p;
        m.map = Some(map);
    }
    super::sector::on_enter_mobile(w, map, p, id);
    id
}

/// `Delete()`: leaves the sector and forgets the item.
pub fn delete_item(w: &mut World, id: ItemId) {
    let Some((map, loc)) = w.item(id).map(|i| (i.map, i.location)) else {
        return;
    };
    if let Some(map) = map {
        super::sector::on_leave_item(w, map, loc, id);
    }
    w.items.shift_remove(&id);
}

/// An item flagged deleted but still listed in its sector, as an enumerator may meet one mid-delete.
pub fn flag_deleted(w: &mut World, id: ItemId) {
    if let Some(it) = w.item_mut(id) {
        it.flags.insert(crate::item::ImplFlag::DELETED);
    }
}

/// Puts `item` inside `container`: it keeps its sector entry but has a parent.
pub fn set_parent(w: &mut World, item: ItemId, container: ItemId) {
    if let Some(it) = w.item_mut(item) {
        it.parent = Some(EntityId::Item(container));
    }
}

/// `TestTerrain.SetLand`.
pub fn set_land(w: &mut World, map: MapId, x: i32, y: i32, id: i16, z: i8) {
    let t = w.maps.tiles_mut(map).expect("map exists");
    let mut block = t.get_land_block(x >> 3, y >> 3).to_vec();
    block[(((y & 7) << 3) + (x & 7)) as usize] = LandTile::new(id, z);
    t.set_land_block(x >> 3, y >> 3, block);
}

/// `TestTerrain.AddStatic`.
pub fn add_static(w: &mut World, map: MapId, x: i32, y: i32, id: u16, z: i8) {
    let t = w.maps.tiles_mut(map).expect("map exists");
    let mut block = t.get_static_block(x >> 3, y >> 3).clone();
    block[(x & 7) as usize][(y & 7) as usize].push(StaticTile::new(id, z));
    t.set_static_block(x >> 3, y >> 3, block);
}

pub fn p(x: i32, y: i32, z: i32) -> Point3D {
    Point3D::new(x, y, z)
}

/// `Prefabs.InnUpstairs`: bounds origin, `Anchor` (south of the counter) and
/// `Target` (north of it), all at Z 20.
pub const INN_UPSTAIRS: (i32, i32) = (640, 256);
pub const INN_ANCHOR: (i32, i32, i32) = (648, 271, 20);
pub const INN_TARGET: (i32, i32, i32) = (649, 262, 20);

/// Stamps `Prefabs.InnUpstairs`: a walled upper floor at Z 20 with an L-shaped counter between the south and north
/// halves; the only way round is past the east end.
pub fn stamp_inn_upstairs(w: &mut World, map: MapId) {
    let (x0, y0) = INN_UPSTAIRS;
    for x in x0..x0 + 24 {
        for y in y0..y0 + 24 {
            let edge = x == x0 || x == x0 + 23 || y == y0 || y == y0 + 23;
            add_static(w, map, x, y, if edge { WALL } else { FLOOR }, 20);
        }
    }
    for x in x0 + 1..=x0 + 16 {
        add_static(w, map, x, y0 + 11, COUNTER, 20);
    }
    for y in y0 + 12..=y0 + 14 {
        add_static(w, map, x0 + 16, y, COUNTER, 20);
    }
}
