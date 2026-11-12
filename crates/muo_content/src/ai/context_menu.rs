//! The pet context menu: the "Command: Kill/Follow/..." entries of a pet.

use std::rc::Rc;

use muo_server::context_menu_entry::ContextMenuEntry;
use muo_server::ids::{EntityId, MobileId};
use muo_server::world::World;

use super::orders::{is_dead_pet_order, is_friend_order};
use crate::creature::control::{self, control_master};
use crate::creature::data::{cd, read};
use crate::creature::enums::*;
use crate::creature::vtable::ctype;
use crate::mx;

/// `BaseAI.GetContextMenuEntries(from, ref list)`.
pub fn get_context_menu_entries(
    w: &mut World,
    id: MobileId,
    from: MobileId,
    list: &mut Vec<ContextMenuEntry>,
) {
    if !mx::alive(w, from)
        || !read(w, id, false, |d| d.controlled)
        || !mx::in_range(w, from, id, 16)
    {
        return;
    }
    let dead = read(w, id, false, |d| d.is_dead_pet);
    let c = ctype(w, id).expect("creature");
    if control_master(w, id) == Some(from) {
        list.push(entry(3006111, OrderType::Attack, !dead)); // Command: Kill
        list.push(entry(3006108, OrderType::Follow, true)); // Command: Follow
        list.push(entry(3006107, OrderType::Guard, !dead)); // Command: Guard
        list.push(entry(3006112, OrderType::Stop, true)); // Command: Stop
        list.push(entry(3006114, OrderType::Stay, true)); // Command: Stay
        if (c.can_drop)(w, id) {
            list.push(entry(3006109, OrderType::Drop, !dead)); // Command: Drop
        }
        list.push(entry(3006098, OrderType::Rename, true)); // Rename
        if !read(w, id, false, |d| d.summoned)
            && !crate::factory::mobile_is(w, id, "Server.Mobiles.GrizzledMare")
        {
            list.push(entry(3006110, OrderType::Friend, true)); // Add Friend
            list.push(entry(3006099, OrderType::Unfriend, true)); // Remove Friend
            list.push(entry(3006113, OrderType::Transfer, !dead)); // Transfer
        }
        list.push(entry(3006118, OrderType::Release, true)); // Release
    } else if (c.is_pet_friend)(w, id, from) {
        list.push(entry(3006108, OrderType::Follow, true)); // Command: Follow
        list.push(entry(3006112, OrderType::Stop, !dead)); // Command: Stop
        list.push(entry(3006114, OrderType::Stay, true)); // Command: Stay
    }
}

/// `new InternalEntry(number, 14, order, enabled)`.
pub(crate) fn entry(number: i32, order: OrderType, enabled: bool) -> ContextMenuEntry {
    let mut e = ContextMenuEntry::new(number, 14);
    e.enabled = enabled;
    e.on_click = Some(Rc::new(move |w, from, target| {
        on_click(w, from, target, order)
    }));
    e
}

fn on_click(w: &mut World, from: MobileId, target: EntityId, order: OrderType) {
    let EntityId::Mobile(bc) = target else { return };
    if !muo_server::mobile::part2::stats::check_alive(w, from, true)
        || mx::deleted(w, bc)
        || cd(w, bc).is_none_or(|d| !d.controlled)
    {
        return;
    }
    if read(w, bc, false, |d| d.is_dead_pet) && is_dead_pet_order(order) {
        return;
    }
    let c = ctype(w, bc).expect("creature");
    let is_owner = control_master(w, bc) == Some(from);
    let is_friend = !is_owner && (c.is_pet_friend)(w, bc, from);
    if !is_owner && !is_friend || is_friend && !is_friend_order(order) {
        return;
    }
    match order {
        OrderType::Follow
        | OrderType::Attack
        | OrderType::Transfer
        | OrderType::Friend
        | OrderType::Unfriend => {
            if matches!(order, OrderType::Transfer | OrderType::Friend)
                && muo_server::mobile::part1::props_b::has_trade(w, from)
            {
                // 1010507: You cannot transfer a pet with a trade pending; 1070947: ... friend a pet ...
                crate::mx::msg(
                    w,
                    from,
                    if order == OrderType::Transfer {
                        1010507
                    } else {
                        1070947
                    },
                );
                return;
            }
            super::speech::begin_pick_target(w, bc, from, order);
        }
        OrderType::Release => {
            // No roll: a refused one would only drain loyalty toward the involuntary release the drain performs
            // anyway. Whoever can command the creature may dismiss it.
            if !(c.can_be_controlled_by)(w, bc, from) {
                return;
            }
            if read(w, bc, false, |d| d.summoned) {
                control::issue_order(w, bc, OrderType::Release, Some(from), None);
                return;
            }
            if let Some(f) = crate::hooks::hooks(w).and_then(|h| h.confirm_release_gump) {
                f(w, from, bc);
            }
        }
        _ => {
            if (c.check_control_chance)(w, bc, from) {
                control::issue_order(w, bc, order, Some(from), None);
            }
        }
    }
}
