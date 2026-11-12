//! Notoriety: the name colors and the rules that pick them.

use crate::ids::MobileId;
use crate::world::World;

pub const INNOCENT: i32 = 1;
pub const ALLY: i32 = 2;
pub const CAN_BE_ATTACKED: i32 = 3;
pub const CRIMINAL: i32 = 4;
pub const ENEMY: i32 = 5;
pub const MURDERER: i32 = 6;
pub const INVULNERABLE: i32 = 7;

/// `Notoriety.Handler` and `Notoriety.Hues`, held in the world's resources.
pub struct Notoriety {
    /// `NotorietyHandler(source, target)`.
    pub handler: Option<fn(&mut World, MobileId, MobileId) -> i32>,
    pub hues: Vec<i32>,
}

impl Default for Notoriety {
    fn default() -> Self {
        Notoriety {
            handler: None,
            hues: vec![0x000, 0x059, 0x03F, 0x3B2, 0x3B2, 0x090, 0x022, 0x035],
        }
    }
}

/// `Notoriety.GetHue`.
pub fn get_hue(w: &World, noto: i32) -> i32 {
    let default = Notoriety::default();
    let hues = w
        .resources
        .get::<Notoriety>()
        .map_or(&default.hues, |n| &n.hues);
    usize::try_from(noto)
        .ok()
        .and_then(|i| hues.get(i))
        .copied()
        .unwrap_or(0)
}

/// `Notoriety.Compute`: the handler's answer, `CanBeAttacked` without one.
pub fn compute(w: &mut World, source: MobileId, target: MobileId) -> i32 {
    let handler = w.resources.get::<Notoriety>().and_then(|n| n.handler);
    handler.map_or(CAN_BE_ATTACKED, |h| h(w, source, target))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hues_and_default() {
        let mut w = World::new();
        assert_eq!(get_hue(&w, 6), 0x22);
        assert_eq!(get_hue(&w, 8), 0);
        assert_eq!(get_hue(&w, -1), 0);
        let a = MobileId(muo_primitives::Serial::new(1));
        assert_eq!(compute(&mut w, a, a), CAN_BE_ATTACKED);
    }
}
