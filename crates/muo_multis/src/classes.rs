//! The thirteen standard house classes. Each is a [`HouseClass`] table over
//! `BASE_HOUSE`, a constructor that builds its doors and sign, and the `[SerializationGenerator(0, false)]` int 0.

use muo_primitives::{Direction, Point3D, Rectangle2D, TimeSpan};
use muo_server::World;
use muo_server::ids::{ItemId, MobileId};
use muo_server::item::ItemType;
use muo_server::type_registry::TypeRegistry;

use crate::house::doors::{
    add_east_door, add_east_door_k, add_south_door, add_south_door_k, add_south_doors,
    add_south_doors_k, create_keys, make_door, set_sign,
};
use crate::house::save::{new_base_house, read_class_version, write_class_version};
use crate::house::{BASE_HOUSE, HouseClass};
use crate::placement_entry::Table;

pub const SMALL_OLD_HOUSE_AREA: [Rectangle2D; 2] = [
    Rectangle2D::new(-3, -3, 7, 7),
    Rectangle2D::new(-1, 4, 3, 1),
];
pub const GUILD_HOUSE_AREA: [Rectangle2D; 2] = [
    Rectangle2D::new(-7, -7, 14, 14),
    Rectangle2D::new(-2, 7, 4, 1),
];
pub const TWO_STORY_HOUSE_AREA: [Rectangle2D; 3] = [
    Rectangle2D::new(-7, 0, 14, 7),
    Rectangle2D::new(-7, -7, 9, 7),
    Rectangle2D::new(-4, 7, 4, 1),
];
pub const TOWER_AREA: [Rectangle2D; 4] = [
    Rectangle2D::new(-7, -7, 16, 14),
    Rectangle2D::new(-1, 7, 4, 2),
    Rectangle2D::new(-11, 0, 4, 7),
    Rectangle2D::new(9, 0, 4, 7),
];
pub const KEEP_AREA: [Rectangle2D; 8] = [
    Rectangle2D::new(-11, -11, 7, 8),
    Rectangle2D::new(-11, 5, 7, 8),
    Rectangle2D::new(6, -11, 7, 8),
    Rectangle2D::new(6, 5, 7, 8),
    Rectangle2D::new(-9, -3, 5, 8),
    Rectangle2D::new(6, -3, 5, 8),
    Rectangle2D::new(-4, -9, 10, 20),
    Rectangle2D::new(-1, 11, 4, 1),
];
pub const CASTLE_AREA: [Rectangle2D; 2] = [
    Rectangle2D::new(-15, -15, 31, 31),
    Rectangle2D::new(-1, 16, 4, 1),
];
pub const LARGE_PATIO_AREA: [Rectangle2D; 2] = [
    Rectangle2D::new(-7, -7, 15, 14),
    Rectangle2D::new(-5, 7, 4, 1),
];
pub const LARGE_MARBLE_AREA: [Rectangle2D; 2] = [
    Rectangle2D::new(-7, -7, 15, 14),
    Rectangle2D::new(-6, 7, 6, 1),
];
pub const SMALL_TOWER_AREA: [Rectangle2D; 2] =
    [Rectangle2D::new(-3, -3, 8, 7), Rectangle2D::new(2, 4, 3, 1)];
pub const LOG_CABIN_AREA: [Rectangle2D; 1] = [Rectangle2D::new(-3, -6, 8, 13)];
pub const SANDSTONE_PATIO_AREA: [Rectangle2D; 2] = [
    Rectangle2D::new(-5, -4, 12, 8),
    Rectangle2D::new(-2, 4, 3, 1),
];
pub const TWO_STORY_VILLA_AREA: [Rectangle2D; 2] = [
    Rectangle2D::new(-5, -5, 11, 11),
    Rectangle2D::new(2, 6, 4, 1),
];
pub const SMALL_SHOP_AREA_1: [Rectangle2D; 2] = [
    Rectangle2D::new(-3, -3, 7, 7),
    Rectangle2D::new(-1, 4, 4, 1),
];
pub const SMALL_SHOP_AREA_2: [Rectangle2D; 2] = [
    Rectangle2D::new(-3, -3, 7, 7),
    Rectangle2D::new(-2, 4, 3, 1),
];

/// `new Deed()` by class name.
fn deed(w: &mut World, name: &str) -> Option<ItemId> {
    muo_content::factory::make_item(w, &format!("Server.Multis.Deeds.{name}"))
}

fn item_id(w: &World, id: ItemId) -> i32 {
    w.item(id).map_or(0, |i| i.item_id)
}

/// One class: the `HouseClass` table and the `ItemType`.
macro_rules! house {
    (
        $name:ident, $cls:ident, $full:literal, $short:literal,
        area: $area:expr,
        ban: ($bx:expr, $by:expr, $bz:expr),
        price: $price:expr,
        convert: $convert:expr, offset: $offset:expr,
        deed: $deed:expr
    ) => {
        pub static $cls: HouseClass = HouseClass {
            area: $area,
            base_ban_location: |_, _| Point3D::new($bx, $by, $bz),
            default_price: |_, _| $price,
            convert_entry: |_, _| $convert,
            convert_offset: $offset,
            get_deed: $deed,
            house_direction: Direction::SOUTH,
            decay_period: TimeSpan::from_days(5),
            is_aos_rules: None,
            is_active: |_, _| true,
            class_name: $short,
            is_inside_extra: None,
        };

        #[doc = concat!("`", $full, "`.")]
        pub static $name: ItemType = ItemType {
            full_name: $full,
            parent: Some(&BASE_HOUSE),
            construct: None,
            family: Some(&$cls),
            serialize: |w, id, out| {
                (BASE_HOUSE.serialize)(w, id, out);
                write_class_version(out);
            },
            deserialize: |w, id, r| {
                (BASE_HOUSE.deserialize)(w, id, r)?;
                read_class_version(r)
            },
            ..BASE_HOUSE
        };
    };
}

house!(SMALL_OLD_HOUSE, SMALL_OLD_HOUSE_CLASS, "Server.Multis.SmallOldHouse", "SmallOldHouse",
area: |_, _| SMALL_OLD_HOUSE_AREA.to_vec(), ban: (2, 4, 0), price: 43800,
convert: Some((Table::Two, 0)), offset: (0, 0, 0),
deed: |w, id| {
    let name = match item_id(w, id) {
        0x64 => "StonePlasterHouseDeed",
        0x66 => "FieldStoneHouseDeed",
        0x68 => "SmallBrickHouseDeed",
        0x6A => "WoodHouseDeed",
        0x6C => "WoodPlasterHouseDeed",
        _ => "ThatchedRoofCottageDeed",
    };
    deed(w, name)
});

house!(GUILD_HOUSE, GUILD_HOUSE_CLASS, "Server.Multis.GuildHouse", "GuildHouse",
    area: |_, _| GUILD_HOUSE_AREA.to_vec(), ban: (4, 8, 0), price: 144500,
    convert: Some((Table::Three, 20)), offset: (-1, -1, 0),
    deed: |w, _| deed(w, "BrickHouseDeed"));

house!(TWO_STORY_HOUSE, TWO_STORY_HOUSE_CLASS, "Server.Multis.TwoStoryHouse", "TwoStoryHouse",
area: |_, _| TWO_STORY_HOUSE_AREA.to_vec(), ban: (2, 8, 0), price: 192400,
convert: None, offset: (0, 0, 0),
deed: |w, id| {
    let name = if item_id(w, id) == 0x76 { "TwoStoryWoodPlasterHouseDeed" } else { "TwoStoryStonePlasterHouseDeed" };
    deed(w, name)
});

house!(TOWER, TOWER_CLASS, "Server.Multis.Tower", "Tower",
    area: |_, _| TOWER_AREA.to_vec(), ban: (5, 8, 0), price: 433200,
    convert: Some((Table::Three, 37)), offset: (0, -1, 0),
    deed: |w, _| deed(w, "TowerDeed"));

house!(KEEP, KEEP_CLASS, "Server.Multis.Keep", "Keep",
    area: |_, _| KEEP_AREA.to_vec(), ban: (5, 13, 0), price: 665200,
    convert: None, offset: (0, 0, 0),
    deed: |w, _| deed(w, "KeepDeed"));

house!(CASTLE, CASTLE_CLASS, "Server.Multis.Castle", "Castle",
    area: |_, _| CASTLE_AREA.to_vec(), ban: (5, 17, 0), price: 1022800,
    convert: None, offset: (0, 0, 0),
    deed: |w, _| deed(w, "CastleDeed"));

house!(LARGE_PATIO_HOUSE, LARGE_PATIO_HOUSE_CLASS, "Server.Multis.LargePatioHouse", "LargePatioHouse",
    area: |_, _| LARGE_PATIO_AREA.to_vec(), ban: (1, 8, 0), price: 152800,
    convert: Some((Table::Three, 29)), offset: (0, -1, 0),
    deed: |w, _| deed(w, "LargePatioDeed"));

house!(LARGE_MARBLE_HOUSE, LARGE_MARBLE_HOUSE_CLASS, "Server.Multis.LargeMarbleHouse", "LargeMarbleHouse",
    area: |_, _| LARGE_MARBLE_AREA.to_vec(), ban: (1, 8, 0), price: 192000,
    convert: Some((Table::Three, 29)), offset: (0, -1, 0),
    deed: |w, _| deed(w, "LargeMarbleDeed"));

house!(SMALL_TOWER, SMALL_TOWER_CLASS, "Server.Multis.SmallTower", "SmallTower",
    area: |_, _| SMALL_TOWER_AREA.to_vec(), ban: (1, 4, 0), price: 88500,
    convert: Some((Table::Two, 6)), offset: (0, 0, 0),
    deed: |w, _| deed(w, "SmallTowerDeed"));

house!(LOG_CABIN, LOG_CABIN_CLASS, "Server.Multis.LogCabin", "LogCabin",
    area: |_, _| LOG_CABIN_AREA.to_vec(), ban: (5, 8, 0), price: 97800,
    convert: Some((Table::Two, 12)), offset: (0, 0, 0),
    deed: |w, _| deed(w, "LogCabinDeed"));

house!(SAND_STONE_PATIO, SAND_STONE_PATIO_CLASS, "Server.Multis.SandStonePatio", "SandStonePatio",
    area: |_, _| SANDSTONE_PATIO_AREA.to_vec(), ban: (4, 6, 0), price: 90900,
    convert: Some((Table::Two, 35)), offset: (0, -1, 0),
    deed: |w, _| deed(w, "SandstonePatioDeed"));

house!(TWO_STORY_VILLA, TWO_STORY_VILLA_CLASS, "Server.Multis.TwoStoryVilla", "TwoStoryVilla",
    area: |_, _| TWO_STORY_VILLA_AREA.to_vec(), ban: (3, 8, 0), price: 136500,
    convert: Some((Table::Two, 31)), offset: (0, 0, 0),
    deed: |w, _| deed(w, "VillaDeed"));

house!(SMALL_SHOP, SMALL_SHOP_CLASS, "Server.Multis.SmallShop", "SmallShop",
    area: |w, id| if item_id(w, id) == 0x40A2 { SMALL_SHOP_AREA_1.to_vec() } else { SMALL_SHOP_AREA_2.to_vec() },
    ban: (3, 4, 0), price: 63000,
    convert: Some((Table::Two, 0)), offset: (0, 0, 0),
    deed: |w, id| deed(w, if item_id(w, id) == 0xA0 { "StoneWorkshopDeed" } else { "MarbleWorkshopDeed" }));

// ---- constructors ----

pub fn new_small_old_house(w: &mut World, owner: MobileId, id: i32) -> ItemId {
    let h = new_base_house(w, &SMALL_OLD_HOUSE, id, Some(owner), 425, 3);
    let k = create_keys(w, h, owner);
    add_south_door_k(w, h, true, 0, 3, 7, k);
    set_sign(w, h, 2, 4, 5);
    h
}

pub fn new_guild_house(w: &mut World, owner: MobileId) -> ItemId {
    let h = new_base_house(w, &GUILD_HOUSE, 0x74, Some(owner), 1100, 8);
    let k = create_keys(w, h, owner);
    add_south_doors_k(w, h, true, -1, 6, 7, k);
    set_sign(w, h, 4, 8, 16);
    add_south_door(w, h, true, -3, -1, 7);
    add_south_door(w, h, true, 3, -1, 7);
    h
}

pub fn new_two_story_house(w: &mut World, owner: MobileId, id: i32) -> ItemId {
    let h = new_base_house(w, &TWO_STORY_HOUSE, id, Some(owner), 1370, 10);
    let k = create_keys(w, h, owner);
    add_south_doors_k(w, h, true, -3, 6, 7, k);
    set_sign(w, h, 2, 8, 16);
    add_south_door(w, h, true, -3, 0, 7);
    add_south_door(w, h, true, if id == 0x76 { -2 } else { -3 }, 0, 27);
    h
}

pub fn new_tower(w: &mut World, owner: MobileId) -> ItemId {
    let h = new_base_house(w, &TOWER, 0x7A, Some(owner), 2119, 15);
    let k = create_keys(w, h, owner);
    add_south_doors_k(w, h, false, 0, 6, 6, k);
    set_sign(w, h, 5, 8, 16);
    add_south_door(w, h, false, 3, -2, 6);
    add_east_door(w, h, false, 1, 4, 26);
    add_east_door(w, h, false, 1, 4, 46);
    h
}

pub fn new_keep(w: &mut World, owner: MobileId) -> ItemId {
    let h = new_base_house(w, &KEEP, 0x7C, Some(owner), 2625, 18);
    let k = create_keys(w, h, owner);
    add_south_doors_k(w, h, false, 0, 10, 6, k);
    set_sign(w, h, 5, 12, 16);
    h
}

pub fn new_castle(w: &mut World, owner: MobileId) -> ItemId {
    let h = new_base_house(w, &CASTLE, 0x7E, Some(owner), 4076, 28);
    let k = create_keys(w, h, owner);
    add_south_doors_k(w, h, false, 0, 15, 6, k);
    set_sign(w, h, 5, 17, 16);
    add_south_doors(w, h, false, 0, 11, 6, true);
    add_south_doors(w, h, false, 0, 5, 6, false);
    add_south_doors(w, h, false, -1, -11, 6, false);
    h
}

pub fn new_large_patio_house(w: &mut World, owner: MobileId) -> ItemId {
    let h = new_base_house(w, &LARGE_PATIO_HOUSE, 0x8C, Some(owner), 1100, 8);
    let k = create_keys(w, h, owner);
    add_south_doors_k(w, h, true, -4, 6, 7, k);
    set_sign(w, h, 1, 8, 16);
    add_east_door(w, h, true, 1, 4, 7);
    // Patio doors are external to prevent res-looting
    add_east_door_k(w, h, true, 1, -4, 7, k);
    add_south_door_k(w, h, true, 4, -1, 7, k);
    h
}

pub fn new_large_marble_house(w: &mut World, owner: MobileId) -> ItemId {
    let h = new_base_house(w, &LARGE_MARBLE_HOUSE, 0x96, Some(owner), 1370, 10);
    let k = create_keys(w, h, owner);
    add_south_doors_k(w, h, false, -4, 3, 4, k);
    set_sign(w, h, 1, 8, 11);
    h
}

pub fn new_small_tower(w: &mut World, owner: MobileId) -> ItemId {
    let h = new_base_house(w, &SMALL_TOWER, 0x98, Some(owner), 580, 4);
    let k = create_keys(w, h, owner);
    add_south_door_k(w, h, false, 3, 3, 6, k);
    set_sign(w, h, 1, 4, 5);
    h
}

pub fn new_log_cabin(w: &mut World, owner: MobileId) -> ItemId {
    let h = new_base_house(w, &LOG_CABIN, 0x9A, Some(owner), 1100, 8);
    let k = create_keys(w, h, owner);
    add_south_door_k(w, h, true, 1, 4, 8, k);
    set_sign(w, h, 5, 8, 20);
    // Balcony door is external to prevent res-looting
    add_south_door_k(w, h, true, 1, 0, 29, k);
    h
}

pub fn new_sand_stone_patio(w: &mut World, owner: MobileId) -> ItemId {
    let h = new_base_house(w, &SAND_STONE_PATIO, 0x9C, Some(owner), 850, 6);
    let k = create_keys(w, h, owner);
    add_south_door_k(w, h, true, -1, 3, 6, k);
    set_sign(w, h, 4, 6, 24);
    h
}

pub fn new_two_story_villa(w: &mut World, owner: MobileId) -> ItemId {
    let h = new_base_house(w, &TWO_STORY_VILLA, 0x9E, Some(owner), 1100, 8);
    let k = create_keys(w, h, owner);
    add_south_doors_k(w, h, true, 3, 1, 5, k);
    set_sign(w, h, 3, 8, 24);
    // Balcony door is external to prevent res-looting
    add_east_door_k(w, h, true, 1, 0, 25, k);
    add_south_door(w, h, true, -3, -1, 25);
    h
}

pub fn new_small_shop(w: &mut World, owner: MobileId, id: i32) -> ItemId {
    use muo_content::items::house_deps::{DoorFacing, deps};
    let h = new_base_house(w, &SMALL_SHOP, id, Some(owner), 425, 3);
    let k = create_keys(w, h, owner);
    if let Some(door) = make_door(w, false, DoorFacing::EastCW) {
        if let Some(f) = deps(w).and_then(|d| d.door_set_locked) {
            f(w, door, true);
        }
        if let Some(f) = deps(w).and_then(|d| d.door_set_key_value) {
            f(w, door, k);
        }
        if let Some(f) = deps(w).and_then(|d| d.door_set_facing) {
            f(w, door, DoorFacing::EastCCW);
        }
        crate::house::doors::add_door(w, h, door, -2, 0, if id == 0xA2 { 24 } else { 27 });
    }
    set_sign(w, h, 3, 4, 7 - if id == 0xA2 { 2 } else { 0 });
    h
}

pub fn register(types: &mut TypeRegistry) {
    for ty in [
        &SMALL_OLD_HOUSE,
        &GUILD_HOUSE,
        &TWO_STORY_HOUSE,
        &TOWER,
        &KEEP,
        &CASTLE,
        &LARGE_PATIO_HOUSE,
        &LARGE_MARBLE_HOUSE,
        &SMALL_TOWER,
        &LOG_CABIN,
        &SAND_STONE_PATIO,
        &TWO_STORY_VILLA,
        &SMALL_SHOP,
    ] {
        types.register_item(ty, &[]);
    }
}
