//! `EventGame` and what CTF, DoubleDom, KingOfTheHill and BombingRun share: the
//! `EventController` item, the team/player score books, the scoreboard gump, the awards at the end of a game and the
//! respawn after a death.
//!
//! An `EventGame` is not an object here: the context carries [`EventGameFlags`] (the controller and its kind) and the
//! game's running state is in [`Games`] while it plays (`OnStart` .. `OnStop`). The controller holds the teams
//! ([`TeamInfo`], each game's `XTeamInfo`) with their scores. `TeamInfo.Game` is the id of the duel context playing.
//!
//! The nested `[PropertyObject]`s (`Team1`..`Team8`) are `Kind::Object` entries of the property registry (`props`).

use std::collections::HashMap;

use indexmap::IndexMap;
use muo_content::gumps::{
    DynamicGumpBuilder, DynamicGumpLogic, DynamicGumpObj, send_gump_to_mobile,
};
use muo_io::{GenericReader, GenericWriter};
use muo_primitives::{Point3D, TimeSpan, html};
use muo_server::World;
use muo_server::ids::{ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType, ops};
use muo_server::map::MapId;
use muo_server::mobile::AccessLevel;
use muo_server::timer::{self, TimerHandle};

use super::speech::{n0, rank};
use super::tournament::{self, TourneyType};
use super::trophy::{TrophyRank, new_trophy};
use super::{DuelContext, EventGameFlags, arena, ctx, ctx_mut, duel_context_of, flow};

/// Which game a controller plays.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum GameKind {
    Ctf,
    Dd,
    Kh,
    Br,
}

/// `CTFPlayerInfo` / `DDPlayerInfo` / `KHPlayerInfo` / `BRPlayerInfo`.
#[derive(Clone, Debug, Default)]
pub struct PlayerInfo {
    pub kills: i32,
    pub captures: i32,
    pub score: i32,
}

/// `CTFTeamInfo` / `DDTeamInfo` / `KHTeamInfo` / `BRTeamInfo`.
#[derive(Clone, Debug)]
pub struct TeamInfo {
    pub team_id: usize,
    /// `Name` (CTF) / `TeamName`.
    pub name: Option<String>,
    pub color: i32,
    /// CTF, DD: where the team's flag goes home / the team's start.
    pub origin: Point3D,
    pub board: Option<ItemId>,
    /// CTF `Flag`.
    pub flag: Option<ItemId>,
    /// BR `Goal`.
    pub goal: Option<ItemId>,
    /// `Game`: the context id of the game being played.
    pub game: Option<u32>,
    pub leader: Option<MobileId>,
    pub players: IndexMap<MobileId, PlayerInfo>,
    pub kills: i32,
    pub captures: i32,
    pub score: i32,
}

impl TeamInfo {
    pub fn new(team_id: usize) -> TeamInfo {
        TeamInfo {
            team_id,
            name: None,
            color: 0,
            origin: Point3D::default(),
            board: None,
            flag: None,
            goal: None,
            game: None,
            leader: None,
            players: IndexMap::new(),
            kills: 0,
            captures: 0,
            score: 0,
        }
    }

    /// `Reset()` without the item links.
    pub fn reset_scores(&mut self) {
        self.kills = 0;
        self.captures = 0;
        self.score = 0;
        self.leader = None;
        self.players.clear();
    }

    /// `this[mob]` (made when absent).
    pub fn player(&mut self, m: MobileId) -> &mut PlayerInfo {
        self.players.entry(m).or_default()
    }

    /// `playerInfo.Kills += d`.
    pub fn add_kills(&mut self, m: MobileId, d: i32) {
        self.player(m).kills += d;
        self.kills += d;
    }

    /// `playerInfo.Captures += d`.
    pub fn add_captures(&mut self, m: MobileId, d: i32) {
        self.player(m).captures += d;
        self.captures += d;
    }

    /// `playerInfo.Score += d`: the team's too, and the leader changes when passed.
    pub fn add_score(&mut self, m: MobileId, d: i32) {
        let now = {
            let p = self.player(m);
            p.score += d;
            p.score
        };
        self.score += d;
        let leader_score = self
            .leader
            .and_then(|l| self.players.get(&l))
            .map(|p| p.score);
        if leader_score.is_none_or(|s| now > s) {
            self.leader = Some(m);
        }
    }

    /// The `Name` the game's text shows: CTF's raw `Name`, the others' `"{TeamName} Team"` (KH: `(none)` for none).
    pub fn text_name(&self, kind: GameKind) -> String {
        match kind {
            GameKind::Ctf => self.name.clone().unwrap_or_default(),
            GameKind::Kh => format!("{} Team", self.name.as_deref().unwrap_or("(none)")),
            _ => format!("{} Team", self.name.as_deref().unwrap_or_default()),
        }
    }

    /// `IRankedCTF.Name`: `"{Name} Team"` for CTF too.
    pub fn ranked_name(&self, kind: GameKind) -> String {
        match kind {
            GameKind::Ctf => format!("{} Team", self.name.as_deref().unwrap_or_default()),
            _ => self.text_name(kind),
        }
    }
}

/// What a controller keeps (the part its subclass serializes).
#[derive(Clone, Debug)]
pub struct EventCtrl {
    pub kind: GameKind,
    pub duration: TimeSpan,
    pub teams: Vec<TeamInfo>,
    /// DD `PointA`, `PointB`.
    pub point_a: Option<ItemId>,
    pub point_b: Option<ItemId>,
    /// KH `Hills` (four slots), `Boards`, `m_ScoreInterval`.
    pub hills: Vec<Option<ItemId>>,
    pub boards: Vec<ItemId>,
    pub score_interval: i32,
    /// BR: see `bombing`.
    pub br: super::bombing::BrCtrl,
}

impl EventCtrl {
    pub fn new(kind: GameKind, teams: usize) -> EventCtrl {
        EventCtrl {
            kind,
            duration: TimeSpan::from_mins(30),
            teams: (0..teams).map(TeamInfo::new).collect(),
            point_a: None,
            point_b: None,
            hills: if kind == GameKind::Kh {
                vec![None; 4]
            } else {
                Vec::new()
            },
            boards: Vec::new(),
            score_interval: 0,
            br: super::bombing::BrCtrl::default(),
        }
    }
}

/// The controller's data.
pub fn ctrl(w: &World, ec: ItemId) -> Option<&EventCtrl> {
    w.item(ec)?.ext.get::<EventCtrl>()
}

pub fn ctrl_mut(w: &mut World, ec: ItemId) -> Option<&mut EventCtrl> {
    w.item_mut(ec)?.ext.get_mut::<EventCtrl>()
}

/// A team of a controller.
pub fn team(w: &World, ec: ItemId, i: usize) -> Option<&TeamInfo> {
    ctrl(w, ec)?.teams.get(i)
}

pub fn team_mut(w: &mut World, ec: ItemId, i: usize) -> Option<&mut TeamInfo> {
    ctrl_mut(w, ec)?.teams.get_mut(i)
}

/// What a game keeps while it plays.
#[derive(Default)]
pub struct GameRt {
    pub finish: TimerHandle,
    /// DD.
    pub cap_stage: i32,
    pub capturable: bool,
    pub capture: TimerHandle,
    pub uncapture: TimerHandle,
    /// BR.
    pub br: super::bombing::BrRt,
}

/// The games being played, by context id.
#[derive(Default)]
pub struct Games {
    pub rt: HashMap<u32, GameRt>,
}

pub fn rt_mut(w: &mut World, id: u32) -> &mut GameRt {
    w.resources.get_mut::<Games>().rt.entry(id).or_default()
}

pub fn rt(w: &World, id: u32) -> Option<&GameRt> {
    w.resources.get::<Games>()?.rt.get(&id)
}

// ---- EventController --------------------------------------------------------------------------------------------

pub const EVENT_CONTROLLER_NAME: &str = "Server.Engines.ConPVP.EventController";

/// `EventController`: the abstract item (a hidden 0x1B7A) that a GM double clicks to edit.
pub static EVENT_CONTROLLER: ItemType = ItemType {
    full_name: EVENT_CONTROLLER_NAME,
    parent: Some(&ITEM_BASE),
    construct: None,
    on_double_click: |w, id, from| {
        if muo_content::mx::access_level(w, from) >= AccessLevel::GameMaster {
            open_properties(w, from, id);
        }
    },
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        out.write_i32(0);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        Ok(())
    },
    ..ITEM_BASE
};

/// `from.SendGump(new PropertiesGump(from, entity))`.
pub fn open_properties(w: &mut World, from: MobileId, id: ItemId) {
    if let Some(f) = w
        .resources
        .get::<muo_commands::hooks::CommandHooks>()
        .and_then(|h| h.properties_gump)
    {
        f(w, from, muo_server::ids::EntityId::Item(id));
    }
}

/// `base()` of every controller: a hidden, unmovable 0x1B7A.
pub fn new_controller(w: &mut World, ty: &'static ItemType, data: EventCtrl) -> ItemId {
    let id = ops::new_item_with_id(w, ty, 0x1B7A);
    ops::set_visible(w, id, false);
    ops::set_movable(w, id, false);
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(data);
    }
    id
}

/// `EventController.Title`.
pub fn title(w: &World, ec: ItemId) -> String {
    match ctrl(w, ec).map(|c| c.kind) {
        Some(GameKind::Ctf) => "CTF",
        Some(GameKind::Dd) => "DoubleDom",
        Some(GameKind::Kh) => "King of the Hill",
        Some(GameKind::Br) => "Bombing Run",
        None => "",
    }
    .to_owned()
}

/// `EventController.GetTeamName(teamID)`.
pub fn team_name(w: &World, ec: ItemId, team_id: usize) -> String {
    let Some(c) = ctrl(w, ec) else {
        return String::new();
    };
    match c.teams.get(team_id % c.teams.len().max(1)) {
        Some(t) => t.text_name(c.kind),
        None => String::new(),
    }
}

/// `EventController.Construct(context)`: the flags the context carries for the game.
pub fn construct(w: &World, ec: ItemId) -> Option<EventGameFlags> {
    let kind = ctrl(w, ec)?.kind;
    Some(EventGameFlags {
        free_consume: true,
        controller: ec,
        kind,
    })
}

// ---- the game's view of the duel ----------------------------------------------------------------------------------

/// `GetTeamID(mob)`: the index of the player's participant, -1 when not a living player of this game. (KotH and
/// Bombing Run also put a `BaseCreature` on team `Team - 1`.)
pub fn team_id(w: &World, id: u32, mob: MobileId) -> i32 {
    let kind = ctx(w, id).and_then(|c| c.event_game).map(|g| g.kind);
    if !muo_content::mx::player(w, mob) {
        if matches!(kind, Some(GameKind::Kh | GameKind::Br))
            && let Some(d) = muo_content::creature::cd(w, mob)
        {
            return d.team - 1;
        }
        return -1;
    }
    if duel_context_of(w, mob) != Some(id) {
        return -1;
    }
    let Some(c) = ctx(w, id) else { return -1 };
    match c.find(mob) {
        Some(p) if !p.eliminated => c.participant_of(mob).map_or(-1, |i| i as i32),
        _ => -1,
    }
}

/// `GetTeamInfo(mob)`: the team index in the controller, if any.
pub fn team_of(w: &World, id: u32, mob: MobileId) -> Option<usize> {
    let ec = ctx(w, id)?.event_game?.controller;
    let t = team_id(w, id, mob);
    let n = ctrl(w, ec)?.teams.len();
    (t >= 0 && n > 0).then(|| t as usize % n)
}

/// `GetColor(mob)`.
pub fn color_of(w: &World, id: u32, mob: MobileId) -> i32 {
    let Some(ec) = ctx(w, id).and_then(|c| c.event_game).map(|g| g.controller) else {
        return -1;
    };
    team_of(w, id, mob)
        .and_then(|t| team(w, ec, t))
        .map_or(-1, |t| t.color)
}

/// `Facet`: the arena's, else the controller's map.
pub fn facet(w: &World, id: u32) -> Option<MapId> {
    let c = ctx(w, id)?;
    if let Some(a) = c.arena {
        return arena::data(w, a).and_then(|d| d.facet);
    }
    w.item(c.event_game?.controller).and_then(|i| i.map)
}

/// `Alert(text)`: the tournament's announcer, and every player's message.
pub fn alert(w: &mut World, id: u32, text: &str) {
    let Some(c) = ctx(w, id) else { return };
    let (t, players) = (c.tournament_ctrl, c.all_players());
    if let Some(t) = t {
        tournament::alert(w, t, &[text]);
    }
    for m in players {
        muo_server::item::deps::send_message_hue(w, m, 0x35, text);
    }
}

/// `ApplyHues(participant, hue)`.
pub fn apply_hues(w: &mut World, id: u32, participant: usize, hue: i32) {
    let Some(p) = ctx(w, id).and_then(|c| c.participants.get(participant)) else {
        return;
    };
    let players: Vec<MobileId> = p.players.iter().flatten().map(|d| d.mobile).collect();
    for m in players {
        muo_server::mobile::part1::props_b::set_solid_hue_override(w, m, hue);
    }
}

/// `DelayBounce(ts, mob, corpse)` of the games: back to the team's start while still in, else outside.
/// (`delete_corpse`: KingOfTheHill deletes the corpse half a minute after.)
pub fn delay_bounce(
    w: &mut World,
    id: u32,
    ts: TimeSpan,
    mob: MobileId,
    corpse: Option<ItemId>,
    delete_corpse: bool,
) {
    timer::start(w, ts, TimeSpan::ZERO, 1, move |w| {
        let still_in = duel_context_of(w, mob)
            .and_then(|i| ctx(w, i))
            .and_then(|c| c.find(mob))
            .is_some_and(|p| !p.eliminated);
        flow::remove_aggressions(w, id, mob);
        if still_in {
            let team = team_id(w, id, mob).max(0) as usize;
            let point = ctx(w, id)
                .and_then(|c| c.arena)
                .and_then(|a| arena::data(w, a))
                .and_then(|d| d.points.get(team % d.points.len().max(1)).copied());
            if let (Some(p), Some(map), Some(ty)) =
                (point, facet(w, id), w.mobile(mob).map(|m| m.ty))
            {
                (ty.move_to_world)(w, mob, p, map);
            }
        } else {
            flow::send_outside(w, id, mob);
        }
        flow::refresh(w, mob, corpse);
        flow::debuff(w, mob);
        flow::cancel_spell(w, mob);
        muo_server::mobile::part1::props_a::set_frozen(w, mob, false);
        if delete_corpse && let Some(c) = corpse.filter(|c| !ops::is_deleted(w, *c)) {
            timer::start(w, TimeSpan::from_secs(30), TimeSpan::ZERO, 1, move |w| {
                if !ops::is_deleted(w, c)
                    && let Some(ty) = w.item(c).map(|i| i.ty)
                {
                    (ty.delete)(w, c);
                }
            });
        }
    });
}

// ---- the hooks the duel calls -----------------------------------------------------------------------------------------

/// `EventGame.CantDoAnything(mob)`.
pub fn cant_do_anything(w: &World, id: u32, mob: MobileId) -> bool {
    let Some(g) = ctx(w, id).and_then(|c| c.event_game) else {
        return false;
    };
    match g.kind {
        GameKind::Kh => super::koth::cant_do_anything(w, id, g.controller, mob),
        GameKind::Br => super::bombing::cant_do_anything(w, id, mob),
        _ => false,
    }
}

/// `EventGame.OnDeath(mob, corpse)`: always false (the duel does not eliminate the player).
pub fn on_death(w: &mut World, id: u32, mob: MobileId, corpse: Option<ItemId>) -> bool {
    let Some(g) = ctx(w, id).and_then(|c| c.event_game) else {
        return true;
    };
    match g.kind {
        GameKind::Ctf => super::ctf::on_death(w, id, g.controller, mob, corpse),
        GameKind::Dd => super::dd::on_death(w, id, g.controller, mob, corpse),
        GameKind::Kh => super::koth::on_death(w, id, g.controller, mob, corpse),
        GameKind::Br => super::bombing::on_death(w, id, g.controller, mob, corpse),
    }
    false
}

/// `EventGame.OnStart()`.
pub fn on_start(w: &mut World, id: u32) {
    let Some(g) = ctx(w, id).and_then(|c| c.event_game) else {
        return;
    };
    match g.kind {
        GameKind::Ctf => super::ctf::on_start(w, id, g.controller),
        GameKind::Dd => super::dd::on_start(w, id, g.controller),
        GameKind::Kh => super::koth::on_start(w, id, g.controller),
        GameKind::Br => super::bombing::on_start(w, id, g.controller),
    }
}

/// `EventGame.OnStop()`.
pub fn on_stop(w: &mut World, id: u32) {
    let Some(g) = ctx(w, id).and_then(|c| c.event_game) else {
        return;
    };
    match g.kind {
        GameKind::Ctf => super::ctf::on_stop(w, id, g.controller),
        GameKind::Dd => super::dd::on_stop(w, id, g.controller),
        GameKind::Kh => super::koth::on_stop(w, id, g.controller),
        GameKind::Br => super::bombing::on_stop(w, id, g.controller),
    }
    if let Some(mut r) = w.resources.get_mut::<Games>().rt.remove(&id) {
        r.finish.cancel(w);
        r.capture.cancel(w);
        r.uncapture.cancel(w);
    }
}

/// Starts the timer that ends the game after `Controller.Duration`.
pub fn start_finish_timer(w: &mut World, id: u32, ec: ItemId) {
    let duration = ctrl(w, ec).map_or(TimeSpan::from_mins(30), |c| c.duration);
    let mut old = std::mem::take(&mut rt_mut(w, id).finish);
    old.cancel(w);
    let h = timer::start(w, duration, TimeSpan::ZERO, 1, move |w| {
        finish_callback(w, id, ec)
    });
    rt_mut(w, id).finish = h;
}

// ---- the scoreboard ---------------------------------------------------------------------------------------------

/// The text that differs between the four scoreboard gumps.
pub trait BoardKind: 'static {
    const TYPE_NAME: &'static str;
    const TITLE: &'static str;
    const CAPTURES: &'static str;
    /// KotH shows "(none)" for a team without a leader.
    const NO_LEADER: Option<&'static str>;
}

pub struct CtfKind;
pub struct DdKind;
pub struct KhKind;
pub struct BrKind;

impl BoardKind for CtfKind {
    const TYPE_NAME: &'static str = "Server.Engines.ConPVP.CTFBoardGump";
    const TITLE: &'static str = "CTF Scoreboard";
    const CAPTURES: &'static str = "Captures:";
    const NO_LEADER: Option<&'static str> = None;
}
impl BoardKind for DdKind {
    const TYPE_NAME: &'static str = "Server.Engines.ConPVP.DDBoardGump";
    const TITLE: &'static str = "DD Scoreboard";
    const CAPTURES: &'static str = "Captures:";
    const NO_LEADER: Option<&'static str> = None;
}
impl BoardKind for KhKind {
    const TYPE_NAME: &'static str = "Server.Engines.ConPVP.KHBoardGump";
    const TITLE: &'static str = "King of the Hill Scoreboard";
    const CAPTURES: &'static str = "Captures:";
    const NO_LEADER: Option<&'static str> = Some("(none)");
}
impl BoardKind for BrKind {
    const TYPE_NAME: &'static str = "Server.Engines.ConPVP.BRBoardGump";
    const TITLE: &'static str = "BR Scoreboard";
    const CAPTURES: &'static str = "Points:";
    const NO_LEADER: Option<&'static str> = None;
}

struct Row {
    name: String,
    color: i32,
    ours: bool,
    score: i32,
    kills: i32,
    captures: i32,
    leader: Option<String>,
}

/// `CTFBoardGump` / `DDBoardGump` / `KHBoardGump` / `BRBoardGump` (always the whole board, never a section).
pub struct BoardGump<K: BoardKind> {
    rows: Vec<Row>,
    _kind: std::marker::PhantomData<K>,
}

/// The order of the teams in a scoreboard and of the final standings.
fn sort_teams(kind: GameKind, teams: &mut [TeamInfo]) {
    use std::cmp::Ordering;
    let key = |a: &TeamInfo, b: &TeamInfo| -> Ordering {
        match kind {
            // `Sort((a, b) => b.Score - a.Score)`
            GameKind::Ctf | GameKind::Dd => (b.score - a.score).cmp(&0),
            // `CompareTo`: score, captures, kills
            GameKind::Kh => b
                .score
                .cmp(&a.score)
                .then(b.captures.cmp(&a.captures))
                .then(b.kills.cmp(&a.kills)),
            // captures, score, kills
            GameKind::Br => b
                .captures
                .cmp(&a.captures)
                .then(b.score.cmp(&a.score))
                .then(b.kills.cmp(&a.kills)),
        }
    };
    muo_primitives::dotnet::net_sort(teams, key);
}

/// The teams playing context `id`, in the order of the participants (`TeamInfo[i % Length]`).
fn playing_teams(w: &World, id: u32, ec: ItemId) -> Vec<TeamInfo> {
    let (Some(c), Some(e)) = (ctx(w, id), ctrl(w, ec)) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for i in 0..c.participants.len() {
        let t = &e.teams[i % e.teams.len().max(1)];
        // CTF's scoreboard and standings skip a team without a flag
        if e.kind == GameKind::Ctf && t.flag.is_none() {
            continue;
        }
        out.push(t.clone());
    }
    out
}

/// `BoardGump.DisplayTo(mob, game)`.
pub fn display_board(w: &mut World, id: u32, to: MobileId) {
    let Some(g) = ctx(w, id).and_then(|c| c.event_game) else {
        return;
    };
    if w.mobile(to).and_then(|m| m.net_state).is_none() {
        return;
    }
    let ours = team_of(w, id, to);
    let kind = g.kind;
    let mut teams = playing_teams(w, id, g.controller);
    sort_teams(kind, &mut teams);
    let rows = teams
        .iter()
        .map(|t| Row {
            name: t.ranked_name(kind),
            color: t.color,
            ours: ours == Some(t.team_id),
            score: t.score,
            kills: t.kills,
            captures: t.captures,
            leader: t.leader.map(|l| super::flow::name(w, l)),
        })
        .collect();
    match kind {
        GameKind::Ctf => send_board::<CtfKind>(w, to, rows),
        GameKind::Dd => send_board::<DdKind>(w, to, rows),
        GameKind::Kh => send_board::<KhKind>(w, to, rows),
        GameKind::Br => send_board::<BrKind>(w, to, rows),
    }
}

fn send_board<K: BoardKind>(w: &mut World, to: MobileId, rows: Vec<Row>) {
    let g = BoardGump::<K> {
        rows,
        _kind: std::marker::PhantomData,
    };
    send_gump_to_mobile(w, to, Box::new(DynamicGumpObj::new(60, 60, g)), true);
}

const LABEL_COLOR32: i32 = 0xFFFFFF;
const BLACK_COLOR32: i32 = 0x000000;

fn colored(
    b: &mut DynamicGumpBuilder,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    text: &str,
    color: i32,
) {
    let t = if color == 0 {
        text.to_owned()
    } else {
        html::color_rgb(text, color)
    };
    b.add_html(x, y, width, height, &t, false, false);
}

#[allow(clippy::too_many_arguments)]
fn bordered(
    b: &mut DynamicGumpBuilder,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    text: &str,
    color: i32,
    border: i32,
) {
    colored(b, x - 1, y - 1, width, height, text, border);
    colored(b, x - 1, y + 1, width, height, text, border);
    colored(b, x + 1, y - 1, width, height, text, border);
    colored(b, x + 1, y + 1, width, height, text, border);
    colored(b, x, y, width, height, text, color);
}

impl<K: BoardKind> DynamicGumpLogic for BoardGump<K> {
    const TYPE_NAME: &'static str = K::TYPE_NAME;

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        let n = self.rows.len() as i32;
        let height = 73 + n * 75 + 28;
        b.set_no_close();
        b.add_page(0);
        b.add_background(1, 1, 398, height, 3600);
        b.add_image_tiled(16, 15, 369, height - 29, 3604);
        for i in 0..n {
            b.add_image_tiled(22, 58 + i * 75, 357, 70, 0x2430);
        }
        b.add_alpha_region(16, 15, 369, height - 29);
        b.add_image(215, -45, 0xEE40, 0);
        bordered(
            b,
            22,
            22,
            294,
            20,
            &super::gumps::center(K::TITLE),
            LABEL_COLOR32,
            BLACK_COLOR32,
        );
        b.add_image_tiled(32, 50, 264, 1, 9107);
        b.add_image_tiled(42, 52, 264, 1, 9157);

        for (i, t) in self.rows.iter().enumerate() {
            let i = i as i32;
            b.add_image(30, 70 + i * 75, 10152, 0);
            b.add_image(30, 85 + i * 75, 10151, 0);
            b.add_image(30, 100 + i * 75, 10151, 0);
            b.add_image(30, 106 + i * 75, 10154, 0);
            b.add_image(
                24,
                60 + i * 75,
                if t.ours { 9730 } else { 9727 },
                t.color - 1,
            );

            let border = if t.color == 0x455 {
                LABEL_COLOR32
            } else {
                BLACK_COLOR32
            };
            let name_color = match t.color {
                0x47E => 0xFFFFFF,
                0x4F2 => 0x3399FF,
                0x4F7 => 0x33FF33,
                0x4FC => 0xFF00FF,
                0x021 => 0xFF3333,
                0x01A => 0xFF66FF,
                0x455 => 0x333333,
                _ => LABEL_COLOR32,
            };
            bordered(
                b,
                60,
                65 + i * 75,
                250,
                20,
                &format!("{}: {}", rank(1 + i), t.name),
                name_color,
                border,
            );

            bordered(
                b,
                50 + 10,
                85 + i * 75,
                100,
                20,
                "Score:",
                0xFFC000,
                BLACK_COLOR32,
            );
            bordered(
                b,
                50 + 15,
                105 + i * 75,
                100,
                20,
                &n0(t.score),
                0xFFC000,
                BLACK_COLOR32,
            );
            bordered(
                b,
                110 + 10,
                85 + i * 75,
                100,
                20,
                "Kills:",
                0xFFC000,
                BLACK_COLOR32,
            );
            bordered(
                b,
                110 + 15,
                105 + i * 75,
                100,
                20,
                &n0(t.kills),
                0xFFC000,
                BLACK_COLOR32,
            );
            bordered(
                b,
                160 + 10,
                85 + i * 75,
                100,
                20,
                K::CAPTURES,
                0xFFC000,
                BLACK_COLOR32,
            );
            bordered(
                b,
                160 + 15,
                105 + i * 75,
                100,
                20,
                &n0(t.captures),
                0xFFC000,
                BLACK_COLOR32,
            );
            bordered(
                b,
                235 + 10,
                85 + i * 75,
                250,
                20,
                "Leader:",
                0xFFC000,
                BLACK_COLOR32,
            );
            if let Some(l) = t.leader.as_deref().or(K::NO_LEADER) {
                bordered(
                    b,
                    235 + 15,
                    105 + i * 75,
                    250,
                    20,
                    l,
                    0xFFC000,
                    BLACK_COLOR32,
                );
            }
        }
        b.add_button(314, height - 42, 247, 248, 1);
    }
}

// ---- the end of a game --------------------------------------------------------------------------------------------

/// The head of a trophy title, "Red v Blue" or "2-team": `tourney` is the tournament's type and sizes.
fn trophy_head(w: &World, id: u32, ec: ItemId) -> String {
    let mut sb = String::new();
    let kind = ctrl(w, ec).map(|c| c.kind);
    if let Some(t) = ctx(w, id)
        .and_then(|c| c.tournament_ctrl)
        .and_then(|t| tournament::get(w, t))
    {
        let parts = ctx(w, id).map_or(0, |c| c.participants.len() as i32);
        match t.tourney_type {
            TourneyType::FreeForAll => {
                sb.push_str(&format!("{}-man FFA", parts * t.players_per_participant))
            }
            TourneyType::RandomTeam => sb.push_str(&format!("{}-team", t.participants_per_match)),
            TourneyType::RedVsBlue => sb.push_str("Red v Blue"),
            // King of the Hill has no Faction branch: it falls to the sizes
            TourneyType::Faction if kind != Some(GameKind::Kh) => {
                sb.push_str(&format!("{}-team Faction", t.participants_per_match));
            }
            _ => {
                for _ in 0..t.participants_per_match {
                    if !sb.is_empty() {
                        sb.push('v');
                    }
                    sb.push_str(&t.players_per_participant.to_string());
                }
            }
        }
    }
    sb.push(' ');
    sb.push_str(&title(w, ec));
    sb
}

/// `Finish_Callback()`: trophies and cheques for every player by the standings, the scoreboard for all, everybody
/// outside the winning team eliminated, then the winner finishes the duel.
pub fn finish_callback(w: &mut World, id: u32, ec: ItemId) {
    let Some(kind) = ctrl(w, ec).map(|c| c.kind) else {
        return;
    };
    let mut teams = playing_teams(w, id, ec);
    sort_teams(kind, &mut teams);
    let head = trophy_head(w, id, ec);
    let winner = teams.first().map(|t| t.team_id);

    for (i, t) in teams.iter().enumerate() {
        let rank_ = match i {
            0 => TrophyRank::Gold,
            1 => TrophyRank::Silver,
            _ => TrophyRank::Bronze,
        };
        for (&mob, pl) in &t.players {
            let leader = t.leader == Some(mob);
            let mut sb = head.clone();
            if leader {
                sb.push_str(" Leader");
            }
            if pl.score > 0 && kind == GameKind::Br {
                // Bombing Run lists the kills and the points (its captures), not the score
                sb.push_str(&format!(
                    ": {} {}",
                    n0(pl.kills),
                    if pl.kills == 1 { "kill" } else { "kills" }
                ));
                if pl.captures > 0 {
                    sb.push_str(&format!(
                        ", {} {}",
                        n0(pl.captures),
                        if pl.captures == 1 { "point" } else { "points" }
                    ));
                }
            } else if pl.score > 0 {
                sb.push_str(&format!(
                    ": {} {}",
                    n0(pl.score),
                    if pl.score == 1 { "point" } else { "points" }
                ));
                // King of the Hill always lists the kills
                if pl.kills > 0 || kind == GameKind::Kh {
                    sb.push_str(&format!(
                        ", {} {}",
                        n0(pl.kills),
                        if pl.kills == 1 { "kill" } else { "kills" }
                    ));
                }
                if pl.captures > 0 {
                    sb.push_str(&format!(
                        ", {} {}",
                        n0(pl.captures),
                        if pl.captures == 1 {
                            "capture"
                        } else {
                            "captures"
                        }
                    ));
                }
            }
            let item = new_trophy(w, Some(sb), rank_);
            if leader {
                (muo_server::item::ITEM_BASE.set_item_id)(w, item, 4810);
            }
            let base = ops::name(w, item).unwrap_or_default();
            let lower = match kind {
                GameKind::Ctf => t.name.clone().unwrap_or_default().to_lowercase() + " team",
                GameKind::Dd | GameKind::Br => t.text_name(kind).to_lowercase() + " team",
                GameKind::Kh => t.text_name(kind).to_lowercase(),
            };
            ops::set_name(w, item, Some(format!("{base}, {lower}")));
            tournament::place_or_bank(w, mob, item);

            let cash = pl.score * 250;
            let what = if kind == GameKind::Kh {
                "game"
            } else {
                "tournament"
            };
            if cash > 0 {
                let check = muo_content::items::gold::new_bank_check(w, cash);
                tournament::place_or_bank(w, mob, check);
                muo_server::mobile::part2::messages::send_message(
                    w,
                    mob,
                    &format!(
                        "You have been awarded a {} trophy and {}gp for your participation in this {what}.",
                        tournament::rank_lower(rank_),
                        n0(cash)
                    ),
                );
            } else {
                muo_server::mobile::part2::messages::send_message(
                    w,
                    mob,
                    &format!(
                        "You have been awarded a {} trophy for your participation in this {what}.",
                        tournament::rank_lower(rank_)
                    ),
                );
            }
        }
    }

    let parts = ctx(w, id).map_or(0, |c| c.participants.len());
    for i in 0..parts {
        let players: Vec<MobileId> = ctx(w, id)
            .and_then(|c| c.participants.get(i))
            .map(|p| p.players.iter().flatten().map(|d| d.mobile).collect())
            .unwrap_or_default();
        for &m in &players {
            display_board(w, id, m);
        }
        if Some(i) == winner {
            continue;
        }
        for m in players {
            super::set_eliminated(w, id, m);
        }
    }

    if let Some(win) = winner {
        flow::finish(w, id, win);
    }
}

#[allow(dead_code)]
fn _unused(_: &dyn GenericReader, _: &dyn GenericWriter, _: &DuelContext, _: Point3D) {
    let _ = ctx_mut;
}

// ---- boards and the owner of a carried item ------------------------------------------------------------------------

/// `m_TeamInfo` of a scoreboard item: the controller and the index of the team it shows (set when a game starts).
#[derive(Clone, Copy, Debug, Default)]
pub struct BoardLink(pub Option<(ItemId, usize)>);

/// A `CTFBoard` / `DDBoard` / `BRBoard`: a 7774 that cannot be moved.
pub fn new_board(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = ops::new_item_with_id(w, ty, 7774);
    ops::set_movable(w, id, false);
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(BoardLink::default());
    }
    id
}

/// The link of a board.
pub fn board_link(w: &World, id: ItemId) -> Option<(ItemId, usize)> {
    w.item(id)?.ext.get::<BoardLink>()?.0
}

pub fn set_board_link(w: &mut World, id: ItemId, link: Option<(ItemId, usize)>) {
    if let Some(l) = w.item_mut(id).and_then(|i| i.ext.get_mut::<BoardLink>()) {
        l.0 = link;
    }
}

/// `OnDoubleClick` of the boards that show their game's scores.
pub fn board_double_click(w: &mut World, id: ItemId, from: MobileId) {
    if let Some((ec, t)) = board_link(w, id)
        && let Some(game) = team(w, ec, t).and_then(|t| t.game)
    {
        display_board(w, game, from);
    }
}

pub fn board_serialize(w: &World, id: ItemId, out: &mut dyn GenericWriter) {
    (ITEM_BASE.serialize)(w, id, out);
    out.write_i32(0);
}

pub fn board_deserialize(
    w: &mut World,
    id: ItemId,
    r: &mut dyn GenericReader,
) -> muo_io::Result<()> {
    (ITEM_BASE.deserialize)(w, id, r)?;
    let _version = r.read_i32()?;
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(BoardLink::default());
    }
    Ok(())
}

/// `FindOwner(parent)` of the carried flags and bombs.
pub fn find_owner(w: &World, parent: muo_server::ids::EntityId) -> Option<MobileId> {
    use muo_server::ids::EntityId;
    match parent {
        EntityId::Item(i) => match ops::root_parent(w, i) {
            Some(EntityId::Mobile(m)) => Some(m),
            _ => None,
        },
        EntityId::Mobile(m) => Some(m),
    }
}
