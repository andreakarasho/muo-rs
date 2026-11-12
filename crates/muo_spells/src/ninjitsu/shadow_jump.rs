//! The Shadow Jump spell.

use muo_content::misc::stamina_system;
use muo_content::regions::house::HOUSE_REGION;
use muo_content::skills::stealth;
use muo_content::spells::helper::{self, Point3dRef, TravelCheck};
use muo_content::spells::hooks::hooks;
use muo_content::spells::spell::{self, SpellId, SpellType};
use muo_content::spells::targeting::{SpellTargetKind, begin_spell_target};
use muo_content::spells::{SpellInfo, TravelCheckType};
use muo_primitives::TimeSpan;
use muo_server::World;
use muo_server::ids::MobileId;
use muo_server::item::effect_item;
use muo_server::mobile::part1::lifecycle;
use muo_server::mobile::part2::{effects, messages as mm};
use muo_server::region as regions;
use muo_server::target::{TargetFlags, Targeted};

use super::ninja::{NINJA_SPELL, NinjaType};
use crate::common::surface_top;

static SHADOW_JUMP: NinjaType = NinjaType {
    required_skill: 50.0,
    required_mana: 15,
    required_skill_pre_ml: None,
    required_mana_pre_ml: None,
};

/// `Shadowjump`.
pub static SHADOWJUMP_SPELL: SpellType = SpellType {
    full_name: "Server.Spells.Ninjitsu.Shadowjump",
    parent: Some(&NINJA_SPELL),
    target_range: Some(|_| 11),
    info: SpellInfo::new("Shadowjump", "", &[])
        .action(-1)
        .hand_effect(9002),
    cast_delay_base: |_, _| TimeSpan::from_secs(1),
    blocked_by_animal_form: false,
    check_cast,
    check_disturb: |_, _, _, _, _| false,
    on_cast,
    family: Some(&SHADOW_JUMP),
    ..NINJA_SPELL
};

fn is_stealthing(w: &World, m: MobileId) -> bool {
    muo_content::player_mobile::data(w, m).is_some_and(|d| d.is_stealthing)
}

/// `CheckTravel(caster, TeleportFrom)` (`at` None) or `CheckTravel(caster, to, map, TeleportTo)`: sends the
/// failure message and answers whether the travel is refused.
fn travel_refused(
    w: &mut World,
    caster: MobileId,
    at: Option<(Option<muo_server::map::MapId>, muo_primitives::Point3D)>,
) -> bool {
    let c = match at {
        None => helper::check_travel(w, caster, TravelCheckType::TeleportFrom),
        Some((map, to)) => {
            helper::check_travel_at(w, Some(caster), map, to, TravelCheckType::TeleportTo)
        }
    };
    send_failure(w, caster, &c);
    !c.allowed
}

fn send_failure(w: &mut World, caster: MobileId, check: &TravelCheck) {
    if let Some(m) = &check.message {
        muo_content::text::send_message_to(w, Some(m), caster);
    }
}

fn target(w: &mut World, sp: SpellId, targeted: Targeted) {
    let Some(caster) = spell::caster(w, sp) else {
        return;
    };
    let orig = Point3dRef::from(targeted);
    let map = w.mobile(caster).and_then(|c| c.map);
    let to = surface_top(w, targeted);
    let from = w.mobile(caster).map(|c| c.location).unwrap_or_default();

    if !is_stealthing(w, caster) {
        mm::send_localized_message(w, caster, 1063087, "", 0x3B2); // You must be in stealth mode to use this ability.
    } else if hooks(w)
        .and_then(|h| h.sigil_exists_on)
        .is_some_and(|f| f(w, caster))
    {
        mm::send_localized_message(w, caster, 1061632, "", 0x3B2); // You can't do that while carrying the sigil.
    } else if stamina_system::is_overloaded(w, caster) {
        mm::send_localized_message(w, caster, 502359, "", 0x22); // Thou art too encumbered to move.
    } else if travel_refused(w, caster, None) || travel_refused(w, caster, Some((map, to))) {
        // failure message sent by the check
    } else if !map
        .is_some_and(|m| muo_server::map::fit::can_spawn_mobile_at(w, m, to.x, to.y, to.z))
        || helper::check_multi(w, to, map, true, 5)
    {
        mm::send_localized_message(w, caster, 502831, "", 0x3B2); // Cannot teleport to that spot.
    } else if regions::is_part_of_type(w, regions::find(w, to, map), &HOUSE_REGION) {
        mm::send_localized_message(w, caster, 502829, "", 0x3B2); // Cannot teleport to that spot.
    } else if spell::check_sequence(w, sp) {
        helper::turn(w, caster, Some(orig));

        let m = caster;
        lifecycle::set_location(w, m, to);
        muo_server::mobile::base::process_delta(w, m);

        let map = w.mobile(m).and_then(|x| x.map);
        let e = effect_item::create_targeted(w, from, map, effect_item::DEFAULT_DURATION);
        muo_server::effects::send_location_particles(w, e, 0x3728, 10, 10, 2023);

        effects::play_sound(w, m, 0x512);

        stealth::stealth_on_use(w, m); // stealth check after the shadow jump
    }
}

fn check_cast(w: &mut World, sp: SpellId) -> bool {
    let Some(caster) = spell::caster(w, sp) else {
        return false;
    };
    // IsStealthing should be moved to Server.Mobiles
    if !is_stealthing(w, caster) {
        mm::send_localized_message(w, caster, 1063087, "", 0x3B2); // You must be in stealth mode to use this ability.
        return false;
    }

    (NINJA_SPELL.check_cast)(w, sp)
}

fn on_cast(w: &mut World, sp: SpellId) {
    if let Some(caster) = spell::caster(w, sp) {
        mm::send_localized_message(w, caster, 1063088, "", 0x3B2); // You prepare to perform a Shadowjump.
    }
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
