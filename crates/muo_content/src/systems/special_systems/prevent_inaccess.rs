//! `PreventInaccess`.
//!
//! This system prevents the inability for server staff to access their server due to data overflows during login.
//! Whenever a staff character's NetState is disposed right after the login process, the character is moved to and
//! logged out at a "safe" alternative. The location the character was moved from will be reported to the player
//! upon the next successful login. This system does not affect non-staff players.

use indexmap::IndexMap;
use muo_primitives::{Point3D, utility};
use muo_server::World;
use muo_server::ids::MobileId;
use muo_server::map::MapId;
use muo_server::mobile::AccessLevel;
use muo_server::mobile::part2::{deps, messages as mm};

use crate::network::events::{self, PlayerLogin};

pub const ENABLED: bool = true;

const DESTINATIONS: [(Point3D, MapId); 4] = [
    (Point3D::new(5275, 1163, 0), MapId::FELUCCA), // Jail
    (Point3D::new(5275, 1163, 0), MapId::TRAMMEL),
    (Point3D::new(5445, 1153, 0), MapId::FELUCCA), // Green acres
    (Point3D::new(5445, 1153, 0), MapId::TRAMMEL),
];

/// `m_MoveHistory`.
#[derive(Default)]
pub struct MoveHistory(IndexMap<MobileId, (Point3D, Option<MapId>)>);

/// `PreventInaccess.Initialize()`.
pub fn initialize(w: &mut World) {
    *w.resources.get_mut::<MoveHistory>() = MoveHistory::default();
    events::subscribe_ranked::<PlayerLogin>(w, 22, |w, e| on_login(w, e.0));
}

fn has_disconnected(w: &World, m: MobileId) -> bool {
    w.mobile(m)
        .and_then(|x| x.net_state)
        .is_none_or(|ns| w.cannot_send_packets(ns))
}

/// `OnLogin(from)`.
pub fn on_login(w: &mut World, from: MobileId) {
    if !ENABLED {
        return;
    }

    let Some(level) = w.mobile(from).map(|m| m.access_level) else {
        return;
    };
    if level < AccessLevel::Counselor {
        return;
    }

    if has_disconnected(w, from) {
        let (loc, map) = w
            .mobile(from)
            .map(|m| (m.location, m.map))
            .unwrap_or_default();
        w.resources
            .get_mut::<MoveHistory>()
            .0
            .entry(from)
            .or_insert((loc, map));

        let (dest, dest_map) =
            *utility::random_element(&mut w.rng, &DESTINATIONS).expect("not empty");

        muo_server::mobile::base::set_location(w, from, dest, false);
        deps::set_map(w, from, Some(dest_map));
    } else if let Some((loc, map)) = w.resources.get_mut::<MoveHistory>().0.shift_remove(&from) {
        let map_name = map
            .and_then(|m| w.maps.get(m))
            .map_or(String::new(), |m| m.name().to_string());
        mm::send_message(
            w,
            from,
            &format!(
                "Your character was moved from {loc} ({map_name}) due to a detected client crash."
            ),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use muo_server::mobile::MOBILE_BASE;

    #[test]
    fn staff_with_a_dead_connection_are_moved_and_remembered() {
        let mut w = muo_server::map::test_support::world();
        let m = w.new_mobile(&MOBILE_BASE);
        {
            let mob = w.mobile_mut(m).unwrap();
            mob.access_level = AccessLevel::GameMaster;
            mob.location = Point3D::new(10, 10, 0);
            mob.map = Some(MapId::FELUCCA);
        }
        on_login(&mut w, m);
        let mob = w.mobile(m).unwrap();
        assert!(
            DESTINATIONS.iter().any(|&(p, _)| p == mob.location),
            "moved to a safe place: {:?}",
            mob.location
        );
        assert_eq!(
            w.resources.get_mut::<MoveHistory>().0.get(&m).map(|e| e.0),
            Some(Point3D::new(10, 10, 0))
        );
        // A second failed login keeps the first origin.
        on_login(&mut w, m);
        assert_eq!(
            w.resources.get_mut::<MoveHistory>().0[&m].0,
            Point3D::new(10, 10, 0)
        );
    }

    #[test]
    fn players_are_left_alone() {
        let mut w = muo_server::map::test_support::world();
        let m = w.new_mobile(&MOBILE_BASE);
        w.mobile_mut(m).unwrap().location = Point3D::new(10, 10, 0);
        on_login(&mut w, m);
        assert_eq!(w.mobile(m).unwrap().location, Point3D::new(10, 10, 0));
        assert!(w.resources.get_mut::<MoveHistory>().0.is_empty());
    }
}
