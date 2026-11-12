//! Whole flows of the dueling system in a test world: a duel from the ready gump through a death (with `Requip`) to
//! `Finish` and `Unregister`, the event games playing to their awards, and a small tournament from `slice` and
//! `start_match` to the awards.
//!
//! The countdowns never deliver a 0 (see `flow`), so a test sends the begin gump's 0 itself.

use muo_primitives::{Point3D, TimeSpan};
use muo_server::World;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::enumerable::find_items_by_type;
use muo_server::item::ops;
use muo_server::map::MapId;
use muo_server::mobile::MOBILE_BASE;
use muo_server::target::Targeted;

use super::arena::{self, ARENA_CONTROLLER};
use super::tournament::{self, Stage, TOURNAMENT_CONTROLLER, TourneyPart, TourneyType};
use super::trophy::TROPHY;
use super::*;

fn world() -> World {
    let mut w = World::new();
    muo_content::register(&mut w.types);
    muo_items::register(&mut w.types);
    register(&mut w.types);
    configure(&mut w);
    w
}

/// A player standing on Felucca with a backpack.
fn player(w: &mut World, x: i32) -> MobileId {
    let m = w.new_mobile(&MOBILE_BASE);
    w.mobile_mut(m).unwrap().player = true;
    muo_server::mobile::base::move_to_world(w, m, Point3D::new(x, 100, 0), MapId::FELUCCA);
    let pack = muo_content::factory::make_item(w, "Server.Items.Backpack").expect("backpack");
    muo_server::mobile::part2::items::add_item(w, m, pack);
    assert!(
        muo_server::mobile::part1::props_b::backpack(w, m).is_some(),
        "the backpack is the player's"
    );
    m
}

fn pack(w: &World, m: MobileId) -> ItemId {
    w.mobile(m).unwrap().backpack.unwrap()
}

fn in_pack(w: &World, m: MobileId, item: ItemId) -> bool {
    ops::is_child_of(w, item, Some(EntityId::Item(pack(w, m))))
}

/// An active arena with eight start points on Felucca.
fn make_arena(w: &mut World, tournament: Option<ItemId>) -> ItemId {
    let a = (ARENA_CONTROLLER.construct.unwrap())(w);
    {
        let d = arena::data_mut(w, a).unwrap();
        d.name = Some("Arena One".to_owned());
        d.facet = Some(MapId::FELUCCA);
        d.outside = Point3D::new(10, 10, 0);
        d.wall = Point3D::new(60, 60, 0);
        d.tournament = tournament;
        for (i, p) in d.points.iter_mut().enumerate() {
            *p = Point3D::new(50 + 5 * i as i32, 60, 0);
        }
    }
    arena::set_active(w, a, true);
    a
}

fn title_of(w: &World, trophy: ItemId) -> Option<String> {
    w.item(trophy)?
        .ext
        .get::<super::trophy::TrophyData>()?
        .title
        .clone()
}

fn cheque_in(w: &World, m: MobileId) -> Option<ItemId> {
    muo_server::item::enumerable::find_items(w, pack(w, m), true, |w, id| {
        muo_content::factory::item_is(w, id, "Server.Items.BankCheck")
    })
    .into_iter()
    .next()
}

fn advance(w: &mut World, ms: i64) {
    for _ in 0..(ms / 1000) {
        w.tick_count += 1000;
        w.now += TimeSpan::from_secs(1);
        muo_server::timer::slice(w);
    }
}

fn gold_in(w: &mut World, container: ItemId, amount: i32) -> ItemId {
    let g = muo_content::items::gold::new_gold(w, amount);
    let ty = w.item(container).unwrap().ty;
    (ty.drop_item)(w, container, g);
    g
}

#[test]
fn pvp_duel_runs_from_ready_to_finish_and_requips_the_loser() {
    let mut w = world();
    let (a, b) = (player(&mut w, 100), player(&mut w, 101));
    let arena = make_arena(&mut w, None);

    let id = flow::new_context(&mut w, a);
    set_slot(&mut w, id, 1, 0, b);
    assert!(flow::check_full(&w, id));
    // the ready countdown has run out: the duel looks for an arena and moves everyone in
    flow::send_ready_gump_count(&mut w, id, 0);
    let c = ctx(&w, id).unwrap();
    assert_eq!(c.arena, Some(arena), "the free arena was taken");
    assert!(c.started_begin_countdown && !c.started);
    assert!(arena::is_occupied(&w, arena));
    let (la, lb) = (w.mobile(a).unwrap().location, w.mobile(b).unwrap().location);
    assert!(
        la.x >= 40 && la.x <= 70 && lb.x >= 40 && lb.x <= 70,
        "both stand in the arena: {la:?} {lb:?}"
    );
    assert_ne!(la, lb);
    assert!(
        w.mobile(a).unwrap().frozen,
        "frozen until the begin countdown ends"
    );

    // the spectator gate stands where the challenge was made (a was at 100,100)
    let gate = w
        .items
        .iter()
        .find(|(_, i)| i.ty.is(&spectator::ARENA_MOONGATE))
        .map(|(id, _)| *id);
    let gate = gate.expect("the ArenaMoongate appeared");
    assert_eq!(w.item(gate).unwrap().location, Point3D::new(100, 100, 0));
    assert_eq!(w.item(gate).unwrap().item_id, 0x1FD4);
    assert_eq!(ops::name(&w, gate).as_deref(), Some("spectator moongate"));

    flow::send_begin_gump(&mut w, id, 0);
    assert!(ctx(&w, id).unwrap().started);
    assert!(!w.mobile(a).unwrap().frozen);

    // a dies carrying 500 gold; the corpse keeps what a cannot take back, Requip returns the rest to the pack
    let corpse = muo_content::items::corpse::new_corpse(&mut w, a, Vec::new());
    let gold = gold_in(&mut w, corpse, 500);
    assert!(!in_pack(&w, a, gold));
    flow::on_death(&mut w, id, a, Some(corpse));
    assert!(
        in_pack(&w, a, gold),
        "Requip: the loser took his things back"
    );
    assert!(
        muo_content::items::corpse::carved(&w, corpse),
        "and the corpse is marked"
    );
    let c = ctx(&w, id).unwrap();
    assert!(c.find(a).unwrap().eliminated);
    assert!(c.finished, "b is the last one standing");

    // nine seconds later the duel is unregistered and the arena free
    advance(&mut w, 10_000);
    assert!(!ctx(&w, id).unwrap().registered);
    // the loser is asked for a rematch: a new context (a fresh pair), not the old one
    assert_ne!(duel_context_of(&w, a), Some(id));
    assert!(!arena::is_occupied(&w, arena));
}

fn team_names(w: &mut World, ec: ItemId, kind: games::GameKind, origin_x: i32) {
    let c = games::ctrl_mut(w, ec).unwrap();
    for (i, t) in c.teams.iter_mut().enumerate().take(2) {
        t.name = Some(if i == 0 { "Red" } else { "Blue" }.to_owned());
        t.color = if i == 0 { 0x21 } else { 0x4F2 };
        t.origin = Point3D::new(origin_x + 10 * i as i32, 70, 0);
    }
    assert_eq!(c.kind, kind);
}

/// A started two-sided duel playing the game of `ec`.
fn game_duel(w: &mut World, ec: ItemId) -> (u32, MobileId, MobileId) {
    let (a, b) = (player(w, 100), player(w, 101));
    ops::move_to_world_map(w, ec, Point3D::new(60, 60, 0), Some(MapId::FELUCCA));
    let id = flow::new_context(w, a);
    set_slot(w, id, 1, 0, b);
    let g = games::construct(w, ec).unwrap();
    let c = ctx_mut(w, id).unwrap();
    c.event_game = Some(g);
    c.started = true;
    c.started_begin_countdown = true;
    (id, a, b)
}

#[test]
fn pvp_ctf_capture_scores_and_the_awards_follow() {
    let mut w = world();
    let ec = ctf::new_controller_ctf(&mut w);
    team_names(&mut w, ec, games::GameKind::Ctf, 40);
    let (fa, fb) = (ctf::new_flag(&mut w), ctf::new_flag(&mut w));
    for (i, f) in [fa, fb].into_iter().enumerate() {
        games::team_mut(&mut w, ec, i).unwrap().flag = Some(f);
    }
    let (id, a, b) = game_duel(&mut w, ec);
    games::on_start(&mut w, id);

    // both flags are home, in their teams' colours, and the players wear their team's colour
    assert_eq!(w.item(fb).unwrap().location, Point3D::new(50, 70, 0));
    assert_eq!(w.item(fa).unwrap().hue, 0x21);
    assert_eq!(w.mobile(a).unwrap().solid_hue_override, 0x21);
    assert_eq!(w.mobile(b).unwrap().solid_hue_override, 0x4F2);
    assert_eq!(games::team_of(&w, id, a), Some(0));
    assert_eq!(games::team_of(&w, id, b), Some(1));

    // b steals red's flag, then captures it on blue's own
    muo_server::mobile::part2::items::place_in_backpack(&mut w, b, fa);
    assert_eq!(
        w.mobile(b).unwrap().solid_hue_override,
        0x4001,
        "the carrier shows as carrying"
    );
    ctf::flag_on_target(&mut w, fa, b, Targeted::Entity(EntityId::Item(fb)));
    let blue = games::team(&w, ec, 1).unwrap();
    assert_eq!((blue.score, blue.captures), (50, 1));
    assert_eq!(blue.leader, Some(b));
    assert_eq!(
        w.item(fa).unwrap().location,
        Point3D::new(40, 70, 0),
        "red's flag is home again"
    );

    // the game ends: blue is first. b gets a gold trophy and 12,500gp, a (no score) nothing, a is eliminated
    games::finish_callback(&mut w, id, ec);
    let trophies = find_items_by_type(&w, pack(&w, b), true, &TROPHY, |_, _| true);
    assert_eq!(trophies.len(), 1);
    let title = title_of(&w, trophies[0]).unwrap();
    assert_eq!(title, " CTF Leader: 50 points, 1 capture");
    assert_eq!(
        ops::name(&w, trophies[0]).as_deref(),
        Some("gold trophy, blue team")
    );
    let cheque = cheque_in(&w, b);
    assert!(cheque.is_some(), "50 points are 12,500gp");
    assert!(ctx(&w, id).unwrap().find(a).unwrap().eliminated);
    assert!(ctx(&w, id).unwrap().finished);

    games::on_stop(&mut w, id);
    assert_eq!(w.mobile(a).unwrap().solid_hue_override, -1);
    assert_eq!(games::team(&w, ec, 0).unwrap().game, None);
}

#[test]
fn pvp_ctf_death_drops_the_flag_and_requips() {
    let mut w = world();
    let ec = ctf::new_controller_ctf(&mut w);
    team_names(&mut w, ec, games::GameKind::Ctf, 40);
    let (fa, fb) = (ctf::new_flag(&mut w), ctf::new_flag(&mut w));
    for (i, f) in [fa, fb].into_iter().enumerate() {
        games::team_mut(&mut w, ec, i).unwrap().flag = Some(f);
    }
    let (id, a, b) = game_duel(&mut w, ec);
    games::on_start(&mut w, id);
    muo_server::mobile::part2::items::place_in_backpack(&mut w, a, fb);
    let corpse = muo_content::items::corpse::new_corpse(&mut w, a, Vec::new());
    let gold = gold_in(&mut w, corpse, 5);

    flow::on_death(&mut w, id, a, Some(corpse));
    // the event game answers "false": the duel does not eliminate a, the flag lies where a died, a keeps playing
    let c = ctx(&w, id).unwrap();
    assert!(!c.find(a).unwrap().eliminated && !c.finished);
    assert!(!in_pack(&w, a, fb) && w.item(fb).unwrap().parent.is_none());
    assert_eq!(w.item(fb).unwrap().location.z, 2);
    assert!(in_pack(&w, a, gold), "Requip");
    let _ = b;
}

#[test]
fn pvp_doubledom_holding_both_points_scores() {
    let mut w = world();
    let ec = dd::new_controller_dd(&mut w);
    team_names(&mut w, ec, games::GameKind::Dd, 40);
    let (pa, pb) = (dd::new_way_point(&mut w), dd::new_way_point(&mut w));
    {
        let c = games::ctrl_mut(&mut w, ec).unwrap();
        c.point_a = Some(pa);
        c.point_b = Some(pb);
    }
    let (id, a, _b) = game_duel(&mut w, ec);
    games::on_start(&mut w, id);
    assert_eq!(w.item(pa).unwrap().hue, dd::NON_CAPTURED_HUE);

    for p in [pa, pb] {
        assert!((dd::DD_WAY_POINT.on_move_over)(&mut w, p, a));
    }
    assert_eq!(w.item(pb).unwrap().hue, 0x21, "red holds the point");
    advance(&mut w, 12_000);
    let red = games::team(&w, ec, 0).unwrap();
    assert_eq!(
        (red.score, red.captures),
        (100, 1),
        "ten seconds of holding both"
    );
    assert_eq!(
        w.item(pa).unwrap().hue,
        dd::UNCAPTURABLE_HUE,
        "the points rest for 30 seconds"
    );
    advance(&mut w, 31_000);
    assert_eq!(w.item(pa).unwrap().hue, dd::NON_CAPTURED_HUE);
    games::on_stop(&mut w, id);
}

#[test]
fn pvp_king_of_the_hill_scores_by_the_interval() {
    let mut w = world();
    let ec = koth::new_controller_kh(&mut w);
    team_names(&mut w, ec, games::GameKind::Kh, 40);
    let hill = (koth::HILL_OF_THE_KING.construct.unwrap())(&mut w);
    ops::move_to_world_map(&mut w, hill, Point3D::new(70, 70, 0), Some(MapId::FELUCCA));
    w.item_mut(hill)
        .unwrap()
        .ext
        .get_mut::<koth::HillData>()
        .unwrap()
        .score_interval = 3;
    games::ctrl_mut(&mut w, ec).unwrap().hills[0] = Some(hill);
    let (id, a, _b) = game_duel(&mut w, ec);
    games::on_start(&mut w, id);

    assert!((koth::HILL_OF_THE_KING.on_move_over)(&mut w, hill, a));
    assert_eq!(koth::king(&w, hill), Some(a));
    assert!(
        games::cant_do_anything(&w, id, a),
        "the king can do nothing"
    );
    advance(&mut w, 4_000);
    let red = games::team(&w, ec, 0).unwrap();
    assert_eq!((red.captures, red.score), (1, 3));
    assert_eq!(koth::captures_so_far(&w, hill), 1);
    assert!((koth::HILL_OF_THE_KING.on_move_off)(&mut w, hill, a));
    assert_eq!(koth::king(&w, hill), None);
    games::on_stop(&mut w, id);
}

#[test]
fn pvp_king_of_the_hill_first_tick_makes_the_kings_player_info() {
    let mut w = world();
    let ec = koth::new_controller_kh(&mut w);
    team_names(&mut w, ec, games::GameKind::Kh, 40);
    let hill = (koth::HILL_OF_THE_KING.construct.unwrap())(&mut w);
    ops::move_to_world_map(&mut w, hill, Point3D::new(70, 70, 0), Some(MapId::FELUCCA));
    w.item_mut(hill)
        .unwrap()
        .ext
        .get_mut::<koth::HillData>()
        .unwrap()
        .score_interval = 30;
    games::ctrl_mut(&mut w, ec).unwrap().hills[0] = Some(hill);
    let (id, a, _b) = game_duel(&mut w, ec);
    games::on_start(&mut w, id);
    assert!((koth::HILL_OF_THE_KING.on_move_over)(&mut w, hill, a));
    assert!(games::team(&w, ec, 0).unwrap().players.is_empty());
    advance(&mut w, 1_500);
    // `ti[King]` made the entry although nothing scored yet: the trophies at the end include the king
    assert!(games::team(&w, ec, 0).unwrap().players.contains_key(&a));
    games::on_stop(&mut w, id);
}

#[test]
fn pvp_bombing_run_starts_with_a_bomb_at_home_and_stops_without_one() {
    let mut w = world();
    let ec = bombing::new_controller_br(&mut w);
    team_names(&mut w, ec, games::GameKind::Br, 40);
    games::ctrl_mut(&mut w, ec).unwrap().br.bomb_home = Point3D::new(65, 65, 0);
    let (id, a, _b) = game_duel(&mut w, ec);
    games::on_start(&mut w, id);
    let bomb = games::rt(&w, id).unwrap().br.bomb.expect("a bomb");
    assert_eq!(w.item(bomb).unwrap().location, Point3D::new(65, 65, 0));
    assert!(
        !w.item(bomb).unwrap().visible(),
        "hidden for 5 to 15 seconds"
    );
    advance(&mut w, 16_000);
    assert!(w.item(bomb).unwrap().visible());
    assert!(!games::cant_do_anything(&w, id, a));
    games::on_stop(&mut w, id);
    assert!(ops::is_deleted(&w, bomb));
}

#[test]
fn pvp_tournament_plays_its_matches_and_awards_the_top_three() {
    let mut w = world();
    let ctrl = (TOURNAMENT_CONTROLLER.construct.unwrap())(&mut w);
    let a = make_arena(&mut w, Some(ctrl));
    let ps: Vec<MobileId> = (0..4).map(|i| player(&mut w, 100 + i)).collect();
    {
        let t = tournament::get_mut(&mut w, ctrl).unwrap();
        t.tourney_type = TourneyType::Standard;
        t.stage = Stage::Signup;
        t.signup_start = muo_primitives::DateTime::MIN;
        for (i, p) in ps.iter().enumerate() {
            t.parts.push(TourneyPart::new(vec![*p]));
            t.participants.push(i);
        }
    }
    // the signup period is over: the bracket is made
    w.now = muo_primitives::DateTime::MIN + TimeSpan::from_days(365);
    tournament::slice(&mut w, ctrl);
    let t = tournament::get(&w, ctrl).unwrap();
    assert_eq!(t.stage, Stage::Fighting);
    assert_eq!((t.levels.len(), t.levels[0].matches.len()), (1, 2));

    // every slice starts the next match in the tournament's arena; the loser of each match is the last of its sides
    let mut played = 0;
    for _ in 0..30 {
        if tournament::get(&w, ctrl).unwrap().stage == Stage::Inactive {
            break;
        }
        tournament::slice(&mut w, ctrl);
        let t = tournament::get(&w, ctrl).unwrap();
        let running: Vec<(u32, Vec<usize>)> = t
            .levels
            .iter()
            .flat_map(|l| l.matches.iter())
            .filter_map(|m| m.context.map(|c| (c, m.participants.clone())))
            .filter(|(c, _)| ctx(&w, *c).is_some_and(|c| c.registered && !c.finished))
            .collect();
        for (id, parts) in running {
            assert_eq!(ctx(&w, id).unwrap().arena, Some(a));
            // the countdown never delivers 0: begin by hand, then the last side falls
            flow::send_begin_gump(&mut w, id, 0);
            assert!(ctx(&w, id).unwrap().started);
            let loser = tournament::get(&w, ctrl).unwrap().parts[*parts.last().unwrap()].players[0];
            flow::on_death(&mut w, id, loser, None);
            assert!(ctx(&w, id).unwrap().finished);
            played += 1;
        }
        advance(&mut w, 10_000);
    }
    assert_eq!(played, 3, "two semifinals and the final");
    assert_eq!(tournament::get(&w, ctrl).unwrap().stage, Stage::Inactive);

    // gold to the champion, silver to the finalist, bronze to the two semifinal losers
    let rank_of = |m: MobileId| -> Vec<String> {
        find_items_by_type(&w, pack(&w, m), true, &TROPHY, |_, _| true)
            .into_iter()
            .map(|t| ops::name(&w, t).unwrap())
            .collect()
    };
    let mut all: Vec<String> = ps.iter().flat_map(|p| rank_of(*p)).collect();
    all.sort();
    assert_eq!(
        all,
        vec![
            "bronze trophy",
            "bronze trophy",
            "gold trophy",
            "silver trophy"
        ]
    );
    let t = tournament::get(&w, ctrl).unwrap();
    let champion = t.parts[t.levels.last().unwrap().matches[0].winner.unwrap()].players[0];
    assert_eq!(rank_of(champion), vec!["gold trophy"]);
    let trophy = find_items_by_type(&w, pack(&w, champion), true, &TROPHY, |_, _| true)[0];
    assert_eq!(title_of(&w, trophy).as_deref(), Some("1v1 Champion"));
    assert!(cheque_in(&w, champion).is_some(), "and the cash");
}

#[test]
fn pvp_tournament_with_an_event_controller_constructs_its_game() {
    let mut w = world();
    let ctrl = (TOURNAMENT_CONTROLLER.construct.unwrap())(&mut w);
    let a = make_arena(&mut w, Some(ctrl));
    let _ = a;
    let ec = koth::new_controller_kh(&mut w);
    ops::move_to_world_map(&mut w, ec, Point3D::new(60, 60, 0), Some(MapId::FELUCCA));
    let (p1, p2) = (player(&mut w, 100), player(&mut w, 101));
    {
        let t = tournament::get_mut(&mut w, ctrl).unwrap();
        t.event_controller = Some(ec);
        t.stage = Stage::Signup;
        t.signup_start = muo_primitives::DateTime::MIN;
        t.parts.push(TourneyPart::new(vec![p1]));
        t.parts.push(TourneyPart::new(vec![p2]));
        t.participants = vec![0, 1];
    }
    w.now = muo_primitives::DateTime::MIN + TimeSpan::from_days(365);
    tournament::slice(&mut w, ctrl);
    tournament::slice(&mut w, ctrl);
    let id = tournament::get(&w, ctrl).unwrap().levels[0].matches[0]
        .context
        .expect("a duel");
    let g = ctx(&w, id)
        .unwrap()
        .event_game
        .expect("the controller's game");
    assert_eq!((g.controller, g.kind), (ec, games::GameKind::Kh));
    assert_eq!(tournament::event_title(&w, ec), "King of the Hill");
    // OnStart ran with the duel's start: the teams are playing
    assert_eq!(games::team(&w, ec, 0).unwrap().game, Some(id));
}

#[test]
fn pvp_event_items_construct_and_round_trip_their_save_bytes() {
    use muo_io::{GenericReader, SaveReader, SaveWriter};
    let mut w = world();
    let classes: [&'static muo_server::item::ItemType; 11] = [
        &ctf::CTF_BOARD,
        &ctf::CTF_FLAG,
        &ctf::CTF_CONTROLLER,
        &dd::DD_BOARD,
        &dd::DD_WAY_POINT,
        &dd::DD_CONTROLLER,
        &koth::KH_BOARD,
        &koth::HILL_OF_THE_KING,
        &koth::KH_CONTROLLER,
        &bombing::BR_BOARD,
        &bombing::BR_CONTROLLER,
    ];
    for ty in classes.into_iter().chain([
        &bombing::BR_GOAL,
        &muo_items::misc::moongates::CONFIRMATION_MOONGATE,
    ]) {
        let id = (ty.construct.expect("constructs"))(&mut w);
        let mut out = SaveWriter::new(true);
        (ty.serialize)(&w, id, &mut out);
        let bytes = out.into_vec();
        let again = w.new_item(ty);
        let mut r = SaveReader::new(&bytes);
        (ty.deserialize)(&mut w, again, &mut r).unwrap_or_else(|e| panic!("{}: {e}", ty.full_name));
        assert_eq!(
            r.position() as usize,
            bytes.len(),
            "{} reads every byte",
            ty.full_name
        );
        let mut out = SaveWriter::new(true);
        (ty.serialize)(&w, again, &mut out);
        assert_eq!(out.into_vec(), bytes, "{}", ty.full_name);
    }
}

#[test]
fn pvp_controller_layouts_match_the_save_format() {
    use muo_io::{GenericReader, SaveReader, SaveWriter};
    let mut w = world();
    // CTFController: Item, EventController's version 0, the controller's version 2, Duration, eight teams
    let ec = ctf::new_controller_ctf(&mut w);
    {
        let c = games::ctrl_mut(&mut w, ec).unwrap();
        c.teams[0].name = Some("Red".to_owned());
        c.teams[0].color = 0x21;
        c.teams[0].origin = Point3D::new(1, 2, 3);
    }
    let mut out = SaveWriter::new(true);
    (muo_server::item::ITEM_BASE.serialize)(&w, ec, &mut out);
    let skip = out.as_slice().len();
    let mut out = SaveWriter::new(true);
    (ctf::CTF_CONTROLLER.serialize)(&w, ec, &mut out);
    let bytes = out.into_vec();
    let mut r = SaveReader::new(&bytes[skip..]);
    assert_eq!(r.read_i32().unwrap(), 0, "EventController");
    assert_eq!(r.read_i32().unwrap(), 2);
    assert_eq!(
        r.read_time_span_ticks().unwrap(),
        TimeSpan::from_mins(30).ticks()
    );
    assert_eq!(r.read_encoded_int().unwrap(), 8);
    assert_eq!(r.read_encoded_int().unwrap(), 2, "team version");
    r.read_serial().unwrap(); // board
    assert_eq!(r.read_string().unwrap().as_deref(), Some("Red"));
    assert_eq!(r.read_encoded_int().unwrap(), 0x21);
    r.read_serial().unwrap(); // flag
    assert_eq!(r.read_point3d().unwrap(), Point3D::new(1, 2, 3));
}

#[test]
fn pvp_properties_of_the_tournament_and_the_arena_are_listed_and_set() {
    use muo_content::props::{Access, Value, resolve_chain};
    use muo_server::mobile::AccessLevel;
    let mut w = world();
    let ctrl = (TOURNAMENT_CONTROLLER.construct.unwrap())(&mut w);
    let e = EntityId::Item(ctrl);
    let set = |w: &mut World, e: EntityId, name: &str, v: Value| {
        let (o, d) = resolve_chain(w, AccessLevel::Administrator, e, name, Access::WRITE)
            .unwrap_or_else(|_| panic!("{name} is listed"));
        (d.set.expect("settable"))(w, o, v).unwrap();
    };
    let get = |w: &mut World, e: EntityId, name: &str| {
        let (o, d) = resolve_chain(w, AccessLevel::Administrator, e, name, Access::READ).unwrap();
        (d.get)(w, o)
    };

    set(&mut w, e, "Tournament.ParticipantsPerMatch", Value::Int(40));
    assert_eq!(
        get(&mut w, e, "Tournament.ParticipantsPerMatch"),
        Value::Int(10),
        "clamped to 2..10 by the setter"
    );
    set(&mut w, e, "Tournament.TourneyType", Value::Enum(3));
    assert_eq!(
        tournament::get(&w, ctrl).unwrap().tourney_type,
        TourneyType::RedVsBlue
    );
    set(
        &mut w,
        e,
        "Tournament.SuddenDeath",
        Value::TimeSpan(TimeSpan::from_mins(3)),
    );
    assert_eq!(
        tournament::get(&w, ctrl).unwrap().sudden_death,
        TimeSpan::from_mins(3)
    );
    assert!(
        resolve_chain(
            &mut w,
            AccessLevel::Administrator,
            e,
            "Tournament.CurrentStage",
            Access::READ
        )
        .unwrap()
        .1
        .set
        .is_none(),
        "CurrentStage has a private setter"
    );

    let a = make_arena(&mut w, None);
    let ea = EntityId::Item(a);
    set(&mut w, ea, "Arena.Name", Value::Str("A Arena".to_owned()));
    assert_eq!(arena::data(&w, a).unwrap().name.as_deref(), Some("A Arena"));
    set(&mut w, ea, "Arena.Tournament", Value::Entity(Some(e)));
    assert_eq!(
        tournament::arenas_of(&w, ctrl),
        vec![a],
        "the tournament's arenas follow the arena's property"
    );
    set(
        &mut w,
        ea,
        "Arena.Points.EdgeEast",
        Value::Point3D(Point3D::new(7, 8, 9)),
    );
    assert_eq!(arena::data(&w, a).unwrap().points[1], Point3D::new(7, 8, 9));
    set(
        &mut w,
        ea,
        "Arena.Zone",
        Value::Rect(muo_primitives::Rectangle2D::new(10, 10, 10, 10)),
    );
    assert!(
        arena::data(&w, a).unwrap().region.is_some(),
        "a zone on a facet makes the safe zone"
    );

    let ec = ctf::new_controller_ctf(&mut w);
    set(
        &mut w,
        EntityId::Item(ec),
        "Duration",
        Value::TimeSpan(TimeSpan::from_mins(5)),
    );
    assert_eq!(
        games::ctrl(&w, ec).unwrap().duration,
        TimeSpan::from_mins(5)
    );
}

#[test]
fn pvp_the_controller_menu_has_an_edit_entry_for_game_masters() {
    let mut w = world();
    let ctrl = (TOURNAMENT_CONTROLLER.construct.unwrap())(&mut w);
    let gm = player(&mut w, 100);
    w.mobile_mut(gm).unwrap().access_level = muo_server::mobile::AccessLevel::GameMaster;
    let mut list = Vec::new();
    (TOURNAMENT_CONTROLLER.get_context_menu_entries)(&mut w, ctrl, gm, &mut list);
    let numbers: Vec<i32> = list.iter().map(|e| e.number).collect();
    assert_eq!(
        numbers,
        vec![3005101, 3005113],
        "Edit, then Start while inactive"
    );
}

#[test]
fn pvp_confirmation_moongate_keeps_its_warning_text_across_a_save() {
    use muo_io::{SaveReader, SaveWriter};
    use muo_items::misc::moongates::{CONFIRMATION_MOONGATE, confirmation};
    let mut w = world();
    let g = (CONFIRMATION_MOONGATE.construct.unwrap())(&mut w);
    muo_items::misc::moongates::confirmation_edit(&mut w, g, |d| {
        d.gump_width = 300;
        d.gump_height = 150;
        d.title_number = 1062051;
        d.message = Some(muo_io::text::TextDefinition::of_text("Spectate?"));
    });
    let mut out = SaveWriter::new(true);
    (CONFIRMATION_MOONGATE.serialize)(&w, g, &mut out);
    let bytes = out.into_vec();
    let again = w.new_item(&CONFIRMATION_MOONGATE);
    (CONFIRMATION_MOONGATE.deserialize)(&mut w, again, &mut SaveReader::new(&bytes)).unwrap();
    let d = confirmation(&w, again).unwrap();
    assert_eq!(
        (d.gump_width, d.gump_height, d.title_number),
        (300, 150, 1062051)
    );
    assert_eq!(
        d.message.as_ref().and_then(|m| m.string.as_deref()),
        Some("Spectate?")
    );
}
