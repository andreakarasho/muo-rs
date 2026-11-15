//! The Heritage addon components with code of their own: `WallTorchComponent`. The addons and deeds that use them
//! are in `addons.rs`.
//!
//! The other addon families with timers or movement (Evil Home Decor, Heritage, Holiday) are in `decor.rs` and
//! `addons.rs`.

use muo_content::items::addon::ADDON_COMPONENT;
use muo_server::World;
use muo_server::ids::{ItemId, MobileId};
use muo_server::item::{ItemType, deps};
use muo_server::type_registry::TypeRegistry;

use crate::simple::leaf;

/// `WallTorchComponent.OnDoubleClick`.
fn wall_torch_on_double_click(w: &mut World, id: ItemId, from: MobileId) {
    let Some(it) = w.item(id) else { return };
    let (loc, map, item_id) = (it.location, it.map, it.item_id);
    if !deps::mobile_in_range(w, from, loc, 2) {
        deps::local_overhead_message(
            w,
            from,
            muo_server::packets::message::MessageType::REGULAR,
            0x3B2,
            1019045, // I can't reach that.
        );
        return;
    }
    let next = match item_id {
        0x3D98 => 0x3D9B,
        0x3D9B => 0x3D98,
        0x3D94 => 0x3D97,
        0x3D97 => 0x3D94,
        other => other,
    };
    if let Some(ty) = w.item(id).map(|i| i.ty) {
        (ty.set_item_id)(w, id, next);
    }
    muo_server::effects::play_sound(w, loc, map, 0x3BE);
}

leaf! {
    /// `Server.Items.WallTorchComponent` (`[Flippable(0x3D98, 0x3D94)]`).
    WALL_TORCH_COMPONENT = "Server.Items.WallTorchComponent" : ADDON_COMPONENT, enc;
    label_number: crate::class_props::label_number,
    on_double_click: wall_torch_on_double_click,
}

pub static ALL: &[(&ItemType, &[&str])] = &[(&WALL_TORCH_COMPONENT, &[])];

pub fn register(types: &mut TypeRegistry) {
    for (t, aliases) in ALL {
        types.register_item(t, aliases);
    }
}
