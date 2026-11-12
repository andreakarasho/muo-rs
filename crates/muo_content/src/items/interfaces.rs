//! `IDyable`, `IScissorable` and `ICarvable`: the interface dispatch of the dye tub, the scissors and the knives.
//! A class lists itself with [`register_dyable`] / [`register_scissorable`] from a `Configure` phase; a class that
//! derives from a listed one is listed too (the lookup walks `ty.parent`).

use std::collections::HashMap;

use muo_server::World;
use muo_server::ids::{ItemId, MobileId};
use muo_server::item::ItemType;

/// `Dye(from, sender)`: (world, item, from, the dye tub).
pub type DyeFn = fn(&mut World, ItemId, MobileId, ItemId) -> bool;
/// `Scissor(from, scissors)`: (world, item, from, the scissors).
pub type ScissorFn = fn(&mut World, ItemId, MobileId, ItemId) -> bool;

/// `Carve(from, item)`: (world, item, from, the knife).
pub type CarveFn = fn(&mut World, ItemId, MobileId, ItemId);

#[derive(Default)]
pub struct ItemInterfaces {
    dye: HashMap<&'static str, DyeFn>,
    scissor: HashMap<&'static str, ScissorFn>,
    carve: HashMap<&'static str, CarveFn>,
}

pub fn register_dyable(w: &mut World, ty: &'static ItemType, f: DyeFn) {
    w.resources
        .get_mut::<ItemInterfaces>()
        .dye
        .insert(ty.full_name, f);
}

pub fn register_scissorable(w: &mut World, ty: &'static ItemType, f: ScissorFn) {
    w.resources
        .get_mut::<ItemInterfaces>()
        .scissor
        .insert(ty.full_name, f);
}

fn find<T: Copy>(
    w: &World,
    id: ItemId,
    pick: impl Fn(&ItemInterfaces) -> &HashMap<&'static str, T>,
) -> Option<T> {
    let table = pick(w.resources.get::<ItemInterfaces>()?);
    let mut t = Some(w.item(id)?.ty);
    while let Some(ty) = t {
        if let Some(f) = table.get(ty.full_name) {
            return Some(*f);
        }
        t = ty.parent;
    }
    None
}

/// `item is IDyable`: `Dye(from, tub)`'s result, or None when the item is not one.
pub fn dye(w: &mut World, id: ItemId, from: MobileId, tub: ItemId) -> Option<bool> {
    let f = find(w, id, |i| &i.dye)?;
    Some(f(w, id, from, tub))
}

/// `item is IScissorable`: `Scissor(from, scissors)`'s result, or None when the item is not one.
pub fn scissor(w: &mut World, id: ItemId, from: MobileId, scissors: ItemId) -> Option<bool> {
    let f = find(w, id, |i| &i.scissor)?;
    Some(f(w, id, from, scissors))
}

/// `item is IDyable`.
pub fn is_dyable(w: &World, id: ItemId) -> bool {
    find(w, id, |i| &i.dye).is_some()
}

/// `item is IScissorable`.
pub fn is_scissorable(w: &World, id: ItemId) -> bool {
    find(w, id, |i| &i.scissor).is_some()
}

pub fn register_carvable(w: &mut World, ty: &'static ItemType, f: CarveFn) {
    w.resources
        .get_mut::<ItemInterfaces>()
        .carve
        .insert(ty.full_name, f);
}

/// `item is ICarvable`: runs `Carve(from, knife)`; false when the item is not one.
pub fn carve(w: &mut World, id: ItemId, from: MobileId, knife: ItemId) -> bool {
    let Some(f) = find(w, id, |i| &i.carve) else {
        return false;
    };
    f(w, id, from, knife);
    true
}

/// `item is ICarvable`.
pub fn is_carvable(w: &World, id: ItemId) -> bool {
    find(w, id, |i| &i.carve).is_some()
}
