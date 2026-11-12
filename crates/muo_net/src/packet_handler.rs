//! Packet handlers, encoded packet handlers, the incoming registration tables, `IsInfoPacket`, and packet
//! utilities.
//!
//! A handler is a fn pointer generic over the context `C` the server supplies (its world). It receives the
//! connection and the packet body: the bytes after the id, and after the length for variable-length
//! packets, cut to the packet's declared length.

use crate::net_state::NetState;

/// Receives a decoded frame: context, connection, body.
pub type ReceiveFn<C> = fn(&mut C, &mut NetState, &[u8]);

/// Throttle callback (`delegate*<int, NetState, bool>`): true defers the packet to a later slice. It gets the
/// context too, which knows `Core.TickCount` and the connection's mobile.
pub type ThrottleFn<C> = fn(&mut C, i32, &mut NetState) -> bool;

/// Length rule that depends on the connection (`PacketHandler.GetLength` is virtual).
pub type LengthFn = fn(&NetState) -> i32;

pub struct PacketHandler<C> {
    packet_id: u8,
    /// Fixed length including the id; 0 or less means the length follows the id as a big-endian `u16`.
    length: i32,
    pub length_fn: Option<LengthFn>,
    pub in_game_only: bool,
    pub out_of_game_only: bool,
    pub on_receive: ReceiveFn<C>,
    pub throttle: Option<ThrottleFn<C>>,
}

impl<C> PacketHandler<C> {
    pub fn new(
        packet_id: u8,
        length: i32,
        in_game_only: bool,
        out_of_game_only: bool,
        on_receive: ReceiveFn<C>,
    ) -> Self {
        Self {
            packet_id,
            length,
            length_fn: None,
            in_game_only,
            out_of_game_only,
            on_receive,
            throttle: None,
        }
    }

    pub fn packet_id(&self) -> u8 {
        self.packet_id
    }

    pub fn get_length(&self, ns: &NetState) -> i32 {
        self.length_fn.map_or(self.length, |f| f(ns))
    }
}

/// Encoded (0xBF-style sub-command) handler. The target entity is the serial the packet names, resolved by
/// the server inside the handler.
pub struct EncodedPacketHandler<C> {
    pub packet_id: u8,
    pub in_game: bool,
    /// context, connection, target serial, encoded body
    pub on_receive: fn(&mut C, &mut NetState, u32, &[u8]),
}

pub struct IncomingPackets<C> {
    handlers: Vec<Option<PacketHandler<C>>>,
    encoded: Vec<Option<EncodedPacketHandler<C>>>,
}

impl<C> Default for IncomingPackets<C> {
    fn default() -> Self {
        Self {
            handlers: (0..0x100).map(|_| None).collect(),
            encoded: (0..0x100).map(|_| None).collect(),
        }
    }
}

impl<C> IncomingPackets<C> {
    pub fn new() -> Self {
        Self::default()
    }

    /// `Register(id, length, ingame, onReceive)`
    pub fn register(
        &mut self,
        packet_id: u8,
        length: i32,
        in_game: bool,
        on_receive: ReceiveFn<C>,
    ) {
        self.register_handler(PacketHandler::new(
            packet_id, length, in_game, false, on_receive,
        ));
    }

    /// `Register(id, length, ingame, outgame, onReceive)`
    pub fn register_full(
        &mut self,
        packet_id: u8,
        length: i32,
        in_game: bool,
        out_game: bool,
        on_receive: ReceiveFn<C>,
    ) {
        self.register_handler(PacketHandler::new(
            packet_id, length, in_game, out_game, on_receive,
        ));
    }

    pub fn register_handler(&mut self, handler: PacketHandler<C>) {
        let id = handler.packet_id as usize;
        self.handlers[id] = Some(handler);
    }

    pub fn get_handler(&self, packet_id: u8) -> Option<&PacketHandler<C>> {
        self.handlers[packet_id as usize].as_ref()
    }

    pub fn register_encoded(
        &mut self,
        packet_id: u8,
        in_game: bool,
        on_receive: fn(&mut C, &mut NetState, u32, &[u8]),
    ) {
        self.encoded[packet_id as usize] = Some(EncodedPacketHandler {
            packet_id,
            in_game,
            on_receive,
        });
    }

    pub fn get_encoded_handler(&self, packet_id: usize) -> Option<&EncodedPacketHandler<C>> {
        self.encoded.get(packet_id).and_then(Option::as_ref)
    }

    pub fn remove_encoded_handler(&mut self, packet_id: usize) {
        if let Some(slot) = self.encoded.get_mut(packet_id) {
            *slot = None;
        }
    }

    /// Sets the throttle of an already-registered handler; ignored if there is none.
    pub fn register_throttler(&mut self, packet_id: u8, throttle: ThrottleFn<C>) {
        if let Some(h) = self.handlers[packet_id as usize].as_mut() {
            h.throttle = Some(throttle);
        }
    }
}

/// Packets that can arrive at any time during the login process and are only informational.
pub fn is_info_packet(packet_id: u8) -> bool {
    matches!(
        packet_id,
        0x01 // Disconnect
        | 0x73 // Ping
        | 0xA4 // SystemInfo
        | 0xB1 // Gump Response
        | 0xBB // Account ID
        | 0xBD // Client Version
        | 0xBE // Assist Version
        | 0xD9 // Hardware Info
        | 0xDD // Gumps (Packed)
        | 0xE1 // Client Type
        | 0xF1 // Freeshard Protocol
        | 0xF4 // CrashReport
    )
}

// --- PacketUtilities ---------------------------------------------------------------------------------

/// `WritePacketLength`: stores the number of bytes written so far at positions 1-2, big-endian.
/// `buffer` is the packet buffer; `written` is the writer's `BytesWritten`.
pub fn write_packet_length(buffer: &mut [u8], written: usize) {
    if buffer.len() >= 3 {
        buffer[1..3].copy_from_slice(&(written as u16).to_be_bytes());
    }
}

/// `InitializePacket`: zeroes the first byte (the packet id) so it can be used as an "already built" flag.
pub fn initialize_packet(buffer: &mut [u8]) -> &mut [u8] {
    if let Some(b) = buffer.first_mut() {
        *b = 0;
    }
    buffer
}

#[cfg(test)]
mod tests {
    use super::*;

    fn noop(_: &mut (), _: &mut NetState, _: &[u8]) {}

    #[test]
    fn registration_and_lookup() {
        let mut t = IncomingPackets::<()>::new();
        assert!(t.get_handler(0x80).is_none());
        t.register(0x80, 62, false, noop);
        t.register_full(0x91, 65, false, true, noop);
        let h = t.get_handler(0x80).unwrap();
        assert_eq!(
            (h.packet_id(), h.in_game_only, h.out_of_game_only),
            (0x80, false, false)
        );
        assert!(t.get_handler(0x91).unwrap().out_of_game_only);

        t.register_throttler(0x80, |_, _, _| true);
        assert!(t.get_handler(0x80).unwrap().throttle.is_some());
        t.register_throttler(0x42, |_, _, _| true); // no handler: ignored
        assert!(t.get_handler(0x42).is_none());
    }

    #[test]
    fn encoded_table_bounds() {
        let mut t = IncomingPackets::<()>::new();
        t.register_encoded(0x28, true, |_, _, _, _| {});
        assert!(t.get_encoded_handler(0x28).unwrap().in_game);
        assert!(t.get_encoded_handler(0x100).is_none());
        t.remove_encoded_handler(0x28);
        t.remove_encoded_handler(0x1000);
        assert!(t.get_encoded_handler(0x28).is_none());
    }

    #[test]
    fn info_packets() {
        for id in [
            0x01, 0x73, 0xA4, 0xB1, 0xBB, 0xBD, 0xBE, 0xD9, 0xDD, 0xE1, 0xF1, 0xF4,
        ] {
            assert!(is_info_packet(id), "{id:#x}");
        }
        assert!(!is_info_packet(0x80));
        assert!(!is_info_packet(0x02));
    }

    #[test]
    fn packet_length_and_init() {
        let mut buf = [0xFF, 0, 0, 1, 2, 3];
        initialize_packet(&mut buf);
        assert_eq!(buf[0], 0);
        buf[0] = 0xB0;
        write_packet_length(&mut buf, 0x0123);
        assert_eq!(buf, [0xB0, 0x01, 0x23, 1, 2, 3]);
        initialize_packet(&mut []); // empty buffer: no panic
    }
}
