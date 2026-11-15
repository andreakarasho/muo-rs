//! Monsters, animals and NPCs.

pub mod classes;
mod common;
pub mod cprops;
#[rustfmt::skip] // generated, and tests/ctors.rs reads the text
mod ctor_tables;
pub(crate) mod hand;
mod props_gen;
#[macro_use]
mod leaf;
#[cfg(test)]
mod ai_tests;
pub mod animals;
#[cfg(test)]
mod creature_template_tests;
pub mod monsters;
pub mod npcs;
#[cfg(test)]
mod tests;

mod template_keys;
mod template_keys_npcs;

/// `World::new()` knowing this crate's template keys, with the class templates loaded once per thread and shared
/// (a test building a world per class would otherwise compile every class script each time).
#[cfg(test)]
pub(crate) fn test_world() -> muo_server::World {
    fn keyed() -> muo_server::World {
        let mut w = muo_server::World::new();
        template_keys::register(&mut w.types);
        template_keys_npcs::register(&mut w.types);
        w
    }
    thread_local! {
        static LOADED: muo_server::World = {
            let mut w = keyed();
            muo_content::templates::table(&mut w);
            w
        };
    }
    let mut w = keyed();
    LOADED.with(|l| muo_content::scripting::share_loaded(l, &mut w));
    w
}

/// Registers this crate's types and phases at startup.
pub fn register(types: &mut muo_server::type_registry::TypeRegistry) {
    template_keys::register(types);
    cprops::register(types);
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
