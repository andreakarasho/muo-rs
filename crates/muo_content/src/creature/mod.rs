//! BaseCreature: the `CreatureType` family vtable (every `virtual` of `BaseCreature` that
//! content classes override), the base class's `MobileType` and the free functions for its non-virtual members.
//!
//! A leaf class is `static ORC_C: CreatureType = CreatureType { .. , ..BASE_CREATURE_C }` plus
//! `static ORC: MobileType = MobileType { family: Some(&ORC_C), parent: Some(&BASE_CREATURE), .. }`, and its
//! constructor calls [`setters::new_creature`] before setting the stats.

pub mod control;
pub mod data;
pub mod death;
pub mod enums;
pub mod events;
pub mod init_ops;
pub mod misc;
pub mod mount;
pub mod overrides;
pub mod packing;
pub mod pets;
pub mod serialize;
pub mod setters;
pub mod solen;
pub mod speeds;
pub mod teach;
#[cfg(test)]
mod tests;
pub mod virtual_mount;
pub mod vtable;

pub use data::{CreatureData, cd, cd_mut, edit, is_creature, read};
pub use vtable::{BASE_CREATURE, BASE_CREATURE_C, CreatureType, ctype};

use muo_server::ids::MobileId;
use muo_server::world::World;

/// `GetType().Name` of the creature.
pub fn type_name(w: &World, id: MobileId) -> &'static str {
    w.mobile(id)
        .map_or("", |m| type_name_of_full(m.ty.full_name))
}

fn type_name_of_full(n: &'static str) -> &'static str {
    n.rsplit('.').next().unwrap_or(n)
}

/// Registers the types and phases of the creature code: `NPCSpeeds.Configure`, `BaseCreature.Configure`,
/// `LoyaltyTimer.Initialize`, the creature event subscriptions and `VirtualMountItem`.
pub fn register(types: &mut muo_server::type_registry::TypeRegistry) {
    use muo_server::type_registry::{DEFAULT_CALL_PRIORITY, Phase};
    types.register_item(&virtual_mount::VIRTUAL_MOUNT_ITEM, &[]);
    types.register_item(&mount::MOUNT_ITEM, &[]);
    types.register_phase(
        Phase::Configure,
        "NPCSpeeds.Configure",
        DEFAULT_CALL_PRIORITY,
        speeds::configure,
    );
    types.register_phase(
        Phase::Configure,
        "BaseCreature.Configure",
        DEFAULT_CALL_PRIORITY,
        misc::configure,
    );
    types.register_phase(
        Phase::Initialize,
        "LoyaltyTimer.Initialize",
        DEFAULT_CALL_PRIORITY,
        control::initialize_loyalty_timer,
    );
    types.register_phase(
        Phase::Initialize,
        "MonsterAbility.Subscribe",
        DEFAULT_CALL_PRIORITY,
        |w| {
            events::subscribe_death(w, crate::abilities::reset_cooldowns_on_death);
        },
    );
}
