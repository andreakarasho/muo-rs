//! Classes declared in scripts: `class("Server.Mobiles.Orc", #{ .. }, |m| { .. })`, evaluated once when the
//! templates load (startup, `[ReloadTemplates`, `[ReloadScripts`).
//!
//! The map holds what a JSON template held besides its statements: `parent` (a new class deriving from a
//! registered one), `inherits`, `ai`, `fightMode`, `rangePerception`, `rangeFight`, `script`, `props` — and may hold
//! statements too (`body: 17`), replayed before the closure. The closure is the constructor body: it runs on every
//! new entity of the class, after its parent's constructor, with the constructor's arguments after the entity.
//! Every template key is also a function of the same name for the closure: `str(m, [96, 120])`,
//! `resists(m, #{ Fire: [20, 30] })`, `creatureHue(m, ..)`; each runs the same setter, so a class written as
//! script draws the RNG exactly as its JSON form did.

use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;

use rhai::{Dynamic, Engine, FnPtr, Map};

use crate::templates::keys::Keys;
use crate::templates::{self, Kind, TemplateTable};
use muo_server::type_registry::EntityType;
use muo_server::world::World;

/// One `class(..)` call.
pub(crate) struct Decl {
    pub name: String,
    pub meta: Map,
    pub ctor: Option<FnPtr>,
    pub script: usize,
}

/// What the `class` function collects while the scripts' top levels run. Load-time scratch only, dropped once the
/// templates are built.
#[derive(Default)]
pub(crate) struct Collector {
    pub current: usize,
    pub decls: Vec<Decl>,
}

/// Registers `class(name, meta)`, `class(name, ctor)` and `class(name, meta, ctor)`.
pub(crate) fn register(e: &mut Engine, c: &Rc<RefCell<Collector>>) {
    let push = |c: &Rc<RefCell<Collector>>, name: &str, meta: Map, ctor: Option<FnPtr>| {
        let mut c = c.borrow_mut();
        let script = c.current;
        c.decls.push(Decl {
            name: name.to_owned(),
            meta,
            ctor,
            script,
        });
    };
    let c1 = c.clone();
    e.register_fn("class", move |name: &str, meta: Map, ctor: FnPtr| {
        push(&c1, name, meta, Some(ctor))
    });
    let c2 = c.clone();
    e.register_fn("class", move |name: &str, meta: Map| {
        push(&c2, name, meta, None)
    });
    let c3 = c.clone();
    e.register_fn("class", move |name: &str, ctor: FnPtr| {
        push(&c3, name, Map::new(), Some(ctor))
    });
}

/// The kind of class `name`: a registered class's, else an abstract Rust ancestor's (from its registered
/// descendants), else (a new class) its `parent`'s, following parents declared in scripts.
fn kind_of(w: &World, decls: &[Decl], name: &str, depth: u32) -> Option<Kind> {
    match w.types.find_type_by_full_name(name, false) {
        Some(EntityType::Mobile(_)) => return Some(Kind::Mobile),
        Some(EntityType::Item(_)) => return Some(Kind::Item),
        None => {}
    }
    // an abstract Rust class (never registered) whose props its subclasses inherit: the kind of a registered
    // class that has it as an ancestor
    let has_ancestor = |mut p: Option<&'static muo_server::item::ItemType>| {
        while let Some(t) = p {
            if t.full_name == name {
                return true;
            }
            p = t.parent;
        }
        false
    };
    if w.types.item_types().iter().any(|t| has_ancestor(t.parent)) {
        return Some(Kind::Item);
    }
    let has_mobile_ancestor = |mut p: Option<&'static muo_server::mobile::MobileType>| {
        while let Some(t) = p {
            if t.full_name == name {
                return true;
            }
            p = t.parent;
        }
        false
    };
    if w.types
        .mobile_types()
        .iter()
        .any(|t| has_mobile_ancestor(t.parent))
    {
        return Some(Kind::Mobile);
    }
    if depth > 32 {
        return None;
    }
    // a template only other templates `inherits` from: their kind
    let inheritor = decls.iter().find(|d| {
        d.meta
            .get("inherits")
            .and_then(|v| v.clone().into_string().ok())
            .is_some_and(|i| i == name)
    });
    if let Some(d) = inheritor
        && let Some(k) = kind_of(w, decls, &d.name, depth + 1)
    {
        return Some(k);
    }
    let d = decls.iter().find(|d| d.name == name)?;
    let parent = d.meta.get("parent")?.clone().into_string().ok()?;
    kind_of(w, decls, &parent, depth + 1)
}

/// Turns the collected declarations into templates of `t`; the constructors go to `ctors` (a template's `ctor` is
/// the index).
pub(crate) fn build(
    w: &World,
    decls: Vec<Decl>,
    script_names: &[String],
    lenient: bool,
    t: &mut TemplateTable,
    ctors: &mut Vec<(usize, FnPtr)>,
) -> Result<(), String> {
    let keys = if lenient {
        Keys::lenient(&w.types)
    } else {
        Keys::of(&w.types)
    };
    for d in &decls {
        let origin = format!("Data/Scripts/{}", script_names[d.script]);
        let err = |m: String| format!("{origin}: class {}: {m}", d.name);
        let kind = kind_of(w, &decls, &d.name, 0)
            .ok_or_else(|| err("not a registered class, and no \"parent\" names one".to_owned()))?;
        let serde_json::Value::Object(mut obj) =
            super::api::to_json(&Dynamic::from_map(d.meta.clone())).map_err(err)?
        else {
            return Err(err("the class map is not a map".to_owned()));
        };
        obj.insert("type".to_owned(), d.name.clone().into());
        let mut tmpl = templates::parse_template(Path::new(&origin), kind, &obj, &keys)?;
        if let Some(f) = &d.ctor {
            tmpl.ctor = Some(ctors.len() as u32);
            ctors.push((d.script, f.clone()));
        }
        templates::insert(t, tmpl)?;
    }
    Ok(())
}
