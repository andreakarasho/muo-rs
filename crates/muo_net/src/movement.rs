//! The movement throttle and each connection's movement state.
//!
//! The throttle's credit/queue/history/RTT arithmetic is here; everything that touches a `Mobile` goes
//! through [`MovementActor`], which `muo_server` implements over its world (move, speed, staff check, the
//! movement-reject packet, the 0xBD probe). Time is the wrapping tick count (`Core.TickCount`), passed in.

use std::collections::VecDeque;
use tracing::{debug, info, warn};

/// `Direction.Running`.
pub const DIRECTION_RUNNING: u8 = 0x80;

const RTT_PROBE_INTERVAL_NORMAL: i32 = 5000;
const RTT_PROBE_INTERVAL_SUSPICIOUS: i32 = 2000;
const RTT_PROBE_INTERVAL_DEFINITE: i32 = 1000;
const RTT_PROBE_JITTER: i32 = 500;
pub const RTT_HISTORY_SIZE: usize = 8;
const STABLE_VARIANCE_THRESHOLD: i64 = 2500;
const MAX_STABLE_LATENCY: i64 = 200;

// The client can have at most 5 unacknowledged movements pending, so with an unmodified client the
// server-side queue never exceeds 4.
const CLIENT_MAX_UNACKED_MOVEMENTS: usize = 5;
const MAX_QUEUE_WITH_UNMODIFIED_CLIENT: usize = CLIENT_MAX_UNACKED_MOVEMENTS - 1;

// Maximum expected packets per second (mounted running = 100ms = 10/sec, plus tolerance)
const MAX_EXPECTED_PACKET_RATE: i32 = 12;

/// `MovementRecordFlags`.
pub mod record_flags {
    pub const RUNNING: u8 = 1;
    pub const MOUNTED: u8 = 2;
    /// Cost was 0 (turn in place)
    pub const DIRECTION_CHANGE_ONLY: u8 = 4;
    /// Packet was queued, not executed immediately
    pub const WAS_QUEUED: u8 = 8;
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub struct QueuedMovement {
    pub direction: u8,
    pub sequence: i32,
}

/// Movement history record for rate calculation.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub struct MovementRecord {
    /// Time since previous packet (ms), capped at 32767
    pub interval: i16,
    /// Expected interval (100ms mounted running, etc.)
    pub target_speed: u16,
    /// Queue size when received
    pub queue_depth: u8,
    pub flags: u8,
}

/// Per-connection movement state.
#[derive(Clone, Debug)]
pub struct MovementState {
    /// `NetState.Sequence`
    pub sequence: i32,
    pub logging: bool,

    pub queue: VecDeque<QueuedMovement>,
    /// Credit buffer for timing jitter
    pub credit: i64,
    /// When next movement is allowed
    pub next_movement_time: i64,
    pub last_queue_depth_check: i64,
    pub has_queued_movements: bool,

    /// Circular buffer, lazily created by the throttle
    pub history: Option<Vec<MovementRecord>>,
    /// Next write position (also serves as count until full)
    pub history_index: usize,
    pub history_full: bool,
    pub last_movement_record_time: i64,
    pub has_movement_record: bool,

    pub consecutive_high_rate_seconds: i32,
    pub last_speed_hack_notification: i64,
    pub speed_hack_notified: bool,
    /// Duration of the last gap > max_chain_gap (for burst forgiveness)
    pub last_gap_duration: i32,

    pub movement_window_start: i64,
    pub movements_in_window: i32,
    pub peak_movement_rate: i32,

    pub rtt_probe_pending: bool,
    pub rtt_probe_time: i64,
    pub rtt_history: Option<[i64; RTT_HISTORY_SIZE]>,
    pub rtt_history_index: usize,
    pub rtt_sample_count: usize,
    pub rtt_variance: i64,
    pub next_rtt_probe: i64,
    pub rtt_probe_interval: i32,
}

impl MovementState {
    pub fn new(now: i64) -> Self {
        Self {
            sequence: 0,
            logging: false,
            queue: VecDeque::new(),
            credit: 0,
            next_movement_time: now,
            last_queue_depth_check: now,
            has_queued_movements: false,
            history: None,
            history_index: 0,
            history_full: false,
            last_movement_record_time: 0,
            has_movement_record: false,
            consecutive_high_rate_seconds: 0,
            last_speed_hack_notification: 0,
            speed_hack_notified: false,
            last_gap_duration: 0,
            movement_window_start: now,
            movements_in_window: 0,
            peak_movement_rate: 0,
            rtt_probe_pending: false,
            rtt_probe_time: 0,
            rtt_history: None,
            rtt_history_index: 0,
            rtt_sample_count: 0,
            rtt_variance: 0,
            next_rtt_probe: now,
            rtt_probe_interval: RTT_PROBE_INTERVAL_NORMAL,
        }
    }

    /// Resets movement state when the sequence needs to be cleared (paralysis, teleport, map change, etc.)
    pub fn reset(&mut self, now: i64) {
        self.queue.clear();
        self.sequence = 0;
        self.next_movement_time = now;
        self.credit = 0;
        self.has_queued_movements = false;

        // Next movement starts a new chain
        self.has_movement_record = false;
        self.history_index = 0;
        self.history_full = false;

        // Sustained detection loses context on teleport/map change
        self.consecutive_high_rate_seconds = 0;
        self.last_gap_duration = 0;
        self.rtt_probe_interval = RTT_PROBE_INTERVAL_NORMAL;

        self.movement_window_start = now;
        self.movements_in_window = 0;
    }

    /// Tracks movement packet rate; returns packets per second in the current window.
    pub fn track_movement_rate(&mut self, now: i64) -> i32 {
        if now - self.movement_window_start >= 1000 {
            if self.movements_in_window > self.peak_movement_rate {
                self.peak_movement_rate = self.movements_in_window;
            }
            self.movement_window_start = now;
            self.movements_in_window = 1;
            return 1;
        }

        self.movements_in_window += 1;
        self.movements_in_window
    }

    pub fn current_movement_rate(&self) -> i32 {
        self.movements_in_window
    }

    pub fn peak_movement_rate(&self) -> i32 {
        self.peak_movement_rate
    }

    /// Most recent RTT measurement, or 0 if none has been recorded.
    pub fn last_rtt(&self) -> i64 {
        match (&self.rtt_history, self.rtt_sample_count) {
            (Some(h), n) if n > 0 => {
                h[(self.rtt_history_index.wrapping_sub(1)) & (RTT_HISTORY_SIZE - 1)]
            }
            _ => 0,
        }
    }

    /// More suspicious = more frequent probes for better evidence.
    pub fn set_probe_frequency(&mut self, suspicion_level: i32) {
        self.rtt_probe_interval = match suspicion_level {
            3.. => RTT_PROBE_INTERVAL_DEFINITE,
            2 => RTT_PROBE_INTERVAL_SUSPICIOUS,
            _ => RTT_PROBE_INTERVAL_NORMAL,
        };
    }

    /// Decides whether a 0xBD probe goes out now; `jitter` makes `Utility.Random(500)` (0..500) for the
    /// next-probe delay and is only called when the probe goes out (no random number is drawn otherwise).
    /// The caller sends the probe when this returns true.
    pub fn maybe_send_rtt_probe(&mut self, now: i64, jitter: impl FnOnce() -> i32) -> bool {
        // Don't send if we're still waiting for a response
        if self.rtt_probe_pending {
            // Timeout after 10 seconds - connection is probably dead or very laggy
            if now - self.rtt_probe_time > 10000 {
                self.rtt_probe_pending = false;
            }
            return false;
        }

        if now - self.next_rtt_probe >= 0 {
            self.rtt_probe_pending = true;
            self.rtt_probe_time = now;
            self.next_rtt_probe = now
                + self.rtt_probe_interval as i64
                + jitter().clamp(0, RTT_PROBE_JITTER - 1) as i64;
            return true;
        }

        false
    }

    /// Records an RTT measurement when the ClientVersion response is received.
    pub fn record_rtt_measurement(&mut self, now: i64) {
        // Not expecting a response (client-initiated version send): ignore silently
        if !self.rtt_probe_pending {
            return;
        }

        let rtt = now - self.rtt_probe_time;
        self.rtt_probe_pending = false;

        // Sanity check - RTT should be positive and reasonable
        if rtt <= 0 || rtt > 10000 {
            return;
        }

        let history = self.rtt_history.get_or_insert([0; RTT_HISTORY_SIZE]);
        history[self.rtt_history_index & (RTT_HISTORY_SIZE - 1)] = rtt;
        self.rtt_history_index = self.rtt_history_index.wrapping_add(1);

        // Saturates at buffer size
        if self.rtt_sample_count < RTT_HISTORY_SIZE {
            self.rtt_sample_count += 1;
        }

        self.update_rtt_variance();
    }

    fn rtt_samples(&self) -> impl Iterator<Item = i64> + '_ {
        self.rtt_history
            .iter()
            .flat_map(|h| h.iter().copied())
            .filter(|&s| s > 0)
    }

    fn update_rtt_variance(&mut self) {
        if self.rtt_history.is_none() {
            self.rtt_variance = 0;
            return;
        }

        let (sum, sum_sq, count) = self
            .rtt_samples()
            .fold((0i64, 0i64, 0i64), |(a, b, c), s| (a + s, b + s * s, c + 1));
        if count < 2 {
            self.rtt_variance = 0;
            return;
        }

        let mean = sum / count;
        // Integer division rounding can produce negative variance
        self.rtt_variance = (sum_sq / count - mean * mean).max(0);
    }

    /// Average RTT from recent measurements.
    pub fn average_rtt(&self) -> i64 {
        let (sum, count) = self
            .rtt_samples()
            .fold((0i64, 0i64), |(a, c), s| (a + s, c + 1));
        if count > 0 { sum / count } else { 0 }
    }

    /// Stable, low-variance, low-latency; needs at least 3 samples.
    pub fn has_stable_connection(&self) -> bool {
        let avg = self.average_rtt();
        self.rtt_sample_count >= 3
            && self.rtt_variance < STABLE_VARIANCE_THRESHOLD
            && avg > 0
            && avg < MAX_STABLE_LATENCY
    }
}

/// What the throttle needs from the player's mobile and connection; implemented by `muo_server`.
pub trait MovementActor {
    /// The mobile is gone (null or deleted).
    fn deleted(&self) -> bool;
    /// `AccessLevel > Player`
    fn is_staff(&self) -> bool;
    /// `Mobile.ComputeMovementSpeed(dir)`
    fn compute_movement_speed(&self, dir: u8) -> i32;
    fn mounted(&self) -> bool;
    /// `Mobile.Move(dir)`: also updates `next_movement_time` and sends the ack; false if blocked.
    /// `st` is this connection's movement state, lent back so the move can read the sequence and bank time.
    fn do_move(&mut self, st: &mut MovementState, dir: u8) -> bool;
    /// `NetState.SendMovementRej(seq, mobile)`
    fn send_movement_rej(&mut self, seq: i32);
    /// `NetState.SendClientVersionRequest()` (`[0xBD, 0x00, 0x03]`)
    fn send_client_version_request(&mut self);
    /// `Utility.Random(n)`
    fn random(&mut self, n: i32) -> i32;
    /// Text for log lines: character, account, location, map and IP.
    fn describe(&self) -> String;
    /// `MovementThrottle.OnSpeedHackDetected`
    fn speed_hack_detected(&mut self, _rate: f32, _verdict: DetectionVerdict, _urgency: &str) {}
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum DetectionVerdict {
    Normal,
    /// Slight anomaly, could be network
    Possible,
    /// Multiple signals point to cheating
    Likely,
    /// Clear speed hacking
    Definite,
}

/// Snapshot of a player's movement throttle state for diagnostic display.
#[derive(Copy, Clone, Debug)]
pub struct MovementStats {
    pub rate: f32,
    pub sample_count: i32,
    pub verdict: DetectionVerdict,
    pub confidence: f32,
    pub average_rtt: i64,
    pub last_rtt: i64,
    pub rtt_variance: i64,
    pub stable_connection: bool,
    pub rtt_sample_count: usize,
    pub queue_depth: usize,
    pub movement_credit: i64,
    pub current_packet_rate: i32,
    pub peak_packet_rate: i32,
    pub burst_size: i32,
    pub preceding_gap: i32,
    pub sustained_seconds: i32,
    pub debug_logging: bool,
}

/// `movementThrottle.*` settings. `speedhack_detection` is `ServerFeatureFlags.SpeedhackDetection`, which
/// is false until the feature-flag manager says otherwise.
#[derive(Clone, Debug)]
pub struct MovementConfig {
    pub max_credit: i32,
    pub max_rtt_bonus: i32,
    pub hard_queue_limit: usize,
    pub movement_history_size: usize,
    pub min_samples_for_rate: i32,
    pub max_chain_gap: i32,
    pub suspicious_rate_threshold: f32,
    pub definite_rate_threshold: f32,
    pub speed_hack_notification_cooldown: i64,
    pub speedhack_detection: bool,
}

impl Default for MovementConfig {
    fn default() -> Self {
        Self {
            max_credit: 200,
            max_rtt_bonus: 150,
            hard_queue_limit: 10,
            movement_history_size: 20,
            min_samples_for_rate: 8,
            max_chain_gap: 2000,
            suspicious_rate_threshold: 1.05,
            definite_rate_threshold: 1.10,
            speed_hack_notification_cooldown: 300_000,
            speedhack_detection: false,
        }
    }
}

/// The throttle: configuration plus the set of connections with queued movements (`connection ids`).
#[derive(Default)]
pub struct MovementThrottle {
    pub config: MovementConfig,
    queued: Vec<usize>,
}

fn next_sequence(seq: i32) -> i32 {
    let n = seq + 1;
    if n == 256 { 1 } else { n }
}

impl MovementThrottle {
    pub fn new(config: MovementConfig) -> Self {
        Self {
            config,
            queued: Vec::new(),
        }
    }

    /// Dynamic credit: high-latency players get more tolerance; stable low-latency gets tighter security.
    fn dynamic_credit(&self, st: &MovementState) -> i64 {
        let avg_rtt = st.average_rtt();
        let max_credit = self.config.max_credit as i64;

        // No RTT data yet, or a stable low-latency connection: base credit
        if avg_rtt <= 0 || (st.has_stable_connection() && avg_rtt < 50) {
            return max_credit;
        }

        max_credit + (avg_rtt / 2).min(self.config.max_rtt_bonus as i64)
    }

    /// Called from the MovementReq handler: validates timing and either executes the movement or queues it.
    /// `id` is the connection id used to find the queue again in [`MovementThrottle::queued`].
    pub fn validate_and_queue_movement(
        &mut self,
        id: usize,
        st: &mut MovementState,
        actor: &mut dyn MovementActor,
        dir: u8,
        seq: i32,
        now: i64,
    ) {
        if actor.deleted() {
            return;
        }

        // Staff bypass all throttling; also bypass when speedhack detection is disabled
        if actor.is_staff() || !self.config.speedhack_detection {
            self.execute_movement(st, actor, dir, seq, now);
            return;
        }

        // Sequence was reset by paralysis, teleport, etc.
        if st.sequence == 0 && seq != 0 {
            Self::reject_and_reset(st, actor, seq, now);
            return;
        }

        st.track_movement_rate(now);

        // RTT probe - only when actively moving
        if st.maybe_send_rtt_probe(now, || actor.random(RTT_PROBE_JITTER)) {
            if st.logging {
                debug!("[RTT-Probe] Sending probe at TickCount={now}");
            }
            actor.send_client_version_request();
        }

        // This has FULL CONTEXT (mounted, running, direction change)
        let cost = actor.compute_movement_speed(dir);

        self.record_movement(st, now, cost, dir, actor);

        // Periodic rate-based detection (every 2 seconds, regardless of queue depth)
        if now - st.last_queue_depth_check >= 2000 {
            st.last_queue_depth_check = now;
            let verdict = self.check_and_notify_staff(st, actor, now);
            st.set_probe_frequency(verdict as i32);
        }

        // positive = on-time/late, negative = early
        let delta = now - st.next_movement_time;

        // Already-queued movements: add to queue to maintain order
        if st.has_queued_movements {
            self.queue_movement(id, st, actor, dir, seq, now);
            return;
        }

        let dynamic_credit = self.dynamic_credit(st);

        if delta < 0 {
            let early_amount = -delta;

            // Credit can go negative up to -dynamic_credit (debt limit)
            if st.credit - early_amount >= -dynamic_credit {
                st.credit -= early_amount;
                if st.logging {
                    debug!(
                        "[Credit] {}: delta={delta}ms early={early_amount}ms credit={}->{}/{dynamic_credit} action=execute",
                        actor.describe(),
                        st.credit + early_amount,
                        st.credit
                    );
                }
                self.execute_movement(st, actor, dir, seq, now);
                return;
            }

            if st.logging {
                debug!(
                    "[Credit] {}: delta={delta}ms early={early_amount}ms credit={}/{dynamic_credit} EXHAUSTED -> queue",
                    actor.describe(),
                    st.credit
                );
            }
            self.queue_movement(id, st, actor, dir, seq, now);
            return;
        }

        // On-time or late - rebuild credit (capped at dynamic max)
        if delta > 0 {
            let prev = st.credit;
            st.credit = (st.credit + delta).min(dynamic_credit);
            if st.logging && st.credit != prev {
                debug!(
                    "[Credit] {}: delta=+{delta}ms credit={prev}->{}/{dynamic_credit} action=execute",
                    actor.describe(),
                    st.credit
                );
            }
        }

        self.execute_movement(st, actor, dir, seq, now);
    }

    fn execute_movement(
        &self,
        st: &mut MovementState,
        actor: &mut dyn MovementActor,
        dir: u8,
        seq: i32,
        now: i64,
    ) {
        if !actor.do_move(st, dir) {
            if st.logging {
                debug!(
                    "[Execute] {}: Move FAILED dir={dir} seq={seq} -> reject+reset",
                    actor.describe()
                );
            }
            // Blocked, paralyzed, frozen, etc.
            Self::reject_and_reset(st, actor, seq, now);
            return;
        }

        if st.logging {
            debug!(
                "[Execute] {}: Move OK dir={dir} seq={seq} nextMove={}ms",
                actor.describe(),
                st.next_movement_time - now
            );
        }

        // Mobile.Move() already updated next_movement_time and sent the ack
        st.sequence = next_sequence(seq);
    }

    fn queue_movement(
        &mut self,
        id: usize,
        st: &mut MovementState,
        actor: &mut dyn MovementActor,
        dir: u8,
        seq: i32,
        now: i64,
    ) {
        if st.queue.len() >= self.config.hard_queue_limit {
            info!(
                "Movement queue overflow: {} | Queue reached hard limit: {}",
                actor.describe(),
                self.config.hard_queue_limit
            );
            Self::reject_and_reset(st, actor, seq, now);
            return;
        }

        st.queue.push_back(QueuedMovement {
            direction: dir,
            sequence: seq,
        });
        st.has_queued_movements = true;
        if !self.queued.contains(&id) {
            self.queued.push(id);
        }

        if st.logging {
            debug!(
                "[Queue] {}: enqueued dir={dir} seq={seq} (depth={})",
                actor.describe(),
                st.queue.len()
            );
        }
    }

    fn reject_and_reset(st: &mut MovementState, actor: &mut dyn MovementActor, seq: i32, now: i64) {
        actor.send_movement_rej(seq);
        st.reset(now);
    }

    /// Connection ids that have queued movements; call [`MovementThrottle::process_movement_queue`] for
    /// each once per slice (`ProcessAllQueues`).
    pub fn queued(&self) -> Vec<usize> {
        self.queued.clone()
    }

    /// A connection went away.
    pub fn forget(&mut self, id: usize) {
        self.queued.retain(|&q| q != id);
    }

    /// Processes the movement queue for one connection and drops it from the set once it is empty.
    pub fn process_movement_queue(
        &mut self,
        id: usize,
        st: &mut MovementState,
        actor: &mut dyn MovementActor,
        now: i64,
    ) {
        self.process_queue_inner(st, actor, now);
        if !st.has_queued_movements {
            self.forget(id);
        }
    }

    fn process_queue_inner(&self, st: &mut MovementState, actor: &mut dyn MovementActor, now: i64) {
        if actor.deleted() {
            Self::clear_queue(st);
            return;
        }

        // Staff don't queue
        if actor.is_staff() {
            Self::drain_queue_immediately(st, actor, now);
            return;
        }

        while !st.queue.is_empty() {
            // Not yet - leave remaining items in queue for next slice
            if now - st.next_movement_time < 0 {
                break;
            }

            let Some(movement) = st.queue.pop_front() else {
                break;
            };
            let remaining = st.queue.len();

            // Sequence was reset (paralysis, teleport, etc.)
            if st.sequence == 0 && movement.sequence != 0 {
                Self::reject_and_reset(st, actor, movement.sequence, now);
                return;
            }

            if !actor.do_move(st, movement.direction) {
                if st.logging {
                    debug!(
                        "[Queue] {}: dequeued FAILED dir={} (remaining={remaining})",
                        actor.describe(),
                        movement.direction
                    );
                }
                Self::reject_and_reset(st, actor, movement.sequence, now);
                return;
            }

            if st.logging {
                let waited = now - st.next_movement_time;
                debug!(
                    "[Queue] {}: dequeued OK dir={} (remaining={remaining}, waited={}ms)",
                    actor.describe(),
                    movement.direction,
                    waited.max(0)
                );
            }

            st.sequence = next_sequence(movement.sequence);
        }

        st.has_queued_movements = !st.queue.is_empty();
    }

    fn drain_queue_immediately(st: &mut MovementState, actor: &mut dyn MovementActor, now: i64) {
        while let Some(movement) = st.queue.pop_front() {
            if !actor.do_move(st, movement.direction) {
                Self::reject_and_reset(st, actor, movement.sequence, now);
                return;
            }
            st.sequence = next_sequence(movement.sequence);
        }

        Self::clear_queue(st);
    }

    fn clear_queue(st: &mut MovementState) {
        st.queue.clear();
        st.has_queued_movements = false;
    }

    /// Resets movement timing for all connected players after a world save, preventing burst rejections.
    /// Pass the state of every connection whose mobile is alive.
    pub fn reset_all_movement_timing<'a>(
        &self,
        states: impl IntoIterator<Item = &'a mut MovementState>,
        now: i64,
    ) {
        for st in states {
            st.next_movement_time = now;
            st.credit = self.config.max_credit as i64; // Give full credit after save
        }
    }

    /// Records a movement in the history buffer for rate analysis. Chain-breaking movements (first in a
    /// sequence, long gaps) are skipped.
    fn record_movement(
        &self,
        st: &mut MovementState,
        now: i64,
        cost: i32,
        dir: u8,
        actor: &dyn MovementActor,
    ) {
        let cfg = &self.config;
        // -1 indicates first movement (no previous time)
        let interval = if st.has_movement_record {
            (now - st.last_movement_record_time) as i32
        } else {
            -1
        };

        // Skip recording for chain-breaking movements. Still update the timestamp for the next movement.
        if interval <= 0 || interval > cfg.max_chain_gap {
            st.last_movement_record_time = now;
            st.has_movement_record = true;

            // Use RTT to tell "stopped moving" from "lagged": a stable low-latency connection with a gap
            // far over RTT stopped, so reset history; otherwise it might be lag, so preserve it.
            if interval > cfg.max_chain_gap {
                let avg_rtt = st.average_rtt();
                let should_reset = st.has_stable_connection()
                    && avg_rtt > 0
                    && avg_rtt < 200
                    && avg_rtt < (interval / 4) as i64;

                if should_reset {
                    st.history_index = 0;
                    st.history_full = false;
                }

                // A large gap followed by a burst of packets = likely lag recovery, not speed hack
                st.last_gap_duration = interval;

                if st.logging {
                    let action = if should_reset {
                        "history reset"
                    } else {
                        "history preserved (possible lag)"
                    };
                    debug!(
                        "[Movement] {}: SKIP recording (gap {interval}ms > {}ms, RTT={avg_rtt}ms stable={} -> {action})",
                        actor.describe(),
                        cfg.max_chain_gap,
                        st.has_stable_connection()
                    );
                }
            } else if st.logging {
                debug!(
                    "[Movement] {}: SKIP recording (first in chain)",
                    actor.describe()
                );
            }

            return;
        }

        let size = cfg.movement_history_size.max(1);
        let history = st
            .history
            .get_or_insert_with(|| vec![MovementRecord::default(); size]);
        if history.len() < size {
            history.resize(size, MovementRecord::default());
        }

        // Direction-only changes (cost = 0) don't contribute to rate calculation and would make the next
        // real move's interval artificially short, so they are not recorded.
        if cost == 0 {
            if st.logging {
                debug!(
                    "[Movement] {}: SKIP direction-only change (preserves interval measurement)",
                    actor.describe()
                );
            }
            return;
        }

        let mut flags = 0u8;
        if dir & DIRECTION_RUNNING != 0 {
            flags |= record_flags::RUNNING;
        }
        if actor.mounted() {
            flags |= record_flags::MOUNTED;
        }
        if st.has_queued_movements {
            flags |= record_flags::WAS_QUEUED;
        }

        let queue_depth = st.queue.len().min(255) as u8;
        let index = st.history_index.min(size - 1);
        // interval is validated: > 0 and <= max_chain_gap
        if let Some(history) = st.history.as_mut() {
            history[index] = MovementRecord {
                interval: interval.clamp(0, i16::MAX as i32) as i16,
                target_speed: cost.clamp(0, u16::MAX as i32) as u16,
                queue_depth,
                flags,
            };
        }

        st.history_index += 1;
        if st.history_index >= size {
            st.history_index = 0;
            st.history_full = true;
        }

        st.last_movement_record_time = now;
        st.has_movement_record = true;

        if st.logging {
            let count = if st.history_full {
                size
            } else {
                st.history_index
            };
            debug!(
                "[Movement] {}: interval={interval}ms target={cost}ms queue={queue_depth} flags={flags} history={count}/{size} RTT={}ms",
                actor.describe(),
                st.average_rtt()
            );
        }
    }

    fn history_count(&self, st: &MovementState) -> usize {
        if st.history_full {
            self.config.movement_history_size
        } else {
            st.history_index
        }
    }

    /// Movement rate ratio: target time / actual time. 1.0 is normal, above 1.0 is faster than allowed.
    /// Returns (rate, sample_count).
    pub fn calculate_movement_rate(&self, st: &MovementState) -> (f32, i32) {
        let Some(history) = st.history.as_ref() else {
            return (1.0, 0);
        };

        let size = self.config.movement_history_size;
        let history_count = self.history_count(st);
        if (history_count as i32) < self.config.min_samples_for_rate {
            return (1.0, 0);
        }

        let mut total_target = 0i64;
        let mut total_actual = 0i64;
        let mut count = 0;

        // Walk backwards through history from most recent. Every entry is valid (interval > 0 and
        // <= max_chain_gap) because of the filtering in record_movement and the history reset on chain breaks.
        for i in 0..history_count {
            let mut idx = st.history_index as isize - 1 - i as isize;
            if idx < 0 {
                idx += size as isize;
            }

            let Some(record) = history.get(idx as usize) else {
                continue;
            };

            // Direction-only changes don't contribute to movement rate
            if record.flags & record_flags::DIRECTION_CHANGE_ONLY != 0 {
                continue;
            }

            total_target += record.target_speed as i64;
            total_actual += record.interval as i64;
            count += 1;
        }

        if count < self.config.min_samples_for_rate || total_actual <= 0 {
            return (1.0, count);
        }

        (total_target as f32 / total_actual as f32, count)
    }

    /// Detects whether recent movements arrived as a burst (several packets with near-zero intervals).
    /// Returns (burst size, preceding gap in ms).
    pub fn detect_recent_burst(&self, st: &MovementState) -> (i32, i32) {
        let Some(history) = st.history.as_ref() else {
            return (0, 0);
        };

        let size = self.config.movement_history_size;
        let history_count = self.history_count(st);
        if history_count < 2 {
            return (0, 0);
        }

        // Packets within 15ms are "simultaneous"
        const BURST_THRESHOLD: i16 = 15;
        let mut burst_size = 0;
        let mut preceding_gap = 0;

        for i in 0..history_count {
            let mut idx = st.history_index as isize - 1 - i as isize;
            if idx < 0 {
                idx += size as isize;
            }

            let Some(record) = history.get(idx as usize) else {
                break;
            };

            // All intervals are > 0 due to the filtering in record_movement
            if record.interval <= BURST_THRESHOLD {
                burst_size += 1;
            } else {
                preceding_gap = record.interval as i32;
                break;
            }
        }

        (burst_size, preceding_gap)
    }

    /// Analyzes movement patterns. Returns (verdict, rate, sample_count, confidence 0..1).
    pub fn analyze_movement(&self, st: &mut MovementState) -> (DetectionVerdict, f32, i32, f32) {
        let cfg = &self.config;
        let (rate, sample_count) = self.calculate_movement_rate(st);
        let mut confidence = 0f32;

        if sample_count < cfg.min_samples_for_rate {
            return (DetectionVerdict::Normal, rate, sample_count, confidence);
        }

        let average_rtt = st.average_rtt();
        let stable = st.has_stable_connection();

        if st.logging {
            debug!(
                "[MovementAnalysis] Rate={rate:.3}, Samples={sample_count}, RTT={average_rtt}ms"
            );
        }

        // Signal 1: movement rate (primary)
        if rate > 1.15 {
            confidence += 0.5;
        } else if rate > cfg.definite_rate_threshold {
            confidence += 0.35;
        } else if rate > cfg.suspicious_rate_threshold {
            confidence += 0.2;
        } else if rate > 1.02 {
            confidence += 0.1;
        }

        // Signal 2: packet rate (secondary)
        let packet_rate = st.current_movement_rate();
        if packet_rate > 15 {
            confidence += 0.15;
        } else if packet_rate > MAX_EXPECTED_PACKET_RATE {
            confidence += 0.1;
        }

        // Signal 2b: queue depth. The client limits unacked movements to 5, so the queue maxes at 4;
        // above that is a modified client.
        let queue_depth = st.queue.len();
        if queue_depth > MAX_QUEUE_WITH_UNMODIFIED_CLIENT {
            confidence += 0.5;
        } else if queue_depth >= MAX_QUEUE_WITH_UNMODIFIED_CLIENT {
            confidence += 0.25;
        } else if queue_depth >= 2 {
            confidence += 0.1;
        }

        // Signal 3: RTT correlation (modifier)
        if stable && average_rtt < 100 {
            // Stable, low-latency connection - problems are more suspicious
            if rate > 1.02 {
                confidence += 0.2;
            }
        } else if st.rtt_variance > 10000 || average_rtt > 300 {
            // Unstable connection - reduce confidence
            confidence *= 0.6;
        }

        // Signal 4: burst pattern
        let (burst_size, _) = self.detect_recent_burst(st);

        if burst_size >= 3 && st.last_gap_duration > cfg.max_chain_gap / 2 {
            // Recent large gap + burst = lag recovery, strong forgiveness
            confidence *= 0.3;
            st.last_gap_duration = 0;
        } else if burst_size >= 3 && rate < cfg.suspicious_rate_threshold {
            // Burst pattern with normal rate = likely legitimate lag
            confidence *= 0.5;
        }

        // Signal 5: sample count modifier
        if sample_count < 10 {
            confidence *= 0.7;
        } else if sample_count >= 20 {
            confidence = (confidence * 1.1).min(1.0);
        }

        let verdict = if (rate > cfg.definite_rate_threshold && confidence > 0.6)
            || rate > 1.20
            // Impossible with an unmodified client, so definite regardless of persistence
            || queue_depth > MAX_QUEUE_WITH_UNMODIFIED_CLIENT
        {
            DetectionVerdict::Definite
        } else if
        // Queue-based detection needs a sustained high queue, not a momentary spike; at the client limit
        // on a stable low-latency link it is Likely (subsumed by the `>= 3` test below, kept to state the
        // order of checks)
        (queue_depth >= 3 && stable)
            || (rate > cfg.suspicious_rate_threshold && confidence > 0.4 && stable)
        {
            DetectionVerdict::Likely
        } else if (rate > 1.03 && confidence > 0.2)
            || packet_rate > 14
            || (queue_depth >= 2 && stable)
        {
            DetectionVerdict::Possible
        } else {
            DetectionVerdict::Normal
        };

        (verdict, rate, sample_count, confidence)
    }

    /// Checks whether staff should be notified about suspicious movement; called periodically. Returns the
    /// verdict for probe frequency adjustment.
    pub fn check_and_notify_staff(
        &self,
        st: &mut MovementState,
        actor: &mut dyn MovementActor,
        now: i64,
    ) -> DetectionVerdict {
        let (verdict, rate, sample_count, confidence) = self.analyze_movement(st);

        if st.logging {
            let (burst, _) = self.detect_recent_burst(st);
            debug!(
                "[RateCheck] {}: rate={rate:.3} samples={sample_count} verdict={verdict:?} confidence={:.0}% queue={} burst={burst} sustained={}s",
                actor.describe(),
                confidence * 100.0,
                st.queue.len(),
                st.consecutive_high_rate_seconds
            );
        }

        // Called every ~2 seconds
        if verdict >= DetectionVerdict::Likely {
            st.consecutive_high_rate_seconds += 2;
        } else {
            st.consecutive_high_rate_seconds = (st.consecutive_high_rate_seconds - 1).max(0);
        }

        let urgency = if verdict == DetectionVerdict::Definite {
            Some("HIGH")
        } else if verdict == DetectionVerdict::Likely && st.consecutive_high_rate_seconds >= 10 {
            Some("MEDIUM")
        } else if verdict == DetectionVerdict::Possible && st.consecutive_high_rate_seconds >= 30 {
            Some("LOW")
        } else {
            None
        };

        if let Some(urgency) = urgency {
            self.notify_staff(
                st,
                actor,
                now,
                rate,
                sample_count,
                confidence,
                verdict,
                urgency,
            );
        }

        verdict
    }

    #[allow(clippy::too_many_arguments)]
    fn notify_staff(
        &self,
        st: &mut MovementState,
        actor: &mut dyn MovementActor,
        now: i64,
        rate: f32,
        sample_count: i32,
        confidence: f32,
        verdict: DetectionVerdict,
        urgency: &str,
    ) {
        // Rate-limit notifications per player
        if st.speed_hack_notified
            && now - st.last_speed_hack_notification < self.config.speed_hack_notification_cooldown
        {
            return;
        }

        st.speed_hack_notified = true;
        st.last_speed_hack_notification = now;

        warn!(
            "[{urgency}] Speed hack detected: {} | Rate: {rate:.2} ({sample_count} samples) | Verdict: {verdict:?} | Confidence: {:.0}% | PacketRate: {}/s (peak: {}/s) | RTT: {}ms (stable: {}) | Sustained: {}s | Queue: {}",
            actor.describe(),
            confidence * 100.0,
            st.current_movement_rate(),
            st.peak_movement_rate(),
            st.average_rtt(),
            st.has_stable_connection(),
            st.consecutive_high_rate_seconds,
            st.queue.len()
        );

        // Server operators hook this to broadcast to staff
        actor.speed_hack_detected(rate, verdict, urgency);
    }

    /// Snapshot of the current movement throttle state for a player.
    pub fn movement_stats(&self, st: &mut MovementState) -> MovementStats {
        let (verdict, rate, sample_count, confidence) = self.analyze_movement(st);
        let (burst_size, preceding_gap) = self.detect_recent_burst(st);

        MovementStats {
            rate,
            sample_count,
            verdict,
            confidence,
            average_rtt: st.average_rtt(),
            last_rtt: st.last_rtt(),
            rtt_variance: st.rtt_variance,
            stable_connection: st.has_stable_connection(),
            rtt_sample_count: st.rtt_sample_count,
            queue_depth: st.queue.len(),
            movement_credit: st.credit,
            current_packet_rate: st.current_movement_rate(),
            peak_packet_rate: st.peak_movement_rate(),
            burst_size,
            preceding_gap,
            sustained_seconds: st.consecutive_high_rate_seconds,
            debug_logging: st.logging,
        }
    }
}

#[cfg(test)]
#[path = "movement_tests.rs"]
mod tests;
