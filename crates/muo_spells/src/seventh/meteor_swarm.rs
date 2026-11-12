//! The Meteor Swarm spell.

use muo_content::core;
use muo_content::spells::helper::{self, Point3dRef, SpellDamageSplit};
use muo_content::spells::magery::{MAGERY_SPELL, SEVENTH_CIRCLE, check_resisted};
use muo_content::spells::spell::{self, SpellId, SpellType};
use muo_content::spells::targeting::{SpellTargetKind, begin_spell_target};
use muo_content::spells::{SpellInfo, reagent};
use muo_primitives::utility;
use muo_server::World;
use muo_server::ids::EntityId;
use muo_server::item::{deps, ops};
use muo_server::mobile::base as mb;
use muo_server::mobile::part2::{effects, messages as mm};
use muo_server::target::{TargetFlags, Targeted};

use crate::common::mobiles_in_range;

/// `MeteorSwarmSpell`: Flam Kal Des Ylem.
pub static METEOR_SWARM_SPELL: SpellType = SpellType {
    full_name: "Server.Spells.Seventh.MeteorSwarmSpell",
    parent: Some(&MAGERY_SPELL),
    target_range: Some(muo_content::spells::targeting::target_range),
    info: SpellInfo::new(
        "Meteor Swarm",
        "Flam Kal Des Ylem",
        &[
            reagent::BLOODMOSS,
            reagent::MANDRAKE_ROOT,
            reagent::SULFUROUS_ASH,
            reagent::SPIDERS_SILK,
        ],
    )
    .action(233)
    .hand_effect(9042)
    .allow_town(false),
    delayed_damage: true,
    on_cast,
    family: Some(&SEVENTH_CIRCLE),
    ..MAGERY_SPELL
};

fn target(w: &mut World, sp: SpellId, targeted: Targeted) {
    let Some(caster) = spell::caster(w, sp) else {
        return;
    };
    if helper::check_town(w, Point3dRef::from(targeted), caster) && spell::check_sequence(w, sp) {
        helper::turn(w, caster, Some(Point3dRef::from(targeted)));

        let loc = match targeted {
            Targeted::Entity(EntityId::Item(i)) => ops::get_world_location(w, i),
            other => helper::point_of(w, Point3dRef::from(other)),
        };

        let map = w.mobile(caster).and_then(|c| c.map);
        let mut player_vs_player = false;

        if map.is_some() {
            let mut queue = Vec::new();

            for m in mobiles_in_range(w, map, loc, 2) {
                if caster == m
                    || !helper::valid_indirect_target(w, caster, m)
                    || !mb::can_be_harmful_msg(w, caster, m, false)
                    || (core::aos(w) && !deps::in_los_mobile(w, caster, m))
                {
                    continue;
                }

                if w.mobile(m).is_some_and(|x| x.player) {
                    player_vs_player = true;
                }

                queue.push(m);
            }

            let mut damage = if core::aos(w) {
                f64::from(spell::get_new_aos_damage(
                    w,
                    sp,
                    51,
                    1,
                    5,
                    player_vs_player,
                    None,
                ))
            } else {
                f64::from(utility::random_from(&mut w.rng, 27, 22))
            };

            let count = queue.len();

            if count > 0 {
                muo_server::effects::play_sound(w, loc, map, 0x160);

                if core::aos(w) && count > 2 {
                    damage = damage * 2.0 / count as f64;
                } else if !core::aos(w) {
                    damage /= count as f64;
                }

                for m in queue {
                    let mut to_deal = damage;

                    if !core::aos(w) && check_resisted(w, sp, m) {
                        to_deal *= 0.5;

                        mm::send_localized_message(w, m, 501783, "", 0x3B2); // You feel yourself resisting magical energy.
                    }

                    let ty = spell::spell_type(w, sp).expect("the spell is casting");
                    to_deal *= (ty.get_damage_scalar)(w, sp, m);

                    mb::do_harmful(w, caster, m, false);
                    helper::damage_aos_spell(
                        w,
                        sp,
                        m,
                        to_deal,
                        SpellDamageSplit {
                            fire: 100,
                            ..Default::default()
                        },
                    );

                    effects::moving_particles(
                        w,
                        caster,
                        EntityId::Mobile(m),
                        0x36D4,
                        7,
                        0,
                        false,
                        true,
                        9501,
                        1,
                        0,
                        0x100,
                    );
                }
            }
        }
    }
}

fn on_cast(w: &mut World, sp: SpellId) {
    begin_spell_target(
        w,
        sp,
        true,
        TargetFlags::NONE,
        false,
        SpellTargetKind::Point,
        target,
    );
}
