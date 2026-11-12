//! The Blade Spirits spell.

use muo_content::core;
use muo_content::creature::control;
use muo_content::factory;
use muo_content::spells::helper;
use muo_content::spells::magery::{FIFTH_CIRCLE, MAGERY_SPELL};
use muo_content::spells::spell::{self, SpellId, SpellType};
use muo_content::spells::targeting::{SpellTargetKind, begin_spell_target};
use muo_content::spells::{SpellInfo, reagent};
use muo_data::expansion::Expansion;
use muo_primitives::{TimeSpan, utility};
use muo_server::World;
use muo_server::mobile::part2::messages as mm;
use muo_server::target::{TargetFlags, Targeted};

use crate::common::surface_top;

/// `BladeSpiritsSpell`: In Jux Hur Ylem.
pub static BLADE_SPIRITS_SPELL: SpellType = SpellType {
    full_name: "Server.Spells.Fifth.BladeSpiritsSpell",
    parent: Some(&MAGERY_SPELL),
    target_range: Some(muo_content::spells::targeting::target_range),
    info: SpellInfo::new(
        "Blade Spirits",
        "In Jux Hur Ylem",
        &[
            reagent::BLACK_PEARL,
            reagent::MANDRAKE_ROOT,
            reagent::NIGHTSHADE,
        ],
    )
    .action(266)
    .hand_effect(9040)
    .allow_town(false),
    on_cast,
    get_cast_delay,
    check_cast,
    family: Some(&FIFTH_CIRCLE),
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
        let secs = if core::aos(w) {
            120
        } else {
            utility::random_from(&mut w.rng, 80, 40)
        };
        let duration = TimeSpan::from_secs(i64::from(secs));
        if let Some(c) = factory::make_mobile(w, "Server.Mobiles.BladeSpirits") {
            control::summon(w, c, false, caster, p, 0x212, duration, None);
        }
    }
}

fn get_cast_delay(w: &mut World, sp: SpellId) -> TimeSpan {
    let exp = core::expansion(w);
    let scalar: i64 = if exp >= Expansion::SE {
        3
    } else if exp >= Expansion::AOS {
        5
    } else {
        4
    };

    let base = (MAGERY_SPELL.get_cast_delay)(w, sp);
    let mut delay = TimeSpan::from_ticks(base.ticks() * scalar);

    // SA made everything 0.25s slower, but that is applied after the scalar
    // So remove 0.25 * scalar to compensate
    if core::sa(w) {
        delay -= TimeSpan::from_secs_f64(0.25 * scalar as f64);
    }

    delay
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
