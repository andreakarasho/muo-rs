//! Equipment packets.

use muo_primitives::Serial;

use super::Pk;
use crate::ids::{EntityId, ItemId};
use crate::net_state::NetStateId;
use crate::world::World;

pub struct EquipInfoAttribute {
    pub number: i32,
    pub charges: i32,
}

impl EquipInfoAttribute {
    /// `new EquipInfoAttribute(number, charges)`; the usual `charges` is -1.
    pub fn new(number: i32, charges: i32) -> Self {
        EquipInfoAttribute { number, charges }
    }
}

pub fn create_display_equipment_info(
    serial: Serial,
    number: i32,
    crafter_name: Option<&str>,
    unidentified: bool,
    attrs: &[EquipInfoAttribute],
) -> Vec<u8> {
    let crafter_name = crafter_name.unwrap_or("");
    let crafter_len = crafter_name.encode_utf16().count();

    let length = 17
        + if crafter_len > 0 { 6 + crafter_len } else { 0 }
        + if unidentified { 4 } else { 0 }
        + attrs.len() * 6;

    let mut pk = Pk::new(length);
    pk.u8(0xBF); // Packet ID
    pk.u16(length as u16);
    pk.u16(0x10); // Subpacket
    pk.serial(serial);
    pk.i32(number);

    if crafter_len > 0 {
        pk.i32(-3); // crafted by
        pk.u16(crafter_len as u16);
        let _ = pk.write_latin1(crafter_name);
    }

    if unidentified {
        pk.i32(-4);
    }

    for attr in attrs {
        pk.i32(attr.number);
        pk.i16(attr.charges as i16);
    }

    pk.i32(-1);
    pk.finish()
}

pub fn send_display_equipment_info(
    w: &mut World,
    ns: NetStateId,
    serial: Serial,
    number: i32,
    crafter_name: Option<&str>,
    unidentified: bool,
    attrs: &[EquipInfoAttribute],
) {
    if w.cannot_send_packets(ns) {
        return;
    }
    let bytes = create_display_equipment_info(serial, number, crafter_name, unidentified, attrs);
    w.send(ns, &bytes);
}

pub fn create_equip_update(w: &World, item: ItemId) -> Vec<u8> {
    let Some(it) = w.item(item) else {
        return Vec::new();
    };
    let mut hue = it.hue;

    let parent_serial = match it.parent {
        Some(EntityId::Mobile(p)) => {
            if let Some(pm) = w.mobile(p)
                && pm.solid_hue_override >= 0
            {
                hue = pm.solid_hue_override;
            }
            p.0
        }
        _ => {
            tracing::warn!("Warning: EquipUpdate on item with !(parent is Mobile)");
            Serial::ZERO
        }
    };

    let mut pk = Pk::new(15);
    pk.u8(0x2E); // Packet ID
    pk.serial(it.serial);
    pk.i16(it.item_id as i16);
    pk.u16(it.layer.0 as u16);
    pk.serial(parent_serial);
    pk.i16(hue as i16);
    pk.finish()
}

pub fn send_equip_update(w: &mut World, ns: NetStateId, item: ItemId) {
    if w.cannot_send_packets(ns) {
        return;
    }
    let bytes = create_equip_update(w, item);
    w.send(ns, &bytes);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::item::ITEM_BASE;
    use crate::layer::Layer;
    use crate::mobile::MOBILE_BASE;
    use crate::net_state::{NetOut, NetStateData};

    fn world_with_ns() -> (World, NetStateId) {
        let mut w = World::new();
        let ns = NetStateId(1);
        w.net_states.insert(ns, NetStateData::default());
        (w, ns)
    }

    fn sent(w: &mut World) -> Vec<u8> {
        match w.net_out.pop() {
            Some(NetOut::Send(_, b)) => b,
            _ => panic!("nothing sent"),
        }
    }

    /// The legacy `DisplayEquipmentInfo`, written out.
    fn legacy_display(
        serial: u32,
        number: i32,
        crafter: &str,
        unid: bool,
        attrs: &[(i32, i16)],
    ) -> Vec<u8> {
        let mut b = vec![0xBF, 0, 0, 0, 0x10];
        b.extend(serial.to_be_bytes());
        b.extend(number.to_be_bytes());
        if !crafter.is_empty() {
            b.extend((-3i32).to_be_bytes());
            b.extend((crafter.len() as u16).to_be_bytes());
            b.extend(crafter.bytes());
        }
        if unid {
            b.extend((-4i32).to_be_bytes());
        }
        for (n, c) in attrs {
            b.extend(n.to_be_bytes());
            b.extend(c.to_be_bytes());
        }
        b.extend((-1i32).to_be_bytes());
        let l = (b.len() as u16).to_be_bytes();
        b[1] = l[0];
        b[2] = l[1];
        b
    }

    #[test]
    fn display_equipment_info() {
        for (name, unid) in [
            (None, false),
            (Some("Some Crafter"), false),
            (Some(""), true),
        ] {
            let (mut w, ns) = world_with_ns();
            let attrs = [
                EquipInfoAttribute::new(500001, 1),
                EquipInfoAttribute::new(500002, 2),
                EquipInfoAttribute::new(500002, 3),
            ];
            send_display_equipment_info(
                &mut w,
                ns,
                Serial::new(0x4000_0001),
                500000,
                name,
                unid,
                &attrs,
            );
            let expected = legacy_display(
                0x4000_0001,
                500000,
                name.unwrap_or(""),
                unid,
                &[(500001, 1), (500002, 2), (500002, 3)],
            );
            assert_eq!(sent(&mut w), expected);
        }
    }

    #[test]
    fn equip_update() {
        let (mut w, ns) = world_with_ns();
        let m = w.new_mobile(&MOBILE_BASE);
        let i = w.new_item(&ITEM_BASE);
        {
            let it = w.item_mut(i).unwrap();
            it.parent = Some(EntityId::Mobile(m));
            it.item_id = 0x1F03;
            it.layer = Layer::Shirt;
            it.hue = 0x21;
        }
        send_equip_update(&mut w, ns, i);
        let it = w.item(i).unwrap();
        let mut e = vec![0x2E];
        e.extend(it.serial.value().to_be_bytes());
        e.extend([0x1F, 0x03, 0, 5]);
        e.extend(m.0.value().to_be_bytes());
        e.extend([0, 0x21]);
        assert_eq!(sent(&mut w), e);

        // SolidHueOverride wins
        w.mobile_mut(m).unwrap().solid_hue_override = 0x300;
        send_equip_update(&mut w, ns, i);
        let b = sent(&mut w);
        assert_eq!(&b[13..15], &[0x03, 0x00]);
    }
}
