//! GolemController.

use muo_server::World;
use muo_server::ids::{ItemId, MobileId};
use muo_server::item::LootType;
use muo_server::mobile::part2::items;

use super::ctors::si;

/// `AddArcane(item)`: a charged arcane piece, tinted and newbied, worn by the controller.
fn add_arcane(w: &mut World, id: MobileId, name: &str) {
    let Some(item) = muo_content::factory::make_item(w, &si(name)) else {
        return;
    };
    set_charges(w, item, 20);
    muo_server::item::base::set_hue(w, item, muo_items::misc::gadgets::DEFAULT_ARCANE_HUE);
    muo_server::item::ops::set_loot_type(w, item, LootType::Newbied);
    items::add_item(w, id, item);
}

/// `eq.CurArcaneCharges = eq.MaxArcaneCharges = value` for the arcane clothing and gloves.
fn set_charges(w: &mut World, item: ItemId, value: i32) {
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

pub fn golem_controller_ctor1(w: &mut World, id: MobileId) {
    add_arcane(w, id, "Robe");
}
pub fn golem_controller_ctor2(w: &mut World, id: MobileId) {
    add_arcane(w, id, "ThighBoots");
}
pub fn golem_controller_ctor3(w: &mut World, id: MobileId) {
    add_arcane(w, id, "LeatherGloves");
}
pub fn golem_controller_ctor4(w: &mut World, id: MobileId) {
    add_arcane(w, id, "Cloak");
}
