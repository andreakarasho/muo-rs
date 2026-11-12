//! The transport half of a connection (send/receive buffering, seed handling, frame parsing, disconnect
//! reasons, keep-alive) and its client version.
//!
//! Left to `muo_server`, because they touch entities: `Mobile`, `Account` objects, trades, menus, hue
//! pickers, city info, `ExpansionInfo`. The server keeps those in its own table keyed by [`NetState::id`]
//! and tells the transport only what framing needs: [`Attachment`] and `account`.

use crate::client_version::{ClientFlags, ClientType, ClientVersion};
use crate::compression;
use crate::encryption::{
    ClientEncryption, Detected, EncryptionConfig, detect_game_encryption, detect_login_encryption,
};
use crate::foreign_protocol::{self, ForeignProtocolMatch};
use crate::movement::MovementState;
use crate::packet_handler::{IncomingPackets, is_info_packet};
use crate::protocol_changes::ProtocolChanges;
use mio::net::TcpStream;
use std::io::{ErrorKind, Read as _, Write as _};
use std::net::{IpAddr, Shutdown, SocketAddr};
use std::time::SystemTime;
use tracing::{debug, info, warn};

/// 64 KiB receive buffer; a packet declaring this many bytes (or more) can never complete.
pub const RECV_BUFFER_SIZE: usize = 1024 * 64;
pub const PACKET_PER_SECOND_THRESHOLD: i64 = 3000;
/// Graceful disconnect gets this long to drain.
pub const DRAIN_TIMEOUT_MS: i64 = 10000;
/// Activity grace period after any traffic (and at connect).
pub const ACTIVITY_TIMEOUT_MS: i64 = 30000;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ParserState {
    AwaitingNextPacket,
    AwaitingPartialPacket,
    ProcessingPacket,
    Throttled,
    Error,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ProtocolState {
    /// Based on the way the seed arrives, we know if this is a login server or a game server connection
    AwaitingSeed,
    LoginServerAwaitingLogin,
    LoginServerAwaitingServerSelect,
    LoginServerServerSelectAck,
    GameServerAwaitingGameServerLogin,
    GameServerLoggedIn,
    Error,
}

/// What the server tells the transport about the connection's mobile (the `Mobile` null/`Deleted` checks
/// the in-game flags need).
#[derive(Copy, Clone, Debug, Eq, PartialEq, Default)]
pub enum Attachment {
    #[default]
    None,
    /// Attached to a live mobile (its serial).
    Mobile(u32),
    /// Attached to a mobile that has since been deleted.
    Deleted,
}

pub(crate) enum ReadResult {
    Data,
    WouldBlock,
    /// No room in the receive buffer; retry once the parser has consumed some.
    Full,
    /// Orderly shutdown or a socket error.
    Closed,
}

pub(crate) enum FlushResult {
    Done,
    Blocked,
    Closed,
}

/// What the parser wants the transport to do with the connection after `handle_receive`.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ReceiveOutcome {
    Done,
    /// A throttle deferred a packet: run `handle_receive(true)` again next slice.
    Throttled,
    /// Still throttled on a re-run.
    ThrottledPending,
}

/// Unread bytes from the client. Consumed from the front; compacted when room is needed.
#[derive(Default)]
pub struct RecvBuffer {
    data: Vec<u8>,
    start: usize,
    end: usize,
}

impl RecvBuffer {
    pub fn readable(&self) -> &[u8] {
        &self.data[self.start..self.end]
    }

    pub fn len(&self) -> usize {
        self.end - self.start
    }

    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }

    fn commit_read(&mut self, n: usize) {
        self.start = (self.start + n).min(self.end);
        if self.start == self.end {
            self.start = 0;
            self.end = 0;
        }
    }

    /// Free tail space (one slot short of the buffer, like the ring it replaces).
    pub(crate) fn writable(&mut self) -> &mut [u8] {
        if self.data.is_empty() {
            self.data = vec![0; RECV_BUFFER_SIZE];
        }
        if self.start > 0 && self.end == self.data.len() {
            self.data.copy_within(self.start..self.end, 0);
            self.end -= self.start;
            self.start = 0;
        }
        let cap = self.data.len() - 1;
        &mut self.data[self.end..cap.max(self.end)]
    }

    pub(crate) fn commit_write(&mut self, n: usize) -> std::ops::Range<usize> {
        let old = self.end;
        self.end += n;
        old..self.end
    }

    pub(crate) fn slice_mut(&mut self, r: std::ops::Range<usize>) -> &mut [u8] {
        &mut self.data[r]
    }

    /// Test hook: appends bytes as if they had been received.
    pub fn push_for_test(&mut self, bytes: &[u8]) {
        let w = self.writable();
        w[..bytes.len()].copy_from_slice(bytes);
        self.commit_write(bytes.len());
    }
}

pub struct NetState {
    pub(crate) id: usize,
    /// Unique per connection; slot ids are reused, this is not.
    pub(crate) serial: u64,
    pub(crate) stream: Option<TcpStream>,
    local: Option<SocketAddr>,
    pub address: IpAddr,
    to_string: String,

    pub(crate) recv: RecvBuffer,
    /// Bytes encrypted/compressed and waiting for the socket; `send_pos` of them already written.
    pub(crate) send_buf: Vec<u8>,
    pub(crate) send_pos: usize,
    pub(crate) max_send_buffer_size: usize,

    pub(crate) running: bool,
    pub(crate) connected: bool,
    pub(crate) disconnect_pending: bool,
    pub(crate) disconnect_queued: bool,
    pub(crate) read_pending: bool,
    pub(crate) close_queued: bool,
    pub(crate) writable_event: bool,
    drain_deadline: i64,
    drain_deadline_armed: bool,

    pub parser_state: ParserState,
    pub protocol_state: ProtocolState,
    encryption: Option<Box<dyn ClientEncryption>>,
    pub(crate) encryption_config: EncryptionConfig,
    version: Option<ClientVersion>,
    protocol_changes: ProtocolChanges,
    pub flags: ClientFlags,

    pub disconnect_reason: String,
    /// Whether ANY inbound bytes have arrived: separates a slow client from a socket held open on purpose.
    pub received_data: bool,
    /// Set when the parser saw something the ban policy wants reported (`ban_reasons`); the transport reads
    /// and clears it after each receive.
    pub(crate) bad_connect: Option<&'static str>,

    pub auth_id: i32,
    pub seed: i32,
    pub seeded: bool,
    pub sent_first_packet: bool,
    pub block_all_packets: bool,
    pub compression_enabled: bool,
    pub current_packet: i32,
    pub assistant: Option<String>,
    /// Username of the authenticated account (`NetState.Account` is non-null).
    pub account: Option<String>,
    pub attachment: Attachment,

    pub movement: MovementState,
    packet_throttles: Option<Box<[i64; 0x100]>>,
    packet_counts: Option<Box<[i64; 0x100]>>,

    packet_logging: bool,
    pub(crate) packet_logging_path: String,
    pub next_activity_check: i64,
    pub(crate) connected_tick: i64,
    pub connected_on: SystemTime,
    /// Wrapping tick count of the current slice (`Core.TickCount`), set by the transport.
    pub(crate) now: i64,
}

impl NetState {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        id: usize,
        serial: u64,
        stream: Option<TcpStream>,
        address: IpAddr,
        now: i64,
        max_send_buffer_size: usize,
        encryption_config: EncryptionConfig,
        packet_logging_path: String,
    ) -> Self {
        let local = stream.as_ref().and_then(|s| s.local_addr().ok());
        Self {
            id,
            serial,
            connected: stream.is_some(),
            stream,
            local,
            address,
            to_string: address.to_string(),
            recv: RecvBuffer::default(),
            send_buf: Vec::new(),
            send_pos: 0,
            max_send_buffer_size,
            running: true,
            disconnect_pending: false,
            disconnect_queued: false,
            read_pending: false,
            close_queued: false,
            writable_event: false,
            drain_deadline: 0,
            drain_deadline_armed: false,
            parser_state: ParserState::AwaitingNextPacket,
            protocol_state: ProtocolState::AwaitingSeed,
            encryption: None,
            encryption_config,
            version: None,
            protocol_changes: ProtocolChanges::empty(),
            flags: ClientFlags::empty(),
            disconnect_reason: String::new(),
            received_data: false,
            bad_connect: None,
            auth_id: 0,
            seed: 0,
            seeded: false,
            sent_first_packet: false,
            block_all_packets: false,
            compression_enabled: false,
            current_packet: 0,
            assistant: None,
            account: None,
            attachment: Attachment::None,
            movement: MovementState::new(now),
            packet_throttles: None,
            packet_counts: None,
            packet_logging: false,
            packet_logging_path,
            next_activity_check: now + ACTIVITY_TIMEOUT_MS,
            connected_tick: now,
            connected_on: SystemTime::now(),
            now,
        }
    }

    /// Slot in the transport's table; stable for the life of the connection.
    pub fn id(&self) -> usize {
        self.id
    }

    pub fn running(&self) -> bool {
        self.running
    }

    /// Whether the socket is connected.
    pub fn is_connected(&self) -> bool {
        self.connected
    }

    /// The local endpoint (address/port) the client connected to.
    pub fn local_end_point(&self) -> Option<SocketAddr> {
        self.local
    }

    pub fn encryption(&self) -> Option<&dyn ClientEncryption> {
        self.encryption.as_deref()
    }

    pub fn set_encryption(&mut self, e: Option<Box<dyn ClientEncryption>>) {
        self.encryption = e;
    }

    pub fn has_encryption(&self) -> bool {
        self.encryption.is_some()
    }

    pub fn disconnect_pending(&self) -> bool {
        self.disconnect_pending
    }

    pub fn is_disconnect_queued(&self) -> bool {
        self.disconnect_queued
    }

    pub fn pending_send_bytes(&self) -> usize {
        self.send_buf.len() - self.send_pos
    }

    pub fn recv_buffer(&self) -> &RecvBuffer {
        &self.recv
    }

    pub fn recv_buffer_mut(&mut self) -> &mut RecvBuffer {
        &mut self.recv
    }

    // --- Client version ---------------------------------------------------------------------------

    pub fn version(&self) -> Option<&ClientVersion> {
        self.version.as_ref()
    }

    /// Setting the version also sets `protocol_changes` (empty for `None`).
    pub fn set_version(&mut self, v: Option<ClientVersion>) {
        self.protocol_changes = v
            .as_ref()
            .map(|v| v.protocol_changes())
            .unwrap_or_else(ProtocolChanges::empty);
        self.version = v;
    }

    pub fn protocol_changes(&self) -> ProtocolChanges {
        self.protocol_changes
    }

    pub fn has_flag(&self, flag: ClientFlags) -> bool {
        self.flags.intersects(flag)
    }

    pub fn has_protocol_changes(&self, changes: ProtocolChanges) -> bool {
        self.protocol_changes.intersects(changes)
    }

    pub fn new_spellbook(&self) -> bool {
        self.has_protocol_changes(ProtocolChanges::NEW_SPELLBOOK)
    }
    pub fn damage_packet(&self) -> bool {
        self.has_protocol_changes(ProtocolChanges::DAMAGE_PACKET)
    }
    pub fn buff_icon(&self) -> bool {
        self.has_protocol_changes(ProtocolChanges::BUFF_ICON)
    }
    pub fn new_haven(&self) -> bool {
        self.has_protocol_changes(ProtocolChanges::NEW_HAVEN)
    }
    pub fn container_grid_lines(&self) -> bool {
        self.has_protocol_changes(ProtocolChanges::CONTAINER_GRID_LINES)
    }
    pub fn extended_supported_features(&self) -> bool {
        self.has_protocol_changes(ProtocolChanges::EXTENDED_SUPPORTED_FEATURES)
    }
    pub fn stygian_abyss(&self) -> bool {
        self.has_protocol_changes(ProtocolChanges::STYGIAN_ABYSS)
    }
    pub fn high_seas(&self) -> bool {
        self.has_protocol_changes(ProtocolChanges::HIGH_SEAS)
    }
    pub fn new_character_list(&self) -> bool {
        self.has_protocol_changes(ProtocolChanges::NEW_CHARACTER_LIST)
    }
    pub fn new_character_creation(&self) -> bool {
        self.has_protocol_changes(ProtocolChanges::NEW_CHARACTER_CREATION)
    }
    pub fn extended_status(&self) -> bool {
        self.has_protocol_changes(ProtocolChanges::EXTENDED_STATUS)
    }
    pub fn new_mobile_incoming(&self) -> bool {
        self.has_protocol_changes(ProtocolChanges::NEW_MOBILE_INCOMING)
    }
    pub fn new_secure_trading(&self) -> bool {
        self.has_protocol_changes(ProtocolChanges::NEW_SECURE_TRADING)
    }

    fn client_type(&self) -> Option<ClientType> {
        self.version.as_ref().map(|v| v.kind)
    }

    pub fn is_uotd_client(&self) -> bool {
        self.has_flag(ClientFlags::UOTD) || self.client_type() == Some(ClientType::Uotd)
    }
    pub fn is_kr_client(&self) -> bool {
        self.client_type() == Some(ClientType::Kr)
    }
    pub fn is_sa_client(&self) -> bool {
        self.client_type() == Some(ClientType::Sa)
    }
    pub fn is_enhanced_client(&self) -> bool {
        matches!(self.client_type(), Some(ClientType::Kr | ClientType::Sa))
    }

    // --- packet throttles / counts ---------------------------------------------------------------

    fn set_packet_time(&mut self, packet_id: u8) {
        let now = self.now;
        self.packet_throttles
            .get_or_insert_with(|| Box::new([0; 0x100]))[packet_id as usize] = now;
    }

    pub fn get_packet_time(&self, packet_id: i32) -> i64 {
        match (&self.packet_throttles, usize::try_from(packet_id)) {
            (Some(t), Ok(i)) if i < 0x100 => t[i],
            _ => 0,
        }
    }

    fn update_packet_count(&mut self, packet_id: u8) {
        self.packet_counts
            .get_or_insert_with(|| Box::new([0; 0x100]))[packet_id as usize] += 1;
    }

    /// Returns the first packet id that exceeded the per-second threshold, else 0. Counts are cleared up to and
    /// including that id; the later ones are left as they are.
    pub fn check_packet_counts(&mut self) -> i32 {
        let Some(counts) = self.packet_counts.as_mut() else {
            return 0;
        };
        for (i, c) in counts.iter_mut().enumerate() {
            if std::mem::take(c) > PACKET_PER_SECOND_THRESHOLD {
                return i as i32;
            }
        }
        0
    }

    // --- logging ---------------------------------------------------------------------------------

    pub fn log_info(&self, text: &str) {
        info!("Client: {}: {}", self.to_string, text);
    }

    pub fn packet_logging(&self) -> bool {
        self.packet_logging
    }

    pub fn set_packet_logging(&mut self, on: bool) {
        self.packet_logging = on;
        if on {
            self.start_packet_log();
        }
    }

    fn packet_log_path(&self) -> std::path::PathBuf {
        std::path::Path::new(&self.packet_logging_path)
            .join(&self.to_string)
            .join("packets.log")
    }

    fn start_packet_log(&self) {
        let path = self.packet_log_path();
        let result = (|| -> std::io::Result<()> {
            if let Some(dir) = path.parent() {
                std::fs::create_dir_all(dir)?;
            }
            let mut f = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&path)?;
            write!(
                f,
                ">>>>>>>>>> Logging started {} <<<<<<<<<<\r\n\r\n\r\n",
                crate::logs::now_string()
            )
        })();
        if let Err(e) = result {
            warn!("{e}");
        }
    }

    fn log_packet(&self, buffer: &[u8], incoming: bool) {
        let Some(first) = buffer.first() else { return };
        let path = self.packet_log_path();
        let _ = (|| -> std::io::Result<()> {
            if let Some(dir) = path.parent() {
                std::fs::create_dir_all(dir)?;
            }
            let mut f = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&path)?;
            let dir = if incoming {
                "Client -> Server"
            } else {
                "Server -> Client"
            };
            let mut text = format!(
                "{}: {} 0x{:02X} (Length: {})\r\n",
                crate::logs::time_of_day_string(),
                dir,
                first,
                buffer.len()
            );
            text.push_str(&crate::logs::format_buffer(buffer));
            text.push_str("\r\n\r\n");
            f.write_all(text.as_bytes())
        })();
    }

    // --- sending ---------------------------------------------------------------------------------

    /// `CannotSendPackets`: `Running` is not checked, so sends between `disconnect()` and the handoff
    /// still go out. After the handoff the socket is draining or closed; new writes would only keep the
    /// buffer from draining.
    pub fn cannot_send_packets(&self) -> bool {
        self.stream.is_none()
            || !self.connected
            || self.disconnect_pending
            || self.block_all_packets
    }

    fn writable_bytes(&self) -> usize {
        // The ring this replaces holds one byte less than its size
        (self.max_send_buffer_size - 1).saturating_sub(self.pending_send_bytes())
    }

    /// Queues a packet: compressed when enabled, then encrypted, then flushed by the next transport slice.
    /// A packet that cannot fit disconnects the client ("send buffer exhausted") instead of being dropped
    /// silently.
    pub fn send(&mut self, span: &[u8]) {
        if span.is_empty() || self.cannot_send_packets() {
            return;
        }

        // Closing; nothing to report
        if !self.running || self.stream.is_none() {
            return;
        }

        let writable = self.writable_bytes();
        if self.send_pos > 0 && self.send_pos == self.send_buf.len() {
            self.send_buf.clear();
            self.send_pos = 0;
        }

        let mut out: Vec<u8>;
        if self.compression_enabled {
            if span.len() > compression::DEFINITE_OVERFLOW {
                self.send_buffer_exhausted(span.len());
                return;
            }

            // Worst case is 11 bits a byte plus the terminal code
            let bound = (span.len() * 11).div_ceil(8) + 8;
            out = vec![0; bound.min(writable)];
            let n = compression::compress(span, &mut out);
            if n == 0 {
                self.send_buffer_exhausted(span.len());
                return;
            }
            out.truncate(n);
        } else {
            if span.len() > writable {
                self.send_buffer_exhausted(span.len());
                return;
            }
            out = span.to_vec();
        }

        if let Some(e) = self.encryption.as_mut() {
            e.server_encrypt(&mut out);
        }

        if self.packet_logging {
            self.log_packet(span, false);
        }

        self.send_buf.extend_from_slice(&out);
    }

    /// Handles a packet that cannot be placed in the send buffer. High unacked means a slow client holding
    /// the buffer; needed approaching capacity means the buffer is too small for this shard and
    /// `network.sendBufferMaxSize` should be raised.
    fn send_buffer_exhausted(&mut self, needed: usize) {
        // One report per disconnect; the first reason wins
        if self.disconnect_queued {
            return;
        }

        let writable = self.writable_bytes();
        let unacked = 0;
        let capacity = self.max_send_buffer_size;

        warn!(
            "{}: send buffer exhausted - needed {needed} bytes, {writable} writable, {unacked} awaiting acknowledgement, {capacity} capacity. Raise network.sendBufferMaxSize (power of two) if this recurs on healthy connections.",
            self.to_string
        );

        self.disconnect(&format!(
            "Send buffer exhausted (needed {needed}, writable {writable}, unacked {unacked}, capacity {capacity})"
        ));
    }

    /// `SendClientVersionRequest`: `[0xBD, 0x00, 0x03]`.
    pub fn send_client_version_request(&mut self) {
        self.send(&[0xBD, 0x00, 0x03]);
    }

    // --- disconnecting ---------------------------------------------------------------------------

    /// Requests a graceful disconnect. The transport hands it to the socket after the flush in `slice()`:
    /// sends made before that are flushed first, sends after it are dropped.
    pub fn disconnect(&mut self, reason: &str) {
        if !self.running || self.stream.is_none() {
            return;
        }

        self.disconnect_reason = reason.to_string();
        self.disconnect_queued = true;
    }

    /// Bounds the graceful drain. Send progress keeps `next_activity_check` moving, so a slow peer could
    /// otherwise hold a closing socket open indefinitely.
    pub(crate) fn arm_drain_deadline(&mut self, cur_ticks: i64) {
        if !self.drain_deadline_armed {
            self.drain_deadline_armed = true;
            self.drain_deadline = cur_ticks + DRAIN_TIMEOUT_MS;
        }
    }

    /// Closes the socket now: pending output is abandoned.
    pub(crate) fn disconnect_immediate(&mut self) {
        if let Some(stream) = self.stream.take() {
            let _ = stream.shutdown(Shutdown::Both);
        }
        self.connected = false;
        self.disconnect_pending = false;
    }

    /// Graceful close once everything queued has been written.
    pub(crate) fn finish_close(&mut self) {
        if let Some(stream) = self.stream.take() {
            let _ = stream.shutdown(Shutdown::Write);
        }
        self.connected = false;
        self.disconnect_pending = false;
    }

    /// Returns true when the connection is closed or has nothing left to say and the transport should now
    /// drop it, or after a dead-peer check.
    pub fn check_alive(&mut self, cur_ticks: i64) {
        if self.stream.is_none() {
            return;
        }

        if self.disconnect_pending {
            self.arm_drain_deadline(cur_ticks); // transport-initiated drains are first seen here

            if cur_ticks - self.drain_deadline >= 0 || self.next_activity_check - cur_ticks < 0 {
                self.log_info("Force disconnecting stuck socket...");
                self.disconnect_immediate();
            }

            return;
        }

        if self.next_activity_check - cur_ticks >= 0 {
            return;
        }

        // Authenticated pre-game clients (login screens): send keep-alive instead of disconnecting.
        // The 0xBD ClientVersionRequest resets next_activity_check when it is sent.
        if self.account.is_some() && self.attachment == Attachment::None {
            self.send_client_version_request();
            return;
        }

        self.log_info("Disconnecting due to inactivity...");
        self.disconnect("Disconnecting due to inactivity.");
    }

    // --- socket I/O (driven by the transport) ----------------------------------------------------

    /// Reads one chunk from the socket into the receive buffer, decrypting the new bytes in place.
    pub(crate) fn read_chunk(&mut self) -> ReadResult {
        let Some(stream) = self.stream.as_mut() else {
            return ReadResult::Closed;
        };

        let space = self.recv.writable();
        if space.is_empty() {
            return ReadResult::Full;
        }

        loop {
            match stream.read(self.recv.writable()) {
                Ok(0) => return ReadResult::Closed,
                Ok(n) => {
                    let range = self.recv.commit_write(n);
                    self.received_data = true;
                    if let Some(e) = self.encryption.as_mut() {
                        e.client_decrypt(self.recv.slice_mut(range));
                    }
                    self.next_activity_check = self.now + ACTIVITY_TIMEOUT_MS;
                    return ReadResult::Data;
                }
                Err(e) if e.kind() == ErrorKind::WouldBlock => return ReadResult::WouldBlock,
                Err(e) if e.kind() == ErrorKind::Interrupted => continue,
                Err(_) => return ReadResult::Closed,
            }
        }
    }

    /// Writes as much queued output as the socket takes. Progress counts as activity.
    pub(crate) fn flush_send(&mut self) -> FlushResult {
        let Some(stream) = self.stream.as_mut() else {
            return FlushResult::Closed;
        };

        while self.send_pos < self.send_buf.len() {
            match stream.write(&self.send_buf[self.send_pos..]) {
                Ok(0) => return FlushResult::Closed,
                Ok(n) => {
                    self.send_pos += n;
                    self.next_activity_check = self.now + ACTIVITY_TIMEOUT_MS;
                }
                Err(e) if e.kind() == ErrorKind::WouldBlock => return FlushResult::Blocked,
                Err(e) if e.kind() == ErrorKind::Interrupted => continue,
                Err(_) => return FlushResult::Closed,
            }
        }

        self.send_buf.clear();
        self.send_pos = 0;
        FlushResult::Done
    }

    // --- receiving -------------------------------------------------------------------------------

    /// Runs the parser over what has been received. Each complete frame is handed to its registered
    /// handler. Never panics on malformed input: bad frames put the connection into the error state and
    /// disconnect it.
    pub fn handle_receive<C>(
        &mut self,
        packets: &IncomingPackets<C>,
        ctx: &mut C,
        throttled: bool,
    ) -> ReceiveOutcome {
        if !self.running || self.stream.is_none() {
            return ReceiveOutcome::Done;
        }

        // The handlers take `&mut self` and a slice of the buffer, so the buffer is lifted out meanwhile.
        let mut recv = std::mem::take(&mut self.recv);
        let outcome = self.handle_receive_inner(&mut recv, packets, ctx, throttled);
        self.recv = recv;
        outcome
    }

    fn handle_error(&mut self, packet_id: u8, packet_length: i64) {
        let msg = format!(
            "{} entered bad state on packet 0x{packet_id:02X} with length {packet_length} while in protocol state {:?} and parser state {:?}",
            self.to_string, self.protocol_state, self.parser_state
        );
        self.disconnect(&msg);
        self.parser_state = ParserState::Error;
        self.protocol_state = ProtocolState::Error;
    }

    fn handle_receive_inner<C>(
        &mut self,
        recv: &mut RecvBuffer,
        packets: &IncomingPackets<C>,
        ctx: &mut C,
        throttled: bool,
    ) -> ReceiveOutcome {
        let mut outcome = ReceiveOutcome::Done;

        // Process as many packets as we can synchronously
        while self.running
            && self.parser_state != ParserState::Error
            && self.protocol_state != ProtocolState::Error
        {
            let length = recv.len();
            if length == 0 {
                break;
            }

            let packet_id = recv.readable()[0];
            let mut packet_length = length as i64;

            // These can arrive at any time and are only informational
            if self.protocol_state != ProtocolState::AwaitingSeed && is_info_packet(packet_id) {
                self.parser_state = ParserState::ProcessingPacket;
                (self.parser_state, packet_length) =
                    self.handle_packet(recv, packets, ctx, packet_id);
            } else {
                match self.protocol_state {
                    ProtocolState::AwaitingSeed => {
                        if !self.state_awaiting_seed(
                            recv,
                            packets,
                            ctx,
                            packet_id,
                            &mut packet_length,
                        ) {
                            return outcome;
                        }
                    }
                    ProtocolState::LoginServerAwaitingLogin => {
                        if !self.state_awaiting_login(
                            recv,
                            packets,
                            ctx,
                            packet_id,
                            &mut packet_length,
                        ) {
                            return outcome;
                        }
                    }
                    ProtocolState::LoginServerAwaitingServerSelect => {
                        if packet_id != 0xA0 {
                            self.handle_error(packet_id, packet_length);
                            return outcome;
                        }

                        self.parser_state = ParserState::ProcessingPacket;
                        (self.parser_state, packet_length) =
                            self.handle_packet(recv, packets, ctx, packet_id);
                        if self.parser_state == ParserState::AwaitingNextPacket {
                            self.protocol_state = ProtocolState::LoginServerServerSelectAck;
                            self.disconnect("");
                        }
                    }
                    ProtocolState::LoginServerServerSelectAck => {
                        // Reset the state because CUO/Orion do not reconnect (STRICT_UO_PROTOCOL would
                        // treat this as an error)
                        self.parser_state = ParserState::AwaitingNextPacket;
                        self.protocol_state = ProtocolState::AwaitingSeed;
                        return outcome;
                    }
                    ProtocolState::GameServerAwaitingGameServerLogin => {
                        // Some clients send 0x80 on game server connection
                        if packet_id == 0x80 || length == 62 {
                            if !self.state_awaiting_login(
                                recv,
                                packets,
                                ctx,
                                packet_id,
                                &mut packet_length,
                            ) {
                                return outcome;
                            }
                        } else if !self.state_awaiting_game_login(
                            recv,
                            packets,
                            ctx,
                            packet_id,
                            &mut packet_length,
                        ) {
                            return outcome;
                        }
                    }
                    ProtocolState::GameServerLoggedIn => {
                        self.parser_state = ParserState::ProcessingPacket;
                        (self.parser_state, packet_length) =
                            self.handle_packet(recv, packets, ctx, packet_id);
                    }
                    ProtocolState::Error => break,
                }
            }

            match self.parser_state {
                ParserState::AwaitingNextPacket => recv.commit_read(packet_length.max(0) as usize),
                ParserState::Throttled => {
                    outcome = if throttled {
                        ReceiveOutcome::ThrottledPending
                    } else {
                        ReceiveOutcome::Throttled
                    };
                    break;
                }
                ParserState::AwaitingPartialPacket => break,
                ParserState::Error => {
                    self.handle_error(packet_id, packet_length);
                    break;
                }
                ParserState::ProcessingPacket => {}
            }
        }

        outcome
    }

    /// `ProtocolState.AwaitingSeed`. Returns false when receive handling stops here.
    fn state_awaiting_seed<C>(
        &mut self,
        recv: &mut RecvBuffer,
        packets: &IncomingPackets<C>,
        ctx: &mut C,
        packet_id: u8,
        packet_length: &mut i64,
    ) -> bool {
        let length = recv.len();

        // Traffic that is positively another protocol. Unlike "does not look like a good client", this
        // cannot misfire on a misconfigured one.
        let (foreign, kind) = foreign_protocol::identify(recv.readable());

        if foreign == ForeignProtocolMatch::Incomplete {
            self.parser_state = ParserState::AwaitingPartialPacket;
            return true;
        }

        if foreign == ForeignProtocolMatch::Confirmed {
            debug!(
                "{} spoke {kind:?} on the game port; disconnecting",
                self.address
            );
            self.bad_connect = Some(crate::bans::ban_reasons::FOREIGN_PROTOCOL);
            self.disconnect("");
            return false;
        }

        if packet_id == 0xEF {
            self.parser_state = ParserState::ProcessingPacket;
            (self.parser_state, *packet_length) = self.handle_packet(recv, packets, ctx, packet_id);
            if self.parser_state == ParserState::AwaitingNextPacket {
                self.protocol_state = ProtocolState::LoginServerAwaitingLogin;
            }
        } else if length >= 4 {
            let r = recv.readable();
            let new_seed =
                ((r[0] as i32) << 24) | ((r[1] as i32) << 16) | ((r[2] as i32) << 8) | r[3] as i32;

            if new_seed == 0 {
                // No real client sends a zero seed, so this is deliberate garbage rather than a damaged
                // connection, unlike the short-read branch below, which a fragmented first segment can
                // reach honestly.
                self.bad_connect = Some(crate::bans::ban_reasons::INVALID_SEED);
                self.disconnect("");
                return false;
            }

            self.seed = new_seed;
            self.seeded = true;
            *packet_length = 4;

            self.parser_state = ParserState::AwaitingNextPacket;
            self.protocol_state = ProtocolState::GameServerAwaitingGameServerLogin;
        } else {
            // Disconnect rather than wait. Waiting would hold a connection slot for the full connecting
            // limit per one- or two-byte client, which is what a flood sends. Only pre-0xEF clients reach here.
            self.disconnect("");
        }

        true
    }

    /// `ProtocolState.LoginServer_AwaitingLogin` (also reached from the game-login state).
    fn state_awaiting_login<C>(
        &mut self,
        recv: &mut RecvBuffer,
        packets: &IncomingPackets<C>,
        ctx: &mut C,
        packet_id: u8,
        packet_length: &mut i64,
    ) -> bool {
        let length = recv.len();
        let cfg = self.encryption_config;
        let mut packet_id = packet_id;

        // Check for unencrypted login packet
        if packet_id == 0x80 {
            if cfg.enabled()
                && !cfg
                    .mode
                    .contains(crate::encryption::EncryptionMode::UNENCRYPTED)
            {
                self.log_info("Unencrypted client rejected by encryption policy.");
                self.handle_error(packet_id, *packet_length);
                return false;
            }

            self.parser_state = ParserState::ProcessingPacket;
            (self.parser_state, *packet_length) = self.handle_packet(recv, packets, ctx, packet_id);
            if self.parser_state == ParserState::AwaitingNextPacket {
                self.protocol_state = ProtocolState::LoginServerAwaitingServerSelect;
            }
            return true;
        }

        // First byte isn't 0x80 - might be encrypted
        if !cfg.enabled() {
            self.log_info("Possible encrypted client detected, disconnecting...");
            self.handle_error(packet_id, *packet_length);
            return false;
        }

        // Need 62 bytes for login packet to attempt decryption
        if length < 62 {
            self.parser_state = ParserState::AwaitingPartialPacket;
            return true;
        }

        // Try to detect and decrypt encrypted login
        let detected = detect_login_encryption(
            &cfg,
            Some(self.address),
            self.version.as_ref(),
            self.seed as u32,
            &recv.readable()[..62],
        );
        match detected {
            Detected::Rejected => {
                self.log_info("Encrypted client detection failed, disconnecting...");
                self.handle_error(packet_id, *packet_length);
                return false;
            }
            Detected::Unencrypted => {}
            Detected::Encrypted(mut enc) => {
                // Decrypt the buffer in place for processing
                let r = recv.start..recv.start + 62;
                enc.client_decrypt(recv.slice_mut(r));
                self.encryption = Some(enc);
            }
        }

        // Now process as normal (first byte should now be 0x80)
        packet_id = recv.readable()[0];

        self.parser_state = ParserState::ProcessingPacket;
        (self.parser_state, *packet_length) = self.handle_packet(recv, packets, ctx, packet_id);
        if self.parser_state == ParserState::AwaitingNextPacket {
            self.protocol_state = ProtocolState::LoginServerAwaitingServerSelect;
        }
        true
    }

    /// The rest of `ProtocolState.GameServer_AwaitingGameServerLogin`.
    fn state_awaiting_game_login<C>(
        &mut self,
        recv: &mut RecvBuffer,
        packets: &IncomingPackets<C>,
        ctx: &mut C,
        packet_id: u8,
        packet_length: &mut i64,
    ) -> bool {
        let length = recv.len();
        let cfg = self.encryption_config;
        let mut packet_id = packet_id;

        // Check for unencrypted game login packet
        if packet_id == 0x91 {
            if cfg.enabled()
                && !cfg
                    .mode
                    .contains(crate::encryption::EncryptionMode::UNENCRYPTED)
            {
                self.log_info("Unencrypted game client rejected by encryption policy.");
                self.handle_error(packet_id, *packet_length);
                return false;
            }

            self.parser_state = ParserState::ProcessingPacket;
            (self.parser_state, *packet_length) = self.handle_packet(recv, packets, ctx, packet_id);
            if self.parser_state == ParserState::AwaitingNextPacket {
                self.protocol_state = ProtocolState::GameServerLoggedIn;
            }
            return true;
        }

        // First byte isn't 0x91 - might be encrypted
        if !cfg.enabled() {
            self.handle_error(packet_id, *packet_length);
            return false;
        }

        // Need 65 bytes for game login packet to attempt decryption
        if length < 65 {
            self.parser_state = ParserState::AwaitingPartialPacket;
            return true;
        }

        match detect_game_encryption(
            &cfg,
            Some(self.address),
            self.seed as u32,
            &recv.readable()[..65],
        ) {
            Detected::Rejected => {
                self.log_info("Encrypted game client detection failed, disconnecting...");
                self.handle_error(packet_id, *packet_length);
                return false;
            }
            Detected::Unencrypted => {}
            Detected::Encrypted(mut enc) => {
                let r = recv.start..recv.start + 65;
                enc.client_decrypt(recv.slice_mut(r));
                self.encryption = Some(enc);
            }
        }

        // Now process as normal (first byte should now be 0x91)
        packet_id = recv.readable()[0];

        self.parser_state = ParserState::ProcessingPacket;
        (self.parser_state, *packet_length) = self.handle_packet(recv, packets, ctx, packet_id);
        if self.parser_state == ParserState::AwaitingNextPacket {
            self.protocol_state = ProtocolState::GameServerLoggedIn;
        }
        true
    }

    /// Frames one packet: `length` is the buffer length, `packet_length` the length this function found.
    fn handle_packet<C>(
        &mut self,
        recv: &RecvBuffer,
        packets: &IncomingPackets<C>,
        ctx: &mut C,
        packet_id: u8,
    ) -> (ParserState, i64) {
        let Some(handler) = packets.get_handler(packet_id) else {
            self.log_info(&format!(
                "Received unknown packet 0x{packet_id:02X} while in state {:?}",
                self.protocol_state
            ));
            return (ParserState::Error, 1);
        };

        let buffer = recv.readable();
        let length = buffer.len() as i64;
        let mut header = 1usize;

        let mut packet_length = handler.get_length(self) as i64;
        if packet_length <= 0 {
            // Variable length packet. See if we have pulled in the length.
            if length < 3 {
                return (ParserState::AwaitingPartialPacket, packet_length);
            }

            packet_length = u16::from_be_bytes([buffer[1], buffer[2]]) as i64;
            header = 3;
            if packet_length < 3 {
                return (ParserState::Error, packet_length);
            }
        }

        // Can never complete: the buffer holds one byte less than its size and a receive arms only into free space
        if packet_length >= RECV_BUFFER_SIZE as i64 {
            self.log_info(&format!(
                "Received packet 0x{packet_id:02X} declaring {packet_length} bytes, more than the receive buffer holds."
            ));
            return (ParserState::Error, packet_length);
        }

        // Not enough data, let's wait for more to come in
        if length < packet_length {
            return (ParserState::AwaitingPartialPacket, packet_length);
        }

        // A fixed length shorter than its own id would leave no body to slice
        if (packet_length as usize) < header {
            return (ParserState::Error, packet_length);
        }

        if handler.in_game_only {
            match self.attachment {
                Attachment::None => {
                    self.log_info(&format!(
                        "Received packet 0x{packet_id:02X} before having been attached to a mobile."
                    ));
                    return (ParserState::Error, packet_length);
                }
                Attachment::Deleted => {
                    self.log_info(&format!(
                        "Received packet 0x{packet_id:02X} after having been attached to a deleted mobile."
                    ));
                    return (ParserState::Error, packet_length);
                }
                Attachment::Mobile(_) => {}
            }
        }

        if handler.out_of_game_only && matches!(self.attachment, Attachment::Mobile(_)) {
            self.log_info(&format!(
                "Received packet 0x{packet_id:02X} after having been attached to a mobile."
            ));
            return (ParserState::Error, packet_length);
        }

        if let Some(throttle) = handler.throttle {
            if throttle(ctx, packet_id as i32, self) {
                return (ParserState::Throttled, packet_length);
            }
            self.set_packet_time(packet_id);
        }

        self.update_packet_count(packet_id);

        if self.packet_logging {
            self.log_packet(&buffer[..packet_length as usize], true);
        }

        // The handler's view is limited to this packet, so `remaining` style reads stop at its end
        let body = &buffer[header..packet_length as usize];
        (handler.on_receive)(ctx, self, body);

        (ParserState::AwaitingNextPacket, packet_length)
    }

    // --- diagnostics logs ------------------------------------------------------------------------

    /// `Trace`: appends the unhandled packet to `unhandled-packets.log`.
    pub fn trace(&self, buffer: &[u8]) {
        let Some(first) = buffer.first() else { return };
        let _ = (|| -> std::io::Result<()> {
            let mut f = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open("unhandled-packets.log")?;
            let text = format!(
                "Client: {}: Unhandled packet 0x{:02X}\r\n{}\r\n\r\n",
                self.to_string,
                first,
                crate::logs::format_buffer(buffer)
            );
            f.write_all(text.as_bytes())
        })();
    }
}

impl std::fmt::Display for NetState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.to_string)
    }
}

impl PartialEq for NetState {
    fn eq(&self, o: &Self) -> bool {
        self.serial == o.serial
    }
}

impl NetState {
    /// `CompareTo`: ordinal comparison of the address text.
    pub fn compare_to(&self, other: &NetState) -> std::cmp::Ordering {
        self.to_string.cmp(&other.to_string)
    }
}

#[cfg(test)]
mod count_tests {
    use super::*;

    #[test]
    fn check_packet_counts_stops_at_the_first_hit_and_keeps_the_later_counts() {
        let mut ns = NetState::new(
            0,
            0,
            None,
            "127.0.0.1".parse().unwrap(),
            0,
            4096,
            EncryptionConfig::default(),
            String::new(),
        );
        for _ in 0..=PACKET_PER_SECOND_THRESHOLD {
            ns.update_packet_count(0x02);
            ns.update_packet_count(0x80);
        }
        assert_eq!(ns.check_packet_counts(), 0x02);
        assert_eq!(ns.check_packet_counts(), 0x80); // 0x80 was not cleared by the first call
        assert_eq!(ns.check_packet_counts(), 0);
    }
}
