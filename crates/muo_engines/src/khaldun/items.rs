//! `KhaldunPitTeleporter`, `RaisableItem` and `RaiseSwitch` (`RaiseSwitch` and
//! `DisappearingRaiseSwitch`).

use std::cell::Cell;
use std::rc::Rc;

use muo_primitives::{DateTime, MessageType, Point3D, TimeSpan};
use muo_server::World;
use muo_server::ext::Ext;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType, ops};
use muo_server::map::MapId;
use muo_server::serialization_ext::{ReadEntityExt, WriteEntityExt};
use muo_server::timer::{self, TimerHandle};
use muo_server::type_registry::TypeRegistry;

fn deleted(w: &World, id: ItemId) -> bool {
    w.item(id).is_none_or(|i| i.deleted())
}

fn loc(w: &World, id: ItemId) -> Point3D {
    w.item(id).map_or(Point3D::ZERO, |i| i.location)
}

fn play_sound(w: &mut World, id: ItemId, sound: i32) {
    let (l, m) = w
        .item(id)
        .map_or((Point3D::ZERO, None), |i| (i.location, i.map));
    muo_server::effects::play_sound(w, l, m, sound);
}

// ---- KhaldunPitTeleporter ----

#[derive(Clone, Copy)]
pub struct PitData {
    pub active: bool,
    pub point_dest: Point3D,
    pub map_dest: Option<MapId>,
}

fn pit(w: &World, id: ItemId) -> Option<PitData> {
    w.item(id)?.ext.get::<PitData>().copied()
}

/// `KhaldunPitTeleporter(pointDest, mapDest)`: `base(0x053B)`, `Movable = false; Hue = 1`.
pub fn pit_new(w: &mut World, point_dest: Point3D, map_dest: Option<MapId>) -> ItemId {
    let id = ops::new_item_with_id(w, &KHALDUN_PIT_TELEPORTER, 0x053B);
    ops::set_movable(w, id, false);
    crate::util::hue(w, id, 1);
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(PitData {
            active: true,
            point_dest,
            map_dest,
        });
    }
    id
}

fn pit_use(w: &mut World, id: ItemId, m: MobileId) {
    let Some(d) = pit(w, id) else { return };
    if !d.active {
        return;
    }
    let near = muo_content::mx::in_range_point(w, m, ops::get_world_location(w, id), 3);
    match d.map_dest {
        Some(map) if map != MapId::INTERNAL && near => {
            muo_content::creature::pets::teleport_pets(w, m, d.point_dest, map, false);
            muo_server::mobile::base::move_to_world(w, m, d.point_dest, map);
        }
        _ => muo_content::mx::msg(w, m, 1019045), // I can't reach that.
    }
}

/// `[SerializationGenerator(0, false)] KhaldunPitTeleporter`.
pub static KHALDUN_PIT_TELEPORTER: ItemType = ItemType {
    full_name: "Server.Items.KhaldunPitTeleporter",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| pit_new(w, Point3D::new(5451, 1374, 0), Some(MapId::FELUCCA))),
    // the floor of the cavern seems to have collapsed here - a faint light is visible at the bottom of the pit
    label_number: |_, _| 1016511,
    on_double_click: pit_use,
    on_double_click_dead: pit_use,
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        let d = pit(w, id).unwrap_or(PitData {
            active: false,
            point_dest: Point3D::ZERO,
            map_dest: None,
        });
        out.write_i32(0);
        out.write_bool(d.active);
        out.write_point3d(d.point_dest);
        out.write_map(d.map_dest);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        let d = PitData {
            active: r.read_bool()?,
            point_dest: r.read_point3d()?,
            map_dest: r.read_map()?,
        };
        if let Some(i) = w.item_mut(id) {
            i.ext = Ext::new(d);
        }
        Ok(())
    },
    ..ITEM_BASE
};

// ---- RaisableItem ----

/// What `RaisableItem` holds.
#[derive(Clone, Copy, Default)]
pub struct RaisableData {
    pub max_elevation: i32,
    pub move_sound: i32,
    pub stop_sound: i32,
    pub close_delay: TimeSpan,
    pub elevation: i32,
    // not serialized: `_raiseTimer != null`
    pub raising: bool,
}

fn raisable(w: &World, id: ItemId) -> Option<RaisableData> {
    w.item(id)?.ext.get::<RaisableData>().copied()
}

fn raisable_mut(w: &mut World, id: ItemId) -> Option<&mut RaisableData> {
    w.item_mut(id)?.ext.get_mut::<RaisableData>()
}

/// `RaisableItem(itemID, maxElevation, moveSound, stopSound, closeDelay)`; the shorter overloads pass
/// `20, -1, -1, 1 minute` and `maxElevation, -1, -1, closeDelay`.
pub fn raisable_new(
    w: &mut World,
    item_id: i32,
    max_elevation: i32,
    move_sound: i32,
    stop_sound: i32,
    close_delay: TimeSpan,
) -> ItemId {
    let id = ops::new_item_with_id(w, &RAISABLE_ITEM, item_id);
    ops::set_movable(w, id, false);
    if let Some(i) = w.item_mut(id) {
        // `_maxElevation = maxElevation` bypasses the clamping setter.
        i.ext = Ext::new(RaisableData {
            max_elevation,
            move_sound,
            stop_sound,
            close_delay,
            elevation: 0,
            raising: false,
        });
    }
    id
}

/// `MaxElevation = value`.
pub fn set_max_elevation(w: &mut World, id: ItemId, value: i32) {
    if let Some(d) = raisable_mut(w, id) {
        d.max_elevation = value.clamp(0, 60);
    }
}

/// `IsRaisable`.
pub fn is_raisable(w: &World, id: ItemId) -> bool {
    raisable(w, id).is_some_and(|d| !d.raising)
}

fn set_z(w: &mut World, id: ItemId, dz: i32) {
    let l = loc(w, id);
    ops::set_location(w, id, Point3D::new(l.x, l.y, l.z + dz));
}

/// `Raise()`: the `RaiseTimer`.
pub fn raise(w: &mut World, id: ItemId) {
    if !is_raisable(w, id) {
        return;
    }
    let close_delay = raisable(w, id).map_or(TimeSpan::ZERO, |d| d.close_delay);
    if let Some(d) = raisable_mut(w, id) {
        d.raising = true;
    }
    let close_time = w.now + close_delay;
    let handle: Rc<Cell<TimerHandle>> = Rc::new(Cell::new(TimerHandle::default()));
    let (step, up) = (Cell::new(0i32), Cell::new(true));
    let h = timer::create(w, TimeSpan::ZERO, TimeSpan::from_millis(500), 0, {
        let handle = handle.clone();
        move |w| raise_tick(w, id, &handle, &step, &up, close_time)
    });
    handle.set(h);
    timer::begin(w, h);
}

fn raise_tick(
    w: &mut World,
    id: ItemId,
    handle: &Rc<Cell<TimerHandle>>,
    step: &Cell<i32>,
    up: &Cell<bool>,
    close_time: DateTime,
) {
    let h = handle.get();
    if deleted(w, id) {
        timer::stop(w, h);
        return;
    }
    let this_step = step.get();
    step.set(this_step + 1);
    if this_step % 3 == 0 {
        let Some(d) = raisable(w, id) else { return };
        if up.get() {
            set_z(w, id, 1);
            let elevation = d.elevation + 1;
            if let Some(x) = raisable_mut(w, id) {
                x.elevation = elevation;
            }
            if elevation >= d.max_elevation {
                timer::stop(w, h);
                if d.stop_sound >= 0 {
                    play_sound(w, id, d.stop_sound);
                }
                up.set(false);
                step.set(0);
                let delay = close_time - w.now;
                timer::start(w, delay, TimeSpan::ZERO, 1, move |w| timer::begin(w, h));
                return;
            }
        } else {
            set_z(w, id, -1);
            let elevation = d.elevation - 1;
            if let Some(x) = raisable_mut(w, id) {
                x.elevation = elevation;
            }
            if elevation <= 0 {
                timer::stop(w, h);
                if d.stop_sound >= 0 {
                    play_sound(w, id, d.stop_sound);
                }
                if let Some(x) = raisable_mut(w, id) {
                    x.raising = false;
                }
                return;
            }
        }
    }
    if let Some(d) = raisable(w, id)
        && d.move_sound >= 0
    {
        play_sound(w, id, d.move_sound);
    }
}

/// `[SerializationGenerator(0)] RaisableItem`.
pub static RAISABLE_ITEM: ItemType = ItemType {
    full_name: "Server.Items.RaisableItem",
    parent: Some(&ITEM_BASE),
    // `RaisableItem(int itemID)` and the other overloads need arguments: see the constructor table.
    construct: None,
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        let d = raisable(w, id).unwrap_or_default();
        out.write_encoded_int(0);
        out.write_encoded_int(d.max_elevation);
        out.write_i32(d.move_sound);
        out.write_i32(d.stop_sound);
        out.write_time_span_ticks(d.close_delay.ticks());
        out.write_i32(d.elevation);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let _version = r.read_encoded_int()?;
        let d = RaisableData {
            max_elevation: r.read_encoded_int()?,
            move_sound: r.read_i32()?,
            stop_sound: r.read_i32()?,
            close_delay: TimeSpan::from_ticks(r.read_time_span_ticks()?),
            elevation: r.read_i32()?,
            raising: false,
        };
        let elevation = d.elevation;
        if let Some(i) = w.item_mut(id) {
            i.ext = Ext::new(d);
        }
        // `[AfterDeserialization] Z -= _elevation`
        timer::start(w, TimeSpan::ZERO, TimeSpan::ZERO, 1, move |w| {
            set_z(w, id, -elevation)
        });
        Ok(())
    },
    ..ITEM_BASE
};

// ---- RaiseSwitch ----

/// What `RaiseSwitch` holds.
#[derive(Clone, Copy, Default)]
pub struct SwitchData {
    pub raisable_item: Option<ItemId>,
    pub reset_timer: Option<TimerHandle>,
}

fn switch(w: &World, id: ItemId) -> SwitchData {
    w.item(id)
        .and_then(|i| i.ext.get::<SwitchData>())
        .copied()
        .unwrap_or_default()
}

fn switch_mut(w: &mut World, id: ItemId) -> Option<&mut SwitchData> {
    w.item_mut(id)?.ext.get_mut::<SwitchData>()
}

/// `RaiseSwitch(itemID = 0x1093)` of class `ty`.
pub fn switch_new(w: &mut World, ty: &'static ItemType, item_id: i32) -> ItemId {
    let id = ops::new_item_with_id(w, ty, item_id);
    ops::set_movable(w, id, false);
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(SwitchData::default());
    }
    id
}

fn is_disappearing(w: &World, id: ItemId) -> bool {
    w.item(id)
        .is_some_and(|i| i.ty.is(&DISAPPEARING_RAISE_SWITCH))
}

fn set_item_id(w: &mut World, id: ItemId, graphic: i32) {
    if let Some(t) = w.item(id).map(|i| i.ty) {
        (t.set_item_id)(w, id, graphic);
    }
}

fn graphic(w: &World, id: ItemId) -> i32 {
    w.item(id).map_or(0, |i| i.item_id)
}

fn stop_reset_timer(w: &mut World, id: ItemId) {
    if let Some(h) = switch_mut(w, id).and_then(|d| d.reset_timer.take()) {
        timer::stop(w, h);
    }
}

fn start_reset_timer(w: &mut World, id: ItemId, delay: TimeSpan) {
    stop_reset_timer(w, id);
    let h = timer::start(w, delay, TimeSpan::ZERO, 1, move |w| {
        if deleted(w, id) {
            return;
        }
        if let Some(d) = switch_mut(w, id) {
            d.reset_timer = None;
        }
        reset(w, id);
    });
    if let Some(d) = switch_mut(w, id) {
        d.reset_timer = Some(h);
    }
}

/// `Flip()` (`DisappearingRaiseSwitch` overrides it with nothing).
fn flip(w: &mut World, id: ItemId) {
    if is_disappearing(w, id) {
        return;
    }
    if graphic(w, id) != 0x1093 {
        set_item_id(w, id, 0x1093);
        stop_reset_timer(w, id);
    } else {
        set_item_id(w, id, 0x1095);
        let delay = switch(w, id)
            .raisable_item
            .and_then(|r| raisable(w, r))
            .map(|d| d.close_delay)
            .filter(|d| *d >= TimeSpan::ZERO)
            .unwrap_or(TimeSpan::from_mins(2));
        start_reset_timer(w, id, delay);
    }
    play_sound(w, id, 0x3E8);
}

/// `Reset()` (nothing for `DisappearingRaiseSwitch`).
fn reset(w: &mut World, id: ItemId) {
    if is_disappearing(w, id) {
        return;
    }
    if graphic(w, id) != 0x1093 {
        flip(w, id);
    }
}

fn switch_use(w: &mut World, id: ItemId, m: MobileId) {
    if !muo_content::mx::in_range_point(w, m, loc(w, id), 2) {
        muo_server::mobile::part2::messages::local_overhead_message_number(
            w,
            m,
            MessageType::REGULAR,
            0x3B2,
            1019045,
            "",
        ); // I can't reach that.
        return;
    }
    if switch(w, id).raisable_item.is_some_and(|r| deleted(w, r))
        && let Some(d) = switch_mut(w, id)
    {
        d.raisable_item = None;
    }
    flip(w, id);
    let Some(r) = switch(w, id).raisable_item else {
        return;
    };
    if is_raisable(w, r) {
        raise(w, r);
        muo_server::mobile::part2::messages::local_overhead_message(
            w,
            m,
            MessageType::REGULAR,
            0x5A,
            true,
            "You hear a grinding noise echoing in the distance.",
        );
    } else {
        muo_server::mobile::part2::messages::local_overhead_message(
            w,
            m,
            MessageType::REGULAR,
            0x5A,
            true,
            "You flip the switch again, but nothing happens.",
        );
    }
}

fn switch_serialize(
    parent: &'static ItemType,
    w: &World,
    id: ItemId,
    out: &mut dyn muo_io::GenericWriter,
) {
    (parent.serialize)(w, id, out);
    out.write_encoded_int(0);
}

/// `[SerializationGenerator(0)] RaiseSwitch`.
pub static RAISE_SWITCH: ItemType = ItemType {
    full_name: "Server.Items.RaiseSwitch",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| switch_new(w, &RAISE_SWITCH, 0x1093)),
    on_double_click: switch_use,
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        out.write_encoded_int(0);
        out.write_item(w, switch(w, id).raisable_item);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let _version = r.read_encoded_int()?;
        let raisable_item = r
            .read_item(w)?
            .filter(|i| w.item(*i).is_some_and(|x| x.ty.is(&RAISABLE_ITEM)));
        if let Some(i) = w.item_mut(id) {
            i.ext = Ext::new(SwitchData {
                raisable_item,
                reset_timer: None,
            });
        }
        // `[AfterDeserialization(false)] Reset()`
        timer::start(w, TimeSpan::ZERO, TimeSpan::ZERO, 1, move |w| reset(w, id));
        Ok(())
    },
    ..ITEM_BASE
};

/// `RaiseSwitch.RaisableItem = value`.
pub fn set_raisable_item(w: &mut World, id: ItemId, value: Option<ItemId>) {
    if let Some(d) = switch_mut(w, id) {
        d.raisable_item = value;
    }
}

// ---- DisappearingRaiseSwitch ----

fn current_range(w: &World, id: ItemId) -> i32 {
    if w.item(id)
        .is_some_and(|i| i.flags.contains(muo_server::item::ImplFlag::VISIBLE))
    {
        3
    } else {
        2
    }
}

/// `Refresh()`: visible as soon as a visible (or staff) mobile is in range.
fn refresh(w: &mut World, id: ItemId) {
    let range = current_range(w, id);
    let l = loc(w, id);
    let map = w.item(id).and_then(|i| i.map);
    let near: Vec<MobileId> =
        muo_server::map::enumerate::mobiles_in_range(w, map, l.x, l.y, range).collect();
    for m in near {
        let (hidden, access) = w
            .mobile(m)
            .map_or((true, muo_server::mobile::AccessLevel::Player), |x| {
                (x.hidden, x.access_level)
            });
        if !hidden || access <= muo_server::mobile::AccessLevel::Player {
            ops::set_visible(w, id, true);
            break;
        }
    }
}

/// `[SerializationGenerator(0)] DisappearingRaiseSwitch`.
pub static DISAPPEARING_RAISE_SWITCH: ItemType = ItemType {
    full_name: "Server.Items.DisappearingRaiseSwitch",
    parent: Some(&RAISE_SWITCH),
    construct: Some(|w| switch_new(w, &DISAPPEARING_RAISE_SWITCH, 0x108F)),
    handles_on_movement: |_, _| true,
    on_movement: |w, id, m, old| {
        let range = current_range(w, id);
        let here = loc(w, id);
        let mloc = muo_content::mx::location(w, m);
        let in_range = |a: Point3D| (a.x - here.x).abs() <= range && (a.y - here.y).abs() <= range;
        if in_range(mloc) || in_range(old) {
            refresh(w, id);
        }
    },
    on_map_change: |w, id| {
        if !deleted(w, id) {
            refresh(w, id);
        }
    },
    on_location_change: |w, id, _old| {
        if !deleted(w, id) {
            refresh(w, id);
        }
    },
    serialize: |w, id, out| switch_serialize(&RAISE_SWITCH, w, id, out),
    deserialize: |w, id, r| {
        (RAISE_SWITCH.deserialize)(w, id, r)?;
        r.read_encoded_int()?;
        // `[AfterDeserialization(false)] Refresh()`
        timer::start(w, TimeSpan::ZERO, TimeSpan::ZERO, 1, move |w| {
            refresh(w, id)
        });
        Ok(())
    },
    ..RAISE_SWITCH
};

pub fn register(types: &mut TypeRegistry) {
    for t in [
        &KHALDUN_PIT_TELEPORTER,
        &RAISABLE_ITEM,
        &RAISE_SWITCH,
        &DISAPPEARING_RAISE_SWITCH,
    ] {
        types.register_item(t, &[]);
    }
}

#[allow(dead_code)]
fn _e(_: EntityId) {}

// ---- accessors of the property tables ----

/// The fields of a `KhaldunPitTeleporter`.
pub fn pit_data(w: &World, id: ItemId) -> Option<PitData> {
    pit(w, id)
}

/// Edits the fields of a `KhaldunPitTeleporter`.
pub fn pit_edit(w: &mut World, id: ItemId, f: impl FnOnce(&mut PitData)) {
    if let Some(d) = w.item_mut(id).and_then(|i| i.ext.get_mut::<PitData>()) {
        f(d);
    }
}

/// The fields of a `RaisableItem`.
pub fn raisable_data(w: &World, id: ItemId) -> Option<RaisableData> {
    raisable(w, id)
}

/// Edits the fields of a `RaisableItem`.
pub fn raisable_edit(w: &mut World, id: ItemId, f: impl FnOnce(&mut RaisableData)) {
    if let Some(d) = raisable_mut(w, id) {
        f(d);
    }
}

/// `RaiseSwitch.RaisableItem`.
pub fn switch_raisable(w: &World, id: ItemId) -> Option<ItemId> {
    w.item(id)?.ext.get::<SwitchData>()?.raisable_item
}
