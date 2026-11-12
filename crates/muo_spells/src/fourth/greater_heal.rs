//! The Greater Heal spell.

use muo_content::creature;
use muo_content::items::weapon_ability::is_wounded;
use muo_content::spells::helper;
use muo_content::spells::magery::{FOURTH_CIRCLE, MAGERY_SPELL};
use muo_content::spells::spell::{self, SpellId, SpellType};
use muo_content::spells::targeting::{SpellTargetKind, begin_spell_target};
use muo_content::spells::{SpellInfo, reagent};
use muo_primitives::{MessageType, utility};
use muo_server::World;
use muo_server::effects::EffectLayer;
use muo_server::mobile::part2::{effects, messages as mm};
use muo_server::skills::{self, SkillName};
use muo_server::target::{TargetFlags, Targeted};

use crate::common::{check_cast_sudden_death, mobile_of, turn_to};

/// `GreaterHealSpell`: In Vas Mani.
pub static GREATER_HEAL_SPELL: SpellType = SpellType {
    full_name: "Server.Spells.Fourth.GreaterHealSpell",
    parent: Some(&MAGERY_SPELL),
    target_range: Some(muo_content::spells::targeting::target_range),
    info: SpellInfo::new(
        "Greater Heal",
        "In Vas Mani",
        &[
            reagent::GARLIC,
            reagent::GINSENG,
            reagent::MANDRAKE_ROOT,
            reagent::SPIDERS_SILK,
        ],
    )
    .action(204)
    .hand_effect(9061),
    on_cast,
    check_cast: check_cast_sudden_death,
    family: Some(&FOURTH_CIRCLE),
    ..MAGERY_SPELL
};

/// `Target(Mobile m)`.
pub fn target(w: &mut World, sp: SpellId, targeted: Targeted) {
    let Some(m) = mobile_of(targeted) else {
        return;
    };
    let Some(caster) = spell::caster(w, sp) else {
        return;
    };
    if creature::misc::is_animated_dead(w, m) {
        mm::send_localized_message(w, caster, 1061654, "", 0x3B2); // You cannot heal that which is not alive.
    } else if muo_server::mobile::base::is_dead_bonded_pet(w, m) {
        mm::send_localized_message(w, caster, 1060177, "", 0x3B2); // You cannot heal a creature that is already dead!
    } else if creature::type_name(w, m) == "Golem" {
        mm::local_overhead_message_number(w, caster, MessageType::REGULAR, 0x3B2, 500951, ""); // You cannot heal that.
    } else if w.mobile(m).is_some_and(|x| x.poison.is_some()) || is_wounded(w, m) {
        let n = if caster == m { 1005000 } else { 1010398 };
        mm::local_overhead_message_number(w, caster, MessageType::REGULAR, 0x22, n, "");
    } else if spell::check_b_sequence(w, sp, m, false) {
        turn_to(w, caster, m);

        // Algorithm: (40% of magery) + (1-10)
        let mut to_heal = (skills::value(w, caster, SkillName::Magery) * 0.4) as i32;
        to_heal += utility::random_from(&mut w.rng, 1, 10);

        helper::heal(w, to_heal, m, Some(caster), true);

        effects::fixed_particles(w, m, 0x376A, 9, 32, 5030, EffectLayer::Waist);
        effects::play_sound(w, m, 0x202);
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
