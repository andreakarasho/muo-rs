//! The addon classes. `leaves.rs` holds the addons,
//! components and deeds that are constants and `AddComponent` rows); the classes with logic are hand-written next to it.

pub mod devices;
pub mod hand;
pub mod hive;
pub mod leaves;

#[cfg(test)]
mod tests;

/// Registers every class of this folder.
pub fn register(types: &mut muo_server::type_registry::TypeRegistry) {
    devices::register(types);
    hive::register(types);
    hand::register(types);
    leaves::register(types);
}
