//! Small helpers every hand-written Items/Special module uses.

use muo_primitives::MessageType;
use muo_server::World;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::{ItemType, deps, ops};
use muo_server::mobile::part2::messages;

/// `item.IsChildOf(from.Backpack)`.
pub fn in_pack(w: &mut World, id: ItemId, from: MobileId) -> bool {
    let pack = muo_server::mobile::part1::props_b::backpack(w, from).map(EntityId::Item);
    ops::is_child_of(w, id, pack)
}

/// `from.SendLocalizedMessage(number)`.
pub fn msg(w: &mut World, to: MobileId, number: i32) {
    deps::send_localized_message(w, to, number);
}

/// `from.SendMessage(text)`.
pub fn text(w: &mut World, to: MobileId, s: &str) {
    deps::send_message(w, to, s);
}

/// `from.LocalOverheadMessage(MessageType.Regular, hue, number)`.
pub fn overhead(w: &mut World, to: MobileId, hue: i32, number: i32) {
    messages::local_overhead_message_number(w, to, MessageType::REGULAR, hue, number, "");
}

/// `item.Delete()`.
pub fn delete(w: &mut World, id: ItemId) {
    muo_server::item::base::delete(w, id);
}

/// `item.Deleted`.
pub fn deleted(w: &World, id: ItemId) -> bool {
    w.item(id).is_none_or(|i| i.deleted())
}

/// `is_a(item, ty)`: `item is T`.
pub fn is_a(w: &World, id: ItemId, ty: &'static ItemType) -> bool {
    w.item(id).is_some_and(|i| i.ty.is(ty))
}

/// `from.NetState`.
pub fn net_state(w: &World, m: MobileId) -> Option<muo_server::net_state::NetStateId> {
    w.mobile(m).and_then(|x| x.net_state)
}

/// `m.AccessLevel`.
pub fn access_level(w: &World, m: MobileId) -> muo_server::mobile::AccessLevel {
    muo_content::mx::access_level(w, m)
}

/// `Account.Username` of a mobile's account, if it has one.
pub fn account_username(w: &World, m: MobileId) -> Option<String> {
    let id = w.mobile(m)?.account?;
    muo_content::accounting::account(w, id).map(|a| a.username.clone())
}

/// `RewardSystem.GetRewardYearLabel(item, args)`: the cliloc to add for a reward item; None when no reward system is
/// installed (see `clothing::special::RewardHooks`).
pub fn reward_year_label(w: &World, id: ItemId) -> Option<i32> {
    w.resources
        .get::<crate::clothing::special::RewardHooks>()
        .and_then(|h| h.year_label)
        .and_then(|f| f(w, id))
}

/// `RewardSystem.CheckIsUsableBy(m, item, args)`: true when no reward system is installed.
pub fn reward_usable_by(w: &mut World, m: MobileId, id: ItemId) -> bool {
    match w
        .resources
        .get::<crate::clothing::special::RewardHooks>()
        .and_then(|h| h.check_is_usable_by)
    {
        Some(f) => f(w, m, id),
        None => true,
    }
}

/// `from.SendLocalizedMessage(number, args)`.
pub fn msg_args(w: &mut World, to: MobileId, number: i32, args: &str) {
    muo_content::mx::msg_args(w, to, number, args);
}
