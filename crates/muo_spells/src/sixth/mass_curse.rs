//! The Mass Curse spell.

use muo_content::core;
use muo_content::spells::helper::{self, Point3dRef};
use muo_content::spells::magery::{MAGERY_SPELL, SIXTH_CIRCLE};
use muo_content::spells::spell::{self, SpellId, SpellType};
use muo_content::spells::targeting::{SpellTargetKind, begin_spell_target};
use muo_content::spells::{SpellInfo, reagent};
use muo_server::World;
use muo_server::effects::EffectLayer;
use muo_server::mobile::StatType;
use muo_server::mobile::base as mb;
use muo_server::mobile::part2::effects;
use muo_server::target::{TargetFlags, Targeted};

use crate::common::{mobiles_in_range, surface_top};

/// `MassCurseSpell`: Vas Des Sanct.
pub static MASS_CURSE_SPELL: SpellType = SpellType {
    full_name: "Server.Spells.Sixth.MassCurseSpell",
    parent: Some(&MAGERY_SPELL),
    target_range: Some(muo_content::spells::targeting::target_range),
    info: SpellInfo::new(
        "Mass Curse",
        "Vas Des Sanct",
        &[
            reagent::GARLIC,
            reagent::NIGHTSHADE,
            reagent::MANDRAKE_ROOT,
            reagent::SULFUROUS_ASH,
        ],
    )
    .action(218)
    .hand_effect(9031)
    .allow_town(false),
    on_cast,
    family: Some(&SIXTH_CIRCLE),
    ..MAGERY_SPELL
};

fn target(w: &mut World, sp: SpellId, targeted: Targeted) {
    let Some(caster) = spell::caster(w, sp) else {
        return;
    };
    if helper::check_town(w, Point3dRef::from(targeted), caster) && spell::check_sequence(w, sp) {
        helper::turn(w, caster, Some(Point3dRef::from(targeted)));
        let p = surface_top(w, targeted);
        let map = w.mobile(caster).and_then(|c| c.map);
        if map.is_some() {
            for m in mobiles_in_range(w, map, p, 2) {
                if core::aos(w)
                    && (m == caster
                        || !helper::valid_indirect_target(w, caster, m)
                        || !mb::can_see_mobile(w, caster, m)
                        || !mb::can_be_harmful_msg(w, caster, m, false))
                {
                    continue;
                }

                mb::do_harmful(w, caster, m, false);

                let length = helper::get_duration(w, caster, m);
                helper::add_stat_curse(w, caster, m, StatType::STR, length, false);
                helper::add_stat_curse(w, caster, m, StatType::DEX, length, true);
                helper::add_stat_curse(w, caster, m, StatType::INT, length, true);

                effects::fixed_particles(w, m, 0x374A, 10, 15, 5028, EffectLayer::Waist);
                effects::play_sound(w, m, 0x1FB);

                spell::harmful_spell(w, sp, m);
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
