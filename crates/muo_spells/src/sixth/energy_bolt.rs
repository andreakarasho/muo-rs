//! The Energy Bolt spell.

use muo_content::core;
use muo_content::spells::helper::{self, SpellDamageSplit};
use muo_content::spells::magery::{MAGERY_SPELL, SIXTH_CIRCLE, check_resisted};
use muo_content::spells::spell::{self, SpellId, SpellType};
use muo_content::spells::targeting::{SpellTargetKind, begin_spell_target};
use muo_content::spells::{SpellCircle, SpellInfo, reagent};
use muo_primitives::utility;
use muo_server::World;
use muo_server::effects::EffectLayer;
use muo_server::ids::EntityId;
use muo_server::mobile::part2::{effects, messages as mm};
use muo_server::target::{TargetFlags, Targeted};

use crate::common::{mobile_of, turn_to};

/// `EnergyBoltSpell`: Corp Por.
pub static ENERGY_BOLT_SPELL: SpellType = SpellType {
    full_name: "Server.Spells.Sixth.EnergyBoltSpell",
    parent: Some(&MAGERY_SPELL),
    target_range: Some(muo_content::spells::targeting::target_range),
    info: SpellInfo::new(
        "Energy Bolt",
        "Corp Por",
        &[reagent::BLACK_PEARL, reagent::NIGHTSHADE],
    )
    .action(230)
    .hand_effect(9022),
    delayed_damage: true,
    on_cast,
    family: Some(&SIXTH_CIRCLE),
    ..MAGERY_SPELL
};

fn target(w: &mut World, sp: SpellId, targeted: Targeted) {
    let Some(mut m) = mobile_of(targeted) else {
        return;
    };
    if spell::check_h_sequence(w, sp, m) {
        let Some(mut source) = spell::caster(w, sp) else {
            return;
        };
        turn_to(w, source, m);
        helper::check_reflect(w, SpellCircle::Sixth as i32, &mut source, &mut m);

        let damage = if core::aos(w) {
            f64::from(spell::get_new_aos_damage(w, sp, 40, 1, 5, true, Some(m)))
        } else {
            let mut d = f64::from(utility::random_from(&mut w.rng, 24, 18));
            if check_resisted(w, sp, m) {
                d *= 0.75;
                mm::send_localized_message(w, m, 501783, "", 0x3B2); // You feel yourself resisting magical energy.
            }
            // Scale damage based on evalint and resist
            let ty = spell::spell_type(w, sp).expect("the spell is casting");
            d * (ty.get_damage_scalar)(w, sp, m)
        };

        effects::moving_particles_layer(
            w,
            source,
            EntityId::Mobile(m),
            0x379F,
            7,
            0,
            false,
            true,
            0,
            0,
            3043,
            4043,
            0x211,
            EffectLayer::RightHand,
            0,
        );
        effects::play_sound(w, source, 0x20A);

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
