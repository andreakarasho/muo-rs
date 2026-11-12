//! Type attributes.
//!
//! `[Constructible]`, `[CommandProperty]` and arbitrary attributes are not looked up through reflection: a class's
//! constructibility is `ItemType::construct`/`MobileType::construct` (the `[Constructible]` constructor, registered
//! by the owning crate), and the access level it needs travels with that registration.
//! `GetCPA(PropertyInfo)` is the `read`/`write` access levels of a `props::PropDef`; `HasAttribute(Type, ...)` (`[PropertyObject]`,
//! `[NoSort]`) is the props gump's `PropKind` in `muo_gumps::m_z::props`.

use muo_server::mobile::AccessLevel;

/// `Attributes.IsConstructible(ctor, accessLevel)`: the constructor has `[Constructible]` and the caller's access
/// level reaches the attribute's. `None` is a constructor without the attribute.
pub fn is_constructible(required: Option<AccessLevel>, access_level: AccessLevel) -> bool {
    required.is_some_and(|r| access_level >= r)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constructible_needs_the_attribute_and_enough_access() {
        assert!(!is_constructible(None, AccessLevel::Owner));
        assert!(is_constructible(
            Some(AccessLevel::GameMaster),
            AccessLevel::Administrator
        ));
        assert!(!is_constructible(
            Some(AccessLevel::GameMaster),
            AccessLevel::Counselor
        ));
    }

    /// `[Constructible]` is `[Constructible(AccessLevel.Player)]`: `Ctor.access: None` reaches every caller.
    #[test]
    fn plain_constructible_is_player_level() {
        let plain = Option::<AccessLevel>::default();
        assert!(is_constructible(
            Some(plain.unwrap_or(AccessLevel::Player)),
            AccessLevel::Player
        ));
    }
}
