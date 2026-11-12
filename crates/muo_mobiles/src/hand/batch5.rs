//! BakeKitsune, BogThing, CrystalLatticeSeeker (monsters5).

use muo_content::creature::BASE_CREATURE_C;
use muo_content::factory;
use muo_content::mx;
use muo_io::{GenericReader, GenericWriter};
use muo_primitives::{Body, TimeSpan};
use muo_server::World;
use muo_server::ids::MobileId;
use muo_server::layer::Layer;
use muo_server::mobile::part1::{props_a, props_b};
use muo_server::mobile::part2::{enumerators, items, messages};
use muo_server::timer::{self, TimerHandle};

use super::ctors::add_hue;
use crate::common::*;

// ---- BakeKitsune ----

/// `_disguiseTimerToken` (not saved).
#[derive(Clone, Copy, Default)]
struct DisguiseToken(TimerHandle);

fn disguise_token(w: &World, id: MobileId) -> TimerHandle {
    w.mobile(id)
        .and_then(|m| m.ext.get::<DisguiseToken>())
        .map_or_else(TimerHandle::default, |t| t.0)
}
fn set_disguise_token(w: &mut World, id: MobileId, h: TimerHandle) {
    if let Some(m) = w.mobile_mut(id) {
        m.ext.insert(DisguiseToken(h));
    }
}
fn cancel_disguise_token(w: &mut World, id: MobileId) {
    let mut h = disguise_token(w, id);
    h.cancel(w);
    set_disguise_token(w, id, h);
}

/// `if (Utility.RandomDouble() < .25) PackItem(Seed.RandomBonsaiSeed());`
pub fn bake_kitsune_ctor1(w: &mut World, id: MobileId) {
    if rnd::double(w) < 0.25 {
        let s = super::seed::random_bonsai_seed(w);
        pack_item(w, id, s);
    }
}

/// `OnCombatantChange`: no `base` call. After a fight, half the time it disguises in 15-30 seconds.
pub fn bake_kitsune_on_combatant_change(w: &mut World, id: MobileId) {
    let controlled = muo_content::creature::cd(w, id).is_some_and(|d| d.controlled);
    let body_mod = w.mobile(id).is_some_and(|m| m.body_mod.0 != 0);
    if mx::combatant(w, id).is_none()
        && !body_mod
        && !controlled
        && !disguise_token(w, id).running(w)
        && rnd::bool(w)
    {
        let secs = rnd::min_max(w, 15, 30);
        let h = timer::start(
            w,
            TimeSpan::from_secs(i64::from(secs)),
            TimeSpan::ZERO,
            1,
            move |w| bake_kitsune_disguise(w, id),
        );
        set_disguise_token(w, id, h);
    }
}

/// `RemoveDisguise(); return base.OnBeforeDeath();`
pub fn bake_kitsune_on_before_death(w: &mut World, id: MobileId) -> bool {
    bake_kitsune_remove_disguise(w, id);
    (muo_content::creature::BASE_CREATURE.on_before_death)(w, id)
}

pub fn bake_kitsune_serialize(_w: &World, _id: MobileId, out: &mut dyn GenericWriter) {
    out.write_i32(0);
}

/// `[AfterDeserialization]`: resistances rolled before the 60 cap come back in range, and the disguise is stripped.
pub fn bake_kitsune_deserialize(
    w: &mut World,
    id: MobileId,
    r: &mut dyn GenericReader,
) -> muo_io::Result<()> {
    let _version = r.read_i32()?;
    if muo_server::mobile::base::physical_resistance(w, id) > 60 {
        set_resistance_range(w, id, ResistanceType::Physical, 40, 60);
        set_resistance_range(w, id, ResistanceType::Fire, 70, 90);
        set_resistance_range(w, id, ResistanceType::Cold, 40, 60);
        set_resistance_range(w, id, ResistanceType::Poison, 40, 60);
        set_resistance_range(w, id, ResistanceType::Energy, 40, 60);
    }
    let h = timer::start(w, TimeSpan::ZERO, TimeSpan::ZERO, 1, move |w| {
        bake_kitsune_remove_disguise(w, id)
    });
    set_disguise_token(w, id, h);
    Ok(())
}

pub fn bake_kitsune_disguise(w: &mut World, id: MobileId) {
    if mx::deleted(w, id) {
        return;
    }
    let controlled = muo_content::creature::cd(w, id).is_some_and(|d| d.controlled);
    let body_mod = w.mobile(id).is_some_and(|m| m.body_mod.0 != 0);
    if mx::combatant(w, id).is_some() || body_mod || controlled {
        return;
    }
    mx::fixed_effect(w, id, 0x376A, 8, 32);
    mx::play_sound(w, id, 0x1FE);

    let female = rnd::bool(w);
    props_b::set_female(w, id, female);
    if female {
        props_b::set_body_mod(w, id, Body(0x191));
        name_random(w, id, "female");
    } else {
        props_b::set_body_mod(w, id, Body(0x190));
        name_random(w, id, "male");
    }
    props_a::set_title(w, id, Some("the mystic llama herder".to_owned()));
    // Race.Human
    let ops = w
        .resources
        .get::<muo_server::race::Races>()
        .and_then(|r| r.default_race())
        .map(|r| r.ops);
    if let Some(ops) = ops {
        let skin = (ops.random_skin_hue)(&mut w.rng);
        hue(w, id, skin);
        let hair = (ops.random_hair)(&mut w.rng, female);
        props_b::set_hair_item_id(w, id, hair);
        let hair_hue = (ops.random_hair_hue)(&mut w.rng);
        props_b::set_hair_hue(w, id, hair_hue);
        let facial = (ops.random_facial_hair)(&mut w.rng, female);
        props_b::set_facial_hair_item_id(w, id, facial);
        props_b::set_facial_hair_hue(w, id, hair_hue);
    }
    let shoe = ["Shoes", "Boots", "Sandals", "ThighBoots"][rnd::below(w, 4) as usize];
    let hu = muo_primitives::utility::random_neutral_hue(&mut w.rng);
    add_hue(w, id, shoe, hu);
    let hu = muo_primitives::utility::random_nondyed_hue(&mut w.rng);
    add_hue(w, id, "Robe", hu);

    cancel_disguise_token(w, id);
    let h = timer::start(w, TimeSpan::from_secs(75), TimeSpan::ZERO, 1, move |w| {
        bake_kitsune_remove_disguise(w, id)
    });
    set_disguise_token(w, id, h);
}

pub fn bake_kitsune_remove_disguise(w: &mut World, id: MobileId) {
    if mx::deleted(w, id) {
        return;
    }
    cancel_disguise_token(w, id);
    // BodyMod is not saved, so after a restart a disguised kitsune comes back without it but still wearing the disguise.
    name(w, id, None);
    props_a::set_title(w, id, None);
    props_b::set_body_mod(w, id, Body(0));
    hue(w, id, 0);
    props_b::set_hair_item_id(w, id, 0);
    props_b::set_hair_hue(w, id, 0);
    props_b::set_facial_hair_item_id(w, id, 0);
    props_b::set_facial_hair_hue(w, id, 0);
    for layer in [Layer::OuterTorso, Layer::Shoes] {
        if let Some(i) = items::find_item_on_layer(w, id, layer) {
            muo_server::item::base::delete(w, i);
        }
    }
}

// ---- BogThing ----

/// `SpawnBogling(m)`: a bogling of the same team near it fights `m`.
pub fn bog_thing_spawn_bogling(w: &mut World, id: MobileId, m: MobileId) {
    let Some(map) = mx::map(w, id) else { return };
    let Some(spawned) = factory::make_mobile(w, "Server.Mobiles.Bogling") else {
        return;
    };
    let team = muo_content::creature::cd(w, id).map_or(0, |d| d.team);
    muo_content::creature::control::set_team(w, spawned, team);
    let here = mx::location(w, id);
    let loc =
        muo_server::map::fit::get_random_nearby_location(w, map, here, 2, 0, 10, 16, false, false);
    muo_server::mobile::base::move_to_world(w, spawned, loc, map);
    mx::set_combatant(w, spawned, Some(m));
}

/// `EatBoglings()`: boglings within 2 tiles heal it by half their hits and die.
pub fn bog_thing_eat_boglings(w: &mut World, id: MobileId) {
    if mx::hits(w, id) >= mx::hits_max(w, id) {
        return;
    }
    let mut queue = vec![];
    for b in enumerators::get_mobiles_in_range(w, id, 2) {
        if !factory::mobile_is(w, b, "Server.Mobiles.Bogling") {
            continue;
        }
        if mx::hits(w, id) >= mx::hits_max(w, id) {
            break;
        }
        let h = mx::hits(w, id) + mx::hits(w, b) / 2;
        props_b::set_hits(w, id, h);
        queue.push(b);
    }
    if queue.is_empty() {
        return;
    }
    let s = 0x3B + rnd::below(w, 2);
    mx::play_sound(w, id, s);
    for b in queue {
        muo_server::mobile::base::delete(w, b);
    }
}

pub fn bog_thing_on_got_melee_attack(w: &mut World, id: MobileId, attacker: MobileId, damage: i32) {
    (BASE_CREATURE_C.on_got_melee_attack)(w, id, attacker, damage);
    if rnd::double(w) < 0.25 {
        if mx::hits(w, id) > mx::hits_max(w, id) / 4 {
            bog_thing_spawn_bogling(w, id, attacker);
        } else {
            bog_thing_eat_boglings(w, id);
        }
    }
}

// ---- CrystalLatticeSeeker ----

/// `Drain(m)`: steals hits, stamina or mana from `m`.
pub fn crystal_lattice_seeker_drain(w: &mut World, id: MobileId, m: MobileId) {
    match rnd::below(w, 3) {
        0 => {
            messages::say_number(w, id, 1042156, "");
            mx::play_sound(w, id, 0x1E6);
            let d = rnd::min_max(w, 3, 6);
            let h = mx::hits(w, id) + d;
            props_b::set_hits(w, id, h);
            let h = mx::hits(w, m) - d;
            props_b::set_hits(w, m, h);
        }
        1 => {
            messages::say_number(w, id, 1042157, "");
            mx::play_sound(w, id, 0x1DF);
            let d = rnd::min_max(w, 10, 25);
            let s = w.mobile(id).map_or(0, |x| x.stam) + d;
            props_b::set_stam(w, id, s);
            let s = w.mobile(m).map_or(0, |x| x.stam) - d;
            props_b::set_stam(w, m, s);
        }
        _ => {
            messages::say_number(w, id, 1042155, "");
            mx::play_sound(w, id, 0x1F8);
            let d = rnd::min_max(w, 15, 25);
            let s = w.mobile(id).map_or(0, |x| x.mana) + d;
            props_b::set_mana(w, id, s);
            let s = w.mobile(m).map_or(0, |x| x.mana) - d;
            props_b::set_mana(w, m, s);
        }
    }
}

pub fn crystal_lattice_seeker_on_gave_melee_attack(
    w: &mut World,
    id: MobileId,
    defender: MobileId,
    damage: i32,
) {
    (BASE_CREATURE_C.on_gave_melee_attack)(w, id, defender, damage);
    if rnd::double(w) < 0.1 {
        crystal_lattice_seeker_drain(w, id, defender);
    }
}

pub fn crystal_lattice_seeker_on_got_melee_attack(
    w: &mut World,
    id: MobileId,
    attacker: MobileId,
    damage: i32,
) {
    (BASE_CREATURE_C.on_got_melee_attack)(w, id, attacker, damage);
    if rnd::double(w) < 0.1 {
        crystal_lattice_seeker_drain(w, id, attacker);
    }
}
