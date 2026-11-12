//! The Noble Sacrifice spell.

use muo_content::creature;
use muo_content::spells::SpellInfo;
use muo_content::spells::helper;
use muo_content::spells::spell::{self, SpellId, SpellType};
use muo_gumps::m_z::resurrect;
use muo_primitives::{TimeSpan, utility};
use muo_server::World;
use muo_server::effects::EffectLayer;
use muo_server::item::deps;
use muo_server::mobile::base as mb;
use muo_server::mobile::part1::props_b;
use muo_server::mobile::part2::{effects, messages as mm};
use muo_server::region as regions;

use super::paladin::{PALADIN_BASE, PALADIN_SPELL, PaladinType, power};
use super::{end_curse_effects, remove_curse_buffs, remove_stat_curses};
use crate::common::{alive, mobiles_in_range};

static NOBLE_SACRIFICE: PaladinType = PaladinType {
    required_skill: 65.0,
    required_mana: 20,
    required_tithing: 30,
    mantra_number: 1060725, // Dium Prostra
    ..PALADIN_BASE
};

/// `NobleSacrificeSpell`: Dium Prostra.
pub static NOBLE_SACRIFICE_SPELL: SpellType = SpellType {
    full_name: "Server.Spells.Chivalry.NobleSacrificeSpell",
    parent: Some(&PALADIN_SPELL),
    info: SpellInfo::new("Noble Sacrifice", "Dium Prostra", &[])
        .action(-1)
        .hand_effect(9002),
    cast_delay_base: |_, _| TimeSpan::from_millis(1500),
    on_cast,
    blocks_movement: |_, _| false,
    family: Some(&NOBLE_SACRIFICE),
    ..PALADIN_SPELL
};

fn on_cast(w: &mut World, sp: SpellId) {
    let Some(caster) = spell::caster(w, sp) else {
        return;
    };
    if spell::check_sequence(w, sp) {
        let (cloc, cmap) = w
            .mobile(caster)
            .map(|c| (c.location, c.map))
            .unwrap_or_default();
        let mut pool = Vec::new();
        for m in mobiles_in_range(w, cmap, cloc, 3) {
            if !creature::misc::is_animated_dead(w, m)
                && caster != m
                && deps::in_los_mobile(w, m, caster)
                && mb::can_be_beneficial_full(w, caster, m, false, true)
                && creature::type_name(w, m) != "Golem"
            {
                pool.push(m);
            }
        }

        effects::play_sound(w, caster, 0x244);
        effects::fixed_particles_hue(w, caster, 0x3709, 1, 30, 9965, 5, 7, EffectLayer::Waist);
        effects::fixed_particles_hue(w, caster, 0x376A, 1, 30, 9502, 5, 3, EffectLayer::Waist);

        /* Attempts to Resurrect, Cure and Heal all targets in a radius around the caster.
         * If any target is successfully assisted, the Paladin's current
         * Hit Points, Mana and Stamina are set to 1.
         * Amount of damage healed is affected by the Caster's Karma, from 8 to 24 hit points.
         */
        let mut sacrifice = false;

        // TODO: Is there really a resurrection chance?
        let karma = w.mobile(caster).map_or(0, |c| c.karma);
        let res_chance = 0.1 + 0.9 * f64::from(karma) / 10000.0;

        for m in pool {
            if !alive(w, m) {
                let r = props_b::region(w, m);
                if regions::is_part_of_name(w, r, "Khaldun", false) {
                    // The veil of death in this area is too strong and resists thy efforts to restore life.
                    mm::send_localized_message(w, caster, 1010395, "", 0x3B2);
                } else if res_chance > utility::random_double(&mut w.rng) {
                    effects::fixed_particles_hue(
                        w,
                        m,
                        0x375A,
                        1,
                        15,
                        5005,
                        5,
                        3,
                        EffectLayer::Head,
                    );
                    let gump = Box::new(resurrect::new(
                        Some(caster),
                        resurrect::ResurrectMessage::Generic,
                        false,
                        0.0,
                        0,
                    ));
                    muo_content::gumps::system::send_gump_to_mobile(w, m, gump, false);

                    sacrifice = true;
                }
            } else {
                let mut send_effect = false;

                if w.mobile(m).is_some_and(|x| x.poison.is_some())
                    && mb::cure_poison(w, m, Some(caster))
                {
                    mb::do_beneficial(w, caster, m);

                    if caster != m {
                        mm::send_localized_message(w, caster, 1010058, "", 0x3B2); // You have cured the target of all poisons!
                    }

                    mm::send_localized_message(w, m, 1010059, "", 0x3B2); // You have been cured of all poisons.
                    send_effect = true;
                    sacrifice = true;
                }

                if w.mobile(m).map_or(0, |x| x.hits) < mb::hits_max(w, m) {
                    let to_heal =
                        (power(w, sp, 10) + utility::random_min_max(&mut w.rng, 0, 2)).clamp(8, 24);

                    mb::do_beneficial(w, caster, m);
                    helper::heal(w, to_heal, m, Some(caster), true);
                    send_effect = true;
                }

                if remove_stat_curses(w, m) {
                    send_effect = true;
                }

                if w.mobile(m).is_some_and(|x| x.paralyzed) {
                    mb::set_paralyzed(w, m, false);
                    send_effect = true;
                }

                send_effect = end_curse_effects(w, m) || send_effect;

                // TODO: Move these into their respective end effect methods
                remove_curse_buffs(w, m);

                if send_effect {
                    effects::fixed_particles_hue(
                        w,
                        m,
                        0x375A,
                        1,
                        15,
                        5005,
                        5,
                        3,
                        EffectLayer::Head,
                    );
                    sacrifice = true;
                }
            }
        }

        if sacrifice {
            let female = w.mobile(caster).is_some_and(|c| c.cur_body().is_female());
            effects::play_sound(w, caster, if female { 0x150 } else { 0x423 });
            props_b::set_hits(w, caster, 1);
            props_b::set_stam(w, caster, 1);
            props_b::set_mana(w, caster, 1);
        }
    }

    spell::finish_sequence(w, sp);
}
