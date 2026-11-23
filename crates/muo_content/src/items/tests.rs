//! Tests of the weapon and armor bases and the six sample leaves. The save bytes are spelled out by hand
//! from the save layouts of `BaseWeapon` and `BaseArmor`, not read back from the writer under test.

use muo_data::expansion::Expansion;
use muo_io::{GenericWriter, SaveReader, SaveWriter};
use muo_server::World;
use muo_server::ids::ItemId;
use muo_server::item::ITEM_BASE;

use super::armor::{self, ArmorQuality};
use super::leaves::*;
use super::weapon::{self, BASE_WEAPON, WeaponDamageLevel, WeaponQuality};
use super::{attr, set_attr};
use crate::aos::{AosAttribute, AosWeaponAttribute, AttrKind};
use crate::craft::CraftResource;

fn hex(s: &str) -> Vec<u8> {
    let s: String = s.split_whitespace().collect();
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}

fn make(w: &mut World, t: &muo_server::item::ItemType) -> ItemId {
    (t.construct.unwrap())(w)
}

fn save(w: &World, id: ItemId) -> Vec<u8> {
    let ty = w.item(id).unwrap().ty;
    let mut out = SaveWriter::new(true);
    (ty.serialize)(w, id, &mut out);
    out.into_vec()
}

/// The `Item` part of a record, as written by the base class.
fn item_part(w: &World, id: ItemId) -> Vec<u8> {
    let mut out = SaveWriter::new(true);
    (ITEM_BASE.serialize)(w, id, &mut out);
    out.into_vec()
}

fn era(w: &mut World, e: Expansion) {
    w.core.expansion = Some(e);
}

/// A fresh item of class `ty` as the loader makes it (`new T(Serial)`).
fn for_load(w: &mut World, serial: u32, ty: &'static muo_server::item::ItemType) -> ItemId {
    w.add_item(muo_server::item::Item::new_for_load(
        muo_primitives::Serial::new(serial),
        ty,
    ))
}

#[test]
fn longsword_is_built_from_its_class() {
    let mut w = World::new();
    let id = make(&mut w, &LONGSWORD);
    let item = w.item(id).unwrap();
    assert_eq!(item.item_id, 0xF61);
    assert_eq!(item.ty.full_name, "Server.Items.Longsword");
    assert!(item.ty.is(&BASE_WEAPON) && item.ty.is(&super::weapon::BASE_SWORD));
    assert!(!item.ty.is(&super::armor::BASE_ARMOR));
    let d = weapon::data(&w, id).unwrap();
    assert!((31..=110).contains(&d.hit_points) && d.hit_points == d.max_hit_points);
    assert_eq!(
        (d.quality, d.resource),
        (WeaponQuality::Regular, CraftResource::Iron)
    );
    assert_eq!((d.str_requirement, d.min_damage, d.speed), (-1, -1, -1.0));
    assert_eq!((item.ty.default_weight)(&w, id), 7.0);
}

#[test]
fn stats_follow_the_era() {
    let mut w = World::new();
    let katana = make(&mut w, &KATANA);
    let bow = make(&mut w, &BOW);
    era(&mut w, Expansion::UOR);
    assert_eq!(
        (
            weapon::str_requirement(&w, katana),
            weapon::speed(&w, katana)
        ),
        (10, 58.0)
    );
    assert_eq!(weapon::max_damage(&w, katana), 26);
    assert_eq!(weapon::min_damage(&w, bow), 9);
    era(&mut w, Expansion::AOS);
    assert_eq!(
        (
            weapon::str_requirement(&w, katana),
            weapon::speed(&w, katana)
        ),
        (25, 46.0)
    );
    assert_eq!(weapon::min_damage(&w, bow), 16, "Core.ML ? 15 : 16");
    era(&mut w, Expansion::ML);
    assert_eq!(weapon::speed(&w, katana), 2.5);
    assert_eq!(
        (weapon::min_damage(&w, bow), weapon::max_damage(&w, bow)),
        (15, 19)
    );
    assert_eq!(weapon::hit_sound(&w, katana), 0x23B);
    assert_eq!(weapon::max_range(&w, bow), 10);
    assert_eq!(weapon::max_range(&w, katana), 1);
    // an instance value wins over the class
    weapon::data_mut(&mut w, katana).unwrap().min_damage = 99;
    assert_eq!(weapon::min_damage(&w, katana), 99);
}

#[test]
fn bow_constructor_sets_layer_and_wood() {
    let mut w = World::new();
    // Bow's template sets `"resource"`, a key muo_items registers; this crate's test world stands in for it.
    crate::templates::keys::item_key(
        &mut w.types,
        "resource",
        |_| Ok(()),
        |w, id, v| {
            let r = v
                .as_str()
                .and_then(|s| super::equip_props::RESOURCES.get(s));
            weapon::set_resource(w, id, r.unwrap_or_default());
        },
    );
    let bow = make(&mut w, &BOW);
    assert_eq!(
        w.item(bow).unwrap().layer,
        muo_server::layer::Layer::TwoHanded
    );
    assert_eq!(
        weapon::data(&w, bow).unwrap().resource,
        CraftResource::RegularWood
    );
    assert_eq!(
        weapon::animation(&w, bow),
        weapon::WeaponAnimation::ShootBow
    );
    assert_eq!(weapon::weapon_type(&w, bow), weapon::WeaponType::Ranged);
    assert_eq!(weapon::ammo_type(&w, bow), Some("Server.Items.Arrow"));
}

#[test]
fn default_longsword_record() {
    let mut w = World::new();
    let id = make(&mut w, &LONGSWORD);
    {
        let d = weapon::data_mut(&mut w, id).unwrap();
        d.hit_points = 50;
        d.max_hit_points = 60;
    }
    // BaseWeapon: version 11, saveFlags HitPoints|MaxHitPoints = 0x30 (an encoded int), the two ints;
    // BaseSword and Longsword: each its own version 0.
    let mut expected = item_part(&w, id);
    expected.extend(hex("0b000000 30 32000000 3c000000 00000000 00000000"));
    assert_eq!(save(&w, id), expected);
}

#[test]
fn weapon_record_with_everything_set() {
    let mut w = World::new();
    let id = make(&mut w, &KATANA);
    weapon::set_damage_level(&mut w, id, WeaponDamageLevel::Ruin);
    weapon::set_quality(&mut w, id, WeaponQuality::Exceptional);
    set_attr(
        &mut w,
        id,
        AttrKind::Attributes,
        AosAttribute::WeaponDamage as u32,
        35,
    );
    {
        let d = weapon::data_mut(&mut w, id).unwrap();
        d.hit_points = 50;
        d.max_hit_points = 60;
        d.crafter = Some("Bob".into());
        d.speed = 3.5;
    }
    // What the generated Serialize does, written out: flags, then the fields in SaveFlag order.
    let mut e = SaveWriter::new(true);
    e.write_i32(11);
    e.write_encoded_int(0x1 | 0x8 | 0x10 | 0x20 | 0x200 | 0x4_0000 | 0x100_0000);
    e.write_encoded_int(1); // DamageLevel.Ruin
    e.write_encoded_int(2); // Quality.Exceptional
    e.write_i32(50);
    e.write_i32(60);
    e.write_opt_string(Some("Bob"));
    e.write_f32(3.5);
    e.write_encoded_int(0); // AosAttributes version
    e.write_u32(0x800); // _names: WeaponDamage
    e.write_encoded_int(1);
    e.write_encoded_int(35);
    e.write_i32(0); // BaseSword
    e.write_i32(0); // Katana
    let mut expected = item_part(&w, id);
    expected.extend(e.into_vec());
    let got = save(&w, id);
    assert_eq!(got, expected);

    // and it loads back into a fresh instance of the class
    let mut w2 = World::new();
    let other = for_load(&mut w2, 0x4000_0009, &KATANA);
    let mut r = SaveReader::new(&got);
    (KATANA.deserialize)(&mut w2, other, &mut r).unwrap();
    assert_eq!(r.remaining(), 0, "the record is read to its end");
    let d = weapon::data(&w2, other).unwrap();
    assert_eq!(
        (d.damage_level, d.quality),
        (WeaponDamageLevel::Ruin, WeaponQuality::Exceptional)
    );
    assert_eq!((d.hit_points, d.max_hit_points, d.speed), (50, 60, 3.5));
    assert_eq!(d.crafter.as_deref(), Some("Bob"));
    assert_eq!(
        d.attributes.get(true, AosAttribute::WeaponDamage as u32),
        35
    );
    assert_eq!(d.str_requirement, -1, "absent flags read as the defaults");
    let (a, b) = (save(&w2, other), save(&w, id));
    assert_eq!(
        a[item_part(&w2, other).len()..],
        b[item_part(&w, id).len()..]
    );
}

#[test]
fn durability_bonus_rescales_hit_points() {
    let mut w = World::new();
    era(&mut w, Expansion::AOS);
    let id = make(&mut w, &KATANA);
    {
        let d = weapon::data_mut(&mut w, id).unwrap();
        d.hit_points = 100;
        d.max_hit_points = 100;
    }
    set_attr(
        &mut w,
        id,
        AttrKind::Weapon,
        AosWeaponAttribute::DurabilityBonus as u32,
        50,
    );
    let d = weapon::data(&w, id).unwrap();
    assert_eq!((d.hit_points, d.max_hit_points), (150, 150));
    assert_eq!(
        attr(
            &w,
            id,
            AttrKind::Weapon,
            AosWeaponAttribute::DurabilityBonus as u32
        ),
        50
    );
    // before AOS the bag reads as empty
    era(&mut w, Expansion::UOR);
    assert_eq!(
        attr(
            &w,
            id,
            AttrKind::Weapon,
            AosWeaponAttribute::DurabilityBonus as u32
        ),
        0
    );
}

#[test]
fn weapon_resistances_come_from_the_bag() {
    let mut w = World::new();
    era(&mut w, Expansion::AOS);
    let id = make(&mut w, &LONGSWORD);
    set_attr(
        &mut w,
        id,
        AttrKind::Weapon,
        AosWeaponAttribute::ResistFireBonus as u32,
        7,
    );
    assert_eq!((LONGSWORD.fire_resistance)(&w, id), 7);
    assert_eq!((LONGSWORD.cold_resistance)(&w, id), 0);
}

#[test]
fn plate_chest_record() {
    let mut w = World::new();
    let id = make(&mut w, &PLATE_CHEST);
    {
        let d = armor::data_mut(&mut w, id).unwrap();
        assert!((50..=65).contains(&d.hit_points));
        d.hit_points = 60;
        d.max_hit_points = 61;
    }
    // BaseArmor: version 10, flags MaxHitPoints|HitPoints = 0x300 (encoded 0x80 0x06), max then hp as encoded
    // ints; PlateChest: version 0.
    let mut expected = item_part(&w, id);
    expected.extend(hex("0a000000 8006 3d 3c 00000000"));
    assert_eq!(save(&w, id), expected);

    let mut w2 = World::new();
    let other = for_load(&mut w2, 0x4000_0002, &PLATE_CHEST);
    let bytes = save(&w, id);
    let mut r = SaveReader::new(&bytes);
    (PLATE_CHEST.deserialize)(&mut w2, other, &mut r).unwrap();
    assert_eq!(r.remaining(), 0);
    let d = armor::data(&w2, other).unwrap();
    assert_eq!((d.hit_points, d.max_hit_points, d.armor_base), (60, 61, -1));
}

#[test]
fn leather_chest_default_resource_is_not_saved() {
    let mut w = World::new();
    let id = make(&mut w, &LEATHER_CHEST);
    assert_eq!(
        armor::data(&w, id).unwrap().resource,
        CraftResource::RegularLeather
    );
    let prefix = item_part(&w, id).len();
    // flags: only the two hit point flags (0x300); no Resource bit although it is not Iron
    let bytes = save(&w, id);
    assert_eq!(&bytes[prefix..prefix + 6], hex("0a000000 8006").as_slice());
    // an Iron leather chest does save it: 0x4300 as an encoded int
    armor::data_mut(&mut w, id).unwrap().resource = CraftResource::Iron;
    let bytes = save(&w, id);
    assert_eq!(&bytes[prefix + 4..prefix + 7], hex("808601").as_slice());
    // a chest loaded without the flag gets the class's default
    let mut w2 = World::new();
    let other = for_load(&mut w2, 0x4000_0003, &LEATHER_CHEST);
    let id2 = make(&mut w2, &LEATHER_CHEST);
    let bytes = save(&w2, id2);
    let mut r = SaveReader::new(&bytes);
    (LEATHER_CHEST.deserialize)(&mut w2, other, &mut r).unwrap();
    assert_eq!(
        armor::data(&w2, other).unwrap().resource,
        CraftResource::RegularLeather
    );
}

#[test]
fn armor_numbers() {
    let mut w = World::new();
    era(&mut w, Expansion::UOR);
    let plate = make(&mut w, &PLATE_CHEST);
    let chain = make(&mut w, &CHAIN_CHEST);
    assert_eq!((PLATE_CHEST.physical_resistance)(&w, plate), 5);
    assert_eq!((CHAIN_CHEST.poison_resistance)(&w, chain), 1);
    assert_eq!(armor::str_requirement(&w, plate), 60);
    assert_eq!(armor::dex_bonus(&w, plate), -8);
    assert_eq!(armor::dex_bonus(&w, chain), -5);
    assert_eq!(armor::base_armor_rating(&w, plate), 40);
    assert_eq!(
        armor::armor_rating(&w, plate),
        40.0,
        "Regular quality, Iron, full durability"
    );
    era(&mut w, Expansion::AOS);
    assert_eq!(
        (
            armor::str_requirement(&w, plate),
            armor::dex_bonus(&w, plate)
        ),
        (95, 0)
    );
    // Dull copper: +6 physical resist, +2 armor rating from UOR on
    armor::set_resource(&mut w, plate, CraftResource::DullCopper);
    assert_eq!((PLATE_CHEST.physical_resistance)(&w, plate), 11);
    assert_eq!(armor::armor_rating(&w, plate), 42.0);
    assert_eq!(w.item(plate).unwrap().hue, 0x973);
    armor::set_quality(&mut w, plate, ArmorQuality::Exceptional);
    assert_eq!(armor::armor_rating(&w, plate), 50.0);
    // worn-down armor protects less
    {
        let d = armor::data_mut(&mut w, plate).unwrap();
        d.max_hit_points = 100;
        d.hit_points = 50;
    }
    assert_eq!(armor::armor_rating(&w, plate), 50.0 * 75.0 / 100.0);
}

#[test]
fn leaves_register() {
    let mut types = muo_server::type_registry::TypeRegistry::new();
    super::leaves::register(&mut types);
    assert!(
        types
            .find_type_by_full_name("Server.Items.Katana", false)
            .is_some()
    );
    assert!(
        types
            .find_type_by_full_name("Server.Items.PlateChest", false)
            .is_some()
    );
}
