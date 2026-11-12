//! The Close Wounds spell.

use muo_content::creature;
use muo_content::items::weapon_ability::is_wounded;
use muo_content::spells::SpellInfo;
use muo_content::spells::helper;
use muo_content::spells::spell::{self, SpellId, SpellType};
use muo_content::spells::targeting::{SpellTargetKind, begin_spell_target};
use muo_primitives::{MessageType, TimeSpan, utility};
use muo_server::World;
use muo_server::effects::EffectLayer;
use muo_server::mobile::base as mb;
use muo_server::mobile::part2::{effects, messages as mm};
use muo_server::target::{TargetFlags, Targeted};

use super::paladin::{PALADIN_BASE, PALADIN_SPELL, PaladinType, check_cast_sudden_death, power};
use crate::common::{mobile_of, turn_to};

static CLOSE_WOUNDS: PaladinType = PaladinType {
    required_skill: 0.0,
    required_mana: 10,
    required_tithing: 10,
    mantra_number: 1060719, // Obsu Vulni
    ..PALADIN_BASE
};

/// `CloseWoundsSpell`: Obsu Vulni.
pub static CLOSE_WOUNDS_SPELL: SpellType = SpellType {
    full_name: "Server.Spells.Chivalry.CloseWoundsSpell",
    parent: Some(&PALADIN_SPELL),
    target_range: Some(muo_content::spells::targeting::target_range),
    info: SpellInfo::new("Close Wounds", "Obsu Vulni", &[])
        .action(-1)
        .hand_effect(9002),
    cast_delay_base: |_, _| TimeSpan::from_millis(1500),
    on_cast,
    check_cast: check_cast_sudden_death,
    family: Some(&CLOSE_WOUNDS),
    ..PALADIN_SPELL
};

fn target(w: &mut World, sp: SpellId, targeted: Targeted) {
    let Some(m) = mobile_of(targeted) else {
        return;
    };
    let Some(caster) = spell::caster(w, sp) else {
        return;
    };
    let caster_loc = w.mobile(caster).map(|c| c.location).unwrap_or_default();
    let (hits, hits_max) = (w.mobile(m).map_or(0, |x| x.hits), mb::hits_max(w, m));

    if !mb::in_range_point3d(w, m, caster_loc, 2) {
        mm::send_localized_message(w, caster, 1060178, "", 0x3B2); // You are too far away to perform that action!
    } else if creature::misc::is_animated_dead(w, m) {
        mm::send_localized_message(w, caster, 1061654, "", 0x3B2); // You cannot heal that which is not alive.
    } else if mb::is_dead_bonded_pet(w, m) {
        mm::send_localized_message(w, caster, 1060177, "", 0x3B2); // You cannot heal a creature that is already dead!
    } else if hits >= hits_max {
        mm::send_localized_message(w, caster, 500955, "", 0x3B2); // That being is not damaged!
    } else if w.mobile(m).is_some_and(|x| x.poison.is_some()) || is_wounded(w, m) {
        let n = if caster == m { 1005000 } else { 1010398 };
        mm::local_overhead_message_number(w, caster, MessageType::REGULAR, 0x3B2, n, "");
    } else if spell::check_b_sequence(w, sp, m, false) {
        turn_to(w, caster, m);

        /* Heals the target for 7 to 39 points of damage.
         * The caster's Karma affects the amount of damage healed.
         */
        // TODO: Should caps be applied?
        let mut to_heal =
            (power(w, sp, 6) + utility::random_min_max(&mut w.rng, 0, 2)).clamp(7, 39);

        if hits + to_heal > hits_max {
            to_heal = hits_max - hits;
        }

        helper::heal(w, to_heal, m, Some(caster), false);

        // You have had ~1_HEALED_AMOUNT~ hit points of damage healed.
        mm::send_localized_message(w, m, 1060203, &to_heal.to_string(), 0x3B2);

        effects::play_sound(w, m, 0x202);
        effects::fixed_particles_hue(w, m, 0x376A, 1, 62, 9923, 3, 3, EffectLayer::Waist);
        effects::fixed_particles_hue(w, m, 0x3779, 1, 46, 9502, 5, 3, EffectLayer::Waist);
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
