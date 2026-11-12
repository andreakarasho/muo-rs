//! PlagueBeast (monsters5): devours the corpses it killed, grows, spawns PlagueSpawn when hit.

use muo_content::creature::{BASE_CREATURE_C, cd, cd_mut};
use muo_content::factory;
use muo_content::items::corpse;
use muo_content::mx;
use muo_io::{GenericReader, GenericWriter};
use muo_primitives::MessageType;
use muo_server::World;
use muo_server::effects::EffectLayer;
use muo_server::ids::{ItemId, MobileId};
use muo_server::mobile::part1::props_b;
use muo_server::mobile::part2::{effects, enumerators, messages};

use crate::common::*;

/// `_hasMetalChest`, `_totalDevoured`, `_devourGoal`.
#[derive(Clone, Copy, Default)]
pub struct PlagueBeastData {
    pub has_metal_chest: bool,
    pub total_devoured: i32,
    pub devour_goal: i32,
}

fn data(w: &World, id: MobileId) -> PlagueBeastData {
    w.mobile(id)
        .and_then(|m| m.ext.get::<PlagueBeastData>())
        .copied()
        .unwrap_or_default()
}
fn update(w: &mut World, id: MobileId, f: impl FnOnce(&mut PlagueBeastData)) {
    let mut d = data(w, id);
    f(&mut d);
    if let Some(m) = w.mobile_mut(id) {
        m.ext.insert(d);
    }
}

fn is_paragon_now(w: &World, id: MobileId) -> bool {
    cd(w, id).is_some_and(|d| d.is_paragon)
}

/// `DevourGoal`.
#[allow(dead_code)] // no caller reaches it yet
pub fn plague_beast_devour_goal(w: &World, id: MobileId) -> i32 {
    let g = data(w, id).devour_goal;
    if is_paragon_now(w, id) { g + 25 } else { g }
}

pub fn plague_beast_serialize(w: &World, id: MobileId, out: &mut dyn GenericWriter) {
    let d = data(w, id);
    out.write_i32(0);
    out.write_bool(d.has_metal_chest);
    out.write_i32(d.total_devoured);
    out.write_i32(d.devour_goal);
}

pub fn plague_beast_deserialize(
    w: &mut World,
    id: MobileId,
    r: &mut dyn GenericReader,
) -> muo_io::Result<()> {
    let _version = r.read_i32()?;
    let has_metal_chest = r.read_bool()?;
    let total_devoured = r.read_i32()?;
    let devour_goal = r.read_i32()?;
    if let Some(m) = w.mobile_mut(id) {
        m.ext.insert(PlagueBeastData {
            has_metal_chest,
            total_devoured,
            devour_goal,
        });
    }
    Ok(())
}

/// `PackArmor(1, 5);`
pub fn plague_beast_ctor1(w: &mut World, id: MobileId) {
    pack_armor(w, id, 1, 5, 1.0);
}

/// `if (Core.ML && Utility.RandomDouble() < 0.33) PackItem(Seed.RandomPeculiarSeed(4));`
pub fn plague_beast_ctor2(w: &mut World, id: MobileId) {
    if w.core_ml() && rnd::double(w) < 0.33 {
        let s = super::seed::random_peculiar_seed(w, 4);
        pack_item(w, id, s);
    }
}

/// `_totalDevoured = 0;`
pub fn plague_beast_ctor3(w: &mut World, id: MobileId) {
    update(w, id, |d| d.total_devoured = 0);
}

/// `_devourGoal = Utility.RandomMinMax(15, 25);`
pub fn plague_beast_ctor4(w: &mut World, id: MobileId) {
    let g = rnd::min_max(w, 15, 25);
    update(w, id, |d| d.devour_goal = g);
}

/// `IncreaseHits(hp)`.
fn increase_hits(w: &mut World, id: MobileId, mut hp: i32) {
    let mut maxhits = 2000;
    if is_paragon_now(w, id) {
        let buff = muo_content::hooks::hooks(w)
            .and_then(|h| h.paragon_hits_buff)
            .unwrap_or(5.0);
        maxhits = (f64::from(maxhits) * buff) as i32;
    }
    if hp < 1000 && !w.core_aos() {
        hp = hp * 100 / 60;
    }
    let seed = cd(w, id).map_or(-1, |d| d.hits_max_seed);
    if seed >= maxhits {
        if let Some(d) = cd_mut(w, id) {
            d.hits_max_seed = maxhits;
        }
        let new_hits = mx::hits(w, id) + hp + rnd::min_max(w, 10, 20);
        props_b::set_hits(w, id, maxhits.min(new_hits));
    } else {
        let (min, max) = (hp / 2 + 10, hp + 20);
        let inc = rnd::min_max(w, min, max);
        if let Some(d) = cd_mut(w, id) {
            d.hits_max_seed += inc;
        }
        let h = mx::hits(w, id) + inc;
        props_b::set_hits(w, id, h);
    }
}

/// `Devour(corpse)`: false when the corpse has no owner.
pub fn plague_beast_devour(w: &mut World, id: MobileId, c: ItemId) -> bool {
    let Some(owner) = corpse::owner(w, c) else {
        return false;
    };
    if muo_content::creature::misc::body_is_human(w, owner) {
        corpse::turn_to_bones(w, c);
    }
    let hits_max = muo_server::mobile::base::hits_max(w, owner);
    increase_hits(w, id, (f64::from(hits_max) * 0.75).ceil() as i32);
    update(w, id, |d| d.total_devoured += 1);
    // * The plague beast absorbs the fleshy remains of the corpse *
    messages::public_overhead_message_number(w, id, MessageType::EMOTE, 0x3B2, 1053033, "", false);
    let d = data(w, id);
    if !d.has_metal_chest && d.total_devoured >= d.devour_goal {
        pack_new(w, id, "Server.Items.MetalChest");
        update(w, id, |d| d.has_metal_chest = true);
    }
    true
}

pub fn plague_beast_on_gave_melee_attack(
    w: &mut World,
    id: MobileId,
    defender: MobileId,
    damage: i32,
) {
    (BASE_CREATURE_C.on_gave_melee_attack)(w, id, defender, damage);
    // Poison.Lethal / Poison.Deadly
    let p = if is_paragon_now(w, id) { 4 } else { 3 };
    muo_server::mobile::base::apply_poison(w, defender, Some(id), muo_server::poison::PoisonId(p));
    effects::fixed_particles(w, defender, 0x374A, 10, 15, 5021, EffectLayer::Waist);
    mx::play_sound(w, defender, 0x1CB);
}

fn spawn_plague_spawn(w: &mut World, id: MobileId, against: MobileId) {
    let spawn = crate::classes::misc4::new_plague_spawn(w, Some(id));
    let team = cd(w, id).map_or(0, |d| d.team);
    muo_content::creature::control::set_team(w, spawn, team);
    if let Some(map) = mx::map(w, id) {
        let loc = mx::location(w, id);
        muo_server::mobile::base::move_to_world(w, spawn, loc, map);
    }
    mx::set_combatant(w, spawn, Some(against));
    messages::say_number(w, id, 1053034, ""); // * The plague beast creates another beast from its flesh! *
}

pub fn plague_beast_on_damaged_by_spell(
    w: &mut World,
    id: MobileId,
    caster: MobileId,
    damage: i32,
) {
    if mx::map(w, id).is_some() && caster != id && rnd::double(w) < 0.25 {
        spawn_plague_spawn(w, id, caster);
    }
    (BASE_CREATURE_C.on_damaged_by_spell)(w, id, caster, damage);
}

pub fn plague_beast_on_got_melee_attack(
    w: &mut World,
    id: MobileId,
    attacker: MobileId,
    damage: i32,
) {
    if mx::map(w, id).is_some() && attacker != id && rnd::double(w) < 0.25 {
        spawn_plague_spawn(w, id, attacker);
    }
    (BASE_CREATURE_C.on_got_melee_attack)(w, id, attacker, damage);
}

pub fn plague_beast_on_think(w: &mut World, id: MobileId) {
    (BASE_CREATURE_C.on_think)(w, id);
    for item in enumerators::get_items_in_range(w, id, 3) {
        if !factory::item_is(w, item, "Server.Items.Corpse") {
            continue;
        }
        if corpse::killer(w, item) == Some(id)
            && corpse::owner(w, item).is_some()
            && !corpse::devour_corpse(w, item)
            && !corpse::devoured(w, item)
        {
            // * The plague beast attempts to absorb the remains, but cannot! *
            messages::public_overhead_message_number(
                w,
                id,
                MessageType::EMOTE,
                0x3B2,
                1053032,
                "",
                false,
            );
        }
    }
}

/// `IDevourer.Devour` for `Corpse.DevourCorpse`: true only for a plague beast killer.
pub fn devour_hook(w: &mut World, killer: MobileId, c: ItemId) -> bool {
    factory::mobile_is(w, killer, "Server.Mobiles.PlagueBeast") && plague_beast_devour(w, killer, c)
}
