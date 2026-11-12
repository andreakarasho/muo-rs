//! Multis: houses, boats, camps, deeds.

pub mod boats;
pub mod camps;
pub mod classes;
pub mod component_verification;
pub mod contest;
pub mod deeds;
pub mod foundation;
pub mod foundation_packets;
pub mod hooks;
pub mod house;
pub mod house_entries;
pub mod house_packets;
pub mod moving_crate;
pub mod placement;
pub mod placement_entry;
pub mod placement_tool;
pub mod preview;
mod props_gen;
pub mod secure_entry;
pub mod secure_gump;
pub mod sign;
pub mod teleporter;
pub mod vendor_inventory;

use muo_server::type_registry::{Phase, TypeRegistry};

/// Registers this crate's types and phases at startup.
pub fn register(types: &mut TypeRegistry) {
    types.register_phase_default(
        muo_server::type_registry::Phase::Configure,
        "muo_multis.PropsGen.Configure",
        props_gen::configure,
    );
    classes::register(types);
    camps::register(types);
    boats::register(types);
    contest::register(types);
    sign::register(types);
    moving_crate::register(types);
    house::transfer::register(types);
    deeds::register(types);
    placement_tool::register(types);
    preview::register(types);
    teleporter::register(types);
    types.register_item(&foundation::HOUSE_FOUNDATION, &[]);
    types.register_phase_default(
        Phase::Configure,
        "Server.Multis.HouseFoundation.Initialize",
        foundation_packets::configure,
    );
    types.register_phase_default(
        Phase::Configure,
        "Server.Multis.BaseHouse.Configure",
        hooks::configure,
    );
    types.register_phase_default(
        Phase::Initialize,
        "Server.Multis.BaseHouse.Initialize",
        hooks::initialize,
    );
}

#[cfg(test)]
mod packet_tests;
#[cfg(test)]
mod tests;
