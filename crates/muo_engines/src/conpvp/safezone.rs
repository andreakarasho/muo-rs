//! `SafeZone`: the guarded region around a dueling arena, and the questions about it that
//! `Region.IsPartOf<SafeZone>()` asks elsewhere (`mobile_in_safe_zone`).

use muo_primitives::{Direction, Point3D, Rectangle2D};
use muo_server::World;
use muo_server::ids::MobileId;
use muo_server::map::MapId;
use muo_server::mobile::AccessLevel;
use muo_server::mobile::part2::messages as mm;
use muo_server::region::{self, RegionId, RegionType};

use muo_content::regions::base_region::{self, RegionHooks};
use muo_content::regions::guarded::GUARDED_REGION;
use muo_content::regions::house::HOUSE_PRIORITY;

use super::{duel_context_of, flow};

/// `SafeZonePriority`.
pub const SAFE_ZONE_PRIORITY: i32 = HOUSE_PRIORITY + 1;

/// `SafeZone.OnMoveInto`.
fn on_move_into(
    w: &mut World,
    id: RegionId,
    m: MobileId,
    d: Direction,
    new_location: Point3D,
    old_location: Point3D,
) -> bool {
    if w.mobile(m).is_some_and(|x| x.player)
        && muo_content::spells::hooks::hooks(w)
            .and_then(|h| h.sigil_exists_on)
            .is_some_and(|f| f(w, m))
    {
        mm::send_message_hue(
            w,
            m,
            0x22,
            "You are holding a sigil and cannot enter this zone.",
        );
        return false;
    }

    let pm = if w.mobile(m).is_some_and(|x| x.player) {
        Some(m)
    } else {
        muo_content::creature::control::summon_master(w, m)
            .filter(|s| w.mobile(*s).is_some_and(|x| x.player))
    };
    if pm
        .and_then(|p| duel_context_of(w, p))
        .and_then(|c| super::ctx(w, c))
        .is_some_and(|c| c.started_begin_countdown)
    {
        return true;
    }

    if flow::check_combat(w, m) {
        mm::send_message_hue(
            w,
            m,
            0x22,
            "You have recently been in combat and cannot enter this zone.",
        );
        return false;
    }

    (GUARDED_REGION.on_move_into)(w, id, m, d, new_location, old_location)
}

fn on_enter(w: &mut World, _id: RegionId, m: MobileId) {
    mm::send_message(
        w,
        m,
        "You have entered a dueling safezone. No combat other than duels are allowed in this zone.",
    );
}

fn on_exit(w: &mut World, _id: RegionId, m: MobileId) {
    mm::send_message(
        w,
        m,
        "You have left a dueling safezone. Combat is now unrestricted.",
    );
}

/// `SafeZone`.
pub static SAFE_ZONE: RegionType = RegionType {
    name: "SafeZone",
    parent: Some(&GUARDED_REGION),
    // `from.AccessLevel >= AccessLevel.GameMaster && base.AllowHousing(from, p)`
    allow_housing: |w, id, from, p| {
        w.mobile(from)
            .is_some_and(|m| m.access_level >= AccessLevel::GameMaster)
            && (GUARDED_REGION.allow_housing)(w, id, from, p)
    },
    on_move_into,
    on_enter,
    on_exit,
    can_use_stuck_menu: |_, _, _| false,
    ..GUARDED_REGION
};

/// `new SafeZone(area, goloc, map, isGuarded)`: registered.
pub fn create(
    w: &mut World,
    area: Rectangle2D,
    go_location: Point3D,
    map: MapId,
    is_guarded: bool,
) -> RegionId {
    let id = region::new_region(
        w,
        &SAFE_ZONE,
        None,
        map,
        None,
        Some(SAFE_ZONE_PRIORITY),
        region::convert_all_to_3d(&[area]),
    );
    base_region::init_data(w, id);
    if let Some(r) = w.region_mut(id) {
        r.go_location = go_location;
    }
    if let Some(d) = base_region::data_mut(w, id) {
        d.guards_disabled = !is_guarded;
    }
    region::register(w, id);
    id
}

/// `Region.IsPartOf<SafeZone>()` of the mobile's region.
pub fn mobile_in_safe_zone(w: &World, m: MobileId) -> bool {
    w.mobile(m)
        .is_some_and(|x| region::is_part_of_type(w, x.region, &SAFE_ZONE))
}

/// Registers the class so `GetRegion<SafeZone>()` finds it.
pub fn configure(w: &mut World) {
    w.resources.get_mut::<RegionHooks>().safe_zone = Some(&SAFE_ZONE);
}
