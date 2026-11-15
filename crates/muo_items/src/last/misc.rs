//! `MelisandesFermentedWine`.

use muo_server::World;
use muo_server::ids::ItemId;
use muo_server::item::ItemType;
use muo_server::type_registry::TypeRegistry;

use crate::simple::{hue, item_id, random};
use crate::skill_items::potions::{GREATER_EXPLOSION_POTION, fx, new_potion};

/// `MelisandesFermentedWine()`: a `GreaterExplosionPotion` that is not stackable, with its own graphic and hue.
pub fn melisandes_fermented_wine_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = new_potion(w, ty, 0, fx::EXPLOSION_GREATER);
    crate::simple::own_template(w, id, "Server.Items.MelisandesFermentedWine");
    item_id(w, id, 0x99B);
    // TODO update
    let h = match random(w, 3) {
        1 => 0xF,
        2 => 0x48D,
        _ => 0xB,
    };
    hue(w, id, h);
    id
}

/// `Server.Items.MelisandesFermentedWine` (`CanDrink` is in `skill_items::potions::can_drink`).
pub static MELISANDES_FERMENTED_WINE: ItemType = ItemType {
    full_name: "Server.Items.MelisandesFermentedWine",
    parent: Some(&GREATER_EXPLOSION_POTION),
    construct: Some(|w| melisandes_fermented_wine_new(w, &MELISANDES_FERMENTED_WINE)),
    label_number: crate::class_props::label_number,
    get_properties: |w, id, list| {
        (GREATER_EXPLOSION_POTION.get_properties)(w, id, list);
        list.add(1074502); // It looks explosive.
        list.add(1075085); // Requirement: Mondain's Legacy
    },
    serialize: |w, id, out| {
        (GREATER_EXPLOSION_POTION.serialize)(w, id, out);
        out.write_i32(0);
    },
    deserialize: |w, id, r| {
        (GREATER_EXPLOSION_POTION.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        Ok(())
    },
    ..GREATER_EXPLOSION_POTION
};

/// Registers the classes of this file.
pub fn register(types: &mut TypeRegistry) {
    types.register_item(&MELISANDES_FERMENTED_WINE, &[]);
}
