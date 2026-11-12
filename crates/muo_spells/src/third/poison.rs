//! The Poison spell.

use muo_content::core;
use muo_content::spells::helper;
use muo_content::spells::magery::{MAGERY_SPELL, THIRD_CIRCLE, check_resisted};
use muo_content::spells::spell::{self, SpellId, SpellType};
use muo_content::spells::targeting::{SpellTargetKind, begin_spell_target};
use muo_content::spells::{SpellCircle, SpellInfo, reagent};
use muo_primitives::utility;
use muo_server::World;
use muo_server::effects::EffectLayer;
use muo_server::mobile::base as mb;
use muo_server::mobile::part2::{effects, messages as mm};
use muo_server::skills::{self, SkillName};
use muo_server::target::{TargetFlags, Targeted};

use crate::common::{distance_sqrt, mobile_of, poison_by_level, turn_to};

/// `PoisonSpell`: In Nox.
pub static POISON_SPELL: SpellType = SpellType {
    full_name: "Server.Spells.Third.PoisonSpell",
    parent: Some(&MAGERY_SPELL),
    target_range: Some(muo_content::spells::targeting::target_range),
    info: SpellInfo::new("Poison", "In Nox", &[reagent::NIGHTSHADE])
        .action(203)
        .hand_effect(9051),
    on_cast,
    family: Some(&THIRD_CIRCLE),
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
        helper::check_reflect(w, SpellCircle::Third as i32, &mut caster, &mut m);
        let caster = spell::caster(w, sp).unwrap_or(caster);

        spell::caster_hurt(w, m);
        mb::set_paralyzed(w, m, false);

        if check_resisted(w, sp, m) {
            mm::send_localized_message(w, m, 501783, "", 0x3B2); // You feel yourself resisting magical energy.
        } else {
            let mut total = skills::value(w, caster, SkillName::Magery);

            let is_pm = crate::common::is_player_mobile(w, caster);
            if is_pm {
                // `pm.DuelContext?.Started != true || Finished || Ruleset.GetOption("Skills", "Poisoning")`
                let allowed = muo_content::spells::hooks::hooks(w)
                    .and_then(|h| h.duel_allows_skill)
                    .is_none_or(|f| f(w, caster, "Poisoning"));
                if allowed {
                    total += skills::value(w, caster, SkillName::Poisoning);
                }
            } else {
                total += skills::value(w, caster, SkillName::Poisoning);
            }

            let dist = distance_sqrt(w, caster, m);
            let mut level: i32;

            if core::aos(w) && dist >= 3.0 {
                level = 0;
            } else {
                if !core::aos(w) && dist >= 3.0 {
                    total -= (dist - 3.0) * 10.0;
                }

                if core::sa(w) && dist >= 2.0 {
                    total -= (dist - 2.0) * 31.0; // 240 -
                }

                level = if total > 200.0 && core::sa(w) && dist <= 2.0 {
                    if utility::random(&mut w.rng, 10) == 0 {
                        4
                    } else {
                        3
                    }
                } else if total > 199.8 {
                    if core::aos(w) || utility::random(&mut w.rng, 10) == 0 {
                        3
                    } else {
                        2
                    }
                } else if total > 170.2 {
                    2
                } else if total > 130.2 {
                    1
                } else {
                    0
                };

                if core::sa(w) && dist > 2.0 {
                    level -= (dist as i32) / 3;
                }
            }

            if let Some(p) = poison_by_level(w, level.max(0)) {
                mb::apply_poison(w, m, Some(caster), p);
            }
        }

        effects::fixed_particles(w, m, 0x374A, 10, 15, 5021, EffectLayer::Waist);
        effects::play_sound(w, m, 0x205);

        spell::harmful_spell(w, sp, m);
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
