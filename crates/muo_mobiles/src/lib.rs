//! Monsters, animals and NPCs.

pub mod classes;
mod common;
#[rustfmt::skip] // generated, and tests/ctors.rs reads the text
mod ctor_tables;
pub(crate) mod hand;
mod props_gen;
#[macro_use]
mod leaf;
#[cfg(test)]
mod ai_tests;
pub mod animals;
pub mod monsters;
pub mod npcs;
#[cfg(test)]
mod tests;

mod template_keys;
mod template_keys_npcs;

/// Registers this crate's types and phases at startup.
pub fn register(types: &mut muo_server::type_registry::TypeRegistry) {
    template_keys::register(types);
    template_keys_npcs::register(types);
    types.register_phase_default(
        muo_server::type_registry::Phase::Configure,
        "muo_mobiles.PropsGen.Configure",
        props_gen::configure,
    );
    monsters::register(types);
    animals::register(types);
    npcs::register(types);
    classes::register(types);
    ctor_tables::register(types);
}
