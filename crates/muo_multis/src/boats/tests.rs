//! Boats: construction, facing, the save layout and the 0xF6 packet.

use muo_io::{SaveReader, SaveWriter};
use muo_primitives::{Point3D, Serial};
use muo_server::World;
use muo_server::ids::{EntityId, ItemId};
use muo_server::item::{Item, ItemType, multi::BASE_MULTI, ops};
use muo_server::mobile::MOBILE_BASE;

use super::items::*;
use super::parts::{self, HOLD, PLANK, TILLER_MAN};
use super::{BASE_BOAT, get_facing, hold, p_plank, s_plank, set_facing, tiller_man};

fn save(w: &World, id: ItemId) -> Vec<u8> {
    let mut out = SaveWriter::new(true);
    (w.item(id).unwrap().ty.serialize)(w, id, &mut out);
    out.into_vec()
}

fn load(w: &mut World, ty: &'static ItemType, bytes: &[u8], serial: u32) -> ItemId {
    let id = w.add_item(Item::new_for_load(Serial::new(serial), ty));
    let mut r = SaveReader::new(bytes);
    (ty.deserialize)(w, id, &mut r).unwrap_or_else(|e| panic!("{}: {e}", ty.full_name));
    assert_eq!(r.remaining(), 0, "{}", ty.full_name);
    id
}

fn loc(w: &World, id: ItemId) -> Point3D {
    w.item(id).unwrap().location
}

#[test]
fn a_new_boat_has_its_parts_facing_north() {
    let mut w = World::new();
    let b = new_small_boat(&mut w);
    assert_eq!(w.item(b).unwrap().item_id, 0x0);
    assert_eq!(w.item(tiller_man(&w, b).unwrap()).unwrap().item_id, 0x3E4E);
    assert_eq!(w.item(hold(&w, b).unwrap()).unwrap().item_id, 0x3EAE);
    assert_eq!(w.item(p_plank(&w, b).unwrap()).unwrap().item_id, 0x3EB1);
    assert_eq!(w.item(s_plank(&w, b).unwrap()).unwrap().item_id, 0x3EB2);
    assert!(w.item(p_plank(&w, b).unwrap()).unwrap().ty.is(&PLANK));
    assert!(
        w.item(tiller_man(&w, b).unwrap())
            .unwrap()
            .ty
            .is(&TILLER_MAN)
    );
    assert!(w.item(hold(&w, b).unwrap()).unwrap().ty.is(&HOLD));
    assert!(!super::anchored(&w, b));
    assert_eq!(super::data(&w, b).unwrap().next_nav_point, -1);
}

#[test]
fn turning_east_rotates_the_parts_and_swaps_the_graphic() {
    let mut w = World::new();
    let b = new_small_boat(&mut w);
    assert!(set_facing(&mut w, b, 2));
    assert_eq!(get_facing(&w, b).bits(), 2);
    assert_eq!(w.item(b).unwrap().item_id, 0x1, "SmallBoat.EastID");
    assert_eq!(w.item(tiller_man(&w, b).unwrap()).unwrap().item_id, 0x3E55);
    assert_eq!(w.item(hold(&w, b).unwrap()).unwrap().item_id, 0x3E65);
    // tiller man: X + dx * -4, hold: X + dx * 4; the port plank (-2, 0) quarter-turned is (0, -2)
    assert_eq!(loc(&w, tiller_man(&w, b).unwrap()), Point3D::new(-4, 0, 0));
    assert_eq!(loc(&w, hold(&w, b).unwrap()), Point3D::new(4, 0, 0));
    assert_eq!(loc(&w, p_plank(&w, b).unwrap()), Point3D::new(0, -2, 0));
    assert_eq!(loc(&w, s_plank(&w, b).unwrap()), Point3D::new(0, 2, 0));
}

#[test]
fn planks_pick_their_graphic_by_side_and_facing() {
    let mut w = World::new();
    let b = new_small_boat(&mut w);
    let (port, star) = (p_plank(&w, b).unwrap(), s_plank(&w, b).unwrap());
    for (dir, p, s) in [
        (0, 0x3EB1, 0x3EB2),
        (2, 0x3E8A, 0x3E85),
        (4, 0x3EB2, 0x3EB1),
        (6, 0x3E85, 0x3E8A),
    ] {
        parts::plank_set_facing(&mut w, port, dir);
        parts::plank_set_facing(&mut w, star, dir);
        assert_eq!(
            (w.item(port).unwrap().item_id, w.item(star).unwrap().item_id),
            (p, s),
            "facing {dir}"
        );
    }
    parts::plank_set_facing(&mut w, port, 0);
    parts::plank_open(&mut w, port);
    assert!(parts::plank_is_open(&w, port));
    assert_eq!(w.item(port).unwrap().item_id, 0x3ED5);
}

#[test]
fn the_boat_record_is_the_multi_part_then_version_5() {
    let mut w = World::new();
    let b = new_medium_boat(&mut w);
    let bytes = save(&w, b);
    let mut base = SaveWriter::new(true);
    (BASE_MULTI.serialize)(&w, b, &mut base);
    let base = base.into_vec();
    assert!(bytes.starts_with(&base));
    let rest = &bytes[base.len()..];
    assert_eq!(rest[0..4], 5_i32.to_le_bytes()); // BaseBoat version
    assert_eq!(rest[4..8], [0xFF; 4]); // no MapItem
    assert_eq!(rest[8..12], (-1_i32).to_le_bytes()); // NextNavPoint
    assert_eq!(rest[12], 0); // Facing (a byte enum)
    // ..then the anchored time (8 bytes), the owner (4), four parts (16), Anchored (1), ShipName (null), version 0
    let tail = &rest[13 + 8..];
    assert_eq!(tail[0..4], [0xFF; 4]);
    assert_eq!(tail[20], 0, "not anchored");
    assert!(
        bytes.ends_with(&0_i32.to_le_bytes()),
        "the leaf's version int"
    );

    let loaded = load(&mut w, &MEDIUM_BOAT, &bytes, 0x4000_0500);
    assert_eq!(super::tiller_man(&w, loaded), super::tiller_man(&w, b));
    assert_eq!(save(&w, loaded), bytes);
}

#[test]
fn deeds_and_docked_boats_round_trip() {
    let mut w = World::new();
    let d = new_large_boat_deed(&mut w);
    let bytes = save(&w, d);
    // version 0, MultiId 0x10, Offset (0, -1, 0), the leaf's version
    let mut tail = Vec::new();
    for v in [0_i32, 0x10, 0, -1, 0, 0] {
        tail.extend_from_slice(&v.to_le_bytes());
    }
    assert!(bytes.ends_with(&tail));
    let l = load(&mut w, &LARGE_BOAT_DEED, &bytes, 0x4000_0501);
    assert_eq!(save(&w, l), bytes);

    let b = new_small_boat(&mut w);
    super::set_ship_name(&mut w, b, Some("Minnow".to_string()));
    let dock = new_small_docked_boat(&mut w, b);
    let bytes = save(&w, dock);
    let l = load(&mut w, &SMALL_DOCKED_BOAT, &bytes, 0x4000_0502);
    assert_eq!(save(&w, l), bytes);
    assert!(bytes.windows(6).any(|x| x == b"Minnow"));
}

#[test]
fn the_move_packet_is_0xf6_with_every_entity_offset() {
    let mut w = World::new();
    let b = new_small_boat(&mut w);
    ops::set_location(&mut w, b, Point3D::new(100, 200, 5));
    let m = w.new_mobile(&MOBILE_BASE);
    muo_server::mobile::part1::lifecycle::set_location(&mut w, m, Point3D::new(101, 201, 5));
    let serial = w.mobile(m).unwrap().serial.value();
    let boat_serial = w.item(b).unwrap().serial.value();
    let p = super::packets::create_move_boat_hs(&w, b, &[EntityId::Mobile(m)], 0, 4, 1, 0);

    let mut want = vec![0xF6, 0, 28];
    want.extend_from_slice(&boat_serial.to_be_bytes());
    want.extend_from_slice(&[4, 0, 0]); // speed, direction, facing
    for v in [101_i16, 200, 5, 1] {
        want.extend_from_slice(&v.to_be_bytes());
    }
    want.extend_from_slice(&serial.to_be_bytes());
    for v in [102_i16, 201, 5] {
        want.extend_from_slice(&v.to_be_bytes());
    }
    assert_eq!(p, want);
    assert_eq!(p.len(), super::packets::move_boat_hs_packet_length(1));
}

#[test]
fn only_the_boat_class_is_abstract() {
    assert!(BASE_BOAT.construct.is_none());
    assert!(SMALL_BOAT.construct.is_some() && LARGE_DRAGON_BOAT_DEED.construct.is_some());
    assert!(SMALL_DOCKED_BOAT.construct.is_none());
}
