//! Container tests. Container iterators could throw when the container changes
//! during the walk; here a walk is a snapshot `Vec` (see `enumerable.rs`), so the ShouldThrowWhenModified rows
//! check what stays observable: the removal during the walk works and the snapshot still lists every match.

use super::container::{self, CONTAINER};
use super::enumerable;
use super::ops;
use super::*;
use crate::ids::ItemId;
use crate::world::World;

static TEST_REAGENT_A: ItemType = ItemType {
    full_name: "Server.Tests.TestReagentA",
    parent: Some(&ITEM_BASE),
    ..ITEM_BASE
};
static TEST_REAGENT_B: ItemType = ItemType {
    full_name: "Server.Tests.TestReagentB",
    parent: Some(&ITEM_BASE),
    ..ITEM_BASE
};
/// `Static` is an `Item` subclass.
static TEST_STATIC: ItemType = ItemType {
    full_name: "Server.Items.Static",
    parent: Some(&ITEM_BASE),
    ..ITEM_BASE
};

fn make_container(w: &mut World) -> ItemId {
    container::new_container(w, &CONTAINER, 0xE75)
}

/// `new Item((Serial)n)` of class `ty` with a stack of `amount`, `hue` when non-zero.
fn make_stack(w: &mut World, ty: &'static ItemType, amount: i32, hue: i32) -> ItemId {
    let id = w.new_item(ty);
    ops::set_stackable(w, id, true);
    ops::set_amount(w, id, amount);
    if hue != 0 {
        w.item_mut(id).unwrap().hue = hue;
    }
    id
}

fn add(w: &mut World, cont: ItemId, item: ItemId) {
    (CONTAINER.add_item)(w, cont, item);
}

/// `CheckHueGrouping`: `b.Hue.CompareTo(a.Hue)`.
fn hue_grouper(w: &World, a: ItemId, b: ItemId) -> i32 {
    w.item(b).unwrap().hue.cmp(&w.item(a).unwrap().hue) as i32
}

fn amount(w: &World, id: ItemId) -> i32 {
    w.item(id).unwrap().amount
}

/// Both container kinds the Theory runs over: a `Container` and a plain `Item` used as one.
fn kinds(w: &mut World) -> [(ItemId, ItemId); 2] {
    let mut out = [(ItemId(Serial::new(0)), ItemId(Serial::new(0))); 2];
    out[0] = (make_container(w), make_container(w));
    out[1] = (w.new_item(&ITEM_BASE), w.new_item(&ITEM_BASE));
    out
}

fn put(w: &mut World, cont: ItemId, item: ItemId) {
    // Item.AddItem is the base behavior for both kinds
    super::base::add_item(w, cont, item);
}

#[test]
fn find_items_by_type() {
    for kind in 0..2 {
        let mut w = World::new();
        let (container, _) = kinds(&mut w)[kind];
        let plain = w.new_item(&ITEM_BASE);
        let static_item = w.new_item(&TEST_STATIC);
        put(&mut w, container, plain);
        put(&mut w, container, static_item);

        let found = enumerable::find_items_by_type(&w, container, true, &TEST_STATIC, |_, _| true);
        assert_eq!(found, vec![static_item]);
    }
}

#[test]
fn find_items_by_type_nested() {
    for kind in 0..2 {
        let mut w = World::new();
        let ks = kinds(&mut w);
        let (container, container2) = ks[kind];
        let container3 = if kind == 0 {
            make_container(&mut w)
        } else {
            w.new_item(&ITEM_BASE)
        };
        let (static1, static2) = (w.new_item(&TEST_STATIC), w.new_item(&TEST_STATIC));
        let plain = w.new_item(&ITEM_BASE);
        put(&mut w, container, plain);
        put(&mut w, container, container2);
        put(&mut w, container2, container3);
        put(&mut w, container3, static2);
        put(&mut w, container2, static1);

        let statics =
            enumerable::find_items_by_type(&w, container, true, &TEST_STATIC, |_, _| true);
        assert_eq!(statics, vec![static1, static2]);
    }
}

#[test]
fn find_items_by_type_not_matching() {
    for kind in 0..2 {
        let mut w = World::new();
        let ks = kinds(&mut w);
        let (container, container2) = ks[kind];
        let (a, b) = (w.new_item(&ITEM_BASE), w.new_item(&ITEM_BASE));
        put(&mut w, container, a);
        put(&mut w, container, container2);
        put(&mut w, container2, b);

        assert!(
            enumerable::find_items_by_type(&w, container, true, &TEST_STATIC, |_, _| true)
                .is_empty()
        );
    }
}

#[test]
fn find_items_by_type_with_removal_during_the_walk() {
    // A snapshot walk survives the removal and still lists the match.
    for kind in 0..2 {
        let mut w = World::new();
        let (container, _) = kinds(&mut w)[kind];
        let (a, s, c) = (
            w.new_item(&ITEM_BASE),
            w.new_item(&TEST_STATIC),
            w.new_item(&ITEM_BASE),
        );
        put(&mut w, container, a);
        put(&mut w, container, s);
        put(&mut w, container, c);

        for item in enumerable::find_items_by_type(&w, container, true, &TEST_STATIC, |_, _| true) {
            if item == s {
                super::base::remove_item(&mut w, container, s);
            }
        }
        assert_eq!(w.item(container).unwrap().items, vec![a, c]);
    }
}

#[test]
fn enumerate_items_by_type_when_modified() {
    for kind in 0..2 {
        let mut w = World::new();
        let (container, _) = kinds(&mut w)[kind];
        let item1 = w.new_item(&ITEM_BASE);
        let item2 = w.new_item(&TEST_STATIC);
        let item3 = w.new_item(&ITEM_BASE);
        put(&mut w, container, item1);
        put(&mut w, container, item2);
        put(&mut w, container, item3);

        for item in enumerable::find_items_by_type(&w, container, true, &TEST_STATIC, |_, _| true) {
            if item == item2 {
                super::base::remove_item(&mut w, container, item2);
            }
        }

        let items = &w.item(container).unwrap().items;
        assert_eq!(items.len(), 2);
        assert_eq!(items, &vec![item1, item3]);
    }
}

#[test]
fn consume_total_single_type_exact_amount_succeeds() {
    let mut w = World::new();
    let c = make_container(&mut w);
    let a = make_stack(&mut w, &TEST_REAGENT_A, 5, 0);
    add(&mut w, c, a);

    assert!(container::consume_total(
        &mut w,
        c,
        &TEST_REAGENT_A,
        5,
        true,
        None
    ));
    assert!(w.item(c).unwrap().items.is_empty()); // stack fully consumed -> Delete()
}

#[test]
fn consume_total_single_type_across_stacks_partial_on_last() {
    let mut w = World::new();
    let c = make_container(&mut w);
    let a = make_stack(&mut w, &TEST_REAGENT_A, 3, 0);
    let b = make_stack(&mut w, &TEST_REAGENT_A, 4, 0);
    add(&mut w, c, a);
    add(&mut w, c, b);

    assert!(container::consume_total(
        &mut w,
        c,
        &TEST_REAGENT_A,
        5,
        true,
        None
    ));
    // BFS order: a fully consumed (3), b partially (2 of 4 remain)
    assert!(w.item(a).is_none());
    assert!(w.item(b).is_some());
    assert_eq!(amount(&w, b), 2);
}

#[test]
fn consume_total_single_type_not_enough_no_consumption() {
    let mut w = World::new();
    let c = make_container(&mut w);
    let a = make_stack(&mut w, &TEST_REAGENT_A, 2, 0);
    let b = make_stack(&mut w, &TEST_REAGENT_A, 2, 0);
    add(&mut w, c, a);
    add(&mut w, c, b);

    assert!(!container::consume_total(
        &mut w,
        c,
        &TEST_REAGENT_A,
        10,
        true,
        None
    ));
    assert_eq!(amount(&w, a), 2);
    assert_eq!(amount(&w, b), 2);
}

#[test]
fn consume_total_multi_type_fails_at_slot1_slot0_untouched() {
    let mut w = World::new();
    let c = make_container(&mut w);
    let ra = make_stack(&mut w, &TEST_REAGENT_A, 5, 0);
    let rb = make_stack(&mut w, &TEST_REAGENT_B, 1, 0); // not enough for slot 1
    add(&mut w, c, ra);
    add(&mut w, c, rb);

    let result = container::consume_total_types(
        &mut w,
        c,
        &[&TEST_REAGENT_A, &TEST_REAGENT_B],
        &[5, 5],
        true,
        None,
    );
    assert_eq!(result, 1);
    // all-or-nothing invariant
    assert_eq!(amount(&w, ra), 5);
    assert_eq!(amount(&w, rb), 1);
}

#[test]
fn consume_total_nested_container_recurse_counts_child_items() {
    let mut w = World::new();
    let outer = make_container(&mut w);
    let inner = make_container(&mut w);
    add(&mut w, outer, inner);
    let a = make_stack(&mut w, &TEST_REAGENT_A, 4, 0);
    add(&mut w, inner, a);
    let b = make_stack(&mut w, &TEST_REAGENT_A, 1, 0);
    add(&mut w, outer, b);

    assert!(container::consume_total(
        &mut w,
        outer,
        &TEST_REAGENT_A,
        5,
        true,
        None
    ));
    // both reagents deleted; the inner container persists in outer
    assert_eq!(w.item(outer).unwrap().items.len(), 1);
    assert!(w.item(inner).unwrap().items.is_empty());
}

#[test]
fn consume_total_callback_fires_per_item_with_delta() {
    let mut w = World::new();
    let c = make_container(&mut w);
    let a = make_stack(&mut w, &TEST_REAGENT_A, 3, 0);
    let b = make_stack(&mut w, &TEST_REAGENT_A, 4, 0);
    add(&mut w, c, a);
    add(&mut w, c, b);

    let mut calls: Vec<(ItemId, i32)> = Vec::new();
    let mut cb = |_: &mut World, item: ItemId, amt: i32| calls.push((item, amt));
    assert!(container::consume_total(
        &mut w,
        c,
        &TEST_REAGENT_A,
        5,
        true,
        Some(&mut cb)
    ));
    assert_eq!(calls, vec![(a, 3), (b, 2)]); // full first stack, partial second
}

#[test]
fn consume_total_grouped_hue_grouping_first_sufficient_group_wins() {
    // Two hue groups: blue (sum 4, insufficient), red (sum 8, sufficient). Red wins, blue is intact.
    let mut w = World::new();
    let c = make_container(&mut w);
    let blue1 = make_stack(&mut w, &TEST_REAGENT_A, 2, 0x10);
    let blue2 = make_stack(&mut w, &TEST_REAGENT_A, 2, 0x10);
    let red1 = make_stack(&mut w, &TEST_REAGENT_A, 5, 0x20);
    let red2 = make_stack(&mut w, &TEST_REAGENT_A, 3, 0x20);
    for i in [blue1, blue2, red1, red2] {
        add(&mut w, c, i);
    }

    let result = container::consume_total_grouped(
        &mut w,
        c,
        &[&TEST_REAGENT_A],
        &[6],
        true,
        None,
        &hue_grouper,
    );
    assert_eq!(result, -1);
    assert_eq!(amount(&w, blue1), 2);
    assert_eq!(amount(&w, blue2), 2);
    // red consumed: 5 + 1 = 6 needed
    assert!(w.item(red1).is_none());
    assert!(w.item(red2).is_some());
    assert_eq!(amount(&w, red2), 2);
}

#[test]
fn consume_total_grouped_grouper_receives_group_leader() {
    let mut w = World::new();
    let c = make_container(&mut w);
    let i1 = make_stack(&mut w, &TEST_REAGENT_A, 1, 0x10);
    let i2 = make_stack(&mut w, &TEST_REAGENT_A, 1, 0x10);
    let i3 = make_stack(&mut w, &TEST_REAGENT_A, 1, 0x10);
    for i in [i1, i2, i3] {
        add(&mut w, c, i);
    }

    let leaders = std::cell::RefCell::new(Vec::new());
    let grouper = |w: &World, a: ItemId, b: ItemId| {
        leaders.borrow_mut().push(a);
        w.item(b).unwrap().hue.cmp(&w.item(a).unwrap().hue) as i32
    };
    container::consume_total_grouped(&mut w, c, &[&TEST_REAGENT_A], &[3], true, None, &grouper);

    // the leader for every comparison in the single group is i1 (the first)
    let leaders = leaders.borrow();
    assert!(!leaders.is_empty());
    assert!(leaders.iter().all(|&s| s == i1));
}

#[test]
fn consume_total_grouped_partial_failure_nothing_consumed_from_any_slot() {
    let mut w = World::new();
    let c = make_container(&mut w);
    let a = make_stack(&mut w, &TEST_REAGENT_A, 5, 0x10);
    let b = make_stack(&mut w, &TEST_REAGENT_B, 1, 0x10);
    add(&mut w, c, a);
    add(&mut w, c, b);

    let result = container::consume_total_grouped(
        &mut w,
        c,
        &[&TEST_REAGENT_A, &TEST_REAGENT_B],
        &[5, 5],
        true,
        None,
        &hue_grouper,
    );
    assert_eq!(result, 1);
    assert_eq!(amount(&w, a), 5);
    assert_eq!(amount(&w, b), 1);
}

#[test]
fn consume_total_grouped_exact_group_amount_succeeds() {
    let mut w = World::new();
    let c = make_container(&mut w);
    let stack = make_stack(&mut w, &TEST_REAGENT_A, 5, 0x10);
    add(&mut w, c, stack);

    let result = container::consume_total_grouped(
        &mut w,
        c,
        &[&TEST_REAGENT_A],
        &[5],
        true,
        None,
        &hue_grouper,
    );
    assert_eq!(result, -1);
    assert!(w.item(stack).is_none());
}

#[test]
fn consume_total_grouped_callback_order_matches_bfs() {
    let mut w = World::new();
    let c = make_container(&mut w);
    let a = make_stack(&mut w, &TEST_REAGENT_A, 3, 0x10);
    let b = make_stack(&mut w, &TEST_REAGENT_A, 4, 0x10);
    add(&mut w, c, a);
    add(&mut w, c, b);

    let mut calls: Vec<(ItemId, i32)> = Vec::new();
    let mut cb = |_: &mut World, item: ItemId, amt: i32| calls.push((item, amt));
    container::consume_total_grouped(
        &mut w,
        c,
        &[&TEST_REAGENT_A],
        &[5],
        true,
        Some(&mut cb),
        &hue_grouper,
    );
    assert_eq!(calls, vec![(a, 3), (b, 2)]);
}

#[test]
fn get_best_group_amount_returns_largest_group_sum() {
    let mut w = World::new();
    let c = make_container(&mut w);
    for (amt, hue) in [(2, 0x10), (2, 0x10), (5, 0x20), (3, 0x20)] {
        let i = make_stack(&mut w, &TEST_REAGENT_A, amt, hue);
        add(&mut w, c, i);
    }
    let best = container::get_best_group_amount(&w, c, &[&TEST_REAGENT_A], true, &hue_grouper);
    assert_eq!(best, 8);
}

#[test]
fn get_best_group_amount_empty_or_no_match_returns_zero() {
    let mut w = World::new();
    let c = make_container(&mut w);
    assert_eq!(
        container::get_best_group_amount(&w, c, &[&TEST_REAGENT_A], true, &hue_grouper),
        0
    );
    let b = make_stack(&mut w, &TEST_REAGENT_B, 5, 0);
    add(&mut w, c, b);
    assert_eq!(
        container::get_best_group_amount(&w, c, &[&TEST_REAGENT_A], true, &hue_grouper),
        0
    );
}

#[test]
fn get_amount_type_array_sums_across_types() {
    let mut w = World::new();
    let c = make_container(&mut w);
    let a = make_stack(&mut w, &TEST_REAGENT_A, 3, 0);
    let b = make_stack(&mut w, &TEST_REAGENT_B, 4, 0);
    let plain = make_stack(&mut w, &ITEM_BASE, 99, 0); // base Item matches neither
    for i in [a, b, plain] {
        add(&mut w, c, i);
    }
    assert_eq!(
        container::get_amount_types(&w, c, &[&TEST_REAGENT_A, &TEST_REAGENT_B], true),
        7
    );
}

#[test]
fn find_item_by_type_with_predicate() {
    let mut w = World::new();
    let c = make_container(&mut w);
    let first = make_stack(&mut w, &TEST_REAGENT_A, 1, 0x10);
    let target = make_stack(&mut w, &TEST_REAGENT_A, 1, 0x20);
    let last = make_stack(&mut w, &TEST_REAGENT_A, 1, 0x10);
    for i in [first, target, last] {
        add(&mut w, c, i);
    }
    let found = enumerable::find_items_by_type(&w, c, true, &TEST_REAGENT_A, |w, i| {
        w.item(i).unwrap().hue == 0x20
    });
    assert_eq!(found.first().copied(), Some(target));
}

#[test]
fn consume_up_to_deletes_exhausted_stacks() {
    let mut w = World::new();
    let c = make_container(&mut w);
    let s1 = make_stack(&mut w, &TEST_REAGENT_A, 3, 0);
    let s2 = make_stack(&mut w, &TEST_REAGENT_A, 3, 0);
    add(&mut w, c, s1);
    add(&mut w, c, s2);

    assert_eq!(
        container::consume_up_to(&mut w, c, &TEST_REAGENT_A, 5, true),
        5
    );
    // BFS: first stack fully (3), second partially (2 of 3 consumed -> 1 remains)
    assert!(w.item(s1).is_none());
    assert!(w.item(s2).is_some());
    assert_eq!(amount(&w, s2), 1);
}

#[test]
fn find_items_by_type_runtime_type_matches_predicate_path() {
    let mut w = World::new();
    let c = make_container(&mut w);
    let a1 = make_stack(&mut w, &TEST_REAGENT_A, 1, 0);
    let b = make_stack(&mut w, &TEST_REAGENT_B, 1, 0);
    let a2 = make_stack(&mut w, &TEST_REAGENT_A, 1, 0);
    let plain = make_stack(&mut w, &ITEM_BASE, 1, 0);
    for i in [a1, b, a2, plain] {
        add(&mut w, c, i);
    }
    let by_type = enumerable::find_items_by_type(&w, c, true, &TEST_REAGENT_A, |_, _| true);
    let by_pred = enumerable::find_items(&w, c, true, |w, i| {
        w.item(i).unwrap().ty.is(&TEST_REAGENT_A)
    });
    assert_eq!(by_type, by_pred);
    assert_eq!(by_type, vec![a1, a2]);
}

#[test]
fn find_items_by_type_type_span_matches_union_of_types() {
    let mut w = World::new();
    let c = make_container(&mut w);
    let a = make_stack(&mut w, &TEST_REAGENT_A, 1, 0);
    let b = make_stack(&mut w, &TEST_REAGENT_B, 1, 0);
    let plain = make_stack(&mut w, &ITEM_BASE, 1, 0);
    for i in [a, b, plain] {
        add(&mut w, c, i);
    }
    let found = enumerable::find_items_by_types(&w, c, true, &[&TEST_REAGENT_A, &TEST_REAGENT_B]);
    assert_eq!(found, vec![a, b]);
}
