//! Doors: `BaseDoor`, the plain doors, the house doors, portcullises and secret doors.
//!
//! State is [`DoorData`] in `Item::ext` (the house door's `Level` and `Facing` are in the same struct); the
//! `virtual`s of `BaseDoor` (`UseLocks`, `IsInside`, `Use`, `OnOpened`, `OnClosed`) are [`DoorClass`].
//!
//! `UseChainedFunctionality` is `false` in every class, but [`chain`] (`GetChain()`) and the chained branches of
//! `IsFreeToClose` / `Use` are implemented; the `Link` / `ChainLink` commands are [`link_commands`].
//!
//! `SetSecureLevelEntry` of `BaseHouseDoor.GetContextMenuEntries` is the house system's `secure_level_entry` hook
//! which takes `securable_level` (set here).

use muo_content::items::gold;
use muo_content::items::house_deps::{DoorFacing, HouseDoorKind, deps_mut};
use muo_content::props::{Ctor, Kind, Param, Value, add_ctors};
use muo_content::utilities::types::{Numeric, PropType};
use muo_io::{GenericReader, GenericWriter};
use muo_primitives::{Point3D, TimeSpan};
use muo_server::World;
use muo_server::ext::Ext;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType, deps, messages, ops};
use muo_server::map::MapId;
use muo_server::mobile::AccessLevel;
use muo_server::packets::message::MessageType;
use muo_server::serialization_ext::{ReadEntityExt, WriteEntityExt};
use muo_server::timer::{self, TimerHandle};
use muo_server::type_registry::{Phase, TypeRegistry};

/// `BaseDoor.m_Offsets`.
const OFFSETS: [(i32, i32, i32); 12] = [
    (-1, 1, 0),
    (1, 1, 0),
    (-1, 0, 0),
    (1, -1, 0),
    (1, 1, 0),
    (1, -1, 0),
    (0, 0, 0),
    (0, -1, 0),
    (0, 0, 0),
    (0, 0, 0),
    (0, 0, 0),
    (0, 0, 0),
];

/// `BaseDoor.GetOffset(facing)`.
pub fn get_offset(facing: DoorFacing) -> Point3D {
    let (x, y, z) = OFFSETS[(facing as i32).clamp(0, 11) as usize];
    Point3D::new(x, y, z)
}

/// The names `[add IronGate WestCW` parses.
pub static FACING_NAMES: &[(&str, i64)] = &[
    ("WestCW", 0),
    ("EastCCW", 1),
    ("WestCCW", 2),
    ("EastCW", 3),
    ("SouthCW", 4),
    ("NorthCCW", 5),
    ("SouthCCW", 6),
    ("NorthCW", 7),
    ("SouthSW", 8),
    ("SouthSE", 9),
    ("WestSS", 10),
    ("WestSN", 11),
];

/// What `BaseDoor` (and `BaseHouseDoor`) add to an `Item`.
#[derive(Clone, Debug)]
pub struct DoorData {
    pub key_value: u32,
    pub open: bool,
    pub locked: bool,
    pub opened_id: i32,
    pub closed_id: i32,
    pub opened_sound: i32,
    pub closed_sound: i32,
    pub offset: Point3D,
    pub link: Option<ItemId>,
    pub timer: Option<TimerHandle>,
    /// `BaseHouseDoor._level` (a `SecureLevel` int).
    pub level: i32,
    /// `BaseHouseDoor._facing`.
    pub facing: i32,
}

fn data(w: &World, id: ItemId) -> Option<&DoorData> {
    w.item(id)?.ext.get::<DoorData>()
}

fn data_mut(w: &mut World, id: ItemId) -> Option<&mut DoorData> {
    w.item_mut(id)?.ext.get_mut::<DoorData>()
}

/// `item is BaseDoor`.
pub fn is_door(w: &World, id: ItemId) -> bool {
    w.item(id).is_some_and(|i| i.ty.is(&BASE_DOOR))
}

/// `item is BaseHouseDoor`.
pub fn is_house_door(w: &World, id: ItemId) -> bool {
    w.item(id).is_some_and(|i| i.ty.is(&BASE_HOUSE_DOOR))
}

pub fn locked(w: &World, id: ItemId) -> bool {
    data(w, id).is_some_and(|d| d.locked)
}

pub fn set_locked(w: &mut World, id: ItemId, v: bool) {
    if let Some(d) = data_mut(w, id) {
        d.locked = v;
    }
}

pub fn key_value(w: &World, id: ItemId) -> u32 {
    data(w, id).map_or(0, |d| d.key_value)
}

pub fn set_key_value(w: &mut World, id: ItemId, v: u32) {
    if let Some(d) = data_mut(w, id) {
        d.key_value = v;
    }
}

/// `ClosedId`.
pub fn closed_id_of(w: &World, id: ItemId) -> i32 {
    data(w, id).map_or(0, |d| d.closed_id)
}

pub fn is_open(w: &World, id: ItemId) -> bool {
    data(w, id).is_some_and(|d| d.open)
}

pub fn offset(w: &World, id: ItemId) -> Point3D {
    data(w, id).map_or(Point3D::new(0, 0, 0), |d| d.offset)
}

/// `Link = link`.
pub fn set_link(w: &mut World, id: ItemId, link: Option<ItemId>) {
    if let Some(d) = data_mut(w, id) {
        d.link = link;
    }
}

/// The `virtual`s of `BaseDoor`.
#[derive(Copy, Clone)]
pub struct DoorClass {
    pub use_locks: fn(&World, ItemId) -> bool,
    pub is_inside: fn(&World, ItemId, MobileId) -> bool,
    pub use_: fn(&mut World, ItemId, MobileId),
    pub on_opened: fn(&mut World, ItemId, MobileId),
    pub on_closed: fn(&mut World, ItemId, MobileId),
    /// `UseChainedFunctionality`.
    pub use_chained_functionality: bool,
}

pub static BASE_DOOR_CLASS: DoorClass = DoorClass {
    use_locks: |_, _| true,
    is_inside: |_, _, _| false,
    use_: use_base,
    on_opened: |_, _, _| {},
    on_closed: |_, _, _| {},
    use_chained_functionality: false,
};

fn class(w: &World, id: ItemId) -> &'static DoorClass {
    w.item(id)
        .and_then(|i| i.ty.family::<DoorClass>())
        .unwrap_or(&BASE_DOOR_CLASS)
}

/// `door.UseLocks()`.
pub fn use_locks(w: &World, id: ItemId) -> bool {
    (class(w, id).use_locks)(w, id)
}

/// `door.Use(from)`.
pub fn use_door(w: &mut World, id: ItemId, from: MobileId) {
    (class(w, id).use_)(w, id, from);
}

/// `BaseDoor.OnTelekinesis(from)` (`ITelekinesisable`): particles and a sound at the door, then `Use(from)`. False
/// when `id` is no door.
pub fn on_telekinesis(w: &mut World, id: ItemId, from: MobileId) -> bool {
    if !is_door(w, id) {
        return false;
    }
    let (loc, map) = w
        .item(id)
        .map_or((Point3D::default(), None), |i| (i.location, i.map));
    let e = muo_server::item::effect_item::create_targeted(
        w,
        loc,
        map,
        muo_server::item::effect_item::DEFAULT_DURATION,
    );
    muo_server::effects::send_location_particles(w, e, 0x376A, 9, 32, 5022);
    muo_server::effects::play_sound(w, loc, map, 0x1F5);
    use_door(w, id, from);
    true
}

/// `new BaseDoor(closedID, openedID, openedSound, closedSound, offset)` of class `ty`.
pub fn new_door(
    w: &mut World,
    ty: &'static ItemType,
    closed_id: i32,
    opened_id: i32,
    opened_sound: i32,
    closed_sound: i32,
    offset: Point3D,
) -> ItemId {
    let id = ops::new_item_with_id(w, ty, closed_id);
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(DoorData {
            key_value: 0,
            open: false,
            locked: false,
            opened_id,
            closed_id,
            opened_sound,
            closed_sound,
            offset,
            link: None,
            timer: None,
            level: 3, // SecureLevel.Anyone
            facing: 0,
        });
    }
    ops::set_movable(w, id, false);
    id
}

/// `Open = value`, with `OnOpenChanged`.
pub fn set_open(w: &mut World, id: ItemId, value: bool) {
    let Some(d) = data_mut(w, id) else { return };
    if d.open == value {
        return;
    }
    d.open = value;
    let d = d.clone();
    if let Some(ty) = w.item(id).map(|i| i.ty) {
        (ty.set_item_id)(w, id, if value { d.opened_id } else { d.closed_id });
    }
    let l = w
        .item(id)
        .map(|i| i.location)
        .unwrap_or(Point3D::new(0, 0, 0));
    let o = d.offset;
    let to = if value {
        Point3D::new(l.x + o.x, l.y + o.y, l.z + o.z)
    } else {
        Point3D::new(l.x - o.x, l.y - o.y, l.z - o.z)
    };
    ops::set_location(w, id, to);
    let (loc, map) = (
        ops::get_world_location(w, id),
        w.item(id).and_then(|i| i.map),
    );
    muo_server::effects::play_sound(
        w,
        loc,
        map,
        if value {
            d.opened_sound
        } else {
            d.closed_sound
        },
    );
    if value {
        if d.timer.is_none() {
            start_timer(w, id);
        }
    } else {
        stop_timer(w, id);
    }
}

fn start_timer(w: &mut World, id: ItemId) {
    let t = timer::start(
        w,
        TimeSpan::from_secs_f64(20.0),
        TimeSpan::from_secs_f64(10.0),
        0,
        move |w| {
            if w.item(id).is_none_or(|i| i.deleted()) {
                stop_timer(w, id);
                return;
            }
            if is_open(w, id) && is_free_to_close(w, id) {
                set_open(w, id, false);
            }
        },
    );
    if let Some(d) = data_mut(w, id) {
        d.timer = Some(t);
    }
}

fn stop_timer(w: &mut World, id: ItemId) {
    let t = data_mut(w, id).and_then(|d| d.timer.take());
    if let Some(mut t) = t {
        t.cancel(w);
    }
}

fn check_fit(w: &World, map: MapId, p: Point3D, height: i32) -> bool {
    if map == MapId::INTERNAL {
        return false;
    }
    for item in muo_server::map::enumerate::items_at(w, Some(map), p.x, p.y) {
        let Some(i) = w.item(item) else { continue };
        if i.item_id as usize <= muo_data::tile_data::TileData::MAX_ITEM_VALUE
            && !i.ty.is(&muo_server::item::multi::BASE_MULTI)
            && !i.ty.is(&BASE_DOOR)
        {
            let id = w.maps.tile_data.item(i.item_id as usize & 0xFFFF);
            if (id.surface() || id.impassable())
                && i.location.z + id.calc_height() > p.z
                && p.z + height > i.location.z
            {
                return false;
            }
        }
    }
    for m in muo_server::map::enumerate::mobiles_at(w, Some(map), p.x, p.y) {
        let Some(mob) = w.mobile(m) else { continue };
        if (!mob.hidden || mob.access_level == AccessLevel::Player)
            && muo_server::mobile::base::alive(w, m)
            && mob.location.z + 16 > p.z
            && p.z + height > mob.location.z
        {
            return false;
        }
    }
    true
}

/// `door.CanClose()`.
pub fn can_close(w: &World, id: ItemId) -> bool {
    let Some(d) = data(w, id) else { return true };
    if !d.open {
        return true;
    }
    let Some(i) = w.item(id) else { return false };
    let Some(map) = i.map else { return false };
    let l = i.location;
    check_fit(
        w,
        map,
        Point3D::new(l.x - d.offset.x, l.y - d.offset.y, l.z - d.offset.z),
        16,
    )
}

/// `door.GetChain()`: the door, then each `Link` that is not deleted and not already in the chain (links are set
/// pairwise by GMs, so a chain can loop back to a door other than the start).
pub fn chain(w: &World, id: ItemId) -> Vec<ItemId> {
    let mut v = vec![id];
    let mut cur = id;
    while let Some(next) = data(w, cur).and_then(|d| d.link) {
        if w.item(next).is_none_or(|i| i.deleted()) || v.contains(&next) {
            break;
        }
        v.push(next);
        cur = next;
    }
    v
}

/// `door.IsFreeToClose()`.
pub fn is_free_to_close(w: &World, id: ItemId) -> bool {
    if !class(w, id).use_chained_functionality {
        return can_close(w, id);
    }
    chain(w, id).into_iter().all(|l| can_close(w, l))
}

fn local_msg(w: &mut World, from: MobileId, number: i32) {
    deps::local_overhead_message(w, from, MessageType::REGULAR, 0x3B2, number);
}

fn use_base(w: &mut World, id: ItemId, from: MobileId) {
    let open = is_open(w, id);
    if locked(w, id) && !open && use_locks(w, id) {
        let gm = w
            .mobile(from)
            .is_some_and(|m| m.access_level >= AccessLevel::GameMaster);
        let pack = muo_server::mobile::part1::props_b::backpack(w, from);
        if gm {
            local_msg(w, from, 502502); // That is locked, but you open it with your godly powers.
        } else if pack.is_some_and(|p| gold::contains_key(w, p, key_value(w, id))) {
            local_msg(w, from, 501282); // You quickly unlock, open, and relock the door
        } else if (class(w, id).is_inside)(w, id, from) {
            local_msg(w, from, 501280); // That is locked, but is usable from the inside.
        } else {
            let doom = w.item(id).is_some_and(|i| i.map == Some(MapId::MALAS))
                && (w.item(id).unwrap().ty.hue)(w, id) == 0x44E;
            if doom {
                messages::send_localized_message_to(w, id, from, 1060014, ""); // Only the dead may pass.
            } else {
                local_msg(w, from, 502503); // That is locked.
            }
            return;
        }
    }
    if open && !is_free_to_close(w, id) {
        return;
    }
    if open {
        (class(w, id).on_closed)(w, id, from);
    } else {
        (class(w, id).on_opened)(w, id, from);
    }
    if class(w, id).use_chained_functionality {
        for l in chain(w, id) {
            set_open(w, l, !open);
        }
        return;
    }
    set_open(w, id, !open);
    let link = data(w, id).and_then(|d| d.link);
    if is_open(w, id)
        && let Some(l) = link
        && w.item(l).is_some_and(|i| !i.deleted())
        && !is_open(w, l)
    {
        set_open(w, l, true);
    }
}

fn on_double_click(w: &mut World, id: ItemId, from: MobileId) {
    let far = w
        .mobile(from)
        .is_some_and(|m| m.access_level == AccessLevel::Player)
        && !deps::mobile_in_range(w, from, ops::get_world_location(w, id), 2);
    if far {
        local_msg(w, from, 1019045); // I can't reach that.
    } else {
        use_door(w, id, from);
    }
}

fn on_delete(w: &mut World, id: ItemId) {
    stop_timer(w, id);
    set_link(w, id, None);
}

/// `BaseDoor.OpenDoorMacroUsed(m)`.
pub fn open_door_macro_used(w: &mut World, m: MobileId) {
    let Some(mob) = w.mobile(m) else { return };
    let Some(map) = mob.map else { return };
    if !muo_server::mobile::part2::stats::check_alive(w, m, true) {
        return;
    }
    let mob = w.mobile(m).expect("checked");
    let (mut x, mut y, z) = (mob.location.x, mob.location.y, mob.location.z);
    muo_server::movement::offset_xy(mob.direction, &mut x, &mut y, 1);
    let items: Vec<ItemId> = muo_server::map::enumerate::items_at(w, Some(map), x, y).collect();
    for item in items {
        let Some(i) = w.item(item) else { continue };
        let height = i32::from(ops::item_data(w, item).height);
        if i.location.z + height > z
            && z + 16 > i.location.z
            && is_door(w, item)
            && deps::can_see(w, m, item)
            && deps::in_los_item(w, m, item)
        {
            deps::send_localized_message(w, m, 500024); // Opening door...
            if let Some(ty) = w.item(item).map(|i| i.ty) {
                (ty.on_double_click)(w, item, m);
            }
            break;
        }
    }
}

const VERSION: i32 = 0;

fn serialize(w: &World, id: ItemId, out: &mut dyn GenericWriter) {
    (ITEM_BASE.serialize)(w, id, out);
    out.write_i32(VERSION);
    let Some(d) = data(w, id) else { return };
    out.write_u32(d.key_value);
    out.write_bool(d.open);
    out.write_bool(d.locked);
    out.write_i32(d.opened_id);
    out.write_i32(d.closed_id);
    out.write_i32(d.opened_sound);
    out.write_i32(d.closed_sound);
    out.write_point3d(d.offset);
    out.write_item(w, d.link);
}

fn deserialize(w: &mut World, id: ItemId, r: &mut dyn GenericReader) -> muo_io::Result<()> {
    (ITEM_BASE.deserialize)(w, id, r)?;
    let _version = r.read_i32()?;
    let d = DoorData {
        key_value: r.read_u32()?,
        open: r.read_bool()?,
        locked: r.read_bool()?,
        opened_id: r.read_i32()?,
        closed_id: r.read_i32()?,
        opened_sound: r.read_i32()?,
        closed_sound: r.read_i32()?,
        offset: r.read_point3d()?,
        link: r.read_item(w)?,
        timer: None,
        level: 3,
        facing: 0,
    };
    let open = d.open;
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(d);
    }
    if open {
        start_timer(w, id); // AfterDeserialization
    }
    Ok(())
}

/// `Server.Items.BaseDoor`.
pub static BASE_DOOR: ItemType = ItemType {
    full_name: "Server.Items.BaseDoor",
    construct: None,
    parent: Some(&ITEM_BASE),
    family: Some(&BASE_DOOR_CLASS),
    on_double_click,
    on_delete,
    serialize,
    deserialize,
    ..ITEM_BASE
};

// ---- BaseHouseDoor ----

fn house_of(w: &World, id: ItemId) -> Option<ItemId> {
    let d = data(w, id)?;
    let loc = w.item(id)?.location;
    let loc = if d.open {
        Point3D::new(loc.x - d.offset.x, loc.y - d.offset.y, loc.z - d.offset.z)
    } else {
        loc
    };
    muo_multis::house::access::find_house_at_point(w, loc, w.item(id)?.map, 20)
}

fn check_access(w: &World, id: ItemId, m: MobileId) -> bool {
    use muo_multis::house::{SecureLevel, access, is_aos_rules};
    let Some(house) = house_of(w, id) else {
        return false;
    };
    if !is_aos_rules(w, house) {
        return true;
    }
    let public = muo_multis::house::data(w, house).is_some_and(|h| h.public);
    if if public {
        access::is_banned(w, house, m)
    } else {
        !access::has_access(w, house, m)
    } {
        return false;
    }
    let level = SecureLevel::from_u8(data(w, id).map_or(3, |d| d.level) as u8);
    access::has_secure_access(w, house, m, level)
}

fn house_on_opened(w: &mut World, id: ItemId, from: MobileId) {
    use muo_multis::house::{access, decay};
    let Some(house) = house_of(w, id) else { return };
    let friend = access::is_friend(w, house, from);
    let player = w
        .mobile(from)
        .is_some_and(|m| m.access_level == AccessLevel::Player);
    if friend && player && decay::refresh_decay(w, house) {
        deps::send_localized_message(w, from, 1043293); // Your house's age and contents have been refreshed.
    }
    let public = muo_multis::house::data(w, house).is_some_and(|h| h.public);
    if public
        && !friend
        && let Some(h) = muo_multis::house::data_mut(w, house)
    {
        h.visits += 1;
    }
}

fn house_is_inside(w: &World, id: ItemId, from: MobileId) -> bool {
    let (r, bs, ss) = (2, 5, 3);
    let (x, y, bw, bh) = match data(w, id).map_or(0, |d| d.facing) {
        0 | 1 => (-r, -r, bs, ss),
        3 | 2 => (-r, 0, bs, ss),
        4 | 5 => (-r, -r, ss, bs),
        7 | 6 => (0, -r, ss, bs),
        _ => return false,
    };
    let (Some(i), Some(m)) = (w.item(id), w.mobile(from)) else {
        return false;
    };
    let rx = m.location.x - i.location.x;
    let ry = m.location.y - i.location.y;
    let az = (m.location.z - i.location.z).abs();
    rx >= x && rx < x + bw && ry >= y && ry < y + bh && az <= 4
}

static BASE_HOUSE_DOOR_CLASS: DoorClass = DoorClass {
    use_locks: |w, id| !house_of(w, id).is_some_and(|h| muo_multis::house::is_aos_rules(w, h)),
    is_inside: house_is_inside,
    use_: |w, id, from| {
        if !check_access(w, id, from) {
            deps::send_localized_message(w, from, 1061637); // You are not allowed to access this.
        } else {
            use_base(w, id, from);
        }
    },
    on_opened: house_on_opened,
    ..BASE_DOOR_CLASS
};

/// `Server.Items.BaseHouseDoor`; `[SerializationGenerator(2, false)]`.
pub static BASE_HOUSE_DOOR: ItemType = ItemType {
    full_name: "Server.Items.BaseHouseDoor",
    construct: None,
    parent: Some(&BASE_DOOR),
    family: Some(&BASE_HOUSE_DOOR_CLASS),
    serialize: |w, id, out| {
        (BASE_DOOR.serialize)(w, id, out);
        out.write_i32(2);
        let d = data(w, id);
        out.write_i32(d.map_or(3, |d| d.level));
        out.write_i32(d.map_or(0, |d| d.facing));
    },
    deserialize: |w, id, r| {
        (BASE_DOOR.deserialize)(w, id, r)?;
        let version = r.read_i32()?;
        // `Deserialize(reader, version)` for an older version reads the same two ints.
        let _ = version;
        let level = r.read_i32()?;
        let facing = r.read_i32()?;
        if let Some(d) = data_mut(w, id) {
            d.level = level;
            d.facing = facing;
        }
        Ok(())
    },
    ..BASE_DOOR
};

// ---- the leaves ----

/// `new FooDoor(facing)`: closed id `base + 2 * facing`, opened id one more.
pub fn facing_door(
    w: &mut World,
    ty: &'static ItemType,
    facing: DoorFacing,
    base: i32,
    os: i32,
    cs: i32,
) -> ItemId {
    let n = 2 * facing as i32;
    new_door(w, ty, base + n, base + 1 + n, os, cs, get_offset(facing))
}

/// `new FooHouseDoor(facing, closed, opened, os, cs, offset)`.
pub fn new_house_door(
    w: &mut World,
    ty: &'static ItemType,
    facing: DoorFacing,
    base: i32,
    os: i32,
    cs: i32,
    auto: bool,
) -> ItemId {
    let n = if auto { 2 * facing as i32 } else { 0 };
    let id = new_door(w, ty, base + n, base + 1 + n, os, cs, get_offset(facing));
    if let Some(d) = data_mut(w, id) {
        d.facing = facing as i32;
        d.level = 3; // SecureLevel.Anyone
    }
    id
}

macro_rules! door_leaves {
    ($parent:ident, $make:expr; $($name:ident = $full:literal, $base:expr, $os:expr, $cs:expr;)*) => {$(
        #[doc = concat!("`", $full, "`.")]
        pub static $name: ItemType = ItemType {
            full_name: $full,
            parent: Some(&$parent),
            // the facing is a constructor argument: no parameterless `[Constructible]`
            construct: None,
            make: Some(|w| $make(w, &$name, DoorFacing::WestCW, $base, $os, $cs)),
            serialize: |w, id, out| {
                ($parent.serialize)(w, id, out);
                out.write_i32(0);
            },
            deserialize: |w, id, r| {
                ($parent.deserialize)(w, id, r)?;
                let _version = r.read_i32()?;
                Ok(())
            },
            ..$parent
        };
    )*};
}

door_leaves! { BASE_DOOR, facing_door;
    IRON_GATE_SHORT = "Server.Items.IronGateShort", 0x84C, 0xEC, 0xF3;
    IRON_GATE = "Server.Items.IronGate", 0x824, 0xEC, 0xF3;
    LIGHT_WOOD_GATE = "Server.Items.LightWoodGate", 0x839, 0xEB, 0xF2;
    DARK_WOOD_GATE = "Server.Items.DarkWoodGate", 0x866, 0xEB, 0xF2;
    METAL_DOOR = "Server.Items.MetalDoor", 0x675, 0xEC, 0xF3;
    BARRED_METAL_DOOR = "Server.Items.BarredMetalDoor", 0x685, 0xEC, 0xF3;
    BARRED_METAL_DOOR2 = "Server.Items.BarredMetalDoor2", 0x1FED, 0xEC, 0xF3;
    RATTAN_DOOR = "Server.Items.RattanDoor", 0x695, 0xEB, 0xF2;
    DARK_WOOD_DOOR = "Server.Items.DarkWoodDoor", 0x6A5, 0xEA, 0xF1;
    MEDIUM_WOOD_DOOR = "Server.Items.MediumWoodDoor", 0x6B5, 0xEA, 0xF1;
    METAL_DOOR2 = "Server.Items.MetalDoor2", 0x6C5, 0xEC, 0xF3;
    LIGHT_WOOD_DOOR = "Server.Items.LightWoodDoor", 0x6D5, 0xEA, 0xF1;
    STRONG_WOOD_DOOR = "Server.Items.StrongWoodDoor", 0x6E5, 0xEA, 0xF1;
    SECRET_STONE_DOOR1 = "Server.Items.SecretStoneDoor1", 0xE8, 0xED, 0xF4;
    SECRET_DUNGEON_DOOR = "Server.Items.SecretDungeonDoor", 0x314, 0xED, 0xF4;
    SECRET_STONE_DOOR2 = "Server.Items.SecretStoneDoor2", 0x324, 0xED, 0xF4;
    SECRET_WOODEN_DOOR = "Server.Items.SecretWoodenDoor", 0x334, 0xED, 0xF4;
    SECRET_LIGHT_WOOD_DOOR = "Server.Items.SecretLightWoodDoor", 0x344, 0xED, 0xF4;
    SECRET_STONE_DOOR3 = "Server.Items.SecretStoneDoor3", 0x354, 0xED, 0xF4;
}

door_leaves! { BASE_HOUSE_DOOR, |w, ty, f, b, os, cs| new_house_door(w, ty, f, b, os, cs, true);
    METAL_HOUSE_DOOR = "Server.Items.MetalHouseDoor", 0x675, 0xEC, 0xF3;
    DARK_WOOD_HOUSE_DOOR = "Server.Items.DarkWoodHouseDoor", 0x6A5, 0xEA, 0xF1;
    GENERIC_HOUSE_DOOR = "Server.Items.GenericHouseDoor", 0x675, 0xEC, 0xF3;
}

macro_rules! portcullis {
    ($name:ident, $full:literal, $gid:expr) => {
        #[doc = concat!("`", $full, "`.")]
        pub static $name: ItemType = ItemType {
            full_name: $full,
            parent: Some(&BASE_DOOR),
            construct: Some(|w| {
                new_door(w, &$name, $gid, $gid, 0xF0, 0xEF, Point3D::new(0, 0, 20))
            }),
            serialize: |w, id, out| {
                (BASE_DOOR.serialize)(w, id, out);
                out.write_i32(0);
            },
            deserialize: |w, id, r| {
                (BASE_DOOR.deserialize)(w, id, r)?;
                let _version = r.read_i32()?;
                Ok(())
            },
            ..BASE_DOOR
        };
    };
}
portcullis!(PORTCULLIS_NS, "Server.Items.PortcullisNS", 0x6F5);
portcullis!(PORTCULLIS_EW, "Server.Items.PortcullisEW", 0x6F6);

// ---- constructors for `[add` ----

const INT: Kind = Kind::Plain(PropType::Numeric(Numeric::Int));
const FACING: Kind = Kind::Plain(PropType::Enum(FACING_NAMES));
static FACING_PARAM: [Param; 1] = [Param {
    name: "facing",
    kind: FACING,
    default: None,
    ty: "DoorFacing",
}];

fn facing_of(v: &[Value]) -> DoorFacing {
    DoorFacing::from_i32(v[0].i32())
}

macro_rules! facing_ctors {
    ($($name:ident, $ctors:ident, $base:expr, $os:expr, $cs:expr;)*) => {$(
        static $ctors: [Ctor; 1] = [Ctor {
            access: None,
            params: &FACING_PARAM,
            make: |w, v| EntityId::Item(facing_door(w, &$name, facing_of(v), $base, $os, $cs)),
        }];
    )*};
}
facing_ctors! {
    IRON_GATE_SHORT, C_IRON_GATE_SHORT, 0x84C, 0xEC, 0xF3;
    IRON_GATE, C_IRON_GATE, 0x824, 0xEC, 0xF3;
    LIGHT_WOOD_GATE, C_LIGHT_WOOD_GATE, 0x839, 0xEB, 0xF2;
    DARK_WOOD_GATE, C_DARK_WOOD_GATE, 0x866, 0xEB, 0xF2;
    METAL_DOOR, C_METAL_DOOR, 0x675, 0xEC, 0xF3;
    BARRED_METAL_DOOR, C_BARRED_METAL_DOOR, 0x685, 0xEC, 0xF3;
    BARRED_METAL_DOOR2, C_BARRED_METAL_DOOR2, 0x1FED, 0xEC, 0xF3;
    RATTAN_DOOR, C_RATTAN_DOOR, 0x695, 0xEB, 0xF2;
    DARK_WOOD_DOOR, C_DARK_WOOD_DOOR, 0x6A5, 0xEA, 0xF1;
    MEDIUM_WOOD_DOOR, C_MEDIUM_WOOD_DOOR, 0x6B5, 0xEA, 0xF1;
    METAL_DOOR2, C_METAL_DOOR2, 0x6C5, 0xEC, 0xF3;
    LIGHT_WOOD_DOOR, C_LIGHT_WOOD_DOOR, 0x6D5, 0xEA, 0xF1;
    STRONG_WOOD_DOOR, C_STRONG_WOOD_DOOR, 0x6E5, 0xEA, 0xF1;
    SECRET_STONE_DOOR1, C_SECRET_STONE_DOOR1, 0xE8, 0xED, 0xF4;
    SECRET_DUNGEON_DOOR, C_SECRET_DUNGEON_DOOR, 0x314, 0xED, 0xF4;
    SECRET_STONE_DOOR2, C_SECRET_STONE_DOOR2, 0x324, 0xED, 0xF4;
    SECRET_WOODEN_DOOR, C_SECRET_WOODEN_DOOR, 0x334, 0xED, 0xF4;
    SECRET_LIGHT_WOOD_DOOR, C_SECRET_LIGHT_WOOD_DOOR, 0x344, 0xED, 0xF4;
    SECRET_STONE_DOOR3, C_SECRET_STONE_DOOR3, 0x354, 0xED, 0xF4;
}

static C_METAL_HOUSE_DOOR: [Ctor; 1] = [Ctor {
    access: None,
    params: &FACING_PARAM,
    make: |w, v| {
        EntityId::Item(new_house_door(
            w,
            &METAL_HOUSE_DOOR,
            facing_of(v),
            0x675,
            0xEC,
            0xF3,
            true,
        ))
    },
}];
static C_DARK_WOOD_HOUSE_DOOR: [Ctor; 1] = [Ctor {
    access: None,
    params: &FACING_PARAM,
    make: |w, v| {
        EntityId::Item(new_house_door(
            w,
            &DARK_WOOD_HOUSE_DOOR,
            facing_of(v),
            0x6A5,
            0xEA,
            0xF1,
            true,
        ))
    },
}];
static GENERIC_PARAMS: [Param; 5] = [
    Param {
        name: "facing",
        kind: FACING,
        default: None,
        ty: "DoorFacing",
    },
    Param {
        name: "baseItemID",
        kind: INT,
        default: None,
        ty: "",
    },
    Param {
        name: "openedSound",
        kind: INT,
        default: None,
        ty: "",
    },
    Param {
        name: "closedSound",
        kind: INT,
        default: None,
        ty: "",
    },
    Param {
        name: "autoAdjust",
        kind: Kind::Plain(PropType::Bool),
        default: Some(Value::Bool(true)),
        ty: "",
    },
];
static C_GENERIC_HOUSE_DOOR: [Ctor; 1] = [Ctor {
    access: None,
    params: &GENERIC_PARAMS,
    make: |w, v| {
        EntityId::Item(new_house_door(
            w,
            &GENERIC_HOUSE_DOOR,
            facing_of(v),
            v[1].i32(),
            v[2].i32(),
            v[3].i32(),
            v[4].bool(),
        ))
    },
}];
static C_PORTCULLIS_NS: [Ctor; 1] = [Ctor {
    access: None,
    params: &[],
    make: |w, _| EntityId::Item((PORTCULLIS_NS.construct.expect("set"))(w)),
}];
static C_PORTCULLIS_EW: [Ctor; 1] = [Ctor {
    access: None,
    params: &[],
    make: |w, _| EntityId::Item((PORTCULLIS_EW.construct.expect("set"))(w)),
}];

// ---- hooks ----

fn make_house_door(w: &mut World, kind: HouseDoorKind) -> Option<ItemId> {
    Some(match kind {
        HouseDoorKind::DarkWood(f) => {
            new_house_door(w, &DARK_WOOD_HOUSE_DOOR, f, 0x6A5, 0xEA, 0xF1, true)
        }
        HouseDoorKind::Metal(f) => new_house_door(w, &METAL_HOUSE_DOOR, f, 0x675, 0xEC, 0xF3, true),
        HouseDoorKind::Named(name, f) => {
            return muo_content::factory::make_item_args(w, name, &[Value::Enum(f as i64)]);
        }
        HouseDoorKind::Generic {
            facing,
            base_item_id,
            open_sound,
            close_sound,
            auto_adjust,
        } => new_house_door(
            w,
            &GENERIC_HOUSE_DOOR,
            facing,
            base_item_id,
            open_sound,
            close_sound,
            auto_adjust,
        ),
    })
}

fn tell(w: &mut World, to: MobileId, text: &str) {
    muo_server::mobile::part2::messages::send_message(w, to, text);
}

fn target_door(t: &muo_server::target::Targeted, w: &World) -> Option<ItemId> {
    match t {
        muo_server::target::Targeted::Entity(EntityId::Item(i)) if is_door(w, *i) => Some(*i),
        _ => None,
    }
}

/// `Link_OnFirstTarget` / `Link_OnSecondTarget` (`first` is None for the first).
fn link_target(w: &mut World, from: MobileId, first: Option<ItemId>) {
    muo_server::target::begin_target(
        w,
        from,
        -1,
        false,
        muo_server::target::TargetFlags::NONE,
        move |w, from, t| match (target_door(&t, w), first) {
            (None, _) => {
                link_target(w, from, first);
                tell(w, from, "That is not a door. Try again.");
            }
            (Some(d), None) => {
                link_target(w, from, Some(d));
                tell(w, from, "Target the second door to link.");
            }
            (Some(second), Some(first)) => {
                set_link(w, first, Some(second));
                set_link(w, second, Some(first));
                tell(w, from, "The doors have been linked.");
            }
        },
    );
}

/// `ChainLink_OnTarget`.
fn chain_link_target(w: &mut World, from: MobileId, list: Vec<ItemId>) {
    muo_server::target::begin_target(
        w,
        from,
        -1,
        false,
        muo_server::target::TargetFlags::NONE,
        move |w, from, t| {
            let mut list = list.clone();
            let Some(door) = target_door(&t, w) else {
                chain_link_target(w, from, list);
                tell(w, from, "That is not a door. Try again.");
                return;
            };
            if list.first() == Some(&door) {
                if list.len() >= 2 {
                    for i in 0..list.len() {
                        set_link(w, list[i], Some(list[(i + 1) % list.len()]));
                    }
                    tell(w, from, "The chain of doors have been linked.");
                } else {
                    chain_link_target(w, from, list);
                    tell(
                        w,
                        from,
                        "You have not yet targeted two unique doors. Target the second door to link.",
                    );
                }
            } else if list.contains(&door) {
                chain_link_target(w, from, list);
                tell(
                    w,
                    from,
                    "You have already targeted that door. Target another door, or retarget the first door to complete the chain.",
                );
            } else {
                list.push(door);
                let first_only = list.len() == 1;
                chain_link_target(w, from, list);
                if first_only {
                    tell(w, from, "Target the second door to link.");
                } else {
                    tell(
                        w,
                        from,
                        "Target another door to link. To complete the chain, retarget the first door.",
                    );
                }
            }
        },
    );
}

/// `BaseDoor.Configure`: the `Link` and `ChainLink` commands.
pub fn link_commands(w: &mut World) {
    use muo_server::commands::{CommandEventArgs, CommandSystem};
    // Usage: Link; Description: Links two targeted doors together.
    CommandSystem::register(
        w,
        "Link",
        AccessLevel::GameMaster,
        &[],
        std::rc::Rc::new(|w: &mut World, e: &mut CommandEventArgs| {
            link_target(w, e.mobile, None);
            tell(w, e.mobile, "Target the first door to link.");
        }),
    );
    // Usage: ChainLink; Description: Chain-links two or more targeted doors together.
    CommandSystem::register(
        w,
        "ChainLink",
        AccessLevel::GameMaster,
        &[],
        std::rc::Rc::new(|w: &mut World, e: &mut CommandEventArgs| {
            chain_link_target(w, e.mobile, Vec::new());
            tell(
                w,
                e.mobile,
                "Target the first of a sequence of doors to link.",
            );
        }),
    );
}

// ---- `[props` ----

use muo_content::props::PropDef;

macro_rules! door_prop {
    ($name:literal, $kind:expr, $get:expr, $set:expr) => {
        PropDef {
            name: $name,
            kind: $kind,
            read: AccessLevel::GameMaster,
            write: AccessLevel::GameMaster,
            get: |w, e| match e {
                EntityId::Item(i) => $get(w, i),
                _ => Value::Null,
            },
            set: Some(|w, e, v| {
                let EntityId::Item(i) = e else {
                    return Err("not an item".to_string());
                };
                $set(w, i, v);
                Ok(())
            }),
        }
    };
}

const DINT: Kind = Kind::Plain(PropType::Numeric(Numeric::Int));

/// The `[SerializedCommandProperty(AccessLevel.GameMaster)]` fields of `BaseDoor`, in field order.
static BASE_DOOR_PROPS: &[PropDef] = &[
    door_prop!(
        "KeyValue",
        Kind::Plain(PropType::Numeric(Numeric::UInt)),
        |w: &World, i| Value::UInt(u64::from(key_value(w, i))),
        |w: &mut World, i, v: Value| set_key_value(w, i, v.i64() as u32)
    ),
    door_prop!(
        "Open",
        Kind::Plain(PropType::Bool),
        |w: &World, i| Value::Bool(is_open(w, i)),
        |w: &mut World, i, v: Value| set_open(w, i, v.bool())
    ),
    door_prop!(
        "Locked",
        Kind::Plain(PropType::Bool),
        |w: &World, i| Value::Bool(locked(w, i)),
        |w: &mut World, i, v: Value| set_locked(w, i, v.bool())
    ),
    door_prop!(
        "OpenedId",
        DINT,
        |w: &World, i| Value::Int(i64::from(data(w, i).map_or(0, |d| d.opened_id))),
        |w: &mut World, i, v: Value| {
            if let Some(d) = data_mut(w, i) {
                d.opened_id = v.i32();
            }
        }
    ),
    door_prop!(
        "ClosedId",
        DINT,
        |w: &World, i| Value::Int(i64::from(closed_id_of(w, i))),
        |w: &mut World, i, v: Value| {
            if let Some(d) = data_mut(w, i) {
                d.closed_id = v.i32();
            }
        }
    ),
    door_prop!(
        "OpenedSound",
        DINT,
        |w: &World, i| Value::Int(i64::from(data(w, i).map_or(0, |d| d.opened_sound))),
        |w: &mut World, i, v: Value| {
            if let Some(d) = data_mut(w, i) {
                d.opened_sound = v.i32();
            }
        }
    ),
    door_prop!(
        "ClosedSound",
        DINT,
        |w: &World, i| Value::Int(i64::from(data(w, i).map_or(0, |d| d.closed_sound))),
        |w: &mut World, i, v: Value| {
            if let Some(d) = data_mut(w, i) {
                d.closed_sound = v.i32();
            }
        }
    ),
    door_prop!(
        "Offset",
        Kind::Plain(PropType::IPoint3D),
        |w: &World, i| Value::Point3D(offset(w, i)),
        |w: &mut World, i, v: Value| {
            if let Some(d) = data_mut(w, i) {
                d.offset = v.point();
            }
        }
    ),
    door_prop!(
        "Link",
        Kind::Plain(PropType::Entity),
        |w: &World, i| Value::Entity(data(w, i).and_then(|d| d.link).map(EntityId::Item)),
        |w: &mut World, i, v: Value| {
            let link = match v.entity() {
                Some(EntityId::Item(l)) if is_door(w, l) => Some(l),
                _ => None,
            };
            set_link(w, i, link);
        }
    ),
];

/// `BaseHouseDoor`' `Level` (a `SecureLevel`) and `Facing`.
static BASE_HOUSE_DOOR_PROPS: &[PropDef] = &[
    door_prop!(
        "Level",
        Kind::Plain(PropType::Enum(&[
            ("Owner", 0),
            ("CoOwners", 1),
            ("Friends", 2),
            ("Anyone", 3),
            ("Guild", 4)
        ])),
        |w: &World, i| Value::Enum(i64::from(data(w, i).map_or(0, |d| d.level))),
        |w: &mut World, i, v: Value| {
            if let Some(d) = data_mut(w, i) {
                d.level = v.i32();
            }
        }
    ),
    door_prop!(
        "Facing",
        Kind::Plain(PropType::Enum(FACING_NAMES)),
        |w: &World, i| Value::Enum(i64::from(data(w, i).map_or(0, |d| d.facing))),
        |w: &mut World, i, v: Value| {
            if let Some(d) = data_mut(w, i) {
                d.facing = v.i32();
            }
        }
    ),
];

/// Sets the hooks of the doors (ai movement, houses, keys, the open-door macro) and the `[add` tables.
pub fn configure(w: &mut World) {
    link_commands(w);
    muo_content::props::add(w, "Server.Items.BaseDoor", BASE_DOOR_PROPS);
    muo_content::props::add(w, "Server.Items.BaseHouseDoor", BASE_HOUSE_DOOR_PROPS);
    let o = w
        .resources
        .get_mut::<muo_content::ai::movement::ObstacleHooks>();
    o.is_door = Some(is_door);
    o.door_blocked_by_lock = Some(|w, d| locked(w, d) && use_locks(w, d));
    o.door_use = Some(use_door);
    let h = deps_mut(w);
    h.make_door = Some(make_house_door);
    h.door_set_locked = Some(set_locked);
    h.door_set_key_value = Some(set_key_value);
    h.door_key_value = Some(key_value);
    h.door_set_link = Some(|w, a, b| set_link(w, a, Some(b)));
    h.door_open_offset = Some(|w, d| is_open(w, d).then(|| offset(w, d)));
    h.door_set_facing = Some(|w, d, f| {
        if let Some(x) = data_mut(w, d) {
            x.facing = f as i32;
        }
    });
    h.door_facing = Some(|w, d| is_house_door(w, d).then(|| data(w, d).map_or(0, |x| x.facing)));
    h.door_link = Some(|w, d| data(w, d).and_then(|x| x.link));
    // `BaseHouse.IsThereVendor` for ContractOfEmployment (the house system is a dependency of this crate)
    w.resources
        .get_mut::<crate::misc::dyes::PlayerVendorHooks>()
        .is_there_vendor = Some(muo_multis::house::vendors::is_there_vendor);
    let h = deps_mut(w);
    h.securables
        .push(|w, d| is_house_door(w, d).then(|| data(w, d).map_or(3, |x| x.level) as u8));
    h.securable_setters.push(|w, d, level| {
        let door = is_house_door(w, d);
        if door && let Some(x) = data_mut(w, d) {
            x.level = i32::from(level);
        }
        door
    });
    muo_content::items::common::add_lockable(
        w,
        muo_content::items::common::Lockable {
            key_value: |w, d| is_door(w, d).then(|| key_value(w, d)),
            toggle: |w, d| {
                if !is_door(w, d) || !use_locks(w, d) {
                    return None;
                }
                let l = !locked(w, d);
                set_locked(w, d, l);
                Some(l)
            },
        },
    );
    w.resources
        .get_mut::<muo_content::network::incoming_game::GameHooks>()
        .open_door_macro = Some(open_door_macro_used);
    for (name, t) in TABLES {
        add_ctors(w, name, t);
    }
}

static TABLES: [(&str, &[Ctor]); 24] = [
    ("Server.Items.IronGateShort", &C_IRON_GATE_SHORT),
    ("Server.Items.IronGate", &C_IRON_GATE),
    ("Server.Items.LightWoodGate", &C_LIGHT_WOOD_GATE),
    ("Server.Items.DarkWoodGate", &C_DARK_WOOD_GATE),
    ("Server.Items.MetalDoor", &C_METAL_DOOR),
    ("Server.Items.BarredMetalDoor", &C_BARRED_METAL_DOOR),
    ("Server.Items.BarredMetalDoor2", &C_BARRED_METAL_DOOR2),
    ("Server.Items.RattanDoor", &C_RATTAN_DOOR),
    ("Server.Items.DarkWoodDoor", &C_DARK_WOOD_DOOR),
    ("Server.Items.MediumWoodDoor", &C_MEDIUM_WOOD_DOOR),
    ("Server.Items.MetalDoor2", &C_METAL_DOOR2),
    ("Server.Items.LightWoodDoor", &C_LIGHT_WOOD_DOOR),
    ("Server.Items.StrongWoodDoor", &C_STRONG_WOOD_DOOR),
    ("Server.Items.SecretStoneDoor1", &C_SECRET_STONE_DOOR1),
    ("Server.Items.SecretDungeonDoor", &C_SECRET_DUNGEON_DOOR),
    ("Server.Items.SecretStoneDoor2", &C_SECRET_STONE_DOOR2),
    ("Server.Items.SecretWoodenDoor", &C_SECRET_WOODEN_DOOR),
    (
        "Server.Items.SecretLightWoodDoor",
        &C_SECRET_LIGHT_WOOD_DOOR,
    ),
    ("Server.Items.SecretStoneDoor3", &C_SECRET_STONE_DOOR3),
    ("Server.Items.MetalHouseDoor", &C_METAL_HOUSE_DOOR),
    ("Server.Items.DarkWoodHouseDoor", &C_DARK_WOOD_HOUSE_DOOR),
    ("Server.Items.GenericHouseDoor", &C_GENERIC_HOUSE_DOOR),
    ("Server.Items.PortcullisNS", &C_PORTCULLIS_NS),
    ("Server.Items.PortcullisEW", &C_PORTCULLIS_EW),
];

/// Registers the door classes.
pub fn register(types: &mut TypeRegistry) {
    types.register_item(&BASE_DOOR, &[]);
    types.register_item(&BASE_HOUSE_DOOR, &[]);
    for t in [
        &IRON_GATE_SHORT,
        &IRON_GATE,
        &LIGHT_WOOD_GATE,
        &DARK_WOOD_GATE,
        &METAL_DOOR,
        &BARRED_METAL_DOOR,
        &BARRED_METAL_DOOR2,
        &RATTAN_DOOR,
        &DARK_WOOD_DOOR,
        &MEDIUM_WOOD_DOOR,
        &METAL_DOOR2,
        &LIGHT_WOOD_DOOR,
        &STRONG_WOOD_DOOR,
        &SECRET_STONE_DOOR1,
        &SECRET_DUNGEON_DOOR,
        &SECRET_STONE_DOOR2,
        &SECRET_WOODEN_DOOR,
        &SECRET_LIGHT_WOOD_DOOR,
        &SECRET_STONE_DOOR3,
        &METAL_HOUSE_DOOR,
        &DARK_WOOD_HOUSE_DOOR,
        &GENERIC_HOUSE_DOOR,
        &PORTCULLIS_NS,
        &PORTCULLIS_EW,
    ] {
        types.register_item(t, &[]);
    }
    types.register_phase_default(
        Phase::Configure,
        "Server.Items.BaseDoor.Configure",
        configure,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use muo_io::{SaveReader, SaveWriter};

    fn save(w: &World, id: ItemId) -> Vec<u8> {
        let mut out = SaveWriter::new(true);
        (w.item(id).unwrap().ty.serialize)(w, id, &mut out);
        out.into_vec()
    }

    fn reload(w: &mut World, ty: &'static ItemType, bytes: &[u8]) -> ItemId {
        let id = w.add_item(muo_server::item::Item::new_for_load(
            muo_primitives::Serial::new(0x4000_0100),
            ty,
        ));
        let mut r = SaveReader::new(bytes);
        (ty.deserialize)(w, id, &mut r).unwrap();
        assert_eq!(r.remaining(), 0);
        id
    }

    #[test]
    fn iron_gate_save_is_the_generated_layout() {
        let mut w = crate::test_world();
        let id = facing_door(&mut w, &IRON_GATE, DoorFacing::SouthCW, 0x824, 0xEC, 0xF3);
        assert_eq!(w.item(id).unwrap().item_id, 0x824 + 8);
        let bytes = save(&w, id);
        let mut want = SaveWriter::new(true);
        (ITEM_BASE.serialize)(&w, id, &mut want);
        want.write_i32(0); // BaseDoor version
        want.write_u32(0); // KeyValue
        want.write_bool(false); // Open
        want.write_bool(false); // Locked
        want.write_i32(0x824 + 9);
        want.write_i32(0x824 + 8);
        want.write_i32(0xEC);
        want.write_i32(0xF3);
        want.write_point3d(Point3D::new(0, 0, 0)); // GetOffset(SouthCW) is (1, 1, 0)? no: index 4
        want.write_item(&w, None);
        want.write_i32(0); // IronGate version
        let mut want = want.into_vec();
        // the offset of SouthCW is m_Offsets[4] = (1, 1, 0)
        let off = want.len() - 4 - 4 - 12;
        want[off..off + 12].copy_from_slice(&[1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(bytes, want);
        let id2 = reload(&mut w, &IRON_GATE, &bytes);
        assert_eq!(save(&w, id2), bytes);
    }

    #[test]
    fn open_moves_the_door_and_close_moves_it_back() {
        let mut w = crate::test_world();
        let id = facing_door(
            &mut w,
            &DARK_WOOD_DOOR,
            DoorFacing::WestCW,
            0x6A5,
            0xEA,
            0xF1,
        );
        ops::set_location(&mut w, id, Point3D::new(100, 100, 5));
        set_open(&mut w, id, true);
        let i = w.item(id).unwrap();
        assert_eq!((i.item_id, i.location), (0x6A6, Point3D::new(99, 101, 5)));
        set_open(&mut w, id, false);
        let i = w.item(id).unwrap();
        assert_eq!((i.item_id, i.location), (0x6A5, Point3D::new(100, 100, 5)));
    }

    #[test]
    fn house_door_saves_level_and_facing_after_the_door() {
        let mut w = crate::test_world();
        let id = new_house_door(
            &mut w,
            &DARK_WOOD_HOUSE_DOOR,
            DoorFacing::EastCW,
            0x6A5,
            0xEA,
            0xF1,
            true,
        );
        let bytes = save(&w, id);
        let tail = &bytes[bytes.len() - 16..];
        assert_eq!(tail, [2, 0, 0, 0, 3, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0]); // version 2, Level Anyone, Facing EastCW, leaf version
        let id2 = reload(&mut w, &DARK_WOOD_HOUSE_DOOR, &bytes);
        assert_eq!(save(&w, id2), bytes);
    }

    #[test]
    fn get_chain_loop_not_through_start_yields_each_door_once() {
        let mut w = crate::test_world();
        let mut door = || facing_door(&mut w, &METAL_DOOR, DoorFacing::WestCW, 0x675, 0xEC, 0xF3);
        let (a, b, c) = (door(), door(), door());
        // A -> B -> C -> B: the chain never returns to A
        set_link(&mut w, a, Some(b));
        set_link(&mut w, b, Some(c));
        set_link(&mut w, c, Some(b));

        assert_eq!(chain(&w, a), vec![a, b, c]);
    }
}
