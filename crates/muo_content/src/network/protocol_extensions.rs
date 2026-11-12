//! Protocol extensions and the freeshard protocol: one packet id (0xF1 for
//! the freeshard protocol) that carries a sub-command byte, each sub-command with its own handler.

use muo_io::SpanReader;
use muo_net::net_state::NetState;
use muo_server::World;

type Receive = fn(&mut World, &mut NetState, &[u8]);

#[derive(Clone, Copy)]
struct Sub {
    in_game_only: bool,
    out_game_only: bool,
    on_receive: Receive,
}

/// `ProtocolExtensions<FreeshardProtocolInfo>.packetHandlers`.
pub struct FreeshardProtocol {
    handlers: [Option<Sub>; 0x100],
}

impl Default for FreeshardProtocol {
    fn default() -> Self {
        Self {
            handlers: [None; 0x100],
        }
    }
}

/// The packet id (`FreeshardProtocolInfo.PacketId`).
pub const PACKET_ID: u8 = 0xF1;

/// `FreeshardProtocol.Configure` (`[CallPriority(10)]`): takes the 0xF1 packet.
pub fn configure(w: &mut World) {
    super::register_packet(w, PACKET_ID, 0, false, false, decode_bundled_packet);
}

/// `FreeshardProtocol.Register(cmd, ingame, outgame, onReceive)`.
pub fn register(w: &mut World, cmd: u8, in_game: bool, out_game: bool, on_receive: Receive) {
    w.resources.get_mut::<FreeshardProtocol>().handlers[usize::from(cmd)] = Some(Sub {
        in_game_only: in_game,
        out_game_only: out_game,
        on_receive,
    });
}

/// `DecodeBundledPacket`.
fn decode_bundled_packet(w: &mut World, ns: &mut NetState, body: &[u8]) {
    let mut r = SpanReader::new(body);
    let Ok(cmd) = r.read_byte() else { return };
    let Some(ph) = w.resources.get_mut::<FreeshardProtocol>().handlers[usize::from(cmd)] else {
        return;
    };

    let id = muo_server::net_glue::ensure_net_state(w, ns);
    let from = w.net_state(id).and_then(|d| d.mobile);
    let deleted = from.is_some_and(|m| w.mobile(m).is_none_or(|mm| mm.deleted));
    if ph.in_game_only {
        if from.is_none() {
            ns.disconnect(&format!("Received packet 0x{PACKET_ID:02X}x{cmd:02X} before having been attached to a mobile."));
            return;
        }
        if deleted {
            ns.disconnect(&format!(
                "Received packet 0x{PACKET_ID:02X}x{cmd:02X} after having been attached to a deleted mobile."
            ));
            return;
        }
    }
    if ph.out_game_only && from.is_some() && !deleted {
        ns.disconnect(&format!(
            "Received packet 0x{PACKET_ID:02X}x{cmd:02X} after having been attached to a mobile."
        ));
        return;
    }
    (ph.on_receive)(w, ns, &body[1..]);
}
