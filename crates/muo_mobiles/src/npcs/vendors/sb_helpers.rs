//! Helpers the generated SBInfo tables call: the spell scroll loops and the preset maps.

use muo_content::loot::REGULAR_SCROLL_TYPES;
use muo_content::vendor::{BuyInfo, SbInstance};
use muo_server::world::World;

/// `for (i = 0; i < limit && i < types.Length; ++i) Add(new GenericBuyInfo(types[i], 12 + i / 8 * 10, 20, itemID, 0))`
/// with the item id `0x1F2E + i`, shifted around the Recall/Mark gap.
pub fn scroll_buy(_w: &mut World, sb: &mut SbInstance, limit: usize) {
    for (i, ty) in REGULAR_SCROLL_TYPES.iter().enumerate().take(limit) {
        let mut item_id = 0x1F2E + i as i32;
        if i == 6 {
            item_id = 0x1F2D;
        } else if i > 6 {
            item_id -= 1;
        }
        sb.buy.push(BuyInfo::new(
            None,
            ty,
            12 + (i / 8) as i32 * 10,
            20,
            item_id,
            0,
        ));
    }
}

/// `for each scroll type: Add(types[i], (i / 8 + 2) * mult)`.
pub fn scroll_sell(sb: &mut SbInstance, mult: i32) {
    for (i, ty) in REGULAR_SCROLL_TYPES.iter().enumerate() {
        sb.sell.add(ty, (i as i32 / 8 + 2) * mult);
    }
}

/// `for each PresetMapEntry: Add(new PresetMapBuyInfo(entry, Utility.RandomMinMax(7, 10), 20))`.
pub fn preset_maps(w: &mut World, sb: &mut SbInstance) {
    let names = muo_content::vendor::hooks(w)
        .and_then(|h| h.preset_map_names)
        .map(|f| f(w))
        .unwrap_or_default();
    for (i, name) in names.into_iter().enumerate() {
        let price = muo_content::hooks::rnd::min_max(w, 7, 10);
        sb.buy.push(BuyInfo::preset_map(i, name, price, 20));
    }
}
