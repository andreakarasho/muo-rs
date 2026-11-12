//! The boat parts: `Plank`, `TillerMan`, `Hold`.

use muo_content::factory::mobile_is;
use muo_io::GenericReader;
use muo_primitives::{Direction, Point3D, TimeSpan};
use muo_server::World;
use muo_server::ids::{ItemId, MobileId};
use muo_server::item::container::{self, CONTAINER};
use muo_server::item::{ITEM_BASE, ItemType, ops};
use muo_server::lr_reason::LRReason;
use muo_server::map::{enumerate, fit};
use muo_server::mobile::AccessLevel;
use muo_server::packets::message::MessageType;
use muo_server::property_list::PropertyList;
use muo_server::serialization_ext::{ReadEntityExt, WriteEntityExt};
use muo_server::timer::{self, TimerHandle};
use muo_server::type_registry::TypeRegistry;

use super::{contains_mobile, is_boat};

fn delete_item(w: &mut World, id: ItemId) {
    if let Some(ty) = w.item(id).map(|i| i.ty) {
        (ty.delete)(w, id);
    }
}

fn graphic(w: &World, id: ItemId) -> i32 {
    w.item(id).map_or(0, |i| i.item_id)
}

fn set_graphic(w: &mut World, id: ItemId, v: i32) {
    if let Some(ty) = w.item(id).map(|i| i.ty) {
        (ty.set_item_id)(w, id, v);
    }
}

fn read_boat(w: &World, r: &mut dyn GenericReader) -> muo_io::Result<Option<ItemId>> {
    Ok(r.read_item(w)?.filter(|&b| is_boat(w, b)))
}

// ---- Plank ----

/// `PlankSide`.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(i32)]
pub enum PlankSide {
    Port = 0,
    Starboard = 1,
}

/// What `Plank` adds to an `Item`.
#[derive(Clone, Debug)]
pub struct PlankData {
    pub boat: Option<ItemId>,
    pub side: PlankSide,
    pub locked: bool,
    pub key_value: u32,
    pub close_timer: Option<TimerHandle>,
}

fn pd(w: &World, id: ItemId) -> Option<&PlankData> {
    w.item(id)?.ext.get::<PlankData>()
}

fn pd_mut(w: &mut World, id: ItemId) -> Option<&mut PlankData> {
    w.item_mut(id)?.ext.get_mut::<PlankData>()
}

/// `new Plank(boat, side, keyValue)`.
pub fn new_plank(w: &mut World, boat: ItemId, side: PlankSide, key_value: u32) -> ItemId {
    let id = ops::new_item_with_id(w, &PLANK, 0x3EB1 + side as i32);
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(PlankData {
            boat: Some(boat),
            side,
            locked: true,
            key_value,
            close_timer: None,
        });
    }
    ops::set_movable(w, id, false);
    id
}

pub fn plank_key_value(w: &World, id: ItemId) -> u32 {
    pd(w, id).map_or(0, |d| d.key_value)
}

/// `KeyValue = value`.
pub fn set_plank_key_value(w: &mut World, id: ItemId, v: u32) {
    if let Some(d) = pd_mut(w, id) {
        d.key_value = v;
    }
}

pub fn plank_locked(w: &World, id: ItemId) -> bool {
    pd(w, id).is_some_and(|d| d.locked)
}

fn plank_boat(w: &World, id: ItemId) -> Option<ItemId> {
    pd(w, id).and_then(|d| d.boat)
}

/// `IsOpen`.
pub fn plank_is_open(w: &World, id: ItemId) -> bool {
    matches!(graphic(w, id), 0x3ED5 | 0x3ED4 | 0x3E84 | 0x3E89)
}

fn starboard(w: &World, id: ItemId) -> bool {
    pd(w, id).is_some_and(|d| d.side == PlankSide::Starboard)
}

/// `SetFacing(dir)` (a `Direction` number).
pub fn plank_set_facing(w: &mut World, id: ItemId, dir: i32) {
    let sb = starboard(w, id);
    let pick = |s: i32, p: i32| if sb { s } else { p };
    let cur = graphic(w, id);
    let v = if plank_is_open(w, id) {
        match dir {
            0 => pick(0x3ED4, 0x3ED5),
            2 => pick(0x3E84, 0x3E89),
            4 => pick(0x3ED5, 0x3ED4),
            6 => pick(0x3E89, 0x3E84),
            _ => cur,
        }
    } else {
        match dir {
            0 => pick(0x3EB2, 0x3EB1),
            2 => pick(0x3E85, 0x3E8A),
            4 => pick(0x3EB1, 0x3EB2),
            6 => pick(0x3E8A, 0x3E85),
            _ => cur,
        }
    };
    set_graphic(w, id, v);
}

fn start_close_timer(w: &mut World, id: ItemId) {
    let h = timer::start(
        w,
        TimeSpan::from_secs(5),
        TimeSpan::from_secs(5),
        0,
        move |w| plank_close(w, id),
    );
    if let Some(d) = pd_mut(w, id) {
        d.close_timer = Some(h);
    }
}

fn stop_close_timer(w: &mut World, id: ItemId) {
    if let Some(h) = pd(w, id).and_then(|d| d.close_timer) {
        timer::stop(w, h);
    }
}

/// `Open()`.
pub fn plank_open(w: &mut World, id: ItemId) {
    if plank_is_open(w, id) || ops::is_deleted(w, id) {
        return;
    }
    stop_close_timer(w, id);
    start_close_timer(w, id);
    let next = match graphic(w, id) {
        0x3EB1 => 0x3ED5,
        0x3E8A => 0x3E89,
        0x3EB2 => 0x3ED4,
        0x3E85 => 0x3E84,
        g => g,
    };
    set_graphic(w, id, next);
    if let Some(b) = plank_boat(w, id) {
        super::refresh(w, b);
    }
}

/// `CanClose()`.
pub fn plank_can_close(w: &World, id: ItemId) -> bool {
    let Some(i) = w.item(id) else { return false };
    if i.map.is_none() || i.deleted() {
        return false;
    }
    let (map, x, y) = (i.map, i.location.x, i.location.y);
    if enumerate::items_at(w, map, x, y).any(|o| o != id) {
        return false;
    }
    enumerate::mobiles_at(w, map, x, y).next().is_none()
}

/// `Close()`.
pub fn plank_close(w: &mut World, id: ItemId) {
    if !plank_is_open(w, id) || !plank_can_close(w, id) || ops::is_deleted(w, id) {
        return;
    }
    stop_close_timer(w, id);
    if let Some(d) = pd_mut(w, id) {
        d.close_timer = None;
    }
    let next = match graphic(w, id) {
        0x3ED5 => 0x3EB1,
        0x3E89 => 0x3E8A,
        0x3ED4 => 0x3EB2,
        0x3E84 => 0x3E85,
        g => g,
    };
    set_graphic(w, id, next);
    if let Some(b) = plank_boat(w, id) {
        super::refresh(w, b);
    }
}

fn local_overhead(w: &mut World, m: MobileId, number: i32) {
    muo_server::item::deps::local_overhead_message(w, m, MessageType::REGULAR, 0x00, number);
}

fn plank_on_move_over(w: &mut World, id: ItemId, from: MobileId) -> bool {
    if !plank_is_open(w, id) {
        return false;
    }
    if mobile_is(w, from, "Server.Factions.BaseFactionGuard") {
        return false;
    }
    let running = w
        .mobile(from)
        .is_some_and(|m| m.direction.bits() & Direction::RUNNING.bits() != 0);
    let boat_has_not = plank_boat(w, id).is_some_and(|b| !contains_mobile(w, b, from));
    if running || boat_has_not {
        return true;
    }
    let Some(map) = w.item(id).and_then(|i| i.map) else {
        return false;
    };
    let (loc, from_z) = (
        w.item(id).map_or(Point3D::ZERO, |i| i.location),
        w.mobile(from).map_or(0, |m| m.location.z),
    );
    let (mut rx, mut ry) = (0, 0);
    match graphic(w, id) {
        0x3ED4 => rx = 1,
        0x3ED5 => rx = -1,
        0x3E84 => ry = 1,
        0x3E89 => ry = -1,
        _ => {}
    }
    let blocked = |w: &mut World, x: i32, y: i32, z: i32| {
        !muo_content::spells::helper::check_multi(w, Point3D::new(x, y, z), Some(map), true, 0)
            && !muo_content::spells::hooks::hooks(w)
                .and_then(|h| h.is_faction_stronghold)
                .is_some_and(|f| f(w, map, Point3D::new(x, y, z)))
    };
    for i in 1..=6 {
        let (x, y) = (loc.x + i * rx, loc.y + i * ry);
        for j in -8..=8 {
            let z = from_z + j;
            if fit::can_fit(w, map, x, y, z, 16, false, false, true) && blocked(w, x, y, z) {
                if i == 1 && (-2..=2).contains(&j) {
                    return true;
                }
                muo_server::mobile::part1::lifecycle::set_location(w, from, Point3D::new(x, y, z));
                return false;
            }
        }
        let z = w.maps.get_average_z(map, x, y);
        if fit::can_fit(w, map, x, y, z, 16, false, false, true) && blocked(w, x, y, z) {
            if i == 1 {
                return true;
            }
            muo_server::mobile::part1::lifecycle::set_location(w, from, Point3D::new(x, y, z));
            return false;
        }
    }
    true
}

fn plank_on_double_click(w: &mut World, id: ItemId, from: MobileId) {
    let Some(boat) = plank_boat(w, id) else {
        return;
    };
    let wl = ops::get_world_location(w, id);
    if !muo_server::mobile::base::in_range_point3d(w, from, wl, 8) {
        return;
    }
    let loc = w.item(id).map_or(Point3D::ZERO, |i| i.location);
    let gm = w
        .mobile(from)
        .is_some_and(|m| m.access_level >= AccessLevel::GameMaster);
    let step_on = |w: &mut World| {
        muo_server::mobile::part1::lifecycle::set_location(
            w,
            from,
            Point3D::new(loc.x, loc.y, loc.z + 3),
        );
    };
    if contains_mobile(w, boat, from) {
        if plank_is_open(w, id) {
            plank_close(w, id);
        } else {
            plank_open(w, id);
        }
    } else if !plank_is_open(w, id) {
        if !plank_locked(w, id) {
            plank_open(w, id);
        } else if gm {
            local_overhead(w, from, 502502); // That is locked but your godly powers allow access
            plank_open(w, id);
        } else {
            local_overhead(w, from, 502503); // That is locked.
        }
    } else if !plank_locked(w, id) {
        step_on(w);
    } else if gm {
        local_overhead(w, from, 502502); // That is locked but your godly powers allow access
        step_on(w);
    } else {
        local_overhead(w, from, 502503); // That is locked.
    }
}

/// `ILockable.KeyValue` of a plank, for the key's target; None when `item` is no plank.
pub fn lockable_key_value(w: &World, item: ItemId) -> Option<u32> {
    w.item(item).filter(|i| i.ty.is(&PLANK))?;
    Some(plank_key_value(w, item))
}

/// `lockable.Locked = !lockable.Locked` for a plank; the new state.
pub fn lockable_toggle(w: &mut World, item: ItemId) -> Option<bool> {
    w.item(item).filter(|i| i.ty.is(&PLANK))?;
    let d = pd_mut(w, item)?;
    d.locked = !d.locked;
    Some(d.locked)
}

/// `Server.Items.Plank`; `[SerializationGenerator(1, false)]`.
pub static PLANK: ItemType = ItemType {
    full_name: "Server.Items.Plank",
    parent: Some(&ITEM_BASE),
    construct: None,
    on_move_over: plank_on_move_over,
    on_double_click: plank_on_double_click,
    on_double_click_dead: plank_on_double_click,
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        out.write_i32(1);
        let d = pd(w, id).cloned().unwrap_or(PlankData {
            boat: None,
            side: PlankSide::Port,
            locked: false,
            key_value: 0,
            close_timer: None,
        });
        out.write_item(w, d.boat);
        out.write_enum(4, d.side as i64);
        out.write_bool(d.locked);
        out.write_u32(d.key_value);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        let boat = read_boat(w, r)?;
        let side = if r.read_enum(4)? == 1 {
            PlankSide::Starboard
        } else {
            PlankSide::Port
        };
        let locked = r.read_bool()?;
        let key_value = r.read_u32()?;
        if let Some(i) = w.item_mut(id) {
            i.ext.insert(PlankData {
                boat,
                side,
                locked,
                key_value,
                close_timer: None,
            });
        }
        // [AfterDeserialization]
        if boat.is_none() {
            timer::start(w, TimeSpan::ZERO, TimeSpan::ZERO, 1, move |w| {
                delete_item(w, id)
            });
        } else if plank_is_open(w, id) {
            start_close_timer(w, id);
        }
        Ok(())
    },
    ..ITEM_BASE
};

// ---- TillerMan ----

/// What `TillerMan` adds.
#[derive(Clone, Debug, Default)]
pub struct TillerData {
    pub boat: Option<ItemId>,
}

fn tiller_boat(w: &World, id: ItemId) -> Option<ItemId> {
    w.item(id)?.ext.get::<TillerData>()?.boat
}

/// `new TillerMan(boat)`.
pub fn new_tiller_man(w: &mut World, boat: ItemId) -> ItemId {
    let id = ops::new_item_with_id(w, &TILLER_MAN, 0x3E4E);
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(TillerData { boat: Some(boat) });
    }
    ops::set_movable(w, id, false);
    id
}

/// `SetFacing(dir)`.
pub fn tiller_set_facing(w: &mut World, id: ItemId, dir: i32) {
    let v = match dir {
        4 => 0x3E4B,
        0 => 0x3E4E,
        6 => 0x3E50,
        2 => 0x3E55,
        _ => graphic(w, id),
    };
    set_graphic(w, id, v);
}

/// `Say(number)`.
pub fn tiller_say(w: &mut World, id: ItemId, number: i32) {
    muo_server::item::messages::public_overhead_message_number(
        w,
        id,
        MessageType::REGULAR,
        0x3B2,
        number,
        "",
    );
}

/// `Say(number, args)`.
pub fn tiller_say_args(w: &mut World, id: ItemId, number: i32, args: &str) {
    muo_server::item::messages::public_overhead_message_number(
        w,
        id,
        MessageType::REGULAR,
        0x3B2,
        number,
        args,
    );
}

/// `Hold`'s `Say`: the hold has none; the tiller man speaks for it.
pub fn hold_say(w: &mut World, boat: ItemId, number: i32) {
    if let Some(t) = super::tiller_man(w, boat) {
        tiller_say(w, t, number);
    }
}

/// `Server.Items.TillerMan`; `[SerializationGenerator(0, false)]`.
pub static TILLER_MAN: ItemType = ItemType {
    full_name: "Server.Items.TillerMan",
    parent: Some(&ITEM_BASE),
    construct: None,
    get_properties: |w, id, list: &mut PropertyList| {
        (ITEM_BASE.get_properties)(w, id, list);
        if let Some(b) = tiller_boat(w, id) {
            list.add(super::status(w, b));
        }
    },
    add_name_property: |w, id, list: &mut PropertyList| match tiller_boat(w, id)
        .and_then(|b| super::ship_name(w, b))
    {
        Some(n) => list.add_str(1042884, &n), // the tiller man of the ~1_SHIP_NAME~
        None => (ITEM_BASE.add_name_property)(w, id, list),
    },
    on_single_click: |w, id, from| match tiller_boat(w, id).and_then(|b| super::ship_name(w, b)) {
        Some(n) => ops::label_to_number(w, id, from, 1042884, &n), // the tiller man of the ~1_SHIP_NAME~
        None => (ITEM_BASE.on_single_click)(w, id, from),
    },
    on_double_click: |w, id, from| {
        let Some(b) = tiller_boat(w, id) else { return };
        if contains_mobile(w, b, from) {
            super::begin_rename(w, b, from);
        } else {
            super::begin_dry_dock(w, b, from);
        }
    },
    on_drag_drop: |w, id, from, dropped| {
        let boat = tiller_boat(w, id);
        let is_map = super::hooks(w)
            .and_then(|h| h.map_pins)
            .is_some_and(|f| f(w, dropped).is_some());
        if is_map
            && let Some(b) = boat
            && super::can_command(w, b, from)
            && contains_mobile(w, b, from)
        {
            super::associate_map(w, b, dropped);
        }
        false
    },
    on_after_delete: |w, id| {
        if let Some(b) = tiller_boat(w, id) {
            delete_item(w, b);
        }
    },
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        out.write_i32(0);
        out.write_item(w, tiller_boat(w, id));
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        let boat = read_boat(w, r)?;
        if let Some(i) = w.item_mut(id) {
            i.ext.insert(TillerData { boat });
        }
        // [AfterDeserialization]
        if boat.is_none() {
            timer::start(w, TimeSpan::ZERO, TimeSpan::ZERO, 1, move |w| {
                delete_item(w, id)
            });
        }
        Ok(())
    },
    ..ITEM_BASE
};

// ---- Hold ----

/// What `Hold` adds.
#[derive(Clone, Debug, Default)]
pub struct HoldData {
    pub boat: Option<ItemId>,
}

fn hold_boat(w: &World, id: ItemId) -> Option<ItemId> {
    w.item(id)?.ext.get::<HoldData>()?.boat
}

/// `new Hold(boat)`.
pub fn new_hold(w: &mut World, boat: ItemId) -> ItemId {
    let id = container::new_container(w, &HOLD, 0x3EAE);
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(HoldData { boat: Some(boat) });
    }
    ops::set_movable(w, id, false);
    id
}

/// `SetFacing(dir)`.
pub fn hold_set_facing(w: &mut World, id: ItemId, dir: i32) {
    let v = match dir {
        2 => 0x3E65,
        6 => 0x3E93,
        0 => 0x3EAE,
        4 => 0x3EB9,
        _ => graphic(w, id),
    };
    set_graphic(w, id, v);
}

/// `_boat?.Contains(from) == true && !_boat.IsMoving`.
fn boat_usable(w: &World, id: ItemId, from: MobileId) -> bool {
    hold_boat(w, id).is_some_and(|b| contains_mobile(w, b, from) && !super::is_moving(w, b))
}

/// `Server.Items.Hold`; `[SerializationGenerator(0, false)]`.
pub static HOLD: ItemType = ItemType {
    full_name: "Server.Items.Hold",
    parent: Some(&CONTAINER),
    construct: None,
    is_deco_container: |_, _| false,
    on_drag_drop: |w, id, from, item| {
        boat_usable(w, id, from) && (CONTAINER.on_drag_drop)(w, id, from, item)
    },
    on_drag_drop_into: |w, id, from, item, p| {
        boat_usable(w, id, from) && (CONTAINER.on_drag_drop_into)(w, id, from, item, p)
    },
    check_item_use: |w, id, from, item| {
        (item == id || boat_usable(w, id, from)) && (CONTAINER.check_item_use)(w, id, from, item)
    },
    check_lift: |w, id, from, item, reject: &mut LRReason| {
        boat_usable(w, id, from) && (CONTAINER.check_lift)(w, id, from, item, reject)
    },
    on_after_delete: |w, id| {
        if let Some(b) = hold_boat(w, id) {
            delete_item(w, b);
        }
    },
    on_double_click: |w, id, from| {
        let boat = hold_boat(w, id);
        match boat {
            Some(b) if contains_mobile(w, b, from) => {
                if super::is_moving(w, b) {
                    hold_say(w, b, 502491); // I can not open the hold while the ship is moving.
                } else {
                    (CONTAINER.on_double_click)(w, id, from);
                }
            }
            Some(b) => hold_say(w, b, 502490), // You must be on the ship to open the hold.
            None => {}
        }
    },
    serialize: |w, id, out| {
        (CONTAINER.serialize)(w, id, out);
        out.write_i32(0);
        out.write_item(w, hold_boat(w, id));
    },
    deserialize: |w, id, r| {
        (CONTAINER.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        let boat = read_boat(w, r)?;
        if let Some(i) = w.item_mut(id) {
            i.ext.insert(HoldData { boat });
        }
        // [AfterDeserialization]
        if boat.is_none() || w.item(id).is_some_and(|i| i.parent.is_some()) {
            timer::start(w, TimeSpan::ZERO, TimeSpan::ZERO, 1, move |w| {
                delete_item(w, id)
            });
        }
        Ok(())
    },
    ..CONTAINER
};

pub fn register(types: &mut TypeRegistry) {
    types.register_item(&PLANK, &[]);
    types.register_item(&TILLER_MAN, &[]);
    types.register_item(&HOLD, &[]);
}
