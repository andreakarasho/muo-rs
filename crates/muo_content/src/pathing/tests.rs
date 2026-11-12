//! Tests for MovementImpl, the A* search and PathFollower on the invented tile kit of `map::test_support`.

use muo_primitives::Direction;
use muo_server::map::MapId;
use muo_server::map::test_support::{WALL, add_static, p, place_mobile, world};
use muo_server::movement::{MovementImpl, offset};

use super::astar::find_path;
use super::movement_impl::IMPL;

#[test]
fn walking_on_open_ground_and_into_a_wall() {
    let mut w = world();
    let m = place_mobile(&mut w, MapId::FELUCCA, p(100, 100, 0));
    assert_eq!(IMPL.check_movement(&mut w, m, Direction::NORTH), (true, 0));
    add_static(&mut w, MapId::FELUCCA, 100, 99, WALL, 0);
    let (ok, z) = IMPL.check_movement(&mut w, m, Direction::NORTH);
    assert!(!ok);
    assert_eq!(z, 0, "a refused step reports the start Z");
    assert!(IMPL.check_movement(&mut w, m, Direction::EAST).0);
}

#[test]
fn a_path_goes_around_a_wall() {
    let mut w = world();
    let m = place_mobile(&mut w, MapId::FELUCCA, p(100, 100, 0));
    for x in 96..=104 {
        add_static(&mut w, MapId::FELUCCA, x, 98, WALL, 0);
    }
    let dirs = find_path(&mut w, m, p(100, 95, 0));
    assert!(!dirs.is_empty(), "a detour exists");
    let mut at = p(100, 100, 0);
    for d in &dirs {
        offset(*d, &mut at, 1);
        assert!(
            !(at.y == 98 && (96..=104).contains(&at.x)),
            "the route never enters the wall"
        );
    }
    assert_eq!((at.x, at.y), (100, 95));
    assert!(dirs.len() >= 9, "around the wall, not through it");
}

#[test]
fn no_route_when_the_goal_is_beyond_the_search_window() {
    let mut w = world();
    let m = place_mobile(&mut w, MapId::FELUCCA, p(100, 100, 0));
    assert!(find_path(&mut w, m, p(200, 100, 0)).is_empty());
}
