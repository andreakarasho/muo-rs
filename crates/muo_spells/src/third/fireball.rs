//! The Fireball spell.

use muo_content::core;
use muo_content::spells::helper::{self, SpellDamageSplit};
use muo_content::spells::magery::{MAGERY_SPELL, THIRD_CIRCLE, check_resisted};
use muo_content::spells::spell::{self, SpellId, SpellType};
use muo_content::spells::targeting::{SpellTargetKind, begin_spell_target};
use muo_content::spells::{SpellCircle, SpellInfo, reagent};
use muo_primitives::utility;
use muo_server::World;
use muo_server::effects::EffectLayer;
use muo_server::ids::EntityId;
use muo_server::mobile::part2::{effects, messages as mm};
use muo_server::target::{TargetFlags, Targeted};

/// `FireballSpell`: Vas Flam, a fire bolt whose damage lands after the missile flies.
pub static FIREBALL_SPELL: SpellType = SpellType {
    full_name: "Server.Spells.Third.FireballSpell",
    parent: Some(&MAGERY_SPELL),
    target_range: Some(muo_content::spells::targeting::target_range),
    info: SpellInfo::new("Fireball", "Vas Flam", &[reagent::BLACK_PEARL])
        .action(203)
        .hand_effect(9041),
    delayed_damage: true,
    on_cast,
    family: Some(&THIRD_CIRCLE),
    ..MAGERY_SPELL
};

/// `Target(Mobile m)`.
fn target(w: &mut World, sp: SpellId, targeted: Targeted) {
    let Targeted::Entity(EntityId::Mobile(mut m)) = targeted else {
        return;
    };

    if spell::check_h_sequence(w, sp, m) {
        let Some(mut source) = spell::caster(w, sp) else {
            return;
        };

        helper::turn(
            w,
            source,
            Some(helper::Point3dRef::Entity(EntityId::Mobile(m))),
        );

        helper::check_reflect(w, SpellCircle::Third as i32, &mut source, &mut m);

        let damage: f64 = if core::aos(w) {
            f64::from(spell::get_new_aos_damage(w, sp, 19, 1, 5, true, Some(m)))
        } else {
            let mut d = f64::from(utility::random_from(&mut w.rng, 10, 7));

            if check_resisted(w, sp, m) {
                d *= 0.75;

                mm::send_localized_message(w, m, 501783, "", 0x3B2); // You feel yourself resisting magical energy.
            }

            let ty = spell::spell_type(w, sp).expect("the spell is casting");
            d *= (ty.get_damage_scalar)(w, sp, m);
            d
        };

        effects::moving_particles_layer(
            w,
            source,
            EntityId::Mobile(m),
            0x36D4,
            7,
            0,
            false,
            true,
            0,
            0,
            9502,
            4019,
            0x160,
            EffectLayer::RightHand,
            0,
        );
        effects::play_sound(w, source, if core::aos(w) { 0x15E } else { 0x44B });

        helper::damage_aos_spell(
            w,
            sp,
            m,
            damage,
            SpellDamageSplit {
                phys: 0,
                fire: 100,
                cold: 0,
                pois: 0,
                nrgy: 0,
                ..Default::default()
            },
        );
    }
}

/// `OnCast()`.
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
