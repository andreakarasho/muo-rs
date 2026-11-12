//! Movement packets.

use muo_primitives::Direction;

use super::Pk;
use crate::ids::MobileId;
use crate::net_state::NetStateId;
use crate::world::World;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum SpeedControlSetting {
    Disable = 0,
    Mount = 1,
    Walk = 2,
}

pub fn create_speed_control(speed_control: SpeedControlSetting) -> Vec<u8> {
    vec![0xBF, 0x00, 0x6, 0x00, 0x26, speed_control as u8]
}

pub fn send_speed_control(w: &mut World, ns: NetStateId, speed_control: SpeedControlSetting) {
    w.send(ns, &create_speed_control(speed_control));
}

pub fn create_move_player(d: Direction) -> Vec<u8> {
    vec![0x97, d.bits()]
}

pub fn send_move_player(w: &mut World, ns: NetStateId, d: Direction) {
    w.send(ns, &create_move_player(d));
}

/// `Notoriety.Compute(source, target)`.
pub use crate::notoriety::compute as notoriety_compute;

pub fn create_movement_ack(seq: i32, noto: i32) -> Vec<u8> {
    vec![0x22, seq as u8, noto as u8]
}

/// `SendMovementAck(seq, Mobile m)`: the notoriety is `Compute(m, m)`.
pub fn send_movement_ack_mobile(w: &mut World, ns: NetStateId, seq: i32, m: MobileId) {
    let noto = notoriety_compute(w, m, m);
    send_movement_ack(w, ns, seq, noto);
}

pub fn send_movement_ack(w: &mut World, ns: NetStateId, seq: i32, noto: i32) {
    w.send(ns, &create_movement_ack(seq, noto));
}

pub fn create_movement_rej(w: &World, seq: i32, m: MobileId) -> Vec<u8> {
    let mut pk = Pk::new(8);
    pk.u8(0x21); // Packet ID
    pk.u8(seq as u8);
    if let Some(m) = w.mobile(m) {
        pk.i16(m.location.x as i16);
        pk.i16(m.location.y as i16);
        pk.u8(m.direction.bits());
        pk.i8(m.location.z as i8);
    }
    pk.finish()
}

pub fn send_movement_rej(w: &mut World, ns: NetStateId, seq: i32, m: MobileId) {
    if w.cannot_send_packets(ns) {
        return;
    }
    let bytes = create_movement_rej(w, seq, m);
    w.send(ns, &bytes);
}

pub fn create_time_sync_response(tick_count: i64) -> Vec<u8> {
    let mut pk = Pk::new(25);
    pk.u8(0xF2); // Packet ID
    for _ in 0..3 {
        let _ = pk.write_i64(tick_count); // ??
    }
    pk.finish()
}

pub fn send_time_sync_response(w: &mut World, ns: NetStateId) {
    if w.cannot_send_packets(ns) {
        return;
    }
    let bytes = create_time_sync_response(w.tick_count);
    w.send(ns, &bytes);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mobile::MOBILE_BASE;
    use crate::net_state::{NetOut, NetStateData};

    fn world_with_ns() -> (World, NetStateId) {
        let mut w = World::new();
        let ns = NetStateId(1);
        w.net_states.insert(ns, NetStateData::default());
        (w, ns)
    }

    fn sent(w: &mut World) -> Vec<u8> {
        match w.net_out.pop() {
            Some(NetOut::Send(_, b)) => b,
            _ => panic!("nothing sent"),
        }
    }

    #[test]
    fn speed_control() {
        for s in [
            SpeedControlSetting::Disable,
            SpeedControlSetting::Mount,
            SpeedControlSetting::Walk,
        ] {
            let (mut w, ns) = world_with_ns();
            send_speed_control(&mut w, ns, s);
            // legacy SpeedControl: 0xBF, length 6, subpacket 0x26, setting
            assert_eq!(sent(&mut w), vec![0xBF, 0, 6, 0, 0x26, s as u8]);
        }
    }

    #[test]
    fn move_player() {
        let (mut w, ns) = world_with_ns();
        send_move_player(&mut w, ns, Direction::LEFT);
        assert_eq!(sent(&mut w), vec![0x97, 5]);
    }

    #[test]
    fn movement_rej() {
        let (mut w, ns) = world_with_ns();
        let id = w.new_mobile(&MOBILE_BASE);
        {
            let m = w.mobile_mut(id).unwrap();
            m.location = muo_primitives::Point3D {
                x: 0x1234,
                y: 0x0102,
                z: -5,
            };
            m.direction = Direction::EAST;
        }
        send_movement_rej(&mut w, ns, 100, id);
        assert_eq!(
            sent(&mut w),
            vec![0x21, 100, 0x12, 0x34, 0x01, 0x02, 2, 0xFB]
        );
    }

    #[test]
    fn movement_ack() {
        let (mut w, ns) = world_with_ns();
        let id = w.new_mobile(&MOBILE_BASE);
        send_movement_ack_mobile(&mut w, ns, 100, id);
        assert_eq!(sent(&mut w), vec![0x22, 100, 3]);
    }

    #[test]
    fn time_sync() {
        let (mut w, ns) = world_with_ns();
        w.tick_count = 0x0102;
        send_time_sync_response(&mut w, ns);
        let b = sent(&mut w);
        assert_eq!(b.len(), 25);
        assert_eq!(&b[1..9], &[0, 0, 0, 0, 0, 0, 1, 2]);
    }
}
