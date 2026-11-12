//! AssistantProtocol: the 0xF0 sub-protocol of assistants.
//!
//! `ProtocolExtensions<AssistantsProtocolInfo>`: `configure` takes packet 0xF0 (variable length, in or out of game)
//! and [`handle_0xf0`] dispatches on the command byte to the table.

use muo_net::net_state::NetState;
use muo_net::packet_handler::ReceiveFn;
use muo_server::World;

pub const PACKET_ID: u8 = 0xF0;

#[derive(Copy, Clone)]
pub struct AssistantHandlerEntry {
    pub in_game_only: bool,
    pub out_of_game_only: bool,
    pub on_receive: ReceiveFn<World>,
}

/// The extension table, indexed by command.
pub struct AssistantProtocol {
    handlers: Vec<Option<AssistantHandlerEntry>>,
}

impl Default for AssistantProtocol {
    fn default() -> Self {
        Self {
            handlers: vec![None; 0x100],
        }
    }
}

/// `AssistantProtocol.Configure()` (priority 10): makes the table.
pub fn configure(w: &mut World) {
    *w.resources.get_mut::<AssistantProtocol>() = AssistantProtocol::default();
    // `IncomingPackets.Register(packetId, 0, false, &DecodeBundledPacket)`
    crate::network::register_packet(w, PACKET_ID, 0, false, false, handle_0xf0);
}

impl AssistantProtocol {
    /// The sub-command ids that have a handler (tests compare them with the expected registrations).
    pub fn ids(&self) -> Vec<u8> {
        (0..=255u8)
            .filter(|&i| self.handlers[usize::from(i)].is_some())
            .collect()
    }
}

/// `Register(cmd, ingame, onReceive)`.
pub fn register(w: &mut World, cmd: u8, ingame: bool, on_receive: ReceiveFn<World>) {
    register_full(w, cmd, ingame, false, on_receive);
}

/// `Register(cmd, ingame, outgame, onReceive)`.
pub fn register_full(
    w: &mut World,
    cmd: u8,
    ingame: bool,
    outgame: bool,
    on_receive: ReceiveFn<World>,
) {
    w.resources.get_mut::<AssistantProtocol>().handlers[cmd as usize] =
        Some(AssistantHandlerEntry {
            in_game_only: ingame,
            out_of_game_only: outgame,
            on_receive,
        });
}

/// `DecodeBundledPacket`: the first byte is the command; unknown commands are ignored.
pub fn handle_0xf0(w: &mut World, state: &mut NetState, body: &[u8]) {
    let Some((&cmd, rest)) = body.split_first() else {
        return;
    };
    let Some(h) = w.resources.get_mut::<AssistantProtocol>().handlers[cmd as usize] else {
        return;
    };

    let id = muo_server::net_glue::ensure_net_state(w, state);
    let from = w.net_state(id).and_then(|d| d.mobile);
    let deleted = from.is_some_and(|m| w.mobile(m).is_none_or(|mm| mm.deleted));

    if h.in_game_only {
        if from.is_none() {
            state.disconnect(&format!(
                "Received packet 0x{PACKET_ID:02X}x{cmd:02X} before having been attached to a mobile."
            ));
            return;
        }
        if deleted {
            state.disconnect(&format!(
                "Received packet 0x{PACKET_ID:02X}x{cmd:02X} after having been attached to a deleted mobile."
            ));
            return;
        }
    }

    if h.out_of_game_only && from.is_some() && !deleted {
        state.disconnect(&format!(
            "Received packet 0x{PACKET_ID:02X}x{cmd:02X} after having been attached to a mobile."
        ));
        return;
    }

    (h.on_receive)(w, state, rest);
}
