//! `FamiliarAI`: Necromancy familiars, command-immune and glued to the caster.
//!
//! `BaseFamiliar.AssistsMaster` (false for DeathAdder and ShadowWisp) is `FamiliarFlags` in the creature's `Ext`;
//! without it a familiar assists, the base class's value.

use muo_primitives::Point3D;
use muo_server::ids::MobileId;
use muo_server::world::World;

use super::base::{BASE_AI, debug_say, prefer_closer_aggressor};
use super::movement::{clear_move_intent, move_to, reset_approach_state};
use super::{AiClass, ApproachOutcome, ai};
use crate::creature::control::control_master;
use crate::creature::data::read;
use crate::creature::enums::OrderType;
use crate::creature::misc::distance_sqrt;
use crate::hooks::rnd;
use crate::mx;

/// `BaseFamiliar.KeepUpRange`.
pub const KEEP_UP_RANGE: i32 = 10;

/// `BaseFamiliar.AssistsMaster`, kept in the creature's `Ext` by the familiar classes that override it.
pub struct FamiliarFlags {
    pub assists_master: bool,
}

pub fn assists_master(w: &World, id: MobileId) -> bool {
    w.mobile(id)
        .and_then(|m| m.ext.get::<FamiliarFlags>())
        .is_none_or(|f| f.assists_master)
}

pub static FAMILIAR_AI: AiClass = AiClass {
    name: "FamiliarAI",
    think: act,
    obey: act,
    issue_order,
    on_aggressive_action,
    can_detect_hidden: |_, _| false,
    ..BASE_AI
};

/// `LeashRange`: assist targets, and the familiar itself, stay within this of the caster.
fn leash_range(w: &World, id: MobileId) -> i32 {
    read(w, id, 0, |d| d.range_perception)
}

/// Command immunity: no issue-phase side effects; rests on Come so TeleportPets still applies.
fn issue_order(
    w: &mut World,
    id: MobileId,
    _order: OrderType,
    _previous: OrderType,
    _issuer: Option<MobileId>,
    _resuming: bool,
    _interrupted: Option<MobileId>,
) -> OrderType {
    super::prod(w, id);
    if read(w, id, false, |d| d.controlled) {
        OrderType::Come
    } else {
        OrderType::None
    }
}

/// No orders to stand down on; never fights for a hidden caster.
fn on_aggressive_action(w: &mut World, id: MobileId, aggressor: MobileId) -> bool {
    let master_hidden =
        control_master(w, id).is_some_and(|m| w.mobile(m).is_some_and(|x| x.hidden));
    if !assists_master(w, id) || w.mobile(aggressor).is_some_and(|m| m.hidden) || master_hidden {
        return false;
    }
    if mx::combatant(w, id).is_none() {
        mx::set_warmode(w, id, true);
        mx::set_combatant(w, id, Some(aggressor));
    } else {
        prefer_closer_aggressor(w, id, aggressor);
    }
    true
}

fn act(w: &mut World, id: MobileId) -> bool {
    let master = control_master(w, id);
    // Deletion is BaseFamiliar.OnThink's.
    let Some(master) = master.filter(|&m| !mx::deleted(w, m)) else {
        return true;
    };
    if mx::deleted(w, id) {
        return true;
    }
    // Left behind: stand down until TeleportPets or the unsummon.
    if mx::map(w, master) != mx::map(w, id) {
        stand_down(w, id);
        return true;
    }
    // Herding outranks combat.
    if read(w, id, false, |d| d.target_location.is_some()) {
        stand_down(w, id);
        if super::base::check_herding(w, id) {
            debug_say(w, id, "Fetching for my master.");
            return true;
        }
    }
    let master_hidden = w.mobile(master).is_some_and(|m| m.hidden);
    if assists_master(w, id) && !master_hidden && try_assist(w, id, master) {
        return true;
    }
    follow(w, id, master);
    true
}

// The caster's target; else the closest mobile in a fight with the caster's side that is still fighting one of us
// (the caster's own Combatant expires while a monster keeps hitting).
fn try_assist(w: &mut World, id: MobileId, master: MobileId) -> bool {
    let mut target = mx::combatant(w, master);
    if !is_valid_assist_target(w, id, master, target) {
        target = mx::combatant(w, id);
        if !is_valid_assist_target(w, id, master, target) || !is_fighting_us(w, id, master, target)
        {
            target = find_aggressor(w, id, master);
            if target.is_none() {
                return false;
            }
        }
    }
    let Some(target) = target else { return false };
    let leash = leash_range(w, id);
    if !mx::in_range(w, id, master, leash) {
        debug_say(w, id, "Too far from my master; returning.");
        return false;
    }
    mx::set_warmode(w, id, true);
    mx::set_combatant(w, id, Some(target));
    if mx::combatant(w, id) != Some(target) {
        return false; // setter refused it
    }
    crate::creature::speeds::set_current_speed_to_active(w, id);
    let name = muo_server::mobile::base::name(w, target).unwrap_or_default();
    debug_say(w, id, &format!("Assisting my master against {name}."));
    let range_fight = read(w, id, 1, |d| d.range_fight);
    move_to(w, id, target, range_fight);
    true
}

fn is_fighting_us(w: &World, id: MobileId, master: MobileId, target: Option<MobileId>) -> bool {
    let Some(combatant) = target.and_then(|t| mx::combatant(w, t)) else {
        return false;
    };
    combatant == id
        || combatant == master
        || (crate::creature::is_creature(w, combatant)
            && control_master(w, combatant) == Some(master))
}

/// Closest mobile in a fight with the caster's side.
fn find_aggressor(w: &mut World, id: MobileId, master: MobileId) -> Option<MobileId> {
    let mut best: Option<MobileId> = None;
    let mut best_dist = f64::MAX;
    // (owner, read the Defender of an Aggressed list)
    for (owner, defenders) in [(master, false), (id, false), (master, true)] {
        let w_ref: &World = w;
        let infos: Vec<(bool, MobileId)> = w_ref.mobile(owner).map_or_else(Vec::new, |m| {
            if defenders {
                m.aggressed
                    .iter()
                    .map(|i| (i.expired(w_ref), i.defender))
                    .collect()
            } else {
                m.aggressors
                    .iter()
                    .map(|i| (i.expired(w_ref), i.attacker))
                    .collect()
            }
        });
        for (expired, other) in infos {
            if expired {
                continue;
            }
            if Some(other) == best
                || !is_valid_assist_target(w, id, master, Some(other))
                || !is_fighting_us(w, id, master, Some(other))
            {
                continue;
            }
            let dist = distance_sqrt(w, master, other);
            if dist < best_dist {
                best = Some(other);
                best_dist = dist;
            }
        }
    }
    best
}

fn is_valid_assist_target(
    w: &mut World,
    id: MobileId,
    master: MobileId,
    target: Option<MobileId>,
) -> bool {
    let Some(t) = target else { return false };
    let Some(tm) = w.mobile(t) else { return false };
    let leash = leash_range(w, id);
    !tm.deleted
        && t != id
        && t != master
        && (tm.ty.alive)(w, t)
        && !tm.hidden
        && tm.map == mx::map(w, id)
        && !(tm.ty.is_dead_bonded_pet)(w, t)
        && tm.access_level == muo_server::mobile::AccessLevel::Player
        && mx::in_range(w, master, t, leash)
        && mx::can_be_harmful_quiet(w, id, t)
}

// Clears the move intent too, or a move-wake resumes the abandoned pursuit.
fn stand_down(w: &mut World, id: MobileId) {
    mx::set_warmode(w, id, false);
    mx::set_combatant(w, id, None);
    crate::creature::speeds::set_current_speed_to_active(w, id);
    clear_move_intent(w, id);
}

fn follow(w: &mut World, id: MobileId, master: MobileId) {
    stand_down(w, id);
    move_to(w, id, master, 1);
    try_keep_up(w, id, master);
}

// Outpaced on open ground, or given up: snap. A live detour finishes; Blocked is still counting.
fn try_keep_up(w: &mut World, id: MobileId, master: MobileId) {
    let snap = match ai(w, id).map(|a| a.last_approach) {
        Some(ApproachOutcome::GaveUp) => true,
        Some(ApproachOutcome::DirectProgress) => !mx::in_range(w, id, master, KEEP_UP_RANGE),
        _ => false,
    };
    if !snap {
        return;
    }
    let Some(loc) = try_find_landing(w, id, master) else {
        return;
    };
    debug_say(w, id, "Keeping up with my master.");
    muo_server::mobile::base::set_location(w, id, loc, true);
    reset_approach_state(w, id);
}

const LANDING_RING: [(i32, i32); 8] = [
    (0, 1),
    (1, 0),
    (0, -1),
    (-1, 0),
    (1, 1),
    (-1, 1),
    (1, -1),
    (-1, -1),
];

// An adjacent tile on the caster's floor this creature can stand on.
fn try_find_landing(w: &mut World, id: MobileId, master: MobileId) -> Option<Point3D> {
    let map = mx::map(w, master)?;
    let m = mx::location(w, master);
    let (can_swim, cant_walk) = w
        .mobile(id)
        .map_or((false, false), |x| (x.can_swim, x.cant_walk));
    let start = rnd::below(w, LANDING_RING.len() as i32) as usize; // no favoured side
    for i in 0..LANDING_RING.len() {
        let (dx, dy) = LANDING_RING[(start + i) % LANDING_RING.len()];
        let (x, y) = (m.x + dx, m.y + dy);
        if let Ok(z) = muo_server::map::fit::can_spawn_mobile(
            w,
            map,
            x,
            y,
            m.z - 5,
            m.z + 5,
            can_swim,
            cant_walk,
        ) {
            return Some(Point3D::new(x, y, z));
        }
    }
    None
}
