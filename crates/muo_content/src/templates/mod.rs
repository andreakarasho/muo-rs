//! JSON content templates: `Data/Templates/{Mobiles,Items}/**/*.json`.
//!
//! Each file is an array of objects. An object names a class (`"type"`) and lists what its constructor sets, in
//! file order (`serde_json` keeps key order), so a template replays the old constructor statement by statement and
//! the RNG is drawn in the same sequence. Keys:
//!
//! - meta: `type`, `inherits` (another template applied first), `parent` (makes a new JSON-only class deriving from
//!   that registered class), `ai`, `fightMode`, `rangePerception`, `rangeFight` (the `BaseCreature` constructor
//!   arguments), `props` (constant class overrides read through [`props`]), `script` (a `Data/Scripts` file).
//! - everything else: one constructor step ([`mobile_steps`], [`item_steps`]).
//!
//! The table lives in `w.resources` as `Rc<TemplateTable>`: clone the `Rc` before applying, so the world is free
//! to mutate. Edits apply to entities made after a (re)load; saved entities keep their saved values.

pub mod item_props;
pub mod item_steps;
pub mod mobile_steps;
pub mod props;
pub mod runtime_types;

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::rc::Rc;

use serde_json::{Map, Value};

use muo_server::ids::{ItemId, MobileId};
use muo_server::mobile::MobileType;
use muo_server::world::World;

use crate::creature::enums::{AIType, DEFAULT_RANGE_PERCEPTION, FightMode};

/// What a template is for.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Kind {
    Mobile,
    Item,
}

/// The `BaseCreature(ai, mode, perception, fight)` constructor arguments.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct CreatureArgs {
    pub ai: AIType,
    pub fight_mode: FightMode,
    pub range_perception: i32,
    pub range_fight: i32,
}

impl Default for CreatureArgs {
    fn default() -> Self {
        Self {
            ai: AIType::AiMelee,
            fight_mode: FightMode::Closest,
            range_perception: DEFAULT_RANGE_PERCEPTION,
            range_fight: 1,
        }
    }
}

/// One parsed template.
#[derive(Debug)]
pub struct Template {
    pub kind: Kind,
    pub type_name: String,
    pub file: PathBuf,
    pub inherits: Option<String>,
    pub parent: Option<String>,
    pub script: Option<String>,
    /// Only the keys present in the file; unset ones fall back to the inherited template, then the default.
    pub ai: Option<AIType>,
    pub fight_mode: Option<FightMode>,
    pub range_perception: Option<i32>,
    pub range_fight: Option<i32>,
    pub props: Map<String, Value>,
    pub mobile_steps: Vec<mobile_steps::Step>,
    pub item_steps: Vec<item_steps::Step>,
}

/// Every loaded template, by full type name.
#[derive(Debug, Default)]
pub struct TemplateTable {
    pub mobiles: HashMap<String, Template>,
    pub items: HashMap<String, Template>,
    /// Strings handed out as `&'static str` (corpse names, ...): leaked once per distinct text.
    interned: HashSet<&'static str>,
}

impl TemplateTable {
    pub fn get(&self, kind: Kind, name: &str) -> Option<&Template> {
        match kind {
            Kind::Mobile => self.mobiles.get(name),
            Kind::Item => self.items.get(name),
        }
    }

    /// The template and then each `inherits` ancestor (cycles are rejected at load).
    pub fn chain<'a>(&'a self, kind: Kind, name: &str) -> impl Iterator<Item = &'a Template> {
        let mut next = self.get(kind, name);
        std::iter::from_fn(move || {
            let t = next?;
            next = t.inherits.as_deref().and_then(|n| self.get(kind, n));
            Some(t)
        })
    }

    /// A `&'static` copy of `s`, leaked at most once per distinct text across reloads.
    pub fn intern(&mut self, s: &str) -> &'static str {
        if let Some(v) = self.interned.get(s) {
            return v;
        }
        let v: &'static str = Box::leak(s.to_owned().into_boxed_str());
        self.interned.insert(v);
        v
    }

    pub fn creature_args(&self, name: &str) -> CreatureArgs {
        let mut a = CreatureArgs::default();
        let chain: Vec<_> = self.chain(Kind::Mobile, name).collect();
        for t in chain.iter().rev() {
            a.ai = t.ai.unwrap_or(a.ai);
            a.fight_mode = t.fight_mode.unwrap_or(a.fight_mode);
            a.range_perception = t.range_perception.unwrap_or(a.range_perception);
            a.range_fight = t.range_fight.unwrap_or(a.range_fight);
        }
        a
    }
}

/// The name of an enum value as its `Debug` text, for values `0..256` that `from` accepts.
pub(crate) fn parse_enum<T: std::fmt::Debug>(
    s: &str,
    from: impl Fn(i32) -> Option<T>,
) -> Option<T> {
    (0..256).filter_map(&from).find(|v| format!("{v:?}") == s)
}

fn field_err(file: &Path, ty: &str, key: &str, msg: impl std::fmt::Display) -> String {
    format!("{}: {ty}: \"{key}\": {msg}", file.display())
}

fn parse_template(file: &Path, kind: Kind, obj: &Map<String, Value>) -> Result<Template, String> {
    let ty = obj
        .get("type")
        .and_then(Value::as_str)
        .ok_or_else(|| format!("{}: a template without a \"type\" string", file.display()))?
        .to_owned();
    let mut t = Template {
        kind,
        type_name: ty.clone(),
        file: file.to_owned(),
        inherits: None,
        parent: None,
        script: None,
        ai: None,
        fight_mode: None,
        range_perception: None,
        range_fight: None,
        props: Map::new(),
        mobile_steps: Vec::new(),
        item_steps: Vec::new(),
    };
    let s = |v: &Value| v.as_str().map(str::to_owned).ok_or("expected a string");
    let n = |v: &Value| {
        v.as_i64()
            .and_then(|v| i32::try_from(v).ok())
            .ok_or("expected an integer")
    };
    for (key, v) in obj {
        let e = |m: &str| field_err(file, &ty, key, m);
        match key.as_str() {
            "type" => {}
            "inherits" => t.inherits = Some(s(v).map_err(e)?),
            "parent" => t.parent = Some(s(v).map_err(e)?),
            "script" => t.script = Some(s(v).map_err(e)?),
            "props" => {
                t.props = v
                    .as_object()
                    .cloned()
                    .ok_or_else(|| e("expected an object"))?
            }
            "ai" if kind == Kind::Mobile => {
                let name = s(v).map_err(e)?;
                t.ai =
                    Some(parse_enum(&name, AIType::from_i32).ok_or_else(|| e("unknown AIType"))?);
            }
            "fightMode" if kind == Kind::Mobile => {
                let name = s(v).map_err(e)?;
                t.fight_mode = Some(
                    parse_enum(&name, FightMode::from_i32).ok_or_else(|| e("unknown FightMode"))?,
                );
            }
            "rangePerception" if kind == Kind::Mobile => {
                t.range_perception = Some(n(v).map_err(e)?)
            }
            "rangeFight" if kind == Kind::Mobile => t.range_fight = Some(n(v).map_err(e)?),
            _ => match kind {
                Kind::Mobile => t
                    .mobile_steps
                    .push(mobile_steps::parse(key, v).map_err(|m| e(&m))?),
                Kind::Item => t
                    .item_steps
                    .push(item_steps::parse(key, v).map_err(|m| e(&m))?),
            },
        }
    }
    Ok(t)
}

fn json_files(dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), String> {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return Ok(()); // an absent folder is an empty one
    };
    let mut entries: Vec<_> = rd.filter_map(Result::ok).map(|e| e.path()).collect();
    entries.sort(); // load order is stable across platforms
    for p in entries {
        if p.is_dir() {
            json_files(&p, out)?;
        } else if p
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("json"))
        {
            out.push(p);
        }
    }
    Ok(())
}

/// Reads `<root>/Mobiles/**/*.json` and `<root>/Items/**/*.json`. Any malformed file, duplicate type, unknown
/// `inherits` or `inherits` cycle is an error naming the file and type.
pub fn load_dir(root: &Path) -> Result<TemplateTable, String> {
    load_dir_keeping(root, HashSet::new())
}

/// [`load_dir`] reusing the interned strings of an earlier table, so a reload leaks only new texts.
fn load_dir_keeping(root: &Path, interned: HashSet<&'static str>) -> Result<TemplateTable, String> {
    let mut table = TemplateTable {
        interned,
        ..TemplateTable::default()
    };
    for (sub, kind) in [("Mobiles", Kind::Mobile), ("Items", Kind::Item)] {
        let mut files = Vec::new();
        json_files(&root.join(sub), &mut files)?;
        for file in files {
            let text =
                std::fs::read_to_string(&file).map_err(|e| format!("{}: {e}", file.display()))?;
            let v: Value =
                serde_json::from_str(&text).map_err(|e| format!("{}: {e}", file.display()))?;
            let arr = v
                .as_array()
                .ok_or_else(|| format!("{}: expected an array of templates", file.display()))?;
            for o in arr {
                let o = o
                    .as_object()
                    .ok_or_else(|| format!("{}: expected a template object", file.display()))?;
                let t = parse_template(&file, kind, o)?;
                let map = match kind {
                    Kind::Mobile => &mut table.mobiles,
                    Kind::Item => &mut table.items,
                };
                if let Some(old) = map.get(&t.type_name) {
                    return Err(format!(
                        "{}: {} is already defined in {}",
                        file.display(),
                        t.type_name,
                        old.file.display()
                    ));
                }
                map.insert(t.type_name.clone(), t);
            }
        }
    }
    for kind in [Kind::Mobile, Kind::Item] {
        let map = match kind {
            Kind::Mobile => &table.mobiles,
            Kind::Item => &table.items,
        };
        for t in map.values() {
            let mut seen = HashSet::new();
            let mut cur = t;
            while let Some(i) = &cur.inherits {
                if !seen.insert(cur.type_name.as_str()) {
                    return Err(format!(
                        "{}: {}: \"inherits\" cycle",
                        t.file.display(),
                        t.type_name
                    ));
                }
                cur = map.get(i).ok_or_else(|| {
                    format!(
                        "{}: {}: \"inherits\": unknown template {i}",
                        cur.file.display(),
                        cur.type_name
                    )
                })?;
            }
        }
    }
    props::intern_all(&mut table);
    Ok(table)
}

/// `Data/Templates` under the server folder.
pub fn templates_dir(w: &World) -> PathBuf {
    w.core.base_directory.join("Data").join("Templates")
}

/// Where a world that was never configured (a test world) finds the templates: `MUO_TEMPLATES`, else the first
/// `Data/Templates` in the current directory or one of its ancestors.
fn fallback_dir() -> Option<PathBuf> {
    if let Some(p) = std::env::var_os("MUO_TEMPLATES") {
        return Some(PathBuf::from(p));
    }
    let cwd = std::env::current_dir().ok()?;
    cwd.ancestors()
        .map(|a| a.join("Data").join("Templates"))
        .find(|p| p.is_dir())
}

/// The table, loading it on first use in a world whose Configure phase did not (test worlds).
pub fn table(w: &mut World) -> Rc<TemplateTable> {
    if let Some(t) = w.resources.get::<Rc<TemplateTable>>() {
        return t.clone();
    }
    let t = match fallback_dir().map(|d| load_dir(&d)) {
        Some(Ok(t)) => t,
        Some(Err(e)) => {
            tracing::error!("Templates: {e}");
            TemplateTable::default()
        }
        None => TemplateTable::default(),
    };
    let t = Rc::new(t);
    w.resources.insert(t.clone());
    t
}

/// The table if one is loaded (read-only callers cannot load it).
pub fn loaded(w: &World) -> Option<&TemplateTable> {
    w.resources.get::<Rc<TemplateTable>>().map(|t| &**t)
}

/// Loads `Data/Templates` and registers the JSON-only classes. Keeps the interned strings of an earlier table.
pub fn reload(w: &mut World) -> Result<(), String> {
    let old = w
        .resources
        .get::<Rc<TemplateTable>>()
        .map(|t| t.interned.clone())
        .unwrap_or_default();
    let mut t = load_dir_keeping(&templates_dir(w), old)?;
    runtime_types::register_all(w, &mut t)?;
    w.resources.insert(Rc::new(t));
    Ok(())
}

/// The Configure phase: a bad template file stops startup.
fn configure(w: &mut World) {
    if let Err(e) = reload(w) {
        w.core.startup_error = Some(format!("Templates: {e}"));
    }
}

/// Registers the Configure phase, early so classes made by later phases see the table.
pub fn register(types: &mut muo_server::type_registry::TypeRegistry) {
    types.register_phase(
        muo_server::type_registry::Phase::Configure,
        "muo_content.Templates.Configure",
        0,
        configure,
    );
}

/// Replays the template of the mobile's class (its `inherits` chain first) on `id`.
pub fn apply_mobile(w: &mut World, id: MobileId) {
    let Some(name) = w.mobile(id).map(|m| m.ty.full_name) else {
        return;
    };
    apply_mobile_as(w, id, name);
}

/// [`apply_mobile`] with the template of `name` (a JSON-only class replays only its own chain).
pub fn apply_mobile_as(w: &mut World, id: MobileId, name: &str) {
    let t = table(w);
    let chain: Vec<_> = t.chain(Kind::Mobile, name).collect();
    if chain.is_empty() {
        tracing::warn!("Templates: no mobile template for {name}");
    }
    for tmpl in chain.into_iter().rev() {
        for s in &tmpl.mobile_steps {
            mobile_steps::apply(w, id, s);
        }
    }
}

/// `new T()` of a creature class whose constructor is its template: the `BaseCreature` arguments and every
/// statement come from the template of `ty.full_name`.
pub fn new_creature(w: &mut World, ty: &'static MobileType) -> MobileId {
    let args = table(w).creature_args(ty.full_name);
    let id = crate::creature::setters::new_creature(
        w,
        ty,
        args.ai,
        args.fight_mode,
        args.range_perception,
        args.range_fight,
    );
    apply_mobile(w, id);
    id
}

/// Replays the template of the item's class on `id`.
pub fn apply_item(w: &mut World, id: ItemId) {
    let Some(name) = w.item(id).map(|i| i.ty.full_name) else {
        return;
    };
    apply_item_as(w, id, name);
}

pub fn apply_item_as(w: &mut World, id: ItemId, name: &str) {
    let t = table(w);
    let chain: Vec<_> = t.chain(Kind::Item, name).collect();
    if chain.is_empty() {
        tracing::warn!("Templates: no item template for {name}");
    }
    for tmpl in chain.into_iter().rev() {
        for s in &tmpl.item_steps {
            item_steps::apply(w, id, s);
        }
    }
}

#[cfg(test)]
mod tests;
