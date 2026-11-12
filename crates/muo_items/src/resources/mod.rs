//! The resources item classes. `leaves.rs` holds the plain table classes; the classes with logic
//! are hand-written in this module.

use muo_server::type_registry::TypeRegistry;

pub mod cloth;
pub mod leaves;
pub mod reagents;

/// Registers the classes of this folder.
pub fn register(types: &mut TypeRegistry) {
    leaves::register(types);
    reagents::register(types);
    cloth::register(types);
}
