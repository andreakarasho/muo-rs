//! Server.Spells.Spellweaving
pub mod arcanist;
pub mod forms;
pub mod items;
pub mod mobiles;
pub mod spells;
pub mod summons;

/// The setup of the Spellweaving hooks and event handlers.
pub fn init(w: &mut muo_server::World) {
    forms::init(w);
    spells::init(w);
    mobiles::init(w);
}
