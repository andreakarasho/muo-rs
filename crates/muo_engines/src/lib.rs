//! Game engines: factions, quests, spawners, crafting, harvesting, champions and the other systems.

#[cfg(test)]
mod behavior_tests;
pub mod bulk_orders;
pub mod canned_evil;
pub mod chat;
pub mod cleanup;
pub mod conpvp;
pub mod craft;
#[rustfmt::skip] // generated, and tests/ctors.rs reads the text
mod ctor_tables;
pub mod decorate_glue;
pub mod doom;
pub mod engines_b_props;
#[cfg(test)]
mod engines_b_tests;
pub mod ethics;
pub mod events;
pub mod faction_items;
pub mod factions;
pub mod feature_flags;
pub mod glue;
pub mod hardware_info;
pub mod harvest;
pub mod help;
pub mod insurance;
pub mod khaldun;
pub mod ml_quests;
pub mod murder;
pub mod party;
pub mod plants;
mod props_gen;
mod props_p3;
pub mod quests;
pub mod rewards;
pub mod spawners;
pub mod special_gumps;
pub mod special_wiring;
pub mod stealables;
pub mod tokuno;
pub mod util;
pub mod virtues;
pub mod wiring2;

/// Registers this crate's types and phases at startup.
pub fn register(types: &mut muo_server::type_registry::TypeRegistry) {
    types.register_phase_default(
        muo_server::type_registry::Phase::Configure,
        "muo_engines.PropsGen.Configure",
        props_gen::configure,
    );
    types.register_phase_default(
        muo_server::type_registry::Phase::Configure,
        "muo_engines.PropsP3.Configure",
        props_p3::configure,
    );
    stealables::register(types);
    ctor_tables::register(types);
    decorate_glue::register(types);
    craft::register(types);
    bulk_orders::register(types);
    conpvp::register(types);
    harvest::register(types);
    ml_quests::register(types);
    quests::register(types);
    spawners::register(types);
    murder::register(types);
    tokuno::register(types);
    canned_evil::register(types);
    doom::register(types);
    rewards::register(types);
    khaldun::register(types);
    plants::register(types);
    types.register_phase_default(
        muo_server::type_registry::Phase::Configure,
        "EnginesB.Props",
        engines_b_props::configure,
    );
    types.register_phase_default(
        muo_server::type_registry::Phase::Configure,
        "Server.Engines.PartySystem.Party.Configure",
        party::configure,
    );
    types.register_phase_default(
        muo_server::type_registry::Phase::Initialize,
        "Server.Engines.PartySystem.Party.Initialize",
        party::initialize,
    );
    types.register_phase_default(
        muo_server::type_registry::Phase::Configure,
        "FeatureFlagCommands.Configure",
        feature_flags::configure,
    );
    types.register_phase_default(
        muo_server::type_registry::Phase::Initialize,
        "FeatureFlagManager.Initialize",
        feature_flags::initialize,
    );
    types.register_phase_default(
        muo_server::type_registry::Phase::Initialize,
        "Server.Misc.Cleanup.Initialize",
        cleanup::initialize,
    );
    types.register_phase_default(
        muo_server::type_registry::Phase::Configure,
        "EventScheduler.Configure",
        events::configure,
    );
    types.register_phase_default(
        muo_server::type_registry::Phase::Configure,
        "PageQueue.Configure",
        help::configure,
    );
    types.register_phase_default(
        muo_server::type_registry::Phase::Configure,
        "ChatSystem.Configure",
        chat::configure,
    );
    types.register_phase_default(
        muo_server::type_registry::Phase::Configure,
        "Server.HardwareInfo.Configure",
        hardware_info::configure,
    );
    types.register_phase_default(
        muo_server::type_registry::Phase::Initialize,
        "Channel.Initialize",
        chat::initialize,
    );
    types.register_phase_default(
        muo_server::type_registry::Phase::Configure,
        "Server.Items.Special.Wiring",
        special_wiring::configure,
    );
    types.register_phase_default(
        muo_server::type_registry::Phase::Configure,
        "VirtueSystem.Configure",
        virtues::configure,
    );
    types.register_phase_default(
        muo_server::type_registry::Phase::Configure,
        "EnginesGlue.Configure",
        glue::configure,
    );
    types.register_phase_default(
        muo_server::type_registry::Phase::Configure,
        "EnginesWiring2.Configure",
        wiring2::configure,
    );
    types.register_phase_default(
        muo_server::type_registry::Phase::Initialize,
        "VirtueTimer.Initialize",
        virtues::initialize,
    );
    types.register_phase(
        muo_server::type_registry::Phase::Initialize,
        "JusticeVirtue.OnPlayerDeleted",
        49,
        virtues::subscribe_justice_deleted,
    );
    types.register_phase_default(
        muo_server::type_registry::Phase::Configure,
        "PlayerMurderSystem.Configure",
        murder::configure,
    );
    types.register_phase_default(
        muo_server::type_registry::Phase::Initialize,
        "PlayerMurderSystem.Initialize",
        murder::initialize,
    );
    types.register_phase_default(
        muo_server::type_registry::Phase::Configure,
        "Insurance.Configure",
        insurance::configure,
    );
    types.register_phase_default(
        muo_server::type_registry::Phase::Configure,
        "StealableArtifacts.Configure",
        stealables::configure,
    );
    faction_items::register(types);
    ethics::mobiles::register(types);
    ethics::legacy::register(types);
    factions::legacy::register(types);
    {
        use muo_server::type_registry::{DEFAULT_CALL_PRIORITY, Phase};
        types.register_phase(
            Phase::Configure,
            "FactionSystem.Configure",
            DEFAULT_CALL_PRIORITY,
            factions::configure,
        );
        types.register_phase(
            Phase::Configure,
            "EthicsSystem.Configure",
            DEFAULT_CALL_PRIORITY,
            ethics::configure,
        );
        types.register_phase(
            Phase::Configure,
            "DuelContext.Configure",
            DEFAULT_CALL_PRIORITY,
            conpvp::configure,
        );
        types.register_phase(
            Phase::Initialize,
            "Faction.Initialize",
            DEFAULT_CALL_PRIORITY,
            factions::initialize,
        );
        types.register_phase(
            Phase::Initialize,
            "Ethic.Initialize",
            DEFAULT_CALL_PRIORITY,
            ethics::initialize,
        );
    }
}
