//! `Beads` (a plain `Item`) and the constructors of the two sample jewels of
//! `muo_content::items::leaves` for the classes derived from them.

use muo_content::items::jewel::new_jewel;
use muo_server::World;
use muo_server::ids::ItemId;
use muo_server::item::{ITEM_BASE, ItemType, ops};
use muo_server::type_registry::TypeRegistry;

/// `GoldRing()`: `base(0x108a)`.
pub fn gold_ring_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    new_jewel(w, ty, 0x108A)
}

/// `Server.Items.Beads` (`[SerializationGenerator(0, false)]`, `base(0x108B)`, `DefaultWeight => 1.0`).
pub static BEADS: ItemType = ItemType {
    full_name: "Server.Items.Beads",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| ops::new_item_with_id(w, &BEADS, 0x108B)),
    default_weight: |_, _| 1.0,
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        out.write_i32(0);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        Ok(())
    },
    ..ITEM_BASE
};

/// Registers the classes of this file.
pub fn register(types: &mut TypeRegistry) {
    types.register_item(&BEADS, &[]);
}
