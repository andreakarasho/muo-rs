//! Special NPCs: champions, the Harrower, paragons and shield guards.

pub mod champions;
pub mod harrower;
pub mod hooks;
pub mod minor;
pub mod paragon;
pub mod shield_guards;

pub(crate) fn register(types: &mut muo_server::type_registry::TypeRegistry) {
    use muo_server::type_registry::{DEFAULT_CALL_PRIORITY, Phase};
    types.register_phase(
        Phase::Initialize,
        "Paragon.Initialize",
        DEFAULT_CALL_PRIORITY,
        paragon::initialize,
    );
    champions::register(types);
    harrower::register(types);
    minor::register(types);
    shield_guards::register(types);
}

#[cfg(test)]
pub(crate) fn all() -> Vec<&'static muo_server::mobile::MobileType> {
    let mut v = champions::all();
    v.extend(harrower::all());
    v.extend(minor::all());
    v.extend(shield_guards::all());
    v
}
