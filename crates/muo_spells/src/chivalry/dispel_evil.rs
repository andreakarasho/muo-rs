//! The Dispel Evil spell.

use muo_content::creature::{self, ctype};
use muo_content::spells::SpellInfo;
use muo_content::spells::helper;
use muo_content::spells::spell::{self, SpellId, SpellType};
use muo_primitives::{TimeSpan, utility};
use muo_server::World;
use muo_server::effects::EffectLayer;
use muo_server::item::effect_item;
use muo_server::mobile::base as mb;
use muo_server::mobile::part1::props_b;
use muo_server::mobile::part2::effects;
use muo_server::skills::{self, SkillName};

use super::paladin::{PALADIN_BASE, PALADIN_SPELL, PaladinType, power};
use crate::common::mobiles_in_range;

fn send_cast_effect(w: &mut World, sp: SpellId) {
    if let Some(caster) = spell::caster(w, sp) {
        effects::fixed_effect_hue(w, caster, 0x37C4, 10, 7, 4, 3); // At player
    }
}

static DISPEL_EVIL: PaladinType = PaladinType {
    required_skill: 35.0,
    required_mana: 10,
    required_tithing: 10,
    mantra_number: 1060721, // Dispiro Malas
    send_cast_effect,
};

/// `DispelEvilSpell`: Dispiro Malas.
pub static DISPEL_EVIL_SPELL: SpellType = SpellType {
    full_name: "Server.Spells.Chivalry.DispelEvilSpell",
    parent: Some(&PALADIN_SPELL),
    info: SpellInfo::new("Dispel Evil", "Dispiro Malas", &[])
        .action(-1)
        .hand_effect(9002),
    cast_delay_base: |_, _| TimeSpan::from_millis(250),
    delayed_damage: false,
    on_cast,
    blocks_movement: |_, _| false,
    family: Some(&DISPEL_EVIL),
    ..PALADIN_SPELL
};

const _: PaladinType = PALADIN_BASE;

fn on_cast(w: &mut World, sp: SpellId) {
    let Some(caster) = spell::caster(w, sp) else {
        return;
    };
    if spell::check_sequence(w, sp) {
        effects::play_sound(w, caster, 0xF5);
        effects::play_sound(w, caster, 0x299);
        effects::fixed_particles_hue(w, caster, 0x37C4, 1, 25, 9922, 14, 3, EffectLayer::Head);

        let dispel_skill = power(w, sp, 2);
        let chiv = skills::value(w, caster, SkillName::Chivalry);

        let (cloc, cmap) = w
            .mobile(caster)
            .map(|c| (c.location, c.map))
            .unwrap_or_default();
        let mut queue = Vec::new();
        for m in mobiles_in_range(w, cmap, cloc, 8) {
            if caster != m
                && helper::valid_indirect_target(w, caster, m)
                && mb::can_be_harmful_msg(w, caster, m, false)
            {
                queue.push(m);
            }
        }

        for m in queue {
            if let Some(bc) = ctype(w, m) {
                let summoned = creature::cd(w, m).is_some_and(|d| d.summoned);
                if summoned && !creature::misc::is_animated_dead(w, m) {
                    let mut dispel_chance = (50.0
                        + 100.0 * (chiv - (bc.dispel_difficulty)(w, m))
                            / ((bc.dispel_focus)(w, m) * 2.0))
                        / 100.0;
                    dispel_chance *= f64::from(dispel_skill) / 100.0;

                    if dispel_chance > utility::random_double(&mut w.rng) {
                        let (loc, map) =
                            w.mobile(m).map(|x| (x.location, x.map)).unwrap_or_default();
                        let e = effect_item::create_targeted(
                            w,
                            loc,
                            map,
                            effect_item::DEFAULT_DURATION,
                        );
                        muo_server::effects::send_location_particles(w, e, 0x3728, 8, 20, 5042);
                        muo_server::effects::play_sound_entity(w, m, 0x201);

                        if let Some(ty) = w.mobile(m).map(|x| x.ty) {
                            (ty.delete)(w, m);
                        }
                        continue;
                    }
                }

                let controlled = creature::cd(w, m).is_some_and(|d| d.controlled);
                let karma = w.mobile(m).map_or(0, |x| x.karma);
                let evil = !controlled && karma < 0;

                if evil {
                    // TODO: Is this right?
                    let fame = w.mobile(m).map_or(0, |x| x.fame);
                    let mut flee_chance =
                        (100.0 - (f64::from(fame) / 2.0).sqrt()) * chiv * f64::from(dispel_skill);
                    flee_chance /= 1_000_000.0;

                    if flee_chance > utility::random_double(&mut w.rng) {
                        creature::misc::begin_flee(w, m, TimeSpan::from_secs(30));
                    }
                }
            }

            let context = helper::get_context(w, m);
            // Trees are not evil! TODO: OSI confirm?
            if context.is_some_and(|c| c.ty.starts_with("Server.Spells.Necromancy.")) {
                // transformed ..
                let necro = skills::value(w, m, SkillName::Necromancy).max(1.0);
                let drain_chance = 0.5 * (chiv / necro);

                if drain_chance > utility::random_double(&mut w.rng) {
                    let drain = 5 * dispel_skill / 100;

                    let stam = w.mobile(m).map_or(0, |x| x.stam);
                    let mana = w.mobile(m).map_or(0, |x| x.mana);
                    props_b::set_stam(w, m, stam - drain);
                    props_b::set_mana(w, m, mana - drain);
                }
            }
        }
    }

    spell::finish_sequence(w, sp);
}
