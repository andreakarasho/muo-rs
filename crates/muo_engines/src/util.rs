//! Small helpers the engines of this crate share.

use muo_primitives::utility;
use muo_server::World;
use muo_server::ids::ItemId;

/// `Hue = value`, through the class's own setter.
pub fn hue(w: &mut World, id: ItemId, value: i32) {
    if let Some(ty) = w.item(id).map(|i| i.ty) {
        (ty.set_hue)(w, id, value);
    }
}

/// `list.RandomElement()`.
pub fn random_element<'a, T>(w: &mut World, list: &'a [T]) -> &'a T {
    &list[utility::random(&mut w.rng, list.len() as i32) as usize]
}
