//! Containers (and `ContainerData`).
//!
//! A `Container` is an `Item` whose [`Item::cont`](crate::item::Item::cont) is set; its contents are the item's
//! `items`, and `_version` is the item's `version`. The `[SerializationGenerator(0, false)]` layout is: a
//! version `int` of 0, a one byte save-flag set (`MaxItems`, `GumpID`, `DropSound`, `LiftOverride` in that
//! order, each only when it differs from its default) and the flagged values in the same order.
//!
//! Virtuals that only containers have (`CheckHold`, `IsPublicContainer`, ..) are `ItemType` entries with the
//! `_default` bodies in this file.

use std::collections::HashMap;

use muo_io::{GenericReader, GenericWriter, Result};
use muo_primitives::{Point3D, Rectangle2D, utility};

use crate::ids::{EntityId, ItemId, MobileId};
use crate::item::deps;
use crate::item::item_type::ITEM_BASE;
use crate::item::ops;
use crate::item::{DeathMoveResult, ItemType, TotalType};
use crate::lr_reason::LRReason;
use crate::mobile::AccessLevel;
use crate::net_state::NetStateId;
use crate::packets::message::MessageType;
use crate::property_list::PropertyList;
use crate::world::World;

/// `Server.Items.Container`. Not constructible by `[add` (the constructor is not `[Constructible]`).
pub static CONTAINER: ItemType = ItemType {
    full_name: "Server.Items.Container",
    parent: Some(&ITEM_BASE),
    construct: None,
    set_item_id,
    on_snoop,
    check_lift,
    check_item_use,
    get_total,
    update_total,
    update_totals,
    on_item_added,
    on_item_removed,
    on_drag_drop,
    on_double_click_secure_trade,
    on_single_click,
    on_delete,
    get_properties,
    on_double_click,
    serialize,
    deserialize,
    ..ITEM_BASE
};

/// What `Container` adds to an `Item`.
#[derive(Clone, Debug)]
pub struct ContainerState {
    /// `_totalGold`, `_totalItems`, `_totalWeight`.
    pub total_gold: i32,
    pub total_items: i32,
    pub total_weight: i32,
    /// `_maxItems`, `_gumpID`, `_dropSound`: -1 means the class default.
    pub max_items: i32,
    pub gump_id: i32,
    pub drop_sound: i32,
    /// `_liftOverride`.
    pub lift_override: bool,
    /// `Openers`.
    pub openers: Option<Vec<MobileId>>,
    /// `_containerData`: cached by `UpdateContainerData`, cleared when the item id changes.
    pub data: Option<ContainerData>,
}

impl Default for ContainerState {
    fn default() -> Self {
        ContainerState {
            total_gold: 0,
            total_items: 0,
            total_weight: 0,
            max_items: -1,
            gump_id: -1,
            drop_sound: -1,
            lift_override: false,
            openers: None,
            data: None,
        }
    }
}

/// `Container.GlobalMaxItems`, `GlobalMaxWeight`, `SnoopHandler`.
pub struct ContainerStatics {
    pub global_max_items: i32,
    pub global_max_weight: i32,
    /// `SnoopHandler(cont, from)`.
    pub snoop_handler: Option<fn(&mut World, ItemId, MobileId)>,
}

impl Default for ContainerStatics {
    fn default() -> Self {
        ContainerStatics {
            global_max_items: 125,
            global_max_weight: 400,
            snoop_handler: None,
        }
    }
}

pub fn statics(w: &mut World) -> &mut ContainerStatics {
    w.resources.get_mut::<ContainerStatics>()
}

fn global_max_items(w: &World) -> i32 {
    w.resources
        .get::<ContainerStatics>()
        .map_or(125, |s| s.global_max_items)
}

fn global_max_weight(w: &World) -> i32 {
    w.resources
        .get::<ContainerStatics>()
        .map_or(400, |s| s.global_max_weight)
}

// ---- ContainerData ----

/// `ContainerData`: the gump, its item area and the drop sound for a container graphic.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct ContainerData {
    pub gump_id: i32,
    pub bounds: Rectangle2D,
    pub drop_sound: i32,
}

const BUILT_IN_DEFAULT: ContainerData = ContainerData {
    gump_id: 0x3C,
    bounds: Rectangle2D::new(44, 65, 142, 94),
    drop_sound: 0x48,
};

/// The statics of `ContainerData`: `Data/containers.cfg`.
pub struct ContainerDataTable {
    table: HashMap<i32, ContainerData>,
    /// `ContainerData.Default`.
    pub default: ContainerData,
    loaded: bool,
}

impl Default for ContainerDataTable {
    fn default() -> Self {
        ContainerDataTable {
            table: HashMap::new(),
            default: BUILT_IN_DEFAULT,
            loaded: false,
        }
    }
}

/// The static constructor of `ContainerData`: reads `Data/containers.cfg` (once).
pub fn ensure_container_data_loaded(w: &mut World) {
    if w.resources.get_mut::<ContainerDataTable>().loaded {
        return;
    }
    let path = w.core.base_directory.join("Data/containers.cfg");
    let table = match std::fs::read_to_string(&path) {
        Ok(text) => parse_containers_cfg(&text),
        Err(_) => ContainerDataTable::default(),
    };
    let t = w.resources.get_mut::<ContainerDataTable>();
    *t = table;
    t.loaded = true;
}

/// The body of the `ContainerData` static constructor over the file's text.
pub fn parse_containers_cfg(text: &str) -> ContainerDataTable {
    let mut table = HashMap::new();
    let mut default: Option<ContainerData> = None;

    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let split: Vec<&str> = line.split('\t').collect();
        if split.len() < 3 {
            continue;
        }
        let Some(gump_id) = muo_primitives::parse::try_to_i32(split[0]) else {
            continue;
        };
        let rect: Vec<&str> = split[1].split(' ').collect();
        if rect.len() < 4 {
            continue;
        }
        let nums: Option<Vec<i32>> = rect[..4]
            .iter()
            .map(|s| muo_primitives::parse::try_to_i32(s))
            .collect();
        let Some(n) = nums else { continue };
        let Some(drop_sound) = muo_primitives::parse::try_to_i32(split[2]) else {
            continue;
        };

        let data = ContainerData {
            gump_id,
            bounds: Rectangle2D::new(n[0], n[1], n[2], n[3]),
            drop_sound,
        };
        default.get_or_insert(data);

        if split.len() >= 4 {
            for part in split[3].split(',') {
                let Some(id) = muo_primitives::parse::try_to_i32(part) else {
                    break; // a bad id abandons the rest of the line
                };
                if let std::collections::hash_map::Entry::Vacant(e) = table.entry(id) {
                    e.insert(data);
                } else {
                    tracing::warn!("double ItemID entry in Data/containers.cfg");
                }
            }
        }
    }

    ContainerDataTable {
        table,
        default: default.unwrap_or(BUILT_IN_DEFAULT),
        loaded: true,
    }
}

/// `ContainerData.GetData(itemID)`.
pub fn get_data(w: &World, item_id: i32) -> ContainerData {
    match w.resources.get::<ContainerDataTable>() {
        Some(t) => t.table.get(&item_id).copied().unwrap_or(t.default),
        None => BUILT_IN_DEFAULT,
    }
}

/// `ContainerData` getter.
pub fn container_data(w: &World, id: ItemId) -> ContainerData {
    let Some(i) = w.item(id) else {
        return BUILT_IN_DEFAULT;
    };
    i.cont
        .as_ref()
        .and_then(|c| c.data)
        .unwrap_or_else(|| get_data(w, i.item_id))
}

// ---- Properties ----

fn state(w: &World, id: ItemId) -> Option<&ContainerState> {
    w.item(id)?.cont.as_deref()
}

fn state_mut(w: &mut World, id: ItemId) -> Option<&mut ContainerState> {
    Some(w.item_mut(id)?.container_mut())
}

/// `MaxItems` getter.
pub fn max_items(w: &World, id: ItemId) -> i32 {
    let raw = state(w, id).map_or(-1, |c| c.max_items);
    if raw == -1 {
        w.item(id).map_or(0, |i| (i.ty.default_max_items)(w, id))
    } else {
        raw
    }
}

/// `MaxItems` setter.
pub fn set_max_items(w: &mut World, id: ItemId, value: i32) {
    if let Some(c) = state_mut(w, id) {
        c.max_items = value;
    }
    ops::invalidate_properties(w, id);
}

/// `GumpID` getter.
pub fn gump_id(w: &World, id: ItemId) -> i32 {
    let raw = state(w, id).map_or(-1, |c| c.gump_id);
    if raw == -1 {
        w.item(id).map_or(0, |i| (i.ty.default_gump_id)(w, id))
    } else {
        raw
    }
}

/// `GumpID` setter.
pub fn set_gump_id(w: &mut World, id: ItemId, value: i32) {
    if let Some(c) = state_mut(w, id) {
        c.gump_id = value;
    }
}

/// `DropSound` getter.
pub fn drop_sound(w: &World, id: ItemId) -> i32 {
    let raw = state(w, id).map_or(-1, |c| c.drop_sound);
    if raw == -1 {
        w.item(id).map_or(0, |i| (i.ty.default_drop_sound)(w, id))
    } else {
        raw
    }
}

/// `DropSound` setter.
pub fn set_drop_sound(w: &mut World, id: ItemId, value: i32) {
    if let Some(c) = state_mut(w, id) {
        c.drop_sound = value;
    }
}

/// `LiftOverride`.
pub fn lift_override(w: &World, id: ItemId) -> bool {
    state(w, id).is_some_and(|c| c.lift_override)
}

/// `LiftOverride` setter.
pub fn set_lift_override(w: &mut World, id: ItemId, value: bool) {
    if let Some(c) = state_mut(w, id) {
        c.lift_override = value;
    }
}

/// `Openers`.
pub fn openers(w: &World, id: ItemId) -> Option<&Vec<MobileId>> {
    state(w, id)?.openers.as_ref()
}

/// `MaxWeight` (virtual).
pub fn max_weight(w: &World, id: ItemId) -> i32 {
    w.item(id).map_or(0, |i| (i.ty.max_weight)(w, id))
}

/// `IsPublicContainer` (virtual).
pub fn is_public_container(w: &World, id: ItemId) -> bool {
    w.item(id)
        .is_some_and(|i| (i.ty.is_public_container)(w, id))
}

/// `UpdateContentsDecayRegistration()`.
pub fn update_contents_decay_registration(w: &mut World, id: ItemId) {
    let items = w.item(id).map(|i| i.items.clone()).unwrap_or_default();
    for item in items {
        ops::update_decay_registration(w, item);
    }
}

pub fn is_child_public_default(w: &World, id: ItemId, _child: ItemId) -> bool {
    is_public_container(w, id)
}

pub fn contents_decay_default(w: &World, id: ItemId) -> bool {
    ops::is_locked_down(w, id) && !ops::is_secure(w, id)
}

pub fn is_deco_container_default(w: &World, id: ItemId) -> bool {
    let Some(i) = w.item(id) else { return false };
    !i.movable()
        && !ops::is_locked_down(w, id)
        && !ops::is_secure(w, id)
        && i.parent.is_none()
        && !lift_override(w, id)
}

pub fn max_weight_default(w: &World, id: ItemId) -> i32 {
    let Some(i) = w.item(id) else { return 0 };
    if let Some(EntityId::Item(p)) = i.parent
        && let Some(pt) = w.item(p).map(|p| p.ty)
        && pt.is(&CONTAINER)
        && (pt.max_weight)(w, p) == 0
    {
        return 0;
    }
    (i.ty.default_max_weight)(w, id)
}

pub fn default_max_items_default(w: &World, _id: ItemId) -> i32 {
    global_max_items(w)
}

pub fn default_max_weight_default(w: &World, _id: ItemId) -> i32 {
    global_max_weight(w)
}

pub fn default_gump_id_default(w: &World, id: ItemId) -> i32 {
    container_data(w, id).gump_id
}

pub fn default_drop_sound_default(w: &World, id: ItemId) -> i32 {
    container_data(w, id).drop_sound
}

pub fn bounds_default(w: &World, id: ItemId) -> Rectangle2D {
    container_data(w, id).bounds
}

/// `UpdateContainerData()`.
pub fn update_container_data_default(w: &mut World, id: ItemId) {
    ensure_container_data_loaded(w);
    let Some(item_id) = w.item(id).map(|i| i.item_id) else {
        return;
    };
    let data = get_data(w, item_id);
    if let Some(c) = state_mut(w, id) {
        c.data = Some(data);
    }
}

/// `GetDroppedSound(item)`.
pub fn get_dropped_sound_default(w: &World, id: ItemId, item: ItemId) -> i32 {
    let drop_sound_of_item = w.item(item).map_or(-1, |i| (i.ty.get_drop_sound)(w, item));
    if drop_sound_of_item != -1 {
        drop_sound_of_item
    } else {
        drop_sound(w, id)
    }
}

/// `GetDroppedSound(item)` through the vtable.
pub fn get_dropped_sound(w: &World, id: ItemId, item: ItemId) -> i32 {
    w.item(id)
        .map_or(-1, |i| (i.ty.get_dropped_sound)(w, id, item))
}

// ---- Overrides of Item virtuals ----

/// `ItemID` setter: a new graphic drops the cached `ContainerData`.
fn set_item_id(w: &mut World, id: ItemId, value: i32) {
    let old_id = w.item(id).map_or(0, |i| i.item_id);
    crate::item::base::set_item_id_base(w, id, value);
    if w.item(id).map_or(0, |i| i.item_id) != old_id
        && let Some(c) = state_mut(w, id)
    {
        c.data = None;
    }
}

fn on_snoop(w: &mut World, id: ItemId, from: MobileId) {
    let handler = w
        .resources
        .get::<ContainerStatics>()
        .and_then(|s| s.snoop_handler);
    if let Some(h) = handler {
        h(w, id, from);
    }
}

fn check_lift(
    w: &mut World,
    id: ItemId,
    from: MobileId,
    item: ItemId,
    reject: &mut LRReason,
) -> bool {
    let is_deco = w.item(id).is_some_and(|i| (i.ty.is_deco_container)(w, id));
    let low_access = w
        .mobile(from)
        .is_some_and(|m| m.access_level < AccessLevel::GameMaster);
    if low_access && is_deco {
        *reject = LRReason::CannotLift;
        return false;
    }
    crate::item::base::check_lift_base(w, id, from, item, reject)
}

fn check_item_use(w: &mut World, id: ItemId, from: MobileId, item: ItemId) -> bool {
    let is_deco = w.item(id).is_some_and(|i| (i.ty.is_deco_container)(w, id));
    let low_access = w
        .mobile(from)
        .is_some_and(|m| m.access_level < AccessLevel::GameMaster);
    if item != id && low_access && is_deco {
        deps::local_overhead_message(w, from, MessageType::REGULAR, 0x3B2, 1019045); // I can't reach that.
        return false;
    }
    crate::item::base::check_item_use_base(w, id, from, item)
}

fn get_total(w: &World, id: ItemId, ty: TotalType) -> i32 {
    let c = state(w, id);
    match ty {
        TotalType::Gold => c.map_or(0, |c| c.total_gold),
        TotalType::Items => c.map_or(0, |c| c.total_items),
        TotalType::Weight => c.map_or(0, |c| c.total_weight),
    }
}

fn update_total(w: &mut World, id: ItemId, sender: ItemId, ty: TotalType, delta: i32) {
    let virtual_sender = w
        .item(sender)
        .is_some_and(|s| (s.ty.is_virtual_item)(w, sender));
    if sender != id && delta != 0 && !virtual_sender {
        match ty {
            TotalType::Gold => {
                if let Some(c) = state_mut(w, id) {
                    c.total_gold += delta;
                }
            }
            TotalType::Items => {
                if let Some(c) = state_mut(w, id) {
                    c.total_items += delta;
                }
                ops::invalidate_properties(w, id);
            }
            TotalType::Weight => {
                if let Some(c) = state_mut(w, id) {
                    c.total_weight += delta;
                }
                ops::invalidate_properties(w, id);
            }
        }
    }
    crate::item::base::update_total_base(w, id, sender, ty, delta);
}

fn update_totals(w: &mut World, id: ItemId) {
    if let Some(c) = state_mut(w, id) {
        c.total_gold = 0;
        c.total_items = 0;
        c.total_weight = 0;
    }

    let items = w.item(id).map(|i| i.items.clone()).unwrap_or_default();
    let (mut gold, mut count, mut weight) = (0, 0, 0);
    for item in items {
        let Some(ty) = w.item(item).map(|i| i.ty) else {
            continue;
        };
        (ty.update_totals)(w, item);

        if (ty.is_virtual_item)(w, item) {
            continue;
        }

        gold += ops::get_total(w, item, TotalType::Gold);
        count += ops::get_total(w, item, TotalType::Items) + 1;
        weight += ops::get_total(w, item, TotalType::Weight) + ops::pile_weight(w, item);
    }
    if let Some(c) = state_mut(w, id) {
        c.total_gold = gold;
        c.total_items = count;
        c.total_weight = weight;
    }
}

fn on_item_added(w: &mut World, id: ItemId, item: ItemId) {
    crate::item::base::on_item_added_base(w, id, item);
    if let Some(i) = w.item_mut(id) {
        i.version += 1;
    }
}

fn on_item_removed(w: &mut World, id: ItemId, item: ItemId) {
    crate::item::base::on_item_removed_base(w, id, item);
    if let Some(i) = w.item_mut(id) {
        i.version += 1;
    }
}

fn on_drag_drop(w: &mut World, id: ItemId, from: MobileId, dropped: ItemId) -> bool {
    let Some(ty) = w.item(id).map(|i| i.ty) else {
        return false;
    };
    if (ty.try_drop_item)(w, id, Some(from), dropped, true, false) {
        let sound = get_dropped_sound(w, id, dropped);
        let loc = ops::get_world_location(w, id);
        deps::send_sound(w, from, sound, loc);
        return true;
    }
    false
}

fn on_double_click_secure_trade(w: &mut World, id: ItemId, from: MobileId) {
    let loc = ops::get_world_location(w, id);
    let Some(ty) = w.item(id).map(|i| i.ty) else {
        return;
    };
    if deps::mobile_in_range(w, from, loc, 2) {
        (ty.display_to)(w, id, from);

        let trade = crate::item::secure_trade_container::trade_of(w, id);
        if let Some(t) = trade {
            let (tf, tt) = deps::trade_parties(w, t);
            if tf == Some(from) {
                if let Some(to) = tt {
                    (ty.display_to)(w, id, to);
                }
            } else if tt == Some(from)
                && let Some(f) = tf
            {
                (ty.display_to)(w, id, f);
            }
        }
    } else {
        deps::send_localized_message(w, from, 500446); // That is too far away.
    }
}

fn on_single_click(w: &mut World, id: ItemId, from: MobileId) {
    crate::item::base::on_single_click_base(w, id, from);

    let Some(ty) = w.item(id).map(|i| i.ty) else {
        return;
    };
    if (ty.check_content_display)(w, id, from) {
        let items = ops::get_total(w, id, TotalType::Items);
        let weight = ops::get_total(w, id, TotalType::Weight);
        let text = if items == 1 {
            format!("({items} item, {weight} stones)")
        } else {
            format!("({items} items, {weight} stones)")
        };
        ops::label_to(w, id, from, &text);
    }
}

fn on_delete(w: &mut World, id: ItemId) {
    crate::item::base::on_delete_base(w, id);
    if let Some(c) = state_mut(w, id) {
        c.openers = None;
    }
}

fn get_properties(w: &World, id: ItemId, list: &mut PropertyList) {
    crate::item::base::get_properties_base(w, id, list);

    let Some(ty) = w.item(id).map(|i| i.ty) else {
        return;
    };
    if (ty.displays_content)(w, id) {
        let total_items = ops::get_total(w, id, TotalType::Items);
        let total_weight = ops::get_total(w, id, TotalType::Weight);
        if deps::core_ml(w) {
            if ops::parents_contain(w, id, &crate::item::containers::BANK_BOX) {
                // Root Parent is the Mobile. Parent could be another container.
                list.add_number_fmt(
                    1073841, // Contents: ~1_COUNT~/~2_MAXCOUNT~ items, ~3_WEIGHT~ stones
                    format_args!("{}\t{}\t{}", total_items, max_items(w, id), total_weight),
                );
            } else {
                list.add_number_fmt(
                    1072241, // Contents: ~1_COUNT~/~2_MAXCOUNT~ items, ~3_WEIGHT~/~4_MAXWEIGHT~ stones
                    format_args!(
                        "{}\t{}\t{}\t{}",
                        total_items,
                        max_items(w, id),
                        total_weight,
                        max_weight(w, id)
                    ),
                );
            }
            // TODO: Where do the other clilocs come into play? 1073839 & 1073840?
        } else {
            // ~1_COUNT~ items, ~2_WEIGHT~ stones
            list.add_number_fmt(1050044, format_args!("{}\t{}", total_items, total_weight));
        }
    }
}

fn on_double_click(w: &mut World, id: ItemId, from: MobileId) {
    let Some(ty) = w.item(id).map(|i| i.ty) else {
        return;
    };
    let loc = ops::get_world_location(w, id);
    let staff = w
        .mobile(from)
        .is_some_and(|m| m.access_level > AccessLevel::Player);
    if staff || deps::mobile_in_range(w, from, loc, 2) {
        (ty.display_to)(w, id, from);
    } else {
        deps::send_localized_message(w, from, 500446); // That is too far away.
    }
}

// ---- Container's own virtuals ----

/// `CheckHold(m, item, message)`: counts items.
pub fn check_hold(
    w: &mut World,
    id: ItemId,
    m: Option<MobileId>,
    item: ItemId,
    message: bool,
) -> bool {
    check_hold_items(w, id, m, item, message, true)
}

/// `CheckHold(m, item, message, checkItems)`.
pub fn check_hold_items(
    w: &mut World,
    id: ItemId,
    m: Option<MobileId>,
    item: ItemId,
    message: bool,
    check_items: bool,
) -> bool {
    let Some(ty) = w.item(id).map(|i| i.ty) else {
        return false;
    };
    (ty.check_hold)(w, id, m, item, message, check_items, 0, 0)
}

#[allow(clippy::too_many_arguments)]
pub fn check_hold_default(
    w: &mut World,
    id: ItemId,
    m: Option<MobileId>,
    item: ItemId,
    message: bool,
    check_items: bool,
    plus_items: i32,
    plus_weight: i32,
) -> bool {
    let Some(ty) = w.item(id).map(|i| i.ty) else {
        return false;
    };
    let staff = m.is_some_and(|m| {
        w.mobile(m)
            .is_some_and(|mob| mob.access_level >= AccessLevel::GameMaster)
    });

    if !staff {
        if (ty.is_deco_container)(w, id) {
            if message && let Some(m) = m {
                (ty.send_cant_store_message)(w, id, m, item);
            }
            return false;
        }

        let max_items = max_items(w, id);
        let item_virtual = w
            .item(item)
            .is_some_and(|i| (i.ty.is_virtual_item)(w, item));

        if check_items
            && max_items != 0
            && ops::get_total(w, id, TotalType::Items)
                + plus_items
                + ops::get_total(w, item, TotalType::Items)
                + i32::from(!item_virtual)
                > max_items
        {
            if message && let Some(m) = m {
                (ty.send_full_items_message)(w, id, m, item);
            }
            return false;
        }

        let max_weight = (ty.max_weight)(w, id);
        if max_weight != 0
            && ops::get_total(w, id, TotalType::Weight)
                + plus_weight
                + ops::get_total(w, item, TotalType::Weight)
                + ops::pile_weight(w, item)
                > max_weight
        {
            if message && let Some(m) = m {
                (ty.send_full_weight_message)(w, id, m, item);
            }
            return false;
        }
    }

    let mut parent = w.item(id).and_then(|i| i.parent);
    while let Some(EntityId::Item(p)) = parent {
        let Some(pt) = w.item(p).map(|i| i.ty) else {
            break;
        };
        if pt.is(&CONTAINER) {
            return (pt.check_hold)(w, p, m, item, message, check_items, plus_items, plus_weight);
        }
        parent = w.item(p).and_then(|i| i.parent);
    }

    true
}

pub fn send_full_items_message_default(w: &mut World, _id: ItemId, to: MobileId, _item: ItemId) {
    // That container cannot hold more items.
    deps::send_localized_message(w, to, 1080017);
}

pub fn send_full_weight_message_default(w: &mut World, _id: ItemId, to: MobileId, _item: ItemId) {
    // That container cannot hold more weight.
    deps::send_localized_message(w, to, 1080016);
}

pub fn send_cant_store_message_default(w: &mut World, _id: ItemId, to: MobileId, _item: ItemId) {
    deps::send_localized_message(w, to, 500176); // That is not your container, you can't store things here.
}

pub fn on_drag_drop_into_default(
    w: &mut World,
    id: ItemId,
    from: MobileId,
    item: ItemId,
    p: Point3D,
) -> bool {
    if !check_hold_items(w, id, Some(from), item, true, true) {
        return false;
    }

    ops::set_location(w, item, Point3D::new(p.x, p.y, 0));
    if let Some(ty) = w.item(id).map(|i| i.ty) {
        (ty.add_item)(w, id, item);
    }

    let sound = get_dropped_sound(w, id, item);
    let loc = ops::get_world_location(w, id);
    deps::send_sound(w, from, sound, loc);

    true
}

pub fn on_stack_attempt_default(
    w: &mut World,
    id: ItemId,
    from: MobileId,
    stack: ItemId,
    dropped: ItemId,
) -> bool {
    check_hold_items(w, id, Some(from), dropped, true, false)
        && w.item(stack)
            .map(|i| i.ty)
            .is_some_and(|t| (t.stack_with)(w, stack, Some(from), dropped, true))
}

pub fn try_drop_item_default(
    w: &mut World,
    id: ItemId,
    from: Option<MobileId>,
    dropped: ItemId,
    send_full_message: bool,
    play_sound: bool,
) -> bool {
    let Some(ty) = w.item(id).map(|i| i.ty) else {
        return false;
    };
    let list = w.item(id).map(|i| i.items.clone()).unwrap_or_default();

    for item in list {
        let Some(it) = w.item(item).map(|i| i.ty) else {
            continue;
        };
        if !it.is(&CONTAINER)
            && (ty.check_hold)(w, id, from, dropped, false, false, 0, 0)
            && (it.stack_with)(w, item, from, dropped, play_sound)
        {
            return true;
        }
    }

    if (ty.check_hold)(w, id, from, dropped, send_full_message, true, 0, 0) {
        (ty.drop_item)(w, id, dropped);
        return true;
    }

    false
}

/// `TryDropItem(from, dropped, sendFullMessage)`.
pub fn try_drop_item(
    w: &mut World,
    id: ItemId,
    from: Option<MobileId>,
    dropped: ItemId,
    send_full_message: bool,
) -> bool {
    match w.item(id).map(|i| i.ty) {
        Some(ty) => (ty.try_drop_item)(w, id, from, dropped, send_full_message, false),
        None => false,
    }
}

pub fn destroy_default(w: &mut World, id: ItemId) {
    let loc = ops::get_world_location(w, id);
    let map = w.item(id).and_then(|i| i.map);

    let mut i = w.item(id).map_or(0, |i| i.items.len());
    while i > 0 {
        i -= 1;
        if let Some(child) = w.item(id).and_then(|c| c.items.get(i).copied()) {
            ops::move_to_world_map(w, child, loc, map);
        }
    }

    if let Some(ty) = w.item(id).map(|i| i.ty) {
        (ty.delete)(w, id);
    }
}

pub fn drop_item_default(w: &mut World, id: ItemId, dropped: ItemId) {
    if w.item(dropped).is_none() {
        return;
    }
    let Some(ty) = w.item(id).map(|i| i.ty) else {
        return;
    };
    (ty.add_item)(w, id, dropped);

    crate::item::bounds::ensure_loaded(w);
    let bounds = ops::get_graphic_bounds(w, dropped);
    let our_bounds = (ty.bounds)(w, id);

    let x = if bounds.width() >= our_bounds.width() {
        (our_bounds.width() - bounds.width()) / 2
    } else {
        utility::random(&mut w.rng, our_bounds.width() - bounds.width())
    };
    let y = if bounds.height() >= our_bounds.height() {
        (our_bounds.height() - bounds.height()) / 2
    } else {
        utility::random(&mut w.rng, our_bounds.height() - bounds.height())
    };

    let x = x + our_bounds.x() - bounds.x();
    let y = y + our_bounds.y() - bounds.y();

    ops::set_location(w, dropped, Point3D::new(x, y, 0));
}

pub fn check_content_display_default(w: &World, id: ItemId, from: MobileId) -> bool {
    let Some(ty) = w.item(id).map(|i| i.ty) else {
        return false;
    };
    let root = ops::root_parent(w, id);
    ((ty.displays_content)(w, id) && root.is_none())
        || matches!(root, Some(EntityId::Item(_)))
        || root == Some(EntityId::Mobile(from))
        || w.mobile(from)
            .is_some_and(|m| m.access_level > AccessLevel::Player)
}

pub fn display_to_default(w: &mut World, id: ItemId, to: MobileId) {
    process_openers(w, id, to);

    let Some(ns) = deps::net_state_of(w, to) else {
        return;
    };
    let Some(ty) = w.item(id).map(|i| i.ty) else {
        return;
    };
    let gump = gump_id(w, id);
    crate::packets::container::send_display_container(w, ns, id.0, gump);

    (ty.send_content_to)(w, id, ns);

    if deps::opl_enabled(w) {
        let items = w.item(id).map(|i| i.items.clone()).unwrap_or_default();
        for item in items {
            crate::packets::entity::send_opl_info_item(w, ns, item);
        }
    }
}

/// `ProcessOpeners(opener)`.
pub fn process_openers(w: &mut World, id: ItemId, opener: MobileId) {
    if is_public_container(w, id) {
        return;
    }

    let mut contains = false;

    let current = state(w, id).and_then(|c| c.openers.clone());
    if let Some(list) = current {
        let world_loc = ops::get_world_location(w, id);
        let map = w.item(id).and_then(|i| i.map);
        let ty = w.item(id).map(|i| i.ty);
        let mut kept = Vec::with_capacity(list.len());

        for mob in list {
            if mob == opener {
                contains = true;
                kept.push(mob);
            } else {
                let range = ty.map_or(18, |t| (t.get_update_range)(w, id, mob));
                let same_map = w.mobile(mob).is_some_and(|m| m.map == map);
                if !same_map || !deps::mobile_in_range(w, mob, world_loc, range) {
                    continue;
                }
                kept.push(mob);
            }
        }
        if let Some(c) = state_mut(w, id) {
            c.openers = Some(kept);
        }
    }

    if !contains {
        if let Some(c) = state_mut(w, id) {
            c.openers.get_or_insert_with(Vec::new).push(opener);
        }
    } else if let Some(c) = state_mut(w, id)
        && c.openers.as_ref().is_some_and(|o| o.is_empty())
    {
        c.openers = None;
    }
}

pub fn send_content_to_default(w: &mut World, id: ItemId, ns: NetStateId) {
    if let Some(m) = w.net_mobile(ns) {
        crate::packets::container::send_container_content(w, ns, m, id);
    }
}

// ---- Searching and consuming ----

/// `OnItemConsumed`.
pub type OnItemConsumed<'a> = &'a mut dyn FnMut(&mut World, ItemId, i32);

/// `CheckItemGroup`: 0 when two items belong to the same group.
pub type CheckItemGroup<'a> = &'a dyn Fn(&World, ItemId, ItemId) -> i32;

fn list_items_by_type(w: &World, id: ItemId, ty: &'static ItemType, recurse: bool) -> Vec<ItemId> {
    crate::item::enumerable::find_items_by_type(w, id, recurse, ty, |_, _| true)
}

fn list_items_by_types(
    w: &World,
    id: ItemId,
    types: &[&'static ItemType],
    recurse: bool,
) -> Vec<ItemId> {
    crate::item::enumerable::find_items_by_types(w, id, recurse, types)
}

fn amount_of(w: &World, id: ItemId) -> i32 {
    w.item(id).map_or(0, |i| i.amount)
}

/// Walks `items` (a breadth first snapshot) in adjacency-based groups defined by `grouper`. Returns the first
/// group whose amount sum is at least `amount` as `(start, length)`.
fn try_find_group_meeting_amount(
    w: &World,
    items: &[ItemId],
    amount: i32,
    grouper: CheckItemGroup,
) -> Option<(usize, usize)> {
    let mut i = 0;
    while i < items.len() {
        let leader = items[i];
        let start = i;
        let mut total = amount_of(w, leader);
        i += 1;
        while i < items.len() && grouper(w, leader, items[i]) == 0 {
            total += amount_of(w, items[i]);
            i += 1;
        }
        if total >= amount {
            return Some((start, i - start));
        }
    }
    None
}

/// The largest group sum across `items` partitioned by `grouper`.
fn best_group_total(w: &World, items: &[ItemId], grouper: CheckItemGroup) -> i32 {
    let mut best = 0;
    let mut i = 0;
    while i < items.len() {
        let leader = items[i];
        let mut total = amount_of(w, leader);
        i += 1;
        while i < items.len() && grouper(w, leader, items[i]) == 0 {
            total += amount_of(w, items[i]);
            i += 1;
        }
        best = best.max(total);
    }
    best
}

/// Consumes `need` units from the slice of `items`, firing `callback` once per item touched with the actual
/// delta. Items with an amount no larger than the delta are deleted through `Item.Consume`.
fn consume_slice(
    w: &mut World,
    items: &[ItemId],
    start: usize,
    length: usize,
    mut need: i32,
    mut callback: Option<OnItemConsumed>,
) {
    let mut k = 0;
    while k < length && need > 0 {
        let item = items[start + k];
        let their_amount = amount_of(w, item);
        let ty = w.item(item).map(|i| i.ty);
        if their_amount < need {
            if let Some(cb) = callback.as_mut() {
                cb(w, item, their_amount);
            }
            if let Some(t) = ty {
                (t.consume)(w, item, their_amount);
            }
            need -= their_amount;
        } else {
            if let Some(cb) = callback.as_mut() {
                cb(w, item, need);
            }
            if let Some(t) = ty {
                (t.consume)(w, item, need);
            }
            return;
        }
        k += 1;
    }
}

/// `HasAmount(type, amount, recurse)`.
fn has_amount(w: &World, id: ItemId, ty: &'static ItemType, amount: i32, recurse: bool) -> bool {
    let mut total = 0;
    for item in list_items_by_type(w, id, ty, recurse) {
        total += amount_of(w, item);
        if total >= amount {
            return true;
        }
    }
    false
}

/// `ConsumeTotalGrouped(Type[] types, amounts, recurse, callback, grouper)`: returns the index of the first
/// slot that cannot be met, or -1 when everything was consumed.
pub fn consume_total_grouped(
    w: &mut World,
    id: ItemId,
    types: &[&'static ItemType],
    amounts: &[i32],
    recurse: bool,
    mut callback: Option<OnItemConsumed>,
    grouper: CheckItemGroup,
) -> i32 {
    assert_eq!(
        types.len(),
        amounts.len(),
        "length of types and amounts must match"
    );

    // Phase 1: every slot must have a single group with sum >= amount before any consume happens (preserves
    // all-or-nothing semantics).
    for (i, ty) in types.iter().enumerate() {
        let items = list_items_by_type(w, id, ty, recurse);
        if try_find_group_meeting_amount(w, &items, amounts[i], grouper).is_none() {
            return i as i32;
        }
    }

    // Phase 2: re-list and consume. Live mutation of the container during `Consume` bumps its version, so
    // each phase walks its own snapshot.
    for (i, ty) in types.iter().enumerate() {
        let items = list_items_by_type(w, id, ty, recurse);
        if let Some((start, len)) = try_find_group_meeting_amount(w, &items, amounts[i], grouper) {
            consume_slice(
                w,
                &items,
                start,
                len,
                amounts[i],
                callback.as_deref_mut().map(|c| c as _),
            );
        }
    }

    -1
}

/// `ConsumeTotalGrouped(Type[][] types, ...)`: each slot accepts any of several types.
pub fn consume_total_grouped_any(
    w: &mut World,
    id: ItemId,
    types: &[&[&'static ItemType]],
    amounts: &[i32],
    recurse: bool,
    mut callback: Option<OnItemConsumed>,
    grouper: CheckItemGroup,
) -> i32 {
    assert_eq!(
        types.len(),
        amounts.len(),
        "length of types and amounts must match"
    );

    for (i, t) in types.iter().enumerate() {
        let items = list_items_by_types(w, id, t, recurse);
        if try_find_group_meeting_amount(w, &items, amounts[i], grouper).is_none() {
            return i as i32;
        }
    }

    for (i, t) in types.iter().enumerate() {
        let items = list_items_by_types(w, id, t, recurse);
        if let Some((start, len)) = try_find_group_meeting_amount(w, &items, amounts[i], grouper) {
            consume_slice(
                w,
                &items,
                start,
                len,
                amounts[i],
                callback.as_deref_mut().map(|c| c as _),
            );
        }
    }

    -1
}

/// `ConsumeTotal(Type[][] types, amounts, recurse, callback)`.
pub fn consume_total_any(
    w: &mut World,
    id: ItemId,
    types: &[&[&'static ItemType]],
    amounts: &[i32],
    recurse: bool,
    mut callback: Option<OnItemConsumed>,
) -> i32 {
    assert_eq!(
        types.len(),
        amounts.len(),
        "length of types and amounts must match"
    );

    // Phase 1: validate every slot before any consume (all-or-nothing).
    for (i, t) in types.iter().enumerate() {
        if get_amount_types(w, id, t, recurse) < amounts[i] {
            return i as i32;
        }
    }

    // Phase 2: materialize per slot and consume.
    for (i, t) in types.iter().enumerate() {
        let items = list_items_by_types(w, id, t, recurse);
        consume_slice(
            w,
            &items,
            0,
            items.len(),
            amounts[i],
            callback.as_deref_mut().map(|c| c as _),
        );
    }

    -1
}

/// `ConsumeTotal(ReadOnlySpan<Type> types, amounts, recurse, callback)`.
pub fn consume_total_types(
    w: &mut World,
    id: ItemId,
    types: &[&'static ItemType],
    amounts: &[i32],
    recurse: bool,
    mut callback: Option<OnItemConsumed>,
) -> i32 {
    assert_eq!(
        types.len(),
        amounts.len(),
        "length of types and amounts must match"
    );

    for (i, ty) in types.iter().enumerate() {
        if get_amount(w, id, ty, recurse) < amounts[i] {
            return i as i32;
        }
    }

    for (i, ty) in types.iter().enumerate() {
        let items = list_items_by_type(w, id, ty, recurse);
        consume_slice(
            w,
            &items,
            0,
            items.len(),
            amounts[i],
            callback.as_deref_mut().map(|c| c as _),
        );
    }

    -1
}

/// `ConsumeTotal(Type type, amount, recurse, callback)`.
pub fn consume_total(
    w: &mut World,
    id: ItemId,
    ty: &'static ItemType,
    amount: i32,
    recurse: bool,
    callback: Option<OnItemConsumed>,
) -> bool {
    if !has_amount(w, id, ty, amount, recurse) {
        return false;
    }
    let items = list_items_by_type(w, id, ty, recurse);
    consume_slice(w, &items, 0, items.len(), amount, callback);
    true
}

/// `ConsumeUpTo(type, amount, recurse)`: returns how much was consumed.
pub fn consume_up_to(
    w: &mut World,
    id: ItemId,
    ty: &'static ItemType,
    amount: i32,
    recurse: bool,
) -> i32 {
    let mut consumed = 0;
    let mut to_delete = Vec::new();

    recurse_consume_up_to(w, id, ty, amount, recurse, &mut consumed, &mut to_delete);

    for item in to_delete {
        if let Some(t) = w.item(item).map(|i| i.ty) {
            (t.delete)(w, item);
        }
    }

    consumed
}

fn recurse_consume_up_to(
    w: &mut World,
    current: ItemId,
    ty: &'static ItemType,
    amount: i32,
    recurse: bool,
    consumed: &mut i32,
    to_delete: &mut Vec<ItemId>,
) {
    let list = match w.item(current) {
        Some(c) if !c.items.is_empty() => c.items.clone(),
        _ => return,
    };

    for item in list {
        let Some(it_ty) = w.item(item).map(|i| i.ty) else {
            continue;
        };
        if it_ty.is(ty) {
            let need = amount - *consumed;
            let their_amount = amount_of(w, item);

            if their_amount <= need {
                to_delete.push(item);
                *consumed += their_amount;
            } else {
                ops::set_amount(w, item, their_amount - need);
                *consumed += need;
                return;
            }
        } else if recurse && it_ty.is(&CONTAINER) {
            recurse_consume_up_to(w, item, ty, amount, true, consumed, to_delete);
        }
    }
}

/// `GetBestGroupAmount(types, recurse, grouper)`.
pub fn get_best_group_amount(
    w: &World,
    id: ItemId,
    types: &[&'static ItemType],
    recurse: bool,
    grouper: CheckItemGroup,
) -> i32 {
    let items = list_items_by_types(w, id, types, recurse);
    best_group_total(w, &items, grouper)
}

/// `GetAmount(Type type, recurse)`.
pub fn get_amount(w: &World, id: ItemId, ty: &'static ItemType, recurse: bool) -> i32 {
    crate::item::enumerable::find_items(w, id, recurse, |w, i| {
        w.item(i).is_some_and(|it| it.ty.is(ty))
    })
    .into_iter()
    .map(|i| amount_of(w, i))
    .sum()
}

/// `GetAmount(Type[] types, recurse)`.
pub fn get_amount_types(w: &World, id: ItemId, types: &[&'static ItemType], recurse: bool) -> i32 {
    list_items_by_types(w, id, types, recurse)
        .into_iter()
        .map(|i| amount_of(w, i))
        .sum()
}

/// `FindItemByType(Type type, recurse)`.
pub fn find_item_by_type(
    w: &World,
    id: ItemId,
    ty: &'static ItemType,
    recurse: bool,
) -> Option<ItemId> {
    crate::item::enumerable::find_item_by_type(w, id, recurse, ty)
}

/// `FindItemByType(Type[] types, recurse)`.
pub fn find_item_by_types(
    w: &World,
    id: ItemId,
    types: &[&'static ItemType],
    recurse: bool,
) -> Option<ItemId> {
    list_items_by_types(w, id, types, recurse)
        .into_iter()
        .next()
}

// ---- Serialization ----

/// The generated `[SerializationGenerator(0, false)]` save flags of `Container`.
mod save_flag {
    pub const MAX_ITEMS: u8 = 0x01;
    pub const GUMP_ID: u8 = 0x02;
    pub const DROP_SOUND: u8 = 0x04;
    pub const LIFT_OVERRIDE: u8 = 0x08;
}

/// The generated `Container.Serialize`: `Item`'s part, then this class's.
pub fn serialize(w: &World, id: ItemId, writer: &mut dyn GenericWriter) {
    (ITEM_BASE.serialize)(w, id, writer);
    serialize_part(w, id, writer);
}

/// Only the part `Container` adds (what a subclass's `serialize` chain follows).
pub fn serialize_part(w: &World, id: ItemId, writer: &mut dyn GenericWriter) {
    let c = state(w, id).cloned().unwrap_or_default();
    writer.write_i32(0); // version

    let mut flags = 0u8;
    if c.max_items != -1 {
        flags |= save_flag::MAX_ITEMS;
    }
    if c.gump_id != -1 {
        flags |= save_flag::GUMP_ID;
    }
    if c.drop_sound != -1 {
        flags |= save_flag::DROP_SOUND;
    }
    if c.lift_override {
        flags |= save_flag::LIFT_OVERRIDE;
    }
    writer.write_encoded_int(i32::from(flags)); // WriteEnum of a 4-byte enum

    if flags & save_flag::MAX_ITEMS != 0 {
        writer.write_encoded_int(c.max_items);
    }
    if flags & save_flag::GUMP_ID != 0 {
        writer.write_encoded_int(c.gump_id);
    }
    if flags & save_flag::DROP_SOUND != 0 {
        writer.write_encoded_int(c.drop_sound);
    }
    // LiftOverride is a flag only: the generator writes no data for it
}

/// The generated `Container.Deserialize` plus `[AfterDeserialization] UpdateContainerData`.
pub fn deserialize(w: &mut World, id: ItemId, reader: &mut dyn GenericReader) -> Result<()> {
    (ITEM_BASE.deserialize)(w, id, reader)?;
    deserialize_part(w, id, reader)?;
    update_container_data_default(w, id);
    Ok(())
}

/// Only the part `Container` adds.
pub fn deserialize_part(w: &mut World, id: ItemId, reader: &mut dyn GenericReader) -> Result<()> {
    let _version = reader.read_i32()?;
    let flags = reader.read_encoded_int()? as u8;

    let max_items = if flags & save_flag::MAX_ITEMS != 0 {
        reader.read_encoded_int()?
    } else {
        -1
    };
    let gump_id = if flags & save_flag::GUMP_ID != 0 {
        reader.read_encoded_int()?
    } else {
        -1
    };
    let drop_sound = if flags & save_flag::DROP_SOUND != 0 {
        reader.read_encoded_int()?
    } else {
        -1
    };
    let lift_override = flags & save_flag::LIFT_OVERRIDE != 0;

    if let Some(c) = state_mut(w, id) {
        c.max_items = max_items;
        c.gump_id = gump_id;
        c.drop_sound = drop_sound;
        c.lift_override = lift_override;
    }
    Ok(())
}

// ---- Constructor ----

/// `new Container(itemID)` of class `ty`: an item whose `_gumpID`, `_dropSound` and `_maxItems` are -1.
pub fn new_container(w: &mut World, ty: &'static ItemType, item_id: i32) -> ItemId {
    let id = ops::new_item_with_id(w, ty, item_id);
    if let Some(i) = w.item_mut(id) {
        *i.container_mut() = ContainerState::default();
    }
    id
}

/// What `Item.OnParentDeath` and friends do not override for containers (kept for BankBox).
pub fn remain_equipped(_w: &mut World, _id: ItemId, _parent: MobileId) -> DeathMoveResult {
    DeathMoveResult::RemainEquipped
}
