//! `BaseContestHouse` (a house whose multi carries its own doors,
//! teleporters and decoration, kept as "fixtures") and the sixteen contest houses.

use indexmap::IndexMap;
use muo_content::items::house_deps::deps;
use muo_data::TileData;
use muo_data::client::multi_data::MultiTileEntry;
use muo_primitives::{Direction, Point3D, Rectangle2D, TimeSpan};
use muo_server::World;
use muo_server::ids::{ItemId, MobileId};
use muo_server::item::{ItemType, ops};
use muo_server::serialization_ext::{ReadEntityExt, WriteEntityExt};
use muo_server::type_registry::TypeRegistry;

use crate::house::doors::{add_door_item_id, set_sign};
use crate::house::save::new_base_house;
use crate::house::{self, BASE_HOUSE, HouseClass, location, map_of};
use crate::teleporter::new_house_teleporter;

/// `ContestHouseType`.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(i32)]
pub enum ContestHouseType {
    Keep = 0,
    Castle = 1,
    Other = 2,
}

impl ContestHouseType {
    fn from_i32(n: i32) -> ContestHouseType {
        match n {
            0 => Self::Keep,
            1 => Self::Castle,
            _ => Self::Other,
        }
    }
}

/// What `BaseContestHouse` adds.
#[derive(Clone, Debug)]
pub struct ContestData {
    pub house_type: ContestHouseType,
    /// `Fixtures`: None is a null list.
    pub fixtures: Option<Vec<ItemId>>,
}

fn cdata(w: &World, id: ItemId) -> Option<ContestData> {
    w.item(id)?.ext.get::<ContestData>().cloned()
}

fn fixtures(w: &World, id: ItemId) -> Vec<ItemId> {
    cdata(w, id).and_then(|d| d.fixtures).unwrap_or_default()
}

/// `AddFixture(item)`.
pub fn add_fixture(w: &mut World, id: ItemId, item: ItemId) {
    if let Some(d) = w.item_mut(id).and_then(|i| i.ext.get_mut::<ContestData>()) {
        d.fixtures.get_or_insert_with(Vec::new).push(item);
    }
}

fn delete(w: &mut World, id: ItemId) {
    if let Some(ty) = w.item(id).map(|i| i.ty) {
        (ty.delete)(w, id);
    }
}

fn contest_area(w: &World, id: ItemId) -> Vec<Rectangle2D> {
    let mcl = house::components(w, id);
    vec![Rectangle2D::new(
        mcl.min().x,
        mcl.min().y,
        mcl.width(),
        mcl.height(),
    )]
}

fn contest_ban_location(w: &World, id: ItemId) -> Point3D {
    let mcl = house::components(w, id);
    Point3D::new(mcl.min().x, mcl.height() - 1 - mcl.center().y, 0)
}

fn is_inside_extra(w: &World, id: ItemId, p: Point3D) -> bool {
    fixtures(w, id).iter().any(|&f| {
        w.item(f)
            .is_some_and(|i| i.ty.is(&crate::teleporter::HOUSE_TELEPORTER) && i.location == p)
    })
}

/// The class table of a contest house: `area` is the class' own `AreaArray` or the multi's box.
macro_rules! contest_class {
    ($name:ident, $short:literal, $area:expr) => {
        pub static $name: HouseClass = HouseClass {
            area: $area,
            base_ban_location: contest_ban_location,
            default_price: |_, _| 0,
            convert_entry: |_, _| None,
            convert_offset: (0, 0, 0),
            get_deed: |_, _| None,
            house_direction: Direction::SOUTH,
            decay_period: TimeSpan::from_days(5),
            is_aos_rules: None,
            is_active: |_, _| true,
            class_name: $short,
            is_inside_extra: Some(is_inside_extra),
        };
    };
}

contest_class!(BASE_CONTEST_HOUSE_CLASS, "BaseContestHouse", contest_area);

/// `new BaseContestHouse(type, multiID, owner, maxLockDown, maxSecure)` of class `ty`.
pub fn new_base_contest_house(
    w: &mut World,
    ty: &'static ItemType,
    house_type: ContestHouseType,
    multi_id: i32,
    owner: MobileId,
    max_lock_down: i32,
    max_secure: i32,
) -> ItemId {
    let id = new_base_house(w, ty, multi_id, Some(owner), max_lock_down, max_secure);
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(ContestData {
            house_type,
            fixtures: None,
        });
    }
    auto_add_fixtures(w, id);
    id
}

/// `SetSign(xOffset, yOffset, zOffset, post)`.
pub fn set_contest_sign(
    w: &mut World,
    id: ItemId,
    x: i32,
    y: i32,
    z: i32,
    post: bool,
    sign_post_id: i32,
) {
    set_sign(w, id, x, y, z);
    let l = location(w, id);
    let map = map_of(w, id);
    let st = new_static(w, 0xB9E);
    if let Some(h) = st {
        ops::move_to_world_map(w, h, Point3D::new(l.x + x, l.y + y, l.z + z), map);
        add_fixture(w, id, h);
    }
    if post {
        let sp = new_static(w, sign_post_id);
        if let Some(p) = sp {
            ops::move_to_world_map(w, p, Point3D::new(l.x + x, l.y + y - 1, l.z + z), map);
            add_fixture(w, id, p);
        }
    }
}

fn new_static(w: &mut World, graphic: i32) -> Option<ItemId> {
    let s = muo_content::factory::make_item(w, "Server.Items.Static")?;
    if let Some(ty) = w.item(s).map(|i| i.ty) {
        (ty.set_item_id)(w, s, graphic);
    }
    Some(s)
}

/// `AddTeleporters(id, offset1, offset2)`.
pub fn add_teleporters(w: &mut World, house_id: ItemId, id: i32, o1: Point3D, o2: Point3D) {
    let t1 = new_house_teleporter(w, id, None);
    let t2 = new_house_teleporter(w, id, None);
    for (a, b) in [(t1, t2), (t2, t1)] {
        if let Some(d) = w
            .item_mut(a)
            .and_then(|i| i.ext.get_mut::<crate::teleporter::TeleporterData>())
        {
            d.target = Some(b);
        }
    }
    let l = location(w, house_id);
    let map = map_of(w, house_id);
    ops::move_to_world_map(w, t1, Point3D::new(l.x + o1.x, l.y + o1.y, o1.z), map);
    ops::move_to_world_map(w, t2, Point3D::new(l.x + o2.x, l.y + o2.y, o2.z), map);
    add_fixture(w, house_id, t1);
    add_fixture(w, house_id, t2);
}

/// `AutoAddFixtures()`: the multi's doors, teleporters and decoration.
pub fn auto_add_fixtures(w: &mut World, id: ItemId) {
    let item_id = w.item(id).map_or(0, |i| i.item_id);
    let entries: Vec<MultiTileEntry> = w.maps.multi_data.get_components(item_id).list.clone();
    let mut teleporters: IndexMap<i32, Vec<MultiTileEntry>> = IndexMap::new();

    for entry in entries.iter().filter(|e| e.flags.is_empty()) {
        let eid = i32::from(entry.item_id);
        if (0x181D..=0x1828).contains(&eid) {
            teleporters.entry(eid).or_default().push(*entry);
        } else {
            let is_door = w
                .maps
                .tile_data
                .item(usize::from(entry.item_id) & TileData::MAX_ITEM_VALUE)
                .door();
            let (ox, oy, oz) = (
                i32::from(entry.offset_x),
                i32::from(entry.offset_y),
                i32::from(entry.offset_z),
            );
            if is_door {
                add_door_item_id(w, id, None, eid, ox, oy, oz); // door
            } else if let Some(st) = new_static(w, eid) {
                let l = location(w, id);
                let map = map_of(w, id);
                ops::move_to_world_map(w, st, Point3D::new(l.x + ox, l.y + oy, oz), map);
                add_fixture(w, id, st);
            }
        }
    }

    let doors = house::d(w, id).doors;
    if let Some(link) = deps(w).and_then(|x| x.door_set_link) {
        for &door in &doors {
            for &check in doors.iter().filter(|&&d| d != door) {
                let near = w.item(door).is_some_and(|i| {
                    let loc = location(w, check);
                    (i.ty.in_range_point3d)(w, door, loc, 1)
                });
                if near {
                    link(w, door, check);
                    link(w, check, door);
                }
            }
        }
    }

    for (key, value) in teleporters {
        if value.len() > 2 {
            tracing::warn!("More than 2 teleporters detected for {key:X}!");
        } else if value.len() <= 1 {
            tracing::warn!("1 or less teleporters detected for {key:X}!");
            continue;
        }
        let p = |e: &MultiTileEntry| {
            Point3D::new(
                i32::from(e.offset_x),
                i32::from(e.offset_y),
                i32::from(e.offset_z),
            )
        };
        add_teleporters(w, id, key, p(&value[0]), p(&value[1]));
    }
}

fn contest_on_location_change(w: &mut World, id: ItemId, old: Point3D) {
    (BASE_HOUSE.on_location_change)(w, id, old);
    let l = location(w, id);
    let (x, y, z) = (l.x - old.x, l.y - old.y, l.z - old.z);
    for item in fixtures(w, id) {
        let p = location(w, item);
        ops::set_location(w, item, Point3D::new(p.x + x, p.y + y, p.z + z));
    }
}

fn contest_on_map_change(w: &mut World, id: ItemId) {
    (BASE_HOUSE.on_map_change)(w, id);
    let map = map_of(w, id);
    for item in fixtures(w, id) {
        ops::set_map(w, item, map);
    }
}

fn contest_on_after_delete(w: &mut World, id: ItemId) {
    (BASE_HOUSE.on_after_delete)(w, id);
    for item in fixtures(w, id) {
        delete(w, item);
    }
}

/// `Server.Multis.BaseContestHouse`.
pub static BASE_CONTEST_HOUSE: ItemType = ItemType {
    full_name: "Server.Multis.BaseContestHouse",
    parent: Some(&BASE_HOUSE),
    construct: None,
    family: Some(&BASE_CONTEST_HOUSE_CLASS),
    on_location_change: contest_on_location_change,
    on_map_change: contest_on_map_change,
    on_after_delete: contest_on_after_delete,
    serialize: |w, id, out| {
        (BASE_HOUSE.serialize)(w, id, out);
        out.write_i32(1); // version
        let d = cdata(w, id).unwrap_or(ContestData {
            house_type: ContestHouseType::Other,
            fixtures: None,
        });
        out.write_enum(4, d.house_type as i64);
        match d.fixtures {
            Some(list) => {
                out.write_bool(true);
                let list: Vec<ItemId> = list
                    .into_iter()
                    .filter(|&i| w.item(i).is_some_and(|x| !x.deleted()))
                    .collect();
                out.write_encoded_int(list.len() as i32);
                for i in list {
                    out.write_item(w, Some(i));
                }
            }
            None => out.write_bool(false),
        }
    },
    deserialize: |w, id, r| {
        (BASE_HOUSE.deserialize)(w, id, r)?;
        let version = r.read_i32()?;
        let mut d = ContestData {
            house_type: ContestHouseType::Keep,
            fixtures: None,
        };
        if version < 1 {
            d.house_type = ContestHouseType::from_i32(r.read_i32()?);
            let count = r.read_i32()?;
            for _ in 0..count {
                if let Some(i) = r.read_item(w)? {
                    d.fixtures.get_or_insert_with(Vec::new).push(i);
                }
            }
        } else {
            d.house_type = ContestHouseType::from_i32(r.read_enum(4)? as i32);
            if r.read_bool()? {
                let n = r.read_encoded_int()?;
                let mut list = Vec::new();
                for _ in 0..n {
                    list.extend(r.read_item(w)?);
                }
                d.fixtures = Some(list);
            }
        }
        if let Some(i) = w.item_mut(id) {
            i.ext.insert(d);
        }
        Ok(())
    },
    ..BASE_HOUSE
};

macro_rules! contest_houses {
    ($($name:ident, $cls:ident, $full:literal, $short:literal, $area:expr, $kind:ident, $id:literal, $lock:literal, $secure:literal,
       ($sx:literal, $sy:literal, $sz:literal), $post:literal, $post_id:expr, $new:ident;)*) => {
        $(
            contest_class!($cls, $short, $area);

            #[doc = concat!("`", $full, "`.")]
            pub static $name: ItemType = ItemType {
                full_name: $full,
                parent: Some(&BASE_CONTEST_HOUSE),
                construct: None,
                family: Some(&$cls),
                serialize: |w, id, out| {
                    (BASE_CONTEST_HOUSE.serialize)(w, id, out);
                    out.write_i32(0);
                },
                deserialize: |w, id, r| {
                    (BASE_CONTEST_HOUSE.deserialize)(w, id, r)?;
                    r.read_i32()?;
                    Ok(())
                },
                ..BASE_CONTEST_HOUSE
            };

            #[doc = concat!("`new ", $short, "(owner)`.")]
            pub fn $new(w: &mut World, owner: MobileId) -> ItemId {
                let h = new_base_contest_house(w, &$name, ContestHouseType::$kind, $id, owner, $lock, $secure);
                set_contest_sign(w, h, $sx, $sy, $sz, $post, $post_id);
                h
            }
        )*

        pub fn register(types: &mut TypeRegistry) {
            types.register_item(&BASE_CONTEST_HOUSE, &[]);
            $(types.register_item(&$name, &[]);)*
        }
    };
}

fn trinsic_area(_: &World, _: ItemId) -> Vec<Rectangle2D> {
    vec![
        Rectangle2D::new(-11, -11, 23, 23),
        Rectangle2D::new(-10, 13, 6, 1),
        Rectangle2D::new(-2, 13, 6, 1),
        Rectangle2D::new(6, 13, 7, 1),
    ]
}

fn gothic_area(_: &World, _: ItemId) -> Vec<Rectangle2D> {
    vec![
        Rectangle2D::new(-15, -15, 31, 31),
        Rectangle2D::new(-14, 16, 11, 1),
        Rectangle2D::new(-2, 16, 6, 1),
        Rectangle2D::new(5, 16, 11, 1),
    ]
}

fn feudal_area(_: &World, _: ItemId) -> Vec<Rectangle2D> {
    vec![
        Rectangle2D::new(-15, -15, 31, 31),
        Rectangle2D::new(5, 16, 1, 1),
        Rectangle2D::new(7, 16, 4, 1),
        Rectangle2D::new(12, 16, 1, 1),
    ]
}

fn grimswind_area(_: &World, _: ItemId) -> Vec<Rectangle2D> {
    vec![
        Rectangle2D::new(-15, -15, 31, 31),
        Rectangle2D::new(-14, 16, 9, 1),
        Rectangle2D::new(-3, 16, 8, 1),
        Rectangle2D::new(7, 16, 9, 1),
    ]
}

// `TraditionalKeep.AreaArray` is `TrinsicKeep`'s with the last rectangle at x 6 (the same numbers).
// `OkinawaSweetDreamCastle` has an `AreaArray` but `Area` is not overridden, so it uses the multi's box.

contest_houses! {
    TRINSIC_KEEP, TRINSIC_KEEP_CLASS, "Server.Multis.TrinsicKeep", "TrinsicKeep", trinsic_area, Keep, 0x147E, 2113, 18, (-11, 13, 7), false, 9, new_trinsic_keep;
    GOTHIC_ROSE_CASTLE, GOTHIC_ROSE_CASTLE_CLASS, "Server.Multis.GothicRoseCastle", "GothicRoseCastle", gothic_area, Castle, 0x147F, 3281, 28, (-15, 16, 7), false, 9, new_gothic_rose_castle;
    ELSA_CASTLE, ELSA_CASTLE_CLASS, "Server.Multis.ElsaCastle", "ElsaCastle", contest_area, Castle, 0x1480, 3281, 28, (-15, 16, 7), false, 9, new_elsa_castle;
    SPIRES, SPIRES_CLASS, "Server.Multis.Spires", "Spires", contest_area, Castle, 0x1481, 3281, 28, (-15, 16, 7), false, 9, new_spires;
    CASTLE_OF_OCEANIA, CASTLE_OF_OCEANIA_CLASS, "Server.Multis.CastleOfOceania", "CastleOfOceania", contest_area, Castle, 0x1482, 3281, 28, (-15, 16, 7), false, 9, new_castle_of_oceania;
    FEUDAL_CASTLE, FEUDAL_CASTLE_CLASS, "Server.Multis.FeudalCastle", "FeudalCastle", feudal_area, Castle, 0x1483, 3281, 28, (-15, 16, 7), true, 9, new_feudal_castle;
    ROBINS_NEST, ROBINS_NEST_CLASS, "Server.Multis.RobinsNest", "RobinsNest", contest_area, Keep, 0x1484, 2113, 18, (-11, 13, 7), false, 9, new_robins_nest;
    TRADITIONAL_KEEP, TRADITIONAL_KEEP_CLASS, "Server.Multis.TraditionalKeep", "TraditionalKeep", trinsic_area, Keep, 0x1485, 2113, 18, (-11, 13, 7), false, 9, new_traditional_keep;
    VILLA_CROWLEY, VILLA_CROWLEY_CLASS, "Server.Multis.VillaCrowley", "VillaCrowley", contest_area, Keep, 0x1486, 2113, 18, (-11, 13, 7), true, 9, new_villa_crowley;
    DARKTHORN_KEEP, DARKTHORN_KEEP_CLASS, "Server.Multis.DarkthornKeep", "DarkthornKeep", contest_area, Keep, 0x1487, 2113, 18, (-11, 13, 7), false, 9, new_darkthorn_keep;
    SANDALWOOD_KEEP, SANDALWOOD_KEEP_CLASS, "Server.Multis.SandalwoodKeep", "SandalwoodKeep", contest_area, Keep, 0x1488, 2113, 18, (-11, 13, 7), true, 353, new_sandalwood_keep;
    CASA_MOGA, CASA_MOGA_CLASS, "Server.Multis.CasaMoga", "CasaMoga", contest_area, Keep, 0x1489, 2113, 18, (-11, 13, 7), false, 9, new_casa_moga;
    ROBINS_ROOST, ROBINS_ROOST_CLASS, "Server.Multis.RobinsRoost", "RobinsRoost", contest_area, Castle, 0x148A, 3281, 28, (-15, 16, 7), true, 9, new_robins_roost;
    CAMELOT, CAMELOT_CLASS, "Server.Multis.Camelot", "Camelot", contest_area, Castle, 0x148B, 3281, 28, (-15, 16, 7), false, 9, new_camelot;
    LACRIMAE_IN_CAELO, LACRIMAE_IN_CAELO_CLASS, "Server.Multis.LacrimaeInCaelo", "LacrimaeInCaelo", contest_area, Castle, 0x148C, 3281, 28, (-15, 16, 7), false, 9, new_lacrimae_in_caelo;
    OKINAWA_SWEET_DREAM_CASTLE, OKINAWA_SWEET_DREAM_CASTLE_CLASS, "Server.Multis.OkinawaSweetDreamCastle", "OkinawaSweetDreamCastle", contest_area, Castle, 0x148D, 3281, 28, (-15, 16, 7), true, 9, new_okinawa_sweet_dream_castle;
    THE_SANDSTONE_CASTLE, THE_SANDSTONE_CASTLE_CLASS, "Server.Multis.TheSandstoneCastle", "TheSandstoneCastle", contest_area, Castle, 0x148E, 3281, 28, (-15, 16, 7), true, 9, new_the_sandstone_castle;
    GRIMSWIND_SISTERS, GRIMSWIND_SISTERS_CLASS, "Server.Multis.GrimswindSisters", "GrimswindSisters", grimswind_area, Castle, 0x148F, 3281, 28, (-15, 16, 7), false, 9, new_grimswind_sisters;
}
