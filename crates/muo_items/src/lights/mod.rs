//! Lights. `base.rs` is `BaseLight` and `BaseEquipableLight`; `leaves.rs` holds the plain table classes; `Torch` and `Lantern` (which override `Ignite`, `OnAdded` and compute
//! their graphics from the current `ItemID`) are written here.

use muo_primitives::TimeSpan;
use muo_server::World;
use muo_server::ids::ItemId;
use muo_server::item::{ItemType, ops};
use muo_server::light_type::LightType;
use muo_server::type_registry::TypeRegistry;

use crate::simple::{hue, leaf};

pub mod base;
pub mod leaves;

pub use base::*;

// ---- Torch ----

/// `Torch()`: `base(0xF6B)`.
pub fn torch_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = base_equipable_light_new(w, ty, 0);
    crate::simple::own_template(w, id, "Server.Items.Torch");
    id
}

static TORCH_CLASS: LightClass = LightClass {
    lit_item_id: crate::class_props::lit_item_id,
    unlit_item_id: crate::class_props::unlit_item_id,
    lit_sound: 0x54,
    unlit_sound: 0x4BB,
    ignite: torch_ignite,
    ..BASE_EQUIPABLE_LIGHT_CLASS
};

leaf! {
    /// `Server.Items.Torch`.
    TORCH = "Server.Items.Torch" : BASE_EQUIPABLE_LIGHT, v0;
    construct: Some(|w| torch_new(w, &TORCH)),
    family: Some(&TORCH_CLASS),
    default_weight: crate::class_props::default_weight,
    on_added: torch_on_added,
}

// ---- Lantern ----

/// `Lantern.FullDuration`.
pub const LANTERN_FULL_DURATION: TimeSpan = TimeSpan::from_mins(20);

/// `Lantern()`: `base(0xA25)`.
pub fn lantern_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = base_equipable_light_new(w, ty, 0);
    crate::simple::own_template(w, id, "Server.Items.Lantern");
    set_duration(
        w,
        id,
        if BURNOUT {
            LANTERN_FULL_DURATION
        } else {
            TimeSpan::ZERO
        },
    );
    set_burning(w, id, false);
    ops::set_light(w, id, LightType::Circle300);
    id
}

fn cur_id(w: &World, id: ItemId) -> i32 {
    w.item(id).map_or(0, |i| i.item_id)
}

static LANTERN_CLASS: LightClass = LightClass {
    // ItemID is 0xA15 or 0xA17 ? ItemID : 0xA22
    lit_item_id: |w, id| {
        let cur = cur_id(w, id);
        if cur == 0xA15 || cur == 0xA17 {
            cur
        } else {
            0xA22
        }
    },
    // ItemID == 0xA18 ? ItemID : 0xA25
    unlit_item_id: |w, id| {
        let cur = cur_id(w, id);
        if cur == 0xA18 { cur } else { 0xA25 }
    },
    ..BASE_EQUIPABLE_LIGHT_CLASS
};

leaf! {
    /// `Server.Items.Lantern`.
    LANTERN = "Server.Items.Lantern" : BASE_EQUIPABLE_LIGHT, v0;
    construct: Some(|w| lantern_new(w, &LANTERN)),
    family: Some(&LANTERN_CLASS),
    default_weight: crate::class_props::default_weight,
}

/// `LanternOfSouls()`: `Hue = 0x482`.
pub fn lantern_of_souls_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = lantern_new(w, ty);
    hue(w, id, 0x482);
    id
}

leaf! {
    /// `Server.Items.LanternOfSouls` `[SerializationGenerator(0)]`.
    LANTERN_OF_SOULS = "Server.Items.LanternOfSouls" : LANTERN, enc;
    construct: Some(|w| lantern_of_souls_new(w, &LANTERN_OF_SOULS)),
    label_number: crate::class_props::label_number,
}

/// Registers the classes of this folder.
pub fn register(types: &mut TypeRegistry) {
    leaves::register(types);
    for t in [&TORCH, &LANTERN, &LANTERN_OF_SOULS] {
        types.register_item(t, &[]);
    }
}
