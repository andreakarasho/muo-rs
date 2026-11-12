//! `PathFollower`. A creature's follower is `AiState::path`; its `Mover` is
//! `ai::movement::do_move_impl(.., true)`, the only mover ever set.

use muo_primitives::{Direction, Point3D, utility};
use muo_server::ids::MobileId;
use muo_server::movement;
use muo_server::world::World;

use super::astar::find_path;
use crate::ai::movement::do_move_impl;
use crate::ai::{Goal, MoveResult, ai_mut};
use crate::mx;

/// `RepathDelay`, 2 seconds in ticks.
const REPATH_DELAY_TICKS: i64 = 20_000_000;

/// `pathfinding.enable`.
pub struct PathConfig {
    pub enabled: bool,
}

impl Default for PathConfig {
    fn default() -> Self {
        PathConfig { enabled: true }
    }
}

/// One `PathFollower`.
pub struct PathFollower {
    pub goal: Goal,
    index: i32,
    last_path_time: i64,
    next: Point3D,
    last_goal_loc: Point3D,
    /// `m_Path`: `None` is null, an empty route is a path whose `Success` is false.
    path: Option<Vec<Direction>>,
}

impl PathFollower {
    pub fn new(goal: Goal) -> PathFollower {
        PathFollower {
            goal,
            index: 0,
            last_path_time: 0,
            next: Point3D::ZERO,
            last_goal_loc: Point3D::ZERO,
            path: None,
        }
    }

    fn success(&self) -> bool {
        self.path.as_ref().is_some_and(|p| !p.is_empty())
    }

    fn goal_location(&self, w: &World) -> Point3D {
        match self.goal {
            Goal::Mobile(m) => mx::location(w, m),
            Goal::Point(p) => p,
        }
    }

    /// `Advance(ref p, index)`.
    fn advance(&self, p: &mut Point3D, index: i32) {
        if let Some(dirs) = self.path.as_ref().filter(|d| !d.is_empty())
            && let Some(&d) = usize::try_from(index).ok().and_then(|i| dirs.get(i))
        {
            movement::offset(d, p, 1);
        }
    }

    /// `ForceRepath`.
    pub fn force_repath(&mut self) {
        self.path = None;
    }

    /// `CheckPath`: true when a new route was computed.
    fn check_path(&mut self, w: &mut World, from: MobileId) -> bool {
        if !w.resources.get::<PathConfig>().is_none_or(|c| c.enabled) {
            return false;
        }
        let goal = self.goal_location(w);
        let now = w.now.ticks();
        let from_loc = mx::location(w, from);
        if self.path.is_some()
            && ((self.success() && goal == self.last_goal_loc)
                || self.last_path_time + REPATH_DELAY_TICKS > now)
            && !(self.success() && check(from_loc, self.last_goal_loc, 0))
        {
            return false;
        }
        self.last_path_time = now;
        self.last_goal_loc = goal;
        self.path = Some(find_path(w, from, goal));
        self.index = 0;
        self.next = from_loc;
        let mut next = self.next;
        self.advance(&mut next, self.index);
        self.next = next;
        true
    }

    fn mv(w: &mut World, from: MobileId, d: Direction) -> MoveResult {
        do_move_impl(w, from, d, true)
    }

    /// `Follow(range)`: true once within `range` of the goal.
    pub fn follow(&mut self, w: &mut World, from: MobileId, range: i32) -> bool {
        let goal = self.goal_location(w);
        if check(mx::location(w, from), goal, range) {
            return true;
        }
        let repathed = self.check_path(w, from);
        let enabled = w.resources.get::<PathConfig>().is_none_or(|c| c.enabled);

        let arrived_after = |w: &World, res: MoveResult| {
            matches!(res, MoveResult::Success | MoveResult::SuccessAutoTurn)
                && check(mx::location(w, from), goal, range)
        };
        if !(enabled && self.success()) {
            let d = direction_to(w, from, goal);
            mx::set_direction(w, from, d);
            let res = Self::mv(w, from, d);
            return arrived_after(w, res);
        }

        let d = direction_to(w, from, self.next);
        mx::set_direction(w, from, d);
        let res = Self::mv(w, from, d);
        if res == MoveResult::Blocked {
            if repathed {
                return false;
            }
            self.path = None;
            self.check_path(w, from);
            if !self.success() {
                let d = direction_to(w, from, goal);
                mx::set_direction(w, from, d);
                let res = Self::mv(w, from, d);
                return arrived_after(w, res);
            }
            let d = direction_to(w, from, self.next);
            mx::set_direction(w, from, d);
            if Self::mv(w, from, d) == MoveResult::Blocked {
                return false;
            }
        }

        let loc = mx::location(w, from);
        if loc.x == self.next.x && loc.y == self.next.y {
            if loc.z == self.next.z {
                self.index += 1;
                let mut next = self.next;
                self.advance(&mut next, self.index);
                self.next = next;
            } else {
                self.path = None;
            }
        }
        check(mx::location(w, from), goal, range)
    }
}

/// `Check(loc, goal, range)`.
pub fn check(loc: Point3D, goal: Point3D, range: i32) -> bool {
    utility::in_range(loc.x, loc.y, goal.x, goal.y, range)
        && (range > 1 || (loc.z - goal.z).abs() < 16)
}

fn direction_to(w: &World, from: MobileId, p: Point3D) -> Direction {
    muo_server::mobile::part2::effects::get_direction_to(w, from, p.x, p.y, false)
}

/// `PathHooks::follow`: runs the creature's follower for its current goal.
pub fn follow_hook(w: &mut World, id: MobileId, goal: Goal, range: i32) -> bool {
    // Take the follower out so it can borrow the world; nothing in a step touches `AiState::path`.
    let Some(mut f) = ai_mut(w, id).and_then(|a| a.path.take()) else {
        let mut f = PathFollower::new(goal);
        let r = f.follow(w, id, range);
        if let Some(a) = ai_mut(w, id) {
            a.path = Some(f);
        }
        return r;
    };
    if f.goal != goal {
        f = PathFollower::new(goal);
    }
    let r = f.follow(w, id, range);
    if let Some(a) = ai_mut(w, id) {
        a.path = Some(f);
    }
    r
}

/// `PathHooks::force_repath`: `Path?.ForceRepath()`.
pub fn force_repath_hook(w: &mut World, id: MobileId) {
    if let Some(f) = ai_mut(w, id).and_then(|a| a.path.as_mut()) {
        f.force_repath();
    }
}
