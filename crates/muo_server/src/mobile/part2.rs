//! Non-virtual members of `Mobile`, second half, plus its messages, enumerators and
//! save migrations (the virtuals are in `base`). Free functions over `&mut World` and a `MobileId`, named in
//! snake_case from the member, overloads with a suffix naming what tells them apart.
//!
//! - `statics`: the `Mobile` statics of this half plus the ones only it reads
//! - `deps`: everything this half calls that another module owns, one name each
//! - `messages`: overhead and system messages
//! - `items`: AddItem/RemoveItem, FindItemOnLayer, backpack and bank, DropHolding, LiftItemDupe
//! - `speech`: damage-free speech helpers (prompts, SayTo) and `GetDirectionTo`
//! - `damage`: damage entries, visible damage, Heal
//! - `stats`: stat mods, InitStats, DefaultMobileInit, regen timers, poison
//! - `effects`: sounds, animations, packets to nearby clients, MovingEffect.. wrappers
//! - `serialization`: Serialize / Deserialize
//! - `migrations`, `enumerators`: the two small partial-class files

pub mod damage;
pub mod deps;
pub mod effects;
pub mod enumerators;
pub mod items;
pub mod messages;
pub mod migrations;
pub mod properties;
pub mod serialization;
pub mod speech;
pub mod statics;
pub mod stats;
