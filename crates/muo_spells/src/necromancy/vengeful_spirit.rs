//! The Vengeful Spirit spell.

use muo_content::creature::control;
use muo_content::spells::hooks::hooks;
use muo_content::spells::spell::{self, SpellId, SpellType};
use muo_content::spells::targeting::{SpellTargetKind, begin_spell_target};
use muo_content::spells::{SpellInfo, reagent};
use muo_primitives::TimeSpan;
use muo_server::World;
use muo_server::effects::EffectLayer;
use muo_server::mobile::part2::{effects, messages as mm};
use muo_server::target::{TargetFlags, Targeted};

use super::necromancer::{NECRO_BASE, NECROMANCER_SPELL, NecroType};
use crate::common::{mobile_of, turn_to};

static VENGEFUL_SPIRIT: NecroType = NecroType {
    required_skill: 80.0,
    required_mana: 41,
    ..NECRO_BASE
};

/// `VengefulSpiritSpell`: Kal Xen Bal Beh.
pub static VENGEFUL_SPIRIT_SPELL: SpellType = SpellType {
    full_name: "Server.Spells.Necromancy.VengefulSpiritSpell",
    parent: Some(&NECROMANCER_SPELL),
    target_range: Some(muo_content::spells::targeting::target_range),
    info: SpellInfo::new(
        "Vengeful Spirit",
        "Kal Xen Bal Beh",
        &[reagent::BAT_WING, reagent::GRAVE_DUST, reagent::PIG_IRON],
    )
    .action(203)
    .hand_effect(9031),
    cast_delay_base: |_, _| TimeSpan::from_secs(2),
    on_cast,
    check_cast,
    family: Some(&VENGEFUL_SPIRIT),
    ..NECROMANCER_SPELL
};

fn target(w: &mut World, sp: SpellId, targeted: Targeted) {
    let Some(m) = mobile_of(targeted) else {
        return;
    };
    let Some(caster) = spell::caster(w, sp) else {
        return;
    };
    if caster == m {
        mm::send_localized_message(w, caster, 1061832, "", 0x3B2); // You cannot exact vengeance on yourself.
    } else if spell::check_h_sequence(w, sp, m) {
        turn_to(w, caster, m);

        /* Summons a Revenant which haunts the target until either the target or the Revenant is dead.
         * Revenants have the ability to track down their targets wherever they may travel.
         * A Revenant's strength is determined by the Necromancy and Spirit Speak skills of the Caster.
         * The effect lasts for ((Spirit Speak skill level * 80) / 120) + 10 seconds.
         */
        let ty = spell::spell_type(w, sp).expect("the spell is casting");
        let duration =
            TimeSpan::from_secs_f64((ty.get_damage_skill)(w, sp, caster) * 80.0 / 120.0 + 10.0);

        let make = hooks(w).and_then(|h| h.new_revenant);
        if let Some(rev) = make.and_then(|f| f(w, caster, m, duration)) {
            let loc = w.mobile(m).map(|x| x.location).unwrap_or_default();
            if control::summon(
                w,
                rev,
                false,
                caster,
                loc,
                0x81,
                TimeSpan::from_secs_f64(duration.total_seconds() + 2.0),
                None,
            ) {
                effects::fixed_particles(w, rev, 0x373A, 1, 15, 9909, EffectLayer::Waist);
            }
        }
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

fn check_cast(w: &mut World, sp: SpellId) -> bool {
    if !(NECROMANCER_SPELL.check_cast)(w, sp) {
        return false;
    }
    let Some(caster) = spell::caster(w, sp) else {
        return false;
    };
    let (followers, max) = w
        .mobile(caster)
        .map_or((0, 0), |c| (c.followers, c.followers_max));
    if followers + 3 > max {
        mm::send_localized_message(w, caster, 1049645, "", 0x3B2); // You have too many followers to summon that creature.
        return false;
    }
    true
}
