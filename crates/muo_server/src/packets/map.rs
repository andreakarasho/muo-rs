//! Map packets.

use super::Pk;
use crate::map::MapId;
use crate::net_state::NetStateId;
use crate::world::World;

/// One map's `Tiles.Patch` counts (`StaticBlocks`, `LandBlocks`); `None` for a map that isn't loaded (writes 0, 0).
pub type MapPatchCounts = Option<(i32, i32)>;

/// The counts never change, so the 41 bytes are equal on every call.
pub fn create_map_patches(maps: &[MapPatchCounts; 4]) -> Vec<u8> {
    let mut p = Pk::new(41);
    p.u8(0xBF).u16(41).u16(0x18).i32(4);
    for m in maps {
        let (s, l) = m.unwrap_or((0, 0));
        p.i32(s).i32(l);
    }
    p.finish()
}

pub fn send_map_patches(w: &mut World, ns: NetStateId, maps: &[MapPatchCounts; 4]) {
    if w.cannot_send_packets(ns) {
        return;
    }
    w.send(ns, &create_map_patches(maps));
}

/// `SendMapPatches()`: the patch counts of `Map.Maps[0..4]` read from the world.
pub fn send_map_patches_of_world(w: &mut World, ns: NetStateId) {
    let mut counts: [MapPatchCounts; 4] = [None; 4];
    for (i, slot) in counts.iter_mut().enumerate() {
        let id = MapId(i as u8);
        if w.maps.get(id).is_some() {
            *slot = w.maps.tiles_mut(id).map(|t| {
                let p = t.patches();
                (p.static_blocks, p.land_blocks)
            });
        }
    }
    send_map_patches(w, ns, &counts);
}

pub fn send_invalid_map(w: &mut World, ns: NetStateId) {
    w.send(ns, &[0xC6]);
}

/// `SendMapChange(map)`; nothing for a null map, pass `None`.
pub fn send_map_change(w: &mut World, ns: NetStateId, map: Option<MapId>) {
    let Some(map) = map else { return };
    w.send(ns, &[0xBF, 0x00, 0x06, 0x00, 0x08, map.0]);
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
    fn map_patches() {
        let maps = [Some((1, 2)), Some((3, 4)), None, Some((0x10203, 5))];
        let b = sent(NetCaps::default(), |w, ns| send_map_patches(w, ns, &maps));
        let mut want = vec![0xBF, 0, 41, 0, 0x18, 0, 0, 0, 4];
        for v in [1i32, 2, 3, 4, 0, 0, 0x10203, 5] {
            want.extend(v.to_be_bytes());
        }
        assert_eq!(b, want);
        assert_eq!(b.len(), 41);
    }

    #[test]
    fn invalid_map_enable() {
        assert_eq!(sent(NetCaps::default(), send_invalid_map), [0xC6]);
    }

    #[test]
    fn map_change() {
        for id in [0u8, 2] {
            let b = sent(NetCaps::default(), |w, ns| {
                send_map_change(w, ns, Some(MapId(id)))
            });
            assert_eq!(b, [0xBF, 0, 6, 0, 8, id]);
        }
        assert!(sent(NetCaps::default(), |w, ns| send_map_change(w, ns, None)).is_empty());
    }
}
