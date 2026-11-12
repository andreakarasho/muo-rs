//! The gump system and the
//! `SendCloseGump`/`SendDisplaySignGump` packets: which gumps each connection has open, sending,
//! finding and closing them, and the 0xB1 response.
//!
//! The feature-flag block of `SendGump` reads [`GumpBlocks`] and the virtue gump response calls [`GumpHooks`]; the
//! feature-flag and virtue systems fill them.

use std::any::Any;

use indexmap::IndexMap;
use muo_io::SpanReader;
use muo_net::net_state::NetState;
use muo_primitives::Serial;
use muo_server::World;
use muo_server::ids::MobileId;
use muo_server::net_state::NetStateId;
use tracing::{info, warn};

use super::base::{
    DynamicGumpLogic, DynamicGumpObj, Gump, LegacyGumpObj, RelayInfo, StaticGumpObj,
};

/// `NetStateGumps.GumpCap`.
const GUMP_CAP: usize = 512;

/// `GumpSystem._gumps` and `BaseGump`'s serial counter.
pub struct GumpSystem {
    gumps: IndexMap<NetStateId, Vec<Box<dyn Gump>>>,
    next_serial: u32,
}

impl Default for GumpSystem {
    fn default() -> Self {
        Self {
            gumps: IndexMap::new(),
            next_serial: 1,
        }
    }
}

fn system(w: &mut World) -> &mut GumpSystem {
    w.resources.get_mut::<GumpSystem>()
}

/// `GumpSystem.Configure`.
pub fn configure(w: &mut World) {
    crate::network::register_packet(w, 0xB1, 0, true, false, display_gump_response);
    w.events
        .before_disconnected
        .subscribe(|w, m| on_disconnected(w, *m));
}

/// `EventSink_Disconnected` (subscribed to `BeforeDisconnected`).
fn on_disconnected(w: &mut World, m: MobileId) {
    if let Some(ns) = w.mobile(m).and_then(|mm| mm.net_state) {
        system(w).gumps.shift_remove(&ns);
    }
}

/// A gump of the logic type `L`, whichever of the three wrappers holds it.
pub fn downcast_logic<L: 'static>(g: &dyn Gump) -> Option<&L> {
    let any: &dyn Any = g.as_any();
    if let Some(d) = any.downcast_ref::<DynamicGumpObj<L>>() {
        return Some(&d.logic);
    }
    if let Some(d) = any.downcast_ref::<StaticGumpObj<L>>() {
        return Some(&d.logic);
    }
    if let Some(d) = any.downcast_ref::<LegacyGumpObj<L>>() {
        return Some(&d.logic);
    }
    None
}
fn is_logic<L: 'static>(g: &dyn Gump) -> bool {
    downcast_logic::<L>(g).is_some()
}

/// `HasGump<T>()`.
pub fn has_gump<L: 'static>(w: &World, ns: NetStateId) -> bool {
    find_gump::<L>(w, ns).is_some()
}

/// `FindGump<T>()`: the gump's logic.
pub fn find_gump<L: 'static>(w: &World, ns: NetStateId) -> Option<&L> {
    w.resources
        .get::<GumpSystem>()?
        .gumps
        .get(&ns)?
        .iter()
        .find_map(|g| downcast_logic::<L>(g.as_ref()))
}

/// `FindGump<T>()`, mutable.
pub fn find_gump_mut<L: 'static>(w: &mut World, ns: NetStateId) -> Option<&mut dyn Gump> {
    system(w)
        .gumps
        .get_mut(&ns)?
        .iter_mut()
        .find(|g| is_logic::<L>(g.as_ref()))
        .map(|g| g.as_mut())
}

/// The gumps open on a connection, in the order they were sent.
pub fn gumps_of(w: &World, ns: NetStateId) -> impl Iterator<Item = &dyn Gump> {
    w.resources
        .get::<GumpSystem>()
        .and_then(|s| s.gumps.get(&ns))
        .into_iter()
        .flat_map(|l| l.iter().map(|g| g.as_ref()))
}

/// `SendCloseGump(typeId, buttonId)`.
pub fn send_close_gump(w: &mut World, ns: NetStateId, type_id: i32, button_id: i32) {
    if w.cannot_send_packets(ns) {
        return;
    }
    let mut p = vec![0xBF, 0x00, 13, 0x00, 0x04];
    p.extend_from_slice(&type_id.to_be_bytes());
    p.extend_from_slice(&button_id.to_be_bytes());
    w.send(ns, &p);
}

/// `SendDisplaySignGump(serial, gumpId, unknown, caption)`.
pub fn send_display_sign_gump(
    w: &mut World,
    ns: NetStateId,
    serial: muo_primitives::Serial,
    gump_id: i32,
    unknown: &str,
    caption: &str,
) {
    if w.cannot_send_packets(ns) {
        return;
    }
    let latin1 = |s: &str| -> Vec<u8> {
        s.chars()
            .map(|c| if (c as u32) < 0x100 { c as u8 } else { b'?' })
            .collect()
    };
    let (unknown, caption) = (latin1(unknown), latin1(caption));
    let length = 15 + unknown.len() + caption.len();
    let mut p = vec![0x8B];
    p.extend_from_slice(&(length as u16).to_be_bytes());
    p.extend_from_slice(&serial.value().to_be_bytes());
    p.extend_from_slice(&(gump_id as i16).to_be_bytes());
    p.extend_from_slice(&((unknown.len() + 1) as i16).to_be_bytes());
    p.extend_from_slice(&unknown);
    p.push(0);
    p.extend_from_slice(&((caption.len() + 1) as i16).to_be_bytes());
    p.extend_from_slice(&caption);
    p.push(0);
    w.send(ns, &p);
}

/// `NetStateGumps.Close<T>()`: true if one was open.
pub fn close_gump<L: 'static>(w: &mut World, ns: NetStateId) -> bool {
    let Some(i) = system(w)
        .gumps
        .get(&ns)
        .and_then(|l| l.iter().position(|g| is_logic::<L>(g.as_ref())))
    else {
        return false;
    };
    let mut g = system(w).gumps.get_mut(&ns).expect("found above").remove(i);
    send_close_gump(w, ns, g.core().type_id, 0);
    g.on_server_close(w, ns);
    true
}

/// `NetStateGumps.Send(gump, singleton)`.
pub fn send_gump(w: &mut World, ns: NetStateId, mut gump: Box<dyn Gump>, singleton: bool) {
    if w.cannot_send_packets(ns) {
        return;
    }
    if gump.core().serial.value() == 0 {
        let s = system(w);
        gump.core_mut().serial = muo_primitives::Serial::new(s.next_serial);
        s.next_serial += 1;
    }

    if singleton || gump.singleton() {
        let type_id = gump.core().type_id;
        let at = system(w)
            .gumps
            .get(&ns)
            .and_then(|l| l.iter().position(|g| g.core().type_id == type_id));
        if let Some(i) = at {
            let mut old = std::mem::replace(
                &mut system(w).gumps.get_mut(&ns).expect("found above")[i],
                placeholder(),
            );
            send_close_gump(w, ns, old.core().type_id, 0);
            old.on_server_close(w, ns);
            send_to(w, ns, gump.as_mut());
            // the replacement goes where the old one was
            system(w).gumps.get_mut(&ns).expect("found above")[i] = gump;
            return;
        }
    }

    if system(w).gumps.get(&ns).map_or(0, Vec::len) >= GUMP_CAP {
        info!("Exceeded gump cap, disconnecting...");
        w.disconnect(ns, "Exceeded gump cap.");
        return;
    }

    send_to(w, ns, gump.as_mut());
    system(w).gumps.entry(ns).or_default().push(gump);
}

/// `FeatureFlagSettings.DefaultGumpBlockedMessage`.
pub const DEFAULT_GUMP_BLOCKED_MESSAGE: &str = "This feature is temporarily disabled.";

/// `FeatureFlagManager`'s gump blocks, which the feature-flag system fills: the gump class's full name (see
/// `Gump::type_name`) -> its reason (`None` for the default message), for active blocks only.
pub struct GumpBlocks {
    /// `FeatureFlagSettings.RequiredAccessLevel`.
    pub required_access_level: muo_server::mobile::AccessLevel,
    pub blocked: indexmap::IndexMap<String, Option<String>>,
}

impl Default for GumpBlocks {
    fn default() -> Self {
        Self {
            required_access_level: muo_server::mobile::AccessLevel::Administrator,
            blocked: indexmap::IndexMap::new(),
        }
    }
}

/// What the gump response reaches in other systems: `VirtueGump.RequestVirtueGump(beholder, beheld)`
/// and `VirtueGump.RequestVirtueItem(beholder, beheld, gumpId)`. Unset: nothing happens.
#[derive(Default)]
pub struct GumpHooks {
    pub request_virtue_gump: Option<fn(&mut World, MobileId, MobileId)>,
    pub request_virtue_item: Option<fn(&mut World, MobileId, MobileId, i32)>,
}

/// `m.SendGump(g, singleton)`.
pub fn send_gump_to_mobile(w: &mut World, m: MobileId, gump: Box<dyn Gump>, singleton: bool) {
    let blocked = w.resources.get::<GumpBlocks>().and_then(|b| {
        let access = w.mobile(m)?.access_level;
        (access < b.required_access_level)
            .then(|| b.blocked.get(gump.type_name()))
            .flatten()
            .map(|reason| {
                reason
                    .clone()
                    .unwrap_or_else(|| DEFAULT_GUMP_BLOCKED_MESSAGE.to_owned())
            })
    });
    if let Some(reason) = blocked {
        muo_server::mobile::part2::messages::send_message_hue(w, m, 0x22, &reason);
        return;
    }
    if let Some(ns) = w.mobile(m).and_then(|mm| mm.net_state) {
        send_gump(w, ns, gump, singleton);
    }
}

/// `BaseGump.SendTo(ns)`.
fn send_to(w: &mut World, ns: NetStateId, gump: &mut dyn Gump) {
    let packet = gump.compile();
    if !gump.core().has_visual_elements {
        warn!("Sending empty gump {}", gump.type_name());
    }
    w.send(ns, &packet);
}

/// A slot filler while the old gump is being closed.
fn placeholder() -> Box<dyn Gump> {
    struct Nothing;
    impl DynamicGumpLogic for Nothing {
        const TYPE_NAME: &'static str = "";
        fn build_layout(&self, _: &mut super::builder::DynamicGumpBuilder) {}
    }
    Box::new(DynamicGumpObj::new(0, 0, Nothing))
}

/// `InvalidGumpResponseException`: logged, the response is dropped.
fn invalid(ns: &NetState, reason: &str) {
    ns.log_info("Invalid gump response, disconnecting...");
    warn!("InvalidGumpResponseException: {reason}");
}

/// `DisplayGumpResponse` (0xB1).
pub fn display_gump_response(w: &mut World, ns: &mut NetState, body: &[u8]) {
    let id = NetStateId(ns.id() as u32);
    let mut r = SpanReader::new(body);
    let (Ok(serial), Ok(type_id), Ok(button_id)) = (r.read_u32(), r.read_i32(), r.read_i32())
    else {
        return;
    };

    let found = system(w).gumps.get(&id).and_then(|l| {
        l.iter()
            .position(|g| g.core().serial.value() == serial && g.core().type_id == type_id)
    });

    if let Some(at) = found {
        let (max_switches, max_text_entries, button_ok) = {
            let g = &system(w).gumps[&id][at];
            (
                g.core().switches,
                g.core().text_entries,
                g.has_button(button_id),
            )
        };
        if !button_ok {
            invalid(ns, &format!("Button {button_id} doesn't exist"));
            return;
        }

        let Ok(switch_count) = r.read_i32() else {
            return;
        };
        if switch_count < 0 || switch_count > max_switches {
            invalid(ns, &format!("Bad switch count {switch_count}"));
            return;
        }
        let mut switches = Vec::with_capacity(switch_count as usize);
        for _ in 0..switch_count {
            match r.read_i32() {
                Ok(s) => switches.push(s),
                Err(_) => return,
            }
        }

        let Ok(text_count) = r.read_i32() else { return };
        if text_count < 0 || text_count > max_text_entries {
            invalid(ns, &format!("Bad text entry count {text_count}"));
            return;
        }
        let mut texts = Vec::with_capacity(text_count as usize);
        for i in 0..text_count {
            let (Ok(text_id), Ok(text_length)) = (r.read_u16(), r.read_u16()) else {
                return;
            };
            if text_length > 239 {
                invalid(ns, &format!("Text entry {i} is too long ({text_length})"));
                return;
            }
            match r.read_big_uni_safe(Some(usize::from(text_length))) {
                Ok(s) => texts.push((text_id, s)),
                Err(_) => return,
            }
        }

        // Remove(state, baseGump)
        let mut gump = system(w)
            .gumps
            .get_mut(&id)
            .expect("found above")
            .remove(at);
        let info = RelayInfo::new(button_id, switches, texts);
        gump.on_response(w, id, &info);
    }

    if type_id == 461 {
        // Virtue gump
        let switch_count = if r.remaining() >= 4 {
            r.read_i32().unwrap_or(0)
        } else {
            0
        };
        let Some(beholder) = w.net_mobile(id) else {
            return;
        };
        if button_id == 1 && switch_count > 0 {
            let beheld = r
                .read_u32()
                .ok()
                .and_then(|s| w.find_mobile(Serial::new(s)))
                .filter(|m| w.mobile(*m).is_some_and(|m| m.player));
            if let (Some(beheld), Some(f)) = (
                beheld,
                w.resources
                    .get::<GumpHooks>()
                    .and_then(|h| h.request_virtue_gump),
            ) {
                f(w, beholder, beheld);
            }
        } else if let Some(beheld) = w.find_mobile(Serial::new(serial))
            && let Some(f) = w
                .resources
                .get::<GumpHooks>()
                .and_then(|h| h.request_virtue_item)
        {
            f(w, beholder, beheld, button_id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gumps::base::DynamicGumpLogic;
    use crate::gumps::builder::DynamicGumpBuilder;
    use muo_server::net_state::NetStateData;

    struct Hello(i32);
    impl DynamicGumpLogic for Hello {
        const TYPE_NAME: &'static str = "Server.Gumps.Hello";
        fn singleton(&self) -> bool {
            true
        }
        fn build_layout(&self, b: &mut DynamicGumpBuilder) {
            b.add_button(0, 0, 1, 2, 5);
            b.add_checkbox(0, 0, 1, 2, false, 9);
            b.add_text_entry(0, 0, 1, 1, 0, 4, "");
        }
    }

    fn world_with_state() -> (World, NetStateId) {
        let mut w = World::new();
        let ns = NetStateId(1);
        w.net_states.insert(ns, NetStateData::new(0));
        (w, ns)
    }

    fn sent(w: &mut World) -> Vec<Vec<u8>> {
        std::mem::take(&mut w.net_out)
            .into_iter()
            .filter_map(|o| match o {
                muo_server::net_state::NetOut::Send(_, b) => Some(b),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn singleton_replaces_and_close_sends_0xbf() {
        let (mut w, ns) = world_with_state();
        send_gump(
            &mut w,
            ns,
            Box::new(DynamicGumpObj::new(0, 0, Hello(1))),
            false,
        );
        let p = sent(&mut w);
        assert_eq!(p.len(), 1);
        assert_eq!(p[0][0], 0xDD);
        assert_eq!(find_gump::<Hello>(&w, ns).map(|h| h.0), Some(1));

        send_gump(
            &mut w,
            ns,
            Box::new(DynamicGumpObj::new(0, 0, Hello(2))),
            false,
        );
        let p = sent(&mut w);
        assert_eq!(p.len(), 2, "close of the old one, then the new one");
        assert_eq!(&p[0][..5], &[0xBF, 0, 13, 0, 4]);
        assert_eq!(p[1][0], 0xDD);
        assert_eq!(gumps_of(&w, ns).count(), 1);
        assert_eq!(find_gump::<Hello>(&w, ns).map(|h| h.0), Some(2));

        assert!(close_gump::<Hello>(&mut w, ns));
        assert!(!has_gump::<Hello>(&w, ns));
        assert!(!close_gump::<Hello>(&mut w, ns));
    }

    #[test]
    fn sign_gump_layout() {
        let (mut w, ns) = world_with_state();
        send_display_sign_gump(
            &mut w,
            ns,
            muo_primitives::Serial::new(5),
            0x1234,
            "u",
            "cap",
        );
        let p = sent(&mut w).remove(0);
        assert_eq!(p.len(), 15 + 1 + 3);
        assert_eq!(u16::from_be_bytes([p[1], p[2]]) as usize, p.len());
        assert_eq!(&p[7..9], &[0x12, 0x34]);
        assert_eq!(&p[9..11], &[0, 2]);
        assert_eq!(&p[11..13], &[b'u', 0]);
    }
}
