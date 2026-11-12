//! Book, bulletin board and map packet tests: the expected packets are written out as byte builders below and the
//! server's packet writer is compared against them.

use muo_net::protocol_changes::ProtocolChanges;
use muo_primitives::Serial;
use muo_server::World;
use muo_server::ids::{ItemId, MobileId};
use muo_server::mobile::MOBILE_BASE;
use muo_server::net_state::{NetOut, NetStateData, NetStateId};

use crate::books::{self, BASE_BOOK};
use crate::bulletin::{self, BULLETIN_BOARD};
use crate::maps::{self, MAP_ITEM};

const FANCY: &str =
    "\u{1F135}\u{1F130}\u{1F13D}\u{1F132}\u{1F148} \u{1F143}\u{1F134}\u{1F147}\u{1F143}";

fn net_world() -> (World, NetStateId) {
    let mut w = muo_server::map::test_support::world();
    let ns = NetStateId(1);
    w.net_states.insert(ns, NetStateData::new(0));
    (w, ns)
}

/// What the world sent to the one connection, as a single packet.
fn sent(w: &mut World) -> Vec<u8> {
    let mut all: Vec<Vec<u8>> = std::mem::take(&mut w.net_out)
        .into_iter()
        .filter_map(|o| match o {
            NetOut::Send(_, b) => Some(b),
            _ => None,
        })
        .collect();
    assert_eq!(all.len(), 1, "exactly one packet");
    all.remove(0)
}

/// A legacy `Packet`: id, optional length, body. `Compile()` patches the length of a variable packet.
struct Packet {
    b: Vec<u8>,
    variable: bool,
}

impl Packet {
    fn fixed(id: u8) -> Packet {
        Packet {
            b: vec![id],
            variable: false,
        }
    }
    fn variable(id: u8) -> Packet {
        Packet {
            b: vec![id, 0, 0],
            variable: true,
        }
    }
    fn u8(&mut self, v: u8) -> &mut Self {
        self.b.push(v);
        self
    }
    fn i16(&mut self, v: i16) -> &mut Self {
        self.b.extend_from_slice(&v.to_be_bytes());
        self
    }
    fn u32(&mut self, v: u32) -> &mut Self {
        self.b.extend_from_slice(&v.to_be_bytes());
        self
    }
    fn bytes(&mut self, v: &[u8]) -> &mut Self {
        self.b.extend_from_slice(v);
        self
    }
    fn compile(mut self) -> Vec<u8> {
        if self.variable {
            let n = self.b.len() as u16;
            self.b[1..3].copy_from_slice(&n.to_be_bytes());
        }
        self.b
    }
}

fn mobile(w: &mut World) -> MobileId {
    w.new_mobile(&MOBILE_BASE)
}

// ---- BookPacketTests ----

fn test_book(w: &mut World, title: &str, author: &str) -> ItemId {
    books::new_book(w, &BASE_BOOK, 0xFF1, Some(title), Some(author), 20, true)
}

#[test]
fn test_book_cover() {
    let (mut w, ns) = net_world();
    let m = mobile(&mut w);
    let (author, title) = (format!("{FANCY} Author"), format!("{FANCY} Title"));
    let book = test_book(&mut w, &title, &author);
    let serial = w.item(book).unwrap().serial;

    // BookHeader: both sit at the origin, so `Writable && from.InRange(book.GetWorldLocation(), 1)` holds
    let near = true;
    let (t, a) = (title.as_bytes(), author.as_bytes());
    let mut p = Packet::fixed(0xD4);
    p.b.extend_from_slice(&[0, 0]); // the length of the variable packet, patched below
    p.u32(serial.value()).u8(1).u8(u8::from(near)).i16(20);
    p.i16((t.len() + 1) as i16)
        .bytes(t)
        .u8(0)
        .i16((a.len() + 1) as i16)
        .bytes(a)
        .u8(0);
    let mut expected = p.b;
    let n = expected.len() as u16;
    expected[1..3].copy_from_slice(&n.to_be_bytes());

    books::send_book_cover(&mut w, ns, m, book);
    assert_eq!(sent(&mut w), expected);
}

#[test]
fn test_book_content() {
    let (mut w, ns) = net_world();
    let book = test_book(&mut w, "Some Title", "Some Author");
    let pages: [Vec<String>; 3] = [
        [
            "Some books start with actual content",
            "This book does not have any actual content",
            "Instead it has several pages of useless text",
        ]
        .map(String::from)
        .to_vec(),
        vec![
            "Another page exists but this page:".to_owned(),
            format!("Has lots of: {FANCY}"),
            format!("And just more: {FANCY}"),
            format!("So everyone can read: {FANCY}"),
        ],
        vec!["The end".to_owned()],
    ];
    {
        let d = books::data_mut(&mut w, book).unwrap();
        for (i, page) in pages.into_iter().enumerate() {
            d.pages[i] = page;
        }
    }
    let serial = w.item(book).unwrap().serial;

    let mut p = Packet::variable(0x66);
    p.u32(serial.value()).i16(20);
    let all = books::data(&w, book).unwrap().pages.clone();
    for (i, page) in all.iter().enumerate() {
        p.i16((i + 1) as i16).i16(page.len() as i16);
        for line in page {
            p.bytes(line.as_bytes()).u8(0);
        }
    }
    books::send_book_content(&mut w, ns, book);
    assert_eq!(sent(&mut w), p.compile());
}

// ---- BulletinBoardPacketTests ----

fn board(w: &mut World, name: Option<&str>) -> ItemId {
    let b = bulletin::base_board_new(w, &BULLETIN_BOARD, 0x234);
    w.item_mut(b)
        .unwrap()
        .ext
        .get_mut::<bulletin::BoardData>()
        .unwrap()
        .board_name = name.map(str::to_owned);
    b
}

#[test]
fn test_send_bb_display_board() {
    for name in [Some("Test Name"), None, Some(FANCY)] {
        let (mut w, ns) = net_world();
        let bb = board(&mut w, name);
        let serial = w.item(bb).unwrap().serial;

        // BBDisplayBoard
        let buffer = name.unwrap_or("").as_bytes();
        let mut p = Packet::fixed(0x71);
        p.b.extend_from_slice(&[0, 0]);
        p.u8(0).u32(serial.value());
        if buffer.len() >= 29 {
            p.bytes(&buffer[..29]).u8(0);
        } else {
            p.bytes(buffer).bytes(&vec![0; 30 - buffer.len()]);
        }
        let mut expected = p.b;
        let n = expected.len() as u16;
        expected[1..3].copy_from_slice(&n.to_be_bytes());

        bulletin::send_bb_display_board(&mut w, ns, bb);
        assert_eq!(sent(&mut w), expected, "{name:?}");
    }
}

/// `BBMessageHeader.WriteString` / `BBMessageContent.WriteString`.
fn write_string(p: &mut Packet, v: &str, padding: bool) {
    let buffer = v.as_bytes();
    let tail = if padding { 2 } else { 1 };
    let len = (buffer.len() + tail).min(255);
    p.u8(len as u8).bytes(&buffer[..len - tail]);
    if padding {
        p.i16(0); // padding compensates for a client bug
    } else {
        p.u8(0);
    }
}

#[test]
fn test_send_bb_header_message() {
    let cases: [(&str, bool, Vec<&str>); 6] = [
        (
            "The Subject",
            false,
            vec!["First Line", "Second Line", "Third Line"],
        ),
        ("", false, vec![""]),
        (FANCY, false, vec!["First Line", "Second Line"]),
        (
            "The Subject",
            true,
            vec!["First Line", "Second Line", "Third Line"],
        ),
        ("", true, vec![""]),
        (FANCY, true, vec!["First Line", "Second Line"]),
    ];
    for (subject, content, lines) in cases {
        let (mut w, ns) = net_world();
        let poster = mobile(&mut w);
        w.mobile_mut(poster).unwrap().name = Some("Kamron".to_owned());
        let bb = board(&mut w, Some("bulletin board"));
        bulletin::post_message(
            &mut w,
            bb,
            poster,
            None,
            subject.to_owned(),
            lines.iter().map(|l| (*l).to_owned()).collect(),
        );
        let msg = w.item(bb).unwrap().items[0];
        let md = bulletin::message(&w, msg).unwrap().clone();
        let (bs, ms) = (w.item(bb).unwrap().serial, w.item(msg).unwrap().serial);

        let mut p = Packet::variable(0x71);
        p.u8(if content { 2 } else { 1 })
            .u32(bs.value())
            .u32(ms.value());
        if !content {
            p.u32(
                md.thread
                    .and_then(|t| w.item(t))
                    .map_or(Serial::ZERO, |t| t.serial)
                    .value(),
            ); // thread serial--parent
        }
        write_string(&mut p, md.posted_name.as_deref().unwrap_or(""), false);
        write_string(&mut p, md.subject.as_deref().unwrap_or(""), false);
        write_string(&mut p, &bulletin::time_as_string(md.time), false);
        if content {
            p.i16(md.posted_body as i16).i16(md.posted_hue as i16);
            let len = md.posted_equip.len().min(255);
            p.u8(len as u8);
            for &(item_id, hue) in md.posted_equip.iter().take(len) {
                p.i16(item_id as i16).i16(hue as i16);
            }
            let len = md.lines.len().min(255);
            p.u8(len as u8);
            for l in md.lines.iter().take(len) {
                write_string(&mut p, l.as_deref().unwrap_or(""), true);
            }
        }

        bulletin::send_bb_message(&mut w, ns, bb, msg, content);
        assert_eq!(sent(&mut w), p.compile(), "{subject:?} content {content}");
    }
}

// ---- TestMapItemPackets ----

#[test]
fn test_send_map_details() {
    for changes in [
        ProtocolChanges::NEW_CHARACTER_LIST,
        ProtocolChanges::empty(),
    ] {
        let (mut w, ns) = net_world();
        w.net_states.get_mut(&ns).unwrap().caps.protocol_changes = changes;
        let map = maps::map_item_new(&mut w, &MAP_ITEM, Some(muo_server::map::MapId::TRAMMEL));
        let (serial, d) = (
            w.item(map).unwrap().serial,
            maps::data(&w, map).unwrap().clone(),
        );

        let new_char_list = changes.contains(ProtocolChanges::NEW_CHARACTER_LIST);
        let mut p = Packet::fixed(if new_char_list { 0xF5 } else { 0x90 });
        p.u32(serial.value()).i16(0x139D);
        p.i16(d.bounds.start.x as i16)
            .i16(d.bounds.start.y as i16)
            .i16(d.bounds.end.x as i16)
            .i16(d.bounds.end.y as i16);
        p.i16(d.width as i16).i16(d.height as i16);
        if new_char_list {
            p.i16(w.maps.get(d.facet.unwrap()).map_or(0, |m| m.map_id) as i16);
        }

        maps::send_map_details(&mut w, ns, map);
        assert_eq!(sent(&mut w), p.compile());
    }
}

#[test]
fn test_send_map_command() {
    for (command, number, x, y) in [(5, 0, 0, 0), (1, 0, 100, 200), (7, 1, 0, 0), (7, 0, 0, 0)] {
        let (mut w, ns) = net_world();
        let map = maps::map_item_new(&mut w, &MAP_ITEM, Some(muo_server::map::MapId::TRAMMEL));
        let serial = w.item(map).unwrap().serial;

        let mut p = Packet::fixed(0x56);
        p.u32(serial.value())
            .u8(command as u8)
            .u8(number as u8)
            .i16(x as i16)
            .i16(y as i16);

        maps::send_map_command(&mut w, ns, map, command as u8, x, y, number > 0);
        assert_eq!(sent(&mut w), p.compile(), "{command} {number} {x} {y}");
    }
}
