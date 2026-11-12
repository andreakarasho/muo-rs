//! The Paralyze spell.

use muo_content::core;
use muo_content::spells::helper;
use muo_content::spells::hooks::hooks;
use muo_content::spells::magery::{FIFTH_CIRCLE, MAGERY_SPELL, check_resisted};
use muo_content::spells::spell::{self, SpellId, SpellType};
use muo_content::spells::targeting::{SpellTargetKind, begin_spell_target};
use muo_content::spells::{SpellCircle, SpellInfo, reagent};
use muo_primitives::TimeSpan;
use muo_server::World;
use muo_server::mobile::base as mb;
use muo_server::mobile::part2::{effects, messages as mm};
use muo_server::skills::{self, SkillName};
use muo_server::target::{TargetFlags, Targeted};

use crate::common::{mobile_of, turn_to};

/// `ParalyzeSpell`: An Ex Por.
pub static PARALYZE_SPELL: SpellType = SpellType {
    full_name: "Server.Spells.Fifth.ParalyzeSpell",
    parent: Some(&MAGERY_SPELL),
    target_range: Some(muo_content::spells::targeting::target_range),
    info: SpellInfo::new(
        "Paralyze",
        "An Ex Por",
        &[
            reagent::GARLIC,
            reagent::MANDRAKE_ROOT,
            reagent::SPIDERS_SILK,
        ],
    )
    .action(218)
    .hand_effect(9012),
    on_cast,
    family: Some(&FIFTH_CIRCLE),
    ..MAGERY_SPELL
};

fn target(w: &mut World, sp: SpellId, targeted: Targeted) {
    let Some(mut m) = mobile_of(targeted) else {
        return;
    };
    let Some(mut caster) = spell::caster(w, sp) else {
        return;
    };

    // `m.Spell?.IsCasting == true && m.Spell is not PaladinSpell`
    let casting_non_paladin = w.mobile(m).and_then(|x| x.spell).is_some_and(|s| {
        spell::is_casting(w, s)
            && !spell::spell_type(w, s)
                .is_some_and(|t| t.full_name.starts_with("Server.Spells.Chivalry."))
    });
    let blocked = w.mobile(m).is_some_and(|x| x.frozen || x.paralyzed) || casting_non_paladin;

    if core::aos(w) && blocked {
        mm::send_localized_message(w, caster, 1061923, "", 0x3B2); // The target is already frozen.
    } else if spell::check_h_sequence(w, sp, m) {
        turn_to(w, caster, m);
        helper::check_reflect(w, SpellCircle::Fifth as i32, &mut caster, &mut m);
        let caster = spell::caster(w, sp).unwrap_or(caster);

        let mut duration: f64;

        let ty = spell::spell_type(w, sp).expect("the spell is casting");
        if core::aos(w) {
            let mut secs = (ty.get_damage_skill)(w, sp, caster) / 10.0
                - (ty.get_resist_skill)(w, sp, m) / 10.0;

            if !w.mobile(m).is_some_and(|x| x.player) {
                secs *= 3.0;
            }

            duration = secs.max(0.0);
        } else {
            // Algorithm: ((20% of magery) + 7) seconds [- 50% if resisted]
            duration = 7.0 + skills::value(w, caster, SkillName::Magery) * 0.2;

            if check_resisted(w, sp, m) {
                duration *= 0.75;
            }
        }

        if let Some(f) = hooks(w).and_then(|h| h.plague_beast_lord_on_paralyzed)
            && f(w, m, caster)
        {
            duration = 120.0;
        }

        mb::paralyze(w, m, TimeSpan::from_secs_f64(duration));
        effects::play_sound(w, m, 0x204);
        effects::fixed_effect(w, m, 0x376A, 6, 1);

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
