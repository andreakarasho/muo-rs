//! Listeners, the accept path and the per-tick `slice`, over `mio`.
//!
//! Every connection owns plain `Vec` buffers and one nonblocking poll per game tick (`slice`) does, in
//! order: unattached-socket sweep, throttled re-parse, accept/receive/send events, flush, graceful
//! disconnect handoff, disposal, and the 5-second liveness check.
//!
//! shortcut: no send-buffer growth tiers, pre-auth buffer pools, or memory-ceiling refusals. A connection
//! may queue up to `max_send_buffer_size - 1` bytes (default 2 MiB) and is disconnected as "send buffer
//! exhausted" beyond that. Upgrade if per-connection memory
//! becomes a concern.

use crate::bans::{BanChannel, BanSettings, ban_reasons};
use crate::connection_filters::ConnectionFilters;
use crate::encryption::EncryptionConfig;
use crate::ip_rate_limiter::IpRateLimiter;
use crate::logs;
use crate::net_state::{
    ACTIVITY_TIMEOUT_MS, FlushResult, NetState, RECV_BUFFER_SIZE, ReadResult, ReceiveOutcome,
};
use crate::network_stats::NetworkStats;
use crate::packet_handler::IncomingPackets;
use mio::net::{TcpListener, TcpStream};
use mio::{Events, Interest, Poll, Token, Waker};
use std::collections::VecDeque;
use std::io;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, info, warn};

const DEFAULT_SEND_BUFFER_SIZE: i64 = 1024 * 256;
const MIN_SEND_BUFFER_SIZE: i64 = 1024 * 64;
const DEFAULT_MAX_SEND_BUFFER_SIZE: i64 = 1024 * 1024 * 2;
/// Transport ceiling; larger values overflow its tier enumeration.
const TRANSPORT_MAX_SEND_BUFFER_SIZE: i64 = 1024 * 1024 * 256;
pub const MAX_CONNECTIONS: usize = 4096;
const ALIVE_CHECK_INTERVAL_MS: i64 = 5000;
/// A connection that has not seeded and sent its first packet by now is dropped.
const CONNECTING_SOCKET_IDLE_LIMIT_MS: i64 = 5000;
const ACCEPTS_PER_LISTENER_PER_SLICE: usize = 1024;
const READS_PER_CONNECTION_PER_SLICE: usize = 16;
const LISTENER_BASE: usize = 1 << 24;
const WAKER_TOKEN: usize = (1 << 24) - 1;

/// Clamps to a power of two between `minimum` and the transport ceiling
/// (`NetState.CoercePowerOfTwoSetting`), warning when it has to adjust.
pub fn coerce_power_of_two_setting(key: &str, configured: i64, minimum: i64) -> i64 {
    let mut size = configured;

    if size > TRANSPORT_MAX_SEND_BUFFER_SIZE {
        warn!(
            "{key} {configured} is above the transport maximum {TRANSPORT_MAX_SEND_BUFFER_SIZE} (capped); using {TRANSPORT_MAX_SEND_BUFFER_SIZE}"
        );
        size = TRANSPORT_MAX_SEND_BUFFER_SIZE;
    }

    if size < minimum {
        warn!("{key} {configured} is below the minimum {minimum} (raised); using {minimum}");
        size = minimum;
    }

    if size > 0 && (size & (size - 1)) != 0 {
        // Rounding up cannot cross the ceiling, itself a power of two
        let rounded = (size as u64).next_power_of_two() as i64;
        warn!("{key} {configured} is not a power of two; using {rounded}");
        size = rounded;
    }

    size
}

/// Network settings. `listeners` is `ServerConfiguration.Listeners`; the rest are the `network.*` keys.
#[derive(Clone, Debug)]
pub struct NetworkConfig {
    pub listeners: Vec<SocketAddr>,
    /// `network.sendBufferSize`
    pub send_buffer_size: usize,
    /// `network.sendBufferMaxSize`
    pub max_send_buffer_size: usize,
    pub max_connections: usize,
    pub encryption: EncryptionConfig,
    pub ban_settings: BanSettings,
    /// `netstate.packetLoggingPath` (default `<base>/Packets`)
    pub packet_logging_path: String,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            listeners: Vec::new(),
            send_buffer_size: DEFAULT_SEND_BUFFER_SIZE as usize,
            max_send_buffer_size: DEFAULT_MAX_SEND_BUFFER_SIZE as usize,
            max_connections: MAX_CONNECTIONS,
            encryption: EncryptionConfig::default(),
            ban_settings: BanSettings::default(),
            packet_logging_path: "Packets".to_string(),
        }
    }
}

impl NetworkConfig {
    /// Applies the configured (possibly unusable) buffer sizes the way `ConfigureNetwork` does.
    pub fn with_buffer_settings(
        mut self,
        send_buffer_size: i64,
        send_buffer_max_size: i64,
    ) -> Self {
        let send = coerce_power_of_two_setting(
            "network.sendBufferSize",
            send_buffer_size,
            MIN_SEND_BUFFER_SIZE,
        );
        let max =
            coerce_power_of_two_setting("network.sendBufferMaxSize", send_buffer_max_size, send);
        self.send_buffer_size = send as usize;
        self.max_send_buffer_size = max as usize;
        self
    }
}

/// What the server needs to hear about connection lifecycles.
pub enum NetEvent {
    /// A connection was accepted; look it up with [`Network::get`].
    Connected(usize),
    /// A connection was disposed. The state is handed over for the server to detach its mobile and account.
    Disconnected(Box<NetState>),
}

pub struct Network<C> {
    pub config: NetworkConfig,
    pub packets: IncomingPackets<C>,
    pub filters: ConnectionFilters,
    pub bans: BanChannel,
    pub rate_limiter: IpRateLimiter,
    /// `EventSink.InvokeSocketConnect`: return false to reject the connection.
    pub socket_connect: Option<Box<dyn FnMut(IpAddr) -> bool>>,

    poll: Poll,
    events: Events,
    waker: Arc<Waker>,
    listeners: Vec<TcpListener>,
    listening_addresses: Vec<SocketAddr>,
    conns: Vec<Option<NetState>>,
    free_slots: Vec<usize>,
    instances: usize,
    next_serial: u64,
    now: i64,

    connecting: VecDeque<(usize, u64)>,
    throttled: Vec<usize>,
    throttled_pending: Vec<usize>,
    closed: Vec<usize>,
    closed_later: Vec<usize>,
    pre_polled: Vec<(Token, bool, bool)>,
    events_out: Vec<NetEvent>,
    next_alive_check: i64,
}

impl<C> Network<C> {
    /// `ConfigureNetwork`. `now` is the current tick count.
    pub fn new(config: NetworkConfig, now: i64) -> io::Result<Self> {
        let poll = Poll::new()?;
        let waker = Arc::new(Waker::new(poll.registry(), Token(WAKER_TOKEN))?);
        let mut conns = Vec::new();
        conns.resize_with(config.max_connections, || None);
        let free_slots = (0..config.max_connections).rev().collect();

        Ok(Self {
            config,
            packets: IncomingPackets::new(),
            filters: ConnectionFilters::default(),
            bans: BanChannel::default(),
            rate_limiter: IpRateLimiter::default_for_server(),
            socket_connect: None,
            poll,
            events: Events::with_capacity(MAX_CONNECTIONS * 2),
            waker,
            listeners: Vec::new(),
            listening_addresses: Vec::new(),
            conns,
            free_slots,
            instances: 0,
            next_serial: 1,
            now,
            connecting: VecDeque::new(),
            throttled: Vec::new(),
            throttled_pending: Vec::new(),
            closed: Vec::new(),
            closed_later: Vec::new(),
            pre_polled: Vec::new(),
            events_out: Vec::new(),
            // Seed from a real tick; a zero default suppresses the sweep when ticks start negative
            next_alive_check: now,
        })
    }

    /// Starts listening on the configured addresses. Returns the addresses actually bound.
    pub fn start(&mut self) -> Vec<SocketAddr> {
        for (i, addr) in self.config.listeners.clone().into_iter().enumerate() {
            let mut listener = match TcpListener::bind(addr) {
                Ok(l) => l,
                Err(e) => {
                    warn!("Failed to create listener for {addr}: {e}");
                    continue;
                }
            };

            let token = Token(LISTENER_BASE + self.listeners.len());
            if let Err(e) = self
                .poll
                .registry()
                .register(&mut listener, token, Interest::READABLE)
            {
                warn!("Failed to create listener for {addr}: {e}");
                continue;
            }
            let _ = i;

            // shortcut: a wildcard listener reports itself, not each NIC address (needs interface
            // enumeration); upgrade when the console banner must list them.
            self.listening_addresses
                .push(listener.local_addr().unwrap_or(addr));
            self.listeners.push(listener);
        }

        for a in &self.listening_addresses {
            info!("Listening: {a}");
        }
        self.listening_addresses.clone()
    }

    pub fn listening_addresses(&self) -> &[SocketAddr] {
        &self.listening_addresses
    }

    /// Closes all listeners.
    pub fn shutdown(&mut self) {
        for mut l in self.listeners.drain(..) {
            let _ = self.poll.registry().deregister(&mut l);
        }
    }

    /// Wakes the game loop if it is blocked in [`Network::wait_for_completion`]; safe from any thread.
    pub fn waker(&self) -> Arc<Waker> {
        self.waker.clone()
    }

    /// Waits for network events or until the timeout expires, so the loop can sleep and stay responsive.
    /// The events are kept for the next `slice`.
    pub fn wait_for_completion(&mut self, timeout_ms: u64) {
        self.poll_events(Some(Duration::from_millis(timeout_ms)));
    }

    fn poll_events(&mut self, timeout: Option<Duration>) {
        if self.poll.poll(&mut self.events, timeout).is_err() {
            return;
        }
        for ev in self.events.iter() {
            self.pre_polled.push((
                ev.token(),
                ev.is_readable() || ev.is_read_closed() || ev.is_error(),
                ev.is_writable(),
            ));
        }
    }

    pub fn get(&self, id: usize) -> Option<&NetState> {
        self.conns.get(id).and_then(Option::as_ref)
    }

    pub fn get_mut(&mut self, id: usize) -> Option<&mut NetState> {
        self.conns.get_mut(id).and_then(Option::as_mut)
    }

    /// All live connections (`NetState.Instances`).
    pub fn instances(&self) -> impl Iterator<Item = &NetState> {
        self.conns.iter().filter_map(Option::as_ref)
    }

    pub fn instances_mut(&mut self) -> impl Iterator<Item = &mut NetState> {
        self.conns.iter_mut().filter_map(Option::as_mut)
    }

    pub fn instance_count(&self) -> usize {
        self.instances
    }

    /// Lifecycle events since the last call.
    pub fn take_events(&mut self) -> Vec<NetEvent> {
        std::mem::take(&mut self.events_out)
    }

    /// True when no queued network work remains for the loop to drain.
    pub fn is_idle(&self) -> bool {
        self.throttled.is_empty()
            && self.throttled_pending.is_empty()
            && self.closed.is_empty()
            && self.closed_later.is_empty()
            && !self
                .instances()
                .any(|n| n.disconnect_queued || n.pending_send_bytes() > 0)
    }

    /// Read-only diagnostics snapshot.
    pub fn get_network_stats(&self) -> NetworkStats {
        NetworkStats {
            connected: self.instances as i32,
            max_connections: self.config.max_connections as i32,
            authenticated: self.instances().filter(|n| n.account.is_some()).count() as i32,
            recv_buffer_size: RECV_BUFFER_SIZE as i32,
            send_buffer_size: self.config.send_buffer_size as i32,
            max_send_buffer_size: self.config.max_send_buffer_size as i32,
            throttled: (self.throttled.len() + self.throttled_pending.len()) as i32,
            flush_pending: self
                .instances()
                .filter(|n| n.pending_send_bytes() > 0)
                .count() as i32,
            pending_disconnects: self.instances().filter(|n| n.disconnect_queued).count() as i32,
            ..Default::default()
        }
    }

    // --- accept path -----------------------------------------------------------------------------

    fn accept_ready(&mut self, listener_index: usize) {
        for _ in 0..ACCEPTS_PER_LISTENER_PER_SLICE {
            let accepted = match self.listeners.get(listener_index) {
                Some(l) => l.accept(),
                None => return,
            };

            match accepted {
                Ok((stream, peer)) => self.handle_accept(stream, peer.ip()),
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => return,
                Err(e) if e.kind() == io::ErrorKind::Interrupted => {}
                Err(e) => {
                    debug!("Accept error: {e}");
                    return;
                }
            }
        }
    }

    fn handle_accept(&mut self, stream: TcpStream, remote_ip: IpAddr) {
        let (allowed, total_attempts) = self.rate_limiter.verify(remote_ip, self.now);
        if !allowed {
            debug!("{remote_ip} Past IP limit threshold ({total_attempts})");

            if self.config.ban_settings.report_rate_limit_trips {
                // Enqueue-only contribution; NOT added to the local firewall set (the limiter already gates
                // it here and the OS bouncer drops it at the kernel).
                self.bans.report(
                    remote_ip,
                    self.config.ban_settings.auto_ban_duration,
                    ban_reasons::RATE_LIMIT,
                );
            }
            return;
        }

        if let Some(denied_by) = self.filters.should_deny(remote_ip) {
            // Whatever a hit implies is the filter's own business; the accept path just drops the socket.
            debug!("{remote_ip} denied by connection filter '{denied_by}'");
            return;
        }

        if let Some(hook) = self.socket_connect.as_mut()
            && !hook(remote_ip)
        {
            debug!("{remote_ip} Rejected by socket handler");
            return;
        }

        self.create_from_socket(stream, remote_ip);
    }

    /// Creates a NetState from an accepted socket.
    pub fn create_from_socket(&mut self, mut stream: TcpStream, address: IpAddr) -> Option<usize> {
        let Some(id) = self.free_slots.pop() else {
            debug!("Failed to create socket (resources exhausted)");
            return None;
        };

        let _ = stream.set_nodelay(true);
        if let Err(e) = self.poll.registry().register(
            &mut stream,
            Token(id),
            Interest::READABLE | Interest::WRITABLE,
        ) {
            debug!("Failed to register socket: {e}");
            self.free_slots.push(id);
            return None;
        }

        let serial = self.next_serial;
        self.next_serial += 1;

        let mut ns = NetState::new(
            id,
            serial,
            Some(stream),
            address,
            self.now,
            self.config.max_send_buffer_size,
            self.config.encryption,
            self.config.packet_logging_path.clone(),
        );
        // Edge-triggered readiness may have fired before registration: look once
        ns.read_pending = true;

        self.conns[id] = Some(ns);
        self.instances += 1;
        self.connecting.push_back((id, serial));
        self.events_out.push(NetEvent::Connected(id));

        info!("Client: {address}: Connected. [{} Online]", self.instances);
        Some(id)
    }

    // --- the tick --------------------------------------------------------------------------------

    /// One game-tick pass over the network (`NetState.Slice`). `now` is the wrapping tick count. Handlers
    /// run inside this call with `ctx`; movement-queue processing between the throttled re-parse and the
    /// events (`MovementThrottle.ProcessAllQueues`) is the server's to call before this.
    pub fn slice(&mut self, ctx: &mut C, now: i64) {
        self.now = now;
        let cur_ticks = now;
        self.closed.append(&mut self.closed_later);
        for ns in self.conns.iter_mut().flatten() {
            ns.now = now;
        }

        self.disconnect_unattached_sockets(now);

        // Process throttled states
        for id in std::mem::take(&mut self.throttled) {
            self.receive_throttled(ctx, id);
        }

        // Enqueued by handle_receive if already throttled and still throttled
        let pending = std::mem::take(&mut self.throttled_pending);
        self.throttled.extend(pending);

        // One poll per tick
        self.poll_events(Some(Duration::ZERO));
        let ready = std::mem::take(&mut self.pre_polled);
        for (token, readable, writable) in ready {
            match token.0 {
                WAKER_TOKEN => {}
                t if t >= LISTENER_BASE => self.accept_ready(t - LISTENER_BASE),
                id => {
                    if let Some(ns) = self.conns.get_mut(id).and_then(Option::as_mut) {
                        ns.read_pending |= readable;
                        ns.writable_event |= writable;
                    }
                }
            }
        }

        // Receive: events first, so the sends handlers make are flushed in this same slice
        for id in 0..self.conns.len() {
            if self.conns[id]
                .as_ref()
                .is_some_and(|n| n.read_pending && n.running)
            {
                self.read_ready(ctx, id);
            }
        }

        // Flush AFTER event processing
        for id in 0..self.conns.len() {
            self.flush_conn(id);
        }

        // Disconnects are handed to the socket AFTER the flush: game logic (sends/disconnects), then
        // receives, then flush, then disconnect; a send after disconnect() in the same tick still goes out.
        for id in 0..self.conns.len() {
            let Some(ns) = self.conns[id].as_mut() else {
                continue;
            };
            if !ns.disconnect_queued {
                continue;
            }

            // Reset the flag to allow re-queueing if a reconnect happens
            ns.disconnect_queued = false;

            let mut closed = false;
            if ns.running && ns.stream.is_some() {
                if ns.pending_send_bytes() == 0 {
                    ns.finish_close();
                    closed = true;
                } else {
                    // Waits for pending sends to flush
                    ns.disconnect_pending = true;
                    ns.arm_drain_deadline(cur_ticks);
                }
            }
            if closed {
                self.mark_closed(id, true);
            }
        }

        // Process disposes
        for id in std::mem::take(&mut self.closed) {
            self.dispose(id);
        }

        // Check for dead connections AFTER processing all events. Receives reset next_activity_check, so
        // after a server stall, buffered client pings update timestamps before this check fires.
        if cur_ticks - self.next_alive_check >= 0 {
            self.next_alive_check = cur_ticks + ALIVE_CHECK_INTERVAL_MS;
            self.check_all_alive(cur_ticks);
        }
    }

    /// Writes out everything queued now (`FlushAll`), e.g. before shutdown.
    pub fn flush_all(&mut self) {
        for id in 0..self.conns.len() {
            self.flush_conn(id);
        }
    }

    fn mark_closed(&mut self, id: usize, later: bool) {
        let Some(ns) = self.conns[id].as_mut() else {
            return;
        };
        if ns.close_queued {
            return;
        }
        ns.close_queued = true;
        if later {
            self.closed_later.push(id);
        } else {
            self.closed.push(id);
        }
    }

    fn flush_conn(&mut self, id: usize) {
        let Some(ns) = self.conns[id].as_mut() else {
            return;
        };
        ns.writable_event = false;

        let mut closed = false;
        if !ns.running || ns.stream.is_none() {
            return;
        }

        if ns.pending_send_bytes() == 0 {
            // A draining disconnect whose output is gone completes now
            if ns.disconnect_pending {
                ns.finish_close();
                closed = true;
            }
        } else {
            match ns.flush_send() {
                FlushResult::Done => {
                    if ns.disconnect_pending {
                        ns.finish_close();
                        closed = true;
                    }
                }
                FlushResult::Blocked => {}
                FlushResult::Closed => {
                    ns.disconnect_immediate();
                    closed = true;
                }
            }
        }

        if closed {
            self.mark_closed(id, false);
        }
    }

    fn receive_throttled(&mut self, ctx: &mut C, id: usize) {
        let Some(ns) = self.conns[id].as_mut() else {
            return;
        };
        if !ns.running {
            return;
        }

        let outcome = ns.handle_receive(&self.packets, ctx, true);
        self.after_receive(id, outcome);
    }

    fn after_receive(&mut self, id: usize, outcome: ReceiveOutcome) {
        match outcome {
            ReceiveOutcome::Done => {}
            ReceiveOutcome::Throttled => {
                if !self.throttled.contains(&id) {
                    self.throttled.push(id);
                }
            }
            ReceiveOutcome::ThrottledPending => {
                if !self.throttled_pending.contains(&id) {
                    self.throttled_pending.push(id);
                }
            }
        }

        let Some(ns) = self.conns[id].as_mut() else {
            return;
        };
        if let Some(reason) = ns.bad_connect.take()
            && self.config.ban_settings.report_bad_connects
        {
            let (addr, ttl) = (ns.address, self.config.ban_settings.bad_connect_duration);
            self.bans.report(addr, ttl, reason);
        }
    }

    /// Reads whatever is ready on a connection, parsing after each chunk like a completion would.
    fn read_ready(&mut self, ctx: &mut C, id: usize) {
        for _ in 0..READS_PER_CONNECTION_PER_SLICE {
            let Some(ns) = self.conns[id].as_mut() else {
                return;
            };
            if !ns.running || ns.stream.is_none() {
                return;
            }

            match ns.read_chunk() {
                ReadResult::Data => {
                    let outcome = ns.handle_receive(&self.packets, ctx, false);
                    self.after_receive(id, outcome);
                }
                ReadResult::WouldBlock => {
                    ns.read_pending = false;
                    return;
                }
                // Parser has not freed room yet (partial packet or throttled): try again next slice
                ReadResult::Full => return,
                ReadResult::Closed => {
                    ns.read_pending = false;
                    ns.disconnect_immediate();
                    self.mark_closed(id, false);
                    return;
                }
            }
        }
        // Budget spent with data possibly left: stays read_pending for the next slice
    }

    /// Drops connections that never finished the seed/first-packet handshake within the idle limit.
    fn disconnect_unattached_sockets(&mut self, now: i64) {
        // Lazy removal: the queue is ordered by connect time, so the first fresh entry ends the sweep
        while let Some(&(id, serial)) = self.connecting.front() {
            let Some(ns) = self.conns[id].as_mut().filter(|n| n.serial == serial) else {
                self.connecting.pop_front();
                continue;
            };

            // Skip already-authenticated or disconnected connections
            if !ns.running || ns.account.is_some() {
                self.connecting.pop_front();
                continue;
            }

            if now - ns.connected_tick < CONNECTING_SOCKET_IDLE_LIMIT_MS {
                break;
            }

            self.connecting.pop_front();

            // Socket must have finished the entire authentication process or be forcibly disconnected
            if ns.sent_first_packet && ns.seeded {
                continue;
            }

            // Only the totally silent ones are evidence. A connection that sent SOME data and ran out of
            // time is far more likely a slow link, and banning those makes the player retry, trip the rate
            // limiter, and compound it into an hours-long ban.
            let silent = !ns.received_data;
            let addr = ns.address;
            ns.disconnect("");

            // Force immediate cleanup: graceful disconnect can get stuck with pending sends
            let force = ns.disconnect_pending;
            if force {
                ns.disconnect_immediate();
            }

            if silent && self.config.ban_settings.report_bad_connects {
                self.bans.report(
                    addr,
                    self.config.ban_settings.bad_connect_duration,
                    ban_reasons::SILENT_CONNECT,
                );
            }
            if force {
                self.mark_closed(id, false);
            }
        }
    }

    /// `CheckAllAlive`
    pub fn check_all_alive(&mut self, cur_ticks: i64) {
        for id in 0..self.conns.len() {
            let Some(ns) = self.conns[id].as_mut() else {
                continue;
            };
            let had_stream = ns.stream.is_some();
            ns.check_alive(cur_ticks);
            let closed = had_stream && ns.stream.is_none();
            if closed {
                self.mark_closed(id, true);
            }
        }
    }

    /// Tears a closed connection down and hands it to the server.
    fn dispose(&mut self, id: usize) {
        let Some(mut ns) = self.conns[id].take() else {
            return;
        };
        ns.running = false;
        logs::trace_disconnect(&ns.disconnect_reason, &ns.address.to_string());

        self.instances = self.instances.saturating_sub(1);
        self.free_slots.push(id);
        self.throttled.retain(|&t| t != id);
        self.throttled_pending.retain(|&t| t != id);

        match &ns.account {
            Some(a) => info!(
                "Client: {}: Disconnected. [{} Online] [{a}]",
                ns.address, self.instances
            ),
            None => info!(
                "Client: {}: Disconnected. [{} Online]",
                ns.address, self.instances
            ),
        }

        self.events_out.push(NetEvent::Disconnected(Box::new(ns)));
    }

    /// Ticks until `next_activity_check` for a fresh connection; exposed for tests of the sweep.
    pub fn activity_timeout_ms() -> i64 {
        ACTIVITY_TIMEOUT_MS
    }
}

#[cfg(test)]
#[path = "network_tests.rs"]
mod tests;
