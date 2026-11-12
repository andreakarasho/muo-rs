//! Tests of the multis: the house classes construct and round-trip their save bytes, the placement tables find
//! their entries, the door table picks the right class.

use muo_io::{SaveReader, SaveWriter};
use muo_primitives::Serial;
use muo_server::World;
use muo_server::ids::ItemId;
use muo_server::item::{Item, ItemType};
use muo_server::mobile::MOBILE_BASE;

use crate::classes::*;
use crate::house::{self, access};

type Maker = (
    &'static ItemType,
    fn(&mut World, muo_server::ids::MobileId) -> ItemId,
    i32,
    i32,
);

fn save(w: &World, id: ItemId) -> Vec<u8> {
    let ty = w.item(id).unwrap().ty;
    let mut out = SaveWriter::new(true);
    (ty.serialize)(w, id, &mut out);
    out.into_vec()
}

fn load(w: &mut World, ty: &'static ItemType, bytes: &[u8], serial: u32) -> ItemId {
    let id = w.add_item(Item::new_for_load(Serial::new(serial), ty));
    let mut r = SaveReader::new(bytes);
    (ty.deserialize)(w, id, &mut r).unwrap_or_else(|e| panic!("{}: {e}", ty.full_name));
    assert_eq!(r.remaining(), 0, "{}", ty.full_name);
    id
}

#[test]
fn every_house_class_round_trips_and_ends_with_its_limits() {
    let makers: [Maker; 13] = [
        (
            &SMALL_OLD_HOUSE,
            |w, o| new_small_old_house(w, o, 0x64),
            425,
            3,
        ),
        (&GUILD_HOUSE, new_guild_house, 1100, 8),
        (
            &TWO_STORY_HOUSE,
            |w, o| new_two_story_house(w, o, 0x76),
            1370,
            10,
        ),
        (&TOWER, new_tower, 2119, 15),
        (&KEEP, new_keep, 2625, 18),
        (&CASTLE, new_castle, 4076, 28),
        (&LARGE_PATIO_HOUSE, new_large_patio_house, 1100, 8),
        (&LARGE_MARBLE_HOUSE, new_large_marble_house, 1370, 10),
        (&SMALL_TOWER, new_small_tower, 580, 4),
        (&LOG_CABIN, new_log_cabin, 1100, 8),
        (&SAND_STONE_PATIO, new_sand_stone_patio, 850, 6),
        (&TWO_STORY_VILLA, new_two_story_villa, 1100, 8),
        (&SMALL_SHOP, |w, o| new_small_shop(w, o, 0xA0), 425, 3),
    ];
    for (ty, make, lockdowns, secures) in makers {
        let mut w = World::new();
        let owner = w.new_mobile(&MOBILE_BASE);
        let id = make(&mut w, owner);
        // a house without a map and at (0,0,0) deletes itself when loaded
        muo_server::item::ops::set_location(
            &mut w,
            id,
            muo_primitives::Point3D::new(1000, 1000, 0),
        );
        assert!(house::is_house(&w, id));
        let h = house::d(&w, id);
        assert_eq!(
            (h.max_lock_downs, h.max_secures),
            (lockdowns, secures),
            "{}",
            ty.full_name
        );
        assert_eq!(h.owner, Some(owner));
        assert!(h.sign.is_some(), "{} has a sign", ty.full_name);
        assert_eq!(access::get_houses(&w, owner), vec![id]);

        let bytes = save(&w, id);
        // ... MaxLockDowns, MaxSecures, then the class' own version int 0
        let tail = &bytes[bytes.len() - 12..];
        assert_eq!(tail[0..4], lockdowns.to_le_bytes(), "{}", ty.full_name);
        assert_eq!(tail[4..8], secures.to_le_bytes(), "{}", ty.full_name);
        assert_eq!(tail[8..12], [0, 0, 0, 0], "{}", ty.full_name);

        let loaded = load(&mut w, ty, &bytes, 0x4000_0100);
        assert_eq!(save(&w, loaded), bytes, "{}", ty.full_name);
        let lh = house::d(&w, loaded);
        assert_eq!(lh.owner, Some(owner));
        assert_eq!((lh.max_lock_downs, lh.max_secures), (lockdowns, secures));
    }
}

#[test]
fn the_house_record_has_version_15_after_the_multi_part() {
    let mut w = World::new();
    w.core.expansion = Some(muo_data::expansion::Expansion::UOR);
    let owner = w.new_mobile(&MOBILE_BASE);
    let id = new_small_old_house(&mut w, owner, 0x64);
    let bytes = save(&w, id);
    let mut base = SaveWriter::new(true);
    (muo_server::item::multi::BASE_MULTI.serialize)(&w, id, &mut base);
    let base = base.into_vec();
    assert!(bytes.starts_with(&base));
    assert_eq!(bytes[base.len()..base.len() + 4], 15_i32.to_le_bytes());
    // no dynamic decay before ML: -1
    assert_eq!(
        bytes[base.len() + 4..base.len() + 8],
        (-1_i32).to_le_bytes()
    );
}

#[test]
fn owners_lists_follow_the_owner() {
    let mut w = World::new();
    let a = w.new_mobile(&MOBILE_BASE);
    let b = w.new_mobile(&MOBILE_BASE);
    let id = new_small_tower(&mut w, a);
    assert!(access::has_house(&w, a));
    access::set_owner(&mut w, id, Some(b));
    assert!(!access::has_house(&w, a));
    assert!(access::has_house(&w, b));
    assert!(access::is_owner(&w, id, b));
    assert!(!access::is_owner(&w, id, a));
}

#[test]
fn placement_entries_are_found_by_class_and_multi_id() {
    let w = World::new();
    let e = crate::placement_entry::find(&w, "SmallOldHouse", 0x66).unwrap();
    assert_eq!(
        (e.multi_id, e.cost, e.storage, e.new_lockdowns),
        (0x66, 37000, 425, 244)
    );
    // a class with one entry gets it whatever the ItemID
    let k = crate::placement_entry::find(&w, "Keep", 1).unwrap();
    assert_eq!(k.multi_id, 0x7C);
    assert_eq!(
        crate::placement_entry::find(&w, "HouseFoundation", 0x13EC)
            .unwrap()
            .description,
        1060241
    );
    assert_eq!(crate::house_entries::CLASSIC_HOUSES.len(), 20);
    assert_eq!(
        crate::house_entries::THREE_STORY_FOUNDATIONS.len()
            + crate::house_entries::TWO_STORY_FOUNDATIONS.len(),
        102
    );
}

#[test]
fn deeds_round_trip_with_their_fields() {
    let mut w = World::new();
    let ty = &crate::deeds::TOWER_DEED;
    let id = (ty.construct.unwrap())(&mut w);
    assert_eq!(crate::deeds::multi_id(&w, id), 0x7A);
    assert_eq!(
        crate::deeds::offset(&w, id),
        muo_primitives::Point3D::new(0, 7, 0)
    );
    let bytes = save(&w, id);
    // ... version 0, offset (0, 7, 0), multi id 0x7A
    let mut tail = vec![0, 0, 0, 0];
    for v in [0_i32, 7, 0, 0x7A] {
        tail.extend_from_slice(&v.to_le_bytes());
    }
    tail.extend_from_slice(&0_i32.to_le_bytes()); // the class' own version
    assert!(bytes.ends_with(&tail[4..]) || bytes.ends_with(&tail));
    let loaded = load(&mut w, ty, &bytes, 0x4000_0101);
    assert_eq!(save(&w, loaded), bytes);
}

#[test]
fn door_graphics_pick_a_class() {
    use crate::house::doors::door_for_item_id;
    assert!(door_for_item_id(0x675).is_some());
    assert!(door_for_item_id(0x2D46).is_some());
    assert!(door_for_item_id(0x1).is_none());
}

#[test]
fn add_fixtures_has_no_time_of_legends_doors() {
    use crate::foundation::fixture_door_for_item_id as f;
    use crate::house::doors::door_for_item_id;
    for id in [0x9AD7, 0x9AE6, 0x9B3C, 0x9B4B] {
        assert!(door_for_item_id(id).is_some());
        assert!(f(id).is_none());
    }
    assert!(f(0x675).is_some());
}

#[test]
fn decay_levels_of_a_new_house_are_like_new() {
    use crate::house::{DecayLevel, decay};
    let mut w = World::new();
    let owner = w.new_mobile(&MOBILE_BASE);
    let id = new_log_cabin(&mut w, owner);
    assert_eq!(decay::old_decay_level(&w, id), DecayLevel::LikeNew);
    w.now += muo_primitives::TimeSpan::from_days(4);
    assert_eq!(decay::old_decay_level(&w, id), DecayLevel::Greatly);
    w.now += muo_primitives::TimeSpan::from_days(2);
    assert_eq!(decay::old_decay_level(&w, id), DecayLevel::Collapsed);
}

#[test]
fn camps_and_the_barrel_round_trip() {
    use crate::camps::*;
    for ty in [
        &BRIGAND_CAMP,
        &LIZARDMEN_CAMP,
        &RAT_CAMP,
        &ORC_CAMP,
        &HEALER_CAMP,
        &BANKER_CAMP,
        &MAGE_CAMP,
    ] {
        let mut w = World::new();
        let id = (ty.construct.unwrap())(&mut w);
        let bytes = save(&w, id);
        // the camp's version int 0 closes the record (the spawn camps then name their prisoner)
        let prisoner = ty.full_name.contains("Brigand")
            || ty.full_name.contains("Lizardmen")
            || ty.full_name.contains("Rat")
            || ty.full_name.contains("Orc");
        let tail = if prisoner { 8 } else { 4 };
        assert_eq!(
            bytes[bytes.len() - tail..bytes.len() - tail + 4],
            [0, 0, 0, 0],
            "{}",
            ty.full_name
        );
        let loaded = load(&mut w, ty, &bytes, 0x4000_0200);
        assert_eq!(save(&w, loaded), bytes, "{}", ty.full_name);
    }
    // The barrel's record is LockableContainer's part and an int 0. (A fresh item from `new_for_load` has no `ContExt`
    // yet, which `LOCKABLE_CONTAINER.deserialize` needs to keep what it reads, so only the record's end is checked.)
    let mut w = World::new();
    let id = (LOCKABLE_BARREL.construct.unwrap())(&mut w);
    let bytes = save(&w, id);
    assert_eq!(bytes[bytes.len() - 4..], [0, 0, 0, 0]);
}

#[test]
fn contest_houses_round_trip() {
    use crate::contest::*;
    let makers: [Maker; 4] = [
        (&TRINSIC_KEEP, new_trinsic_keep, 2113, 18),
        (&GOTHIC_ROSE_CASTLE, new_gothic_rose_castle, 3281, 28),
        (&FEUDAL_CASTLE, new_feudal_castle, 3281, 28),
        (&SANDALWOOD_KEEP, new_sandalwood_keep, 2113, 18),
    ];
    for (ty, make, lockdowns, secures) in makers {
        let mut w = World::new();
        let owner = w.new_mobile(&MOBILE_BASE);
        let id = make(&mut w, owner);
        muo_server::item::ops::set_location(
            &mut w,
            id,
            muo_primitives::Point3D::new(1000, 1000, 0),
        );
        let h = house::d(&w, id);
        assert_eq!((h.max_lock_downs, h.max_secures), (lockdowns, secures));
        let bytes = save(&w, id);
        let loaded = load(&mut w, ty, &bytes, 0x4000_0300);
        assert_eq!(save(&w, loaded), bytes, "{}", ty.full_name);
    }
}

// ---- Houses/HouseDecayPurityTests ----

#[test]
fn reading_decay_level_does_not_stamp_last_refreshed() {
    use muo_primitives::{Point3D, TimeSpan};
    use muo_server::map::MapId;

    let mut w = muo_server::map::test_support::world();
    house::statics(&mut w).decay_enabled = false; // every house is Ageless, the branch that used to restamp on read

    let owner = w.new_mobile(&MOBILE_BASE);
    muo_server::mobile::part2::stats::default_mobile_init(&mut w, owner);
    muo_server::mobile::part1::lifecycle::move_to_world(
        &mut w,
        owner,
        Point3D::new(1400, 1400, 0),
        MapId::FELUCCA,
    );

    let h = new_small_old_house(&mut w, owner, 0x64);
    muo_server::item::ops::move_to_world_map(
        &mut w,
        h,
        Point3D::new(1400, 1400, 0),
        Some(MapId::FELUCCA),
    );

    let placed = w.now - TimeSpan::from_days(2);
    house::data_mut(&mut w, h).unwrap().last_refreshed = placed;

    assert!(!house::decay::can_decay(&w, h));
    assert_eq!(house::decay::decay_level(&w, h), house::DecayLevel::Ageless);
    assert_eq!(house::data(&w, h).unwrap().last_refreshed, placed);
}

#[test]
fn treasure_region_covers_five_tiles_around_the_spot_on_both_facets() {
    let mut w = muo_server::map::test_support::world();
    let dir = std::env::temp_dir().join(format!("muo_treasure_cfg_{}", std::process::id()));
    std::fs::create_dir_all(dir.join("Data")).unwrap();
    std::fs::write(dir.join("Data/treasure.cfg"), "1000 1000\nbad line\n").unwrap();
    w.core.base_directory = dir.clone();
    house::decay::initialize_treasure_regions(&mut w);
    for map in [
        muo_server::map::MapId::FELUCCA,
        muo_server::map::MapId::TRAMMEL,
    ] {
        let near =
            muo_server::region::find(&w, muo_primitives::Point3D::new(1005, 995, 0), Some(map));
        assert!(muo_server::region::is_part_of_type_name(
            &w,
            near,
            "TreasureRegion"
        ));
        let far =
            muo_server::region::find(&w, muo_primitives::Point3D::new(1006, 1000, 0), Some(map));
        assert!(!muo_server::region::is_part_of_type_name(
            &w,
            far,
            "TreasureRegion"
        ));
    }
    let _ = std::fs::remove_dir_all(dir);
}
