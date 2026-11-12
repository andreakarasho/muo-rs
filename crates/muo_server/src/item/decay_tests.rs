//! Decay registration tests. The clock is `w.now`; `advance_decay` is
//! `SimulatedClockScope.AdvanceDecay` (256 ms steps through the wheel, stopping once the item is deleted).

use muo_io::{SaveReader, SaveWriter};
use muo_primitives::{DateTime, Point3D, TimeSpan};

use super::container::{self, CONTAINER};
use super::decay;
use super::ops;
use super::*;
use crate::ids::ItemId;
use crate::map::MapId;
use crate::map::test_support::world;
use crate::world::World;

const FEL: MapId = MapId::FELUCCA;

fn setup() -> World {
    let mut w = world();
    decay::reset_for_tests(&mut w);
    w
}

fn put(w: &mut World, id: ItemId, x: i32) {
    ops::move_to_world_map(w, id, Point3D::new(x, 100, 0), Some(FEL));
}

fn decay_time(w: &World, id: ItemId) -> TimeSpan {
    (w.item(id).unwrap().ty.decay_time)(w, id)
}

fn advance_decay(w: &mut World, duration: TimeSpan, item: ItemId) {
    let start = w.now;
    let deadline = start + duration;
    let mut now = start;
    while now <= deadline && w.item(item).is_some() {
        w.now = now;
        decay::process_tick(w, now);
        now += TimeSpan::from_millis(256);
    }
}

fn is_deleted(w: &World, id: ItemId) -> bool {
    w.item(id).is_none_or(|i| i.deleted())
}

/// Stands in for a region that refuses decay (e.g. HouseRegion over a locked down item).
static REFUSES_DECAY: ItemType = ItemType {
    full_name: "Server.Tests.RefusesDecayItem",
    parent: Some(&ITEM_BASE),
    on_decay: |_, _| false,
    ..ITEM_BASE
};

/// A class with a `[Constructible]` zero parameter constructor (`Item(int itemID = 0)`); `LiftItemDupe` needs one.
static CONSTRUCTIBLE: ItemType = ItemType {
    full_name: "Server.Tests.ConstructibleItem",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| w.new_item(&CONSTRUCTIBLE)),
    ..ITEM_BASE
};

#[test]
fn item_on_ground_actually_decays_after_decay_time() {
    let mut w = setup();
    let item = w.new_item(&ITEM_BASE);
    put(&mut w, item, 100);
    assert!(decay::is_registered(&w, item));

    let d = decay_time(&w, item) + TimeSpan::from_mins(2);
    advance_decay(&mut w, d, item);

    assert!(
        is_deleted(&w, item),
        "Item on the ground must decay once DecayTime elapses."
    );
}

#[test]
fn item_on_ground_does_not_decay_before_decay_time() {
    let mut w = setup();
    let item = w.new_item(&ITEM_BASE);
    put(&mut w, item, 102);

    let d = decay_time(&w, item) - TimeSpan::from_mins(2);
    advance_decay(&mut w, d, item);

    assert!(
        !is_deleted(&w, item),
        "Item must not decay before DecayTime elapses."
    );
}

#[test]
fn decay_slot_agrees_with_structures_across_lifecycle() {
    let mut w = setup();
    let item = w.new_item(&ITEM_BASE);

    put(&mut w, item, 103);
    assert_ne!(w.item(item).unwrap().decay_slot, decay::SLOT_NONE);
    assert!(decay::is_registered(&w, item));

    // lifted to the cursor: untracked, and no stale slot left behind
    ops::internalize(&mut w, item);
    assert_eq!(w.item(item).unwrap().decay_slot, decay::SLOT_NONE);
    assert!(!decay::is_registered(&w, item));

    // back to the ground: tracked again
    put(&mut w, item, 104);
    assert_ne!(w.item(item).unwrap().decay_slot, decay::SLOT_NONE);
    assert!(decay::is_registered(&w, item));

    (w.item(item).unwrap().ty.delete)(&mut w, item);
    assert!(
        !decay::is_registered(&w, item),
        "A deleted item must leave no trace in the scheduler."
    );
}

#[test]
fn container_destroy_contents_dropped_to_ground_are_tracked() {
    let mut w = setup();
    let pack = container::new_container(&mut w, &CONTAINER, 0xE75);
    put(&mut w, pack, 105);

    let item = w.new_item(&ITEM_BASE);
    (CONTAINER.add_item)(&mut w, pack, item);
    assert!(!decay::is_registered(&w, item));

    container::destroy_default(&mut w, pack);

    assert!((w.item(item).unwrap().ty.can_decay)(&w, item));
    assert!(
        decay::is_registered(&w, item),
        "Contents spilled by Container.Destroy must be tracked."
    );
}

/// Unfreeze (movable) or unhide (visible) an item with a stale LastMoved: it gets a fresh decay window.
fn stale_item_gets_a_fresh_window(x: i32, toggle: fn(&mut World, ItemId, bool)) {
    let mut w = setup();
    let start = w.now;
    let item = w.new_item(&ITEM_BASE);
    put(&mut w, item, x);

    toggle(&mut w, item, false);
    assert!(
        !decay::is_registered(&w, item),
        "A frozen/hidden item must not be tracked for decay."
    );

    w.now = start + TimeSpan::from_days(30);
    toggle(&mut w, item, true);
    assert!(
        decay::is_registered(&w, item),
        "An unfrozen/unhidden item must be tracked for decay."
    );

    let d = decay_time(&w, item) - TimeSpan::from_mins(2);
    advance_decay(&mut w, d, item);
    assert!(
        !is_deleted(&w, item),
        "must get a full decay window, not vanish immediately."
    );

    advance_decay(&mut w, TimeSpan::from_mins(4), item);
    assert!(
        is_deleted(&w, item),
        "must still decay once the fresh window elapses."
    );
}

#[test]
fn stale_immovable_item_made_movable_gets_a_fresh_decay_window() {
    stale_item_gets_a_fresh_window(107, ops::set_movable);
}

#[test]
fn stale_hidden_item_made_visible_gets_a_fresh_decay_window() {
    stale_item_gets_a_fresh_window(109, ops::set_visible);
}

#[test]
fn refused_decay_does_not_rewrite_last_moved() {
    let mut w = setup();
    let item = w.new_item(&REFUSES_DECAY);
    put(&mut w, item, 110);
    let last_moved = w.item(item).unwrap().last_moved;

    let d = decay_time(&w, item) + TimeSpan::from_mins(2);
    advance_decay(&mut w, d, item);

    assert!(
        !is_deleted(&w, item),
        "A refused decay must not delete the item."
    );
    assert!(
        decay::is_registered(&w, item),
        "A refused decay must leave the item tracked."
    );
    assert_eq!(w.item(item).unwrap().last_moved, last_moved);
}

/// A movable item frozen then unfrozen 30 days later: it holds a decay reset stamp.
fn stamped_item(w: &mut World, x: i32) -> ItemId {
    let start = w.now;
    let item = w.new_item(&ITEM_BASE);
    put(w, item, x);
    ops::set_movable(w, item, false);
    w.now = start + TimeSpan::from_days(30);
    ops::set_movable(w, item, true);
    item
}

#[test]
fn fresh_decay_window_survives_serialization() {
    let mut w = setup();
    let item = stamped_item(&mut w, 111);
    let expected = ops::scheduled_decay_time(&w, item);

    let mut out = SaveWriter::new(true);
    (ITEM_BASE.serialize)(&w, item, &mut out);

    let copy = w.new_item(&ITEM_BASE);
    let bytes = out.into_vec();
    (ITEM_BASE.deserialize)(&mut w, copy, &mut SaveReader::new(&bytes)).unwrap();

    // the stamp is stored as a delta, so it ages only by the time between write and read
    let diff = ops::scheduled_decay_time(&w, copy) - expected;
    assert!(
        diff.ticks().abs() <= TimeSpan::from_secs(5).ticks(),
        "The restarted decay window must survive a save/load cycle."
    );
}

#[test]
fn moving_an_item_clears_a_superseded_decay_reset_stamp() {
    let mut w = setup();
    let item = stamped_item(&mut w, 112);
    assert_ne!(w.item(item).unwrap().decay_reset, DateTime::MIN);

    w.now += TimeSpan::from_mins(1);
    put(&mut w, item, 113);

    let i = w.item(item).unwrap();
    assert_eq!(i.decay_reset, DateTime::MIN);
    assert_eq!(
        ops::scheduled_decay_time(&w, item),
        i.last_moved + decay_time(&w, item)
    );
    assert!(decay::is_registered(&w, item));
}

#[test]
fn item_becoming_ineligible_drops_the_decay_reset_stamp() {
    let mut w = setup();
    let item = stamped_item(&mut w, 115);
    assert_ne!(w.item(item).unwrap().decay_reset, DateTime::MIN);

    ops::set_movable(&mut w, item, false);

    assert_eq!(w.item(item).unwrap().decay_reset, DateTime::MIN);
}

#[test]
fn stamped_item_added_to_container_drops_the_decay_reset_stamp() {
    let mut w = setup();
    let pack = container::new_container(&mut w, &CONTAINER, 0xE75);
    put(&mut w, pack, 116);
    let item = stamped_item(&mut w, 117);
    assert_ne!(w.item(item).unwrap().decay_reset, DateTime::MIN);

    (CONTAINER.add_item)(&mut w, pack, item);

    assert_eq!(w.item(item).unwrap().decay_reset, DateTime::MIN);
}

#[test]
fn item_moved_to_real_map_via_map_setter_is_registered_for_decay() {
    let mut w = setup();
    let item = w.new_item(&ITEM_BASE);
    assert!(!decay::is_registered(&w, item));

    ops::set_map(&mut w, item, Some(FEL));

    assert!((w.item(item).unwrap().ty.can_decay)(&w, item));
    assert!(
        decay::is_registered(&w, item),
        "Item placed on a map via the Map setter must be tracked."
    );
}

#[test]
fn partial_lift_of_ground_stack_leaves_remainder_registered_for_decay() {
    let mut w = setup();
    let stack = w.new_item(&CONSTRUCTIBLE);
    ops::set_stackable(&mut w, stack, true);
    ops::set_amount(&mut w, stack, 10);
    put(&mut w, stack, 114);

    let remainder = crate::mobile::part2::items::lift_item_dupe(&mut w, stack, 3)
        .expect("a dupe of the remainder");

    let r = w.item(remainder).unwrap();
    assert_eq!(r.amount, 7);
    assert!(r.parent.is_none());
    assert_eq!(r.map, Some(FEL));
    assert!(
        decay::is_registered(&w, remainder),
        "The remainder of a partially lifted ground stack must be tracked for decay."
    );
}

// AnchoredItemSerializationTests.ItemBytes_AreStable_AcrossSavesAtDifferentTimes: Item v11 stores
// LastMoved and DecayResetTime as anchored time, so the bytes are a function of item state only.
#[test]
fn item_bytes_are_stable_across_saves_at_different_times() {
    let mut w = setup();
    let start = w.now;
    let item = w.new_item(&ITEM_BASE);
    w.item_mut(item).unwrap().item_id = 0x1F13;
    put(&mut w, item, 120);
    ops::restart_decay(&mut w, item);

    let mut first = SaveWriter::new(true);
    (ITEM_BASE.serialize)(&w, item, &mut first);

    // a save hours later, with no state change, must produce identical bytes
    w.now = start + TimeSpan::from_hours(5);
    let mut second = SaveWriter::new(true);
    (ITEM_BASE.serialize)(&w, item, &mut second);

    assert_eq!(first.as_slice(), second.as_slice());
}
