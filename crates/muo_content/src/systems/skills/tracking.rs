//! Tracking, TrackArrow, TrackTimer, QuestArrow and the outgoing arrow packets.
//!
//! The "what do you wish to track" and "select the one" gumps are drawn by the Gumps crate: [`on_use`] and
//! [`display_track_who`] hand it the question or the list through [`TrackingHooks`]; the gump's responses call
//! [`track_what_response`] and [`track_who_response`].
//!
//! `configure` registers 0xBF/0x07 ([`quest_arrow_packet`]).

use indexmap::IndexMap;
use muo_net::net_state::NetState;
use muo_primitives::{Point2D, Serial, TimeSpan, utility};
use muo_server::World;
use muo_server::ids::MobileId;
use muo_server::map::MapId;
use muo_server::mobile::AccessLevel;
use muo_server::mobile::base as mb;
use muo_server::mobile::part2::enumerators;
use muo_server::net_state::NetStateId;
use muo_server::skills::{self, SkillName};
use muo_server::timer::{self, TimerHandle};

use super::common::{check_skill, msg};
use super::skills_info::set_callback;
use crate::core;
use crate::spells::helper::under_transformation_of;

/// `MaxClosest`: we only track the closest 12.
pub const MAX_CLOSEST: usize = 12;

// ---- OutgoingArrowPackets ---------------------------------------------------------------------------------------

/// `SendArrow(command, x, y, s)`: 0xBA. High Seas clients get the coordinates and serial on both commands; others
/// get the coordinates for set and -1, -1 for cancel.
pub fn arrow_packet(command: u8, x: i32, y: i32, s: Serial, high_seas: bool) -> Vec<u8> {
    let mut p = vec![0xBA, command];
    if high_seas {
        p.extend_from_slice(&(x as i16).to_be_bytes());
        p.extend_from_slice(&(y as i16).to_be_bytes());
        p.extend_from_slice(&s.value().to_be_bytes());
    } else if command == 1 {
        p.extend_from_slice(&(x as i16).to_be_bytes());
        p.extend_from_slice(&(y as i16).to_be_bytes());
    } else {
        p.extend_from_slice(&(-1i16).to_be_bytes());
        p.extend_from_slice(&(-1i16).to_be_bytes());
    }
    p
}

fn send_arrow(w: &mut World, ns: Option<NetStateId>, command: u8, x: i32, y: i32, s: Serial) {
    let Some(ns) = ns else { return };
    if w.cannot_send_packets(ns) {
        return;
    }
    let hs = w.net_state(ns).is_some_and(|d| d.caps.high_seas());
    w.send(ns, &arrow_packet(command, x, y, s, hs));
}

/// `SendCancelArrow(x, y, s)`.
pub fn send_cancel_arrow(w: &mut World, ns: Option<NetStateId>, x: i32, y: i32, s: Serial) {
    send_arrow(w, ns, 0, x, y, s);
}

/// `SendSetArrow(x, y, s)`.
pub fn send_set_arrow(w: &mut World, ns: Option<NetStateId>, x: i32, y: i32, s: Serial) {
    send_arrow(w, ns, 1, x, y, s);
}

// ---- QuestArrow, TrackArrow, TrackTimer ---------------------------------------------------------------------------

/// A `QuestArrow` (`TrackArrow` when `track` is set).
#[derive(Clone, Copy, Debug)]
pub struct QuestArrow {
    pub mobile: MobileId,
    pub target: MobileId,
    pub running: bool,
    /// `TrackArrow`: its `TrackTimer` and whether the tracker still has a tracking-info entry to clear (`m_From`).
    pub track: Option<TrackState>,
}

#[derive(Clone, Copy, Debug)]
pub struct TrackState {
    pub timer: TimerHandle,
    /// `m_From`: nulled by a right click.
    pub from: Option<MobileId>,
}

/// `PlayerMobile.QuestArrow`, and `Tracking._table`.
#[derive(Default)]
pub struct Tracking {
    arrows: IndexMap<MobileId, QuestArrow>,
    table: IndexMap<MobileId, TrackingInfo>,
}

#[derive(Clone, Copy)]
struct TrackingInfo {
    location: Point2D,
    map: Option<MapId>,
    target: MobileId,
}

type ShowTrackWhoGumpFn = fn(&mut World, MobileId, &[MobileId], i32);
/// What the Gumps crate shows.
#[derive(Default)]
pub struct TrackingHooks {
    /// `new TrackWhatGump()` after closing `TrackWhoGump` and `TrackWhatGump`.
    pub show_track_what_gump: Option<fn(&mut World, MobileId)>,
    /// `new TrackWhoGump(mobs, range)`.
    pub show_track_who_gump: Option<ShowTrackWhoGumpFn>,
}

fn state(w: &mut World) -> &mut Tracking {
    w.resources.get_mut::<Tracking>()
}

fn net_state(w: &World, m: MobileId) -> Option<NetStateId> {
    w.mobile(m).and_then(|x| x.net_state)
}

fn pos(w: &World, m: MobileId) -> (i32, i32, Serial) {
    w.mobile(m).map_or((0, 0, Serial::ZERO), |x| {
        (x.location.x, x.location.y, x.serial)
    })
}

/// `PlayerMobile.QuestArrow`.
pub fn quest_arrow(w: &mut World, pm: MobileId) -> Option<QuestArrow> {
    state(w).arrows.get(&pm).copied()
}

/// `new TrackArrow(pm, target, range)` assigned as the player's arrow (stopping the previous one), with its
/// `TrackTimer` started. Nothing is drawn until the first timer tick.
pub fn set_track_arrow(w: &mut World, pm: MobileId, target: MobileId, range: i32) {
    // PlayerMobile.QuestArrow.set: m_QuestArrow?.Stop()
    stop_arrow(w, pm);

    let timer = timer::start(
        w,
        TimeSpan::from_millis(250),
        TimeSpan::from_millis(2500),
        0,
        track_timer_tick(pm, target, range),
    );
    state(w).arrows.insert(
        pm,
        QuestArrow {
            mobile: pm,
            target,
            running: true,
            track: Some(TrackState {
                timer,
                from: Some(pm),
            }),
        },
    );
}

/// `TrackTimer.OnTick`, as the timer's closure (`m_LastX/Y` are its captured state).
fn track_timer_tick(
    from: MobileId,
    target: MobileId,
    range: i32,
) -> impl FnMut(&mut World) + 'static {
    let (mut last_x, mut last_y) = (0, 0);
    move |w: &mut World| {
        let running = state(w)
            .arrows
            .get(&from)
            .is_some_and(|a| a.running && a.target == target);
        if !running {
            stop_timer(w, from);
            return;
        }

        let gone = {
            let f = w.mobile(from);
            let t = w.mobile(target);
            match (f, t) {
                (Some(f), Some(t)) => {
                    f.net_state.is_none()
                        || f.deleted
                        || t.deleted
                        || f.map != t.map
                        || !mb::in_range_point3d(w, from, t.location, range)
                        || (t.hidden && t.access_level > f.access_level)
                }
                _ => true,
            }
        };
        if gone {
            stop_arrow(w, from);
            stop_timer(w, from);
            return;
        }

        let (x, y, _) = pos(w, target);
        if last_x != x || last_y != y {
            last_x = x;
            last_y = y;

            update_arrow(w, from);
        }
    }
}

fn stop_timer(w: &mut World, from: MobileId) {
    if let Some(mut t) = state(w)
        .arrows
        .get(&from)
        .and_then(|a| a.track)
        .map(|t| t.timer)
    {
        t.cancel(w);
    }
}

/// `QuestArrow.Update()`.
pub fn update_arrow(w: &mut World, pm: MobileId) {
    let Some(a) = quest_arrow(w, pm) else { return };
    if !a.running {
        return;
    }
    let (x, y, s) = pos(w, a.target);
    let ns = net_state(w, a.mobile);
    send_set_arrow(w, ns, x, y, s);
}

/// `QuestArrow.Stop()`: clears the player's arrow, tells the client, and runs `OnStop` (the tracking clean-up).
pub fn stop_arrow(w: &mut World, pm: MobileId) {
    let Some(a) = quest_arrow(w, pm) else { return };
    if !a.running {
        return;
    }
    let (x, y, s) = pos(w, a.target);
    // Mobile.ClearQuestArrow()
    state(w).arrows.shift_remove(&pm);
    let ns = net_state(w, a.mobile);
    send_cancel_arrow(w, ns, x, y, s);

    // OnStop (TrackArrow)
    if let Some(t) = a.track {
        let mut timer = t.timer;
        timer.cancel(w);
        if let Some(from) = t.from {
            clear_tracking_info(w, from);
            msg(w, from, 503177); // You have lost your quarry.
        }
    }
}

/// `QuestArrow.OnClick(rightClick)` of a `TrackArrow`: a right click ends the tracking quietly.
pub fn arrow_on_click(w: &mut World, pm: MobileId, right_click: bool) {
    if !right_click {
        return;
    }
    let Some(a) = quest_arrow(w, pm) else { return };
    if a.track.is_some() {
        clear_tracking_info(w, pm);
        // m_From = null
        if let Some(t) = state(w).arrows.get_mut(&pm).and_then(|a| a.track.as_mut()) {
            t.from = None;
        }
        stop_arrow(w, pm);
    }
}

/// `QuestArrow(state, reader)`: the 0xBF/0x07 packet from the client.
pub fn quest_arrow_packet(w: &mut World, ns: &mut NetState, body: &[u8]) {
    let Some(from) = w.net_mobile(NetStateId(ns.id() as u32)) else {
        return;
    };
    if !w
        .mobile(from)
        .is_some_and(|m| m.ty.is(&crate::player_mobile::PLAYER_MOBILE))
    {
        return;
    }
    let right_click = body.first().is_some_and(|&b| b != 0);
    arrow_on_click(w, from, right_click);
}

/// `PlayerMobile.OnLogout`: `ClearQuestArrow()` forgets the arrow without stopping it.
pub fn clear_quest_arrow(w: &mut World, pm: MobileId) {
    state(w).arrows.shift_remove(&pm);
}

// ---- Tracking ---------------------------------------------------------------------------------------------------

/// `Tracking.OnUse(m)`.
pub fn on_use(w: &mut World, m: MobileId) -> TimeSpan {
    if w.mobile(m)
        .is_some_and(|x| x.ty.is(&crate::player_mobile::PLAYER_MOBILE))
    {
        msg(w, m, 1011350); // What do you wish to track?

        if let Some(f) = w
            .resources
            .get::<TrackingHooks>()
            .and_then(|h| h.show_track_what_gump)
        {
            f(w, m);
        }
    }

    TimeSpan::from_secs(10) // 10 second delay before being able to re-use a skill
}

/// `AddInfo(tracker, target)`.
pub fn add_info(w: &mut World, tracker: MobileId, target: MobileId) {
    let (loc, map) = w
        .mobile(target)
        .map(|m| (m.location, m.map))
        .unwrap_or_default();
    state(w).table.insert(
        tracker,
        TrackingInfo {
            location: Point2D::new(loc.x, loc.y),
            map,
            target,
        },
    );
}

/// `GetStalkingBonus(tracker, target)`.
pub fn get_stalking_bonus(w: &mut World, tracker: MobileId, target: MobileId) -> f64 {
    let Some(info) = state(w).table.shift_remove(&tracker) else {
        return 0.0;
    };
    let (loc, map) = w
        .mobile(target)
        .map(|m| (m.location, m.map))
        .unwrap_or_default();
    if info.target != target || info.map != map {
        return 0.0;
    }

    let x_delta = f64::from(info.location.x - loc.x);
    let y_delta = f64::from(info.location.y - loc.y);

    let bonus = (x_delta * x_delta + y_delta * y_delta).sqrt();

    if core::ml(w) {
        bonus.min(10.0 + skills::value(w, tracker, SkillName::Tracking) / 10.0)
    } else {
        bonus
    }
}

/// `ClearTrackingInfo(tracker)`.
pub fn clear_tracking_info(w: &mut World, tracker: MobileId) {
    state(w).table.shift_remove(&tracker);
}

/// `TrackWhatGump.OnResponse`: buttons 1 to 4 pick animals, monsters, human NPCs or players.
pub fn track_what_response(w: &mut World, pm: MobileId, button_id: i32) {
    if (1..=4).contains(&button_id)
        && w.mobile(pm)
            .is_some_and(|m| m.ty.is(&crate::player_mobile::PLAYER_MOBILE))
    {
        let success = check_skill(w, pm, SkillName::Tracking, 0.0, 21.1);
        display_track_who(w, success, pm, button_id - 1);
    }
}

/// `TrackWhoGump.DisplayTo(success, from, type)`.
pub fn display_track_who(w: &mut World, success: bool, from: MobileId, ty: i32) {
    if !success {
        msg(w, from, 1018092); // You see no evidence of those in the area.
        return;
    }

    if w.mobile(from).and_then(|m| m.map).is_none() {
        return;
    }

    check_skill(w, from, SkillName::Tracking, 21.1, 100.0); // Passive gain

    // 10 tiles base + 10 tiles per 10 skill
    let range = 10 + skills::value(w, from, SkillName::Tracking) as i32 / 10 * 10;

    let mobs = get_closest_mobs(w, from, range, ty);

    if !mobs.is_empty() {
        if let Some(f) = w
            .resources
            .get::<TrackingHooks>()
            .and_then(|h| h.show_track_who_gump)
        {
            f(w, from, &mobs, range);
        }
        msg(w, from, 1018093); // Select the one you would like to track.
    } else if ty == 0 {
        msg(w, from, 502991); // You see no evidence of animals in the area.
    } else if ty == 1 {
        msg(w, from, 502993); // You see no evidence of creatures in the area.
    } else {
        msg(w, from, 502995); // You see no evidence of people in the area.
    }
}

/// `TrackWhoGump.OnResponse`: button `n` picks the nth listed mobile.
pub fn track_who_response(
    w: &mut World,
    pm: MobileId,
    targets: &[MobileId],
    range: i32,
    button_id: i32,
) {
    let index = button_id - 1;
    if index >= 0
        && (index as usize) < targets.len()
        && (index as usize) < MAX_CLOSEST
        && w.mobile(pm)
            .is_some_and(|m| m.ty.is(&crate::player_mobile::PLAYER_MOBILE))
    {
        let m = targets[index as usize];

        set_track_arrow(w, pm, m, range * 2);

        if core::se(w) {
            add_info(w, pm, m);
        }
    }
}

/// `GetClosestMobs(from, range, type)`: the closest 12, nearest first.
pub fn get_closest_mobs(w: &mut World, from: MobileId, range: i32, ty: i32) -> Vec<MobileId> {
    let loc = w.mobile(from).map(|m| m.location).unwrap_or_default();

    // We only track the closest 12
    let mut mobs: Vec<MobileId> = Vec::with_capacity(MAX_CLOSEST);
    let mut distances: Vec<f64> = Vec::with_capacity(MAX_CLOSEST);
    let mut max_distance = f64::MAX;

    for (m, min_distance) in enumerators::get_mobiles_in_range_by_distance(w, from, range) {
        if mobs.len() == MAX_CLOSEST && f64::from(min_distance) > max_distance {
            break;
        }

        let (alive, hidden, level) = match w.mobile(m) {
            Some(x) => ((x.ty.alive)(w, m), x.hidden, x.access_level),
            None => continue,
        };
        let from_level = w
            .mobile(from)
            .map_or(AccessLevel::Player, |x| x.access_level);
        if m == from
            || (core::aos(w) && !alive)
            || (hidden && level != AccessLevel::Player && from_level <= level)
            || !is_valid_mobile_type(w, m, ty)
            || !check_difficulty(w, from, m)
        {
            continue;
        }

        let mloc = w.mobile(m).map(|x| x.location).unwrap_or_default();
        let distance = utility::distance_to_sqrt_3d(mloc, loc);

        if mobs.len() >= MAX_CLOSEST && distance >= max_distance {
            continue;
        }

        // The first slot whose distance is greater: the new one goes before it.
        let search_limit = mobs.len().min(MAX_CLOSEST - 1);
        let mut insert_index = search_limit;
        for (i, d) in distances.iter().take(search_limit).enumerate() {
            if distance < *d {
                insert_index = i;
                break;
            }
        }

        if mobs.len() == MAX_CLOSEST {
            // Shift elements to make room for insertion: the farthest one falls off the end.
            mobs.pop();
            distances.pop();
        }
        mobs.insert(insert_index, m);
        distances.insert(insert_index, distance);

        if mobs.len() == MAX_CLOSEST {
            max_distance = distances[MAX_CLOSEST - 1];
        }
    }

    mobs
}

/// Tracking players uses tracking and detect hidden vs. hiding and stealth.
fn check_difficulty(w: &mut World, from: MobileId, m: MobileId) -> bool {
    if !core::aos(w) || !w.mobile(m).is_some_and(|x| x.player) {
        return true;
    }

    let mut tracking = skills::fixed(w, from, SkillName::Tracking);
    let detect_hidden = skills::fixed(w, from, SkillName::DetectHidden);

    if core::ml(w) && w.mobile(m).and_then(|x| x.race) == Some(muo_server::race::RaceId(1)) {
        tracking /= 2; // The 'Guide' says that it requires twice as Much tracking SKILL to track an elf.  Not the total difficulty to track.
    }

    let hiding = skills::fixed(w, m, SkillName::Hiding);
    let stealth = skills::fixed(w, m, SkillName::Stealth);
    let mut divisor = hiding + stealth;

    // Necromancy forms affect tracking difficulty
    if under_transformation_of(w, m, "Server.Spells.Necromancy.HorrificBeastSpell") {
        divisor -= 200;
    } else if under_transformation_of(w, m, "Server.Spells.Necromancy.VampiricEmbraceSpell")
        && divisor < 500
    {
        divisor = 500;
    } else if under_transformation_of(w, m, "Server.Spells.Necromancy.WraithFormSpell")
        && divisor <= 2000
    {
        divisor += 200;
    }

    let chance = if divisor > 0 {
        if core::se(w) {
            50 * (tracking * 2 + detect_hidden) / divisor
        } else {
            50 * (tracking + detect_hidden + 10 * utility::random_min_max(&mut w.rng, 1, 20))
                / divisor
        }
    } else {
        100
    };

    chance >= 100 || chance > utility::random(&mut w.rng, 100)
}

fn is_valid_mobile_type(w: &World, m: MobileId, ty: i32) -> bool {
    let Some(mob) = w.mobile(m) else { return false };
    let table = w.resources.get::<muo_primitives::BodyTable>();
    let player = mob.player;
    // `m.Body` is the body mod when there is one.
    let body = muo_server::mobile::part1::props_b::body(w, m);
    match ty {
        0 => !player && table.is_some_and(|t| t.is_animal(body)),
        1 => !player && table.is_some_and(|t| t.is_monster(body)),
        2 => !player && table.is_some_and(|t| t.is_human(body)),
        _ => player,
    }
}

/// `Tracking.Configure()`.
pub fn configure(w: &mut World) {
    crate::network::extended_commands::register_extended(w, 0x07, true, quest_arrow_packet);
}

pub fn initialize(w: &mut World) {
    set_callback(w, SkillName::Tracking, on_use);
}

#[cfg(test)]
mod tests {
    use super::*;
    use muo_server::map::test_support::{p, place_mobile, world};
    use muo_server::skills::Skills;

    const FEL: MapId = MapId::FELUCCA;

    fn tracker_world(center: muo_primitives::Point3D) -> (World, MobileId) {
        let mut w = world();
        let (table, _) = muo_primitives::BodyTable::parse("216\tAnimal\n400\tHuman\n");
        w.resources.insert(table);
        let rows: Vec<String> = (0..58)
            .map(|i| {
                format!(
                    r#"{{"SkillID":{i},"Name":"S{i}","Title":"T","StrScale":0,"DexScale":0,"IntScale":0,"StatTotal":0,"StrGain":0,"DexGain":0,"IntGain":0,"GainFactor":1.0,"PrimaryStat":"Str","SecondaryStat":"Dex"}}"#
                )
            })
            .collect();
        w.resources.get_mut::<muo_server::skills::SkillTable>().info =
            muo_data::skill_info::SkillInfo::from_json(&format!("[{}]", rows.join(","))).unwrap();
        let t = place_mobile(&mut w, FEL, center);
        w.mobile_mut(t).unwrap().skills = Some(Skills::default());
        skills::set_base_fixed(&mut w, t, SkillName::Tracking, 1000); // 100.0 skill = 110 range
        (w, t)
    }

    fn animal(w: &mut World, at: muo_primitives::Point3D) -> MobileId {
        let a = place_mobile(w, FEL, at);
        w.mobile_mut(a).unwrap().body = muo_primitives::Body(216); // Llama body - an animal body
        a
    }

    fn range_of(w: &mut World, t: MobileId) -> i32 {
        (10 + skills::value(w, t, SkillName::Tracking) as i32).clamp(0, 100)
    }

    #[test]
    fn finds_the_closest_12_when_many_are_available() {
        let center = p(1000, 1000, 0);
        let (mut w, t) = tracker_world(center);
        let animals: Vec<_> = (1..=20)
            .map(|d| animal(&mut w, p(1000 + d, 1000, 0)))
            .collect();
        let range = range_of(&mut w, t);
        let result = get_closest_mobs(&mut w, t, range, 0);

        assert_eq!(result.len(), 12);
        // sorted by distance, the first is the closest, the last at distance 12, the 13th is not there
        assert_eq!(result[0], animals[0]);
        assert_eq!(result[11], animals[11]);
        assert!(!result.contains(&animals[12]));
        let dist =
            |m: MobileId| utility::distance_to_sqrt_3d(w.mobile(m).unwrap().location, center);
        for pair in result.windows(2) {
            assert!(dist(pair[0]) <= dist(pair[1]));
        }
    }

    #[test]
    fn finds_all_when_exactly_12_or_fewer_are_available() {
        let (mut w, t) = tracker_world(p(2000, 2000, 0));
        let twelve: Vec<_> = (1..=12)
            .map(|d| animal(&mut w, p(2000 + d, 2000, 0)))
            .collect();
        let range = range_of(&mut w, t);
        let result = get_closest_mobs(&mut w, t, range, 0);
        assert_eq!(result.len(), 12);
        assert!(twelve.iter().all(|m| result.contains(m)));

        let (mut w, t) = tracker_world(p(3000, 3000, 0));
        let five: Vec<_> = (1..=5)
            .map(|d| animal(&mut w, p(3000 + d, 3000, 0)))
            .collect();
        let range = range_of(&mut w, t);
        let result = get_closest_mobs(&mut w, t, range, 0);
        assert_eq!(result.len(), 5);
        assert!(five.iter().all(|m| result.contains(m)));
    }

    #[test]
    fn early_exit_keeps_the_true_closest_when_sectors_interleave() {
        let center = p(1500, 1500, 0);
        let (mut w, t) = tracker_world(center);
        let mobs: Vec<_> = (0..15)
            .map(|i| {
                let d = i + 1;
                let at = if i % 2 == 0 {
                    p(1500 + d, 1500, 0)
                } else {
                    p(1500, 1500 + d, 0)
                };
                animal(&mut w, at)
            })
            .collect();
        let range = range_of(&mut w, t);
        let result = get_closest_mobs(&mut w, t, range, 0);

        let dist =
            |m: MobileId| utility::distance_to_sqrt_3d(w.mobile(m).unwrap().location, center);
        let mut all: Vec<(MobileId, f64)> = mobs.iter().map(|&m| (m, dist(m))).collect();
        all.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        assert_eq!(result.len(), 12);
        for m in &result {
            assert!(all.iter().take(12).any(|(x, _)| x == m));
        }
        for pair in result.windows(2) {
            assert!(dist(pair[0]) <= dist(pair[1]), "sorted by distance");
        }
        let max_in = result.iter().map(|&m| dist(m)).fold(0.0, f64::max);
        let min_out = all.iter().skip(12).map(|a| a.1).fold(f64::MAX, f64::min);
        assert!(max_in <= min_out);
    }

    #[test]
    fn mobile_types_select_players_animals_and_humans() {
        let (mut w, t) = tracker_world(p(100, 100, 0));
        let a = animal(&mut w, p(101, 100, 0));
        let h = place_mobile(&mut w, FEL, p(102, 100, 0));
        w.mobile_mut(h).unwrap().body = muo_primitives::Body(400);
        let pl = place_mobile(&mut w, FEL, p(103, 100, 0));
        w.mobile_mut(pl).unwrap().player = true;
        assert!(
            is_valid_mobile_type(&w, a, 0)
                && !is_valid_mobile_type(&w, a, 2)
                && !is_valid_mobile_type(&w, a, 3)
        );
        assert!(is_valid_mobile_type(&w, h, 2) && !is_valid_mobile_type(&w, h, 0));
        assert!(is_valid_mobile_type(&w, pl, 3) && !is_valid_mobile_type(&w, pl, 0));
        let _ = t;
    }

    #[test]
    fn stalking_bonus_is_the_distance_walked_once() {
        let (mut w, t) = tracker_world(p(100, 100, 0));
        let target = place_mobile(&mut w, FEL, p(110, 100, 0));
        w.core.expansion = Some(muo_data::expansion::Expansion::SE);
        add_info(&mut w, t, target);
        w.mobile_mut(target).unwrap().location = p(113, 104, 0);
        assert_eq!(get_stalking_bonus(&mut w, t, target), 5.0);
        // The entry is used up.
        assert_eq!(get_stalking_bonus(&mut w, t, target), 0.0);
        // From ML on it is capped at 10 + Tracking / 10 = 20.
        w.core.expansion = Some(muo_data::expansion::Expansion::ML);
        add_info(&mut w, t, target);
        w.mobile_mut(target).unwrap().location = p(213, 104, 0);
        assert_eq!(get_stalking_bonus(&mut w, t, target), 20.0);
        // A different target resets nothing but returns no bonus.
        add_info(&mut w, t, target);
        let other = place_mobile(&mut w, FEL, p(120, 100, 0));
        assert_eq!(get_stalking_bonus(&mut w, t, other), 0.0);
    }

    #[test]
    fn arrow_packets() {
        let s = Serial::new(0x1024);
        assert_eq!(
            arrow_packet(0, 0, 0, Serial::ZERO, false),
            [0xBA, 0, 0xFF, 0xFF, 0xFF, 0xFF]
        );
        assert_eq!(
            arrow_packet(1, 100, 10, Serial::ZERO, false),
            [0xBA, 1, 0, 100, 0, 10]
        );
        assert_eq!(
            arrow_packet(0, 100, 10, s, true),
            [0xBA, 0, 0, 100, 0, 10, 0, 0, 0x10, 0x24]
        );
        assert_eq!(
            arrow_packet(1, 100, 10, s, true),
            [0xBA, 1, 0, 100, 0, 10, 0, 0, 0x10, 0x24]
        );
        // x, y written as shorts
        assert_eq!(
            arrow_packet(1, 100_000, 100_000, Serial::ZERO, false)[2..],
            [0x86, 0xA0, 0x86, 0xA0]
        );
    }

    // Arrow packet tests: the legacy writers (CancelArrow,
    // SetArrow, CancelArrowHS, SetArrowHS) spelled out as expected bytes, compared with what a connection is sent.
    fn short(v: i32) -> [u8; 2] {
        (v as i16).to_be_bytes()
    }

    fn sent_arrow(high_seas: bool, set: bool, x: i32, y: i32, s: Serial) -> Vec<u8> {
        use muo_net::protocol_changes::ProtocolChanges;
        use muo_server::net_state::{NetCaps, NetOut, NetStateData};
        let mut w = World::new();
        let ns = NetStateId(1);
        let caps = NetCaps {
            protocol_changes: if high_seas {
                ProtocolChanges::HIGH_SEAS
            } else {
                ProtocolChanges::empty()
            },
            ..Default::default()
        };
        w.net_states.insert(
            ns,
            NetStateData {
                caps,
                ..Default::default()
            },
        );
        if set {
            send_set_arrow(&mut w, Some(ns), x, y, s);
        } else {
            send_cancel_arrow(&mut w, Some(ns), x, y, s);
        }
        match w.net_out.pop() {
            Some(NetOut::Send(_, b)) => b,
            _ => Vec::new(),
        }
    }

    #[test]
    fn cancel_arrow() {
        // new CancelArrow(): 0xBA, 0, -1, -1
        assert_eq!(
            sent_arrow(false, false, 0, 0, Serial::ZERO),
            [0xBA, 0, 0xFF, 0xFF, 0xFF, 0xFF]
        );
    }

    #[test]
    fn set_arrow() {
        for (x, y) in [(0, 0), (100, 10), (100000, 100000)] {
            let mut want = vec![0xBA, 1];
            want.extend(short(x));
            want.extend(short(y));
            assert_eq!(sent_arrow(false, true, x, y, Serial::ZERO), want, "{x} {y}");
        }
    }

    #[test]
    fn cancel_arrow_hs() {
        let serial = Serial::new(0x1024);
        for (x, y) in [(0, 0), (100, 10), (100000, 100000)] {
            let mut want = vec![0xBA, 0];
            want.extend(short(x));
            want.extend(short(y));
            want.extend(0x1024u32.to_be_bytes());
            assert_eq!(sent_arrow(true, false, x, y, serial), want, "{x} {y}");
        }
    }

    #[test]
    fn set_arrow_hs() {
        let serial = Serial::new(0x1024);
        for (x, y) in [(0, 0), (100, 10), (100000, 100000)] {
            let mut want = vec![0xBA, 1];
            want.extend(short(x));
            want.extend(short(y));
            want.extend(0x1024u32.to_be_bytes());
            assert_eq!(sent_arrow(true, true, x, y, serial), want, "{x} {y}");
        }
    }
}
