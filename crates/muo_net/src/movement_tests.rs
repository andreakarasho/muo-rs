//! Movement throttle and latency tests, plus throttle flow tests.

use super::*;

fn throttle() -> MovementThrottle {
    MovementThrottle::default()
}

/// A movement state with the given step history.
fn state_with_history(records: &[(i16, u16)]) -> MovementState {
    let mut st = MovementState::new(0);
    if records.is_empty() {
        return st;
    }
    let mut h = vec![MovementRecord::default(); 20];
    for &(interval, target_speed) in records {
        h[st.history_index] = MovementRecord {
            interval,
            target_speed,
            queue_depth: 0,
            flags: 0,
        };
        st.history_index += 1;
        if st.history_index >= 20 {
            st.history_index = 0;
            st.history_full = true;
        }
    }
    st.history = Some(h);
    st
}

fn rate_of(records: &[(i16, u16)]) -> (f32, i32) {
    throttle().calculate_movement_rate(&state_with_history(records))
}

#[test]
fn rate_no_history_returns_one() {
    assert_eq!(rate_of(&[]), (1.0, 0));
}

#[test]
fn rate_insufficient_samples_returns_one() {
    // Fewer than 8 samples: early return with sample count 0
    assert_eq!(rate_of(&[(100, 100); 3]), (1.0, 0));
}

#[test]
fn rate_normal_fast_slow() {
    assert_eq!(rate_of(&[(100, 100); 10]), (1.0, 10));
    assert_eq!(rate_of(&[(50, 100); 10]), (2.0, 10));
    assert_eq!(rate_of(&[(200, 100); 10]), (0.5, 10));
}

#[test]
fn rate_mixed_speeds_returns_average() {
    let (rate, n) = rate_of(&[
        (80, 100),
        (100, 100),
        (90, 100),
        (100, 100),
        (80, 100),
        (90, 100),
        (100, 100),
        (80, 100),
        (90, 100),
        (90, 100),
    ]);
    assert!(rate > 1.10 && rate < 1.12, "rate {rate}");
    assert_eq!(n, 10);
}

fn burst_of(records: &[(i16, u16)]) -> (i32, i32) {
    throttle().detect_recent_burst(&state_with_history(records))
}

#[test]
fn burst_detection() {
    assert_eq!(burst_of(&[(100, 100); 10]).0, 0);

    let mut r = vec![(100, 100); 5];
    r.extend([(500, 100), (5, 100), (5, 100), (5, 100), (5, 100)]);
    assert_eq!(burst_of(&r), (4, 500));

    let mut r = vec![(100, 100); 8];
    r.extend([(200, 100), (10, 100)]);
    assert_eq!(burst_of(&r), (1, 200));
}

fn analyze(st: &mut MovementState) -> (DetectionVerdict, f32, i32, f32) {
    throttle().analyze_movement(st)
}

#[test]
fn analyze_normal_movement() {
    let mut st = state_with_history(&[(100, 100); 10]);
    let (v, rate, n, _) = analyze(&mut st);
    assert_eq!((v, rate, n), (DetectionVerdict::Normal, 1.0, 10));
}

#[test]
fn analyze_clear_speed_hack_is_definite() {
    let mut st = state_with_history(&[(77, 100); 10]);
    let (v, rate, ..) = analyze(&mut st);
    assert_eq!(v, DetectionVerdict::Definite);
    assert!(rate > 1.25, "rate {rate}");
}

#[test]
fn analyze_burst_after_large_gap_reduces_confidence() {
    let mut r = vec![(100, 100); 5];
    r.extend([(5, 100); 5]);
    let mut st = state_with_history(&r);
    st.last_gap_duration = 1500; // > max_chain_gap / 2
    let (_, _, _, confidence) = analyze(&mut st);
    assert!(confidence < 0.5, "confidence {confidence}");
    assert_eq!(st.last_gap_duration, 0);
}

#[test]
fn analyze_high_queue_is_definite() {
    let mut st = state_with_history(&[(95, 100); 10]);
    for _ in 0..5 {
        st.queue.push_back(QueuedMovement::default());
    }
    assert_eq!(analyze(&mut st).0, DetectionVerdict::Definite);
}

#[test]
fn analyze_unstable_connection_reduces_confidence() {
    let mut unstable = state_with_history(&[(90, 100); 10]);
    unstable.rtt_history = Some([50, 200, 80, 350, 100, 250, 90, 300]);
    unstable.rtt_sample_count = 8;
    unstable.rtt_variance = 15000;
    let c_unstable = analyze(&mut unstable).3;

    let mut stable = state_with_history(&[(90, 100); 10]);
    stable.rtt_history = Some([50, 52, 48, 51, 49, 50, 51, 50]);
    stable.rtt_sample_count = 8;
    stable.rtt_variance = 4;
    let c_stable = analyze(&mut stable).3;

    assert!(
        c_unstable < c_stable,
        "unstable {c_unstable} stable {c_stable}"
    );
}

fn rtt_state(history: [i64; 8], samples: usize, variance: i64) -> MovementState {
    let mut st = MovementState::new(0);
    st.rtt_history = Some(history);
    st.rtt_sample_count = samples;
    st.rtt_variance = variance;
    st
}

#[test]
fn stable_connection_rules() {
    assert!(rtt_state([50, 52, 48, 51, 49, 50, 51, 50], 8, 4).has_stable_connection());
    // High but consistent latency is not stable
    assert!(!rtt_state([500, 502, 498, 501, 499, 500, 501, 500], 8, 4).has_stable_connection());
    assert!(!rtt_state([50, 200, 30, 180, 40, 150, 60, 170], 8, 5000).has_stable_connection());
    // Needs at least 3 samples
    assert!(!rtt_state([50, 50, 0, 0, 0, 0, 0, 0], 2, 0).has_stable_connection());
}

#[test]
fn burst_after_gap_in_wrapped_full_history_forgives_and_clears_gap() {
    // 17 normal steps, then a 3-step burst ending at the wrap
    let mut st = MovementState::new(0);
    let mut h = vec![
        MovementRecord {
            interval: 100,
            target_speed: 100,
            queue_depth: 0,
            flags: 0
        };
        20
    ];
    for r in h.iter_mut().skip(17) {
        r.interval = 10;
    }
    st.history = Some(h);
    st.history_index = 0;
    st.history_full = true;
    st.last_gap_duration = 2500; // a lag spike longer than the chain gap preceded the burst

    let (verdict, rate, _, confidence) = analyze(&mut st);
    assert_eq!(st.last_gap_duration, 0);
    assert!(confidence < 0.5, "confidence {confidence} rate {rate}");
    assert_ne!(verdict, DetectionVerdict::Definite);
}

#[test]
fn rtt_probe_and_measurement() {
    let mut st = MovementState::new(1000);
    assert!(st.maybe_send_rtt_probe(1000, || 7));
    assert!(!st.maybe_send_rtt_probe(1001, || 0)); // pending
    assert_eq!(st.next_rtt_probe, 1000 + 5000 + 7);
    st.record_rtt_measurement(1040);
    assert_eq!(st.average_rtt(), 40);
    assert_eq!(st.last_rtt(), 40);
    // A response nobody asked for is ignored
    st.record_rtt_measurement(2000);
    assert_eq!(st.rtt_sample_count, 1);
    // Pending probe times out after 10 seconds
    st.next_rtt_probe = 0;
    assert!(st.maybe_send_rtt_probe(20_000, || 0));
    assert!(!st.maybe_send_rtt_probe(31_000, || 0));
    assert!(!st.rtt_probe_pending);
}

#[derive(Default)]
struct Actor {
    moves: Vec<u8>,
    rejects: Vec<i32>,
    fail_moves: bool,
    staff: bool,
}

impl MovementActor for Actor {
    fn deleted(&self) -> bool {
        false
    }
    fn is_staff(&self) -> bool {
        self.staff
    }
    fn compute_movement_speed(&self, _dir: u8) -> i32 {
        200
    }
    fn mounted(&self) -> bool {
        false
    }
    fn do_move(&mut self, _st: &mut MovementState, dir: u8) -> bool {
        if self.fail_moves {
            return false;
        }
        self.moves.push(dir);
        true
    }
    fn send_movement_rej(&mut self, seq: i32) {
        self.rejects.push(seq);
    }
    fn send_client_version_request(&mut self) {}
    fn random(&mut self, _n: i32) -> i32 {
        0
    }
    fn describe(&self) -> String {
        "test".into()
    }
}

#[test]
fn disabled_detection_executes_immediately_and_advances_sequence() {
    let mut t = throttle();
    let mut st = MovementState::new(0);
    let mut a = Actor::default();
    t.validate_and_queue_movement(1, &mut st, &mut a, 2, 0, 0);
    t.validate_and_queue_movement(1, &mut st, &mut a, 2, 1, 0);
    assert_eq!(a.moves, vec![2, 2]);
    assert_eq!(st.sequence, 2);

    // Sequence wraps 255 -> 1
    t.validate_and_queue_movement(1, &mut st, &mut a, 2, 255, 0);
    assert_eq!(st.sequence, 1);

    a.fail_moves = true;
    t.validate_and_queue_movement(1, &mut st, &mut a, 2, 1, 0);
    assert_eq!(a.rejects, vec![1]);
    assert_eq!(st.sequence, 0);
}

#[test]
fn early_packets_use_credit_then_queue_then_drain() {
    let mut t = MovementThrottle::new(MovementConfig {
        speedhack_detection: true,
        ..Default::default()
    });
    let mut st = MovementState::new(0);
    st.next_movement_time = 1000; // movement not allowed until t=1000
    let mut a = Actor::default();

    // 100ms early, within the 200ms credit: executes
    t.validate_and_queue_movement(7, &mut st, &mut a, 0, 0, 900);
    assert_eq!(a.moves.len(), 1);
    assert_eq!(st.credit, -100);

    // Another 150ms early exhausts credit (-250 < -200): queued
    t.validate_and_queue_movement(7, &mut st, &mut a, 1, 1, 850);
    assert_eq!(a.moves.len(), 1);
    assert_eq!(st.queue.len(), 1);
    assert_eq!(t.queued(), vec![7]);

    // Not yet due
    t.process_movement_queue(7, &mut st, &mut a, 900);
    assert_eq!(a.moves.len(), 1);
    // Due
    t.process_movement_queue(7, &mut st, &mut a, 1000);
    assert_eq!(a.moves.len(), 2);
    assert!(t.queued().is_empty());
    assert!(!st.has_queued_movements);
}

#[test]
fn hard_queue_limit_rejects_and_resets() {
    let mut t = MovementThrottle::new(MovementConfig {
        speedhack_detection: true,
        hard_queue_limit: 2,
        ..Default::default()
    });
    let mut st = MovementState::new(0);
    st.next_movement_time = 10_000;
    st.credit = -200;
    st.sequence = 5;
    let mut a = Actor::default();
    for seq in 5..8 {
        t.validate_and_queue_movement(1, &mut st, &mut a, 0, seq, 0);
    }
    assert_eq!(a.rejects, vec![7]);
    assert_eq!(st.sequence, 0);
    assert!(st.queue.is_empty());
}

#[test]
fn rtt_probe_draws_no_random_number_unless_it_sends() {
    let mut st = MovementState::new(1000);
    st.next_rtt_probe = 9000; // not due yet
    assert!(!st.maybe_send_rtt_probe(1000, || panic!("drew a random number")));
    st.rtt_probe_pending = true;
    assert!(!st.maybe_send_rtt_probe(1000, || panic!("drew a random number")));
}
