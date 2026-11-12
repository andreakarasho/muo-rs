//! The Mind Blast spell.

use muo_content::core;
use muo_content::spells::helper::{self, SpellDamageSplit};
use muo_content::spells::magery::{FIFTH_CIRCLE, MAGERY_SPELL, check_resisted};
use muo_content::spells::spell::{self, SpellId, SpellType};
use muo_content::spells::targeting::{SpellTargetKind, begin_spell_target};
use muo_content::spells::{SpellCircle, SpellInfo, reagent};
use muo_primitives::{TimeSpan, utility};
use muo_server::World;
use muo_server::effects::EffectLayer;
use muo_server::ids::MobileId;
use muo_server::mobile::base as mb;
use muo_server::mobile::part2::{effects, messages as mm};
use muo_server::skills::{self, SkillName};
use muo_server::target::{TargetFlags, Targeted};
use muo_server::timer;

use crate::common::{mobile_of, turn_to};

/// `MindBlastSpell`: Por Corp Wis.
pub static MIND_BLAST_SPELL: SpellType = SpellType {
    full_name: "Server.Spells.Fifth.MindBlastSpell",
    parent: Some(&MAGERY_SPELL),
    target_range: Some(muo_content::spells::targeting::target_range),
    info: SpellInfo::new(
        "Mind Blast",
        "Por Corp Wis",
        &[
            reagent::BLACK_PEARL,
            reagent::MANDRAKE_ROOT,
            reagent::NIGHTSHADE,
            reagent::SULFUROUS_ASH,
        ],
    )
    .action(218)
    .hand_effect(9002)
    .pre_aos(218, 9032),
    delayed_damage_unless_aos: true,
    on_cast,
    get_slayer_damage_scalar: |_, _, _| 1.0,
    family: Some(&FIFTH_CIRCLE),
    ..MAGERY_SPELL
};

fn target(w: &mut World, sp: SpellId, targeted: Targeted) {
    let Some(m) = mobile_of(targeted) else {
        return;
    };
    let Some(caster) = spell::caster(w, sp) else {
        return;
    };
    if core::aos(w) {
        if mb::can_be_harmful(w, caster, m) && spell::check_sequence(w, sp) {
            let mut from = caster;
            let mut tgt = m;
            turn_to(w, from, tgt);
            helper::check_reflect(w, SpellCircle::Fifth as i32, &mut from, &mut tgt);

            let int = w.mobile(caster).map_or(0, |c| (c.ty.int)(w, caster));
            let damage = (((skills::value(w, caster, SkillName::Magery) + f64::from(int)) / 5.0)
                as i32)
                .min(60);

            timer::start(w, TimeSpan::from_secs(1), TimeSpan::ZERO, 1, move |w| {
                aos_delay_callback(w, sp, caster, tgt, m, damage)
            });
        }
    } else if spell::check_h_sequence(w, sp, m) {
        let mut from = caster;
        let mut tgt = m;
        turn_to(w, from, tgt);
        helper::check_reflect(w, SpellCircle::Fifth as i32, &mut from, &mut tgt);

        // Algorithm: (highestStat - lowestStat) / 2 [- 50% if resisted]
        let (s, d, i) = w.mobile(tgt).map_or((0, 0, 0), |x| {
            ((x.ty.str)(w, tgt), (x.ty.dex)(w, tgt), (x.ty.int)(w, tgt))
        });
        let mut highest = s;
        let mut lowest = s;
        if d > highest {
            highest = d;
        }
        if d < lowest {
            lowest = d;
        }
        if i > highest {
            highest = i;
        }
        if i < lowest {
            lowest = i;
        }
        if highest > 150 {
            highest = 150;
        }
        if lowest > 150 {
            lowest = 150;
        }

        let ty = spell::spell_type(w, sp).expect("the spell is casting");
        let scalar = (ty.get_damage_scalar)(w, sp, m);
        let mut damage = (scalar * f64::from(highest - lowest) / 2.0).min(45.0); // Many users prefer 3 or 4

        if check_resisted(w, sp, tgt) {
            damage /= 2.0;
            mm::send_localized_message(w, tgt, 501783, "", 0x3B2); // You feel yourself resisting magical energy.
        }

        effects::fixed_particles(w, from, 0x374A, 10, 15, 2038, EffectLayer::Head);
        effects::fixed_particles(w, tgt, 0x374A, 10, 15, 5038, EffectLayer::Head);
        effects::play_sound(w, tgt, 0x213);

        helper::damage_aos_spell(
            w,
            sp,
            tgt,
            damage,
            SpellDamageSplit {
                cold: 100,
                ..Default::default()
            },
        );
    }
}

fn aos_delay_callback(
    w: &mut World,
    sp: SpellId,
    caster: MobileId,
    target: MobileId,
    defender: MobileId,
    damage: i32,
) {
    if mb::harmful_check(w, caster, defender) {
        let d = utility::random_min_max(&mut w.rng, damage, damage + 4);
        helper::damage_aos_spell(
            w,
            sp,
            target,
            f64::from(d),
            SpellDamageSplit {
                cold: 100,
                ..Default::default()
            },
        );
        effects::fixed_particles_hue(w, target, 0x374A, 10, 15, 5038, 1181, 2, EffectLayer::Head);
        effects::play_sound(w, target, 0x213);
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
