//! The Harm spell.

use muo_content::core;
use muo_content::spells::helper::{self, SpellDamageSplit};
use muo_content::spells::magery::{MAGERY_SPELL, SECOND_CIRCLE, check_resisted};
use muo_content::spells::spell::{self, SpellId, SpellType};
use muo_content::spells::targeting::{SpellTargetKind, begin_spell_target};
use muo_content::spells::{SpellCircle, SpellInfo, reagent};
use muo_data::expansion::Expansion;
use muo_primitives::utility;
use muo_server::World;
use muo_server::effects::EffectLayer;
use muo_server::mobile::base as mb;
use muo_server::mobile::part2::{effects, messages as mm};
use muo_server::target::{TargetFlags, Targeted};

use crate::common::{mobile_of, turn_to};

/// `HarmSpell`: An Mani.
pub static HARM_SPELL: SpellType = SpellType {
    full_name: "Server.Spells.Second.HarmSpell",
    parent: Some(&MAGERY_SPELL),
    target_range: Some(muo_content::spells::targeting::target_range),
    info: SpellInfo::new(
        "Harm",
        "An Mani",
        &[reagent::NIGHTSHADE, reagent::SPIDERS_SILK],
    )
    .action(212)
    .hand_effect(9001)
    .pre_aos(212, 9041),
    delayed_damage: false,
    on_cast,
    get_slayer_damage_scalar: |_, _, _| 1.0,
    family: Some(&SECOND_CIRCLE),
    ..MAGERY_SPELL
};

fn target(w: &mut World, sp: SpellId, targeted: Targeted) {
    let Some(mut m) = mobile_of(targeted) else {
        return;
    };
    if spell::check_h_sequence(w, sp, m) {
        let Some(mut caster) = spell::caster(w, sp) else {
            return;
        };
        turn_to(w, caster, m);
        // `CheckReflect((int)Circle, Caster, ref m)`
        helper::check_reflect(w, SpellCircle::Second as i32, &mut caster, &mut m);
        let caster = spell::caster(w, sp).unwrap_or(caster);

        let mut damage: f64;
        if core::aos(w) {
            damage = f64::from(spell::get_new_aos_damage(w, sp, 17, 1, 5, true, Some(m)));
        } else {
            damage = f64::from(utility::random_from(&mut w.rng, 1, 15));
            if check_resisted(w, sp, m) {
                damage *= 0.75;
                mm::send_localized_message(w, m, 501783, "", 0x3B2); // You feel yourself resisting magical energy.
            }
            let ty = spell::spell_type(w, sp).expect("the spell is casting");
            damage *= (ty.get_damage_scalar)(w, sp, m);
        }

        if core::expansion(w) >= Expansion::UOTD {
            let loc = w.mobile(caster).map(|c| c.location);
            let near = |w: &World, r: i32| loc.is_some_and(|l| mb::in_range_point3d(w, m, l, r));
            if !near(w, 2) {
                damage *= 0.25; // 1/4 damage at > 2 tile range
            } else if !near(w, 1) {
                damage *= 0.50; // 1/2 damage at 2 tile range
            }
        }

        if core::aos(w) {
            effects::fixed_particles_hue(w, m, 0x374A, 10, 30, 5013, 1153, 2, EffectLayer::Waist);
            effects::play_sound(w, m, 0x0FC);
        } else {
            effects::fixed_particles(w, m, 0x374A, 10, 15, 5013, EffectLayer::Waist);
            effects::play_sound(w, m, 0x1F1);
        }

        helper::damage_aos_spell(
            w,
            sp,
            m,
            damage,
            SpellDamageSplit {
                cold: 100,
                ..Default::default()
            },
        );
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
