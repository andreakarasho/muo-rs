//! The equipment list and the cursor (`AddItem`, `RemoveItem`, `FindItemOnLayer`, `DropHolding`,
//! `PlaceInBackpack`, `LiftItemDupe`, `ConvertHair`..).

use muo_primitives::Point3D;

use super::deps;
use crate::ids::{EntityId, ItemId, MobileId};
use crate::item::{ItemDelta, TotalType};
use crate::layer::Layer;
use crate::map::MapId;
use crate::world::World;

pub fn add_item(w: &mut World, id: MobileId, item: ItemId) {
    let Some(it) = w.item(item) else { return };
    if it.deleted() {
        return;
    }
    let ity = it.ty;
    let parent = it.parent;

    if parent == Some(EntityId::Mobile(id)) {
        return;
    }

    match parent {
        Some(EntityId::Mobile(pm)) => remove_item(w, pm, item),
        Some(EntityId::Item(pi)) => {
            if let Some(pty) = w.item(pi).map(|p| p.ty) {
                (pty.remove_item)(w, pi, item);
            }
        }
        None => deps::item_send_remove_packet(w, item),
    }

    let Some(m) = w.mobile(id) else { return };
    let (mty, map) = (m.ty, m.map);
    if let Some(it) = w.item_mut(item) {
        it.parent = Some(EntityId::Mobile(id));
    }
    deps::item_set_map(w, item, map);

    if let Some(m) = w.mobile_mut(id) {
        m.items.push(item);
    }

    if !(ity.is_virtual_item)(w, item) {
        let gold = (ity.get_total)(w, item, TotalType::Gold);
        (mty.update_total)(w, id, item, TotalType::Gold, gold);
        let items = (ity.get_total)(w, item, TotalType::Items);
        (mty.update_total)(w, id, item, TotalType::Items, items + 1);
        let weight = (ity.get_total)(w, item, TotalType::Weight) + deps::item_pile_weight(w, item);
        (mty.update_total)(w, id, item, TotalType::Weight, weight);
    }

    deps::item_delta(w, item, ItemDelta::UPDATE);

    (ity.on_added)(w, item, EntityId::Mobile(id));
    (mty.on_item_added)(w, id, item);

    if has_resistance(w, item) {
        (mty.update_resistances)(w, id);
    }
}

/// Any of the five resistances of `item` is not 0.
fn has_resistance(w: &World, item: ItemId) -> bool {
    let Some(ity) = w.item(item).map(|i| i.ty) else {
        return false;
    };
    (ity.physical_resistance)(w, item) != 0
        || (ity.fire_resistance)(w, item) != 0
        || (ity.cold_resistance)(w, item) != 0
        || (ity.poison_resistance)(w, item) != 0
        || (ity.energy_resistance)(w, item) != 0
}

pub fn remove_item(w: &mut World, id: MobileId, item: ItemId) {
    let Some(m) = w.mobile_mut(id) else { return };
    let Some(pos) = m.items.iter().position(|&i| i == item) else {
        return;
    };
    m.items.remove(pos);
    let mty = m.ty;

    deps::item_send_remove_packet(w, item);

    let Some(ity) = w.item(item).map(|i| i.ty) else {
        return;
    };
    if !(ity.is_virtual_item)(w, item) {
        let gold = (ity.get_total)(w, item, TotalType::Gold);
        (mty.update_total)(w, id, item, TotalType::Gold, -gold);
        let items = (ity.get_total)(w, item, TotalType::Items);
        (mty.update_total)(w, id, item, TotalType::Items, -(items + 1));
        let weight = (ity.get_total)(w, item, TotalType::Weight) + deps::item_pile_weight(w, item);
        (mty.update_total)(w, id, item, TotalType::Weight, -weight);
    }

    if let Some(it) = w.item_mut(item) {
        it.parent = None;
    }

    (ity.on_removed)(w, item, EntityId::Mobile(id));
    (mty.on_item_removed)(w, id, item);

    if has_resistance(w, item) {
        (mty.update_resistances)(w, id);
    }
}

/// The first undeleted item on `layer`.
pub fn find_item_on_layer(w: &World, id: MobileId, layer: Layer) -> Option<ItemId> {
    let m = w.mobile(id)?;
    m.items.iter().copied().find(|&i| {
        w.item(i)
            .is_some_and(|it| !it.deleted() && it.layer == layer)
    })
}

/// The cached bank box, looked up again when it is gone or no longer ours.
pub fn find_bank_no_create(w: &mut World, id: MobileId) -> Option<ItemId> {
    let cached = w.mobile(id)?.bank_box;
    let valid = cached.is_some_and(|b| {
        w.item(b)
            .is_some_and(|it| !it.deleted() && it.parent == Some(EntityId::Mobile(id)))
    });
    if !valid {
        let found =
            find_item_on_layer(w, id, Layer::Bank).filter(|&i| deps::item_is_bank_box(w, i));
        w.mobile_mut(id)?.bank_box = found;
    }
    w.mobile(id)?.bank_box
}

/// `FindBankNoCreate()` followed by `box?.Opened == true` then `box.Close()`: closes an open bank box.
pub fn close_bank_box(w: &mut World, id: MobileId) {
    if let Some(b) = find_bank_no_create(w, id)
        && deps::bank_box_opened(w, b)
    {
        deps::bank_box_close(w, b);
    }
}

pub fn has_free_hand(w: &World, id: MobileId) -> bool {
    find_item_on_layer(w, id, Layer::TwoHanded).is_none()
}

pub fn drop_holding(w: &mut World, id: MobileId) {
    let Some(holding) = w.mobile(id).and_then(|m| m.holding) else {
        return;
    };
    let back = w.item(holding).is_some_and(|it| {
        !it.deleted() && it.held_by == Some(id) && it.map == Some(MapId::INTERNAL)
    });
    if back {
        add_to_backpack(w, id, holding);
    }
    deps::set_holding(w, id, None);
    deps::item_clear_bounce(w, holding);
}

pub fn place_in_backpack(w: &mut World, id: MobileId, item: ItemId) -> bool {
    if w.item(item).is_none_or(|i| i.deleted()) {
        return false;
    }
    match w.mobile(id).and_then(|m| m.backpack) {
        Some(pack) => deps::container_try_drop_item(w, pack, id, item, false),
        None => false,
    }
}

/// Into the pack, else on the ground where the mobile is (or logged out).
pub fn add_to_backpack(w: &mut World, id: MobileId, item: ItemId) -> bool {
    if w.item(item).is_none_or(|i| i.deleted()) {
        return false;
    }
    if !place_in_backpack(w, id, item) {
        let Some(m) = w.mobile(id) else { return false };
        let (mut loc, mut map): (Point3D, Option<MapId>) = (m.location, m.map);
        if (map.is_none() || map == Some(MapId::INTERNAL)) && m.logout_map.is_some() {
            loc = m.logout_location;
            map = m.logout_map;
        }
        deps::item_move_to_world(w, item, loc, map);
        return false;
    }
    true
}

/// `Map = Map.Internal`.
pub fn internalize(w: &mut World, id: MobileId) {
    deps::set_map(w, id, Some(MapId::INTERNAL));
}

/// A hair item becomes the inline hair id and hue.
pub fn convert_hair(w: &mut World, id: MobileId) {
    for (layer, facial) in [(Layer::Hair, false), (Layer::FacialHair, true)] {
        if let Some(hair) = find_item_on_layer(w, id, layer) {
            let (item_id, hue) = w.item(hair).map_or((0, 0), |h| (h.item_id, h.hue));
            if let Some(m) = w.mobile_mut(id) {
                if facial {
                    m.facial_hair_item_id = item_id;
                    m.facial_hair_hue = hue;
                } else {
                    m.hair_item_id = item_id;
                    m.hair_hue = hue;
                }
            }
            deps::item_delete(w, hair);
        }
    }
}

/// Splits `amount` off a stack. `old` keeps `amount`, the new item gets the rest.
/// `None` when the item's class has no parameterless constructor.
pub fn lift_item_dupe(w: &mut World, old: ItemId, amount: i32) -> Option<ItemId> {
    let ity = w.item(old)?.ty;
    let Some(construct) = ity.construct else {
        tracing::warn!(
            "[{} {}]: Item must have a zero parameter constructor to be separated from a stack.",
            old.0,
            ity.full_name
        );
        return None;
    };
    let item = construct(w);

    let (
        old_amount,
        visible,
        movable,
        loot_type,
        direction,
        hue,
        item_id,
        loc,
        layer,
        name,
        weight,
        pc,
        parent,
        map,
    ) = {
        let o = w.item_mut(old)?;
        let old_amount = o.amount;
        o.amount = amount;
        (
            old_amount,
            o.visible(),
            o.movable(),
            o.loot_type,
            o.direction,
            o.hue,
            o.item_id,
            o.location,
            o.layer,
            o.name.clone(),
            o.weight,
            o.player_constructed,
            o.parent,
            o.map,
        )
    };

    if let Some(n) = w.item_mut(item) {
        n.set_flag(crate::item::ImplFlag::VISIBLE, visible);
        n.set_flag(crate::item::ImplFlag::MOVABLE, movable);
        n.loot_type = loot_type;
        n.direction = direction;
        n.hue = hue;
        n.item_id = item_id;
        n.location = loc;
        n.layer = layer;
        n.name = name;
        n.weight = weight;
        n.player_constructed = pc;
        n.amount = old_amount - amount;
    }

    // A parented remainder gets its map from AddItem (parent first, then map), keeping the split off the decay
    // scheduler; a ground remainder is placed and enrolled here.
    if parent.is_none() {
        deps::item_set_map(w, item, map);
    }

    (ity.on_after_duped)(w, old, item);

    match parent {
        Some(EntityId::Mobile(pm)) => add_item(w, pm, item),
        Some(EntityId::Item(pi)) => {
            if let Some(pty) = w.item(pi).map(|p| p.ty) {
                (pty.add_item)(w, pi, item);
            }
        }
        None => {}
    }

    deps::item_delta(w, item, ItemDelta::UPDATE);
    Some(item)
}
