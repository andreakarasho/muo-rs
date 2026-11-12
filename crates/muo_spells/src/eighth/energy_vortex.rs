//! The Energy Vortex spell.

use muo_content::core;
use muo_content::creature::control;
use muo_content::factory;
use muo_content::spells::helper;
use muo_content::spells::magery::{EIGHTH_CIRCLE, MAGERY_SPELL};
use muo_content::spells::spell::{self, SpellId, SpellType};
use muo_content::spells::targeting::{SpellTargetKind, begin_spell_target};
use muo_content::spells::{SpellInfo, reagent};
use muo_primitives::{TimeSpan, utility};
use muo_server::World;
use muo_server::mobile::part2::messages as mm;
use muo_server::target::{TargetFlags, Targeted};

use crate::common::surface_top;

/// `EnergyVortexSpell`: Vas Corp Por.
pub static ENERGY_VORTEX_SPELL: SpellType = SpellType {
    full_name: "Server.Spells.Eighth.EnergyVortexSpell",
    parent: Some(&MAGERY_SPELL),
    target_range: Some(muo_content::spells::targeting::target_range),
    info: SpellInfo::new(
        "Energy Vortex",
        "Vas Corp Por",
        &[
            reagent::BLOODMOSS,
            reagent::BLACK_PEARL,
            reagent::MANDRAKE_ROOT,
            reagent::NIGHTSHADE,
        ],
    )
    .action(260)
    .hand_effect(9032)
    .allow_town(false),
    on_cast,
    check_cast,
    family: Some(&EIGHTH_CIRCLE),
    ..MAGERY_SPELL
};

fn target(w: &mut World, sp: SpellId, targeted: Targeted) {
    let Some(caster) = spell::caster(w, sp) else {
        return;
    };
    let map = w.mobile(caster).and_then(|c| c.map);
    let p = surface_top(w, targeted);

    if !map.is_some_and(|m| muo_server::map::fit::can_spawn_mobile_at(w, m, p.x, p.y, p.z)) {
        mm::send_localized_message(w, caster, 501942, "", 0x3B2); // That location is blocked.
    } else if helper::check_town(w, helper::Point3dRef::Point(p), caster)
        && spell::check_sequence(w, sp)
    {
        // TODO: Check Demo for pre-T2A.
        let duration = if core::t2a(w) {
            TimeSpan::from_secs(90)
        } else {
            TimeSpan::from_secs(i64::from(utility::random_from(&mut w.rng, 80, 40)))
        };

        if let Some(c) = factory::make_mobile(w, "Server.Mobiles.EnergyVortex") {
            control::summon(w, c, false, caster, p, 0x212, duration, None);
        }
    }
}

fn check_cast(w: &mut World, sp: SpellId) -> bool {
    if !spell::base_check_cast(w, sp) {
        return false;
    }
    let Some(caster) = spell::caster(w, sp) else {
        return false;
    };
    let (followers, max) = w
        .mobile(caster)
        .map_or((0, 0), |c| (c.followers, c.followers_max));
    if followers + if core::se(w) { 2 } else { 1 } > max {
        mm::send_localized_message(w, caster, 1049645, "", 0x3B2); // You have too many followers to summon that creature.
        return false;
    }
    true
}

fn on_cast(w: &mut World, sp: SpellId) {
    begin_spell_target(
        w,
        sp,
        true,
        TargetFlags::NONE,
        true,
        SpellTargetKind::Point,
        target,
    );
}
