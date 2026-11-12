//! The staff commands and advanced search. The property tables the commands rely on are `muo_content::props`.

// Style lints that fight the shape of the commands (nested `if`s that state each check, command constructors that
// take no arguments, handlers that take many arguments).

pub mod add;
pub mod add_menus;
pub mod advanced_search;
pub mod batch;
pub mod decorate;
pub mod file_commands;
pub mod generic;
pub mod handlers;
pub mod handlers_rest;
pub mod help_info;
pub mod hooks;
pub mod location_command;
pub mod logging;
pub mod misc_commands;
pub mod network_commands;
pub mod properties;
pub mod teleporters;

use muo_server::type_registry::{Phase, TypeRegistry};

/// Registers this crate's types and phases at startup.
pub fn register(types: &mut TypeRegistry) {
    types.register_phase_default(
        Phase::Configure,
        "Server.Commands.Generic.TargetCommands.Configure",
        generic::configure,
    );
    types.register_phase_default(
        Phase::Configure,
        "Server.Network.NetworkCommands.Configure",
        network_commands::configure,
    );
    types.register_phase_default(
        Phase::Configure,
        "Server.Commands.Properties.Configure",
        properties::configure,
    );
    types.register_phase_default(
        Phase::Configure,
        "Server.Commands.CommandHandlers.Configure",
        handlers::configure,
    );
    types.register_phase_default(
        Phase::Configure,
        "Server.Commands.MiscCommands.Configure",
        misc_commands::configure,
    );
    types.register_phase_default(
        Phase::Configure,
        "Server.Commands.Decorate.Configure",
        decorate::configure,
    );
    types.register_phase_default(
        Phase::Configure,
        "Server.Commands.Batch.Configure",
        batch::configure,
    );
    types.register_phase_default(
        Phase::Configure,
        "Server.Commands.FileCommands.Configure",
        file_commands::configure,
    );
    types.register_phase_default(
        Phase::Configure,
        "Server.Commands.HelpInfo.Configure",
        help_info::configure,
    );
    types.register_phase_default(
        Phase::Initialize,
        "Server.Commands.HelpInfo.Initialize",
        help_info::initialize,
    );
    types.register_phase_default(
        Phase::Configure,
        "Server.Engines.AdvancedSearch.AdvancedSearchCommand.Configure",
        advanced_search::configure,
    );
    types.register_phase_default(
        Phase::Configure,
        "Server.Commands.GenTeleporter.Configure",
        teleporters::configure,
    );
    types.register_phase_default(
        Phase::Configure,
        "Server.Gumps.AddGump.Configure",
        add_menus::configure,
    );
    types.register_phase_default(
        Phase::Configure,
        "Server.Commands.Add.Configure",
        add::configure,
    );
    types.register_phase_default(
        Phase::Initialize,
        "Server.Commands.CommandLogging.Initialize",
        logging::initialize,
    );
}

#[cfg(test)]
mod behavior_tests;
#[cfg(test)]
mod tests;
