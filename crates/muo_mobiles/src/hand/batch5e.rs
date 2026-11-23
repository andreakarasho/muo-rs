//! EtherealWarrior (monsters5).

use muo_content::creature::BASE_CREATURE_C;
use muo_content::mx;
use muo_content::player_mobile;
use muo_content::vendor;
use muo_primitives::{DateTime, Point3D, TimeSpan};
use muo_server::World;
use muo_server::ids::MobileId;
use muo_server::mobile::part1::props_b;

use crate::common::*;

#[derive(Clone, Copy)]
struct NextResurrect(DateTime);

/// `OnMovement(from, oldLocation)`: no `base` call.
pub fn ethereal_warrior_on_movement(w: &mut World, id: MobileId, from: MobileId, old: Point3D) {
    if mx::alive(w, from) || player_mobile::data(w, from).is_none() {
        return;
    }
    let next = w
        .mobile(id)
        .and_then(|m| m.ext.get::<NextResurrect>())
        .map_or(DateTime::MIN, |n| n.0);
    let frozen = w.mobile(from).is_some_and(|m| m.frozen);
    if !frozen
        && w.now >= next
        && mx::in_range(w, id, from, 4)
        && !mx::in_range_point(w, id, old, 4)
        && mx::in_los(w, id, from)
    {
        let until = w.now + TimeSpan::from_secs(2);
        if let Some(m) = w.mobile_mut(id) {
            m.ext.insert(NextResurrect(until));
        }
        let (criminal, karma) = w.mobile(from).map_or((false, 0), |m| (m.criminal, m.karma));
        let murderer = w.mobile(from).is_some_and(|m| (m.ty.murderer)(w, from));
        if !criminal && !murderer && karma > 0 {
            let (loc, map) = (mx::location(w, from), mx::map(w, from));
            if map.is_some_and(|map| {
                muo_server::map::fit::can_fit(w, map, loc.x, loc.y, loc.z, 16, false, false, true)
            }) {
                let d = mx::get_direction_to(w, id, from);
                mx::set_direction(w, id, d);
                mx::play_sound(w, from, 0x1F2);
                mx::fixed_effect(w, from, 0x376A, 10, 16);
                if let Some(f) = vendor::hooks(w).and_then(|h| h.resurrect_gump) {
                    f(w, from, id, 0); // new ResurrectGump(from, ResurrectMessage.Healer)
                }
            }
        }
    }
}

/// `Damage(Utility.Random(10, 10), this); Stam -= Utility.Random(10, 10); Mana -= Utility.Random(10, 10);`
fn zap(w: &mut World, id: MobileId, other: MobileId) {
    let d = rnd::min_max(w, 10, 19);
    mx::damage(w, other, d, Some(id));
    let n = rnd::min_max(w, 10, 19);
    let s = w.mobile(other).map_or(0, |m| m.stam) - n;
    props_b::set_stam(w, other, s);
    let n = rnd::min_max(w, 10, 19);
    let s = w.mobile(other).map_or(0, |m| m.mana) - n;
    props_b::set_mana(w, other, s);
}

pub fn ethereal_warrior_on_gave_melee_attack(
    w: &mut World,
    id: MobileId,
    defender: MobileId,
    damage: i32,
) {
    (BASE_CREATURE_C.on_gave_melee_attack)(w, id, defender, damage);
    zap(w, id, defender);
}

pub fn ethereal_warrior_on_got_melee_attack(
    w: &mut World,
    id: MobileId,
    attacker: MobileId,
    damage: i32,
) {
    (BASE_CREATURE_C.on_got_melee_attack)(w, id, attacker, damage);
    zap(w, id, attacker);
}

/// `OnDamage(amount, from, willKill)` of InterredGrizzle: a pool of ooze now and then (the code is Ilhenir's).
pub fn interred_grizzle_on_damage(
    w: &mut World,
    id: MobileId,
    amount: i32,
    from: Option<MobileId>,
    will_kill: bool,
) {
    crate::classes::ilhenir::interred_grizzle_on_damage(w, id, amount, from, will_kill);
}

// ---- Protector ----

fn human_ops(w: &World) -> Option<&'static muo_server::race::RaceOps> {
    w.resources
        .get::<muo_server::race::Races>()
        .and_then(|r| r.default_race())
        .map(|r| r.ops)
}

/// `HairItemID = Race.Human.RandomHair(this);`
pub fn protector_hair(w: &mut World, id: MobileId) {
    let female = w.mobile(id).is_some_and(|m| m.female);
    if let Some(ops) = human_ops(w) {
        let h = (ops.random_hair)(&mut w.rng, female);
        props_b::set_hair_item_id(w, id, h);
    }
}

/// `HairHue = Race.Human.RandomHairHue();`
pub fn protector_hair_hue(w: &mut World, id: MobileId) {
    if let Some(ops) = human_ops(w) {
        let h = (ops.random_hair_hue)(&mut w.rng);
        props_b::set_hair_hue(w, id, h);
    }
}

/// `GenerateLoot(spawning)`: nothing on spawn; on death the loot of both passes (`base.GenerateLoot(true)` then
/// `base.GenerateLoot(false)`).
pub fn protector_generate_loot_spawning(w: &mut World, id: MobileId, spawning: bool) {
    if spawning {
        return; // No loot/backpack on spawn
    }
    muo_content::creature::setters::base_generate_loot_spawning(w, id, true);
    muo_content::creature::setters::base_generate_loot_spawning(w, id, false);
}
