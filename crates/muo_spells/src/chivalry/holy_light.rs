//! The Holy Light spell.

use muo_content::core;
use muo_content::spells::SpellInfo;
use muo_content::spells::helper::{self, SpellDamageSplit};
use muo_content::spells::spell::{self, SpellId, SpellType};
use muo_primitives::{Point3D, TimeSpan, utility};
use muo_server::World;
use muo_server::effects::send_location_particles_full;
use muo_server::item::{deps, effect_item};
use muo_server::mobile::base as mb;
use muo_server::mobile::part2::effects;

use super::paladin::{PALADIN_BASE, PALADIN_SPELL, PaladinType, power};
use crate::common::mobiles_in_range;

static HOLY_LIGHT: PaladinType = PaladinType {
    required_skill: 55.0,
    required_mana: 10,
    required_tithing: 10,
    mantra_number: 1060724, // Augus Luminos
    ..PALADIN_BASE
};

/// `HolyLightSpell`: Augus Luminos.
pub static HOLY_LIGHT_SPELL: SpellType = SpellType {
    full_name: "Server.Spells.Chivalry.HolyLightSpell",
    parent: Some(&PALADIN_SPELL),
    info: SpellInfo::new("Holy Light", "Augus Luminos", &[])
        .action(-1)
        .hand_effect(9002),
    cast_delay_base: |_, _| TimeSpan::from_millis(1750),
    delayed_damage: false,
    on_cast,
    blocks_movement: |_, _| false,
    family: Some(&HOLY_LIGHT),
    ..PALADIN_SPELL
};

fn on_cast(w: &mut World, sp: SpellId) {
    let Some(caster) = spell::caster(w, sp) else {
        return;
    };
    if spell::check_sequence(w, sp) {
        effects::play_sound(w, caster, 0x212);
        effects::play_sound(w, caster, 0x206);

        let (loc, map) = w
            .mobile(caster)
            .map(|c| (c.location, c.map))
            .unwrap_or_default();

        let e = effect_item::create_targeted(w, loc, map, effect_item::DEFAULT_DURATION);
        send_location_particles_full(w, e, 0x376A, 1, 29, 0x47D, 2, 9962, 0);
        let e = effect_item::create_targeted(
            w,
            Point3D::new(loc.x, loc.y, loc.z - 7),
            map,
            effect_item::DEFAULT_DURATION,
        );
        send_location_particles_full(w, e, 0x37C4, 1, 29, 0x47D, 2, 9502, 0);

        let mut queue = Vec::new();
        for m in mobiles_in_range(w, map, loc, 3) {
            if caster == m
                || !helper::valid_indirect_target(w, caster, m)
                || !mb::can_be_harmful_msg(w, caster, m, false)
                || (core::aos(w) && !deps::in_los_mobile(w, caster, m))
            {
                continue;
            }

            queue.push(m);
        }

        for m in queue {
            let damage =
                (power(w, sp, 10) + utility::random_min_max(&mut w.rng, 0, 2)).clamp(8, 24);

            mb::do_harmful(w, caster, m, false);
            helper::damage_aos_spell(
                w,
                sp,
                m,
                f64::from(damage),
                SpellDamageSplit {
                    nrgy: 100,
                    ..Default::default()
                },
            );
        }
    }

    spell::finish_sequence(w, sp);
}
