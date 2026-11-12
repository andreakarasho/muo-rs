//! The Chain Lightning spell.

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

/// `ChainLightningSpell`: Vas Ort Grav.
pub static CHAIN_LIGHTNING_SPELL: SpellType = SpellType {
    full_name: "Server.Spells.Seventh.ChainLightningSpell",
    parent: Some(&MAGERY_SPELL),
    target_range: Some(muo_content::spells::targeting::target_range),
    info: SpellInfo::new(
        "Chain Lightning",
        "Vas Ort Grav",
        &[
            reagent::BLACK_PEARL,
            reagent::BLOODMOSS,
            reagent::MANDRAKE_ROOT,
            reagent::SULFUROUS_ASH,
        ],
    )
    .action(209)
    .hand_effect(9022)
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
    // `(p as Item)?.GetWorldLocation() ?? new Point3D(p)`
    let loc = match targeted {
        Targeted::Entity(EntityId::Item(i)) => ops::get_world_location(w, i),
        other => helper::point_of(w, Point3dRef::from(other)),
    };

    if helper::check_town_at(w, loc, caster) && spell::check_sequence(w, sp) {
        helper::turn(w, caster, Some(Point3dRef::from(targeted)));

        let map = w.mobile(caster).and_then(|c| c.map);
        if map.is_some() {
            let mut pvp = false;
            let mut pool = Vec::new();

            for m in mobiles_in_range(w, map, loc, 2) {
                if (core::aos(w) && (m == caster || !deps::in_los_mobile(w, caster, m)))
                    || !helper::valid_indirect_target(w, caster, m)
                    || !mb::can_be_harmful_msg(w, caster, m, false)
                {
                    continue;
                }

                if w.mobile(m).is_some_and(|x| x.player) {
                    pvp = true;
                }

                pool.push(m);
            }

            if !pool.is_empty() {
                let mut damage = if core::aos(w) {
                    f64::from(spell::get_new_aos_damage(w, sp, 51, 1, 5, pvp, None))
                } else {
                    f64::from(utility::random_from(&mut w.rng, 27, 22))
                };

                if pool.len() > 2 {
                    if core::aos(w) {
                        damage *= 2.0;
                    }

                    damage /= pool.len() as f64;
                }

                for m in pool {
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
                            nrgy: 100,
                            ..Default::default()
                        },
                    );

                    effects::bolt_effect(w, m, 0);
                }
            } else {
                effects::play_sound(w, caster, 0x29);
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
