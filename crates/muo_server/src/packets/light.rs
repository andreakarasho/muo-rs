//! Light level packets.

use super::Pk;
use crate::net_state::NetStateId;
use crate::world::World;
use muo_primitives::Serial;

pub fn create_personal_light_level(serial: Serial, level: i32) -> Vec<u8> {
    let mut p = Pk::new(6);
    p.u8(0x4E).serial(serial).u8(level as u8);
    p.finish()
}

pub fn send_personal_light_level(w: &mut World, ns: NetStateId, serial: Serial, level: i32) {
    if w.cannot_send_packets(ns) {
        return;
    }
    w.send(ns, &create_personal_light_level(serial, level));
}

/// `SendGlobalLightLevel(level = 0)`.
pub fn send_global_light_level(w: &mut World, ns: NetStateId, level: i32) {
    w.send(ns, &[0x4F, level as u8]);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net_state::NetCaps;
    use crate::net_state::{NetOut, NetStateData, NetStateId};
    use crate::world::World;

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

    #[test]
    fn global_light_level() {
        // GlobalLightLevel(5): 0x4F, (sbyte)5
        assert_eq!(
            sent(NetCaps::default(), |w, ns| send_global_light_level(
                w, ns, 5
            )),
            [0x4F, 5]
        );
        assert_eq!(
            sent(NetCaps::default(), |w, ns| send_global_light_level(
                w, ns, -1
            )),
            [0x4F, 0xFF]
        );
    }

    #[test]
    fn personal_light_level() {
        let b = sent(NetCaps::default(), |w, ns| {
            send_personal_light_level(w, ns, Serial::new(0x1024), 5)
        });
        assert_eq!(b, [0x4E, 0, 0, 0x10, 0x24, 5]);
    }
}
