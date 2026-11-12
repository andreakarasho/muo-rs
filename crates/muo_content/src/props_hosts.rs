//! `Mobile`'s `[CommandProperty]` members that read engine state (`Weapon`, `Mount`, `Mounted`, `Race`, `NetState`,
//! `Account`, `Last*Gain`) and the `Account` object a mobile hosts (`[PropertyObject]`: `Account.Email`..).
//!
//! An object that is not an entity (`Account`, `NetState`, `Guild`) has no `EntityId`: its table is a `Kind::Object`
//! whose getters and setters take the *host* entity, here the mobile whose `Account` it is;
//! `Guildstone.Guild` is hosted by the stone in muo_items.

use muo_server::ids::{EntityId, MobileId};
use muo_server::interfaces::WeaponRef;
use muo_server::mobile::AccessLevel;
use muo_server::mobile::part1::{props_a, props_b};
use muo_server::race::RaceId;
use muo_server::world::World;

use crate::accounting;
use crate::props::{self, Kind, PropDef, Value};
use crate::utilities::types::{Numeric, PropType};

const GM: AccessLevel = AccessLevel::GameMaster;
const ADMIN: AccessLevel = AccessLevel::Administrator;
const OWNER: AccessLevel = AccessLevel::Owner;
const INT: Kind = Kind::Plain(PropType::Numeric(Numeric::Int));
const RACES: &[(&str, i64)] = &[("Human", 0), ("Elf", 1), ("Gargoyle", 2)];

fn mob(e: EntityId) -> Option<MobileId> {
    match e {
        EntityId::Mobile(m) => Some(m),
        EntityId::Item(_) => None,
    }
}

fn account_of(w: &World, e: EntityId) -> Option<&accounting::Account> {
    let id = w.mobile(mob(e)?)?.account?;
    accounting::account(w, id)
}

fn account_mut_of(w: &mut World, e: EntityId) -> Option<&mut accounting::Account> {
    let id = w.mobile(mob(e)?)?.account?;
    accounting::account_mut(w, id)
}

/// `Account`'s `[CommandProperty]` members (`TotalGold` / `TotalPlat` / `Email` are `[SerializedCommandProperty]`
/// fields with a private generated setter: a write still goes through reflection).
pub static ACCOUNT: &[PropDef] = &[
    PropDef {
        name: "TotalGold",
        kind: INT,
        read: ADMIN,
        write: ADMIN,
        get: |w, e| account_of(w, e).map_or(Value::Null, |a| Value::Int(i64::from(a.total_gold))),
        set: Some(|w, e, v| {
            account_mut_of(w, e)
                .map(|a| a.total_gold = v.i32())
                .ok_or_else(|| "That mobile has no account.".to_string())
        }),
    },
    PropDef {
        name: "TotalPlat",
        kind: INT,
        read: ADMIN,
        write: ADMIN,
        get: |w, e| account_of(w, e).map_or(Value::Null, |a| Value::Int(i64::from(a.total_plat))),
        set: Some(|w, e, v| {
            account_mut_of(w, e)
                .map(|a| a.total_plat = v.i32())
                .ok_or_else(|| "That mobile has no account.".to_string())
        }),
    },
    PropDef {
        name: "Email",
        kind: Kind::Plain(PropType::Str),
        read: ADMIN,
        write: ADMIN,
        get: |w, e| {
            account_of(w, e).map_or(Value::Null, |a| {
                a.email.clone().map_or(Value::Null, Value::Str)
            })
        },
        set: Some(|w, e, v| {
            account_mut_of(w, e)
                .map(|a| a.email = v.string())
                .ok_or_else(|| "That mobile has no account.".to_string())
        }),
    },
    // `[CommandProperty(AccessLevel.GameMaster, readOnly: true)]`
    PropDef {
        name: "Created",
        kind: Kind::DateTime,
        read: GM,
        write: GM,
        get: |w, e| account_of(w, e).map_or(Value::Null, |a| Value::DateTime(a.created)),
        set: None,
    },
];

/// `NetState` has no `[CommandProperty]` members of its own.
pub static NET_STATE: &[PropDef] = &[];

macro_rules! gain {
    ($name:literal, $f:ident) => {
        PropDef {
            name: $name,
            kind: Kind::DateTime,
            read: GM,
            write: GM,
            get: |w, e| {
                mob(e)
                    .and_then(|m| w.mobile(m))
                    .map_or(Value::Null, |x| Value::DateTime(x.$f))
            },
            set: Some(|w, e, v| {
                let (Some(m), Value::DateTime(t)) = (mob(e).and_then(|m| w.mobile_mut(m)), v)
                else {
                    return Err("That is not properly formatted.".to_string());
                };
                m.$f = t;
                Ok(())
            }),
        }
    };
}

static MOBILE_MORE: &[PropDef] = &[
    // `m_Race ??= Race.DefaultRace`; the setter changes the body and hue (`Mobile.Race.set`)
    PropDef {
        name: "Race",
        kind: Kind::Plain(PropType::Enum(RACES)),
        read: GM,
        write: GM,
        get: |w, e| match mob(e).and_then(|m| w.mobile(m)) {
            Some(x) => Value::Enum(i64::from(x.race.unwrap_or(RaceId(0)).0)),
            None => Value::Null,
        },
        set: Some(|w, e, v| {
            let Some(m) = mob(e) else {
                return Err("That is not a mobile.".to_string());
            };
            props_a::set_race(w, m, Some(RaceId(v.i64() as u8)));
            Ok(())
        }),
    },
    // `[CommandProperty(AccessLevel.GameMaster, AccessLevel.Administrator, canModify: true)] IAccount Account { get; set; }`
    PropDef {
        name: "Account",
        kind: Kind::Object("Account", ACCOUNT),
        read: GM,
        write: ADMIN,
        get: |w, e| match account_of(w, e) {
            Some(a) => Value::Text(a.username.clone()),
            None => Value::Null,
        },
        set: None,
    },
    // `m_NetState is { IsConnected: true } ? m_NetState : null`, `ToString()` is the address
    PropDef {
        name: "NetState",
        kind: Kind::Object("NetState", NET_STATE),
        read: GM,
        write: OWNER,
        get: |w, e| {
            let ns = mob(e)
                .and_then(|m| w.mobile(m))
                .and_then(|x| x.net_state)
                .and_then(|id| w.net_state(id));
            match ns {
                Some(d) if !d.cannot_send => Value::Text(d.description.clone()),
                _ => Value::Null,
            }
        },
        set: None,
    },
    gain!("LastStrGain", last_str_gain),
    gain!("LastIntGain", last_int_gain),
    gain!("LastDexGain", last_dex_gain),
    // `IWeapon`: the weapon item (an entity), or `Mobile.DefaultWeapon` (fists)
    PropDef {
        name: "Weapon",
        kind: Kind::Plain(PropType::Entity),
        read: GM,
        write: GM,
        get: |w, e| match mob(e).and_then(|m| muo_server::mobile::base::weapon(w, m)) {
            Some(WeaponRef::Item(i)) => Value::Entity(Some(EntityId::Item(i))),
            Some(WeaponRef::Default) => Value::Text("Server.Items.Fists".to_string()),
            None => Value::Null,
        },
        set: None,
    },
    PropDef {
        name: "Mount",
        kind: Kind::Plain(PropType::Entity),
        read: GM,
        write: GM,
        get: |w, e| match mob(e).and_then(|m| props_b::mount(w, m)) {
            Some(m) => Value::Entity(Some(EntityId::Mobile(m))),
            None => Value::Null,
        },
        set: None,
    },
    PropDef {
        name: "Mounted",
        kind: Kind::Plain(PropType::Bool),
        read: GM,
        write: GM,
        get: |w, e| Value::Bool(mob(e).is_some_and(|m| props_b::mount(w, m).is_some())),
        set: None,
    },
];

pub fn register(w: &mut World) {
    props::add(w, "Server.Mobile", MOBILE_MORE);
}
