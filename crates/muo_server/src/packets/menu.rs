//! Menu packets, plus `SendDisplayContextMenu`
//! (the 0xBF/0x14 packet).

use bitflags::bitflags;
use muo_primitives::Serial;

use super::Pk;
use crate::context_menu_entry::{ContextMenu, ContextMenuEntry};
use crate::ids::EntityId;
use crate::menus::{ItemListEntry, Menu, MenuKind};
use crate::net_state::NetStateId;
use crate::world::World;

bitflags! {
    /// `CMEFlags`.
    #[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash)]
    pub struct CmeFlags: i32 {
        const NONE = 0x00;
        const DISABLED = 0x01;
        const ARROW = 0x02;
        const HIGHLIGHTED = 0x04;
        const COLORED = 0x20;
    }
}

/// `Core.GlobalUpdateRange`.
// shortcut: there is no setting for it (a property nobody assigns), so a const.
pub const GLOBAL_UPDATE_RANGE: i32 = 18;

/// `string.Length` in UTF-16 units, as the length byte counted it.
fn len16(s: &str) -> usize {
    s.encode_utf16().count()
}

/// 0x7C header and question; the caller adds the entries.
fn menu_header(serial: i32, question: &str) -> Pk {
    let mut p = Pk::new(11 + len16(question));
    p.u8(0x7C)
        .u16(0)
        .i32(serial)
        .u16(0)
        .u8(len16(question) as u8);
    let _ = p.write_latin1(question);
    p
}

/// `SendDisplayItemListMenu` (0x7C).
pub fn create_display_item_list_menu(
    serial: i32,
    question: &str,
    entries: &[ItemListEntry],
) -> Vec<u8> {
    let n = entries.len() as u8;
    let mut p = menu_header(serial, question);
    p.u8(n);
    for e in entries.iter().take(n as usize) {
        let name = e.name.trim();
        p.u16(e.item_id as u16)
            .i16(e.hue as i16)
            .u8(len16(name) as u8);
        let _ = p.write_latin1(name);
    }
    p.finish_with_length()
}

/// `SendDisplayQuestionMenu` (0x7C).
pub fn create_display_question_menu(serial: i32, question: &str, answers: &[String]) -> Vec<u8> {
    let n = answers.len() as u8;
    let mut p = menu_header(serial, question);
    p.u8(n);
    for a in answers.iter().take(n as usize) {
        let a = a.trim();
        p.i32(0).u8(len16(a) as u8);
        let _ = p.write_latin1(a);
    }
    p.finish_with_length()
}

/// The 0x7C packet of either kind of menu.
pub fn create_display_menu(menu: &Menu) -> Vec<u8> {
    match &menu.kind {
        MenuKind::ItemList { question, entries } => {
            create_display_item_list_menu(menu.serial, question.trim(), entries)
        }
        MenuKind::Question { question, answers } => {
            create_display_question_menu(menu.serial, question.trim(), answers)
        }
    }
}

#[cfg(test)]
pub fn send_display_menu(w: &mut World, ns: NetStateId, menu: &Menu) {
    w.send(ns, &create_display_menu(menu));
}

/// `SendDisplayContextMenu` (0xBF, subcommand 0x14). `in_range(range)` is `menu.From.InRange(p, range)` for the
/// target's location `p`.
pub fn create_display_context_menu(
    entries: &[ContextMenuEntry],
    target: Serial,
    new_command: bool,
    in_range: impl Fn(i32) -> bool,
) -> Vec<u8> {
    let n = entries.len() as u8;
    let mut p = Pk::new(12 + n as usize * 8);
    p.u8(0xBF)
        .u16(0)
        .i16(0x14)
        .i16(if new_command { 0x02 } else { 0x01 })
        .serial(target)
        .u8(n);
    for (i, e) in entries.iter().take(n as usize).enumerate() {
        let range = if e.range == -1 {
            GLOBAL_UPDATE_RANGE
        } else {
            e.range
        };
        let mut flags = e.flags;
        if !(e.enabled && in_range(range)) {
            flags |= CmeFlags::DISABLED;
        }
        if new_command {
            p.i32(e.number).i16(i as i16).i16(flags.bits() as i16);
        } else {
            p.i16(i as i16).u16((e.number - 3_000_000) as u16);
            let color = e.color & 0xFFFF;
            if color != 0xFFFF {
                flags |= CmeFlags::COLORED;
            }
            p.i16(flags.bits() as i16);
            if flags.contains(CmeFlags::COLORED) {
                p.i16(color as i16);
            }
        }
    }
    p.finish_with_length()
}

pub fn send_display_context_menu(w: &mut World, ns: NetStateId, menu: &ContextMenu) {
    let new_command =
        w.net_state(ns).is_some_and(|d| d.caps.new_haven()) && menu.requires_new_packet;
    let loc = match menu.target {
        EntityId::Mobile(m) => w.mobile(m).map(|x| x.location).unwrap_or_default(),
        EntityId::Item(i) => crate::map::tiles::world_location(w, i).unwrap_or_default(),
    };
    let from = menu.from;
    let bytes =
        create_display_context_menu(&menu.entries, menu.target.serial(), new_command, |r| {
            crate::mobile::base::in_range_point3d(w, from, loc, r)
        });
    w.send(ns, &bytes);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net_state::{NetOut, NetStateData};
    use muo_net::protocol_changes::ProtocolChanges;

    /// Legacy `DisplayItemListMenu`: serial, short 0, ascii strings, no padding.
    #[test]
    fn item_list_menu() {
        let mut w = World::new();
        let menu = Menu::item_list(
            &mut w,
            "Which item would you choose?",
            vec![
                ItemListEntry::new("Item 1", 0x01, 0, 0),
                ItemListEntry::new("Item 2", 0x100, 0, 0),
                ItemListEntry::new("Item 3", 0x1000, 250, 0),
            ],
        );
        let mut e = vec![0x7C, 0, 0];
        e.extend(1i32.to_be_bytes());
        e.extend([0, 0]);
        e.push(28);
        e.extend(b"Which item would you choose?");
        e.push(3);
        for (id, hue, name) in [
            (1u16, 0i16, "Item 1"),
            (0x100, 0, "Item 2"),
            (0x1000, 250, "Item 3"),
        ] {
            e.extend(id.to_be_bytes());
            e.extend(hue.to_be_bytes());
            e.push(name.len() as u8);
            e.extend(name.bytes());
        }
        let len = e.len() as u16;
        e[1..3].copy_from_slice(&len.to_be_bytes());
        assert_eq!(create_display_menu(&menu), e);
    }

    #[test]
    fn question_menu() {
        let mut w = World::new();
        let menu = Menu::question(
            &mut w,
            "Which option would you choose?",
            vec!["Option 1".into(), "Option 2".into(), "Option 3".into()],
        );
        let mut e = vec![0x7C, 0, 0];
        e.extend(1i32.to_be_bytes());
        e.extend([0, 0]);
        e.push(30);
        e.extend(b"Which option would you choose?");
        e.push(3);
        for a in ["Option 1", "Option 2", "Option 3"] {
            e.extend([0, 0, 0, 0]);
            e.push(a.len() as u8);
            e.extend(a.bytes());
        }
        let len = e.len() as u16;
        e[1..3].copy_from_slice(&len.to_be_bytes());
        let ns = NetStateId(1);
        w.net_states.insert(ns, NetStateData::default());
        send_display_menu(&mut w, ns, &menu);
        let Some(NetOut::Send(_, b)) = w.net_out.last() else {
            panic!("nothing sent")
        };
        assert_eq!(*b, e);
    }

    fn entries(new_packet: bool) -> Vec<ContextMenuEntry> {
        let mut v = vec![
            ContextMenuEntry::new(3000001, -1),
            ContextMenuEntry::new(3000002, -1),
            ContextMenuEntry::new(3000003, -1),
        ];
        if new_packet {
            v.push(ContextMenuEntry::new(500000, -1));
        }
        v
    }

    /// Legacy `DisplayContextMenuOld` (command 1) and `DisplayContextMenu` (command 2), in range.
    #[test]
    fn context_menu_both_commands() {
        let target = Serial::new(0x4000_0001);
        for new_packet in [false, true] {
            for new_haven in [false, true] {
                let es = entries(new_packet);
                let new_command = new_haven && new_packet;
                let got = create_display_context_menu(&es, target, new_command, |_| true);
                let mut e = vec![0xBF, 0, 0, 0, 0x14, 0, if new_command { 2 } else { 1 }];
                e.extend(0x4000_0001u32.to_be_bytes());
                e.push(es.len() as u8);
                for (i, en) in es.iter().enumerate() {
                    if new_command {
                        e.extend(en.number.to_be_bytes());
                        e.extend((i as i16).to_be_bytes());
                        e.extend([0, 0]);
                    } else {
                        e.extend((i as i16).to_be_bytes());
                        e.extend(((en.number - 3_000_000) as u16).to_be_bytes());
                        e.extend([0, 0]);
                    }
                }
                let len = e.len() as u16;
                e[1..3].copy_from_slice(&len.to_be_bytes());
                assert_eq!(got, e, "new_packet={new_packet} new_haven={new_haven}");
            }
        }
    }

    #[test]
    fn context_menu_disabled_and_colored() {
        let mut a = ContextMenuEntry::new(1, 2);
        a.color = 0x1234;
        let mut b = ContextMenuEntry::new(2, -1);
        b.enabled = false;
        let got = create_display_context_menu(&[a, b], Serial::new(1), false, |r| r > 2);
        // Entry 0 is out of range: disabled|colored = 0x21 then the color. Entry 1 is not enabled.
        let e = [
            0xBF, 0, 0x1A, 0, 0x14, 0, 1, 0, 0, 0, 1, 2, 0, 0, 0, 1, 0, 0x21, 0x12, 0x34, 0, 1, 0,
            2, 0, 1,
        ];
        assert_eq!(got, e);
    }

    #[test]
    fn send_context_menu_uses_the_new_haven_cap() {
        let mut w = World::new();
        let ns = NetStateId(1);
        let mut d = NetStateData::default();
        d.caps.protocol_changes = ProtocolChanges::NEW_HAVEN;
        w.net_states.insert(ns, d);
        let m = w.new_mobile(&crate::mobile::mobile_type::MOBILE_BASE);
        let menu = ContextMenu::new(m, EntityId::Mobile(m), entries(true));
        assert!(menu.requires_new_packet);
        send_display_context_menu(&mut w, ns, &menu);
        let Some(NetOut::Send(_, b)) = w.net_out.last() else {
            panic!("nothing sent")
        };
        assert_eq!(b[6], 2);
    }
}
