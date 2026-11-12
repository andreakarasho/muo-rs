//! Entity packets (property list info and the like).

use muo_primitives::Serial;

use super::Pk;
use crate::ids::{EntityId, ItemId, MobileId};
use crate::item::item_type::ItemType;
use crate::net_state::NetStateId;
use crate::world::World;

pub const OPL_PACKET_LENGTH: usize = 9;
pub const REMOVE_ENTITY_LENGTH: usize = 5;
pub const MAX_WORLD_ENTITY_PACKET_LENGTH: usize = 26;

/// `CreateOPLInfo(buffer, serial, hash)`: 0xDC.
pub fn create_opl_info(serial: Serial, hash: i32) -> Vec<u8> {
    let mut pk = Pk::new(OPL_PACKET_LENGTH);
    pk.u8(0xDC).serial(serial).i32(hash);
    pk.finish()
}

/// `SendOPLInfo(serial, hash)`.
pub fn send_opl_info(w: &mut World, ns: NetStateId, serial: Serial, hash: i32) {
    if w.cannot_send_packets(ns) {
        return;
    }
    w.send(ns, &create_opl_info(serial, hash));
}

/// `SendOPLInfo(IObjectPropertyListEntity)`: `obj.PropertyList.Hash`. An entity whose list was never built has no
/// hash to send (the `PropertyList` getter builds it), so the caller must have built it.
pub fn send_opl_info_item(w: &mut World, ns: NetStateId, id: ItemId) {
    let hash = w
        .item(id)
        .and_then(|i| i.property_list.as_ref().map(|p| p.hash()));
    if let Some(hash) = hash {
        send_opl_info(w, ns, id.0, hash);
    }
}

/// `SendOPLInfo(IObjectPropertyListEntity)` for a mobile.
pub fn send_opl_info_mobile(w: &mut World, ns: NetStateId, id: MobileId) {
    let hash = w
        .mobile(id)
        .and_then(|m| m.property_list.as_ref().map(|p| p.hash()));
    if let Some(hash) = hash {
        send_opl_info(w, ns, id.0, hash);
    }
}

/// `CreateRemoveEntity`: 0x1D.
pub fn create_remove_entity(serial: Serial) -> Vec<u8> {
    let mut pk = Pk::new(REMOVE_ENTITY_LENGTH);
    pk.u8(0x1D).serial(serial);
    pk.finish()
}

/// `SendRemoveEntity`.
pub fn send_remove_entity(w: &mut World, ns: NetStateId, serial: Serial) {
    if w.cannot_send_packets(ns) {
        return;
    }
    w.send(ns, &create_remove_entity(serial));
}

fn is_base_multi(ty: &'static ItemType) -> bool {
    ty.is(&crate::item::multi::BASE_MULTI)
}

/// `CreateWorldEntity`: 0xF3. `is_hs` is `NetState.HighSeas`.
pub fn create_world_entity(w: &World, entity: EntityId, is_hs: bool) -> Vec<u8> {
    let mut pk = Pk::new(MAX_WORLD_ENTITY_PACKET_LENGTH);
    pk.u8(0xF3).i16(0x1); // packet id, command

    let mut ty = 0u8;
    let gfx: i32;
    let mut amount = 1i32;
    let hue: i32;
    let mut light = 0u8;
    let mut flags = 0i32;
    let (serial, loc) = match entity {
        EntityId::Item(id) => {
            let Some(item) = w.item(id) else {
                return Vec::new();
            };
            let mask = if is_hs { 0xFFFF } else { 0x7FFF };
            if is_base_multi(item.ty) {
                ty = 2;
                gfx = item.item_id & mask;
                hue = item.hue;
                amount = item.amount;
            } else {
                // type = 3 if is damageable
                gfx = item.item_id & mask;
                hue = item.hue;
                amount = item.amount;
                light = item.direction.bits();
                flags = (item.ty.get_packet_flags)(w, id);
            }
            (id.0, item.location)
        }
        EntityId::Mobile(id) => {
            let Some(m) = w.mobile(id) else {
                return Vec::new();
            };
            ty = 1;
            gfx = crate::mobile::part1::props_b::body(w, id).0;
            hue = (m.ty.hue)(w, id);
            flags = (m.ty.get_packet_flags)(w, id, true);
            (id.0, m.location)
        }
    };

    pk.u8(ty).serial(serial).u16(gfx as u16).u8(0);
    pk.i16(amount as i16).i16(amount as i16); // Min, Max
    pk.i16((loc.x & 0x7FFF) as i16)
        .i16((loc.y & 0x3FFF) as i16)
        .i8(loc.z as i8);
    pk.u8(light).i16(hue as i16).u8(flags as u8);
    if is_hs {
        pk.i16(0);
    }
    pk.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opl_info() {
        assert_eq!(
            create_opl_info(Serial::new(0x4000_0001), 0x4001_0203),
            [0xDC, 0x40, 0, 0, 1, 0x40, 1, 2, 3]
        );
    }

    #[test]
    fn remove_entity() {
        assert_eq!(
            create_remove_entity(Serial::new(0x0102_0304)),
            [0x1D, 1, 2, 3, 4]
        );
    }
}
