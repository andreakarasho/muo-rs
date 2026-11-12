//! What the field items (`FireFieldItem`, `PoisonField`, `ParalyzeField`) share: the caster, the end time, the timer
//! that shows the item once it is known to fit and in line of sight, and the generated serializer (version 1:
//! `[damage,] caster, anchored end`).

use muo_io::{GenericReader, GenericWriter, Result};
use muo_primitives::{DateTime, Point3D, TimeSpan};
use muo_server::World;
use muo_server::ext::Ext;
use muo_server::ids::{ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType, base, deps, effect_item, ops};
use muo_server::light_type::LightType;
use muo_server::map::MapId;
use muo_server::serialization_ext::{ReadEntityExt, WriteEntityExt};
use muo_server::timer::{self, TimerHandle};

use muo_content::spells::helper;

/// What a field item adds to an `Item`.
#[derive(Clone, Debug, Default)]
pub struct FieldData {
    pub damage: i32,
    pub caster: Option<MobileId>,
    pub end: DateTime,
    pub timer: TimerHandle,
}

/// The parts of a field class that differ.
pub struct FieldClass {
    pub ty: &'static ItemType,
    /// The timer's interval.
    pub interval: TimeSpan,
    /// The effect shown when the field becomes visible.
    pub effect: i32,
    /// Whether the item serializes a damage value (`FireFieldItem`).
    pub has_damage: bool,
    /// The tick of a visible, unexpired field.
    pub on_tick: fn(&mut World, ItemId, MobileId),
}

pub fn data(w: &World, id: ItemId) -> Option<&FieldData> {
    w.item(id)?.ext.get::<FieldData>()
}

pub fn data_mut(w: &mut World, id: ItemId) -> Option<&mut FieldData> {
    w.item_mut(id)?.ext.get_mut::<FieldData>()
}

/// The constructor common to the fields: `AdjustField`, invisible, `Light`, `MoveToWorld`, the caster, the end and
/// the timer (`InLOS` is read after the move).
#[allow(clippy::too_many_arguments)]
pub fn new_field(
    w: &mut World,
    class: &'static FieldClass,
    item_id: i32,
    mut loc: Point3D,
    caster: MobileId,
    map: Option<MapId>,
    duration: TimeSpan,
    val: i32,
    damage: i32,
) -> ItemId {
    let id = ops::new_item_with_id(w, class.ty, item_id);
    let can_fit = helper::adjust_field(w, &mut loc, map, 12, false);
    ops::set_visible(w, id, false);
    ops::set_movable(w, id, false);
    ops::set_light(w, id, LightType::Circle300);
    ops::move_to_world_map(w, id, loc, map);
    let end = w.now + duration;
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(FieldData {
            damage,
            caster: Some(caster),
            end,
            timer: TimerHandle::default(),
        });
    }
    let in_los = deps::in_los_item(w, caster, id);
    start_timer(
        w,
        class,
        id,
        TimeSpan::from_secs_f64(f64::from(val.abs()) * 0.2),
        in_los,
        can_fit,
    );
    id
}

fn start_timer(
    w: &mut World,
    class: &'static FieldClass,
    id: ItemId,
    delay: TimeSpan,
    in_los: bool,
    can_fit: bool,
) {
    let h = timer::start(w, delay, class.interval, 0, move |w| {
        tick(w, class, id, in_los, can_fit)
    });
    if let Some(d) = data_mut(w, id) {
        d.timer = h;
    }
}

fn stop_timer(w: &mut World, id: ItemId) {
    if let Some(d) = data_mut(w, id) {
        let mut t = d.timer;
        d.timer = TimerHandle::default();
        t.cancel(w);
    }
}

/// `InternalTimer.OnTick`.
fn tick(w: &mut World, class: &'static FieldClass, id: ItemId, in_los: bool, can_fit: bool) {
    if ops::is_deleted(w, id) {
        return;
    }
    let visible = w.item(id).is_some_and(|i| i.visible());
    if !visible {
        if in_los && can_fit {
            ops::set_visible(w, id, true);
        } else {
            base::delete(w, id);
        }

        if !ops::is_deleted(w, id) {
            base::process_delta(w, id);
            let (loc, map) = w.item(id).map(|i| (i.location, i.map)).unwrap_or_default();
            let e = effect_item::create_targeted(w, loc, map, effect_item::DEFAULT_DURATION);
            muo_server::effects::send_location_particles(w, e, 0x376A, 9, 10, class.effect);
        }
    } else if w.now > data(w, id).map_or(DateTime::MIN, |d| d.end) {
        base::delete(w, id);
        stop_timer(w, id);
    } else {
        let caster = data(w, id).and_then(|d| d.caster);
        let map = w.item(id).and_then(|i| i.map);
        let (Some(_), Some(caster)) = (map, caster) else {
            return;
        };
        (class.on_tick)(w, id, caster);
    }
}

/// `OnAfterDelete`: `_timer?.Stop()`.
pub fn on_after_delete(w: &mut World, id: ItemId) {
    base::on_after_delete_base(w, id);
    stop_timer(w, id);
}

pub fn serialize(w: &World, id: ItemId, out: &mut dyn GenericWriter, has_damage: bool) {
    (ITEM_BASE.serialize)(w, id, out);
    out.write_i32(1); // SerializationVersion
    let d = data(w, id).cloned().unwrap_or_default();
    if has_damage {
        out.write_i32(d.damage);
    }
    out.write_mobile(w, d.caster);
    out.write_anchored_time_ticks(d.end.ticks());
}

/// Reads the item and starts the timer of `AfterDeserialization` (`new InternalTimer(this, Zero, true, true)`).
pub fn deserialize(
    w: &mut World,
    id: ItemId,
    r: &mut dyn GenericReader,
    class: &'static FieldClass,
) -> Result<()> {
    (ITEM_BASE.deserialize)(w, id, r)?;
    let version = r.read_i32()?;
    let now = w.now;
    let damage = if class.has_damage { r.read_i32()? } else { 0 };
    let caster = r.read_mobile(w)?;
    let end = if version == 0 {
        r.read_delta_time_ticks(now.ticks())?
    } else {
        r.read_anchored_time_ticks()?
    };
    let end = DateTime::try_from_ticks(end).unwrap_or(DateTime::MIN);
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(FieldData {
            damage,
            caster,
            end,
            timer: TimerHandle::default(),
        });
    }
    start_timer(w, class, id, TimeSpan::ZERO, true, true);
    Ok(())
}
