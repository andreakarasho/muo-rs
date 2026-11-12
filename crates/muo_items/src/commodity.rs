//! `ICommodity`: the classes a commodity deed can be filled with. The generated
//! folders list theirs in `IMPLEMENTS` / `COMMODITIES`; classes written by hand call [`register`].
//! `IsDeedable` can depend on the expansion (`DaemonBone`: `Core.ML`), so it is a function.

use std::collections::HashMap;

use muo_server::World;
use muo_server::ids::ItemId;
use muo_server::item::ItemType;

/// `IsDeedable` of the commodity classes by full name.
#[derive(Default)]
pub struct Commodities(HashMap<&'static str, fn(&World) -> bool>);

/// Declares `ty` (and so every class derived from it) an `ICommodity`.
pub fn register(w: &mut World, ty: &'static ItemType, deedable: fn(&World) -> bool) {
    w.resources
        .get_mut::<Commodities>()
        .0
        .insert(ty.full_name, deedable);
}

/// `item as ICommodity`: `Some(IsDeedable)` for a commodity, `None` otherwise.
pub fn deedable(w: &World, id: ItemId) -> Option<bool> {
    let table = w.resources.get::<Commodities>()?;
    let mut t = Some(w.item(id)?.ty);
    while let Some(ty) = t {
        if let Some(f) = table.0.get(ty.full_name) {
            return Some(f(w));
        }
        t = ty.parent;
    }
    None
}

/// Whether `ty` or a base class lists `interface` in the generated `IMPLEMENTS` tables.
pub fn implements(ty: &'static ItemType, interface: &str) -> bool {
    let mut t = Some(ty);
    while let Some(c) = t {
        if crate::simple::implements_tables().any(|(i, n)| std::ptr::eq(i, c) && n == interface) {
            return true;
        }
        t = c.parent;
    }
    false
}

/// Configure phase: the generated `COMMODITIES` rows and the classes whose `ICommodity` comes from `IMPLEMENTS`.
pub fn configure(w: &mut World) {
    for (ty, deed) in crate::simple::commodity_tables() {
        register(w, ty, if deed { |_| true } else { |_| false });
    }
    // The resource bases of muo_content (`BaseIngot : ICommodity`, ..): `IsDeedable => true` for all of them.
    {
        use muo_content::items::resource::{
            BASE_HIDES, BASE_INGOT, BASE_LEATHER, BASE_SCALES, BOARD, LOG,
        };
        for ty in [
            &BASE_INGOT,
            &BASE_LEATHER,
            &BASE_HIDES,
            &BASE_SCALES,
            &BOARD,
            &LOG,
        ] {
            register(w, ty, |_| true);
        }
    }
    for (ty, iface) in crate::simple::implements_tables() {
        if iface == "ICommodity"
            && w.resources
                .get::<Commodities>()
                .is_none_or(|c| !c.0.contains_key(ty.full_name))
        {
            register(w, ty, |_| true);
        }
    }
}
