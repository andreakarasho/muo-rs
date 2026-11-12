//! Tests of BaseArmor's behavior and older saves, BaseShield, BaseClothing, BaseHat and BaseJewel. Save bytes are
//! spelled out by hand from the save layouts of `BaseArmor`, `BaseClothing`, `BaseHat` and `BaseJewel`, not read
//! back from the writer under test.

use muo_data::expansion::Expansion;
use muo_io::{GenericWriter, SaveReader, SaveWriter};
use muo_primitives::Serial;
use muo_server::World;
use muo_server::ids::{EntityId, ItemId};
use muo_server::item::{ITEM_BASE, ItemType};
use muo_server::mobile::MOBILE_BASE;
use muo_server::skills::SkillName;

use super::armor::{
    self, ArmorDurabilityLevel, ArmorProtectionLevel, ArmorQuality, BASE_ARMOR, BASE_SHIELD,
};
use super::clothing::{self, ClothingQuality};
use super::equip_hooks::CraftArgs;
use super::jewel::{self, GemType};
use super::leaves::*;
use super::{attr, set_attr};
use crate::aos::{AosArmorAttribute, AosAttribute, AosElementAttribute, AttrKind, Attrs};
use crate::craft::CraftResource;

fn hex(s: &str) -> Vec<u8> {
    let s: String = s.split_whitespace().collect();
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}

fn make(w: &mut World, t: &ItemType) -> ItemId {
    (t.construct.unwrap())(w)
}

fn save(w: &World, id: ItemId) -> Vec<u8> {
    let ty = w.item(id).unwrap().ty;
    let mut out = SaveWriter::new(true);
    (ty.serialize)(w, id, &mut out);
    out.into_vec()
}

fn item_part(w: &World, id: ItemId) -> Vec<u8> {
    let mut out = SaveWriter::new(true);
    (ITEM_BASE.serialize)(w, id, &mut out);
    out.into_vec()
}

fn era(w: &mut World, e: Expansion) {
    w.core.expansion = Some(e);
}

fn for_load(w: &mut World, serial: u32, ty: &'static ItemType) -> ItemId {
    w.add_item(muo_server::item::Item::new_for_load(
        Serial::new(serial),
        ty,
    ))
}

/// Loads `body` (what follows the `Item` part) into a fresh `ty` item, whose own `Item` part is `prefix`.
fn load(w: &mut World, ty: &'static ItemType, prefix: &[u8], body: Vec<u8>) -> ItemId {
    let id = for_load(w, 0x4000_1000 + w.items.len() as u32, ty);
    let mut bytes = prefix.to_vec();
    bytes.extend(body);
    let mut r = SaveReader::new(&bytes);
    (ty.deserialize)(w, id, &mut r).unwrap();
    assert_eq!(r.remaining(), 0, "the record is read to its end");
    id
}

fn prefix_of(ty: &'static ItemType) -> Vec<u8> {
    let mut w = World::new();
    let id = make(&mut w, ty);
    item_part(&w, id)
}

/// BaseHat: version 0 and the shipwreck flag; Cap: version 0.
fn hat_tail(shipwrecked: bool) -> Vec<u8> {
    let mut e = SaveWriter::new(true);
    e.write_i32(0);
    e.write_bool(shipwrecked);
    e.write_i32(0);
    e.into_vec()
}

fn empty_bag(out: &mut SaveWriter) {
    out.write_encoded_int(0);
    out.write_u32(0);
    out.write_encoded_int(0);
}

// ---- armor ----

#[test]
fn armor_version_10_with_everything_set() {
    let mut w = World::new();
    let id = make(&mut w, &PLATE_CHEST);
    armor::set_quality(&mut w, id, ArmorQuality::Exceptional);
    armor::set_protection_level(&mut w, id, ArmorProtectionLevel::Guarding);
    armor::set_identified(&mut w, id, true);
    armor::set_crafter(&mut w, id, Some("Bob".into()));
    {
        let d = armor::data_mut(&mut w, id).unwrap();
        d.hit_points = 60;
        d.max_hit_points = 61;
        d.physical_bonus = 3;
    }
    let mut e = SaveWriter::new(true);
    e.write_i32(10);
    // PhysicalBonus 0x4, Identified 0x80, MaxHitPoints 0x100, HitPoints 0x200, Crafter 0x400, Quality 0x800,
    // ProtectionLevel 0x2000
    e.write_encoded_int(0x4 | 0x80 | 0x100 | 0x200 | 0x400 | 0x800 | 0x2000);
    e.write_encoded_int(3);
    e.write_encoded_int(61);
    e.write_encoded_int(60);
    e.write_opt_string(Some("Bob"));
    e.write_encoded_int(2);
    e.write_encoded_int(2);
    e.write_i32(0);
    let mut expected = item_part(&w, id);
    expected.extend(e.into_vec());
    let got = save(&w, id);
    assert_eq!(got, expected);

    let mut w2 = World::new();
    let other = load(
        &mut w2,
        &PLATE_CHEST,
        &item_part(&w, id),
        got[item_part(&w, id).len()..].to_vec(),
    );
    let d = armor::data(&w2, other).unwrap();
    assert_eq!(
        (
            d.quality,
            d.protection_level,
            d.identified,
            d.physical_bonus
        ),
        (
            ArmorQuality::Exceptional,
            ArmorProtectionLevel::Guarding,
            true,
            3
        )
    );
    assert_eq!(d.crafter.as_deref(), Some("Bob"));
}

#[test]
fn armor_version_9_keeps_every_field_and_player_constructed() {
    let mut w = World::new();
    let mut e = SaveWriter::new(true);
    e.write_i32(9);
    // Identified 0x80, Crafter 0x400, Durability 0x1000, ProtectionLevel 0x2000, Resource 0x4000,
    // PlayerConstructed 0x1000000
    e.write_encoded_int(0x80 | 0x400 | 0x1000 | 0x2000 | 0x4000 | 0x100_0000);
    e.write_opt_string(Some("Ann"));
    e.write_encoded_int(3);
    e.write_encoded_int(4);
    e.write_encoded_int(CraftResource::Gold as i32);
    let mut body = e.into_vec();
    body.extend(hex("00000000"));
    let id = load(&mut w, &PLATE_CHEST, &prefix_of(&PLATE_CHEST), body);
    let d = armor::data(&w, id).unwrap();
    assert_eq!(d.crafter.as_deref(), Some("Ann"));
    assert_eq!(
        (d.durability, d.protection_level, d.resource, d.identified),
        (
            ArmorDurabilityLevel::Massive,
            ArmorProtectionLevel::Fortification,
            CraftResource::Gold,
            true
        )
    );
    assert!(w.item(id).unwrap().player_constructed);
    assert_eq!(d.quality, ArmorQuality::Regular, "no flag reads Regular");
    assert_eq!(d.armor_base, -1);
}

#[test]
fn armor_version_8_forgets_the_protection_level() {
    let mut w = World::new();
    let mut e = SaveWriter::new(true);
    e.write_i32(8);
    // Crafter 0x400 (a mobile), Durability 0x1000, ProtectionLevel 0x2000
    e.write_encoded_int(0x400 | 0x1000 | 0x2000);
    e.write_serial(Serial::new(0)); // no such mobile
    e.write_encoded_int(2);
    e.write_encoded_int(3);
    e.write_i32(0);
    let id = load(&mut w, &PLATE_CHEST, &prefix_of(&PLATE_CHEST), e.into_vec());
    let d = armor::data(&w, id).unwrap();
    assert_eq!(d.durability, ArmorDurabilityLevel::Substantial);
    assert_eq!(
        d.protection_level,
        ArmorProtectionLevel::Regular,
        "MigrateFrom(V8Content) never copies ProtectionLevel"
    );
    assert_eq!(d.crafter, None);
}

#[test]
fn armor_before_codegen_leaves_quality_low() {
    let mut w = World::new();
    let mut e = SaveWriter::new(true);
    e.write_i32(5);
    // MaxHitPoints 0x100, HitPoints 0x200, PlayerConstructed 0x1000000
    e.write_encoded_int(0x100 | 0x200 | 0x100_0000);
    e.write_encoded_int(40);
    e.write_encoded_int(35);
    e.write_i32(0);
    let id = load(&mut w, &PLATE_CHEST, &prefix_of(&PLATE_CHEST), e.into_vec());
    let d = armor::data(&w, id).unwrap();
    assert_eq!((d.max_hit_points, d.hit_points), (40, 35));
    assert_eq!(
        d.quality,
        ArmorQuality::Low,
        "the old reader never sets _quality when it is not flagged, and the enum's zero is Low"
    );
    assert_eq!(d.resource, CraftResource::Iron);
    assert!(w.item(id).unwrap().player_constructed);
}

#[test]
fn armor_rating_and_stats_follow_the_expansion() {
    let mut w = World::new();
    let id = make(&mut w, &PLATE_CHEST);
    armor::data_mut(&mut w, id).unwrap().str_bonus = 5;
    set_attr(
        &mut w,
        id,
        AttrKind::Attributes,
        AosAttribute::BonusStr as u32,
        3,
    );
    era(&mut w, Expansion::AOS);
    assert_eq!(
        armor::compute_stat_bonus(&w, id, muo_server::mobile::StatType::STR),
        8
    );
    // 95 Str requirement, 20% lower
    set_attr(
        &mut w,
        id,
        AttrKind::Armor,
        AosArmorAttribute::LowerStatReq as u32,
        20,
    );
    assert_eq!(
        armor::compute_stat_req(&w, id, muo_server::mobile::StatType::STR),
        76
    );
    era(&mut w, Expansion::UOR);
    assert_eq!(
        armor::compute_stat_req(&w, id, muo_server::mobile::StatType::STR),
        60
    );
}

#[test]
fn armor_equip_puts_stat_mods_on_the_wearer_and_removal_takes_them_off() {
    let mut w = World::new();
    era(&mut w, Expansion::AOS);
    let m = w.new_mobile(&MOBILE_BASE);
    let id = make(&mut w, &PLATE_CHEST);
    armor::data_mut(&mut w, id).unwrap().dex_bonus = 4;
    assert!((PLATE_CHEST.on_equip)(&mut w, id, m));
    let serial = w.item(id).unwrap().serial;
    let names: Vec<_> = w
        .mobile(m)
        .unwrap()
        .stat_mods
        .iter()
        .map(|s| (s.name.clone().unwrap(), s.offset))
        .collect();
    assert_eq!(names, [(format!("{serial}Dex"), 4)]);
    (PLATE_CHEST.on_removed)(&mut w, id, EntityId::Mobile(m));
    assert!(w.mobile(m).unwrap().stat_mods.is_empty());
}

#[test]
fn skill_bonuses_become_skill_mods_while_worn() {
    let mut w = World::new();
    era(&mut w, Expansion::AOS);
    let m = w.new_mobile(&MOBILE_BASE);
    let id = make(&mut w, &PLATE_CHEST);
    set_attr(
        &mut w,
        id,
        AttrKind::SkillBonuses,
        1,
        Attrs::skill_bonus_value(SkillName::Swords, 7.5),
    );
    (PLATE_CHEST.on_added)(&mut w, id, EntityId::Mobile(m));
    let mods = &w.mobile(m).unwrap().skill_mods;
    assert_eq!(mods.len(), 1);
    assert_eq!(
        (mods[0].skill, mods[0].value, mods[0].obey_cap),
        (SkillName::Swords, 7.5, true)
    );
    assert!(mods[0].relative);
    // AddTo removes first: wearing it again does not stack
    (PLATE_CHEST.on_added)(&mut w, id, EntityId::Mobile(m));
    assert_eq!(w.mobile(m).unwrap().skill_mods.len(), 1);
    (PLATE_CHEST.on_removed)(&mut w, id, EntityId::Mobile(m));
    assert!(w.mobile(m).unwrap().skill_mods.is_empty());
}

#[test]
fn a_duped_armor_has_its_own_bags() {
    let mut w = World::new();
    era(&mut w, Expansion::AOS);
    let a = make(&mut w, &PLATE_CHEST);
    let b = make(&mut w, &PLATE_CHEST);
    set_attr(
        &mut w,
        a,
        AttrKind::Attributes,
        AosAttribute::Luck as u32,
        40,
    );
    armor::data_mut(&mut w, a).unwrap().hit_points = 20;
    armor::data_mut(&mut w, a).unwrap().max_hit_points = 30;
    (PLATE_CHEST.on_after_duped)(&mut w, a, b);
    assert_eq!(
        attr(&w, b, AttrKind::Attributes, AosAttribute::Luck as u32),
        40
    );
    set_attr(
        &mut w,
        b,
        AttrKind::Attributes,
        AosAttribute::Luck as u32,
        0,
    );
    assert_eq!(
        attr(&w, a, AttrKind::Attributes, AosAttribute::Luck as u32),
        40
    );
    let d = armor::data(&w, b).unwrap();
    assert_eq!((d.hit_points, d.max_hit_points), (20, 30));
}

#[test]
fn exceptional_crafted_armor_gets_fifteen_bonus_points() {
    let mut w = World::new();
    era(&mut w, Expansion::SE);
    let m = w.new_mobile(&MOBILE_BASE);
    let id = make(&mut w, &PLATE_CHEST);
    let mut a = CraftArgs::new(2, true, m, CraftResource::Iron);
    a.do_not_color = true;
    assert_eq!(armor::on_craft(&mut w, id, &a), 2);
    let d = armor::data(&w, id).unwrap();
    assert_eq!(d.quality, ArmorQuality::Exceptional);
    assert!(d.identified);
    assert_eq!(
        d.physical_bonus + d.fire_bonus + d.cold_bonus + d.poison_bonus + d.energy_bonus,
        15
    );
    assert_eq!(w.item(id).unwrap().hue, 0);

    // ML and a shield: no resist bonuses at all
    era(&mut w, Expansion::ML);
    let shield = make(&mut w, &BUCKLER);
    armor::on_craft(
        &mut w,
        shield,
        &CraftArgs::new(2, false, m, CraftResource::Iron),
    );
    let d = armor::data(&w, shield).unwrap();
    assert_eq!(
        d.physical_bonus + d.fire_bonus + d.cold_bonus + d.poison_bonus + d.energy_bonus,
        0
    );
}

#[test]
fn armor_on_hit_absorbs_and_wears() {
    let mut w = World::new();
    era(&mut w, Expansion::UOR);
    let armor_id = make(&mut w, &PLATE_CHEST);
    let sword = make(&mut w, &KATANA);
    let (mut worn, mut absorbed_total) = (false, 0);
    for _ in 0..400 {
        let before = armor::data(&w, armor_id).unwrap().hit_points;
        let left = armor::on_hit(&mut w, armor_id, sword, 50);
        // armor rating 40 at most: absorbs up to 39 of 50
        assert!((11..=50).contains(&left), "{left}");
        absorbed_total += 50 - left;
        let after = armor::data(&w, armor_id).unwrap().hit_points;
        assert!(after <= before);
        worn |= after < before;
        if w.item(armor_id).is_none_or(|i| i.deleted()) {
            break;
        }
    }
    assert!(worn && absorbed_total > 0);
}

#[test]
fn shield_rating_and_on_hit() {
    let mut w = World::new();
    era(&mut w, Expansion::UOR);
    let id = make(&mut w, &BUCKLER);
    assert!(w.item(id).unwrap().ty.is(&BASE_SHIELD));
    assert!(w.item(id).unwrap().ty.is(&BASE_ARMOR));
    // not worn: the plain rating
    assert_eq!(armor::armor_rating(&w, id), 7.0);
    // pre-AOS and not worn: no protection from the shield, the damage passes
    let sword = make(&mut w, &KATANA);
    assert_eq!(armor::on_hit(&mut w, id, sword, 12), 12);
    // AOS: the shield absorbs everything and wears
    era(&mut w, Expansion::AOS);
    for _ in 0..20 {
        assert_eq!(armor::on_hit(&mut w, id, sword, 12), 0);
    }
}

#[test]
fn leaves_register_and_construct() {
    let mut types = muo_server::type_registry::TypeRegistry::new();
    super::leaves::register(&mut types);
    for name in [
        "Server.Items.Buckler",
        "Server.Items.Cap",
        "Server.Items.FancyShirt",
        "Server.Items.GoldRing",
        "Server.Items.Necklace",
    ] {
        assert!(
            types.find_type_by_full_name(name, false).is_some(),
            "{name}"
        );
    }
    let mut w = World::new();
    let cap = make(&mut w, &CAP);
    assert_eq!(w.item(cap).unwrap().layer, muo_server::layer::Layer::Helm);
    let shirt = make(&mut w, &FANCY_SHIRT);
    assert_eq!(
        w.item(shirt).unwrap().layer,
        muo_server::layer::Layer::Shirt
    );
    let ring = make(&mut w, &GOLD_RING);
    assert_eq!(w.item(ring).unwrap().layer, muo_server::layer::Layer::Ring);
    let neck = make(&mut w, &NECKLACE);
    assert_eq!(w.item(neck).unwrap().layer, muo_server::layer::Layer::Neck);
}

// ---- clothing ----

#[test]
fn cap_default_record() {
    let mut w = World::new();
    let id = make(&mut w, &CAP);
    {
        let d = clothing::data_mut(&mut w, id).unwrap();
        assert!((20..=30).contains(&d.hit_points) && d.hit_points == d.max_hit_points);
        d.hit_points = 25;
        d.max_hit_points = 25;
    }
    // BaseClothing: version 8, flags MaxHitPoints|HitPoints = 0x60, the two encoded ints; BaseHat: version 0 and
    // IsShipwreckedItem; Cap: version 0.
    let mut expected = item_part(&w, id);
    expected.extend(hex("08000000 60 19 19 00000000 00 00000000"));
    assert_eq!(save(&w, id), expected);
}

#[test]
fn clothing_version_8_full_record_round_trips() {
    let mut w = World::new();
    let id = make(&mut w, &FANCY_SHIRT);
    era(&mut w, Expansion::AOS);
    clothing::set_quality(&mut w, id, ClothingQuality::Exceptional);
    clothing::set_crafter(&mut w, id, Some("Bob".into()));
    clothing::set_str_requirement(&mut w, id, 15);
    set_attr(
        &mut w,
        id,
        AttrKind::Element,
        AosElementAttribute::Cold as u32,
        4,
    );
    {
        let d = clothing::data_mut(&mut w, id).unwrap();
        d.hit_points = 9;
        d.max_hit_points = 10;
    }
    let mut e = SaveWriter::new(true);
    e.write_i32(8);
    // Resistances 0x10, MaxHitPoints 0x20, HitPoints 0x40, Crafter 0x80, Quality 0x100, StrRequirement 0x200.
    // The resource is the class default (None) and is not written.
    e.write_encoded_int(0x10 | 0x20 | 0x40 | 0x80 | 0x100 | 0x200);
    e.write_encoded_int(0); // AosElementAttributes version
    e.write_u32(0x4);
    e.write_encoded_int(1);
    e.write_encoded_int(4);
    e.write_encoded_int(10);
    e.write_encoded_int(9);
    e.write_opt_string(Some("Bob"));
    e.write_encoded_int(2);
    e.write_i32(15); // a plain int
    e.write_i32(0); // BaseShirt
    e.write_i32(0); // FancyShirt
    let mut expected = item_part(&w, id);
    expected.extend(e.into_vec());
    let got = save(&w, id);
    assert_eq!(got, expected);

    let mut w2 = World::new();
    let other = load(
        &mut w2,
        &FANCY_SHIRT,
        &item_part(&w, id),
        got[item_part(&w, id).len()..].to_vec(),
    );
    let d = clothing::data(&w2, other).unwrap();
    assert_eq!((d.hit_points, d.max_hit_points, d.str_req), (9, 10, 15));
    assert_eq!(d.quality, ClothingQuality::Exceptional);
    assert_eq!(d.crafter.as_deref(), Some("Bob"));
    era(&mut w2, Expansion::AOS);
    assert_eq!((FANCY_SHIRT.cold_resistance)(&w2, other), 4);
}

#[test]
fn clothing_current_layout_reads_a_missing_resource_as_none() {
    let mut w = World::new();
    let mut e = SaveWriter::new(true);
    e.write_i32(8);
    e.write_encoded_int(0x20 | 0x40);
    e.write_encoded_int(10);
    e.write_encoded_int(10);
    e.write_bytes(&hat_tail(false));
    let id = load(&mut w, &CAP, &prefix_of(&CAP), e.into_vec());
    // BaseClothing's generated reader has no `else` for the resource: it stays None even where the class
    // default is something else (a Cap's default is None too; the quirk shows on Boots).
    assert_eq!(
        clothing::data(&w, id).unwrap().resource,
        CraftResource::None
    );
    assert_eq!(clothing::data(&w, id).unwrap().str_req, -1);
}

#[test]
fn clothing_version_7_migrates() {
    let mut w = World::new();
    let mut e = SaveWriter::new(true);
    e.write_i32(7);
    // Resource 0x1, PlayerConstructed 0x80, Crafter 0x100 (string), Quality 0x200, StrRequirement 0x400
    e.write_encoded_int(0x1 | 0x80 | 0x100 | 0x200 | 0x400 | 0x20 | 0x40);
    e.write_encoded_int(CraftResource::RegularLeather as i32);
    e.write_encoded_int(30);
    e.write_encoded_int(12);
    e.write_opt_string(Some("Cy"));
    e.write_encoded_int(2);
    e.write_i32(25);
    e.write_bytes(&hat_tail(true));
    let id = load(&mut w, &CAP, &prefix_of(&CAP), e.into_vec());
    let d = clothing::data(&w, id).unwrap();
    assert_eq!(d.resource, CraftResource::RegularLeather);
    assert_eq!((d.max_hit_points, d.hit_points, d.str_req), (30, 12, 25));
    assert_eq!(d.crafter.as_deref(), Some("Cy"));
    assert!(w.item(id).unwrap().player_constructed);
    assert!(d.is_shipwrecked);
}

#[test]
fn clothing_version_6_forgets_the_hit_points() {
    let mut w = World::new();
    let mut e = SaveWriter::new(true);
    e.write_i32(6);
    // MaxHitPoints 0x20, HitPoints 0x40, Crafter 0x100 (a mobile)
    e.write_encoded_int(0x20 | 0x40 | 0x100);
    e.write_encoded_int(30);
    e.write_encoded_int(12);
    e.write_serial(Serial::new(0));
    e.write_bytes(&hat_tail(false));
    let id = load(&mut w, &CAP, &prefix_of(&CAP), e.into_vec());
    let d = clothing::data(&w, id).unwrap();
    assert_eq!(
        (d.max_hit_points, d.hit_points),
        (30, 0),
        "MigrateFrom(V6Content) never copies HitPoints"
    );
    assert_eq!(d.resource, CraftResource::None, "DefaultResource of a Cap");
}

#[test]
fn clothing_before_codegen_reads_the_str_requirement_as_an_encoded_int() {
    let mut w = World::new();
    let mut e = SaveWriter::new(true);
    e.write_i32(5);
    e.write_encoded_int(0x400 | 0x80);
    e.write_encoded_int(300); // a two byte encoded int; a plain int would be four bytes
    e.write_bytes(&hat_tail(false));
    let id = load(&mut w, &CAP, &prefix_of(&CAP), e.into_vec());
    assert_eq!(clothing::data(&w, id).unwrap().str_req, 300);
    // no hit points in the record: AfterDeserialization rolls them
    let d = clothing::data(&w, id).unwrap();
    assert!((20..=30).contains(&d.hit_points) && d.hit_points == d.max_hit_points);
}

#[test]
fn clothing_numbers_and_crafting() {
    let mut w = World::new();
    let shirt = make(&mut w, &FANCY_SHIRT);
    era(&mut w, Expansion::UOR);
    assert_eq!(clothing::str_requirement(&w, shirt), 0);
    era(&mut w, Expansion::AOS);
    assert_eq!(clothing::str_requirement(&w, shirt), 10);
    assert_eq!(
        clothing::compute_stat_req(&w, shirt, muo_server::mobile::StatType::STR),
        10
    );

    // a hat crafted with exceptional quality: 15 resist points from SE on (BaseHat.OnCraft), then BaseClothing's
    let m = w.new_mobile(&MOBILE_BASE);
    era(&mut w, Expansion::SE);
    let cap = make(&mut w, &CAP);
    let mut a = CraftArgs::new(2, true, m, CraftResource::None);
    a.res_hue = 0x44;
    clothing::on_craft(&mut w, cap, &a);
    let total: i32 = [
        AosElementAttribute::Physical,
        AosElementAttribute::Fire,
        AosElementAttribute::Cold,
        AosElementAttribute::Poison,
        AosElementAttribute::Energy,
    ]
    .iter()
    .map(|e| attr(&w, cap, AttrKind::Element, *e as u32))
    .sum();
    // a bag entry is created per increment, so the sum is 15 whatever the spread
    assert_eq!(total, 15);
    assert_eq!(
        w.item(cap).unwrap().hue,
        0x44,
        "DefaultResource None: Hue = resHue"
    );

    // a shirt crafted exceptional has no extra bonuses (only BaseHat does)
    let shirt2 = make(&mut w, &FANCY_SHIRT);
    clothing::on_craft(
        &mut w,
        shirt2,
        &CraftArgs::new(2, false, m, CraftResource::None),
    );
    assert!(clothing::data(&w, shirt2).unwrap().resistances.is_empty());
}

#[test]
fn clothing_equip_adds_stat_bonuses_and_dye_rules() {
    let mut w = World::new();
    era(&mut w, Expansion::AOS);
    let m = w.new_mobile(&MOBILE_BASE);
    let other = w.new_mobile(&MOBILE_BASE);
    let id = make(&mut w, &FANCY_SHIRT);
    set_attr(
        &mut w,
        id,
        AttrKind::Attributes,
        AosAttribute::BonusInt as u32,
        6,
    );
    (FANCY_SHIRT.on_added)(&mut w, id, EntityId::Mobile(m));
    assert_eq!(w.mobile(m).unwrap().stat_mods.len(), 1);
    (FANCY_SHIRT.on_removed)(&mut w, id, EntityId::Mobile(m));
    assert!(w.mobile(m).unwrap().stat_mods.is_empty());

    // dyeing: allowed on the ground, and on a worn piece only by the wearer
    let tub = |w: &mut World, h: i32| {
        let t = muo_server::item::ops::new_item_with_id(w, &muo_server::item::ITEM_BASE, 0xFAB);
        w.item_mut(t).unwrap().hue = h;
        t
    };
    let (t1, t2, t3) = (tub(&mut w, 0x21), tub(&mut w, 0x22), tub(&mut w, 0x23));
    assert!(clothing::dye(&mut w, id, m, t1));
    assert_eq!(w.item(id).unwrap().hue, 0x21);
    muo_server::item::ops::set_parent(&mut w, id, Some(EntityId::Mobile(m)));
    assert!(!clothing::dye(&mut w, id, other, t2));
    assert!(clothing::dye(&mut w, id, m, t3));
}

#[test]
fn clothing_on_hit_wears_it_down() {
    let mut w = World::new();
    era(&mut w, Expansion::UOR);
    let id = make(&mut w, &CAP);
    let sword = make(&mut w, &KATANA);
    for _ in 0..300 {
        let left = clothing::on_hit(&mut w, id, sword, 50);
        assert!((1..=4).contains(&left));
        if w.item(id).is_none_or(|i| i.deleted()) {
            return;
        }
    }
    let d = clothing::data(&w, id).unwrap();
    assert!(d.hit_points <= d.max_hit_points);
}

// ---- jewels ----

#[test]
fn gold_ring_default_record() {
    let mut w = World::new();
    let id = make(&mut w, &GOLD_RING);
    // BaseJewel: version 5, max and current hit points (0: the ring has none), resource Iron, gem type None, three
    // empty bags, gem count; BaseRing and GoldRing: version 0.
    let mut expected = item_part(&w, id);
    expected.extend(hex(
        "05000000 00 00 01 00  00 00000000 00  00 00000000 00  00 00000000 00  00  00000000 00000000",
    ));
    assert_eq!(save(&w, id), expected);
    assert_eq!(w.item(id).unwrap().layer, muo_server::layer::Layer::Ring);
}

#[test]
fn jewel_gem_names_and_crafting() {
    let mut w = World::new();
    let m = w.new_mobile(&MOBILE_BASE);
    let id = make(&mut w, &GOLD_RING);
    assert_eq!(
        (GOLD_RING.label_number)(&w, id),
        muo_server::item::base::label_number(&w, id)
    );
    jewel::set_gem_type(&mut w, id, GemType::Ruby);
    assert_eq!((GOLD_RING.label_number)(&w, id), 1044176 + 4 - 1);
    let neck = make(&mut w, &NECKLACE);
    jewel::set_gem_type(&mut w, neck, GemType::Diamond);
    assert_eq!((NECKLACE.label_number)(&w, neck), 1044241 + 9 - 1);

    let mut a = CraftArgs::new(0, false, m, CraftResource::Gold);
    a.second_resource = Some(GemType::Emerald);
    assert_eq!(jewel::on_craft(&mut w, id, &a), 1);
    let d = jewel::data(&w, id).unwrap();
    assert_eq!(
        (d.resource, d.gem_type),
        (CraftResource::Gold, GemType::Emerald)
    );
    assert_eq!(w.item(id).unwrap().hue, 0x8A5);
    // T2A gem stacks: nothing can consume them without the backpack hook, so a plain piece comes out
    let mut b = CraftArgs::new(0, false, m, CraftResource::Iron);
    b.pending_gem = Some((GemType::Diamond, 1000));
    jewel::on_craft(&mut w, id, &b);
    assert_eq!(jewel::data(&w, id).unwrap().gem_count, 0);
}

#[test]
fn jewel_version_4_and_older_load_without_a_gem_count() {
    let mut w = World::new();
    for version in [4, 3] {
        let mut e = SaveWriter::new(true);
        e.write_i32(version);
        e.write_encoded_int(0); // MaxHitPoints
        e.write_encoded_int(0);
        e.write_encoded_int(CraftResource::Gold as i32);
        e.write_encoded_int(GemType::Amber as i32);
        let mut bag = SaveWriter::new(true);
        // attributes with one value: Luck
        bag.write_encoded_int(0);
        bag.write_u32(AosAttribute::Luck as u32);
        bag.write_encoded_int(1);
        bag.write_encoded_int(25);
        e.write_bytes(&bag.into_vec());
        empty_bag(&mut e);
        empty_bag(&mut e);
        e.write_i32(0); // BaseRing
        e.write_i32(0); // GoldRing
        let id = load(&mut w, &GOLD_RING, &prefix_of(&GOLD_RING), e.into_vec());
        let d = jewel::data(&w, id).unwrap();
        assert_eq!(
            (d.resource, d.gem_type, d.gem_count),
            (CraftResource::Gold, GemType::Amber, 0)
        );
        era(&mut w, Expansion::AOS);
        assert_eq!(
            attr(&w, id, AttrKind::Attributes, AosAttribute::Luck as u32),
            25
        );
        era(&mut w, Expansion::UOR);
    }
}

#[test]
fn jewel_resistances_and_equip() {
    let mut w = World::new();
    era(&mut w, Expansion::AOS);
    let m = w.new_mobile(&MOBILE_BASE);
    let id = make(&mut w, &GOLD_RING);
    set_attr(
        &mut w,
        id,
        AttrKind::Element,
        AosElementAttribute::Fire as u32,
        9,
    );
    assert_eq!((GOLD_RING.fire_resistance)(&w, id), 9);
    assert_eq!((GOLD_RING.cold_resistance)(&w, id), 0);
    set_attr(
        &mut w,
        id,
        AttrKind::Attributes,
        AosAttribute::BonusDex as u32,
        5,
    );
    (GOLD_RING.on_added)(&mut w, id, EntityId::Mobile(m));
    assert_eq!(w.mobile(m).unwrap().stat_mods.len(), 1);
    (GOLD_RING.on_removed)(&mut w, id, EntityId::Mobile(m));
    assert!(w.mobile(m).unwrap().stat_mods.is_empty());
    // before AOS a worn jewel does nothing
    era(&mut w, Expansion::UOR);
    (GOLD_RING.on_added)(&mut w, id, EntityId::Mobile(m));
    assert!(w.mobile(m).unwrap().stat_mods.is_empty());
}
