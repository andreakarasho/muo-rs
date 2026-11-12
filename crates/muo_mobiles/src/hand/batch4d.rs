//! MeerCaptain and MeerEternal.

use muo_content::creature::{BASE_CREATURE_C, control};
use muo_content::factory;
use muo_content::misc::aos_helpers;
use muo_content::mx;
use muo_primitives::{DateTime, TimeSpan};
use muo_server::World;
use muo_server::effects::EffectLayer;
use muo_server::ids::MobileId;
use muo_server::mobile::part1::props_b;
use muo_server::mobile::part2::{effects, enumerators, messages};
use muo_server::timer;

use super::batch4c::{NextAbility, set_next};
use crate::common::*;

fn next_ability(w: &World, id: MobileId) -> DateTime {
    w.mobile(id)
        .and_then(|m| m.ext.get::<NextAbility>())
        .map_or(DateTime::MIN, |n| n.0)
}

fn set_next_ability_secs(w: &mut World, id: MobileId, lo: i32, hi: i32) {
    let secs = rnd::min_max(w, lo, hi);
    set_next(w, id, w.now + TimeSpan::from_secs(i64::from(secs)));
}

// ---- MeerCaptain ----

/// `var pack = new Backpack(); ..; AddItem(new Crossbow()); PackItem(pack); m_NextAbilityTime = ..`
pub fn meer_captain_ctor1(w: &mut World, id: MobileId) {
    use muo_server::item::container::{drop_item_default, try_drop_item};
    let pack = factory::make_item(w, "Server.Items.Backpack");
    if let Some(p) = pack {
        for _ in 0..2 {
            let n = rnd::min_max(w, 10, 20);
            if let Some(i) = factory::make_item_amount(w, "Server.Items.Bolt", n) {
                drop_item_default(w, p, i);
            }
        }
        // The crossbow is wielded; the sword would share its hands, so it is carried as loot.
        let sword = match rnd::below(w, 6) {
            0 => "Longsword",
            1 => "Cutlass",
            2 => "Broadsword",
            3 => "Katana",
            4 => "Scimitar",
            _ => "VikingSword",
        };
        if let Some(i) = factory::make_item(w, &format!("Server.Items.{sword}")) {
            drop_item_default(w, p, i);
        }
        let bag = factory::make_item(w, "Server.Items.Bag");
        let count = rnd::min_max(w, 10, 20);
        if let Some(bag) = bag {
            for _ in 0..count {
                let Some(item) = muo_content::loot::random_reagent(w) else {
                    continue;
                };
                if !try_drop_item(w, bag, Some(id), item, false) {
                    muo_server::item::base::delete(w, item);
                }
            }
            drop_item_default(w, p, bag);
        }
    }
    super::ctors::add(w, id, "Crossbow");
    pack_item(w, id, pack);
    set_next_ability_secs(w, id, 2, 5);
}

/// `OnThink`: a magic absorb while fighting, and a heal for meer warriors nearby every 10-15 seconds.
pub fn meer_captain_on_think(w: &mut World, id: MobileId) {
    if mx::combatant(w, id).is_some() && w.mobile(id).is_some_and(|m| m.magic_damage_absorb < 1) {
        let n = rnd::min_max(w, 5, 7);
        if let Some(m) = w.mobile_mut(id) {
            m.magic_damage_absorb = n;
        }
        effects::fixed_particles(w, id, 0x375A, 10, 15, 5037, EffectLayer::Waist);
        mx::play_sound(w, id, 0x1E9);
    }
    if w.now >= next_ability(w, id) {
        set_next_ability_secs(w, id, 10, 15);
        for m in enumerators::get_mobiles_in_range(w, id, 8) {
            if !factory::mobile_is(w, m, "Server.Mobiles.MeerWarrior")
                || !control::is_friend(w, id, m)
                || !muo_server::mobile::base::can_be_beneficial(w, id, m)
                || mx::hits(w, m) >= muo_server::mobile::base::hits_max(w, m)
                || props_b::poisoned(w, m)
                || muo_content::items::weapon_ability::is_wounded(w, m)
            {
                continue;
            }
            muo_server::mobile::base::do_beneficial(w, id, m);
            let to_heal = rnd::min_max(w, 20, 30);
            let d = mx::get_direction_to(w, id, m);
            mx::set_direction(w, id, d);
            muo_server::mobile::part2::damage::heal_from(w, m, to_heal, Some(id), true);
            effects::fixed_particles(w, m, 0x376A, 9, 32, 5030, EffectLayer::Waist);
            mx::play_sound(w, m, 0x202);
        }
    }
    (BASE_CREATURE_C.on_think)(w, id);
}

// ---- MeerEternal ----

/// `m_NextAbilityTime = Core.Now + TimeSpan.FromSeconds(Utility.RandomMinMax(2, 5));`
pub fn meer_eternal_ctor1(w: &mut World, id: MobileId) {
    set_next_ability_secs(w, id, 2, 5);
}

fn say_ascii(w: &mut World, id: MobileId, text: &str) {
    messages::say(w, id, true, text);
}

/// `Hits += AOS.Damage(m, this, amount, 100, 0, 0, 0, 0)`: the leech heals by what it dealt.
fn leech(w: &mut World, id: MobileId, m: MobileId, amount: i32) {
    let dealt = aos_helpers::damage(
        w,
        m,
        Some(id),
        amount,
        aos_helpers::DamageSplit {
            phys: 100,
            fire: 0,
            cold: 0,
            pois: 0,
            nrgy: 0,
        },
        Default::default(),
    );
    let h = mx::hits(w, id);
    props_b::set_hits(w, id, h + dealt);
}

/// The two particle streams of a leech: `from.MovingParticles(to, ..)`.
fn particles(
    w: &mut World,
    from: MobileId,
    to: MobileId,
    first: i32,
    hue: i32,
    effect: i32,
    explode_effect: i32,
) {
    let to = mx::entity(to);
    effects::moving_particles_layer(
        w,
        from,
        to,
        first,
        1,
        0,
        false,
        false,
        hue,
        0,
        effect,
        1,
        0,
        EffectLayer::None,
        0x100,
    );
    effects::moving_particles_layer(
        w,
        from,
        to,
        0x0001,
        1,
        0,
        false,
        true,
        hue,
        0,
        effect,
        explode_effect,
        0,
        EffectLayer::None,
        0,
    );
}

fn area_leech(w: &mut World, id: MobileId) {
    let next = next_ability(w, id);
    set_next(w, id, next + TimeSpan::from_millis(2500));
    say_ascii(w, id, "Beware, mortals!  You have provoked my wrath!");
    effects::fixed_particles_hue(w, id, 0x376A, 10, 10, 9537, 33, 0, EffectLayer::Waist);
    timer::start(w, TimeSpan::from_secs(5), TimeSpan::ZERO, 1, move |w| {
        area_leech_finish(w, id)
    });
}

fn area_leech_finish(w: &mut World, id: MobileId) {
    let mut queue = Vec::new();
    for m in enumerators::get_mobiles_in_range(w, id, 6) {
        if mx::can_be_harmful(w, id, m) && (BASE_CREATURE_C.is_enemy)(w, id, m) {
            queue.push(m);
        }
    }
    if queue.is_empty() {
        say_ascii(w, id, "Bah! You have escaped my grasp this time, mortal!");
        return;
    }
    let scalar = match queue.len() {
        1 => 0.75,
        2 => 0.50,
        _ => 0.25,
    };
    for m in queue {
        let r = rnd::min_max(w, -5, 5);
        let damage = (f64::from(mx::hits(w, m)) * scalar) as i32 + r;
        particles(w, m, id, 0x36F4, 32, 9535, 9536);
        mx::do_harmful(w, id, m);
        leech(w, id, m, damage.max(1));
    }
    say_ascii(w, id, "If I cannot cleanse thy soul, I will destroy it!");
}

fn focused_leech(w: &mut World, id: MobileId, combatant: MobileId, message: &str) {
    say_ascii(w, id, message);
    timer::start(w, TimeSpan::from_millis(500), TimeSpan::ZERO, 1, move |w| {
        if !mx::can_be_harmful(w, id, combatant) {
            return;
        }
        // Stage 1
        particles(w, id, combatant, 0x36FA, 1108, 9533, 9534);
        mx::play_sound(w, id, 0x1FB);
        timer::start(w, TimeSpan::from_secs(1), TimeSpan::ZERO, 1, move |w| {
            if mx::can_be_harmful(w, id, combatant) {
                // Stage 2
                particles(w, combatant, id, 0x36F4, 32, 9535, 9536);
                mx::play_sound(w, id, 0x209);
                mx::do_harmful(w, id, combatant);
                let amount = rnd::min_max(w, 30, 40) - if w.core_aos() { 0 } else { 10 };
                leech(w, id, combatant, amount);
            }
        });
    });
}

/// `OnThink`: every 10-15 seconds one of three focused leeches or the area leech.
pub fn meer_eternal_on_think(w: &mut World, id: MobileId) {
    if w.now >= next_ability(w, id)
        && let Some(c) = mx::combatant(w, id)
        && mx::map(w, c) == mx::map(w, id)
        && mx::in_range(w, c, id, 12)
    {
        set_next_ability_secs(w, id, 10, 15);
        match rnd::below(w, 4) {
            0 => focused_leech(
                w,
                id,
                c,
                "Thine essence will fill my withering body with strength!",
            ),
            1 => focused_leech(
                w,
                id,
                c,
                "I rebuke thee, worm, and cleanse thy vile spirit of its tainted blood!",
            ),
            2 => focused_leech(
                w,
                id,
                c,
                "I devour your life's essence to strengthen my resolve!",
            ),
            _ => area_leech(w, id),
        }
    }
    (BASE_CREATURE_C.on_think)(w, id);
}
