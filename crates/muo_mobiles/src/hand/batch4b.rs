//! Swoop, KazeKemono, LadyOfTheSnow, Yamandon: the melee-attack and counter specials of the SE/ML monsters.

use std::collections::HashMap;

use muo_content::creature::{BASE_CREATURE_C, cd, control};
use muo_content::misc::aos_helpers::{self, DamageOptions, DamageSplit};
use muo_content::mx;
use muo_primitives::TimeSpan;
use muo_server::World;
use muo_server::ids::MobileId;
use muo_server::mobile::ResistanceType;
use muo_server::mods::ResistanceMod;
use muo_server::timer::{self, TimerHandle};

use crate::common::*;

type Table = HashMap<MobileId, TimerHandle>;

fn physical_resistance(w: &mut World, m: MobileId) -> i32 {
    w.mobile(m)
        .map(|x| x.ty)
        .map_or(0, |t| (t.physical_resistance)(w, m))
}
fn energy_resistance(w: &mut World, m: MobileId) -> i32 {
    w.mobile(m)
        .map(|x| x.ty)
        .map_or(0, |t| (t.energy_resistance)(w, m))
}

/// `timer.DoExpire()` of an `ExpireTimer` holding a resistance mod: the mod goes, the timer stops.
fn expire_mod(w: &mut World, mut timer: TimerHandle, m: MobileId, mod_name: &str) {
    muo_server::mobile::base::remove_resistance_mod_by_name(w, m, mod_name);
    timer.cancel(w);
}

/// The weakening blow shared by Swoop (`PhysicalResistGraspingClaw`) and Kaze Kemono: the old effect of `table`
/// ends (`again` is its message, `first` the start message), the new mod lasts `secs` and says `end_msg` when it ends.
#[allow(clippy::too_many_arguments)]
fn weaken(
    w: &mut World,
    table: fn(&mut MobileTables) -> &mut Table,
    defender: MobileId,
    ty: ResistanceType,
    mod_name: &'static str,
    effect: impl FnOnce(&mut World, MobileId) -> i32,
    remove_old: bool,
    first: i32,
    again: i32,
    secs: i64,
    end_msg: i32,
) {
    let old = if remove_old {
        table(tables(w)).remove(&defender)
    } else {
        table(tables(w)).get(&defender).copied()
    };
    if let Some(t) = old {
        expire_mod(w, t, defender, mod_name);
        mx::msg(w, defender, again);
    } else {
        mx::msg(w, defender, first);
    }
    let offset = effect(w, defender);
    let m = ResistanceMod::new(w, ty, Some(mod_name), offset, None);
    mx::fixed_effect(w, defender, 0x37B9, 10, 5);
    muo_server::mobile::base::add_resistance_mod(w, defender, m);
    let h = timer::start(w, TimeSpan::from_secs(secs), TimeSpan::ZERO, 1, move |w| {
        mx::msg(w, defender, end_msg);
        muo_server::mobile::base::remove_resistance_mod_by_name(w, defender, mod_name);
        table(tables(w)).remove(&defender);
    });
    table(tables(w)).insert(defender, h);
}

// ---- Swoop ----

/// `PackArcaneScroll(0, 1)`.
pub fn swoop_ctor1(w: &mut World, id: MobileId) {
    pack_arcane_scroll_range(w, id, 0, 1);
}

/// `OnGaveMeleeAttack`: 10% of the blows make the defender 15% more susceptible to physical damage for 5 seconds.
pub fn swoop_on_gave_melee_attack(w: &mut World, id: MobileId, defender: MobileId, damage: i32) {
    (BASE_CREATURE_C.on_gave_melee_attack)(w, id, defender, damage);
    if rnd::double(w) < 0.9 {
        return;
    }
    weaken(
        w,
        |t| &mut t.swoop,
        defender,
        ResistanceType::Physical,
        "PhysicalResistGraspingClaw",
        |w, d| -(physical_resistance(w, d) * 15 / 100),
        true,
        1070836, // The blow from the creature's claws has made you more susceptible to physical attacks.
        1070837, // The creature lands another blow in your weakened state.
        5,
        1070838, // Your resistance to physical attacks has returned.
    );
}

// ---- KazeKemono ----

/// Flurry of Twigs (10%, physical -15% for 5 seconds), else Chlorophyll Blast (5%, energy -50% for 10 seconds).
pub fn kaze_kemono_on_gave_melee_attack(
    w: &mut World,
    id: MobileId,
    defender: MobileId,
    damage: i32,
) {
    (BASE_CREATURE_C.on_gave_melee_attack)(w, id, defender, damage);
    if rnd::double(w) < 0.1 {
        weaken(
            w,
            |t| &mut t.flurry,
            defender,
            ResistanceType::Physical,
            "PhysicalResistFlurryOfTwigs",
            |w, d| -(physical_resistance(w, d) * 15 / 100),
            true,
            1070850, // The creature's flurry of twigs has made you more susceptible to physical attacks!
            1070851, // The creature lands another blow in your weakened state.
            5,
            1070852, // Your resistance to physical attacks has returned.
        );
        return;
    }
    if rnd::double(w) < 0.05 {
        weaken(
            w,
            |t| &mut t.chlorophyll,
            defender,
            ResistanceType::Energy,
            "EnergyResistChlorophyllBlast",
            |w, d| -(energy_resistance(w, d) / 2),
            false,
            1070827, // The creature's attack has made you more susceptible to energy attacks!
            1070828, // The creature continues to hinder your energy resistance!
            10,
            1070829, // Your resistance to energy attacks has returned.
        );
    }
}

// ---- LadyOfTheSnow ----

/// `if (Utility.RandomDouble() < 0.25) PackItem(Seed.RandomBonsaiSeed());`
pub fn lady_of_the_snow_ctor1(w: &mut World, id: MobileId) {
    if rnd::double(w) < 0.25 {
        let s = super::seed::random_bonsai_seed(w);
        pack_item(w, id, s);
    }
}

fn expire_snow(w: &mut World, defender: MobileId) {
    if let Some(mut h) = tables(w).snow.remove(&defender) {
        h.cancel(w);
    }
}

/// Cold Wind: 10% of the blows freeze the defender's lungs, 2 damage a second for 5 seconds.
pub fn lady_of_the_snow_on_gave_melee_attack(
    w: &mut World,
    id: MobileId,
    defender: MobileId,
    damage: i32,
) {
    (BASE_CREATURE_C.on_gave_melee_attack)(w, id, defender, damage);
    if rnd::double(w) < 0.9 {
        return;
    }
    if tables(w).snow.contains_key(&defender) {
        expire_snow(w, defender); // timer.DoExpire()
        mx::msg(w, defender, 1070831); // The freezing wind continues to blow!
    } else {
        mx::msg(w, defender, 1070832); // An icy wind surrounds you, freezing your lungs as you breathe!
    }
    let mut count = 0;
    let h = timer::start(
        w,
        TimeSpan::from_secs(1),
        TimeSpan::from_secs(1),
        0,
        move |w| {
            // DrainLife()
            if mx::alive(w, defender) {
                mx::damage(w, defender, 2, Some(id));
            } else {
                expire_snow(w, defender);
                return;
            }
            count += 1;
            if count >= 5 {
                expire_snow(w, defender);
                mx::msg(w, defender, 1070830); // The icy wind dissipates.
            }
        },
    );
    tables(w).snow.insert(defender, h);
}

// ---- Yamandon ----

/// `if (Utility.RandomDouble() < .50) PackItem(Seed.RandomBonsaiSeed());`
pub fn yamandon_ctor1(w: &mut World, id: MobileId) {
    if rnd::double(w) < 0.5 {
        let s = super::seed::random_bonsai_seed(w);
        pack_item(w, id, s);
    }
}
/// `PackItem(new Eggs(2))`.
pub fn yamandon_ctor2(w: &mut World, id: MobileId) {
    pack_new_amount(w, id, &super::ctors::si("Eggs"), 2);
}

pub fn yamandon_on_damaged_by_spell(w: &mut World, id: MobileId, attacker: MobileId, damage: i32) {
    (BASE_CREATURE_C.on_damaged_by_spell)(w, id, attacker, damage);
    yamandon_do_counter(w, id, attacker);
}
pub fn yamandon_on_got_melee_attack(w: &mut World, id: MobileId, attacker: MobileId, damage: i32) {
    (BASE_CREATURE_C.on_got_melee_attack)(w, id, attacker, damage);
    yamandon_do_counter(w, id, attacker);
}

/// `DoCounter(attacker)`: 20% of the time a poison burst (20-25 unresistable, lethal poison) around the attacker (or
/// its master when near).
fn yamandon_do_counter(w: &mut World, id: MobileId, attacker: MobileId) {
    if mx::map(w, id).is_none() {
        return;
    }
    let attacker_is_creature = cd(w, attacker).is_some();
    if cd(w, attacker).is_some_and(|d| d.bard_provoked) {
        return;
    }
    if rnd::double(w) >= 0.2 {
        return;
    }
    let mut target = None;
    if attacker_is_creature {
        target = control::get_master(w, attacker);
    }
    if !target.is_some_and(|t| mx::in_range(w, t, id, 18)) {
        target = Some(attacker);
    }
    let target = target.unwrap_or(attacker);
    mx::animate(w, id, 10, 4, 1, true, false, 0);
    let my_team = cd(w, id).map_or(0, |d| d.team);
    let mut queue = Vec::new();
    for m in muo_server::mobile::part2::enumerators::get_mobiles_in_range(w, target, 8) {
        if m == id {
            continue;
        }
        let player_alive = mx::player(w, m) && mx::alive(w, m);
        if !(mx::can_be_harmful(w, id, m) || player_alive) {
            continue;
        }
        if !cd(w, m).is_some_and(|d| d.controlled || d.summoned || d.team != my_team) {
            continue;
        }
        queue.push(m);
    }
    for m in queue {
        mx::do_harmful(w, id, m);
        let amount = rnd::min_max(w, 20, 25);
        aos_helpers::damage(
            w,
            m,
            Some(id),
            amount,
            DamageSplit {
                phys: 0,
                fire: 0,
                cold: 0,
                pois: 100,
                nrgy: 0,
            },
            DamageOptions {
                ignore_armor: true,
                ..Default::default()
            },
        );
        muo_server::mobile::part2::effects::fixed_particles_hue(
            w,
            m,
            0x36BD,
            1,
            10,
            0x1F78,
            0xA6,
            0,
            muo_server::effects::EffectLayer::None,
        );
        muo_server::mobile::base::apply_poison(w, m, Some(id), muo_server::poison::PoisonId(4));
    }
}
