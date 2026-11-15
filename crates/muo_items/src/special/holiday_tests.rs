//! Tests of `Holiday Stuff/` (holiday4, holiday5), ToggleItem, the aquarium rewards and the Halloween bones.

use muo_io::{SaveReader, SaveWriter};
use muo_server::World;
use muo_server::ids::ItemId;
use muo_server::item::{Item, ItemType, ops};

use super::{aquarium2, holiday4, holiday5, toggle};

fn save(w: &World, id: ItemId) -> Vec<u8> {
    let ty = w.item(id).unwrap().ty;
    let mut out = SaveWriter::new(true);
    (ty.serialize)(w, id, &mut out);
    out.into_vec()
}

fn load(t: &'static ItemType, bytes: &[u8]) -> (World, ItemId) {
    let mut w = crate::keyed_world();
    let id = w.add_item(Item::new_for_load(
        muo_primitives::Serial::new(0x4000_0001),
        t,
    ));
    let mut r = SaveReader::new(bytes);
    (t.deserialize)(&mut w, id, &mut r).unwrap_or_else(|e| panic!("{}: {e}", t.full_name));
    assert_eq!(r.remaining(), 0, "{} is read to its end", t.full_name);
    (w, id)
}

#[test]
fn string_fields_save_after_the_version_and_come_back() {
    for (t, make) in [
        (
            &holiday4::CUPIDS_ARROW,
            (holiday4::CUPIDS_ARROW.construct.unwrap()),
        ),
        (
            &holiday4::HALLOWEEN_PUMPKIN,
            holiday4::halloween_pumpkin_new,
        ),
    ] {
        let mut w = World::new();
        let id = make(&mut w);
        let bytes = save(&w, id);
        let (w2, id2) = load(t, &bytes);
        assert_eq!(save(&w2, id2), bytes, "{}", t.full_name);
    }
}

#[test]
fn a_painted_mask_is_named_after_its_painter() {
    let mut w = crate::keyed_world();
    let id = (holiday4::PAINTED_DAEMON_MASK.construct.unwrap())(&mut w);
    let item = w.item(id).unwrap();
    let name = (item.ty.default_name)(&w, id).unwrap();
    assert!(name.starts_with("Daemon Mask hand painted by "), "{name}");
    assert!((0x4a92..=0x4a93).contains(&item.item_id));
    let bytes = save(&w, id);
    let (w2, id2) = load(&holiday4::PAINTED_DAEMON_MASK, &bytes);
    assert_eq!(save(&w2, id2), bytes);
    assert_eq!(
        (w2.item(id2).unwrap().ty.default_name)(&w2, id2),
        Some(name)
    );
}

#[test]
fn the_unconstructible_classes_still_round_trip() {
    for (t, make) in [
        (
            &holiday4::ANGEL_DECORATION,
            ops::new_item_with_id as fn(&mut World, &'static ItemType, i32) -> ItemId,
        ),
        (&holiday4::ROCKING_HORSE, ops::new_item_with_id),
    ] {
        let mut w = World::new();
        let id = make(&mut w, t, 0x46FA);
        let bytes = save(&w, id);
        let (w2, id2) = load(t, &bytes);
        assert_eq!(save(&w2, id2), bytes, "{}", t.full_name);
    }
    let mut w = World::new();
    let bones = holiday4::player_bones_new(&mut w, "Ann");
    assert_eq!(w.item(bones).unwrap().name.as_deref(), Some("Ann's bones"));
    assert!((0x0ECA..=0x0ED2).contains(&w.item(bones).unwrap().item_id));
    let bytes = save(&w, bones);
    let (w2, id2) = load(&holiday4::PLAYER_BONES, &bytes);
    assert_eq!(save(&w2, id2), bytes);
}

#[test]
fn a_bear_takes_its_owner_and_three_lines() {
    let mut w = World::new();
    let id = holiday4::st_valentines_panda_new(&mut w, Some("Ann"));
    assert_eq!(w.item(id).unwrap().item_id, 0x48E0);
    let bytes = save(&w, id);
    let (w2, id2) = load(&holiday4::ST_VALENTINES_PANDA, &bytes);
    assert_eq!(save(&w2, id2), bytes);
    let name = (w2.item(id2).unwrap().ty.default_name)(&w2, id2);
    assert_eq!(name.as_deref(), Some("Ann's St. Valentine Bear"));
}

#[test]
fn toggle_item_flips_its_graphic_and_hides_graphic_one() {
    let mut w = World::new();
    let id = toggle::toggle_item_new(&mut w, 0x1, 0x2A0, None);
    assert!(!w.item(id).unwrap().movable());
    toggle::toggle(&mut w, id);
    assert_eq!(w.item(id).unwrap().item_id, 0x2A0);
    assert!(w.item(id).unwrap().visible());
    toggle::toggle(&mut w, id);
    assert_eq!(w.item(id).unwrap().item_id, 0x1);
    assert!(!w.item(id).unwrap().visible());
    let bytes = save(&w, id);
    let (w2, id2) = load(&toggle::TOGGLE_ITEM, &bytes);
    assert_eq!(save(&w2, id2), bytes);
}

#[test]
fn the_winter_items_and_addons_round_trip() {
    let mut w = World::new();
    let dipped = holiday4::light_of_the_winter_solstice_new(&mut w, Some("Ann"));
    let bytes = save(&w, dipped);
    let (w2, id2) = load(&holiday4::LIGHT_OF_THE_WINTER_SOLSTICE, &bytes);
    assert_eq!(save(&w2, id2), bytes);

    let deed = holiday5::mistletoe_deed_new(&mut w, 0x21);
    assert_eq!(w.item(deed).unwrap().hue, 0x21);
    let bytes = save(&w, deed);
    let (w2, id2) = load(&holiday5::MISTLETOE_DEED, &bytes);
    assert_eq!(save(&w2, id2), bytes);

    let flies = holiday5::fireflies_new(&mut w, 0x2336);
    assert!(!w.item(flies).unwrap().movable());
    let bytes = save(&w, flies);
    let (w2, id2) = load(&holiday5::FIREFLIES, &bytes);
    assert_eq!(save(&w2, id2), bytes);
}

#[test]
fn the_aquarium_rewards_derive_from_the_fishing_items() {
    let mut w = World::new();
    let net = (aquarium2::AQUARIUM_FISH_NET.construct.unwrap())(&mut w);
    assert_eq!(w.item(net).unwrap().item_id, 0xDC8);
    let pole = (aquarium2::CAPTAIN_BLACKHEARTS_FISHING_POLE
        .construct
        .unwrap())(&mut w);
    assert_eq!(w.item(pole).unwrap().item_id, 0x0DC0);
    let msg = (aquarium2::AQUARIUM_MESSAGE.construct.unwrap())(&mut w);
    for (t, id) in [
        (&aquarium2::AQUARIUM_FISH_NET, net),
        (&aquarium2::CAPTAIN_BLACKHEARTS_FISHING_POLE, pole),
        (&aquarium2::AQUARIUM_MESSAGE, msg),
    ] {
        let bytes = save(&w, id);
        let (w2, id2) = load(t, &bytes);
        assert_eq!(save(&w2, id2), bytes, "{}", t.full_name);
    }
}

#[test]
fn st_valentines_bear_edit_limit_is_a_property_and_not_saved() {
    use muo_content::props::Value;
    use muo_server::ids::EntityId;
    let mut w = World::new();
    let id = super::holiday4::st_valentines_panda_new(&mut w, None);
    let def = &super::holiday4::BEAR_EDIT_LIMIT_PROPS[0];
    assert_eq!(def.name, "EditLimit");
    let d = muo_primitives::DateTime::from_ymd(2026, 2, 14).unwrap();
    (def.set.unwrap())(&mut w, EntityId::Item(id), Value::DateTime(d)).unwrap();
    assert!(matches!((def.get)(&mut w, EntityId::Item(id)), Value::DateTime(x) if x == d));
    let ty = w.item(id).unwrap().ty;
    let mut a = muo_io::SaveWriter::new(true);
    (ty.serialize)(&w, id, &mut a);
    let id2 = super::holiday4::st_valentines_panda_new(&mut w, None);
    let mut b = muo_io::SaveWriter::new(true);
    (ty.serialize)(&w, id2, &mut b);
    assert_eq!(
        a.as_slice().len(),
        b.as_slice().len(),
        "the limit adds nothing to the save"
    );
}
