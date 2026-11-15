//! Constant class overrides moved to JSON: a template's `"props": { "corpseName": "an orcish corpse", .. }`.
//!
//! A vtable member that used to be `|_w, _id| <constant>` becomes a function that reads the prop of the entity's
//! class here. The lookup walks the class chain (the class, then its Rust parent classes), and for each class its
//! template `inherits` chain, so a subclass without the prop gets its parent's, as `..PARENT` did.

use serde_json::Value;

use muo_server::ids::{ItemId, MobileId};
use muo_server::world::World;

use super::{Kind, TemplateTable, loaded};

fn find<'a>(
    t: &'a TemplateTable,
    kind: Kind,
    names: impl Iterator<Item = &'static str>,
    key: &str,
) -> Option<&'a Value> {
    for n in names {
        for tmpl in t.chain(kind, n) {
            if let Some(v) = tmpl.props.get(key) {
                return Some(v);
            }
        }
    }
    None
}

/// The prop `key` of the mobile's class, `None` when no class in its chain sets it or no table is loaded.
pub fn mobile<'a>(w: &'a World, id: MobileId, key: &str) -> Option<&'a Value> {
    let ty = w.mobile(id)?.ty;
    let names = std::iter::successors(Some(ty), |t| t.parent).map(|t| t.full_name);
    find(loaded(w)?, Kind::Mobile, names, key)
}

/// The prop `key` of the item's class.
pub fn item<'a>(w: &'a World, id: ItemId, key: &str) -> Option<&'a Value> {
    let ty = w.item(id)?.ty;
    let names = std::iter::successors(Some(ty), |t| t.parent).map(|t| t.full_name);
    find(loaded(w)?, Kind::Item, names, key)
}

pub fn as_i32(v: Option<&Value>) -> Option<i32> {
    v?.as_i64().and_then(|n| i32::try_from(n).ok())
}

pub fn as_f64(v: Option<&Value>) -> Option<f64> {
    v?.as_f64()
}

pub fn as_bool(v: Option<&Value>) -> Option<bool> {
    v?.as_bool()
}

/// A string prop as `&'static str` (every string prop is interned when the table loads). An explicit JSON `null`
/// is `Some(None)`: the class overrides the member to return nothing.
pub fn as_static_str(w: &World, v: Option<&Value>) -> Option<Option<&'static str>> {
    match v? {
        Value::Null => Some(None),
        Value::String(s) => Some(loaded(w)?.interned.get(s.as_str()).copied()),
        _ => None,
    }
}

/// Interns every string prop of the table, so [`as_static_str`] can hand them out.
pub(super) fn intern_all(t: &mut TemplateTable) {
    let mut strs = Vec::new();
    for tmpl in t.mobiles.values().chain(t.items.values()) {
        for v in tmpl.props.values() {
            collect(v, &mut strs);
        }
    }
    for s in strs {
        t.intern(&s);
    }
}

fn collect(v: &Value, out: &mut Vec<String>) {
    match v {
        Value::String(s) => out.push(s.clone()),
        Value::Array(a) => a.iter().for_each(|v| collect(v, out)),
        Value::Object(o) => o.values().for_each(|v| collect(v, out)),
        _ => {}
    }
}
