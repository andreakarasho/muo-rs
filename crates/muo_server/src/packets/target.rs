//! Target packets.

use muo_primitives::Point3D;

use super::Pk;
use crate::net_state::NetStateId;
use crate::target::{MultiTarget, TargetFlags, TargetId};
use crate::world::World;

/// `SendMultiTargetReq` (0x99): 30 bytes on High Seas, else 26.
pub fn create_multi_target_req(
    allow_ground: bool,
    target_id: TargetId,
    flags: TargetFlags,
    multi: MultiTarget,
    high_seas: bool,
) -> Vec<u8> {
    let mut p = Pk::new(if high_seas { 30 } else { 26 });
    p.u8(0x99)
        .bool(allow_ground)
        .i32(target_id.0 as i32)
        .u8(flags.bits())
        .zeros(11);
    let Point3D { x, y, z } = multi.offset;
    p.i16(multi.multi_id as i16)
        .i16(x as i16)
        .i16(y as i16)
        .i16(z as i16);
    if high_seas {
        p.i32(0);
    }
    p.finish()
}

pub fn send_multi_target_req(
    w: &mut World,
    ns: NetStateId,
    allow_ground: bool,
    target_id: TargetId,
    flags: TargetFlags,
    multi: MultiTarget,
) {
    if w.cannot_send_packets(ns) {
        return;
    }
    let high_seas = w.net_state(ns).is_some_and(|d| d.caps.high_seas());
    let bytes = create_multi_target_req(allow_ground, target_id, flags, multi, high_seas);
    w.send(ns, &bytes);
}

/// `SendCancelTarget`: 0x6C with cancel type 3.
pub fn create_cancel_target() -> Vec<u8> {
    let mut b = vec![0u8; 19];
    b[0] = 0x6C;
    b[6] = 0x03;
    b
}

/// `ns?.Send(..)`: no `CannotSendPackets` check beyond `Send`'s own.
pub fn send_cancel_target(w: &mut World, ns: NetStateId) {
    w.send(ns, &create_cancel_target());
}

/// `SendTargetReq` (0x6C).
pub fn create_target_req(allow_ground: bool, target_id: TargetId, flags: TargetFlags) -> Vec<u8> {
    let mut p = Pk::new(19);
    p.u8(0x6C)
        .bool(allow_ground)
        .i32(target_id.0 as i32)
        .u8(flags.bits())
        .zeros(12);
    p.finish()
}

pub fn send_target_req(
    w: &mut World,
    ns: NetStateId,
    allow_ground: bool,
    target_id: TargetId,
    flags: TargetFlags,
) {
    if w.cannot_send_packets(ns) {
        return;
    }
    let bytes = create_target_req(allow_ground, target_id, flags);
    w.send(ns, &bytes);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net_state::{NetOut, NetStateData};
    use muo_net::protocol_changes::ProtocolChanges;

    /// The legacy `TargetReq`: id 0x6C, length 19, then `Fill()` zeros.
    #[test]
    fn target_req() {
        let b = create_target_req(true, TargetId(7), TargetFlags::BENEFICIAL);
        let mut e = vec![0u8; 19];
        e[0] = 0x6C;
        e[1] = 1;
        e[5] = 7;
        e[6] = 2;
        assert_eq!(b, e);
    }

    #[test]
    fn cancel_target_is_the_static_packet() {
        let e = [0x6C, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        assert_eq!(create_cancel_target(), e);
    }

    /// The legacy `MultiTargetReq`/`MultiTargetReqHS`: header, fill, then four shorts at offset 18.
    #[test]
    fn multi_target_req() {
        let m = MultiTarget {
            multi_id: 0x1024,
            offset: Point3D {
                x: 1000,
                y: 100,
                z: 10,
            },
        };
        for hs in [false, true] {
            let b = create_multi_target_req(true, TargetId(3), TargetFlags::NONE, m, hs);
            let mut e = vec![0u8; if hs { 30 } else { 26 }];
            e[0] = 0x99;
            e[1] = 1;
            e[5] = 3;
            e[18..20].copy_from_slice(&0x1024i16.to_be_bytes());
            e[20..22].copy_from_slice(&1000i16.to_be_bytes());
            e[22..24].copy_from_slice(&100i16.to_be_bytes());
            e[24..26].copy_from_slice(&10i16.to_be_bytes());
            assert_eq!(b, e);
        }
    }

    #[test]
    fn send_reads_the_client_era() {
        let ns = NetStateId(1);
        for hs in [false, true] {
            let mut w = World::new();
            let mut d = NetStateData::default();
            if hs {
                d.caps.protocol_changes = ProtocolChanges::HIGH_SEAS;
            }
            w.net_states.insert(ns, d);
            let m = MultiTarget {
                multi_id: 1,
                offset: Point3D::default(),
            };
            send_multi_target_req(&mut w, ns, true, TargetId(1), TargetFlags::NONE, m);
            let Some(NetOut::Send(_, b)) = w.net_out.last() else {
                panic!("nothing sent")
            };
            assert_eq!(b.len(), if hs { 30 } else { 26 });
        }
    }
}
