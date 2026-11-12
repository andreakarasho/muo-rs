//! The Body Parts classes that are plain tables (a constructor and constant overrides); the classes with logic
//! are hand-written next to this file.

use crate::simple::leaf;
use muo_server::World;
use muo_server::ids::ItemId;
use muo_server::item::{ITEM_BASE, ItemType, ops};
use muo_server::type_registry::TypeRegistry;

/// `LeftArm()`.
pub fn left_arm_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    ops::new_item_with_id(w, ty, 0x1DA1)
}

leaf! {
    /// `Server.Items.LeftArm`.
    LEFT_ARM = "Server.Items.LeftArm" : ITEM_BASE, v0;
    construct: Some(|w| left_arm_new(w, &LEFT_ARM)),
}

/// `LeftLeg()`.
pub fn left_leg_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    ops::new_item_with_id(w, ty, 0x1DA3)
}

leaf! {
    /// `Server.Items.LeftLeg`.
    LEFT_LEG = "Server.Items.LeftLeg" : ITEM_BASE, v0;
    construct: Some(|w| left_leg_new(w, &LEFT_LEG)),
}

/// `RightArm()`.
pub fn right_arm_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    ops::new_item_with_id(w, ty, 0x1DA2)
}

leaf! {
    /// `Server.Items.RightArm`.
    RIGHT_ARM = "Server.Items.RightArm" : ITEM_BASE, v0;
    construct: Some(|w| right_arm_new(w, &RIGHT_ARM)),
}

/// `RightLeg()`.
pub fn right_leg_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    ops::new_item_with_id(w, ty, 0x1DA4)
}

leaf! {
    /// `Server.Items.RightLeg`.
    RIGHT_LEG = "Server.Items.RightLeg" : ITEM_BASE, v0;
    construct: Some(|w| right_leg_new(w, &RIGHT_LEG)),
}

/// `Torso()`.
pub fn torso_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    ops::new_item_with_id(w, ty, 0x1D9F)
}

leaf! {
    /// `Server.Items.Torso`.
    TORSO = "Server.Items.Torso" : ITEM_BASE, v0;
    construct: Some(|w| torso_new(w, &TORSO)),
    default_weight: |_, _| 2.0,
}

/// Every class of this file: the type and its `[TypeAlias]` names.
pub static ALL: &[(&ItemType, &[&str])] = &[
    (&LEFT_ARM, &[]),
    (&LEFT_LEG, &[]),
    (&RIGHT_ARM, &[]),
    (&RIGHT_LEG, &[]),
    (&TORSO, &[]),
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
