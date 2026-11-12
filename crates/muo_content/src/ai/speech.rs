//! The pet command parser (`OnSpeech`) and the pet login checks (`PetLogin`).

use muo_primitives::dotnet::InvariantCase;
use muo_primitives::{DateTime, MessageType, Point3D};
use muo_server::events::SpeechEventArgs;
use muo_server::ids::MobileId;
use muo_server::mobile::AccessLevel;
use muo_server::mobile::part2::messages;
use muo_server::world::World;

use super::{ai, ai_mut};
use crate::creature::control::{self, control_master};
use crate::creature::data::{cd, read};
use crate::creature::enums::*;
use crate::creature::vtable::ctype;
use crate::hooks::{hooks, pet_player_hooks, rnd};
use crate::mx;

/// `WasNamed(speech)`: the speech starts with the creature's name.
pub fn was_named(w: &World, id: MobileId, speech: &str) -> bool {
    muo_server::mobile::base::name(w, id)
        .is_some_and(|n| !n.is_empty() && speech.lower_inv().starts_with(&n.lower_inv()))
}

/// `GetFirstKeyword(e, keywords)`: the first listed keyword the speech carries, else 0.
fn first_keyword(e: &SpeechEventArgs, keywords: &[i32]) -> i32 {
    keywords
        .iter()
        .copied()
        .find(|&k| e.has_keyword(k))
        .unwrap_or(0)
}

/// HandlesOnSpeech
pub fn handles_on_speech(w: &World, id: MobileId, from: MobileId) -> bool {
    if mx::access_level(w, from) >= AccessLevel::GameMaster {
        return true;
    }
    let c = ctype(w, id);
    let commandable = c.is_some_and(|c| (c.commandable)(w, id));
    let controlled = read(w, id, false, |d| d.controlled);
    if mx::alive(w, from)
        && controlled
        && commandable
        && (control_master(w, id) == Some(from)
            || c.is_some_and(|c| (c.is_pet_friend)(w, id, from)))
    {
        return true;
    }
    mx::alive(w, from)
        && mx::in_range_point(w, id, mx::location(w, from), 3)
        && c.is_some_and(|c| (c.is_human_in_town)(w, id))
}

/// OnSpeech
pub fn on_speech(w: &mut World, id: MobileId, e: &mut SpeechEventArgs) {
    let c = ctype(w, id).expect("creature");
    if was_named(w, id, &e.speech)
        && mx::alive(w, e.mobile)
        && mx::in_range(w, e.mobile, id, 3)
        && (c.is_human_in_town)(w, id)
        && (handle_move_command(w, id, e)
            || handle_time_command(w, id, e)
            || handle_train_command(w, id, e))
    {
        return;
    }
    // Staff first, so "<name> obey" reaches a controlled pet.
    if mx::access_level(w, e.mobile) >= AccessLevel::GameMaster && handle_gm_commands(w, id, e) {
        return;
    }
    let (controlled, commandable) = (read(w, id, false, |d| d.controlled), (c.commandable)(w, id));
    if controlled && commandable {
        // Exactly one handler per utterance: named addresses this pet, "all" every pet in range.
        if was_named(w, id, &e.speech) {
            named_on_speech_pet(w, id, e);
        } else {
            all_on_speech_pet(w, id, e);
        }
    }
}

/// `_lastOrder` throttle shared by the move and time requests: true while still inside 5 seconds.
fn throttled(w: &mut World, id: MobileId) -> bool {
    let now = w.now;
    let last = super::ai(w, id).map_or(DateTime::MIN, |a| a.last_order);
    if (now.ticks() - last.ticks()) < 5 * 10_000_000 {
        return true;
    }
    if let Some(a) = super::ai_mut(w, id) {
        a.last_order = now;
    }
    false
}

fn handle_move_command(w: &mut World, id: MobileId, e: &SpeechEventArgs) -> bool {
    if !e.has_keyword(0x9D) {
        return false; // *move*
    }
    if throttled(w, id) {
        return true;
    }
    let loc = mx::location(w, id);
    let (new_x, new_y, new_z) = (
        loc.x + rnd::min_max(w, -1, 1),
        loc.y + rnd::min_max(w, -1, 1),
        loc.z,
    );
    let fits = mx::map(w, id).is_some_and(|map| {
        muo_server::map::fit::can_fit(w, map, new_x, new_y, new_z, 16, false, false, true)
    });
    if fits {
        messages::public_overhead_message_number(
            w,
            id,
            MessageType::REGULAR,
            0x3B2,
            501516,
            "",
            false,
        ); // Excuse me?
        muo_server::mobile::base::set_location(w, id, Point3D::new(new_x, new_y, new_z), false);
    } else {
        // You're standing too close, go away.
        messages::public_overhead_message_number(
            w,
            id,
            MessageType::REGULAR,
            0x3B2,
            501487,
            "",
            false,
        );
    }
    true
}

/// `Clock.GetTime(map, x, y, out generalNumber, ..)`.
fn clock_general_number(w: &World, id: MobileId) -> i32 {
    // WorldStart is 1997-09-01; a UO minute is 5 seconds.
    let world_start = DateTime::from_ymd(1997, 9, 1).unwrap_or(DateTime::MIN);
    let secs = (w.now.ticks() - world_start.ticks()) as f64 / 10_000_000.0;
    let mut total_minutes = (secs / 5.0) as i32;
    if let Some(map) = mx::map(w, id).and_then(|m| w.maps.get(m)) {
        total_minutes += map.map_index * 320;
    }
    total_minutes += mx::location(w, id).x / 16;
    let hours = total_minutes / 60 % 24;
    match hours {
        20.. => 1042957, // It's late at night
        16.. => 1042956, // It's early in the evening
        13.. => 1042955, // It's the afternoon
        12.. => 1042954, // It's around noon
        8.. => 1042953,  // It's late in the morning
        4.. => 1042952,  // It's early in the morning
        1.. => 1042951,  // It's the middle of the night
        _ => 1042950,    // 'Tis the witching hour. 12 Midnight.
    }
}

fn handle_time_command(w: &mut World, id: MobileId, e: &SpeechEventArgs) -> bool {
    if !e.has_keyword(0x9E) {
        return false; // *time*
    }
    if throttled(w, id) {
        return true;
    }
    let n = clock_general_number(w, id);
    messages::public_overhead_message_number(w, id, MessageType::REGULAR, 0x3B2, n, "", false);
    true
}

fn handle_train_command(w: &mut World, id: MobileId, e: &SpeechEventArgs) -> bool {
    if !e.has_keyword(0x6C) {
        return false; // *train*
    }
    crate::creature::teach::handle_training(w, id, e.mobile);
    true
}

fn handle_gm_commands(w: &mut World, id: MobileId, e: &SpeechEventArgs) -> bool {
    // "all obey" is for wild creatures; a controlled pet must be named.
    let controlled = read(w, id, false, |d| d.controlled);
    if !crate::creature::pets::find_my_name(w, id, &e.speech, !controlled)
        || !e.speech.lower_inv().contains("obey")
    {
        return false;
    }
    control::set_control_master(w, id, Some(e.mobile));
    true
}

fn owner_and_friend(w: &World, id: MobileId, from: MobileId) -> Option<bool> {
    if !mx::in_range_point(w, id, mx::location(w, from), 14) {
        return None;
    }
    let is_owner = control_master(w, id) == Some(from);
    let c = ctype(w, id).expect("creature");
    let is_friend = !is_owner && (c.is_pet_friend)(w, id, from);
    (is_owner || is_friend).then_some(is_owner)
}

fn all_on_speech_pet(w: &mut World, id: MobileId, e: &SpeechEventArgs) {
    let from = e.mobile;
    let Some(is_owner) = owner_and_friend(w, id, from) else {
        return;
    };
    match first_keyword(
        e,
        &[
            0x164, 0x165, 0x166, 0x167, 0x168, 0x169, 0x16B, 0x16C, 0x170,
        ],
    ) {
        0x164 => handle_come(w, id, from, is_owner),
        0x165 => begin_pick_target(w, id, from, OrderType::Follow),
        0x166 | 0x16B => handle_guard(w, id, from, is_owner),
        0x167 => handle_stay_stop_follow(w, id, from, OrderType::Stop, None),
        0x168 | 0x169 => handle_attack(w, id, from, is_owner),
        0x16C => handle_stay_stop_follow(w, id, from, OrderType::Follow, Some(from)),
        0x170 => handle_stay_stop_follow(w, id, from, OrderType::Stay, None),
        _ => {}
    }
}

fn named_on_speech_pet(w: &mut World, id: MobileId, e: &SpeechEventArgs) {
    let from = e.mobile;
    let Some(is_owner) = owner_and_friend(w, id, from) else {
        return;
    };
    match first_keyword(
        e,
        &[
            0x155, 0x156, 0x15A, 0x15B, 0x15C, 0x15D, 0x15E, 0x161, 0x163, 0x16D, 0x16E, 0x16F,
        ],
    ) {
        0x155 => handle_come(w, id, from, is_owner),
        0x156 => {
            let (dead, summoned) = read(w, id, (false, false), |d| (d.is_dead_pet, d.summoned));
            if is_owner && !dead && !summoned && check_control_chance(w, id, from) {
                control::issue_order(w, id, OrderType::Drop, Some(from), None);
            }
        }
        0x15A => begin_pick_target(w, id, from, OrderType::Follow),
        0x15B => {
            if is_owner && check_control_chance(w, id, from) {
                if read(w, id, false, |d| d.summoned)
                    || crate::factory::mobile_is(w, id, "Server.Mobiles.GrizzledMare")
                {
                    mx::msg(w, from, 1005481); // Summoned creatures are loyal only to their summoners.
                    return;
                }
                if muo_server::mobile::part1::props_b::has_trade(w, from) {
                    mx::msg(w, from, 1070947); // You cannot friend a pet with a trade pending
                    return;
                }
                begin_pick_target(w, id, from, OrderType::Friend);
            }
        }
        0x15C => handle_guard(w, id, from, is_owner),
        0x15D | 0x15E => handle_attack(w, id, from, is_owner),
        0x161 => handle_stay_stop_follow(w, id, from, OrderType::Stop, None),
        0x163 => handle_stay_stop_follow(w, id, from, OrderType::Follow, Some(from)),
        0x16D => {
            if !is_owner {
                return;
            }
            // No control roll: see InternalEntry.HandleReleaseOrder.
            let c = ctype(w, id).expect("creature");
            if !(c.can_be_controlled_by)(w, id, from) {
                return;
            }
            if read(w, id, false, |d| d.summoned) {
                control::issue_order(w, id, OrderType::Release, Some(from), None);
                return;
            }
            // `from.SendGump(new ConfirmReleaseGump(from, Mobile))` (muo_gumps sets the hook)
            if let Some(f) = hooks(w).and_then(|h| h.confirm_release_gump) {
                f(w, from, id);
            }
        }
        0x16E => {
            let dead = read(w, id, false, |d| d.is_dead_pet);
            if is_owner && !dead && check_control_chance(w, id, from) {
                if read(w, id, false, |d| d.summoned)
                    || crate::factory::mobile_is(w, id, "Server.Mobiles.GrizzledMare")
                {
                    mx::msg(w, from, 1005487); // You cannot transfer ownership of a summoned creature.
                    return;
                }
                if muo_server::mobile::part1::props_b::has_trade(w, from) {
                    mx::msg(w, from, 1010507); // You cannot transfer a pet with a trade pending
                    return;
                }
                begin_pick_target(w, id, from, OrderType::Transfer);
            }
        }
        0x16F => handle_stay_stop_follow(w, id, from, OrderType::Stay, None),
        _ => {}
    }
}

fn check_control_chance(w: &mut World, id: MobileId, from: MobileId) -> bool {
    (ctype(w, id).expect("creature").check_control_chance)(w, id, from)
}

fn handle_come(w: &mut World, id: MobileId, from: MobileId, is_owner: bool) {
    if is_owner && check_control_chance(w, id, from) {
        control::issue_order(w, id, OrderType::Come, Some(from), None);
    }
}

fn handle_guard(w: &mut World, id: MobileId, from: MobileId, is_owner: bool) {
    if is_owner && check_control_chance(w, id, from) {
        control::issue_order(w, id, OrderType::Guard, Some(from), None);
    }
}

fn handle_stay_stop_follow(
    w: &mut World,
    id: MobileId,
    from: MobileId,
    order: OrderType,
    target: Option<MobileId>,
) {
    if check_control_chance(w, id, from) {
        control::issue_order(w, id, order, Some(from), target);
    }
}

fn handle_attack(w: &mut World, id: MobileId, from: MobileId, is_owner: bool) {
    if is_owner {
        begin_pick_target(w, id, from, OrderType::Attack);
    }
}

/// The pets waiting on one `AIControlMobileTarget` (`_list`) and its `Order`, by target id.
#[derive(Default)]
pub struct PickTargets(
    indexmap::IndexMap<muo_server::target::TargetId, (OrderType, Vec<MobileId>)>,
);

/// BeginPickTarget: the player picks a target, then `EndPickTarget` issues the order.
pub fn begin_pick_target(w: &mut World, id: MobileId, from: MobileId, order: OrderType) {
    use muo_server::target::{self, Target, TargetFlags, Targeted};
    if !is_valid_pick(w, id, from, order) {
        return;
    }
    let current = w.mobile(from).and_then(|m| m.target);
    if current.is_none() {
        match order {
            OrderType::Transfer => mx::msg(w, from, 502038), // Click on the person to transfer ownership to.
            OrderType::Friend => mx::msg(w, from, 502020), // Click on the player whom you wish to make a co-owner.
            OrderType::Unfriend => mx::msg(w, from, 1070948), // Click on the player whom you wish to remove as a co-owner.
            _ => {}
        }
        // `new AIControlMobileTarget(this, order)`: range -1, harmful for an Attack order.
        let flags = if order == OrderType::Attack {
            TargetFlags::HARMFUL
        } else {
            TargetFlags::NONE
        };
        let mut t = Target::new(w, -1, false, flags);
        let tid = t.id;
        t.handlers.on_target = Some(std::rc::Rc::new(
            move |w: &mut World, from: MobileId, targeted: Targeted| {
                let list = w.resources.get_mut::<PickTargets>().0.shift_remove(&tid);
                if let (
                    Some((order, ais)),
                    Targeted::Entity(muo_server::ids::EntityId::Mobile(m)),
                ) = (list, targeted)
                {
                    for ai in ais {
                        end_pick_target(w, ai, from, m, order);
                    }
                }
            },
        ));
        target::add_target(w, t);
        let live: Vec<_> = w.targets.keys().copied().collect();
        let picks = w.resources.get_mut::<PickTargets>();
        picks.0.retain(|k, _| live.contains(k)); // targets that were canceled never ran their handler
        picks.0.insert(tid, (order, vec![id]));
        target::set_mobile_target(w, from, Some(tid));
    } else if let Some(t) = current
        && let Some((o, list)) = w.resources.get_mut::<PickTargets>().0.get_mut(&t)
        && *o == order
        && !list.contains(&id)
    {
        list.push(id);
    }
}

fn is_valid_pick(w: &mut World, id: MobileId, from: MobileId, order: OrderType) -> bool {
    let controlled = read(w, id, false, |d| d.controlled);
    if mx::deleted(w, id)
        || !controlled
        || !mx::in_range(w, from, id, 14)
        || mx::map(w, from) != mx::map(w, id)
    {
        return false;
    }
    if !muo_server::mobile::part2::stats::check_alive(w, from, true) {
        return false;
    }
    let is_owner = control_master(w, id) == Some(from);
    let c = ctype(w, id).expect("creature");
    let is_friend = !is_owner && (c.is_pet_friend)(w, id, from);
    if !is_owner && !is_friend {
        return false;
    }
    !(is_friend && !super::orders::is_friend_order(order))
}

/// EndPickTarget: called when the picked target arrives.
pub fn end_pick_target(
    w: &mut World,
    id: MobileId,
    from: MobileId,
    target: MobileId,
    order: OrderType,
) {
    if !is_valid_pick(w, id, from, order)
        || order == OrderType::Attack && !can_attack_target(w, id, from, target)
    {
        return;
    }
    if check_control_chance(w, id, from) {
        control::issue_order(w, id, order, Some(from), Some(target));
    }
}

fn can_attack_target(w: &mut World, id: MobileId, from: MobileId, target: MobileId) -> bool {
    if w.mobile(target).is_some_and(|m| m.hidden) {
        return false;
    }
    let c = ctype(w, id).expect("creature");
    if cd(w, target).is_some()
        && ctype(w, target).is_some_and(|t| (t.is_scary_to_pets)(w, target))
        && (c.is_scared_of_scary_things)(w, id)
    {
        mx::say_to(w, id, from, "Your pet refuses to attack this creature!");
        return false;
    }
    const RED: [&str; 5] = [
        "RedSolenInfiltratorQueen",
        "RedSolenInfiltratorWarrior",
        "RedSolenQueen",
        "RedSolenWarrior",
        "RedSolenWorker",
    ];
    const BLACK: [&str; 5] = [
        "BlackSolenInfiltratorQueen",
        "BlackSolenInfiltratorWarrior",
        "BlackSolenQueen",
        "BlackSolenWarrior",
        "BlackSolenWorker",
    ];
    let is_any = |w: &World, names: &[&str]| {
        names
            .iter()
            .any(|n| crate::factory::mobile_is(w, target, &format!("Server.Mobiles.{n}")))
    };
    if super::super::creature::solen::check_red_friendship(w, from) && is_any(w, &RED)
        || super::super::creature::solen::check_black_friendship(w, from) && is_any(w, &BLACK)
    {
        mx::msg(w, from, 1063106); // You can not force your pet to attack a creature you are protected from.
        return false;
    }
    if crate::factory::mobile_is(w, target, "Server.Factions.BaseFactionGuard") {
        mx::say_to(w, id, from, "Your pet refuses to attack the guard.");
        return false;
    }
    true
}

// ---- PetLogin -----------------------------------------------------------------------------------------------

/// Within this many tiles of the master we assume the pet was following; otherwise staying.
const FOLLOW_RANGE: i32 = 12;

/// `PetLoginHandler.DeriveFollowerOrders`: `PersistentOrder` is runtime-only; ControlOrder and Home are saved. A pet
/// saved mid-transient gets Follow near the master, else Stay. Call on `PlayerLoginEvent`.
pub fn derive_follower_orders(w: &mut World, master: MobileId) {
    let Some(followers) = pet_player_hooks(w)
        .and_then(|h| h.all_followers)
        .map(|f| f(w, master))
    else {
        return;
    };
    for follower in followers {
        let ok = cd(w, follower).is_some_and(|d| d.controlled)
            && !mx::deleted(w, follower)
            && control_master(w, follower) == Some(master)
            && ai(w, follower).is_some_and(|a| a.persistent_order == OrderType::None);
        if !ok {
            continue;
        }
        let restored = read(w, follower, OrderType::None, |d| d.control_order);
        if matches!(
            restored,
            OrderType::Stay | OrderType::Follow | OrderType::Guard
        ) {
            super::orders::restore_persistent_order(w, follower, restored);
            continue;
        }
        let near = mx::map(w, follower) == mx::map(w, master)
            && crate::creature::misc::distance_sqrt(w, follower, master) <= FOLLOW_RANGE as f64;
        let derived = if near {
            OrderType::Follow
        } else {
            OrderType::Stay
        };
        // ControlTarget first: SetPersistentOrder records it as the Follow target, and a mid-Attack save still holds
        // the victim.
        crate::creature::edit(w, follower, |d| {
            d.control_target = if near { Some(master) } else { None }
        });
        super::orders::set_persistent_order_pub(w, follower, derived);
        let t = read(w, follower, None, |d| d.control_target);
        control::set_control_order(w, follower, derived, None, true, t);
    }
}

#[allow(dead_code)]
fn _unused(w: &mut World, id: MobileId) {
    let _ = (
        ai_mut(w, id).is_some(),
        MessageType::REGULAR,
        messages::send_message,
    );
}
