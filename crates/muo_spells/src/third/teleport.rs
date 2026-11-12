//! The Teleport spell.

use muo_content::creature::pets;
use muo_content::misc::stamina_system;
use muo_content::regions::house::HOUSE_REGION;
use muo_content::spells::helper::{self, Point3dRef};
use muo_content::spells::hooks::hooks;
use muo_content::spells::magery::{MAGERY_SPELL, THIRD_CIRCLE};
use muo_content::spells::spell::{self, SpellId, SpellType};
use muo_content::spells::targeting::{SpellTargetKind, begin_spell_target};
use muo_content::spells::{SpellInfo, TravelCheckType, reagent};
use muo_primitives::Point3D;
use muo_server::World;
use muo_server::effects::send_location_particles;
use muo_server::item::effect_item;
use muo_server::mobile::part1::lifecycle;
use muo_server::mobile::part2::{effects, messages as mm};
use muo_server::region as tree;
use muo_server::target::{TargetFlags, Targeted};

use crate::common::{item_is, surface_top};

/// `TeleportSpell`: Rel Por.
pub static TELEPORT_SPELL: SpellType = SpellType {
    full_name: "Server.Spells.Third.TeleportSpell",
    parent: Some(&MAGERY_SPELL),
    target_range: Some(muo_content::spells::targeting::target_range),
    info: SpellInfo::new(
        "Teleport",
        "Rel Por",
        &[reagent::BLOODMOSS, reagent::MANDRAKE_ROOT],
    )
    .action(215)
    .hand_effect(9031),
    on_cast,
    check_cast,
    family: Some(&THIRD_CIRCLE),
    ..MAGERY_SPELL
};

fn sigil_exists_on(w: &World, m: muo_server::ids::MobileId) -> bool {
    hooks(w)
        .and_then(|h| h.sigil_exists_on)
        .is_some_and(|f| f(w, m))
}

/// `CheckTravel(caster, to, map, TeleportTo)`: sends the failure message and answers whether it is refused.
fn travel_to_refused(
    w: &mut World,
    caster: muo_server::ids::MobileId,
    map: Option<muo_server::map::MapId>,
    to: Point3D,
) -> bool {
    let c = helper::check_travel_at(w, Some(caster), map, to, TravelCheckType::TeleportTo);
    if !c.allowed
        && let Some(m) = &c.message
    {
        muo_content::text::send_message_to(w, Some(m), caster);
    }
    !c.allowed
}

fn target(w: &mut World, sp: SpellId, targeted: Targeted) {
    let Some(caster) = spell::caster(w, sp) else {
        return;
    };
    let orig = Point3dRef::from(targeted);
    let map = w.mobile(caster).and_then(|c| c.map);
    let to = surface_top(w, targeted);
    let from = w.mobile(caster).map(|c| c.location).unwrap_or_default();

    if sigil_exists_on(w, caster) {
        mm::send_localized_message(w, caster, 1061632, "", 0x3B2); // You can't do that while carrying the sigil.
    } else if stamina_system::is_overloaded(w, caster) {
        mm::send_localized_message(w, caster, 502359, "", 0x22); // Thou art too encumbered to move.
    } else if !helper::check_travel(w, caster, TravelCheckType::TeleportFrom).allowed {
        let c = helper::check_travel(w, caster, TravelCheckType::TeleportFrom);
        if let Some(m) = &c.message {
            muo_content::text::send_message_to(w, Some(m), caster);
        }
    } else if travel_to_refused(w, caster, map, to) {
        // failure message sent above
    } else if !map
        .is_some_and(|m| muo_server::map::fit::can_spawn_mobile_at(w, m, to.x, to.y, to.z))
    {
        mm::send_localized_message(w, caster, 501942, "", 0x3B2); // That location is blocked.
    } else if helper::check_multi(w, to, map, true, 0) {
        mm::send_localized_message(w, caster, 502831, "", 0x3B2); // Cannot teleport to that spot.
    } else if tree::is_part_of_type(w, tree::find(w, to, map), &HOUSE_REGION) {
        mm::send_localized_message(w, caster, 502829, "", 0x3B2); // Cannot teleport to that spot.
    } else if spell::check_sequence(w, sp) {
        helper::turn(w, caster, Some(orig));

        let m = caster;
        lifecycle::set_location(w, m, to);
        muo_server::mobile::base::process_delta(w, m);

        if w.mobile(m).is_some_and(|x| x.player) {
            let map = w.mobile(m).and_then(|x| x.map);
            let e = effect_item::create_targeted(w, from, map, effect_item::DEFAULT_DURATION);
            send_location_particles(w, e, 0x3728, 10, 10, 2023);

            let e = effect_item::create_targeted(w, to, map, effect_item::DEFAULT_DURATION);
            send_location_particles(w, e, 0x3728, 10, 10, 5023);
        } else {
            effects::fixed_particles(
                w,
                m,
                0x376A,
                9,
                32,
                0x13AF,
                muo_server::effects::EffectLayer::Waist,
            );
        }

        effects::play_sound(w, m, 0x1FE);

        // Use a queue just in case OnMoveOver changes the item's sector
        let queue: Vec<_> = muo_server::mobile::part2::enumerators::get_items_at(w, m)
            .into_iter()
            .filter(|&i| {
                item_is(w, i, "Server.Spells.Sixth.ParalyzeField")
                    || item_is(w, i, "Server.Spells.Fifth.PoisonField")
                    || item_is(w, i, "Server.Spells.Fourth.FireFieldItem")
            })
            .collect();
        for item in queue {
            if let Some(ty) = w.item(item).map(|i| i.ty) {
                (ty.on_move_over)(w, item, m);
            }
        }
    }
    let _ = pets::teleport_pets;
    let _ = Point3D::ZERO;
}

fn check_cast(w: &mut World, sp: SpellId) -> bool {
    let Some(caster) = spell::caster(w, sp) else {
        return false;
    };
    if sigil_exists_on(w, caster) {
        mm::send_localized_message(w, caster, 1061632, "", 0x3B2); // You can't do that while carrying the sigil.
        return false;
    }
    if stamina_system::is_overloaded(w, caster) {
        mm::send_localized_message(w, caster, 502359, "", 0x22); // Thou art too encumbered to move.
        return false;
    }
    let c = helper::check_travel(w, caster, TravelCheckType::TeleportFrom);
    if !c.allowed {
        if let Some(m) = &c.message {
            muo_content::text::send_message_to(w, Some(m), caster);
        }
        return false;
    }
    true
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
