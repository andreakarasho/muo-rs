//! The special addon families (Heritage, Evil Home Decor, Broken Furniture, Holiday): the addons, components and
//! deeds that are plain tables. Those with logic are hand-written next to this file.

use muo_content::items::addon::{self, ADDON_COMPONENT, BASE_ADDON, BASE_ADDON_DEED};
use muo_content::items::addon_container::{self, BASE_ADDON_CONTAINER, BASE_ADDON_CONTAINER_DEED};
use muo_server::World;
use muo_server::ids::ItemId;
use muo_server::item::{ItemType, LootType, ops};
use muo_server::type_registry::TypeRegistry;

use crate::simple::leaf;

#[allow(dead_code)]
fn item_hue(w: &mut World, id: ItemId, value: i32) {
    crate::simple::hue(w, id, value);
}

leaf! {
    /// `Server.Items.BrokenArmoireComponent`.
    BROKEN_ARMOIRE_COMPONENT = "Server.Items.BrokenArmoireComponent" : ADDON_COMPONENT, enc;
    label_number: crate::class_props::label_number,
}

/// `BrokenArmoireAddon()`.
pub fn broken_armoire_addon_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_addon(w, &BROKEN_ARMOIRE_ADDON)
}
leaf! {
    /// `Server.Items.BrokenArmoireAddon`.
    BROKEN_ARMOIRE_ADDON = "Server.Items.BrokenArmoireAddon" : BASE_ADDON, enc;
    construct: Some(broken_armoire_addon_new),
    family: Some(&crate::addons::TMPL_ADDON_CLASS),
}

/// `BrokenArmoireDeed()`.
pub fn broken_armoire_deed_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_deed(w, &BROKEN_ARMOIRE_DEED)
}
leaf! {
    /// `Server.Items.BrokenArmoireDeed`.
    BROKEN_ARMOIRE_DEED = "Server.Items.BrokenArmoireDeed" : BASE_ADDON_DEED, enc;
    construct: Some(broken_armoire_deed_new),
    family: Some(&crate::addons::TMPL_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `BrokenBedAddon(bool east)`.
pub fn broken_bed_addon_new(w: &mut World, east: bool) -> ItemId {
    crate::addons::tmpl_direction_addon(w, &BROKEN_BED_ADDON, east)
}
leaf! {
    /// `Server.Items.BrokenBedAddon`.
    BROKEN_BED_ADDON = "Server.Items.BrokenBedAddon" : BASE_ADDON, enc;
    family: Some(&crate::addons::TMPL_ADDON_CLASS),
}

/// `BrokenBedDeed()`.
pub fn broken_bed_deed_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_deed(w, &BROKEN_BED_DEED)
}
leaf! {
    /// `Server.Items.BrokenBedDeed`.
    BROKEN_BED_DEED = "Server.Items.BrokenBedDeed" : BASE_ADDON_DEED, enc;
    construct: Some(broken_bed_deed_new),
    family: Some(&crate::addons::TMPL_DIRECTION_DEED_CLASS),
    on_double_click: addon::direction_deed_on_double_click,
    label_number: crate::class_props::label_number,
}

leaf! {
    /// `Server.Items.BrokenBookcaseComponent`.
    BROKEN_BOOKCASE_COMPONENT = "Server.Items.BrokenBookcaseComponent" : ADDON_COMPONENT, enc;
    label_number: crate::class_props::label_number,
}

/// `BrokenBookcaseAddon()`.
pub fn broken_bookcase_addon_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_addon(w, &BROKEN_BOOKCASE_ADDON)
}
leaf! {
    /// `Server.Items.BrokenBookcaseAddon`.
    BROKEN_BOOKCASE_ADDON = "Server.Items.BrokenBookcaseAddon" : BASE_ADDON, enc;
    construct: Some(broken_bookcase_addon_new),
    family: Some(&crate::addons::TMPL_ADDON_CLASS),
}

/// `BrokenBookcaseDeed()`.
pub fn broken_bookcase_deed_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_deed(w, &BROKEN_BOOKCASE_DEED)
}
leaf! {
    /// `Server.Items.BrokenBookcaseDeed`.
    BROKEN_BOOKCASE_DEED = "Server.Items.BrokenBookcaseDeed" : BASE_ADDON_DEED, enc;
    construct: Some(broken_bookcase_deed_new),
    family: Some(&crate::addons::TMPL_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

leaf! {
    /// `Server.Items.BrokenChestOfDrawersComponent`.
    BROKEN_CHEST_OF_DRAWERS_COMPONENT = "Server.Items.BrokenChestOfDrawersComponent" : ADDON_COMPONENT, enc;
    label_number: crate::class_props::label_number,
}

/// `BrokenChestOfDrawersAddon()`.
pub fn broken_chest_of_drawers_addon_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_addon(w, &BROKEN_CHEST_OF_DRAWERS_ADDON)
}
leaf! {
    /// `Server.Items.BrokenChestOfDrawersAddon`.
    BROKEN_CHEST_OF_DRAWERS_ADDON = "Server.Items.BrokenChestOfDrawersAddon" : BASE_ADDON, enc;
    construct: Some(broken_chest_of_drawers_addon_new),
    family: Some(&crate::addons::TMPL_ADDON_CLASS),
}

/// `BrokenChestOfDrawersDeed()`.
pub fn broken_chest_of_drawers_deed_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_deed(w, &BROKEN_CHEST_OF_DRAWERS_DEED)
}
leaf! {
    /// `Server.Items.BrokenChestOfDrawersDeed`.
    BROKEN_CHEST_OF_DRAWERS_DEED = "Server.Items.BrokenChestOfDrawersDeed" : BASE_ADDON_DEED, enc;
    construct: Some(broken_chest_of_drawers_deed_new),
    family: Some(&crate::addons::TMPL_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

leaf! {
    /// `Server.Items.BrokenCoveredChairComponent`.
    BROKEN_COVERED_CHAIR_COMPONENT = "Server.Items.BrokenCoveredChairComponent" : ADDON_COMPONENT, enc;
    label_number: crate::class_props::label_number,
}

/// `BrokenCoveredChairAddon()`.
pub fn broken_covered_chair_addon_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_addon(w, &BROKEN_COVERED_CHAIR_ADDON)
}
leaf! {
    /// `Server.Items.BrokenCoveredChairAddon`.
    BROKEN_COVERED_CHAIR_ADDON = "Server.Items.BrokenCoveredChairAddon" : BASE_ADDON, enc;
    construct: Some(broken_covered_chair_addon_new),
    family: Some(&crate::addons::TMPL_ADDON_CLASS),
}

/// `BrokenCoveredChairDeed()`.
pub fn broken_covered_chair_deed_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_deed(w, &BROKEN_COVERED_CHAIR_DEED)
}
leaf! {
    /// `Server.Items.BrokenCoveredChairDeed`.
    BROKEN_COVERED_CHAIR_DEED = "Server.Items.BrokenCoveredChairDeed" : BASE_ADDON_DEED, enc;
    construct: Some(broken_covered_chair_deed_new),
    family: Some(&crate::addons::TMPL_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

leaf! {
    /// `Server.Items.BrokenFallenChairComponent`.
    BROKEN_FALLEN_CHAIR_COMPONENT = "Server.Items.BrokenFallenChairComponent" : ADDON_COMPONENT, enc;
    label_number: crate::class_props::label_number,
}

/// `BrokenFallenChairAddon()`.
pub fn broken_fallen_chair_addon_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_addon(w, &BROKEN_FALLEN_CHAIR_ADDON)
}
leaf! {
    /// `Server.Items.BrokenFallenChairAddon`.
    BROKEN_FALLEN_CHAIR_ADDON = "Server.Items.BrokenFallenChairAddon" : BASE_ADDON, enc;
    construct: Some(broken_fallen_chair_addon_new),
    family: Some(&crate::addons::TMPL_ADDON_CLASS),
}

/// `BrokenFallenChairDeed()`.
pub fn broken_fallen_chair_deed_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_deed(w, &BROKEN_FALLEN_CHAIR_DEED)
}
leaf! {
    /// `Server.Items.BrokenFallenChairDeed`.
    BROKEN_FALLEN_CHAIR_DEED = "Server.Items.BrokenFallenChairDeed" : BASE_ADDON_DEED, enc;
    construct: Some(broken_fallen_chair_deed_new),
    family: Some(&crate::addons::TMPL_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `BrokenVanityAddon(bool east)`.
pub fn broken_vanity_addon_new(w: &mut World, east: bool) -> ItemId {
    crate::addons::tmpl_direction_addon(w, &BROKEN_VANITY_ADDON, east)
}
leaf! {
    /// `Server.Items.BrokenVanityAddon`.
    BROKEN_VANITY_ADDON = "Server.Items.BrokenVanityAddon" : BASE_ADDON, enc;
    family: Some(&crate::addons::TMPL_ADDON_CLASS),
}

/// `BrokenVanityDeed()`.
pub fn broken_vanity_deed_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_deed(w, &BROKEN_VANITY_DEED)
}
leaf! {
    /// `Server.Items.BrokenVanityDeed`.
    BROKEN_VANITY_DEED = "Server.Items.BrokenVanityDeed" : BASE_ADDON_DEED, enc;
    construct: Some(broken_vanity_deed_new),
    family: Some(&crate::addons::TMPL_DIRECTION_DEED_CLASS),
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
    crate::addons::tmpl_addon(w, &STANDING_BROKEN_CHAIR_ADDON)
}
leaf! {
    /// `Server.Items.StandingBrokenChairAddon`.
    STANDING_BROKEN_CHAIR_ADDON = "Server.Items.StandingBrokenChairAddon" : BASE_ADDON, enc;
    construct: Some(standing_broken_chair_addon_new),
    family: Some(&crate::addons::TMPL_ADDON_CLASS),
}

/// `StandingBrokenChairDeed()`.
pub fn standing_broken_chair_deed_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_deed(w, &STANDING_BROKEN_CHAIR_DEED)
}
leaf! {
    /// `Server.Items.StandingBrokenChairDeed`.
    STANDING_BROKEN_CHAIR_DEED = "Server.Items.StandingBrokenChairDeed" : BASE_ADDON_DEED, enc;
    construct: Some(standing_broken_chair_deed_new),
    family: Some(&crate::addons::TMPL_DEED_CLASS),
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
    crate::addons::tmpl_addon(w, &BONE_TABLE_ADDON)
}
leaf! {
    /// `Server.Items.BoneTableAddon`.
    BONE_TABLE_ADDON = "Server.Items.BoneTableAddon" : BASE_ADDON, enc;
    construct: Some(bone_table_addon_new),
    family: Some(&crate::addons::TMPL_ADDON_CLASS),
}

/// `BoneTableDeed()`.
pub fn bone_table_deed_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_deed(w, &BONE_TABLE_DEED)
}
leaf! {
    /// `Server.Items.BoneTableDeed`.
    BONE_TABLE_DEED = "Server.Items.BoneTableDeed" : BASE_ADDON_DEED, enc;
    construct: Some(bone_table_deed_new),
    family: Some(&crate::addons::TMPL_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

leaf! {
    /// `Server.Items.BoneThroneComponent`.
    BONE_THRONE_COMPONENT = "Server.Items.BoneThroneComponent" : ADDON_COMPONENT, enc;
    on_move_over: |w, id, m| addon::move_over_sound(w, id, m, 0x54B, 0x54D),
    label_number: crate::class_props::label_number,
}

/// `BoneThroneAddon()`.
pub fn bone_throne_addon_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_addon(w, &BONE_THRONE_ADDON)
}
leaf! {
    /// `Server.Items.BoneThroneAddon`.
    BONE_THRONE_ADDON = "Server.Items.BoneThroneAddon" : BASE_ADDON, enc;
    construct: Some(bone_throne_addon_new),
    family: Some(&crate::addons::TMPL_ADDON_CLASS),
}

/// `BoneThroneDeed()`.
pub fn bone_throne_deed_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_deed(w, &BONE_THRONE_DEED)
}
leaf! {
    /// `Server.Items.BoneThroneDeed`.
    BONE_THRONE_DEED = "Server.Items.BoneThroneDeed" : BASE_ADDON_DEED, enc;
    construct: Some(bone_throne_deed_new),
    family: Some(&crate::addons::TMPL_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

leaf! {
    /// `Server.Items.MountedPixieBlueComponent`.
    MOUNTED_PIXIE_BLUE_COMPONENT = "Server.Items.MountedPixieBlueComponent" : ADDON_COMPONENT, enc;
    on_double_click: |w, id, from| addon::reach_sound(w, id, from, 0x55C, 0x55E),
    label_number: crate::class_props::label_number,
}

/// `MountedPixieBlueAddon()`.
pub fn mounted_pixie_blue_addon_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_addon(w, &MOUNTED_PIXIE_BLUE_ADDON)
}
leaf! {
    /// `Server.Items.MountedPixieBlueAddon`.
    MOUNTED_PIXIE_BLUE_ADDON = "Server.Items.MountedPixieBlueAddon" : BASE_ADDON, enc;
    family: Some(&crate::addons::TMPL_ADDON_CLASS),
}

/// `MountedPixieBlueDeed()`.
pub fn mounted_pixie_blue_deed_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_deed(w, &MOUNTED_PIXIE_BLUE_DEED)
}
leaf! {
    /// `Server.Items.MountedPixieBlueDeed`.
    MOUNTED_PIXIE_BLUE_DEED = "Server.Items.MountedPixieBlueDeed" : BASE_ADDON_DEED, enc;
    construct: Some(mounted_pixie_blue_deed_new),
    family: Some(&crate::addons::TMPL_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

leaf! {
    /// `Server.Items.MountedPixieGreenComponent`.
    MOUNTED_PIXIE_GREEN_COMPONENT = "Server.Items.MountedPixieGreenComponent" : ADDON_COMPONENT, enc;
    on_double_click: |w, id, from| addon::reach_sound(w, id, from, 0x554, 0x557),
    label_number: crate::class_props::label_number,
}

/// `MountedPixieGreenAddon()`.
pub fn mounted_pixie_green_addon_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_addon(w, &MOUNTED_PIXIE_GREEN_ADDON)
}
leaf! {
    /// `Server.Items.MountedPixieGreenAddon`.
    MOUNTED_PIXIE_GREEN_ADDON = "Server.Items.MountedPixieGreenAddon" : BASE_ADDON, enc;
    family: Some(&crate::addons::TMPL_ADDON_CLASS),
}

/// `MountedPixieGreenDeed()`.
pub fn mounted_pixie_green_deed_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_deed(w, &MOUNTED_PIXIE_GREEN_DEED)
}
leaf! {
    /// `Server.Items.MountedPixieGreenDeed`.
    MOUNTED_PIXIE_GREEN_DEED = "Server.Items.MountedPixieGreenDeed" : BASE_ADDON_DEED, enc;
    construct: Some(mounted_pixie_green_deed_new),
    family: Some(&crate::addons::TMPL_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

leaf! {
    /// `Server.Items.MountedPixieLimeComponent`.
    MOUNTED_PIXIE_LIME_COMPONENT = "Server.Items.MountedPixieLimeComponent" : ADDON_COMPONENT, enc;
    on_double_click: |w, id, from| addon::reach_sound(w, id, from, 0x55F, 0x561),
    label_number: crate::class_props::label_number,
}

/// `MountedPixieLimeAddon()`.
pub fn mounted_pixie_lime_addon_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_addon(w, &MOUNTED_PIXIE_LIME_ADDON)
}
leaf! {
    /// `Server.Items.MountedPixieLimeAddon`.
    MOUNTED_PIXIE_LIME_ADDON = "Server.Items.MountedPixieLimeAddon" : BASE_ADDON, enc;
    family: Some(&crate::addons::TMPL_ADDON_CLASS),
}

/// `MountedPixieLimeDeed()`.
pub fn mounted_pixie_lime_deed_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_deed(w, &MOUNTED_PIXIE_LIME_DEED)
}
leaf! {
    /// `Server.Items.MountedPixieLimeDeed`.
    MOUNTED_PIXIE_LIME_DEED = "Server.Items.MountedPixieLimeDeed" : BASE_ADDON_DEED, enc;
    construct: Some(mounted_pixie_lime_deed_new),
    family: Some(&crate::addons::TMPL_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

leaf! {
    /// `Server.Items.MountedPixieOrangeComponent`.
    MOUNTED_PIXIE_ORANGE_COMPONENT = "Server.Items.MountedPixieOrangeComponent" : ADDON_COMPONENT, enc;
    on_double_click: |w, id, from| addon::reach_sound(w, id, from, 0x558, 0x55B),
    label_number: crate::class_props::label_number,
}

/// `MountedPixieOrangeAddon()`.
pub fn mounted_pixie_orange_addon_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_addon(w, &MOUNTED_PIXIE_ORANGE_ADDON)
}
leaf! {
    /// `Server.Items.MountedPixieOrangeAddon`.
    MOUNTED_PIXIE_ORANGE_ADDON = "Server.Items.MountedPixieOrangeAddon" : BASE_ADDON, enc;
    family: Some(&crate::addons::TMPL_ADDON_CLASS),
}

/// `MountedPixieOrangeDeed()`.
pub fn mounted_pixie_orange_deed_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_deed(w, &MOUNTED_PIXIE_ORANGE_DEED)
}
leaf! {
    /// `Server.Items.MountedPixieOrangeDeed`.
    MOUNTED_PIXIE_ORANGE_DEED = "Server.Items.MountedPixieOrangeDeed" : BASE_ADDON_DEED, enc;
    construct: Some(mounted_pixie_orange_deed_new),
    family: Some(&crate::addons::TMPL_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

leaf! {
    /// `Server.Items.MountedPixieWhiteComponent`.
    MOUNTED_PIXIE_WHITE_COMPONENT = "Server.Items.MountedPixieWhiteComponent" : ADDON_COMPONENT, enc;
    on_double_click: |w, id, from| addon::reach_sound(w, id, from, 0x562, 0x564),
    label_number: crate::class_props::label_number,
}

/// `MountedPixieWhiteAddon()`.
pub fn mounted_pixie_white_addon_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_addon(w, &MOUNTED_PIXIE_WHITE_ADDON)
}
leaf! {
    /// `Server.Items.MountedPixieWhiteAddon`.
    MOUNTED_PIXIE_WHITE_ADDON = "Server.Items.MountedPixieWhiteAddon" : BASE_ADDON, enc;
    family: Some(&crate::addons::TMPL_ADDON_CLASS),
}

/// `MountedPixieWhiteDeed()`.
pub fn mounted_pixie_white_deed_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_deed(w, &MOUNTED_PIXIE_WHITE_DEED)
}
leaf! {
    /// `Server.Items.MountedPixieWhiteDeed`.
    MOUNTED_PIXIE_WHITE_DEED = "Server.Items.MountedPixieWhiteDeed" : BASE_ADDON_DEED, enc;
    construct: Some(mounted_pixie_white_deed_new),
    family: Some(&crate::addons::TMPL_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `AppleTrunkAddon()`.
pub fn apple_trunk_addon_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_addon(w, &APPLE_TRUNK_ADDON)
}
leaf! {
    /// `Server.Items.AppleTrunkAddon`.
    APPLE_TRUNK_ADDON = "Server.Items.AppleTrunkAddon" : BASE_ADDON, enc;
    construct: Some(apple_trunk_addon_new),
    family: Some(&crate::addons::TMPL_ADDON_CLASS),
}

/// `AppleTrunkDeed()`.
pub fn apple_trunk_deed_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_deed(w, &APPLE_TRUNK_DEED)
}
leaf! {
    /// `Server.Items.AppleTrunkDeed`.
    APPLE_TRUNK_DEED = "Server.Items.AppleTrunkDeed" : BASE_ADDON_DEED, enc;
    construct: Some(apple_trunk_deed_new),
    family: Some(&crate::addons::TMPL_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `BlueDecorativeRugAddon()`.
pub fn blue_decorative_rug_addon_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_addon(w, &BLUE_DECORATIVE_RUG_ADDON)
}
leaf! {
    /// `Server.Items.BlueDecorativeRugAddon`.
    BLUE_DECORATIVE_RUG_ADDON = "Server.Items.BlueDecorativeRugAddon" : BASE_ADDON, enc;
    construct: Some(blue_decorative_rug_addon_new),
    family: Some(&crate::addons::TMPL_ADDON_CLASS),
}

/// `BlueDecorativeRugDeed()`.
pub fn blue_decorative_rug_deed_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_deed(w, &BLUE_DECORATIVE_RUG_DEED)
}
leaf! {
    /// `Server.Items.BlueDecorativeRugDeed`.
    BLUE_DECORATIVE_RUG_DEED = "Server.Items.BlueDecorativeRugDeed" : BASE_ADDON_DEED, enc;
    construct: Some(blue_decorative_rug_deed_new),
    family: Some(&crate::addons::TMPL_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `BlueFancyRugAddon()`.
pub fn blue_fancy_rug_addon_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_addon(w, &BLUE_FANCY_RUG_ADDON)
}
leaf! {
    /// `Server.Items.BlueFancyRugAddon`.
    BLUE_FANCY_RUG_ADDON = "Server.Items.BlueFancyRugAddon" : BASE_ADDON, enc;
    construct: Some(blue_fancy_rug_addon_new),
    family: Some(&crate::addons::TMPL_ADDON_CLASS),
}

/// `BlueFancyRugDeed()`.
pub fn blue_fancy_rug_deed_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_deed(w, &BLUE_FANCY_RUG_DEED)
}
leaf! {
    /// `Server.Items.BlueFancyRugDeed`.
    BLUE_FANCY_RUG_DEED = "Server.Items.BlueFancyRugDeed" : BASE_ADDON_DEED, enc;
    construct: Some(blue_fancy_rug_deed_new),
    family: Some(&crate::addons::TMPL_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `BluePlainRugAddon()`.
pub fn blue_plain_rug_addon_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_addon(w, &BLUE_PLAIN_RUG_ADDON)
}
leaf! {
    /// `Server.Items.BluePlainRugAddon`.
    BLUE_PLAIN_RUG_ADDON = "Server.Items.BluePlainRugAddon" : BASE_ADDON, enc;
    construct: Some(blue_plain_rug_addon_new),
    family: Some(&crate::addons::TMPL_ADDON_CLASS),
}

/// `BluePlainRugDeed()`.
pub fn blue_plain_rug_deed_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_deed(w, &BLUE_PLAIN_RUG_DEED)
}
leaf! {
    /// `Server.Items.BluePlainRugDeed`.
    BLUE_PLAIN_RUG_DEED = "Server.Items.BluePlainRugDeed" : BASE_ADDON_DEED, enc;
    construct: Some(blue_plain_rug_deed_new),
    family: Some(&crate::addons::TMPL_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `BoilingCauldronAddon()`.
pub fn boiling_cauldron_addon_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_container(w, &BOILING_CAULDRON_ADDON)
}
leaf! {
    /// `Server.Items.BoilingCauldronAddon`.
    BOILING_CAULDRON_ADDON = "Server.Items.BoilingCauldronAddon" : BASE_ADDON_CONTAINER, enc;
    construct: Some(boiling_cauldron_addon_new),
    family: Some(&crate::addons::TMPL_CONTAINER_CLASS),
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
leaf! {
    /// `Server.Items.BoilingCauldronDeed`.
    BOILING_CAULDRON_DEED = "Server.Items.BoilingCauldronDeed" : BASE_ADDON_CONTAINER_DEED, enc;
    construct: Some(boiling_cauldron_deed_new),
    family: Some(&crate::addons::TMPL_CONTAINER_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `CherryBlossomTreeAddon()`.
pub fn cherry_blossom_tree_addon_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_addon(w, &CHERRY_BLOSSOM_TREE_ADDON)
}
leaf! {
    /// `Server.Items.CherryBlossomTreeAddon`.
    CHERRY_BLOSSOM_TREE_ADDON = "Server.Items.CherryBlossomTreeAddon" : BASE_ADDON, enc;
    construct: Some(cherry_blossom_tree_addon_new),
    family: Some(&crate::addons::TMPL_ADDON_CLASS),
}

/// `CherryBlossomTreeDeed()`.
pub fn cherry_blossom_tree_deed_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_deed(w, &CHERRY_BLOSSOM_TREE_DEED)
}
leaf! {
    /// `Server.Items.CherryBlossomTreeDeed`.
    CHERRY_BLOSSOM_TREE_DEED = "Server.Items.CherryBlossomTreeDeed" : BASE_ADDON_DEED, enc;
    construct: Some(cherry_blossom_tree_deed_new),
    family: Some(&crate::addons::TMPL_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `CherryBlossomTrunkAddon()`.
pub fn cherry_blossom_trunk_addon_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_addon(w, &CHERRY_BLOSSOM_TRUNK_ADDON)
}
leaf! {
    /// `Server.Items.CherryBlossomTrunkAddon`.
    CHERRY_BLOSSOM_TRUNK_ADDON = "Server.Items.CherryBlossomTrunkAddon" : BASE_ADDON, enc;
    construct: Some(cherry_blossom_trunk_addon_new),
    family: Some(&crate::addons::TMPL_ADDON_CLASS),
}

/// `CherryBlossomTrunkDeed()`.
pub fn cherry_blossom_trunk_deed_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_deed(w, &CHERRY_BLOSSOM_TRUNK_DEED)
}
leaf! {
    /// `Server.Items.CherryBlossomTrunkDeed`.
    CHERRY_BLOSSOM_TRUNK_DEED = "Server.Items.CherryBlossomTrunkDeed" : BASE_ADDON_DEED, enc;
    construct: Some(cherry_blossom_trunk_deed_new),
    family: Some(&crate::addons::TMPL_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `CinnamonFancyRugAddon()`.
pub fn cinnamon_fancy_rug_addon_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_addon(w, &CINNAMON_FANCY_RUG_ADDON)
}
leaf! {
    /// `Server.Items.CinnamonFancyRugAddon`.
    CINNAMON_FANCY_RUG_ADDON = "Server.Items.CinnamonFancyRugAddon" : BASE_ADDON, enc;
    construct: Some(cinnamon_fancy_rug_addon_new),
    family: Some(&crate::addons::TMPL_ADDON_CLASS),
}

/// `CinnamonFancyRugDeed()`.
pub fn cinnamon_fancy_rug_deed_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_deed(w, &CINNAMON_FANCY_RUG_DEED)
}
leaf! {
    /// `Server.Items.CinnamonFancyRugDeed`.
    CINNAMON_FANCY_RUG_DEED = "Server.Items.CinnamonFancyRugDeed" : BASE_ADDON_DEED, enc;
    construct: Some(cinnamon_fancy_rug_deed_new),
    family: Some(&crate::addons::TMPL_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `GoldenDecorativeRugAddon()`.
pub fn golden_decorative_rug_addon_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_addon(w, &GOLDEN_DECORATIVE_RUG_ADDON)
}
leaf! {
    /// `Server.Items.GoldenDecorativeRugAddon`.
    GOLDEN_DECORATIVE_RUG_ADDON = "Server.Items.GoldenDecorativeRugAddon" : BASE_ADDON, enc;
    construct: Some(golden_decorative_rug_addon_new),
    family: Some(&crate::addons::TMPL_ADDON_CLASS),
}

/// `GoldenDecorativeRugDeed()`.
pub fn golden_decorative_rug_deed_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_deed(w, &GOLDEN_DECORATIVE_RUG_DEED)
}
leaf! {
    /// `Server.Items.GoldenDecorativeRugDeed`.
    GOLDEN_DECORATIVE_RUG_DEED = "Server.Items.GoldenDecorativeRugDeed" : BASE_ADDON_DEED, enc;
    construct: Some(golden_decorative_rug_deed_new),
    family: Some(&crate::addons::TMPL_DEED_CLASS),
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
    crate::addons::tmpl_direction_addon(w, &HANGING_AXES_ADDON, east)
}
leaf! {
    /// `Server.Items.HangingAxesAddon`.
    HANGING_AXES_ADDON = "Server.Items.HangingAxesAddon" : BASE_ADDON, enc;
    family: Some(&crate::addons::TMPL_ADDON_CLASS),
}

/// `HangingAxesDeed()`.
pub fn hanging_axes_deed_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_deed(w, &HANGING_AXES_DEED)
}
leaf! {
    /// `Server.Items.HangingAxesDeed`.
    HANGING_AXES_DEED = "Server.Items.HangingAxesDeed" : BASE_ADDON_DEED, enc;
    construct: Some(hanging_axes_deed_new),
    family: Some(&crate::addons::TMPL_DIRECTION_DEED_CLASS),
    on_double_click: addon::direction_deed_on_double_click,
    label_number: crate::class_props::label_number,
}

/// `HangingSwordsAddon(bool east)`.
pub fn hanging_swords_addon_new(w: &mut World, east: bool) -> ItemId {
    crate::addons::tmpl_direction_addon(w, &HANGING_SWORDS_ADDON, east)
}
leaf! {
    /// `Server.Items.HangingSwordsAddon`.
    HANGING_SWORDS_ADDON = "Server.Items.HangingSwordsAddon" : BASE_ADDON, enc;
    family: Some(&crate::addons::TMPL_ADDON_CLASS),
}

/// `HangingSwordsDeed()`.
pub fn hanging_swords_deed_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_deed(w, &HANGING_SWORDS_DEED)
}
leaf! {
    /// `Server.Items.HangingSwordsDeed`.
    HANGING_SWORDS_DEED = "Server.Items.HangingSwordsDeed" : BASE_ADDON_DEED, enc;
    construct: Some(hanging_swords_deed_new),
    family: Some(&crate::addons::TMPL_DIRECTION_DEED_CLASS),
    on_double_click: addon::direction_deed_on_double_click,
    label_number: crate::class_props::label_number,
}

leaf! {
    /// `Server.Items.LargeFishingNetComponent`.
    LARGE_FISHING_NET_COMPONENT = "Server.Items.LargeFishingNetComponent" : ADDON_COMPONENT, enc;
    label_number: crate::class_props::label_number,
}

/// `LargeFishingNetAddon()`.
pub fn large_fishing_net_addon_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_addon(w, &LARGE_FISHING_NET_ADDON)
}
leaf! {
    /// `Server.Items.LargeFishingNetAddon`.
    LARGE_FISHING_NET_ADDON = "Server.Items.LargeFishingNetAddon" : BASE_ADDON, enc;
    construct: Some(large_fishing_net_addon_new),
    family: Some(&crate::addons::TMPL_ADDON_CLASS),
}

/// `LargeFishingNetDeed()`.
pub fn large_fishing_net_deed_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_deed(w, &LARGE_FISHING_NET_DEED)
}
leaf! {
    /// `Server.Items.LargeFishingNetDeed`.
    LARGE_FISHING_NET_DEED = "Server.Items.LargeFishingNetDeed" : BASE_ADDON_DEED, enc;
    construct: Some(large_fishing_net_deed_new),
    family: Some(&crate::addons::TMPL_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `PeachTrunkAddon()`.
pub fn peach_trunk_addon_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_addon(w, &PEACH_TRUNK_ADDON)
}
leaf! {
    /// `Server.Items.PeachTrunkAddon`.
    PEACH_TRUNK_ADDON = "Server.Items.PeachTrunkAddon" : BASE_ADDON, enc;
    construct: Some(peach_trunk_addon_new),
    family: Some(&crate::addons::TMPL_ADDON_CLASS),
}

/// `PeachTrunkDeed()`.
pub fn peach_trunk_deed_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_deed(w, &PEACH_TRUNK_DEED)
}
leaf! {
    /// `Server.Items.PeachTrunkDeed`.
    PEACH_TRUNK_DEED = "Server.Items.PeachTrunkDeed" : BASE_ADDON_DEED, enc;
    construct: Some(peach_trunk_deed_new),
    family: Some(&crate::addons::TMPL_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `PinkFancyRugAddon()`.
pub fn pink_fancy_rug_addon_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_addon(w, &PINK_FANCY_RUG_ADDON)
}
leaf! {
    /// `Server.Items.PinkFancyRugAddon`.
    PINK_FANCY_RUG_ADDON = "Server.Items.PinkFancyRugAddon" : BASE_ADDON, enc;
    construct: Some(pink_fancy_rug_addon_new),
    family: Some(&crate::addons::TMPL_ADDON_CLASS),
}

/// `PinkFancyRugDeed()`.
pub fn pink_fancy_rug_deed_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_deed(w, &PINK_FANCY_RUG_DEED)
}
leaf! {
    /// `Server.Items.PinkFancyRugDeed`.
    PINK_FANCY_RUG_DEED = "Server.Items.PinkFancyRugDeed" : BASE_ADDON_DEED, enc;
    construct: Some(pink_fancy_rug_deed_new),
    family: Some(&crate::addons::TMPL_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `RedPlainRugAddon()`.
pub fn red_plain_rug_addon_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_addon(w, &RED_PLAIN_RUG_ADDON)
}
leaf! {
    /// `Server.Items.RedPlainRugAddon`.
    RED_PLAIN_RUG_ADDON = "Server.Items.RedPlainRugAddon" : BASE_ADDON, enc;
    construct: Some(red_plain_rug_addon_new),
    family: Some(&crate::addons::TMPL_ADDON_CLASS),
}

/// `RedPlainRugDeed()`.
pub fn red_plain_rug_deed_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_deed(w, &RED_PLAIN_RUG_DEED)
}
leaf! {
    /// `Server.Items.RedPlainRugDeed`.
    RED_PLAIN_RUG_DEED = "Server.Items.RedPlainRugDeed" : BASE_ADDON_DEED, enc;
    construct: Some(red_plain_rug_deed_new),
    family: Some(&crate::addons::TMPL_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

leaf! {
    /// `Server.Items.ScarecrowComponent`.
    SCARECROW_COMPONENT = "Server.Items.ScarecrowComponent" : ADDON_COMPONENT, enc;
    label_number: crate::class_props::label_number,
}

/// `ScarecrowAddon()`.
pub fn scarecrow_addon_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_addon(w, &SCARECROW_ADDON)
}
leaf! {
    /// `Server.Items.ScarecrowAddon`.
    SCARECROW_ADDON = "Server.Items.ScarecrowAddon" : BASE_ADDON, enc;
    construct: Some(scarecrow_addon_new),
    family: Some(&crate::addons::TMPL_ADDON_CLASS),
}

/// `ScarecrowDeed()`.
pub fn scarecrow_deed_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_deed(w, &SCARECROW_DEED)
}
leaf! {
    /// `Server.Items.ScarecrowDeed`.
    SCARECROW_DEED = "Server.Items.ScarecrowDeed" : BASE_ADDON_DEED, enc;
    construct: Some(scarecrow_deed_new),
    family: Some(&crate::addons::TMPL_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

leaf! {
    /// `Server.Items.SmallFishingNetComponent`.
    SMALL_FISHING_NET_COMPONENT = "Server.Items.SmallFishingNetComponent" : ADDON_COMPONENT, enc;
    label_number: crate::class_props::label_number,
}

/// `SmallFishingNetAddon()`.
pub fn small_fishing_net_addon_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_addon(w, &SMALL_FISHING_NET_ADDON)
}
leaf! {
    /// `Server.Items.SmallFishingNetAddon`.
    SMALL_FISHING_NET_ADDON = "Server.Items.SmallFishingNetAddon" : BASE_ADDON, enc;
    construct: Some(small_fishing_net_addon_new),
    family: Some(&crate::addons::TMPL_ADDON_CLASS),
}

/// `SmallFishingNetDeed()`.
pub fn small_fishing_net_deed_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_deed(w, &SMALL_FISHING_NET_DEED)
}
leaf! {
    /// `Server.Items.SmallFishingNetDeed`.
    SMALL_FISHING_NET_DEED = "Server.Items.SmallFishingNetDeed" : BASE_ADDON_DEED, enc;
    construct: Some(small_fishing_net_deed_new),
    family: Some(&crate::addons::TMPL_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `StoneStatueAddon(bool east)`.
pub fn stone_statue_addon_new(w: &mut World, east: bool) -> ItemId {
    crate::addons::tmpl_direction_addon(w, &STONE_STATUE_ADDON, east)
}
leaf! {
    /// `Server.Items.StoneStatueAddon`.
    STONE_STATUE_ADDON = "Server.Items.StoneStatueAddon" : BASE_ADDON, enc;
    family: Some(&crate::addons::TMPL_ADDON_CLASS),
}

/// `StoneStatueDeed()`.
pub fn stone_statue_deed_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_deed(w, &STONE_STATUE_DEED)
}
leaf! {
    /// `Server.Items.StoneStatueDeed`.
    STONE_STATUE_DEED = "Server.Items.StoneStatueDeed" : BASE_ADDON_DEED, enc;
    construct: Some(stone_statue_deed_new),
    family: Some(&crate::addons::TMPL_DIRECTION_DEED_CLASS),
    on_double_click: addon::direction_deed_on_double_click,
    label_number: crate::class_props::label_number,
}

leaf! {
    /// `Server.Items.SuitOfGoldArmorComponent`.
    SUIT_OF_GOLD_ARMOR_COMPONENT = "Server.Items.SuitOfGoldArmorComponent" : ADDON_COMPONENT, enc;
    label_number: crate::class_props::label_number,
}

/// `SuitOfGoldArmorAddon()`.
pub fn suit_of_gold_armor_addon_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_addon(w, &SUIT_OF_GOLD_ARMOR_ADDON)
}
leaf! {
    /// `Server.Items.SuitOfGoldArmorAddon`.
    SUIT_OF_GOLD_ARMOR_ADDON = "Server.Items.SuitOfGoldArmorAddon" : BASE_ADDON, enc;
    construct: Some(suit_of_gold_armor_addon_new),
    family: Some(&crate::addons::TMPL_ADDON_CLASS),
}

/// `SuitOfGoldArmorDeed()`.
pub fn suit_of_gold_armor_deed_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_deed(w, &SUIT_OF_GOLD_ARMOR_DEED)
}
leaf! {
    /// `Server.Items.SuitOfGoldArmorDeed`.
    SUIT_OF_GOLD_ARMOR_DEED = "Server.Items.SuitOfGoldArmorDeed" : BASE_ADDON_DEED, enc;
    construct: Some(suit_of_gold_armor_deed_new),
    family: Some(&crate::addons::TMPL_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

leaf! {
    /// `Server.Items.SuitOfSilverArmorComponent`.
    SUIT_OF_SILVER_ARMOR_COMPONENT = "Server.Items.SuitOfSilverArmorComponent" : ADDON_COMPONENT, enc;
    label_number: crate::class_props::label_number,
}

/// `SuitOfSilverArmorAddon()`.
pub fn suit_of_silver_armor_addon_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_addon(w, &SUIT_OF_SILVER_ARMOR_ADDON)
}
leaf! {
    /// `Server.Items.SuitOfSilverArmorAddon`.
    SUIT_OF_SILVER_ARMOR_ADDON = "Server.Items.SuitOfSilverArmorAddon" : BASE_ADDON, enc;
    construct: Some(suit_of_silver_armor_addon_new),
    family: Some(&crate::addons::TMPL_ADDON_CLASS),
}

/// `SuitOfSilverArmorDeed()`.
pub fn suit_of_silver_armor_deed_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_deed(w, &SUIT_OF_SILVER_ARMOR_DEED)
}
leaf! {
    /// `Server.Items.SuitOfSilverArmorDeed`.
    SUIT_OF_SILVER_ARMOR_DEED = "Server.Items.SuitOfSilverArmorDeed" : BASE_ADDON_DEED, enc;
    construct: Some(suit_of_silver_armor_deed_new),
    family: Some(&crate::addons::TMPL_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `TableWithBlueClothAddon()`.
pub fn table_with_blue_cloth_addon_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_addon(w, &TABLE_WITH_BLUE_CLOTH_ADDON)
}
leaf! {
    /// `Server.Items.TableWithBlueClothAddon`.
    TABLE_WITH_BLUE_CLOTH_ADDON = "Server.Items.TableWithBlueClothAddon" : BASE_ADDON, enc;
    construct: Some(table_with_blue_cloth_addon_new),
    family: Some(&crate::addons::TMPL_ADDON_CLASS),
}

/// `TableWithBlueClothDeed()`.
pub fn table_with_blue_cloth_deed_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_deed(w, &TABLE_WITH_BLUE_CLOTH_DEED)
}
leaf! {
    /// `Server.Items.TableWithBlueClothDeed`.
    TABLE_WITH_BLUE_CLOTH_DEED = "Server.Items.TableWithBlueClothDeed" : BASE_ADDON_DEED, enc;
    construct: Some(table_with_blue_cloth_deed_new),
    family: Some(&crate::addons::TMPL_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `TableWithOrangeClothAddon()`.
pub fn table_with_orange_cloth_addon_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_addon(w, &TABLE_WITH_ORANGE_CLOTH_ADDON)
}
leaf! {
    /// `Server.Items.TableWithOrangeClothAddon`.
    TABLE_WITH_ORANGE_CLOTH_ADDON = "Server.Items.TableWithOrangeClothAddon" : BASE_ADDON, enc;
    construct: Some(table_with_orange_cloth_addon_new),
    family: Some(&crate::addons::TMPL_ADDON_CLASS),
}

/// `TableWithOrangeClothDeed()`.
pub fn table_with_orange_cloth_deed_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_deed(w, &TABLE_WITH_ORANGE_CLOTH_DEED)
}
leaf! {
    /// `Server.Items.TableWithOrangeClothDeed`.
    TABLE_WITH_ORANGE_CLOTH_DEED = "Server.Items.TableWithOrangeClothDeed" : BASE_ADDON_DEED, enc;
    construct: Some(table_with_orange_cloth_deed_new),
    family: Some(&crate::addons::TMPL_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `TableWithPurpleClothAddon()`.
pub fn table_with_purple_cloth_addon_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_addon(w, &TABLE_WITH_PURPLE_CLOTH_ADDON)
}
leaf! {
    /// `Server.Items.TableWithPurpleClothAddon`.
    TABLE_WITH_PURPLE_CLOTH_ADDON = "Server.Items.TableWithPurpleClothAddon" : BASE_ADDON, enc;
    construct: Some(table_with_purple_cloth_addon_new),
    family: Some(&crate::addons::TMPL_ADDON_CLASS),
}

/// `TableWithPurpleClothDeed()`.
pub fn table_with_purple_cloth_deed_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_deed(w, &TABLE_WITH_PURPLE_CLOTH_DEED)
}
leaf! {
    /// `Server.Items.TableWithPurpleClothDeed`.
    TABLE_WITH_PURPLE_CLOTH_DEED = "Server.Items.TableWithPurpleClothDeed" : BASE_ADDON_DEED, enc;
    construct: Some(table_with_purple_cloth_deed_new),
    family: Some(&crate::addons::TMPL_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `TableWithRedClothAddon()`.
pub fn table_with_red_cloth_addon_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_addon(w, &TABLE_WITH_RED_CLOTH_ADDON)
}
leaf! {
    /// `Server.Items.TableWithRedClothAddon`.
    TABLE_WITH_RED_CLOTH_ADDON = "Server.Items.TableWithRedClothAddon" : BASE_ADDON, enc;
    construct: Some(table_with_red_cloth_addon_new),
    family: Some(&crate::addons::TMPL_ADDON_CLASS),
}

/// `TableWithRedClothDeed()`.
pub fn table_with_red_cloth_deed_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_deed(w, &TABLE_WITH_RED_CLOTH_DEED)
}
leaf! {
    /// `Server.Items.TableWithRedClothDeed`.
    TABLE_WITH_RED_CLOTH_DEED = "Server.Items.TableWithRedClothDeed" : BASE_ADDON_DEED, enc;
    construct: Some(table_with_red_cloth_deed_new),
    family: Some(&crate::addons::TMPL_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `UnmadeBedAddon(bool east)`.
pub fn unmade_bed_addon_new(w: &mut World, east: bool) -> ItemId {
    crate::addons::tmpl_direction_addon(w, &UNMADE_BED_ADDON, east)
}
leaf! {
    /// `Server.Items.UnmadeBedAddon`.
    UNMADE_BED_ADDON = "Server.Items.UnmadeBedAddon" : BASE_ADDON, enc;
    family: Some(&crate::addons::TMPL_ADDON_CLASS),
}

/// `UnmadeBedDeed()`.
pub fn unmade_bed_deed_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_deed(w, &UNMADE_BED_DEED)
}
leaf! {
    /// `Server.Items.UnmadeBedDeed`.
    UNMADE_BED_DEED = "Server.Items.UnmadeBedDeed" : BASE_ADDON_DEED, enc;
    construct: Some(unmade_bed_deed_new),
    family: Some(&crate::addons::TMPL_DIRECTION_DEED_CLASS),
    on_double_click: addon::direction_deed_on_double_click,
    label_number: crate::class_props::label_number,
}

/// `VanityAddon(bool east)`.
pub fn vanity_addon_new(w: &mut World, east: bool) -> ItemId {
    crate::addons::tmpl_direction_addon(w, &VANITY_ADDON, east)
}
leaf! {
    /// `Server.Items.VanityAddon`.
    VANITY_ADDON = "Server.Items.VanityAddon" : BASE_ADDON_CONTAINER, enc;
    family: Some(&crate::addons::TMPL_CONTAINER_CLASS),
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
leaf! {
    /// `Server.Items.VanityDeed`.
    VANITY_DEED = "Server.Items.VanityDeed" : BASE_ADDON_CONTAINER_DEED, enc;
    construct: Some(vanity_deed_new),
    family: Some(&crate::addons::TMPL_DIRECTION_CONTAINER_DEED_CLASS),
    on_double_click: addon::direction_deed_on_double_click,
    label_number: crate::class_props::label_number,
}

/// `WallTorchAddon()`.
pub fn wall_torch_addon_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_addon(w, &WALL_TORCH_ADDON)
}
leaf! {
    /// `Server.Items.WallTorchAddon`.
    WALL_TORCH_ADDON = "Server.Items.WallTorchAddon" : BASE_ADDON, enc;
    family: Some(&crate::addons::TMPL_ADDON_CLASS),
}

/// `WallTorchDeed()`.
pub fn wall_torch_deed_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_deed(w, &WALL_TORCH_DEED)
}
leaf! {
    /// `Server.Items.WallTorchDeed`.
    WALL_TORCH_DEED = "Server.Items.WallTorchDeed" : BASE_ADDON_DEED, enc;
    construct: Some(wall_torch_deed_new),
    family: Some(&crate::addons::TMPL_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

leaf! {
    /// `Server.Items.WoodenCoffinComponent`.
    WOODEN_COFFIN_COMPONENT = "Server.Items.WoodenCoffinComponent" : ADDON_COMPONENT, enc;
    label_number: crate::class_props::label_number,
}

/// `WoodenCoffinAddon(bool east)`.
pub fn wooden_coffin_addon_new(w: &mut World, east: bool) -> ItemId {
    crate::addons::tmpl_direction_addon(w, &WOODEN_COFFIN_ADDON, east)
}
leaf! {
    /// `Server.Items.WoodenCoffinAddon`.
    WOODEN_COFFIN_ADDON = "Server.Items.WoodenCoffinAddon" : BASE_ADDON, enc;
    family: Some(&crate::addons::TMPL_ADDON_CLASS),
}

/// `WoodenCoffinDeed()`.
pub fn wooden_coffin_deed_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_deed(w, &WOODEN_COFFIN_DEED)
}
leaf! {
    /// `Server.Items.WoodenCoffinDeed`.
    WOODEN_COFFIN_DEED = "Server.Items.WoodenCoffinDeed" : BASE_ADDON_DEED, enc;
    construct: Some(wooden_coffin_deed_new),
    family: Some(&crate::addons::TMPL_DIRECTION_DEED_CLASS),
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
