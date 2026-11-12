//! The ML quest system. See [`engine`] for the system, [`model`] for the data classes.

pub mod commands;
pub mod defs;
pub mod engine;
pub mod gumps;
pub mod item_support;
pub mod items;
pub mod items2;
pub mod items_gen;
pub mod model;
pub(crate) mod npc_support;
pub mod npcs;
mod npcs_escort;
mod npcs_gen;
mod npcs_more;
pub mod persistence;
pub mod quester;
pub mod quester_names;
pub mod race_change;
mod setup;
pub mod transient;

pub use setup::register;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod tests2;
