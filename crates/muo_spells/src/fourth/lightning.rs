//! The Lightning spell.

use muo_content::core;
use muo_content::spells::helper::{self, SpellDamageSplit};
use muo_content::spells::magery::{FOURTH_CIRCLE, MAGERY_SPELL, check_resisted};
use muo_content::spells::spell::{self, SpellId, SpellType};
use muo_content::spells::targeting::{SpellTargetKind, begin_spell_target};
use muo_content::spells::{SpellCircle, SpellInfo, reagent};
use muo_primitives::utility;
use muo_server::World;
use muo_server::mobile::part2::{effects, messages as mm};
use muo_server::target::{TargetFlags, Targeted};

use crate::common::{mobile_of, turn_to};

/// `LightningSpell`: Por Ort Grav.
pub static LIGHTNING_SPELL: SpellType = SpellType {
    full_name: "Server.Spells.Fourth.LightningSpell",
    parent: Some(&MAGERY_SPELL),
    target_range: Some(muo_content::spells::targeting::target_range),
    info: SpellInfo::new(
        "Lightning",
        "Por Ort Grav",
        &[reagent::MANDRAKE_ROOT, reagent::SULFUROUS_ASH],
    )
    .action(239)
    .hand_effect(9021),
    delayed_damage: false,
    on_cast,
    family: Some(&FOURTH_CIRCLE),
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
        helper::check_reflect(w, SpellCircle::Fourth as i32, &mut caster, &mut m);

        let damage = if core::aos(w) {
            f64::from(spell::get_new_aos_damage(w, sp, 23, 1, 4, true, Some(m)))
        } else {
            let mut d = f64::from(utility::random_from(&mut w.rng, 12, 9));
            if check_resisted(w, sp, m) {
                d *= 0.75;
                mm::send_localized_message(w, m, 501783, "", 0x3B2); // You feel yourself resisting magical energy.
            }
            let ty = spell::spell_type(w, sp).expect("the spell is casting");
            d * (ty.get_damage_scalar)(w, sp, m)
        };

        effects::bolt_effect(w, m, 0);

        helper::damage_aos_spell(
            w,
            sp,
            m,
            damage,
            SpellDamageSplit {
                nrgy: 100,
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
