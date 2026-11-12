//! The Dispel spell.

use muo_content::creature::ctype;
use muo_content::spells::magery::{MAGERY_SPELL, SIXTH_CIRCLE, check_resisted};
use muo_content::spells::spell::{self, SpellId, SpellType};
use muo_content::spells::targeting::{SpellTargetKind, begin_spell_target};
use muo_content::spells::{SpellInfo, reagent};
use muo_primitives::utility;
use muo_server::World;
use muo_server::ids::MobileId;
use muo_server::item::effect_item;
use muo_server::mobile::part2::{effects, messages as mm};
use muo_server::skills::{self, SkillName};
use muo_server::target::{TargetFlags, Targeted};

use crate::common::{mobile_of, turn_to};

/// `DispelSpell`: An Ort.
pub static DISPEL_SPELL: SpellType = SpellType {
    full_name: "Server.Spells.Sixth.DispelSpell",
    parent: Some(&MAGERY_SPELL),
    target_range: Some(muo_content::spells::targeting::target_range),
    info: SpellInfo::new(
        "Dispel",
        "An Ort",
        &[
            reagent::GARLIC,
            reagent::MANDRAKE_ROOT,
            reagent::SULFUROUS_ASH,
        ],
    )
    .action(218)
    .hand_effect(9002),
    on_cast,
    family: Some(&SIXTH_CIRCLE),
    ..MAGERY_SPELL
};

/// The `WeaponHooks::dispel_check_resisted` body: `new DispelSpell(caster).CheckResisted(target)`.
pub fn new_dispel_check_resisted(w: &mut World, caster: MobileId, target: MobileId) -> bool {
    let sp = spell::new_spell(w, &DISPEL_SPELL, caster, None);
    let r = check_resisted(w, sp, target);
    spell::remove_spell(w, sp);
    r
}

fn target(w: &mut World, sp: SpellId, targeted: Targeted) {
    let Some(m) = mobile_of(targeted) else {
        return;
    };
    let Some(caster) = spell::caster(w, sp) else {
        return;
    };
    let bc = ctype(w, m).filter(|c| (c.is_dispellable)(w, m));
    let Some(bc) = bc else {
        mm::send_localized_message(w, caster, 1005049, "", 0x3B2); // That cannot be dispelled.
        return;
    };

    if spell::check_h_sequence(w, sp, m) {
        turn_to(w, caster, m);

        let dispel_chance = (50.0
            + 100.0 * (skills::value(w, caster, SkillName::Magery) - (bc.dispel_difficulty)(w, m))
                / ((bc.dispel_focus)(w, m) * 2.0))
            / 100.0;

        if dispel_chance > utility::random_double(&mut w.rng) {
            let (loc, map) = w.mobile(m).map(|x| (x.location, x.map)).unwrap_or_default();
            let e = effect_item::create_targeted(w, loc, map, effect_item::DEFAULT_DURATION);
            muo_server::effects::send_location_particles(w, e, 0x3728, 8, 20, 5042);
            muo_server::effects::play_sound_entity(w, m, 0x201);

            if let Some(ty) = w.mobile(m).map(|x| x.ty) {
                (ty.delete)(w, m);
            }
        } else {
            effects::fixed_effect(w, m, 0x3779, 10, 20);
            mm::send_localized_message(w, caster, 1010084, "", 0x3B2); // The creature resisted the attempt to dispel it!
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
