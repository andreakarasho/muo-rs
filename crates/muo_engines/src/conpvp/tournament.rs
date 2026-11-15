//! Tournaments: `Tournament`, `TournamentPyramid`, `TourneyParticipant`, `TourneyMatch`, `TournamentController`,
//! `TournamentSignupItem`, `TournamentBracketItem` and `TournamentRegistrar`.
//!
//! A `Tournament` lives in the `TournamentController` item (`TournamentData` in its ext). Every `TourneyParticipant`
//! is kept in `Tournament::parts` and addressed by its index (the pyramid, the matches, `Participants` and
//! `Undefeated` hold indexes, so "the same participant" is an equal index).
//! A match is `(level, index)` into `Tournament::levels`. The arenas of a tournament are the `ArenaController`s whose
//! `Tournament` is the controller (`Arena.Tournament`'s setter keeps `Tournament.Arenas`; the order here is the order
//! of the controllers).
//!
//! The `[CommandProperty]` surface is in `props` (flattened onto the controller, whose properties gump the "edit" entry
//! opens). A tournament's `EventController` makes the duels' event game in `start_match` (`games`).

use muo_content::gumps::{close_gump, has_gump};
use muo_io::{GenericReader, GenericWriter};
use muo_primitives::{DateTime, MessageType, Point3D, TimeSpan};
use muo_server::World;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType, ops};
use muo_server::mobile::MobileType;
use muo_server::mobile::part2::messages as mm;
use muo_server::serialization_ext::{ReadEntityExt, WriteEntityExt};
use muo_server::timer;
use muo_server::type_registry::TypeRegistry;

use super::trophy::{TrophyRank, new_trophy};
use super::{DuelContext, DuelPlayer, Ruleset, arena, flow, gumps, ladder, st_mut};

pub const CONTROLLER_NAME: &str = "Server.Engines.ConPVP.TournamentController";

const SLICE_INTERVAL: TimeSpan = TimeSpan::from_secs(12);

/// `TournamentStage`.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Stage {
    Inactive,
    Signup,
    Fighting,
}

/// `GroupingType`.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum GroupingType {
    HighVsLow,
    Nearest,
    Random,
}

impl GroupingType {
    pub(super) fn from_i32(v: i32) -> GroupingType {
        match v {
            1 => GroupingType::Nearest,
            2 => GroupingType::Random,
            _ => GroupingType::HighVsLow,
        }
    }
}

/// `TieType`.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum TieType {
    Random,
    Highest,
    Lowest,
    FullElimination,
    FullAdvancement,
}

impl TieType {
    pub(super) fn from_i32(v: i32) -> TieType {
        match v {
            1 => TieType::Highest,
            2 => TieType::Lowest,
            3 => TieType::FullElimination,
            4 => TieType::FullAdvancement,
            _ => TieType::Random,
        }
    }
}

/// `TourneyType`.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum TourneyType {
    Standard,
    FreeForAll,
    RandomTeam,
    RedVsBlue,
    Faction,
}

impl TourneyType {
    pub(super) fn from_i32(v: i32) -> TourneyType {
        match v {
            1 => TourneyType::FreeForAll,
            2 => TourneyType::RandomTeam,
            3 => TourneyType::RedVsBlue,
            4 => TourneyType::Faction,
            _ => TourneyType::Standard,
        }
    }
}

/// `TourneyParticipant`.
#[derive(Clone, Debug, Default)]
pub struct TourneyPart {
    pub players: Vec<MobileId>,
    pub log: Vec<String>,
    pub free_advances: i32,
}

impl TourneyPart {
    pub fn new(players: Vec<MobileId>) -> TourneyPart {
        TourneyPart {
            players,
            log: Vec::new(),
            free_advances: 0,
        }
    }
}

/// `TourneyMatch`.
#[derive(Clone, Debug)]
pub struct TourneyMatch {
    pub participants: Vec<usize>,
    pub winner: Option<usize>,
    /// `Context`: the duel context id while it is `Registered`.
    pub context: Option<u32>,
}

/// `PyramidLevel`.
#[derive(Clone, Debug, Default)]
pub struct Level {
    pub matches: Vec<TourneyMatch>,
    pub free_advance: Option<usize>,
}

/// `Tournament`.
#[derive(Clone, Debug)]
pub struct Tournament {
    pub event_controller: Option<ItemId>,
    pub sudden_death_rounds: i32,
    pub tourney_type: TourneyType,
    pub group_type: GroupingType,
    pub tie_type: TieType,
    pub sudden_death: TimeSpan,
    pub ruleset: Ruleset,
    pub participants_per_match: i32,
    pub players_per_participant: i32,
    pub level_requirement: i32,
    pub faction_restricted: bool,
    pub signup_period: TimeSpan,
    pub signup_start: DateTime,
    pub stage: Stage,
    /// `Pyramid.Levels`.
    pub levels: Vec<Level>,
    /// `Participants` / `Undefeated`, as indexes into `parts`.
    pub participants: Vec<usize>,
    pub undefeated: Vec<usize>,
    /// Every `TourneyParticipant` made.
    pub parts: Vec<TourneyPart>,
}

impl Tournament {
    /// `new Tournament()` (the ruleset needs the layout).
    pub fn new(w: &World) -> Tournament {
        let mut ruleset = Ruleset::new(super::ruleset::root(w));
        ruleset.apply_default(0);
        Tournament {
            event_controller: None,
            sudden_death_rounds: 0,
            tourney_type: TourneyType::Standard,
            group_type: GroupingType::HighVsLow,
            tie_type: TieType::Random,
            sudden_death: TimeSpan::ZERO,
            ruleset,
            participants_per_match: 2,
            players_per_participant: 1,
            level_requirement: 0,
            faction_restricted: false,
            signup_period: TimeSpan::from_mins(10),
            signup_start: DateTime::MIN,
            stage: Stage::Inactive,
            levels: Vec::new(),
            participants: Vec::new(),
            undefeated: Vec::new(),
            parts: Vec::new(),
        }
    }

    /// `IsNotoRestricted`.
    pub fn is_noto_restricted(&self) -> bool {
        self.tourney_type != TourneyType::Standard
    }

    /// `IsFactionRestricted`.
    pub fn is_faction_restricted(&self) -> bool {
        self.faction_restricted || self.tourney_type == TourneyType::Faction
    }

    /// `HasParticipant(mob)`.
    pub fn has_participant(&self, m: MobileId) -> bool {
        self.participants
            .iter()
            .any(|p| self.parts[*p].players.contains(&m))
    }

    /// `ParticipantsPerMatch`'s setter.
    pub fn set_participants_per_match(&mut self, v: i32) {
        self.participants_per_match = v.clamp(2, 10);
    }

    /// `PlayersPerParticipant`'s setter.
    pub fn set_players_per_participant(&mut self, v: i32) {
        self.players_per_participant = v.clamp(1, 10);
    }

    fn add_part(&mut self, players: Vec<MobileId>) -> usize {
        self.parts.push(TourneyPart::new(players));
        self.parts.len() - 1
    }

    /// `part.NameList`.
    pub fn name_list(&self, w: &World, part: usize) -> String {
        let players = &self.parts[part].players;
        let mut sb = String::new();
        for (i, mob) in players.iter().enumerate() {
            if !sb.is_empty() {
                if players.len() == 2 {
                    sb.push_str(" and ");
                } else if i + 1 < players.len() {
                    sb.push_str(", ");
                } else {
                    sb.push_str(", and ");
                }
            }
            sb.push_str(&flow::name(w, *mob));
        }
        if sb.is_empty() {
            "Empty".to_owned()
        } else {
            sb
        }
    }
}

// ---- storage -------------------------------------------------------------------------------------------------------

/// The ext of a `TournamentController`.
pub struct TournamentData(pub Tournament);

/// `TournamentController.m_Instances`.
#[derive(Default)]
pub struct Controllers(pub Vec<ItemId>);

/// The tournament of a controller.
pub fn get(w: &World, ctrl: ItemId) -> Option<&Tournament> {
    w.item(ctrl)?.ext.get::<TournamentData>().map(|d| &d.0)
}

pub fn get_mut(w: &mut World, ctrl: ItemId) -> Option<&mut Tournament> {
    w.item_mut(ctrl)?
        .ext
        .get_mut::<TournamentData>()
        .map(|d| &mut d.0)
}

fn controllers(w: &mut World) -> &mut Controllers {
    w.resources.get_mut::<Controllers>()
}

/// `TournamentController.IsActive`.
pub fn is_active(w: &World) -> bool {
    w.resources.get::<Controllers>().is_some_and(|c| {
        c.0.iter().any(|id| {
            w.item(*id).is_some_and(|i| !i.deleted())
                && get(w, *id).is_some_and(|t| t.stage != Stage::Inactive)
        })
    })
}

/// `Tournament.Arenas`.
pub fn arenas_of(w: &World, ctrl: ItemId) -> Vec<ItemId> {
    let Some(a) = w.resources.get::<arena::Arenas>() else {
        return Vec::new();
    };
    a.controllers
        .iter()
        .copied()
        .filter(|c| arena::data(w, *c).is_some_and(|d| d.tournament == Some(ctrl)))
        .collect()
}

// ---- alerts --------------------------------------------------------------------------------------------------------

/// `Alert(params string[])`: every arena.
pub fn alert(w: &mut World, ctrl: ItemId, alerts: &[&str]) {
    for a in arenas_of(w, ctrl) {
        alert_arena(w, Some(a), alerts);
    }
}

/// `Alert(arena, params string[])`: the arena's announcer says each line, half a second apart.
pub fn alert_arena(w: &mut World, arena_id: Option<ItemId>, alerts: &[&str]) {
    let Some(a) = arena_id else { return };
    if arena::data(w, a).and_then(|d| d.announcer).is_none() {
        return;
    }
    let alerts: Vec<String> = alerts.iter().map(|s| (*s).to_owned()).collect();
    let mut count = 0;
    let half = TimeSpan::from_millis(500);
    timer::start(w, half, half, alerts.len() as i32, move |w| {
        let Some(ann) = arena::data(w, a).and_then(|d| d.announcer) else {
            return;
        };
        if let Some(text) = alerts.get(count) {
            mm::public_overhead_message(
                w,
                ann,
                MessageType::REGULAR,
                0x35,
                false,
                text,
                true,
                muo_server::mobile::AccessLevel::Player,
            );
        }
        count += 1;
    });
}

// ---- pyramid -------------------------------------------------------------------------------------------------------

fn total_ladder_xp(w: &mut World, t: &Tournament, part: usize) -> i32 {
    let Some(l) = ladder::instance(w) else {
        return 0;
    };
    let mut total = 0;
    for m in t.parts[part].players.clone() {
        if let Some(e) = ladder::find(w, l, m) {
            total += e.experience;
        }
    }
    total
}

/// `Pyramid.AddLevel(partsPerMatch, participants, groupType, tourneyType)`.
pub fn add_level(w: &mut World, ctrl: ItemId, parts_per_match: i32, participants: &[usize]) {
    let Some(mut t) = get(w, ctrl).cloned() else {
        return;
    };
    let (group_type, tourney_type) = (t.group_type, t.tourney_type);
    let mut copy: Vec<usize> = participants.to_vec();

    if matches!(group_type, GroupingType::Nearest | GroupingType::HighVsLow) {
        // `copy.Sort()`: by descending ladder experience.
        let keys: Vec<i32> = copy.iter().map(|p| total_ladder_xp(w, &t, *p)).collect();
        let mut pairs: Vec<(usize, i32)> = copy.iter().copied().zip(keys).collect();
        muo_primitives::dotnet::net_sort(&mut pairs, |a, b| b.1.cmp(&a.1));
        copy = pairs.into_iter().map(|p| p.0).collect();
    }

    let mut level = Level::default();
    let ppm = parts_per_match;

    match tourney_type {
        TourneyType::RedVsBlue => {
            let parts: Vec<usize> = (0..2).map(|_| t.add_part(Vec::new())).collect();
            for &c in &copy {
                for mob in t.parts[c].players.clone() {
                    if muo_server::mobile::base::murderer(w, mob) {
                        t.parts[parts[0]].players.push(mob);
                    } else {
                        t.parts[parts[1]].players.push(mob);
                    }
                }
            }
            level.matches.push(new_match(w, &mut t, parts));
        }
        TourneyType::Faction => {
            let parts: Vec<usize> = (0..ppm.max(0)).map(|_| t.add_part(Vec::new())).collect();
            for (i, &c) in copy.iter().enumerate() {
                for mob in t.parts[c].players.clone() {
                    let mut index: i32 = -1;
                    if ppm == 4 {
                        if let Some(f) = crate::factions::find(w, mob) {
                            index = crate::factions::tables::FACTION_DEFS[f].sort;
                        }
                    } else if ppm == 2 {
                        if crate::ethics::EthicKind::Evil.is_eligible(w, mob) {
                            index = 0;
                        } else if crate::ethics::EthicKind::Hero.is_eligible(w, mob) {
                            index = 1;
                        }
                    }
                    if index < 0 || index >= ppm {
                        index = (i as i32) % ppm;
                    }
                    t.parts[parts[index as usize]].players.push(mob);
                }
            }
            level.matches.push(new_match(w, &mut t, parts));
        }
        TourneyType::RandomTeam => {
            let parts: Vec<usize> = (0..ppm.max(0)).map(|_| t.add_part(Vec::new())).collect();
            for (i, &c) in copy.iter().enumerate() {
                let add = t.parts[c].players.clone();
                t.parts[parts[i % parts.len()]].players.extend(add);
            }
            level.matches.push(new_match(w, &mut t, parts));
        }
        TourneyType::FreeForAll => {
            level.matches.push(new_match(w, &mut t, copy.clone()));
        }
        TourneyType::Standard => {
            if ppm >= 2 && participants.len() as i32 % ppm == 1 {
                let low = participants
                    .iter()
                    .map(|p| t.parts[*p].free_advances)
                    .min()
                    .unwrap_or(i32::MAX);
                let mut to_advance: Vec<usize> = participants
                    .iter()
                    .copied()
                    .filter(|p| t.parts[*p].free_advances == low)
                    .collect();
                if to_advance.is_empty() {
                    to_advance = copy.clone(); // sanity
                }
                let random = *muo_primitives::utility::random_element(&mut w.rng, &to_advance)
                    .expect("participants");
                t.parts[random]
                    .log
                    .push("Advanced automatically due to an odd number of challengers.".to_owned());
                level.free_advance = Some(random);
                t.parts[random].free_advances += 1;
                if let Some(pos) = copy.iter().position(|p| *p == random) {
                    copy.remove(pos);
                }
            }

            while ppm > 0 && copy.len() as i32 >= ppm {
                let mut this_match = Vec::new();
                for i in 0..ppm {
                    let idx = match group_type {
                        GroupingType::HighVsLow => {
                            (i * (copy.len() as i32 - 1) / (ppm - 1)) as usize
                        }
                        GroupingType::Nearest => 0,
                        GroupingType::Random => {
                            muo_primitives::utility::random(&mut w.rng, copy.len() as i32) as usize
                        }
                    };
                    this_match.push(copy[idx]);
                    copy.remove(idx);
                }
                level.matches.push(new_match(w, &mut t, this_match));
            }

            if copy.len() > 1 {
                level.matches.push(new_match(w, &mut t, copy.clone()));
            }
        }
    }

    t.levels.push(level);
    if let Some(cur) = get_mut(w, ctrl) {
        // Only the pieces the pyramid touches are written back, the rest of the tournament is as it was.
        cur.parts = t.parts;
        cur.levels = t.levels;
    }
}

/// `new TourneyMatch(participants)`: logs the pairing on each participant.
fn new_match(w: &World, t: &mut Tournament, participants: Vec<usize>) -> TourneyMatch {
    for i in 0..participants.len() {
        let part = participants[i];
        let mut sb = String::from("Matched in a duel against ");
        if participants.len() > 2 {
            sb.push_str(&format!(
                "{} other {}: ",
                participants.len() - 1,
                if t.parts[part].players.len() == 1 {
                    "players"
                } else {
                    "teams"
                }
            ));
        }
        let mut has_appended = false;
        for (j, &participant) in participants.iter().enumerate() {
            if i == j {
                continue;
            }
            if has_appended {
                sb.push_str(", ");
            }
            sb.push_str(&t.name_list(w, participant));
            has_appended = true;
        }
        sb.push('.');
        t.parts[part].log.push(sb);
    }
    TourneyMatch {
        participants,
        winner: None,
        context: None,
    }
}

// ---- match ---------------------------------------------------------------------------------------------------------

/// `TourneyMatch.InProgress`: `Context?.Registered == true`.
pub fn match_in_progress(w: &World, m: &TourneyMatch) -> bool {
    m.context
        .is_some_and(|c| super::ctx(w, c).is_some_and(|c| c.registered))
}

/// `TourneyMatch.Start(arena, tourney)`.
pub fn start_match(w: &mut World, ctrl: ItemId, level: usize, index: usize, arena_id: ItemId) {
    let Some(t) = get(w, ctrl).cloned() else {
        return;
    };
    let Some(m) = t
        .levels
        .get(level)
        .and_then(|l| l.matches.get(index))
        .cloned()
    else {
        return;
    };
    let Some(&first_part) = m.participants.first() else {
        return;
    };
    let Some(&initiator) = t.parts[first_part].players.first() else {
        return;
    };

    let mut ruleset = Ruleset::new(t.ruleset.layout.clone());
    ruleset.apply_default(0);
    ruleset.options = t.ruleset.options.clone(); // SetAll(false) then Or(tourney options)
    let mut dc = DuelContext::empty(initiator, ruleset);
    dc.tournament = Some(t.is_noto_restricted());
    dc.tournament_ctrl = Some(ctrl);
    dc.tmatch = Some((level, index));
    dc.override_arena = Some(arena_id);
    let id = st_mut(w).add(dc);

    for (i, &tp) in m.participants.iter().enumerate() {
        let mut p = st_mut(w).new_participant(t.parts[tp].players.len());
        p.tourney_part = Some(tp);
        if let Some(c) = super::ctx_mut(w, id) {
            c.participants.push(p);
        }
        for &mob in &t.parts[tp].players {
            super::participant_add(w, id, i, mob);
        }
        if let Some(c) = super::ctx_mut(w, id)
            && let Some(p) = c.participants.get_mut(i)
        {
            for pl in p.players.iter_mut().flatten() {
                pl.ready = true;
            }
        }
    }

    if let Some(ec) = t.event_controller
        && let Some(g) = super::games::construct(w, ec)
        && let Some(c) = super::ctx_mut(w, id)
    {
        c.event_game = Some(g); // `dc.m_EventGame = tourney.EventController.Construct(dc)`
    }

    if t.sudden_death > TimeSpan::ZERO
        && (t.sudden_death_rounds == 0 || t.levels.len() as i32 <= t.sudden_death_rounds)
    {
        flow::start_sudden_death(w, id, t.sudden_death);
    }

    flow::send_ready_gump_count(w, id, 0);

    if super::ctx(w, id).is_some_and(|c| c.started_begin_countdown) {
        if let Some(tm) = get_mut(w, ctrl)
            && let Some(mm_) = tm
                .levels
                .get_mut(level)
                .and_then(|l| l.matches.get_mut(index))
        {
            mm_.context = Some(id);
        }
        for &tp in &m.participants {
            for &mob in &t.parts[tp].players {
                for view in muo_content::mx::mobiles_in_range(w, mob, 18) {
                    if !muo_server::mobile::base::can_see_mobile(w, mob, view) {
                        let ns = w.mobile(mob).and_then(|x| x.net_state);
                        let serial = w.mobile(view).map(|x| x.serial);
                        if let (Some(ns), Some(serial)) = (ns, serial) {
                            muo_server::packets::entity::send_remove_entity(w, ns, serial);
                        }
                    }
                }
                mm::local_overhead_message(
                    w,
                    mob,
                    MessageType::EMOTE,
                    0x3B2,
                    false,
                    "* Your mind focuses intently on the fight and all other distractions fade away *",
                );
            }
        }
    } else {
        flow::unregister(w, id, false);
        flow::stop_countdown(w, id);
    }
}

// ---- results -------------------------------------------------------------------------------------------------------

/// `winner.TourneyPart.WonMatch(match)` / `LostMatch`.
pub fn won_match(w: &mut World, ctrl: ItemId, part: usize) {
    if let Some(t) = get_mut(w, ctrl) {
        t.parts[part].log.push("Match won.".to_owned());
    }
}

pub fn lost_match(w: &mut World, ctrl: ItemId, part: usize) {
    if let Some(t) = get_mut(w, ctrl) {
        t.parts[part].log.push("Match lost.".to_owned());
    }
}

/// `m_Match.Winner = winner`.
pub fn set_winner(w: &mut World, ctrl: ItemId, m: (usize, usize), winner: usize) {
    if let Some(t) = get_mut(w, ctrl)
        && let Some(x) = t.levels.get_mut(m.0).and_then(|l| l.matches.get_mut(m.1))
    {
        x.winner = Some(winner);
    }
}

/// The "advanced" line of a tie: everyone else in `remaining` is no longer undefeated.
fn advanced_line(
    w: &World,
    t: &Tournament,
    remaining: &[usize],
    advanced: usize,
    undefeated: &mut Vec<usize>,
    sb: &mut String,
) {
    for r in remaining {
        if *r != advanced {
            undefeated.retain(|p| p != r);
        }
    }
    sb.push_str(&format!(
        "In accordance with the rules, {} {} advanced.",
        t.name_list(w, advanced),
        if t.parts[advanced].players.len() == 1 {
            "is"
        } else {
            "are"
        }
    ));
}

/// `Tournament.HandleTie(arena, match, remaining)`.
pub fn handle_tie(
    w: &mut World,
    ctrl: ItemId,
    arena_id: Option<ItemId>,
    m: (usize, usize),
    remaining: &[usize],
) {
    if remaining.len() == 1 {
        handle_won(w, ctrl, arena_id, m, remaining[0]);
    }
    if remaining.len() < 2 {
        return;
    }
    let Some(t) = get(w, ctrl).cloned() else {
        return;
    };
    let Some(mt) = t.levels.get(m.0).and_then(|l| l.matches.get(m.1)).cloned() else {
        return;
    };

    let mut sb = String::from("The match has ended in a tie ");
    sb.push_str(if remaining.len() == 2 {
        "between "
    } else {
        "among "
    });
    sb.push_str(&remaining.len().to_string());
    sb.push_str(if t.parts[remaining[0]].players.len() == 1 {
        " players: "
    } else {
        " teams: "
    });

    let mut undefeated = t.undefeated.clone();
    let mut has_appended = false;
    for &part in &mt.participants {
        if remaining.contains(&part) {
            if has_appended {
                sb.push_str(", ");
            }
            sb.push_str(&t.name_list(w, part));
            has_appended = true;
        } else {
            undefeated.retain(|p| *p != part);
        }
    }
    sb.push_str(". ");

    let whole = if remaining.len() == 2 { "both" } else { "all" };
    let mut tie_type = t.tie_type;
    if tie_type == TieType::FullElimination && remaining.len() >= undefeated.len() {
        tie_type = TieType::FullAdvancement;
    }

    match tie_type {
        TieType::FullAdvancement => {
            sb.push_str(&format!(
                "In accordance with the rules, {whole} parties are advanced."
            ));
        }
        TieType::FullElimination => {
            for r in remaining {
                undefeated.retain(|p| p != r);
            }
            sb.push_str(&format!(
                "In accordance with the rules, {whole} parties are eliminated."
            ));
        }
        TieType::Random => {
            let advanced =
                *muo_primitives::utility::random_element(&mut w.rng, remaining).expect("remaining");
            advanced_line(w, &t, remaining, advanced, &mut undefeated, &mut sb);
        }
        TieType::Highest | TieType::Lowest => {
            let mut advanced: Option<(usize, i32)> = None;
            for &part in remaining {
                let xp = total_ladder_xp(w, &t, part);
                let better = match advanced {
                    None => true,
                    Some((_, a)) => {
                        if tie_type == TieType::Highest {
                            xp > a
                        } else {
                            xp < a
                        }
                    }
                };
                if better {
                    advanced = Some((part, xp));
                }
            }
            if let Some((a, _)) = advanced {
                advanced_line(w, &t, remaining, a, &mut undefeated, &mut sb);
            }
        }
    }

    if let Some(t) = get_mut(w, ctrl) {
        t.undefeated = undefeated;
    }
    alert_arena(w, arena_id, &[&sb]);
}

/// `Tournament.HandleWon(arena, match, winner)`.
pub fn handle_won(
    w: &mut World,
    ctrl: ItemId,
    arena_id: Option<ItemId>,
    m: (usize, usize),
    winner: usize,
) {
    let Some(t) = get(w, ctrl).cloned() else {
        return;
    };
    let Some(mt) = t.levels.get(m.0).and_then(|l| l.matches.get(m.1)).cloned() else {
        return;
    };

    let mut sb = String::from("The match is complete. ");
    sb.push_str(&t.name_list(w, winner));
    sb.push_str(if t.parts[winner].players.len() > 1 {
        " have bested "
    } else {
        " has bested "
    });
    if mt.participants.len() > 2 {
        sb.push_str(&format!(
            "{} other {}: ",
            mt.participants.len() - 1,
            if t.parts[winner].players.len() == 1 {
                "players"
            } else {
                "teams"
            }
        ));
    }
    let mut has_appended = false;
    let mut undefeated = t.undefeated.clone();
    for &part in &mt.participants {
        if part == winner {
            continue;
        }
        undefeated.retain(|p| *p != part);
        if has_appended {
            sb.push_str(", ");
        }
        sb.push_str(&t.name_list(w, part));
        has_appended = true;
    }
    sb.push('.');

    if let Some(t) = get_mut(w, ctrl) {
        t.undefeated = undefeated;
    }
    if t.tourney_type == TourneyType::Standard {
        alert_arena(w, arena_id, &[&sb]);
    }
}

/// `Tournament.OnEliminated(player)`: `part` is the duel participant (index) in context `id`.
pub fn on_eliminated(w: &mut World, id: u32, part: usize) {
    let Some(c) = super::ctx(w, id) else { return };
    let (Some(ctrl), Some(p)) = (c.tournament_ctrl, c.participants.get(part)) else {
        return;
    };
    if !p.eliminated() {
        return;
    }
    let tp = p.tourney_part;
    let rem = c.participants.iter().filter(|p| !p.eliminated()).count();
    let Some(t) = get(w, ctrl) else { return };
    if t.tourney_type != TourneyType::FreeForAll {
        return;
    }
    let Some(tp) = tp else { return };
    let players = t.parts[tp].players.clone();
    let cash = compute_cash_award(t);
    if rem == 1 {
        give_awards_to(w, ctrl, &players, TrophyRank::Silver, cash / 2);
    } else if rem == 2 {
        give_awards_to(w, ctrl, &players, TrophyRank::Bronze, cash / 4);
    }
}

fn compute_cash_award(t: &Tournament) -> i32 {
    t.participants.len() as i32 * t.players_per_participant * 2500
}

/// `GiveAwards()`.
fn give_awards(w: &mut World, ctrl: ItemId) {
    let Some(t) = get(w, ctrl).cloned() else {
        return;
    };
    match t.tourney_type {
        TourneyType::FreeForAll => {
            let Some(top) = t.levels.last() else { return };
            if top.free_advance.is_some() || top.matches.len() != 1 {
                return;
            }
            if let Some(winner) = top.matches[0].winner {
                give_awards_to(
                    w,
                    ctrl,
                    &t.parts[winner].players,
                    TrophyRank::Gold,
                    compute_cash_award(&t),
                );
            }
        }
        TourneyType::Standard => {
            if t.levels.len() < 2 {
                return;
            }
            let top = &t.levels[t.levels.len() - 1];
            if top.free_advance.is_some() || top.matches.len() != 1 {
                return;
            }
            let cash = compute_cash_award(&t);
            let mt = &top.matches[0];
            let winner = mt.winner;
            for &part in &mt.participants {
                if Some(part) == winner {
                    give_awards_to(w, ctrl, &t.parts[part].players, TrophyRank::Gold, cash);
                } else {
                    give_awards_to(
                        w,
                        ctrl,
                        &t.parts[part].players,
                        TrophyRank::Silver,
                        cash / 2,
                    );
                }
            }
            let next = &t.levels[t.levels.len() - 2];
            if next.matches.len() > 2 {
                return;
            }
            for mt in &next.matches {
                for &part in &mt.participants {
                    if Some(part) != mt.winner {
                        give_awards_to(
                            w,
                            ctrl,
                            &t.parts[part].players,
                            TrophyRank::Bronze,
                            cash / 4,
                        );
                    }
                }
            }
        }
        _ => {}
    }
}

pub(super) fn rank_lower(rank: TrophyRank) -> &'static str {
    match rank {
        TrophyRank::Gold => "gold",
        TrophyRank::Silver => "silver",
        TrophyRank::Bronze => "bronze",
    }
}

/// `GiveAwards(players, rank, cash)`.
fn give_awards_to(
    w: &mut World,
    ctrl: ItemId,
    players: &[MobileId],
    rank: TrophyRank,
    mut cash: i32,
) {
    if players.is_empty() {
        return;
    }
    let Some(t) = get(w, ctrl) else { return };
    if players.len() > 1 {
        cash /= players.len() as i32 - 1;
    }
    cash += 500;
    cash /= 1000;
    cash *= 1000;

    let mut sb = String::new();
    match t.tourney_type {
        TourneyType::FreeForAll => {
            sb.push_str(&format!(
                "{}-man FFA",
                t.participants.len() as i32 * t.players_per_participant
            ));
        }
        TourneyType::RandomTeam => sb.push_str(&format!("{}-Team", t.participants_per_match)),
        TourneyType::Faction => sb.push_str(&format!("{}-Team Faction", t.participants_per_match)),
        TourneyType::RedVsBlue => sb.push_str("Red v Blue"),
        TourneyType::Standard => {
            for _ in 0..t.participants_per_match {
                if !sb.is_empty() {
                    sb.push('v');
                }
                sb.push_str(&t.players_per_participant.to_string());
            }
        }
    }
    if let Some(ec) = t.event_controller {
        sb.push(' ');
        sb.push_str(&event_title(w, ec));
    }
    sb.push_str(" Champion");
    let title = sb;

    for &mob in players {
        if muo_content::mx::deleted(w, mob) {
            continue;
        }
        let item = new_trophy(w, Some(title.clone()), rank);
        place_or_bank(w, mob, item);
        if cash > 0 {
            let check = muo_content::items::gold::new_bank_check(w, cash);
            place_or_bank(w, mob, check);
            mm::send_message(
                w,
                mob,
                &format!(
                    "You have been awarded a {} trophy and {}gp for your participation in this tournament.",
                    rank_lower(rank),
                    muo_content::items::gold::n0(cash as i64)
                ),
            );
        } else {
            mm::send_message(
                w,
                mob,
                &format!(
                    "You have been awarded a {} trophy for your participation in this tournament.",
                    rank_lower(rank)
                ),
            );
        }
    }
}

/// `if (!mob.PlaceInBackpack(item)) mob.BankBox.DropItem(item)`.
pub(super) fn place_or_bank(w: &mut World, mob: MobileId, item: ItemId) {
    if muo_server::mobile::part2::items::place_in_backpack(w, mob, item) {
        return;
    }
    if let Some(bank) = muo_server::mobile::part1::props_b::bank_box(w, mob)
        && let Some(ty) = w.item(bank).map(|i| i.ty)
    {
        (ty.drop_item)(w, bank, item);
    }
}

// ---- event controllers ---------------------------------------------------------------------------------------------

/// `EventController.Title`.
pub(super) fn event_title(w: &World, ec: ItemId) -> String {
    super::games::title(w, ec)
}

/// `EventController.GetTeamName(teamID)`.
pub(super) fn event_team_name(w: &World, ec: ItemId, team: usize) -> String {
    super::games::team_name(w, ec, team)
}

// ---- slice ---------------------------------------------------------------------------------------------------------

/// A participant is "bad" when one of its players cannot play.
fn is_bad(w: &World, t: &Tournament, part: usize) -> bool {
    t.parts[part].players.iter().any(|&check| {
        let m = w.mobile(check);
        muo_content::mx::deleted(w, check)
            || m.is_none_or(|m| m.map.is_none() || m.map == Some(muo_server::map::MapId::INTERNAL))
            || !muo_content::mx::alive(w, check)
            || crate::factions::items::exists_on(w, check)
            || w.mobile(check).is_some_and(|m| {
                muo_server::region::is_part_of_type(w, m.region, &muo_content::jail::JAIL_REGION)
            })
    })
}

fn disqualify(w: &mut World, t: &Tournament, part: usize) {
    for &m in &t.parts[part].players {
        mm::send_message(w, m, "You have been disqualified from the tournament.");
    }
}

/// The "has completed" alert for the last undefeated participant. `event_bang`: the first call site has a "!" on the
/// event team line, the one after a disqualification has none.
fn announce_winner(w: &mut World, ctrl: ItemId, winner: usize, event_bang: bool) {
    let Some(t) = get(w, ctrl).cloned() else {
        return;
    };
    let multi = t.parts[winner].players.len() > 1;
    // `Pyramid.Levels[0].Matches[0].Participants.IndexOf(winner)`; any failure below just skips the announcement.
    let first = t.levels.first().and_then(|l| l.matches.first());
    let idx = || {
        first.map(|m| {
            m.participants
                .iter()
                .position(|p| *p == winner)
                .map_or(-1, |i| i as i32)
        })
    };
    let line = if let Some(ec) = t.event_controller {
        let Some(i) = idx() else { return };
        // `GetTeamName(-1)` is invalid: nothing is said.
        if i < 0 {
            return;
        }
        format!(
            "Team {} has won{}",
            event_team_name(w, ec, i as usize),
            if event_bang { "!" } else { "" }
        )
    } else if t.tourney_type == TourneyType::RandomTeam {
        let Some(i) = idx() else { return };
        format!("Team {} has won!", i + 1)
    } else if t.tourney_type == TourneyType::Faction {
        let Some(i) = idx() else { return };
        if t.participants_per_match == 4 {
            let name = match i {
                0 => "Minax",
                1 => "Council of Mages",
                2 => "True Britannians",
                3 => "Shadowlords",
                _ => "(null)",
            };
            format!("The {name} team has won!")
        } else if t.participants_per_match == 2 {
            format!("The {} team has won!", if i == 0 { "Evil" } else { "Hero" })
        } else {
            format!("Team {} has won!", i + 1)
        }
    } else if t.tourney_type == TourneyType::RedVsBlue {
        let Some(i) = idx() else { return };
        format!("Team {} has won!", if i == 0 { "Red" } else { "Blue" })
    } else {
        format!(
            "{} {} the champion{}.",
            t.name_list(w, winner),
            if multi { "are" } else { "is" },
            if multi { "s" } else { "" }
        )
    };
    alert(w, ctrl, &["The tournament has completed!", &line]);
}

/// `Slice()`.
pub fn slice(w: &mut World, ctrl: ItemId) {
    let Some(t) = get(w, ctrl).cloned() else {
        return;
    };
    match t.stage {
        Stage::Signup => slice_signup(w, ctrl, t),
        Stage::Fighting => slice_fighting(w, ctrl, t),
        Stage::Inactive => {}
    }
}

fn slice_signup(w: &mut World, ctrl: ItemId, t: Tournament) {
    let until = t.signup_start + t.signup_period - w.now;
    if until <= TimeSpan::ZERO {
        let mut participants = t.participants.clone();
        for i in (0..participants.len()).rev() {
            if is_bad(w, &t, participants[i]) {
                disqualify(w, &t, participants[i]);
                participants.remove(i);
            }
        }
        if let Some(tm) = get_mut(w, ctrl) {
            tm.participants = participants.clone();
        }
        if participants.len() >= 2 {
            if let Some(tm) = get_mut(w, ctrl) {
                tm.stage = Stage::Fighting;
                tm.undefeated.clear();
                tm.levels.clear();
            }
            add_level(w, ctrl, t.participants_per_match, &participants);
            if let Some(tm) = get_mut(w, ctrl) {
                let level = tm.levels[0].clone();
                if let Some(fa) = level.free_advance {
                    tm.undefeated.push(fa);
                }
                for m in &level.matches {
                    tm.undefeated.extend(m.participants.iter().copied());
                }
            }
            alert(
                w,
                ctrl,
                &["Hear ye! Hear ye!", "The tournament will begin shortly."],
            );
        } else {
            alert(w, ctrl, &["Is this all?", "Pitiful. Tournament cancelled."]);
            if let Some(tm) = get_mut(w, ctrl) {
                tm.stage = Stage::Inactive;
            }
        }
    } else if (until.total_seconds() - 60.0).abs() < SLICE_INTERVAL.total_seconds() / 2.0 {
        alert(
            w,
            ctrl,
            &[
                "Last call!",
                "If you wish to enter the tournament, sign up with the registrar now.",
            ],
        );
    } else if (until.total_seconds() - 300.0).abs() < SLICE_INTERVAL.total_seconds() / 2.0 {
        alert(
            w,
            ctrl,
            &[
                "The tournament will begin in 5 minutes.",
                "Sign up now before it's too late.",
            ],
        );
    }
}

fn slice_fighting(w: &mut World, ctrl: ItemId, t: Tournament) {
    if t.undefeated.len() == 1 {
        announce_winner(w, ctrl, t.undefeated[0], true);
        give_awards(w, ctrl);
        if let Some(tm) = get_mut(w, ctrl) {
            tm.stage = Stage::Inactive;
            tm.undefeated.clear();
        }
        return;
    }
    let Some(active) = t.levels.last().cloned() else {
        return;
    };
    let level = t.levels.len() - 1;
    let mut still_going = false;
    for (i, m) in active.matches.iter().enumerate() {
        if m.winner.is_none() {
            still_going = true;
            if !match_in_progress(w, m) {
                for a in arenas_of(w, ctrl) {
                    if !arena::is_occupied(w, a) {
                        start_match(w, ctrl, level, i, a);
                        break;
                    }
                }
            }
        }
    }
    if still_going {
        return;
    }

    let Some(cur) = get(w, ctrl).cloned() else {
        return;
    };
    let mut undefeated = cur.undefeated.clone();
    let mut i = undefeated.len();
    while i > 0 {
        i -= 1;
        let part = undefeated[i];
        if !is_bad(w, &cur, part) {
            continue;
        }
        disqualify(w, &cur, part);
        undefeated.remove(i);
        if undefeated.len() == 1 {
            let winner = undefeated[0];
            if let Some(tm) = get_mut(w, ctrl) {
                tm.undefeated = undefeated.clone();
            }
            announce_winner(w, ctrl, winner, false);
            give_awards(w, ctrl);
            if let Some(tm) = get_mut(w, ctrl) {
                tm.stage = Stage::Inactive;
                tm.undefeated.clear();
            }
            // `Undefeated.Clear(); break;`, then `Undefeated.Count > 1` is false.
            undefeated.clear();
            break;
        }
    }
    if get(w, ctrl).is_some_and(|tm| tm.stage == Stage::Fighting)
        && let Some(tm) = get_mut(w, ctrl)
    {
        tm.undefeated = undefeated.clone();
    }
    if undefeated.len() > 1 {
        add_level(w, ctrl, cur.participants_per_match, &undefeated);
    }
}

// ---- controller item -----------------------------------------------------------------------------------------------

fn start_slice(w: &mut World, ctrl: ItemId) {
    timer::start(w, SLICE_INTERVAL, SLICE_INTERVAL, 0, move |w| {
        if get(w, ctrl).is_some() {
            slice(w, ctrl);
        }
    });
}

fn write_tournament(w: &World, t: &Tournament, out: &mut dyn GenericWriter) {
    out.write_encoded_int(5);
    out.write_bool(t.faction_restricted);
    out.write_item(w, t.event_controller);
    out.write_encoded_int(t.sudden_death_rounds);
    out.write_encoded_int(t.tourney_type as i32);
    out.write_encoded_int(t.group_type as i32);
    out.write_encoded_int(t.tie_type as i32);
    out.write_time_span_ticks(t.sudden_death.ticks());
    out.write_encoded_int(t.participants_per_match);
    out.write_encoded_int(t.players_per_participant);
    out.write_time_span_ticks(t.signup_period.ticks());
}

/// `new Tournament(reader)`. (Versions 1 and up read the sudden death bytes into `SignupPeriod`, then
/// reads `SignupPeriod` again: `SuddenDeath` stays zero. Kept for save compatibility.)
fn read_tournament(w: &World, r: &mut dyn GenericReader) -> muo_io::Result<Tournament> {
    let version = r.read_encoded_int()?;
    let mut t = Tournament::new(w);
    if version >= 5 {
        t.faction_restricted = r.read_bool()?;
    }
    if version >= 4 {
        t.event_controller = r.read_item(w)?;
    }
    if version >= 3 {
        t.sudden_death_rounds = r.read_encoded_int()?;
    }
    if version >= 2 {
        t.tourney_type = TourneyType::from_i32(r.read_encoded_int()?);
    }
    if version >= 1 {
        t.group_type = GroupingType::from_i32(r.read_encoded_int()?);
        t.tie_type = TieType::from_i32(r.read_encoded_int()?);
        t.signup_period = TimeSpan::from_ticks(r.read_time_span_ticks()?);
    }
    if version < 3 {
        t.sudden_death_rounds = 3;
    }
    t.participants_per_match = r.read_encoded_int()?;
    t.players_per_participant = r.read_encoded_int()?;
    t.signup_period = TimeSpan::from_ticks(r.read_time_span_ticks()?);
    Ok(t)
}

/// `TournamentController`.
pub static TOURNAMENT_CONTROLLER: ItemType = ItemType {
    full_name: CONTROLLER_NAME,
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let id = ops::new_item_with_id(w, &TOURNAMENT_CONTROLLER, 0x1B7A);
        ops::set_visible(w, id, false);
        ops::set_movable(w, id, false);
        let t = Tournament::new(w);
        if let Some(i) = w.item_mut(id) {
            i.ext.insert(TournamentData(t));
        }
        controllers(w).0.push(id);
        start_slice(w, id);
        id
    }),
    default_name: |_, _| Some("tournament controller".to_owned()),
    get_context_menu_entries: |w, id, from, list| {
        (ITEM_BASE.get_context_menu_entries)(w, id, from, list);
        if muo_content::mx::access_level(w, from) >= muo_server::mobile::AccessLevel::GameMaster
            && let Some(t) = get(w, id)
        {
            // EditEntry (5101): `PropertiesGump(from, Tournament)`; the properties of the tournament are listed on the
            // controller (see `props`), so the gump of the controller shows them.
            let mut edit = muo_server::context_menu_entry::ContextMenuEntry::new(5101, -1);
            edit.on_click = Some(std::rc::Rc::new(
                |w: &mut World, from: MobileId, target: EntityId| {
                    let EntityId::Item(c) = target else { return };
                    if get(w, c).is_some() {
                        super::games::open_properties(w, from, c);
                    }
                },
            ));
            list.push(edit);
            if t.stage == Stage::Inactive {
                let mut e = muo_server::context_menu_entry::ContextMenuEntry::new(5113, -1);
                e.on_click = Some(std::rc::Rc::new(
                    |w: &mut World, _from: MobileId, target: EntityId| {
                        let EntityId::Item(c) = target else { return };
                        if get(w, c).is_none_or(|t| t.stage != Stage::Inactive) {
                            return;
                        }
                        let now = w.now;
                        if let Some(t) = get_mut(w, c) {
                            t.signup_start = now;
                            t.stage = Stage::Signup;
                            t.participants.clear();
                            t.levels.clear();
                        }
                        alert(
                            w,
                            c,
                            &[
                                "Hear ye! Hear ye!",
                                "Tournament signup has opened. You can enter by signing up with the registrar.",
                            ],
                        );
                    },
                ));
                list.push(e);
            }
        }
    },
    on_double_click: |w, id, from| {
        if muo_content::mx::access_level(w, from) >= muo_server::mobile::AccessLevel::GameMaster
            && get(w, id).is_some()
        {
            if let Some(ns) = w.mobile(from).and_then(|m| m.net_state) {
                close_gump::<gumps::RulesetGump>(w, ns);
                close_gump::<gumps::PickRulesetGump>(w, ns);
            }
            gumps::display_pick_ruleset(w, from, None, Some(id));
        }
    },
    on_delete: |w, id| {
        (ITEM_BASE.on_delete)(w, id);
        controllers(w).0.retain(|x| *x != id);
    },
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        out.write_i32(0);
        if let Some(t) = get(w, id) {
            write_tournament(w, t, out);
        }
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let version = r.read_i32()?;
        if version == 0 {
            let t = read_tournament(w, r)?;
            if let Some(i) = w.item_mut(id) {
                i.ext.insert(TournamentData(t));
            }
            start_slice(w, id);
        }
        controllers(w).0.push(id);
        Ok(())
    },
    ..ITEM_BASE
};

// ---- bracket item and signup book -----------------------------------------------------------------------------------

/// The `Tournament` field of the bracket item and the signup book (and `Registrar` of the book).
#[derive(Clone, Debug, Default)]
pub struct LinkData {
    pub tournament: Option<ItemId>,
    pub registrar: Option<MobileId>,
}

fn link(w: &World, id: ItemId) -> LinkData {
    w.item(id)
        .and_then(|i| i.ext.get::<LinkData>())
        .cloned()
        .unwrap_or_default()
}

fn in_reach(w: &mut World, id: ItemId, from: MobileId) -> bool {
    let loc = ops::get_world_location(w, id);
    if !muo_content::mx::in_range_point(w, from, loc, 2) {
        mm::local_overhead_message_number(w, from, MessageType::REGULAR, 0x3B2, 1019045, ""); // I can't reach that
        false
    } else {
        true
    }
}

/// `TournamentBracketItem`.
pub static TOURNAMENT_BRACKET_ITEM: ItemType = ItemType {
    full_name: "Server.Engines.ConPVP.TournamentBracketItem",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let id = ops::new_item_with_id(w, &TOURNAMENT_BRACKET_ITEM, 0);
        muo_content::templates::apply_item_as(w, id, TOURNAMENT_BRACKET_ITEM.full_name);
        if let Some(i) = w.item_mut(id) {
            i.ext.insert(LinkData::default());
        }
        id
    }),
    default_name: |w, id| crate::template_keys::item_name(w, id, ITEM_BASE.default_name),
    on_double_click: |w, id, from| {
        if !in_reach(w, id, from) {
            return;
        }
        if let Some(ctrl) = link(w, id).tournament
            && get(w, ctrl).is_some()
        {
            super::tournament_gumps::display_bracket(
                w,
                from,
                ctrl,
                super::tournament_gumps::BracketType::Index,
                None,
            );
        }
    },
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        out.write_i32(0);
        out.write_item(w, link(w, id).tournament);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        let tournament = r.read_item(w)?;
        if let Some(i) = w.item_mut(id) {
            i.ext.insert(LinkData {
                tournament,
                registrar: None,
            });
        }
        Ok(())
    },
    ..ITEM_BASE
};

fn registrar_say(w: &mut World, reg: Option<MobileId>, from: MobileId, hue: i32, text: &str) {
    if let Some(r) = reg {
        let ns = w.mobile(from).and_then(|m| m.net_state);
        mm::private_overhead_message(w, r, MessageType::REGULAR, hue, false, text, ns);
    }
}

/// `TournamentSignupItem`.
pub static TOURNAMENT_SIGNUP_ITEM: ItemType = ItemType {
    full_name: "Server.Engines.ConPVP.TournamentSignupItem",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let id = ops::new_item_with_id(w, &TOURNAMENT_SIGNUP_ITEM, 0);
        muo_content::templates::apply_item_as(w, id, TOURNAMENT_SIGNUP_ITEM.full_name);
        if let Some(i) = w.item_mut(id) {
            i.ext.insert(LinkData::default());
        }
        id
    }),
    default_name: |w, id| crate::template_keys::item_name(w, id, ITEM_BASE.default_name),
    on_double_click: |w, id, from| {
        if !in_reach(w, id, from) {
            return;
        }
        let l = link(w, id);
        let Some(ctrl) = l.tournament else { return };
        let Some(t) = get(w, ctrl).cloned() else {
            return;
        };
        let reg = l.registrar;

        if let Some(r) = reg {
            // `Registrar.Direction = Registrar.GetDirectionTo(this)`
            let loc = ops::get_world_location(w, id);
            let d = muo_server::mobile::part2::effects::get_direction_to(w, r, loc.x, loc.y, false);
            muo_content::mx::set_direction(w, r, d);
        }

        match t.stage {
            Stage::Fighting => {
                if reg.is_some() {
                    if t.has_participant(from) {
                        registrar_say(w, reg, from, 0x35, "Excuse me? You are already signed up.");
                    } else {
                        registrar_say(
                            w,
                            reg,
                            from,
                            0x22,
                            "The tournament has already begun. You are too late to signup now.",
                        );
                    }
                }
            }
            Stage::Inactive => registrar_say(w, reg, from, 0x35, "The tournament is closed."),
            Stage::Signup => {
                let lad = ladder::instance(w);
                let entry = lad.and_then(|l| ladder::find(w, l, from));
                if let Some(e) = entry
                    && ladder::get_level(e.experience) < t.level_requirement
                {
                    registrar_say(
                        w,
                        reg,
                        from,
                        0x35,
                        "You have not yet proven yourself a worthy dueler.",
                    );
                    return;
                }
                if t.is_faction_restricted() && crate::factions::find(w, from).is_none() {
                    registrar_say(
                        w,
                        reg,
                        from,
                        0x35,
                        "Only those who have declared their faction allegiance may participate.",
                    );
                    return;
                }
                let ns = w.mobile(from).and_then(|m| m.net_state);
                if ns.is_some_and(|ns| has_gump::<super::tournament_gumps::AcceptTeamGump>(w, ns)) {
                    registrar_say(
                        w,
                        reg,
                        from,
                        0x22,
                        "You must first respond to the offer I've given you.",
                    );
                } else if ns.is_some_and(|ns| has_gump::<gumps::AcceptDuelGump>(w, ns)) {
                    registrar_say(w, reg, from, 0x22, "You must first cancel your duel offer.");
                } else if super::duel_context_of(w, from).is_some() {
                    registrar_say(
                        w,
                        reg,
                        from,
                        0x22,
                        "You are already participating in a duel.",
                    );
                } else if !t.has_participant(from) {
                    super::tournament_gumps::display_confirm_signup(w, from, reg, ctrl, vec![from]);
                } else {
                    registrar_say(
                        w,
                        reg,
                        from,
                        0x35,
                        "You have already entered this tournament.",
                    );
                }
            }
        }
    },
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        out.write_i32(0);
        let l = link(w, id);
        out.write_item(w, l.tournament);
        out.write_mobile(w, l.registrar);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        let tournament = r.read_item(w)?;
        let registrar = r.read_mobile(w)?;
        if let Some(i) = w.item_mut(id) {
            i.ext.insert(LinkData {
                tournament,
                registrar,
            });
        }
        Ok(())
    },
    ..ITEM_BASE
};

// ---- registrar ---------------------------------------------------------------------------------------------------

/// `TournamentRegistrar.Tournament`.
#[derive(Clone, Debug, Default)]
pub struct RegistrarData {
    pub tournament: Option<ItemId>,
}

fn registrar_tournament(w: &World, id: MobileId) -> Option<ItemId> {
    w.mobile(id)?.ext.get::<RegistrarData>()?.tournament
}

fn start_announce(w: &mut World, id: MobileId) {
    let half_minute = TimeSpan::from_secs(30);
    timer::start(w, half_minute, half_minute, 0, move |w| {
        // `Timer.DelayCall` on a deleted mobile keeps firing, but nothing is heard.
        if w.mobile(id).is_none() {
            return;
        }
        let stage = registrar_tournament(w, id)
            .and_then(|c| get(w, c))
            .map(|t| t.stage);
        if stage == Some(Stage::Signup) {
            mm::public_overhead_message(
                w,
                id,
                MessageType::REGULAR,
                0x35,
                false,
                "Come one, come all! Do you aspire to be a fighter of great renown? Join this tournament and show the world your abilities.",
                true,
                muo_server::mobile::AccessLevel::Player,
            );
        }
    });
}

const REGISTRAR_LOCK: &str = "Server.Engines.ConPVP.TournamentRegistrar";

fn registrar_on_movement(w: &mut World, id: MobileId, m: MobileId, old_location: Point3D) {
    (muo_mobiles::npcs::banker::BANKER.on_movement)(w, id, m, old_location);
    let Some(ctrl) = registrar_tournament(w, id) else {
        return;
    };
    let Some(t) = get(w, ctrl).cloned() else {
        return;
    };
    let here = muo_content::mx::location(w, id);
    if muo_content::mx::in_range_point(w, m, here, 4)
        && ((old_location.x - here.x).abs() > 4 || (old_location.y - here.y).abs() > 4)
        && t.stage == Stage::Signup
        && muo_server::mobile::part1::combat::can_begin_action(w, m, REGISTRAR_LOCK)
    {
        let lad = ladder::instance(w);
        let entry = lad.and_then(|l| ladder::find(w, l, m));
        if let Some(e) = entry
            && ladder::get_level(e.experience) < t.level_requirement
        {
            return;
        }
        if t.is_faction_restricted() && crate::factions::find(w, m).is_none() {
            return;
        }
        if t.has_participant(m) {
            return;
        }
        let female = w.mobile(m).is_some_and(|x| x.female);
        let ns = w.mobile(m).and_then(|x| x.net_state);
        mm::private_overhead_message(
            w,
            id,
            MessageType::REGULAR,
            0x35,
            false,
            &format!(
                "Hello m'{}. Dost thou wish to enter this tournament? You need only to write your name in this book.",
                if female { "Lady" } else { "Lord" }
            ),
            ns,
        );
        muo_server::mobile::part1::combat::begin_action(w, m, REGISTRAR_LOCK);
        timer::start(w, TimeSpan::from_secs(10), TimeSpan::ZERO, 1, move |w| {
            muo_server::mobile::part1::combat::end_action(w, m, REGISTRAR_LOCK);
        });
    }
}

/// `TournamentRegistrar`: a `Banker` that announces a tournament in signup.
pub static TOURNAMENT_REGISTRAR: MobileType = MobileType {
    full_name: "Server.Engines.ConPVP.TournamentRegistrar",
    parent: Some(&muo_mobiles::npcs::banker::BANKER),
    construct: Some(|w| {
        let id = muo_content::vendor::new_vendor(w, &TOURNAMENT_REGISTRAR, Some("the banker"));
        if let Some(m) = w.mobile_mut(id) {
            m.ext.insert(RegistrarData::default());
        }
        start_announce(w, id);
        id
    }),
    on_movement: registrar_on_movement,
    serialize: |w, id, out| {
        (muo_mobiles::npcs::banker::BANKER.serialize)(w, id, out);
        out.write_i32(0);
        out.write_item(w, registrar_tournament(w, id));
    },
    deserialize: |w, id, r| {
        (muo_mobiles::npcs::banker::BANKER.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        let tournament = r.read_item(w)?;
        if let Some(m) = w.mobile_mut(id) {
            m.ext.insert(RegistrarData { tournament });
        }
        start_announce(w, id);
        Ok(())
    },
    ..muo_mobiles::npcs::banker::BANKER
};

/// Registers the item and mobile classes.
pub fn register(types: &mut TypeRegistry) {
    types.register_item(&TOURNAMENT_CONTROLLER, &[]);
    types.register_item(&TOURNAMENT_BRACKET_ITEM, &[]);
    types.register_item(&TOURNAMENT_SIGNUP_ITEM, &[]);
    types.register_mobile(&TOURNAMENT_REGISTRAR, &[]);
}

#[allow(dead_code)]
fn _unused(_: DuelPlayer) {}
