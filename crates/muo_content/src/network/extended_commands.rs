//! Incoming extended commands: packet 0xBF and its table of
//! sub-commands. Other content registers its own with [`register_extended`] (the context menu's 0x13/0x15 are here).
//!
//! The systems a sub-command calls that live elsewhere (party, spellbooks, fists, bandages) are the `fn` fields of
//! [`GameHooks`]; an unset one does nothing.

use muo_io::SpanReader;
use muo_net::net_state::{Attachment, NetState};
use muo_primitives::Serial;
use muo_server::World;
use muo_server::ids::{ItemId, MobileId};
use muo_server::mobile::part2::stats::stat_lock_from_u8;
use muo_server::mobile::{self, AccessLevel};

use super::incoming_game::{GameHooks, begin};
use super::{events, register_packet};

type Handler = fn(&mut World, &mut NetState, &[u8]);

#[derive(Copy, Clone)]
struct Entry {
    in_game: bool,
    out_game: bool,
    on_receive: Handler,
}

/// `_extendedHandlers`.
pub struct ExtendedHandlers(Vec<Option<Entry>>);

impl Default for ExtendedHandlers {
    fn default() -> Self {
        Self(vec![None; 0x100])
    }
}

impl ExtendedHandlers {
    /// The sub-command ids that have a handler (the orphan tests check them against the expected registrations).
    pub fn ids(&self) -> Vec<u8> {
        (0..=255u8)
            .filter(|&i| self.0[usize::from(i)].is_some())
            .collect()
    }
}

/// `ValidAnimations`.
pub const VALID_ANIMATIONS: [i32; 32] = [
    6, 21, 32, 33, 100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115,
    116, 117, 118, 119, 120, 121, 123, 124, 125, 126, 127, 128,
];

/// `RegisterExtended(packetID, ingame, outgame, onReceive)`.
pub fn register_extended_full(w: &mut World, id: i32, in_game: bool, out_game: bool, f: Handler) {
    if (0..0x100).contains(&id) {
        w.resources.get_mut::<ExtendedHandlers>().0[id as usize] = Some(Entry {
            in_game,
            out_game,
            on_receive: f,
        });
    }
}

/// `RegisterExtended(packetID, ingame, onReceive)`.
pub fn register_extended(w: &mut World, id: i32, in_game: bool, f: Handler) {
    register_extended_full(w, id, in_game, false, f);
}

/// `RemoveExtendedHandler`.
pub fn remove_extended_handler(w: &mut World, id: i32) {
    if (0..0x100).contains(&id) {
        w.resources.get_mut::<ExtendedHandlers>().0[id as usize] = None;
    }
}

/// `[GeneratedEvent("TargetByResourceMacro")]`.
pub struct TargetByResourceMacro {
    pub mobile: Option<MobileId>,
    pub item: Option<ItemId>,
    pub resource_type: i16,
}

pub fn configure(w: &mut World) {
    register_packet(w, 0xBF, 0, true, false, extended_command);

    register_extended(w, 0x05, false, |_, _, _| {}); // ScreenSize: reads two ints
    register_extended(w, 0x06, true, party_message);
    register_extended(w, 0x09, true, disarm_request);
    register_extended(w, 0x0A, true, stun_request);
    register_extended(w, 0x0B, false, language);
    register_extended(w, 0x0C, true, |_, _, _| {}); // CloseStatus: reads a serial
    register_extended(w, 0x0E, true, animate);
    register_extended(w, 0x0F, false, |_, _, _| {});
    register_extended(w, 0x10, true, query_properties);
    register_extended(w, 0x1A, true, stat_lock_change);
    register_extended(w, 0x1C, true, cast_spell);
    register_extended(w, 0x24, false, |_, _, _| {}); // UnhandledBF
    register_extended(w, 0x2C, true, bandage_target);
    register_extended(w, 0x2D, true, targeted_spell);
    register_extended(w, 0x2E, true, targeted_skill_use);
    register_extended(w, 0x30, true, target_by_resource_macro);
    register_extended(w, 0x32, true, toggle_flying);
    // ContextMenuSystem.Configure
    register_extended(
        w,
        0x13,
        true,
        crate::systems::context_menus::context_menu_request,
    );
    register_extended(
        w,
        0x15,
        true,
        crate::systems::context_menus::context_menu_response,
    );
}

fn extended_command(w: &mut World, ns: &mut NetState, body: &[u8]) {
    let Ok(packet_id) = SpanReader::new(body).read_u16() else {
        return;
    };
    let entry = w
        .resources
        .get::<ExtendedHandlers>()
        .and_then(|t| t.0.get(usize::from(packet_id)).copied().flatten());
    let Some(entry) = entry else {
        ns.trace(body);
        return;
    };
    if entry.in_game {
        match ns.attachment {
            Attachment::None => {
                ns.disconnect(&format!(
                    "Received packet 0x{packet_id:02X} before having been attached to a mobile."
                ));
                return;
            }
            Attachment::Deleted => {
                ns.disconnect(&format!(
                    "Received packet 0x{packet_id:02X} after having been attached to a deleted mobile."
                ));
                return;
            }
            Attachment::Mobile(_) => {}
        }
    }
    if entry.out_game && matches!(ns.attachment, Attachment::Mobile(_)) {
        ns.disconnect(&format!(
            "Received packet 0x{packet_id:02X} after having been attached to a mobile."
        ));
        return;
    }
    (entry.on_receive)(w, ns, &body[2..]);
}

fn hooks(w: &World) -> Option<&GameHooks> {
    w.resources.get::<GameHooks>()
}

fn party_message(w: &mut World, ns: &mut NetState, body: &[u8]) {
    let (_, from) = begin(w, ns);
    let Some(from) = from else { return };
    let Some(p) = hooks(w).and_then(|h| h.party) else {
        return;
    };
    let mut r = SpanReader::new(body);
    let Ok(sub) = r.read_byte() else { return };
    let mobile_of = |w: &World, r: &mut SpanReader| {
        r.read_u32()
            .ok()
            .and_then(|s| w.find_mobile(Serial::new(s)))
    };
    match sub {
        0x01 => (p.on_add)(w, from),
        0x02 => {
            let m = mobile_of(w, &mut r);
            (p.on_remove)(w, from, m)
        }
        0x03 => {
            let m = mobile_of(w, &mut r);
            let text = r.read_big_uni_safe(None).unwrap_or_default();
            (p.on_private_message)(w, from, m, &text)
        }
        0x04 => {
            let text = r.read_big_uni_safe(None).unwrap_or_default();
            (p.on_public_message)(w, from, &text)
        }
        0x06 => {
            if let Ok(b) = r.read_boolean() {
                (p.on_set_can_loot)(w, from, b)
            }
        }
        0x08 => {
            let m = mobile_of(w, &mut r);
            (p.on_accept)(w, from, m)
        }
        0x09 => {
            let m = mobile_of(w, &mut r);
            (p.on_decline)(w, from, m)
        }
        _ => ns.trace(body),
    }
}

fn animate(w: &mut World, ns: &mut NetState, body: &[u8]) {
    let (_, from) = begin(w, ns);
    let Some(from) = from else { return };
    let Ok(action) = SpanReader::new(body).read_i32() else {
        return;
    };
    let Some(m) = w.mobile(from) else { return };
    let (ty, body_id) = (m.ty, muo_server::mobile::part1::props_b::body(w, from));
    if VALID_ANIMATIONS.contains(&action)
        && (ty.alive)(w, from)
        && w.resources
            .get::<muo_primitives::BodyTable>()
            .is_some_and(|t| t.is_human(body_id))
        && !mobile::part1::props_b::mounted(w, from)
    {
        mobile::base::animate(w, from, action, 7, 1, true, false, 0);
    }
}

fn cast_spell(w: &mut World, ns: &mut NetState, body: &[u8]) {
    let (_, from) = begin(w, ns);
    let Some(from) = from else { return };
    let mut r = SpanReader::new(body);
    let Ok(has_book) = r.read_i16() else { return };
    let book = if has_book == 1 {
        let Ok(s) = r.read_u32() else { return };
        w.find_item(Serial::new(s))
    } else {
        None
    };
    let Ok(spell) = r.read_i16() else { return };
    if let Some(f) = hooks(w).and_then(|h| h.cast_spell_request) {
        f(w, from, i32::from(spell) - 1, book);
    }
}

fn toggle_flying(w: &mut World, ns: &mut NetState, _body: &[u8]) {
    let (_, from) = begin(w, ns);
    if let Some(from) = from {
        crate::player_mobile::members::toggle_flying(w, from);
    }
}

fn stun_request(w: &mut World, ns: &mut NetState, _body: &[u8]) {
    let (_, from) = begin(w, ns);
    if let (Some(from), Some(f)) = (from, hooks(w).and_then(|h| h.fists_stun_request)) {
        f(w, from);
    }
}

fn disarm_request(w: &mut World, ns: &mut NetState, _body: &[u8]) {
    let (_, from) = begin(w, ns);
    if let (Some(from), Some(f)) = (from, hooks(w).and_then(|h| h.fists_disarm_request)) {
        f(w, from);
    }
}

fn stat_lock_change(w: &mut World, ns: &mut NetState, body: &[u8]) {
    let (_, from) = begin(w, ns);
    let Some(from) = from else { return };
    let mut r = SpanReader::new(body);
    let (Ok(stat), Ok(lock)) = (r.read_byte(), r.read_byte()) else {
        return;
    };
    let lock = stat_lock_from_u8(if lock > 2 { 0 } else { lock });
    use mobile::part1::props_a::{set_dex_lock, set_int_lock, set_str_lock};
    match stat {
        0 => set_str_lock(w, from, lock),
        1 => set_dex_lock(w, from, lock),
        2 => set_int_lock(w, from, lock),
        _ => {}
    }
}

fn language(w: &mut World, ns: &mut NetState, body: &[u8]) {
    let (_, from) = begin(w, ns);
    let Some(from) = from else { return };
    if let Ok(l) = SpanReader::new(body).read_ascii(Some(4))
        && let Some(m) = w.mobile_mut(from)
    {
        m.language = Some(l);
    }
}

fn query_properties(w: &mut World, ns: &mut NetState, body: &[u8]) {
    if !w.core.opl_enabled {
        return;
    }
    let (id, from) = begin(w, ns);
    let Some(from) = from else { return };
    let Ok(v) = SpanReader::new(body).read_u32() else {
        return;
    };
    let s = Serial::new(v);
    let floc = w.mobile(from).map(|m| m.location).unwrap_or_default();
    if s.is_mobile() {
        if let Some(m) = w.find_mobile(s)
            && mobile::base::can_see_mobile(w, from, m)
            && mobile::part1::combat::in_update_range(
                floc,
                w.mobile(m).map(|mm| mm.location).unwrap_or_default(),
            )
        {
            mobile::base::send_properties_to(w, m, id);
        }
    } else if s.is_item()
        && let Some(i) = w
            .find_item(s)
            .filter(|i| w.item(*i).is_some_and(|ii| !ii.deleted()))
        && mobile::base::can_see_item(w, from, i)
        && mobile::part1::combat::in_update_range(
            floc,
            muo_server::item::ops::get_world_location(w, i),
        )
    {
        muo_server::item::base::send_properties_to(w, i, id);
    }
}

fn bandage_target(w: &mut World, ns: &mut NetState, body: &[u8]) {
    let (_, from) = begin(w, ns);
    let Some(from) = from else { return };
    let Some(m) = w.mobile(from) else { return };
    let (access, next) = (m.access_level, m.next_action_time);
    let tick = w.tick_count;
    if access >= AccessLevel::Counselor || tick.wrapping_sub(next) >= 0 {
        let mut r = SpanReader::new(body);
        let (Ok(b), Ok(t)) = (r.read_u32(), r.read_u32()) else {
            return;
        };
        let Some(bandage) = w.find_item(Serial::new(b)) else {
            return;
        };
        let Some(target) = w.find_mobile(Serial::new(t)) else {
            return;
        };
        if let Some(f) = hooks(w).and_then(|h| h.bandage_target_request) {
            f(w, from, bandage, target);
        }
        let delay = i64::from(mobile::part1::props_a::action_delay(w));
        if let Some(m) = w.mobile_mut(from) {
            m.next_action_time = tick + delay;
        }
    } else {
        mobile::base::send_action_message(w, from);
    }
}

fn targeted_spell(w: &mut World, ns: &mut NetState, body: &[u8]) {
    let (_, from) = begin(w, ns);
    let mut r = SpanReader::new(body);
    let (Ok(spell), Ok(s)) = (r.read_i16(), r.read_u32()) else {
        return;
    };
    let Some(from) = from else { return };
    let e = w.find_entity(Serial::new(s));
    if let Some(f) = hooks(w).and_then(|h| h.targeted_spell) {
        f(w, from, e, spell.wrapping_sub(1));
    }
}

fn targeted_skill_use(w: &mut World, ns: &mut NetState, body: &[u8]) {
    let (_, from) = begin(w, ns);
    let mut r = SpanReader::new(body);
    let (Ok(skill), Ok(s)) = (r.read_i16(), r.read_u32()) else {
        return;
    };
    let Some(from) = from else { return };
    let e = w.find_entity(Serial::new(s));
    crate::player_mobile::members::targeted_skill_use(w, from, e, i32::from(skill));
}

fn target_by_resource_macro(w: &mut World, ns: &mut NetState, body: &[u8]) {
    let (_, from) = begin(w, ns);
    let mut r = SpanReader::new(body);
    let Ok(s) = r.read_u32() else { return };
    let serial = Serial::new(s);
    if serial.is_item() {
        let item = w.find_item(serial);
        let Ok(resource_type) = r.read_i16() else {
            return;
        };
        events::invoke(
            w,
            &mut TargetByResourceMacro {
                mobile: from,
                item,
                resource_type,
            },
        );
    }
}

/// The `AssistantProtocol` sub-command ids that have a handler (the orphan tests check them).
pub fn assistant_protocol_ids(w: &World) -> Vec<u8> {
    w.resources
        .get::<crate::systems::assistants::assistant_protocol::AssistantProtocol>()
        .map(|a| a.ids())
        .unwrap_or_default()
}
