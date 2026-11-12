//! Vendor sell packets.

use muo_primitives::Serial;

use super::Pk;
use crate::ids::ItemId;
use crate::net_state::NetStateId;
use crate::world::World;

// shortcut: the type lives with its packet instead of BaseVendor; the same fields and bytes.
#[derive(Clone, Debug)]
pub struct SellItemState {
    pub item: ItemId,
    pub price: i32,
    pub name: Option<String>,
}

fn utf16_len(s: &str) -> usize {
    s.encode_utf16().count()
}

/// `set` is the `HashSet<SellItemState>` in its enumeration order (callers keep an insertion-ordered set).
pub fn create_vendor_sell_list(w: &World, vendor: Serial, set: &[SellItemState]) -> Vec<u8> {
    let mut max_length = 9;
    for sis in set {
        let item_name = w.item(sis.item).and_then(|i| i.name.as_deref());
        max_length += 14
            + utf16_len(item_name.unwrap_or("")).max(utf16_len(sis.name.as_deref().unwrap_or("")));
    }

    let mut p = Pk::new(max_length);
    p.u8(0x9E); // Packet ID
    p.zeros(2);
    p.serial(vendor);
    p.u16(set.len() as u16);

    for sis in set {
        let Some(item) = w.item(sis.item) else {
            continue;
        };
        p.serial(item.serial);
        p.u16(item.item_id as u16);
        p.u16(item.hue as u16);
        p.u16(item.amount as u16);
        p.u16(sis.price as u16);

        // (item.Name?.Trim()).DefaultIfNullOrEmpty(sis.Name ?? "")
        let trimmed = item
            .name
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty());
        let name = trimmed.unwrap_or_else(|| sis.name.as_deref().unwrap_or(""));
        p.u16(utf16_len(name) as u16);
        p.write_latin1(name).ok();
    }
    p.finish_with_length()
}

pub fn send_vendor_sell_list(w: &mut World, ns: NetStateId, vendor: Serial, set: &[SellItemState]) {
    if w.cannot_send_packets(ns) {
        return;
    }
    let b = create_vendor_sell_list(w, vendor, set);
    w.send(ns, &b);
}

pub fn create_end_vendor_sell(vendor: Serial) -> Vec<u8> {
    let mut p = Pk::new(8);
    p.u8(0x3B); // Packet ID
    p.u16(8);
    p.serial(vendor);
    p.u8(0);
    p.finish()
}

pub fn send_end_vendor_sell(w: &mut World, ns: NetStateId, vendor: Serial) {
    if w.cannot_send_packets(ns) {
        return;
    }
    w.send(ns, &create_end_vendor_sell(vendor));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::item::ITEM_BASE;

    // Legacy VendorSellList (0x9E): length, vendor, count, then per item serial, itemID, hue, amount, price,
    // name length, name.
    #[test]
    fn vendor_sell_list() {
        let mut w = World::new();
        let i1 = w.new_item(&ITEM_BASE);
        let i2 = w.new_item(&ITEM_BASE);
        let i3 = w.new_item(&ITEM_BASE);
        w.item_mut(i2).unwrap().name = Some("Second Item".into());
        let set = [
            SellItemState {
                item: i1,
                price: 100,
                name: Some("Item 1".into()),
            },
            SellItemState {
                item: i2,
                price: 100000,
                name: Some("Item 2".into()),
            },
            SellItemState {
                item: i3,
                price: 1,
                name: Some("Item 3".into()),
            },
        ];
        let vendor = Serial::new(0x1024);
        let b = create_vendor_sell_list(&w, vendor, &set);

        let mut e = vec![0x9E, 0, 0, 0, 0, 0x10, 0x24, 0, 3];
        for (s, n) in set.iter().zip(["Item 1", "Second Item", "Item 3"]) {
            e.extend(s.item.0.value().to_be_bytes());
            e.extend([0, 0, 0, 0, 0, 1]); // itemID 0, hue 0, amount 1
            e.extend((s.price as u16).to_be_bytes());
            e.extend((n.len() as u16).to_be_bytes());
            e.extend(n.bytes());
        }
        let len = (e.len() as u16).to_be_bytes();
        e[1..3].copy_from_slice(&len);
        assert_eq!(b, e);
    }

    #[test]
    fn blank_item_name_falls_back_to_state_name() {
        let mut w = World::new();
        let i = w.new_item(&ITEM_BASE);
        w.item_mut(i).unwrap().name = Some("   ".into());
        let set = [SellItemState {
            item: i,
            price: 1,
            name: None,
        }];
        let b = create_vendor_sell_list(&w, Serial::new(1), &set);
        assert_eq!(&b[b.len() - 2..], [0, 0]); // empty name
    }

    #[test]
    fn end_vendor_sell() {
        assert_eq!(
            create_end_vendor_sell(Serial::new(0x1024)),
            [0x3B, 0, 8, 0, 0, 0x10, 0x24, 0]
        );
    }
}
