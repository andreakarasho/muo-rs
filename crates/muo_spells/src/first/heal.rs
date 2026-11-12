//! The Heal spell.

use muo_content::core;
use muo_content::creature;
use muo_content::items::weapon_ability::is_wounded;
use muo_content::spells::helper;
use muo_content::spells::magery::{FIRST_CIRCLE, MAGERY_SPELL};
use muo_content::spells::spell::{self, SpellId, SpellType};
use muo_content::spells::targeting::{SpellTargetKind, begin_spell_target};
use muo_content::spells::{SpellInfo, reagent};
use muo_primitives::{MessageType, utility};
use muo_server::World;
use muo_server::effects::EffectLayer;
use muo_server::ids::EntityId;
use muo_server::mobile::part2::{effects, messages as mm};
use muo_server::skills::{self, SkillName};
use muo_server::target::{TargetFlags, Targeted};

/// `HealSpell`: In Mani.
pub static HEAL_SPELL: SpellType = SpellType {
    full_name: "Server.Spells.First.HealSpell",
    parent: Some(&MAGERY_SPELL),
    target_range: Some(muo_content::spells::targeting::target_range),
    info: SpellInfo::new(
        "Heal",
        "In Mani",
        &[reagent::GARLIC, reagent::GINSENG, reagent::SPIDERS_SILK],
    )
    .action(224)
    .hand_effect(9061),
    check_cast: crate::common::check_cast_sudden_death,
    on_cast,
    family: Some(&FIRST_CIRCLE),
    ..MAGERY_SPELL
};

/// `Target(Mobile m)`.
pub fn target(w: &mut World, sp: SpellId, targeted: Targeted) {
    let Targeted::Entity(EntityId::Mobile(m)) = targeted else {
        return;
    };
    let Some(caster) = spell::caster(w, sp) else {
        return;
    };
    if muo_server::mobile::base::is_dead_bonded_pet(w, m) {
        mm::send_localized_message(w, caster, 1060177, "", 0x3B2); // You cannot heal a creature that is already dead!
    } else if creature::misc::is_animated_dead(w, m) {
        mm::send_localized_message(w, caster, 1061654, "", 0x3B2); // You cannot heal that which is not alive.
    } else if creature::type_name(w, m) == "Golem" {
        mm::local_overhead_message_number(w, caster, MessageType::REGULAR, 0x3B2, 500951, ""); // You cannot heal that.
    } else if w.mobile(m).is_some_and(|x| x.poison.is_some()) || is_wounded(w, m) {
        let n = if caster == m { 1005000 } else { 1010398 };
        mm::local_overhead_message_number(w, caster, MessageType::REGULAR, 0x22, n, "");
    } else if spell::check_b_sequence(w, sp, m, false) {
        helper::turn(
            w,
            caster,
            Some(helper::Point3dRef::Entity(EntityId::Mobile(m))),
        );

        let magery = skills::value(w, caster, SkillName::Magery);
        let mut to_heal: f64;
        if core::aos(w) {
            to_heal = magery / 12.0;
            to_heal += f64::from(utility::random_min_max(&mut w.rng, 1, 4));
            if core::se(w) && caster != m {
                to_heal *= 1.5;
            }
        } else {
            to_heal = f64::from((magery * 0.1) as i32);
            to_heal += f64::from(utility::random_from(&mut w.rng, 1, 5));
        }

        helper::heal(w, to_heal as i32, m, Some(caster), true);
        effects::fixed_particles(w, m, 0x376A, 9, 32, 5005, EffectLayer::Waist);
        effects::play_sound(w, m, 0x1F2);
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
