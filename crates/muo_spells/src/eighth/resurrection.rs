//! The Resurrection spell.

use muo_content::creature::ctype;
use muo_content::spells::magery::{EIGHTH_CIRCLE, MAGERY_SPELL};
use muo_content::spells::spell::{self, SpellId, SpellType};
use muo_content::spells::targeting::{SpellTargetKind, begin_spell_target};
use muo_content::spells::{SpellInfo, reagent};
use muo_gumps::m_z::resurrect;
use muo_server::World;
use muo_server::mobile::base as mb;
use muo_server::mobile::part2::{effects, messages as mm};
use muo_server::region as regions;
use muo_server::target::{TargetFlags, Targeted};

use crate::common::{alive, check_cast_sudden_death, mobile_of, turn_to};

/// `ResurrectionSpell`: An Corp.
pub static RESURRECTION_SPELL: SpellType = SpellType {
    full_name: "Server.Spells.Eighth.ResurrectionSpell",
    parent: Some(&MAGERY_SPELL),
    target_range: Some(|_| 1),
    info: SpellInfo::new(
        "Resurrection",
        "An Corp",
        &[reagent::BLOODMOSS, reagent::GARLIC, reagent::GINSENG],
    )
    .action(245)
    .hand_effect(9062),
    on_cast,
    check_cast: check_cast_sudden_death,
    family: Some(&EIGHTH_CIRCLE),
    ..MAGERY_SPELL
};

/// `TargetRange => 1`.
const TARGET_RANGE: i32 = 1;

/// `Region.Find(m.Location, m.Map).IsPartOf("Khaldun")` of the target.
fn in_khaldun(w: &World, m: muo_server::ids::MobileId) -> bool {
    let r = muo_server::mobile::part1::props_b::region(w, m);
    regions::is_part_of_name(w, r, "Khaldun", false)
}

fn target(w: &mut World, sp: SpellId, targeted: Targeted) {
    let Some(m) = mobile_of(targeted) else {
        return;
    };
    let Some(caster) = spell::caster(w, sp) else {
        return;
    };
    let player = w.mobile(m).is_some_and(|x| x.player);
    let (mloc, mmap) = w.mobile(m).map(|x| (x.location, x.map)).unwrap_or_default();
    let in_range = w
        .mobile(caster)
        .map(|c| c.location)
        .is_some_and(|l| mb::in_range_point3d(w, m, l, TARGET_RANGE));

    if m == caster {
        mm::send_localized_message(w, caster, 501039, "", 0x3B2); // Thou can not resurrect thyself.
    } else if ctype(w, m).is_some() && mb::is_dead_bonded_pet(w, m) {
        mm::send_message(w, caster, "Target can not be revived this way.");
    } else if !player {
        mm::send_localized_message(w, caster, 501043, "", 0x3B2); // Target is not a being.
    } else if alive(w, m) {
        mm::send_localized_message(w, caster, 501041, "", 0x3B2); // Target is not dead.
    } else if !in_range {
        mm::send_localized_message(w, caster, 501042, "", 0x3B2); // Target is not close enough.
    } else if !mmap.is_some_and(|mp| {
        muo_server::map::fit::can_fit(w, mp, mloc.x, mloc.y, mloc.z, 16, false, false, true)
    }) {
        mm::send_localized_message(w, caster, 501042, "", 0x3B2); // Target can not be resurrected at that location.
        mm::send_localized_message(w, m, 502391, "", 0x3B2); // Thou can not be resurrected there!
    } else if in_khaldun(w, m) {
        // The veil of death in this area is too strong and resists thy efforts to restore life.
        mm::send_localized_message(w, caster, 1010395, "", 0x3B2);
    } else if spell::check_b_sequence(w, sp, m, true) {
        turn_to(w, caster, m);

        effects::play_sound(w, m, 0x214);
        effects::fixed_effect(w, m, 0x376A, 10, 16);

        let gump = Box::new(resurrect::new(
            Some(caster),
            resurrect::ResurrectMessage::Generic,
            false,
            0.0,
            0,
        ));
        muo_content::gumps::system::send_gump_to_mobile(w, m, gump, false);
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
