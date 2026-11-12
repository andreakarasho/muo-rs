//! The ML quest-giver mobiles. `npcs_gen.rs` holds the ones on `BaseCreature`; the classes on other bases
//! (vendors, escortables, mages...) are in `npcs_more` and `npcs_escort`.

use muo_server::ids::MobileId;
use muo_server::type_registry::TypeRegistry;
use muo_server::world::World;

use super::npcs_gen;

/// `BaseCreature.Shout(pm)` of the creature's class, if it overrides it.
pub fn shout_for(w: &World, creature: MobileId) -> Option<fn(&mut World, MobileId, MobileId)> {
    let name = w.mobile(creature)?.ty.full_name;
    npcs_gen::SHOUTS
        .iter()
        .chain(super::npcs_more::SHOUTS.iter())
        .chain(super::npcs_escort::SHOUTS.iter())
        .find(|(n, _)| *n == name)
        .map(|(_, f)| *f)
}

/// Registers the mobile classes.
pub fn register(types: &mut TypeRegistry) {
    npcs_gen::register(types);
    super::npcs_more::register(types);
    super::npcs_escort::register(types);
}
