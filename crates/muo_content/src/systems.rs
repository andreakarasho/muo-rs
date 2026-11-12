//! The "systems" round: Spells/Base, Skills, Targets, Regions, Context Menus, Systems, Text, Utilities,
//! Configuration, Console, Compression, Assistants, World Saves, Special Systems. One file so that the shared
//! `lib.rs` needs one line.

// Style lints that fight the shape of the game rules: nested `if`s that mirror the order of the checks,
// vtable fn-pointer types, and signatures that take every argument a rule reads.

pub mod assistants;
pub mod compression;
pub mod configuration;
pub mod console;
pub mod context_menus;
pub mod jail;
pub mod regions;
pub mod skills;
pub mod special_systems;
pub mod spells;
pub mod targets;
pub mod text;
pub mod utilities;
pub mod world_saves;

use muo_server::type_registry::{Phase, TypeRegistry};

pub fn register(t: &mut TypeRegistry) {
    assistants::register(t);
    world_saves::register(t);
    regions::register(t);
    spells::register(t);
    skills::register(t);
    special_systems::register(t);
    register_login(t);
    t.register_phase_default(
        Phase::Configure,
        "Server.Systems.JailSystem.JailSystem.Configure",
        jail::configure,
    );
    t.register_phase_default(
        Phase::Configure,
        "Server.ExpansionConfiguration.Configure",
        configuration::expansion_configuration::configure,
    );
    t.register_phase_default(
        Phase::Configure,
        "Server.Configurations.EmailConfiguration.Configure",
        configuration::email_configuration::configure,
    );
    t.register_phase_default(
        Phase::Configure,
        "Server.Misc.ConsoleCommands.Configure",
        console::configure,
    );
}

/// The `[OnEvent(nameof(PlayerMobile.PlayerLoginEvent))]` handlers of this round, subscribed in an `Initialize`
/// phase. (`PreventInaccess` subscribes its own in its `Initialize`.)
fn subscribe_login_handlers(w: &mut muo_server::World) {
    use crate::network::events::{PlayerLogin, subscribe_ranked};
    subscribe_ranked::<PlayerLogin>(w, 2, |w, e| assistants::assistant_handler::on_login(w, e.0));
    subscribe_ranked::<PlayerLogin>(w, 20, |w, e| skills::anti_macro::on_login(w, e.0));
    subscribe_ranked::<PlayerLogin>(w, 18, |w, e| regions::house::on_login(w, e.0));
    subscribe_ranked::<PlayerLogin>(w, 19, |w, e| {
        regions::simple::twisted_weald_on_login(w, e.0)
    });
    subscribe_ranked::<PlayerLogin>(w, 21, |w, e| special_systems::gift_giving::on_login(w, e.0));
    // `PetLoginHandler.OnLogin`
    subscribe_ranked::<PlayerLogin>(w, 15, |w, e| {
        crate::ai::speech::derive_follower_orders(w, e.0)
    });
}

pub fn register_login(t: &mut TypeRegistry) {
    t.register_phase_default(
        Phase::Initialize,
        "Server.Systems.LoginHandlers.Initialize",
        subscribe_login_handlers,
    );
}
