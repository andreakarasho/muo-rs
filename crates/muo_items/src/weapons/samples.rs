//! Constructors of the three sample leaves of `muo_content::items::leaves` (whose `construct` closures fix the
//! type), for the classes derived from them: `new Longsword()`, `new Katana()`, `new Bow()` for a derived `ty`.

use super::*;

/// `Longsword()`: `base(0xF61)`.
pub fn longsword_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    new_weapon(w, ty, 0xF61)
}

/// `Katana()`: `base(0x13FF)`.
pub fn katana_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    new_weapon(w, ty, 0x13FF)
}

/// `Bow()`: `base(0x13B2)`, then `Layer = Layer.TwoHanded; Resource = CraftResource.RegularWood;`.
pub fn bow_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = new_weapon(w, ty, 0x13B2);
    ctor::set_layer(w, id, Layer::TwoHanded);
    set_resource(w, id, CraftResource::RegularWood);
    id
}
