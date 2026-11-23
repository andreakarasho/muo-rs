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
    /// `item_class` / `mobile_class` say it; `class` infers it.
    pub kind: Option<Kind>,
    /// `c.on_use(..)`, `c.on_death(..)`, `c.on_outfit(..)`, ..: (hook name, closure).
    pub hooks: Vec<(String, FnPtr)>,
}

/// What the `class` function collects while the scripts' top levels run. Load-time scratch only, dropped once the
/// templates are built.
#[derive(Default)]
pub(crate) struct Collector {
    pub current: usize,
    pub decls: Vec<Decl>,
    /// `shop(..)` and other content: (kind, name, value).
    pub data: Vec<(String, String, rhai::Dynamic)>,
}

/// Registers `class`, `item_class` and `mobile_class`, each as `(name, meta)`, `(name, ctor)` and
/// `(name, meta, ctor)`. `class` infers the kind from the registered classes; the other two state it, so a class
/// loads even in a world that registered none (a test world).
pub(crate) fn register(e: &mut Engine, c: &Rc<RefCell<Collector>>) {
    fn push(
        c: &Rc<RefCell<Collector>>,
        kind: Option<Kind>,
        name: &str,
        meta: Map,
        ctor: Option<FnPtr>,
    ) {
        let mut c = c.borrow_mut();
        let script = c.current;
        c.decls.push(Decl {
            name: name.to_owned(),
            meta,
            ctor,
            script,
            kind,
            hooks: Vec::new(),
        });
    }
    for (fname, kind) in [
        ("class", None),
        ("item_class", Some(Kind::Item)),
        ("mobile_class", Some(Kind::Mobile)),
    ] {
        let c1 = c.clone();
        e.register_fn(fname, move |name: &str, meta: Map, ctor: FnPtr| {
            push(&c1, kind, name, meta, Some(ctor))
        });
        let c2 = c.clone();
        e.register_fn(fname, move |name: &str, meta: Map| {
            push(&c2, kind, name, meta, None)
        });
        let c3 = c.clone();
        e.register_fn(fname, move |name: &str, ctor: FnPtr| {
            push(&c3, kind, name, Map::new(), Some(ctor))
        });
    }
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
/// The full name of class `name`: itself if registered or not a short name of one, else the unique registered
/// class of that short name (`"Orc"` -> `"Server.Mobiles.Orc"`).
pub(crate) fn full_name(w: &World, name: &str) -> Result<String, String> {
    if name.contains('.') || w.types.find_type_by_full_name(name, false).is_some() {
        return Ok(name.to_owned());
    }
    match w.types.types_named(name) {
        [] => Ok(name.to_owned()),
        [one] => Ok(one.full_name().to_owned()),
        many => Err(format!(
            "{name} names {} classes ({}); write the full name",
            many.len(),
            many.iter()
                .map(|t| t.full_name())
                .collect::<Vec<_>>()
                .join(", ")
        )),
    }
}

pub(crate) fn build(
    w: &World,
    mut decls: Vec<Decl>,
    script_names: &[String],
    lenient: bool,
    t: &mut TemplateTable,
    ctors: &mut Vec<(usize, FnPtr)>,
    hooks: &mut Vec<(String, String, usize, FnPtr)>,
) -> Result<(), String> {
    for d in &mut decls {
        let origin = format!("Data/Scripts/{}", script_names[d.script]);
        d.name = full_name(w, &d.name).map_err(|e| format!("{origin}: {e}"))?;
        for k in ["parent", "inherits"] {
            if let Some(v) = d.meta.get(k).and_then(|v| v.clone().into_string().ok()) {
                let full =
                    full_name(w, &v).map_err(|e| format!("{origin}: class {}: {e}", d.name))?;
                d.meta.insert(k.into(), full.into());
            }
        }
    }
    let keys = if lenient {
        Keys::lenient(&w.types)
    } else {
        Keys::of(&w.types)
    };
    for d in &decls {
        let origin = format!("Data/Scripts/{}", script_names[d.script]);
        let err = |m: String| format!("{origin}: class {}: {m}", d.name);
        let kind = d
            .kind
            .or_else(|| kind_of(w, &decls, &d.name, 0))
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
        for (hook, f) in &d.hooks {
            if hook == "on_outfit" {
                // run where the NPC outfit runs: a prop naming the closure
                tmpl.props.insert(
                    "npcOutfit".into(),
                    serde_json::json!({ "$script": ctors.len() }),
                );
                ctors.push((d.script, f.clone()));
            } else {
                hooks.push((d.name.clone(), hook.clone(), d.script, f.clone()));
            }
        }
        templates::insert(t, tmpl)?;
    }
    Ok(())
}
