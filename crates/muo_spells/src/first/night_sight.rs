//! The Night Sight spell.

use muo_content::core;
use muo_content::misc::light_cycle;
use muo_content::player_mobile::buff::{BuffIcon, BuffInfo, add_buff};
use muo_content::spells::helper;
use muo_content::spells::magery::{FIRST_CIRCLE, MAGERY_SPELL};
use muo_content::spells::spell::{self, SpellId, SpellType};
use muo_content::spells::targeting::{SpellTargetKind, begin_spell_target};
use muo_content::spells::{SpellInfo, reagent};
use muo_server::World;
use muo_server::effects::EffectLayer;
use muo_server::ids::EntityId;
use muo_server::mobile::part1::{combat, props_a};
use muo_server::mobile::part2::{effects, messages as mm};
use muo_server::skills::{self, SkillName};
use muo_server::target::{TargetFlags, Targeted};

/// `NightSightSpell`: In Lor.
pub static NIGHT_SIGHT_SPELL: SpellType = SpellType {
    full_name: "Server.Spells.First.NightSightSpell",
    parent: Some(&MAGERY_SPELL),
    target_range: Some(|_| 12),
    info: SpellInfo::new(
        "Night Sight",
        "In Lor",
        &[reagent::SULFUROUS_ASH, reagent::SPIDERS_SILK],
    )
    .action(236)
    .hand_effect(9031),
    on_cast,
    family: Some(&FIRST_CIRCLE),
    ..MAGERY_SPELL
};

fn target(w: &mut World, sp: SpellId, targeted: Targeted) {
    let Targeted::Entity(EntityId::Mobile(m)) = targeted else {
        return;
    };
    let Some(caster) = spell::caster(w, sp) else {
        return;
    };
    if spell::check_b_sequence(w, sp, m, false) {
        helper::turn(
            w,
            caster,
            Some(helper::Point3dRef::Entity(EntityId::Mobile(m))),
        );

        if combat::begin_action(w, m, "Server.LightCycle") {
            light_cycle::start_night_sight_timer(w, m);

            let who = if core::aos(w) { m } else { caster };
            let skill = skills::value(w, who, SkillName::Magery) / 100.0;
            let level = (f64::from(light_cycle::DUNGEON_LEVEL) * skill) as i32;
            props_a::set_light_level(w, m, level.max(0));

            effects::fixed_particles(w, m, 0x376A, 9, 32, 5007, EffectLayer::Waist);
            effects::play_sound(w, m, 0x1E3);

            // Night Sight/You ignore lighting effects
            if w.mobile(m)
                .is_some_and(|x| x.ty.is(&muo_content::player_mobile::PLAYER_MOBILE))
            {
                add_buff(w, m, BuffInfo::new(BuffIcon::NightSight, 1075643));
            }
        } else if m == caster {
            mm::send_message(w, m, "You already have nightsight.");
        } else {
            mm::send_message(w, m, "They already have nightsight.");
        }
    }
}

fn on_cast(w: &mut World, sp: SpellId) {
    begin_spell_target(
        w,
        sp,
        false,
        TargetFlags::BENEFICIAL,
        false,
        SpellTargetKind::Mobile,
        target,
    );
}
