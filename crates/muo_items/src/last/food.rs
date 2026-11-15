//! The chocolates that derive from `CandyCane`.

use muo_server::World;
use muo_server::ids::ItemId;
use muo_server::item::{ItemType, LootType, ops};
use muo_server::type_registry::TypeRegistry;

use crate::simple::{hue, leaf};
use crate::special::holiday::CANDY_CANE;
use muo_content::items::food::new_food;

/// `Chocolate()`: `CandyCane(itemID)` (not stackable, blessed), then `Hue` and `LootType.Regular`.
fn chocolate_new(w: &mut World, ty: &'static ItemType, item_id: i32, h: i32) -> ItemId {
    let id = new_food(w, ty, item_id, 1);
    ops::set_stackable(w, id, false);
    ops::set_loot_type(w, id, LootType::Blessed);
    hue(w, id, h);
    ops::set_loot_type(w, id, LootType::Regular);
    id
}

leaf! {
    /// `Server.Items.DarkChocolate`.
    DARK_CHOCOLATE = "Server.Items.DarkChocolate" : CANDY_CANE, v0;
    construct: Some(|w| chocolate_new(w, &DARK_CHOCOLATE, 0xF10, 0x465)),
    label_number: crate::class_props::label_number, // Dark chocolate
    default_weight: crate::class_props::default_weight,
}

leaf! {
    /// `Server.Items.MilkChocolate`.
    MILK_CHOCOLATE = "Server.Items.MilkChocolate" : CANDY_CANE, v0;
    construct: Some(|w| chocolate_new(w, &MILK_CHOCOLATE, 0xF18, 0x461)),
    label_number: crate::class_props::label_number, // Milk chocolate
    default_weight: crate::class_props::default_weight,
}

leaf! {
    /// `Server.Items.WhiteChocolate`.
    WHITE_CHOCOLATE = "Server.Items.WhiteChocolate" : CANDY_CANE, v0;
    construct: Some(|w| chocolate_new(w, &WHITE_CHOCOLATE, 0xF11, 0x47E)),
    label_number: crate::class_props::label_number, // White chocolate
    default_weight: crate::class_props::default_weight,
}

/// Registers the classes of this file.
pub fn register(types: &mut TypeRegistry) {
    for t in [&DARK_CHOCOLATE, &MILK_CHOCOLATE, &WHITE_CHOCOLATE] {
        types.register_item(t, &[]);
    }
}
