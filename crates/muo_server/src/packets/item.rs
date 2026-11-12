//! Item packets.

use super::Pk;
use crate::ids::ItemId;
use crate::item::Item;
use crate::net_state::NetStateId;
use crate::world::World;

/// `item is BaseMulti`.
fn is_base_multi(item: &Item) -> bool {
    item.ty.is(&crate::item::multi::BASE_MULTI)
}

/// `CreateWorldItem` (0x1A).
pub fn create_world_item(w: &World, id: ItemId) -> Option<Vec<u8>> {
    let item = w.item(id)?;
    let item_id = if is_base_multi(item) {
        item.item_id | 0x4000
    } else {
        item.item_id & 0x3FFF
    };
    let amount = item.amount;
    let has_amount = amount != 0;
    let serial = if has_amount {
        item.serial.value() | 0x8000_0000
    } else {
        item.serial.value() & 0x7FFF_FFFF
    };
    let loc = item.location;
    let hue = item.hue;
    let flags = (item.ty.get_packet_flags)(w, id);
    let direction = i32::from(item.direction.bits());
    let has_direction = direction != 0;
    let has_hue = hue != 0;
    let has_flags = flags != 0;
    let x = if has_direction {
        loc.x | 0x8000
    } else {
        loc.x & 0x7FFF
    };
    let y =
        (loc.y & 0x3FFF) | if has_hue { 0x8000 } else { 0 } | if has_flags { 0x4000 } else { 0 };
    let length = 14
        + if has_amount { 2 } else { 0 }
        + if has_direction { 1 } else { 0 }
        + if has_hue { 2 } else { 0 }
        + if has_flags { 1 } else { 0 };

    let mut p = Pk::new(length);
    p.u8(0x1A); // Packet ID
    p.u16(length as u16);
    p.u32(serial);
    p.u16(item_id as u16);
    if has_amount {
        p.u16(amount as u16);
    }
    p.u16(x as u16);
    p.u16(y as u16);
    if has_direction {
        p.u8(direction as u8);
    }
    p.i8(loc.z as i8);
    if has_hue {
        p.u16(hue as u16);
    }
    if has_flags {
        p.u8(flags as u8);
    }
    Some(p.finish())
}

/// `OutgoingEntityPackets.CreateWorldEntity` (0xF3) of an item: `entity.rs` owns it.
pub fn create_world_entity_item(w: &World, id: ItemId, is_hs: bool) -> Option<Vec<u8>> {
    w.item(id)?;
    Some(super::entity::create_world_entity(
        w,
        crate::ids::EntityId::Item(id),
        is_hs,
    ))
}

/// `SendWorldItem`.
pub fn send_world_item(w: &mut World, ns: NetStateId, item: ItemId) {
    if w.cannot_send_packets(ns) {
        return;
    }
    let Some(c) = w.net_state(ns).map(|d| d.caps.clone()) else {
        return;
    };
    let bytes = if c.stygian_abyss() {
        create_world_entity_item(w, item, c.high_seas())
    } else {
        create_world_item(w, item)
    };
    if let Some(b) = bytes {
        w.send(ns, &b);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::item::ITEM_BASE;
    use muo_primitives::Direction;

    // The legacy WorldItem packet: 0x1A, length, serial|0x80000000 when it has an amount, itemID & 0x3FFF,
    // amount, x (|0x8000 with a direction), y (|0x8000 hue, |0x4000 flags), direction, z, hue, flags.
    #[test]
    fn world_item_plain() {
        let mut w = World::new();
        let id = w.new_item(&ITEM_BASE);
        {
            let it = w.item_mut(id).unwrap();
            it.item_id = 0x0EED;
            it.amount = 1;
            it.location = muo_primitives::Point3D {
                x: 100,
                y: 200,
                z: -5,
            };
        }
        let b = create_world_item(&w, id).unwrap();
        // a new item is Movable: flags 0x20, so y carries 0x4000
        let mut e = vec![0x1A, 0, 17];
        e.extend((id.0.value() | 0x8000_0000).to_be_bytes());
        e.extend([0x0E, 0xED, 0, 1, 0, 100, 0x40, 200, 0xFB, 0x20]);
        assert_eq!(b, e);
    }

    #[test]
    fn world_item_hue_direction_no_amount() {
        let mut w = World::new();
        let id = w.new_item(&ITEM_BASE);
        {
            let it = w.item_mut(id).unwrap();
            it.item_id = 0xFFFF;
            it.amount = 0;
            it.hue = 0x21;
            it.direction = Direction::from_bits_retain(3);
            it.location = muo_primitives::Point3D {
                x: 0x1234,
                y: 0x4321,
                z: 7,
            };
        }
        let b = create_world_item(&w, id).unwrap();
        let mut e = vec![0x1A, 0, 18];
        e.extend((id.0.value() & 0x7FFF_FFFF).to_be_bytes());
        e.extend([0x3F, 0xFF]);
        e.extend((0x1234u16 | 0x8000).to_be_bytes());
        e.extend(((0x4321u16 & 0x3FFF) | 0xC000).to_be_bytes());
        e.extend([3, 7, 0, 0x21, 0x20]); // Movable
        assert_eq!(b, e);
    }

    #[test]
    fn world_entity_item() {
        let mut w = World::new();
        let id = w.new_item(&ITEM_BASE);
        {
            let it = w.item_mut(id).unwrap();
            it.item_id = 0x1234;
            it.amount = 2;
            it.hue = 5;
            it.location = muo_primitives::Point3D { x: 10, y: 20, z: 3 };
        }
        let mut e = vec![0xF3, 0, 1, 0];
        e.extend(id.0.value().to_be_bytes());
        e.extend([0x12, 0x34, 0, 0, 2, 0, 2, 0, 10, 0, 20, 3, 0, 0, 5, 0x20]); // Movable
        assert_eq!(create_world_entity_item(&w, id, false).unwrap(), e);
        e.extend([0, 0]);
        assert_eq!(create_world_entity_item(&w, id, true).unwrap(), e);
    }
}
