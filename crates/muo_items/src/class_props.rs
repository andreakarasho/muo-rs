//! Constant per-class members of the non-equipment items, moved to their template's `"props"` (`"labelNumber"`,
//! `"defaultWeight"`, `"gumpId"`, ..). A class that had `label_number: |_, _| 1041100` now has
//! `label_number: class_props::label_number` and the value in `Data/Templates/Items`.
//!
//! Each member reads the prop of the item's class chain; without one (no table loaded, or a server owner removed
//! it) it falls back to the nearest Rust ancestor whose member is not this reader, as `..PARENT` did.

use muo_content::templates::{self, Kind, props};
use muo_server::World;
use muo_server::ids::ItemId;
use muo_server::item::ItemType;
use serde_json::Value;

macro_rules! member {
    ($(#[$m:meta])* $name:ident, $field:ident, $key:literal, $ret:ty, |$v:ident| $conv:expr) => {
        $(#[$m])*
        pub fn $name(w: &World, id: ItemId) -> $ret {
            if let Some(r) = props::item(w, id, $key).and_then(|$v| $conv) {
                return r;
            }
            let me: fn(&World, ItemId) -> $ret = $name;
            let mut t = w.item(id).map(|i| i.ty);
            while let Some(ty) = t {
                if !std::ptr::fn_addr_eq(ty.$field, me) {
                    return (ty.$field)(w, id);
                }
                t = ty.parent;
            }
            Default::default()
        }
    };
}

member!(
    /// `LabelNumber`: the `labelNumber` prop.
    label_number, label_number, "labelNumber", i32, |v| props::as_i32(Some(v))
);
member!(
    /// `DefaultWeight`: the `defaultWeight` prop.
    default_weight, default_weight, "defaultWeight", f64, |v| props::as_f64(Some(v))
);
member!(
    /// `DefaultName`: the `defaultName` prop.
    default_name, default_name, "defaultName", Option<String>, |v| v.as_str().map(|s| Some(s.to_owned()))
);
member!(
    /// `DefaultGumpID`: the `gumpId` prop.
    default_gump_id, default_gump_id, "gumpId", i32, |v| props::as_i32(Some(v))
);
member!(
    /// `DefaultDropSound`: the `dropSound` prop.
    default_drop_sound, default_drop_sound, "dropSound", i32, |v| props::as_i32(Some(v))
);
member!(
    /// `DefaultMaxItems`: the `maxItems` prop.
    default_max_items, default_max_items, "maxItems", i32, |v| props::as_i32(Some(v))
);
member!(
    /// `DefaultMaxWeight`: the `maxWeight` prop.
    default_max_weight, default_max_weight, "maxWeight", i32, |v| props::as_i32(Some(v))
);

member!(
    /// `Bounds` of a container: the `containerBounds` prop, `[x, y, width, height]`.
    bounds, bounds, "containerBounds", muo_primitives::Rectangle2D, |v| match v.as_array().map(Vec::as_slice) {
        Some([x, y, wd, h]) => Some(muo_primitives::Rectangle2D::new(
            props::as_i32(Some(x))?,
            props::as_i32(Some(y))?,
            props::as_i32(Some(wd))?,
            props::as_i32(Some(h))?,
        )),
        _ => None,
    }
);

/// `BaseLight.LitItemID`: the `litItemId` prop (0, `BaseLight`'s, without one).
pub fn lit_item_id(w: &World, id: ItemId) -> i32 {
    props::as_i32(props::item(w, id, "litItemId")).unwrap_or(0)
}

/// `BaseLight.UnlitItemID`: the `unlitItemId` prop (0, `BaseLight`'s, without one).
pub fn unlit_item_id(w: &World, id: ItemId) -> i32 {
    props::as_i32(props::item(w, id, "unlitItemId")).unwrap_or(0)
}

/// The prop `key` of class `ty` (its class chain), for callers that hold a class and no item.
pub fn of_class<'a>(w: &'a World, ty: &'static ItemType, key: &str) -> Option<&'a Value> {
    let t = templates::loaded(w)?;
    std::iter::successors(Some(ty), |c| c.parent)
        .find_map(|c| t.chain(Kind::Item, c.full_name).find_map(|tm| tm.props.get(key)))
}

/// A string prop of the item's class (`None` when no class in its chain sets one).
pub fn str_of(w: &World, id: ItemId, key: &str) -> Option<&'static str> {
    props::as_static_str(w, props::item(w, id, key)).flatten()
}

/// A list-of-strings prop of the item's class (empty when unset).
pub fn str_list(w: &World, id: ItemId, key: &str) -> Vec<&'static str> {
    match props::item(w, id, key) {
        Some(Value::Array(a)) => a
            .iter()
            .filter_map(|v| props::as_static_str(w, Some(v)).flatten())
            .collect(),
        _ => Vec::new(),
    }
}

/// A string prop of class `ty`.
pub fn class_str(w: &World, ty: &'static ItemType, key: &str) -> Option<&'static str> {
    props::as_static_str(w, of_class(w, ty, key)).flatten()
}
