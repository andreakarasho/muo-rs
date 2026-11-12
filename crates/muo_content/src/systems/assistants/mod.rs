//! Assistants: negotiating features with Razor/UOSteam.
pub mod assistant_configuration;
pub mod assistant_features;
pub mod assistant_handler;
pub mod assistant_protocol;

use muo_server::type_registry::{Phase, TypeRegistry};

pub fn register(t: &mut TypeRegistry) {
    t.register_phase_default(
        Phase::Configure,
        "Server.Assistants.AssistantConfiguration.Configure",
        assistant_configuration::configure,
    );
    // [CallPriority(10)]: the 0xF0 table exists before AssistantHandler registers into it.
    t.register_phase(
        Phase::Configure,
        "Server.Network.AssistantProtocol.Configure",
        10,
        assistant_protocol::configure,
    );
    t.register_phase_default(
        Phase::Configure,
        "Server.Assistants.AssistantHandler.Configure",
        assistant_handler::configure,
    );
}
