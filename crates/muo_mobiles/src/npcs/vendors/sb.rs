//! The `SBInfo` tables, what each vendor buys and sells: `Data/Templates/Vendors/*.json`.
//!
//! Each file is an array of `{"name": "SBAlchemist", "build": [statement, ..]}`; the statements run in order (the
//! RNG is drawn in that order):
//! - `{"sell": "<class>", "price": n}`: `GenericSellInfo.Add(type, price)`.
//! - `{"buy": "<class>", "name": "<cliloc or text>", "price": n, "amount": n | {"from": a, "count": c},
//!   "itemId": n, "hue": n | "randomDyed" | .., "arg": n}`: a `GenericBuyInfo`; with `"controlSlots": n` an
//!   `AnimalBuyInfo`, with `"beverage": <BeverageType>` a `BeverageBuyInfo`. `name` and `arg` are optional.
//! - `{"era": "AOS", "then": [..], "else": [..]}`: `if (Core.Expansion >= era)`.
//! - `{"if": "newVendorSystem" | "newGuildSystem", "then": [..], "else": [..]}`.
//! - `{"oneIn": n, "then": [..], "else": [..]}`: `if (Utility.Random(n) == 0)`.
//! - `{"scrollBuy": limit}`, `{"scrollSell": mult}`, `{"presetMaps": true}`: the spell scroll and preset map loops.
//!
//! The tables reload with the templates (`[ReloadTemplates`).

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::rc::Rc;

use serde_json::Value;

use super::sb_helpers::{preset_maps, scroll_buy, scroll_sell};
use crate::template_keys_npcs::Hue;
use muo_content::hooks::rnd;
use muo_content::templates::{self, TemplateTable};
use muo_content::vendor::{BuyInfo, SbInfo, SbInstance};
use muo_data::expansion::Expansion;
use muo_server::ids::MobileId;
use muo_server::world::World;

/// The `SBInfo` classes other crates name.
pub static SBMAGE: SbInfo = SbInfo {
    name: "SBMage",
    build: |w, v, sb| build(w, v, "SBMage", sb),
};
pub static SBNINJA: SbInfo = SbInfo {
    name: "SBNinja",
    build: |w, v, sb| build(w, v, "SBNinja", sb),
};
pub static SBSAMURAI: SbInfo = SbInfo {
    name: "SBSamurai",
    build: |w, v, sb| build(w, v, "SBSamurai", sb),
};

#[derive(Debug)]
enum Amount {
    Fixed(i32),
    /// `Utility.Random(from, count)`.
    From(i32, i32),
}

#[derive(Debug)]
enum BuyKind {
    Generic,
    Animal(i32),
    Beverage(i32),
}

#[derive(Debug)]
struct Buy {
    kind: BuyKind,
    type_name: &'static str,
    name: Option<String>,
    price: i32,
    amount: Amount,
    item_id: i32,
    hue: Hue,
    arg: Option<i32>,
}

#[derive(Debug)]
enum Cond {
    Era(Expansion),
    NewVendorSystem,
    NewGuildSystem,
    OneIn(i32),
}

#[derive(Debug)]
enum Stmt {
    Sell(&'static str, i32),
    Buy(Buy),
    If(Cond, Vec<Stmt>, Vec<Stmt>),
    ScrollBuy(usize),
    ScrollSell(i32),
    PresetMaps,
}

/// Every loaded `SBInfo`, and the template table it was loaded with (a template reload reloads it).
#[derive(Default)]
pub struct SbTable {
    sbs: HashMap<String, Vec<Stmt>>,
    with: Option<Rc<TemplateTable>>,
}

impl SbTable {
    pub fn contains(&self, name: &str) -> bool {
        self.sbs.contains_key(name)
    }
}

/// Class names handed out as `&'static str`, leaked once per distinct text across reloads.
#[derive(Default)]
struct Interned(HashSet<&'static str>);

impl Interned {
    fn get(&mut self, s: &str) -> &'static str {
        if let Some(v) = self.0.get(s) {
            return v;
        }
        let v: &'static str = Box::leak(s.to_owned().into_boxed_str());
        self.0.insert(v);
        v
    }
}

fn int(v: Option<&Value>, what: &str) -> Result<i32, String> {
    v.and_then(Value::as_i64)
        .and_then(|n| i32::try_from(n).ok())
        .ok_or_else(|| format!("\"{what}\": expected an integer"))
}

fn block(v: Option<&Value>, names: &mut Interned) -> Result<Vec<Stmt>, String> {
    match v {
        None => Ok(Vec::new()),
        Some(Value::Array(a)) => a.iter().map(|s| stmt(s, names)).collect(),
        Some(_) => Err("expected an array of statements".into()),
    }
}

fn stmt(v: &Value, names: &mut Interned) -> Result<Stmt, String> {
    let o = v
        .as_object()
        .ok_or_else(|| format!("expected a statement object, got {v}"))?;
    let branches = |names: &mut Interned| -> Result<(Vec<Stmt>, Vec<Stmt>), String> {
        Ok((block(o.get("then"), names)?, block(o.get("else"), names)?))
    };
    if let Some(t) = o.get("sell") {
        let t = t.as_str().ok_or("\"sell\": expected a class name")?;
        return Ok(Stmt::Sell(names.get(t), int(o.get("price"), "price")?));
    }
    if let Some(t) = o.get("buy") {
        let t = t.as_str().ok_or("\"buy\": expected a class name")?;
        let kind = match (o.get("controlSlots"), o.get("beverage")) {
            (Some(c), None) => BuyKind::Animal(int(Some(c), "controlSlots")?),
            (None, Some(b)) => BuyKind::Beverage(int(Some(b), "beverage")?),
            (None, None) => BuyKind::Generic,
            _ => return Err("controlSlots and beverage together".into()),
        };
        let amount = match o.get("amount") {
            Some(Value::Object(a)) => Amount::From(
                int(a.get("from"), "amount.from")?,
                int(a.get("count"), "amount.count")?,
            ),
            a => Amount::Fixed(int(a, "amount")?),
        };
        let name = match o.get("name") {
            None => None,
            Some(n) => Some(n.as_str().ok_or("\"name\": expected a string")?.to_owned()),
        };
        return Ok(Stmt::Buy(Buy {
            kind,
            type_name: names.get(t),
            name,
            price: int(o.get("price"), "price")?,
            amount,
            item_id: int(o.get("itemId"), "itemId")?,
            hue: Hue::parse(o.get("hue").unwrap_or(&Value::from(0)))?,
            arg: o.get("arg").map(|a| int(Some(a), "arg")).transpose()?,
        }));
    }
    if let Some(e) = o.get("era") {
        let name = e.as_str().ok_or("\"era\": expected an era name")?;
        let era = (0..64)
            .filter_map(Expansion::from_index)
            .find(|x| format!("{x:?}") == name)
            .ok_or_else(|| format!("unknown era {name}"))?;
        let (t, f) = branches(names)?;
        return Ok(Stmt::If(Cond::Era(era), t, f));
    }
    if let Some(c) = o.get("if") {
        let c = match c.as_str() {
            Some("newVendorSystem") => Cond::NewVendorSystem,
            Some("newGuildSystem") => Cond::NewGuildSystem,
            _ => return Err(format!("\"if\": unknown condition {c}")),
        };
        let (t, f) = branches(names)?;
        return Ok(Stmt::If(c, t, f));
    }
    if let Some(n) = o.get("oneIn") {
        let n = int(Some(n), "oneIn")?;
        let (t, f) = branches(names)?;
        return Ok(Stmt::If(Cond::OneIn(n), t, f));
    }
    if let Some(n) = o.get("scrollBuy") {
        let n = n.as_u64().ok_or("\"scrollBuy\": expected a count")?;
        return Ok(Stmt::ScrollBuy(n as usize));
    }
    if let Some(n) = o.get("scrollSell") {
        return Ok(Stmt::ScrollSell(int(Some(n), "scrollSell")?));
    }
    if o.get("presetMaps").is_some() {
        return Ok(Stmt::PresetMaps);
    }
    Err(format!("unknown statement {v}"))
}

fn load_dir(dir: &Path, names: &mut Interned) -> Result<HashMap<String, Vec<Stmt>>, String> {
    let rd = std::fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let mut files: Vec<PathBuf> = rd
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e.eq_ignore_ascii_case("json")))
        .collect();
    files.sort();
    let mut sbs = HashMap::new();
    for f in files {
        let err = |m: String| format!("{}: {m}", f.display());
        let text = std::fs::read_to_string(&f).map_err(|e| err(e.to_string()))?;
        let v: Value = serde_json::from_str(&text).map_err(|e| err(e.to_string()))?;
        let arr = v
            .as_array()
            .ok_or_else(|| err("expected an array".into()))?;
        for o in arr {
            let name = o
                .get("name")
                .and_then(Value::as_str)
                .ok_or_else(|| err("an SBInfo without a \"name\" string".into()))?;
            let build = block(o.get("build"), names).map_err(|m| err(format!("{name}: {m}")))?;
            if sbs.insert(name.to_owned(), build).is_some() {
                return Err(err(format!("{name} is defined twice")));
            }
        }
    }
    Ok(sbs)
}

/// `Data/Templates/Vendors`: next to the loaded templates, else where a test world finds them.
fn vendors_dir(w: &World) -> Option<PathBuf> {
    let d = templates::templates_dir(w).join("Vendors");
    if d.is_dir() {
        return Some(d);
    }
    if let Some(p) = std::env::var_os("MUO_TEMPLATES") {
        return Some(PathBuf::from(p).join("Vendors"));
    }
    let cwd = std::env::current_dir().ok()?;
    cwd.ancestors()
        .map(|a| a.join("Data").join("Templates").join("Vendors"))
        .find(|p| p.is_dir())
}

/// Reads `Data/Templates/Vendors` into the world.
pub fn reload(w: &mut World) -> Result<(), String> {
    let with = templates::table(w);
    let mut names = std::mem::take(w.resources.get_mut::<Interned>());
    let r = match vendors_dir(w) {
        Some(d) => load_dir(&d, &mut names),
        None => Err("Data/Templates/Vendors is missing".to_owned()),
    };
    *w.resources.get_mut::<Interned>() = names;
    let (sbs, res) = match r {
        Ok(s) => (s, Ok(())),
        Err(e) => (HashMap::new(), Err(e)),
    };
    w.resources.insert(Rc::new(SbTable {
        sbs,
        with: Some(with),
    }));
    res
}

/// The tables, (re)loaded when the templates were (re)loaded since.
pub fn table(w: &mut World) -> Rc<SbTable> {
    let t = templates::table(w);
    if let Some(s) = w.resources.get::<Rc<SbTable>>()
        && s.with.as_ref().is_some_and(|x| Rc::ptr_eq(x, &t))
    {
        return s.clone();
    }
    if let Err(e) = reload(w) {
        tracing::error!("Vendors: {e}");
    }
    w.resources.get::<Rc<SbTable>>().cloned().unwrap_or_default()
}

/// The Configure phase (after the templates): a bad file stops startup.
pub(crate) fn configure(w: &mut World) {
    if let Err(e) = reload(w) {
        w.core.startup_error = Some(format!("Vendors: {e}"));
    }
}

fn run(w: &mut World, vendor: MobileId, list: &[Stmt], sb: &mut SbInstance) {
    for s in list {
        match s {
            Stmt::Sell(t, p) => sb.sell.add(t, *p),
            Stmt::Buy(b) => {
                let amount = match b.amount {
                    Amount::Fixed(n) => n,
                    Amount::From(a, c) => rnd::from(w, a, c),
                };
                let hue = b.hue.roll(w);
                let name = b.name.as_deref();
                let mut info = match b.kind {
                    BuyKind::Generic => {
                        BuyInfo::new(name, b.type_name, b.price, amount, b.item_id, hue)
                    }
                    BuyKind::Animal(cs) => {
                        BuyInfo::animal(cs, name, b.type_name, b.price, amount, b.item_id, hue)
                    }
                    BuyKind::Beverage(c) => {
                        BuyInfo::beverage(name, b.type_name, c, b.price, amount, b.item_id, hue)
                    }
                };
                if let Some(a) = b.arg {
                    info = info.with_arg(a);
                }
                sb.buy.push(info);
            }
            Stmt::If(c, then, otherwise) => {
                let yes = match c {
                    Cond::Era(e) => w.expansion() >= *e,
                    Cond::NewVendorSystem => muo_content::vendor::new_vendor_system(w),
                    Cond::NewGuildSystem => muo_content::misc::guild::new_guild_system(w),
                    Cond::OneIn(n) => rnd::below(w, *n) == 0,
                };
                run(w, vendor, if yes { then } else { otherwise }, sb);
            }
            Stmt::ScrollBuy(n) => scroll_buy(w, sb, *n),
            Stmt::ScrollSell(m) => scroll_sell(sb, *m),
            Stmt::PresetMaps => preset_maps(w, sb),
        }
    }
}

/// Runs the `SBInfo` named `name` into `sb`; an unknown name builds nothing (logged).
pub fn build(w: &mut World, vendor: MobileId, name: &str, sb: &mut SbInstance) {
    let t = table(w);
    match t.sbs.get(name) {
        Some(list) => run(w, vendor, list, sb),
        None => tracing::warn!("Vendors: no SBInfo {name}"),
    }
}

/// `new SBAlchemist()` for vendor `vendor`.
pub fn named(w: &mut World, vendor: MobileId, name: &str) -> SbInstance {
    let mut sb = SbInstance::default();
    build(w, vendor, name, &mut sb);
    sb
}

/// `InitSBInfo()` of a vendor class whose list is its `"npcSbInfos"` prop: SBInfo names, and
/// `{"oneOf": [[..], ..]}` (`switch (Utility.Random(n))`), `{"if": "tokunoVendor", "then": [..], "else": [..]}`,
/// `{"era": "AOS", "then": [..], "else": [..]}`.
pub fn from_props(w: &mut World, vendor: MobileId) -> Vec<SbInstance> {
    let list = templates::props::mobile(w, vendor, "npcSbInfos").cloned();
    let mut out = Vec::new();
    if let Some(Value::Array(a)) = list {
        sb_list(w, vendor, &a, &mut out);
    }
    out
}

fn sb_list(w: &mut World, vendor: MobileId, list: &[Value], out: &mut Vec<SbInstance>) {
    let arr = |v: Option<&Value>| v.and_then(Value::as_array).cloned().unwrap_or_default();
    for s in list {
        match s {
            Value::String(n) => out.push(named(w, vendor, n)),
            Value::Object(o) if o.contains_key("oneOf") => {
                let blocks = arr(o.get("oneOf"));
                if !blocks.is_empty() {
                    let i = rnd::below(w, blocks.len() as i32) as usize;
                    sb_list(w, vendor, &arr(blocks.get(i)), out);
                }
            }
            Value::Object(o) => {
                let yes = match (o.get("if").and_then(Value::as_str), o.get("era")) {
                    (Some("tokunoVendor"), _) => muo_content::vendor::is_tokuno_vendor(w, vendor),
                    (_, Some(Value::String(e))) => (0..64)
                        .filter_map(Expansion::from_index)
                        .find(|x| format!("{x:?}") == *e)
                        .is_some_and(|e| w.expansion() >= e),
                    _ => {
                        tracing::warn!("Vendors: a bad npcSbInfos entry {s}");
                        continue;
                    }
                };
                let b = arr(o.get(if yes { "then" } else { "else" }));
                sb_list(w, vendor, &b, out);
            }
            _ => tracing::warn!("Vendors: a bad npcSbInfos entry {s}"),
        }
    }
}
