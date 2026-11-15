//! The bitmap A* path algorithm, with MovementPath's constructor.
//!
//! shortcut: every cell is expanded the "slow path" way (one `CheckMovement` per direction). A `StepCache` bitmap
//! would only answer the same question faster (same successor order), so the paths are the same; the open list
//! sifts like a quaternary-heap `PriorityQueue`, so ties break in a fixed order. The cache, `PathfindRecorder`,
//! `PathDiag` and the `PathCache` commands are not implemented (upgrade when a profile asks for it).

use muo_primitives::{Direction, Point3D, utility};
use muo_server::ids::MobileId;
use muo_server::map::{MapId, enumerate};
use muo_server::movement::{self, MovementImpl};
use muo_server::world::World;

use super::movement_impl::{IMPL, MovementStatics, can_move_over, is_door_id};
use crate::ai::movement::MovementFlags;
use crate::creature::vtable::ctype;

/// `System.Collections.Generic.PriorityQueue<int, int>`: a 4-ary array heap that isn't stable, so nodes of equal
/// priority come out in an order only this exact sift decides; `BinaryHeap` would pick other equally short paths.
#[derive(Default)]
struct PriorityQueue {
    nodes: Vec<(i32, i32)>, // (element, priority)
}

impl PriorityQueue {
    fn enqueue(&mut self, element: i32, priority: i32) {
        // MoveUp
        let mut i = self.nodes.len();
        self.nodes.push((element, priority));
        while i > 0 {
            let parent = (i - 1) >> 2;
            if priority < self.nodes[parent].1 {
                self.nodes[i] = self.nodes[parent];
                i = parent;
            } else {
                break;
            }
        }
        self.nodes[i] = (element, priority);
    }

    /// `TryDequeue`: `(element, priority)` of the minimum.
    fn dequeue(&mut self) -> Option<(i32, i32)> {
        let root = *self.nodes.first()?;
        let last = self.nodes.pop()?;
        let size = self.nodes.len();
        if size > 0 {
            // MoveDownDefaultComparer(last, 0)
            let mut i = 0;
            loop {
                let first = (i << 2) + 1;
                if first >= size {
                    break;
                }
                let (mut min_child, mut min_index) = (self.nodes[first], first);
                for c in first + 1..(first + 4).min(size) {
                    if self.nodes[c].1 < min_child.1 {
                        min_child = self.nodes[c];
                        min_index = c;
                    }
                }
                if last.1 <= min_child.1 {
                    break;
                }
                self.nodes[i] = min_child;
                i = min_index;
            }
            self.nodes[i] = last;
        }
        Some(root)
    }
}

const AREA_SIZE: i32 = 38;
const PLANE_COUNT: i32 = 13;
const NODE_COUNT: usize = (AREA_SIZE * AREA_SIZE * PLANE_COUNT) as usize;
const PLANE_OFFSET: i32 = 128;
const PLANE_HEIGHT: i32 = 20;
const PERSON_HEIGHT: i32 = 16;
const MOBILE_HEIGHT: i32 = 15;

#[derive(Copy, Clone, Default)]
struct PathNode {
    cost: i32,
    total: i32,
    parent: i32,
    z: i32,
}

/// The scratch of one `BitmapAStarAlgorithm` instance (about 320 KB, kept in `w.resources`).
pub struct PathScratch {
    nodes: Vec<PathNode>,
    states: Vec<u8>,
    /// `MaxSearchNodes` (`pathfinding.maxSearchNodes`).
    pub max_search_nodes: i32,
}

impl Default for PathScratch {
    fn default() -> Self {
        PathScratch {
            nodes: Vec::new(),
            states: Vec::new(),
            max_search_nodes: 1000,
        }
    }
}

/// `PathAlgorithm.GetDirection`.
fn get_direction(xs: i32, ys: i32, xd: i32, yd: i32) -> Direction {
    const CALC: [Direction; 9] = [
        Direction::UP,
        Direction::NORTH,
        Direction::RIGHT,
        Direction::WEST,
        Direction::NORTH,
        Direction::EAST,
        Direction::LEFT,
        Direction::SOUTH,
        Direction::DOWN,
    ];
    let v = (yd + 1 - ys) * 3 + (xd + 1 - xs);
    if !(0..9).contains(&v) {
        Direction::NORTH
    } else {
        CALC[v as usize]
    }
}

struct Search {
    goal: Point3D,
    x_offset: i32,
    y_offset: i32,
    mover: MobileId,
    map: MapId,
    ignore_doors: bool,
    ignore_spell_fields: bool,
    ignore_movable_impassables: bool,
}

impl Search {
    fn heuristic(&self, x: i32, y: i32, z: i32) -> i32 {
        let x = (x - (self.goal.x - self.x_offset)) * 11;
        let y = (y - (self.goal.y - self.y_offset)) * 11;
        let z = z - self.goal.z;
        x * x + y * y + z * z
    }

    fn index(&self, x: i32, y: i32, z: i32) -> i32 {
        (x - self.x_offset)
            + (y - self.y_offset) * AREA_SIZE
            + ((z + PLANE_OFFSET) / PLANE_HEIGHT) * AREA_SIZE * AREA_SIZE
    }

    /// `IsBlockedByDynamic`.
    fn blocked_by_dynamic(&self, w: &World, x: i32, y: i32, z: i32) -> bool {
        let our_top = z + PERSON_HEIGHT;
        for id in super::movement_impl::items_at(w, self.map, x, y) {
            let Some(item) = w.item(id) else { continue };
            let item_id = (item.item_id & 0xFFFF) as usize;
            let d = w.maps.tile_data.item(item_id);
            if !d.impassable_surface() {
                continue;
            }
            if self.ignore_movable_impassables && item.movable() {
                continue;
            }
            if self.ignore_doors && is_door_id(d, item_id) {
                continue;
            }
            if self.ignore_spell_fields && matches!(item_id, 0x82 | 0x3946 | 0x3956) {
                continue;
            }
            let check_z = item.location.z;
            if check_z + d.calc_height() > z && our_top > check_z {
                return true;
            }
        }
        // The goal cell is usually occupied by whatever is chased; the follower stops short of it anyway.
        if !(x == self.goal.x && y == self.goal.y) {
            for mob in super::movement_impl::mobiles_at(w, self.map, x, y) {
                if mob == self.mover {
                    continue;
                }
                let Some(mz) = w.mobile(mob).map(|o| o.location.z) else {
                    continue;
                };
                if mz + MOBILE_HEIGHT > z
                    && z + MOBILE_HEIGHT > mz
                    && !can_move_over(w, self.mover, mob)
                {
                    return true;
                }
            }
        }
        false
    }

    /// `GetSuccessorsSlowPath`: `(node index)` of each walkable neighbour of node `p`, with its Z stored.
    fn successors(&self, w: &mut World, nodes: &mut [PathNode], p: i32) -> Vec<i32> {
        let px = p % AREA_SIZE;
        let py = p / AREA_SIZE % AREA_SIZE;
        let p3d = Point3D::new(px + self.x_offset, py + self.y_offset, nodes[p as usize].z);
        let mut out = Vec::with_capacity(8);
        for i in 0..8 {
            let (mut x, mut y) = (px, py);
            let d = Direction::from_bits_retain(i as u8);
            movement::offset_xy(d, &mut x, &mut y, 1);
            if !(0..AREA_SIZE).contains(&x) || !(0..AREA_SIZE).contains(&y) {
                continue;
            }
            let (ok, z) = IMPL.check_movement_at(w, self.mover, self.map, p3d, d);
            if !ok {
                continue;
            }
            let (abs_x, abs_y) = (x + self.x_offset, y + self.y_offset);
            if self.blocked_by_dynamic(w, abs_x, abs_y, z) {
                continue;
            }
            let idx = self.index(abs_x, abs_y, z);
            if idx >= 0 && (idx as usize) < NODE_COUNT {
                nodes[idx as usize].z = z;
                out.push(idx);
            }
        }
        out
    }
}

/// `MovementPath`'s constructor: the directions of the route, empty when there is none (`Success` is non-empty).
pub fn find_path(w: &mut World, m: MobileId, goal: Point3D) -> Vec<Direction> {
    let Some((map, start)) = w.mobile(m).map(|mm| (mm.map, mm.location)) else {
        return Vec::new();
    };
    let Some(map) = map else { return Vec::new() };
    if map == MapId::INTERNAL || utility::in_range(start.x, start.y, goal.x, goal.y, 1) {
        return Vec::new();
    }
    find(w, m, map, start, goal).unwrap_or_default()
}

/// `BitmapAStarAlgorithm.Find` (`CheckCondition` is the range test).
fn find(
    w: &mut World,
    m: MobileId,
    map: MapId,
    start: Point3D,
    goal: Point3D,
) -> Option<Vec<Direction>> {
    if !utility::in_range(start.x, start.y, goal.x, goal.y, AREA_SIZE) {
        return None;
    }
    super::recorder::record_if_enabled(w, m, map, start, goal);
    let is_creature = crate::creature::is_creature(w, m);
    let (can_open_doors, can_move_over_obstacles) = match ctype(w, m) {
        Some(c) if is_creature => ((c.can_open_doors)(w, m), (c.can_move_over_obstacles)(w, m)),
        _ => (false, false),
    };
    let mm = w.mobile(m)?;
    let spectral = !(mm.ty.alive)(w, m)
        || muo_server::mobile::part1::props_b::body(w, m).0 == 0x3DB
        || (mm.ty.is_dead_bonded_pet)(w, m);
    let is_player_mobile = mm.ty.is(&crate::player_mobile::PLAYER_MOBILE);
    let mut s = Search {
        goal,
        x_offset: (start.x + goal.x - AREA_SIZE) / 2,
        y_offset: (start.y + goal.y - AREA_SIZE) / 2,
        mover: m,
        map,
        ignore_doors: can_open_doors || spectral,
        ignore_spell_fields: is_player_mobile && map != MapId::FELUCCA,
        ignore_movable_impassables: can_move_over_obstacles,
    };

    // The search area's mobiles and blocking items by tile, read once (see MovementStatics::mobs_at).
    let area = muo_primitives::Rectangle2D::new(
        s.x_offset - 1,
        s.y_offset - 1,
        AREA_SIZE + 2,
        AREA_SIZE + 2,
    );
    let mut mobs = super::movement_impl::ByTile::default();
    for mob in enumerate::mobiles_in_bounds(w, Some(map), area, false).collect::<Vec<_>>() {
        if let Some(p) = w.mobile(mob).map(|o| o.location) {
            mobs.entry((p.x, p.y)).or_insert_with(Vec::new).push(mob);
        }
    }
    let mut items = super::movement_impl::ByTile::default();
    for id in enumerate::items_in_bounds(w, Some(map), area, false).collect::<Vec<_>>() {
        if super::movement_impl::may_block(w, id)
            && let Some(p) = w.item(id).map(|i| i.location)
        {
            items.entry((p.x, p.y)).or_insert_with(Vec::new).push(id);
        }
    }
    let statics = w.resources.get_mut::<MovementStatics>();
    statics.mobs_at = Some(mobs);
    statics.items_at = Some(items);
    let mut scratch = std::mem::take(w.resources.get_mut::<PathScratch>());
    let result = run(
        w,
        &mut scratch,
        &mut s,
        start,
        can_open_doors,
        can_move_over_obstacles,
    );
    *w.resources.get_mut::<PathScratch>() = scratch;
    let statics = w.resources.get_mut::<MovementStatics>();
    statics.mobs_at = None;
    statics.items_at = None;
    result
}

fn run(
    w: &mut World,
    scratch: &mut PathScratch,
    s: &mut Search,
    start: Point3D,
    can_open_doors: bool,
    can_move_over_obstacles: bool,
) -> Option<Vec<Direction>> {
    let is_creature = crate::creature::is_creature(w, s.mover);
    if scratch.nodes.len() != NODE_COUNT {
        scratch.nodes = vec![PathNode::default(); NODE_COUNT];
        scratch.states = vec![0; NODE_COUNT];
    }
    scratch.states.fill(0);
    let from_node = s.index(start.x, start.y, start.z);
    let dest_node = s.index(s.goal.x, s.goal.y, s.goal.z);
    if from_node < 0 || from_node as usize >= NODE_COUNT {
        return None;
    }
    {
        let n = &mut scratch.nodes[from_node as usize];
        n.cost = 0;
        n.total = 0;
        n.parent = -1;
        n.z = start.z;
    }
    scratch.nodes[from_node as usize].total =
        s.heuristic(start.x - s.x_offset, start.y - s.y_offset, start.z);
    let mut open = PriorityQueue::default();
    open.enqueue(from_node, scratch.nodes[from_node as usize].total);
    scratch.states[from_node as usize] = 1;

    let mut depth = 0;
    while let Some((best, best_total)) = open.dequeue() {
        depth += 1;
        if depth > scratch.max_search_nodes {
            break;
        }
        // Duplicate, lower priority
        if scratch.states[best as usize] == 2 || scratch.nodes[best as usize].total != best_total {
            continue;
        }
        scratch.states[best as usize] = 2;

        // MovementImpl reads these statics, so every CheckMovement below needs them set for this mobile.
        if is_creature {
            w.resources.get_mut::<MovementStatics>().always_ignore_doors = can_open_doors;
            w.resources
                .get_mut::<MovementFlags>()
                .ignore_movable_impassables = can_move_over_obstacles;
        }
        w.resources.get_mut::<MovementStatics>().goal = s.goal;
        let vals = s.successors(w, &mut scratch.nodes, best);
        w.resources.get_mut::<MovementStatics>().always_ignore_doors = false;
        w.resources
            .get_mut::<MovementFlags>()
            .ignore_movable_impassables = false;
        w.resources.get_mut::<MovementStatics>().goal = Point3D::ZERO;

        for (i, &new_node) in vals.iter().enumerate() {
            if scratch.states[new_node as usize] == 2 {
                continue;
            }
            let is_diagonal = i % 2 == 1;
            let new_cost = scratch.nodes[best as usize].cost + if is_diagonal { 14 } else { 10 };
            let new_total = new_cost
                + s.heuristic(
                    new_node % AREA_SIZE,
                    new_node / AREA_SIZE % AREA_SIZE,
                    scratch.nodes[new_node as usize].z,
                );
            if scratch.states[new_node as usize] == 0
                || new_total < scratch.nodes[new_node as usize].total
            {
                let n = &mut scratch.nodes[new_node as usize];
                n.parent = best;
                n.cost = new_cost;
                n.total = new_total;
                open.enqueue(new_node, new_total);
                scratch.states[new_node as usize] = 1;
            }
            if new_node != dest_node {
                continue;
            }
            let mut path = Vec::new();
            let mut cur = new_node;
            let mut parent = scratch.nodes[cur as usize].parent;
            while parent != -1 {
                path.push(get_direction(
                    parent % AREA_SIZE,
                    parent / AREA_SIZE % AREA_SIZE,
                    cur % AREA_SIZE,
                    cur / AREA_SIZE % AREA_SIZE,
                ));
                cur = parent;
                parent = scratch.nodes[cur as usize].parent;
                if cur == from_node {
                    break;
                }
            }
            path.reverse();
            return Some(path);
        }
    }
    None
}
