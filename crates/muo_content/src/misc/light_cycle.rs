//! `LightCycle`: the global and per-player light levels over the UO day.

use std::rc::Rc;

use muo_primitives::{DateTime, TimeSpan};
use muo_server::World;
use muo_server::commands::{CommandEventArgs, CommandSystem};
use muo_server::ids::MobileId;
use muo_server::mobile::AccessLevel;
use muo_server::mobile::part2::messages;
use muo_server::timer;

pub const DAY_LEVEL: i32 = 0;
pub const NIGHT_LEVEL: i32 = 12;
pub const DUNGEON_LEVEL: i32 = 26;
pub const JAIL_LEVEL: i32 = 9;

/// `LightCycle.LevelOverride` (`int.MinValue` is none).
struct LevelOverride(i32);

impl Default for LevelOverride {
    fn default() -> Self {
        LevelOverride(i32::MIN)
    }
}

pub fn level_override(w: &World) -> i32 {
    w.resources.get::<LevelOverride>().map_or(i32::MIN, |l| l.0)
}

/// `NetState.Instances`' mobiles, in connection order.
fn connected_mobiles(w: &World) -> Vec<MobileId> {
    w.net_states.values().filter_map(|d| d.mobile).collect()
}

/// `LightCycle.LevelOverride = value`: every connected mobile re-checks its light.
pub fn set_level_override(w: &mut World, value: i32) {
    w.resources.get_mut::<LevelOverride>().0 = value;
    for m in connected_mobiles(w) {
        if let Some(ty) = w.mobile(m).map(|x| x.ty) {
            (ty.check_light_levels)(w, m, false);
        }
    }
}

/// `Clock.GetTime(map, x, y, out hours, out minutes)`: the UO time of day,
/// five real seconds per UO minute counted from 1997-09-01, a facet and a longitude offset.
pub fn clock_time(w: &World, map_index: Option<i32>, x: i32) -> (i32, i32) {
    let world_start = DateTime::from_ymd(1997, 9, 1).unwrap_or(DateTime::MIN);
    let span = w.now - world_start;
    let mut total_minutes = (span.total_seconds() / 5.0) as i32;
    if let Some(i) = map_index {
        total_minutes = total_minutes.wrapping_add(i * 320);
    }
    // Really on OSI this must be by subserver
    total_minutes = total_minutes.wrapping_add(x / 16);
    ((total_minutes / 60) % 24, total_minutes % 60)
}

/// `LightCycle.ComputeLevelFor(from)`.
pub fn compute_level_for(w: &World, from: MobileId) -> i32 {
    let o = level_override(w);
    if o > i32::MIN {
        return o;
    }
    let Some(m) = w.mobile(from) else {
        return NIGHT_LEVEL;
    };
    let map_index = m.map.and_then(|id| w.maps.get(id)).map(|map| map.map_index);
    let (hours, minutes) = clock_time(w, map_index, m.location.x);

    /* OSI times:
     *
     * Midnight ->  3:59 AM : Night
     *  4:00 AM -> 11:59 PM : Day
     *
     * RunUO times:
     *
     * 10:00 PM -> 11:59 PM : Scale to night
     * Midnight ->  3:59 AM : Night
     *  4:00 AM ->  5:59 AM : Scale to day
     *  6:00 AM ->  9:59 PM : Day
     */
    match hours {
        i32::MIN..=3 => NIGHT_LEVEL,
        4..=5 => NIGHT_LEVEL + ((hours - 4) * 60 + minutes) * (DAY_LEVEL - NIGHT_LEVEL) / 120,
        6..=21 => DAY_LEVEL,
        22..=23 => DAY_LEVEL + ((hours - 22) * 60 + minutes) * (NIGHT_LEVEL - DAY_LEVEL) / 120,
        _ => NIGHT_LEVEL,
    }
}

/// `LightCycle.Configure`: `[GlobalLight <value>]`.
pub fn configure(w: &mut World) {
    CommandSystem::register(
        w,
        "GlobalLight",
        AccessLevel::GameMaster,
        &[],
        Rc::new(|w: &mut World, e: &mut CommandEventArgs| {
            // Usage: GlobalLight <value>; Description: Sets the current global light level.
            if e.length() >= 1 {
                set_level_override(w, e.get_int32(0));
                let text = format!(
                    "Global light level override has been changed to {}.",
                    level_override(w)
                );
                messages::send_message(w, e.mobile, &text);
            } else {
                set_level_override(w, i32::MIN);
                messages::send_message(
                    w,
                    e.mobile,
                    "Global light level override has been cleared.",
                );
            }
        }),
    );
}

/// `LightCycle.Initialize`: the timer that rechecks every connected mobile's light every five seconds.
pub fn initialize(w: &mut World) {
    crate::network::events::subscribe_ranked::<crate::network::events::PlayerLogin>(
        w,
        12,
        |w, e| on_login(w, e.0),
    );
    timer::start(w, TimeSpan::ZERO, TimeSpan::from_secs(5), 0, |w| {
        for m in connected_mobiles(w) {
            if let Some(ty) = w.mobile(m).map(|x| x.ty) {
                (ty.check_light_levels)(w, m, false);
            }
        }
    });
}

/// `LightCycle.OnLogin` (`[OnEvent(PlayerLoginEvent)]`): `pm.CheckLightLevels(true)`.
pub fn on_login(w: &mut World, pm: MobileId) {
    if let Some(ty) = w.mobile(pm).map(|x| x.ty) {
        (ty.check_light_levels)(w, pm, true);
    }
}

/// `LightCycle.NightSightTimer`: after `Random(15, 25)` minutes the owner's night sight ends.
pub fn start_night_sight_timer(w: &mut World, owner: MobileId) -> timer::TimerHandle {
    let minutes = muo_primitives::utility::random_from(&mut w.rng, 15, 25);
    timer::start(
        w,
        TimeSpan::from_mins(i64::from(minutes)),
        TimeSpan::ZERO,
        1,
        move |w| {
            muo_server::mobile::part1::combat::end_action(w, owner, "Server.LightCycle");
            muo_server::mobile::part1::props_a::set_light_level(w, owner, 0);
            crate::player_mobile::buff::remove_buff(
                w,
                owner,
                crate::player_mobile::buff::BuffIcon::NightSight,
            );
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn override_wins_and_dusk_scales() {
        let mut w = World::new();
        let m = w.new_mobile(&muo_server::mobile::MOBILE_BASE);
        set_level_override(&mut w, 7);
        assert_eq!(compute_level_for(&w, m), 7);
        set_level_override(&mut w, i32::MIN);
        let level = compute_level_for(&w, m);
        assert!((DAY_LEVEL..=NIGHT_LEVEL).contains(&level));
    }
}
