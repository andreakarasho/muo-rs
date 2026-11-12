//! The Cunning spell.

use muo_content::player_mobile::buff::BuffIcon;
use muo_content::spells::magery::{MAGERY_SPELL, SECOND_CIRCLE};
use muo_content::spells::spell::{SpellId, SpellType};
use muo_content::spells::targeting::{SpellTargetKind, begin_spell_target};
use muo_content::spells::{SpellInfo, reagent};
use muo_server::World;
use muo_server::effects::EffectLayer;
use muo_server::mobile::StatType;
use muo_server::target::{TargetFlags, Targeted};

use crate::common::{check_cast_sudden_death, stat_bonus_spell};

/// `CunningSpell`: Uus Wis.
pub static CUNNING_SPELL: SpellType = SpellType {
    full_name: "Server.Spells.Second.CunningSpell",
    parent: Some(&MAGERY_SPELL),
    target_range: Some(muo_content::spells::targeting::target_range),
    info: SpellInfo::new(
        "Cunning",
        "Uus Wis",
        &[reagent::MANDRAKE_ROOT, reagent::NIGHTSHADE],
    )
    .action(212)
    .hand_effect(9061),
    on_cast,
    check_cast: check_cast_sudden_death,
    family: Some(&SECOND_CIRCLE),
    ..MAGERY_SPELL
};

fn target(w: &mut World, sp: SpellId, targeted: Targeted) {
    stat_bonus_spell(
        w,
        sp,
        targeted,
        StatType::INT,
        (0x375A, 10, 15, 5011, EffectLayer::Head),
        0x1EB,
        (BuffIcon::Cunning, 1075843),
    );
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
