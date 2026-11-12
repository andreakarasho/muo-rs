//! What the generated humanoid constructors (`humans.rs`) call: equipping by class name, gold packs.

use muo_content::hooks::rnd;
use muo_server::ids::{ItemId, MobileId};
use muo_server::item::{container, ops};
use muo_server::mobile::part2::items;
use muo_server::world::World;

pub use super::hires::{BASE_HIRE, BASE_HIRE_C, new_hire};

/// `Female`.
pub fn npc_female(w: &World, id: MobileId) -> bool {
    w.mobile(id).is_some_and(|m| m.female)
}

/// `EquipItem(new T(hue))`: the hue argument was read before the constructor ran. A refused item stays
/// unreferenced.
pub fn equip_new(w: &mut World, id: MobileId, full_name: &str, hue: Option<i32>) {
    if let Some(it) = muo_content::factory::make_item(w, full_name) {
        if let Some(h) = hue {
            muo_server::item::base::set_hue(w, it, h);
        }
        muo_server::mobile::base::equip_item(w, id, it);
    }
}

/// `AddItem(new T(hue))`.
pub fn add_new_hued(w: &mut World, id: MobileId, full_name: &str, hue: Option<i32>) {
    if let Some(it) = muo_content::factory::make_item(w, full_name) {
        if let Some(h) = hue {
            muo_server::item::base::set_hue(w, it, h);
        }
        items::add_item(w, id, it);
    }
}

/// `EquipItem(item)` of an item picked by a `switch` expression (`null` arms equip nothing).
pub fn equip_pick(w: &mut World, id: MobileId, pick: Option<&str>) {
    if let Some(n) = pick {
        equip_new(w, id, n, None);
    }
}

/// `var pack = new Backpack { Movable = false }; pack.DropItem(new Gold(min, max)); AddItem(pack);`
pub fn add_gold_pack(w: &mut World, id: MobileId, min: i32, max: i32) {
    let Some(pack) = muo_content::factory::make_item(w, "Server.Items.Backpack") else {
        return;
    };
    ops::set_movable(w, pack, false);
    let amount = rnd::min_max(w, min, max);
    if let Some(g) = muo_content::factory::make_item(w, "Server.Items.Gold") {
        ops::set_amount(w, g, amount);
        container::drop_item_default(w, pack, g);
    }
    items::add_item(w, id, pack);
}

#[allow(dead_code)]
fn _unused(_: ItemId) {}

/// `new Backpack { Movable = false }`.
pub fn new_unmovable_pack(w: &mut World) -> Option<ItemId> {
    let pack = muo_content::factory::make_item(w, "Server.Items.Backpack")?;
    ops::set_movable(w, pack, false);
    Some(pack)
}

/// `pack.DropItem(new Gold(min, max))`.
pub fn drop_gold(w: &mut World, pack: Option<ItemId>, min: i32, max: i32) {
    let amount = rnd::min_max(w, min, max);
    if let (Some(pack), Some(g)) = (
        pack,
        muo_content::factory::make_item(w, "Server.Items.Gold"),
    ) {
        ops::set_amount(w, g, amount);
        container::drop_item_default(w, pack, g);
    }
}

/// `AddItem(pack)`.
pub fn add_pack(w: &mut World, id: MobileId, pack: Option<ItemId>) {
    if let Some(p) = pack {
        items::add_item(w, id, p);
    }
}
