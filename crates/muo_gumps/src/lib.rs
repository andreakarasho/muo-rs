//! The gumps: staff tools, houses, guilds, vendors and the other dialogs, `a_l` and `m_z` by name.

pub mod a_l;
pub mod m_z;

/// Registers this crate's types and phases at startup.
pub fn register(types: &mut muo_server::type_registry::TypeRegistry) {
    a_l::register(types);
    m_z::register(types);
}
