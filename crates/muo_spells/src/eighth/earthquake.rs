//! The Earthquake spell.

use muo_content::core;
use muo_content::spells::helper::{self, SpellDamageSplit};
use muo_content::spells::magery::{EIGHTH_CIRCLE, MAGERY_SPELL};
use muo_content::spells::spell::{self, SpellId, SpellType};
use muo_content::spells::{SpellInfo, reagent};
use muo_primitives::{TimeSpan, utility};
use muo_server::World;
use muo_server::item::deps;
use muo_server::mobile::base as mb;
use muo_server::mobile::part2::effects;
use muo_server::skills::{self, SkillName};

use crate::common::mobiles_in_range;

/// `EarthquakeSpell`: In Vas Por.
pub static EARTHQUAKE_SPELL: SpellType = SpellType {
    full_name: "Server.Spells.Eighth.EarthquakeSpell",
    parent: Some(&MAGERY_SPELL),
    info: SpellInfo::new(
        "Earthquake",
        "In Vas Por",
        &[
            reagent::BLOODMOSS,
            reagent::GINSENG,
            reagent::MANDRAKE_ROOT,
            reagent::SULFUROUS_ASH,
        ],
    )
    .action(233)
    .hand_effect(9012)
    .allow_town(false),
    delayed_damage_unless_aos: true,
    on_cast,
    family: Some(&EIGHTH_CIRCLE),
    ..MAGERY_SPELL
};

fn on_cast(w: &mut World, sp: SpellId) {
    let Some(caster) = spell::caster(w, sp) else {
        return;
    };
    let loc = w.mobile(caster).map(|c| c.location).unwrap_or_default();
    if helper::check_town_at(w, loc, caster) && spell::check_sequence(w, sp) {
        effects::play_sound(w, caster, 0x220);

        let map = w.mobile(caster).and_then(|c| c.map);
        if map.is_none() {
            spell::finish_sequence(w, sp);
            return;
        }

        let range = 1 + (skills::value(w, caster, SkillName::Magery) / 15.0) as i32;
        let mut queue = Vec::new();
        for m in mobiles_in_range(w, map, loc, range) {
            if caster != m
                && helper::valid_indirect_target(w, caster, m)
                && mb::can_be_harmful_msg(w, caster, m, false)
                && (!core::aos(w) || deps::in_los_mobile(w, caster, m))
            {
                queue.push(m);
            }
        }

        for m in queue {
            let hits = w.mobile(m).map_or(0, |x| x.hits);
            let player = w.mobile(m).is_some_and(|x| x.player);
            let mut damage: i32;

            if core::aos(w) {
                damage = hits / 2;

                if !player {
                    damage = damage.clamp(15, 100);
                }

                damage += utility::random_min_max(&mut w.rng, 0, 15);
            } else {
                damage = hits * 6 / 10;

                if !player && damage < 10 {
                    damage = 10;
                } else if damage > 75 {
                    damage = 75;
                }
            }

            mb::do_harmful(w, caster, m, false);
            helper::damage_aos(
                w,
                None,
                TimeSpan::ZERO,
                m,
                Some(caster),
                f64::from(damage),
                SpellDamageSplit {
                    phys: 100,
                    ..Default::default()
                },
            );
        }
    }

    spell::finish_sequence(w, sp);
}
