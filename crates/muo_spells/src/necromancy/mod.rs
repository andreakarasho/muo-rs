//! Server.Spells.Necromancy
pub mod animate_dead;
pub mod blood_oath;
pub mod corpse_skin;
pub mod curse_weapon;
pub mod evil_omen;
pub mod exorcism;
pub mod forms;
pub mod mind_rot;
pub mod necromancer;
pub mod pain_spike;
pub mod poison_strike;
pub mod strangle;
pub mod summon_familiar;
pub mod vengeful_spirit;
pub mod wither;

/// The setup of the Necromancy spells' hooks and event handlers.
pub fn init(w: &mut muo_server::World) {
    animate_dead::init(w);
    blood_oath::init(w);
    evil_omen::init(w);
    forms::init(w);
    mind_rot::init(w);
    summon_familiar::init(w);
}
