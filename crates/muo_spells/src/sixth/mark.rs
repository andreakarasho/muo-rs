//! The Mark spell.

use muo_content::spells::helper;
use muo_content::spells::hooks::hooks;
use muo_content::spells::magery::{MAGERY_SPELL, SIXTH_CIRCLE};
use muo_content::spells::spell::{self, SpellId, SpellType};
use muo_content::spells::targeting::{SpellTargetKind, begin_spell_target};
use muo_content::spells::{SpellInfo, TravelCheckType, reagent};
use muo_primitives::MessageType;
use muo_server::World;
use muo_server::mobile::part2::messages as mm;
use muo_server::target::{TargetFlags, Targeted};

use crate::common::item_of;

/// `MarkSpell`: Kal Por Ylem.
pub static MARK_SPELL: SpellType = SpellType {
    full_name: "Server.Spells.Sixth.MarkSpell",
    parent: Some(&MAGERY_SPELL),
    target_range: Some(muo_content::spells::targeting::target_range),
    info: SpellInfo::new(
        "Mark",
        "Kal Por Ylem",
        &[
            reagent::BLACK_PEARL,
            reagent::BLOODMOSS,
            reagent::MANDRAKE_ROOT,
        ],
    )
    .action(218)
    .hand_effect(9002),
    on_cast,
    check_cast,
    family: Some(&SIXTH_CIRCLE),
    ..MAGERY_SPELL
};

fn send_failure(w: &mut World, caster: muo_server::ids::MobileId, c: &helper::TravelCheck) {
    if let Some(m) = &c.message {
        muo_content::text::send_message_to(w, Some(m), caster);
    }
}

fn target(w: &mut World, sp: SpellId, targeted: Targeted) {
    let Some(item) = item_of(targeted) else {
        return;
    };
    let Some(caster) = spell::caster(w, sp) else {
        return;
    };
    let is_rune = hooks(w)
        .and_then(|h| h.is_recall_rune)
        .is_some_and(|f| f(w, item));
    if !is_rune {
        let name = w
            .mobile(caster)
            .and_then(|c| (c.ty.name)(w, caster))
            .unwrap_or_default();
        // I cannot mark that object.
        // NetState.SendMessageLocalized(serial, body, Regular, 0x3B2, 3, 501797, Caster.Name): name, not args
        if let Some((ns, serial)) = w.mobile(caster).map(|c| (c.net_state, c.serial)) {
            let body = muo_server::mobile::part1::props_b::body(w, caster).body_id();
            if let Some(ns) = ns {
                muo_server::packets::message::send_message_localized(
                    w,
                    ns,
                    serial,
                    body,
                    muo_server::packets::message::MessageType::from_bits_retain(
                        MessageType::REGULAR.bits() as u8,
                    ),
                    0x3B2,
                    3,
                    501797,
                    &name,
                    "",
                );
            }
        }
        return;
    }

    let travel = helper::check_travel(w, caster, TravelCheckType::Mark);
    let (loc, map) = w
        .mobile(caster)
        .map(|c| (c.location, c.map))
        .unwrap_or_default();
    if !travel.allowed {
        send_failure(w, caster, &travel);
    } else if helper::check_multi(w, loc, map, !muo_content::core::aos(w), 0) {
        mm::send_localized_message(w, caster, 501942, "", 0x3B2); // That location is blocked.
    } else if !is_child_of_backpack(w, item, caster) {
        // You must have this rune in your backpack in order to mark it.
        mm::local_overhead_message_number(w, caster, MessageType::REGULAR, 0x3B2, 1062422, "");
    } else if spell::check_sequence(w, sp) {
        if let Some(f) = hooks(w).and_then(|h| h.rune_mark) {
            f(w, item, caster);
        }
        muo_server::mobile::part2::effects::play_sound(w, caster, 0x1FA);
        muo_server::effects::send_location_effect_entity(w, caster, 14201, 16, 10, 0, 0);
    }
}

/// `rune.IsChildOf(Caster.Backpack)`.
fn is_child_of_backpack(
    w: &mut World,
    item: muo_server::ids::ItemId,
    caster: muo_server::ids::MobileId,
) -> bool {
    let Some(pack) = muo_server::mobile::part1::props_b::backpack(w, caster) else {
        return false;
    };
    muo_server::item::ops::is_child_of(w, item, Some(pack.into()))
}

fn on_cast(w: &mut World, sp: SpellId) {
    begin_spell_target(
        w,
        sp,
        false,
        TargetFlags::NONE,
        false,
        SpellTargetKind::Item,
        target,
    );
}

fn check_cast(w: &mut World, sp: SpellId) -> bool {
    if spell::base_check_cast(w, sp) {
        return true;
    }
    let Some(caster) = spell::caster(w, sp) else {
        return false;
    };
    let c = helper::check_travel(w, caster, TravelCheckType::Mark);
    if !c.allowed {
        send_failure(w, caster, &c);
        return false;
    }
    true
}
