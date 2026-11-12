//! AI movement: steps, approach, move intent,
//! random walks. Routing around obstacles is the pathing engine's (`PathHooks`); without it a blocked approach
//! has no detour.

use muo_primitives::{Direction, Point3D, utility};
use muo_server::ids::MobileId;
use muo_server::map::{fit, los};
use muo_server::movement::Movement;
use muo_server::world::World;

use super::base::debug_say;
use super::{ApproachOutcome, Goal, MoveResult, PathHooks, ai, ai_mut};
use crate::creature::data::{cd, read};
use crate::creature::enums::*;
use crate::creature::misc::distance_sqrt;
use crate::creature::vtable::ctype;
use crate::hooks::rnd;
use crate::mx;

/// Consecutive move-eligible ticks a creature may fail to improve its best distance to a STATIONARY goal before
/// it gives up and idles.
const APPROACH_GIVE_UP_TICKS: i32 = 40;

fn edit_ai(w: &mut World, id: MobileId, f: impl FnOnce(&mut super::AiState)) {
    if let Some(a) = ai_mut(w, id) {
        f(a);
    }
}

/// BadlyHurtMoveDelay: inflates a step delay while badly hurt; damage slows steps, never decisions.
pub fn badly_hurt_move_delay(w: &World, id: MobileId, delay: f64) -> f64 {
    let hs = w.expansion() >= muo_data::expansion::Expansion::HS;
    let Some(m) = w.mobile(id) else { return delay };
    let (stat_min, stat_max) = if hs {
        (m.stam, (m.ty.stam_max)(w, id))
    } else {
        (m.hits, (m.ty.hits_max)(w, id))
    };
    let dead = read(w, id, false, |d| d.is_dead_pet);
    let c = ctype(w, id);
    let reduce = c.is_some_and(|c| (c.reduce_speed_with_damage)(w, id) || (c.is_subdued)(w, id));
    if !dead && reduce && stat_max > 0 && (stat_min as f64) < stat_max as f64 * 0.3 {
        let stat = stat_min as f64 / stat_max as f64;
        if stat < 0.1 {
            return delay + 0.15;
        }
        if stat < 0.2 {
            return delay + 0.1;
        }
        return delay + 0.05;
    }
    delay
}

/// CanMoveNow
pub fn can_move_now(w: &World, id: MobileId) -> bool {
    ai(w, id).is_some_and(|a| w.tick_count - a.next_move >= 0)
}

/// Seconds per step as the client observes it: the move clock plus the hurt inflation.
fn effective_step_delay(w: &World, id: MobileId) -> f64 {
    let step = crate::creature::speeds::current_move_speed(w, id);
    if w.core_aos() && is_following_master(w, id) {
        step
    } else {
        badly_hurt_move_delay(w, id, step)
    }
}

fn mounted_or_flying(w: &mut World, id: MobileId) -> bool {
    muo_server::mobile::part1::props_b::mounted(w, id) || w.mobile(id).is_some_and(|m| m.flying)
}

/// ShouldRun: the Running bit only selects the client's per-step interpolation.
pub fn should_run(w: &mut World, id: MobileId) -> bool {
    let mounted = mounted_or_flying(w, id);
    let (walk_foot, run_foot, walk_mount, run_mount) =
        w.resources
            .get::<Movement>()
            .map_or((400, 200, 200, 100), |m| {
                (
                    m.walk_foot_delay,
                    m.run_foot_delay,
                    m.walk_mount_delay,
                    m.run_mount_delay,
                )
            });
    let walk_delay = if mounted { walk_mount } else { walk_foot } as f64;
    let pace = effective_step_delay(w, id) * 1000.0;
    if pace >= walk_delay {
        return false;
    }
    let run_delay = if mounted { run_mount } else { run_foot } as f64;
    let last = w.mobile(id).map_or(0, |m| m.last_move_time);
    pace < run_delay || ((w.tick_count - last) as f64) < walk_delay
}

/// One step per period, paced from the step just taken: no debt accrual.
fn consume_move_budget(w: &mut World, id: MobileId) {
    let ms = ((effective_step_delay(w, id) * 1000.0) as i64).max(50);
    let next = w.tick_count + ms;
    edit_ai(w, id, |a| a.next_move = next);
}

/// CheckMove
pub fn check_move(w: &World, id: MobileId) -> bool {
    !(mx::deleted(w, id) || ctype(w, id).is_some_and(|c| (c.disallow_all_moves)(w, id)))
}

/// DoMove
pub fn do_move(w: &mut World, id: MobileId, d: Direction, bad_state_ok: bool) -> bool {
    let res = do_move_impl(w, id, d, bad_state_ok);
    matches!(res, MoveResult::Success | MoveResult::SuccessAutoTurn)
        || bad_state_ok && res == MoveResult::BadState
}

fn is_in_bad_state(w: &World, id: MobileId) -> bool {
    let Some(m) = w.mobile(id) else { return true };
    m.deleted
        || m.frozen
        || (m.ty.paralyzed)(w, id)
        || m.spell
            .is_some_and(|sp| crate::spells::spell::is_casting(w, sp))
        || ctype(w, id).is_some_and(|c| (c.disallow_all_moves)(w, id))
}

/// DoMoveImpl
pub fn do_move_impl(w: &mut World, id: MobileId, d: Direction, _bad_state_ok: bool) -> MoveResult {
    if is_in_bad_state(w, id) || !can_move_now(w, id) {
        return MoveResult::BadState;
    }
    let mut d = (d & Direction::MASK)
        | if should_run(w, id) {
            Direction::RUNNING
        } else {
            Direction::empty()
        };
    let cur = w.mobile(id).map_or(Direction::NORTH, |m| m.direction);
    if (cur & Direction::MASK) != (d & Direction::MASK) {
        muo_server::mobile::part1::props_b::set_direction(w, id, d);
    }
    if let Some(m) = w.mobile_mut(id) {
        m.pushing = false;
    }
    let mob_direction = w.mobile(id).map_or(d, |m| m.direction);
    if try_move(w, id, d) {
        // Obeying pets are paced by their order handlers.
        if !is_obeying_move_order(w, id) {
            let active = w.mobile(id).is_some_and(|m| m.warmode) || mx::combatant(w, id).is_some();
            if active {
                crate::creature::speeds::set_current_speed_to_active(w, id);
            } else {
                crate::creature::speeds::set_current_speed_to_passive(w, id);
            }
        }
        consume_move_budget(w, id);
        return MoveResult::Success;
    }
    if (mob_direction & Direction::MASK) != (d & Direction::MASK) {
        muo_server::mobile::part1::props_b::set_direction(w, id, d);
        d = Direction::empty();
        let _ = d;
        return MoveResult::SuccessAutoTurn;
    }
    handle_blocked_movement(w, id, d)
}

fn try_move(w: &mut World, id: MobileId, d: Direction) -> bool {
    let c = ctype(w, id).expect("creature");
    let ignore = (c.can_move_over_obstacles)(w, id) && !(c.can_destroy_obstacles)(w, id);
    w.resources
        .get_mut::<MovementFlags>()
        .ignore_movable_impassables = ignore;
    let ty = w.mobile(id).map(|m| m.ty).expect("mobile");
    let r = (ty.move_dir)(w, id, d);
    w.resources
        .get_mut::<MovementFlags>()
        .ignore_movable_impassables = false;
    r
}

/// `MovementImpl.IgnoreMovableImpassables` (read by the movement implementation).
#[derive(Default)]
pub struct MovementFlags {
    pub ignore_movable_impassables: bool,
}

fn handle_blocked_movement(w: &mut World, id: MobileId, d: Direction) -> MoveResult {
    let was_pushing = w.mobile(id).is_some_and(|m| m.pushing);
    let c = ctype(w, id).expect("creature");
    if ((c.can_open_doors)(w, id) || (c.can_destroy_obstacles)(w, id))
        && !try_clear_obstacles(w, id, d)
    {
        return MoveResult::Success;
    }
    try_alternate_movement(w, id, was_pushing)
}

fn try_alternate_movement(w: &mut World, id: MobileId, was_pushing: bool) -> MoveResult {
    let offset = if rnd::below(w, 2) == 0 { 1 } else { -1 };
    let c = ctype(w, id).expect("creature");
    for _ in 0..2 {
        (c.turn_internal)(w, id, offset);
        let dir = w.mobile(id).map_or(Direction::NORTH, |m| m.direction);
        let ty = w.mobile(id).map(|m| m.ty).expect("mobile");
        if (ty.move_dir)(w, id, dir) {
            consume_move_budget(w, id);
            return MoveResult::SuccessAutoTurn;
        }
    }
    if was_pushing {
        MoveResult::BadState
    } else {
        MoveResult::Blocked
    }
}

/// TryClearObstacles: false when a door was opened or an obstacle destroyed and the move now works.
fn try_clear_obstacles(w: &mut World, id: MobileId, d: Direction) -> bool {
    use muo_server::item::container::CONTAINER;
    debug_say(w, id, "My movement is blocked. Trying to push through.");
    let Some(map) = mx::map(w, id) else {
        return true;
    };
    let (mut x, mut y) = (mx::location(w, id).x, mx::location(w, id).y);
    muo_server::movement::offset_xy(d, &mut x, &mut y, 1);
    let z = mx::location(w, id).z;
    let c = ctype(w, id).expect("creature");
    let (can_open, can_destroy) = ((c.can_open_doors)(w, id), (c.can_destroy_obstacles)(w, id));

    // GatherObstacles
    let near: Vec<_> = muo_server::map::enumerate::items_in_range(w, Some(map), x, y, 1).collect();
    let mut queue: std::collections::VecDeque<_> = Default::default();
    let mut destroyables = 0;
    for item in near {
        if is_valid_door(w, id, item, x, y, can_open)
            || is_valid_destroyable(w, id, item, can_destroy)
        {
            queue.push_back(item);
            if !is_door(w, item) {
                destroyables += 1;
            }
        }
    }
    if destroyables > 0 {
        muo_server::effects::play_sound(w, Point3D::new(x, y, z), Some(map), 0x3B3);
    }
    // ProcessObstacles
    if queue.is_empty() {
        return true;
    }
    while let Some(item) = queue.pop_front() {
        if is_door(w, item) {
            debug_say(w, id, "Opening the door.");
            if let Some(f) = w.resources.get::<ObstacleHooks>().and_then(|h| h.door_use) {
                f(w, item, id);
            }
        } else {
            let Some(ty) = w.item(item).map(|i| i.ty) else {
                continue;
            };
            debug_say(
                w,
                id,
                &format!(
                    "Destroying item: {}",
                    ty.full_name.rsplit('.').next().unwrap_or("")
                ),
            );
            if ty.is(&CONTAINER) {
                // ProcessContainer
                let cz = w.item(item).map_or(0, |i| i.location.z);
                let children = w.item(item).map(|i| i.items.clone()).unwrap_or_default();
                for ch in children {
                    let Some(ci) = w.item(ch) else { continue };
                    let data = w.maps.tile_data.item((ci.item_id & 0xFFFF) as usize);
                    if ci.movable() && data.impassable() && cz + i32::from(data.height) > z {
                        queue.push_back(ch);
                    }
                }
                (ty.destroy)(w, item);
            } else {
                (ty.delete)(w, item);
            }
        }
    }
    let ty = w.mobile(id).map(|m| m.ty).expect("mobile");
    !(ty.move_dir)(w, id, d)
}

fn is_door(w: &World, item: muo_server::ids::ItemId) -> bool {
    w.resources
        .get::<ObstacleHooks>()
        .and_then(|h| h.is_door)
        .is_some_and(|f| f(w, item))
}

fn is_valid_door(
    w: &World,
    id: MobileId,
    item: muo_server::ids::ItemId,
    x: i32,
    y: i32,
    can_open: bool,
) -> bool {
    if !can_open || !is_door(w, item) {
        return false;
    }
    let Some(i) = w.item(item) else { return false };
    let mz = mx::location(w, id).z;
    let height = i32::from(w.maps.tile_data.item((i.item_id & 0xFFFF) as usize).height);
    if i.location.z + height <= mz || mz + 16 <= i.location.z {
        return false;
    }
    if i.location.x != x || i.location.y != y {
        return false;
    }
    // `!door.Locked || !door.UseLocks()`
    !w.resources
        .get::<ObstacleHooks>()
        .and_then(|h| h.door_blocked_by_lock)
        .is_some_and(|f| f(w, item))
}

fn is_valid_destroyable(
    w: &World,
    id: MobileId,
    item: muo_server::ids::ItemId,
    can_destroy: bool,
) -> bool {
    let Some(i) = w.item(item) else { return false };
    let data = w.maps.tile_data.item((i.item_id & 0xFFFF) as usize);
    if !can_destroy || !i.movable() || !data.impassable() {
        return false;
    }
    let mz = mx::location(w, id).z;
    if i.location.z + i32::from(data.height) <= mz || mz + 16 <= i.location.z {
        return false;
    }
    let loc = muo_server::item::ops::get_world_location(w, item);
    mx::in_range_point(w, id, loc, 1)
}

/// `BaseDoor` calls (the doors are leaf items of another crate): `is_door` is `item is BaseDoor`,
/// `door_blocked_by_lock` is `door.Locked && door.UseLocks()`, `door_use` is `door.Use(from)`. Unset: no doors.
#[derive(Default)]
pub struct ObstacleHooks {
    pub is_door: Option<fn(&World, muo_server::ids::ItemId) -> bool>,
    pub door_blocked_by_lock: Option<fn(&World, muo_server::ids::ItemId) -> bool>,
    pub door_use: Option<fn(&mut World, muo_server::ids::ItemId, MobileId)>,
}

// ---- approach ----------------------------------------------------------------------------------------------

fn path_follow(w: &mut World, id: MobileId, goal: Goal, range: i32) -> bool {
    match w.resources.get::<PathHooks>().and_then(|h| h.follow) {
        Some(f) => f(w, id, goal, range),
        None => {
            // shortcut: reached only when `pathing::init` never ran (unit tests of the AI alone); the server always
            // sets the hook, so no A* detour is skipped in play.
            let loc = match goal {
                Goal::Mobile(m) => mx::location(w, m),
                Goal::Point(p) => p,
            };
            if mx::in_range_point(w, id, loc, range) {
                return true;
            }
            let dir = effects_dir(w, id, loc);
            do_move_impl(w, id, dir, true);
            false
        }
    }
}

fn effects_dir(w: &World, id: MobileId, p: Point3D) -> Direction {
    muo_server::mobile::part2::effects::get_direction_to(w, id, p.x, p.y, false)
}

/// ApproachTarget: the centralized progress-based "move toward target until within range" decision.
pub fn approach_target(w: &mut World, id: MobileId, target: MobileId, range: i32) -> bool {
    let set = |w: &mut World, o: ApproachOutcome| edit_ai(w, id, |a| a.last_approach = o);
    if mx::deleted(w, id) || !check_move(w, id) || mx::deleted(w, target) {
        set(w, ApproachOutcome::InvalidGoal);
        clear_move_intent(w, id);
        return false;
    }
    if mx::in_range(w, id, target, range) {
        set(w, ApproachOutcome::Arrived);
        reset_approach(w, id);
        clear_move_intent(w, id);
        return true;
    }
    // Already gave up on this exact (unreachable) goal: idle until it moves.
    let (gave_up, goal, gave_loc) = ai(w, id).map_or((false, None, Point3D::ZERO), |a| {
        (
            a.approach_gave_up,
            a.approach_goal,
            a.approach_gave_up_goal_loc,
        )
    });
    if gave_up && goal == Some(target) {
        if mx::location(w, target) == gave_loc {
            set(w, ApproachOutcome::GaveUp);
            clear_move_intent(w, id);
            return false;
        }
        reset_approach(w, id); // target moved: try again fresh
    }
    renew_move_intent(w, id, Goal::Mobile(target), range);

    // FAST PATH: a greedy step, counted as success only when it fully succeeded and got us closer.
    if ai(w, id).is_some_and(|a| a.path.is_none()) && mx::in_los(w, id, target) {
        let before = distance_sqrt(w, id, target);
        let dir = mx::get_direction_to(w, id, target);
        let res = do_move_impl(w, id, dir, true);
        if res == MoveResult::BadState {
            set(w, ApproachOutcome::Waiting);
            return true; // not allowed to move this tick (frozen/casting/throttled); not a failure
        }
        if res == MoveResult::Success && distance_sqrt(w, id, target) < before {
            set(w, ApproachOutcome::DirectProgress);
            reset_approach(w, id);
            return true; // healthy en-route progress
        }
    }
    // PLANNING PATH: a persistent path, never discarded by a greedy step.
    if ai(w, id).is_some_and(|a| a.path.as_ref().map(|p| p.goal) != Some(Goal::Mobile(target))) {
        edit_ai(w, id, |a| {
            a.path = Some(crate::pathing::PathFollower::new(Goal::Mobile(target)))
        });
    }
    // Sample move-eligibility BEFORE the attempt: a successful step consumes the move budget.
    let could_move = can_move_now(w, id) && !is_in_bad_state(w, id);
    let loc_before = mx::location(w, id);
    if path_follow(w, id, Goal::Mobile(target), range) {
        set(w, ApproachOutcome::Arrived);
        reset_approach(w, id);
        return true;
    }
    track_approach_progress(w, id, target, could_move);
    if ai(w, id).is_some_and(|a| a.approach_gave_up) {
        set(w, ApproachOutcome::GaveUp);
        return false;
    }
    // En-route progress is success; failure only when a move-eligible tick took no step.
    let progressed = mx::location(w, id) != loc_before || !could_move;
    set(
        w,
        if progressed {
            ApproachOutcome::Routing
        } else {
            ApproachOutcome::Blocked
        },
    );
    progressed
}

/// MoveToPoint: walks toward a fixed point until within `range` (0 = onto the tile). False on arrival or when
/// genuinely unable to make progress.
pub fn move_to_point(w: &mut World, id: MobileId, goal: Point3D, range: i32) -> bool {
    if mx::deleted(w, id) || !check_move(w, id) {
        clear_move_intent(w, id);
        return false;
    }
    if ai(w, id).is_some_and(|a| a.path.as_ref().map(|p| p.goal) != Some(Goal::Point(goal))) {
        edit_ai(w, id, |a| {
            a.path = Some(crate::pathing::PathFollower::new(Goal::Point(goal)))
        });
    }
    renew_move_intent(w, id, Goal::Point(goal), range);
    let could_move = can_move_now(w, id) && !is_in_bad_state(w, id);
    let loc_before = mx::location(w, id);
    if path_follow(w, id, Goal::Point(goal), range) {
        edit_ai(w, id, |a| a.path = None);
        clear_move_intent(w, id);
        return false; // arrived
    }
    let progressed = mx::location(w, id) != loc_before || !could_move;
    if !progressed {
        clear_move_intent(w, id);
    }
    progressed
}

/// Best-distance stuck detection (`TrackApproachProgress`).
fn track_approach_progress(w: &mut World, id: MobileId, target: MobileId, could_move: bool) {
    if !could_move {
        return; // a tick that was never allowed to move (stun, stall) is not a stall
    }
    let dist = distance_sqrt(w, id, target);
    let goal_loc = mx::location(w, target);
    let Some(a) = ai(w, id) else { return };
    // New goal, or the goal moved (active chase): reset the stall baseline.
    if a.approach_goal != Some(target) || goal_loc != a.approach_goal_loc {
        edit_ai(w, id, |a| {
            a.approach_goal = Some(target);
            a.approach_goal_loc = goal_loc;
            a.approach_best_dist = dist;
            a.approach_stall_ticks = 0;
            a.approach_gave_up = false;
        });
        return;
    }
    // Stationary goal: getting closer than ever resets the stall.
    if dist < a.approach_best_dist {
        edit_ai(w, id, |a| {
            a.approach_best_dist = dist;
            a.approach_stall_ticks = 0;
        });
        return;
    }
    let ticks = a.approach_stall_ticks + 1;
    if ticks >= APPROACH_GIVE_UP_TICKS {
        edit_ai(w, id, |a| {
            a.approach_stall_ticks = ticks;
            a.approach_gave_up = true;
            a.approach_gave_up_goal_loc = goal_loc;
            a.path = None;
        });
        clear_move_intent(w, id);
    } else {
        edit_ai(w, id, |a| a.approach_stall_ticks = ticks);
    }
}

fn reset_approach(w: &mut World, id: MobileId) {
    edit_ai(w, id, |a| {
        a.path = None;
        a.approach_goal = None;
        a.approach_goal_loc = Point3D::ZERO;
        a.approach_best_dist = 0.0;
        a.approach_stall_ticks = 0;
        a.approach_gave_up = false;
    });
}

/// ResetApproachState: drops the path, stall state and move intent (after a relocation).
pub fn reset_approach_state(w: &mut World, id: MobileId) {
    reset_approach(w, id);
    clear_move_intent(w, id);
}

fn renew_move_intent(w: &mut World, id: MobileId, goal: Goal, range: i32) {
    // A live pursuit renews every think tick; unrenewed intent dies on its own.
    let expire = w.tick_count + (read(w, id, 0.0, |d| d.current_speed) * 2000.0) as i64 + 250;
    edit_ai(w, id, |a| {
        a.move_intent = Some(goal);
        a.move_intent_range = range;
        a.move_intent_expire = expire;
    });
}

pub fn clear_move_intent(w: &mut World, id: MobileId) {
    edit_ai(w, id, |a| a.move_intent = None);
}

/// TryGetMoveWake: true while a durable movement goal is live; the value is the tick the budget elapses.
pub fn try_get_move_wake(w: &World, id: MobileId) -> Option<i64> {
    let a = ai(w, id)?;
    (a.move_intent.is_some() && w.tick_count - a.move_intent_expire < 0).then_some(a.next_move)
}

/// ContinueMove: advances the current pursuit by one step on a movement-clock wake; no decisions run.
pub fn continue_move(w: &mut World, id: MobileId) {
    let Some(next_move) = try_get_move_wake(w, id) else {
        return;
    };
    if w.tick_count - next_move < 0 {
        return;
    }
    let Some((intent, range)) =
        ai(w, id).and_then(|a| a.move_intent.map(|g| (g, a.move_intent_range)))
    else {
        return;
    };
    match intent {
        Goal::Mobile(m) => {
            approach_target(w, id, m, range);
        }
        Goal::Point(p) => {
            move_to_point(w, id, p, range);
        }
    }
}

/// MoveTo
pub fn move_to(w: &mut World, id: MobileId, m: MobileId, range: i32) -> bool {
    if mx::deleted(w, id) || !check_move(w, id) || mx::deleted(w, m) {
        edit_ai(w, id, |a| a.last_approach = ApproachOutcome::InvalidGoal);
        clear_move_intent(w, id);
        return false;
    }
    if mx::in_range(w, id, m, range) {
        edit_ai(w, id, |a| a.last_approach = ApproachOutcome::Arrived);
        reset_approach(w, id);
        clear_move_intent(w, id);
        return true;
    }
    if use_group_movement(w, id, m, range) {
        return move_to_with_group(w, id, m, range);
    }
    approach_target(w, id, m, range)
}

pub fn is_following_master(w: &World, id: MobileId) -> bool {
    let Some(d) = cd(w, id) else { return false };
    d.controlled
        && d.control_order == OrderType::Follow
        && d.control_target == crate::creature::control::control_master(w, id)
        && mx::combatant(w, id).is_none()
}

/// IsPacingToMaster: following its master, or guarding from outside guard range.
pub fn is_pacing_to_master(w: &World, id: MobileId) -> bool {
    let Some(d) = cd(w, id) else { return false };
    if !d.controlled || mx::combatant(w, id).is_some() {
        return false;
    }
    match d.control_order {
        OrderType::Follow => d.control_target == crate::creature::control::control_master(w, id),
        OrderType::Guard => crate::creature::control::control_master(w, id).is_some_and(|m| {
            !mx::deleted(w, m) && distance_sqrt(w, id, m) as i32 > super::base::GUARD_RANGE
        }),
        _ => false,
    }
}

/// IsObeyingMoveOrder: a pet executing a movement order outside combat; its order handler owns its speed.
pub fn is_obeying_move_order(w: &World, id: MobileId) -> bool {
    cd(w, id).is_some_and(|d| {
        d.controlled
            && mx::combatant(w, id).is_none()
            && matches!(
                d.control_order,
                OrderType::Come | OrderType::Follow | OrderType::Guard
            )
    })
}

fn move_to_with_collision_avoidance(
    w: &mut World,
    id: MobileId,
    target: MobileId,
    range: i32,
) -> bool {
    let direction = mx::get_direction_to(w, id, target);
    // Wall-slide auto-turns must not count as progress, or a creature pinned on geometry reports success forever.
    let res = do_move_impl(w, id, direction, true);
    if matches!(res, MoveResult::Success | MoveResult::BadState) {
        return true;
    }
    let dv = (direction & Direction::MASK).bits() as i32;
    for i in 1..=3 {
        let cw = Direction::from_bits_retain(((dv + i) % 8) as u8);
        if do_move_impl(w, id, cw, true) == MoveResult::Success {
            return true;
        }
        let ccw = Direction::from_bits_retain(((dv - i + 8) % 8) as u8);
        if do_move_impl(w, id, ccw, true) == MoveResult::Success {
            return true;
        }
    }
    // Tactical sidesteps exhausted: route around the obstacle via the centralized approach primitive.
    approach_target(w, id, target, range)
}

/// WalkMobileRange
pub fn walk_mobile_range(
    w: &mut World,
    id: MobileId,
    m: MobileId,
    steps: i32,
    want_min: i32,
    want_max: i32,
) -> bool {
    if mx::deleted(w, id) || !check_move(w, id) {
        edit_ai(w, id, |a| a.last_approach = ApproachOutcome::InvalidGoal);
        return false;
    }
    for _ in 0..steps {
        let curr = distance_sqrt(w, id, m) as i32;
        if curr >= want_min && curr <= want_max {
            edit_ai(w, id, |a| a.last_approach = ApproachOutcome::Arrived);
            return true;
        }
        if !move_towards_or_away_from(w, id, m, curr, want_max) {
            return false;
        }
    }
    let dist = distance_sqrt(w, id, m);
    dist >= want_min as f64 && dist <= want_max as f64
}

fn move_towards_or_away_from(
    w: &mut World,
    id: MobileId,
    m: MobileId,
    curr: i32,
    want_max: i32,
) -> bool {
    if curr > want_max {
        // Too far: approach via the centralized progress-based primitive.
        return approach_target(w, id, m, want_max);
    }
    // Too close: back away. Retreat keeps the simple greedy behavior.
    let away = mx::get_direction_to(w, m, id);
    if do_move(w, id, away, true) {
        edit_ai(w, id, |a| a.path = None);
        return true;
    }
    false
}

// ---- group movement -----------------------------------------------------------------------------------------

/// The `_reservedPositions` and `_lastGroupUpdateTime` statics.
#[derive(Default)]
pub struct GroupState {
    reserved: indexmap::IndexMap<MobileId, Point3D>,
    last_update: i64,
}

fn cleanup_reserved_positions(w: &mut World) {
    let entries: Vec<(MobileId, Point3D)> = w
        .resources
        .get_mut::<GroupState>()
        .reserved
        .iter()
        .map(|(a, b)| (*a, *b))
        .collect();
    for (m, p) in entries {
        let gone = mx::deleted(w, m) || {
            let l = mx::location(w, m);
            utility::distance_xy(l.x, l.y, p.x, p.y) < 1.0
        };
        if gone {
            w.resources
                .get_mut::<GroupState>()
                .reserved
                .shift_remove(&m);
        }
    }
}

/// Crowding refinement for the final approach: engages only near the target when allies contest the ring.
fn use_group_movement(w: &World, id: MobileId, target: MobileId, range: i32) -> bool {
    mx::combatant(w, id) == Some(target)
        && !read(w, id, false, |d| d.controlled)
        && mx::in_range(w, id, target, range + 2)
        && count_crowding_allies(w, id, target, range) > 0
}

fn count_crowding_allies(w: &World, id: MobileId, target: MobileId, range: i32) -> i32 {
    let team = read(w, id, 0, |d| d.team);
    mx::mobiles_in_range(w, target, range + 1)
        .into_iter()
        .filter(|&m| {
            m != id
                && mx::combatant(w, m) == Some(target)
                && cd(w, m).is_some_and(|b| !b.controlled && b.team == team)
        })
        .count() as i32
}

fn nearby_allies(w: &World, id: MobileId, target: MobileId) -> Vec<MobileId> {
    let team = read(w, id, 0, |d| d.team);
    mx::mobiles_in_range(w, id, 8)
        .into_iter()
        .filter(|&m| {
            m != id
                && mx::combatant(w, m) == Some(target)
                && cd(w, m).is_some_and(|b| !b.controlled && b.team == team)
        })
        .collect()
}

pub fn move_to_with_group(w: &mut World, id: MobileId, target: MobileId, range: i32) -> bool {
    let now = w.tick_count;
    if now - w.resources.get_mut::<GroupState>().last_update > 1000 {
        cleanup_reserved_positions(w);
        w.resources.get_mut::<GroupState>().last_update = now;
    }
    let allies = nearby_allies(w, id, target);
    let optimal = calculate_optimal_position(w, id, target, &allies, range);
    if optimal == Point3D::ZERO {
        return move_to_with_collision_avoidance(w, id, target, range);
    }
    w.resources
        .get_mut::<GroupState>()
        .reserved
        .insert(id, optimal);
    let mut direction = effects_dir(w, id, optimal);
    if rnd::below(w, 3) == 0 {
        direction = adjusted_direction(w, direction);
    }
    let res = do_move_impl(w, id, direction, true);
    if matches!(res, MoveResult::Success | MoveResult::BadState) {
        return true;
    }
    // A blocked or wall-slid step is not progress: route around the obstacle.
    approach_target(w, id, target, range)
}

fn calculate_optimal_position(
    w: &mut World,
    id: MobileId,
    target: MobileId,
    allies: &[MobileId],
    range: i32,
) -> Point3D {
    let target_loc = mx::location(w, target);
    let mut best = Point3D::ZERO;
    let mut best_score = -1.0;
    let here = mx::location(w, id);
    'outer: for x in -range..=range {
        for y in -range..=range {
            if x.abs() + y.abs() != range {
                continue;
            }
            let test = Point3D::new(target_loc.x + x, target_loc.y + y, target_loc.z);
            let distance = utility::distance_xy(here.x, here.y, test.x, test.y);
            if distance < range as f64 || distance > (range + 3) as f64 || !can_move_to(w, id, test)
            {
                continue;
            }
            let score = score_position(w, id, test, distance, target, allies);
            if score > best_score {
                best_score = score;
                best = test;
                if score > 20.0 {
                    break 'outer;
                }
            }
        }
    }
    best
}

fn score_position(
    w: &mut World,
    id: MobileId,
    position: Point3D,
    current_distance: f64,
    target: MobileId,
    allies: &[MobileId],
) -> f64 {
    let mut score = -(current_distance * 2.0);
    for &ally in allies {
        let l = mx::location(w, ally);
        let ad = utility::distance_xy(l.x, l.y, position.x, position.y);
        if ad < 2.0 {
            score -= 50.0;
        } else if ad < 3.0 {
            score -= 20.0;
        }
    }
    let reserved: Vec<(MobileId, Point3D)> = w
        .resources
        .get_mut::<GroupState>()
        .reserved
        .iter()
        .map(|(a, b)| (*a, *b))
        .collect();
    for (m, p) in reserved {
        if m != id && utility::distance_xy(p.x, p.y, position.x, position.y) < 2.0 {
            score -= 30.0;
        }
    }
    if let Some(map) = mx::map(w, id) {
        let tl = mx::location(w, target);
        if los::line_of_sight(w, map, position, tl) {
            score += 10.0;
        }
    }
    score + rnd::double(w) * 5.0
}

fn can_move_to(w: &mut World, id: MobileId, l: Point3D) -> bool {
    mx::map(w, id).is_some_and(|map| fit::can_fit(w, map, l.x, l.y, l.z, 16, false, false, true))
}

fn adjusted_direction(w: &mut World, original: Direction) -> Direction {
    let adjustment = rnd::below(w, 3) - 1;
    let n = (original & Direction::MASK).bits() as i32 + adjustment;
    Direction::from_bits_retain(
        (if n < 0 {
            n + 8
        } else if n >= 8 {
            n - 8
        } else {
            n
        }) as u8,
    )
}

// ---- random walks ----------------------------------------------------------------------------------------------

/// WalkRandom
pub fn walk_random(
    w: &mut World,
    id: MobileId,
    chance_to_not_move: i32,
    chance_to_dir: i32,
    steps: i32,
) {
    if mx::deleted(w, id) || !check_move(w, id) || chance_to_not_move <= 0 {
        return;
    }
    let max_steps = steps.min(3);
    for _ in 0..max_steps {
        if rnd::below(w, 1 + chance_to_not_move) == 0 {
            let d = get_random_direction(w, id, chance_to_dir);
            do_move(w, id, d, false);
        }
    }
}

fn get_random_direction(w: &mut World, id: MobileId, chance_to_dir: i32) -> Direction {
    let r = rnd::below(w, 8 * (chance_to_dir + 1));
    if r < 8 {
        Direction::from_bits_retain(r as u8)
    } else {
        w.mobile(id).map_or(Direction::NORTH, |m| m.direction)
    }
}

/// WalkRandomIdle
pub fn walk_random_idle(w: &mut World, id: MobileId) {
    let idle = |w: &mut World| (ctype(w, id).expect("creature").check_idle)(w, id);
    if check_move(w, id) && can_move_now(w, id) && !idle(w) {
        walk_random_in_home(w, id, 3, 2, 1);
    }
}

/// WalkRandomInHome
pub fn walk_random_in_home(
    w: &mut World,
    id: MobileId,
    chance_to_not_move: i32,
    chance_to_dir: i32,
    steps: i32,
) {
    if mx::deleted(w, id) || !check_move(w, id) {
        return;
    }
    let home = read(w, id, Point3D::ZERO, |d| d.home);
    if home == Point3D::ZERO {
        walk_random_no_home(w, id, chance_to_not_move, chance_to_dir, steps);
    } else {
        walk_random_with_home(w, id, chance_to_not_move, chance_to_dir, steps);
    }
}

/// WalkRandomNoHome: a RegionSpawner's creature walks only inside its SpawnRegion.
fn walk_random_no_home(
    w: &mut World,
    id: MobileId,
    chance_to_not_move: i32,
    chance_to_dir: i32,
    steps: i32,
) {
    use muo_server::mobile::part2::deps as sd;
    let region = w
        .mobile(id)
        .and_then(|m| m.spawner)
        .and_then(|sp| sd::spawner_region_spawner_region(w, sp));
    let Some(region) = region else {
        walk_random(w, id, chance_to_not_move, chance_to_dir, steps);
        return;
    };
    let accepts = w.mobile(id).map(|m| m.region).is_some_and(|r| {
        w.region(r)
            .is_some_and(|reg| (reg.ty.accepts_spawns_from)(w, r, region))
    });
    let go = w.region(region).map_or(Point3D::ZERO, |r| r.go_location);
    if accepts {
        if let Some(m) = w.mobile_mut(id) {
            m.walk_region = Some(region);
        }
        walk_random(w, id, chance_to_not_move, chance_to_dir, steps);
        if let Some(m) = w.mobile_mut(id) {
            m.walk_region = None;
        }
    } else if go != Point3D::ZERO && rnd::bool(w) {
        let d = effects_dir(w, id, go);
        do_move(w, id, d, false);
    } else {
        walk_random(w, id, chance_to_not_move, chance_to_dir, 1);
    }
}

fn walk_random_with_home(
    w: &mut World,
    id: MobileId,
    chance_to_not_move: i32,
    chance_to_dir: i32,
    steps: i32,
) {
    let (home, range_home) = read(w, id, (Point3D::ZERO, 0), |d| (d.home, d.range_home));
    if range_home == 0 {
        if mx::location(w, id) != home {
            let d = effects_dir(w, id, home);
            do_move(w, id, d, false);
        }
        return;
    }
    for _ in 0..steps {
        let l = mx::location(w, id);
        let curr = utility::distance_xy(l.x, l.y, home.x, home.y) as i32;
        if curr > range_home {
            let d = effects_dir(w, id, home);
            do_move(w, id, d, false);
        } else if curr < range_home * 2 / 3 || rnd::below(w, 10) <= 5 {
            walk_random(w, id, chance_to_not_move, chance_to_dir, 1);
        } else {
            let d = effects_dir(w, id, home);
            do_move(w, id, d, false);
        }
    }
}
