//! The Remove Curse spell.

use muo_content::core;
use muo_content::spells::SpellInfo;
use muo_content::spells::spell::{self, SpellId, SpellType};
use muo_content::spells::targeting::{SpellTargetKind, begin_spell_target};
use muo_primitives::{Point3D, TimeSpan, utility};
use muo_server::World;
use muo_server::effects::{EffectLayer, Targeted as FxTargeted, send_moving_particles_layer};
use muo_server::mobile::base as mb;
use muo_server::mobile::part2::effects;
use muo_server::target::{TargetFlags, Targeted};

use super::paladin::{PALADIN_BASE, PALADIN_SPELL, PaladinType, check_cast_sudden_death};
use super::{end_curse_effects, remove_curse_buffs, remove_stat_curses};
use crate::common::{mobile_of, turn_to};

static REMOVE_CURSE: PaladinType = PaladinType {
    required_skill: 5.0,
    required_mana: 20,
    required_tithing: 10,
    mantra_number: 1060726, // Extermo Vomica
    ..PALADIN_BASE
};

/// `RemoveCurseSpell`: Extermo Vomica.
pub static REMOVE_CURSE_SPELL: SpellType = SpellType {
    full_name: "Server.Spells.Chivalry.RemoveCurseSpell",
    parent: Some(&PALADIN_SPELL),
    target_range: Some(muo_content::spells::targeting::target_range),
    info: SpellInfo::new("Remove Curse", "Extermo Vomica", &[])
        .action(-1)
        .hand_effect(9002),
    // https://uo.com/wiki/ultima-online-wiki/publish-notes/publish-108/ - 1.5 -> 2.0
    // According to tests, this includes the 0.25s added penalty, so we are adjusting to 1.75s base.
    cast_delay_base: |w, _| {
        TimeSpan::from_millis(
            if core::expansion(w) >= muo_data::expansion::Expansion::EJ {
                1750
            } else {
                1500
            },
        )
    },
    on_cast,
    check_cast: check_cast_sudden_death,
    family: Some(&REMOVE_CURSE),
    ..PALADIN_SPELL
};

fn target(w: &mut World, sp: SpellId, targeted: Targeted) {
    let Some(m) = mobile_of(targeted) else {
        return;
    };
    let Some(caster) = spell::caster(w, sp) else {
        return;
    };
    if spell::check_b_sequence(w, sp, m, false) {
        turn_to(w, caster, m);

        /* Attempts to remove all Curse effects from Target.
         * Curses include Mage spells such as Clumsy, Weaken, Feeblemind and Paralyze
         * as well as all Necromancer curses.
         * Chance of removing curse is affected by Caster's Karma.
         */
        let karma = w.mobile(caster).map_or(0, |c| c.karma);
        let chance = if karma < -5000 {
            0
        } else if karma < 0 {
            (f64::from(20000 + karma).sqrt()) as i32 - 122
        } else if karma < 5625 {
            (f64::from(karma).sqrt()) as i32 + 25
        } else {
            100
        };

        if chance > utility::random(&mut w.rng, 100) {
            effects::play_sound(w, m, 0xF6);
            effects::play_sound(w, m, 0x1F7);
            effects::fixed_particles_hue(w, m, 0x3709, 1, 30, 9963, 13, 3, EffectLayer::Head);

            let (loc, _) = w.mobile(m).map(|x| (x.location, x.map)).unwrap_or_default();
            let map = w.mobile(caster).and_then(|c| c.map);
            let from = FxTargeted::point(Point3D::new(loc.x, loc.y, loc.z - 10), map);
            let to = FxTargeted::point(Point3D::new(loc.x, loc.y, loc.z + 50), map);
            send_moving_particles_layer(
                w,
                from,
                to,
                0x2255,
                1,
                0,
                false,
                false,
                13,
                3,
                9501,
                1,
                0,
                i32::from(EffectLayer::Head.0),
                0x100,
            );

            remove_stat_curses(w, m);

            mb::set_paralyzed(w, m, false);

            end_curse_effects(w, m);

            remove_curse_buffs(w, m);
        } else {
            effects::play_sound(w, m, 0x1DF);
        }
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
