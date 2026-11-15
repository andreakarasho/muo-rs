//! The addons, components and deeds that are plain tables (constants and `AddComponent` rows). Those with logic are
//! hand-written next to this file.

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
use muo_server::item::ItemType;
use muo_server::type_registry::TypeRegistry;

use crate::simple::leaf;

#[allow(dead_code)]
fn item_hue(w: &mut World, id: ItemId, value: i32) {
    crate::simple::hue(w, id, value);
}

/// `AbbatoirAddon()`.
pub fn abbatoir_addon_new(w: &mut World) -> ItemId {
    let id = addon::new_addon(w, &ABBATOIR_ADDON);
    abbatoir_components(w, id);
    id
}
/// `AbbatoirAddon()`'s `AddComponent` calls, for a derived class too (`SummoningAltar`).
pub fn abbatoir_components(w: &mut World, id: ItemId) {
    addon::add_components(
        w,
        id,
        &[
            (Comp::Plain(4622), -1, -1, 0),
            (Comp::Plain(4623), 0, -1, 0),
            (Comp::Plain(4624), 1, -1, 0),
            (Comp::Plain(4629), -1, 0, 0),
            (Comp::Plain(4630), 0, 0, 0),
            (Comp::Plain(4625), 1, 0, 0),
            (Comp::Plain(4628), -1, 1, 0),
            (Comp::Plain(4627), 0, 1, 0),
            (Comp::Plain(4626), 1, 1, 0),
        ],
    );
}
static ABBATOIR_ADDON_CLASS: AddonClass = AddonClass {
    deed: Some(abbatoir_deed_new),
    ..BASE_ADDON_CLASS
};
leaf! {
    /// `Server.Items.AbbatoirAddon`.
    ABBATOIR_ADDON = "Server.Items.AbbatoirAddon" : BASE_ADDON, v0;
    construct: Some(abbatoir_addon_new),
    family: Some(&ABBATOIR_ADDON_CLASS),
}

/// `AbbatoirDeed()`.
pub fn abbatoir_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &ABBATOIR_DEED)
}
static ABBATOIR_DEED_CLASS: DeedClass = DeedClass {
    addon: |w, _deed| abbatoir_addon_new(w),
    select_direction: None,
};
leaf! {
    /// `Server.Items.AbbatoirDeed`.
    ABBATOIR_DEED = "Server.Items.AbbatoirDeed" : BASE_ADDON_DEED, v0;
    construct: Some(abbatoir_deed_new),
    family: Some(&ABBATOIR_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `AlchemistTableEastAddon()`.
pub fn alchemist_table_east_addon_new(w: &mut World) -> ItemId {
    let id = addon::new_addon(w, &ALCHEMIST_TABLE_EAST_ADDON);
    addon::add_components(w, id, &[(Comp::Plain(11731), 0, 0, 0)]);
    id
}
static ALCHEMIST_TABLE_EAST_ADDON_CLASS: AddonClass = AddonClass {
    deed: Some(alchemist_table_east_deed_new),
    ..BASE_ADDON_CLASS
};
leaf! {
    /// `Server.Items.AlchemistTableEastAddon`.
    ALCHEMIST_TABLE_EAST_ADDON = "Server.Items.AlchemistTableEastAddon" : BASE_ADDON, enc;
    construct: Some(alchemist_table_east_addon_new),
    family: Some(&ALCHEMIST_TABLE_EAST_ADDON_CLASS),
}

/// `AlchemistTableEastDeed()`.
pub fn alchemist_table_east_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &ALCHEMIST_TABLE_EAST_DEED)
}
static ALCHEMIST_TABLE_EAST_DEED_CLASS: DeedClass = DeedClass {
    addon: |w, _deed| alchemist_table_east_addon_new(w),
    select_direction: None,
};
leaf! {
    /// `Server.Items.AlchemistTableEastDeed`.
    ALCHEMIST_TABLE_EAST_DEED = "Server.Items.AlchemistTableEastDeed" : BASE_ADDON_DEED, enc;
    construct: Some(alchemist_table_east_deed_new),
    family: Some(&ALCHEMIST_TABLE_EAST_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `AlchemistTableSouthAddon()`.
pub fn alchemist_table_south_addon_new(w: &mut World) -> ItemId {
    let id = addon::new_addon(w, &ALCHEMIST_TABLE_SOUTH_ADDON);
    addon::add_components(w, id, &[(Comp::Plain(11732), 0, 0, 0)]);
    id
}
static ALCHEMIST_TABLE_SOUTH_ADDON_CLASS: AddonClass = AddonClass {
    deed: Some(alchemist_table_south_deed_new),
    ..BASE_ADDON_CLASS
};
leaf! {
    /// `Server.Items.AlchemistTableSouthAddon`.
    ALCHEMIST_TABLE_SOUTH_ADDON = "Server.Items.AlchemistTableSouthAddon" : BASE_ADDON, enc;
    construct: Some(alchemist_table_south_addon_new),
    family: Some(&ALCHEMIST_TABLE_SOUTH_ADDON_CLASS),
}

/// `AlchemistTableSouthDeed()`.
pub fn alchemist_table_south_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &ALCHEMIST_TABLE_SOUTH_DEED)
}
static ALCHEMIST_TABLE_SOUTH_DEED_CLASS: DeedClass = DeedClass {
    addon: |w, _deed| alchemist_table_south_addon_new(w),
    select_direction: None,
};
leaf! {
    /// `Server.Items.AlchemistTableSouthDeed`.
    ALCHEMIST_TABLE_SOUTH_DEED = "Server.Items.AlchemistTableSouthDeed" : BASE_ADDON_DEED, enc;
    construct: Some(alchemist_table_south_deed_new),
    family: Some(&ALCHEMIST_TABLE_SOUTH_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `AnvilEastAddon()`.
pub fn anvil_east_addon_new(w: &mut World) -> ItemId {
    let id = addon::new_addon(w, &ANVIL_EAST_ADDON);
    addon::add_components(w, id, &[(Comp::Anvil(4015), 0, 0, 0)]);
    id
}
static ANVIL_EAST_ADDON_CLASS: AddonClass = AddonClass {
    deed: Some(anvil_east_deed_new),
    ..BASE_ADDON_CLASS
};
leaf! {
    /// `Server.Items.AnvilEastAddon`.
    ANVIL_EAST_ADDON = "Server.Items.AnvilEastAddon" : BASE_ADDON, v0;
    construct: Some(anvil_east_addon_new),
    family: Some(&ANVIL_EAST_ADDON_CLASS),
}

/// `AnvilEastDeed()`.
pub fn anvil_east_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &ANVIL_EAST_DEED)
}
static ANVIL_EAST_DEED_CLASS: DeedClass = DeedClass {
    addon: |w, _deed| anvil_east_addon_new(w),
    select_direction: None,
};
leaf! {
    /// `Server.Items.AnvilEastDeed`.
    ANVIL_EAST_DEED = "Server.Items.AnvilEastDeed" : BASE_ADDON_DEED, v0;
    construct: Some(anvil_east_deed_new),
    family: Some(&ANVIL_EAST_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `AnvilSouthAddon()`.
pub fn anvil_south_addon_new(w: &mut World) -> ItemId {
    let id = addon::new_addon(w, &ANVIL_SOUTH_ADDON);
    addon::add_components(w, id, &[(Comp::Anvil(4016), 0, 0, 0)]);
    id
}
static ANVIL_SOUTH_ADDON_CLASS: AddonClass = AddonClass {
    deed: Some(anvil_south_deed_new),
    ..BASE_ADDON_CLASS
};
leaf! {
    /// `Server.Items.AnvilSouthAddon`.
    ANVIL_SOUTH_ADDON = "Server.Items.AnvilSouthAddon" : BASE_ADDON, v0;
    construct: Some(anvil_south_addon_new),
    family: Some(&ANVIL_SOUTH_ADDON_CLASS),
}

/// `AnvilSouthDeed()`.
pub fn anvil_south_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &ANVIL_SOUTH_DEED)
}
static ANVIL_SOUTH_DEED_CLASS: DeedClass = DeedClass {
    addon: |w, _deed| anvil_south_addon_new(w),
    select_direction: None,
};
leaf! {
    /// `Server.Items.AnvilSouthDeed`.
    ANVIL_SOUTH_DEED = "Server.Items.AnvilSouthDeed" : BASE_ADDON_DEED, v0;
    construct: Some(anvil_south_deed_new),
    family: Some(&ANVIL_SOUTH_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `ArcaneBookshelfEastAddon()`.
pub fn arcane_bookshelf_east_addon_new(w: &mut World) -> ItemId {
    let id = addon_container::new_addon_container(w, &ARCANE_BOOKSHELF_EAST_ADDON, 0x3084);
    addon_container::add_components(w, id, &[(CComp::Plain(12421), -1, 0, 0)]);
    id
}
static ARCANE_BOOKSHELF_EAST_ADDON_CLASS: ContainerAddonClass = ContainerAddonClass {
    deed: Some(arcane_bookshelf_east_deed_new),
    retain_deed_hue: true,
    ..BASE_ADDON_CONTAINER_CLASS
};
leaf! {
    /// `Server.Items.ArcaneBookshelfEastAddon`.
    ARCANE_BOOKSHELF_EAST_ADDON = "Server.Items.ArcaneBookshelfEastAddon" : BASE_ADDON_CONTAINER, enc;
    construct: Some(arcane_bookshelf_east_addon_new),
    family: Some(&ARCANE_BOOKSHELF_EAST_ADDON_CLASS),
    default_gump_id: crate::class_props::default_gump_id,
    default_drop_sound: crate::class_props::default_drop_sound,
}

/// `ArcaneBookshelfEastDeed()`.
pub fn arcane_bookshelf_east_deed_new(w: &mut World) -> ItemId {
    addon_container::new_deed(w, &ARCANE_BOOKSHELF_EAST_DEED)
}
static ARCANE_BOOKSHELF_EAST_DEED_CLASS: ContainerDeedClass = ContainerDeedClass {
    addon: |w, _deed| arcane_bookshelf_east_addon_new(w),
    select_direction: None,
};
leaf! {
    /// `Server.Items.ArcaneBookshelfEastDeed`.
    ARCANE_BOOKSHELF_EAST_DEED = "Server.Items.ArcaneBookshelfEastDeed" : BASE_ADDON_CONTAINER_DEED, enc;
    construct: Some(arcane_bookshelf_east_deed_new),
    family: Some(&ARCANE_BOOKSHELF_EAST_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `ArcaneBookshelfSouthAddon()`.
pub fn arcane_bookshelf_south_addon_new(w: &mut World) -> ItemId {
    let id = addon_container::new_addon_container(w, &ARCANE_BOOKSHELF_SOUTH_ADDON, 0x2DEF);
    addon_container::add_components(w, id, &[(CComp::Localized(8612, 1031760), -1, 0, 0)]);
    id
}
static ARCANE_BOOKSHELF_SOUTH_ADDON_CLASS: ContainerAddonClass = ContainerAddonClass {
    deed: Some(arcane_bookshelf_south_deed_new),
    retain_deed_hue: true,
    ..BASE_ADDON_CONTAINER_CLASS
};
leaf! {
    /// `Server.Items.ArcaneBookshelfSouthAddon`.
    ARCANE_BOOKSHELF_SOUTH_ADDON = "Server.Items.ArcaneBookshelfSouthAddon" : BASE_ADDON_CONTAINER, enc;
    construct: Some(arcane_bookshelf_south_addon_new),
    family: Some(&ARCANE_BOOKSHELF_SOUTH_ADDON_CLASS),
    default_gump_id: crate::class_props::default_gump_id,
    default_drop_sound: crate::class_props::default_drop_sound,
    label_number: crate::class_props::label_number,
}

/// `ArcaneBookshelfSouthDeed()`.
pub fn arcane_bookshelf_south_deed_new(w: &mut World) -> ItemId {
    addon_container::new_deed(w, &ARCANE_BOOKSHELF_SOUTH_DEED)
}
static ARCANE_BOOKSHELF_SOUTH_DEED_CLASS: ContainerDeedClass = ContainerDeedClass {
    addon: |w, _deed| arcane_bookshelf_south_addon_new(w),
    select_direction: None,
};
leaf! {
    /// `Server.Items.ArcaneBookshelfSouthDeed`.
    ARCANE_BOOKSHELF_SOUTH_DEED = "Server.Items.ArcaneBookshelfSouthDeed" : BASE_ADDON_CONTAINER_DEED, enc;
    construct: Some(arcane_bookshelf_south_deed_new),
    family: Some(&ARCANE_BOOKSHELF_SOUTH_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `ArcaneCircleAddon()`.
pub fn arcane_circle_addon_new(w: &mut World) -> ItemId {
    let id = addon::new_addon(w, &ARCANE_CIRCLE_ADDON);
    addon::add_components(
        w,
        id,
        &[
            (Comp::Plain(12419), -1, -1, 0),
            (Comp::Plain(12416), -1, 0, 0),
            (Comp::Plain(12418), 0, -1, 0),
            (Comp::Plain(12417), 1, -1, 0),
            (Comp::Plain(12413), -1, 1, 0),
            (Comp::Plain(12415), 0, 0, 0),
            (Comp::Plain(12414), 1, 0, 0),
            (Comp::Plain(12412), 0, 1, 0),
            (Comp::Plain(12411), 1, 1, 0),
        ],
    );
    id
}
static ARCANE_CIRCLE_ADDON_CLASS: AddonClass = AddonClass {
    deed: Some(arcane_circle_deed_new),
    ..BASE_ADDON_CLASS
};
leaf! {
    /// `Server.Items.ArcaneCircleAddon`.
    ARCANE_CIRCLE_ADDON = "Server.Items.ArcaneCircleAddon" : BASE_ADDON, enc;
    construct: Some(arcane_circle_addon_new),
    family: Some(&ARCANE_CIRCLE_ADDON_CLASS),
}

/// `ArcaneCircleDeed()`.
pub fn arcane_circle_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &ARCANE_CIRCLE_DEED)
}
static ARCANE_CIRCLE_DEED_CLASS: DeedClass = DeedClass {
    addon: |w, _deed| arcane_circle_addon_new(w),
    select_direction: None,
};
leaf! {
    /// `Server.Items.ArcaneCircleDeed`.
    ARCANE_CIRCLE_DEED = "Server.Items.ArcaneCircleDeed" : BASE_ADDON_DEED, enc;
    construct: Some(arcane_circle_deed_new),
    family: Some(&ARCANE_CIRCLE_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `ArcanistStatueEastAddon()`.
pub fn arcanist_statue_east_addon_new(w: &mut World) -> ItemId {
    let id = addon::new_addon(w, &ARCANIST_STATUE_EAST_ADDON);
    addon::add_components(w, id, &[(Comp::Plain(11534), 0, 0, 0)]);
    id
}
static ARCANIST_STATUE_EAST_ADDON_CLASS: AddonClass = AddonClass {
    deed: Some(arcanist_statue_east_deed_new),
    ..BASE_ADDON_CLASS
};
leaf! {
    /// `Server.Items.ArcanistStatueEastAddon`.
    ARCANIST_STATUE_EAST_ADDON = "Server.Items.ArcanistStatueEastAddon" : BASE_ADDON, enc;
    construct: Some(arcanist_statue_east_addon_new),
    family: Some(&ARCANIST_STATUE_EAST_ADDON_CLASS),
}

/// `ArcanistStatueEastDeed()`.
pub fn arcanist_statue_east_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &ARCANIST_STATUE_EAST_DEED)
}
static ARCANIST_STATUE_EAST_DEED_CLASS: DeedClass = DeedClass {
    addon: |w, _deed| arcanist_statue_east_addon_new(w),
    select_direction: None,
};
leaf! {
    /// `Server.Items.ArcanistStatueEastDeed`.
    ARCANIST_STATUE_EAST_DEED = "Server.Items.ArcanistStatueEastDeed" : BASE_ADDON_DEED, enc;
    construct: Some(arcanist_statue_east_deed_new),
    family: Some(&ARCANIST_STATUE_EAST_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `ArcanistStatueSouthAddon()`.
pub fn arcanist_statue_south_addon_new(w: &mut World) -> ItemId {
    let id = addon::new_addon(w, &ARCANIST_STATUE_SOUTH_ADDON);
    addon::add_components(w, id, &[(Comp::Plain(11535), 0, 0, 0)]);
    id
}
static ARCANIST_STATUE_SOUTH_ADDON_CLASS: AddonClass = AddonClass {
    deed: Some(arcanist_statue_south_deed_new),
    ..BASE_ADDON_CLASS
};
leaf! {
    /// `Server.Items.ArcanistStatueSouthAddon`.
    ARCANIST_STATUE_SOUTH_ADDON = "Server.Items.ArcanistStatueSouthAddon" : BASE_ADDON, enc;
    construct: Some(arcanist_statue_south_addon_new),
    family: Some(&ARCANIST_STATUE_SOUTH_ADDON_CLASS),
}

/// `ArcanistStatueSouthDeed()`.
pub fn arcanist_statue_south_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &ARCANIST_STATUE_SOUTH_DEED)
}
static ARCANIST_STATUE_SOUTH_DEED_CLASS: DeedClass = DeedClass {
    addon: |w, _deed| arcanist_statue_south_addon_new(w),
    select_direction: None,
};
leaf! {
    /// `Server.Items.ArcanistStatueSouthDeed`.
    ARCANIST_STATUE_SOUTH_DEED = "Server.Items.ArcanistStatueSouthDeed" : BASE_ADDON_DEED, enc;
    construct: Some(arcanist_statue_south_deed_new),
    family: Some(&ARCANIST_STATUE_SOUTH_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `BrownBearRugEastAddon()`.
pub fn brown_bear_rug_east_addon_new(w: &mut World) -> ItemId {
    let id = addon::new_addon(w, &BROWN_BEAR_RUG_EAST_ADDON);
    addon::add_components(
        w,
        id,
        &[
            (Comp::Plain(7744), 1, 1, 0),
            (Comp::Plain(7745), 1, 0, 0),
            (Comp::Plain(7746), 1, -1, 0),
            (Comp::Plain(7747), 0, -1, 0),
            (Comp::Plain(7748), 0, 0, 0),
            (Comp::Plain(7749), 0, 1, 0),
            (Comp::Plain(7750), -1, 1, 0),
            (Comp::Plain(7751), -1, 0, 0),
            (Comp::Plain(7752), -1, -1, 0),
        ],
    );
    id
}
static BROWN_BEAR_RUG_EAST_ADDON_CLASS: AddonClass = AddonClass {
    deed: Some(brown_bear_rug_east_deed_new),
    ..BASE_ADDON_CLASS
};
leaf! {
    /// `Server.Items.BrownBearRugEastAddon`.
    BROWN_BEAR_RUG_EAST_ADDON = "Server.Items.BrownBearRugEastAddon" : BASE_ADDON, v0;
    construct: Some(brown_bear_rug_east_addon_new),
    family: Some(&BROWN_BEAR_RUG_EAST_ADDON_CLASS),
}

/// `BrownBearRugEastDeed()`.
pub fn brown_bear_rug_east_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &BROWN_BEAR_RUG_EAST_DEED)
}
static BROWN_BEAR_RUG_EAST_DEED_CLASS: DeedClass = DeedClass {
    addon: |w, _deed| brown_bear_rug_east_addon_new(w),
    select_direction: None,
};
leaf! {
    /// `Server.Items.BrownBearRugEastDeed`.
    BROWN_BEAR_RUG_EAST_DEED = "Server.Items.BrownBearRugEastDeed" : BASE_ADDON_DEED, v0;
    construct: Some(brown_bear_rug_east_deed_new),
    family: Some(&BROWN_BEAR_RUG_EAST_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `BrownBearRugSouthAddon()`.
pub fn brown_bear_rug_south_addon_new(w: &mut World) -> ItemId {
    let id = addon::new_addon(w, &BROWN_BEAR_RUG_SOUTH_ADDON);
    addon::add_components(
        w,
        id,
        &[
            (Comp::Plain(7734), 1, 1, 0),
            (Comp::Plain(7735), 0, 1, 0),
            (Comp::Plain(7736), -1, 1, 0),
            (Comp::Plain(7737), -1, 0, 0),
            (Comp::Plain(7738), 0, 0, 0),
            (Comp::Plain(7739), 1, 0, 0),
            (Comp::Plain(7740), 1, -1, 0),
            (Comp::Plain(7741), 0, -1, 0),
            (Comp::Plain(7742), -1, -1, 0),
        ],
    );
    id
}
static BROWN_BEAR_RUG_SOUTH_ADDON_CLASS: AddonClass = AddonClass {
    deed: Some(brown_bear_rug_south_deed_new),
    ..BASE_ADDON_CLASS
};
leaf! {
    /// `Server.Items.BrownBearRugSouthAddon`.
    BROWN_BEAR_RUG_SOUTH_ADDON = "Server.Items.BrownBearRugSouthAddon" : BASE_ADDON, v0;
    construct: Some(brown_bear_rug_south_addon_new),
    family: Some(&BROWN_BEAR_RUG_SOUTH_ADDON_CLASS),
}

/// `BrownBearRugSouthDeed()`.
pub fn brown_bear_rug_south_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &BROWN_BEAR_RUG_SOUTH_DEED)
}
static BROWN_BEAR_RUG_SOUTH_DEED_CLASS: DeedClass = DeedClass {
    addon: |w, _deed| brown_bear_rug_south_addon_new(w),
    select_direction: None,
};
leaf! {
    /// `Server.Items.BrownBearRugSouthDeed`.
    BROWN_BEAR_RUG_SOUTH_DEED = "Server.Items.BrownBearRugSouthDeed" : BASE_ADDON_DEED, v0;
    construct: Some(brown_bear_rug_south_deed_new),
    family: Some(&BROWN_BEAR_RUG_SOUTH_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `PolarBearRugEastAddon()`.
pub fn polar_bear_rug_east_addon_new(w: &mut World) -> ItemId {
    let id = addon::new_addon(w, &POLAR_BEAR_RUG_EAST_ADDON);
    addon::add_components(
        w,
        id,
        &[
            (Comp::Plain(7763), 1, 1, 0),
            (Comp::Plain(7764), 1, 0, 0),
            (Comp::Plain(7765), 1, -1, 0),
            (Comp::Plain(7766), 0, -1, 0),
            (Comp::Plain(7767), 0, 0, 0),
            (Comp::Plain(7768), 0, 1, 0),
            (Comp::Plain(7769), -1, 1, 0),
            (Comp::Plain(7770), -1, 0, 0),
            (Comp::Plain(7771), -1, -1, 0),
        ],
    );
    id
}
static POLAR_BEAR_RUG_EAST_ADDON_CLASS: AddonClass = AddonClass {
    deed: Some(polar_bear_rug_east_deed_new),
    ..BASE_ADDON_CLASS
};
leaf! {
    /// `Server.Items.PolarBearRugEastAddon`.
    POLAR_BEAR_RUG_EAST_ADDON = "Server.Items.PolarBearRugEastAddon" : BASE_ADDON, v0;
    construct: Some(polar_bear_rug_east_addon_new),
    family: Some(&POLAR_BEAR_RUG_EAST_ADDON_CLASS),
}

/// `PolarBearRugEastDeed()`.
pub fn polar_bear_rug_east_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &POLAR_BEAR_RUG_EAST_DEED)
}
static POLAR_BEAR_RUG_EAST_DEED_CLASS: DeedClass = DeedClass {
    addon: |w, _deed| polar_bear_rug_east_addon_new(w),
    select_direction: None,
};
leaf! {
    /// `Server.Items.PolarBearRugEastDeed`.
    POLAR_BEAR_RUG_EAST_DEED = "Server.Items.PolarBearRugEastDeed" : BASE_ADDON_DEED, v0;
    construct: Some(polar_bear_rug_east_deed_new),
    family: Some(&POLAR_BEAR_RUG_EAST_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `PolarBearRugSouthAddon()`.
pub fn polar_bear_rug_south_addon_new(w: &mut World) -> ItemId {
    let id = addon::new_addon(w, &POLAR_BEAR_RUG_SOUTH_ADDON);
    addon::add_components(
        w,
        id,
        &[
            (Comp::Plain(7753), 1, 1, 0),
            (Comp::Plain(7754), 0, 1, 0),
            (Comp::Plain(7755), -1, 1, 0),
            (Comp::Plain(7756), -1, 0, 0),
            (Comp::Plain(7757), 0, 0, 0),
            (Comp::Plain(7758), 1, 0, 0),
            (Comp::Plain(7759), 1, -1, 0),
            (Comp::Plain(7760), 0, -1, 0),
            (Comp::Plain(7761), -1, -1, 0),
        ],
    );
    id
}
static POLAR_BEAR_RUG_SOUTH_ADDON_CLASS: AddonClass = AddonClass {
    deed: Some(polar_bear_rug_south_deed_new),
    ..BASE_ADDON_CLASS
};
leaf! {
    /// `Server.Items.PolarBearRugSouthAddon`.
    POLAR_BEAR_RUG_SOUTH_ADDON = "Server.Items.PolarBearRugSouthAddon" : BASE_ADDON, v0;
    construct: Some(polar_bear_rug_south_addon_new),
    family: Some(&POLAR_BEAR_RUG_SOUTH_ADDON_CLASS),
}

/// `PolarBearRugSouthDeed()`.
pub fn polar_bear_rug_south_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &POLAR_BEAR_RUG_SOUTH_DEED)
}
static POLAR_BEAR_RUG_SOUTH_DEED_CLASS: DeedClass = DeedClass {
    addon: |w, _deed| polar_bear_rug_south_addon_new(w),
    select_direction: None,
};
leaf! {
    /// `Server.Items.PolarBearRugSouthDeed`.
    POLAR_BEAR_RUG_SOUTH_DEED = "Server.Items.PolarBearRugSouthDeed" : BASE_ADDON_DEED, v0;
    construct: Some(polar_bear_rug_south_deed_new),
    family: Some(&POLAR_BEAR_RUG_SOUTH_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `BloodPentagram()`.
pub fn blood_pentagram_new(w: &mut World) -> ItemId {
    let id = addon::new_addon(w, &BLOOD_PENTAGRAM);
    addon::add_components(
        w,
        id,
        &[
            (Comp::Plain(7417), 0, 1, 0),
            (Comp::Plain(7416), 0, 2, 0),
            (Comp::Plain(7415), 0, 3, 0),
            (Comp::Plain(7414), 0, 4, 0),
            (Comp::Plain(7413), 0, 5, 0),
            (Comp::Plain(7419), 1, 0, 0),
            (Comp::Plain(7418), 1, 1, 0),
            (Comp::Plain(7433), 1, 2, 0),
            (Comp::Plain(7432), 1, 3, 0),
            (Comp::Plain(7431), 1, 4, 0),
            (Comp::Plain(7412), 1, 5, 0),
            (Comp::Plain(7420), 2, 0, 0),
            (Comp::Plain(7434), 2, 1, 0),
            (Comp::Plain(7441), 2, 2, 0),
            (Comp::Plain(7440), 2, 3, 0),
            (Comp::Plain(7430), 2, 4, 0),
            (Comp::Plain(7411), 2, 5, 0),
            (Comp::Plain(7421), 3, 0, 0),
            (Comp::Plain(7435), 3, 1, 0),
            (Comp::Plain(7442), 3, 2, 0),
            (Comp::Plain(7439), 3, 3, 0),
            (Comp::Plain(7429), 3, 4, 0),
            (Comp::Plain(7410), 3, 5, 0),
            (Comp::Plain(7422), 4, 0, 0),
            (Comp::Plain(7436), 4, 1, 0),
            (Comp::Plain(7437), 4, 2, 0),
            (Comp::Plain(7438), 4, 3, 0),
            (Comp::Plain(7428), 4, 4, 0),
            (Comp::Plain(7409), 4, 5, 0),
            (Comp::Plain(7423), 5, 0, 0),
            (Comp::Plain(7424), 5, 1, 0),
            (Comp::Plain(7425), 5, 2, 0),
            (Comp::Plain(7426), 5, 3, 0),
            (Comp::Plain(7427), 5, 4, 0),
        ],
    );
    id
}
static BLOOD_PENTAGRAM_CLASS: AddonClass = AddonClass { ..BASE_ADDON_CLASS };
leaf! {
    /// `Server.Items.BloodPentagram`.
    BLOOD_PENTAGRAM = "Server.Items.BloodPentagram" : BASE_ADDON, v0;
    construct: Some(blood_pentagram_new),
    family: Some(&BLOOD_PENTAGRAM_CLASS),
}

/// `ElvenBedEastAddon()`.
pub fn elven_bed_east_addon_new(w: &mut World) -> ItemId {
    let id = addon::new_addon(w, &ELVEN_BED_EAST_ADDON);
    addon::add_components(
        w,
        id,
        &[(Comp::Plain(12365), 0, 0, 0), (Comp::Plain(12364), 1, 0, 0)],
    );
    id
}
static ELVEN_BED_EAST_ADDON_CLASS: AddonClass = AddonClass {
    deed: Some(elven_bed_east_deed_new),
    ..BASE_ADDON_CLASS
};
leaf! {
    /// `Server.Items.ElvenBedEastAddon`.
    ELVEN_BED_EAST_ADDON = "Server.Items.ElvenBedEastAddon" : BASE_ADDON, enc;
    construct: Some(elven_bed_east_addon_new),
    family: Some(&ELVEN_BED_EAST_ADDON_CLASS),
}

/// `ElvenBedEastDeed()`.
pub fn elven_bed_east_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &ELVEN_BED_EAST_DEED)
}
static ELVEN_BED_EAST_DEED_CLASS: DeedClass = DeedClass {
    addon: |w, _deed| elven_bed_east_addon_new(w),
    select_direction: None,
};
leaf! {
    /// `Server.Items.ElvenBedEastDeed`.
    ELVEN_BED_EAST_DEED = "Server.Items.ElvenBedEastDeed" : BASE_ADDON_DEED, enc;
    construct: Some(elven_bed_east_deed_new),
    family: Some(&ELVEN_BED_EAST_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `ElvenBedSouthAddon()`.
pub fn elven_bed_south_addon_new(w: &mut World) -> ItemId {
    let id = addon::new_addon(w, &ELVEN_BED_SOUTH_ADDON);
    addon::add_components(
        w,
        id,
        &[
            (Comp::Plain(12368), 0, 0, 0),
            (Comp::Plain(12369), 0, -1, 0),
        ],
    );
    id
}
static ELVEN_BED_SOUTH_ADDON_CLASS: AddonClass = AddonClass {
    deed: Some(elven_bed_south_deed_new),
    ..BASE_ADDON_CLASS
};
leaf! {
    /// `Server.Items.ElvenBedSouthAddon`.
    ELVEN_BED_SOUTH_ADDON = "Server.Items.ElvenBedSouthAddon" : BASE_ADDON, enc;
    construct: Some(elven_bed_south_addon_new),
    family: Some(&ELVEN_BED_SOUTH_ADDON_CLASS),
}

/// `ElvenBedSouthDeed()`.
pub fn elven_bed_south_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &ELVEN_BED_SOUTH_DEED)
}
static ELVEN_BED_SOUTH_DEED_CLASS: DeedClass = DeedClass {
    addon: |w, _deed| elven_bed_south_addon_new(w),
    select_direction: None,
};
leaf! {
    /// `Server.Items.ElvenBedSouthDeed`.
    ELVEN_BED_SOUTH_DEED = "Server.Items.ElvenBedSouthDeed" : BASE_ADDON_DEED, enc;
    construct: Some(elven_bed_south_deed_new),
    family: Some(&ELVEN_BED_SOUTH_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `ElvenDresserEastAddon()`.
pub fn elven_dresser_east_addon_new(w: &mut World) -> ItemId {
    let id = addon_container::new_addon_container(w, &ELVEN_DRESSER_EAST_ADDON, 0x30E4);
    addon_container::add_components(w, id, &[(CComp::Plain(12515), 0, -1, 0)]);
    id
}
static ELVEN_DRESSER_EAST_ADDON_CLASS: ContainerAddonClass = ContainerAddonClass {
    deed: Some(elven_dresser_east_deed_new),
    retain_deed_hue: true,
    ..BASE_ADDON_CONTAINER_CLASS
};
leaf! {
    /// `Server.Items.ElvenDresserEastAddon`.
    ELVEN_DRESSER_EAST_ADDON = "Server.Items.ElvenDresserEastAddon" : BASE_ADDON_CONTAINER, enc;
    construct: Some(elven_dresser_east_addon_new),
    family: Some(&ELVEN_DRESSER_EAST_ADDON_CLASS),
    default_gump_id: crate::class_props::default_gump_id,
    default_drop_sound: crate::class_props::default_drop_sound,
}

/// `ElvenDresserEastDeed()`.
pub fn elven_dresser_east_deed_new(w: &mut World) -> ItemId {
    addon_container::new_deed(w, &ELVEN_DRESSER_EAST_DEED)
}
static ELVEN_DRESSER_EAST_DEED_CLASS: ContainerDeedClass = ContainerDeedClass {
    addon: |w, _deed| elven_dresser_east_addon_new(w),
    select_direction: None,
};
leaf! {
    /// `Server.Items.ElvenDresserEastDeed`.
    ELVEN_DRESSER_EAST_DEED = "Server.Items.ElvenDresserEastDeed" : BASE_ADDON_CONTAINER_DEED, enc;
    construct: Some(elven_dresser_east_deed_new),
    family: Some(&ELVEN_DRESSER_EAST_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `ElvenDresserSouthAddon()`.
pub fn elven_dresser_south_addon_new(w: &mut World) -> ItemId {
    let id = addon_container::new_addon_container(w, &ELVEN_DRESSER_SOUTH_ADDON, 0x30E6);
    addon_container::add_components(w, id, &[(CComp::Plain(12517), -1, 0, 0)]);
    id
}
static ELVEN_DRESSER_SOUTH_ADDON_CLASS: ContainerAddonClass = ContainerAddonClass {
    deed: Some(elven_dresser_south_deed_new),
    retain_deed_hue: true,
    ..BASE_ADDON_CONTAINER_CLASS
};
leaf! {
    /// `Server.Items.ElvenDresserSouthAddon`.
    ELVEN_DRESSER_SOUTH_ADDON = "Server.Items.ElvenDresserSouthAddon" : BASE_ADDON_CONTAINER, enc;
    construct: Some(elven_dresser_south_addon_new),
    family: Some(&ELVEN_DRESSER_SOUTH_ADDON_CLASS),
    default_gump_id: crate::class_props::default_gump_id,
    default_drop_sound: crate::class_props::default_drop_sound,
}

/// `ElvenDresserSouthDeed()`.
pub fn elven_dresser_south_deed_new(w: &mut World) -> ItemId {
    addon_container::new_deed(w, &ELVEN_DRESSER_SOUTH_DEED)
}
static ELVEN_DRESSER_SOUTH_DEED_CLASS: ContainerDeedClass = ContainerDeedClass {
    addon: |w, _deed| elven_dresser_south_addon_new(w),
    select_direction: None,
};
leaf! {
    /// `Server.Items.ElvenDresserSouthDeed`.
    ELVEN_DRESSER_SOUTH_DEED = "Server.Items.ElvenDresserSouthDeed" : BASE_ADDON_CONTAINER_DEED, enc;
    construct: Some(elven_dresser_south_deed_new),
    family: Some(&ELVEN_DRESSER_SOUTH_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `ElvenForgeAddon()`.
pub fn elven_forge_addon_new(w: &mut World) -> ItemId {
    let id = addon::new_addon(w, &ELVEN_FORGE_ADDON);
    addon::add_components(w, id, &[(Comp::Plain(11736), 0, 0, 0)]);
    id
}
static ELVEN_FORGE_ADDON_CLASS: AddonClass = AddonClass {
    deed: Some(elven_forge_deed_new),
    ..BASE_ADDON_CLASS
};
leaf! {
    /// `Server.Items.ElvenForgeAddon`.
    ELVEN_FORGE_ADDON = "Server.Items.ElvenForgeAddon" : BASE_ADDON, enc;
    construct: Some(elven_forge_addon_new),
    family: Some(&ELVEN_FORGE_ADDON_CLASS),
    label_number: crate::class_props::label_number,
}

/// `ElvenForgeDeed()`.
pub fn elven_forge_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &ELVEN_FORGE_DEED)
}
static ELVEN_FORGE_DEED_CLASS: DeedClass = DeedClass {
    addon: |w, _deed| elven_forge_addon_new(w),
    select_direction: None,
};
leaf! {
    /// `Server.Items.ElvenForgeDeed`.
    ELVEN_FORGE_DEED = "Server.Items.ElvenForgeDeed" : BASE_ADDON_DEED, enc;
    construct: Some(elven_forge_deed_new),
    family: Some(&ELVEN_FORGE_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `ElvenLoveseatEastAddon()`.
pub fn elven_loveseat_east_addon_new(w: &mut World) -> ItemId {
    let id = addon::new_addon(w, &ELVEN_LOVESEAT_EAST_ADDON);
    addon::add_components(
        w,
        id,
        &[
            (Comp::Plain(12426), 0, 0, 0),
            (Comp::Plain(12427), 0, -1, 0),
        ],
    );
    id
}
static ELVEN_LOVESEAT_EAST_ADDON_CLASS: AddonClass = AddonClass {
    deed: Some(elven_loveseat_east_deed_new),
    ..BASE_ADDON_CLASS
};
leaf! {
    /// `Server.Items.ElvenLoveseatEastAddon`.
    ELVEN_LOVESEAT_EAST_ADDON = "Server.Items.ElvenLoveseatEastAddon" : BASE_ADDON, enc;
    construct: Some(elven_loveseat_east_addon_new),
    family: Some(&ELVEN_LOVESEAT_EAST_ADDON_CLASS),
}

/// `ElvenLoveseatEastDeed()`.
pub fn elven_loveseat_east_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &ELVEN_LOVESEAT_EAST_DEED)
}
static ELVEN_LOVESEAT_EAST_DEED_CLASS: DeedClass = DeedClass {
    addon: |w, _deed| elven_loveseat_east_addon_new(w),
    select_direction: None,
};
leaf! {
    /// `Server.Items.ElvenLoveseatEastDeed`.
    ELVEN_LOVESEAT_EAST_DEED = "Server.Items.ElvenLoveseatEastDeed" : BASE_ADDON_DEED, enc;
    construct: Some(elven_loveseat_east_deed_new),
    family: Some(&ELVEN_LOVESEAT_EAST_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `ElvenLoveseatSouthAddon()`.
pub fn elven_loveseat_south_addon_new(w: &mut World) -> ItemId {
    let id = addon::new_addon(w, &ELVEN_LOVESEAT_SOUTH_ADDON);
    addon::add_components(
        w,
        id,
        &[
            (Comp::Plain(12424), 0, 0, 0),
            (Comp::Plain(12425), -1, 0, 0),
        ],
    );
    id
}
static ELVEN_LOVESEAT_SOUTH_ADDON_CLASS: AddonClass = AddonClass {
    deed: Some(elven_loveseat_south_deed_new),
    ..BASE_ADDON_CLASS
};
leaf! {
    /// `Server.Items.ElvenLoveseatSouthAddon`.
    ELVEN_LOVESEAT_SOUTH_ADDON = "Server.Items.ElvenLoveseatSouthAddon" : BASE_ADDON, enc;
    construct: Some(elven_loveseat_south_addon_new),
    family: Some(&ELVEN_LOVESEAT_SOUTH_ADDON_CLASS),
}

/// `ElvenLoveseatSouthDeed()`.
pub fn elven_loveseat_south_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &ELVEN_LOVESEAT_SOUTH_DEED)
}
static ELVEN_LOVESEAT_SOUTH_DEED_CLASS: DeedClass = DeedClass {
    addon: |w, _deed| elven_loveseat_south_addon_new(w),
    select_direction: None,
};
leaf! {
    /// `Server.Items.ElvenLoveseatSouthDeed`.
    ELVEN_LOVESEAT_SOUTH_DEED = "Server.Items.ElvenLoveseatSouthDeed" : BASE_ADDON_DEED, enc;
    construct: Some(elven_loveseat_south_deed_new),
    family: Some(&ELVEN_LOVESEAT_SOUTH_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `ElvenStoveEastAddon()`.
pub fn elven_stove_east_addon_new(w: &mut World) -> ItemId {
    let id = addon::new_addon(w, &ELVEN_STOVE_EAST_ADDON);
    addon::add_components(w, id, &[(Comp::Plain(11739), 0, 0, 0)]);
    id
}
static ELVEN_STOVE_EAST_ADDON_CLASS: AddonClass = AddonClass {
    deed: Some(elven_stove_east_deed_new),
    ..BASE_ADDON_CLASS
};
leaf! {
    /// `Server.Items.ElvenStoveEastAddon`.
    ELVEN_STOVE_EAST_ADDON = "Server.Items.ElvenStoveEastAddon" : BASE_ADDON, enc;
    construct: Some(elven_stove_east_addon_new),
    family: Some(&ELVEN_STOVE_EAST_ADDON_CLASS),
}

/// `ElvenStoveEastDeed()`.
pub fn elven_stove_east_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &ELVEN_STOVE_EAST_DEED)
}
static ELVEN_STOVE_EAST_DEED_CLASS: DeedClass = DeedClass {
    addon: |w, _deed| elven_stove_east_addon_new(w),
    select_direction: None,
};
leaf! {
    /// `Server.Items.ElvenStoveEastDeed`.
    ELVEN_STOVE_EAST_DEED = "Server.Items.ElvenStoveEastDeed" : BASE_ADDON_DEED, enc;
    construct: Some(elven_stove_east_deed_new),
    family: Some(&ELVEN_STOVE_EAST_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `ElvenStoveSouthAddon()`.
pub fn elven_stove_south_addon_new(w: &mut World) -> ItemId {
    let id = addon::new_addon(w, &ELVEN_STOVE_SOUTH_ADDON);
    addon::add_components(w, id, &[(Comp::Plain(11740), 0, 0, 0)]);
    id
}
static ELVEN_STOVE_SOUTH_ADDON_CLASS: AddonClass = AddonClass {
    deed: Some(elven_stove_south_deed_new),
    ..BASE_ADDON_CLASS
};
leaf! {
    /// `Server.Items.ElvenStoveSouthAddon`.
    ELVEN_STOVE_SOUTH_ADDON = "Server.Items.ElvenStoveSouthAddon" : BASE_ADDON, enc;
    construct: Some(elven_stove_south_addon_new),
    family: Some(&ELVEN_STOVE_SOUTH_ADDON_CLASS),
}

/// `ElvenStoveSouthDeed()`.
pub fn elven_stove_south_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &ELVEN_STOVE_SOUTH_DEED)
}
static ELVEN_STOVE_SOUTH_DEED_CLASS: DeedClass = DeedClass {
    addon: |w, _deed| elven_stove_south_addon_new(w),
    select_direction: None,
};
leaf! {
    /// `Server.Items.ElvenStoveSouthDeed`.
    ELVEN_STOVE_SOUTH_DEED = "Server.Items.ElvenStoveSouthDeed" : BASE_ADDON_DEED, enc;
    construct: Some(elven_stove_south_deed_new),
    family: Some(&ELVEN_STOVE_SOUTH_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `ElvenWashBasinEastAddon()`.
pub fn elven_wash_basin_east_addon_new(w: &mut World) -> ItemId {
    let id = addon_container::new_addon_container(w, &ELVEN_WASH_BASIN_EAST_ADDON, 0x30E0);
    addon_container::add_components(w, id, &[(CComp::Plain(12511), 0, -1, 0)]);
    id
}
static ELVEN_WASH_BASIN_EAST_ADDON_CLASS: ContainerAddonClass = ContainerAddonClass {
    deed: Some(elven_wash_basin_east_deed_new),
    retain_deed_hue: true,
    ..BASE_ADDON_CONTAINER_CLASS
};
leaf! {
    /// `Server.Items.ElvenWashBasinEastAddon`.
    ELVEN_WASH_BASIN_EAST_ADDON = "Server.Items.ElvenWashBasinEastAddon" : BASE_ADDON_CONTAINER, enc;
    construct: Some(elven_wash_basin_east_addon_new),
    family: Some(&ELVEN_WASH_BASIN_EAST_ADDON_CLASS),
    default_gump_id: crate::class_props::default_gump_id,
    default_drop_sound: crate::class_props::default_drop_sound,
}

/// `ElvenWashBasinEastDeed()`.
pub fn elven_wash_basin_east_deed_new(w: &mut World) -> ItemId {
    addon_container::new_deed(w, &ELVEN_WASH_BASIN_EAST_DEED)
}
static ELVEN_WASH_BASIN_EAST_DEED_CLASS: ContainerDeedClass = ContainerDeedClass {
    addon: |w, _deed| elven_wash_basin_east_addon_new(w),
    select_direction: None,
};
leaf! {
    /// `Server.Items.ElvenWashBasinEastDeed`.
    ELVEN_WASH_BASIN_EAST_DEED = "Server.Items.ElvenWashBasinEastDeed" : BASE_ADDON_CONTAINER_DEED, enc;
    construct: Some(elven_wash_basin_east_deed_new),
    family: Some(&ELVEN_WASH_BASIN_EAST_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `ElvenWashBasinSouthAddon()`.
pub fn elven_wash_basin_south_addon_new(w: &mut World) -> ItemId {
    let id = addon_container::new_addon_container(w, &ELVEN_WASH_BASIN_SOUTH_ADDON, 0x30E2);
    addon_container::add_components(w, id, &[(CComp::Plain(12513), -1, 0, 0)]);
    id
}
static ELVEN_WASH_BASIN_SOUTH_ADDON_CLASS: ContainerAddonClass = ContainerAddonClass {
    deed: Some(elven_wash_basin_south_deed_new),
    retain_deed_hue: true,
    ..BASE_ADDON_CONTAINER_CLASS
};
leaf! {
    /// `Server.Items.ElvenWashBasinSouthAddon`.
    ELVEN_WASH_BASIN_SOUTH_ADDON = "Server.Items.ElvenWashBasinSouthAddon" : BASE_ADDON_CONTAINER, enc;
    construct: Some(elven_wash_basin_south_addon_new),
    family: Some(&ELVEN_WASH_BASIN_SOUTH_ADDON_CLASS),
    default_gump_id: crate::class_props::default_gump_id,
    default_drop_sound: crate::class_props::default_drop_sound,
}

/// `ElvenWashBasinSouthDeed()`.
pub fn elven_wash_basin_south_deed_new(w: &mut World) -> ItemId {
    addon_container::new_deed(w, &ELVEN_WASH_BASIN_SOUTH_DEED)
}
static ELVEN_WASH_BASIN_SOUTH_DEED_CLASS: ContainerDeedClass = ContainerDeedClass {
    addon: |w, _deed| elven_wash_basin_south_addon_new(w),
    select_direction: None,
};
leaf! {
    /// `Server.Items.ElvenWashBasinSouthDeed`.
    ELVEN_WASH_BASIN_SOUTH_DEED = "Server.Items.ElvenWashBasinSouthDeed" : BASE_ADDON_CONTAINER_DEED, enc;
    construct: Some(elven_wash_basin_south_deed_new),
    family: Some(&ELVEN_WASH_BASIN_SOUTH_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `FancyElvenTableEastAddon()`.
pub fn fancy_elven_table_east_addon_new(w: &mut World) -> ItemId {
    let id = addon::new_addon(w, &FANCY_ELVEN_TABLE_EAST_ADDON);
    addon::add_components(
        w,
        id,
        &[
            (Comp::Plain(12436), -1, 0, 0),
            (Comp::Plain(12435), 0, 0, 0),
            (Comp::Plain(12434), 1, 0, 0),
        ],
    );
    id
}
static FANCY_ELVEN_TABLE_EAST_ADDON_CLASS: AddonClass = AddonClass {
    deed: Some(fancy_elven_table_east_deed_new),
    ..BASE_ADDON_CLASS
};
leaf! {
    /// `Server.Items.FancyElvenTableEastAddon`.
    FANCY_ELVEN_TABLE_EAST_ADDON = "Server.Items.FancyElvenTableEastAddon" : BASE_ADDON, enc;
    construct: Some(fancy_elven_table_east_addon_new),
    family: Some(&FANCY_ELVEN_TABLE_EAST_ADDON_CLASS),
}

/// `FancyElvenTableEastDeed()`.
pub fn fancy_elven_table_east_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &FANCY_ELVEN_TABLE_EAST_DEED)
}
static FANCY_ELVEN_TABLE_EAST_DEED_CLASS: DeedClass = DeedClass {
    addon: |w, _deed| fancy_elven_table_east_addon_new(w),
    select_direction: None,
};
leaf! {
    /// `Server.Items.FancyElvenTableEastDeed`.
    FANCY_ELVEN_TABLE_EAST_DEED = "Server.Items.FancyElvenTableEastDeed" : BASE_ADDON_DEED, enc;
    construct: Some(fancy_elven_table_east_deed_new),
    family: Some(&FANCY_ELVEN_TABLE_EAST_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `FancyElvenTableSouthAddon()`.
pub fn fancy_elven_table_south_addon_new(w: &mut World) -> ItemId {
    let id = addon::new_addon(w, &FANCY_ELVEN_TABLE_SOUTH_ADDON);
    addon::add_components(
        w,
        id,
        &[
            (Comp::Plain(12437), 0, 1, 0),
            (Comp::Plain(12438), 0, 0, 0),
            (Comp::Plain(12439), 0, -1, 0),
        ],
    );
    id
}
static FANCY_ELVEN_TABLE_SOUTH_ADDON_CLASS: AddonClass = AddonClass {
    deed: Some(fancy_elven_table_south_deed_new),
    ..BASE_ADDON_CLASS
};
leaf! {
    /// `Server.Items.FancyElvenTableSouthAddon`.
    FANCY_ELVEN_TABLE_SOUTH_ADDON = "Server.Items.FancyElvenTableSouthAddon" : BASE_ADDON, enc;
    construct: Some(fancy_elven_table_south_addon_new),
    family: Some(&FANCY_ELVEN_TABLE_SOUTH_ADDON_CLASS),
}

/// `FancyElvenTableSouthDeed()`.
pub fn fancy_elven_table_south_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &FANCY_ELVEN_TABLE_SOUTH_DEED)
}
static FANCY_ELVEN_TABLE_SOUTH_DEED_CLASS: DeedClass = DeedClass {
    addon: |w, _deed| fancy_elven_table_south_addon_new(w),
    select_direction: None,
};
leaf! {
    /// `Server.Items.FancyElvenTableSouthDeed`.
    FANCY_ELVEN_TABLE_SOUTH_DEED = "Server.Items.FancyElvenTableSouthDeed" : BASE_ADDON_DEED, enc;
    construct: Some(fancy_elven_table_south_deed_new),
    family: Some(&FANCY_ELVEN_TABLE_SOUTH_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `FireColumnAddon(bool bloody = false)`.
pub fn fire_column_addon_new(w: &mut World, bloody: bool) -> ItemId {
    let id = addon::new_addon(w, &FIRE_COLUMN_ADDON);
    addon::add_components(
        w,
        id,
        &[
            (Comp::Plain(933), 0, 0, 0),
            (Comp::Plain(933), 0, 0, 5),
            (Comp::Plain(933), 0, 0, 10),
            (Comp::Plain(933), 0, 0, 15),
            (Comp::Plain(6587), 0, 0, 21),
            (Comp::Plain(6571), 0, 0, 23),
        ],
    );
    if bloody {
        addon::add_components(
            w,
            id,
            &[
                (Comp::Plain(4651), -2, 0, 0),
                (Comp::Plain(4654), 0, -2, 0),
                (Comp::Plain(4653), -1, 1, 0),
                (Comp::Plain(4655), 1, -1, 0),
                (Comp::Plain(4653), 0, 1, 0),
                (Comp::Plain(4650), 1, 0, 0),
                (Comp::Plain(4651), 2, -1, 0),
                (Comp::Plain(4651), 0, 2, 0),
                (Comp::Plain(4654), 1, 1, 0),
            ],
        );
    }
    id
}
static FIRE_COLUMN_ADDON_CLASS: AddonClass = AddonClass {
    share_hue: false,
    ..BASE_ADDON_CLASS
};
leaf! {
    /// `Server.Items.FireColumnAddon`.
    FIRE_COLUMN_ADDON = "Server.Items.FireColumnAddon" : BASE_ADDON, v0;
    construct: Some(|w| fire_column_addon_new(w, false)),
    family: Some(&FIRE_COLUMN_ADDON_CLASS),
}

/// `LightFlowerTapestryEastAddon()`.
pub fn light_flower_tapestry_east_addon_new(w: &mut World) -> ItemId {
    let id = addon::new_addon(w, &LIGHT_FLOWER_TAPESTRY_EAST_ADDON);
    addon::add_components(
        w,
        id,
        &[(Comp::Plain(4060), 0, 0, 0), (Comp::Plain(4059), 0, 1, 0)],
    );
    id
}
static LIGHT_FLOWER_TAPESTRY_EAST_ADDON_CLASS: AddonClass = AddonClass {
    deed: Some(light_flower_tapestry_east_deed_new),
    ..BASE_ADDON_CLASS
};
leaf! {
    /// `Server.Items.LightFlowerTapestryEastAddon`.
    LIGHT_FLOWER_TAPESTRY_EAST_ADDON = "Server.Items.LightFlowerTapestryEastAddon" : BASE_ADDON, v0;
    construct: Some(light_flower_tapestry_east_addon_new),
    family: Some(&LIGHT_FLOWER_TAPESTRY_EAST_ADDON_CLASS),
}

/// `LightFlowerTapestryEastDeed()`.
pub fn light_flower_tapestry_east_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &LIGHT_FLOWER_TAPESTRY_EAST_DEED)
}
static LIGHT_FLOWER_TAPESTRY_EAST_DEED_CLASS: DeedClass = DeedClass {
    addon: |w, _deed| light_flower_tapestry_east_addon_new(w),
    select_direction: None,
};
leaf! {
    /// `Server.Items.LightFlowerTapestryEastDeed`.
    LIGHT_FLOWER_TAPESTRY_EAST_DEED = "Server.Items.LightFlowerTapestryEastDeed" : BASE_ADDON_DEED, v0;
    construct: Some(light_flower_tapestry_east_deed_new),
    family: Some(&LIGHT_FLOWER_TAPESTRY_EAST_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `LightFlowerTapestrySouthAddon()`.
pub fn light_flower_tapestry_south_addon_new(w: &mut World) -> ItemId {
    let id = addon::new_addon(w, &LIGHT_FLOWER_TAPESTRY_SOUTH_ADDON);
    addon::add_components(
        w,
        id,
        &[(Comp::Plain(4057), 0, 0, 0), (Comp::Plain(4058), 1, 0, 0)],
    );
    id
}
static LIGHT_FLOWER_TAPESTRY_SOUTH_ADDON_CLASS: AddonClass = AddonClass {
    deed: Some(light_flower_tapestry_south_deed_new),
    ..BASE_ADDON_CLASS
};
leaf! {
    /// `Server.Items.LightFlowerTapestrySouthAddon`.
    LIGHT_FLOWER_TAPESTRY_SOUTH_ADDON = "Server.Items.LightFlowerTapestrySouthAddon" : BASE_ADDON, v0;
    construct: Some(light_flower_tapestry_south_addon_new),
    family: Some(&LIGHT_FLOWER_TAPESTRY_SOUTH_ADDON_CLASS),
}

/// `LightFlowerTapestrySouthDeed()`.
pub fn light_flower_tapestry_south_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &LIGHT_FLOWER_TAPESTRY_SOUTH_DEED)
}
static LIGHT_FLOWER_TAPESTRY_SOUTH_DEED_CLASS: DeedClass = DeedClass {
    addon: |w, _deed| light_flower_tapestry_south_addon_new(w),
    select_direction: None,
};
leaf! {
    /// `Server.Items.LightFlowerTapestrySouthDeed`.
    LIGHT_FLOWER_TAPESTRY_SOUTH_DEED = "Server.Items.LightFlowerTapestrySouthDeed" : BASE_ADDON_DEED, v0;
    construct: Some(light_flower_tapestry_south_deed_new),
    family: Some(&LIGHT_FLOWER_TAPESTRY_SOUTH_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `DarkFlowerTapestryEastAddon()`.
pub fn dark_flower_tapestry_east_addon_new(w: &mut World) -> ItemId {
    let id = addon::new_addon(w, &DARK_FLOWER_TAPESTRY_EAST_ADDON);
    addon::add_components(
        w,
        id,
        &[(Comp::Plain(4064), 0, 0, 0), (Comp::Plain(4063), 0, 1, 0)],
    );
    id
}
static DARK_FLOWER_TAPESTRY_EAST_ADDON_CLASS: AddonClass = AddonClass {
    deed: Some(dark_flower_tapestry_east_deed_new),
    ..BASE_ADDON_CLASS
};
leaf! {
    /// `Server.Items.DarkFlowerTapestryEastAddon`.
    DARK_FLOWER_TAPESTRY_EAST_ADDON = "Server.Items.DarkFlowerTapestryEastAddon" : BASE_ADDON, v0;
    construct: Some(dark_flower_tapestry_east_addon_new),
    family: Some(&DARK_FLOWER_TAPESTRY_EAST_ADDON_CLASS),
}

/// `DarkFlowerTapestryEastDeed()`.
pub fn dark_flower_tapestry_east_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &DARK_FLOWER_TAPESTRY_EAST_DEED)
}
static DARK_FLOWER_TAPESTRY_EAST_DEED_CLASS: DeedClass = DeedClass {
    addon: |w, _deed| dark_flower_tapestry_east_addon_new(w),
    select_direction: None,
};
leaf! {
    /// `Server.Items.DarkFlowerTapestryEastDeed`.
    DARK_FLOWER_TAPESTRY_EAST_DEED = "Server.Items.DarkFlowerTapestryEastDeed" : BASE_ADDON_DEED, v0;
    construct: Some(dark_flower_tapestry_east_deed_new),
    family: Some(&DARK_FLOWER_TAPESTRY_EAST_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `DarkFlowerTapestrySouthAddon()`.
pub fn dark_flower_tapestry_south_addon_new(w: &mut World) -> ItemId {
    let id = addon::new_addon(w, &DARK_FLOWER_TAPESTRY_SOUTH_ADDON);
    addon::add_components(
        w,
        id,
        &[(Comp::Plain(4061), 0, 0, 0), (Comp::Plain(4062), 1, 0, 0)],
    );
    id
}
static DARK_FLOWER_TAPESTRY_SOUTH_ADDON_CLASS: AddonClass = AddonClass {
    deed: Some(dark_flower_tapestry_south_deed_new),
    ..BASE_ADDON_CLASS
};
leaf! {
    /// `Server.Items.DarkFlowerTapestrySouthAddon`.
    DARK_FLOWER_TAPESTRY_SOUTH_ADDON = "Server.Items.DarkFlowerTapestrySouthAddon" : BASE_ADDON, v0;
    construct: Some(dark_flower_tapestry_south_addon_new),
    family: Some(&DARK_FLOWER_TAPESTRY_SOUTH_ADDON_CLASS),
}

/// `DarkFlowerTapestrySouthDeed()`.
pub fn dark_flower_tapestry_south_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &DARK_FLOWER_TAPESTRY_SOUTH_DEED)
}
static DARK_FLOWER_TAPESTRY_SOUTH_DEED_CLASS: DeedClass = DeedClass {
    addon: |w, _deed| dark_flower_tapestry_south_addon_new(w),
    select_direction: None,
};
leaf! {
    /// `Server.Items.DarkFlowerTapestrySouthDeed`.
    DARK_FLOWER_TAPESTRY_SOUTH_DEED = "Server.Items.DarkFlowerTapestrySouthDeed" : BASE_ADDON_DEED, v0;
    construct: Some(dark_flower_tapestry_south_deed_new),
    family: Some(&DARK_FLOWER_TAPESTRY_SOUTH_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `GiantWeb1()`.
pub fn giant_web1_new(w: &mut World) -> ItemId {
    let id = addon::new_addon(w, &GIANT_WEB1);
    addon::add_components(
        w,
        id,
        &[
            (Comp::Plain(4280), 4, -4, 0),
            (Comp::Plain(4281), 3, -3, 0),
            (Comp::Plain(4282), 2, -2, 0),
            (Comp::Plain(4283), 1, -1, 0),
            (Comp::Plain(4284), 0, 0, 0),
        ],
    );
    id
}
static GIANT_WEB1_CLASS: AddonClass = AddonClass { ..BASE_ADDON_CLASS };
leaf! {
    /// `Server.Items.GiantWeb1`.
    GIANT_WEB1 = "Server.Items.GiantWeb1" : BASE_ADDON, enc;
    construct: Some(giant_web1_new),
    family: Some(&GIANT_WEB1_CLASS),
}

/// `GiantWeb2()`.
pub fn giant_web2_new(w: &mut World) -> ItemId {
    let id = addon::new_addon(w, &GIANT_WEB2);
    addon::add_components(
        w,
        id,
        &[
            (Comp::Plain(4285), 0, 0, 0),
            (Comp::Plain(4286), 1, -1, 0),
            (Comp::Plain(4287), 2, -2, 0),
            (Comp::Plain(4288), 3, -3, 0),
            (Comp::Plain(4289), 4, -4, 0),
        ],
    );
    id
}
static GIANT_WEB2_CLASS: AddonClass = AddonClass { ..BASE_ADDON_CLASS };
leaf! {
    /// `Server.Items.GiantWeb2`.
    GIANT_WEB2 = "Server.Items.GiantWeb2" : BASE_ADDON, enc;
    construct: Some(giant_web2_new),
    family: Some(&GIANT_WEB2_CLASS),
}

/// `GiantWeb3()`.
pub fn giant_web3_new(w: &mut World) -> ItemId {
    let id = addon::new_addon(w, &GIANT_WEB3);
    addon::add_components(
        w,
        id,
        &[
            (Comp::Plain(4290), 0, 0, 0),
            (Comp::Plain(4291), 1, -1, 0),
            (Comp::Plain(4292), 2, -2, 0),
            (Comp::Plain(4293), 3, -3, 0),
        ],
    );
    id
}
static GIANT_WEB3_CLASS: AddonClass = AddonClass { ..BASE_ADDON_CLASS };
leaf! {
    /// `Server.Items.GiantWeb3`.
    GIANT_WEB3 = "Server.Items.GiantWeb3" : BASE_ADDON, enc;
    construct: Some(giant_web3_new),
    family: Some(&GIANT_WEB3_CLASS),
}

/// `GiantWeb4()`.
pub fn giant_web4_new(w: &mut World) -> ItemId {
    let id = addon::new_addon(w, &GIANT_WEB4);
    addon::add_components(
        w,
        id,
        &[
            (Comp::Plain(4294), 3, -3, 0),
            (Comp::Plain(4295), 2, -2, 0),
            (Comp::Plain(4296), 1, -1, 0),
            (Comp::Plain(4297), 0, 0, 0),
        ],
    );
    id
}
static GIANT_WEB4_CLASS: AddonClass = AddonClass { ..BASE_ADDON_CLASS };
leaf! {
    /// `Server.Items.GiantWeb4`.
    GIANT_WEB4 = "Server.Items.GiantWeb4" : BASE_ADDON, enc;
    construct: Some(giant_web4_new),
    family: Some(&GIANT_WEB4_CLASS),
}

/// `GiantWeb5()`.
pub fn giant_web5_new(w: &mut World) -> ItemId {
    let id = addon::new_addon(w, &GIANT_WEB5);
    addon::add_components(
        w,
        id,
        &[
            (Comp::Plain(4298), 0, 0, 0),
            (Comp::Plain(4299), 1, -1, 0),
            (Comp::Plain(4300), 2, -2, 0),
            (Comp::Plain(4301), 3, -3, 0),
        ],
    );
    id
}
static GIANT_WEB5_CLASS: AddonClass = AddonClass { ..BASE_ADDON_CLASS };
leaf! {
    /// `Server.Items.GiantWeb5`.
    GIANT_WEB5 = "Server.Items.GiantWeb5" : BASE_ADDON, enc;
    construct: Some(giant_web5_new),
    family: Some(&GIANT_WEB5_CLASS),
}

/// `GiantWeb6()`.
pub fn giant_web6_new(w: &mut World) -> ItemId {
    let id = addon::new_addon(w, &GIANT_WEB6);
    addon::add_components(
        w,
        id,
        &[
            (Comp::Plain(4302), 3, -3, 0),
            (Comp::Plain(4303), 2, -2, 0),
            (Comp::Plain(4304), 1, -1, 0),
            (Comp::Plain(4305), 0, 0, 0),
        ],
    );
    id
}
static GIANT_WEB6_CLASS: AddonClass = AddonClass { ..BASE_ADDON_CLASS };
leaf! {
    /// `Server.Items.GiantWeb6`.
    GIANT_WEB6 = "Server.Items.GiantWeb6" : BASE_ADDON, enc;
    construct: Some(giant_web6_new),
    family: Some(&GIANT_WEB6_CLASS),
}

/// `GozaMatEastAddon(int hue = 0)`.
pub fn goza_mat_east_addon_new(w: &mut World, hue: i32) -> ItemId {
    let id = addon::new_addon(w, &GOZA_MAT_EAST_ADDON);
    addon::add_components(
        w,
        id,
        &[
            (Comp::Localized(10404, 1030688), 1, 0, 0),
            (Comp::Localized(10405, 1030688), 0, 0, 0),
        ],
    );
    item_hue(w, id, hue);
    id
}
static GOZA_MAT_EAST_ADDON_CLASS: AddonClass = AddonClass {
    deed: Some(goza_mat_east_deed_new),
    retain_deed_hue: true,
    ..BASE_ADDON_CLASS
};
leaf! {
    /// `Server.Items.GozaMatEastAddon`.
    GOZA_MAT_EAST_ADDON = "Server.Items.GozaMatEastAddon" : BASE_ADDON, v0;
    construct: Some(|w| goza_mat_east_addon_new(w, 0)),
    family: Some(&GOZA_MAT_EAST_ADDON_CLASS),
}

/// `GozaMatEastDeed()`.
pub fn goza_mat_east_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &GOZA_MAT_EAST_DEED)
}
static GOZA_MAT_EAST_DEED_CLASS: DeedClass = DeedClass {
    addon: |w, deed| goza_mat_east_addon_new(w, addon::deed_hue(w, deed)),
    select_direction: None,
};
leaf! {
    /// `Server.Items.GozaMatEastDeed`.
    GOZA_MAT_EAST_DEED = "Server.Items.GozaMatEastDeed" : BASE_ADDON_DEED, v0;
    construct: Some(goza_mat_east_deed_new),
    family: Some(&GOZA_MAT_EAST_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `GozaMatSouthAddon(int hue = 0)`.
pub fn goza_mat_south_addon_new(w: &mut World, hue: i32) -> ItemId {
    let id = addon::new_addon(w, &GOZA_MAT_SOUTH_ADDON);
    addon::add_components(
        w,
        id,
        &[
            (Comp::Localized(10406, 1030688), 0, 1, 0),
            (Comp::Localized(10407, 1030688), 0, 0, 0),
        ],
    );
    item_hue(w, id, hue);
    id
}
static GOZA_MAT_SOUTH_ADDON_CLASS: AddonClass = AddonClass {
    deed: Some(goza_mat_south_deed_new),
    retain_deed_hue: true,
    ..BASE_ADDON_CLASS
};
leaf! {
    /// `Server.Items.GozaMatSouthAddon`.
    GOZA_MAT_SOUTH_ADDON = "Server.Items.GozaMatSouthAddon" : BASE_ADDON, v0;
    construct: Some(|w| goza_mat_south_addon_new(w, 0)),
    family: Some(&GOZA_MAT_SOUTH_ADDON_CLASS),
}

/// `GozaMatSouthDeed()`.
pub fn goza_mat_south_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &GOZA_MAT_SOUTH_DEED)
}
static GOZA_MAT_SOUTH_DEED_CLASS: DeedClass = DeedClass {
    addon: |w, deed| goza_mat_south_addon_new(w, addon::deed_hue(w, deed)),
    select_direction: None,
};
leaf! {
    /// `Server.Items.GozaMatSouthDeed`.
    GOZA_MAT_SOUTH_DEED = "Server.Items.GozaMatSouthDeed" : BASE_ADDON_DEED, v0;
    construct: Some(goza_mat_south_deed_new),
    family: Some(&GOZA_MAT_SOUTH_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `SquareGozaMatEastAddon(int hue = 0)`.
pub fn square_goza_mat_east_addon_new(w: &mut World, hue: i32) -> ItemId {
    let id = addon::new_addon(w, &SQUARE_GOZA_MAT_EAST_ADDON);
    addon::add_components(w, id, &[(Comp::Localized(10408, 1030688), 0, 0, 0)]);
    item_hue(w, id, hue);
    id
}
static SQUARE_GOZA_MAT_EAST_ADDON_CLASS: AddonClass = AddonClass {
    deed: Some(square_goza_mat_east_deed_new),
    retain_deed_hue: true,
    ..BASE_ADDON_CLASS
};
leaf! {
    /// `Server.Items.SquareGozaMatEastAddon`.
    SQUARE_GOZA_MAT_EAST_ADDON = "Server.Items.SquareGozaMatEastAddon" : BASE_ADDON, v0;
    construct: Some(|w| square_goza_mat_east_addon_new(w, 0)),
    family: Some(&SQUARE_GOZA_MAT_EAST_ADDON_CLASS),
    label_number: crate::class_props::label_number,
}

/// `SquareGozaMatEastDeed()`.
pub fn square_goza_mat_east_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &SQUARE_GOZA_MAT_EAST_DEED)
}
static SQUARE_GOZA_MAT_EAST_DEED_CLASS: DeedClass = DeedClass {
    addon: |w, deed| square_goza_mat_east_addon_new(w, addon::deed_hue(w, deed)),
    select_direction: None,
};
leaf! {
    /// `Server.Items.SquareGozaMatEastDeed`.
    SQUARE_GOZA_MAT_EAST_DEED = "Server.Items.SquareGozaMatEastDeed" : BASE_ADDON_DEED, v0;
    construct: Some(square_goza_mat_east_deed_new),
    family: Some(&SQUARE_GOZA_MAT_EAST_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `SquareGozaMatSouthAddon(int hue = 0)`.
pub fn square_goza_mat_south_addon_new(w: &mut World, hue: i32) -> ItemId {
    let id = addon::new_addon(w, &SQUARE_GOZA_MAT_SOUTH_ADDON);
    addon::add_components(w, id, &[(Comp::Localized(10409, 1030688), 0, 0, 0)]);
    item_hue(w, id, hue);
    id
}
static SQUARE_GOZA_MAT_SOUTH_ADDON_CLASS: AddonClass = AddonClass {
    deed: Some(square_goza_mat_south_deed_new),
    retain_deed_hue: true,
    ..BASE_ADDON_CLASS
};
leaf! {
    /// `Server.Items.SquareGozaMatSouthAddon`.
    SQUARE_GOZA_MAT_SOUTH_ADDON = "Server.Items.SquareGozaMatSouthAddon" : BASE_ADDON, v0;
    construct: Some(|w| square_goza_mat_south_addon_new(w, 0)),
    family: Some(&SQUARE_GOZA_MAT_SOUTH_ADDON_CLASS),
}

/// `SquareGozaMatSouthDeed()`.
pub fn square_goza_mat_south_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &SQUARE_GOZA_MAT_SOUTH_DEED)
}
static SQUARE_GOZA_MAT_SOUTH_DEED_CLASS: DeedClass = DeedClass {
    addon: |w, deed| square_goza_mat_south_addon_new(w, addon::deed_hue(w, deed)),
    select_direction: None,
};
leaf! {
    /// `Server.Items.SquareGozaMatSouthDeed`.
    SQUARE_GOZA_MAT_SOUTH_DEED = "Server.Items.SquareGozaMatSouthDeed" : BASE_ADDON_DEED, v0;
    construct: Some(square_goza_mat_south_deed_new),
    family: Some(&SQUARE_GOZA_MAT_SOUTH_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `BrocadeGozaMatEastAddon(int hue = 0)`.
pub fn brocade_goza_mat_east_addon_new(w: &mut World, hue: i32) -> ItemId {
    let id = addon::new_addon(w, &BROCADE_GOZA_MAT_EAST_ADDON);
    addon::add_components(
        w,
        id,
        &[
            (Comp::Localized(10411, 1030688), 0, 0, 0),
            (Comp::Localized(10410, 1030688), 1, 0, 0),
        ],
    );
    item_hue(w, id, hue);
    id
}
static BROCADE_GOZA_MAT_EAST_ADDON_CLASS: AddonClass = AddonClass {
    deed: Some(brocade_goza_mat_east_deed_new),
    retain_deed_hue: true,
    ..BASE_ADDON_CLASS
};
leaf! {
    /// `Server.Items.BrocadeGozaMatEastAddon`.
    BROCADE_GOZA_MAT_EAST_ADDON = "Server.Items.BrocadeGozaMatEastAddon" : BASE_ADDON, v0;
    construct: Some(|w| brocade_goza_mat_east_addon_new(w, 0)),
    family: Some(&BROCADE_GOZA_MAT_EAST_ADDON_CLASS),
}

/// `BrocadeGozaMatEastDeed()`.
pub fn brocade_goza_mat_east_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &BROCADE_GOZA_MAT_EAST_DEED)
}
static BROCADE_GOZA_MAT_EAST_DEED_CLASS: DeedClass = DeedClass {
    addon: |w, deed| brocade_goza_mat_east_addon_new(w, addon::deed_hue(w, deed)),
    select_direction: None,
};
leaf! {
    /// `Server.Items.BrocadeGozaMatEastDeed`.
    BROCADE_GOZA_MAT_EAST_DEED = "Server.Items.BrocadeGozaMatEastDeed" : BASE_ADDON_DEED, v0;
    construct: Some(brocade_goza_mat_east_deed_new),
    family: Some(&BROCADE_GOZA_MAT_EAST_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `BrocadeGozaMatSouthAddon(int hue = 0)`.
pub fn brocade_goza_mat_south_addon_new(w: &mut World, hue: i32) -> ItemId {
    let id = addon::new_addon(w, &BROCADE_GOZA_MAT_SOUTH_ADDON);
    addon::add_components(
        w,
        id,
        &[
            (Comp::Localized(10413, 1030688), 0, 0, 0),
            (Comp::Localized(10412, 1030688), 0, 1, 0),
        ],
    );
    item_hue(w, id, hue);
    id
}
static BROCADE_GOZA_MAT_SOUTH_ADDON_CLASS: AddonClass = AddonClass {
    deed: Some(brocade_goza_mat_south_deed_new),
    retain_deed_hue: true,
    ..BASE_ADDON_CLASS
};
leaf! {
    /// `Server.Items.BrocadeGozaMatSouthAddon`.
    BROCADE_GOZA_MAT_SOUTH_ADDON = "Server.Items.BrocadeGozaMatSouthAddon" : BASE_ADDON, v0;
    construct: Some(|w| brocade_goza_mat_south_addon_new(w, 0)),
    family: Some(&BROCADE_GOZA_MAT_SOUTH_ADDON_CLASS),
}

/// `BrocadeGozaMatSouthDeed()`.
pub fn brocade_goza_mat_south_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &BROCADE_GOZA_MAT_SOUTH_DEED)
}
static BROCADE_GOZA_MAT_SOUTH_DEED_CLASS: DeedClass = DeedClass {
    addon: |w, deed| brocade_goza_mat_south_addon_new(w, addon::deed_hue(w, deed)),
    select_direction: None,
};
leaf! {
    /// `Server.Items.BrocadeGozaMatSouthDeed`.
    BROCADE_GOZA_MAT_SOUTH_DEED = "Server.Items.BrocadeGozaMatSouthDeed" : BASE_ADDON_DEED, v0;
    construct: Some(brocade_goza_mat_south_deed_new),
    family: Some(&BROCADE_GOZA_MAT_SOUTH_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `BrocadeSquareGozaMatEastAddon(int hue = 0)`.
pub fn brocade_square_goza_mat_east_addon_new(w: &mut World, hue: i32) -> ItemId {
    let id = addon::new_addon(w, &BROCADE_SQUARE_GOZA_MAT_EAST_ADDON);
    addon::add_components(w, id, &[(Comp::Localized(10414, 1030688), 0, 0, 0)]);
    item_hue(w, id, hue);
    id
}
static BROCADE_SQUARE_GOZA_MAT_EAST_ADDON_CLASS: AddonClass = AddonClass {
    deed: Some(brocade_square_goza_mat_east_deed_new),
    retain_deed_hue: true,
    ..BASE_ADDON_CLASS
};
leaf! {
    /// `Server.Items.BrocadeSquareGozaMatEastAddon`.
    BROCADE_SQUARE_GOZA_MAT_EAST_ADDON = "Server.Items.BrocadeSquareGozaMatEastAddon" : BASE_ADDON, v0;
    construct: Some(|w| brocade_square_goza_mat_east_addon_new(w, 0)),
    family: Some(&BROCADE_SQUARE_GOZA_MAT_EAST_ADDON_CLASS),
}

/// `BrocadeSquareGozaMatEastDeed()`.
pub fn brocade_square_goza_mat_east_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &BROCADE_SQUARE_GOZA_MAT_EAST_DEED)
}
static BROCADE_SQUARE_GOZA_MAT_EAST_DEED_CLASS: DeedClass = DeedClass {
    addon: |w, deed| brocade_square_goza_mat_east_addon_new(w, addon::deed_hue(w, deed)),
    select_direction: None,
};
leaf! {
    /// `Server.Items.BrocadeSquareGozaMatEastDeed`.
    BROCADE_SQUARE_GOZA_MAT_EAST_DEED = "Server.Items.BrocadeSquareGozaMatEastDeed" : BASE_ADDON_DEED, v0;
    construct: Some(brocade_square_goza_mat_east_deed_new),
    family: Some(&BROCADE_SQUARE_GOZA_MAT_EAST_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `BrocadeSquareGozaMatSouthAddon(int hue = 0)`.
pub fn brocade_square_goza_mat_south_addon_new(w: &mut World, hue: i32) -> ItemId {
    let id = addon::new_addon(w, &BROCADE_SQUARE_GOZA_MAT_SOUTH_ADDON);
    addon::add_components(w, id, &[(Comp::Localized(10415, 1030688), 0, 0, 0)]);
    item_hue(w, id, hue);
    id
}
static BROCADE_SQUARE_GOZA_MAT_SOUTH_ADDON_CLASS: AddonClass = AddonClass {
    deed: Some(brocade_square_goza_mat_south_deed_new),
    retain_deed_hue: true,
    ..BASE_ADDON_CLASS
};
leaf! {
    /// `Server.Items.BrocadeSquareGozaMatSouthAddon`.
    BROCADE_SQUARE_GOZA_MAT_SOUTH_ADDON = "Server.Items.BrocadeSquareGozaMatSouthAddon" : BASE_ADDON, v0;
    construct: Some(|w| brocade_square_goza_mat_south_addon_new(w, 0)),
    family: Some(&BROCADE_SQUARE_GOZA_MAT_SOUTH_ADDON_CLASS),
}

/// `BrocadeSquareGozaMatSouthDeed()`.
pub fn brocade_square_goza_mat_south_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &BROCADE_SQUARE_GOZA_MAT_SOUTH_DEED)
}
static BROCADE_SQUARE_GOZA_MAT_SOUTH_DEED_CLASS: DeedClass = DeedClass {
    addon: |w, deed| brocade_square_goza_mat_south_addon_new(w, addon::deed_hue(w, deed)),
    select_direction: None,
};
leaf! {
    /// `Server.Items.BrocadeSquareGozaMatSouthDeed`.
    BROCADE_SQUARE_GOZA_MAT_SOUTH_DEED = "Server.Items.BrocadeSquareGozaMatSouthDeed" : BASE_ADDON_DEED, v0;
    construct: Some(brocade_square_goza_mat_south_deed_new),
    family: Some(&BROCADE_SQUARE_GOZA_MAT_SOUTH_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `GrayBrickFireplaceEastAddon()`.
pub fn gray_brick_fireplace_east_addon_new(w: &mut World) -> ItemId {
    let id = addon::new_addon(w, &GRAY_BRICK_FIREPLACE_EAST_ADDON);
    addon::add_components(
        w,
        id,
        &[(Comp::Plain(2365), 0, 0, 0), (Comp::Plain(2359), 0, 1, 0)],
    );
    id
}
static GRAY_BRICK_FIREPLACE_EAST_ADDON_CLASS: AddonClass = AddonClass {
    deed: Some(gray_brick_fireplace_east_deed_new),
    ..BASE_ADDON_CLASS
};
leaf! {
    /// `Server.Items.GrayBrickFireplaceEastAddon`.
    GRAY_BRICK_FIREPLACE_EAST_ADDON = "Server.Items.GrayBrickFireplaceEastAddon" : BASE_ADDON, v0;
    construct: Some(gray_brick_fireplace_east_addon_new),
    family: Some(&GRAY_BRICK_FIREPLACE_EAST_ADDON_CLASS),
}

/// `GrayBrickFireplaceEastDeed()`.
pub fn gray_brick_fireplace_east_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &GRAY_BRICK_FIREPLACE_EAST_DEED)
}
static GRAY_BRICK_FIREPLACE_EAST_DEED_CLASS: DeedClass = DeedClass {
    addon: |w, _deed| gray_brick_fireplace_east_addon_new(w),
    select_direction: None,
};
leaf! {
    /// `Server.Items.GrayBrickFireplaceEastDeed`.
    GRAY_BRICK_FIREPLACE_EAST_DEED = "Server.Items.GrayBrickFireplaceEastDeed" : BASE_ADDON_DEED, v0;
    construct: Some(gray_brick_fireplace_east_deed_new),
    family: Some(&GRAY_BRICK_FIREPLACE_EAST_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `GrayBrickFireplaceSouthAddon()`.
pub fn gray_brick_fireplace_south_addon_new(w: &mut World) -> ItemId {
    let id = addon::new_addon(w, &GRAY_BRICK_FIREPLACE_SOUTH_ADDON);
    addon::add_components(
        w,
        id,
        &[(Comp::Plain(2379), -1, 0, 0), (Comp::Plain(2373), 0, 0, 0)],
    );
    id
}
static GRAY_BRICK_FIREPLACE_SOUTH_ADDON_CLASS: AddonClass = AddonClass {
    deed: Some(gray_brick_fireplace_south_deed_new),
    ..BASE_ADDON_CLASS
};
leaf! {
    /// `Server.Items.GrayBrickFireplaceSouthAddon`.
    GRAY_BRICK_FIREPLACE_SOUTH_ADDON = "Server.Items.GrayBrickFireplaceSouthAddon" : BASE_ADDON, v0;
    construct: Some(gray_brick_fireplace_south_addon_new),
    family: Some(&GRAY_BRICK_FIREPLACE_SOUTH_ADDON_CLASS),
}

/// `GrayBrickFireplaceSouthDeed()`.
pub fn gray_brick_fireplace_south_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &GRAY_BRICK_FIREPLACE_SOUTH_DEED)
}
static GRAY_BRICK_FIREPLACE_SOUTH_DEED_CLASS: DeedClass = DeedClass {
    addon: |w, _deed| gray_brick_fireplace_south_addon_new(w),
    select_direction: None,
};
leaf! {
    /// `Server.Items.GrayBrickFireplaceSouthDeed`.
    GRAY_BRICK_FIREPLACE_SOUTH_DEED = "Server.Items.GrayBrickFireplaceSouthDeed" : BASE_ADDON_DEED, v0;
    construct: Some(gray_brick_fireplace_south_deed_new),
    family: Some(&GRAY_BRICK_FIREPLACE_SOUTH_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `LargeBedEastAddon()`.
pub fn large_bed_east_addon_new(w: &mut World) -> ItemId {
    let id = addon::new_addon(w, &LARGE_BED_EAST_ADDON);
    addon::add_components(
        w,
        id,
        &[
            (Comp::Plain(2685), 0, 0, 0),
            (Comp::Plain(2684), 0, 1, 0),
            (Comp::Plain(2681), 1, 0, 0),
            (Comp::Plain(2680), 1, 1, 0),
        ],
    );
    id
}
static LARGE_BED_EAST_ADDON_CLASS: AddonClass = AddonClass {
    deed: Some(large_bed_east_deed_new),
    ..BASE_ADDON_CLASS
};
leaf! {
    /// `Server.Items.LargeBedEastAddon`.
    LARGE_BED_EAST_ADDON = "Server.Items.LargeBedEastAddon" : BASE_ADDON, v0;
    construct: Some(large_bed_east_addon_new),
    family: Some(&LARGE_BED_EAST_ADDON_CLASS),
}

/// `LargeBedEastDeed()`.
pub fn large_bed_east_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &LARGE_BED_EAST_DEED)
}
static LARGE_BED_EAST_DEED_CLASS: DeedClass = DeedClass {
    addon: |w, _deed| large_bed_east_addon_new(w),
    select_direction: None,
};
leaf! {
    /// `Server.Items.LargeBedEastDeed`.
    LARGE_BED_EAST_DEED = "Server.Items.LargeBedEastDeed" : BASE_ADDON_DEED, v0;
    construct: Some(large_bed_east_deed_new),
    family: Some(&LARGE_BED_EAST_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `LargeBedSouthAddon()`.
pub fn large_bed_south_addon_new(w: &mut World) -> ItemId {
    let id = addon::new_addon(w, &LARGE_BED_SOUTH_ADDON);
    addon::add_components(
        w,
        id,
        &[
            (Comp::Plain(2691), 0, 0, 0),
            (Comp::Plain(2687), 0, 1, 0),
            (Comp::Plain(2690), 1, 0, 0),
            (Comp::Plain(2686), 1, 1, 0),
        ],
    );
    id
}
static LARGE_BED_SOUTH_ADDON_CLASS: AddonClass = AddonClass {
    deed: Some(large_bed_south_deed_new),
    ..BASE_ADDON_CLASS
};
leaf! {
    /// `Server.Items.LargeBedSouthAddon`.
    LARGE_BED_SOUTH_ADDON = "Server.Items.LargeBedSouthAddon" : BASE_ADDON, v0;
    construct: Some(large_bed_south_addon_new),
    family: Some(&LARGE_BED_SOUTH_ADDON_CLASS),
}

/// `LargeBedSouthDeed()`.
pub fn large_bed_south_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &LARGE_BED_SOUTH_DEED)
}
static LARGE_BED_SOUTH_DEED_CLASS: DeedClass = DeedClass {
    addon: |w, _deed| large_bed_south_addon_new(w),
    select_direction: None,
};
leaf! {
    /// `Server.Items.LargeBedSouthDeed`.
    LARGE_BED_SOUTH_DEED = "Server.Items.LargeBedSouthDeed" : BASE_ADDON_DEED, v0;
    construct: Some(large_bed_south_deed_new),
    family: Some(&LARGE_BED_SOUTH_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `LargeForgeEastAddon()`.
pub fn large_forge_east_addon_new(w: &mut World) -> ItemId {
    let id = addon::new_addon(w, &LARGE_FORGE_EAST_ADDON);
    addon::add_components(
        w,
        id,
        &[
            (Comp::Forge(6534), 0, 0, 0),
            (Comp::Forge(6538), 0, 1, 0),
            (Comp::Forge(6550), 0, 2, 0),
            (Comp::Forge(6546), 0, 3, 0),
        ],
    );
    id
}
static LARGE_FORGE_EAST_ADDON_CLASS: AddonClass = AddonClass {
    deed: Some(large_forge_east_deed_new),
    ..BASE_ADDON_CLASS
};
leaf! {
    /// `Server.Items.LargeForgeEastAddon`.
    LARGE_FORGE_EAST_ADDON = "Server.Items.LargeForgeEastAddon" : BASE_ADDON, v0;
    construct: Some(large_forge_east_addon_new),
    family: Some(&LARGE_FORGE_EAST_ADDON_CLASS),
}

/// `LargeForgeEastDeed()`.
pub fn large_forge_east_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &LARGE_FORGE_EAST_DEED)
}
static LARGE_FORGE_EAST_DEED_CLASS: DeedClass = DeedClass {
    addon: |w, _deed| large_forge_east_addon_new(w),
    select_direction: None,
};
leaf! {
    /// `Server.Items.LargeForgeEastDeed`.
    LARGE_FORGE_EAST_DEED = "Server.Items.LargeForgeEastDeed" : BASE_ADDON_DEED, v0;
    construct: Some(large_forge_east_deed_new),
    family: Some(&LARGE_FORGE_EAST_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `LargeForgeSouthAddon()`.
pub fn large_forge_south_addon_new(w: &mut World) -> ItemId {
    let id = addon::new_addon(w, &LARGE_FORGE_SOUTH_ADDON);
    addon::add_components(
        w,
        id,
        &[
            (Comp::Forge(6522), 0, 0, 0),
            (Comp::Forge(6526), 1, 0, 0),
            (Comp::Forge(6562), 2, 0, 0),
            (Comp::Forge(6558), 3, 0, 0),
        ],
    );
    id
}
static LARGE_FORGE_SOUTH_ADDON_CLASS: AddonClass = AddonClass {
    deed: Some(large_forge_south_deed_new),
    ..BASE_ADDON_CLASS
};
leaf! {
    /// `Server.Items.LargeForgeSouthAddon`.
    LARGE_FORGE_SOUTH_ADDON = "Server.Items.LargeForgeSouthAddon" : BASE_ADDON, v0;
    construct: Some(large_forge_south_addon_new),
    family: Some(&LARGE_FORGE_SOUTH_ADDON_CLASS),
}

/// `LargeForgeSouthDeed()`.
pub fn large_forge_south_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &LARGE_FORGE_SOUTH_DEED)
}
static LARGE_FORGE_SOUTH_DEED_CLASS: DeedClass = DeedClass {
    addon: |w, _deed| large_forge_south_addon_new(w),
    select_direction: None,
};
leaf! {
    /// `Server.Items.LargeForgeSouthDeed`.
    LARGE_FORGE_SOUTH_DEED = "Server.Items.LargeForgeSouthDeed" : BASE_ADDON_DEED, v0;
    construct: Some(large_forge_south_deed_new),
    family: Some(&LARGE_FORGE_SOUTH_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `LargeStoneTableEastAddon(int hue = 0)`.
pub fn large_stone_table_east_addon_new(w: &mut World, hue: i32) -> ItemId {
    let id = addon::new_addon(w, &LARGE_STONE_TABLE_EAST_ADDON);
    addon::add_components(
        w,
        id,
        &[
            (Comp::Plain(4610), 0, 0, 0),
            (Comp::Plain(4611), 0, 1, 0),
            (Comp::Plain(4609), 0, 2, 0),
        ],
    );
    item_hue(w, id, hue);
    id
}
static LARGE_STONE_TABLE_EAST_ADDON_CLASS: AddonClass = AddonClass {
    deed: Some(large_stone_table_east_deed_new),
    retain_deed_hue: true,
    ..BASE_ADDON_CLASS
};
leaf! {
    /// `Server.Items.LargeStoneTableEastAddon`.
    LARGE_STONE_TABLE_EAST_ADDON = "Server.Items.LargeStoneTableEastAddon" : BASE_ADDON, v0;
    construct: Some(|w| large_stone_table_east_addon_new(w, 0)),
    family: Some(&LARGE_STONE_TABLE_EAST_ADDON_CLASS),
}

/// `LargeStoneTableEastDeed()`.
pub fn large_stone_table_east_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &LARGE_STONE_TABLE_EAST_DEED)
}
static LARGE_STONE_TABLE_EAST_DEED_CLASS: DeedClass = DeedClass {
    addon: |w, deed| large_stone_table_east_addon_new(w, addon::deed_hue(w, deed)),
    select_direction: None,
};
leaf! {
    /// `Server.Items.LargeStoneTableEastDeed`.
    LARGE_STONE_TABLE_EAST_DEED = "Server.Items.LargeStoneTableEastDeed" : BASE_ADDON_DEED, v0;
    construct: Some(large_stone_table_east_deed_new),
    family: Some(&LARGE_STONE_TABLE_EAST_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `LargeStoneTableSouthAddon(int hue = 0)`.
pub fn large_stone_table_south_addon_new(w: &mut World, hue: i32) -> ItemId {
    let id = addon::new_addon(w, &LARGE_STONE_TABLE_SOUTH_ADDON);
    addon::add_components(
        w,
        id,
        &[
            (Comp::Plain(4613), 0, 0, 0),
            (Comp::Plain(4614), 1, 0, 0),
            (Comp::Plain(4612), 2, 0, 0),
        ],
    );
    item_hue(w, id, hue);
    id
}
static LARGE_STONE_TABLE_SOUTH_ADDON_CLASS: AddonClass = AddonClass {
    deed: Some(large_stone_table_south_deed_new),
    retain_deed_hue: true,
    ..BASE_ADDON_CLASS
};
leaf! {
    /// `Server.Items.LargeStoneTableSouthAddon`.
    LARGE_STONE_TABLE_SOUTH_ADDON = "Server.Items.LargeStoneTableSouthAddon" : BASE_ADDON, v0;
    construct: Some(|w| large_stone_table_south_addon_new(w, 0)),
    family: Some(&LARGE_STONE_TABLE_SOUTH_ADDON_CLASS),
}

/// `LargeStoneTableSouthDeed()`.
pub fn large_stone_table_south_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &LARGE_STONE_TABLE_SOUTH_DEED)
}
static LARGE_STONE_TABLE_SOUTH_DEED_CLASS: DeedClass = DeedClass {
    addon: |w, deed| large_stone_table_south_addon_new(w, addon::deed_hue(w, deed)),
    select_direction: None,
};
leaf! {
    /// `Server.Items.LargeStoneTableSouthDeed`.
    LARGE_STONE_TABLE_SOUTH_DEED = "Server.Items.LargeStoneTableSouthDeed" : BASE_ADDON_DEED, v0;
    construct: Some(large_stone_table_south_deed_new),
    family: Some(&LARGE_STONE_TABLE_SOUTH_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `MediumStoneTableEastAddon(int hue = 0)`.
pub fn medium_stone_table_east_addon_new(w: &mut World, hue: i32) -> ItemId {
    let id = addon::new_addon(w, &MEDIUM_STONE_TABLE_EAST_ADDON);
    addon::add_components(
        w,
        id,
        &[(Comp::Plain(4610), 0, 0, 0), (Comp::Plain(4609), 0, 1, 0)],
    );
    item_hue(w, id, hue);
    id
}
static MEDIUM_STONE_TABLE_EAST_ADDON_CLASS: AddonClass = AddonClass {
    deed: Some(medium_stone_table_east_deed_new),
    retain_deed_hue: true,
    ..BASE_ADDON_CLASS
};
leaf! {
    /// `Server.Items.MediumStoneTableEastAddon`.
    MEDIUM_STONE_TABLE_EAST_ADDON = "Server.Items.MediumStoneTableEastAddon" : BASE_ADDON, v0;
    construct: Some(|w| medium_stone_table_east_addon_new(w, 0)),
    family: Some(&MEDIUM_STONE_TABLE_EAST_ADDON_CLASS),
}

/// `MediumStoneTableEastDeed()`.
pub fn medium_stone_table_east_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &MEDIUM_STONE_TABLE_EAST_DEED)
}
static MEDIUM_STONE_TABLE_EAST_DEED_CLASS: DeedClass = DeedClass {
    addon: |w, deed| medium_stone_table_east_addon_new(w, addon::deed_hue(w, deed)),
    select_direction: None,
};
leaf! {
    /// `Server.Items.MediumStoneTableEastDeed`.
    MEDIUM_STONE_TABLE_EAST_DEED = "Server.Items.MediumStoneTableEastDeed" : BASE_ADDON_DEED, v0;
    construct: Some(medium_stone_table_east_deed_new),
    family: Some(&MEDIUM_STONE_TABLE_EAST_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `MediumStoneTableSouthAddon(int hue = 0)`.
pub fn medium_stone_table_south_addon_new(w: &mut World, hue: i32) -> ItemId {
    let id = addon::new_addon(w, &MEDIUM_STONE_TABLE_SOUTH_ADDON);
    addon::add_components(
        w,
        id,
        &[(Comp::Plain(4613), 0, 0, 0), (Comp::Plain(4612), 1, 0, 0)],
    );
    item_hue(w, id, hue);
    id
}
static MEDIUM_STONE_TABLE_SOUTH_ADDON_CLASS: AddonClass = AddonClass {
    deed: Some(medium_stone_table_south_deed_new),
    retain_deed_hue: true,
    ..BASE_ADDON_CLASS
};
leaf! {
    /// `Server.Items.MediumStoneTableSouthAddon`.
    MEDIUM_STONE_TABLE_SOUTH_ADDON = "Server.Items.MediumStoneTableSouthAddon" : BASE_ADDON, v0;
    construct: Some(|w| medium_stone_table_south_addon_new(w, 0)),
    family: Some(&MEDIUM_STONE_TABLE_SOUTH_ADDON_CLASS),
}

/// `MediumStoneTableSouthDeed()`.
pub fn medium_stone_table_south_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &MEDIUM_STONE_TABLE_SOUTH_DEED)
}
static MEDIUM_STONE_TABLE_SOUTH_DEED_CLASS: DeedClass = DeedClass {
    addon: |w, deed| medium_stone_table_south_addon_new(w, addon::deed_hue(w, deed)),
    select_direction: None,
};
leaf! {
    /// `Server.Items.MediumStoneTableSouthDeed`.
    MEDIUM_STONE_TABLE_SOUTH_DEED = "Server.Items.MediumStoneTableSouthDeed" : BASE_ADDON_DEED, v0;
    construct: Some(medium_stone_table_south_deed_new),
    family: Some(&MEDIUM_STONE_TABLE_SOUTH_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `OrnateElvenChestEastAddon()`.
pub fn ornate_elven_chest_east_addon_new(w: &mut World) -> ItemId {
    let id = addon_container::new_addon_container(w, &ORNATE_ELVEN_CHEST_EAST_ADDON, 0x309A);
    addon_container::add_components(w, id, &[(CComp::Plain(12443), 0, -1, 0)]);
    id
}
static ORNATE_ELVEN_CHEST_EAST_ADDON_CLASS: ContainerAddonClass = ContainerAddonClass {
    deed: Some(ornate_elven_chest_east_deed_new),
    retain_deed_hue: true,
    ..BASE_ADDON_CONTAINER_CLASS
};
leaf! {
    /// `Server.Items.OrnateElvenChestEastAddon`.
    ORNATE_ELVEN_CHEST_EAST_ADDON = "Server.Items.OrnateElvenChestEastAddon" : BASE_ADDON_CONTAINER, enc;
    construct: Some(ornate_elven_chest_east_addon_new),
    family: Some(&ORNATE_ELVEN_CHEST_EAST_ADDON_CLASS),
    default_gump_id: crate::class_props::default_gump_id,
    default_drop_sound: crate::class_props::default_drop_sound,
}

/// `OrnateElvenChestEastDeed()`.
pub fn ornate_elven_chest_east_deed_new(w: &mut World) -> ItemId {
    addon_container::new_deed(w, &ORNATE_ELVEN_CHEST_EAST_DEED)
}
static ORNATE_ELVEN_CHEST_EAST_DEED_CLASS: ContainerDeedClass = ContainerDeedClass {
    addon: |w, _deed| ornate_elven_chest_east_addon_new(w),
    select_direction: None,
};
leaf! {
    /// `Server.Items.OrnateElvenChestEastDeed`.
    ORNATE_ELVEN_CHEST_EAST_DEED = "Server.Items.OrnateElvenChestEastDeed" : BASE_ADDON_CONTAINER_DEED, enc;
    construct: Some(ornate_elven_chest_east_deed_new),
    family: Some(&ORNATE_ELVEN_CHEST_EAST_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `OrnateElvenChestSouthAddon()`.
pub fn ornate_elven_chest_south_addon_new(w: &mut World) -> ItemId {
    let id = addon_container::new_addon_container(w, &ORNATE_ELVEN_CHEST_SOUTH_ADDON, 0x3098);
    addon_container::add_components(w, id, &[(CComp::Plain(12441), -1, 0, 0)]);
    id
}
static ORNATE_ELVEN_CHEST_SOUTH_ADDON_CLASS: ContainerAddonClass = ContainerAddonClass {
    deed: Some(ornate_elven_chest_south_deed_new),
    retain_deed_hue: true,
    ..BASE_ADDON_CONTAINER_CLASS
};
leaf! {
    /// `Server.Items.OrnateElvenChestSouthAddon`.
    ORNATE_ELVEN_CHEST_SOUTH_ADDON = "Server.Items.OrnateElvenChestSouthAddon" : BASE_ADDON_CONTAINER, enc;
    construct: Some(ornate_elven_chest_south_addon_new),
    family: Some(&ORNATE_ELVEN_CHEST_SOUTH_ADDON_CLASS),
    default_gump_id: crate::class_props::default_gump_id,
    default_drop_sound: crate::class_props::default_drop_sound,
}

/// `OrnateElvenChestSouthDeed()`.
pub fn ornate_elven_chest_south_deed_new(w: &mut World) -> ItemId {
    addon_container::new_deed(w, &ORNATE_ELVEN_CHEST_SOUTH_DEED)
}
static ORNATE_ELVEN_CHEST_SOUTH_DEED_CLASS: ContainerDeedClass = ContainerDeedClass {
    addon: |w, _deed| ornate_elven_chest_south_addon_new(w),
    select_direction: None,
};
leaf! {
    /// `Server.Items.OrnateElvenChestSouthDeed`.
    ORNATE_ELVEN_CHEST_SOUTH_DEED = "Server.Items.OrnateElvenChestSouthDeed" : BASE_ADDON_CONTAINER_DEED, enc;
    construct: Some(ornate_elven_chest_south_deed_new),
    family: Some(&ORNATE_ELVEN_CHEST_SOUTH_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `OrnateElvenTableEastAddon()`.
pub fn ornate_elven_table_east_addon_new(w: &mut World) -> ItemId {
    let id = addon::new_addon(w, &ORNATE_ELVEN_TABLE_EAST_ADDON);
    addon::add_components(
        w,
        id,
        &[
            (Comp::Plain(12430), -1, 0, 0),
            (Comp::Plain(12429), 0, 0, 0),
            (Comp::Plain(12428), 1, 0, 0),
        ],
    );
    id
}
static ORNATE_ELVEN_TABLE_EAST_ADDON_CLASS: AddonClass = AddonClass {
    deed: Some(ornate_elven_table_east_deed_new),
    ..BASE_ADDON_CLASS
};
leaf! {
    /// `Server.Items.OrnateElvenTableEastAddon`.
    ORNATE_ELVEN_TABLE_EAST_ADDON = "Server.Items.OrnateElvenTableEastAddon" : BASE_ADDON, enc;
    construct: Some(ornate_elven_table_east_addon_new),
    family: Some(&ORNATE_ELVEN_TABLE_EAST_ADDON_CLASS),
}

/// `OrnateElvenTableEastDeed()`.
pub fn ornate_elven_table_east_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &ORNATE_ELVEN_TABLE_EAST_DEED)
}
static ORNATE_ELVEN_TABLE_EAST_DEED_CLASS: DeedClass = DeedClass {
    addon: |w, _deed| ornate_elven_table_east_addon_new(w),
    select_direction: None,
};
leaf! {
    /// `Server.Items.OrnateElvenTableEastDeed`.
    ORNATE_ELVEN_TABLE_EAST_DEED = "Server.Items.OrnateElvenTableEastDeed" : BASE_ADDON_DEED, enc;
    construct: Some(ornate_elven_table_east_deed_new),
    family: Some(&ORNATE_ELVEN_TABLE_EAST_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `OrnateElvenTableSouthAddon()`.
pub fn ornate_elven_table_south_addon_new(w: &mut World) -> ItemId {
    let id = addon::new_addon(w, &ORNATE_ELVEN_TABLE_SOUTH_ADDON);
    addon::add_components(
        w,
        id,
        &[
            (Comp::Plain(12431), 0, 1, 0),
            (Comp::Plain(12432), 0, 0, 0),
            (Comp::Plain(12433), 0, -1, 0),
        ],
    );
    id
}
static ORNATE_ELVEN_TABLE_SOUTH_ADDON_CLASS: AddonClass = AddonClass {
    deed: Some(ornate_elven_table_south_deed_new),
    ..BASE_ADDON_CLASS
};
leaf! {
    /// `Server.Items.OrnateElvenTableSouthAddon`.
    ORNATE_ELVEN_TABLE_SOUTH_ADDON = "Server.Items.OrnateElvenTableSouthAddon" : BASE_ADDON, enc;
    construct: Some(ornate_elven_table_south_addon_new),
    family: Some(&ORNATE_ELVEN_TABLE_SOUTH_ADDON_CLASS),
}

/// `OrnateElvenTableSouthDeed()`.
pub fn ornate_elven_table_south_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &ORNATE_ELVEN_TABLE_SOUTH_DEED)
}
static ORNATE_ELVEN_TABLE_SOUTH_DEED_CLASS: DeedClass = DeedClass {
    addon: |w, _deed| ornate_elven_table_south_addon_new(w),
    select_direction: None,
};
leaf! {
    /// `Server.Items.OrnateElvenTableSouthDeed`.
    ORNATE_ELVEN_TABLE_SOUTH_DEED = "Server.Items.OrnateElvenTableSouthDeed" : BASE_ADDON_DEED, enc;
    construct: Some(ornate_elven_table_south_deed_new),
    family: Some(&ORNATE_ELVEN_TABLE_SOUTH_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `ParrotPerchAddon()`.
pub fn parrot_perch_addon_new(w: &mut World) -> ItemId {
    let id = addon::new_addon(w, &PARROT_PERCH_ADDON);
    addon::add_components(w, id, &[(Comp::Plain(12276), 0, 0, 0)]);
    id
}
static PARROT_PERCH_ADDON_CLASS: AddonClass = AddonClass {
    deed: Some(parrot_perch_deed_new),
    ..BASE_ADDON_CLASS
};
leaf! {
    /// `Server.Items.ParrotPerchAddon`.
    PARROT_PERCH_ADDON = "Server.Items.ParrotPerchAddon" : BASE_ADDON, enc;
    construct: Some(parrot_perch_addon_new),
    family: Some(&PARROT_PERCH_ADDON_CLASS),
}

/// `ParrotPerchDeed()`.
pub fn parrot_perch_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &PARROT_PERCH_DEED)
}
static PARROT_PERCH_DEED_CLASS: DeedClass = DeedClass {
    addon: |w, _deed| parrot_perch_addon_new(w),
    select_direction: None,
};
leaf! {
    /// `Server.Items.ParrotPerchDeed`.
    PARROT_PERCH_DEED = "Server.Items.ParrotPerchDeed" : BASE_ADDON_DEED, enc;
    construct: Some(parrot_perch_deed_new),
    family: Some(&PARROT_PERCH_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `PentagramAddon()`.
pub fn pentagram_addon_new(w: &mut World) -> ItemId {
    let id = addon::new_addon(w, &PENTAGRAM_ADDON);
    addon::add_components(
        w,
        id,
        &[
            (Comp::Plain(4071), -1, -1, 0),
            (Comp::Plain(4072), 0, -1, 0),
            (Comp::Plain(4075), 1, -1, 0),
            (Comp::Plain(4070), -1, 0, 0),
            (Comp::Plain(4074), 0, 0, 0),
            (Comp::Plain(4078), 1, 0, 0),
            (Comp::Plain(4073), -1, 1, 0),
            (Comp::Plain(4076), 0, 1, 0),
            (Comp::Plain(4077), 1, 1, 0),
        ],
    );
    id
}
static PENTAGRAM_ADDON_CLASS: AddonClass = AddonClass {
    deed: Some(pentagram_deed_new),
    ..BASE_ADDON_CLASS
};
leaf! {
    /// `Server.Items.PentagramAddon`.
    PENTAGRAM_ADDON = "Server.Items.PentagramAddon" : BASE_ADDON, v0;
    construct: Some(pentagram_addon_new),
    family: Some(&PENTAGRAM_ADDON_CLASS),
}

/// `PentagramDeed()`.
pub fn pentagram_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &PENTAGRAM_DEED)
}
static PENTAGRAM_DEED_CLASS: DeedClass = DeedClass {
    addon: |w, _deed| pentagram_addon_new(w),
    select_direction: None,
};
leaf! {
    /// `Server.Items.PentagramDeed`.
    PENTAGRAM_DEED = "Server.Items.PentagramDeed" : BASE_ADDON_DEED, v0;
    construct: Some(pentagram_deed_new),
    family: Some(&PENTAGRAM_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `PyramidAddon()`.
pub fn pyramid_addon_new(w: &mut World) -> ItemId {
    let id = addon::new_addon(w, &PYRAMID_ADDON);
    addon::add_components(
        w,
        id,
        &[
            (Comp::Plain(1006), 0, 0, 5),
            (Comp::Plain(1011), -1, -1, 5),
            (Comp::Plain(1012), 1, 1, 5),
            (Comp::Plain(1013), 1, -1, 5),
            (Comp::Plain(1014), -1, 1, 5),
            (Comp::Plain(1011), -2, -2, 0),
            (Comp::Plain(1012), 2, 2, 0),
            (Comp::Plain(1013), 2, -2, 0),
            (Comp::Plain(1014), -2, 2, 0),
            (Comp::Plain(1007), -1, 2, 0),
            (Comp::Plain(1008), 2, -1, 0),
            (Comp::Plain(1009), -1, -2, 0),
            (Comp::Plain(1010), -2, -1, 0),
            (Comp::Plain(1007), 0, 2, 0),
            (Comp::Plain(1008), 2, 0, 0),
            (Comp::Plain(1009), 0, -2, 0),
            (Comp::Plain(1010), -2, 0, 0),
            (Comp::Plain(1007), 1, 2, 0),
            (Comp::Plain(1008), 2, 1, 0),
            (Comp::Plain(1009), 1, -2, 0),
            (Comp::Plain(1010), -2, 1, 0),
            (Comp::Plain(1007), 0, 1, 5),
            (Comp::Plain(1008), 1, 0, 5),
            (Comp::Plain(1009), 0, -1, 5),
            (Comp::Plain(1010), -1, 0, 5),
        ],
    );
    id
}
static PYRAMID_ADDON_CLASS: AddonClass = AddonClass {
    share_hue: false,
    ..BASE_ADDON_CLASS
};
leaf! {
    /// `Server.Items.PyramidAddon`.
    PYRAMID_ADDON = "Server.Items.PyramidAddon" : BASE_ADDON, enc;
    construct: Some(pyramid_addon_new),
    family: Some(&PYRAMID_ADDON_CLASS),
}

/// `SandstoneFireplaceEastAddon()`.
pub fn sandstone_fireplace_east_addon_new(w: &mut World) -> ItemId {
    let id = addon::new_addon(w, &SANDSTONE_FIREPLACE_EAST_ADDON);
    addon::add_components(
        w,
        id,
        &[(Comp::Plain(1161), 0, 0, 0), (Comp::Plain(1141), 0, 1, 0)],
    );
    id
}
static SANDSTONE_FIREPLACE_EAST_ADDON_CLASS: AddonClass = AddonClass {
    deed: Some(sandstone_fireplace_east_deed_new),
    ..BASE_ADDON_CLASS
};
leaf! {
    /// `Server.Items.SandstoneFireplaceEastAddon`.
    SANDSTONE_FIREPLACE_EAST_ADDON = "Server.Items.SandstoneFireplaceEastAddon" : BASE_ADDON, v0;
    construct: Some(sandstone_fireplace_east_addon_new),
    family: Some(&SANDSTONE_FIREPLACE_EAST_ADDON_CLASS),
}

/// `SandstoneFireplaceEastDeed()`.
pub fn sandstone_fireplace_east_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &SANDSTONE_FIREPLACE_EAST_DEED)
}
static SANDSTONE_FIREPLACE_EAST_DEED_CLASS: DeedClass = DeedClass {
    addon: |w, _deed| sandstone_fireplace_east_addon_new(w),
    select_direction: None,
};
leaf! {
    /// `Server.Items.SandstoneFireplaceEastDeed`.
    SANDSTONE_FIREPLACE_EAST_DEED = "Server.Items.SandstoneFireplaceEastDeed" : BASE_ADDON_DEED, v0;
    construct: Some(sandstone_fireplace_east_deed_new),
    family: Some(&SANDSTONE_FIREPLACE_EAST_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `SandstoneFireplaceSouthAddon()`.
pub fn sandstone_fireplace_south_addon_new(w: &mut World) -> ItemId {
    let id = addon::new_addon(w, &SANDSTONE_FIREPLACE_SOUTH_ADDON);
    addon::add_components(
        w,
        id,
        &[(Comp::Plain(1154), -1, 0, 0), (Comp::Plain(1147), 0, 0, 0)],
    );
    id
}
static SANDSTONE_FIREPLACE_SOUTH_ADDON_CLASS: AddonClass = AddonClass {
    deed: Some(sandstone_fireplace_south_deed_new),
    ..BASE_ADDON_CLASS
};
leaf! {
    /// `Server.Items.SandstoneFireplaceSouthAddon`.
    SANDSTONE_FIREPLACE_SOUTH_ADDON = "Server.Items.SandstoneFireplaceSouthAddon" : BASE_ADDON, v0;
    construct: Some(sandstone_fireplace_south_addon_new),
    family: Some(&SANDSTONE_FIREPLACE_SOUTH_ADDON_CLASS),
}

/// `SandstoneFireplaceSouthDeed()`.
pub fn sandstone_fireplace_south_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &SANDSTONE_FIREPLACE_SOUTH_DEED)
}
static SANDSTONE_FIREPLACE_SOUTH_DEED_CLASS: DeedClass = DeedClass {
    addon: |w, _deed| sandstone_fireplace_south_addon_new(w),
    select_direction: None,
};
leaf! {
    /// `Server.Items.SandstoneFireplaceSouthDeed`.
    SANDSTONE_FIREPLACE_SOUTH_DEED = "Server.Items.SandstoneFireplaceSouthDeed" : BASE_ADDON_DEED, v0;
    construct: Some(sandstone_fireplace_south_deed_new),
    family: Some(&SANDSTONE_FIREPLACE_SOUTH_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `SandstoneFountainAddon()`.
pub fn sandstone_fountain_addon_new(w: &mut World) -> ItemId {
    let id = addon::new_addon(w, &SANDSTONE_FOUNTAIN_ADDON);
    addon::add_components(
        w,
        id,
        &[
            (Comp::Plain(6595), -2, 1, 0),
            (Comp::Plain(6596), -1, 1, 0),
            (Comp::Plain(6597), 0, 1, 0),
            (Comp::Plain(6598), 1, 1, 0),
            (Comp::Plain(6599), 1, 0, 0),
            (Comp::Plain(6600), 1, -1, 0),
            (Comp::Plain(6601), 1, -2, 0),
            (Comp::Plain(6602), 0, -2, 0),
            (Comp::Plain(6603), 0, -1, 0),
            (Comp::Plain(6604), 0, 0, 0),
            (Comp::Plain(6605), -1, 0, 0),
            (Comp::Plain(6606), -2, 0, 0),
            (Comp::Plain(6607), -2, -1, 0),
            (Comp::Plain(6608), -1, -1, 0),
            (Comp::Plain(6609), -1, -2, 0),
            (Comp::Plain(6611), -2, -2, 0),
        ],
    );
    id
}
static SANDSTONE_FOUNTAIN_ADDON_CLASS: AddonClass = AddonClass { ..BASE_ADDON_CLASS };
leaf! {
    /// `Server.Items.SandstoneFountainAddon`.
    SANDSTONE_FOUNTAIN_ADDON = "Server.Items.SandstoneFountainAddon" : BASE_ADDON, v0;
    construct: Some(sandstone_fountain_addon_new),
    family: Some(&SANDSTONE_FOUNTAIN_ADDON_CLASS),
}

/// `SerpentPillarAddon()`.
pub fn serpent_pillar_addon_new(w: &mut World) -> ItemId {
    let id = addon::new_addon(w, &SERPENT_PILLAR_ADDON);
    addon::add_components(
        w,
        id,
        &[
            (Comp::Plain(9020), -2, 1, 0),
            (Comp::Plain(9021), -2, 0, 0),
            (Comp::Plain(9022), -2, -1, 0),
            (Comp::Plain(9023), -2, -2, 0),
            (Comp::Plain(9024), -1, -2, 0),
            (Comp::Plain(9025), 0, -2, 0),
            (Comp::Plain(9026), 1, -2, 0),
            (Comp::Plain(9027), -1, 1, 0),
            (Comp::Plain(9028), -1, 0, 0),
            (Comp::Plain(9029), -1, -1, 0),
            (Comp::Plain(9030), 0, 1, 0),
            (Comp::Plain(9031), 0, 0, 0),
            (Comp::Plain(9032), 0, -1, 0),
            (Comp::Plain(9033), 1, 0, 0),
            (Comp::Plain(9034), 1, -1, 0),
        ],
    );
    id
}
static SERPENT_PILLAR_ADDON_CLASS: AddonClass = AddonClass { ..BASE_ADDON_CLASS };
leaf! {
    /// `Server.Items.SerpentPillarAddon`.
    SERPENT_PILLAR_ADDON = "Server.Items.SerpentPillarAddon" : BASE_ADDON, v0;
    construct: Some(serpent_pillar_addon_new),
    family: Some(&SERPENT_PILLAR_ADDON_CLASS),
}

/// `ShrineOfWisdomAddon()`.
pub fn shrine_of_wisdom_addon_new(w: &mut World) -> ItemId {
    let id = addon::new_addon(w, &SHRINE_OF_WISDOM_ADDON);
    addon::add_components(
        w,
        id,
        &[
            (Comp::Typed(shrine_of_wisdom_component_new, 5315), 0, 0, 0),
            (Comp::Typed(shrine_of_wisdom_component_new, 5318), 1, 0, 0),
            (Comp::Typed(shrine_of_wisdom_component_new, 5332), 0, 1, 0),
            (Comp::Typed(shrine_of_wisdom_component_new, 5333), 1, 1, 0),
        ],
    );
    item_hue(w, id, 1150);
    id
}
static SHRINE_OF_WISDOM_ADDON_CLASS: AddonClass = AddonClass { ..BASE_ADDON_CLASS };
leaf! {
    /// `Server.Items.ShrineOfWisdomAddon`.
    SHRINE_OF_WISDOM_ADDON = "Server.Items.ShrineOfWisdomAddon" : BASE_ADDON, v0;
    construct: Some(shrine_of_wisdom_addon_new),
    family: Some(&SHRINE_OF_WISDOM_ADDON_CLASS),
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
    let id = addon::new_addon(w, &SKULL_PILE_ADDON);
    addon::add_components(
        w,
        id,
        &[
            (Comp::Plain(6872), 1, 1, 0),
            (Comp::Plain(6873), 0, 1, 0),
            (Comp::Plain(6874), -1, 1, 0),
            (Comp::Plain(6875), 0, 0, 0),
            (Comp::Plain(6876), 1, 0, 0),
            (Comp::Plain(6877), 1, -1, 0),
            (Comp::Plain(6878), 2, -1, 0),
            (Comp::Plain(6879), 2, 0, 0),
        ],
    );
    id
}
static SKULL_PILE_ADDON_CLASS: AddonClass = AddonClass { ..BASE_ADDON_CLASS };
leaf! {
    /// `Server.Items.SkullPileAddon`.
    SKULL_PILE_ADDON = "Server.Items.SkullPileAddon" : BASE_ADDON, enc;
    construct: Some(skull_pile_addon_new),
    family: Some(&SKULL_PILE_ADDON_CLASS),
}

/// `SmallBedEastAddon()`.
pub fn small_bed_east_addon_new(w: &mut World) -> ItemId {
    let id = addon::new_addon(w, &SMALL_BED_EAST_ADDON);
    addon::add_components(
        w,
        id,
        &[(Comp::Plain(2653), 0, 0, 0), (Comp::Plain(2658), 1, 0, 0)],
    );
    id
}
static SMALL_BED_EAST_ADDON_CLASS: AddonClass = AddonClass {
    deed: Some(small_bed_east_deed_new),
    ..BASE_ADDON_CLASS
};
leaf! {
    /// `Server.Items.SmallBedEastAddon`.
    SMALL_BED_EAST_ADDON = "Server.Items.SmallBedEastAddon" : BASE_ADDON, v0;
    construct: Some(small_bed_east_addon_new),
    family: Some(&SMALL_BED_EAST_ADDON_CLASS),
}

/// `SmallBedEastDeed()`.
pub fn small_bed_east_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &SMALL_BED_EAST_DEED)
}
static SMALL_BED_EAST_DEED_CLASS: DeedClass = DeedClass {
    addon: |w, _deed| small_bed_east_addon_new(w),
    select_direction: None,
};
leaf! {
    /// `Server.Items.SmallBedEastDeed`.
    SMALL_BED_EAST_DEED = "Server.Items.SmallBedEastDeed" : BASE_ADDON_DEED, v0;
    construct: Some(small_bed_east_deed_new),
    family: Some(&SMALL_BED_EAST_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `SmallBedSouthAddon()`.
pub fn small_bed_south_addon_new(w: &mut World) -> ItemId {
    let id = addon::new_addon(w, &SMALL_BED_SOUTH_ADDON);
    addon::add_components(
        w,
        id,
        &[(Comp::Plain(2659), 0, 0, 0), (Comp::Plain(2652), 0, 1, 0)],
    );
    id
}
static SMALL_BED_SOUTH_ADDON_CLASS: AddonClass = AddonClass {
    deed: Some(small_bed_south_deed_new),
    ..BASE_ADDON_CLASS
};
leaf! {
    /// `Server.Items.SmallBedSouthAddon`.
    SMALL_BED_SOUTH_ADDON = "Server.Items.SmallBedSouthAddon" : BASE_ADDON, v0;
    construct: Some(small_bed_south_addon_new),
    family: Some(&SMALL_BED_SOUTH_ADDON_CLASS),
}

/// `SmallBedSouthDeed()`.
pub fn small_bed_south_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &SMALL_BED_SOUTH_DEED)
}
static SMALL_BED_SOUTH_DEED_CLASS: DeedClass = DeedClass {
    addon: |w, _deed| small_bed_south_addon_new(w),
    select_direction: None,
};
leaf! {
    /// `Server.Items.SmallBedSouthDeed`.
    SMALL_BED_SOUTH_DEED = "Server.Items.SmallBedSouthDeed" : BASE_ADDON_DEED, v0;
    construct: Some(small_bed_south_deed_new),
    family: Some(&SMALL_BED_SOUTH_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `SmallForgeAddon()`.
pub fn small_forge_addon_new(w: &mut World) -> ItemId {
    let id = addon::new_addon(w, &SMALL_FORGE_ADDON);
    addon::add_components(w, id, &[(Comp::Forge(4017), 0, 0, 0)]);
    id
}
static SMALL_FORGE_ADDON_CLASS: AddonClass = AddonClass {
    deed: Some(small_forge_deed_new),
    ..BASE_ADDON_CLASS
};
leaf! {
    /// `Server.Items.SmallForgeAddon`.
    SMALL_FORGE_ADDON = "Server.Items.SmallForgeAddon" : BASE_ADDON, v0;
    construct: Some(small_forge_addon_new),
    family: Some(&SMALL_FORGE_ADDON_CLASS),
}

/// `SmallForgeDeed()`.
pub fn small_forge_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &SMALL_FORGE_DEED)
}
static SMALL_FORGE_DEED_CLASS: DeedClass = DeedClass {
    addon: |w, _deed| small_forge_addon_new(w),
    select_direction: None,
};
leaf! {
    /// `Server.Items.SmallForgeDeed`.
    SMALL_FORGE_DEED = "Server.Items.SmallForgeDeed" : BASE_ADDON_DEED, v0;
    construct: Some(small_forge_deed_new),
    family: Some(&SMALL_FORGE_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `SquirrelStatueEastAddon()`.
pub fn squirrel_statue_east_addon_new(w: &mut World) -> ItemId {
    let id = addon::new_addon(w, &SQUIRREL_STATUE_EAST_ADDON);
    addon::add_components(w, id, &[(Comp::Plain(11536), 0, 0, 0)]);
    id
}
static SQUIRREL_STATUE_EAST_ADDON_CLASS: AddonClass = AddonClass {
    deed: Some(squirrel_statue_east_deed_new),
    ..BASE_ADDON_CLASS
};
leaf! {
    /// `Server.Items.SquirrelStatueEastAddon`.
    SQUIRREL_STATUE_EAST_ADDON = "Server.Items.SquirrelStatueEastAddon" : BASE_ADDON, enc;
    construct: Some(squirrel_statue_east_addon_new),
    family: Some(&SQUIRREL_STATUE_EAST_ADDON_CLASS),
}

/// `SquirrelStatueEastDeed()`.
pub fn squirrel_statue_east_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &SQUIRREL_STATUE_EAST_DEED)
}
static SQUIRREL_STATUE_EAST_DEED_CLASS: DeedClass = DeedClass {
    addon: |w, _deed| squirrel_statue_east_addon_new(w),
    select_direction: None,
};
leaf! {
    /// `Server.Items.SquirrelStatueEastDeed`.
    SQUIRREL_STATUE_EAST_DEED = "Server.Items.SquirrelStatueEastDeed" : BASE_ADDON_DEED, enc;
    construct: Some(squirrel_statue_east_deed_new),
    family: Some(&SQUIRREL_STATUE_EAST_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `SquirrelStatueSouthAddon()`.
pub fn squirrel_statue_south_addon_new(w: &mut World) -> ItemId {
    let id = addon::new_addon(w, &SQUIRREL_STATUE_SOUTH_ADDON);
    addon::add_components(w, id, &[(Comp::Plain(11537), 0, 0, 0)]);
    id
}
static SQUIRREL_STATUE_SOUTH_ADDON_CLASS: AddonClass = AddonClass {
    deed: Some(squirrel_statue_south_deed_new),
    ..BASE_ADDON_CLASS
};
leaf! {
    /// `Server.Items.SquirrelStatueSouthAddon`.
    SQUIRREL_STATUE_SOUTH_ADDON = "Server.Items.SquirrelStatueSouthAddon" : BASE_ADDON, enc;
    construct: Some(squirrel_statue_south_addon_new),
    family: Some(&SQUIRREL_STATUE_SOUTH_ADDON_CLASS),
}

/// `SquirrelStatueSouthDeed()`.
pub fn squirrel_statue_south_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &SQUIRREL_STATUE_SOUTH_DEED)
}
static SQUIRREL_STATUE_SOUTH_DEED_CLASS: DeedClass = DeedClass {
    addon: |w, _deed| squirrel_statue_south_addon_new(w),
    select_direction: None,
};
leaf! {
    /// `Server.Items.SquirrelStatueSouthDeed`.
    SQUIRREL_STATUE_SOUTH_DEED = "Server.Items.SquirrelStatueSouthDeed" : BASE_ADDON_DEED, enc;
    construct: Some(squirrel_statue_south_deed_new),
    family: Some(&SQUIRREL_STATUE_SOUTH_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `StoneFireplaceEastAddon()`.
pub fn stone_fireplace_east_addon_new(w: &mut World) -> ItemId {
    let id = addon::new_addon(w, &STONE_FIREPLACE_EAST_ADDON);
    addon::add_components(
        w,
        id,
        &[(Comp::Plain(2393), 0, 0, 0), (Comp::Plain(2387), 0, 1, 0)],
    );
    id
}
static STONE_FIREPLACE_EAST_ADDON_CLASS: AddonClass = AddonClass {
    deed: Some(stone_fireplace_east_deed_new),
    ..BASE_ADDON_CLASS
};
leaf! {
    /// `Server.Items.StoneFireplaceEastAddon`.
    STONE_FIREPLACE_EAST_ADDON = "Server.Items.StoneFireplaceEastAddon" : BASE_ADDON, v0;
    construct: Some(stone_fireplace_east_addon_new),
    family: Some(&STONE_FIREPLACE_EAST_ADDON_CLASS),
}

/// `StoneFireplaceEastDeed()`.
pub fn stone_fireplace_east_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &STONE_FIREPLACE_EAST_DEED)
}
static STONE_FIREPLACE_EAST_DEED_CLASS: DeedClass = DeedClass {
    addon: |w, _deed| stone_fireplace_east_addon_new(w),
    select_direction: None,
};
leaf! {
    /// `Server.Items.StoneFireplaceEastDeed`.
    STONE_FIREPLACE_EAST_DEED = "Server.Items.StoneFireplaceEastDeed" : BASE_ADDON_DEED, v0;
    construct: Some(stone_fireplace_east_deed_new),
    family: Some(&STONE_FIREPLACE_EAST_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `StoneFireplaceSouthAddon()`.
pub fn stone_fireplace_south_addon_new(w: &mut World) -> ItemId {
    let id = addon::new_addon(w, &STONE_FIREPLACE_SOUTH_ADDON);
    addon::add_components(
        w,
        id,
        &[(Comp::Plain(2407), -1, 0, 0), (Comp::Plain(2401), 0, 0, 0)],
    );
    id
}
static STONE_FIREPLACE_SOUTH_ADDON_CLASS: AddonClass = AddonClass {
    deed: Some(stone_fireplace_south_deed_new),
    ..BASE_ADDON_CLASS
};
leaf! {
    /// `Server.Items.StoneFireplaceSouthAddon`.
    STONE_FIREPLACE_SOUTH_ADDON = "Server.Items.StoneFireplaceSouthAddon" : BASE_ADDON, v0;
    construct: Some(stone_fireplace_south_addon_new),
    family: Some(&STONE_FIREPLACE_SOUTH_ADDON_CLASS),
}

/// `StoneFireplaceSouthDeed()`.
pub fn stone_fireplace_south_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &STONE_FIREPLACE_SOUTH_DEED)
}
static STONE_FIREPLACE_SOUTH_DEED_CLASS: DeedClass = DeedClass {
    addon: |w, _deed| stone_fireplace_south_addon_new(w),
    select_direction: None,
};
leaf! {
    /// `Server.Items.StoneFireplaceSouthDeed`.
    STONE_FIREPLACE_SOUTH_DEED = "Server.Items.StoneFireplaceSouthDeed" : BASE_ADDON_DEED, v0;
    construct: Some(stone_fireplace_south_deed_new),
    family: Some(&STONE_FIREPLACE_SOUTH_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `StoneFountainAddon()`.
pub fn stone_fountain_addon_new(w: &mut World) -> ItemId {
    let id = addon::new_addon(w, &STONE_FOUNTAIN_ADDON);
    addon::add_components(
        w,
        id,
        &[
            (Comp::Plain(5937), -2, 1, 0),
            (Comp::Plain(5938), -1, 1, 0),
            (Comp::Plain(5939), 0, 1, 0),
            (Comp::Plain(5940), 1, 1, 0),
            (Comp::Plain(5941), 1, 0, 0),
            (Comp::Plain(5942), 1, -1, 0),
            (Comp::Plain(5943), 1, -2, 0),
            (Comp::Plain(5944), 0, -2, 0),
            (Comp::Plain(5945), 0, -1, 0),
            (Comp::Plain(5946), 0, 0, 0),
            (Comp::Plain(5947), -1, 0, 0),
            (Comp::Plain(5948), -2, 0, 0),
            (Comp::Plain(5949), -2, -1, 0),
            (Comp::Plain(5950), -1, -1, 0),
            (Comp::Plain(5951), -1, -2, 0),
            (Comp::Plain(5953), -2, -2, 0),
        ],
    );
    id
}
static STONE_FOUNTAIN_ADDON_CLASS: AddonClass = AddonClass { ..BASE_ADDON_CLASS };
leaf! {
    /// `Server.Items.StoneFountainAddon`.
    STONE_FOUNTAIN_ADDON = "Server.Items.StoneFountainAddon" : BASE_ADDON, v0;
    construct: Some(stone_fountain_addon_new),
    family: Some(&STONE_FOUNTAIN_ADDON_CLASS),
}

/// `StoneOvenEastAddon()`.
pub fn stone_oven_east_addon_new(w: &mut World) -> ItemId {
    let id = addon::new_addon(w, &STONE_OVEN_EAST_ADDON);
    addon::add_components(
        w,
        id,
        &[(Comp::Plain(2348), 0, 0, 0), (Comp::Plain(2347), 0, 1, 0)],
    );
    id
}
static STONE_OVEN_EAST_ADDON_CLASS: AddonClass = AddonClass {
    deed: Some(stone_oven_east_deed_new),
    ..BASE_ADDON_CLASS
};
leaf! {
    /// `Server.Items.StoneOvenEastAddon`.
    STONE_OVEN_EAST_ADDON = "Server.Items.StoneOvenEastAddon" : BASE_ADDON, v0;
    construct: Some(stone_oven_east_addon_new),
    family: Some(&STONE_OVEN_EAST_ADDON_CLASS),
}

/// `StoneOvenEastDeed()`.
pub fn stone_oven_east_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &STONE_OVEN_EAST_DEED)
}
static STONE_OVEN_EAST_DEED_CLASS: DeedClass = DeedClass {
    addon: |w, _deed| stone_oven_east_addon_new(w),
    select_direction: None,
};
leaf! {
    /// `Server.Items.StoneOvenEastDeed`.
    STONE_OVEN_EAST_DEED = "Server.Items.StoneOvenEastDeed" : BASE_ADDON_DEED, v0;
    construct: Some(stone_oven_east_deed_new),
    family: Some(&STONE_OVEN_EAST_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `StoneOvenSouthAddon()`.
pub fn stone_oven_south_addon_new(w: &mut World) -> ItemId {
    let id = addon::new_addon(w, &STONE_OVEN_SOUTH_ADDON);
    addon::add_components(
        w,
        id,
        &[(Comp::Plain(2353), -1, 0, 0), (Comp::Plain(2352), 0, 0, 0)],
    );
    id
}
static STONE_OVEN_SOUTH_ADDON_CLASS: AddonClass = AddonClass {
    deed: Some(stone_oven_south_deed_new),
    ..BASE_ADDON_CLASS
};
leaf! {
    /// `Server.Items.StoneOvenSouthAddon`.
    STONE_OVEN_SOUTH_ADDON = "Server.Items.StoneOvenSouthAddon" : BASE_ADDON, v0;
    construct: Some(stone_oven_south_addon_new),
    family: Some(&STONE_OVEN_SOUTH_ADDON_CLASS),
}

/// `StoneOvenSouthDeed()`.
pub fn stone_oven_south_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &STONE_OVEN_SOUTH_DEED)
}
static STONE_OVEN_SOUTH_DEED_CLASS: DeedClass = DeedClass {
    addon: |w, _deed| stone_oven_south_addon_new(w),
    select_direction: None,
};
leaf! {
    /// `Server.Items.StoneOvenSouthDeed`.
    STONE_OVEN_SOUTH_DEED = "Server.Items.StoneOvenSouthDeed" : BASE_ADDON_DEED, v0;
    construct: Some(stone_oven_south_deed_new),
    family: Some(&STONE_OVEN_SOUTH_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `SmallStretchedHideEastAddon()`.
pub fn small_stretched_hide_east_addon_new(w: &mut World) -> ItemId {
    let id = addon::new_addon(w, &SMALL_STRETCHED_HIDE_EAST_ADDON);
    addon::add_components(w, id, &[(Comp::Plain(4201), 0, 0, 0)]);
    id
}
static SMALL_STRETCHED_HIDE_EAST_ADDON_CLASS: AddonClass = AddonClass {
    deed: Some(small_stretched_hide_east_deed_new),
    ..BASE_ADDON_CLASS
};
leaf! {
    /// `Server.Items.SmallStretchedHideEastAddon`.
    SMALL_STRETCHED_HIDE_EAST_ADDON = "Server.Items.SmallStretchedHideEastAddon" : BASE_ADDON, v0;
    construct: Some(small_stretched_hide_east_addon_new),
    family: Some(&SMALL_STRETCHED_HIDE_EAST_ADDON_CLASS),
}

/// `SmallStretchedHideEastDeed()`.
pub fn small_stretched_hide_east_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &SMALL_STRETCHED_HIDE_EAST_DEED)
}
static SMALL_STRETCHED_HIDE_EAST_DEED_CLASS: DeedClass = DeedClass {
    addon: |w, _deed| small_stretched_hide_east_addon_new(w),
    select_direction: None,
};
leaf! {
    /// `Server.Items.SmallStretchedHideEastDeed`.
    SMALL_STRETCHED_HIDE_EAST_DEED = "Server.Items.SmallStretchedHideEastDeed" : BASE_ADDON_DEED, v0;
    construct: Some(small_stretched_hide_east_deed_new),
    family: Some(&SMALL_STRETCHED_HIDE_EAST_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `SmallStretchedHideSouthAddon()`.
pub fn small_stretched_hide_south_addon_new(w: &mut World) -> ItemId {
    let id = addon::new_addon(w, &SMALL_STRETCHED_HIDE_SOUTH_ADDON);
    addon::add_components(w, id, &[(Comp::Plain(4218), 0, 0, 0)]);
    id
}
static SMALL_STRETCHED_HIDE_SOUTH_ADDON_CLASS: AddonClass = AddonClass {
    deed: Some(small_stretched_hide_south_deed_new),
    ..BASE_ADDON_CLASS
};
leaf! {
    /// `Server.Items.SmallStretchedHideSouthAddon`.
    SMALL_STRETCHED_HIDE_SOUTH_ADDON = "Server.Items.SmallStretchedHideSouthAddon" : BASE_ADDON, v0;
    construct: Some(small_stretched_hide_south_addon_new),
    family: Some(&SMALL_STRETCHED_HIDE_SOUTH_ADDON_CLASS),
}

/// `SmallStretchedHideSouthDeed()`.
pub fn small_stretched_hide_south_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &SMALL_STRETCHED_HIDE_SOUTH_DEED)
}
static SMALL_STRETCHED_HIDE_SOUTH_DEED_CLASS: DeedClass = DeedClass {
    addon: |w, _deed| small_stretched_hide_south_addon_new(w),
    select_direction: None,
};
leaf! {
    /// `Server.Items.SmallStretchedHideSouthDeed`.
    SMALL_STRETCHED_HIDE_SOUTH_DEED = "Server.Items.SmallStretchedHideSouthDeed" : BASE_ADDON_DEED, v0;
    construct: Some(small_stretched_hide_south_deed_new),
    family: Some(&SMALL_STRETCHED_HIDE_SOUTH_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `MediumStretchedHideEastAddon()`.
pub fn medium_stretched_hide_east_addon_new(w: &mut World) -> ItemId {
    let id = addon::new_addon(w, &MEDIUM_STRETCHED_HIDE_EAST_ADDON);
    addon::add_components(w, id, &[(Comp::Plain(4203), 0, 0, 0)]);
    id
}
static MEDIUM_STRETCHED_HIDE_EAST_ADDON_CLASS: AddonClass = AddonClass {
    deed: Some(medium_stretched_hide_east_deed_new),
    ..BASE_ADDON_CLASS
};
leaf! {
    /// `Server.Items.MediumStretchedHideEastAddon`.
    MEDIUM_STRETCHED_HIDE_EAST_ADDON = "Server.Items.MediumStretchedHideEastAddon" : BASE_ADDON, v0;
    construct: Some(medium_stretched_hide_east_addon_new),
    family: Some(&MEDIUM_STRETCHED_HIDE_EAST_ADDON_CLASS),
}

/// `MediumStretchedHideEastDeed()`.
pub fn medium_stretched_hide_east_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &MEDIUM_STRETCHED_HIDE_EAST_DEED)
}
static MEDIUM_STRETCHED_HIDE_EAST_DEED_CLASS: DeedClass = DeedClass {
    addon: |w, _deed| medium_stretched_hide_east_addon_new(w),
    select_direction: None,
};
leaf! {
    /// `Server.Items.MediumStretchedHideEastDeed`.
    MEDIUM_STRETCHED_HIDE_EAST_DEED = "Server.Items.MediumStretchedHideEastDeed" : BASE_ADDON_DEED, v0;
    construct: Some(medium_stretched_hide_east_deed_new),
    family: Some(&MEDIUM_STRETCHED_HIDE_EAST_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `MediumStretchedHideSouthAddon()`.
pub fn medium_stretched_hide_south_addon_new(w: &mut World) -> ItemId {
    let id = addon::new_addon(w, &MEDIUM_STRETCHED_HIDE_SOUTH_ADDON);
    addon::add_components(w, id, &[(Comp::Plain(4220), 0, 0, 0)]);
    id
}
static MEDIUM_STRETCHED_HIDE_SOUTH_ADDON_CLASS: AddonClass = AddonClass {
    deed: Some(medium_stretched_hide_south_deed_new),
    ..BASE_ADDON_CLASS
};
leaf! {
    /// `Server.Items.MediumStretchedHideSouthAddon`.
    MEDIUM_STRETCHED_HIDE_SOUTH_ADDON = "Server.Items.MediumStretchedHideSouthAddon" : BASE_ADDON, v0;
    construct: Some(medium_stretched_hide_south_addon_new),
    family: Some(&MEDIUM_STRETCHED_HIDE_SOUTH_ADDON_CLASS),
}

/// `MediumStretchedHideSouthDeed()`.
pub fn medium_stretched_hide_south_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &MEDIUM_STRETCHED_HIDE_SOUTH_DEED)
}
static MEDIUM_STRETCHED_HIDE_SOUTH_DEED_CLASS: DeedClass = DeedClass {
    addon: |w, _deed| medium_stretched_hide_south_addon_new(w),
    select_direction: None,
};
leaf! {
    /// `Server.Items.MediumStretchedHideSouthDeed`.
    MEDIUM_STRETCHED_HIDE_SOUTH_DEED = "Server.Items.MediumStretchedHideSouthDeed" : BASE_ADDON_DEED, v0;
    construct: Some(medium_stretched_hide_south_deed_new),
    family: Some(&MEDIUM_STRETCHED_HIDE_SOUTH_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `TallElvenBedEastAddon()`.
pub fn tall_elven_bed_east_addon_new(w: &mut World) -> ItemId {
    let id = addon::new_addon(w, &TALL_ELVEN_BED_EAST_ADDON);
    addon::add_components(
        w,
        id,
        &[
            (Comp::Plain(12372), 0, 0, 0),
            (Comp::Plain(12371), 1, 0, 0),
            (Comp::Plain(12373), 2, -1, 0),
            (Comp::Plain(12370), 2, 0, 0),
        ],
    );
    id
}
static TALL_ELVEN_BED_EAST_ADDON_CLASS: AddonClass = AddonClass {
    deed: Some(tall_elven_bed_east_deed_new),
    ..BASE_ADDON_CLASS
};
leaf! {
    /// `Server.Items.TallElvenBedEastAddon`.
    TALL_ELVEN_BED_EAST_ADDON = "Server.Items.TallElvenBedEastAddon" : BASE_ADDON, enc;
    construct: Some(tall_elven_bed_east_addon_new),
    family: Some(&TALL_ELVEN_BED_EAST_ADDON_CLASS),
}

/// `TallElvenBedEastDeed()`.
pub fn tall_elven_bed_east_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &TALL_ELVEN_BED_EAST_DEED)
}
static TALL_ELVEN_BED_EAST_DEED_CLASS: DeedClass = DeedClass {
    addon: |w, _deed| tall_elven_bed_east_addon_new(w),
    select_direction: None,
};
leaf! {
    /// `Server.Items.TallElvenBedEastDeed`.
    TALL_ELVEN_BED_EAST_DEED = "Server.Items.TallElvenBedEastDeed" : BASE_ADDON_DEED, enc;
    construct: Some(tall_elven_bed_east_deed_new),
    family: Some(&TALL_ELVEN_BED_EAST_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `TallElvenBedSouthAddon()`.
pub fn tall_elven_bed_south_addon_new(w: &mut World) -> ItemId {
    let id = addon::new_addon(w, &TALL_ELVEN_BED_SOUTH_ADDON);
    addon::add_components(
        w,
        id,
        &[
            (Comp::Plain(12376), 0, 0, 0),
            (Comp::Plain(12375), -1, 1, 0),
            (Comp::Plain(12377), 0, -1, 0),
            (Comp::Plain(12374), 0, 1, 0),
        ],
    );
    id
}
static TALL_ELVEN_BED_SOUTH_ADDON_CLASS: AddonClass = AddonClass {
    deed: Some(tall_elven_bed_south_deed_new),
    ..BASE_ADDON_CLASS
};
leaf! {
    /// `Server.Items.TallElvenBedSouthAddon`.
    TALL_ELVEN_BED_SOUTH_ADDON = "Server.Items.TallElvenBedSouthAddon" : BASE_ADDON, enc;
    construct: Some(tall_elven_bed_south_addon_new),
    family: Some(&TALL_ELVEN_BED_SOUTH_ADDON_CLASS),
}

/// `TallElvenBedSouthDeed()`.
pub fn tall_elven_bed_south_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &TALL_ELVEN_BED_SOUTH_DEED)
}
static TALL_ELVEN_BED_SOUTH_DEED_CLASS: DeedClass = DeedClass {
    addon: |w, _deed| tall_elven_bed_south_addon_new(w),
    select_direction: None,
};
leaf! {
    /// `Server.Items.TallElvenBedSouthDeed`.
    TALL_ELVEN_BED_SOUTH_DEED = "Server.Items.TallElvenBedSouthDeed" : BASE_ADDON_DEED, enc;
    construct: Some(tall_elven_bed_south_deed_new),
    family: Some(&TALL_ELVEN_BED_SOUTH_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `Telescope()`.
pub fn telescope_new(w: &mut World) -> ItemId {
    let id = addon::new_addon(w, &TELESCOPE);
    addon::add_components(
        w,
        id,
        &[
            (Comp::Plain(5268), 0, 5, 0),
            (Comp::Plain(5211), 0, 6, 0),
            (Comp::Plain(5210), 0, 7, 0),
            (Comp::Plain(5269), 1, 4, 0),
            (Comp::Plain(5212), 1, 7, 0),
            (Comp::Plain(5213), 1, 8, 0),
            (Comp::Plain(5270), 2, 3, 0),
            (Comp::Plain(5273), 2, 4, 0),
            (Comp::Plain(5262), 2, 6, 0),
            (Comp::Plain(5267), 2, 7, 0),
            (Comp::Plain(5266), 2, 8, 0),
            (Comp::Plain(5214), 2, 9, 0),
            (Comp::Plain(5209), 2, 10, 0),
            (Comp::Plain(5271), 3, 2, 0),
            (Comp::Plain(5215), 3, 9, 0),
            (Comp::Plain(5217), 3, 10, 0),
            (Comp::Plain(5274), 4, 1, 0),
            (Comp::Plain(5272), 4, 2, 0),
            (Comp::Plain(5263), 4, 4, 0),
            (Comp::Plain(5261), 4, 6, 0),
            (Comp::Plain(5256), 4, 8, 0),
            (Comp::Plain(5216), 4, 9, 0),
            (Comp::Plain(5218), 4, 10, 0),
            (Comp::Plain(5245), 5, 0, 0),
            (Comp::Plain(5264), 5, 4, 0),
            (Comp::Plain(5259), 5, 5, 0),
            (Comp::Plain(5258), 5, 6, 0),
            (Comp::Plain(5254), 5, 7, 0),
            (Comp::Plain(5253), 5, 8, 0),
            (Comp::Plain(5244), 6, 0, 0),
            (Comp::Plain(5265), 6, 4, 0),
            (Comp::Plain(5260), 6, 5, 0),
            (Comp::Plain(5257), 6, 6, 0),
            (Comp::Plain(5255), 6, 7, 0),
            (Comp::Plain(5252), 6, 8, 0),
            (Comp::Plain(5219), 6, 10, 0),
            (Comp::Plain(5243), 7, 0, 0),
            (Comp::Plain(5247), 7, 3, 0),
            (Comp::Plain(5248), 7, 4, 0),
            (Comp::Plain(5250), 7, 5, 0),
            (Comp::Plain(5225), 7, 6, 0),
            (Comp::Plain(5224), 7, 7, 0),
            (Comp::Plain(5221), 7, 8, 0),
            (Comp::Plain(5220), 7, 9, 0),
            (Comp::Plain(5242), 8, 0, 0),
            (Comp::Plain(5241), 8, 1, 0),
            (Comp::Plain(5239), 8, 2, 0),
            (Comp::Plain(5246), 8, 3, 0),
            (Comp::Plain(5249), 8, 4, 0),
            (Comp::Plain(5251), 8, 5, 0),
            (Comp::Plain(5226), 8, 6, 0),
            (Comp::Plain(5223), 8, 7, 0),
            (Comp::Plain(5222), 8, 8, 0),
            (Comp::Plain(5240), 9, 1, 0),
            (Comp::Plain(5237), 9, 2, 0),
            (Comp::Plain(5236), 9, 3, 0),
            (Comp::Plain(5231), 9, 4, 0),
            (Comp::Plain(5230), 9, 5, 0),
            (Comp::Plain(5229), 9, 6, 0),
            (Comp::Plain(5227), 9, 7, 0),
            (Comp::Plain(5238), 10, 2, 0),
            (Comp::Plain(5235), 10, 3, 0),
            (Comp::Plain(5232), 10, 4, 0),
            (Comp::Plain(5233), 10, 5, 0),
            (Comp::Plain(5234), 10, 6, 0),
        ],
    );
    id
}
static TELESCOPE_CLASS: AddonClass = AddonClass { ..BASE_ADDON_CLASS };
leaf! {
    /// `Server.Items.Telescope`.
    TELESCOPE = "Server.Items.Telescope" : BASE_ADDON, v0;
    construct: Some(telescope_new),
    family: Some(&TELESCOPE_CLASS),
}

/// `WarriorStatueEastAddon()`.
pub fn warrior_statue_east_addon_new(w: &mut World) -> ItemId {
    let id = addon::new_addon(w, &WARRIOR_STATUE_EAST_ADDON);
    addon::add_components(w, id, &[(Comp::Plain(11538), 0, 0, 0)]);
    id
}
static WARRIOR_STATUE_EAST_ADDON_CLASS: AddonClass = AddonClass {
    deed: Some(warrior_statue_east_deed_new),
    ..BASE_ADDON_CLASS
};
leaf! {
    /// `Server.Items.WarriorStatueEastAddon`.
    WARRIOR_STATUE_EAST_ADDON = "Server.Items.WarriorStatueEastAddon" : BASE_ADDON, enc;
    construct: Some(warrior_statue_east_addon_new),
    family: Some(&WARRIOR_STATUE_EAST_ADDON_CLASS),
}

/// `WarriorStatueEastDeed()`.
pub fn warrior_statue_east_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &WARRIOR_STATUE_EAST_DEED)
}
static WARRIOR_STATUE_EAST_DEED_CLASS: DeedClass = DeedClass {
    addon: |w, _deed| warrior_statue_east_addon_new(w),
    select_direction: None,
};
leaf! {
    /// `Server.Items.WarriorStatueEastDeed`.
    WARRIOR_STATUE_EAST_DEED = "Server.Items.WarriorStatueEastDeed" : BASE_ADDON_DEED, enc;
    construct: Some(warrior_statue_east_deed_new),
    family: Some(&WARRIOR_STATUE_EAST_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `WarriorStatueSouthAddon()`.
pub fn warrior_statue_south_addon_new(w: &mut World) -> ItemId {
    let id = addon::new_addon(w, &WARRIOR_STATUE_SOUTH_ADDON);
    addon::add_components(w, id, &[(Comp::Plain(11539), 0, 0, 0)]);
    id
}
static WARRIOR_STATUE_SOUTH_ADDON_CLASS: AddonClass = AddonClass {
    deed: Some(warrior_statue_south_deed_new),
    ..BASE_ADDON_CLASS
};
leaf! {
    /// `Server.Items.WarriorStatueSouthAddon`.
    WARRIOR_STATUE_SOUTH_ADDON = "Server.Items.WarriorStatueSouthAddon" : BASE_ADDON, enc;
    construct: Some(warrior_statue_south_addon_new),
    family: Some(&WARRIOR_STATUE_SOUTH_ADDON_CLASS),
}

/// `WarriorStatueSouthDeed()`.
pub fn warrior_statue_south_deed_new(w: &mut World) -> ItemId {
    addon::new_deed(w, &WARRIOR_STATUE_SOUTH_DEED)
}
static WARRIOR_STATUE_SOUTH_DEED_CLASS: DeedClass = DeedClass {
    addon: |w, _deed| warrior_statue_south_addon_new(w),
    select_direction: None,
};
leaf! {
    /// `Server.Items.WarriorStatueSouthDeed`.
    WARRIOR_STATUE_SOUTH_DEED = "Server.Items.WarriorStatueSouthDeed" : BASE_ADDON_DEED, enc;
    construct: Some(warrior_statue_south_deed_new),
    family: Some(&WARRIOR_STATUE_SOUTH_DEED_CLASS),
    label_number: crate::class_props::label_number,
}

/// `WaterVatEast()`.
pub fn water_vat_east_new(w: &mut World) -> ItemId {
    let id = addon::new_addon(w, &WATER_VAT_EAST);
    addon::add_components(
        w,
        id,
        &[
            (Comp::Plain(5464), 0, 0, 0),
            (Comp::Plain(5342), -1, 1, 0),
            (Comp::Plain(5458), 0, 1, 0),
            (Comp::Plain(5343), 1, -1, 0),
            (Comp::Plain(5460), 1, 0, 0),
            (Comp::Plain(5465), 1, 1, 0),
            (Comp::Plain(5456), 1, 3, 0),
            (Comp::Plain(5461), 3, 1, 0),
            (Comp::Plain(5335), 2, 2, 0),
            (Comp::Plain(8612), 2, -1, 0),
            (Comp::Plain(8612), 3, 0, 0),
            (Comp::Plain(8612), -1, 2, 0),
            (Comp::Plain(8612), 0, 3, 0),
        ],
    );
    id
}
static WATER_VAT_EAST_CLASS: AddonClass = AddonClass { ..BASE_ADDON_CLASS };
leaf! {
    /// `Server.Items.WaterVatEast`.
    WATER_VAT_EAST = "Server.Items.WaterVatEast" : BASE_ADDON, enc;
    construct: Some(water_vat_east_new),
    family: Some(&WATER_VAT_EAST_CLASS),
}

/// `WaterVatSouth()`.
pub fn water_vat_south_new(w: &mut World) -> ItemId {
    let id = addon::new_addon(w, &WATER_VAT_SOUTH);
    addon::add_components(
        w,
        id,
        &[
            (Comp::Plain(5464), 0, 0, 0),
            (Comp::Plain(5342), -1, 1, 0),
            (Comp::Plain(5458), 0, 1, 0),
            (Comp::Plain(5343), 1, -1, 0),
            (Comp::Plain(5460), 1, 0, 0),
            (Comp::Plain(5465), 1, 1, 0),
            (Comp::Plain(5457), 1, 3, 0),
            (Comp::Plain(5462), 3, 1, 0),
            (Comp::Plain(5335), 2, 2, 0),
            (Comp::Plain(8612), 2, -1, 0),
            (Comp::Plain(8612), 3, 0, 0),
            (Comp::Plain(8612), -1, 2, 0),
            (Comp::Plain(8612), 0, 3, 0),
        ],
    );
    id
}
static WATER_VAT_SOUTH_CLASS: AddonClass = AddonClass { ..BASE_ADDON_CLASS };
leaf! {
    /// `Server.Items.WaterVatSouth`.
    WATER_VAT_SOUTH = "Server.Items.WaterVatSouth" : BASE_ADDON, enc;
    construct: Some(water_vat_south_new),
    family: Some(&WATER_VAT_SOUTH_CLASS),
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
