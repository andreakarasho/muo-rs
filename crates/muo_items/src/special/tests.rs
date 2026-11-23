//! Tests of the Items/Special classes: unique names, each class constructs and round-trips its save bytes.

use std::collections::HashSet;

use muo_io::{SaveReader, SaveWriter};
use muo_server::World;
use muo_server::ids::ItemId;
use muo_server::item::{Item, ItemType};

fn all() -> Vec<(&'static ItemType, &'static [&'static str])> {
    let mut v = super::items::ALL.to_vec();
    v.push((&super::containers::CLOSED_BARREL, &[]));
    v
}

fn save(w: &World, id: ItemId) -> Vec<u8> {
    let ty = w.item(id).unwrap().ty;
    let mut out = SaveWriter::new(true);
    (ty.serialize)(w, id, &mut out);
    out.into_vec()
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
}

#[test]
fn every_class_constructs_and_round_trips() {
    for (t, _) in all() {
        let mut w = crate::test_world();
        let id = (t.construct.unwrap())(&mut w);
        assert_eq!(w.item(id).unwrap().ty.full_name, t.full_name);
        let bytes = save(&w, id);
        let mut w2 = crate::test_world();
        let id2 = w2.add_item(Item::new_for_load(
            muo_primitives::Serial::new(0x4000_0001),
            t,
        ));
        let mut r = SaveReader::new(&bytes);
        (t.deserialize)(&mut w2, id2, &mut r).unwrap_or_else(|e| panic!("{}: {e}", t.full_name));
        assert_eq!(r.remaining(), 0, "{} is read to its end", t.full_name);
        assert_eq!(save(&w2, id2), bytes, "{}", t.full_name);
    }
}

// ---- the hand-written classes of soulstone, statuette, boxes, anniv, gifts and scrolls ----

fn hand() -> Vec<&'static ItemType> {
    use super::*;
    [
        &soulstone::SOULSTONE,
        &soulstone::SOULSTONE_FRAGMENT,
        &soulstone::BLUE_SOULSTONE,
        &soulstone::RED_SOULSTONE,
        &statuette::MONSTER_STATUETTE,
        &statuette::HERITAGE_TOKEN,
        &statuette::GRIZZLED_MARE_STATUETTE,
        &boxes::RED_VELVET_GIFT_BOX,
        &boxes::VALENTINES_CARD_SOUTH,
        &boxes::VALENTINES_CARD_EAST,
        &boxes::COMMODITY_DEED_BOX,
        &anniv::DUPRES_SHIELD,
        &anniv::QUIVER_OF_INFINITY,
        &anniv::FERRET_FORM_TALISMAN,
        &anniv::REPTALON_FORM_TALISMAN,
        &gifts::TAPESTRY_OF_SOSARIA,
        &gifts::ROSE_OF_TRINSIC,
        &gifts::ROSE_OF_TRINSIC_PETAL,
        &gifts::HEARTH_OF_HOME_FIRE_DEED,
        &gifts::ZOOGI_FUNGUS,
        &gifts::POWDER_OF_TRANSLOCATION,
        &scrolls::POWER_SCROLL,
        &scrolls::STAT_CAP_SCROLL,
        &scrolls::SCROLL_OF_ALACRITY,
        &scrolls::SCROLL_OF_TRANSCENDENCE,
        &fish::AQUARIUM_FOOD,
        &fish::FISH_BONES,
        &fish::SHELL,
        &fish::TOY_BOAT,
        &fish::VACATION_WAFER,
        &fish::CRAFTYS_FISHING_HAT,
        &fish::WATERLOGGED_BOOTS,
    ]
    .into_iter()
    .chain(fish::FISH.iter().copied())
    .collect()
}

#[test]
fn hand_written_classes_construct_and_round_trip() {
    for t in hand() {
        let mut w = crate::test_world();
        let id = (t.construct.unwrap())(&mut w);
        assert_eq!(w.item(id).unwrap().ty.full_name, t.full_name);
        let bytes = save(&w, id);
        let mut w2 = crate::test_world();
        let id2 = w2.add_item(Item::new_for_load(
            muo_primitives::Serial::new(0x4000_0001),
            t,
        ));
        let mut r = SaveReader::new(&bytes);
        (t.deserialize)(&mut w2, id2, &mut r).unwrap_or_else(|e| panic!("{}: {e}", t.full_name));
        assert_eq!(r.remaining(), 0, "{} is read to its end", t.full_name);
        assert_eq!(save(&w2, id2), bytes, "{}", t.full_name);
    }
}

#[test]
fn soulstone_golden_tail() {
    use super::soulstone::*;
    let mut w = crate::test_world();
    let id = soulstone_new(&mut w, &SOULSTONE, Some("acct".into()), 0x2A93, 0x2A94);
    let bytes = save(&w, id);
    // version 4 (encoded), no last user, level 0 (an enum is an encoded int), active, inactive, account "acct",
    // skill 0, value 0.0
    let mut tail = vec![4u8, 0, 0];
    tail.extend_from_slice(&0x2A94i32.to_le_bytes());
    tail.extend_from_slice(&0x2A93i32.to_le_bytes());
    tail.extend_from_slice(&[1, 4, b'a', b'c', b'c', b't', 0]);
    tail.extend_from_slice(&0f64.to_le_bytes());
    assert!(bytes.ends_with(&tail), "{bytes:02x?}");
    // an empty stone shows its inactive graphic; storing a skill switches to the active one
    assert!(is_empty(&w, id));
    assert_eq!((SOULSTONE.item_id)(&w, id), 0x2A93);
}

#[test]
fn monster_statuette_type_and_hue() {
    use super::statuette::*;
    let mut w = crate::test_world();
    let id = monster_statuette_new(&mut w, kind::RED_DEATH);
    assert_eq!((MONSTER_STATUETTE.item_id)(&w, id), 0x2617);
    assert_eq!(w.item(id).unwrap().hue, 0x21);
    let bytes = save(&w, id);
    // version 0, type 29 (an enum is an encoded int), turned off, not a reward item
    assert!(bytes.ends_with(&[0, 0, 0, 0, 29, 0, 0]));
    set_type(&mut w, id, kind::HALLOWEEN_GHOUL);
    assert_eq!(w.item(id).unwrap().hue, 0xF4);
}

#[test]
fn scroll_names_follow_the_value() {
    use super::scrolls::*;
    let mut w = crate::test_world();
    let id = power_scroll_new(&mut w, muo_server::skills::SkillName::Magery, 110.0);
    assert_eq!(value(&w, id), 110.0);
    // a 105 scroll is not cursed, a 110 one is
    assert_eq!(
        w.item(id).unwrap().loot_type,
        muo_server::item::LootType::Cursed
    );
    let id = power_scroll_new(&mut w, muo_server::skills::SkillName::Magery, 105.0);
    assert_eq!(
        w.item(id).unwrap().loot_type,
        muo_server::item::LootType::Regular
    );
}

#[test]
fn dye_hook_receives_the_tub() {
    // sender.FailMessage: a tub whose class answers another message is what the refusal shows
    use muo_content::items::clothing::{tub_dyed_hue, tub_fail_message};
    let mut w = crate::test_world();
    let tub = muo_server::item::ops::new_item_with_id(&mut w, &muo_server::item::ITEM_BASE, 0xFAB);
    assert_eq!(tub_fail_message(&w, tub), 1042083);
    muo_content::items::equip_hooks::hooks_mut(&mut w).dye_tub = Some(|_, _| (0x21, 1042084));
    assert_eq!(
        (tub_dyed_hue(&w, tub), tub_fail_message(&w, tub)),
        (0x21, 1042084)
    );
}

#[test]
fn earring_box_set_holds_five_earrings() {
    let mut w = crate::test_world();
    let id = (super::boxes::EARRING_BOX_SET.construct.unwrap())(&mut w);
    assert_eq!(w.item(id).unwrap().items.len(), 5);
}

#[test]
fn a_fish_dies_after_five_minutes() {
    use super::fish::*;
    let mut w = crate::test_world();
    let id = (JELLYFISH.construct.unwrap())(&mut w);
    assert!(!is_dead(&w, id));
    assert_eq!(description(&w, id), 1073622);
    kill(&mut w, id);
    assert!(is_dead(&w, id));
    assert_eq!(description(&w, id), 1073623);
}
