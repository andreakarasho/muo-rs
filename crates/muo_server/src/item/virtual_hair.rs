//! Hair and facial hair are virtual items, sent as an
//! equip update (0x2E) and removed with 0x1D.

use muo_primitives::Serial;

use crate::ids::MobileId;
use crate::layer::Layer;
use crate::net_state::NetStateId;
use crate::packets::Pk;
use crate::world::World;

pub const EQUIP_UPDATE_PACKET_LENGTH: usize = 15;
pub const REMOVE_PACKET_LENGTH: usize = 5;

/// `CreateHairEquipUpdatePacket`.
pub fn create_hair_equip_update_packet(
    w: &World,
    m: MobileId,
    hair_serial: u32,
    item_id: i32,
    hue: i32,
    layer: Layer,
) -> Vec<u8> {
    let (serial, solid) = w.mobile(m).map_or((Serial::ZERO, -1), |mob| {
        (mob.serial, mob.solid_hue_override)
    });
    let mut pk = Pk::new(EQUIP_UPDATE_PACKET_LENGTH);
    pk.u8(0x2E); // Packet ID
    pk.u32(hair_serial);
    pk.i16(item_id as i16);
    pk.u8(0);
    pk.u8(layer.0);
    pk.serial(serial);
    pk.i16(if solid >= 0 { solid } else { hue } as i16);
    pk.finish()
}

/// `SendHairEquipUpdatePacket`.
pub fn send_hair_equip_update_packet(
    w: &mut World,
    ns: NetStateId,
    m: MobileId,
    hair_serial: u32,
    item_id: i32,
    hue: i32,
    layer: Layer,
) {
    if w.cannot_send_packets(ns) {
        return;
    }
    let bytes = create_hair_equip_update_packet(w, m, hair_serial, item_id, hue, layer);
    w.send(ns, &bytes);
}

/// `CreateRemoveHairPacket`.
pub fn create_remove_hair_packet(hair_serial: u32) -> Vec<u8> {
    let mut pk = Pk::new(REMOVE_PACKET_LENGTH);
    pk.u8(0x1D); // Packet ID
    pk.u32(hair_serial);
    pk.finish()
}

/// `SendRemoveHairPacket`.
pub fn send_remove_hair_packet(w: &mut World, ns: NetStateId, hair_serial: u32) {
    if w.cannot_send_packets(ns) {
        return;
    }
    w.send(ns, &create_remove_hair_packet(hair_serial));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn remove_packet_bytes() {
        assert_eq!(
            create_remove_hair_packet(0x4000_0001),
            [0x1D, 0x40, 0, 0, 1]
        );
    }

    // Virtual hair packet tests.
    use crate::mobile::MOBILE_BASE;
    use crate::mobile::part1::props_b;
    use crate::mobile::part2::stats::default_mobile_init;
    use crate::net_state::{NetOut, NetStateData};

    fn hair_mobile(w: &mut World) -> MobileId {
        let m = w.new_mobile(&MOBILE_BASE);
        default_mobile_init(w, m);
        m
    }

    #[test]
    fn send_virtual_hair_update() {
        let mut w = World::new();
        let m = hair_mobile(&mut w);
        // HairHue is ignored while there is no hair item
        props_b::set_hair_hue(&mut w, m, 0x1000);
        props_b::set_hair_item_id(&mut w, m, 0x2000);
        let hair = props_b::hair_serial(&mut w, m).value();
        let serial = w.mobile(m).unwrap().serial.value();

        let ns = NetStateId(1);
        w.net_states.insert(ns, NetStateData::default());
        send_hair_equip_update_packet(&mut w, ns, m, hair, 0x2000, 0, Layer::Hair);
        let Some(NetOut::Send(_, bytes)) = w.net_out.pop() else {
            panic!("nothing sent")
        };
        let mut want = vec![0x2E];
        want.extend(hair.to_be_bytes());
        want.extend([0x20, 0x00, 0, Layer::Hair.0]);
        want.extend(serial.to_be_bytes());
        want.extend([0, 0]);
        assert_eq!(bytes, want);
        assert_eq!(bytes.len(), EQUIP_UPDATE_PACKET_LENGTH);
    }

    #[test]
    fn remove_hair_uses_equipped_serial() {
        let mut w = World::new();
        let m = hair_mobile(&mut w);
        props_b::set_hair_item_id(&mut w, m, 0x2000);
        props_b::set_hair_hue(&mut w, m, 0x1000);
        let equipped = props_b::hair_serial(&mut w, m);
        assert_ne!(equipped, Serial::ZERO);

        props_b::set_hair_item_id(&mut w, m, 0); // remove

        // the serial survives removal so the client can remove the right entity
        assert_eq!(props_b::hair_serial(&mut w, m), equipped);
        // the hue is cleared with the item id
        assert_eq!(w.mobile(m).unwrap().hair_hue, 0);
    }

    #[test]
    fn remove_facial_hair_uses_equipped_serial() {
        let mut w = World::new();
        let m = hair_mobile(&mut w);
        props_b::set_facial_hair_item_id(&mut w, m, 0x2040);
        props_b::set_facial_hair_hue(&mut w, m, 0x1000);
        let equipped = props_b::facial_hair_serial(&mut w, m);
        assert_ne!(equipped, Serial::ZERO);

        props_b::set_facial_hair_item_id(&mut w, m, 0);

        assert_eq!(props_b::facial_hair_serial(&mut w, m), equipped);
        assert_eq!(w.mobile(m).unwrap().facial_hair_hue, 0);
    }

    #[test]
    fn send_remove_virtual_hair() {
        let mut w = World::new();
        let m = hair_mobile(&mut w);
        props_b::set_hair_item_id(&mut w, m, 0x2000);
        let hair = props_b::hair_serial(&mut w, m).value();
        let ns = NetStateId(1);
        w.net_states.insert(ns, NetStateData::default());
        send_remove_hair_packet(&mut w, ns, hair);
        let Some(NetOut::Send(_, bytes)) = w.net_out.pop() else {
            panic!("nothing sent")
        };
        let mut want = vec![0x1D];
        want.extend(hair.to_be_bytes());
        assert_eq!(bytes, want);
    }
}
