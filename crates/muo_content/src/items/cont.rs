//! The container classes: BaseContainer, Backpack, Pouch, Bag, .., LockableContainer, TrappableContainer
//! (+ ExecuteTrap), BaseTreasureChest and TreasureChest.
//!
//! `Server.Items.Container` itself is `muo_server::item::container`; here are the classes above it. State is
//! [`ContExt`] in `Item::ext` for the trappable/lockable/treasure classes (the plain bags keep `Ext::NONE`).
//! What a class overrides that is code and is not an `ItemType` virtual (`Open`, `Locked`, `ExecuteTrap`,
//! `SetLockLevel`, `GenerateTreasure`) is in [`ContClass`], inherited with `..PARENT_CLASS`.
//!
//! `LockableContainer.OnCraft` is `craft::on_craft`, `SetSecureLevelEntry` the `secure_level_entry` hook and the
//! dye tub's `IDyable` the `muo_items` registrations. The other classes of Items/Containers are in `muo_items`
//! (FurnitureContainer, MarkContainer, ParagonChest, Strongbox, TreasureMapChest, FillableContainer).

use muo_io::{GenericReader, GenericWriter, IoError, Result};
use muo_primitives::{Point3D, TimeSpan, utility};
use muo_server::World;
use muo_server::ext::Ext;
use muo_server::ids::{ItemId, MobileId};
use muo_server::item::container::{self, CONTAINER};
use muo_server::item::deps;
use muo_server::item::{ItemType, effect_item, ops};
use muo_server::layer::Layer;
use muo_server::lr_reason::LRReason;
use muo_server::mobile::AccessLevel;
use muo_server::packets::message::MessageType;
use muo_server::timer::{self, TimerHandle};
use muo_server::type_registry::TypeRegistry;

use super::common::hooks;
use crate::core;
use crate::hooks::{AosDamage, hooks as content_hooks};

// ---- state ----

int_enum! {
    /// `TrapType`.
    pub enum TrapType { None = 0, MagicTrap = 1, ExplosionTrap = 2, DartTrap = 3, PoisonTrap = 4 }
}

int_enum! {
    /// `BaseTreasureChest.TreasureLevel`.
    pub enum TreasureLevel { Level1 = 0, Level2 = 1, Level3 = 2, Level4 = 3, Level5 = 4, Level6 = 5 }
}

/// What `TrappableContainer`, `LockableContainer` and `BaseTreasureChest` add to a `Container`.
#[derive(Clone, Debug)]
pub struct ContExt {
    pub trap_level: i32,
    pub trap_power: i32,
    pub trap_type: TrapType,
    // LockableContainer
    pub is_shipwrecked_item: bool,
    pub trap_on_lockpick: bool,
    pub required_skill: i32,
    pub max_lock_level: i32,
    pub key_value: u32,
    pub lock_level: i32,
    pub locked: bool,
    /// `Picker` (not serialized).
    pub picker: Option<MobileId>,
    // BaseTreasureChest
    pub level: TreasureLevel,
    pub min_spawn_time: TimeSpan,
    pub max_spawn_time: TimeSpan,
    /// `_resetTimer` (not serialized).
    pub reset_timer: TimerHandle,
}

impl Default for ContExt {
    fn default() -> Self {
        ContExt {
            trap_level: 0,
            trap_power: 0,
            trap_type: TrapType::None,
            is_shipwrecked_item: false,
            trap_on_lockpick: false,
            required_skill: 0,
            max_lock_level: 0,
            key_value: 0,
            lock_level: 0,
            locked: false,
            picker: None,
            level: TreasureLevel::Level1,
            min_spawn_time: TimeSpan::ZERO,
            max_spawn_time: TimeSpan::ZERO,
            reset_timer: TimerHandle::default(),
        }
    }
}

pub fn ext(w: &World, id: ItemId) -> Option<&ContExt> {
    w.item(id)?.ext.get::<ContExt>()
}

pub fn ext_mut(w: &mut World, id: ItemId) -> Option<&mut ContExt> {
    w.item_mut(id)?.ext.get_mut::<ContExt>()
}

fn read<T>(w: &World, id: ItemId, default: T, f: impl FnOnce(&ContExt) -> T) -> T {
    ext(w, id).map_or(default, f)
}

/// The virtuals of these classes that are not `Item` virtuals.
#[derive(Copy, Clone)]
pub struct ContClass {
    /// `Open(from)`.
    pub open: fn(&mut World, ItemId, MobileId),
    /// `TrapOnOpen`.
    pub trap_on_open: fn(&World, ItemId) -> bool,
    /// `ExecuteTrap(from)`.
    pub execute_trap: fn(&mut World, ItemId, MobileId) -> bool,
    /// `OnTelekinesis(from)`.
    pub on_telekinesis: fn(&mut World, ItemId, MobileId),
    /// `Locked` and its setter.
    pub locked: fn(&World, ItemId) -> bool,
    pub set_locked: fn(&mut World, ItemId, bool),
    /// `CheckLocked(from)`.
    pub check_locked: fn(&mut World, ItemId, MobileId) -> bool,
    /// `LockPick(from)`.
    pub lock_pick: fn(&mut World, ItemId, MobileId),
    /// `SetLockLevel()` and `GenerateTreasure()` of a treasure chest.
    pub set_lock_level: fn(&mut World, ItemId),
    pub generate_treasure: fn(&mut World, ItemId),
}

pub static BASE_CONTAINER_CLASS: ContClass = ContClass {
    open: open_base,
    trap_on_open: |_, _| true,
    execute_trap: |_, _, _| false,
    on_telekinesis: |_, _, _| {},
    locked: |_, _| false,
    set_locked: |_, _, _| {},
    check_locked: |_, _, _| false,
    lock_pick: |_, _, _| {},
    set_lock_level: |_, _| {},
    generate_treasure: |_, _| {},
};

/// The `ContClass` of an item (BaseContainer's when its type has none).
pub fn class(w: &World, id: ItemId) -> &'static ContClass {
    w.item(id)
        .and_then(|i| i.ty.family::<ContClass>())
        .unwrap_or(&BASE_CONTAINER_CLASS)
}

// ---- BaseContainer ----

fn is_secure(w: &World, id: ItemId) -> bool {
    ops::is_secure(w, id)
}

fn default_max_weight(w: &World, id: ItemId) -> i32 {
    if is_secure(w, id) {
        0
    } else {
        container::default_max_weight_default(w, id)
    }
}

fn display_to(w: &mut World, id: ItemId, to: MobileId) {
    let staff_ok = w
        .mobile(to)
        .is_some_and(|m| m.access_level >= AccessLevel::Administrator);
    // `to.AccessLevel < FeatureFlagSettings.RequiredAccessLevel`
    if !staff_ok
        && let Some(f) = hooks(w).and_then(|h| h.container_access_blocked)
        && let Some(ty) = w.item(id).map(|i| i.ty)
        && let Some(reason) = f(w, ty.full_name)
    {
        deps::send_message_hue(w, to, 0x22, &reason);
        return;
    }
    container::display_to_default(w, id, to);
}

fn is_accessible_to(w: &mut World, id: ItemId, m: MobileId) -> bool {
    let house = hooks(w)
        .and_then(|h| h.house_check_accessible)
        .is_none_or(|f| f(w, m, id));
    house && muo_server::item::base::is_accessible_to_base(w, id, m)
}

#[allow(clippy::too_many_arguments)]
fn check_hold(
    w: &mut World,
    id: ItemId,
    m: Option<MobileId>,
    item: ItemId,
    message: bool,
    check_items: bool,
    plus_items: i32,
    plus_weight: i32,
) -> bool {
    if is_secure(w, id)
        && let Some(f) = hooks(w).and_then(|h| h.house_check_hold)
        && !f(
            w,
            m,
            id,
            item,
            message,
            check_items,
            plus_items,
            plus_weight,
        )
    {
        return false;
    }
    container::check_hold_default(
        w,
        id,
        m,
        item,
        message,
        check_items,
        plus_items,
        plus_weight,
    )
}

fn check_item_use(w: &mut World, id: ItemId, from: MobileId, item: ItemId) -> bool {
    if (container::is_deco_container_default(w, id)
        || (w.item(id).map(|i| i.ty.is_deco_container)).is_some_and(|f| f(w, id)))
        && w.item(item)
            .is_some_and(|i| super::is_named(i.ty, "Server.Items.BaseBook"))
    {
        return true;
    }
    muo_server::item::base::check_item_use_base(w, id, from, item)
}

fn house_drop_ok(w: &mut World, id: ItemId, from: MobileId, dropped: ItemId) -> bool {
    hooks(w)
        .and_then(|h| h.house_drop_check)
        .is_none_or(|f| f(w, id, from, dropped))
}

/// `BaseContainer.TryDropItem(from, dropped, sendFullMessage)`.
fn try_drop_item(
    w: &mut World,
    id: ItemId,
    from: Option<MobileId>,
    dropped: ItemId,
    send_full_message: bool,
    play_sound: bool,
) -> bool {
    // The 4-argument overload (playSound) is Container's own; only the 3-argument one is overridden.
    if play_sound {
        return container::try_drop_item_default(
            w,
            id,
            from,
            dropped,
            send_full_message,
            play_sound,
        );
    }
    let Some(ty) = w.item(id).map(|i| i.ty) else {
        return false;
    };
    if !(ty.check_hold)(w, id, from, dropped, send_full_message, true, 0, 0) {
        return false;
    }
    // `from` is expected on this path (the house check needs it).
    if let Some(f) = from
        && !house_drop_ok(w, id, f, dropped)
    {
        return false;
    }
    let list = w.item(id).map(|i| i.items.clone()).unwrap_or_default();
    for item in list {
        let Some(it) = w.item(item).map(|i| i.ty) else {
            continue;
        };
        if !it.is(&CONTAINER) && (it.stack_with)(w, item, from, dropped, false) {
            return true;
        }
    }
    (ty.drop_item)(w, id, dropped);
    true
}

fn on_drag_drop_into(w: &mut World, id: ItemId, from: MobileId, item: ItemId, p: Point3D) -> bool {
    if !container::check_hold_items(w, id, Some(from), item, true, true) {
        return false;
    }
    if !house_drop_ok(w, id, from, item) {
        return false;
    }
    ops::set_location(w, item, Point3D::new(p.x, p.y, 0));
    if let Some(ty) = w.item(id).map(|i| i.ty) {
        (ty.add_item)(w, id, item);
    }
    let sound = container::get_dropped_sound(w, id, item);
    let loc = ops::get_world_location(w, id);
    deps::send_sound(w, from, sound, loc);
    true
}

fn update_total(
    w: &mut World,
    id: ItemId,
    sender: ItemId,
    ty: muo_server::item::TotalType,
    delta: i32,
) {
    (CONTAINER.update_total)(w, id, sender, ty, delta);
    if ty == muo_server::item::TotalType::Weight
        && let Some(muo_server::ids::EntityId::Mobile(m)) = ops::root_parent(w, id)
    {
        // `(RootParent as Mobile)?.InvalidateProperties()`
        {
            muo_server::mobile::part2::properties::invalidate_properties(w, m);
        }
    }
}

fn on_double_click(w: &mut World, id: ItemId, from: MobileId) {
    let staff = w
        .mobile(from)
        .is_some_and(|m| m.access_level > AccessLevel::Player);
    let loc = ops::get_world_location(w, id);
    let in_range = deps::mobile_in_range(w, from, loc, 2);
    let vendor = hooks(w)
        .and_then(|h| h.root_is_player_vendor)
        .is_some_and(|f| f(w, id));
    if staff || in_range || vendor {
        (class(w, id).open)(w, id, from);
    } else {
        deps::local_overhead_message(w, from, MessageType::REGULAR, 0x3B2, 1019045); // I can't reach that.
    }
}

/// `BaseContainer.Open`: `DisplayTo(from)`.
fn open_base(w: &mut World, id: ItemId, from: MobileId) {
    if let Some(ty) = w.item(id).map(|i| i.ty) {
        (ty.display_to)(w, id, from);
    }
}

/// `Open(from)` through the class.
pub fn open(w: &mut World, id: ItemId, from: MobileId) {
    (class(w, id).open)(w, id, from);
}

/// `item is BaseContainer`.
pub fn is_base_container(w: &World, id: ItemId) -> bool {
    w.item(id).is_some_and(|i| i.ty.is(&BASE_CONTAINER))
}

/// `GetContextMenuEntries`: `SetSecureLevelEntry.AddTo(from, this, ref list)` is the house system's entry.
fn get_context_menu_entries(
    w: &mut World,
    id: ItemId,
    from: MobileId,
    list: &mut Vec<muo_server::context_menu_entry::ContextMenuEntry>,
) {
    (CONTAINER.get_context_menu_entries)(w, id, from, list);
    if let Some(e) = hooks(w)
        .and_then(|h| h.secure_level_entry)
        .and_then(|f| f(w, from, id))
    {
        list.push(e);
    }
}

/// `Server.Items.BaseContainer`.
pub static BASE_CONTAINER: ItemType = ItemType {
    full_name: "Server.Items.BaseContainer",
    parent: Some(&CONTAINER),
    construct: None,
    family: Some(&BASE_CONTAINER_CLASS),
    default_max_weight,
    display_to,
    is_accessible_to,
    check_hold,
    check_item_use,
    try_drop_item,
    on_drag_drop_into,
    update_total,
    on_double_click,
    get_context_menu_entries,
    ..CONTAINER
};

// ---- constructors ----

/// `new BaseContainer(itemID)` of class `ty`.
pub fn new_base_container(w: &mut World, ty: &'static ItemType, item_id: i32) -> ItemId {
    container::new_container(w, ty, item_id)
}

fn new_with_ext(w: &mut World, ty: &'static ItemType, item_id: i32) -> ItemId {
    let id = new_base_container(w, ty, item_id);
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(ContExt::default());
    }
    id
}

/// `new LockableContainer(itemID)`: `MaxLockLevel = 100`.
pub fn new_lockable(w: &mut World, ty: &'static ItemType, item_id: i32) -> ItemId {
    let id = new_with_ext(w, ty, item_id);
    if let Some(d) = ext_mut(w, id) {
        d.max_lock_level = 100;
    }
    id
}

/// `new TrappableContainer(itemID)`.
pub fn new_trappable(w: &mut World, ty: &'static ItemType, item_id: i32) -> ItemId {
    new_with_ext(w, ty, item_id)
}

// ---- TrappableContainer ----

fn send_message_to_number(w: &mut World, id: ItemId, to: MobileId, number: i32, hue: i32) {
    if ops::is_deleted(w, id) {
        return;
    }
    let can = w
        .mobile(to)
        .map(|m| m.ty)
        .is_some_and(|m| (m.can_see_item)(w, to, id));
    if !can {
        return;
    }
    let Some(ns) = deps::net_state_of(w, to) else {
        return;
    };
    let item_id = w.item(id).map_or(0, |i| i.item_id);
    muo_server::packets::message::send_message_localized(
        w,
        ns,
        id.0,
        item_id,
        MessageType::REGULAR,
        hue,
        3,
        number,
        "",
        "",
    );
}

fn send_message_to_text(w: &mut World, id: ItemId, to: MobileId, text: &str, hue: i32) {
    if ops::is_deleted(w, id) {
        return;
    }
    let can = w
        .mobile(to)
        .map(|m| m.ty)
        .is_some_and(|m| (m.can_see_item)(w, to, id));
    if !can {
        return;
    }
    let Some(ns) = deps::net_state_of(w, to) else {
        return;
    };
    let item_id = w.item(id).map_or(0, |i| i.item_id);
    muo_server::packets::message::send_message(
        w,
        ns,
        id.0,
        item_id,
        MessageType::REGULAR,
        hue,
        3,
        false,
        Some("ENU"),
        "",
        text,
    );
}

fn effect_loc(
    w: &mut World,
    p: Point3D,
    map: Option<muo_server::map::MapId>,
    item: i32,
    dur: i32,
    speed: i32,
) {
    muo_server::effects::send_location_effect(w, p, map, item, dur, speed, 0, 0);
}

fn aos_damage(w: &mut World, to: MobileId, d: AosDamage) {
    match content_hooks(w).and_then(|h| h.aos_damage) {
        Some(f) => {
            f(w, to, None, d); // AOS.Damage(m, damage, ..) has no attacker
        }
        // shortcut: unset only before `creature::misc` registers `aos_damage` (every running server sets it).
        None => muo_server::mobile::base::damage(w, to, d.amount, None, true, false),
    }
}

/// `TrappableContainer.ExecuteTrap`.
fn execute_trap(w: &mut World, id: ItemId, from: MobileId) -> bool {
    let Some(d) = ext(w, id).cloned() else {
        return false;
    };
    if d.trap_type == TrapType::None {
        return false;
    }
    if w.mobile(from)
        .is_some_and(|m| m.access_level >= AccessLevel::GameMaster)
    {
        send_message_to_text(
            w,
            id,
            from,
            "That is trapped, but you open it with your godly powers.",
            0x3B2,
        );
        return false;
    }
    send_message_to_number(w, id, from, 502999, 0x3B2); // You set off a trap!
    let loc = ops::get_world_location(w, id);
    let map = w.item(id).and_then(|i| i.map);
    match d.trap_type {
        TrapType::ExplosionTrap => {
            if deps::mobile_in_range(w, from, loc, 3) {
                let damage = if d.trap_level > 0 {
                    super::random_min_max(w, 10, 30) * d.trap_level
                } else {
                    d.trap_power
                };
                aos_damage(
                    w,
                    from,
                    AosDamage {
                        amount: damage,
                        fire: 100,
                        ..Default::default()
                    },
                );
                deps::local_overhead_message(w, from, MessageType::REGULAR, 0x2A, 503000);
            }
            effect_loc(w, loc, map, 0x36BD, 15, 10);
            muo_server::effects::play_sound(w, loc, map, 0x307);
        }
        TrapType::MagicTrap => {
            if deps::mobile_in_range(w, from, loc, 1) {
                muo_server::mobile::base::damage(w, from, d.trap_power, None, true, false);
            }
            muo_server::effects::play_sound(w, loc, map, 0x307);
            for (dx, dy, dz) in [(-1, 0, 0), (1, 0, 0), (0, -1, 0), (0, 1, 0), (1, 1, 11)] {
                effect_loc(
                    w,
                    Point3D::new(loc.x + dx, loc.y + dy, loc.z + dz),
                    map,
                    0x36BD,
                    15,
                    10,
                );
            }
        }
        TrapType::DartTrap => {
            if deps::mobile_in_range(w, from, loc, 3) {
                let damage = if d.trap_level > 0 {
                    super::random_min_max(w, 5, 15) * d.trap_level
                } else {
                    d.trap_power
                };
                aos_damage(
                    w,
                    from,
                    AosDamage {
                        amount: damage,
                        phys: 100,
                        ..Default::default()
                    },
                );
                deps::local_overhead_message(w, from, MessageType::REGULAR, 0x62, 502998);
            }
            muo_server::effects::play_sound(w, loc, map, 0x223);
        }
        TrapType::PoisonTrap => {
            if deps::mobile_in_range(w, from, loc, 3) {
                let level = if d.trap_level > 0 {
                    Some((d.trap_level - 1).clamp(0, 4))
                } else {
                    aos_damage(
                        w,
                        from,
                        AosDamage {
                            amount: d.trap_power,
                            pois: 100,
                            ..Default::default()
                        },
                    );
                    None // Poison.Greater
                };
                if let Some(p) = poison_for(w, level) {
                    muo_server::mobile::base::apply_poison(w, from, Some(from), p);
                }
                deps::local_overhead_message(w, from, MessageType::REGULAR, 0x44, 503004);
            }
            effect_loc(w, loc, map, 0x113A, 10, 20);
            muo_server::effects::play_sound(w, loc, map, 0x231);
        }
        TrapType::None => {}
    }
    if let Some(d) = ext_mut(w, id) {
        d.trap_type = TrapType::None;
        d.trap_power = 0;
        d.trap_level = 0;
    }
    true
}

/// `Poison.GetPoison(level)`, or `Poison.Greater` for `None`.
fn poison_for(w: &World, level: Option<i32>) -> Option<muo_server::poison::PoisonId> {
    let p = w.resources.get::<muo_server::poison::Poisons>()?;
    match level {
        Some(l) => p.get_poison_by_index(l).map(|x| x.id()),
        None => p.get_poison("Greater").map(|x| x.id()),
    }
}

fn trappable_open(w: &mut World, id: ItemId, from: MobileId) {
    let cls = class(w, id);
    if !(cls.trap_on_open)(w, id) || !(cls.execute_trap)(w, id, from) {
        open_base(w, id, from);
    }
}

fn on_telekinesis_trappable(w: &mut World, id: ItemId, from: MobileId) {
    let loc = muo_server::item::base::location(w, id);
    let map = w.item(id).and_then(|i| i.map);
    let fx_item = effect_item::create(w, loc, map, effect_item::DEFAULT_DURATION);
    muo_server::effects::send_location_particles(w, fx_item, 0x376A, 9, 32, 5022);
    muo_server::effects::play_sound(w, loc, map, 0x1F5);
    let cls = class(w, id);
    if (cls.trap_on_open)(w, id) {
        (cls.execute_trap)(w, id, from);
    }
}

pub static TRAPPABLE_CLASS: ContClass = ContClass {
    open: trappable_open,
    execute_trap,
    on_telekinesis: on_telekinesis_trappable,
    ..BASE_CONTAINER_CLASS
};

fn write_trappable(w: &World, id: ItemId, out: &mut dyn GenericWriter) {
    let d = ext(w, id).cloned().unwrap_or_default();
    out.write_i32(3);
    out.write_i32(d.trap_level);
    out.write_i32(d.trap_power);
    out.write_enum(4, d.trap_type as i64);
}

fn read_trappable(w: &mut World, id: ItemId, r: &mut dyn GenericReader) -> Result<()> {
    // Versions below 3 have the same three fields (`Deserialize(reader, version)`).
    let _version = r.read_i32()?;
    let trap_level = r.read_i32()?;
    let trap_power = r.read_i32()?;
    let t = r.read_enum(4)? as i32;
    let trap_type = TrapType::from_i32(t)
        .ok_or_else(|| IoError::InvalidData(format!("TrapType: no value {t}")))?;
    if let Some(d) = ext_mut(w, id) {
        d.trap_level = trap_level;
        d.trap_power = trap_power;
        d.trap_type = trap_type;
    }
    Ok(())
}

/// `Server.Items.TrappableContainer`.
pub static TRAPPABLE_CONTAINER: ItemType = ItemType {
    full_name: "Server.Items.TrappableContainer",
    parent: Some(&BASE_CONTAINER),
    family: Some(&TRAPPABLE_CLASS),
    serialize: |w, id, out| {
        (BASE_CONTAINER.serialize)(w, id, out);
        write_trappable(w, id, out);
    },
    deserialize: |w, id, r| {
        (BASE_CONTAINER.deserialize)(w, id, r)?;
        // The loader builds entities with no state; every class below reads into this.
        if let Some(i) = w.item_mut(id)
            && i.ext.get::<ContExt>().is_none()
        {
            i.ext.insert(ContExt::default());
        }
        read_trappable(w, id, r)
    },
    ..BASE_CONTAINER
};

// ---- LockableContainer ----

fn lockable_locked(w: &World, id: ItemId) -> bool {
    read(w, id, false, |d| d.locked)
}

fn lockable_set_locked(w: &mut World, id: ItemId, value: bool) {
    if let Some(d) = ext_mut(w, id) {
        d.locked = value;
        if value {
            d.picker = None;
        }
    }
    ops::invalidate_properties(w, id);
}

/// `Locked`.
pub fn locked(w: &World, id: ItemId) -> bool {
    (class(w, id).locked)(w, id)
}

/// `Locked = value` through the class.
pub fn set_locked(w: &mut World, id: ItemId, value: bool) {
    (class(w, id).set_locked)(w, id, value);
}

/// `LockPick(from)` through the class.
pub fn lock_pick(w: &mut World, id: ItemId, from: MobileId) {
    (class(w, id).lock_pick)(w, id, from);
}

fn lockable_lock_pick(w: &mut World, id: ItemId, from: MobileId) {
    set_locked(w, id, false);
    if let Some(d) = ext_mut(w, id) {
        d.picker = Some(from);
    }
    if read(w, id, false, |d| d.trap_on_lockpick)
        && (class(w, id).execute_trap)(w, id, from)
        && let Some(d) = ext_mut(w, id)
    {
        d.trap_on_lockpick = false;
    }
}

fn lockable_check_locked(w: &mut World, id: ItemId, from: MobileId) -> bool {
    if !locked(w, id) {
        return false;
    }
    let inaccessible = w
        .mobile(from)
        .is_none_or(|m| m.access_level < AccessLevel::GameMaster);
    let number = if inaccessible {
        501747 // It appears to be locked.
    } else {
        502502 // That is locked, but you open it with your godly powers.
    };
    if let Some(ns) = deps::net_state_of(w, from) {
        let item_id = w.item(id).map_or(0, |i| i.item_id);
        muo_server::packets::message::send_message_localized(
            w,
            ns,
            id.0,
            item_id,
            MessageType::REGULAR,
            0x3B2,
            3,
            number,
            "",
            "",
        );
    }
    inaccessible
}

fn lockable_open(w: &mut World, id: ItemId, from: MobileId) {
    if (class(w, id).check_locked)(w, id, from) {
        return;
    }
    trappable_open(w, id, from);
}

fn lockable_on_telekinesis(w: &mut World, id: ItemId, from: MobileId) {
    if (class(w, id).check_locked)(w, id, from) {
        let loc = muo_server::item::base::location(w, id);
        let map = w.item(id).and_then(|i| i.map);
        let fx_item = effect_item::create(w, loc, map, effect_item::DEFAULT_DURATION);
        muo_server::effects::send_location_particles(w, fx_item, 0x376A, 9, 32, 5022);
        muo_server::effects::play_sound(w, loc, map, 0x1F5);
        return;
    }
    on_telekinesis_trappable(w, id, from);
}

fn is_gm(w: &World, m: MobileId) -> bool {
    w.mobile(m)
        .is_some_and(|m| m.access_level >= AccessLevel::GameMaster)
}

fn lockable_try_drop_item(
    w: &mut World,
    id: ItemId,
    from: Option<MobileId>,
    dropped: ItemId,
    send_full_message: bool,
    play_sound: bool,
) -> bool {
    if !play_sound
        && let Some(f) = from
        && !is_gm(w, f)
        && locked(w, id)
    {
        deps::send_localized_message(w, f, 501747); // It appears to be locked.
        return false;
    }
    try_drop_item(w, id, from, dropped, send_full_message, play_sound)
}

fn lockable_on_drag_drop_into(
    w: &mut World,
    id: ItemId,
    from: MobileId,
    item: ItemId,
    p: Point3D,
) -> bool {
    if !is_gm(w, from) && locked(w, id) {
        deps::send_localized_message(w, from, 501747);
        return false;
    }
    on_drag_drop_into(w, id, from, item, p)
}

fn lockable_check_lift(
    w: &mut World,
    id: ItemId,
    from: MobileId,
    item: ItemId,
    reject: &mut LRReason,
) -> bool {
    CONTAINER_CHECK_LIFT(w, id, from, item, reject)
        && (item == id || is_gm(w, from) || !locked(w, id))
}

#[allow(non_snake_case)]
fn CONTAINER_CHECK_LIFT(
    w: &mut World,
    id: ItemId,
    from: MobileId,
    item: ItemId,
    r: &mut LRReason,
) -> bool {
    (CONTAINER.check_lift)(w, id, from, item, r)
}

fn lockable_check_item_use(w: &mut World, id: ItemId, from: MobileId, item: ItemId) -> bool {
    if !check_item_use(w, id, from, item) {
        return false;
    }
    if item != id && !is_gm(w, from) && locked(w, id) {
        deps::local_overhead_message(w, from, MessageType::REGULAR, 0x3B2, 1019045); // I can't reach that.
        return false;
    }
    true
}

fn lockable_check_content_display(w: &World, id: ItemId, from: MobileId) -> bool {
    !locked(w, id) && container::check_content_display_default(w, id, from)
}

fn lockable_on_double_click_secure_trade(w: &mut World, id: ItemId, from: MobileId) {
    if (class(w, id).check_locked)(w, id, from) {
        return;
    }
    (CONTAINER.on_double_click_secure_trade)(w, id, from);
}

fn lockable_on_snoop(w: &mut World, id: ItemId, from: MobileId) {
    if (class(w, id).check_locked)(w, id, from) {
        return;
    }
    (CONTAINER.on_snoop)(w, id, from);
}

fn lockable_add_name_properties(
    w: &World,
    id: ItemId,
    list: &mut muo_server::property_list::PropertyList,
) {
    muo_server::item::base::add_name_properties_base(w, id, list);
    if read(w, id, false, |d| d.is_shipwrecked_item) {
        list.add(1041645); // recovered from a shipwreck
    }
}

fn lockable_on_single_click(w: &mut World, id: ItemId, from: MobileId) {
    (CONTAINER.on_single_click)(w, id, from);
    if read(w, id, false, |d| d.is_shipwrecked_item) {
        ops::label_to_number(w, id, from, 1041645, ""); // recovered from a shipwreck
    }
}

pub static LOCKABLE_CLASS: ContClass = ContClass {
    trap_on_open: |w, id| !read(w, id, false, |d| d.trap_on_lockpick),
    open: lockable_open,
    on_telekinesis: lockable_on_telekinesis,
    locked: lockable_locked,
    set_locked: lockable_set_locked,
    check_locked: lockable_check_locked,
    lock_pick: lockable_lock_pick,
    ..TRAPPABLE_CLASS
};

/// `Server.Items.LockableContainer`.
pub static LOCKABLE_CONTAINER: ItemType = ItemType {
    full_name: "Server.Items.LockableContainer",
    parent: Some(&TRAPPABLE_CONTAINER),
    family: Some(&LOCKABLE_CLASS),
    displays_content: |w, id| !lockable_locked(w, id),
    check_content_display: lockable_check_content_display,
    try_drop_item: lockable_try_drop_item,
    on_drag_drop_into: lockable_on_drag_drop_into,
    check_lift: lockable_check_lift,
    check_item_use: lockable_check_item_use,
    on_double_click_secure_trade: lockable_on_double_click_secure_trade,
    on_snoop: lockable_on_snoop,
    add_name_properties: lockable_add_name_properties,
    on_single_click: lockable_on_single_click,
    serialize: |w, id, out| {
        (TRAPPABLE_CONTAINER.serialize)(w, id, out);
        let d = ext(w, id).cloned().unwrap_or_default();
        out.write_i32(0);
        out.write_bool(d.is_shipwrecked_item);
        out.write_bool(d.trap_on_lockpick);
        out.write_i32(d.required_skill);
        out.write_i32(d.max_lock_level);
        out.write_u32(d.key_value);
        out.write_i32(d.lock_level);
        out.write_bool(d.locked);
    },
    deserialize: |w, id, r| {
        (TRAPPABLE_CONTAINER.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        let shipwrecked = r.read_bool()?;
        let trap_on_lockpick = r.read_bool()?;
        let required_skill = r.read_i32()?;
        let max_lock_level = r.read_i32()?;
        let key_value = r.read_u32()?;
        let lock_level = r.read_i32()?;
        let locked = r.read_bool()?;
        if let Some(d) = ext_mut(w, id) {
            d.is_shipwrecked_item = shipwrecked;
            d.trap_on_lockpick = trap_on_lockpick;
            d.required_skill = required_skill;
            d.max_lock_level = max_lock_level;
            d.key_value = key_value;
            d.lock_level = lock_level;
            d.locked = locked;
        }
        Ok(())
    },
    ..TRAPPABLE_CONTAINER
};

// ---- BaseTreasureChest ----

fn treasure_set_locked(w: &mut World, id: ItemId, value: bool) {
    if lockable_locked(w, id) != value {
        lockable_set_locked(w, id, value);
        if !value {
            start_reset_timer(w, id);
        }
        ops::invalidate_properties(w, id);
    }
}

fn start_reset_timer(w: &mut World, id: ItemId) {
    let Some(d) = ext(w, id).cloned() else { return };
    let mut h = d.reset_timer;
    h.cancel(w);
    let ticks = utility::random_min_max_i64(
        &mut w.rng,
        d.min_spawn_time.ticks(),
        d.max_spawn_time.ticks(),
    );
    let h = timer::start(
        w,
        TimeSpan::from_ticks(ticks),
        TimeSpan::ZERO,
        1,
        move |w| reset(w, id),
    );
    if let Some(d) = ext_mut(w, id) {
        d.reset_timer = h;
    }
}

fn treasure_set_lock_level(w: &mut World, id: ItemId) {
    let lock = match read(w, id, TreasureLevel::Level1, |d| d.level) {
        TreasureLevel::Level1 => 5,
        TreasureLevel::Level2 => 20,
        TreasureLevel::Level3 => 50,
        TreasureLevel::Level4 => 70,
        TreasureLevel::Level5 => 90,
        TreasureLevel::Level6 => 100,
    };
    if let Some(d) = ext_mut(w, id) {
        d.lock_level = lock;
        d.required_skill = lock;
    }
}

fn treasure_generate(w: &mut World, id: ItemId) {
    let (lo, hi) = match read(w, id, TreasureLevel::Level1, |d| d.level) {
        TreasureLevel::Level1 => (100, 300),
        TreasureLevel::Level2 => (300, 600),
        TreasureLevel::Level3 => (600, 900),
        TreasureLevel::Level4 => (900, 1200),
        TreasureLevel::Level5 => (1200, 5000),
        TreasureLevel::Level6 => (5000, 9000),
    };
    let gold = super::random_min_max(w, lo, hi);
    let g = super::gold::new_gold(w, gold);
    if let Some(ty) = w.item(id).map(|i| i.ty) {
        (ty.drop_item)(w, id, g);
    }
}

/// `BaseTreasureChest.ClearContents`.
pub fn clear_contents(w: &mut World, id: ItemId) {
    let mut i = w.item(id).map_or(0, |i| i.items.len());
    while i > 0 {
        i -= 1;
        if let Some(c) = w.item(id).and_then(|c| c.items.get(i).copied())
            && let Some(ty) = w.item(c).map(|i| i.ty)
        {
            (ty.delete)(w, c);
        }
    }
}

/// `BaseTreasureChest.Reset`.
pub fn reset(w: &mut World, id: ItemId) {
    if let Some(d) = ext_mut(w, id) {
        let mut h = d.reset_timer;
        d.reset_timer = TimerHandle::default();
        h.cancel(w);
    }
    set_locked(w, id, true);
    clear_contents(w, id);
    (class(w, id).generate_treasure)(w, id);
}

pub static TREASURE_CLASS: ContClass = ContClass {
    set_locked: treasure_set_locked,
    set_lock_level: treasure_set_lock_level,
    generate_treasure: treasure_generate,
    ..LOCKABLE_CLASS
};

/// `new BaseTreasureChest(itemID, level)` of class `ty`.
pub fn new_treasure_chest(
    w: &mut World,
    ty: &'static ItemType,
    item_id: i32,
    level: TreasureLevel,
) -> ItemId {
    let id = new_lockable(w, ty, item_id);
    if let Some(d) = ext_mut(w, id) {
        d.level = level;
        d.min_spawn_time = TimeSpan::from_mins(10);
        d.max_spawn_time = TimeSpan::from_mins(60);
    }
    set_locked(w, id, true);
    ops::set_movable(w, id, false);
    let cls = class(w, id);
    (cls.set_lock_level)(w, id);
    (cls.generate_treasure)(w, id);
    id
}

/// `Server.Items.BaseTreasureChest`.
pub static BASE_TREASURE_CHEST: ItemType = ItemType {
    full_name: "Server.Items.BaseTreasureChest",
    parent: Some(&LOCKABLE_CONTAINER),
    // `BaseTreasureChest(int itemID)` is not `[Constructible]`.
    construct: None,
    make: Some(|w| new_treasure_chest(w, &BASE_TREASURE_CHEST, 0xE43, TreasureLevel::Level2)),
    family: Some(&TREASURE_CLASS),
    is_deco_container: |_, _| false,
    default_name: |w, id| {
        Some(
            if lockable_locked(w, id) {
                "a locked treasure chest"
            } else {
                "a treasure chest"
            }
            .to_string(),
        )
    },
    serialize: |w, id, out| {
        (LOCKABLE_CONTAINER.serialize)(w, id, out);
        let d = ext(w, id).cloned().unwrap_or_default();
        out.write_i32(1);
        out.write_enum(4, d.level as i64);
        out.write_time_span_ticks(d.min_spawn_time.ticks());
        out.write_time_span_ticks(d.max_spawn_time.ticks());
    },
    deserialize: |w, id, r| {
        (LOCKABLE_CONTAINER.deserialize)(w, id, r)?;
        let version = r.read_i32()?;
        let (level, min, max);
        if version < 1 {
            // Deserialize(reader, version): a byte level, spawn times in minutes as shorts.
            let l = i32::from(r.read_u8()?);
            level = TreasureLevel::from_i32(l)
                .ok_or_else(|| IoError::InvalidData(format!("TreasureLevel: no value {l}")))?;
            min = TimeSpan::from_mins(i64::from(r.read_i16()?));
            max = TimeSpan::from_mins(i64::from(r.read_i16()?));
        } else {
            let l = r.read_enum(4)? as i32;
            level = TreasureLevel::from_i32(l)
                .ok_or_else(|| IoError::InvalidData(format!("TreasureLevel: no value {l}")))?;
            min = TimeSpan::from_ticks(r.read_time_span_ticks()?);
            max = TimeSpan::from_ticks(r.read_time_span_ticks()?);
        }
        if let Some(d) = ext_mut(w, id) {
            d.level = level;
            d.min_spawn_time = min;
            d.max_spawn_time = max;
        }
        // [AfterDeserialization]
        if !locked(w, id) {
            start_reset_timer(w, id);
        }
        Ok(())
    },
    ..LOCKABLE_CONTAINER
};

// ---- leaf classes ----

/// One concrete container class whose save is `parent part + version int 0`.
/// `$ctor` builds it (`new_base_container` / `new_lockable` / `new_trappable`); `@init` is the constructor body.
macro_rules! container_leaf {
    // `[SerializationGenerator(0)]`: the class's own version is an encoded int.
    (
        @enc
        $(#[$m:meta])*
        $name:ident = $full:literal : $parent:ident, graphic $gid:expr, $ctor:ident;
        $($f:ident : $v:expr,)*
    ) => {
        $(#[$m])*
        pub static $name: ItemType = ItemType {
            full_name: $full,
            parent: Some(&$parent),
            construct: Some(|w| $ctor(w, &$name, $gid)),
            serialize: |w, id, out| {
                ($parent.serialize)(w, id, out);
                out.write_encoded_int(0);
            },
            deserialize: |w, id, r| {
                ($parent.deserialize)(w, id, r)?;
                let _version = r.read_encoded_int()?;
                Ok(())
            },
            $($f: $v,)*
            ..$parent
        };
    };
    // The same for a class with no parameterless `[Constructible]`: `make` instead of `construct`.
    (
        @make
        $(#[$m:meta])*
        $name:ident = $full:literal : $parent:ident, graphic $gid:expr, $ctor:ident;
        $(@init $init:expr;)?
        $($f:ident : $v:expr,)*
    ) => {
        $(#[$m])*
        pub static $name: ItemType = ItemType {
            full_name: $full,
            parent: Some(&$parent),
            construct: None,
            make: Some(|w| {
                let id = $ctor(w, &$name, $gid);
                $( let init: fn(&mut World, ItemId) = $init; init(w, id); )?
                id
            }),
            serialize: |w, id, out| {
                ($parent.serialize)(w, id, out);
                out.write_i32(0);
            },
            deserialize: |w, id, r| {
                ($parent.deserialize)(w, id, r)?;
                let _version = r.read_i32()?;
                Ok(())
            },
            $($f: $v,)*
            ..$parent
        };
    };
    (
        $(#[$m:meta])*
        $name:ident = $full:literal : $parent:ident, graphic $gid:expr, $ctor:ident;
        $(@init $init:expr;)?
        $($f:ident : $v:expr,)*
    ) => {
        $(#[$m])*
        pub static $name: ItemType = ItemType {
            full_name: $full,
            parent: Some(&$parent),
            construct: Some(|w| {
                let id = $ctor(w, &$name, $gid);
                $( let init: fn(&mut World, ItemId) = $init; init(w, id); )?
                id
            }),
            serialize: |w, id, out| {
                ($parent.serialize)(w, id, out);
                out.write_i32(0);
            },
            deserialize: |w, id, r| {
                ($parent.deserialize)(w, id, r)?;
                let _version = r.read_i32()?;
                Ok(())
            },
            $($f: $v,)*
            ..$parent
        };
    };
}

fn backpack_default_max_weight(w: &World, id: ItemId) -> i32 {
    // Core.ML && Parent is Mobile m && m.Player && m.Backpack == this
    if core::ml(w)
        && let Some(muo_server::ids::EntityId::Mobile(m)) = w.item(id).and_then(|i| i.parent)
        && let Some(mob) = w.mobile(m)
        && mob.player
        && muo_server::mobile::part2::items::find_item_on_layer(w, m, Layer::Backpack) == Some(id)
    {
        return 550;
    }
    default_max_weight(w, id)
}

container_leaf! {
    /// `Server.Items.Backpack`.
    BACKPACK = "Server.Items.Backpack" : BASE_CONTAINER, graphic 0xE75, new_base_container;
    @init |w, id| { (BACKPACK.set_layer)(w, id, Layer::Backpack); };
    default_weight: |_, _| 3.0,
    default_max_weight: backpack_default_max_weight,
}

fn backpack_construct(w: &mut World, ty: &'static ItemType, id: i32) -> ItemId {
    new_base_container(w, ty, id)
}

container_leaf! {
    /// `Server.Items.StrongBackpack` (pack animals).
    STRONG_BACKPACK = "Server.Items.StrongBackpack" : BACKPACK, graphic 0xE75, backpack_construct;
    @init |w, id| { (STRONG_BACKPACK.set_layer)(w, id, Layer::Backpack); };
    default_weight: |_, _| 13.0,
    default_max_weight: |_, _| 1600,
    check_hold: |w, id, m, item, _message, check_items, plus_items, plus_weight| {
        check_hold(w, id, m, item, false, check_items, plus_items, plus_weight)
    },
    check_content_display: |w, id, from| {
        // RootParent is BaseCreature creature && creature.ControlMaster == from || base
        let master = match ops::root_parent(w, id) {
            Some(muo_server::ids::EntityId::Mobile(m)) => crate::creature::control::control_master(w, m),
            _ => None,
        };
        master == Some(from) || container::check_content_display_default(w, id, from)
    },
}

container_leaf! {
    @make
    /// `Server.Items.CreatureBackpack` (BaseCreature's pack). Its constructor takes the creature's name.
    CREATURE_BACKPACK = "Server.Items.CreatureBackpack" : BACKPACK, graphic 0xE75, backpack_construct;
    @init |w, id| {
        (CREATURE_BACKPACK.set_layer)(w, id, Layer::Backpack);
        (CREATURE_BACKPACK.set_hue)(w, id, 5);
    };
    default_weight: |_, _| 3.0,
    add_name_property: |w, id, list| {
        match w.item(id).and_then(|i| i.name.clone()) {
            Some(n) => list.add_str(1075257, &n), // Contents of ~1_PETNAME~'s pack.
            None => muo_server::item::base::add_name_property_base(w, id, list),
        }
    },
    on_item_removed: |w, id, item| {
        if w.item(id).is_some_and(|i| i.items.is_empty())
            && let Some(ty) = w.item(id).map(|i| i.ty) {
                (ty.delete)(w, id);
            }
        (BASE_CONTAINER.on_item_removed)(w, id, item);
    },
    on_drag_lift: |w, id, from| {
        let _ = id;
        if w.mobile(from).is_some_and(|m| m.access_level > AccessLevel::Player) {
            return true;
        }
        deps::send_localized_message(w, from, 500169); // You cannot pick that up.
        false
    },
    on_drag_drop_into: |_, _, _, _, _| false,
    try_drop_item: |_, _, _, _, _, _| false,
}

/// `new CreatureBackpack(name)`.
pub fn new_creature_backpack(w: &mut World, name: &str) -> ItemId {
    let id = new_base_container(w, &CREATURE_BACKPACK, 0xE75);
    ops::set_name(w, id, Some(name.to_string()));
    (CREATURE_BACKPACK.set_layer)(w, id, Layer::Backpack);
    (CREATURE_BACKPACK.set_hue)(w, id, 5);
    id
}

container_leaf! {
    /// `Server.Items.Pouch`.
    POUCH = "Server.Items.Pouch" : TRAPPABLE_CONTAINER, graphic 0xE79, new_trappable;
    default_weight: |_, _| 1.0,
}

/// `Server.Items.BaseBagBall` (abstract, `[SerializationGenerator(0, false)]`: weight 1).
pub static BASE_BAG_BALL: ItemType = ItemType {
    full_name: "Server.Items.BaseBagBall",
    parent: Some(&BASE_CONTAINER),
    construct: None,
    serialize: |w, id, out| {
        (BASE_CONTAINER.serialize)(w, id, out);
        out.write_i32(0);
    },
    deserialize: |w, id, r| {
        (BASE_CONTAINER.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        Ok(())
    },
    default_weight: |_, _| 1.0,
    ..BASE_CONTAINER
};
container_leaf! {
    /// `Server.Items.SmallBagBall`.
    SMALL_BAG_BALL = "Server.Items.SmallBagBall" : BASE_BAG_BALL, graphic 0x2256, new_base_container;
}
container_leaf! {
    /// `Server.Items.LargeBagBall`.
    LARGE_BAG_BALL = "Server.Items.LargeBagBall" : BASE_BAG_BALL, graphic 0x2257, new_base_container;
}
container_leaf! {
    /// `Server.Items.Bag`.
    BAG = "Server.Items.Bag" : BASE_CONTAINER, graphic 0xE76, new_base_container;
    default_weight: |_, _| 2.0,
}
container_leaf! {
    /// `Server.Items.Barrel`.
    BARREL = "Server.Items.Barrel" : BASE_CONTAINER, graphic 0xE77, new_base_container;
    default_weight: |_, _| 25.0,
}
container_leaf! {
    /// `Server.Items.Keg`.
    KEG = "Server.Items.Keg" : BASE_CONTAINER, graphic 0xE7F, new_base_container;
    default_weight: |_, _| 15.0,
}
container_leaf! {
    /// `Server.Items.PicnicBasket`.
    PICNIC_BASKET = "Server.Items.PicnicBasket" : BASE_CONTAINER, graphic 0xE7A, new_base_container;
    default_weight: |_, _| 2.0,
}
container_leaf! {
    /// `Server.Items.Basket`.
    BASKET = "Server.Items.Basket" : BASE_CONTAINER, graphic 0x990, new_base_container;
    default_weight: |_, _| 1.0,
}

/// The lockable chests and boxes: `NAME = "full" , graphic, weight (None keeps Item's)`.
/// `[Flippable]` is data for the flip command, which is not an `ItemType` member yet (see weapon.rs).
macro_rules! lockable_leaves {
    ($($(#[$m:meta])* $name:ident = $full:literal, $gid:expr $(, weight $weight:expr)?;)+) => {
        $( container_leaf! {
            $(#[$m])*
            $name = $full : LOCKABLE_CONTAINER, graphic $gid, new_lockable;
            $( default_weight: |_, _| $weight, )?
        } )+
    };
}

lockable_leaves! {
    /// `Server.Items.WoodenBox` `[Flippable(0x9AA, 0xE7D)]`.
    WOODEN_BOX = "Server.Items.WoodenBox", 0x9AA, weight 4.0;
    /// `Server.Items.SmallCrate` `[Flippable(0x9A9, 0xE7E)]`.
    SMALL_CRATE = "Server.Items.SmallCrate", 0x9A9, weight 2.0;
    /// `Server.Items.MediumCrate` `[Flippable(0xE3F, 0xE3E)]`.
    MEDIUM_CRATE = "Server.Items.MediumCrate", 0xE3F, weight 2.0;
    /// `Server.Items.LargeCrate` `[Flippable(0xE3D, 0xE3C)]`.
    LARGE_CRATE = "Server.Items.LargeCrate", 0xE3D, weight 1.0;
    /// `Server.Items.MetalBox` `[Flippable(0x9A8, 0xE80)]`.
    METAL_BOX = "Server.Items.MetalBox", 0x9A8;
    /// `Server.Items.MetalChest` `[Flippable(0x9AB, 0xE7C)]`.
    METAL_CHEST = "Server.Items.MetalChest", 0x9AB;
    /// `Server.Items.MetalGoldenChest` `[Flippable(0xE41, 0xE40)]`.
    METAL_GOLDEN_CHEST = "Server.Items.MetalGoldenChest", 0xE41;
    /// `Server.Items.WoodenChest` `[Flippable(0xE43, 0xE42)]`.
    WOODEN_CHEST = "Server.Items.WoodenChest", 0xE43, weight 2.0;
    /// `Server.Items.PlainWoodenChest` `[Flippable(0x280B, 0x280C)]`.
    PLAIN_WOODEN_CHEST = "Server.Items.PlainWoodenChest", 0x280B;
    /// `Server.Items.OrnateWoodenChest` `[Flippable(0x280D, 0x280E)]`.
    ORNATE_WOODEN_CHEST = "Server.Items.OrnateWoodenChest", 0x280D;
    /// `Server.Items.GildedWoodenChest` `[Flippable(0x280F, 0x2810)]`.
    GILDED_WOODEN_CHEST = "Server.Items.GildedWoodenChest", 0x280F;
    /// `Server.Items.FinishedWoodenChest` `[Flippable(0x2813, 0x2814)]`.
    FINISHED_WOODEN_CHEST = "Server.Items.FinishedWoodenChest", 0x2813;
}

container_leaf! {
    @enc
    /// `Server.Items.RarewoodChest` `[Flippable(0x2DF1, 0x2DF2)]` (`[SerializationGenerator(0)]`).
    RAREWOOD_CHEST = "Server.Items.RarewoodChest" : LOCKABLE_CONTAINER, graphic 0x2DF1, new_lockable;
}
container_leaf! {
    @enc
    /// `Server.Items.DecorativeBox` `[Flippable(0x2DF3, 0x2DF4)]`, constructed with graphic 0x2DF4.
    DECORATIVE_BOX = "Server.Items.DecorativeBox" : LOCKABLE_CONTAINER, graphic 0x2DF4, new_lockable;
}

container_leaf! {
    /// `Server.Items.WoodenFootLocker` `[Flippable(0x2811, 0x2812)]`: `GumpID = 0x10B`.
    WOODEN_FOOT_LOCKER = "Server.Items.WoodenFootLocker" : LOCKABLE_CONTAINER, graphic 0x2811, new_lockable;
    @init |w, id| { container::set_gump_id(w, id, 0x10B); };
}

/// The treasure chests (`BaseTreasureChest` at level 2).
macro_rules! treasure_leaves {
    ($($(#[$m:meta])* $name:ident = $full:literal, $gid:expr;)+) => {
        $( $(#[$m])*
        pub static $name: ItemType = ItemType {
            full_name: $full,
            parent: Some(&BASE_TREASURE_CHEST),
            construct: Some(|w| new_treasure_chest(w, &$name, $gid, TreasureLevel::Level2)),
            serialize: |w, id, out| {
                (BASE_TREASURE_CHEST.serialize)(w, id, out);
                out.write_i32(0);
            },
            deserialize: |w, id, r| {
                (BASE_TREASURE_CHEST.deserialize)(w, id, r)?;
                let _version = r.read_i32()?;
                Ok(())
            },
            ..BASE_TREASURE_CHEST
        }; )+
    };
}

treasure_leaves! {
    /// `Server.Items.WoodenTreasureChest` `[Flippable(0xE43, 0xE42)]`.
    WOODEN_TREASURE_CHEST = "Server.Items.WoodenTreasureChest", 0xE43;
    /// `Server.Items.MetalGoldenTreasureChest` `[Flippable(0xE41, 0xE40)]`.
    METAL_GOLDEN_TREASURE_CHEST = "Server.Items.MetalGoldenTreasureChest", 0xE41;
    /// `Server.Items.MetalTreasureChest` `[Flippable(0x9AB, 0xE7C)]`.
    METAL_TREASURE_CHEST = "Server.Items.MetalTreasureChest", 0x9AB;
}

/// Registers the classes of this file.
pub fn register(types: &mut TypeRegistry) {
    // The base classes with `[SerializationGenerator]` that the server itself makes: they are in saves too.
    for ty in [
        &CONTAINER,
        &BASE_TREASURE_CHEST,
        &muo_server::item::containers::BANK_BOX,
        &muo_server::item::secure_trade_container::SECURE_TRADE_CONTAINER,
    ] {
        types.register_item(ty, &[]);
    }
    for ty in [
        &BACKPACK,
        &STRONG_BACKPACK,
        &CREATURE_BACKPACK,
        &POUCH,
        &SMALL_BAG_BALL,
        &LARGE_BAG_BALL,
        &BAG,
        &BARREL,
        &KEG,
        &PICNIC_BASKET,
        &BASKET,
        &WOODEN_BOX,
        &SMALL_CRATE,
        &MEDIUM_CRATE,
        &LARGE_CRATE,
        &METAL_BOX,
        &METAL_CHEST,
        &METAL_GOLDEN_CHEST,
        &WOODEN_CHEST,
        &PLAIN_WOODEN_CHEST,
        &ORNATE_WOODEN_CHEST,
        &GILDED_WOODEN_CHEST,
        &WOODEN_FOOT_LOCKER,
        &FINISHED_WOODEN_CHEST,
        &RAREWOOD_CHEST,
        &DECORATIVE_BOX,
        &WOODEN_TREASURE_CHEST,
        &METAL_GOLDEN_TREASURE_CHEST,
        &METAL_TREASURE_CHEST,
    ] {
        types.register_item(ty, &[]);
    }
}

/// `LockableContainer.OnCraft(..)` (ICraftable): a tinkering check makes the box lockable with a copper key.
pub fn lockable_on_craft(w: &mut World, id: ItemId, a: &super::equip_hooks::CraftArgs) -> i32 {
    use muo_server::skills::SkillName;
    let from = a.from;
    if muo_server::mobile::part2::stats::check_skill(w, from, SkillName::Tinkering, -5.0, 15.0) {
        deps::send_localized_message(w, from, 500636); // Your tinker skill was sufficient to make the item lockable.
        let value = super::gold::random_value(w);
        let key = super::gold::new_key_of(w, super::gold::KeyType::Copper, value, None);
        if let Some(e) = ext_mut(w, id) {
            e.key_value = value;
        }
        if let Some(t) = w.item(id).map(|i| i.ty) {
            (t.drop_item)(w, id, key);
        }
        let tinkering = muo_server::skills::value(w, from, SkillName::Tinkering);
        let level = (tinkering * 0.8) as i32;
        if let Some(e) = ext_mut(w, id) {
            e.required_skill = (level - 4).min(95);
            e.max_lock_level = (level + 35).min(95);
            // Lock level of 0 means it is not pickable, so change it to -1
            e.lock_level = if level == 14 {
                -1
            } else {
                (level - 14).min(95)
            };
        }
    } else {
        deps::send_localized_message(w, from, 500637); // Your tinker skill was insufficient to make the item lockable.
    }
    1
}
