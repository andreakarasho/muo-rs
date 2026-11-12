//! A call `x.F()` of a virtual member runs the most derived override; only an override's `base.F()` reaches the
//! parent's body. The Rust shape: `mobile::base::F` / `item::base::F` dispatch through the type's vtable and
//! `F_base` is the parent body (the vtable default).
//! These tests prove the shape on key members (`MoveToWorld` overrides were once skipped by their callers).

use std::sync::atomic::{AtomicUsize, Ordering::SeqCst};

use muo_primitives::Point3D;
use muo_server::item::base as ib;
use muo_server::item::item_type::ITEM_BASE;
use muo_server::map::MapId;
use muo_server::mobile::base as mb;
use muo_server::mobile::mobile_type::MOBILE_BASE;
use muo_server::{ItemId, ItemType, MobileId, MobileType, World};

static M_MOVE: AtomicUsize = AtomicUsize::new(0);
static M_DELETE: AtomicUsize = AtomicUsize::new(0);
static M_HUE: AtomicUsize = AtomicUsize::new(0);
static I_DELETE: AtomicUsize = AtomicUsize::new(0);
static I_HUE: AtomicUsize = AtomicUsize::new(0);

static OVERRIDER: MobileType = MobileType {
    full_name: "Test.Overrider",
    parent: Some(&MOBILE_BASE),
    // `base.MoveToWorld(..)`: the parent's body, not the dispatcher
    move_to_world: |w, id, l, map| {
        M_MOVE.fetch_add(1, SeqCst);
        mb::move_to_world_base(w, id, l, map);
    },
    delete: |w, id| {
        M_DELETE.fetch_add(1, SeqCst);
        mb::delete_base(w, id);
    },
    set_hue: |w, id, v| {
        M_HUE.fetch_add(1, SeqCst);
        mb::set_hue_base(w, id, v);
    },
    ..MOBILE_BASE
};

static ITEM_OVERRIDER: ItemType = ItemType {
    full_name: "Test.ItemOverrider",
    parent: Some(&ITEM_BASE),
    delete: |w, id| {
        I_DELETE.fetch_add(1, SeqCst);
        ib::delete_base(w, id);
    },
    set_hue: |w, id, v| {
        I_HUE.fetch_add(1, SeqCst);
        ib::set_hue_base(w, id, v);
    },
    ..ITEM_BASE
};

#[test]
fn the_common_mobile_functions_run_the_override() {
    let mut w = World::new();
    let m: MobileId = w.new_mobile(&OVERRIDER);
    mb::move_to_world(&mut w, m, Point3D::new(10, 10, 0), MapId::FELUCCA);
    assert_eq!(M_MOVE.load(SeqCst), 1, "MoveToWorld override skipped");
    assert_eq!(
        w.mobile(m).unwrap().map,
        Some(MapId::FELUCCA),
        "the override reached the base body"
    );
    mb::set_hue(&mut w, m, 5);
    assert_eq!(M_HUE.load(SeqCst), 1, "Hue setter override skipped");
    mb::delete(&mut w, m);
    assert_eq!(M_DELETE.load(SeqCst), 1, "Delete override skipped");
    assert!(w.mobile(m).is_none_or(|m| m.deleted));
}

#[test]
fn the_common_item_functions_run_the_override() {
    let mut w = World::new();
    let i: ItemId = w.new_item(&ITEM_OVERRIDER);
    ib::set_hue(&mut w, i, 7);
    assert_eq!(I_HUE.load(SeqCst), 1, "Hue setter override skipped");
    ib::delete(&mut w, i);
    assert_eq!(I_DELETE.load(SeqCst), 1, "Delete override skipped");
    assert!(muo_server::item::ops::is_deleted(&w, i));
}
