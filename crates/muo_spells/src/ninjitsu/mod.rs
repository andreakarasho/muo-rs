//! Server.Spells.Ninjitsu
pub mod animal_form;
pub mod clone;
pub mod mirror_image;
pub mod moves;
pub mod ninja;
pub mod shadow_jump;

/// The setup of the Ninjitsu hooks and event handlers.
pub fn init(w: &mut muo_server::World) {
    animal_form::init(w);
    mirror_image::init(w);
    clone::init(w);
    moves::init(w);
}
