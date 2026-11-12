//! The entity-aware parts of the network and movement code of `NetState`:
//! the glue between `muo_net::Network<World>` (which lives beside the world) and the `World`.
//!
//! - [`NetworkHooks`] owns the `Network<World>`; it is the loop's `network_slice`: movement queues, outbox pump,
//!   `Network::slice` (handlers run inside it with `&mut World`), connect/disconnect events, outbox pump, flush.
//! - [`WorldMovementActor`] is muo_net's `MovementActor` over a `World` (the movement throttle applied to a
//!   `Mobile` and its `NetState`).
//! - [`dispose_net_state`] is the world half of `NetState.Dispose`.

use muo_net::movement::{DetectionVerdict, MovementActor, MovementState};
use muo_net::net_state::NetState;
use muo_net::network::{NetEvent, Network};
use muo_primitives::Direction;
use muo_primitives::utility;

use crate::core_loop::LoopHooks;
use crate::ids::MobileId;
use crate::mobile::AccessLevel;
use crate::net_state::{NetOut, NetStateData, NetStateId};
use crate::world::World;

/// `MovementThrottle.OnSpeedHackDetected`: subscribers get the world, the connection, its mobile, the rate, the
/// verdict and the urgency text.
#[allow(clippy::type_complexity)]
#[derive(Default)]
pub struct SpeedHackListeners(
    pub Vec<Box<dyn FnMut(&mut World, NetStateId, MobileId, f32, DetectionVerdict, &str)>>,
);

/// The transport's last `GetNetworkStats()`, for gumps (the `Network` lives beside the world).
#[derive(Default)]
pub struct NetStatsSnapshot {
    pub at: i64,
    pub stats: muo_net::network_stats::NetworkStats,
    /// `DumpNetStates` was run: the next `slice` (which holds the transport) writes the CSV.
    pub dump_requested: bool,
}

/// `DumpNetStates.Configure`: `[DumpNetStates` (Developer) asks the pump for a CSV of every connection.
pub fn dump_net_states_configure(w: &mut World) {
    crate::commands::CommandSystem::register(
        w,
        "DumpNetStates",
        AccessLevel::Developer,
        &[],
        std::rc::Rc::new(|w, _| w.resources.get_mut::<NetStatsSnapshot>().dump_requested = true),
    );
}

fn dump_net_states<C>(w: &World, net: &Network<C>) {
    use muo_net::net_state::{ParserState, ProtocolState};
    let (y, mo, d) = w.now.ymd();
    let name = format!(
        "netstatedump-{y}-{mo}-{d}-{:02}-{:02}-{:02}_{}.csv",
        w.now.hour(),
        w.now.minute(),
        w.now.second(),
        w.tick_count
    );
    let mut out = String::from(
        "NetState, ConnectedOn, NextActivityCheck, SocketConnected, ProtocolState, ParserState, SendBufferSize
",
    );
    for ns in net.instances() {
        let protocol = match ns.protocol_state {
            ProtocolState::AwaitingSeed => "AwaitingSeed",
            ProtocolState::LoginServerAwaitingLogin => "LoginServer_AwaitingLogin",
            ProtocolState::LoginServerAwaitingServerSelect => "LoginServer_AwaitingServerSelect",
            ProtocolState::LoginServerServerSelectAck => "LoginServer_ServerSelectAck",
            ProtocolState::GameServerAwaitingGameServerLogin => {
                "GameServer_AwaitingGameServerLogin"
            }
            ProtocolState::GameServerLoggedIn => "GameServer_LoggedIn",
            ProtocolState::Error => "Error",
        };
        let parser = match ns.parser_state {
            ParserState::AwaitingNextPacket => "AwaitingNextPacket",
            ParserState::AwaitingPartialPacket => "AwaitingPartialPacket",
            ParserState::ProcessingPacket => "ProcessingPacket",
            ParserState::Throttled => "Throttled",
            ParserState::Error => "Error",
        };
        let secs = ns
            .connected_on
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_secs() as i64);
        let on = muo_primitives::DateTime::from_unix_seconds(secs)
            .unwrap_or(muo_primitives::DateTime::MIN);
        out.push_str(&format!(
            "{ns}, {on}, {}, {}, {protocol}, {parser}, {}
",
            ns.next_activity_check,
            if ns.is_connected() { "True" } else { "False" },
            ns.pending_send_bytes()
        ));
    }
    if let Err(e) = std::fs::write(&name, out) {
        tracing::warn!("DumpNetStates: {name}: {e}");
    }
}

/// `MovementThrottle`'s world-facing operations, as `muo_net`'s trait wants them.
pub struct WorldMovementActor<'a> {
    pub w: &'a mut World,
    pub ns: NetStateId,
    pub mobile: Option<MobileId>,
}

impl WorldMovementActor<'_> {
    fn live_mobile(&self) -> Option<MobileId> {
        let id = self.mobile?;
        self.w.mobile(id).filter(|m| !m.deleted).map(|_| id)
    }
}

impl MovementActor for WorldMovementActor<'_> {
    fn deleted(&self) -> bool {
        self.live_mobile().is_none()
    }

    fn is_staff(&self) -> bool {
        self.live_mobile()
            .and_then(|id| self.w.mobile(id))
            .is_some_and(|m| m.access_level > AccessLevel::Player)
    }

    fn compute_movement_speed(&self, dir: u8) -> i32 {
        let Some(id) = self.live_mobile() else {
            return 0;
        };
        // ComputeMovementSpeed(dir) = ComputeMovementSpeed(dir, true)
        crate::mobile::base::compute_movement_speed_base(
            self.w,
            id,
            Direction::from_bits_retain(dir),
            true,
        )
    }

    fn mounted(&self) -> bool {
        // shortcut: Mobile.Mount also looks on Layer.Mount; the cached mount item covers a mounted player
        self.live_mobile()
            .and_then(|id| self.w.mobile(id))
            .is_some_and(|m| m.mount_item.is_some())
    }

    fn do_move(&mut self, st: &mut MovementState, dir: u8) -> bool {
        let Some(id) = self.live_mobile() else {
            return false;
        };
        // Mobile.Move reads and writes this connection's sequence and banked time: lend the state back
        // to the world for the call.
        if let Some(d) = self.w.net_state_mut(self.ns) {
            std::mem::swap(&mut d.movement, st);
        }
        let ok = crate::mobile::base::move_dir(self.w, id, Direction::from_bits_retain(dir));
        if let Some(d) = self.w.net_state_mut(self.ns) {
            std::mem::swap(&mut d.movement, st);
        }
        ok
    }

    fn send_movement_rej(&mut self, seq: i32) {
        if let Some(id) = self.mobile {
            crate::packets::movement::send_movement_rej(self.w, self.ns, seq, id);
        }
    }

    fn send_client_version_request(&mut self) {
        self.w.send(self.ns, &[0xBD, 0x00, 0x03]);
    }

    fn random(&mut self, n: i32) -> i32 {
        utility::random(&mut self.w.rng, n)
    }

    fn describe(&self) -> String {
        match self.mobile.and_then(|id| self.w.mobile(id)) {
            // Mobile.ToString()
            Some(m) => format!(
                "0x{:X} \"{}\"",
                m.serial.value(),
                (m.ty.name)(self.w, self.mobile.unwrap()).unwrap_or_default()
            ),
            None => self
                .w
                .net_state(self.ns)
                .map(|d| d.description.clone())
                .unwrap_or_default(),
        }
    }

    fn speed_hack_detected(&mut self, rate: f32, verdict: DetectionVerdict, urgency: &str) {
        let Some(id) = self.mobile else { return };
        let mut listeners = std::mem::take(&mut self.w.resources.get_mut::<SpeedHackListeners>().0);
        for l in &mut listeners {
            l(self.w, self.ns, id, rate, verdict, urgency);
        }
        // Listeners added during the call stay after the existing ones
        let added = std::mem::take(&mut self.w.resources.get_mut::<SpeedHackListeners>().0);
        listeners.extend(added);
        self.w.resources.get_mut::<SpeedHackListeners>().0 = listeners;
    }
}

/// Runs `f` with the connection's movement state taken out of the world and an actor over the world.
fn with_movement<R>(
    w: &mut World,
    ns: NetStateId,
    f: impl FnOnce(
        &mut muo_net::movement::MovementThrottle,
        &mut MovementState,
        &mut WorldMovementActor<'_>,
    ) -> R,
) -> Option<R> {
    let now = w.tick_count;
    let d = w.net_state_mut(ns)?;
    let mobile = d.mobile;
    let mut st = std::mem::replace(&mut d.movement, MovementState::new(now));
    let mut throttle = std::mem::take(&mut w.movement_throttle);
    let mut actor = WorldMovementActor { w, ns, mobile };
    let r = f(&mut throttle, &mut st, &mut actor);
    w.movement_throttle = throttle;
    if let Some(d) = w.net_state_mut(ns) {
        d.movement = st;
    }
    Some(r)
}

/// `MovementThrottle.ValidateAndQueueMovement(ns, dir, seq)`, called by the MovementReq handler.
pub fn validate_and_queue_movement(w: &mut World, ns: NetStateId, dir: u8, seq: i32) {
    let now = w.tick_count;
    with_movement(w, ns, |th, st, actor| {
        th.validate_and_queue_movement(ns.0 as usize, st, actor, dir, seq, now);
    });
}

/// `MovementThrottle.ProcessAllQueues()`.
pub fn process_all_movement_queues(w: &mut World) {
    let now = w.tick_count;
    for id in w.movement_throttle.queued() {
        let ns = NetStateId(id as u32);
        if with_movement(w, ns, |th, st, actor| {
            th.process_movement_queue(id, st, actor, now);
        })
        .is_none()
        {
            w.movement_throttle.forget(id); // the connection went away
        }
    }
}

/// `NetState.ResetMovementState()`.
pub fn reset_movement_state(w: &mut World, ns: NetStateId) {
    let now = w.tick_count;
    if let Some(d) = w.net_state_mut(ns) {
        d.movement.reset(now);
    }
}

/// The world half of `NetState.Dispose`: the mobile and account let go of the connection (`m.NetState = null`
/// runs the `Mobile.NetState` setter while the state still exists).
pub fn dispose_net_state(w: &mut World, ns: NetStateId, transport: &NetState) {
    let Some(mid) = w.net_states.get(&ns).and_then(|d| d.mobile) else {
        return dispose_rest(w, ns, transport);
    };
    if w.mobile(mid).is_some_and(|m| m.net_state == Some(ns)) {
        crate::mobile::part1::props_b::set_net_state(w, mid, None);
    }
    dispose_rest(w, ns, transport);
}

fn dispose_rest(w: &mut World, ns: NetStateId, transport: &NetState) {
    let Some(d) = w.net_states.shift_remove(&ns) else {
        return;
    };
    w.movement_throttle.forget(ns.0 as usize);
    let _ = (d, transport); // the transport has logged "Disconnected."
}

/// Owns the transport and runs it as the loop's `NetState.Slice()`.
pub struct NetworkHooks {
    pub net: Network<World>,
}

impl NetworkHooks {
    pub fn new(net: Network<World>) -> Self {
        Self { net }
    }

    /// Hands the world's queued packets and disconnects to the transport in order, then refreshes the mirrors.
    pub fn pump(&mut self, w: &mut World) {
        for out in std::mem::take(&mut w.net_out) {
            match out {
                NetOut::Send(id, bytes) => {
                    if let Some(ns) = self.net.get_mut(id.0 as usize) {
                        ns.send(&bytes);
                    }
                }
                NetOut::Disconnect(id, reason) => {
                    if let Some(ns) = self.net.get_mut(id.0 as usize) {
                        ns.disconnect(&reason);
                    }
                }
                NetOut::UnblockPackets(id) => {
                    if let Some(ns) = self.net.get_mut(id.0 as usize) {
                        ns.block_all_packets = false;
                    }
                }
                NetOut::SetAccount(id, account) => {
                    if let Some(ns) = self.net.get_mut(id.0 as usize) {
                        ns.account = account;
                    }
                }
                NetOut::BanRetract(ip) => self.net.bans.retract(ip),
                NetOut::SetPacketLogging(id, on) => {
                    if let Some(ns) = self.net.get_mut(id.0 as usize) {
                        ns.set_packet_logging(on);
                    }
                }
            }
        }
        for (id, d) in w.net_states.iter_mut() {
            match self.net.get(id.0 as usize) {
                Some(ns) => {
                    d.cannot_send = ns.cannot_send_packets();
                    sync_caps(d, ns);
                }
                None => d.cannot_send = true,
            }
        }
    }

    fn handle_events(&mut self, w: &mut World) {
        for ev in self.net.take_events() {
            match ev {
                NetEvent::Connected(id) => {
                    // a packet handler may have made it already (see `ensure_net_state`)
                    if let Some(ns) = self.net.get(id) {
                        ensure_net_state(w, ns);
                    } else {
                        w.net_states
                            .entry(NetStateId(id as u32))
                            .or_insert_with(|| NetStateData::new(w.tick_count));
                    }
                }
                NetEvent::Disconnected(ns) => {
                    dispose_net_state(w, NetStateId(ns.id() as u32), &ns);
                }
            }
        }
    }

    /// One `NetState.Slice()`.
    pub fn slice(&mut self, w: &mut World) {
        // MovementThrottle.ProcessAllQueues runs before the receive pass
        process_all_movement_queues(w);
        self.pump(w);
        self.net.slice(w, w.tick_count);
        self.handle_events(w);
        self.pump(w);
        self.net.flush_all();
        // `NetState.GetNetworkStats()` for the admin gump, at most every 250 ms (it walks the connections).
        let snap = w.resources.get_mut::<NetStatsSnapshot>();
        if w.tick_count - snap.at >= 250 {
            snap.at = w.tick_count;
            snap.stats = self.net.get_network_stats();
        }
        if std::mem::take(&mut snap.dump_requested) {
            dump_net_states(w, &self.net);
        }
    }
}

/// The world's side of a connection, made if it does not exist yet. The transport accepts a connection and may
/// hand its first packets to a handler in the same `slice`, before the `Connected` event is processed, so a
/// handler that needs the mirror calls this first.
pub fn ensure_net_state(w: &mut World, ns: &NetState) -> NetStateId {
    let id = NetStateId(ns.id() as u32);
    if !w.net_states.contains_key(&id) {
        let mut d = NetStateData::new(w.tick_count);
        d.address = Some(ns.address);
        d.local_end_point = ns.local_end_point();
        d.description = ns.to_string();
        sync_caps(&mut d, ns);
        w.net_states.insert(id, d);
    }
    id
}

fn sync_caps(d: &mut NetStateData, ns: &NetState) {
    if d.caps.version.as_ref() != ns.version() {
        d.caps.version = ns.version().cloned();
    }
    d.caps.protocol_changes = ns.protocol_changes();
    d.caps.flags = ns.flags;
    if d.account != ns.account {
        d.account = ns.account.clone();
    }
    if d.assistant != ns.assistant {
        d.assistant = ns.assistant.clone();
    }
}

impl LoopHooks for NetworkHooks {
    fn network_slice(&mut self, w: &mut World) {
        self.slice(w);
    }

    fn is_idle(&mut self, w: &mut World) -> bool {
        w.net_out.is_empty() && self.net.is_idle()
    }

    fn wait_for_completion(&mut self, _w: &mut World, ms: i32) {
        self.net.wait_for_completion(ms.max(0) as u64);
    }
}
