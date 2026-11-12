//! The gump system. See `base` for how a gump class is written.

pub mod base;
pub mod base_grid;
#[cfg(test)]
mod behavior_tests;
pub mod builder;
pub mod grid;
pub mod gump_types;
pub mod layout;
pub mod legacy;
pub mod props_config;
pub mod props_layout;
pub mod system;

pub use base::{
    DynamicGumpLogic, DynamicGumpObj, Gump, LegacyGumpLogic, LegacyGumpObj, RelayInfo,
    StaticGumpLogic, StaticGumpObj,
};
pub use builder::{DynamicGumpBuilder, GumpBuilder, GumpStrings, HtmlStyle, StaticGumpBuilder};
pub use layout::{GumpButtonType, GumpFlags};
pub use system::{close_gump, find_gump, has_gump, send_gump, send_gump_to_mobile};
