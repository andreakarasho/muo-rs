//! Damage packets.

use super::Pk;
use crate::net_state::{NetCaps, NetStateId};
use crate::world::World;
use muo_primitives::Serial;

/// `caps.damage_packet()` is `ns.DamagePacket` (the protocol change): the 0x0B form, else the old 0xBF/0x22 one.
pub fn create_damage(caps: &NetCaps, serial: Serial, amount: i32) -> Vec<u8> {
    let new = caps.damage_packet();
    let mut p = Pk::new(if new { 7 } else { 11 });
    if new {
        p.u8(0x0B)
            .serial(serial)
            .u16(amount.clamp(0, 0xFFFF) as u16);
    } else {
        p.u8(0xBF)
            .u16(11)
            .u16(0x22)
            .u8(1)
            .serial(serial)
            .u8(amount.clamp(0, 0xFF) as u8);
    }
    p.finish()
}

pub fn send_damage(w: &mut World, ns: NetStateId, serial: Serial, amount: i32) {
    if w.cannot_send_packets(ns) {
        return;
    }
    let Some(d) = w.net_state(ns) else { return };
    let bytes = create_damage(&d.caps, serial, amount);
    w.send(ns, &bytes);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net_state::{NetOut, NetStateData, NetStateId};
    use crate::world::World;
    use muo_net::protocol_changes::ProtocolChanges;

    /// Bytes the closure queued for a fresh connection with the given caps.
    fn sent(caps: NetCaps, f: impl FnOnce(&mut World, NetStateId)) -> Vec<u8> {
        let mut w = World::new();
        let ns = NetStateId(1);
        w.net_states.insert(
            ns,
            NetStateData {
                caps,
                ..Default::default()
            },
        );
        f(&mut w, ns);
        match w.net_out.pop() {
            Some(NetOut::Send(_, b)) => b,
            _ => Vec::new(),
        }
    }

    fn caps(new: bool) -> NetCaps {
        NetCaps {
            protocol_changes: if new {
                ProtocolChanges::DAMAGE_PACKET
            } else {
                ProtocolChanges::empty()
            },
            ..Default::default()
        }
    }

    #[test]
    fn damage_packet_old() {
        for (amt, want) in [(10, 10u8), (-5, 0), (1024, 255)] {
            let b = sent(caps(false), |w, ns| {
                send_damage(w, ns, Serial::new(0x1024), amt)
            });
            assert_eq!(b, [0xBF, 0, 11, 0, 0x22, 1, 0, 0, 0x10, 0x24, want]);
        }
    }

    #[test]
    fn damage_packet() {
        for (amt, want) in [(10, 10u16), (-5, 0), (1024, 1024), (100000, 0xFFFF)] {
            let b = sent(caps(true), |w, ns| {
                send_damage(w, ns, Serial::new(0x1024), amt)
            });
            assert_eq!(b, [0x0B, 0, 0, 0x10, 0x24, (want >> 8) as u8, want as u8]);
        }
    }
}
