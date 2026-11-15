//! Tests of the clothing, jewel, gem, suit, quiver, wand and talisman classes: the registered names are unique,
//! every class constructs and round-trips its save bytes in each era, and a few records are checked against fixed
//! save bytes.

use std::collections::HashSet;

use muo_content::creature::speeds::{NpcSpeeds, SpeedClassEntry};
use muo_data::expansion::Expansion;
use muo_io::{SaveReader, SaveWriter};
use muo_server::World;
use muo_server::ids::{ItemId, MobileId};
use muo_server::item::{Item, ItemType};
use muo_server::mobile::AccessLevel;
use muo_server::type_registry::TypeRegistry;

use crate::{clothing, gems, jewels, quivers, suits, talismans, wands};

/// Every class of the modules of this task (not the samples of `muo_content::items::leaves`).
fn registry() -> TypeRegistry {
    let mut types = TypeRegistry::default();
    clothing::register(&mut types);
    jewels::register(&mut types);
    gems::register(&mut types);
    suits::register(&mut types);
    quivers::register(&mut types);
    wands::register(&mut types);
    talismans::register(&mut types);
    types
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

/// The classes that have no `[Constructible]` constructor, with the one the code calls.
fn build_unconstructible(w: &mut World, ty: &'static ItemType) -> Option<ItemId> {
    Some(match ty.full_name {
        "Server.Items.BaseQuiver" => quivers::new_quiver(w, ty, 0x2FB7),
        "Server.Items.BaseTalisman" => talismans::new_talisman_random(w, ty),
        "Server.Items.AdminRobe" => suits::admin_robe_new(w, ty),
        "Server.Items.CounselorRobe" => suits::counselor_robe_new(w, ty),
        "Server.Items.GMRobe" => suits::gm_robe_new(w, ty),
        "Server.Items.SeerRobe" => suits::seer_robe_new(w, ty),
        "Server.Items.StaffRobe" => suits::staff_robe_new(w, ty, AccessLevel::Seer),
        _ => return None,
    })
}

#[test]
fn registered_names_are_unique() {
    let types = registry();
    let mut names = HashSet::new();
    for t in types.item_types() {
        assert!(names.insert(t.full_name), "{} twice", t.full_name);
        assert!(t.full_name.starts_with("Server.Items."), "{}", t.full_name);
    }
    for t in types.mobile_types() {
        assert!(names.insert(t.full_name), "{} twice", t.full_name);
    }
    // 66 + 9 clothing (Cap and FancyShirt are samples registered by muo_content), 17 jewels + Beads (GoldRing and
    // Necklace are samples), 9 gems, 9 suits, 6 quivers, 11 wands, BaseTalisman, RandomTalisman and 5 talisman items.
    assert_eq!(types.item_types().len(), 75 + 18 + 9 + 9 + 6 + 11 + 7);
    assert_eq!(types.mobile_types().len(), 18);
}

#[test]
fn every_class_constructs_and_round_trips() {
    let types = registry();
    for era in [None, Some(Expansion::AOS), Some(Expansion::ML)] {
        for &ty in types.item_types() {
            let mut w = crate::test_world();
            w.core.expansion = era;
            muo_items_register(&mut w);
            let id = match ty.construct {
                Some(f) => f(&mut w),
                None => build_unconstructible(&mut w, ty)
                    .unwrap_or_else(|| panic!("{} has no constructor", ty.full_name)),
            };
            assert_eq!(w.item(id).unwrap().ty.full_name, ty.full_name);
            if talismans::is_talisman(&w, id) {
                // `ReadType` finds a class only when it is registered; the killers are creatures of other crates
                let text = muo_io::text::TextDefinition::of_number(1072494);
                talismans::set_killer_type(
                    &mut w,
                    id,
                    "Server.Mobiles.SummonedCow",
                    text.clone(),
                    25,
                );
                talismans::set_protection_type(
                    &mut w,
                    id,
                    "Server.Mobiles.SummonedSheep",
                    text.clone(),
                    15,
                );
                talismans::set_summoner_type(&mut w, id, "Server.Mobiles.SummonedCow", text);
            }
            let bytes = save(&w, id);

            let mut w2 = crate::test_world();
            w2.core.expansion = era;
            muo_items_register(&mut w2);
            let other = for_load(&mut w2, 0x4000_0001, ty);
            let mut r = SaveReader::new(&bytes);
            (ty.deserialize)(&mut w2, other, &mut r)
                .unwrap_or_else(|e| panic!("{}: {e}", ty.full_name));
            assert_eq!(r.remaining(), 0, "{} is read to its end", ty.full_name);
            let again = save(&w2, other);
            // A clothing record written without its resource flag loads as `CraftResource.None` (BaseClothing's
            // generated reader has no `else`), so a class with a default resource differs once; every record is
            // stable from the second save on.
            let has_default_resource = muo_content::items::clothing::data(&w, id)
                .is_some_and(|d| d.resource != muo_content::craft::CraftResource::None)
                && bytes != again;
            if !has_default_resource {
                // the serial is the only difference between the two records
                assert_eq!(
                    bytes.len(),
                    again.len(),
                    "{} {bytes:?} {again:?}",
                    ty.full_name
                );
                assert_eq!(&bytes[8..], &again[8..], "{} {era:?}", ty.full_name);
            }
            let third = for_load(&mut w2, 0x4000_0002, ty);
            let mut r = SaveReader::new(&again);
            (ty.deserialize)(&mut w2, third, &mut r)
                .unwrap_or_else(|e| panic!("{}: {e}", ty.full_name));
            assert_eq!(r.remaining(), 0, "{} is read to its end", ty.full_name);
            assert_eq!(
                &again[8..],
                &save(&w2, third)[8..],
                "{} is stable",
                ty.full_name
            );
        }
    }
}

/// `w.types` holds what `ReadType` looks up (the talismans' `TalismanAttribute.Type`).
fn muo_items_register(w: &mut World) {
    crate::register(&mut w.types);
    muo_content::items::leaves::register(&mut w.types);
}

#[test]
fn every_summon_constructs_and_round_trips() {
    let mut w = crate::test_world();
    w.resources
        .get_mut::<NpcSpeeds>()
        .register(SpeedClassEntry {
            level: "Medium".into(),
            active: 0.3,
            passive: 0.6,
            active_move: 0.0,
            passive_move: 0.0,
            types: vec![],
        });
    for &ty in talismans::summons::LEAVES {
        let id: MobileId = (ty.construct.unwrap())(&mut w);
        let mut out = SaveWriter::new(true);
        (ty.serialize)(&w, id, &mut out);
        let bytes = out.into_vec();
        let other = w.new_mobile(ty);
        let mut r = SaveReader::new(&bytes);
        (ty.deserialize)(&mut w, other, &mut r).unwrap_or_else(|e| panic!("{}: {e}", ty.full_name));
        assert_eq!(r.remaining(), 0, "{} is read to its end", ty.full_name);
    }
}

#[test]
fn a_gem_record_is_the_item_and_a_version() {
    let mut w = crate::test_world();
    let id = (gems::AMBER.construct.unwrap())(&mut w);
    let item = w.item(id).unwrap();
    assert_eq!(
        (item.item_id, item.amount, item.stackable()),
        (0xF25, 1, true)
    );
    let bytes = save(&w, id);
    // `[SerializationGenerator(0, false)]`: the version is an int
    assert_eq!(&bytes[bytes.len() - 4..], &[0, 0, 0, 0]);
}

#[test]
fn gem_types_map_to_their_classes() {
    use muo_content::items::jewel::GemType;
    for g in 1..=9 {
        let gem = GemType::from_i32(g).unwrap();
        let ty = gems::gem_item_type(gem).unwrap();
        assert_eq!(gems::gem_type_of(ty), gem);
    }
    assert!(gems::gem_item_type(GemType::None).is_none());
    assert_eq!(gems::gem_type_of(&clothing::special::CLOAK), GemType::None);
}

#[test]
fn the_arcane_cloak_swaps_its_graphic_and_writes_encoded_charges() {
    let mut w = crate::test_world();
    let id = (clothing::special::CLOAK.construct.unwrap())(&mut w);
    assert_eq!(w.item(id).unwrap().item_id, 0x1515);
    clothing::special::set_max_arcane_charges(&mut w, id, 20);
    clothing::special::set_cur_arcane_charges(&mut w, id, 20);
    assert_eq!(w.item(id).unwrap().item_id, 0x26AD);
    let bytes = save(&w, id);
    // version 2 (an int), then the two charges as encoded ints
    assert_eq!(&bytes[bytes.len() - 6..], &[2, 0, 0, 0, 20, 20]);
    // no charges left: the cloak loses its hue, and back to normal when not arcane
    clothing::special::set_cur_arcane_charges(&mut w, id, 0);
    assert_eq!(w.item(id).unwrap().hue, 0);
    clothing::special::set_max_arcane_charges(&mut w, id, 0);
    assert_eq!(w.item(id).unwrap().item_id, 0x1515);
}

#[test]
fn an_old_arcane_robe_record_reads_its_bool_and_ints() {
    let mut w = crate::test_world();
    let id = (clothing::special::ROBE.construct.unwrap())(&mut w);
    let mut bytes = save(&w, id);
    // replace the trailing `version 2, 0, 0` by `version 1, true, 5, 10`
    bytes.truncate(bytes.len() - 6);
    bytes.extend_from_slice(&[1, 0, 0, 0, 1, 5, 0, 0, 0, 10, 0, 0, 0]);
    let other = for_load(&mut w, 0x4000_0002, &clothing::special::ROBE);
    let mut r = SaveReader::new(&bytes);
    (clothing::special::ROBE.deserialize)(&mut w, other, &mut r).unwrap();
    assert_eq!(r.remaining(), 0);
    let d = muo_content::items::clothing::data(&w, other).unwrap();
    assert_eq!(d.arcane_charges, (5, 10));
}

#[test]
fn the_reward_cloak_keeps_its_label_and_blessing() {
    let mut w = crate::test_world();
    let id = clothing::special::reward_cloak_new(
        &mut w,
        &clothing::special::REWARD_CLOAK,
        0x30,
        1076217,
    );
    assert_eq!(
        w.item(id).unwrap().loot_type,
        muo_server::item::LootType::Blessed
    );
    let ty = w.item(id).unwrap().ty;
    assert_eq!((ty.label_number)(&w, id), 1076217);
    let bytes = save(&w, id);
    // version 0, `_number`, `_isRewardItem`
    let n = bytes.len();
    let mut tail = vec![0, 0, 0, 0];
    tail.extend_from_slice(&1076217i32.to_le_bytes());
    tail.push(0);
    assert_eq!(&bytes[n - 9..], &tail[..]);
}

#[test]
fn a_death_robe_decays_and_stops_when_picked_up() {
    let mut w = crate::test_world();
    let id = (clothing::special::DEATH_ROBE.construct.unwrap())(&mut w);
    assert_eq!(w.item(id).unwrap().hue, 2301);
    assert!(
        muo_content::items::clothing::data(&w, id)
            .unwrap()
            .decay_timer
            != Default::default()
    );
    clothing::special::death_robe_begin_decay(&mut w, id, muo_primitives::TimeSpan::from_secs(60));
    let bytes = save(&w, id);
    // the robe's own part ends with version 4 and the anchored time of the timer's next tick (not MinValue)
    assert_eq!(&bytes[bytes.len() - 12..bytes.len() - 8], &[4, 0, 0, 0]);
}

#[test]
fn a_quiver_weighs_less_and_holds_one_ammo_type() {
    let mut w = crate::test_world();
    let id = (quivers::ELVEN_QUIVER.construct.unwrap())(&mut w);
    let d = quivers::data(&w, id).unwrap();
    assert_eq!(
        (d.capacity, d.weight_reduction, d.damage_increase),
        (500, 30, 10)
    );
    assert_eq!(w.item(id).unwrap().layer, muo_server::layer::Layer::Cloak);
    let fire = (quivers::QUIVER_OF_FIRE.construct.unwrap())(&mut w);
    assert_eq!(w.item(fire).unwrap().hue, 0x4E7);
    let mut split = [0; 7];
    quivers::alter_bow_damage(&w, fire, &mut split);
    assert_eq!(split, [50, 50, 0, 0, 0, 0, 0]);
    // the elven quiver's own part is its encoded version 0; before it, the base's flags (damage increase 0x8,
    // weight reduction 0x4, quality Low is not Regular 0x20, capacity 0x40) and values
    let bytes = save(&w, id);
    // (flags are an encoded int, the quality `Low` an encoded enum, the elven quiver's version an encoded int)
    let tail = [0x6C, 30, 0, 0, 0, 10, 0, 0, 0, 0, 0xF4, 1, 0, 0, 0];
    assert!(bytes.ends_with(&tail[..]), "{bytes:?}");
}

#[test]
fn a_wand_has_charges_and_mage_weapon() {
    let mut w = crate::test_world();
    w.core.expansion = Some(Expansion::AOS);
    let id = (wands::HEAL_WAND.construct.unwrap())(&mut w);
    let c = wands::charges(&w, id);
    assert!((10..=25).contains(&c), "{c}");
    assert_eq!(wands::wand_effect(&w, id), Some(wands::WandEffect::Healing));
    let graphic = w.item(id).unwrap().item_id;
    assert!((0xDF2..=0xDF5).contains(&graphic));
    w.core.expansion = Some(Expansion::ML);
    let id = (wands::HEAL_WAND.construct.unwrap())(&mut w);
    assert!((10..=109).contains(&wands::charges(&w, id)));
    // BaseWand: version 1 (int), the effect (an encoded int), the charges; the leaf: version 0
    let bytes = save(&w, id);
    let n = bytes.len();
    let charges = wands::charges(&w, id).to_le_bytes();
    assert_eq!(&bytes[n - 4..], &[0, 0, 0, 0]);
    assert_eq!(&bytes[n - 8..n - 4], &charges);
    assert_eq!(bytes[n - 9], wands::WandEffect::Healing as u8);
    assert_eq!(&bytes[n - 13..n - 9], &[1, 0, 0, 0]);
}

#[test]
fn a_random_talisman_round_trips_its_attributes() {
    let mut w = crate::test_world();
    muo_items_register(&mut w);
    for _ in 0..50 {
        let id = (talismans::RANDOM_TALISMAN.construct.unwrap())(&mut w);
        // `ReadType` finds a class only when it is registered; the killers are creatures of other crates
        let text = muo_io::text::TextDefinition::of_number(1072494);
        talismans::set_killer_type(&mut w, id, "Server.Mobiles.SummonedCow", text.clone(), 25);
        talismans::set_protection_type(
            &mut w,
            id,
            "Server.Mobiles.SummonedSheep",
            text.clone(),
            15,
        );
        talismans::set_summoner_type(&mut w, id, "Server.Mobiles.SummonedCow", text);
        let a = save(&w, id);
        let other = for_load(&mut w, 0x4000_0100, &talismans::RANDOM_TALISMAN);
        let mut r = SaveReader::new(&a);
        (talismans::RANDOM_TALISMAN.deserialize)(&mut w, other, &mut r).unwrap();
        assert_eq!(r.remaining(), 0);
        assert_eq!(&a[8..], &save(&w, other)[8..]);
        let d = talismans::data(&w, id).unwrap();
        assert_eq!(d.charges, d.max_charges);
    }
}

#[test]
fn the_talisman_tables_have_matching_lengths() {
    assert_eq!(
        talismans::data::KILLERS.len(),
        talismans::data::KILLER_LABELS.len()
    );
    assert_eq!(
        talismans::data::SUMMONS.len(),
        talismans::data::SUMMON_LABELS.len()
    );
}

#[test]
fn staff_robe_hue_follows_the_access_level() {
    let mut w = crate::test_world();
    let id = suits::staff_robe_new(&mut w, &suits::STAFF_ROBE, AccessLevel::Counselor);
    assert_eq!(w.item(id).unwrap().hue, 0x3);
    suits::set_access_level(&mut w, id, AccessLevel::Administrator);
    assert_eq!(w.item(id).unwrap().hue, 0x488);
    assert!(!w.item(id).unwrap().movable());
}
