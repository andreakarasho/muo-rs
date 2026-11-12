//! Tests of the armor and shield classes. Save bytes are the fixed layout of each class's
//! serializer.

use std::collections::HashSet;

use muo_content::aos::{AosAttribute, AosWeaponAttribute, AttrKind};
use muo_content::items::armor;
use muo_content::items::leaves::{CHAIN_CHEST, LEATHER_CHEST, PLATE_CHEST};
use muo_content::items::{attr, set_attr};
use muo_data::expansion::Expansion;
use muo_io::{SaveReader, SaveWriter};
use muo_server::World;
use muo_server::ids::ItemId;
use muo_server::item::{ITEM_BASE, Item, ItemType, LootType};

use super::leaves::{self, *};
use super::special::*;

fn all() -> Vec<(&'static ItemType, &'static [&'static str])> {
    let mut v: Vec<(&'static ItemType, &'static [&'static str])> = leaves::ALL.to_vec();
    for t in [
        &PLATE_CHEST,
        &LEATHER_CHEST,
        &CHAIN_CHEST,
        &ELVEN_GLASSES,
        &LEATHER_GLOVES,
        &LEAF_GLOVES,
        &CHAOS_SHIELD,
        &ORDER_SHIELD,
    ] {
        v.push((t, &[]));
    }
    v
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

#[test]
fn every_file_has_a_class_and_names_are_unique() {
    let mut names = HashSet::new();
    let list = all();
    for (t, aliases) in &list {
        assert!(names.insert(t.full_name), "{} twice", t.full_name);
        for a in *aliases {
            assert!(names.insert(*a), "alias {a} twice");
        }
        assert!(t.full_name.starts_with("Server.Items."));
    }
    // 190 files, less BaseArmor, BaseArmor.Migrations, BaseShield and ArmorEnums
    assert_eq!(list.len(), 186);
    let mut types = muo_server::type_registry::TypeRegistry::default();
    super::register(&mut types);
    // the three samples are registered by muo_content::items::leaves; `BaseShield` is concrete (the shields
    // derive from it) and is registered too
    assert_eq!(types.item_types().len(), 184);
}

#[test]
fn every_class_constructs_and_round_trips() {
    for era in [None, Some(Expansion::AOS), Some(Expansion::ML)] {
        for (ty, _) in all() {
            let mut w = World::new();
            w.core.expansion = era;
            let id = (ty.construct.unwrap())(&mut w);
            assert_eq!(w.item(id).unwrap().ty.full_name, ty.full_name);
            assert!(armor::is_armor(&w, id), "{}", ty.full_name);
            let d = armor::data(&w, id).unwrap();
            assert!(
                d.hit_points == d.max_hit_points,
                "{} {} {}",
                ty.full_name,
                d.hit_points,
                d.max_hit_points
            );
            let bytes = save(&w, id);

            let mut w2 = World::new();
            w2.core.expansion = era;
            let other = for_load(&mut w2, 0x4000_0001, ty);
            let mut r = SaveReader::new(&bytes);
            (ty.deserialize)(&mut w2, other, &mut r)
                .unwrap_or_else(|e| panic!("{}: {e}", ty.full_name));
            assert_eq!(r.remaining(), 0, "{} is read to its end", ty.full_name);
            let again = save(&w2, other);
            // the serial is the only difference between the two records
            assert_eq!(bytes.len(), again.len(), "{}", ty.full_name);
            assert_eq!(
                &bytes[bytes.len().min(8)..],
                &again[again.len().min(8)..],
                "{}",
                ty.full_name
            );
        }
    }
}

#[test]
fn no_field_class_saves_its_version_after_the_parent() {
    let mut w = World::new();
    // ArmorOfFortune: [SerializationGenerator(0, false)] over StuddedChest (same shape): two int versions
    let id = (ARMOR_OF_FORTUNE.construct.unwrap())(&mut w);
    let bytes = save(&w, id);
    let mut base = SaveWriter::new(true);
    (armor_base().serialize)(&w, id, &mut base);
    let mut want = base.into_vec();
    want.extend([0, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(bytes, want);

    // BrambleCoat: [SerializationGenerator(0)] over WoodlandChest: encoded int versions
    let id = (BRAMBLE_COAT.construct.unwrap())(&mut w);
    let bytes = save(&w, id);
    let mut base = SaveWriter::new(true);
    (armor_base().serialize)(&w, id, &mut base);
    let mut want = base.into_vec();
    want.extend([0, 0]);
    assert_eq!(bytes, want);
}

fn armor_base() -> &'static ItemType {
    &muo_content::items::armor::BASE_ARMOR
}

#[test]
fn constructor_effects() {
    let mut w = World::new();
    w.core.expansion = Some(Expansion::AOS);
    let id = (ARMOR_OF_FORTUNE.construct.unwrap())(&mut w);
    let item = w.item(id).unwrap();
    assert_eq!((item.item_id, item.hue), (0x13DB, 0x501));
    assert_eq!(
        attr(&w, id, AttrKind::Attributes, AosAttribute::Luck as u32),
        200
    );
    assert_eq!(
        attr(
            &w,
            id,
            AttrKind::Attributes,
            AosAttribute::LowerRegCost as u32
        ),
        40
    );
    assert_eq!(
        attr(
            &w,
            id,
            AttrKind::Armor,
            muo_content::aos::AosArmorAttribute::MageArmor as u32
        ),
        1
    );
    let d = armor::data(&w, id).unwrap();
    assert_eq!((d.hit_points, d.max_hit_points), (255, 255));

    // LeggingsOfBane: DurabilityBonus 100 rescales, then both hit point values are set to 255
    let id = (LEGGINGS_OF_BANE.construct.unwrap())(&mut w);
    let d = armor::data(&w, id).unwrap();
    assert_eq!((d.hit_points, d.max_hit_points), (255, 255));

    // ShadowDancerLeggings: ItemID = 0x13D2 and two skill bonuses
    let id = (SHADOW_DANCER_LEGGINGS.construct.unwrap())(&mut w);
    assert_eq!(w.item(id).unwrap().item_id, 0x13D2);
    let d = armor::data(&w, id).unwrap();
    assert_eq!(
        d.skill_bonuses
            .skill_bonus(true, 0)
            .map(|(s, b)| (s as i32, b)),
        Some((47, 20.0))
    );
    assert_eq!(
        d.skill_bonuses
            .skill_bonus(true, 1)
            .map(|(s, b)| (s as i32, b)),
        Some((33, 20.0))
    );

    // ValorGauntlets: Blessed, Weight 4
    let id = (VALOR_GAUNTLETS.construct.unwrap())(&mut w);
    assert_eq!(w.item(id).unwrap().loot_type, LootType::Blessed);
    assert_eq!(w.item(id).unwrap().weight, 4.0);
}

#[test]
fn stats_come_from_the_class() {
    let mut w = World::new();
    // GargishLeatherWingArmor overrides the total resistances
    let id = (GARGISH_LEATHER_WING_ARMOR.construct.unwrap())(&mut w);
    let ty = w.item(id).unwrap().ty;
    assert_eq!(
        [
            (ty.physical_resistance)(&w, id),
            (ty.fire_resistance)(&w, id),
            (ty.cold_resistance)(&w, id),
            (ty.poison_resistance)(&w, id),
            (ty.energy_resistance)(&w, id),
        ],
        [0; 5]
    );
    assert_eq!(
        (ty.required_races)(&w, id),
        muo_server::race::ALLOW_GARGOYLES_ONLY
    );

    // a bone armor of the era: LeatherCap-like numbers through the derived chain
    let id = (BRAMBLE_COAT.construct.unwrap())(&mut w);
    let ty = w.item(id).unwrap().ty;
    assert_eq!((ty.physical_resistance)(&w, id), 10);
    assert_eq!((ty.label_number)(&w, id), 1072925);

    // Buckler (shield)
    let id = (BUCKLER.construct.unwrap())(&mut w);
    assert_eq!(armor::base_armor_rating(&w, id), 7);
}

#[test]
fn trade_glasses_derive_from_elven_glasses() {
    let mut w = World::new();
    w.core.expansion = Some(Expansion::AOS);
    let id = (TRADE_GLASSES.construct.unwrap())(&mut w);
    let item = w.item(id).unwrap();
    assert_eq!(item.item_id, 0x2FB8);
    assert!(item.ty.is(&ELVEN_GLASSES));
    assert_eq!(
        attr(&w, id, AttrKind::Attributes, AosAttribute::BonusStr as u32),
        10
    );
    // [SerializationGenerator(0, false)] over ElvenGlasses: version int 0 + save flags 0, then its own version 0
    let bytes = save(&w, id);
    assert_eq!(&bytes[bytes.len() - 9..], &[0; 9]);

    // hit effects are the ElvenGlasses' own bag and are saved
    set_attr(
        &mut w,
        id,
        AttrKind::Weapon,
        AosWeaponAttribute::HitHarm as u32,
        25,
    );
    let bytes = save(&w, id);
    let mut w2 = World::new();
    w2.core.expansion = Some(Expansion::AOS);
    let other = for_load(&mut w2, 0x4000_0002, &TRADE_GLASSES);
    let mut r = SaveReader::new(&bytes);
    (TRADE_GLASSES.deserialize)(&mut w2, other, &mut r).unwrap();
    assert_eq!(r.remaining(), 0);
    assert_eq!(
        attr(
            &w2,
            other,
            AttrKind::Weapon,
            AosWeaponAttribute::HitHarm as u32
        ),
        25
    );
}

#[test]
fn arcane_gloves_charges_change_the_graphic_and_save() {
    let mut w = World::new();
    let id = (LEATHER_GLOVES.construct.unwrap())(&mut w);
    assert_eq!(w.item(id).unwrap().item_id, 0x13C6);
    assert!(!is_arcane(&w, id));
    set_max_arcane_charges(&mut w, id, 10, 0x13C6);
    set_cur_arcane_charges(&mut w, id, 7, 0x13C6);
    assert!(is_arcane(&w, id));
    assert_eq!(w.item(id).unwrap().item_id, 0x26B0);
    let bytes = save(&w, id);
    // version 2 as an int, then the two charges as encoded ints
    assert_eq!(&bytes[bytes.len() - 6..], &[2, 0, 0, 0, 7, 10]);
    set_cur_arcane_charges(&mut w, id, 0, 0x13C6);
    assert_eq!(w.item(id).unwrap().hue, 0);
    set_max_arcane_charges(&mut w, id, 0, 0x13C6);
    assert_eq!(w.item(id).unwrap().item_id, 0x13C6);
    leather_gloves_flip(&mut w, id);
    assert_eq!(w.item(id).unwrap().item_id, 0x13CE);

    // LeafGloves: encoded version 1
    let id = (LEAF_GLOVES.construct.unwrap())(&mut w);
    set_max_arcane_charges(&mut w, id, 5, 0x2FC6);
    set_cur_arcane_charges(&mut w, id, 3, 0x2FC6);
    let bytes = save(&w, id);
    assert_eq!(&bytes[bytes.len() - 3..], &[1, 3, 5]);

    // an old record: version 0 with the bool-prefixed ints
    let mut old = {
        let mut base = SaveWriter::new(true);
        let id = (LEAF_GLOVES.construct.unwrap())(&mut w);
        (BASE_ARMOR_FOR_TEST.serialize)(&w, id, &mut base);
        base.into_vec()
    };
    old.extend([0, 1, 4, 0, 0, 0, 9, 0, 0, 0]);
    let mut w2 = World::new();
    let other = for_load(&mut w2, 0x4000_0003, &LEAF_GLOVES);
    let mut r = SaveReader::new(&old);
    (LEAF_GLOVES.deserialize)(&mut w2, other, &mut r).unwrap();
    assert_eq!(armor::data(&w2, other).unwrap().arcane_charges, (4, 9));
}

use muo_content::items::armor::BASE_ARMOR as BASE_ARMOR_FOR_TEST;

#[test]
fn shields_of_a_guild_war() {
    let mut w = World::new();
    w.core.expansion = Some(Expansion::UOR);
    let id = (CHAOS_SHIELD.construct.unwrap())(&mut w);
    // before AOS they are newbied
    assert_eq!(w.item(id).unwrap().loot_type, LootType::Newbied);
    w.core.expansion = Some(Expansion::AOS);
    let id = (ORDER_SHIELD.construct.unwrap())(&mut w);
    assert_eq!(w.item(id).unwrap().loot_type, LootType::Regular);
    assert_eq!(armor::base_armor_rating(&w, id), 30);
    let _ = ITEM_BASE.full_name;
    assert!(metal_kite_dye(&mut w));
}

fn metal_kite_dye(w: &mut World) -> bool {
    let id = (METAL_KITE_SHIELD.construct.unwrap())(w);
    let ok = metal_kite_shield_dye(w, id, 0x44);
    ok && w.item(id).unwrap().hue == 0x44
}

#[test]
fn aliases_are_registered() {
    let (_, aliases) = leaves::ALL
        .iter()
        .find(|(t, _)| t.full_name == "Server.Items.GargishClothArmsType1")
        .unwrap();
    assert_eq!(
        *aliases,
        &[
            "Server.Items.GargishClothArms",
            "Server.Items.GargishClothArmsArmor"
        ]
    );
}
