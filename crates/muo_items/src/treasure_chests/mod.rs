//! The treasure chests item classes. `leaves.rs` holds the plain table classes; the classes with logic
//! are hand-written in this module.

use muo_server::type_registry::TypeRegistry;

pub mod chests;
pub mod leaves;

/// Registers the classes of this folder.
pub fn register(types: &mut TypeRegistry) {
    leaves::register(types);
    chests::register(types);
}
