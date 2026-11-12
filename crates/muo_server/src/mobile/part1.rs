//! Non-virtual members of `Mobile`, first half (the virtuals are in `base`). Free functions over
//! `&mut World` and a `MobileId`, named in snake_case from the member: a property with logic is `name(w, id)` and
//! `set_name(w, id, value)`; an overload gets a suffix naming what tells it apart.
//!
//! - `props_a`: DamageEntry, statics, constructor-time state, properties up to Karma
//! - `props_b`: Luck .. PropertyList
//! - `lifecycle`: Serialize, Delete, Map, Location, ProcessDelta, resistances, name properties
//! - `combat`: aggressor expiry, skill mods, actions, aggression, totals, targets, Move

pub mod combat;
pub mod lifecycle;
pub mod props_a;
pub mod props_b;
