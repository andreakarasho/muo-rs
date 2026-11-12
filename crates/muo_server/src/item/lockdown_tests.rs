//! Locked-down container decay, player-constructed stacking and container bulk removal tests.

use super::container::{self, CONTAINER};
use super::decay;
use super::ops;
use super::*;
use crate::ids::ItemId;
use crate::map::MapId;
use crate::map::test_support::world;
use crate::world::World;
use muo_primitives::{Point3D, TimeSpan};

const FEL: MapId = MapId::FELUCCA;

fn setup() -> World {
    let mut w = world();
    decay::reset_for_tests(&mut w);
    // Content assigns these at startup (BaseHouse); the engine tests have no content.
    let s = ops::statics(&mut w);
    if s.locked_down_flag == 0 {
        s.locked_down_flag = 1;
        s.secure_flag = 2;
    }
    w
}

fn keeps_contents(_: &World, _: ItemId) -> bool {
    false
}

static KEEPS_CONTENTS_CHEST: ItemType = ItemType {
    full_name: "Server.Tests.KeepsContentsChest",
    parent: Some(&CONTAINER),
    contents_decay: keeps_contents,
    ..CONTAINER
};

fn new_loot(w: &mut World) -> ItemId {
    let id = w.new_item(&ITEM_BASE);
    w.item_mut(id).unwrap().item_id = 0x1F03;
    ops::set_movable(w, id, true);
    ops::set_visible(w, id, true);
    id
}

fn place_chest(w: &mut World, ty: &'static ItemType, x: i32, locked_down: bool) -> ItemId {
    let chest = container::new_container(w, ty, 0xE43);
    ops::set_movable(w, chest, false);
    ops::move_to_world_map(w, chest, Point3D::new(x, 100, 0), Some(FEL));
    if locked_down {
        ops::set_locked_down(w, chest, true);
    }
    chest
}

fn registered(w: &World, id: ItemId) -> bool {
    decay::is_registered(w, id)
}

#[test]
fn item_dropped_into_locked_down_container_is_tracked() {
    let mut w = setup();
    let chest = place_chest(&mut w, &CONTAINER, 120, true);
    let loot = new_loot(&mut w);

    (CONTAINER.add_item)(&mut w, chest, loot);

    assert!((w.item(loot).unwrap().ty.can_decay)(&w, loot));
    assert!(registered(&w, loot));
}

#[test]
fn item_in_ordinary_container_is_not_tracked() {
    let mut w = setup();
    let chest = container::new_container(&mut w, &CONTAINER, 0xE43);
    ops::move_to_world_map(&mut w, chest, Point3D::new(121, 100, 0), Some(FEL));
    let loot = new_loot(&mut w);

    (CONTAINER.add_item)(&mut w, chest, loot);

    assert!(!(w.item(loot).unwrap().ty.can_decay)(&w, loot));
    assert!(!registered(&w, loot));
}

#[test]
fn locking_and_releasing_container_registers_and_unregisters_contents() {
    let mut w = setup();
    let chest = place_chest(&mut w, &CONTAINER, 122, false);
    let loot = new_loot(&mut w);
    (CONTAINER.add_item)(&mut w, chest, loot);

    assert!(!registered(&w, loot));

    ops::set_locked_down(&mut w, chest, true);
    assert!(registered(&w, loot));

    ops::set_secure(&mut w, chest, true); // secure containers keep their contents
    assert!(!registered(&w, loot));

    ops::set_secure(&mut w, chest, false);
    assert!(registered(&w, loot));

    ops::set_locked_down(&mut w, chest, false);
    assert!(!registered(&w, loot));
}

#[test]
fn container_that_keeps_contents_does_not_track_them() {
    let mut w = setup();
    let board = place_chest(&mut w, &KEEPS_CONTENTS_CHEST, 123, true);
    let piece = new_loot(&mut w);

    (CONTAINER.add_item)(&mut w, board, piece);

    assert!(!registered(&w, piece));
}

#[test]
fn nested_container_contents_are_not_tracked() {
    let mut w = setup();
    let chest = place_chest(&mut w, &CONTAINER, 124, true);
    let bag = container::new_container(&mut w, &CONTAINER, 0xE43);
    (CONTAINER.add_item)(&mut w, chest, bag);
    let loot = new_loot(&mut w);
    (CONTAINER.add_item)(&mut w, bag, loot);

    assert!(registered(&w, bag));
    assert!(!registered(&w, loot));
}

#[test]
fn locked_down_item_inside_locked_down_container_is_not_tracked() {
    let mut w = setup();
    let chest = place_chest(&mut w, &CONTAINER, 125, true);
    let loot = new_loot(&mut w);
    (CONTAINER.add_item)(&mut w, chest, loot);
    ops::set_locked_down(&mut w, loot, true);

    assert!(!registered(&w, loot));
}

#[test]
fn tracked_contents_decay_on_schedule() {
    let mut w = setup();
    let chest = place_chest(&mut w, &CONTAINER, 126, true);
    let loot = new_loot(&mut w);
    (CONTAINER.add_item)(&mut w, chest, loot);

    let duration = (w.item(loot).unwrap().ty.decay_time)(&w, loot) + TimeSpan::from_mins(2);
    let deadline = w.now + duration;
    let mut now = w.now;
    while now <= deadline && w.item(loot).is_some() {
        w.now = now;
        decay::process_tick(&mut w, now);
        now += TimeSpan::from_millis(256);
    }

    assert!(
        w.item(loot).is_none(),
        "Contents of a locked-down container must decay."
    );
    assert!(
        w.item(chest).is_some(),
        "The locked-down container itself must not decay."
    );
}

// ---- PlayerConstructedStackingTests ----

fn construct_stackable(w: &mut World) -> ItemId {
    let id = w.new_item(&STACKABLE_ITEM);
    ops::set_stackable(w, id, true);
    id
}

/// A real stackable type: `LiftItemDupe` builds the remainder through the parameterless constructor and the
/// type restores `Stackable` there, as every genuine stackable does.
static STACKABLE_ITEM: ItemType = ItemType {
    full_name: "Server.Tests.StackableItem",
    parent: Some(&ITEM_BASE),
    construct: Some(construct_stackable),
    ..ITEM_BASE
};

fn make_stack(w: &mut World, amount: i32, player_constructed: bool) -> ItemId {
    let id = construct_stackable(w);
    ops::set_amount(w, id, amount);
    w.item_mut(id).unwrap().player_constructed = player_constructed;
    id
}

#[test]
fn lift_item_dupe_copies_player_constructed_to_remainder() {
    for pc in [false, true] {
        let mut w = setup();
        let stack = make_stack(&mut w, 10, pc);

        let remainder =
            crate::mobile::part2::items::lift_item_dupe(&mut w, stack, 4).expect("a remainder");

        assert_ne!(remainder, stack);
        assert_eq!(w.item(stack).unwrap().amount, 4);
        assert_eq!(w.item(remainder).unwrap().amount, 6);
        assert_eq!(w.item(remainder).unwrap().player_constructed, pc);
    }
}

#[test]
fn split_halves_remain_stackable_with_each_other() {
    let mut w = setup();
    let stack = make_stack(&mut w, 10, true);
    let remainder =
        crate::mobile::part2::items::lift_item_dupe(&mut w, stack, 4).expect("a remainder");

    let ty = w.item(stack).unwrap().ty;
    assert!((ty.can_stack_with)(&w, stack, remainder));
    assert!((ty.stack_with)(&mut w, stack, None, remainder, false));
    assert_eq!(w.item(stack).unwrap().amount, 10);
    assert!(w.item(stack).unwrap().player_constructed);
}

// ---- ContainerBulkRemovalTests ----

#[test]
fn deleting_large_container_deletes_all_children_and_totals_stay_consistent() {
    const CHILD_COUNT: usize = 20_000;
    let mut w = setup();
    let outer = container::new_container(&mut w, &CONTAINER, 0xE75);
    let pack = container::new_container(&mut w, &CONTAINER, 0xE75);
    container::drop_item_default(&mut w, outer, pack);

    let mut children = Vec::with_capacity(CHILD_COUNT);
    for _ in 0..CHILD_COUNT {
        let c = w.new_item(&ITEM_BASE);
        w.item_mut(c).unwrap().item_id = 0x1234;
        container::drop_item_default(&mut w, pack, c);
        children.push(c);
    }
    assert_eq!(
        ops::get_total(&w, outer, TotalType::Items),
        children.len() as i32 + 1
    );

    (w.item(pack).unwrap().ty.delete)(&mut w, pack);

    assert!(
        children
            .iter()
            .all(|&c| w.item(c).is_none_or(|i| i.deleted()))
    );
    assert!(w.item(outer).unwrap().items.is_empty());
    assert_eq!(ops::get_total(&w, outer, TotalType::Items), 0);
}

#[test]
fn removing_from_front_middle_and_end_removes_exactly_that_item() {
    let mut w = setup();
    let pack = container::new_container(&mut w, &CONTAINER, 0xE75);
    let mut items = Vec::new();
    for _ in 0..5 {
        let c = w.new_item(&ITEM_BASE);
        w.item_mut(c).unwrap().item_id = 0x1234;
        container::drop_item_default(&mut w, pack, c);
        items.push(c);
    }

    for i in [0, 2, 4] {
        (w.item(items[i]).unwrap().ty.delete)(&mut w, items[i]);
    }

    assert_eq!(w.item(pack).unwrap().items, vec![items[1], items[3]]);
}
