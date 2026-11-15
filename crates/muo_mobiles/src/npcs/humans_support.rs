//! What the humanoid classes (`humans.rs`) and their template keys (`template_keys_npcs`) call.

use muo_content::hooks::rnd;
use muo_server::ids::MobileId;
use muo_server::item::{container, ops};
use muo_server::mobile::part2::items;
use muo_server::world::World;

pub use super::hires::{BASE_HIRE, BASE_HIRE_C, new_template_hire};

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
