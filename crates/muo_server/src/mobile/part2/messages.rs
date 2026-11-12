//! The overhead and system messages of a mobile.
//!
//! There are no interpolated-handler overloads (`ref RawInterpolatedStringHandler`): callers `format!`.
//! `ReadOnlySpan<char>` arguments are `&str`. Optional arguments of the overloads are explicit here, and the
//! names carry the overload: `public_overhead_message` (text), `public_overhead_message_number`
//! (localized) and `public_overhead_message_affix` (localized with an affix).
//!
//! `OutgoingMessagePackets.Broadcast*` builds the packet once and sends it to each client the filter allows. The
//! filters need `InLOS` (which reads tiles, so `&mut World`), so the clients are collected first and filtered
//! here instead of through `packets::message::broadcast_*`.

use muo_primitives::{MessageType, Point3D, Serial};

use super::deps;
use crate::ids::MobileId;
use crate::mobile::AccessLevel;
use crate::net_state::NetStateId;
use crate::packets::message as pk;
pub use crate::packets::message::AffixType;
use crate::world::World;

/// The three `IBroadcastFilter` structs.
#[derive(Copy, Clone, Debug)]
enum Filter {
    /// `PublicVisibilityFilter`.
    Public {
        source: MobileId,
        no_line_of_sight: bool,
        access_level: AccessLevel,
    },
    /// `LocalizedVisibilityFilter`.
    Localized {
        source: MobileId,
        no_line_of_sight: bool,
    },
    /// `NonlocalVisibilityFilter`.
    Nonlocal {
        source: MobileId,
        exclude: Option<NetStateId>,
    },
}

impl Filter {
    fn allow(self, w: &mut World, state: NetStateId) -> bool {
        let Some(viewer) = w.net_state(state).and_then(|d| d.mobile) else {
            return false;
        };
        match self {
            Filter::Public {
                source,
                no_line_of_sight,
                access_level,
            } => {
                w.mobile(viewer)
                    .is_some_and(|v| v.access_level >= access_level)
                    && deps::can_see_mobile(w, viewer, source)
                    && (no_line_of_sight || deps::in_los_mobile(w, viewer, source))
            }
            Filter::Localized {
                source,
                no_line_of_sight,
            } => {
                deps::can_see_mobile(w, viewer, source)
                    && (no_line_of_sight || deps::in_los_mobile(w, viewer, source))
            }
            Filter::Nonlocal { source, exclude } => {
                Some(state) != exclude && deps::can_see_mobile(w, viewer, source)
            }
        }
    }
}

/// What a mobile says as seen by the packet builders: serial, body, name, location, map.
struct Speaker {
    serial: Serial,
    body: i32,
    name: String,
    language: Option<String>,
    map: Option<crate::map::MapId>,
    location: Point3D,
}

fn speaker(w: &World, id: MobileId) -> Option<Speaker> {
    let m = w.mobile(id)?;
    Some(Speaker {
        serial: m.serial,
        body: crate::mobile::part1::props_b::body(w, id).body_id(),
        // The packets write a null name as "".
        name: (m.ty.name)(w, id).unwrap_or_default(),
        language: m.language.clone(),
        map: m.map,
        location: m.location,
    })
}

fn kind(ty: MessageType) -> pk::MessageType {
    pk::MessageType::from_bits_retain(ty.bits() as u8)
}

/// Sends `bytes` to every client in range of the speaker that `filter` allows.
fn broadcast(w: &mut World, s: &Speaker, bytes: &[u8], filter: Filter) {
    let Some(map) = s.map else { return };
    let range = w.maps.global_max_update_range;
    let states: Vec<NetStateId> =
        crate::map::enumerate::clients_in_range(w, Some(map), s.location.x, s.location.y, range)
            .collect();
    for ns in states {
        if filter.allow(w, ns) {
            w.send(ns, bytes);
        }
    }
}

fn speech_hue(w: &World, id: MobileId) -> i32 {
    w.mobile(id).map_or(0, |m| m.speech_hue)
}

// ---------- Say / Emote / Whisper / Yell ----------

/// `Say(bool ascii, text)`.
pub fn say(w: &mut World, id: MobileId, ascii: bool, t: &str) {
    let hue = speech_hue(w, id);
    public_overhead_message(
        w,
        id,
        MessageType::REGULAR,
        hue,
        ascii,
        t,
        true,
        AccessLevel::Player,
    );
}

/// `Say(text)`.
pub fn say_text(w: &mut World, id: MobileId, t: &str) {
    say(w, id, false, t);
}

/// `Say(int number, AffixType type, affix, args)`.
pub fn say_affix(w: &mut World, id: MobileId, number: i32, ty: AffixType, affix: &str, args: &str) {
    let hue = speech_hue(w, id);
    public_overhead_message_affix(
        w,
        id,
        MessageType::REGULAR,
        hue,
        number,
        ty,
        affix,
        args,
        false,
        AccessLevel::Player,
    );
}

/// `Say(int number, args = "")`.
pub fn say_number(w: &mut World, id: MobileId, number: i32, args: &str) {
    let hue = speech_hue(w, id);
    public_overhead_message_number(w, id, MessageType::REGULAR, hue, number, args, true);
}

/// `Emote(text)`.
pub fn emote(w: &mut World, id: MobileId, t: &str) {
    let hue = w.mobile(id).map_or(0, |m| m.emote_hue);
    public_overhead_message(
        w,
        id,
        MessageType::EMOTE,
        hue,
        false,
        t,
        true,
        AccessLevel::Player,
    );
}

/// `Emote(int number, args = "")`.
pub fn emote_number(w: &mut World, id: MobileId, number: i32, args: &str) {
    let hue = w.mobile(id).map_or(0, |m| m.emote_hue);
    public_overhead_message_number(w, id, MessageType::EMOTE, hue, number, args, true);
}

/// `Whisper(text)`.
pub fn whisper(w: &mut World, id: MobileId, t: &str) {
    let hue = w.mobile(id).map_or(0, |m| m.whisper_hue);
    public_overhead_message(
        w,
        id,
        MessageType::WHISPER,
        hue,
        false,
        t,
        true,
        AccessLevel::Player,
    );
}

/// `Whisper(int number, args = "")`.
pub fn whisper_number(w: &mut World, id: MobileId, number: i32, args: &str) {
    let hue = w.mobile(id).map_or(0, |m| m.whisper_hue);
    public_overhead_message_number(w, id, MessageType::WHISPER, hue, number, args, true);
}

/// `Yell(text)`.
pub fn yell(w: &mut World, id: MobileId, t: &str) {
    let hue = w.mobile(id).map_or(0, |m| m.yell_hue);
    public_overhead_message(
        w,
        id,
        MessageType::YELL,
        hue,
        false,
        t,
        true,
        AccessLevel::Player,
    );
}

/// `Yell(int number, args = "")`.
pub fn yell_number(w: &mut World, id: MobileId, number: i32, args: &str) {
    let hue = w.mobile(id).map_or(0, |m| m.yell_hue);
    public_overhead_message_number(w, id, MessageType::YELL, hue, number, args, true);
}

// ---------- PublicOverheadMessage ----------

/// `PublicOverheadMessage(type, hue, ascii, text, noLineOfSight = true, accessLevel = Player)`.
#[allow(clippy::too_many_arguments)]
pub fn public_overhead_message(
    w: &mut World,
    id: MobileId,
    ty: MessageType,
    hue: i32,
    ascii: bool,
    t: &str,
    no_line_of_sight: bool,
    access_level: AccessLevel,
) {
    let Some(s) = speaker(w, id) else { return };
    let bytes = pk::create_message(
        s.serial,
        s.body,
        kind(ty),
        hue,
        3,
        ascii,
        s.language.as_deref(),
        &s.name,
        t,
    );
    let filter = Filter::Public {
        source: id,
        no_line_of_sight,
        access_level,
    };
    broadcast(w, &s, &bytes, filter);
}

/// `PublicOverheadMessage(type, hue, number, args = "", noLineOfSight = true)`.
pub fn public_overhead_message_number(
    w: &mut World,
    id: MobileId,
    ty: MessageType,
    hue: i32,
    number: i32,
    args: &str,
    no_line_of_sight: bool,
) {
    let Some(s) = speaker(w, id) else { return };
    let bytes =
        pk::create_message_localized(s.serial, s.body, kind(ty), hue, 3, number, &s.name, args);
    let filter = Filter::Localized {
        source: id,
        no_line_of_sight,
    };
    broadcast(w, &s, &bytes, filter);
}

/// `PublicOverheadMessage(type, hue, number, affixType, affix, args = "", noLineOfSight = false, accessLevel = Player)`.
#[allow(clippy::too_many_arguments)]
pub fn public_overhead_message_affix(
    w: &mut World,
    id: MobileId,
    ty: MessageType,
    hue: i32,
    number: i32,
    affix_type: AffixType,
    affix: &str,
    args: &str,
    no_line_of_sight: bool,
    access_level: AccessLevel,
) {
    let Some(s) = speaker(w, id) else { return };
    let bytes = pk::create_message_localized_affix(
        s.serial,
        s.body,
        kind(ty),
        hue,
        3,
        number,
        &s.name,
        affix_type,
        affix,
        args,
    );
    let filter = Filter::Public {
        source: id,
        no_line_of_sight,
        access_level,
    };
    broadcast(w, &s, &bytes, filter);
}

// ---------- PrivateOverheadMessage ----------

/// `PrivateOverheadMessage(type, hue, ascii, text, NetState)`. Nothing is sent to a null state.
pub fn private_overhead_message(
    w: &mut World,
    id: MobileId,
    ty: MessageType,
    hue: i32,
    ascii: bool,
    t: &str,
    state: Option<NetStateId>,
) {
    let (Some(state), Some(s)) = (state, speaker(w, id)) else {
        return;
    };
    pk::send_message(
        w,
        state,
        s.serial,
        s.body,
        kind(ty),
        hue,
        3,
        ascii,
        s.language.as_deref(),
        &s.name,
        t,
    );
}

/// `PrivateOverheadMessage(type, hue, number, args = "", NetState)`.
pub fn private_overhead_message_number(
    w: &mut World,
    id: MobileId,
    ty: MessageType,
    hue: i32,
    number: i32,
    args: &str,
    state: Option<NetStateId>,
) {
    let (Some(state), Some(s)) = (state, speaker(w, id)) else {
        return;
    };
    pk::send_message_localized(
        w,
        state,
        s.serial,
        s.body,
        kind(ty),
        hue,
        3,
        number,
        &s.name,
        args,
    );
}

// ---------- LocalOverheadMessage ----------

/// `LocalOverheadMessage(type, hue, ascii, text)`: to the mobile's own client.
pub fn local_overhead_message(
    w: &mut World,
    id: MobileId,
    ty: MessageType,
    hue: i32,
    ascii: bool,
    t: &str,
) {
    let ns = w.mobile(id).and_then(|m| m.net_state);
    private_overhead_message(w, id, ty, hue, ascii, t, ns);
}

/// `LocalOverheadMessage(type, hue, number, args = "")`.
pub fn local_overhead_message_number(
    w: &mut World,
    id: MobileId,
    ty: MessageType,
    hue: i32,
    number: i32,
    args: &str,
) {
    let ns = w.mobile(id).and_then(|m| m.net_state);
    private_overhead_message_number(w, id, ty, hue, number, args, ns);
}

// ---------- NonlocalOverheadMessage ----------

/// `NonlocalOverheadMessage(type, hue, number, args = "")`: to everyone in range but the mobile's own client.
pub fn nonlocal_overhead_message_number(
    w: &mut World,
    id: MobileId,
    ty: MessageType,
    hue: i32,
    number: i32,
    args: &str,
) {
    let Some(s) = speaker(w, id) else { return };
    let exclude = w.mobile(id).and_then(|m| m.net_state);
    let bytes =
        pk::create_message_localized(s.serial, s.body, kind(ty), hue, 3, number, &s.name, args);
    broadcast(
        w,
        &s,
        &bytes,
        Filter::Nonlocal {
            source: id,
            exclude,
        },
    );
}

/// `NonlocalOverheadMessage(type, hue, ascii, text)`.
pub fn nonlocal_overhead_message(
    w: &mut World,
    id: MobileId,
    ty: MessageType,
    hue: i32,
    ascii: bool,
    t: &str,
) {
    let Some(s) = speaker(w, id) else { return };
    let exclude = w.mobile(id).and_then(|m| m.net_state);
    let bytes = pk::create_message(
        s.serial,
        s.body,
        kind(ty),
        hue,
        3,
        ascii,
        s.language.as_deref(),
        &s.name,
        t,
    );
    broadcast(
        w,
        &s,
        &bytes,
        Filter::Nonlocal {
            source: id,
            exclude,
        },
    );
}

// ---------- SendLocalizedMessage / SendMessage / SendAsciiMessage ----------

fn own_state(w: &World, id: MobileId) -> Option<NetStateId> {
    w.mobile(id).and_then(|m| m.net_state)
}

/// `SendLocalizedMessage(number, args = "", hue = 0x3B2)`.
pub fn send_localized_message(w: &mut World, id: MobileId, number: i32, args: &str, hue: i32) {
    if let Some(ns) = own_state(w, id) {
        pk::send_message_localized(
            w,
            ns,
            Serial::MINUS_ONE,
            -1,
            pk::MessageType::REGULAR,
            hue,
            3,
            number,
            "System",
            args,
        );
    }
}

/// `SendLocalizedMessage(number, append, affix, args = "", hue = 0x3B2)`.
pub fn send_localized_message_affix(
    w: &mut World,
    id: MobileId,
    number: i32,
    append: bool,
    affix: &str,
    args: &str,
    hue: i32,
) {
    let affix_type = (if append {
        AffixType::APPEND
    } else {
        AffixType::PREPEND
    }) | AffixType::SYSTEM;
    if let Some(ns) = own_state(w, id) {
        pk::send_message_localized_affix(
            w,
            ns,
            Serial::MINUS_ONE,
            -1,
            pk::MessageType::REGULAR,
            hue,
            3,
            number,
            "System",
            affix_type,
            affix,
            args,
        );
    }
}

/// `SendMessage(text)`.
pub fn send_message(w: &mut World, id: MobileId, t: &str) {
    send_message_hue(w, id, 0x3B2, t);
}

/// `SendMessage(hue, text)`.
pub fn send_message_hue(w: &mut World, id: MobileId, hue: i32, t: &str) {
    if let Some(ns) = own_state(w, id) {
        pk::send_message(
            w,
            ns,
            Serial::MINUS_ONE,
            -1,
            pk::MessageType::REGULAR,
            hue,
            3,
            false,
            Some("ENU"),
            "System",
            t,
        );
    }
}

/// `SendAsciiMessage(text)`.
pub fn send_ascii_message(w: &mut World, id: MobileId, t: &str) {
    send_ascii_message_hue(w, id, 0x3B2, t);
}

/// `SendAsciiMessage(hue, text)`.
pub fn send_ascii_message_hue(w: &mut World, id: MobileId, hue: i32, t: &str) {
    if let Some(ns) = own_state(w, id) {
        pk::send_message(
            w,
            ns,
            Serial::MINUS_ONE,
            -1,
            pk::MessageType::REGULAR,
            hue,
            3,
            true,
            None,
            "System",
            t,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mobile::MOBILE_BASE;

    #[test]
    fn affix_flags() {
        assert_eq!(AffixType::PREPEND.bits() | AffixType::SYSTEM.bits(), 3);
        assert_eq!(AffixType::APPEND.bits(), 0);
    }

    #[test]
    fn messages_without_a_client_are_no_ops() {
        let mut w = World::new();
        let id = w.new_mobile(&MOBILE_BASE);
        send_message(&mut w, id, "hello");
        send_localized_message(&mut w, id, 1019048, "", 0x3B2);
        send_ascii_message(&mut w, id, "x");
        local_overhead_message(&mut w, id, MessageType::REGULAR, 0, false, "x");
        // On Map.Internal nobody is in range, so a public message goes nowhere and must not panic.
        say(&mut w, id, false, "hi");
        assert!(w.net_out.is_empty());
    }
}
