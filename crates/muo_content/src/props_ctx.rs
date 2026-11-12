//! `[CommandProperty]` objects a `PlayerMobile` returns: `Virtues` (`VirtueContext`), `ChampionTitles`
//! (`ChampionTitleContext`) and the two members that read the murder context (`ShortTermMurders`, `PingPongs`). An object that is
//! not an entity is a `Kind::Object` whose sub-properties take the *host*, here the player.
//! The getters create the context when there is none (`GetOrCreateVirtues`), which is why a getter takes `&mut World`.

use muo_primitives::{DateTime, TimeSpan};
use muo_server::ids::{EntityId, MobileId};
use muo_server::mobile::AccessLevel;
use muo_server::world::World;

use crate::player_mobile::tables::{self, ChampionTitle, MurderTable};
use crate::props::{self, Kind, PropDef, Value, kinds};

const CNS: AccessLevel = AccessLevel::Counselor;
const GM: AccessLevel = AccessLevel::GameMaster;
const ADMIN: AccessLevel = AccessLevel::Administrator;

fn mob(e: EntityId) -> Option<MobileId> {
    match e {
        EntityId::Mobile(m) => Some(m),
        EntityId::Item(_) => None,
    }
}

fn dt(v: &Value) -> DateTime {
    match v {
        Value::DateTime(t) => *t,
        _ => DateTime::MIN,
    }
}

fn ts(v: &Value) -> TimeSpan {
    match v {
        Value::TimeSpan(t) => *t,
        _ => TimeSpan::ZERO,
    }
}

// ---- VirtueContext (VirtueSystem.GetOrCreateVirtues) ----

macro_rules! virtue {
    ($name:literal, $kind:expr, $rd:expr, $wr:expr, |$c:ident| $get:expr, |$c2:ident, $v:ident| $set:expr) => {
        PropDef {
            name: $name,
            kind: $kind,
            read: $rd,
            write: $wr,
            get: |w, e| match mob(e) {
                Some(m) => {
                    let $c = tables::virtues_mut(w, m);
                    $get
                }
                None => Value::Null,
            },
            set: Some(|w, e, $v| {
                let m = mob(e).ok_or_else(|| "That is not a mobile.".to_string())?;
                let $c2 = tables::virtues_mut(w, m);
                $set;
                Ok(())
            }),
        }
    };
}

macro_rules! virtue_value {
    ($name:literal, $i:literal) => {
        virtue!(
            $name,
            kinds::INT,
            CNS,
            GM,
            |c| Value::Int(c.values.as_ref().map_or(0, |v| i64::from(v[$i]))),
            |c, v| c.set_value($i, v.i32())
        )
    };
}

static VIRTUE_SUBS: &[PropDef] = &[
    virtue!(
        "LastSacrificeGain",
        Kind::DateTime,
        GM,
        ADMIN,
        |c| Value::DateTime(c.last_sacrifice_gain),
        |c, v| c.last_sacrifice_gain = dt(&v)
    ),
    virtue!(
        "LastSacrificeLoss",
        Kind::DateTime,
        GM,
        ADMIN,
        |c| Value::DateTime(c.last_sacrifice_loss),
        |c, v| c.last_sacrifice_loss = dt(&v)
    ),
    virtue!(
        "AvailableResurrects",
        kinds::INT,
        GM,
        GM,
        |c| Value::Int(i64::from(c.available_resurrects)),
        |c, v| c.available_resurrects = v.i32()
    ),
    virtue!(
        "LastJusticeLoss",
        Kind::DateTime,
        GM,
        ADMIN,
        |c| Value::DateTime(c.last_justice_loss),
        |c, v| c.last_justice_loss = dt(&v)
    ),
    virtue!(
        "LastCompassionLoss",
        Kind::DateTime,
        GM,
        ADMIN,
        |c| Value::DateTime(c.last_compassion_loss),
        |c, v| c.last_compassion_loss = dt(&v)
    ),
    virtue!(
        "NextCompassionDay",
        Kind::DateTime,
        GM,
        GM,
        |c| Value::DateTime(c.next_compassion_day),
        |c, v| c.next_compassion_day = dt(&v)
    ),
    virtue!(
        "CompassionGains",
        kinds::INT,
        GM,
        GM,
        |c| Value::Int(i64::from(c.compassion_gains)),
        |c, v| c.compassion_gains = v.i32()
    ),
    virtue!(
        "LastValorLoss",
        Kind::DateTime,
        GM,
        ADMIN,
        |c| Value::DateTime(c.last_valor_loss),
        |c, v| c.last_valor_loss = dt(&v)
    ),
    virtue!(
        "LastHonorUse",
        Kind::DateTime,
        GM,
        GM,
        |c| Value::DateTime(c.last_honor_use),
        |c, v| c.last_honor_use = dt(&v)
    ),
    virtue!(
        "HonorActive",
        kinds::BOOL,
        GM,
        ADMIN,
        |c| Value::Bool(c.honor_active),
        |c, v| c.honor_active = v.bool()
    ),
    virtue_value!("Humility", 0),
    virtue_value!("Sacrifice", 1),
    virtue_value!("Compassion", 2),
    virtue_value!("Spirituality", 3),
    virtue_value!("Valor", 4),
    virtue_value!("Honor", 5),
    virtue_value!("Justice", 6),
    virtue_value!("Honesty", 7),
];

// ---- ChampionTitleContext (ChampionTitleSystem.GetOrCreateChampionTitleContext) ----

macro_rules! champion {
    ($name:literal, $kind:expr, |$c:ident| $get:expr, |$w:ident, $m:ident, $c2:ident, $v:ident| $set:expr) => {
        PropDef {
            name: $name,
            kind: $kind,
            read: GM,
            write: GM,
            get: |w, e| match mob(e) {
                Some(m) => {
                    let $c = tables::champion_titles_mut(w, m);
                    $get
                }
                None => Value::Null,
            },
            set: Some(|$w, e, $v| {
                let $m = mob(e).ok_or_else(|| "That is not a mobile.".to_string())?;
                $set;
                Ok(())
            }),
        }
    };
}

/// `SetValue(type, value)`: no title and `value <= 0` stays none, `value <= 0` removes it (and invalidates the player),
/// else the title is made and stamped `Core.Now`.
fn set_title_value(w: &mut World, m: MobileId, slot: usize, value: i32) {
    let now = w.now;
    let c = tables::champion_titles_mut(w, m);
    match &mut c.titles[slot] {
        Some(_) if value <= 0 => c.titles[slot] = None,
        Some(t) => {
            t.value = value;
            t.last_decay = now;
        }
        None => {
            c.titles[slot] = Some(ChampionTitle {
                value,
                last_decay: now,
            })
        }
    }
    muo_server::mobile::part2::properties::invalidate_properties(w, m);
}

/// `SetLastDecay(type, value)`: makes the title when there is none.
fn set_title_decay(w: &mut World, m: MobileId, slot: usize, value: DateTime) {
    let c = tables::champion_titles_mut(w, m);
    c.titles[slot]
        .get_or_insert(ChampionTitle {
            value: 0,
            last_decay: DateTime::MIN,
        })
        .last_decay = value;
}

macro_rules! title_value {
    ($name:literal, $slot:literal) => {
        champion!(
            $name,
            kinds::INT,
            |c| Value::Int(c.titles[$slot].as_ref().map_or(0, |t| i64::from(t.value))),
            |w, m, c, v| set_title_value(w, m, $slot, v.i32())
        )
    };
}

macro_rules! title_decay {
    ($name:literal, $slot:literal) => {
        champion!(
            $name,
            Kind::DateTime,
            |c| Value::DateTime(
                c.titles[$slot]
                    .as_ref()
                    .map_or(DateTime::MIN, |t| t.last_decay)
            ),
            |w, m, c, v| set_title_decay(w, m, $slot, dt(&v))
        )
    };
}

// The slots are the field order: Abyss, Arachnid, ColdBlood, ForestLord, VerminHorde, UnholyTerror, SleepingDragon, Corrupt, Glade.
static CHAMPION_SUBS: &[PropDef] = &[
    champion!(
        "Harrower",
        kinds::INT,
        |c| Value::Int(i64::from(c.harrower)),
        |w, m, c, v| tables::champion_titles_mut(w, m).harrower = v.i32()
    ),
    title_value!("AbyssValue", 0),
    title_decay!("AbyssLastDecay", 0),
    title_value!("ArachnidValue", 1),
    title_decay!("ArachnidLastDecay", 1),
    title_value!("ColdBloodValue", 2),
    title_decay!("ColdBloodLastDecay", 2),
    title_value!("ForestLordValue", 3),
    title_decay!("ForestLordLastDecay", 3),
    title_value!("VerminHordeValue", 4),
    title_decay!("VerminHordeLastDecay", 4),
    title_value!("UnholyTerrorValue", 5),
    title_decay!("UnholyTerrorLastDecay", 5),
    title_value!("SleepingDragonValue", 6),
    title_decay!("SleepingDragonLastDecay", 6),
    title_value!("CorruptValue", 7),
    title_decay!("CorruptLastDecay", 7),
    title_value!("GladeValue", 8),
    title_decay!("GladeLastDecay", 8),
];

// ---- MurderContext (PlayerMurderSystem.GetMurderContext / GetOrCreateMurderContext) ----

fn murder<T>(w: &World, m: MobileId, f: impl FnOnce(&tables::MurderContext) -> T, none: T) -> T {
    w.resources
        .get::<MurderTable>()
        .and_then(|t| t.0.get(&m))
        .map_or(none, f)
}

fn murder_mut(w: &mut World, m: MobileId) -> &mut tables::MurderContext {
    w.resources.get_mut::<MurderTable>().0.entry(m).or_default()
}

macro_rules! murder_prop {
    ($name:literal, $kind:expr, $none:expr, |$c:ident| $get:expr, |$c2:ident, $v:ident| $set:expr) => {
        PropDef {
            name: $name,
            kind: $kind,
            read: GM,
            write: GM,
            get: |w, e| match mob(e) {
                Some(m) => murder(w, m, |$c| $get, $none),
                None => Value::Null,
            },
            set: Some(|w, e, $v| {
                let m = mob(e).ok_or_else(|| "That is not a mobile.".to_string())?;
                let $c2 = murder_mut(w, m);
                $set;
                Ok(())
            }),
        }
    };
}

// `AllowShortTermMurdersChange`: `value = Math.Max(value, 0)`
macro_rules! short_term {
    () => {
        murder_prop!(
            "ShortTermMurders",
            kinds::INT,
            Value::Int(0),
            |c| Value::Int(i64::from(c.short_term_murders)),
            |c, v| {
                let n = v.i32();
                if n != c.short_term_murders {
                    c.short_term_murders = n.max(0);
                }
            }
        )
    };
}

macro_rules! ping_pongs {
    () => {
        murder_prop!(
            "PingPongs",
            kinds::INT,
            Value::Int(0),
            |c| Value::Int(i64::from(c.ping_pongs)),
            |c, v| c.ping_pongs = v.i32()
        )
    };
}

/// `MurderContext`'s own members (what `[CommandProperty]` shows for the context object).
static MURDER_SUBS: &[PropDef] = &[
    murder_prop!(
        "ShortTermElapse",
        Kind::TimeSpan,
        Value::TimeSpan(TimeSpan::ZERO),
        |c| Value::TimeSpan(c.short_term_elapse),
        |c, v| c.short_term_elapse = ts(&v)
    ),
    murder_prop!(
        "LongTermElapse",
        Kind::TimeSpan,
        Value::TimeSpan(TimeSpan::ZERO),
        |c| Value::TimeSpan(c.long_term_elapse),
        |c, v| c.long_term_elapse = ts(&v)
    ),
    short_term!(),
    ping_pongs!(),
    murder_prop!(
        "Bounty",
        kinds::INT,
        Value::Int(0),
        |c| Value::Int(i64::from(c.bounty)),
        |c, v| c.bounty = v.i32()
    ),
    murder_prop!(
        "LastMurderTime",
        Kind::DateTime,
        Value::DateTime(DateTime::MIN),
        |c| Value::DateTime(c.last_murder_time),
        |c, v| c.last_murder_time = dt(&v)
    ),
];

macro_rules! pm_murder_prop {
    ($name:literal, $field:ident, $hook:ident) => {
        PropDef {
            name: $name,
            kind: kinds::INT,
            read: GM,
            write: GM,
            get: |w, e| match mob(e) {
                Some(m) => murder(w, m, |c| Value::Int(i64::from(c.$field)), Value::Int(0)),
                None => Value::Null,
            },
            set: Some(|w, e, v| {
                let m = mob(e).ok_or_else(|| "That is not a mobile.".to_string())?;
                crate::player_mobile::hooks::$hook(w, m, v.i32());
                Ok(())
            }),
        }
    };
}

static PLAYER_MOBILE_OBJECTS: &[PropDef] = &[
    PropDef {
        name: "Virtues",
        kind: Kind::Object("VirtueContext", VIRTUE_SUBS),
        read: CNS,
        write: CNS,
        get: |w, e| match mob(e) {
            Some(m) => {
                tables::virtues_mut(w, m);
                Value::Text("...".to_string())
            }
            None => Value::Null,
        },
        set: None,
    },
    PropDef {
        name: "ChampionTitles",
        kind: Kind::Object("ChampionTitleContext", CHAMPION_SUBS),
        read: GM,
        write: GM,
        get: |w, e| match mob(e) {
            Some(m) => {
                tables::champion_titles_mut(w, m);
                Value::Text("...".to_string())
            }
            None => Value::Null,
        },
        set: None,
    },
    // `PlayerMobile.ShortTermMurders`/`PingPongs` set through the murder system (its update, kill timer reset).
    pm_murder_prop!(
        "ShortTermMurders",
        short_term_murders,
        manually_set_short_term_murders
    ),
    pm_murder_prop!("PingPongs", ping_pongs, manually_set_ping_pong),
];

pub fn configure(w: &mut World) {
    props::add(w, "Server.Mobiles.PlayerMobile", PLAYER_MOBILE_OBJECTS);
    props::add(w, "MurderContext", MURDER_SUBS);
}
