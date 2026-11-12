//! A* search and multi routing tests, and the tie-breaking of the node
//! queue, on the invented tile kit of `map::test_support` (flat ground, Z 0).

use muo_data::TileFlag;
use muo_data::client::multi_data::{MultiComponentList, MultiTileEntry};
use muo_primitives::{Direction, Point3D};
use muo_server::ids::{ItemId, MobileId};
use muo_server::map::MapId;
use muo_server::map::test_support::{
    COUNTER, FLOOR, WALL, add_static, delete_item, entry, p, place_item, place_mobile, place_multi,
    world,
};
use muo_server::mobile::MobileType;
use muo_server::movement::{MovementImpl, offset};
use muo_server::world::World;

use super::astar::find_path;
use super::movement_impl::IMPL;

use crate::creature::enums::{AIType, FightMode};
use crate::creature::setters::new_creature;
use crate::creature::speeds::{NpcSpeeds, SpeedClassEntry};
use crate::creature::vtable::{BASE_CREATURE, BASE_CREATURE_C, CreatureType};

// ---- tie-breaking: the open list's 4-ary heap order ---------------------------------------------------------
//
// The expected routes are pinned: the search's loop over a 4-ary array heap (`PriorityQueue<int, int>` order),
// over the same flat ground with walls as blocked cells.
// Many equally short routes exist in each; only the queue's sift order says which one comes out.

const N: Direction = Direction::NORTH;
const NE: Direction = Direction::RIGHT;
const E: Direction = Direction::EAST;
const SE: Direction = Direction::DOWN;
const S: Direction = Direction::SOUTH;
const SW: Direction = Direction::LEFT;
const W: Direction = Direction::WEST;
const NW: Direction = Direction::UP;

fn route(w: &mut World, from: Point3D, to: Point3D) -> Vec<Direction> {
    let m = place_mobile(w, MapId::FELUCCA, from);
    find_path(w, m, to)
}

#[test]
fn ties_on_open_ground_break_in_queue_order() {
    type Case = ((i32, i32), (i32, i32), &'static [Direction]);
    let cases: [Case; 7] = [
        ((100, 100), (105, 103), &[SE, SE, SE, E, E]),
        ((100, 100), (97, 108), &[SW, SW, SW, S, S, S, S, S]),
        ((100, 100), (110, 100), &[E; 10]),
        ((100, 100), (107, 98), &[NE, NE, E, E, E, E, E]),
        ((100, 100), (104, 104), &[SE, SE, SE, SE]),
        ((100, 100), (109, 105), &[SE, SE, SE, SE, SE, E, E, E, E]),
        ((1500, 1600), (1497, 1599), &[NW, W, W]),
    ];
    for ((sx, sy), (gx, gy), want) in cases {
        let mut w = world();
        assert_eq!(
            route(&mut w, p(sx, sy, 0), p(gx, gy, 0)),
            want,
            "({sx},{sy}) -> ({gx},{gy})"
        );
    }
    let _ = N;
}

#[test]
fn ties_around_walls_break_in_queue_order() {
    // A row of wall across y 98: the detour goes round its east end.
    let mut w = world();
    for x in 96..=104 {
        add_static(&mut w, MapId::FELUCCA, x, 98, WALL, 0);
    }
    assert_eq!(
        route(&mut w, p(100, 100, 0), p(100, 95, 0)),
        [NE, E, E, E, NE, NW, NW, NW, W, W]
    );
    // A 3x3 block of wall between start and goal.
    let mut w = world();
    for x in 103..=105 {
        for y in 99..=101 {
            add_static(&mut w, MapId::FELUCCA, x, y, WALL, 0);
        }
    }
    assert_eq!(
        route(&mut w, p(100, 100, 0), p(108, 100, 0)),
        [E, NE, NE, E, E, SE, SE, E]
    );
}

/// Scattered walls where the 4-ary queue and a (priority, node) heap (Rust's `BinaryHeap`) give different, equally
/// short routes: the expected ones are the 4-ary queue's.
#[test]
fn ties_among_scattered_walls_break_in_queue_order() {
    type Case = (&'static [(i32, i32)], (i32, i32), &'static [Direction]);
    let cases: [Case; 3] = [
        (
            &[
                (100, 97),
                (100, 98),
                (103, 99),
                (103, 103),
                (107, 107),
                (107, 108),
                (108, 101),
                (108, 102),
                (108, 103),
            ],
            (100, 95),
            &[N, NE, N, NW, N],
        ),
        (
            &[
                (98, 98),
                (98, 99),
                (98, 100),
                (98, 101),
                (98, 102),
                (99, 100),
                (100, 108),
                (101, 100),
                (101, 108),
                (102, 108),
                (103, 105),
                (106, 99),
                (107, 99),
                (108, 99),
                (109, 99),
            ],
            (95, 100),
            &[SW, S, SW, NW, NW, NW],
        ),
        (
            &[
                (99, 97),
                (99, 107),
                (99, 108),
                (99, 109),
                (99, 110),
                (100, 102),
                (101, 97),
                (102, 98),
                (102, 99),
                (102, 102),
                (104, 108),
                (106, 97),
                (107, 97),
                (107, 99),
                (107, 100),
                (107, 101),
                (107, 102),
            ],
            (99, 94),
            &[NW, N, NE, NW, N, N],
        ),
    ];
    for (walls, (gx, gy), want) in cases {
        let mut w = world();
        for &(x, y) in walls {
            add_static(&mut w, MapId::FELUCCA, x, y, WALL, 0);
        }
        assert_eq!(
            route(&mut w, p(100, 100, 0), p(gx, gy, 0)),
            want,
            "goal ({gx},{gy})"
        );
    }
}

// ---- A* search ------------------------------------------------------------------------------------------------
//
// Not covered, as they only read StepCache counters (BuildsTotal, FallthroughNotBuilt): there is no step cache (every
// cell is expanded the way the slow path does, which is what the cache must equal), so there is nothing to count:
// Find_SinglePathfindOnClearedCache_DoesNotBuildAnyChunk and Find_TwoPathfindsOverlappingChunks_PromoteToBuildOnSecondFind
// (the promotion gate), and the "uses the cache, not the slow path" halves of the capability tests. What each of
// those tests finds is kept below.

const Z: i32 = 0;

fn cells_of(start: Point3D, dirs: &[Direction]) -> Vec<(i32, i32)> {
    let mut at = start;
    dirs.iter()
        .map(|&d| {
            offset(d, &mut at, 1);
            (at.x, at.y)
        })
        .collect()
}

#[test]
fn default_walker_finds_a_path() {
    // DefaultWalker_FindsPath_ViaCacheFastPath [2], FeatureFlagDisabled_RoutesToSlowPath_NoCacheUse and
    // Find_CacheOn_EqualsCacheOff_NoMultiControl [2] (the same two routes).
    for (gx, gy, want) in [(1498, 1598, &[NW, NW][..]), (1497, 1599, &[NW, W, W][..])] {
        let mut w = world();
        let m = place_mobile(&mut w, MapId::FELUCCA, p(1500, 1600, Z));
        let first = find_path(&mut w, m, p(gx, gy, Z));
        assert_eq!(first, want);
        // Both runs (cache on, cache off) are this one search: it repeats exactly.
        assert_eq!(find_path(&mut w, m, p(gx, gy, Z)), first);
    }
}

static FLYING_C: CreatureType = CreatureType {
    can_fly: |_, _| true,
    ..BASE_CREATURE_C
};
static FLYING: MobileType = MobileType {
    full_name: "Server.Tests.FlyingStub",
    parent: Some(&BASE_CREATURE),
    family: Some(&FLYING_C),
    ..BASE_CREATURE
};
static DOOR_OPENER_C: CreatureType = CreatureType {
    can_open_doors: |_, _| true,
    ..BASE_CREATURE_C
};
static DOOR_OPENER: MobileType = MobileType {
    full_name: "Server.Tests.DoorOpenerStub",
    parent: Some(&BASE_CREATURE),
    family: Some(&DOOR_OPENER_C),
    ..BASE_CREATURE
};
static CLIMBER_C: CreatureType = CreatureType {
    can_move_over_obstacles: |_, _| true,
    ..BASE_CREATURE_C
};
static CLIMBER: MobileType = MobileType {
    full_name: "Server.Tests.ObstacleClimberStub",
    parent: Some(&BASE_CREATURE),
    family: Some(&CLIMBER_C),
    ..BASE_CREATURE
};

fn stub_creature(w: &mut World, ty: &'static MobileType, at: Point3D) -> MobileId {
    w.resources
        .get_mut::<NpcSpeeds>()
        .register(SpeedClassEntry {
            level: "Medium".into(),
            active: 0.3,
            passive: 0.6,
            ..Default::default()
        });
    let m = new_creature(w, ty, AIType::AiAnimal, FightMode::Closest, 10, 1);
    muo_server::mobile::base::move_to_world(w, m, at, MapId::FELUCCA);
    m
}

#[test]
fn capability_creatures_and_players_find_the_path() {
    let (start, goal) = (p(1500, 1600, Z), p(1498, 1598, Z));
    // SwimCreature_FindsPath_ViaCacheCapabilityOverlay [2], NonGmPlayer_UsesCache_WithStrictDiagonalRule,
    // FlyCreature_RoutesToSlowPath_NoCacheUse, DoorCreature_ and ObstacleCreature_: on open ground each takes the
    // diagonal.
    let mut w = world();
    let swimmer = place_mobile(&mut w, MapId::FELUCCA, start);
    w.mobile_mut(swimmer).unwrap().can_swim = true;
    assert_eq!(find_path(&mut w, swimmer, goal), [NW, NW]);
    assert_eq!(find_path(&mut w, swimmer, p(1497, 1599, Z)), [NW, W, W]);

    let mut w = world();
    let player = place_mobile(&mut w, MapId::FELUCCA, start);
    w.mobile_mut(player).unwrap().player = true;
    assert_eq!(find_path(&mut w, player, goal), [NW, NW]);

    for ty in [&FLYING, &DOOR_OPENER, &CLIMBER] {
        let mut w = world();
        let c = stub_creature(&mut w, ty, start);
        assert_eq!(find_path(&mut w, c, goal), [NW, NW], "{}", ty.full_name);
    }
}

/// The route round something on the direct line (1499, 1599) goes N,W or W,N, never through it.
fn assert_goes_round(w: &mut World, m: MobileId) {
    let start = p(1500, 1600, Z);
    let dirs = find_path(w, m, p(1498, 1598, Z));
    assert!(!dirs.is_empty());
    let cells = cells_of(start, &dirs);
    assert!(
        !cells.contains(&(1499, 1599)),
        "path traversed the blocked cell: {cells:?}"
    );
    assert_eq!(cells.last(), Some(&(1498, 1598)));
}

#[test]
fn dynamic_obstacle_pass_rejects_a_cell_occupied_by_a_living_mobile() {
    let mut w = world();
    let walker = place_mobile(&mut w, MapId::FELUCCA, p(1500, 1600, Z));
    place_mobile(&mut w, MapId::FELUCCA, p(1499, 1599, Z));
    assert_goes_round(&mut w, walker);
}

#[test]
fn dynamic_obstacle_pass_rejects_a_cell_occupied_by_an_impassable_item() {
    let mut w = world();
    let walker = place_mobile(&mut w, MapId::FELUCCA, p(1500, 1600, Z));
    place_item(&mut w, MapId::FELUCCA, p(1499, 1599, Z), i32::from(COUNTER));
    assert_goes_round(&mut w, walker);
}

// ---- Engines/Pathing/Multi ----------------------------------------------------------------------------------
//
// Not covered (they test the cache and its multi mask, not the walk): MultiCacheUsedTests and MultiMaskCacheTests
// (hit counters, local indices, Z packing), MultiMaskSynthesisTests (the synthesizer against CheckMovement).
// MultiEdgeCaseTests keep their geometry and run the CheckMovement side of the sweep alone; the
// `StepCache.TryGetMask(..).HitKind == Fallthrough_Multi` asserts inside BoatDeck_, RedesignReRegistered_,
// PlacedMulti_ and CleanMap_ are dropped (what is left of each: registration and walkability).

const HOUSE_Z: i16 = 1;

/// `TestMultis.WalledHouse`.
fn walled_house(radius: i16, doorway_on_south: bool) -> Vec<MultiTileEntry> {
    let mut v = Vec::new();
    for x in -radius..=radius {
        for y in -radius..=radius {
            let edge = x == -radius || x == radius || y == -radius || y == radius;
            let doorway = if doorway_on_south {
                x == 0 && y == radius
            } else {
                x == radius && y == 0
            };
            let id = if edge && !doorway { WALL } else { FLOOR };
            v.push(entry(id, x, y, HOUSE_Z, TileFlag::BACKGROUND));
        }
    }
    v
}

/// `TestMultis.Boat`.
fn boat() -> Vec<MultiTileEntry> {
    let mut v = Vec::new();
    for x in -2..=2 {
        for y in -3..=3 {
            v.push(entry(
                if x == -2 || x == 2 { WALL } else { FLOOR },
                x,
                y,
                0,
                TileFlag::BACKGROUND,
            ));
        }
    }
    v
}

fn guild_house() -> Vec<MultiTileEntry> {
    walled_house(3, true)
}

fn tower() -> Vec<MultiTileEntry> {
    walled_house(4, false)
}

fn components(w: &World, multi: ItemId) -> MultiComponentList {
    w.item(multi)
        .unwrap()
        .ext
        .get::<MultiComponentList>()
        .unwrap()
        .clone()
}

type Cell = (i32, i32);

fn tile_flags(w: &World, id: u16) -> TileFlag {
    w.maps.tile_data.item(usize::from(id)).flags
}

/// `MultiArt.FootprintCells`.
fn footprint(w: &World, multi: ItemId) -> Vec<Cell> {
    let (mcl, at) = (components(w, multi), w.item(multi).unwrap().location);
    let mut v = Vec::new();
    for lx in 0..mcl.width() {
        for ly in 0..mcl.height() {
            if !mcl.tiles[lx as usize][ly as usize].is_empty() {
                v.push((at.x + mcl.min().x + lx, at.y + mcl.min().y + ly));
            }
        }
    }
    v
}

/// `MultiArt.FootprintWithHalo`.
fn footprint_with_halo(w: &World, multi: ItemId) -> std::collections::BTreeSet<Cell> {
    let mut set = std::collections::BTreeSet::new();
    for (x, y) in footprint(w, multi) {
        for dx in -1..=1 {
            for dy in -1..=1 {
                set.insert((x + dx, y + dy));
            }
        }
    }
    set
}

/// `FindWallCell` (`wall`: impassable and not a surface) and `FindFloorCell` (a surface that isn't impassable).
fn find_cell(w: &World, multi: ItemId, wall: bool) -> Option<Cell> {
    let (mcl, at) = (components(w, multi), w.item(multi).unwrap().location);
    for lx in 0..mcl.width() {
        for ly in 0..mcl.height() {
            for t in &mcl.tiles[lx as usize][ly as usize] {
                let f = tile_flags(w, t.id);
                let hit = if wall {
                    f.contains(TileFlag::IMPASSABLE) && !f.contains(TileFlag::SURFACE)
                } else {
                    f.contains(TileFlag::SURFACE) && !f.contains(TileFlag::IMPASSABLE)
                };
                if hit {
                    return Some((at.x + mcl.min().x + lx, at.y + mcl.min().y + ly));
                }
            }
        }
    }
    None
}

fn has_multi_tile_at(w: &World, multi: ItemId, x: i32, y: i32) -> bool {
    let (mcl, at) = (components(w, multi), w.item(multi).unwrap().location);
    let (lx, ly) = (x - at.x + mcl.center().x, y - at.y + mcl.center().y);
    lx >= 0
        && ly >= 0
        && lx < mcl.width()
        && ly < mcl.height()
        && !mcl.tiles[lx as usize][ly as usize].is_empty()
}

fn sector_has_multis(w: &World, x: i32, y: i32) -> bool {
    w.maps
        .get(MapId::TRAMMEL)
        .unwrap()
        .sector(x >> 4, y >> 4)
        .has_multis()
}

fn walker(w: &mut World, at: Point3D) -> MobileId {
    place_mobile(w, MapId::TRAMMEL, at)
}

fn step_ok(w: &mut World, from: Point3D, d: Direction, m: MobileId) -> (bool, i32) {
    IMPL.check_movement_at(w, m, MapId::TRAMMEL, from, d)
}

fn dir(d: u8) -> Direction {
    Direction::from_bits_retain(d)
}

/// `AssertSynthesizerMatchesCheckMovement` without the synthesizer: sweeps the footprint and its halo and checks
/// what the sweep checks of the oracle (it saw multi cells, walkable and blocked transitions).
fn sweep(w: &mut World, multi: ItemId) {
    let at = w.item(multi).unwrap().location;
    let mover = walker(w, at);
    let (mut touched, mut walkable, mut blocked) = (0, 0, 0);
    for (cx, cy) in footprint_with_halo(w, multi) {
        let from = p(cx, cy, at.z);
        if has_multi_tile_at(w, multi, cx, cy) {
            touched += 1;
        }
        for d in 0..8 {
            if step_ok(w, from, dir(d), mover).0 {
                walkable += 1;
            } else {
                blocked += 1;
            }
        }
    }
    assert!(touched > 0, "sweep touched no multi-covered cells");
    assert!(walkable > 0, "sweep observed no walkable transitions");
    assert!(blocked > 0, "sweep observed no blocked transitions");
}

#[test]
fn boat_deck_has_walkable_surface_cells() {
    let mut w = world();
    let boat = place_multi(&mut w, MapId::TRAMMEL, p(352, 288, 0), &boat());
    let floor =
        find_cell(&w, boat, false).expect("non-vacuity: boat deck must have surface (floor) tiles");
    assert!(
        sector_has_multis(&w, floor.0, floor.1),
        "the deck's sector knows the boat"
    );
}

#[test]
fn redesign_re_registered_routes_the_new_footprint() {
    let mut w = world();
    let loc = p(1500, 1600, 0);
    let foundation = place_multi(&mut w, MapId::TRAMMEL, loc, &guild_house());
    let seeded_shape = {
        let c = components(&w, foundation);
        (c.width(), c.height(), c.list.len())
    };
    let before = find_cell(&w, foundation, false)
        .expect("non-vacuity: initial design must have a floor cell");
    assert!(sector_has_multis(&w, before.0, before.1));

    // Internalize, swap the components, MoveToWorld back: the sectors follow the new footprint.
    muo_server::item::ops::internalize(&mut w, foundation);
    w.item_mut(foundation).unwrap().ext =
        muo_server::Ext::new(MultiComponentList::from_entries(&tower()));
    muo_server::item::ops::move_to_world_map(&mut w, foundation, loc, Some(MapId::TRAMMEL));

    let after = find_cell(&w, foundation, false)
        .expect("non-vacuity: redesigned design must have a floor cell");
    assert!(
        sector_has_multis(&w, after.0, after.1),
        "redesigned footprint must re-register its sector"
    );
    // The redesign changed this foundation's list; a GuildHouse placed afterwards keeps the shared footprint.
    let other = place_multi(&mut w, MapId::TRAMMEL, p(1520, 1600, 0), &guild_house());
    let c = components(&w, other);
    assert_eq!(seeded_shape, (c.width(), c.height(), c.list.len()));
    assert_ne!(components(&w, foundation).width(), c.width());
}

/// `Replays path through CheckMovement from start.`
fn assert_every_step_walkable(w: &mut World, m: MobileId, start: Point3D, path: &[Direction]) {
    let mut at = start;
    for &d in path {
        let (ok, z) = step_ok(w, at, d, m);
        assert!(ok, "step {d:?} from {at:?} is refused");
        offset(d, &mut at, 1);
        at.z = z;
    }
}

/// `PathAround_NeverTraversesAWallCell` and `Find_CacheOn_EqualsCacheOff_RoutesAroundHouse` (the same start, goal
/// and house; the cache-off half is the only search here, so it is run twice).
#[test]
fn path_around_never_traverses_a_wall_cell() {
    let mut w = world();
    let multi = place_multi(&mut w, MapId::TRAMMEL, p(1480, 1620, 0), &guild_house());
    let wall = find_cell(&w, multi, true).expect("non-vacuity: house must have wall cells");
    let (start, goal) = (p(1480, 1630, 0), p(1480, 1610, 0));
    let m = walker(&mut w, start);
    let path = find_path(&mut w, m, goal);
    assert!(!path.is_empty(), "non-vacuity: a route around must exist");
    let cells = cells_of(start, &path);
    assert!(!cells.contains(&wall), "path traversed wall cell {wall:?}");
    assert_eq!(cells.last(), Some(&(1480, 1610)));
    assert_eq!(
        find_path(&mut w, m, goal),
        path,
        "the same search repeats exactly"
    );
    assert_every_step_walkable(&mut w, m, start, &path);
}

/// `Find_CacheOn_EqualsCacheOff_WithHousePresent` [2]: the house between start and goal. A cache would be compared
/// with the slow path; here the search is the slow path, so what is left is that it agrees with itself on a second
/// run and that its route is one `CheckMovement` allows from step to step.
#[test]
fn a_path_across_a_house_is_repeatable_and_walkable() {
    for (sx, sy, gx, gy) in [(1500, 1600, 1500, 1612), (1494, 1606, 1512, 1606)] {
        let mut w = world();
        place_multi(
            &mut w,
            MapId::TRAMMEL,
            p((sx + gx) / 2, (sy + gy) / 2, 0),
            &guild_house(),
        );
        let (start, goal) = (p(sx, sy, 0), p(gx, gy, 0));
        let m = walker(&mut w, start);
        let first = find_path(&mut w, m, goal);
        assert_eq!(
            find_path(&mut w, m, goal),
            first,
            "({sx},{sy}) -> ({gx},{gy})"
        );
        assert_every_step_walkable(&mut w, m, start, &first);
    }
}

/// True if EVERY neighbour fails to step into `target`.
fn neighbours_blocked_into(w: &mut World, m: MobileId, target: Cell) -> bool {
    for d in 0..8u8 {
        let mut from = p(target.0, target.1, 0);
        offset(dir((d + 4) & 7), &mut from, 1);
        muo_server::mobile::base::move_to_world(w, m, from, MapId::TRAMMEL);
        if step_ok(w, from, dir(d), m).0 {
            let mut to = from;
            offset(dir(d), &mut to, 1);
            if (to.x, to.y) == target {
                return false;
            }
        }
    }
    true
}

#[test]
fn demolish_reopens_covered_cells() {
    let mut w = world();
    let multi = place_multi(&mut w, MapId::TRAMMEL, p(1480, 1620, 0), &guild_house());
    // A wall cell with no surface in its stack, over passable terrain (the flat ground is).
    let mcl = components(&w, multi);
    let mut cell = None;
    'find: for lx in 0..mcl.width() {
        for ly in 0..mcl.height() {
            let stack = &mcl.tiles[lx as usize][ly as usize];
            let wall = stack.iter().any(|t| {
                let f = tile_flags(&w, t.id);
                f.contains(TileFlag::IMPASSABLE) && !f.contains(TileFlag::SURFACE)
            });
            let surface = stack
                .iter()
                .any(|t| tile_flags(&w, t.id).contains(TileFlag::SURFACE));
            if wall && !surface {
                cell = Some((1480 + mcl.min().x + lx, 1620 + mcl.min().y + ly));
                break 'find;
            }
        }
    }
    let cell = cell.expect("non-vacuity: house must have a wall cell over passable terrain");
    let m = walker(&mut w, p(cell.0, cell.1, 0));

    assert!(
        neighbours_blocked_into(&mut w, m, cell),
        "wall cell should block entry while the house stands"
    );
    delete_item(&mut w, multi);
    assert!(
        !neighbours_blocked_into(&mut w, m, cell),
        "cell {cell:?} should reopen after demolish"
    );
}

#[test]
fn wall_cell_cannot_be_entered_from_any_adjacent_cell() {
    let mut w = world();
    let multi = place_multi(&mut w, MapId::TRAMMEL, p(1500, 1600, 0), &guild_house());
    let m = walker(&mut w, p(1500, 1600, 0));
    let wall = find_cell(&w, multi, true).expect("non-vacuity: house MCL must contain a wall tile");
    let (mut entries, mut successful) = (0, 0);
    for around in 0..8 {
        let mut from = p(wall.0, wall.1, 0);
        offset(dir(around), &mut from, 1);
        for d in 0..8 {
            if !step_ok(&mut w, from, dir(d), m).0 {
                continue;
            }
            successful += 1;
            let mut to = from;
            offset(dir(d), &mut to, 1);
            if (to.x, to.y) == wall {
                entries += 1;
            }
        }
    }
    assert!(
        successful > 0,
        "non-vacuity: walker must be able to move near the wall"
    );
    assert_eq!(entries, 0);
}

#[test]
fn floor_cell_is_standable() {
    let mut w = world();
    let multi = place_multi(&mut w, MapId::TRAMMEL, p(1500, 1600, 0), &guild_house());
    let m = walker(&mut w, p(1500, 1600, 0));
    let floor =
        find_cell(&w, multi, false).expect("non-vacuity: house MCL must contain a floor tile");
    let mut entered = false;
    for d in 0..8 {
        let mut from = p(floor.0, floor.1, 0);
        offset(dir((d + 4) & 7), &mut from, 1);
        let mut to = from;
        offset(dir(d), &mut to, 1);
        if step_ok(&mut w, from, dir(d), m).0 && (to.x, to.y) == floor {
            entered = true;
            break;
        }
    }
    assert!(
        entered,
        "expected the floor cell {floor:?} to be reachable from a neighbour"
    );
}

#[test]
fn placed_multi_sets_the_sectors_flag_and_deleting_it_clears_it() {
    // PlacedMulti_RoutesFootprintAndHalo_ToFallthroughMulti (the HasMultis half) and CleanMap_AfterMultiRemoved.
    let mut w = world();
    assert!(!sector_has_multis(&w, 1500, 1600));
    let multi = place_multi(&mut w, MapId::TRAMMEL, p(1500, 1600, 0), &guild_house());
    assert!(
        sector_has_multis(&w, 1500, 1600),
        "placing a multi must set Sector.HasMultis"
    );
    assert!(!footprint_with_halo(&w, multi).is_empty());
    delete_item(&mut w, multi);
    assert!(
        !sector_has_multis(&w, 1500, 1600),
        "removing the only multi must clear HasMultis"
    );
}

#[test]
fn overlapping_multis_sweep() {
    // OverlappingMultis_SynthesizerMatchesCheckMovement: origins 3 apart so the footprints intersect.
    let mut w = world();
    let a = place_multi(&mut w, MapId::TRAMMEL, p(1480, 1620, 0), &guild_house());
    let b = place_multi(&mut w, MapId::TRAMMEL, p(1483, 1620, 0), &guild_house());
    let set_b: std::collections::BTreeSet<Cell> = footprint(&w, b).into_iter().collect();
    assert!(
        footprint(&w, a).iter().any(|c| set_b.contains(c)),
        "the two footprints must actually intersect"
    );
    sweep(&mut w, a);
    sweep(&mut w, b);
}

#[test]
fn boat_over_water_sweep() {
    // BoatOverWater_SynthesizerMatchesCheckMovement: the deck stands at world Z 0.
    let mut w = world();
    let boat = place_multi(&mut w, MapId::TRAMMEL, p(352, 288, 0), &boat());
    sweep(&mut w, boat);
}

#[test]
fn redesigned_foundation_sweep() {
    // RedesignedFoundation_SynthesizerMatchesNewFootprint.
    let mut w = world();
    let loc = p(1480, 1620, 0);
    let f = place_multi(&mut w, MapId::TRAMMEL, loc, &guild_house());
    muo_server::item::ops::internalize(&mut w, f);
    w.item_mut(f).unwrap().ext = muo_server::Ext::new(MultiComponentList::from_entries(&tower()));
    muo_server::item::ops::move_to_world_map(&mut w, f, loc, Some(MapId::TRAMMEL));
    sweep(&mut w, f);
}

// ---- Terrain/TerrainScopeCacheTests ----
//
// These tests are about `StepCache` serving a mask built before terrain was stamped. There is no step cache:
// every query reads the tile matrix, so the same assertions check that a stamp is seen at
// once, inside a scope and after the scope ends.

fn north_walkable(w: &mut World, m: MobileId, x: i32, y: i32) -> bool {
    IMPL.check_movement_at(w, m, MapId::TRAMMEL, p(x, y, 0), Direction::NORTH)
        .0
}

#[test]
fn terrain_stamped_in_a_scope_is_not_served_stale() {
    // Default ground, clear of every prefab.
    let (x, y) = (1300, 1300);
    let mut w = world();
    let m = walker(&mut w, p(x, y, 0));
    assert!(north_walkable(&mut w, m, x, y));

    // `using (TestTerrain.Scope())`: the stamp is undone when the scope ends.
    let saved = w
        .maps
        .tiles_mut(MapId::TRAMMEL)
        .unwrap()
        .get_static_block((x) >> 3, (y - 1) >> 3)
        .clone();
    add_static(&mut w, MapId::TRAMMEL, x, y - 1, WALL, 0);
    assert!(!north_walkable(&mut w, m, x, y));
    w.maps
        .tiles_mut(MapId::TRAMMEL)
        .unwrap()
        .set_static_block((x) >> 3, (y - 1) >> 3, saved);

    assert!(north_walkable(&mut w, m, x, y));
}

#[test]
fn terrain_stamped_after_a_query_inside_a_scope_is_not_served_stale() {
    let (x, y) = (1300, 1300);
    let mut w = world();
    let m = walker(&mut w, p(x, y, 0));
    assert!(north_walkable(&mut w, m, x, y));

    add_static(&mut w, MapId::TRAMMEL, x, y - 1, WALL, 0);

    assert!(!north_walkable(&mut w, m, x, y));
}
