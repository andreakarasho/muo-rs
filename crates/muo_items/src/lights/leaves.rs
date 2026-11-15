//! The Lights classes that are plain tables (a constructor and constant overrides); the classes with logic
//! are hand-written next to this file.

use crate::lights::{
    BASE_EQUIPABLE_LIGHT_CLASS, BASE_LIGHT_CLASS, BURNOUT, LightClass, base_equipable_light_new,
    base_light_new, set_burning, set_duration,
};
use crate::simple::leaf;
use muo_primitives::TimeSpan;
use muo_server::World;
use muo_server::ids::ItemId;
use muo_server::item::{ITEM_BASE, ItemType, ops};
use muo_server::light_type::LightType;
use muo_server::type_registry::TypeRegistry;

use crate::lights::BASE_EQUIPABLE_LIGHT;
use crate::lights::BASE_LIGHT;

/// `Brazier()`.
pub fn brazier_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = base_light_new(w, ty, 0xE31);
    ops::set_movable(w, id, false);
    set_duration(w, id, TimeSpan::ZERO);
    set_burning(w, id, true);
    ops::set_light(w, id, LightType::Circle225);
    id
}

static BRAZIER_CLASS: LightClass = LightClass {
    lit_item_id: |_, _| 0xE31,
    ..BASE_LIGHT_CLASS
};
leaf! {
    /// `Server.Items.Brazier`.
    BRAZIER = "Server.Items.Brazier" : BASE_LIGHT, v0;
    construct: Some(|w| brazier_new(w, &BRAZIER)),
    family: Some(&BRAZIER_CLASS),
    default_weight: |_, _| 20.0,
}

/// `BrazierTall()`.
pub fn brazier_tall_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = base_light_new(w, ty, 0x19AA);
    ops::set_movable(w, id, false);
    set_duration(w, id, TimeSpan::ZERO);
    set_burning(w, id, true);
    ops::set_light(w, id, LightType::Circle300);
    id
}

static BRAZIER_TALL_CLASS: LightClass = LightClass {
    lit_item_id: |_, _| 0x19AA,
    ..BASE_LIGHT_CLASS
};
leaf! {
    /// `Server.Items.BrazierTall`.
    BRAZIER_TALL = "Server.Items.BrazierTall" : BASE_LIGHT, v0;
    construct: Some(|w| brazier_tall_new(w, &BRAZIER_TALL)),
    family: Some(&BRAZIER_TALL_CLASS),
    default_weight: |_, _| 25.0,
}

/// `CandelabraStand()`.
pub fn candelabra_stand_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = base_light_new(w, ty, 0xA29);
    set_duration(w, id, TimeSpan::ZERO);
    set_burning(w, id, false);
    ops::set_light(w, id, LightType::Circle225);
    id
}

static CANDELABRA_STAND_CLASS: LightClass = LightClass {
    lit_item_id: |_, _| 0xB26,
    unlit_item_id: |_, _| 0xA29,
    ..BASE_LIGHT_CLASS
};
leaf! {
    /// `Server.Items.CandelabraStand`.
    CANDELABRA_STAND = "Server.Items.CandelabraStand" : BASE_LIGHT, v0;
    construct: Some(|w| candelabra_stand_new(w, &CANDELABRA_STAND)),
    family: Some(&CANDELABRA_STAND_CLASS),
    default_weight: |_, _| 20.0,
}

/// `Candle()`.
pub fn candle_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = base_equipable_light_new(w, ty, 0xA28);
    set_duration(
        w,
        id,
        if BURNOUT {
            TimeSpan::from_mins(20)
        } else {
            TimeSpan::ZERO
        },
    );
    set_burning(w, id, false);
    ops::set_stackable(w, id, true);
    ops::set_light(w, id, LightType::Circle150);
    id
}

static CANDLE_CLASS: LightClass = LightClass {
    lit_item_id: |_, _| 0xA0F,
    unlit_item_id: |_, _| 0xA28,
    ..BASE_EQUIPABLE_LIGHT_CLASS
};
leaf! {
    /// `Server.Items.Candle`.
    CANDLE = "Server.Items.Candle" : BASE_EQUIPABLE_LIGHT, v0;
    construct: Some(|w| candle_new(w, &CANDLE)),
    family: Some(&CANDLE_CLASS),
    default_weight: |_, _| 1.0,
}

/// `CandleLarge()`.
pub fn candle_large_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = base_light_new(w, ty, 0xA26);
    set_duration(
        w,
        id,
        if BURNOUT {
            TimeSpan::from_mins(25)
        } else {
            TimeSpan::ZERO
        },
    );
    set_burning(w, id, false);
    ops::set_light(w, id, LightType::Circle150);
    id
}

static CANDLE_LARGE_CLASS: LightClass = LightClass {
    lit_item_id: |_, _| 0xB1A,
    unlit_item_id: |_, _| 0xA26,
    ..BASE_LIGHT_CLASS
};
leaf! {
    /// `Server.Items.CandleLarge`.
    CANDLE_LARGE = "Server.Items.CandleLarge" : BASE_LIGHT, v0;
    construct: Some(|w| candle_large_new(w, &CANDLE_LARGE)),
    family: Some(&CANDLE_LARGE_CLASS),
    default_weight: |_, _| 2.0,
}

/// `CandleLong()`.
pub fn candle_long_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = base_light_new(w, ty, 0x1433);
    set_duration(
        w,
        id,
        if BURNOUT {
            TimeSpan::from_mins(30)
        } else {
            TimeSpan::ZERO
        },
    );
    set_burning(w, id, false);
    ops::set_light(w, id, LightType::Circle150);
    id
}

pub static CANDLE_LONG_CLASS: LightClass = LightClass {
    lit_item_id: |_, _| 0x1430,
    unlit_item_id: |_, _| 0x1433,
    ..BASE_LIGHT_CLASS
};
leaf! {
    /// `Server.Items.CandleLong`.
    CANDLE_LONG = "Server.Items.CandleLong" : BASE_LIGHT, v0;
    construct: Some(|w| candle_long_new(w, &CANDLE_LONG)),
    family: Some(&CANDLE_LONG_CLASS),
    default_weight: |_, _| 1.0,
}

/// `CandleShort()`.
pub fn candle_short_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = base_light_new(w, ty, 0x142F);
    set_duration(
        w,
        id,
        if BURNOUT {
            TimeSpan::from_mins(25)
        } else {
            TimeSpan::ZERO
        },
    );
    set_burning(w, id, false);
    ops::set_light(w, id, LightType::Circle150);
    id
}

static CANDLE_SHORT_CLASS: LightClass = LightClass {
    lit_item_id: |_, _| 0x142C,
    unlit_item_id: |_, _| 0x142F,
    ..BASE_LIGHT_CLASS
};
leaf! {
    /// `Server.Items.CandleShort`.
    CANDLE_SHORT = "Server.Items.CandleShort" : BASE_LIGHT, v0;
    construct: Some(|w| candle_short_new(w, &CANDLE_SHORT)),
    family: Some(&CANDLE_SHORT_CLASS),
    default_weight: |_, _| 1.0,
}

/// `CandleSkull()`.
pub fn candle_skull_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = base_light_new(w, ty, 0x1853);
    set_duration(
        w,
        id,
        if BURNOUT {
            TimeSpan::from_mins(25)
        } else {
            TimeSpan::ZERO
        },
    );
    set_burning(w, id, false);
    ops::set_light(w, id, LightType::Circle150);
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
    default_weight: |_, _| 5.0,
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
    let id = base_light_new(w, ty, 0xA1D);
    ops::set_movable(w, id, false);
    set_duration(w, id, TimeSpan::ZERO);
    set_burning(w, id, false);
    ops::set_light(w, id, LightType::Circle300);
    id
}

static HANGING_LANTERN_CLASS: LightClass = LightClass {
    lit_item_id: |_, _| 0xA1A,
    unlit_item_id: |_, _| 0xA1D,
    ..BASE_LIGHT_CLASS
};
leaf! {
    /// `Server.Items.HangingLantern`.
    HANGING_LANTERN = "Server.Items.HangingLantern" : BASE_LIGHT, v0;
    construct: Some(|w| hanging_lantern_new(w, &HANGING_LANTERN)),
    family: Some(&HANGING_LANTERN_CLASS),
    default_weight: |_, _| 40.0,
}

/// `LampPost1()`.
pub fn lamp_post1_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = base_light_new(w, ty, 0xB21);
    ops::set_movable(w, id, false);
    set_duration(w, id, TimeSpan::ZERO);
    set_burning(w, id, false);
    ops::set_light(w, id, LightType::Circle300);
    id
}

static LAMP_POST1_CLASS: LightClass = LightClass {
    lit_item_id: |_, _| 0xB20,
    unlit_item_id: |_, _| 0xB21,
    ..BASE_LIGHT_CLASS
};
leaf! {
    /// `Server.Items.LampPost1`.
    LAMP_POST1 = "Server.Items.LampPost1" : BASE_LIGHT, v0;
    construct: Some(|w| lamp_post1_new(w, &LAMP_POST1)),
    family: Some(&LAMP_POST1_CLASS),
    default_weight: |_, _| 40.0,
}

/// `LampPost2()`.
pub fn lamp_post2_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = base_light_new(w, ty, 0xB23);
    ops::set_movable(w, id, false);
    set_duration(w, id, TimeSpan::ZERO);
    set_burning(w, id, false);
    ops::set_light(w, id, LightType::Circle300);
    id
}

static LAMP_POST2_CLASS: LightClass = LightClass {
    lit_item_id: |_, _| 0xB22,
    unlit_item_id: |_, _| 0xB23,
    ..BASE_LIGHT_CLASS
};
leaf! {
    /// `Server.Items.LampPost2`.
    LAMP_POST2 = "Server.Items.LampPost2" : BASE_LIGHT, v0;
    construct: Some(|w| lamp_post2_new(w, &LAMP_POST2)),
    family: Some(&LAMP_POST2_CLASS),
    default_weight: |_, _| 40.0,
}

/// `LampPost3()`.
pub fn lamp_post3_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = base_light_new(w, ty, 0xb25);
    ops::set_movable(w, id, false);
    set_duration(w, id, TimeSpan::ZERO);
    set_burning(w, id, false);
    ops::set_light(w, id, LightType::Circle300);
    id
}

static LAMP_POST3_CLASS: LightClass = LightClass {
    lit_item_id: |_, _| 0xB24,
    unlit_item_id: |_, _| 0xB25,
    ..BASE_LIGHT_CLASS
};
leaf! {
    /// `Server.Items.LampPost3`.
    LAMP_POST3 = "Server.Items.LampPost3" : BASE_LIGHT, v0;
    construct: Some(|w| lamp_post3_new(w, &LAMP_POST3)),
    family: Some(&LAMP_POST3_CLASS),
    default_weight: |_, _| 40.0,
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
    let id = base_light_new(w, ty, 0x24BE);
    ops::set_movable(w, id, true);
    set_duration(w, id, TimeSpan::ZERO);
    set_burning(w, id, false);
    ops::set_light(w, id, LightType::Circle150);
    id
}

static PAPER_LANTERN_CLASS: LightClass = LightClass {
    lit_item_id: |_, _| 0x24BD,
    unlit_item_id: |_, _| 0x24BE,
    ..BASE_LIGHT_CLASS
};
leaf! {
    /// `Server.Items.PaperLantern`.
    PAPER_LANTERN = "Server.Items.PaperLantern" : BASE_LIGHT, v0;
    construct: Some(|w| paper_lantern_new(w, &PAPER_LANTERN)),
    family: Some(&PAPER_LANTERN_CLASS),
    default_weight: |_, _| 3.0,
}

/// `RoundPaperLantern()`.
pub fn round_paper_lantern_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = base_light_new(w, ty, 0x24CA);
    ops::set_movable(w, id, true);
    set_duration(w, id, TimeSpan::ZERO);
    set_burning(w, id, false);
    ops::set_light(w, id, LightType::Circle150);
    id
}

static ROUND_PAPER_LANTERN_CLASS: LightClass = LightClass {
    lit_item_id: |_, _| 0x24C9,
    unlit_item_id: |_, _| 0x24CA,
    ..BASE_LIGHT_CLASS
};
leaf! {
    /// `Server.Items.RoundPaperLantern`.
    ROUND_PAPER_LANTERN = "Server.Items.RoundPaperLantern" : BASE_LIGHT, v0;
    construct: Some(|w| round_paper_lantern_new(w, &ROUND_PAPER_LANTERN)),
    family: Some(&ROUND_PAPER_LANTERN_CLASS),
    default_weight: |_, _| 3.0,
}

/// `ShojiLantern()`.
pub fn shoji_lantern_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = base_light_new(w, ty, 0x24BC);
    ops::set_movable(w, id, true);
    set_duration(w, id, TimeSpan::ZERO);
    set_burning(w, id, false);
    ops::set_light(w, id, LightType::Circle150);
    id
}

static SHOJI_LANTERN_CLASS: LightClass = LightClass {
    lit_item_id: |_, _| 0x24BB,
    unlit_item_id: |_, _| 0x24BC,
    ..BASE_LIGHT_CLASS
};
leaf! {
    /// `Server.Items.ShojiLantern`.
    SHOJI_LANTERN = "Server.Items.ShojiLantern" : BASE_LIGHT, v0;
    construct: Some(|w| shoji_lantern_new(w, &SHOJI_LANTERN)),
    family: Some(&SHOJI_LANTERN_CLASS),
    default_weight: |_, _| 3.0,
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
