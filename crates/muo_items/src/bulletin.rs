//! Bulletin boards: `BulletinBoard`, its packets, `BulletinEquip` and `BulletinMessage`.
//!
//! A board is an `Item` whose children are its `BulletinMessage`s; `BaseBulletinBoard.HandleBBRequest` is the
//! family virtual of [`BoardClass`] (no class overrides it).

use muo_io::{GenericReader, GenericWriter, SpanReader};
use muo_primitives::{DateTime, MessageType, TimeSpan};
use muo_server::World;
use muo_server::ext::Ext;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType, deps, ops};
use muo_server::mobile::AccessLevel;
use muo_server::net_state::NetStateId;
use muo_server::packets::Pk;
use muo_server::serialization_ext::{ReadEntityExt, WriteEntityExt};
use muo_server::type_registry::{Phase, TypeRegistry};

use crate::misc::{read_version, write_version};

/// `BulletinBoardSystem`'s settings (`bulletinboards.*`).
#[derive(Copy, Clone, Debug)]
pub struct BoardSettings {
    /// `ThreadCreateTime`.
    pub create: TimeSpan,
    /// `ThreadDeletionTime`.
    pub deletion: TimeSpan,
    /// `ThreadReplyTime`.
    pub reply: TimeSpan,
}

impl Default for BoardSettings {
    fn default() -> Self {
        Self {
            create: TimeSpan::from_mins(2),
            deletion: TimeSpan::from_hours(6),
            reply: TimeSpan::from_secs(30),
        }
    }
}

fn settings(w: &World) -> BoardSettings {
    w.resources
        .get::<BoardSettings>()
        .copied()
        .unwrap_or_default()
}

/// `BulletinBoardSystem.Configure()`.
fn configure(w: &mut World) {
    let create = w
        .config
        .get_or_update_setting("bulletinboards.creationTimeDelay", TimeSpan::from_mins(2))
        .unwrap_or(TimeSpan::from_mins(2));
    let deletion = w
        .config
        .get_or_update_setting("bulletinboards.expireDuration", TimeSpan::from_hours(6))
        .unwrap_or(TimeSpan::from_hours(6));
    let reply = w
        .config
        .get_or_update_setting("bulletinboards.replyDelay", TimeSpan::from_secs(30))
        .unwrap_or(TimeSpan::from_secs(30));
    *w.resources.get_mut::<BoardSettings>() = BoardSettings {
        create,
        deletion,
        reply,
    };
    // IncomingPackets.Register(0x71, 0, true, &BBClientRequest)
    muo_content::network::register_packet(w, 0x71, 0, true, false, on_bb_client_request);
}

/// `BulletinBoardSystem.CheckCreateTime(time)`.
pub fn check_create_time(w: &World, time: DateTime) -> bool {
    time + settings(w).create < w.now
}

/// `BulletinBoardSystem.CheckDeletionTime(time)`.
pub fn check_deletion_time(w: &World, time: DateTime) -> bool {
    time + settings(w).deletion < w.now
}

/// `BulletinBoardSystem.CheckReplyTime(time)`.
pub fn check_reply_time(w: &World, time: DateTime) -> bool {
    time + settings(w).reply < w.now
}

// ---- BulletinMessage ----

/// What `BulletinMessage` adds to an `Item`.
#[derive(Clone, Debug, Default)]
pub struct MessageData {
    pub poster: Option<MobileId>,
    pub subject: Option<String>,
    pub time: DateTime,
    pub last_post_time: DateTime,
    pub thread: Option<ItemId>,
    pub posted_name: Option<String>,
    pub posted_body: i32,
    pub posted_hue: i32,
    /// `BulletinEquip`: `(itemID, hue)`.
    pub posted_equip: Vec<(i32, i32)>,
    pub lines: Vec<Option<String>>,
}

pub fn message(w: &World, id: ItemId) -> Option<&MessageData> {
    w.item(id)?.ext.get::<MessageData>()
}

fn message_mut(w: &mut World, id: ItemId) -> Option<&mut MessageData> {
    w.item_mut(id)?.ext.get_mut::<MessageData>()
}

pub fn is_message(w: &World, id: ItemId) -> bool {
    w.item(id).is_some_and(|i| i.ty.is(&BULLETIN_MESSAGE))
}

pub fn is_board(w: &World, id: ItemId) -> bool {
    w.item(id).is_some_and(|i| i.ty.is(&BASE_BULLETIN_BOARD))
}

/// `new BulletinMessage(poster, thread, subject, lines)`.
pub fn bulletin_message_new(
    w: &mut World,
    poster: MobileId,
    thread: Option<ItemId>,
    subject: String,
    lines: Vec<String>,
) -> ItemId {
    let id = crate::simple::from_template(w, &BULLETIN_MESSAGE, "Server.Items.BulletinMessage");
    let now = w.now;
    let (name, body, hue, items) = w.mobile(poster).map_or((None, 0, 0, Vec::new()), |m| {
        (
            (m.ty.name)(w, poster),
            muo_server::mobile::part1::props_b::body(w, poster).body_id(),
            (m.ty.hue)(w, poster),
            m.items.clone(),
        )
    });
    let mut equip = Vec::new();
    for item in items {
        if let Some(i) = w.item(item) {
            let l = i.layer.0 as i32;
            if l >= muo_server::layer::Layer::OneHanded.0 as i32
                && l <= muo_server::layer::Layer::Mount.0 as i32
            {
                equip.push((i.item_id, i.hue));
            }
        }
    }
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(MessageData {
            poster: Some(poster),
            subject: Some(subject),
            time: now,
            last_post_time: now,
            thread,
            posted_name: name,
            posted_body: body,
            posted_hue: hue,
            posted_equip: equip,
            lines: lines.into_iter().map(Some).collect(),
        });
    }
    id
}

/// `GetTimeAsString()`: `MMM dd, yyyy`.
pub fn time_as_string(t: DateTime) -> String {
    const MONTHS: [&str; 12] = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    let (y, m, d) = t.ymd();
    format!(
        "{} {:02}, {}",
        MONTHS[(m as usize).saturating_sub(1) % 12],
        d,
        y
    )
}

fn serialize_message(w: &World, id: ItemId, out: &mut dyn GenericWriter) {
    (ITEM_BASE.serialize)(w, id, out);
    out.write_i32(2);
    let d = message(w, id).cloned().unwrap_or_default();
    out.write_mobile(w, d.poster);
    out.write_opt_string(d.subject.as_deref());
    out.write_date_time_ticks(d.time.ticks());
    out.write_date_time_ticks(d.last_post_time.ticks());
    out.write_item(w, d.thread);
    out.write_opt_string(d.posted_name.as_deref());
    out.write_i32(d.posted_body);
    out.write_i32(d.posted_hue);
    out.write_encoded_int(d.posted_equip.len() as i32);
    for (item_id, hue) in d.posted_equip {
        out.write_i32(item_id);
        out.write_i32(hue);
    }
    out.write_encoded_int(d.lines.len() as i32);
    for l in d.lines {
        out.write_opt_string(l.as_deref());
    }
}

fn deserialize_message(w: &mut World, id: ItemId, r: &mut dyn GenericReader) -> muo_io::Result<()> {
    (ITEM_BASE.deserialize)(w, id, r)?;
    let version = r.read_i32()?;
    let mut d = MessageData::default();
    if version < 2 {
        d.poster = r.read_mobile(w)?;
        d.subject = r.read_string()?;
        d.time = DateTime::from_ticks(r.read_date_time_ticks()?);
        d.last_post_time = DateTime::from_ticks(r.read_date_time_ticks()?);
        r.read_bool()?; // Has thread
        d.thread = r.read_item(w)?;
        d.posted_name = r.read_string()?;
        d.posted_body = r.read_i32()?;
        d.posted_hue = r.read_i32()?;
        let n = r.read_i32()?;
        for _ in 0..n.max(0) {
            d.posted_equip.push((r.read_i32()?, r.read_i32()?));
        }
        let n = r.read_i32()?;
        for _ in 0..n.max(0) {
            d.lines.push(r.read_string()?);
        }
    } else {
        d.poster = r.read_mobile(w)?;
        d.subject = r.read_string()?;
        d.time = DateTime::from_ticks(r.read_date_time_ticks()?);
        d.last_post_time = DateTime::from_ticks(r.read_date_time_ticks()?);
        d.thread = r.read_item(w)?;
        d.posted_name = r.read_string()?;
        d.posted_body = r.read_i32()?;
        d.posted_hue = r.read_i32()?;
        let n = r.read_encoded_int()?;
        for _ in 0..n.max(0) {
            d.posted_equip.push((r.read_i32()?, r.read_i32()?));
        }
        let n = r.read_encoded_int()?;
        for _ in 0..n.max(0) {
            d.lines.push(r.read_string()?);
        }
    }
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(d);
    }
    Ok(())
}

/// `Server.Items.BulletinMessage` `[SerializationGenerator(2, false)]`.
pub static BULLETIN_MESSAGE: ItemType = ItemType {
    full_name: "Server.Items.BulletinMessage",
    parent: Some(&ITEM_BASE),
    construct: None,
    check_target: |_, _, _, _, _| false,
    is_accessible_to: |_, _, _| false,
    serialize: |w, id, out| serialize_message(w, id, out),
    deserialize: |w, id, r| deserialize_message(w, id, r),
    ..ITEM_BASE
};

// ---- BaseBulletinBoard ----

/// The virtual `HandleBBRequest(packetID, from, reader)` of `BaseBulletinBoard`; `true` handles the request.
pub struct BoardClass {
    pub handle_bb_request: fn(&mut World, ItemId, u8, MobileId, &mut SpanReader) -> bool,
}

pub static BASE_BOARD_CLASS: BoardClass = BoardClass {
    handle_bb_request: |_, _, _, _, _| false,
};

fn board_class(w: &World, id: ItemId) -> &'static BoardClass {
    w.item(id)
        .and_then(|i| i.ty.family::<BoardClass>())
        .unwrap_or(&BASE_BOARD_CLASS)
}

#[derive(Clone, Debug, Default)]
pub struct BoardData {
    pub board_name: Option<String>,
}

pub fn board_name(w: &World, id: ItemId) -> Option<String> {
    w.item(id)?.ext.get::<BoardData>()?.board_name.clone()
}

/// `BaseBulletinBoard(itemID)`.
pub fn base_board_new(w: &mut World, ty: &'static ItemType, item_id: i32) -> ItemId {
    let id = ops::new_item_with_id(w, ty, item_id);
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(BoardData {
            board_name: Some("bulletin board".to_string()),
        });
    }
    ops::set_movable(w, id, false);
    id
}

fn messages_of(w: &World, board: ItemId) -> Vec<ItemId> {
    w.item(board).map(|i| i.items.clone()).unwrap_or_default()
}

/// `BaseBulletinBoard.Cleanup()`: expired top-level threads and everything under them go.
pub fn cleanup(w: &mut World, board: ItemId) {
    let mut queue: Vec<ItemId> = Vec::new();
    let items = messages_of(w, board);
    for &item in items.iter().rev() {
        if !is_message(w, item) || ops::is_deleted(w, item) {
            continue;
        }
        let d = message(w, item).cloned().unwrap_or_default();
        // Top level thread expired
        if d.thread.is_none() && check_deletion_time(w, d.last_post_time) {
            queue.push(item);
        }
    }
    let mut at = 0;
    while at < queue.len() {
        let msg = queue[at];
        at += 1;
        // RecurseDelete: all messages and sub-threads are also in the same bulletin board container
        for &check in messages_of(w, board).iter().rev() {
            if !is_message(w, check) || ops::is_deleted(w, check) {
                continue;
            }
            if message(w, check).and_then(|d| d.thread) == Some(msg) {
                queue.push(check);
            }
        }
        if let Some(ty) = w.item(msg).map(|i| i.ty) {
            (ty.delete)(w, msg);
        }
    }
}

/// `GetLastPostTime(poster, onlyCheckRoot, out lastPostTime)`.
pub fn get_last_post_time(
    w: &World,
    board: ItemId,
    poster: MobileId,
    only_check_root: bool,
) -> Option<DateTime> {
    let mut last = DateTime::MIN;
    let mut was_set = false;
    for item in messages_of(w, board) {
        let Some(d) = message(w, item) else { continue };
        if d.poster != Some(poster) {
            continue;
        }
        if only_check_root && d.thread.is_some() {
            continue;
        }
        if d.time > last {
            was_set = true;
            last = d.time;
        }
    }
    was_set.then_some(last)
}

/// `CheckRange(from)`.
pub fn check_range(w: &World, board: ItemId, from: MobileId) -> bool {
    let Some(m) = w.mobile(from) else {
        return false;
    };
    if m.access_level >= AccessLevel::GameMaster {
        return true;
    }
    let loc = ops::get_world_location(w, board);
    m.map == w.item(board).and_then(|i| i.map) && deps::mobile_in_range(w, from, loc, 2)
}

/// `PostMessage(from, thread, subject, lines)`.
pub fn post_message(
    w: &mut World,
    board: ItemId,
    from: MobileId,
    thread: Option<ItemId>,
    subject: String,
    lines: Vec<String>,
) {
    let now = w.now;
    if let Some(d) = thread.and_then(|t| message_mut(w, t)) {
        d.last_post_time = now;
    }
    let msg = bulletin_message_new(w, from, thread, subject, lines);
    if let Some(ty) = w.item(board).map(|i| i.ty) {
        (ty.add_item)(w, board, msg);
    }
}

fn board_double_click(w: &mut World, id: ItemId, from: MobileId) {
    if !check_range(w, id, from) {
        muo_server::mobile::part2::messages::local_overhead_message_number(
            w,
            from,
            MessageType::REGULAR,
            0x3B2,
            1019045,
            "",
        );
        return;
    }
    cleanup(w, id);
    if let Some(ns) = deps::net_state_of(w, from) {
        send_bb_display_board(w, ns, id);
        muo_server::packets::container::send_container_content(w, ns, from, id);
    }
}

/// `Server.Items.BaseBulletinBoard` `[SerializationGenerator(0, false)]` (abstract).
pub static BASE_BULLETIN_BOARD: ItemType = ItemType {
    full_name: "Server.Items.BaseBulletinBoard",
    parent: Some(&ITEM_BASE),
    construct: None,
    family: Some(&BASE_BOARD_CLASS),
    on_double_click: board_double_click,
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        write_version(out, false);
        out.write_opt_string(board_name(w, id).as_deref());
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        read_version(r, false)?;
        let name = r.read_string()?;
        if let Some(i) = w.item_mut(id) {
            i.ext = Ext::new(BoardData { board_name: name });
        }
        // [AfterDeserialization(false)] Cleanup
        muo_server::timer::start(w, TimeSpan::ZERO, TimeSpan::ZERO, 1, move |w| {
            cleanup(w, id)
        });
        Ok(())
    },
    ..ITEM_BASE
};

/// `Server.Items.BulletinBoard`; `[Flippable(0x1E5E, 0x1E5F)]`.
pub static BULLETIN_BOARD: ItemType = ItemType {
    full_name: "Server.Items.BulletinBoard",
    parent: Some(&BASE_BULLETIN_BOARD),
    construct: Some(|w| base_board_new(w, &BULLETIN_BOARD, 0x1E5E)),
    serialize: |w, id, out| {
        (BASE_BULLETIN_BOARD.serialize)(w, id, out);
        write_version(out, false);
    },
    deserialize: |w, id, r| {
        (BASE_BULLETIN_BOARD.deserialize)(w, id, r)?;
        read_version(r, false)?;
        Ok(())
    },
    ..BASE_BULLETIN_BOARD
};

/// `[Flippable(0x1E5E, 0x1E5F)]` of `BulletinBoard`.
pub static FLIPPABLE: &[(&ItemType, i32, i32)] = &[(&BULLETIN_BOARD, 0x1E5E, 0x1E5F)];

// ---- packets (BulletinBoardPackets) ----

/// `FormatTS(ts)`.
pub fn format_ts(ts: TimeSpan) -> String {
    let total = ts.total_seconds().round_ties_even() as i32;
    let (minutes, seconds) = (total / 60, total % 60);
    let plural = |n: i32| if n == 1 { "" } else { "s" };
    if minutes != 0 && seconds != 0 {
        format!(
            "{minutes} minute{} and {seconds} second{}",
            plural(minutes),
            plural(seconds)
        )
    } else if minutes != 0 {
        format!("{minutes} minute{}", plural(minutes))
    } else {
        format!("{seconds} second{}", plural(seconds))
    }
}

fn on_bb_client_request(w: &mut World, ns: &mut muo_net::net_state::NetState, body: &[u8]) {
    let id = muo_server::net_glue::ensure_net_state(w, ns);
    let Some(from) = w.net_state(id).and_then(|d| d.mobile) else {
        return;
    };
    let mut r = SpanReader::new(body);
    let (Ok(packet_id), Ok(serial)) = (r.read_byte(), r.read_u32()) else {
        return;
    };
    let Some(board) = w
        .find_item(muo_primitives::Serial::new(serial))
        .filter(|b| is_board(w, *b))
    else {
        return;
    };
    if !check_range(w, board, from) {
        return;
    }
    if (board_class(w, board).handle_bb_request)(w, board, packet_id, from, &mut r) {
        return;
    }
    match packet_id {
        3 => bb_request_content(w, from, board, &mut r),
        4 => bb_request_header(w, from, board, &mut r),
        5 => bb_post_message(w, from, board, &mut r),
        6 => bb_remove_message(w, from, board, &mut r),
        _ => {}
    }
}

fn read_message(w: &World, board: ItemId, r: &mut SpanReader) -> Option<ItemId> {
    let serial = r.read_u32().ok()?;
    let msg = w.find_item(muo_primitives::Serial::new(serial))?;
    (is_message(w, msg) && w.item(msg).and_then(|i| i.parent) == Some(EntityId::Item(board)))
        .then_some(msg)
}

fn bb_request_content(w: &mut World, from: MobileId, board: ItemId, r: &mut SpanReader) {
    let Some(msg) = read_message(w, board, r) else {
        return;
    };
    if let Some(ns) = deps::net_state_of(w, from) {
        send_bb_message(w, ns, board, msg, true);
    }
}

fn bb_request_header(w: &mut World, from: MobileId, board: ItemId, r: &mut SpanReader) {
    let Some(msg) = read_message(w, board, r) else {
        return;
    };
    if let Some(ns) = deps::net_state_of(w, from) {
        send_bb_message(w, ns, board, msg, false);
    }
}

fn bb_post_message(w: &mut World, from: MobileId, board: ItemId, r: &mut SpanReader) {
    let Ok(thread_serial) = r.read_u32() else {
        return;
    };
    let mut thread = w
        .find_item(muo_primitives::Serial::new(thread_serial))
        .filter(|t| is_message(w, *t));
    if let Some(t) = thread
        && w.item(t).and_then(|i| i.parent) != Some(EntityId::Item(board))
    {
        thread = None;
    }
    let mut breakout = 0;
    while let Some(up) = thread.and_then(|t| message(w, t)).and_then(|d| d.thread) {
        if breakout >= 10 {
            break;
        }
        breakout += 1;
        thread = Some(up);
    }
    if let Some(last) = get_last_post_time(w, board, from, thread.is_none()) {
        if thread.is_none() {
            if !check_create_time(w, last) {
                let s = format!(
                    "You must wait {} before creating a new thread.",
                    format_ts(settings(w).create)
                );
                deps::send_message(w, from, &s);
                return;
            }
        } else if !check_reply_time(w, last) {
            let s = format!(
                "You must wait {} before replying to another thread.",
                format_ts(settings(w).reply)
            );
            deps::send_message(w, from, &s);
            return;
        }
    }
    let Ok(len) = r.read_byte() else { return };
    let Ok(subject) = r.read_utf8_safe(Some(usize::from(len))) else {
        return;
    };
    if subject.is_empty() {
        return;
    }
    let Ok(count) = r.read_byte() else { return };
    if count == 0 {
        return;
    }
    let mut lines = Vec::with_capacity(usize::from(count));
    for _ in 0..count {
        let Ok(len) = r.read_byte() else { return };
        let Ok(line) = r.read_utf8_safe(Some(usize::from(len))) else {
            return;
        };
        lines.push(line);
    }
    post_message(w, board, from, thread, subject, lines);
}

fn bb_remove_message(w: &mut World, from: MobileId, board: ItemId, r: &mut SpanReader) {
    let Some(msg) = read_message(w, board, r) else {
        return;
    };
    let gm = w
        .mobile(from)
        .is_some_and(|m| m.access_level >= AccessLevel::GameMaster);
    if !gm && message(w, msg).and_then(|d| d.poster) != Some(from) {
        return;
    }
    if let Some(ty) = w.item(msg).map(|i| i.ty) {
        (ty.delete)(w, msg);
    }
}

/// `SendBBDisplayBoard(board)`.
pub fn send_bb_display_board(w: &mut World, ns: NetStateId, board: ItemId) {
    if w.cannot_send_packets(ns) {
        return;
    }
    let Some(serial) = w.item(board).map(|i| i.serial) else {
        return;
    };
    let mut p = Pk::new(38);
    p.u8(0x71); // Packet ID
    p.u16(38);
    p.u8(0x00); // Command
    p.serial(serial);
    let name = board_name(w, board).unwrap_or_default();
    // text.AsSpan(0, Math.Min(text.Length, 30)): at most 30 UTF-16 units, cut off mid character if need be.
    let units: Vec<u16> = name.encode_utf16().take(30).collect();
    let text = String::from_utf16_lossy(&units);
    let bytes = text.as_bytes();
    let n = bytes.len().min(29);
    p.bytes(&bytes[..n]);
    p.zeros(30 - n); // terminator
    let out = p.finish();
    w.send(ns, &out);
}

/// `WriteString(text, buffer, pad)`: a length byte, at most 254 (253 padded) UTF-8 bytes, then a terminator (two
/// zero bytes for the padded form, which compensates for an old client bug).
fn write_bb_string(p: &mut Pk, text: &str, pad: bool) {
    let tail = if pad { 2 } else { 1 };
    let max = if pad { 253 } else { 254 };
    let b = text.as_bytes();
    let n = b.len().min(max);
    p.u8((n + tail) as u8);
    p.bytes(&b[..n]);
    if pad {
        p.u16(0);
    } else {
        p.u8(0);
    }
}

/// `SendBBMessage(board, msg, content)`.
pub fn send_bb_message(w: &mut World, ns: NetStateId, board: ItemId, msg: ItemId, content: bool) {
    if w.cannot_send_packets(ns) {
        return;
    }
    let (Some(bs), Some(ms), Some(d)) = (
        w.item(board).map(|i| i.serial),
        w.item(msg).map(|i| i.serial),
        message(w, msg).cloned(),
    ) else {
        return;
    };
    let mut p = Pk::new(64);
    p.u8(0x71); // Packet ID
    p.u16(0); // length
    p.u8(if content { 0x02 } else { 0x01 }); // Command
    p.serial(bs);
    p.serial(ms);
    if !content {
        p.serial(
            d.thread
                .and_then(|t| w.item(t))
                .map_or(muo_primitives::Serial::ZERO, |t| t.serial),
        );
    }
    write_bb_string(&mut p, d.posted_name.as_deref().unwrap_or(""), false);
    write_bb_string(&mut p, d.subject.as_deref().unwrap_or(""), false);
    write_bb_string(&mut p, &time_as_string(d.time), false);
    if content {
        let equip_len = d.posted_equip.len().min(255);
        let lines_len = d.lines.len().min(255);
        p.i16(d.posted_body as i16);
        p.i16(d.posted_hue as i16);
        p.u8(equip_len as u8);
        for &(item_id, hue) in d.posted_equip.iter().take(equip_len) {
            p.i16(item_id as i16);
            p.i16(hue as i16);
        }
        p.u8(lines_len as u8);
        for l in d.lines.iter().take(lines_len) {
            write_bb_string(&mut p, l.as_deref().unwrap_or(""), true);
        }
    }
    let out = p.finish_with_length();
    w.send(ns, &out);
}

// ---- registration ----

/// All the classes of this folder.
pub static ALL: &[&ItemType] = &[&BULLETIN_MESSAGE, &BASE_BULLETIN_BOARD, &BULLETIN_BOARD];

/// Registers the classes of this folder.
pub fn register(types: &mut TypeRegistry) {
    for t in ALL {
        types.register_item(t, &[]);
    }
    types.register_phase_default(
        Phase::Configure,
        "Server.Items.BulletinBoardSystem.Configure",
        configure,
    );
}
