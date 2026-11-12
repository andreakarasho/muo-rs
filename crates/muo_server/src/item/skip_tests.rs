//! Container child skip tests.

use muo_io::{SaveReader, SaveWriter};

use super::container::{self, CONTAINER};
use super::*;
use crate::ids::ItemId;
use crate::world::World;

static SKIPPING_CONTAINER: ItemType = ItemType {
    full_name: "Server.Tests.SkippingContainer",
    parent: Some(&CONTAINER),
    skips_child_serialization: |_, _| true,
    ..CONTAINER
};

fn skipping(w: &mut World) -> ItemId {
    container::new_container(w, &SKIPPING_CONTAINER, 0xE75)
}

fn normal(w: &mut World) -> ItemId {
    container::new_container(w, &CONTAINER, 0xE75)
}

fn item(w: &mut World) -> ItemId {
    let id = w.new_item(&ITEM_BASE);
    w.item_mut(id).unwrap().item_id = 0x1234;
    id
}

fn skip(w: &World, id: ItemId) -> bool {
    (w.item(id).unwrap().ty.skip_serialization)(w, id)
}

#[test]
fn child_of_skipping_container_skips_until_moved_out() {
    let mut w = World::new();
    let pack = skipping(&mut w);
    let normal = normal(&mut w);
    let it = item(&mut w);

    container::drop_item_default(&mut w, pack, it);
    assert!(skip(&w, it));
    assert!(!skip(&w, pack));

    container::drop_item_default(&mut w, normal, it);
    assert!(!skip(&w, it));
}

#[test]
fn grandchild_of_skipping_container_is_not_skipped() {
    let mut w = World::new();
    let pack = skipping(&mut w);
    let inner = normal(&mut w);
    let it = item(&mut w);

    container::drop_item_default(&mut w, pack, inner);
    container::drop_item_default(&mut w, inner, it);

    assert!(skip(&w, inner));
    assert!(!skip(&w, it));
}

// A skipped child is not saved, so the container must load without it.
// A written child serial could resolve at load to an unrelated item that was handed the same serial.
#[test]
fn skipping_container_does_not_write_alive_child_serial() {
    let mut w = World::new();
    let pack = skipping(&mut w);
    let it = item(&mut w);
    container::drop_item_default(&mut w, pack, it);

    let mut out = SaveWriter::new(true);
    (SKIPPING_CONTAINER.serialize)(&w, pack, &mut out);
    let bytes = out.into_vec();

    let copy = skipping(&mut w);
    (SKIPPING_CONTAINER.deserialize)(&mut w, copy, &mut SaveReader::new(&bytes)).unwrap();

    assert!(w.item(copy).unwrap().items.is_empty());
}

// An item can be saved naming a skipping container as parent while absent from its list; it must load
// as parentless so the missing-parent path deletes it rather than leaving an unreachable orphan.
#[test]
fn item_loading_with_skipping_container_as_parent_clears_parent() {
    let mut w = World::new();
    let pack = skipping(&mut w);
    let it = item(&mut w);
    container::drop_item_default(&mut w, pack, it);

    let mut out = SaveWriter::new(true);
    (ITEM_BASE.serialize)(&w, it, &mut out);
    let bytes = out.into_vec();

    (w.item(it).unwrap().ty.delete)(&mut w, it);

    let copy = w.new_item(&ITEM_BASE);
    (ITEM_BASE.deserialize)(&mut w, copy, &mut SaveReader::new(&bytes)).unwrap();

    assert!(w.item(copy).is_none_or(|c| c.parent.is_none()));
}
