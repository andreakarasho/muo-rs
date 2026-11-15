//! Arcane charges of the GolemController's equipment (`creatureItem` `arcaneCharges`).

use muo_server::World;
use muo_server::ids::ItemId;

/// `eq.CurArcaneCharges = eq.MaxArcaneCharges = value` for the arcane clothing and gloves.
pub(crate) fn set_arcane_charges(w: &mut World, item: ItemId, value: i32) {
    use muo_items::{armor, clothing};
    if clothing::special::arcane_equip(w, item).is_some() {
        clothing::special::set_max_arcane_charges(w, item, value);
        clothing::special::set_cur_arcane_charges(w, item, value);
    } else if w
        .item(item)
        .is_some_and(|i| i.ty.is(&armor::special::LEATHER_GLOVES))
    {
        armor::special::set_max_arcane_charges(w, item, value, 0x13C6);
        armor::special::set_cur_arcane_charges(w, item, value, 0x13C6);
    }
}
