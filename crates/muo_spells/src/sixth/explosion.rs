//! The Explosion spell.

use muo_content::core;
use muo_content::spells::helper::{self, SpellDamageSplit};
use muo_content::spells::magery::{MAGERY_SPELL, SIXTH_CIRCLE, check_resisted};
use muo_content::spells::spell::{self, SpellId, SpellType};
use muo_content::spells::targeting::{SpellTargetKind, begin_spell_target};
use muo_content::spells::{SpellCircle, SpellInfo, reagent};
use muo_primitives::{TimeSpan, utility};
use muo_server::World;
use muo_server::effects::EffectLayer;
use muo_server::mobile::base as mb;
use muo_server::mobile::part2::{effects, messages as mm};
use muo_server::target::{TargetFlags, Targeted};
use muo_server::timer;

use crate::common::{alive, mobile_of, turn_to};

/// `ExplosionSpell`: Vas Ort Flam.
pub static EXPLOSION_SPELL: SpellType = SpellType {
    full_name: "Server.Spells.Sixth.ExplosionSpell",
    parent: Some(&MAGERY_SPELL),
    target_range: Some(muo_content::spells::targeting::target_range),
    info: SpellInfo::new(
        "Explosion",
        "Vas Ort Flam",
        &[reagent::BLOODMOSS, reagent::MANDRAKE_ROOT],
    )
    .action(230)
    .hand_effect(9041),
    // AOSNoDelayedDamageStackingSelf
    delayed_damage_family: Some(&[]),
    delayed_damage: false,
    on_cast,
    family: Some(&SIXTH_CIRCLE),
    ..MAGERY_SPELL
};

fn target(w: &mut World, sp: SpellId, targeted: Targeted) {
    let Some(m) = mobile_of(targeted) else {
        return;
    };
    let Some(caster) = spell::caster(w, sp) else {
        return;
    };
    if core::sa(w) && spell::has_delayed_damage_context(w, sp, m) {
        let ty = spell::spell_type(w, sp).expect("the spell is casting");
        (ty.do_hurt_fizzle)(w, sp);
        return;
    }

    if mb::can_be_harmful(w, caster, m) && spell::check_sequence(w, sp) {
        let defender = m;
        turn_to(w, caster, m);

        let mut c = caster;
        let mut target = m;
        helper::check_reflect(w, SpellCircle::Sixth as i32, &mut c, &mut target);

        // InternalTimer: the context is started by its constructor
        let delay = TimeSpan::from_secs_f64(if core::aos(w) { 3.0 } else { 2.5 });
        let h = timer::start(w, delay, TimeSpan::ZERO, 1, move |w| {
            on_tick(w, sp, caster, defender, target)
        });
        spell::start_delayed_damage_context(w, sp, caster, h);
    }
}

fn on_tick(
    w: &mut World,
    sp: SpellId,
    attacker: muo_server::ids::MobileId,
    defender: muo_server::ids::MobileId,
    target: muo_server::ids::MobileId,
) {
    if w.mobile(defender).is_none_or(|d| d.deleted) || !alive(w, defender) {
        return;
    }

    if mb::harmful_check(w, attacker, defender) {
        let damage = if core::aos(w) {
            f64::from(spell::get_new_aos_damage(
                w,
                sp,
                40,
                1,
                5,
                true,
                Some(defender),
            ))
        } else {
            let mut d = f64::from(utility::random_from(&mut w.rng, 23, 22));
            if check_resisted(w, sp, target) {
                d *= 0.75;
                mm::send_localized_message(w, target, 501783, "", 0x3B2); // You feel yourself resisting magical energy.
            }
            let ty = spell::spell_type(w, sp).expect("the spell exists");
            d * (ty.get_damage_scalar)(w, sp, target)
        };

        effects::fixed_particles(w, target, 0x36BD, 20, 10, 5044, EffectLayer::Head);
        effects::play_sound(w, target, 0x307);

        helper::damage_aos_spell(
            w,
            sp,
            target,
            damage,
            SpellDamageSplit {
                fire: 100,
                ..Default::default()
            },
        );
    }

    spell::remove_delayed_damage_context(w, sp, attacker);
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
