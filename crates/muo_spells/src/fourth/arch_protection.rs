//! The Arch Protection spell.

use indexmap::IndexMap;
use muo_content::core;
use muo_content::spells::helper::{self, Point3dRef};
use muo_content::spells::hooks::hooks;
use muo_content::spells::magery::{FOURTH_CIRCLE, MAGERY_SPELL};
use muo_content::spells::spell::{self, SpellId, SpellType};
use muo_content::spells::targeting::{SpellTargetKind, begin_spell_target};
use muo_content::spells::{SpellInfo, reagent};
use muo_primitives::TimeSpan;
use muo_server::World;
use muo_server::effects::EffectLayer;
use muo_server::ids::MobileId;
use muo_server::mobile::base as mb;
use muo_server::mobile::part1::{combat, props_a};
use muo_server::mobile::part2::effects;
use muo_server::skills::{self, SkillName};
use muo_server::target::{TargetFlags, Targeted};
use muo_server::timer;

use crate::common::{mobiles_in_range, surface_top};
use crate::second::protection;

/// `ArchProtectionSpell`: Vas Uus Sanct.
pub static ARCH_PROTECTION_SPELL: SpellType = SpellType {
    full_name: "Server.Spells.Fourth.ArchProtectionSpell",
    parent: Some(&MAGERY_SPELL),
    target_range: Some(muo_content::spells::targeting::target_range),
    info: SpellInfo::new(
        "Arch Protection",
        "Vas Uus Sanct",
        &[
            reagent::GARLIC,
            reagent::GINSENG,
            reagent::MANDRAKE_ROOT,
            reagent::SULFUROUS_ASH,
        ],
    )
    .action(239)
    .hand_effect(9011)
    .pre_aos(215, 9011),
    on_cast,
    family: Some(&FOURTH_CIRCLE),
    ..MAGERY_SPELL
};

/// `_table`.
#[derive(Default)]
struct Table(IndexMap<MobileId, i32>);

fn table(w: &mut World) -> &mut IndexMap<MobileId, i32> {
    &mut w.resources.get_mut::<Table>().0
}

const ACTION_KEY: &str = "Server.Spells.Fourth.ArchProtectionSpell";

fn target(w: &mut World, sp: SpellId, targeted: Targeted) {
    let Some(caster) = spell::caster(w, sp) else {
        return;
    };
    let Some(map) = w.mobile(caster).and_then(|c| c.map) else {
        return;
    };

    if spell::check_sequence(w, sp) {
        helper::turn(w, caster, Some(Point3dRef::from(targeted)));
        let loc = surface_top(w, targeted);

        if !core::aos(w) {
            muo_server::effects::play_sound(w, loc, Some(map), 0x299);
        }

        let range = if core::aos(w) { 2 } else { 3 };
        let mut targets = Vec::new();
        for m in mobiles_in_range(w, Some(map), loc, range) {
            if mb::can_be_beneficial_msg(w, caster, m, false) {
                targets.push(m);
            }
        }

        if core::aos(w) {
            let party_contains = hooks(w).and_then(|h| h.party_contains);
            for m in targets {
                if m == caster || party_contains.is_some_and(|f| f(w, caster, m)) {
                    mb::do_beneficial(w, caster, m);
                    protection::toggle(w, caster, m);
                }
            }
        } else if !core::uor(w) {
            for m in targets {
                if !protection::has_t2a_protection(w, m) {
                    mb::do_beneficial(w, caster, m);
                    protection::apply_t2a_protection(w, caster, m);
                    effects::play_sound(w, m, 0x1F7);
                }
            }
        } else {
            let val = (skills::value(w, caster, SkillName::Magery) / 10.0 + 1.0) as i32;
            for m in targets {
                if combat::begin_action(w, m, ACTION_KEY) {
                    mb::do_beneficial(w, caster, m);

                    let cur = w.mobile(m).map_or(0, |x| x.virtual_armor_mod);
                    props_a::set_virtual_armor_mod(w, m, cur + val);
                    table(w).insert(m, val);

                    let secs = (skills::value(w, caster, SkillName::Magery) * 1.2).min(144.0);
                    timer::start(
                        w,
                        TimeSpan::from_secs_f64(secs),
                        TimeSpan::ZERO,
                        1,
                        move |w| remove_entry(w, m),
                    );

                    effects::fixed_particles(w, m, 0x375A, 9, 20, 5027, EffectLayer::Waist);
                    effects::play_sound(w, m, 0x1F7);
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

/// `RemoveEntry(m)`.
pub fn remove_entry(w: &mut World, m: MobileId) {
    if let Some(v) = table(w).shift_remove(&m) {
        combat::end_action(w, m, ACTION_KEY);
        let cur = w.mobile(m).map_or(0, |x| x.virtual_armor_mod);
        props_a::set_virtual_armor_mod(w, m, cur - v.min(cur));
    }
}
