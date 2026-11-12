//! `Name = m_Names.RandomElement();` of the ophidians.

use muo_primitives::utility;
use muo_server::World;
use muo_server::ids::MobileId;

fn random_name(w: &mut World, id: MobileId, names: &[&str]) {
    let n = utility::random_element(&mut w.rng, names).map(|s| (*s).to_owned());
    crate::common::name(w, id, n);
}

pub fn ophidian_archmage_ctor1(w: &mut World, id: MobileId) {
    random_name(w, id, &["an ophidian justicar", "an ophidian zealot"]);
}
pub fn ophidian_knight_ctor1(w: &mut World, id: MobileId) {
    random_name(w, id, &["an ophidian knight-errant", "an ophidian avenger"]);
}
pub fn ophidian_mage_ctor1(w: &mut World, id: MobileId) {
    random_name(
        w,
        id,
        &["an ophidian apprentice mage", "an ophidian shaman"],
    );
}
pub fn ophidian_warrior_ctor1(w: &mut World, id: MobileId) {
    random_name(w, id, &["an ophidian warrior", "an ophidian enforcer"]);
}
