//! The containers item classes. `leaves.rs` holds the plain table classes; the classes with logic
//! are hand-written in this module.

use muo_server::type_registry::TypeRegistry;

pub mod fillable;
pub mod fillable_data;
pub mod leaves;
pub mod strong_box;

/// Registers the classes of this folder.
pub fn register(types: &mut TypeRegistry) {
    leaves::register(types);
    fillable::register(types);
    strong_box::register(types);
}
