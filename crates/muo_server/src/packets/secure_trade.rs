//! Secure trade packets.

use super::Pk;
use crate::ids::{ItemId, MobileId};
use crate::net_state::{NetCaps, NetStateId};
use crate::world::World;

/// `TradeFlag`.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum TradeFlag {
    Display,
    Close,
    Update,
    UpdateGold,
    UpdateLedger,
}

pub fn create_display_secure_trade(
    them: MobileId,
    first: ItemId,
    second: ItemId,
    name: Option<&str>,
) -> Vec<u8> {
    let mut p = Pk::new(47);
    p.u8(0x6F); // Packet ID
    p.u16(47); // Length
    p.u8(TradeFlag::Display as u8);
    p.serial(them.0);
    p.serial(first.0);
    p.serial(second.0);
    p.bool(true);
    p.write_latin1_fixed(name.unwrap_or(""), 30).ok();
    p.finish()
}

pub fn send_display_secure_trade(
    w: &mut World,
    ns: NetStateId,
    them: MobileId,
    first: ItemId,
    second: ItemId,
    name: Option<&str>,
) {
    if w.cannot_send_packets(ns) {
        return;
    }
    w.send(ns, &create_display_secure_trade(them, first, second, name));
}

pub fn create_close_secure_trade(cont: ItemId) -> Vec<u8> {
    let mut p = Pk::new(17);
    p.u8(0x6F); // Packet ID
    p.u16(17); // Length
    p.u8(TradeFlag::Close as u8);
    p.serial(cont.0);
    p.i32(0);
    p.i32(0);
    p.bool(false);
    p.finish()
}

pub fn send_close_secure_trade(w: &mut World, ns: NetStateId, cont: ItemId) {
    if w.cannot_send_packets(ns) {
        return;
    }
    w.send(ns, &create_close_secure_trade(cont));
}

pub fn create_update_secure_trade(
    cont: ItemId,
    flag: TradeFlag,
    first: i32,
    second: i32,
) -> Vec<u8> {
    let mut p = Pk::new(17);
    p.u8(0x6F); // Packet ID
    p.u16(17); // Length
    p.u8(flag as u8);
    p.serial(cont.0);
    p.i32(first);
    p.i32(second);
    p.bool(false);
    p.finish()
}

/// `SendUpdateSecureTrade(cont, first, second)`.
pub fn send_update_secure_trade(
    w: &mut World,
    ns: NetStateId,
    cont: ItemId,
    first: bool,
    second: bool,
) {
    send_update_secure_trade_flag(
        w,
        ns,
        cont,
        TradeFlag::Update,
        i32::from(first),
        i32::from(second),
    );
}

/// `SendUpdateSecureTrade(cont, flag, first, second)`.
pub fn send_update_secure_trade_flag(
    w: &mut World,
    ns: NetStateId,
    cont: ItemId,
    flag: TradeFlag,
    first: i32,
    second: i32,
) {
    if w.cannot_send_packets(ns) {
        return;
    }
    w.send(ns, &create_update_secure_trade(cont, flag, first, second));
}

pub fn create_secure_trade_equip(
    w: &World,
    caps: &NetCaps,
    item: ItemId,
    m: MobileId,
) -> Option<Vec<u8>> {
    let it = w.item(item)?;
    let grid = caps.container_grid_lines();
    let mut p = Pk::new(if grid { 21 } else { 20 });
    p.u8(0x25); // Packet ID
    p.serial(it.serial);
    p.i16(it.item_id as i16);
    p.u8(0);
    p.i16(it.amount as i16);
    p.i16(it.location.x as i16);
    p.i16(it.location.y as i16);
    if grid {
        p.u8(0);
    }
    p.serial(m.0);
    p.i16(it.hue as i16);
    Some(p.finish())
}

pub fn send_secure_trade_equip(w: &mut World, ns: NetStateId, item: ItemId, m: MobileId) {
    if w.cannot_send_packets(ns) {
        return;
    }
    let Some(c) = w.net_state(ns).map(|d| d.caps.clone()) else {
        return;
    };
    if let Some(b) = create_secure_trade_equip(w, &c, item, m) {
        w.send(ns, &b);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::item::ITEM_BASE;
    use muo_net::protocol_changes::ProtocolChanges;
    use muo_primitives::Serial;

    fn caps(pc: ProtocolChanges) -> NetCaps {
        NetCaps {
            protocol_changes: pc,
            ..Default::default()
        }
    }

    // Legacy DisplaySecureTrade: 0x6F, length, flag, three serials, true, 30-byte latin1 name.
    #[test]
    fn display_secure_trade() {
        let b = create_display_secure_trade(
            MobileId(Serial::new(0x1)),
            ItemId(Serial::new(0x4000_0001)),
            ItemId(Serial::new(0x4000_0002)),
            Some("Trader"),
        );
        let mut e = vec![0x6F, 0, 47, 0];
        e.extend([0, 0, 0, 1, 0x40, 0, 0, 1, 0x40, 0, 0, 2, 1]);
        let mut name = b"Trader".to_vec();
        name.resize(30, 0);
        e.extend(name);
        assert_eq!(b, e);
        // a null name is the empty string
        let b = create_display_secure_trade(
            MobileId(Serial::new(1)),
            ItemId(Serial::new(2)),
            ItemId(Serial::new(3)),
            None,
        );
        assert_eq!(b.len(), 47);
        assert!(b[17..].iter().all(|x| *x == 0));
    }

    #[test]
    fn close_secure_trade() {
        let b = create_close_secure_trade(ItemId(Serial::new(0x4000_0001)));
        assert_eq!(
            b,
            [0x6F, 0, 17, 1, 0x40, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0]
        );
    }

    #[test]
    fn update_secure_trade() {
        let c = ItemId(Serial::new(0x4000_0001));
        let b = create_update_secure_trade(c, TradeFlag::Update, 1, 0);
        assert_eq!(
            b,
            [0x6F, 0, 17, 2, 0x40, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0]
        );
        let b = create_update_secure_trade(c, TradeFlag::UpdateGold, 5, 258);
        assert_eq!(
            b,
            [0x6F, 0, 17, 3, 0x40, 0, 0, 1, 0, 0, 0, 5, 0, 0, 1, 2, 0]
        );
    }

    #[test]
    fn secure_trade_equip() {
        let mut w = World::new();
        let id = w.new_item(&ITEM_BASE);
        {
            let it = w.item_mut(id).unwrap();
            it.item_id = 0x1234;
            it.amount = 3;
            it.hue = 0x21;
        }
        let m = MobileId(Serial::new(0x77));
        let s = id.0.value().to_be_bytes();
        let mut e = vec![0x25];
        e.extend(s);
        e.extend([0x12, 0x34, 0, 0, 3, 0, 0, 0, 0]);
        let mut grid = e.clone();
        grid.push(0);
        e.extend([0, 0, 0, 0x77, 0, 0x21]);
        grid.extend([0, 0, 0, 0x77, 0, 0x21]);
        let b = create_secure_trade_equip(&w, &caps(ProtocolChanges::empty()), id, m).unwrap();
        assert_eq!(b, e);
        let b = create_secure_trade_equip(&w, &caps(ProtocolChanges::CONTAINER_GRID_LINES), id, m)
            .unwrap();
        assert_eq!(b, grid);
    }
}
