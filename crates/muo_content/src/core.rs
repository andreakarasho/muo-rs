//! `Core.AOS`, `Core.ML` and the other expansion switches, read from the world's `Core.Expansion`.

use muo_data::expansion::Expansion;
use muo_server::World;

/// `Core.Expansion`: `World::expansion` (`w.core.expansion`, ML before the configuration prompts ran).
pub fn expansion(w: &World) -> Expansion {
    w.expansion()
}

pub fn t2a(w: &World) -> bool {
    expansion(w) >= Expansion::T2A
}

pub fn uor(w: &World) -> bool {
    expansion(w) >= Expansion::UOR
}

pub fn lbr(w: &World) -> bool {
    expansion(w) >= Expansion::LBR
}

pub fn aos(w: &World) -> bool {
    expansion(w) >= Expansion::AOS
}

pub fn se(w: &World) -> bool {
    expansion(w) >= Expansion::SE
}

pub fn ml(w: &World) -> bool {
    expansion(w) >= Expansion::ML
}

pub fn sa(w: &World) -> bool {
    expansion(w) >= Expansion::SA
}

pub fn hs(w: &World) -> bool {
    expansion(w) >= Expansion::HS
}

pub fn tol(w: &World) -> bool {
    expansion(w) >= Expansion::TOL
}
