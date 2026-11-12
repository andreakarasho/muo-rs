//! The Reveal spell.

use muo_content::core;
use muo_content::creature::type_name;
use muo_content::spells::helper::{self, Point3dRef};
use muo_content::spells::magery::{MAGERY_SPELL, SIXTH_CIRCLE};
use muo_content::spells::spell::{self, SpellId, SpellType};
use muo_content::spells::targeting::{SpellTargetKind, begin_spell_target};
use muo_content::spells::{SpellInfo, reagent};
use muo_primitives::utility;
use muo_server::World;
use muo_server::effects::EffectLayer;
use muo_server::ids::MobileId;
use muo_server::mobile::AccessLevel;
use muo_server::mobile::part2::effects;
use muo_server::skills::{self, SkillName};
use muo_server::target::{TargetFlags, Targeted};

use super::invisibility;
use crate::common::{mobiles_in_range, revealing_action, surface_top};

/// `RevealSpell`: Wis Quas.
pub static REVEAL_SPELL: SpellType = SpellType {
    full_name: "Server.Spells.Sixth.RevealSpell",
    parent: Some(&MAGERY_SPELL),
    target_range: Some(muo_content::spells::targeting::target_range),
    info: SpellInfo::new(
        "Reveal",
        "Wis Quas",
        &[reagent::BLOODMOSS, reagent::SULFUROUS_ASH],
    )
    .action(206)
    .hand_effect(9002),
    on_cast,
    family: Some(&SIXTH_CIRCLE),
    ..MAGERY_SPELL
};

fn target(w: &mut World, sp: SpellId, targeted: Targeted) {
    let Some(caster) = spell::caster(w, sp) else {
        return;
    };
    if spell::check_sequence(w, sp) {
        helper::turn(w, caster, Some(Point3dRef::from(targeted)));
        let p = surface_top(w, targeted);
        let map = w.mobile(caster).and_then(|c| c.map);
        if map.is_some() {
            let range = 1 + (skills::value(w, caster, SkillName::Magery) / 20.0) as i32;

            for m in mobiles_in_range(w, map, p, range) {
                let loc = w.mobile(m).map(|x| x.location).unwrap_or_default();
                if type_name(w, m) == "ShadowKnight" && (loc.x != p.x || loc.y != p.y) {
                    continue;
                }

                let (hidden, m_level) = w
                    .mobile(m)
                    .map_or((false, AccessLevel::Player), |x| (x.hidden, x.access_level));
                let c_level = w
                    .mobile(caster)
                    .map_or(AccessLevel::Player, |x| x.access_level);
                if !hidden
                    || (m_level > AccessLevel::Player && c_level <= m_level)
                    || !check_difficulty(w, caster, m)
                {
                    continue;
                }

                revealing_action(w, m);

                effects::fixed_particles(w, m, 0x375A, 9, 20, 5049, EffectLayer::Head);
                effects::play_sound(w, m, 0x1FD);
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

// Reveal uses magery and detect hidden vs. hide and stealth
fn check_difficulty(w: &mut World, from: MobileId, m: MobileId) -> bool {
    // Reveal always reveals vs. invisibility spell
    if !core::aos(w) || invisibility::has_timer(w, m) {
        return true;
    }

    let magery = skills::fixed(w, from, SkillName::Magery);
    let detect_hidden = skills::fixed(w, from, SkillName::DetectHidden);

    let hiding = skills::fixed(w, m, SkillName::Hiding);
    let stealth = skills::fixed(w, m, SkillName::Stealth);

    let divisor = hiding + stealth;

    let chance = if divisor > 0 {
        50 * (magery + detect_hidden) / divisor
    } else {
        100
    };

    chance > utility::random(&mut w.rng, 100)
}
