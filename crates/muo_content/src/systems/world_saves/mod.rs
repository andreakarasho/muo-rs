//! World saves: autosave, backups, archives.
pub mod archive;
pub mod archive_journal;
pub mod auto_archive;
pub mod auto_save;
pub mod local_time;
pub mod save_commands;

use muo_server::type_registry::{Phase, TypeRegistry};

pub fn register(t: &mut TypeRegistry) {
    // Before anything that logs or reads the local time.
    t.register_phase(
        Phase::Configure,
        "Server.Misc.LocalTime.Configure",
        100,
        local_time::configure,
    );
    t.register_phase_default(
        Phase::Configure,
        "Server.Saves.AutoSave.Configure",
        auto_save::configure,
    );
    t.register_phase_default(
        Phase::Configure,
        "Server.Saves.AutoArchive.Configure",
        auto_archive::configure,
    );
    t.register_phase_default(
        Phase::Initialize,
        "Server.Saves.AutoArchive.Initialize",
        auto_archive::initialize,
    );
    t.register_phase_default(
        Phase::Configure,
        "Server.Commands.SaveCommands.Configure",
        save_commands::configure,
    );
}
