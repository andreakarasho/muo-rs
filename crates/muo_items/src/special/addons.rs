//! The special addon families (Heritage, Evil Home Decor, Broken Furniture, Holiday): the addons, components and
//! deeds that are plain tables. Those with logic are hand-written next to this file.

use muo_content::items::addon::{
    self, ADDON_COMPONENT, AddonClass, BASE_ADDON, BASE_ADDON_CLASS, BASE_ADDON_DEED, Comp,
    DeedClass,
};
use muo_content::items::addon_container::{
    self, BASE_ADDON_CONTAINER, BASE_ADDON_CONTAINER_CLASS, BASE_ADDON_CONTAINER_DEED, CComp,
    ContainerAddonClass, ContainerDeedClass,
};
use muo_server::World;
use muo_server::ids::ItemId;
use muo_server::item::{ItemType, LootType, ops};
use muo_server::type_registry::TypeRegistry;

use super::hand::*;
use crate::simple::leaf;

#[allow(dead_code)]
fn item_hue(w: &mut World, id: ItemId, value: i32) {
    crate::simple::hue(w, id, value);
}

/// `BrokenArmoireComponent()`.
pub fn broken_armoire_component_new(w: &mut World) -> ItemId {
    addon::new_component(w, &BROKEN_ARMOIRE_COMPONENT, 0xC12)
}
leaf! {
    /// `Server.Items.BrokenArmoireComponent`.
    BROKEN_ARMOIRE_COMPONENT = "Server.Items.BrokenArmoireComponent" : ADDON_COMPONENT, enc;
    label_number: crate::class_props::label_number,
}

/// `BrokenArmoireAddon()`.
pub fn broken_armoire_addon_new(w: &mut World) -> ItemId {
    let id = addon::new_addon(w, &BROKEN_ARMOIRE_ADDON);
    addon::add_components(
        w,
        id,
        &[(Comp::Custom(broken_armoire_component_new), 0, 0, 0)],
    );
    id
}
static BROKEN_ARMOIRE_ADDON_CLASS: AddonClass = AddonClass {
    deed: Some(broken_armoire_deed_new),
    ..BASE_ADDON_CLASS
};
leaf! {
    /// `Server.Items.BrokenArmoireAddon`.
    BROKEN_ARMOIRE_ADDON = "Server.Items.BrokenArmoireAddon" : BASE_ADDON, enc;
    construct: Some(broken_armoire_addon_new),
    family: Some(&BROKEN_ARMOIRE_ADDON_CLASS),
}

/// `BrokenArmoireDeed()`.
pub fn broken_armoire_deed_new(w: &mut World) -> ItemId {
    let id = addon::new_deed(w, &BROKEN_ARMOIRE_DEED);
    ops::set_loot_type(w, id, LootType::Blessed);
    id
}
static BROKEN_ARMOIRE_DEED_CLASS: DeedClass = DeedClass {
    addon: |w, _deed| broken_armoire_addon_new(w),
    select_direction: None,
};
leaf! {
    /// `Server.Items.BrokenArmoireDeed`.
    BROKEN_ARMOIRE_DEED = "Server.Items.BrokenArmoireDeed" : BASE_ADDON_DEED, enc;
    construct: Some(broken_armoire_deed_new),
    family: Some(&BROKEN_ARMOIRE_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `BrokenBedAddon(bool east)`.
pub fn broken_bed_addon_new(w: &mut World, east: bool) -> ItemId {
    let id = addon::new_addon(w, &BROKEN_BED_ADDON);
    if east {
        addon::add_components(
            w,
            id,
            &[
                (Comp::Localized(6293, 1076263), 0, 0, 0),
                (Comp::Localized(6292, 1076263), 0, 1, 0),
                (Comp::Localized(6295, 1076263), 1, 0, 0),
                (Comp::Localized(6294, 1076263), 1, 1, 0),
            ],
        );
    } else {
        addon::add_components(
            w,
            id,
            &[
                (Comp::Localized(6297, 1076263), 0, 0, 0),
                (Comp::Localized(6296, 1076263), 1, 0, 0),
                (Comp::Localized(6299, 1076263), 0, 1, 0),
                (Comp::Localized(6298, 1076263), 1, 1, 0),
            ],
        );
    }
    id
}
static BROKEN_BED_ADDON_CLASS: AddonClass = AddonClass {
    deed: Some(broken_bed_deed_new),
    ..BASE_ADDON_CLASS
};
leaf! {
    /// `Server.Items.BrokenBedAddon`.
    BROKEN_BED_ADDON = "Server.Items.BrokenBedAddon" : BASE_ADDON, enc;
    family: Some(&BROKEN_BED_ADDON_CLASS),
}

/// `BrokenBedDeed()`.
pub fn broken_bed_deed_new(w: &mut World) -> ItemId {
    let id = addon::new_deed(w, &BROKEN_BED_DEED);
    ops::set_loot_type(w, id, LootType::Blessed);
    id
}
static BROKEN_BED_DEED_CLASS: DeedClass = DeedClass {
    addon: |w, deed| broken_bed_addon_new(w, addon::deed_east(w, deed)),
    select_direction: Some(1076749),
};
leaf! {
    /// `Server.Items.BrokenBedDeed`.
    BROKEN_BED_DEED = "Server.Items.BrokenBedDeed" : BASE_ADDON_DEED, enc;
    construct: Some(broken_bed_deed_new),
    family: Some(&BROKEN_BED_DEED_CLASS),
    on_double_click: addon::direction_deed_on_double_click,
    label_number: crate::class_props::label_number,
}

/// `BrokenBookcaseComponent()`.
pub fn broken_bookcase_component_new(w: &mut World) -> ItemId {
    addon::new_component(w, &BROKEN_BOOKCASE_COMPONENT, 0xC14)
}
leaf! {
    /// `Server.Items.BrokenBookcaseComponent`.
    BROKEN_BOOKCASE_COMPONENT = "Server.Items.BrokenBookcaseComponent" : ADDON_COMPONENT, enc;
    label_number: crate::class_props::label_number,
}

/// `BrokenBookcaseAddon()`.
pub fn broken_bookcase_addon_new(w: &mut World) -> ItemId {
    let id = addon::new_addon(w, &BROKEN_BOOKCASE_ADDON);
    addon::add_components(
        w,
        id,
        &[(Comp::Custom(broken_bookcase_component_new), 0, 0, 0)],
    );
    id
}
static BROKEN_BOOKCASE_ADDON_CLASS: AddonClass = AddonClass {
    deed: Some(broken_bookcase_deed_new),
    ..BASE_ADDON_CLASS
};
leaf! {
    /// `Server.Items.BrokenBookcaseAddon`.
    BROKEN_BOOKCASE_ADDON = "Server.Items.BrokenBookcaseAddon" : BASE_ADDON, enc;
    construct: Some(broken_bookcase_addon_new),
    family: Some(&BROKEN_BOOKCASE_ADDON_CLASS),
}

/// `BrokenBookcaseDeed()`.
pub fn broken_bookcase_deed_new(w: &mut World) -> ItemId {
    let id = addon::new_deed(w, &BROKEN_BOOKCASE_DEED);
    ops::set_loot_type(w, id, LootType::Blessed);
    id
}
static BROKEN_BOOKCASE_DEED_CLASS: DeedClass = DeedClass {
    addon: |w, _deed| broken_bookcase_addon_new(w),
    select_direction: None,
};
leaf! {
    /// `Server.Items.BrokenBookcaseDeed`.
    BROKEN_BOOKCASE_DEED = "Server.Items.BrokenBookcaseDeed" : BASE_ADDON_DEED, enc;
    construct: Some(broken_bookcase_deed_new),
    family: Some(&BROKEN_BOOKCASE_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `BrokenChestOfDrawersComponent()`.
pub fn broken_chest_of_drawers_component_new(w: &mut World) -> ItemId {
    addon::new_component(w, &BROKEN_CHEST_OF_DRAWERS_COMPONENT, 0xC24)
}
leaf! {
    /// `Server.Items.BrokenChestOfDrawersComponent`.
    BROKEN_CHEST_OF_DRAWERS_COMPONENT = "Server.Items.BrokenChestOfDrawersComponent" : ADDON_COMPONENT, enc;
    label_number: crate::class_props::label_number,
}

/// `BrokenChestOfDrawersAddon()`.
pub fn broken_chest_of_drawers_addon_new(w: &mut World) -> ItemId {
    let id = addon::new_addon(w, &BROKEN_CHEST_OF_DRAWERS_ADDON);
    addon::add_components(
        w,
        id,
        &[(Comp::Custom(broken_chest_of_drawers_component_new), 0, 0, 0)],
    );
    id
}
static BROKEN_CHEST_OF_DRAWERS_ADDON_CLASS: AddonClass = AddonClass {
    deed: Some(broken_chest_of_drawers_deed_new),
    ..BASE_ADDON_CLASS
};
leaf! {
    /// `Server.Items.BrokenChestOfDrawersAddon`.
    BROKEN_CHEST_OF_DRAWERS_ADDON = "Server.Items.BrokenChestOfDrawersAddon" : BASE_ADDON, enc;
    construct: Some(broken_chest_of_drawers_addon_new),
    family: Some(&BROKEN_CHEST_OF_DRAWERS_ADDON_CLASS),
}

/// `BrokenChestOfDrawersDeed()`.
pub fn broken_chest_of_drawers_deed_new(w: &mut World) -> ItemId {
    let id = addon::new_deed(w, &BROKEN_CHEST_OF_DRAWERS_DEED);
    ops::set_loot_type(w, id, LootType::Blessed);
    id
}
static BROKEN_CHEST_OF_DRAWERS_DEED_CLASS: DeedClass = DeedClass {
    addon: |w, _deed| broken_chest_of_drawers_addon_new(w),
    select_direction: None,
};
leaf! {
    /// `Server.Items.BrokenChestOfDrawersDeed`.
    BROKEN_CHEST_OF_DRAWERS_DEED = "Server.Items.BrokenChestOfDrawersDeed" : BASE_ADDON_DEED, enc;
    construct: Some(broken_chest_of_drawers_deed_new),
    family: Some(&BROKEN_CHEST_OF_DRAWERS_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `BrokenCoveredChairComponent()`.
pub fn broken_covered_chair_component_new(w: &mut World) -> ItemId {
    addon::new_component(w, &BROKEN_COVERED_CHAIR_COMPONENT, 0xC17)
}
leaf! {
    /// `Server.Items.BrokenCoveredChairComponent`.
    BROKEN_COVERED_CHAIR_COMPONENT = "Server.Items.BrokenCoveredChairComponent" : ADDON_COMPONENT, enc;
    label_number: crate::class_props::label_number,
}

/// `BrokenCoveredChairAddon()`.
pub fn broken_covered_chair_addon_new(w: &mut World) -> ItemId {
    let id = addon::new_addon(w, &BROKEN_COVERED_CHAIR_ADDON);
    addon::add_components(
        w,
        id,
        &[(Comp::Custom(broken_covered_chair_component_new), 0, 0, 0)],
    );
    id
}
static BROKEN_COVERED_CHAIR_ADDON_CLASS: AddonClass = AddonClass {
    deed: Some(broken_covered_chair_deed_new),
    ..BASE_ADDON_CLASS
};
leaf! {
    /// `Server.Items.BrokenCoveredChairAddon`.
    BROKEN_COVERED_CHAIR_ADDON = "Server.Items.BrokenCoveredChairAddon" : BASE_ADDON, enc;
    construct: Some(broken_covered_chair_addon_new),
    family: Some(&BROKEN_COVERED_CHAIR_ADDON_CLASS),
}

/// `BrokenCoveredChairDeed()`.
pub fn broken_covered_chair_deed_new(w: &mut World) -> ItemId {
    let id = addon::new_deed(w, &BROKEN_COVERED_CHAIR_DEED);
    ops::set_loot_type(w, id, LootType::Blessed);
    id
}
static BROKEN_COVERED_CHAIR_DEED_CLASS: DeedClass = DeedClass {
    addon: |w, _deed| broken_covered_chair_addon_new(w),
    select_direction: None,
};
leaf! {
    /// `Server.Items.BrokenCoveredChairDeed`.
    BROKEN_COVERED_CHAIR_DEED = "Server.Items.BrokenCoveredChairDeed" : BASE_ADDON_DEED, enc;
    construct: Some(broken_covered_chair_deed_new),
    family: Some(&BROKEN_COVERED_CHAIR_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `BrokenFallenChairComponent()`.
pub fn broken_fallen_chair_component_new(w: &mut World) -> ItemId {
    addon::new_component(w, &BROKEN_FALLEN_CHAIR_COMPONENT, 0xC19)
}
leaf! {
    /// `Server.Items.BrokenFallenChairComponent`.
    BROKEN_FALLEN_CHAIR_COMPONENT = "Server.Items.BrokenFallenChairComponent" : ADDON_COMPONENT, enc;
    label_number: crate::class_props::label_number,
}

/// `BrokenFallenChairAddon()`.
pub fn broken_fallen_chair_addon_new(w: &mut World) -> ItemId {
    let id = addon::new_addon(w, &BROKEN_FALLEN_CHAIR_ADDON);
    addon::add_components(
        w,
        id,
        &[(Comp::Custom(broken_fallen_chair_component_new), 0, 0, 0)],
    );
    id
}
static BROKEN_FALLEN_CHAIR_ADDON_CLASS: AddonClass = AddonClass {
    deed: Some(broken_fallen_chair_deed_new),
    ..BASE_ADDON_CLASS
};
leaf! {
    /// `Server.Items.BrokenFallenChairAddon`.
    BROKEN_FALLEN_CHAIR_ADDON = "Server.Items.BrokenFallenChairAddon" : BASE_ADDON, enc;
    construct: Some(broken_fallen_chair_addon_new),
    family: Some(&BROKEN_FALLEN_CHAIR_ADDON_CLASS),
}

/// `BrokenFallenChairDeed()`.
pub fn broken_fallen_chair_deed_new(w: &mut World) -> ItemId {
    let id = addon::new_deed(w, &BROKEN_FALLEN_CHAIR_DEED);
    ops::set_loot_type(w, id, LootType::Blessed);
    id
}
static BROKEN_FALLEN_CHAIR_DEED_CLASS: DeedClass = DeedClass {
    addon: |w, _deed| broken_fallen_chair_addon_new(w),
    select_direction: None,
};
leaf! {
    /// `Server.Items.BrokenFallenChairDeed`.
    BROKEN_FALLEN_CHAIR_DEED = "Server.Items.BrokenFallenChairDeed" : BASE_ADDON_DEED, enc;
    construct: Some(broken_fallen_chair_deed_new),
    family: Some(&BROKEN_FALLEN_CHAIR_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `BrokenVanityAddon(bool east)`.
pub fn broken_vanity_addon_new(w: &mut World, east: bool) -> ItemId {
    let id = addon::new_addon(w, &BROKEN_VANITY_ADDON);
    if east {
        addon::add_components(
            w,
            id,
            &[
                (Comp::Localized(3104, 1076260), 0, 0, 0),
                (Comp::Localized(3105, 1076260), 0, -1, 0),
            ],
        );
    } else {
        addon::add_components(
            w,
            id,
            &[
                (Comp::Localized(3106, 1076260), 0, 0, 0),
                (Comp::Localized(3107, 1076260), -1, 0, 0),
            ],
        );
    }
    id
}
static BROKEN_VANITY_ADDON_CLASS: AddonClass = AddonClass {
    deed: Some(broken_vanity_deed_new),
    ..BASE_ADDON_CLASS
};
leaf! {
    /// `Server.Items.BrokenVanityAddon`.
    BROKEN_VANITY_ADDON = "Server.Items.BrokenVanityAddon" : BASE_ADDON, enc;
    family: Some(&BROKEN_VANITY_ADDON_CLASS),
}

/// `BrokenVanityDeed()`.
pub fn broken_vanity_deed_new(w: &mut World) -> ItemId {
    let id = addon::new_deed(w, &BROKEN_VANITY_DEED);
    ops::set_loot_type(w, id, LootType::Blessed);
    id
}
static BROKEN_VANITY_DEED_CLASS: DeedClass = DeedClass {
    addon: |w, deed| broken_vanity_addon_new(w, addon::deed_east(w, deed)),
    select_direction: Some(1076747),
};
leaf! {
    /// `Server.Items.BrokenVanityDeed`.
    BROKEN_VANITY_DEED = "Server.Items.BrokenVanityDeed" : BASE_ADDON_DEED, enc;
    construct: Some(broken_vanity_deed_new),
    family: Some(&BROKEN_VANITY_DEED_CLASS),
    on_double_click: addon::direction_deed_on_double_click,
    label_number: crate::class_props::label_number,
}

/// `StandingBrokenChairComponent()`.
pub fn standing_broken_chair_component_new(w: &mut World) -> ItemId {
    addon::new_component(w, &STANDING_BROKEN_CHAIR_COMPONENT, 0xC1B)
}
leaf! {
    /// `Server.Items.StandingBrokenChairComponent`.
    STANDING_BROKEN_CHAIR_COMPONENT = "Server.Items.StandingBrokenChairComponent" : ADDON_COMPONENT, enc;
    label_number: crate::class_props::label_number,
}

/// `StandingBrokenChairAddon()`.
pub fn standing_broken_chair_addon_new(w: &mut World) -> ItemId {
    let id = addon::new_addon(w, &STANDING_BROKEN_CHAIR_ADDON);
    addon::add_components(
        w,
        id,
        &[(Comp::Custom(standing_broken_chair_component_new), 0, 0, 0)],
    );
    id
}
static STANDING_BROKEN_CHAIR_ADDON_CLASS: AddonClass = AddonClass {
    deed: Some(standing_broken_chair_deed_new),
    ..BASE_ADDON_CLASS
};
leaf! {
    /// `Server.Items.StandingBrokenChairAddon`.
    STANDING_BROKEN_CHAIR_ADDON = "Server.Items.StandingBrokenChairAddon" : BASE_ADDON, enc;
    construct: Some(standing_broken_chair_addon_new),
    family: Some(&STANDING_BROKEN_CHAIR_ADDON_CLASS),
}

/// `StandingBrokenChairDeed()`.
pub fn standing_broken_chair_deed_new(w: &mut World) -> ItemId {
    let id = addon::new_deed(w, &STANDING_BROKEN_CHAIR_DEED);
    ops::set_loot_type(w, id, LootType::Blessed);
    id
}
static STANDING_BROKEN_CHAIR_DEED_CLASS: DeedClass = DeedClass {
    addon: |w, _deed| standing_broken_chair_addon_new(w),
    select_direction: None,
};
leaf! {
    /// `Server.Items.StandingBrokenChairDeed`.
    STANDING_BROKEN_CHAIR_DEED = "Server.Items.StandingBrokenChairDeed" : BASE_ADDON_DEED, enc;
    construct: Some(standing_broken_chair_deed_new),
    family: Some(&STANDING_BROKEN_CHAIR_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `BoneCouchComponent(itemID)`.
pub fn bone_couch_component_new(w: &mut World, item_id: i32) -> ItemId {
    addon::new_component(w, &BONE_COUCH_COMPONENT, item_id)
}
leaf! {
    /// `Server.Items.BoneCouchComponent`.
    BONE_COUCH_COMPONENT = "Server.Items.BoneCouchComponent" : ADDON_COMPONENT, enc;
    on_move_over: |w, id, m| addon::move_over_sound(w, id, m, 0x547, 0x54A),
    label_number: crate::class_props::label_number,
}

/// `BoneTableAddon()`.
pub fn bone_table_addon_new(w: &mut World) -> ItemId {
    let id = addon::new_addon(w, &BONE_TABLE_ADDON);
    addon::add_components(w, id, &[(Comp::Localized(10844, 1074478), 0, 0, 0)]);
    id
}
static BONE_TABLE_ADDON_CLASS: AddonClass = AddonClass {
    deed: Some(bone_table_deed_new),
    ..BASE_ADDON_CLASS
};
leaf! {
    /// `Server.Items.BoneTableAddon`.
    BONE_TABLE_ADDON = "Server.Items.BoneTableAddon" : BASE_ADDON, enc;
    construct: Some(bone_table_addon_new),
    family: Some(&BONE_TABLE_ADDON_CLASS),
}

/// `BoneTableDeed()`.
pub fn bone_table_deed_new(w: &mut World) -> ItemId {
    let id = addon::new_deed(w, &BONE_TABLE_DEED);
    ops::set_loot_type(w, id, LootType::Blessed);
    id
}
static BONE_TABLE_DEED_CLASS: DeedClass = DeedClass {
    addon: |w, _deed| bone_table_addon_new(w),
    select_direction: None,
};
leaf! {
    /// `Server.Items.BoneTableDeed`.
    BONE_TABLE_DEED = "Server.Items.BoneTableDeed" : BASE_ADDON_DEED, enc;
    construct: Some(bone_table_deed_new),
    family: Some(&BONE_TABLE_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `BoneThroneComponent()`.
pub fn bone_throne_component_new(w: &mut World) -> ItemId {
    addon::new_component(w, &BONE_THRONE_COMPONENT, 0x2A58)
}
leaf! {
    /// `Server.Items.BoneThroneComponent`.
    BONE_THRONE_COMPONENT = "Server.Items.BoneThroneComponent" : ADDON_COMPONENT, enc;
    on_move_over: |w, id, m| addon::move_over_sound(w, id, m, 0x54B, 0x54D),
    label_number: crate::class_props::label_number,
}

/// `BoneThroneAddon()`.
pub fn bone_throne_addon_new(w: &mut World) -> ItemId {
    let id = addon::new_addon(w, &BONE_THRONE_ADDON);
    addon::add_components(w, id, &[(Comp::Custom(bone_throne_component_new), 0, 0, 0)]);
    id
}
static BONE_THRONE_ADDON_CLASS: AddonClass = AddonClass {
    deed: Some(bone_throne_deed_new),
    ..BASE_ADDON_CLASS
};
leaf! {
    /// `Server.Items.BoneThroneAddon`.
    BONE_THRONE_ADDON = "Server.Items.BoneThroneAddon" : BASE_ADDON, enc;
    construct: Some(bone_throne_addon_new),
    family: Some(&BONE_THRONE_ADDON_CLASS),
}

/// `BoneThroneDeed()`.
pub fn bone_throne_deed_new(w: &mut World) -> ItemId {
    let id = addon::new_deed(w, &BONE_THRONE_DEED);
    ops::set_loot_type(w, id, LootType::Blessed);
    id
}
static BONE_THRONE_DEED_CLASS: DeedClass = DeedClass {
    addon: |w, _deed| bone_throne_addon_new(w),
    select_direction: None,
};
leaf! {
    /// `Server.Items.BoneThroneDeed`.
    BONE_THRONE_DEED = "Server.Items.BoneThroneDeed" : BASE_ADDON_DEED, enc;
    construct: Some(bone_throne_deed_new),
    family: Some(&BONE_THRONE_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `MountedPixieBlueComponent()`.
pub fn mounted_pixie_blue_component_new(w: &mut World) -> ItemId {
    addon::new_component(w, &MOUNTED_PIXIE_BLUE_COMPONENT, 0x2A75)
}
leaf! {
    /// `Server.Items.MountedPixieBlueComponent`.
    MOUNTED_PIXIE_BLUE_COMPONENT = "Server.Items.MountedPixieBlueComponent" : ADDON_COMPONENT, enc;
    on_double_click: |w, id, from| addon::reach_sound(w, id, from, 0x55C, 0x55E),
    label_number: crate::class_props::label_number,
}

/// `MountedPixieBlueAddon()`.
pub fn mounted_pixie_blue_addon_new(w: &mut World) -> ItemId {
    let id = addon::new_addon(w, &MOUNTED_PIXIE_BLUE_ADDON);
    addon::add_components(
        w,
        id,
        &[(Comp::Custom(mounted_pixie_blue_component_new), 0, 0, 0)],
    );
    id
}
static MOUNTED_PIXIE_BLUE_ADDON_CLASS: AddonClass = AddonClass {
    deed: Some(mounted_pixie_blue_deed_new),
    ..BASE_ADDON_CLASS
};
leaf! {
    /// `Server.Items.MountedPixieBlueAddon`.
    MOUNTED_PIXIE_BLUE_ADDON = "Server.Items.MountedPixieBlueAddon" : BASE_ADDON, enc;
    family: Some(&MOUNTED_PIXIE_BLUE_ADDON_CLASS),
}

/// `MountedPixieBlueDeed()`.
pub fn mounted_pixie_blue_deed_new(w: &mut World) -> ItemId {
    let id = addon::new_deed(w, &MOUNTED_PIXIE_BLUE_DEED);
    ops::set_loot_type(w, id, LootType::Blessed);
    id
}
static MOUNTED_PIXIE_BLUE_DEED_CLASS: DeedClass = DeedClass {
    addon: |w, _deed| mounted_pixie_blue_addon_new(w),
    select_direction: None,
};
leaf! {
    /// `Server.Items.MountedPixieBlueDeed`.
    MOUNTED_PIXIE_BLUE_DEED = "Server.Items.MountedPixieBlueDeed" : BASE_ADDON_DEED, enc;
    construct: Some(mounted_pixie_blue_deed_new),
    family: Some(&MOUNTED_PIXIE_BLUE_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `MountedPixieGreenComponent()`.
pub fn mounted_pixie_green_component_new(w: &mut World) -> ItemId {
    addon::new_component(w, &MOUNTED_PIXIE_GREEN_COMPONENT, 0x2A71)
}
leaf! {
    /// `Server.Items.MountedPixieGreenComponent`.
    MOUNTED_PIXIE_GREEN_COMPONENT = "Server.Items.MountedPixieGreenComponent" : ADDON_COMPONENT, enc;
    on_double_click: |w, id, from| addon::reach_sound(w, id, from, 0x554, 0x557),
    label_number: crate::class_props::label_number,
}

/// `MountedPixieGreenAddon()`.
pub fn mounted_pixie_green_addon_new(w: &mut World) -> ItemId {
    let id = addon::new_addon(w, &MOUNTED_PIXIE_GREEN_ADDON);
    addon::add_components(
        w,
        id,
        &[(Comp::Custom(mounted_pixie_green_component_new), 0, 0, 0)],
    );
    id
}
static MOUNTED_PIXIE_GREEN_ADDON_CLASS: AddonClass = AddonClass {
    deed: Some(mounted_pixie_green_deed_new),
    ..BASE_ADDON_CLASS
};
leaf! {
    /// `Server.Items.MountedPixieGreenAddon`.
    MOUNTED_PIXIE_GREEN_ADDON = "Server.Items.MountedPixieGreenAddon" : BASE_ADDON, enc;
    family: Some(&MOUNTED_PIXIE_GREEN_ADDON_CLASS),
}

/// `MountedPixieGreenDeed()`.
pub fn mounted_pixie_green_deed_new(w: &mut World) -> ItemId {
    let id = addon::new_deed(w, &MOUNTED_PIXIE_GREEN_DEED);
    ops::set_loot_type(w, id, LootType::Blessed);
    id
}
static MOUNTED_PIXIE_GREEN_DEED_CLASS: DeedClass = DeedClass {
    addon: |w, _deed| mounted_pixie_green_addon_new(w),
    select_direction: None,
};
leaf! {
    /// `Server.Items.MountedPixieGreenDeed`.
    MOUNTED_PIXIE_GREEN_DEED = "Server.Items.MountedPixieGreenDeed" : BASE_ADDON_DEED, enc;
    construct: Some(mounted_pixie_green_deed_new),
    family: Some(&MOUNTED_PIXIE_GREEN_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `MountedPixieLimeComponent()`.
pub fn mounted_pixie_lime_component_new(w: &mut World) -> ItemId {
    addon::new_component(w, &MOUNTED_PIXIE_LIME_COMPONENT, 0x2A77)
}
leaf! {
    /// `Server.Items.MountedPixieLimeComponent`.
    MOUNTED_PIXIE_LIME_COMPONENT = "Server.Items.MountedPixieLimeComponent" : ADDON_COMPONENT, enc;
    on_double_click: |w, id, from| addon::reach_sound(w, id, from, 0x55F, 0x561),
    label_number: crate::class_props::label_number,
}

/// `MountedPixieLimeAddon()`.
pub fn mounted_pixie_lime_addon_new(w: &mut World) -> ItemId {
    let id = addon::new_addon(w, &MOUNTED_PIXIE_LIME_ADDON);
    addon::add_components(
        w,
        id,
        &[(Comp::Custom(mounted_pixie_lime_component_new), 0, 0, 0)],
    );
    id
}
static MOUNTED_PIXIE_LIME_ADDON_CLASS: AddonClass = AddonClass {
    deed: Some(mounted_pixie_lime_deed_new),
    ..BASE_ADDON_CLASS
};
leaf! {
    /// `Server.Items.MountedPixieLimeAddon`.
    MOUNTED_PIXIE_LIME_ADDON = "Server.Items.MountedPixieLimeAddon" : BASE_ADDON, enc;
    family: Some(&MOUNTED_PIXIE_LIME_ADDON_CLASS),
}

/// `MountedPixieLimeDeed()`.
pub fn mounted_pixie_lime_deed_new(w: &mut World) -> ItemId {
    let id = addon::new_deed(w, &MOUNTED_PIXIE_LIME_DEED);
    ops::set_loot_type(w, id, LootType::Blessed);
    id
}
static MOUNTED_PIXIE_LIME_DEED_CLASS: DeedClass = DeedClass {
    addon: |w, _deed| mounted_pixie_lime_addon_new(w),
    select_direction: None,
};
leaf! {
    /// `Server.Items.MountedPixieLimeDeed`.
    MOUNTED_PIXIE_LIME_DEED = "Server.Items.MountedPixieLimeDeed" : BASE_ADDON_DEED, enc;
    construct: Some(mounted_pixie_lime_deed_new),
    family: Some(&MOUNTED_PIXIE_LIME_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `MountedPixieOrangeComponent()`.
pub fn mounted_pixie_orange_component_new(w: &mut World) -> ItemId {
    addon::new_component(w, &MOUNTED_PIXIE_ORANGE_COMPONENT, 0x2A73)
}
leaf! {
    /// `Server.Items.MountedPixieOrangeComponent`.
    MOUNTED_PIXIE_ORANGE_COMPONENT = "Server.Items.MountedPixieOrangeComponent" : ADDON_COMPONENT, enc;
    on_double_click: |w, id, from| addon::reach_sound(w, id, from, 0x558, 0x55B),
    label_number: crate::class_props::label_number,
}

/// `MountedPixieOrangeAddon()`.
pub fn mounted_pixie_orange_addon_new(w: &mut World) -> ItemId {
    let id = addon::new_addon(w, &MOUNTED_PIXIE_ORANGE_ADDON);
    addon::add_components(
        w,
        id,
        &[(Comp::Custom(mounted_pixie_orange_component_new), 0, 0, 0)],
    );
    id
}
static MOUNTED_PIXIE_ORANGE_ADDON_CLASS: AddonClass = AddonClass {
    deed: Some(mounted_pixie_orange_deed_new),
    ..BASE_ADDON_CLASS
};
leaf! {
    /// `Server.Items.MountedPixieOrangeAddon`.
    MOUNTED_PIXIE_ORANGE_ADDON = "Server.Items.MountedPixieOrangeAddon" : BASE_ADDON, enc;
    family: Some(&MOUNTED_PIXIE_ORANGE_ADDON_CLASS),
}

/// `MountedPixieOrangeDeed()`.
pub fn mounted_pixie_orange_deed_new(w: &mut World) -> ItemId {
    let id = addon::new_deed(w, &MOUNTED_PIXIE_ORANGE_DEED);
    ops::set_loot_type(w, id, LootType::Blessed);
    id
}
static MOUNTED_PIXIE_ORANGE_DEED_CLASS: DeedClass = DeedClass {
    addon: |w, _deed| mounted_pixie_orange_addon_new(w),
    select_direction: None,
};
leaf! {
    /// `Server.Items.MountedPixieOrangeDeed`.
    MOUNTED_PIXIE_ORANGE_DEED = "Server.Items.MountedPixieOrangeDeed" : BASE_ADDON_DEED, enc;
    construct: Some(mounted_pixie_orange_deed_new),
    family: Some(&MOUNTED_PIXIE_ORANGE_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `MountedPixieWhiteComponent()`.
pub fn mounted_pixie_white_component_new(w: &mut World) -> ItemId {
    addon::new_component(w, &MOUNTED_PIXIE_WHITE_COMPONENT, 0x2A79)
}
leaf! {
    /// `Server.Items.MountedPixieWhiteComponent`.
    MOUNTED_PIXIE_WHITE_COMPONENT = "Server.Items.MountedPixieWhiteComponent" : ADDON_COMPONENT, enc;
    on_double_click: |w, id, from| addon::reach_sound(w, id, from, 0x562, 0x564),
    label_number: crate::class_props::label_number,
}

/// `MountedPixieWhiteAddon()`.
pub fn mounted_pixie_white_addon_new(w: &mut World) -> ItemId {
    let id = addon::new_addon(w, &MOUNTED_PIXIE_WHITE_ADDON);
    addon::add_components(
        w,
        id,
        &[(Comp::Custom(mounted_pixie_white_component_new), 0, 0, 0)],
    );
    id
}
static MOUNTED_PIXIE_WHITE_ADDON_CLASS: AddonClass = AddonClass {
    deed: Some(mounted_pixie_white_deed_new),
    ..BASE_ADDON_CLASS
};
leaf! {
    /// `Server.Items.MountedPixieWhiteAddon`.
    MOUNTED_PIXIE_WHITE_ADDON = "Server.Items.MountedPixieWhiteAddon" : BASE_ADDON, enc;
    family: Some(&MOUNTED_PIXIE_WHITE_ADDON_CLASS),
}

/// `MountedPixieWhiteDeed()`.
pub fn mounted_pixie_white_deed_new(w: &mut World) -> ItemId {
    let id = addon::new_deed(w, &MOUNTED_PIXIE_WHITE_DEED);
    ops::set_loot_type(w, id, LootType::Blessed);
    id
}
static MOUNTED_PIXIE_WHITE_DEED_CLASS: DeedClass = DeedClass {
    addon: |w, _deed| mounted_pixie_white_addon_new(w),
    select_direction: None,
};
leaf! {
    /// `Server.Items.MountedPixieWhiteDeed`.
    MOUNTED_PIXIE_WHITE_DEED = "Server.Items.MountedPixieWhiteDeed" : BASE_ADDON_DEED, enc;
    construct: Some(mounted_pixie_white_deed_new),
    family: Some(&MOUNTED_PIXIE_WHITE_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `AppleTrunkAddon()`.
pub fn apple_trunk_addon_new(w: &mut World) -> ItemId {
    let id = addon::new_addon(w, &APPLE_TRUNK_ADDON);
    addon::add_components(w, id, &[(Comp::Localized(3480, 1076785), 0, 0, 0)]);
    id
}
static APPLE_TRUNK_ADDON_CLASS: AddonClass = AddonClass {
    deed: Some(apple_trunk_deed_new),
    ..BASE_ADDON_CLASS
};
leaf! {
    /// `Server.Items.AppleTrunkAddon`.
    APPLE_TRUNK_ADDON = "Server.Items.AppleTrunkAddon" : BASE_ADDON, enc;
    construct: Some(apple_trunk_addon_new),
    family: Some(&APPLE_TRUNK_ADDON_CLASS),
}

/// `AppleTrunkDeed()`.
pub fn apple_trunk_deed_new(w: &mut World) -> ItemId {
    let id = addon::new_deed(w, &APPLE_TRUNK_DEED);
    ops::set_loot_type(w, id, LootType::Blessed);
    id
}
static APPLE_TRUNK_DEED_CLASS: DeedClass = DeedClass {
    addon: |w, _deed| apple_trunk_addon_new(w),
    select_direction: None,
};
leaf! {
    /// `Server.Items.AppleTrunkDeed`.
    APPLE_TRUNK_DEED = "Server.Items.AppleTrunkDeed" : BASE_ADDON_DEED, enc;
    construct: Some(apple_trunk_deed_new),
    family: Some(&APPLE_TRUNK_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `BlueDecorativeRugAddon()`.
pub fn blue_decorative_rug_addon_new(w: &mut World) -> ItemId {
    let id = addon::new_addon(w, &BLUE_DECORATIVE_RUG_ADDON);
    addon::add_components(
        w,
        id,
        &[
            (Comp::Localized(2770, 1076589), 1, 1, 0),
            (Comp::Localized(2771, 1076589), -1, -1, 0),
            (Comp::Localized(2772, 1076589), -1, 1, 0),
            (Comp::Localized(2773, 1076589), 1, -1, 0),
            (Comp::Localized(2774, 1076589), -1, 0, 0),
            (Comp::Localized(2775, 1076589), 0, -1, 0),
            (Comp::Localized(2776, 1076589), 1, 0, 0),
            (Comp::Localized(2777, 1076589), 0, 1, 0),
            (Comp::Localized(2769, 1076589), 0, 0, 0),
        ],
    );
    id
}
static BLUE_DECORATIVE_RUG_ADDON_CLASS: AddonClass = AddonClass {
    deed: Some(blue_decorative_rug_deed_new),
    ..BASE_ADDON_CLASS
};
leaf! {
    /// `Server.Items.BlueDecorativeRugAddon`.
    BLUE_DECORATIVE_RUG_ADDON = "Server.Items.BlueDecorativeRugAddon" : BASE_ADDON, enc;
    construct: Some(blue_decorative_rug_addon_new),
    family: Some(&BLUE_DECORATIVE_RUG_ADDON_CLASS),
}

/// `BlueDecorativeRugDeed()`.
pub fn blue_decorative_rug_deed_new(w: &mut World) -> ItemId {
    let id = addon::new_deed(w, &BLUE_DECORATIVE_RUG_DEED);
    ops::set_loot_type(w, id, LootType::Blessed);
    id
}
static BLUE_DECORATIVE_RUG_DEED_CLASS: DeedClass = DeedClass {
    addon: |w, _deed| blue_decorative_rug_addon_new(w),
    select_direction: None,
};
leaf! {
    /// `Server.Items.BlueDecorativeRugDeed`.
    BLUE_DECORATIVE_RUG_DEED = "Server.Items.BlueDecorativeRugDeed" : BASE_ADDON_DEED, enc;
    construct: Some(blue_decorative_rug_deed_new),
    family: Some(&BLUE_DECORATIVE_RUG_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `BlueFancyRugAddon()`.
pub fn blue_fancy_rug_addon_new(w: &mut World) -> ItemId {
    let id = addon::new_addon(w, &BLUE_FANCY_RUG_ADDON);
    addon::add_components(
        w,
        id,
        &[
            (Comp::Localized(2754, 1076273), 1, 1, 0),
            (Comp::Localized(2755, 1076273), -1, -1, 0),
            (Comp::Localized(2756, 1076273), -1, 1, 0),
            (Comp::Localized(2757, 1076273), 1, -1, 0),
            (Comp::Localized(2806, 1076273), -1, 0, 0),
            (Comp::Localized(2807, 1076273), 0, -1, 0),
            (Comp::Localized(2808, 1076273), 1, 0, 0),
            (Comp::Localized(2809, 1076273), 0, 1, 0),
            (Comp::Localized(2810, 1076273), 0, 0, 0),
        ],
    );
    id
}
static BLUE_FANCY_RUG_ADDON_CLASS: AddonClass = AddonClass {
    deed: Some(blue_fancy_rug_deed_new),
    ..BASE_ADDON_CLASS
};
leaf! {
    /// `Server.Items.BlueFancyRugAddon`.
    BLUE_FANCY_RUG_ADDON = "Server.Items.BlueFancyRugAddon" : BASE_ADDON, enc;
    construct: Some(blue_fancy_rug_addon_new),
    family: Some(&BLUE_FANCY_RUG_ADDON_CLASS),
}

/// `BlueFancyRugDeed()`.
pub fn blue_fancy_rug_deed_new(w: &mut World) -> ItemId {
    let id = addon::new_deed(w, &BLUE_FANCY_RUG_DEED);
    ops::set_loot_type(w, id, LootType::Blessed);
    id
}
static BLUE_FANCY_RUG_DEED_CLASS: DeedClass = DeedClass {
    addon: |w, _deed| blue_fancy_rug_addon_new(w),
    select_direction: None,
};
leaf! {
    /// `Server.Items.BlueFancyRugDeed`.
    BLUE_FANCY_RUG_DEED = "Server.Items.BlueFancyRugDeed" : BASE_ADDON_DEED, enc;
    construct: Some(blue_fancy_rug_deed_new),
    family: Some(&BLUE_FANCY_RUG_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `BluePlainRugAddon()`.
pub fn blue_plain_rug_addon_new(w: &mut World) -> ItemId {
    let id = addon::new_addon(w, &BLUE_PLAIN_RUG_ADDON);
    addon::add_components(
        w,
        id,
        &[
            (Comp::Localized(2754, 1076585), 1, 1, 0),
            (Comp::Localized(2755, 1076585), -1, -1, 0),
            (Comp::Localized(2756, 1076585), -1, 1, 0),
            (Comp::Localized(2757, 1076585), 1, -1, 0),
            (Comp::Localized(2806, 1076585), -1, 0, 0),
            (Comp::Localized(2807, 1076585), 0, -1, 0),
            (Comp::Localized(2808, 1076585), 1, 0, 0),
            (Comp::Localized(2809, 1076585), 0, 1, 0),
            (Comp::Localized(2752, 1076585), 0, 0, 0),
        ],
    );
    id
}
static BLUE_PLAIN_RUG_ADDON_CLASS: AddonClass = AddonClass {
    deed: Some(blue_plain_rug_deed_new),
    ..BASE_ADDON_CLASS
};
leaf! {
    /// `Server.Items.BluePlainRugAddon`.
    BLUE_PLAIN_RUG_ADDON = "Server.Items.BluePlainRugAddon" : BASE_ADDON, enc;
    construct: Some(blue_plain_rug_addon_new),
    family: Some(&BLUE_PLAIN_RUG_ADDON_CLASS),
}

/// `BluePlainRugDeed()`.
pub fn blue_plain_rug_deed_new(w: &mut World) -> ItemId {
    let id = addon::new_deed(w, &BLUE_PLAIN_RUG_DEED);
    ops::set_loot_type(w, id, LootType::Blessed);
    id
}
static BLUE_PLAIN_RUG_DEED_CLASS: DeedClass = DeedClass {
    addon: |w, _deed| blue_plain_rug_addon_new(w),
    select_direction: None,
};
leaf! {
    /// `Server.Items.BluePlainRugDeed`.
    BLUE_PLAIN_RUG_DEED = "Server.Items.BluePlainRugDeed" : BASE_ADDON_DEED, enc;
    construct: Some(blue_plain_rug_deed_new),
    family: Some(&BLUE_PLAIN_RUG_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `BoilingCauldronAddon()`.
pub fn boiling_cauldron_addon_new(w: &mut World) -> ItemId {
    let id = addon_container::new_addon_container(w, &BOILING_CAULDRON_ADDON, 0x2068);
    addon_container::add_components(
        w,
        id,
        &[
            (CComp::Localized(4012, 1076267), 0, 0, 0),
            (CComp::Localized(2416, 1076267), 0, 0, 8),
        ],
    );
    id
}
static BOILING_CAULDRON_ADDON_CLASS: ContainerAddonClass = ContainerAddonClass {
    deed: Some(boiling_cauldron_deed_new),
    ..BASE_ADDON_CONTAINER_CLASS
};
leaf! {
    /// `Server.Items.BoilingCauldronAddon`.
    BOILING_CAULDRON_ADDON = "Server.Items.BoilingCauldronAddon" : BASE_ADDON_CONTAINER, enc;
    construct: Some(boiling_cauldron_addon_new),
    family: Some(&BOILING_CAULDRON_ADDON_CLASS),
    label_number: crate::class_props::label_number,
    default_gump_id: crate::class_props::default_gump_id,
    default_drop_sound: crate::class_props::default_drop_sound,
}

/// `BoilingCauldronDeed()`.
pub fn boiling_cauldron_deed_new(w: &mut World) -> ItemId {
    let id = addon_container::new_deed(w, &BOILING_CAULDRON_DEED);
    ops::set_loot_type(w, id, LootType::Blessed);
    id
}
static BOILING_CAULDRON_DEED_CLASS: ContainerDeedClass = ContainerDeedClass {
    addon: |w, _deed| boiling_cauldron_addon_new(w),
    select_direction: None,
};
leaf! {
    /// `Server.Items.BoilingCauldronDeed`.
    BOILING_CAULDRON_DEED = "Server.Items.BoilingCauldronDeed" : BASE_ADDON_CONTAINER_DEED, enc;
    construct: Some(boiling_cauldron_deed_new),
    family: Some(&BOILING_CAULDRON_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `CherryBlossomTreeAddon()`.
pub fn cherry_blossom_tree_addon_new(w: &mut World) -> ItemId {
    let id = addon::new_addon(w, &CHERRY_BLOSSOM_TREE_ADDON);
    addon::add_components(
        w,
        id,
        &[
            (Comp::Localized(9966, 1076268), 0, 0, 0),
            (Comp::Localized(12578, 1076268), 0, 0, 0),
        ],
    );
    id
}
static CHERRY_BLOSSOM_TREE_ADDON_CLASS: AddonClass = AddonClass {
    deed: Some(cherry_blossom_tree_deed_new),
    ..BASE_ADDON_CLASS
};
leaf! {
    /// `Server.Items.CherryBlossomTreeAddon`.
    CHERRY_BLOSSOM_TREE_ADDON = "Server.Items.CherryBlossomTreeAddon" : BASE_ADDON, enc;
    construct: Some(cherry_blossom_tree_addon_new),
    family: Some(&CHERRY_BLOSSOM_TREE_ADDON_CLASS),
}

/// `CherryBlossomTreeDeed()`.
pub fn cherry_blossom_tree_deed_new(w: &mut World) -> ItemId {
    let id = addon::new_deed(w, &CHERRY_BLOSSOM_TREE_DEED);
    ops::set_loot_type(w, id, LootType::Blessed);
    id
}
static CHERRY_BLOSSOM_TREE_DEED_CLASS: DeedClass = DeedClass {
    addon: |w, _deed| cherry_blossom_tree_addon_new(w),
    select_direction: None,
};
leaf! {
    /// `Server.Items.CherryBlossomTreeDeed`.
    CHERRY_BLOSSOM_TREE_DEED = "Server.Items.CherryBlossomTreeDeed" : BASE_ADDON_DEED, enc;
    construct: Some(cherry_blossom_tree_deed_new),
    family: Some(&CHERRY_BLOSSOM_TREE_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `CherryBlossomTrunkAddon()`.
pub fn cherry_blossom_trunk_addon_new(w: &mut World) -> ItemId {
    let id = addon::new_addon(w, &CHERRY_BLOSSOM_TRUNK_ADDON);
    addon::add_components(w, id, &[(Comp::Localized(9966, 1076784), 0, 0, 0)]);
    id
}
static CHERRY_BLOSSOM_TRUNK_ADDON_CLASS: AddonClass = AddonClass {
    deed: Some(cherry_blossom_trunk_deed_new),
    ..BASE_ADDON_CLASS
};
leaf! {
    /// `Server.Items.CherryBlossomTrunkAddon`.
    CHERRY_BLOSSOM_TRUNK_ADDON = "Server.Items.CherryBlossomTrunkAddon" : BASE_ADDON, enc;
    construct: Some(cherry_blossom_trunk_addon_new),
    family: Some(&CHERRY_BLOSSOM_TRUNK_ADDON_CLASS),
}

/// `CherryBlossomTrunkDeed()`.
pub fn cherry_blossom_trunk_deed_new(w: &mut World) -> ItemId {
    let id = addon::new_deed(w, &CHERRY_BLOSSOM_TRUNK_DEED);
    ops::set_loot_type(w, id, LootType::Blessed);
    id
}
static CHERRY_BLOSSOM_TRUNK_DEED_CLASS: DeedClass = DeedClass {
    addon: |w, _deed| cherry_blossom_trunk_addon_new(w),
    select_direction: None,
};
leaf! {
    /// `Server.Items.CherryBlossomTrunkDeed`.
    CHERRY_BLOSSOM_TRUNK_DEED = "Server.Items.CherryBlossomTrunkDeed" : BASE_ADDON_DEED, enc;
    construct: Some(cherry_blossom_trunk_deed_new),
    family: Some(&CHERRY_BLOSSOM_TRUNK_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `CinnamonFancyRugAddon()`.
pub fn cinnamon_fancy_rug_addon_new(w: &mut World) -> ItemId {
    let id = addon::new_addon(w, &CINNAMON_FANCY_RUG_ADDON);
    addon::add_components(
        w,
        id,
        &[
            (Comp::Localized(2787, 1076587), 1, 1, 0),
            (Comp::Localized(2788, 1076587), -1, -1, 0),
            (Comp::Localized(2789, 1076587), -1, 1, 0),
            (Comp::Localized(2790, 1076587), 1, -1, 0),
            (Comp::Localized(2791, 1076587), -1, 0, 0),
            (Comp::Localized(2792, 1076587), 0, -1, 0),
            (Comp::Localized(2793, 1076587), 1, 0, 0),
            (Comp::Localized(2794, 1076587), 0, 1, 0),
            (Comp::Localized(2795, 1076587), 0, 0, 0),
        ],
    );
    id
}
static CINNAMON_FANCY_RUG_ADDON_CLASS: AddonClass = AddonClass {
    deed: Some(cinnamon_fancy_rug_deed_new),
    ..BASE_ADDON_CLASS
};
leaf! {
    /// `Server.Items.CinnamonFancyRugAddon`.
    CINNAMON_FANCY_RUG_ADDON = "Server.Items.CinnamonFancyRugAddon" : BASE_ADDON, enc;
    construct: Some(cinnamon_fancy_rug_addon_new),
    family: Some(&CINNAMON_FANCY_RUG_ADDON_CLASS),
}

/// `CinnamonFancyRugDeed()`.
pub fn cinnamon_fancy_rug_deed_new(w: &mut World) -> ItemId {
    let id = addon::new_deed(w, &CINNAMON_FANCY_RUG_DEED);
    ops::set_loot_type(w, id, LootType::Blessed);
    id
}
static CINNAMON_FANCY_RUG_DEED_CLASS: DeedClass = DeedClass {
    addon: |w, _deed| cinnamon_fancy_rug_addon_new(w),
    select_direction: None,
};
leaf! {
    /// `Server.Items.CinnamonFancyRugDeed`.
    CINNAMON_FANCY_RUG_DEED = "Server.Items.CinnamonFancyRugDeed" : BASE_ADDON_DEED, enc;
    construct: Some(cinnamon_fancy_rug_deed_new),
    family: Some(&CINNAMON_FANCY_RUG_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `GoldenDecorativeRugAddon()`.
pub fn golden_decorative_rug_addon_new(w: &mut World) -> ItemId {
    let id = addon::new_addon(w, &GOLDEN_DECORATIVE_RUG_ADDON);
    addon::add_components(
        w,
        id,
        &[
            (Comp::Localized(2779, 1076586), 1, 1, 0),
            (Comp::Localized(2780, 1076586), -1, -1, 0),
            (Comp::Localized(2781, 1076586), -1, 1, 0),
            (Comp::Localized(2782, 1076586), 1, -1, 0),
            (Comp::Localized(2783, 1076586), -1, 0, 0),
            (Comp::Localized(2784, 1076586), 0, -1, 0),
            (Comp::Localized(2785, 1076586), 1, 0, 0),
            (Comp::Localized(2786, 1076586), 0, 1, 0),
            (Comp::Localized(2778, 1076586), 0, 0, 0),
        ],
    );
    id
}
static GOLDEN_DECORATIVE_RUG_ADDON_CLASS: AddonClass = AddonClass {
    deed: Some(golden_decorative_rug_deed_new),
    ..BASE_ADDON_CLASS
};
leaf! {
    /// `Server.Items.GoldenDecorativeRugAddon`.
    GOLDEN_DECORATIVE_RUG_ADDON = "Server.Items.GoldenDecorativeRugAddon" : BASE_ADDON, enc;
    construct: Some(golden_decorative_rug_addon_new),
    family: Some(&GOLDEN_DECORATIVE_RUG_ADDON_CLASS),
}

/// `GoldenDecorativeRugDeed()`.
pub fn golden_decorative_rug_deed_new(w: &mut World) -> ItemId {
    let id = addon::new_deed(w, &GOLDEN_DECORATIVE_RUG_DEED);
    ops::set_loot_type(w, id, LootType::Blessed);
    id
}
static GOLDEN_DECORATIVE_RUG_DEED_CLASS: DeedClass = DeedClass {
    addon: |w, _deed| golden_decorative_rug_addon_new(w),
    select_direction: None,
};
leaf! {
    /// `Server.Items.GoldenDecorativeRugDeed`.
    GOLDEN_DECORATIVE_RUG_DEED = "Server.Items.GoldenDecorativeRugDeed" : BASE_ADDON_DEED, enc;
    construct: Some(golden_decorative_rug_deed_new),
    family: Some(&GOLDEN_DECORATIVE_RUG_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `GuillotineComponent()`.
pub fn guillotine_component_new(w: &mut World) -> ItemId {
    addon::new_component(w, &GUILLOTINE_COMPONENT, 0x125E)
}
leaf! {
    /// `Server.Items.GuillotineComponent`.
    GUILLOTINE_COMPONENT = "Server.Items.GuillotineComponent" : ADDON_COMPONENT, enc;
    label_number: crate::class_props::label_number,
}

/// `HangingAxesAddon(bool east)`.
pub fn hanging_axes_addon_new(w: &mut World, east: bool) -> ItemId {
    let id = addon::new_addon(w, &HANGING_AXES_ADDON);
    if east {
        addon::add_components(
            w,
            id,
            &[
                (Comp::Localized(5482, 1076271), 0, 0, 0),
                (Comp::Localized(5483, 1076271), 0, -1, 0),
            ],
        );
    } else {
        addon::add_components(
            w,
            id,
            &[
                (Comp::Localized(5480, 1076271), 0, 0, 0),
                (Comp::Localized(5481, 1076271), 1, 0, 0),
            ],
        );
    }
    id
}
static HANGING_AXES_ADDON_CLASS: AddonClass = AddonClass {
    deed: Some(hanging_axes_deed_new),
    ..BASE_ADDON_CLASS
};
leaf! {
    /// `Server.Items.HangingAxesAddon`.
    HANGING_AXES_ADDON = "Server.Items.HangingAxesAddon" : BASE_ADDON, enc;
    family: Some(&HANGING_AXES_ADDON_CLASS),
}

/// `HangingAxesDeed()`.
pub fn hanging_axes_deed_new(w: &mut World) -> ItemId {
    let id = addon::new_deed(w, &HANGING_AXES_DEED);
    ops::set_loot_type(w, id, LootType::Blessed);
    id
}
static HANGING_AXES_DEED_CLASS: DeedClass = DeedClass {
    addon: |w, deed| hanging_axes_addon_new(w, addon::deed_east(w, deed)),
    select_direction: Some(1076745),
};
leaf! {
    /// `Server.Items.HangingAxesDeed`.
    HANGING_AXES_DEED = "Server.Items.HangingAxesDeed" : BASE_ADDON_DEED, enc;
    construct: Some(hanging_axes_deed_new),
    family: Some(&HANGING_AXES_DEED_CLASS),
    on_double_click: addon::direction_deed_on_double_click,
    label_number: crate::class_props::label_number,
}

/// `HangingSwordsAddon(bool east)`.
pub fn hanging_swords_addon_new(w: &mut World, east: bool) -> ItemId {
    let id = addon::new_addon(w, &HANGING_SWORDS_ADDON);
    if east {
        addon::add_components(
            w,
            id,
            &[
                (Comp::Localized(5478, 1076272), 0, 0, 0),
                (Comp::Localized(5479, 1076272), 0, -1, 0),
            ],
        );
    } else {
        addon::add_components(
            w,
            id,
            &[
                (Comp::Localized(5476, 1076272), 0, 0, 0),
                (Comp::Localized(5477, 1076272), 1, 0, 0),
            ],
        );
    }
    id
}
static HANGING_SWORDS_ADDON_CLASS: AddonClass = AddonClass {
    deed: Some(hanging_swords_deed_new),
    ..BASE_ADDON_CLASS
};
leaf! {
    /// `Server.Items.HangingSwordsAddon`.
    HANGING_SWORDS_ADDON = "Server.Items.HangingSwordsAddon" : BASE_ADDON, enc;
    family: Some(&HANGING_SWORDS_ADDON_CLASS),
}

/// `HangingSwordsDeed()`.
pub fn hanging_swords_deed_new(w: &mut World) -> ItemId {
    let id = addon::new_deed(w, &HANGING_SWORDS_DEED);
    ops::set_loot_type(w, id, LootType::Blessed);
    id
}
static HANGING_SWORDS_DEED_CLASS: DeedClass = DeedClass {
    addon: |w, deed| hanging_swords_addon_new(w, addon::deed_east(w, deed)),
    select_direction: Some(1076746),
};
leaf! {
    /// `Server.Items.HangingSwordsDeed`.
    HANGING_SWORDS_DEED = "Server.Items.HangingSwordsDeed" : BASE_ADDON_DEED, enc;
    construct: Some(hanging_swords_deed_new),
    family: Some(&HANGING_SWORDS_DEED_CLASS),
    on_double_click: addon::direction_deed_on_double_click,
    label_number: crate::class_props::label_number,
}

/// `LargeFishingNetComponent()`.
pub fn large_fishing_net_component_new(w: &mut World) -> ItemId {
    addon::new_component(w, &LARGE_FISHING_NET_COMPONENT, 0x3D8E)
}
leaf! {
    /// `Server.Items.LargeFishingNetComponent`.
    LARGE_FISHING_NET_COMPONENT = "Server.Items.LargeFishingNetComponent" : ADDON_COMPONENT, enc;
    label_number: crate::class_props::label_number,
}

/// `LargeFishingNetAddon()`.
pub fn large_fishing_net_addon_new(w: &mut World) -> ItemId {
    let id = addon::new_addon(w, &LARGE_FISHING_NET_ADDON);
    addon::add_components(
        w,
        id,
        &[(Comp::Custom(large_fishing_net_component_new), 0, 0, 0)],
    );
    id
}
static LARGE_FISHING_NET_ADDON_CLASS: AddonClass = AddonClass {
    deed: Some(large_fishing_net_deed_new),
    ..BASE_ADDON_CLASS
};
leaf! {
    /// `Server.Items.LargeFishingNetAddon`.
    LARGE_FISHING_NET_ADDON = "Server.Items.LargeFishingNetAddon" : BASE_ADDON, enc;
    construct: Some(large_fishing_net_addon_new),
    family: Some(&LARGE_FISHING_NET_ADDON_CLASS),
}

/// `LargeFishingNetDeed()`.
pub fn large_fishing_net_deed_new(w: &mut World) -> ItemId {
    let id = addon::new_deed(w, &LARGE_FISHING_NET_DEED);
    ops::set_loot_type(w, id, LootType::Blessed);
    id
}
static LARGE_FISHING_NET_DEED_CLASS: DeedClass = DeedClass {
    addon: |w, _deed| large_fishing_net_addon_new(w),
    select_direction: None,
};
leaf! {
    /// `Server.Items.LargeFishingNetDeed`.
    LARGE_FISHING_NET_DEED = "Server.Items.LargeFishingNetDeed" : BASE_ADDON_DEED, enc;
    construct: Some(large_fishing_net_deed_new),
    family: Some(&LARGE_FISHING_NET_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `PeachTrunkAddon()`.
pub fn peach_trunk_addon_new(w: &mut World) -> ItemId {
    let id = addon::new_addon(w, &PEACH_TRUNK_ADDON);
    addon::add_components(w, id, &[(Comp::Localized(3484, 1076786), 0, 0, 0)]);
    id
}
static PEACH_TRUNK_ADDON_CLASS: AddonClass = AddonClass {
    deed: Some(peach_trunk_deed_new),
    ..BASE_ADDON_CLASS
};
leaf! {
    /// `Server.Items.PeachTrunkAddon`.
    PEACH_TRUNK_ADDON = "Server.Items.PeachTrunkAddon" : BASE_ADDON, enc;
    construct: Some(peach_trunk_addon_new),
    family: Some(&PEACH_TRUNK_ADDON_CLASS),
}

/// `PeachTrunkDeed()`.
pub fn peach_trunk_deed_new(w: &mut World) -> ItemId {
    let id = addon::new_deed(w, &PEACH_TRUNK_DEED);
    ops::set_loot_type(w, id, LootType::Blessed);
    id
}
static PEACH_TRUNK_DEED_CLASS: DeedClass = DeedClass {
    addon: |w, _deed| peach_trunk_addon_new(w),
    select_direction: None,
};
leaf! {
    /// `Server.Items.PeachTrunkDeed`.
    PEACH_TRUNK_DEED = "Server.Items.PeachTrunkDeed" : BASE_ADDON_DEED, enc;
    construct: Some(peach_trunk_deed_new),
    family: Some(&PEACH_TRUNK_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `PinkFancyRugAddon()`.
pub fn pink_fancy_rug_addon_new(w: &mut World) -> ItemId {
    let id = addon::new_addon(w, &PINK_FANCY_RUG_ADDON);
    addon::add_components(
        w,
        id,
        &[
            (Comp::Localized(2798, 1076590), 1, 1, 0),
            (Comp::Localized(2799, 1076590), -1, -1, 0),
            (Comp::Localized(2800, 1076590), -1, 1, 0),
            (Comp::Localized(2801, 1076590), 1, -1, 0),
            (Comp::Localized(2802, 1076590), -1, 0, 0),
            (Comp::Localized(2803, 1076590), 0, -1, 0),
            (Comp::Localized(2804, 1076590), 1, 0, 0),
            (Comp::Localized(2805, 1076590), 0, 1, 0),
            (Comp::Localized(2796, 1076590), 0, 0, 0),
        ],
    );
    id
}
static PINK_FANCY_RUG_ADDON_CLASS: AddonClass = AddonClass {
    deed: Some(pink_fancy_rug_deed_new),
    ..BASE_ADDON_CLASS
};
leaf! {
    /// `Server.Items.PinkFancyRugAddon`.
    PINK_FANCY_RUG_ADDON = "Server.Items.PinkFancyRugAddon" : BASE_ADDON, enc;
    construct: Some(pink_fancy_rug_addon_new),
    family: Some(&PINK_FANCY_RUG_ADDON_CLASS),
}

/// `PinkFancyRugDeed()`.
pub fn pink_fancy_rug_deed_new(w: &mut World) -> ItemId {
    let id = addon::new_deed(w, &PINK_FANCY_RUG_DEED);
    ops::set_loot_type(w, id, LootType::Blessed);
    id
}
static PINK_FANCY_RUG_DEED_CLASS: DeedClass = DeedClass {
    addon: |w, _deed| pink_fancy_rug_addon_new(w),
    select_direction: None,
};
leaf! {
    /// `Server.Items.PinkFancyRugDeed`.
    PINK_FANCY_RUG_DEED = "Server.Items.PinkFancyRugDeed" : BASE_ADDON_DEED, enc;
    construct: Some(pink_fancy_rug_deed_new),
    family: Some(&PINK_FANCY_RUG_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `RedPlainRugAddon()`.
pub fn red_plain_rug_addon_new(w: &mut World) -> ItemId {
    let id = addon::new_addon(w, &RED_PLAIN_RUG_ADDON);
    addon::add_components(
        w,
        id,
        &[
            (Comp::Localized(2761, 1076588), 1, 1, 0),
            (Comp::Localized(2762, 1076588), -1, -1, 0),
            (Comp::Localized(2763, 1076588), -1, 1, 0),
            (Comp::Localized(2764, 1076588), 1, -1, 0),
            (Comp::Localized(2765, 1076588), -1, 0, 0),
            (Comp::Localized(2766, 1076588), 0, -1, 0),
            (Comp::Localized(2767, 1076588), 1, 0, 0),
            (Comp::Localized(2768, 1076588), 0, 1, 0),
            (Comp::Localized(2758, 1076588), 0, 0, 0),
        ],
    );
    id
}
static RED_PLAIN_RUG_ADDON_CLASS: AddonClass = AddonClass {
    deed: Some(red_plain_rug_deed_new),
    ..BASE_ADDON_CLASS
};
leaf! {
    /// `Server.Items.RedPlainRugAddon`.
    RED_PLAIN_RUG_ADDON = "Server.Items.RedPlainRugAddon" : BASE_ADDON, enc;
    construct: Some(red_plain_rug_addon_new),
    family: Some(&RED_PLAIN_RUG_ADDON_CLASS),
}

/// `RedPlainRugDeed()`.
pub fn red_plain_rug_deed_new(w: &mut World) -> ItemId {
    let id = addon::new_deed(w, &RED_PLAIN_RUG_DEED);
    ops::set_loot_type(w, id, LootType::Blessed);
    id
}
static RED_PLAIN_RUG_DEED_CLASS: DeedClass = DeedClass {
    addon: |w, _deed| red_plain_rug_addon_new(w),
    select_direction: None,
};
leaf! {
    /// `Server.Items.RedPlainRugDeed`.
    RED_PLAIN_RUG_DEED = "Server.Items.RedPlainRugDeed" : BASE_ADDON_DEED, enc;
    construct: Some(red_plain_rug_deed_new),
    family: Some(&RED_PLAIN_RUG_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `ScarecrowComponent()`.
pub fn scarecrow_component_new(w: &mut World) -> ItemId {
    addon::new_component(w, &SCARECROW_COMPONENT, 0x1E34)
}
leaf! {
    /// `Server.Items.ScarecrowComponent`.
    SCARECROW_COMPONENT = "Server.Items.ScarecrowComponent" : ADDON_COMPONENT, enc;
    label_number: crate::class_props::label_number,
}

/// `ScarecrowAddon()`.
pub fn scarecrow_addon_new(w: &mut World) -> ItemId {
    let id = addon::new_addon(w, &SCARECROW_ADDON);
    addon::add_components(w, id, &[(Comp::Custom(scarecrow_component_new), 0, 0, 0)]);
    id
}
static SCARECROW_ADDON_CLASS: AddonClass = AddonClass {
    deed: Some(scarecrow_deed_new),
    ..BASE_ADDON_CLASS
};
leaf! {
    /// `Server.Items.ScarecrowAddon`.
    SCARECROW_ADDON = "Server.Items.ScarecrowAddon" : BASE_ADDON, enc;
    construct: Some(scarecrow_addon_new),
    family: Some(&SCARECROW_ADDON_CLASS),
}

/// `ScarecrowDeed()`.
pub fn scarecrow_deed_new(w: &mut World) -> ItemId {
    let id = addon::new_deed(w, &SCARECROW_DEED);
    ops::set_loot_type(w, id, LootType::Blessed);
    id
}
static SCARECROW_DEED_CLASS: DeedClass = DeedClass {
    addon: |w, _deed| scarecrow_addon_new(w),
    select_direction: None,
};
leaf! {
    /// `Server.Items.ScarecrowDeed`.
    SCARECROW_DEED = "Server.Items.ScarecrowDeed" : BASE_ADDON_DEED, enc;
    construct: Some(scarecrow_deed_new),
    family: Some(&SCARECROW_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `SmallFishingNetComponent()`.
pub fn small_fishing_net_component_new(w: &mut World) -> ItemId {
    addon::new_component(w, &SMALL_FISHING_NET_COMPONENT, 0x1EA3)
}
leaf! {
    /// `Server.Items.SmallFishingNetComponent`.
    SMALL_FISHING_NET_COMPONENT = "Server.Items.SmallFishingNetComponent" : ADDON_COMPONENT, enc;
    label_number: crate::class_props::label_number,
}

/// `SmallFishingNetAddon()`.
pub fn small_fishing_net_addon_new(w: &mut World) -> ItemId {
    let id = addon::new_addon(w, &SMALL_FISHING_NET_ADDON);
    addon::add_components(
        w,
        id,
        &[(Comp::Custom(small_fishing_net_component_new), 0, 0, 0)],
    );
    id
}
static SMALL_FISHING_NET_ADDON_CLASS: AddonClass = AddonClass {
    deed: Some(small_fishing_net_deed_new),
    ..BASE_ADDON_CLASS
};
leaf! {
    /// `Server.Items.SmallFishingNetAddon`.
    SMALL_FISHING_NET_ADDON = "Server.Items.SmallFishingNetAddon" : BASE_ADDON, enc;
    construct: Some(small_fishing_net_addon_new),
    family: Some(&SMALL_FISHING_NET_ADDON_CLASS),
}

/// `SmallFishingNetDeed()`.
pub fn small_fishing_net_deed_new(w: &mut World) -> ItemId {
    let id = addon::new_deed(w, &SMALL_FISHING_NET_DEED);
    ops::set_loot_type(w, id, LootType::Blessed);
    id
}
static SMALL_FISHING_NET_DEED_CLASS: DeedClass = DeedClass {
    addon: |w, _deed| small_fishing_net_addon_new(w),
    select_direction: None,
};
leaf! {
    /// `Server.Items.SmallFishingNetDeed`.
    SMALL_FISHING_NET_DEED = "Server.Items.SmallFishingNetDeed" : BASE_ADDON_DEED, enc;
    construct: Some(small_fishing_net_deed_new),
    family: Some(&SMALL_FISHING_NET_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `StoneStatueAddon(bool east)`.
pub fn stone_statue_addon_new(w: &mut World, east: bool) -> ItemId {
    let id = addon::new_addon(w, &STONE_STATUE_ADDON);
    if east {
        addon::add_components(
            w,
            id,
            &[
                (Comp::Localized(5022, 1076284), 0, 0, 0),
                (Comp::Localized(5023, 1076284), -1, 0, 0),
                (Comp::Localized(5024, 1076284), 0, -1, 0),
            ],
        );
    } else {
        addon::add_components(
            w,
            id,
            &[
                (Comp::Localized(4767, 1076284), 0, 0, 0),
                (Comp::Localized(4768, 1076284), 0, -1, 0),
                (Comp::Localized(4769, 1076284), -1, 0, 0),
            ],
        );
    }
    id
}
static STONE_STATUE_ADDON_CLASS: AddonClass = AddonClass {
    deed: Some(stone_statue_deed_new),
    ..BASE_ADDON_CLASS
};
leaf! {
    /// `Server.Items.StoneStatueAddon`.
    STONE_STATUE_ADDON = "Server.Items.StoneStatueAddon" : BASE_ADDON, enc;
    family: Some(&STONE_STATUE_ADDON_CLASS),
}

/// `StoneStatueDeed()`.
pub fn stone_statue_deed_new(w: &mut World) -> ItemId {
    let id = addon::new_deed(w, &STONE_STATUE_DEED);
    ops::set_loot_type(w, id, LootType::Blessed);
    id
}
static STONE_STATUE_DEED_CLASS: DeedClass = DeedClass {
    addon: |w, deed| stone_statue_addon_new(w, addon::deed_east(w, deed)),
    select_direction: Some(1076579),
};
leaf! {
    /// `Server.Items.StoneStatueDeed`.
    STONE_STATUE_DEED = "Server.Items.StoneStatueDeed" : BASE_ADDON_DEED, enc;
    construct: Some(stone_statue_deed_new),
    family: Some(&STONE_STATUE_DEED_CLASS),
    on_double_click: addon::direction_deed_on_double_click,
    label_number: crate::class_props::label_number,
}

/// `SuitOfGoldArmorComponent()`.
pub fn suit_of_gold_armor_component_new(w: &mut World) -> ItemId {
    addon::new_component(w, &SUIT_OF_GOLD_ARMOR_COMPONENT, 0x3DAA)
}
leaf! {
    /// `Server.Items.SuitOfGoldArmorComponent`.
    SUIT_OF_GOLD_ARMOR_COMPONENT = "Server.Items.SuitOfGoldArmorComponent" : ADDON_COMPONENT, enc;
    label_number: crate::class_props::label_number,
}

/// `SuitOfGoldArmorAddon()`.
pub fn suit_of_gold_armor_addon_new(w: &mut World) -> ItemId {
    let id = addon::new_addon(w, &SUIT_OF_GOLD_ARMOR_ADDON);
    addon::add_components(
        w,
        id,
        &[(Comp::Custom(suit_of_gold_armor_component_new), 0, 0, 0)],
    );
    id
}
static SUIT_OF_GOLD_ARMOR_ADDON_CLASS: AddonClass = AddonClass {
    deed: Some(suit_of_gold_armor_deed_new),
    ..BASE_ADDON_CLASS
};
leaf! {
    /// `Server.Items.SuitOfGoldArmorAddon`.
    SUIT_OF_GOLD_ARMOR_ADDON = "Server.Items.SuitOfGoldArmorAddon" : BASE_ADDON, enc;
    construct: Some(suit_of_gold_armor_addon_new),
    family: Some(&SUIT_OF_GOLD_ARMOR_ADDON_CLASS),
}

/// `SuitOfGoldArmorDeed()`.
pub fn suit_of_gold_armor_deed_new(w: &mut World) -> ItemId {
    let id = addon::new_deed(w, &SUIT_OF_GOLD_ARMOR_DEED);
    ops::set_loot_type(w, id, LootType::Blessed);
    id
}
static SUIT_OF_GOLD_ARMOR_DEED_CLASS: DeedClass = DeedClass {
    addon: |w, _deed| suit_of_gold_armor_addon_new(w),
    select_direction: None,
};
leaf! {
    /// `Server.Items.SuitOfGoldArmorDeed`.
    SUIT_OF_GOLD_ARMOR_DEED = "Server.Items.SuitOfGoldArmorDeed" : BASE_ADDON_DEED, enc;
    construct: Some(suit_of_gold_armor_deed_new),
    family: Some(&SUIT_OF_GOLD_ARMOR_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `SuitOfSilverArmorComponent()`.
pub fn suit_of_silver_armor_component_new(w: &mut World) -> ItemId {
    addon::new_component(w, &SUIT_OF_SILVER_ARMOR_COMPONENT, 0x3D86)
}
leaf! {
    /// `Server.Items.SuitOfSilverArmorComponent`.
    SUIT_OF_SILVER_ARMOR_COMPONENT = "Server.Items.SuitOfSilverArmorComponent" : ADDON_COMPONENT, enc;
    label_number: crate::class_props::label_number,
}

/// `SuitOfSilverArmorAddon()`.
pub fn suit_of_silver_armor_addon_new(w: &mut World) -> ItemId {
    let id = addon::new_addon(w, &SUIT_OF_SILVER_ARMOR_ADDON);
    addon::add_components(
        w,
        id,
        &[(Comp::Custom(suit_of_silver_armor_component_new), 0, 0, 0)],
    );
    id
}
static SUIT_OF_SILVER_ARMOR_ADDON_CLASS: AddonClass = AddonClass {
    deed: Some(suit_of_silver_armor_deed_new),
    ..BASE_ADDON_CLASS
};
leaf! {
    /// `Server.Items.SuitOfSilverArmorAddon`.
    SUIT_OF_SILVER_ARMOR_ADDON = "Server.Items.SuitOfSilverArmorAddon" : BASE_ADDON, enc;
    construct: Some(suit_of_silver_armor_addon_new),
    family: Some(&SUIT_OF_SILVER_ARMOR_ADDON_CLASS),
}

/// `SuitOfSilverArmorDeed()`.
pub fn suit_of_silver_armor_deed_new(w: &mut World) -> ItemId {
    let id = addon::new_deed(w, &SUIT_OF_SILVER_ARMOR_DEED);
    ops::set_loot_type(w, id, LootType::Blessed);
    id
}
static SUIT_OF_SILVER_ARMOR_DEED_CLASS: DeedClass = DeedClass {
    addon: |w, _deed| suit_of_silver_armor_addon_new(w),
    select_direction: None,
};
leaf! {
    /// `Server.Items.SuitOfSilverArmorDeed`.
    SUIT_OF_SILVER_ARMOR_DEED = "Server.Items.SuitOfSilverArmorDeed" : BASE_ADDON_DEED, enc;
    construct: Some(suit_of_silver_armor_deed_new),
    family: Some(&SUIT_OF_SILVER_ARMOR_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `TableWithBlueClothAddon()`.
pub fn table_with_blue_cloth_addon_new(w: &mut World) -> ItemId {
    let id = addon::new_addon(w, &TABLE_WITH_BLUE_CLOTH_ADDON);
    addon::add_components(w, id, &[(Comp::Localized(4492, 1076276), 0, 0, 0)]);
    id
}
static TABLE_WITH_BLUE_CLOTH_ADDON_CLASS: AddonClass = AddonClass {
    deed: Some(table_with_blue_cloth_deed_new),
    ..BASE_ADDON_CLASS
};
leaf! {
    /// `Server.Items.TableWithBlueClothAddon`.
    TABLE_WITH_BLUE_CLOTH_ADDON = "Server.Items.TableWithBlueClothAddon" : BASE_ADDON, enc;
    construct: Some(table_with_blue_cloth_addon_new),
    family: Some(&TABLE_WITH_BLUE_CLOTH_ADDON_CLASS),
}

/// `TableWithBlueClothDeed()`.
pub fn table_with_blue_cloth_deed_new(w: &mut World) -> ItemId {
    let id = addon::new_deed(w, &TABLE_WITH_BLUE_CLOTH_DEED);
    ops::set_loot_type(w, id, LootType::Blessed);
    id
}
static TABLE_WITH_BLUE_CLOTH_DEED_CLASS: DeedClass = DeedClass {
    addon: |w, _deed| table_with_blue_cloth_addon_new(w),
    select_direction: None,
};
leaf! {
    /// `Server.Items.TableWithBlueClothDeed`.
    TABLE_WITH_BLUE_CLOTH_DEED = "Server.Items.TableWithBlueClothDeed" : BASE_ADDON_DEED, enc;
    construct: Some(table_with_blue_cloth_deed_new),
    family: Some(&TABLE_WITH_BLUE_CLOTH_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `TableWithOrangeClothAddon()`.
pub fn table_with_orange_cloth_addon_new(w: &mut World) -> ItemId {
    let id = addon::new_addon(w, &TABLE_WITH_ORANGE_CLOTH_ADDON);
    addon::add_components(w, id, &[(Comp::Localized(4494, 1076278), 0, 0, 0)]);
    id
}
static TABLE_WITH_ORANGE_CLOTH_ADDON_CLASS: AddonClass = AddonClass {
    deed: Some(table_with_orange_cloth_deed_new),
    ..BASE_ADDON_CLASS
};
leaf! {
    /// `Server.Items.TableWithOrangeClothAddon`.
    TABLE_WITH_ORANGE_CLOTH_ADDON = "Server.Items.TableWithOrangeClothAddon" : BASE_ADDON, enc;
    construct: Some(table_with_orange_cloth_addon_new),
    family: Some(&TABLE_WITH_ORANGE_CLOTH_ADDON_CLASS),
}

/// `TableWithOrangeClothDeed()`.
pub fn table_with_orange_cloth_deed_new(w: &mut World) -> ItemId {
    let id = addon::new_deed(w, &TABLE_WITH_ORANGE_CLOTH_DEED);
    ops::set_loot_type(w, id, LootType::Blessed);
    id
}
static TABLE_WITH_ORANGE_CLOTH_DEED_CLASS: DeedClass = DeedClass {
    addon: |w, _deed| table_with_orange_cloth_addon_new(w),
    select_direction: None,
};
leaf! {
    /// `Server.Items.TableWithOrangeClothDeed`.
    TABLE_WITH_ORANGE_CLOTH_DEED = "Server.Items.TableWithOrangeClothDeed" : BASE_ADDON_DEED, enc;
    construct: Some(table_with_orange_cloth_deed_new),
    family: Some(&TABLE_WITH_ORANGE_CLOTH_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `TableWithPurpleClothAddon()`.
pub fn table_with_purple_cloth_addon_new(w: &mut World) -> ItemId {
    let id = addon::new_addon(w, &TABLE_WITH_PURPLE_CLOTH_ADDON);
    addon::add_components(w, id, &[(Comp::Localized(4491, 1076275), 0, 0, 0)]);
    id
}
static TABLE_WITH_PURPLE_CLOTH_ADDON_CLASS: AddonClass = AddonClass {
    deed: Some(table_with_purple_cloth_deed_new),
    ..BASE_ADDON_CLASS
};
leaf! {
    /// `Server.Items.TableWithPurpleClothAddon`.
    TABLE_WITH_PURPLE_CLOTH_ADDON = "Server.Items.TableWithPurpleClothAddon" : BASE_ADDON, enc;
    construct: Some(table_with_purple_cloth_addon_new),
    family: Some(&TABLE_WITH_PURPLE_CLOTH_ADDON_CLASS),
}

/// `TableWithPurpleClothDeed()`.
pub fn table_with_purple_cloth_deed_new(w: &mut World) -> ItemId {
    let id = addon::new_deed(w, &TABLE_WITH_PURPLE_CLOTH_DEED);
    ops::set_loot_type(w, id, LootType::Blessed);
    id
}
static TABLE_WITH_PURPLE_CLOTH_DEED_CLASS: DeedClass = DeedClass {
    addon: |w, _deed| table_with_purple_cloth_addon_new(w),
    select_direction: None,
};
leaf! {
    /// `Server.Items.TableWithPurpleClothDeed`.
    TABLE_WITH_PURPLE_CLOTH_DEED = "Server.Items.TableWithPurpleClothDeed" : BASE_ADDON_DEED, enc;
    construct: Some(table_with_purple_cloth_deed_new),
    family: Some(&TABLE_WITH_PURPLE_CLOTH_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `TableWithRedClothAddon()`.
pub fn table_with_red_cloth_addon_new(w: &mut World) -> ItemId {
    let id = addon::new_addon(w, &TABLE_WITH_RED_CLOTH_ADDON);
    addon::add_components(w, id, &[(Comp::Localized(4493, 1076277), 0, 0, 0)]);
    id
}
static TABLE_WITH_RED_CLOTH_ADDON_CLASS: AddonClass = AddonClass {
    deed: Some(table_with_red_cloth_deed_new),
    ..BASE_ADDON_CLASS
};
leaf! {
    /// `Server.Items.TableWithRedClothAddon`.
    TABLE_WITH_RED_CLOTH_ADDON = "Server.Items.TableWithRedClothAddon" : BASE_ADDON, enc;
    construct: Some(table_with_red_cloth_addon_new),
    family: Some(&TABLE_WITH_RED_CLOTH_ADDON_CLASS),
}

/// `TableWithRedClothDeed()`.
pub fn table_with_red_cloth_deed_new(w: &mut World) -> ItemId {
    let id = addon::new_deed(w, &TABLE_WITH_RED_CLOTH_DEED);
    ops::set_loot_type(w, id, LootType::Blessed);
    id
}
static TABLE_WITH_RED_CLOTH_DEED_CLASS: DeedClass = DeedClass {
    addon: |w, _deed| table_with_red_cloth_addon_new(w),
    select_direction: None,
};
leaf! {
    /// `Server.Items.TableWithRedClothDeed`.
    TABLE_WITH_RED_CLOTH_DEED = "Server.Items.TableWithRedClothDeed" : BASE_ADDON_DEED, enc;
    construct: Some(table_with_red_cloth_deed_new),
    family: Some(&TABLE_WITH_RED_CLOTH_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `UnmadeBedAddon(bool east)`.
pub fn unmade_bed_addon_new(w: &mut World, east: bool) -> ItemId {
    let id = addon::new_addon(w, &UNMADE_BED_ADDON);
    if east {
        addon::add_components(
            w,
            id,
            &[
                (Comp::Localized(2700, 1076279), 0, 0, 0),
                (Comp::Localized(2701, 1076279), 0, -1, 0),
                (Comp::Localized(2704, 1076279), -1, 0, 0),
                (Comp::Localized(2705, 1076279), -1, -1, 0),
            ],
        );
    } else {
        addon::add_components(
            w,
            id,
            &[
                (Comp::Localized(3504, 1076279), 0, 0, 0),
                (Comp::Localized(3505, 1076279), -1, 0, 0),
                (Comp::Localized(3508, 1076279), 0, -1, 0),
                (Comp::Localized(3509, 1076279), -1, -1, 0),
            ],
        );
    }
    id
}
static UNMADE_BED_ADDON_CLASS: AddonClass = AddonClass {
    deed: Some(unmade_bed_deed_new),
    ..BASE_ADDON_CLASS
};
leaf! {
    /// `Server.Items.UnmadeBedAddon`.
    UNMADE_BED_ADDON = "Server.Items.UnmadeBedAddon" : BASE_ADDON, enc;
    family: Some(&UNMADE_BED_ADDON_CLASS),
}

/// `UnmadeBedDeed()`.
pub fn unmade_bed_deed_new(w: &mut World) -> ItemId {
    let id = addon::new_deed(w, &UNMADE_BED_DEED);
    ops::set_loot_type(w, id, LootType::Blessed);
    id
}
static UNMADE_BED_DEED_CLASS: DeedClass = DeedClass {
    addon: |w, deed| unmade_bed_addon_new(w, addon::deed_east(w, deed)),
    select_direction: Some(1076580),
};
leaf! {
    /// `Server.Items.UnmadeBedDeed`.
    UNMADE_BED_DEED = "Server.Items.UnmadeBedDeed" : BASE_ADDON_DEED, enc;
    construct: Some(unmade_bed_deed_new),
    family: Some(&UNMADE_BED_DEED_CLASS),
    on_double_click: addon::direction_deed_on_double_click,
    label_number: crate::class_props::label_number,
}

/// `VanityAddon(bool east)`.
pub fn vanity_addon_new(w: &mut World, east: bool) -> ItemId {
    let id =
        addon_container::new_addon_container(w, &VANITY_ADDON, if east { 0xA44 } else { 0xA3C });
    if east {
        addon_container::add_components(w, id, &[(CComp::Plain(2629), 0, -1, 0)]);
    } else {
        addon_container::add_components(w, id, &[(CComp::Plain(2621), -1, 0, 0)]);
    }
    id
}
static VANITY_ADDON_CLASS: ContainerAddonClass = ContainerAddonClass {
    deed: Some(vanity_deed_new),
    ..BASE_ADDON_CONTAINER_CLASS
};
leaf! {
    /// `Server.Items.VanityAddon`.
    VANITY_ADDON = "Server.Items.VanityAddon" : BASE_ADDON_CONTAINER, enc;
    family: Some(&VANITY_ADDON_CLASS),
    label_number: crate::class_props::label_number,
    default_gump_id: crate::class_props::default_gump_id,
    default_drop_sound: crate::class_props::default_drop_sound,
}

/// `VanityDeed()`.
pub fn vanity_deed_new(w: &mut World) -> ItemId {
    let id = addon_container::new_deed(w, &VANITY_DEED);
    ops::set_loot_type(w, id, LootType::Blessed);
    id
}
static VANITY_DEED_CLASS: ContainerDeedClass = ContainerDeedClass {
    addon: |w, deed| vanity_addon_new(w, addon_container::deed_east(w, deed)),
    select_direction: Some(1076744),
};
leaf! {
    /// `Server.Items.VanityDeed`.
    VANITY_DEED = "Server.Items.VanityDeed" : BASE_ADDON_CONTAINER_DEED, enc;
    construct: Some(vanity_deed_new),
    family: Some(&VANITY_DEED_CLASS),
    on_double_click: addon::direction_deed_on_double_click,
    label_number: crate::class_props::label_number,
}

/// `WallTorchAddon()`.
pub fn wall_torch_addon_new(w: &mut World) -> ItemId {
    let id = addon::new_addon(w, &WALL_TORCH_ADDON);
    addon::add_components(w, id, &[(Comp::Custom(wall_torch_component_new), 0, 0, 0)]);
    id
}
static WALL_TORCH_ADDON_CLASS: AddonClass = AddonClass {
    deed: Some(wall_torch_deed_new),
    ..BASE_ADDON_CLASS
};
leaf! {
    /// `Server.Items.WallTorchAddon`.
    WALL_TORCH_ADDON = "Server.Items.WallTorchAddon" : BASE_ADDON, enc;
    family: Some(&WALL_TORCH_ADDON_CLASS),
}

/// `WallTorchDeed()`.
pub fn wall_torch_deed_new(w: &mut World) -> ItemId {
    let id = addon::new_deed(w, &WALL_TORCH_DEED);
    ops::set_loot_type(w, id, LootType::Blessed);
    id
}
static WALL_TORCH_DEED_CLASS: DeedClass = DeedClass {
    addon: |w, _deed| wall_torch_addon_new(w),
    select_direction: None,
};
leaf! {
    /// `Server.Items.WallTorchDeed`.
    WALL_TORCH_DEED = "Server.Items.WallTorchDeed" : BASE_ADDON_DEED, enc;
    construct: Some(wall_torch_deed_new),
    family: Some(&WALL_TORCH_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `WoodenCoffinComponent(itemID)`.
pub fn wooden_coffin_component_new(w: &mut World, item_id: i32) -> ItemId {
    addon::new_component(w, &WOODEN_COFFIN_COMPONENT, item_id)
}
leaf! {
    /// `Server.Items.WoodenCoffinComponent`.
    WOODEN_COFFIN_COMPONENT = "Server.Items.WoodenCoffinComponent" : ADDON_COMPONENT, enc;
    label_number: crate::class_props::label_number,
}

/// `WoodenCoffinAddon(bool east)`.
pub fn wooden_coffin_addon_new(w: &mut World, east: bool) -> ItemId {
    let id = addon::new_addon(w, &WOODEN_COFFIN_ADDON);
    if east {
        addon::add_components(
            w,
            id,
            &[
                (Comp::Typed(wooden_coffin_component_new, 7233), 0, 0, 0),
                (Comp::Typed(wooden_coffin_component_new, 7234), 1, 0, 0),
                (Comp::Typed(wooden_coffin_component_new, 7235), 2, 0, 0),
            ],
        );
    } else {
        addon::add_components(
            w,
            id,
            &[
                (Comp::Typed(wooden_coffin_component_new, 7247), 0, 0, 0),
                (Comp::Typed(wooden_coffin_component_new, 7248), 0, 1, 0),
                (Comp::Typed(wooden_coffin_component_new, 7249), 0, 2, 0),
            ],
        );
    }
    id
}
static WOODEN_COFFIN_ADDON_CLASS: AddonClass = AddonClass {
    deed: Some(wooden_coffin_deed_new),
    ..BASE_ADDON_CLASS
};
leaf! {
    /// `Server.Items.WoodenCoffinAddon`.
    WOODEN_COFFIN_ADDON = "Server.Items.WoodenCoffinAddon" : BASE_ADDON, enc;
    family: Some(&WOODEN_COFFIN_ADDON_CLASS),
}

/// `WoodenCoffinDeed()`.
pub fn wooden_coffin_deed_new(w: &mut World) -> ItemId {
    let id = addon::new_deed(w, &WOODEN_COFFIN_DEED);
    ops::set_loot_type(w, id, LootType::Blessed);
    id
}
static WOODEN_COFFIN_DEED_CLASS: DeedClass = DeedClass {
    addon: |w, deed| wooden_coffin_addon_new(w, addon::deed_east(w, deed)),
    select_direction: Some(1076748),
};
leaf! {
    /// `Server.Items.WoodenCoffinDeed`.
    WOODEN_COFFIN_DEED = "Server.Items.WoodenCoffinDeed" : BASE_ADDON_DEED, enc;
    construct: Some(wooden_coffin_deed_new),
    family: Some(&WOODEN_COFFIN_DEED_CLASS),
    on_double_click: addon::direction_deed_on_double_click,
    label_number: crate::class_props::label_number,
}

/// Every class of this file, with its `[TypeAlias]`es.
pub static ALL: &[(&ItemType, &[&str])] = &[
    (&BROKEN_ARMOIRE_COMPONENT, &[]),
    (&BROKEN_ARMOIRE_ADDON, &[]),
    (&BROKEN_ARMOIRE_DEED, &[]),
    (&BROKEN_BED_ADDON, &[]),
    (&BROKEN_BED_DEED, &[]),
    (&BROKEN_BOOKCASE_COMPONENT, &[]),
    (&BROKEN_BOOKCASE_ADDON, &[]),
    (&BROKEN_BOOKCASE_DEED, &[]),
    (&BROKEN_CHEST_OF_DRAWERS_COMPONENT, &[]),
    (&BROKEN_CHEST_OF_DRAWERS_ADDON, &[]),
    (&BROKEN_CHEST_OF_DRAWERS_DEED, &[]),
    (&BROKEN_COVERED_CHAIR_COMPONENT, &[]),
    (&BROKEN_COVERED_CHAIR_ADDON, &[]),
    (&BROKEN_COVERED_CHAIR_DEED, &[]),
    (&BROKEN_FALLEN_CHAIR_COMPONENT, &[]),
    (&BROKEN_FALLEN_CHAIR_ADDON, &[]),
    (&BROKEN_FALLEN_CHAIR_DEED, &[]),
    (&BROKEN_VANITY_ADDON, &[]),
    (&BROKEN_VANITY_DEED, &[]),
    (&STANDING_BROKEN_CHAIR_COMPONENT, &[]),
    (&STANDING_BROKEN_CHAIR_ADDON, &[]),
    (&STANDING_BROKEN_CHAIR_DEED, &[]),
    (&BONE_COUCH_COMPONENT, &[]),
    (&BONE_TABLE_ADDON, &[]),
    (&BONE_TABLE_DEED, &[]),
    (&BONE_THRONE_COMPONENT, &[]),
    (&BONE_THRONE_ADDON, &[]),
    (&BONE_THRONE_DEED, &[]),
    (&MOUNTED_PIXIE_BLUE_COMPONENT, &[]),
    (&MOUNTED_PIXIE_BLUE_ADDON, &[]),
    (&MOUNTED_PIXIE_BLUE_DEED, &[]),
    (&MOUNTED_PIXIE_GREEN_COMPONENT, &[]),
    (&MOUNTED_PIXIE_GREEN_ADDON, &[]),
    (&MOUNTED_PIXIE_GREEN_DEED, &[]),
    (&MOUNTED_PIXIE_LIME_COMPONENT, &[]),
    (&MOUNTED_PIXIE_LIME_ADDON, &[]),
    (&MOUNTED_PIXIE_LIME_DEED, &[]),
    (&MOUNTED_PIXIE_ORANGE_COMPONENT, &[]),
    (&MOUNTED_PIXIE_ORANGE_ADDON, &[]),
    (&MOUNTED_PIXIE_ORANGE_DEED, &[]),
    (&MOUNTED_PIXIE_WHITE_COMPONENT, &[]),
    (&MOUNTED_PIXIE_WHITE_ADDON, &[]),
    (&MOUNTED_PIXIE_WHITE_DEED, &[]),
    (&APPLE_TRUNK_ADDON, &[]),
    (&APPLE_TRUNK_DEED, &[]),
    (&BLUE_DECORATIVE_RUG_ADDON, &[]),
    (&BLUE_DECORATIVE_RUG_DEED, &[]),
    (&BLUE_FANCY_RUG_ADDON, &[]),
    (&BLUE_FANCY_RUG_DEED, &[]),
    (&BLUE_PLAIN_RUG_ADDON, &[]),
    (&BLUE_PLAIN_RUG_DEED, &[]),
    (&BOILING_CAULDRON_ADDON, &[]),
    (&BOILING_CAULDRON_DEED, &[]),
    (&CHERRY_BLOSSOM_TREE_ADDON, &[]),
    (&CHERRY_BLOSSOM_TREE_DEED, &[]),
    (&CHERRY_BLOSSOM_TRUNK_ADDON, &[]),
    (&CHERRY_BLOSSOM_TRUNK_DEED, &[]),
    (&CINNAMON_FANCY_RUG_ADDON, &[]),
    (&CINNAMON_FANCY_RUG_DEED, &[]),
    (&GOLDEN_DECORATIVE_RUG_ADDON, &[]),
    (&GOLDEN_DECORATIVE_RUG_DEED, &[]),
    (&GUILLOTINE_COMPONENT, &[]),
    (&HANGING_AXES_ADDON, &[]),
    (&HANGING_AXES_DEED, &[]),
    (&HANGING_SWORDS_ADDON, &[]),
    (&HANGING_SWORDS_DEED, &[]),
    (&LARGE_FISHING_NET_COMPONENT, &[]),
    (&LARGE_FISHING_NET_ADDON, &[]),
    (&LARGE_FISHING_NET_DEED, &[]),
    (&PEACH_TRUNK_ADDON, &[]),
    (&PEACH_TRUNK_DEED, &[]),
    (&PINK_FANCY_RUG_ADDON, &[]),
    (&PINK_FANCY_RUG_DEED, &[]),
    (&RED_PLAIN_RUG_ADDON, &[]),
    (&RED_PLAIN_RUG_DEED, &[]),
    (&SCARECROW_COMPONENT, &[]),
    (&SCARECROW_ADDON, &[]),
    (&SCARECROW_DEED, &[]),
    (&SMALL_FISHING_NET_COMPONENT, &[]),
    (&SMALL_FISHING_NET_ADDON, &[]),
    (&SMALL_FISHING_NET_DEED, &[]),
    (&STONE_STATUE_ADDON, &[]),
    (&STONE_STATUE_DEED, &[]),
    (&SUIT_OF_GOLD_ARMOR_COMPONENT, &[]),
    (&SUIT_OF_GOLD_ARMOR_ADDON, &[]),
    (&SUIT_OF_GOLD_ARMOR_DEED, &[]),
    (&SUIT_OF_SILVER_ARMOR_COMPONENT, &[]),
    (&SUIT_OF_SILVER_ARMOR_ADDON, &[]),
    (&SUIT_OF_SILVER_ARMOR_DEED, &[]),
    (&TABLE_WITH_BLUE_CLOTH_ADDON, &[]),
    (&TABLE_WITH_BLUE_CLOTH_DEED, &[]),
    (&TABLE_WITH_ORANGE_CLOTH_ADDON, &[]),
    (&TABLE_WITH_ORANGE_CLOTH_DEED, &[]),
    (&TABLE_WITH_PURPLE_CLOTH_ADDON, &[]),
    (&TABLE_WITH_PURPLE_CLOTH_DEED, &[]),
    (&TABLE_WITH_RED_CLOTH_ADDON, &[]),
    (&TABLE_WITH_RED_CLOTH_DEED, &[]),
    (&UNMADE_BED_ADDON, &[]),
    (&UNMADE_BED_DEED, &[]),
    (&VANITY_ADDON, &[]),
    (&VANITY_DEED, &[]),
    (&WALL_TORCH_ADDON, &[]),
    (&WALL_TORCH_DEED, &[]),
    (&WOODEN_COFFIN_COMPONENT, &[]),
    (&WOODEN_COFFIN_ADDON, &[]),
    (&WOODEN_COFFIN_DEED, &[]),
];

pub fn register(types: &mut TypeRegistry) {
    for (t, aliases) in ALL {
        types.register_item(t, aliases);
    }
}
