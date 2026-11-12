//! Network: packet handlers and the shard's outgoing packets.
//!
//! Incoming packet handlers are `fn(&mut World, &mut muo_net::NetState, &[u8])` (the transport connection and the
//! bytes after the id, and after the length for variable packets). A content module registers them in its
//! `Configure` with [`register_packet`]; the owner of the `muo_net::Network<World>` calls
//! [`IncomingRegistrations::apply`] once on `net.packets` before listening.

pub mod entity_packets;
pub mod events;
pub mod extended_commands;
pub mod game_server;
pub mod gateway_server;
pub mod incoming_account;
pub mod incoming_game;
pub mod map_uo;
pub use bans::login_allowlist;
pub mod bans;
pub mod outgoing;
pub mod protocol_extensions;
pub mod uo_gateway;

use muo_net::net_state::NetState;
use muo_net::packet_handler::{IncomingPackets, LengthFn, PacketHandler, ThrottleFn};
use muo_server::World;

type Receive = fn(&mut World, &mut NetState, &[u8]);

#[derive(Clone)]
pub enum Registration {
    /// `IncomingPackets.Register(id, length, ingame, outgame, onReceive)`.
    Packet {
        id: u8,
        length: i32,
        in_game: bool,
        out_game: bool,
        on_receive: Receive,
        length_fn: Option<LengthFn>,
    },
    /// `IncomingPackets.RegisterEncoded(id, ingame, onReceive)`.
    Encoded {
        id: u8,
        in_game: bool,
        on_receive: fn(&mut World, &mut NetState, u32, &[u8]),
    },
    /// `IncomingPackets.RegisterThrottler(id, t)`.
    Throttle { id: u8, throttle: ThrottleFn<World> },
}

/// Everything content registered, in order (a later registration of the same id replaces the earlier).
#[derive(Default)]
pub struct IncomingRegistrations(pub Vec<Registration>);

impl IncomingRegistrations {
    pub fn apply(&self, packets: &mut IncomingPackets<World>) {
        for r in &self.0 {
            match r {
                Registration::Packet {
                    id,
                    length,
                    in_game,
                    out_game,
                    on_receive,
                    length_fn,
                } => {
                    let mut h = PacketHandler::new(*id, *length, *in_game, *out_game, *on_receive);
                    h.length_fn = *length_fn;
                    packets.register_handler(h);
                }
                Registration::Encoded {
                    id,
                    in_game,
                    on_receive,
                } => {
                    packets.register_encoded(*id, *in_game, *on_receive);
                }
                Registration::Throttle { id, throttle } => {
                    packets.register_throttler(*id, *throttle)
                }
            }
        }
    }
}

/// `IncomingPackets.Register(new ContainerGridPacketHandler(..))`-style handlers whose length depends on the
/// connection.
pub fn register_packet_length_fn(
    w: &mut World,
    id: u8,
    length: i32,
    in_game: bool,
    out_game: bool,
    on_receive: Receive,
    length_fn: LengthFn,
) {
    w.resources
        .get_mut::<IncomingRegistrations>()
        .0
        .push(Registration::Packet {
            id,
            length,
            in_game,
            out_game,
            on_receive,
            length_fn: Some(length_fn),
        });
}

/// `IncomingPackets.Register(id, onReceive, length, ingameOnly, outgameOnly)`.
pub fn register_packet(
    w: &mut World,
    id: u8,
    length: i32,
    in_game: bool,
    out_game: bool,
    on_receive: Receive,
) {
    w.resources
        .get_mut::<IncomingRegistrations>()
        .0
        .push(Registration::Packet {
            id,
            length,
            in_game,
            out_game,
            on_receive,
            length_fn: None,
        });
}

/// `IncomingPackets.RegisterEncoded(id, ingame, onReceive)`.
pub fn register_encoded(
    w: &mut World,
    id: u8,
    in_game: bool,
    on_receive: fn(&mut World, &mut NetState, u32, &[u8]),
) {
    w.resources
        .get_mut::<IncomingRegistrations>()
        .0
        .push(Registration::Encoded {
            id,
            in_game,
            on_receive,
        });
}

/// `IncomingPackets.RegisterThrottler(id, t)`.
pub fn register_throttler(w: &mut World, id: u8, throttle: ThrottleFn<World>) {
    w.resources
        .get_mut::<IncomingRegistrations>()
        .0
        .push(Registration::Throttle { id, throttle });
}

/// The `Configure` of every Network/Packets file.
pub fn configure(w: &mut World) {
    protocol_extensions::configure(w);
    extended_commands::configure(w);
    bans::configure(w);
    map_uo::configure(w);
    uo_gateway::configure(w);
    incoming_account::configure(w);
    incoming_game::configure(w);
}

/// The `Initialize` of the Network files.
pub fn initialize(w: &mut World) {
    incoming_account::initialize(w);
    bans::initialize(w);
}
