//! The Lights classes that are plain tables (a constructor and constant overrides); the classes with logic
//! are hand-written next to this file.

use crate::lights::{
    BASE_EQUIPABLE_LIGHT_CLASS, BASE_LIGHT_CLASS, LightClass, base_equipable_light_new,
    base_light_new,
};
use crate::simple::leaf;
use muo_server::World;
use muo_server::ids::ItemId;
use muo_server::item::{ITEM_BASE, ItemType};
use muo_server::type_registry::TypeRegistry;

use crate::lights::BASE_EQUIPABLE_LIGHT;
use crate::lights::BASE_LIGHT;

/// `Brazier()`.
pub fn brazier_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = base_light_new(w, ty, 0);
    crate::simple::own_template(w, id, "Server.Items.Brazier");
    id
}

static BRAZIER_CLASS: LightClass = LightClass {
    lit_item_id: crate::class_props::lit_item_id,
    ..BASE_LIGHT_CLASS
};
leaf! {
    /// `Server.Items.Brazier`.
    BRAZIER = "Server.Items.Brazier" : BASE_LIGHT, v0;
    construct: Some(|w| brazier_new(w, &BRAZIER)),
    family: Some(&BRAZIER_CLASS),
    default_weight: crate::class_props::default_weight,
}

/// `BrazierTall()`.
pub fn brazier_tall_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = base_light_new(w, ty, 0);
    crate::simple::own_template(w, id, "Server.Items.BrazierTall");
    id
}

static BRAZIER_TALL_CLASS: LightClass = LightClass {
    lit_item_id: crate::class_props::lit_item_id,
    ..BASE_LIGHT_CLASS
};
leaf! {
    /// `Server.Items.BrazierTall`.
    BRAZIER_TALL = "Server.Items.BrazierTall" : BASE_LIGHT, v0;
    construct: Some(|w| brazier_tall_new(w, &BRAZIER_TALL)),
    family: Some(&BRAZIER_TALL_CLASS),
    default_weight: crate::class_props::default_weight,
}

/// `CandelabraStand()`.
pub fn candelabra_stand_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = base_light_new(w, ty, 0);
    crate::simple::own_template(w, id, "Server.Items.CandelabraStand");
    id
}

static CANDELABRA_STAND_CLASS: LightClass = LightClass {
    lit_item_id: crate::class_props::lit_item_id,
    unlit_item_id: crate::class_props::unlit_item_id,
    ..BASE_LIGHT_CLASS
};
leaf! {
    /// `Server.Items.CandelabraStand`.
    CANDELABRA_STAND = "Server.Items.CandelabraStand" : BASE_LIGHT, v0;
    construct: Some(|w| candelabra_stand_new(w, &CANDELABRA_STAND)),
    family: Some(&CANDELABRA_STAND_CLASS),
    default_weight: crate::class_props::default_weight,
}

/// `Candle()`.
pub fn candle_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = base_equipable_light_new(w, ty, 0);
    crate::simple::own_template(w, id, "Server.Items.Candle");
    id
}

static CANDLE_CLASS: LightClass = LightClass {
    lit_item_id: crate::class_props::lit_item_id,
    unlit_item_id: crate::class_props::unlit_item_id,
    ..BASE_EQUIPABLE_LIGHT_CLASS
};
leaf! {
    /// `Server.Items.Candle`.
    CANDLE = "Server.Items.Candle" : BASE_EQUIPABLE_LIGHT, v0;
    construct: Some(|w| candle_new(w, &CANDLE)),
    family: Some(&CANDLE_CLASS),
    default_weight: crate::class_props::default_weight,
}

/// `CandleLarge()`.
pub fn candle_large_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = base_light_new(w, ty, 0);
    crate::simple::own_template(w, id, "Server.Items.CandleLarge");
    id
}

static CANDLE_LARGE_CLASS: LightClass = LightClass {
    lit_item_id: crate::class_props::lit_item_id,
    unlit_item_id: crate::class_props::unlit_item_id,
    ..BASE_LIGHT_CLASS
};
leaf! {
    /// `Server.Items.CandleLarge`.
    CANDLE_LARGE = "Server.Items.CandleLarge" : BASE_LIGHT, v0;
    construct: Some(|w| candle_large_new(w, &CANDLE_LARGE)),
    family: Some(&CANDLE_LARGE_CLASS),
    default_weight: crate::class_props::default_weight,
}

/// `CandleLong()`.
pub fn candle_long_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = base_light_new(w, ty, 0);
    crate::simple::own_template(w, id, "Server.Items.CandleLong");
    id
}

pub static CANDLE_LONG_CLASS: LightClass = LightClass {
    lit_item_id: crate::class_props::lit_item_id,
    unlit_item_id: crate::class_props::unlit_item_id,
    ..BASE_LIGHT_CLASS
};
leaf! {
    /// `Server.Items.CandleLong`.
    CANDLE_LONG = "Server.Items.CandleLong" : BASE_LIGHT, v0;
    construct: Some(|w| candle_long_new(w, &CANDLE_LONG)),
    family: Some(&CANDLE_LONG_CLASS),
    default_weight: crate::class_props::default_weight,
}

/// `CandleShort()`.
pub fn candle_short_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = base_light_new(w, ty, 0);
    crate::simple::own_template(w, id, "Server.Items.CandleShort");
    id
}

static CANDLE_SHORT_CLASS: LightClass = LightClass {
    lit_item_id: crate::class_props::lit_item_id,
    unlit_item_id: crate::class_props::unlit_item_id,
    ..BASE_LIGHT_CLASS
};
leaf! {
    /// `Server.Items.CandleShort`.
    CANDLE_SHORT = "Server.Items.CandleShort" : BASE_LIGHT, v0;
    construct: Some(|w| candle_short_new(w, &CANDLE_SHORT)),
    family: Some(&CANDLE_SHORT_CLASS),
    default_weight: crate::class_props::default_weight,
}

/// `CandleSkull()`.
pub fn candle_skull_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = base_light_new(w, ty, 0);
    crate::simple::own_template(w, id, "Server.Items.CandleSkull");
    id
}

static CANDLE_SKULL_CLASS: LightClass = LightClass {
    lit_item_id: |w, id| {
        let cur = w.item(id).map_or(0, |i| i.item_id);
        if cur == 0x1583 || cur == 0x1854 {
            0x1854
        } else {
            0x1858
        }
    },
    unlit_item_id: |w, id| {
        let cur = w.item(id).map_or(0, |i| i.item_id);
        if cur == 0x1853 || cur == 0x1584 {
            0x1853
        } else {
            0x1857
        }
    },
    ..BASE_LIGHT_CLASS
};
leaf! {
    /// `Server.Items.CandleSkull`.
    CANDLE_SKULL = "Server.Items.CandleSkull" : BASE_LIGHT, v0;
    construct: Some(|w| candle_skull_new(w, &CANDLE_SKULL)),
    family: Some(&CANDLE_SKULL_CLASS),
    default_weight: crate::class_props::default_weight,
}

/// `DarkSource()`.
pub fn dark_source_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.DarkSource")
}

leaf! {
    /// `Server.Items.DarkSource`.
    DARK_SOURCE = "Server.Items.DarkSource" : ITEM_BASE, v0;
    construct: Some(|w| dark_source_new(w, &DARK_SOURCE)),
}

/// `HangingLantern()`.
pub fn hanging_lantern_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = base_light_new(w, ty, 0);
    crate::simple::own_template(w, id, "Server.Items.HangingLantern");
    id
}

static HANGING_LANTERN_CLASS: LightClass = LightClass {
    lit_item_id: crate::class_props::lit_item_id,
    unlit_item_id: crate::class_props::unlit_item_id,
    ..BASE_LIGHT_CLASS
};
leaf! {
    /// `Server.Items.HangingLantern`.
    HANGING_LANTERN = "Server.Items.HangingLantern" : BASE_LIGHT, v0;
    construct: Some(|w| hanging_lantern_new(w, &HANGING_LANTERN)),
    family: Some(&HANGING_LANTERN_CLASS),
    default_weight: crate::class_props::default_weight,
}

/// `LampPost1()`.
pub fn lamp_post1_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = base_light_new(w, ty, 0);
    crate::simple::own_template(w, id, "Server.Items.LampPost1");
    id
}

static LAMP_POST1_CLASS: LightClass = LightClass {
    lit_item_id: crate::class_props::lit_item_id,
    unlit_item_id: crate::class_props::unlit_item_id,
    ..BASE_LIGHT_CLASS
};
leaf! {
    /// `Server.Items.LampPost1`.
    LAMP_POST1 = "Server.Items.LampPost1" : BASE_LIGHT, v0;
    construct: Some(|w| lamp_post1_new(w, &LAMP_POST1)),
    family: Some(&LAMP_POST1_CLASS),
    default_weight: crate::class_props::default_weight,
}

/// `LampPost2()`.
pub fn lamp_post2_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = base_light_new(w, ty, 0);
    crate::simple::own_template(w, id, "Server.Items.LampPost2");
    id
}

static LAMP_POST2_CLASS: LightClass = LightClass {
    lit_item_id: crate::class_props::lit_item_id,
    unlit_item_id: crate::class_props::unlit_item_id,
    ..BASE_LIGHT_CLASS
};
leaf! {
    /// `Server.Items.LampPost2`.
    LAMP_POST2 = "Server.Items.LampPost2" : BASE_LIGHT, v0;
    construct: Some(|w| lamp_post2_new(w, &LAMP_POST2)),
    family: Some(&LAMP_POST2_CLASS),
    default_weight: crate::class_props::default_weight,
}

/// `LampPost3()`.
pub fn lamp_post3_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = base_light_new(w, ty, 0);
    crate::simple::own_template(w, id, "Server.Items.LampPost3");
    id
}

static LAMP_POST3_CLASS: LightClass = LightClass {
    lit_item_id: crate::class_props::lit_item_id,
    unlit_item_id: crate::class_props::unlit_item_id,
    ..BASE_LIGHT_CLASS
};
leaf! {
    /// `Server.Items.LampPost3`.
    LAMP_POST3 = "Server.Items.LampPost3" : BASE_LIGHT, v0;
    construct: Some(|w| lamp_post3_new(w, &LAMP_POST3)),
    family: Some(&LAMP_POST3_CLASS),
    default_weight: crate::class_props::default_weight,
}

/// `LightSource()`.
pub fn light_source_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.LightSource")
}

leaf! {
    /// `Server.Items.LightSource`.
    LIGHT_SOURCE = "Server.Items.LightSource" : ITEM_BASE, v0;
    construct: Some(|w| light_source_new(w, &LIGHT_SOURCE)),
}

/// `PaperLantern()`.
pub fn paper_lantern_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = base_light_new(w, ty, 0);
    crate::simple::own_template(w, id, "Server.Items.PaperLantern");
    id
}

static PAPER_LANTERN_CLASS: LightClass = LightClass {
    lit_item_id: crate::class_props::lit_item_id,
    unlit_item_id: crate::class_props::unlit_item_id,
    ..BASE_LIGHT_CLASS
};
leaf! {
    /// `Server.Items.PaperLantern`.
    PAPER_LANTERN = "Server.Items.PaperLantern" : BASE_LIGHT, v0;
    construct: Some(|w| paper_lantern_new(w, &PAPER_LANTERN)),
    family: Some(&PAPER_LANTERN_CLASS),
    default_weight: crate::class_props::default_weight,
}

/// `RoundPaperLantern()`.
pub fn round_paper_lantern_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = base_light_new(w, ty, 0);
    crate::simple::own_template(w, id, "Server.Items.RoundPaperLantern");
    id
}

static ROUND_PAPER_LANTERN_CLASS: LightClass = LightClass {
    lit_item_id: crate::class_props::lit_item_id,
    unlit_item_id: crate::class_props::unlit_item_id,
    ..BASE_LIGHT_CLASS
};
leaf! {
    /// `Server.Items.RoundPaperLantern`.
    ROUND_PAPER_LANTERN = "Server.Items.RoundPaperLantern" : BASE_LIGHT, v0;
    construct: Some(|w| round_paper_lantern_new(w, &ROUND_PAPER_LANTERN)),
    family: Some(&ROUND_PAPER_LANTERN_CLASS),
    default_weight: crate::class_props::default_weight,
}

/// `ShojiLantern()`.
pub fn shoji_lantern_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = base_light_new(w, ty, 0);
    crate::simple::own_template(w, id, "Server.Items.ShojiLantern");
    id
}

static SHOJI_LANTERN_CLASS: LightClass = LightClass {
    lit_item_id: crate::class_props::lit_item_id,
    unlit_item_id: crate::class_props::unlit_item_id,
    ..BASE_LIGHT_CLASS
};
leaf! {
    /// `Server.Items.ShojiLantern`.
    SHOJI_LANTERN = "Server.Items.ShojiLantern" : BASE_LIGHT, v0;
    construct: Some(|w| shoji_lantern_new(w, &SHOJI_LANTERN)),
    family: Some(&SHOJI_LANTERN_CLASS),
    default_weight: crate::class_props::default_weight,
}

/// Every class of this file: the type and its `[TypeAlias]` names.
pub static ALL: &[(&ItemType, &[&str])] = &[
    (&BRAZIER, &[]),
    (&BRAZIER_TALL, &[]),
    (&CANDELABRA_STAND, &[]),
    (&CANDLE, &[]),
    (&CANDLE_LARGE, &[]),
    (&CANDLE_LONG, &[]),
    (&CANDLE_SHORT, &[]),
    (&CANDLE_SKULL, &[]),
    (&DARK_SOURCE, &[]),
    (&HANGING_LANTERN, &[]),
    (&LAMP_POST1, &[]),
    (&LAMP_POST2, &[]),
    (&LAMP_POST3, &[]),
    (&LIGHT_SOURCE, &[]),
    (&PAPER_LANTERN, &[]),
    (&ROUND_PAPER_LANTERN, &[]),
    (&SHOJI_LANTERN, &[]),
];

/// `[Flippable(a, b)]` of the classes (the flip command reads it).
pub static FLIPPABLE: &[(&ItemType, i32, i32)] = &[];

/// The interfaces a class lists after its base class (`is ICommodity` and the like); none has a consumer yet.
pub static IMPLEMENTS: &[(&ItemType, &str)] = &[];

/// `ICommodity` members of a class: (type, `IsDeedable`); `DescriptionNumber` is `LabelNumber`.
pub static COMMODITIES: &[(&ItemType, bool)] = &[];

/// Classes with `[Flippable]` or `[DynamicFlipping]` and no graphics pair (the flip is the class itself).
pub static OWN_FLIP: &[&ItemType] = &[&PAPER_LANTERN, &ROUND_PAPER_LANTERN, &SHOJI_LANTERN];

/// Classes with `[Furniture]`.
pub static FURNITURE: &[&ItemType] = &[];

/// Registers the classes of this file.
pub fn register(types: &mut TypeRegistry) {
    for (t, aliases) in ALL {
        types.register_item(t, aliases);
    }
}
