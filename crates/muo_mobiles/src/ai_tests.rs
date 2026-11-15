//! AI tests that need real classes of this crate: the familiar AI (the five familiars, timer-wheel driven on the
//! invented tile kit) and the summon acquire rules (the EnergyVortex).

use muo_content::ai::{self, movement};
use muo_content::creature::control::{issue_order, set_control_master, set_master, set_summoned};
use muo_content::creature::enums::{AIType, FightMode, OrderType};
use muo_content::creature::setters::new_creature;
use muo_content::creature::{BASE_CREATURE, BASE_CREATURE_C, cd, cd_mut};
use muo_content::mx;
use muo_primitives::{Body, Point2D, Point3D, TimeSpan};
use muo_server::World;
use muo_server::ids::MobileId;
use muo_server::map::MapId;
use muo_server::map::test_support;
use muo_server::mobile::part1::props_b::{set_hidden, set_warmode};
use muo_server::mobile::{MOBILE_BASE, MobileType};

use crate::monsters::leaves::ENERGY_VORTEX;
use crate::npcs::familiars::{
    DARK_WOLF_FAMILIAR, DEATH_ADDER, HORDE_MINION_FAMILIAR, SHADOW_WISP_FAMILIAR,
    VAMPIRE_BAT_FAMILIAR,
};

fn world() -> World {
    let mut w = test_support::world();
    // the familiars' templates use the NPC keys (`npcSpeed`)
    crate::template_keys_npcs::register(&mut w.types);
    ai::configure_for_tests(&mut w);
    w
}

const TRAMMEL: MapId = MapId::TRAMMEL;

fn place(w: &mut World, m: MobileId, x: i32, y: i32, map: MapId) {
    muo_server::mobile::base::move_to_world(w, m, Point3D::new(x, y, 0), map);
}

fn loc(w: &World, m: MobileId) -> Point3D {
    w.mobile(m).unwrap().location
}

fn in_range(w: &World, a: MobileId, b: MobileId, range: i32) -> bool {
    let (la, lb) = (loc(w, a), loc(w, b));
    (la.x - lb.x).abs().max((la.y - lb.y).abs()) <= range
}

fn dist(w: &World, a: MobileId, b: MobileId) -> f64 {
    let (la, lb) = (loc(w, a), loc(w, b));
    f64::from((la.x - lb.x).pow(2) + (la.y - lb.y).pow(2)).sqrt()
}

/// `TimerWheelScope.RunFor`.
fn run_for(w: &mut World, ms: i64) {
    let deadline = w.tick_count + ms;
    while w.tick_count - deadline < 0 {
        w.tick_count += muo_server::timer::TICK_RATE;
        w.now += TimeSpan::from_millis(muo_server::timer::TICK_RATE);
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
        w.now += TimeSpan::from_millis(muo_server::timer::TICK_RATE);
        muo_server::timer::slice(w);
    }
    condition(w)
}

// ---- FamiliarAITests ----

/// A player on the Trammel kit: `Player = true`, body 0x190, 100 in every stat.
fn master(w: &mut World, x: i32, y: i32) -> MobileId {
    let p = muo_content::player_mobile::new_player_mobile(w);
    {
        let m = w.mobile_mut(p).unwrap();
        m.player = true;
        m.body = Body(0x190);
        m.str = 100;
        m.dex = 100;
        m.int = 100;
    }
    place(w, p, x, y, TRAMMEL);
    p
}

/// 0 dark wolf, 1 vampire bat, 2 shadow wisp, 3 death adder, 4 horde minion; summoned for an hour.
fn familiar(w: &mut World, kind: i32, master: MobileId, x: i32, y: i32) -> MobileId {
    let ty: &'static MobileType = match kind {
        0 => &DARK_WOLF_FAMILIAR,
        1 => &VAMPIRE_BAT_FAMILIAR,
        2 => &SHADOW_WISP_FAMILIAR,
        3 => &DEATH_ADDER,
        _ => &HORDE_MINION_FAMILIAR,
    };
    let f = (ty.make.unwrap())(w);
    assert!(muo_content::creature::control::summon(
        w,
        f,
        true,
        master,
        Point3D::new(x, y, 0),
        -1,
        TimeSpan::from_secs(3600),
        None
    ));
    f
}

/// `EnemyStub`: a plain Mobile, body 0x190, Str 100.
fn enemy(w: &mut World, x: i32, y: i32) -> MobileId {
    let e = w.new_mobile(&MOBILE_BASE);
    {
        let m = w.mobile_mut(e).unwrap();
        m.body = Body(0x190);
        m.str = 100;
    }
    place(w, e, x, y, TRAMMEL);
    e
}

fn combatant(w: &World, m: MobileId) -> Option<MobileId> {
    mx::combatant(w, m)
}

fn warmode(w: &World, m: MobileId) -> bool {
    w.mobile(m).unwrap().warmode
}

fn hidden(w: &World, m: MobileId) -> bool {
    w.mobile(m).unwrap().hidden
}

fn order(w: &World, f: MobileId) -> OrderType {
    cd(w, f).unwrap().control_order
}

fn range_of(
    w: &World,
    f: MobileId,
    f_range: impl Fn(&muo_content::creature::CreatureData) -> i32,
) -> i32 {
    f_range(cd(w, f).unwrap())
}

#[test]
fn familiar_follows_without_backtracking() {
    let mut w = world();
    let p = master(&mut w, 1500, 1600);
    let f = familiar(&mut w, 0, p, 1500, 1600);
    run_for(&mut w, 400); // past the activation spread
    place(&mut w, p, 1494, 1600, TRAMMEL);

    let mut best = dist(&w, f, p);
    let mut regressed = false;
    let arrived = run_until(
        &mut w,
        |w| {
            let d = dist(w, f, p);
            if d > best + 0.01 {
                regressed = true;
            }
            best = best.min(d);
            in_range(w, f, p, 1)
        },
        3000,
    );

    assert!(arrived, "familiar must reach range 1 of its master");
    assert!(
        !regressed,
        "familiar must never step away from the master while following"
    );
    assert_eq!(order(&w, f), OrderType::Come);
    assert_eq!(cd(&w, f).unwrap().home, Point3D::ZERO);
}

#[test]
fn familiar_orders_are_inert() {
    let mut w = world();
    let p = master(&mut w, 1500, 1600);
    let f = familiar(&mut w, 0, p, 1500, 1600);
    run_for(&mut w, 400);

    issue_order(&mut w, f, OrderType::Stay, Some(p), None);

    assert_eq!(cd(&w, f).unwrap().home, Point3D::ZERO);
    assert_eq!(cd(&w, f).unwrap().current_speed, 0.1);

    place(&mut w, p, 1495, 1600, TRAMMEL);
    assert!(
        run_until(&mut w, |w| in_range(w, f, p, 1), 3000),
        "a familiar under a Stay order still follows"
    );
}

#[test]
fn familiar_assists_masters_target_only_if_combat_capable() {
    for (kind, assists) in [(0, true), (1, true), (2, false), (3, false), (4, true)] {
        let mut w = world();
        let p = master(&mut w, 1500, 1600);
        let f = familiar(&mut w, kind, p, 1501, 1600);
        let e = enemy(&mut w, 1495, 1600);
        run_for(&mut w, 400);

        set_warmode(&mut w, p, true);
        mx::set_combatant(&mut w, p, Some(e));

        if assists {
            let fight = range_of(&w, f, |d| d.range_fight);
            assert!(
                run_until(
                    &mut w,
                    |w| combatant(w, f) == Some(e) && in_range(w, f, e, fight),
                    4000
                ),
                "a combat familiar must engage the caster's target (kind {kind})"
            );
            assert!(warmode(&w, f));
            assert_eq!(order(&w, f), OrderType::Come);
        } else {
            run_for(&mut w, 2000);
            assert_eq!(combatant(&w, f), None, "kind {kind}");
            assert!(!warmode(&w, f));
            assert!(
                in_range(&w, f, p, 1),
                "a non-combat familiar stays with the caster"
            );
        }
    }
}

#[test]
fn familiar_drops_target_when_master_hides() {
    let mut w = world();
    let p = master(&mut w, 1500, 1600);
    let f = familiar(&mut w, 0, p, 1501, 1600);
    let e = enemy(&mut w, 1495, 1600);
    run_for(&mut w, 400);
    set_warmode(&mut w, p, true);
    mx::set_combatant(&mut w, p, Some(e));
    assert!(run_until(&mut w, |w| combatant(w, f) == Some(e), 2000));

    set_hidden(&mut w, p, true);
    assert!(run_until(
        &mut w,
        |w| combatant(w, f).is_none() && hidden(w, f),
        1000
    ));
}

#[test]
fn familiar_leash_never_engages_a_target_far_from_the_master() {
    let mut w = world();
    let p = master(&mut w, 1500, 1600);
    let f = familiar(&mut w, 0, p, 1501, 1600);
    let rp = range_of(&w, f, |d| d.range_perception);
    let e = enemy(&mut w, 1500 - rp - 4, 1600); // beyond the leash
    run_for(&mut w, 400);
    set_warmode(&mut w, p, true);
    mx::set_combatant(&mut w, p, Some(e));
    run_for(&mut w, 4000);

    assert_eq!(combatant(&w, f), None);
    assert!(
        in_range(&w, f, p, 1),
        "assist must not carry the familiar past the leash"
    );
}

#[test]
fn familiar_retaliates_if_combat_capable_and_stays_on_come_order() {
    let mut w = world();
    let p = master(&mut w, 1500, 1600);
    let wolf = familiar(&mut w, 0, p, 1501, 1600);
    let wisp = familiar(&mut w, 2, p, 1499, 1600);
    let e = enemy(&mut w, 1503, 1600);
    run_for(&mut w, 400);

    // Combatant setter -> DoHarmful -> AggressiveAction with ChangingCombatant: the path that issues a stand-down
    // pet an Attack order.
    mx::set_combatant(&mut w, e, Some(wolf));
    assert!(
        run_until(&mut w, |w| combatant(w, wolf) == Some(e), 1000),
        "a combat familiar fights back"
    );
    assert_eq!(order(&w, wolf), OrderType::Come);

    mx::set_combatant(&mut w, e, Some(wisp));
    run_for(&mut w, 1000);
    assert_eq!(combatant(&w, wisp), None);
    assert!(!warmode(&w, wisp));
}

#[test]
fn familiar_keep_up_snaps_when_far_on_open_ground() {
    let mut w = world();
    let p = master(&mut w, 1500, 1600);
    let f = familiar(&mut w, 0, p, 1500, 1600);
    run_for(&mut w, 400);

    place(
        &mut w,
        p,
        1500 - muo_content::ai::familiar::KEEP_UP_RANGE - 2,
        1600,
        TRAMMEL,
    );
    run_for(&mut w, 300); // one think with a greedy step

    assert!(
        in_range(&w, f, p, 1),
        "familiar must snap adjacent to a master 12 tiles away on open ground"
    );
}

#[test]
fn familiar_keep_up_does_not_snap_while_routing() {
    // An upstairs room with an L-shaped counter: master north, familiar south; ~20-step detour.
    let mut w = world();
    test_support::stamp_inn_upstairs(&mut w, TRAMMEL);
    let (t, a) = (test_support::INN_TARGET, test_support::INN_ANCHOR);
    let p = master(&mut w, t.0, t.1);
    let f = familiar(&mut w, 0, p, a.0, a.1);
    muo_server::mobile::base::move_to_world(&mut w, p, Point3D::new(t.0, t.1, t.2), TRAMMEL);
    muo_server::mobile::base::move_to_world(&mut w, f, Point3D::new(a.0, a.1, a.2), TRAMMEL);

    // Observed from the first tick so an early snap cannot hide behind "arrived".
    let mut teleported = false;
    let mut last = loc(&w, f);
    let arrived = run_until(
        &mut w,
        |w| {
            let now = loc(w, f);
            if now != last && (now.x - last.x).abs().max((now.y - last.y).abs()) > 1 {
                teleported = true;
            }
            last = now;
            in_range(w, f, p, 1)
        },
        8000,
    );

    assert!(arrived, "familiar walks the detour");
    assert!(
        !teleported,
        "a working detour must not be short-circuited by a snap"
    );
}

#[test]
fn familiar_keep_up_snaps_after_give_up() {
    let mut w = world();
    let p = master(&mut w, 1500, 1596);
    let f = familiar(&mut w, 0, p, 1500, 1602);

    // 5x5 ring, open 3x3 interior: a landing tile exists, no route reaches it.
    for x in 1498..=1502 {
        for y in 1594..=1598 {
            if x > 1498 && x < 1502 && y > 1594 && y < 1598 {
                continue;
            }
            test_support::add_static(&mut w, TRAMMEL, x, y, test_support::COUNTER, 0);
        }
    }
    run_for(&mut w, 400);

    assert!(
        run_until(&mut w, |w| in_range(w, f, p, 1), 15000),
        "after giving up on an unreachable master the familiar snaps to it"
    );
}

#[test]
fn familiar_mirrors_hidden_even_while_walking() {
    let mut w = world();
    let p = master(&mut w, 1500, 1600);
    let f = familiar(&mut w, 0, p, 1500, 1600);
    run_for(&mut w, 400);

    set_hidden(&mut w, p, true);
    place(&mut w, p, 1495, 1600, TRAMMEL);
    assert!(run_until(&mut w, |w| hidden(w, f), 500));

    let mut stayed_hidden = true;
    let arrived = run_until(
        &mut w,
        |w| {
            stayed_hidden &= hidden(w, f);
            in_range(w, f, p, 1)
        },
        3000,
    );

    assert!(arrived);
    assert!(stayed_hidden, "steps must not strip the mirror");

    set_hidden(&mut w, p, false);
    assert!(run_until(&mut w, |w| !hidden(w, f), 500));
}

#[test]
fn familiar_herding_target_location_wins_and_clears() {
    let mut w = world();
    let p = master(&mut w, 1500, 1600);
    let f = familiar(&mut w, 4, p, 1500, 1600);
    run_for(&mut w, 400);

    let goal = Point2D::new(1500, 1594);
    cd_mut(&mut w, f).unwrap().target_location = Some(goal);

    assert!(
        run_until(
            &mut w,
            |w| cd(w, f).unwrap().target_location.is_none(),
            6000
        ),
        "familiar walks to the herding target"
    );
    let l = loc(&w, f);
    assert!((l.x - goal.x).abs().max((l.y - goal.y).abs()) <= 1);
}

#[test]
fn non_combat_familiar_refuses_any_combatant() {
    let mut w = world();
    let p = master(&mut w, 1500, 1600);
    let wisp = familiar(&mut w, 2, p, 1501, 1600);
    let e = enemy(&mut w, 1502, 1600);

    mx::set_combatant(&mut w, wisp, Some(e));
    assert_eq!(combatant(&w, wisp), None);
}

#[test]
fn hidden_caster_familiar_refuses_retaliation() {
    let mut w = world();
    let p = master(&mut w, 1500, 1600);
    let wolf = familiar(&mut w, 0, p, 1501, 1600);
    let e = enemy(&mut w, 1502, 1600);
    run_for(&mut w, 400);
    set_hidden(&mut w, p, true);
    assert!(run_until(&mut w, |w| hidden(w, wolf), 500));

    mx::set_combatant(&mut w, e, Some(wolf));

    // Synchronous: the veto is at the setter.
    assert_eq!(combatant(&w, wolf), None);
    assert!(!warmode(&w, wolf));
    run_for(&mut w, 500);
    assert_eq!(combatant(&w, wolf), None);
    assert!(hidden(&w, wolf));
}

#[test]
fn left_behind_familiar_stands_down() {
    let mut w = world();
    let p = master(&mut w, 1500, 1600);
    let wolf = familiar(&mut w, 0, p, 1501, 1600);
    let e = enemy(&mut w, 1495, 1600);
    run_for(&mut w, 400);
    set_warmode(&mut w, p, true);
    mx::set_combatant(&mut w, p, Some(e));
    assert!(run_until(&mut w, |w| combatant(w, wolf) == Some(e), 2000));

    place(&mut w, p, 1500, 1600, MapId::FELUCCA);
    assert!(run_until(
        &mut w,
        |w| combatant(w, wolf).is_none() && !warmode(w, wolf),
        500
    ));
}

#[test]
fn herding_outranks_combat_and_stands_down() {
    let mut w = world();
    let p = master(&mut w, 1500, 1600);
    let f = familiar(&mut w, 4, p, 1501, 1600);
    let e = enemy(&mut w, 1502, 1600);
    run_for(&mut w, 400);
    set_warmode(&mut w, p, true);
    mx::set_combatant(&mut w, p, Some(e));
    assert!(run_until(&mut w, |w| combatant(w, f) == Some(e), 2000));

    // Caster visible and fighting: only herding clears this.
    let goal = Point2D::new(1500, 1594);
    cd_mut(&mut w, f).unwrap().target_location = Some(goal);
    let d = |w: &World| {
        let l = loc(w, f);
        f64::from((l.x - goal.x).pow(2) + (l.y - goal.y).pow(2)).sqrt()
    };
    let dist_before = d(&w);
    run_for(&mut w, 300);

    assert_eq!(combatant(&w, f), None);
    assert!(!warmode(&w, f));
    assert!(cd(&w, f).unwrap().target_location.is_some());
    assert!(
        d(&w) < dist_before,
        "the fetch makes progress while combat is set aside"
    );
    assert_eq!(combatant(&w, p), Some(e));
}

#[test]
fn familiar_assists_against_what_the_casters_pet_is_fighting() {
    let mut w = world();
    let p = master(&mut w, 1500, 1600);
    let f = familiar(&mut w, 0, p, 1501, 1600);
    let e = enemy(&mut w, 1496, 1600);
    let pet = new_creature(
        &mut w,
        &PET_STUB,
        AIType::AiAnimal,
        FightMode::Closest,
        10,
        1,
    );
    w.mobile_mut(pet).unwrap().body = Body(0xC8);
    place(&mut w, pet, 1497, 1600, TRAMMEL);
    set_control_master(&mut w, pet, Some(p));
    run_for(&mut w, 400);

    // The pet attacks; the caster is credited indirectly without a Combatant.
    mx::set_combatant(&mut w, pet, Some(e));
    muo_server::mobile::base::do_harmful(&mut w, p, e, true);
    mx::set_combatant(&mut w, e, Some(pet));
    assert_eq!(combatant(&w, p), None);

    let fight = range_of(&w, f, |d| d.range_fight);
    assert!(
        run_until(
            &mut w,
            |w| combatant(w, f) == Some(e) && in_range(w, f, e, fight),
            4000
        ),
        "familiar joins the fight the caster's pet is in"
    );
}

#[test]
fn familiar_defends_an_attacked_caster_whose_own_combatant_expired() {
    let mut w = world();
    let p = master(&mut w, 1500, 1600);
    let f = familiar(&mut w, 0, p, 1501, 1600);
    let e = enemy(&mut w, 1496, 1600);
    run_for(&mut w, 400);

    mx::set_combatant(&mut w, e, Some(p));
    mx::set_combatant(&mut w, p, None); // expired
    assert_eq!(combatant(&w, p), None);

    let fight = range_of(&w, f, |d| d.range_fight);
    assert!(
        run_until(
            &mut w,
            |w| combatant(w, f) == Some(e) && in_range(w, f, e, fight),
            4000
        ),
        "familiar defends the caster from a live aggressor"
    );
}

#[test]
fn familiar_drops_a_target_that_no_longer_fights_anyone() {
    let mut w = world();
    let p = master(&mut w, 1500, 1600);
    let f = familiar(&mut w, 0, p, 1501, 1600);
    let e = enemy(&mut w, 1495, 1600);
    run_for(&mut w, 400);
    set_warmode(&mut w, p, true);
    mx::set_combatant(&mut w, p, Some(e));
    assert!(run_until(&mut w, |w| combatant(w, f) == Some(e), 2000));

    // Caster stops, target disengages.
    mx::set_combatant(&mut w, p, None);
    set_warmode(&mut w, p, false);
    mx::set_combatant(&mut w, e, None);

    assert!(
        run_until(
            &mut w,
            |w| combatant(w, f).is_none() && in_range(w, f, p, 1),
            4000
        ),
        "familiar disengages and returns"
    );
}

#[test]
fn familiar_assist_end_leaves_no_stale_move_intent() {
    let mut w = world();
    let p = master(&mut w, 1500, 1600);
    let f = familiar(&mut w, 0, p, 1501, 1600);
    let e = enemy(&mut w, 1495, 1600);
    run_for(&mut w, 400);
    set_warmode(&mut w, p, true);
    mx::set_combatant(&mut w, p, Some(e));
    assert!(run_until(&mut w, |w| combatant(w, f) == Some(e), 2000));

    // Assist ends with the familiar already beside the caster (MoveTo's arrival return).
    place(&mut w, f, 1501, 1600, TRAMMEL);
    mx::set_combatant(&mut w, p, None);
    set_warmode(&mut w, p, false);
    mx::set_combatant(&mut w, e, None);
    assert!(run_until(&mut w, |w| combatant(w, f).is_none(), 500));

    assert!(
        movement::try_get_move_wake(&w, f).is_none(),
        "no pursuit may survive the stand-down"
    );
}

// ---- SummonAcquireRulesTests ----

/// `PetTestStub`: an AnimalAI dog (perception 10, fight 1).
static PET_STUB_C: muo_content::creature::CreatureType =
    muo_content::creature::CreatureType { ..BASE_CREATURE_C };
static PET_STUB: MobileType = MobileType {
    full_name: "Server.Tests.PetTestStub",
    parent: Some(&BASE_CREATURE),
    family: Some(&PET_STUB_C),
    ..BASE_CREATURE
};

fn player(w: &mut World, map: MapId, x: i32, intel: i32) -> MobileId {
    let p = muo_content::player_mobile::new_player_mobile(w);
    w.mobile_mut(p).unwrap().player = true;
    muo_server::mobile::part1::props_b::set_raw_int(w, p, intel);
    place(w, p, x, 1600, map);
    p
}

fn summon_of(w: &mut World, summon: MobileId, caster: MobileId, x: i32) -> MobileId {
    set_summoned(w, summon, true);
    set_master(w, summon, Some(caster));
    let map = w.mobile(caster).unwrap().map.unwrap();
    place(w, summon, x, 1600, map);
    muo_content::ai::timer::pause(w, summon);
    summon
}

fn acquire(w: &mut World, creature: MobileId) -> bool {
    let tick = w.tick_count;
    cd_mut(w, creature).unwrap().next_reacquire_time = tick - 1;
    let (rp, mode) = {
        let d = cd(w, creature).unwrap();
        (d.range_perception, d.fight_mode)
    };
    muo_content::ai::base::acquire_focus_mob(w, creature, rp, mode, false, false, true)
}

fn focus(w: &World, creature: MobileId) -> Option<MobileId> {
    cd(w, creature).unwrap().focus_mob
}

#[test]
fn pre_aos_felucca_vortex_prefers_anyone_over_its_caster() {
    let mut w = world();
    w.core.expansion = Some(muo_data::expansion::Expansion::UOR);

    // The caster outranks the bystander on the vortex's int-over-distance ranking.
    let caster = player(&mut w, MapId::FELUCCA, 1500, 100);
    let bystander = player(&mut w, MapId::FELUCCA, 1496, 10);
    let v = (ENERGY_VORTEX.construct.unwrap())(&mut w);
    let vortex = summon_of(&mut w, v, caster, 1499);

    assert!(acquire(&mut w, vortex));
    assert_eq!(focus(&w, vortex), Some(bystander));
}

#[test]
fn vortex_spares_its_caster_outside_pre_aos_felucca() {
    for (expansion, map_index) in [
        (muo_data::expansion::Expansion::UOR, 1),
        (muo_data::expansion::Expansion::AOS, 0),
    ] {
        let mut w = world();
        w.core.expansion = Some(expansion);

        let map = if map_index == 1 {
            MapId::TRAMMEL
        } else {
            MapId::FELUCCA
        };
        let caster = player(&mut w, map, 1500, 100);
        let v = (ENERGY_VORTEX.construct.unwrap())(&mut w);
        let vortex = summon_of(&mut w, v, caster, 1499);

        assert!(!acquire(&mut w, vortex), "{expansion:?}");
        assert_eq!(focus(&w, vortex), None);
    }
}

#[test]
fn masterless_summon_does_not_break_acquisition() {
    let mut w = world();
    w.core.expansion = Some(muo_data::expansion::Expansion::AOS);

    player(&mut w, MapId::FELUCCA, 1500, 100);
    let hunter = new_creature(
        &mut w,
        &PET_STUB,
        AIType::AiAnimal,
        FightMode::Closest,
        10,
        1,
    );
    w.mobile_mut(hunter).unwrap().body = Body(0xC8);
    place(&mut w, hunter, 1499, 1600, MapId::FELUCCA);
    muo_content::ai::timer::pause(&mut w, hunter);

    let masterless = new_creature(
        &mut w,
        &PET_STUB,
        AIType::AiAnimal,
        FightMode::Closest,
        10,
        1,
    );
    set_summoned(&mut w, masterless, true);
    place(&mut w, masterless, 1498, 1600, MapId::FELUCCA);
    muo_content::ai::timer::pause(&mut w, masterless);

    assert!(acquire(&mut w, hunter));
}

// ---- MountRegionTests ----

use muo_content::regions::base_region::{BASE_REGION, BASE_REGION_FAMILY, BaseRegionType};
use muo_server::region::RegionType;

// `MountTestRegion(allowed)`: a BaseRegion whose `MountsAllowed` is the argument.
static MOUNTS_ALLOWED: BaseRegionType = BaseRegionType {
    mounts_allowed: true,
    ..BASE_REGION_FAMILY
};
static MOUNTS_DENIED: BaseRegionType = BaseRegionType {
    mounts_allowed: false,
    ..BASE_REGION_FAMILY
};
static MOUNT_REGION_ALLOWED: RegionType = RegionType {
    family: Some(&MOUNTS_ALLOWED),
    ..BASE_REGION
};
static MOUNT_REGION_DENIED: RegionType = RegionType {
    family: Some(&MOUNTS_DENIED),
    ..BASE_REGION
};

#[test]
fn mounting_respects_active_region() {
    use muo_data::expansion::Expansion;
    let eras = [
        Expansion::None,
        Expansion::T2A,
        Expansion::UOR,
        Expansion::UOTD,
        Expansion::LBR,
        Expansion::AOS,
        Expansion::SE,
        Expansion::ML,
        Expansion::SA,
        Expansion::HS,
        Expansion::TOL,
        Expansion::EJ,
    ];
    for era in eras {
        for allowed in [false, true] {
            for ethereal in [false, true] {
                let mut w = world();
                w.core.expansion = Some(era);
                let ty = if allowed {
                    &MOUNT_REGION_ALLOWED
                } else {
                    &MOUNT_REGION_DENIED
                };
                let region = muo_server::region::new_region(
                    &mut w,
                    ty,
                    Some("MountRegionTest"),
                    MapId::FELUCCA,
                    None,
                    Some(100),
                    vec![muo_primitives::Rectangle3D::new(
                        990, 990, -128, 20, 20, 256,
                    )],
                );
                muo_server::region::register(&mut w, region);

                let player = muo_content::player_mobile::new_player_mobile(&mut w);
                let pack = (muo_content::items::cont::BACKPACK.construct.unwrap())(&mut w);
                muo_server::mobile::part2::items::add_item(&mut w, player, pack);
                place(&mut w, player, 1000, 1000, MapId::FELUCCA);
                assert_eq!(w.mobile(player).unwrap().region, region, "{era:?}");

                let case = format!("{era:?} allowed {allowed} ethereal {ethereal}");
                if ethereal {
                    let statue =
                        (crate::classes::ethereal::ETHEREAL_HORSE.construct.unwrap())(&mut w);
                    let drop = w.item(pack).unwrap().ty.drop_item;
                    drop(&mut w, pack, statue);
                    assert_eq!(
                        w.item(statue).unwrap().parent,
                        Some(muo_server::ids::EntityId::Item(pack)),
                        "{case}"
                    );
                    assert_eq!(
                        crate::classes::ethereal::validate(&mut w, statue, player),
                        allowed,
                        "{case}"
                    );
                } else {
                    let horse = (crate::animals::leaves::HORSE.construct.unwrap())(&mut w);
                    let at = loc(&w, player);
                    muo_server::mobile::base::move_to_world(&mut w, horse, at, MapId::FELUCCA);
                    set_control_master(&mut w, horse, Some(player));
                    let ty = w.mobile(horse).unwrap().ty;
                    (ty.on_double_click)(&mut w, horse, player);
                    assert_eq!(
                        muo_content::creature::mount::rider(&w, horse) == Some(player),
                        allowed,
                        "{case}"
                    );
                    assert_eq!(
                        muo_server::mobile::part1::props_b::mounted(&mut w, player),
                        allowed,
                        "{case}"
                    );
                }
            }
        }
    }
}
