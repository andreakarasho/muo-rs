//! Template keys of the non-equipment items (`muo_content::templates::keys`).

use muo_content::templates::keys::{check_int, int, item_key};
use muo_server::World;
use muo_server::ids::ItemId;
use muo_server::type_registry::TypeRegistry;

/// The family state `T` of an item, to set one field of it.
fn ext_mut<T: 'static>(w: &mut World, id: ItemId) -> Option<&mut T> {
    w.item_mut(id).and_then(|i| i.ext.get_mut::<T>())
}

pub(crate) fn register(types: &mut TypeRegistry) {
    // `SpellScroll.SpellID`.
    item_key(types, "spellId", check_int, |w, id, v| {
        crate::skill_items::scrolls::set_spell_id(w, id, int(v));
    });
    // `Food.FillFactor`.
    item_key(types, "foodFillFactor", check_int, |w, id, v| {
        muo_content::items::food::set_fill_factor(w, id, int(v));
    });
    // `CookableFood.CookingLevel`.
    item_key(types, "cookingLevel", check_int, |w, id, v| {
        if let Some(d) = ext_mut::<crate::food::cooking::CookData>(w, id) {
            d.level = int(v);
        }
    });
}
