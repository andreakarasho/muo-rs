//! The gumps M to Z, and the props gumps.

pub mod common;
pub mod hooks;
pub mod house_ops;
pub mod houses;
pub mod jail;
pub mod ops;
pub mod player_vendor;
pub mod props;
pub mod resurrect;
pub mod reward;
pub mod runebook;
pub mod simple;
pub mod skills;
pub mod tracking;
pub mod vendor_rental;
pub mod virtual_check;
pub mod warning;
pub mod who;

use muo_server::type_registry::{Phase, TypeRegistry};

pub fn register(t: &mut TypeRegistry) {
    t.register_phase_default(
        Phase::Configure,
        "Server.Gumps.StaticWarningGump.Configure",
        warning::configure,
    );
    t.register_phase_default(
        Phase::Configure,
        "Server.Gumps.PropertiesGump.Configure",
        props::configure,
    );
    t.register_phase_default(
        Phase::Configure,
        "Server.Gumps.ToTAdminGump.Configure",
        simple::tot_admin_configure,
    );
    t.register_phase_default(
        Phase::Configure,
        "Server.Gumps.RenounceYoungGump.Configure",
        simple::configure_renounce_young,
    );
    t.register_phase_default(
        Phase::Configure,
        "Server.Gumps.WhoGump.Configure",
        who::configure,
    );
    t.register_phase_default(
        Phase::Configure,
        "Server.Mobiles.PlayerMobile.GumpsConfigure",
        simple::player_mobile_gumps_configure,
    );
    t.register_phase_default(
        Phase::Configure,
        "Server.Gumps.SelectAddonDirectionGump.Configure",
        simple::select_addon_direction_configure,
    );
    t.register_phase_default(
        Phase::Configure,
        "Server.Systems.JailSystem.JailRecordGump.Configure",
        jail::configure,
    );
    t.register_phase_default(
        Phase::Configure,
        "Server.SkillHandlers.TrackWhatGump.Configure",
        tracking::configure,
    );
    t.register_phase_default(
        Phase::Configure,
        "Server.Items.VirtualCheck.Configure",
        virtual_check::configure,
    );
    t.register_phase_default(
        Phase::Configure,
        "Server.Gumps.ViewHousesGump.Configure",
        houses::view_houses_configure,
    );
    t.register_phase_default(
        Phase::Configure,
        "Server.Gumps.VendorInventoryGump.Configure",
        houses::configure,
    );
}

#[cfg(test)]
mod tests;
