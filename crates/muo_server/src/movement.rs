//! Movement rules (`Server.Movement.Movement` and `IMovementImpl`).

use muo_primitives::{Direction, Point3D};

use crate::ids::MobileId;
use crate::map::MapId;
use crate::world::World;

/// `IMovementImpl`.
pub trait MovementImpl: Sync {
    /// `CheckMovement(Mobile, Direction, out newZ)`.
    fn check_movement(&self, w: &mut World, m: MobileId, d: Direction) -> (bool, i32);
    /// `CheckMovement(Mobile, Map, Point3D, Direction, out newZ)`.
    fn check_movement_at(
        &self,
        w: &mut World,
        m: MobileId,
        map: MapId,
        loc: Point3D,
        d: Direction,
    ) -> (bool, i32);
}

/// The statics of `Movement`, held in the world's resources. All zero until [`configure`] runs.
#[derive(Default)]
pub struct Movement {
    pub imp: Option<&'static dyn MovementImpl>,
    pub walk_foot_delay: i32,
    pub run_foot_delay: i32,
    pub walk_mount_delay: i32,
    pub run_mount_delay: i32,
    pub turn_delay: i32,
}

/// `Movement.Configure`: the delays are `ServerConfiguration.GetOrUpdateSetting(key, default)`, written on first boot.
pub fn configure(w: &mut World) {
    let c = &mut w.config;
    let mut get =
        |key: &str, default: i32| c.get_or_update_setting(key, default).unwrap_or(default);
    let (turn, walk_foot, run_foot, walk_mount, run_mount) = (
        get("movement.delay.turn", 0),
        get("movement.delay.walkFoot", 400),
        get("movement.delay.runFoot", 200),
        get("movement.delay.walkMount", 200),
        get("movement.delay.runMount", 100),
    );
    let m = w.resources.get_mut::<Movement>();
    m.turn_delay = turn;
    m.walk_foot_delay = walk_foot;
    m.run_foot_delay = run_foot;
    m.walk_mount_delay = walk_mount;
    m.run_mount_delay = run_mount;
}

/// `Movement.CheckMovement(m, d, out newZ)`: false with the mobile's own Z without an implementation.
pub fn check_movement(w: &mut World, m: MobileId, d: Direction) -> (bool, i32) {
    match w.resources.get::<Movement>().and_then(|mv| mv.imp) {
        Some(imp) => imp.check_movement(w, m, d),
        None => (false, w.mobile(m).map_or(0, |mm| mm.location.z)),
    }
}

/// `Movement.CheckMovement(m, map, loc, d, out newZ)`.
pub fn check_movement_at(
    w: &mut World,
    m: MobileId,
    map: MapId,
    loc: Point3D,
    d: Direction,
) -> (bool, i32) {
    match w.resources.get::<Movement>().and_then(|mv| mv.imp) {
        Some(imp) => imp.check_movement_at(w, m, map, loc, d),
        None => (false, w.mobile(m).map_or(0, |mm| mm.location.z)),
    }
}

/// `Movement.Offset(d, ref x, ref y, count)`.
pub fn offset_xy(d: Direction, x: &mut i32, y: &mut i32, count: i32) {
    match d & Direction::MASK {
        Direction::NORTH => *y -= count,
        Direction::SOUTH => *y += count,
        Direction::WEST => *x -= count,
        Direction::EAST => *x += count,
        Direction::RIGHT => {
            *x += count;
            *y -= count;
        }
        Direction::LEFT => {
            *x -= count;
            *y += count;
        }
        Direction::DOWN => {
            *x += count;
            *y += count;
        }
        Direction::UP => {
            *x -= count;
            *y -= count;
        }
        _ => {}
    }
}

/// `Movement.Offset(d, ref Point3D, count)`.
pub fn offset(d: Direction, p: &mut Point3D, count: i32) {
    let (mut x, mut y) = (p.x, p.y);
    offset_xy(d, &mut x, &mut y, count);
    *p = Point3D::new(x, y, p.z);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn offsets() {
        let mut p = Point3D::new(10, 10, 5);
        offset(Direction::NORTH, &mut p, 1);
        assert_eq!(p, Point3D::new(10, 9, 5));
        offset(Direction::DOWN | Direction::RUNNING, &mut p, 2);
        assert_eq!(p, Point3D::new(12, 11, 5));
        offset(Direction::LEFT, &mut p, 1);
        assert_eq!(p, Point3D::new(11, 12, 5));
    }

    #[test]
    fn without_impl_fails_with_own_z() {
        let mut w = World::new();
        let id = w.new_mobile(&crate::mobile::MOBILE_BASE);
        w.mobile_mut(id).unwrap().location = Point3D::new(1, 2, 7);
        assert_eq!(check_movement(&mut w, id, Direction::NORTH), (false, 7));
    }
}
