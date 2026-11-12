//! The Gate Travel spell. `GateTravelMoongate` derives from `Moongate` and is in muo_items
//! (`misc/moongates.rs`), which sets `SpellHooks::new_gate_travel_moongate` and `link_gate_travel_moongates`.

use muo_content::core;
use muo_content::spells::helper::{self, TravelCheck};
use muo_content::spells::hooks::hooks;
use muo_content::spells::magery::{MAGERY_SPELL, SEVENTH_CIRCLE};
use muo_content::spells::spell::{self, SpellId, SpellType};
use muo_content::spells::targeting::{RecallType, begin_recall_target};
use muo_content::spells::{SpellInfo, TravelCheckType, reagent};
use muo_primitives::Point3D;
use muo_server::World;
use muo_server::ids::{ItemId, MobileId};
use muo_server::map::MapId;
use muo_server::mobile::base as mb;
use muo_server::mobile::part2::messages as mm;

use crate::common::{is_player_mobile, item_is};
use crate::fourth::recall::RecallState;

pub static GATE_TRAVEL_INTERFACE: RecallType = RecallType { effect };

/// `GateTravelSpell`: Vas Rel Por.
pub static GATE_TRAVEL_SPELL: SpellType = SpellType {
    full_name: "Server.Spells.Seventh.GateTravelSpell",
    parent: Some(&MAGERY_SPELL),
    target_range: Some(muo_content::spells::targeting::target_range),
    info: SpellInfo::new(
        "Gate Travel",
        "Vas Rel Por",
        &[
            reagent::BLACK_PEARL,
            reagent::MANDRAKE_ROOT,
            reagent::SULFUROUS_ASH,
        ],
    )
    .action(263)
    .hand_effect(9032),
    on_cast,
    check_cast,
    family: Some(&SEVENTH_CIRCLE),
    interface: Some(&GATE_TRAVEL_INTERFACE),
    ..MAGERY_SPELL
};

/// `new GateTravelSpell(caster, entry, scroll)`.
pub fn new_gate_travel_spell(
    w: &mut World,
    caster: MobileId,
    entry: Option<(Point3D, Option<MapId>)>,
    scroll: Option<ItemId>,
) -> SpellId {
    let id = spell::new_spell(w, &GATE_TRAVEL_SPELL, caster, scroll);
    if let Some(s) = spell::spells(w).get_mut(id) {
        s.ext = muo_server::Ext::new(RecallState { entry, book: None });
    }
    id
}

fn entry(w: &World, sp: SpellId) -> Option<(Point3D, Option<MapId>)> {
    spell::spell(w, sp)
        .and_then(|s| s.ext.get::<RecallState>().cloned())
        .and_then(|s| s.entry)
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
    let (cloc, cmap) = w
        .mobile(caster)
        .map(|c| (c.location, c.map))
        .unwrap_or_default();

    if sigil_exists_on(w, caster) {
        mm::send_localized_message(w, caster, 1061632, "", 0x3B2); // You can't do that while carrying the sigil.
        return;
    }
    let Some(map) = map.filter(|&m| core::aos(w) || cmap == Some(m)) else {
        mm::send_localized_message(w, caster, 1005570, "", 0x3B2); // You can not gate to another facet.
        return;
    };

    let from_check = helper::check_travel(w, caster, TravelCheckType::GateFrom);
    if !from_check.allowed {
        send_failure(w, caster, &from_check);
        return;
    }
    let to_check =
        helper::check_travel_at(w, Some(caster), Some(map), loc, TravelCheckType::GateTo);
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
        mm::send_localized_message(w, caster, 1005564, "", 0x22); // Wouldst thou flee during the heat of battle??
    } else if !muo_server::map::fit::can_spawn_mobile_at(w, map, loc.x, loc.y, loc.z)
        || (check_multi && helper::check_multi(w, loc, Some(map), true, 0))
    {
        mm::send_localized_message(w, caster, 501942, "", 0x3B2); // That location is blocked.
    }
    // SE restricted stacking gates
    else if core::se(w) && (gate_exists_at(w, Some(map), loc) || gate_exists_at(w, cmap, cloc)) {
        mm::send_localized_message(w, caster, 1071242, "", 0x3B2); // There is already a gate there.
    } else if spell::check_sequence(w, sp) {
        mm::send_localized_message(w, caster, 501024, "", 0x3B2); // You open a magical gate to another location
        muo_server::effects::play_sound(w, cloc, cmap, 0x20E);

        let make = hooks(w).and_then(|h| h.new_gate_travel_moongate);
        let link = hooks(w).and_then(|h| h.link_gate_travel_moongates);
        if let Some(make) = make {
            let first = make(w, loc, Some(map), cloc, cmap);
            muo_server::effects::play_sound(w, loc, Some(map), 0x20E);
            let second = make(w, cloc, cmap, loc, Some(map));
            if let (Some(link), Some(a), Some(b)) = (link, first, second) {
                link(w, a, b);
            }
        }
    }
}

fn on_cast(w: &mut World, sp: SpellId) {
    match entry(w, sp) {
        None => begin_recall_target(w, sp, false),
        Some((loc, map)) => {
            effect(w, sp, loc, map, true);
            spell::finish_sequence(w, sp);
        }
    }
}

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

    let c = helper::check_travel(w, caster, TravelCheckType::GateFrom);
    if !c.allowed {
        send_failure(w, caster, &c);
        return false;
    }

    true
}

/// `GateExistsAt(map, loc)`: a `Moongate` or `PublicMoongate` on the tile.
fn gate_exists_at(w: &World, map: Option<MapId>, loc: Point3D) -> bool {
    muo_server::map::enumerate::items_at(w, map, loc.x, loc.y).any(|i| {
        item_is(w, i, "Server.Items.Moongate") || item_is(w, i, "Server.Items.PublicMoongate")
    })
}
