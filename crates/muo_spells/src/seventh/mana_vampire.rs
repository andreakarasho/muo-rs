//! The Mana Vampire spell.

use muo_content::core;
use muo_content::spells::helper;
use muo_content::spells::magery::{MAGERY_BASE, MAGERY_SPELL, MageryType, check_resisted};
use muo_content::spells::spell::{self, SpellId, SpellType};
use muo_content::spells::targeting::{SpellTargetKind, begin_spell_target};
use muo_content::spells::{SpellCircle, SpellInfo, reagent};
use muo_server::World;
use muo_server::effects::EffectLayer;
use muo_server::mobile::base as mb;
use muo_server::mobile::part1::props_b;
use muo_server::mobile::part2::{effects, messages as mm};
use muo_server::target::{TargetFlags, Targeted};

use crate::common::{mobile_of, turn_to};

/// `GetResistPercent(target) => 98.0`.
static MANA_VAMPIRE_FAMILY: MageryType = MageryType {
    circle: SpellCircle::Seventh,
    get_resist_percent: |_, _, _| 98.0,
    ..MAGERY_BASE
};

/// `ManaVampireSpell`: Ort Sanct.
pub static MANA_VAMPIRE_SPELL: SpellType = SpellType {
    full_name: "Server.Spells.Seventh.ManaVampireSpell",
    parent: Some(&MAGERY_SPELL),
    target_range: Some(muo_content::spells::targeting::target_range),
    info: SpellInfo::new(
        "Mana Vampire",
        "Ort Sanct",
        &[
            reagent::BLACK_PEARL,
            reagent::BLOODMOSS,
            reagent::MANDRAKE_ROOT,
            reagent::SPIDERS_SILK,
        ],
    )
    .action(221)
    .hand_effect(9032),
    on_cast,
    family: Some(&MANA_VAMPIRE_FAMILY),
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
        let caster = spell::caster(w, sp).unwrap_or(caster);

        spell::caster_hurt(w, m);
        mb::set_paralyzed(w, m, false);

        let mut to_drain = 0;

        if core::aos(w) {
            let ty = spell::spell_type(w, sp).expect("the spell is casting");
            to_drain =
                ((ty.get_damage_skill)(w, sp, caster) - (ty.get_resist_skill)(w, sp, m)) as i32;

            if !w.mobile(m).is_some_and(|x| x.player) {
                to_drain /= 2;
            }
        } else if check_resisted(w, sp, m) {
            mm::send_localized_message(w, m, 501783, "", 0x3B2); // You feel yourself resisting magical energy.
        } else {
            to_drain = w.mobile(m).map_or(0, |x| x.mana);
        }

        // Will not drain more than the target has currently
        let mana = w.mobile(m).map_or(0, |x| x.mana);
        to_drain = to_drain.min(mana);

        props_b::set_mana(w, m, mana - to_drain);
        let cmana = w.mobile(caster).map_or(0, |x| x.mana);
        props_b::set_mana(w, caster, cmana + to_drain);

        if core::aos(w) {
            effects::fixed_particles_hue(w, m, 0x374A, 1, 15, 5054, 23, 7, EffectLayer::Head);
            effects::play_sound(w, m, 0x1F9);

            effects::fixed_particles(w, caster, 0x0000, 10, 5, 2054, EffectLayer::Head);
        } else {
            effects::fixed_particles(w, m, 0x374A, 10, 15, 5054, EffectLayer::Head);
            effects::play_sound(w, m, 0x1F9);
        }

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
