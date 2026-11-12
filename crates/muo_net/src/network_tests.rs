//! Loopback tests of the transport (127.0.0.1, port 0): disconnects, receive framing guards, the accept
//! path, ban reporting, and encryption flows.

use super::*;
use crate::bans::BanReporter;
use crate::connection_filters::ConnectionFilter;
use crate::encryption::{ClientEncryption, GameEncryption};
use crate::net_state::{Attachment, ProtocolState};
use std::cell::RefCell;
use std::io::{Read, Write};
use std::rc::Rc;
use std::time::Instant;

#[derive(Default)]
struct Ctx {
    frames: Vec<(u8, Vec<u8>)>,
}

fn on_login(c: &mut Ctx, _ns: &mut NetState, body: &[u8]) {
    c.frames.push((0x91, body.to_vec()));
}
fn on_var(c: &mut Ctx, _ns: &mut NetState, body: &[u8]) {
    c.frames.push((0x42, body.to_vec()));
}
fn on_ingame(c: &mut Ctx, _ns: &mut NetState, body: &[u8]) {
    c.frames.push((0x43, body.to_vec()));
}

struct Harness {
    net: Network<Ctx>,
    ctx: Ctx,
    start: Instant,
    addr: SocketAddr,
}

impl Harness {
    fn new() -> Self {
        crate::logs::set_directory(std::env::temp_dir());
        let config = NetworkConfig {
            listeners: vec!["127.0.0.1:0".parse().unwrap()],
            ..Default::default()
        };
        let mut net = Network::new(config, 0).unwrap();
        net.packets.register(0x91, 65, false, on_login);
        net.packets.register(0x42, 0, false, on_var);
        net.packets.register(0x43, 3, true, on_ingame);
        let addr = net.start()[0];
        Self {
            net,
            ctx: Ctx::default(),
            start: Instant::now(),
            addr,
        }
    }

    fn tick(&self) -> i64 {
        self.start.elapsed().as_millis() as i64
    }

    fn slice(&mut self) {
        let now = self.tick();
        self.net.slice(&mut self.ctx, now);
    }

    fn pump(&mut self, mut done: impl FnMut(&Network<Ctx>, &Ctx) -> bool) {
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            self.slice();
            if done(&self.net, &self.ctx) {
                return;
            }
            assert!(Instant::now() < deadline, "timed out");
            std::thread::sleep(Duration::from_millis(1));
        }
    }

    fn spin(&mut self, slices: usize) {
        for _ in 0..slices {
            self.slice();
            std::thread::sleep(Duration::from_millis(2));
        }
    }

    /// Connects a client and waits for the server to accept it.
    fn connect(&mut self) -> (std::net::TcpStream, usize) {
        let client = std::net::TcpStream::connect(self.addr).unwrap();
        client.set_nodelay(true).unwrap();
        client
            .set_read_timeout(Some(Duration::from_millis(20)))
            .unwrap();
        let before = self.net.instance_count();
        self.pump(|n, _| n.instance_count() > before);
        let id = self.net.instances().last().unwrap().id();
        (client, id)
    }
}

fn read_all(client: &mut std::net::TcpStream, n: usize) -> Vec<u8> {
    let mut out = Vec::new();
    let deadline = Instant::now() + Duration::from_secs(5);
    let mut buf = [0u8; 8192];
    while out.len() < n {
        assert!(
            Instant::now() < deadline,
            "timed out reading, got {}",
            out.len()
        );
        match client.read(&mut buf) {
            Ok(0) => break,
            Ok(k) => out.extend_from_slice(&buf[..k]),
            Err(_) => {}
        }
    }
    out
}

fn game_login_packet() -> [u8; 65] {
    let mut p = [0u8; 65];
    p[0] = 0x91;
    p[1..5].copy_from_slice(&0x1234_5678u32.to_be_bytes());
    p[5..9].copy_from_slice(b"user");
    p[35..39].copy_from_slice(b"pass");
    p
}

#[test]
fn coerce_power_of_two_setting_cases() {
    let min = 64 * 1024;
    for (configured, expected) in [
        (3 * 1024 * 1024, 4 * 1024 * 1024),
        (512 * 1024 * 1024, 256 * 1024 * 1024),
        (1024, 64 * 1024),
        (2 * 1024 * 1024, 2 * 1024 * 1024),
    ] {
        assert_eq!(
            coerce_power_of_two_setting("network.sendBufferMaxSize", configured, min),
            expected
        );
    }
}

#[test]
fn seed_then_unencrypted_game_login_reaches_the_handler() {
    let mut h = Harness::new();
    let (mut client, id) = h.connect();

    client.write_all(&0x7F00_0001u32.to_be_bytes()).unwrap();
    client.write_all(&game_login_packet()).unwrap();
    h.pump(|_, c| !c.frames.is_empty());

    let ns = h.net.get(id).unwrap();
    assert_eq!(ns.seed, 0x7F00_0001);
    assert!(ns.seeded);
    assert_eq!(ns.protocol_state, ProtocolState::GameServerLoggedIn);
    assert_eq!(h.ctx.frames[0].0, 0x91);
    assert_eq!(h.ctx.frames[0].1.len(), 64);
    assert_eq!(&h.ctx.frames[0].1[..4], &0x1234_5678u32.to_be_bytes());
}

#[test]
fn variable_length_packet_split_across_reads_is_delivered_once() {
    let mut h = Harness::new();
    let (mut client, id) = h.connect();
    h.net.get_mut(id).unwrap().protocol_state = ProtocolState::GameServerLoggedIn;

    client.write_all(&[0x42, 0x00, 0x07, 1, 2]).unwrap();
    h.spin(5);
    assert!(h.ctx.frames.is_empty());
    assert_eq!(h.net.get(id).unwrap().recv_buffer().len(), 5);

    // The rest of the first packet and a whole second one in one write
    client.write_all(&[3, 4, 0x42, 0x00, 0x04, 9]).unwrap();
    h.pump(|_, c| c.frames.len() == 2);
    assert_eq!(h.ctx.frames[0], (0x42, vec![1, 2, 3, 4]));
    assert_eq!(h.ctx.frames[1], (0x42, vec![9]));
    assert!(h.net.get(id).unwrap().recv_buffer().is_empty());
}

#[test]
fn malformed_frames_disconnect_and_never_panic() {
    // Unknown packet id
    let mut h = Harness::new();
    let (mut client, id) = h.connect();
    h.net.get_mut(id).unwrap().protocol_state = ProtocolState::GameServerLoggedIn;
    client.write_all(&[0x7E, 1, 2, 3]).unwrap();
    h.pump(|n, _| {
        n.get(id)
            .is_some_and(|ns| ns.protocol_state == ProtocolState::Error)
    });
    assert!(
        h.net
            .get(id)
            .unwrap()
            .disconnect_reason
            .contains("bad state")
    );

    // Variable length claiming less than its own header
    let mut h = Harness::new();
    let (mut client, id) = h.connect();
    h.net.get_mut(id).unwrap().protocol_state = ProtocolState::GameServerLoggedIn;
    client.write_all(&[0x42, 0x00, 0x01]).unwrap();
    h.pump(|n, _| {
        n.get(id)
            .is_some_and(|ns| ns.protocol_state == ProtocolState::Error)
    });

    // The largest declarable length simply waits for the rest
    let mut h = Harness::new();
    let (mut client, id) = h.connect();
    h.net.get_mut(id).unwrap().protocol_state = ProtocolState::GameServerLoggedIn;
    client.write_all(&[0x42, 0xFF, 0xFF]).unwrap();
    h.pump(|n, _| n.get(id).is_some_and(|ns| ns.recv_buffer().len() == 3));
    assert!(h.net.get(id).unwrap().running());
}

#[test]
fn in_game_only_requires_a_live_mobile() {
    let mut h = Harness::new();
    let (mut client, id) = h.connect();
    h.net.get_mut(id).unwrap().protocol_state = ProtocolState::GameServerLoggedIn;
    client.write_all(&[0x43, 0, 0]).unwrap();
    h.pump(|n, _| {
        n.get(id)
            .is_some_and(|ns| ns.protocol_state == ProtocolState::Error)
    });
    assert!(h.ctx.frames.is_empty());

    let mut h = Harness::new();
    let (mut client, id) = h.connect();
    {
        let ns = h.net.get_mut(id).unwrap();
        ns.protocol_state = ProtocolState::GameServerLoggedIn;
        ns.attachment = Attachment::Mobile(1);
    }
    client.write_all(&[0x43, 7, 8]).unwrap();
    h.pump(|_, c| !c.frames.is_empty());
    assert_eq!(h.ctx.frames[0], (0x43, vec![7, 8]));

    // A deleted mobile is an error too
    h.net.get_mut(id).unwrap().attachment = Attachment::Deleted;
    client.write_all(&[0x43, 7, 8]).unwrap();
    h.pump(|n, _| {
        n.get(id)
            .is_some_and(|ns| ns.protocol_state == ProtocolState::Error)
    });
}

type ReportLog = Rc<RefCell<Vec<(IpAddr, String)>>>;

struct Recorder(ReportLog);

impl BanReporter for Recorder {
    fn name(&self) -> &str {
        "recorder"
    }
    fn register(&mut self) {}
    fn start(&mut self, _t: &crate::CancelToken) {}
    fn stop(&mut self) {}
    fn report(&mut self, address: IpAddr, _ttl: Duration, reason: &str) {
        self.0.borrow_mut().push((address, reason.to_string()));
    }
    fn can_retract(&self) -> bool {
        false
    }
    fn retract(&mut self, _a: IpAddr) {}
}

fn with_recorder(h: &mut Harness) -> ReportLog {
    let log: ReportLog = Rc::default();
    h.net.bans.register(Box::new(Recorder(log.clone())));
    log
}

#[test]
fn zero_seed_is_reported_and_disconnected() {
    let mut h = Harness::new();
    let reports = with_recorder(&mut h);
    let (mut client, id) = h.connect();
    client.write_all(&[0, 0, 0, 0, 0x91]).unwrap();
    h.pump(|n, _| n.get(id).is_none());
    assert_eq!(
        reports.borrow().as_slice(),
        &[("127.0.0.1".parse().unwrap(), "invalid-seed".to_string())]
    );
    assert_eq!(read_all(&mut client, 1), Vec::<u8>::new()); // peer sees EOF
}

#[test]
fn http_on_the_game_port_is_reported_as_foreign() {
    let mut h = Harness::new();
    let reports = with_recorder(&mut h);
    let (mut client, id) = h.connect();
    client
        .write_all(b"GET / HTTP/1.1\r\nHost: x\r\n\r\n")
        .unwrap();
    h.pump(|n, _| n.get(id).is_none());
    assert_eq!(reports.borrow()[0].1, "foreign-protocol");
}

#[test]
fn silent_connection_is_swept_and_reported() {
    let mut h = Harness::new();
    let reports = with_recorder(&mut h);
    let (mut client, id) = h.connect();

    // Inside the idle limit nothing happens
    let t0 = h.tick();
    h.net.slice(&mut h.ctx, t0 + 4000);
    assert!(h.net.get(id).is_some_and(|n| !n.is_disconnect_queued()));

    // Past it the disconnect is queued, handed off in that slice, and disposed in the next
    h.net.slice(&mut h.ctx, t0 + 5100);
    h.net.slice(&mut h.ctx, t0 + 5101);
    assert!(h.net.get(id).is_none());
    assert_eq!(reports.borrow()[0].1, "silent-connect");
    assert_eq!(read_all(&mut client, 1), Vec::<u8>::new());

    let events = h.net.take_events();
    assert!(
        events
            .iter()
            .any(|e| matches!(e, NetEvent::Disconnected(ns) if ns.id() == id))
    );
}

#[test]
fn a_one_byte_connection_is_dropped_without_a_ban() {
    let mut h = Harness::new();
    let reports = with_recorder(&mut h);
    let (mut client, id) = h.connect();
    // Waiting would hold a slot per one- or two-byte client, so it is dropped at once (not evidence)
    client.write_all(&[0x01]).unwrap();
    h.pump(|n, _| n.get(id).is_none());
    assert!(reports.borrow().is_empty());
}

struct DenyAll;

impl ConnectionFilter for DenyAll {
    fn name(&self) -> &str {
        "deny-all"
    }
    fn register(&mut self) {}
    fn start(&mut self, _t: &crate::CancelToken) {}
    fn stop(&mut self) {}
    fn should_deny(&mut self, _a: IpAddr) -> Result<bool, String> {
        Ok(true)
    }
}

#[test]
fn denied_and_rejected_connections_are_dropped_at_accept() {
    let mut h = Harness::new();
    h.net.filters.register(Box::new(DenyAll));
    let mut client = std::net::TcpStream::connect(h.addr).unwrap();
    client
        .set_read_timeout(Some(Duration::from_millis(20)))
        .unwrap();
    h.spin(50);
    assert_eq!(h.net.instance_count(), 0);
    assert_eq!(read_all(&mut client, 1), Vec::<u8>::new());

    let mut h = Harness::new();
    h.net.socket_connect = Some(Box::new(|_| false));
    let _client = std::net::TcpStream::connect(h.addr).unwrap();
    h.spin(50);
    assert_eq!(h.net.instance_count(), 0);
}

#[test]
fn send_before_disconnect_is_delivered_then_peer_sees_eof() {
    let mut h = Harness::new();
    let (mut client, id) = h.connect();
    let payload = [0x8C, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

    let ns = h.net.get_mut(id).unwrap();
    ns.send(&payload);
    ns.disconnect("redirect");
    h.slice(); // flush, then handoff

    assert_eq!(read_all(&mut client, payload.len()), payload);
    // FIN follows once the send is out
    assert_eq!(read_all(&mut client, 1), Vec::<u8>::new());
    assert!(h.net.get(id).is_some_and(|n| !n.is_connected()));

    // After the handoff nothing more can be queued
    let ns = h.net.get_mut(id).unwrap();
    ns.send(&[0x73, 0x00]);
    assert!(ns.cannot_send_packets());
    assert_eq!(ns.pending_send_bytes(), 0);

    h.slice();
    assert!(h.net.get(id).is_none());
}

#[test]
fn large_send_before_disconnect_is_fully_drained() {
    let mut h = Harness::new();
    let (mut client, id) = h.connect();
    // Larger than the loopback socket buffers, so the drain spans several slices
    let chunk = 32 * 1024;
    let payload: Vec<u8> = (0..6 * chunk).map(|i| (i * 31 + i / 7) as u8).collect();
    {
        let ns = h.net.get_mut(id).unwrap();
        for part in payload.chunks(chunk) {
            ns.send(part);
        }
        ns.disconnect("redirect");
    }

    let mut got = Vec::new();
    let deadline = Instant::now() + Duration::from_secs(10);
    let mut buf = [0u8; 16384];
    while got.len() < payload.len() {
        assert!(Instant::now() < deadline, "timed out at {}", got.len());
        h.slice();
        if let Ok(n) = client.read(&mut buf) {
            if n == 0 {
                break;
            }
            got.extend_from_slice(&buf[..n]);
        }
    }
    assert_eq!(got, payload);
    assert_eq!(read_all(&mut client, 1), Vec::<u8>::new());
}

#[test]
fn pending_disconnect_force_closes_after_drain_timeout() {
    let mut h = Harness::new();
    let (_client, id) = h.connect();
    let ns = h.net.get_mut(id).unwrap();
    ns.disconnect_pending = true; // as if the handoff found unsent output

    let armed = 1_000_000;
    ns.next_activity_check = armed + 10 * crate::net_state::DRAIN_TIMEOUT_MS; // only the deadline can fire
    ns.arm_drain_deadline(armed);

    ns.check_alive(armed + crate::net_state::DRAIN_TIMEOUT_MS - 1);
    assert!(ns.is_connected());

    ns.check_alive(armed + crate::net_state::DRAIN_TIMEOUT_MS);
    assert!(!ns.is_connected());
}

#[test]
fn send_buffer_exhausted_keeps_the_first_reason() {
    let mut h = Harness::new();
    let (_client, id) = h.connect();
    let max = h.net.config.max_send_buffer_size;
    let ns = h.net.get_mut(id).unwrap();

    ns.send(&vec![0; max + 1]); // cannot fit; queues the disconnect
    ns.send(&vec![0; max + 2]); // same tick; must not re-report

    assert!(
        ns.disconnect_reason
            .contains(&format!("needed {}", max + 1)),
        "{}",
        ns.disconnect_reason
    );
    assert!(ns.is_disconnect_queued());
    assert_eq!(ns.pending_send_bytes(), 0);
}

#[test]
fn authenticated_pre_game_client_gets_a_version_request_instead_of_a_disconnect() {
    let mut h = Harness::new();
    let (mut client, id) = h.connect();
    let ns = h.net.get_mut(id).unwrap();
    ns.account = Some("user".into());
    let t = ns.next_activity_check + 1;
    ns.check_alive(t);
    assert!(!ns.is_disconnect_queued());
    h.slice();
    assert_eq!(read_all(&mut client, 3), vec![0xBD, 0x00, 0x03]);

    // Without an account it is a disconnect
    let ns = h.net.get_mut(id).unwrap();
    ns.account = None;
    let t = ns.next_activity_check + 1;
    ns.check_alive(t);
    assert!(ns.is_disconnect_queued());
}

#[test]
fn compression_is_applied_to_outgoing_packets() {
    let mut h = Harness::new();
    let (mut client, id) = h.connect();
    let data = [0x11u8, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77];
    let mut expected = vec![0u8; 64];
    let n = crate::compression::compress(&data, &mut expected);
    expected.truncate(n);

    let ns = h.net.get_mut(id).unwrap();
    ns.compression_enabled = true;
    ns.send(&data);
    h.slice();
    assert_eq!(read_all(&mut client, expected.len()), expected);
}

#[test]
fn encrypted_game_login_is_detected_and_the_stream_is_encrypted_both_ways() {
    let mut h = Harness::new();
    let (mut client, id) = h.connect();
    let seed = 0x0A0B_0C0Du32;

    // The client encrypts everything after the seed with the seed-keyed stream
    let mut login = game_login_packet();
    login[1..5].copy_from_slice(&0x0000_00AAu32.to_be_bytes()); // an auth id that is not the seed
    GameEncryption::new(seed).client_decrypt(&mut login); // xor with the same table = encrypt

    client.write_all(&seed.to_be_bytes()).unwrap();
    client.write_all(&login).unwrap();
    h.pump(|_, c| !c.frames.is_empty());

    assert!(h.net.get(id).unwrap().has_encryption());
    assert_eq!(h.ctx.frames[0].1[..4], 0x0000_00AAu32.to_be_bytes());

    h.net
        .get_mut(id)
        .unwrap()
        .send(&(1..=18).collect::<Vec<u8>>());
    h.slice();
    let mut got = read_all(&mut client, 18);
    GameEncryption::new(seed).server_encrypt(&mut got); // xor again with the key stream = decrypt
    assert_eq!(got, (1..=18).collect::<Vec<u8>>());
}

#[test]
fn encrypted_clients_are_rejected_when_the_mode_forbids_it() {
    let mut h = Harness::new();
    h.net.config.encryption.mode = crate::encryption::EncryptionMode::UNENCRYPTED;
    let (mut client, id) = h.connect();
    let seed = 0x0A0B_0C0Du32;
    let mut login = game_login_packet();
    login[1..5].copy_from_slice(&0x0000_00AAu32.to_be_bytes());
    GameEncryption::new(seed).client_decrypt(&mut login);

    client.write_all(&seed.to_be_bytes()).unwrap();
    client.write_all(&login).unwrap();
    h.pump(|n, _| {
        n.get(id)
            .is_some_and(|ns| ns.protocol_state == ProtocolState::Error)
    });
    assert!(h.ctx.frames.is_empty());
}

#[test]
fn server_select_ack_resets_to_awaiting_seed() {
    let mut h = Harness::new();
    let (mut client, id) = h.connect();
    h.net.get_mut(id).unwrap().protocol_state = ProtocolState::LoginServerServerSelectAck;
    client.write_all(&[0xAA]).unwrap();
    h.pump(|n, _| {
        n.get(id)
            .is_some_and(|ns| ns.protocol_state == ProtocolState::AwaitingSeed)
    });
    assert!(h.net.get(id).unwrap().running());
}

#[test]
fn stats_and_idle() {
    let mut h = Harness::new();
    let (_client, id) = h.connect();
    h.net.get_mut(id).unwrap().account = Some("a".into());
    let stats = h.net.get_network_stats();
    assert_eq!((stats.connected, stats.authenticated), (1, 1));
    assert_eq!(stats.max_connections, 4096);
    assert!(h.net.is_idle());
    h.net.get_mut(id).unwrap().send(&[1]);
    assert!(!h.net.is_idle());
}

#[test]
fn slot_ids_are_reused_but_serials_are_not() {
    let mut h = Harness::new();
    let (client, id) = h.connect();
    let serial = h.net.get(id).unwrap().serial;
    drop(client);
    h.pump(|n, _| n.get(id).is_none());
    let (_c2, id2) = h.connect();
    assert_eq!(id2, id);
    assert_ne!(h.net.get(id2).unwrap().serial, serial);
}
