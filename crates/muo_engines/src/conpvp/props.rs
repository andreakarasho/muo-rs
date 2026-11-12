//! The `[CommandProperty]` surface of the dueling system for `[props`, `[get` and `[set`: the `Tournament` of a
//! `TournamentController`, the `Arena` (and `ArenaStartPoints`) of an `ArenaController`, and the settings of the event
//! controllers.
//!
//! The members of the nested `[PropertyObject]`s are sub-properties: `[set Tournament.SuddenDeathRounds 3` on a
//! `TournamentController`, `[set Arena.Bounds ..`, `[set Arena.Points.EdgeWest ..` on an `ArenaController`. The
//! controller's "edit" entry opens the properties gump of the controller. The event controllers' `Team1`..`Team8`
//! objects of every game are `Kind::Object` entries (CTF `Team1`..`Team8`, DoubleDom `Team1`/`Team2`, King of the Hill
//! `Team1_W`..`Team8_NE`, Bombing Run `Team1`..`Team4`) with the `[CommandProperty]` members of each team class.

use muo_content::props::{Kind, PropDef, Value};
use muo_content::utilities::types::PropType;
use muo_primitives::{Rectangle2D, TimeSpan};
use muo_server::World;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::mobile::AccessLevel;

use super::tournament::{GroupingType, Stage, TieType, TourneyType, get, get_mut};
use super::{arena, games};

const GM: AccessLevel = AccessLevel::GameMaster;

const INT: Kind = Kind::Plain(PropType::Numeric(
    muo_content::utilities::types::Numeric::Int,
));
const BOOL: Kind = Kind::Plain(PropType::Bool);
const STR: Kind = Kind::Plain(PropType::Str);
const POINT: Kind = Kind::Plain(PropType::IPoint3D);
const ENTITY: Kind = Kind::Plain(PropType::Entity);
const TIME_SPAN: Kind = Kind::TimeSpan;
const RECT: Kind = Kind::Rect2D;
const TOURNEY_TYPE: Kind = Kind::Plain(PropType::Enum(&[
    ("Standard", 0),
    ("FreeForAll", 1),
    ("RandomTeam", 2),
    ("RedVsBlue", 3),
    ("Faction", 4),
]));
const GROUP_TYPE: Kind = Kind::Plain(PropType::Enum(&[
    ("HighVsLow", 0),
    ("Nearest", 1),
    ("Random", 2),
]));
const TIE_TYPE: Kind = Kind::Plain(PropType::Enum(&[
    ("Random", 0),
    ("Highest", 1),
    ("Lowest", 2),
    ("FullElimination", 3),
    ("FullAdvancement", 4),
]));
const STAGE: Kind = Kind::Plain(PropType::Enum(&[
    ("Inactive", 0),
    ("Signup", 1),
    ("Fighting", 2),
]));

fn span(v: &Value) -> Result<TimeSpan, String> {
    match v {
        Value::TimeSpan(t) => Ok(*t),
        _ => Err("not a time span".to_owned()),
    }
}

fn rect(v: &Value) -> Result<Rectangle2D, String> {
    match v {
        Value::Rect(r) => Ok(*r),
        _ => Err("not a rectangle".to_owned()),
    }
}

fn item_of(v: &Value) -> Option<ItemId> {
    match v {
        Value::Entity(Some(EntityId::Item(i))) => Some(*i),
        _ => None,
    }
}

fn mobile_of(v: &Value) -> Option<MobileId> {
    match v {
        Value::Entity(Some(EntityId::Mobile(m))) => Some(*m),
        _ => None,
    }
}

fn ent(i: Option<ItemId>) -> Value {
    Value::Entity(i.map(EntityId::Item))
}

fn only_item(e: EntityId) -> Result<ItemId, String> {
    match e {
        EntityId::Item(i) => Ok(i),
        _ => Err("not an item".to_owned()),
    }
}

macro_rules! prop {
    ($name:literal, $kind:expr, $get:expr, $set:expr) => {
        PropDef {
            name: $name,
            kind: $kind,
            read: GM,
            write: GM,
            get: |w, e| match e {
                EntityId::Item(i) => $get(w, i),
                _ => Value::Null,
            },
            set: Some(|w, e, v| $set(w, only_item(e)?, v)),
        }
    };
    ($name:literal, $kind:expr, $get:expr) => {
        PropDef {
            name: $name,
            kind: $kind,
            read: GM,
            write: GM,
            get: |w, e| match e {
                EntityId::Item(i) => $get(w, i),
                _ => Value::Null,
            },
            set: None,
        }
    };
}

// ---- Tournament (on the TournamentController) ----------------------------------------------------------------------------

fn tv<T>(w: &World, i: ItemId, f: impl FnOnce(&super::tournament::Tournament) -> T, none: T) -> T {
    get(w, i).map_or(none, f)
}

fn te(
    w: &mut World,
    i: ItemId,
    f: impl FnOnce(&mut super::tournament::Tournament),
) -> Result<(), String> {
    get_mut(w, i)
        .map(f)
        .ok_or_else(|| "no tournament".to_owned())
}

static TOURNAMENT_SUBS: &[PropDef] = &[
    prop!(
        "EventController",
        ENTITY,
        |w: &World, i| ent(tv(w, i, |t| t.event_controller, None)),
        |w: &mut World, i, v: Value| te(w, i, |t| t.event_controller = item_of(&v))
    ),
    prop!(
        "SuddenDeathRounds",
        INT,
        |w: &World, i| Value::Int(i64::from(tv(w, i, |t| t.sudden_death_rounds, 0))),
        |w: &mut World, i, v: Value| te(w, i, |t| t.sudden_death_rounds = v.i32())
    ),
    prop!(
        "TourneyType",
        TOURNEY_TYPE,
        |w: &World, i| Value::Enum(tv(w, i, |t| t.tourney_type as i64, 0)),
        |w: &mut World, i, v: Value| te(w, i, |t| t.tourney_type = TourneyType::from_i32(v.i32()))
    ),
    prop!(
        "GroupType",
        GROUP_TYPE,
        |w: &World, i| Value::Enum(tv(w, i, |t| t.group_type as i64, 0)),
        |w: &mut World, i, v: Value| te(w, i, |t| t.group_type = GroupingType::from_i32(v.i32()))
    ),
    prop!(
        "TieType",
        TIE_TYPE,
        |w: &World, i| Value::Enum(tv(w, i, |t| t.tie_type as i64, 0)),
        |w: &mut World, i, v: Value| te(w, i, |t| t.tie_type = TieType::from_i32(v.i32()))
    ),
    prop!(
        "SuddenDeath",
        TIME_SPAN,
        |w: &World, i| Value::TimeSpan(tv(w, i, |t| t.sudden_death, Default::default())),
        |w: &mut World, i, v: Value| {
            let s = span(&v)?;
            te(w, i, |t| t.sudden_death = s)
        }
    ),
    prop!(
        "ParticipantsPerMatch",
        INT,
        |w: &World, i| Value::Int(i64::from(tv(w, i, |t| t.participants_per_match, 0))),
        |w: &mut World, i, v: Value| te(w, i, |t| t.set_participants_per_match(v.i32()))
    ),
    prop!(
        "PlayersPerParticipant",
        INT,
        |w: &World, i| Value::Int(i64::from(tv(w, i, |t| t.players_per_participant, 0))),
        |w: &mut World, i, v: Value| te(w, i, |t| t.set_players_per_participant(v.i32()))
    ),
    prop!(
        "LevelRequirement",
        INT,
        |w: &World, i| Value::Int(i64::from(tv(w, i, |t| t.level_requirement, 0))),
        |w: &mut World, i, v: Value| te(w, i, |t| t.level_requirement = v.i32())
    ),
    prop!(
        "FactionRestricted",
        BOOL,
        |w: &World, i| Value::Bool(tv(w, i, |t| t.faction_restricted, false)),
        |w: &mut World, i, v: Value| te(w, i, |t| t.faction_restricted = v.bool())
    ),
    prop!(
        "SignupPeriod",
        TIME_SPAN,
        |w: &World, i| Value::TimeSpan(tv(w, i, |t| t.signup_period, Default::default())),
        |w: &mut World, i, v: Value| {
            let s = span(&v)?;
            te(w, i, |t| t.signup_period = s)
        }
    ),
    prop!("SignupStart", Kind::DateTime, |w: &World, i| {
        Value::DateTime(tv(w, i, |t| t.signup_start, Default::default()))
    }),
    prop!("CurrentStage", STAGE, |w: &World, i| Value::Enum(tv(
        w,
        i,
        |t| match t.stage {
            Stage::Inactive => 0,
            Stage::Signup => 1,
            Stage::Fighting => 2,
        },
        0
    ))),
];

// ---- Arena (on the ArenaController) --------------------------------------------------------------------------------------

fn ad<T>(w: &World, i: ItemId, f: impl FnOnce(&arena::ArenaData) -> T, none: T) -> T {
    arena::data(w, i).map_or(none, f)
}

fn ae(w: &mut World, i: ItemId, f: impl FnOnce(&mut arena::ArenaData)) -> Result<(), String> {
    arena::data_mut(w, i)
        .map(f)
        .ok_or_else(|| "no arena".to_owned())
}

macro_rules! point_prop {
    ($name:literal, $field:ident) => {
        prop!(
            $name,
            POINT,
            |w: &World, i| Value::Point3D(ad(w, i, |d| d.$field, Default::default())),
            |w: &mut World, i, v: Value| ae(w, i, |d| d.$field = v.point())
        )
    };
}

macro_rules! start_point {
    ($name:literal, $n:literal) => {
        prop!(
            $name,
            POINT,
            |w: &World, i| Value::Point3D(ad(w, i, |d| d.points[$n], Default::default())),
            |w: &mut World, i, v: Value| ae(w, i, |d| d.points[$n] = v.point())
        )
    };
}

static ARENA_SUBS: &[PropDef] = &[
    prop!(
        "Ladder",
        ENTITY,
        |w: &World, i| ent(ad(w, i, |d| d.ladder, None)),
        |w: &mut World, i, v: Value| ae(w, i, |d| d.ladder = item_of(&v))
    ),
    prop!(
        "IsGuarded",
        BOOL,
        |w: &World, i| Value::Bool(ad(w, i, |d| d.is_guarded, false)),
        |w: &mut World, i, v: Value| {
            arena::set_is_guarded(w, i, v.bool());
            Ok(())
        }
    ),
    prop!(
        "Tournament",
        ENTITY,
        |w: &World, i| ent(ad(w, i, |d| d.tournament, None)),
        |w: &mut World, i, v: Value| {
            arena::set_tournament(w, i, item_of(&v));
            Ok(())
        }
    ),
    prop!(
        "Announcer",
        ENTITY,
        |w: &World, i| Value::Entity(ad(w, i, |d| d.announcer, None).map(EntityId::Mobile)),
        |w: &mut World, i, v: Value| ae(w, i, |d| d.announcer = mobile_of(&v))
    ),
    prop!(
        "Name",
        STR,
        |w: &World, i| ad(w, i, |d| d.name.clone(), None).map_or(Value::Null, Value::Str),
        |w: &mut World, i, v: Value| {
            arena::set_name(w, i, v.string());
            Ok(())
        }
    ),
    prop!(
        "Facet",
        Kind::Map,
        |w: &World, i| Value::Map(ad(w, i, |d| d.facet, None)),
        |w: &mut World, i, v: Value| {
            arena::set_facet(w, i, if let Value::Map(m) = v { m } else { None });
            Ok(())
        }
    ),
    prop!(
        "Bounds",
        RECT,
        |w: &World, i| Value::Rect(ad(w, i, |d| d.bounds, Default::default())),
        |w: &mut World, i, v: Value| {
            let r = rect(&v)?;
            ae(w, i, |d| d.bounds = r)
        }
    ),
    prop!(
        "Zone",
        RECT,
        |w: &World, i| Value::Rect(ad(w, i, |d| d.zone, Default::default())),
        |w: &mut World, i, v: Value| {
            let r = rect(&v)?;
            arena::set_zone(w, i, r);
            Ok(())
        }
    ),
    point_prop!("Outside", outside),
    point_prop!("GateIn", gate_in),
    prop!(
        "GateOut",
        POINT,
        |w: &World, i| Value::Point3D(ad(w, i, |d| d.gate_out, Default::default())),
        |w: &mut World, i, v: Value| {
            arena::set_gate_out(w, i, v.point());
            Ok(())
        }
    ),
    point_prop!("Wall", wall),
    prop!("IsOccupied", BOOL, |w: &World, i| Value::Bool(
        arena::is_occupied(w, i)
    )),
    prop!(
        "Active",
        BOOL,
        |w: &World, i| Value::Bool(ad(w, i, |d| d.active, false)),
        |w: &mut World, i, v: Value| {
            arena::set_active(w, i, v.bool());
            Ok(())
        }
    ),
    PropDef {
        name: "Points",
        kind: Kind::Object("ArenaStartPoints", ARENA_START_POINTS),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) if arena::data(w, i).is_some() => Value::Text("...".into()),
            _ => Value::Null,
        },
        set: None,
    },
    PropDef {
        name: "ForceEvict",
        kind: BOOL,
        read: AccessLevel::Administrator,
        write: AccessLevel::Administrator,
        get: |_, _| Value::Bool(false),
        set: Some(|w, e, v| {
            if v.bool() {
                arena::evict(w, only_item(e)?);
            }
            Ok(())
        }),
    },
];

static ARENA_START_POINTS: &[PropDef] = &[
    start_point!("EdgeWest", 0),
    start_point!("EdgeEast", 1),
    start_point!("EdgeNorth", 2),
    start_point!("EdgeSouth", 3),
    start_point!("CornerNW", 4),
    start_point!("CornerSE", 5),
    start_point!("CornerSW", 6),
    start_point!("CornerNE", 7),
];

// ---- the event controllers -------------------------------------------------------------------------------------------

fn ce<T>(w: &World, i: ItemId, f: impl FnOnce(&games::EventCtrl) -> T, none: T) -> T {
    games::ctrl(w, i).map_or(none, f)
}

fn cm(w: &mut World, i: ItemId, f: impl FnOnce(&mut games::EventCtrl)) -> Result<(), String> {
    games::ctrl_mut(w, i)
        .map(f)
        .ok_or_else(|| "no controller".to_owned())
}

macro_rules! hill_prop {
    ($name:literal, $n:literal) => {
        prop!(
            $name,
            ENTITY,
            |w: &World, i| ent(ce(w, i, |c| c.hills.get($n).copied().flatten(), None)),
            |w: &mut World, i, v: Value| cm(w, i, |c| {
                if let Some(h) = c.hills.get_mut($n) {
                    *h = item_of(&v);
                }
            })
        )
    };
}

macro_rules! team_board {
    ($n:literal) => {
        prop!(
            "Board",
            ENTITY,
            |w: &World, i| ent(ce(w, i, |c| c.teams.get($n).and_then(|t| t.board), None)),
            |w: &mut World, i, v: Value| cm(w, i, |c| {
                if let Some(t) = c.teams.get_mut($n) {
                    t.board = item_of(&v);
                }
            })
        )
    };
}
macro_rules! team_color {
    ($n:literal) => {
        prop!(
            "Color",
            INT,
            |w: &World, i| Value::Int(i64::from(ce(
                w,
                i,
                |c| c.teams.get($n).map_or(0, |t| t.color),
                0
            ))),
            |w: &mut World, i, v: Value| cm(w, i, |c| {
                if let Some(t) = c.teams.get_mut($n) {
                    t.color = v.i32();
                }
            })
        )
    };
}
macro_rules! team_name {
    ($pname:literal, $n:literal) => {
        prop!(
            $pname,
            STR,
            |w: &World, i| ce(w, i, |c| c.teams.get($n).and_then(|t| t.name.clone()), None)
                .map_or(Value::Null, Value::Str),
            |w: &mut World, i, v: Value| cm(w, i, |c| {
                if let Some(t) = c.teams.get_mut($n) {
                    t.name = v.string();
                }
            })
        )
    };
}
macro_rules! team_origin {
    ($n:literal) => {
        prop!(
            "Origin",
            POINT,
            |w: &World, i| Value::Point3D(ce(
                w,
                i,
                |c| c.teams.get($n).map_or(Default::default(), |t| t.origin),
                Default::default()
            )),
            |w: &mut World, i, v: Value| cm(w, i, |c| {
                if let Some(t) = c.teams.get_mut($n) {
                    t.origin = v.point();
                }
            })
        )
    };
}
/// `CTFTeamInfo`'s `[CommandProperty]` members: Board, Color, Name, Flag, Origin.
macro_rules! ctf_team_subs {
    ($n:literal) => {
        &[
            team_board!($n),
            team_color!($n),
            team_name!("Name", $n),
            prop!(
                "Flag",
                ENTITY,
                |w: &World, i| ent(ce(w, i, |c| c.teams.get($n).and_then(|t| t.flag), None)),
                |w: &mut World, i, v: Value| cm(w, i, |c| {
                    if let Some(t) = c.teams.get_mut($n) {
                        t.flag = item_of(&v);
                    }
                })
            ),
            team_origin!($n),
        ]
    };
}
/// `DDTeamInfo`: Board, Color, TeamName, Origin. `KHTeamInfo`: Color, TeamName.
macro_rules! dd_team_subs {
    ($n:literal) => {
        &[
            team_board!($n),
            team_color!($n),
            team_name!("TeamName", $n),
            team_origin!($n),
        ]
    };
}
macro_rules! kh_team_subs {
    ($n:literal) => {
        &[team_color!($n), team_name!("TeamName", $n)]
    };
}

/// A `[PropertyObject]` team member of a controller; shown only while the controller has that team.
macro_rules! team_obj {
    ($name:literal, $ty:literal, $subs:expr, $n:literal) => {
        PropDef {
            name: $name,
            kind: Kind::Object($ty, $subs),
            read: GM,
            write: GM,
            get: |w, e| match e {
                EntityId::Item(i) if games::team(w, i, $n).is_some() => Value::Text("...".into()),
                _ => Value::Null,
            },
            set: None,
        }
    };
}

static CTF_TEAMS: [&[PropDef]; 8] = [
    ctf_team_subs!(0),
    ctf_team_subs!(1),
    ctf_team_subs!(2),
    ctf_team_subs!(3),
    ctf_team_subs!(4),
    ctf_team_subs!(5),
    ctf_team_subs!(6),
    ctf_team_subs!(7),
];
static DD_TEAMS: [&[PropDef]; 2] = [dd_team_subs!(0), dd_team_subs!(1)];
static KH_TEAMS: [&[PropDef]; 8] = [
    kh_team_subs!(0),
    kh_team_subs!(1),
    kh_team_subs!(2),
    kh_team_subs!(3),
    kh_team_subs!(4),
    kh_team_subs!(5),
    kh_team_subs!(6),
    kh_team_subs!(7),
];

static CTF_PROPS: &[PropDef] = &[
    team_obj!("Team1", "CTFTeamInfo", CTF_TEAMS[0], 0),
    team_obj!("Team2", "CTFTeamInfo", CTF_TEAMS[1], 1),
    team_obj!("Team3", "CTFTeamInfo", CTF_TEAMS[2], 2),
    team_obj!("Team4", "CTFTeamInfo", CTF_TEAMS[3], 3),
    team_obj!("Team5", "CTFTeamInfo", CTF_TEAMS[4], 4),
    team_obj!("Team6", "CTFTeamInfo", CTF_TEAMS[5], 5),
    team_obj!("Team7", "CTFTeamInfo", CTF_TEAMS[6], 6),
    team_obj!("Team8", "CTFTeamInfo", CTF_TEAMS[7], 7),
    prop!(
        "Duration",
        TIME_SPAN,
        |w: &World, i| Value::TimeSpan(ce(w, i, |c| c.duration, Default::default())),
        |w: &mut World, i, v: Value| {
            let s = span(&v)?;
            cm(w, i, |c| c.duration = s)
        }
    ),
];

static DD_PROPS: &[PropDef] = &[
    team_obj!("Team1", "DDTeamInfo", DD_TEAMS[0], 0),
    team_obj!("Team2", "DDTeamInfo", DD_TEAMS[1], 1),
    prop!(
        "PointA",
        ENTITY,
        |w: &World, i| ent(ce(w, i, |c| c.point_a, None)),
        |w: &mut World, i, v: Value| cm(w, i, |c| c.point_a = item_of(&v))
    ),
    prop!(
        "PointB",
        ENTITY,
        |w: &World, i| ent(ce(w, i, |c| c.point_b, None)),
        |w: &mut World, i, v: Value| cm(w, i, |c| c.point_b = item_of(&v))
    ),
    prop!(
        "Duration",
        TIME_SPAN,
        |w: &World, i| Value::TimeSpan(ce(w, i, |c| c.duration, Default::default())),
        |w: &mut World, i, v: Value| {
            let s = span(&v)?;
            cm(w, i, |c| c.duration = s)
        }
    ),
];

static KH_PROPS: &[PropDef] = &[
    team_obj!("Team1_W", "KHTeamInfo", KH_TEAMS[0], 0),
    team_obj!("Team2_E", "KHTeamInfo", KH_TEAMS[1], 1),
    team_obj!("Team3_N", "KHTeamInfo", KH_TEAMS[2], 2),
    team_obj!("Team4_S", "KHTeamInfo", KH_TEAMS[3], 3),
    team_obj!("Team5_NW", "KHTeamInfo", KH_TEAMS[4], 4),
    team_obj!("Team6_SE", "KHTeamInfo", KH_TEAMS[5], 5),
    team_obj!("Team7_SW", "KHTeamInfo", KH_TEAMS[6], 6),
    team_obj!("Team8_NE", "KHTeamInfo", KH_TEAMS[7], 7),
    hill_prop!("Hill1", 0),
    hill_prop!("Hill2", 1),
    hill_prop!("Hill3", 2),
    hill_prop!("Hill4", 3),
    prop!(
        "Duration",
        TIME_SPAN,
        |w: &World, i| Value::TimeSpan(ce(w, i, |c| c.duration, Default::default())),
        |w: &mut World, i, v: Value| {
            let s = span(&v)?;
            cm(w, i, |c| c.duration = s)
        }
    ),
];

/// `BRTeamInfo`'s `[CommandProperty]` members (Board, Color, TeamName, Goal) of team `$n` of a `BRController`.
macro_rules! br_team_subs {
    ($n:literal) => {
        &[
            prop!(
                "Board",
                ENTITY,
                |w: &World, i| ent(ce(w, i, |c| c.teams.get($n).and_then(|t| t.board), None)),
                |w: &mut World, i, v: Value| cm(w, i, |c| {
                    if let Some(t) = c.teams.get_mut($n) {
                        t.board = item_of(&v);
                    }
                })
            ),
            prop!(
                "Color",
                INT,
                |w: &World, i| Value::Int(i64::from(ce(
                    w,
                    i,
                    |c| c.teams.get($n).map_or(0, |t| t.color),
                    0
                ))),
                |w: &mut World, i, v: Value| cm(w, i, |c| {
                    if let Some(t) = c.teams.get_mut($n) {
                        t.color = v.i32();
                    }
                })
            ),
            prop!(
                "TeamName",
                STR,
                |w: &World, i| ce(w, i, |c| c.teams.get($n).and_then(|t| t.name.clone()), None)
                    .map_or(Value::Null, Value::Str),
                |w: &mut World, i, v: Value| cm(w, i, |c| {
                    if let Some(t) = c.teams.get_mut($n) {
                        t.name = v.string();
                    }
                })
            ),
            prop!(
                "Goal",
                ENTITY,
                |w: &World, i| ent(ce(w, i, |c| c.teams.get($n).and_then(|t| t.goal), None)),
                // `m_Goal = value; m_Goal?.Team = this`
                |w: &mut World, i, v: Value| {
                    let goal = item_of(&v);
                    cm(w, i, |c| {
                        if let Some(t) = c.teams.get_mut($n) {
                            t.goal = goal;
                        }
                    })?;
                    if let Some(g) = goal {
                        super::bombing::set_goal_team(w, g, Some((i, $n)));
                    }
                    Ok(())
                }
            ),
        ]
    };
}

static BR_TEAM1: &[PropDef] = br_team_subs!(0);
static BR_TEAM2: &[PropDef] = br_team_subs!(1);
static BR_TEAM3: &[PropDef] = br_team_subs!(2);
static BR_TEAM4: &[PropDef] = br_team_subs!(3);

/// `Team1`..`Team4` of a `BRController`: `[PropertyObject]`s over the controller.
macro_rules! br_team {
    ($name:literal, $subs:ident, $n:literal) => {
        PropDef {
            name: $name,
            kind: Kind::Object("BRTeamInfo", $subs),
            read: GM,
            write: GM,
            get: |w, e| match e {
                EntityId::Item(i) if games::team(w, i, $n).is_some() => Value::Text("...".into()),
                _ => Value::Null,
            },
            set: None,
        }
    };
}

static BR_PROPS: &[PropDef] = &[
    br_team!("Team1", BR_TEAM1, 0),
    br_team!("Team2", BR_TEAM2, 1),
    br_team!("Team3", BR_TEAM3, 2),
    br_team!("Team4", BR_TEAM4, 3),
    prop!(
        "BombHome",
        POINT,
        |w: &World, i| Value::Point3D(ce(w, i, |c| c.br.bomb_home, Default::default())),
        |w: &mut World, i, v: Value| cm(w, i, |c| c.br.bomb_home = v.point())
    ),
    prop!(
        "Duration",
        TIME_SPAN,
        |w: &World, i| Value::TimeSpan(ce(w, i, |c| c.duration, Default::default())),
        |w: &mut World, i, v: Value| {
            let s = span(&v)?;
            cm(w, i, |c| c.duration = s)
        }
    ),
];

/// `TournamentController.Tournament` and `ArenaController.Arena`: `canModify: true` objects over the controller.
static TOURNAMENT_PROPS: &[PropDef] = &[PropDef {
    name: "Tournament",
    kind: Kind::Object("Tournament", TOURNAMENT_SUBS),
    read: GM,
    write: GM,
    get: |w, e| match e {
        EntityId::Item(i) if get(w, i).is_some() => Value::Text("...".into()),
        _ => Value::Null,
    },
    set: None,
}];

static ARENA_PROPS: &[PropDef] = &[
    prop!(
        "IsPrivate",
        BOOL,
        |w: &World, i| Value::Bool(ad(w, i, |d| d.is_private, false)),
        |w: &mut World, i, v: Value| ae(w, i, |d| d.is_private = v.bool())
    ),
    PropDef {
        name: "Arena",
        kind: Kind::Object("Arena", ARENA_SUBS),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) if arena::data(w, i).is_some() => Value::Text("...".into()),
            _ => Value::Null,
        },
        set: None,
    },
];

/// Adds the tables.
pub fn configure(w: &mut World) {
    muo_content::props::add(w, super::tournament::CONTROLLER_NAME, TOURNAMENT_PROPS);
    muo_content::props::add(w, super::arena::CONTROLLER_NAME, ARENA_PROPS);
    muo_content::props::add(w, "Server.Engines.ConPVP.CTFController", CTF_PROPS);
    muo_content::props::add(w, "Server.Engines.ConPVP.DDController", DD_PROPS);
    muo_content::props::add(w, "Server.Engines.ConPVP.KHController", KH_PROPS);
    muo_content::props::add(w, "Server.Engines.ConPVP.BRController", BR_PROPS);
}

/// `LadderEntry`'s `[CommandProperty]` members, read off the entry of the owner mobile.
fn entry_int(w: &mut World, e: EntityId, f: fn(&super::ladder::LadderEntry) -> i32) -> Value {
    let (EntityId::Mobile(m), Some(lad)) = (e, super::ladder::instance(w)) else {
        return Value::Null;
    };
    super::ladder::find_no_create(w, lad, m).map_or(Value::Null, |x| Value::Int(i64::from(f(x))))
}

pub static LADDER_ENTRY_PROPS: &[PropDef] = &[
    PropDef {
        name: "Experience",
        kind: Kind::Plain(PropType::Numeric(
            muo_content::utilities::types::Numeric::Int,
        )),
        read: GM,
        write: AccessLevel::Administrator,
        get: |w, e| entry_int(w, e, |x| x.experience),
        // `m_Experience = value; m_Ladder.UpdateEntry(this)`
        set: Some(|w, e, v| {
            let n = v.i32();
            if let (EntityId::Mobile(m), Some(lad)) = (e, super::ladder::instance(w)) {
                if let Some(x) = super::ladder::entry_mut(w, lad, m) {
                    x.experience = n;
                }
                super::ladder::update_entry(w, lad, m);
            }
            Ok(())
        }),
    },
    PropDef {
        name: "Wins",
        kind: Kind::Plain(PropType::Numeric(
            muo_content::utilities::types::Numeric::Int,
        )),
        read: GM,
        write: AccessLevel::Administrator,
        get: |w, e| entry_int(w, e, |x| x.wins),
        set: Some(|w, e, v| {
            let n = v.i32();
            if let (EntityId::Mobile(m), Some(lad)) = (e, super::ladder::instance(w))
                && let Some(x) = super::ladder::entry_mut(w, lad, m)
            {
                x.wins = n;
            }
            Ok(())
        }),
    },
    PropDef {
        name: "Losses",
        kind: Kind::Plain(PropType::Numeric(
            muo_content::utilities::types::Numeric::Int,
        )),
        read: GM,
        write: AccessLevel::Administrator,
        get: |w, e| entry_int(w, e, |x| x.losses),
        set: Some(|w, e, v| {
            let n = v.i32();
            if let (EntityId::Mobile(m), Some(lad)) = (e, super::ladder::instance(w))
                && let Some(x) = super::ladder::entry_mut(w, lad, m)
            {
                x.losses = n;
            }
            Ok(())
        }),
    },
    PropDef {
        name: "Rank",
        kind: Kind::Plain(PropType::Numeric(
            muo_content::utilities::types::Numeric::Int,
        )),
        read: GM,
        write: GM,
        get: |w, e| entry_int(w, e, |x| x.index as i32),
        set: None,
    },
];

#[cfg(test)]
mod br_team_tests {
    use super::*;

    /// `Team2.Goal` / `.Color` / `.TeamName` on a Bombing Run controller: the goal takes the team's colour.
    #[test]
    fn a_bombing_run_team_is_edited_through_its_sub_properties() {
        let mut w = World::new();
        muo_content::register(&mut w.types);
        muo_items::register(&mut w.types);
        super::super::register(&mut w.types);
        let ec = super::super::bombing::new_controller_br(&mut w);
        let goal = muo_content::factory::make_item(&mut w, "Server.Engines.ConPVP.BRGoal");
        let goal = goal.expect("BRGoal is registered");
        let set = |w: &mut World, name: &str, v: Value| {
            (BR_TEAM2
                .iter()
                .find(|d| d.name == name)
                .unwrap()
                .set
                .unwrap())(w, EntityId::Item(ec), v)
            .unwrap()
        };
        set(&mut w, "Color", Value::Int(0x4F2));
        set(&mut w, "TeamName", Value::Str("Blue".into()));
        set(&mut w, "Goal", Value::Entity(Some(EntityId::Item(goal))));
        let t = games::team(&w, ec, 1).unwrap();
        assert_eq!(
            (t.color, t.name.as_deref(), t.goal),
            (0x4F2, Some("Blue"), Some(goal))
        );
        assert_eq!(w.item(goal).unwrap().hue, 0x4F2);
        assert!(games::team(&w, ec, 0).unwrap().goal.is_none());
    }

    /// Team objects of CTF, DD and KH: the members are those of the team class, edited in place.
    #[test]
    fn ctf_dd_and_kh_teams_are_edited_through_their_sub_properties() {
        let mut w = World::new();
        muo_content::register(&mut w.types);
        muo_items::register(&mut w.types);
        super::super::register(&mut w.types);
        let ctf = super::super::ctf::new_controller_ctf(&mut w);
        let dd = super::super::dd::new_controller_dd(&mut w);
        let kh = super::super::koth::new_controller_kh(&mut w);
        let set = |w: &mut World, subs: &[PropDef], ec, name: &str, v: Value| {
            (subs.iter().find(|d| d.name == name).unwrap().set.unwrap())(w, EntityId::Item(ec), v)
                .unwrap()
        };
        set(&mut w, CTF_TEAMS[7], ctf, "Name", Value::Str("Red".into()));
        set(&mut w, CTF_TEAMS[7], ctf, "Color", Value::Int(33));
        set(
            &mut w,
            CTF_TEAMS[7],
            ctf,
            "Origin",
            Value::Point3D(muo_primitives::Point3D::new(1, 2, 3)),
        );
        let t = games::team(&w, ctf, 7).unwrap();
        assert_eq!(
            (t.name.as_deref(), t.color, t.origin.z),
            (Some("Red"), 33, 3)
        );
        assert!(games::team(&w, ctf, 0).unwrap().name.is_none());
        set(&mut w, DD_TEAMS[1], dd, "TeamName", Value::Str("B".into()));
        assert_eq!(games::team(&w, dd, 1).unwrap().name.as_deref(), Some("B"));
        assert!(
            CTF_PROPS.iter().any(|d| d.name == "Team8")
                && DD_PROPS.iter().any(|d| d.name == "Team2")
        );
        set(&mut w, KH_TEAMS[3], kh, "Color", Value::Int(7));
        assert_eq!(games::team(&w, kh, 3).unwrap().color, 7);
        assert!(KH_PROPS.iter().any(|d| d.name == "Team8_NE"));
    }
}
