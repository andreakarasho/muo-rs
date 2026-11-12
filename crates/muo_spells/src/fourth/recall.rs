//! The Recall spell.

use muo_content::core;
use muo_content::creature::pets;
use muo_content::misc::stamina_system;
use muo_content::spells::helper::{self, TravelCheck};
use muo_content::spells::hooks::hooks;
use muo_content::spells::magery::{FOURTH_CIRCLE, MAGERY_SPELL};
use muo_content::spells::spell::{self, SpellId, SpellType};
use muo_content::spells::targeting::{RecallType, begin_recall_target};
use muo_content::spells::{SpellInfo, TravelCheckType, reagent};
use muo_primitives::Point3D;
use muo_server::World;
use muo_server::ids::{ItemId, MobileId};
use muo_server::map::MapId;
use muo_server::mobile::base as mb;
use muo_server::mobile::part2::{effects, messages as mm};

/// What `RecallSpell`'s constructors keep: the runebook entry and book of the runebook form.
#[derive(Clone, Debug, Default)]
pub struct RecallState {
    /// `m_Entry`: the runebook entry's location and map.
    pub entry: Option<(Point3D, Option<MapId>)>,
    /// `m_Book`.
    pub book: Option<ItemId>,
}

/// The wraith form class a recall casts at skill 0 under.
const WRAITH_FORM: &str = "Server.Spells.Necromancy.WraithFormSpell";

pub static RECALL_INTERFACE: RecallType = RecallType { effect };

/// `RecallSpell`: Kal Ort Por.
pub static RECALL_SPELL: SpellType = SpellType {
    full_name: "Server.Spells.Fourth.RecallSpell",
    parent: Some(&MAGERY_SPELL),
    target_range: Some(muo_content::spells::targeting::target_range),
    info: SpellInfo::new(
        "Recall",
        "Kal Ort Por",
        &[
            reagent::BLACK_PEARL,
            reagent::BLOODMOSS,
            reagent::MANDRAKE_ROOT,
        ],
    )
    .action(239)
    .hand_effect(9031),
    on_cast,
    check_cast,
    get_cast_skills,
    family: Some(&FOURTH_CIRCLE),
    interface: Some(&RECALL_INTERFACE),
    ..MAGERY_SPELL
};

/// `new RecallSpell(caster, entry, book, scroll)`: the runebook form.
pub fn new_recall_spell(
    w: &mut World,
    caster: MobileId,
    entry: Option<(Point3D, Option<MapId>)>,
    book: Option<ItemId>,
    scroll: Option<ItemId>,
) -> SpellId {
    let id = spell::new_spell(w, &RECALL_SPELL, caster, scroll);
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
    let caster_map = w.mobile(caster).and_then(|m| m.map);

    if sigil_exists_on(w, caster) {
        mm::send_localized_message(w, caster, 1061632, "", 0x3B2); // You can't do that while carrying the sigil.
        return;
    }
    let Some(map) = map.filter(|&m| core::aos(w) || caster_map == Some(m)) else {
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

    let is_pm = w
        .mobile(caster)
        .is_some_and(|m| m.ty.is(&muo_content::player_mobile::PLAYER_MOBILE));
    if map == MapId::FELUCCA && is_pm && muo_content::player_mobile::young(w, caster) {
        mm::send_localized_message(w, caster, 1049543, "", 0x3B2); // You decide against traveling to Felucca while you are still young.
    } else if mb::murderer(w, caster) && map != MapId::FELUCCA {
        mm::send_localized_message(w, caster, 1019004, "", 0x3B2); // You are not allowed to travel there.
    } else if w.mobile(caster).is_some_and(|m| m.criminal) {
        mm::send_localized_message(w, caster, 1005561, "", 0x22); // Thou'rt a criminal and cannot escape so easily.
    } else if helper::check_combat(w, caster) {
        mm::send_localized_message(w, caster, 1005564, "", 0x22); // Wouldst thou flee during the heat of battle??
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

        effects::play_sound(w, caster, 0x1FC);
        mb::move_to_world(w, caster, loc, map);
        effects::play_sound(w, caster, 0x1FC);
    }
}

/// The runebook, which the items crate owns.
#[derive(Default)]
pub struct RecallHooks {
    /// `Runebook.CurCharges`.
    pub charges: Option<fn(&World, ItemId) -> i32>,
    pub add_charges: Option<fn(&mut World, ItemId, i32)>,
}

fn runebook_charges(w: &World, book: ItemId) -> i32 {
    w.resources
        .get::<RecallHooks>()
        .and_then(|h| h.charges)
        .map_or(i32::MAX, |f| f(w, book))
}

fn add_runebook_charges(w: &mut World, book: ItemId, delta: i32) {
    if let Some(f) = w.resources.get::<RecallHooks>().and_then(|h| h.add_charges) {
        f(w, book, delta);
    }
}

/// `GetCastSkills`: no skill under Wraith Form, or recalling with a runebook charge from Samurai Empire on.
fn get_cast_skills(w: &mut World, sp: SpellId) -> (f64, f64) {
    let caster = spell::caster(w, sp);
    if caster.is_some_and(|c| helper::under_transformation_of(w, c, WRAITH_FORM)) {
        (0.0, 0.0)
    } else if core::se(w) && state(w, sp).book.is_some() {
        // recall using Runebook charge
        (0.0, 0.0)
    } else {
        // base.GetCastSkills
        (MAGERY_SPELL.get_cast_skills)(w, sp)
    }
}

/// `OnCast()`.
fn on_cast(w: &mut World, sp: SpellId) {
    match state(w, sp).entry {
        None => begin_recall_target(w, sp, true),
        Some((loc, map)) => {
            effect(w, sp, loc, map, true);
            spell::finish_sequence(w, sp);
        }
    }
}

/// `CheckCast()`.
fn check_cast(w: &mut World, sp: SpellId) -> bool {
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
        mm::send_localized_message(w, caster, 1005564, "", 0x22); // Wouldst thou flee during the heat of battle??
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
