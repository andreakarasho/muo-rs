//! Boat packets: the 0xF6 move boat packet of High Seas clients and the 0xF7 display of
//! everything on a boat.

use muo_server::World;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::net_state::NetStateId;
use muo_server::packets::container_builder::PacketContainerBuilder;
use muo_server::packets::entity::create_world_entity;

/// `GetMoveBoatHSPacketLength(entityCount)`.
pub fn move_boat_hs_packet_length(entity_count: usize) -> usize {
    18 + entity_count.min(0xFFFF) * 10
}

fn serial_xyz(w: &World, e: EntityId) -> Option<(u32, i32, i32, i32)> {
    match e {
        EntityId::Item(i) => w
            .item(i)
            .map(|x| (x.serial.value(), x.location.x, x.location.y, x.location.z)),
        EntityId::Mobile(m) => w
            .mobile(m)
            .map(|x| (x.serial.value(), x.location.x, x.location.y, x.location.z)),
    }
}

/// `CreateMoveBoatHS(...)`: the packet for `boat` moving `(x_offset, y_offset)` with `entities` aboard.
pub fn create_move_boat_hs(
    w: &World,
    boat: ItemId,
    entities: &[EntityId],
    d: i32,
    speed: i32,
    x_offset: i32,
    y_offset: i32,
) -> Vec<u8> {
    let count = entities.len().min(0xFFFF);
    let Some(b) = w.item(boat) else {
        return Vec::new();
    };
    let mut out = Vec::with_capacity(move_boat_hs_packet_length(count));
    out.push(0xF6);
    out.extend_from_slice(&[0, 0]); // Length
    out.extend_from_slice(&b.serial.value().to_be_bytes());
    out.push(speed as u8);
    out.push(d as u8);
    out.push(super::facing(w, boat) as u8);
    out.extend_from_slice(&((b.location.x + x_offset) as i16).to_be_bytes());
    out.extend_from_slice(&((b.location.y + y_offset) as i16).to_be_bytes());
    out.extend_from_slice(&(b.location.z as i16).to_be_bytes());
    out.extend_from_slice(&(count as i16).to_be_bytes());
    for &e in entities.iter().take(count) {
        let (serial, x, y, z) = serial_xyz(w, e).unwrap_or((0, 0, 0, 0));
        out.extend_from_slice(&serial.to_be_bytes());
        out.extend_from_slice(&((x + x_offset) as i16).to_be_bytes());
        out.extend_from_slice(&((y + y_offset) as i16).to_be_bytes());
        out.extend_from_slice(&(z as i16).to_be_bytes());
    }
    let len = out.len() as u16;
    out[1..3].copy_from_slice(&len.to_be_bytes());
    out
}

/// `SendMoveBoatHS(boat, entities, d, speed, xOffset, yOffset)`.
#[allow(clippy::too_many_arguments)] // SendMoveBoatHS(boat, entities, d, speed, xOffset, yOffset)
pub fn send_move_boat_hs(
    w: &mut World,
    ns: NetStateId,
    boat: ItemId,
    entities: &[EntityId],
    d: i32,
    speed: i32,
    x_offset: i32,
    y_offset: i32,
) {
    if w.cannot_send_packets(ns) {
        return;
    }
    let p = create_move_boat_hs(w, boat, entities, d, speed, x_offset, y_offset);
    w.send(ns, &p);
}

/// The clients in range of the boat that get the packet before the locations change (`Move`).
pub fn send_move_boat_hs_to_clients(
    w: &mut World,
    boat: ItemId,
    entities: &[EntityId],
    d: i32,
    client_speed: i32,
    x_offset: i32,
    y_offset: i32,
) {
    let Some(ty) = w.item(boat).map(|i| i.ty) else {
        return;
    };
    let (loc, map) = match w.item(boat) {
        Some(i) => (i.location, i.map),
        None => return,
    };
    let range = (ty.get_max_update_range)(w, boat);
    // The packet is built once and reused
    let packet = create_move_boat_hs(w, boat, entities, d, client_speed, x_offset, y_offset);
    let clients: Vec<NetStateId> =
        muo_server::map::enumerate::clients_in_range(w, map, loc.x, loc.y, range).collect();
    for ns in clients {
        let Some(m) = w
            .net_state(ns)
            .filter(|n| n.caps.high_seas())
            .and_then(|n| n.mobile)
        else {
            continue;
        };
        if muo_server::item::deps::can_see(w, m, boat)
            && muo_server::item::deps::mobile_in_range(w, m, loc, (ty.get_update_range)(w, boat, m))
            && !w.cannot_send_packets(ns)
        {
            w.send(ns, &packet);
        }
    }
}

/// `SendDisplayBoatHS(beholder, boat)`.
pub fn send_display_boat_hs(w: &mut World, ns: NetStateId, beholder: MobileId, boat: ItemId) {
    if !w.net_state(ns).is_some_and(|d| d.caps.high_seas()) || w.cannot_send_packets(ns) {
        return;
    }
    let mut builder = PacketContainerBuilder::new();
    for e in super::get_moving_entities(w, boat, true) {
        let can_see = match e {
            EntityId::Item(i) => muo_server::mobile::base::can_see_item(w, beholder, i),
            EntityId::Mobile(m) => muo_server::mobile::base::can_see_mobile(w, beholder, m),
        };
        if !can_see {
            continue;
        }
        builder.push(&create_world_entity(w, e, true));
    }
    w.send(ns, &builder.finish());
}
