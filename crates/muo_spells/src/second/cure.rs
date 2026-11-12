//! The Cure spell.

use muo_content::core;
use muo_content::spells::magery::{MAGERY_SPELL, SECOND_CIRCLE};
use muo_content::spells::spell::{self, SpellId, SpellType};
use muo_content::spells::targeting::{SpellTargetKind, begin_spell_target};
use muo_content::spells::{SpellInfo, reagent};
use muo_primitives::utility;
use muo_server::World;
use muo_server::effects::EffectLayer;
use muo_server::mobile::base as mb;
use muo_server::mobile::part2::{effects, messages as mm};
use muo_server::skills::{self, SkillName};
use muo_server::target::{TargetFlags, Targeted};

use crate::common::{check_cast_sudden_death, mobile_of, poison_level, turn_to};

/// `CureSpell`: An Nox.
pub static CURE_SPELL: SpellType = SpellType {
    full_name: "Server.Spells.Second.CureSpell",
    parent: Some(&MAGERY_SPELL),
    target_range: Some(muo_content::spells::targeting::target_range),
    info: SpellInfo::new("Cure", "An Nox", &[reagent::GARLIC, reagent::GINSENG])
        .action(212)
        .hand_effect(9061),
    on_cast,
    check_cast: check_cast_sudden_death,
    family: Some(&SECOND_CIRCLE),
    ..MAGERY_SPELL
};

fn target(w: &mut World, sp: SpellId, targeted: Targeted) {
    let Some(m) = mobile_of(targeted) else {
        return;
    };
    let Some(caster) = spell::caster(w, sp) else {
        return;
    };
    if spell::check_b_sequence(w, sp, m, false) {
        turn_to(w, caster, m);

        if let Some(level) = poison_level(w, m) {
            let per = if core::aos(w) {
                if level < 4 { 3300 } else { 3100 }
            } else {
                1750
            };
            let mut chance_to_cure = 10000
                + (skills::value(w, caster, SkillName::Magery) * 75.0) as i32
                - (level + 1) * per;
            chance_to_cure /= 100;

            if chance_to_cure > utility::random(&mut w.rng, 100) {
                if mb::cure_poison(w, m, Some(caster)) {
                    if caster != m {
                        mm::send_localized_message(w, caster, 1010058, "", 0x3B2); // You have cured the target of all poisons!
                    }
                    mm::send_localized_message(w, m, 1010059, "", 0x3B2); // You have been cured of all poisons.
                }
            } else {
                mm::send_localized_message(w, caster, 1010060, "", 0x3B2); // You have failed to cure your target!
            }
        }

        effects::fixed_particles(w, m, 0x373A, 10, 15, 5012, EffectLayer::Waist);
        effects::play_sound(w, m, 0x1E0);
    }
}

fn on_cast(w: &mut World, sp: SpellId) {
    begin_spell_target(
        w,
        sp,
        false,
        TargetFlags::BENEFICIAL,
        false,
        SpellTargetKind::Mobile,
        target,
    );
}
