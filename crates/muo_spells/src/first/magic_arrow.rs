//! The Magic Arrow spell.

use muo_content::core;
use muo_content::spells::helper::{self, SpellDamageSplit};
use muo_content::spells::magery::{FIRST_CIRCLE, MAGERY_SPELL, check_resisted};
use muo_content::spells::spell::{self, SpellId, SpellType};
use muo_content::spells::targeting::{SpellTargetKind, begin_spell_target};
use muo_content::spells::{SpellCircle, SpellInfo, reagent};
use muo_primitives::utility;
use muo_server::World;
use muo_server::effects::EffectLayer;
use muo_server::ids::EntityId;
use muo_server::mobile::part2::{effects, messages as mm};
use muo_server::target::{TargetFlags, Targeted};

/// `MagicArrowSpell`: In Por Ylem.
pub static MAGIC_ARROW_SPELL: SpellType = SpellType {
    full_name: "Server.Spells.First.MagicArrowSpell",
    parent: Some(&MAGERY_SPELL),
    target_range: Some(muo_content::spells::targeting::target_range),
    info: SpellInfo::new("Magic Arrow", "In Por Ylem", &[reagent::SULFUROUS_ASH])
        .action(212)
        .hand_effect(9041),
    delayed_damage: true,
    // AOSNoDelayedDamageStackingSelf
    delayed_damage_family: Some(&[]),
    on_cast,
    family: Some(&FIRST_CIRCLE),
    ..MAGERY_SPELL
};

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

        if core::sa(w) && spell::has_delayed_damage_context(w, sp, m) {
            let ty = spell::spell_type(w, sp).expect("the spell is casting");
            (ty.do_hurt_fizzle)(w, sp);
            return;
        }

        helper::check_reflect(w, SpellCircle::First as i32, &mut source, &mut m);

        let damage = if core::aos(w) {
            f64::from(spell::get_new_aos_damage(w, sp, 10, 1, 4, true, Some(m)))
        } else {
            let mut d = f64::from(utility::random_from(&mut w.rng, 4, 4));
            if check_resisted(w, sp, m) {
                d *= 0.75;
                mm::send_localized_message(w, m, 501783, "", 0x3B2); // You feel yourself resisting magical energy.
            }
            let ty = spell::spell_type(w, sp).expect("the spell is casting");
            d * (ty.get_damage_scalar)(w, sp, m)
        };

        effects::moving_particles_layer(
            w,
            source,
            EntityId::Mobile(m),
            0x36E4,
            5,
            0,
            false,
            false,
            0,
            0,
            3006,
            0,
            0,
            EffectLayer::RightHand,
            0,
        );
        effects::play_sound(w, source, 0x1E5);

        helper::damage_aos_spell(
            w,
            sp,
            m,
            damage,
            SpellDamageSplit {
                fire: 100,
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
