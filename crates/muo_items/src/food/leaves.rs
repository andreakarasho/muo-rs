//! The Food classes that are plain tables (a constructor and constant overrides); the classes with logic
//! are hand-written next to this file.

use crate::simple::{hue, leaf};
use muo_server::World;
use muo_server::ids::ItemId;
use muo_server::item::{ITEM_BASE, ItemType, ops};
use muo_server::type_registry::TypeRegistry;

/// `EmptyBentoBox()`.
pub fn empty_bento_box_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.EmptyBentoBox")
}

leaf! {
    /// `Server.Items.EmptyBentoBox`.
    EMPTY_BENTO_BOX = "Server.Items.EmptyBentoBox" : ITEM_BASE, v0;
    construct: Some(|w| empty_bento_box_new(w, &EMPTY_BENTO_BOX)),
    default_weight: |_, _| 5.0,
}

/// `GreenTeaBasket()`.
pub fn green_tea_basket_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.GreenTeaBasket")
}

leaf! {
    /// `Server.Items.GreenTeaBasket`.
    GREEN_TEA_BASKET = "Server.Items.GreenTeaBasket" : ITEM_BASE, v0;
    construct: Some(|w| green_tea_basket_new(w, &GREEN_TEA_BASKET)),
    default_weight: |_, _| 10.0,
}

/// `Glass()`.
pub fn glass_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.Glass")
}

leaf! {
    /// `Server.Items.Glass`.
    GLASS = "Server.Items.Glass" : ITEM_BASE, v0;
    construct: Some(|w| glass_new(w, &GLASS)),
    default_weight: |_, _| 0.1,
}

/// `GlassBottle()`.
pub fn glass_bottle_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.GlassBottle")
}

leaf! {
    /// `Server.Items.GlassBottle`.
    GLASS_BOTTLE = "Server.Items.GlassBottle" : ITEM_BASE, v0;
    construct: Some(|w| glass_bottle_new(w, &GLASS_BOTTLE)),
    default_weight: |_, _| 0.3,
}

/// `EmptyPewterBowl()`.
pub fn empty_pewter_bowl_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.EmptyPewterBowl")
}

leaf! {
    /// `Server.Items.EmptyPewterBowl`.
    EMPTY_PEWTER_BOWL = "Server.Items.EmptyPewterBowl" : ITEM_BASE, v0;
    construct: Some(|w| empty_pewter_bowl_new(w, &EMPTY_PEWTER_BOWL)),
    default_weight: |_, _| 1.0,
}

/// `EmptyPewterTub()`.
pub fn empty_pewter_tub_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.EmptyPewterTub")
}

leaf! {
    /// `Server.Items.EmptyPewterTub`.
    EMPTY_PEWTER_TUB = "Server.Items.EmptyPewterTub" : ITEM_BASE, v0;
    construct: Some(|w| empty_pewter_tub_new(w, &EMPTY_PEWTER_TUB)),
    default_weight: |_, _| 2.0,
}

/// `EmptyWoodenBowl()`.
pub fn empty_wooden_bowl_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.EmptyWoodenBowl")
}

leaf! {
    /// `Server.Items.EmptyWoodenBowl`.
    EMPTY_WOODEN_BOWL = "Server.Items.EmptyWoodenBowl" : ITEM_BASE, v0;
    construct: Some(|w| empty_wooden_bowl_new(w, &EMPTY_WOODEN_BOWL)),
    default_weight: |_, _| 1.0,
}

/// `EmptyWoodenTub()`.
pub fn empty_wooden_tub_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.EmptyWoodenTub")
}

leaf! {
    /// `Server.Items.EmptyWoodenTub`.
    EMPTY_WOODEN_TUB = "Server.Items.EmptyWoodenTub" : ITEM_BASE, v0;
    construct: Some(|w| empty_wooden_tub_new(w, &EMPTY_WOODEN_TUB)),
    default_weight: |_, _| 2.0,
}

/// `CocoaButter()`.
pub fn cocoa_butter_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.CocoaButter")
}

leaf! {
    /// `Server.Items.CocoaButter`.
    COCOA_BUTTER = "Server.Items.CocoaButter" : ITEM_BASE, v0;
    construct: Some(|w| cocoa_butter_new(w, &COCOA_BUTTER)),
    label_number: |_, _| 1080005,
    default_weight: |_, _| 1.0,
}

/// `CocoaLiquor()`.
pub fn cocoa_liquor_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.CocoaLiquor")
}

leaf! {
    /// `Server.Items.CocoaLiquor`.
    COCOA_LIQUOR = "Server.Items.CocoaLiquor" : ITEM_BASE, v0;
    construct: Some(|w| cocoa_liquor_new(w, &COCOA_LIQUOR)),
    label_number: |_, _| 1080007,
    default_weight: |_, _| 1.0,
}

/// `CocoaPulp(i32 amount)`.
pub fn cocoa_pulp_new(w: &mut World, ty: &'static ItemType, amount: i32) -> ItemId {
    let id = ops::new_item_with_id(w, ty, 0xF7C);
    hue(w, id, 0x219);
    ops::set_stackable(w, id, true);
    ops::set_amount(w, id, amount);
    id
}

leaf! {
    /// `Server.Items.CocoaPulp`.
    COCOA_PULP = "Server.Items.CocoaPulp" : ITEM_BASE, v0;
    construct: Some(|w| cocoa_pulp_new(w, &COCOA_PULP, 1)),
    label_number: |_, _| 1080530,
    default_weight: |_, _| 1.0,
}

/// `SackOfSugar(i32 amount)`.
pub fn sack_of_sugar_new(w: &mut World, ty: &'static ItemType, amount: i32) -> ItemId {
    let id = ops::new_item_with_id(w, ty, 0x1039);
    hue(w, id, 0x461);
    ops::set_stackable(w, id, true);
    ops::set_amount(w, id, amount);
    id
}

leaf! {
    /// `Server.Items.SackOfSugar`.
    SACK_OF_SUGAR = "Server.Items.SackOfSugar" : ITEM_BASE, v0;
    construct: Some(|w| sack_of_sugar_new(w, &SACK_OF_SUGAR, 1)),
    label_number: |_, _| 1080003,
    default_weight: |_, _| 1.0,
}

/// `Vanilla(i32 amount)`.
pub fn vanilla_new(w: &mut World, ty: &'static ItemType, amount: i32) -> ItemId {
    let id = ops::new_item_with_id(w, ty, 0xE2A);
    hue(w, id, 0x462);
    ops::set_stackable(w, id, true);
    ops::set_amount(w, id, amount);
    id
}

leaf! {
    /// `Server.Items.Vanilla`.
    VANILLA = "Server.Items.Vanilla" : ITEM_BASE, v0;
    construct: Some(|w| vanilla_new(w, &VANILLA, 1)),
    label_number: |_, _| 1080009,
    default_weight: |_, _| 1.0,
}

/// `Eggshells()`.
pub fn eggshells_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.Eggshells")
}

leaf! {
    /// `Server.Items.Eggshells`.
    EGGSHELLS = "Server.Items.Eggshells" : ITEM_BASE, v0;
    construct: Some(|w| eggshells_new(w, &EGGSHELLS)),
    default_weight: |_, _| 0.5,
}

/// `WoodenBowl()`.
pub fn wooden_bowl_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.WoodenBowl")
}

leaf! {
    /// `Server.Items.WoodenBowl`.
    WOODEN_BOWL = "Server.Items.WoodenBowl" : ITEM_BASE, v0;
    construct: Some(|w| wooden_bowl_new(w, &WOODEN_BOWL)),
    default_weight: |_, _| 1.0,
}

/// `SheafOfHay()`.
pub fn sheaf_of_hay_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.SheafOfHay")
}

leaf! {
    /// `Server.Items.SheafOfHay`.
    SHEAF_OF_HAY = "Server.Items.SheafOfHay" : ITEM_BASE, v0;
    construct: Some(|w| sheaf_of_hay_new(w, &SHEAF_OF_HAY)),
    default_weight: |_, _| 10.0,
}

/// Every class of this file: the type and its `[TypeAlias]` names.
pub static ALL: &[(&ItemType, &[&str])] = &[
    (&EMPTY_BENTO_BOX, &[]),
    (&GREEN_TEA_BASKET, &[]),
    (&GLASS, &[]),
    (&GLASS_BOTTLE, &[]),
    (&EMPTY_PEWTER_BOWL, &[]),
    (&EMPTY_PEWTER_TUB, &["Server.Items.EmptyLargePewterBowl"]),
    (&EMPTY_WOODEN_BOWL, &[]),
    (&EMPTY_WOODEN_TUB, &["Server.Items.EmptyLargeWoodenBowl"]),
    (&COCOA_BUTTER, &[]),
    (&COCOA_LIQUOR, &[]),
    (&COCOA_PULP, &[]),
    (&SACK_OF_SUGAR, &[]),
    (&VANILLA, &[]),
    (&EGGSHELLS, &[]),
    (&WOODEN_BOWL, &[]),
    (&SHEAF_OF_HAY, &[]),
];

/// `[Flippable(a, b)]` of the classes (the flip command reads it).
pub static FLIPPABLE: &[(&ItemType, i32, i32)] = &[];

/// The interfaces a class lists after its base class (`is ICommodity` and the like); none has a consumer yet.
pub static IMPLEMENTS: &[(&ItemType, &str)] = &[];

/// `ICommodity` members of a class: (type, `IsDeedable`); `DescriptionNumber` is `LabelNumber`.
pub static COMMODITIES: &[(&ItemType, bool)] = &[];

/// Classes with `[Flippable]` or `[DynamicFlipping]` and no graphics pair (the flip is the class itself).
pub static OWN_FLIP: &[&ItemType] = &[];

/// Classes with `[Furniture]`.
pub static FURNITURE: &[&ItemType] = &[];

/// Registers the classes of this file.
pub fn register(types: &mut TypeRegistry) {
    for (t, aliases) in ALL {
        types.register_item(t, aliases);
    }
}
