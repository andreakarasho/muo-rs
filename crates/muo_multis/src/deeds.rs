//! The house deeds and their placement target.
//!
//! A deed is `HouseDeed` state ([`DeedData`]: the offset and the multi id) over a [`DeedClass`] (its `Area` and
//! `GetHouse`), the same shape as a house class.

use muo_primitives::{Direction, Point3D, Rectangle2D};
use muo_server::World;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType, LootType, ops};
use muo_server::region;
use muo_server::target::{Target, TargetFlags, Targeted, add_target, set_mobile_target};
use muo_server::type_registry::TypeRegistry;

use crate::classes::*;
use crate::house::access::{has_account_house, msg};
use crate::house::decay::TEMP_NO_HOUSING_REGION;
use crate::placement::{self, HousePlacementResult};

/// What `HouseDeed` adds (`[SerializableField]` offset and multiID).
#[derive(Clone, Copy, Debug)]
pub struct DeedData {
    pub offset: Point3D,
    pub multi_id: i32,
}

/// What a concrete deed overrides.
pub struct DeedClass {
    pub area: fn() -> Vec<Rectangle2D>,
    /// `GetHouse(owner)`: the new house.
    pub get_house: fn(&mut World, MobileId) -> Option<ItemId>,
    pub house_direction: Direction,
}

pub static BASE_DEED_CLASS: DeedClass = DeedClass {
    area: Vec::new,
    get_house: |_, _| None,
    house_direction: Direction::SOUTH,
};

fn class(w: &World, id: ItemId) -> &'static DeedClass {
    w.item(id)
        .and_then(|i| i.ty.family::<DeedClass>())
        .unwrap_or(&BASE_DEED_CLASS)
}

/// `item is HouseDeed`.
pub fn is_deed(w: &World, id: ItemId) -> bool {
    w.item(id).is_some_and(|i| i.ty.is(&HOUSE_DEED))
}

fn data(w: &World, id: ItemId) -> Option<DeedData> {
    w.item(id)?.ext.get::<DeedData>().copied()
}

/// `HouseDeed.Offset`.
pub fn offset(w: &World, id: ItemId) -> Point3D {
    data(w, id).map_or(Point3D::ZERO, |d| d.offset)
}

/// `HouseDeed.MultiID`.
pub fn multi_id(w: &World, id: ItemId) -> i32 {
    data(w, id).map_or(0, |d| d.multi_id)
}

/// `new HouseDeed(id, offset)` of class `ty`.
pub fn new_deed(w: &mut World, ty: &'static ItemType, id: i32, offset: Point3D) -> ItemId {
    let item = ops::new_item_with_id(w, ty, 0x14F0);
    ops::set_loot_type(w, item, LootType::Newbied);
    if let Some(i) = w.item_mut(item) {
        i.ext.insert(DeedData {
            offset,
            multi_id: id,
        });
    }
    item
}

fn in_pack(w: &World, deed: ItemId, from: MobileId) -> bool {
    let pack = w.mobile(from).and_then(|m| m.backpack);
    match pack {
        Some(p) => ops::is_child_of(w, deed, Some(EntityId::Item(p))),
        None => ops::is_child_of(w, deed, None),
    }
}

fn is_gm(w: &World, m: MobileId) -> bool {
    w.mobile(m)
        .is_some_and(|x| x.access_level >= muo_server::mobile::AccessLevel::GameMaster)
}

fn on_double_click(w: &mut World, id: ItemId, from: MobileId) {
    if !in_pack(w, id, from) {
        msg(w, from, 1042001); // That must be in your pack for you to use it.
    } else if !is_gm(w, from) && has_account_house(w, from) {
        msg(w, from, 501271); // You already own a house, you may not place another!
    } else {
        // House placement cancellation could result in a 60 second delay in the return of your deed.
        msg(w, from, 1010433);
        begin_placement_target(w, from, id);
    }
}

/// `from.Target = new HousePlacementTarget(deed)`.
fn begin_placement_target(w: &mut World, from: MobileId, deed: ItemId) {
    let mut t = Target::new_multi(
        w,
        multi_id(w, deed),
        offset(w, deed),
        10,
        true,
        TargetFlags::NONE,
    );
    t.handlers.on_target = Some(std::rc::Rc::new(move |w, from, targeted| {
        placement_target(w, from, deed, targeted)
    }));
    let id = add_target(w, t);
    set_mobile_target(w, from, Some(id));
}

/// The point a `MultiTarget` was aimed at: an item's top, a mobile's location, or the point itself.
pub fn target_point(w: &World, targeted: Targeted) -> Option<Point3D> {
    Some(match targeted {
        Targeted::Entity(EntityId::Item(i)) => ops::get_world_top(w, i),
        Targeted::Entity(EntityId::Mobile(m)) => w.mobile(m)?.location,
        Targeted::Land(l) => l.location,
        Targeted::Static(s) => s.location,
    })
}

/// The refusal messages shared by `HousePlacementTarget` and `NewHousePlacementTarget` when the region does not
/// allow housing at `p`.
pub fn housing_refused(w: &mut World, from: MobileId, reg: region::RegionId) {
    if region::is_part_of_type(w, reg, &TEMP_NO_HOUSING_REGION) {
        // Lord British has decreed a 'no build' period, thus you cannot build this house at this time.
        msg(w, from, 501270);
    } else if region::is_part_of_type_name(w, reg, "TreasureRegion")
        || region::is_part_of_type(w, reg, &muo_content::regions::house::HOUSE_REGION)
    {
        // The house could not be created here.  Either something is blocking the house, or the house would not be
        // on valid terrain.
        msg(w, from, 1043287);
    } else if region::is_part_of_type_name(w, reg, "HouseRaffleRegion") {
        msg(w, from, 1150493); // You must have a deed for this plot of land in order to build here.
    } else {
        msg(w, from, 501265); // Housing can not be created in this area.
    }
}

/// `HousePlacementTarget.OnTarget`.
fn placement_target(w: &mut World, from: MobileId, deed: ItemId, targeted: Targeted) {
    let Some(p) = target_point(w, targeted) else {
        return;
    };
    let map = w.mobile(from).and_then(|m| m.map);
    let reg = region::find(w, p, map);
    if is_gm(w, from) || placement::allow_housing(w, reg, from, p) {
        on_placement(w, deed, from, p);
    } else {
        housing_refused(w, from, reg);
    }
}

/// `OnPlacement(from, p)`.
pub fn on_placement(w: &mut World, deed: ItemId, from: MobileId, p: Point3D) {
    if w.item(deed).is_none_or(|i| i.deleted()) {
        return;
    }
    if !in_pack(w, deed, from) {
        msg(w, from, 1042001); // That must be in your pack for you to use it.
    } else if !is_gm(w, from) && has_account_house(w, from) {
        msg(w, from, 501271); // You already own a house, you may not place another!
    } else {
        let o = offset(w, deed);
        let center = Point3D::new(p.x - o.x, p.y - o.y, p.z - o.z);
        let direction = class(w, deed).house_direction;
        let (res, to_move) = placement::check(w, from, multi_id(w, deed), center, direction);
        match res {
            HousePlacementResult::Valid => {
                let Some(house) = (class(w, deed).get_house)(w, from) else {
                    return;
                };
                let map = w.mobile(from).and_then(|m| m.map);
                ops::move_to_world_map(w, house, center, map);
                if let Some(ty) = w.item(deed).map(|i| i.ty) {
                    (ty.delete)(w, deed);
                }
                let ban = crate::house::ban_location(w, house);
                for e in to_move {
                    match e {
                        EntityId::Mobile(m) => {
                            muo_server::mobile::part1::lifecycle::set_location(w, m, ban);
                        }
                        EntityId::Item(i) => ops::set_location(w, i, ban),
                    }
                }
            }
            HousePlacementResult::BadItem
            | HousePlacementResult::BadLand
            | HousePlacementResult::BadStatic
            | HousePlacementResult::BadRegionHidden => {
                // The house could not be created here.  Either something is blocking the house, or the house would
                // not be on valid terrain.
                msg(w, from, 1043287);
            }
            HousePlacementResult::NoSurface => {
                muo_server::mobile::part2::messages::send_message(
                    w,
                    from,
                    "The house could not be created here.  Part of the foundation would not be on any surface.",
                );
            }
            HousePlacementResult::BadRegion => msg(w, from, 501265), // Housing cannot be created in this area.
            HousePlacementResult::BadRegionTemp => {
                // Lord British has decreed a 'no build' period, thus you cannot build this house at this time.
                msg(w, from, 501270);
            }
            HousePlacementResult::BadRegionRaffle => {
                msg(w, from, 1150493); // You must have a deed for this plot of land in order to build here.
            }
            HousePlacementResult::InvalidCastleKeep => {}
        }
    }
}

/// `Server.Multis.Deeds.HouseDeed` (abstract).
pub static HOUSE_DEED: ItemType = ItemType {
    full_name: "Server.Multis.Deeds.HouseDeed",
    parent: Some(&ITEM_BASE),
    construct: None,
    family: Some(&BASE_DEED_CLASS),
    default_weight: |_, _| 1.0,
    on_double_click,
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        out.write_i32(0); // version
        let d = data(w, id).unwrap_or(DeedData {
            offset: Point3D::ZERO,
            multi_id: 0,
        });
        out.write_point3d(d.offset);
        out.write_i32(d.multi_id);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        let offset = r.read_point3d()?;
        let multi_id = r.read_i32()?;
        if let Some(i) = w.item_mut(id) {
            i.ext.insert(DeedData { offset, multi_id });
        }
        Ok(())
    },
    ..ITEM_BASE
};

macro_rules! deeds {
    ($($name:ident, $cls:ident, $full:literal, $label:literal, $id:literal, ($ox:literal, $oy:literal, $oz:literal),
       $area:expr, $house:expr;)*) => {
        $(
            pub static $cls: DeedClass = DeedClass {
                area: || $area.to_vec(),
                get_house: |w, o| Some(($house)(w, o)),
                house_direction: Direction::SOUTH,
            };

            #[doc = concat!("`", $full, "`.")]
            pub static $name: ItemType = ItemType {
                full_name: $full,
                parent: Some(&HOUSE_DEED),
                construct: Some(|w| new_deed(w, &$name, $id, Point3D::new($ox, $oy, $oz))),
                family: Some(&$cls),
                label_number: |_, _| $label,
                serialize: |w, id, out| {
                    (HOUSE_DEED.serialize)(w, id, out);
                    out.write_i32(0);
                },
                deserialize: |w, id, r| {
                    (HOUSE_DEED.deserialize)(w, id, r)?;
                    r.read_i32()?;
                    Ok(())
                },
                ..HOUSE_DEED
            };
        )*

        fn register_all(types: &mut TypeRegistry) {
            $(types.register_item(&$name, &[]);)*
        }
    };
}

deeds! {
    STONE_PLASTER_HOUSE_DEED, STONE_PLASTER_HOUSE_DEED_CLASS, "Server.Multis.Deeds.StonePlasterHouseDeed", 1041211, 0x64, (0, 4, 0),
        SMALL_OLD_HOUSE_AREA, |w, o| new_small_old_house(w, o, 0x64);
    FIELD_STONE_HOUSE_DEED, FIELD_STONE_HOUSE_DEED_CLASS, "Server.Multis.Deeds.FieldStoneHouseDeed", 1041212, 0x66, (0, 4, 0),
        SMALL_OLD_HOUSE_AREA, |w, o| new_small_old_house(w, o, 0x66);
    SMALL_BRICK_HOUSE_DEED, SMALL_BRICK_HOUSE_DEED_CLASS, "Server.Multis.Deeds.SmallBrickHouseDeed", 1041213, 0x68, (0, 4, 0),
        SMALL_OLD_HOUSE_AREA, |w, o| new_small_old_house(w, o, 0x68);
    WOOD_HOUSE_DEED, WOOD_HOUSE_DEED_CLASS, "Server.Multis.Deeds.WoodHouseDeed", 1041214, 0x6A, (0, 4, 0),
        SMALL_OLD_HOUSE_AREA, |w, o| new_small_old_house(w, o, 0x6A);
    WOOD_PLASTER_HOUSE_DEED, WOOD_PLASTER_HOUSE_DEED_CLASS, "Server.Multis.Deeds.WoodPlasterHouseDeed", 1041215, 0x6C, (0, 4, 0),
        SMALL_OLD_HOUSE_AREA, |w, o| new_small_old_house(w, o, 0x6C);
    THATCHED_ROOF_COTTAGE_DEED, THATCHED_ROOF_COTTAGE_DEED_CLASS, "Server.Multis.Deeds.ThatchedRoofCottageDeed", 1041216, 0x6E, (0, 4, 0),
        SMALL_OLD_HOUSE_AREA, |w, o| new_small_old_house(w, o, 0x6E);
    BRICK_HOUSE_DEED, BRICK_HOUSE_DEED_CLASS, "Server.Multis.Deeds.BrickHouseDeed", 1041219, 0x74, (-1, 7, 0),
        GUILD_HOUSE_AREA, new_guild_house;
    TWO_STORY_WOOD_PLASTER_HOUSE_DEED, TWO_STORY_WOOD_PLASTER_HOUSE_DEED_CLASS, "Server.Multis.Deeds.TwoStoryWoodPlasterHouseDeed", 1041220, 0x76, (-3, 7, 0),
        TWO_STORY_HOUSE_AREA, |w, o| new_two_story_house(w, o, 0x76);
    TWO_STORY_STONE_PLASTER_HOUSE_DEED, TWO_STORY_STONE_PLASTER_HOUSE_DEED_CLASS, "Server.Multis.Deeds.TwoStoryStonePlasterHouseDeed", 1041221, 0x78, (-3, 7, 0),
        TWO_STORY_HOUSE_AREA, |w, o| new_two_story_house(w, o, 0x78);
    TOWER_DEED, TOWER_DEED_CLASS, "Server.Multis.Deeds.TowerDeed", 1041222, 0x7A, (0, 7, 0),
        TOWER_AREA, new_tower;
    KEEP_DEED, KEEP_DEED_CLASS, "Server.Multis.Deeds.KeepDeed", 1041223, 0x7C, (0, 11, 0),
        KEEP_AREA, new_keep;
    CASTLE_DEED, CASTLE_DEED_CLASS, "Server.Multis.Deeds.CastleDeed", 1041224, 0x7E, (0, 16, 0),
        CASTLE_AREA, new_castle;
    LARGE_PATIO_DEED, LARGE_PATIO_DEED_CLASS, "Server.Multis.Deeds.LargePatioDeed", 1041231, 0x8C, (-4, 7, 0),
        LARGE_PATIO_AREA, new_large_patio_house;
    LARGE_MARBLE_DEED, LARGE_MARBLE_DEED_CLASS, "Server.Multis.Deeds.LargeMarbleDeed", 1041236, 0x96, (-4, 7, 0),
        LARGE_MARBLE_AREA, new_large_marble_house;
    SMALL_TOWER_DEED, SMALL_TOWER_DEED_CLASS, "Server.Multis.Deeds.SmallTowerDeed", 1041237, 0x98, (3, 4, 0),
        SMALL_TOWER_AREA, new_small_tower;
    LOG_CABIN_DEED, LOG_CABIN_DEED_CLASS, "Server.Multis.Deeds.LogCabinDeed", 1041238, 0x9A, (1, 6, 0),
        LOG_CABIN_AREA, new_log_cabin;
    SANDSTONE_PATIO_DEED, SANDSTONE_PATIO_DEED_CLASS, "Server.Multis.Deeds.SandstonePatioDeed", 1041239, 0x9C, (-1, 4, 0),
        SANDSTONE_PATIO_AREA, new_sand_stone_patio;
    VILLA_DEED, VILLA_DEED_CLASS, "Server.Multis.Deeds.VillaDeed", 1041240, 0x9E, (3, 6, 0),
        TWO_STORY_VILLA_AREA, new_two_story_villa;
    STONE_WORKSHOP_DEED, STONE_WORKSHOP_DEED_CLASS, "Server.Multis.Deeds.StoneWorkshopDeed", 1041241, 0xA0, (-1, 4, 0),
        SMALL_SHOP_AREA_2, |w, o| new_small_shop(w, o, 0xA0);
    MARBLE_WORKSHOP_DEED, MARBLE_WORKSHOP_DEED_CLASS, "Server.Multis.Deeds.MarbleWorkshopDeed", 1041242, 0xA2, (-1, 4, 0),
        SMALL_SHOP_AREA_1, |w, o| new_small_shop(w, o, 0xA2);
}

pub fn register(types: &mut TypeRegistry) {
    register_all(types);
}
