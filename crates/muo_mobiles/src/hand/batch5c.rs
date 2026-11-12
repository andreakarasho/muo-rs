//! MeerMage (monsters5): the swarm of insects, `StopEffect` as the death events, the torch and `AOS.Damage` call it.

use muo_content::creature::{BASE_CREATURE_C, cd, control};
use muo_content::mx;
use muo_primitives::{DateTime, MessageType, Point3D, TimeSpan};
use muo_server::World;
use muo_server::effects::EffectLayer;
use muo_server::ids::MobileId;
use muo_server::layer::Layer;
use muo_server::mobile::part2::{effects, items, messages};
use muo_server::timer::{self};

use super::batch4c::{NextAbility, set_next};
use crate::classes::enraged;
use crate::common::*;

/// `m_NextAbilityTime = Core.Now + TimeSpan.FromSeconds(Utility.RandomMinMax(2, 5));`
pub fn meer_mage_ctor1(w: &mut World, id: MobileId) {
    let secs = rnd::min_max(w, 2, 5);
    set_next(w, id, w.now + TimeSpan::from_secs(i64::from(secs)));
}

/// `IsEnemy(m) => (m as BaseEnraged)?.Master != this && base.IsEnemy(m)`.
pub fn meer_mage_is_enemy(w: &mut World, id: MobileId, m: MobileId) -> bool {
    let enraged_of_ours = muo_content::factory::mobile_is(w, m, "Server.Mobiles.BaseEnraged")
        && cd(w, m).is_some_and(|d| d.master == Some(id));
    !enraged_of_ours && (BASE_CREATURE_C.is_enemy)(w, id, m)
}

/// `UnderEffect(m)`.
pub fn meer_mage_under_effect(w: &mut World, m: MobileId) -> bool {
    tables(w).meer_mage.contains_key(&m)
}

/// `StopEffect(m, message)`: the `PlayerDeathEvent`, `CreatureDeathEvent`, `AOS.Damage` and `Torch` hook.
pub fn meer_mage_stop_effect(w: &mut World, m: MobileId, message: bool) {
    let Some(mut h) = tables(w).meer_mage.remove(&m) else {
        return;
    };
    if message {
        // * The open flame begins to scatter the swarm of insects! *
        let hue = w.mobile(m).map_or(0, |x| x.speech_hue);
        messages::public_overhead_message_number(w, m, MessageType::EMOTE, hue, 1071925, "", false);
    }
    h.cancel(w);
}

fn do_effect(w: &mut World, id: MobileId, m: MobileId, count: i32) {
    if !mx::alive(w, m) {
        meer_mage_stop_effect(w, m, false);
        return;
    }
    let torch = items::find_item_on_layer(w, m, Layer::TwoHanded)
        .filter(|&i| muo_content::factory::item_is(w, i, "Server.Items.Torch"));
    if torch.is_some_and(|t| muo_items::lights::base::burning(w, t)) {
        meer_mage_stop_effect(w, m, true);
        return;
    }
    if count % 4 == 0 {
        let hue = w.mobile(m).map_or(0, |x| x.speech_hue);
        // * The swarm of insects bites and stings your flesh! *
        messages::local_overhead_message_number(w, m, MessageType::EMOTE, hue, 1071905, "");
        // * ~1_VAL~ is stung by a swarm of insects *
        messages::nonlocal_overhead_message_number(w, m, MessageType::EMOTE, hue, 1071924, "");
    }
    effects::fixed_particles(w, m, 0x91C, 10, 180, 9539, EffectLayer::Waist);
    mx::play_sound(w, m, 0x00E);
    mx::play_sound(w, m, 0x1BC);
    let amount = rnd::min_max(w, 30, 40) - if w.core_aos() { 0 } else { 10 };
    muo_content::misc::aos_helpers::damage(
        w,
        m,
        Some(id),
        amount,
        muo_content::misc::aos_helpers::DamageSplit {
            phys: 100,
            fire: 0,
            cold: 0,
            pois: 0,
            nrgy: 0,
        },
        Default::default(),
    );
    if !mx::alive(w, m) {
        meer_mage_stop_effect(w, m, false);
    }
}

fn next_ability(w: &World, id: MobileId) -> DateTime {
    w.mobile(id)
        .and_then(|m| m.ext.get::<NextAbility>())
        .map_or(DateTime::MIN, |n| n.0)
}

fn meer_in_range(w: &World, id: MobileId, other: MobileId) -> bool {
    mx::map(w, other) == mx::map(w, id) && mx::in_range(w, other, id, 12)
}

pub fn meer_mage_on_think(w: &mut World, id: MobileId) {
    if w.now >= next_ability(w, id)
        && let Some(mut combatant) = mx::combatant(w, id)
        && meer_in_range(w, id, combatant)
        && muo_content::creature::ctype(w, id).is_some_and(|c| (c.is_enemy)(w, id, combatant))
        && !meer_mage_under_effect(w, combatant)
    {
        let secs = rnd::min_max(w, 20, 30);
        set_next(w, id, w.now + TimeSpan::from_secs(i64::from(secs)));

        if cd(w, combatant).is_some()
            && let Some(cm) = control::control_master(w, combatant)
            && !mx::deleted(w, cm)
            && mx::alive(w, cm)
            && meer_in_range(w, id, cm)
            && !meer_mage_under_effect(w, cm)
        {
            mx::set_combatant(w, id, Some(cm));
            combatant = cm;
        }

        if rnd::double(w) < 0.1 {
            const COORD: [(i32, i32); 5] = [(-4, -6), (4, -6), (0, -8), (-5, 5), (5, 5)];
            let here = mx::location(w, combatant);
            let map = mx::map(w, combatant);
            for (i, (dx, dy)) in COORD.iter().enumerate() {
                let (x, y) = (here.x + dx, here.y + dy);
                let Some(map) = map else { break };
                let z = w.maps.get_average_z(map, x, y);
                if !muo_server::map::fit::can_spawn_mobile_at(w, map, x, y, z) {
                    continue;
                }
                let rabid = match i {
                    0 => enraged::new_enraged_rabbit(w, id),
                    1 => enraged::new_enraged_hind(w, id),
                    2 => enraged::new_enraged_hart(w, id),
                    3 => enraged::new_enraged_black_bear(w, id),
                    _ => enraged::new_enraged_eagle(w, id),
                };
                if let Some(d) = muo_content::creature::cd_mut(w, rabid) {
                    d.focus_mob = Some(combatant);
                }
                muo_server::mobile::base::move_to_world(w, rabid, Point3D::new(x, y, z), map);
            }
            // Creatures of the forest, I call to thee!  Aid me in the fight against all that is evil!
            messages::say_number(w, id, 1071932, "");
        } else if mx::player(w, combatant) {
            // I call a plague of insects to sting your flesh!
            messages::say_number(w, id, 1071931, "");
            let mut count = 0;
            let h = timer::start(
                w,
                TimeSpan::from_millis(500),
                TimeSpan::from_secs(7),
                0,
                move |w| {
                    let c = count;
                    count += 1;
                    do_effect(w, id, combatant, c);
                },
            );
            tables(w).meer_mage.insert(combatant, h);
        }
    }
    (BASE_CREATURE_C.on_think)(w, id);
}
