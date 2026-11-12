//! Hand-written classes with logic of their own that belong to none of the other modules.

use muo_server::type_registry::TypeRegistry;

pub mod artifacts;
pub mod chests;
pub mod containers;
pub mod deeds;
pub mod food;
pub mod lights;
pub mod mark;
pub mod misc;

/// Registers the classes of this module.
pub fn register(types: &mut TypeRegistry) {
    artifacts::register(types);
    chests::register(types);
    containers::register(types);
    deeds::register(types);
    food::register(types);
    lights::register(types);
    mark::register(types);
    misc::register(types);
}

#[cfg(test)]
mod tests;
