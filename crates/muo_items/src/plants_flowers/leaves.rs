//! The PlantsFlowers classes that are plain tables (a constructor and constant overrides); the classes with logic
//! are hand-written next to this file.

use crate::simple::leaf;
use muo_server::World;
use muo_server::ids::ItemId;
use muo_server::item::{ITEM_BASE, ItemType};
use muo_server::type_registry::TypeRegistry;

/// `LargeEmptyPot()`.
pub fn large_empty_pot_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.LargeEmptyPot")
}

leaf! {
    /// `Server.Items.LargeEmptyPot`.
    LARGE_EMPTY_POT = "Server.Items.LargeEmptyPot" : ITEM_BASE, v0;
    construct: Some(|w| large_empty_pot_new(w, &LARGE_EMPTY_POT)),
    default_weight: crate::class_props::default_weight,
}

/// `SmallEmptyPot()`.
pub fn small_empty_pot_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.SmallEmptyPot")
}

leaf! {
    /// `Server.Items.SmallEmptyPot`.
    SMALL_EMPTY_POT = "Server.Items.SmallEmptyPot" : ITEM_BASE, v0;
    construct: Some(|w| small_empty_pot_new(w, &SMALL_EMPTY_POT)),
    default_weight: crate::class_props::default_weight,
}

/// `PottedCactus()`.
pub fn potted_cactus_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.PottedCactus")
}

leaf! {
    /// `Server.Items.PottedCactus`.
    POTTED_CACTUS = "Server.Items.PottedCactus" : ITEM_BASE, v0;
    construct: Some(|w| potted_cactus_new(w, &POTTED_CACTUS)),
    default_weight: crate::class_props::default_weight,
}

/// `PottedCactus1()`.
pub fn potted_cactus1_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.PottedCactus1")
}

leaf! {
    /// `Server.Items.PottedCactus1`.
    POTTED_CACTUS1 = "Server.Items.PottedCactus1" : ITEM_BASE, v0;
    construct: Some(|w| potted_cactus1_new(w, &POTTED_CACTUS1)),
    default_weight: crate::class_props::default_weight,
}

/// `PottedCactus2()`.
pub fn potted_cactus2_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.PottedCactus2")
}

leaf! {
    /// `Server.Items.PottedCactus2`.
    POTTED_CACTUS2 = "Server.Items.PottedCactus2" : ITEM_BASE, v0;
    construct: Some(|w| potted_cactus2_new(w, &POTTED_CACTUS2)),
    default_weight: crate::class_props::default_weight,
}

/// `PottedCactus3()`.
pub fn potted_cactus3_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.PottedCactus3")
}

leaf! {
    /// `Server.Items.PottedCactus3`.
    POTTED_CACTUS3 = "Server.Items.PottedCactus3" : ITEM_BASE, v0;
    construct: Some(|w| potted_cactus3_new(w, &POTTED_CACTUS3)),
    default_weight: crate::class_props::default_weight,
}

/// `PottedCactus4()`.
pub fn potted_cactus4_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.PottedCactus4")
}

leaf! {
    /// `Server.Items.PottedCactus4`.
    POTTED_CACTUS4 = "Server.Items.PottedCactus4" : ITEM_BASE, v0;
    construct: Some(|w| potted_cactus4_new(w, &POTTED_CACTUS4)),
    default_weight: crate::class_props::default_weight,
}

/// `PottedCactus5()`.
pub fn potted_cactus5_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.PottedCactus5")
}

leaf! {
    /// `Server.Items.PottedCactus5`.
    POTTED_CACTUS5 = "Server.Items.PottedCactus5" : ITEM_BASE, v0;
    construct: Some(|w| potted_cactus5_new(w, &POTTED_CACTUS5)),
    default_weight: crate::class_props::default_weight,
}

/// `PottedPlant()`.
pub fn potted_plant_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.PottedPlant")
}

leaf! {
    /// `Server.Items.PottedPlant`.
    POTTED_PLANT = "Server.Items.PottedPlant" : ITEM_BASE, v0;
    construct: Some(|w| potted_plant_new(w, &POTTED_PLANT)),
    default_weight: crate::class_props::default_weight,
}

/// `PottedPlant1()`.
pub fn potted_plant1_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.PottedPlant1")
}

leaf! {
    /// `Server.Items.PottedPlant1`.
    POTTED_PLANT1 = "Server.Items.PottedPlant1" : ITEM_BASE, v0;
    construct: Some(|w| potted_plant1_new(w, &POTTED_PLANT1)),
    default_weight: crate::class_props::default_weight,
}

/// `PottedPlant2()`.
pub fn potted_plant2_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.PottedPlant2")
}

leaf! {
    /// `Server.Items.PottedPlant2`.
    POTTED_PLANT2 = "Server.Items.PottedPlant2" : ITEM_BASE, v0;
    construct: Some(|w| potted_plant2_new(w, &POTTED_PLANT2)),
    default_weight: crate::class_props::default_weight,
}

/// `PottedTree()`.
pub fn potted_tree_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.PottedTree")
}

leaf! {
    /// `Server.Items.PottedTree`.
    POTTED_TREE = "Server.Items.PottedTree" : ITEM_BASE, v0;
    construct: Some(|w| potted_tree_new(w, &POTTED_TREE)),
    default_weight: crate::class_props::default_weight,
}

/// `PottedTree1()`.
pub fn potted_tree1_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.PottedTree1")
}

leaf! {
    /// `Server.Items.PottedTree1`.
    POTTED_TREE1 = "Server.Items.PottedTree1" : ITEM_BASE, v0;
    construct: Some(|w| potted_tree1_new(w, &POTTED_TREE1)),
    default_weight: crate::class_props::default_weight,
}

/// Every class of this file: the type and its `[TypeAlias]` names.
pub static ALL: &[(&ItemType, &[&str])] = &[
    (&LARGE_EMPTY_POT, &[]),
    (&SMALL_EMPTY_POT, &[]),
    (&POTTED_CACTUS, &[]),
    (&POTTED_CACTUS1, &[]),
    (&POTTED_CACTUS2, &[]),
    (&POTTED_CACTUS3, &[]),
    (&POTTED_CACTUS4, &[]),
    (&POTTED_CACTUS5, &[]),
    (&POTTED_PLANT, &[]),
    (&POTTED_PLANT1, &[]),
    (&POTTED_PLANT2, &[]),
    (&POTTED_TREE, &[]),
    (&POTTED_TREE1, &[]),
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
