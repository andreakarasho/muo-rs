//! `SpawnerEntry.Properties` and `.Parameters`: the part of `BaseSpawner.Spawn(entry, out flags)` that picks a
//! constructor from the parameter text and sets `[CommandProperty]` members from `name value` pairs, over the
//! tables of `muo_content::props` instead of reflection.

use muo_content::props::{self, PropDef, Value};
use muo_primitives::dotnet::IgnoreCase;
use muo_server::commands::split;
use muo_server::ids::EntityId;
use muo_server::mobile::AccessLevel;
use muo_server::type_registry::EntityType;
use muo_server::world::World;

/// The class names of `ty` and its ancestors, root first.
fn chain_root_first(ty: EntityType) -> Vec<&'static str> {
    let mut chain = Vec::new();
    match ty {
        EntityType::Item(t) => {
            let mut c = Some(t);
            while let Some(x) = c {
                chain.push(x.full_name);
                c = x.parent;
            }
        }
        EntityType::Mobile(t) => {
            let mut c = Some(t);
            while let Some(x) = c {
                chain.push(x.full_name);
                c = x.parent;
            }
        }
    }
    chain.reverse();
    chain
}

/// A resolved `name value` pair.
pub struct Assignment {
    def: &'static PropDef,
    text: String,
}

/// `FormatProperties` + `GetTypeProperties`: the pairs of `properties` (an odd last word is dropped) with their
/// property, or `None` when one names no property of the type, or one that is not writable by a Developer.
pub fn resolve_props(
    w: &World,
    ty: EntityType,
    properties: Option<&str>,
) -> Option<Vec<Assignment>> {
    let text = properties.map(str::trim).unwrap_or("");
    let args = if text.is_empty() {
        Vec::new()
    } else {
        split(text)
    };
    if args.len() < 2 {
        return Some(Vec::new());
    }
    let all = props::props_of_class(w, &chain_root_first(ty));
    let mut out = Vec::with_capacity(args.len() / 2);
    for pair in args.as_chunks::<2>().0 {
        let def = all.iter().find(|d| d.name.eq_ic(&pair[0]))?;
        if def.write > AccessLevel::Developer || def.set.is_none() {
            return None;
        }
        out.push(Assignment {
            def,
            text: pair[1].clone(),
        });
    }
    Some(out)
}

/// The constructor choice: no parameters is the parameterless constructor, otherwise the first `[Constructible]`
/// overload (reachable at Developer) with exactly that many parameters whose values parse.
pub fn construct(w: &mut World, ty: EntityType, parameters: Option<&str>) -> Option<EntityId> {
    let text = parameters.map(str::trim).unwrap_or("");
    // `string.IsNullOrEmpty(entry.Parameters) ? [] : entry.Parameters.Trim().Split(' ')`
    let args: Vec<String> = if parameters.is_none_or(str::is_empty) {
        Vec::new()
    } else {
        text.split(' ').map(str::to_owned).collect()
    };
    let table = props::ctors_of(w, ty.full_name());
    if args.is_empty() {
        return match ty {
            EntityType::Item(t) => t.construct.map(|c| EntityId::Item(c(w))),
            EntityType::Mobile(t) => t.construct.map(|c| EntityId::Mobile(c(w))),
        };
    }
    for c in table? {
        // `Constructible` without a level is the plain attribute
        if !c.access.is_none_or(|r| AccessLevel::Developer >= r) || c.params.len() != args.len() {
            continue;
        }
        if let Some(values) = props::parse_ctor_args(w, c.params, &args) {
            return Some((c.make)(w, &values));
        }
    }
    None
}

/// Sets the properties; false when a value does not parse (or its setter refuses).
pub fn apply(w: &mut World, entity: EntityId, assignments: &[Assignment]) -> bool {
    for a in assignments {
        let Ok(value): Result<Value, _> = props::parse_for(w, a.def, &a.text) else {
            return false;
        };
        let Some(set) = a.def.set else { return false };
        // `SetValue` throws on a bad type, which `Spawn` does not catch around the loop: it ends in the outer catch
        if set(w, entity, value).is_err() {
            return false;
        }
    }
    true
}
