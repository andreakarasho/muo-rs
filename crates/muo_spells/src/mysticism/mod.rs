//! Server.Spells.Mysticism
pub mod mystic;
pub mod spell_plague;
pub mod spells;

/// The setup of the Mysticism hooks and event handlers.
pub fn init(w: &mut muo_server::World) {
    spells::init(w);
    spell_plague::init(w);
}
