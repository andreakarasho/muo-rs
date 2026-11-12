//! Container packets.

use muo_primitives::Serial;

use super::Pk;
use crate::ids::{ItemId, MobileId};
use crate::net_state::{NetCaps, NetStateId};
use crate::world::World;

/// `Item.QuestItemHue`.
pub const QUEST_ITEM_HUE: i32 = 0x4EA;

// 0x3C's length and item-count fields are both 16-bit, so a container's contents can never
// legally fill more than this many bytes regardless of how many items it actually holds.
const MAX_CONTAINER_CONTENT_BYTES: usize = u16::MAX as usize;

fn caps(w: &World, ns: NetStateId) -> Option<NetCaps> {
    w.net_state(ns).map(|d| d.caps.clone())
}

/// `SendDisplaySpellbook`.
pub fn send_display_spellbook(w: &mut World, ns: NetStateId, book: Serial) {
    send_display_container(w, ns, book, -1);
}

/// `SendSpellbookContent`. `aos` is `Core.AOS` (not a `World` field yet).
pub fn send_spellbook_content(
    w: &mut World,
    ns: NetStateId,
    aos: bool,
    book: Serial,
    graphic: i32,
    offset: i32,
    content: u64,
) {
    if w.cannot_send_packets(ns) {
        return;
    }
    let new_spellbook = caps(w, ns).is_some_and(|c| c.new_spellbook());
    if aos && new_spellbook {
        send_new_spellbook_content(w, ns, book, graphic, offset, content);
    } else {
        send_old_spellbook_content(w, ns, book, offset, content);
    }
}

pub fn create_new_spellbook_content(
    book: Serial,
    graphic: i32,
    offset: i32,
    content: u64,
) -> Vec<u8> {
    let mut p = Pk::new(23);
    p.u8(0xBF); // Packet ID
    p.u16(23); // Length
    p.i16(0x1B); // Subpacket
    p.i16(0x01); // Command
    p.serial(book);
    p.i16(graphic as i16);
    p.i16(offset as i16);
    for i in 0..8 {
        p.u8((content >> (i * 8)) as u8);
    }
    p.finish()
}

pub fn send_new_spellbook_content(
    w: &mut World,
    ns: NetStateId,
    book: Serial,
    graphic: i32,
    offset: i32,
    content: u64,
) {
    if w.cannot_send_packets(ns) {
        return;
    }
    w.send(
        ns,
        &create_new_spellbook_content(book, graphic, offset, content),
    );
}

pub fn create_old_spellbook_content(
    caps: &NetCaps,
    book: Serial,
    offset: i32,
    content: u64,
) -> Vec<u8> {
    let grid = caps.container_grid_lines();
    let count = content.count_ones() as usize;
    let length = 5 + count * if grid { 20 } else { 19 };

    let mut p = Pk::new(length);
    p.u8(0x3C); // Packet ID
    p.u16(length as u16);
    p.u16(count as u16);
    for i in 0..64i32 {
        if content & (1u64 << i) != 0 {
            p.i32(0x7FFF_FFFF - i);
            p.u16(0); // child ItemID
            p.u8(0); // ItemID offset
            p.u16((i + offset) as u16); // Amount
            p.i32(0); // X, Y
            if grid {
                p.u8(0); // Grid Location
            }
            p.serial(book);
            p.i16(0); // Quest Hue
        }
    }
    p.finish()
}

pub fn send_old_spellbook_content(
    w: &mut World,
    ns: NetStateId,
    book: Serial,
    offset: i32,
    content: u64,
) {
    if w.cannot_send_packets(ns) {
        return;
    }
    let Some(c) = caps(w, ns) else { return };
    w.send(ns, &create_old_spellbook_content(&c, book, offset, content));
}

pub fn create_display_container(caps: &NetCaps, cont: Serial, gump_id: i32) -> Vec<u8> {
    let hs = caps.high_seas();
    let mut p = Pk::new(if hs { 9 } else { 7 });
    p.u8(0x24); // Packet ID
    p.serial(cont);
    p.u16(gump_id as u16);
    if hs {
        p.i16(0x7D);
    }
    p.finish()
}

pub fn send_display_container(w: &mut World, ns: NetStateId, cont: Serial, gump_id: i32) {
    if w.cannot_send_packets(ns) {
        return;
    }
    let Some(c) = caps(w, ns) else { return };
    w.send(ns, &create_display_container(&c, cont, gump_id));
}

/// One 0x25 / 0x3C entry body after the serial: itemID .. hue, shared by the content packets.
#[allow(clippy::too_many_arguments)]
fn write_entry(
    p: &mut Pk,
    caps: &NetCaps,
    item_id: i32,
    amount: i32,
    x: i32,
    y: i32,
    parent: Serial,
    hue: i32,
) {
    p.u16(item_id as u16);
    p.u8(0); // signed, itemID offset
    p.u16(amount.min(u16::MAX as i32) as u16);
    p.i16(x as i16);
    p.i16(y as i16);
    if caps.container_grid_lines() {
        p.u8(0); // Grid Location?
    }
    p.serial(parent);
    p.u16(hue as u16);
}

pub fn create_container_content_update(w: &World, caps: &NetCaps, item: ItemId) -> Option<Vec<u8>> {
    let it = w.item(item)?;
    let parent_serial = match it.parent {
        Some(crate::ids::EntityId::Item(p)) => p.0,
        _ => {
            tracing::warn!(
                "ContainerContentUpdate on Item {} ({:?}) where parent is not an Item",
                it.ty.full_name.rsplit('.').next().unwrap_or(""),
                it.serial
            );
            Serial::ZERO
        }
    };

    let mut p = Pk::new(if caps.container_grid_lines() { 21 } else { 20 });
    p.u8(0x25); // Packet ID
    p.serial(it.serial);
    let hue = if it.quest_item() {
        QUEST_ITEM_HUE
    } else {
        it.hue
    };
    write_entry(
        &mut p,
        caps,
        it.item_id,
        it.amount,
        it.location.x,
        it.location.y,
        parent_serial,
        hue,
    );
    Some(p.finish())
}

pub fn send_container_content_update(w: &mut World, ns: NetStateId, item: ItemId) {
    if w.cannot_send_packets(ns) {
        return;
    }
    let Some(c) = caps(w, ns) else { return };
    if let Some(b) = create_container_content_update(w, &c, item) {
        w.send(ns, &b);
    }
}

pub fn create_container_content(
    w: &mut World,
    caps: &NetCaps,
    beholder: MobileId,
    beheld: ItemId,
) -> Option<Vec<u8>> {
    let ty = w.mobile(beholder)?.ty;
    let children = w.item(beheld)?.items.clone();
    // `CanSee` can have side effects, so it runs before the packet borrows the world.
    let seen: Vec<bool> = children
        .iter()
        .map(|&c| w.item(c).is_some_and(|ch| !ch.deleted()) && (ty.can_see_item)(w, beholder, c))
        .collect();
    let cont = w.item(beheld)?;
    let count = cont.items.len();
    let entry_size = if caps.container_grid_lines() { 20 } else { 19 };
    let max_entries = (MAX_CONTAINER_CONTENT_BYTES - 5) / entry_size;
    let entries_to_send = count.min(max_entries);

    let mut p = Pk::new(5 + entries_to_send * entry_size);
    p.u8(0x3C); // Packet ID
    p.zeros(4); // Length & written count

    let mut written = 0usize;
    let mut i = 0;
    while i < count && written < entries_to_send {
        let child_id = cont.items[i];
        i += 1;
        let Some(child) = w.item(child_id) else {
            continue;
        };
        if !child.deleted() && seen.get(i - 1).copied().unwrap_or(false) {
            p.serial(child.serial);
            let hue = if child.quest_item() {
                QUEST_ITEM_HUE
            } else {
                child.hue
            };
            let loc = child.location;
            write_entry(
                &mut p,
                caps,
                child.item_id,
                child.amount,
                loc.x,
                loc.y,
                cont.serial,
                hue,
            );
            written += 1;
        }
    }

    let mut v = p.finish();
    let len = v.len() as u16;
    v[1..3].copy_from_slice(&len.to_be_bytes());
    v[3..5].copy_from_slice(&(written as u16).to_be_bytes());
    Some(v)
}

pub fn send_container_content(w: &mut World, ns: NetStateId, beholder: MobileId, beheld: ItemId) {
    if w.cannot_send_packets(ns) {
        return;
    }
    let Some(c) = caps(w, ns) else { return };
    if let Some(b) = create_container_content(w, &c, beholder, beheld) {
        w.send(ns, &b);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::item::ITEM_BASE;
    use crate::mobile::mobile_type::MOBILE_BASE;
    use muo_net::protocol_changes::ProtocolChanges;

    fn caps(pc: ProtocolChanges) -> NetCaps {
        NetCaps {
            protocol_changes: pc,
            ..Default::default()
        }
    }

    const GRID: ProtocolChanges = ProtocolChanges::CONTAINER_GRID_LINES;
    const HS: ProtocolChanges = ProtocolChanges::HIGH_SEAS;

    #[test]
    fn container_display() {
        let b = create_display_container(&caps(ProtocolChanges::empty()), Serial::new(0x1024), 100);
        assert_eq!(b, [0x24, 0, 0, 0x10, 0x24, 0, 100]);
    }

    #[test]
    fn container_display_hs() {
        let b = create_display_container(&caps(GRID | HS), Serial::new(0x1024), 100);
        assert_eq!(b, [0x24, 0, 0, 0x10, 0x24, 0, 100, 0, 0x7D]);
    }

    #[test]
    fn display_spellbook() {
        let b = create_display_container(&caps(ProtocolChanges::empty()), Serial::new(0x1024), -1);
        assert_eq!(b, [0x24, 0, 0, 0x10, 0x24, 0xFF, 0xFF]);
        let b = create_display_container(&caps(GRID | HS), Serial::new(0x1024), -1);
        assert_eq!(b, [0x24, 0, 0, 0x10, 0x24, 0xFF, 0xFF, 0, 0x7D]);
    }

    #[test]
    fn new_spellbook_content() {
        let b = create_new_spellbook_content(Serial::new(0x1024), 100, 10, 0x123456789ABCDEF0);
        let mut e = vec![0xBF, 0, 23, 0, 0x1B, 0, 1, 0, 0, 0x10, 0x24, 0, 100, 0, 10];
        e.extend([0xF0, 0xDE, 0xBC, 0x9A, 0x78, 0x56, 0x34, 0x12]);
        assert_eq!(b, e);
    }

    // The legacy SpellbookContent: id, count, then 19 (20 with grid) bytes per set bit, lowest bit first.
    fn legacy_spellbook(grid: bool, book: u32, offset: u16, content: u64) -> Vec<u8> {
        let n = content.count_ones() as usize;
        let mut v = vec![0x3C];
        v.extend(((5 + n * if grid { 20 } else { 19 }) as u16).to_be_bytes());
        v.extend((n as u16).to_be_bytes());
        for i in 0..64u32 {
            if content & (1 << i) != 0 {
                v.extend((0x7FFF_FFFFu32 - i).to_be_bytes());
                v.extend([0, 0, 0]);
                v.extend((i as u16 + offset).to_be_bytes());
                v.extend([0, 0, 0, 0]);
                if grid {
                    v.push(0);
                }
                v.extend(book.to_be_bytes());
                v.extend([0, 0]);
            }
        }
        v
    }

    #[test]
    fn spellbook_content() {
        let c = 0x123456789ABCDEF0;
        let b = create_old_spellbook_content(
            &caps(ProtocolChanges::empty()),
            Serial::new(0x1024),
            10,
            c,
        );
        assert_eq!(b, legacy_spellbook(false, 0x1024, 10, c));
        let b = create_old_spellbook_content(&caps(GRID), Serial::new(0x1024), 10, c);
        assert_eq!(b, legacy_spellbook(true, 0x1024, 10, c));
    }

    fn setup() -> (World, NetStateId) {
        let mut w = World::new();
        let ns = NetStateId(1);
        w.net_states.insert(ns, Default::default());
        (w, ns)
    }

    // The legacy ContainerContentUpdate for `new Item(serial)` whose parent isn't an item: parent serial 0.
    #[test]
    fn container_content_update() {
        let (mut w, _) = setup();
        let id = w.new_item(&ITEM_BASE);
        let s = id.0.value().to_be_bytes();
        let mut e = vec![0x25];
        e.extend(s);
        e.extend([0, 0, 0, 0, 1, 0, 0, 0, 0]); // itemID 0, offset, amount 1, x 0, y 0
        e.extend([0, 0, 0, 0, 0, 0]); // parent 0, hue 0
        e.truncate(20);
        let b = create_container_content_update(&w, &caps(ProtocolChanges::empty()), id).unwrap();
        assert_eq!(b.len(), 20);
        assert_eq!(b, e);
        let b = create_container_content_update(&w, &caps(GRID), id).unwrap();
        assert_eq!(b.len(), 21);
        assert_eq!(b[15], 0); // grid location
    }

    fn viewed_container(w: &mut World, n: usize) -> (crate::ids::ItemId, MobileId) {
        // both default to Map.Internal, which CanSee refuses
        let map = Some(crate::map::MapId::FELUCCA);
        let cont = w.new_item(&ITEM_BASE);
        for _ in 0..n {
            let c = w.new_item(&ITEM_BASE);
            w.item_mut(c).unwrap().map = map;
            w.item_mut(cont).unwrap().items.push(c);
        }
        w.item_mut(cont).unwrap().map = map;
        let m = w.new_mobile(&MOBILE_BASE);
        w.mobile_mut(m).unwrap().map = map;
        (cont, m)
    }

    #[test]
    fn container_content() {
        let (mut w, _) = setup();
        let (cont, m) = viewed_container(&mut w, 1);
        let child = w.item(cont).unwrap().items[0];
        for (pc, size) in [(ProtocolChanges::empty(), 19), (GRID, 20)] {
            let b = create_container_content(&mut w, &caps(pc), m, cont).unwrap();
            let mut e = vec![0x3C, 0, (5 + size) as u8, 0, 1];
            e.extend(child.0.value().to_be_bytes());
            e.extend([0, 0, 0, 0, 1, 0, 0, 0, 0]);
            if size == 20 {
                e.push(0);
            }
            e.extend(cont.0.value().to_be_bytes());
            e.extend([0, 0]);
            assert_eq!(b, e);
        }
    }

    #[test]
    fn container_content_caps_at_protocol_limit() {
        for (grid, max_entries, max_length) in [(false, 3448usize, 65517u16), (true, 3276, 65525)] {
            let (mut w, _) = setup();
            let (cont, m) = viewed_container(&mut w, max_entries + 1);
            let pc = if grid { GRID } else { ProtocolChanges::empty() };
            let b = create_container_content(&mut w, &caps(pc), m, cont).unwrap();
            assert_eq!(b[0], 0x3C);
            assert_eq!(u16::from_be_bytes([b[1], b[2]]), max_length);
            assert_eq!(b.len(), max_length as usize);
            assert_eq!(u16::from_be_bytes([b[3], b[4]]) as usize, max_entries);
        }
    }

    #[test]
    fn container_content_many_matches_per_entry_layout() {
        let (mut w, _) = setup();
        let (cont, m) = viewed_container(&mut w, 100);
        let b = create_container_content(&mut w, &caps(ProtocolChanges::empty()), m, cont).unwrap();
        assert_eq!(b.len(), 5 + 100 * 19);
        assert_eq!(u16::from_be_bytes([b[3], b[4]]), 100);
    }

    #[test]
    fn send_respects_cannot_send() {
        let (mut w, ns) = setup();
        send_display_container(&mut w, ns, Serial::new(1), 5);
        assert_eq!(w.net_out.len(), 1);
        w.net_state_mut(ns).unwrap().cannot_send = true;
        send_display_container(&mut w, ns, Serial::new(1), 5);
        assert_eq!(w.net_out.len(), 1);
    }
}
