//! `LargeForgeWest` and `LargeForgeEast`, each a
//! forge of three pieces that move, change map and are deleted together. The two further pieces of a forge are
//! its nested `InternalItem` and `InternalItem2` (`Server.Items.LargeForgeWest+InternalItem`..).
//!
//! `LargeForgeWest` saves its two pieces (`[SerializableField]`); `LargeForgeEast` saves only the version (its
//! `_item` fields have no attribute), so after a load an east forge has no pieces (kept for save compatibility).

use muo_io::{GenericReader, GenericWriter, Result};
use muo_primitives::Point3D;
use muo_server::World;
use muo_server::ext::Ext;
use muo_server::ids::ItemId;
use muo_server::item::{ITEM_BASE, ItemType, base, ops};
use muo_server::serialization_ext::{ReadEntityExt, WriteEntityExt};
use muo_server::type_registry::TypeRegistry;

use super::{read_version, write_version};

/// A forge's pieces: `_item` (+1) and `_item2` (+2).
#[derive(Clone, Copy, Debug, Default)]
struct Pieces {
    item: Option<ItemId>,
    item2: Option<ItemId>,
}

/// A piece's `_parent`.
#[derive(Clone, Copy, Debug)]
struct Parent(Option<ItemId>);

fn pieces(w: &World, id: ItemId) -> Pieces {
    w.item(id)
        .and_then(|i| i.ext.get::<Pieces>())
        .copied()
        .unwrap_or_default()
}

fn parent_of(w: &World, id: ItemId) -> Option<ItemId> {
    w.item(id)
        .and_then(|i| i.ext.get::<Parent>())
        .and_then(|p| p.0)
}

/// One kind of forge: the item ids and the offset of each piece from the main one.
struct Layout {
    main: i32,
    piece: i32,
    piece2: i32,
    /// Offset of `_item`, `_item2` from the main piece (the pieces' parent moves back by it).
    step: (i32, i32),
    west: bool,
}

const WEST: Layout = Layout {
    main: 0x199A,
    piece: 0x1996,
    piece2: 0x1992,
    step: (0, 1),
    west: true,
};
const EAST: Layout = Layout {
    main: 0x197A,
    piece: 0x197E,
    piece2: 0x1982,
    step: (1, 0),
    west: false,
};

fn layout(w: &World, id: ItemId) -> &'static Layout {
    if w.item(id).is_some_and(|i| i.ty.is(&LARGE_FORGE_EAST)) {
        &EAST
    } else {
        &WEST
    }
}

/// The offset of the piece at `index` (1 or 2).
fn offset(l: &Layout, index: i32) -> (i32, i32) {
    (l.step.0 * index, l.step.1 * index)
}

fn new_piece(w: &mut World, ty: &'static ItemType, item_id: i32, parent: ItemId) -> ItemId {
    let id = ops::new_item_with_id(w, ty, item_id);
    ops::set_movable(w, id, false);
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(Parent(Some(parent)));
    }
    id
}

fn new_forge(w: &mut World, ty: &'static ItemType, l: &Layout) -> ItemId {
    let id = ops::new_item_with_id(w, ty, l.main);
    ops::set_movable(w, id, false);
    let (p1, p2) = if l.west {
        (&WEST_PIECE, &WEST_PIECE2)
    } else {
        (&EAST_PIECE, &EAST_PIECE2)
    };
    let item = new_piece(w, p1, l.piece, id);
    let item2 = new_piece(w, p2, l.piece2, id);
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(Pieces {
            item: Some(item),
            item2: Some(item2),
        });
    }
    id
}

/// `new LargeForgeWest()`.
pub fn new_large_forge_west(w: &mut World) -> ItemId {
    new_forge(w, &LARGE_FORGE_WEST, &WEST)
}

/// `new LargeForgeEast()`.
pub fn new_large_forge_east(w: &mut World) -> ItemId {
    new_forge(w, &LARGE_FORGE_EAST, &EAST)
}

fn main_on_location_change(w: &mut World, id: ItemId, old: Point3D) {
    base::on_location_change_base(w, id, old);
    let Some(loc) = w.item(id).map(|i| i.location) else {
        return;
    };
    let l = layout(w, id);
    let p = pieces(w, id);
    for (index, piece) in [(1, p.item), (2, p.item2)] {
        if let Some(piece) = piece {
            let (dx, dy) = offset(l, index);
            ops::set_location(w, piece, Point3D::new(loc.x + dx, loc.y + dy, loc.z));
        }
    }
}

fn main_on_map_change(w: &mut World, id: ItemId) {
    base::on_map_change_base(w, id);
    let map = w.item(id).and_then(|i| i.map);
    let p = pieces(w, id);
    for piece in [p.item, p.item2].into_iter().flatten() {
        ops::set_map(w, piece, map);
    }
}

fn main_on_after_delete(w: &mut World, id: ItemId) {
    base::on_after_delete_base(w, id);
    let p = pieces(w, id);
    for piece in [p.item, p.item2].into_iter().flatten() {
        base::delete(w, piece);
    }
}

/// `InternalItem.OnLocationChange` / `InternalItem2`'s: the forge moves back by the piece's offset.
fn piece_on_location_change(w: &mut World, id: ItemId, old: Point3D, index: i32) {
    base::on_location_change_base(w, id, old); // base body: the helper is the overrides' body
    let Some(loc) = w.item(id).map(|i| i.location) else {
        return;
    };
    if let Some(parent) = parent_of(w, id) {
        let l = layout(w, parent);
        let (dx, dy) = offset(l, index);
        ops::set_location(w, parent, Point3D::new(loc.x - dx, loc.y - dy, loc.z));
    }
}

fn piece_on_map_change(w: &mut World, id: ItemId) {
    base::on_map_change_base(w, id);
    let map = w.item(id).and_then(|i| i.map);
    if let Some(parent) = parent_of(w, id) {
        ops::set_map(w, parent, map);
    }
}

fn piece_on_after_delete(w: &mut World, id: ItemId) {
    base::on_after_delete_base(w, id);
    if let Some(parent) = parent_of(w, id) {
        base::delete(w, parent);
    }
}

fn version_only_serialize(w: &World, id: ItemId, out: &mut dyn GenericWriter) {
    (ITEM_BASE.serialize)(w, id, out);
    write_version(out, false);
}

fn version_only_deserialize(w: &mut World, id: ItemId, r: &mut dyn GenericReader) -> Result<()> {
    (ITEM_BASE.deserialize)(w, id, r)?;
    read_version(r, false)?;
    Ok(())
}

/// `Server.Items.LargeForgeWest`.
pub static LARGE_FORGE_WEST: ItemType = ItemType {
    full_name: "Server.Items.LargeForgeWest",
    parent: Some(&ITEM_BASE),
    construct: Some(new_large_forge_west),
    on_location_change: main_on_location_change,
    on_map_change: main_on_map_change,
    on_after_delete: main_on_after_delete,
    serialize: |w, id, out| {
        version_only_serialize(w, id, out);
        let p = pieces(w, id);
        out.write_item(w, p.item);
        out.write_item(w, p.item2);
    },
    deserialize: |w, id, r| {
        version_only_deserialize(w, id, r)?;
        let item = r.read_item(w)?;
        let item2 = r.read_item(w)?;
        if let Some(i) = w.item_mut(id) {
            i.ext = Ext::new(Pieces { item, item2 });
        }
        Ok(())
    },
    ..ITEM_BASE
};

/// `Server.Items.LargeForgeEast`.
pub static LARGE_FORGE_EAST: ItemType = ItemType {
    full_name: "Server.Items.LargeForgeEast",
    construct: Some(new_large_forge_east),
    serialize: version_only_serialize,
    deserialize: |w, id, r| {
        version_only_deserialize(w, id, r)?;
        if let Some(i) = w.item_mut(id) {
            i.ext = Ext::new(Pieces::default());
        }
        Ok(())
    },
    ..LARGE_FORGE_WEST
};

macro_rules! piece {
    ($name:ident, $full:literal, $index:expr) => {
        #[doc = concat!("`", $full, "`.")]
        pub static $name: ItemType = ItemType {
            full_name: $full,
            parent: Some(&ITEM_BASE),
            construct: None,
            on_location_change: |w, id, old| piece_on_location_change(w, id, old, $index),
            on_map_change: piece_on_map_change,
            on_after_delete: piece_on_after_delete,
            serialize: version_only_serialize,
            deserialize: |w, id, r| {
                version_only_deserialize(w, id, r)?;
                if let Some(i) = w.item_mut(id) {
                    i.ext = Ext::new(Parent(None)); // `_parent` is not saved
                }
                Ok(())
            },
            ..ITEM_BASE
        };
    };
}

piece!(WEST_PIECE, "Server.Items.LargeForgeWest+InternalItem", 1);
piece!(WEST_PIECE2, "Server.Items.LargeForgeWest+InternalItem2", 2);
piece!(EAST_PIECE, "Server.Items.LargeForgeEast+InternalItem", 1);
piece!(EAST_PIECE2, "Server.Items.LargeForgeEast+InternalItem2", 2);

/// Registers the large forges.
pub fn register(types: &mut TypeRegistry) {
    for t in [
        &LARGE_FORGE_WEST,
        &LARGE_FORGE_EAST,
        &WEST_PIECE,
        &WEST_PIECE2,
        &EAST_PIECE,
        &EAST_PIECE2,
    ] {
        types.register_item(t, &[]);
    }
}
