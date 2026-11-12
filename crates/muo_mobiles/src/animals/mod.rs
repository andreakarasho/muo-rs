//! The animals: the table classes in `leaves.rs` plus the hand-written ones.

pub mod leaves;

pub fn register(types: &mut muo_server::type_registry::TypeRegistry) {
    leaves::register(types);
}
