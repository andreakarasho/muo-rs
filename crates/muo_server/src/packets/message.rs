//! Message packets (single, broadcast, interpolated)
//! and `MessageType`.
//!
//! The interpolated-string-handler overloads are the same functions taking `&str` (callers use `format!`).
//! In the broadcast loops every allowed client gets
//! the same bytes: they are built once here.

use bitflags::bitflags;
use muo_primitives::{Point3D, Serial};

use super::Pk;
use crate::effects::clients_in_range;
use crate::map::MapId;
use crate::net_state::NetStateId;
use crate::world::World;

bitflags! {
    /// `MessageType` (written as a byte).
    #[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
    pub struct MessageType: u8 {
        const REGULAR = 0x00;
        const SYSTEM = 0x01;
        const EMOTE = 0x02;
        const LABEL = 0x06;
        const FOCUS = 0x07;
        const WHISPER = 0x08;
        const YELL = 0x09;
        const SPELL = 0x0A;
        const GUILD = 0x0D;
        const ALLIANCE = 0x0E;
        const COMMAND = 0x0F;
        const ENCODED = 0xC0;
    }

    /// `AffixType` (written as a byte).
    #[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
    pub struct AffixType: u8 {
        const APPEND = 0x00;
        const PREPEND = 0x01;
        const SYSTEM = 0x02;
    }
}

/// `IBroadcastFilter.Allow(NetState)`.
pub trait BroadcastFilter {
    fn allow(&self, w: &mut World, ns: NetStateId) -> bool;
}

impl<F: Fn(&mut World, NetStateId) -> bool> BroadcastFilter for F {
    fn allow(&self, w: &mut World, ns: NetStateId) -> bool {
        self(w, ns)
    }
}

#[allow(clippy::too_many_arguments)]
pub fn send_message_localized(
    w: &mut World,
    ns: NetStateId,
    serial: Serial,
    graphic: i32,
    kind: MessageType,
    hue: i32,
    font: i32,
    number: i32,
    name: &str,
    args: &str,
) {
    if w.cannot_send_packets(ns) {
        return;
    }
    let bytes = create_message_localized(serial, graphic, kind, hue, font, number, name, args);
    w.send(ns, &bytes);
}

pub fn get_max_message_localized_length(args_char_count: usize) -> usize {
    50 + args_char_count * 2
}

#[allow(clippy::too_many_arguments)]
pub fn create_message_localized(
    serial: Serial,
    graphic: i32,
    kind: MessageType,
    hue: i32,
    font: i32,
    number: i32,
    name: &str,
    args: &str,
) -> Vec<u8> {
    let hue = if hue == 0 { 0x3B2 } else { hue };

    let mut p = Pk::new(get_max_message_localized_length(args.len()));
    p.u8(0xC1).u16(0); // length is patched by finish_with_length
    p.serial(serial)
        .i16(graphic as i16)
        .u8(kind.bits())
        .i16(hue as i16)
        .i16(font as i16)
        .i32(number);
    p.write_latin1_fixed(name, 30).ok();
    p.write_little_uni_null(args).ok();
    p.finish_with_length()
}

#[allow(clippy::too_many_arguments)]
pub fn send_message_localized_affix(
    w: &mut World,
    ns: NetStateId,
    serial: Serial,
    graphic: i32,
    kind: MessageType,
    hue: i32,
    font: i32,
    number: i32,
    name: &str,
    affix_type: AffixType,
    affix: &str,
    args: &str,
) {
    if w.cannot_send_packets(ns) {
        return;
    }
    let bytes = create_message_localized_affix(
        serial, graphic, kind, hue, font, number, name, affix_type, affix, args,
    );
    w.send(ns, &bytes);
}

pub fn get_max_message_localized_affix_length(
    affix_char_count: usize,
    args_char_count: usize,
) -> usize {
    52 + affix_char_count + args_char_count * 2
}

#[allow(clippy::too_many_arguments)]
pub fn create_message_localized_affix(
    serial: Serial,
    graphic: i32,
    kind: MessageType,
    hue: i32,
    font: i32,
    number: i32,
    name: &str,
    affix_type: AffixType,
    affix: &str,
    args: &str,
) -> Vec<u8> {
    let hue = if hue == 0 { 0x3B2 } else { hue };

    let mut p = Pk::new(get_max_message_localized_affix_length(
        affix.len(),
        args.len(),
    ));
    p.u8(0xCC).u16(0);
    p.serial(serial)
        .i16(graphic as i16)
        .u8(kind.bits())
        .i16(hue as i16)
        .i16(font as i16)
        .i32(number)
        .u8(affix_type.bits());
    p.write_latin1_fixed(name, 30).ok();
    p.write_latin1_null(affix).ok();
    p.write_big_uni_null(args).ok();
    p.finish_with_length()
}

/// `lang` of `None` is null (becomes "ENU").
#[allow(clippy::too_many_arguments)]
pub fn send_message(
    w: &mut World,
    ns: NetStateId,
    serial: Serial,
    graphic: i32,
    kind: MessageType,
    hue: i32,
    font: i32,
    ascii: bool,
    lang: Option<&str>,
    name: &str,
    text: &str,
) {
    if w.cannot_send_packets(ns) {
        return;
    }
    let bytes = create_message(serial, graphic, kind, hue, font, ascii, lang, name, text);
    w.send(ns, &bytes);
}

pub fn get_max_message_length(text_char_count: usize) -> usize {
    50 + text_char_count * 2
}

#[allow(clippy::too_many_arguments)]
pub fn create_message(
    serial: Serial,
    graphic: i32,
    kind: MessageType,
    hue: i32,
    font: i32,
    ascii: bool,
    lang: Option<&str>,
    name: &str,
    text: &str,
) -> Vec<u8> {
    let lang = lang.unwrap_or("ENU");
    let hue = if hue == 0 { 0x3B2 } else { hue };

    let mut p = Pk::new(get_max_message_length(text.len()));
    p.u8(if ascii { 0x1C } else { 0xAE }).u16(0); // Packet ID, length
    p.serial(serial)
        .i16(graphic as i16)
        .u8(kind.bits())
        .i16(hue as i16)
        .i16(font as i16);
    if ascii {
        p.write_latin1_fixed(name, 30).ok();
        p.write_latin1_null(text).ok();
    } else {
        p.write_ascii_fixed(lang, 4).ok();
        p.write_latin1_fixed(name, 30).ok();
        p.write_big_uni_null(text).ok();
    }
    p.finish_with_length()
}

pub fn send_follow_message(w: &mut World, ns: NetStateId, s1: Serial, s2: Serial) {
    if w.cannot_send_packets(ns) {
        return;
    }
    let mut p = Pk::new(9);
    p.u8(0x15).serial(s1).serial(s2); // Packet ID
    w.send(ns, &p.finish());
}

/// `prompt` is `Prompt.Serial`; `None` is a null prompt.
pub fn send_prompt(w: &mut World, ns: NetStateId, prompt: Option<Serial>) {
    // Only a null prompt is checked here (`Send` itself drops a dead connection).
    let Some(prompt) = prompt else { return };
    if w.net_state(ns).is_none() {
        return;
    }
    let mut p = Pk::new(21);
    p.u8(0xC2).u16(21).serial(prompt).serial(prompt).zeros(10); // Packet ID
    w.send(ns, &p.finish());
}

pub fn send_help_response(w: &mut World, ns: NetStateId, s: Serial, text: Option<&str>) {
    let text = text.map(str::trim).unwrap_or("");
    if w.net_state(ns).is_none() || text.is_empty() {
        return;
    }
    let length = 9 + text.encode_utf16().count() * 2;
    let mut p = Pk::new(length);
    p.u8(0xB7).u16(length as u16).serial(s);
    p.write_big_uni_null(text).ok();
    w.send(ns, &p.finish());
}

/// `range` of `None` is the overload without one (`Core.GlobalMaxUpdateRange`).
#[allow(clippy::too_many_arguments)]
pub fn broadcast_message_localized(
    w: &mut World,
    map: Option<MapId>,
    location: Point3D,
    range: Option<i32>,
    serial: Serial,
    graphic: i32,
    kind: MessageType,
    hue: i32,
    font: i32,
    number: i32,
    name: &str,
    args: &str,
    filter: &impl BroadcastFilter,
) {
    let Some(map) = map else { return };
    let bytes = create_message_localized(serial, graphic, kind, hue, font, number, name, args);
    broadcast(w, map, location, range, filter, &bytes);
}

#[allow(clippy::too_many_arguments)]
pub fn broadcast_message_localized_affix(
    w: &mut World,
    map: Option<MapId>,
    location: Point3D,
    serial: Serial,
    graphic: i32,
    kind: MessageType,
    hue: i32,
    font: i32,
    number: i32,
    name: &str,
    affix_type: AffixType,
    affix: &str,
    args: &str,
    filter: &impl BroadcastFilter,
) {
    let Some(map) = map else { return };
    let bytes = create_message_localized_affix(
        serial, graphic, kind, hue, font, number, name, affix_type, affix, args,
    );
    broadcast(w, map, location, None, filter, &bytes);
}

#[allow(clippy::too_many_arguments)]
pub fn broadcast_message(
    w: &mut World,
    map: Option<MapId>,
    location: Point3D,
    range: Option<i32>,
    serial: Serial,
    graphic: i32,
    kind: MessageType,
    hue: i32,
    font: i32,
    ascii: bool,
    lang: Option<&str>,
    name: &str,
    text: &str,
    filter: &impl BroadcastFilter,
) {
    let Some(map) = map else { return };
    let bytes = create_message(serial, graphic, kind, hue, font, ascii, lang, name, text);
    broadcast(w, map, location, range, filter, &bytes);
}

fn broadcast(
    w: &mut World,
    map: MapId,
    location: Point3D,
    range: Option<i32>,
    filter: &impl BroadcastFilter,
    bytes: &[u8],
) {
    for ns in clients_in_range(w, map, location, range) {
        if filter.allow(w, ns) {
            w.send(ns, bytes);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net_state::{NetOut, NetStateData};

    fn world() -> (World, NetStateId) {
        let mut w = World::new();
        let ns = NetStateId(1);
        w.net_states.insert(ns, NetStateData::default());
        (w, ns)
    }

    fn sent(w: &World) -> Vec<u8> {
        match &w.net_out[0] {
            NetOut::Send(_, b) => b.clone(),
            _ => panic!("not a send"),
        }
    }

    /// Legacy fixed 30 byte name.
    fn fixed30(s: &str) -> Vec<u8> {
        let mut v = s.as_bytes().to_vec();
        v.resize(30, 0);
        v
    }

    fn uni_be(s: &str) -> Vec<u8> {
        let mut v: Vec<u8> = s.encode_utf16().flat_map(|c| c.to_be_bytes()).collect();
        v.extend([0, 0]);
        v
    }

    fn uni_le(s: &str) -> Vec<u8> {
        let mut v: Vec<u8> = s.encode_utf16().flat_map(|c| c.to_le_bytes()).collect();
        v.extend([0, 0]);
        v
    }

    fn with_len(mut v: Vec<u8>) -> Vec<u8> {
        let n = v.len() as u16;
        v[1..3].copy_from_slice(&n.to_be_bytes());
        v
    }

    // serial 0x1024, graphic 0x100, type Label, hue 1024, font 3
    const COMMON: [u8; 11] = [0, 0, 0x10, 0x24, 0x01, 0x00, 0x06, 0x04, 0x00, 0x00, 0x03];

    fn packet(id: u8, tail: &[u8]) -> Vec<u8> {
        let mut e = vec![id, 0, 0];
        e.extend(COMMON);
        e.extend(tail);
        with_len(e)
    }

    fn localized_expected(args: &str) -> Vec<u8> {
        let mut t = 150000i32.to_be_bytes().to_vec();
        t.extend(fixed30("Stuff"));
        t.extend(uni_le(args));
        packet(0xC1, &t)
    }

    #[test]
    fn message_localized() {
        for args in ["Arguments", "value: 42"] {
            let (mut w, ns) = world();
            send_message_localized(
                &mut w,
                ns,
                Serial::new(0x1024),
                0x100,
                MessageType::LABEL,
                1024,
                3,
                150000,
                "Stuff",
                args,
            );
            assert_eq!(sent(&w), localized_expected(args));
        }
    }

    #[test]
    fn message_localized_affix() {
        for args in ["Arguments", "value: 42"] {
            let (mut w, ns) = world();
            send_message_localized_affix(
                &mut w,
                ns,
                Serial::new(0x1024),
                0x100,
                MessageType::LABEL,
                1024,
                3,
                150000,
                "Stuff",
                AffixType::SYSTEM,
                "Affix",
                args,
            );
            let mut t = 150000i32.to_be_bytes().to_vec();
            t.push(2); // System
            t.extend(fixed30("Stuff"));
            t.extend(b"Affix\0");
            t.extend(uni_be(args));
            assert_eq!(sent(&w), packet(0xCC, &t));
        }
    }

    #[test]
    fn unicode_message() {
        for text in ["Some Text", "hello, Alice"] {
            let (mut w, ns) = world();
            send_message(
                &mut w,
                ns,
                Serial::new(0x1024),
                0x100,
                MessageType::LABEL,
                1024,
                3,
                false,
                Some("ENU"),
                "Stuff",
                text,
            );
            let mut t = b"ENU\0".to_vec();
            t.extend(fixed30("Stuff"));
            t.extend(uni_be(text));
            assert_eq!(sent(&w), packet(0xAE, &t));
        }
    }

    #[test]
    fn ascii_message() {
        let (mut w, ns) = world();
        send_message(
            &mut w,
            ns,
            Serial::new(0x1024),
            0x100,
            MessageType::LABEL,
            1024,
            3,
            true,
            None,
            "Stuff",
            "Some Text",
        );
        let mut t = fixed30("Stuff");
        t.extend(b"Some Text\0");
        assert_eq!(sent(&w), packet(0x1C, &t));
    }

    #[test]
    fn hue_zero_is_3b2() {
        let b = create_message(
            Serial::new(1),
            0,
            MessageType::REGULAR,
            0,
            0,
            true,
            None,
            "",
            "",
        );
        assert_eq!(b[10..12], [0x03, 0xB2]);
    }

    #[test]
    fn follow_message() {
        let (mut w, ns) = world();
        send_follow_message(&mut w, ns, Serial::new(0x1024), Serial::new(2));
        assert_eq!(sent(&w), [0x15, 0, 0, 0x10, 0x24, 0, 0, 0, 2]);
    }

    #[test]
    fn help_response() {
        let (mut w, ns) = world();
        let text = "This is some testing text";
        send_help_response(&mut w, ns, Serial::new(0x100), Some(text));
        let mut e = vec![0xB7, 0, 0, 0, 0, 0x01, 0x00];
        e.extend(uni_be(text));
        assert_eq!(sent(&w), with_len(e));
    }

    #[test]
    fn prompt() {
        let (mut w, ns) = world();
        send_prompt(&mut w, ns, Some(Serial::new(0x1234)));
        let mut e = vec![0xC2, 0, 21, 0, 0, 0x12, 0x34, 0, 0, 0x12, 0x34];
        e.extend([0; 10]);
        assert_eq!(sent(&w), e);
    }

    #[test]
    fn dead_connection_sends_nothing() {
        let (mut w, ns) = world();
        w.net_state_mut(ns).unwrap().cannot_send = true;
        send_follow_message(&mut w, ns, Serial::new(1), Serial::new(2));
        assert!(w.net_out.is_empty());
    }
}
