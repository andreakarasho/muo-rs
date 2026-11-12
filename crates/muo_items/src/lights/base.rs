//! `BaseLight` and `BaseEquipableLight` (the two abstract bases; the leaves are in
//! `leaves.rs`). State is [`LightData`] in `Item::ext`; the virtuals a leaf overrides (`LitItemID`,
//! `UnlitItemID`, `BurntOutItemID`, the three sounds, and `Ignite` for the equipable lights) are the [`LightClass`]
//! table of the class.
//!
//! `Torch.OnAdded`/`Torch.Ignite` stop `MeerMage`'s effect on the holder (`LightHooks::meer_mage_stop`, set by the
//! mobiles crate). `Douse` with a running duration but no timer aborts the call
//! after the item id changed (the call returns at that point).

use muo_io::{GenericReader, GenericWriter, Result};
use muo_primitives::{DateTime, TimeSpan};
use muo_server::World;
use muo_server::ext::Ext;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType, base, deps, ops};
use muo_server::layer::Layer;
use muo_server::mobile::AccessLevel;
use muo_server::timer::{self, TimerHandle};

use crate::simple::{item_id, layer};

/// `BaseLight.Burnout`: `public static readonly bool Burnout = false`.
pub const BURNOUT: bool = false;

/// What `BaseLight` adds to an `Item`.
#[derive(Clone, Debug, Default)]
pub struct LightData {
    pub burnt_out: bool,
    pub burning: bool,
    /// The raw `_duration` field.
    pub duration: TimeSpan,
    pub protected: bool,
    pub burn_timer: TimerHandle,
}

/// Calls into systems of other crates (the mobiles), as `fn` pointers in `w.resources`; unset does nothing.
#[derive(Default)]
pub struct LightHooks {
    /// `MeerMage.StopEffect(mobile, true)`.
    pub meer_mage_stop: Option<fn(&mut World, MobileId, bool)>,
}

pub fn data(w: &World, id: ItemId) -> Option<&LightData> {
    w.item(id)?.ext.get::<LightData>()
}

pub fn data_mut(w: &mut World, id: ItemId) -> Option<&mut LightData> {
    w.item_mut(id)?.ext.get_mut::<LightData>()
}

/// `item is BaseLight`.
pub fn is_light(w: &World, id: ItemId) -> bool {
    w.item(id).is_some_and(|i| i.ty.is(&BASE_LIGHT))
}

/// The virtuals of `BaseLight` that a leaf overrides.
#[derive(Copy, Clone)]
pub struct LightClass {
    /// `LitItemID` (abstract; a few leaves read `ItemID`).
    pub lit_item_id: fn(&World, ItemId) -> i32,
    /// `UnlitItemID`.
    pub unlit_item_id: fn(&World, ItemId) -> i32,
    /// `BurntOutItemID`.
    pub burnt_out_item_id: fn(&World, ItemId) -> i32,
    pub lit_sound: i32,
    pub unlit_sound: i32,
    pub burnt_out_sound: i32,
    /// `Ignite()`.
    pub ignite: fn(&mut World, ItemId),
    /// `Douse()`.
    pub douse: fn(&mut World, ItemId),
    /// `Burn()`.
    pub burn: fn(&mut World, ItemId),
}

pub static BASE_LIGHT_CLASS: LightClass = LightClass {
    lit_item_id: |_, _| 0,
    unlit_item_id: |_, _| 0,
    burnt_out_item_id: |_, _| 0,
    lit_sound: 0x47,
    unlit_sound: 0x3be,
    burnt_out_sound: 0x4b8,
    ignite: ignite_base,
    douse: douse_base,
    burn: burn_base,
};

pub static BASE_EQUIPABLE_LIGHT_CLASS: LightClass = LightClass {
    ignite: ignite_equipable,
    ..BASE_LIGHT_CLASS
};

pub fn class(w: &World, id: ItemId) -> &'static LightClass {
    w.item(id)
        .and_then(|i| i.ty.family::<LightClass>())
        .unwrap_or(&BASE_LIGHT_CLASS)
}

// ---- BaseLight ----

/// `new BaseLight(itemID)` of class `ty`.
pub fn base_light_new(w: &mut World, ty: &'static ItemType, item_id: i32) -> ItemId {
    let id = ops::new_item_with_id(w, ty, item_id);
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(LightData::default());
    }
    id
}

pub fn burnt_out(w: &World, id: ItemId) -> bool {
    data(w, id).is_some_and(|d| d.burnt_out)
}

/// `BurntOut = value`.
pub fn set_burnt_out(w: &mut World, id: ItemId, value: bool) {
    if let Some(d) = data_mut(w, id) {
        d.burnt_out = value;
    }
}

pub fn protected(w: &World, id: ItemId) -> bool {
    data(w, id).is_some_and(|d| d.protected)
}

/// `Protected = value`.
pub fn set_protected(w: &mut World, id: ItemId, value: bool) {
    if let Some(d) = data_mut(w, id) {
        d.protected = value;
    }
}

pub fn burning(w: &World, id: ItemId) -> bool {
    data(w, id).is_some_and(|d| d.burning)
}

/// `Burning = value`: `if (_burning != value) { _burning = true; DoTimer(_duration); }` (sets `true`
/// whatever `value` is).
pub fn set_burning(w: &mut World, id: ItemId, value: bool) {
    let Some(d) = data(w, id) else { return };
    if d.burning != value {
        let duration = d.duration;
        if let Some(d) = data_mut(w, id) {
            d.burning = true;
        }
        do_timer(w, id, duration);
    }
}

/// `Duration`: the time left on the timer while it burns, else the field.
pub fn duration(w: &World, id: ItemId) -> TimeSpan {
    let Some(d) = data(w, id) else {
        return TimeSpan::ZERO;
    };
    if d.duration != TimeSpan::ZERO && d.burning && d.burn_timer != TimerHandle::default() {
        d.burn_timer.next(w) - w.now
    } else {
        d.duration
    }
}

/// `Duration = value`.
pub fn set_duration(w: &mut World, id: ItemId, value: TimeSpan) {
    if let Some(d) = data_mut(w, id) {
        d.duration = value;
    }
}

fn sound_at(w: &mut World, id: ItemId, sound: i32) {
    let loc = ops::get_world_location(w, id);
    let map = w.item(id).and_then(|i| i.map);
    muo_server::effects::play_sound(w, loc, map, sound);
}

/// `PlayLitSound()`.
pub fn play_lit_sound(w: &mut World, id: ItemId) {
    let s = class(w, id).lit_sound;
    if s != 0 {
        sound_at(w, id, s);
    }
}

/// `PlayUnlitSound()`.
pub fn play_unlit_sound(w: &mut World, id: ItemId) {
    let c = class(w, id);
    let sound = if burnt_out(w, id) && c.burnt_out_sound != 0 {
        c.burnt_out_sound
    } else {
        c.unlit_sound
    };
    if sound != 0 {
        sound_at(w, id, sound);
    }
}

/// `Ignite()` through the class.
pub fn ignite(w: &mut World, id: ItemId) {
    (class(w, id).ignite)(w, id);
}

fn ignite_base(w: &mut World, id: ItemId) {
    if !burnt_out(w, id) {
        play_lit_sound(w, id);
        let lit = (class(w, id).lit_item_id)(w, id);
        let duration = data(w, id).map_or(TimeSpan::ZERO, |d| d.duration);
        if let Some(d) = data_mut(w, id) {
            d.burning = true;
        }
        item_id(w, id, lit);
        do_timer(w, id, duration);
    }
}

/// `Douse()` through the class.
pub fn douse(w: &mut World, id: ItemId) {
    (class(w, id).douse)(w, id);
}

/// `BaseLight.Douse()`.
pub fn douse_base(w: &mut World, id: ItemId) {
    if let Some(d) = data_mut(w, id) {
        d.burning = false;
    }
    let c = class(w, id);
    let burnt = burnt_out(w, id);
    let burnt_id = (c.burnt_out_item_id)(w, id);
    let new_id = if burnt && burnt_id != 0 {
        burnt_id
    } else {
        (c.unlit_item_id)(w, id)
    };
    item_id(w, id, new_id);

    let Some(d) = data(w, id) else { return };
    if burnt {
        if let Some(d) = data_mut(w, id) {
            d.duration = TimeSpan::ZERO;
        }
    } else if d.duration != TimeSpan::ZERO {
        // `_burnTimer.Next` with no timer: the call ends here
        if d.burn_timer == TimerHandle::default() {
            return;
        }
        let left = d.burn_timer.next(w) - w.now;
        if let Some(d) = data_mut(w, id) {
            d.duration = left;
        }
    }
    let mut h = data(w, id).map_or(TimerHandle::default(), |d| d.burn_timer);
    h.cancel(w);
    if let Some(d) = data_mut(w, id) {
        d.burn_timer = TimerHandle::default();
    }
    play_unlit_sound(w, id);
}

/// `Burn()` through the class.
pub fn burn(w: &mut World, id: ItemId) {
    (class(w, id).burn)(w, id);
}

/// `BaseLight.Burn()`.
pub fn burn_base(w: &mut World, id: ItemId) {
    set_burnt_out(w, id, true);
    douse(w, id);
}

/// `DoTimer(delay)`.
fn do_timer(w: &mut World, id: ItemId, delay: TimeSpan) {
    let mut old = data(w, id).map_or(TimerHandle::default(), |d| d.burn_timer);
    if let Some(d) = data_mut(w, id) {
        d.duration = delay;
    }
    old.cancel(w);
    if let Some(d) = data_mut(w, id) {
        d.burn_timer = TimerHandle::default();
    }
    if delay == TimeSpan::ZERO {
        return;
    }
    let h = timer::start(w, delay, TimeSpan::ZERO, 1, move |w| {
        // InternalTimer.OnTick: `if (m_Light?.Deleted == false) m_Light.Burn()`
        if w.item(id).is_some() && !ops::is_deleted(w, id) {
            burn(w, id);
        }
    });
    if let Some(d) = data_mut(w, id) {
        d.burn_timer = h;
    }
}

/// `DeserializeTimer(delay)`.
fn deserialize_timer(w: &mut World, id: ItemId, delay: TimeSpan) {
    if let Some(d) = data(w, id)
        && d.burning
        && d.duration != TimeSpan::ZERO
    {
        do_timer(w, id, delay);
    }
}

fn on_double_click(w: &mut World, id: ItemId, from: MobileId) {
    let Some(d) = data(w, id) else { return };
    if d.burnt_out {
        return;
    }
    if d.protected
        && w.mobile(from)
            .is_some_and(|m| m.access_level == AccessLevel::Player)
    {
        return;
    }
    let loc = ops::get_world_location(w, id);
    if !muo_server::mobile::base::in_range_point3d(w, from, loc, 2) {
        return;
    }
    if !d.burning {
        ignite(w, id);
    } else if (class(w, id).unlit_item_id)(w, id) != 0 {
        douse(w, id);
    }
}

fn to_date(ticks: i64) -> DateTime {
    DateTime::try_from_ticks(ticks).unwrap_or(DateTime::MIN)
}

fn serialize(w: &World, id: ItemId, out: &mut dyn GenericWriter) {
    (ITEM_BASE.serialize)(w, id, out);
    out.write_i32(2); // SerializationVersion
    let d = data(w, id).cloned().unwrap_or_default();
    out.write_bool(d.burnt_out);
    out.write_bool(d.burning);
    out.write_time_span_ticks(d.duration.ticks());
    out.write_bool(d.protected);
    let next = if d.burn_timer == TimerHandle::default() {
        DateTime::MIN
    } else {
        d.burn_timer.next(w)
    };
    out.write_anchored_time_ticks(next.ticks());
}

fn deserialize(w: &mut World, id: ItemId, r: &mut dyn GenericReader) -> Result<()> {
    (ITEM_BASE.deserialize)(w, id, r)?;
    let version = r.read_i32()?;
    let now = w.now;
    let mut d = LightData::default();
    let mut timer_delay = None;
    let mut legacy_timer = false;
    d.burnt_out = r.read_bool()?;
    d.burning = r.read_bool()?;
    d.duration = TimeSpan::from_ticks(r.read_time_span_ticks()?);
    d.protected = r.read_bool()?;
    match version {
        1 => {
            // V1Content: BurnTimerNext (delta time), then MigrateFrom
            let next = to_date(r.read_delta_time_ticks(now.ticks())?);
            if next != DateTime::MIN {
                timer_delay = Some(next - now);
            }
        }
        2.. => {
            let next = to_date(r.read_anchored_time_ticks()?);
            if next != DateTime::MIN {
                timer_delay = Some(next - now);
            }
        }
        _ => {
            // Deserialize(reader, version): the delta time is only there for a burning light with a duration
            if d.burning && d.duration != TimeSpan::ZERO {
                let next = to_date(r.read_delta_time_ticks(now.ticks())?);
                timer_delay = Some(next - now);
                legacy_timer = true;
            }
        }
    }
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(d);
    }
    if let Some(delay) = timer_delay {
        if legacy_timer {
            do_timer(w, id, delay);
        } else {
            deserialize_timer(w, id, delay);
        }
    }
    Ok(())
}

/// `Server.Items.BaseLight` (abstract): `[SerializationGenerator(2, false)]`.
pub static BASE_LIGHT: ItemType = ItemType {
    full_name: "Server.Items.BaseLight",
    parent: Some(&ITEM_BASE),
    construct: None,
    family: Some(&BASE_LIGHT_CLASS),
    on_double_click,
    serialize,
    deserialize,
    ..ITEM_BASE
};

// ---- BaseEquipableLight ----

/// `new BaseEquipableLight(itemID)`: `Layer = Layer.TwoHanded`.
pub fn base_equipable_light_new(w: &mut World, ty: &'static ItemType, item_id: i32) -> ItemId {
    let id = base_light_new(w, ty, item_id);
    layer(w, id, Layer::TwoHanded);
    id
}

/// `SplitStack()`: lifts one off a stack and gives it this light's state.
fn split_stack(w: &mut World, id: ItemId) -> Option<ItemId> {
    let (stackable, amount, light) = {
        let i = w.item(id)?;
        (
            i.flags.contains(muo_server::item::ImplFlag::STACKABLE),
            i.amount,
            i.light(),
        )
    };
    if !stackable || amount < 2 {
        return None;
    }
    let stack = muo_server::mobile::part2::items::lift_item_dupe(w, id, 1)?;
    let d = data(w, id).cloned().unwrap_or_default();
    let dur = duration(w, id);
    if is_light(w, stack) {
        set_burnt_out(w, stack, d.burnt_out);
        set_duration(w, stack, dur);
        set_protected(w, stack, d.protected);
    }
    if let Some(l) = light {
        ops::set_light(w, stack, l);
    }
    Some(stack)
}

fn named(w: &World, id: ItemId, name: &str) -> bool {
    w.item(id)
        .is_some_and(|i| muo_content::items::is_named(i.ty, name))
}

fn equipable_on_equip(w: &mut World, id: ItemId, from: MobileId) -> bool {
    if !base::on_equip_base(w, id, from) {
        return false;
    }
    let stack = split_stack(w, id);
    if let Some(stack) = stack {
        let backpack = muo_server::mobile::part1::props_b::backpack(w, from);
        let parent = w.item(stack).and_then(|i| i.parent);
        if parent != backpack.map(EntityId::Item) {
            if deps_add_to_backpack(w, from, stack) {
                if named(w, id, "Server.Items.Candle") {
                    deps::send_localized_message(w, from, 502967); // You put the remaining unlit candles into your backpack.
                } else if named(w, id, "Server.Items.Torch") {
                    deps::send_localized_message(w, from, 502970); // You put the remaining unlit torches into your backpack.
                }
            } else {
                let (loc, map) = w
                    .mobile(from)
                    .map_or((Default::default(), None), |m| (m.location, m.map));
                ops::move_to_world_map(w, stack, loc, map);
            }
        }
    }
    true
}

fn deps_add_to_backpack(w: &mut World, from: MobileId, item: ItemId) -> bool {
    muo_server::mobile::part2::items::add_to_backpack(w, from, item)
}

fn ignite_equipable(w: &mut World, id: ItemId) {
    let parent = w.item(id).and_then(|i| i.parent);
    let root = ops::root_parent(w, id);
    if !matches!(parent, Some(EntityId::Mobile(_)))
        && let Some(EntityId::Mobile(holder)) = root
    {
        if muo_server::mobile::base::equip_item(w, holder, id) {
            if named(w, id, "Server.Items.Candle") {
                deps::send_localized_message(w, holder, 502969); // You put the candle in your left hand.
            } else if named(w, id, "Server.Items.Torch") {
                deps::send_localized_message(w, holder, 502972); // You put the torch in your left hand.
            }
            // No message for lanterns?
        } else {
            split_stack(w, id);
            let (loc, map) = w
                .mobile(holder)
                .map_or((Default::default(), None), |m| (m.location, m.map));
            ops::move_to_world_map(w, id, loc, map);
            if named(w, id, "Server.Items.Candle") {
                deps::send_localized_message(w, holder, 502968); // You cannot hold the candle, so it has been placed at your feet.
            } else if named(w, id, "Server.Items.Torch") {
                // 502971 has the wrong message
                deps::send_message(
                    w,
                    holder,
                    "You cannot hold the torch, so it has been placed at your feet.",
                );
            }
            // No message for lanterns?
        }
    } else {
        split_stack(w, id);
    }
    ignite_base(w, id);
}

fn equipable_on_added(w: &mut World, id: ItemId, parent: EntityId) {
    if burning(w, id) && matches!(parent, EntityId::Item(c) if ops::is_container(w, c)) {
        douse(w, id);
    }
    base::on_added_base(w, id, parent);
}

/// `Server.Items.BaseEquipableLight` (abstract): `[SerializationGenerator(0, false)]`.
pub static BASE_EQUIPABLE_LIGHT: ItemType = ItemType {
    full_name: "Server.Items.BaseEquipableLight",
    parent: Some(&BASE_LIGHT),
    construct: None,
    family: Some(&BASE_EQUIPABLE_LIGHT_CLASS),
    on_equip: equipable_on_equip,
    on_added: equipable_on_added,
    serialize: |w, id, out| {
        (BASE_LIGHT.serialize)(w, id, out);
        out.write_i32(0);
    },
    deserialize: |w, id, r| {
        (BASE_LIGHT.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        Ok(())
    },
    ..BASE_LIGHT
};

// ---- Torch ----

fn stop_meer_mage(w: &mut World, id: ItemId, holder: Option<EntityId>) {
    if let Some(EntityId::Mobile(m)) = holder
        && burning(w, id)
        && let Some(f) = w
            .resources
            .get::<LightHooks>()
            .and_then(|h| h.meer_mage_stop)
    {
        f(w, m, true);
    }
}

/// `Torch.Ignite()`: `base.Ignite()`, then the holder's `MeerMage` effect stops.
pub fn torch_ignite(w: &mut World, id: ItemId) {
    ignite_equipable(w, id);
    let parent = w.item(id).and_then(|i| i.parent);
    stop_meer_mage(w, id, parent);
}

/// `Torch.OnAdded(parent)`.
pub fn torch_on_added(w: &mut World, id: ItemId, parent: EntityId) {
    equipable_on_added(w, id, parent);
    stop_meer_mage(w, id, Some(parent));
}
