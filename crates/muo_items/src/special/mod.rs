//! Special items. `items.rs` holds the plain `Item`
//! classes that are constants only; classes with logic are hand-written modules next to it.

use muo_io::{GenericReader, GenericWriter, Result};
use muo_server::World;
use muo_server::ids::ItemId;
use muo_server::type_registry::{Phase, TypeRegistry};

pub mod addons;
pub mod anniv;
pub mod anniv2;
pub mod aquarium;
pub mod aquarium2;
pub mod boxes;
pub mod containers;
pub mod decor;
pub mod fish;
pub mod gifts;
pub mod hand;
pub mod holiday;
pub mod holiday2;
pub mod holiday3;
pub mod holiday4;
pub mod holiday5;
pub mod hooks;
pub mod items;
pub mod leaves;
pub mod mutation;
pub mod props;
pub mod raffle;
pub mod rewards2;
pub mod scrolls;
pub mod solen;
pub mod soulstone;
pub mod statuette;
pub mod toggle;
pub mod util;
pub mod vet;
pub mod vet2;
pub mod vet3;
pub mod vet4;
pub mod vet5;
pub mod water;
pub mod winter_gifts;

#[cfg(test)]
mod aquarium_tests;
#[cfg(test)]
mod holiday_tests;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests5;
#[cfg(test)]
mod vet_tests;

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

/// `ItemID = value`.
pub(crate) fn item_id_of(w: &mut World, id: ItemId, value: i32) {
    if let Some(ty) = w.item(id).map(|i| i.ty) {
        (ty.set_item_id)(w, id, value);
    }
}

/// `Hue = value`.
pub(crate) fn hue(w: &mut World, id: ItemId, value: i32) {
    if let Some(ty) = w.item(id).map(|i| i.ty) {
        (ty.set_hue)(w, id, value);
    }
}

/// Registers every class of this folder.
pub fn register(types: &mut TypeRegistry) {
    types.register_phase_default(
        muo_server::type_registry::Phase::Configure,
        "muo_items.AquariumProps.Configure",
        props::configure,
    );
    items::register(types);
    leaves::register(types);
    water::register(types);
    addons::register(types);
    decor::register(types);
    hand::register(types);
    types.register_item(&containers::CLOSED_BARREL, &[]);
    soulstone::register(types);
    boxes::register(types);
    anniv::register(types);
    anniv2::register(types);
    solen::register(types);
    mutation::register(types);
    gifts::register(types);
    scrolls::register(types);
    fish::register(types);
    aquarium::register(types);
    aquarium2::register(types);
    raffle::register(types);
    statuette::register(types);
    vet::register(types);
    vet2::register(types);
    vet3::register(types);
    vet4::register(types);
    vet5::register(types);
    holiday::register(types);
    holiday::register_globes(types);
    holiday::register_more(types);
    holiday2::register(types);
    holiday3::register(types);
    holiday4::register(types);
    holiday5::register(types);
    toggle::register(types);
    winter_gifts::register(types);
    rewards2::register(types);
    types.register_phase_default(
        Phase::Configure,
        "Server.Items.Special.Configure",
        configure,
    );
}

/// Hooks, `[Constructible]` overload tables and flips of the hand-written classes.
fn configure(w: &mut World) {
    soulstone::configure(w);
    statuette::configure(w);
    gifts::configure(w);
    anniv2::configure(w);
    solen::configure(w);
    mutation::configure(w);
    scrolls::configure(w);
    vet::configure(w);
    vet2::configure(w);
    vet3::configure(w);
    vet4::configure(w);
    vet5::configure(w);
    holiday::configure(w);
    holiday2::configure(w);
    holiday3::configure(w);
    holiday4::configure(w);
    holiday5::configure(w);
}
