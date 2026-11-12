//! The statements a weapon constructor runs after `base(..)`: property setters on the item, called on the item
//! under construction (so a virtual setter runs the class's own override).

use muo_content::aos::{AttrKind, Attrs};
use muo_content::items::slayer::SlayerName;
use muo_content::items::{set_attr, weapon};
use muo_server::World;
use muo_server::ids::ItemId;
use muo_server::item::ops;
use muo_server::layer::Layer;
use muo_server::skills::SkillName;

/// `Hue = value`.
pub fn set_hue(w: &mut World, id: ItemId, value: i32) {
    if let Some(ty) = w.item(id).map(|i| i.ty) {
        (ty.set_hue)(w, id, value);
    }
}

/// `ItemID = value`.
pub fn set_item_id(w: &mut World, id: ItemId, value: i32) {
    if let Some(ty) = w.item(id).map(|i| i.ty) {
        (ty.set_item_id)(w, id, value);
    }
}

/// `Layer = value`.
pub fn set_layer(w: &mut World, id: ItemId, value: Layer) {
    if let Some(ty) = w.item(id).map(|i| i.ty) {
        (ty.set_layer)(w, id, value);
    }
}

/// `Slayer = value` (a `[SerializableField] [InvalidateProperties]`).
pub fn set_slayer(w: &mut World, id: ItemId, value: SlayerName) {
    if let Some(d) = weapon::data_mut(w, id)
        && d.slayer != value as i32
    {
        d.slayer = value as i32;
        ops::invalidate_properties(w, id);
    }
}

/// `Slayer2 = (SlayerName)value`.
pub fn set_slayer2(w: &mut World, id: ItemId, value: i32) {
    if let Some(d) = weapon::data_mut(w, id)
        && d.slayer2 != value
    {
        d.slayer2 = value;
        ops::invalidate_properties(w, id);
    }
}

/// `Balanced = value` (BaseRanged).
pub fn set_balanced(w: &mut World, id: ItemId, value: bool) {
    if let Some(d) = weapon::data_mut(w, id)
        && d.balanced != value
    {
        d.balanced = value;
        ops::invalidate_properties(w, id);
    }
}

/// `Velocity = value` (BaseRanged).
pub fn set_velocity(w: &mut World, id: ItemId, value: i32) {
    if let Some(d) = weapon::data_mut(w, id)
        && d.velocity != value
    {
        d.velocity = value;
        ops::invalidate_properties(w, id);
    }
}

/// `UsesRemaining = value` (BaseAxe, BasePoleArm).
pub fn set_uses_remaining(w: &mut World, id: ItemId, value: i32) {
    if let Some(d) = weapon::data_mut(w, id)
        && d.uses_remaining != value
    {
        d.uses_remaining = value;
        ops::invalidate_properties(w, id);
    }
}

/// `ShowUsesRemaining = value` (BaseAxe, BasePoleArm).
pub fn set_show_uses_remaining(w: &mut World, id: ItemId, value: bool) {
    if let Some(d) = weapon::data_mut(w, id)
        && d.show_uses_remaining != value
    {
        d.show_uses_remaining = value;
        ops::invalidate_properties(w, id);
    }
}

/// `SkillBonuses.SetValues(index, skill, bonus)`.
pub fn skill_bonus(w: &mut World, id: ItemId, index: u32, skill: SkillName, bonus: f64) {
    set_attr(
        w,
        id,
        AttrKind::SkillBonuses,
        1 << index,
        Attrs::skill_bonus_value(skill, bonus),
    );
}
