//! Non-virtual members of `Mobile`: aggressor expiry, action locks, warmode delay, line of sight,
//! paralyze/freeze expiry, aggression bookkeeping, targets and `Move`. The virtual members of the range
//! (`AddSkillMod`, `Move`, `AggressiveAction`..) are in `mobile::base`; they call the helpers here.
//! `ClearTarget` and `BeginTarget` are `target::clear_target` and `target::begin_target`.

use muo_primitives::{Direction, Point3D, TimeSpan};

use crate::ids::{EntityId, ItemId, MobileId};
use crate::map::MapId;
use crate::map::los::LosObject;
use crate::mobile::AccessLevel;
use crate::timer::{self, TimerHandle};
use crate::world::World;

/// `Mobile.WarmodeCatchCount`.
const WARMODE_CATCH_COUNT: i32 = 4;
/// `Mobile.ExpireCombatantDelay` (one minute).
const EXPIRE_COMBATANT_DELAY: TimeSpan = TimeSpan::from_millis(60_000);
/// `Mobile.ExpireAggressorsDelay`.
const EXPIRE_AGGRESSORS_DELAY: TimeSpan = TimeSpan::from_millis(5000);
/// `Core.GlobalUpdateRange`.
// shortcut: there is no setting for it (a property nobody assigns), so a const.
const GLOBAL_UPDATE_RANGE: i32 = 18;
/// `Move`'s drift cap for the connection's `_nextMovementTime`.
const MAX_DRIFT: i64 = 200;

/// `Utility.InUpdateRange(p1, p2)`.
pub fn in_update_range(a: Point3D, b: Point3D) -> bool {
    muo_primitives::utility::in_range_3d(a, b, GLOBAL_UPDATE_RANGE)
}

/// `SendLocalizedMessage(number)`.
fn send_localized(w: &mut World, id: MobileId, number: i32) {
    crate::mobile::part2::messages::send_localized_message(w, id, number, "", 0x3B2);
}

/// Cancels the timer stored in a mobile field and clears it (`token.Cancel()`).
fn cancel_timer(w: &mut World, id: MobileId, field: fn(&mut crate::Mobile) -> &mut TimerHandle) {
    let Some(m) = w.mobile_mut(id) else { return };
    let mut h = *field(m);
    h.cancel(w);
    if let Some(m) = w.mobile_mut(id) {
        *field(m) = h;
    }
}

/// `NetState.SendMobileIncoming(this, other)` behind its guards: the client exists and `CanSee(other)` (and,
/// with `ranged`, `Utility.InUpdateRange`).
#[allow(clippy::needless_return)] // the packet send goes after these checks
fn notify_incoming(w: &mut World, id: MobileId, other: MobileId, ranged: bool) {
    let Some(m) = w.mobile(id) else { return };
    let Some(ns) = m.net_state else { return };
    let (ty, loc) = (m.ty, m.location);
    if !(ty.can_see_mobile)(w, id, other) {
        return;
    }
    if ranged
        && !w
            .mobile(other)
            .is_some_and(|o| in_update_range(loc, o.location))
    {
        return;
    }
    crate::packets::mobile::send_mobile_incoming(w, ns, id, other);
}

pub fn update_aggr_expire(w: &mut World, id: MobileId) {
    let Some(m) = w.mobile(id) else { return };
    if m.deleted || (m.aggressors.is_empty() && m.aggressed.is_empty()) {
        stop_aggr_expire(w, id);
    } else if !m.expire_aggr_timer.running(w) {
        let h = timer::start(
            w,
            EXPIRE_AGGRESSORS_DELAY,
            EXPIRE_AGGRESSORS_DELAY,
            0,
            move |w| expire_aggr(w, id),
        );
        if let Some(m) = w.mobile_mut(id) {
            m.expire_aggr_timer = h;
        }
    }
}

/// `ExpireAggr`, the repeating timer's callback.
fn expire_aggr(w: &mut World, id: MobileId) {
    let Some(m) = w.mobile(id) else {
        // the mobile is gone; stop this timer rather than let it tick forever
        if let Some(mut h) = timer::current(w) {
            h.cancel(w);
        }
        return;
    };
    if m.deleted || (m.aggressors.is_empty() && m.aggressed.is_empty()) {
        stop_aggr_expire(w, id);
    } else {
        check_aggr_expire(w, id);
    }
}

pub fn stop_aggr_expire(w: &mut World, id: MobileId) {
    cancel_timer(w, id, |m| &mut m.expire_aggr_timer);
}

/// `CheckAggrExpire`: drops the expired entries on both lists, and on the other side of each.
pub fn check_aggr_expire(w: &mut World, id: MobileId) {
    let len = w.mobile(id).map_or(0, |m| m.aggressors.len());
    for i in (0..len).rev() {
        let Some(info) = w.mobile(id).and_then(|m| m.aggressors.get(i)).cloned() else {
            continue;
        };
        if info.expired(w) {
            remove_aggressed(w, info.attacker, id);
            if let Some(m) = w.mobile_mut(id)
                && i < m.aggressors.len()
            {
                m.aggressors.remove(i);
            }
            notify_incoming(w, id, info.attacker, true);
        }
    }

    let len = w.mobile(id).map_or(0, |m| m.aggressed.len());
    for i in (0..len).rev() {
        let Some(info) = w.mobile(id).and_then(|m| m.aggressed.get(i)).cloned() else {
            continue;
        };
        if info.expired(w) {
            remove_aggressor(w, info.defender, id);
            if let Some(m) = w.mobile_mut(id)
                && i < m.aggressed.len()
            {
                m.aggressed.remove(i);
            }
            notify_incoming(w, id, info.defender, true);
        }
    }

    update_aggr_expire(w, id);
}

/// `RemoveAggressed(aggressed)`: drops the entry whose defender is `aggressed`.
pub fn remove_aggressed(w: &mut World, id: MobileId, aggressed: MobileId) {
    let Some(m) = w.mobile_mut(id) else { return };
    if m.deleted {
        return;
    }
    if let Some(i) = m.aggressed.iter().position(|a| a.defender == aggressed) {
        m.aggressed.remove(i);
        notify_incoming(w, id, aggressed, false);
    }
    update_aggr_expire(w, id);
}

/// `RemoveAggressor(aggressor)`: drops the entry whose attacker is `aggressor`.
pub fn remove_aggressor(w: &mut World, id: MobileId, aggressor: MobileId) {
    let Some(m) = w.mobile_mut(id) else { return };
    if m.deleted {
        return;
    }
    if let Some(i) = m.aggressors.iter().position(|a| a.attacker == aggressor) {
        m.aggressors.remove(i);
        notify_incoming(w, id, aggressor, false);
    }
    update_aggr_expire(w, id);
}

/// What `AggressiveAction(aggressor, criminal)` does once the event has run: restarts the
/// combatant expiry, refreshes or adds the entries on both mobiles, then sets the combatant. `base` calls it.
pub fn record_aggression(w: &mut World, id: MobileId, aggressor: MobileId, criminal: bool) {
    let now = w.now;
    let Some(ty) = w.mobile(id).map(|m| m.ty) else {
        return;
    };

    if (ty.combatant)(w, id) == Some(aggressor) {
        cancel_timer(w, id, |m| &mut m.expire_combatant_timer);
        let h = timer::start(w, EXPIRE_COMBATANT_DELAY, TimeSpan::ZERO, 1, move |w| {
            super::props_a::expire_combatant(w, id)
        });
        if let Some(m) = w.mobile_mut(id) {
            m.expire_combatant_timer = h;
        }
    }

    let mut add_aggressor = true;
    if let Some(m) = w.mobile_mut(id) {
        for info in m.aggressors.iter_mut().filter(|i| i.attacker == aggressor) {
            info.refresh(now);
            info.criminal_aggression = criminal;
            info.can_report_murder = criminal;
            add_aggressor = false;
        }
    }
    if let Some(a) = w.mobile_mut(aggressor) {
        for info in a.aggressors.iter_mut().filter(|i| i.attacker == id) {
            info.refresh(now);
            add_aggressor = false;
        }
    }

    let mut add_aggressed = true;
    if let Some(m) = w.mobile_mut(id) {
        for info in m.aggressed.iter_mut().filter(|i| i.defender == aggressor) {
            info.refresh(now);
            add_aggressed = false;
        }
    }
    if let Some(a) = w.mobile_mut(aggressor) {
        for info in a.aggressed.iter_mut().filter(|i| i.defender == id) {
            info.refresh(now);
            info.criminal_aggression = criminal;
            info.can_report_murder = criminal;
            add_aggressed = false;
        }
    }

    let mut set_combatant = false;

    if add_aggressor {
        let info = crate::aggressor_info::AggressorInfo::create(w, aggressor, id, criminal);
        if let Some(m) = w.mobile_mut(id) {
            m.aggressors.push(info);
        }
        notify_incoming(w, id, aggressor, false);
        if (ty.combatant)(w, id).is_none() {
            set_combatant = true;
        }
        update_aggr_expire(w, id);
    }

    if add_aggressed {
        let info = crate::aggressor_info::AggressorInfo::create(w, aggressor, id, criminal);
        if let Some(a) = w.mobile_mut(aggressor) {
            a.aggressed.push(info);
        }
        notify_incoming(w, id, aggressor, false);
        if (ty.combatant)(w, id).is_none() {
            set_combatant = true;
        }
        // Called on `this`, not on the aggressor whose list changed
        update_aggr_expire(w, id);
    }

    if set_combatant {
        (ty.set_combatant)(w, id, Some(aggressor));
    }

    // Region.OnAggressed(aggressor, this, criminal)
    let region = w.mobile(id).map(|m| m.region);
    if let Some(r) = region
        && let Some(rty) = w.region(r).map(|r| r.ty)
    {
        (rty.on_aggressed)(w, r, aggressor, id, criminal);
    }
}

/// `DelayChangeWarmode(value)`: throttles warmode flips.
pub fn delay_change_warmode(w: &mut World, id: MobileId, value: bool) {
    let now = w.now;
    let (spam_catch, spam_delay) = (
        super::props_a::warmode_spam_catch(w),
        super::props_a::warmode_spam_delay(w),
    );
    let Some(m) = w.mobile_mut(id) else { return };
    if m.warmode_changes > WARMODE_CATCH_COUNT {
        m.warmode_spam_value = value;
        m.warmode_changes += 1;
        return;
    }
    if m.warmode == value {
        return;
    }
    if now > m.next_warmode_change || m.warmode_changes == 0 {
        m.warmode_changes = 1;
        m.next_warmode_change = now + spam_catch;
    } else {
        let old = m.warmode_changes;
        m.warmode_changes += 1;
        if old == WARMODE_CATCH_COUNT {
            timer::start(w, spam_delay, TimeSpan::ZERO, 1, move |w| {
                warmode_spam_timeout(w, id)
            });
            return;
        }
    }
    super::props_b::set_warmode(w, id, value);
}

fn warmode_spam_timeout(w: &mut World, id: MobileId) {
    let Some(value) = w.mobile(id).map(|m| m.warmode_spam_value) else {
        return;
    };
    super::props_b::set_warmode(w, id, value);
    if let Some(m) = w.mobile_mut(id) {
        m.warmode_changes = 0;
    }
}

pub fn in_los(w: &mut World, id: MobileId, target: MobileId) -> bool {
    let Some(m) = w.mobile(id) else { return false };
    let Some(map) = m.map.filter(|_| !m.deleted) else {
        return false;
    };
    target == id
        || m.access_level > AccessLevel::Player
        || crate::map::los::line_of_sight_mobiles(w, map, id, target)
}

pub fn in_los_object(w: &mut World, id: MobileId, target: LosObject) -> bool {
    let Some(m) = w.mobile(id) else { return false };
    let Some(map) = m.map.filter(|_| !m.deleted) else {
        return false;
    };
    if target == LosObject::Mobile(id) || m.access_level > AccessLevel::Player {
        return true;
    }
    if let LosObject::Item(item) = target
        && crate::map::tiles::root_parent(w, item) == Some(EntityId::Mobile(id))
    {
        return true;
    }
    crate::map::los::line_of_sight_objects(w, map, LosObject::Mobile(id), target)
}

pub fn in_los_point(w: &mut World, id: MobileId, target: Point3D) -> bool {
    let Some(m) = w.mobile(id) else { return false };
    let Some(map) = m.map.filter(|_| !m.deleted) else {
        return false;
    };
    m.access_level > AccessLevel::Player
        || crate::map::los::line_of_sight_mobile_point(w, map, id, target)
}

pub fn at_point(w: &World, id: MobileId, x: i32, y: i32) -> bool {
    w.mobile(id)
        .is_some_and(|m| m.location.x == x && m.location.y == y)
}

/// `BeginAction(toLock)` / `BeginAction<T>()`: false when `key` is already locked. The generic
/// form passes the type's name.
pub fn begin_action(w: &mut World, id: MobileId, key: &'static str) -> bool {
    let Some(m) = w.mobile_mut(id) else {
        return false;
    };
    if m.actions.contains(&key) {
        return false;
    }
    m.actions.push(key);
    true
}

pub fn can_begin_action(w: &World, id: MobileId, key: &'static str) -> bool {
    !w.mobile(id).is_some_and(|m| m.actions.contains(&key))
}

pub fn end_action(w: &mut World, id: MobileId, key: &'static str) {
    if let Some(m) = w.mobile_mut(id)
        && let Some(i) = m.actions.iter().position(|k| *k == key)
    {
        m.actions.remove(i);
    }
}

fn expire_paralyzed(w: &mut World, id: MobileId) {
    if let Some(ty) = w.mobile(id).map(|m| m.ty) {
        (ty.set_paralyzed)(w, id, false);
    }
}

pub fn start_paralyze_timer(w: &mut World, id: MobileId, duration: TimeSpan) {
    let h = timer::start(w, duration, TimeSpan::ZERO, 1, move |w| {
        expire_paralyzed(w, id)
    });
    if let Some(m) = w.mobile_mut(id) {
        m.para_timer = h;
    }
}

fn expire_frozen(w: &mut World, id: MobileId) {
    super::props_a::set_frozen(w, id, false);
}

pub fn start_freeze_timer(w: &mut World, id: MobileId, duration: TimeSpan) {
    let h = timer::start(w, duration, TimeSpan::ZERO, 1, move |w| {
        expire_frozen(w, id)
    });
    if let Some(m) = w.mobile_mut(id) {
        m.frozen_timer = h;
    }
}

/// `ToString()`: `{Serial} "{Name}"`.
pub fn to_string(w: &World, id: MobileId) -> String {
    let Some(m) = w.mobile(id) else {
        return String::new();
    };
    format!(
        "{} \"{}\"",
        m.serial,
        (m.ty.name)(w, id).unwrap_or_default()
    )
}

/// The wait shared by `SendSkillMessage`/`SendActionMessage`: true (and the next time is
/// set) when the message may be sent now.
pub fn claim_action_message(w: &mut World, id: MobileId) -> bool {
    let tick = w.tick_count;
    let delay = i64::from(super::props_a::action_message_delay(w));
    let Some(m) = w.mobile_mut(id) else {
        return false;
    };
    if m.next_action_message.wrapping_sub(tick) >= 0 {
        return false;
    }
    m.next_action_message = tick + delay;
    true
}

/// `SendSkillMessage`'s message once claimed: 500118.
pub fn send_skill_message_text(w: &mut World, id: MobileId) {
    send_localized(w, id, 500118); // You must wait a few moments to use another skill.
}

/// `SendActionMessage`'s message once claimed: 500119.
pub fn send_action_message_text(w: &mut World, id: MobileId) {
    send_localized(w, id, 500119); // You must wait to perform another action.
}

/// `Mobile` or `Item`, for the `OnMoveOff`/`OnMoveOver`/`OnMovement` calls `Move` makes on both.
#[derive(Copy, Clone)]
enum Mover {
    Mobile(MobileId),
    Item(ItemId),
}

fn on_move_off(w: &mut World, e: Mover, m: MobileId) -> bool {
    match e {
        Mover::Mobile(o) => match w.mobile(o).map(|x| x.ty) {
            Some(ty) => (ty.on_move_off)(w, o, m),
            None => true,
        },
        Mover::Item(o) => match w.item(o).map(|x| x.ty) {
            Some(ty) => (ty.on_move_off)(w, o, m),
            None => true,
        },
    }
}

fn on_move_over(w: &mut World, e: Mover, m: MobileId) -> bool {
    match e {
        Mover::Mobile(o) => match w.mobile(o).map(|x| x.ty) {
            Some(ty) => (ty.on_move_over)(w, o, m),
            None => true,
        },
        Mover::Item(o) => match w.item(o).map(|x| x.ty) {
            Some(ty) => (ty.on_move_over)(w, o, m),
            None => true,
        },
    }
}

/// `m != this && m.X == x && m.Y == y && m.Z + 15 > z && z + 15 > m.Z` (the `!= this` only when `skip_self`).
fn mobile_at(
    w: &World,
    id: MobileId,
    o: MobileId,
    x: i32,
    y: i32,
    z: i32,
    skip_self: bool,
) -> bool {
    if skip_self && o == id {
        return false;
    }
    w.mobile(o).is_some_and(|m| {
        m.location.x == x && m.location.y == y && m.location.z + 15 > z && z + 15 > m.location.z
    })
}

/// `item.AtWorldPoint(x, y) && (item.Z == z || item.Z + item.ItemData.Height > z && z + 15 > item.Z)`.
fn item_at(w: &World, item: ItemId, x: i32, y: i32, z: i32) -> bool {
    let Some(i) = w.item(item) else { return false };
    if !crate::map::tiles::at_world_point(i, x, y) {
        return false;
    }
    let h = i32::from(crate::map::tiles::item_data(w, i).height);
    i.location.z == z || (i.location.z + h > z && z + 15 > i.location.z)
}

/// `InternalOnMove(d)`: the virtual `OnMove`, then the `Movement` event.
pub fn internal_on_move(w: &mut World, id: MobileId, d: Direction) -> bool {
    let Some(ty) = w.mobile(id).map(|m| m.ty) else {
        return false;
    };
    if !(ty.on_move)(w, id, d) {
        return false;
    }
    let mut e = crate::events::MovementEventArgs::new(id, d);
    crate::events::invoke(w, |ev| &mut ev.movement, &mut e);
    !e.blocked
}

/// `CanMove(d, oldLocation, out newLocation)`. The new location is returned even on failure.
pub fn can_move(w: &mut World, id: MobileId, d: Direction, old: Point3D) -> (bool, Point3D) {
    let mut new_loc = old;
    let Some(ty) = w.mobile(id).map(|m| m.ty) else {
        return (false, new_loc);
    };

    if !crate::mobile::part2::deps::spell_on_caster_moving(w, id, d) {
        return (false, new_loc);
    }
    if w.mobile(id).is_some_and(|m| m.paralyzed || m.frozen) {
        send_localized(w, id, 500111); // You are frozen and can not move.
        return (false, new_loc);
    }

    let (ok, new_z) = (ty.check_movement)(w, id, d);
    if !ok {
        return (false, new_loc);
    }

    let (old_x, old_y, old_z) = (old.x, old.y, old.z);
    let (mut x, mut y) = (old_x, old_y);
    crate::movement::offset_xy(d, &mut x, &mut y, 1);
    new_loc = Point3D::new(x, y, new_z);

    let map: Option<MapId> = w.mobile_mut(id).and_then(|m| {
        m.pushing = false;
        m.map
    });
    let Some(map_id) = map else {
        return (false, new_loc);
    };
    if !crate::region::can_move(w, id, d, new_loc, old, Some(map_id)) {
        return (false, new_loc);
    }

    let Some(map) = w.maps.get(map_id) else {
        return (false, new_loc);
    };
    let same_sector = map.sector_key(old_x >> 4, old_y >> 4) == map.sector_key(x >> 4, y >> 4);
    let old_mobiles = map.sector_at(old_x, old_y).mobiles.clone();
    let old_items = map.sector_at(old_x, old_y).items.clone();
    let new_mobiles = map.sector_at(x, y).mobiles.clone();
    let new_items = map.sector_at(x, y).items.clone();

    if !same_sector {
        // off the old tile (mobiles, then items), then over the new one. Each group is gathered up front;
        // the group's members are re-checked as they are reached
        for o in old_mobiles {
            if mobile_at(w, id, o, old_x, old_y, old_z, true)
                && !on_move_off(w, Mover::Mobile(o), id)
            {
                return (false, new_loc);
            }
        }
        for i in old_items {
            if item_at(w, i, old_x, old_y, old_z) && !on_move_off(w, Mover::Item(i), id) {
                return (false, new_loc);
            }
        }
        for o in new_mobiles {
            if mobile_at(w, id, o, x, y, new_z, false) && !on_move_over(w, Mover::Mobile(o), id) {
                return (false, new_loc);
            }
        }
        for i in new_items {
            if item_at(w, i, x, y, new_z) && !on_move_over(w, Mover::Item(i), id) {
                return (false, new_loc);
            }
        }
    } else {
        // one sector: each entity gets flags (1 = moving off, 2 = moving over), all gathered per kind first
        let mut flagged: Vec<(Mover, u8)> = Vec::new();
        for o in old_mobiles {
            let mut flag = u8::from(mobile_at(w, id, o, old_x, old_y, old_z, true));
            if mobile_at(w, id, o, x, y, new_z, false) {
                flag += 2;
            }
            if flag > 0 {
                flagged.push((Mover::Mobile(o), flag));
            }
        }
        for (e, flag) in flagged {
            if (flag > 0 && !on_move_off(w, e, id)) || (flag > 1 && !on_move_over(w, e, id)) {
                return (false, new_loc);
            }
        }
        let mut flagged: Vec<(Mover, u8)> = Vec::new();
        for i in old_items {
            let mut flag = u8::from(item_at(w, i, old_x, old_y, old_z));
            if item_at(w, i, x, y, new_z) {
                flag += 2;
            }
            if flag > 0 {
                flagged.push((Mover::Item(i), flag));
            }
        }
        for (e, flag) in flagged {
            if (flag > 0 && !on_move_off(w, e, id)) || (flag > 1 && !on_move_over(w, e, id)) {
                return (false, new_loc);
            }
        }
    }

    if !internal_on_move(w, id, d) {
        return (false, new_loc);
    }

    let tick = w.tick_count;
    if let Some(m) = w.mobile_mut(id) {
        m.last_move_time = tick;
    }
    (true, new_loc)
}

/// `ComputeMovementSpeed()`: the speed for the mobile's own direction, without the turn check.
pub fn compute_movement_speed_current(w: &World, id: MobileId) -> i32 {
    let Some(m) = w.mobile(id) else { return 0 };
    (m.ty.compute_movement_speed)(w, id, m.direction, false)
}

/// The `NetState._nextMovementTime` bookkeeping of `Move`.
pub fn charge_movement(w: &mut World, id: MobileId, d: Direction) {
    let Some(m) = w.mobile(id) else { return };
    let Some(ns) = m.net_state else { return };
    let cost = i64::from((m.ty.compute_movement_speed)(w, id, d, true));
    let now = w.tick_count;
    let mut seq = 0;
    if let Some(data) = w.net_state_mut(ns) {
        let mv = &mut data.movement;
        seq = mv.sequence;
        let delta = now - mv.next_movement_time;
        if cost == 0 {
            // direction-only turn: reset to now to prevent turn accumulation
            mv.next_movement_time = now;
        } else {
            // never more than MAX_DRIFT behind now, so lateness can't be banked for speed hacking
            if delta > MAX_DRIFT {
                mv.next_movement_time = now - MAX_DRIFT;
            }
            mv.next_movement_time += cost;
        }
    }
    crate::packets::movement::send_movement_ack_mobile(w, ns, seq, id);
}

/// What `Move` does after `SetLocation`/`SetDirection`: the clients in range are sent the
/// move and everything near that wants it gets `OnMovement(this, oldLocation)`.
pub fn notify_moved(w: &mut World, id: MobileId, old_location: Point3D) {
    let Some(m) = w.mobile(id) else { return };
    let (Some(map), loc) = (m.map, m.location) else {
        return;
    };
    let range = w.maps.global_max_update_range;

    let mut move_queue: Vec<Mover> =
        crate::map::enumerate::mobiles_in_range(w, Some(map), loc.x, loc.y, range)
            .filter(|&o| o != id)
            .map(Mover::Mobile)
            .collect();
    let viewers: Vec<(MobileId, crate::net_state::NetStateId)> = move_queue
        .iter()
        .filter_map(|e| match *e {
            Mover::Mobile(o) => w.mobile(o).and_then(|x| x.net_state.map(|ns| (o, ns))),
            _ => None,
        })
        .collect();
    let mut cache = crate::packets::mobile::MobileMovingCache::default();
    for (o, ns) in viewers {
        let near = w
            .mobile(o)
            .map(|x| (x.ty, in_update_range(loc, x.location)));
        let ok = near.is_some_and(|(ty, near)| near && (ty.can_see_mobile)(w, o, id));
        if ok {
            crate::packets::mobile::send_mobile_moving_using_cache_notoriety(
                w, ns, &mut cache, o, id,
            );
        }
    }
    move_queue.extend(
        crate::map::enumerate::items_in_range(w, Some(map), loc.x, loc.y, range)
            .filter(|&i| {
                w.item(i)
                    .is_some_and(|it| (it.ty.handles_on_movement)(w, i))
            })
            .map(Mover::Item),
    );
    for e in move_queue {
        match e {
            Mover::Mobile(o) => {
                if let Some(ty) = w.mobile(o).map(|x| x.ty) {
                    (ty.on_movement)(w, o, id, old_location);
                }
            }
            Mover::Item(o) => {
                if let Some(ty) = w.item(o).map(|x| x.ty) {
                    (ty.on_movement)(w, o, id, old_location);
                }
            }
        }
    }
}

pub fn clear_target(w: &mut World, id: MobileId) {
    if let Some(m) = w.mobile_mut(id) {
        m.target = None;
    }
}

// `BeginTarget` (SimpleTarget/SimpleStateTarget) is `target::begin_target`: a closure replaces the callback and state.

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aggressor_info::AggressorInfo;
    use crate::mobile::MOBILE_BASE;

    fn two() -> (World, MobileId, MobileId) {
        let mut w = World::new();
        let a = w.new_mobile(&MOBILE_BASE);
        let b = w.new_mobile(&MOBILE_BASE);
        (w, a, b)
    }

    #[test]
    fn aggressive_action_adds_both_sides_once() {
        let (mut w, victim, attacker) = two();
        (MOBILE_BASE.aggressive_action_criminal)(&mut w, victim, attacker, true);
        let v = w.mobile(victim).unwrap();
        assert_eq!(v.aggressors.len(), 1);
        assert_eq!(
            (v.aggressors[0].attacker, v.aggressors[0].defender),
            (attacker, victim)
        );
        assert!(v.aggressors[0].criminal_aggression && v.aggressors[0].can_report_murder);
        assert_eq!(w.mobile(attacker).unwrap().aggressed.len(), 1);
        assert!(v.expire_aggr_timer.running(&w));

        // a second hit refreshes instead of adding
        w.now += TimeSpan::from_secs(30);
        (MOBILE_BASE.aggressive_action_criminal)(&mut w, victim, attacker, false);
        let v = w.mobile(victim).unwrap();
        assert_eq!(v.aggressors.len(), 1);
        assert_eq!(v.aggressors[0].last_combat_time, w.now);
        assert!(!v.aggressors[0].criminal_aggression);
        assert_eq!(w.mobile(attacker).unwrap().aggressed.len(), 1);
    }

    #[test]
    fn self_aggression_is_ignored() {
        let (mut w, a, _) = two();
        (MOBILE_BASE.aggressive_action_criminal)(&mut w, a, a, false);
        assert!(w.mobile(a).unwrap().aggressors.is_empty());
    }

    #[test]
    fn expiry_removes_both_sides_and_stops_timer() {
        let (mut w, victim, attacker) = two();
        (MOBILE_BASE.aggressive_action)(&mut w, victim, attacker);
        w.now += TimeSpan::from_mins(3);
        check_aggr_expire(&mut w, victim);
        assert!(w.mobile(victim).unwrap().aggressors.is_empty());
        assert!(w.mobile(attacker).unwrap().aggressed.is_empty());
        assert!(!w.mobile(victim).unwrap().expire_aggr_timer.running(&w));
    }

    #[test]
    fn remove_aggressor_and_aggressed() {
        let (mut w, victim, attacker) = two();
        let info = AggressorInfo::create(&w, attacker, victim, false);
        w.mobile_mut(victim).unwrap().aggressors.push(info.clone());
        w.mobile_mut(attacker).unwrap().aggressed.push(info);
        remove_aggressor(&mut w, victim, attacker);
        remove_aggressed(&mut w, attacker, victim);
        assert!(w.mobile(victim).unwrap().aggressors.is_empty());
        assert!(w.mobile(attacker).unwrap().aggressed.is_empty());
    }

    #[test]
    fn action_locks() {
        let (mut w, a, _) = two();
        assert!(begin_action(&mut w, a, "k"));
        assert!(!begin_action(&mut w, a, "k"));
        assert!(!can_begin_action(&w, a, "k"));
        end_action(&mut w, a, "k");
        assert!(can_begin_action(&w, a, "k"));
    }

    #[test]
    fn skill_mods_add_get_remove() {
        use crate::mods::SkillMod;
        use crate::skills::SkillName;
        let (mut w, a, _) = two();
        let m = SkillMod::new_default(&mut w, SkillName::Magery, Some("x"), true, 5.0);
        let mid = m.id;
        (MOBILE_BASE.add_skill_mod)(&mut w, a, m);
        assert_eq!(w.mobile(a).unwrap().skill_mods[0].owner, Some(a));
        assert_eq!((MOBILE_BASE.get_skill_mod)(&w, a, "x"), Some(0));
        assert_eq!((MOBILE_BASE.get_skill_mod)(&w, a, "y"), None);
        (MOBILE_BASE.remove_skill_mod)(&mut w, a, mid);
        assert!(w.mobile(a).unwrap().skill_mods.is_empty());

        let t = SkillMod::new_timed(
            &mut w,
            SkillName::Magery,
            Some("t"),
            true,
            1.0,
            TimeSpan::from_secs(5),
        );
        (MOBILE_BASE.add_skill_mod)(&mut w, a, t);
        w.now += TimeSpan::from_secs(6);
        (MOBILE_BASE.validate_skill_mods)(&mut w, a);
        assert!(w.mobile(a).unwrap().skill_mods.is_empty());
        let d = SkillMod::new_default(&mut w, SkillName::Tactics, Some("n"), true, 1.0);
        (MOBILE_BASE.add_skill_mod)(&mut w, a, d);
        (MOBILE_BASE.remove_skill_mod_by_name)(&mut w, a, "n");
        assert!(w.mobile(a).unwrap().skill_mods.is_empty());
    }
}
