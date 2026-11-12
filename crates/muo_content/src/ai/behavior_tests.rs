//! AI behavior tests: the creatures are `PetTestStub` (an AnimalAI dog with fixed speeds), a bare
//! `AnimalAI` follower and plain walkers, on the invented tile kit of `muo_server::map::test_support`.

use muo_primitives::{Body, Direction, Point3D};
use muo_server::World;
use muo_server::ids::MobileId;
use muo_server::map::MapId;
use muo_server::mobile::MOBILE_BASE;

use super::*;
use crate::creature::control::*;
use crate::creature::data::cd;
use crate::creature::enums::{AIType, FightMode};
use crate::creature::setters::new_creature;
use crate::creature::speeds;
use crate::creature::vtable::{BASE_CREATURE, BASE_CREATURE_C};
use muo_server::mobile::MobileType;

static PET_STUB_C: crate::creature::vtable::CreatureType =
    crate::creature::vtable::CreatureType { ..BASE_CREATURE_C };
static PET_STUB: MobileType = MobileType {
    full_name: "Server.Tests.PetTestStub",
    parent: Some(&BASE_CREATURE),
    family: Some(&PET_STUB_C),
    ..BASE_CREATURE
};

/// A world with the invented tile kit, and the NPC speed table pinned to what `PetTestStub.GetSpeeds` returns.
fn world() -> World {
    let mut w = muo_server::map::test_support::world();
    super::configure_for_tests(&mut w);
    w
}

/// `new PetTestStub()`: AI_Animal, FightMode.Closest, perception 10, fight 1, body 0xC8 (a dog).
fn pet_stub(w: &mut World) -> MobileId {
    let id = new_creature(w, &PET_STUB, AIType::AiAnimal, FightMode::Closest, 10, 1);
    w.mobile_mut(id).unwrap().body = Body(0xC8);
    id
}

fn place(w: &mut World, m: MobileId, x: i32, y: i32, map: MapId) {
    muo_server::mobile::base::move_to_world(w, m, Point3D::new(x, y, 0), map);
}

/// A plain player mobile that is a valid friend.
fn real_player(w: &mut World) -> MobileId {
    crate::player_mobile::new_player_mobile(w)
}

// ---- PetFriendMembershipTests ----

#[test]
fn add_pet_friend_twice_stores_one_membership() {
    let mut w = world();
    let (pet, friend) = (pet_stub(&mut w), real_player(&mut w));
    add_pet_friend(&mut w, pet, friend);
    add_pet_friend(&mut w, pet, friend);
    assert_eq!(cd(&w, pet).unwrap().friends.as_ref().map(Vec::len), Some(1));
    assert!(is_pet_friend(&w, pet, friend));
}

#[test]
fn remove_pet_friend_after_repeated_add_revokes_membership() {
    let mut w = world();
    let (pet, friend) = (pet_stub(&mut w), real_player(&mut w));
    add_pet_friend(&mut w, pet, friend);
    add_pet_friend(&mut w, pet, friend);
    remove_pet_friend(&mut w, pet, friend);
    assert!(!is_pet_friend(&w, pet, friend));
    assert!(cd(&w, pet).unwrap().friends.is_none());
}

#[test]
fn remove_and_readd_leaves_other_friends_unchanged() {
    let mut w = world();
    let (pet, first, second) = (pet_stub(&mut w), real_player(&mut w), real_player(&mut w));
    add_pet_friend(&mut w, pet, first);
    add_pet_friend(&mut w, pet, second);
    remove_pet_friend(&mut w, pet, first);

    assert!(!is_pet_friend(&w, pet, first));
    assert!(is_pet_friend(&w, pet, second));
    assert_eq!(cd(&w, pet).unwrap().friends.as_ref().map(Vec::len), Some(1));

    add_pet_friend(&mut w, pet, first);
    assert!(is_pet_friend(&w, pet, first));
    assert!(is_pet_friend(&w, pet, second));
    assert_eq!(cd(&w, pet).unwrap().friends.as_ref().map(Vec::len), Some(2));
}

// ---- AIDeactivationTests ----

#[test]
fn deactivate_without_world_map_stops_timer() {
    for (internal_map, controlled) in [(false, false), (false, true), (true, false), (true, true)] {
        let mut w = world();
        let creature = pet_stub(&mut w);
        set_controlled(&mut w, creature, controlled);
        w.mobile_mut(creature).unwrap().map = internal_map.then_some(MapId::INTERNAL);
        activate(&mut w, creature);
        assert!(
            timer::running(&w, creature),
            "internal {internal_map} controlled {controlled}"
        );

        deactivate(&mut w, creature);

        assert!(
            !timer::running(&w, creature),
            "internal {internal_map} controlled {controlled}"
        );
    }
}

// ---- RunFlagTests (the ones that do not step over terrain) ----

fn spawn_runner(w: &mut World, active_move: f64) -> MobileId {
    let pet = pet_stub(w);
    place(w, pet, 1000, 1000, MapId::FELUCCA);
    timer::pause(w, pet);
    // 0 hits, 0 stam, as the stub starts: the fixture runs Expansion.EJ (HS), so BadlyHurtMoveDelay reads
    // Stam against StamMax = Dex clamped 0..65000 = 0, and `statMax > 0` keeps the stub from counting as hurt.
    speeds::set_move_speed(w, pet, active_move, active_move * 3.0);
    speeds::set_current_speed_to_active(w, pet);
    w.mobile_mut(pet).unwrap().last_move_time = w.tick_count; // mid-cadence unless a test says otherwise
    pet
}

#[test]
fn foot_creature_runs_only_when_faster_than_walk() {
    for (active_move, expected) in [
        (0.3, true),
        (0.125, true),
        (0.4, false),
        (0.45, false),
        (1.05, false),
    ] {
        let mut w = world();
        let pet = spawn_runner(&mut w, active_move);
        assert_eq!(speeds::current_move_speed(&w, pet), active_move);
        assert_eq!(movement::should_run(&mut w, pet), expected, "{active_move}");
    }
}

#[test]
fn flying_creature_uses_mount_thresholds() {
    for (active_move, expected) in [(0.3, false), (0.15, true)] {
        let mut w = world();
        let pet = spawn_runner(&mut w, active_move);
        w.mobile_mut(pet).unwrap().flying = true;
        assert_eq!(movement::should_run(&mut w, pet), expected, "{active_move}");
    }
}

#[test]
fn badly_hurt_slows_below_walk_drops_to_walk() {
    let mut w = world();
    let pet = spawn_runner(&mut w, 0.35);
    assert!(movement::should_run(&mut w, pet));

    // The hurt inflation is on the observed step pace, so the flag follows it.
    crate::creature::setters::set_hits(&mut w, pet, 100);
    w.mobile_mut(pet).unwrap().hits = 5;
    crate::creature::setters::set_stam(&mut w, pet, 100);
    w.mobile_mut(pet).unwrap().hits = 5;
    w.mobile_mut(pet).unwrap().stam = 5;

    assert!(!movement::should_run(&mut w, pet));
}

// An isolated step (after standing at least a walk interval) renders alone and darts if run-flagged, so it walks;
// continuing cadences and true sprinters keep the flag.
#[test]
fn isolated_step_drops_to_walk() {
    let mut w = world();
    let pet = spawn_runner(&mut w, 0.3);
    w.mobile_mut(pet).unwrap().last_move_time = w.tick_count - 1000;
    assert!(!movement::should_run(&mut w, pet));
}

#[test]
fn isolated_step_sprinter_still_runs() {
    let mut w = world();
    let pet = spawn_runner(&mut w, 0.125);
    w.mobile_mut(pet).unwrap().last_move_time = w.tick_count - 1000;
    assert!(movement::should_run(&mut w, pet));
}

#[test]
fn do_move_stamps_running_bit() {
    for (active_move, expected) in [(0.3, true), (0.45, false)] {
        let mut w = world();
        let pet = spawn_runner(&mut w, active_move);
        place(&mut w, pet, 1500, 1600, MapId::TRAMMEL);
        set_next_move(&mut w, pet, 0);
        let start = w.mobile(pet).unwrap().location;

        assert!(movement::do_move(&mut w, pet, Direction::WEST, false));
        assert_ne!(w.mobile(pet).unwrap().location, start);
        assert_eq!(
            w.mobile(pet)
                .unwrap()
                .direction
                .contains(Direction::RUNNING),
            expected,
            "{active_move}"
        );
    }
}

#[test]
fn stall_does_not_bank_catch_up_steps() {
    let mut w = world();
    let pet = spawn_runner(&mut w, 0.3);
    place(&mut w, pet, 1500, 1600, MapId::TRAMMEL);
    let tick = w.tick_count;
    set_next_move(&mut w, pet, tick - 1000);

    assert!(movement::do_move(&mut w, pet, Direction::WEST, false));

    // A stall must restart the cadence at full pace: banked catch-up steps release as a burst the client renders as
    // a sprint/teleport.
    assert!(!movement::can_move_now(&w, pet));
    assert!(ai(&w, pet).unwrap().next_move - w.tick_count > 250);
}

#[test]
fn late_step_does_not_earn_a_quicker_follow_up() {
    let mut w = world();
    let pet = spawn_runner(&mut w, 0.3);
    place(&mut w, pet, 1500, 1600, MapId::TRAMMEL);
    w.mobile_mut(pet).unwrap().warmode = true; // keep the active move clock through the step

    // The step lands 200ms past the budget - under one period, the reactive mirroring case.
    let tick = w.tick_count;
    set_next_move(&mut w, pet, tick - 200);

    assert!(movement::do_move(&mut w, pet, Direction::WEST, false));

    // The debt must not be repaid: a sub-period catch-up step follows ~100ms behind and renders as a dart pair.
    assert!(ai(&w, pet).unwrap().next_move - w.tick_count > 250);
}

// ---- AcquisitionTests ----

fn wild_stub(w: &mut World) -> MobileId {
    let id = new_creature(w, &PET_STUB, AIType::AiMelee, FightMode::Closest, 16, 1);
    w.mobile_mut(id).unwrap().body = Body(0xC9);
    speeds::set_speed(w, id, 0.3, 0.6, true);
    id
}

static INSTANT_C: crate::creature::vtable::CreatureType = crate::creature::vtable::CreatureType {
    acquire_on_approach_delay: |_, _| muo_primitives::TimeSpan::ZERO,
    ..BASE_CREATURE_C
};
static INSTANT_STUB: MobileType = MobileType {
    full_name: "Server.Tests.InstantStub",
    parent: Some(&BASE_CREATURE),
    family: Some(&INSTANT_C),
    ..BASE_CREATURE
};

/// `TargetStub`: a plain Mobile with body 0x190.
fn target_stub(w: &mut World, x: i32, player: bool) -> MobileId {
    let m = w.new_mobile(&MOBILE_BASE);
    {
        let mm = w.mobile_mut(m).unwrap();
        mm.body = Body(0x190);
        mm.player = player;
    }
    place(w, m, x, 1600, MapId::TRAMMEL);
    m
}

fn spawn_wild(w: &mut World, x: i32) -> MobileId {
    let bc = wild_stub(w);
    place(w, bc, x, 1600, MapId::TRAMMEL);
    timer::pause(w, bc);
    bc
}

fn reacquire_delay_ms(w: &World, bc: MobileId) -> i64 {
    (ctype(w, bc).unwrap().reacquire_delay)(w, bc).total_milliseconds() as i64
}

fn focus(w: &World, bc: MobileId) -> Option<MobileId> {
    cd(w, bc).unwrap().focus_mob
}

fn acquire(w: &mut World, bc: MobileId) -> bool {
    let (rp, mode) = {
        let d = cd(w, bc).unwrap();
        (d.range_perception, d.fight_mode)
    };
    base::acquire_focus_mob(w, bc, rp, mode, false, false, true)
}

fn set_next_reacquire(w: &mut World, bc: MobileId, v: i64) {
    crate::creature::data::cd_mut(w, bc)
        .unwrap()
        .next_reacquire_time = v;
}

fn next_reacquire(w: &World, bc: MobileId) -> i64 {
    cd(w, bc).unwrap().next_reacquire_time
}

#[test]
fn empty_scan_honors_reacquire_delay() {
    let mut w = world();
    let bc = spawn_wild(&mut w, 1500);
    let tick = w.tick_count;
    set_next_reacquire(&mut w, bc, tick);

    assert!(!acquire(&mut w, bc));
    assert_eq!(
        reacquire_delay_ms(&w, bc),
        next_reacquire(&w, bc) - w.tick_count
    );
}

#[test]
fn wedged_gate_self_heals() {
    let mut w = world();
    let bc = spawn_wild(&mut w, 1500);
    let target = target_stub(&mut w, 1497, false);

    // Illegal deadline (beyond ReacquireDelay): must read as open, not block forever.
    let v = w.tick_count + reacquire_delay_ms(&w, bc) * 6;
    set_next_reacquire(&mut w, bc, v);

    assert!(acquire(&mut w, bc));
    assert_eq!(focus(&w, bc), Some(target));
}

#[test]
fn movement_clamps_scan_deadline_only_for_enemies_in_range() {
    // (wild mover, distance, notices)
    for (wild_mover, distance, notices) in [
        (false, 5, true),   // an enemy moving inside approach range (10) clamps the deadline
        (true, 5, false),   // a same-team wild creature is not an enemy - ignored
        (false, 12, false), // inside RangePerception but outside approach range - poll only
        (false, 20, false), // outside approach range (10) is ignored
    ] {
        let mut w = world();
        let bc = spawn_wild(&mut w, 1500);
        let armed = w.tick_count + reacquire_delay_ms(&w, bc); // ArmFullDelay
        set_next_reacquire(&mut w, bc, armed);

        let mover = if wild_mover {
            spawn_wild(&mut w, 1500 - distance)
        } else {
            target_stub(&mut w, 1500 - distance, true)
        };

        crate::creature::overrides::on_movement(&mut w, bc, mover, Point3D::new(1400, 1600, 0));

        let remaining = next_reacquire(&w, bc) - w.tick_count;
        if notices {
            // Clamped to the approach delay, never opened outright.
            let approach = (ctype(&w, bc).unwrap().acquire_on_approach_delay)(&w, bc)
                .total_milliseconds() as i64;
            assert!((1..=approach).contains(&remaining), "{remaining}");
        } else {
            assert_eq!(
                next_reacquire(&w, bc),
                armed,
                "wild {wild_mover} distance {distance}"
            );
        }
    }
}

#[test]
fn zero_approach_delay_opens_gate_immediately() {
    let mut w = world();
    let bc = new_creature(
        &mut w,
        &INSTANT_STUB,
        AIType::AiMelee,
        FightMode::Closest,
        16,
        1,
    );
    w.mobile_mut(bc).unwrap().body = Body(0xC9);
    speeds::set_speed(&mut w, bc, 0.3, 0.6, true);
    place(&mut w, bc, 1500, 1600, MapId::TRAMMEL);
    timer::pause(&mut w, bc);
    let armed = w.tick_count + reacquire_delay_ms(&w, bc);
    set_next_reacquire(&mut w, bc, armed);

    let mover = target_stub(&mut w, 1495, true);
    crate::creature::overrides::on_movement(&mut w, bc, mover, Point3D::new(1400, 1600, 0));

    // Zero = the gate opens and the AI is prodded to think now; no direct engage.
    assert!(w.tick_count - next_reacquire(&w, bc) >= 0);
    assert_eq!(w.mobile(bc).unwrap().combatant, None);
    assert!(timer::running(&w, bc));
}

#[test]
fn repeated_movement_does_not_shorten_below_approach_delay() {
    let mut w = world();
    let bc = spawn_wild(&mut w, 1500);
    let armed = w.tick_count + reacquire_delay_ms(&w, bc);
    set_next_reacquire(&mut w, bc, armed);

    let mover = target_stub(&mut w, 1495, true);
    crate::creature::overrides::on_movement(&mut w, bc, mover, Point3D::new(1400, 1600, 0));
    let after_first = next_reacquire(&w, bc);

    crate::creature::overrides::on_movement(&mut w, bc, mover, Point3D::new(1496, 1600, 0));

    assert_eq!(next_reacquire(&w, bc), after_first);
}

#[test]
fn successful_acquire_holds_full_delay() {
    let mut w = world();
    let bc = spawn_wild(&mut w, 1500);
    let target = target_stub(&mut w, 1497, false);
    let tick = w.tick_count;
    set_next_reacquire(&mut w, bc, tick);

    assert!(acquire(&mut w, bc));
    assert_eq!(focus(&w, bc), Some(target));
    assert_eq!(
        reacquire_delay_ms(&w, bc),
        next_reacquire(&w, bc) - w.tick_count
    );
}

// ---- ApproachOutcomeTests / ApproachTargetTests / HerdingTests ----

fn new_follower(w: &mut World, x: i32, y: i32) -> MobileId {
    let bc = pet_stub(w);
    place(w, bc, x, y, MapId::TRAMMEL);
    timer::pause(w, bc); // we drive movement manually; no background timer
    bc
}

fn new_walker(w: &mut World, x: i32, y: i32) -> MobileId {
    let t = w.new_mobile(&MOBILE_BASE);
    w.mobile_mut(t).unwrap().body = Body(0xC9);
    place(w, t, x, y, MapId::TRAMMEL);
    t
}

fn counter_at(w: &mut World, x: i32, y: i32) {
    muo_server::map::test_support::add_static(
        w,
        MapId::TRAMMEL,
        x,
        y,
        muo_server::map::test_support::COUNTER,
        0,
    );
}

fn in_range(w: &World, a: MobileId, b: MobileId, range: i32) -> bool {
    let (la, lb) = (w.mobile(a).unwrap().location, w.mobile(b).unwrap().location);
    (la.x - lb.x).abs().max((la.y - lb.y).abs()) <= range
}

fn last_approach(w: &World, bc: MobileId) -> ApproachOutcome {
    ai(w, bc).unwrap().last_approach
}

/// Drives the pet follow primitive once per tick; true once it reaches within `arrive` tiles (Chebyshev, like the
/// game's own range semantics).
fn drive_follow(
    w: &mut World,
    bc: MobileId,
    target: MobileId,
    arrive: i32,
    max_ticks: i32,
) -> bool {
    for _ in 0..max_ticks {
        set_next_move(w, bc, 0);
        movement::walk_mobile_range(w, bc, target, 1, 1, 2);
        if in_range(w, bc, target, arrive) {
            return true;
        }
    }
    false
}

#[test]
fn open_ground_reports_direct_progress_then_arrived() {
    let mut w = world();
    let bc = new_follower(&mut w, 1500, 1600);
    let target = new_walker(&mut w, 1497, 1600);

    set_next_move(&mut w, bc, 0);
    movement::move_to(&mut w, bc, target, 1);
    assert_eq!(last_approach(&w, bc), ApproachOutcome::DirectProgress);

    movement::move_to(&mut w, bc, target, 1); // budget consumed: no step
    assert_eq!(last_approach(&w, bc), ApproachOutcome::Waiting);

    set_next_move(&mut w, bc, 0);
    movement::move_to(&mut w, bc, target, 1); // lands adjacent: still a progress step
    assert_eq!(last_approach(&w, bc), ApproachOutcome::DirectProgress);

    set_next_move(&mut w, bc, 0);
    movement::move_to(&mut w, bc, target, 1);
    assert_eq!(last_approach(&w, bc), ApproachOutcome::Arrived);
}

fn ring_around(w: &mut World, cx: i32, cy: i32) {
    // Impassable ring around the target: unreachable.
    for x in cx - 1..=cx + 1 {
        for y in cy - 1..=cy + 1 {
            if (x, y) != (cx, cy) {
                counter_at(w, x, y);
            }
        }
    }
}

#[test]
fn walled_target_reports_gave_up() {
    let mut w = world();
    let bc = new_follower(&mut w, 1500, 1601);
    let target = new_walker(&mut w, 1500, 1596);
    ring_around(&mut w, 1500, 1596);

    let mut saw_routing = false;
    for _ in 0..120 {
        set_next_move(&mut w, bc, 0);
        movement::move_to(&mut w, bc, target, 1);
        saw_routing |= matches!(
            last_approach(&w, bc),
            ApproachOutcome::Routing | ApproachOutcome::Blocked
        );
    }

    assert!(
        saw_routing,
        "a walled target must route or block before giving up"
    );
    assert_eq!(last_approach(&w, bc), ApproachOutcome::GaveUp);
}

#[test]
fn open_terrain_reaches_target_without_pathfinding() {
    let mut w = world();
    let bc = new_follower(&mut w, 1500, 1600);
    let target = new_walker(&mut w, 1495, 1600); // 5 tiles west, open
    assert!(
        drive_follow(&mut w, bc, target, 2, 40),
        "creature should reach an open-terrain target by stepping"
    );
}

#[test]
fn move_to_catches_target_walking_away_open_terrain() {
    let mut w = world();
    let bc = new_follower(&mut w, 1500, 1600);
    let target = new_walker(&mut w, 1498, 1600);

    let mut caught = false;
    for i in 0..60 {
        set_next_move(&mut w, bc, 0);
        movement::move_to(&mut w, bc, target, 1);

        // Target walks west every other tick for its first several steps, then stops, so a same-speed chaser
        // eventually closes the gap.
        let t = w.mobile(target).unwrap().location;
        if i % 2 == 0 && i < 16 && t.x > 1490 {
            muo_server::mobile::base::move_to_world(
                &mut w,
                target,
                Point3D::new(t.x - 1, t.y, t.z),
                MapId::TRAMMEL,
            );
        }
        if in_range(&w, bc, target, 1) {
            caught = true;
            break;
        }
    }
    assert!(
        caught,
        "chaser must catch a target that walks away then stops"
    );
}

#[test]
fn unreachable_target_gives_up_and_idles() {
    let mut w = world();
    let bc = new_follower(&mut w, 1500, 1601);
    let target = new_walker(&mut w, 1500, 1596);
    ring_around(&mut w, 1500, 1596);

    // Run well past the stuck window so give-up engages.
    for _ in 0..120 {
        set_next_move(&mut w, bc, 0);
        movement::move_to(&mut w, bc, target, 1);
    }

    // After giving up, the creature must idle (not oscillate) while the goal is still.
    let idle_start = w.mobile(bc).unwrap().location;
    let mut stayed_idle = true;
    for _ in 0..20 {
        set_next_move(&mut w, bc, 0);
        movement::move_to(&mut w, bc, target, 1);
        if w.mobile(bc).unwrap().location != idle_start {
            stayed_idle = false;
            break;
        }
    }

    assert!(
        !in_range(&w, bc, target, 1),
        "walled-off target must not be reached"
    );
    assert!(
        stayed_idle,
        "after giving up, the creature must idle, not shuffle"
    );
}

#[test]
fn wall_between_routes_around_reaches_target() {
    let mut w = world();
    let bc = new_follower(&mut w, 1500, 1600);
    let target = new_walker(&mut w, 1500, 1595); // 5 tiles north
    // Impassable wall on Y=1598 from X=1497..1503: blocks the straight line; ends open.
    for x in 1497..=1503 {
        counter_at(&mut w, x, 1598);
    }
    assert!(
        drive_follow(&mut w, bc, target, 2, 200),
        "creature must route around the wall to reach the target"
    );
}

#[test]
fn herded_creature_reaches_the_tile_and_clears_target_location() {
    let mut w = world();
    let bc = new_follower(&mut w, 1500, 1600);
    let goal = muo_primitives::Point2D::new(1500, 1595);
    crate::creature::data::cd_mut(&mut w, bc)
        .unwrap()
        .target_location = Some(goal);

    for _ in 0..40 {
        if cd(&w, bc).unwrap().target_location.is_none() {
            break;
        }
        set_next_move(&mut w, bc, 0);
        base::check_herding(&mut w, bc);
    }

    assert!(cd(&w, bc).unwrap().target_location.is_none());
    let loc = w.mobile(bc).unwrap().location;
    assert_eq!((loc.x, loc.y), (goal.x, goal.y));
}

// ---- PetOrderTests (the standing-order state machine) ----

fn spawn_pair(w: &mut World, master_at: (i32, i32), pet_at: (i32, i32)) -> (MobileId, MobileId) {
    let master = real_player(w);
    place(w, master, master_at.0, master_at.1, MapId::FELUCCA);
    let pet = pet_stub(w);
    place(w, pet, pet_at.0, pet_at.1, MapId::FELUCCA);
    set_control_master(w, pet, Some(master)); // sets Controlled, Home=Zero, ControlOrder=Come
    (master, pet)
}

fn order(w: &World, pet: MobileId) -> OrderType {
    cd(w, pet).unwrap().control_order
}

fn persistent(w: &World, pet: MobileId) -> OrderType {
    orders::persistent_order(w, pet)
}

fn home(w: &World, pet: MobileId) -> Point3D {
    cd(w, pet).unwrap().home
}

fn loc(w: &World, m: MobileId) -> Point3D {
    w.mobile(m).unwrap().location
}

fn set_order(w: &mut World, pet: MobileId, o: OrderType) {
    set_control_order_raw(w, pet, o);
}

fn set_target(w: &mut World, pet: MobileId, t: Option<MobileId>) {
    crate::creature::data::cd_mut(w, pet)
        .unwrap()
        .control_target = t;
}

fn target(w: &World, pet: MobileId) -> Option<MobileId> {
    cd(w, pet).unwrap().control_target
}

fn p(x: i32, y: i32) -> Point3D {
    Point3D::new(x, y, 0)
}

#[test]
fn set_persistent_order_stay_anchors_home_to_current_location() {
    let mut w = world();
    let (_, pet) = spawn_pair(&mut w, (1000, 1000), (1005, 1000));
    orders::set_persistent_order_pub(&mut w, pet, OrderType::Stay);
    assert_eq!(persistent(&w, pet), OrderType::Stay);
    assert_eq!(home(&w, pet), loc(&w, pet));
}

#[test]
fn set_persistent_order_follow_clears_anchor() {
    let mut w = world();
    let (_, pet) = spawn_pair(&mut w, (1000, 1000), (1005, 1000));
    set_home(&mut w, pet, p(900, 900)); // stale anchor
    orders::set_persistent_order_pub(&mut w, pet, OrderType::Follow);
    assert_eq!(persistent(&w, pet), OrderType::Follow);
    assert_eq!(home(&w, pet), Point3D::ZERO);
}

#[test]
fn stop_while_attacking_falls_back_to_persistent_follow() {
    let mut w = world();
    let (_, pet) = spawn_pair(&mut w, (1000, 1000), (1002, 1000));
    set_order(&mut w, pet, OrderType::Follow); // persistent = Follow
    set_order(&mut w, pet, OrderType::Attack); // transient
    assert_eq!(persistent(&w, pet), OrderType::Follow);

    set_order(&mut w, pet, OrderType::Stop);

    assert_eq!(order(&w, pet), OrderType::Follow); // resumed standing order
    assert_eq!(persistent(&w, pet), OrderType::Follow);
}

#[test]
fn stop_while_attacking_resumed_follow_targets_the_master() {
    let mut w = world();
    let (master, pet) = spawn_pair(&mut w, (1000, 1000), (1001, 1000));
    set_target(&mut w, pet, Some(master));
    set_order(&mut w, pet, OrderType::Follow);
    set_order(&mut w, pet, OrderType::Attack);
    assert_eq!(persistent(&w, pet), OrderType::Follow);

    set_order(&mut w, pet, OrderType::Stop);

    assert_eq!(order(&w, pet), OrderType::Follow);
    assert_eq!(target(&w, pet), Some(master));
}

#[test]
fn stop_while_attacking_resumed_follow_keeps_a_pet_friend_as_its_target() {
    let mut w = world();
    let (_, pet) = spawn_pair(&mut w, (1000, 1000), (1001, 1000));
    let friend = real_player(&mut w);
    place(&mut w, friend, 1002, 1000, MapId::FELUCCA);
    let victim = pet_stub(&mut w);
    place(&mut w, victim, 1003, 1000, MapId::FELUCCA);

    set_target(&mut w, pet, Some(friend)); // a pet friend said "all follow me"
    set_order(&mut w, pet, OrderType::Follow);
    set_target(&mut w, pet, Some(victim)); // then "all kill": the attack order takes the target
    set_order(&mut w, pet, OrderType::Attack);

    set_order(&mut w, pet, OrderType::Stop);

    assert_eq!(order(&w, pet), OrderType::Follow);
    assert_eq!(target(&w, pet), Some(friend)); // still the friend, not the owner
}

#[test]
fn stop_while_attacking_resumed_follow_survives_the_next_think() {
    let mut w = world();
    let (_, pet) = spawn_pair(&mut w, (1000, 1000), (1001, 1000));
    set_order(&mut w, pet, OrderType::Follow);
    set_order(&mut w, pet, OrderType::Attack);
    set_order(&mut w, pet, OrderType::Stop); // resumes Follow

    orders::obey(&mut w, pet);

    assert_eq!(order(&w, pet), OrderType::Follow); // not dropped to idle
    assert_eq!(persistent(&w, pet), OrderType::Follow);
}

#[test]
fn stop_while_following_cancels_to_idle_none() {
    let mut w = world();
    let (_, pet) = spawn_pair(&mut w, (1000, 1000), (1002, 1000));
    set_order(&mut w, pet, OrderType::Follow);

    set_order(&mut w, pet, OrderType::Stop);

    assert_eq!(order(&w, pet), OrderType::None);
    assert_eq!(persistent(&w, pet), OrderType::None);
    assert_eq!(loc(&w, pet), home(&w, pet)); // idle anchor = where stopped
}

// Publish 51: stop cancels a stay and the pet idles, anchored where it stands.
#[test]
fn stop_while_staying_cancels_to_idle_none() {
    let mut w = world();
    let (_, pet) = spawn_pair(&mut w, (1000, 1000), (1005, 1005));
    set_order(&mut w, pet, OrderType::Stay); // Home = post
    assert_eq!(home(&w, pet), p(1005, 1005));

    set_order(&mut w, pet, OrderType::Stop);

    assert_eq!(order(&w, pet), OrderType::None);
    assert_eq!(persistent(&w, pet), OrderType::None);
    assert_eq!(loc(&w, pet), home(&w, pet)); // idle anchor = where stopped
}

// "all come" rests into Stay on arrival; "all stop" must not leave the pet at that post.
#[test]
fn come_then_arrive_then_stop_idles_rather_than_staying() {
    let mut w = world();
    let (_, pet) = spawn_pair(&mut w, (1000, 1000), (1001, 1000));
    assert_eq!(order(&w, pet), OrderType::Come);

    orders::obey(&mut w, pet); // within 2 tiles -> Stay
    assert_eq!(order(&w, pet), OrderType::Stay);

    set_order(&mut w, pet, OrderType::Stop);

    assert_eq!(order(&w, pet), OrderType::None);
    assert_eq!(persistent(&w, pet), OrderType::None);
    assert_eq!(loc(&w, pet), home(&w, pet));
}

#[test]
fn stay_then_follow_then_stop_does_not_return_to_old_stay_anchor() {
    let mut w = world();
    let post_a = p(1005, 1005);
    let (_, pet) = spawn_pair(&mut w, (1000, 1000), (1005, 1005));
    set_order(&mut w, pet, OrderType::Stay); // Home = A
    set_order(&mut w, pet, OrderType::Follow); // Home cleared to Zero
    place(&mut w, pet, 1050, 1050, MapId::FELUCCA); // walked to B
    set_order(&mut w, pet, OrderType::Stop); // stop while following

    assert_ne!(home(&w, pet), post_a); // never re-acquires A
    assert_eq!(loc(&w, pet), home(&w, pet)); // idles at B
}

#[test]
fn attack_target_lost_resumes_persistent_follow() {
    let mut w = world();
    let (_, pet) = spawn_pair(&mut w, (1000, 1000), (1002, 1000));
    set_order(&mut w, pet, OrderType::Follow); // persistent = Follow
    set_order(&mut w, pet, OrderType::Attack);
    set_target(&mut w, pet, None); // target gone

    orders::do_order_attack(&mut w, pet); // invalid-target path

    assert_eq!(order(&w, pet), OrderType::Follow);
}

#[test]
fn stay_pet_attacks_then_target_lost_returns_to_original_post() {
    let mut w = world();
    let post = p(1005, 1005);
    let (_, pet) = spawn_pair(&mut w, (1000, 1000), (1005, 1005));
    set_order(&mut w, pet, OrderType::Stay); // persistent = Stay, Home = post
    set_order(&mut w, pet, OrderType::Attack);
    place(&mut w, pet, 1060, 1060, MapId::FELUCCA); // chased far to the "corpse"
    set_target(&mut w, pet, None);

    orders::do_order_attack(&mut w, pet);

    assert_eq!(order(&w, pet), OrderType::Stay);
    assert_eq!(home(&w, pet), post); // anchor still the original post, not the corpse
}

#[test]
fn staying_pet_does_not_attempt_to_move() {
    let mut w = world();
    let (_, pet) = spawn_pair(&mut w, (1000, 1000), (1005, 1005));
    set_order(&mut w, pet, OrderType::Stay);
    w.mobile_mut(pet).unwrap().direction = Direction::NORTH;

    for _ in 0..40 {
        orders::do_order_stay(&mut w, pet);
    }

    assert_eq!(w.mobile(pet).unwrap().direction, Direction::NORTH); // frozen -> no wander attempts
}

#[test]
fn release_order_clears_the_master_and_starts_the_delete_countdown() {
    let mut w = world();
    let (master, pet) = spawn_pair(&mut w, (1000, 1000), (1001, 1000));
    set_is_bonded(&mut w, pet, true);
    let followers = w.mobile(master).unwrap().followers;

    set_order(&mut w, pet, OrderType::Release);

    assert!(!cd(&w, pet).unwrap().controlled);
    assert_eq!(control_master(&w, pet), None);
    assert!(!cd(&w, pet).unwrap().is_bonded);
    assert_eq!(
        w.mobile(master).unwrap().followers,
        followers - cd(&w, pet).unwrap().control_slots
    );
    assert!(cd(&w, pet).unwrap().pending_delete_timer.running(&w));
    assert_eq!(home(&w, pet), loc(&w, pet));
}

#[test]
fn release_without_spawner_anchors_home_to_current_location() {
    let mut w = world();
    let (_, pet) = spawn_pair(&mut w, (1000, 1000), (1010, 1010));
    set_order(&mut w, pet, OrderType::Stay); // sets Home to loc
    set_home(&mut w, pet, p(800, 800)); // simulate a stale anchor
    w.mobile_mut(pet).unwrap().spawner = None;

    set_order(&mut w, pet, OrderType::Release);

    assert_eq!(home(&w, pet), p(1010, 1010)); // released where it stands, not the stale point
}

#[test]
fn login_near_master_derives_follow() {
    let mut w = world();
    let (master, pet) = spawn_pair(&mut w, (1000, 1000), (1001, 1000));
    assert_eq!(persistent(&w, pet), OrderType::None);
    speech::derive_follower_orders(&mut w, master);
    assert_eq!(persistent(&w, pet), OrderType::Follow);
}

#[test]
fn login_far_from_master_derives_stay() {
    let mut w = world();
    let (master, pet) = spawn_pair(&mut w, (1000, 1000), (1040, 1000));
    speech::derive_follower_orders(&mut w, master);
    assert_eq!(persistent(&w, pet), OrderType::Stay);
}

/// `pet.ChangeAIType(pet.AI)`: what AfterDeserialization does, a fresh AI with PersistentOrder = None.
fn change_ai(w: &mut World, pet: MobileId) {
    let ai_type = cd(w, pet).unwrap().current_ai;
    change_ai_type(w, pet, ai_type);
}

#[test]
fn login_restored_stay_keeps_its_post_anchor() {
    let mut w = world();
    let post = p(1005, 1005);
    let (master, pet) = spawn_pair(&mut w, (1000, 1000), (1005, 1005));
    set_order(&mut w, pet, OrderType::Stay); // Home = post
    change_ai(&mut w, pet);
    assert_eq!(persistent(&w, pet), OrderType::None);

    speech::derive_follower_orders(&mut w, master); // master within 12 tiles

    assert_eq!(order(&w, pet), OrderType::Stay);
    assert_eq!(persistent(&w, pet), OrderType::Stay);
    assert_eq!(home(&w, pet), post); // not zeroed by a proximity-derived Follow
}

#[test]
fn login_restored_none_near_master_issues_follow() {
    let mut w = world();
    let (master, pet) = spawn_pair(&mut w, (1000, 1000), (1001, 1000));
    set_order(&mut w, pet, OrderType::Follow);
    set_order(&mut w, pet, OrderType::Stop); // -> None, no standing order
    change_ai(&mut w, pet);
    w.mobile_mut(master).unwrap().hidden = true;

    speech::derive_follower_orders(&mut w, master);

    assert_eq!(order(&w, pet), OrderType::Follow);
    assert_eq!(target(&w, pet), Some(master));
    assert_eq!(persistent(&w, pet), OrderType::Follow);
    assert!(w.mobile(master).unwrap().hidden); // system-issued: nobody revealed
}

#[test]
fn login_restored_attack_far_from_master_issues_stay() {
    let mut w = world();
    let (master, pet) = spawn_pair(&mut w, (1000, 1000), (1040, 1000));
    set_order(&mut w, pet, OrderType::Attack); // rests with no valid target
    change_ai(&mut w, pet);

    speech::derive_follower_orders(&mut w, master);

    assert_eq!(order(&w, pet), OrderType::Stay);
    assert_eq!(loc(&w, pet), home(&w, pet));
}

#[test]
fn login_restored_attack_near_master_follows_the_master_not_the_victim() {
    let mut w = world();
    let (master, pet) = spawn_pair(&mut w, (1000, 1000), (1001, 1000));
    let victim = real_player(&mut w);
    place(&mut w, victim, 1003, 1000, MapId::FELUCCA);
    crate::creature::control::issue_order(
        &mut w,
        pet,
        OrderType::Attack,
        Some(master),
        Some(victim),
    ); // saved mid-fight
    change_ai(&mut w, pet); // post-load fresh AI

    speech::derive_follower_orders(&mut w, master);

    assert_eq!(order(&w, pet), OrderType::Follow);
    assert_eq!(target(&w, pet), Some(master));
    assert_eq!(persistent(&w, pet), OrderType::Follow);
}

fn spawn_victim(w: &mut World) -> MobileId {
    let victim = pet_stub(w);
    place(w, victim, 1003, 1000, MapId::FELUCCA);
    victim
}

#[test]
fn drop_mid_attack_keeps_the_attack_and_its_target() {
    let mut w = world();
    let (master, pet) = spawn_pair(&mut w, (1000, 1000), (1001, 1000));
    let victim = spawn_victim(&mut w);
    set_order(&mut w, pet, OrderType::Follow); // standing order
    crate::creature::control::issue_order(
        &mut w,
        pet,
        OrderType::Attack,
        Some(master),
        Some(victim),
    );

    crate::creature::control::issue_order(&mut w, pet, OrderType::Drop, Some(master), None);

    assert_eq!(order(&w, pet), OrderType::Attack);
    assert_eq!(target(&w, pet), Some(victim));
    assert_eq!(w.mobile(pet).unwrap().combatant, Some(victim));
}

#[test]
fn rename_mid_attack_keeps_the_attack() {
    let mut w = world();
    let (master, pet) = spawn_pair(&mut w, (1000, 1000), (1001, 1000));
    let victim = spawn_victim(&mut w);
    set_order(&mut w, pet, OrderType::Follow);
    crate::creature::control::issue_order(
        &mut w,
        pet,
        OrderType::Attack,
        Some(master),
        Some(victim),
    );

    crate::creature::control::issue_order(&mut w, pet, OrderType::Rename, Some(master), None);

    assert_eq!(order(&w, pet), OrderType::Attack);
    assert_eq!(target(&w, pet), Some(victim));
}

#[test]
fn friend_refusal_mid_attack_keeps_the_attack() {
    let mut w = world();
    let (master, pet) = spawn_pair(&mut w, (1000, 1000), (1001, 1000));
    let friend = real_player(&mut w);
    place(&mut w, friend, 1002, 1000, MapId::FELUCCA);
    add_pet_friend(&mut w, pet, friend); // already a friend -> refusal
    let victim = spawn_victim(&mut w);
    set_order(&mut w, pet, OrderType::Follow);
    crate::creature::control::issue_order(
        &mut w,
        pet,
        OrderType::Attack,
        Some(master),
        Some(victim),
    );

    crate::creature::control::issue_order(
        &mut w,
        pet,
        OrderType::Friend,
        Some(master),
        Some(friend),
    );

    assert_eq!(order(&w, pet), OrderType::Attack);
    assert_eq!(target(&w, pet), Some(victim));
}

#[test]
fn drop_mid_attack_with_the_target_gone_falls_back_to_the_standing_order() {
    let mut w = world();
    let (master, pet) = spawn_pair(&mut w, (1000, 1000), (1001, 1000));
    let victim = spawn_victim(&mut w);
    set_order(&mut w, pet, OrderType::Follow);
    crate::creature::control::issue_order(
        &mut w,
        pet,
        OrderType::Attack,
        Some(master),
        Some(victim),
    );
    muo_server::mobile::base::delete(&mut w, victim);

    crate::creature::control::issue_order(&mut w, pet, OrderType::Drop, Some(master), None);

    assert_eq!(order(&w, pet), OrderType::Follow);
}

#[test]
fn drop_mid_attack_does_not_repeat_the_harm() {
    let mut w = world();
    let (master, pet) = spawn_pair(&mut w, (1000, 1000), (1001, 1000));
    let victim = spawn_victim(&mut w);
    set_order(&mut w, pet, OrderType::Follow);
    crate::creature::control::issue_order(
        &mut w,
        pet,
        OrderType::Attack,
        Some(master),
        Some(victim),
    );
    let aggressors = w.mobile(victim).unwrap().aggressors.len();

    crate::creature::control::issue_order(&mut w, pet, OrderType::Drop, Some(master), None);

    assert_eq!(w.mobile(victim).unwrap().aggressors.len(), aggressors);
}

#[test]
fn friend_refused_rests_at_persistent_order_and_obey_does_not_repeat() {
    let mut w = world();
    let (master, pet) = spawn_pair(&mut w, (1000, 1000), (1001, 1000));
    set_order(&mut w, pet, OrderType::Follow); // persistent = Follow
    let friend = real_player(&mut w);
    place(&mut w, friend, 1002, 1000, MapId::FELUCCA);
    add_pet_friend(&mut w, pet, friend); // already a friend -> refusal 1049691

    crate::creature::control::issue_order(
        &mut w,
        pet,
        OrderType::Friend,
        Some(master),
        Some(friend),
    );

    assert_eq!(order(&w, pet), OrderType::Follow); // never rests at Friend
    assert!(orders::is_restable_order(order(&w, pet)));

    orders::obey(&mut w, pet);
    assert_eq!(order(&w, pet), OrderType::Follow);
}

#[test]
fn unfriend_of_non_friend_rests_at_persistent_order() {
    let mut w = world();
    let (master, pet) = spawn_pair(&mut w, (1000, 1000), (1001, 1000));
    set_order(&mut w, pet, OrderType::Stay); // persistent = Stay
    let stranger = real_player(&mut w);
    place(&mut w, stranger, 1002, 1000, MapId::FELUCCA);

    crate::creature::control::issue_order(
        &mut w,
        pet,
        OrderType::Unfriend,
        Some(master),
        Some(stranger),
    );

    assert_eq!(order(&w, pet), OrderType::Stay);
    assert_eq!(persistent(&w, pet), OrderType::Stay);
}

#[test]
fn friend_accepted_leaves_the_standing_order_alone() {
    let mut w = world();
    let post = p(1001, 1000);
    let (master, pet) = spawn_pair(&mut w, (1000, 1000), (1001, 1000));
    set_order(&mut w, pet, OrderType::Stay); // the owner's standing order, anchored at the post
    let friend = real_player(&mut w);
    w.mobile_mut(friend).unwrap().player = true; // SpawnPlayer: new PlayerMobile { Player = true }
    place(&mut w, friend, 1002, 1000, MapId::FELUCCA);

    crate::creature::control::issue_order(
        &mut w,
        pet,
        OrderType::Friend,
        Some(master),
        Some(friend),
    );

    assert!(is_pet_friend(&w, pet, friend));
    assert_eq!(order(&w, pet), OrderType::Stay); // still staying
    assert_eq!(persistent(&w, pet), OrderType::Stay); // owner's order not rewritten
    assert_eq!(home(&w, pet), post); // and not re-anchored
}

// ---- GuardOrderTests ----

fn spawn_guarding_pet(w: &mut World) -> (MobileId, MobileId) {
    let master = real_player(w);
    place(w, master, 1500, 1600, MapId::TRAMMEL);
    let pet = pet_stub(w);
    place(w, pet, 1499, 1600, MapId::TRAMMEL);
    set_control_master(w, pet, Some(master));
    timer::pause(w, pet); // drive manually
    set_order(w, pet, OrderType::Guard);
    timer::pause(w, pet); // the order change restarts the timer
    (master, pet)
}

fn spawn_aggressor(w: &mut World, pet: MobileId, x: i32, attacking: Option<MobileId>) -> MobileId {
    let aggr = new_walker(w, x, 1600);
    assert!(
        mx_in_los(w, pet, aggr),
        "no LOS from pet to aggressor at {x}"
    );
    if let Some(a) = attacking {
        muo_server::mobile::base::set_combatant(w, aggr, Some(a));
        assert_eq!(w.mobile(aggr).unwrap().combatant, Some(a));
    }
    aggr
}

fn mx_in_los(w: &mut World, a: MobileId, b: MobileId) -> bool {
    crate::mx::in_los(w, a, b)
}

fn combatant(w: &World, m: MobileId) -> Option<MobileId> {
    w.mobile(m).unwrap().combatant
}

#[test]
fn guard_engage_keeps_guard_order() {
    let mut w = world();
    let (master, pet) = spawn_guarding_pet(&mut w);
    let aggr = spawn_aggressor(&mut w, pet, 1498, Some(master));

    orders::obey(&mut w, pet);

    assert_eq!(combatant(&w, pet), Some(aggr));
    assert_eq!(order(&w, pet), OrderType::Guard);
    assert_eq!(persistent(&w, pet), OrderType::Guard);
}

#[test]
fn guard_retargets_to_aggressor_closest_to_master() {
    let mut w = world();
    let (master, pet) = spawn_guarding_pet(&mut w);
    let far = spawn_aggressor(&mut w, pet, 1495, Some(master));
    let near = spawn_aggressor(&mut w, pet, 1498, Some(master));
    muo_server::mobile::base::set_combatant(&mut w, pet, Some(far)); // already fighting the far aggressor

    orders::obey(&mut w, pet);

    assert_eq!(combatant(&w, pet), Some(near)); // defends the master, not the current fight
    assert_eq!(order(&w, pet), OrderType::Guard);
}

#[test]
fn explicit_attack_resumes_guard_without_chaining_into_attack() {
    let mut w = world();
    let (master, pet) = spawn_guarding_pet(&mut w);
    // Explicit kill order on a target that then becomes invalid.
    let victim = spawn_aggressor(&mut w, pet, 1498, None);
    set_target(&mut w, pet, Some(victim));
    set_order(&mut w, pet, OrderType::Attack);
    w.mobile_mut(victim).unwrap().hidden = true;
    // A second aggressor is still after the master; FightMode.Closest would chain it.
    let aggr2 = spawn_aggressor(&mut w, pet, 1497, Some(master));

    orders::obey(&mut w, pet); // attack completes -> resume the persistent Guard
    assert_eq!(order(&w, pet), OrderType::Guard);

    orders::obey(&mut w, pet); // the guard scan engages the remaining aggressor in-order
    assert_eq!(combatant(&w, pet), Some(aggr2));
    assert_eq!(order(&w, pet), OrderType::Guard);
}

#[test]
fn peaceful_guard_stands_down() {
    let mut w = world();
    let (_, pet) = spawn_guarding_pet(&mut w);
    assert!(w.mobile(pet).unwrap().warmode); // the guard order opens in war stance
    orders::obey(&mut w, pet); // nothing to guard against
    assert!(!w.mobile(pet).unwrap().warmode);
    assert_eq!(combatant(&w, pet), None);
    assert_eq!(cd(&w, pet).unwrap().focus_mob, None);
}

// ---- PetRetaliationTests: publish 51, a pet told to follow, come, stay or stop will not attack anything ----

static STAND_DOWN_C: crate::creature::vtable::CreatureType =
    crate::creature::vtable::CreatureType {
        stands_down_on_command: |_, _| true,
        ..BASE_CREATURE_C
    };
static STAND_DOWN_PET: MobileType = MobileType {
    full_name: "Server.Tests.StandDownPet",
    parent: Some(&BASE_CREATURE),
    family: Some(&STAND_DOWN_C),
    ..BASE_CREATURE
};
static FIGHT_BACK_C: crate::creature::vtable::CreatureType =
    crate::creature::vtable::CreatureType {
        stands_down_on_command: |_, _| false,
        ..BASE_CREATURE_C
    };
static FIGHT_BACK_PET: MobileType = MobileType {
    full_name: "Server.Tests.FightBackPet",
    parent: Some(&BASE_CREATURE),
    family: Some(&FIGHT_BACK_C),
    ..BASE_CREATURE
};
static UNCOMMANDABLE_C: crate::creature::vtable::CreatureType =
    crate::creature::vtable::CreatureType {
        stands_down_on_command: |_, _| true,
        commandable: |_, _| false,
        ..BASE_CREATURE_C
    };
static UNCOMMANDABLE_PET: MobileType = MobileType {
    full_name: "Server.Tests.UncommandablePet",
    parent: Some(&BASE_CREATURE),
    family: Some(&UNCOMMANDABLE_C),
    ..BASE_CREATURE
};

fn typed_stub(w: &mut World, ty: &'static MobileType) -> MobileId {
    let id = new_creature(w, ty, AIType::AiAnimal, FightMode::Closest, 10, 1);
    w.mobile_mut(id).unwrap().body = Body(0xC8);
    id
}

/// `Spawn<T>()`: a master at (1000,1000) and a controlled pet of class `ty` at (1001,1000), its timer stopped.
fn spawn_typed(w: &mut World, ty: &'static MobileType) -> (MobileId, MobileId) {
    let master = real_player(w);
    place(w, master, 1000, 1000, MapId::FELUCCA);
    let pet = typed_stub(w, ty);
    place(w, pet, 1001, 1000, MapId::FELUCCA);
    set_control_master(w, pet, Some(master));
    timer::pause(w, pet);
    (master, pet)
}

fn give_order(w: &mut World, pet: MobileId, master: MobileId, o: OrderType) {
    if o == OrderType::None {
        // Publish 51's "stop": stops, may wander, will not attack
        set_target(w, pet, Some(master));
        set_order(w, pet, OrderType::Follow);
        set_order(w, pet, OrderType::Stop);
        return;
    }
    set_target(w, pet, Some(master));
    set_order(w, pet, o);
}

fn attack(w: &mut World, pet: MobileId) -> MobileId {
    let attacker = pet_stub(w);
    place(w, attacker, 1002, 1000, MapId::FELUCCA);
    timer::pause(w, pet);
    muo_server::mobile::base::set_combatant(w, attacker, Some(pet)); // a mob starts attacking the pet
    attacker
}

#[test]
fn stand_down_order_ignores_the_attacker() {
    for o in [
        OrderType::Follow,
        OrderType::Come,
        OrderType::Stay,
        OrderType::None,
    ] {
        let mut w = world();
        let (master, pet) = spawn_typed(&mut w, &STAND_DOWN_PET);
        give_order(&mut w, pet, master, o);
        let resting = order(&w, pet);
        attack(&mut w, pet);
        assert_eq!(order(&w, pet), resting, "{o:?}"); // never converts to Attack
        assert_eq!(combatant(&w, pet), None, "{o:?}");
        assert!(!w.mobile(pet).unwrap().warmode, "{o:?}");
    }
}

#[test]
fn without_stand_down_the_same_orders_retaliate() {
    for o in [
        OrderType::Follow,
        OrderType::Come,
        OrderType::Stay,
        OrderType::None,
    ] {
        let mut w = world();
        let (master, pet) = spawn_typed(&mut w, &FIGHT_BACK_PET);
        give_order(&mut w, pet, master, o);
        let attacker = attack(&mut w, pet);
        assert_eq!(order(&w, pet), OrderType::Attack, "{o:?}");
        assert_eq!(combatant(&w, pet), Some(attacker), "{o:?}");
    }
}

// No damage callback may put a stand-down pet back into combat behind the policy's back.
#[test]
fn stand_down_pet_stays_down_through_repeated_damage() {
    use muo_data::expansion::Expansion;
    for (era, spell_damage) in [
        (Expansion::AOS, false),
        (Expansion::AOS, true),
        (Expansion::ML, false),
        (Expansion::ML, true),
    ] {
        let mut w = world();
        w.core.expansion = Some(era);
        let (master, pet) = spawn_typed(&mut w, &STAND_DOWN_PET);
        give_order(&mut w, pet, master, OrderType::Follow);
        let attacker = attack(&mut w, pet);
        assert_eq!(order(&w, pet), OrderType::Follow); // the initial aggression stood down
        for _ in 0..500 {
            if order(&w, pet) != OrderType::Follow {
                break;
            }
            if spell_damage {
                (ctype(&w, pet).unwrap().on_damaged_by_spell)(&mut w, pet, attacker, 1);
            } else {
                crate::creature::overrides::on_damage(&mut w, pet, 1, Some(attacker), false);
            }
        }
        assert_eq!(order(&w, pet), OrderType::Follow, "{era:?} {spell_damage}");
        assert_eq!(combatant(&w, pet), None);
    }
}

// "Guard: the pet should guard as it does currently."
#[test]
fn guarding_pet_still_fights_under_stand_down() {
    let mut w = world();
    let (master, pet) = spawn_typed(&mut w, &STAND_DOWN_PET);
    give_order(&mut w, pet, master, OrderType::Guard);
    let attacker = attack(&mut w, pet);
    assert_eq!(order(&w, pet), OrderType::Guard);
    assert_eq!(combatant(&w, pet), Some(attacker));
    assert!(w.mobile(pet).unwrap().warmode);
}

// The publish speaks of commanded pets: a creature nobody can give an order to must still fight back.
#[test]
fn wild_creature_retaliates_under_stand_down() {
    let mut w = world();
    let creature = typed_stub(&mut w, &STAND_DOWN_PET);
    place(&mut w, creature, 1001, 1000, MapId::FELUCCA);
    timer::pause(&mut w, creature);
    assert_eq!(order(&w, creature), OrderType::None);
    let attacker = attack(&mut w, creature);
    assert_eq!(combatant(&w, creature), Some(attacker));
    assert!(w.mobile(creature).unwrap().warmode);
}

// energy vortex, blade spirits: Summoned with a SummonMaster, never Controlled
#[test]
fn uncontrolled_summon_retaliates_under_stand_down() {
    let mut w = world();
    let caster = real_player(&mut w);
    place(&mut w, caster, 1000, 1000, MapId::FELUCCA);
    let summon = typed_stub(&mut w, &STAND_DOWN_PET);
    set_summoned(&mut w, summon, true);
    set_master(&mut w, summon, Some(caster));
    place(&mut w, summon, 1001, 1000, MapId::FELUCCA);
    timer::pause(&mut w, summon);
    assert!(!cd(&w, summon).unwrap().controlled);
    assert_eq!(order(&w, summon), OrderType::None);
    let attacker = attack(&mut w, summon);
    assert_eq!(combatant(&w, summon), Some(attacker));
    assert!(w.mobile(summon).unwrap().warmode);
}

// familiar, escortee: Controlled with a master, but not Commandable
#[test]
fn uncommandable_creature_retaliates_under_stand_down() {
    let mut w = world();
    let (master, familiar) = spawn_typed(&mut w, &UNCOMMANDABLE_PET);
    set_target(&mut w, familiar, Some(master));
    set_order(&mut w, familiar, OrderType::Follow); // system-issued, not a command
    let attacker = attack(&mut w, familiar);
    assert_eq!(combatant(&w, familiar), Some(attacker));
    assert!(w.mobile(familiar).unwrap().warmode);
}

// ---- ForcedAITests ----

thread_local! { static FORCED_CONSTRUCTIONS: std::cell::Cell<i32> = const { std::cell::Cell::new(0) }; }

static FORCED_C: crate::creature::vtable::CreatureType = crate::creature::vtable::CreatureType {
    // not sector-gated, and each read of ForcedAI counts one construction
    forced_ai: |_, _| {
        FORCED_CONSTRUCTIONS.set(FORCED_CONSTRUCTIONS.get() + 1);
        Some(&classes::MELEE_AI)
    },
    player_range_sensitive: |_, _| false,
    ..BASE_CREATURE_C
};
static FORCED_STUB: MobileType = MobileType {
    full_name: "Server.Tests.ForcedStub",
    parent: Some(&BASE_CREATURE),
    family: Some(&FORCED_C),
    ..BASE_CREATURE
};

#[test]
fn change_ai_type_constructs_forced_ai_once() {
    let mut w = world();
    FORCED_CONSTRUCTIONS.set(0);
    let bc = new_creature(
        &mut w,
        &FORCED_STUB,
        AIType::AiMelee,
        FightMode::Closest,
        10,
        1,
    );
    assert_eq!(FORCED_CONSTRUCTIONS.get(), 1);
    assert!(std::ptr::eq(class_of(&w, bc).unwrap(), &classes::MELEE_AI));
}

// ---- PetOrderTests (the rest: renames, release, speech and context menu orders) ----

/// `SpawnPlayer`: `new PlayerMobile { Player = true }` on Felucca.
fn spawn_player(w: &mut World, x: i32, y: i32) -> MobileId {
    let pm = real_player(w);
    w.mobile_mut(pm).unwrap().player = true;
    place(w, pm, x, y, MapId::FELUCCA);
    pm
}

fn set_pet_name(w: &mut World, pet: MobileId, name: &str) {
    crate::creature::overrides::set_name(w, pet, Some(name.into()));
}

fn hidden(w: &World, m: MobileId) -> bool {
    w.mobile(m).unwrap().hidden
}

fn set_hidden(w: &mut World, m: MobileId, v: bool) {
    muo_server::mobile::part1::props_b::set_hidden(w, m, v);
}

fn say(w: &mut World, pet: MobileId, from: MobileId, text: &str, keywords: &[i32]) {
    let mut e = muo_server::events::SpeechEventArgs::new(
        from,
        text.into(),
        muo_primitives::MessageType::REGULAR,
        0x3B2,
        keywords.to_vec(),
    );
    on_speech(w, pet, &mut e);
}

#[test]
fn rename_while_following_keeps_following_the_master() {
    let mut w = world();
    let (master, pet) = spawn_pair(&mut w, (1000, 1000), (1001, 1000));
    crate::creature::control::issue_order(
        &mut w,
        pet,
        OrderType::Follow,
        Some(master),
        Some(master),
    );

    crate::creature::control::issue_order(&mut w, pet, OrderType::Rename, Some(master), None); // the menu passes no target

    assert_eq!(order(&w, pet), OrderType::Follow);
    assert_eq!(target(&w, pet), Some(master)); // restored, not null

    orders::obey(&mut w, pet);
    assert_eq!(order(&w, pet), OrderType::Follow); // no "no one to follow" -> None
}

#[test]
fn same_order_twice_re_runs_issue() {
    let mut w = world();
    let (_, pet) = spawn_pair(&mut w, (1000, 1000), (1005, 1005));
    set_order(&mut w, pet, OrderType::Stay); // Home = A
    place(&mut w, pet, 1050, 1050, MapId::FELUCCA);

    set_order(&mut w, pet, OrderType::Stay); // reissued: re-anchor

    assert_eq!(home(&w, pet), loc(&w, pet));
}

#[test]
fn loyalty_release_and_manual_release_produce_the_same_end_state() {
    let mut w = world();
    let (master_a, pet_a) = spawn_pair(&mut w, (1000, 1000), (1001, 1000));
    let (master_b, pet_b) = spawn_pair(&mut w, (1100, 1100), (1101, 1100));
    set_pet_name(&mut w, pet_a, "Rex");
    set_pet_name(&mut w, pet_b, "Rex");
    set_is_bonded(&mut w, pet_a, true);
    set_is_bonded(&mut w, pet_b, true);

    crate::creature::control::issue_order(&mut w, pet_a, OrderType::Release, Some(master_a), None); // player
    set_order(&mut w, pet_b, OrderType::Release); // what the loyalty drain does

    for pet in [pet_a, pet_b] {
        assert!(!cd(&w, pet).unwrap().controlled);
        assert_eq!(control_master(&w, pet), None);
        assert!(!cd(&w, pet).unwrap().is_bonded);
        assert_eq!(w.mobile(pet).unwrap().name, None);
        assert_eq!(order(&w, pet), OrderType::None);
        assert!(cd(&w, pet).unwrap().pending_delete_timer.running(&w));
        assert_eq!(home(&w, pet), loc(&w, pet));
    }
    assert_eq!(
        w.mobile(master_a).unwrap().followers,
        w.mobile(master_b).unwrap().followers
    );
}

#[test]
fn summoned_pet_released_is_killed_not_released() {
    let mut w = world();
    let (master, pet) = spawn_pair(&mut w, (1000, 1000), (1001, 1000));
    set_summoned(&mut w, pet, true);
    set_master(&mut w, pet, Some(master));

    set_order(&mut w, pet, OrderType::Release);

    assert!(w.mobile(pet).is_none_or(|m| m.deleted) || !crate::mx::alive(&w, pet));
}

#[test]
fn obey_on_a_legacy_transient_order_falls_back_to_persistent() {
    let mut w = world();
    let (_, pet) = spawn_pair(&mut w, (1000, 1000), (1001, 1000));
    set_order(&mut w, pet, OrderType::Follow); // persistent = Follow

    // a pre-refactor save resting at Rename
    crate::creature::data::cd_mut(&mut w, pet)
        .unwrap()
        .control_order = OrderType::Rename;
    assert_eq!(order(&w, pet), OrderType::Rename);

    orders::obey(&mut w, pet);

    assert_eq!(order(&w, pet), OrderType::Follow);
}

#[test]
fn speech_command_from_a_friend_reveals_the_friend_not_the_master() {
    let mut w = world();
    let (master, pet) = spawn_pair(&mut w, (1000, 1000), (1001, 1000));
    let friend = spawn_player(&mut w, 1002, 1000);
    add_pet_friend(&mut w, pet, friend);
    set_hidden(&mut w, master, true);
    set_hidden(&mut w, friend, true);

    say(&mut w, pet, friend, "all stay", &[0x170]); // "all stay" keyword 0x170

    assert_eq!(order(&w, pet), OrderType::Stay);
    assert!(!hidden(&w, friend));
    assert!(hidden(&w, master));
}

fn click(w: &mut World, number: i32, order: OrderType, from: MobileId, pet: MobileId) {
    context_menu::entry(number, order, true).click(w, from, muo_server::ids::EntityId::Mobile(pet));
}

#[test]
fn context_menu_command_from_a_friend_reveals_the_friend_not_the_master() {
    let mut w = world();
    let (master, pet) = spawn_pair(&mut w, (1000, 1000), (1001, 1000));
    let friend = spawn_player(&mut w, 1002, 1000);
    add_pet_friend(&mut w, pet, friend);
    set_hidden(&mut w, master, true);
    set_hidden(&mut w, friend, true);

    click(&mut w, 3006114, OrderType::Stay, friend, pet); // Command: Stay

    assert_eq!(order(&w, pet), OrderType::Stay);
    assert!(!hidden(&w, friend));
    assert!(hidden(&w, master));
}

#[test]
fn context_menu_command_from_a_friend_refuses_non_friend_orders() {
    let mut w = world();
    let (_, pet) = spawn_pair(&mut w, (1000, 1000), (1001, 1000));
    let friend = spawn_player(&mut w, 1002, 1000);
    add_pet_friend(&mut w, pet, friend);
    set_order(&mut w, pet, OrderType::Follow);

    click(&mut w, 3006107, OrderType::Guard, friend, pet); // Command: Guard

    assert_eq!(order(&w, pet), OrderType::Follow);
}

#[test]
fn context_menu_rename_leaves_the_pet_on_a_restable_order() {
    let mut w = world();
    let (master, pet) = spawn_pair(&mut w, (1000, 1000), (1001, 1000));
    set_order(&mut w, pet, OrderType::Follow);

    click(&mut w, 3006098, OrderType::Rename, master, pet); // Rename

    assert_eq!(order(&w, pet), OrderType::Follow);
}

#[test]
fn control_order_round_trips_and_load_does_not_run_issue() {
    let mut w = world();
    let post = p(1001, 1000);
    let (_, pet) = spawn_pair(&mut w, (1000, 1000), (1001, 1000));
    set_order(&mut w, pet, OrderType::Stay); // Home = post
    place(&mut w, pet, 1020, 1000, MapId::FELUCCA); // displaced: Home != Location
    assert_eq!(home(&w, pet), post);

    let mut out = muo_io::SaveWriter::new(true);
    (w.mobile(pet).unwrap().ty.serialize)(&w, pet, &mut out);
    let buffer = out.into_vec();

    let copy = pet_stub(&mut w); // the Issue gate is ai != null
    let mut r = muo_io::SaveReader::new(&buffer);
    let ty = w.mobile(copy).unwrap().ty;
    (ty.deserialize)(&mut w, copy, &mut r).unwrap();

    assert_eq!(order(&w, copy), OrderType::Stay);
    assert_eq!(loc(&w, copy), p(1020, 1000));
    assert_eq!(home(&w, copy), post); // not re-anchored
}

#[test]
fn speech_command_without_the_pets_name_is_ignored() {
    let mut w = world();
    let (master, pet) = spawn_pair(&mut w, (1000, 1000), (1001, 1000));
    set_pet_name(&mut w, pet, "Rex");
    set_order(&mut w, pet, OrderType::Stay);

    // bare "come" (keyword 0x155) with no name: not for this pet
    say(&mut w, pet, master, "come", &[0x155]);
    assert_eq!(order(&w, pet), OrderType::Stay);

    say(&mut w, pet, master, "Rex come", &[0x155]);
    assert_eq!(order(&w, pet), OrderType::Come);
}

#[test]
fn all_command_issues_once() {
    let mut w = world();
    let (master, pet) = spawn_pair(&mut w, (1000, 1000), (1001, 1000));
    set_pet_name(&mut w, pet, "Rex");
    set_order(&mut w, pet, OrderType::Follow);
    let post = loc(&w, pet);

    // The client emits both 0x170 ("all stay") and 0x16F ("*stay") for "all stay".
    say(&mut w, pet, master, "all stay", &[0x170, 0x16F]);

    assert_eq!(order(&w, pet), OrderType::Stay);
    assert_eq!(home(&w, pet), post);
    // The named keyword alone must be ignored: the speech starts with "all", not the name.
    place(&mut w, pet, 1010, 1010, MapId::FELUCCA);
    say(&mut w, pet, master, "all stay", &[0x16F]);
    assert_eq!(home(&w, pet), post);
}

#[test]
fn speech_command_from_a_friend_cannot_come_guard_or_drop() {
    let mut w = world();
    let (_, pet) = spawn_pair(&mut w, (1000, 1000), (1001, 1000));
    set_pet_name(&mut w, pet, "Rex");
    let friend = spawn_player(&mut w, 1002, 1000);
    add_pet_friend(&mut w, pet, friend);
    set_order(&mut w, pet, OrderType::Stay);
    let post = home(&w, pet);

    say(&mut w, pet, friend, "Rex come", &[0x155]);
    assert_eq!(order(&w, pet), OrderType::Stay);

    say(&mut w, pet, friend, "Rex guard", &[0x15C]);
    assert_eq!(order(&w, pet), OrderType::Stay);
    assert_eq!(persistent(&w, pet), OrderType::Stay);

    set_is_bonded(&mut w, pet, true); // CanDrop
    say(&mut w, pet, friend, "Rex drop", &[0x156]);
    assert_eq!(order(&w, pet), OrderType::Stay);
    assert_eq!(home(&w, pet), post); // never re-issued

    // The friend can still Stay/Follow/Stop.
    say(&mut w, pet, friend, "Rex follow me", &[0x163]);
    assert_eq!(order(&w, pet), OrderType::Follow);
    assert_eq!(target(&w, pet), Some(friend));
}

#[test]
fn gm_all_obey_does_not_take_controlled_pets() {
    let mut w = world();
    let (master, pet) = spawn_pair(&mut w, (1000, 1000), (1001, 1000));
    set_pet_name(&mut w, pet, "Rex");
    let gm = spawn_player(&mut w, 1002, 1000);
    muo_server::mobile::part1::props_a::set_access_level(
        &mut w,
        gm,
        muo_server::mobile::AccessLevel::GameMaster,
    );

    // The mass form is for wild creatures; a controlled pet must be named.
    say(&mut w, pet, gm, "all obey", &[]);
    assert_eq!(control_master(&w, pet), Some(master));

    say(&mut w, pet, gm, "Rex obey", &[]);
    assert_eq!(control_master(&w, pet), Some(gm));
}

static IDLE_C: crate::creature::vtable::CreatureType = crate::creature::vtable::CreatureType {
    check_idle: |_, _| true, // ForceIdle
    ..BASE_CREATURE_C
};
static IDLE_STUB: MobileType = MobileType {
    full_name: "Server.Tests.PetTestStub",
    parent: Some(&BASE_CREATURE),
    family: Some(&IDLE_C),
    ..BASE_CREATURE
};

#[test]
fn idle_pet_does_not_attempt_to_move_while_resting() {
    let mut w = world();
    let (_, pet) = spawn_pair(&mut w, (1000, 1000), (1002, 1000));
    // the ForceIdle stub: same creature, CheckIdle() reports resting
    w.mobile_mut(pet).unwrap().ty = &IDLE_STUB;
    set_order(&mut w, pet, OrderType::Follow);
    set_order(&mut w, pet, OrderType::Stop); // -> idle None
    assert_eq!(order(&w, pet), OrderType::None);

    w.mobile_mut(pet).unwrap().direction = Direction::NORTH;
    let start = loc(&w, pet);
    for _ in 0..40 {
        // Reopen the move clock every pass, so CheckIdle is the only gate left standing.
        let tick = w.tick_count;
        set_next_move(&mut w, pet, tick);
        assert!(movement::can_move_now(&w, pet));
        orders::do_order_none(&mut w, pet);
    }

    assert_eq!(w.mobile(pet).unwrap().direction, Direction::NORTH); // gated by CheckIdle -> never attempts a step
    assert_eq!(loc(&w, pet), start);
}

// ---- GuardFriendlyFireTests ----

/// `Spawn()`: an owner at (1500,1600) on Felucca and a guarding pet beside it, its timer stopped.
fn spawn_ff(w: &mut World) -> (MobileId, MobileId) {
    let owner = real_player(w);
    place(w, owner, 1500, 1600, MapId::FELUCCA);
    let pet = other_pet(w, owner, 1499);
    crate::creature::control::issue_order(w, pet, OrderType::Guard, Some(owner), None);
    timer::pause(w, pet);
    (owner, pet)
}

fn other_pet(w: &mut World, owner: MobileId, x: i32) -> MobileId {
    let pet = pet_stub(w);
    place(w, pet, x, 1600, MapId::FELUCCA);
    set_control_master(w, pet, Some(owner));
    timer::pause(w, pet);
    pet
}

/// A free `PetTestStub` with its timer stopped.
fn loose_stub(w: &mut World, x: i32) -> MobileId {
    let m = pet_stub(w);
    place(w, m, x, 1600, MapId::FELUCCA);
    timer::pause(w, m);
    m
}

fn acquire_ten(w: &mut World, pet: MobileId) -> bool {
    let mode = cd(w, pet).unwrap().fight_mode;
    base::acquire_focus_mob(w, pet, 10, mode, false, false, true)
}

fn expire_reacquire(w: &mut World, pet: MobileId) {
    let tick = w.tick_count;
    set_next_reacquire(w, pet, tick - 1);
}

#[test]
fn guard_fallback_does_not_acquire_its_owner() {
    let mut w = world();
    let (owner, pet) = spawn_ff(&mut w);
    assert_eq!(target(&w, pet), None); // speech dispatch clears this; guard must not fall through to wild acquisition.
    assert!(muo_server::mobile::base::can_see_object(
        &mut w,
        pet,
        Some(muo_server::ids::EntityId::Mobile(owner))
    ));
    assert!(crate::mx::in_los(&mut w, pet, owner));
    expire_reacquire(&mut w, pet);
    assert!(!acquire_ten_range(&mut w, pet));
    assert_eq!(focus(&w, pet), None);
    assert_eq!(combatant(&w, pet), None);
}

/// `AcquireFocusMob(pet.RangePerception, pet.FightMode, false, false, true)`.
fn acquire_ten_range(w: &mut World, pet: MobileId) -> bool {
    acquire(w, pet)
}

#[test]
fn guard_drops_stale_combat_against_another_pet_of_its_owner() {
    let mut w = world();
    let (owner, pet) = spawn_ff(&mut w);
    let sibling = other_pet(&mut w, owner, 1498);
    crate::creature::control::issue_order(
        &mut w,
        pet,
        OrderType::Attack,
        Some(owner),
        Some(sibling),
    );
    assert_eq!(combatant(&w, pet), Some(sibling));
    crate::creature::control::issue_order(&mut w, pet, OrderType::Guard, Some(owner), None);
    orders::do_order_guard(&mut w, pet);
    assert_eq!(combatant(&w, pet), None);
    assert_eq!(focus(&w, pet), None);
    assert_eq!(order(&w, pet), OrderType::Guard);
}

#[test]
fn guard_does_not_retaliate_against_owner_friendly_damage() {
    let mut w = world();
    let (owner, pet) = spawn_ff(&mut w);
    muo_server::mobile::base::aggressive_action(&mut w, pet, owner);
    assert_eq!(combatant(&w, pet), None);
    assert_eq!(order(&w, pet), OrderType::Guard);
}

#[test]
fn guard_does_not_retaliate_against_sibling_friendly_damage() {
    let mut w = world();
    let (owner, pet) = spawn_ff(&mut w);
    let sibling = other_pet(&mut w, owner, 1498);
    muo_server::mobile::base::aggressive_action(&mut w, pet, sibling);
    assert_eq!(combatant(&w, pet), None);
}

#[test]
fn guard_reacquires_an_enemy_attacking_its_owner() {
    let mut w = world();
    let (owner, pet) = spawn_ff(&mut w);
    let enemy = loose_stub(&mut w, 1498);
    crate::mx::set_combatant(&mut w, enemy, Some(owner));
    assert_eq!(combatant(&w, enemy), Some(owner));
    assert!(acquire_ten(&mut w, pet));
    assert_eq!(focus(&w, pet), Some(enemy));
}

#[test]
fn guard_ignores_sibling_in_owners_aggressor_history() {
    let mut w = world();
    let (owner, pet) = spawn_ff(&mut w);
    let sibling = other_pet(&mut w, owner, 1498);
    muo_server::mobile::base::aggressive_action(&mut w, owner, sibling);
    assert!(
        w.mobile(owner)
            .unwrap()
            .aggressors
            .iter()
            .any(|e| e.attacker == sibling)
    );
    assert!(!acquire_ten(&mut w, pet));
    assert_eq!(focus(&w, pet), None);
}

#[test]
fn guard_ignores_own_controlled_summon_attacking_owner() {
    let mut w = world();
    let (owner, pet) = spawn_ff(&mut w);
    let summon = other_pet(&mut w, owner, 1498);
    set_summoned(&mut w, summon, true);
    set_master(&mut w, summon, Some(owner));
    crate::mx::set_combatant(&mut w, summon, Some(owner));
    assert_eq!(combatant(&w, summon), Some(owner));
    muo_server::mobile::base::aggressive_action(&mut w, pet, summon);
    assert_eq!(combatant(&w, pet), None);
    assert!(!acquire_ten(&mut w, pet));
    assert_eq!(focus(&w, pet), None);
}

#[test]
fn guard_defends_against_own_uncontrolled_summon_attacking_owner() {
    let mut w = world();
    let (owner, pet) = spawn_ff(&mut w);
    let vortex = loose_stub(&mut w, 1498);
    set_summoned(&mut w, vortex, true);
    set_master(&mut w, vortex, Some(owner));
    crate::mx::set_combatant(&mut w, vortex, Some(owner));
    assert_eq!(combatant(&w, vortex), Some(owner));
    assert!(acquire_ten(&mut w, pet));
    assert_eq!(focus(&w, pet), Some(vortex));
}

#[test]
fn bard_provocation_still_overrides_guard() {
    let mut w = world();
    let (owner, pet) = spawn_ff(&mut w);
    {
        let d = crate::creature::data::cd_mut(&mut w, pet).unwrap();
        d.bard_provoked = true;
        d.bard_target = Some(owner);
    }
    assert!(acquire_ten(&mut w, pet));
    assert_eq!(focus(&w, pet), Some(owner));
}

#[test]
fn non_combat_orders_do_not_acquire_their_friendly_target() {
    for o in [OrderType::Follow, OrderType::Come, OrderType::Stay] {
        let mut w = world();
        let (owner, pet) = spawn_ff(&mut w);
        crate::creature::control::issue_order(&mut w, pet, o, Some(owner), Some(owner));
        assert!(!acquire_ten(&mut w, pet), "{o:?}");
        assert_eq!(focus(&w, pet), None, "{o:?}");
    }
}

#[test]
fn guarding_mage_does_not_turn_on_owner_when_enemy_disappears() {
    let mut w = world();
    let (owner, _) = spawn_ff(&mut w);
    let mage = new_creature(&mut w, &PET_STUB, AIType::AiMage, FightMode::Closest, 10, 1);
    w.mobile_mut(mage).unwrap().body = Body(0xC8);
    place(&mut w, mage, 1498, 1600, MapId::FELUCCA);
    set_control_master(&mut w, mage, Some(owner));
    crate::creature::control::issue_order(&mut w, mage, OrderType::Guard, Some(owner), None);
    timer::pause(&mut w, mage);
    let enemy = loose_stub(&mut w, 1497);
    crate::mx::set_combatant(&mut w, mage, Some(enemy));
    set_hidden(&mut w, enemy, true);
    expire_reacquire(&mut w, mage);
    assert!(std::ptr::eq(class_of(&w, mage).unwrap(), &mage::MAGE_AI));
    (class_of(&w, mage).unwrap().do_action_combat)(&mut w, mage);
    assert_ne!(combatant(&w, mage), Some(owner));
    assert_eq!(focus(&w, mage), None);
    orders::do_order_guard(&mut w, mage);
    assert_eq!(combatant(&w, mage), None);
}

// ---- InnUpstairs prefab: ApproachTargetTests' counter scenes ----

use muo_server::map::test_support::{INN_ANCHOR, INN_TARGET};

fn place_z(w: &mut World, m: MobileId, p: (i32, i32, i32)) {
    muo_server::mobile::base::move_to_world(w, m, Point3D::new(p.0, p.1, p.2), MapId::TRAMMEL);
}

fn inn_pair(w: &mut World) -> (MobileId, MobileId) {
    muo_server::map::test_support::stamp_inn_upstairs(w, MapId::TRAMMEL);
    let bc = pet_stub(w);
    place_z(w, bc, INN_ANCHOR);
    timer::pause(w, bc); // we drive movement manually; no background timer
    let target = w.new_mobile(&MOBILE_BASE);
    w.mobile_mut(target).unwrap().body = Body(0xC9);
    place_z(w, target, INN_TARGET);
    (bc, target)
}

// An upstairs room with an L-shaped counter: the pet is south of it, the master north.
#[test]
fn inn_counter_pet_reaches_master() {
    let mut w = world();
    let (bc, target) = inn_pair(&mut w);

    // The straight line north runs into the counter, so arriving means the pet went round it.
    let below_counter = Point3D::new(INN_ANCHOR.0, INN_TARGET.1 + 6, INN_ANCHOR.2);
    assert!(
        !muo_server::movement::check_movement_at(
            &mut w,
            bc,
            MapId::TRAMMEL,
            below_counter,
            Direction::NORTH
        )
        .0
    );

    // 200 ticks is generous for the ~20-step detour at one step per tick.
    assert!(
        drive_follow(&mut w, bc, target, 2, 200),
        "pet must navigate around the L-shaped counter to reach the master"
    );
}

#[test]
fn move_to_reaches_target_around_counter() {
    let mut w = world();
    let (bc, target) = inn_pair(&mut w);

    let mut arrived = false;
    for _ in 0..200 {
        set_next_move(&mut w, bc, 0);
        movement::move_to(&mut w, bc, target, 1);
        if in_range(&w, bc, target, 1) {
            arrived = true;
            break;
        }
    }

    assert!(
        arrived,
        "MoveTo chaser must reach the target around the counter"
    );
}

// ---- GuardFollowTests ----

#[test]
fn guard_follow_steps_toward_master_and_registers_move_intent() {
    let mut w = world();
    // Pet 6 tiles east of the master, open terrain.
    let master = real_player(&mut w);
    place(&mut w, master, 1494, 1600, MapId::TRAMMEL);
    let pet = pet_stub(&mut w);
    place(&mut w, pet, 1500, 1600, MapId::TRAMMEL);
    set_control_master(&mut w, pet, Some(master));

    timer::pause(&mut w, pet); // drive manually
    set_order(&mut w, pet, OrderType::Guard);
    timer::pause(&mut w, pet); // the order change may restart the timer

    let start = loc(&w, pet);
    set_next_move(&mut w, pet, 0);
    orders::obey(&mut w, pet);

    assert!(
        loc(&w, pet) != start,
        "a guarding pet beyond guard range must step toward its master"
    );
    // Without a move intent, guard-following only steps on the think grid.
    assert!(
        movement::try_get_move_wake(&w, pet).is_some(),
        "guard-following must register a move intent"
    );

    // The return is paced by FollowMoveSpeed; the think clock stays on the active value
    // and the per-step speed flip must not undo either.
    assert_eq!(cd(&w, pet).unwrap().current_speed, 0.2);
    let follow = (ctype(&w, pet).unwrap().follow_move_speed)(&w, pet);
    assert_eq!(speeds::current_move_speed(&w, pet), follow);
}

// ---- PetPacingTests: pet order handlers own the speed clocks ----

fn current_speed(w: &World, pet: MobileId) -> f64 {
    cd(w, pet).unwrap().current_speed
}

fn follow_move_speed(w: &World, pet: MobileId) -> f64 {
    (ctype(w, pet).unwrap().follow_move_speed)(w, pet)
}

// Movement orders run active, resting orders run passive; the move clock follows.
#[test]
fn order_issue_sets_think_clock() {
    let mut w = world();
    let (master, pet) = spawn_pair(&mut w, (1000, 1000), (1001, 1000));
    speeds::set_move_speed(&mut w, pet, 0.3, 0.9);
    speeds::set_current_speed_to_passive(&mut w, pet);

    set_order(&mut w, pet, OrderType::Come);
    assert_eq!(current_speed(&w, pet), 0.2);
    assert_eq!(speeds::current_move_speed(&w, pet), 0.3); // verbatim active -> activeMove

    set_order(&mut w, pet, OrderType::Stay);
    assert_eq!(current_speed(&w, pet), 0.4);
    assert_eq!(speeds::current_move_speed(&w, pet), 0.9);

    set_target(&mut w, pet, Some(master));
    set_order(&mut w, pet, OrderType::Follow);
    assert_eq!(current_speed(&w, pet), 0.2);

    set_order(&mut w, pet, OrderType::Guard);
    assert_eq!(current_speed(&w, pet), 0.2);
}

// The follow pace caps the step delay and leaves the think clock on the active value.
#[test]
fn follow_master_paces_steps_without_inflating_the_think_clock() {
    let mut w = world();
    let (master, pet) = spawn_pair(&mut w, (1000, 1000), (1001, 1000));
    speeds::set_move_speed(&mut w, pet, 0.3, 0.9);

    set_target(&mut w, pet, Some(master));
    set_order(&mut w, pet, OrderType::Follow); // fixture era is EJ

    assert_eq!(current_speed(&w, pet), 0.2); // active think, not the follow pace
    assert_eq!(
        speeds::current_move_speed(&w, pet),
        follow_move_speed(&w, pet)
    ); // capped at the follow pace
}

// A creature configured faster than the follow pace keeps its own.
#[test]
fn follow_master_keeps_a_faster_configured_pace() {
    let mut w = world();
    let (master, pet) = spawn_pair(&mut w, (1000, 1000), (1001, 1000));
    let faster = follow_move_speed(&w, pet) / 2.0;
    speeds::set_move_speed(&mut w, pet, faster, 0.9);

    set_target(&mut w, pet, Some(master));
    set_order(&mut w, pet, OrderType::Follow);

    assert_eq!(speeds::current_move_speed(&w, pet), faster);
}

// The move-clock override survives the order: it is capped while following, not overwritten.
#[test]
fn follow_master_leaves_the_configured_move_clock_alone() {
    let mut w = world();
    let (master, pet) = spawn_pair(&mut w, (1000, 1000), (1001, 1000));
    speeds::set_move_speed(&mut w, pet, 0.3, 0.9);

    set_target(&mut w, pet, Some(master));
    set_order(&mut w, pet, OrderType::Follow);
    orders::obey(&mut w, pet);
    set_order(&mut w, pet, OrderType::Stay);

    assert_eq!(cd(&w, pet).unwrap().active_move_speed, 0.3);
    assert_eq!(cd(&w, pet).unwrap().passive_move_speed, 0.9);
    assert_eq!(speeds::current_move_speed(&w, pet), 0.9); // resting on its own passive pace again
}

// Pre-AOS pets follow at their own pace; nothing caps them.
static SPRINTING_C: crate::creature::vtable::CreatureType = crate::creature::vtable::CreatureType {
    follow_move_speed: |_, _| 0.125,
    ..BASE_CREATURE_C
};
static SPRINTING_PET: MobileType = MobileType {
    full_name: "Server.Tests.SprintingPet",
    parent: Some(&BASE_CREATURE),
    family: Some(&SPRINTING_C),
    ..BASE_CREATURE
};

// A shard paces follows in any era by overriding the property, not by patching the AI.
#[test]
fn follow_move_speed_override_paces_follows_in_any_era() {
    let mut w = world();
    w.core.expansion = Some(muo_data::expansion::Expansion::UOR);
    let master = real_player(&mut w);
    place(&mut w, master, 1000, 1000, MapId::FELUCCA);
    let pet = typed_stub(&mut w, &SPRINTING_PET);
    place(&mut w, pet, 1001, 1000, MapId::FELUCCA);
    set_control_master(&mut w, pet, Some(master));
    speeds::set_move_speed(&mut w, pet, 0.3, 0.9);

    set_target(&mut w, pet, Some(master));
    set_order(&mut w, pet, OrderType::Follow);

    assert_eq!(speeds::current_move_speed(&w, pet), 0.125);
}

// A guarding pet outside guard range closes at the follow pace, thinking on its active clock.
#[test]
fn guard_return_paces_steps_without_inflating_the_think_clock() {
    let mut w = world();
    let (_, pet) = spawn_pair(&mut w, (1000, 1000), (1006, 1000));
    speeds::set_move_speed(&mut w, pet, 0.3, 0.9);

    set_order(&mut w, pet, OrderType::Guard);

    assert_eq!(current_speed(&w, pet), 0.2);
    assert_eq!(
        speeds::current_move_speed(&w, pet),
        follow_move_speed(&w, pet)
    );
}

// Obeying the follow order must not write the pace into either clock.
#[test]
fn follow_master_obey_keeps_the_think_clock_active() {
    let mut w = world();
    let (master, pet) = spawn_pair(&mut w, (1000, 1000), (1001, 1000));
    speeds::set_move_speed(&mut w, pet, 0.3, 0.9);
    timer::pause(&mut w, pet);

    set_target(&mut w, pet, Some(master));
    set_order(&mut w, pet, OrderType::Follow); // fixture era is EJ
    orders::obey(&mut w, pet);

    assert_eq!(current_speed(&w, pet), 0.2);
    assert_eq!(
        speeds::current_move_speed(&w, pet),
        follow_move_speed(&w, pet)
    );
}

// At the master's side a guarding pet stays active: no stale-warmode passive, no sprint.
#[test]
fn guard_at_masters_side_is_active() {
    let mut w = world();
    let (_, pet) = spawn_pair(&mut w, (1000, 1000), (1001, 1000));
    speeds::set_move_speed(&mut w, pet, 0.3, 0.9);
    timer::pause(&mut w, pet);
    speeds::set_current_speed_to_passive(&mut w, pet);

    set_order(&mut w, pet, OrderType::Guard);
    orders::obey(&mut w, pet); // nothing to guard against, master adjacent

    assert_eq!(current_speed(&w, pet), 0.2);
    assert_eq!(speeds::current_move_speed(&w, pet), 0.3);
}

// `ThinkProbe`: counts OnThink; `ForceIdle` is on, so there is no wandering and the cadence is pure.
thread_local! {
    static THINKS: std::cell::Cell<i32> = const { std::cell::Cell::new(0) };
}
static PROBE_C: crate::creature::vtable::CreatureType = crate::creature::vtable::CreatureType {
    on_think: |w, id| {
        THINKS.set(THINKS.get() + 1);
        (BASE_CREATURE_C.on_think)(w, id)
    },
    check_idle: |_, _| true,
    ..BASE_CREATURE_C
};
static PROBE_PET: MobileType = MobileType {
    full_name: "Server.Tests.ThinkProbe",
    parent: Some(&BASE_CREATURE),
    family: Some(&PROBE_C),
    ..BASE_CREATURE
};

/// `TimerWheelScope.RunFor`: the clock advances by the wheel's step, turning it every step.
fn run_for(w: &mut World, ms: i64) {
    let deadline = w.tick_count + ms;
    while w.tick_count - deadline < 0 {
        w.tick_count += muo_server::timer::TICK_RATE;
        w.now += muo_primitives::TimeSpan::from_millis(muo_server::timer::TICK_RATE);
        muo_server::timer::slice(w);
    }
}

/// `TimerWheelScope.RunUntil`.
fn run_until(w: &mut World, mut condition: impl FnMut(&mut World) -> bool, max_ms: i64) -> bool {
    let deadline = w.tick_count + max_ms;
    while w.tick_count - deadline < 0 {
        if condition(w) {
            return true;
        }
        w.tick_count += muo_server::timer::TICK_RATE;
        w.now += muo_primitives::TimeSpan::from_millis(muo_server::timer::TICK_RATE);
        muo_server::timer::slice(w);
    }
    condition(w)
}

// Runs past the spawn stagger; returns right after a think with the next 0.4s away.
fn settled_probe(w: &mut World) -> (MobileId, MobileId) {
    THINKS.set(0);
    let master = real_player(w);
    place(w, master, 1000, 1000, MapId::FELUCCA);
    let pet = typed_stub(w, &PROBE_PET);
    place(w, pet, 1001, 1000, MapId::FELUCCA);
    set_control_master(w, pet, Some(master));
    set_order(w, pet, OrderType::Stay);

    let settled = run_until(w, |_| THINKS.get() >= 2, 8000);
    assert!(settled, "the AI must reach a steady think cadence");
    (master, pet)
}

#[test]
fn order_change_wakes_stale_think_timer() {
    let mut w = world();
    let (master, pet) = settled_probe(&mut w);
    let thinks_before = THINKS.get();

    run_for(&mut w, 200); // mid-wait, next think ~200ms out
    assert_eq!(THINKS.get(), thinks_before);

    set_target(&mut w, pet, Some(master));
    set_order(&mut w, pet, OrderType::Follow);

    run_for(&mut w, 80);
    assert!(
        THINKS.get() > thinks_before,
        "a fresh order must wake the AI promptly"
    );
}

#[test]
fn speed_up_reschedules_pending_wake() {
    let mut w = world();
    let (_, pet) = settled_probe(&mut w);
    let thinks_before = THINKS.get();

    run_for(&mut w, 200); // mid-wait, next think ~200ms out
    assert_eq!(THINKS.get(), thinks_before);

    speeds::set_current_speed(&mut w, pet, 0.1);

    run_for(&mut w, 120);
    assert!(
        THINKS.get() > thinks_before,
        "a speed-up must reschedule the pending wake"
    );
}

// ---- DetectHiddenTimerTests ----

thread_local! {
    static DETECTIONS: std::cell::Cell<i32> = const { std::cell::Cell::new(0) };
}

/// `DetectingAI`: Think() is true, DetectHidden() only counts.
static DETECTING_AI: AiClass = AiClass {
    name: "DetectingAI",
    think: |_, _| true,
    detect_hidden: |_, _| DETECTIONS.set(DETECTIONS.get() + 1),
    ..base::BASE_AI
};
static DETECTING_C: crate::creature::vtable::CreatureType = crate::creature::vtable::CreatureType {
    player_range_sensitive: |_, _| false,
    forced_ai: |_, _| Some(&DETECTING_AI),
    ..BASE_CREATURE_C
};
static DETECTING_CREATURE: MobileType = MobileType {
    full_name: "Server.Tests.DetectingCreature",
    parent: Some(&BASE_CREATURE),
    family: Some(&DETECTING_C),
    ..BASE_CREATURE
};

/// `SkillsInfo.Configure()`: the 58 skills (no stat scales: a skill's value is its base).
fn skill_table(w: &mut World) {
    let info = |i: i32| muo_data::skill_info::SkillInfo {
        skill_id: i,
        name: format!("S{i}"),
        title: String::new(),
        str_scale: 0.0,
        dex_scale: 0.0,
        int_scale: 0.0,
        stat_total: 0.0,
        str_gain: 0.0,
        dex_gain: 0.0,
        int_gain: 0.0,
        gain_factor: 0.0,
        profession_skill_name: String::new(),
        primary_stat: muo_data::skill_info::Stat::Str,
        secondary_stat: muo_data::skill_info::Stat::Dex,
    };
    w.resources.get_mut::<muo_server::skills::SkillTable>().info = (0..58).map(info).collect();
}

#[test]
fn detect_hidden_schedules_bounded_cooldown_and_waits_before_retry() {
    for (intelligence, min_delay, max_delay) in [
        (0, 108000, 132000),
        (1, 108000, 132000),
        (100, 108000, 132000),
        (250, 108000, 132000),
        (251, 107100, 130900),
        (1000, 27000, 33000),
        (30000, 900, 1100),
    ] {
        let mut w = world();
        skill_table(&mut w);
        DETECTIONS.set(0);
        let creature = typed_stub(&mut w, &DETECTING_CREATURE);
        muo_server::mobile::base::set_int(&mut w, creature, intelligence.max(1));
        if intelligence == 0 {
            let m = muo_server::mods::StatMod::new(
                &mut w,
                muo_server::mobile::StatType::INT,
                Some("DetectHiddenZeroInt"),
                -1,
                muo_primitives::TimeSpan::ZERO,
                None,
            );
            muo_server::mobile::base::add_stat_mod(&mut w, creature, m);
        }
        let int = (w.mobile(creature).unwrap().ty.int)(&w, creature);
        assert_eq!(int, intelligence, "row {intelligence}");
        muo_server::skills::set_base(
            &mut w,
            creature,
            muo_server::skills::SkillName::DetectHidden,
            100.0,
        );
        place(&mut w, creature, 1500, 1600, MapId::FELUCCA);
        timer::pause(&mut w, creature);
        timer::activate(&mut w, creature);

        assert!(
            run_until(&mut w, |_| DETECTIONS.get() > 0, 1024),
            "The AI timer did not reach the expected detection."
        );
        assert_eq!(DETECTIONS.get(), 1);
        let next = ai(&w, creature).unwrap().next_detect_hidden;
        assert!(
            (min_delay..=max_delay).contains(&(next - w.tick_count)),
            "row {intelligence}: {}",
            next - w.tick_count
        );
        assert!(timer::running(&w, creature));

        // Stop one wheel step short of the cooldown: no retry may have fired yet.
        let rest = next - muo_server::timer::TICK_RATE - w.tick_count;
        run_for(&mut w, rest);

        assert_eq!(DETECTIONS.get(), 1);
        assert!(
            run_until(&mut w, |_| DETECTIONS.get() > 1, 1024),
            "The AI timer did not reach the expected detection."
        );
        assert_eq!(DETECTIONS.get(), 2);
        let next = ai(&w, creature).unwrap().next_detect_hidden;
        assert!(
            (min_delay..=max_delay).contains(&(next - w.tick_count)),
            "row {intelligence}"
        );
    }
}
