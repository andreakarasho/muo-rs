//! Tests of the dueling system: the save layouts of the items (round trips, and the exact bytes of the generated and
//! hand-written `Serialize`), the ladder, the arena's start points, and the rules of finishing a duel.

use muo_io::{GenericReader, GenericWriter, SaveReader, SaveWriter};
use muo_primitives::{Point2D, Point3D, Rectangle2D};
use muo_server::World;
use muo_server::item::{ITEM_BASE, ItemType};
use muo_server::map::MapId;
use muo_server::mobile::MOBILE_BASE;

use super::arena::{self, ARENA_CONTROLLER, ArenaData, EXIT_TELEPORTER};
use super::ladder::{self, LADDER_CONTROLLER, PREFERENCES_CONTROLLER};
use super::ladder_gump::LADDER_ITEM;
use super::trophy::{self, TROPHY, TrophyRank};
use super::*;

fn bytes_of(w: &World, ty: &ItemType, id: muo_server::ids::ItemId) -> Vec<u8> {
    let mut out = SaveWriter::new(true);
    (ty.serialize)(w, id, &mut out);
    out.into_vec()
}

/// The part a class writes after `Item`'s own.
fn tail_of(w: &World, ty: &ItemType, id: muo_server::ids::ItemId) -> Vec<u8> {
    let mut base = SaveWriter::new(true);
    (ITEM_BASE.serialize)(w, id, &mut base);
    bytes_of(w, ty, id)[base.as_slice().len()..].to_vec()
}

/// Loads `bytes` (a save of an item of class `ty`) into a new item and writes it again.
fn reload(w: &mut World, ty: &'static ItemType, bytes: &[u8]) -> Vec<u8> {
    let id = w.new_item(ty);
    let mut r = SaveReader::new(bytes);
    (ty.deserialize)(w, id, &mut r).expect("deserializes");
    assert_eq!(r.position() as usize, bytes.len(), "reads every byte");
    bytes_of(w, ty, id)
}

#[test]
fn pvp_trophy_save_layout_and_round_trip() {
    let mut w = World::new();
    let owner = w.new_mobile(&MOBILE_BASE);
    let id = trophy::new_trophy(&mut w, Some("Duel Champion".to_owned()), TrophyRank::Gold);
    w.item_mut(id)
        .unwrap()
        .ext
        .get_mut::<trophy::TrophyData>()
        .unwrap()
        .owner = Some(owner);

    assert_eq!(
        ((TROPHY.default_name)(&w, id)).as_deref(),
        Some("gold trophy")
    );
    assert_eq!(w.item(id).unwrap().hue, 2213, "UpdateStyle: gold");

    let tail = tail_of(&w, &TROPHY, id);
    let mut r = SaveReader::new(&tail);
    assert_eq!(r.read_i32().unwrap(), 2);
    assert_eq!(r.read_string().unwrap().as_deref(), Some("Duel Champion"));
    assert_eq!(r.read_i32().unwrap(), 2); // TrophyRank.Gold
    assert_eq!(r.read_serial().unwrap(), w.mobile(owner).unwrap().serial);
    r.read_date_time_ticks().unwrap();
    assert_eq!(r.position() as usize, tail.len());

    let bytes = bytes_of(&w, &TROPHY, id);
    assert_eq!(reload(&mut w, &TROPHY, &bytes), bytes);
}

#[test]
fn pvp_ladder_controller_layout_and_find_quirk() {
    let mut w = World::new();
    let (a, b) = (w.new_mobile(&MOBILE_BASE), w.new_mobile(&MOBILE_BASE));
    let c = (LADDER_CONTROLLER.construct.unwrap())(&mut w);
    assert_eq!(ladder::instance(&w), Some(c), "Ladder.Instance ??= Ladder");

    // Entries come from a save: the first is higher in the table.
    let mut save = SaveWriter::new(true);
    (ITEM_BASE.serialize)(&w, c, &mut save);
    save.write_i32(1);
    save.write_encoded_int(1); // Ladder version
    save.write_encoded_int(2);
    for (m, xp, wins, losses) in [(a, 500, 4, 1), (b, 300, 2, 3)] {
        save.write_serial(w.mobile(m).unwrap().serial);
        save.write_encoded_int(xp);
        save.write_encoded_int(wins);
        save.write_encoded_int(losses);
    }
    save.write_bool(true);
    let bytes = save.into_vec();
    assert_eq!(reload(&mut w, &LADDER_CONTROLLER, &bytes), bytes);

    // `Find` of a mobile with an entry makes a fresh one (xp 0) and appends it; with none, it returns null.
    let c2 = {
        let id = w.new_item(&LADDER_CONTROLLER);
        let mut r = SaveReader::new(&bytes);
        (LADDER_CONTROLLER.deserialize)(&mut w, id, &mut r).unwrap();
        id
    };
    let stranger = w.new_mobile(&MOBILE_BASE);
    assert!(ladder::find(&mut w, c2, stranger).is_none());
    let e = ladder::find(&mut w, c2, a).expect("a has an entry");
    assert_eq!((e.experience, e.wins, e.losses), (0, 0, 0));
    assert_eq!(e.index, 2, "appended");
    assert_eq!(
        w.item(c2)
            .unwrap()
            .ext
            .get::<ladder::Ladder>()
            .unwrap()
            .entries
            .len(),
        3
    );
}

#[test]
fn pvp_preferences_controller_round_trip() {
    let mut w = World::new();
    let m = w.new_mobile(&MOBILE_BASE);
    let c = (PREFERENCES_CONTROLLER.construct.unwrap())(&mut w);
    assert_eq!(ladder::preferences_instance(&w), Some(c));
    w.item_mut(c)
        .unwrap()
        .ext
        .get_mut::<ladder::Preferences>()
        .unwrap()
        .find(m)
        .disliked = vec![Some("Arena One".to_owned()), None];
    let tail = tail_of(&w, &PREFERENCES_CONTROLLER, c);
    let mut r = SaveReader::new(&tail);
    assert_eq!(r.read_i32().unwrap(), 0);
    assert_eq!(r.read_encoded_int().unwrap(), 0);
    assert_eq!(r.read_encoded_int().unwrap(), 1);
    assert_eq!(r.read_serial().unwrap(), w.mobile(m).unwrap().serial);
    assert_eq!(r.read_encoded_int().unwrap(), 2);
    assert_eq!(r.read_string().unwrap().as_deref(), Some("Arena One"));
    assert_eq!(r.read_string().unwrap(), None);

    let bytes = bytes_of(&w, &PREFERENCES_CONTROLLER, c);
    assert_eq!(reload(&mut w, &PREFERENCES_CONTROLLER, &bytes), bytes);
}

#[test]
fn pvp_arena_controller_layout_and_round_trip() {
    let mut w = World::new();
    let c = (ARENA_CONTROLLER.construct.unwrap())(&mut w);
    let p = w.new_mobile(&MOBILE_BASE);
    {
        let d = arena::data_mut(&mut w, c).unwrap();
        d.is_private = true;
        d.is_guarded = true;
        d.name = Some("Arena One".to_owned());
        d.facet = Some(MapId::FELUCCA);
        d.bounds = Rectangle2D::from_points(Point2D::new(10, 10), Point2D::new(40, 40));
        d.outside = Point3D::new(5, 5, 0);
        d.wall = Point3D::new(25, 25, 0);
        d.points[0] = Point3D::new(12, 25, 0);
        d.players = vec![p];
    }

    let tail = tail_of(&w, &ARENA_CONTROLLER, c);
    let mut r = SaveReader::new(&tail);
    assert_eq!(r.read_i32().unwrap(), 1);
    assert!(r.read_bool().unwrap(), "IsPrivate");
    assert_eq!(r.read_encoded_int().unwrap(), 7, "Arena version");
    assert!(r.read_bool().unwrap(), "IsGuarded");

    let bytes = bytes_of(&w, &ARENA_CONTROLLER, c);
    // The reload makes a SafeZone only when the zone is set; the arena is not active, so it is not listed.
    let re = reload(&mut w, &ARENA_CONTROLLER, &bytes);
    assert_eq!(re, bytes);
    assert!(
        arena::arenas(&w).is_empty(),
        "inactive arenas are not in Arenas"
    );
    let _ = ArenaData::default();
}

#[test]
fn pvp_exit_teleporter_and_wall_constructors() {
    let mut w = World::new();
    let tp = (EXIT_TELEPORTER.construct.or(EXIT_TELEPORTER.make).unwrap())(&mut w);
    assert_eq!(w.item(tp).unwrap().item_id, 0x1822);
    assert_eq!(w.item(tp).unwrap().hue, 0x482);
    let m = w.new_mobile(&MOBILE_BASE);
    arena::teleporter_register(&mut w, tp, m);
    let bytes = bytes_of(&w, &EXIT_TELEPORTER, tp);
    assert_eq!(reload(&mut w, &EXIT_TELEPORTER, &bytes), bytes);
    let tail = tail_of(&w, &EXIT_TELEPORTER, tp);
    let mut r = SaveReader::new(&tail);
    assert_eq!(r.read_i32().unwrap(), 0);
    assert_eq!(r.read_encoded_int().unwrap(), 1, "one entry");
}

#[test]
fn pvp_exit_teleporter_register_drops_expired_entries_before_the_match() {
    let mut w = World::new();
    let tp = (EXIT_TELEPORTER.construct.or(EXIT_TELEPORTER.make).unwrap())(&mut w);
    let (a, b) = (w.new_mobile(&MOBILE_BASE), w.new_mobile(&MOBILE_BASE));
    arena::teleporter_register(&mut w, tp, a);
    arena::teleporter_register(&mut w, tp, b);
    w.now += muo_primitives::TimeSpan::from_millis(31 * 60_000);
    // `Find(b)` passes `a` (expired) and removes it, then updates `b`'s entry in place.
    arena::teleporter_register(&mut w, tp, b);
    let tail = tail_of(&w, &EXIT_TELEPORTER, tp);
    let mut r = SaveReader::new(&tail);
    r.read_i32().unwrap();
    assert_eq!(r.read_encoded_int().unwrap(), 1);
}

#[test]
fn pvp_ladder_item_round_trip() {
    let mut w = World::new();
    let id = (LADDER_ITEM.construct.unwrap())(&mut w);
    assert_eq!(w.item(id).unwrap().item_id, 0x117F);
    let bytes = bytes_of(&w, &LADDER_ITEM, id);
    assert_eq!(reload(&mut w, &LADDER_ITEM, &bytes), bytes);
}

fn two_sides(w: &mut World) -> (u32, MobileId, MobileId) {
    let (a, b) = (w.new_mobile(&MOBILE_BASE), w.new_mobile(&MOBILE_BASE));
    let id = flow::new_context(w, a);
    set_slot(w, id, 1, 0, b);
    (id, a, b)
}

#[test]
fn pvp_new_context_has_two_participants_with_the_initiator_first() {
    let mut w = World::new();
    let (id, a, b) = two_sides(&mut w);
    let c = ctx(&w, id).unwrap();
    assert_eq!(c.participants.len(), 2);
    assert_eq!(c.participant_of(a), Some(0));
    assert_eq!(c.participant_of(b), Some(1));
    assert!(c.is_one_vs_one());
    assert_eq!(duel_context_of(&w, b), Some(id));
    assert!(flow::check_full(&w, id));
    assert_ne!(c.participants[0].uid, c.participants[1].uid);
}

#[test]
fn pvp_check_completion_needs_all_but_one_eliminated() {
    let mut w = World::new();
    let (id, a, b) = two_sides(&mut w);
    assert_eq!(flow::check_completion(&w, id), None);
    set_eliminated(&mut w, id, a);
    assert_eq!(flow::check_completion(&w, id), Some(1), "b wins");
    set_eliminated(&mut w, id, b);
    // Both eliminated: the second is counted last, the winner is the one not eliminated, or the first.
    assert_eq!(flow::check_completion(&w, id), Some(0));
}

#[test]
fn pvp_finish_marks_the_duel_and_a_second_finish_does_nothing() {
    let mut w = World::new();
    let (id, a, _b) = two_sides(&mut w);
    set_eliminated(&mut w, id, a);
    flow::finish(&mut w, id, 1);
    assert!(ctx(&w, id).unwrap().finished);
    flow::finish(&mut w, id, 0);
    assert!(ctx(&w, id).unwrap().finished);
}

#[test]
fn pvp_unregister_releases_the_players() {
    let mut w = World::new();
    let (id, a, b) = two_sides(&mut w);
    flow::unregister(&mut w, id, false);
    assert!(!ctx(&w, id).unwrap().registered);
    assert_eq!(duel_context_of(&w, a), None);
    assert_eq!(duel_context_of(&w, b), None);
}

#[test]
fn pvp_countdown_ticks_from_count_down_to_one() {
    // `StartCountdown(n, cb)` is `StartTimer(1s, n, ..)`: the callbacks see `RemainingCount` n..1, never 0, so
    // the `count == 0` branch of `SendReadyGump` (where the duel would start) is not reached.
    let mut w = World::new();
    let (id, _a, _b) = two_sides(&mut w);
    flow::start_countdown(&mut w, id, 3, CountdownKind::Ready);
    let mut seen = Vec::new();
    for _ in 0..6 {
        w.tick_count += 1000;
        muo_server::timer::slice(&mut w);
        let n = ctx(&w, id).unwrap().ready_count;
        if n != 0 && seen.last() != Some(&n) {
            seen.push(n);
        }
    }
    assert_eq!(seen, vec![3, 2, 1]);
    assert!(!ctx(&w, id).unwrap().started_begin_countdown);
}

#[test]
fn pvp_special_move_titles_are_swapped() {
    let mut w = World::new();
    let (id, a, _b) = two_sides(&mut w);
    {
        let c = ctx_mut(&mut w, id).unwrap();
        c.started_begin_countdown = true;
        c.ruleset
            .set_option("Bushido", "Honorable Execution", false);
    }
    // InstAllowSpecialMove swaps the titles: a NinjaMove is looked up under "Bushido", a SamuraiMove under "Ninjitsu".
    // FindByOption sets the title to null only for the failing node's own children, so "Honorable Execution" (a Bushido
    // option) asked under "Ninjitsu" is not found: allowed, and "Backstab" asked under "Bushido" is not found either.
    assert!(allow_special_move(
        &mut w,
        a,
        "Honorable Execution",
        Some(false)
    ));
    ctx_mut(&mut w, id)
        .unwrap()
        .ruleset
        .set_option("Ninjitsu", "Backstab", false);
    assert!(allow_special_move(&mut w, a, "Backstab", Some(true)));
    assert!(!allow_special_move(&mut w, a, "Backstab", Some(false)));
    assert!(allow_special_move(&mut w, a, "Backstab", None));
}

#[test]
fn pvp_ruleset_layout_matches_expansion_flags() {
    let aos = RulesetLayout::build(true, true, true);
    let pre = RulesetLayout::build(false, false, false);
    assert!(aos.find_by_title("Ninjitsu").is_some() && pre.find_by_title("Ninjitsu").is_none());
    assert!(aos.total_length() > pre.total_length());
    let ids = pre.find_by_option(Some("Weapons"), "Runics");
    assert!(ids.is_some(), "pre-AOS weapons have Runics");
    assert!(aos.find_by_option(Some("Weapons"), "Runics").is_none());
}

// ---- tournaments ---------------------------------------------------------------------------------------------

use super::tournament::{
    self, Stage, TOURNAMENT_BRACKET_ITEM, TOURNAMENT_CONTROLLER, TOURNAMENT_SIGNUP_ITEM,
    TourneyType,
};

fn controller(w: &mut World) -> muo_server::ids::ItemId {
    (TOURNAMENT_CONTROLLER.construct.unwrap())(w)
}

#[test]
fn pvp_tournament_controller_save_layout_and_load_quirk() {
    let mut w = World::new();
    let id = controller(&mut w);
    assert_eq!(w.item(id).unwrap().item_id, 0x1B7A);
    {
        let t = tournament::get_mut(&mut w, id).unwrap();
        t.tourney_type = TourneyType::RandomTeam;
        t.sudden_death = muo_primitives::TimeSpan::from_mins(3);
        t.sudden_death_rounds = 2;
        t.set_participants_per_match(4);
        t.faction_restricted = true;
    }
    let tail = tail_of(&w, &TOURNAMENT_CONTROLLER, id);
    let mut r = SaveReader::new(&tail);
    assert_eq!(r.read_i32().unwrap(), 0);
    assert_eq!(r.read_encoded_int().unwrap(), 5);
    assert!(r.read_bool().unwrap());
    r.read_serial().unwrap();
    assert_eq!(r.read_encoded_int().unwrap(), 2, "sudden death rounds");
    assert_eq!(r.read_encoded_int().unwrap(), 2, "RandomTeam");
    assert_eq!(r.read_encoded_int().unwrap(), 0, "HighVsLow");
    assert_eq!(r.read_encoded_int().unwrap(), 0, "Random tie");
    assert_eq!(
        r.read_time_span_ticks().unwrap(),
        muo_primitives::TimeSpan::from_mins(3).ticks()
    );
    assert_eq!(r.read_encoded_int().unwrap(), 4);
    assert_eq!(r.read_encoded_int().unwrap(), 1);
    assert_eq!(
        r.read_time_span_ticks().unwrap(),
        muo_primitives::TimeSpan::from_mins(10).ticks()
    );
    assert_eq!(r.position() as usize, tail.len());

    // The reader puts the sudden death bytes into the signup period (and then reads the period): SuddenDeath is lost.
    let bytes = bytes_of(&w, &TOURNAMENT_CONTROLLER, id);
    let again = w.new_item(&TOURNAMENT_CONTROLLER);
    let mut r = SaveReader::new(&bytes);
    (TOURNAMENT_CONTROLLER.deserialize)(&mut w, again, &mut r).unwrap();
    assert_eq!(r.position() as usize, bytes.len());
    let t = tournament::get(&w, again).unwrap();
    assert_eq!(t.sudden_death, muo_primitives::TimeSpan::ZERO);
    assert_eq!(t.signup_period, muo_primitives::TimeSpan::from_mins(10));
    assert_eq!(t.stage, Stage::Inactive);
    assert!(t.faction_restricted && t.tourney_type == TourneyType::RandomTeam);
}

#[test]
fn pvp_tournament_signup_and_bracket_items_round_trip() {
    let mut w = World::new();
    let ctrl = controller(&mut w);
    let book = (TOURNAMENT_SIGNUP_ITEM.construct.unwrap())(&mut w);
    assert_eq!(w.item(book).unwrap().item_id, 4029);
    w.item_mut(book)
        .unwrap()
        .ext
        .get_mut::<tournament::LinkData>()
        .unwrap()
        .tournament = Some(ctrl);
    let tail = tail_of(&w, &TOURNAMENT_SIGNUP_ITEM, book);
    assert_eq!(tail.len(), 4 + 4 + 4, "version, tournament, registrar");
    let bytes = bytes_of(&w, &TOURNAMENT_SIGNUP_ITEM, book);
    assert_eq!(reload(&mut w, &TOURNAMENT_SIGNUP_ITEM, &bytes), bytes);

    let bracket = (TOURNAMENT_BRACKET_ITEM.construct.unwrap())(&mut w);
    assert_eq!(w.item(bracket).unwrap().item_id, 3774);
    let bytes = bytes_of(&w, &TOURNAMENT_BRACKET_ITEM, bracket);
    assert_eq!(reload(&mut w, &TOURNAMENT_BRACKET_ITEM, &bytes), bytes);
}

#[test]
fn pvp_pyramid_standard_pairs_and_free_advance() {
    let mut w = World::new();
    let ctrl = controller(&mut w);
    let mobs: Vec<MobileId> = (0..5).map(|_| w.new_mobile(&MOBILE_BASE)).collect();
    {
        let t = tournament::get_mut(&mut w, ctrl).unwrap();
        t.group_type = tournament::GroupingType::Random;
        for m in &mobs {
            t.parts.push(tournament::TourneyPart::new(vec![*m]));
            let i = t.parts.len() - 1;
            t.participants.push(i);
        }
    }
    let parts = tournament::get(&w, ctrl).unwrap().participants.clone();
    tournament::add_level(&mut w, ctrl, 2, &parts);
    let t = tournament::get(&w, ctrl).unwrap();
    let level = &t.levels[0];
    assert!(level.free_advance.is_some(), "an odd count: one advances");
    assert_eq!(level.matches.len(), 2);
    assert!(level.matches.iter().all(|m| m.participants.len() == 2));
    let fa = level.free_advance.unwrap();
    assert_eq!(t.parts[fa].free_advances, 1);
    assert_eq!(
        t.parts[fa].log,
        vec!["Advanced automatically due to an odd number of challengers."]
    );
    let m0 = &level.matches[0];
    assert!(t.parts[m0.participants[0]].log[0].starts_with("Matched in a duel against "));
}

#[test]
fn pvp_tournament_win_removes_the_beaten_from_undefeated_and_alerts_are_active() {
    let mut w = World::new();
    let ctrl = controller(&mut w);
    assert!(!tournament::is_active(&w));
    let (a, b) = (w.new_mobile(&MOBILE_BASE), w.new_mobile(&MOBILE_BASE));
    {
        let t = tournament::get_mut(&mut w, ctrl).unwrap();
        t.parts.push(tournament::TourneyPart::new(vec![a]));
        t.parts.push(tournament::TourneyPart::new(vec![b]));
        t.undefeated = vec![0, 1];
        t.levels.push(tournament::Level {
            matches: vec![tournament::TourneyMatch {
                participants: vec![0, 1],
                winner: None,
                context: None,
            }],
            free_advance: None,
        });
        t.stage = Stage::Fighting;
    }
    assert!(tournament::is_active(&w));
    tournament::handle_won(&mut w, ctrl, None, (0, 0), 0);
    assert_eq!(tournament::get(&w, ctrl).unwrap().undefeated, vec![0]);
}

/// `vli`: the LadderEntry's props: Experience's setter re-sorts, Rank is read-only.
#[test]
fn pvp_ladder_entry_props_for_vli() {
    use muo_content::props::Value;
    use muo_server::ids::EntityId;
    let mut w = World::new();
    let (a, b) = (w.new_mobile(&MOBILE_BASE), w.new_mobile(&MOBILE_BASE));
    let c = (LADDER_CONTROLLER.construct.unwrap())(&mut w);
    let mut save = SaveWriter::new(true);
    (ITEM_BASE.serialize)(&w, c, &mut save);
    save.write_i32(1);
    save.write_encoded_int(1);
    save.write_encoded_int(2);
    for (m, xp, wins, losses) in [(a, 500, 4, 1), (b, 300, 2, 3)] {
        save.write_serial(w.mobile(m).unwrap().serial);
        save.write_encoded_int(xp);
        save.write_encoded_int(wins);
        save.write_encoded_int(losses);
    }
    save.write_bool(true);
    let bytes = save.into_vec();
    (LADDER_CONTROLLER.deserialize)(&mut w, c, &mut SaveReader::new(&bytes)).unwrap();
    let p = |n: &str| {
        super::props::LADDER_ENTRY_PROPS
            .iter()
            .find(|d| d.name == n)
            .unwrap()
    };
    let eb = EntityId::Mobile(b);
    assert_eq!((p("Rank").get)(&mut w, eb).i32(), 1);
    assert!(p("Rank").set.is_none());
    (p("Experience").set.unwrap())(&mut w, eb, Value::Int(900)).unwrap();
    assert_eq!((p("Rank").get)(&mut w, eb).i32(), 0, "the entry moved up");
    assert_eq!((p("Wins").get)(&mut w, eb).i32(), 2);
    assert_eq!((p("Losses").get)(&mut w, EntityId::Mobile(a)).i32(), 1);
}
