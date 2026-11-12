//! Tests of the weapon combat half, the migrations and the abilities. The old-version records are written field by
//! field in the order of the version 10 layout (`V10Content`) and of the older migrations.

use muo_data::expansion::Expansion;
use muo_data::skill_info::SkillInfo;
use muo_io::{GenericWriter, SaveReader, SaveWriter};
use muo_server::World;
use muo_server::ids::{ItemId, MobileId};
use muo_server::item::ITEM_BASE;
use muo_server::mobile::MOBILE_BASE;
use muo_server::skills::{self, SkillName, SkillTable};

use super::leaves::*;
use super::slayer::{self, SlayerName};
use super::weapon::{self, Ability, WeaponDamageLevel, WeaponQuality};
use super::weapon_ability as wa;
use super::weapon_combat as wc;
use crate::aos::{AosAttribute, AosElementAttribute, AttrKind};
use crate::craft::CraftResource;

fn make(w: &mut World, t: &muo_server::item::ItemType) -> ItemId {
    (t.construct.unwrap())(w)
}

fn era(w: &mut World, e: Expansion) {
    w.core.expansion = Some(e);
}

fn skill_table(w: &mut World) {
    let info = |i: i32| SkillInfo {
        skill_id: i,
        name: format!("S{i}"),
        title: String::new(),
        str_scale: 0.0,
        dex_scale: 0.0,
        int_scale: 0.0,
        stat_total: 0.0,
        str_gain: 0.0,
        dex_gain: 0.0,
        int_gain: 0.0,
        gain_factor: 0.0,
        profession_skill_name: String::new(),
        primary_stat: muo_data::skill_info::Stat::Str,
        secondary_stat: muo_data::skill_info::Stat::Dex,
    };
    w.resources.get_mut::<SkillTable>().info = (0..58).map(info).collect();
}

fn mobile(w: &mut World) -> MobileId {
    skill_table(w);
    w.new_mobile(&MOBILE_BASE)
}

/// A record the way the loader hands it over: a fresh weapon of class `ty` and the bytes after the `Item` part.
fn load(
    w: &mut World,
    ty: &'static muo_server::item::ItemType,
    body: impl FnOnce(&mut SaveWriter),
) -> ItemId {
    let id = w.add_item(muo_server::item::Item::new_for_load(
        muo_primitives::Serial::new(0x4000_0001),
        ty,
    ));
    let donor = make(w, &KATANA);
    let mut out = SaveWriter::new(true);
    (ITEM_BASE.serialize)(w, donor, &mut out);
    body(&mut out);
    out.write_i32(0); // BaseSword (and Katana) version
    out.write_i32(0);
    let bytes = out.into_vec();
    let mut r = SaveReader::new(&bytes);
    (ty.deserialize)(w, id, &mut r).unwrap_or_else(|e| panic!("load failed: {e}"));
    id
}

#[test]
fn version_10_record_loads_with_player_constructed_on_the_item() {
    let mut w = World::new();
    let id = load(&mut w, &KATANA, |o| {
        o.write_i32(10);
        // V10SaveFlag: DamageLevel | Quality | HitPoints | MaxHitPoints | Crafter | Identified | Speed |
        // PlayerConstructed | EngravedText
        o.write_enum(
            4,
            0x1 | 0x8 | 0x10 | 0x20 | 0x200 | 0x400 | 0x4_0000 | 0x0400_0000 | 0x4000_0000,
        );
        o.write_enum(4, 2); // Might
        o.write_enum(4, 2); // Exceptional
        o.write_i32(40);
        o.write_i32(50);
        o.write_opt_string(Some("Bob"));
        o.write_f32(3.25);
        o.write_opt_string(Some("engraved"));
    });
    let d = weapon::data(&w, id).unwrap();
    assert_eq!(d.damage_level, WeaponDamageLevel::Might);
    assert_eq!(d.quality, WeaponQuality::Exceptional);
    assert_eq!((d.hit_points, d.max_hit_points), (40, 50));
    assert_eq!(d.crafter.as_deref(), Some("Bob"));
    assert!(d.identified);
    assert_eq!(d.speed, 3.25);
    assert_eq!(d.engraved_text.as_deref(), Some("engraved"));
    assert_eq!(d.str_requirement, -1, "absent flags read as the defaults");
    assert_eq!(d.resource, CraftResource::Iron);
    assert!(w.item(id).unwrap().player_constructed);
}

#[test]
fn version_10_record_with_nothing_set_gets_random_hit_points() {
    let mut w = World::new();
    let id = load(&mut w, &KATANA, |o| {
        o.write_i32(10);
        o.write_enum(4, 0);
    });
    let d = weapon::data(&w, id).unwrap();
    assert!(
        (31..=90).contains(&d.hit_points) && d.hit_points == d.max_hit_points,
        "AfterDeserialization"
    );
    assert_eq!(d.quality, WeaponQuality::Regular);
    assert!(!w.item(id).unwrap().player_constructed);
}

#[test]
fn legacy_record_field_by_field() {
    let mut w = World::new();
    let id = load(&mut w, &KATANA, |o| {
        o.write_i32(8);
        o.write_i32(0x8 | 0x10 | 0x20 | 0x40 | 0x400 | 0x4_0000 | 0x10_0000);
        o.write_i32(2); // Quality
        o.write_i32(7); // Hits
        o.write_i32(9); // MaxHits
        o.write_i32(SlayerName::Silver as i32);
        // Identified: version >= 6, nothing to read
        o.write_i32(5); // Speed as an int below version 9
        o.write_i32(SkillName::Fencing as i32);
    });
    let d = weapon::data(&w, id).unwrap();
    assert_eq!(d.quality, WeaponQuality::Exceptional);
    assert_eq!((d.hit_points, d.max_hit_points), (7, 9));
    assert_eq!(d.slayer, SlayerName::Silver as i32);
    assert!(d.identified);
    assert_eq!(d.speed, 5.0);
    assert_eq!(d.skill, SkillName::Fencing as i32);
}

#[test]
fn legacy_version_9_reads_a_float_speed_and_old_identified_flag_reads_a_bool() {
    let mut w = World::new();
    let id = load(&mut w, &KATANA, |o| {
        o.write_i32(9);
        o.write_i32(0x10 | 0x20 | 0x4_0000);
        o.write_i32(3);
        o.write_i32(3);
        o.write_f32(2.5);
    });
    assert_eq!(weapon::data(&w, id).unwrap().speed, 2.5);

    let id = load(&mut w, &KATANA, |o| {
        o.write_i32(5);
        o.write_i32(0x10 | 0x20 | 0x400);
        o.write_i32(3);
        o.write_i32(3);
        o.write_bool(false); // version < 6: the bool is read
    });
    assert!(!weapon::data(&w, id).unwrap().identified);
}

#[test]
fn legacy_attribute_bags_and_player_constructed() {
    let mut w = World::new();
    let id = load(&mut w, &KATANA, |o| {
        o.write_i32(7);
        // Attributes | PlayerConstructed | ElementalDamages
        o.write_i32(0x100_0000 | 0x400_0000 | 0x2000_0000);
        // BaseAttributes: version 0, names (uint), one value
        o.write_encoded_int(0);
        o.write_u32(AosAttribute::WeaponDamage as u32);
        o.write_encoded_int(1);
        o.write_encoded_int(35);
        o.write_encoded_int(0);
        o.write_u32(AosElementAttribute::Fire as u32);
        o.write_encoded_int(1);
        o.write_encoded_int(100);
    });
    w.core.expansion = Some(Expansion::AOS);
    assert_eq!(
        super::attr(
            &w,
            id,
            AttrKind::Attributes,
            AosAttribute::WeaponDamage as u32
        ),
        35
    );
    assert_eq!(
        super::attr(&w, id, AttrKind::Element, AosElementAttribute::Fire as u32),
        100
    );
    assert!(w.item(id).unwrap().player_constructed);
}

#[test]
fn migrated_record_saves_as_version_11() {
    let mut w = World::new();
    let id = load(&mut w, &KATANA, |o| {
        o.write_i32(10);
        o.write_enum(4, 0x10 | 0x20);
        o.write_i32(12);
        o.write_i32(20);
    });
    let ty = w.item(id).unwrap().ty;
    let mut out = SaveWriter::new(true);
    (ty.serialize)(&w, id, &mut out);
    let bytes = out.into_vec();
    let mut donor_out = SaveWriter::new(true);
    (ITEM_BASE.serialize)(&w, id, &mut donor_out);
    let item_len = donor_out.into_vec().len();
    assert_eq!(
        &bytes[item_len..item_len + 4],
        &[11, 0, 0, 0],
        "BaseWeapon is saved as version 11"
    );
}

#[test]
fn swing_delay_by_era() {
    let mut w = World::new();
    let katana = make(&mut w, &KATANA);
    let m = mobile(&mut w);
    w.mobile_mut(m).unwrap().stam = 100;

    era(&mut w, Expansion::ML);
    // ticks = floor((2.5 * 4 - 100 / 30) * 100 / 100) = 7 -> 1.75 s
    assert_eq!(
        wc::get_delay(&mut w, katana, m).total_milliseconds(),
        1750.0
    );
    era(&mut w, Expansion::SE);
    // speed 46: ticks = floor(80000 / ((100 + 100) * 46) - 2) = 6 -> 1.5 s
    assert_eq!(
        wc::get_delay(&mut w, katana, m).total_milliseconds(),
        1500.0
    );
    era(&mut w, Expansion::AOS);
    // v = 200 * 46, floor(40000 / 9200) * 0.5 = 2.0
    assert_eq!(
        wc::get_delay(&mut w, katana, m).total_milliseconds(),
        2000.0
    );
    era(&mut w, Expansion::UOR);
    // 15000 / (200 * 58)
    let ms = wc::get_delay(&mut w, katana, m).total_milliseconds();
    assert!((ms - 15000.0 / 11600.0 * 1000.0).abs() < 1.0, "{ms}");
    // speed 0 never swings
    weapon::data_mut(&mut w, katana).unwrap().speed = 0.0;
    assert_eq!(
        wc::get_delay(&mut w, katana, m).total_milliseconds(),
        3_600_000.0
    );
}

#[test]
fn damage_types_split_the_physical_share() {
    let mut w = World::new();
    era(&mut w, Expansion::ML);
    let katana = make(&mut w, &KATANA);
    let t = wc::get_damage_types(&w, katana, None);
    assert_eq!((t.phys, t.fire, t.cold), (100, 0, 0));

    super::set_attr(
        &mut w,
        katana,
        AttrKind::Element,
        AosElementAttribute::Fire as u32,
        30,
    );
    weapon::set_resource(&mut w, katana, CraftResource::Valorite);
    let t = wc::get_damage_types(&w, katana, None);
    // Valorite: fire 10, cold 20, poison 10, energy 20 of the 70 left after the 30 fire from the bag; applied in
    // the order cold, energy, fire, poison.
    assert_eq!(
        (t.cold, t.nrgy, t.fire, t.pois, t.phys),
        (20, 20, 40, 10, 10)
    );
    assert_eq!(
        t.phys + t.fire + t.cold + t.pois + t.nrgy + t.chaos + t.direct,
        100
    );
}

#[test]
fn damage_scales_with_durability_and_hit_chance_bonus_with_accuracy() {
    let mut w = World::new();
    let katana = make(&mut w, &KATANA);
    {
        let d = weapon::data_mut(&mut w, katana).unwrap();
        d.hit_points = 25;
        d.max_hit_points = 50;
    }
    // scale = 50 + 50 * 25 / 50 = 75
    assert_eq!(wc::scale_damage_by_durability(&w, katana, 40), 30);
    era(&mut w, Expansion::AOS);
    weapon::set_accuracy_level(&mut w, katana, weapon::WeaponAccuracyLevel::Eminently);
    assert_eq!(wc::hit_chance_bonus(&w, katana), 6);
    era(&mut w, Expansion::UOR);
    assert_eq!(wc::hit_chance_bonus(&w, katana), 0);
}

#[test]
fn slayer_groups_and_titles() {
    let mut w = World::new();
    era(&mut w, Expansion::AOS);
    let (silver, _) = slayer::entry_by_name(&w, SlayerName::Silver).unwrap();
    assert_eq!(slayer::title(&w, silver), 1060479);
    era(&mut w, Expansion::UOR);
    assert_eq!(slayer::title(&w, silver), 1017384);
    assert!(
        slayer::entry_by_name(&w, SlayerName::DaemonDismissal).is_some(),
        "pre-AOS abyss group has it"
    );
    era(&mut w, Expansion::AOS);
    assert!(
        slayer::entry_by_name(&w, SlayerName::DaemonDismissal).is_none(),
        "moved up to the super slayer"
    );
    let (ogre, group) = slayer::entry_by_name(&w, SlayerName::OgreTrashing).unwrap();
    assert_eq!(
        ogre.types,
        &[
            "Server.Mobiles.Ogre",
            "Server.Mobiles.OgreLord",
            "Server.Mobiles.ArcticOgreLord"
        ]
    );
    // humanoids oppose the undead
    assert!(!slayer::opposition_super_slays(
        &w,
        group,
        MobileId(muo_primitives::Serial::new(1))
    ));
}

#[test]
fn ability_table_matches_the_index_order() {
    assert_eq!(wa::ABILITIES[1], Some(Ability::ArmorIgnore));
    assert_eq!(wa::ABILITIES[6], Some(Ability::Dismount));
    assert_eq!(wa::ABILITIES[31], Some(Ability::MysticArc));
    for (i, a) in wa::ABILITIES.iter().enumerate().skip(1) {
        assert_eq!(a.map(|a| a as usize), Some(i));
    }
}

#[test]
fn ability_mana_drops_with_skill_and_doubles_on_cooldown() {
    let mut w = World::new();
    era(&mut w, Expansion::ML);
    let m = mobile(&mut w);
    assert_eq!(wa::calculate_mana(&mut w, Ability::ArmorIgnore, m), 30);
    skills::set_base(&mut w, m, SkillName::Swords, 100.0);
    skills::set_base(&mut w, m, SkillName::Tactics, 100.0);
    skills::set_base(&mut w, m, SkillName::Fencing, 100.0);
    // 200 total skill: -5
    assert_eq!(wa::calculate_mana(&mut w, Ability::ArmorIgnore, m), 25);
    {
        let mob = w.mobile_mut(m).unwrap();
        mob.int = 100;
        mob.mana = 100;
    }
    assert!(wa::check_mana(&mut w, Ability::ArmorIgnore, m, true));
    assert_eq!(w.mobile(m).unwrap().mana, 75);
    // a second special move within 3 seconds costs double
    assert_eq!(wa::calculate_mana(&mut w, Ability::ArmorIgnore, m), 50);
}

#[test]
fn abilities_are_not_current_before_aos() {
    let mut w = World::new();
    era(&mut w, Expansion::UOR);
    let m = mobile(&mut w);
    assert!(!wa::set_current_ability(
        &mut w,
        m,
        Some(Ability::ArmorIgnore)
    ));
    assert_eq!(wa::current_ability(&mut w, m), None);
}

#[test]
fn hit_lower_effects_expire_once() {
    let mut w = World::new();
    let m = mobile(&mut w);
    assert!(wa::hit_lower_apply_attack(&mut w, m));
    assert!(!wa::hit_lower_apply_attack(&mut w, m));
    assert!(wa::hit_lower_is_under_attack_effect(&w, m));
    assert!(!wa::hit_lower_is_under_defense_effect(&w, m));
}

#[test]
fn properties_list_the_weapon_lines() {
    let mut w = World::new();
    era(&mut w, Expansion::ML);
    let katana = make(&mut w, &KATANA);
    super::set_attr(
        &mut w,
        katana,
        AttrKind::Attributes,
        AosAttribute::WeaponDamage as u32,
        35,
    );
    let mut list = muo_server::property_list::PropertyList::new(muo_primitives::Serial::new(1));
    (w.item(katana).unwrap().ty.get_properties)(&w, katana, &mut list);
    list.terminate();
    let buf = list.buffer().to_vec();
    let has = |n: i32| buf.windows(4).any(|x| x == n.to_be_bytes());
    assert!(has(1060401), "damage increase");
    assert!(has(1061168), "weapon damage");
    assert!(has(1061167), "weapon speed");
    assert!(has(1061824), "one-handed weapon");
    assert!(has(1061172), "skill required: swordsmanship");
    assert!(has(1060639), "durability");
    assert!(has(1060403), "physical damage 100%");
}

#[test]
fn thrown_range_scales_with_the_wielder() {
    // BaseThrown.DefMaxRange without a wielder is MaxThrowRange.
    let mut w = World::new();
    let bow = make(&mut w, &BOW);
    assert_eq!(weapon::max_range(&w, bow), 10);
}

fn fighters(w: &mut World) -> (MobileId, MobileId) {
    let a = mobile(w);
    let d = w.new_mobile(&MOBILE_BASE);
    for m in [a, d] {
        let mob = w.mobile_mut(m).unwrap();
        mob.str = 100;
        mob.dex = 100;
        mob.int = 100;
        mob.hits = 100;
        mob.stam = 100;
        mob.mana = 100;
    }
    (a, d)
}

#[test]
fn a_hit_hurts_in_every_era() {
    for e in [Expansion::UOR, Expansion::AOS, Expansion::ML] {
        let mut w = World::new();
        era(&mut w, e);
        let katana = make(&mut w, &KATANA);
        let (a, d) = fighters(&mut w);
        let before = w.mobile(d).unwrap().hits;
        wc::on_hit(&mut w, katana, a, d, 1.0);
        assert!(
            w.mobile(d).is_none_or(|m| m.hits < before),
            "{e:?}: the defender lost hit points"
        );
    }
}

#[test]
fn old_damage_formula_halves_for_a_non_player_attacker() {
    let mut w = World::new();
    era(&mut w, Expansion::UOR);
    let katana = make(&mut w, &KATANA);
    let (a, d) = fighters(&mut w);
    // 5-26 base (old katana), tactics 0 halves: damage = base * 0.5 * (1 + str/500) -> halved again for the NPC
    for _ in 0..50 {
        let dmg = wc::compute_damage(&mut w, katana, a, d);
        assert!((0..=20).contains(&dmg), "{dmg}");
    }
}

#[test]
fn anatomy_and_strength_raise_aos_damage() {
    let mut w = World::new();
    era(&mut w, Expansion::AOS);
    let katana = make(&mut w, &KATANA);
    let (a, _) = fighters(&mut w);
    let low = wc::scale_damage_aos(&mut w, katana, a, 10.0, false);
    // strength 100: 100 * 0.3 + 5 = 35 -> 0.35; no skills
    assert!((low - 13.5).abs() < 1e-9, "{low}");
    skills::set_base(&mut w, a, SkillName::Tactics, 100.0);
    let high = wc::scale_damage_aos(&mut w, katana, a, 10.0, false);
    // tactics 100: 100 * 0.625 + 6.25 = 68.75 -> +0.6875
    assert!((high - 20.375).abs() < 1e-9, "{high}");
}

#[test]
fn parry_without_a_shield_or_skill_never_succeeds_before_the_roll() {
    let mut w = World::new();
    era(&mut w, Expansion::AOS);
    let (_, d) = fighters(&mut w);
    // no parry skill: aosChance = 0 > RandomDouble() is false
    for _ in 0..20 {
        assert!(!wc::check_parry(&mut w, d));
    }
}

#[test]
fn bleed_ticks_five_times_then_ends() {
    let mut w = World::new();
    era(&mut w, Expansion::AOS);
    let (a, d) = fighters(&mut w);
    wa::begin_bleed(&mut w, d, a);
    assert!(wa::is_bleeding(&w, d));
    wa::end_bleed(&mut w, d, false);
    assert!(!wa::is_bleeding(&w, d));
}

// ---- throwing --------------------------------------------------------------------------------------------------

mod throwing {
    use super::*;
    use muo_primitives::{Point3D, TimeSpan};
    use muo_server::layer::Layer;
    use muo_server::mobile::StatType;
    use muo_server::mods::StatMod;

    /// `TestThrown`: StrReq 60, throw band 4..7.
    static TEST_THROWN_CLASS: weapon::WeaponClass = weapon::WeaponClass {
        aos_str_req: 60,
        min_throw_range: 4,
        ..weapon::BASE_THROWN_CLASS
    };
    static TEST_THROWN: muo_server::item::ItemType = muo_server::item::ItemType {
        full_name: "Server.Tests.TestThrown",
        parent: Some(&weapon::BASE_THROWN),
        construct: Some(|w| {
            let id = weapon::new_weapon(w, &TEST_THROWN, 0x8FF);
            // Weapons take their layer from tiledata, which the tests don't load
            (TEST_THROWN.set_layer)(w, id, Layer::OneHanded);
            id
        }),
        family: Some(&TEST_THROWN_CLASS),
        ..weapon::BASE_THROWN
    };
    static BUCKLER: muo_server::item::ItemType = muo_server::item::ItemType {
        full_name: "Server.Items.Buckler",
        parent: Some(&super::super::armor::BASE_SHIELD),
        construct: Some(|w| {
            let id = super::super::armor::new_armor(w, &BUCKLER, 0x1B73);
            (BUCKLER.set_layer)(w, id, Layer::TwoHanded);
            id
        }),
        ..super::super::armor::BASE_SHIELD
    };

    fn at(w: &mut World, x: i32) -> MobileId {
        let m = mobile(w);
        w.mobile_mut(m).unwrap().location = Point3D::new(x, 500, 0);
        m
    }

    fn hit_chance(w: &mut World, weapon_id: ItemId, a: MobileId, d: MobileId, chance: f64) -> f64 {
        let f = weapon::class(w, weapon_id).modify_hit_chance;
        f(w, weapon_id, a, d, chance)
    }

    fn close(a: f64, b: f64) {
        assert!((a - b).abs() < 1e-10, "{a} != {b}");
    }

    #[test]
    fn modify_hit_chance_optimal_range_no_shield_no_change() {
        let mut w = World::new();
        let (attacker, defender) = (at(&mut w, 5000), at(&mut w, 5005)); // distance 5, within range 4..7
        let weapon_id = make(&mut w, &TEST_THROWN);
        skills::set_base_fixed(&mut w, attacker, SkillName::Throwing, 0);
        w.mobile_mut(attacker).unwrap().dex = 10;
        close(hit_chance(&mut w, weapon_id, attacker, defender, 0.8), 0.8);
    }

    #[test]
    fn modify_hit_chance_close_quarters_minimum_skill_applies_near_full_penalty() {
        let mut w = World::new();
        let (attacker, defender) = (at(&mut w, 5100), at(&mut w, 5101)); // distance 1
        let weapon_id = make(&mut w, &TEST_THROWN);
        skills::set_base_fixed(&mut w, attacker, SkillName::Throwing, 0);
        w.mobile_mut(attacker).unwrap().dex = 1; // minimum
        close(
            hit_chance(&mut w, weapon_id, attacker, defender, 0.8),
            0.6805,
        );
    }

    #[test]
    fn modify_hit_chance_close_quarters_max_skill_and_dex_no_change() {
        let mut w = World::new();
        let (attacker, defender) = (at(&mut w, 5200), at(&mut w, 5200)); // same tile
        let weapon_id = make(&mut w, &TEST_THROWN);
        skills::set_base_fixed(&mut w, attacker, SkillName::Throwing, 1200);
        w.mobile_mut(attacker).unwrap().dex = 120;
        close(hit_chance(&mut w, weapon_id, attacker, defender, 0.8), 0.8);
    }

    #[test]
    fn modify_hit_chance_below_min_range_applies_flat_penalty() {
        let mut w = World::new();
        let (attacker, defender) = (at(&mut w, 5300), at(&mut w, 5302)); // distance 2
        let weapon_id = make(&mut w, &TEST_THROWN);
        close(hit_chance(&mut w, weapon_id, attacker, defender, 0.8), 0.68);
    }

    fn shield_case(parry_fixed: i32, expected: f64) {
        let mut w = World::new();
        let (attacker, defender) = (at(&mut w, 5400), at(&mut w, 5405)); // distance 5, optimal
        let weapon_id = make(&mut w, &TEST_THROWN);
        let shield = make(&mut w, &BUCKLER);
        skills::set_base_fixed(&mut w, attacker, SkillName::Parry, parry_fixed);
        muo_server::mobile::part2::items::add_item(&mut w, attacker, shield);
        close(
            hit_chance(&mut w, weapon_id, attacker, defender, 0.8),
            expected,
        );
    }

    #[test]
    fn modify_hit_chance_shield_100_parry_applies_correct_penalty() {
        shield_case(1000, 0.704);
    }

    #[test]
    fn modify_hit_chance_shield_120_parry_applies_correct_penalty() {
        shield_case(1200, 0.72);
    }

    #[test]
    fn modify_hit_chance_shield_zero_parry_caps_at_90_percent() {
        shield_case(0, 0.08);
    }

    /// Wields a `TestThrown` with `raw_str` Str and returns (wielder, weapon).
    fn wielded(w: &mut World, x: i32, raw_str: i32) -> (MobileId, ItemId) {
        let attacker = at(w, x);
        let weapon_id = make(w, &TEST_THROWN);
        w.mobile_mut(attacker).unwrap().str = raw_str;
        muo_server::mobile::part2::items::add_item(w, attacker, weapon_id);
        assert_eq!(
            w.item(weapon_id).unwrap().parent,
            Some(muo_server::ids::EntityId::Mobile(attacker))
        );
        (attacker, weapon_id)
    }

    #[test]
    fn def_max_range_at_str_req_equals_min_throw_range() {
        let mut w = World::new();
        let (_, wp) = wielded(&mut w, 5800, 60);
        assert_eq!(weapon::max_range(&w, wp), 4);
    }

    #[test]
    fn def_max_range_at_140_str_equals_max_throw_range() {
        let mut w = World::new();
        let (_, wp) = wielded(&mut w, 5810, 140);
        assert_eq!(weapon::max_range(&w, wp), 7);
    }

    #[test]
    fn def_max_range_above_max_str_is_capped_at_max_throw_range() {
        let mut w = World::new();
        let (_, wp) = wielded(&mut w, 5820, 200); // uncapped formula would give 9
        assert_eq!(weapon::max_range(&w, wp), 7);
    }

    #[test]
    fn def_max_range_below_str_req_floored_at_min_throw_range() {
        let mut w = World::new();
        let (_, wp) = wielded(&mut w, 5830, 10); // below StrReq 60
        assert_eq!(weapon::max_range(&w, wp), 4);
    }

    fn damage(w: &mut World, wp: ItemId, a: MobileId, d: MobileId, dmg: i32) -> i32 {
        let f = weapon::class(w, wp).modify_damage;
        f(w, wp, a, d, dmg)
    }

    #[test]
    fn modify_damage_at_max_range_reduces_47_percent() {
        let mut w = World::new();
        let (attacker, wp) = wielded(&mut w, 5900, 140); // MaxRange == MaxThrowRange == 7
        let defender = at(&mut w, 5907); // distance 7 == MaxRange
        assert_eq!(damage(&mut w, wp, attacker, defender, 100), 53);
    }

    #[test]
    fn modify_damage_within_range_no_change() {
        let mut w = World::new();
        let (attacker, wp) = wielded(&mut w, 5920, 140);
        let defender = at(&mut w, 5925); // distance 5 < 7
        assert_eq!(damage(&mut w, wp, attacker, defender, 100), 100);
    }

    #[test]
    fn modify_hit_chance_close_quarters_uses_raw_dex_ignores_stat_mods() {
        let mut w = World::new();
        let (attacker, defender) = (at(&mut w, 5940), at(&mut w, 5941)); // distance 1
        let wp = make(&mut w, &TEST_THROWN);
        skills::set_base_fixed(&mut w, attacker, SkillName::Throwing, 0);
        w.mobile_mut(attacker).unwrap().dex = 40;
        // RawDex-based mitigation = (0 + 40)/20 = 2.0 -> penalty = (12 - 2)/100 = 0.10
        let baseline = hit_chance(&mut w, wp, attacker, defender, 0.8); // 0.70
        let m = StatMod::new(
            &mut w,
            StatType::DEX,
            Some("curse"),
            -30,
            TimeSpan::ZERO,
            None,
        ); // effective Dex 10
        muo_server::mobile::base::add_stat_mod(&mut w, attacker, m);
        let with_mod = hit_chance(&mut w, wp, attacker, defender, 0.8);
        close(baseline, 0.70);
        close(baseline, with_mod); // unchanged because RawDex is used
    }
}

// ---- weapon ability packets ------------------------------------------------------------------------------------

mod ability_packets {
    use super::*;
    use muo_server::net_state::{NetOut, NetStateData, NetStateId};

    fn net_world() -> (World, NetStateId) {
        let mut w = World::new();
        let ns = NetStateId(1);
        w.net_states.insert(ns, NetStateData::new(0));
        (w, ns)
    }

    fn sent(w: &mut World) -> Vec<u8> {
        let mut all: Vec<Vec<u8>> = std::mem::take(&mut w.net_out)
            .into_iter()
            .filter_map(|o| match o {
                NetOut::Send(_, b) => Some(b),
                _ => None,
            })
            .collect();
        assert_eq!(all.len(), 1);
        all.remove(0)
    }

    #[test]
    fn test_special_ability() {
        for (ability_id, active) in [(0_i16, true), (0, false), (100, true), (1000, false)] {
            let (mut w, ns) = net_world();
            // ToggleSpecialAbility: 0xBF, length, 0x25, ability id, active
            let mut expected = vec![0xBF, 0, 0, 0, 0x25];
            expected.extend_from_slice(&ability_id.to_be_bytes());
            expected.push(u8::from(active));
            let n = expected.len() as u16;
            expected[1..3].copy_from_slice(&n.to_be_bytes());

            wa::send_toggle_special_ability(&mut w, ns, ability_id, active);
            assert_eq!(sent(&mut w), expected);
        }
    }

    #[test]
    fn test_clear_ability() {
        let (mut w, ns) = net_world();
        // ClearWeaponAbility: 0xBF, length, 0x21
        wa::send_clear_weapon_ability(&mut w, ns);
        assert_eq!(sent(&mut w), vec![0xBF, 0, 5, 0, 0x21]);
    }
}
