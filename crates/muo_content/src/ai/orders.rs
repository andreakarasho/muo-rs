//! Pet orders: the Issue phase (what a command does when given) and the Obey phase
//! (what a pet does each think under its standing order).

use muo_primitives::{DateTime, MessageType, Point3D};
use muo_server::ids::MobileId;
use muo_server::mobile::part2::messages;
use muo_server::world::World;

use super::base::{debug_say, is_valid_combatant};
use super::{ActionType, ai, ai_mut, movement, set_action};
use crate::creature::control::{self, control_master, issue_order as creature_issue_order};
use crate::creature::data::{cd, cd_mut, read};
use crate::creature::enums::*;
use crate::creature::misc::distance_sqrt;
use crate::creature::vtable::ctype;
use crate::hooks::pet_player_hooks;
use crate::mx;

pub fn is_restable_order(order: OrderType) -> bool {
    matches!(
        order,
        OrderType::None
            | OrderType::Come
            | OrderType::Guard
            | OrderType::Attack
            | OrderType::Stay
            | OrderType::Follow
    )
}

pub fn is_friend_order(order: OrderType) -> bool {
    matches!(order, OrderType::Follow | OrderType::Stay | OrderType::Stop)
}

// Everything else stands the pet down before it runs.
fn keeps_combat_posture(order: OrderType) -> bool {
    matches!(
        order,
        OrderType::Attack
            | OrderType::Guard
            | OrderType::Drop
            | OrderType::Friend
            | OrderType::Unfriend
            | OrderType::Rename
    )
}

/// Publish 51: told any of these, a pet "will not attack anything, even if it is attacked".
pub fn is_stand_down_order(order: OrderType) -> bool {
    matches!(
        order,
        OrderType::Follow | OrderType::Come | OrderType::Stay | OrderType::None
    )
}

/// Orders a dead bonded pet refuses.
pub fn is_dead_pet_order(order: OrderType) -> bool {
    matches!(
        order,
        OrderType::Guard | OrderType::Attack | OrderType::Transfer | OrderType::Drop
    )
}

/// `AIObject.PersistentOrder`.
pub fn persistent_order(w: &World, id: MobileId) -> OrderType {
    ai(w, id).map_or(OrderType::None, |a| a.persistent_order)
}

/// SetPersistentOrder: the controlled-pet wander anchor (Home) is a pure function of the persistent command.
pub fn set_persistent_order_pub(w: &mut World, id: MobileId, order: OrderType) {
    set_persistent_order(w, id, order);
}

fn set_persistent_order(w: &mut World, id: MobileId, order: OrderType) {
    let target = if order == OrderType::Follow {
        read(w, id, None, |d| d.control_target)
    } else {
        None
    };
    if let Some(a) = ai_mut(w, id) {
        a.persistent_order = order;
        a.persistent_target = target;
    }
    let home = if matches!(order, OrderType::Follow | OrderType::Guard) {
        Point3D::ZERO
    } else {
        mx::location(w, id)
    };
    control::set_home(w, id, home);
}

/// RestorePersistentOrder: adopt a saved standing order; Home and ControlTarget were saved with it.
pub fn restore_persistent_order(w: &mut World, id: MobileId, order: OrderType) {
    let target = if order == OrderType::Follow {
        read(w, id, None, |d| d.control_target)
    } else {
        None
    };
    if let Some(a) = ai_mut(w, id) {
        a.persistent_order = order;
        a.persistent_target = target;
    }
}

fn is_invalid_control_target(w: &World, id: MobileId, target: Option<MobileId>) -> bool {
    match target {
        None => true,
        Some(t) => {
            mx::deleted(w, t)
                || mx::map(w, t) != mx::map(w, id)
                || !mx::alive(w, t)
                || w.mobile(t)
                    .is_some_and(|m| (m.ty.is_dead_bonded_pet)(w, t) || m.hidden)
        }
    }
}

/// The standing order is the fallback only for an interrupted order that cannot resume: a transient, or an attack
/// whose target is gone.
fn resume_interrupted(
    w: &mut World,
    id: MobileId,
    previous: OrderType,
    interrupted: Option<MobileId>,
) -> OrderType {
    if !is_restable_order(previous)
        || previous == OrderType::Attack && is_invalid_control_target(w, id, interrupted)
    {
        return persistent_order(w, id);
    }
    if let Some(d) = cd_mut(w, id) {
        d.control_target = interrupted;
    }
    previous
}

/// Resume the standing command without re-deriving it or re-anchoring Home.
fn resume_persistent_order(w: &mut World, id: MobileId) {
    let p = persistent_order(w, id);
    let target = read(w, id, None, |d| d.control_target);
    control::set_control_order(w, id, p, None, true, target);
}

fn name(w: &World, id: MobileId) -> String {
    muo_server::mobile::base::name_base(w, id).unwrap_or_default()
}

fn young(w: &World, m: MobileId) -> bool {
    pet_player_hooks(w)
        .and_then(|h| h.young)
        .is_some_and(|f| f(w, m))
}

/// Issue phase. `issuer` is the only mobile revealed (None = system-issued); `resuming` marks a fallback to the
/// standing order. Returns the order to rest in.
pub fn issue_order(
    w: &mut World,
    id: MobileId,
    order: OrderType,
    previous: OrderType,
    issuer: Option<MobileId>,
    resuming: bool,
    interrupted: Option<MobileId>,
) -> OrderType {
    if mx::deleted(w, id) {
        return order;
    }
    super::prod(w, id);
    if let Some(i) = issuer
        && let Some(ty) = w.mobile(i).map(|m| m.ty)
    {
        (ty.revealing_action)(w, i);
    }
    // Dropping Warmode nulls Combatant through the Mobile setter, which would turn Attack's single Combatant write
    // into a re-write (DoHarmful again) and flap Guard's war stance.
    crate::creature::edit(w, id, |d| d.focus_mob = None);
    if !keeps_combat_posture(order) {
        mx::set_warmode(w, id, false); // also nulls Combatant via the setter
        mx::set_combatant(w, id, None);
    }
    let speeds = crate::creature::speeds::set_current_speed_to_passive;
    let active = crate::creature::speeds::set_current_speed_to_active;
    let idle = |w: &mut World| {
        let snd = w
            .mobile(id)
            .map(|m| m.ty)
            .map_or(0, |m| (m.get_idle_sound)(w, id));
        mx::play_sound(w, id, snd);
    };
    match order {
        OrderType::None => {
            crate::creature::edit(w, id, |d| d.control_target = None);
            speeds(w, id);
            OrderType::None
        }
        OrderType::Come => {
            active(w, id);
            OrderType::Come
        }
        OrderType::Stay => {
            speeds(w, id);
            if resuming {
                crate::creature::edit(w, id, |d| d.control_target = None); // a transient's target does not carry over
            } else {
                set_persistent_order(w, id, OrderType::Stay); // anchors Home at the post
                idle(w);
            }
            OrderType::Stay
        }
        OrderType::Follow => {
            active(w, id);
            if resuming {
                // the standing Follow's target, never a transient's
                let pt = ai(w, id)
                    .and_then(|a| a.persistent_target)
                    .filter(|&t| !mx::deleted(w, t));
                let t = pt.or_else(|| control_master(w, id));
                crate::creature::edit(w, id, |d| d.control_target = t);
            } else {
                set_persistent_order(w, id, OrderType::Follow); // Home = Zero, remembers the target
                idle(w);
            }
            OrderType::Follow
        }
        OrderType::Guard => {
            mx::set_warmode(w, id, true); // the guard order opens in war stance
            active(w, id);
            if resuming {
                crate::creature::edit(w, id, |d| d.control_target = None);
            } else {
                set_persistent_order(w, id, OrderType::Guard);
                let snd = w
                    .mobile(id)
                    .map(|m| m.ty)
                    .map_or(0, |m| (m.get_attack_sound)(w, id));
                mx::play_sound(w, id, snd);
                if let Some(m) = control_master(w, id) {
                    let n = name(w, id);
                    mx::msg_args(w, m, 1049671, &n); // ~1_NAME~ is now guarding you.
                }
            }
            OrderType::Guard
        }
        OrderType::Attack => {
            let target = read(w, id, None, |d| d.control_target);
            let valid = target.is_some_and(|t| !mx::deleted(w, t) && mx::alive(w, t));
            let t = if valid { target } else { None };
            crate::creature::edit(w, id, |d| d.focus_mob = t);
            mx::set_combatant(w, id, t); // the one Combatant write of the Attack command
            if valid {
                set_action(w, id, ActionType::Combat);
            }
            mx::set_warmode(w, id, true);
            active(w, id);
            // A resumed attack is not a new command: no bark.
            if !resuming {
                let snd = w
                    .mobile(id)
                    .map(|m| m.ty)
                    .map_or(0, |m| (m.get_attack_sound)(w, id));
                mx::play_sound(w, id, snd);
            }
            OrderType::Attack
        }
        OrderType::Stop => {
            crate::creature::edit(w, id, |d| d.control_target = None);
            match previous {
                OrderType::Follow | OrderType::Guard | OrderType::Stay => {
                    set_persistent_order(w, id, OrderType::None); // cancel the standing order; idle anchor = here
                    OrderType::None
                }
                _ => {
                    // No standing order: idle here, anchored (a Zero Home wanders without bounds).
                    if persistent_order(w, id) == OrderType::None {
                        set_persistent_order(w, id, OrderType::None);
                    }
                    persistent_order(w, id)
                }
            }
        }
        OrderType::Drop => {
            let (dead, can_drop) = (
                read(w, id, false, |d| d.is_dead_pet),
                ctype(w, id).is_some_and(|c| (c.can_drop)(w, id)),
            );
            if !dead && can_drop {
                debug_say(w, id, "I am ordered to drop my items.");
                drop_items(w, id);
            }
            resume_interrupted(w, id, previous, interrupted)
        }
        OrderType::Friend => issue_friend(w, id, previous, interrupted),
        OrderType::Unfriend => issue_unfriend(w, id, previous, interrupted),
        OrderType::Release => issue_release(w, id),
        OrderType::Transfer => issue_transfer(w, id),
        OrderType::Rename => {
            let to = issuer.or_else(|| control_master(w, id));
            if let Some(to) = to {
                if read(w, id, false, |d| d.summoned) {
                    mx::send_message(w, to, "You cannot rename a summoned creature.");
                } else {
                    mx::send_message(w, to, "Change name on pet health bar.");
                }
            }
            resume_interrupted(w, id, previous, interrupted)
        }
        // Patrol and anything unimplemented
        _ => persistent_order(w, id),
    }
}

/// IssueTransfer: opens a secure trade holding a `TransferItem` for the pet.
fn issue_transfer(w: &mut World, id: MobileId) -> OrderType {
    use super::transfer::{is_in_combat, new_transfer_item, send_transfer_refusal};
    use muo_server::secure_trade;
    let keep = |w: &World| persistent_order(w, id);
    if read(w, id, false, |d| d.is_dead_pet) {
        return keep(w);
    }
    let (from, to) = (
        control_master(w, id),
        read(w, id, None, |d| d.control_target),
    );
    let (Some(from), Some(to)) = (from, to) else {
        return keep(w);
    };
    if mx::deleted(w, from) || mx::deleted(w, to) || from == to || !mx::player(w, to) {
        return keep(w);
    }
    debug_say(w, id, &format!("Beginning transfer with {}", name(w, to)));
    let (yf, yt) = (young(w, from), young(w, to));
    if yf && !yt {
        mx::msg(w, from, 502040); // As a young player, you may not friend pets to older players.
        return keep(w);
    }
    if !yf && yt {
        mx::msg(w, from, 502041); // As an older player, you may not friend pets to young players.
        return keep(w);
    }
    let c = ctype(w, id).expect("creature");
    if !(c.can_be_controlled_by)(w, id, to) {
        send_transfer_refusal(w, from, to, 1043248, 1043249);
        return keep(w);
    }
    if !(c.can_be_controlled_by)(w, id, from) {
        send_transfer_refusal(w, from, to, 1043250, 1043251);
        return keep(w);
    }
    // The stand-down already cleared Combatant; the aggressor lists and the combat cooldown gate this.
    let next_combat = w.mobile(id).map_or(0, |m| m.next_combat_time);
    if is_in_combat(w, id) || w.tick_count.wrapping_sub(next_combat) < 0 {
        mx::send_message(w, from, "You can not transfer a pet while in combat.");
        mx::send_message(w, to, "You can not transfer a pet while in combat.");
        return keep(w);
    }
    let ns = |w: &World, m| w.mobile(m).and_then(|x| x.net_state);
    let (Some(from_state), Some(to_state)) = (ns(w, from), ns(w, to)) else {
        return keep(w);
    };
    let has_trade = |w: &World, m| muo_server::mobile::part1::props_b::has_trade(w, m);
    if has_trade(w, from) || has_trade(w, to) {
        mx::msg(w, from, 1010507); // You cannot transfer a pet with a trade pending
        mx::msg(w, to, 1010507);
        return keep(w);
    }
    if let Some(container) = secure_trade::add_trade(w, from_state, to_state) {
        let item = new_transfer_item(w, id);
        if let Some(ty) = w.item(container).map(|i| i.ty) {
            (ty.drop_item)(w, container, item);
        }
    }
    // Hold position while the trade window is open.
    let snd = w
        .mobile(id)
        .map(|m| m.ty)
        .map_or(0, |m| (m.get_idle_sound)(w, id));
    mx::play_sound(w, id, snd);
    crate::creature::speeds::set_current_speed_to_passive(w, id);
    set_persistent_order(w, id, OrderType::Stay);
    OrderType::Stay
}

fn drop_items(w: &mut World, id: MobileId) {
    let Some(pack) = muo_server::mobile::part1::props_b::backpack(w, id) else {
        return;
    };
    let items: Vec<_> = w.item(pack).map(|i| i.items.clone()).unwrap_or_default();
    let (loc, map) = (mx::location(w, id), mx::map(w, id));
    if let Some(map) = map {
        for it in items.into_iter().rev() {
            muo_server::item::ops::move_to_world_map(w, it, loc, Some(map));
        }
    }
}

fn issue_friend(
    w: &mut World,
    id: MobileId,
    previous: OrderType,
    interrupted: Option<MobileId>,
) -> OrderType {
    let from = control_master(w, id);
    let to = read(w, id, None, |d| d.control_target);
    let resume = |w: &mut World| resume_interrupted(w, id, previous, interrupted);
    let Some(from) = from.filter(|&f| !mx::deleted(w, f)) else {
        return resume(w);
    };
    let (yf, yt) = (young(w, from), to.is_some_and(|t| young(w, t)));
    if yf && !yt {
        mx::msg(w, from, 502040); // As a young player, you may not friend pets to older players.
        return resume(w);
    }
    if !yf && yt {
        mx::msg(w, from, 502041); // As an older player, you may not friend pets to young players.
        return resume(w);
    }
    let Some(to) = to.filter(|&t| !mx::deleted(w, t) && t != from && mx::player(w, t)) else {
        messages::public_overhead_message_number(
            w,
            id,
            MessageType::REGULAR,
            0x3B2,
            502039,
            "",
            false,
        ); // *looks confused*
        return resume(w);
    };
    let beneficial = w
        .mobile(from)
        .map(|m| m.ty)
        .is_some_and(|ty| (ty.can_be_beneficial_msg)(w, from, to, true));
    if !beneficial {
        return resume(w);
    }
    let has_trade = |w: &World, m| muo_server::mobile::part1::props_b::has_trade(w, m);
    if has_trade(w, from) || has_trade(w, to) {
        let who = if has_trade(w, from) { from } else { to };
        mx::msg(w, who, 1070947); // You cannot friend a pet with a trade pending
        return resume(w);
    }
    let c = ctype(w, id).expect("creature");
    if (c.is_pet_friend)(w, id, to) {
        mx::msg(w, from, 1049691); // That person is already a friend.
        return resume(w);
    }
    if !(c.allow_new_pet_friend)(w, id) {
        mx::msg(w, from, 1005482); // Your pet does not seem to be interested in making new friends right now.
        return resume(w);
    }
    let (pn, tn, fname) = (name(w, id), name(w, to), name(w, from));
    mx::msg_args(w, from, 1049676, &format!("{pn}\t{tn}")); // ~1_NAME~ will now accept movement commands from ~2_NAME~.
    mx::msg_args(w, to, 1043246, &format!("{fname}\t{pn}")); // ~1_NAME~ has granted you the ability to give orders to their pet ~2_PET_NAME~.
    // This creature will now consider you as a friend.
    (c.add_pet_friend)(w, id, to);
    resume(w)
}

fn issue_unfriend(
    w: &mut World,
    id: MobileId,
    previous: OrderType,
    interrupted: Option<MobileId>,
) -> OrderType {
    let from = control_master(w, id);
    let to = read(w, id, None, |d| d.control_target);
    let resume = |w: &mut World| resume_interrupted(w, id, previous, interrupted);
    let (Some(from), Some(to)) = (from, to) else {
        messages::public_overhead_message_number(
            w,
            id,
            MessageType::REGULAR,
            0x3B2,
            502039,
            "",
            false,
        );
        return resume(w);
    };
    if mx::deleted(w, from) || mx::deleted(w, to) || from == to || !mx::player(w, to) {
        messages::public_overhead_message_number(
            w,
            id,
            MessageType::REGULAR,
            0x3B2,
            502039,
            "",
            false,
        ); // *looks confused*
        return resume(w);
    }
    let c = ctype(w, id).expect("creature");
    if !(c.is_pet_friend)(w, id, to) {
        mx::msg(w, from, 1070953); // That person is not a friend.
        return resume(w);
    }
    let (pn, tn, fname) = (name(w, id), name(w, to), name(w, from));
    mx::msg_args(w, from, 1070951, &format!("{pn}\t{tn}")); // ~1_NAME~ will no longer accept movement commands from ~2_NAME~.
    mx::msg_args(w, to, 1070952, &format!("{fname}\t{pn}")); // ~1_NAME~ has no longer granted you the ability to give orders to their pet ~2_PET_NAME~.
    (c.remove_pet_friend)(w, id, to);
    resume(w)
}

/// SetControlMaster(null) assigns ControlOrder = None underneath; the funnel keeps that write.
fn issue_release(w: &mut World, id: MobileId) -> OrderType {
    if read(w, id, false, |d| d.summoned) {
        if let Some(ty) = w.mobile(id).map(|m| m.ty) {
            (ty.kill)(w, id);
        }
        // A vetoed death leaves the summon controlled; it keeps its standing order.
        return if mx::deleted(w, id) || !mx::alive(w, id) {
            OrderType::None
        } else {
            persistent_order(w, id)
        };
    }
    debug_say(w, id, "I have been released to the wild.");
    if w.mobile(id)
        .and_then(|m| (m.ty.name)(w, id))
        .is_some_and(|n| !n.is_empty())
        && let Some(ty) = w.mobile(id).map(|m| m.ty)
    {
        (ty.set_name)(w, id, None);
    }
    let snd = w
        .mobile(id)
        .map(|m| m.ty)
        .map_or(0, |m| (m.get_idle_sound)(w, id));
    mx::play_sound(w, id, snd);
    crate::creature::edit(w, id, |d| {
        d.control_target = None;
        d.bonding_begin = DateTime::MIN;
        d.owner_abandon_time = DateTime::MIN;
    });
    control::set_is_bonded(w, id, false);
    // Nothing of the old master survives a re-tame.
    control::clear_pet_friends(w, id);
    if let Some(a) = ai_mut(w, id) {
        a.persistent_order = OrderType::None;
        a.persistent_target = None;
    }
    control::set_control_master(w, id, None);
    use muo_server::mobile::part2::deps as sd;
    if let Some(sp) = w.mobile(id).and_then(|m| m.spawner) {
        let map = w
            .item(muo_server::ids::ItemId(muo_primitives::Serial::new(sp.0)))
            .and_then(|i| i.map)
            .unwrap_or(muo_server::map::MapId::INTERNAL);
        let home = sd::spawner_get_spawn_position(w, sp, id, map);
        control::set_home(w, id, home);
        let range = sd::spawner_walking_range(w, sp);
        crate::creature::data::edit(w, id, |d| d.range_home = range);
    } else {
        // No spawner: anchor here rather than path toward a stale stay anchor.
        let loc = mx::location(w, id);
        control::set_home(w, id, loc);
        set_action(w, id, ActionType::Wander);
    }
    let c = ctype(w, id).expect("creature");
    if (c.delete_on_release)(w, id) || read(w, id, false, |d| d.is_dead_pet) {
        if let Some(ty) = w.mobile(id).map(|m| m.ty) {
            (ty.delete)(w, id);
        }
    } else {
        control::begin_delete_timer(w, id);
        if (c.can_drop)(w, id) {
            (c.drop_backpack)(w, id);
        }
    }
    OrderType::None
}

// ---- Obey ------------------------------------------------------------------------------------------------

/// Only restable orders arrive here; anything else is a pre-refactor save and resumes the standing order.
pub fn obey(w: &mut World, id: MobileId) -> bool {
    if mx::deleted(w, id) {
        return false;
    }
    match read(w, id, OrderType::None, |d| d.control_order) {
        OrderType::None => do_order_none(w, id),
        OrderType::Come => do_order_come(w, id),
        OrderType::Guard => do_order_guard(w, id),
        OrderType::Attack => do_order_attack(w, id),
        OrderType::Stay => do_order_stay(w, id),
        OrderType::Follow => do_order_follow(w, id),
        _ => {
            resume_persistent_order(w, id);
            true
        }
    }
}

pub(crate) fn do_order_none(w: &mut World, id: MobileId) -> bool {
    debug_say(w, id, "I currently have no orders.");
    let c = mx::combatant(w, id);
    let valid = is_valid_combatant(w, id, c);
    mx::set_warmode(w, id, valid);
    // A standing order is resumed through ResumePersistentOrder, never re-derived here.
    movement::walk_random_idle(w, id);
    true
}

fn do_order_come(w: &mut World, id: MobileId) -> bool {
    if super::base::check_herding(w, id) {
        debug_say(w, id, "I am being herded.");
        return true;
    }
    let Some(master) = control_master(w, id).filter(|&m| !mx::deleted(w, m)) else {
        return true;
    };
    movement::walk_mobile_range(w, id, master, 1, 1, 2);
    if distance_sqrt(w, id, master) <= 2.0 {
        control::set_control_order_raw(w, id, OrderType::Stay);
    }
    true
}

fn do_order_follow(w: &mut World, id: MobileId) -> bool {
    if super::base::check_herding(w, id) {
        debug_say(w, id, "I am being herded.");
        return true;
    }
    let target = read(w, id, None, |d| d.control_target);
    match target.filter(|&t| !mx::deleted(w, t) && t != id) {
        Some(t) => follow_target(w, id, t),
        None => {
            debug_say(w, id, "I have no one to follow.");
            control::set_control_order_raw(w, id, OrderType::None);
        }
    }
    true
}

fn follow_target(w: &mut World, id: MobileId, target: MobileId) {
    let rp = read(w, id, 0, |d| d.range_perception);
    let current = distance_sqrt(w, id, target) as i32;
    if current > rp {
        debug_say(w, id, "Master is missing. Staying put.");
        return;
    }
    debug_say(
        w,
        id,
        &format!("I am ordered to follow {}.", name(w, target)),
    );
    if current > 1 {
        movement::walk_mobile_range(w, id, target, 1, 1, 2);
    }
}

pub(crate) fn do_order_guard(w: &mut World, id: MobileId) -> bool {
    let master = control_master(w, id);
    let dead = read(w, id, false, |d| d.is_dead_pet);
    let Some(master) = master.filter(|&m| !mx::deleted(w, m) && !dead) else {
        return true;
    };
    match find_guard_target(w, id) {
        Some(c) => {
            debug_say(w, id, &format!("Attacking target: {}", name(w, c)));
            // Engage without leaving the Guard order so tags, recall handling, and retargeting persist.
            mx::set_combatant(w, id, Some(c));
            crate::creature::edit(w, id, |d| d.focus_mob = Some(c));
            set_action(w, id, ActionType::Combat);
            let class = ai(w, id).expect("ai").class;
            (class.think)(w, id);
        }
        None => {
            debug_say(w, id, &format!("Guarding my master, {}.", name(w, master)));
            // Stand down; a stale Warmode would skew the return pace.
            crate::creature::edit(w, id, |d| d.focus_mob = None);
            mx::set_warmode(w, id, false);
            mx::set_combatant(w, id, None);
            let distance = distance_sqrt(w, id, master) as i32;
            // Alert either way; FollowMoveSpeed caps the steps of the return itself.
            crate::creature::speeds::set_current_speed_to_active(w, id);
            if distance > super::base::GUARD_RANGE {
                movement::walk_mobile_range(w, id, master, 1, 1, super::base::GUARD_RANGE);
            } else {
                movement::walk_random(w, id, 3, 1, 1);
            }
        }
    }
    true
}

pub(crate) fn do_order_attack(w: &mut World, id: MobileId) -> bool {
    if read(w, id, false, |d| d.is_dead_pet) {
        return true;
    }
    let target = read(w, id, None, |d| d.control_target);
    if is_invalid_control_target(w, id, target) {
        handle_invalid_control_target(w, id);
    } else {
        let t = target.expect("valid");
        debug_say(w, id, &format!("Attacking target: {}", name(w, t)));
        // OnAggressiveAction can swap Combatant; the commanded target wins.
        if mx::combatant(w, id) != Some(t) {
            mx::set_combatant(w, id, Some(t));
        }
        let class = ai(w, id).expect("ai").class;
        (class.think)(w, id);
    }
    true
}

fn handle_invalid_control_target(w: &mut World, id: MobileId) {
    debug_say(w, id, "Target is either dead, hidden, or out of range.");
    resume_persistent_order(w, id);
    // A resumed Guard engages through its own scan; other fallbacks chain an explicit Attack.
    let (order, mode) = read(w, id, (OrderType::None, FightMode::None), |d| {
        (d.control_order, d.fight_mode)
    });
    if order == OrderType::Guard || !matches!(mode, FightMode::Closest | FightMode::Aggressor) {
        return;
    }
    if let Some(next) = find_guard_target(w, id) {
        creature_issue_order(w, id, OrderType::Attack, None, Some(next));
        debug_say(
            w,
            id,
            &format!("{} is still hostile! Engaging...", name(w, next)),
        );
        let class = ai(w, id).expect("ai").class;
        (class.think)(w, id);
    }
}

/// IsGuardAlly: uncontrolled summons (energy vortex, blade spirits) turn on their caster, so a guard still
/// defends against them.
pub fn is_guard_ally(w: &World, id: MobileId, target: MobileId) -> bool {
    let master = control_master(w, id);
    target == id
        || master.is_some_and(|m| {
            target == m || control_master(w, target) == Some(m) && cd(w, target).is_some()
        })
}

/// FindGuardTarget: the aggressor closest to the master; the current combatant is kept unless a strictly closer
/// one exists. Never mutates order state.
pub fn find_guard_target(w: &mut World, id: MobileId) -> Option<MobileId> {
    let master = control_master(w, id);
    let anchor = master.unwrap_or(id);
    let current = mx::combatant(w, id);
    let rp = read(w, id, 0, |d| d.range_perception);
    let mut best =
        current.filter(|&c| !is_guard_ally(w, id, c) && is_valid_combatant(w, id, Some(c)));
    let mut best_dist = best.map_or(f64::MAX, |b| distance_sqrt(w, b, anchor));
    for aggr in mx::mobiles_in_range(w, id, rp) {
        let hostile_to_us = mx::combatant(w, aggr) == Some(id)
            || master.is_some_and(|m| mx::combatant(w, aggr) == Some(m));
        if Some(aggr) == best
            || is_guard_ally(w, id, aggr)
            || w.mobile(aggr)
                .is_some_and(|m| (m.ty.is_dead_bonded_pet)(w, aggr))
            || !mx::alive(w, aggr)
            || !hostile_to_us
        {
            continue;
        }
        let dist = distance_sqrt(w, aggr, anchor);
        if dist < best_dist && can_see(w, id, aggr) && mx::in_los(w, id, aggr) {
            best = Some(aggr);
            best_dist = dist;
        }
    }
    if let Some(m) = master {
        let attackers: Vec<MobileId> = w
            .mobile(m)
            .map(|x| x.aggressors.iter().map(|a| a.attacker).collect())
            .unwrap_or_default();
        for aggressor in attackers {
            if Some(aggressor) == best
                || mx::deleted(w, aggressor)
                || is_guard_ally(w, id, aggressor)
                || !mx::alive(w, aggressor)
                || w.mobile(aggressor)
                    .is_some_and(|x| (x.ty.is_dead_bonded_pet)(w, aggressor))
                || !mx::in_range(w, id, aggressor, rp)
            {
                continue;
            }
            let dist = distance_sqrt(w, aggressor, anchor);
            if dist < best_dist && can_see(w, id, aggressor) && mx::in_los(w, id, aggressor) {
                best = Some(aggressor);
                best_dist = dist;
            }
        }
    }
    best
}

fn can_see(w: &mut World, id: MobileId, m: MobileId) -> bool {
    w.mobile(id)
        .map(|x| x.ty)
        .is_some_and(|x| (x.can_see_mobile)(w, id, m))
}

pub(crate) fn do_order_stay(w: &mut World, id: MobileId) -> bool {
    if super::base::check_herding(w, id) {
        debug_say(w, id, "I am being herded.");
    } else {
        debug_say(w, id, "I have been ordered to stay.");
    }
    // Hold position at the post (Home). Stand still when there; only walk back if displaced (e.g. after chasing a
    // kill). No idle shuffle.
    let home = read(w, id, Point3D::ZERO, |d| d.home);
    if home != Point3D::ZERO && mx::location(w, id) != home {
        let d = muo_server::mobile::part2::effects::get_direction_to(w, id, home.x, home.y, false);
        movement::do_move(w, id, d, false);
    }
    true
}
