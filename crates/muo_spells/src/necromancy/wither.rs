//! The Wither spell.

use muo_content::core;
use muo_content::misc::aos_helpers;
use muo_content::spells::helper::{self, SpellDamageSplit};
use muo_content::spells::spell::{self, SpellId, SpellType};
use muo_content::spells::{SpellInfo, reagent};
use muo_data::expansion::Expansion;
use muo_primitives::{TimeSpan, utility};
use muo_server::World;
use muo_server::effects::EffectLayer;
use muo_server::item::{deps, effect_item};
use muo_server::mobile::base as mb;
use muo_server::mobile::part2::effects;

use super::necromancer::{NECRO_BASE, NECROMANCER_SPELL, NecroType};
use super::poison_strike::{is_monster, skip_creature};
use crate::common::mobiles_in_range;

static WITHER: NecroType = NecroType {
    required_skill: 60.0,
    required_mana: 23,
    ..NECRO_BASE
};

/// `WitherSpell`: Kal Vas An Flam.
pub static WITHER_SPELL: SpellType = SpellType {
    full_name: "Server.Spells.Necromancy.WitherSpell",
    parent: Some(&NECROMANCER_SPELL),
    info: SpellInfo::new(
        "Wither",
        "Kal Vas An Flam",
        &[reagent::NOX_CRYSTAL, reagent::GRAVE_DUST, reagent::PIG_IRON],
    )
    .action(203)
    .hand_effect(9031),
    cast_delay_base: |w, _| {
        let e = core::expansion(w);
        TimeSpan::from_millis(if e >= Expansion::SA {
            1250
        } else if e >= Expansion::ML {
            1500
        } else {
            1000
        })
    },
    delayed_damage: false,
    on_cast,
    family: Some(&WITHER),
    ..NECROMANCER_SPELL
};

fn on_cast(w: &mut World, sp: SpellId) {
    let Some(caster) = spell::caster(w, sp) else {
        return;
    };
    if spell::check_sequence(w, sp) {
        /* Creates a withering frost around the Caster, which deals Cold Damage to all valid targets in a radius
         * of 5 tiles.
         */
        let (loc, map) = w
            .mobile(caster)
            .map(|c| (c.location, c.map))
            .unwrap_or_default();
        if map.is_some() {
            let mut pool = Vec::new();
            let monster = is_monster(w, caster);

            for targ in mobiles_in_range(w, map, loc, if core::ml(w) { 4 } else { 5 }) {
                if targ == caster
                    || !deps::in_los_mobile(w, caster, targ)
                    || (!monster && !helper::valid_indirect_target(w, caster, targ))
                    || !mb::can_be_harmful_msg(w, caster, targ, false)
                {
                    continue;
                }

                if monster && w.mobile(targ).is_some_and(|x| x.player) {
                    continue;
                }

                // Animate dead casting poison strike shouldn't hit: familiars or player or pets
                if skip_creature(w, caster, targ, monster) {
                    continue;
                }

                pool.push(targ);
            }

            muo_server::effects::play_sound(w, loc, map, 0x1FB);
            muo_server::effects::play_sound(w, loc, map, 0x10B);

            let e = effect_item::create_targeted(w, loc, map, effect_item::DEFAULT_DURATION);
            muo_server::effects::send_location_particles_full(w, e, 0x37CC, 1, 40, 97, 3, 9917, 0);

            for m in pool {
                mb::do_harmful(w, caster, m, false);

                effects::fixed_particles_hue(w, m, 0x374A, 1, 15, 9502, 97, 3, EffectLayer(255));

                let mut damage = f64::from(utility::random_min_max(&mut w.rng, 30, 35));

                let ty = spell::spell_type(w, sp).expect("the spell is casting");
                let karma = w.mobile(m).map_or(0, |x| x.karma);
                damage *=
                    300.0 + f64::from(karma) / 100.0 + (ty.get_damage_skill)(w, sp, caster) * 10.0;
                damage /= 1000.0;

                let mut sdi_bonus = aos_helpers::attribute_value(
                    w,
                    caster,
                    muo_content::aos::AosAttribute::SpellDamage,
                );

                // PvP spell damage increase cap of 15% from an item's magic property in Publish 33(SE)
                if core::se(w)
                    && w.mobile(m).is_some_and(|x| x.player)
                    && w.mobile(caster).is_some_and(|x| x.player)
                    && sdi_bonus > 15
                {
                    sdi_bonus = 15;
                }

                damage *= f64::from(100 + sdi_bonus);
                damage /= 100.0;

                // TODO: cap?
                helper::damage_aos_spell(
                    w,
                    sp,
                    m,
                    damage,
                    SpellDamageSplit {
                        cold: 100,
                        ..Default::default()
                    },
                );
            }
        }
    }

    spell::finish_sequence(w, sp);
}
