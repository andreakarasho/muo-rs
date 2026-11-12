//! The Mass Dispel spell.

use muo_content::creature::ctype;
use muo_content::spells::helper::{self, Point3dRef};
use muo_content::spells::magery::{MAGERY_SPELL, SEVENTH_CIRCLE};
use muo_content::spells::spell::{self, SpellId, SpellType};
use muo_content::spells::targeting::{SpellTargetKind, begin_spell_target};
use muo_content::spells::{SpellInfo, reagent};
use muo_primitives::utility;
use muo_server::World;
use muo_server::item::effect_item;
use muo_server::mobile::base as mb;
use muo_server::mobile::part2::effects;
use muo_server::skills::{self, SkillName};
use muo_server::target::{TargetFlags, Targeted};

use crate::common::{mobiles_in_range, surface_top};

/// `MassDispelSpell`: Vas An Ort.
pub static MASS_DISPEL_SPELL: SpellType = SpellType {
    full_name: "Server.Spells.Seventh.MassDispelSpell",
    parent: Some(&MAGERY_SPELL),
    target_range: Some(muo_content::spells::targeting::target_range),
    info: SpellInfo::new(
        "Mass Dispel",
        "Vas An Ort",
        &[
            reagent::GARLIC,
            reagent::MANDRAKE_ROOT,
            reagent::BLACK_PEARL,
            reagent::SULFUROUS_ASH,
        ],
    )
    .action(263)
    .hand_effect(9002),
    on_cast,
    family: Some(&SEVENTH_CIRCLE),
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
            let mut queue = Vec::new();

            // `map.GetMobilesInRange<BaseCreature>(p, 8)`
            for bc in mobiles_in_range(w, map, p, 8) {
                let Some(c) = ctype(w, bc) else {
                    continue;
                };
                if !((c.is_dispellable)(w, bc) && mb::can_be_harmful_msg(w, caster, bc, false)) {
                    continue;
                }

                let dispel_chance = (50.0
                    + 100.0
                        * (skills::value(w, caster, SkillName::Magery)
                            - (c.dispel_difficulty)(w, bc))
                        / ((c.dispel_focus)(w, bc) * 2.0))
                    / 100.0;

                if dispel_chance > utility::random_double(&mut w.rng) {
                    let (loc, bmap) = w
                        .mobile(bc)
                        .map(|x| (x.location, x.map))
                        .unwrap_or_default();
                    let e =
                        effect_item::create_targeted(w, loc, bmap, effect_item::DEFAULT_DURATION);
                    muo_server::effects::send_location_particles(w, e, 0x3728, 8, 20, 5042);
                    muo_server::effects::play_sound_entity(w, bc, 0x201);

                    queue.push(bc);
                } else {
                    mb::do_harmful(w, caster, bc, false);
                    effects::fixed_effect(w, bc, 0x3779, 10, 20);
                }
            }

            for bc in queue {
                if let Some(ty) = w.mobile(bc).map(|x| x.ty) {
                    (ty.delete)(w, bc);
                }
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
