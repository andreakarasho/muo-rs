//! The Weaken spell.

use muo_content::player_mobile::buff::{BuffIcon, BuffInfo};
use muo_content::spells::helper;
use muo_content::spells::magery::{FIRST_CIRCLE, MAGERY_SPELL};
use muo_content::spells::spell::{self, SpellId, SpellType};
use muo_content::spells::targeting::{SpellTargetKind, begin_spell_target};
use muo_content::spells::{SpellCircle, SpellInfo, reagent};
use muo_io::text::TextDefinition;
use muo_server::World;
use muo_server::effects::EffectLayer;
use muo_server::ids::EntityId;
use muo_server::mobile::StatType;
use muo_server::mobile::part2::effects;
use muo_server::target::{TargetFlags, Targeted};

/// `WeakenSpell`: Des Mani.
pub static WEAKEN_SPELL: SpellType = SpellType {
    full_name: "Server.Spells.First.WeakenSpell",
    parent: Some(&MAGERY_SPELL),
    target_range: Some(muo_content::spells::targeting::target_range),
    info: SpellInfo::new(
        "Weaken",
        "Des Mani",
        &[reagent::GARLIC, reagent::NIGHTSHADE],
    )
    .action(212)
    .hand_effect(9031),
    on_cast,
    family: Some(&FIRST_CIRCLE),
    ..MAGERY_SPELL
};

fn target(w: &mut World, sp: SpellId, targeted: Targeted) {
    let Targeted::Entity(EntityId::Mobile(mut m)) = targeted else {
        return;
    };
    let Some(caster) = spell::caster(w, sp) else {
        return;
    };
    if spell::check_h_sequence(w, sp, m) {
        helper::turn(
            w,
            caster,
            Some(helper::Point3dRef::Entity(EntityId::Mobile(m))),
        );
        let mut c = caster;
        helper::check_reflect(w, SpellCircle::First as i32, &mut c, &mut m);

        // TODO: StoneForm immunity
        let length = helper::get_duration(w, caster, m);
        helper::add_stat_curse(w, caster, m, StatType::STR, length, false);

        spell::caster_hurt(w, m);
        muo_server::mobile::base::set_paralyzed(w, m, false);

        effects::fixed_particles(w, m, 0x3779, 10, 15, 5009, EffectLayer::Waist);
        effects::play_sound(w, m, 0x1E6);

        let percentage = (helper::get_offset_scalar(w, caster, m, true) * 100.0) as i32;
        if w.mobile(m)
            .is_some_and(|x| x.ty.is(&muo_content::player_mobile::PLAYER_MOBILE))
        {
            muo_content::player_mobile::buff::add_buff(
                w,
                m,
                BuffInfo::new(BuffIcon::Weaken, 1075837)
                    .duration(length)
                    .args(TextDefinition::of_text(percentage.to_string())),
            );
        }
        spell::harmful_spell(w, sp, m);
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
