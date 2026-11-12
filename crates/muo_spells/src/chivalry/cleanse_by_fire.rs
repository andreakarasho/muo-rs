//! The Cleanse By Fire spell.

use muo_content::misc::aos_helpers::{self, DamageOptions, DamageSplit};
use muo_content::spells::SpellInfo;
use muo_content::spells::spell::{self, SpellId, SpellType};
use muo_content::spells::targeting::{SpellTargetKind, begin_spell_target};
use muo_primitives::{Point3D, TimeSpan, utility};
use muo_server::World;
use muo_server::effects::{EffectLayer, Targeted as FxTargeted, send_moving_particles_layer};
use muo_server::mobile::base as mb;
use muo_server::mobile::part2::{effects, messages as mm};
use muo_server::skills::{self, SkillName};
use muo_server::target::{TargetFlags, Targeted};

use super::paladin::{PALADIN_BASE, PALADIN_SPELL, PaladinType, check_cast_sudden_death, power};
use crate::common::{mobile_of, poison_level, turn_to};

static CLEANSE_BY_FIRE: PaladinType = PaladinType {
    required_skill: 5.0,
    required_mana: 10,
    required_tithing: 10,
    mantra_number: 1060718, // Expor Flamus
    ..PALADIN_BASE
};

/// `CleanseByFireSpell`: Expor Flamus.
pub static CLEANSE_BY_FIRE_SPELL: SpellType = SpellType {
    full_name: "Server.Spells.Chivalry.CleanseByFireSpell",
    parent: Some(&PALADIN_SPELL),
    target_range: Some(muo_content::spells::targeting::target_range),
    info: SpellInfo::new("Cleanse By Fire", "Expor Flamus", &[])
        .action(-1)
        .hand_effect(9002),
    cast_delay_base: |_, _| TimeSpan::from_secs(1),
    on_cast,
    check_cast: check_cast_sudden_death,
    family: Some(&CLEANSE_BY_FIRE),
    ..PALADIN_SPELL
};

fn target(w: &mut World, sp: SpellId, targeted: Targeted) {
    let Some(m) = mobile_of(targeted) else {
        return;
    };
    let Some(caster) = spell::caster(w, sp) else {
        return;
    };
    if w.mobile(m).is_none_or(|x| x.poison.is_none()) {
        mm::send_localized_message(w, caster, 1060176, "", 0x3B2); // That creature is not poisoned!
    } else if spell::check_b_sequence(w, sp, m, false) {
        turn_to(w, caster, m);

        /* Cures the target of poisons, but causes the caster to be burned by fire damage for 13-55 hit points.
         * The amount of fire damage is lessened if the caster has high Karma.
         */
        if let Some(level) = poison_level(w, m) {
            // Cleanse by fire is now difficulty based
            let mut chance_to_cure = 10000
                + (skills::value(w, caster, SkillName::Chivalry) * 75.0) as i32
                - (level + 1) * 2000;
            chance_to_cure /= 100;

            if chance_to_cure > utility::random(&mut w.rng, 100) {
                if mb::cure_poison(w, m, Some(caster)) {
                    if caster != m {
                        mm::send_localized_message(w, caster, 1010058, "", 0x3B2); // You have cured the target of all poisons!
                    }
                    mm::send_localized_message(w, m, 1010059, "", 0x3B2); // You have been cured of all poisons.
                }
            } else {
                mm::send_localized_message(w, caster, 1010060, "", 0x3B2); // You have failed to cure your target!
            }
        }

        effects::play_sound(w, m, 0x1E0);
        effects::fixed_particles_hue(w, m, 0x373A, 1, 15, 5012, 3, 2, EffectLayer::Waist);

        let (loc, map) = w.mobile(m).map(|x| (x.location, x.map)).unwrap_or_default();
        let from = FxTargeted::point(Point3D::new(loc.x, loc.y, loc.z - 5), map);
        let to = FxTargeted::point(Point3D::new(loc.x, loc.y, loc.z + 45), map);
        send_moving_particles_layer(
            w,
            from,
            to,
            0x374B,
            1,
            0,
            false,
            false,
            63,
            2,
            9501,
            1,
            0,
            i32::from(EffectLayer::Head.0),
            0x100,
        );

        effects::play_sound(w, caster, 0x208);
        effects::fixed_particles_hue(w, caster, 0x3709, 1, 30, 9934, 0, 7, EffectLayer::Waist);

        let damage = (50 - power(w, sp, 4)).clamp(13, 55);
        aos_helpers::damage(
            w,
            caster,
            Some(caster),
            damage,
            DamageSplit {
                fire: 100,
                ..Default::default()
            },
            DamageOptions {
                keep_alive: true,
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
        TargetFlags::BENEFICIAL,
        false,
        SpellTargetKind::Mobile,
        target,
    );
}
