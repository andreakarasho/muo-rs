//! The ML quest items. `items_gen.rs` holds the simple ones; those with their own behavior are written below.

use muo_content::hooks::rnd;
use muo_content::items::cont::{BACKPACK, new_base_container};
use muo_primitives::{MessageType, utility};
use muo_server::World;
use muo_server::ids::{ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType, ops};
use muo_server::layer::Layer;
use muo_server::map::MapId;
use muo_server::mobile::part2::messages as mm;
use muo_server::type_registry::TypeRegistry;

use super::engine;
use super::item_support::{drop_new, hue, layer, leaf};
use super::model::flag;

// ---- BedlamTeleporter ------------------------------------------------------------------------------------

/// `BedlamTeleporter(): base(0x124D) => Movable = false`.
pub fn bedlam_teleporter_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = ops::new_item_with_id(w, ty, 0x124D);
    ops::set_movable(w, id, false);
    id
}

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
    make: Some(|w| bedlam_teleporter_new(w, &BEDLAM_TELEPORTER)),
    label_number: |_, _| 1074161, // Access to Bedlam by invitation only
    on_double_click: bedlam_teleporter_double_click,
}

// ---- NibbetSatchel ---------------------------------------------------------------------------------------

/// `NibbetSatchel()`.
pub fn nibbet_satchel_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = new_base_container(w, ty, 0xE75);
    layer(w, id, Layer::Backpack);
    let h = utility::random_bright_hue(&mut w.rng);
    hue(w, id, h);
    drop_new(w, id, "TinkerTools", None);
    let (name, amount) = match rnd::below(w, 10) {
        0 => ("Springs", Some(3)),
        1 => ("Axle", Some(3)),
        2 => ("Hinge", Some(3)),
        3 => ("Key", None),
        4 => ("Scissors", None),
        5 => ("BarrelTap", None),
        6 => ("BarrelHoops", None),
        7 => ("Gears", Some(3)),
        8 => ("Lockpick", Some(3)),
        _ => ("ClockFrame", Some(3)), // 9
    };
    drop_new(w, id, name, amount);
    id
}

leaf! {
    /// `Server.Items.NibbetSatchel`.
    NIBBET_SATCHEL = "Server.Items.NibbetSatchel" : BACKPACK, v0;
    construct: Some(|w| nibbet_satchel_new(w, &NIBBET_SATCHEL)),
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
