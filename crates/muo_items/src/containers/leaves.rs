//! The Containers classes that are plain tables (a constructor and constant overrides); the classes with logic
//! are hand-written next to this file.

use crate::simple::leaf;
use muo_content::items::cont::new_base_container;
use muo_server::World;
use muo_server::ids::ItemId;
use muo_server::item::ItemType;
use muo_server::type_registry::TypeRegistry;

use muo_content::items::cont::BASE_CONTAINER;

/// `CherryArmoire()`.
pub fn cherry_armoire_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = new_base_container(w, ty, 0);
    crate::simple::own_template(w, id, "Server.Items.CherryArmoire")
}

leaf! {
    /// `Server.Items.CherryArmoire`.
    CHERRY_ARMOIRE = "Server.Items.CherryArmoire" : BASE_CONTAINER, v0;
    construct: Some(|w| cherry_armoire_new(w, &CHERRY_ARMOIRE)),
    default_weight: crate::class_props::default_weight,
}

/// `Drawer()`.
pub fn drawer_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = new_base_container(w, ty, 0);
    crate::simple::own_template(w, id, "Server.Items.Drawer")
}

leaf! {
    /// `Server.Items.Drawer`.
    DRAWER = "Server.Items.Drawer" : BASE_CONTAINER, v0;
    construct: Some(|w| drawer_new(w, &DRAWER)),
    default_weight: crate::class_props::default_weight,
}

/// `ElegantArmoire()`.
pub fn elegant_armoire_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = new_base_container(w, ty, 0);
    crate::simple::own_template(w, id, "Server.Items.ElegantArmoire")
}

leaf! {
    /// `Server.Items.ElegantArmoire`.
    ELEGANT_ARMOIRE = "Server.Items.ElegantArmoire" : BASE_CONTAINER, v0;
    construct: Some(|w| elegant_armoire_new(w, &ELEGANT_ARMOIRE)),
    default_weight: crate::class_props::default_weight,
}

/// `EmptyBookcase()`.
pub fn empty_bookcase_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = new_base_container(w, ty, 0);
    crate::simple::own_template(w, id, "Server.Items.EmptyBookcase")
}

leaf! {
    /// `Server.Items.EmptyBookcase`.
    EMPTY_BOOKCASE = "Server.Items.EmptyBookcase" : BASE_CONTAINER, v0;
    construct: Some(|w| empty_bookcase_new(w, &EMPTY_BOOKCASE)),
}

/// `FancyDrawer()`.
pub fn fancy_drawer_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = new_base_container(w, ty, 0);
    crate::simple::own_template(w, id, "Server.Items.FancyDrawer")
}

leaf! {
    /// `Server.Items.FancyDrawer`.
    FANCY_DRAWER = "Server.Items.FancyDrawer" : BASE_CONTAINER, v0;
    construct: Some(|w| fancy_drawer_new(w, &FANCY_DRAWER)),
    default_weight: crate::class_props::default_weight,
}

/// `FancyElvenArmoire()`.
pub fn fancy_elven_armoire_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = new_base_container(w, ty, 0);
    crate::simple::own_template(w, id, "Server.Items.FancyElvenArmoire")
}

leaf! {
    /// `Server.Items.FancyElvenArmoire`.
    FANCY_ELVEN_ARMOIRE = "Server.Items.FancyElvenArmoire" : BASE_CONTAINER, enc;
    construct: Some(|w| fancy_elven_armoire_new(w, &FANCY_ELVEN_ARMOIRE)),
    default_weight: crate::class_props::default_weight,
    default_gump_id: crate::class_props::default_gump_id,
    default_drop_sound: crate::class_props::default_drop_sound,
}

/// `FullBookcase()`.
pub fn full_bookcase_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = new_base_container(w, ty, 0);
    crate::simple::own_template(w, id, "Server.Items.FullBookcase")
}

leaf! {
    /// `Server.Items.FullBookcase`.
    FULL_BOOKCASE = "Server.Items.FullBookcase" : BASE_CONTAINER, v0;
    construct: Some(|w| full_bookcase_new(w, &FULL_BOOKCASE)),
    default_weight: crate::class_props::default_weight,
}

/// `MapleArmoire()`.
pub fn maple_armoire_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = new_base_container(w, ty, 0);
    crate::simple::own_template(w, id, "Server.Items.MapleArmoire")
}

leaf! {
    /// `Server.Items.MapleArmoire`.
    MAPLE_ARMOIRE = "Server.Items.MapleArmoire" : BASE_CONTAINER, v0;
    construct: Some(|w| maple_armoire_new(w, &MAPLE_ARMOIRE)),
    default_weight: crate::class_props::default_weight,
}

/// `RedArmoire()`.
pub fn red_armoire_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = new_base_container(w, ty, 0);
    crate::simple::own_template(w, id, "Server.Items.RedArmoire")
}

leaf! {
    /// `Server.Items.RedArmoire`.
    RED_ARMOIRE = "Server.Items.RedArmoire" : BASE_CONTAINER, v0;
    construct: Some(|w| red_armoire_new(w, &RED_ARMOIRE)),
    default_weight: crate::class_props::default_weight,
}

/// `ShortCabinet()`.
pub fn short_cabinet_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = new_base_container(w, ty, 0);
    crate::simple::own_template(w, id, "Server.Items.ShortCabinet")
}

leaf! {
    /// `Server.Items.ShortCabinet`.
    SHORT_CABINET = "Server.Items.ShortCabinet" : BASE_CONTAINER, v0;
    construct: Some(|w| short_cabinet_new(w, &SHORT_CABINET)),
    default_weight: crate::class_props::default_weight,
}

/// `SimpleElvenArmoire()`.
pub fn simple_elven_armoire_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = new_base_container(w, ty, 0);
    crate::simple::own_template(w, id, "Server.Items.SimpleElvenArmoire")
}

leaf! {
    /// `Server.Items.SimpleElvenArmoire`.
    SIMPLE_ELVEN_ARMOIRE = "Server.Items.SimpleElvenArmoire" : BASE_CONTAINER, enc;
    construct: Some(|w| simple_elven_armoire_new(w, &SIMPLE_ELVEN_ARMOIRE)),
    default_weight: crate::class_props::default_weight,
    default_gump_id: crate::class_props::default_gump_id,
    default_drop_sound: crate::class_props::default_drop_sound,
}

/// `TallCabinet()`.
pub fn tall_cabinet_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = new_base_container(w, ty, 0);
    crate::simple::own_template(w, id, "Server.Items.TallCabinet")
}

leaf! {
    /// `Server.Items.TallCabinet`.
    TALL_CABINET = "Server.Items.TallCabinet" : BASE_CONTAINER, v0;
    construct: Some(|w| tall_cabinet_new(w, &TALL_CABINET)),
    default_weight: crate::class_props::default_weight,
}

/// Every class of this file: the type and its `[TypeAlias]` names.
pub static ALL: &[(&ItemType, &[&str])] = &[
    (&CHERRY_ARMOIRE, &[]),
    (&DRAWER, &[]),
    (&ELEGANT_ARMOIRE, &[]),
    (&EMPTY_BOOKCASE, &[]),
    (&FANCY_DRAWER, &[]),
    (&FANCY_ELVEN_ARMOIRE, &[]),
    (&FULL_BOOKCASE, &[]),
    (&MAPLE_ARMOIRE, &[]),
    (&RED_ARMOIRE, &[]),
    (&SHORT_CABINET, &[]),
    (&SIMPLE_ELVEN_ARMOIRE, &[]),
    (&TALL_CABINET, &[]),
];

/// `[Flippable(a, b)]` of the classes (the flip command reads it).
pub static FLIPPABLE: &[(&ItemType, i32, i32)] = &[
    (&CHERRY_ARMOIRE, 0x285D, 0x285E),
    (&DRAWER, 0xa2c, 0xa34),
    (&ELEGANT_ARMOIRE, 0x2859, 0x285A),
    (&EMPTY_BOOKCASE, 0xa9d, 0xa9e),
    (&FANCY_DRAWER, 0xa30, 0xa38),
    (&FANCY_ELVEN_ARMOIRE, 0x2D07, 0x2D08),
    (&MAPLE_ARMOIRE, 0x285B, 0x285C),
    (&RED_ARMOIRE, 0x2857, 0x2858),
    (&SHORT_CABINET, 0x2817, 0x2818),
    (&SIMPLE_ELVEN_ARMOIRE, 0x2D05, 0x2D06),
    (&TALL_CABINET, 0x2815, 0x2816),
];

/// The interfaces a class lists after its base class (`is ICommodity` and the like); none has a consumer yet.
pub static IMPLEMENTS: &[(&ItemType, &str)] = &[];

/// `ICommodity` members of a class: (type, `IsDeedable`); `DescriptionNumber` is `LabelNumber`.
pub static COMMODITIES: &[(&ItemType, bool)] = &[];

/// Classes with `[Flippable]` or `[DynamicFlipping]` and no graphics pair (the flip is the class itself).
pub static OWN_FLIP: &[&ItemType] = &[];

/// Classes with `[Furniture]`.
pub static FURNITURE: &[&ItemType] = &[
    &CHERRY_ARMOIRE,
    &DRAWER,
    &ELEGANT_ARMOIRE,
    &EMPTY_BOOKCASE,
    &FANCY_DRAWER,
    &FANCY_ELVEN_ARMOIRE,
    &FULL_BOOKCASE,
    &MAPLE_ARMOIRE,
    &RED_ARMOIRE,
    &SHORT_CABINET,
    &SIMPLE_ELVEN_ARMOIRE,
    &TALL_CABINET,
];

/// Registers the classes of this file.
pub fn register(types: &mut TypeRegistry) {
    for (t, aliases) in ALL {
        types.register_item(t, aliases);
    }
}
