//! BaseAI: the decision loop, actions, target acquisition, fleeing.

use muo_primitives::{Direction, MessageType, Point3D, TimeSpan, utility};
use muo_server::ids::MobileId;
use muo_server::mobile::AccessLevel;
use muo_server::mobile::part1::props_a;
use muo_server::mobile::part2::{effects, messages};
use muo_server::speech_event::SpeechEventArgs;
use muo_server::world::World;

use super::{ActionType, AiClass, ai, ai_mut, movement, set_action, timer};
use crate::abilities::{self, MonsterAbilityTrigger};
use crate::creature::data::{cd, cd_mut, read};
use crate::creature::enums::*;
use crate::creature::misc::distance_sqrt;
use crate::creature::vtable::ctype;
use crate::hooks::rnd;
use crate::mx;

// How far a guarding pet may drift from its master before it closes the gap.
pub const GUARD_RANGE: i32 = 3;
const GUARD_GRACE_DURATION: i64 = 10_000;
const LKP_FRESH_DURATION: i64 = 30_000;
const INVESTIGATE_DURATION: i64 = 15_000;

pub static BASE_AI: AiClass = AiClass {
    name: "BaseAI",
    flee_health_threshold: 0.0,
    flee_chance: 0.0,
    backoff_chance: 0.0,
    think,
    obey: super::orders::obey,
    issue_order: super::orders::issue_order,
    do_action_wander,
    do_action_combat,
    do_action_guard,
    do_action_flee,
    do_action_interact: |_, _| true,
    do_action_backoff,
    on_aggressive_action,
    handles_on_speech: super::speech::handles_on_speech,
    on_speech: super::speech::on_speech,
    can_detect_hidden: |w, id| {
        muo_server::skills::value(w, id, muo_server::skills::SkillName::DetectHidden) > 0.0
    },
    detect_hidden,
    on_at_way_point: |_, _| true,
    on_teleported,
};

/// The tail of `new BaseAI(m)`: the timer node, activation when the sector is live, `Action = Wander`.
pub fn construct(w: &mut World, id: MobileId) {
    timer::create(w, id);
    let sensitive = ctype(w, id).is_some_and(|c| (c.player_range_sensitive)(w, id));
    let loading = w.state == muo_server::world::WorldState::Loading;
    let (map, active) = (
        mx::map(w, id),
        crate::creature::overrides::sector_active(w, id),
    );
    if !sensitive
        || !loading && map.is_some_and(|m| m != muo_server::map::MapId::INTERNAL) && active
    {
        timer::activate(w, id);
    }
    if let Some(a) = ai_mut(w, id) {
        a.action = ActionType::Wander;
    }
}

pub fn debug_say(w: &mut World, id: MobileId, message: &str) {
    debug_say_cooldown(w, id, message, 5000);
}

pub fn debug_say_cooldown(w: &mut World, id: MobileId, message: &str, cooldown_ms: i64) {
    if !read(w, id, false, |d| d.debug) {
        return;
    }
    let tick = w.tick_count;
    let ready = ai(w, id).is_some_and(|a| a.next_debug_message - tick <= 0);
    if ready {
        messages::public_overhead_message(
            w,
            id,
            MessageType::REGULAR,
            41,
            false,
            message,
            false,
            AccessLevel::Player,
        );
        if let Some(a) = ai_mut(w, id) {
            a.next_debug_message = tick + cooldown_ms;
        }
    }
}

fn name_of(w: &World, id: MobileId) -> String {
    muo_server::mobile::base::name(w, id).unwrap_or_default()
}

/// `OnActionChanged`: a change of course invalidates between-think movement continuation.
pub fn on_action_changed(w: &mut World, id: MobileId) {
    movement::clear_move_intent(w, id);
    let action = super::action(w, id).unwrap_or_default();
    match action {
        ActionType::Wander => {
            edit(w, id, |d| d.focus_mob = None);
            mx::set_warmode(w, id, false);
            mx::set_combatant(w, id, None);
        }
        ActionType::Combat => mx::set_warmode(w, id, true),
        ActionType::Guard => {
            mx::set_warmode(w, id, true);
            mx::set_combatant(w, id, None);
            // Investigate a fresh last-seen position that is not already in view; the guard grace period begins
            // once the investigation ends.
            let tick = w.tick_count;
            let (target, loc, expire) = ai(w, id).map_or((None, Point3D::ZERO, 0), |a| {
                (a.lkp_target, a.lkp_location, a.lkp_expire_tick)
            });
            let rp = read(w, id, 0, |d| d.range_perception);
            let investigating = target.is_some()
                && tick - expire < 0
                && !(mx::in_range_point(w, id, loc, 1)
                    || in_los_point(w, id, loc) && mx::in_range_point(w, id, loc, rp));
            if let Some(a) = ai_mut(w, id) {
                a.investigating = investigating;
                a.investigate_stop_tick = tick + INVESTIGATE_DURATION;
                a.guard_stop_tick = tick + GUARD_GRACE_DURATION;
                a.lkp_goal_set = false;
            }
        }
        ActionType::Flee => {
            edit(w, id, |d| d.focus_mob = None);
            mx::set_warmode(w, id, true);
        }
        ActionType::Interact | ActionType::Backoff => mx::set_warmode(w, id, false),
    }
}

fn edit(w: &mut World, id: MobileId, f: impl FnOnce(&mut crate::creature::CreatureData)) {
    crate::creature::edit(w, id, f);
}

fn in_los_point(w: &mut World, id: MobileId, p: Point3D) -> bool {
    muo_server::mobile::part1::combat::in_los_point(w, id, p)
}

// ---- Think -----------------------------------------------------------------------------

pub fn think(w: &mut World, id: MobileId) -> bool {
    if mx::deleted(w, id) || mx::map(w, id).is_none() {
        return false;
    }
    if check_flee(w, id) {
        return true;
    }
    let action = super::action(w, id).unwrap_or_default();
    if action == ActionType::Combat {
        update_last_known_location(w, id);
    } else if matches!(action, ActionType::Wander | ActionType::Guard) {
        try_reengage_last_known(w, id);
    }
    let class = ai(w, id).expect("ai").class;
    let c = ctype(w, id).expect("creature");
    // `Action` is re-read: the engage above can change it.
    match super::action(w, id).unwrap_or_default() {
        ActionType::Wander => {
            (c.on_action_wander)(w, id);
            (class.do_action_wander)(w, id)
        }
        ActionType::Combat => {
            (c.on_action_combat)(w, id);
            (class.do_action_combat)(w, id)
        }
        ActionType::Guard => {
            (c.on_action_guard)(w, id);
            (class.do_action_guard)(w, id)
        }
        ActionType::Flee => {
            (c.on_action_flee)(w, id);
            (class.do_action_flee)(w, id)
        }
        ActionType::Interact => {
            (c.on_action_interact)(w, id);
            (class.do_action_interact)(w, id)
        }
        ActionType::Backoff => {
            (c.on_action_backoff)(w, id);
            (class.do_action_backoff)(w, id)
        }
    }
}

// ---- retaliation policy -------------------------------------------------------------------------------

/// OnAggressiveAction
pub fn on_aggressive_action(w: &mut World, id: MobileId, aggressor: MobileId) -> bool {
    if crate::creature::overrides::refuses_guard_target(w, id, aggressor) {
        debug_say(w, id, "Friendly fire; I keep guarding.");
        return false;
    }
    // Only a creature somebody can command has been told anything; wild creatures rest on None.
    let c = ctype(w, id).expect("creature");
    let told_to_stand_down = crate::creature::control::control_master(w, id).is_some()
        && (c.commandable)(w, id)
        && super::orders::is_stand_down_order(read(w, id, OrderType::None, |d| d.control_order));
    if (c.stands_down_on_command)(w, id) && told_to_stand_down {
        debug_say(
            w,
            id,
            "I'm being attacked but my master told me not to fight.",
        );
        mx::set_warmode(w, id, false);
        return false;
    }
    prefer_closer_aggressor(w, id, aggressor);
    let changing = muo_server::mobile::part1::props_a::changing_combatant(w, aggressor);
    if changing && told_to_stand_down {
        // With stand-down off, a resting pet answers a direct attack with a full Attack order.
        crate::creature::control::issue_order(w, id, OrderType::Attack, None, Some(aggressor));
    } else if mx::combatant(w, id).is_none() && !read(w, id, false, |d| d.bard_pacified) {
        mx::set_warmode(w, id, true);
        mx::set_combatant(w, id, Some(aggressor));
    }
    true
}

/// A visible aggressor closer than the current combatant takes over.
pub fn prefer_closer_aggressor(w: &mut World, id: MobileId, aggressor: MobileId) {
    if w.mobile(aggressor).is_some_and(|m| m.hidden) {
        return;
    }
    if let Some(current) = mx::combatant(w, id)
        && current != aggressor
        && distance_sqrt(w, id, aggressor) < distance_sqrt(w, id, current)
    {
        mx::set_combatant(w, id, Some(aggressor));
    }
}

// ---- last-known-position tracking -----------------------------------------------------------------------

fn update_last_known_location(w: &mut World, id: MobileId) {
    let Some(c) = mx::combatant(w, id) else {
        return;
    };
    let can_see = w
        .mobile(id)
        .map(|m| m.ty)
        .is_some_and(|m| (m.can_see_mobile)(w, id, c));
    if !mx::deleted(w, c) && mx::map(w, c) == mx::map(w, id) && can_see && mx::in_los(w, id, c) {
        let tick = w.tick_count;
        let loc = mx::location(w, c);
        if let Some(a) = ai_mut(w, id) {
            a.lkp_target = Some(c);
            a.lkp_location = loc;
            a.lkp_expire_tick = tick + LKP_FRESH_DURATION;
        }
    }
}

fn clear_last_known(w: &mut World, id: MobileId) {
    if let Some(a) = ai_mut(w, id) {
        a.lkp_target = None;
        a.lkp_goal_set = false;
        a.investigating = false;
    }
}

/// Re-engages the last-seen target when it returns to view within perception range, bypassing the reacquire
/// throttle.
fn try_reengage_last_known(w: &mut World, id: MobileId) -> bool {
    let Some(target) = ai(w, id).and_then(|a| a.lkp_target) else {
        return false;
    };
    let expire = ai(w, id).map_or(0, |a| a.lkp_expire_tick);
    let dead_pet = read(w, target, false, |d| d.is_dead_pet);
    if mx::deleted(w, target)
        || !mx::alive(w, target)
        || mx::map(w, target) != mx::map(w, id)
        || dead_pet
        || w.tick_count - expire >= 0
    {
        clear_last_known(w, id);
        return false;
    }
    let (controlled, pacified, provoked, mode, rp) =
        read(w, id, (false, false, false, FightMode::None, 0), |d| {
            (
                d.controlled,
                d.bard_pacified,
                d.bard_provoked,
                d.fight_mode,
                d.range_perception,
            )
        });
    if controlled || pacified || provoked || mode == FightMode::None {
        return false;
    }
    let can_see = w
        .mobile(id)
        .map(|m| m.ty)
        .is_some_and(|m| (m.can_see_mobile)(w, id, target));
    if !mx::in_range(w, id, target, rp)
        || !can_see
        || !mx::in_los(w, id, target)
        || !mx::can_be_harmful_quiet(w, id, target)
    {
        return false;
    }
    debug_say(w, id, "There you are!");
    mx::set_combatant(w, id, Some(target));
    edit(w, id, |d| d.focus_mob = None);
    set_action(w, id, ActionType::Combat);
    true
}

fn investigate_last_known(w: &mut World, id: MobileId) -> bool {
    let Some((target, loc, stop)) =
        ai(w, id).map(|a| (a.lkp_target, a.lkp_location, a.investigate_stop_tick))
    else {
        return false;
    };
    if target.is_none() || w.tick_count - stop >= 0 {
        return false;
    }
    let rp = read(w, id, 0, |d| d.range_perception);
    if mx::in_range_point(w, id, loc, 1)
        || in_los_point(w, id, loc) && mx::in_range_point(w, id, loc, rp)
    {
        debug_say(w, id, "They truly disappeared...");
        return false;
    }
    if let Some(a) = ai_mut(w, id) {
        a.lkp_goal_set = true;
    }
    movement::move_to_point(w, id, loc, 1)
}

// ---- actions ---------------------------------------------------------------------------------------------

/// DoActionWander
pub fn do_action_wander(w: &mut World, id: MobileId) -> bool {
    if check_herding(w, id) {
        let n = read(w, id, None, |d| d.control_target)
            .map(|t| name_of(w, t))
            .unwrap_or_else(|| "Unknown".into());
        debug_say(w, id, &format!("I am being herded by {n}."));
    } else if should_backoff(w, id) {
        debug_say(w, id, "Detected threat, backing off!");
        set_action(w, id, ActionType::Backoff);
        return true;
    } else if read(w, id, None, |d| d.current_way_point).is_some() {
        handle_way_point(w, id);
    } else if ctype(w, id).is_some_and(|c| (c.is_animated_dead)(w, id)) {
        follow_master(w, id);
        if movement::check_move(w, id) && movement::can_move_now(w, id) && !check_idle(w, id) {
            movement::walk_random_in_home(w, id, 3, 2, 1);
        }
    } else if movement::check_move(w, id) && movement::can_move_now(w, id) && !check_idle(w, id) {
        movement::walk_random_in_home(w, id, 3, 2, 1);
    }
    true
}

fn check_idle(w: &mut World, id: MobileId) -> bool {
    (ctype(w, id).expect("creature").check_idle)(w, id)
}

fn handle_way_point(w: &mut World, id: MobileId) {
    let Some(point) = read(w, id, None, |d| d.current_way_point) else {
        return;
    };
    let (ploc, pmap, pdeleted) = match w.item(point) {
        Some(i) => (i.location, i.map, i.deleted()),
        None => return,
    };
    let loc = mx::location(w, id);
    let parentless = w.item(point).is_some_and(|i| i.parent.is_none());
    if (ploc.x != loc.x || ploc.y != loc.y) && pmap == mx::map(w, id) && parentless && !pdeleted {
        debug_say(
            w,
            id,
            &format!("Moving towards waypoint {}, {}.", ploc.x, ploc.y),
        );
        let dir = effects::get_direction_to(w, id, ploc.x, ploc.y, false);
        movement::do_move(w, id, dir, false);
    } else if (ai(w, id).expect("ai").class.on_at_way_point)(w, id) {
        debug_say(
            w,
            id,
            &format!("I have reached waypoint {}, {}.", ploc.x, ploc.y),
        );
        if let Some(f) = crate::hooks::world_hooks(w).and_then(|h| h.waypoint_next) {
            let next = f(w, point);
            edit(w, id, |d| d.current_way_point = next);
        }
    }
}

fn follow_master(w: &mut World, id: MobileId) {
    let Some(master) = crate::creature::control::summon_master(w, id) else {
        return;
    };
    let rp = read(w, id, 0, |d| d.range_perception);
    if mx::map(w, master) == mx::map(w, id) && mx::in_range(w, master, id, rp) {
        movement::move_to(w, id, master, 1);
    }
}

/// DoActionCombat
pub fn do_action_combat(w: &mut World, id: MobileId) -> bool {
    if w.core_aos() && check_herding(w, id) {
        let n = read(w, id, None, |d| d.control_target)
            .map(|t| name_of(w, t))
            .unwrap_or_else(|| "Unknown".into());
        debug_say(w, id, &format!("I am being herded by {n}."));
        return true;
    }
    let combatant = mx::combatant(w, id);
    if !is_valid_combatant(w, id, combatant) {
        debug_say(w, id, "My combatant is missing. Wandering...");
        edit(w, id, |d| d.focus_mob = None);
        mx::set_warmode(w, id, false);
        mx::set_combatant(w, id, None);
        set_action(w, id, ActionType::Wander);
        return true;
    }
    let c = combatant.expect("valid");
    if abilities::trigger_ability(w, id, MonsterAbilityTrigger::COMBAT_ACTION, Some(c)) {
        debug_say(w, id, &format!("I used my abilities on {}!", name_of(w, c)));
    }
    true
}

pub fn is_valid_combatant(w: &mut World, id: MobileId, combatant: Option<MobileId>) -> bool {
    is_valid_focus_mob(w, id, combatant)
        && combatant.is_some_and(|c| mx::in_los(w, id, c) || is_hostile(w, id, c))
}

pub fn is_valid_focus_mob(w: &mut World, id: MobileId, focus: Option<MobileId>) -> bool {
    let Some(f) = focus else { return false };
    let Some(m) = w.mobile(f) else { return false };
    let (deleted, map, fty, access) = (m.deleted, m.map, m.ty, m.access_level);
    let rp = read(w, id, 0, |d| d.range_perception);
    !deleted
        && map == mx::map(w, id)
        && (fty.alive)(w, f)
        && !read(w, f, false, |d| d.is_dead_pet)
        && access == AccessLevel::Player
        && w.mobile(id)
            .map(|x| x.ty)
            .is_some_and(|x| (x.can_see_mobile)(w, id, f))
        && mx::in_range(w, id, f, rp)
}

/// DoActionGuard
pub fn do_action_guard(w: &mut World, id: MobileId) -> bool {
    if ai(w, id).is_some_and(|a| a.investigating) {
        if investigate_last_known(w, id) {
            return true;
        }
        let tick = w.tick_count;
        if let Some(a) = ai_mut(w, id) {
            a.investigating = false;
            a.guard_stop_tick = tick + GUARD_GRACE_DURATION;
        }
    }
    let stop = ai(w, id).map_or(0, |a| a.guard_stop_tick);
    if w.tick_count - stop < 0 {
        debug_say(w, id, "I am on guard.");
        if rnd::below(w, 8) == 0 {
            let d = Direction::from_bits_retain(rnd::below(w, 8) as u8);
            muo_server::mobile::part1::props_b::set_direction(w, id, d);
        }
        return true;
    }
    debug_say(w, id, "I stopped being on guard. Going home...");
    set_action(w, id, ActionType::Wander);
    true
}

/// DoActionFlee
pub fn do_action_flee(w: &mut World, id: MobileId) -> bool {
    let from = mx::combatant(w, id);
    if !is_valid_combatant(w, id, from) {
        debug_say(w, id, "Combatant is missing.");
        movement::walk_random_in_home(w, id, 3, 2, 1);
        return true;
    }
    debug_say(w, id, "I am fleeing!");
    let from = from.expect("valid");
    let dir = mx::get_direction_to(w, from, id);
    movement::do_move(w, id, dir, false);
    true
}

// ---- herding ---------------------------------------------------------------------------------------------

/// `HandleSpecialHerdingCase` reaches the Dark Tides quest (`muo_engines::quests`).
#[derive(Default)]
pub struct HerdingHooks {
    /// `pm.Quest is DarkTidesQuest` fetching the scroll of Abraxus: `(familiar, master)`.
    pub fetch_abraxus: Option<fn(&mut World, MobileId, MobileId)>,
}

/// CheckHerding
pub fn check_herding(w: &mut World, id: MobileId) -> bool {
    let Some(target) = read(w, id, None, |d| d.target_location) else {
        if let Some(a) = ai_mut(w, id) {
            a.herd_goal = None;
        }
        return false;
    };
    let loc = mx::location(w, id);
    let distance = utility::distance_xy(loc.x, loc.y, target.x, target.y);
    if (1.0..=15.0).contains(&distance) {
        let cached = ai(w, id).and_then(|a| a.herd_goal);
        if cached.is_none_or(|g| g.x != target.x || g.y != target.y) {
            // A cached goal keeps the path persistent across ticks.
            let z = mx::map(w, id).map_or(loc.z, |m| w.maps.get_average_z(m, target.x, target.y));
            if let Some(a) = ai_mut(w, id) {
                a.herd_goal = Some(Point3D::new(target.x, target.y, z));
            }
        }
        let goal = ai(w, id).and_then(|a| a.herd_goal).expect("set above");
        movement::move_to_point(w, id, goal, 0);
        return true;
    }
    // `distance < 1 && IsSpecialHerdingCase(target)`: a HordeMinionFamiliar at (1076, 450)
    if distance < 1.0
        && target.x == 1076
        && target.y == 450
        && crate::factory::mobile_is(w, id, "Server.Mobiles.HordeMinionFamiliar")
        && let Some(master) = crate::creature::control::control_master(w, id)
            .filter(|&m| w.mobile(m).is_some_and(|x| x.player))
        && let Some(f) = w
            .resources
            .get::<HerdingHooks>()
            .and_then(|h| h.fetch_abraxus)
    {
        f(w, id, master);
    }
    edit(w, id, |d| d.target_location = None);
    if let Some(a) = ai_mut(w, id) {
        a.herd_goal = None;
    }
    false
}

// ---- bard effects ----------------------------------------------------------------------------------------

pub fn do_bard_pacified(w: &mut World, id: MobileId) {
    let end = read(w, id, w.now, |d| d.bard_end_time);
    if w.now < end {
        debug_say(w, id, "I am pacified. Can not fight.");
        mx::set_warmode(w, id, false);
        mx::set_combatant(w, id, None);
    } else {
        debug_say(w, id, "I am free from pacification.");
        edit(w, id, |d| d.bard_pacified = false);
    }
}

pub fn do_bard_provoked(w: &mut World, id: MobileId) {
    let (end, master, target, rp) = read(w, id, (w.now, None, None, 0), |d| {
        (
            d.bard_end_time,
            d.bard_master,
            d.bard_target,
            d.range_perception,
        )
    });
    let lost = |w: &World, m: Option<MobileId>| {
        m.is_none_or(|m| {
            mx::deleted(w, m)
                || mx::map(w, m) != mx::map(w, id)
                || distance_sqrt(w, id, m) > rp as f64
        })
    };
    let reset = |w: &mut World, msg: &str| {
        debug_say(w, id, msg);
        edit(w, id, |d| {
            d.bard_provoked = false;
            d.bard_master = None;
            d.bard_target = None;
        });
        mx::set_warmode(w, id, false);
        mx::set_combatant(w, id, None);
    };
    if w.now >= end && lost(w, master) {
        reset(w, "Provoker missing.");
    } else if lost(w, target) {
        reset(w, "Provoke target missing.");
    } else {
        mx::set_combatant(w, id, target);
        set_action(w, id, ActionType::Combat);
    }
}

// ---- flee / backoff ---------------------------------------------------------------------------------------

/// CheckFlee
pub fn check_flee(w: &mut World, id: MobileId) -> bool {
    let c = ctype(w, id).expect("creature");
    if (c.check_flee)(w, id) {
        if mx::combatant(w, id).is_none() {
            set_action(w, id, ActionType::Wander);
            return false;
        }
        set_action(w, id, ActionType::Flee);
        return true;
    }
    if start_flee(w, id) {
        let secs = rnd::from(w, 10, 30);
        (c.begin_flee)(w, id, TimeSpan::from_secs(secs as i64));
        set_action(w, id, ActionType::Flee);
        return true;
    }
    false
}

fn start_flee(w: &mut World, id: MobileId) -> bool {
    let c = ctype(w, id).expect("creature");
    if !(c.can_flee)(w, id) {
        return false;
    }
    let class = ai(w, id).expect("ai").class;
    let hmax = mx::hits_max(w, id);
    let hit_percent = mx::hits(w, id) as f64 / hmax as f64;
    if hit_percent > class.flee_health_threshold {
        return false;
    }
    rnd::double(w) < class.flee_chance
}

fn should_backoff(w: &mut World, id: MobileId) -> bool {
    let class = ai(w, id).expect("ai").class;
    if mx::combatant(w, id).is_some() || class.backoff_chance <= 0.0 {
        return false;
    }
    let (rp, mode) = read(w, id, (0, FightMode::None), |d| {
        (d.range_perception, d.fight_mode)
    });
    if acquire_focus_mob(w, id, rp, mode, false, false, true) {
        return rnd::double(w) < class.backoff_chance;
    }
    false
}

/// DoActionBackoff
pub fn do_action_backoff(w: &mut World, id: MobileId) -> bool {
    let rp = read(w, id, 0, |d| d.range_perception);
    if acquire_focus_mob(w, id, rp * 2, FightMode::Closest, true, false, true) {
        let focus = read(w, id, None, |d| d.focus_mob);
        if let Some(f) = focus
            && movement::walk_mobile_range(w, id, f, 1, rp, rp * 2)
        {
            debug_say(w, id, "I backed off to safety. Wandering...");
            set_action(w, id, ActionType::Wander);
        }
    } else {
        debug_say(w, id, "Focus target missing. Wandering...");
        set_action(w, id, ActionType::Wander);
    }
    true
}

pub fn on_teleported(w: &mut World, id: MobileId) {
    debug_say(w, id, "Teleported; recalculating path...");
    if let Some(f) = w
        .resources
        .get::<super::PathHooks>()
        .and_then(|h| h.force_repath)
    {
        f(w, id);
    }
}

// ---- target acquisition ----------------------------------------------------------------------------------

fn is_hostile(w: &World, id: MobileId, from: MobileId) -> bool {
    let Some(m) = w.mobile(id) else { return false };
    mx::combatant(w, id) == Some(from)
        || mx::combatant(w, from) == Some(id)
        || m.aggressors.iter().any(|a| a.defender == from)
        || m.aggressed.iter().any(|a| a.attacker == from)
}

pub fn is_hostile_pub(w: &World, id: MobileId, from: MobileId) -> bool {
    is_hostile(w, id, from)
}

/// AcquireFocusMob
pub fn acquire_focus_mob(
    w: &mut World,
    id: MobileId,
    range: i32,
    acq: FightMode,
    player_only: bool,
    fac_friend: bool,
    fac_foe: bool,
) -> bool {
    if mx::deleted(w, id) || mx::map(w, id).is_none() {
        return false;
    }
    let (pacified, provoked, controlled) = read(w, id, (false, false, false), |d| {
        (d.bard_pacified, d.bard_provoked, d.controlled)
    });
    if pacified {
        return false;
    }
    if provoked {
        // HandleBardProvoked
        let target = read(w, id, None, |d| d.bard_target);
        if target.is_none_or(|t| mx::deleted(w, t)) {
            edit(w, id, |d| d.focus_mob = None);
        } else {
            edit(w, id, |d| d.focus_mob = target);
            return true;
        }
    }
    // A controlled creature must not fall through to wild target selection when its order has no target.
    if controlled {
        return handle_controlled(w, id);
    }
    let c = ctype(w, id).expect("creature");
    if let Some(f) = (c.constant_focus)(w, id) {
        debug_say(
            w,
            id,
            &format!("Acquired focused target: {}.", name_of(w, f)),
        );
        edit(w, id, |d| d.focus_mob = Some(f));
        return true;
    }
    if acq == FightMode::None {
        edit(w, id, |d| d.focus_mob = None);
        return false;
    }
    // HandleAggressor
    let no_allegiance = ctype(w, id).is_some_and(|c| {
        (c.faction_allegiance)(w, id).is_none() && (c.ethic_allegiance)(w, id).is_none()
    });
    if acq == FightMode::Aggressor
        && w.mobile(id)
            .is_some_and(|m| m.aggressors.is_empty() && m.aggressed.is_empty())
        && no_allegiance
    {
        edit(w, id, |d| d.focus_mob = None);
        return false;
    }
    let reacquire_delay = (c.reacquire_delay)(w, id).total_milliseconds() as i64;
    let gate_remaining = read(w, id, 0, |d| d.next_reacquire_time) - w.tick_count;
    if gate_remaining > 0 && gate_remaining <= reacquire_delay {
        edit(w, id, |d| d.focus_mob = None);
        return false;
    }
    debug_say_cooldown(w, id, "Acquiring new target...", 0);
    let acquired = acquire_new_focus_mob(w, id, range, acq, player_only, fac_friend, fac_foe);
    // Reaction time is the approach path (BaseCreature.ScheduleAcquireOnApproach), not this poll.
    let next = w.tick_count + reacquire_delay;
    edit(w, id, |d| d.next_reacquire_time = next);
    acquired
}

fn handle_controlled(w: &mut World, id: MobileId) -> bool {
    let (order, ctarget, rp) = read(w, id, (OrderType::None, None, 0), |d| {
        (d.control_order, d.control_target, d.range_perception)
    });
    if order == OrderType::Guard {
        let t = super::orders::find_guard_target(w, id);
        edit(w, id, |d| d.focus_mob = t);
        return t.is_some();
    }
    // Follow and other non-combat orders may target the owner or a friend. Only an explicit attack order supplies a
    // combat target.
    if order != OrderType::Attack {
        edit(w, id, |d| d.focus_mob = None);
        return false;
    }
    if let Some(t) = ctarget {
        let ok = !mx::deleted(w, t)
            && !w.mobile(t).is_some_and(|m| m.hidden)
            && mx::alive(w, t)
            && !w.mobile(t).is_some_and(|m| (m.ty.is_dead_bonded_pet)(w, t))
            && mx::in_range(w, id, t, rp * 2);
        if ok {
            edit(w, id, |d| d.focus_mob = Some(t));
            return true;
        }
        if Some(t) != crate::creature::control::control_master(w, id) {
            edit(w, id, |d| d.control_target = None);
        }
    }
    edit(w, id, |d| d.focus_mob = None);
    false
}

fn acquire_new_focus_mob(
    w: &mut World,
    id: MobileId,
    range: i32,
    acq: FightMode,
    player_only: bool,
    fac_friend: bool,
    fac_foe: bool,
) -> bool {
    let mut new_focus: Option<MobileId> = None;
    let mut enemy_summon: Option<MobileId> = None;
    let mut val = f64::MIN;
    let mut enemy_summon_val = f64::MIN;
    let mut summon_master_found: Option<MobileId> = None;
    let loc = mx::location(w, id);
    let map = mx::map(w, id);
    let summon_master = crate::creature::control::summon_master(w, id);
    let candidates: Vec<MobileId> =
        muo_server::map::enumerate::mobiles_in_range(w, map, loc.x, loc.y, range).collect();
    let c = ctype(w, id).expect("creature");
    for m in candidates {
        if is_invalid_target(w, id, m, player_only) {
            continue;
        }
        let is_bc = cd(w, m).is_some();
        if is_invalid_summon_target(w, id, m)
            || is_invalid_faction_target(w, id, m, fac_friend, fac_foe)
            || is_invalid_fight_mode_target(w, id, m, acq, is_bc)
        {
            continue;
        }
        // Only a summon that ignores the acquire rules gets here with its caster; anyone else comes first.
        if Some(m) == summon_master {
            if mx::in_los(w, id, m) {
                summon_master_found = Some(m);
            }
            continue;
        }
        let their = (c.get_fight_mode_ranking)(w, id, m, acq, player_only);
        if their > val && mx::in_los(w, id, m) {
            new_focus = Some(m);
            val = their;
        } else if w.core_aos()
            && their > enemy_summon_val
            // the cheap test before line of sight: every candidate that is not the best so far reaches this branch
            && cd(w, m).is_some_and(|b| b.summoned && !b.controlled)
            && mx::in_los(w, id, m)
        {
            enemy_summon = Some(m);
            enemy_summon_val = their;
        }
    }
    let focus = new_focus.or(enemy_summon).or(summon_master_found);
    edit(w, id, |d| d.focus_mob = focus);
    focus.is_some()
}

fn is_invalid_target(w: &mut World, id: MobileId, m: MobileId, player_only: bool) -> bool {
    let Some(x) = w.mobile(m) else { return true };
    let (deleted, blessed, xty, access, player) =
        (x.deleted, x.blessed, x.ty, x.access_level, x.player);
    deleted
        || blessed
        || m == id
        || crate::factory::mobile_is(w, m, "Server.Mobiles.BaseFamiliar")
        || !(xty.alive)(w, m)
        || (xty.is_dead_bonded_pet)(w, m)
        || access > AccessLevel::Player
        || player_only && !player
        || !w
            .mobile(id)
            .map(|me| me.ty)
            .is_some_and(|me| (me.can_see_mobile)(w, id, m))
}

fn is_invalid_summon_target(w: &mut World, id: MobileId, m: MobileId) -> bool {
    use crate::creature::control::summon_master;
    // A summon whose caster was deleted before a reload comes back with no master.
    if w.core_aos() {
        let sm = summon_master(w, m);
        if sm == Some(id) || sm.is_some_and(|s| !mx::player(w, s) && is_hostile(w, id, s)) {
            return true;
        }
    }
    let Some(master) = summon_master(w, id) else {
        return false;
    };
    let animated = ctype(w, id).is_some_and(|c| (c.is_animated_dead)(w, id));
    if animated {
        let m_animated = ctype(w, m).is_some_and(|c| (c.is_animated_dead)(w, m));
        let m_controlled = read(w, m, false, |d| d.controlled);
        if mx::player(w, m) || m_animated || m_controlled {
            return true;
        }
    }
    let follows = ctype(w, id).is_some_and(|c| (c.follows_acquire_rules)(w, id));
    let valid_indirect = crate::hooks::hooks(w)
        .and_then(|h| h.valid_indirect_target)
        .is_none_or(|f| f(w, master, m));
    follows && (m == master || !valid_indirect)
}

fn is_invalid_faction_target(
    w: &mut World,
    id: MobileId,
    m: MobileId,
    fac_friend: bool,
    fac_foe: bool,
) -> bool {
    let c = ctype(w, id).expect("creature");
    fac_friend && !(c.is_friend)(w, id, m)
        || fac_foe
            && (!(c.is_enemy)(w, id, m) || !fac_friend && !mx::can_be_harmful_quiet(w, id, m))
}

fn is_invalid_fight_mode_target(
    w: &mut World,
    id: MobileId,
    m: MobileId,
    acq: FightMode,
    is_bc: bool,
) -> bool {
    if !matches!(acq, FightMode::Aggressor | FightMode::Evil) {
        return false;
    }
    let c = ctype(w, id).expect("creature");
    let valid = is_hostile(w, id, m)
        || (c.get_faction_allegiance)(w, id, m) == Allegiance::Enemy
        || (c.get_ethic_allegiance)(w, id, m) == Allegiance::Enemy;
    // Valid if FightMode is Evil and the target's karma is negative
    let karma = if is_bc {
        crate::creature::control::get_master(w, m).map_or_else(
            || w.mobile(m).map_or(0, |x| x.karma),
            |g| w.mobile(g).map_or(0, |x| x.karma),
        )
    } else {
        w.mobile(m).map_or(0, |x| x.karma)
    };
    !valid && (acq != FightMode::Evil || karma >= 0)
}

// ---- hidden detection ------------------------------------------------------------------------------------

/// DetectHidden
pub fn detect_hidden(w: &mut World, id: MobileId) {
    use muo_server::skills::{self, SkillName};
    if mx::deleted(w, id) || mx::map(w, id).is_none() {
        return;
    }
    let class = ai(w, id).expect("ai").class;
    if !(class.can_detect_hidden)(w, id) {
        return;
    }
    debug_say(w, id, "Checking for hidden entities...");
    let src = skills::value(w, id, SkillName::DetectHidden);
    if src <= 0.0 {
        return;
    }
    let rp = read(w, id, 0, |d| d.range_perception);
    for trg in mx::mobiles_in_range(w, id, rp) {
        let ok = trg != id
            && w.mobile(trg)
                .is_some_and(|m| m.player && m.hidden && m.access_level == AccessLevel::Player)
            && mx::alive(w, trg)
            && mx::in_los(w, id, trg);
        if ok {
            debug_say(w, id, &format!("Trying to detect: {}", name_of(w, trg)));
            let hiding = skills::value(w, trg, SkillName::Hiding) / 2.9;
            let stealth = skills::value(w, trg, SkillName::Stealth) / 1.8;
            let chance = (src / 10.0).max(src / 1.2 - hiding.min(stealth)) / 100.0;
            if chance > rnd::double(w) {
                if let Some(ty) = w.mobile(trg).map(|m| m.ty) {
                    (ty.revealing_action)(w, trg);
                }
                mx::msg(w, trg, 500814); // You have been revealed!
            }
        }
    }
}

/// Deactivate
pub fn deactivate(w: &mut World, id: MobileId) {
    let map = mx::map(w, id);
    let controlled = read(w, id, false, |d| d.controlled);
    if map.is_none()
        || map == Some(muo_server::map::MapId::INTERNAL)
        || !controlled && !crate::creature::overrides::sector_active(w, id)
    {
        timer::pause(w, id);
    }
    // ShouldReturnToHome(Mobile.Spawner)
    use muo_server::mobile::part2::deps as sd;
    let spawner = w.mobile(id).and_then(|m| m.spawner);
    if let Some(sp) = spawner
        && sd::spawner_return_on_deactivate(w, sp)
        && !controlled
        && !sd::spawner_is_in_spawn_bounds(w, sp, mx::location(w, id))
    {
        // Timer.StartTimer(ReturnToHome)
        muo_server::timer::start(w, TimeSpan::ZERO, TimeSpan::ZERO, 1, move |w| {
            let Some(sp) = w.mobile(id).and_then(|m| m.spawner) else {
                return;
            };
            let spawner_map = w
                .item(muo_server::ids::ItemId(muo_primitives::Serial::new(sp.0)))
                .and_then(|i| i.map);
            if let Some(map) = spawner_map {
                let loc = sd::spawner_get_spawn_position(w, sp, id, map);
                if loc != Point3D::ZERO {
                    muo_server::mobile::base::move_to_world(w, id, loc, map);
                }
            }
            if !timer::running(w, id) {
                timer::activate(w, id);
            }
        });
    }
}

#[allow(dead_code)]
fn _unused(w: &mut World, id: MobileId, e: &SpeechEventArgs) -> bool {
    let _ = (props_a::core_aos(w), cd_mut(w, id).is_some(), e);
    false
}
