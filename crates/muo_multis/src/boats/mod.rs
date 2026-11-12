//! `BaseBoat`: the boat multi, its commands, movement, turning, docking and decay.
//!
//! State is [`BoatData`] in `Item::ext`; what a concrete boat overrides (`NorthID`.., `HoldDistance`,
//! `TillerManDistance`, the plank and mark offsets, `DockedBoat`) is its [`BoatClass`] (`ItemType::family`). The
//! planks, tiller man and hold are in `parts`, the deeds and docked boats in `items`, the 0xF6/0xF3 packets in
//! `packets`.
//!
//! The `[CommandProperty]` table of `BaseBoat` is in `props_gen.rs`.

pub mod items;
pub mod packets;
pub mod parts;
pub mod strandedness;

#[cfg(test)]
mod tests;

use muo_content::core;
use muo_content::factory::item_is;
use muo_content::items::gold::{self, KeyType};
use muo_data::client::multi_data::MultiComponentList;
use muo_io::{GenericReader, GenericWriter, Result};
use muo_primitives::{DateTime, Direction, Point2D, Point3D, Rectangle2D, TimeSpan, utility};
use muo_server::World;
use muo_server::events::SpeechEventArgs;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::multi::{self, BASE_MULTI};
use muo_server::item::{ItemType, ops};
use muo_server::map::{MapId, enumerate, tiles};
use muo_server::mobile::AccessLevel;
use muo_server::movement;
use muo_server::packets::message::MessageType;
use muo_server::serialization_ext::{ReadEntityExt, WriteEntityExt};
use muo_server::timer::{self, TimerHandle};
use muo_server::type_registry::TypeRegistry;

use parts::{plank_set_facing, tiller_say};

/// `BoatOrder`.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum BoatOrder {
    Move,
    Course,
    Single,
}

/// `DryDockResult`.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum DryDockResult {
    Valid,
    Dead,
    NoKey,
    NotAnchored,
    Mobiles,
    Items,
    Hold,
    Decaying,
}

// The directions, as the numbers `Direction` has.
const FORWARD: i32 = 0; // North
const FORWARD_LEFT: i32 = 7; // Up
const FORWARD_RIGHT: i32 = 1; // Right
const BACKWARD: i32 = 4; // South
const BACKWARD_LEFT: i32 = 5; // Left
const BACKWARD_RIGHT: i32 = 3; // Down
const LEFT: i32 = 6; // West
const RIGHT: i32 = 2; // East

const SLOW_SPEED: i32 = 1;
const SLOW_DRIFT_SPEED: i32 = 1;
const FAST_DRIFT_SPEED: i32 = 1;

fn boat_decay_delay() -> TimeSpan {
    TimeSpan::from_days(9)
}

/// `NewBoatMovement`.
fn new_boat_movement(w: &World) -> bool {
    core::hs(w)
}

fn secs(s: f64) -> TimeSpan {
    TimeSpan::from_secs_f64(s)
}

fn slow_interval(w: &World) -> TimeSpan {
    secs(if new_boat_movement(w) { 0.50 } else { 0.75 })
}
fn fast_interval(w: &World) -> TimeSpan {
    secs(if new_boat_movement(w) { 0.25 } else { 0.75 })
}
fn fast_speed(w: &World) -> i32 {
    if new_boat_movement(w) { 1 } else { 3 }
}
fn slow_drift_interval(w: &World) -> TimeSpan {
    secs(if new_boat_movement(w) { 0.50 } else { 1.50 })
}
fn fast_drift_interval(w: &World) -> TimeSpan {
    secs(if new_boat_movement(w) { 0.25 } else { 0.75 })
}

/// `BritWrap`, `IlshWrap`, `TokunoWrap`.
fn wrap_for(map: Option<MapId>) -> Vec<Rectangle2D> {
    if map == Some(MapId::ILSHENAR) {
        vec![Rectangle2D::new(16, 16, 2304 - 32, 1600 - 32)]
    } else if map == Some(MapId::TOKUNO) {
        vec![Rectangle2D::new(16, 16, 1448 - 32, 1448 - 32)]
    } else {
        vec![
            Rectangle2D::new(16, 16, 5120 - 32, 4096 - 32),
            Rectangle2D::new(5136, 2320, 992, 1760),
        ]
    }
}

/// What a concrete boat overrides.
pub struct BoatClass {
    pub north_id: i32,
    pub east_id: i32,
    pub south_id: i32,
    pub west_id: i32,
    pub hold_distance: i32,
    pub tiller_man_distance: i32,
    pub starboard_offset: (i32, i32),
    pub port_offset: (i32, i32),
    pub mark_offset: (i32, i32, i32),
    /// `DockedBoat => new XDockedBoat(this)`.
    pub docked_boat: fn(&mut World, ItemId) -> Option<ItemId>,
}

pub static BASE_BOAT_CLASS: BoatClass = BoatClass {
    north_id: 0,
    east_id: 0,
    south_id: 0,
    west_id: 0,
    hold_distance: 0,
    tiller_man_distance: 0,
    starboard_offset: (0, 0),
    port_offset: (0, 0),
    mark_offset: (0, 0, 0),
    docked_boat: |_, _| None,
};

/// A `MoveTimer`: the handle and what `Interval`/`Count` read.
#[derive(Copy, Clone, Debug)]
pub struct MoveTimerState {
    pub handle: TimerHandle,
    pub interval: TimeSpan,
    pub count: i32,
}

/// A `TurnTimer`: `_turn` and the `_message` it was made with.
#[derive(Copy, Clone, Debug)]
pub struct TurnTimerState {
    pub handle: TimerHandle,
    pub turn: i32,
    pub message: bool,
}

/// What `BaseBoat` adds to an `Item`.
#[derive(Clone, Debug)]
pub struct BoatData {
    pub map_item: Option<ItemId>,
    pub next_nav_point: i32,
    /// `_facing` as a `Direction` number (0..7).
    pub facing: i32,
    pub time_of_decay: DateTime,
    pub owner: Option<MobileId>,
    pub p_plank: Option<ItemId>,
    pub s_plank: Option<ItemId>,
    pub tiller_man: Option<ItemId>,
    pub hold: Option<ItemId>,
    pub anchored: bool,
    pub ship_name: Option<String>,
    pub moving: i32,
    pub speed: i32,
    pub order: BoatOrder,
    pub client_speed: i32,
    pub decaying: bool,
    pub turn_timer: Option<TurnTimerState>,
    pub move_timer: Option<MoveTimerState>,
}

impl Default for BoatData {
    fn default() -> Self {
        BoatData {
            map_item: None,
            next_nav_point: 0,
            facing: 0,
            time_of_decay: DateTime::MIN,
            owner: None,
            p_plank: None,
            s_plank: None,
            tiller_man: None,
            hold: None,
            anchored: false,
            ship_name: None,
            moving: 0,
            speed: 0,
            order: BoatOrder::Move,
            client_speed: 0,
            decaying: false,
            turn_timer: None,
            move_timer: None,
        }
    }
}

/// `BaseBoat.Boats`.
#[derive(Default)]
pub struct BoatStatics {
    pub boats: Vec<ItemId>,
}

type MapPinsFn = fn(&World, ItemId) -> Option<Vec<Point2D>>;
type MapConvertFn = fn(&World, ItemId, i32, i32) -> (i32, i32);

/// What the boats ask of `MapItem` (Items/Maps, a crate this one is a dependency of): its pins, the conversion of
/// a pin to world coordinates, and whether it is a `BlankMap`. Unset: no item is a map.
#[derive(Default)]
pub struct BoatHooks {
    /// `item is MapItem`: its `Pins`.
    pub map_pins: Option<MapPinsFn>,
    /// `map.ConvertToWorld(x, y, out wx, out wy)`.
    pub map_convert_to_world: Option<MapConvertFn>,
    /// `item is BlankMap`.
    pub is_blank_map: Option<fn(&World, ItemId) -> bool>,
}

pub fn hooks(w: &World) -> Option<&BoatHooks> {
    w.resources.get::<BoatHooks>()
}

pub fn statics(w: &mut World) -> &mut BoatStatics {
    w.resources.get_mut::<BoatStatics>()
}

pub fn data(w: &World, id: ItemId) -> Option<&BoatData> {
    w.item(id)?.ext.get::<BoatData>()
}

pub fn data_mut(w: &mut World, id: ItemId) -> Option<&mut BoatData> {
    w.item_mut(id)?.ext.get_mut::<BoatData>()
}

/// `item is BaseBoat`.
pub fn is_boat(w: &World, id: ItemId) -> bool {
    w.item(id).is_some_and(|i| i.ty.is(&BASE_BOAT))
}

pub fn class(w: &World, id: ItemId) -> &'static BoatClass {
    w.item(id)
        .and_then(|i| i.ty.family::<BoatClass>())
        .unwrap_or(&BASE_BOAT_CLASS)
}

fn location(w: &World, id: ItemId) -> Point3D {
    w.item(id).map_or(Point3D::ZERO, |i| i.location)
}

fn map_of(w: &World, id: ItemId) -> Option<MapId> {
    w.item(id).and_then(|i| i.map)
}

fn deleted(w: &World, id: ItemId) -> bool {
    w.item(id).is_none_or(|i| i.deleted())
}

fn delete_item(w: &mut World, id: ItemId) {
    if let Some(ty) = w.item(id).map(|i| i.ty) {
        (ty.delete)(w, id);
    }
}

fn facing(w: &World, b: ItemId) -> i32 {
    data(w, b).map_or(0, |d| d.facing)
}

/// `Facing`.
pub fn get_facing(w: &World, b: ItemId) -> Direction {
    Direction::from_bits_retain(facing(w, b) as u8)
}

pub fn owner(w: &World, b: ItemId) -> Option<MobileId> {
    data(w, b).and_then(|d| d.owner)
}

pub fn ship_name(w: &World, b: ItemId) -> Option<String> {
    data(w, b).and_then(|d| d.ship_name.clone())
}

pub fn tiller_man(w: &World, b: ItemId) -> Option<ItemId> {
    data(w, b).and_then(|d| d.tiller_man)
}

pub fn hold(w: &World, b: ItemId) -> Option<ItemId> {
    data(w, b).and_then(|d| d.hold)
}

pub fn p_plank(w: &World, b: ItemId) -> Option<ItemId> {
    data(w, b).and_then(|d| d.p_plank)
}

pub fn s_plank(w: &World, b: ItemId) -> Option<ItemId> {
    data(w, b).and_then(|d| d.s_plank)
}

pub fn anchored(w: &World, b: ItemId) -> bool {
    data(w, b).is_some_and(|d| d.anchored)
}

/// `Anchored = value` (marks the boat dirty).
pub fn set_anchored(w: &mut World, b: ItemId, v: bool) {
    if let Some(d) = data_mut(w, b) {
        d.anchored = v;
    }
}

/// `Owner = value`.
pub fn set_owner(w: &mut World, b: ItemId, v: Option<MobileId>) {
    if let Some(d) = data_mut(w, b) {
        d.owner = v;
    }
}

/// `TimeOfDecay = value` (the tiller man's properties change).
pub fn set_time_of_decay(w: &mut World, b: ItemId, v: DateTime) {
    if let Some(d) = data_mut(w, b) {
        if d.time_of_decay == v {
            return;
        }
        d.time_of_decay = v;
    }
    if let Some(t) = tiller_man(w, b) {
        ops::invalidate_properties(w, t);
    }
}

/// `ShipName = value`.
pub fn set_ship_name(w: &mut World, b: ItemId, v: Option<String>) {
    if let Some(d) = data_mut(w, b) {
        if d.ship_name == v {
            return;
        }
        d.ship_name = v;
    }
    if let Some(t) = tiller_man(w, b) {
        ops::invalidate_properties(w, t);
    }
}

/// `IsMoving => _moveTimer?.Running == true`.
pub fn is_moving(w: &World, b: ItemId) -> bool {
    data(w, b)
        .and_then(|d| d.move_timer)
        .is_some_and(|t| t.handle.running(w))
}

/// `Status`: the cliloc of the condition.
pub fn status(w: &World, b: ItemId) -> i32 {
    let tod = data(w, b).map_or(DateTime::MIN, |d| d.time_of_decay);
    let start = w.now - (tod - boat_decay_delay());
    if start < TimeSpan::from_hours(1) {
        1043010 // This structure is like new.
    } else if start < TimeSpan::from_days(2) {
        1043011 // This structure is slightly worn.
    } else if start < TimeSpan::from_days(3) {
        1043012 // This structure is somewhat worn.
    } else if start < TimeSpan::from_days(4) {
        1043013 // This structure is fairly worn.
    } else if start < TimeSpan::from_days(5) {
        1043014 // This structure is greatly worn.
    } else {
        1043015 // This structure is in danger of collapsing.
    }
}

/// `FindBoatAt(loc, map)`.
pub fn find_boat_at(w: &World, loc: Point3D, map: Option<MapId>) -> Option<ItemId> {
    let map = map.filter(|&m| m != MapId::INTERNAL)?;
    enumerate::multis_in_sector(w, Some(map), loc.x, loc.y)
        .find(|&b| is_boat(w, b) && contains(w, b, loc.x, loc.y))
}

/// `Contains(x, y)` of a boat: the multi, the tiller man, the hold and the planks.
fn contains_xy(w: &World, b: ItemId, x: i32, y: i32) -> bool {
    let at = |i: Option<ItemId>| {
        i.is_some_and(|i| {
            w.item(i)
                .is_some_and(|it| it.location.x == x && it.location.y == y)
        })
    };
    multi::contains_default(w, b, x, y)
        || at(tiller_man(w, b))
        || at(hold(w, b))
        || at(p_plank(w, b))
        || at(s_plank(w, b))
}

/// `Contains(x, y)` through the vtable.
pub fn contains(w: &World, b: ItemId, x: i32, y: i32) -> bool {
    multi::contains(w, b, x, y)
}

/// `Contains(Mobile)`.
pub fn contains_mobile(w: &World, b: ItemId, m: MobileId) -> bool {
    multi::contains_mobile(w, b, m)
}

/// `Contains(Item)`.
pub fn contains_item(w: &World, b: ItemId, i: ItemId) -> bool {
    multi::contains_item(w, b, i)
}

/// `Rotate(p, count)`.
pub fn rotate(w: &World, b: ItemId, p: Point3D, count: i32) -> Point3D {
    let l = location(w, b);
    let (mut rx, mut ry) = (p.x - l.x, p.y - l.y);
    for _ in 0..count {
        let t = rx;
        rx = -ry;
        ry = t;
    }
    Point3D::new(l.x + rx, l.y + ry, p.z)
}

/// `GetRotatedLocation(x, y)`.
pub fn get_rotated_location(w: &World, b: ItemId, x: i32, y: i32) -> Point3D {
    let l = location(w, b);
    rotate(w, b, Point3D::new(l.x + x, l.y + y, l.z), facing(w, b) / 2)
}

/// `GetMarkedLocation()`.
pub fn get_marked_location(w: &World, b: ItemId) -> Point3D {
    let l = location(w, b);
    let m = class(w, b).mark_offset;
    rotate(
        w,
        b,
        Point3D::new(l.x + m.0, l.y + m.1, l.z + m.2),
        facing(w, b) / 2,
    )
}

/// `CheckKey(keyValue)`.
pub fn check_key(w: &World, b: ItemId, key_value: u32) -> bool {
    s_plank(w, b).is_some_and(|p| parts::plank_key_value(w, p) == key_value)
        || p_plank(w, b).is_some_and(|p| parts::plank_key_value(w, p) == key_value)
}

/// `Refresh()`.
pub fn refresh(w: &mut World, b: ItemId) {
    let t = w.now + boat_decay_delay();
    set_time_of_decay_silent(w, b, t);
    if let Some(t) = tiller_man(w, b) {
        ops::invalidate_properties(w, t);
    }
}

fn set_time_of_decay_silent(w: &mut World, b: ItemId, v: DateTime) {
    if let Some(d) = data_mut(w, b) {
        d.time_of_decay = v;
    }
}

// ---- construction ----

/// `new BaseBoat()` of class `ty`.
pub fn new_boat(w: &mut World, ty: &'static ItemType) -> ItemId {
    let b = multi::new_base_multi(w, ty, 0x0);
    let decay = w.now + boat_decay_delay();
    if let Some(i) = w.item_mut(b) {
        i.ext.insert(BoatData {
            time_of_decay: decay,
            next_nav_point: 0,
            ..Default::default()
        });
    }
    // TillerMan = new TillerMan(this); Hold = new Hold(this);
    let t = parts::new_tiller_man(w, b);
    let h = parts::new_hold(w, b);
    let pp = parts::new_plank(w, b, parts::PlankSide::Port, 0);
    let sp = parts::new_plank(w, b, parts::PlankSide::Starboard, 0);
    if let Some(d) = data_mut(w, b) {
        d.tiller_man = Some(t);
        d.hold = Some(h);
        d.p_plank = Some(pp);
        d.s_plank = Some(sp);
    }
    let (l, map) = (location(w, b), map_of(w, b));
    let c = class(w, b);
    ops::move_to_world_map(
        w,
        pp,
        Point3D::new(l.x + c.port_offset.0, l.y + c.port_offset.1, l.z),
        map,
    );
    ops::move_to_world_map(
        w,
        sp,
        Point3D::new(l.x + c.starboard_offset.0, l.y + c.starboard_offset.1, l.z),
        map,
    );

    set_facing(w, b, 0); // Facing = Direction.North
    if let Some(d) = data_mut(w, b) {
        d.next_nav_point = -1;
    }
    ops::set_movable(w, b, false);
    statics(w).boats.push(b);
    b
}

/// `CreateKeys(m)`.
pub fn create_keys(w: &mut World, b: ItemId, m: MobileId) -> u32 {
    let value = gold::random_value(w);
    let pack_key = gold::new_key_of(w, KeyType::Gold, value, Some(b));
    let bank_key = gold::new_key_of(w, KeyType::Gold, value, Some(b));
    for k in [pack_key, bank_key] {
        if let Some(d) = w.item_mut(k).and_then(|i| i.ext.get_mut::<gold::KeyData>()) {
            d.max_range = 10;
        }
        ops::set_name(w, k, Some("a ship key".to_string()));
    }
    let bank = muo_server::mobile::part1::props_b::bank_box(w, m);
    let dropped = bank.is_some_and(|bb| {
        muo_server::item::container::try_drop_item(w, bb, Some(m), bank_key, false)
    });
    if !dropped {
        delete_item(w, bank_key);
    } else {
        local_overhead(w, m, 502484); // A ship's key is now in my safety deposit box.
    }
    if muo_server::mobile::part2::items::add_to_backpack(w, m, pack_key) {
        local_overhead(w, m, 502485); // A ship's key is now in my backpack.
    } else {
        local_overhead(w, m, 502483); // A ship's key is now at my feet.
    }
    value
}

fn local_overhead(w: &mut World, m: MobileId, number: i32) {
    muo_server::item::deps::local_overhead_message(w, m, MessageType::REGULAR, 0x3B2, number);
}

/// `RemoveKeys(m)`.
pub fn remove_keys(w: &mut World, b: ItemId, m: MobileId) {
    let mut key_value = p_plank(w, b).map_or(0, |p| parts::plank_key_value(w, p));
    if key_value == 0 {
        key_value = s_plank(w, b).map_or(0, |p| parts::plank_key_value(w, p));
    }
    gold::remove_keys_mobile(w, m, key_value);
}

// ---- components ----

/// `UpdateComponents()`.
pub fn update_components(w: &mut World, b: ItemId) {
    let c = class(w, b);
    let f = facing(w, b);
    let map = map_of(w, b);
    if let Some(p) = p_plank(w, b) {
        let at = get_rotated_location(w, b, c.port_offset.0, c.port_offset.1);
        ops::move_to_world_map(w, p, at, map);
        plank_set_facing(w, p, f);
    }
    if let Some(p) = s_plank(w, b) {
        let at = get_rotated_location(w, b, c.starboard_offset.0, c.starboard_offset.1);
        ops::move_to_world_map(w, p, at, map);
        plank_set_facing(w, p, f);
    }
    let (mut xo, mut yo) = (0, 0);
    movement::offset_xy(Direction::from_bits_retain(f as u8), &mut xo, &mut yo, 1);
    let l = location(w, b);
    if let Some(t) = tiller_man(w, b) {
        let z = location(w, t).z;
        ops::set_location(
            w,
            t,
            Point3D::new(
                l.x + xo * c.tiller_man_distance + i32::from(f == 0),
                l.y + yo * c.tiller_man_distance,
                z,
            ),
        );
        parts::tiller_set_facing(w, t, f);
        ops::invalidate_properties(w, t);
    }
    if let Some(h) = hold(w, b) {
        let z = location(w, h).z;
        ops::set_location(
            w,
            h,
            Point3D::new(l.x + xo * c.hold_distance, l.y + yo * c.hold_distance, z),
        );
        parts::hold_set_facing(w, h, f);
    }
}

/// `UpdateAllComponents()`.
pub fn update_all_components(w: &mut World) {
    let boats = statics(w).boats.clone();
    for b in boats.into_iter().rev() {
        update_components(w, b);
    }
}

// ---- decay ----

fn should_decay(w: &World, b: ItemId) -> bool {
    !is_moving(w, b) && data(w, b).is_some_and(|d| w.now >= d.time_of_decay)
}

/// `CheckDecay()`.
pub fn check_decay(w: &mut World, b: ItemId) -> bool {
    if data(w, b).is_some_and(|d| d.decaying) {
        return true;
    }
    if should_decay(w, b) {
        start_decay_timer(w, b);
        if let Some(d) = data_mut(w, b) {
            d.decaying = true;
        }
        return true;
    }
    false
}

/// `new DecayTimer(this).Start()`.
fn start_decay_timer(w: &mut World, b: ItemId) {
    let mut count = 0;
    timer::start(
        w,
        TimeSpan::from_secs(1),
        TimeSpan::from_secs(5),
        0,
        move |w| {
            if count == 5 {
                delete_item(w, b);
                if let Some(h) = timer::current(w) {
                    timer::stop(w, h);
                }
            } else {
                let l = location(w, b);
                ops::set_location(w, b, Point3D::new(l.x, l.y, l.z - 1));
                if let Some(t) = tiller_man(w, b) {
                    tiller_say(w, t, 1007168 + count);
                }
                count += 1;
            }
        },
    );
}

fn say(w: &mut World, b: ItemId, number: i32) {
    if let Some(t) = tiller_man(w, b) {
        tiller_say(w, t, number);
    }
}

fn say_args(w: &mut World, b: ItemId, number: i32, args: &str) {
    if let Some(t) = tiller_man(w, b) {
        parts::tiller_say_args(w, t, number, args);
    }
}

// ---- anchor ----

/// `LowerAnchor(message)`.
pub fn lower_anchor(w: &mut World, b: ItemId, message: bool) -> bool {
    if check_decay(w, b) {
        return false;
    }
    if anchored(w, b) {
        if message {
            say(w, b, 501445); // Ar, the anchor was already dropped sir.
        }
        return false;
    }
    stop_move(w, b, false);
    set_anchored(w, b, true);
    if message {
        say(w, b, 501444); // Ar, anchor dropped sir.
    }
    true
}

/// `RaiseAnchor(message)`.
pub fn raise_anchor(w: &mut World, b: ItemId, message: bool) -> bool {
    if check_decay(w, b) {
        return false;
    }
    if !anchored(w, b) {
        if message {
            say(w, b, 501447); // Ar, the anchor has not been dropped sir.
        }
        return false;
    }
    set_anchored(w, b, false);
    if message {
        say(w, b, 501446); // Ar, anchor raised sir.
    }
    true
}

// ---- moving ----

/// `StartMove(dir, fast)`.
pub fn start_move_fast(w: &mut World, b: ItemId, dir: i32, fast: bool) -> bool {
    if check_decay(w, b) {
        return false;
    }
    let drift = dir != FORWARD && dir != FORWARD_LEFT && dir != FORWARD_RIGHT;
    let interval = if fast {
        if drift {
            fast_drift_interval(w)
        } else {
            fast_interval(w)
        }
    } else if drift {
        slow_drift_interval(w)
    } else {
        slow_interval(w)
    };
    let speed = if fast {
        if drift {
            FAST_DRIFT_SPEED
        } else {
            fast_speed(w)
        }
    } else if drift {
        SLOW_DRIFT_SPEED
    } else {
        SLOW_SPEED
    };
    let client_speed = if fast { 0x4 } else { 0x3 };
    if start_move(w, b, dir, speed, client_speed, interval, false, true) {
        say(w, b, 501429); // Aye aye sir.
        return true;
    }
    false
}

/// `OneMove(dir)`.
pub fn one_move(w: &mut World, b: ItemId, dir: i32) -> bool {
    if check_decay(w, b) {
        return false;
    }
    let drift = dir != FORWARD;
    let interval = if drift {
        fast_drift_interval(w)
    } else {
        fast_interval(w)
    };
    let speed = if drift {
        FAST_DRIFT_SPEED
    } else {
        fast_speed(w)
    };
    if start_move(w, b, dir, speed, 0x1, interval, true, true) {
        say(w, b, 501429); // Aye aye sir.
        return true;
    }
    false
}

/// `StartMove(dir, speed, clientSpeed, interval, single, message)`.
#[allow(clippy::too_many_arguments)]
pub fn start_move(
    w: &mut World,
    b: ItemId,
    dir: i32,
    speed: i32,
    client_speed: i32,
    interval: TimeSpan,
    single: bool,
    message: bool,
) -> bool {
    if check_decay(w, b) {
        return false;
    }
    if anchored(w, b) {
        if message {
            say(w, b, 501419); // Ar, the anchor is down sir!
        }
        return false;
    }
    if let Some(d) = data_mut(w, b) {
        d.moving = dir;
        d.speed = speed;
        d.client_speed = client_speed;
        d.order = BoatOrder::Move;
    }
    safely_start_move_timer(w, b, interval, single);
    true
}

/// `SafelyStartMoveTimer(interval, single)`.
fn safely_start_move_timer(w: &mut World, b: ItemId, interval: TimeSpan, single: bool) -> bool {
    let single_num = i32::from(single);
    let existing = data(w, b).and_then(|d| d.move_timer);
    let new_timer = |w: &mut World, delay: TimeSpan| {
        let h = timer::create(w, delay, interval, single_num, move |w| stop_boat(w, b));
        MoveTimerState {
            handle: h,
            interval,
            count: single_num,
        }
    };
    let state = if let Some(t) = existing {
        // Do not allow them to travel faster simply by respamming the command
        if t.handle.running(w) && t.interval == interval && t.count == single_num {
            return false;
        }
        // Have they issued a new command too soon? Ignore it if so
        let next_ms = (t.handle.next(w) - w.now).total_milliseconds();
        if next_ms < 0.0 && next_ms > -500.0 {
            return false;
        }
        // Changing the interval or flipping between commands: "Forward" and "Forward One"
        let mut delay = TimeSpan::ZERO;
        if interval.total_milliseconds() > next_ms.abs() {
            let added = (interval.total_milliseconds() - next_ms.abs()) as i64;
            delay = TimeSpan::from_millis(added);
        }
        timer::stop(w, t.handle);
        // We can reuse the timer if it's not a single count
        if t.count == 0 && single_num == 0 {
            timer::set_delay(w, t.handle, delay);
            timer::set_interval(w, t.handle, interval);
            MoveTimerState {
                handle: t.handle,
                interval,
                count: 0,
            }
        } else {
            timer::release(w, t.handle);
            new_timer(w, delay)
        }
    } else {
        new_timer(w, interval)
    };
    if let Some(d) = data_mut(w, b) {
        d.move_timer = Some(state);
    }
    timer::begin(w, state.handle);
    true
}

/// `StopBoat()`: the move timer's tick.
fn stop_boat(w: &mut World, b: ItemId) {
    if !do_movement(w, b, true) {
        stop_move(w, b, false);
    }
}

/// `StopMove(message)`.
pub fn stop_move(w: &mut World, b: ItemId, message: bool) -> bool {
    if check_decay(w, b) {
        return false;
    }
    if let Some(t) = data(w, b).and_then(|d| d.move_timer)
        && !t.handle.running(w)
    {
        if message {
            say(w, b, 501443); // Er, the ship is not moving sir.
        }
        return false;
    }
    let timer = data(w, b).and_then(|d| d.move_timer);
    if let Some(d) = data_mut(w, b) {
        d.moving = 0;
        d.speed = 0;
        d.client_speed = 0;
    }
    if let Some(t) = timer {
        timer::stop(w, t.handle);
    }
    if message {
        say(w, b, 501429); // Aye aye sir.
    }
    true
}

/// `StartTurn(offset, message)`.
pub fn start_turn(w: &mut World, b: ItemId, offset: i32, message: bool) -> bool {
    if check_decay(w, b) {
        return false;
    }
    if anchored(w, b) {
        if message {
            say(w, b, 501419); // Ar, the anchor is down sir!
        }
        return false;
    }
    let order = data(w, b).map(|d| d.order);
    if order != Some(BoatOrder::Move)
        && let Some(t) = data(w, b).and_then(|d| d.turn_timer)
    {
        timer::stop(w, t.handle);
    }
    if data(w, b).is_some_and(|d| d.turn_timer.is_none()) {
        let h = timer::create(w, TimeSpan::from_millis(500), TimeSpan::ZERO, 1, move |w| {
            let Some(t) = data(w, b).and_then(|d| d.turn_timer) else {
                return;
            };
            turn(w, b, t.turn, t.message);
            timer::stop(w, t.handle);
        });
        if let Some(d) = data_mut(w, b) {
            d.turn_timer = Some(TurnTimerState {
                handle: h,
                turn: offset,
                message,
            });
        }
    }
    let Some(t) = data(w, b).and_then(|d| d.turn_timer) else {
        return false;
    };
    if t.handle.running(w) && t.turn == offset {
        // Dont let them spin around in circles like crazy
        return true;
    }
    // Have they issued a new command too soon? Ignore it if so
    let next_ms = (t.handle.next(w) - w.now).total_milliseconds();
    if next_ms < 0.0 && next_ms > -250.0 {
        return true;
    }
    if let Some(d) = data_mut(w, b)
        && let Some(tt) = d.turn_timer.as_mut()
    {
        tt.turn = offset;
    }
    timer::stop(w, t.handle);
    timer::begin(w, t.handle);
    if message {
        say(w, b, 501429); // Aye aye sir.
    }
    true
}

/// `Turn(offset, message)`.
pub fn turn(w: &mut World, b: ItemId, offset: i32, message: bool) -> bool {
    if let Some(t) = data(w, b).and_then(|d| d.turn_timer) {
        timer::stop(w, t.handle);
    }
    if check_decay(w, b) {
        return false;
    }
    if anchored(w, b) {
        if message {
            say(w, b, 501419); // Ar, the anchor is down sir!
        }
        return false;
    }
    if set_facing(w, b, (facing(w, b) + offset) & 0x7) {
        return true;
    }
    if message {
        say(w, b, 501423); // Ar, can't turn sir.
    }
    false
}

// ---- CanFit ----

/// `CanFit(p, map, itemID)`.
pub fn can_fit(w: &mut World, b: ItemId, p: Point3D, map: Option<MapId>, item_id: i32) -> bool {
    let Some(map) = map.filter(|&m| m != MapId::INTERNAL) else {
        return false;
    };
    if deleted(w, b) || check_decay(w, b) {
        return false;
    }
    let new_components: MultiComponentList = w.maps.multi_data.get_components(item_id).clone();
    let (min, width, height) = (
        new_components.min(),
        new_components.width(),
        new_components.height(),
    );

    for x in 0..width {
        for y in 0..height {
            let tx = p.x + min.x + x;
            let ty = p.y + min.y + y;
            if new_components.tiles[x as usize][y as usize].is_empty() || contains(w, b, tx, ty) {
                continue;
            }
            let land = w.maps.land_tile(map, tx, ty);
            let mut has_water = i32::from(land.z) == p.z
                && ((168..=171).contains(&land.id) || (310..=311).contains(&land.id));
            for tile in tiles::static_and_multi_tiles(w, map, tx, ty) {
                let is_water = (0x1796..=0x17B2).contains(&tile.id);
                if i32::from(tile.z) == p.z && is_water {
                    has_water = true;
                } else if i32::from(tile.z) >= p.z && !is_water {
                    return false;
                }
            }
            if !has_water {
                return false;
            }
        }
    }

    let bounds = Rectangle2D::new(p.x + min.x, p.y + min.y, width, height);
    let items: Vec<ItemId> = enumerate::items_in_bounds(w, Some(map), bounds, false).collect();
    for item in items {
        let Some(it) = w.item(item) else { continue };
        if it.ty.is(&BASE_MULTI)
            || it.item_id > muo_data::TileData::MAX_ITEM_VALUE as i32
            || it.location.z < p.z
            || !it.visible()
        {
            continue;
        }
        let x = it.location.x - p.x + min.x;
        let y = it.location.y - p.y + min.y;
        // Out of bounds, return false - cannot fit
        if (x < 0
            || x >= width
            || y < 0
            || y >= height
            || !new_components.tiles[x as usize][y as usize].is_empty())
            && !contains_item(w, b, item)
        {
            return false;
        }
    }
    true
}

/// `IsValidLocation(p, map)`.
pub fn is_valid_location(p: Point3D, map: Option<MapId>) -> bool {
    wrap_for(map)
        .iter()
        .any(|r| r.contains(Point2D::new(p.x, p.y)))
}

/// `GetMovementFor(x, y, out maxSpeed)`.
fn get_movement_for(w: &World, b: ItemId, x: i32, y: i32) -> (i32, i32) {
    let l = location(w, b);
    let (dx, dy) = (x - l.x, y - l.y);
    let (adx, ady) = (dx.abs(), dy.abs());
    let dir = utility::get_direction(l.x, l.y, x, y).bits() as i32;
    let max_speed = if dir % 2 == 0 {
        (adx - ady).abs()
    } else {
        adx.min(ady)
    };
    ((dir - facing(w, b)) & 0x7, max_speed)
}

/// `DoMovement(message)`.
pub fn do_movement(w: &mut World, b: ItemId, message: bool) -> bool {
    let (dir, speed, client_speed);
    let d = data(w, b).cloned().unwrap_or_default();
    if d.order == BoatOrder::Move {
        dir = d.moving;
        speed = d.speed;
        client_speed = d.client_speed;
    } else {
        let map_item = d.map_item.filter(|&m| !deleted(w, m));
        let Some(mi) = map_item else {
            if message {
                say(w, b, 502513); // I have seen no map, sir.
            }
            return false;
        };
        let mi_loc = ops::get_world_location(w, mi);
        let mi_map = map_of(w, mi);
        if map_of(w, b) != mi_map || !contains(w, b, mi_loc.x, mi_loc.y) {
            if message {
                say(w, b, 502514); // The map is too far away from me, sir.
            }
            return false;
        }
        let pins = hooks(w)
            .and_then(|h| h.map_pins)
            .and_then(|f| f(w, mi))
            .unwrap_or_default();
        let on_brit = map_of(w, b) == Some(MapId::TRAMMEL) || map_of(w, b) == Some(MapId::FELUCCA);
        if !on_brit || d.next_nav_point < 0 || d.next_nav_point as usize >= pins.len() {
            if message {
                say(w, b, 1042551); // I don't see that navpoint, sir.
            }
            return false;
        }
        let dest = pins[d.next_nav_point as usize];
        let (x, y) = hooks(w)
            .and_then(|h| h.map_convert_to_world)
            .map_or((0, 0), |f| f(w, mi, dest.x, dest.y));
        let (dir0, max_speed) = get_movement_for(w, b, x, y);
        if max_speed == 0 {
            if message && d.order == BoatOrder::Single {
                say_args(w, b, 1042874, &(d.next_nav_point + 1).to_string()); // We have arrived at nav point ~1_POINT_NUM~ , sir.
            }
            if (d.next_nav_point + 1) as usize <= pins.len().saturating_sub(1)
                && d.next_nav_point + 1 < pins.len() as i32
            {
                let next = d.next_nav_point + 1;
                if let Some(dm) = data_mut(w, b) {
                    dm.next_nav_point = next;
                }
                if d.order == BoatOrder::Course {
                    if message {
                        say_args(w, b, 1042875, &(next + 1).to_string()); // Heading to nav point ~1_POINT_NUM~, sir.
                    }
                    return true;
                }
                return false;
            }
            if let Some(dm) = data_mut(w, b) {
                dm.next_nav_point = -1;
            }
            if message && d.order == BoatOrder::Course {
                say(w, b, 502515); // The course is completed, sir.
            }
            return false;
        }
        if dir0 == LEFT || dir0 == BACKWARD_LEFT || dir0 == BACKWARD {
            return turn(w, b, -2, true);
        }
        if dir0 == RIGHT || dir0 == BACKWARD_RIGHT {
            return turn(w, b, 2, true);
        }
        dir = dir0;
        speed = d.speed.min(max_speed);
        client_speed = 0x4;
    }
    do_move(w, b, dir, speed, client_speed, true)
}

/// `Move(dir, speed, clientSpeed, message)`.
pub fn do_move(
    w: &mut World,
    b: ItemId,
    dir: i32,
    mut speed: i32,
    client_speed: i32,
    message: bool,
) -> bool {
    let map = map_of(w, b);
    if map.is_none() || deleted(w, b) || check_decay(w, b) {
        return false;
    }
    if anchored(w, b) {
        if message {
            say(w, b, 501419); // Ar, the anchor is down sir!
        }
        return false;
    }
    let (mut rx, mut ry) = (0, 0);
    let d = (facing(w, b) + dir) & 0x7;
    movement::offset_xy(Direction::from_bits_retain(d as u8), &mut rx, &mut ry, 1);
    let l = location(w, b);
    let item_id = w.item(b).map_or(0, |i| i.item_id);

    for i in 1..=speed {
        if !can_fit(
            w,
            b,
            Point3D::new(l.x + i * rx, l.y + i * ry, l.z),
            map,
            item_id,
        ) {
            if i == 1 {
                if message {
                    say(w, b, 501424); // Ar, we've stopped sir.
                }
                return false;
            }
            speed = i - 1;
            break;
        }
    }

    let mut x_offset = speed * rx;
    let mut y_offset = speed * ry;
    let mut new_x = l.x + x_offset;
    let mut new_y = l.y + y_offset;

    for rect in wrap_for(map) {
        if rect.contains(Point2D::new(l.x, l.y)) && !rect.contains(Point2D::new(new_x, new_y)) {
            if new_x < rect.x() {
                new_x = rect.x() + rect.width() - 1;
            } else if new_x >= rect.x() + rect.width() {
                new_x = rect.x();
            }
            if new_y < rect.y() {
                new_y = rect.y() + rect.height() - 1;
            } else if new_y >= rect.y() + rect.height() {
                new_y = rect.y();
            }
            for j in 1..=speed {
                if !can_fit(
                    w,
                    b,
                    Point3D::new(new_x + j * rx, new_y + j * ry, l.z),
                    map,
                    item_id,
                ) {
                    if message {
                        say(w, b, 501424); // Ar, we've stopped sir.
                    }
                    return false;
                }
            }
            x_offset = new_x - l.x;
            y_offset = new_y - l.y;
        }
    }

    if !new_boat_movement(w) || x_offset.abs() > 1 || y_offset.abs() > 1 {
        teleport(w, b, x_offset, y_offset, 0);
        return true;
    }

    let list = get_moving_entities(w, b, true);
    // Packet must be sent before actual locations are changed
    packets::send_move_boat_hs_to_clients(w, b, &list, d, client_speed, x_offset, y_offset);

    let parts_of = [tiller_man(w, b), hold(w, b)];
    for e in &list {
        match *e {
            EntityId::Item(item) => {
                // We want to enable smooth movement for boat parts, but they are ACTUALLY moved when the boat moves
                let is_part = parts_of.contains(&Some(item))
                    || w.item(item).is_some_and(|i| i.ty.is(&parts::PLANK));
                if !is_part {
                    set_no_move(w, EntityId::Item(item), true);
                    let p = location(w, item);
                    ops::set_location(w, item, Point3D::new(p.x + x_offset, p.y + y_offset, p.z));
                    set_no_move(w, EntityId::Item(item), false);
                }
            }
            EntityId::Mobile(m) => {
                set_no_move(w, EntityId::Mobile(m), true);
                let p = w.mobile(m).map_or(Point3D::ZERO, |x| x.location);
                muo_server::mobile::part1::lifecycle::set_location(
                    w,
                    m,
                    Point3D::new(p.x + x_offset, p.y + y_offset, p.z),
                );
                set_no_move(w, EntityId::Mobile(m), false);
            }
        }
    }
    let boat_parts: Vec<ItemId> = [tiller_man(w, b), s_plank(w, b), p_plank(w, b), hold(w, b)]
        .into_iter()
        .flatten()
        .collect();
    for &p in &boat_parts {
        set_no_move(w, EntityId::Item(p), true);
    }
    set_no_move(w, EntityId::Item(b), true);
    let l = location(w, b);
    ops::set_location(w, b, Point3D::new(l.x + x_offset, l.y + y_offset, l.z));
    set_no_move(w, EntityId::Item(b), false);
    for &p in &boat_parts {
        set_no_move(w, EntityId::Item(p), false);
    }
    true
}

fn set_no_move(w: &mut World, e: EntityId, v: bool) {
    match e {
        EntityId::Item(i) => {
            if let Some(x) = w.item_mut(i) {
                x.no_move_hs = v;
            }
        }
        EntityId::Mobile(m) => {
            if let Some(x) = w.mobile_mut(m) {
                x.no_move_hs = v;
            }
        }
    }
}

/// `Teleport(xOffset, yOffset, zOffset)`.
pub fn teleport(w: &mut World, b: ItemId, x_offset: i32, y_offset: i32, z_offset: i32) {
    for e in get_moving_entities(w, b, false) {
        match e {
            EntityId::Item(i) => {
                let p = location(w, i);
                ops::set_location(
                    w,
                    i,
                    Point3D::new(p.x + x_offset, p.y + y_offset, p.z + z_offset),
                );
            }
            EntityId::Mobile(m) => {
                let p = w.mobile(m).map_or(Point3D::ZERO, |x| x.location);
                muo_server::mobile::part1::lifecycle::set_location(
                    w,
                    m,
                    Point3D::new(p.x + x_offset, p.y + y_offset, p.z + z_offset),
                );
            }
        }
    }
    let l = location(w, b);
    ops::set_location(
        w,
        b,
        Point3D::new(l.x + x_offset, l.y + y_offset, l.z + z_offset),
    );
}

/// `GetMovingEntities(includeBoat)`: the mobiles on the boat, then its items (not effects or spawners; not the
/// tiller man, planks and hold unless `include_boat`).
pub fn get_moving_entities(w: &World, b: ItemId, include_boat: bool) -> Vec<EntityId> {
    let map = map_of(w, b);
    let mut out = Vec::new();
    if map.is_none() || map == Some(MapId::INTERNAL) {
        return out;
    }
    let mcl = multi::components(w, b);
    let l = location(w, b);
    let rect = Rectangle2D::new(
        l.x + mcl.min().x,
        l.y + mcl.min().y,
        mcl.width(),
        mcl.height(),
    );
    for m in enumerate::mobiles_in_bounds(w, map, rect, false) {
        if contains_mobile(w, b, m) {
            out.push(EntityId::Mobile(m));
        }
    }
    let boat_parts = [tiller_man(w, b), s_plank(w, b), p_plank(w, b), hold(w, b)];
    let z = l.z;
    for item in enumerate::items_in_bounds(w, map, rect, false) {
        // Skip the boat, effects, spawners, or parts of the boat
        if item == b
            || item_is(w, item, "Server.Items.EffectItem")
            || item_is(w, item, "Server.Engines.Spawners.BaseSpawner")
            || (!include_boat && boat_parts.contains(&Some(item)))
        {
            continue;
        }
        let Some(it) = w.item(item) else { continue };
        // TODO: remove the visible check and use something better
        if it.visible()
            && it.location.z >= z
            && contains(w, b, it.location.x, it.location.y)
            && it.map == map
        {
            out.push(EntityId::Item(item));
        }
    }
    out
}

// ---- facing ----

/// `SetFacing(facing)`.
pub fn set_facing(w: &mut World, b: ItemId, new_facing: i32) -> bool {
    if w.item(b).is_some_and(|i| i.parent.is_some()) {
        return false;
    }
    if check_decay(w, b) {
        return false;
    }
    let map = map_of(w, b);
    let c = class(w, b);
    if map != Some(MapId::INTERNAL) && map.is_some() {
        let id = match new_facing {
            0 => Some(c.north_id),
            2 => Some(c.east_id),
            4 => Some(c.south_id),
            6 => Some(c.west_id),
            _ => None,
        };
        if let Some(id) = id
            && !can_fit(w, b, location(w, b), map, id)
        {
            return false;
        }
    }
    let old = facing(w, b);
    if let Some(d) = data_mut(w, b) {
        d.facing = new_facing;
    }
    if let Some(t) = tiller_man(w, b) {
        parts::tiller_set_facing(w, t, new_facing);
    }
    if let Some(h) = hold(w, b) {
        parts::hold_set_facing(w, h, new_facing);
    }
    if let Some(p) = p_plank(w, b) {
        plank_set_facing(w, p, new_facing);
    }
    if let Some(p) = s_plank(w, b) {
        plank_set_facing(w, p, new_facing);
    }
    let (mut xo, mut yo) = (0, 0);
    movement::offset_xy(
        Direction::from_bits_retain(new_facing as u8),
        &mut xo,
        &mut yo,
        1,
    );
    let count = ((new_facing - old) & 0x7) / 2;

    let entities = get_moving_entities(w, b, false);
    for e in &entities {
        if let EntityId::Mobile(m) = *e {
            let dir = w.mobile(m).map_or(0, |x| i32::from(x.direction.bits()));
            muo_server::mobile::part1::props_b::set_direction(
                w,
                m,
                Direction::from_bits_retain(((dir - old + new_facing) as u8) & 7),
            );
        }
    }
    for e in entities {
        match e {
            EntityId::Item(i) => {
                let p = rotate(w, b, location(w, i), count);
                ops::set_location(w, i, p);
            }
            EntityId::Mobile(m) => {
                let cur = w.mobile(m).map_or(Point3D::ZERO, |x| x.location);
                let p = rotate(w, b, cur, count);
                muo_server::mobile::part1::lifecycle::set_location(w, m, p);
            }
        }
    }

    let l = location(w, b);
    if let Some(t) = tiller_man(w, b) {
        let z = location(w, t).z;
        ops::set_location(
            w,
            t,
            Point3D::new(
                l.x + xo * c.tiller_man_distance + i32::from(new_facing == 0),
                l.y + yo * c.tiller_man_distance,
                z,
            ),
        );
    }
    if let Some(h) = hold(w, b) {
        let z = location(w, h).z;
        ops::set_location(
            w,
            h,
            Point3D::new(l.x + xo * c.hold_distance, l.y + yo * c.hold_distance, z),
        );
    }
    if let Some(p) = p_plank(w, b) {
        let q = rotate(w, b, location(w, p), count);
        ops::set_location(w, p, q);
    }
    if let Some(p) = s_plank(w, b) {
        let q = rotate(w, b, location(w, p), count);
        ops::set_location(w, p, q);
    }
    let item_id = match new_facing {
        0 => Some(c.north_id),
        2 => Some(c.east_id),
        4 => Some(c.south_id),
        6 => Some(c.west_id),
        _ => None,
    };
    if let Some(id) = item_id
        && let Some(ty) = w.item(b).map(|i| i.ty)
    {
        (ty.set_item_id)(w, b, id);
    }
    true
}

// ---- naming, maps, courses ----

/// `BeginRename(from)`.
pub fn begin_rename(w: &mut World, b: ItemId, from: MobileId) {
    if check_decay(w, b) {
        return;
    }
    let gm = w
        .mobile(from)
        .is_some_and(|m| m.access_level >= AccessLevel::GameMaster);
    if !gm && owner(w, b) != Some(from) {
        // Arr, don't do that! | Arr, leave me alone! | Arr, watch what thour'rt doing, matey! | Arr! Do that again..
        let n = utility::random_from(&mut w.rng, 1042876, 4);
        say(w, b, n);
        return;
    }
    say(w, b, 502580); // What dost thou wish to name thy ship?
    let prompt = muo_server::prompt::Prompt {
        on_response: Some(std::rc::Rc::new(move |w, from, text| {
            end_rename(w, b, from, text)
        })),
        on_cancel: None,
    };
    let id = muo_server::prompt::add_prompt(w, prompt);
    muo_server::mobile::part1::props_b::set_prompt(w, from, Some(id));
}

/// `EndRename(from, newName)`.
pub fn end_rename(w: &mut World, b: ItemId, from: MobileId, new_name: &str) {
    if deleted(w, b) || check_decay(w, b) {
        return;
    }
    let gm = w
        .mobile(from)
        .is_some_and(|m| m.access_level >= AccessLevel::GameMaster);
    if !gm && owner(w, b) != Some(from) {
        say(w, b, 1042880); // Arr! Only the owner of the ship may change its name!
        return;
    }
    if !muo_server::mobile::base::alive(w, from) {
        say(w, b, 502582); // You appear to be dead.
        return;
    }
    let t = new_name.trim();
    rename(
        w,
        b,
        if t.is_empty() {
            None
        } else {
            Some(t.to_string())
        },
    );
}

/// `Rename(newName)`.
pub fn rename(w: &mut World, b: ItemId, new_name: Option<String>) {
    if check_decay(w, b) {
        return;
    }
    let mut new_name = new_name;
    if let Some(n) = new_name.as_mut()
        && n.chars().count() > 40
    {
        *n = n.chars().take(40).collect();
    }
    if ship_name(w, b) == new_name {
        say(w, b, 502531); // Yes, sir.
        return;
    }
    set_ship_name(w, b, new_name);
    match ship_name(w, b) {
        Some(n) if tiller_man(w, b).is_some() => say_args(w, b, 1042885, &n), // This ship is now called the ~1_NEW_SHIP_NAME~.
        _ => say(w, b, 502534), // This ship now has no name.
    }
}

/// `SetName(e)`: "name ..." spoken.
fn set_name(w: &mut World, b: ItemId, e: &SpeechEventArgs) {
    if check_decay(w, b) {
        return;
    }
    let gm = w
        .mobile(e.mobile)
        .is_some_and(|m| m.access_level >= AccessLevel::GameMaster);
    if !gm && owner(w, b) != Some(e.mobile) {
        say(w, b, 1042880); // Arr! Only the owner of the ship may change its name!
        return;
    }
    if !muo_server::mobile::base::alive(w, e.mobile) {
        say(w, b, 502582); // You appear to be dead.
        return;
    }
    // e.Speech.Length > 8: UTF-16 units
    if e.speech.encode_utf16().count() > 8 {
        let rest: String =
            String::from_utf16_lossy(&e.speech.encode_utf16().skip(8).collect::<Vec<u16>>());
        let t = rest.trim();
        rename(
            w,
            b,
            if t.is_empty() {
                None
            } else {
                Some(t.to_string())
            },
        );
    }
}

/// `RemoveName(m)`.
fn remove_name(w: &mut World, b: ItemId, m: MobileId) {
    if check_decay(w, b) {
        return;
    }
    let gm = w
        .mobile(m)
        .is_some_and(|x| x.access_level >= AccessLevel::GameMaster);
    if !gm && owner(w, b) != Some(m) {
        say(w, b, 1042880); // Arr! Only the owner of the ship may change its name!
        return;
    }
    if !muo_server::mobile::base::alive(w, m) {
        say(w, b, 502582); // You appear to be dead.
        return;
    }
    if ship_name(w, b).is_none() {
        say(w, b, 502526); // Ar, this ship has no name.
        return;
    }
    set_ship_name(w, b, None);
    say(w, b, 502534); // This ship now has no name.
}

/// `GiveName(m)`.
fn give_name(w: &mut World, b: ItemId) {
    if tiller_man(w, b).is_none() || check_decay(w, b) {
        return;
    }
    match ship_name(w, b) {
        None => say(w, b, 502526),              // Ar, this ship has no name.
        Some(n) => say_args(w, b, 1042881, &n), // This is the ~1_BOAT_NAME~.
    }
}

/// `GiveNavPoint()`.
fn give_nav_point(w: &mut World, b: ItemId) {
    if tiller_man(w, b).is_none() || check_decay(w, b) {
        return;
    }
    let n = data(w, b).map_or(-1, |d| d.next_nav_point);
    if n < 0 {
        say(w, b, 1042882); // I have no current nav point.
    } else {
        say_args(w, b, 1042883, &(n + 1).to_string()); // My current destination navpoint is nav ~1_NAV_POINT_NUM~.
    }
}

/// `AssociateMap(map)`.
pub fn associate_map(w: &mut World, b: ItemId, map: ItemId) {
    if check_decay(w, b) {
        return;
    }
    let pins = hooks(w)
        .and_then(|h| h.map_pins)
        .and_then(|f| f(w, map))
        .unwrap_or_default();
    if hooks(w)
        .and_then(|h| h.is_blank_map)
        .is_some_and(|f| f(w, map))
    {
        say(w, b, 502575); // Ar, that is not a map, tis but a blank piece of paper!
    } else if pins.is_empty() {
        say(w, b, 502576); // Arrrr, this map has no course on it!
    } else {
        stop_move(w, b, false);
        if let Some(d) = data_mut(w, b) {
            d.map_item = Some(map);
            d.next_nav_point = -1;
        }
        say(w, b, 502577); // A map!
    }
}

/// `StartCourse(navPoint, single, message)`.
pub fn start_course_nav(
    w: &mut World,
    b: ItemId,
    nav_point: &str,
    single: bool,
    message: bool,
) -> bool {
    let chars: Vec<char> = nav_point.chars().collect();
    let mut number: i32 = -1;
    if let Some(start) = chars.iter().position(|c| c.is_ascii_digit()) {
        let s: String = chars[start..].iter().collect();
        number = s.trim().parse::<i32>().unwrap_or(-1);
        // `int.TryParse` also accepts a sign and surrounding white space; anything else fails
        if number != -1 {
            number -= 1;
            let pins = data(w, b)
                .and_then(|d| d.map_item)
                .and_then(|m| hooks(w).and_then(|h| h.map_pins).and_then(|f| f(w, m)));
            match pins {
                Some(p) if number >= 0 && (number as usize) < p.len() => {}
                _ => number = -1,
            }
        }
    }
    if number == -1 {
        if message {
            say(w, b, 1042551); // I don't see that navpoint, sir.
        }
        return false;
    }
    if let Some(d) = data_mut(w, b) {
        d.next_nav_point = number;
    }
    start_course(w, b, single, message)
}

/// `StartCourse(single, message)`.
pub fn start_course(w: &mut World, b: ItemId, single: bool, message: bool) -> bool {
    if check_decay(w, b) {
        return false;
    }
    if anchored(w, b) {
        if message {
            say(w, b, 501419); // Ar, the anchor is down sir!
        }
        return false;
    }
    let map_item = data(w, b)
        .and_then(|d| d.map_item)
        .filter(|&m| !deleted(w, m));
    let Some(mi) = map_item else {
        if message {
            say(w, b, 502513); // I have seen no map, sir.
        }
        return false;
    };
    let mi_loc = ops::get_world_location(w, mi);
    if map_of(w, b) != map_of(w, mi) || !contains(w, b, mi_loc.x, mi_loc.y) {
        if message {
            say(w, b, 502514); // The map is too far away from me, sir.
        }
        return false;
    }
    let pins = hooks(w)
        .and_then(|h| h.map_pins)
        .and_then(|f| f(w, mi))
        .unwrap_or_default();
    let n = data(w, b).map_or(-1, |d| d.next_nav_point);
    let brit = map_of(w, b) == Some(MapId::TRAMMEL) || map_of(w, b) == Some(MapId::FELUCCA);
    if !brit || n < 0 || n as usize >= pins.len() {
        if message {
            say(w, b, 1042551); // I don't see that navpoint, sir.
        }
        return false;
    }
    let fs = fast_speed(w);
    if let Some(d) = data_mut(w, b) {
        d.speed = fs;
        d.order = if single {
            BoatOrder::Single
        } else {
            BoatOrder::Course
        };
    }
    let fi = fast_interval(w);
    if !safely_start_move_timer(w, b, fi, single) {
        return false;
    }
    if message {
        say(w, b, 501429); // Aye aye sir.
    }
    true
}

// ---- dry dock ----

/// `CheckDryDock(from)`.
pub fn check_dry_dock(w: &mut World, b: ItemId, from: MobileId) -> DryDockResult {
    if check_decay(w, b) {
        return DryDockResult::Decaying;
    }
    if !muo_server::mobile::base::alive(w, from) {
        return DryDockResult::Dead;
    }
    let pack = muo_server::mobile::part1::props_b::backpack(w, from);
    let has = |w: &World, plank: Option<ItemId>| {
        plank.is_some_and(|p| {
            pack.is_some_and(|pk| gold::contains_key(w, pk, parts::plank_key_value(w, p)))
        })
    };
    // `(SPlank == null || !ContainsKey(..)) && (PPlank == null || !ContainsKey(..))`
    if !has(w, s_plank(w, b)) && !has(w, p_plank(w, b)) {
        return DryDockResult::NoKey;
    }
    if !anchored(w, b) {
        return DryDockResult::NotAnchored;
    }
    if hold(w, b).is_some_and(|h| w.item(h).is_some_and(|i| !i.items.is_empty())) {
        return DryDockResult::Hold;
    }
    let map = map_of(w, b);
    if map.is_none() || map == Some(MapId::INTERNAL) {
        return DryDockResult::Items;
    }
    if let Some(e) = get_moving_entities(w, b, false).first() {
        return if matches!(e, EntityId::Mobile(_)) {
            DryDockResult::Mobiles
        } else {
            DryDockResult::Items
        };
    }
    DryDockResult::Valid
}

fn dry_dock_message(w: &mut World, from: MobileId, result: DryDockResult) {
    let number = match result {
        DryDockResult::Dead => 502493,         // You appear to be dead.
        DryDockResult::NoKey => 502494,        // You must have a key to the ship to dock the boat.
        DryDockResult::NotAnchored => 1010570, // You must lower the anchor to dock the boat.
        DryDockResult::Mobiles => 502495,      // You cannot dock the ship with beings on board!
        DryDockResult::Items => 502496,        // You cannot dock the ship with a cluttered deck.
        DryDockResult::Hold => 502497,         // Make sure your hold is empty, and try again!
        _ => return,
    };
    muo_server::item::deps::send_localized_message(w, from, number);
}

/// `BeginDryDock(from)`.
pub fn begin_dry_dock(w: &mut World, b: ItemId, from: MobileId) {
    if check_decay(w, b) {
        return;
    }
    let result = check_dry_dock(w, b, from);
    if result == DryDockResult::Valid {
        items::send_confirm_dry_dock_gump(w, from, b);
    } else {
        dry_dock_message(w, from, result);
    }
}

/// `EndDryDock(from)`.
pub fn end_dry_dock(w: &mut World, b: ItemId, from: MobileId) {
    if deleted(w, b) || check_decay(w, b) {
        return;
    }
    let result = check_dry_dock(w, b, from);
    dry_dock_message(w, from, result);
    if result != DryDockResult::Valid {
        return;
    }
    let Some(docked) = (class(w, b).docked_boat)(w, b) else {
        return;
    };
    remove_keys(w, b, from);
    muo_server::mobile::part2::items::add_to_backpack(w, from, docked);
    delete_item(w, b);
}

/// `CanCommand(m)`.
pub fn can_command(_w: &World, _b: ItemId, _m: MobileId) -> bool {
    true
}

// ---- speech ----

fn on_speech(w: &mut World, b: ItemId, e: &mut SpeechEventArgs) {
    if check_decay(w, b) {
        return;
    }
    let from = e.mobile;
    if !(can_command(w, b, from) && contains_mobile(w, b, from)) {
        return;
    }
    for &keyword in &e.keywords {
        if !(0x42..=0x6B).contains(&keyword) {
            continue;
        }
        match keyword {
            0x42 => set_name(w, b, e),
            0x43 => remove_name(w, b, from),
            0x44 => give_name(w, b),
            0x45 => {
                start_move_fast(w, b, FORWARD, true);
            }
            0x46 => {
                start_move_fast(w, b, BACKWARD, true);
            }
            0x47 => {
                start_move_fast(w, b, LEFT, true);
            }
            0x48 => {
                start_move_fast(w, b, RIGHT, true);
            }
            0x4B => {
                start_move_fast(w, b, FORWARD_LEFT, true);
            }
            0x4C => {
                start_move_fast(w, b, FORWARD_RIGHT, true);
            }
            0x4D => {
                start_move_fast(w, b, BACKWARD_LEFT, true);
            }
            0x4E => {
                start_move_fast(w, b, BACKWARD_RIGHT, true);
            }
            0x4F => {
                stop_move(w, b, true);
            }
            0x50 => {
                start_move_fast(w, b, LEFT, false);
            }
            0x51 => {
                start_move_fast(w, b, RIGHT, false);
            }
            0x52 => {
                start_move_fast(w, b, FORWARD, false);
            }
            0x53 => {
                start_move_fast(w, b, BACKWARD, false);
            }
            0x54 => {
                start_move_fast(w, b, FORWARD_LEFT, false);
            }
            0x55 => {
                start_move_fast(w, b, FORWARD_RIGHT, false);
            }
            0x56 => {
                start_move_fast(w, b, BACKWARD_RIGHT, false);
            }
            0x57 => {
                start_move_fast(w, b, BACKWARD_LEFT, false);
            }
            0x58 => {
                one_move(w, b, LEFT);
            }
            0x59 => {
                one_move(w, b, RIGHT);
            }
            0x5A => {
                one_move(w, b, FORWARD);
            }
            0x5B => {
                one_move(w, b, BACKWARD);
            }
            0x5C => {
                one_move(w, b, FORWARD_LEFT);
            }
            0x5D => {
                one_move(w, b, FORWARD_RIGHT);
            }
            0x5E => {
                one_move(w, b, BACKWARD_RIGHT);
            }
            0x5F => {
                one_move(w, b, BACKWARD_LEFT);
            }
            0x49 | 0x65 => {
                start_turn(w, b, 2, true); // turn right
            }
            0x4A | 0x66 => {
                start_turn(w, b, -2, true); // turn left
            }
            0x67 => {
                start_turn(w, b, -4, true); // turn around, come about
            }
            0x68 => {
                start_move_fast(w, b, FORWARD, true);
            }
            0x69 => {
                stop_move(w, b, true);
            }
            0x6A => {
                lower_anchor(w, b, true);
            }
            0x6B => {
                raise_anchor(w, b, true);
            }
            0x60 => give_nav_point(w, b), // nav
            0x61 => {
                // start
                if let Some(d) = data_mut(w, b) {
                    d.next_nav_point = 0;
                }
                start_course(w, b, false, true);
            }
            0x62 => {
                start_course(w, b, false, true); // continue
            }
            0x63 => {
                let s = e.speech.clone();
                start_course_nav(w, b, &s, false, true); // goto*
            }
            0x64 => {
                let s = e.speech.clone();
                start_course_nav(w, b, &s, true, true); // single*
            }
            _ => {}
        }
        break;
    }
}

// ---- virtuals ----

fn on_after_delete(w: &mut World, b: ItemId) {
    for part in [tiller_man(w, b), hold(w, b), p_plank(w, b), s_plank(w, b)]
        .into_iter()
        .flatten()
    {
        delete_item(w, part);
    }
    if let Some(d) = data(w, b) {
        for t in [
            d.turn_timer.map(|t| t.handle),
            d.move_timer.map(|t| t.handle),
        ]
        .into_iter()
        .flatten()
        {
            timer::stop(w, t);
        }
    }
    statics(w).boats.retain(|&x| x != b);
}

fn on_location_change(w: &mut World, b: ItemId, old: Point3D) {
    (BASE_MULTI.on_location_change)(w, b, old);
    let l = location(w, b);
    for part in [tiller_man(w, b), hold(w, b), p_plank(w, b), s_plank(w, b)]
        .into_iter()
        .flatten()
    {
        let p = location(w, part);
        ops::set_location(
            w,
            part,
            Point3D::new(
                l.x + (p.x - old.x),
                l.y + (p.y - old.y),
                l.z + (p.z - old.z),
            ),
        );
    }
}

fn on_map_change(w: &mut World, b: ItemId) {
    (BASE_MULTI.on_map_change)(w, b);
    let map = map_of(w, b);
    for part in [tiller_man(w, b), hold(w, b), p_plank(w, b), s_plank(w, b)]
        .into_iter()
        .flatten()
    {
        ops::set_map(w, part, map);
    }
}

// ---- save ----

/// The version `BaseBoat` writes.
pub const VERSION: i32 = 5;

fn serialize(w: &World, b: ItemId, out: &mut dyn GenericWriter) {
    (BASE_MULTI.serialize)(w, b, out);
    out.write_i32(VERSION);
    let d = data(w, b).cloned().unwrap_or_default();
    out.write_item(w, d.map_item);
    out.write_i32(d.next_nav_point);
    out.write_enum(1, i64::from(d.facing));
    out.write_anchored_time_ticks(d.time_of_decay.ticks());
    out.write_mobile(w, d.owner);
    out.write_item(w, d.p_plank);
    out.write_item(w, d.s_plank);
    out.write_item(w, d.tiller_man);
    out.write_item(w, d.hold);
    out.write_bool(d.anchored);
    out.write_opt_string(d.ship_name.as_deref());
}

fn deserialize(w: &mut World, b: ItemId, r: &mut dyn GenericReader) -> Result<()> {
    (BASE_MULTI.deserialize)(w, b, r)?;
    let version = r.read_i32()?;
    // Versions below 4 have the same fields; 4 and below hold the time of
    // decay as a delta time, 5 as an anchored time.
    let mut d = BoatData {
        map_item: r.read_item(w)?,
        next_nav_point: r.read_i32()?,
        facing: r.read_enum(1)? as i32,
        ..BoatData::default()
    };
    let ticks = if version <= 4 {
        r.read_delta_time_ticks(DateTime::utc_now().ticks())?
    } else {
        r.read_anchored_time_ticks()?
    };
    d.time_of_decay = DateTime::from_ticks(ticks);
    d.owner = r.read_mobile(w)?;
    d.p_plank = r
        .read_item(w)?
        .filter(|&i| w.item(i).is_some_and(|x| x.ty.is(&parts::PLANK)));
    d.s_plank = r
        .read_item(w)?
        .filter(|&i| w.item(i).is_some_and(|x| x.ty.is(&parts::PLANK)));
    d.tiller_man = r
        .read_item(w)?
        .filter(|&i| w.item(i).is_some_and(|x| x.ty.is(&parts::TILLER_MAN)));
    d.hold = r
        .read_item(w)?
        .filter(|&i| w.item(i).is_some_and(|x| x.ty.is(&parts::HOLD)));
    d.anchored = r.read_bool()?;
    d.ship_name = r.read_string()?;
    if let Some(i) = w.item_mut(b) {
        i.ext.insert(d);
    }
    // [AfterDeserialization(false)] (a delayed call): Boats.Add(this); CheckDecay()
    timer::start(w, TimeSpan::ZERO, TimeSpan::ZERO, 1, move |w| {
        if deleted(w, b) {
            return;
        }
        statics(w).boats.push(b);
        check_decay(w, b);
    });
    Ok(())
}

/// `Server.Multis.BaseBoat` (abstract).
pub static BASE_BOAT: ItemType = ItemType {
    full_name: "Server.Multis.BaseBoat",
    parent: Some(&BASE_MULTI),
    construct: None,
    family: Some(&BASE_BOAT_CLASS),
    handles_on_speech: |_, _| true,
    allows_relative_drop: |_, _| true,
    on_speech,
    multi_contains: contains_xy,
    on_after_delete,
    on_location_change,
    on_map_change,
    serialize,
    deserialize,
    ..BASE_MULTI
};

/// `BaseBoat.Initialize`: `WorldLoad`/`WorldSave` fix the parts' places.
pub fn initialize(w: &mut World) {
    strandedness::initialize(w);
    w.events
        .world_load
        .subscribe(|w, _| update_all_components(w));
    w.events
        .world_save
        .subscribe(|w, _| update_all_components(w));
    // Fix tiller/hold/plank locations
    update_all_components(w);
}

pub fn register(types: &mut TypeRegistry) {
    parts::register(types);
    items::register(types);
    types.register_phase_default(
        muo_server::type_registry::Phase::Initialize,
        "Server.Multis.BaseBoat.Initialize",
        initialize,
    );
}
