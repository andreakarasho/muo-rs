//! The Strangle spell.

use indexmap::IndexMap;
use muo_content::misc::aos_helpers::{self, DamageOptions, DamageSplit};
use muo_content::player_mobile::buff::{BuffIcon, BuffInfo, add_buff};
use muo_content::spells::spell::{self, SpellId, SpellType};
use muo_content::spells::targeting::{SpellTargetKind, begin_spell_target};
use muo_content::spells::{SpellInfo, reagent};
use muo_io::text::TextDefinition;
use muo_primitives::{DateTime, TimeSpan, utility};
use muo_server::World;
use muo_server::effects::EffectLayer;
use muo_server::ids::MobileId;
use muo_server::mobile::base as mb;
use muo_server::mobile::part2::{effects, messages as mm};
use muo_server::skills::{self, SkillName};
use muo_server::target::{TargetFlags, Targeted};
use muo_server::timer::{self, TimerHandle};

use super::necromancer::{NECRO_BASE, NECROMANCER_SPELL, NecroType};
use crate::common::{alive, is_player_mobile, mobile_of, revealing_action, turn_to};

static STRANGLE: NecroType = NecroType {
    required_skill: 65.0,
    required_mana: 29,
    ..NECRO_BASE
};

/// `StrangleSpell`: In Bal Nox.
pub static STRANGLE_SPELL: SpellType = SpellType {
    full_name: "Server.Spells.Necromancy.StrangleSpell",
    parent: Some(&NECROMANCER_SPELL),
    target_range: Some(muo_content::spells::targeting::target_range),
    info: SpellInfo::new(
        "Strangle",
        "In Bal Nox",
        &[reagent::DAEMON_BLOOD, reagent::NOX_CRYSTAL],
    )
    .action(209)
    .hand_effect(9031),
    cast_delay_base: |_, _| TimeSpan::from_secs(2),
    on_cast,
    family: Some(&STRANGLE),
    ..NECROMANCER_SPELL
};

/// `InternalTimer`'s state.
struct Entry {
    timer: TimerHandle,
    from: MobileId,
    max_base_damage: f64,
    min_base_damage: f64,
    max_count: i32,
    count: i32,
    hit_delay: i32,
    next_hit: DateTime,
}

#[derive(Default)]
struct Table(IndexMap<MobileId, Entry>);

fn table(w: &mut World) -> &mut IndexMap<MobileId, Entry> {
    &mut w.resources.get_mut::<Table>().0
}

fn target(w: &mut World, sp: SpellId, targeted: Targeted) {
    let Some(m) = mobile_of(targeted) else {
        return;
    };
    let Some(caster) = spell::caster(w, sp) else {
        return;
    };
    if spell::check_h_sequence(w, sp, m) {
        turn_to(w, caster, m);

        // SpellHelper.CheckReflect( (int)this.Circle, Caster, ref m ); Irrelevant after AoS
        /* Temporarily chokes off the air supply of the target with poisonous fumes. The target is inflicted with
         * poison damage over time. The amount of damage dealt each "hit" is based off of the caster's Spirit Speak
         * skill and the Target's current Stamina. The less Stamina the target has, the more damage is done by
         * Strangle. Duration of the effect is Spirit Speak skill level / 10 rounds, with a minimum number of 4
         * rounds. The first round of damage is dealt after 5 seconds, and every next round after that comes 1
         * second sooner than the one before, until there is only 1 second between rounds.
         */
        spell::caster_hurt(w, m);

        effects::play_sound(w, m, 0x22F);
        effects::fixed_particles_hue(w, m, 0x36CB, 1, 9, 9911, 67, 5, EffectLayer::Head);
        effects::fixed_particles_hue(w, m, 0x374A, 1, 17, 9502, 1108, 4, EffectLayer(255));

        // According to testing on OSI, it is refreshed.
        if let Some(mut e) = table(w).shift_remove(&m) {
            e.timer.cancel(w);
        }

        start_entry(w, m, caster);

        spell::harmful_spell(w, sp, m);

        /* Note: OSI has a bug where the last "tick" is at 0 seconds and never happens.
         * We opt for the last tick at 1 second, so the duration is 1 second longer than it needs to be.
         */
        // Calculations for the buff bar
        let power = (skills::value(w, caster, SkillName::SpiritSpeak) / 10.0).max(4.0);

        // Closed formula based on loop:
        // MaxCount = 4 -> 5 + (4 + 3 + 2 + 1)
        // MaxCount > 4 -> 5 + (for each count, Ceiling((1.0 + 5 * count) / power) not to exceed 5)
        let total_length = if power % 5.0 == 0.0 {
            3.0 * power + 5.0
        } else {
            3.0 * power + 3.0
        };

        let duration = TimeSpan::from_secs_f64(total_length);

        const MIN_DAMAGE: i32 = 4;
        let max_damage = (power as i32 + 1) * 3;
        let args = format!("{MIN_DAMAGE}\t{max_damage}");

        if is_player_mobile(w, m) {
            add_buff(
                w,
                m,
                BuffInfo::with_secondary(BuffIcon::Strangle, 1075794, 1075795)
                    .duration(duration)
                    .args(TextDefinition::of_text(args)),
            );
        }
    }
}

/// `new InternalTimer(target, from)` started.
fn start_entry(w: &mut World, target: MobileId, from: MobileId) {
    let power = skills::value(w, from, SkillName::SpiritSpeak) / 10.0;
    let hit_delay = 5;
    let next_hit = w.now + TimeSpan::from_secs(i64::from(hit_delay));
    let max_count = 4.max(power as i32);

    let h = timer::start(
        w,
        TimeSpan::from_millis(100),
        TimeSpan::from_millis(100),
        0,
        move |w| on_tick(w, target),
    );
    table(w).insert(
        target,
        Entry {
            timer: h,
            from,
            min_base_damage: power - 2.0,
            max_base_damage: power + 1.0,
            hit_delay,
            next_hit,
            max_count,
            count: max_count,
        },
    );
}

fn stop(w: &mut World, target: MobileId) {
    if let Some(mut e) = table(w).shift_remove(&target) {
        e.timer.cancel(w);
    }
}

fn on_tick(w: &mut World, target: MobileId) {
    if !alive(w, target) {
        stop(w, target);
        return;
    }

    let now = w.now;
    let Some(e) = table(w).get_mut(&target) else {
        return;
    };
    if now < e.next_hit {
        return;
    }

    e.count -= 1;
    if e.count == 0 {
        mm::send_localized_message(w, target, 1061687, "", 0x3B2); // You can breathe normally again.
        stop(w, target);
        return;
    }

    if e.hit_delay > 1 {
        if e.max_count < 5 {
            e.hit_delay -= 1;
        } else {
            e.hit_delay = ((1.0 + 5.0 * f64::from(e.count)) / f64::from(e.max_count))
                .ceil()
                .min(5.0) as i32;
        }
    }

    e.next_hit = now + TimeSpan::from_secs(i64::from(e.hit_delay));
    let (min, max, from) = (e.min_base_damage, e.max_base_damage, e.from);

    let mut damage = utility::random_min_max_f64(&mut w.rng, min, max);

    let (stam, stam_max) = (
        w.mobile(target).map_or(0, |x| x.stam),
        mb::stam_max(w, target),
    );
    damage *= 3.0 - f64::from(stam) / f64::from(stam_max) * 2.0;

    if damage < 1.0 {
        damage = 1.0;
    }

    if !w.mobile(target).is_some_and(|x| x.player) {
        damage *= 1.75;
    }

    aos_helpers::damage(
        w,
        target,
        Some(from),
        damage as i32,
        DamageSplit {
            pois: 100,
            ..Default::default()
        },
        DamageOptions::default(),
    );

    // OSI: randomly revealed between first and third damage tick, guessing 60% chance
    if utility::random_double(&mut w.rng) < 0.40 {
        revealing_action(w, target);
    }
}

fn on_cast(w: &mut World, sp: SpellId) {
    begin_spell_target(
        w,
        sp,
        false,
        TargetFlags::HARMFUL,
        false,
        SpellTargetKind::Mobile,
        target,
    );
}

/// `RemoveCurse(m)`.
pub fn remove_curse(w: &mut World, m: MobileId) -> bool {
    if table(w).contains_key(&m) {
        stop(w, m);
        mm::send_localized_message(w, m, 1061687, "", 0x3B2); // You can breath normally again.
        return true;
    }
    false
}
