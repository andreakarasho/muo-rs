//! The Sacred Journey spell.

use muo_content::core;
use muo_content::creature::pets;
use muo_content::misc::stamina_system;
use muo_content::spells::helper::{self, TravelCheck};
use muo_content::spells::hooks::hooks;
use muo_content::spells::spell::{self, SpellId, SpellType};
use muo_content::spells::targeting::{RecallType, begin_recall_target};
use muo_content::spells::{SpellInfo, TravelCheckType};
use muo_primitives::{Point3D, TimeSpan};
use muo_server::World;
use muo_server::ids::{ItemId, MobileId};
use muo_server::item::effect_item;
use muo_server::map::MapId;
use muo_server::mobile::base as mb;
use muo_server::mobile::part2::{effects, messages as mm};

use super::paladin::{PALADIN_BASE, PALADIN_SPELL, PaladinType};
use crate::common::is_player_mobile;
use crate::fourth::recall::RecallState;

static SACRED_JOURNEY: PaladinType = PaladinType {
    required_skill: 15.0,
    required_mana: 10,
    required_tithing: 15,
    mantra_number: 1060727, // Sanctum Viatas
    ..PALADIN_BASE
};

pub static SACRED_JOURNEY_INTERFACE: RecallType = RecallType { effect };

/// `SacredJourneySpell`: Sanctum Viatas.
pub static SACRED_JOURNEY_SPELL: SpellType = SpellType {
    full_name: "Server.Spells.Chivalry.SacredJourneySpell",
    parent: Some(&PALADIN_SPELL),
    target_range: Some(muo_content::spells::targeting::target_range),
    info: SpellInfo::new("Sacred Journey", "Sanctum Viatas", &[])
        .action(-1)
        .hand_effect(9002),
    cast_delay_base: |_, _| TimeSpan::from_millis(1500),
    on_cast,
    check_cast,
    blocks_movement: |_, _| false,
    family: Some(&SACRED_JOURNEY),
    interface: Some(&SACRED_JOURNEY_INTERFACE),
    ..PALADIN_SPELL
};

/// `new SacredJourneySpell(caster, entry, book, scroll)`.
pub fn new_sacred_journey_spell(
    w: &mut World,
    caster: MobileId,
    entry: Option<(Point3D, Option<MapId>)>,
    book: Option<ItemId>,
    scroll: Option<ItemId>,
) -> SpellId {
    let id = spell::new_spell(w, &SACRED_JOURNEY_SPELL, caster, scroll);
    if let Some(s) = spell::spells(w).get_mut(id) {
        s.ext = muo_server::Ext::new(RecallState { entry, book });
    }
    id
}

fn state(w: &World, sp: SpellId) -> RecallState {
    spell::spell(w, sp)
        .and_then(|s| s.ext.get::<RecallState>().cloned())
        .unwrap_or_default()
}

fn send_failure(w: &mut World, caster: MobileId, check: &TravelCheck) {
    if let Some(m) = &check.message {
        muo_content::text::send_message_to(w, Some(m), caster);
    }
}

fn sigil_exists_on(w: &World, m: MobileId) -> bool {
    hooks(w)
        .and_then(|h| h.sigil_exists_on)
        .is_some_and(|f| f(w, m))
}

/// `Effect(loc, map, checkMulti)`.
fn effect(w: &mut World, sp: SpellId, loc: Point3D, map: Option<MapId>, check_multi: bool) {
    let Some(caster) = spell::caster(w, sp) else {
        return;
    };
    let book = state(w, sp).book;
    let cmap = w.mobile(caster).and_then(|m| m.map);

    if sigil_exists_on(w, caster) {
        mm::send_localized_message(w, caster, 1061632, "", 0x3B2); // You can't do that while carrying the sigil.
        return;
    }
    let Some(map) = map.filter(|&m| core::aos(w) || cmap == Some(m)) else {
        mm::send_localized_message(w, caster, 1005569, "", 0x3B2); // You can not recall to another facet.
        return;
    };

    let from_check = helper::check_travel(w, caster, TravelCheckType::RecallFrom);
    if !from_check.allowed {
        send_failure(w, caster, &from_check);
        return;
    }
    let to_check =
        helper::check_travel_at(w, Some(caster), Some(map), loc, TravelCheckType::RecallTo);
    if !to_check.allowed {
        send_failure(w, caster, &to_check);
        return;
    }

    if map == MapId::FELUCCA
        && is_player_mobile(w, caster)
        && muo_content::player_mobile::young(w, caster)
    {
        mm::send_localized_message(w, caster, 1049543, "", 0x3B2); // You decide against traveling to Felucca while you are still young.
    } else if mb::murderer(w, caster) && map != MapId::FELUCCA {
        mm::send_localized_message(w, caster, 1019004, "", 0x3B2); // You are not allowed to travel there.
    } else if w.mobile(caster).is_some_and(|m| m.criminal) {
        mm::send_localized_message(w, caster, 1005561, "", 0x22); // Thou'rt a criminal and cannot escape so easily.
    } else if helper::check_combat(w, caster) {
        mm::send_localized_message(w, caster, 1061282, "", 0x3B2); // You cannot use the Sacred Journey ability to flee from combat.
    } else if stamina_system::is_overloaded(w, caster) {
        mm::send_localized_message(w, caster, 502359, "", 0x22); // Thou art too encumbered to move.
    } else if !muo_server::map::fit::can_spawn_mobile_at(w, map, loc.x, loc.y, loc.z)
        || (check_multi && helper::check_multi(w, loc, Some(map), true, 0))
    {
        mm::send_localized_message(w, caster, 501942, "", 0x3B2); // That location is blocked.
    } else if book.is_some_and(|b| runebook_charges(w, b) <= 0) {
        mm::send_localized_message(w, caster, 502412, "", 0x3B2); // There are no charges left on that item.
    } else if spell::check_sequence(w, sp) {
        pets::teleport_pets(w, caster, loc, map, true);

        if let Some(b) = book {
            add_runebook_charges(w, b, -1);
        }

        let (cloc, cmap) = w
            .mobile(caster)
            .map(|m| (m.location, m.map))
            .unwrap_or_default();
        let e = effect_item::create_targeted(w, cloc, cmap, effect_item::DEFAULT_DURATION);
        muo_server::effects::send_location_particles(w, e, 0, 0, 0, 5033);

        effects::play_sound(w, caster, 0x1FC);
        mb::move_to_world(w, caster, loc, map);
        effects::play_sound(w, caster, 0x1FC);
    }
}

fn runebook_charges(w: &World, book: ItemId) -> i32 {
    w.resources
        .get::<crate::fourth::recall::RecallHooks>()
        .and_then(|h| h.charges)
        .map_or(i32::MAX, |f| f(w, book))
}

fn add_runebook_charges(w: &mut World, book: ItemId, delta: i32) {
    if let Some(f) = w
        .resources
        .get::<crate::fourth::recall::RecallHooks>()
        .and_then(|h| h.add_charges)
    {
        f(w, book, delta);
    }
}

fn on_cast(w: &mut World, sp: SpellId) {
    match state(w, sp).entry {
        None => begin_recall_target(w, sp, true),
        Some((loc, map)) => {
            effect(w, sp, loc, map, true);
            spell::finish_sequence(w, sp);
        }
    }
}

fn check_cast(w: &mut World, sp: SpellId) -> bool {
    if !(PALADIN_SPELL.check_cast)(w, sp) {
        return false;
    }
    let Some(caster) = spell::caster(w, sp) else {
        return false;
    };

    if sigil_exists_on(w, caster) {
        mm::send_localized_message(w, caster, 1061632, "", 0x3B2); // You can't do that while carrying the sigil.
        return false;
    }

    if w.mobile(caster).is_some_and(|m| m.criminal) {
        mm::send_localized_message(w, caster, 1005561, "", 0x22); // Thou'rt a criminal and cannot escape so easily.
        return false;
    }

    if helper::check_combat(w, caster) {
        mm::send_localized_message(w, caster, 1061282, "", 0x3B2); // You cannot use the Sacred Journey ability to flee from combat.
        return false;
    }

    if stamina_system::is_overloaded(w, caster) {
        mm::send_localized_message(w, caster, 502359, "", 0x22); // Thou art too encumbered to move.
        return false;
    }

    let check = helper::check_travel(w, caster, TravelCheckType::RecallFrom);
    if !check.allowed {
        send_failure(w, caster, &check);
        return false;
    }

    true
}
