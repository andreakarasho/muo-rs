//! Tests of the Treasures of Tokuno, CannedEvil, Doom, Khaldun, Plants and Veteran Rewards engines: constructors,
//! save round trips, golden bytes and the pure tables.

use muo_io::{GenericWriter, SaveReader, SaveWriter};
use muo_primitives::{Point3D, Rectangle2D};
use muo_server::World;
use muo_server::ids::ItemId;
use muo_server::item::{Item, ItemType};

fn save(w: &World, id: ItemId) -> Vec<u8> {
    let ty = w.item(id).unwrap().ty;
    let mut out = SaveWriter::new(true);
    (ty.serialize)(w, id, &mut out);
    out.into_vec()
}

fn load(w: &mut World, ty: &'static ItemType, bytes: &[u8]) -> ItemId {
    let serial = w.next_item_serial();
    let id = w.add_item(Item::new_for_load(serial, ty));
    let mut r = SaveReader::new(bytes);
    (ty.deserialize)(w, id, &mut r).expect("loads");
    id
}

/// The bytes of `Item`'s own part, which the leaf bytes follow.
fn base_bytes(w: &World, id: ItemId) -> Vec<u8> {
    let mut out = SaveWriter::new(true);
    (muo_server::item::ITEM_BASE.serialize)(w, id, &mut out);
    out.into_vec()
}

#[test]
fn tokuno_pigments_round_trip_and_golden() {
    use crate::tokuno::pigments::{self, PIGMENTS_OF_TOKUNO};
    let mut w = World::new();
    let id = pigments::pigments_new(&mut w, 3, None); // InvulnerabilityBlue, 50 uses
    assert_eq!(pigments::uses_remaining(&w, id), 50);
    assert_eq!(w.item(id).unwrap().hue, 0x4F2);
    let bytes = save(&w, id);
    let mut want = base_bytes(&w, id);
    // BasePigments: version 0 (int), UsesRemaining (encoded 50), then Type: version 0 (int), enum (encoded 3)
    want.extend_from_slice(&[0, 0, 0, 0, 50]);
    want.extend_from_slice(&[0, 0, 0, 0, 3]);
    assert_eq!(bytes, want);
    let id2 = load(&mut w, &PIGMENTS_OF_TOKUNO, &bytes);
    assert_eq!(pigments::pigment_type(&w, id2), 3);
    assert_eq!(pigments::uses_remaining(&w, id2), 50);
    // The label is not saved: a loaded pigment has none.
    assert_eq!(pigments::data(&w, id2).unwrap().label, 0);
    // None and FadedCoal+ start with 10 uses
    let p = pigments::pigments_new(&mut w, 0, None);
    assert_eq!(pigments::uses_remaining(&w, p), 10);
    let p = pigments::pigments_new(&mut w, 11, None);
    assert_eq!(pigments::uses_remaining(&w, p), 10);
}

#[test]
fn tokuno_urn_names_are_saved() {
    use crate::tokuno::artifacts::{self, ANCIENT_URN};
    let mut w = World::new();
    let id = artifacts::ancient_urn_new(&mut w, Some("Akira".to_owned()));
    let bytes = save(&w, id);
    let id2 = load(&mut w, &ANCIENT_URN, &bytes);
    assert_eq!(save(&w, id2), bytes);
    assert_eq!(artifacts::NAMES.len(), 35);
}

#[test]
fn tokuno_lists_have_the_expected_sizes() {
    use crate::tokuno::lists;
    assert_eq!(lists::LESSER_ARTIFACTS_ONE.len(), 20);
    assert_eq!(lists::LESSER_ARTIFACTS_TWO.len(), 25);
    assert_eq!(lists::LESSER_ARTIFACTS_THREE.len(), 26);
    assert_eq!(lists::LESSER_ARTIFACTS_TOTAL.len(), 23);
    assert_eq!(lists::STEALABLE.len(), 81);
    assert!(crate::tokuno::gumps::greater_artifacts(crate::tokuno::Era::ToTThree).len() == 9);
}

#[test]
fn champion_info_table() {
    use crate::canned_evil::info::{self, ChampionSpawnType};
    let w = World::new();
    assert_eq!(info::TABLE.len(), 9);
    assert_eq!(info::get_info(5).ty, ChampionSpawnType::UnholyTerror);
    assert_eq!(info::get_info(99).ty, ChampionSpawnType::Abyss);
    // level 1 of UnholyTerror depends on Core.AOS
    assert!(info::spawn_types(&w, 5, 0).len() == 4 || info::spawn_types(&w, 5, 0).len() == 5);
    assert_eq!(info::get_info(0).champion, "Server.Mobiles.Semidar");
}

#[test]
fn champion_spawn_round_trips() {
    use crate::canned_evil::spawn::{self, CHAMPION_SPAWN};
    let mut w = World::new();
    let id = spawn::new_spawn(&mut w, &CHAMPION_SPAWN);
    spawn::set_spawn_area(&mut w, id, Rectangle2D::new(10, 20, 30, 40));
    spawn::data_mut(&mut w, id).unwrap().kills = 7;
    spawn::set_type(&mut w, id, 4);
    let bytes = save(&w, id);
    let id2 = load(&mut w, &CHAMPION_SPAWN, &bytes);
    let d = spawn::data(&w, id2).unwrap();
    assert_eq!(
        (d.kills, d.ty, d.spawn_area),
        (7, 4, Rectangle2D::new(10, 20, 30, 40))
    );
    assert!(d.platform.is_some() && d.altar.is_some() && d.idol.is_some());
    assert_eq!(spawn::max_kills(&w, id2), 256);
    assert_eq!(spawn::get_sub_level(&w, id2), 0);
    // the layout of version 12 is what the reader reads back
    assert_eq!(save(&w, id2), bytes);
}

#[test]
fn champion_skull_golden() {
    use crate::canned_evil::info::ChampionSkullType;
    use crate::canned_evil::items;
    let mut w = World::new();
    let id = items::skull_new(&mut w, ChampionSkullType::Venom);
    assert_eq!(w.item(id).unwrap().hue, 0x172);
    let mut want = base_bytes(&w, id);
    want.extend_from_slice(&[2, 0, 0, 0, 2]); // version 2, Venom
    assert_eq!(save(&w, id), want);
}

#[test]
fn champion_titles_award_and_decay() {
    use crate::canned_evil::info::ChampionSpawnType;
    use crate::canned_evil::titles;
    use muo_content::player_mobile::tables;
    let mut w = World::new();
    let pm = w.new_mobile(&muo_server::mobile::MOBILE_BASE);
    titles::award(&mut w, pm, ChampionSpawnType::Abyss, 90);
    titles::award(&mut w, pm, ChampionSpawnType::Abyss, 30);
    assert_eq!(
        tables::champion_titles_mut(&mut w, pm).titles[0]
            .unwrap()
            .value,
        120
    );
    titles::award_harrower_title(&mut w, pm);
    assert_eq!(tables::champion_titles_mut(&mut w, pm).harrower, 1);
}

#[test]
fn champion_titles_glade_and_corrupt_use_the_save_field_slots() {
    use crate::canned_evil::info::ChampionSpawnType;
    use crate::canned_evil::titles;
    use muo_content::player_mobile::tables;
    let mut w = World::new();
    let pm = w.new_mobile(&muo_server::mobile::MOBILE_BASE);
    titles::award(&mut w, pm, ChampionSpawnType::Corrupt, 5); // field 8 of the context: slot 7
    titles::award(&mut w, pm, ChampionSpawnType::Glade, 7); // field 9: slot 8
    let c = tables::champion_titles_mut(&mut w, pm);
    assert_eq!(
        (c.titles[7].unwrap().value, c.titles[8].unwrap().value),
        (5, 7)
    );
}

#[test]
fn rewards_year_labels_and_levels() {
    use crate::rewards;
    // A bronze RewardCloak is from the first year: label 1076216 + 1
    assert_eq!(
        rewards::get_reward_year_label("Server.Items.RewardCloak", Some(&[0x972, 1041286])),
        1076217
    );
    // 10 years: 1076216 + (10 - 9 + 4240); 12: 1076216 + (12 - 11 + 37585)
    assert_eq!(
        rewards::get_reward_year_label("Server.Mobiles.EtherealCuSidhe", None),
        1076216 + 1 + 4240
    );
    assert_eq!(
        rewards::get_reward_year_label("Server.Mobiles.EtherealHiryu", None),
        1076216 + 1 + 37585
    );
    // an item without an entry is year 0
    assert_eq!(rewards::get_reward_year("Server.Items.Gold", None), 0);
    assert_eq!(rewards::LISTS.len(), 12);
}

#[test]
fn plant_tables() {
    use crate::plants::{self, PlantHueInfo, PlantTypeInfo, hue};
    assert_eq!(PlantHueInfo::get(hue::PLAIN).gump_hue, 0x835);
    assert_eq!(PlantHueInfo::get(12345).hue, 0); // unknown: Plain
    assert!(PlantHueInfo::get(hue::BRIGHT_RED).is_bright());
    assert!(!PlantHueInfo::get(hue::RED).is_bright());
    assert_eq!(
        PlantTypeInfo::get(plants::plant_type::COCOA_TREE).item_id,
        0xC9E
    );
    assert_eq!(PlantTypeInfo::get(1000).item_id, 0xC83);
    assert_eq!(PlantTypeInfo::get(0).name(), 1020000 + 0xC83);
    let mut w = muo_server::World::new();
    // two different crossable primaries make the mix, the same one makes it bright
    assert!(plants::is_hue_crossable(hue::RED));
    let mixed = plants::cross_hue(&mut w, hue::RED, hue::RED);
    assert!(mixed == hue::RED | hue::BRIGHT || mixed == hue::BLACK || mixed == hue::WHITE);
}

#[test]
fn plant_seed_round_trips() {
    use crate::plants::{hue, seed};
    let mut w = World::new();
    let id = seed::seed_new(&mut w, 8, hue::BLUE, true).unwrap();
    assert_eq!(w.item(id).unwrap().hue, 0x53D);
    let bytes = save(&w, id);
    let mut want = base_bytes(&w, id);
    want.extend_from_slice(&[3, 0, 0, 0]);
    want.push(8); // PlantType.Fern (encoded)
    // PlantHue.Blue = 0x4 | 0x4000000 | 0x2000000 (encoded int: 7-bit groups)
    let mut tmp = SaveWriter::new(true);
    tmp.write_encoded_int(hue::BLUE);
    want.extend_from_slice(&tmp.into_vec());
    want.push(1); // ShowType
    assert_eq!(bytes, want);
    let id2 = load(&mut w, &seed::SEED, &bytes);
    assert_eq!(seed::data(&w, id2).unwrap().plant_type, 8);
}

#[test]
fn khaldun_books_have_their_pages() {
    use crate::khaldun::books;
    assert_eq!(books::GRIMMOCH_JOURNAL1_CONTENT.pages.len(), 5);
    let mut w = World::new();
    let id = books::grimmoch_journal1_new(&mut w);
    let g = w.item(id).unwrap().item_id;
    assert!(g == 0xFF1 || g == 0xFF2);
    let bytes = save(&w, id);
    let id2 = load(&mut w, &books::GRIMMOCH_JOURNAL1, &bytes);
    assert_eq!(save(&w, id2), bytes);
}

#[test]
fn khaldun_pit_teleporter_and_raisable_round_trip() {
    use crate::khaldun::items;
    let mut w = World::new();
    let id = items::pit_new(
        &mut w,
        Point3D::new(1, 2, 3),
        Some(muo_server::map::MapId::FELUCCA),
    );
    let bytes = save(&w, id);
    let id2 = load(&mut w, &items::KHALDUN_PIT_TELEPORTER, &bytes);
    assert_eq!(save(&w, id2), bytes);
    let r = items::raisable_new(
        &mut w,
        0x1234,
        30,
        -1,
        -1,
        muo_primitives::TimeSpan::from_mins(1),
    );
    let bytes = save(&w, r);
    let r2 = load(&mut w, &items::RAISABLE_ITEM, &bytes);
    assert_eq!(save(&w, r2), bytes);
    assert!(items::is_raisable(&w, r2));
}

#[test]
fn doom_gauntlet_spawner_round_trips() {
    use crate::doom::gauntlet::{self, GAUNTLET_SPAWNER};
    let mut w = World::new();
    let id = gauntlet::new_spawner(&mut w, Some("Impaler"));
    gauntlet::data_mut(&mut w, id).unwrap().region_bounds = Rectangle2D::new(5, 6, 7, 8);
    let bytes = save(&w, id);
    let id2 = load(&mut w, &GAUNTLET_SPAWNER, &bytes);
    let d = gauntlet::data(&w, id2).unwrap();
    assert_eq!(d.type_name.as_deref(), Some("Impaler"));
    assert_eq!(d.region_bounds, Rectangle2D::new(5, 6, 7, 8));
    assert_eq!(save(&w, id2), bytes);
}

#[test]
fn writer_is_in_scope() {
    // keeps `GenericWriter` imported for the golden helpers above
    let mut out = SaveWriter::new(true);
    out.write_i32(1);
    assert_eq!(out.into_vec(), vec![1, 0, 0, 0]);
}

fn engines_world() -> World {
    let mut w = World::new();
    muo_content::register(&mut w.types);
    muo_items::register(&mut w.types);
    crate::register(&mut w.types);
    w
}

#[test]
fn plant_item_golden_bytes_and_round_trip() {
    use crate::plants::item::{self, PLANT_ITEM};
    let mut w = World::new();
    let id = item::plant_new(&mut w, true);
    assert_eq!(w.item(id).unwrap().item_id, 0x1602);
    let bytes = save(&w, id);
    let mut want = base_bytes(&w, id);
    want.extend_from_slice(&[3, 0, 0, 0]); // version
    want.push(0x20); // SaveFlag.PlantSystem only: Owner level, BowlOfDirt, type 0 and PlantHue.None are defaults
    // PlantSystem: version 3, flags FertileDirt | Hits (water is 0, left seeds and resources are 8)
    want.extend_from_slice(&[3, 0, 0, 0]);
    want.push(0x09);
    let sys = item::sys(&w, id).unwrap();
    let mut tmp = SaveWriter::new(true);
    tmp.write_date_time_ticks(sys.next_growth.ticks());
    tmp.write_i32(10); // Hits = MaxHits of a bowl of dirt
    want.extend_from_slice(&tmp.into_vec());
    assert_eq!(bytes, want);
    let id2 = load(&mut w, &PLANT_ITEM, &bytes);
    assert_eq!(save(&w, id2), bytes);
    assert!(item::sys(&w, id2).unwrap().fertile_dirt);
    assert_eq!(item::sys(&w, id2).unwrap().left_seeds, 8);
}

#[test]
fn plant_old_client_property_list_leaves_out_the_container() {
    use crate::plants::item;
    let mut w = World::new();
    let id = item::plant_new(&mut w, false);
    let new = muo_server::item::ops::property_list(&mut w, id)
        .unwrap()
        .clone();
    let old = item::old_client_list(&mut w, id).unwrap().clone();
    // a ~1_val~ of ~2_val~ dirt: #1150435 (container) is only an argument for 7.0.12.0 and later
    assert_eq!(new.header, 1060830);
    assert_eq!(old.header, 1060830);
    assert!(
        new.header_args
            .as_deref()
            .unwrap()
            .starts_with("#1150435	#")
    );
    assert!(!old.header_args.as_deref().unwrap().contains(char::from(9)));
    assert_ne!(new.hash(), old.hash());
    // ClearProperties drops the old list too
    let ty = w.item(id).unwrap().ty;
    (ty.clear_properties)(&mut w, id);
    assert!(item::data(&w, id).unwrap().old_list.is_none());
}

#[test]
fn plant_status_changes_update_graphic_and_hits() {
    use crate::plants::{hue, item, plant_type};
    let mut w = World::new();
    let id = item::plant_new(&mut w, false);
    item::set_plant_type(&mut w, id, plant_type::FERN);
    item::set_plant_hue(&mut w, id, hue::BLUE);
    item::set_status(&mut w, id, item::status::SEED);
    assert_eq!(item::sys(&w, id).unwrap().hits, 12); // 10 + 1 * 2, at full health
    item::set_status(&mut w, id, item::status::PLANT);
    assert_eq!(w.item(id).unwrap().item_id, 0x1600);
    item::set_status(&mut w, id, item::status::FULL_GROWN_PLANT);
    assert_eq!(w.item(id).unwrap().item_id, 0xC9F); // Fern
    assert_eq!(w.item(id).unwrap().hue, 0x53D);
    item::set_status(&mut w, id, item::status::DECORATIVE_PLANT);
    assert!(item::sys(&w, id).is_none());
    // out of the enum: ignored
    item::set_status(&mut w, id, 12);
    assert_eq!(item::plant_status(&w, id), item::status::DECORATIVE_PLANT);
    // a decorative plant saves no PlantSystem: flags PlantStatus | PlantType | PlantHue
    let bytes = save(&w, id);
    let id2 = load(&mut w, &item::PLANT_ITEM, &bytes);
    assert_eq!(item::plant_status(&w, id2), item::status::DECORATIVE_PLANT);
    assert_eq!(item::plant_hue(&w, id2), hue::BLUE);
}

#[test]
fn plant_dies_when_hits_reach_zero() {
    use crate::plants::item;
    let mut w = World::new();
    let id = item::plant_new(&mut w, false);
    item::set_status(&mut w, id, item::status::SEED);
    item::set_hits(&mut w, id, 0);
    // a seed that dies returns to a bowl of dirt, reset
    assert_eq!(item::plant_status(&w, id), item::status::BOWL_OF_DIRT);
    assert_eq!(item::sys(&w, id).unwrap().hits, 10);
    item::set_status(&mut w, id, item::status::FULL_GROWN_PLANT);
    item::set_hits(&mut w, id, 0);
    assert_eq!(item::plant_status(&w, id), item::status::DEAD_TWIGS);
}

#[test]
fn plant_potions_and_water_clamp() {
    use crate::plants::item;
    use muo_items::skill_items::potions::fx;
    let mut w = World::new();
    let id = item::plant_new(&mut w, false);
    assert_eq!(
        item::apply_potion(&mut w, id, fx::HEAL_GREATER, false),
        (false, 1053066)
    ); // dirt only
    item::set_status(&mut w, id, item::status::SEED);
    assert_eq!(
        item::apply_potion(&mut w, id, fx::HEAL, false),
        (false, 1053068)
    );
    assert_eq!(
        item::apply_potion(&mut w, id, fx::REFRESH, false),
        (false, 1053069)
    );
    assert_eq!(
        item::apply_potion(&mut w, id, fx::HEAL_GREATER, false),
        (true, 1053067)
    );
    assert_eq!(
        item::apply_potion(&mut w, id, fx::HEAL_GREATER, false),
        (true, 1053067)
    );
    assert_eq!(
        item::apply_potion(&mut w, id, fx::HEAL_GREATER, false),
        (false, 1053065)
    ); // already soaked
    assert_eq!(item::sys(&w, id).unwrap().heal_potion, 2);
    item::set_water(&mut w, id, 9);
    assert_eq!(item::sys(&w, id).unwrap().water, 4);
}

#[test]
fn plant_resources_table() {
    use crate::plants::{hue, item, plant_type};
    assert_eq!(
        item::resource_info(plant_type::SNAKE_PLANT, hue::BRIGHT_GREEN),
        Some("Server.Items.GreenThorns")
    );
    assert_eq!(
        item::resource_info(plant_type::SNAKE_PLANT, hue::GREEN),
        None
    );
}

#[test]
fn plant_misc_items_construct_and_round_trip() {
    use crate::plants::misc;
    let mut w = engines_world();
    for ty in [
        &misc::FERTILE_DIRT,
        &misc::PLANT_BOWL,
        &misc::ORANGE_PETALS,
        &misc::RED_LEAVES,
        &misc::GREEN_THORNS,
    ] {
        let id = (ty.construct.unwrap())(&mut w);
        let bytes = save(&w, id);
        let id2 = load(&mut w, ty, &bytes);
        assert_eq!(save(&w, id2), bytes);
    }
    let id = (misc::GREEN_THORNS.construct.unwrap())(&mut w);
    assert_eq!(w.item(id).unwrap().hue, 0x42);
    assert!(w.item(id).unwrap().stackable());
    // the hole of the sand effect is never saved
    assert!(!misc::GREEN_THORNS_SH_TELEPORTER.construct.is_some());
    assert!((misc::GREEN_THORNS_SH_TELEPORTER.skip_serialization)(
        &w, id
    ));
    // the worm
    let worm = (misc::GIANT_ICE_WORM.construct.unwrap())(&mut w);
    assert_eq!(w.mobile(worm).unwrap().body.0, 89);
}

#[test]
fn puzzle_solution_matching() {
    use crate::khaldun::puzzle::{cylinder::*, matches};
    let sol = [RED, RED, BLUE, GREEN, YELLOW];
    assert_eq!(matches(&sol, &sol), (true, 5, 0));
    // one red and the blue are right, the other red is not in the guess twice
    let guess = [RED, BLUE, RED, ORANGE, ORANGE];
    let (ok, cyl, col) = matches(&guess, &sol);
    assert!(!ok);
    assert_eq!((cyl, col), (1, 2));
}

#[test]
fn puzzle_chest_round_trips() {
    use crate::khaldun::puzzle::{self, METAL_GOLDEN_PUZZLE_CHEST};
    let mut w = engines_world();
    let id = (METAL_GOLDEN_PUZZLE_CHEST.construct.unwrap())(&mut w);
    // the hints are allocated after the base constructor generated them: three zeros
    let bytes = save(&w, id);
    let id2 = load(&mut w, &METAL_GOLDEN_PUZZLE_CHEST, &bytes);
    assert_eq!(save(&w, id2), bytes);
    let _ = puzzle::LENGTH;
}

#[test]
fn statue_animation_packet_golden() {
    use crate::rewards::statue::statue_animation_packet;
    let p = statue_animation_packet(muo_primitives::Serial::new(0x1234), 1, 4, 0);
    assert_eq!(
        p,
        [
            0xBF, 0, 17, 0, 0x19, 5, 0, 0, 0x12, 0x34, 0, 0xFF, 1, 0, 4, 0, 0
        ]
    );
}

#[test]
fn statue_makers_hue_label_and_save() {
    use crate::rewards::statue as st;
    let mut w = engines_world();
    let jade = (st::JADE_STATUE_MAKER.construct.unwrap())(&mut w);
    assert_eq!(w.item(jade).unwrap().item_id, 0x32F0);
    assert_eq!(w.item(jade).unwrap().hue, 0xB8F + 4);
    let bytes = save(&w, jade);
    let mut want = base_bytes(&w, jade);
    want.extend_from_slice(&[1, 0, 1]); // CharacterStatueMaker: version 1, IsRewardItem false, StatueType Jade
    want.push(0); // JadeStatueMaker: version 0
    assert_eq!(bytes, want);
    let jade2 = load(&mut w, &st::JADE_STATUE_MAKER, &bytes);
    assert_eq!(st::maker_type(&w, jade2), 1);
    assert_eq!(save(&w, jade2), bytes);
}

#[test]
fn statue_is_cloned_from_a_player_and_saved() {
    use crate::rewards::statue as st;
    let mut w = engines_world();
    let from = w.new_mobile(&muo_server::mobile::MOBILE_BASE);
    muo_server::mobile::base::set_name(&mut w, from, Some("Sculptor".to_owned()));
    let s = st::statue_new(&mut w, from, st::statue_type::BRONZE);
    assert_eq!(w.mobile(s).unwrap().name.as_deref(), Some("Sculptor"));
    assert_eq!(w.mobile(s).unwrap().hue, 0xB8F + 8);
    st::sculpt(&mut w, s, from);
    assert_eq!(
        st::statue(&w, s).unwrap().sculpted_by.as_deref(),
        Some("Sculptor")
    );
    let plinth = st::plinth_new(&mut w, s);
    st::statue_mut(&mut w, s).unwrap().plinth = Some(plinth);
    assert_eq!(w.item(plinth).unwrap().hue, 0xB8F + 8);
    let bytes = {
        let ty = w.mobile(s).unwrap().ty;
        let mut out = SaveWriter::new(true);
        (ty.serialize)(&w, s, &mut out);
        out.into_vec()
    };
    assert!(!bytes.is_empty());
}

#[test]
fn gates_construct_and_round_trip() {
    use crate::canned_evil::gates;
    let mut w = engines_world();
    let g = gates::star_room_gate_new(&mut w, false);
    assert_eq!(w.item(g).unwrap().item_id, 0x1FD4);
    let bytes = save(&w, g);
    let g2 = load(&mut w, &gates::STAR_ROOM_GATE, &bytes);
    assert_eq!(save(&w, g2), bytes);
}

#[test]
fn tome_and_flute_artifacts_construct() {
    use crate::tokuno::artifacts as a;
    let mut w = engines_world();
    let tome = (a::TOME_OF_LOST_KNOWLEDGE.construct.unwrap())(&mut w);
    assert_eq!(w.item(tome).unwrap().hue, 0x530);
    let bytes = save(&w, tome);
    let tome2 = load(&mut w, &a::TOME_OF_LOST_KNOWLEDGE, &bytes);
    assert_eq!(save(&w, tome2), bytes);
    let flute = (a::FLUTE_OF_RENEWAL.construct.unwrap())(&mut w);
    let d = muo_items::skill_items::instruments::data(&w, flute).unwrap();
    assert!(d.replenishes_charges && d.uses_remaining == 300 && d.slayer != 0);
}

/// `PlayerMobile.CanSee(Mobile)` calls `CharacterStatue.OnRequestedAnimation(this)`, which sends the pose packet.
#[test]
fn a_player_seeing_a_statue_is_sent_its_animation() {
    use crate::rewards::statue as st;
    use muo_server::net_state::{NetStateData, NetStateId};
    let mut w = engines_world();
    crate::rewards::configure(&mut w);
    let from = w.new_mobile(&muo_server::mobile::MOBILE_BASE);
    let s = st::statue_new(&mut w, from, st::statue_type::BRONZE);
    let viewer = muo_content::player_mobile::new_player_mobile(&mut w);
    let ns = NetStateId(7);
    w.net_states.insert(ns, NetStateData::default());
    muo_server::mobile::part1::props_b::set_net_state(&mut w, viewer, Some(ns));
    w.net_out.clear();
    let ty = w.mobile(viewer).unwrap().ty;
    (ty.can_see_mobile)(&mut w, viewer, s);
    assert!(
        !w.net_out.is_empty(),
        "the statue's animation packet was sent"
    );
}

#[test]
fn champion_titles_are_properties_of_a_player() {
    use muo_content::props::{Access, Value, resolve_chain};
    use muo_server::ids::EntityId;
    use muo_server::mobile::AccessLevel;
    let mut w = World::new();
    muo_content::props::register_core(&mut w);
    let pm = muo_content::player_mobile::new_player_mobile(&mut w);
    let e = EntityId::Mobile(pm);
    let (o, d) = resolve_chain(
        &mut w,
        AccessLevel::Administrator,
        e,
        "ChampionTitles.GladeValue",
        Access::WRITE,
    )
    .unwrap();
    (d.set.unwrap())(&mut w, o, Value::Int(250)).unwrap();
    let (o, d) = resolve_chain(
        &mut w,
        AccessLevel::Administrator,
        e,
        "ChampionTitles.GladeValue",
        Access::READ,
    )
    .unwrap();
    assert_eq!((d.get)(&mut w, o), Value::Int(250));
    let (o, d) = resolve_chain(
        &mut w,
        AccessLevel::Administrator,
        e,
        "ChampionTitles.CorruptValue",
        Access::READ,
    )
    .unwrap();
    assert_eq!((d.get)(&mut w, o), Value::Int(0)); // a different slot
}
