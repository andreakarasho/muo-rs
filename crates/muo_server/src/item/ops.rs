//! The non-virtual members of `Item` as free functions over `&mut World` and an id. The virtual members' base
//! behavior is in `base.rs`.
//!
//! Where a virtual property is assigned (`Location = x`, `ItemID = x`..) these call the vtable entry.
//! The statics of `Item` are [`ItemStatics`] in the world's resources.

use std::collections::VecDeque;

use muo_primitives::{DateTime, Direction, Point3D, Rectangle2D, TimeSpan};

use crate::ids::{EntityId, ItemId, MobileId};
use crate::item::container::CONTAINER;
use crate::item::deps;
use crate::item::{
    BounceInfo, DECAY_SLOT_NONE, ExpandFlag, ImplFlag, ItemDelta, LootType, TotalType,
};
use crate::light_type::LightType;
use crate::map::MapId;
use crate::world::World;

pub use crate::item::decay::{register as decay_register, unregister as decay_unregister};

/// `Item.QuestItemHue`.
pub const QUEST_ITEM_HUE: i32 = 0x4EA;

/// The statics of `Item`.
pub struct ItemStatics {
    /// `Item.DefaultDecayTime`.
    pub default_decay_time: TimeSpan,
    /// `Item.LockedDownFlag`.
    pub locked_down_flag: i32,
    /// `Item.SecureFlag`.
    pub secure_flag: i32,
    /// `Item.ScissorCopyLootType`.
    pub scissor_copy_loot_type: bool,
    /// `m_DeltaQueue`.
    pub delta_queue: VecDeque<ItemId>,
    /// `m_OpenSlots` (DropToWorld's scratch).
    pub open_slots: i32,
}

impl Default for ItemStatics {
    fn default() -> Self {
        ItemStatics {
            default_decay_time: TimeSpan::from_hours(1),
            locked_down_flag: 0,
            secure_flag: 0,
            scissor_copy_loot_type: false,
            delta_queue: VecDeque::new(),
            open_slots: 0,
        }
    }
}

pub fn statics(w: &mut World) -> &mut ItemStatics {
    w.resources.get_mut::<ItemStatics>()
}

fn statics_ref(w: &World) -> Option<&ItemStatics> {
    w.resources.get::<ItemStatics>()
}

/// `Item.DefaultDecayTime`.
pub fn default_decay_time(w: &World) -> TimeSpan {
    statics_ref(w).map_or(TimeSpan::from_hours(1), |s| s.default_decay_time)
}

/// `item?.Deleted != false`: an id that no longer resolves counts as deleted.
pub fn is_deleted(w: &World, id: ItemId) -> bool {
    w.item(id).is_none_or(|i| i.deleted())
}

/// `new Item(itemID)` of class `ty` (the `[Constructible]` constructors of every class call this first).
pub fn new_item_with_id(w: &mut World, ty: &'static crate::item::ItemType, item_id: i32) -> ItemId {
    let id = w.new_item(ty);
    if let Some(i) = w.item_mut(id) {
        i.item_id = item_id;
    }
    id
}

fn ty_of(w: &World, id: ItemId) -> Option<&'static crate::item::ItemType> {
    w.item(id).map(|i| i.ty)
}

/// `this is Container`.
pub fn is_container(w: &World, id: ItemId) -> bool {
    ty_of(w, id).is_some_and(|t| t.is(&CONTAINER))
}

// ---- Flags and small properties ----

pub fn set_flag(w: &mut World, id: ItemId, flag: ImplFlag, value: bool) {
    if let Some(i) = w.item_mut(id) {
        i.set_flag(flag, value);
    }
}

/// `TempFlags` setter.
pub fn set_temp_flags(w: &mut World, id: ItemId, value: i32) {
    if let Some(i) = w.item_mut(id) {
        i.temp_flags = value;
    }
}

/// `SavedFlags` setter.
pub fn set_saved_flags(w: &mut World, id: ItemId, value: i32) {
    if let Some(i) = w.item_mut(id) {
        i.saved_flags = value;
    }
}

/// `HeldBy` setter.
pub fn set_held_by(w: &mut World, id: ItemId, value: Option<MobileId>) {
    if let Some(i) = w.item_mut(id) {
        i.held_by = value;
    }
}

/// `LootType` setter.
pub fn set_loot_type(w: &mut World, id: ItemId, value: LootType) {
    let Some(i) = w.item_mut(id) else { return };
    if i.loot_type != value {
        i.loot_type = value;
        let ty = i.ty;
        if (ty.display_loot_type)(w, id) {
            invalidate_properties(w, id);
        }
    }
}

/// `LastMoved` setter: a move at or past the reset stamp supersedes it.
pub fn set_last_moved_value(w: &mut World, id: ItemId, value: DateTime) {
    if let Some(i) = w.item_mut(id) {
        i.last_moved = value;
        if i.decay_reset != DateTime::MIN && i.decay_reset <= value {
            i.decay_reset = DateTime::MIN;
        }
    }
}

/// `Stackable` setter.
pub fn set_stackable(w: &mut World, id: ItemId, value: bool) {
    set_flag(w, id, ImplFlag::STACKABLE, value);
}

/// `Visible` setter.
pub fn set_visible(w: &mut World, id: ItemId, value: bool) {
    let Some(i) = w.item(id) else { return };
    if i.visible() == value {
        return;
    }
    let (map, ty) = (i.map, i.ty);
    set_flag(w, id, ImplFlag::VISIBLE, value);

    if let Some(map) = map {
        let world_loc = get_world_location(w, id);
        for (ns, m) in deps::clients_in_range(w, map, world_loc, (ty.get_max_update_range)(w, id)) {
            if !deps::can_see(w, m, id)
                && deps::mobile_in_range(w, m, world_loc, (ty.get_update_range)(w, id, m))
            {
                crate::packets::entity::send_remove_entity(w, ns, id.0);
            }
        }
    }

    delta(w, id, ItemDelta::UPDATE);
    restart_decay(w, id);
}

/// `Movable` setter.
pub fn set_movable(w: &mut World, id: ItemId, value: bool) {
    let Some(i) = w.item(id) else { return };
    if i.movable() != value {
        set_flag(w, id, ImplFlag::MOVABLE, value);
        delta(w, id, ItemDelta::UPDATE);
        restart_decay(w, id);
    }
}

/// `GetTempFlag(flag)`.
pub fn get_temp_flag(w: &World, id: ItemId, flag: i32) -> bool {
    w.item(id).is_some_and(|i| i.temp_flags & flag != 0)
}

/// `SetTempFlag(flag, value)`.
pub fn set_temp_flag(w: &mut World, id: ItemId, flag: i32, value: bool) {
    if let Some(i) = w.item_mut(id) {
        if value {
            i.temp_flags |= flag;
        } else {
            i.temp_flags &= !flag;
        }
    }
}

/// `GetSavedFlag(flag)`.
pub fn get_saved_flag(w: &World, id: ItemId, flag: i32) -> bool {
    w.item(id).is_some_and(|i| i.saved_flags & flag != 0)
}

/// `SetSavedFlag(flag, value)`.
pub fn set_saved_flag(w: &mut World, id: ItemId, flag: i32, value: bool) {
    if let Some(i) = w.item_mut(id) {
        if value {
            i.saved_flags |= flag;
        } else {
            i.saved_flags &= !flag;
        }
    }
}

/// `IsLockedDown`.
pub fn is_locked_down(w: &World, id: ItemId) -> bool {
    get_temp_flag(w, id, statics_ref(w).map_or(0, |s| s.locked_down_flag))
}

/// `IsLockedDown` setter.
pub fn set_locked_down(w: &mut World, id: ItemId, value: bool) {
    let flag = statics(w).locked_down_flag;
    if get_temp_flag(w, id, flag) == value {
        return;
    }
    set_temp_flag(w, id, flag, value);
    invalidate_properties(w, id);
    on_security_changed(w, id);
}

/// `IsSecure`.
pub fn is_secure(w: &World, id: ItemId) -> bool {
    get_temp_flag(w, id, statics_ref(w).map_or(0, |s| s.secure_flag))
}

/// `IsSecure` setter.
pub fn set_secure(w: &mut World, id: ItemId, value: bool) {
    let flag = statics(w).secure_flag;
    if get_temp_flag(w, id, flag) == value {
        return;
    }
    set_temp_flag(w, id, flag, value);
    invalidate_properties(w, id);
    on_security_changed(w, id);
}

/// Lockdown and secure status decide whether this item and, for a container, its direct contents are
/// decay-eligible (see `CanDecay`); re-evaluate both.
fn on_security_changed(w: &mut World, id: ItemId) {
    update_decay_registration(w, id);
    if is_container(w, id) {
        crate::item::container::update_contents_decay_registration(w, id);
    }
}

/// `TotalGold`, `TotalItems`, `TotalWeight`: `GetTotal(type)`.
pub fn get_total(w: &World, id: ItemId, ty: TotalType) -> i32 {
    ty_of(w, id).map_or(0, |t| (t.get_total)(w, id, ty))
}

/// `Weight` getter.
pub fn weight(w: &World, id: ItemId) -> f64 {
    let Some(i) = w.item(id) else { return 0.0 };
    if i.weight >= 0.0 {
        i.weight
    } else {
        (i.ty.default_weight)(w, id)
    }
}

/// `PileWeight`: `(int)Math.Ceiling(Weight * Amount)`.
pub fn pile_weight(w: &World, id: ItemId) -> i32 {
    let amount = w.item(id).map_or(0, |i| i.amount);
    (weight(w, id) * f64::from(amount)).ceil() as i32
}

/// `Weight` setter.
pub fn set_weight(w: &mut World, id: ItemId, value: f64) {
    let Some(i) = w.item(id) else { return };
    let (ty, amount) = (i.ty, i.amount);
    let default_weight = (ty.default_weight)(w, id);
    let old_weight = if i.weight >= 0.0 {
        i.weight
    } else {
        default_weight
    };

    if old_weight == value {
        return;
    }

    if let Some(i) = w.item_mut(id) {
        i.weight = if value < 0.0 || default_weight == value {
            -1.0
        } else {
            value
        };
    }

    let delta = pile_weight(w, id) - (old_weight * f64::from(amount)).ceil() as i32;
    update_total(w, id, id, TotalType::Weight, delta);
    invalidate_properties(w, id);
}

/// `UpdateTotal(sender, type, delta)` on the item (virtual).
pub fn update_total(w: &mut World, id: ItemId, sender: ItemId, ty: TotalType, delta: i32) {
    if let Some(t) = ty_of(w, id) {
        (t.update_total)(w, id, sender, ty, delta);
    }
}

/// `RootParent`: the topmost parent (an item with no parent ends the walk and is itself returned).
pub fn root_parent(w: &World, id: ItemId) -> Option<EntityId> {
    let mut p = w.item(id)?.parent;
    while let Some(EntityId::Item(item)) = p {
        match w.item(item).and_then(|i| i.parent) {
            Some(next) => p = Some(next),
            None => break,
        }
    }
    p
}

/// `IsDupedEquipLayer()`.
pub fn is_duped_equip_layer(w: &World, id: ItemId) -> bool {
    let Some(i) = w.item(id) else { return false };
    match i.parent {
        Some(EntityId::Mobile(m)) => deps::find_item_on_layer(w, m, i.layer) != Some(id),
        _ => false,
    }
}

/// `ItemData`: the tiledata row of this item's graphic.
pub fn item_data(w: &World, id: ItemId) -> &muo_data::tile_data::ItemData {
    let item_id = w.item(id).map_or(0, |i| i.item_id);
    let idx = (item_id as usize) & muo_data::TileData::MAX_ITEM_VALUE;
    w.maps.tile_data.item(idx)
}

/// `Name` getter: `CompactInfo.m_Name ?? DefaultName`.
pub fn name(w: &World, id: ItemId) -> Option<String> {
    let i = w.item(id)?;
    i.name.clone().or_else(|| (i.ty.default_name)(w, id))
}

/// `Name` setter.
pub fn set_name(w: &mut World, id: ItemId, value: Option<String>) {
    let Some(ty) = ty_of(w, id) else { return };
    let default_name = (ty.default_name)(w, id);
    if let Some(i) = w.item_mut(id) {
        i.name = match value {
            None => None,
            Some(v) if Some(&v) == default_name.as_ref() => None,
            some => some,
        };
    }
    invalidate_properties(w, id);
}

/// `Parent` setter.
pub fn set_parent(w: &mut World, id: ItemId, value: Option<EntityId>) {
    let Some(i) = w.item(id) else { return };
    if i.parent == value {
        return;
    }
    let (old_parent, map) = (i.parent, i.map);

    if let Some(map) = map
        && old_parent.is_none()
        && value.is_some()
    {
        deps::map_on_leave(w, map, id);
    }

    if let Some(i) = w.item_mut(id) {
        i.parent = value;
    }

    if let Some(map) = map
        && old_parent.is_some()
        && value.is_none()
    {
        deps::map_on_enter(w, map, id);
    }
}

/// `Light` setter.
pub fn set_light(w: &mut World, id: ItemId, value: LightType) {
    let Some(i) = w.item_mut(id) else { return };
    let bits = Direction::from_bits_retain(value as i32 as u8);
    if i.direction != bits {
        i.direction = bits;
        delta(w, id, ItemDelta::UPDATE);
    }
}

/// `Direction` setter.
pub fn set_direction(w: &mut World, id: ItemId, value: Direction) {
    let Some(i) = w.item_mut(id) else { return };
    if i.direction != value {
        i.direction = value;
        delta(w, id, ItemDelta::UPDATE);
    }
}

/// `Amount` setter.
pub fn set_amount(w: &mut World, id: ItemId, value: i32) {
    let Some(i) = w.item(id) else { return };
    let (old_value, ty) = (i.amount, i.ty);
    if old_value == value {
        return;
    }
    let old_pile_weight = pile_weight(w, id);
    if let Some(i) = w.item_mut(id) {
        i.amount = value;
    }

    if value <= 0 {
        tracing::error!(
            "Item {} ({:?}) was changed to amount of {}, but must be at least 1",
            ty.full_name,
            id.0,
            value
        );
    }

    let new_pile_weight = pile_weight(w, id);
    update_total(
        w,
        id,
        id,
        TotalType::Weight,
        new_pile_weight - old_pile_weight,
    );
    (ty.on_amount_change)(w, id, old_value);
    delta(w, id, ItemDelta::UPDATE);

    if old_value > 1 || value > 1 {
        invalidate_properties(w, id);
    }

    if let Some(i) = w.item(id)
        && !i.stackable()
        && i.amount > 1
    {
        tracing::warn!(
            "{:?}: Amount changed for non-stackable item '{}'. ({})",
            id.0,
            ty.full_name,
            i.amount
        );
    }
}

/// `InSecureTrade`.
pub fn in_secure_trade(w: &World, id: ItemId) -> bool {
    get_secure_trade_cont(w, id).is_some()
}

/// `QuestItem` setter.
pub fn set_quest_item(w: &mut World, id: ItemId, value: bool) {
    set_flag(w, id, ImplFlag::QUEST_ITEM, value);
    invalidate_properties(w, id);
    delta(w, id, ItemDelta::UPDATE);
}

/// `Insured` setter.
pub fn set_insured(w: &mut World, id: ItemId, value: bool) {
    set_flag(w, id, ImplFlag::INSURED, value);
    invalidate_properties(w, id);
}

/// `PaidInsurance` setter.
pub fn set_paid_insurance(w: &mut World, id: ItemId, value: bool) {
    set_flag(w, id, ImplFlag::PAID_INSURANCE, value);
}

/// `BlessedFor` setter.
pub fn set_blessed_for(w: &mut World, id: ItemId, value: Option<MobileId>) {
    if let Some(i) = w.item_mut(id) {
        i.blessed_for = value;
    }
    invalidate_properties(w, id);
}

/// `Spawner` setter.
pub fn set_spawner(w: &mut World, id: ItemId, value: Option<crate::interfaces::SpawnerId>) {
    let Some(i) = w.item_mut(id) else { return };
    let old = i.spawner;
    i.spawner = value;
    if old != value {
        restart_decay(w, id);
    }
}

// ---- Location ----

/// `X`, `Y`, `Z` setters go through the virtual `Location`.
pub fn set_location(w: &mut World, id: ItemId, value: Point3D) {
    if let Some(ty) = ty_of(w, id) {
        (ty.set_location)(w, id, value);
    }
}

pub fn set_x(w: &mut World, id: ItemId, x: i32) {
    if let Some(l) = w.item(id).map(|i| i.location) {
        set_location(w, id, Point3D::new(x, l.y, l.z));
    }
}

pub fn set_y(w: &mut World, id: ItemId, y: i32) {
    if let Some(l) = w.item(id).map(|i| i.location) {
        set_location(w, id, Point3D::new(l.x, y, l.z));
    }
}

pub fn set_z(w: &mut World, id: ItemId, z: i32) {
    if let Some(l) = w.item(id).map(|i| i.location) {
        set_location(w, id, Point3D::new(l.x, l.y, z));
    }
}

/// `GetExpandFlags()`.
pub fn get_expand_flags(w: &World, id: ItemId) -> ExpandFlag {
    let Some(i) = w.item(id) else {
        return ExpandFlag::NONE;
    };
    let mut flags = ExpandFlag::NONE;
    if i.blessed_for.is_some() {
        flags |= ExpandFlag::BLESSED;
    }
    if i.bounce.is_some() {
        flags |= ExpandFlag::BOUNCE;
    }
    if i.held_by.is_some() {
        flags |= ExpandFlag::HOLDER;
    }
    // `m_Items != null`: the list exists once anything was added (an emptied list keeps existing).
    if !i.items.is_empty() {
        flags |= ExpandFlag::ITEMS;
    }
    if i.name.is_some() {
        flags |= ExpandFlag::NAME;
    }
    if i.spawner.is_some() {
        flags |= ExpandFlag::SPAWNER;
    }
    if i.saved_flags != 0 {
        flags |= ExpandFlag::SAVE_FLAG;
    }
    if i.temp_flags != 0 {
        flags |= ExpandFlag::TEMP_FLAG;
    }
    if i.weight >= 0.0 {
        flags |= ExpandFlag::WEIGHT;
    }
    flags
}

// ---- Bounce ----

/// `RecordBounce()`: `new BounceInfo(this)`.
pub fn record_bounce(w: &mut World, id: ItemId) {
    let world_loc = get_world_location(w, id);
    if let Some(i) = w.item_mut(id) {
        i.bounce = Some(BounceInfo {
            location: i.location,
            world_loc,
            map: i.map,
            parent: i.parent,
        });
    }
}

/// `ClearBounce()`.
pub fn clear_bounce(w: &mut World, id: ItemId) {
    let Some(bounce) = w.item_mut(id).and_then(|i| i.bounce.take()) else {
        return;
    };
    match bounce.parent {
        Some(EntityId::Item(p)) => {
            if !is_deleted(w, p)
                && let Some(ty) = ty_of(w, p)
            {
                (ty.on_item_bounce_cleared)(w, p, id);
            }
        }
        Some(EntityId::Mobile(m)) => {
            if !deps::mobile_deleted(w, m)
                && let Some(ty) = w.mobile(m).map(|m| m.ty)
            {
                (ty.on_item_bounce_cleared)(w, m, id);
            }
        }
        None => {}
    }
}

/// `Bounce(from)`: puts a refused drop back where it came from.
pub fn bounce(w: &mut World, id: ItemId, from: MobileId) {
    let Some(i) = w.item(id) else { return };
    let parent = i.parent;
    match parent {
        Some(EntityId::Item(p)) => {
            if let Some(ty) = ty_of(w, p) {
                (ty.remove_item)(w, p, id);
            }
        }
        Some(EntityId::Mobile(m)) => deps::mobile_remove_item(w, m, id),
        None => {}
    }
    if let Some(i) = w.item_mut(id) {
        i.parent = None;
    }

    let (from_loc, from_map) = w
        .mobile(from)
        .map_or((Point3D::default(), None), |m| (m.location, m.map));

    let Some(bounce) = w.item(id).and_then(|i| i.bounce.clone()) else {
        move_to_world_map(w, id, from_loc, from_map);
        return;
    };

    let parent = match bounce.parent {
        Some(EntityId::Item(p)) if !is_deleted(w, p) => Some(EntityId::Item(p)),
        Some(EntityId::Mobile(m)) if !deps::mobile_deleted(w, m) => Some(EntityId::Mobile(m)),
        _ => None,
    };

    match parent {
        None => move_to_world_map(w, id, from_loc, from_map),
        Some(EntityId::Item(ip)) => {
            let root = root_parent(w, ip);
            let rpm = match root {
                Some(EntityId::Mobile(m)) => Some(m),
                _ => None,
            };
            let ip_ty = w.item(ip).map(|i| i.ty);
            let accessible = ip_ty.is_some_and(|t| (t.is_accessible_to)(w, ip, from));
            let nonlocal_ok = rpm.is_none_or(|m| deps::check_nonlocal_drop(w, m, from, id, ip));
            let (ip_movable, ip_map) = w.item(ip).map_or((false, None), |i| (i.movable(), i.map));
            let root_loc = root.map(|r| entity_location(w, r));
            if accessible
                && nonlocal_ok
                && (!ip_movable
                    || rpm == Some(from)
                    || (ip_map == bounce.map && root_loc == Some(bounce.world_loc)))
            {
                set_location(w, id, bounce.location);
                if let Some(t) = ip_ty {
                    (t.add_item)(w, ip, id);
                }
            } else {
                move_to_world_map(w, id, from_loc, from_map);
            }
        }
        Some(EntityId::Mobile(mobile)) => {
            let equipped = w
                .mobile(mobile)
                .map(|m| m.ty)
                .is_some_and(|t| (t.equip_item)(w, mobile, id));
            if !equipped {
                move_to_world_map(w, id, bounce.world_loc, bounce.map);
            }
        }
    }

    clear_bounce(w, id);
}

fn entity_location(w: &World, e: EntityId) -> Point3D {
    match e {
        EntityId::Item(i) => w.item(i).map_or_else(Point3D::default, |i| i.location),
        EntityId::Mobile(m) => w.mobile(m).map_or_else(Point3D::default, |m| m.location),
    }
}

// ---- Decay ----

/// `ScheduledDecayTime`.
pub fn scheduled_decay_time(w: &World, id: ItemId) -> DateTime {
    let Some(i) = w.item(id) else {
        return DateTime::MIN;
    };
    let latest = if i.decay_reset > i.last_moved {
        i.decay_reset
    } else {
        i.last_moved
    };
    latest + (i.ty.decay_time)(w, id)
}

/// `DecayResetTime` setter.
fn set_decay_reset_time(w: &mut World, id: ItemId, value: DateTime) {
    if let Some(i) = w.item_mut(id) {
        i.decay_reset = value;
    }
}

/// `RestartDecay()`.
pub fn restart_decay(w: &mut World, id: ItemId) {
    let Some(ty) = ty_of(w, id) else { return };
    if (ty.can_decay)(w, id) {
        let now = w.now;
        if scheduled_decay_time(w, id) < now + (ty.decay_time)(w, id) {
            set_decay_reset_time(w, id, now);
        }
    }
    update_decay_registration(w, id);
}

/// `UpdateDecayRegistration()`.
pub fn update_decay_registration(w: &mut World, id: ItemId) {
    let Some(ty) = ty_of(w, id) else { return };
    decay_unregister(w, id);
    if (ty.can_decay)(w, id) {
        decay_register(w, id);
    } else {
        // No countdown to anchor while ineligible; re-eligibility always re-anchors.
        set_decay_reset_time(w, id, DateTime::MIN);
    }
}

/// `SetLastMoved()`.
pub fn set_last_moved(w: &mut World, id: ItemId) {
    let now = w.now;
    set_last_moved_value(w, id, now);
    update_decay_registration(w, id);
}

// ---- Position helpers ----

/// `GetWorldLocation()`.
pub fn get_world_location(w: &World, id: ItemId) -> Point3D {
    match root_parent(w, id) {
        None => w.item(id).map_or_else(Point3D::default, |i| i.location),
        Some(root) => entity_location(w, root),
    }
}

/// `GetWorldTop()`.
pub fn get_world_top(w: &World, id: ItemId) -> Point3D {
    if let Some(EntityId::Item(root)) = root_parent(w, id) {
        return get_world_top(w, root);
    }
    let loc = w.item(id).map_or_else(Point3D::default, |i| i.location);
    Point3D::new(loc.x, loc.y, loc.z + item_data(w, id).calc_height())
}

/// `GetSurfaceTop()`: the point of the topmost surface item above (`None` is "this item itself").
pub fn get_surface_top(w: &World, id: ItemId) -> SurfaceTop {
    if let Some(EntityId::Item(root)) = root_parent(w, id) {
        return get_surface_top(w, root);
    }
    let data = item_data(w, id);
    if data.surface() {
        let loc = w.item(id).map_or_else(Point3D::default, |i| i.location);
        SurfaceTop::Point(Point3D::new(loc.x, loc.y, loc.z + data.calc_height()))
    } else {
        SurfaceTop::Item(id)
    }
}

/// What `GetSurfaceTop` returns: a point, or the item itself (an `IPoint3D`).
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum SurfaceTop {
    Point(Point3D),
    Item(ItemId),
}

/// `GetSecureTradeCont()`: the first `SecureTradeContainer` on the way up (this item included).
pub fn get_secure_trade_cont(w: &World, id: ItemId) -> Option<ItemId> {
    let mut p = Some(EntityId::Item(id));
    while let Some(EntityId::Item(item)) = p {
        let i = w.item(item)?;
        if i.ty
            .is(&crate::item::secure_trade_container::SECURE_TRADE_CONTAINER)
        {
            return Some(item);
        }
        p = i.parent;
    }
    None
}

/// `IsChildOf(o)`.
pub fn is_child_of(w: &World, id: ItemId, o: Option<EntityId>) -> bool {
    is_child_of_null(w, id, o, false)
}

/// `IsChildOf(o, allowNull)`.
pub fn is_child_of_null(w: &World, id: ItemId, o: Option<EntityId>, allow_null: bool) -> bool {
    let Some(i) = w.item(id) else { return false };
    let mut p = i.parent;
    if (p.is_none() || o.is_none()) && !allow_null {
        return false;
    }
    if p == o {
        return true;
    }
    while let Some(EntityId::Item(item)) = p {
        let Some(next) = w.item(item).and_then(|i| i.parent) else {
            break;
        };
        p = Some(next);
        if p == o {
            return true;
        }
    }
    false
}

/// `ParentsContain<T>()`.
pub fn parents_contain(w: &World, id: ItemId, ty: &'static crate::item::ItemType) -> bool {
    let mut p = w.item(id).and_then(|i| i.parent);
    while let Some(EntityId::Item(item)) = p {
        let Some(it) = w.item(item) else { break };
        if it.ty.is(ty) {
            return true;
        }
        match it.parent {
            Some(next) => p = Some(next),
            None => break,
        }
    }
    false
}

/// `AtWorldPoint(x, y)`.
pub fn at_world_point(w: &World, id: ItemId, x: i32, y: i32) -> bool {
    w.item(id)
        .is_some_and(|i| i.parent.is_none() && i.location.x == x && i.location.y == y)
}

/// `AtPoint(x, y)`.
pub fn at_point(w: &World, id: ItemId, x: i32, y: i32) -> bool {
    w.item(id)
        .is_some_and(|i| i.location.x == x && i.location.y == y)
}

/// `GetGraphicBounds()`.
pub fn get_graphic_bounds(w: &World, id: ItemId) -> Rectangle2D {
    let Some(i) = w.item(id) else {
        return Rectangle2D::default();
    };
    let mut item_id = i.item_id;
    let mut doubled = i.amount > 1;

    if (0xEEA..=0xEF2).contains(&item_id) {
        // coins
        let coin_base = (item_id - 0xEEA) / 3 * 3 + 0xEEA;
        doubled = false;
        item_id = match i.amount {
            ..=1 => coin_base,
            2..=5 => coin_base + 1,
            _ => coin_base + 2,
        };
    }

    let mut bounds = crate::item::bounds::bounds(w, (item_id & 0x3FFF) as usize);
    if doubled {
        bounds.set(
            bounds.x(),
            bounds.y(),
            bounds.width() + 5,
            bounds.height() + 5,
        );
    }
    bounds
}

// ---- Deltas ----

/// `Delta(flags)`.
pub fn delta(w: &mut World, id: ItemId, flags: ItemDelta) {
    let Some(i) = w.item_mut(id) else { return };
    if i.map.is_none_or(|m| m == MapId::INTERNAL) {
        return;
    }
    i.delta_flags |= flags;
    if !i.get_flag(ImplFlag::IN_QUEUE) {
        i.set_flag(ImplFlag::IN_QUEUE, true);
        statics(w).delta_queue.push_back(id);
    }
}

/// `RemDelta(flags)`.
pub fn rem_delta(w: &mut World, id: ItemId, flags: ItemDelta) {
    if let Some(i) = w.item_mut(id) {
        i.delta_flags &= !flags;
    }
}

/// `Item.HasQueuedDeltas`.
pub fn has_queued_deltas(w: &World) -> bool {
    statics_ref(w).is_some_and(|s| !s.delta_queue.is_empty())
}

/// `Item.ProcessDeltaQueue()`.
pub fn process_delta_queue(w: &mut World) {
    let mut limit = statics(w).delta_queue.len() as i64;
    loop {
        limit -= 1;
        if limit < 0 {
            break;
        }
        let Some(id) = statics(w).delta_queue.pop_front() else {
            break;
        };
        let Some(ty) = ty_of(w, id) else { continue };
        set_flag(w, id, ImplFlag::IN_QUEUE, false);
        (ty.process_delta)(w, id);
    }
    let left = statics(w).delta_queue.len();
    if left > 0 {
        tracing::warn!("{left} items left in delta queue after processing.");
    }
}

// ---- Moving ----

/// `MoveToWorld(location, map)`.
pub fn move_to_world_map(w: &mut World, id: ItemId, location: Point3D, map: Option<MapId>) {
    let Some(i) = w.item(id) else { return };
    if i.deleted() {
        return;
    }
    let (ty, old_real_location, parent) = (i.ty, i.location, i.parent);
    let old_location = get_world_location(w, id);

    // Register at the end instead: CanDecay() reads parent, map and location, none of which are final yet.
    let now = w.now;
    set_last_moved_value(w, id, now);

    // A parented item's removal already reached every client that was sent it via RemoveItem below; sending
    // another remove at oldLocation would broadcast it to bystanders who never knew about it.
    let was_contained = parent.is_some();

    match parent {
        Some(EntityId::Mobile(m)) => deps::mobile_remove_item(w, m, id),
        Some(EntityId::Item(p)) => {
            if let Some(pt) = ty_of(w, p) {
                (pt.remove_item)(w, p, id);
            }
        }
        None => {}
    }

    let cur_map = w.item(id).and_then(|i| i.map);
    if cur_map != map {
        let old = cur_map;

        if let Some(m) = cur_map {
            deps::map_on_leave(w, m, id);
            if !was_contained && old_location.x != 0 {
                send_remove_packet_at(w, id, old_location);
            }
        }

        if let Some(i) = w.item_mut(id) {
            i.location = location;
        }
        (ty.on_location_change)(w, id, old_real_location);

        let items = w.item(id).map(|i| i.items.clone()).unwrap_or_default();
        for child in items {
            set_map(w, child, map);
        }

        if let Some(i) = w.item_mut(id) {
            i.map = map;
        }
        if let Some(m) = map {
            deps::map_on_enter(w, m, id);
        }

        (ty.on_map_change)(w, id);

        if let Some(m) = map {
            let loc = w.item(id).map_or(location, |i| i.location);
            for (ns, mob) in deps::clients_in_range(w, m, loc, (ty.get_max_update_range)(w, id)) {
                if deps::can_see(w, mob, id)
                    && deps::mobile_in_range(w, mob, loc, (ty.get_update_range)(w, id, mob))
                {
                    let caps = w
                        .net_state(ns)
                        .map(|d| (d.caps.high_seas(), d.caps.stygian_abyss()));
                    let packet = match caps {
                        Some((true, _)) => {
                            crate::packets::item::create_world_entity_item(w, id, true)
                        }
                        Some((false, true)) => {
                            crate::packets::item::create_world_entity_item(w, id, false)
                        }
                        _ => crate::packets::item::create_world_item(w, id),
                    };
                    (ty.send_info_to)(w, id, ns, packet.as_deref().unwrap_or(&[]));
                }
            }
        }

        rem_delta(w, id, ItemDelta::UPDATE);

        if old.is_none() || old == Some(MapId::INTERNAL) {
            invalidate_properties(w, id);
        }
    } else if let Some(m) = cur_map {
        if !was_contained && old_location.x != 0 {
            for (ns, mob) in
                deps::clients_in_range(w, m, old_location, (ty.get_max_update_range)(w, id))
            {
                if !deps::mobile_in_range(w, mob, location, (ty.get_update_range)(w, id, mob)) {
                    crate::packets::entity::send_remove_entity(w, ns, id.0);
                }
            }
        }

        let old_internal_location = w.item(id).map_or(location, |i| i.location);
        if let Some(i) = w.item_mut(id) {
            i.location = location;
        }
        (ty.on_location_change)(w, id, old_real_location);

        for (ns, mob) in deps::clients_in_range(w, m, location, (ty.get_max_update_range)(w, id)) {
            if deps::can_see(w, mob, id)
                && deps::mobile_in_range(w, mob, location, (ty.get_update_range)(w, id, mob))
            {
                (ty.send_info_to)(w, id, ns, &[]);
            }
        }

        deps::map_on_move(w, m, old_internal_location, id);
        rem_delta(w, id, ItemDelta::UPDATE);
    } else {
        set_map(w, id, map);
        set_location(w, id, location);
    }

    update_decay_registration(w, id);
}

/// `Internalize()`.
pub fn internalize(w: &mut World, id: ItemId) {
    move_to_world_map(w, id, Point3D::default(), Some(MapId::INTERNAL));
}

/// `Map` setter.
pub fn set_map(w: &mut World, id: ItemId, value: Option<MapId>) {
    let Some(i) = w.item(id) else { return };
    if i.map == value {
        return;
    }
    let (old, parent, ty) = (i.map, i.parent, i.ty);

    if let Some(m) = old {
        if parent.is_none() {
            deps::map_on_leave(w, m, id);
        }

        // A contained item only changes map along with its parent or root; their own removal already clears
        // this subtree for every client that knew about it.
        let parent_is_container = matches!(parent, Some(EntityId::Item(p)) if is_container(w, p));
        if !parent_is_container {
            send_remove_packet(w, id);
        }
    }

    let items = w.item(id).map(|i| i.items.clone()).unwrap_or_default();
    for child in items {
        set_map(w, child, value);
    }

    if let Some(i) = w.item_mut(id) {
        i.map = value;
    }

    if parent.is_none()
        && let Some(m) = value
    {
        deps::map_on_enter(w, m, id);
    }

    delta(w, id, ItemDelta::UPDATE);
    (ty.on_map_change)(w, id);

    if parent.is_none() {
        // A map change is a move; nothing else updates decay registration for a raw Map change.
        set_last_moved(w, id);
    }

    if old.is_none() || old == Some(MapId::INTERNAL) {
        invalidate_properties(w, id);
    }
}

// ---- Properties ----

/// `InvalidateProperties()` on the item (virtual).
pub fn invalidate_properties(w: &mut World, id: ItemId) {
    if let Some(ty) = ty_of(w, id) {
        (ty.invalidate_properties)(w, id);
    }
}

/// `PropertyList` getter: builds the list on first use.
pub fn property_list(w: &mut World, id: ItemId) -> Option<&crate::property_list::PropertyList> {
    if w.item(id)?.property_list.is_none() {
        let mut list = crate::property_list::PropertyList::new(id.0);
        // Publish the list before building it so a nested InvalidateProperties can see the build in progress.
        list.is_building = true;
        initialize_property_list(w, id, &mut list);
        if let Some(i) = w.item_mut(id) {
            i.property_list = Some(list);
        }
    }
    w.item(id)?.property_list.as_ref()
}

/// `InitializePropertyList(list)`.
pub fn initialize_property_list(
    w: &World,
    id: ItemId,
    list: &mut crate::property_list::PropertyList,
) {
    list.is_building = true;
    if let Some(ty) = ty_of(w, id) {
        (ty.get_properties)(w, id, list);
        (ty.append_child_properties)(w, id, list);
    }
    list.terminate();
    list.is_building = false;
}

// ---- Private children (ProcessDelta, SendRemovePacket, IsSentTo) ----

/// `TryGetPrivateParent(out cont)`.
pub fn try_get_private_parent(w: &World, id: ItemId) -> Option<ItemId> {
    let Some(EntityId::Item(cont)) = w.item(id)?.parent else {
        return None;
    };
    let ct = w.item(cont)?.ty;
    if !ct.is(&CONTAINER) {
        return None;
    }
    if (ct.is_public_container)(w, cont) || (ct.is_child_public)(w, cont, id) {
        None
    } else {
        Some(cont)
    }
}

/// `GetPrivateChildRecipients(cont, out root, out tradeFrom, out tradeTo)`.
pub fn get_private_child_recipients(
    w: &World,
    id: ItemId,
    cont: ItemId,
) -> (Option<MobileId>, Option<MobileId>, Option<MobileId>) {
    let root = match root_parent(w, cont) {
        Some(EntityId::Mobile(m)) => Some(m),
        _ => None,
    };
    let (from, to) = crate::item::secure_trade_container::trade_of(w, id)
        .map_or((None, None), |t| deps::trade_parties(w, t));
    (root, from, to)
}

fn send_remove_to(w: &mut World, id: ItemId, m: Option<MobileId>, world_loc: Point3D) {
    let Some(m) = m else { return };
    let Some(ns) = deps::net_state_of(w, m) else {
        return;
    };
    let Some(ty) = ty_of(w, id) else { return };
    let item_map = w.item(id).and_then(|i| i.map);
    let same_map = w.mobile(m).is_some_and(|mob| mob.map == item_map);
    if same_map && deps::mobile_in_range(w, m, world_loc, (ty.get_update_range)(w, id, m)) {
        crate::packets::entity::send_remove_entity(w, ns, id.0);
    }
}

/// Same recipient gate as `send_remove_to`, plus CanSee: this is an informational OPL push, not cleanup of a
/// reference the client may already hold regardless of visibility.
pub fn send_private_opl_to(w: &mut World, id: ItemId, m: Option<MobileId>, world_loc: Point3D) {
    let Some(m) = m else { return };
    let Some(ns) = deps::net_state_of(w, m) else {
        return;
    };
    let Some(ty) = ty_of(w, id) else { return };
    let item_map = w.item(id).and_then(|i| i.map);
    let same_map = w.mobile(m).is_some_and(|mob| mob.map == item_map);
    if deps::can_see(w, m, id)
        && same_map
        && deps::mobile_in_range(w, m, world_loc, (ty.get_update_range)(w, id, m))
    {
        (ty.send_opl_packet_to)(w, id, ns);
    }
}

/// `SendRemovePacket()`.
pub fn send_remove_packet(w: &mut World, id: ItemId) {
    let loc = get_world_location(w, id);
    send_remove_packet_at(w, id, loc);
}

/// `SendRemovePacket(worldLoc)`.
pub fn send_remove_packet_at(w: &mut World, id: ItemId, world_loc: Point3D) {
    let Some(i) = w.item(id) else { return };
    if i.deleted() {
        return;
    }
    let Some(map) = i.map else { return };
    let ty = i.ty;

    if let Some(cont) = try_get_private_parent(w, id) {
        // Private children are only ever sent to these recipients (see ProcessDelta); nobody else knows them.
        let (root, trade_from, trade_to) = get_private_child_recipients(w, id, cont);

        send_remove_to(w, id, root, world_loc);
        if trade_from != root {
            send_remove_to(w, id, trade_from, world_loc);
        }
        if trade_to != root && trade_to != trade_from {
            send_remove_to(w, id, trade_to, world_loc);
        }

        let openers = w
            .item(cont)
            .and_then(|c| c.cont.as_ref())
            .and_then(|c| c.openers.clone())
            .unwrap_or_default();
        for mob in openers {
            if Some(mob) != root && Some(mob) != trade_from && Some(mob) != trade_to {
                send_remove_to(w, id, Some(mob), world_loc);
            }
        }
        return;
    }

    for (ns, m) in deps::clients_in_range(w, map, world_loc, (ty.get_max_update_range)(w, id)) {
        if deps::mobile_in_range(w, m, world_loc, (ty.get_update_range)(w, id, m)) {
            crate::packets::entity::send_remove_entity(w, ns, id.0);
        }
    }
}

/// `IsSentTo(m)`.
pub fn is_sent_to(w: &mut World, id: ItemId, m: Option<MobileId>) -> bool {
    let Some(m) = m else { return false };
    if is_deleted(w, id) {
        return false;
    }
    let Some(ty) = ty_of(w, id) else { return false };
    let world_loc = get_world_location(w, id);
    let in_range =
        |w: &World| deps::mobile_in_range(w, m, world_loc, (ty.get_update_range)(w, id, m));

    if let Some(cont) = try_get_private_parent(w, id) {
        let (root, trade_from, trade_to) = get_private_child_recipients(w, id, cont);

        if Some(m) == root {
            // Mirrors ProcessDelta's own root gate: CanSee and in range.
            return deps::can_see(w, m, id) && in_range(w);
        }

        let is_opener = w
            .item(cont)
            .and_then(|c| c.cont.as_ref())
            .and_then(|c| c.openers.as_ref())
            .is_some_and(|o| o.contains(&m));
        let is_trade_or_opener = Some(m) == trade_from || Some(m) == trade_to || is_opener;

        // Trade parties and openers are re-validated against ProcessDelta's own gate (CanSee, current map,
        // update range), so an invisible item or a stale Openers entry never resyncs to them.
        let item_map = w.item(id).and_then(|i| i.map);
        return is_trade_or_opener
            && deps::can_see(w, m, id)
            && w.mobile(m).is_some_and(|mob| mob.map == item_map)
            && in_range(w);
    }

    deps::can_see(w, m, id) && in_range(w)
}

// ---- Labels ----

/// `LabelTo(to, number, args)`.
pub fn label_to_number(w: &mut World, id: ItemId, to: MobileId, number: i32, args: &str) {
    if let Some(ns) = deps::net_state_of(w, to) {
        deps::label_localized(w, ns, id, number, "", args);
    }
}

/// `LabelTo(to, text)`.
pub fn label_to(w: &mut World, id: ItemId, to: MobileId, text: &str) {
    if let Some(ns) = deps::net_state_of(w, to) {
        deps::label_text(w, ns, id, text);
    }
}

/// `LabelToAffix(to, number, type, affix, args)`.
pub fn label_to_affix(
    w: &mut World,
    id: ItemId,
    to: MobileId,
    number: i32,
    affix_type: crate::packets::message::AffixType,
    affix: &str,
    args: &str,
) {
    if let Some(ns) = deps::net_state_of(w, to) {
        deps::label_localized_affix(w, ns, id, number, affix_type, affix, args);
    }
}

/// `CheckRace(Race)`: the race's flag is in `RequiredRaces`.
pub fn required_races(w: &World, id: ItemId) -> i32 {
    ty_of(w, id).map_or(0, |t| (t.required_races)(w, id))
}

/// Resets `slot` bookkeeping after an item is gone from the scheduler (used by tests).
pub fn decay_slot(w: &World, id: ItemId) -> i8 {
    w.item(id).map_or(DECAY_SLOT_NONE, |i| i.decay_slot)
}

/// `ItemID = value` through the class (`Container` and `BaseMulti` override the setter).
pub fn set_item_id(w: &mut World, id: ItemId, value: i32) {
    if let Some(ty) = ty_of(w, id) {
        (ty.set_item_id)(w, id, value);
    }
}
