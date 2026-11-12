//! Combat packets.

use super::Pk;
use crate::net_state::NetStateId;
use crate::world::World;
use muo_primitives::Serial;

pub fn create_swing(attacker: Serial, defender: Serial) -> Vec<u8> {
    let mut p = Pk::new(10);
    p.u8(0x2F).u8(0).serial(attacker).serial(defender);
    p.finish()
}

pub fn send_swing(w: &mut World, ns: NetStateId, attacker: Serial, defender: Serial) {
    if w.cannot_send_packets(ns) {
        return;
    }
    w.send(ns, &create_swing(attacker, defender));
}

pub fn send_set_war_mode(w: &mut World, ns: NetStateId, warmode: bool) {
    w.send(ns, &[0x72, warmode as u8, 0x00, 0x32, 0x00]);
}

pub fn create_change_combatant(combatant: Serial) -> Vec<u8> {
    let mut p = Pk::new(5);
    p.u8(0xAA).serial(combatant);
    p.finish()
}

pub fn send_change_combatant(w: &mut World, ns: NetStateId, combatant: Serial) {
    if w.cannot_send_packets(ns) {
        return;
    }
    w.send(ns, &create_change_combatant(combatant));
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
    fn swing() {
        let b = sent(NetCaps::default(), |w, ns| {
            send_swing(w, ns, Serial::new(0x1024), Serial::new(0x2048))
        });
        assert_eq!(b, [0x2F, 0, 0, 0, 0x10, 0x24, 0, 0, 0x20, 0x48]);
    }

    #[test]
    fn set_war_mode() {
        for m in [true, false] {
            let b = sent(NetCaps::default(), |w, ns| send_set_war_mode(w, ns, m));
            assert_eq!(b, [0x72, m as u8, 0, 0x32, 0]);
        }
    }

    #[test]
    fn change_combatant() {
        let b = sent(NetCaps::default(), |w, ns| {
            send_change_combatant(w, ns, Serial::new(0x1024))
        });
        assert_eq!(b, [0xAA, 0, 0, 0x10, 0x24]);
    }
}
