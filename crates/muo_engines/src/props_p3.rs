//! `[CommandProperty]` objects of the engines (task props3): the event controllers' `Team1`..`Team8` (`CTFTeamInfo`,
//! `DDTeamInfo`, `KHTeamInfo`, `BRTeamInfo`), `LadderController.Ladder`, `PreferencesController.Preferences`, the ethics
//! `Player` of a `PlayerMobile` and the `MLQuestContext` members. An object that is not an entity is a `Kind::Object`
//! whose sub-properties take the host entity: the controller for a team, the player for the
//! ethics player and the quest context.

use muo_content::props::{self, Kind, PropDef, Value, kinds};
use muo_primitives::Point3D;
use muo_primitives::Serial;
use muo_server::World;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::mobile::AccessLevel;

use crate::conpvp::games::{self, TeamInfo};

const GM: AccessLevel = AccessLevel::GameMaster;
const ADMIN: AccessLevel = AccessLevel::Administrator;

fn item(e: EntityId) -> Option<ItemId> {
    match e {
        EntityId::Item(i) => Some(i),
        EntityId::Mobile(_) => None,
    }
}

fn mob(e: EntityId) -> Option<MobileId> {
    match e {
        EntityId::Mobile(m) => Some(m),
        EntityId::Item(_) => None,
    }
}

fn ent_item(i: Option<ItemId>) -> Value {
    Value::Entity(i.map(EntityId::Item))
}

fn item_arg(v: &Value) -> Option<ItemId> {
    match v.entity() {
        Some(EntityId::Item(i)) => Some(i),
        _ => None,
    }
}

// ---- the teams of an event controller ------------------------------------------------------------------------------

fn tget(w: &World, e: EntityId, idx: usize) -> Option<&TeamInfo> {
    games::team(w, item(e)?, idx)
}

fn tset(
    w: &mut World,
    e: EntityId,
    idx: usize,
    f: impl FnOnce(&mut TeamInfo),
) -> Result<(), String> {
    let i = item(e).ok_or_else(|| "That is not an item.".to_string())?;
    games::team_mut(w, i, idx)
        .map(f)
        .ok_or_else(|| "That controller has no such team.".to_string())
}

macro_rules! team_prop {
    ($name:literal, $kind:expr, $idx:literal, |$t:ident| $get:expr, |$t2:ident, $v:ident| $set:expr) => {
        PropDef {
            name: $name,
            kind: $kind,
            read: GM,
            write: GM,
            get: |w, e| tget(w, e, $idx).map_or(Value::Null, |$t| $get),
            set: Some(|w, e, $v| tset(w, e, $idx, |$t2| $set)),
        }
    };
}

macro_rules! team_color {
    ($idx:literal) => {
        team_prop!(
            "Color",
            kinds::INT,
            $idx,
            |t| Value::Int(i64::from(t.color)),
            |t, v| t.color = v.i32()
        )
    };
}
macro_rules! team_name {
    ($name:literal, $idx:literal) => {
        team_prop!(
            $name,
            kinds::STR,
            $idx,
            |t| t.name.clone().map_or(Value::Null, Value::Str),
            |t, v| t.name = v.string()
        )
    };
}
macro_rules! team_board {
    ($idx:literal) => {
        team_prop!(
            "Board",
            kinds::ENTITY,
            $idx,
            |t| ent_item(t.board),
            |t, v| t.board = item_arg(&v)
        )
    };
}
macro_rules! team_origin {
    ($idx:literal) => {
        team_prop!(
            "Origin",
            kinds::POINT3,
            $idx,
            |t| Value::Point3D(t.origin),
            |t, v| t.origin = v.point()
        )
    };
}

// CTFTeamInfo: Board, Color, Name, Flag, Origin
macro_rules! ctf_team {
    ($ident:ident, $idx:literal) => {
        static $ident: &[PropDef] = &[
            team_board!($idx),
            team_color!($idx),
            team_name!("Name", $idx),
            team_prop!("Flag", kinds::ENTITY, $idx, |t| ent_item(t.flag), |t, v| {
                t.flag = item_arg(&v)
            }),
            team_origin!($idx),
        ];
    };
}
// DDTeamInfo: Board, Color, TeamName, Origin
macro_rules! dd_team {
    ($ident:ident, $idx:literal) => {
        static $ident: &[PropDef] = &[
            team_board!($idx),
            team_color!($idx),
            team_name!("TeamName", $idx),
            team_origin!($idx),
        ];
    };
}
// KHTeamInfo: Color, TeamName
macro_rules! kh_team {
    ($ident:ident, $idx:literal) => {
        static $ident: &[PropDef] = &[team_color!($idx), team_name!("TeamName", $idx)];
    };
}
// BRTeamInfo: Board, Color, TeamName, Goal
macro_rules! br_team {
    ($ident:ident, $idx:literal) => {
        static $ident: &[PropDef] = &[
            team_board!($idx),
            team_color!($idx),
            team_name!("TeamName", $idx),
            team_prop!("Goal", kinds::ENTITY, $idx, |t| ent_item(t.goal), |t, v| {
                t.goal = item_arg(&v)
            }),
        ];
    };
}

ctf_team!(CTF0, 0);
ctf_team!(CTF1, 1);
ctf_team!(CTF2, 2);
ctf_team!(CTF3, 3);
ctf_team!(CTF4, 4);
ctf_team!(CTF5, 5);
ctf_team!(CTF6, 6);
ctf_team!(CTF7, 7);
dd_team!(DD0, 0);
dd_team!(DD1, 1);
kh_team!(KH0, 0);
kh_team!(KH1, 1);
kh_team!(KH2, 2);
kh_team!(KH3, 3);
kh_team!(KH4, 4);
kh_team!(KH5, 5);
kh_team!(KH6, 6);
kh_team!(KH7, 7);
br_team!(BR0, 0);
br_team!(BR1, 1);
br_team!(BR2, 2);
br_team!(BR3, 3);

/// `public XTeamInfo TeamN => TeamInfo[n]`.
macro_rules! team_obj {
    ($name:literal, $class:literal, $subs:ident, $idx:literal) => {
        PropDef {
            name: $name,
            kind: Kind::Object($class, $subs),
            read: GM,
            write: GM,
            get: |w, e| {
                if tget(w, e, $idx).is_some() {
                    Value::Text("...".to_string())
                } else {
                    Value::Null
                }
            },
            set: None,
        }
    };
}

static CTF_CONTROLLER: &[PropDef] = &[
    team_obj!("Team1", "CTFTeamInfo", CTF0, 0),
    team_obj!("Team2", "CTFTeamInfo", CTF1, 1),
    team_obj!("Team3", "CTFTeamInfo", CTF2, 2),
    team_obj!("Team4", "CTFTeamInfo", CTF3, 3),
    team_obj!("Team5", "CTFTeamInfo", CTF4, 4),
    team_obj!("Team6", "CTFTeamInfo", CTF5, 5),
    team_obj!("Team7", "CTFTeamInfo", CTF6, 6),
    team_obj!("Team8", "CTFTeamInfo", CTF7, 7),
];
static DD_CONTROLLER: &[PropDef] = &[
    team_obj!("Team1", "DDTeamInfo", DD0, 0),
    team_obj!("Team2", "DDTeamInfo", DD1, 1),
];
static KH_CONTROLLER: &[PropDef] = &[
    team_obj!("Team1_W", "KHTeamInfo", KH0, 0),
    team_obj!("Team2_E", "KHTeamInfo", KH1, 1),
    team_obj!("Team3_N", "KHTeamInfo", KH2, 2),
    team_obj!("Team4_S", "KHTeamInfo", KH3, 3),
    team_obj!("Team5_NW", "KHTeamInfo", KH4, 4),
    team_obj!("Team6_SE", "KHTeamInfo", KH5, 5),
    team_obj!("Team7_SW", "KHTeamInfo", KH6, 6),
    team_obj!("Team8_NE", "KHTeamInfo", KH7, 7),
];
static BR_CONTROLLER: &[PropDef] = &[
    team_obj!("Team1", "BRTeamInfo", BR0, 0),
    team_obj!("Team2", "BRTeamInfo", BR1, 1),
    team_obj!("Team3", "BRTeamInfo", BR2, 2),
    team_obj!("Team4", "BRTeamInfo", BR3, 3),
];

// ---- the ladder and the preferences ---------------------------------------------------------------------------------

/// `LadderController.Ladder { get; private set; }` and `PreferencesController.Preferences` (`AccessLevel.Administrator`):
/// objects with no `[CommandProperty]` members of their own, shown by their `ToString()`.
static LADDER_CONTROLLER: &[PropDef] = &[PropDef {
    name: "Ladder",
    kind: Kind::Object("Ladder", &[]),
    read: ADMIN,
    write: ADMIN,
    get: |w, e| match item(e) {
        Some(i)
            if w.item(i)
                .is_some_and(|x| x.ext.get::<crate::conpvp::ladder::Ladder>().is_some()) =>
        {
            Value::Text("Server.Engines.ConPVP.Ladder".to_string())
        }
        _ => Value::Null,
    },
    set: None,
}];

static PREFERENCES_CONTROLLER: &[PropDef] = &[PropDef {
    name: "Preferences",
    kind: Kind::Object("Preferences", &[]),
    read: ADMIN,
    write: ADMIN,
    get: |w, e| match item(e) {
        Some(i)
            if w.item(i)
                .is_some_and(|x| x.ext.get::<crate::conpvp::ladder::Preferences>().is_some()) =>
        {
            Value::Text("Server.Engines.ConPVP.Preferences".to_string())
        }
        _ => Value::Null,
    },
    set: None,
}];

// ---- the ethics Player of a PlayerMobile ----------------------------------------------------------------------------

fn ethic_player(w: &World, e: EntityId) -> Option<Serial> {
    crate::ethics::st(w)?.attached.get(&mob(e)?).copied()
}

fn ethic_data(w: &World, e: EntityId) -> Option<&crate::ethics::PlayerData> {
    crate::ethics::st(w)?.player(ethic_player(w, e)?)
}

fn ethic_set(
    w: &mut World,
    e: EntityId,
    f: impl FnOnce(&mut crate::ethics::PlayerData),
) -> Result<(), String> {
    let s = ethic_player(w, e).ok_or_else(|| "That player has no ethic.".to_string())?;
    crate::ethics::st_mut(w)
        .player_mut(s)
        .map(f)
        .ok_or_else(|| "That player has no ethic.".to_string())
}

static ETHICS_PLAYER: &[PropDef] = &[
    PropDef {
        name: "Power",
        kind: kinds::INT,
        read: GM,
        write: ADMIN,
        get: |w, e| ethic_data(w, e).map_or(Value::Null, |p| Value::Int(i64::from(p.power))),
        set: Some(|w, e, v| ethic_set(w, e, |p| p.power = v.i32())),
    },
    PropDef {
        name: "History",
        kind: kinds::INT,
        read: GM,
        write: ADMIN,
        get: |w, e| ethic_data(w, e).map_or(Value::Null, |p| Value::Int(i64::from(p.history))),
        set: Some(|w, e, v| ethic_set(w, e, |p| p.history = v.i32())),
    },
    PropDef {
        name: "Steed",
        kind: kinds::ENTITY,
        read: GM,
        write: ADMIN,
        get: |w, e| {
            ethic_data(w, e).map_or(Value::Null, |p| {
                Value::Entity(p.steed.map(EntityId::Mobile))
            })
        },
        set: Some(|w, e, v| ethic_set(w, e, |p| p.steed = v.mobile())),
    },
    PropDef {
        name: "Familiar",
        kind: kinds::ENTITY,
        read: GM,
        write: ADMIN,
        get: |w, e| {
            ethic_data(w, e).map_or(Value::Null, |p| {
                Value::Entity(p.familiar.map(EntityId::Mobile))
            })
        },
        set: Some(|w, e, v| ethic_set(w, e, |p| p.familiar = v.mobile())),
    },
    // the getter runs `FinishShield()` once the hour is over
    PropDef {
        name: "IsShielded",
        kind: kinds::BOOL,
        read: GM,
        write: GM,
        get: |w, e| match ethic_player(w, e) {
            Some(s) => Value::Bool(crate::ethics::is_shielded(w, s)),
            None => Value::Null,
        },
        set: None,
    },
];

/// `PlayerMobile.ShortTermMurders` / `PingPongs`: read from the murder context (0 when there is none), written through
/// `PlayerMurderSystem.ManuallySetShortTermMurders` / `ManuallySetPingPong`.
fn murder_int(
    w: &World,
    e: EntityId,
    f: fn(&muo_content::player_mobile::tables::MurderContext) -> i32,
) -> Value {
    let Some(m) = mob(e) else { return Value::Null };
    Value::Int(
        w.resources
            .get::<muo_content::player_mobile::tables::MurderTable>()
            .and_then(|t| t.0.get(&m))
            .map_or(0, |c| i64::from(f(c))),
    )
}

static PLAYER_MOBILE: &[PropDef] = &[
    PropDef {
        name: "ShortTermMurders",
        kind: kinds::INT,
        read: GM,
        write: GM,
        get: |w, e| murder_int(w, e, |c| c.short_term_murders),
        set: Some(|w, e, v| {
            crate::murder::manually_set_short_term_murders(
                w,
                mob(e).ok_or_else(|| "That is not a mobile.".to_string())?,
                v.i32(),
            );
            Ok(())
        }),
    },
    PropDef {
        name: "PingPongs",
        kind: kinds::INT,
        read: GM,
        write: GM,
        get: |w, e| murder_int(w, e, |c| c.ping_pongs),
        set: Some(|w, e, v| {
            crate::murder::manually_set_ping_pong(
                w,
                mob(e).ok_or_else(|| "That is not a mobile.".to_string())?,
                v.i32(),
            );
            Ok(())
        }),
    },
    PropDef {
        name: "EthicPlayer",
        kind: Kind::Object("Player", ETHICS_PLAYER),
        read: GM,
        write: GM,
        get: |w, e| {
            if ethic_player(w, e).is_some() {
                Value::Text("...".to_string())
            } else {
                Value::Null
            }
        },
        set: None,
    },
];

// ---- MLQuestContext (host: the player) -------------------------------------------------------------------------------

fn ml_flag(w: &World, e: EntityId, flag: i32) -> Value {
    match mob(e).and_then(|m| crate::ml_quests::engine::get_context(w, m)) {
        Some(c) => Value::Bool(c.get_flag(flag)),
        None => Value::Bool(false),
    }
}

fn ml_set_flag(w: &mut World, e: EntityId, flag: i32, v: &Value) -> Result<(), String> {
    let m = mob(e).ok_or_else(|| "That is not a mobile.".to_string())?;
    crate::ml_quests::engine::get_or_create_context(w, m).set_flag(flag, v.bool());
    Ok(())
}

macro_rules! ml_flag_prop {
    ($name:literal, $flag:expr) => {
        PropDef {
            name: $name,
            kind: kinds::BOOL,
            read: GM,
            write: GM,
            get: |w, e| ml_flag(w, e, $flag),
            set: Some(|w, e, v| ml_set_flag(w, e, $flag, &v)),
        }
    };
}

static ML_QUEST_CONTEXT: &[PropDef] = &[
    PropDef {
        name: "IsFull",
        kind: kinds::BOOL,
        read: GM,
        write: GM,
        get: |w, e| {
            Value::Bool(
                mob(e)
                    .and_then(|m| crate::ml_quests::engine::get_context(w, m))
                    .is_some_and(|c| c.is_full()),
            )
        },
        set: None,
    },
    ml_flag_prop!("Spellweaving", crate::ml_quests::model::flag::SPELLWEAVING),
    ml_flag_prop!("SummonFey", crate::ml_quests::model::flag::SUMMON_FEY),
    ml_flag_prop!("SummonFiend", crate::ml_quests::model::flag::SUMMON_FIEND),
    ml_flag_prop!("BedlamAccess", crate::ml_quests::model::flag::BEDLAM_ACCESS),
];

// ---- the rest -------------------------------------------------------------------------------------------------------

/// `SpawnerBorder.Spawner { get; set; }` (`[CommandProperty(AccessLevel.Developer, readOnly: true)]`): the spawner whose
/// borders these are, found through the `[ShowSpawnerBorders` table (the border keeps no back link).
static SPAWNER_BORDER: &[PropDef] = &[PropDef {
    name: "Spawner",
    kind: kinds::ENTITY,
    read: AccessLevel::Developer,
    write: AccessLevel::Developer,
    get: |w, e| match item(e) {
        Some(b) => ent_item(
            w.resources
                .get::<crate::spawners::commands::Projections>()
                .and_then(|p| {
                    p.spawners
                        .iter()
                        .find(|(_, borders)| borders.contains(&b))
                        .map(|(s, _)| *s)
                }),
        ),
        None => Value::Null,
    },
    set: None,
}];

/// `BulkOrderBook.Filter` (`BOBFilter`, a `[PropertyObject]`-less class: shown by its `ToString()`).
static BULK_ORDER_BOOK: &[PropDef] = &[PropDef {
    name: "Filter",
    kind: Kind::Object("BOBFilter", &[]),
    read: GM,
    write: GM,
    get: |w, e| match item(e) {
        Some(i) if crate::bulk_orders::book::book(w, i).is_some() => {
            Value::Text("Server.Engines.BulkOrders.BOBFilter".to_string())
        }
        _ => Value::Null,
    },
    set: None,
}];

/// `MLQuestTeleporter.Message` and `TicketTeleporter.Message`.
static QUEST_TELEPORTER_MESSAGE: &[PropDef] = &[muo_content::text_def_prop!(
    "Message",
    GM,
    GM,
    |w: &World, e: EntityId| item(e)
        .and_then(|i| w.item(i))
        .and_then(|o| o.ext.get::<crate::ml_quests::items2::QuestTeleporterData>())
        .and_then(|d| d.message.clone()),
    Some(|w: &mut World, e: EntityId, v: Value| {
        if let Some(i) = item(e)
            && let Some(d) = w.item_mut(i).and_then(|o| {
                o.ext
                    .get_mut::<crate::ml_quests::items2::QuestTeleporterData>()
            })
        {
            d.message = match v {
                Value::TextDef(t) => Some(t),
                _ => None,
            };
        }
        Ok(())
    })
)];

/// `MLQuestTeleporter.QuestType` / `TicketTeleporter.TicketType` (`Type`, `[InvalidateProperties]`).
macro_rules! type_prop {
    ($name:literal, $quest:literal) => {
        PropDef {
            name: $name,
            kind: Kind::Plain(muo_content::utilities::types::PropType::Type),
            read: GM,
            write: GM,
            get: |w, e| {
                item(e).map_or(Value::Null, |i| {
                    crate::ml_quests::items2::type_prop_get(w, i, $quest)
                })
            },
            set: Some(|w, e, v| match item(e) {
                Some(i) => crate::ml_quests::items2::type_prop_set(w, i, $quest, &v),
                None => Ok(()),
            }),
        }
    };
}

static MLQUEST_TELEPORTER_TYPE: &[PropDef] = &[type_prop!("QuestType", true)];
static TICKET_TELEPORTER_TYPE: &[PropDef] = &[type_prop!("TicketType", false)];

/// `HillOfTheKing.m_KingTimer` (a `[CommandProperty]` field): a `Timer` shows its type's name, null until the first king.
static HILL_OF_THE_KING: &[PropDef] = &[PropDef {
    name: "KingTimer",
    kind: Kind::Text,
    read: GM,
    write: GM,
    get: |w, e| match item(e)
        .and_then(|i| w.item(i))
        .and_then(|o| o.ext.get::<crate::conpvp::koth::HillData>())
    {
        Some(d) if d.has_timer => {
            Value::Text("Server.Engines.ConPVP.HillOfTheKing+KingTimer".to_string())
        }
        _ => Value::Null,
    },
    set: None,
}];

pub fn configure(w: &mut World) {
    props::add(w, "Server.Engines.ConPVP.HillOfTheKing", HILL_OF_THE_KING);
    props::add(w, "Server.Engines.Spawners.SpawnerBorder", SPAWNER_BORDER);
    props::add(
        w,
        "Server.Engines.BulkOrders.BulkOrderBook",
        BULK_ORDER_BOOK,
    );
    props::add(
        w,
        "Server.Engines.MLQuests.Items.MLQuestTeleporter",
        QUEST_TELEPORTER_MESSAGE,
    );
    props::add(
        w,
        "Server.Engines.MLQuests.Items.MLQuestTeleporter",
        MLQUEST_TELEPORTER_TYPE,
    );
    props::add(
        w,
        "Server.Engines.MLQuests.Items.TicketTeleporter",
        TICKET_TELEPORTER_TYPE,
    );
    props::add(
        w,
        "Server.Engines.MLQuests.Items.TicketTeleporter",
        QUEST_TELEPORTER_MESSAGE,
    );
    props::add(w, "Server.Engines.ConPVP.CTFController", CTF_CONTROLLER);
    props::add(w, "Server.Engines.ConPVP.DDController", DD_CONTROLLER);
    props::add(w, "Server.Engines.ConPVP.KHController", KH_CONTROLLER);
    props::add(w, "Server.Engines.ConPVP.BRController", BR_CONTROLLER);
    props::add(
        w,
        "Server.Engines.ConPVP.LadderController",
        LADDER_CONTROLLER,
    );
    props::add(
        w,
        "Server.Engines.ConPVP.PreferencesController",
        PREFERENCES_CONTROLLER,
    );
    // `LadderEntry` is opened by the duel speech through `SubObj` (conpvp/speech.rs), over the mobile it belongs to
    props::add(w, "LadderEntry", crate::conpvp::props::LADDER_ENTRY_PROPS);
    props::add(w, "Server.Mobiles.PlayerMobile", PLAYER_MOBILE);
    // no property returns an `MLQuestContext`: its members are read over the player (as `LadderEntry`)
    props::add(w, "MLQuestContext", ML_QUEST_CONTEXT);
}

#[allow(dead_code)]
fn _unused(_: Point3D) {}
