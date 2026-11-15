//! Tests of the items, quest givers and race changer of Engines/ML Quests that sit on other crates' classes.

use muo_content::player_mobile::new_player_mobile;
use muo_io::text::TextDefinition;
use muo_io::{GenericReader, SaveReader, SaveWriter};
use muo_primitives::{Point3D, TimeSpan};
use muo_server::World;
use muo_server::ids::{ItemId, MobileId};
use muo_server::item::{ItemType, ops};
use muo_server::mobile::MobileType;

use super::engine;
use super::items2::*;
use super::model::*;
use super::npcs_more;
use super::race_change::{self, DARIUS, NEDRICK, RACE_CHANGE_DEED};
use super::transient::*;

fn world() -> World {
    let mut w = World::new();
    muo_content::register(&mut w.types);
    muo_items::register(&mut w.types);
    muo_mobiles::register(&mut w.types);
    super::register(&mut w.types);
    crate::template_keys::register(&mut w.types);
    state(&mut w).enabled = true;
    w
}

fn item_bytes(w: &World, id: ItemId) -> Vec<u8> {
    let ty = w.item(id).unwrap().ty;
    let mut out = SaveWriter::new(true);
    (ty.serialize)(w, id, &mut out);
    out.into_vec()
}

fn mobile_bytes(w: &World, id: MobileId) -> Vec<u8> {
    let ty = w.mobile(id).unwrap().ty;
    let mut out = SaveWriter::new(true);
    (ty.serialize)(w, id, &mut out);
    out.into_vec()
}

/// Builds the class, saves it, loads the bytes into a new item of the class and saves again.
fn round_trip_item(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = (ty.construct.expect("constructible"))(w);
    let bytes = item_bytes(w, id);
    let id2 = ops::new_item_with_id(w, ty, 0);
    let mut r = SaveReader::new(&bytes);
    (ty.deserialize)(w, id2, &mut r).unwrap();
    assert_eq!(r.remaining(), 0, "{} reads what it wrote", ty.full_name);
    assert_eq!(
        item_bytes(w, id2),
        bytes,
        "{} saves what it read",
        ty.full_name
    );
    id
}

#[test]
fn ml_quests_leaf_items_construct_and_round_trip_with_their_own_hues_and_labels() {
    let mut w = world();
    for (ty, hue, label) in [
        (&AN_OLD_NECKLACE, 0x222, 1075525),
        (&AN_OLD_RING, 0x222, 1075524),
        (&REGINAS_RING, 0, 1075305),
        (&TRANSPARENT_HEART, 0x4AB, 1075400),
        (&ANDROS_GRATITUDE, 0, 1075345),
        (&BLEACH, 0, 1075375),
        (&MINIATURE_MUSHROOM, 0, 1073138),
        (&PEPPERCORN_FISHSTEAK, 0x222, 1075557),
        (&PIXIE_LEG, 0x1C2, 1074613),
        (&PRISMATIC_AMBER, 0, 1075299),
        (&GIFT_FOR_ARIELLE, 0x2C4, 1074356),
        (&PRISMATIC_CRYSTAL, 0x32, 1074269),
    ] {
        let id = round_trip_item(&mut w, ty);
        let i = w.item(id).unwrap();
        assert!(i.ty.is(ty), "{}", ty.full_name);
        assert_eq!(i.hue, hue, "{} hue", ty.full_name);
        assert_eq!((ty.label_number)(&w, id), label, "{} label", ty.full_name);
    }
    let id = round_trip_item(&mut w, &GUSTAR_SHROUD);
    assert_eq!(w.item(id).unwrap().hue, 0x479);
    assert_eq!(w.item(id).unwrap().item_id, 0x2684);
    round_trip_item(&mut w, &RED_LEATHER_BOOK);
}

#[test]
fn ml_quests_classes_on_a_parent_write_the_parents_bytes_then_a_version() {
    let mut w = world();
    let id = (AN_OLD_RING.construct.unwrap())(&mut w);
    let ring = (muo_content::items::leaves::GOLD_RING.construct.unwrap())(&mut w);
    let (a, b) = (item_bytes(&w, id), item_bytes(&w, ring));
    // `AnOldRing` is a GoldRing with Hue 0x222 (the item writes it as `WriteEncodedInt`, 2 bytes for 0x222)
    // and one more version int
    assert_eq!(a.len(), b.len() + 2 + 4);
    let mut tail = SaveReader::new(&a[a.len() - 4..]);
    assert_eq!(tail.read_i32().unwrap(), 0);
}

#[test]
fn ml_quests_transient_items_expire_and_keep_their_life_in_the_save() {
    let mut w = world();
    for (ty, secs) in [
        (&A_PERSONAL_LETTER_ADDRESSED_TO_AHIE, 30 * 60),
        (&CRATE_FOR_SLEDGE, 3600),
        (&SPECKLED_POISON_SAC, 3600),
        (&BATTERED_BUCKET, 600),
    ] {
        let id = round_trip_item(&mut w, ty);
        let exp = muo_spells::spellweaving::items::expiration(&w, id);
        assert_eq!(exp - w.now, TimeSpan::from_secs(secs), "{}", ty.full_name);
        assert!((ty.nontransferable)(&w, id));
    }
    let bucket = (BATTERED_BUCKET.construct.unwrap())(&mut w);
    assert_eq!(
        (BATTERED_BUCKET.default_name)(&w, bucket),
        Some("battered bucket".to_string())
    );
}

#[test]
fn ml_quests_reward_bags_hold_up_to_their_count_of_items() {
    let mut w = world();
    for (ty, max) in [
        (&SMALL_BAG_OF_TRINKETS, 1),
        (&BAG_OF_TRINKETS, 2),
        (&BAG_OF_TREASURE, 3),
        (&LARGE_BAG_OF_TREASURE, 4),
        (&REWARD_STRONGBOX, 5),
    ] {
        let id = round_trip_item(&mut w, ty);
        let n = w.item(id).unwrap().items.len();
        assert!(n <= max, "{}: {n} items", ty.full_name);
    }
}

#[test]
fn ml_quests_satchels_are_bright_backpacks_that_round_trip() {
    let mut w = world();
    for ty in [
        &TAILOR_SATCHEL,
        &BLACKSMITH_SATCHEL,
        &TINKER_SATCHEL,
        &FLETCHING_SATCHEL,
        &CARPENTRY_SATCHEL,
    ] {
        let id = round_trip_item(&mut w, ty);
        let i = w.item(id).unwrap();
        assert_eq!(i.layer, muo_server::layer::Layer::Backpack);
        assert_ne!(i.hue, 0, "{} is dyed", ty.full_name);
    }
}

#[test]
fn ml_quests_teleporters_save_the_type_hash_and_the_message_after_the_teleporter() {
    let mut w = world();
    let quest = "Server.Engines.MLQuests.Definitions.Aemaeth1";
    let id = quest_teleporter_new(
        &mut w,
        &ML_QUEST_TELEPORTER,
        Point3D::new(1, 2, 3),
        None,
        Some(quest),
        Some(TextDefinition::of_number(1074000)),
    );
    let bytes = item_bytes(&w, id);
    let mut base = SaveWriter::new(true);
    (muo_items::misc::teleporter::TELEPORTER.serialize)(&w, id, &mut base);
    let mut r = SaveReader::new(&bytes[base.as_slice().len()..]);
    assert_eq!(r.read_i32().unwrap(), 1);
    assert_eq!(r.read_u8().unwrap(), 2);
    assert_eq!(
        r.read_u64().unwrap(),
        muo_primitives::hash::compute_hash64(quest)
    );
    assert_eq!(r.read_encoded_int().unwrap(), 1);
    assert_eq!(r.read_encoded_int().unwrap(), 1074000);
    assert_eq!(r.remaining(), 0);

    let id2 = ops::new_item_with_id(&mut w, &ML_QUEST_TELEPORTER, 0);
    let mut r = SaveReader::new(&bytes);
    (ML_QUEST_TELEPORTER.deserialize)(&mut w, id2, &mut r).unwrap();
    assert_eq!(r.remaining(), 0);
    assert_eq!(item_bytes(&w, id2), bytes);

    // the default constructor: no type, no message
    let plain = round_trip_item(&mut w, &TICKET_TELEPORTER);
    let b = item_bytes(&w, plain);
    assert_eq!(&b[b.len() - 2..], &[0, 3]); // no type, a null message
}

// `QuestType` / `TicketType` are `Type` properties: the saved hash names the class back.
#[test]
fn ml_quests_teleporter_type_properties_name_the_class_and_set_it() {
    use muo_content::props::Value;
    let mut w = world();
    let class = super::defs::ALL
        .iter()
        .find(|c| c.name() == "Aemaeth1")
        .expect("Aemaeth1 is registered");
    engine::register_quest(&mut w, class, &[]);
    let id = quest_teleporter_new(
        &mut w,
        &ML_QUEST_TELEPORTER,
        Point3D::new(1, 2, 3),
        None,
        Some(class.full_name),
        None,
    );
    assert_eq!(
        type_prop_get(&w, id, true),
        Value::Text(class.full_name.to_owned())
    );
    type_prop_set(&mut w, id, true, &Value::Null).unwrap();
    assert_eq!(type_prop_get(&w, id, true), Value::Null);
    type_prop_set(&mut w, id, true, &Value::Str("Aemaeth1".into())).unwrap();
    assert_eq!(
        type_prop_get(&w, id, true),
        Value::Text(class.full_name.to_owned())
    );
    assert!(type_prop_set(&mut w, id, true, &Value::Str("NoSuchQuest".into())).is_err());

    let t = quest_teleporter_new(
        &mut w,
        &TICKET_TELEPORTER,
        Point3D::new(1, 2, 3),
        None,
        None,
        None,
    );
    type_prop_set(&mut w, t, false, &Value::Str("Gold".into())).unwrap();
    assert_eq!(
        type_prop_get(&w, t, false),
        Value::Text("Server.Items.Gold".into())
    );
}
#[test]
fn ml_quests_race_change_deed_is_blessed_and_round_trips() {
    let mut w = world();
    let id = round_trip_item(&mut w, &RACE_CHANGE_DEED);
    assert_eq!(w.item(id).unwrap().item_id, 0x14F0);
    assert_eq!(
        (RACE_CHANGE_DEED.default_name)(&w, id),
        Some("a race change deed".to_string())
    );
    assert_eq!(
        w.item(id).unwrap().loot_type,
        muo_server::item::LootType::Blessed
    );
}

fn round_trip_mobile(w: &mut World, ty: &'static MobileType) -> MobileId {
    let id = (ty.construct.expect("constructible"))(w);
    let bytes = mobile_bytes(w, id);
    let id2 = w.new_mobile(ty);
    let mut r = SaveReader::new(&bytes);
    (ty.deserialize)(w, id2, &mut r).unwrap();
    assert_eq!(r.remaining(), 0, "{} reads what it wrote", ty.full_name);
    assert_eq!(
        mobile_bytes(w, id2),
        bytes,
        "{} saves what it read",
        ty.full_name
    );
    id
}

#[test]
fn ml_quests_instructors_on_vendor_bases_build_and_round_trip() {
    let mut w = world();
    for (ty, title, hue) in [
        (&npcs_more::AELORN, "the Chivalry Instructor", 0x83EA),
        (&npcs_more::ROBYN, "the Archery Instructor", 0x83EA),
        (
            &npcs_more::AMELIA_YOUNGSTONE,
            "the Tinkering Instructor",
            0x83EA,
        ),
        (&npcs_more::MULCIVIKH, "the Necromancy Instructor", 0x83EA),
        (&npcs_more::MORGANNA, "the Spirit Speak Instructor", 0x83EA),
        (
            &npcs_more::GEORGE_HEPHAESTUS,
            "the Blacksmith Instructor",
            0x83EA,
        ),
        (&npcs_more::RYUICHI, "the Ninjitsu Instructor", 0x8403),
        (&npcs_more::HAMATO, "the Bushido Instructor", 0x8403),
        (&npcs_more::SIR_HELPER, "the Profession Guide", 0x83EA),
    ] {
        let id = round_trip_mobile(&mut w, ty);
        let m = w.mobile(id).unwrap();
        assert!(m.ty.is(ty));
        assert_eq!(m.title.as_deref(), Some(title), "{}", ty.full_name);
        assert_eq!(m.hue, hue, "{}", ty.full_name);
    }
}

#[test]
fn ml_quests_creatures_on_monster_bases_are_vendor_ai_and_round_trip() {
    let mut w = world();
    for (ty, name, invulnerable) in [
        (&npcs_more::MAUL_THE_BEAR, "Maul", true),
        (&npcs_more::STRONGROOT, "Strongroot", true),
        (&npcs_more::BRAVEHORNS_MATE, "bravehorn's mate", false),
        (&npcs_more::HUNTSMAN, "Huntsman", true),
        (&npcs_more::ARIELLE, "Arielle", true),
    ] {
        let id = round_trip_mobile(&mut w, ty);
        let c = muo_content::creature::ctype(&w, id).unwrap();
        assert_eq!((c.default_name)(&w, id), Some(name));
        assert_eq!(
            (c.is_invulnerable)(&w, id),
            invulnerable,
            "{}",
            ty.full_name
        );
        let tamable = muo_content::creature::data::read(&w, id, true, |d| d.tamable_raw);
        if ty.full_name.ends_with("MaulTheBear") || ty.full_name.ends_with("BravehornsMate") {
            assert!(!tamable);
        }
    }
}

#[test]
fn ml_quests_race_collectors_talk_to_the_other_race_only() {
    let mut w = world();
    let darius = round_trip_mobile(&mut w, &DARIUS);
    let nedrick = round_trip_mobile(&mut w, &NEDRICK);
    assert_eq!(w.mobile(darius).unwrap().title.as_deref(), Some("the wise"));
    assert_eq!(
        w.mobile(nedrick).unwrap().title.as_deref(),
        Some("the iron worker")
    );
    let pm = new_player_mobile(&mut w);
    // without a context nothing is done yet; the pending table starts empty
    assert!(engine::get_context(&w, pm).is_none());
    assert!(!race_change::is_wearing_equipment(&w, pm));
}
