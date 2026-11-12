//! ItemFixes: patches the tile data the client files get wrong.

use muo_data::tile_data::TileFlag;
use muo_server::World;

/// `ItemFixes.Initialize()`.
pub fn initialize(w: &mut World) {
    let items = &mut w.maps.tile_data.items;
    if items.is_empty() {
        return;
    }
    // Missing NoShoot flags
    for id in [0x2A0, 0x3E0, 0x3E1] {
        if let Some(i) = items.get_mut(id) {
            i.flags |= TileFlag::NO_SHOOT;
        }
    }

    // Incorrect height
    if let Some(i) = items.get_mut(0x34D2) {
        i.height = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn patches_flags_and_height() {
        let mut w = World::new();
        w.maps.tile_data = muo_data::tile_data::TileData::empty();
        assert!(w.maps.tile_data.items.len() > 0x34D2);
        w.maps.tile_data.items[0x34D2].height = 9;
        initialize(&mut w);
        let t = &w.maps.tile_data.items;
        assert!(
            t[0x2A0].flags.contains(TileFlag::NO_SHOOT)
                && t[0x3E0].flags.contains(TileFlag::NO_SHOOT)
        );
        assert!(t[0x3E1].flags.contains(TileFlag::NO_SHOOT));
        assert!(!t[0x2A1].flags.contains(TileFlag::NO_SHOOT));
        assert_eq!(t[0x34D2].height, 0);
    }
}
