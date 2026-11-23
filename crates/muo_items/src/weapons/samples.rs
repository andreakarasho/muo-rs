//! Constructors of the three sample leaves of `muo_content::items::leaves` (whose `construct` closures fix the
//! type), for the classes derived from them: `new Longsword()`, `new Katana()`, `new Bow()` for a derived `ty`. The
//! graphic and the constructor statements are the sample's template.

use muo_content::items::equip_props::graphic;

use super::*;

/// `new T()` of sample `of` for class `ty`: `base(graphic)`, then the sample's template.
fn sample_new(w: &mut World, ty: &'static ItemType, of: &'static ItemType) -> ItemId {
    let g = graphic(w, of);
    let id = new_weapon(w, ty, g);
    muo_content::templates::apply_item_as(w, id, of.full_name);
    id
}

/// `Longsword()`.
pub fn longsword_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    sample_new(w, ty, &LONGSWORD)
}

/// `Katana()`.
pub fn katana_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    sample_new(w, ty, &KATANA)
}

/// `Bow()`: `Layer = Layer.TwoHanded; Resource = CraftResource.RegularWood;` are its template.
pub fn bow_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    sample_new(w, ty, &BOW)
}
