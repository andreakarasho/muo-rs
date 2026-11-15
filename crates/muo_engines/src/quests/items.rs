//! The quest engine's items: `QuestItem`, `DynamicTeleporter`, `HornOfRetreat` and `EnchantedSextant`.

use muo_io::{GenericReader, GenericWriter};
use muo_primitives::{Point2D, Point3D, TimeSpan};
use muo_server::World;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::{DeathMoveResult, ITEM_BASE, ItemType, ops};
use muo_server::map::MapId;
use muo_server::mobile::AccessLevel;
use muo_server::serialization_ext::ReadEntityExt;
use muo_server::timer::TimerHandle;

use super::hooks::quest_hooks;

fn is_player(w: &World, m: MobileId) -> bool {
    w.mobile(m)
        .is_some_and(|m| m.ty.is(&muo_content::player_mobile::PLAYER_MOBILE))
}

/// `[SerializationGenerator(0, false)]` with no fields: the parent part and an int version 0.
pub fn write_version0(out: &mut dyn GenericWriter) {
    out.write_i32(0);
}

pub fn read_version(r: &mut dyn GenericReader) -> muo_io::Result<i32> {
    r.read_i32()
}

/// One item class that is "parent part + int version 0" (`[SerializationGenerator(0, false)]`, no fields):
/// `NAME = "full" : PARENT, |w| ctor; field: value,`. `family` (when given) is the class's own table.
#[macro_export]
macro_rules! quest_item {
    // `@make`: no parameterless `[Constructible]` constructor (content makes it with `factory::make_item`).
    (
        @make
        $(#[$m:meta])*
        $name:ident = $full:literal : $parent:ident, |$w:ident| $ctor:expr;
        $($f:ident : $v:expr,)*
    ) => {
        $(#[$m])*
        pub static $name: muo_server::item::ItemType = muo_server::item::ItemType {
            full_name: $full,
            parent: Some(&$parent),
            construct: None,
            make: Some(|$w| $ctor),
            serialize: |w, id, out| {
                ($parent.serialize)(w, id, out);
                $crate::quests::items::write_version0(out);
            },
            deserialize: |w, id, r| {
                ($parent.deserialize)(w, id, r)?;
                $crate::quests::items::read_version(r)?;
                Ok(())
            },
            $($f: $v,)*
            ..$parent
        };
    };
    (
        $(#[$m:meta])*
        $name:ident = $full:literal : $parent:ident, |$w:ident| $ctor:expr;
        $($f:ident : $v:expr,)*
    ) => {
        $(#[$m])*
        pub static $name: muo_server::item::ItemType = muo_server::item::ItemType {
            full_name: $full,
            parent: Some(&$parent),
            construct: Some(|$w| $ctor),
            serialize: |w, id, out| {
                ($parent.serialize)(w, id, out);
                $crate::quests::items::write_version0(out);
            },
            deserialize: |w, id, r| {
                ($parent.deserialize)(w, id, r)?;
                $crate::quests::items::read_version(r)?;
                Ok(())
            },
            $($f: $v,)*
            ..$parent
        };
    };
}

// ---- QuestItem ----

/// The virtuals of `QuestItem`, the item type's `family`.
pub struct QuestItemClass {
    /// `CanDrop(pm)`.
    pub can_drop: fn(&World, ItemId, MobileId) -> bool,
    /// `Accepted` (`Deleted` in the base).
    pub accepted: fn(&World, ItemId) -> bool,
}

pub static QUEST_ITEM_CLASS: QuestItemClass = QuestItemClass {
    can_drop: |_, _, _| true,
    accepted: |w, id| w.item(id).is_none_or(|i| i.deleted()),
};

fn class(w: &World, id: ItemId) -> &'static QuestItemClass {
    w.item(id)
        .and_then(|i| i.ty.family::<QuestItemClass>())
        .unwrap_or(&QUEST_ITEM_CLASS)
}

/// The tail all three drop overrides share: a quest item may only leave the top level of the pack while it is needed
/// by staff or by a player whose quest no longer needs it.
fn after_drop(w: &mut World, item: ItemId, from: MobileId, ret: bool, message: i32) -> bool {
    if !ret {
        return ret;
    }
    let c = class(w, item);
    let backpack = w.mobile(from).and_then(|m| m.backpack).map(EntityId::Item);
    let parent = w.item(item).and_then(|i| i.parent);
    if (c.accepted)(w, item) || (parent.is_some() && parent == backpack) {
        return ret;
    }
    if w.mobile(from)
        .is_some_and(|m| m.access_level > AccessLevel::Player)
    {
        return true;
    }
    if !is_player(w, from) || (c.can_drop)(w, item, from) {
        return true;
    }
    muo_content::mx::msg(w, from, message);
    false
}

/// `Server.Engines.Quests.QuestItem` (abstract).
pub static QUEST_ITEM: ItemType = ItemType {
    full_name: "Server.Engines.Quests.QuestItem",
    parent: Some(&ITEM_BASE),
    construct: None,
    family: Some(&QUEST_ITEM_CLASS),
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        write_version0(out);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        read_version(r)?;
        Ok(())
    },
    drop_to_world: |w, id, from, p| {
        let ret = (ITEM_BASE.drop_to_world)(w, id, from, p);
        // You can only drop quest items into the top-most level of your backpack while you still need them for
        // your quest.
        after_drop(w, id, from, ret, 1049343)
    },
    drop_to_mobile: |w, id, from, target, p| {
        let ret = (ITEM_BASE.drop_to_mobile)(w, id, from, target, p);
        // You decide against trading the item.  You still need it for your quest.
        after_drop(w, id, from, ret, 1049344)
    },
    drop_to_item: |w, id, from, target, p| {
        let ret = (ITEM_BASE.drop_to_item)(w, id, from, target, p);
        after_drop(w, id, from, ret, 1049343)
    },
    on_parent_death: |w, id, parent| {
        if is_player(w, parent) && !(class(w, id).can_drop)(w, id, parent) {
            DeathMoveResult::MoveToBackpack
        } else {
            (ITEM_BASE.on_parent_death)(w, id, parent)
        }
    },
    ..ITEM_BASE
};

// ---- DynamicTeleporter ----

/// The virtuals of `DynamicTeleporter`.
pub struct TeleporterClass {
    /// `GetDestination(player, ref loc, ref map)`: `None` when it does not work.
    pub get_destination: fn(&mut World, ItemId, MobileId) -> Option<(Point3D, MapId)>,
    /// `NotWorkingMessage`.
    pub not_working_message: i32,
}

pub static TELEPORTER_CLASS: TeleporterClass = TeleporterClass {
    get_destination: |_, _, _| None,
    not_working_message: 500309,
};

/// `DynamicTeleporter(itemID = 0x1822, hue = 0x482)`.
pub fn new_teleporter(w: &mut World, ty: &'static ItemType, item_id: i32, hue: i32) -> ItemId {
    let id = ops::new_item_with_id(w, ty, item_id);
    ops::set_movable(w, id, false);
    muo_server::item::base::set_hue(w, id, hue);
    id
}

/// `Server.Engines.Quests.DynamicTeleporter` (abstract).
pub static DYNAMIC_TELEPORTER: ItemType = ItemType {
    full_name: "Server.Engines.Quests.DynamicTeleporter",
    parent: Some(&ITEM_BASE),
    construct: None,
    family: Some(&TELEPORTER_CLASS),
    label_number: |w, id| {
        crate::template_keys::item_int(w, id, "labelNumber", ITEM_BASE.label_number)
    },
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        write_version0(out);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        read_version(r)?;
        Ok(())
    },
    on_move_over: |w, id, m| {
        if is_player(w, m) {
            let Some(c) = w.item(id).and_then(|i| i.ty.family::<TeleporterClass>()) else {
                return (ITEM_BASE.on_move_over)(w, id, m);
            };
            if let Some((loc, map)) = (c.get_destination)(w, id, m) {
                muo_content::creature::pets::teleport_pets(w, m, loc, map, false);
                muo_content::mx::play_sound(w, m, 0x1FE);
                muo_server::mobile::base::move_to_world(w, m, loc, map);
                return false;
            }
            muo_content::mx::msg(w, m, c.not_working_message);
        }
        (ITEM_BASE.on_move_over)(w, id, m)
    },
    ..ITEM_BASE
};

// ---- HornOfRetreat ----

/// `HornOfRetreat`'s `DestLoc`, `DestMap` and `Charges`.
#[derive(Clone)]
pub struct HornData {
    pub dest_loc: Point3D,
    pub dest_map: Option<MapId>,
    pub charges: i32,
    /// `_timerToken`.
    timer: TimerHandle,
}

/// The virtuals of `HornOfRetreat`.
pub struct HornClass {
    /// `ValidateUse(from)`.
    pub validate_use: fn(&World, ItemId, MobileId) -> bool,
}

pub static HORN_CLASS: HornClass = HornClass {
    validate_use: |_, _, _| true,
};

pub fn horn(w: &World, id: ItemId) -> Option<&HornData> {
    w.item(id)?.ext.get::<HornData>()
}

pub fn horn_mut(w: &mut World, id: ItemId) -> Option<&mut HornData> {
    w.item_mut(id)?.ext.get_mut::<HornData>()
}

/// `HornOfRetreat()`.
pub fn new_horn(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = ops::new_item_with_id(w, ty, 0xFC4);
    muo_server::item::base::set_hue(w, id, 0x482);
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(HornData {
            dest_loc: Point3D::ZERO,
            dest_map: None,
            charges: 10,
            timer: TimerHandle::default(),
        });
    }
    id
}

fn horn_use(w: &mut World, id: ItemId, from: MobileId) {
    let pack = w.mobile(from).and_then(|m| m.backpack);
    if !pack.is_some_and(|p| ops::is_child_of(w, id, Some(EntityId::Item(p)))) {
        muo_server::item::messages::send_localized_message_to(w, id, from, 1042001, ""); // That must be in your pack for you to use it.
        return;
    }
    let validate = w
        .item(id)
        .and_then(|i| i.ty.family::<HornClass>())
        .unwrap_or(&HORN_CLASS)
        .validate_use;
    let map = muo_content::mx::map(w, from);
    let timer_running = horn(w, id).is_some_and(|h| h.timer.running(w));
    let charges = horn(w, id).map_or(0, |h| h.charges);

    if !validate(w, id, from) {
        muo_server::item::messages::send_localized_message_to(w, id, from, 500309, ""); // Nothing Happens.
    } else if w.core_ml() && map != Some(MapId::TRAMMEL) && map != Some(MapId::MALAS) {
        muo_content::mx::msg(w, from, 1076154); // You can only use this in Trammel and Malas.
    } else if timer_running {
        muo_server::item::messages::send_localized_message_to(w, id, from, 1042144, ""); // This is currently in use.
    } else if charges > 0 {
        muo_content::mx::animate(w, from, 34, 7, 1, true, false, 0);
        muo_content::mx::play_sound(w, from, 0xFF);
        muo_content::mx::msg(w, from, 1049115); // You play the horn and a sense of peace overcomes you...

        if let Some(h) = horn_mut(w, id) {
            h.charges -= 1;
        }
        invalidate(w, id);

        let h = muo_server::timer::start(w, TimeSpan::from_secs(5), TimeSpan::ZERO, 1, move |w| {
            play_timer_callback(w, id, from)
        });
        if let Some(d) = horn_mut(w, id) {
            d.timer = h;
        }
    } else {
        muo_server::item::messages::send_localized_message_to(w, id, from, 1042544, ""); // This item is out of charges.
    }
}

fn invalidate(w: &mut World, id: ItemId) {
    if let Some(f) = w.item(id).map(|i| i.ty.invalidate_properties) {
        f(w, id);
    }
}

/// `PlayTimer_Callback(from)`.
fn play_timer_callback(w: &mut World, id: ItemId, from: MobileId) {
    if let Some(mut h) = horn(w, id).map(|h| h.timer) {
        h.cancel(w);
    }
    let Some((dest, dest_map, hue)) = horn(w, id).map(|h| {
        (
            h.dest_loc,
            h.dest_map,
            w.item(id).map_or(0, |i| (i.ty.hue)(w, id)),
        )
    }) else {
        return;
    };
    // HornOfRetreatMoongate(destLoc, destMap, caster, hue): a `Moongate` that is gone after ten seconds
    let gate = muo_items::misc::moongates::horn_gate_new(w, dest, dest_map, from, hue);
    let (loc, map) = (
        muo_content::mx::location(w, from),
        muo_content::mx::map(w, from),
    );
    ops::move_to_world_map(w, gate, loc, map);
    muo_content::mx::play_sound(w, from, 0x20E);
    let name = w
        .mobile(from)
        .and_then(|m| (m.ty.name)(w, from))
        .unwrap_or_default();
    muo_server::item::messages::send_localized_message_to(w, gate, from, 1049102, &name); // Quickly ~1_NAME~! Onward through the gate!
}

/// `Server.Engines.Quests.HornOfRetreat`.
pub static HORN_OF_RETREAT: ItemType = ItemType {
    full_name: "Server.Engines.Quests.HornOfRetreat",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| new_horn(w, &HORN_OF_RETREAT)),
    family: Some(&HORN_CLASS),
    default_weight: |w, id| {
        crate::template_keys::item_float(w, id, "defaultWeight", ITEM_BASE.default_weight)
    },
    label_number: |w, id| {
        crate::template_keys::item_int(w, id, "labelNumber", ITEM_BASE.label_number)
    },
    get_properties: |w, id, list| {
        (ITEM_BASE.get_properties)(w, id, list);
        list.add_int(1060741, horn(w, id).map_or(0, |h| h.charges)); // charges: ~1_val~
    },
    on_double_click: horn_use,
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        out.write_i32(0);
        let h = horn(w, id);
        out.write_point3d(h.map_or(Point3D::ZERO, |h| h.dest_loc));
        out.write_map_index(h.and_then(|h| h.dest_map).map(|m| m.0));
        out.write_i32(h.map_or(0, |h| h.charges));
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        r.read_i32()?;
        let data = HornData {
            dest_loc: r.read_point3d()?,
            dest_map: r.read_map()?,
            charges: r.read_i32()?,
            timer: TimerHandle::default(),
        };
        if let Some(i) = w.item_mut(id) {
            i.ext.insert(data);
        }
        Ok(())
    },
    ..ITEM_BASE
};

// ---- EnchantedSextant ----

const LONG_DISTANCE: f64 = 300.0;
const SHORT_DISTANCE: f64 = 5.0;

static TRAMMEL_BANKS: [Point2D; 18] = [
    Point2D::new(652, 820),
    Point2D::new(1813, 2825),
    Point2D::new(3734, 2149),
    Point2D::new(2503, 552),
    Point2D::new(3764, 1317),
    Point2D::new(587, 2146),
    Point2D::new(1655, 1606),
    Point2D::new(1425, 1690),
    Point2D::new(4471, 1156),
    Point2D::new(1317, 3773),
    Point2D::new(2881, 684),
    Point2D::new(2731, 2192),
    Point2D::new(3620, 2617),
    Point2D::new(2880, 3472),
    Point2D::new(1897, 2684),
    Point2D::new(5346, 74),
    Point2D::new(5275, 3977),
    Point2D::new(5669, 3131),
];

static FELUCCA_BANKS: [Point2D; 18] = [
    Point2D::new(652, 820),
    Point2D::new(1813, 2825),
    Point2D::new(3734, 2149),
    Point2D::new(2503, 552),
    Point2D::new(3764, 1317),
    Point2D::new(3695, 2511),
    Point2D::new(587, 2146),
    Point2D::new(1655, 1606),
    Point2D::new(1425, 1690),
    Point2D::new(4471, 1156),
    Point2D::new(1317, 3773),
    Point2D::new(2881, 684),
    Point2D::new(2731, 2192),
    Point2D::new(2880, 3472),
    Point2D::new(1897, 2684),
    Point2D::new(5346, 74),
    Point2D::new(5275, 3977),
    Point2D::new(5669, 3131),
];

static MALAS_BANKS: [Point2D; 2] = [Point2D::new(996, 519), Point2D::new(2048, 1345)];

/// `from.GetDistanceToSqrt(p)`.
fn distance_sqrt(from: Point3D, p: Point2D) -> f64 {
    let (dx, dy) = (f64::from(from.x - p.x), f64::from(from.y - p.y));
    (dx * dx + dy * dy).sqrt()
}

/// `(int)from.GetDirectionTo(p)`.
fn direction_to(w: &World, from: MobileId, p: Point2D) -> i32 {
    i32::from(
        muo_server::mobile::part2::effects::get_direction_to(w, from, p.x, p.y, false)
            .facing()
            .bits(),
    )
}

fn sextant_use(w: &mut World, id: ItemId, from: MobileId) {
    let world_loc = ops::get_world_location(w, id);
    if !muo_content::mx::in_range_point(w, from, world_loc, 2) {
        super::util::local_overhead_number(w, from, 0x3B2, 1019045); // I can't reach that.
        return;
    }

    let map = muo_content::mx::map(w, from);
    let banks: Option<&[Point2D]> = if map == Some(MapId::TRAMMEL) {
        Some(&TRAMMEL_BANKS)
    } else if map == Some(MapId::FELUCCA) {
        Some(&FELUCCA_BANKS)
    } else if map == Some(MapId::ILSHENAR) {
        // The magic of the sextant fails...
        send_label(w, from, id, 0x482, 1061684);
        return;
    } else if map == Some(MapId::MALAS) {
        Some(&MALAS_BANKS)
    } else {
        None
    };
    let moongates: Vec<Point3D> = map
        .filter(|m| *m == MapId::TRAMMEL || *m == MapId::FELUCCA || *m == MapId::MALAS)
        .and_then(|m| quest_hooks(w).and_then(|h| h.moongates).map(|f| f(w, m)))
        .unwrap_or_default();

    let from_loc = muo_content::mx::location(w, from);
    let mut closest_moongate = Point3D::ZERO;
    let mut moongate_distance = f64::MAX;
    for p in &moongates {
        let dist = distance_sqrt(from_loc, Point2D::new(p.x, p.y));
        if moongate_distance > dist {
            closest_moongate = *p;
            moongate_distance = dist;
        }
    }

    let mut closest_bank = Point2D::ZERO;
    let mut bank_distance = f64::MAX;
    for p in banks.unwrap_or(&[]) {
        let dist = distance_sqrt(from_loc, *p);
        if bank_distance > dist {
            closest_bank = *p;
            bank_distance = dist;
        }
    }

    let moon_msg = if moongate_distance == f64::MAX {
        1048021 // The sextant fails to find a Moongate nearby.
    } else if moongate_distance > LONG_DISTANCE {
        1046449
            + direction_to(
                w,
                from,
                Point2D::new(closest_moongate.x, closest_moongate.y),
            ) // A moongate is * from here
    } else if moongate_distance > SHORT_DISTANCE {
        1048010
            + direction_to(
                w,
                from,
                Point2D::new(closest_moongate.x, closest_moongate.y),
            ) // There is a Moongate * of here.
    } else {
        1048018 // You are next to a Moongate at the moment.
    };
    send_label(w, from, id, 0x482, moon_msg);

    let bank_msg = if bank_distance == f64::MAX {
        1048020 // The sextant fails to find a Bank nearby.
    } else if bank_distance > LONG_DISTANCE {
        1046462 + direction_to(w, from, closest_bank) // A town is * from here
    } else if bank_distance > SHORT_DISTANCE {
        1048002 + direction_to(w, from, closest_bank) // There is a city Bank * of here.
    } else {
        1048019 // You are next to a Bank at the moment.
    };
    send_label(w, from, id, 0x5AA, bank_msg);
}

/// `from.NetState.SendMessageLocalized(Serial, ItemID, MessageType.Label, hue, 3, number)`.
fn send_label(w: &mut World, from: MobileId, item: ItemId, hue: i32, number: i32) {
    let Some(ns) = muo_server::mobile::part1::props_b::net_state(w, from) else {
        return;
    };
    let (serial, graphic) = w
        .item(item)
        .map_or((muo_primitives::Serial::MINUS_ONE, 0), |i| {
            (i.serial, i.item_id)
        });
    muo_server::packets::message::send_message_localized(
        w,
        ns,
        serial,
        graphic,
        muo_server::packets::message::MessageType::LABEL,
        hue,
        3,
        number,
        "",
        "",
    );
}

quest_item! {
    /// `Server.Items.EnchantedSextant`.
    ENCHANTED_SEXTANT = "Server.Items.EnchantedSextant" : ITEM_BASE, |w| ops::new_item_with_id(w, &ENCHANTED_SEXTANT, 0x1058);
    default_weight: |_, _| 2.0,
    label_number: |_, _| 1046226, // an enchanted sextant
    on_double_click: sextant_use,
}

/// The item classes of Core/Items.
pub static ITEMS: &[&ItemType] = &[
    &QUEST_ITEM,
    &DYNAMIC_TELEPORTER,
    &HORN_OF_RETREAT,
    &ENCHANTED_SEXTANT,
];
