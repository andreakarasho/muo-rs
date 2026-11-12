//! The Spell Plague spell.

use indexmap::IndexMap;
use muo_content::core;
use muo_content::player_mobile::buff::{BuffIcon, BuffInfo, add_buff, remove_buff};
use muo_content::spells::helper::{self, SpellDamageSplit};
use muo_content::spells::hooks::hooks_mut;
use muo_content::spells::spell::{self, SpellId, SpellType};
use muo_content::spells::targeting::{SpellTargetKind, begin_spell_target};
use muo_content::spells::{SpellCircle, SpellInfo, reagent};
use muo_primitives::{DateTime, TimeSpan, utility};
use muo_server::World;
use muo_server::effects::EffectLayer;
use muo_server::ids::MobileId;
use muo_server::mobile::part2::effects;
use muo_server::skills::{self, SkillName};
use muo_server::target::{TargetFlags, Targeted};
use muo_server::timer::{self, TimerHandle};

use super::mystic::{MYSTIC_SPELL, MysticType, mystic};
use crate::common::{is_player_mobile, mobile_of, turn_to};

static SPELL_PLAGUE: MysticType = mystic(SpellCircle::Seventh);

/// `SpellPlagueSpell`: Vas Rel Jux Ort.
pub static SPELL_PLAGUE_SPELL: SpellType = SpellType {
    full_name: "Server.Spells.Mysticism.SpellPlagueSpell",
    parent: Some(&MYSTIC_SPELL),
    target_range: Some(muo_content::spells::targeting::target_range),
    info: SpellInfo::new(
        "Spell Plague",
        "Vas Rel Jux Ort",
        &[
            reagent::DAEMON_BONE,
            reagent::DRAGONS_BLOOD,
            reagent::NIGHTSHADE,
            reagent::SULFUROUS_ASH,
        ],
    )
    .action(-1)
    .hand_effect(9002),
    on_cast,
    family: Some(&SPELL_PLAGUE),
    ..MYSTIC_SPELL
};

/// `SpellPlagueTimer`: the plagues of a target are chained one after the other.
struct Plague {
    spell: SpellId,
    timer: TimerHandle,
    explosions: i32,
    next_explosion: DateTime,
    /// `_next`: the plagues queued behind this one.
    queue: Vec<SpellId>,
}

#[derive(Default)]
struct Table(IndexMap<MobileId, Plague>);

fn table(w: &mut World) -> &mut IndexMap<MobileId, Plague> {
    &mut w.resources.get_mut::<Table>().0
}

/// Sets `SpellHooks::spell_plague_on_damaged`.
pub fn init(w: &mut World) {
    hooks_mut(w).spell_plague_on_damaged = Some(on_mobile_damaged);
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

fn target(w: &mut World, sp: SpellId, targeted: Targeted) {
    let Some(mut m) = mobile_of(targeted) else {
        return;
    };
    let Some(mut caster) = spell::caster(w, sp) else {
        return;
    };
    if spell::check_h_sequence(w, sp, m) {
        turn_to(w, caster, m);

        helper::check_reflect(w, 6, &mut caster, &mut m);

        /* The target is hit with an explosion of chaos damage and then inflicted with the spell plague curse. Each
         * time the target is damaged while under the effect of the spell plague, they may suffer an explosion of
         * chaos damage. The initial chance to trigger the explosion starts at 90% and reduces by 30% every time an
         * explosion occurs. Once the target is afflicted by 3 explosions or 8 seconds have passed, that spell
         * plague is removed from the target. Spell Plague will stack with other spell plagues so that they are
         * applied one after the other.
         */
        visual_effect(w, m);

        // Before Time of Legends, SDI was not applied to initial damage.
        let damage = f64::from(spell::get_new_aos_damage(
            w,
            sp,
            33,
            1,
            5,
            core::tol(w),
            Some(m),
        ));
        helper::damage_aos_spell(w, sp, m, damage, SpellDamageSplit::default());

        if table(w).contains_key(&m) {
            if let Some(p) = table(w).get_mut(&m) {
                p.queue.push(sp);
            }
        } else {
            start_plague(w, m, sp);
            let _ = &mut caster;
        }
    }
}

/// `RemoveEffect(m)`.
pub fn remove_effect(w: &mut World, m: MobileId) -> bool {
    if let Some(mut p) = table(w).shift_remove(&m) {
        p.timer.cancel(w);
        if is_player_mobile(w, m) {
            remove_buff(w, m, BuffIcon::SpellPlague);
        }
        return true;
    }
    false
}

/// `OnMobileDamaged(m)`.
pub fn on_mobile_damaged(w: &mut World, m: MobileId) {
    on_damage(w, m);
}

fn visual_effect(w: &mut World, to: MobileId) {
    effects::play_sound(w, to, 0x658);
    effects::fixed_particles_full(w, to, 0x3728, 1, 13, 0x26B8, 0x47E, 7, EffectLayer::Head, 0);
    effects::fixed_particles_full(w, to, 0x3779, 1, 15, 0x251E, 0x43, 7, EffectLayer::Head, 0);
}

/// `StartPlague()`.
fn start_plague(w: &mut World, target: MobileId, sp: SpellId) {
    if is_player_mobile(w, target) {
        add_buff(
            w,
            target,
            BuffInfo::with_secondary(BuffIcon::SpellPlague, 1031690, 1080167)
                .duration(TimeSpan::from_millis(8500)),
        );
    }

    let next_explosion = w.now + TimeSpan::from_secs(1);
    let h = timer::start(w, TimeSpan::from_secs(8), TimeSpan::ZERO, 1, move |w| {
        do_next_plague(w, target)
    });
    let queue = table(w)
        .shift_remove(&target)
        .map(|p| p.queue)
        .unwrap_or_default();
    table(w).insert(
        target,
        Plague {
            spell: sp,
            timer: h,
            explosions: 0,
            next_explosion,
            queue,
        },
    );
}

/// `OnDamage()`.
fn on_damage(w: &mut World, target: MobileId) {
    let now = w.now;
    let Some(p) = table(w).get(&target) else {
        return;
    };
    if now < p.next_explosion {
        return;
    }

    let (sp, explosions) = (p.spell, p.explosions);
    let mut explo_chance = 90 - explosions * 30;

    let resist = skills::fixed(w, target, SkillName::MagicResist) - 700;
    if resist > 0 {
        explo_chance -= resist / 100 * 3;
    }

    if explo_chance > utility::random(&mut w.rng, 100) {
        visual_effect(w, target);

        let damage = f64::from(spell::get_new_aos_damage(
            w,
            sp,
            15 + explosions * 3,
            1,
            5,
            false,
            Some(target),
        ));

        if let Some(p) = table(w).get_mut(&target) {
            p.explosions += 1;
            p.next_explosion = now + TimeSpan::from_secs(1);
        }

        helper::damage_aos_spell(
            w,
            sp,
            target,
            damage,
            SpellDamageSplit {
                chaos: 100,
                ..Default::default()
            },
        );

        let done = table(w).get(&target).is_some_and(|p| p.explosions >= 3);
        if done {
            do_next_plague(w, target);
        }
    }
}

/// `DoNextPlague()`.
fn do_next_plague(w: &mut World, target: MobileId) {
    let Some(mut p) = table(w).shift_remove(&target) else {
        return;
    };
    p.timer.cancel(w);
    let now = w.now;

    if !p.queue.is_empty() {
        let next = p.queue.remove(0);
        // The plague queued behind keeps the plagues behind it.
        table(w).insert(
            target,
            Plague {
                spell: next,
                timer: TimerHandle::default(),
                explosions: 0,
                next_explosion: now,
                queue: p.queue,
            },
        );
        start_plague(w, target, next);
    } else if is_player_mobile(w, target) {
        remove_buff(w, target, BuffIcon::SpellPlague);
    }
}
