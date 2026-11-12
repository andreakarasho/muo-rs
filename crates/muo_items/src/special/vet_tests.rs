//! Tests of the veteran reward, holiday and bulk order reward classes (vet, vet2, holiday, holiday2, rewards2).

use muo_io::{SaveReader, SaveWriter};
use muo_server::World;
use muo_server::ids::ItemId;
use muo_server::item::{Item, ItemType};

fn save(w: &World, id: ItemId) -> Vec<u8> {
    let ty = w.item(id).unwrap().ty;
    let mut out = SaveWriter::new(true);
    (ty.serialize)(w, id, &mut out);
    out.into_vec()
}

fn round_trip(w: &World, id: ItemId) {
    let t: &'static ItemType = w.item(id).unwrap().ty;
    let bytes = save(w, id);
    let mut w2 = World::new();
    let id2 = w2.add_item(Item::new_for_load(
        muo_primitives::Serial::new(0x4000_0001),
        t,
    ));
    let mut r = SaveReader::new(&bytes);
    (t.deserialize)(&mut w2, id2, &mut r).unwrap_or_else(|e| panic!("{}: {e}", t.full_name));
    assert_eq!(r.remaining(), 0, "{} is read to its end", t.full_name);
    // an addon reads its components by serial, and the other world has none
    if !muo_content::items::addon::is_addon(w, id) {
        assert_eq!(save(&w2, id2), bytes, "{}", t.full_name);
    }
}

fn constructible() -> Vec<&'static ItemType> {
    use super::*;
    vec![
        &vet::BLOODY_PENTAGRAM_ADDON,
        &vet::BLOODY_PENTAGRAM_DEED,
        &vet::STONE_ANKH,
        &vet::STONE_ANKH_DEED,
        &vet::MINOTAUR_STATUE_DEED,
        &vet::TREE_STUMP_DEED,
        &vet::REWARD_BRAZIER,
        &vet::REWARD_BRAZIER_DEED,
        &vet::REWARD_POTTED_CACTUS,
        &vet::POTTED_CACTUS_DEED,
        &vet2::BANNER_DEED,
        &vet2::FLAMING_HEAD,
        &vet2::FLAMING_HEAD_DEED,
        &vet2::MINI_HOUSE_ADDON,
        &vet2::MINI_HOUSE_DEED,
        &vet2::CONTEST_MINI_HOUSE,
        &vet2::CONTEST_MINI_HOUSE_DEED,
        &holiday::GIFT_BOX,
        &holiday::WINTER_GIFT_PACKAGE_2003,
        &holiday::GIFT_BOX_RECTANGLE,
        &holiday::GIFT_BOX_CUBE,
        &holiday::GIFT_BOX_CYLINDER,
        &holiday::GIFT_BOX_OCTOGON,
        &holiday::GIFT_BOX_ANGEL,
        &holiday::GIFT_BOX_NEON,
        &holiday::GREEN_STOCKING,
        &holiday::RED_STOCKING,
        &holiday::SNOW_PILE_DECO,
        &holiday::HOLIDAY_BELL,
        &holiday::SNOWMAN,
        &holiday::ICY_PATCH,
        &holiday::CANDY_CANE,
        &holiday::GINGER_BREAD_COOKIE,
        &holiday::SNOW_GLOBE_ONE,
        &holiday::SNOW_GLOBE_TWO,
        &holiday::SNOW_GLOBE_THREE,
        &holiday::HOLIDAY_POTTED_PLANT,
        &holiday::POTTED_PLANT_DEED,
        &holiday::BAD_CARD,
        &holiday::SPAM,
        &holiday::SNOW_STATUE_PEGASUS,
        &holiday::SNOW_STATUE_DEED,
        &holiday2::WREATH_ADDON,
        &holiday2::WREATH_DEED,
        &vet4::HANGING_SKELETON,
        &vet4::HANGING_SKELETON_DEED,
        &vet3::ANKH_OF_SACRIFICE_DEED,
        &vet3::MINING_CART_DEED,
        &vet3::DECORATIVE_SHIELD,
        &vet3::DECORATIVE_SHIELD_DEED,
        &rewards2::OSSIAN_GRIMOIRE,
        &rewards2::COLORED_ANVIL,
        &rewards2::POWDER_OF_TEMPERAMENT,
    ]
}

#[test]
fn reward_and_holiday_classes_construct_and_round_trip() {
    for t in constructible() {
        let mut w = World::new();
        let id = (t.construct.unwrap())(&mut w);
        assert_eq!(w.item(id).unwrap().ty.full_name, t.full_name);
        round_trip(&w, id);
    }
}

#[test]
fn the_classes_that_need_an_argument_round_trip() {
    use super::*;
    let mut w = World::new();
    let ids = [
        vet2::banner_new(&mut w, 0x15AE),
        vet3::mining_cart_new(&mut w, 102),
        vet3::ankh_of_sacrifice_addon_new(&mut w, true),
        vet::minotaur_statue_new(&mut w, 100),
        vet::tree_stump_new(&mut w, 0xE56),
        rewards2::ancient_smithy_hammer_new(&mut w, 5, 600),
        rewards2::leather_gloves_of_mining_new_bonus(&mut w, 1),
        rewards2::studded_gloves_of_mining_new_bonus(&mut w, 3),
        rewards2::ringmail_gloves_of_mining_new_bonus(&mut w, 5),
    ];
    for id in ids {
        round_trip(&w, id);
    }
}

#[test]
fn reward_flag_is_saved_after_the_base() {
    use super::*;
    let mut w = World::new();
    let id = (vet::BLOODY_PENTAGRAM_DEED.construct.unwrap())(&mut w);
    vet::set_reward(&mut w, id, true);
    let bytes = save(&w, id);
    // encoded version 0, IsRewardItem
    assert!(bytes.ends_with(&[0, 1]), "{bytes:02x?}");
}

#[test]
fn a_mini_house_builds_its_square_of_components() {
    use super::*;
    let mut w = World::new();
    // Castle: 16 graphics, a 4x4 square
    let id = vet2::mini_house_addon_new(&mut w, 11);
    assert_eq!(muo_content::items::addon::components(&w, id).len(), 16);
    // Church At Night: the last graphic is 0x1 and builds nothing
    let id = vet2::mini_house_addon_new(&mut w, 21);
    assert_eq!(muo_content::items::addon::components(&w, id).len(), 3);
}

#[test]
fn tree_stump_version_zero_loads() {
    use super::*;
    let mut w = World::new();
    w.now = muo_primitives::DateTime::from_ticks(1_000_000_000_000_000);
    let id = vet::tree_stump_new(&mut w, 0xE56);
    let mut bytes = save(&w, id);
    // turn the encoded version 1 into 0: the last field is then a next-check time, not the last check time
    let n = bytes.len() - (1 + 1 + 4 + 8);
    bytes[n] = 0;
    let mut w2 = World::new();
    w2.now = w.now;
    let id2 = w2.add_item(Item::new_for_load(
        muo_primitives::Serial::new(0x4000_0002),
        &vet::TREE_STUMP,
    ));
    let mut r = SaveReader::new(&bytes);
    (vet::TREE_STUMP.deserialize)(&mut w2, id2, &mut r).unwrap();
    assert_eq!(r.remaining(), 0);
}

#[test]
fn snowman_has_a_title_and_the_globes_a_place() {
    use super::*;
    let mut w = World::new();
    let id = (holiday::SNOWMAN.construct.unwrap())(&mut w);
    holiday::set_snowman_title(&mut w, id, Some("Zippy".into()));
    round_trip(&w, id);
    let g = holiday::snow_globe_one_new(&mut w, 3);
    assert_eq!((holiday::SNOW_GLOBE_ONE.label_number)(&w, g), 1041454 + 3);
    let g = holiday::snow_globe_three_new(&mut w, 5);
    assert_eq!((holiday::SNOW_GLOBE_THREE.label_number)(&w, g), 1075441);
}

#[test]
fn gloves_of_mining_are_not_constructible_without_a_bonus() {
    for t in [
        &super::rewards2::LEATHER_GLOVES_OF_MINING,
        &super::rewards2::STUDDED_GLOVES_OF_MINING,
        &super::rewards2::RINGMAIL_GLOVES_OF_MINING,
    ] {
        assert!(t.construct.is_none(), "{}", t.full_name);
    }
}
