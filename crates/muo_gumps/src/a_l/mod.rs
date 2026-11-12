//! The gumps A to L.

pub mod add_door;
pub mod admin;
pub mod ban_duration;
pub mod base_confirm;
pub mod base_image_tile_buttons;
pub mod client;
pub mod comments;
pub mod common;
pub mod confirm_break_crystal;
pub mod confirm_heritage;
pub mod confirm_release;
pub mod go;
pub mod guilds;
pub mod heritage_token;
pub mod hooks;
pub mod house_gump;
pub mod houses;
pub mod interface;
pub mod new_guilds;
#[cfg(test)]
mod tests;

use muo_server::type_registry::{Phase, TypeRegistry};

pub fn register(t: &mut TypeRegistry) {
    t.register_phase_default(
        Phase::Configure,
        "Server.Gumps.AlHooks.Configure",
        hooks::configure,
    );
    t.register_phase_default(
        Phase::Configure,
        "Server.Gumps.AddDoorGump.Configure",
        add_door::configure,
    );
    t.register_phase_default(
        Phase::Configure,
        "Server.Gumps.AdminGump.Configure",
        admin::configure,
    );
    t.register_phase_default(
        Phase::Configure,
        "Server.Gumps.CommentsGump.Configure",
        comments::configure,
    );
    t.register_phase_default(
        Phase::Configure,
        "Server.Gumps.ConfirmReleaseGump.Configure",
        confirm_release::configure,
    );
}
