//! The construction item classes. `leaves.rs` holds the plain table classes; the classes with logic
//! are hand-written in this module.

use muo_server::type_registry::TypeRegistry;

pub mod door_generator;
pub mod doors;
pub mod leaves;
pub mod paired;
pub mod walls;

#[cfg(test)]
mod house_tests;

/// Registers the classes of this folder.
pub fn register(types: &mut TypeRegistry) {
    leaves::register(types);
    doors::register(types);
    door_generator::register(types);
    paired::register(types);
    walls::register(types);
}
