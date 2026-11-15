//! The Construction classes that are plain tables (a constructor and constant overrides); the classes with logic
//! are hand-written next to this file.

use crate::simple::{leaf, random_from};
use muo_server::World;
use muo_server::ids::ItemId;
use muo_server::item::{ITEM_BASE, ItemType, ops};
use muo_server::type_registry::TypeRegistry;

/// `WoodenBench()`.
pub fn wooden_bench_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.WoodenBench")
}

leaf! {
    /// `Server.Items.WoodenBench`.
    WOODEN_BENCH = "Server.Items.WoodenBench" : ITEM_BASE, v0;
    construct: Some(|w| wooden_bench_new(w, &WOODEN_BENCH)),
    default_weight: crate::class_props::default_weight,
}

/// `BambooChair()`.
pub fn bamboo_chair_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.BambooChair")
}

leaf! {
    /// `Server.Items.BambooChair`.
    BAMBOO_CHAIR = "Server.Items.BambooChair" : ITEM_BASE, v0;
    construct: Some(|w| bamboo_chair_new(w, &BAMBOO_CHAIR)),
    default_weight: crate::class_props::default_weight,
}

/// `BigElvenChair()`.
pub fn big_elven_chair_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.BigElvenChair")
}

leaf! {
    /// `Server.Items.BigElvenChair`.
    BIG_ELVEN_CHAIR = "Server.Items.BigElvenChair" : ITEM_BASE, enc;
    construct: Some(|w| big_elven_chair_new(w, &BIG_ELVEN_CHAIR)),
}

/// `ElvenReadingChair()`.
pub fn elven_reading_chair_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.ElvenReadingChair")
}

leaf! {
    /// `Server.Items.ElvenReadingChair`.
    ELVEN_READING_CHAIR = "Server.Items.ElvenReadingChair" : ITEM_BASE, enc;
    construct: Some(|w| elven_reading_chair_new(w, &ELVEN_READING_CHAIR)),
}

/// `FancyWoodenChairCushion()`.
pub fn fancy_wooden_chair_cushion_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.FancyWoodenChairCushion")
}

leaf! {
    /// `Server.Items.FancyWoodenChairCushion`.
    FANCY_WOODEN_CHAIR_CUSHION = "Server.Items.FancyWoodenChairCushion" : ITEM_BASE, v0;
    construct: Some(|w| fancy_wooden_chair_cushion_new(w, &FANCY_WOODEN_CHAIR_CUSHION)),
    default_weight: crate::class_props::default_weight,
}

/// `OrnateElvenChair()`.
pub fn ornate_elven_chair_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.OrnateElvenChair")
}

leaf! {
    /// `Server.Items.OrnateElvenChair`.
    ORNATE_ELVEN_CHAIR = "Server.Items.OrnateElvenChair" : ITEM_BASE, enc;
    construct: Some(|w| ornate_elven_chair_new(w, &ORNATE_ELVEN_CHAIR)),
    default_weight: crate::class_props::default_weight,
}

/// `StoneChair()`.
pub fn stone_chair_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.StoneChair")
}

leaf! {
    /// `Server.Items.StoneChair`.
    STONE_CHAIR = "Server.Items.StoneChair" : ITEM_BASE, v0;
    construct: Some(|w| stone_chair_new(w, &STONE_CHAIR)),
    default_weight: crate::class_props::default_weight,
}

/// `WoodenChair()`.
pub fn wooden_chair_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.WoodenChair")
}

leaf! {
    /// `Server.Items.WoodenChair`.
    WOODEN_CHAIR = "Server.Items.WoodenChair" : ITEM_BASE, v0;
    construct: Some(|w| wooden_chair_new(w, &WOODEN_CHAIR)),
    default_weight: crate::class_props::default_weight,
}

/// `WoodenChairCushion()`.
pub fn wooden_chair_cushion_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.WoodenChairCushion")
}

leaf! {
    /// `Server.Items.WoodenChairCushion`.
    WOODEN_CHAIR_CUSHION = "Server.Items.WoodenChairCushion" : ITEM_BASE, v0;
    construct: Some(|w| wooden_chair_cushion_new(w, &WOODEN_CHAIR_CUSHION)),
    default_weight: crate::class_props::default_weight,
}

/// `FootStool()`.
pub fn foot_stool_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.FootStool")
}

leaf! {
    /// `Server.Items.FootStool`.
    FOOT_STOOL = "Server.Items.FootStool" : ITEM_BASE, v0;
    construct: Some(|w| foot_stool_new(w, &FOOT_STOOL)),
    default_weight: crate::class_props::default_weight,
}

/// `Stool()`.
pub fn stool_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.Stool")
}

leaf! {
    /// `Server.Items.Stool`.
    STOOL = "Server.Items.Stool" : ITEM_BASE, v0;
    construct: Some(|w| stool_new(w, &STOOL)),
    default_weight: crate::class_props::default_weight,
}

/// `Throne()`.
pub fn throne_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.Throne")
}

leaf! {
    /// `Server.Items.Throne`.
    THRONE = "Server.Items.Throne" : ITEM_BASE, v0;
    construct: Some(|w| throne_new(w, &THRONE)),
    default_weight: crate::class_props::default_weight,
}

/// `WoodenThrone()`.
pub fn wooden_throne_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.WoodenThrone")
}

leaf! {
    /// `Server.Items.WoodenThrone`.
    WOODEN_THRONE = "Server.Items.WoodenThrone" : ITEM_BASE, v0;
    construct: Some(|w| wooden_throne_new(w, &WOODEN_THRONE)),
    default_weight: crate::class_props::default_weight,
}

/// `DecorativeShield1()`.
pub fn decorative_shield1_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.DecorativeShield1")
}

leaf! {
    /// `Server.Items.DecorativeShield1`.
    DECORATIVE_SHIELD1 = "Server.Items.DecorativeShield1" : ITEM_BASE, v0;
    construct: Some(|w| decorative_shield1_new(w, &DECORATIVE_SHIELD1)),
}

/// `DecorativeShield10()`.
pub fn decorative_shield10_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.DecorativeShield10")
}

leaf! {
    /// `Server.Items.DecorativeShield10`.
    DECORATIVE_SHIELD10 = "Server.Items.DecorativeShield10" : ITEM_BASE, v0;
    construct: Some(|w| decorative_shield10_new(w, &DECORATIVE_SHIELD10)),
}

/// `DecorativeShield11()`.
pub fn decorative_shield11_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.DecorativeShield11")
}

leaf! {
    /// `Server.Items.DecorativeShield11`.
    DECORATIVE_SHIELD11 = "Server.Items.DecorativeShield11" : ITEM_BASE, v0;
    construct: Some(|w| decorative_shield11_new(w, &DECORATIVE_SHIELD11)),
}

/// `DecorativeShield2()`.
pub fn decorative_shield2_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.DecorativeShield2")
}

leaf! {
    /// `Server.Items.DecorativeShield2`.
    DECORATIVE_SHIELD2 = "Server.Items.DecorativeShield2" : ITEM_BASE, v0;
    construct: Some(|w| decorative_shield2_new(w, &DECORATIVE_SHIELD2)),
}

/// `DecorativeShield3()`.
pub fn decorative_shield3_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.DecorativeShield3")
}

leaf! {
    /// `Server.Items.DecorativeShield3`.
    DECORATIVE_SHIELD3 = "Server.Items.DecorativeShield3" : ITEM_BASE, v0;
    construct: Some(|w| decorative_shield3_new(w, &DECORATIVE_SHIELD3)),
}

/// `DecorativeShield4()`.
pub fn decorative_shield4_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.DecorativeShield4")
}

leaf! {
    /// `Server.Items.DecorativeShield4`.
    DECORATIVE_SHIELD4 = "Server.Items.DecorativeShield4" : ITEM_BASE, v0;
    construct: Some(|w| decorative_shield4_new(w, &DECORATIVE_SHIELD4)),
}

/// `DecorativeShield5()`.
pub fn decorative_shield5_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.DecorativeShield5")
}

leaf! {
    /// `Server.Items.DecorativeShield5`.
    DECORATIVE_SHIELD5 = "Server.Items.DecorativeShield5" : ITEM_BASE, v0;
    construct: Some(|w| decorative_shield5_new(w, &DECORATIVE_SHIELD5)),
}

/// `DecorativeShield6()`.
pub fn decorative_shield6_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.DecorativeShield6")
}

leaf! {
    /// `Server.Items.DecorativeShield6`.
    DECORATIVE_SHIELD6 = "Server.Items.DecorativeShield6" : ITEM_BASE, v0;
    construct: Some(|w| decorative_shield6_new(w, &DECORATIVE_SHIELD6)),
}

/// `DecorativeShield7()`.
pub fn decorative_shield7_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.DecorativeShield7")
}

leaf! {
    /// `Server.Items.DecorativeShield7`.
    DECORATIVE_SHIELD7 = "Server.Items.DecorativeShield7" : ITEM_BASE, v0;
    construct: Some(|w| decorative_shield7_new(w, &DECORATIVE_SHIELD7)),
}

/// `DecorativeShield8()`.
pub fn decorative_shield8_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.DecorativeShield8")
}

leaf! {
    /// `Server.Items.DecorativeShield8`.
    DECORATIVE_SHIELD8 = "Server.Items.DecorativeShield8" : ITEM_BASE, v0;
    construct: Some(|w| decorative_shield8_new(w, &DECORATIVE_SHIELD8)),
}

/// `DecorativeShield9()`.
pub fn decorative_shield9_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.DecorativeShield9")
}

leaf! {
    /// `Server.Items.DecorativeShield9`.
    DECORATIVE_SHIELD9 = "Server.Items.DecorativeShield9" : ITEM_BASE, v0;
    construct: Some(|w| decorative_shield9_new(w, &DECORATIVE_SHIELD9)),
}

/// `DecorativeShieldSword1North()`.
pub fn decorative_shield_sword1_north_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.DecorativeShieldSword1North")
}

leaf! {
    /// `Server.Items.DecorativeShieldSword1North`.
    DECORATIVE_SHIELD_SWORD1_NORTH = "Server.Items.DecorativeShieldSword1North" : ITEM_BASE, v0;
    construct: Some(|w| decorative_shield_sword1_north_new(w, &DECORATIVE_SHIELD_SWORD1_NORTH)),
}

/// `DecorativeShieldSword1West()`.
pub fn decorative_shield_sword1_west_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.DecorativeShieldSword1West")
}

leaf! {
    /// `Server.Items.DecorativeShieldSword1West`.
    DECORATIVE_SHIELD_SWORD1_WEST = "Server.Items.DecorativeShieldSword1West" : ITEM_BASE, v0;
    construct: Some(|w| decorative_shield_sword1_west_new(w, &DECORATIVE_SHIELD_SWORD1_WEST)),
}

/// `DecorativeShieldSword2North()`.
pub fn decorative_shield_sword2_north_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.DecorativeShieldSword2North")
}

leaf! {
    /// `Server.Items.DecorativeShieldSword2North`.
    DECORATIVE_SHIELD_SWORD2_NORTH = "Server.Items.DecorativeShieldSword2North" : ITEM_BASE, v0;
    construct: Some(|w| decorative_shield_sword2_north_new(w, &DECORATIVE_SHIELD_SWORD2_NORTH)),
}

/// `DecorativeShieldSword2West()`.
pub fn decorative_shield_sword2_west_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.DecorativeShieldSword2West")
}

leaf! {
    /// `Server.Items.DecorativeShieldSword2West`.
    DECORATIVE_SHIELD_SWORD2_WEST = "Server.Items.DecorativeShieldSword2West" : ITEM_BASE, v0;
    construct: Some(|w| decorative_shield_sword2_west_new(w, &DECORATIVE_SHIELD_SWORD2_WEST)),
}

/// `DecorativeAxeNorth()`.
pub fn decorative_axe_north_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.DecorativeAxeNorth")
}

leaf! {
    /// `Server.Items.DecorativeAxeNorth`.
    DECORATIVE_AXE_NORTH = "Server.Items.DecorativeAxeNorth" : ITEM_BASE, v0;
    construct: Some(|w| decorative_axe_north_new(w, &DECORATIVE_AXE_NORTH)),
}

/// `DecorativeAxeWest()`.
pub fn decorative_axe_west_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.DecorativeAxeWest")
}

leaf! {
    /// `Server.Items.DecorativeAxeWest`.
    DECORATIVE_AXE_WEST = "Server.Items.DecorativeAxeWest" : ITEM_BASE, v0;
    construct: Some(|w| decorative_axe_west_new(w, &DECORATIVE_AXE_WEST)),
}

/// `DecorativeBowNorth()`.
pub fn decorative_bow_north_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.DecorativeBowNorth")
}

leaf! {
    /// `Server.Items.DecorativeBowNorth`.
    DECORATIVE_BOW_NORTH = "Server.Items.DecorativeBowNorth" : ITEM_BASE, v0;
    construct: Some(|w| decorative_bow_north_new(w, &DECORATIVE_BOW_NORTH)),
}

/// `DecorativeBowWest()`.
pub fn decorative_bow_west_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.DecorativeBowWest")
}

leaf! {
    /// `Server.Items.DecorativeBowWest`.
    DECORATIVE_BOW_WEST = "Server.Items.DecorativeBowWest" : ITEM_BASE, v0;
    construct: Some(|w| decorative_bow_west_new(w, &DECORATIVE_BOW_WEST)),
}

/// `GiantReplicaAcorn()`.
pub fn giant_replica_acorn_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.GiantReplicaAcorn")
}

leaf! {
    /// `Server.Items.GiantReplicaAcorn`.
    GIANT_REPLICA_ACORN = "Server.Items.GiantReplicaAcorn" : ITEM_BASE, enc;
    construct: Some(|w| giant_replica_acorn_new(w, &GIANT_REPLICA_ACORN)),
    default_weight: crate::class_props::default_weight,
    label_number: crate::class_props::label_number,
}

/// `MountedDreadHorn()`.
pub fn mounted_dread_horn_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.MountedDreadHorn")
}

leaf! {
    /// `Server.Items.MountedDreadHorn`.
    MOUNTED_DREAD_HORN = "Server.Items.MountedDreadHorn" : ITEM_BASE, enc;
    construct: Some(|w| mounted_dread_horn_new(w, &MOUNTED_DREAD_HORN)),
    default_weight: crate::class_props::default_weight,
    label_number: crate::class_props::label_number,
}

/// `LadyPortrait1()`.
pub fn lady_portrait1_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.LadyPortrait1")
}

leaf! {
    /// `Server.Items.LadyPortrait1`.
    LADY_PORTRAIT1 = "Server.Items.LadyPortrait1" : ITEM_BASE, v0;
    construct: Some(|w| lady_portrait1_new(w, &LADY_PORTRAIT1)),
}

/// `LadyPortrait2()`.
pub fn lady_portrait2_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.LadyPortrait2")
}

leaf! {
    /// `Server.Items.LadyPortrait2`.
    LADY_PORTRAIT2 = "Server.Items.LadyPortrait2" : ITEM_BASE, v0;
    construct: Some(|w| lady_portrait2_new(w, &LADY_PORTRAIT2)),
}

/// `LargePainting()`.
pub fn large_painting_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.LargePainting")
}

leaf! {
    /// `Server.Items.LargePainting`.
    LARGE_PAINTING = "Server.Items.LargePainting" : ITEM_BASE, v0;
    construct: Some(|w| large_painting_new(w, &LARGE_PAINTING)),
}

/// `ManPortrait1()`.
pub fn man_portrait1_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.ManPortrait1")
}

leaf! {
    /// `Server.Items.ManPortrait1`.
    MAN_PORTRAIT1 = "Server.Items.ManPortrait1" : ITEM_BASE, v0;
    construct: Some(|w| man_portrait1_new(w, &MAN_PORTRAIT1)),
}

/// `ManPortrait2()`.
pub fn man_portrait2_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.ManPortrait2")
}

leaf! {
    /// `Server.Items.ManPortrait2`.
    MAN_PORTRAIT2 = "Server.Items.ManPortrait2" : ITEM_BASE, v0;
    construct: Some(|w| man_portrait2_new(w, &MAN_PORTRAIT2)),
}

/// `WomanPortrait1()`.
pub fn woman_portrait1_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.WomanPortrait1")
}

leaf! {
    /// `Server.Items.WomanPortrait1`.
    WOMAN_PORTRAIT1 = "Server.Items.WomanPortrait1" : ITEM_BASE, v0;
    construct: Some(|w| woman_portrait1_new(w, &WOMAN_PORTRAIT1)),
}

/// `WomanPortrait2()`.
pub fn woman_portrait2_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.WomanPortrait2")
}

leaf! {
    /// `Server.Items.WomanPortrait2`.
    WOMAN_PORTRAIT2 = "Server.Items.WomanPortrait2" : ITEM_BASE, v0;
    construct: Some(|w| woman_portrait2_new(w, &WOMAN_PORTRAIT2)),
}

/// `BaseFloor(i32 item_id, i32 count)`.
pub fn base_floor_new(w: &mut World, ty: &'static ItemType, item_id: i32, count: i32) -> ItemId {
    let t0 = random_from(w, item_id, count);
    let id = ops::new_item_with_id(w, ty, t0);
    ops::set_movable(w, id, false);
    id
}

leaf! {
    /// `Server.Items.BaseFloor` (abstract).
    BASE_FLOOR = "Server.Items.BaseFloor" : ITEM_BASE, v0;
    construct: None,
}

/// `BlueSlateFloorCenter()`.
pub fn blue_slate_floor_center_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    base_floor_new(w, ty, 0x49B, 1)
}

leaf! {
    /// `Server.Items.BlueSlateFloorCenter`.
    BLUE_SLATE_FLOOR_CENTER = "Server.Items.BlueSlateFloorCenter" : BASE_FLOOR, v0;
    construct: Some(|w| blue_slate_floor_center_new(w, &BLUE_SLATE_FLOOR_CENTER)),
}

/// `BricksFloor1()`.
pub fn bricks_floor1_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    base_floor_new(w, ty, 0x4E2, 8)
}

leaf! {
    /// `Server.Items.BricksFloor1`.
    BRICKS_FLOOR1 = "Server.Items.BricksFloor1" : BASE_FLOOR, v0;
    construct: Some(|w| bricks_floor1_new(w, &BRICKS_FLOOR1)),
}

/// `BricksFloor2()`.
pub fn bricks_floor2_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    base_floor_new(w, ty, 0x537, 4)
}

leaf! {
    /// `Server.Items.BricksFloor2`.
    BRICKS_FLOOR2 = "Server.Items.BricksFloor2" : BASE_FLOOR, v0;
    construct: Some(|w| bricks_floor2_new(w, &BRICKS_FLOOR2)),
}

/// `CaveFloorCenter()`.
pub fn cave_floor_center_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    base_floor_new(w, ty, 0x53B, 4)
}

leaf! {
    /// `Server.Items.CaveFloorCenter`.
    CAVE_FLOOR_CENTER = "Server.Items.CaveFloorCenter" : BASE_FLOOR, v0;
    construct: Some(|w| cave_floor_center_new(w, &CAVE_FLOOR_CENTER)),
}

/// `CaveFloorEast()`.
pub fn cave_floor_east_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    base_floor_new(w, ty, 0x544, 3)
}

leaf! {
    /// `Server.Items.CaveFloorEast`.
    CAVE_FLOOR_EAST = "Server.Items.CaveFloorEast" : BASE_FLOOR, v0;
    construct: Some(|w| cave_floor_east_new(w, &CAVE_FLOOR_EAST)),
}

/// `CaveFloorNorth()`.
pub fn cave_floor_north_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    base_floor_new(w, ty, 0x54D, 3)
}

leaf! {
    /// `Server.Items.CaveFloorNorth`.
    CAVE_FLOOR_NORTH = "Server.Items.CaveFloorNorth" : BASE_FLOOR, v0;
    construct: Some(|w| cave_floor_north_new(w, &CAVE_FLOOR_NORTH)),
}

/// `CaveFloorSouth()`.
pub fn cave_floor_south_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    base_floor_new(w, ty, 0x541, 3)
}

leaf! {
    /// `Server.Items.CaveFloorSouth`.
    CAVE_FLOOR_SOUTH = "Server.Items.CaveFloorSouth" : BASE_FLOOR, v0;
    construct: Some(|w| cave_floor_south_new(w, &CAVE_FLOOR_SOUTH)),
}

/// `CaveFloorWest()`.
pub fn cave_floor_west_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    base_floor_new(w, ty, 0x54A, 3)
}

leaf! {
    /// `Server.Items.CaveFloorWest`.
    CAVE_FLOOR_WEST = "Server.Items.CaveFloorWest" : BASE_FLOOR, v0;
    construct: Some(|w| cave_floor_west_new(w, &CAVE_FLOOR_WEST)),
}

/// `CobblestonesFloor()`.
pub fn cobblestones_floor_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    base_floor_new(w, ty, 0x515, 4)
}

leaf! {
    /// `Server.Items.CobblestonesFloor`.
    COBBLESTONES_FLOOR = "Server.Items.CobblestonesFloor" : BASE_FLOOR, v0;
    construct: Some(|w| cobblestones_floor_new(w, &COBBLESTONES_FLOOR)),
}

/// `DarkSandstoneFloorN()`.
pub fn dark_sandstone_floor_n_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    base_floor_new(w, ty, 0x52F, 4)
}

leaf! {
    /// `Server.Items.DarkSandstoneFloorN`.
    DARK_SANDSTONE_FLOOR_N = "Server.Items.DarkSandstoneFloorN" : BASE_FLOOR, v0;
    construct: Some(|w| dark_sandstone_floor_n_new(w, &DARK_SANDSTONE_FLOOR_N)),
}

/// `DarkSandstoneFloorW()`.
pub fn dark_sandstone_floor_w_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    base_floor_new(w, ty, 0x533, 4)
}

leaf! {
    /// `Server.Items.DarkSandstoneFloorW`.
    DARK_SANDSTONE_FLOOR_W = "Server.Items.DarkSandstoneFloorW" : BASE_FLOOR, v0;
    construct: Some(|w| dark_sandstone_floor_w_new(w, &DARK_SANDSTONE_FLOOR_W)),
}

/// `GreenMarbleFloor()`.
pub fn green_marble_floor_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    base_floor_new(w, ty, 0x50F, 2)
}

leaf! {
    /// `Server.Items.GreenMarbleFloor`.
    GREEN_MARBLE_FLOOR = "Server.Items.GreenMarbleFloor" : BASE_FLOOR, v0;
    construct: Some(|w| green_marble_floor_new(w, &GREEN_MARBLE_FLOOR)),
}

/// `GreyFlagstones()`.
pub fn grey_flagstones_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    base_floor_new(w, ty, 0x4FC, 4)
}

leaf! {
    /// `Server.Items.GreyFlagstones`.
    GREY_FLAGSTONES = "Server.Items.GreyFlagstones" : BASE_FLOOR, v0;
    construct: Some(|w| grey_flagstones_new(w, &GREY_FLAGSTONES)),
}

/// `GreyMarbleFloor()`.
pub fn grey_marble_floor_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    base_floor_new(w, ty, 0x511, 4)
}

leaf! {
    /// `Server.Items.GreyMarbleFloor`.
    GREY_MARBLE_FLOOR = "Server.Items.GreyMarbleFloor" : BASE_FLOOR, v0;
    construct: Some(|w| grey_marble_floor_new(w, &GREY_MARBLE_FLOOR)),
}

/// `GreySlateFloor()`.
pub fn grey_slate_floor_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    base_floor_new(w, ty, 0x49C, 1)
}

leaf! {
    /// `Server.Items.GreySlateFloor`.
    GREY_SLATE_FLOOR = "Server.Items.GreySlateFloor" : BASE_FLOOR, v0;
    construct: Some(|w| grey_slate_floor_new(w, &GREY_SLATE_FLOOR)),
}

/// `MarbleFloor()`.
pub fn marble_floor_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    base_floor_new(w, ty, 0x50D, 2)
}

leaf! {
    /// `Server.Items.MarbleFloor`.
    MARBLE_FLOOR = "Server.Items.MarbleFloor" : BASE_FLOOR, v0;
    construct: Some(|w| marble_floor_new(w, &MARBLE_FLOOR)),
}

/// `MarblePavers()`.
pub fn marble_pavers_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    base_floor_new(w, ty, 0x495, 4)
}

leaf! {
    /// `Server.Items.MarblePavers`.
    MARBLE_PAVERS = "Server.Items.MarblePavers" : BASE_FLOOR, v0;
    construct: Some(|w| marble_pavers_new(w, &MARBLE_PAVERS)),
}

/// `SandFlagstones()`.
pub fn sand_flagstones_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    base_floor_new(w, ty, 0x500, 4)
}

leaf! {
    /// `Server.Items.SandFlagstones`.
    SAND_FLAGSTONES = "Server.Items.SandFlagstones" : BASE_FLOOR, v0;
    construct: Some(|w| sand_flagstones_new(w, &SAND_FLAGSTONES)),
}

/// `SandstoneFloorN()`.
pub fn sandstone_floor_n_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    base_floor_new(w, ty, 0x525, 4)
}

leaf! {
    /// `Server.Items.SandstoneFloorN`.
    SANDSTONE_FLOOR_N = "Server.Items.SandstoneFloorN" : BASE_FLOOR, v0;
    construct: Some(|w| sandstone_floor_n_new(w, &SANDSTONE_FLOOR_N)),
}

/// `SandstoneFloorW()`.
pub fn sandstone_floor_w_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    base_floor_new(w, ty, 0x529, 4)
}

leaf! {
    /// `Server.Items.SandstoneFloorW`.
    SANDSTONE_FLOOR_W = "Server.Items.SandstoneFloorW" : BASE_FLOOR, v0;
    construct: Some(|w| sandstone_floor_w_new(w, &SANDSTONE_FLOOR_W)),
}

/// `StonePaversDark()`.
pub fn stone_pavers_dark_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    base_floor_new(w, ty, 0x521, 4)
}

leaf! {
    /// `Server.Items.StonePaversDark`.
    STONE_PAVERS_DARK = "Server.Items.StonePaversDark" : BASE_FLOOR, v0;
    construct: Some(|w| stone_pavers_dark_new(w, &STONE_PAVERS_DARK)),
}

/// `StonePaversLight()`.
pub fn stone_pavers_light_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    base_floor_new(w, ty, 0x519, 4)
}

leaf! {
    /// `Server.Items.StonePaversLight`.
    STONE_PAVERS_LIGHT = "Server.Items.StonePaversLight" : BASE_FLOOR, v0;
    construct: Some(|w| stone_pavers_light_new(w, &STONE_PAVERS_LIGHT)),
}

/// `StonePaversMedium()`.
pub fn stone_pavers_medium_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    base_floor_new(w, ty, 0x51D, 4)
}

leaf! {
    /// `Server.Items.StonePaversMedium`.
    STONE_PAVERS_MEDIUM = "Server.Items.StonePaversMedium" : BASE_FLOOR, v0;
    construct: Some(|w| stone_pavers_medium_new(w, &STONE_PAVERS_MEDIUM)),
}

/// `BarrelHoops()`.
pub fn barrel_hoops_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.BarrelHoops")
}

leaf! {
    /// `Server.Items.BarrelHoops`.
    BARREL_HOOPS = "Server.Items.BarrelHoops" : ITEM_BASE, v0;
    construct: Some(|w| barrel_hoops_new(w, &BARREL_HOOPS)),
    default_weight: crate::class_props::default_weight,
    label_number: crate::class_props::label_number,
}

/// `BarrelLid()`.
pub fn barrel_lid_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.BarrelLid")
}

leaf! {
    /// `Server.Items.BarrelLid`.
    BARREL_LID = "Server.Items.BarrelLid" : ITEM_BASE, v0;
    construct: Some(|w| barrel_lid_new(w, &BARREL_LID)),
    default_weight: crate::class_props::default_weight,
}

/// `BarrelStaves()`.
pub fn barrel_staves_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.BarrelStaves")
}

leaf! {
    /// `Server.Items.BarrelStaves`.
    BARREL_STAVES = "Server.Items.BarrelStaves" : ITEM_BASE, v0;
    construct: Some(|w| barrel_staves_new(w, &BARREL_STAVES)),
    default_weight: crate::class_props::default_weight,
}

/// `BarrelTap()`.
pub fn barrel_tap_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.BarrelTap")
}

leaf! {
    /// `Server.Items.BarrelTap`.
    BARREL_TAP = "Server.Items.BarrelTap" : ITEM_BASE, v0;
    construct: Some(|w| barrel_tap_new(w, &BARREL_TAP)),
    default_weight: crate::class_props::default_weight,
}

/// `Easel()`.
pub fn easel_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.Easel")
}

leaf! {
    /// `Server.Items.Easel`.
    EASEL = "Server.Items.Easel" : ITEM_BASE, v0;
    construct: Some(|w| easel_new(w, &EASEL)),
    default_weight: crate::class_props::default_weight,
}

/// `MeltedWax()`.
pub fn melted_wax_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.MeltedWax")
}

leaf! {
    /// `Server.Items.MeltedWax`.
    MELTED_WAX = "Server.Items.MeltedWax" : ITEM_BASE, v0;
    construct: Some(|w| melted_wax_new(w, &MELTED_WAX)),
    label_number: crate::class_props::label_number,
}

/// `ShortMusicStand()`.
pub fn short_music_stand_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.ShortMusicStand")
}

leaf! {
    /// `Server.Items.ShortMusicStand`.
    SHORT_MUSIC_STAND = "Server.Items.ShortMusicStand" : ITEM_BASE, v0;
    construct: Some(|w| short_music_stand_new(w, &SHORT_MUSIC_STAND)),
    default_weight: crate::class_props::default_weight,
}

/// `TallMusicStand()`.
pub fn tall_music_stand_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.TallMusicStand")
}

leaf! {
    /// `Server.Items.TallMusicStand`.
    TALL_MUSIC_STAND = "Server.Items.TallMusicStand" : ITEM_BASE, v0;
    construct: Some(|w| tall_music_stand_new(w, &TALL_MUSIC_STAND)),
    default_weight: crate::class_props::default_weight,
}

/// `Obelisk()`.
pub fn obelisk_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.Obelisk")
}

leaf! {
    /// `Server.Items.Obelisk`.
    OBELISK = "Server.Items.Obelisk" : ITEM_BASE, v0;
    construct: Some(|w| obelisk_new(w, &OBELISK)),
    label_number: crate::class_props::label_number,
}

/// `BambooScreen()`.
pub fn bamboo_screen_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.BambooScreen")
}

leaf! {
    /// `Server.Items.BambooScreen`.
    BAMBOO_SCREEN = "Server.Items.BambooScreen" : ITEM_BASE, v0;
    construct: Some(|w| bamboo_screen_new(w, &BAMBOO_SCREEN)),
    default_weight: crate::class_props::default_weight,
}

/// `ShojiScreen()`.
pub fn shoji_screen_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.ShojiScreen")
}

leaf! {
    /// `Server.Items.ShojiScreen`.
    SHOJI_SCREEN = "Server.Items.ShojiScreen" : ITEM_BASE, v0;
    construct: Some(|w| shoji_screen_new(w, &SHOJI_SCREEN)),
    default_weight: crate::class_props::default_weight,
}

/// `BustEast()`.
pub fn bust_east_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.BustEast")
}

leaf! {
    /// `Server.Items.BustEast`.
    BUST_EAST = "Server.Items.BustEast" : ITEM_BASE, v0;
    construct: Some(|w| bust_east_new(w, &BUST_EAST)),
    default_weight: crate::class_props::default_weight,
    force_show_properties: |w, _| w.core.opl_enabled,
}

/// `BustSouth()`.
pub fn bust_south_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.BustSouth")
}

leaf! {
    /// `Server.Items.BustSouth`.
    BUST_SOUTH = "Server.Items.BustSouth" : ITEM_BASE, v0;
    construct: Some(|w| bust_south_new(w, &BUST_SOUTH)),
    default_weight: crate::class_props::default_weight,
    force_show_properties: |w, _| w.core.opl_enabled,
}

/// `SmallTowerSculpture()`.
pub fn small_tower_sculpture_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.SmallTowerSculpture")
}

leaf! {
    /// `Server.Items.SmallTowerSculpture`.
    SMALL_TOWER_SCULPTURE = "Server.Items.SmallTowerSculpture" : ITEM_BASE, v0;
    construct: Some(|w| small_tower_sculpture_new(w, &SMALL_TOWER_SCULPTURE)),
    default_weight: crate::class_props::default_weight,
}

/// `StatueEast()`.
pub fn statue_east_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.StatueEast")
}

leaf! {
    /// `Server.Items.StatueEast`.
    STATUE_EAST = "Server.Items.StatueEast" : ITEM_BASE, v0;
    construct: Some(|w| statue_east_new(w, &STATUE_EAST)),
    default_weight: crate::class_props::default_weight,
    force_show_properties: |w, _| w.core.opl_enabled,
}

/// `StatueEast2()`.
pub fn statue_east2_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.StatueEast2")
}

leaf! {
    /// `Server.Items.StatueEast2`.
    STATUE_EAST2 = "Server.Items.StatueEast2" : ITEM_BASE, v0;
    construct: Some(|w| statue_east2_new(w, &STATUE_EAST2)),
    default_weight: crate::class_props::default_weight,
    force_show_properties: |w, _| w.core.opl_enabled,
}

/// `StatueNorth()`.
pub fn statue_north_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.StatueNorth")
}

leaf! {
    /// `Server.Items.StatueNorth`.
    STATUE_NORTH = "Server.Items.StatueNorth" : ITEM_BASE, v0;
    construct: Some(|w| statue_north_new(w, &STATUE_NORTH)),
    default_weight: crate::class_props::default_weight,
    force_show_properties: |w, _| w.core.opl_enabled,
}

/// `StatuePegasus()`.
pub fn statue_pegasus_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.StatuePegasus")
}

leaf! {
    /// `Server.Items.StatuePegasus`.
    STATUE_PEGASUS = "Server.Items.StatuePegasus" : ITEM_BASE, v0;
    construct: Some(|w| statue_pegasus_new(w, &STATUE_PEGASUS)),
    default_weight: crate::class_props::default_weight,
    force_show_properties: |w, _| w.core.opl_enabled,
}

/// `StatuePegasus2()`.
pub fn statue_pegasus2_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.StatuePegasus2")
}

leaf! {
    /// `Server.Items.StatuePegasus2`.
    STATUE_PEGASUS2 = "Server.Items.StatuePegasus2" : ITEM_BASE, v0;
    construct: Some(|w| statue_pegasus2_new(w, &STATUE_PEGASUS2)),
    default_weight: crate::class_props::default_weight,
    force_show_properties: |w, _| w.core.opl_enabled,
}

/// `StatueSouth()`.
pub fn statue_south_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.StatueSouth")
}

leaf! {
    /// `Server.Items.StatueSouth`.
    STATUE_SOUTH = "Server.Items.StatueSouth" : ITEM_BASE, v0;
    construct: Some(|w| statue_south_new(w, &STATUE_SOUTH)),
    default_weight: crate::class_props::default_weight,
    force_show_properties: |w, _| w.core.opl_enabled,
}

/// `StatueSouth2()`.
pub fn statue_south2_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.StatueSouth2")
}

leaf! {
    /// `Server.Items.StatueSouth2`.
    STATUE_SOUTH2 = "Server.Items.StatueSouth2" : ITEM_BASE, v0;
    construct: Some(|w| statue_south2_new(w, &STATUE_SOUTH2)),
    default_weight: crate::class_props::default_weight,
    force_show_properties: |w, _| w.core.opl_enabled,
}

/// `StatueSouthEast()`.
pub fn statue_south_east_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.StatueSouthEast")
}

leaf! {
    /// `Server.Items.StatueSouthEast`.
    STATUE_SOUTH_EAST = "Server.Items.StatueSouthEast" : ITEM_BASE, v0;
    construct: Some(|w| statue_south_east_new(w, &STATUE_SOUTH_EAST)),
    default_weight: crate::class_props::default_weight,
    force_show_properties: |w, _| w.core.opl_enabled,
}

/// `StatueWest()`.
pub fn statue_west_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.StatueWest")
}

leaf! {
    /// `Server.Items.StatueWest`.
    STATUE_WEST = "Server.Items.StatueWest" : ITEM_BASE, v0;
    construct: Some(|w| statue_west_new(w, &STATUE_WEST)),
    default_weight: crate::class_props::default_weight,
    force_show_properties: |w, _| w.core.opl_enabled,
}

/// `LargeVase()`.
pub fn large_vase_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.LargeVase")
}

leaf! {
    /// `Server.Items.LargeVase`.
    LARGE_VASE = "Server.Items.LargeVase" : ITEM_BASE, v0;
    construct: Some(|w| large_vase_new(w, &LARGE_VASE)),
    default_weight: crate::class_props::default_weight,
}

/// `SmallUrn()`.
pub fn small_urn_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.SmallUrn")
}

leaf! {
    /// `Server.Items.SmallUrn`.
    SMALL_URN = "Server.Items.SmallUrn" : ITEM_BASE, v0;
    construct: Some(|w| small_urn_new(w, &SMALL_URN)),
    default_weight: crate::class_props::default_weight,
}

/// `Vase()`.
pub fn vase_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.Vase")
}

leaf! {
    /// `Server.Items.Vase`.
    VASE = "Server.Items.Vase" : ITEM_BASE, v0;
    construct: Some(|w| vase_new(w, &VASE)),
    default_weight: crate::class_props::default_weight,
}

/// `CoveredChair()`.
pub fn covered_chair_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.CoveredChair")
}

leaf! {
    /// `Server.Items.CoveredChair`.
    COVERED_CHAIR = "Server.Items.CoveredChair" : ITEM_BASE, v0;
    construct: Some(|w| covered_chair_new(w, &COVERED_CHAIR)),
}

/// `RuinedArmoire()`.
pub fn ruined_armoire_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.RuinedArmoire")
}

leaf! {
    /// `Server.Items.RuinedArmoire`.
    RUINED_ARMOIRE = "Server.Items.RuinedArmoire" : ITEM_BASE, v0;
    construct: Some(|w| ruined_armoire_new(w, &RUINED_ARMOIRE)),
}

/// `RuinedBookcase()`.
pub fn ruined_bookcase_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.RuinedBookcase")
}

leaf! {
    /// `Server.Items.RuinedBookcase`.
    RUINED_BOOKCASE = "Server.Items.RuinedBookcase" : ITEM_BASE, v0;
    construct: Some(|w| ruined_bookcase_new(w, &RUINED_BOOKCASE)),
}

/// `RuinedBooks()`.
pub fn ruined_books_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.RuinedBooks")
}

leaf! {
    /// `Server.Items.RuinedBooks`.
    RUINED_BOOKS = "Server.Items.RuinedBooks" : ITEM_BASE, v0;
    construct: Some(|w| ruined_books_new(w, &RUINED_BOOKS)),
}

/// `RuinedChair()`.
pub fn ruined_chair_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.RuinedChair")
}

leaf! {
    /// `Server.Items.RuinedChair`.
    RUINED_CHAIR = "Server.Items.RuinedChair" : ITEM_BASE, v0;
    construct: Some(|w| ruined_chair_new(w, &RUINED_CHAIR)),
}

/// `RuinedClock()`.
pub fn ruined_clock_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.RuinedClock")
}

leaf! {
    /// `Server.Items.RuinedClock`.
    RUINED_CLOCK = "Server.Items.RuinedClock" : ITEM_BASE, v0;
    construct: Some(|w| ruined_clock_new(w, &RUINED_CLOCK)),
}

/// `RuinedDrawers()`.
pub fn ruined_drawers_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.RuinedDrawers")
}

leaf! {
    /// `Server.Items.RuinedDrawers`.
    RUINED_DRAWERS = "Server.Items.RuinedDrawers" : ITEM_BASE, v0;
    construct: Some(|w| ruined_drawers_new(w, &RUINED_DRAWERS)),
}

/// `RuinedFallenChairA()`.
pub fn ruined_fallen_chair_a_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.RuinedFallenChairA")
}

leaf! {
    /// `Server.Items.RuinedFallenChairA`.
    RUINED_FALLEN_CHAIR_A = "Server.Items.RuinedFallenChairA" : ITEM_BASE, v0;
    construct: Some(|w| ruined_fallen_chair_a_new(w, &RUINED_FALLEN_CHAIR_A)),
}

/// `RuinedFallenChairB()`.
pub fn ruined_fallen_chair_b_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.RuinedFallenChairB")
}

leaf! {
    /// `Server.Items.RuinedFallenChairB`.
    RUINED_FALLEN_CHAIR_B = "Server.Items.RuinedFallenChairB" : ITEM_BASE, v0;
    construct: Some(|w| ruined_fallen_chair_b_new(w, &RUINED_FALLEN_CHAIR_B)),
}

/// `RuinedPainting()`.
pub fn ruined_painting_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.RuinedPainting")
}

leaf! {
    /// `Server.Items.RuinedPainting`.
    RUINED_PAINTING = "Server.Items.RuinedPainting" : ITEM_BASE, v0;
    construct: Some(|w| ruined_painting_new(w, &RUINED_PAINTING)),
}

/// `WoodDebris()`.
pub fn wood_debris_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.WoodDebris")
}

leaf! {
    /// `Server.Items.WoodDebris`.
    WOOD_DEBRIS = "Server.Items.WoodDebris" : ITEM_BASE, v0;
    construct: Some(|w| wood_debris_new(w, &WOOD_DEBRIS)),
}

/// `ElvenPodium()`.
pub fn elven_podium_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.ElvenPodium")
}

leaf! {
    /// `Server.Items.ElvenPodium`.
    ELVEN_PODIUM = "Server.Items.ElvenPodium" : ITEM_BASE, enc;
    construct: Some(|w| elven_podium_new(w, &ELVEN_PODIUM)),
    default_weight: crate::class_props::default_weight,
}

/// `ElegantLowTable()`.
pub fn elegant_low_table_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.ElegantLowTable")
}

leaf! {
    /// `Server.Items.ElegantLowTable`.
    ELEGANT_LOW_TABLE = "Server.Items.ElegantLowTable" : ITEM_BASE, v0;
    construct: Some(|w| elegant_low_table_new(w, &ELEGANT_LOW_TABLE)),
    default_weight: crate::class_props::default_weight,
}

/// `LargeTable()`.
pub fn large_table_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.LargeTable")
}

leaf! {
    /// `Server.Items.LargeTable`.
    LARGE_TABLE = "Server.Items.LargeTable" : ITEM_BASE, v0;
    construct: Some(|w| large_table_new(w, &LARGE_TABLE)),
    default_weight: crate::class_props::default_weight,
}

/// `Nightstand()`.
pub fn nightstand_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.Nightstand")
}

leaf! {
    /// `Server.Items.Nightstand`.
    NIGHTSTAND = "Server.Items.Nightstand" : ITEM_BASE, v0;
    construct: Some(|w| nightstand_new(w, &NIGHTSTAND)),
    default_weight: crate::class_props::default_weight,
}

/// `PlainLowTable()`.
pub fn plain_low_table_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.PlainLowTable")
}

leaf! {
    /// `Server.Items.PlainLowTable`.
    PLAIN_LOW_TABLE = "Server.Items.PlainLowTable" : ITEM_BASE, v0;
    construct: Some(|w| plain_low_table_new(w, &PLAIN_LOW_TABLE)),
    default_weight: crate::class_props::default_weight,
}

/// `YewWoodTable()`.
pub fn yew_wood_table_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.YewWoodTable")
}

leaf! {
    /// `Server.Items.YewWoodTable`.
    YEW_WOOD_TABLE = "Server.Items.YewWoodTable" : ITEM_BASE, v0;
    construct: Some(|w| yew_wood_table_new(w, &YEW_WOOD_TABLE)),
    default_weight: crate::class_props::default_weight,
}

/// `WritingTable()`.
pub fn writing_table_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.WritingTable")
}

leaf! {
    /// `Server.Items.WritingTable`.
    WRITING_TABLE = "Server.Items.WritingTable" : ITEM_BASE, v0;
    construct: Some(|w| writing_table_new(w, &WRITING_TABLE)),
    default_weight: crate::class_props::default_weight,
}

/// Every class of this file: the type and its `[TypeAlias]` names.
pub static ALL: &[(&ItemType, &[&str])] = &[
    (&WOODEN_BENCH, &[]),
    (&BAMBOO_CHAIR, &[]),
    (&BIG_ELVEN_CHAIR, &[]),
    (&ELVEN_READING_CHAIR, &[]),
    (&FANCY_WOODEN_CHAIR_CUSHION, &[]),
    (&ORNATE_ELVEN_CHAIR, &[]),
    (&STONE_CHAIR, &[]),
    (&WOODEN_CHAIR, &[]),
    (&WOODEN_CHAIR_CUSHION, &[]),
    (&FOOT_STOOL, &[]),
    (&STOOL, &[]),
    (&THRONE, &[]),
    (&WOODEN_THRONE, &[]),
    (&DECORATIVE_SHIELD1, &[]),
    (&DECORATIVE_SHIELD10, &[]),
    (&DECORATIVE_SHIELD11, &[]),
    (&DECORATIVE_SHIELD2, &[]),
    (&DECORATIVE_SHIELD3, &[]),
    (&DECORATIVE_SHIELD4, &[]),
    (&DECORATIVE_SHIELD5, &[]),
    (&DECORATIVE_SHIELD6, &[]),
    (&DECORATIVE_SHIELD7, &[]),
    (&DECORATIVE_SHIELD8, &[]),
    (&DECORATIVE_SHIELD9, &[]),
    (&DECORATIVE_SHIELD_SWORD1_NORTH, &[]),
    (&DECORATIVE_SHIELD_SWORD1_WEST, &[]),
    (&DECORATIVE_SHIELD_SWORD2_NORTH, &[]),
    (&DECORATIVE_SHIELD_SWORD2_WEST, &[]),
    (&DECORATIVE_AXE_NORTH, &[]),
    (&DECORATIVE_AXE_WEST, &[]),
    (&DECORATIVE_BOW_NORTH, &[]),
    (&DECORATIVE_BOW_WEST, &[]),
    (&GIANT_REPLICA_ACORN, &[]),
    (&MOUNTED_DREAD_HORN, &[]),
    (&LADY_PORTRAIT1, &[]),
    (&LADY_PORTRAIT2, &[]),
    (&LARGE_PAINTING, &[]),
    (&MAN_PORTRAIT1, &[]),
    (&MAN_PORTRAIT2, &[]),
    (&WOMAN_PORTRAIT1, &[]),
    (&WOMAN_PORTRAIT2, &[]),
    (&BLUE_SLATE_FLOOR_CENTER, &[]),
    (&BRICKS_FLOOR1, &[]),
    (&BRICKS_FLOOR2, &[]),
    (&CAVE_FLOOR_CENTER, &[]),
    (&CAVE_FLOOR_EAST, &[]),
    (&CAVE_FLOOR_NORTH, &[]),
    (&CAVE_FLOOR_SOUTH, &[]),
    (&CAVE_FLOOR_WEST, &[]),
    (&COBBLESTONES_FLOOR, &[]),
    (&DARK_SANDSTONE_FLOOR_N, &[]),
    (&DARK_SANDSTONE_FLOOR_W, &[]),
    (&GREEN_MARBLE_FLOOR, &[]),
    (&GREY_FLAGSTONES, &[]),
    (&GREY_MARBLE_FLOOR, &[]),
    (&GREY_SLATE_FLOOR, &[]),
    (&MARBLE_FLOOR, &[]),
    (&MARBLE_PAVERS, &[]),
    (&SAND_FLAGSTONES, &[]),
    (&SANDSTONE_FLOOR_N, &[]),
    (&SANDSTONE_FLOOR_W, &[]),
    (&STONE_PAVERS_DARK, &[]),
    (&STONE_PAVERS_LIGHT, &[]),
    (&STONE_PAVERS_MEDIUM, &[]),
    (&BARREL_HOOPS, &[]),
    (&BARREL_LID, &[]),
    (&BARREL_STAVES, &[]),
    (&BARREL_TAP, &[]),
    (&EASEL, &["Server.Items.Easle"]),
    (&MELTED_WAX, &[]),
    (&SHORT_MUSIC_STAND, &[]),
    (&TALL_MUSIC_STAND, &[]),
    (&OBELISK, &[]),
    (&BAMBOO_SCREEN, &[]),
    (&SHOJI_SCREEN, &[]),
    (&BUST_EAST, &[]),
    (&BUST_SOUTH, &[]),
    (&SMALL_TOWER_SCULPTURE, &[]),
    (&STATUE_EAST, &[]),
    (&STATUE_EAST2, &[]),
    (&STATUE_NORTH, &[]),
    (&STATUE_PEGASUS, &[]),
    (&STATUE_PEGASUS2, &[]),
    (&STATUE_SOUTH, &[]),
    (&STATUE_SOUTH2, &[]),
    (&STATUE_SOUTH_EAST, &[]),
    (&STATUE_WEST, &[]),
    (&LARGE_VASE, &[]),
    (&SMALL_URN, &[]),
    (&VASE, &[]),
    (&COVERED_CHAIR, &[]),
    (&RUINED_ARMOIRE, &[]),
    (&RUINED_BOOKCASE, &[]),
    (&RUINED_BOOKS, &[]),
    (&RUINED_CHAIR, &[]),
    (&RUINED_CLOCK, &[]),
    (&RUINED_DRAWERS, &[]),
    (&RUINED_FALLEN_CHAIR_A, &[]),
    (&RUINED_FALLEN_CHAIR_B, &[]),
    (&RUINED_PAINTING, &[]),
    (&WOOD_DEBRIS, &[]),
    (&ELVEN_PODIUM, &[]),
    (&ELEGANT_LOW_TABLE, &[]),
    (&LARGE_TABLE, &[]),
    (&NIGHTSTAND, &[]),
    (&PLAIN_LOW_TABLE, &[]),
    (&YEW_WOOD_TABLE, &[]),
    (&WRITING_TABLE, &[]),
];

/// `[Flippable(a, b)]` of the classes (the flip command reads it).
pub static FLIPPABLE: &[(&ItemType, i32, i32)] = &[
    (&WOODEN_BENCH, 0xB2D, 0xB2C),
    (&ELVEN_READING_CHAIR, 0x2DF5, 0x2DF6),
    (&THRONE, 0xB32, 0xB33),
    (&DECORATIVE_SHIELD1, 0x156C, 0x156D),
    (&DECORATIVE_SHIELD10, 0x157E, 0x157F),
    (&DECORATIVE_SHIELD11, 0x1580, 0x1581),
    (&DECORATIVE_SHIELD2, 0x156E, 0x156F),
    (&DECORATIVE_SHIELD3, 0x1570, 0x1571),
    (&DECORATIVE_SHIELD4, 0x1572, 0x1573),
    (&DECORATIVE_SHIELD5, 0x1574, 0x1575),
    (&DECORATIVE_SHIELD6, 0x1576, 0x1577),
    (&DECORATIVE_SHIELD7, 0x1578, 0x1579),
    (&DECORATIVE_SHIELD8, 0x157A, 0x157B),
    (&DECORATIVE_SHIELD9, 0x157C, 0x157D),
    (&MOUNTED_DREAD_HORN, 0x3158, 0x3159),
    (&LADY_PORTRAIT1, 0x0EA6, 0x0EA5),
    (&LADY_PORTRAIT2, 0x0EA7, 0x0EA8),
    (&MAN_PORTRAIT1, 0x0EA2, 0x0EA1),
    (&MAN_PORTRAIT2, 0x0EA3, 0x0EA4),
    (&WOMAN_PORTRAIT1, 0x0E9F, 0x0EC8),
    (&WOMAN_PORTRAIT2, 0x0EE7, 0x0EC9),
    (&SHORT_MUSIC_STAND, 0xEB6, 0xEB8),
    (&TALL_MUSIC_STAND, 0xEBB, 0xEBC),
    (&COVERED_CHAIR, 0xC17, 0xC18),
    (&RUINED_ARMOIRE, 0xC13, 0xC12),
    (&RUINED_BOOKCASE, 0xC14, 0xC15),
    (&RUINED_DRAWERS, 0xC24, 0xC25),
    (&RUINED_FALLEN_CHAIR_A, 0xC10, 0xC11),
    (&RUINED_FALLEN_CHAIR_B, 0xC19, 0xC1A),
    (&ELVEN_PODIUM, 0x2DDD, 0x2DDE),
    (&LARGE_TABLE, 0xB90, 0xB7D),
    (&NIGHTSTAND, 0xB35, 0xB34),
    (&YEW_WOOD_TABLE, 0xB8F, 0xB7C),
];

/// The interfaces a class lists after its base class (`is ICommodity` and the like); none has a consumer yet.
pub static IMPLEMENTS: &[(&ItemType, &str)] = &[];

/// `ICommodity` members of a class: (type, `IsDeedable`); `DescriptionNumber` is `LabelNumber`.
pub static COMMODITIES: &[(&ItemType, bool)] = &[];

/// Classes with `[Flippable]` or `[DynamicFlipping]` and no graphics pair (the flip is the class itself).
pub static OWN_FLIP: &[&ItemType] = &[
    &BIG_ELVEN_CHAIR,
    &ELVEN_READING_CHAIR,
    &ORNATE_ELVEN_CHAIR,
    &STONE_CHAIR,
];

/// Classes with `[Furniture]`.
pub static FURNITURE: &[&ItemType] = &[
    &WOODEN_BENCH,
    &BAMBOO_CHAIR,
    &FANCY_WOODEN_CHAIR_CUSHION,
    &WOODEN_CHAIR,
    &WOODEN_CHAIR_CUSHION,
    &FOOT_STOOL,
    &STOOL,
    &THRONE,
    &WOODEN_THRONE,
    &GIANT_REPLICA_ACORN,
    &EASEL,
    &SHORT_MUSIC_STAND,
    &TALL_MUSIC_STAND,
    &BAMBOO_SCREEN,
    &SHOJI_SCREEN,
    &ELVEN_PODIUM,
    &ELEGANT_LOW_TABLE,
    &LARGE_TABLE,
    &NIGHTSTAND,
    &PLAIN_LOW_TABLE,
    &YEW_WOOD_TABLE,
    &WRITING_TABLE,
];

/// Registers the classes of this file.
pub fn register(types: &mut TypeRegistry) {
    for (t, aliases) in ALL {
        types.register_item(t, aliases);
    }
}
