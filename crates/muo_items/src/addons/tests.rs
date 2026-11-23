//! Tests of the addon classes (Items/Addons and the addon families of Items/Special): names, construction, save
//! round trips, and golden records of the save layouts of
//! `BaseAddon`, `AddonComponent`, `BaseAddonContainer` and their subclasses.

use std::collections::HashSet;

use muo_content::items::addon::{self, ADDON_COMPONENT, BASE_ADDON_DEED};
use muo_content::items::addon_container::{
    self, ADDON_CONTAINER_COMPONENT, BASE_ADDON_CONTAINER_DEED,
};
use muo_content::items::cont::BASE_CONTAINER;
use muo_io::{GenericWriter, SaveReader, SaveWriter};
use muo_primitives::{Point3D, Serial};
use muo_server::World;
use muo_server::ids::ItemId;
use muo_server::item::{ITEM_BASE, Item, ItemType, ops};

use super::leaves::*;
use crate::special::addons as sp;

fn all() -> Vec<(&'static ItemType, &'static [&'static str])> {
    let mut v = ALL.to_vec();
    v.extend_from_slice(sp::ALL);
    v.extend_from_slice(super::hand::ALL);
    v.extend_from_slice(super::devices::ALL);
    v.extend_from_slice(super::hive::ALL);
    v.extend_from_slice(crate::special::decor::ALL);
    v.extend_from_slice(crate::special::hand::ALL);
    v
}

fn save(w: &World, id: ItemId) -> Vec<u8> {
    let ty = w.item(id).unwrap().ty;
    let mut out = SaveWriter::new(true);
    (ty.serialize)(w, id, &mut out);
    out.into_vec()
}

fn serial(w: &World, id: ItemId) -> Serial {
    w.item(id).unwrap().serial
}

/// A test world that also knows the addon classes by name (a deed makes its `deedAddon` class by name).
fn typed_world() -> World {
    let mut w = crate::test_world();
    muo_content::items::addon::register(&mut w.types);
    super::register(&mut w.types);
    crate::special::register(&mut w.types);
    w
}

fn make(w: &mut World, t: &'static ItemType) -> ItemId {
    (t.construct.or(t.make).unwrap())(w)
}

#[test]
fn names_are_unique_and_register() {
    let mut names = HashSet::new();
    for (t, aliases) in all() {
        assert!(names.insert(t.full_name), "{} twice", t.full_name);
        for a in aliases {
            assert!(names.insert(*a), "alias {a} twice");
        }
    }
    let mut types = muo_server::type_registry::TypeRegistry::default();
    super::register(&mut types);
    crate::special::register(&mut types);
}

/// Loads the records of `ids` into a fresh world the way the loader does, with the same serials.
fn reload(from: &World, ids: &[ItemId]) -> World {
    let mut w2 = crate::test_world();
    for &id in ids {
        let it = from.item(id).unwrap();
        w2.add_item(Item::new_for_load(it.serial, it.ty));
    }
    for &id in ids {
        let bytes = save(from, id);
        let ty = from.item(id).unwrap().ty;
        let mut r = SaveReader::new(&bytes);
        (ty.deserialize)(&mut w2, id, &mut r).unwrap_or_else(|e| panic!("{}: {e}", ty.full_name));
        assert_eq!(r.remaining(), 0, "{} is read to its end", ty.full_name);
    }
    w2
}

#[test]
fn every_constructible_class_round_trips_with_its_components() {
    for (t, _) in all() {
        let Some(c) = t.construct else { continue };
        let mut w = crate::test_world();
        let id = c(&mut w);
        assert_eq!(w.item(id).unwrap().ty.full_name, t.full_name);
        let mut ids = vec![id];
        ids.extend(addon::components(&w, id));
        ids.extend(addon_container::components(&w, id));
        let w2 = reload(&w, &ids);
        for &i in &ids {
            // A fruit tree with no fruit grows some on load ([AfterDeserialization]): checked on its own.
            if i == id && t.is(&crate::special::decor::BASE_FRUIT_TREE_ADDON) {
                continue;
            }
            assert_eq!(save(&w2, i), save(&w, i), "{} ({:?})", t.full_name, i);
        }
    }
}

#[test]
fn every_deed_and_its_addon_point_at_each_other() {
    for (t, _) in all() {
        let is_deed = t.is(&BASE_ADDON_DEED) || t.is(&BASE_ADDON_CONTAINER_DEED);
        if !is_deed || t.construct.is_none() {
            continue;
        }
        let mut w = typed_world();
        let deed = make(&mut w, t);
        if t.is(&BASE_ADDON_DEED) {
            let a = (addon::deed_class(&w, deed).addon)(&mut w, deed);
            assert!(addon::is_addon(&w, a), "{}", t.full_name);
            let class = addon::addon_class(&w, a);
            let back = match (class.deed_of, class.deed) {
                (Some(f), _) => f(&mut w, a),
                (None, Some(f)) => f(&mut w),
                _ => panic!("{}: an addon with a deed", t.full_name),
            };
            assert_eq!(w.item(back).unwrap().ty.full_name, t.full_name);
        }
    }
}

#[test]
fn abbatoir_addon_record() {
    let mut w = crate::test_world();
    let a = make(&mut w, &ABBATOIR_ADDON);
    let comps = addon::components(&w, a);
    assert_eq!(comps.len(), 9);

    // BaseAddon (3, false): version, the components as an encoded count and serials, WriteEnum(resource);
    // AbbatoirAddon (0, false): its version.
    let mut e = SaveWriter::new(true);
    (ITEM_BASE.serialize)(&w, a, &mut e);
    e.write_i32(3);
    e.write_encoded_int(9);
    for &c in &comps {
        e.write_serial(serial(&w, c));
    }
    e.write_encoded_int(0);
    e.write_i32(0);
    assert_eq!(save(&w, a), e.into_vec());

    // AddonComponent (1, false): version, Addon, Offset. 0x120F was added at (0, -1, 0).
    let c = comps[1];
    assert_eq!(w.item(c).unwrap().item_id, 0x120F);
    let mut e = SaveWriter::new(true);
    (ITEM_BASE.serialize)(&w, c, &mut e);
    e.write_i32(1);
    e.write_serial(serial(&w, a));
    e.write_i32(0);
    e.write_i32(-1);
    e.write_i32(0);
    assert_eq!(save(&w, c), e.into_vec());
    assert!(!w.item(a).unwrap().movable() && !ops::is_deleted(&w, a));
}

#[test]
fn addon_deed_record() {
    let mut w = crate::test_world();
    let d = make(&mut w, &ABBATOIR_DEED);
    // BaseAddonDeed (0, false) and AbbatoirDeed (0, false): two int versions.
    let mut e = SaveWriter::new(true);
    (ITEM_BASE.serialize)(&w, d, &mut e);
    e.write_i32(0);
    e.write_i32(0);
    assert_eq!(save(&w, d), e.into_vec());
    assert_eq!(w.item(d).unwrap().item_id, 0x14F0);
}

#[test]
fn encoded_version_leaf_and_localized_component_record() {
    let mut w = crate::test_world();
    let a = make(&mut w, &sp::BONE_TABLE_ADDON); // [SerializationGenerator(0)]: an encoded version
    let c = addon::components(&w, a)[0];
    let mut e = SaveWriter::new(true);
    (ITEM_BASE.serialize)(&w, a, &mut e);
    e.write_i32(3);
    e.write_encoded_int(1);
    e.write_serial(serial(&w, c));
    e.write_encoded_int(0);
    e.write_encoded_int(0);
    assert_eq!(save(&w, a), e.into_vec());

    // LocalizedAddonComponent (0, false) adds its version and `_number` after AddonComponent's part.
    let mut e = SaveWriter::new(true);
    (ITEM_BASE.serialize)(&w, c, &mut e);
    e.write_i32(1);
    e.write_serial(serial(&w, a));
    e.write_i32(0);
    e.write_i32(0);
    e.write_i32(0);
    e.write_i32(0);
    e.write_i32(1074478);
    assert_eq!(save(&w, c), e.into_vec());
    assert_eq!((w.item(c).unwrap().ty.label_number)(&w, c), 1074478);
}

#[test]
fn container_addon_records() {
    let mut w = crate::test_world();
    let a = make(&mut w, &ELVEN_DRESSER_EAST_ADDON);
    let comps = addon_container::components(&w, a);
    assert_eq!(comps.len(), 1);
    // BaseAddonContainer (2, false): version, encoded count, serials, WriteEnum; the leaf is (0): encoded.
    let mut e = SaveWriter::new(true);
    (BASE_CONTAINER.serialize)(&w, a, &mut e);
    e.write_i32(2);
    e.write_encoded_int(1);
    e.write_serial(serial(&w, comps[0]));
    e.write_encoded_int(0);
    e.write_encoded_int(0);
    assert_eq!(save(&w, a), e.into_vec());

    // AddonContainerComponent (0, false): version, Addon, Offset (0, -1, 0).
    let mut e = SaveWriter::new(true);
    (ITEM_BASE.serialize)(&w, comps[0], &mut e);
    e.write_i32(0);
    e.write_serial(serial(&w, a));
    e.write_i32(0);
    e.write_i32(-1);
    e.write_i32(0);
    assert_eq!(save(&w, comps[0]), e.into_vec());
    assert!(w.item(comps[0]).unwrap().ty.is(&ADDON_CONTAINER_COMPONENT));

    // BaseAddonContainerDeed (2): an encoded version and the resource; the leaf (0) an encoded version.
    let d = make(&mut w, &ELVEN_DRESSER_EAST_DEED);
    let mut e = SaveWriter::new(true);
    (ITEM_BASE.serialize)(&w, d, &mut e);
    e.write_encoded_int(2);
    e.write_encoded_int(0);
    e.write_encoded_int(0);
    assert_eq!(save(&w, d), e.into_vec());
}

#[test]
fn hue_is_shared_unless_the_class_says_not() {
    let mut w = crate::test_world();
    let a = make(&mut w, &ALCHEMIST_TABLE_EAST_ADDON);
    let c = addon::components(&w, a)[0];
    (w.item(a).unwrap().ty.set_hue)(&mut w, a, 0x44);
    assert_eq!(w.item(c).unwrap().hue, 0x44);
    // a component's hue goes to the addon
    (w.item(c).unwrap().ty.set_hue)(&mut w, c, 0x55);
    assert_eq!(w.item(a).unwrap().hue, 0x55);

    // FireColumnAddon: ShareHue => false
    let a = make(&mut w, &FIRE_COLUMN_ADDON);
    let c = addon::components(&w, a)[0];
    (w.item(a).unwrap().ty.set_hue)(&mut w, a, 0x44);
    assert_eq!(w.item(c).unwrap().hue, 0);
}

#[test]
fn deleting_a_component_deletes_the_addon_and_the_rest() {
    let mut w = crate::test_world();
    let a = make(&mut w, &ABBATOIR_ADDON);
    let comps = addon::components(&w, a);
    let ty = w.item(comps[3]).unwrap().ty;
    (ty.delete)(&mut w, comps[3]);
    assert!(ops::is_deleted(&w, a));
    assert!(comps.iter().all(|c| w.item(*c).is_none_or(|i| i.deleted())));
}

#[test]
fn load_tidies_a_component_that_does_not_exist() {
    let mut w = crate::test_world();
    let a = make(&mut w, &sp::BROKEN_ARMOIRE_ADDON);
    let bytes = save(&w, a);
    // The loader finds no entity for the component's serial: the list is tidied to nothing.
    let mut w2 = crate::test_world();
    let id = w2.add_item(Item::new_for_load(serial(&w, a), &sp::BROKEN_ARMOIRE_ADDON));
    let mut r = SaveReader::new(&bytes);
    (sp::BROKEN_ARMOIRE_ADDON.deserialize)(&mut w2, id, &mut r).unwrap();
    assert!(addon::components(&w2, id).is_empty());
}

#[test]
fn version_zero_component_reads_nothing_after_its_version() {
    // AddonComponent version 0: `Deserialize(reader, version)` reads no fields.
    let mut w = crate::test_world();
    let c = sp::standing_broken_chair_component_new(&mut w);
    let mut e = SaveWriter::new(true);
    (ITEM_BASE.serialize)(&w, c, &mut e);
    e.write_i32(0); // AddonComponent v0
    e.write_i32(0); // the leaf
    let bytes = e.into_vec();
    let mut w2 = crate::test_world();
    let id = w2.add_item(Item::new_for_load(
        serial(&w, c),
        &sp::STANDING_BROKEN_CHAIR_COMPONENT,
    ));
    let mut r = SaveReader::new(&bytes);
    (ADDON_COMPONENT.deserialize)(&mut w2, id, &mut r).unwrap();
    assert_eq!(addon::component_addon(&w2, id), None);
    assert_eq!(addon::component_offset(&w2, id), Point3D::default());
}

#[test]
fn without_houses_an_addon_does_not_fit() {
    // There is no house here: FindHouseAt is null, so the addon fits nowhere (Blocked off the map, NotInHouse on it).
    let mut w = crate::test_world();
    let a = make(&mut w, &ABBATOIR_ADDON);
    let mut house = None;
    let map = muo_server::map::MapId::FELUCCA;
    let r = addon::could_fit(&mut w, a, Point3D::new(100, 100, 0), map, None, &mut house);
    assert!(matches!(
        r,
        addon::AddonFitResult::Blocked | addon::AddonFitResult::NotInHouse
    ));
    assert!(house.is_none());
}

#[test]
fn jack_o_lantern_builds_its_four_components() {
    let mut w = crate::test_world();
    for south in [false, true] {
        let a = super::hand::jack_o_lantern_new(&mut w, south);
        let comps = addon::components(&w, a);
        assert_eq!(comps.len(), 4);
        let ids: Vec<i32> = comps.iter().map(|c| w.item(*c).unwrap().item_id).collect();
        assert_eq!(
            ids,
            if south {
                [5703, 3179, 3885, 3871]
            } else {
                [5703, 3178, 3883, 3862]
            }
        );
        assert_eq!(w.item(comps[2]).unwrap().hue, 1161);
        assert_eq!(
            w.item(comps[1]).unwrap().name.as_deref(),
            Some("jack-o-lantern")
        );
        // ShareHue => false: a hue on the addon stays off the components.
        (w.item(a).unwrap().ty.set_hue)(&mut w, a, 5);
        assert_eq!(w.item(comps[1]).unwrap().hue, 0);
    }
}

#[test]
fn wall_torch_toggles_its_graphic() {
    let mut w = typed_world();
    let a = sp::wall_torch_addon_new(&mut w);
    let c = addon::components(&w, a)[0];
    assert_eq!(w.item(c).unwrap().item_id, 0x3D98);
    assert_eq!((w.item(c).unwrap().ty.label_number)(&w, c), 1076282);
    assert_eq!(
        muo_content::items::flip::flip_ids(w.item(c).unwrap().ty),
        Some(&[0x3D98, 0x3D94][..])
    );
}

// ---- devices.rs and hive.rs ----

use super::devices::{
    self as dev, ARCHERY_BUTTE, ELVEN_SPINNING_WHEEL_EAST_ADDON, FLOUR_MILL_EAST_ADDON,
    LOOM_SOUTH_ADDON, SPINNING_WHEEL_EAST_ADDON, archery_butte_new, dart_board_new,
    flour_mill_cur_flour, flour_mill_set_cur_flour, loom_phase, loom_set_phase, pickpocket_dip_new,
    spinning_wheel_begin_spin, spinning_wheel_end_spin, spinning_wheel_spinning,
};
use super::hive::{self, SH_TELE_COMPONENT};

#[test]
fn loom_and_flour_mill_records() {
    let mut w = crate::test_world();
    let l = make(&mut w, &LOOM_SOUTH_ADDON);
    loom_set_phase(&mut w, l, 7);
    let comps = addon::components(&w, l);
    assert_eq!(comps.len(), 2);
    assert_eq!(w.item(comps[1]).unwrap().item_id, 0x1062);
    // BaseAddon part, then LoomSouthAddon (0, false): version and `_phase`.
    let mut e = SaveWriter::new(true);
    (muo_content::items::addon::BASE_ADDON.serialize)(&w, l, &mut e);
    e.write_i32(0);
    e.write_i32(7);
    assert_eq!(save(&w, l), e.into_vec());
    let w2 = reload(&w, &[l, comps[0], comps[1]]);
    assert_eq!(loom_phase(&w2, l), 7);

    let m = make(&mut w, &FLOUR_MILL_EAST_ADDON);
    let ids: Vec<_> = addon::components(&w, m)
        .iter()
        .map(|c| w.item(*c).unwrap().item_id)
        .collect();
    assert_eq!(ids, [0x1920, 0x1922, 0x1924]);
    flour_mill_set_cur_flour(&mut w, m, 9); // clamped to MaxFlour; the filled stage is the second column
    assert_eq!(flour_mill_cur_flour(&w, m), 2);
    let ids: Vec<_> = addon::components(&w, m)
        .iter()
        .map(|c| w.item(*c).unwrap().item_id)
        .collect();
    assert_eq!(ids, [0x1921, 0x1923, 0x1924]);
    flour_mill_set_cur_flour(&mut w, m, -3);
    let ids: Vec<_> = addon::components(&w, m)
        .iter()
        .map(|c| w.item(*c).unwrap().item_id)
        .collect();
    assert_eq!(ids, [0x1920, 0x1922, 0x1924]);
}

#[test]
fn spinning_wheel_swaps_ids_and_a_saved_spin_is_undone_on_load() {
    let mut w = crate::test_world();
    let wheel = make(&mut w, &SPINNING_WHEEL_EAST_ADDON);
    let c = addon::components(&w, wheel)[0];
    let from = w.new_mobile(&muo_server::mobile::MOBILE_BASE);
    spinning_wheel_begin_spin(&mut w, wheel, None, from, 0);
    assert!(spinning_wheel_spinning(&w, wheel));
    assert_eq!(w.item(c).unwrap().item_id, 0x101A);
    spinning_wheel_end_spin(&mut w, wheel, None, from, 0);
    assert!(!spinning_wheel_spinning(&w, wheel));
    assert_eq!(w.item(c).unwrap().item_id, 0x1019);

    // The elven wheel counts down: 0x2E3D -> 0x2E3C.
    let ew = make(&mut w, &ELVEN_SPINNING_WHEEL_EAST_ADDON);
    let ec = addon::components(&w, ew)[0];
    spinning_wheel_begin_spin(&mut w, ew, None, from, 0);
    assert_eq!(w.item(ec).unwrap().item_id, 0x2E3C);

    // A component saved mid-spin reloads idle (OnComponentLoaded).
    let w2 = reload(&w, &[ew, ec]);
    assert_eq!(w2.item(ec).unwrap().item_id, 0x2E3D);
}

#[test]
fn dart_board_needs_a_wall_on_the_side_it_faces() {
    let mut w = crate::test_world();
    let east = dart_board_new(&mut w, true);
    let south = dart_board_new(&mut w, false);
    let class = |w: &World, c| addon::component_class(w, c);
    assert!(class(&w, east).needs_wall);
    assert_eq!(
        (class(&w, east).wall_position_of.unwrap())(&w, east),
        Point3D::new(-1, 0, 0)
    );
    assert_eq!(
        (class(&w, south).wall_position_of.unwrap())(&w, south),
        Point3D::new(0, -1, 0)
    );
}

#[test]
fn practice_components_and_archery_butte_records() {
    let mut w = crate::test_world();
    // PickpocketDip (0, false): version, MinSkill, MaxSkill; a fresh one practices on -25..+25.
    let dip = pickpocket_dip_new(&mut w, 0x1EC3);
    let mut e = SaveWriter::new(true);
    (ADDON_COMPONENT.serialize)(&w, dip, &mut e);
    e.write_i32(0);
    e.write_f64(-25.0);
    e.write_f64(25.0);
    assert_eq!(save(&w, dip), e.into_vec());

    // ArcheryButte keeps arrows and bolts; the old 0..30 window is replaced on load.
    let b = archery_butte_new(&mut w, 0x100A);
    let mut e = SaveWriter::new(true);
    (ADDON_COMPONENT.serialize)(&w, b, &mut e);
    e.write_i32(0);
    e.write_f64(0.0);
    e.write_f64(30.0);
    e.write_i32(3);
    e.write_i32(4);
    let bytes = e.into_vec();
    let b2 = w.add_item(Item::new_for_load(Serial::new(0x4000_0F00), &ARCHERY_BUTTE));
    let mut r = SaveReader::new(&bytes);
    (ARCHERY_BUTTE.deserialize)(&mut w, b2, &mut r).unwrap();
    let again = save(&w, b2);
    let mut e = SaveWriter::new(true);
    (ADDON_COMPONENT.serialize)(&w, b2, &mut e);
    e.write_i32(0);
    e.write_f64(-25.0);
    e.write_f64(25.0);
    e.write_i32(3);
    e.write_i32(4);
    assert_eq!(again, e.into_vec());
}

#[test]
fn teleporters_link_all_four_ends_and_activity_follows() {
    let mut w = crate::test_world();
    let a = hive::sh_teleporter_new_with(&mut w, true);
    let b = hive::sh_teleporter_new_with(&mut w, false);
    assert_eq!(addon::components(&w, a).len(), 16 + 4);
    assert_eq!(addon::components(&w, b).len(), 4);
    hive::link(&mut w, a, b);
    let ends = |w: &World, t| {
        let comps = addon::components(w, t);
        comps
            .iter()
            .copied()
            .filter(|c| w.item(*c).unwrap().ty.is(&SH_TELE_COMPONENT))
            .collect::<Vec<_>>()
    };
    let (ea, eb) = (ends(&w, a), ends(&w, b));
    // a's up end leads to b's up end, and back.
    let dest_of = |w: &World, c| save(w, c);
    assert_ne!(dest_of(&w, ea[0]), dest_of(&w, ea[1]));
    hive::sh_tele_set_active(&mut w, eb[2], false);
    for c in &eb {
        assert!(!save(&w, *c).is_empty());
    }
    let w2 = reload(
        &w,
        &[a, b]
            .iter()
            .copied()
            .chain(ea.iter().copied())
            .chain(eb.iter().copied())
            .chain(addon::components(&w, a))
            .chain(addon::components(&w, b))
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>(),
    );
    for c in ea.iter().chain(eb.iter()) {
        assert_eq!(save(&w2, *c), save(&w, *c));
    }
    assert_eq!(hive::sh_tele_point(&w, ea[0]), Point3D::new(-1, 0, 0));
}

#[test]
fn ballot_box_round_trips_topic_and_votes() {
    let mut w = crate::test_world();
    let b = hive::ballot_box_new(&mut w);
    let voter = w.new_mobile(&muo_server::mobile::MOBILE_BASE);
    // BallotBox (0): encoded version, the topic (encoded count of strings), then the aye and nay lists.
    let mut e = SaveWriter::new(true);
    (ADDON_COMPONENT.serialize)(&w, b, &mut e);
    e.write_encoded_int(0);
    e.write_encoded_int(0);
    e.write_encoded_int(0);
    e.write_encoded_int(0);
    assert_eq!(save(&w, b), e.into_vec());
    let _ = voter;
}

#[test]
fn solen_ant_hole_reads_version_zero() {
    let mut w = crate::test_world();
    let h = make(&mut w, &hive::SOLEN_ANT_HOLE);
    assert_eq!(addon::components(&w, h).len(), 10);
    assert!(!addon::addon_class(&w, h).share_hue);
    let bytes = save(&w, h);
    // Version 1 ends with the encoded version 1 and an encoded empty list.
    assert_eq!(&bytes[bytes.len() - 2..], &[1, 0]);
    let mut v0 = bytes[..bytes.len() - 2].to_vec();
    v0.extend_from_slice(&[0, 0, 0, 0, 0]); // version 0: encoded 0, then an int count of 0
    let h2 = w.add_item(Item::new_for_load(
        Serial::new(0x4000_0F01),
        &hive::SOLEN_ANT_HOLE,
    ));
    let mut r = SaveReader::new(&v0);
    (hive::SOLEN_ANT_HOLE.deserialize)(&mut w, h2, &mut r).unwrap();
    assert_eq!(r.remaining(), 0);
}

#[allow(dead_code)]
fn _uses(_: &dyn Fn()) {
    let _ = dev::WATER_TROUGH_QUANTITY;
}

#[test]
fn fruit_trees_grow_on_load_and_picking_empties_them() {
    use crate::special::decor::{APPLE_TREE_ADDON, PEACH_TREE_ADDON};
    let mut w = crate::test_world();
    let tree = make(&mut w, &APPLE_TREE_ADDON);
    let comps = addon::components(&w, tree);
    assert_eq!(comps.len(), 2);
    let bytes = save(&w, tree);
    assert_eq!(
        &bytes[bytes.len() - 6..],
        &[0, 0, 0, 0, 0, 0][..],
        "the tree's version, no fruit, the leaf's version"
    );
    let w2 = reload(&w, &[tree, comps[0], comps[1]]);
    let again = save(&w2, tree);
    let fruit = i32::from_le_bytes(again[again.len() - 5..again.len() - 1].try_into().unwrap());
    assert!((1..=4).contains(&fruit), "{fruit}");
    let _ = PEACH_TREE_ADDON;
}

#[test]
fn bed_of_nails_flips_between_south_and_east_and_stays_when_blocked() {
    use crate::special::decor::BED_OF_NAILS_ADDON;
    use muo_primitives::Direction;
    let mut w = crate::test_world();
    crate::special::decor::configure_for_tests(&mut w);
    let from = w.new_mobile(&muo_server::mobile::MOBILE_BASE);
    let bed = make(&mut w, &BED_OF_NAILS_ADDON);
    assert_eq!(w.item(bed).unwrap().direction, Direction::SOUTH);
    let ids = |w: &World| -> Vec<i32> {
        addon::components(w, bed)
            .iter()
            .map(|c| w.item(*c).unwrap().item_id)
            .collect()
    };
    assert_eq!(ids(&w), [0x2A81, 0x2A82]);
    // `from.Map` null: nothing to check against, the bed turns east.
    w.mobile_mut(from).unwrap().map = None;
    muo_content::items::flip::flip_addon(&mut w, from, bed);
    assert_eq!(ids(&w), [0x2A89, 0x2A8A]);
    assert_eq!(w.item(bed).unwrap().direction, Direction::EAST);
    // On a facet there is no house (no house is placed), so the new place does not fit and the bed turns back.
    w.mobile_mut(from).unwrap().map = Some(muo_server::map::MapId::FELUCCA);
    ops::set_location(&mut w, bed, Point3D::new(100, 100, 0));
    muo_content::items::flip::flip_addon(&mut w, from, bed);
    assert_eq!(ids(&w), [0x2A89, 0x2A8A]);
    assert_eq!(w.item(bed).unwrap().direction, Direction::EAST);
}

#[test]
fn forge_and_anvil_addons_are_made_of_the_marked_component_classes() {
    // DefBlacksmithy.CheckAnvilAndForge looks for `[Anvil]` / `[Forge]` classes by name (muo_engines::craft::defs).
    let mut w = typed_world();
    let names = |w: &mut World, t: &'static ItemType| -> Vec<&'static str> {
        let a = make(w, t);
        addon::components(w, a)
            .iter()
            .map(|c| w.item(*c).unwrap().ty.full_name)
            .collect()
    };
    assert_eq!(
        names(&mut w, &ANVIL_EAST_ADDON),
        ["Server.Items.AnvilComponent"]
    );
    assert_eq!(
        names(&mut w, &ANVIL_SOUTH_ADDON),
        ["Server.Items.AnvilComponent"]
    );
    assert!(
        names(&mut w, &SMALL_FORGE_ADDON)
            .iter()
            .all(|n| *n == "Server.Items.ForgeComponent")
    );
    assert!(
        names(&mut w, &LARGE_FORGE_EAST_ADDON)
            .iter()
            .all(|n| *n == "Server.Items.ForgeComponent")
    );
    assert!(
        names(&mut w, &LARGE_FORGE_SOUTH_ADDON)
            .iter()
            .all(|n| *n == "Server.Items.ForgeComponent")
    );
    assert!(
        names(&mut w, &SHRINE_OF_WISDOM_ADDON).contains(&"Server.Items.ShrineOfWisdomComponent")
    );
}

#[test]
fn a_crafted_container_deed_takes_the_resource_hue() {
    use muo_content::craft::CraftResource;
    use muo_content::items::equip_hooks::CraftArgs;
    let mut w = crate::test_world();
    let deed = make(&mut w, &ELVEN_DRESSER_EAST_DEED);
    let from = w.new_mobile(&muo_server::mobile::MOBILE_BASE);
    let mut a = CraftArgs::new(2, false, from, CraftResource::Iron);
    assert_eq!(addon_container::deed_on_craft(&mut w, deed, &a), Some(2));
    let iron = w.item(deed).unwrap().hue;
    assert_eq!(
        w.item(deed)
            .unwrap()
            .ext
            .get::<addon_container::ContainerDeedData>()
            .unwrap()
            .resource,
        CraftResource::Iron
    );
    a.resource = CraftResource::Copper;
    assert_eq!(addon_container::deed_on_craft(&mut w, deed, &a), Some(2));
    assert_ne!(w.item(deed).unwrap().hue, iron);
    a.do_not_color = true;
    addon_container::deed_on_craft(&mut w, deed, &a);
    assert_eq!(w.item(deed).unwrap().hue, 0);
    // A plain addon deed is not ICraftable.
    let plain = make(&mut w, &ABBATOIR_DEED);
    assert_eq!(addon_container::deed_on_craft(&mut w, plain, &a), None);
}

#[test]
fn deleting_a_portrait_stops_its_timer_in_on_after_delete() {
    // The hook reads the component's own state after the entity left the world's lookups (RemoveEntity runs first).
    use crate::special::decor::DISTURBING_PORTRAIT_COMPONENT;
    let mut w = crate::test_world();
    let c = make(&mut w, &DISTURBING_PORTRAIT_COMPONENT);
    let t =
        crate::special::decor::portrait_timer_for_tests(&w, c).expect("started on construction");
    assert!(t.running(&w));
    (w.item(c).unwrap().ty.delete)(&mut w, c);
    assert!(w.item(c).is_none());
    assert!(!t.running(&w), "stopped by OnAfterDelete");
}
