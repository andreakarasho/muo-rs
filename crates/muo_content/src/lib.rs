//! Game content: mobiles, items, systems and the network handlers of the shard.

pub mod abilities;
pub mod accounting;
pub mod ai;
pub mod aos;
pub mod core;
pub mod craft;
pub mod creature;
#[rustfmt::skip] // generated, and tests/ctors.rs reads the text
mod ctor_tables;
pub mod factory;
pub mod gumps;
pub mod hooks;
pub mod http_tls;
pub mod items;
pub mod login;
pub mod loot;
pub mod loot_pack;
pub mod misc;
pub mod ml_quest_hooks;
pub mod mx;
pub mod network;
pub mod pathing;
pub mod player_mobile;
pub mod props;
pub mod props_ctx;
mod props_gen;
pub mod props_hosts;
pub mod skills_props;
mod systems;
pub mod template_keys;
pub mod templates;
pub mod vendor;
pub mod wiring;
pub use systems::*;

/// Registers every content class and phase function into the type registry.
pub fn register(types: &mut muo_server::type_registry::TypeRegistry) {
    ctor_tables::register(types);
    systems::register(types);
    player_mobile::register(types);
    items::leaves::register(types);
    items::common::register(types);
    login::register(types);
    creature::register(types);
    templates::register(types);
    pathing::register(types);
    vendor::register(types);
    types.register_phase_default(
        muo_server::type_registry::Phase::Configure,
        "Server.Commands.Properties.RegisterCoreProps",
        props::register_core,
    );
    types.register_phase_default(
        muo_server::type_registry::Phase::Configure,
        "muo_content.PropsCtx.Configure",
        props_ctx::configure,
    );
    types.register_phase_default(
        muo_server::type_registry::Phase::Configure,
        "Wiring.Configure",
        wiring::configure,
    );
    types.register_phase_default(
        muo_server::type_registry::Phase::Configure,
        "Server.Misc.LanguageStatistics.Configure",
        misc::language_statistics::configure,
    );
}
