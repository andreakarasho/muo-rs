//! King of the Hill: stand on a hill until it scores. `KHController` (eight teams, four
//! hills, boards), `HillOfTheKing`, `KHBoard` and the game's rules.

use muo_io::{GenericReader, GenericWriter};
use muo_primitives::{MessageType, TimeSpan};
use muo_server::World;
use muo_server::ids::{ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType, ops};
use muo_server::mobile::AccessLevel;
use muo_server::serialization_ext::{ReadEntityExt, WriteEntityExt};
use muo_server::timer::{self, TimerHandle};
use muo_server::type_registry::TypeRegistry;

use super::games::{
    self, EVENT_CONTROLLER, EventCtrl, GameKind, TeamInfo, alert, ctrl, ctrl_mut, display_board,
    new_controller, team, team_mut, team_of,
};
use super::{ctx, flow};

// ---- KHBoard ----------------------------------------------------------------------------------------------------------

/// `KHBoard`: `Controller` is saved, `m_Game` (a context id) is not.
#[derive(Clone, Copy, Debug, Default)]
pub struct KhBoardData {
    pub controller: Option<ItemId>,
    pub game: Option<u32>,
}

fn board(w: &World, id: ItemId) -> KhBoardData {
    w.item(id)
        .and_then(|i| i.ext.get::<KhBoardData>())
        .copied()
        .unwrap_or_default()
}

/// `KHBoard.Controller = value`: `OnControllerChanged` removes it from the old controller's boards, adds it to the new.
pub fn set_board_controller(w: &mut World, b: ItemId, value: Option<ItemId>) {
    let old = board(w, b).controller;
    if old == value {
        return;
    }
    if let Some(d) = w.item_mut(b).and_then(|i| i.ext.get_mut::<KhBoardData>()) {
        d.controller = value;
    }
    if let Some(c) = old.and_then(|o| ctrl_mut(w, o)) {
        c.boards.retain(|x| *x != b);
        if let Some(d) = w.item_mut(b).and_then(|i| i.ext.get_mut::<KhBoardData>()) {
            d.game = None;
        }
    }
    if let Some(c) = value.and_then(|v| ctrl_mut(w, v)) {
        c.boards.push(b);
    }
}

/// `KHBoard`.
pub static KH_BOARD: ItemType = ItemType {
    full_name: "Server.Engines.ConPVP.KHBoard",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let id = ops::new_item_with_id(w, &KH_BOARD, 0);
        muo_content::templates::apply_item_as(w, id, KH_BOARD.full_name);
        if let Some(i) = w.item_mut(id) {
            i.ext.insert(KhBoardData::default());
        }
        id
    }),
    on_double_click: |w, id, from| match board(w, id).game {
        Some(game) => display_board(w, game, from),
        None => muo_server::item::deps::send_message(
            w,
            from,
            "There is no King of the Hill game in progress.",
        ),
    },
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        out.write_i32(0);
        out.write_item(w, board(w, id).controller);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        let controller = r.read_item(w)?;
        if let Some(i) = w.item_mut(id) {
            i.ext.insert(KhBoardData {
                controller,
                game: None,
            });
        }
        Ok(())
    },
    ..ITEM_BASE
};

// ---- HillOfTheKing ----------------------------------------------------------------------------------------------------

/// `HillOfTheKing`: `ScoreInterval` is saved; the rest is the game it is in and the `KingTimer`.
#[derive(Clone, Debug, Default)]
pub struct HillData {
    pub score_interval: i32,
    pub game: Option<u32>,
    pub king: Option<MobileId>,
    /// `m_KingTimer` (None until the first king) and its `Captures`.
    pub has_timer: bool,
    pub timer: TimerHandle,
    pub captures: i32,
    pub counter: i32,
}

fn hill(w: &World, id: ItemId) -> Option<&HillData> {
    w.item(id)?.ext.get::<HillData>()
}

fn hill_mut(w: &mut World, id: ItemId) -> Option<&mut HillData> {
    w.item_mut(id)?.ext.get_mut::<HillData>()
}

/// `King`.
pub fn king(w: &World, id: ItemId) -> Option<MobileId> {
    hill(w, id).and_then(|h| h.king)
}

/// `CapturesSoFar`.
pub fn captures_so_far(w: &World, id: ItemId) -> i32 {
    hill(w, id).map_or(0, |h| h.captures)
}

fn stop_timer(w: &mut World, id: ItemId) {
    if let Some(h) = hill_mut(w, id) {
        let mut t = std::mem::take(&mut h.timer);
        t.cancel(w);
    }
}

/// `Game = value`.
pub fn set_hill_game(w: &mut World, id: ItemId, game: Option<u32>) {
    if hill(w, id).is_some_and(|h| h.game != game) {
        stop_timer(w, id);
        if let Some(h) = hill_mut(w, id) {
            h.game = game;
            h.king = None;
        }
    }
}

fn say(w: &mut World, id: ItemId, text: &str) {
    muo_server::item::messages::public_overhead_message(
        w,
        id,
        muo_server::packets::message::MessageType::REGULAR,
        0x0481,
        false,
        text,
    );
}

fn can_be_king(w: &World, id: ItemId, m: MobileId) -> bool {
    let Some(h) = hill(w, id) else { return false };
    let Some(game) = h.game else { return false };
    // Mobile exists and is alive and is a player?
    if muo_content::mx::deleted(w, m)
        || !muo_content::mx::alive(w, m)
        || !muo_content::mx::player(w, m)
    {
        return false;
    }
    // Not current king (or they are the current king)
    if h.king.is_some_and(|k| k != m) {
        return false;
    }
    // They are on a team
    team_of(w, game, m).is_some()
}

fn de_kingify(w: &mut World, id: ItemId) {
    say(w, id, "Free!");
    stop_timer(w, id);
    if let Some(h) = hill_mut(w, id) {
        h.king = None;
    }
}

fn re_kingify(w: &mut World, id: ItemId, m: MobileId) {
    let Some(game) = hill(w, id).and_then(|h| h.game) else {
        return;
    };
    if team_of(w, game, m).is_none() {
        return;
    }
    stop_timer(w, id);
    let one = TimeSpan::from_secs(1);
    let t = timer::start(w, one, one, 0, move |w| king_tick(w, id));
    if let Some(h) = hill_mut(w, id) {
        h.king = Some(m);
        h.has_timer = true;
        h.timer = t;
        // StartHillTicker
        h.captures = 0;
        h.counter = 0;
    }
    // `King.Name != null` (an empty name still announces)
    if let Some(name) = w.mobile(m).and_then(|x| (x.ty.name)(w, m)) {
        say(w, id, &format!("Taken by {name}!"));
    }
}

/// `KingTimer.OnTick`.
fn king_tick(w: &mut World, id: ItemId) {
    let Some(h) = hill(w, id).cloned() else {
        return;
    };
    let Some(game) = h.game.filter(|_| !ops::is_deleted(w, id)) else {
        stop_timer(w, id);
        return;
    };
    let Some(king) = h
        .king
        .filter(|k| !muo_content::mx::deleted(w, *k) && muo_content::mx::alive(w, *k))
    else {
        de_kingify(w, id);
        stop_timer(w, id);
        return;
    };
    let Some(ti) = team_of(w, game, king) else {
        de_kingify(w, id);
        stop_timer(w, id);
        return;
    };

    // `pi = ti[King]` makes the king's `KHPlayerInfo` when absent (it is in `Players` for the trophies from now on)
    if let Some(t) = ctx(w, game)
        .and_then(|c| c.event_game)
        .and_then(|g| team_mut(w, g.controller, ti))
    {
        t.player(king);
    }

    let counter = h.counter + 1;
    muo_server::mobile::base::revealing_action(w, king);

    if counter >= h.score_interval {
        let name = ops::name(w, id)
            .filter(|n| !n.is_empty())
            .unwrap_or_else(|| "the hill".to_owned());
        let kname = flow::name(w, king);
        let ec = ctx(w, game)
            .and_then(|c| c.event_game)
            .map(|g| g.controller);
        let tname = ec
            .and_then(|e| team(w, e, ti))
            .map(|t| t.text_name(GameKind::Kh))
            .unwrap_or_default();
        alert(w, game, &format!("{kname} ({tname}) is king of {name}!"));
        say(w, id, "Capture!");
        if let Some(t) = ec.and_then(|e| team_mut(w, e, ti)) {
            t.add_captures(king, 1);
            t.add_score(king, counter);
        }
        if let Some(h) = hill_mut(w, id) {
            h.captures += 1;
            h.counter = 0;
        }
    } else {
        if let Some(h) = hill_mut(w, id) {
            h.counter = counter;
        }
        say(w, id, &(h.score_interval - counter).to_string());
    }
}

/// `OnKingDied(king, kingTeam, killer, killerTeam)`.
fn on_king_died(
    w: &mut World,
    id: ItemId,
    game: u32,
    king: MobileId,
    king_team: Option<usize>,
    killer: Option<MobileId>,
    killer_team: Option<usize>,
) {
    if let (Some(killer), Some(kt), Some(lt)) = (killer, king_team, killer_team)
        && captures_so_far(w, id) > 0
        && let Some(ec) = ctx(w, game)
            .and_then(|c| c.event_game)
            .map(|g| g.controller)
    {
        let (kn, ln) = (flow::name(w, king), flow::name(w, killer));
        let (kteam, lteam) = (
            team(w, ec, kt)
                .map(|t| t.text_name(GameKind::Kh))
                .unwrap_or_default(),
            team(w, ec, lt)
                .map(|t| t.text_name(GameKind::Kh))
                .unwrap_or_default(),
        );
        alert(
            w,
            game,
            &format!("{kn} ({kteam}) was dethroned by {ln} ({lteam})!"),
        );
    }
    de_kingify(w, id);
}

/// `HillOfTheKing`.
pub static HILL_OF_THE_KING: ItemType = ItemType {
    full_name: "Server.Engines.ConPVP.HillOfTheKing",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let id = ops::new_item_with_id(w, &HILL_OF_THE_KING, 0);
        muo_content::templates::apply_item_as(w, id, HILL_OF_THE_KING.full_name);
        if let Some(i) = w.item_mut(id) {
            i.ext.insert(HillData::default());
        }
        id
    }),
    on_move_over: |w, id, m| {
        if hill(w, id).is_none_or(|h| h.game.is_none()) || !muo_content::mx::alive(w, m) {
            return (ITEM_BASE.on_move_over)(w, id, m);
        }
        if can_be_king(w, id, m) {
            if (ITEM_BASE.on_move_over)(w, id, m) {
                re_kingify(w, id, m);
                return true;
            }
        } else if muo_content::mx::access_level(w, m) == AccessLevel::Player {
            // Decrease their stam a little so they don't keep pushing someone out of the way
            let (stam, max) = (
                w.mobile(m).map_or(0, |x| x.stam),
                w.mobile(m).map_or(0, |x| (x.ty.stam_max)(w, m)),
            );
            if stam >= max {
                muo_server::mobile::part1::props_b::set_stam(w, m, stam - 5);
            }
        }
        false
    },
    on_move_off: |w, id, m| {
        if (ITEM_BASE.on_move_off)(w, id, m) {
            if king(w, id) == Some(m) {
                de_kingify(w, id);
            }
            return true;
        }
        false
    },
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        out.write_i32(0);
        out.write_encoded_int(hill(w, id).map_or(0, |h| h.score_interval));
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        let score_interval = r.read_encoded_int()?;
        if let Some(i) = w.item_mut(id) {
            i.ext.insert(HillData {
                score_interval,
                ..HillData::default()
            });
        }
        Ok(())
    },
    ..ITEM_BASE
};

// ---- KHController -----------------------------------------------------------------------------------------------------

/// `new KHController()`.
pub fn new_controller_kh(w: &mut World) -> ItemId {
    new_controller(w, &KH_CONTROLLER, EventCtrl::new(GameKind::Kh, 8))
}

/// `KHTeamInfo.Serialize`.
fn write_team(t: &TeamInfo, out: &mut dyn GenericWriter) {
    out.write_encoded_int(0);
    out.write_opt_string(t.name.as_deref());
    out.write_encoded_int(t.color);
}

fn read_team(id: usize, r: &mut dyn GenericReader) -> muo_io::Result<TeamInfo> {
    let mut t = TeamInfo::new(id);
    if r.read_encoded_int()? == 0 {
        t.name = r.read_string()?;
        t.color = r.read_encoded_int()?;
    }
    Ok(t)
}

/// `KHController`.
pub static KH_CONTROLLER: ItemType = ItemType {
    full_name: "Server.Engines.ConPVP.KHController",
    parent: Some(&EVENT_CONTROLLER),
    construct: Some(new_controller_kh),
    serialize: |w, id, out| {
        (EVENT_CONTROLLER.serialize)(w, id, out);
        out.write_i32(0);
        let Some(c) = ctrl(w, id) else { return };
        out.write_encoded_int(c.score_interval);
        out.write_time_span_ticks(c.duration.ticks());
        // `Boards.Tidy()`
        let boards: Vec<ItemId> = c
            .boards
            .iter()
            .copied()
            .filter(|b| !ops::is_deleted(w, *b))
            .collect();
        out.write_item_list(w, &boards);
        out.write_encoded_int(c.hills.len() as i32);
        for h in &c.hills {
            out.write_item(w, *h);
        }
        out.write_encoded_int(c.teams.len() as i32);
        for t in &c.teams {
            write_team(t, out);
        }
    },
    deserialize: |w, id, r| {
        (EVENT_CONTROLLER.deserialize)(w, id, r)?;
        let version = r.read_i32()?;
        let mut c = EventCtrl::new(GameKind::Kh, 8);
        if version == 0 {
            c.score_interval = r.read_encoded_int()?;
            c.duration = TimeSpan::from_ticks(r.read_time_span_ticks()?);
            c.boards = r.read_item_list(w)?;
            let n = r.read_encoded_int()?.max(0) as usize;
            c.hills = Vec::with_capacity(n);
            for _ in 0..n {
                c.hills.push(r.read_item(w)?);
            }
            let n = r.read_encoded_int()?.max(0) as usize;
            c.teams = Vec::with_capacity(n);
            for i in 0..n {
                c.teams.push(read_team(i, r)?);
            }
        }
        if let Some(i) = w.item_mut(id) {
            i.ext.insert(c);
        }
        Ok(())
    },
    ..EVENT_CONTROLLER
};

// ---- KHGame -----------------------------------------------------------------------------------------------------------

/// `KHGame.CantDoAnything(mob)`: the king can do nothing.
pub fn cant_do_anything(w: &World, id: u32, ec: ItemId, mob: MobileId) -> bool {
    team_of(w, id, mob).is_some()
        && ctrl(w, ec).is_some_and(|c| c.hills.iter().flatten().any(|h| king(w, *h) == Some(mob)))
}

/// `KHGame.OnDeath`.
pub fn on_death(w: &mut World, id: u32, ec: ItemId, mob: MobileId, corpse: Option<ItemId>) {
    let killer = muo_server::mobile::part2::damage::find_most_recent_damager(w, mob, false);
    let vict = team_of(w, id, mob);
    let mut bonus = 0;
    let killer_team = killer
        .filter(|k| muo_content::mx::player(w, *k))
        .and_then(|k| team_of(w, id, k));

    let hills: Vec<ItemId> = ctrl(w, ec)
        .map(|c| c.hills.iter().flatten().copied().collect())
        .unwrap_or_default();
    for h in hills {
        if king(w, h) == Some(mob) {
            bonus += captures_so_far(w, h);
            on_king_died(w, h, id, mob, vict, killer, killer_team);
        }
        // `Hills[i].King == killer`: a hill without a king (and a death without a killer) count too
        if king(w, h) == killer {
            bonus += 2;
        }
    }

    if let (Some(ti), Some(killer)) = (killer_team, killer)
        && Some(ti) != vict
        && let Some(t) = team_mut(w, ec, ti)
    {
        t.add_kills(killer, 1);
        t.add_score(killer, 1 + bonus);
    }

    display_board(w, id, mob);
    flow::requip(w, id, mob, corpse);
    games::delay_bounce(w, id, TimeSpan::from_secs(30), mob, corpse, true);
}

/// `KHGame.OnStart`.
pub fn on_start(w: &mut World, id: u32, ec: ItemId) {
    let n = ctrl(w, ec).map_or(0, |c| c.teams.len());
    for i in 0..n {
        if let Some(t) = team_mut(w, ec, i) {
            t.game = Some(id);
            t.reset_scores();
        }
    }
    let parts = ctx(w, id).map_or(0, |c| c.participants.len());
    for i in 0..parts {
        if let Some(color) = team(w, ec, i % n.max(1)).map(|t| t.color) {
            games::apply_hues(w, id, i, color);
        }
    }
    let (hills, boards) = ctrl(w, ec)
        .map(|c| {
            (
                c.hills.iter().flatten().copied().collect::<Vec<_>>(),
                c.boards.clone(),
            )
        })
        .unwrap_or_default();
    for h in hills {
        set_hill_game(w, h, Some(id));
    }
    for b in boards {
        if !ops::is_deleted(w, b)
            && let Some(d) = w.item_mut(b).and_then(|i| i.ext.get_mut::<KhBoardData>())
        {
            d.game = Some(id);
        }
    }
    games::start_finish_timer(w, id, ec);
}

/// `KHGame.OnStop`.
pub fn on_stop(w: &mut World, id: u32, ec: ItemId) {
    let n = ctrl(w, ec).map_or(0, |c| c.teams.len());
    for i in 0..n {
        if let Some(t) = team_mut(w, ec, i) {
            t.game = None;
        }
    }
    let (hills, boards) = ctrl(w, ec)
        .map(|c| {
            (
                c.hills.iter().flatten().copied().collect::<Vec<_>>(),
                c.boards.clone(),
            )
        })
        .unwrap_or_default();
    for h in hills {
        set_hill_game(w, h, None);
    }
    for b in boards {
        if let Some(d) = w.item_mut(b).and_then(|i| i.ext.get_mut::<KhBoardData>()) {
            d.game = None;
        }
    }
    let parts = ctx(w, id).map_or(0, |c| c.participants.len());
    for i in 0..parts {
        games::apply_hues(w, id, i, -1);
    }
}

/// Registers the classes.
pub fn register(types: &mut TypeRegistry) {
    types.register_item(&KH_BOARD, &[]);
    types.register_item(&HILL_OF_THE_KING, &[]);
    types.register_item(&KH_CONTROLLER, &[]);
}

#[allow(dead_code)]
fn _unused() {
    let _ = MessageType::REGULAR;
}
