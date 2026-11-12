//! Item tests: the save layout against bytes from a real save, containers, decay.

use muo_io::{SaveReader, SaveWriter};
use muo_primitives::{DateTime, Serial, TimeSpan};

use super::container::{self, CONTAINER};
use super::decay;
use super::ops;
use super::*;
use crate::ids::{EntityId, ItemId, MobileId};
use crate::mobile::MOBILE_BASE;
use crate::world::World;

fn hex(s: &str) -> Vec<u8> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}

fn add(w: &mut World, serial: u32, ty: &'static ItemType) -> ItemId {
    w.add_item(Item::new_at(Serial::new(serial), ty, DateTime::MIN))
}

/// The Backpack record 0x40000003 of `Saves/Items` of a reference save, up to the `Backpack`'s own version int:
/// `Item` (version 11) and then `Container` (version 0, no flags).
const BACKPACK: &str = "0b000000900610040dd36a31f937df08f51c15010000000400000004000040060000400700004005000040010000000000";

fn backpack(w: &mut World) -> ItemId {
    w.new_mobile(&MOBILE_BASE); // serial 1
    let cont = add(w, 0x4000_0003, &CONTAINER);
    let kids = [0x4000_0004, 0x4000_0006, 0x4000_0007, 0x4000_0005].map(|s| add(w, s, &ITEM_BASE));
    let last_moved = i64::from_le_bytes(hex("0dd36a31f937df08").try_into().unwrap());
    let i = w.item_mut(cont).unwrap();
    i.item_id = 0xE75;
    i.layer = crate::layer::Layer::Backpack;
    i.parent = Some(EntityId::Mobile(MobileId(Serial::new(1))));
    i.items = kids.to_vec();
    i.set_flag(ImplFlag::MOVABLE, false);
    i.last_moved = DateTime::from_ticks(last_moved);
    cont
}

#[test]
fn container_record_matches_the_reference_save() {
    let mut w = World::new();
    let cont = backpack(&mut w);
    let mut out = SaveWriter::new(true);
    (CONTAINER.serialize)(&w, cont, &mut out);
    assert_eq!(out.as_slice(), hex(BACKPACK));
}

#[test]
fn container_record_loads() {
    let mut w = World::new();
    let cont = backpack(&mut w);
    let bytes = hex(BACKPACK);

    let loaded = add(&mut w, 0x4000_0010, &CONTAINER);
    let mut r = SaveReader::new(&bytes);
    (CONTAINER.deserialize)(&mut w, loaded, &mut r).unwrap();

    let (a, b) = (w.item(cont).unwrap(), w.item(loaded).unwrap());
    assert_eq!(
        (a.item_id, a.layer, a.parent, &a.items),
        (b.item_id, b.layer, b.parent, &b.items)
    );
    assert_eq!(
        (a.flags, a.last_moved, a.map),
        (b.flags, b.last_moved, b.map)
    );
}

#[test]
fn container_flags_round_trip() {
    let mut w = World::new();
    let cont = container::new_container(&mut w, &CONTAINER, 0xE75);
    container::set_max_items(&mut w, cont, 20);
    container::set_gump_id(&mut w, cont, 0x3D);
    container::set_lift_override(&mut w, cont, true);

    let mut out = SaveWriter::new(true);
    container::serialize_part(&w, cont, &mut out);
    // version 0, flags MaxItems | GumpID | LiftOverride, 20, 0x3D (LiftOverride is a flag only, per the generator)
    assert_eq!(out.as_slice(), [0, 0, 0, 0, 0x0B, 20, 0x3D]);

    let bytes = out.into_vec();
    let other = container::new_container(&mut w, &CONTAINER, 0xE75);
    container::deserialize_part(&mut w, other, &mut SaveReader::new(&bytes)).unwrap();
    assert_eq!(container::max_items(&w, other), 20);
    assert_eq!(container::gump_id(&w, other), 0x3D);
    assert!(container::lift_override(&w, other));
}

#[test]
fn adding_and_removing_updates_totals() {
    let mut w = World::new();
    let cont = container::new_container(&mut w, &CONTAINER, 0xE75);
    let item = w.new_item(&ITEM_BASE);
    w.item_mut(item).unwrap().weight = 2.0;

    (CONTAINER.add_item)(&mut w, cont, item);
    assert_eq!(ops::get_total(&w, cont, TotalType::Items), 1);
    assert_eq!(ops::get_total(&w, cont, TotalType::Weight), 2);
    assert_eq!(w.item(item).unwrap().parent, Some(EntityId::Item(cont)));
    assert_eq!(w.item(cont).unwrap().version, 1);

    (CONTAINER.remove_item)(&mut w, cont, item);
    assert_eq!(ops::get_total(&w, cont, TotalType::Items), 0);
    assert_eq!(ops::get_total(&w, cont, TotalType::Weight), 0);
    assert_eq!(w.item(item).unwrap().parent, None);
}

#[test]
fn items_are_found_breadth_first() {
    let mut w = World::new();
    let outer = container::new_container(&mut w, &CONTAINER, 0xE75);
    let inner = container::new_container(&mut w, &CONTAINER, 0xE75);
    let (a, b, c) = (
        w.new_item(&ITEM_BASE),
        w.new_item(&ITEM_BASE),
        w.new_item(&ITEM_BASE),
    );
    (CONTAINER.add_item)(&mut w, outer, inner);
    (CONTAINER.add_item)(&mut w, outer, a);
    (CONTAINER.add_item)(&mut w, inner, b);
    (CONTAINER.add_item)(&mut w, inner, c);

    let all = enumerable::find_items(&w, outer, true, |_, _| true);
    assert_eq!(all, vec![inner, a, b, c]);
    let top = enumerable::find_items(&w, outer, false, |_, _| true);
    assert_eq!(top, vec![inner, a]);
}

#[test]
fn consume_total_deletes_and_reduces() {
    let mut w = World::new();
    let cont = container::new_container(&mut w, &CONTAINER, 0xE75);
    let (a, b) = (w.new_item(&ITEM_BASE), w.new_item(&ITEM_BASE));
    for i in [a, b] {
        ops::set_stackable(&mut w, i, true);
        ops::set_amount(&mut w, i, 10);
        (CONTAINER.add_item)(&mut w, cont, i);
    }
    assert_eq!(container::get_amount(&w, cont, &ITEM_BASE, true), 20);
    assert!(!container::consume_total(
        &mut w, cont, &ITEM_BASE, 25, true, None
    ));
    assert!(container::consume_total(
        &mut w, cont, &ITEM_BASE, 15, true, None
    ));
    assert!(w.item(a).is_none());
    assert_eq!(w.item(b).unwrap().amount, 5);
}

#[test]
fn ground_items_decay() {
    let mut w = World::new();
    decay::configure(&mut w, decay::DecayConfig::default());
    let item = w.new_item(&ITEM_BASE);
    w.item_mut(item).unwrap().map = Some(crate::map::MapId(0));
    ops::update_decay_registration(&mut w, item);
    assert!(decay::is_registered(&w, item));
    assert_eq!(
        ops::scheduled_decay_time(&w, item),
        w.now + TimeSpan::from_hours(1)
    );

    // Not yet due.
    let later = w.now + TimeSpan::from_mins(10);
    decay::process_tick(&mut w, later);
    assert!(w.item(item).is_some());

    // Walk the wheel until the hour is up.
    let mut now = w.now;
    for _ in 0..14 {
        now += TimeSpan::from_mins(5);
        decay::process_tick(&mut w, now);
    }
    assert!(w.item(item).is_none(), "decayed");
}

#[test]
fn picking_up_stops_decay() {
    let mut w = World::new();
    decay::configure(&mut w, decay::DecayConfig::default());
    let cont = container::new_container(&mut w, &CONTAINER, 0xE75);
    let item = w.new_item(&ITEM_BASE);
    w.item_mut(item).unwrap().map = Some(crate::map::MapId(0));
    ops::update_decay_registration(&mut w, item);
    assert!(decay::is_registered(&w, item));

    (CONTAINER.add_item)(&mut w, cont, item);
    assert!(!decay::is_registered(&w, item));
}

#[test]
fn containers_cfg_parses() {
    let t = container::parse_containers_cfg(
        "# comment\n0x3C\t44 65 142 94\t0x48\t0xE75,0xE76\nbad line\n0x3D\t0 0 1 1\t0x49\t0xE75\n",
    );
    assert_eq!(t.default.gump_id, 0x3C);
    let mut w = World::new();
    *w.resources.get_mut::<container::ContainerDataTable>() = t;
    assert_eq!(container::get_data(&w, 0xE76).gump_id, 0x3C);
    assert_eq!(
        container::get_data(&w, 0xE75).gump_id,
        0x3C,
        "the first entry of an id wins"
    );
    assert_eq!(
        container::get_data(&w, 0x1).drop_sound,
        0x48,
        "unknown ids use the default"
    );
}

mod detach_order {
    use super::*;
    use crate::mobile::MobileType;
    use std::cell::RefCell;

    thread_local!(static SEEN: RefCell<Vec<(bool, bool, bool)>> = const { RefCell::new(Vec::new()) });

    fn item_hook(w: &mut World, id: ItemId) {
        let own = w.item(id).is_some_and(|i| i.deleted());
        let found = w.find_item(id.0).is_some();
        SEEN.with(|s| s.borrow_mut().push((own, found, w.item(id).is_some())));
    }

    fn mobile_hook(w: &mut World, id: MobileId) {
        let found = w.find_mobile(id.0).is_some();
        SEEN.with(|s| s.borrow_mut().push((true, found, w.mobile(id).is_some())));
    }

    static PROBE_ITEM: ItemType = ItemType {
        on_after_delete: item_hook,
        ..ITEM_BASE
    };
    static PROBE_MOBILE: MobileType = MobileType {
        on_after_delete: mobile_hook,
        ..MOBILE_BASE
    };

    // OnAfterDelete runs after World.RemoveEntity: lookups by serial fail, the object itself still reads.
    #[test]
    fn item_is_hidden_from_lookups_but_readable_in_on_after_delete() {
        SEEN.with(|s| s.borrow_mut().clear());
        let mut w = World::new();
        let i = w.new_item(&PROBE_ITEM);
        base::delete(&mut w, i);
        assert_eq!(SEEN.with(|s| s.borrow().clone()), vec![(true, false, true)]);
        assert!(w.item(i).is_none() && w.find_item(i.0).is_none() && w.detaching.is_empty());
    }

    #[test]
    fn mobile_is_hidden_from_lookups_but_readable_in_on_after_delete() {
        SEEN.with(|s| s.borrow_mut().clear());
        let mut w = World::new();
        let m = w.new_mobile(&PROBE_MOBILE);
        crate::mobile::part1::lifecycle::delete(&mut w, m);
        assert_eq!(SEEN.with(|s| s.borrow().clone()), vec![(true, false, true)]);
        assert!(w.mobile(m).is_none() && w.detaching.is_empty());
    }
}

#[test]
fn read1_dupe_copies_temp_and_saved_flags() {
    // CopyProperties copies every public settable property without [IgnoreDupe], TempFlags and SavedFlags included.
    let mut w = World::new();
    let a = w.new_item(&ITEM_BASE);
    let b = w.new_item(&ITEM_BASE);
    ops::set_temp_flags(&mut w, a, 0x5);
    ops::set_saved_flags(&mut w, a, 0x6);
    (ITEM_BASE.dupe)(&mut w, a, b);
    let b = w.item(b).unwrap();
    assert_eq!((b.temp_flags, b.saved_flags), (0x5, 0x6));
}

/// `(Layer)b` and `(LootType)b` keep a byte without a name, so the save writes it back as read.
#[test]
fn unnamed_layer_and_loot_type_bytes_survive_a_save() {
    let mut w = World::new();
    let item = w.new_item(&ITEM_BASE);
    {
        let i = w.item_mut(item).unwrap();
        i.layer = Layer::from_raw(0x1E);
        i.loot_type = LootType::from_raw(9);
    }
    let mut out = SaveWriter::new(true);
    (ITEM_BASE.serialize)(&w, item, &mut out);
    let bytes = out.into_vec();

    let other = w.new_item(&ITEM_BASE);
    (ITEM_BASE.deserialize)(&mut w, other, &mut SaveReader::new(&bytes)).unwrap();
    let i = w.item(other).unwrap();
    assert_eq!((i.layer.0, i.loot_type.0), (0x1E, 9));
}
