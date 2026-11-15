//! `Candelabra`, `HeatingStand`, `RedHangingLantern`, `WallSconce`, `WallTorch` and `WhiteHangingLantern`.
//! The `Flip()` of the last four is in `own_flips.rs`.

use muo_primitives::TimeSpan;
use muo_server::World;
use muo_server::ids::ItemId;
use muo_server::item::{ItemType, ops};
use muo_server::light_type::LightType;
use muo_server::type_registry::TypeRegistry;

use crate::lights::{
    BASE_LIGHT, BASE_LIGHT_CLASS, LightClass, base_light_new, douse_base, set_burning, set_duration,
};
use crate::simple::leaf;

fn cur_id(w: &World, id: ItemId) -> i32 {
    w.item(id).map_or(0, |i| i.item_id)
}

// ---- Candelabra ----

/// `Candelabra._isShipwreckedItem` (`[SerializableField(0)]`).
#[derive(Clone, Copy, Debug, Default)]
pub struct Shipwrecked(pub bool);

fn shipwrecked(w: &World, id: ItemId) -> bool {
    w.item(id)
        .and_then(|i| i.ext.get::<Shipwrecked>())
        .is_some_and(|s| s.0)
}

/// `IsShipwreckedItem = value`.
pub fn set_shipwrecked(w: &mut World, id: ItemId, value: bool) {
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(Shipwrecked(value));
    }
}

/// `Candelabra()`: `base(0xA27)`.
pub fn candelabra_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = base_light_new(w, ty, 0);
    crate::simple::own_template(w, id, "Server.Items.Candelabra");
    set_shipwrecked(w, id, false);
    set_duration(w, id, TimeSpan::ZERO); // Never burnt out
    set_burning(w, id, false);
    ops::set_light(w, id, LightType::Circle225);
    id
}

static CANDELABRA_CLASS: LightClass = LightClass {
    lit_item_id: crate::class_props::lit_item_id,
    unlit_item_id: crate::class_props::unlit_item_id,
    ..BASE_LIGHT_CLASS
};

/// `Server.Items.Candelabra`.
pub static CANDELABRA: ItemType = ItemType {
    full_name: "Server.Items.Candelabra",
    parent: Some(&BASE_LIGHT),
    construct: Some(|w| candelabra_new(w, &CANDELABRA)),
    family: Some(&CANDELABRA_CLASS),
    default_weight: crate::class_props::default_weight,
    add_name_properties: |w, id, list| {
        (BASE_LIGHT.add_name_properties)(w, id, list);
        if shipwrecked(w, id) {
            list.add(1041645); // recovered from a shipwreck
        }
    },
    on_single_click: |w, id, from| {
        (BASE_LIGHT.on_single_click)(w, id, from);
        if shipwrecked(w, id) {
            ops::label_to_number(w, id, from, 1041645, ""); // recovered from a shipwreck
        }
    },
    serialize: |w, id, out| {
        (BASE_LIGHT.serialize)(w, id, out);
        out.write_i32(0);
        out.write_bool(shipwrecked(w, id));
    },
    deserialize: |w, id, r| {
        (BASE_LIGHT.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        let value = r.read_bool()?;
        set_shipwrecked(w, id, value);
        Ok(())
    },
    ..BASE_LIGHT
};

// ---- HeatingStand ----

/// `HeatingStand()`: `base(0x1849)`.
pub fn heating_stand_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = base_light_new(w, ty, 0);
    crate::simple::own_template(w, id, "Server.Items.HeatingStand");
    id
}

/// The tail both `Ignite()` and `Douse()` run after the base call.
fn heating_stand_light(w: &mut World, id: ItemId) {
    let cur = cur_id(w, id);
    if cur == 0x184A {
        ops::set_light(w, id, LightType::Circle150);
    } else if cur == 0x1849 {
        ops::set_light(w, id, LightType::Empty);
    }
}

static HEATING_STAND_CLASS: LightClass = LightClass {
    lit_item_id: crate::class_props::lit_item_id,
    unlit_item_id: crate::class_props::unlit_item_id,
    ignite: |w, id| {
        (BASE_LIGHT_CLASS.ignite)(w, id);
        heating_stand_light(w, id);
    },
    douse: |w, id| {
        douse_base(w, id);
        heating_stand_light(w, id);
    },
    ..BASE_LIGHT_CLASS
};

leaf! {
    /// `Server.Items.HeatingStand`.
    HEATING_STAND = "Server.Items.HeatingStand" : BASE_LIGHT, v0;
    construct: Some(|w| heating_stand_new(w, &HEATING_STAND)),
    family: Some(&HEATING_STAND_CLASS),
    default_weight: crate::class_props::default_weight,
}

// ---- the flippable lights ----

/// One light that never burns out; `movable` and `light` are what the constructor sets.
macro_rules! flip_light {
    ($(#[$m:meta])* $name:ident, $class:ident, $new:ident = $full:literal,
     lit $lit:expr, unlit $unlit:expr;) => {
        static $class: LightClass = LightClass {
            lit_item_id: |w, id| { let cur = cur_id(w, id); $lit(cur) },
            unlit_item_id: |w, id| { let cur = cur_id(w, id); $unlit(cur) },
            ..BASE_LIGHT_CLASS
        };

        /// The constructor.
        pub fn $new(w: &mut World, ty: &'static ItemType) -> ItemId {
            // graphic, Movable, Duration (never burnt out), Burning and Light are the template
            let id = base_light_new(w, ty, 0);
            crate::simple::own_template(w, id, $full)
        }

        leaf! {
            $(#[$m])*
            $name = $full : BASE_LIGHT, v0;
            construct: Some(|w| $new(w, &$name)),
            family: Some(&$class),
            default_weight: crate::class_props::default_weight,
        }
    };
}

flip_light! {
    /// `Server.Items.RedHangingLantern`.
    RED_HANGING_LANTERN, RED_HANGING_LANTERN_CLASS, red_hanging_lantern_new = "Server.Items.RedHangingLantern",
    lit |c| if c == 0x24C2 { 0x24C1 } else { 0x24C3 },
    unlit |c| if c == 0x24C1 { 0x24C2 } else { 0x24C4 };
}

flip_light! {
    /// `Server.Items.WhiteHangingLantern`.
    WHITE_HANGING_LANTERN, WHITE_HANGING_LANTERN_CLASS, white_hanging_lantern_new = "Server.Items.WhiteHangingLantern",
    lit |c| if c == 0x24C6 { 0x24C5 } else { 0x24C7 },
    unlit |c| if c == 0x24C5 { 0x24C6 } else { 0x24C8 };
}

flip_light! {
    /// `Server.Items.WallSconce`.
    WALL_SCONCE, WALL_SCONCE_CLASS, wall_sconce_new = "Server.Items.WallSconce",
    lit |c| if c == 0x9FB { 0x9FD } else { 0xA02 },
    unlit |c| if c == 0x9FD { 0x9FB } else { 0xA00 };
}

flip_light! {
    /// `Server.Items.WallTorch`.
    WALL_TORCH, WALL_TORCH_CLASS, wall_torch_new = "Server.Items.WallTorch",
    lit |c| if c == 0xA05 { 0xA07 } else { 0xA0C },
    unlit |c| if c == 0xA07 { 0xA05 } else { 0xA0A };
}

/// Registers the classes of this file.
pub fn register(types: &mut TypeRegistry) {
    for t in [
        &CANDELABRA,
        &HEATING_STAND,
        &RED_HANGING_LANTERN,
        &WHITE_HANGING_LANTERN,
        &WALL_SCONCE,
        &WALL_TORCH,
    ] {
        types.register_item(t, &[]);
    }
}
