//! The Arch Cure spell.

use muo_content::core;
use muo_content::spells::helper::{self, Point3dRef};
use muo_content::spells::magery::{FOURTH_CIRCLE, MAGERY_SPELL};
use muo_content::spells::spell::{self, SpellId, SpellType};
use muo_content::spells::targeting::{SpellTargetKind, begin_spell_target};
use muo_content::spells::{SpellInfo, reagent};
use muo_primitives::{TimeSpan, utility};
use muo_server::World;
use muo_server::effects::EffectLayer;
use muo_server::ids::MobileId;
use muo_server::map::MapRules;
use muo_server::mobile::base as mb;
use muo_server::mobile::part2::{effects, messages as mm};
use muo_server::notoriety;
use muo_server::poison::Poisons;
use muo_server::skills::{self, SkillName};
use muo_server::target::{TargetFlags, Targeted};

use crate::common::{mobiles_in_range, surface_top};

/// `ArchCureSpell`: Vas An Nox.
pub static ARCH_CURE_SPELL: SpellType = SpellType {
    full_name: "Server.Spells.Fourth.ArchCureSpell",
    parent: Some(&MAGERY_SPELL),
    target_range: Some(muo_content::spells::targeting::target_range),
    info: SpellInfo::new(
        "Arch Cure",
        "Vas An Nox",
        &[reagent::GARLIC, reagent::GINSENG, reagent::MANDRAKE_ROOT],
    )
    .action(215)
    .hand_effect(9061),
    on_cast,
    // Arch cure is now 1/4th of a second faster
    cast_delay_base: |w, id| (MAGERY_SPELL.cast_delay_base)(w, id) - TimeSpan::from_secs_f64(0.25),
    family: Some(&FOURTH_CIRCLE),
    ..MAGERY_SPELL
};

fn target(w: &mut World, sp: SpellId, targeted: Targeted) {
    let Some(caster) = spell::caster(w, sp) else {
        return;
    };
    if spell::check_sequence(w, sp) {
        let ip = Point3dRef::from(targeted);
        helper::turn(w, caster, Some(ip));
        let loc = surface_top(w, targeted);

        let map = w.mobile(caster).and_then(|c| c.map);
        if let Some(map_id) = map {
            let direct_target = match targeted {
                Targeted::Entity(muo_server::ids::EntityId::Mobile(m)) => Some(m),
                _ => None,
            };
            let felucca_rules = w
                .maps
                .get(map_id)
                .is_some_and(|m| m.rules == MapRules::empty());

            let mut pool: Vec<MobileId> = Vec::new();

            // You can target any living mobile directly, beneficial checks apply
            if let Some(d) = direct_target
                && mb::can_be_beneficial_msg(w, caster, d, false)
            {
                pool.push(d);
            }

            for m in mobiles_in_range(w, map, loc, 2) {
                if Some(m) != direct_target && area_can_target(w, caster, m, felucca_rules) {
                    pool.push(m);
                }
            }

            muo_server::effects::play_sound(w, loc, map, 0x299);

            let mut cured = 0;
            for m in pool {
                mb::do_beneficial(w, caster, m);

                let poison = w.mobile(m).and_then(|x| x.poison);
                if let Some(p) = poison {
                    let level = w
                        .resources
                        .get::<Poisons>()
                        .and_then(|ps| {
                            let inc = ps.increase_level(Some(p))?;
                            ps.get_poison_by_index(i32::from(inc.0)).map(|x| x.level)
                        })
                        .unwrap_or(0);
                    let mut chance_to_cure = 10000
                        + (skills::value(w, caster, SkillName::Magery) * 75.0) as i32
                        - level * 1750;
                    chance_to_cure /= 100;
                    chance_to_cure -= 1;

                    if chance_to_cure > utility::random(&mut w.rng, 100)
                        && mb::cure_poison(w, m, Some(caster))
                    {
                        cured += 1;
                    }
                }

                effects::fixed_particles(w, m, 0x373A, 10, 15, 5012, EffectLayer::Waist);
                effects::play_sound(w, m, 0x1E0);
            }

            if cured > 0 {
                mm::send_localized_message(w, caster, 1010058, "", 0x3B2); // You have cured the target of all poisons!
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

fn area_can_target(w: &mut World, caster: MobileId, target: MobileId, felucca_rules: bool) -> bool {
    /* Arch cure area effect won't cure aggressors, victims, murderers, criminals or monsters.
     * In Felucca, it will also not cure summons and pets.
     * For red players it will only cure themselves and guild members.
     */
    if !mb::can_be_beneficial_msg(w, caster, target, false) {
        return false;
    }

    if core::aos(w) && target != caster {
        if is_aggressor(w, caster, target) || is_aggressed(w, caster, target) {
            return false;
        }

        if (!is_innocent_to(w, caster, target) || !is_innocent_to(w, target, caster))
            && !is_ally_to(w, caster, target)
        {
            return false;
        }

        let is_pm = w
            .mobile(target)
            .is_some_and(|x| x.ty.is(&muo_content::player_mobile::PLAYER_MOBILE));
        if felucca_rules && !is_pm {
            return false;
        }
    }

    true
}

fn is_aggressor(w: &World, caster: MobileId, m: MobileId) -> bool {
    w.mobile(caster).is_some_and(|c| {
        c.aggressors
            .iter()
            .any(|info| m == info.attacker && !info.expired(w))
    })
}

fn is_aggressed(w: &World, caster: MobileId, m: MobileId) -> bool {
    w.mobile(caster).is_some_and(|c| {
        c.aggressed
            .iter()
            .any(|info| m == info.defender && !info.expired(w))
    })
}

fn is_innocent_to(w: &mut World, from: MobileId, to: MobileId) -> bool {
    notoriety::compute(w, from, to) == notoriety::INNOCENT
}

fn is_ally_to(w: &mut World, from: MobileId, to: MobileId) -> bool {
    notoriety::compute(w, from, to) == notoriety::ALLY
}
