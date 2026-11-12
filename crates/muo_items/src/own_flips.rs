//! The `Flip()` methods that a bare `[Flippable]` invokes (`FlippableAttribute.Flip` with no ItemIDs), for the
//! classes in this crate whose flip is not a `ClothingClass::flip` (those are found through the class).
//!
//! Futon, ParagonChest, the hanging lanterns, WallSconce and WallTorch are registered by class name: their `Flip()`
//! reads and writes only `ItemID` (and `Light`), so it works whether or not the class is registered (BlueSoulstone
//! registers its own in `special::soulstone`; PaperLantern, RoundPaperLantern, ShojiLantern and the Reward clothes
//! declare `[Flippable]` without a `Flip()` method, which does nothing).

use muo_server::World;
use muo_server::ids::ItemId;
use muo_server::item::ops;
use muo_server::light_type::LightType;
use muo_server::type_registry::{Phase, TypeRegistry};

/// `ItemID = ItemID switch { a => b, .. }`: a graphic not listed stays.
fn swap_ids(w: &mut World, id: ItemId, pairs: &[(i32, i32)]) {
    let cur = w.item(id).map_or(0, |i| i.item_id);
    if let Some(&(_, to)) = pairs.iter().find(|(from, _)| *from == cur) {
        crate::simple::item_id(w, id, to);
    }
}

/// `Light = Light switch { WestBig => NorthBig, NorthBig => WestBig, _ => Light }`.
fn flip_big_light(w: &mut World, id: ItemId) {
    // `Light` is the item's direction byte.
    let light = w.item(id).map_or(-1, |i| i32::from(i.direction.bits()));
    if light == LightType::WestBig as i32 {
        ops::set_light(w, id, LightType::NorthBig);
    } else if light == LightType::NorthBig as i32 {
        ops::set_light(w, id, LightType::WestBig);
    }
}

fn flip_hanging_lantern(w: &mut World, id: ItemId, pairs: &[(i32, i32)]) {
    ops::set_light(w, id, LightType::Circle300);
    swap_ids(w, id, pairs);
}

pub(crate) fn configure(w: &mut World) {
    use muo_content::items::flip::register_own_flip;
    register_own_flip(
        w,
        "Server.Items.LeatherGloves",
        crate::armor::special::leather_gloves_flip,
    );
    register_own_flip(
        w,
        "Server.Items.LeafGloves",
        crate::armor::special::leaf_gloves_flip,
    );
    register_own_flip(w, "Server.Items.Futon", |w, id| {
        swap_ids(
            w,
            id,
            &[
                (0x295C, 0x295D),
                (0x295E, 0x295F),
                (0x295D, 0x295C),
                (0x295F, 0x295E),
            ],
        );
    });
    register_own_flip(w, "Server.Items.ParagonChest", |w, id| {
        swap_ids(
            w,
            id,
            &[
                (0x9AB, 0xE7C),
                (0xE7C, 0x9AB),
                (0xE40, 0xE41),
                (0xE41, 0xE40),
            ],
        );
    });
    register_own_flip(w, "Server.Items.RedHangingLantern", |w, id| {
        flip_hanging_lantern(
            w,
            id,
            &[
                (0x24C2, 0x24C4),
                (0x24C1, 0x24C3),
                (0x24C4, 0x24C2),
                (0x24C3, 0x24C1),
            ],
        );
    });
    register_own_flip(w, "Server.Items.WhiteHangingLantern", |w, id| {
        flip_hanging_lantern(
            w,
            id,
            &[
                (0x24C6, 0x24C8),
                (0x24C5, 0x24C7),
                (0x24C8, 0x24C6),
                (0x24C7, 0x24C5),
            ],
        );
    });
    register_own_flip(w, "Server.Items.WallSconce", |w, id| {
        flip_big_light(w, id);
        swap_ids(
            w,
            id,
            &[
                (0x9FB, 0xA00),
                (0x9FD, 0xA02),
                (0xA00, 0x9FB),
                (0xA02, 0x9FD),
            ],
        );
    });
    register_own_flip(w, "Server.Items.WallTorch", |w, id| {
        flip_big_light(w, id);
        swap_ids(
            w,
            id,
            &[
                (0xA05, 0xA0A),
                (0xA07, 0xA0C),
                (0xA0A, 0xA05),
                (0xA0C, 0xA07),
            ],
        );
    });
}

pub fn register(types: &mut TypeRegistry) {
    types.register_phase_default(
        Phase::Configure,
        "Server.Items.FlippableAttribute.Configure",
        configure,
    );
}
