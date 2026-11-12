//! Default bodies of `Item`'s virtual members: the base behavior. A default that does nothing
//! here (an empty base method) is a bare `{}`.

use muo_io::{GenericReader, GenericWriter};
use muo_primitives::{Point2D, Point3D, TimeSpan, utility};

use crate::context_menu_entry::ContextMenuEntry;
use crate::ids::{EntityId, ItemId, MobileId};
use crate::item::item_type::ITEM_BASE;
use crate::item::{DeathMoveResult, LootType, TotalType};
use crate::layer::Layer;
use crate::lr_reason::LRReason;
use crate::map::MapId;
use crate::mobile::AccessLevel;
use crate::net_state::NetStateId;
use crate::property_list::PropertyList;
use crate::speech_event::SpeechEventArgs;
use crate::target::{TargetId, Targeted};
use crate::world::World;

/// `[Constructible]` `new Item()`.
pub fn construct(w: &mut World) -> ItemId {
    w.new_item(&ITEM_BASE)
}

/// `x.decays(..)`: the virtual call (an override runs; `decays_base` is what `base.decays(..)` reaches).
pub fn decays(w: &World, id: ItemId) -> bool {
    match w.item(id).map(|m| m.ty) {
        Some(ty) => (ty.decays)(w, id),
        None => decays_base(w, id),
    }
}

pub fn decays_base(w: &World, id: ItemId) -> bool {
    w.item(id)
        .is_some_and(|i| i.movable() && i.visible() && i.spawner.is_none())
}

/// `x.force_show_properties(..)`: the virtual call (an override runs; `force_show_properties_base` is what `base.force_show_properties(..)` reaches).
pub fn force_show_properties(w: &World, id: ItemId) -> bool {
    match w.item(id).map(|m| m.ty) {
        Some(ty) => (ty.force_show_properties)(w, id),
        None => force_show_properties_base(w, id),
    }
}

pub fn force_show_properties_base(_w: &World, _id: ItemId) -> bool {
    false
}

/// `x.handles_on_movement(..)`: the virtual call (an override runs; `handles_on_movement_base` is what `base.handles_on_movement(..)` reaches).
pub fn handles_on_movement(w: &World, id: ItemId) -> bool {
    match w.item(id).map(|m| m.ty) {
        Some(ty) => (ty.handles_on_movement)(w, id),
        None => handles_on_movement_base(w, id),
    }
}

pub fn handles_on_movement_base(_w: &World, _id: ItemId) -> bool {
    false
}

/// `x.is_virtual_item(..)`: the virtual call (an override runs; `is_virtual_item_base` is what `base.is_virtual_item(..)` reaches).
pub fn is_virtual_item(w: &World, id: ItemId) -> bool {
    match w.item(id).map(|m| m.ty) {
        Some(ty) => (ty.is_virtual_item)(w, id),
        None => is_virtual_item_base(w, id),
    }
}

pub fn is_virtual_item_base(_w: &World, _id: ItemId) -> bool {
    false
}

/// `x.can_see_staff_only(..)`: the virtual call (an override runs; `can_see_staff_only_base` is what `base.can_see_staff_only(..)` reaches).
pub fn can_see_staff_only(w: &World, id: ItemId, a2: MobileId) -> bool {
    match w.item(id).map(|m| m.ty) {
        Some(ty) => (ty.can_see_staff_only)(w, id, a2),
        None => can_see_staff_only_base(w, id, a2),
    }
}

pub fn can_see_staff_only_base(w: &World, _id: ItemId, from: MobileId) -> bool {
    w.mobile(from)
        .is_some_and(|m| m.access_level > AccessLevel::Counselor)
}

/// `x.label_number(..)`: the virtual call (an override runs; `label_number_base` is what `base.label_number(..)` reaches).
pub fn label_number(w: &World, id: ItemId) -> i32 {
    match w.item(id).map(|m| m.ty) {
        Some(ty) => (ty.label_number)(w, id),
        None => label_number_base(w, id),
    }
}

pub fn label_number_base(w: &World, id: ItemId) -> i32 {
    w.item(id).map_or(0, |i| {
        if i.item_id < 0x4000 {
            1_020_000 + i.item_id
        } else {
            1_078_872 + i.item_id
        }
    })
}

/// `x.hue(..)`: the virtual call (an override runs; `hue_base` is what `base.hue(..)` reaches).
pub fn hue(w: &World, id: ItemId) -> i32 {
    match w.item(id).map(|m| m.ty) {
        Some(ty) => (ty.hue)(w, id),
        None => hue_base(w, id),
    }
}

pub fn hue_base(w: &World, id: ItemId) -> i32 {
    w.item(id).map_or(0, |i| i.hue)
}

/// `x.nontransferable(..)`: the virtual call (an override runs; `nontransferable_base` is what `base.nontransferable(..)` reaches).
pub fn nontransferable(w: &World, id: ItemId) -> bool {
    match w.item(id).map(|m| m.ty) {
        Some(ty) => (ty.nontransferable)(w, id),
        None => nontransferable_base(w, id),
    }
}

pub fn nontransferable_base(w: &World, id: ItemId) -> bool {
    w.item(id).is_some_and(|i| i.quest_item())
}

pub fn layer(w: &World, id: ItemId) -> Layer {
    w.item(id).map_or(Layer::Invalid, |i| i.layer)
}

/// `x.physical_resistance(..)`: the virtual call (an override runs; `physical_resistance_base` is what `base.physical_resistance(..)` reaches).
pub fn physical_resistance(w: &World, id: ItemId) -> i32 {
    match w.item(id).map(|m| m.ty) {
        Some(ty) => (ty.physical_resistance)(w, id),
        None => physical_resistance_base(w, id),
    }
}

pub fn physical_resistance_base(_w: &World, _id: ItemId) -> i32 {
    0
}

/// `x.fire_resistance(..)`: the virtual call (an override runs; `fire_resistance_base` is what `base.fire_resistance(..)` reaches).
pub fn fire_resistance(w: &World, id: ItemId) -> i32 {
    match w.item(id).map(|m| m.ty) {
        Some(ty) => (ty.fire_resistance)(w, id),
        None => fire_resistance_base(w, id),
    }
}

pub fn fire_resistance_base(_w: &World, _id: ItemId) -> i32 {
    0
}

/// `x.cold_resistance(..)`: the virtual call (an override runs; `cold_resistance_base` is what `base.cold_resistance(..)` reaches).
pub fn cold_resistance(w: &World, id: ItemId) -> i32 {
    match w.item(id).map(|m| m.ty) {
        Some(ty) => (ty.cold_resistance)(w, id),
        None => cold_resistance_base(w, id),
    }
}

pub fn cold_resistance_base(_w: &World, _id: ItemId) -> i32 {
    0
}

/// `x.poison_resistance(..)`: the virtual call (an override runs; `poison_resistance_base` is what `base.poison_resistance(..)` reaches).
pub fn poison_resistance(w: &World, id: ItemId) -> i32 {
    match w.item(id).map(|m| m.ty) {
        Some(ty) => (ty.poison_resistance)(w, id),
        None => poison_resistance_base(w, id),
    }
}

pub fn poison_resistance_base(_w: &World, _id: ItemId) -> i32 {
    0
}

/// `x.energy_resistance(..)`: the virtual call (an override runs; `energy_resistance_base` is what `base.energy_resistance(..)` reaches).
pub fn energy_resistance(w: &World, id: ItemId) -> i32 {
    match w.item(id).map(|m| m.ty) {
        Some(ty) => (ty.energy_resistance)(w, id),
        None => energy_resistance_base(w, id),
    }
}

pub fn energy_resistance_base(_w: &World, _id: ItemId) -> i32 {
    0
}

pub fn item_id(w: &World, id: ItemId) -> i32 {
    w.item(id).map_or(0, |i| i.item_id)
}

/// `x.default_name(..)`: the virtual call (an override runs; `default_name_base` is what `base.default_name(..)` reaches).
pub fn default_name(w: &World, id: ItemId) -> Option<String> {
    match w.item(id).map(|m| m.ty) {
        Some(ty) => (ty.default_name)(w, id),
        None => default_name_base(w, id),
    }
}

pub fn default_name_base(_w: &World, _id: ItemId) -> Option<String> {
    None
}

/// `x.handles_on_speech(..)`: the virtual call (an override runs; `handles_on_speech_base` is what `base.handles_on_speech(..)` reaches).
pub fn handles_on_speech(w: &World, id: ItemId) -> bool {
    match w.item(id).map(|m| m.ty) {
        Some(ty) => (ty.handles_on_speech)(w, id),
        None => handles_on_speech_base(w, id),
    }
}

pub fn handles_on_speech_base(_w: &World, _id: ItemId) -> bool {
    false
}

/// `x.blocks_fit(..)`: the virtual call (an override runs; `blocks_fit_base` is what `base.blocks_fit(..)` reaches).
pub fn blocks_fit(w: &World, id: ItemId) -> bool {
    match w.item(id).map(|m| m.ty) {
        Some(ty) => (ty.blocks_fit)(w, id),
        None => blocks_fit_base(w, id),
    }
}

pub fn blocks_fit_base(_w: &World, _id: ItemId) -> bool {
    false
}

/// `x.can_target(..)`: the virtual call (an override runs; `can_target_base` is what `base.can_target(..)` reaches).
pub fn can_target(w: &World, id: ItemId) -> bool {
    match w.item(id).map(|m| m.ty) {
        Some(ty) => (ty.can_target)(w, id),
        None => can_target_base(w, id),
    }
}

pub fn can_target_base(_w: &World, _id: ItemId) -> bool {
    true
}

/// `x.display_loot_type(..)`: the virtual call (an override runs; `display_loot_type_base` is what `base.display_loot_type(..)` reaches).
pub fn display_loot_type(w: &World, id: ItemId) -> bool {
    match w.item(id).map(|m| m.ty) {
        Some(ty) => (ty.display_loot_type)(w, id),
        None => display_loot_type_base(w, id),
    }
}

pub fn display_loot_type_base(_w: &World, _id: ItemId) -> bool {
    true
}

pub fn hued_item_id(w: &World, id: ItemId) -> i32 {
    w.item(id).map_or(0, |i| i.item_id)
}

/// `x.get_properties(..)`: the virtual call (an override runs; `get_properties_base` is what `base.get_properties(..)` reaches).
pub fn get_properties(w: &World, id: ItemId, a2: &mut PropertyList) {
    match w.item(id).map(|m| m.ty) {
        Some(ty) => (ty.get_properties)(w, id, a2),
        None => get_properties_base(w, id, a2),
    }
}

pub fn get_properties_base(w: &World, id: ItemId, list: &mut PropertyList) {
    if let Some(ty) = w.item(id).map(|i| i.ty) {
        (ty.add_name_properties)(w, id, list);
        (ty.append_child_name_properties)(w, id, list);
    }
}

pub fn on_before_spawn(_w: &mut World, _id: ItemId, _location: Point3D, _map: MapId) {}

pub fn on_after_spawn(_w: &mut World, _id: ItemId) {}

pub fn location(w: &World, id: ItemId) -> Point3D {
    w.item(id).map_or(Point3D::default(), |i| i.location)
}

/// `x.in_range_point2d(..)`: the virtual call (an override runs; `in_range_point2d_base` is what `base.in_range_point2d(..)` reaches).
pub fn in_range_point2d(w: &World, id: ItemId, a2: Point2D, a3: i32) -> bool {
    match w.item(id).map(|m| m.ty) {
        Some(ty) => (ty.in_range_point2d)(w, id, a2, a3),
        None => in_range_point2d_base(w, id, a2, a3),
    }
}

pub fn in_range_point2d_base(w: &World, id: ItemId, p: Point2D, range: i32) -> bool {
    w.item(id)
        .is_some_and(|i| utility::in_range(p.x, p.y, i.location.x, i.location.y, range))
}

/// `x.in_range_point3d(..)`: the virtual call (an override runs; `in_range_point3d_base` is what `base.in_range_point3d(..)` reaches).
pub fn in_range_point3d(w: &World, id: ItemId, a2: Point3D, a3: i32) -> bool {
    match w.item(id).map(|m| m.ty) {
        Some(ty) => (ty.in_range_point3d)(w, id, a2, a3),
        None => in_range_point3d_base(w, id, a2, a3),
    }
}

pub fn in_range_point3d_base(w: &World, id: ItemId, p: Point3D, range: i32) -> bool {
    w.item(id)
        .is_some_and(|i| utility::in_range(p.x, p.y, i.location.x, i.location.y, range))
}

pub fn on_help_request(_w: &mut World, _id: ItemId, _from: MobileId) {}

/// `x.allow_secure_trade(..)`: the virtual call (an override runs; `allow_secure_trade_base` is what `base.allow_secure_trade(..)` reaches).
pub fn allow_secure_trade(
    w: &mut World,
    id: ItemId,
    a2: MobileId,
    a3: MobileId,
    a4: MobileId,
    a5: bool,
) -> bool {
    match w.item(id).map(|m| m.ty) {
        Some(ty) => (ty.allow_secure_trade)(w, id, a2, a3, a4, a5),
        None => allow_secure_trade_base(w, id, a2, a3, a4, a5),
    }
}

pub fn allow_secure_trade_base(
    _w: &mut World,
    _id: ItemId,
    _from: MobileId,
    _to: MobileId,
    _new_owner: MobileId,
    _accepted: bool,
) -> bool {
    true
}

/// `x.on_secure_trade(..)`: the virtual call (an override runs; `on_secure_trade_base` is what `base.on_secure_trade(..)` reaches).
pub fn on_secure_trade(
    w: &mut World,
    id: ItemId,
    a2: MobileId,
    a3: MobileId,
    a4: MobileId,
    a5: bool,
) {
    match w.item(id).map(|m| m.ty) {
        Some(ty) => (ty.on_secure_trade)(w, id, a2, a3, a4, a5),
        None => on_secure_trade_base(w, id, a2, a3, a4, a5),
    }
}

pub fn on_secure_trade_base(
    _w: &mut World,
    _id: ItemId,
    _from: MobileId,
    _to: MobileId,
    _new_owner: MobileId,
    _accepted: bool,
) {
}

/// `x.check_property_conflict(..)`: the virtual call (an override runs; `check_property_conflict_base` is what `base.check_property_conflict(..)` reaches).
pub fn check_property_conflict(w: &World, id: ItemId, a2: MobileId) -> bool {
    match w.item(id).map(|m| m.ty) {
        Some(ty) => (ty.check_property_conflict)(w, id, a2),
        None => check_property_conflict_base(w, id, a2),
    }
}

pub fn check_property_conflict_base(_w: &World, _id: ItemId, _m: MobileId) -> bool {
    false
}

/// `x.is_child_visible_to(..)`: the virtual call (an override runs; `is_child_visible_to_base` is what `base.is_child_visible_to(..)` reaches).
pub fn is_child_visible_to(w: &World, id: ItemId, a2: MobileId, a3: ItemId) -> bool {
    match w.item(id).map(|m| m.ty) {
        Some(ty) => (ty.is_child_visible_to)(w, id, a2, a3),
        None => is_child_visible_to_base(w, id, a2, a3),
    }
}

pub fn is_child_visible_to_base(_w: &World, _id: ItemId, _m: MobileId, _child: ItemId) -> bool {
    true
}

/// `x.allow_equipped_cast(..)`: the virtual call (an override runs; `allow_equipped_cast_base` is what `base.allow_equipped_cast(..)` reaches).
pub fn allow_equipped_cast(w: &World, id: ItemId, a2: MobileId) -> bool {
    match w.item(id).map(|m| m.ty) {
        Some(ty) => (ty.allow_equipped_cast)(w, id, a2),
        None => allow_equipped_cast_base(w, id, a2),
    }
}

pub fn allow_equipped_cast_base(_w: &World, _id: ItemId, _from: MobileId) -> bool {
    false
}

/// `x.check_conflicting_layer(..)`: the virtual call (an override runs; `check_conflicting_layer_base` is what `base.check_conflicting_layer(..)` reaches).
pub fn check_conflicting_layer(
    w: &mut World,
    id: ItemId,
    a2: MobileId,
    a3: ItemId,
    a4: Layer,
) -> bool {
    match w.item(id).map(|m| m.ty) {
        Some(ty) => (ty.check_conflicting_layer)(w, id, a2, a3, a4),
        None => check_conflicting_layer_base(w, id, a2, a3, a4),
    }
}

pub fn check_conflicting_layer_base(
    w: &mut World,
    id: ItemId,
    _m: MobileId,
    _item: ItemId,
    layer: Layer,
) -> bool {
    w.item(id).is_some_and(|i| i.layer == layer)
}

/// `x.required_races(..)`: the virtual call (an override runs; `required_races_base` is what `base.required_races(..)` reaches).
pub fn required_races(w: &World, id: ItemId) -> i32 {
    match w.item(id).map(|m| m.ty) {
        Some(ty) => (ty.required_races)(w, id),
        None => required_races_base(w, id),
    }
}

pub fn required_races_base(_w: &World, _id: ItemId) -> i32 {
    0x7
}

/// `x.verify_move(..)`: the virtual call (an override runs; `verify_move_base` is what `base.verify_move(..)` reaches).
pub fn verify_move(w: &mut World, id: ItemId, a2: MobileId) -> bool {
    match w.item(id).map(|m| m.ty) {
        Some(ty) => (ty.verify_move)(w, id, a2),
        None => verify_move_base(w, id, a2),
    }
}

pub fn verify_move_base(w: &mut World, id: ItemId, _from: MobileId) -> bool {
    w.item(id).is_some_and(|i| i.movable())
}

/// `x.on_move_off(..)`: the virtual call (an override runs; `on_move_off_base` is what `base.on_move_off(..)` reaches).
pub fn on_move_off(w: &mut World, id: ItemId, a2: MobileId) -> bool {
    match w.item(id).map(|m| m.ty) {
        Some(ty) => (ty.on_move_off)(w, id, a2),
        None => on_move_off_base(w, id, a2),
    }
}

pub fn on_move_off_base(_w: &mut World, _id: ItemId, _m: MobileId) -> bool {
    true
}

/// `x.on_move_over(..)`: the virtual call (an override runs; `on_move_over_base` is what `base.on_move_over(..)` reaches).
pub fn on_move_over(w: &mut World, id: ItemId, a2: MobileId) -> bool {
    match w.item(id).map(|m| m.ty) {
        Some(ty) => (ty.on_move_over)(w, id, a2),
        None => on_move_over_base(w, id, a2),
    }
}

pub fn on_move_over_base(_w: &mut World, _id: ItemId, _m: MobileId) -> bool {
    true
}

/// `x.on_movement(..)`: the virtual call (an override runs; `on_movement_base` is what `base.on_movement(..)` reaches).
pub fn on_movement(w: &mut World, id: ItemId, a2: MobileId, a3: Point3D) {
    match w.item(id).map(|m| m.ty) {
        Some(ty) => (ty.on_movement)(w, id, a2, a3),
        None => on_movement_base(w, id, a2, a3),
    }
}

pub fn on_movement_base(_w: &mut World, _id: ItemId, _m: MobileId, _old_location: Point3D) {}

/// `x.on_map_change(..)`: the virtual call (an override runs; `on_map_change_base` is what `base.on_map_change(..)` reaches).
pub fn on_map_change(w: &mut World, id: ItemId) {
    match w.item(id).map(|m| m.ty) {
        Some(ty) => (ty.on_map_change)(w, id),
        None => on_map_change_base(w, id),
    }
}

pub fn on_map_change_base(_w: &mut World, _id: ItemId) {}

/// `x.on_removed(..)`: the virtual call (an override runs; `on_removed_base` is what `base.on_removed(..)` reaches).
pub fn on_removed(w: &mut World, id: ItemId, a2: EntityId) {
    match w.item(id).map(|m| m.ty) {
        Some(ty) => (ty.on_removed)(w, id, a2),
        None => on_removed_base(w, id, a2),
    }
}

pub fn on_removed_base(_w: &mut World, _id: ItemId, _parent: EntityId) {}

/// `x.on_added(..)`: the virtual call (an override runs; `on_added_base` is what `base.on_added(..)` reaches).
pub fn on_added(w: &mut World, id: ItemId, a2: EntityId) {
    match w.item(id).map(|m| m.ty) {
        Some(ty) => (ty.on_added)(w, id, a2),
        None => on_added_base(w, id, a2),
    }
}

pub fn on_added_base(_w: &mut World, _id: ItemId, _parent: EntityId) {}

/// `x.get_max_update_range(..)`: the virtual call (an override runs; `get_max_update_range_base` is what `base.get_max_update_range(..)` reaches).
pub fn get_max_update_range(w: &World, id: ItemId) -> i32 {
    match w.item(id).map(|m| m.ty) {
        Some(ty) => (ty.get_max_update_range)(w, id),
        None => get_max_update_range_base(w, id),
    }
}

pub fn get_max_update_range_base(_w: &World, _id: ItemId) -> i32 {
    18
}

/// `x.get_update_range(..)`: the virtual call (an override runs; `get_update_range_base` is what `base.get_update_range(..)` reaches).
pub fn get_update_range(w: &World, id: ItemId, a2: MobileId) -> i32 {
    match w.item(id).map(|m| m.ty) {
        Some(ty) => (ty.get_update_range)(w, id, a2),
        None => get_update_range_base(w, id, a2),
    }
}

pub fn get_update_range_base(_w: &World, _id: ItemId, _m: MobileId) -> i32 {
    18
}

/// `x.get_total(..)`: the virtual call (an override runs; `get_total_base` is what `base.get_total(..)` reaches).
pub fn get_total(w: &World, id: ItemId, a2: TotalType) -> i32 {
    match w.item(id).map(|m| m.ty) {
        Some(ty) => (ty.get_total)(w, id, a2),
        None => get_total_base(w, id, a2),
    }
}

pub fn get_total_base(_w: &World, _id: ItemId, _ty: TotalType) -> i32 {
    0
}

/// `x.update_totals(..)`: the virtual call (an override runs; `update_totals_base` is what `base.update_totals(..)` reaches).
pub fn update_totals(w: &mut World, id: ItemId) {
    match w.item(id).map(|m| m.ty) {
        Some(ty) => (ty.update_totals)(w, id),
        None => update_totals_base(w, id),
    }
}

pub fn update_totals_base(_w: &mut World, _id: ItemId) {}

/// `x.on_parent_deleted(..)`: the virtual call (an override runs; `on_parent_deleted_base` is what `base.on_parent_deleted(..)` reaches).
pub fn on_parent_deleted(w: &mut World, id: ItemId, a2: EntityId) {
    match w.item(id).map(|m| m.ty) {
        Some(ty) => (ty.on_parent_deleted)(w, id, a2),
        None => on_parent_deleted_base(w, id, a2),
    }
}

pub fn on_parent_deleted_base(w: &mut World, id: ItemId, _parent: EntityId) {
    if let Some(ty) = w.item(id).map(|i| i.ty) {
        (ty.delete)(w, id);
    }
}

/// `x.on_after_delete(..)`: the virtual call (an override runs; `on_after_delete_base` is what `base.on_after_delete(..)` reaches).
pub fn on_after_delete(w: &mut World, id: ItemId) {
    match w.item(id).map(|m| m.ty) {
        Some(ty) => (ty.on_after_delete)(w, id),
        None => on_after_delete_base(w, id),
    }
}

pub fn on_after_delete_base(_w: &mut World, _id: ItemId) {}

/// `x.on_after_duped(..)`: the virtual call (an override runs; `on_after_duped_base` is what `base.on_after_duped(..)` reaches).
pub fn on_after_duped(w: &mut World, id: ItemId, a2: ItemId) {
    match w.item(id).map(|m| m.ty) {
        Some(ty) => (ty.on_after_duped)(w, id, a2),
        None => on_after_duped_base(w, id, a2),
    }
}

pub fn on_after_duped_base(_w: &mut World, _id: ItemId, _new_item: ItemId) {}

/// `x.on_drag_lift(..)`: the virtual call (an override runs; `on_drag_lift_base` is what `base.on_drag_lift(..)` reaches).
pub fn on_drag_lift(w: &mut World, id: ItemId, a2: MobileId) -> bool {
    match w.item(id).map(|m| m.ty) {
        Some(ty) => (ty.on_drag_lift)(w, id, a2),
        None => on_drag_lift_base(w, id, a2),
    }
}

pub fn on_drag_lift_base(_w: &mut World, _id: ItemId, _from: MobileId) -> bool {
    true
}

/// `x.on_equip(..)`: the virtual call (an override runs; `on_equip_base` is what `base.on_equip(..)` reaches).
pub fn on_equip(w: &mut World, id: ItemId, a2: MobileId) -> bool {
    match w.item(id).map(|m| m.ty) {
        Some(ty) => (ty.on_equip)(w, id, a2),
        None => on_equip_base(w, id, a2),
    }
}

pub fn on_equip_base(_w: &mut World, _id: ItemId, _from: MobileId) -> bool {
    true
}

/// `x.on_amount_change(..)`: the virtual call (an override runs; `on_amount_change_base` is what `base.on_amount_change(..)` reaches).
pub fn on_amount_change(w: &mut World, id: ItemId, a2: i32) {
    match w.item(id).map(|m| m.ty) {
        Some(ty) => (ty.on_amount_change)(w, id, a2),
        None => on_amount_change_base(w, id, a2),
    }
}

pub fn on_amount_change_base(_w: &mut World, _id: ItemId, _old_value: i32) {}

/// `x.on_speech(..)`: the virtual call (an override runs; `on_speech_base` is what `base.on_speech(..)` reaches).
pub fn on_speech(w: &mut World, id: ItemId, a2: &mut SpeechEventArgs) {
    match w.item(id).map(|m| m.ty) {
        Some(ty) => (ty.on_speech)(w, id, a2),
        None => on_speech_base(w, id, a2),
    }
}

pub fn on_speech_base(_w: &mut World, _id: ItemId, _e: &mut SpeechEventArgs) {}

/// `x.get_lift_sound(..)`: the virtual call (an override runs; `get_lift_sound_base` is what `base.get_lift_sound(..)` reaches).
pub fn get_lift_sound(w: &World, id: ItemId, a2: MobileId) -> i32 {
    match w.item(id).map(|m| m.ty) {
        Some(ty) => (ty.get_lift_sound)(w, id, a2),
        None => get_lift_sound_base(w, id, a2),
    }
}

pub fn get_lift_sound_base(_w: &World, _id: ItemId, _from: MobileId) -> i32 {
    0x57
}

/// `x.get_drop_sound(..)`: the virtual call (an override runs; `get_drop_sound_base` is what `base.get_drop_sound(..)` reaches).
pub fn get_drop_sound(w: &World, id: ItemId) -> i32 {
    match w.item(id).map(|m| m.ty) {
        Some(ty) => (ty.get_drop_sound)(w, id),
        None => get_drop_sound_base(w, id),
    }
}

pub fn get_drop_sound_base(_w: &World, _id: ItemId) -> i32 {
    -1
}

/// `x.on_snoop(..)`: the virtual call (an override runs; `on_snoop_base` is what `base.on_snoop(..)` reaches).
pub fn on_snoop(w: &mut World, id: ItemId, a2: MobileId) {
    match w.item(id).map(|m| m.ty) {
        Some(ty) => (ty.on_snoop)(w, id, a2),
        None => on_snoop_base(w, id, a2),
    }
}

pub fn on_snoop_base(_w: &mut World, _id: ItemId, _from: MobileId) {}

pub fn check_newbied(w: &World, id: ItemId) -> bool {
    w.item(id).is_some_and(|i| i.loot_type == LootType::Newbied)
}

/// `x.on_sector_activate(..)`: the virtual call (an override runs; `on_sector_activate_base` is what `base.on_sector_activate(..)` reaches).
pub fn on_sector_activate(w: &mut World, id: ItemId) {
    match w.item(id).map(|m| m.ty) {
        Some(ty) => (ty.on_sector_activate)(w, id),
        None => on_sector_activate_base(w, id),
    }
}

pub fn on_sector_activate_base(_w: &mut World, _id: ItemId) {}

/// `x.on_sector_deactivate(..)`: the virtual call (an override runs; `on_sector_deactivate_base` is what `base.on_sector_deactivate(..)` reaches).
pub fn on_sector_deactivate(w: &mut World, id: ItemId) {
    match w.item(id).map(|m| m.ty) {
        Some(ty) => (ty.on_sector_deactivate)(w, id),
        None => on_sector_deactivate_base(w, id),
    }
}

pub fn on_sector_deactivate_base(_w: &mut World, _id: ItemId) {}

/// `x.on_location_change(..)`: the virtual call (an override runs; `on_location_change_base` is what `base.on_location_change(..)` reaches).
pub fn on_location_change(w: &mut World, id: ItemId, a2: Point3D) {
    match w.item(id).map(|m| m.ty) {
        Some(ty) => (ty.on_location_change)(w, id, a2),
        None => on_location_change_base(w, id, a2),
    }
}

pub fn on_location_change_base(_w: &mut World, _id: ItemId, _old_location: Point3D) {}

/// `x.on_double_click(..)`: the virtual call (an override runs; `on_double_click_base` is what `base.on_double_click(..)` reaches).
pub fn on_double_click(w: &mut World, id: ItemId, a2: MobileId) {
    match w.item(id).map(|m| m.ty) {
        Some(ty) => (ty.on_double_click)(w, id, a2),
        None => on_double_click_base(w, id, a2),
    }
}

pub fn on_double_click_base(_w: &mut World, _id: ItemId, _from: MobileId) {}

pub fn on_double_click_out_of_range(_w: &mut World, _id: ItemId, _from: MobileId) {}

pub fn on_double_click_cant_see(_w: &mut World, _id: ItemId, _from: MobileId) {}

// ===== bodies =====

use crate::item::container::CONTAINER;
use crate::item::deps;
use crate::item::multi::BASE_MULTI;
use crate::item::ops;
use crate::item::{ImplFlag, ItemDelta};
use crate::packets::message::MessageType;

fn ty_of(w: &World, id: ItemId) -> Option<&'static crate::item::ItemType> {
    w.item(id).map(|i| i.ty)
}

fn parent_of(w: &World, id: ItemId) -> Option<EntityId> {
    w.item(id).and_then(|i| i.parent)
}

/// `x.display_weight(..)`: the virtual call (an override runs; `display_weight_base` is what `base.display_weight(..)` reaches).
pub fn display_weight(w: &World, id: ItemId) -> bool {
    match w.item(id).map(|m| m.ty) {
        Some(ty) => (ty.display_weight)(w, id),
        None => display_weight_base(w, id),
    }
}

pub fn display_weight_base(w: &World, id: ItemId) -> bool {
    let Some(i) = w.item(id) else { return false };
    deps::core_ml(w)
        && (i.movable()
            || ops::is_locked_down(w, id)
            || ops::is_secure(w, id)
            || ops::item_data(w, id).weight != 255)
}

/// `x.decay_time(..)`: the virtual call (an override runs; `decay_time_base` is what `base.decay_time(..)` reaches).
pub fn decay_time(w: &World, id: ItemId) -> TimeSpan {
    match w.item(id).map(|m| m.ty) {
        Some(ty) => (ty.decay_time)(w, id),
        None => decay_time_base(w, id),
    }
}

pub fn decay_time_base(w: &World, _id: ItemId) -> TimeSpan {
    ops::default_decay_time(w)
}

/// `x.default_weight(..)`: the virtual call (an override runs; `default_weight_base` is what `base.default_weight(..)` reaches).
pub fn default_weight(w: &World, id: ItemId) -> f64 {
    match w.item(id).map(|m| m.ty) {
        Some(ty) => (ty.default_weight)(w, id),
        None => default_weight_base(w, id),
    }
}

pub fn default_weight_base(w: &World, id: ItemId) -> f64 {
    let Some(i) = w.item(id) else { return 0.0 };
    if i.item_id < 0
        || i.item_id > muo_data::TileData::MAX_ITEM_VALUE as i32
        || i.ty.is(&BASE_MULTI)
    {
        return 0.0;
    }
    let weight = ops::item_data(w, id).weight;
    if weight == 255 || weight == 0 {
        1.0
    } else {
        f64::from(weight)
    }
}

/// `x.set_hue(..)`: the virtual call (an override runs; `set_hue_base` is what `base.set_hue(..)` reaches).
pub fn set_hue(w: &mut World, id: ItemId, a2: i32) {
    match w.item(id).map(|m| m.ty) {
        Some(ty) => (ty.set_hue)(w, id, a2),
        None => set_hue_base(w, id, a2),
    }
}

pub fn set_hue_base(w: &mut World, id: ItemId, value: i32) {
    if let Some(i) = w.item_mut(id)
        && i.hue != value
    {
        i.hue = value;
        ops::delta(w, id, ItemDelta::UPDATE);
    }
}

pub fn set_layer(w: &mut World, id: ItemId, value: Layer) {
    if let Some(i) = w.item_mut(id)
        && i.layer != value
    {
        i.layer = value;
        ops::delta(w, id, ItemDelta::EQUIP_ONLY);
    }
}

/// `x.set_item_id(..)`: the virtual call (an override runs; `set_item_id_base` is what `base.set_item_id(..)` reaches).
pub fn set_item_id(w: &mut World, id: ItemId, a2: i32) {
    match w.item(id).map(|m| m.ty) {
        Some(ty) => (ty.set_item_id)(w, id, a2),
        None => set_item_id_base(w, id, a2),
    }
}

pub fn set_item_id_base(w: &mut World, id: ItemId, value: i32) {
    let Some(i) = w.item(id) else { return };
    if i.item_id != value {
        let old_pile_weight = ops::pile_weight(w, id);
        if let Some(i) = w.item_mut(id) {
            i.item_id = value;
        }
        let new_pile_weight = ops::pile_weight(w, id);
        ops::update_total(
            w,
            id,
            id,
            TotalType::Weight,
            new_pile_weight - old_pile_weight,
        );
        ops::invalidate_properties(w, id);
        ops::delta(w, id, ItemDelta::UPDATE);
    }
}

/// `x.serialize(..)`: the virtual call (an override runs; `serialize_base` is what `base.serialize(..)` reaches).
pub fn serialize(w: &World, id: ItemId, a2: &mut dyn GenericWriter) {
    match w.item(id).map(|m| m.ty) {
        Some(ty) => (ty.serialize)(w, id, a2),
        None => serialize_base(w, id, a2),
    }
}

pub fn serialize_base(w: &World, id: ItemId, writer: &mut dyn GenericWriter) {
    crate::item::serialize::serialize(w, id, writer);
}

/// `x.process_delta(..)`: the virtual call (an override runs; `process_delta_base` is what `base.process_delta(..)` reaches).
pub fn process_delta(w: &mut World, id: ItemId) {
    match w.item(id).map(|m| m.ty) {
        Some(ty) => (ty.process_delta)(w, id),
        None => process_delta_base(w, id),
    }
}

pub fn process_delta_base(w: &mut World, id: ItemId) {
    let Some(i) = w.item(id) else { return };
    let flags = i.delta_flags;
    let (map, ty) = (i.map, i.ty);

    ops::set_flag(w, id, ImplFlag::IN_QUEUE, false);
    if let Some(i) = w.item_mut(id) {
        i.delta_flags = ItemDelta::NONE;
    }

    let Some(map) = map else { return };
    if ops::is_deleted(w, id) {
        return;
    }

    let world_loc = ops::get_world_location(w, id);
    let update = flags.contains(ItemDelta::UPDATE);
    let parent = parent_of(w, id);

    if update
        && let Some(EntityId::Item(cont_parent)) = parent
        && let Some(ct) = ty_of(w, cont_parent)
        && ct.is(&CONTAINER)
        && !(ct.is_public_container)(w, cont_parent)
    {
        let root_parent = match ops::root_parent(w, cont_parent) {
            Some(EntityId::Mobile(m)) => Some(m),
            _ => None,
        };
        let mut trade_recip: Option<MobileId> = None;

        if let Some(rp) = root_parent
            && let Some(ns) = deps::net_state_of(w, rp)
            && deps::can_see(w, rp, id)
            && deps::mobile_in_range(w, rp, world_loc, (ty.get_update_range)(w, id, rp))
        {
            crate::packets::container::send_container_content_update(w, ns, id);
            (ty.send_opl_packet_to)(w, id, ns);
        }

        if let Some(trade) = crate::item::secure_trade_container::trade_of(w, id) {
            let (from, to) = deps::trade_parties(w, trade);
            if let Some(t) = from
                && Some(t) != root_parent
            {
                trade_recip = Some(t);
            }
            if let Some(t) = to
                && Some(t) != root_parent
            {
                trade_recip = Some(t);
            }

            if let Some(tr) = trade_recip
                && let Some(ns) = deps::net_state_of(w, tr)
                && deps::can_see(w, tr, id)
                && deps::mobile_in_range(w, tr, world_loc, (ty.get_update_range)(w, id, tr))
            {
                crate::packets::container::send_container_content_update(w, ns, id);
                (ty.send_opl_packet_to)(w, id, ns);
            }
        }

        let openers = crate::item::container::openers(w, cont_parent).cloned();
        if let Some(mut list) = openers {
            let mut i = 0;
            while i < list.len() {
                let mob = list[i];
                let range = (ty.get_update_range)(w, id, mob);
                let same_map = w.mobile(mob).is_some_and(|m| m.map == Some(map));
                if !same_map || !deps::mobile_in_range(w, mob, world_loc, range) {
                    list.remove(i);
                } else {
                    if Some(mob) == root_parent || Some(mob) == trade_recip {
                        i += 1;
                        continue;
                    }
                    if let Some(ns) = deps::net_state_of(w, mob)
                        && deps::can_see(w, mob, id)
                    {
                        crate::packets::container::send_container_content_update(w, ns, id);
                        (ty.send_opl_packet_to)(w, id, ns);
                    }
                    i += 1;
                }
            }
            if let Some(c) = w.item_mut(cont_parent).and_then(|c| c.cont.as_deref_mut()) {
                c.openers = if list.is_empty() { None } else { Some(list) };
            }
        }
        return;
    }

    // A properties-only invalidation (no Update) on a private child must stay as scoped as every other
    // private-child send, otherwise a bystander who was never sent the item learns its OPL hash.
    if !update
        && deps::opl_enabled(w)
        && flags.contains(ItemDelta::PROPERTIES)
        && let Some(cont) = ops::try_get_private_parent(w, id)
    {
        let (root, trade_from, trade_to) = ops::get_private_child_recipients(w, id, cont);

        ops::send_private_opl_to(w, id, root, world_loc);
        if trade_from != root {
            ops::send_private_opl_to(w, id, trade_from, world_loc);
        }
        if trade_to != root && trade_to != trade_from {
            ops::send_private_opl_to(w, id, trade_to, world_loc);
        }

        let openers = crate::item::container::openers(w, cont)
            .cloned()
            .unwrap_or_default();
        for mob in openers {
            if Some(mob) != root && Some(mob) != trade_from && Some(mob) != trade_to {
                ops::send_private_opl_to(w, id, Some(mob), world_loc);
            }
        }
        return;
    }

    // A second item sharing an equipment layer is omitted by SendMobileIncoming (0x78); sending it on its own
    // via EquipUpdate/OPL would leave the client with two items on one slot.
    if ops::is_duped_equip_layer(w, id) {
        return;
    }

    for (ns, m) in deps::clients_in_range(w, map, world_loc, (ty.get_max_update_range)(w, id)) {
        if !deps::can_see(w, m, id)
            || !deps::mobile_in_range(w, m, world_loc, (ty.get_update_range)(w, id, m))
        {
            continue;
        }

        if update {
            match parent {
                None => (ty.send_info_to)(w, id, ns, &[]),
                Some(p) => {
                    match p {
                        EntityId::Item(_) => {
                            crate::packets::container::send_container_content_update(w, ns, id)
                        }
                        EntityId::Mobile(_) => {
                            crate::packets::equipment::send_equip_update(w, ns, id)
                        }
                    }
                    (ty.send_opl_packet_to)(w, id, ns);
                }
            }
        } else if flags.contains(ItemDelta::EQUIP_ONLY)
            && matches!(parent, Some(EntityId::Mobile(_)))
        {
            crate::packets::equipment::send_equip_update(w, ns, id);
            (ty.send_opl_packet_to)(w, id, ns);
        } else if deps::opl_enabled(w) && flags.contains(ItemDelta::PROPERTIES) {
            (ty.send_opl_packet_to)(w, id, ns);
        }
    }
}

/// `x.delete(..)`: the virtual call (an override runs; `delete_base` is what `base.delete(..)` reaches).
pub fn delete(w: &mut World, id: ItemId) {
    match w.item(id).map(|m| m.ty) {
        Some(ty) => (ty.delete)(w, id),
        None => delete_base(w, id),
    }
}

pub fn delete_base(w: &mut World, id: ItemId) {
    let Some(ty) = ty_of(w, id) else { return };
    if ops::is_deleted(w, id) {
        return;
    }

    (ty.on_delete)(w, id);
    ops::decay_unregister(w, id);

    let mut i = w.item(id).map_or(0, |i| i.items.len());
    while i > 0 {
        i -= 1;
        if let Some(child) = w.item(id).and_then(|c| c.items.get(i).copied())
            && let Some(ct) = ty_of(w, child)
        {
            (ct.on_parent_deleted)(w, child, EntityId::Item(id));
        }
    }

    ops::send_remove_packet(w, id);

    ops::set_flag(w, id, ImplFlag::DELETED, true);

    match parent_of(w, id) {
        Some(EntityId::Mobile(m)) => deps::mobile_remove_item(w, m, id),
        Some(EntityId::Item(p)) => {
            if let Some(pt) = ty_of(w, p) {
                (pt.remove_item)(w, p, id);
            }
        }
        None => {}
    }

    ops::clear_bounce(w, id);

    let (map, parent) = w.item(id).map_or((None, None), |i| (i.map, i.parent));
    if let Some(m) = map {
        if parent.is_none() {
            deps::map_on_leave(w, m, id);
        }
        if let Some(i) = w.item_mut(id) {
            i.map = None;
        }
    }

    // `World.RemoveEntity` runs and then OnAfterDelete on the detached object: the item stays in the table for the
    // handler's reads but `find_item` and the enumerations skip it (`World::detaching`).
    w.detaching.push(id.0);
    (ty.on_after_delete)(w, id);
    w.detaching.retain(|&s| s != id.0);

    // World.RemoveEntity(this)
    w.remove_item(id);
    (ty.clear_properties)(w, id);
}

/// `x.skip_serialization(..)`: the virtual call (an override runs; `skip_serialization_base` is what `base.skip_serialization(..)` reaches).
pub fn skip_serialization(w: &World, id: ItemId) -> bool {
    match w.item(id).map(|m| m.ty) {
        Some(ty) => (ty.skip_serialization)(w, id),
        None => skip_serialization_base(w, id),
    }
}

pub fn skip_serialization_base(w: &World, id: ItemId) -> bool {
    if let Some(EntityId::Item(p)) = parent_of(w, id)
        && let Some(pt) = ty_of(w, p)
    {
        return pt.is(&CONTAINER) && (pt.skips_child_serialization)(w, p);
    }
    false
}

pub fn set_location(w: &mut World, id: ItemId, value: Point3D) {
    let Some(i) = w.item(id) else { return };
    let old_location = i.location;
    if old_location == value {
        return;
    }
    let (map, parent, ty) = (i.map, i.parent, i.ty);

    if let Some(map) = map {
        if parent.is_none() {
            if old_location.x != 0 {
                for (ns, m) in
                    deps::clients_in_range(w, map, old_location, (ty.get_max_update_range)(w, id))
                {
                    if !deps::mobile_in_range(w, m, value, (ty.get_update_range)(w, id, m)) {
                        crate::packets::entity::send_remove_entity(w, ns, id.0);
                    }
                }
            }

            let old_loc = old_location;
            if let Some(i) = w.item_mut(id) {
                i.location = value;
            }

            ops::set_last_moved(w, id);

            for (ns, m) in deps::clients_in_range(w, map, value, (ty.get_max_update_range)(w, id)) {
                let (hs, no_move, delta_update) = (
                    w.net_state(ns).is_some_and(|d| d.caps.high_seas()),
                    w.item(id).is_some_and(|i| i.no_move_hs),
                    w.item(id)
                        .is_some_and(|i| i.delta_flags.contains(ItemDelta::UPDATE)),
                );
                if deps::can_see(w, m, id)
                    && deps::mobile_in_range(w, m, value, (ty.get_update_range)(w, id, m))
                    && (!hs
                        || !no_move
                        || delta_update
                        || !deps::mobile_in_range(w, m, old_loc, (ty.get_update_range)(w, id, m)))
                {
                    (ty.send_info_to)(w, id, ns, &[]);
                }
            }

            ops::rem_delta(w, id, ItemDelta::UPDATE);
        } else {
            if let Some(i) = w.item_mut(id) {
                i.location = value;
            }
            if matches!(parent, Some(EntityId::Item(_))) {
                ops::delta(w, id, ItemDelta::UPDATE);
            }
        }

        if parent.is_none() {
            deps::map_on_move(w, map, old_location, id);
        }
    } else if let Some(i) = w.item_mut(id) {
        i.location = value;
    }

    (ty.on_location_change)(w, id, old_location);
}

/// `x.send_properties_to(..)`: the virtual call (an override runs; `send_properties_to_base` is what `base.send_properties_to(..)` reaches).
pub fn send_properties_to(w: &mut World, id: ItemId, a2: NetStateId) {
    match w.item(id).map(|m| m.ty) {
        Some(ty) => (ty.send_properties_to)(w, id, a2),
        None => send_properties_to_base(w, id, a2),
    }
}

pub fn send_properties_to_base(w: &mut World, id: ItemId, ns: NetStateId) {
    if let Some(buf) = ops::property_list(w, id).map(|l| l.buffer().to_vec()) {
        w.send(ns, &buf);
    }
}

/// `x.add_name_property(..)`: the virtual call (an override runs; `add_name_property_base` is what `base.add_name_property(..)` reaches).
pub fn add_name_property(w: &World, id: ItemId, a2: &mut PropertyList) {
    match w.item(id).map(|m| m.ty) {
        Some(ty) => (ty.add_name_property)(w, id, a2),
        None => add_name_property_base(w, id, a2),
    }
}

pub fn add_name_property_base(w: &World, id: ItemId, list: &mut PropertyList) {
    let Some(i) = w.item(id) else { return };
    let name = ops::name(w, id);
    let label = (i.ty.label_number)(w, id);
    match name {
        None => {
            if i.amount <= 1 {
                list.add(label);
            } else {
                // ~1_NUMBER~ ~2_ITEMNAME~
                list.add_number_fmt(1050039, format_args!("{}\t#{}", i.amount, label));
            }
        }
        Some(name) => {
            if i.amount <= 1 {
                list.add_text(&name);
            } else {
                list.add_number_fmt(1050039, format_args!("{}\t{}", i.amount, name));
            }
        }
    }
}

pub fn add_loot_type_property(w: &World, id: ItemId, list: &mut PropertyList) {
    let Some(i) = w.item(id) else { return };
    if i.loot_type == LootType::Blessed {
        list.add(1038021); // blessed
    } else if i.loot_type == LootType::Cursed {
        list.add(1049643); // cursed
    } else if deps::insurance_enabled(w) && i.insured() {
        list.add(1061682); // <b>insured</b>
    }
}

pub fn add_resistance_properties(w: &World, id: ItemId, list: &mut PropertyList) {
    let Some(ty) = ty_of(w, id) else { return };
    let v = (ty.physical_resistance)(w, id);
    if v != 0 {
        list.add_int(1060448, v); // physical resist ~1_val~%
    }
    let v = (ty.fire_resistance)(w, id);
    if v != 0 {
        list.add_int(1060447, v); // fire resist ~1_val~%
    }
    let v = (ty.cold_resistance)(w, id);
    if v != 0 {
        list.add_int(1060445, v); // cold resist ~1_val~%
    }
    let v = (ty.poison_resistance)(w, id);
    if v != 0 {
        list.add_int(1060449, v); // poison resist ~1_val~%
    }
    let v = (ty.energy_resistance)(w, id);
    if v != 0 {
        list.add_int(1060446, v); // energy resist ~1_val~%
    }
}

pub fn add_weight_property(w: &World, id: ItemId, list: &mut PropertyList) {
    let weight = ops::pile_weight(w, id) + ops::get_total(w, id, TotalType::Weight);
    list.add_number_fmt(
        if weight == 1 { 1072788 } else { 1072789 },
        format_args!("{weight}"),
    );
}

/// `x.add_name_properties(..)`: the virtual call (an override runs; `add_name_properties_base` is what `base.add_name_properties(..)` reaches).
pub fn add_name_properties(w: &World, id: ItemId, a2: &mut PropertyList) {
    match w.item(id).map(|m| m.ty) {
        Some(ty) => (ty.add_name_properties)(w, id, a2),
        None => add_name_properties_base(w, id, a2),
    }
}

pub fn add_name_properties_base(w: &World, id: ItemId, list: &mut PropertyList) {
    let Some(i) = w.item(id) else { return };
    let ty = i.ty;
    (ty.add_name_property)(w, id, list);

    if ops::is_secure(w, id) {
        (ty.add_secure_property)(w, id, list);
    } else if ops::is_locked_down(w, id) {
        (ty.add_locked_down_property)(w, id, list);
    }

    if let Some(b) = i.blessed_for
        && !deps::mobile_deleted(w, b)
    {
        (ty.add_blessed_for_property)(w, id, list, b);
    }

    if (ty.display_loot_type)(w, id) {
        (ty.add_loot_type_property)(w, id, list);
    }

    if (ty.display_weight)(w, id) {
        (ty.add_weight_property)(w, id, list);
    }

    if i.quest_item() {
        (ty.add_quest_item_property)(w, id, list);
    }
}

pub fn add_quest_item_property(_w: &World, _id: ItemId, list: &mut PropertyList) {
    list.add(1072351); // Quest Item
}

pub fn add_secure_property(_w: &World, _id: ItemId, list: &mut PropertyList) {
    list.add(501644); // locked down & secure
}

pub fn add_locked_down_property(_w: &World, _id: ItemId, list: &mut PropertyList) {
    list.add(501643); // locked down
}

pub fn add_blessed_for_property(w: &World, _id: ItemId, list: &mut PropertyList, m: MobileId) {
    let name = w
        .mobile(m)
        .and_then(|mm| (mm.ty.name)(w, m))
        .unwrap_or_default();
    list.add_str(1062203, &name); // Blessed for ~1_NAME~
}

/// `x.get_child_properties(..)`: the virtual call (an override runs; `get_child_properties_base` is what `base.get_child_properties(..)` reaches).
pub fn get_child_properties(w: &World, id: ItemId, a2: &mut PropertyList, a3: ItemId) {
    match w.item(id).map(|m| m.ty) {
        Some(ty) => (ty.get_child_properties)(w, id, a2, a3),
        None => get_child_properties_base(w, id, a2, a3),
    }
}

pub fn get_child_properties_base(w: &World, id: ItemId, list: &mut PropertyList, item: ItemId) {
    match parent_of(w, id) {
        Some(EntityId::Item(p)) => {
            if let Some(t) = ty_of(w, p) {
                (t.get_child_properties)(w, p, list, item);
            }
        }
        Some(EntityId::Mobile(m)) => {
            if let Some(t) = w.mobile(m).map(|m| m.ty) {
                (t.get_child_properties)(w, m, list, item);
            }
        }
        None => {}
    }
}

/// `x.get_child_name_properties(..)`: the virtual call (an override runs; `get_child_name_properties_base` is what `base.get_child_name_properties(..)` reaches).
pub fn get_child_name_properties(w: &World, id: ItemId, a2: &mut PropertyList, a3: ItemId) {
    match w.item(id).map(|m| m.ty) {
        Some(ty) => (ty.get_child_name_properties)(w, id, a2, a3),
        None => get_child_name_properties_base(w, id, a2, a3),
    }
}

pub fn get_child_name_properties_base(
    w: &World,
    id: ItemId,
    list: &mut PropertyList,
    item: ItemId,
) {
    match parent_of(w, id) {
        Some(EntityId::Item(p)) => {
            if let Some(t) = ty_of(w, p) {
                (t.get_child_name_properties)(w, p, list, item);
            }
        }
        Some(EntityId::Mobile(m)) => {
            if let Some(t) = w.mobile(m).map(|m| m.ty) {
                (t.get_child_name_properties)(w, m, list, item);
            }
        }
        None => {}
    }
}

pub fn check_race(w: &mut World, id: ItemId, from: MobileId, message: bool) -> bool {
    let Some(ty) = ty_of(w, id) else { return false };
    let required = (ty.required_races)(w, id);
    let (allowed, plural, gargoyle) = deps::race_allows(w, from, required);
    if allowed {
        return true;
    }
    if !message {
        return false;
    }

    if required == crate::race::ALLOW_GARGOYLES_ONLY {
        deps::local_overhead_message(w, from, MessageType::REGULAR, 0x3B2, 1111707); // Only gargoyles can wear this.
    } else if gargoyle {
        deps::local_overhead_message(w, from, MessageType::REGULAR, 0x3B2, 1111708); // Gargoyles can't wear this.
    } else if required == crate::race::ALLOW_ELVES_ONLY {
        deps::send_localized_message(w, from, 1072203); // Only Elves may use this.
    } else {
        deps::send_message(w, from, &format!("{plural} may not use this."));
    }
    false
}

/// `x.can_equip(..)`: the virtual call (an override runs; `can_equip_base` is what `base.can_equip(..)` reaches).
pub fn can_equip(w: &mut World, id: ItemId, a2: MobileId) -> bool {
    match w.item(id).map(|m| m.ty) {
        Some(ty) => (ty.can_equip)(w, id, a2),
        None => can_equip_base(w, id, a2),
    }
}

pub fn can_equip_base(w: &mut World, id: ItemId, m: MobileId) -> bool {
    let layer = w.item(id).map_or(Layer::Invalid, |i| i.layer);
    layer != Layer::Invalid && deps::find_item_on_layer(w, m, layer).is_none()
}

/// `x.get_child_context_menu_entries(..)`: the virtual call (an override runs; `get_child_context_menu_entries_base` is what `base.get_child_context_menu_entries(..)` reaches).
pub fn get_child_context_menu_entries(
    w: &mut World,
    id: ItemId,
    a2: MobileId,
    a3: &mut Vec<ContextMenuEntry>,
    a4: ItemId,
) {
    match w.item(id).map(|m| m.ty) {
        Some(ty) => (ty.get_child_context_menu_entries)(w, id, a2, a3, a4),
        None => get_child_context_menu_entries_base(w, id, a2, a3, a4),
    }
}

pub fn get_child_context_menu_entries_base(
    w: &mut World,
    id: ItemId,
    from: MobileId,
    list: &mut Vec<ContextMenuEntry>,
    item: ItemId,
) {
    match parent_of(w, id) {
        Some(EntityId::Item(p)) => {
            if let Some(t) = ty_of(w, p) {
                (t.get_child_context_menu_entries)(w, p, from, list, item);
            }
        }
        Some(EntityId::Mobile(m)) => {
            if let Some(t) = w.mobile(m).map(|m| m.ty) {
                (t.get_child_context_menu_entries)(w, m, from, list, item);
            }
        }
        None => {}
    }
}

/// `x.get_context_menu_entries(..)`: the virtual call (an override runs; `get_context_menu_entries_base` is what `base.get_context_menu_entries(..)` reaches).
pub fn get_context_menu_entries(
    w: &mut World,
    id: ItemId,
    a2: MobileId,
    a3: &mut Vec<ContextMenuEntry>,
) {
    match w.item(id).map(|m| m.ty) {
        Some(ty) => (ty.get_context_menu_entries)(w, id, a2, a3),
        None => get_context_menu_entries_base(w, id, a2, a3),
    }
}

pub fn get_context_menu_entries_base(
    w: &mut World,
    id: ItemId,
    from: MobileId,
    list: &mut Vec<ContextMenuEntry>,
) {
    match parent_of(w, id) {
        Some(EntityId::Item(p)) => {
            if let Some(t) = ty_of(w, p) {
                (t.get_child_context_menu_entries)(w, p, from, list, id);
            }
        }
        Some(EntityId::Mobile(m)) => {
            if let Some(t) = w.mobile(m).map(|m| m.ty) {
                (t.get_child_context_menu_entries)(w, m, from, list, id);
            }
        }
        None => {}
    }
}

/// Shared by `OnParentDeath` and `OnInventoryDeath` (they differ only in the first answer).
fn death_move(
    w: &mut World,
    id: ItemId,
    parent: MobileId,
    not_movable: DeathMoveResult,
) -> DeathMoveResult {
    let Some(ty) = ty_of(w, id) else {
        return DeathMoveResult::MoveToCorpse;
    };
    if !w.item(id).is_some_and(|i| i.movable()) {
        return not_movable;
    }
    let Some(mty) = w.mobile(parent).map(|m| m.ty) else {
        return DeathMoveResult::MoveToCorpse;
    };
    if (mty.keeps_items_on_death)(w, parent) {
        return DeathMoveResult::MoveToBackpack;
    }
    if (ty.check_blessed)(w, id, parent) {
        return DeathMoveResult::MoveToBackpack;
    }
    if (ty.check_newbied)(w, id) && !(mty.murderer)(w, parent) {
        return DeathMoveResult::MoveToBackpack;
    }
    if w.mobile(parent).is_some_and(|m| m.player) && (ty.nontransferable)(w, id) {
        return DeathMoveResult::MoveToBackpack;
    }
    DeathMoveResult::MoveToCorpse
}

/// `x.on_parent_death(..)`: the virtual call (an override runs; `on_parent_death_base` is what `base.on_parent_death(..)` reaches).
pub fn on_parent_death(w: &mut World, id: ItemId, a2: MobileId) -> DeathMoveResult {
    match w.item(id).map(|m| m.ty) {
        Some(ty) => (ty.on_parent_death)(w, id, a2),
        None => on_parent_death_base(w, id, a2),
    }
}

pub fn on_parent_death_base(w: &mut World, id: ItemId, parent: MobileId) -> DeathMoveResult {
    death_move(w, id, parent, DeathMoveResult::RemainEquipped)
}

pub fn on_inventory_death(w: &mut World, id: ItemId, parent: MobileId) -> DeathMoveResult {
    death_move(w, id, parent, DeathMoveResult::MoveToBackpack)
}

pub fn move_to_world(w: &mut World, id: ItemId, location: Point3D) {
    let map = w.item(id).and_then(|i| i.map);
    ops::move_to_world_map(w, id, location, map);
}

pub fn label_loot_type_to(w: &mut World, id: ItemId, to: MobileId) {
    match w.item(id).map(|i| i.loot_type) {
        Some(LootType::Blessed) => ops::label_to_number(w, id, to, 1041362, ""), // (blessed)
        Some(LootType::Cursed) => ops::label_to(w, id, to, "(cursed)"),
        _ => {}
    }
}

pub fn can_decay(w: &World, id: ItemId) -> bool {
    let Some(i) = w.item(id) else { return false };
    if !((i.ty.decays)(w, id) && i.map != Some(MapId::INTERNAL)) {
        return false;
    }
    match i.parent {
        None => true,
        Some(EntityId::Item(p)) => {
            w.item(p)
                .is_some_and(|pi| pi.ty.is(&CONTAINER) && (pi.ty.contents_decay)(w, p))
                && !ops::is_locked_down(w, id)
                && !ops::is_secure(w, id)
        }
        Some(EntityId::Mobile(_)) => false,
    }
}

/// `x.on_decay(..)`: the virtual call (an override runs; `on_decay_base` is what `base.on_decay(..)` reaches).
pub fn on_decay(w: &mut World, id: ItemId) -> bool {
    match w.item(id).map(|m| m.ty) {
        Some(ty) => (ty.on_decay)(w, id),
        None => on_decay_base(w, id),
    }
}

pub fn on_decay_base(w: &mut World, id: ItemId) -> bool {
    let Some(ty) = ty_of(w, id) else { return false };
    (ty.can_decay)(w, id) && deps::region_on_decay(w, id)
}

/// `x.can_stack_with(..)`: the virtual call (an override runs; `can_stack_with_base` is what `base.can_stack_with(..)` reaches).
pub fn can_stack_with(w: &World, id: ItemId, a2: ItemId) -> bool {
    match w.item(id).map(|m| m.ty) {
        Some(ty) => (ty.can_stack_with)(w, id, a2),
        None => can_stack_with_base(w, id, a2),
    }
}

pub fn can_stack_with_base(w: &World, id: ItemId, dropped: ItemId) -> bool {
    let (Some(a), Some(d)) = (w.item(id), w.item(dropped)) else {
        return false;
    };
    d.stackable()
        && a.stackable()
        && std::ptr::eq(d.ty, a.ty)
        && (d.ty.item_id)(w, dropped) == (a.ty.item_id)(w, id)
        && (d.ty.hue)(w, dropped) == (a.ty.hue)(w, id)
        && ops::name(w, dropped) == ops::name(w, id)
        && d.amount + a.amount <= 60000
        && dropped != id
}

/// `x.stack_with(..)`: the virtual call (an override runs; `stack_with_base` is what `base.stack_with(..)` reaches).
pub fn stack_with(w: &mut World, id: ItemId, a2: Option<MobileId>, a3: ItemId, a4: bool) -> bool {
    match w.item(id).map(|m| m.ty) {
        Some(ty) => (ty.stack_with)(w, id, a2, a3, a4),
        None => stack_with_base(w, id, a2, a3, a4),
    }
}

pub fn stack_with_base(
    w: &mut World,
    id: ItemId,
    from: Option<MobileId>,
    dropped: ItemId,
    play_sound: bool,
) -> bool {
    let Some(ty) = ty_of(w, id) else { return false };
    if !(ty.can_stack_with)(w, id, dropped) {
        return false;
    }

    let (dropped_loot, dropped_pc, dropped_amount, dty) = match w.item(dropped) {
        Some(d) => (d.loot_type, d.player_constructed, d.amount, d.ty),
        None => return false,
    };
    if let Some(i) = w.item_mut(id)
        && i.loot_type != dropped_loot
    {
        i.loot_type = LootType::Regular;
    }

    let amount = w.item(id).map_or(0, |i| i.amount);
    ops::set_amount(w, id, amount + dropped_amount);
    if let Some(i) = w.item_mut(id)
        && i.player_constructed != dropped_pc
    {
        i.player_constructed = false;
    }

    (dty.delete)(w, dropped);

    if play_sound && let Some(from) = from {
        let mut sound_id = (ty.get_drop_sound)(w, id);
        if sound_id == -1 {
            sound_id = 0x42;
        }
        let loc = ops::get_world_location(w, id);
        deps::send_sound(w, from, sound_id, loc);
    }

    true
}

/// `x.on_drag_drop(..)`: the virtual call (an override runs; `on_drag_drop_base` is what `base.on_drag_drop(..)` reaches).
pub fn on_drag_drop(w: &mut World, id: ItemId, a2: MobileId, a3: ItemId) -> bool {
    match w.item(id).map(|m| m.ty) {
        Some(ty) => (ty.on_drag_drop)(w, id, a2, a3),
        None => on_drag_drop_base(w, id, a2, a3),
    }
}

pub fn on_drag_drop_base(w: &mut World, id: ItemId, from: MobileId, dropped: ItemId) -> bool {
    let container_parent = match parent_of(w, id) {
        Some(EntityId::Item(p)) if ty_of(w, p).is_some_and(|t| t.is(&CONTAINER)) => Some(p),
        _ => None,
    };
    let success = match container_parent {
        Some(p) => {
            let pt = ty_of(w, p).expect("checked above");
            (pt.on_stack_attempt)(w, p, from, id, dropped)
        }
        None => {
            let ty = ty_of(w, id).expect("item exists");
            (ty.stack_with)(w, id, Some(from), dropped, true)
        }
    };

    if let Some(sp) = w.item(id).and_then(|i| i.spawner).filter(|_| success) {
        crate::mobile::part2::deps::spawner_remove(w, sp, EntityId::Item(id));
        ops::set_spawner(w, id, None);
    }

    success
}

pub fn append_child_properties(w: &World, id: ItemId, list: &mut PropertyList) {
    match parent_of(w, id) {
        Some(EntityId::Item(p)) => {
            if let Some(t) = ty_of(w, p) {
                (t.get_child_properties)(w, p, list, id);
            }
        }
        Some(EntityId::Mobile(m)) => {
            if let Some(t) = w.mobile(m).map(|m| m.ty) {
                (t.get_child_properties)(w, m, list, id);
            }
        }
        None => {}
    }
}

/// `x.append_child_name_properties(..)`: the virtual call (an override runs; `append_child_name_properties_base` is what `base.append_child_name_properties(..)` reaches).
pub fn append_child_name_properties(w: &World, id: ItemId, a2: &mut PropertyList) {
    match w.item(id).map(|m| m.ty) {
        Some(ty) => (ty.append_child_name_properties)(w, id, a2),
        None => append_child_name_properties_base(w, id, a2),
    }
}

pub fn append_child_name_properties_base(w: &World, id: ItemId, list: &mut PropertyList) {
    match parent_of(w, id) {
        Some(EntityId::Item(p)) => {
            if let Some(t) = ty_of(w, p) {
                (t.get_child_name_properties)(w, p, list, id);
            }
        }
        Some(EntityId::Mobile(m)) => {
            if let Some(t) = w.mobile(m).map(|m| m.ty) {
                (t.get_child_name_properties)(w, m, list, id);
            }
        }
        None => {}
    }
}

/// `x.clear_properties(..)`: the virtual call (an override runs; `clear_properties_base` is what `base.clear_properties(..)` reaches).
pub fn clear_properties(w: &mut World, id: ItemId) {
    match w.item(id).map(|m| m.ty) {
        Some(ty) => (ty.clear_properties)(w, id),
        None => clear_properties_base(w, id),
    }
}

pub fn clear_properties_base(w: &mut World, id: ItemId) {
    if let Some(i) = w.item_mut(id) {
        i.property_list = None;
    }
}

/// `x.invalidate_properties(..)`: the virtual call (an override runs; `invalidate_properties_base` is what `base.invalidate_properties(..)` reaches).
pub fn invalidate_properties(w: &mut World, id: ItemId) {
    match w.item(id).map(|m| m.ty) {
        Some(ty) => (ty.invalidate_properties)(w, id),
        None => invalidate_properties_base(w, id),
    }
}

pub fn invalidate_properties_base(w: &mut World, id: ItemId) {
    if !deps::opl_enabled(w) {
        return;
    }
    let Some(i) = w.item(id) else { return };
    // Always a bug in the property getter (never invalidate from inside
    // GetProperties). Rust cannot reach this: the list is built from `&World`.
    if i.property_list.as_ref().is_some_and(|l| l.is_building) {
        tracing::error!(
            "{} called InvalidateProperties() while its property list was being built.",
            i.ty.full_name
        );
        return;
    }
    let ty = i.ty;

    if i.map.is_some_and(|m| m != MapId::INTERNAL) && !w.loading() {
        let (old_hash, new_hash);
        if let Some(mut list) = w.item_mut(id).and_then(|i| i.property_list.take()) {
            old_hash = Some(list.hash());
            list.reset();
            ops::initialize_property_list(w, id, &mut list);
            new_hash = list.hash();
            if let Some(i) = w.item_mut(id) {
                i.property_list = Some(list);
            }
        } else {
            old_hash = None;
            new_hash = ops::property_list(w, id).map_or(0, |l| l.hash());
        }

        if old_hash != Some(new_hash) {
            ops::delta(w, id, ItemDelta::PROPERTIES);
        }
    } else {
        (ty.clear_properties)(w, id);
    }
}

pub fn get_packet_flags(w: &World, id: ItemId) -> i32 {
    let Some(i) = w.item(id) else { return 0 };
    let mut flags = 0;
    if !i.visible() {
        flags |= 0x80;
    }
    if i.movable() || (i.ty.force_show_properties)(w, id) {
        flags |= 0x20;
    }
    flags
}

/// `x.deserialize(..)`: the virtual call (an override runs; `deserialize_base` is what `base.deserialize(..)` reaches).
pub fn deserialize(w: &mut World, id: ItemId, a2: &mut dyn GenericReader) -> muo_io::Result<()> {
    match w.item(id).map(|m| m.ty) {
        Some(ty) => (ty.deserialize)(w, id, a2),
        None => deserialize_base(w, id, a2),
    }
}

pub fn deserialize_base(
    w: &mut World,
    id: ItemId,
    reader: &mut dyn GenericReader,
) -> muo_io::Result<()> {
    crate::item::serialize::deserialize(w, id, reader)
}

/// `x.send_info_to(..)`: the virtual call (an override runs; `send_info_to_base` is what `base.send_info_to(..)` reaches).
pub fn send_info_to(w: &mut World, id: ItemId, a2: NetStateId, a3: &[u8]) {
    match w.item(id).map(|m| m.ty) {
        Some(ty) => (ty.send_info_to)(w, id, a2, a3),
        None => send_info_to_base(w, id, a2, a3),
    }
}

pub fn send_info_to_base(w: &mut World, id: ItemId, ns: NetStateId, world: &[u8]) {
    if let Some(ty) = ty_of(w, id) {
        (ty.send_world_packet_to)(w, id, ns, world);
        (ty.send_opl_packet_to)(w, id, ns);
    }
}

/// `x.send_opl_packet_to(..)`: the virtual call (an override runs; `send_opl_packet_to_base` is what `base.send_opl_packet_to(..)` reaches).
pub fn send_opl_packet_to(w: &mut World, id: ItemId, a2: NetStateId) {
    match w.item(id).map(|m| m.ty) {
        Some(ty) => (ty.send_opl_packet_to)(w, id, a2),
        None => send_opl_packet_to_base(w, id, a2),
    }
}

pub fn send_opl_packet_to_base(w: &mut World, id: ItemId, ns: NetStateId) {
    if deps::opl_enabled(w) {
        crate::item::ops::property_list(w, id); // the PropertyList getter builds it
        crate::packets::entity::send_opl_info_item(w, ns, id);
    }
}

/// `x.send_world_packet_to(..)`: the virtual call (an override runs; `send_world_packet_to_base` is what `base.send_world_packet_to(..)` reaches).
pub fn send_world_packet_to(w: &mut World, id: ItemId, a2: NetStateId, a3: &[u8]) {
    match w.item(id).map(|m| m.ty) {
        Some(ty) => (ty.send_world_packet_to)(w, id, a2, a3),
        None => send_world_packet_to_base(w, id, a2, a3),
    }
}

pub fn send_world_packet_to_base(w: &mut World, id: ItemId, ns: NetStateId, world: &[u8]) {
    if !world.is_empty() {
        w.send(ns, world);
        return;
    }
    crate::packets::item::send_world_item(w, ns, id);
}

/// `x.update_total(..)`: the virtual call (an override runs; `update_total_base` is what `base.update_total(..)` reaches).
pub fn update_total(w: &mut World, id: ItemId, a2: ItemId, a3: TotalType, a4: i32) {
    match w.item(id).map(|m| m.ty) {
        Some(ty) => (ty.update_total)(w, id, a2, a3, a4),
        None => update_total_base(w, id, a2, a3, a4),
    }
}

pub fn update_total_base(w: &mut World, id: ItemId, sender: ItemId, ty: TotalType, delta: i32) {
    let Some(it) = ty_of(w, id) else { return };
    if (it.is_virtual_item)(w, id) {
        return;
    }
    let Some(i) = w.item(id) else { return };
    match i.parent {
        Some(EntityId::Item(p)) => {
            if let Some(t) = ty_of(w, p) {
                (t.update_total)(w, p, sender, ty, delta);
            }
        }
        Some(EntityId::Mobile(m)) => {
            if let Some(t) = w.mobile(m).map(|m| m.ty) {
                (t.update_total)(w, m, sender, ty, delta);
            }
        }
        None => {
            if let Some(h) = i.held_by
                && let Some(t) = w.mobile(h).map(|m| m.ty)
            {
                (t.update_total)(w, h, sender, ty, delta);
            }
        }
    }
}

/// `x.handle_invalid_transfer(..)`: the virtual call (an override runs; `handle_invalid_transfer_base` is what `base.handle_invalid_transfer(..)` reaches).
pub fn handle_invalid_transfer(w: &mut World, id: ItemId, a2: MobileId) {
    match w.item(id).map(|m| m.ty) {
        Some(ty) => (ty.handle_invalid_transfer)(w, id, a2),
        None => handle_invalid_transfer_base(w, id, a2),
    }
}

pub fn handle_invalid_transfer_base(w: &mut World, id: ItemId, from: MobileId) {
    // OSI sends 1074769, bug!
    if w.item(id).is_some_and(|i| i.quest_item()) {
        // You can only drop quest items into the top-most level of your backpack while you still need them
        // for your quest.
        deps::send_localized_message(w, from, 1049343);
    }
}

/// `x.add_item(..)`: the virtual call (an override runs; `add_item_base` is what `base.add_item(..)` reaches).
pub fn add_item(w: &mut World, id: ItemId, a2: ItemId) {
    match w.item(id).map(|m| m.ty) {
        Some(ty) => (ty.add_item)(w, id, a2),
        None => add_item_base(w, id, a2),
    }
}

pub fn add_item_base(w: &mut World, id: ItemId, item: ItemId) {
    if ops::is_deleted(w, item) || w.item(item).and_then(|i| i.parent) == Some(EntityId::Item(id)) {
        return;
    }
    let Some(ty) = ty_of(w, id) else { return };

    if item == id {
        tracing::warn!(
            "Adding item to itself: ({:?} {}).AddItem({:?} {})",
            id.0,
            ty.full_name,
            item.0,
            ty.full_name
        );
        return;
    }

    if ops::is_child_of(w, id, Some(EntityId::Item(item))) {
        tracing::warn!(
            "Adding parent item to child: [{:?} {}].AddItem( [{:?}] )",
            id.0,
            ty.full_name,
            item.0
        );
        return;
    }

    let ity = ty_of(w, item).expect("exists");
    match parent_of(w, item) {
        Some(EntityId::Mobile(m)) => deps::mobile_remove_item(w, m, item),
        Some(EntityId::Item(p)) => {
            if let Some(pt) = ty_of(w, p) {
                (pt.remove_item)(w, p, item);
            }
        }
        None => ops::send_remove_packet(w, item),
    }

    ops::set_parent(w, item, Some(EntityId::Item(id)));
    let map = w.item(id).and_then(|i| i.map);
    ops::set_map(w, item, map);

    if let Some(i) = w.item_mut(id) {
        i.items.push(item);
        if !i.ty.is(&CONTAINER) {
            i.version += 1;
        }
    }

    if !(ity.is_virtual_item)(w, item) {
        let gold = ops::get_total(w, item, TotalType::Gold);
        let items = ops::get_total(w, item, TotalType::Items);
        let weight = ops::get_total(w, item, TotalType::Weight) + ops::pile_weight(w, item);
        ops::update_total(w, id, item, TotalType::Gold, gold);
        ops::update_total(w, id, item, TotalType::Items, items + 1);
        ops::update_total(w, id, item, TotalType::Weight, weight);
    }

    ops::delta(w, item, ItemDelta::UPDATE);
    ops::update_decay_registration(w, item);

    (ity.on_added)(w, item, EntityId::Item(id));
    (ty.on_item_added)(w, id, item);
}

/// `x.on_delete(..)`: the virtual call (an override runs; `on_delete_base` is what `base.on_delete(..)` reaches).
pub fn on_delete(w: &mut World, id: ItemId) {
    match w.item(id).map(|m| m.ty) {
        Some(ty) => (ty.on_delete)(w, id),
        None => on_delete_base(w, id),
    }
}

pub fn on_delete_base(w: &mut World, id: ItemId) {
    if let Some(sp) = w.item(id).and_then(|i| i.spawner) {
        crate::mobile::part2::deps::spawner_remove(w, sp, EntityId::Item(id));
        ops::set_spawner(w, id, None);
    }
}

/// `x.remove_item(..)`: the virtual call (an override runs; `remove_item_base` is what `base.remove_item(..)` reaches).
pub fn remove_item(w: &mut World, id: ItemId, a2: ItemId) {
    match w.item(id).map(|m| m.ty) {
        Some(ty) => (ty.remove_item)(w, id, a2),
        None => remove_item_base(w, id, a2),
    }
}

pub fn remove_item_base(w: &mut World, id: ItemId, item: ItemId) {
    let Some(ty) = ty_of(w, id) else { return };
    let Some(i) = w.item(id) else { return };

    // Bulk removal (Delete, purges) walks children back to front; check the tail first.
    let index = if i.items.last() == Some(&item) {
        Some(i.items.len() - 1)
    } else {
        i.items.iter().position(|&x| x == item)
    };
    let Some(index) = index else { return };

    if let Some(i) = w.item_mut(id) {
        i.items.remove(index);
        if !i.ty.is(&CONTAINER) {
            i.version += 1;
        }
    }

    ops::send_remove_packet(w, item);

    let ity = ty_of(w, item);
    let virtual_item = ity.is_some_and(|t| (t.is_virtual_item)(w, item));
    if !virtual_item {
        let gold = ops::get_total(w, item, TotalType::Gold);
        let items = ops::get_total(w, item, TotalType::Items);
        let weight = ops::get_total(w, item, TotalType::Weight) + ops::pile_weight(w, item);
        ops::update_total(w, id, item, TotalType::Gold, -gold);
        ops::update_total(w, id, item, TotalType::Items, -(items + 1));
        ops::update_total(w, id, item, TotalType::Weight, -weight);
    }

    ops::set_parent(w, item, None);
    ops::update_decay_registration(w, item);

    if let Some(t) = ity {
        (t.on_removed)(w, item, EntityId::Item(id));
    }
    (ty.on_item_removed)(w, id, item);
}

/// `x.dupe(..)`: the virtual call (an override runs; `dupe_base` is what `base.dupe(..)` reaches).
pub fn dupe(w: &mut World, id: ItemId, a2: ItemId) {
    match w.item(id).map(|m| m.ty) {
        Some(ty) => (ty.dupe)(w, id, a2),
        None => dupe_base(w, id, a2),
    }
}

pub fn dupe_base(w: &mut World, id: ItemId, new_item: ItemId) {
    // CopyProperties: every public settable property of the class that is not [IgnoreDupe] is copied
    // (reflection, in declaration order). These are the ones `Item` itself has; a subclass's own properties
    // are copied by its `dupe` override.
    // shortcut: no reflection, upgrade if a subclass relies on an inherited property not listed here.
    let Some(src) = w.item(id) else { return };
    let (item_id, hue, amount, layer, direction, loot) = (
        src.item_id,
        src.hue,
        src.amount,
        src.layer,
        src.direction,
        src.loot_type,
    );
    let (name, weight, location, map) =
        (ops::name(w, id), ops::weight(w, id), src.location, src.map);
    let (movable, visible, stackable, quest, insured, paid) = (
        src.movable(),
        src.visible(),
        src.stackable(),
        src.quest_item(),
        src.insured(),
        src.paid_insurance(),
    );
    let (blessed_for, pc, no_move, last_moved) = (
        src.blessed_for,
        src.player_constructed,
        src.no_move_hs,
        src.last_moved,
    );
    let (temp_flags, saved_flags) = (src.temp_flags, src.saved_flags);
    let ty = src.ty;

    // TempFlags and SavedFlags are public settable properties without [IgnoreDupe], so CopyProperties copied them
    // (the lockdown bits included).
    ops::set_temp_flags(w, new_item, temp_flags);
    ops::set_saved_flags(w, new_item, saved_flags);
    (ty.set_item_id)(w, new_item, item_id);
    (ty.set_hue)(w, new_item, hue);
    ops::set_amount(w, new_item, amount);
    (ty.set_layer)(w, new_item, layer);
    ops::set_direction(w, new_item, direction);
    ops::set_loot_type(w, new_item, loot);
    ops::set_name(w, new_item, name);
    ops::set_weight(w, new_item, weight);
    ops::set_movable(w, new_item, movable);
    ops::set_visible(w, new_item, visible);
    ops::set_stackable(w, new_item, stackable);
    ops::set_quest_item(w, new_item, quest);
    ops::set_insured(w, new_item, insured);
    ops::set_paid_insurance(w, new_item, paid);
    ops::set_blessed_for(w, new_item, blessed_for);
    ops::set_map(w, new_item, map);
    ops::set_location(w, new_item, location);
    if let Some(n) = w.item_mut(new_item) {
        n.player_constructed = pc;
        n.no_move_hs = no_move;
    }
    ops::set_last_moved_value(w, new_item, last_moved);

    (ty.on_after_duped)(w, id, new_item);
}

/// `x.on_dropped_to_mobile(..)`: the virtual call (an override runs; `on_dropped_to_mobile_base` is what `base.on_dropped_to_mobile(..)` reaches).
pub fn on_dropped_to_mobile(w: &mut World, id: ItemId, a2: MobileId, a3: MobileId) -> bool {
    match w.item(id).map(|m| m.ty) {
        Some(ty) => (ty.on_dropped_to_mobile)(w, id, a2, a3),
        None => on_dropped_to_mobile_base(w, id, a2, a3),
    }
}

pub fn on_dropped_to_mobile_base(
    w: &mut World,
    id: ItemId,
    from: MobileId,
    _target: MobileId,
) -> bool {
    let nontransferable = ty_of(w, id).is_some_and(|t| (t.nontransferable)(w, id));
    if nontransferable && w.mobile(from).is_some_and(|m| m.player) {
        handle_invalid_transfer(w, id, from);
        return false;
    }
    true
}

fn mobile_deleted_or_missing(w: &World, m: MobileId) -> bool {
    deps::mobile_deleted(w, m)
}

/// `x.drop_to_mobile(..)`: the virtual call (an override runs; `drop_to_mobile_base` is what `base.drop_to_mobile(..)` reaches).
pub fn drop_to_mobile(w: &mut World, id: ItemId, a2: MobileId, a3: MobileId, a4: Point3D) -> bool {
    match w.item(id).map(|m| m.ty) {
        Some(ty) => (ty.drop_to_mobile)(w, id, a2, a3, a4),
        None => drop_to_mobile_base(w, id, a2, a3, a4),
    }
}

pub fn drop_to_mobile_base(
    w: &mut World,
    id: ItemId,
    from: MobileId,
    target: MobileId,
    _p: Point3D,
) -> bool {
    let Some(ty) = ty_of(w, id) else { return false };
    if ops::is_deleted(w, id)
        || mobile_deleted_or_missing(w, from)
        || mobile_deleted_or_missing(w, target)
    {
        return false;
    }
    let (Some(fm), Some(tm)) = (w.mobile(from), w.mobile(target)) else {
        return false;
    };
    if fm.map != tm.map || fm.map.is_none() {
        return false;
    }
    let (fty, tty) = (fm.ty, tm.ty);
    let tloc = tm.location;
    let staff = fm.access_level >= AccessLevel::GameMaster;
    if !(staff || deps::mobile_in_range(w, from, tloc, 2)) {
        return false;
    }
    (fty.can_see_mobile)(w, from, target)
        && deps::in_los_mobile(w, from, target)
        && (fty.on_dropped_item_to_mobile)(w, from, id, target)
        && (ty.on_dropped_to_mobile)(w, id, from, target)
        && (tty.on_drag_drop)(w, target, from, id)
}

/// `x.on_dropped_into(..)`: the virtual call (an override runs; `on_dropped_into_base` is what `base.on_dropped_into(..)` reaches).
pub fn on_dropped_into(w: &mut World, id: ItemId, a2: MobileId, a3: ItemId, a4: Point3D) -> bool {
    match w.item(id).map(|m| m.ty) {
        Some(ty) => (ty.on_dropped_into)(w, id, a2, a3, a4),
        None => on_dropped_into_base(w, id, a2, a3, a4),
    }
}

pub fn on_dropped_into_base(
    w: &mut World,
    id: ItemId,
    from: MobileId,
    target: ItemId,
    p: Point3D,
) -> bool {
    let Some(fty) = w.mobile(from).map(|m| m.ty) else {
        return false;
    };
    if !(fty.on_dropped_item_into)(w, from, id, target, p) {
        return false;
    }

    let nontransferable = ty_of(w, id).is_some_and(|t| (t.nontransferable)(w, id));
    let backpack = w.mobile(from).and_then(|m| m.backpack);
    if nontransferable && w.mobile(from).is_some_and(|m| m.player) && Some(target) != backpack {
        handle_invalid_transfer(w, id, from);
        return false;
    }

    match ty_of(w, target) {
        Some(tt) => (tt.on_drag_drop_into)(w, target, from, id, p),
        None => false,
    }
}

/// Shared prefix of OnDroppedOnto and DropToItem.
fn drop_item_precheck(w: &mut World, from: MobileId, target: ItemId) -> bool {
    if deps::mobile_deleted(w, from) || ops::is_deleted(w, target) {
        return false;
    }
    let (Some(fm), Some(tm)) = (w.mobile(from), w.item(target)) else {
        return false;
    };
    if fm.map != tm.map || fm.map.is_none() || tm.map.is_none() {
        return false;
    }
    let staff = fm.access_level >= AccessLevel::GameMaster;
    let fty = fm.ty;
    let tloc = ops::get_world_location(w, target);
    if !staff && !deps::mobile_in_range(w, from, tloc, 2) {
        return false;
    }
    if !(fty.can_see_item)(w, from, target) || !deps::in_los_item(w, from, target) {
        return false;
    }
    let tty = w.item(target).expect("exists").ty;
    (tty.is_accessible_to)(w, target, from)
}

pub fn on_dropped_onto(w: &mut World, id: ItemId, from: MobileId, target: ItemId) -> bool {
    if ops::is_deleted(w, id) || !drop_item_precheck(w, from, target) {
        return false;
    }
    let Some(fty) = w.mobile(from).map(|m| m.ty) else {
        return false;
    };
    if !(fty.on_dropped_item_onto)(w, from, id, target) {
        return false;
    }

    let nontransferable = ty_of(w, id).is_some_and(|t| (t.nontransferable)(w, id));
    let backpack = w.mobile(from).and_then(|m| m.backpack);
    if nontransferable && w.mobile(from).is_some_and(|m| m.player) && Some(target) != backpack {
        handle_invalid_transfer(w, id, from);
        return false;
    }

    match ty_of(w, target) {
        Some(tt) => (tt.on_drag_drop)(w, target, from, id),
        None => false,
    }
}

/// `x.drop_to_item(..)`: the virtual call (an override runs; `drop_to_item_base` is what `base.drop_to_item(..)` reaches).
pub fn drop_to_item(w: &mut World, id: ItemId, a2: MobileId, a3: ItemId, a4: Point3D) -> bool {
    match w.item(id).map(|m| m.ty) {
        Some(ty) => (ty.drop_to_item)(w, id, a2, a3, a4),
        None => drop_to_item_base(w, id, a2, a3, a4),
    }
}

pub fn drop_to_item_base(
    w: &mut World,
    id: ItemId,
    from: MobileId,
    target: ItemId,
    p: Point3D,
) -> bool {
    let Some(ty) = ty_of(w, id) else { return false };
    if ops::is_deleted(w, id) || !drop_item_precheck(w, from, target) {
        return false;
    }

    if let Some(EntityId::Mobile(root)) = ops::root_parent(w, target)
        && !deps::check_nonlocal_drop(w, root, from, id, target)
    {
        return false;
    }

    let Some(fty) = w.mobile(from).map(|m| m.ty) else {
        return false;
    };
    if !(fty.on_dropped_item_to_item)(w, from, id, target, p) {
        return false;
    }

    if ty_of(w, target).is_some_and(|t| t.is(&CONTAINER)) && p.x != -1 && p.y != -1 {
        return (ty.on_dropped_into)(w, id, from, target, p);
    }
    (ty.on_dropped_onto)(w, id, from, target)
}

/// `x.on_dropped_to_world(..)`: the virtual call (an override runs; `on_dropped_to_world_base` is what `base.on_dropped_to_world(..)` reaches).
pub fn on_dropped_to_world(w: &mut World, id: ItemId, a2: MobileId, a3: Point3D) -> bool {
    match w.item(id).map(|m| m.ty) {
        Some(ty) => (ty.on_dropped_to_world)(w, id, a2, a3),
        None => on_dropped_to_world_base(w, id, a2, a3),
    }
}

pub fn on_dropped_to_world_base(w: &mut World, id: ItemId, from: MobileId, _p: Point3D) -> bool {
    let nontransferable = ty_of(w, id).is_some_and(|t| (t.nontransferable)(w, id));
    if nontransferable && w.mobile(from).is_some_and(|m| m.player) {
        handle_invalid_transfer(w, id, from);
        return false;
    }
    true
}

/// `x.drop_to_world(..)`: the virtual call (an override runs; `drop_to_world_base` is what `base.drop_to_world(..)` reaches).
pub fn drop_to_world(w: &mut World, id: ItemId, a2: MobileId, a3: Point3D) -> bool {
    match w.item(id).map(|m| m.ty) {
        Some(ty) => (ty.drop_to_world)(w, id, a2, a3),
        None => drop_to_world_base(w, id, a2, a3),
    }
}

pub fn drop_to_world_base(w: &mut World, id: ItemId, from: MobileId, p: Point3D) -> bool {
    let Some(ty) = ty_of(w, id) else { return false };
    if ops::is_deleted(w, id) || deps::mobile_deleted(w, from) {
        return false;
    }
    let Some(map) = w.mobile(from).and_then(|m| m.map) else {
        return false;
    };
    if !deps::mobile_in_range(w, from, p, 2) {
        return false;
    }

    let (x, y) = (p.x, p.y);
    let mut z = i32::MIN;

    let max_z = w.mobile(from).map_or(0, |m| m.location.z) + 16;

    let land_tile = w.maps.land_tile(map, x, y);
    let land_flags = w
        .maps
        .tile_data
        .land((land_tile.id as usize) & muo_data::TileData::MAX_LAND_VALUE)
        .flags;

    let (land_z, land_avg, _) = w.maps.get_average_z_full(map, x, y);

    let ignored = land_tile.ignored();
    if !ignored
        && !land_flags.contains(muo_data::tile_data::TileFlag::IMPASSABLE)
        && land_avg <= max_z
    {
        z = land_avg;
    }

    let statics = crate::map::tiles::static_and_multi_tiles(w, map, x, y);
    for tile in &statics {
        let data = w
            .maps
            .tile_data
            .item((tile.id as usize) & muo_data::TileData::MAX_ITEM_VALUE);
        if !data.surface() {
            continue;
        }
        let top = i32::from(tile.z) + data.calc_height();
        if top > max_z || top < z {
            continue;
        }
        z = top;
    }

    let mut items = Vec::new();
    for item in deps::items_at(w, map, p) {
        let Some(it) = w.item(item) else { continue };
        if it.ty.is(&BASE_MULTI) || it.item_id > muo_data::TileData::MAX_ITEM_VALUE as i32 {
            continue;
        }
        let data = ops::item_data(w, item);
        if data.surface() {
            let top = it.location.z + data.calc_height();
            if top <= max_z && top >= z {
                z = top;
            }
        }
        items.push(item);
    }

    if z == i32::MIN || z > max_z {
        return false;
    }

    let mut open_slots: i32 = (1 << 20) - 1;
    let surface_z = z;

    let mut mark = |check_z: i32, check_top: i32, surface: bool| {
        let mut check_top = check_top;
        if check_top == check_z && !surface {
            check_top += 1;
        }
        let z_start = (check_z - z).max(0);
        let z_end = (check_top - z).min(19);
        if z_start >= 20 || z_end < 0 {
            return;
        }
        let bit_count = z_end - z_start;
        open_slots &= !(((1 << bit_count) - 1) << z_start);
    };

    for tile in &statics {
        let data = w
            .maps
            .tile_data
            .item((tile.id as usize) & muo_data::TileData::MAX_ITEM_VALUE);
        let check_z = i32::from(tile.z);
        mark(check_z, check_z + data.calc_height(), data.surface());
    }
    for &item in &items {
        let data = ops::item_data(w, item);
        let check_z = w.item(item).map_or(0, |i| i.location.z);
        mark(check_z, check_z + data.calc_height(), data.surface());
    }
    ops::statics(w).open_slots = open_slots;

    let own = ops::item_data(w, id);
    let mut height = i32::from(own.height);
    if height == 0 {
        height += 1;
    }
    if height > 30 {
        height = 30;
    }

    let mut match_bits = (1 << height) - 1;
    let mut okay = false;

    for i in 0..20 {
        if i + height > 20 {
            match_bits >>= 1;
        }
        okay = ((open_slots >> i) & match_bits) == match_bits;
        if okay {
            z += i;
            break;
        }
    }

    if !okay {
        return false;
    }

    let mut height = i32::from(own.height);
    if height == 0 {
        height += 1;
    }

    if land_avg > z && z + height > land_z {
        return false;
    }

    if land_flags.contains(muo_data::tile_data::TileFlag::IMPASSABLE)
        && land_avg > surface_z
        && z + height > land_z
    {
        return false;
    }

    for tile in &statics {
        let data = w
            .maps
            .tile_data
            .item((tile.id as usize) & muo_data::TileData::MAX_ITEM_VALUE);
        let check_z = i32::from(tile.z);
        let check_top = check_z + data.calc_height();

        if check_top > z && z + height > check_z {
            return false;
        }
        if (data.surface() || data.impassable()) && check_top > surface_z && z + height > check_z {
            return false;
        }
    }

    for &item in &items {
        let data = ops::item_data(w, item);
        let iz = w.item(item).map_or(0, |i| i.location.z);
        if iz + data.calc_height() > z && z + height > iz {
            return false;
        }
    }

    let p = Point3D::new(x, y, z);

    if !deps::in_los_point(w, from, Point3D::new(x, y, z + 1)) {
        return false;
    }

    let Some(fty) = w.mobile(from).map(|m| m.ty) else {
        return false;
    };
    if !(fty.on_dropped_item_to_world)(w, from, id, p) {
        return false;
    }

    if !(ty.on_dropped_to_world)(w, id, from, p) {
        return false;
    }

    let sound_id = (ty.get_drop_sound)(w, id);

    let from_map = w.mobile(from).and_then(|m| m.map);
    ops::move_to_world_map(w, id, p, from_map);

    let loc = ops::get_world_location(w, id);
    deps::send_sound(w, from, if sound_id == -1 { 0x42 } else { sound_id }, loc);

    true
}

/// `x.on_item_added(..)`: the virtual call (an override runs; `on_item_added_base` is what `base.on_item_added(..)` reaches).
pub fn on_item_added(w: &mut World, id: ItemId, a2: ItemId) {
    match w.item(id).map(|m| m.ty) {
        Some(ty) => (ty.on_item_added)(w, id, a2),
        None => on_item_added_base(w, id, a2),
    }
}

pub fn on_item_added_base(w: &mut World, id: ItemId, item: ItemId) {
    match parent_of(w, id) {
        Some(EntityId::Item(p)) => {
            if let Some(t) = ty_of(w, p) {
                (t.on_sub_item_added)(w, p, item);
            }
        }
        Some(EntityId::Mobile(m)) => {
            if let Some(t) = w.mobile(m).map(|m| m.ty) {
                (t.on_sub_item_added)(w, m, item);
            }
        }
        None => {}
    }
}

/// `x.on_item_removed(..)`: the virtual call (an override runs; `on_item_removed_base` is what `base.on_item_removed(..)` reaches).
pub fn on_item_removed(w: &mut World, id: ItemId, a2: ItemId) {
    match w.item(id).map(|m| m.ty) {
        Some(ty) => (ty.on_item_removed)(w, id, a2),
        None => on_item_removed_base(w, id, a2),
    }
}

pub fn on_item_removed_base(w: &mut World, id: ItemId, item: ItemId) {
    match parent_of(w, id) {
        Some(EntityId::Item(p)) => {
            if let Some(t) = ty_of(w, p) {
                (t.on_sub_item_removed)(w, p, item);
            }
        }
        Some(EntityId::Mobile(m)) => {
            if let Some(t) = w.mobile(m).map(|m| m.ty) {
                (t.on_sub_item_removed)(w, m, item);
            }
        }
        None => {}
    }
}

/// `x.on_sub_item_added(..)`: the virtual call (an override runs; `on_sub_item_added_base` is what `base.on_sub_item_added(..)` reaches).
pub fn on_sub_item_added(w: &mut World, id: ItemId, a2: ItemId) {
    match w.item(id).map(|m| m.ty) {
        Some(ty) => (ty.on_sub_item_added)(w, id, a2),
        None => on_sub_item_added_base(w, id, a2),
    }
}

pub fn on_sub_item_added_base(w: &mut World, id: ItemId, item: ItemId) {
    on_item_added(w, id, item);
}

/// `x.on_sub_item_removed(..)`: the virtual call (an override runs; `on_sub_item_removed_base` is what `base.on_sub_item_removed(..)` reaches).
pub fn on_sub_item_removed(w: &mut World, id: ItemId, a2: ItemId) {
    match w.item(id).map(|m| m.ty) {
        Some(ty) => (ty.on_sub_item_removed)(w, id, a2),
        None => on_sub_item_removed_base(w, id, a2),
    }
}

pub fn on_sub_item_removed_base(w: &mut World, id: ItemId, item: ItemId) {
    on_item_removed(w, id, item);
}

/// `x.on_item_bounce_cleared(..)`: the virtual call (an override runs; `on_item_bounce_cleared_base` is what `base.on_item_bounce_cleared(..)` reaches).
pub fn on_item_bounce_cleared(w: &mut World, id: ItemId, a2: ItemId) {
    match w.item(id).map(|m| m.ty) {
        Some(ty) => (ty.on_item_bounce_cleared)(w, id, a2),
        None => on_item_bounce_cleared_base(w, id, a2),
    }
}

pub fn on_item_bounce_cleared_base(w: &mut World, id: ItemId, item: ItemId) {
    match parent_of(w, id) {
        Some(EntityId::Item(p)) => {
            if let Some(t) = ty_of(w, p) {
                (t.on_sub_item_bounce_cleared)(w, p, item);
            }
        }
        Some(EntityId::Mobile(m)) => {
            if let Some(t) = w.mobile(m).map(|m| m.ty) {
                (t.on_sub_item_bounce_cleared)(w, m, item);
            }
        }
        None => {}
    }
}

pub fn on_sub_item_bounce_cleared(w: &mut World, id: ItemId, item: ItemId) {
    on_item_bounce_cleared(w, id, item);
}

/// `x.check_target(..)`: the virtual call (an override runs; `check_target_base` is what `base.check_target(..)` reaches).
pub fn check_target(w: &mut World, id: ItemId, a2: MobileId, a3: TargetId, a4: Targeted) -> bool {
    match w.item(id).map(|m| m.ty) {
        Some(ty) => (ty.check_target)(w, id, a2, a3, a4),
        None => check_target_base(w, id, a2, a3, a4),
    }
}

pub fn check_target_base(
    w: &mut World,
    id: ItemId,
    from: MobileId,
    targ: TargetId,
    targeted: Targeted,
) -> bool {
    match parent_of(w, id) {
        Some(EntityId::Item(p)) => match ty_of(w, p) {
            Some(t) => (t.check_target)(w, p, from, targ, targeted),
            None => true,
        },
        Some(EntityId::Mobile(m)) => match w.mobile(m).map(|m| m.ty) {
            Some(t) => (t.check_target)(w, m, from, targ, targeted),
            None => true,
        },
        None => true,
    }
}

/// `x.is_accessible_to(..)`: the virtual call (an override runs; `is_accessible_to_base` is what `base.is_accessible_to(..)` reaches).
pub fn is_accessible_to(w: &mut World, id: ItemId, a2: MobileId) -> bool {
    match w.item(id).map(|m| m.ty) {
        Some(ty) => (ty.is_accessible_to)(w, id, a2),
        None => is_accessible_to_base(w, id, a2),
    }
}

pub fn is_accessible_to_base(w: &mut World, id: ItemId, check: MobileId) -> bool {
    if let Some(EntityId::Item(p)) = parent_of(w, id) {
        return match ty_of(w, p) {
            Some(t) => (t.is_accessible_to)(w, p, check),
            None => true,
        };
    }
    deps::region_check_accessibility(w, id, check)
}

/// `x.on_item_used(..)`: the virtual call (an override runs; `on_item_used_base` is what `base.on_item_used(..)` reaches).
pub fn on_item_used(w: &mut World, id: ItemId, a2: MobileId, a3: ItemId) {
    match w.item(id).map(|m| m.ty) {
        Some(ty) => (ty.on_item_used)(w, id, a2, a3),
        None => on_item_used_base(w, id, a2, a3),
    }
}

pub fn on_item_used_base(w: &mut World, id: ItemId, from: MobileId, item: ItemId) {
    match parent_of(w, id) {
        Some(EntityId::Item(p)) => {
            if let Some(t) = ty_of(w, p) {
                (t.on_item_used)(w, p, from, item);
            }
        }
        Some(EntityId::Mobile(m)) => {
            if let Some(t) = w.mobile(m).map(|m| m.ty) {
                (t.on_item_used)(w, m, from, item);
            }
        }
        None => {}
    }
}

/// `x.check_item_use(..)`: the virtual call (an override runs; `check_item_use_base` is what `base.check_item_use(..)` reaches).
pub fn check_item_use(w: &mut World, id: ItemId, a2: MobileId, a3: ItemId) -> bool {
    match w.item(id).map(|m| m.ty) {
        Some(ty) => (ty.check_item_use)(w, id, a2, a3),
        None => check_item_use_base(w, id, a2, a3),
    }
}

pub fn check_item_use_base(w: &mut World, id: ItemId, from: MobileId, item: ItemId) -> bool {
    match parent_of(w, id) {
        Some(EntityId::Item(p)) => match ty_of(w, p) {
            Some(t) => (t.check_item_use)(w, p, from, item),
            None => true,
        },
        Some(EntityId::Mobile(m)) => match w.mobile(m).map(|m| m.ty) {
            Some(t) => (t.check_item_use)(w, m, from, item),
            None => true,
        },
        None => true,
    }
}

/// `x.on_item_lifted(..)`: the virtual call (an override runs; `on_item_lifted_base` is what `base.on_item_lifted(..)` reaches).
pub fn on_item_lifted(w: &mut World, id: ItemId, a2: MobileId, a3: ItemId) {
    match w.item(id).map(|m| m.ty) {
        Some(ty) => (ty.on_item_lifted)(w, id, a2, a3),
        None => on_item_lifted_base(w, id, a2, a3),
    }
}

pub fn on_item_lifted_base(w: &mut World, id: ItemId, from: MobileId, item: ItemId) {
    match parent_of(w, id) {
        Some(EntityId::Item(p)) => {
            if let Some(t) = ty_of(w, p) {
                (t.on_item_lifted)(w, p, from, item);
            }
        }
        Some(EntityId::Mobile(m)) => {
            if let Some(t) = w.mobile(m).map(|m| m.ty) {
                (t.on_item_lifted)(w, m, from, item);
            }
        }
        None => {}
    }
}

/// `x.check_lift(..)`: the virtual call (an override runs; `check_lift_base` is what `base.check_lift(..)` reaches).
pub fn check_lift(w: &mut World, id: ItemId, a2: MobileId, a3: ItemId, a4: &mut LRReason) -> bool {
    match w.item(id).map(|m| m.ty) {
        Some(ty) => (ty.check_lift)(w, id, a2, a3, a4),
        None => check_lift_base(w, id, a2, a3, a4),
    }
}

pub fn check_lift_base(
    w: &mut World,
    id: ItemId,
    from: MobileId,
    item: ItemId,
    reject: &mut LRReason,
) -> bool {
    match parent_of(w, id) {
        Some(EntityId::Item(p)) => match ty_of(w, p) {
            Some(t) => (t.check_lift)(w, p, from, item, reject),
            None => true,
        },
        Some(EntityId::Mobile(m)) => match w.mobile(m).map(|m| m.ty) {
            Some(t) => (t.check_lift)(w, m, from, item, reject),
            None => true,
        },
        None => true,
    }
}

/// `x.on_single_click_contained(..)`: the virtual call (an override runs; `on_single_click_contained_base` is what `base.on_single_click_contained(..)` reaches).
pub fn on_single_click_contained(w: &mut World, id: ItemId, a2: MobileId, a3: ItemId) {
    match w.item(id).map(|m| m.ty) {
        Some(ty) => (ty.on_single_click_contained)(w, id, a2, a3),
        None => on_single_click_contained_base(w, id, a2, a3),
    }
}

pub fn on_single_click_contained_base(w: &mut World, id: ItemId, from: MobileId, item: ItemId) {
    if let Some(EntityId::Item(p)) = parent_of(w, id)
        && let Some(t) = ty_of(w, p)
    {
        (t.on_single_click_contained)(w, p, from, item);
    }
}

/// `x.on_aos_single_click(..)`: the virtual call (an override runs; `on_aos_single_click_base` is what `base.on_aos_single_click(..)` reaches).
pub fn on_aos_single_click(w: &mut World, id: ItemId, a2: MobileId) {
    match w.item(id).map(|m| m.ty) {
        Some(ty) => (ty.on_aos_single_click)(w, id, a2),
        None => on_aos_single_click_base(w, id, a2),
    }
}

pub fn on_aos_single_click_base(w: &mut World, id: ItemId, from: MobileId) {
    let (header, args) = match ops::property_list(w, id) {
        Some(l) => (l.header, l.header_args.clone().unwrap_or_default()),
        None => return,
    };
    if header > 0
        && let Some(ns) = deps::net_state_of(w, from)
    {
        let name = ops::name(w, id).unwrap_or_default();
        deps::label_localized(w, ns, id, header, &name, &args);
    }
}

/// `x.on_single_click(..)`: the virtual call (an override runs; `on_single_click_base` is what `base.on_single_click(..)` reaches).
pub fn on_single_click(w: &mut World, id: ItemId, a2: MobileId) {
    match w.item(id).map(|m| m.ty) {
        Some(ty) => (ty.on_single_click)(w, id, a2),
        None => on_single_click_base(w, id, a2),
    }
}

pub fn on_single_click_base(w: &mut World, id: ItemId, from: MobileId) {
    let Some(i) = w.item(id) else { return };
    let deleted = i.deleted();
    if deleted || !deps::can_see(w, from, id) {
        return;
    }
    let Some(i) = w.item(id) else { return };
    let (ty, amount) = (i.ty, i.amount);

    if (ty.display_loot_type)(w, id) {
        (ty.label_loot_type_to)(w, id, from);
    }

    let Some(ns) = deps::net_state_of(w, from) else {
        return;
    };
    if w.cannot_send_packets(ns) {
        return;
    }

    let name = ops::name(w, id);
    let label = (ty.label_number)(w, id);
    match name {
        None => {
            if amount <= 1 {
                deps::label_localized(w, ns, id, label, "", "");
            } else {
                deps::label_localized_affix(
                    w,
                    ns,
                    id,
                    label,
                    crate::packets::message::AffixType::APPEND,
                    &format!(" : {amount}"),
                    "",
                );
            }
        }
        Some(name) => {
            if amount > 1 {
                deps::label_text(w, ns, id, &format!("{name} : {amount}"));
            } else {
                deps::label_text(w, ns, id, &name);
            }
        }
    }
}

pub fn scissor_helper(
    w: &mut World,
    id: ItemId,
    from: MobileId,
    new_item: ItemId,
    amount_per_old_item: i32,
    carry_hue: bool,
) {
    let Some(i) = w.item(id) else { return };
    // let's not go over 60000
    let amount = i.amount.min(60000 / amount_per_old_item);

    let our_hue = (i.ty.hue)(w, id);
    let this_map = i.map;
    let this_parent = i.parent;
    let world_loc = ops::get_world_location(w, id);
    let loot = i.loot_type;
    let ty = i.ty;

    (ty.consume)(w, id, amount);

    ops::set_amount(w, new_item, amount * amount_per_old_item);

    if let Some(nt) = ty_of(w, new_item) {
        if carry_hue {
            (nt.set_hue)(w, new_item, our_hue);
        }
        if w.resources
            .get::<ops::ItemStatics>()
            .is_some_and(|s| s.scissor_copy_loot_type)
        {
            ops::set_loot_type(w, new_item, loot);
        }
    }

    let dropped = match this_parent {
        Some(EntityId::Item(p)) if ty_of(w, p).is_some_and(|t| t.is(&CONTAINER)) => {
            crate::item::container::try_drop_item(w, p, Some(from), new_item, false)
        }
        _ => false,
    };
    if !dropped {
        ops::move_to_world_map(w, new_item, world_loc, this_map);
    }
}

pub fn consume(w: &mut World, id: ItemId, amount: i32) {
    let Some(i) = w.item(id) else { return };
    let (cur, ty) = (i.amount, i.ty);
    if cur <= amount {
        (ty.delete)(w, id);
        return;
    }
    ops::set_amount(w, id, cur - amount);
}

pub fn replace_with(w: &mut World, id: ItemId, new_item: ItemId) {
    let Some(i) = w.item(id) else { return };
    let (loc, map, ty) = (i.location, i.map, i.ty);
    match i.parent {
        Some(EntityId::Item(p)) if ty_of(w, p).is_some_and(|t| t.is(&CONTAINER)) => {
            if let Some(pt) = ty_of(w, p) {
                (pt.add_item)(w, p, new_item);
            }
            ops::set_location(w, new_item, loc);
        }
        _ => {
            let world_loc = ops::get_world_location(w, id);
            ops::move_to_world_map(w, new_item, world_loc, map);
        }
    }
    (ty.delete)(w, id);
}

pub fn check_blessed(w: &World, id: ItemId, m: MobileId) -> bool {
    let Some(i) = w.item(id) else { return false };
    i.loot_type == LootType::Blessed
        || (deps::insurance_enabled(w) && i.insured())
        || i.blessed_for == Some(m)
}

pub fn is_standard_loot(w: &World, id: ItemId) -> bool {
    let Some(i) = w.item(id) else { return false };
    (!deps::insurance_enabled(w) || !i.insured())
        && i.blessed_for.is_none()
        && i.loot_type == LootType::Regular
}

/// `x.on_double_click_dead(..)`: the virtual call (an override runs; `on_double_click_dead_base` is what `base.on_double_click_dead(..)` reaches).
pub fn on_double_click_dead(w: &mut World, id: ItemId, a2: MobileId) {
    match w.item(id).map(|m| m.ty) {
        Some(ty) => (ty.on_double_click_dead)(w, id, a2),
        None => on_double_click_dead_base(w, id, a2),
    }
}

pub fn on_double_click_dead_base(w: &mut World, _id: ItemId, from: MobileId) {
    deps::local_overhead_message(w, from, MessageType::REGULAR, 0x3B2, 1019048); // I am dead and cannot do that.
}

/// `x.on_double_click_not_accessible(..)`: the virtual call (an override runs; `on_double_click_not_accessible_base` is what `base.on_double_click_not_accessible(..)` reaches).
pub fn on_double_click_not_accessible(w: &mut World, id: ItemId, a2: MobileId) {
    match w.item(id).map(|m| m.ty) {
        Some(ty) => (ty.on_double_click_not_accessible)(w, id, a2),
        None => on_double_click_not_accessible_base(w, id, a2),
    }
}

pub fn on_double_click_not_accessible_base(w: &mut World, _id: ItemId, from: MobileId) {
    deps::send_localized_message(w, from, 500447); // That is not accessible.
}

/// `x.on_double_click_secure_trade(..)`: the virtual call (an override runs; `on_double_click_secure_trade_base` is what `base.on_double_click_secure_trade(..)` reaches).
pub fn on_double_click_secure_trade(w: &mut World, id: ItemId, a2: MobileId) {
    match w.item(id).map(|m| m.ty) {
        Some(ty) => (ty.on_double_click_secure_trade)(w, id, a2),
        None => on_double_click_secure_trade_base(w, id, a2),
    }
}

pub fn on_double_click_secure_trade_base(w: &mut World, _id: ItemId, from: MobileId) {
    deps::send_localized_message(w, from, 500447); // That is not accessible.
}
