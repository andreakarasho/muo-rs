//! The ML quest items. `items_gen.rs` holds the simple ones; those with their own behavior are written below.

use muo_content::items::cont::BACKPACK;
use muo_primitives::MessageType;
use muo_server::World;
use muo_server::ids::{ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ops};
use muo_server::map::MapId;
use muo_server::mobile::part2::messages as mm;
use muo_server::type_registry::TypeRegistry;

use super::engine;
use super::item_support::leaf;
use super::model::flag;

// ---- BedlamTeleporter ------------------------------------------------------------------------------------

/// `BedlamTeleporter.OnDoubleClick`.
fn bedlam_teleporter_double_click(w: &mut World, id: ItemId, from: MobileId) {
    let dest = muo_primitives::Point3D::new(120, 1682, 0);
    let loc = ops::get_world_location(w, id);

    let in_range = w
        .mobile(from)
        .is_some_and(|m| (m.ty.in_range_point3d)(w, from, loc, 2));
    if !in_range {
        mm::local_overhead_message_number(w, from, MessageType::REGULAR, 0x3B2, 1019045, ""); // I can't reach that.
        return;
    }

    let access = muo_content::player_mobile::data(w, from).is_some()
        && engine::get_context(w, from).is_some_and(|c| c.get_flag(flag::BEDLAM_ACCESS));
    if access {
        muo_content::creature::pets::teleport_pets(w, from, dest, MapId::MALAS, false);
        muo_server::mobile::base::move_to_world(w, from, dest, MapId::MALAS);
    } else {
        // You press and push on the iron maiden, but nothing happens.
        mm::send_localized_message(w, from, 1074276, "", 0x3B2);
    }
}

leaf! {
    /// `Server.Engines.MLQuests.Items.BedlamTeleporter`.
    BEDLAM_TELEPORTER = "Server.Engines.MLQuests.Items.BedlamTeleporter" : ITEM_BASE, v0;
    construct: None,
    make: Some(|w| super::item_support::template_new(w, &BEDLAM_TELEPORTER)),
    label_number: |w, id| crate::template_keys::item_label(w, id, ITEM_BASE.label_number),
    on_double_click: bedlam_teleporter_double_click,
}

// ---- NibbetSatchel ---------------------------------------------------------------------------------------

leaf! {
    /// `Server.Items.NibbetSatchel`.
    NIBBET_SATCHEL = "Server.Items.NibbetSatchel" : BACKPACK, v0;
    construct: Some(|w| super::item_support::template_container(w, &NIBBET_SATCHEL)),
}

/// Registers the item classes.
pub fn register(types: &mut TypeRegistry) {
    super::items_gen::register(types);
    super::items2::register(types);
    super::transient::register(types);
    super::race_change::register(types);
    types.register_item(&BEDLAM_TELEPORTER, &[]);
    types.register_item(&NIBBET_SATCHEL, &[]);
}
