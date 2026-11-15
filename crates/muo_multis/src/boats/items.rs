//! `BaseBoatDeed`, `BaseDockedBoat`, `ConfirmDryDockGump` and the small, medium and large (dragon) boats: the six
//! boats with their deeds and docked forms, the deed and dock targets and the dry dock confirmation.

use muo_content::gumps::builder::StaticGumpBuilder;
use muo_content::gumps::{RelayInfo, StaticGumpLogic, StaticGumpObj, send_gump_to_mobile};
use muo_primitives::Point3D;
use muo_server::World;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType, LootType, ops};
use muo_server::map::MapId;
use muo_server::mobile::AccessLevel;
use muo_server::net_state::NetStateId;
use muo_server::packets::message::MessageType;
use muo_server::region;
use muo_server::target::{Target, TargetFlags, Targeted, add_target, set_mobile_target};
use muo_server::type_registry::TypeRegistry;

use super::{BASE_BOAT, BoatClass, find_boat_at, is_valid_location, new_boat};

fn msg(w: &mut World, m: MobileId, number: i32) {
    muo_server::item::deps::send_localized_message(w, m, number);
}

fn delete_item(w: &mut World, id: ItemId) {
    if let Some(ty) = w.item(id).map(|i| i.ty) {
        (ty.delete)(w, id);
    }
}

/// `IsChildOf(from.Backpack)`.
fn in_pack(w: &World, item: ItemId, from: MobileId) -> bool {
    let pack = w.mobile(from).and_then(|m| m.backpack).map(EntityId::Item);
    ops::is_child_of(w, item, pack)
}

// ---- ConfirmDryDockGump ----

struct ConfirmDryDockGump {
    boat: ItemId,
}

impl StaticGumpLogic for ConfirmDryDockGump {
    const TYPE_NAME: &'static str = "Server.Multis.ConfirmDryDockGump";

    fn build_layout(&self, b: &mut StaticGumpBuilder) {
        b.add_page(0);
        b.add_background(0, 0, 220, 170, 5054);
        b.add_background(10, 10, 200, 150, 3000);
        b.add_html_localized(20, 20, 180, 80, 1018319, true, false); // Do you wish to dry dock this boat?
        b.add_html_localized(55, 100, 140, 25, 1011011, false, false); // CONTINUE
        b.add_button(20, 100, 4005, 4007, 2);
        b.add_html_localized(55, 125, 140, 25, 1011012, false, false); // CANCEL
        b.add_button(20, 125, 4005, 4007, 1);
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        if info.button_id == 2
            && let Some(m) = w.net_mobile(sender)
        {
            super::end_dry_dock(w, self.boat, m);
        }
    }
}

/// `from.SendGump(new ConfirmDryDockGump(this))`.
pub fn send_confirm_dry_dock_gump(w: &mut World, from: MobileId, boat: ItemId) {
    send_gump_to_mobile(
        w,
        from,
        Box::new(StaticGumpObj::new(150, 200, ConfirmDryDockGump { boat })),
        false,
    );
}

// ---- the boats ----

macro_rules! boat_leaf {
    ($name:ident, $cls:ident, $full:literal, $new:ident, [$n:expr, $e:expr, $s:expr, $w:expr], hold: $hold:expr,
     tiller: $tiller:expr, starboard: $sb:expr, port: $port:expr, mark: $mark:expr, docked: $docked:ident) => {
        pub static $cls: BoatClass = BoatClass {
            north_id: $n,
            east_id: $e,
            south_id: $s,
            west_id: $w,
            hold_distance: $hold,
            tiller_man_distance: $tiller,
            starboard_offset: $sb,
            port_offset: $port,
            mark_offset: $mark,
            docked_boat: |w, boat| Some($docked(w, boat)),
        };

        #[doc = concat!("`", $full, "`.")]
        pub static $name: ItemType = ItemType {
            full_name: $full,
            parent: Some(&BASE_BOAT),
            construct: Some(|w| $new(w)),
            family: Some(&$cls),
            serialize: |w, id, out| {
                (BASE_BOAT.serialize)(w, id, out);
                out.write_i32(0);
            },
            deserialize: |w, id, r| {
                (BASE_BOAT.deserialize)(w, id, r)?;
                let _version = r.read_i32()?;
                Ok(())
            },
            ..BASE_BOAT
        };

        pub fn $new(w: &mut World) -> ItemId {
            new_boat(w, &$name)
        }
    };
}

boat_leaf!(SMALL_BOAT, SMALL_BOAT_CLASS, "Server.Multis.SmallBoat", new_small_boat, [0x0, 0x1, 0x2, 0x3],
    hold: 4, tiller: -4, starboard: (2, 0), port: (-2, 0), mark: (0, 1, 3), docked: new_small_docked_boat);
boat_leaf!(SMALL_DRAGON_BOAT, SMALL_DRAGON_BOAT_CLASS, "Server.Multis.SmallDragonBoat", new_small_dragon_boat,
    [0x4, 0x5, 0x6, 0x7], hold: 4, tiller: -4, starboard: (2, 0), port: (-2, 0), mark: (0, 1, 3),
    docked: new_small_docked_dragon_boat);
boat_leaf!(MEDIUM_BOAT, MEDIUM_BOAT_CLASS, "Server.Multis.MediumBoat", new_medium_boat, [0x8, 0x9, 0xA, 0xB],
    hold: 4, tiller: -5, starboard: (2, 0), port: (-2, 0), mark: (0, 1, 3), docked: new_medium_docked_boat);
boat_leaf!(MEDIUM_DRAGON_BOAT, MEDIUM_DRAGON_BOAT_CLASS, "Server.Multis.MediumDragonBoat", new_medium_dragon_boat,
    [0xC, 0xD, 0xE, 0xF], hold: 4, tiller: -5, starboard: (2, 0), port: (-2, 0), mark: (0, 1, 3),
    docked: new_medium_docked_dragon_boat);
boat_leaf!(LARGE_BOAT, LARGE_BOAT_CLASS, "Server.Multis.LargeBoat", new_large_boat, [0x10, 0x11, 0x12, 0x13],
    hold: 5, tiller: -5, starboard: (2, -1), port: (-2, -1), mark: (0, 0, 3), docked: new_large_docked_boat);
boat_leaf!(LARGE_DRAGON_BOAT, LARGE_DRAGON_BOAT_CLASS, "Server.Multis.LargeDragonBoat", new_large_dragon_boat,
    [0x14, 0x15, 0x16, 0x17], hold: 5, tiller: -5, starboard: (2, -1), port: (-2, -1), mark: (0, 0, 3),
    docked: new_large_docked_dragon_boat);

// ---- BaseBoatDeed ----

/// What `BaseBoatDeed` adds.
#[derive(Clone, Debug, Default)]
pub struct BoatDeedData {
    pub multi_id: i32,
    pub offset: Point3D,
}

/// What a concrete deed overrides: `Boat => new XBoat()` and the label.
pub struct BoatDeedClass {
    pub boat: fn(&mut World) -> ItemId,
}

fn deed_data(w: &World, id: ItemId) -> BoatDeedData {
    w.item(id)
        .and_then(|i| i.ext.get::<BoatDeedData>())
        .cloned()
        .unwrap_or_default()
}

fn deed_class(w: &World, id: ItemId) -> Option<&'static BoatDeedClass> {
    w.item(id).and_then(|i| i.ty.family::<BoatDeedClass>())
}

/// `new XBoatDeed()`: `base(id, offset)`.
/// `BaseBoatDeed(id, offset)` of class `ty`: the class's template (it inherits `BaseBoatDeed`'s), then
/// the multi and offset of its props `multiId` and `offset`.
fn new_deed(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = ops::new_item_with_id(w, ty, 0);
    muo_content::templates::apply_item_as(w, id, ty.full_name);
    let (multi_id, offset) = crate::template_props::deed_props(w, id);
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(BoatDeedData { multi_id, offset });
    }
    id
}

fn in_dungeon_or_house(w: &World, p: Point3D, map: Option<MapId>) -> (bool, bool) {
    let reg = region::find(w, p, map);
    let dungeon = region::is_part_of_type(w, reg, &muo_content::regions::DUNGEON_REGION);
    let house = region::is_part_of_type(w, reg, &muo_content::regions::house::HOUSE_REGION);
    let champion = match map {
        Some(m) => muo_content::spells::hooks::hooks(w)
            .and_then(|h| h.is_champion_spawn)
            .is_some_and(|f| f(w, m, p)),
        None => false,
    };
    (dungeon, house || champion)
}

/// `BaseBoatDeed.OnPlacement(from, p)` (for `placed_by_deed`) and `BaseDockedBoat.OnPlacement`.
fn place_boat(w: &mut World, item: ItemId, from: MobileId, p: Point3D, deed: bool) {
    if ops::is_deleted(w, item) {
        return;
    }
    if !in_pack(w, item, from) {
        msg(w, from, 1042001); // That must be in your pack for you to use it.
        return;
    }
    let Some(map) = w.mobile(from).and_then(|m| m.map) else {
        return;
    };
    let gm = w
        .mobile(from)
        .is_some_and(|m| m.access_level >= AccessLevel::GameMaster);

    // The deed checks these before it builds the boat, the docked boat after.
    let make_boat = |w: &mut World| -> Option<ItemId> {
        if deed {
            deed_class(w, item).map(|c| (c.boat)(w))
        } else {
            docked_class(w, item).map(|c| (c.boat)(w))
        }
    };
    let mut boat = None;
    if !deed {
        boat = make_boat(w);
        if boat.is_none() {
            return;
        }
    }
    if muo_content::items::house_deps::deps(w)
        .and_then(|d| d.boat_placement_blocked)
        .is_some_and(|f| f(w, from))
    {
        muo_server::mobile::part2::messages::send_message_hue(
            w,
            from,
            0x22,
            "Boat placement is temporarily disabled.",
        );
        if !deed {
            return; // the docked boat stops here and leaves the built boat orphaned; the deed carries on (an old slip, kept)
        }
    }
    if deed {
        if !gm && (map == MapId::ILSHENAR || map == MapId::MALAS) {
            msg(w, from, 1043284); // A ship can not be created here.
            return;
        }
        let from_loc = w.mobile(from).map_or(Point3D::ZERO, |m| m.location);
        let in_house_region = {
            let reg = w.mobile(from).map(|m| m.region);
            reg.is_some_and(|r| {
                region::is_part_of_type(w, r, &muo_content::regions::house::HOUSE_REGION)
            })
        };
        if in_house_region || find_boat_at(w, from_loc, Some(map)).is_some() {
            // You may not place a ship while on another ship or inside a house.
            muo_server::mobile::part2::messages::send_localized_message(w, from, 1010568, "", 0x25);
            return;
        }
        boat = make_boat(w);
    }
    let Some(boat) = boat else { return };

    let offset = if deed {
        deed_data(w, item).offset
    } else {
        docked_data(w, item).0
    };
    let p = Point3D::new(p.x - offset.x, p.y - offset.y, p.z - offset.z);
    let item_id = w.item(boat).map_or(0, |i| i.item_id);
    let valid_map = deed || (map != MapId::ILSHENAR && map != MapId::MALAS);
    if is_valid_location(p, Some(map))
        && super::can_fit(w, boat, p, Some(map), item_id)
        && valid_map
    {
        delete_item(w, item);
        super::set_owner(w, boat, Some(from));
        super::set_anchored(w, boat, true);
        if !deed {
            super::set_ship_name(w, boat, docked_data(w, item).1);
        }
        let key_value = super::create_keys(w, boat, from);
        for plank in [super::p_plank(w, boat), super::s_plank(w, boat)]
            .into_iter()
            .flatten()
        {
            super::parts::set_plank_key_value(w, plank, key_value);
        }
        ops::move_to_world_map(w, boat, p, Some(map));
    } else {
        delete_item(w, boat);
        msg(w, from, 1043284); // A ship can not be created here.
    }
}

/// `new InternalTarget(...)`: the multi target of the deed (`deed`) or the docked boat.
fn begin_placement_target(w: &mut World, item: ItemId, from: MobileId, deed: bool) {
    let (multi_id, offset) = if deed {
        let d = deed_data(w, item);
        (d.multi_id, d.offset)
    } else {
        let d = docked_data(w, item);
        (docked_multi_id(w, item), d.0)
    };
    let mut t = Target::new_multi(w, multi_id, offset, 10, true, TargetFlags::NONE);
    t.handlers.on_target = Some(std::rc::Rc::new(move |w, from, targeted| {
        let p = match targeted {
            // The deed: `if (ip is Item item) ip = from;`; the docked boat uses the item's top.
            Targeted::Entity(EntityId::Item(i)) if deed => {
                let _ = i;
                w.mobile(from).map_or(Point3D::ZERO, |m| m.location)
            }
            other => match crate::deeds::target_point(w, other) {
                Some(p) => p,
                None => return,
            },
        };
        let map = w.mobile(from).and_then(|m| m.map);
        let (dungeon, house_or_champion) = in_dungeon_or_house(w, p, map);
        if dungeon {
            msg(w, from, 502488); // You can not place a ship inside a dungeon.
        } else if house_or_champion {
            msg(w, from, 1042549); // A boat may not be placed in this area.
        } else {
            place_boat(w, item, from, p, deed);
        }
    }));
    let id = add_target(w, t);
    set_mobile_target(w, from, Some(id));
}

fn deed_on_double_click(w: &mut World, id: ItemId, from: MobileId) {
    let map = w.mobile(from).and_then(|m| m.map);
    let gm = w
        .mobile(from)
        .is_some_and(|m| m.access_level >= AccessLevel::GameMaster);
    if !in_pack(w, id, from) {
        msg(w, from, 1042001); // That must be in your pack for you to use it.
    } else if !gm && (map == Some(MapId::ILSHENAR) || map == Some(MapId::MALAS)) {
        muo_server::mobile::part2::messages::send_localized_message(w, from, 1010567, "", 0x25); // You may not place a boat from this location.
    } else {
        if muo_content::core::se(w) {
            msg(w, from, 502482); // Where do you wish to place the ship?
        } else {
            muo_server::item::deps::local_overhead_message(
                w,
                from,
                MessageType::REGULAR,
                0x3B2,
                502482,
            );
        }
        begin_placement_target(w, id, from, true);
    }
}

macro_rules! deed_leaf {
    ($name:ident, $cls:ident, $full:literal, $new:ident, $boat:ident) => {
        pub static $cls: BoatDeedClass = BoatDeedClass { boat: $boat };

        #[doc = concat!("`", $full, "`.")]
        pub static $name: ItemType = ItemType {
            full_name: $full,
            parent: Some(&ITEM_BASE),
            construct: Some(|w| $new(w)),
            family: Some(&$cls),
            label_number: |w, id| {
                crate::template_props::item_int(w, id, "labelNumber", ITEM_BASE.label_number)
            },
            default_weight: |_, _| 1.0,
            on_double_click: deed_on_double_click,
            serialize: |w, id, out| {
                (ITEM_BASE.serialize)(w, id, out);
                out.write_i32(0);
                let d = deed_data(w, id);
                out.write_i32(d.multi_id);
                out.write_point3d(d.offset);
                out.write_i32(0); // the leaf's version
            },
            deserialize: |w, id, r| {
                (ITEM_BASE.deserialize)(w, id, r)?;
                let _version = r.read_i32()?;
                let multi_id = r.read_i32()?;
                let offset = r.read_point3d()?;
                if let Some(i) = w.item_mut(id) {
                    i.ext.insert(BoatDeedData { multi_id, offset });
                }
                let _leaf = r.read_i32()?;
                Ok(())
            },
            ..ITEM_BASE
        };

        pub fn $new(w: &mut World) -> ItemId {
            new_deed(w, &$name)
        }
    };
}

deed_leaf!(
    SMALL_BOAT_DEED,
    SMALL_BOAT_DEED_CLASS,
    "Server.Multis.SmallBoatDeed",
    new_small_boat_deed,
    new_small_boat
);
deed_leaf!(
    SMALL_DRAGON_BOAT_DEED,
    SMALL_DRAGON_BOAT_DEED_CLASS,
    "Server.Multis.SmallDragonBoatDeed",
    new_small_dragon_boat_deed,
    new_small_dragon_boat
);
deed_leaf!(
    MEDIUM_BOAT_DEED,
    MEDIUM_BOAT_DEED_CLASS,
    "Server.Multis.MediumBoatDeed",
    new_medium_boat_deed,
    new_medium_boat
);
deed_leaf!(
    MEDIUM_DRAGON_BOAT_DEED,
    MEDIUM_DRAGON_BOAT_DEED_CLASS,
    "Server.Multis.MediumDragonBoatDeed",
    new_medium_dragon_boat_deed,
    new_medium_dragon_boat
);
deed_leaf!(
    LARGE_BOAT_DEED,
    LARGE_BOAT_DEED_CLASS,
    "Server.Multis.LargeBoatDeed",
    new_large_boat_deed,
    new_large_boat
);
deed_leaf!(
    LARGE_DRAGON_BOAT_DEED,
    LARGE_DRAGON_BOAT_DEED_CLASS,
    "Server.Multis.LargeDragonBoatDeed",
    new_large_dragon_boat_deed,
    new_large_dragon_boat
);

// ---- BaseDockedBoat ----

/// What `BaseDockedBoat` adds.
#[derive(Clone, Debug, Default)]
pub struct DockedData {
    pub multi_id: i32,
    pub offset: Point3D,
    pub ship_name: Option<String>,
}

/// `Boat => new XBoat()` of a docked boat.
pub struct DockedClass {
    pub boat: fn(&mut World) -> ItemId,
}

fn docked_class(w: &World, id: ItemId) -> Option<&'static DockedClass> {
    w.item(id).and_then(|i| i.ty.family::<DockedClass>())
}

/// (offset, ship name) of a docked boat.
fn docked_data(w: &World, id: ItemId) -> (Point3D, Option<String>) {
    w.item(id)
        .and_then(|i| i.ext.get::<DockedData>())
        .map_or((Point3D::ZERO, None), |d| (d.offset, d.ship_name.clone()))
}

fn docked_multi_id(w: &World, id: ItemId) -> i32 {
    w.item(id)
        .and_then(|i| i.ext.get::<DockedData>())
        .map_or(0, |d| d.multi_id)
}

/// `new XDockedBoat(boat)`: `base(id, offset, boat)`.
fn new_docked(
    w: &mut World,
    ty: &'static ItemType,
    multi_id: i32,
    offset: Point3D,
    boat: ItemId,
) -> ItemId {
    let id = ops::new_item_with_id(w, ty, 0x14F4);
    ops::set_loot_type(w, id, LootType::Blessed);
    let ship_name = super::ship_name(w, boat);
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(DockedData {
            multi_id,
            offset,
            ship_name,
        });
    }
    id
}

macro_rules! docked_leaf {
    ($name:ident, $cls:ident, $full:literal, $new:ident, $multi:expr, $offset:expr, $boat:ident) => {
        pub static $cls: DockedClass = DockedClass { boat: $boat };

        #[doc = concat!("`", $full, "`.")]
        pub static $name: ItemType = ItemType {
            full_name: $full,
            parent: Some(&ITEM_BASE),
            construct: None,
            family: Some(&$cls),
            default_weight: |_, _| 1.0,
            on_double_click: |w, id, from| {
                if !in_pack(w, id, from) {
                    msg(w, from, 1042001); // That must be in your pack for you to use it.
                } else {
                    msg(w, from, 502482); // Where do you wish to place the ship?
                    begin_placement_target(w, id, from, false);
                }
            },
            add_name_property: |w, id, list| match docked_data(w, id).1 {
                Some(n) => list.add_text(&n),
                None => (ITEM_BASE.add_name_property)(w, id, list),
            },
            on_single_click: |w, id, from| match docked_data(w, id).1 {
                Some(n) => ops::label_to(w, id, from, &n),
                None => (ITEM_BASE.on_single_click)(w, id, from),
            },
            serialize: |w, id, out| {
                (ITEM_BASE.serialize)(w, id, out);
                out.write_i32(0);
                let d = w
                    .item(id)
                    .and_then(|i| i.ext.get::<DockedData>())
                    .cloned()
                    .unwrap_or_default();
                out.write_i32(d.multi_id);
                out.write_point3d(d.offset);
                out.write_opt_string(d.ship_name.as_deref());
                out.write_i32(0); // the leaf's version
            },
            deserialize: |w, id, r| {
                (ITEM_BASE.deserialize)(w, id, r)?;
                let _version = r.read_i32()?;
                let multi_id = r.read_i32()?;
                let offset = r.read_point3d()?;
                let ship_name = r.read_string()?;
                if let Some(i) = w.item_mut(id) {
                    i.ext.insert(DockedData {
                        multi_id,
                        offset,
                        ship_name,
                    });
                }
                let _leaf = r.read_i32()?;
                Ok(())
            },
            ..ITEM_BASE
        };

        /// `new XDockedBoat(boat)`.
        pub fn $new(w: &mut World, boat: ItemId) -> ItemId {
            new_docked(w, &$name, $multi, $offset, boat)
        }
    };
}

docked_leaf!(
    SMALL_DOCKED_BOAT,
    SMALL_DOCKED_BOAT_CLASS,
    "Server.Multis.SmallDockedBoat",
    new_small_docked_boat,
    0x0,
    Point3D::ZERO,
    new_small_boat
);
docked_leaf!(
    SMALL_DOCKED_DRAGON_BOAT,
    SMALL_DOCKED_DRAGON_BOAT_CLASS,
    "Server.Multis.SmallDockedDragonBoat",
    new_small_docked_dragon_boat,
    0x4,
    Point3D::ZERO,
    new_small_dragon_boat
);
docked_leaf!(
    MEDIUM_DOCKED_BOAT,
    MEDIUM_DOCKED_BOAT_CLASS,
    "Server.Multis.MediumDockedBoat",
    new_medium_docked_boat,
    0x8,
    Point3D::ZERO,
    new_medium_boat
);
docked_leaf!(
    MEDIUM_DOCKED_DRAGON_BOAT,
    MEDIUM_DOCKED_DRAGON_BOAT_CLASS,
    "Server.Multis.MediumDockedDragonBoat",
    new_medium_docked_dragon_boat,
    0xC,
    Point3D::ZERO,
    new_medium_dragon_boat
);
docked_leaf!(
    LARGE_DOCKED_BOAT,
    LARGE_DOCKED_BOAT_CLASS,
    "Server.Multis.LargeDockedBoat",
    new_large_docked_boat,
    0x10,
    Point3D { x: 0, y: -1, z: 0 },
    new_large_boat
);
docked_leaf!(
    LARGE_DOCKED_DRAGON_BOAT,
    LARGE_DOCKED_DRAGON_BOAT_CLASS,
    "Server.Multis.LargeDockedDragonBoat",
    new_large_docked_dragon_boat,
    0x14,
    Point3D { x: 0, y: -1, z: 0 },
    new_large_dragon_boat
);

pub fn register(types: &mut TypeRegistry) {
    for t in [
        &SMALL_BOAT,
        &SMALL_DRAGON_BOAT,
        &MEDIUM_BOAT,
        &MEDIUM_DRAGON_BOAT,
        &LARGE_BOAT,
        &LARGE_DRAGON_BOAT,
        &SMALL_BOAT_DEED,
        &SMALL_DRAGON_BOAT_DEED,
        &MEDIUM_BOAT_DEED,
        &MEDIUM_DRAGON_BOAT_DEED,
        &LARGE_BOAT_DEED,
        &LARGE_DRAGON_BOAT_DEED,
        &SMALL_DOCKED_BOAT,
        &SMALL_DOCKED_DRAGON_BOAT,
        &MEDIUM_DOCKED_BOAT,
        &MEDIUM_DOCKED_DRAGON_BOAT,
        &LARGE_DOCKED_BOAT,
        &LARGE_DOCKED_DRAGON_BOAT,
    ] {
        types.register_item(t, &[]);
    }
}
