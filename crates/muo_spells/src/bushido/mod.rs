//! Server.Spells.Bushido
pub mod confidence;
pub mod counter_attack;
pub mod evasion;
pub mod moves;
pub mod samurai;

/// The setup of the Bushido hooks.
pub fn init(w: &mut muo_server::World) {
    confidence::init(w);
    counter_attack::init(w);
    evasion::init(w);
    moves::init(w);
}
