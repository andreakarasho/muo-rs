//! `[props`, and the get / set / increase helpers `[get`, `[set`,
//! `[increase` and the props gump share. Reflection is the property table of `muo_content::props`.

use std::rc::Rc;

use muo_content::props::{self, Access, Kind, PropDef, Value};
use muo_content::utilities::types::{Numeric, PropType};
use muo_primitives::Serial;
use muo_server::commands::{CommandEventArgs, CommandSystem};
use muo_server::ids::{EntityId, MobileId};
use muo_server::item::deps::send_localized_message;
use muo_server::mobile::AccessLevel;
use muo_server::mobile::part2::messages::send_message;
use muo_server::target::{TargetFlags, Targeted, begin_target};
use muo_server::world::World;

use crate::generic::is_accessible;
use crate::hooks::CommandHooks;
use crate::logging;

pub fn configure(w: &mut World) {
    CommandSystem::register(
        w,
        "Props",
        AccessLevel::Counselor,
        &[],
        Rc::new(props_on_command),
    );
}

fn level_of(w: &World, m: MobileId) -> AccessLevel {
    w.mobile(m).map_or(AccessLevel::Player, |x| x.access_level)
}

/// `new PropertiesGump(from, o)`; the gump is the `muo_gumps` crate's (`CommandHooks::properties_gump`).
pub(crate) fn open_properties_gump(w: &mut World, from: MobileId, o: EntityId) {
    // muo_gumps sets the hook (`m_z::props::configure`)
    if let Some(open) = w.resources.get_mut::<CommandHooks>().properties_gump {
        open(w, from, o);
    }
}

/// `[Usage("Props [serial]")]`
fn props_on_command(w: &mut World, e: &mut CommandEventArgs) {
    let from = e.mobile;
    if e.length() == 1 {
        match w.find_entity(Serial::new(e.get_uint32(0))) {
            None => send_message(w, from, "No object with that serial was found."),
            Some(ent) if !is_accessible(w, from, Some(Targeted::Entity(ent))) => {
                send_localized_message(w, from, 500447)
            }
            Some(ent) => open_properties_gump(w, from, ent),
        }
    } else {
        begin_target(w, from, -1, true, TargetFlags::NONE, |w, from, o| {
            if !is_accessible(w, from, Some(o)) {
                send_localized_message(w, from, 500447); // That is not accessible.
            } else if let Targeted::Entity(ent) = o {
                open_properties_gump(w, from, ent);
            } else if let Some(open) = w.resources.get_mut::<CommandHooks>().properties_gump_target
            {
                open(w, from, o); // a LandTarget / StaticTarget has [CommandProperty]s too
            }
        });
    }
}

/// `Properties.GetValue`: `Name = value`, or the reason it cannot be read.
pub fn get_value(w: &mut World, from: MobileId, e: EntityId, name: &str) -> String {
    match props::resolve_chain_named(w, level_of(w, from), e, name, Access::READ) {
        Err(reason) => reason,
        Ok((owner, def, names)) => internal_get_value(w, owner, def, &names),
    }
}

/// `InternalGetValue(o, p, chain)`: the chain is shown with the links' declared names.
fn internal_get_value(w: &mut World, owner: EntityId, def: &PropDef, chain: &[&str]) -> String {
    let v = (def.get)(w, owner);
    let text = props::describe(w, def, &v);
    format!("{} = {text}", chain.join("."))
}

/// `Properties.SetValue`.
pub fn set_value(w: &mut World, from: MobileId, e: EntityId, name: &str, value: &str) -> String {
    let (owner, def) = match props::resolve_chain(w, level_of(w, from), e, name, Access::WRITE) {
        Err(reason) => return reason,
        Ok(r) => r,
    };
    internal_set_value(w, from, e, owner, def, name, value, true)
}

#[allow(clippy::too_many_arguments)]
pub fn internal_set_value(
    w: &mut World,
    from: MobileId,
    log_object: EntityId,
    owner: EntityId,
    def: &'static PropDef,
    given_name: &str,
    value: &str,
    should_log: bool,
) -> String {
    match props::parse_for(w, def, value) {
        Err(m) => m,
        Ok(v) => set_direct(w, from, log_object, owner, def, given_name, v, should_log),
    }
}

/// `Properties.SetDirect`.
#[allow(clippy::too_many_arguments)]
pub fn set_direct(
    w: &mut World,
    from: MobileId,
    log_object: EntityId,
    owner: EntityId,
    def: &'static PropDef,
    given_name: &str,
    to_set: Value,
    should_log: bool,
) -> String {
    let from_level = level_of(w, from);
    // Don't allow staff to be modified if they are under StaffAccess command.
    if let EntityId::Mobile(m) = owner
        && let Some(mob) = w.mobile(m)
    {
        let acct_level = mob
            .account
            .and_then(|a| muo_content::accounting::account(w, a))
            .map(|a| a.access_level);
        if mob.access_level > from_level || acct_level.is_some_and(|l| l > from_level) {
            return "You cannot modify a higher privileged mobile.".into();
        }
    }
    if def.name == "AccessLevel"
        && let Value::Enum(n) = &to_set
    {
        let new_level = AccessLevel::from_i32(*n as i32).unwrap_or(AccessLevel::Player);
        let req = match new_level {
            AccessLevel::Administrator => AccessLevel::Developer,
            l if l >= AccessLevel::Developer => AccessLevel::Owner,
            _ => AccessLevel::Administrator,
        };
        if from_level < req {
            return "You do not have access to that level.".into();
        }
    }
    if should_log {
        let text = match &to_set {
            Value::Null | Value::Entity(None) | Value::Poison(None) | Value::Map(None) => {
                "(-null-)".to_string()
            }
            v => props::display(w, &def.kind, v),
        };
        logging::log_change_property(w, from, log_object, given_name, &text);
    }
    let Some(set) = def.set else {
        return "An exception was caught, the property may not be set.".into();
    };
    match set(w, owner, to_set) {
        Ok(()) => "Property has been set.".into(),
        Err(_) => "An exception was caught, the property may not be set.".into(),
    }
}

fn is_numeric(k: &Kind) -> bool {
    matches!(
        k,
        Kind::Plain(PropType::Numeric(_) | PropType::Float | PropType::Double | PropType::Decimal)
    )
}

/// `Properties.IncreaseValue`: `args` are `{name offset}` pairs.
pub fn increase_value(w: &mut World, from: MobileId, o: EntityId, args: &[String]) -> String {
    let n = args.len() / 2;
    let mut real: Vec<(EntityId, &'static PropDef)> = Vec::new();
    let mut values: Vec<i64> = Vec::new();
    let (mut positive, mut negative) = (false, false);
    for i in 0..n {
        let name = &args[i * 2];
        let vs = &args[1 + i * 2];
        let parsed = match vs.strip_prefix("0x") {
            // `Convert.ToInt32(s, 16)` reads eight digits as two's complement
            Some(hex) => u32::from_str_radix(hex, 16).ok().map(|v| v as i32),
            None => vs.parse::<i32>().ok(),
        };
        let Some(v) = parsed else {
            return "Offset value could not be parsed.".into();
        };
        match v.cmp(&0) {
            std::cmp::Ordering::Greater => positive = true,
            std::cmp::Ordering::Less => negative = true,
            std::cmp::Ordering::Equal => return "Zero is not a valid value to offset.".into(),
        }
        values.push(i64::from(v));
        match props::resolve_chain(w, level_of(w, from), o, name, Access::READ_WRITE) {
            Ok(r) => real.push(r),
            Err(reason) => return reason,
        }
    }
    for (i, (owner, def)) in real.iter().enumerate() {
        let cur = (def.get)(w, *owner);
        // a string is `IConvertible` but only converts when it holds a number
        if let Value::Str(s) = &cur {
            let Ok(n) = s.trim().parse::<i64>() else {
                return "Value could not be converted".into();
            };
            let Some(set) = def.set else {
                return "Value could not be converted".into();
            };
            if set(w, *owner, Value::Str((n + values[i]).to_string())).is_err() {
                return "Value could not be converted".into();
            }
            continue;
        }
        if !is_numeric(&def.kind) && !matches!(cur, Value::Enum(_) | Value::Bool(_)) {
            return "Property is not IConvertable.".into();
        }
        let v = cur.i64() + values[i];
        let new = match &def.kind {
            Kind::Plain(PropType::Float | PropType::Double | PropType::Decimal) => {
                Value::Float(v as f64)
            }
            Kind::Plain(PropType::Enum(_)) => Value::Enum(v),
            Kind::Plain(PropType::Numeric(Numeric::UInt | Numeric::ULong)) => Value::UInt(v as u64),
            _ => Value::Int(v),
        };
        let Some(set) = def.set else {
            return "Value could not be converted".into();
        };
        if set(w, *owner, new).is_err() {
            return "Value could not be converted".into();
        }
    }
    if real.len() == 1 {
        return if positive {
            "The property has been increased."
        } else {
            "The property has been decreased."
        }
        .into();
    }
    if positive && negative {
        return "The properties have been changed.".into();
    }
    if positive {
        "The properties have been increased."
    } else {
        "The properties have been decreased."
    }
    .into()
}
