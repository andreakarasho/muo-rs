//! The Curse spell.

use indexmap::IndexMap;
use muo_content::core;
use muo_content::player_mobile::buff::{BuffIcon, BuffInfo, add_buff};
use muo_content::spells::helper;
use muo_content::spells::magery::{FOURTH_CIRCLE, MAGERY_SPELL};
use muo_content::spells::spell::{self, SpellId, SpellType};
use muo_content::spells::targeting::{SpellTargetKind, begin_spell_target};
use muo_content::spells::{SpellCircle, SpellInfo, reagent};
use muo_io::text::TextDefinition;
use muo_primitives::TimeSpan;
use muo_server::World;
use muo_server::effects::EffectLayer;
use muo_server::ids::MobileId;
use muo_server::mobile::part2::effects;
use muo_server::mobile::{StatType, base as mb};
use muo_server::target::{TargetFlags, Targeted};
use muo_server::timer::{self, TimerHandle};

use crate::common::{is_player_mobile, mobile_of, turn_to};

/// `CurseSpell`: Des Sanct.
pub static CURSE_SPELL: SpellType = SpellType {
    full_name: "Server.Spells.Fourth.CurseSpell",
    parent: Some(&MAGERY_SPELL),
    target_range: Some(muo_content::spells::targeting::target_range),
    info: SpellInfo::new(
        "Curse",
        "Des Sanct",
        &[reagent::NIGHTSHADE, reagent::GARLIC, reagent::SULFUROUS_ASH],
    )
    .action(227)
    .hand_effect(9031),
    on_cast,
    family: Some(&FOURTH_CIRCLE),
    ..MAGERY_SPELL
};

/// `_table`.
#[derive(Default)]
struct Table(IndexMap<MobileId, TimerHandle>);

fn table(w: &mut World) -> &mut IndexMap<MobileId, TimerHandle> {
    &mut w.resources.get_mut::<Table>().0
}

/// `DoCurse(caster, m)`.
pub fn do_curse(w: &mut World, caster: MobileId, m: MobileId) -> bool {
    let duration = helper::get_duration(w, caster, m);
    if duration == TimeSpan::ZERO {
        return false;
    }

    // Allow curse duration to refresh if the new curse is longer
    let existing = table(w).get(&m).copied();
    if let Some(t) = existing
        && t.next(w) - w.now > duration
        && core::aos(w)
    {
        return false;
    }

    let new_str = helper::get_offset(w, caster, m, StatType::STR, true);
    let new_dex = helper::get_offset(w, caster, m, StatType::DEX, true);
    let new_int = helper::get_offset(w, caster, m, StatType::INT, true);

    if new_str == 0 && new_dex == 0 && new_int == 0 {
        return false;
    }

    let old_str = helper::get_curse(w, caster, m, StatType::STR);
    let old_dex = helper::get_curse(w, caster, m, StatType::DEX);
    let old_int = helper::get_curse(w, caster, m, StatType::INT);

    if old_str > new_str && old_dex > new_dex && old_int > new_int {
        return false;
    }

    let str_curse =
        helper::add_stat_curse_value(w, caster, m, StatType::STR, new_str, duration, false);
    let dex_curse =
        helper::add_stat_curse_value(w, caster, m, StatType::DEX, new_dex, duration, false);
    let int_curse =
        helper::add_stat_curse_value(w, caster, m, StatType::INT, new_int, duration, true);

    if !str_curse && !dex_curse && !int_curse {
        return false;
    }

    if let Some(mut t) = existing {
        t.cancel(w);
    }

    mb::update_resistances(w, m);

    let h = timer::start(w, duration, TimeSpan::ZERO, 1, move |w| {
        remove_effect(w, m);
    });
    table(w).insert(m, h);

    spell::caster_hurt(w, m);
    mb::set_paralyzed(w, m, false);

    effects::fixed_particles(w, m, 0x374A, 10, 15, 5028, EffectLayer::Waist);
    effects::play_sound(w, m, 0x1E1);

    let percentage = (helper::get_offset_scalar(w, caster, m, true) * 100.0) as i32;
    let args = format!(
        "{percentage}\t{percentage}\t{percentage}\t{}\t{}\t{}\t{}",
        10, 10, 10, 10
    );
    if is_player_mobile(w, m) {
        add_buff(
            w,
            m,
            BuffInfo::with_secondary(BuffIcon::Curse, 1075835, 1075836)
                .duration(duration)
                .args(TextDefinition::of_text(args)),
        );
    }

    true
}

fn target(w: &mut World, sp: SpellId, targeted: Targeted) {
    let Some(mut m) = mobile_of(targeted) else {
        return;
    };
    if spell::check_h_sequence(w, sp, m) {
        let Some(mut caster) = spell::caster(w, sp) else {
            return;
        };
        turn_to(w, caster, m);
        helper::check_reflect(w, SpellCircle::Fourth as i32, &mut caster, &mut m);
        let caster = spell::caster(w, sp).unwrap_or(caster);

        if do_curse(w, caster, m) {
            spell::harmful_spell(w, sp, m);
        } else {
            let ty = spell::spell_type(w, sp).expect("the spell is casting");
            (ty.do_hurt_fizzle)(w, sp);
        }
    }
}

fn on_cast(w: &mut World, sp: SpellId) {
    begin_spell_target(
        w,
        sp,
        false,
        TargetFlags::HARMFUL,
        false,
        SpellTargetKind::Mobile,
        target,
    );
}

/// `RemoveEffect(m)`.
pub fn remove_effect(w: &mut World, m: MobileId) -> bool {
    if let Some(mut t) = table(w).shift_remove(&m) {
        t.cancel(w);
        mb::update_resistances(w, m);
        return true;
    }
    false
}

/// `UnderEffect(m)` without needing the world mutably.
pub fn under_effect_ref(w: &World, m: MobileId) -> bool {
    w.resources
        .get::<Table>()
        .is_some_and(|t| t.0.contains_key(&m))
}
