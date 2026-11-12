//! Messages that come from an item. The `RawInterpolatedStringHandler`
//! overloads are the same functions taking a `format!` string.

use crate::ids::{ItemId, MobileId};
use crate::item::deps;
use crate::item::ops;
use crate::net_state::NetStateId;
use crate::packets::message::{self, AffixType, MessageType};
use crate::world::World;

/// `PublicVisibilityFilter.Allow(state)`: the connection's mobile can see the item and is in its update range.
fn public_visibility(
    id: ItemId,
    world_loc: muo_primitives::Point3D,
) -> impl Fn(&mut World, NetStateId) -> bool {
    move |w, ns| {
        let Some(m) = w.net_mobile(ns) else {
            return false;
        };
        let Some(ty) = w.item(id).map(|i| i.ty) else {
            return false;
        };
        deps::can_see(w, m, id)
            && deps::mobile_in_range(w, m, world_loc, (ty.get_update_range)(w, id, m))
    }
}

/// `PublicOverheadMessage(type, hue, ascii, text)`.
pub fn public_overhead_message(
    w: &mut World,
    id: ItemId,
    kind: MessageType,
    hue: i32,
    ascii: bool,
    text: &str,
) {
    let Some(i) = w.item(id) else { return };
    let (map, serial, graphic, ty) = (i.map, i.serial, i.item_id, i.ty);
    let world_loc = ops::get_world_location(w, id);
    let name = ops::name(w, id).unwrap_or_default();
    message::broadcast_message(
        w,
        map,
        world_loc,
        Some((ty.get_max_update_range)(w, id)),
        serial,
        graphic,
        kind,
        hue,
        3,
        ascii,
        Some("ENU"),
        &name,
        text,
        &public_visibility(id, world_loc),
    );
}

/// `PublicOverheadMessage(type, hue, number, args)`.
pub fn public_overhead_message_number(
    w: &mut World,
    id: ItemId,
    kind: MessageType,
    hue: i32,
    number: i32,
    args: &str,
) {
    let Some(i) = w.item(id) else { return };
    let (map, serial, graphic, ty) = (i.map, i.serial, i.item_id, i.ty);
    let world_loc = ops::get_world_location(w, id);
    let name = ops::name(w, id).unwrap_or_default();
    message::broadcast_message_localized(
        w,
        map,
        world_loc,
        Some((ty.get_max_update_range)(w, id)),
        serial,
        graphic,
        kind,
        hue,
        3,
        number,
        &name,
        args,
        &public_visibility(id, world_loc),
    );
}

/// `SendLocalizedMessageTo(to, number, args)`.
pub fn send_localized_message_to(w: &mut World, id: ItemId, to: MobileId, number: i32, args: &str) {
    if ops::is_deleted(w, id) || !deps::can_see(w, to, id) {
        return;
    }
    send_localized_hue(w, id, to, number, 0x3B2, args);
}

/// `SendLocalizedMessageTo(to, number, affixType, affix, args)`.
pub fn send_localized_message_to_affix(
    w: &mut World,
    id: ItemId,
    to: MobileId,
    number: i32,
    affix_type: AffixType,
    affix: &str,
    args: &str,
) {
    if ops::is_deleted(w, id) || !deps::can_see(w, to, id) {
        return;
    }
    let Some(ns) = deps::net_state_of(w, to) else {
        return;
    };
    let Some(i) = w.item(id) else { return };
    let (serial, graphic) = (i.serial, i.item_id);
    message::send_message_localized_affix(
        w,
        ns,
        serial,
        graphic,
        MessageType::REGULAR,
        0x3B2,
        3,
        number,
        "",
        affix_type,
        affix,
        args,
    );
}

/// `SendLocalizedMessageTo(to, number, hue, args)`: no visibility check.
pub fn send_localized_hue(
    w: &mut World,
    id: ItemId,
    to: MobileId,
    number: i32,
    hue: i32,
    args: &str,
) {
    let Some(ns) = deps::net_state_of(w, to) else {
        return;
    };
    let Some(i) = w.item(id) else { return };
    let (serial, graphic) = (i.serial, i.item_id);
    message::send_message_localized(
        w,
        ns,
        serial,
        graphic,
        MessageType::REGULAR,
        hue,
        3,
        number,
        "",
        args,
    );
}

/// `SendMessageTo(to, text, hue)`.
pub fn send_message_to(w: &mut World, id: ItemId, to: MobileId, text: &str, hue: i32) {
    let Some(ns) = deps::net_state_of(w, to) else {
        return;
    };
    let Some(i) = w.item(id) else { return };
    let (serial, graphic) = (i.serial, i.item_id);
    message::send_message(
        w,
        ns,
        serial,
        graphic,
        MessageType::REGULAR,
        hue,
        3,
        false,
        Some("ENU"),
        "",
        text,
    );
}
