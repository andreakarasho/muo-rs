//! The Flame Strike spell.

use muo_content::core;
use muo_content::spells::helper::{self, SpellDamageSplit};
use muo_content::spells::magery::{MAGERY_SPELL, SEVENTH_CIRCLE, check_resisted};
use muo_content::spells::spell::{self, SpellId, SpellType};
use muo_content::spells::targeting::{SpellTargetKind, begin_spell_target};
use muo_content::spells::{SpellCircle, SpellInfo, reagent};
use muo_primitives::utility;
use muo_server::World;
use muo_server::effects::EffectLayer;
use muo_server::mobile::part2::{effects, messages as mm};
use muo_server::target::{TargetFlags, Targeted};

use crate::common::{mobile_of, turn_to};

/// `FlameStrikeSpell`: Kal Vas Flam.
pub static FLAME_STRIKE_SPELL: SpellType = SpellType {
    full_name: "Server.Spells.Seventh.FlameStrikeSpell",
    parent: Some(&MAGERY_SPELL),
    target_range: Some(muo_content::spells::targeting::target_range),
    info: SpellInfo::new(
        "Flame Strike",
        "Kal Vas Flam",
        &[reagent::SPIDERS_SILK, reagent::SULFUROUS_ASH],
    )
    .action(245)
    .hand_effect(9042),
    delayed_damage: true,
    on_cast,
    family: Some(&SEVENTH_CIRCLE),
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
        helper::check_reflect(w, SpellCircle::Seventh as i32, &mut caster, &mut m);

        let damage = if core::aos(w) {
            f64::from(spell::get_new_aos_damage(w, sp, 48, 1, 5, true, Some(m)))
        } else {
            let mut d = f64::from(utility::random_from(&mut w.rng, 27, 22));
            if check_resisted(w, sp, m) {
                d *= 0.6;
                mm::send_localized_message(w, m, 501783, "", 0x3B2); // You feel yourself resisting magical energy.
            }
            let ty = spell::spell_type(w, sp).expect("the spell is casting");
            d * (ty.get_damage_scalar)(w, sp, m)
        };

        effects::fixed_particles(w, m, 0x3709, 10, 30, 5052, EffectLayer::LeftFoot);
        effects::play_sound(w, m, 0x208);

        helper::damage_aos_spell(
            w,
            sp,
            m,
            damage,
            SpellDamageSplit {
                fire: 100,
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
