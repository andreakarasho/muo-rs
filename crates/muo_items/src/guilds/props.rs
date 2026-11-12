//! `Guildstone.Guild` / `GuildstoneDeed.Guild`: the `Guild` object (`BaseGuild` and `Guild` `[CommandProperty]` members) a
//! stone hosts. `Guild` is not an entity, so its table is a `Kind::Object` whose getters and setters take the *stone*,
//! which finds its guild.

use muo_content::misc::guild_class as gc;
use muo_content::props::{Kind, PropDef, Value};
use muo_content::utilities::types::{Numeric, PropType};
use muo_server::guild::{self, GuildId, GuildType};
use muo_server::ids::EntityId;
use muo_server::mobile::AccessLevel;
use muo_server::world::World;

const CNS: AccessLevel = AccessLevel::Counselor;
const GM: AccessLevel = AccessLevel::GameMaster;
const STR: Kind = Kind::Plain(PropType::Str);
const GUILD_TYPES: &[(&str, i64)] = &[("Regular", 0), ("Chaos", 1), ("Order", 2)];

/// The guild a table call is about: the guild of the stone that hosts it, or (a guild whose stone is gone, shown by
/// `[GuildProps` over the viewer) the one `props::sub_key` names.
fn of(w: &World, e: EntityId) -> Option<GuildId> {
    match e {
        EntityId::Item(i) => super::stone_guild(w, i),
        EntityId::Mobile(_) => match muo_content::props::sub_key(w) {
            usize::MAX => None,
            k => guild::find_guild(w, muo_primitives::Serial::new(k as u32)),
        },
    }
}

fn text(s: Option<String>) -> Value {
    s.map_or(Value::Null, Value::Str)
}

pub static GUILD: &[PropDef] = &[
    PropDef {
        name: "Serial",
        kind: Kind::Plain(PropType::Serial),
        read: CNS,
        write: CNS,
        get: |w, e| {
            of(w, e)
                .and_then(|g| guild::guild(w, g))
                .map_or(Value::Null, |b| Value::UInt(u64::from(b.serial.value())))
        },
        set: None,
    },
    // `[CommandProperty(AccessLevel.GameMaster, readOnly: true)]`
    PropDef {
        name: "Created",
        kind: Kind::DateTime,
        read: GM,
        write: GM,
        get: |w, e| {
            of(w, e)
                .and_then(|g| guild::guild(w, g))
                .map_or(Value::Null, |b| Value::DateTime(b.created))
        },
        set: None,
    },
    PropDef {
        name: "Leader",
        kind: Kind::Plain(PropType::Entity),
        read: GM,
        write: GM,
        // the getter runs `CalculateGuildmaster()` for a disbanded guild
        get: |w, e| match of(w, e) {
            Some(g) => Value::Entity(gc::leader(w, g).map(EntityId::Mobile)),
            None => Value::Null,
        },
        set: Some(|w, e, v| {
            let g = of(w, e).ok_or_else(|| "That stone has no guild.".to_string())?;
            gc::set_leader(w, g, v.mobile());
            Ok(())
        }),
    },
    PropDef {
        name: "AllianceName",
        kind: STR,
        read: CNS,
        write: CNS,
        get: |w, e| {
            text(
                of(w, e)
                    .and_then(|g| gc::alliance(w, g))
                    .map(|k| gc::alliance_name(w, k)),
            )
        },
        set: None,
    },
    PropDef {
        name: "AllianceLeader",
        kind: Kind::Text,
        read: CNS,
        write: CNS,
        get: |w, e| match of(w, e)
            .and_then(|g| gc::alliance(w, g))
            .and_then(|k| gc::alliance_leader(w, k))
        {
            Some(l) => Value::Text(gc::name(w, l)),
            None => Value::Null,
        },
        set: None,
    },
    PropDef {
        name: "IsAllianceMember",
        kind: Kind::Plain(PropType::Bool),
        read: CNS,
        write: CNS,
        get: |w, e| match of(w, e) {
            Some(g) => {
                Value::Bool(gc::alliance(w, g).is_some_and(|k| gc::alliance_is_member(w, k, g)))
            }
            None => Value::Null,
        },
        set: None,
    },
    PropDef {
        name: "IsAlliancePendingMember",
        kind: Kind::Plain(PropType::Bool),
        read: CNS,
        write: CNS,
        get: |w, e| match of(w, e) {
            Some(g) => Value::Bool(
                gc::alliance(w, g).is_some_and(|k| gc::alliance_is_pending_member(w, k, g)),
            ),
            None => Value::Null,
        },
        set: None,
    },
    PropDef {
        name: "Guildstone",
        kind: Kind::Plain(PropType::Entity),
        read: GM,
        write: GM,
        get: |w, e| match of(w, e).and_then(|g| gc::data(w, g)) {
            Some(d) => Value::Entity(d.guildstone.map(EntityId::Item)),
            None => Value::Null,
        },
        set: Some(|w, e, v| {
            let g = of(w, e).ok_or_else(|| "That stone has no guild.".to_string())?;
            if let Some(d) = gc::data_mut(w, g) {
                d.guildstone = match v.entity() {
                    Some(EntityId::Item(i)) => Some(i),
                    _ => None,
                };
            }
            Ok(())
        }),
    },
    PropDef {
        name: "Teleporter",
        kind: Kind::Plain(PropType::Entity),
        read: GM,
        write: GM,
        get: |w, e| match of(w, e).and_then(|g| gc::data(w, g)) {
            Some(d) => Value::Entity(d.teleporter.map(EntityId::Item)),
            None => Value::Null,
        },
        set: Some(|w, e, v| {
            let g = of(w, e).ok_or_else(|| "That stone has no guild.".to_string())?;
            if let Some(d) = gc::data_mut(w, g) {
                d.teleporter = match v.entity() {
                    Some(EntityId::Item(i)) => Some(i),
                    _ => None,
                };
            }
            Ok(())
        }),
    },
    // `Name = value`: `InvalidateMemberProperties(true)`, `Guildstone?.InvalidateProperties()`
    PropDef {
        name: "Name",
        kind: STR,
        read: GM,
        write: GM,
        get: |w, e| of(w, e).map_or(Value::Null, |g| Value::Str(gc::name(w, g))),
        set: Some(|w, e, v| {
            let g = of(w, e).ok_or_else(|| "That stone has no guild.".to_string())?;
            gc::set_name(w, g, &v.string().unwrap_or_default());
            Ok(())
        }),
    },
    PropDef {
        name: "Website",
        kind: STR,
        read: GM,
        write: GM,
        get: |w, e| {
            text(
                of(w, e)
                    .and_then(|g| gc::data(w, g))
                    .and_then(|d| d.website.clone()),
            )
        },
        set: Some(|w, e, v| {
            let g = of(w, e).ok_or_else(|| "That stone has no guild.".to_string())?;
            if let Some(d) = gc::data_mut(w, g) {
                d.website = v.string();
            }
            Ok(())
        }),
    },
    PropDef {
        name: "Abbreviation",
        kind: STR,
        read: GM,
        write: GM,
        get: |w, e| {
            of(w, e)
                .and_then(|g| guild::guild(w, g))
                .map_or(Value::Null, |b| Value::Str(b.abbreviation.clone()))
        },
        set: Some(|w, e, v| {
            let g = of(w, e).ok_or_else(|| "That stone has no guild.".to_string())?;
            gc::set_abbreviation(w, g, &v.string().unwrap_or_default());
            Ok(())
        }),
    },
    PropDef {
        name: "Charter",
        kind: STR,
        read: GM,
        write: GM,
        get: |w, e| {
            text(
                of(w, e)
                    .and_then(|g| gc::data(w, g))
                    .and_then(|d| d.charter.clone()),
            )
        },
        set: Some(|w, e, v| {
            let g = of(w, e).ok_or_else(|| "That stone has no guild.".to_string())?;
            if let Some(d) = gc::data_mut(w, g) {
                d.charter = v.string();
            }
            Ok(())
        }),
    },
    // `OrderChaos ? m_Type : GuildType.Regular`; the setter stamps `TypeLastChange`
    PropDef {
        name: "Type",
        kind: Kind::Plain(PropType::Enum(GUILD_TYPES)),
        read: GM,
        write: GM,
        get: |w, e| of(w, e).map_or(Value::Null, |g| Value::Enum(gc::guild_type(w, g) as i64)),
        set: Some(|w, e, v| {
            let g = of(w, e).ok_or_else(|| "That stone has no guild.".to_string())?;
            gc::set_type(
                w,
                g,
                match v.i64() {
                    1 => GuildType::Chaos,
                    2 => GuildType::Order,
                    _ => GuildType::Regular,
                },
            );
            Ok(())
        }),
    },
    PropDef {
        name: "LastFealty",
        kind: Kind::DateTime,
        read: GM,
        write: GM,
        get: |w, e| {
            of(w, e)
                .and_then(|g| gc::data(w, g))
                .map_or(Value::Null, |d| Value::DateTime(d.last_fealty))
        },
        set: Some(|w, e, v| {
            let g = of(w, e).ok_or_else(|| "That stone has no guild.".to_string())?;
            if let (Some(d), Value::DateTime(t)) = (gc::data_mut(w, g), v) {
                d.last_fealty = t;
            }
            Ok(())
        }),
    },
    // `{ get; private set; }`: a reflection write still reaches it
    PropDef {
        name: "TypeLastChange",
        kind: Kind::DateTime,
        read: GM,
        write: GM,
        get: |w, e| {
            of(w, e)
                .and_then(|g| gc::data(w, g))
                .map_or(Value::Null, |d| Value::DateTime(d.type_last_change))
        },
        set: Some(|w, e, v| {
            let g = of(w, e).ok_or_else(|| "That stone has no guild.".to_string())?;
            if let (Some(d), Value::DateTime(t)) = (gc::data_mut(w, g), v) {
                d.type_last_change = t;
            }
            Ok(())
        }),
    },
];

fn guild_object(w: &mut World, e: EntityId) -> Value {
    match of(w, e).and_then(|g| guild::guild(w, g)) {
        Some(b) => Value::Text(b.display()),
        None => Value::Null,
    }
}

/// `[SerializedCommandProperty(AccessLevel.GameMaster)] Guild` of the stone and its deed (`setter: "private"`: a
/// reflection write is not offered here, a `Guild` is not parsable).
pub static STONE: &[PropDef] = &[PropDef {
    name: "Guild",
    kind: Kind::Object("Guild", GUILD),
    read: GM,
    write: GM,
    get: guild_object,
    set: None,
}];

pub fn configure(w: &mut World) {
    muo_content::props::add(w, "Server.Items.Guildstone", STONE);
    muo_content::props::add(w, "Server.Items.GuildstoneDeed", STONE);
}

#[allow(dead_code)]
const _: Numeric = Numeric::Int;
