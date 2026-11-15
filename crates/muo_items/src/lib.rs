//! The item classes of the server: one module per item family.

pub mod addons;
pub mod armor;
pub mod body_parts;
pub mod books;
pub mod bulletin;
pub mod champion_artifacts;
pub mod clothing;
pub mod commodity;
pub mod construction;
pub mod containers;
#[rustfmt::skip] // generated, and tests/ctors.rs reads the text
mod ctor_tables;
pub mod decorate_glue;
pub mod decoration_artifacts;
pub mod deeds;
pub mod facial;
pub mod farming;
pub mod food;
pub mod games;
pub mod gems;
pub mod guilds;
pub mod jewels;
pub mod last;
pub mod lights;
pub mod maps;
pub mod minor_artifacts;
pub mod misc;
pub mod new_haven_quest_rewards;
pub mod own_flips;
pub mod plants_flowers;
mod props_gen;
mod props_hand;
pub mod quivers;
pub mod resources;
pub mod simple;
pub mod skill_items;
pub mod small_items;
pub mod special;
pub mod suits;
pub mod talismans;
pub mod traps;
pub mod treasure_chests;
pub mod wands;
pub mod weapons;
pub mod wiring;

pub mod class_props;
mod template_keys_equipment;
mod template_keys_misc;

/// Registers this crate's types and phases at startup.
pub fn register(types: &mut muo_server::type_registry::TypeRegistry) {
    template_keys_equipment::register(types);
    template_keys_misc::register(types);
    types.register_phase_default(
        muo_server::type_registry::Phase::Configure,
        "muo_items.PropsGen.Configure",
        props_gen::configure,
    );
    types.register_phase_default(
        muo_server::type_registry::Phase::Configure,
        "muo_items.PropsHand.Configure",
        props_hand::configure,
    );
    ctor_tables::register(types);
    decorate_glue::register(types);
    addons::register(types);
    armor::register(types);
    guilds::register(types);
    games::register(types);
    maps::register(types);
    books::register(types);
    bulletin::register(types);
    clothing::register(types);
    jewels::register(types);
    gems::register(types);
    suits::register(types);
    quivers::register(types);
    wands::register(types);
    talismans::register(types);
    simple::register(types);
    last::register(types);
    small_items::register(types);
    misc::register(types);
    own_flips::register(types);
    special::register(types);
    weapons::register(types);
    skill_items::register(types);
    wiring::register(types);
}

/// A bare test world that knows this crate's template keys (a bare world's template load skips unknown keys).
#[cfg(test)]
pub(crate) fn keyed_world() -> muo_server::World {
    let mut w = muo_server::World::new();
    template_keys_misc::register(&mut w.types);
    muo_content::templates::table(&mut w); // class props read the loaded table
    w
}

#[cfg(test)]
mod leaf_clothing_tests;

#[cfg(test)]
mod simple_hand_tests;

#[cfg(test)]
mod flip_tests;

#[cfg(test)]
mod packet_tests;

#[cfg(test)]
mod template_tests;

#[cfg(test)]
mod template_tests_misc;
