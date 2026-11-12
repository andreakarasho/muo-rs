//! Skill items: the craft and harvest tools and what hangs off them. `tools.rs` holds the plain tools.

use muo_io::{GenericReader, GenericWriter, Result};
use muo_server::type_registry::TypeRegistry;

pub mod bandage;
pub mod camping;
pub mod dyes;
pub mod fishing;
pub mod forges;
pub mod hue_pickers;
pub mod instruments;
pub mod item_hooks;
pub mod misc;
pub mod ninja;
pub mod plain;
pub mod potion_keg;
pub mod potion_throw;
pub mod potions;
pub mod runebook;
pub mod runic_tools;
pub mod scrolls;
pub mod small;
pub mod spellbook;
#[rustfmt::skip]
mod scroll_list;
pub mod thief;
pub mod tools;
pub mod trophy;

#[cfg(test)]
mod tests;

/// The save version of a class with no fields of its own: `[SerializationGenerator(0, false)]` writes an int,
/// `[SerializationGenerator(0)]` an encoded int.
pub fn write_version(out: &mut dyn GenericWriter, encoded: bool) {
    if encoded {
        out.write_encoded_int(0);
    } else {
        out.write_i32(0);
    }
}

/// Reads what [`write_version`] wrote.
pub fn read_version(r: &mut dyn GenericReader, encoded: bool) -> Result<i32> {
    if encoded {
        r.read_encoded_int()
    } else {
        r.read_i32()
    }
}

/// `Hue = value`.
pub(crate) fn hue(w: &mut muo_server::World, id: muo_server::ids::ItemId, value: i32) {
    if let Some(ty) = w.item(id).map(|i| i.ty) {
        (ty.set_hue)(w, id, value);
    }
}

/// The item class named `full_name`.
pub(crate) fn item_type(
    w: &muo_server::World,
    full_name: &str,
) -> Option<&'static muo_server::item::ItemType> {
    match w.types.find_type_by_full_name(full_name, false)? {
        muo_server::type_registry::EntityType::Item(t) => Some(t),
        _ => None,
    }
}

/// Registers every class of this folder.
pub fn register(types: &mut TypeRegistry) {
    tools::register(types);
    camping::register(types);
    bandage::register(types);
    fishing::register(types);
    forges::register(types);
    trophy::register(types);
    misc::register(types);
    thief::register(types);
    runic_tools::register(types);
    plain::register(types);
    scrolls::register(types);
    potions::register(types);
    potion_keg::register(types);
    potion_throw::register(types);
    spellbook::register(types);
    runebook::register(types);
    instruments::register(types);
    small::register(types);
    dyes::register(types);
    ninja::register(types);
    item_hooks::register(types);
}
