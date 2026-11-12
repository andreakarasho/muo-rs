//! `MarkContainer`.

use muo_content::items::cont::{self, ContClass, LOCKABLE_CLASS, LOCKABLE_CONTAINER, new_lockable};
use muo_content::props::{Ctor, Kind, Param, PropDef, Value, add_ctors};
use muo_content::utilities::types::PropType;
use muo_io::{GenericReader, GenericWriter};
use muo_primitives::{DateTime, Point3D, TimeSpan};
use muo_server::World;
use muo_server::ids::{EntityId, ItemId};
use muo_server::item::{ItemType, ops};
use muo_server::map::MapId;
use muo_server::mobile::AccessLevel;
use muo_server::serialization_ext::{ReadEntityExt, WriteEntityExt};
use muo_server::timer::{self, TimerHandle};
use muo_server::type_registry::{Phase, TypeRegistry};

use crate::simple::hue;
use crate::skill_items::runebook;

/// `ILockpickable.MagicLock`.
const MAGIC_LOCK: i32 = -255;

/// The fields of `MarkContainer`.
#[derive(Clone, Debug, Default)]
pub struct MarkData {
    pub auto_lock: bool,
    pub relock: TimerHandle,
    pub target_map: Option<MapId>,
    pub target: Point3D,
    pub description: Option<String>,
}

fn md(w: &World, id: ItemId) -> MarkData {
    w.item(id)
        .and_then(|i| i.ext.get::<MarkData>())
        .cloned()
        .unwrap_or_default()
}

fn md_mut(w: &mut World, id: ItemId) -> Option<&mut MarkData> {
    w.item_mut(id)?.ext.get_mut::<MarkData>()
}

/// `new InternalTimer(container, delay)`: relocks the container when it fires.
fn start_relock(w: &mut World, id: ItemId, delay: TimeSpan) {
    let h = timer::start(w, delay, TimeSpan::ZERO, 1, move |w| {
        cont::set_locked(w, id, true);
        if let Some(d) = cont::ext_mut(w, id) {
            d.lock_level = MAGIC_LOCK;
        }
    });
    if let Some(d) = md_mut(w, id) {
        d.relock = h;
    }
}

/// `StopTimer()`.
pub fn stop_timer(w: &mut World, id: ItemId) {
    if let Some(d) = md_mut(w, id) {
        let mut h = std::mem::take(&mut d.relock);
        h.cancel(w);
    }
}

fn has_timer(w: &World, id: ItemId) -> bool {
    md(w, id).relock != TimerHandle::default()
}

/// `Locked = value`: the lockable setter, then the auto-relock timer.
fn set_locked(w: &mut World, id: ItemId, value: bool) {
    (LOCKABLE_CLASS.set_locked)(w, id, value);
    if md(w, id).auto_lock {
        stop_timer(w, id);
        if !cont::locked(w, id) {
            start_relock(w, id, TimeSpan::from_mins(5));
        }
    }
}

/// `AutoLock = value` (`OnAutoLockChanged`).
pub fn set_auto_lock(w: &mut World, id: ItemId, value: bool) {
    if md(w, id).auto_lock == value {
        return;
    }
    if let Some(d) = md_mut(w, id) {
        d.auto_lock = value;
    }
    if !value {
        stop_timer(w, id);
    } else if !cont::locked(w, id) && !has_timer(w, id) {
        start_relock(w, id, TimeSpan::from_mins(5));
    }
}

/// `Bone = value`.
fn set_bone(w: &mut World, id: ItemId, bone: bool) {
    crate::simple::item_id(w, id, if bone { 0xECA } else { 0xE79 });
    hue(w, id, if bone { 1102 } else { 0 });
}

/// `MarkContainer(bone, locked)`.
pub fn mark_container_new(w: &mut World, bone: bool, locked: bool) -> ItemId {
    let id = new_lockable(w, &MARK_CONTAINER, if bone { 0xECA } else { 0xE79 });
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(MarkData::default());
    }
    ops::set_movable(w, id, false);
    if bone {
        hue(w, id, 1102);
    }
    if let Some(d) = md_mut(w, id) {
        d.auto_lock = locked;
    }
    cont::set_locked(w, id, locked);
    if locked && let Some(d) = cont::ext_mut(w, id) {
        d.lock_level = MAGIC_LOCK;
    }
    id
}

/// `Mark(rune)`.
pub fn mark(w: &mut World, id: ItemId, rune: ItemId) {
    let d = md(w, id);
    if d.target_map.is_some() {
        runebook::set_marked(w, rune, true);
        runebook::set_target_map(w, rune, d.target_map);
        runebook::set_target(w, rune, d.target);
        runebook::set_rune_description(w, rune, d.description);
        runebook::set_house(w, rune, None);
    }
}

fn is_rune(w: &World, item: ItemId) -> bool {
    muo_content::factory::item_is(w, item, "Server.Items.RecallRune")
}

static MARK_CLASS: ContClass = ContClass {
    set_locked,
    ..LOCKABLE_CLASS
};

fn write(w: &World, id: ItemId, out: &mut dyn GenericWriter) {
    let d = md(w, id);
    out.write_bool(d.auto_lock);
    let next = if d.relock == TimerHandle::default() {
        DateTime::MIN
    } else {
        d.relock.next(w)
    };
    out.write_anchored_time_ticks(next.ticks());
    out.write_map(d.target_map);
    out.write_point3d(d.target);
    out.write_opt_string(d.description.as_deref());
}

fn read(w: &mut World, id: ItemId, r: &mut dyn GenericReader, version: i32) -> muo_io::Result<()> {
    let mut d = MarkData {
        auto_lock: r.read_bool()?,
        ..MarkData::default()
    };
    let ticks = if version == 0 {
        r.read_delta_time_ticks(w.now.ticks())?
    } else {
        r.read_anchored_time_ticks()?
    };
    let next = DateTime::from_ticks(ticks);
    d.target_map = r.read_map()?;
    d.target = r.read_point3d()?;
    d.description = r.read_string()?;
    let auto_lock = d.auto_lock;
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(d);
    }
    // `DeserializeRelockTimer(delay)`: `if (!Locked && _autoLock) _relockTimer = new InternalTimer(this, delay)`
    if next != DateTime::MIN && !cont::locked(w, id) && auto_lock {
        start_relock(w, id, next - w.now);
    }
    Ok(())
}

/// `Server.Items.MarkContainer` (`[SerializationGenerator(1, false)]`).
pub static MARK_CONTAINER: ItemType = ItemType {
    full_name: "Server.Items.MarkContainer",
    parent: Some(&LOCKABLE_CONTAINER),
    construct: Some(|w| mark_container_new(w, false, false)),
    family: Some(&MARK_CLASS),
    is_deco_container: |_, _| false,
    on_drag_drop: |w, id, from, dropped| {
        if is_rune(w, dropped) && (LOCKABLE_CONTAINER.on_drag_drop)(w, id, from, dropped) {
            mark(w, id, dropped);
            return true;
        }
        false
    },
    on_drag_drop_into: |w, id, from, dropped, p| {
        if is_rune(w, dropped) && (LOCKABLE_CONTAINER.on_drag_drop_into)(w, id, from, dropped, p) {
            mark(w, id, dropped);
            return true;
        }
        false
    },
    on_after_delete: |w, id| {
        stop_timer(w, id);
        (LOCKABLE_CONTAINER.on_after_delete)(w, id);
    },
    serialize: |w, id, out| {
        (LOCKABLE_CONTAINER.serialize)(w, id, out);
        out.write_i32(1);
        write(w, id, out);
    },
    deserialize: |w, id, r| {
        (LOCKABLE_CONTAINER.deserialize)(w, id, r)?;
        let version = r.read_i32()?;
        read(w, id, r, version)
    },
    ..LOCKABLE_CONTAINER
};

// ---- `[props`, `[add` ----

macro_rules! mark_prop {
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

static MARK_PROPS: &[PropDef] = &[
    mark_prop!(
        "AutoLock",
        Kind::Plain(PropType::Bool),
        |w: &World, i| Value::Bool(md(w, i).auto_lock),
        |w: &mut World, i, v: Value| set_auto_lock(w, i, v.bool())
    ),
    mark_prop!(
        "Bone",
        Kind::Plain(PropType::Bool),
        |w: &World, i| Value::Bool(w.item(i).is_some_and(|x| x.item_id == 0xECA)),
        |w: &mut World, i, v: Value| set_bone(w, i, v.bool())
    ),
    mark_prop!(
        "TargetMap",
        Kind::Map,
        |w: &World, i| Value::Map(md(w, i).target_map),
        |w: &mut World, i, v: Value| {
            if let Value::Map(m) = v
                && let Some(d) = md_mut(w, i)
            {
                d.target_map = m;
            }
        }
    ),
    mark_prop!(
        "Target",
        Kind::Plain(PropType::IPoint3D),
        |w: &World, i| Value::Point3D(md(w, i).target),
        |w: &mut World, i, v: Value| {
            if let Some(d) = md_mut(w, i) {
                d.target = v.point();
            }
        }
    ),
    mark_prop!(
        "Description",
        Kind::Plain(PropType::Str),
        |w: &World, i| md(w, i).description.map_or(Value::Null, Value::Str),
        |w: &mut World, i, v: Value| {
            if let Some(d) = md_mut(w, i) {
                d.description = v.string();
            }
        }
    ),
];

const BOOL: Kind = Kind::Plain(PropType::Bool);

static MARK_CTORS: [Ctor; 1] = [Ctor {
    access: None,
    params: &[
        Param {
            name: "bone",
            kind: BOOL,
            default: Some(Value::Bool(false)),
            ty: "",
        },
        Param {
            name: "locked",
            kind: BOOL,
            default: Some(Value::Bool(false)),
            ty: "",
        },
    ],
    make: |w, v| EntityId::Item(mark_container_new(w, v[0].bool(), v[1].bool())),
}];

/// `[Decorate`'s `new MarkContainer(bone, locked) { TargetMap = map, Description = "strange location" }`.
pub fn decorate_new(w: &mut World, bone: bool, locked: bool, map: Option<MapId>) -> ItemId {
    let id = mark_container_new(w, bone, locked);
    if let Some(d) = md_mut(w, id) {
        d.target_map = map;
        d.description = Some("strange location".to_string());
    }
    id
}

/// `markCont.Target = Point3D.Parse(extra)` (a bad `extra` is ignored, as the `catch {}`).
pub fn decorate_set_target(w: &mut World, id: ItemId, extra: &str) {
    if let (Some(p), Some(d)) = (Point3D::try_parse(extra), md_mut(w, id)) {
        d.target = p;
    }
}

/// `CreateMalasPassage(x, y, z, xTarget, yTarget, zTarget, bone, locked)`.
fn create_malas_passage(
    w: &mut World,
    at: (i32, i32, i32),
    target: (i32, i32, i32),
    bone: bool,
    locked: bool,
) {
    let location = Point3D::new(at.0, at.1, at.2);
    let exists = muo_server::map::enumerate::items_at(w, Some(MapId::MALAS), at.0, at.1).any(|i| {
        w.item(i)
            .is_some_and(|x| x.ty.is(&MARK_CONTAINER) && x.location.z == at.2)
    });
    if exists {
        return;
    }
    let c = mark_container_new(w, bone, locked);
    if let Some(d) = md_mut(w, c) {
        d.target_map = Some(MapId::MALAS);
        d.target = Point3D::new(target.0, target.1, target.2);
        d.description = Some("strange location".to_string());
    }
    ops::move_to_world_map(w, c, location, Some(MapId::MALAS));
}

fn configure(w: &mut World) {
    use muo_server::commands::{CommandEventArgs, CommandSystem};
    add_ctors(w, "Server.Items.MarkContainer", &MARK_CTORS);
    muo_content::props::add(w, "Server.Items.MarkContainer", MARK_PROPS);
    // Usage: SecretLocGen; Description: Generates mark containers to Malas secret locations.
    CommandSystem::register(
        w,
        "SecretLocGen",
        AccessLevel::Developer,
        &[],
        std::rc::Rc::new(|w: &mut World, e: &mut CommandEventArgs| {
            create_malas_passage(w, (951, 546, -70), (1006, 994, -70), false, false);
            create_malas_passage(w, (914, 192, -79), (1019, 1062, -70), false, false);
            create_malas_passage(w, (1614, 143, -90), (1214, 1313, -90), false, false);
            create_malas_passage(w, (2176, 324, -90), (1554, 172, -90), false, false);
            create_malas_passage(w, (864, 812, -90), (1061, 1161, -70), false, false);
            create_malas_passage(w, (1051, 1434, -85), (1076, 1244, -70), false, true);
            create_malas_passage(w, (1326, 523, -87), (1201, 1554, -70), false, false);
            create_malas_passage(w, (424, 189, -1), (2333, 1501, -90), true, false);
            create_malas_passage(w, (1313, 1115, -85), (1183, 462, -45), false, false);
            muo_server::mobile::part2::messages::send_message(
                w,
                e.mobile,
                "Secret mark containers have been created.",
            );
        }),
    );
}

/// Registers the class.
pub fn register(types: &mut TypeRegistry) {
    types.register_item(&MARK_CONTAINER, &[]);
    types.register_phase_default(
        Phase::Configure,
        "Server.Items.MarkContainer.Configure",
        configure,
    );
}
