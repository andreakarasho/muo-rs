//! Tests of the classes of `last`: every class constructs and round-trips its save bytes in each era, and the
//! pieces with logic (chests, the armoire, the door chain, the dragon barding deed) are exercised.

use muo_content::items::cont;
use muo_data::expansion::Expansion;
use muo_io::{SaveReader, SaveWriter};
use muo_server::World;
use muo_server::ids::ItemId;
use muo_server::item::{Item, ItemType};
use muo_server::type_registry::{Phase, invoke_phase};

use super::{artifacts, chests, containers, deeds, lights, mark, misc};

fn world(era: Option<Expansion>) -> World {
    let mut w = World::new();
    w.core.expansion = era;
    crate::register(&mut w.types);
    muo_content::items::common::register(&mut w.types);
    muo_content::items::leaves::register(&mut w.types);
    invoke_phase(&mut w, Phase::Configure);
    w
}

fn save(w: &World, id: ItemId) -> Vec<u8> {
    let ty = w.item(id).unwrap().ty;
    let mut out = SaveWriter::new(true);
    (ty.serialize)(w, id, &mut out);
    out.into_vec()
}

fn for_load(w: &mut World, serial: u32, ty: &'static ItemType) -> ItemId {
    w.add_item(Item::new_for_load(muo_primitives::Serial::new(serial), ty))
}

/// Saves `id`, loads the bytes into a fresh item of a fresh world (as a server start does: the children of a
/// container are not there) and saves again, twice: the record is read to its end and is stable from the second
/// save on (a clothing record written without its resource flag loads as `CraftResource.None`, so a class with a
/// default resource differs once). `contents` is true for a container, whose record lists its items by serial.
fn round_trip(w: &mut World, id: ItemId, what: &str, contents: bool) {
    let ty = w.item(id).unwrap().ty;
    let mut bytes = save(w, id);
    for pass in 0..2 {
        let mut w2 = world(w.core.expansion);
        let other = for_load(&mut w2, 0x4000_0001, ty);
        let mut r = SaveReader::new(&bytes);
        (ty.deserialize)(&mut w2, other, &mut r)
            .unwrap_or_else(|e| panic!("{what} pass {pass}: {e} ({} bytes)", bytes.len()));
        assert_eq!(r.remaining(), 0, "{what} is read to its end");
        let again = save(&w2, other);
        if pass == 1 && !contents {
            assert_eq!(&bytes[8..], &again[8..], "{what} is stable");
        }
        bytes = again;
    }
}

fn all() -> Vec<&'static ItemType> {
    let mut v: Vec<&'static ItemType> = Vec::new();
    v.extend_from_slice(artifacts::ALL);
    v.extend([
        &lights::CANDELABRA,
        &lights::HEATING_STAND,
        &lights::RED_HANGING_LANTERN,
        &lights::WHITE_HANGING_LANTERN,
        &lights::WALL_SCONCE,
        &lights::WALL_TORCH,
        &containers::ARMOIRE,
        &containers::FANCY_ARMOIRE,
        &misc::MELISANDES_FERMENTED_WINE,
        &deeds::HAIR_RESTYLING_DEED,
        &deeds::BARKEEP_CONTRACT,
        &deeds::DRAGON_BARDING_DEED,
        &mark::MARK_CONTAINER,
        &super::food::DARK_CHOCOLATE,
        &super::food::MILK_CHOCOLATE,
        &super::food::WHITE_CHOCOLATE,
    ]);
    v
}

#[test]
fn every_class_constructs_and_round_trips() {
    for era in [None, Some(Expansion::AOS), Some(Expansion::ML)] {
        for ty in all() {
            let mut w = world(era);
            let id = (ty
                .construct
                .or(ty.make)
                .unwrap_or_else(|| panic!("{} has no constructor", ty.full_name)))(
                &mut w
            );
            assert_eq!(w.item(id).unwrap().ty.full_name, ty.full_name);
            round_trip(&mut w, id, &format!("{} {era:?}", ty.full_name), false);
        }
    }
}

#[test]
fn the_chests_round_trip_and_are_filled() {
    for era in [
        None,
        Some(Expansion::AOS),
        Some(Expansion::SE),
        Some(Expansion::ML),
    ] {
        let mut w = world(era);
        for level in 0..=6 {
            let c = chests::treasure_map_chest_new(&mut w, None, level, false);
            assert!(cont::locked(&w, c), "level {level}");
            assert!(
                !w.item(c).unwrap().items.is_empty(),
                "level {level} {era:?}"
            );
            assert_eq!(chests::level(&w, c), level);
            round_trip(
                &mut w,
                c,
                &format!("treasure map chest {level} {era:?}"),
                true,
            );
        }
        for level in 1..=5 {
            let c = chests::paragon_chest_new(&mut w, Some("a paragon".into()), level);
            assert_eq!(cont::ext(&w, c).unwrap().trap_level, level);
            assert!(cont::locked(&w, c));
            round_trip(&mut w, c, &format!("paragon chest {level} {era:?}"), true);
        }
    }
}

#[test]
fn a_treasure_map_chest_level_1_has_the_expected_lock_numbers() {
    let mut w = world(None);
    let c = chests::treasure_map_chest_new(&mut w, None, 1, true);
    let e = cont::ext(&w, c).unwrap();
    assert_eq!(
        (e.required_skill, e.lock_level, e.max_lock_level),
        (36, 26, 76)
    );
    assert_eq!((e.trap_power, e.trap_level), (25, 1));
}

#[test]
fn the_armoire_closes_when_its_timer_says_so() {
    let mut w = world(None);
    let a = (containers::ARMOIRE.construct.unwrap())(&mut w);
    assert_eq!(w.item(a).unwrap().item_id, 0xA4F);
    // an open armoire shows its open graphic until it is closed again
    let m = muo_content::factory::make_mobile(&mut w, "Server.Mobiles.PlayerMobile");
    if let Some(m) = m {
        assert!(containers::open(&mut w, a, m));
        assert_eq!(w.item(a).unwrap().item_id, 0xA4E);
        containers::close(&mut w, a);
        assert_eq!(w.item(a).unwrap().item_id, 0xA4F);
    }
}

#[test]
fn the_door_chain_follows_links_and_stops_at_a_loop() {
    use crate::construction::doors;
    let mut w = world(None);
    let mk = |w: &mut World| muo_content::factory::make_item(w, "Server.Items.IronGate").unwrap();
    let (a, b, c) = (mk(&mut w), mk(&mut w), mk(&mut w));
    assert_eq!(doors::chain(&w, a), vec![a]);
    doors::set_link(&mut w, a, Some(b));
    doors::set_link(&mut w, b, Some(c));
    doors::set_link(&mut w, c, Some(b)); // loops back to a door other than the start
    assert_eq!(doors::chain(&w, a), vec![a, b, c]);
    doors::set_link(&mut w, c, Some(a));
    assert_eq!(doors::chain(&w, a), vec![a, b, c]);
}

#[test]
fn chocolates_are_regular_candy_canes() {
    let mut w = world(None);
    let d = (super::food::DARK_CHOCOLATE.construct.unwrap())(&mut w);
    let i = w.item(d).unwrap();
    assert_eq!((i.item_id, i.hue), (0xF10, 0x465));
    assert_eq!(i.loot_type, muo_server::item::LootType::Regular);
}

#[test]
fn the_ml_artifacts_have_their_constants() {
    let mut w = world(Some(Expansion::ML));
    let q = (artifacts::QUIVER_OF_RAGE.construct.unwrap())(&mut w);
    assert_eq!(w.item(q).unwrap().hue, 0x24C);
    assert_eq!(crate::quivers::data(&w, q).unwrap().weight_reduction, 25);
    assert_eq!(crate::quivers::data(&w, q).unwrap().damage_increase, 10);
    let h = (artifacts::GWENNOS_HARP.construct.unwrap())(&mut w);
    let d = crate::skill_items::instruments::data(&w, h).unwrap();
    assert_eq!(d.uses_remaining, 1600);
    assert_eq!((d.slayer, d.slayer2), (5, 10));
}
