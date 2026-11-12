//! BaseCreatureSerializationTests, MoveSpeedTests, LootingRightsTests, plus tests for
//! the ability machinery and the loot tables.

use muo_io::{SaveReader, SaveWriter};
use muo_primitives::{Point3D, TimeSpan};
use muo_server::ids::MobileId;
use muo_server::mobile::{MOBILE_BASE, MobileType};
use muo_server::world::World;

use super::control::*;
use super::data::{cd, cd_mut};
use super::enums::*;
use super::setters::new_creature;
use super::speeds::{self, NpcSpeeds, SpeedClassEntry};
use super::vtable::{BASE_CREATURE, BASE_CREATURE_C, CreatureType};

/// A creature class with the table speeds 0.3/0.6 (the stub of MoveSpeedTests).
static SPEED_STUB_C: CreatureType = CreatureType { ..BASE_CREATURE_C };
static SPEED_STUB: MobileType = MobileType {
    full_name: "Server.Mobiles.SpeedStub",
    parent: Some(&BASE_CREATURE),
    family: Some(&SPEED_STUB_C),
    ..BASE_CREATURE
};

fn world() -> World {
    let mut w = World::new();
    let t = w.resources.get_mut::<NpcSpeeds>();
    t.register(SpeedClassEntry {
        level: "Medium".into(),
        active: 0.3,
        passive: 0.6,
        active_move: 0.0,
        passive_move: 0.0,
        types: vec![],
    });
    w
}

fn creature(w: &mut World) -> MobileId {
    new_creature(
        w,
        &SPEED_STUB,
        AIType::AiAnimal,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    )
}

fn plain(w: &mut World) -> MobileId {
    w.new_mobile(&MOBILE_BASE)
}

fn snapshot(w: &World, id: MobileId) -> Vec<u8> {
    let mut out = SaveWriter::new(true);
    (w.mobile(id).unwrap().ty.serialize)(w, id, &mut out);
    out.into_vec()
}

/// Loads `buffer` into a fresh creature and checks the reader consumed every byte.
fn load(w: &mut World, buffer: &[u8]) -> MobileId {
    let copy = creature(w);
    let mut r = SaveReader::new(buffer);
    let ty = w.mobile(copy).unwrap().ty;
    (ty.deserialize)(w, copy, &mut r).unwrap();
    assert_eq!(r.remaining(), 0, "exact consumption");
    copy
}

fn d(w: &World, id: MobileId) -> &super::CreatureData {
    cd(w, id).unwrap()
}

// ---- MoveSpeedTests --------------------------------------------------------------------------------------

#[test]
fn move_speeds_inherit_think_values_by_default() {
    let mut w = world();
    let bc = creature(&mut w);
    assert_eq!(
        (d(&w, bc).active_move_speed, d(&w, bc).passive_move_speed),
        (0.0, 0.0)
    );
    assert_eq!(speeds::current_move_speed(&w, bc), d(&w, bc).current_speed);
}

#[test]
fn current_move_speed_resolves_per_mode_when_overridden() {
    let mut w = world();
    let bc = creature(&mut w);
    speeds::set_move_speed(&mut w, bc, 0.45, 0.9);
    assert_eq!(d(&w, bc).current_speed, 0.6);
    assert_eq!(speeds::current_move_speed(&w, bc), 0.9);
    speeds::set_current_speed_to_active(&mut w, bc);
    assert_eq!(d(&w, bc).current_speed, 0.3);
    assert_eq!(speeds::current_move_speed(&w, bc), 0.45);
}

#[test]
fn bespoke_pace_stays_fused() {
    let mut w = world();
    let bc = creature(&mut w);
    speeds::set_move_speed(&mut w, bc, 0.45, 0.9);
    speeds::set_current_speed(&mut w, bc, 0.11);
    assert_eq!(speeds::current_move_speed(&w, bc), 0.11);
}

#[test]
fn set_speed_clears_move_overrides() {
    let mut w = world();
    let bc = creature(&mut w);
    speeds::set_move_speed(&mut w, bc, 0.45, 0.9);
    speeds::set_speed(&mut w, bc, 0.2, 0.4, true);
    assert_eq!(
        (d(&w, bc).active_move_speed, d(&w, bc).passive_move_speed),
        (0.0, 0.0)
    );
}

#[test]
fn non_positive_move_speed_clears_that_override() {
    let mut w = world();
    let bc = creature(&mut w);
    speeds::set_move_speed(&mut w, bc, 0.45, 0.9);
    speeds::set_active_move_speed(&mut w, bc, 0.0);
    assert_eq!(d(&w, bc).active_move_speed, 0.0);
    assert_eq!(d(&w, bc).passive_move_speed, 0.9);
    speeds::set_current_speed_to_active(&mut w, bc);
    assert_eq!(speeds::current_move_speed(&w, bc), 0.3);
    speeds::set_active_move_speed(&mut w, bc, -1.0);
    assert_eq!(
        d(&w, bc).active_move_speed,
        0.0,
        "negative coerces to inherit"
    );
}

#[test]
fn scale_move_speed_scales_overrides_and_leaves_inherit_alone() {
    let mut w = world();
    let bc = creature(&mut w);
    speeds::set_active_move_speed(&mut w, bc, 0.6);
    speeds::scale_move_speed(&mut w, bc, 1.0 / 1.2);
    assert!((d(&w, bc).active_move_speed - 0.5).abs() < 1e-12);
    assert_eq!(d(&w, bc).passive_move_speed, 0.0);
}

#[test]
fn herding_drives_move_clock_think_untouched() {
    let mut w = world();
    let bc = creature(&mut w);
    speeds::set_move_speed(&mut w, bc, 0.45, 1.05);
    cd_mut(&mut w, bc).unwrap().target_location = Some(muo_primitives::Point2D::new(10, 10));
    assert_eq!(d(&w, bc).current_speed, 0.6);
    assert_eq!(speeds::current_move_speed(&w, bc), 0.3);
    cd_mut(&mut w, bc).unwrap().target_location = None;
    assert_eq!(speeds::current_move_speed(&w, bc), 1.05);
}

#[test]
fn migration_matching_think_speeds_adopt_table_move_values() {
    let mut w = world();
    w.resources
        .get_mut::<NpcSpeeds>()
        .register(SpeedClassEntry {
            level: "Medium".into(),
            active: 0.3,
            passive: 0.6,
            active_move: 0.45,
            passive_move: 0.9,
            types: vec![],
        });
    let bc = creature(&mut w);
    // The constructor already adopts the table's move values; clear them to test the migration itself.
    speeds::clear_move_speed(&mut w, bc);
    speeds::migrate_move_speeds(&mut w, bc);
    assert_eq!(
        (d(&w, bc).active_move_speed, d(&w, bc).passive_move_speed),
        (0.45, 0.9)
    );
}

#[test]
fn migration_tuned_think_speeds_keep_inheriting() {
    let mut w = world();
    w.resources
        .get_mut::<NpcSpeeds>()
        .register(SpeedClassEntry {
            level: "Medium".into(),
            active: 0.3,
            passive: 0.6,
            active_move: 0.45,
            passive_move: 0.9,
            types: vec![],
        });
    let bc = creature(&mut w);
    speeds::set_speed(&mut w, bc, 0.35, 0.6, true);
    speeds::migrate_move_speeds(&mut w, bc);
    assert_eq!(
        (d(&w, bc).active_move_speed, d(&w, bc).passive_move_speed),
        (0.0, 0.0)
    );
}

#[test]
fn snap_speeds_to_table_undoes_scaling_drift_keeps_tuned_values() {
    let mut w = world();
    w.resources
        .get_mut::<NpcSpeeds>()
        .register(SpeedClassEntry {
            level: "Medium".into(),
            active: 0.3,
            passive: 0.6,
            active_move: 0.45,
            passive_move: 0.9,
            types: vec![],
        });
    let bc = creature(&mut w);
    speeds::set_move_speed(&mut w, bc, 0.45, 0.9);
    speeds::scale_move_speed(&mut w, bc, 1.0 / 1.2);
    speeds::scale_move_speed(&mut w, bc, 1.2);
    assert_ne!(
        d(&w, bc).active_move_speed,
        0.45,
        "does not survive /1.2 then *1.2 bit-exactly"
    );
    speeds::snap_speeds_to_table(&mut w, bc);
    assert_eq!(d(&w, bc).active_move_speed, 0.45);
    assert_eq!(d(&w, bc).passive_move_speed, 0.9);
    // A hand-tuned value is nowhere near the epsilon and must keep.
    speeds::set_move_speed(&mut w, bc, 0.7, 0.9);
    speeds::snap_speeds_to_table(&mut w, bc);
    assert_eq!(d(&w, bc).active_move_speed, 0.7);
}

#[test]
fn speed_table_lookup_prefers_class_bucket_then_type_then_medium() {
    let mut w = World::new();
    let t = w.resources.get_mut::<NpcSpeeds>();
    let e = |level: &str, a, p, types: &[&str]| SpeedClassEntry {
        level: level.into(),
        active: a,
        passive: p,
        active_move: 0.0,
        passive_move: 0.0,
        types: types.iter().map(|s| (*s).into()).collect(),
    };
    t.register(e("Medium", 0.25, 0.5, &[]));
    t.register(e("Slow", 0.3, 0.6, &["SpeedStub"]));
    let bc = w.new_mobile(&SPEED_STUB);
    assert_eq!(
        speeds::get_speeds(&w, bc),
        (0.3, 0.6),
        "found by the class name in the bucket's type list"
    );
    let other = w.new_mobile(&BASE_CREATURE);
    assert_eq!(
        speeds::get_speeds(&w, other),
        (0.25, 0.5),
        "falls back to Medium"
    );
}

#[test]
fn speed_table_json_parses_as_the_data_file_does() {
    let text = r#"[{"level":"Slow","active":0.3,"passive":0.6,"activeMove":0.6,"passiveMove":1.2,"types":["Rat","Zombie"]},
                   {"level":"VerySlow","active":0.4,"passive":0.8,"types":[]}]"#;
    let v: Vec<SpeedClassEntry> = serde_json::from_str(text).unwrap();
    assert_eq!(v.len(), 2);
    assert_eq!((v[0].active_move, v[0].passive_move), (0.6, 1.2));
    assert_eq!(v[1].active_move, 0.0, "absent activeMove is 0 = inherit");
}

// ---- BaseCreatureSerializationTests ----------------------------------------------------------------------

#[test]
fn default_creature_round_trips_and_elides_everything() {
    let mut w = world();
    let bc = creature(&mut w);
    let buffer = snapshot(&w, bc);
    let copy = load(&mut w, &buffer);
    let c = d(&w, copy);
    assert_eq!(c.current_ai, AIType::AiAnimal);
    assert_eq!(c.range_perception, DEFAULT_RANGE_PERCEPTION);
    assert_eq!(
        (c.active_speed, c.passive_speed, c.current_speed),
        (0.3, 0.6, 0.6)
    );
    assert_eq!(c.physical_damage, 100);
    assert_eq!((c.loyalty, c.control_slots), (MAX_LOYALTY, 1));
    assert!(c.owners.is_empty());
}

#[test]
fn default_creature_writes_no_optional_field() {
    let mut w = world();
    let bc = creature(&mut w);
    let buffer = snapshot(&w, bc);
    let mut base = SaveWriter::new(true);
    (MOBILE_BASE.serialize)(&w, bc, &mut base);
    let tail = &buffer[base.into_vec().len()..];
    // version 24 as an int, an empty ulong flag mask, and DefaultAI (AiAnimal = 2) as an encoded int.
    assert_eq!(tail, [24, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2]);
}

#[test]
fn back_to_back_saves_are_byte_identical() {
    let mut w = world();
    let bc = creature(&mut w);
    super::setters::set_damage(&mut w, bc, 5, 10);
    cd_mut(&mut w, bc).unwrap().physical_resistance_seed = 25;
    assert_eq!(snapshot(&w, bc), snapshot(&w, bc));
}

#[test]
fn populated_creature_round_trips() {
    let mut w = world();
    let bc = creature(&mut w);
    let master = plain(&mut w);
    {
        let c = cd_mut(&mut w, bc).unwrap();
        c.tamable_raw = true;
        c.min_tame_skill = 47.1;
    }
    assert!(set_control_master(&mut w, bc, Some(master)));
    cd_mut(&mut w, bc).unwrap().owners.push(master);
    set_control_order_raw(&mut w, bc, OrderType::Guard);
    super::setters::set_damage(&mut w, bc, 11, 17);
    speeds::set_speed(&mut w, bc, 0.2, 0.4, true);
    speeds::set_move_speed(&mut w, bc, 0.25, 0.5);
    {
        let c = cd_mut(&mut w, bc).unwrap();
        c.physical_resistance_seed = 40;
        c.energy_resist_seed = 15;
        c.fire_damage = 25;
        c.physical_damage = 75;
        c.hits_max_seed = 250;
        c.loyalty = 55;
        c.home = Point3D::new(1000, 1100, 5);
        c.range_home = 4;
        c.team = 3;
        c.is_bonded = true;
        c.remove_if_untamed = true;
        c.remove_step = 2;
        c.corpse_name_override = Some("a test corpse".into());
        c.bonding_begin = muo_primitives::DateTime::from_ymd(2024, 1, 2).unwrap();
    }
    let copy = {
        let b = snapshot(&w, bc);
        load(&mut w, &b)
    };
    let c = d(&w, copy);
    assert!(c.controlled);
    assert_eq!(control_master(&w, copy), Some(master));
    assert_eq!(c.control_order, OrderType::Guard);
    assert!(c.tamable());
    assert_eq!(c.min_tame_skill, 47.1);
    assert_eq!((c.damage_min, c.damage_max), (11, 17));
    assert_eq!(
        (
            c.active_speed,
            c.passive_speed,
            c.active_move_speed,
            c.passive_move_speed
        ),
        (0.2, 0.4, 0.25, 0.5)
    );
    assert_eq!((c.physical_resistance_seed, c.energy_resist_seed), (40, 15));
    assert_eq!(
        (c.fire_damage, c.physical_damage, c.hits_max_seed, c.loyalty),
        (25, 75, 250, 55)
    );
    assert_eq!(
        (c.home, c.range_home, c.team),
        (Point3D::new(1000, 1100, 5), 4, 3)
    );
    assert!(c.is_bonded && c.remove_if_untamed);
    assert_eq!(
        (c.remove_step, c.corpse_name_override.as_deref()),
        (2, Some("a test corpse"))
    );
    assert_eq!(
        c.bonding_begin,
        muo_primitives::DateTime::from_ymd(2024, 1, 2).unwrap()
    );
    assert_eq!(c.owners.last(), Some(&master));
}

/// One serialized master; the Controlled and Summoned flags decide which views expose it.
#[test]
fn master_shapes_round_trip() {
    for (controlled, summoned) in [(true, false), (true, true), (false, true), (false, false)] {
        let mut w = world();
        let bc = creature(&mut w);
        let master = plain(&mut w);
        set_summoned(&mut w, bc, summoned);
        if controlled {
            assert!(set_control_master(&mut w, bc, Some(master)));
        } else {
            set_master(&mut w, bc, Some(master));
        }
        let copy = {
            let b = snapshot(&w, bc);
            load(&mut w, &b)
        };
        let c = d(&w, copy);
        assert_eq!(
            (c.controlled, c.summoned, c.master),
            (controlled, summoned, Some(master))
        );
        assert_eq!(control_master(&w, copy), controlled.then_some(master));
        assert_eq!(summon_master(&w, copy), summoned.then_some(master));
    }
}

#[test]
fn followers_follow_the_master_slots() {
    let mut w = world();
    let bc = creature(&mut w);
    let master = plain(&mut w);
    assert!(set_control_master(&mut w, bc, Some(master)));
    assert_eq!(w.mobile(master).unwrap().followers, 1);
    set_master(&mut w, bc, None);
    assert_eq!(w.mobile(master).unwrap().followers, 0);
    // A creature never becomes its own master.
    set_master(&mut w, bc, Some(bc));
    assert_eq!(d(&w, bc).master, None);
}

#[test]
fn reference_fields_round_trip() {
    let mut w = world();
    let bc = creature(&mut w);
    let friend = plain(&mut w);
    add_pet_friend(&mut w, bc, friend);
    add_pet_friend(&mut w, bc, friend);
    cd_mut(&mut w, bc).unwrap().home_map = Some(muo_server::map::MapId::FELUCCA);
    let copy = {
        let b = snapshot(&w, bc);
        load(&mut w, &b)
    };
    assert_eq!(d(&w, copy).friends.as_deref(), Some(&[friend][..]));
    assert_eq!(d(&w, copy).home_map, Some(muo_server::map::MapId::FELUCCA));
    remove_pet_friend(&mut w, copy, friend);
    assert_eq!(d(&w, copy).friends, None, "an empty list is the null list");
}

#[test]
fn running_delete_timer_round_trips() {
    let mut w = world();
    let bc = creature(&mut w);
    begin_delete_timer(&mut w, bc);
    let h = d(&w, bc).pending_delete_timer;
    assert!(h.running(&w));
    let copy = {
        let b = snapshot(&w, bc);
        load(&mut w, &b)
    };
    let h2 = d(&w, copy).pending_delete_timer;
    // Anchored: the remaining countdown survives, not the absolute deadline.
    let left = h2.next(&w).since(w.now);
    assert!(
        left <= TimeSpan::from_days(3) && left >= TimeSpan::from_days(3) - TimeSpan::from_secs(5),
        "{left:?}"
    );
}

#[test]
fn legacy_v22_stream_loads_through_the_pre_codegen_path() {
    // A byte-authentic version 22 tail: every field of the old Serialize for a default MeleeAI creature.
    let mut w = world();
    let bc = creature(&mut w);
    let mut base = SaveWriter::new(true);
    (MOBILE_BASE.serialize)(&w, bc, &mut base);
    let mut out = base;
    use muo_io::GenericWriter;
    out.write_i32(22);
    out.write_i32(AIType::AiAnimal as i32); // current
    out.write_i32(AIType::AiAnimal as i32); // default
    out.write_i32(16); // range perception
    out.write_i32(1); // range fight
    out.write_i32(0); // team
    out.write_f64(0.3);
    out.write_f64(0.6);
    out.write_f64(0.6);
    out.write_i32(0); // home x, y, z
    out.write_i32(0);
    out.write_i32(0);
    out.write_i32(10); // range home (v1)
    out.write_i32(4); // fight mode (v2): Closest
    out.write_bool(false); // controlled
    out.write_i32(-1); // control master
    out.write_i32(-1); // control target
    out.write_point3d(Point3D::ZERO);
    out.write_i32(0); // control order
    out.write_f64(0.0); // min tame skill
    out.write_bool(false); // tamable
    out.write_bool(false); // summoned
    out.write_i32(1); // control slots
    out.write_i32(100); // loyalty (v3)
    out.write_i32(-1); // way point (v4)
    out.write_i32(-1); // master (v5)
    for _ in 0..5 {
        out.write_i32(-1); // hits/stam/mana seed, damage min/max (v6)
    }
    for (res, dmg) in [(0, 100), (0, 0), (0, 0), (0, 0), (0, 0)] {
        out.write_i32(res); // v7: resist seed then damage, five times
        out.write_i32(dmg);
    }
    out.write_i32(0); // owners (v8)
    out.write_bool(false); // dead pet (v10)
    out.write_bool(false); // bonded
    out.write_date_time_ticks(0);
    out.write_date_time_ticks(0);
    out.write_bool(false); // has generated loot (v11)
    out.write_bool(false); // paragon (v12)
    out.write_bool(false); // has friends (v13)
    out.write_bool(false); // remove if untamed (v14)
    out.write_i32(0); // remove step
    out.write_time_span_ticks(0); // delete time (v17)
    out.write_opt_string(None); // corpse name override (v18)
    out.write_u8(0xFF); // home map (v19)
    out.write_f64(0.0); // active move speed (v22)
    out.write_f64(0.0);
    let bytes = out.into_vec();
    let copy = load(&mut w, &bytes);
    let c = d(&w, copy);
    assert_eq!(
        (c.default_ai, c.fight_mode, c.loyalty, c.range_home),
        (AIType::AiAnimal, FightMode::Closest, 100, 10)
    );
    assert_eq!((c.active_speed, c.passive_speed), (0.3, 0.6));
}

// ---- LootingRightsTests ----------------------------------------------------------------------------------

fn player(w: &mut World) -> MobileId {
    let id = plain(w);
    w.mobile_mut(id).unwrap().player = true;
    id
}

fn register_damage(w: &mut World, victim: MobileId, amount: i32, from: MobileId) {
    let ty = w.mobile(victim).unwrap().ty;
    (ty.register_damage)(w, victim, amount, Some(from));
}

#[test]
fn two_player_damagers_sort_descending_and_the_first_damager_takes_the_bonus() {
    let mut w = world();
    let victim = plain(&mut w);
    let (first, second) = (player(&mut w), player(&mut w));
    register_damage(&mut w, victim, 100, first);
    register_damage(&mut w, victim, 40, second); // second is the most recent, first is the "first damager"
    // hitsMax < 200 puts the bar at topDamage / 2.
    let rights = super::death::get_looting_rights(&mut w, victim, 100);
    assert_eq!(rights.len(), 2);
    assert_eq!((rights[0].mobile, rights[1].mobile), (first, second));
    // The first damager, the least recent entry, gets the 1.25x bonus; nobody else does.
    assert_eq!((rights[0].damage, rights[1].damage), (125, 40));
    // topDamage 125 / 2 = 62, so 40 is below the bar.
    assert!(rights[0].has_right && !rights[1].has_right);
}

#[test]
fn pet_damage_credits_the_master_not_the_pet() {
    let mut w = world();
    let victim = plain(&mut w);
    let master = player(&mut w);
    let pet = creature(&mut w);
    set_control_master(&mut w, pet, Some(master));
    let wild = plain(&mut w); // no damage master, and not a player
    register_damage(&mut w, victim, 50, pet);
    register_damage(&mut w, victim, 20, wild);
    let rights = super::death::get_looting_rights(&mut w, victim, 100);
    // The master is credited through the entry's Responsible sub-entry, and is the only one.
    assert_eq!(rights.len(), 1);
    assert_eq!(rights[0].mobile, master);
    assert_eq!(rights[0].damage, 62); // 50, then the first-damager 1.25x bonus
    assert!(rights[0].has_right);
}

#[test]
fn hits_max_bands_set_the_bar() {
    let mut w = world();
    let victim = plain(&mut w);
    let (a, b) = (player(&mut w), player(&mut w));
    register_damage(&mut w, victim, 1000, a);
    register_damage(&mut w, victim, 150, b);
    // top 1250 after the bonus; the bar is /16 at 3000+, /8 at 1000+, /4 at 200+, /2 below.
    for (hits_max, b_has_right) in [(3000, true), (1000, false), (200, false), (100, false)] {
        let r = super::death::get_looting_rights(&mut w, victim, hits_max);
        assert_eq!(r[1].has_right, b_has_right, "hitsMax {hits_max}");
    }
}

#[test]
fn treasures_of_tokuno_roll_once_per_death_not_once_per_holder() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static CALLS: AtomicUsize = AtomicUsize::new(0);
    CALLS.store(0, Ordering::SeqCst);
    let mut w = world();
    w.resources
        .get_mut::<crate::hooks::DeathHooks>()
        .on_death_rights_holder = Some(|_, _, _, _| {
        CALLS.fetch_add(1, Ordering::SeqCst);
    });
    let victim = creature(&mut w);
    w.mobile_mut(victim).unwrap().map = Some(muo_server::map::MapId::TOKUNO);
    let (a, b) = (player(&mut w), player(&mut w));
    register_damage(&mut w, victim, 100, a);
    register_damage(&mut w, victim, 90, b); // both keep their rights
    super::death::on_death(&mut w, victim, None);
    assert_eq!(CALLS.load(Ordering::SeqCst), 1);
    // off Tokuno and outside the two regions: no roll at all
    CALLS.store(0, Ordering::SeqCst);
    let other = creature(&mut w);
    register_damage(&mut w, other, 100, a);
    super::death::on_death(&mut w, other, None);
    assert_eq!(CALLS.load(Ordering::SeqCst), 0);
}

// ---- ability machinery ------------------------------------------------------------------------------------

mod ability {
    use super::*;
    use crate::abilities::*;

    static TICKS: std::sync::atomic::AtomicI32 = std::sync::atomic::AtomicI32::new(0);

    static COUNTING: MonsterAbility = MonsterAbility {
        name: "Counting",
        trigger: MonsterAbilityTrigger::COMBAT_ACTION,
        min_cooldown: TimeSpan::from_secs(30),
        max_cooldown: TimeSpan::from_secs(30),
        do_trigger: |w, a, t, s, tg| {
            TICKS.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            base_trigger(w, a, t, s, tg);
        },
        ..BASE
    };
    static ABILITIES: &[&MonsterAbility] = &[&COUNTING];
    static BREATHER_C: CreatureType = CreatureType {
        get_monster_abilities: |_, _| ABILITIES,
        ..BASE_CREATURE_C
    };
    static BREATHER: MobileType = MobileType {
        full_name: "Server.Mobiles.Breather",
        parent: Some(&BASE_CREATURE),
        family: Some(&BREATHER_C),
        ..BASE_CREATURE
    };

    #[test]
    fn cooldown_is_the_real_rate_limit_and_death_clears_it() {
        let mut w = world();
        let bc = new_creature(
            &mut w,
            &BREATHER,
            AIType::AiMelee,
            FightMode::Closest,
            16,
            1,
        );
        let before = TICKS.load(std::sync::atomic::Ordering::SeqCst);
        // The mobile must be alive and present: new mobiles are.
        assert!(trigger_ability(
            &mut w,
            bc,
            MonsterAbilityTrigger::COMBAT_ACTION,
            None
        ));
        assert!(is_ability_on_cooldown(&mut w, bc, &COUNTING));
        assert!(
            !trigger_ability(&mut w, bc, MonsterAbilityTrigger::COMBAT_ACTION, None),
            "on cooldown"
        );
        assert_eq!(TICKS.load(std::sync::atomic::Ordering::SeqCst), before + 1);
        // 30 s later the cooldown has lapsed.
        w.tick_count += 30_001;
        assert!(!is_ability_on_cooldown(&mut w, bc, &COUNTING));
        // `[OnEvent(CreatureDeathEvent)] ResetCooldownsOnDeath`.
        set_ability_cooldown(&mut w, bc, &COUNTING, TimeSpan::from_secs(30));
        reset_cooldowns_on_death(&mut w, bc);
        assert!(!is_ability_on_cooldown(&mut w, bc, &COUNTING));
    }

    #[test]
    fn a_trigger_does_not_re_enter_itself() {
        static NESTED: MonsterAbility = MonsterAbility {
            name: "Nested",
            trigger: MonsterAbilityTrigger::TAKE_MELEE_DAMAGE,
            do_trigger: |w, _, _, s, _| {
                // A handler that triggers the same trigger again must be refused.
                assert!(!trigger_ability(
                    w,
                    s,
                    MonsterAbilityTrigger::TAKE_MELEE_DAMAGE,
                    None
                ));
            },
            ..BASE
        };
        static LIST: &[&MonsterAbility] = &[&NESTED];
        static C: CreatureType = CreatureType {
            get_monster_abilities: |_, _| LIST,
            ..BASE_CREATURE_C
        };
        static T: MobileType = MobileType {
            full_name: "Server.Mobiles.Nester",
            parent: Some(&BASE_CREATURE),
            family: Some(&C),
            ..BASE_CREATURE
        };
        let mut w = world();
        let bc = new_creature(&mut w, &T, AIType::AiMelee, FightMode::Closest, 16, 1);
        assert!(trigger_ability(
            &mut w,
            bc,
            MonsterAbilityTrigger::TAKE_MELEE_DAMAGE,
            None
        ));
        assert!(
            trigger_ability(&mut w, bc, MonsterAbilityTrigger::TAKE_MELEE_DAMAGE, None),
            "released after the call"
        );
    }

    #[test]
    fn group_picks_one_member_by_weight_and_unions_triggers() {
        static A: MonsterAbility = MonsterAbility {
            name: "A",
            trigger: MonsterAbilityTrigger::THINK,
            ..BASE
        };
        static B: MonsterAbility = MonsterAbility {
            name: "B",
            trigger: MonsterAbilityTrigger::COMBAT_ACTION,
            ..BASE
        };
        static MEMBERS: [Weighted; 2] = [Weighted::new(1, &A), Weighted::new(1, &B)];
        static G: MonsterAbility = group(
            &MEMBERS,
            MonsterAbilityTrigger::THINK.or(MonsterAbilityTrigger::COMBAT_ACTION),
        );
        assert!(
            G.trigger.intersects(MonsterAbilityTrigger::THINK)
                && G.trigger.intersects(MonsterAbilityTrigger::COMBAT_ACTION)
        );
        assert!(has_group_member(&G, &A) && has_group_member(&G, &B));
    }

    fn has_group_member(g: &'static MonsterAbility, a: &'static MonsterAbility) -> bool {
        g.group.iter().any(|m| same(m.value, a))
    }

    #[test]
    fn trigger_masks_match_the_enum() {
        assert_eq!(MonsterAbilityTrigger::GIVE_DAMAGE.0, 0x14);
        assert_eq!(MonsterAbilityTrigger::TAKE_DAMAGE.0, 0x0A);
        assert_eq!(MonsterAbilityTrigger::DEATH.0, 0x40);
    }
}

// ---- pet orders -------------------------------------------------------------------------------------------

#[test]
fn stay_anchors_home_and_stop_cancels_it() {
    let mut w = world();
    let bc = creature(&mut w);
    let master = plain(&mut w);
    set_control_master(&mut w, bc, Some(master));
    w.mobile_mut(bc).unwrap().location = Point3D::new(10, 20, 0);
    issue_order(&mut w, bc, OrderType::Stay, Some(master), None);
    assert_eq!(d(&w, bc).control_order, OrderType::Stay);
    assert_eq!(
        d(&w, bc).home,
        Point3D::new(10, 20, 0),
        "Stay anchors Home at the post"
    );
    assert_eq!(crate::ai::persistent_order(&w, bc), OrderType::Stay);
    issue_order(&mut w, bc, OrderType::Stop, Some(master), None);
    assert_eq!(d(&w, bc).control_order, OrderType::None);
    assert_eq!(crate::ai::persistent_order(&w, bc), OrderType::None);
}

#[test]
fn follow_clears_home_and_remembers_its_target() {
    let mut w = world();
    let bc = creature(&mut w);
    let master = plain(&mut w);
    set_control_master(&mut w, bc, Some(master));
    issue_order(&mut w, bc, OrderType::Follow, Some(master), Some(master));
    assert_eq!(d(&w, bc).control_order, OrderType::Follow);
    assert_eq!(d(&w, bc).home, Point3D::ZERO);
    assert_eq!(d(&w, bc).control_target, Some(master));
    // A transient order resumes the standing Follow.
    issue_order(&mut w, bc, OrderType::Friend, Some(master), Some(master));
    assert_eq!(d(&w, bc).control_order, OrderType::Follow);
}

#[test]
fn release_frees_a_tamed_pet() {
    let mut w = world();
    let bc = creature(&mut w);
    let master = plain(&mut w);
    set_control_master(&mut w, bc, Some(master));
    cd_mut(&mut w, bc).unwrap().is_bonded = true;
    issue_order(&mut w, bc, OrderType::Release, Some(master), None);
    let c = d(&w, bc);
    assert!(!c.controlled && !c.is_bonded);
    assert_eq!(c.master, None);
    assert_eq!(w.mobile(master).unwrap().followers, 0);
    assert!(
        c.pending_delete_timer.running(&w),
        "a released pet starts its delete countdown"
    );
}

#[test]
fn the_creature_fields_match_the_constructor() {
    let mut w = world();
    let bc = creature(&mut w);
    let c = d(&w, bc);
    assert_eq!(
        (c.loyalty, c.range_home, c.range_fight, c.fight_mode),
        (MAX_LOYALTY, 10, 1, FightMode::Closest)
    );
    assert_eq!(
        (c.default_ai, c.current_ai),
        (AIType::AiAnimal, AIType::AiAnimal)
    );
    assert!(crate::ai::class_of(&w, bc).is_some_and(|c| c.name == "AnimalAI"));
    // The pre-v22 helper `ScaleType` etc. are plain enums; spot-check the numeric values saves rely on.
    assert_eq!(
        (
            OrderType::Rename as i32,
            FightMode::Evil as i32,
            AIType::AiThief as i32
        ),
        (13, 5, 9)
    );
    assert_eq!(MAX_LOYALTY_INCREASE, 10);
}

// ---- context menu, speech, control chance ------------------------------------------------------------------

fn tame(w: &mut World) -> (MobileId, MobileId) {
    let bc = creature(w);
    let master = plain(w);
    w.mobile_mut(master).unwrap().map = Some(muo_server::map::MapId::FELUCCA);
    w.mobile_mut(bc).unwrap().map = Some(muo_server::map::MapId::FELUCCA);
    set_control_master(w, bc, Some(master));
    (bc, master)
}

#[test]
fn pet_context_menu_lists_the_owner_commands_and_the_friend_subset() {
    let mut w = world();
    let (bc, master) = tame(&mut w);
    let friend = plain(&mut w);
    w.mobile_mut(friend).unwrap().map = Some(muo_server::map::MapId::FELUCCA);
    add_pet_friend(&mut w, bc, friend);
    let mut owner = Vec::new();
    (w.mobile(bc).unwrap().ty.get_context_menu_entries)(&mut w, bc, master, &mut owner);
    let numbers: Vec<i32> = owner.iter().map(|e| e.number).collect();
    // Kill, Follow, Guard, Stop, Stay, Rename, Add/Remove Friend, Transfer, Release (no Drop: not bonded).
    assert_eq!(
        numbers,
        [
            3006111, 3006108, 3006107, 3006112, 3006114, 3006098, 3006110, 3006099, 3006113,
            3006118
        ]
    );
    let mut pal = Vec::new();
    (w.mobile(bc).unwrap().ty.get_context_menu_entries)(&mut w, bc, friend, &mut pal);
    assert_eq!(
        pal.iter().map(|e| e.number).collect::<Vec<_>>(),
        [3006108, 3006112, 3006114]
    );
    let mut stranger = Vec::new();
    let other = plain(&mut w);
    (w.mobile(bc).unwrap().ty.get_context_menu_entries)(&mut w, bc, other, &mut stranger);
    assert!(stranger.is_empty());
    // Clicking Stay issues the order (a tamed pet always passes the control roll with no MinTameSkill).
    owner[4].click(&mut w, master, muo_server::ids::EntityId::Mobile(bc));
    assert_eq!(d(&w, bc).control_order, OrderType::Stay);
}

#[test]
fn named_speech_orders_the_pet() {
    let mut w = world();
    let (bc, master) = tame(&mut w);
    w.mobile_mut(bc).unwrap().name = Some("fido".into());
    w.mobile_mut(bc).unwrap().player = false;
    let say = |w: &mut World, text: &str, kw: i32| {
        let mut e = muo_server::events::SpeechEventArgs::new(
            master,
            text.into(),
            muo_primitives::MessageType::REGULAR,
            0,
            vec![kw],
        );
        crate::ai::on_speech(w, bc, &mut e);
    };
    say(&mut w, "fido stay", 0x16F);
    assert_eq!(d(&w, bc).control_order, OrderType::Stay);
    say(&mut w, "fido guard", 0x15C);
    assert_eq!(d(&w, bc).control_order, OrderType::Guard);
    say(&mut w, "all stop", 0x167);
    assert_eq!(
        d(&w, bc).control_order,
        OrderType::None,
        "Stop from Guard rests at None"
    );
    // A stranger's command is ignored.
    let other = plain(&mut w);
    w.mobile_mut(other).unwrap().map = Some(muo_server::map::MapId::FELUCCA);
    let mut e = muo_server::events::SpeechEventArgs::new(
        other,
        "fido stay".into(),
        muo_primitives::MessageType::REGULAR,
        0,
        vec![0x16F],
    );
    crate::ai::on_speech(&mut w, bc, &mut e);
    assert_eq!(d(&w, bc).control_order, OrderType::None);
    assert!(crate::ai::speech::was_named(&w, bc, "Fido come here"));
    assert!(!crate::ai::speech::was_named(&w, bc, "rex come"));
}

#[test]
fn control_chance_is_one_for_an_easy_or_summoned_pet() {
    let mut w = world();
    let (bc, master) = tame(&mut w);
    let c = w.mobile(bc).unwrap().ty.family::<CreatureType>().unwrap();
    assert_eq!(
        (c.get_control_chance)(&mut w, bc, master, false),
        1.0,
        "MinTameSkill <= 29.1"
    );
    cd_mut(&mut w, bc).unwrap().min_tame_skill = 80.0;
    // No taming or lore skill: (700 + bonus) clamps at the 220 floor, the loyalty penalty is 0 at full loyalty.
    let chance = (c.get_control_chance)(&mut w, bc, master, false);
    assert!((0.22 - 1e-9..=0.99).contains(&chance), "{chance}");
    cd_mut(&mut w, bc).unwrap().loyalty = 50;
    let lower = (c.get_control_chance)(&mut w, bc, master, false);
    assert!(
        lower < chance,
        "loyalty below max costs 10 per point: {lower} < {chance}"
    );
    cd_mut(&mut w, bc).unwrap().summoned = true;
    assert_eq!((c.get_control_chance)(&mut w, bc, master, false), 1.0);
}

#[test]
fn find_my_name_needs_every_word() {
    let mut w = world();
    let bc = creature(&mut w);
    w.mobile_mut(bc).unwrap().name = Some("a big dog".into());
    assert!(super::pets::find_my_name(&w, bc, "a big dog come", false));
    assert!(!super::pets::find_my_name(&w, bc, "a big cat come", false));
    assert!(
        super::pets::find_my_name(&w, bc, "a big cat come all", true),
        "all matches every pet"
    );
    assert!(
        !super::pets::find_my_name(&w, bc, "dog", false),
        "shorter than the name"
    );
}

// ---- MasterViewsTests, DeletedMasterTests ----------------------------------------------------------------

fn real_player(w: &mut World) -> MobileId {
    crate::wiring::configure(w);
    // PlayerMobile.Configure
    let h = w.resources.get_mut::<crate::hooks::PetPlayerHooks>();
    h.add_follower = Some(crate::player_mobile::add_follower);
    h.remove_follower = Some(|w, pm, c| {
        crate::player_mobile::remove_follower(w, pm, c);
    });
    crate::player_mobile::new_player_mobile(w)
}

fn followers(w: &World, m: MobileId) -> i32 {
    w.mobile(m).unwrap().followers
}

#[derive(Copy, Clone, Debug)]
enum Shape {
    Wild,
    Pet,
    ControlledSummon,
    UncontrolledSummon,
    Enraged,
}

#[test]
fn master_views_by_shape() {
    for shape in [
        Shape::Wild,
        Shape::Pet,
        Shape::ControlledSummon,
        Shape::UncontrolledSummon,
        Shape::Enraged,
    ] {
        let mut w = world();
        let bc = creature(&mut w);
        let master = real_player(&mut w);

        match shape {
            Shape::Pet => {
                set_control_master(&mut w, bc, Some(master));
            }
            Shape::ControlledSummon => {
                set_control_master(&mut w, bc, Some(master));
                set_summoned(&mut w, bc, true);
            }
            Shape::UncontrolledSummon => {
                set_summoned(&mut w, bc, true);
                set_master(&mut w, bc, Some(master));
            }
            Shape::Enraged => set_master(&mut w, bc, Some(master)),
            Shape::Wild => {}
        }

        let controlled = matches!(shape, Shape::Pet | Shape::ControlledSummon);
        let summoned = matches!(shape, Shape::ControlledSummon | Shape::UncontrolledSummon);

        assert_eq!(
            d(&w, bc).master,
            (!matches!(shape, Shape::Wild)).then_some(master),
            "{shape:?}"
        );
        assert_eq!(
            control_master(&w, bc),
            controlled.then_some(master),
            "{shape:?}"
        );
        assert_eq!(
            summon_master(&w, bc),
            summoned.then_some(master),
            "{shape:?}"
        );
        assert_eq!(
            get_master(&w, bc),
            (controlled || summoned).then_some(master),
            "{shape:?}"
        );
    }
}

#[test]
fn master_reassignment_moves_follower_slots() {
    let mut w = world();
    let bc = creature(&mut w);
    cd_mut(&mut w, bc).unwrap().control_slots = 2;
    let (first, second) = (real_player(&mut w), real_player(&mut w));

    set_master(&mut w, bc, Some(first));
    assert_eq!(followers(&w, first), 2);

    set_master(&mut w, bc, Some(second));
    assert_eq!(followers(&w, first), 0);
    assert_eq!(followers(&w, second), 2);
    assert_eq!(control_master(&w, bc), None); // not Controlled

    set_master(&mut w, bc, None);
    assert_eq!(followers(&w, second), 0);
}

#[test]
fn set_control_master_null_keeps_an_uncontrolled_summons_caster() {
    let mut w = world();
    let bc = creature(&mut w);
    let caster = real_player(&mut w);
    set_summoned(&mut w, bc, true);
    set_master(&mut w, bc, Some(caster));

    set_control_master(&mut w, bc, None);

    assert_eq!(summon_master(&w, bc), Some(caster));
    assert_eq!(followers(&w, caster), d(&w, bc).control_slots);
}

#[test]
fn delete_returns_the_casters_follower_slots() {
    let mut w = world();
    let bc = creature(&mut w);
    let caster = real_player(&mut w);
    set_summoned(&mut w, bc, true);
    set_master(&mut w, bc, Some(caster));

    muo_server::mobile::base::delete(&mut w, bc);

    assert_eq!(followers(&w, caster), 0);
    assert!(cd(&w, bc).is_none_or(|c| c.master.is_none()));
}

#[test]
fn deleting_a_player_releases_an_unbonded_pet() {
    let mut w = world();
    let owner = real_player(&mut w);
    let pet = creature(&mut w);
    set_control_master(&mut w, pet, Some(owner));

    muo_server::mobile::base::delete(&mut w, owner);

    assert!(w.mobile(pet).is_some_and(|m| !m.deleted));
    assert!(!d(&w, pet).controlled);
    assert_eq!(d(&w, pet).master, None);
    assert!(d(&w, pet).pending_delete_timer.running(&w));
}

#[test]
fn deleting_a_player_deletes_a_bonded_pet() {
    for dead in [false, true] {
        let mut w = world();
        let owner = real_player(&mut w);
        let pet = creature(&mut w);
        set_control_master(&mut w, pet, Some(owner));
        set_is_bonded(&mut w, pet, true);
        cd_mut(&mut w, pet).unwrap().is_dead_pet = dead;

        muo_server::mobile::base::delete(&mut w, owner);

        assert!(w.mobile(pet).is_none_or(|m| m.deleted), "dead {dead}");
    }
}

#[test]
fn deleting_a_player_deletes_its_summons() {
    for controlled in [false, true] {
        let mut w = world();
        let caster = real_player(&mut w);
        let summon = creature(&mut w);
        if controlled {
            set_control_master(&mut w, summon, Some(caster));
        }
        set_summoned(&mut w, summon, true);
        set_master(&mut w, summon, Some(caster));

        muo_server::mobile::base::delete(&mut w, caster);

        assert!(
            w.mobile(summon).is_none_or(|m| m.deleted),
            "controlled {controlled}"
        );
    }
}

// ---- BaseCreatureSerializationTests: the byte-authentic legacy v22 tail and the v23 stream ---------------------------

fn melee(w: &mut World) -> MobileId {
    new_creature(
        w,
        &SPEED_STUB,
        AIType::AiMelee,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    )
}

fn load_melee(w: &mut World, buffer: &[u8]) -> MobileId {
    let copy = melee(w);
    let mut r = SaveReader::new(buffer);
    let ty = w.mobile(copy).unwrap().ty;
    (ty.deserialize)(w, copy, &mut r).unwrap();
    assert_eq!(r.remaining(), 0, "exact consumption");
    copy
}

/// A plain `Mobile` section (identical layout for every Mobile subclass), as the donor writes it.
fn mobile_section(w: &mut World) -> SaveWriter {
    let donor = plain(w);
    let mut out = SaveWriter::new(true);
    (MOBILE_BASE.serialize)(w, donor, &mut out);
    out
}

/// `WriteLegacyV22Tail`.
fn write_legacy_v22_tail(
    w: &World,
    out: &mut SaveWriter,
    controlled: bool,
    control_master: Option<MobileId>,
    summoned: bool,
    summon_master: Option<MobileId>,
    summon_end: muo_primitives::DateTime,
) {
    use muo_io::GenericWriter;
    use muo_server::serialization_ext::WriteEntityExt;
    out.write_i32(22); // version
    out.write_i32(AIType::AiMelee as i32); // current AI
    out.write_i32(AIType::AiMelee as i32); // default AI
    out.write_i32(10); // RangePerception
    out.write_i32(1); // RangeFight
    out.write_i32(0); // Team
    out.write_f64(0.3); // active (matches the stub table)
    out.write_f64(0.6); // passive
    out.write_f64(0.6); // current
    out.write_i32(2000); // Home X
    out.write_i32(2100); // Home Y
    out.write_i32(7); // Home Z
    out.write_i32(6); // RangeHome
    out.write_i32(FightMode::Closest as i32);
    out.write_bool(controlled);
    out.write_mobile(w, control_master);
    out.write_mobile(w, None); // control target
    out.write_point3d(Point3D::ZERO); // control dest
    out.write_i32(OrderType::None as i32);
    out.write_f64(0.0); // min tame skill
    out.write_bool(true); // tamable
    out.write_bool(summoned);
    if summoned {
        out.write_anchored_time_ticks(summon_end.ticks());
    }
    out.write_i32(2); // control slots
    out.write_i32(73); // loyalty
    out.write_item(w, None); // waypoint
    out.write_mobile(w, summon_master);
    out.write_i32(180); // hits seed
    out.write_i32(-1); // stam seed
    out.write_i32(-1); // mana seed
    out.write_i32(7); // damage min
    out.write_i32(14); // damage max
    out.write_i32(30); // phys resist
    out.write_i32(100); // phys damage
    out.write_i32(10); // fire resist
    out.write_i32(0); // fire damage
    for _ in 0..6 {
        out.write_i32(0); // cold, poison, energy resist and damage
    }
    out.write_i32(0); // owners
    out.write_bool(false); // dead pet
    out.write_bool(false); // bonded
    out.write_date_time_ticks(muo_primitives::DateTime::MIN.ticks()); // bonding begin
    out.write_date_time_ticks(muo_primitives::DateTime::MIN.ticks()); // abandon time
    out.write_bool(true); // has generated loot
    out.write_bool(false); // paragon
    out.write_bool(false); // has friends
    out.write_bool(false); // remove if untamed
    out.write_i32(0); // remove step
    out.write_time_span_ticks(0); // delete time left
    out.write_opt_string(None); // corpse name override
    out.write_u8(0xFF); // home map
    out.write_f64(0.0); // active move speed (v22)
    out.write_f64(0.0); // passive move speed (v22)
}

fn load_legacy_v22(
    w: &mut World,
    controlled: bool,
    control_master: Option<MobileId>,
    summoned: bool,
    summon_master: Option<MobileId>,
    summon_end: muo_primitives::DateTime,
) -> MobileId {
    let mut out = mobile_section(w);
    write_legacy_v22_tail(
        w,
        &mut out,
        controlled,
        control_master,
        summoned,
        summon_master,
        summon_end,
    );
    load_melee(w, &out.into_vec())
}

#[test]
fn legacy_v22_stream_loads_through_legacy_path() {
    let mut w = world();
    let now = w.now;
    let copy = load_legacy_v22(&mut w, false, None, false, None, now);
    let c = d(&w, copy);
    assert_eq!(c.range_perception, 10);
    assert_eq!(c.home, Point3D::new(2000, 2100, 7));
    assert_eq!(c.range_home, 6);
    assert!(c.tamable_raw);
    assert_eq!(c.control_slots, 2);
    assert_eq!(c.loyalty, 73);
    assert_eq!(c.hits_max_seed, 180);
    assert_eq!((c.damage_min, c.damage_max), (7, 14));
    assert_eq!(c.physical_resistance_seed, 30);
    assert_eq!(c.fire_resist_seed, 10);
    assert_eq!(c.active_speed, 0.3);
    // v22 wrote explicit zeros for the move overrides ("inherit"), so the resolved pace falls back to the think clock.
    assert_eq!(c.active_move_speed, 0.0);
    assert_eq!(speeds::current_move_speed(&w, copy), 0.6); // passive mode, inheriting
}

#[test]
fn legacy_v22_stream_keeps_both_master_references() {
    for (controlled, has_control_master, summoned, has_summon_master) in [
        (true, true, false, false),  // controlled pet
        (true, true, true, true),    // controlled summon, SummonEnd on the wire
        (false, false, false, true), // EnragedCreature shape: SummonMaster only
    ] {
        let mut w = world();
        let master = real_player(&mut w);
        let summon_end = w.now + TimeSpan::from_mins(5);
        let copy = load_legacy_v22(
            &mut w,
            controlled,
            has_control_master.then_some(master),
            summoned,
            has_summon_master.then_some(master),
            summon_end,
        );
        let c = d(&w, copy);
        assert_eq!(c.controlled, controlled);
        assert_eq!(c.summoned, summoned);
        assert_eq!(
            control_master(&w, copy),
            has_control_master.then_some(master)
        );
        assert_eq!(c.master, Some(master));
        assert_eq!(summon_master(&w, copy), summoned.then_some(master));
        if summoned {
            let one = TimeSpan::from_secs(1);
            assert!(c.summon_end >= summon_end - one && c.summon_end <= summon_end + one);
        }
    }
}

// Before summon ownership followed transfers, a traded summon kept its caster as summon master.
#[test]
fn legacy_v22_stream_differing_masters_owner_wins() {
    let mut w = world();
    let (owner, caster) = (real_player(&mut w), real_player(&mut w));
    let end = w.now + TimeSpan::from_mins(5);
    let copy = load_legacy_v22(&mut w, true, Some(owner), true, Some(caster), end);
    assert_eq!(control_master(&w, copy), Some(owner));
    assert_eq!(summon_master(&w, copy), Some(owner));
}

// v23 bit positions: every save-flagged field in schema order (DefaultAI is not flagged).
const V23_CONTROLLED: u32 = 13;
const V23_CONTROL_MASTER: u32 = 14;
const V23_SUMMONED: u32 = 20;
const V23_SUMMON_END: u32 = 21;
const V23_SUMMON_MASTER: u32 = 22;
const V23_REMOVE_STEP: u32 = 50;
const V23_CORPSE_NAME_OVERRIDE: u32 = 52;

// v23 wrote the owner and the summoner to separate slots; v24 keeps one master.
#[test]
fn v23_stream_migrates_to_one_master_and_defaults_absent_fields() {
    use muo_io::GenericWriter;
    use muo_server::serialization_ext::WriteEntityExt;
    let mut w = world();
    let (owner, caster) = (real_player(&mut w), real_player(&mut w));
    let summon_end = w.now + TimeSpan::from_mins(5);

    let mut out = mobile_section(&mut w);
    out.write_i32(23); // version
    out.write_u64(
        1u64 << V23_CONTROLLED
            | 1 << V23_CONTROL_MASTER
            | 1 << V23_SUMMONED
            | 1 << V23_SUMMON_END
            | 1 << V23_SUMMON_MASTER
            | 1 << V23_REMOVE_STEP
            | 1 << V23_CORPSE_NAME_OVERRIDE,
    );
    out.write_encoded_int(AIType::AiMelee as i32); // DefaultAI
    out.write_mobile(&w, Some(owner)); // ControlMaster
    out.write_anchored_time_ticks(summon_end.ticks()); // SummonEnd
    out.write_mobile(&w, Some(caster)); // SummonMaster
    out.write_encoded_int(3); // RemoveStep
    out.write_opt_string(Some("a migrated corpse")); // CorpseNameOverride

    let copy = load_melee(&mut w, &out.into_vec());
    let c = d(&w, copy);
    assert!(c.controlled);
    assert!(c.summoned);
    assert_eq!(control_master(&w, copy), Some(owner));
    assert_eq!(summon_master(&w, copy), Some(owner));
    let one = TimeSpan::from_secs(1);
    assert!(c.summon_end >= summon_end - one && c.summon_end <= summon_end + one);
    assert_eq!(c.remove_step, 3);
    assert_eq!(c.corpse_name_override.as_deref(), Some("a migrated corpse"));

    // Absent fields take the class defaults, not default(T).
    assert_eq!(c.current_ai, AIType::AiMelee);
    assert_eq!(c.hits_max_seed, -1);
    assert_eq!(c.control_slots, 1);
    assert_eq!(c.loyalty, MAX_LOYALTY);
    assert_eq!(c.physical_damage, 100);
    assert_eq!(c.range_home, 10);
    assert_eq!(c.fight_mode, FightMode::Closest);
    assert_eq!(c.active_speed, 0.3);
    assert_eq!(c.passive_speed, 0.6);
}

#[test]
fn move_speed_overrides_survive_serialization() {
    for overridden in [true, false] {
        let mut w = world();
        let bc = creature(&mut w);
        if overridden {
            speeds::set_move_speed(&mut w, bc, 0.45, 0.9);
        }

        // `load` checks the exact consumption: the BaseCreature tail is the last block
        let b = snapshot(&w, bc);
        let copy = load(&mut w, &b);

        let c = d(&w, copy);
        assert_eq!(c.active_move_speed, if overridden { 0.45 } else { 0.0 });
        assert_eq!(c.passive_move_speed, if overridden { 0.9 } else { 0.0 });
    }
}

// ---- read_creature fixes -----------------------------------------------------------------------------------

static TEACHER_C: CreatureType = CreatureType {
    can_teach: |_, _| true,
    ..BASE_CREATURE_C
};
static TEACHER: MobileType = MobileType {
    full_name: "Server.Mobiles.TeacherStub",
    parent: Some(&BASE_CREATURE),
    family: Some(&TEACHER_C),
    ..BASE_CREATURE
};

#[test]
fn teaching_stealth_needs_the_eras_hiding_requirement() {
    use muo_data::expansion::Expansion;
    use muo_server::skills::{self, SkillName};
    let mut w = world();
    let bc = new_creature(
        &mut w,
        &TEACHER,
        AIType::AiAnimal,
        FightMode::Closest,
        16,
        1,
    );
    let from = plain(&mut w);
    let mut sk = skills::Skills::default();
    sk.ensure(64);
    w.mobile_mut(from).unwrap().skills = Some(sk);
    w.mobile_mut(from)
        .unwrap()
        .skills
        .as_mut()
        .unwrap()
        .get(SkillName::Hiding as usize)
        .unwrap()
        .base = 500; // 50.0
    let stealth = SkillName::Stealth as i32;
    let check = super::misc::check_teach;
    w.core.expansion = Some(Expansion::UOR); // Stealth.HidingRequirement 80
    assert!(!check(&w, bc, stealth, from));
    w.core.expansion = Some(Expansion::SE); // 50
    assert!(check(&w, bc, stealth, from));
    w.core.expansion = Some(Expansion::ML); // 30
    assert!(check(&w, bc, stealth, from));
}

#[test]
fn abandoned_pet_only_despawns_when_its_last_owner_still_exists() {
    let mut w = world();
    let stray = creature(&mut w);
    cd_mut(&mut w, stray).unwrap().owners =
        vec![MobileId(muo_primitives::Serial::new(0x3FFF_FF00))];
    super::serialize::after_deserialization(&mut w, stray);
    assert!(
        !d(&w, stray).pending_delete_timer.running(&w),
        "a vanished owner is LastOwner == null"
    );
    let kept = creature(&mut w);
    let owner = plain(&mut w);
    cd_mut(&mut w, kept).unwrap().owners = vec![owner];
    super::serialize::after_deserialization(&mut w, kept);
    assert!(d(&w, kept).pending_delete_timer.running(&w));
}

static HERO_C: CreatureType = CreatureType {
    ethic_allegiance: |_, _| Some(0),
    ..BASE_CREATURE_C
};
static HERO: MobileType = MobileType {
    full_name: "Server.Mobiles.HeroStub",
    parent: Some(&BASE_CREATURE),
    family: Some(&HERO_C),
    ..BASE_CREATURE
};

#[test]
fn a_shield_only_spares_creatures_of_the_players_ethic() {
    let mut w = world();
    // The hook stands for "shielded Hero player": it vetoes only a creature with no ethic or a Hero.
    crate::hooks::hooks_mut(&mut w).ethic_shielded =
        Some(|_, _, ours| ours.is_none() || ours == Some(0));
    let target = plain(&mut w);
    let neutral = creature(&mut w);
    let hero = new_creature(&mut w, &HERO, AIType::AiAnimal, FightMode::Closest, 16, 1);
    assert!(!super::misc::is_enemy(&mut w, neutral, target));
    assert!(!super::misc::is_enemy(&mut w, hero, target));
    crate::hooks::hooks_mut(&mut w).ethic_shielded = Some(|_, _, ours| ours == Some(1));
    assert!(
        super::misc::is_enemy(&mut w, hero, target),
        "an Evil shield doesn't spare a Hero creature"
    );
}
