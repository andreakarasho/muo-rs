//! Targeting (`Target`, `LandTarget`, `StaticTarget`, `MultiTarget`, `TargetCancelType`, `TargetFlags`) and
//! `SimpleTarget`.
//!
//! A `Target` subclass is a [`Target`] value with plain data plus [`TargetHandlers`] (the overridden virtuals as
//! closures; an unset handler is the base behavior). It is stored in `World::targets` from [`add_target`] until
//! `Invoke`, `Cancel` or `Timeout` ends it; `Mobile::target` holds its id. `SimpleStateTarget<T>` is a closure
//! that captured its state.
//!
//! shortcut: the four `CanTarget(from, X, ref loc, ref map)` virtuals are not overridable (no content overrides
//! them); add a handler when one does. Handlers are `Rc` so a call can run while the target stays in the world
//! (a callback may read it through its id); they hold no entity references.

use std::rc::Rc;

use bitflags::bitflags;
use muo_primitives::{Point3D, TimeSpan};

use crate::ids::{EntityId, ItemId, MobileId};
use crate::map::MapId;
use crate::map::tiles::{root_parent, world_location};
use crate::mobile::AccessLevel;
use crate::net_state::NetStateId;
use crate::timer::{self, TimerHandle};
use crate::world::World;

/// A `Target` reference: its `TargetID`.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct TargetId(pub u32);

bitflags! {
    /// `TargetFlags`.
    #[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash)]
    pub struct TargetFlags: u8 {
        const NONE = 0x00;
        const HARMFUL = 0x01;
        const BENEFICIAL = 0x02;
    }
}

/// `TargetCancelType`.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum TargetCancelType {
    Overridden,
    Canceled,
    Disconnected,
    Timeout,
}

/// `LandTarget`. The caller (the target packet handler) fills `location.z` with `map.GetAverageZ` and `tile_id`
/// with `map.Tiles.GetLandTile(x, y).ID & TileData.MaxLandValue`.
// shortcut: `Name`/`Flags` (TileData.LandTable[tile_id]) are not stored; no script reads them and the table is indexed by `tile_id` when needed, so nothing differs.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct LandTarget {
    pub location: Point3D,
    pub tile_id: i32,
}

/// `StaticTarget`.
// shortcut: `Name`/`Flags` (TileData.ItemTable[item_id]) are not stored; no script reads them and the table is indexed by `item_id` when needed, so nothing differs.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct StaticTarget {
    pub location: Point3D,
    pub item_id: i32,
    pub hue: i32,
}

impl StaticTarget {
    /// The constructor: `item_id` is already `& TileData.MaxItemValue`, `calc_height` is
    /// `TileData.ItemTable[item_id].CalcHeight`, added to `z`.
    pub fn new(location: Point3D, item_id: i32, calc_height: i32, hue: i32) -> StaticTarget {
        let mut location = location;
        location.z += calc_height;
        StaticTarget {
            location,
            item_id,
            hue,
        }
    }
}

/// The `object targeted` a target hands to `CheckTarget`/`OnTarget`.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Targeted {
    Entity(EntityId),
    Land(LandTarget),
    Static(StaticTarget),
}

/// `MultiTarget`'s extra state: the multi to preview and where.
#[derive(Copy, Clone, Debug)]
pub struct MultiTarget {
    pub multi_id: i32,
    pub offset: Point3D,
}

pub type TargetHandler = Rc<dyn Fn(&mut World, MobileId, Targeted)>;
pub type TargetCancelHandler = Rc<dyn Fn(&mut World, MobileId, TargetCancelType)>;
pub type TargetFinishHandler = Rc<dyn Fn(&mut World, MobileId)>;

/// The overridden virtuals. `None` is the base method.
#[derive(Clone, Default)]
pub struct TargetHandlers {
    pub on_target: Option<TargetHandler>,
    /// Default: message 500447.
    pub on_target_not_accessible: Option<TargetHandler>,
    /// Default: message 500447.
    pub on_target_in_secure_trade: Option<TargetHandler>,
    /// Default: message 500447.
    pub on_nonlocal_target: Option<TargetHandler>,
    /// Default: message 500237.
    pub on_cant_see_target: Option<TargetHandler>,
    /// Default: message 500237.
    pub on_target_out_of_los: Option<TargetHandler>,
    /// Default: message 500446.
    pub on_target_out_of_range: Option<TargetHandler>,
    pub on_target_deleted: Option<TargetHandler>,
    /// Default: message 500447.
    pub on_target_untargetable: Option<TargetHandler>,
    pub on_target_cancel: Option<TargetCancelHandler>,
    pub on_target_finish: Option<TargetFinishHandler>,
}

/// `Target` (and `MultiTarget` when `multi` is set).
pub struct Target {
    pub id: TargetId,
    pub timeout_time: i64,
    pub timeout_timer: TimerHandle,
    pub check_los: bool,
    pub disallow_multis: bool,
    pub allow_nonlocal: bool,
    pub range: i32,
    pub allow_ground: bool,
    pub flags: TargetFlags,
    pub multi: Option<MultiTarget>,
    /// `ISpellTarget<T>.Spell`: the spell a `SpellTarget<T>` belongs to.
    pub spell: Option<crate::interfaces::SpellId>,
    /// `[PlayerVendorTarget]`: the target class may be used on the goods of a player vendor.
    pub player_vendor_target: bool,
    /// `SpellTarget<T>`'s `CanTarget` overrides: `base.CanTarget(..) && typeof(T).IsAssignableFrom(..)` for statics,
    /// mobiles and items (the base checks and their messages run first). Land targets are not filtered.
    pub accepts: Option<fn(&Targeted) -> bool>,
    pub handlers: TargetHandlers,
}

/// The statics `Target.m_NextTargetID`, `BaseMenu._nextSerial`, `Prompt.m_Serials`, `HuePicker._nextSerial`.
#[derive(Debug, Default)]
pub struct UiSerials {
    pub target: i32,
    pub menu: i32,
    pub prompt: i32,
    pub hue_picker: u32,
}

impl Target {
    /// `new Target(range, allowGround, flags)`: takes the next TargetID. Add it with [`add_target`].
    pub fn new(w: &mut World, range: i32, allow_ground: bool, flags: TargetFlags) -> Target {
        w.ui_serials.target = w.ui_serials.target.wrapping_add(1);
        Target {
            id: TargetId(w.ui_serials.target as u32),
            timeout_time: 0,
            timeout_timer: TimerHandle::default(),
            check_los: true,
            disallow_multis: false,
            allow_nonlocal: false,
            range,
            allow_ground,
            flags,
            multi: None,
            spell: None,
            player_vendor_target: false,
            accepts: None,
            handlers: TargetHandlers::default(),
        }
    }

    /// `new MultiTarget(multiID, offset, range, allowGround, flags)`; the usual values are range 10, ground allowed.
    pub fn new_multi(
        w: &mut World,
        multi_id: i32,
        offset: Point3D,
        range: i32,
        allow_ground: bool,
        flags: TargetFlags,
    ) -> Target {
        let mut t = Target::new(w, range, allow_ground, flags);
        t.multi = Some(MultiTarget { multi_id, offset });
        t
    }

    /// `new SimpleTarget(range, flags, allowGround, callback)`.
    pub fn simple(
        w: &mut World,
        range: i32,
        flags: TargetFlags,
        allow_ground: bool,
        callback: impl Fn(&mut World, MobileId, Targeted) + 'static,
    ) -> Target {
        let mut t = Target::new(w, range, allow_ground, flags);
        t.handlers.on_target = Some(Rc::new(callback));
        t
    }
}

/// Stores a target in the world; it lives until it is invoked, canceled or timed out.
pub fn add_target(w: &mut World, t: Target) -> TargetId {
    let id = t.id;
    w.targets.insert(id, t);
    id
}

/// `Mobile.BeginTarget(range, allowGround, flags, callback)`.
pub fn begin_target(
    w: &mut World,
    m: MobileId,
    range: i32,
    allow_ground: bool,
    flags: TargetFlags,
    callback: impl Fn(&mut World, MobileId, Targeted) + 'static,
) -> TargetId {
    let t = Target::simple(w, range, flags, allow_ground, callback);
    let id = add_target(w, t);
    set_mobile_target(w, m, Some(id));
    id
}

/// `Mobile.ClearTarget()`.
pub fn clear_target(w: &mut World, m: MobileId) {
    if let Some(m) = w.mobile_mut(m) {
        m.target = None;
    }
}

/// `Mobile.Target = value`.
pub fn set_mobile_target(w: &mut World, m: MobileId, new: Option<TargetId>) {
    let Some(mob) = w.mobile(m) else { return };
    let old = mob.target;
    if old == new {
        return;
    }
    let ty = mob.ty;
    if let Some(mob) = w.mobile_mut(m) {
        mob.target = None;
    }
    if let (Some(old), Some(_)) = (old, new) {
        cancel(w, old, m, TargetCancelType::Overridden);
    }
    let Some(mob) = w.mobile_mut(m) else { return };
    mob.target = new;
    let (locked, ns) = (mob.target_locked, mob.net_state);
    if let (Some(n), false) = (new, locked) {
        send_target_to(w, n, ns);
    }
    (ty.on_target_change)(w, m);
}

/// `Target.SendTargetTo(ns)`: the multi target request for a `MultiTarget`, else the plain one.
pub fn send_target_to(w: &mut World, t: TargetId, ns: Option<NetStateId>) {
    let (Some(ns), Some(target)) = (ns, w.targets.get(&t)) else {
        return;
    };
    let multi = target
        .multi
        .map(|m| (m, target.allow_ground, target.id, target.flags));
    let plain = (target.allow_ground, target.id, target.flags);
    match multi {
        Some((m, g, id, f)) => crate::packets::target::send_multi_target_req(w, ns, g, id, f, m),
        None => crate::packets::target::send_target_req(w, ns, plain.0, plain.1, plain.2),
    }
}

/// `Target.Cancel(Mobile m)`.
pub fn cancel_mobile_target(w: &mut World, m: MobileId) {
    let Some(mob) = w.mobile(m) else { return };
    let (ns, target) = (mob.net_state, mob.target);
    if let Some(ns) = ns {
        crate::packets::target::send_cancel_target(w, ns);
    }
    if let Some(t) = target {
        cancel(w, t, m, TargetCancelType::Canceled);
    }
}

/// `Target.BeginTimeout(from, delay)`: `delay` in milliseconds.
pub fn begin_timeout(w: &mut World, t: TargetId, from: MobileId, delay: i64) {
    let tick = w.tick_count;
    let Some(target) = w.targets.get_mut(&t) else {
        return;
    };
    target.timeout_time = tick + delay;
    let mut old = std::mem::take(&mut target.timeout_timer);
    old.cancel(w);
    let h = timer::start(
        w,
        TimeSpan::from_millis(delay),
        TimeSpan::ZERO,
        1,
        move |w| {
            if w.mobile(from).is_some_and(|m| m.target == Some(t)) {
                timeout(w, t, from);
            }
        },
    );
    if let Some(target) = w.targets.get_mut(&t) {
        target.timeout_timer = h;
    }
}

/// `Target.CancelTimeout()`.
pub fn cancel_timeout(w: &mut World, t: TargetId) {
    let Some(target) = w.targets.get_mut(&t) else {
        return;
    };
    let mut h = std::mem::take(&mut target.timeout_timer);
    h.cancel(w);
}

/// `Target.Timeout(m)`.
pub fn timeout(w: &mut World, t: TargetId, m: MobileId) {
    if let Some(ns) = w.mobile(m).and_then(|x| x.net_state) {
        crate::packets::target::send_cancel_target(w, ns);
    }
    cancel_timeout(w, t);
    clear_target(w, m);
    call_cancel(w, t, m, TargetCancelType::Timeout);
    call_finish(w, t, m);
    w.targets.shift_remove(&t);
}

/// `Target.Cancel(from, type)`.
pub fn cancel(w: &mut World, t: TargetId, from: MobileId, kind: TargetCancelType) {
    cancel_timeout(w, t);
    clear_target(w, from);
    call_cancel(w, t, from, kind);
    call_finish(w, t, from);
    w.targets.shift_remove(&t);
}

fn call_cancel(w: &mut World, t: TargetId, from: MobileId, kind: TargetCancelType) {
    if let Some(f) = w
        .targets
        .get(&t)
        .and_then(|x| x.handlers.on_target_cancel.clone())
    {
        f(w, from, kind);
    }
}

fn call_finish(w: &mut World, t: TargetId, from: MobileId) {
    if let Some(f) = w
        .targets
        .get(&t)
        .and_then(|x| x.handlers.on_target_finish.clone())
    {
        f(w, from);
    }
}

macro_rules! dispatch {
    ($name:ident, $field:ident, $default:expr) => {
        fn $name(w: &mut World, t: TargetId, from: MobileId, targeted: Targeted) {
            match w.targets.get(&t).and_then(|x| x.handlers.$field.clone()) {
                Some(f) => f(w, from, targeted),
                None => {
                    if let Some(n) = $default {
                        send_localized_message(w, from, n);
                    }
                }
            }
        }
    };
}

dispatch!(on_target, on_target, None::<i32>);
dispatch!(
    on_target_not_accessible,
    on_target_not_accessible,
    Some(500447)
);
dispatch!(
    on_target_in_secure_trade,
    on_target_in_secure_trade,
    Some(500447)
);
dispatch!(on_nonlocal_target, on_nonlocal_target, Some(500447));
dispatch!(on_cant_see_target, on_cant_see_target, Some(500237));
dispatch!(on_target_out_of_los, on_target_out_of_los, Some(500237));
dispatch!(on_target_out_of_range, on_target_out_of_range, Some(500446));
dispatch!(on_target_deleted, on_target_deleted, None::<i32>);
dispatch!(on_target_untargetable, on_target_untargetable, Some(500447));

/// `from.SendLocalizedMessage(number)`.
fn send_localized_message(w: &mut World, from: MobileId, number: i32) {
    crate::mobile::part2::messages::send_localized_message(w, from, number, "", 0x3B2);
}

/// `from.InLOS(targeted)` (`Mobile.InLOS(object)`; a land or static target is a point).
fn in_los(w: &mut World, from: MobileId, targeted: &Targeted) -> bool {
    use crate::map::los::LosObject;
    match targeted {
        Targeted::Entity(EntityId::Item(i)) => {
            crate::mobile::part1::combat::in_los_object(w, from, LosObject::Item(*i))
        }
        Targeted::Entity(EntityId::Mobile(m)) => {
            crate::mobile::part1::combat::in_los_object(w, from, LosObject::Mobile(*m))
        }
        Targeted::Land(l) => crate::mobile::part1::combat::in_los_point(w, from, l.location),
        Targeted::Static(s) => crate::mobile::part1::combat::in_los_point(w, from, s.location),
    }
}

/// `item.InSecureTrade`.
fn in_secure_trade(w: &World, item: ItemId) -> bool {
    crate::item::ops::get_secure_trade_cont(w, item).is_some()
}

/// `from.Region.OnTarget(from, target, targeted)`.
fn region_on_target(w: &mut World, from: MobileId, t: TargetId, targeted: &Targeted) -> bool {
    let r = crate::mobile::part1::props_b::region(w, from);
    match w.region(r).map(|r| r.ty) {
        Some(ty) => (ty.on_target)(w, r, from, t, targeted),
        None => true,
    }
}

/// `from.CanSee(object)`: land and static targets are always visible.
fn can_see(w: &mut World, from: MobileId, targeted: &Targeted) -> bool {
    match targeted {
        Targeted::Entity(e) => crate::mobile::base::can_see_object(w, from, Some(*e)),
        _ => true,
    }
}

/// The checks of the four `CanTarget` overloads: `Some((loc, map))` when valid.
fn can_target(
    w: &mut World,
    t: TargetId,
    from: MobileId,
    targeted: Targeted,
) -> Option<(Point3D, Option<MapId>)> {
    let from_map = w.mobile(from).and_then(|m| m.map);
    let (allow_ground, allow_nonlocal, accepts) = {
        let x = &w.targets[&t];
        (x.allow_ground, x.allow_nonlocal, x.accepts)
    };
    let type_ok = |t: &Targeted| accepts.is_none_or(|f| f(t));
    match targeted {
        // A misbehaving client or macro when ground is not allowed.
        Targeted::Land(l) => allow_ground.then_some((l.location, from_map)),
        Targeted::Static(s) => type_ok(&targeted).then_some((s.location, from_map)),
        Targeted::Entity(EntityId::Item(i)) => {
            let Some(ity) = w.item(i).filter(|x| !x.deleted()).map(|x| x.ty) else {
                on_target_deleted(w, t, from, targeted);
                return None;
            };
            if !(ity.can_target)(w, i) {
                on_target_untargetable(w, t, from, targeted);
                return None;
            }
            if !allow_nonlocal
                && matches!(root_parent(w, i), Some(EntityId::Mobile(r)) if r != from)
                && w.mobile(from)
                    .is_some_and(|m| m.access_level == AccessLevel::Player)
            {
                on_nonlocal_target(w, t, from, targeted);
                return None;
            }
            type_ok(&targeted).then(|| {
                (
                    world_location(w, i).unwrap_or_default(),
                    w.item(i).and_then(|x| x.map),
                )
            })
        }
        Targeted::Entity(EntityId::Mobile(m)) => {
            let Some(mty) = w.mobile(m).filter(|x| !x.deleted).map(|x| x.ty) else {
                on_target_deleted(w, t, from, targeted);
                return None;
            };
            if !(mty.can_target)(w, m) {
                on_target_untargetable(w, t, from, targeted);
                return None;
            }
            w.mobile(m)
                .filter(|_| type_ok(&targeted))
                .map(|x| (x.location, x.map))
        }
    }
}

/// `Target.Invoke(from, targeted)`.
pub fn invoke(w: &mut World, t: TargetId, from: MobileId, targeted: Targeted) {
    if !w.targets.contains_key(&t) {
        return;
    }
    cancel_timeout(w, t);
    clear_target(w, from);

    if w.mobile(from).is_none_or(|m| m.deleted) {
        call_cancel(w, t, from, TargetCancelType::Canceled);
        call_finish(w, t, from);
        w.targets.shift_remove(&t);
        return;
    }

    let (range, check_los) = (w.targets[&t].range, w.targets[&t].check_los);
    let from_map = w.mobile(from).and_then(|m| m.map);
    let item = match targeted {
        Targeted::Entity(EntityId::Item(i)) => Some(i),
        _ => None,
    };
    let mobile = match targeted {
        Targeted::Entity(EntityId::Mobile(m)) => Some(m),
        _ => None,
    };

    // When invalid, the CanTarget overload already reported it (every Targeted is a valid type).
    if let Some((loc, map)) = can_target(w, t, from, targeted) {
        if map.is_none()
            || map != from_map
            || (range >= 0 && !crate::mobile::base::in_range_point3d(w, from, loc, range))
        {
            on_target_out_of_range(w, t, from, targeted);
        } else if !can_see(w, from, &targeted) {
            on_cant_see_target(w, t, from, targeted);
        } else if check_los && !in_los(w, from, &targeted) {
            on_target_out_of_los(w, t, from, targeted);
        } else if item.is_some_and(|i| in_secure_trade(w, i)) {
            on_target_in_secure_trade(w, t, from, targeted);
        } else if item.is_some_and(|i| !item_accessible(w, i, from)) {
            on_target_not_accessible(w, t, from, targeted);
        } else if item.is_some_and(|i| !item_check_target(w, i, from, t, targeted))
            || mobile.is_some_and(|m| !mobile_check_target(w, m, from, t, targeted))
        {
            on_target_untargetable(w, t, from, targeted);
        } else if region_on_target(w, from, t, &targeted) {
            on_target(w, t, from, targeted);
        }
    }

    call_finish(w, t, from);
    w.targets.shift_remove(&t);
}

/// `Invoke(from, null)`: the target packet named a serial nobody has (`World.FindMobile`/`FindItem` gave null), so
/// the `_ => isValidTargetType = false` arm reports `OnTargetUntargetable` and the target ends. The handler gets an
/// item id that resolves to nothing.
pub fn invoke_null(w: &mut World, t: TargetId, from: MobileId) {
    if !w.targets.contains_key(&t) {
        return;
    }
    cancel_timeout(w, t);
    clear_target(w, from);
    if w.mobile(from).is_none_or(|m| m.deleted) {
        call_cancel(w, t, from, TargetCancelType::Canceled);
    } else {
        let none = Targeted::Entity(EntityId::Item(ItemId(muo_primitives::Serial::new(0))));
        on_target_untargetable(w, t, from, none);
    }
    call_finish(w, t, from);
    w.targets.shift_remove(&t);
}

fn item_accessible(w: &mut World, i: ItemId, from: MobileId) -> bool {
    match w.item(i).map(|x| x.ty) {
        Some(ty) => (ty.is_accessible_to)(w, i, from),
        None => true,
    }
}

fn item_check_target(
    w: &mut World,
    i: ItemId,
    from: MobileId,
    t: TargetId,
    targeted: Targeted,
) -> bool {
    match w.item(i).map(|x| x.ty) {
        Some(ty) => (ty.check_target)(w, i, from, t, targeted),
        None => true,
    }
}

fn mobile_check_target(
    w: &mut World,
    m: MobileId,
    from: MobileId,
    t: TargetId,
    targeted: Targeted,
) -> bool {
    match w.mobile(m).map(|x| x.ty) {
        Some(ty) => (ty.check_target)(w, m, from, t, targeted),
        None => true,
    }
}

/// `TargetCallback(Mobile from, object targeted)`.
pub type TargetCallback = Rc<dyn Fn(&mut World, MobileId, Targeted)>;

/// `SimpleTarget`.
#[derive(Clone)]
pub struct SimpleTarget {
    pub range: i32,
    pub flags: TargetFlags,
    pub allow_ground: bool,
    callback: Option<TargetCallback>,
}

impl SimpleTarget {
    pub fn new(
        range: i32,
        flags: TargetFlags,
        allow_ground: bool,
        callback: Option<TargetCallback>,
    ) -> Self {
        SimpleTarget {
            range,
            flags,
            allow_ground,
            callback,
        }
    }

    /// `OnTarget`.
    pub fn on_target(&self, w: &mut World, from: MobileId, targeted: Targeted) {
        if let Some(cb) = &self.callback {
            cb(w, from, targeted);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use super::*;
    use crate::net_state::{NetOut, NetStateData};

    thread_local! { static LOG: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) }; }

    fn log(s: &str) {
        LOG.with(|l| l.borrow_mut().push(s.to_string()));
    }
    fn take_log() -> Vec<String> {
        LOG.with(|l| std::mem::take(&mut *l.borrow_mut()))
    }

    fn logging(w: &mut World) -> Target {
        let mut t = Target::new(w, 10, true, TargetFlags::BENEFICIAL);
        t.handlers.on_target = Some(Rc::new(|_, _, _| log("target")));
        t.handlers.on_target_cancel = Some(Rc::new(|_, _, k| log(&format!("cancel {k:?}"))));
        t.handlers.on_target_finish = Some(Rc::new(|_, _| log("finish")));
        t.handlers.on_target_out_of_range = Some(Rc::new(|_, _, _| log("range")));
        t
    }

    fn mobile_with_ns(w: &mut World) -> (MobileId, NetStateId) {
        let m = w.new_mobile(&crate::mobile::mobile_type::MOBILE_BASE);
        let ns = NetStateId(1);
        w.net_states.insert(ns, NetStateData::default());
        let mob = w.mobile_mut(m).unwrap();
        mob.net_state = Some(ns);
        mob.map = Some(MapId(0));
        (m, ns)
    }

    fn land(x: i32) -> Targeted {
        Targeted::Land(LandTarget {
            location: Point3D { x, y: 0, z: 0 },
            tile_id: 3,
        })
    }

    #[test]
    fn ids_are_sequential_from_one() {
        let mut w = World::new();
        assert_eq!(
            Target::new(&mut w, 1, false, TargetFlags::NONE).id,
            TargetId(1)
        );
        assert_eq!(
            Target::new(&mut w, 1, false, TargetFlags::NONE).id,
            TargetId(2)
        );
    }

    #[test]
    fn invoke_land_target_runs_on_target_then_finish() {
        let mut w = World::new();
        let (m, _) = mobile_with_ns(&mut w);
        take_log();
        let t = logging(&mut w);
        let id = add_target(&mut w, t);
        set_mobile_target(&mut w, m, Some(id));
        invoke(&mut w, id, m, land(1));
        assert_eq!(take_log(), ["target", "finish"]);
        assert_eq!(w.mobile(m).unwrap().target, None);
        assert!(w.targets.is_empty());
    }

    #[test]
    fn out_of_range_skips_on_target() {
        let mut w = World::new();
        let (m, _) = mobile_with_ns(&mut w);
        take_log();
        let t = logging(&mut w);
        let id = add_target(&mut w, t);
        invoke(&mut w, id, m, land(50));
        assert_eq!(take_log(), ["range", "finish"]);
    }

    #[test]
    fn ground_is_refused_when_not_allowed() {
        let mut w = World::new();
        let (m, _) = mobile_with_ns(&mut w);
        take_log();
        let mut t = logging(&mut w);
        t.allow_ground = false;
        let id = add_target(&mut w, t);
        invoke(&mut w, id, m, land(1));
        assert_eq!(take_log(), ["finish"]);
    }

    #[test]
    fn replacing_a_target_cancels_the_old_one_as_overridden() {
        let mut w = World::new();
        let (m, ns) = mobile_with_ns(&mut w);
        take_log();
        let a = logging(&mut w);
        let a = add_target(&mut w, a);
        set_mobile_target(&mut w, m, Some(a));
        let b = Target::new(&mut w, 1, false, TargetFlags::NONE);
        let b = add_target(&mut w, b);
        set_mobile_target(&mut w, m, Some(b));
        assert_eq!(take_log(), ["cancel Overridden", "finish"]);
        assert_eq!(w.mobile(m).unwrap().target, Some(b));
        assert!(!w.targets.contains_key(&a));
        let sent = w
            .net_out
            .iter()
            .filter(|o| matches!(o, NetOut::Send(n, b) if *n == ns && b[0] == 0x6C))
            .count();
        assert_eq!(sent, 2);
    }

    #[test]
    fn timeout_cancels_and_finishes() {
        let mut w = World::new();
        let (m, _) = mobile_with_ns(&mut w);
        take_log();
        let t = logging(&mut w);
        let id = add_target(&mut w, t);
        set_mobile_target(&mut w, m, Some(id));
        begin_timeout(&mut w, id, m, 1000);
        assert_eq!(w.targets[&id].timeout_time, 1000);
        assert!(w.targets[&id].timeout_timer.running(&w));
        timeout(&mut w, id, m);
        assert_eq!(take_log(), ["cancel Timeout", "finish"]);
        assert_eq!(w.mobile(m).unwrap().target, None);
    }

    #[test]
    fn unknown_serial_ends_the_target_untargetable() {
        // IncomingTargetingPackets: World.FindMobile gave null -> Invoke(from, null) -> OnTargetUntargetable, finish.
        let mut w = World::new();
        let (m, _) = mobile_with_ns(&mut w);
        take_log();
        let mut t = logging(&mut w);
        t.handlers.on_target_untargetable = Some(Rc::new(|_, _, _| log("untargetable")));
        let id = add_target(&mut w, t);
        set_mobile_target(&mut w, m, Some(id));
        invoke_null(&mut w, id, m);
        assert_eq!(take_log(), ["untargetable", "finish"]);
        assert_eq!(w.mobile(m).unwrap().target, None);
        assert!(w.targets.is_empty());
    }

    #[test]
    fn wrong_kind_ends_silently_before_range() {
        // SpellTarget<Mobile>.CanTarget(StaticTarget) is false: no range message, no OnTarget, finish only.
        let mut w = World::new();
        let (m, _) = mobile_with_ns(&mut w);
        take_log();
        let mut t = logging(&mut w);
        t.accepts = Some(|t| matches!(t, Targeted::Entity(EntityId::Mobile(_))));
        let id = add_target(&mut w, t);
        let far = Targeted::Static(StaticTarget::new(Point3D { x: 50, y: 0, z: 0 }, 1, 0, 0));
        invoke(&mut w, id, m, far);
        assert_eq!(take_log(), ["finish"]);
    }

    #[test]
    fn static_target_adds_calc_height() {
        let s = StaticTarget::new(Point3D { x: 1, y: 2, z: 3 }, 0x100, 4, 0);
        assert_eq!(s.location.z, 7);
    }
}
