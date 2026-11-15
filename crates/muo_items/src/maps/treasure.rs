//! `TreasureMap`.
//!
//! The digging half is here too: `OnBeginDig` (set as `HarvestHooks::treasure_map_begin_dig` by the harvest
//! engine), `DigTarget`, `DigTimer`, `TreasureChestDirt`, the guardian `Spawn`s and the `Decode`/`Open`/`Dig`
//! context menu entries.
//!
//! The chest is a `TreasureMapChest` (`last/chests.rs`): it is made through that class's constructor table
//! (`owner, level, temporary`) and [`TreasureChestHooks`] (`Temporary`, `AddToGuardians`). A spawn class that is not
//! registered spawns nothing.

use std::sync::OnceLock;

use muo_primitives::{MessageType, Point2D, Point3D, Rectangle2D, TimeSpan, utility};
use muo_server::World;
use muo_server::context_menu_entry::ContextMenuEntry;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::ItemType;
use muo_server::item::{LootType, base, deps, messages, ops};
use muo_server::map::MapId;
use muo_server::mobile::AccessLevel;
use muo_server::serialization_ext::{ReadEntityExt, WriteEntityExt};
use muo_server::skills::SkillName;
use muo_server::type_registry::TypeRegistry;

use super::{
    MAP_ITEM, MapItemClass, add_world_pin, data_mut as map_data_mut, deserialize_map_item,
    display_to_base, map_item_new, serialize_map_item,
};
use crate::misc::{random_min_max, read_version, write_version};

/// `TreasureMap.LootChance`.
pub const LOOT_CHANCE: f64 = 0.01;

/// What `TreasureMap` adds to a `MapItem`.
#[derive(Clone, Debug, Default)]
pub struct TreasureData {
    pub completed_by: Option<MobileId>,
    pub level: i32,
    pub completed: bool,
    pub decoder: Option<MobileId>,
    pub chest_map: Option<MapId>,
    pub chest_location: Point2D,
}

pub fn treasure(w: &World, id: ItemId) -> TreasureData {
    super::data(w, id)
        .and_then(|d| d.treasure.clone())
        .unwrap_or_default()
}

pub fn treasure_mut(w: &mut World, id: ItemId) -> Option<&mut TreasureData> {
    super::data_mut(w, id).map(|d| d.treasure.get_or_insert_with(Default::default))
}

// ---- locations ----

struct Locations {
    all: Vec<Point2D>,
    haven: Vec<Point2D>,
}

static LOCATIONS: OnceLock<Locations> = OnceLock::new();

/// `IsInHavenIsland`.
pub fn is_in_haven_island(loc: Point2D) -> bool {
    loc.x >= 3314 && loc.x <= 3814 && loc.y >= 2345 && loc.y <= 3095
}

/// `LoadLocations`: `Data/treasure.cfg`, one `x y` per line; a bad line is skipped.
fn load_locations(w: &World) -> &'static Locations {
    LOCATIONS.get_or_init(|| {
        let mut all = Vec::new();
        let mut haven = Vec::new();
        if let Ok(text) = std::fs::read_to_string(w.core.base_directory.join("Data/treasure.cfg")) {
            for line in text.lines() {
                let mut split = line.split(' ');
                let (Some(x), Some(y)) = (split.next(), split.next()) else {
                    continue;
                };
                let (Ok(x), Ok(y)) = (x.trim().parse::<i32>(), y.trim().parse::<i32>()) else {
                    continue;
                };
                let loc = Point2D::new(x, y);
                all.push(loc);
                if is_in_haven_island(loc) {
                    haven.push(loc);
                }
            }
        }
        Locations { all, haven }
    })
}

/// `GetRandomLocation()`.
pub fn get_random_location(w: &mut World) -> Point2D {
    let list = &load_locations(w).all;
    utility::random_element(&mut w.rng, list)
        .copied()
        .unwrap_or(Point2D::ZERO)
}

/// `GetRandomHavenLocation()`.
pub fn get_random_haven_location(w: &mut World) -> Point2D {
    let list = &load_locations(w).haven;
    utility::random_element(&mut w.rng, list)
        .copied()
        .unwrap_or(Point2D::ZERO)
}

// ---- construction ----

/// `TreasureMap(level, map)`.
pub fn treasure_map_new(
    w: &mut World,
    ty: &'static ItemType,
    level: i32,
    map: Option<MapId>,
) -> ItemId {
    let id = map_item_new(w, ty, None);
    let chest_location = if level == 0 {
        get_random_haven_location(w)
    } else {
        get_random_location(w)
    };
    if let Some(d) = map_data_mut(w, id) {
        d.treasure = Some(TreasureData {
            level,
            chest_map: map,
            chest_location,
            ..Default::default()
        });
        d.width = 300;
        d.height = 300;
    }
    const WIDTH: i32 = 600;
    const HEIGHT: i32 = 600;
    let mut x1 = chest_location.x - random_min_max(w, WIDTH / 4, WIDTH / 4 * 3);
    let mut y1 = chest_location.y - random_min_max(w, HEIGHT / 4, HEIGHT / 4 * 3);
    if x1 < 0 {
        x1 = 0;
    }
    if y1 < 0 {
        y1 = 0;
    }
    let mut x2 = x1 + WIDTH;
    let mut y2 = y1 + HEIGHT;
    if x2 > 5119 {
        x2 = 5119;
    }
    if y2 > 4095 {
        y2 = 4095;
    }
    x1 = x2 - WIDTH;
    y1 = y2 - HEIGHT;
    if let Some(d) = map_data_mut(w, id) {
        d.bounds = Rectangle2D::new(x1, y1, WIDTH, HEIGHT);
        d.protected = true;
    }
    add_world_pin(w, id, chest_location.x, chest_location.y);
    id
}

// ---- behavior ----

fn check_young(w: &mut World, id: ItemId, from: MobileId) -> bool {
    if w.mobile(from)
        .is_some_and(|m| m.access_level >= AccessLevel::GameMaster)
    {
        return true;
    }
    if muo_content::player_mobile::young(w, from) {
        return true;
    }
    if treasure(w, id).decoder == Some(from) {
        if let Some(t) = treasure_mut(w, id) {
            t.level = 1;
        }
        ops::invalidate_properties(w, id);
        deps::send_localized_message(w, from, 1046446); // This is now a level one treasure map.
        return true;
    }
    false
}

fn min_skill_level(level: i32) -> f64 {
    match level {
        1 => -3.0,
        2 => 41.0,
        3 => 51.0,
        4 => 61.0,
        5 | 6 => 70.0,
        _ => 0.0,
    }
}

fn has_required_skill(w: &World, id: ItemId, from: MobileId) -> bool {
    muo_server::skills::value_ref(w, from, SkillName::Cartography)
        >= min_skill_level(treasure(w, id).level)
}

/// `TreasureMap.Decode(from)`.
pub fn decode(w: &mut World, id: ItemId, from: MobileId) {
    let t = treasure(w, id);
    if t.completed || t.decoder.is_some() {
        return;
    }
    if t.level == 0 {
        if !check_young(w, id, from) {
            deps::send_localized_message(w, from, 1046447); // Only a young player may use this treasure map.
            return;
        }
    } else {
        let min_skill = min_skill_level(t.level);
        if muo_server::skills::value_ref(w, from, SkillName::Cartography) < min_skill {
            deps::send_localized_message(w, from, 503013); // The map is too difficult to attempt to decode.
        }
        let max_skill = min_skill + 60.0;
        if !muo_server::mobile::part2::stats::check_skill(
            w,
            from,
            SkillName::Cartography,
            min_skill,
            max_skill,
        ) {
            // You fail to make anything of the map.
            muo_server::mobile::part2::messages::local_overhead_message_number(
                w,
                from,
                MessageType::REGULAR,
                0x3B2,
                503018,
                "",
            );
            return;
        }
    }
    // You successfully decode a treasure map!
    muo_server::mobile::part2::messages::local_overhead_message_number(
        w,
        from,
        MessageType::REGULAR,
        0x3B2,
        503019,
        "",
    );
    if let Some(t) = treasure_mut(w, id) {
        t.decoder = Some(from);
    }
    ops::invalidate_properties(w, id);
    if muo_content::core::aos(w) {
        ops::set_loot_type(w, id, LootType::Blessed);
    }
    super::display_to(w, id, from);
}

fn display_to(w: &mut World, id: ItemId, from: MobileId) {
    let t = treasure(w, id);
    if t.completed {
        messages::send_localized_message_to(w, id, from, 503014, ""); // This treasure hunt has already been completed.
    } else if t.level == 0 && !check_young(w, id, from) {
        deps::send_localized_message(w, from, 1046447); // Only a young player may use this treasure map.
        return;
    } else if t.decoder != Some(from) && !has_required_skill(w, id, from) {
        // You did not decode this map and have no clue where to look for the treasure.
        deps::send_localized_message(w, from, 503031);
        return;
    } else {
        // The treasure is marked by the red pin. Grab a shovel and go dig it up!
        messages::send_localized_message_to(w, id, from, 503017, "");
    }
    muo_server::mobile::part2::effects::play_sound(w, from, 0x249);
    display_to_base(w, id, from);
}

fn label_number(w: &World, id: ItemId) -> i32 {
    let t = treasure(w, id);
    if t.decoder.is_some() {
        return if t.level == 6 {
            1063453
        } else {
            1041516 + t.level
        };
    }
    if t.level == 6 {
        1063452
    } else {
        1041510 + t.level
    }
}

fn raw_name(w: &World, m: Option<MobileId>) -> String {
    m.and_then(|m| muo_server::mobile::part1::props_b::raw_name(w, m))
        .unwrap_or_else(|| "someone".to_string())
}

// ---- serialization ----

fn after_deserialization(w: &mut World, id: ItemId) {
    if muo_content::core::aos(w)
        && treasure(w, id).decoder.is_some()
        && w.item(id).is_some_and(|i| i.loot_type == LootType::Regular)
    {
        ops::set_loot_type(w, id, LootType::Blessed);
    }
}

/// `Server.Items.TreasureMap`.
pub static TREASURE_MAP_CLASS: MapItemClass = MapItemClass {
    display_to,
    ..super::MAP_ITEM_CLASS
};

pub static TREASURE_MAP: ItemType = ItemType {
    full_name: "Server.Items.TreasureMap",
    parent: Some(&MAP_ITEM),
    construct: None,
    make: Some(|w| treasure_map_new(w, &TREASURE_MAP, 0, None)), // `new TreasureMap(0, null)`; `[add` uses `TREASURE_MAP_CTORS`
    family: Some(&TREASURE_MAP_CLASS),
    get_context_menu_entries: treasure_context_entries,
    label_number: |w, id| label_number(w, id),
    on_double_click: |w, id, from| {
        let loc = ops::get_world_location(w, id);
        if !deps::mobile_in_range(w, from, loc, 2) {
            // I can't reach that.
            muo_server::mobile::part2::messages::local_overhead_message_number(
                w,
                from,
                MessageType::REGULAR,
                0x3B2,
                1019045,
                "",
            );
            return;
        }
        let t = treasure(w, id);
        if !t.completed && t.decoder.is_none() {
            decode(w, id, from);
        } else {
            super::display_to(w, id, from);
        }
    },
    get_properties: |w, id, list| {
        base::get_properties_base(w, id, list);
        let t = treasure(w, id);
        list.add(if t.chest_map == Some(MapId::FELUCCA) {
            1041502
        } else {
            1041503
        }); // for somewhere in Felucca : Trammel
        if t.completed {
            list.add_str(1041507, &raw_name(w, t.completed_by)); // completed by ~1_val~
        }
    },
    on_single_click: |w, id, from| {
        let t = treasure(w, id);
        let facet = if t.chest_map == Some(MapId::FELUCCA) {
            1041502
        } else {
            1041503
        };
        if t.completed {
            if let Some(ns) = deps::net_state_of(w, from) {
                let (serial, graphic) =
                    w.item(id).map_or((muo_primitives::Serial::new(0), 0), |i| {
                        (i.serial, i.item_id)
                    });
                let affix = format!(" completed by {}", raw_name(w, t.completed_by));
                muo_server::packets::message::send_message_localized_affix(
                    w,
                    ns,
                    serial,
                    graphic,
                    muo_server::packets::message::MessageType::LABEL,
                    0x3B2,
                    3,
                    1048030,
                    "",
                    muo_server::packets::message::AffixType::APPEND,
                    &affix,
                    "",
                );
            }
        } else if t.decoder.is_some() {
            let number = if t.level == 6 {
                1063453
            } else {
                1041516 + t.level
            };
            ops::label_to_number(w, id, from, number, "");
        } else if t.level == 6 {
            ops::label_to_number(
                w,
                id,
                from,
                1041522,
                &format!("#{}\t \t#{}", 1063452, facet),
            );
        } else {
            ops::label_to_number(
                w,
                id,
                from,
                1041522,
                &format!("#{}\t \t#{}", 1041510 + t.level, facet),
            );
        }
    },
    serialize: |w, id, out| {
        serialize_map_item(w, id, out);
        write_version(out, false);
        let t = treasure(w, id);
        out.write_mobile(w, t.completed_by);
        out.write_i32(t.level);
        out.write_bool(t.completed);
        out.write_mobile(w, t.decoder);
        out.write_map(t.chest_map);
        out.write_point2d(t.chest_location);
    },
    deserialize: |w, id, r| {
        deserialize_map_item(w, id, r)?;
        read_version(r, false)?;
        let t = TreasureData {
            completed_by: r.read_mobile(w)?,
            level: r.read_i32()?,
            completed: r.read_bool()?,
            decoder: r.read_mobile(w)?,
            chest_map: r.read_map()?,
            chest_location: r.read_point2d()?,
        };
        if let Some(d) = map_data_mut(w, id) {
            d.treasure = Some(t);
        }
        after_deserialization(w, id);
        Ok(())
    },
    ..MAP_ITEM
};

// ---- digging ----

/// What the dig asks of `TreasureMapChest`: `chest.Temporary = value` and `chest.AddToGuardians(bc)`. Unset:
/// nothing happens.
#[derive(Default)]
pub struct TreasureChestHooks {
    pub set_temporary: Option<fn(&mut World, ItemId, bool)>,
    pub add_to_guardians: Option<fn(&mut World, ItemId, MobileId)>,
}

const SPAWN_TYPES: [&[&str]; 7] = [
    &["HeadlessOne", "Skeleton"],
    &["Mongbat", "Ratman", "HeadlessOne", "Skeleton", "Zombie"],
    &[
        "OrcishMage",
        "Gargoyle",
        "Gazer",
        "HellHound",
        "EarthElemental",
    ],
    &[
        "Lich",
        "OgreLord",
        "DreadSpider",
        "AirElemental",
        "FireElemental",
    ],
    &[
        "DreadSpider",
        "LichLord",
        "Daemon",
        "ElderGazer",
        "OgreLord",
    ],
    &[
        "LichLord",
        "Daemon",
        "ElderGazer",
        "PoisonElemental",
        "BloodElemental",
    ],
    &[
        "AncientWyrm",
        "Balron",
        "BloodElemental",
        "PoisonElemental",
        "Titan",
    ],
];

/// `TreasureMap.Spawn(level, p, guardian)`.
pub fn spawn_creature(w: &mut World, level: i32, p: Point3D, guardian: bool) -> Option<MobileId> {
    if level < 0 || level as usize >= SPAWN_TYPES.len() {
        return None;
    }
    let list = SPAWN_TYPES[level as usize];
    let pick = utility::random(&mut w.rng, list.len() as i32) as usize;
    let bc = muo_content::factory::make_mobile(w, &format!("Server.Mobiles.{}", list[pick]))?;
    muo_content::creature::control::set_home(w, bc, p);
    muo_content::creature::data::edit(w, bc, |c| c.range_home = 5);
    if guardian && level == 0 {
        muo_server::mobile::base::set_name(w, bc, Some("a chest guardian".to_string()));
        muo_server::mobile::base::set_hue(w, bc, 0x835);
    }
    Some(bc)
}

/// `TreasureMap.Spawn(level, p, map, target, guardian)`.
pub fn spawn(
    w: &mut World,
    level: i32,
    p: Point3D,
    map: Option<MapId>,
    target: Option<MobileId>,
    guardian: bool,
) -> Option<MobileId> {
    let map = map?;
    let c = spawn_creature(w, level, p, guardian)?;
    let mut spawned = false;
    let mut i = 0;
    while !spawned && i < 10 {
        let x = p.x - 3 + utility::random(&mut w.rng, 7);
        let y = p.y - 3 + utility::random(&mut w.rng, 7);
        if muo_server::map::fit::can_spawn_mobile_at(w, map, x, y, p.z) {
            muo_server::mobile::base::move_to_world(w, c, Point3D::new(x, y, p.z), map);
            spawned = true;
        } else {
            let z = w.maps.get_average_z(map, x, y);
            if muo_server::map::fit::can_spawn_mobile_at(w, map, x, y, z) {
                muo_server::mobile::base::move_to_world(w, c, Point3D::new(x, y, z), map);
                spawned = true;
            }
        }
        i += 1;
    }
    if !spawned {
        muo_server::mobile::base::delete(w, c);
        return None;
    }
    if let Some(t) = target {
        muo_content::mx::set_combatant(w, c, Some(t));
    }
    Some(c)
}

/// `TreasureMap.HasDiggingTool(m)`: a mining tool anywhere in the backpack.
pub fn has_digging_tool(w: &World, m: MobileId) -> bool {
    let Some(pack) = w.mobile(m).and_then(|x| x.backpack) else {
        return false;
    };
    let mut queue = std::collections::VecDeque::from([pack]);
    while let Some(c) = queue.pop_front() {
        let Some(it) = w.item(c) else { continue };
        for &child in &it.items {
            if muo_content::items::tool::is_tool(w, child)
                && muo_content::items::tool::class(w, child).harvest_system == "Mining"
            {
                return true;
            }
            queue.push_back(child);
        }
    }
    false
}

/// `TreasureMap.OnBeginDig(from)`.
pub fn on_begin_dig(w: &mut World, id: ItemId, from: MobileId) {
    let t = treasure(w, id);
    let map = w.mobile(from).and_then(|m| m.map);
    if t.completed {
        deps::send_localized_message(w, from, 503028); // The treasure for this map has already been found.
    } else if t.level == 0 && !check_young(w, id, from) {
        deps::send_localized_message(w, from, 1046447); // Only a young player may use this treasure map.
    } else if t.decoder != Some(from) && !has_required_skill(w, id, from) {
        // You did not decode this map and have no clue where to look for the treasure.
        deps::send_localized_message(w, from, 503031);
    } else if !muo_server::mobile::part1::combat::can_begin_action(w, from, "TreasureMap") {
        deps::send_localized_message(w, from, 503020); // You are already digging treasure.
    } else if map != t.chest_map {
        deps::send_localized_message(w, from, 1010479); // You seem to be in the right place, but may be on the wrong facet!
    } else {
        deps::send_localized_message(w, from, 503033); // Where do you wish to dig?
        muo_server::target::begin_target(
            w,
            from,
            6,
            true,
            muo_server::target::TargetFlags::NONE,
            move |w, from, t| dig_target(w, id, from, t),
        );
    }
}

fn dig_target(w: &mut World, id: ItemId, from: MobileId, targeted: muo_server::target::Targeted) {
    use muo_server::target::Targeted;
    if ops::is_deleted(w, id) {
        return;
    }
    let t = treasure(w, id);
    let map = t.chest_map;
    let from_map = w.mobile(from).and_then(|m| m.map);
    if t.completed {
        deps::send_localized_message(w, from, 503028);
    } else if t.decoder != Some(from) && !has_required_skill(w, id, from) {
        deps::send_localized_message(w, from, 503031);
    } else if !muo_server::mobile::part1::combat::can_begin_action(w, from, "TreasureMap") {
        deps::send_localized_message(w, from, 503020);
    } else if !has_digging_tool(w, from) {
        deps::send_localized_message(w, from, 1114416); // You must have a digging tool to dig for treasure.
    } else if from_map != map {
        deps::send_localized_message(w, from, 1010479);
    } else {
        let targ3d = match targeted {
            Targeted::Entity(EntityId::Item(i)) => ops::get_world_location(w, i),
            Targeted::Entity(EntityId::Mobile(m)) => {
                w.mobile(m).map_or(Point3D::default(), |x| x.location)
            }
            Targeted::Land(l) => l.location,
            Targeted::Static(s) => s.location,
        };
        let skill = muo_server::skills::value_ref(w, from, SkillName::Mining);
        let max_range = if skill >= 100.0 {
            4
        } else if skill >= 81.0 {
            3
        } else if skill >= 51.0 {
            2
        } else {
            1
        };
        let loc = t.chest_location;
        let chest3d0 = Point3D::new(loc.x, loc.y, 0);
        if utility::in_range_3d(targ3d, chest3d0, max_range) {
            let at = w.mobile(from).map_or(Point3D::default(), |m| m.location);
            if at.x == loc.x && at.y == loc.y {
                // The chest can't be dug up because you are standing on top of it.
                deps::send_localized_message(w, from, 503030);
            } else if let Some(map) = map {
                let z = w.maps.get_average_z(map, loc.x, loc.y);
                if !muo_server::map::fit::can_fit(w, map, loc.x, loc.y, z, 16, true, true, true) {
                    // You have found the treasure chest but something is keeping it from being dug up.
                    deps::send_localized_message(w, from, 503021);
                } else if muo_server::mobile::part1::combat::begin_action(w, from, "TreasureMap") {
                    start_dig_timer(w, from, id, Point3D::new(loc.x, loc.y, z), map);
                } else {
                    deps::send_localized_message(w, from, 503020);
                }
            }
        } else if t.level > 0 {
            if utility::in_range_3d(targ3d, chest3d0, 8) {
                // We're close, but not quite
                deps::send_localized_message(w, from, 503032); // You dig and dig but no treasure seems to be here.
            } else {
                deps::send_localized_message(w, from, 503035); // You dig and dig but fail to find any treasure.
            }
        } else if utility::in_range_3d(targ3d, chest3d0, 8) {
            muo_server::mobile::part2::messages::send_ascii_message_hue(
                w,
                from,
                0x44,
                "The treasure chest is very close!",
            );
        } else {
            use muo_primitives::Direction as D;
            let dir = utility::get_direction_3d(targ3d, chest3d0);
            let s_dir = match dir {
                D::NORTH => "north",
                D::RIGHT => "northeast",
                D::EAST => "east",
                D::DOWN => "southeast",
                D::SOUTH => "south",
                D::LEFT => "southwest",
                D::WEST => "west",
                _ => "northwest",
            };
            muo_server::mobile::part2::messages::send_ascii_message_hue(
                w,
                from,
                0x44,
                &format!("Try looking for the treasure chest more to the {s_dir}."),
            );
        }
    }
}

/// `TreasureChestDirt()`.
pub fn dirt_new(w: &mut World) -> ItemId {
    let id =
        crate::simple::from_template(w, &TREASURE_CHEST_DIRT, "Server.Items.TreasureChestDirt");
    muo_server::timer::start(w, TimeSpan::from_mins(2), TimeSpan::ZERO, 1, move |w| {
        if let Some(ty) = w.item(id).map(|i| i.ty) {
            (ty.delete)(w, id);
        }
    });
    id
}

fn set_graphic(w: &mut World, id: ItemId, graphic: i32) {
    if let Some(ty) = w.item(id).map(|i| i.ty) {
        (ty.set_item_id)(w, id, graphic);
    }
}

fn delete_item(w: &mut World, id: ItemId) {
    if let Some(ty) = w.item(id).map(|i| i.ty) {
        (ty.delete)(w, id);
    }
}

/// `DigTimer`.
fn start_dig_timer(w: &mut World, from: MobileId, map_item: ItemId, location: Point3D, map: MapId) {
    let (nst, nsp, nat, lmt) = w.mobile(from).map_or((0, 0, 0, 0), |m| {
        (
            m.next_skill_time,
            m.next_spell_time,
            m.next_action_time,
            m.last_move_time,
        )
    });
    let handle = std::rc::Rc::new(std::cell::Cell::new(
        muo_server::timer::TimerHandle::default(),
    ));
    let (mut count, mut chest, mut dirt1, mut dirt2): (
        i32,
        Option<ItemId>,
        Option<ItemId>,
        Option<ItemId>,
    ) = (0, None, None, None);
    let h2 = handle.clone();
    let h = muo_server::timer::start(w, TimeSpan::ZERO, TimeSpan::from_secs(1), 0, move |w| {
        let terminate =
            |w: &mut World, chest: Option<ItemId>, d1: Option<ItemId>, d2: Option<ItemId>| {
                let mut hh = h2.get();
                hh.cancel(w);
                muo_server::mobile::part1::combat::end_action(w, from, "TreasureMap");
                for i in [chest, d1, d2].into_iter().flatten() {
                    delete_item(w, i);
                }
            };
        let Some(m) = w.mobile(from) else {
            terminate(w, chest, dirt1, dirt2);
            return;
        };
        if nst != m.next_skill_time || nsp != m.next_spell_time || nat != m.next_action_time {
            terminate(w, chest, dirt1, dirt2);
            return;
        }
        if lmt != m.last_move_time {
            // You cannot move around while digging up treasure. You will need to start digging anew.
            deps::send_localized_message(w, from, 503023);
            terminate(w, chest, dirt1, dirt2);
            return;
        }
        let mut z = chest.map_or(i32::MIN, |c| {
            w.item(c).map_or(i32::MIN, |i| {
                i.location.z + i32::from(ops::item_data(w, c).height)
            })
        });
        let mut height = 16;
        if z > location.z {
            height -= z - location.z;
        } else {
            z = location.z;
        }
        if !muo_server::map::fit::can_fit(
            w, map, location.x, location.y, z, height, true, true, false,
        ) {
            // You stop digging because something is directly on top of the treasure chest.
            deps::send_localized_message(w, from, 503024);
            terminate(w, chest, dirt1, dirt2);
            return;
        }
        count += 1;
        muo_server::mobile::base::revealing_action(w, from);
        let dir = muo_server::mobile::part2::effects::get_direction_to(
            w, from, location.x, location.y, false,
        );
        muo_content::mx::set_direction(w, from, dir);
        if count > 1 && dirt1.is_none() {
            let d1 = dirt_new(w);
            ops::move_to_world_map(w, d1, location, Some(map));
            dirt1 = Some(d1);
            let d2 = dirt_new(w);
            ops::move_to_world_map(
                w,
                d2,
                Point3D::new(location.x, location.y - 1, location.z),
                Some(map),
            );
            dirt2 = Some(d2);
        }
        if count == 5 {
            if let Some(d) = dirt1 {
                set_graphic(w, d, 0x913);
            }
        } else if count == 10 {
            if let Some(d) = dirt1 {
                set_graphic(w, d, 0x914);
            }
            if let Some(d) = dirt2 {
                set_graphic(w, d, 0x914);
            }
        } else if count > 10 {
            match chest {
                None => {
                    let level = treasure(w, map_item).level;
                    let args = [
                        muo_content::props::Value::Entity(Some(EntityId::Mobile(from))),
                        muo_content::props::Value::Int(i64::from(level)),
                        muo_content::props::Value::Bool(true),
                    ];
                    if let Some(EntityId::Item(c)) = crate::misc::gadgets::create_with_args_pub(
                        w,
                        "Server.Items.TreasureMapChest",
                        &args,
                    ) {
                        ops::move_to_world_map(
                            w,
                            c,
                            Point3D::new(location.x, location.y, location.z - 15),
                            Some(map),
                        );
                        chest = Some(c);
                    }
                }
                Some(c) => {
                    ops::set_z(w, c, w.item(c).map_or(0, |i| i.location.z) + 1);
                }
            }
            if let Some(c) = chest {
                let l = w.item(c).map_or(Point3D::default(), |i| i.location);
                muo_server::effects::play_sound(w, l, Some(map), 0x33B);
            }
        }
        let chest_z = chest.and_then(|c| w.item(c)).map(|i| i.location.z);
        if chest_z.is_some_and(|cz| cz >= location.z) {
            let mut hh = h2.get();
            hh.cancel(w);
            muo_server::mobile::part1::combat::end_action(w, from, "TreasureMap");
            let c = chest.expect("a chest was made");
            if let Some(f) = w
                .resources
                .get::<TreasureChestHooks>()
                .and_then(|h| h.set_temporary)
            {
                f(w, c, false);
            }
            if let Some(d) = treasure_mut(w, map_item) {
                d.completed = true;
                d.completed_by = Some(from);
            }
            ops::invalidate_properties(w, map_item);
            let level = treasure(w, map_item).level;
            let spawns = match level {
                0 => 3,
                1 => 0,
                _ => 4,
            };
            let (cloc, cmap) = w
                .item(c)
                .map_or((Point3D::default(), None), |i| (i.location, i.map));
            for _ in 0..spawns {
                if let Some(bc) = spawn(w, level, cloc, cmap, None, true)
                    && let Some(f) = w
                        .resources
                        .get::<TreasureChestHooks>()
                        .and_then(|h| h.add_to_guardians)
                {
                    f(w, c, bc);
                }
            }
        } else {
            let human = muo_content::creature::misc::body_is_human(w, from);
            let mounted = muo_server::mobile::part1::props_b::mounted(w, from);
            if human && !mounted {
                muo_content::mx::animate(w, from, 11, 5, 1, true, false, 0);
            }
            let sound = 0x125 + count % 2;
            muo_server::timer::start(
                w,
                TimeSpan::from_secs_f64(0.9),
                TimeSpan::ZERO,
                1,
                move |w| {
                    muo_content::mx::play_sound(w, from, sound);
                },
            );
        }
    });
    handle.set(h);
}

/// `TreasureChestDirt` `[SerializationGenerator(0)]`.
pub static TREASURE_CHEST_DIRT: ItemType = ItemType {
    full_name: "Server.Items.TreasureChestDirt",
    parent: Some(&muo_server::item::ITEM_BASE),
    construct: None,
    make: Some(dirt_new),
    skip_serialization: |_, _| true,
    serialize: |w, id, out| {
        (muo_server::item::ITEM_BASE.serialize)(w, id, out);
        out.write_encoded_int(0);
    },
    deserialize: |w, id, r| {
        (muo_server::item::ITEM_BASE.deserialize)(w, id, r)?;
        r.read_encoded_int()?;
        Ok(())
    },
    ..muo_server::item::ITEM_BASE
};

fn treasure_context_entries(
    w: &mut World,
    id: ItemId,
    from: MobileId,
    list: &mut Vec<ContextMenuEntry>,
) {
    base::get_context_menu_entries_base(w, id, from, list);
    let t = treasure(w, id);
    if !t.completed {
        if t.decoder.is_none() {
            let mut e = ContextMenuEntry::new(6147, 2); // Decode Map
            e.on_click = Some(std::rc::Rc::new(|w, from, target| {
                if let EntityId::Item(m) = target
                    && !ops::is_deleted(w, m)
                {
                    decode(w, m, from);
                }
            }));
            list.push(e);
        } else {
            let mut e = ContextMenuEntry::new(6150, 2); // Open Map
            e.on_click = Some(std::rc::Rc::new(|w, from, target| {
                if let EntityId::Item(m) = target
                    && !ops::is_deleted(w, m)
                {
                    display_to(w, m, from);
                }
            }));
            list.push(e);
            let mut e = ContextMenuEntry::new(6148, 2); // Dig for Treasure
            e.enabled = has_digging_tool(w, from);
            e.on_click = Some(std::rc::Rc::new(|w, from, target| {
                let EntityId::Item(m) = target else { return };
                if ops::is_deleted(w, m) {
                    return;
                }
                if !has_digging_tool(w, from) {
                    deps::send_localized_message(w, from, 1114416); // You must have a digging tool to dig for treasure.
                    return;
                }
                on_begin_dig(w, m, from);
            }));
            list.push(e);
        }
    }
}

/// Registers the classes of this file.
pub fn register(types: &mut TypeRegistry) {
    types.register_item(&TREASURE_MAP, &[]);
    types.register_item(&TREASURE_CHEST_DIRT, &[]);
}
