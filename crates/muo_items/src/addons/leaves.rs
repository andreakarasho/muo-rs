//! The addons, components and deeds that are plain tables (constants and `AddComponent` rows). Those with logic are
//! hand-written next to this file.

use muo_content::items::addon::{
    self, ADDON_COMPONENT, BASE_ADDON, BASE_ADDON_DEED,
};
use muo_content::items::addon_container::{
    self, BASE_ADDON_CONTAINER, BASE_ADDON_CONTAINER_DEED,
};
use muo_server::World;
use muo_server::ids::ItemId;
use muo_server::item::ItemType;
use muo_server::type_registry::TypeRegistry;

use crate::simple::leaf;

#[allow(dead_code)]
fn item_hue(w: &mut World, id: ItemId, value: i32) {
    crate::simple::hue(w, id, value);
}

/// `AbbatoirAddon()`.
pub fn abbatoir_addon_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_addon(w, &ABBATOIR_ADDON)
}
/// `AbbatoirAddon()`'s `AddComponent` calls (its template), for a derived class too (`SummoningAltar`).
pub fn abbatoir_components(w: &mut World, id: ItemId) {
    muo_content::templates::apply_item_as(w, id, ABBATOIR_ADDON.full_name);
}
leaf! {
    /// `Server.Items.AbbatoirAddon`.
    ABBATOIR_ADDON = "Server.Items.AbbatoirAddon" : BASE_ADDON, v0;
    construct: Some(abbatoir_addon_new),
    family: Some(&crate::addons::TMPL_ADDON_CLASS),
}

/// `AbbatoirDeed()`.
pub fn abbatoir_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &ABBATOIR_DEED)
}
leaf! {
    /// `Server.Items.AbbatoirDeed`.
    ABBATOIR_DEED = "Server.Items.AbbatoirDeed" : BASE_ADDON_DEED, v0;
    construct: Some(abbatoir_deed_new),
    family: Some(&crate::addons::TMPL_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `AlchemistTableEastAddon()`.
pub fn alchemist_table_east_addon_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_addon(w, &ALCHEMIST_TABLE_EAST_ADDON)
}
leaf! {
    /// `Server.Items.AlchemistTableEastAddon`.
    ALCHEMIST_TABLE_EAST_ADDON = "Server.Items.AlchemistTableEastAddon" : BASE_ADDON, enc;
    construct: Some(alchemist_table_east_addon_new),
    family: Some(&crate::addons::TMPL_ADDON_CLASS),
}

/// `AlchemistTableEastDeed()`.
pub fn alchemist_table_east_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &ALCHEMIST_TABLE_EAST_DEED)
}
leaf! {
    /// `Server.Items.AlchemistTableEastDeed`.
    ALCHEMIST_TABLE_EAST_DEED = "Server.Items.AlchemistTableEastDeed" : BASE_ADDON_DEED, enc;
    construct: Some(alchemist_table_east_deed_new),
    family: Some(&crate::addons::TMPL_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `AlchemistTableSouthAddon()`.
pub fn alchemist_table_south_addon_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_addon(w, &ALCHEMIST_TABLE_SOUTH_ADDON)
}
leaf! {
    /// `Server.Items.AlchemistTableSouthAddon`.
    ALCHEMIST_TABLE_SOUTH_ADDON = "Server.Items.AlchemistTableSouthAddon" : BASE_ADDON, enc;
    construct: Some(alchemist_table_south_addon_new),
    family: Some(&crate::addons::TMPL_ADDON_CLASS),
}

/// `AlchemistTableSouthDeed()`.
pub fn alchemist_table_south_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &ALCHEMIST_TABLE_SOUTH_DEED)
}
leaf! {
    /// `Server.Items.AlchemistTableSouthDeed`.
    ALCHEMIST_TABLE_SOUTH_DEED = "Server.Items.AlchemistTableSouthDeed" : BASE_ADDON_DEED, enc;
    construct: Some(alchemist_table_south_deed_new),
    family: Some(&crate::addons::TMPL_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `AnvilEastAddon()`.
pub fn anvil_east_addon_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_addon(w, &ANVIL_EAST_ADDON)
}
leaf! {
    /// `Server.Items.AnvilEastAddon`.
    ANVIL_EAST_ADDON = "Server.Items.AnvilEastAddon" : BASE_ADDON, v0;
    construct: Some(anvil_east_addon_new),
    family: Some(&crate::addons::TMPL_ADDON_CLASS),
}

/// `AnvilEastDeed()`.
pub fn anvil_east_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &ANVIL_EAST_DEED)
}
leaf! {
    /// `Server.Items.AnvilEastDeed`.
    ANVIL_EAST_DEED = "Server.Items.AnvilEastDeed" : BASE_ADDON_DEED, v0;
    construct: Some(anvil_east_deed_new),
    family: Some(&crate::addons::TMPL_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `AnvilSouthAddon()`.
pub fn anvil_south_addon_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_addon(w, &ANVIL_SOUTH_ADDON)
}
leaf! {
    /// `Server.Items.AnvilSouthAddon`.
    ANVIL_SOUTH_ADDON = "Server.Items.AnvilSouthAddon" : BASE_ADDON, v0;
    construct: Some(anvil_south_addon_new),
    family: Some(&crate::addons::TMPL_ADDON_CLASS),
}

/// `AnvilSouthDeed()`.
pub fn anvil_south_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &ANVIL_SOUTH_DEED)
}
leaf! {
    /// `Server.Items.AnvilSouthDeed`.
    ANVIL_SOUTH_DEED = "Server.Items.AnvilSouthDeed" : BASE_ADDON_DEED, v0;
    construct: Some(anvil_south_deed_new),
    family: Some(&crate::addons::TMPL_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `ArcaneBookshelfEastAddon()`.
pub fn arcane_bookshelf_east_addon_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_container(w, &ARCANE_BOOKSHELF_EAST_ADDON)
}
leaf! {
    /// `Server.Items.ArcaneBookshelfEastAddon`.
    ARCANE_BOOKSHELF_EAST_ADDON = "Server.Items.ArcaneBookshelfEastAddon" : BASE_ADDON_CONTAINER, enc;
    construct: Some(arcane_bookshelf_east_addon_new),
    family: Some(&crate::addons::TMPL_CONTAINER_CLASS),
    default_gump_id: crate::class_props::default_gump_id,
    default_drop_sound: crate::class_props::default_drop_sound,
}

/// `ArcaneBookshelfEastDeed()`.
pub fn arcane_bookshelf_east_deed_new(w: &mut World) -> ItemId {
    addon_container::new_deed(w, &ARCANE_BOOKSHELF_EAST_DEED)
}
leaf! {
    /// `Server.Items.ArcaneBookshelfEastDeed`.
    ARCANE_BOOKSHELF_EAST_DEED = "Server.Items.ArcaneBookshelfEastDeed" : BASE_ADDON_CONTAINER_DEED, enc;
    construct: Some(arcane_bookshelf_east_deed_new),
    family: Some(&crate::addons::TMPL_CONTAINER_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `ArcaneBookshelfSouthAddon()`.
pub fn arcane_bookshelf_south_addon_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_container(w, &ARCANE_BOOKSHELF_SOUTH_ADDON)
}
leaf! {
    /// `Server.Items.ArcaneBookshelfSouthAddon`.
    ARCANE_BOOKSHELF_SOUTH_ADDON = "Server.Items.ArcaneBookshelfSouthAddon" : BASE_ADDON_CONTAINER, enc;
    construct: Some(arcane_bookshelf_south_addon_new),
    family: Some(&crate::addons::TMPL_CONTAINER_CLASS),
    default_gump_id: crate::class_props::default_gump_id,
    default_drop_sound: crate::class_props::default_drop_sound,
    label_number: crate::class_props::label_number,
}

/// `ArcaneBookshelfSouthDeed()`.
pub fn arcane_bookshelf_south_deed_new(w: &mut World) -> ItemId {
    addon_container::new_deed(w, &ARCANE_BOOKSHELF_SOUTH_DEED)
}
leaf! {
    /// `Server.Items.ArcaneBookshelfSouthDeed`.
    ARCANE_BOOKSHELF_SOUTH_DEED = "Server.Items.ArcaneBookshelfSouthDeed" : BASE_ADDON_CONTAINER_DEED, enc;
    construct: Some(arcane_bookshelf_south_deed_new),
    family: Some(&crate::addons::TMPL_CONTAINER_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `ArcaneCircleAddon()`.
pub fn arcane_circle_addon_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_addon(w, &ARCANE_CIRCLE_ADDON)
}
leaf! {
    /// `Server.Items.ArcaneCircleAddon`.
    ARCANE_CIRCLE_ADDON = "Server.Items.ArcaneCircleAddon" : BASE_ADDON, enc;
    construct: Some(arcane_circle_addon_new),
    family: Some(&crate::addons::TMPL_ADDON_CLASS),
}

/// `ArcaneCircleDeed()`.
pub fn arcane_circle_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &ARCANE_CIRCLE_DEED)
}
leaf! {
    /// `Server.Items.ArcaneCircleDeed`.
    ARCANE_CIRCLE_DEED = "Server.Items.ArcaneCircleDeed" : BASE_ADDON_DEED, enc;
    construct: Some(arcane_circle_deed_new),
    family: Some(&crate::addons::TMPL_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `ArcanistStatueEastAddon()`.
pub fn arcanist_statue_east_addon_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_addon(w, &ARCANIST_STATUE_EAST_ADDON)
}
leaf! {
    /// `Server.Items.ArcanistStatueEastAddon`.
    ARCANIST_STATUE_EAST_ADDON = "Server.Items.ArcanistStatueEastAddon" : BASE_ADDON, enc;
    construct: Some(arcanist_statue_east_addon_new),
    family: Some(&crate::addons::TMPL_ADDON_CLASS),
}

/// `ArcanistStatueEastDeed()`.
pub fn arcanist_statue_east_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &ARCANIST_STATUE_EAST_DEED)
}
leaf! {
    /// `Server.Items.ArcanistStatueEastDeed`.
    ARCANIST_STATUE_EAST_DEED = "Server.Items.ArcanistStatueEastDeed" : BASE_ADDON_DEED, enc;
    construct: Some(arcanist_statue_east_deed_new),
    family: Some(&crate::addons::TMPL_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `ArcanistStatueSouthAddon()`.
pub fn arcanist_statue_south_addon_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_addon(w, &ARCANIST_STATUE_SOUTH_ADDON)
}
leaf! {
    /// `Server.Items.ArcanistStatueSouthAddon`.
    ARCANIST_STATUE_SOUTH_ADDON = "Server.Items.ArcanistStatueSouthAddon" : BASE_ADDON, enc;
    construct: Some(arcanist_statue_south_addon_new),
    family: Some(&crate::addons::TMPL_ADDON_CLASS),
}

/// `ArcanistStatueSouthDeed()`.
pub fn arcanist_statue_south_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &ARCANIST_STATUE_SOUTH_DEED)
}
leaf! {
    /// `Server.Items.ArcanistStatueSouthDeed`.
    ARCANIST_STATUE_SOUTH_DEED = "Server.Items.ArcanistStatueSouthDeed" : BASE_ADDON_DEED, enc;
    construct: Some(arcanist_statue_south_deed_new),
    family: Some(&crate::addons::TMPL_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `BrownBearRugEastAddon()`.
pub fn brown_bear_rug_east_addon_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_addon(w, &BROWN_BEAR_RUG_EAST_ADDON)
}
leaf! {
    /// `Server.Items.BrownBearRugEastAddon`.
    BROWN_BEAR_RUG_EAST_ADDON = "Server.Items.BrownBearRugEastAddon" : BASE_ADDON, v0;
    construct: Some(brown_bear_rug_east_addon_new),
    family: Some(&crate::addons::TMPL_ADDON_CLASS),
}

/// `BrownBearRugEastDeed()`.
pub fn brown_bear_rug_east_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &BROWN_BEAR_RUG_EAST_DEED)
}
leaf! {
    /// `Server.Items.BrownBearRugEastDeed`.
    BROWN_BEAR_RUG_EAST_DEED = "Server.Items.BrownBearRugEastDeed" : BASE_ADDON_DEED, v0;
    construct: Some(brown_bear_rug_east_deed_new),
    family: Some(&crate::addons::TMPL_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `BrownBearRugSouthAddon()`.
pub fn brown_bear_rug_south_addon_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_addon(w, &BROWN_BEAR_RUG_SOUTH_ADDON)
}
leaf! {
    /// `Server.Items.BrownBearRugSouthAddon`.
    BROWN_BEAR_RUG_SOUTH_ADDON = "Server.Items.BrownBearRugSouthAddon" : BASE_ADDON, v0;
    construct: Some(brown_bear_rug_south_addon_new),
    family: Some(&crate::addons::TMPL_ADDON_CLASS),
}

/// `BrownBearRugSouthDeed()`.
pub fn brown_bear_rug_south_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &BROWN_BEAR_RUG_SOUTH_DEED)
}
leaf! {
    /// `Server.Items.BrownBearRugSouthDeed`.
    BROWN_BEAR_RUG_SOUTH_DEED = "Server.Items.BrownBearRugSouthDeed" : BASE_ADDON_DEED, v0;
    construct: Some(brown_bear_rug_south_deed_new),
    family: Some(&crate::addons::TMPL_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `PolarBearRugEastAddon()`.
pub fn polar_bear_rug_east_addon_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_addon(w, &POLAR_BEAR_RUG_EAST_ADDON)
}
leaf! {
    /// `Server.Items.PolarBearRugEastAddon`.
    POLAR_BEAR_RUG_EAST_ADDON = "Server.Items.PolarBearRugEastAddon" : BASE_ADDON, v0;
    construct: Some(polar_bear_rug_east_addon_new),
    family: Some(&crate::addons::TMPL_ADDON_CLASS),
}

/// `PolarBearRugEastDeed()`.
pub fn polar_bear_rug_east_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &POLAR_BEAR_RUG_EAST_DEED)
}
leaf! {
    /// `Server.Items.PolarBearRugEastDeed`.
    POLAR_BEAR_RUG_EAST_DEED = "Server.Items.PolarBearRugEastDeed" : BASE_ADDON_DEED, v0;
    construct: Some(polar_bear_rug_east_deed_new),
    family: Some(&crate::addons::TMPL_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `PolarBearRugSouthAddon()`.
pub fn polar_bear_rug_south_addon_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_addon(w, &POLAR_BEAR_RUG_SOUTH_ADDON)
}
leaf! {
    /// `Server.Items.PolarBearRugSouthAddon`.
    POLAR_BEAR_RUG_SOUTH_ADDON = "Server.Items.PolarBearRugSouthAddon" : BASE_ADDON, v0;
    construct: Some(polar_bear_rug_south_addon_new),
    family: Some(&crate::addons::TMPL_ADDON_CLASS),
}

/// `PolarBearRugSouthDeed()`.
pub fn polar_bear_rug_south_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &POLAR_BEAR_RUG_SOUTH_DEED)
}
leaf! {
    /// `Server.Items.PolarBearRugSouthDeed`.
    POLAR_BEAR_RUG_SOUTH_DEED = "Server.Items.PolarBearRugSouthDeed" : BASE_ADDON_DEED, v0;
    construct: Some(polar_bear_rug_south_deed_new),
    family: Some(&crate::addons::TMPL_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `BloodPentagram()`.
pub fn blood_pentagram_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_addon(w, &BLOOD_PENTAGRAM)
}
leaf! {
    /// `Server.Items.BloodPentagram`.
    BLOOD_PENTAGRAM = "Server.Items.BloodPentagram" : BASE_ADDON, v0;
    construct: Some(blood_pentagram_new),
}

/// `ElvenBedEastAddon()`.
pub fn elven_bed_east_addon_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_addon(w, &ELVEN_BED_EAST_ADDON)
}
leaf! {
    /// `Server.Items.ElvenBedEastAddon`.
    ELVEN_BED_EAST_ADDON = "Server.Items.ElvenBedEastAddon" : BASE_ADDON, enc;
    construct: Some(elven_bed_east_addon_new),
    family: Some(&crate::addons::TMPL_ADDON_CLASS),
}

/// `ElvenBedEastDeed()`.
pub fn elven_bed_east_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &ELVEN_BED_EAST_DEED)
}
leaf! {
    /// `Server.Items.ElvenBedEastDeed`.
    ELVEN_BED_EAST_DEED = "Server.Items.ElvenBedEastDeed" : BASE_ADDON_DEED, enc;
    construct: Some(elven_bed_east_deed_new),
    family: Some(&crate::addons::TMPL_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `ElvenBedSouthAddon()`.
pub fn elven_bed_south_addon_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_addon(w, &ELVEN_BED_SOUTH_ADDON)
}
leaf! {
    /// `Server.Items.ElvenBedSouthAddon`.
    ELVEN_BED_SOUTH_ADDON = "Server.Items.ElvenBedSouthAddon" : BASE_ADDON, enc;
    construct: Some(elven_bed_south_addon_new),
    family: Some(&crate::addons::TMPL_ADDON_CLASS),
}

/// `ElvenBedSouthDeed()`.
pub fn elven_bed_south_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &ELVEN_BED_SOUTH_DEED)
}
leaf! {
    /// `Server.Items.ElvenBedSouthDeed`.
    ELVEN_BED_SOUTH_DEED = "Server.Items.ElvenBedSouthDeed" : BASE_ADDON_DEED, enc;
    construct: Some(elven_bed_south_deed_new),
    family: Some(&crate::addons::TMPL_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `ElvenDresserEastAddon()`.
pub fn elven_dresser_east_addon_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_container(w, &ELVEN_DRESSER_EAST_ADDON)
}
leaf! {
    /// `Server.Items.ElvenDresserEastAddon`.
    ELVEN_DRESSER_EAST_ADDON = "Server.Items.ElvenDresserEastAddon" : BASE_ADDON_CONTAINER, enc;
    construct: Some(elven_dresser_east_addon_new),
    family: Some(&crate::addons::TMPL_CONTAINER_CLASS),
    default_gump_id: crate::class_props::default_gump_id,
    default_drop_sound: crate::class_props::default_drop_sound,
}

/// `ElvenDresserEastDeed()`.
pub fn elven_dresser_east_deed_new(w: &mut World) -> ItemId {
    addon_container::new_deed(w, &ELVEN_DRESSER_EAST_DEED)
}
leaf! {
    /// `Server.Items.ElvenDresserEastDeed`.
    ELVEN_DRESSER_EAST_DEED = "Server.Items.ElvenDresserEastDeed" : BASE_ADDON_CONTAINER_DEED, enc;
    construct: Some(elven_dresser_east_deed_new),
    family: Some(&crate::addons::TMPL_CONTAINER_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `ElvenDresserSouthAddon()`.
pub fn elven_dresser_south_addon_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_container(w, &ELVEN_DRESSER_SOUTH_ADDON)
}
leaf! {
    /// `Server.Items.ElvenDresserSouthAddon`.
    ELVEN_DRESSER_SOUTH_ADDON = "Server.Items.ElvenDresserSouthAddon" : BASE_ADDON_CONTAINER, enc;
    construct: Some(elven_dresser_south_addon_new),
    family: Some(&crate::addons::TMPL_CONTAINER_CLASS),
    default_gump_id: crate::class_props::default_gump_id,
    default_drop_sound: crate::class_props::default_drop_sound,
}

/// `ElvenDresserSouthDeed()`.
pub fn elven_dresser_south_deed_new(w: &mut World) -> ItemId {
    addon_container::new_deed(w, &ELVEN_DRESSER_SOUTH_DEED)
}
leaf! {
    /// `Server.Items.ElvenDresserSouthDeed`.
    ELVEN_DRESSER_SOUTH_DEED = "Server.Items.ElvenDresserSouthDeed" : BASE_ADDON_CONTAINER_DEED, enc;
    construct: Some(elven_dresser_south_deed_new),
    family: Some(&crate::addons::TMPL_CONTAINER_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `ElvenForgeAddon()`.
pub fn elven_forge_addon_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_addon(w, &ELVEN_FORGE_ADDON)
}
leaf! {
    /// `Server.Items.ElvenForgeAddon`.
    ELVEN_FORGE_ADDON = "Server.Items.ElvenForgeAddon" : BASE_ADDON, enc;
    construct: Some(elven_forge_addon_new),
    family: Some(&crate::addons::TMPL_ADDON_CLASS),
    label_number: crate::class_props::label_number,
}

/// `ElvenForgeDeed()`.
pub fn elven_forge_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &ELVEN_FORGE_DEED)
}
leaf! {
    /// `Server.Items.ElvenForgeDeed`.
    ELVEN_FORGE_DEED = "Server.Items.ElvenForgeDeed" : BASE_ADDON_DEED, enc;
    construct: Some(elven_forge_deed_new),
    family: Some(&crate::addons::TMPL_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `ElvenLoveseatEastAddon()`.
pub fn elven_loveseat_east_addon_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_addon(w, &ELVEN_LOVESEAT_EAST_ADDON)
}
leaf! {
    /// `Server.Items.ElvenLoveseatEastAddon`.
    ELVEN_LOVESEAT_EAST_ADDON = "Server.Items.ElvenLoveseatEastAddon" : BASE_ADDON, enc;
    construct: Some(elven_loveseat_east_addon_new),
    family: Some(&crate::addons::TMPL_ADDON_CLASS),
}

/// `ElvenLoveseatEastDeed()`.
pub fn elven_loveseat_east_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &ELVEN_LOVESEAT_EAST_DEED)
}
leaf! {
    /// `Server.Items.ElvenLoveseatEastDeed`.
    ELVEN_LOVESEAT_EAST_DEED = "Server.Items.ElvenLoveseatEastDeed" : BASE_ADDON_DEED, enc;
    construct: Some(elven_loveseat_east_deed_new),
    family: Some(&crate::addons::TMPL_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `ElvenLoveseatSouthAddon()`.
pub fn elven_loveseat_south_addon_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_addon(w, &ELVEN_LOVESEAT_SOUTH_ADDON)
}
leaf! {
    /// `Server.Items.ElvenLoveseatSouthAddon`.
    ELVEN_LOVESEAT_SOUTH_ADDON = "Server.Items.ElvenLoveseatSouthAddon" : BASE_ADDON, enc;
    construct: Some(elven_loveseat_south_addon_new),
    family: Some(&crate::addons::TMPL_ADDON_CLASS),
}

/// `ElvenLoveseatSouthDeed()`.
pub fn elven_loveseat_south_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &ELVEN_LOVESEAT_SOUTH_DEED)
}
leaf! {
    /// `Server.Items.ElvenLoveseatSouthDeed`.
    ELVEN_LOVESEAT_SOUTH_DEED = "Server.Items.ElvenLoveseatSouthDeed" : BASE_ADDON_DEED, enc;
    construct: Some(elven_loveseat_south_deed_new),
    family: Some(&crate::addons::TMPL_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `ElvenStoveEastAddon()`.
pub fn elven_stove_east_addon_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_addon(w, &ELVEN_STOVE_EAST_ADDON)
}
leaf! {
    /// `Server.Items.ElvenStoveEastAddon`.
    ELVEN_STOVE_EAST_ADDON = "Server.Items.ElvenStoveEastAddon" : BASE_ADDON, enc;
    construct: Some(elven_stove_east_addon_new),
    family: Some(&crate::addons::TMPL_ADDON_CLASS),
}

/// `ElvenStoveEastDeed()`.
pub fn elven_stove_east_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &ELVEN_STOVE_EAST_DEED)
}
leaf! {
    /// `Server.Items.ElvenStoveEastDeed`.
    ELVEN_STOVE_EAST_DEED = "Server.Items.ElvenStoveEastDeed" : BASE_ADDON_DEED, enc;
    construct: Some(elven_stove_east_deed_new),
    family: Some(&crate::addons::TMPL_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `ElvenStoveSouthAddon()`.
pub fn elven_stove_south_addon_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_addon(w, &ELVEN_STOVE_SOUTH_ADDON)
}
leaf! {
    /// `Server.Items.ElvenStoveSouthAddon`.
    ELVEN_STOVE_SOUTH_ADDON = "Server.Items.ElvenStoveSouthAddon" : BASE_ADDON, enc;
    construct: Some(elven_stove_south_addon_new),
    family: Some(&crate::addons::TMPL_ADDON_CLASS),
}

/// `ElvenStoveSouthDeed()`.
pub fn elven_stove_south_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &ELVEN_STOVE_SOUTH_DEED)
}
leaf! {
    /// `Server.Items.ElvenStoveSouthDeed`.
    ELVEN_STOVE_SOUTH_DEED = "Server.Items.ElvenStoveSouthDeed" : BASE_ADDON_DEED, enc;
    construct: Some(elven_stove_south_deed_new),
    family: Some(&crate::addons::TMPL_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `ElvenWashBasinEastAddon()`.
pub fn elven_wash_basin_east_addon_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_container(w, &ELVEN_WASH_BASIN_EAST_ADDON)
}
leaf! {
    /// `Server.Items.ElvenWashBasinEastAddon`.
    ELVEN_WASH_BASIN_EAST_ADDON = "Server.Items.ElvenWashBasinEastAddon" : BASE_ADDON_CONTAINER, enc;
    construct: Some(elven_wash_basin_east_addon_new),
    family: Some(&crate::addons::TMPL_CONTAINER_CLASS),
    default_gump_id: crate::class_props::default_gump_id,
    default_drop_sound: crate::class_props::default_drop_sound,
}

/// `ElvenWashBasinEastDeed()`.
pub fn elven_wash_basin_east_deed_new(w: &mut World) -> ItemId {
    addon_container::new_deed(w, &ELVEN_WASH_BASIN_EAST_DEED)
}
leaf! {
    /// `Server.Items.ElvenWashBasinEastDeed`.
    ELVEN_WASH_BASIN_EAST_DEED = "Server.Items.ElvenWashBasinEastDeed" : BASE_ADDON_CONTAINER_DEED, enc;
    construct: Some(elven_wash_basin_east_deed_new),
    family: Some(&crate::addons::TMPL_CONTAINER_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `ElvenWashBasinSouthAddon()`.
pub fn elven_wash_basin_south_addon_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_container(w, &ELVEN_WASH_BASIN_SOUTH_ADDON)
}
leaf! {
    /// `Server.Items.ElvenWashBasinSouthAddon`.
    ELVEN_WASH_BASIN_SOUTH_ADDON = "Server.Items.ElvenWashBasinSouthAddon" : BASE_ADDON_CONTAINER, enc;
    construct: Some(elven_wash_basin_south_addon_new),
    family: Some(&crate::addons::TMPL_CONTAINER_CLASS),
    default_gump_id: crate::class_props::default_gump_id,
    default_drop_sound: crate::class_props::default_drop_sound,
}

/// `ElvenWashBasinSouthDeed()`.
pub fn elven_wash_basin_south_deed_new(w: &mut World) -> ItemId {
    addon_container::new_deed(w, &ELVEN_WASH_BASIN_SOUTH_DEED)
}
leaf! {
    /// `Server.Items.ElvenWashBasinSouthDeed`.
    ELVEN_WASH_BASIN_SOUTH_DEED = "Server.Items.ElvenWashBasinSouthDeed" : BASE_ADDON_CONTAINER_DEED, enc;
    construct: Some(elven_wash_basin_south_deed_new),
    family: Some(&crate::addons::TMPL_CONTAINER_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `FancyElvenTableEastAddon()`.
pub fn fancy_elven_table_east_addon_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_addon(w, &FANCY_ELVEN_TABLE_EAST_ADDON)
}
leaf! {
    /// `Server.Items.FancyElvenTableEastAddon`.
    FANCY_ELVEN_TABLE_EAST_ADDON = "Server.Items.FancyElvenTableEastAddon" : BASE_ADDON, enc;
    construct: Some(fancy_elven_table_east_addon_new),
    family: Some(&crate::addons::TMPL_ADDON_CLASS),
}

/// `FancyElvenTableEastDeed()`.
pub fn fancy_elven_table_east_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &FANCY_ELVEN_TABLE_EAST_DEED)
}
leaf! {
    /// `Server.Items.FancyElvenTableEastDeed`.
    FANCY_ELVEN_TABLE_EAST_DEED = "Server.Items.FancyElvenTableEastDeed" : BASE_ADDON_DEED, enc;
    construct: Some(fancy_elven_table_east_deed_new),
    family: Some(&crate::addons::TMPL_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `FancyElvenTableSouthAddon()`.
pub fn fancy_elven_table_south_addon_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_addon(w, &FANCY_ELVEN_TABLE_SOUTH_ADDON)
}
leaf! {
    /// `Server.Items.FancyElvenTableSouthAddon`.
    FANCY_ELVEN_TABLE_SOUTH_ADDON = "Server.Items.FancyElvenTableSouthAddon" : BASE_ADDON, enc;
    construct: Some(fancy_elven_table_south_addon_new),
    family: Some(&crate::addons::TMPL_ADDON_CLASS),
}

/// `FancyElvenTableSouthDeed()`.
pub fn fancy_elven_table_south_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &FANCY_ELVEN_TABLE_SOUTH_DEED)
}
leaf! {
    /// `Server.Items.FancyElvenTableSouthDeed`.
    FANCY_ELVEN_TABLE_SOUTH_DEED = "Server.Items.FancyElvenTableSouthDeed" : BASE_ADDON_DEED, enc;
    construct: Some(fancy_elven_table_south_deed_new),
    family: Some(&crate::addons::TMPL_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `FireColumnAddon(bool bloody = false)`: a bloody column adds its `addonComponentsBloody` prop rows.
pub fn fire_column_addon_new(w: &mut World, bloody: bool) -> ItemId {
    let id = crate::addons::tmpl_addon(w, &FIRE_COLUMN_ADDON);
    if bloody
        && let Some(rows) =
            muo_content::templates::props::item(w, id, "addonComponentsBloody").cloned()
    {
        crate::template_keys_misc2::add_component_rows(w, id, &rows);
    }
    id
}
leaf! {
    /// `Server.Items.FireColumnAddon`.
    FIRE_COLUMN_ADDON = "Server.Items.FireColumnAddon" : BASE_ADDON, v0;
    construct: Some(|w| fire_column_addon_new(w, false)),
}

/// `LightFlowerTapestryEastAddon()`.
pub fn light_flower_tapestry_east_addon_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_addon(w, &LIGHT_FLOWER_TAPESTRY_EAST_ADDON)
}
leaf! {
    /// `Server.Items.LightFlowerTapestryEastAddon`.
    LIGHT_FLOWER_TAPESTRY_EAST_ADDON = "Server.Items.LightFlowerTapestryEastAddon" : BASE_ADDON, v0;
    construct: Some(light_flower_tapestry_east_addon_new),
    family: Some(&crate::addons::TMPL_ADDON_CLASS),
}

/// `LightFlowerTapestryEastDeed()`.
pub fn light_flower_tapestry_east_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &LIGHT_FLOWER_TAPESTRY_EAST_DEED)
}
leaf! {
    /// `Server.Items.LightFlowerTapestryEastDeed`.
    LIGHT_FLOWER_TAPESTRY_EAST_DEED = "Server.Items.LightFlowerTapestryEastDeed" : BASE_ADDON_DEED, v0;
    construct: Some(light_flower_tapestry_east_deed_new),
    family: Some(&crate::addons::TMPL_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `LightFlowerTapestrySouthAddon()`.
pub fn light_flower_tapestry_south_addon_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_addon(w, &LIGHT_FLOWER_TAPESTRY_SOUTH_ADDON)
}
leaf! {
    /// `Server.Items.LightFlowerTapestrySouthAddon`.
    LIGHT_FLOWER_TAPESTRY_SOUTH_ADDON = "Server.Items.LightFlowerTapestrySouthAddon" : BASE_ADDON, v0;
    construct: Some(light_flower_tapestry_south_addon_new),
    family: Some(&crate::addons::TMPL_ADDON_CLASS),
}

/// `LightFlowerTapestrySouthDeed()`.
pub fn light_flower_tapestry_south_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &LIGHT_FLOWER_TAPESTRY_SOUTH_DEED)
}
leaf! {
    /// `Server.Items.LightFlowerTapestrySouthDeed`.
    LIGHT_FLOWER_TAPESTRY_SOUTH_DEED = "Server.Items.LightFlowerTapestrySouthDeed" : BASE_ADDON_DEED, v0;
    construct: Some(light_flower_tapestry_south_deed_new),
    family: Some(&crate::addons::TMPL_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `DarkFlowerTapestryEastAddon()`.
pub fn dark_flower_tapestry_east_addon_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_addon(w, &DARK_FLOWER_TAPESTRY_EAST_ADDON)
}
leaf! {
    /// `Server.Items.DarkFlowerTapestryEastAddon`.
    DARK_FLOWER_TAPESTRY_EAST_ADDON = "Server.Items.DarkFlowerTapestryEastAddon" : BASE_ADDON, v0;
    construct: Some(dark_flower_tapestry_east_addon_new),
    family: Some(&crate::addons::TMPL_ADDON_CLASS),
}

/// `DarkFlowerTapestryEastDeed()`.
pub fn dark_flower_tapestry_east_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &DARK_FLOWER_TAPESTRY_EAST_DEED)
}
leaf! {
    /// `Server.Items.DarkFlowerTapestryEastDeed`.
    DARK_FLOWER_TAPESTRY_EAST_DEED = "Server.Items.DarkFlowerTapestryEastDeed" : BASE_ADDON_DEED, v0;
    construct: Some(dark_flower_tapestry_east_deed_new),
    family: Some(&crate::addons::TMPL_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `DarkFlowerTapestrySouthAddon()`.
pub fn dark_flower_tapestry_south_addon_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_addon(w, &DARK_FLOWER_TAPESTRY_SOUTH_ADDON)
}
leaf! {
    /// `Server.Items.DarkFlowerTapestrySouthAddon`.
    DARK_FLOWER_TAPESTRY_SOUTH_ADDON = "Server.Items.DarkFlowerTapestrySouthAddon" : BASE_ADDON, v0;
    construct: Some(dark_flower_tapestry_south_addon_new),
    family: Some(&crate::addons::TMPL_ADDON_CLASS),
}

/// `DarkFlowerTapestrySouthDeed()`.
pub fn dark_flower_tapestry_south_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &DARK_FLOWER_TAPESTRY_SOUTH_DEED)
}
leaf! {
    /// `Server.Items.DarkFlowerTapestrySouthDeed`.
    DARK_FLOWER_TAPESTRY_SOUTH_DEED = "Server.Items.DarkFlowerTapestrySouthDeed" : BASE_ADDON_DEED, v0;
    construct: Some(dark_flower_tapestry_south_deed_new),
    family: Some(&crate::addons::TMPL_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `GiantWeb1()`.
pub fn giant_web1_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_addon(w, &GIANT_WEB1)
}
leaf! {
    /// `Server.Items.GiantWeb1`.
    GIANT_WEB1 = "Server.Items.GiantWeb1" : BASE_ADDON, enc;
    construct: Some(giant_web1_new),
}

/// `GiantWeb2()`.
pub fn giant_web2_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_addon(w, &GIANT_WEB2)
}
leaf! {
    /// `Server.Items.GiantWeb2`.
    GIANT_WEB2 = "Server.Items.GiantWeb2" : BASE_ADDON, enc;
    construct: Some(giant_web2_new),
}

/// `GiantWeb3()`.
pub fn giant_web3_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_addon(w, &GIANT_WEB3)
}
leaf! {
    /// `Server.Items.GiantWeb3`.
    GIANT_WEB3 = "Server.Items.GiantWeb3" : BASE_ADDON, enc;
    construct: Some(giant_web3_new),
}

/// `GiantWeb4()`.
pub fn giant_web4_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_addon(w, &GIANT_WEB4)
}
leaf! {
    /// `Server.Items.GiantWeb4`.
    GIANT_WEB4 = "Server.Items.GiantWeb4" : BASE_ADDON, enc;
    construct: Some(giant_web4_new),
}

/// `GiantWeb5()`.
pub fn giant_web5_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_addon(w, &GIANT_WEB5)
}
leaf! {
    /// `Server.Items.GiantWeb5`.
    GIANT_WEB5 = "Server.Items.GiantWeb5" : BASE_ADDON, enc;
    construct: Some(giant_web5_new),
}

/// `GiantWeb6()`.
pub fn giant_web6_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_addon(w, &GIANT_WEB6)
}
leaf! {
    /// `Server.Items.GiantWeb6`.
    GIANT_WEB6 = "Server.Items.GiantWeb6" : BASE_ADDON, enc;
    construct: Some(giant_web6_new),
}

/// `GozaMatEastAddon(int hue = 0)`.
pub fn goza_mat_east_addon_new(w: &mut World, hue: i32) -> ItemId {
    let id = crate::addons::tmpl_addon(w, &GOZA_MAT_EAST_ADDON);
    item_hue(w, id, hue);
    id
}
leaf! {
    /// `Server.Items.GozaMatEastAddon`.
    GOZA_MAT_EAST_ADDON = "Server.Items.GozaMatEastAddon" : BASE_ADDON, v0;
    construct: Some(|w| goza_mat_east_addon_new(w, 0)),
    family: Some(&crate::addons::TMPL_ADDON_CLASS),
}

/// `GozaMatEastDeed()`.
pub fn goza_mat_east_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &GOZA_MAT_EAST_DEED)
}
leaf! {
    /// `Server.Items.GozaMatEastDeed`.
    GOZA_MAT_EAST_DEED = "Server.Items.GozaMatEastDeed" : BASE_ADDON_DEED, v0;
    construct: Some(goza_mat_east_deed_new),
    family: Some(&crate::addons::TMPL_HUED_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `GozaMatSouthAddon(int hue = 0)`.
pub fn goza_mat_south_addon_new(w: &mut World, hue: i32) -> ItemId {
    let id = crate::addons::tmpl_addon(w, &GOZA_MAT_SOUTH_ADDON);
    item_hue(w, id, hue);
    id
}
leaf! {
    /// `Server.Items.GozaMatSouthAddon`.
    GOZA_MAT_SOUTH_ADDON = "Server.Items.GozaMatSouthAddon" : BASE_ADDON, v0;
    construct: Some(|w| goza_mat_south_addon_new(w, 0)),
    family: Some(&crate::addons::TMPL_ADDON_CLASS),
}

/// `GozaMatSouthDeed()`.
pub fn goza_mat_south_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &GOZA_MAT_SOUTH_DEED)
}
leaf! {
    /// `Server.Items.GozaMatSouthDeed`.
    GOZA_MAT_SOUTH_DEED = "Server.Items.GozaMatSouthDeed" : BASE_ADDON_DEED, v0;
    construct: Some(goza_mat_south_deed_new),
    family: Some(&crate::addons::TMPL_HUED_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `SquareGozaMatEastAddon(int hue = 0)`.
pub fn square_goza_mat_east_addon_new(w: &mut World, hue: i32) -> ItemId {
    let id = crate::addons::tmpl_addon(w, &SQUARE_GOZA_MAT_EAST_ADDON);
    item_hue(w, id, hue);
    id
}
leaf! {
    /// `Server.Items.SquareGozaMatEastAddon`.
    SQUARE_GOZA_MAT_EAST_ADDON = "Server.Items.SquareGozaMatEastAddon" : BASE_ADDON, v0;
    construct: Some(|w| square_goza_mat_east_addon_new(w, 0)),
    family: Some(&crate::addons::TMPL_ADDON_CLASS),
    label_number: crate::class_props::label_number,
}

/// `SquareGozaMatEastDeed()`.
pub fn square_goza_mat_east_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &SQUARE_GOZA_MAT_EAST_DEED)
}
leaf! {
    /// `Server.Items.SquareGozaMatEastDeed`.
    SQUARE_GOZA_MAT_EAST_DEED = "Server.Items.SquareGozaMatEastDeed" : BASE_ADDON_DEED, v0;
    construct: Some(square_goza_mat_east_deed_new),
    family: Some(&crate::addons::TMPL_HUED_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `SquareGozaMatSouthAddon(int hue = 0)`.
pub fn square_goza_mat_south_addon_new(w: &mut World, hue: i32) -> ItemId {
    let id = crate::addons::tmpl_addon(w, &SQUARE_GOZA_MAT_SOUTH_ADDON);
    item_hue(w, id, hue);
    id
}
leaf! {
    /// `Server.Items.SquareGozaMatSouthAddon`.
    SQUARE_GOZA_MAT_SOUTH_ADDON = "Server.Items.SquareGozaMatSouthAddon" : BASE_ADDON, v0;
    construct: Some(|w| square_goza_mat_south_addon_new(w, 0)),
    family: Some(&crate::addons::TMPL_ADDON_CLASS),
}

/// `SquareGozaMatSouthDeed()`.
pub fn square_goza_mat_south_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &SQUARE_GOZA_MAT_SOUTH_DEED)
}
leaf! {
    /// `Server.Items.SquareGozaMatSouthDeed`.
    SQUARE_GOZA_MAT_SOUTH_DEED = "Server.Items.SquareGozaMatSouthDeed" : BASE_ADDON_DEED, v0;
    construct: Some(square_goza_mat_south_deed_new),
    family: Some(&crate::addons::TMPL_HUED_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `BrocadeGozaMatEastAddon(int hue = 0)`.
pub fn brocade_goza_mat_east_addon_new(w: &mut World, hue: i32) -> ItemId {
    let id = crate::addons::tmpl_addon(w, &BROCADE_GOZA_MAT_EAST_ADDON);
    item_hue(w, id, hue);
    id
}
leaf! {
    /// `Server.Items.BrocadeGozaMatEastAddon`.
    BROCADE_GOZA_MAT_EAST_ADDON = "Server.Items.BrocadeGozaMatEastAddon" : BASE_ADDON, v0;
    construct: Some(|w| brocade_goza_mat_east_addon_new(w, 0)),
    family: Some(&crate::addons::TMPL_ADDON_CLASS),
}

/// `BrocadeGozaMatEastDeed()`.
pub fn brocade_goza_mat_east_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &BROCADE_GOZA_MAT_EAST_DEED)
}
leaf! {
    /// `Server.Items.BrocadeGozaMatEastDeed`.
    BROCADE_GOZA_MAT_EAST_DEED = "Server.Items.BrocadeGozaMatEastDeed" : BASE_ADDON_DEED, v0;
    construct: Some(brocade_goza_mat_east_deed_new),
    family: Some(&crate::addons::TMPL_HUED_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `BrocadeGozaMatSouthAddon(int hue = 0)`.
pub fn brocade_goza_mat_south_addon_new(w: &mut World, hue: i32) -> ItemId {
    let id = crate::addons::tmpl_addon(w, &BROCADE_GOZA_MAT_SOUTH_ADDON);
    item_hue(w, id, hue);
    id
}
leaf! {
    /// `Server.Items.BrocadeGozaMatSouthAddon`.
    BROCADE_GOZA_MAT_SOUTH_ADDON = "Server.Items.BrocadeGozaMatSouthAddon" : BASE_ADDON, v0;
    construct: Some(|w| brocade_goza_mat_south_addon_new(w, 0)),
    family: Some(&crate::addons::TMPL_ADDON_CLASS),
}

/// `BrocadeGozaMatSouthDeed()`.
pub fn brocade_goza_mat_south_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &BROCADE_GOZA_MAT_SOUTH_DEED)
}
leaf! {
    /// `Server.Items.BrocadeGozaMatSouthDeed`.
    BROCADE_GOZA_MAT_SOUTH_DEED = "Server.Items.BrocadeGozaMatSouthDeed" : BASE_ADDON_DEED, v0;
    construct: Some(brocade_goza_mat_south_deed_new),
    family: Some(&crate::addons::TMPL_HUED_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `BrocadeSquareGozaMatEastAddon(int hue = 0)`.
pub fn brocade_square_goza_mat_east_addon_new(w: &mut World, hue: i32) -> ItemId {
    let id = crate::addons::tmpl_addon(w, &BROCADE_SQUARE_GOZA_MAT_EAST_ADDON);
    item_hue(w, id, hue);
    id
}
leaf! {
    /// `Server.Items.BrocadeSquareGozaMatEastAddon`.
    BROCADE_SQUARE_GOZA_MAT_EAST_ADDON = "Server.Items.BrocadeSquareGozaMatEastAddon" : BASE_ADDON, v0;
    construct: Some(|w| brocade_square_goza_mat_east_addon_new(w, 0)),
    family: Some(&crate::addons::TMPL_ADDON_CLASS),
}

/// `BrocadeSquareGozaMatEastDeed()`.
pub fn brocade_square_goza_mat_east_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &BROCADE_SQUARE_GOZA_MAT_EAST_DEED)
}
leaf! {
    /// `Server.Items.BrocadeSquareGozaMatEastDeed`.
    BROCADE_SQUARE_GOZA_MAT_EAST_DEED = "Server.Items.BrocadeSquareGozaMatEastDeed" : BASE_ADDON_DEED, v0;
    construct: Some(brocade_square_goza_mat_east_deed_new),
    family: Some(&crate::addons::TMPL_HUED_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `BrocadeSquareGozaMatSouthAddon(int hue = 0)`.
pub fn brocade_square_goza_mat_south_addon_new(w: &mut World, hue: i32) -> ItemId {
    let id = crate::addons::tmpl_addon(w, &BROCADE_SQUARE_GOZA_MAT_SOUTH_ADDON);
    item_hue(w, id, hue);
    id
}
leaf! {
    /// `Server.Items.BrocadeSquareGozaMatSouthAddon`.
    BROCADE_SQUARE_GOZA_MAT_SOUTH_ADDON = "Server.Items.BrocadeSquareGozaMatSouthAddon" : BASE_ADDON, v0;
    construct: Some(|w| brocade_square_goza_mat_south_addon_new(w, 0)),
    family: Some(&crate::addons::TMPL_ADDON_CLASS),
}

/// `BrocadeSquareGozaMatSouthDeed()`.
pub fn brocade_square_goza_mat_south_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &BROCADE_SQUARE_GOZA_MAT_SOUTH_DEED)
}
leaf! {
    /// `Server.Items.BrocadeSquareGozaMatSouthDeed`.
    BROCADE_SQUARE_GOZA_MAT_SOUTH_DEED = "Server.Items.BrocadeSquareGozaMatSouthDeed" : BASE_ADDON_DEED, v0;
    construct: Some(brocade_square_goza_mat_south_deed_new),
    family: Some(&crate::addons::TMPL_HUED_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `GrayBrickFireplaceEastAddon()`.
pub fn gray_brick_fireplace_east_addon_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_addon(w, &GRAY_BRICK_FIREPLACE_EAST_ADDON)
}
leaf! {
    /// `Server.Items.GrayBrickFireplaceEastAddon`.
    GRAY_BRICK_FIREPLACE_EAST_ADDON = "Server.Items.GrayBrickFireplaceEastAddon" : BASE_ADDON, v0;
    construct: Some(gray_brick_fireplace_east_addon_new),
    family: Some(&crate::addons::TMPL_ADDON_CLASS),
}

/// `GrayBrickFireplaceEastDeed()`.
pub fn gray_brick_fireplace_east_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &GRAY_BRICK_FIREPLACE_EAST_DEED)
}
leaf! {
    /// `Server.Items.GrayBrickFireplaceEastDeed`.
    GRAY_BRICK_FIREPLACE_EAST_DEED = "Server.Items.GrayBrickFireplaceEastDeed" : BASE_ADDON_DEED, v0;
    construct: Some(gray_brick_fireplace_east_deed_new),
    family: Some(&crate::addons::TMPL_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `GrayBrickFireplaceSouthAddon()`.
pub fn gray_brick_fireplace_south_addon_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_addon(w, &GRAY_BRICK_FIREPLACE_SOUTH_ADDON)
}
leaf! {
    /// `Server.Items.GrayBrickFireplaceSouthAddon`.
    GRAY_BRICK_FIREPLACE_SOUTH_ADDON = "Server.Items.GrayBrickFireplaceSouthAddon" : BASE_ADDON, v0;
    construct: Some(gray_brick_fireplace_south_addon_new),
    family: Some(&crate::addons::TMPL_ADDON_CLASS),
}

/// `GrayBrickFireplaceSouthDeed()`.
pub fn gray_brick_fireplace_south_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &GRAY_BRICK_FIREPLACE_SOUTH_DEED)
}
leaf! {
    /// `Server.Items.GrayBrickFireplaceSouthDeed`.
    GRAY_BRICK_FIREPLACE_SOUTH_DEED = "Server.Items.GrayBrickFireplaceSouthDeed" : BASE_ADDON_DEED, v0;
    construct: Some(gray_brick_fireplace_south_deed_new),
    family: Some(&crate::addons::TMPL_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `LargeBedEastAddon()`.
pub fn large_bed_east_addon_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_addon(w, &LARGE_BED_EAST_ADDON)
}
leaf! {
    /// `Server.Items.LargeBedEastAddon`.
    LARGE_BED_EAST_ADDON = "Server.Items.LargeBedEastAddon" : BASE_ADDON, v0;
    construct: Some(large_bed_east_addon_new),
    family: Some(&crate::addons::TMPL_ADDON_CLASS),
}

/// `LargeBedEastDeed()`.
pub fn large_bed_east_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &LARGE_BED_EAST_DEED)
}
leaf! {
    /// `Server.Items.LargeBedEastDeed`.
    LARGE_BED_EAST_DEED = "Server.Items.LargeBedEastDeed" : BASE_ADDON_DEED, v0;
    construct: Some(large_bed_east_deed_new),
    family: Some(&crate::addons::TMPL_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `LargeBedSouthAddon()`.
pub fn large_bed_south_addon_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_addon(w, &LARGE_BED_SOUTH_ADDON)
}
leaf! {
    /// `Server.Items.LargeBedSouthAddon`.
    LARGE_BED_SOUTH_ADDON = "Server.Items.LargeBedSouthAddon" : BASE_ADDON, v0;
    construct: Some(large_bed_south_addon_new),
    family: Some(&crate::addons::TMPL_ADDON_CLASS),
}

/// `LargeBedSouthDeed()`.
pub fn large_bed_south_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &LARGE_BED_SOUTH_DEED)
}
leaf! {
    /// `Server.Items.LargeBedSouthDeed`.
    LARGE_BED_SOUTH_DEED = "Server.Items.LargeBedSouthDeed" : BASE_ADDON_DEED, v0;
    construct: Some(large_bed_south_deed_new),
    family: Some(&crate::addons::TMPL_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `LargeForgeEastAddon()`.
pub fn large_forge_east_addon_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_addon(w, &LARGE_FORGE_EAST_ADDON)
}
leaf! {
    /// `Server.Items.LargeForgeEastAddon`.
    LARGE_FORGE_EAST_ADDON = "Server.Items.LargeForgeEastAddon" : BASE_ADDON, v0;
    construct: Some(large_forge_east_addon_new),
    family: Some(&crate::addons::TMPL_ADDON_CLASS),
}

/// `LargeForgeEastDeed()`.
pub fn large_forge_east_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &LARGE_FORGE_EAST_DEED)
}
leaf! {
    /// `Server.Items.LargeForgeEastDeed`.
    LARGE_FORGE_EAST_DEED = "Server.Items.LargeForgeEastDeed" : BASE_ADDON_DEED, v0;
    construct: Some(large_forge_east_deed_new),
    family: Some(&crate::addons::TMPL_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `LargeForgeSouthAddon()`.
pub fn large_forge_south_addon_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_addon(w, &LARGE_FORGE_SOUTH_ADDON)
}
leaf! {
    /// `Server.Items.LargeForgeSouthAddon`.
    LARGE_FORGE_SOUTH_ADDON = "Server.Items.LargeForgeSouthAddon" : BASE_ADDON, v0;
    construct: Some(large_forge_south_addon_new),
    family: Some(&crate::addons::TMPL_ADDON_CLASS),
}

/// `LargeForgeSouthDeed()`.
pub fn large_forge_south_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &LARGE_FORGE_SOUTH_DEED)
}
leaf! {
    /// `Server.Items.LargeForgeSouthDeed`.
    LARGE_FORGE_SOUTH_DEED = "Server.Items.LargeForgeSouthDeed" : BASE_ADDON_DEED, v0;
    construct: Some(large_forge_south_deed_new),
    family: Some(&crate::addons::TMPL_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `LargeStoneTableEastAddon(int hue = 0)`.
pub fn large_stone_table_east_addon_new(w: &mut World, hue: i32) -> ItemId {
    let id = crate::addons::tmpl_addon(w, &LARGE_STONE_TABLE_EAST_ADDON);
    item_hue(w, id, hue);
    id
}
leaf! {
    /// `Server.Items.LargeStoneTableEastAddon`.
    LARGE_STONE_TABLE_EAST_ADDON = "Server.Items.LargeStoneTableEastAddon" : BASE_ADDON, v0;
    construct: Some(|w| large_stone_table_east_addon_new(w, 0)),
    family: Some(&crate::addons::TMPL_ADDON_CLASS),
}

/// `LargeStoneTableEastDeed()`.
pub fn large_stone_table_east_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &LARGE_STONE_TABLE_EAST_DEED)
}
leaf! {
    /// `Server.Items.LargeStoneTableEastDeed`.
    LARGE_STONE_TABLE_EAST_DEED = "Server.Items.LargeStoneTableEastDeed" : BASE_ADDON_DEED, v0;
    construct: Some(large_stone_table_east_deed_new),
    family: Some(&crate::addons::TMPL_HUED_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `LargeStoneTableSouthAddon(int hue = 0)`.
pub fn large_stone_table_south_addon_new(w: &mut World, hue: i32) -> ItemId {
    let id = crate::addons::tmpl_addon(w, &LARGE_STONE_TABLE_SOUTH_ADDON);
    item_hue(w, id, hue);
    id
}
leaf! {
    /// `Server.Items.LargeStoneTableSouthAddon`.
    LARGE_STONE_TABLE_SOUTH_ADDON = "Server.Items.LargeStoneTableSouthAddon" : BASE_ADDON, v0;
    construct: Some(|w| large_stone_table_south_addon_new(w, 0)),
    family: Some(&crate::addons::TMPL_ADDON_CLASS),
}

/// `LargeStoneTableSouthDeed()`.
pub fn large_stone_table_south_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &LARGE_STONE_TABLE_SOUTH_DEED)
}
leaf! {
    /// `Server.Items.LargeStoneTableSouthDeed`.
    LARGE_STONE_TABLE_SOUTH_DEED = "Server.Items.LargeStoneTableSouthDeed" : BASE_ADDON_DEED, v0;
    construct: Some(large_stone_table_south_deed_new),
    family: Some(&crate::addons::TMPL_HUED_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `MediumStoneTableEastAddon(int hue = 0)`.
pub fn medium_stone_table_east_addon_new(w: &mut World, hue: i32) -> ItemId {
    let id = crate::addons::tmpl_addon(w, &MEDIUM_STONE_TABLE_EAST_ADDON);
    item_hue(w, id, hue);
    id
}
leaf! {
    /// `Server.Items.MediumStoneTableEastAddon`.
    MEDIUM_STONE_TABLE_EAST_ADDON = "Server.Items.MediumStoneTableEastAddon" : BASE_ADDON, v0;
    construct: Some(|w| medium_stone_table_east_addon_new(w, 0)),
    family: Some(&crate::addons::TMPL_ADDON_CLASS),
}

/// `MediumStoneTableEastDeed()`.
pub fn medium_stone_table_east_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &MEDIUM_STONE_TABLE_EAST_DEED)
}
leaf! {
    /// `Server.Items.MediumStoneTableEastDeed`.
    MEDIUM_STONE_TABLE_EAST_DEED = "Server.Items.MediumStoneTableEastDeed" : BASE_ADDON_DEED, v0;
    construct: Some(medium_stone_table_east_deed_new),
    family: Some(&crate::addons::TMPL_HUED_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `MediumStoneTableSouthAddon(int hue = 0)`.
pub fn medium_stone_table_south_addon_new(w: &mut World, hue: i32) -> ItemId {
    let id = crate::addons::tmpl_addon(w, &MEDIUM_STONE_TABLE_SOUTH_ADDON);
    item_hue(w, id, hue);
    id
}
leaf! {
    /// `Server.Items.MediumStoneTableSouthAddon`.
    MEDIUM_STONE_TABLE_SOUTH_ADDON = "Server.Items.MediumStoneTableSouthAddon" : BASE_ADDON, v0;
    construct: Some(|w| medium_stone_table_south_addon_new(w, 0)),
    family: Some(&crate::addons::TMPL_ADDON_CLASS),
}

/// `MediumStoneTableSouthDeed()`.
pub fn medium_stone_table_south_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &MEDIUM_STONE_TABLE_SOUTH_DEED)
}
leaf! {
    /// `Server.Items.MediumStoneTableSouthDeed`.
    MEDIUM_STONE_TABLE_SOUTH_DEED = "Server.Items.MediumStoneTableSouthDeed" : BASE_ADDON_DEED, v0;
    construct: Some(medium_stone_table_south_deed_new),
    family: Some(&crate::addons::TMPL_HUED_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `OrnateElvenChestEastAddon()`.
pub fn ornate_elven_chest_east_addon_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_container(w, &ORNATE_ELVEN_CHEST_EAST_ADDON)
}
leaf! {
    /// `Server.Items.OrnateElvenChestEastAddon`.
    ORNATE_ELVEN_CHEST_EAST_ADDON = "Server.Items.OrnateElvenChestEastAddon" : BASE_ADDON_CONTAINER, enc;
    construct: Some(ornate_elven_chest_east_addon_new),
    family: Some(&crate::addons::TMPL_CONTAINER_CLASS),
    default_gump_id: crate::class_props::default_gump_id,
    default_drop_sound: crate::class_props::default_drop_sound,
}

/// `OrnateElvenChestEastDeed()`.
pub fn ornate_elven_chest_east_deed_new(w: &mut World) -> ItemId {
    addon_container::new_deed(w, &ORNATE_ELVEN_CHEST_EAST_DEED)
}
leaf! {
    /// `Server.Items.OrnateElvenChestEastDeed`.
    ORNATE_ELVEN_CHEST_EAST_DEED = "Server.Items.OrnateElvenChestEastDeed" : BASE_ADDON_CONTAINER_DEED, enc;
    construct: Some(ornate_elven_chest_east_deed_new),
    family: Some(&crate::addons::TMPL_CONTAINER_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `OrnateElvenChestSouthAddon()`.
pub fn ornate_elven_chest_south_addon_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_container(w, &ORNATE_ELVEN_CHEST_SOUTH_ADDON)
}
leaf! {
    /// `Server.Items.OrnateElvenChestSouthAddon`.
    ORNATE_ELVEN_CHEST_SOUTH_ADDON = "Server.Items.OrnateElvenChestSouthAddon" : BASE_ADDON_CONTAINER, enc;
    construct: Some(ornate_elven_chest_south_addon_new),
    family: Some(&crate::addons::TMPL_CONTAINER_CLASS),
    default_gump_id: crate::class_props::default_gump_id,
    default_drop_sound: crate::class_props::default_drop_sound,
}

/// `OrnateElvenChestSouthDeed()`.
pub fn ornate_elven_chest_south_deed_new(w: &mut World) -> ItemId {
    addon_container::new_deed(w, &ORNATE_ELVEN_CHEST_SOUTH_DEED)
}
leaf! {
    /// `Server.Items.OrnateElvenChestSouthDeed`.
    ORNATE_ELVEN_CHEST_SOUTH_DEED = "Server.Items.OrnateElvenChestSouthDeed" : BASE_ADDON_CONTAINER_DEED, enc;
    construct: Some(ornate_elven_chest_south_deed_new),
    family: Some(&crate::addons::TMPL_CONTAINER_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `OrnateElvenTableEastAddon()`.
pub fn ornate_elven_table_east_addon_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_addon(w, &ORNATE_ELVEN_TABLE_EAST_ADDON)
}
leaf! {
    /// `Server.Items.OrnateElvenTableEastAddon`.
    ORNATE_ELVEN_TABLE_EAST_ADDON = "Server.Items.OrnateElvenTableEastAddon" : BASE_ADDON, enc;
    construct: Some(ornate_elven_table_east_addon_new),
    family: Some(&crate::addons::TMPL_ADDON_CLASS),
}

/// `OrnateElvenTableEastDeed()`.
pub fn ornate_elven_table_east_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &ORNATE_ELVEN_TABLE_EAST_DEED)
}
leaf! {
    /// `Server.Items.OrnateElvenTableEastDeed`.
    ORNATE_ELVEN_TABLE_EAST_DEED = "Server.Items.OrnateElvenTableEastDeed" : BASE_ADDON_DEED, enc;
    construct: Some(ornate_elven_table_east_deed_new),
    family: Some(&crate::addons::TMPL_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `OrnateElvenTableSouthAddon()`.
pub fn ornate_elven_table_south_addon_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_addon(w, &ORNATE_ELVEN_TABLE_SOUTH_ADDON)
}
leaf! {
    /// `Server.Items.OrnateElvenTableSouthAddon`.
    ORNATE_ELVEN_TABLE_SOUTH_ADDON = "Server.Items.OrnateElvenTableSouthAddon" : BASE_ADDON, enc;
    construct: Some(ornate_elven_table_south_addon_new),
    family: Some(&crate::addons::TMPL_ADDON_CLASS),
}

/// `OrnateElvenTableSouthDeed()`.
pub fn ornate_elven_table_south_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &ORNATE_ELVEN_TABLE_SOUTH_DEED)
}
leaf! {
    /// `Server.Items.OrnateElvenTableSouthDeed`.
    ORNATE_ELVEN_TABLE_SOUTH_DEED = "Server.Items.OrnateElvenTableSouthDeed" : BASE_ADDON_DEED, enc;
    construct: Some(ornate_elven_table_south_deed_new),
    family: Some(&crate::addons::TMPL_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `ParrotPerchAddon()`.
pub fn parrot_perch_addon_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_addon(w, &PARROT_PERCH_ADDON)
}
leaf! {
    /// `Server.Items.ParrotPerchAddon`.
    PARROT_PERCH_ADDON = "Server.Items.ParrotPerchAddon" : BASE_ADDON, enc;
    construct: Some(parrot_perch_addon_new),
    family: Some(&crate::addons::TMPL_ADDON_CLASS),
}

/// `ParrotPerchDeed()`.
pub fn parrot_perch_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &PARROT_PERCH_DEED)
}
leaf! {
    /// `Server.Items.ParrotPerchDeed`.
    PARROT_PERCH_DEED = "Server.Items.ParrotPerchDeed" : BASE_ADDON_DEED, enc;
    construct: Some(parrot_perch_deed_new),
    family: Some(&crate::addons::TMPL_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `PentagramAddon()`.
pub fn pentagram_addon_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_addon(w, &PENTAGRAM_ADDON)
}
leaf! {
    /// `Server.Items.PentagramAddon`.
    PENTAGRAM_ADDON = "Server.Items.PentagramAddon" : BASE_ADDON, v0;
    construct: Some(pentagram_addon_new),
    family: Some(&crate::addons::TMPL_ADDON_CLASS),
}

/// `PentagramDeed()`.
pub fn pentagram_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &PENTAGRAM_DEED)
}
leaf! {
    /// `Server.Items.PentagramDeed`.
    PENTAGRAM_DEED = "Server.Items.PentagramDeed" : BASE_ADDON_DEED, v0;
    construct: Some(pentagram_deed_new),
    family: Some(&crate::addons::TMPL_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `PyramidAddon()`.
pub fn pyramid_addon_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_addon(w, &PYRAMID_ADDON)
}
leaf! {
    /// `Server.Items.PyramidAddon`.
    PYRAMID_ADDON = "Server.Items.PyramidAddon" : BASE_ADDON, enc;
    construct: Some(pyramid_addon_new),
}

/// `SandstoneFireplaceEastAddon()`.
pub fn sandstone_fireplace_east_addon_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_addon(w, &SANDSTONE_FIREPLACE_EAST_ADDON)
}
leaf! {
    /// `Server.Items.SandstoneFireplaceEastAddon`.
    SANDSTONE_FIREPLACE_EAST_ADDON = "Server.Items.SandstoneFireplaceEastAddon" : BASE_ADDON, v0;
    construct: Some(sandstone_fireplace_east_addon_new),
    family: Some(&crate::addons::TMPL_ADDON_CLASS),
}

/// `SandstoneFireplaceEastDeed()`.
pub fn sandstone_fireplace_east_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &SANDSTONE_FIREPLACE_EAST_DEED)
}
leaf! {
    /// `Server.Items.SandstoneFireplaceEastDeed`.
    SANDSTONE_FIREPLACE_EAST_DEED = "Server.Items.SandstoneFireplaceEastDeed" : BASE_ADDON_DEED, v0;
    construct: Some(sandstone_fireplace_east_deed_new),
    family: Some(&crate::addons::TMPL_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `SandstoneFireplaceSouthAddon()`.
pub fn sandstone_fireplace_south_addon_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_addon(w, &SANDSTONE_FIREPLACE_SOUTH_ADDON)
}
leaf! {
    /// `Server.Items.SandstoneFireplaceSouthAddon`.
    SANDSTONE_FIREPLACE_SOUTH_ADDON = "Server.Items.SandstoneFireplaceSouthAddon" : BASE_ADDON, v0;
    construct: Some(sandstone_fireplace_south_addon_new),
    family: Some(&crate::addons::TMPL_ADDON_CLASS),
}

/// `SandstoneFireplaceSouthDeed()`.
pub fn sandstone_fireplace_south_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &SANDSTONE_FIREPLACE_SOUTH_DEED)
}
leaf! {
    /// `Server.Items.SandstoneFireplaceSouthDeed`.
    SANDSTONE_FIREPLACE_SOUTH_DEED = "Server.Items.SandstoneFireplaceSouthDeed" : BASE_ADDON_DEED, v0;
    construct: Some(sandstone_fireplace_south_deed_new),
    family: Some(&crate::addons::TMPL_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `SandstoneFountainAddon()`.
pub fn sandstone_fountain_addon_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_addon(w, &SANDSTONE_FOUNTAIN_ADDON)
}
leaf! {
    /// `Server.Items.SandstoneFountainAddon`.
    SANDSTONE_FOUNTAIN_ADDON = "Server.Items.SandstoneFountainAddon" : BASE_ADDON, v0;
    construct: Some(sandstone_fountain_addon_new),
}

/// `SerpentPillarAddon()`.
pub fn serpent_pillar_addon_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_addon(w, &SERPENT_PILLAR_ADDON)
}
leaf! {
    /// `Server.Items.SerpentPillarAddon`.
    SERPENT_PILLAR_ADDON = "Server.Items.SerpentPillarAddon" : BASE_ADDON, v0;
    construct: Some(serpent_pillar_addon_new),
}

/// `ShrineOfWisdomAddon()`.
pub fn shrine_of_wisdom_addon_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_addon(w, &SHRINE_OF_WISDOM_ADDON)
}
leaf! {
    /// `Server.Items.ShrineOfWisdomAddon`.
    SHRINE_OF_WISDOM_ADDON = "Server.Items.ShrineOfWisdomAddon" : BASE_ADDON, v0;
    construct: Some(shrine_of_wisdom_addon_new),
}

/// `ShrineOfWisdomComponent(itemID)`.
pub fn shrine_of_wisdom_component_new(w: &mut World, item_id: i32) -> ItemId {
    addon::new_component(w, &SHRINE_OF_WISDOM_COMPONENT, item_id)
}
leaf! {
    /// `Server.Items.ShrineOfWisdomComponent`.
    SHRINE_OF_WISDOM_COMPONENT = "Server.Items.ShrineOfWisdomComponent" : ADDON_COMPONENT, v0;
    label_number: crate::class_props::label_number,
}

/// `SkullPileAddon()`.
pub fn skull_pile_addon_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_addon(w, &SKULL_PILE_ADDON)
}
leaf! {
    /// `Server.Items.SkullPileAddon`.
    SKULL_PILE_ADDON = "Server.Items.SkullPileAddon" : BASE_ADDON, enc;
    construct: Some(skull_pile_addon_new),
}

/// `SmallBedEastAddon()`.
pub fn small_bed_east_addon_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_addon(w, &SMALL_BED_EAST_ADDON)
}
leaf! {
    /// `Server.Items.SmallBedEastAddon`.
    SMALL_BED_EAST_ADDON = "Server.Items.SmallBedEastAddon" : BASE_ADDON, v0;
    construct: Some(small_bed_east_addon_new),
    family: Some(&crate::addons::TMPL_ADDON_CLASS),
}

/// `SmallBedEastDeed()`.
pub fn small_bed_east_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &SMALL_BED_EAST_DEED)
}
leaf! {
    /// `Server.Items.SmallBedEastDeed`.
    SMALL_BED_EAST_DEED = "Server.Items.SmallBedEastDeed" : BASE_ADDON_DEED, v0;
    construct: Some(small_bed_east_deed_new),
    family: Some(&crate::addons::TMPL_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `SmallBedSouthAddon()`.
pub fn small_bed_south_addon_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_addon(w, &SMALL_BED_SOUTH_ADDON)
}
leaf! {
    /// `Server.Items.SmallBedSouthAddon`.
    SMALL_BED_SOUTH_ADDON = "Server.Items.SmallBedSouthAddon" : BASE_ADDON, v0;
    construct: Some(small_bed_south_addon_new),
    family: Some(&crate::addons::TMPL_ADDON_CLASS),
}

/// `SmallBedSouthDeed()`.
pub fn small_bed_south_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &SMALL_BED_SOUTH_DEED)
}
leaf! {
    /// `Server.Items.SmallBedSouthDeed`.
    SMALL_BED_SOUTH_DEED = "Server.Items.SmallBedSouthDeed" : BASE_ADDON_DEED, v0;
    construct: Some(small_bed_south_deed_new),
    family: Some(&crate::addons::TMPL_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `SmallForgeAddon()`.
pub fn small_forge_addon_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_addon(w, &SMALL_FORGE_ADDON)
}
leaf! {
    /// `Server.Items.SmallForgeAddon`.
    SMALL_FORGE_ADDON = "Server.Items.SmallForgeAddon" : BASE_ADDON, v0;
    construct: Some(small_forge_addon_new),
    family: Some(&crate::addons::TMPL_ADDON_CLASS),
}

/// `SmallForgeDeed()`.
pub fn small_forge_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &SMALL_FORGE_DEED)
}
leaf! {
    /// `Server.Items.SmallForgeDeed`.
    SMALL_FORGE_DEED = "Server.Items.SmallForgeDeed" : BASE_ADDON_DEED, v0;
    construct: Some(small_forge_deed_new),
    family: Some(&crate::addons::TMPL_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `SquirrelStatueEastAddon()`.
pub fn squirrel_statue_east_addon_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_addon(w, &SQUIRREL_STATUE_EAST_ADDON)
}
leaf! {
    /// `Server.Items.SquirrelStatueEastAddon`.
    SQUIRREL_STATUE_EAST_ADDON = "Server.Items.SquirrelStatueEastAddon" : BASE_ADDON, enc;
    construct: Some(squirrel_statue_east_addon_new),
    family: Some(&crate::addons::TMPL_ADDON_CLASS),
}

/// `SquirrelStatueEastDeed()`.
pub fn squirrel_statue_east_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &SQUIRREL_STATUE_EAST_DEED)
}
leaf! {
    /// `Server.Items.SquirrelStatueEastDeed`.
    SQUIRREL_STATUE_EAST_DEED = "Server.Items.SquirrelStatueEastDeed" : BASE_ADDON_DEED, enc;
    construct: Some(squirrel_statue_east_deed_new),
    family: Some(&crate::addons::TMPL_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `SquirrelStatueSouthAddon()`.
pub fn squirrel_statue_south_addon_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_addon(w, &SQUIRREL_STATUE_SOUTH_ADDON)
}
leaf! {
    /// `Server.Items.SquirrelStatueSouthAddon`.
    SQUIRREL_STATUE_SOUTH_ADDON = "Server.Items.SquirrelStatueSouthAddon" : BASE_ADDON, enc;
    construct: Some(squirrel_statue_south_addon_new),
    family: Some(&crate::addons::TMPL_ADDON_CLASS),
}

/// `SquirrelStatueSouthDeed()`.
pub fn squirrel_statue_south_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &SQUIRREL_STATUE_SOUTH_DEED)
}
leaf! {
    /// `Server.Items.SquirrelStatueSouthDeed`.
    SQUIRREL_STATUE_SOUTH_DEED = "Server.Items.SquirrelStatueSouthDeed" : BASE_ADDON_DEED, enc;
    construct: Some(squirrel_statue_south_deed_new),
    family: Some(&crate::addons::TMPL_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `StoneFireplaceEastAddon()`.
pub fn stone_fireplace_east_addon_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_addon(w, &STONE_FIREPLACE_EAST_ADDON)
}
leaf! {
    /// `Server.Items.StoneFireplaceEastAddon`.
    STONE_FIREPLACE_EAST_ADDON = "Server.Items.StoneFireplaceEastAddon" : BASE_ADDON, v0;
    construct: Some(stone_fireplace_east_addon_new),
    family: Some(&crate::addons::TMPL_ADDON_CLASS),
}

/// `StoneFireplaceEastDeed()`.
pub fn stone_fireplace_east_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &STONE_FIREPLACE_EAST_DEED)
}
leaf! {
    /// `Server.Items.StoneFireplaceEastDeed`.
    STONE_FIREPLACE_EAST_DEED = "Server.Items.StoneFireplaceEastDeed" : BASE_ADDON_DEED, v0;
    construct: Some(stone_fireplace_east_deed_new),
    family: Some(&crate::addons::TMPL_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `StoneFireplaceSouthAddon()`.
pub fn stone_fireplace_south_addon_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_addon(w, &STONE_FIREPLACE_SOUTH_ADDON)
}
leaf! {
    /// `Server.Items.StoneFireplaceSouthAddon`.
    STONE_FIREPLACE_SOUTH_ADDON = "Server.Items.StoneFireplaceSouthAddon" : BASE_ADDON, v0;
    construct: Some(stone_fireplace_south_addon_new),
    family: Some(&crate::addons::TMPL_ADDON_CLASS),
}

/// `StoneFireplaceSouthDeed()`.
pub fn stone_fireplace_south_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &STONE_FIREPLACE_SOUTH_DEED)
}
leaf! {
    /// `Server.Items.StoneFireplaceSouthDeed`.
    STONE_FIREPLACE_SOUTH_DEED = "Server.Items.StoneFireplaceSouthDeed" : BASE_ADDON_DEED, v0;
    construct: Some(stone_fireplace_south_deed_new),
    family: Some(&crate::addons::TMPL_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `StoneFountainAddon()`.
pub fn stone_fountain_addon_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_addon(w, &STONE_FOUNTAIN_ADDON)
}
leaf! {
    /// `Server.Items.StoneFountainAddon`.
    STONE_FOUNTAIN_ADDON = "Server.Items.StoneFountainAddon" : BASE_ADDON, v0;
    construct: Some(stone_fountain_addon_new),
}

/// `StoneOvenEastAddon()`.
pub fn stone_oven_east_addon_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_addon(w, &STONE_OVEN_EAST_ADDON)
}
leaf! {
    /// `Server.Items.StoneOvenEastAddon`.
    STONE_OVEN_EAST_ADDON = "Server.Items.StoneOvenEastAddon" : BASE_ADDON, v0;
    construct: Some(stone_oven_east_addon_new),
    family: Some(&crate::addons::TMPL_ADDON_CLASS),
}

/// `StoneOvenEastDeed()`.
pub fn stone_oven_east_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &STONE_OVEN_EAST_DEED)
}
leaf! {
    /// `Server.Items.StoneOvenEastDeed`.
    STONE_OVEN_EAST_DEED = "Server.Items.StoneOvenEastDeed" : BASE_ADDON_DEED, v0;
    construct: Some(stone_oven_east_deed_new),
    family: Some(&crate::addons::TMPL_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `StoneOvenSouthAddon()`.
pub fn stone_oven_south_addon_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_addon(w, &STONE_OVEN_SOUTH_ADDON)
}
leaf! {
    /// `Server.Items.StoneOvenSouthAddon`.
    STONE_OVEN_SOUTH_ADDON = "Server.Items.StoneOvenSouthAddon" : BASE_ADDON, v0;
    construct: Some(stone_oven_south_addon_new),
    family: Some(&crate::addons::TMPL_ADDON_CLASS),
}

/// `StoneOvenSouthDeed()`.
pub fn stone_oven_south_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &STONE_OVEN_SOUTH_DEED)
}
leaf! {
    /// `Server.Items.StoneOvenSouthDeed`.
    STONE_OVEN_SOUTH_DEED = "Server.Items.StoneOvenSouthDeed" : BASE_ADDON_DEED, v0;
    construct: Some(stone_oven_south_deed_new),
    family: Some(&crate::addons::TMPL_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `SmallStretchedHideEastAddon()`.
pub fn small_stretched_hide_east_addon_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_addon(w, &SMALL_STRETCHED_HIDE_EAST_ADDON)
}
leaf! {
    /// `Server.Items.SmallStretchedHideEastAddon`.
    SMALL_STRETCHED_HIDE_EAST_ADDON = "Server.Items.SmallStretchedHideEastAddon" : BASE_ADDON, v0;
    construct: Some(small_stretched_hide_east_addon_new),
    family: Some(&crate::addons::TMPL_ADDON_CLASS),
}

/// `SmallStretchedHideEastDeed()`.
pub fn small_stretched_hide_east_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &SMALL_STRETCHED_HIDE_EAST_DEED)
}
leaf! {
    /// `Server.Items.SmallStretchedHideEastDeed`.
    SMALL_STRETCHED_HIDE_EAST_DEED = "Server.Items.SmallStretchedHideEastDeed" : BASE_ADDON_DEED, v0;
    construct: Some(small_stretched_hide_east_deed_new),
    family: Some(&crate::addons::TMPL_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `SmallStretchedHideSouthAddon()`.
pub fn small_stretched_hide_south_addon_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_addon(w, &SMALL_STRETCHED_HIDE_SOUTH_ADDON)
}
leaf! {
    /// `Server.Items.SmallStretchedHideSouthAddon`.
    SMALL_STRETCHED_HIDE_SOUTH_ADDON = "Server.Items.SmallStretchedHideSouthAddon" : BASE_ADDON, v0;
    construct: Some(small_stretched_hide_south_addon_new),
    family: Some(&crate::addons::TMPL_ADDON_CLASS),
}

/// `SmallStretchedHideSouthDeed()`.
pub fn small_stretched_hide_south_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &SMALL_STRETCHED_HIDE_SOUTH_DEED)
}
leaf! {
    /// `Server.Items.SmallStretchedHideSouthDeed`.
    SMALL_STRETCHED_HIDE_SOUTH_DEED = "Server.Items.SmallStretchedHideSouthDeed" : BASE_ADDON_DEED, v0;
    construct: Some(small_stretched_hide_south_deed_new),
    family: Some(&crate::addons::TMPL_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `MediumStretchedHideEastAddon()`.
pub fn medium_stretched_hide_east_addon_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_addon(w, &MEDIUM_STRETCHED_HIDE_EAST_ADDON)
}
leaf! {
    /// `Server.Items.MediumStretchedHideEastAddon`.
    MEDIUM_STRETCHED_HIDE_EAST_ADDON = "Server.Items.MediumStretchedHideEastAddon" : BASE_ADDON, v0;
    construct: Some(medium_stretched_hide_east_addon_new),
    family: Some(&crate::addons::TMPL_ADDON_CLASS),
}

/// `MediumStretchedHideEastDeed()`.
pub fn medium_stretched_hide_east_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &MEDIUM_STRETCHED_HIDE_EAST_DEED)
}
leaf! {
    /// `Server.Items.MediumStretchedHideEastDeed`.
    MEDIUM_STRETCHED_HIDE_EAST_DEED = "Server.Items.MediumStretchedHideEastDeed" : BASE_ADDON_DEED, v0;
    construct: Some(medium_stretched_hide_east_deed_new),
    family: Some(&crate::addons::TMPL_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `MediumStretchedHideSouthAddon()`.
pub fn medium_stretched_hide_south_addon_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_addon(w, &MEDIUM_STRETCHED_HIDE_SOUTH_ADDON)
}
leaf! {
    /// `Server.Items.MediumStretchedHideSouthAddon`.
    MEDIUM_STRETCHED_HIDE_SOUTH_ADDON = "Server.Items.MediumStretchedHideSouthAddon" : BASE_ADDON, v0;
    construct: Some(medium_stretched_hide_south_addon_new),
    family: Some(&crate::addons::TMPL_ADDON_CLASS),
}

/// `MediumStretchedHideSouthDeed()`.
pub fn medium_stretched_hide_south_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &MEDIUM_STRETCHED_HIDE_SOUTH_DEED)
}
leaf! {
    /// `Server.Items.MediumStretchedHideSouthDeed`.
    MEDIUM_STRETCHED_HIDE_SOUTH_DEED = "Server.Items.MediumStretchedHideSouthDeed" : BASE_ADDON_DEED, v0;
    construct: Some(medium_stretched_hide_south_deed_new),
    family: Some(&crate::addons::TMPL_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `TallElvenBedEastAddon()`.
pub fn tall_elven_bed_east_addon_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_addon(w, &TALL_ELVEN_BED_EAST_ADDON)
}
leaf! {
    /// `Server.Items.TallElvenBedEastAddon`.
    TALL_ELVEN_BED_EAST_ADDON = "Server.Items.TallElvenBedEastAddon" : BASE_ADDON, enc;
    construct: Some(tall_elven_bed_east_addon_new),
    family: Some(&crate::addons::TMPL_ADDON_CLASS),
}

/// `TallElvenBedEastDeed()`.
pub fn tall_elven_bed_east_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &TALL_ELVEN_BED_EAST_DEED)
}
leaf! {
    /// `Server.Items.TallElvenBedEastDeed`.
    TALL_ELVEN_BED_EAST_DEED = "Server.Items.TallElvenBedEastDeed" : BASE_ADDON_DEED, enc;
    construct: Some(tall_elven_bed_east_deed_new),
    family: Some(&crate::addons::TMPL_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `TallElvenBedSouthAddon()`.
pub fn tall_elven_bed_south_addon_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_addon(w, &TALL_ELVEN_BED_SOUTH_ADDON)
}
leaf! {
    /// `Server.Items.TallElvenBedSouthAddon`.
    TALL_ELVEN_BED_SOUTH_ADDON = "Server.Items.TallElvenBedSouthAddon" : BASE_ADDON, enc;
    construct: Some(tall_elven_bed_south_addon_new),
    family: Some(&crate::addons::TMPL_ADDON_CLASS),
}

/// `TallElvenBedSouthDeed()`.
pub fn tall_elven_bed_south_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &TALL_ELVEN_BED_SOUTH_DEED)
}
leaf! {
    /// `Server.Items.TallElvenBedSouthDeed`.
    TALL_ELVEN_BED_SOUTH_DEED = "Server.Items.TallElvenBedSouthDeed" : BASE_ADDON_DEED, enc;
    construct: Some(tall_elven_bed_south_deed_new),
    family: Some(&crate::addons::TMPL_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `Telescope()`.
pub fn telescope_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_addon(w, &TELESCOPE)
}
leaf! {
    /// `Server.Items.Telescope`.
    TELESCOPE = "Server.Items.Telescope" : BASE_ADDON, v0;
    construct: Some(telescope_new),
}

/// `WarriorStatueEastAddon()`.
pub fn warrior_statue_east_addon_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_addon(w, &WARRIOR_STATUE_EAST_ADDON)
}
leaf! {
    /// `Server.Items.WarriorStatueEastAddon`.
    WARRIOR_STATUE_EAST_ADDON = "Server.Items.WarriorStatueEastAddon" : BASE_ADDON, enc;
    construct: Some(warrior_statue_east_addon_new),
    family: Some(&crate::addons::TMPL_ADDON_CLASS),
}

/// `WarriorStatueEastDeed()`.
pub fn warrior_statue_east_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &WARRIOR_STATUE_EAST_DEED)
}
leaf! {
    /// `Server.Items.WarriorStatueEastDeed`.
    WARRIOR_STATUE_EAST_DEED = "Server.Items.WarriorStatueEastDeed" : BASE_ADDON_DEED, enc;
    construct: Some(warrior_statue_east_deed_new),
    family: Some(&crate::addons::TMPL_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `WarriorStatueSouthAddon()`.
pub fn warrior_statue_south_addon_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_addon(w, &WARRIOR_STATUE_SOUTH_ADDON)
}
leaf! {
    /// `Server.Items.WarriorStatueSouthAddon`.
    WARRIOR_STATUE_SOUTH_ADDON = "Server.Items.WarriorStatueSouthAddon" : BASE_ADDON, enc;
    construct: Some(warrior_statue_south_addon_new),
    family: Some(&crate::addons::TMPL_ADDON_CLASS),
}

/// `WarriorStatueSouthDeed()`.
pub fn warrior_statue_south_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &WARRIOR_STATUE_SOUTH_DEED)
}
leaf! {
    /// `Server.Items.WarriorStatueSouthDeed`.
    WARRIOR_STATUE_SOUTH_DEED = "Server.Items.WarriorStatueSouthDeed" : BASE_ADDON_DEED, enc;
    construct: Some(warrior_statue_south_deed_new),
    family: Some(&crate::addons::TMPL_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `WaterVatEast()`.
pub fn water_vat_east_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_addon(w, &WATER_VAT_EAST)
}
leaf! {
    /// `Server.Items.WaterVatEast`.
    WATER_VAT_EAST = "Server.Items.WaterVatEast" : BASE_ADDON, enc;
    construct: Some(water_vat_east_new),
}

/// `WaterVatSouth()`.
pub fn water_vat_south_new(w: &mut World) -> ItemId {
    crate::addons::tmpl_addon(w, &WATER_VAT_SOUTH)
}
leaf! {
    /// `Server.Items.WaterVatSouth`.
    WATER_VAT_SOUTH = "Server.Items.WaterVatSouth" : BASE_ADDON, enc;
    construct: Some(water_vat_south_new),
}

/// Every class of this file, with its `[TypeAlias]`es.
pub static ALL: &[(&ItemType, &[&str])] = &[
    (&ABBATOIR_ADDON, &[]),
    (&ABBATOIR_DEED, &[]),
    (&ALCHEMIST_TABLE_EAST_ADDON, &[]),
    (&ALCHEMIST_TABLE_EAST_DEED, &[]),
    (&ALCHEMIST_TABLE_SOUTH_ADDON, &[]),
    (&ALCHEMIST_TABLE_SOUTH_DEED, &[]),
    (&ANVIL_EAST_ADDON, &[]),
    (&ANVIL_EAST_DEED, &[]),
    (&ANVIL_SOUTH_ADDON, &[]),
    (&ANVIL_SOUTH_DEED, &[]),
    (&ARCANE_BOOKSHELF_EAST_ADDON, &[]),
    (&ARCANE_BOOKSHELF_EAST_DEED, &[]),
    (&ARCANE_BOOKSHELF_SOUTH_ADDON, &[]),
    (&ARCANE_BOOKSHELF_SOUTH_DEED, &[]),
    (&ARCANE_CIRCLE_ADDON, &[]),
    (&ARCANE_CIRCLE_DEED, &[]),
    (&ARCANIST_STATUE_EAST_ADDON, &[]),
    (&ARCANIST_STATUE_EAST_DEED, &[]),
    (&ARCANIST_STATUE_SOUTH_ADDON, &[]),
    (&ARCANIST_STATUE_SOUTH_DEED, &[]),
    (&BROWN_BEAR_RUG_EAST_ADDON, &[]),
    (&BROWN_BEAR_RUG_EAST_DEED, &[]),
    (&BROWN_BEAR_RUG_SOUTH_ADDON, &[]),
    (&BROWN_BEAR_RUG_SOUTH_DEED, &[]),
    (&POLAR_BEAR_RUG_EAST_ADDON, &[]),
    (&POLAR_BEAR_RUG_EAST_DEED, &[]),
    (&POLAR_BEAR_RUG_SOUTH_ADDON, &[]),
    (&POLAR_BEAR_RUG_SOUTH_DEED, &[]),
    (&BLOOD_PENTAGRAM, &[]),
    (&ELVEN_BED_EAST_ADDON, &[]),
    (&ELVEN_BED_EAST_DEED, &[]),
    (&ELVEN_BED_SOUTH_ADDON, &[]),
    (&ELVEN_BED_SOUTH_DEED, &[]),
    (&ELVEN_DRESSER_EAST_ADDON, &[]),
    (&ELVEN_DRESSER_EAST_DEED, &[]),
    (&ELVEN_DRESSER_SOUTH_ADDON, &[]),
    (&ELVEN_DRESSER_SOUTH_DEED, &[]),
    (&ELVEN_FORGE_ADDON, &[]),
    (&ELVEN_FORGE_DEED, &[]),
    (&ELVEN_LOVESEAT_EAST_ADDON, &[]),
    (&ELVEN_LOVESEAT_EAST_DEED, &[]),
    (&ELVEN_LOVESEAT_SOUTH_ADDON, &[]),
    (&ELVEN_LOVESEAT_SOUTH_DEED, &[]),
    (&ELVEN_STOVE_EAST_ADDON, &[]),
    (&ELVEN_STOVE_EAST_DEED, &[]),
    (&ELVEN_STOVE_SOUTH_ADDON, &[]),
    (&ELVEN_STOVE_SOUTH_DEED, &[]),
    (&ELVEN_WASH_BASIN_EAST_ADDON, &[]),
    (&ELVEN_WASH_BASIN_EAST_DEED, &[]),
    (&ELVEN_WASH_BASIN_SOUTH_ADDON, &[]),
    (&ELVEN_WASH_BASIN_SOUTH_DEED, &[]),
    (&FANCY_ELVEN_TABLE_EAST_ADDON, &[]),
    (&FANCY_ELVEN_TABLE_EAST_DEED, &[]),
    (&FANCY_ELVEN_TABLE_SOUTH_ADDON, &[]),
    (&FANCY_ELVEN_TABLE_SOUTH_DEED, &[]),
    (&FIRE_COLUMN_ADDON, &[]),
    (&LIGHT_FLOWER_TAPESTRY_EAST_ADDON, &[]),
    (&LIGHT_FLOWER_TAPESTRY_EAST_DEED, &[]),
    (&LIGHT_FLOWER_TAPESTRY_SOUTH_ADDON, &[]),
    (&LIGHT_FLOWER_TAPESTRY_SOUTH_DEED, &[]),
    (&DARK_FLOWER_TAPESTRY_EAST_ADDON, &[]),
    (&DARK_FLOWER_TAPESTRY_EAST_DEED, &[]),
    (&DARK_FLOWER_TAPESTRY_SOUTH_ADDON, &[]),
    (&DARK_FLOWER_TAPESTRY_SOUTH_DEED, &[]),
    (&GIANT_WEB1, &[]),
    (&GIANT_WEB2, &[]),
    (&GIANT_WEB3, &[]),
    (&GIANT_WEB4, &[]),
    (&GIANT_WEB5, &[]),
    (&GIANT_WEB6, &[]),
    (&GOZA_MAT_EAST_ADDON, &[]),
    (&GOZA_MAT_EAST_DEED, &[]),
    (&GOZA_MAT_SOUTH_ADDON, &[]),
    (&GOZA_MAT_SOUTH_DEED, &[]),
    (&SQUARE_GOZA_MAT_EAST_ADDON, &[]),
    (&SQUARE_GOZA_MAT_EAST_DEED, &[]),
    (&SQUARE_GOZA_MAT_SOUTH_ADDON, &[]),
    (&SQUARE_GOZA_MAT_SOUTH_DEED, &[]),
    (&BROCADE_GOZA_MAT_EAST_ADDON, &[]),
    (&BROCADE_GOZA_MAT_EAST_DEED, &[]),
    (&BROCADE_GOZA_MAT_SOUTH_ADDON, &[]),
    (&BROCADE_GOZA_MAT_SOUTH_DEED, &[]),
    (&BROCADE_SQUARE_GOZA_MAT_EAST_ADDON, &[]),
    (&BROCADE_SQUARE_GOZA_MAT_EAST_DEED, &[]),
    (&BROCADE_SQUARE_GOZA_MAT_SOUTH_ADDON, &[]),
    (&BROCADE_SQUARE_GOZA_MAT_SOUTH_DEED, &[]),
    (&GRAY_BRICK_FIREPLACE_EAST_ADDON, &[]),
    (&GRAY_BRICK_FIREPLACE_EAST_DEED, &[]),
    (&GRAY_BRICK_FIREPLACE_SOUTH_ADDON, &[]),
    (&GRAY_BRICK_FIREPLACE_SOUTH_DEED, &[]),
    (&LARGE_BED_EAST_ADDON, &[]),
    (&LARGE_BED_EAST_DEED, &[]),
    (&LARGE_BED_SOUTH_ADDON, &[]),
    (&LARGE_BED_SOUTH_DEED, &[]),
    (&LARGE_FORGE_EAST_ADDON, &[]),
    (&LARGE_FORGE_EAST_DEED, &[]),
    (&LARGE_FORGE_SOUTH_ADDON, &[]),
    (&LARGE_FORGE_SOUTH_DEED, &[]),
    (&LARGE_STONE_TABLE_EAST_ADDON, &[]),
    (&LARGE_STONE_TABLE_EAST_DEED, &[]),
    (&LARGE_STONE_TABLE_SOUTH_ADDON, &[]),
    (&LARGE_STONE_TABLE_SOUTH_DEED, &[]),
    (&MEDIUM_STONE_TABLE_EAST_ADDON, &[]),
    (&MEDIUM_STONE_TABLE_EAST_DEED, &[]),
    (&MEDIUM_STONE_TABLE_SOUTH_ADDON, &[]),
    (&MEDIUM_STONE_TABLE_SOUTH_DEED, &[]),
    (&ORNATE_ELVEN_CHEST_EAST_ADDON, &[]),
    (&ORNATE_ELVEN_CHEST_EAST_DEED, &[]),
    (&ORNATE_ELVEN_CHEST_SOUTH_ADDON, &[]),
    (&ORNATE_ELVEN_CHEST_SOUTH_DEED, &[]),
    (&ORNATE_ELVEN_TABLE_EAST_ADDON, &[]),
    (&ORNATE_ELVEN_TABLE_EAST_DEED, &[]),
    (&ORNATE_ELVEN_TABLE_SOUTH_ADDON, &[]),
    (&ORNATE_ELVEN_TABLE_SOUTH_DEED, &[]),
    (&PARROT_PERCH_ADDON, &[]),
    (&PARROT_PERCH_DEED, &[]),
    (&PENTAGRAM_ADDON, &[]),
    (&PENTAGRAM_DEED, &[]),
    (&PYRAMID_ADDON, &[]),
    (&SANDSTONE_FIREPLACE_EAST_ADDON, &[]),
    (&SANDSTONE_FIREPLACE_EAST_DEED, &[]),
    (&SANDSTONE_FIREPLACE_SOUTH_ADDON, &[]),
    (&SANDSTONE_FIREPLACE_SOUTH_DEED, &[]),
    (&SANDSTONE_FOUNTAIN_ADDON, &[]),
    (&SERPENT_PILLAR_ADDON, &[]),
    (&SHRINE_OF_WISDOM_ADDON, &[]),
    (&SHRINE_OF_WISDOM_COMPONENT, &[]),
    (&SKULL_PILE_ADDON, &[]),
    (&SMALL_BED_EAST_ADDON, &[]),
    (&SMALL_BED_EAST_DEED, &[]),
    (&SMALL_BED_SOUTH_ADDON, &[]),
    (&SMALL_BED_SOUTH_DEED, &[]),
    (&SMALL_FORGE_ADDON, &[]),
    (&SMALL_FORGE_DEED, &[]),
    (&SQUIRREL_STATUE_EAST_ADDON, &[]),
    (&SQUIRREL_STATUE_EAST_DEED, &[]),
    (&SQUIRREL_STATUE_SOUTH_ADDON, &[]),
    (&SQUIRREL_STATUE_SOUTH_DEED, &[]),
    (&STONE_FIREPLACE_EAST_ADDON, &[]),
    (&STONE_FIREPLACE_EAST_DEED, &[]),
    (&STONE_FIREPLACE_SOUTH_ADDON, &[]),
    (&STONE_FIREPLACE_SOUTH_DEED, &[]),
    (&STONE_FOUNTAIN_ADDON, &[]),
    (&STONE_OVEN_EAST_ADDON, &[]),
    (&STONE_OVEN_EAST_DEED, &[]),
    (&STONE_OVEN_SOUTH_ADDON, &[]),
    (&STONE_OVEN_SOUTH_DEED, &[]),
    (&SMALL_STRETCHED_HIDE_EAST_ADDON, &[]),
    (&SMALL_STRETCHED_HIDE_EAST_DEED, &[]),
    (&SMALL_STRETCHED_HIDE_SOUTH_ADDON, &[]),
    (&SMALL_STRETCHED_HIDE_SOUTH_DEED, &[]),
    (&MEDIUM_STRETCHED_HIDE_EAST_ADDON, &[]),
    (&MEDIUM_STRETCHED_HIDE_EAST_DEED, &[]),
    (&MEDIUM_STRETCHED_HIDE_SOUTH_ADDON, &[]),
    (&MEDIUM_STRETCHED_HIDE_SOUTH_DEED, &[]),
    (&TALL_ELVEN_BED_EAST_ADDON, &[]),
    (&TALL_ELVEN_BED_EAST_DEED, &[]),
    (&TALL_ELVEN_BED_SOUTH_ADDON, &[]),
    (&TALL_ELVEN_BED_SOUTH_DEED, &[]),
    (&TELESCOPE, &[]),
    (&WARRIOR_STATUE_EAST_ADDON, &[]),
    (&WARRIOR_STATUE_EAST_DEED, &[]),
    (&WARRIOR_STATUE_SOUTH_ADDON, &[]),
    (&WARRIOR_STATUE_SOUTH_DEED, &[]),
    (&WATER_VAT_EAST, &[]),
    (&WATER_VAT_SOUTH, &[]),
];

pub fn register(types: &mut TypeRegistry) {
    for (t, aliases) in ALL {
        types.register_item(t, aliases);
    }
}
