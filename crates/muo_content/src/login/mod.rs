//! The login path: Accounting, Network, the Misc/ files a login uses, and character creation.
//! `register` adds their `Configure` and `Initialize` functions to the type registry, in load order
//! (accounting first: `Accounts.Configure` creates the persistence the others load into).

pub mod account_prompt;
pub mod character_creation;
pub mod client_verification;
pub mod crash_guard;
pub mod login_stats;
pub mod profession_info;
pub mod server_access;
pub mod server_list;
pub mod shard_poller;
pub mod small;
pub mod welcome_timer;

use muo_server::type_registry::{Phase, TypeRegistry};

use crate::accounting::{self, handler, persistence::AccountsPersistence};
use crate::network;

pub fn register(t: &mut TypeRegistry) {
    use Phase::{Configure, Initialize};
    shard_poller::register(t);
    // priorities: lower first; Accounts and AccountSecurity before everything that reads them
    t.register_phase(
        Configure,
        "Server.Accounting.Accounts.Configure",
        10,
        AccountsPersistence::register,
    );
    t.register_phase(
        Configure,
        "Server.World.Guilds.Configure",
        10,
        crate::misc::guild_persistence::GuildsPersistence::register,
    );
    t.register_phase_default(
        Configure,
        "Server.Guilds.Guild.Configure",
        crate::misc::guild_class::configure,
    );
    t.register_phase_default(
        Initialize,
        "Server.Guilds.GuildMaintenanceTimer.Initialize",
        crate::misc::guild_class::maintenance_initialize,
    );
    t.register_phase(
        Configure,
        "Server.Accounting.Security.AccountSecurity.Configure",
        10,
        handler::configure,
    );
    t.register_phase_default(
        Configure,
        "Server.Misc.ServerList.Configure",
        server_list::configure,
    );
    t.register_phase_default(
        Configure,
        "Server.Misc.ServerAccess.Configure",
        server_access::configure,
    );
    t.register_phase_default(
        Configure,
        "Server.Misc.ClientVerification.Configure",
        client_verification::configure,
    );
    t.register_phase_default(
        Configure,
        "Server.Misc.CrashGuard.Configure",
        crash_guard::configure,
    );
    t.register_phase_default(
        Configure,
        "Server.Misc.AutoRestart.Configure",
        small::auto_restart_configure,
    );
    t.register_phase_default(
        Configure,
        "Server.Network.PacketThrottles.Configure",
        small::packet_throttles_configure,
    );
    t.register_phase_default(
        Configure,
        "Server.Network.IncomingPackets.Configure",
        network::configure,
    );
    t.register_phase_default(
        Configure,
        "Server.Gumps.GumpSystem.Configure",
        crate::gumps::system::configure,
    );

    t.register_phase_default(
        Initialize,
        "Server.Accounting.Account.Initialize",
        accounting::initialize,
    );
    t.register_phase_default(
        Initialize,
        "Server.Misc.AccountHandler.Initialize",
        handler::initialize,
    );
    t.register_phase_default(
        Initialize,
        "Server.Misc.ServerList.Initialize",
        server_list::initialize,
    );
    t.register_phase_default(
        Initialize,
        "Server.Misc.ServerAccess.Initialize",
        server_access::initialize,
    );
    t.register_phase_default(
        Initialize,
        "Server.Misc.ClientVerification.Initialize",
        client_verification::initialize,
    );
    t.register_phase_default(
        Initialize,
        "Server.Misc.AccountPrompt.Initialize",
        account_prompt::initialize,
    );
    t.register_phase_default(
        Initialize,
        "Server.Misc.WelcomeTimer.Initialize",
        welcome_timer::initialize,
    );
    t.register_phase_default(
        Initialize,
        "Server.Misc.CrashGuard.Initialize",
        crash_guard::initialize,
    );
    t.register_phase_default(
        Initialize,
        "Server.Misc.Paperdoll.Initialize",
        small::paperdoll_initialize,
    );
    t.register_phase_default(
        Initialize,
        "Server.Misc.Keywords.Initialize",
        small::keywords_initialize,
    );
    t.register_phase_default(
        Initialize,
        "Server.ValidationQueue.Initialize",
        small::validation_queue_initialize,
    );
    t.register_phase_default(
        Initialize,
        "Server.Network.PacketThrottles.Initialize",
        small::packet_throttles_initialize,
    );
    t.register_phase_default(
        Initialize,
        "Server.Engines.CharacterCreation.CharacterCreation.Initialize",
        character_creation::initialize,
    );
    // LoginStats.OnLogin is not a PlayerLoginEvent subscriber: the server never sends the welcome line.
    t.register_phase_default(Initialize, "Server.Network.Initialize", network::initialize);
}
