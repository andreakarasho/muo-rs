//! Vendor buy packets.

use muo_primitives::Serial;

use super::Pk;
use crate::ids::MobileId;
use crate::layer::Layer;
use crate::net_state::{NetCaps, NetStateId};
use crate::world::World;

// shortcut: the type lives with its packet instead of BaseVendor; the same fields and bytes.
#[derive(Clone, Debug)]
pub struct BuyItemState {
    pub price: i32,
    pub my_serial: Serial,
    pub container_serial: Serial,
    pub item_id: i32,
    pub amount: i32,
    pub hue: i32,
    pub description: Option<String>,
}

impl BuyItemState {
    /// `new BuyItemState(name, cont, serial, price, amount, itemID, hue)`.
    pub fn new(
        name: Option<&str>,
        cont: Serial,
        serial: Serial,
        price: i32,
        amount: i32,
        item_id: i32,
        hue: i32,
    ) -> Self {
        BuyItemState {
            price,
            my_serial: serial,
            container_serial: cont,
            item_id,
            amount,
            hue,
            description: name.map(str::to_string),
        }
    }
}

pub fn create_vendor_buy_content(caps: &NetCaps, list: &[BuyItemState]) -> Vec<u8> {
    let grid = caps.container_grid_lines();
    let length = 5 + list.len() * if grid { 20 } else { 19 };
    let mut p = Pk::new(length);
    p.u8(0x3C); // Packet ID
    p.u16(length as u16);
    p.i16(list.len() as i16);

    for i in (0..list.len()).rev() {
        let bis = &list[i];
        p.serial(bis.my_serial);
        p.u16(bis.item_id as u16);
        p.u8(0); // itemID offset
        p.u16(bis.amount as u16);
        p.i16((i + 1) as i16); // x
        p.i16(1); // y
        if grid {
            p.u8(0); // Grid Location?
        }
        p.serial(bis.container_serial);
        p.u16(bis.hue as u16);
    }
    p.finish()
}

pub fn send_vendor_buy_content(w: &mut World, ns: NetStateId, list: &[BuyItemState]) {
    if w.cannot_send_packets(ns) {
        return;
    }
    let Some(c) = w.net_state(ns).map(|d| d.caps.clone()) else {
        return;
    };
    w.send(ns, &create_vendor_buy_content(&c, list));
}

pub fn create_display_buy_list(caps: &NetCaps, vendor: Serial) -> Vec<u8> {
    let hs = caps.high_seas();
    let mut p = Pk::new(if hs { 9 } else { 7 });
    p.u8(0x24); // Packet ID
    p.serial(vendor);
    p.i16(0x30); // Vendor Buy Window
    if hs {
        p.i16(0);
    }
    p.finish()
}

pub fn send_display_buy_list(w: &mut World, ns: NetStateId, vendor: Serial) {
    if w.cannot_send_packets(ns) {
        return;
    }
    let Some(c) = w.net_state(ns).map(|d| d.caps.clone()) else {
        return;
    };
    w.send(ns, &create_display_buy_list(&c, vendor));
}

/// `vendor.FindItemOnLayer<Container>(Layer.ShopBuy)?.Serial`: the first live item on the layer, if it is a Container.
fn shop_buy_container(w: &World, vendor: MobileId) -> Option<Serial> {
    let m = w.mobile(vendor)?;
    let item = m
        .items
        .iter()
        .filter_map(|id| w.item(*id))
        .find(|i| !i.deleted() && i.layer == Layer::ShopBuy)?;
    item.ty
        .is(&crate::item::container::CONTAINER)
        .then_some(item.serial)
}

pub fn create_vendor_buy_list(w: &World, vendor: MobileId, list: &[BuyItemState]) -> Vec<u8> {
    // `6 + list[i].Description?.Length ?? 0` parses as `(6 + len?) ?? 0`: a null description adds 0, not 6.
    let mut length = 8;
    for bis in list {
        if let Some(d) = &bis.description {
            length += 6 + d.encode_utf16().count();
        }
    }

    let mut p = Pk::new(length);
    p.u8(0x74); // Packet ID
    p.u16(length as u16);
    p.serial(shop_buy_container(w, vendor).unwrap_or(Serial::MINUS_ONE));
    p.u8(list.len() as u8);

    for bis in list {
        p.i32(bis.price);
        let desc = bis.description.as_deref().unwrap_or("");
        p.u8((desc.encode_utf16().count() + 1) as u8);
        p.write_latin1_null(desc).ok();
    }
    p.finish()
}

pub fn send_vendor_buy_list(
    w: &mut World,
    ns: NetStateId,
    vendor: MobileId,
    list: &[BuyItemState],
) {
    if w.cannot_send_packets(ns) {
        return;
    }
    let b = create_vendor_buy_list(w, vendor, list);
    w.send(ns, &b);
}

pub fn create_end_vendor_buy(vendor: Serial) -> Vec<u8> {
    let mut p = Pk::new(8);
    p.u8(0x3B); // Packet ID
    p.u16(8);
    p.serial(vendor);
    p.u8(0); // Buy count
    p.finish()
}

pub fn send_end_vendor_buy(w: &mut World, ns: NetStateId, vendor: Serial) {
    if w.cannot_send_packets(ns) {
        return;
    }
    w.send(ns, &create_end_vendor_buy(vendor));
}

#[cfg(test)]
mod tests {
    use super::*;
    use muo_net::protocol_changes::ProtocolChanges;

    fn caps(pc: ProtocolChanges) -> NetCaps {
        NetCaps {
            protocol_changes: pc,
            ..Default::default()
        }
    }

    fn states() -> Vec<BuyItemState> {
        let c = Serial::new(0x4000_0001);
        vec![
            BuyItemState::new(
                Some("First Item"),
                c,
                Serial::new(0x4000_0002),
                10,
                1,
                0x01,
                0,
            ),
            BuyItemState::new(
                Some("Second Item"),
                c,
                Serial::new(0x4000_0003),
                20,
                2,
                0x0A,
                0,
            ),
            BuyItemState::new(
                Some("Third Item"),
                c,
                Serial::new(0x4000_0004),
                30,
                10,
                0x0F,
                0,
            ),
        ]
    }

    // Legacy VendorBuyContent: entries written last to first, x = index + 1, y = 1.
    #[test]
    fn vendor_buy_content() {
        let list = states();
        for grid in [false, true] {
            let pc = if grid {
                ProtocolChanges::CONTAINER_GRID_LINES
            } else {
                ProtocolChanges::empty()
            };
            let mut e = vec![0x3C];
            e.extend((5 + 3 * if grid { 20u16 } else { 19 }).to_be_bytes());
            e.extend(3u16.to_be_bytes());
            for i in (0..3).rev() {
                e.extend(list[i].my_serial.value().to_be_bytes());
                e.extend((list[i].item_id as u16).to_be_bytes());
                e.push(0);
                e.extend((list[i].amount as u16).to_be_bytes());
                e.extend((i as u16 + 1).to_be_bytes());
                e.extend(1u16.to_be_bytes());
                if grid {
                    e.push(0);
                }
                e.extend(list[i].container_serial.value().to_be_bytes());
                e.extend([0, 0]);
            }
            assert_eq!(create_vendor_buy_content(&caps(pc), &list), e);
        }
    }

    #[test]
    fn display_buy_list() {
        let v = Serial::new(1);
        assert_eq!(
            create_display_buy_list(&caps(ProtocolChanges::empty()), v),
            [0x24, 0, 0, 0, 1, 0, 0x30]
        );
        assert_eq!(
            create_display_buy_list(&caps(ProtocolChanges::HIGH_SEAS), v),
            [0x24, 0, 0, 0, 1, 0, 0x30, 0, 0]
        );
    }

    // Legacy VendorBuyList: shop container (or -1), count, per entry price and "len+1, text, 0".
    #[test]
    fn vendor_buy_list() {
        let mut w = World::new();
        let vendor = w.new_mobile(&crate::mobile::mobile_type::MOBILE_BASE);
        let list = states();
        let b = create_vendor_buy_list(&w, vendor, &list);

        let mut e = vec![0x74, 0, 0, 0xFF, 0xFF, 0xFF, 0xFF, 3];
        for s in &list {
            e.extend((s.price as u32).to_be_bytes());
            let d = s.description.as_deref().unwrap();
            e.push(d.len() as u8 + 1);
            e.extend(d.bytes());
            e.push(0);
        }
        let len = (e.len() as u16).to_be_bytes();
        e[1..3].copy_from_slice(&len);
        assert_eq!(b, e);
    }

    // `6 + Description?.Length ?? 0` adds 0 for a null description: the length field is 6 short of the bytes.
    #[test]
    fn vendor_buy_list_null_description_quirk() {
        let mut w = World::new();
        let vendor = w.new_mobile(&crate::mobile::mobile_type::MOBILE_BASE);
        let list = [BuyItemState::new(
            None,
            Serial::new(1),
            Serial::new(2),
            5,
            1,
            1,
            0,
        )];
        let b = create_vendor_buy_list(&w, vendor, &list);
        assert_eq!(u16::from_be_bytes([b[1], b[2]]), 8);
        assert_eq!(b.len(), 8 + 4 + 2);
    }

    #[test]
    fn end_vendor_buy() {
        assert_eq!(
            create_end_vendor_buy(Serial::new(0x1024)),
            [0x3B, 0, 8, 0, 0, 0x10, 0x24, 0]
        );
    }
}
