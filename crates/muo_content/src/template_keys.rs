//! Class props of item classes whose constant members moved to `Data/Scripts/classes/items` (`"props"`), and the
//! constructor of a class that is only its template. A member reads its prop through the class chain; without the
//! prop it answers what its Rust parent answers (`..PARENT`). Shared by the content crates above.

use muo_server::ids::ItemId;
use muo_server::item::ItemType;
use muo_server::world::World;

use crate::templates::props;

/// `new T()` of an `Item` class whose constructor is its template (the graphic is the template's `itemId`).
pub fn template_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = muo_server::item::ops::new_item_with_id(w, ty, 0);
    crate::templates::apply_item_as(w, id, ty.full_name);
    id
}

/// An int member (`labelNumber`, `defaultGumpId`, ..).
pub fn item_int(w: &World, id: ItemId, key: &str, parent: fn(&World, ItemId) -> i32) -> i32 {
    props::as_i32(props::item(w, id, key)).unwrap_or_else(|| parent(w, id))
}

/// A number member (`defaultWeight`).
pub fn item_float(w: &World, id: ItemId, key: &str, parent: fn(&World, ItemId) -> f64) -> f64 {
    props::as_f64(props::item(w, id, key)).unwrap_or_else(|| parent(w, id))
}

/// A bool member (`nontransferable`, ..).
pub fn item_bool(w: &World, id: ItemId, key: &str, parent: fn(&World, ItemId) -> bool) -> bool {
    props::as_bool(props::item(w, id, key)).unwrap_or_else(|| parent(w, id))
}

/// `DefaultName`: the `defaultName` prop (`null` for none).
pub fn item_name(
    w: &World,
    id: ItemId,
    parent: fn(&World, ItemId) -> Option<String>,
) -> Option<String> {
    match props::as_static_str(w, props::item(w, id, "defaultName")) {
        Some(s) => s.map(str::to_owned),
        None => parent(w, id),
    }
}
