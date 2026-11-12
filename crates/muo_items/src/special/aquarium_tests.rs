//! Tests of Items/Aquarium (Aquarium, FishBowl) and Items/Special/House Raffle.

use muo_io::{SaveReader, SaveWriter};
use muo_primitives::{DateTime, Rectangle2D};
use muo_server::World;
use muo_server::ids::ItemId;
use muo_server::item::{Item, ItemType};

use super::{aquarium, fish, raffle};

fn save(w: &World, id: ItemId) -> Vec<u8> {
    let ty = w.item(id).unwrap().ty;
    let mut out = SaveWriter::new(true);
    (ty.serialize)(w, id, &mut out);
    out.into_vec()
}

/// Loads `bytes` as a `t` into a fresh world.
fn load(t: &'static ItemType, bytes: &[u8]) -> (World, ItemId) {
    let mut w = World::new();
    let id = w.add_item(Item::new_for_load(
        muo_primitives::Serial::new(0x4000_0001),
        t,
    ));
    let mut r = SaveReader::new(bytes);
    (t.deserialize)(&mut w, id, &mut r).unwrap_or_else(|e| panic!("{}: {e}", t.full_name));
    assert_eq!(r.remaining(), 0, "{} is read to its end", t.full_name);
    (w, id)
}

fn tank(w: &mut World) -> ItemId {
    aquarium::aquarium_new(w, 0x3060)
}

#[test]
fn deeds_and_bowl_round_trip() {
    for t in [
        &aquarium::AQUARIUM_EAST_DEED,
        &aquarium::AQUARIUM_NORTH_DEED,
        &aquarium::FISH_BOWL,
    ] {
        let mut w = World::new();
        let id = (t.construct.unwrap())(&mut w);
        let bytes = save(&w, id);
        let (w2, id2) = load(t, &bytes);
        assert_eq!(save(&w2, id2), bytes, "{}", t.full_name);
    }
}

#[test]
fn a_new_aquarium_is_fed_and_has_room_for_thirty() {
    let mut w = World::new();
    let id = tank(&mut w);
    let d = aquarium::data(&w, id).unwrap();
    assert_eq!((d.food.state, d.water.state), (3, 4));
    assert!((1..=2).contains(&d.food.maintain));
    assert!(d.food.improve > d.food.maintain);
    assert!((1..=3).contains(&d.water.maintain));
    assert_eq!(aquarium::max_live_creatures(&w, id), 30);
    assert!(aquarium::optimal_state(&w, id));
    assert!(!aquarium::is_full(&w, id));
    assert_eq!(aquarium::food_number(&w, id), 1074240);
    assert_eq!(aquarium::water_number(&w, id), 1074246);
}

#[test]
fn the_aquarium_saves_version_four_with_its_timer_and_state() {
    let mut w = World::new();
    let id = tank(&mut w);
    let bytes = save(&w, id);
    let (w2, id2) = load(&aquarium::AQUARIUM, &bytes);
    let (a, b) = (
        aquarium::data(&w, id).unwrap(),
        aquarium::data(&w2, id2).unwrap(),
    );
    assert_eq!(
        (a.food.maintain, a.food.improve, a.water.maintain),
        (b.food.maintain, b.food.improve, b.water.maintain)
    );
    assert!(
        b.timer.is_some(),
        "the evaluation timer is rebuilt from the saved Next"
    );
}

#[test]
fn fish_are_added_and_half_of_the_requested_kills_happen() {
    let mut w = World::new();
    let id = tank(&mut w);
    for _ in 0..4 {
        let f = (fish::MINOC_BLUE_FISH.construct.unwrap())(&mut w);
        assert!(aquarium::add_fish(&mut w, id, None, Some(f)));
    }
    assert_eq!(aquarium::data(&w, id).unwrap().live_creatures, 4);
    // `for (i = 0; i < amount; i++) { ...; amount -= 1; }` kills half
    aquarium::kill_fish(&mut w, id, 4);
    assert_eq!(aquarium::dead_creatures(&w, id), 2);
    assert_eq!(aquarium::data(&w, id).unwrap().live_creatures, 2);
    assert_eq!(
        aquarium::data(&w, id).unwrap().events,
        vec![1074366, 1074366]
    );
}

#[test]
fn a_dead_fish_is_refused() {
    let mut w = World::new();
    let id = tank(&mut w);
    let f = (fish::SHRIMP.construct.unwrap())(&mut w);
    fish::kill(&mut w, f);
    assert!(!aquarium::add_fish(&mut w, id, None, Some(f)));
}

#[test]
fn only_the_listed_decorations_are_accepted() {
    let mut w = World::new();
    let id = tank(&mut w);
    let shell = (fish::SHELL.construct.unwrap())(&mut w);
    let wafer = (fish::VACATION_WAFER.construct.unwrap())(&mut w);
    assert!(aquarium::accepts(&w, shell));
    assert!(!aquarium::accepts(&w, wafer));
    assert!(aquarium::add_decoration(&mut w, id, None, Some(shell)));
    assert!(!aquarium::add_decoration(&mut w, id, None, Some(wafer)));
}

#[test]
fn a_day_of_neglect_makes_the_food_worse() {
    let mut w = World::new();
    let id = tank(&mut w);
    // the first evaluation only flips to the evaluate day, the second runs it: nothing was added, so food drops
    aquarium::evaluate(&mut w, id);
    aquarium::evaluate(&mut w, id);
    let d = aquarium::data(&w, id).unwrap();
    assert_eq!(d.food.state, 2);
    assert_eq!(d.water.state, 3);
    assert!(d.events.contains(&1074368));
    assert!(d.events.contains(&1074370));
}

#[test]
fn the_stone_saves_its_settings_and_reloads() {
    let mut w = World::new();
    let id = raffle::stone_new(&mut w);
    raffle::set_ticket_price(&mut w, id, 1234);
    raffle::set_plot_bounds(&mut w, id, Rectangle2D::new(100, 200, 10, 12));
    let bytes = save(&w, id);
    let (w2, id2) = load(&raffle::HOUSE_RAFFLE_STONE, &bytes);
    let d = raffle::stone(&w2, id2).unwrap();
    assert_eq!(d.ticket_price, 1234);
    assert_eq!(d.plot_bounds, Rectangle2D::new(100, 200, 10, 12));
    assert_eq!(d.duration, raffle::DEFAULT_DURATION);
    assert_eq!(save(&w2, id2), bytes);
}

#[test]
fn a_version_three_stone_reads_and_drops_entries_of_deleted_characters() {
    let mut w = World::new();
    let id = raffle::stone_new(&mut w);
    let mut bytes = save(&w, id);
    // the Item part, then the int version 4 -> 3 and the old layout (a plain int entry count)
    let base_len = {
        let mut out = SaveWriter::new(true);
        (muo_server::item::ITEM_BASE.serialize)(&w, id, &mut out);
        out.into_vec().len()
    };
    bytes[base_len..base_len + 4].copy_from_slice(&3i32.to_le_bytes());
    // the last byte of v4 is the encoded entry count 0; the legacy count is a 4 byte int
    bytes.pop();
    bytes.extend_from_slice(&1i32.to_le_bytes());
    // one entry: encoded version 0, mobile -1 (a deleted character), IPv4 1.2.3.4, a date
    bytes.push(0);
    bytes.extend_from_slice(&(-1i32).to_le_bytes());
    bytes.extend_from_slice(&[4, 1, 2, 3, 4]);
    bytes.extend_from_slice(&DateTime::MIN.ticks().to_le_bytes());
    let (w2, id2) = load(&raffle::HOUSE_RAFFLE_STONE, &bytes);
    assert!(raffle::stone(&w2, id2).unwrap().entries.is_empty());
}

#[test]
fn the_deed_round_trips_and_a_stoneless_deed_is_expired() {
    let mut w = World::new();
    let s = raffle::stone_new(&mut w);
    let deed = raffle::deed_new_with(&mut w, Some(s), None);
    assert!(!raffle::deed_is_expired(&w, deed));
    let bytes = save(&w, deed);
    // the stone is another item the second world does not have, so it reads as none
    let (w2, d2) = load(&raffle::HOUSE_RAFFLE_DEED, &bytes);
    assert_eq!(
        raffle::deed(&w2, d2).unwrap().plot_location,
        raffle::deed(&w, deed).unwrap().plot_location
    );
    let bare = raffle::deed_new_with(&mut w, None, None);
    assert!(raffle::deed_is_expired(&w, bare));
}

#[test]
fn formats_match_the_expected_strings() {
    assert_eq!(raffle::format_price(0), "FREE");
    assert_eq!(raffle::format_price(5000), "5000 gold");
    assert_eq!(
        raffle::date_time_string(DateTime::from_ymd_hms(2026, 3, 9, 14, 5, 7).unwrap()),
        "03/09/2026 14:05:07"
    );
}
