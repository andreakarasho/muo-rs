//! The Bless spell.

use muo_content::player_mobile::buff::{BuffIcon, BuffInfo, add_buff};
use muo_content::spells::helper;
use muo_content::spells::magery::{MAGERY_SPELL, THIRD_CIRCLE};
use muo_content::spells::spell::{self, SpellId, SpellType};
use muo_content::spells::targeting::{SpellTargetKind, begin_spell_target};
use muo_content::spells::{SpellInfo, reagent};
use muo_io::text::TextDefinition;
use muo_server::World;
use muo_server::effects::EffectLayer;
use muo_server::mobile::StatType;
use muo_server::mobile::part2::effects;
use muo_server::target::{TargetFlags, Targeted};

use crate::common::{check_cast_sudden_death, is_player_mobile, mobile_of, turn_to};

/// `BlessSpell`: Rel Sanct.
pub static BLESS_SPELL: SpellType = SpellType {
    full_name: "Server.Spells.Third.BlessSpell",
    parent: Some(&MAGERY_SPELL),
    target_range: Some(muo_content::spells::targeting::target_range),
    info: SpellInfo::new(
        "Bless",
        "Rel Sanct",
        &[reagent::GARLIC, reagent::MANDRAKE_ROOT],
    )
    .action(203)
    .hand_effect(9061),
    on_cast,
    check_cast: check_cast_sudden_death,
    family: Some(&THIRD_CIRCLE),
    ..MAGERY_SPELL
};

fn target(w: &mut World, sp: SpellId, targeted: Targeted) {
    let Some(m) = mobile_of(targeted) else {
        return;
    };
    let Some(caster) = spell::caster(w, sp) else {
        return;
    };
    if spell::check_b_sequence(w, sp, m, false) {
        turn_to(w, caster, m);

        let length = helper::get_duration(w, caster, m);
        helper::add_stat_bonus(w, caster, m, StatType::STR, length, false);
        helper::add_stat_bonus(w, caster, m, StatType::DEX, length, true);
        helper::add_stat_bonus(w, caster, m, StatType::INT, length, true);

        effects::fixed_particles(w, m, 0x373A, 10, 15, 5018, EffectLayer::Waist);
        effects::play_sound(w, m, 0x1EA);

        let percentage = (helper::get_offset_scalar(w, caster, m, false) * 100.0) as i32;
        let args = format!("{percentage}\t{percentage}\t{percentage}");
        if is_player_mobile(w, m) {
            add_buff(
                w,
                m,
                BuffInfo::with_secondary(BuffIcon::Bless, 1075847, 1075848)
                    .duration(length)
                    .args(TextDefinition::of_text(args)),
            );
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
