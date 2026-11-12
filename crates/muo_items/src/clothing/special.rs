//! The clothing classes with logic or state of their own:
//! - `Cloak`, `Robe`, `ThighBoots`: `IArcaneEquip` is two fields of
//!   `ClothingData` (`arcane_charges`) and the graphics each class swaps between ([`arcane_ids`]); the bare
//!   `[Flippable]` is `ClothingClass::flip`.
//! - `RewardCloak`, `RewardRobe`, `RewardDress`: `IRewardItem` (`ClothingData::{number, is_reward_item}`); the
//!   veteran reward system is behind [`RewardHooks`].
//! - `DeathRobe` (a `Robe` that decays), `OrcishKinMask`, `SavageMask`.
//! - the constructors of the samples `Cap` and `FancyShirt` of `muo_content::items::leaves`, for derived classes.

use muo_content::items::clothing::{
    self, BASE_CLOAK, BASE_CLOAK_CLASS, BASE_HAT, BASE_HAT_CLASS, BASE_OUTER_TORSO,
    BASE_OUTER_TORSO_CLASS, BASE_SHOES, BASE_SHOES_CLASS, ClothingClass, new_clothing,
};
use muo_content::items::parent_mobile;
use muo_io::{GenericReader, GenericWriter, Result};
use muo_primitives::{DateTime, TimeSpan, utility};
use muo_server::World;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::{ItemType, LootType, deps, ops};
use muo_server::property_list::PropertyList;
use muo_server::timer::{self, TimerHandle};
use muo_server::type_registry::TypeRegistry;

use super::dye_refused;
use crate::armor::hue;

// ---- the samples ----

// ---- veteran rewards ----

/// What `IRewardItem`'s `RewardSystem` (Engines/VeteranRewards) answers, set by the owning crate. Unset: no year
/// label, and every reward item is usable by everyone (as when no reward entry matches the item).
#[derive(Default)]
pub struct RewardHooks {
    /// `RewardSystem.GetRewardYearLabel(item, new object[] { Hue, _number })`: the cliloc to add.
    pub year_label: Option<fn(&World, ItemId) -> Option<i32>>,
    /// `RewardSystem.CheckIsUsableBy(m, item, new object[] { Hue, _number })`.
    pub check_is_usable_by: Option<fn(&mut World, MobileId, ItemId) -> bool>,
    /// `RewardSystem.GetRewardLevel(m)`. Unset: 0.
    pub reward_level: Option<fn(&World, MobileId) -> i32>,
}

fn reward_year_label(w: &World, id: ItemId) -> Option<i32> {
    w.resources
        .get::<RewardHooks>()
        .and_then(|h| h.year_label)
        .and_then(|f| f(w, id))
}

fn reward_usable_by(w: &mut World, m: MobileId, id: ItemId) -> bool {
    match w
        .resources
        .get::<RewardHooks>()
        .and_then(|h| h.check_is_usable_by)
    {
        Some(f) => f(w, m, id),
        None => true,
    }
}

fn reward_number(w: &World, id: ItemId) -> i32 {
    clothing::data(w, id).map_or(0, |d| d.number)
}

fn is_reward_item(w: &World, id: ItemId) -> bool {
    clothing::data(w, id).is_some_and(|d| d.is_reward_item)
}

/// `mobile.VirtualArmorMod += delta` when the parent is a mobile.
fn add_virtual_armor_mod(w: &mut World, parent: Option<MobileId>, delta: i32) {
    if let Some(m) = parent
        && let Some(v) = w.mobile(m).map(|m| m.virtual_armor_mod)
    {
        muo_server::mobile::part1::props_a::set_virtual_armor_mod(w, m, v + delta);
    }
}

fn reward_on_added(
    w: &mut World,
    id: ItemId,
    parent: EntityId,
    base: fn(&mut World, ItemId, EntityId),
) {
    base(w, id, parent);
    if let EntityId::Mobile(m) = parent {
        add_virtual_armor_mod(w, Some(m), 2);
    }
}

fn reward_on_removed(
    w: &mut World,
    id: ItemId,
    parent: EntityId,
    base: fn(&mut World, ItemId, EntityId),
) {
    base(w, id, parent);
    if let EntityId::Mobile(m) = parent {
        add_virtual_armor_mod(w, Some(m), -2);
    }
}

/// `LabelNumber => _number > 0 ? _number : base.LabelNumber`.
fn reward_label(w: &World, id: ItemId, base: fn(&World, ItemId) -> i32) -> i32 {
    match reward_number(w, id) {
        n if n > 0 => n,
        _ => base(w, id),
    }
}

/// Writes `_number` and `_isRewardItem` (fields 0 and 1 of the three reward classes).
fn reward_write(w: &World, id: ItemId, out: &mut dyn GenericWriter) {
    out.write_i32(0);
    out.write_i32(reward_number(w, id));
    out.write_bool(is_reward_item(w, id));
}

/// Reads what [`reward_write`] wrote and runs `[AfterDeserialization]`.
fn reward_read(w: &mut World, id: ItemId, r: &mut dyn GenericReader) -> Result<()> {
    let _version = r.read_i32()?;
    let number = r.read_i32()?;
    let is_reward = r.read_bool()?;
    if let Some(d) = clothing::data_mut(w, id) {
        d.number = number;
        d.is_reward_item = is_reward;
    }
    add_virtual_armor_mod(w, parent_mobile(w, id), 2);
    Ok(())
}

/// The shared tail of the three reward constructors: `LootType = Blessed; _number = labelNumber`.
fn reward_init(w: &mut World, id: ItemId, label_number: i32) {
    ops::set_loot_type(w, id, LootType::Blessed);
    if let Some(d) = clothing::data_mut(w, id) {
        d.number = label_number;
    }
}

fn reward_get_properties(
    w: &World,
    id: ItemId,
    list: &mut PropertyList,
    base: fn(&World, ItemId, &mut PropertyList),
    needs_ml: bool,
) {
    base(w, id, list);
    if (!needs_ml || muo_content::core::ml(w))
        && is_reward_item(w, id)
        && let Some(n) = reward_year_label(w, id)
    {
        list.add(n); // X Year Veteran Reward
    }
}

fn reward_can_equip(
    w: &mut World,
    id: ItemId,
    m: MobileId,
    base: fn(&mut World, ItemId, MobileId) -> bool,
) -> bool {
    base(w, id, m) && (!is_reward_item(w, id) || reward_usable_by(w, m, id))
}

// ---- RewardCloak ----

/// `RewardCloak(hue, labelNumber)`: `base(0x1515, hue)`.
pub fn reward_cloak_new(
    w: &mut World,
    ty: &'static ItemType,
    hue: i32,
    label_number: i32,
) -> ItemId {
    let id = new_clothing(w, ty, 0x1515, hue);
    reward_init(w, id, label_number);
    id
}

/// `RewardCloak`' constants.
pub static REWARD_CLOAK_CLASS: ClothingClass = ClothingClass {
    default_weight: Some(5.0),
    base_physical_resistance: 3,
    dye: dye_refused,
    ..BASE_CLOAK_CLASS
};

/// `Server.Items.RewardCloak` (`[SerializationGenerator(0, false)]`; the bare `[Flippable]` has no `Flip()`).
pub static REWARD_CLOAK: ItemType = ItemType {
    full_name: "Server.Items.RewardCloak",
    parent: Some(&BASE_CLOAK),
    construct: Some(|w| reward_cloak_new(w, &REWARD_CLOAK, 0, 0)),
    family: Some(&REWARD_CLOAK_CLASS),
    label_number: |w, id| reward_label(w, id, BASE_CLOAK.label_number),
    on_added: |w, id, p| reward_on_added(w, id, p, BASE_CLOAK.on_added),
    on_removed: |w, id, p| reward_on_removed(w, id, p, BASE_CLOAK.on_removed),
    get_properties: |w, id, l| reward_get_properties(w, id, l, BASE_CLOAK.get_properties, true),
    can_equip: |w, id, m| reward_can_equip(w, id, m, BASE_CLOAK.can_equip),
    serialize: |w, id, out| {
        (BASE_CLOAK.serialize)(w, id, out);
        reward_write(w, id, out);
    },
    deserialize: |w, id, r| {
        (BASE_CLOAK.deserialize)(w, id, r)?;
        reward_read(w, id, r)
    },
    ..BASE_CLOAK
};

// ---- RewardRobe, RewardDress ----

/// `RewardRobe(hue, labelNumber)`: `base(0x1F03, hue)`.
pub fn reward_robe_new(
    w: &mut World,
    ty: &'static ItemType,
    hue: i32,
    label_number: i32,
) -> ItemId {
    let id = new_clothing(w, ty, 0x1F03, hue);
    reward_init(w, id, label_number);
    id
}

/// `RewardRobe`' constants.
pub static REWARD_ROBE_CLASS: ClothingClass = ClothingClass {
    default_weight: Some(3.0),
    base_physical_resistance: 3,
    dye: dye_refused,
    ..BASE_OUTER_TORSO_CLASS
};

/// `Server.Items.RewardRobe`.
pub static REWARD_ROBE: ItemType = ItemType {
    full_name: "Server.Items.RewardRobe",
    parent: Some(&BASE_OUTER_TORSO),
    construct: Some(|w| reward_robe_new(w, &REWARD_ROBE, 0, 0)),
    family: Some(&REWARD_ROBE_CLASS),
    label_number: |w, id| reward_label(w, id, BASE_OUTER_TORSO.label_number),
    on_added: |w, id, p| reward_on_added(w, id, p, BASE_OUTER_TORSO.on_added),
    on_removed: |w, id, p| reward_on_removed(w, id, p, BASE_OUTER_TORSO.on_removed),
    get_properties: |w, id, l| {
        reward_get_properties(w, id, l, BASE_OUTER_TORSO.get_properties, true)
    },
    can_equip: |w, id, m| reward_can_equip(w, id, m, BASE_OUTER_TORSO.can_equip),
    serialize: |w, id, out| {
        (BASE_OUTER_TORSO.serialize)(w, id, out);
        reward_write(w, id, out);
    },
    deserialize: |w, id, r| {
        (BASE_OUTER_TORSO.deserialize)(w, id, r)?;
        reward_read(w, id, r)
    },
    ..BASE_OUTER_TORSO
};

/// `RewardDress(hue, labelNumber)`: `base(0x1F01, hue)`.
pub fn reward_dress_new(
    w: &mut World,
    ty: &'static ItemType,
    hue: i32,
    label_number: i32,
) -> ItemId {
    let id = new_clothing(w, ty, 0x1F01, hue);
    reward_init(w, id, label_number);
    id
}

/// `RewardDress`' constants.
pub static REWARD_DRESS_CLASS: ClothingClass = ClothingClass {
    default_weight: Some(2.0),
    base_physical_resistance: 3,
    dye: dye_refused,
    ..BASE_OUTER_TORSO_CLASS
};

/// `Server.Items.RewardDress` (its `GetProperties` has no `Core.ML` test).
pub static REWARD_DRESS: ItemType = ItemType {
    full_name: "Server.Items.RewardDress",
    parent: Some(&BASE_OUTER_TORSO),
    construct: Some(|w| reward_dress_new(w, &REWARD_DRESS, 0, 0)),
    family: Some(&REWARD_DRESS_CLASS),
    label_number: |w, id| reward_label(w, id, BASE_OUTER_TORSO.label_number),
    on_added: |w, id, p| reward_on_added(w, id, p, BASE_OUTER_TORSO.on_added),
    on_removed: |w, id, p| reward_on_removed(w, id, p, BASE_OUTER_TORSO.on_removed),
    get_properties: |w, id, l| {
        reward_get_properties(w, id, l, BASE_OUTER_TORSO.get_properties, false)
    },
    can_equip: |w, id, m| reward_can_equip(w, id, m, BASE_OUTER_TORSO.can_equip),
    serialize: |w, id, out| {
        (BASE_OUTER_TORSO.serialize)(w, id, out);
        reward_write(w, id, out);
    },
    deserialize: |w, id, r| {
        (BASE_OUTER_TORSO.deserialize)(w, id, r)?;
        reward_read(w, id, r)
    },
    ..BASE_OUTER_TORSO
};

// ---- IArcaneEquip ----

/// The graphics an arcane class swaps between: `(arcane, normal)` where `normal` is what `Update()` restores.
/// `None` when the class is not an `IArcaneEquip`.
pub fn arcane_ids(ty: &'static ItemType) -> Option<(i32, i32)> {
    if ty.is(&ROBE) {
        Some((0x26AE, 0x1F04))
    } else if ty.is(&CLOAK) {
        Some((0x26AD, 0x1515))
    } else if ty.is(&THIGH_BOOTS) {
        Some((0x26AF, 0x1711))
    } else {
        None
    }
}

/// `item as IArcaneEquip`: `(CurArcaneCharges, MaxArcaneCharges)` when the item is a cloak, robe or pair of boots
/// that implement it (the gloves are `armor::special`). The setters are [`set_cur_arcane_charges`] and
/// [`set_max_arcane_charges`].
pub fn arcane_equip(w: &World, id: ItemId) -> Option<(i32, i32)> {
    arcane_ids(w.item(id)?.ty)?;
    clothing::data(w, id).map(|d| d.arcane_charges)
}

/// `IsArcane => _maxArcaneCharges > 0 && _curArcaneCharges >= 0`.
pub fn is_arcane(w: &World, id: ItemId) -> bool {
    clothing::data(w, id).is_some_and(|d| d.arcane_charges.1 > 0 && d.arcane_charges.0 >= 0)
}

/// `Update()`.
pub fn arcane_update(w: &mut World, id: ItemId) {
    let Some((arcane, normal)) = w.item(id).and_then(|i| arcane_ids(i.ty)) else {
        return;
    };
    let current = w.item(id).map_or(0, |i| i.item_id);
    let cur = clothing::data(w, id).map_or(0, |d| d.arcane_charges.0);
    if is_arcane(w, id) {
        item_id(w, id, arcane);
    } else if current == arcane {
        item_id(w, id, normal);
    }
    if is_arcane(w, id) && cur == 0 {
        hue(w, id, 0);
    }
}

fn item_id(w: &mut World, id: ItemId, value: i32) {
    if let Some(ty) = w.item(id).map(|i| i.ty) {
        (ty.set_item_id)(w, id, value);
    }
}

/// `CurArcaneCharges` setter (marks dirty, invalidates properties, runs `Update()`).
pub fn set_cur_arcane_charges(w: &mut World, id: ItemId, value: i32) {
    let Some(d) = clothing::data_mut(w, id) else {
        return;
    };
    if d.arcane_charges.0 == value {
        return;
    }
    d.arcane_charges.0 = value;
    ops::invalidate_properties(w, id);
    arcane_update(w, id);
}

/// `MaxArcaneCharges` setter.
pub fn set_max_arcane_charges(w: &mut World, id: ItemId, value: i32) {
    let Some(d) = clothing::data_mut(w, id) else {
        return;
    };
    if d.arcane_charges.1 == value {
        return;
    }
    d.arcane_charges.1 = value;
    ops::invalidate_properties(w, id);
    arcane_update(w, id);
}

fn arcane_get_properties(
    w: &World,
    id: ItemId,
    list: &mut PropertyList,
    base: fn(&World, ItemId, &mut PropertyList),
) {
    base(w, id, list);
    if is_arcane(w, id)
        && let Some(d) = clothing::data(w, id)
    {
        // arcane charges: ~1_val~ / ~2_val~
        list.add_str(
            1061837,
            &format!("{}\t{}", d.arcane_charges.0, d.arcane_charges.1),
        );
    }
}

fn arcane_single_click(
    w: &mut World,
    id: ItemId,
    from: MobileId,
    base: fn(&mut World, ItemId, MobileId),
) {
    base(w, id, from);
    if is_arcane(w, id)
        && let Some(d) = clothing::data(w, id)
    {
        let args = format!("{}\t{}", d.arcane_charges.0, d.arcane_charges.1);
        ops::label_to_number(w, id, from, 1061837, &args);
    }
}

/// `Flip()` of the three: swaps between the two graphics of the pair, anything else is left alone.
fn flip_pair(w: &mut World, id: ItemId, a: i32, b: i32) {
    let current = w.item(id).map_or(0, |i| i.item_id);
    if current == a {
        item_id(w, id, b);
    } else if current == b {
        item_id(w, id, a);
    }
}

fn cloak_flip(w: &mut World, id: ItemId) {
    flip_pair(w, id, 0x1515, 0x1530);
}

fn robe_flip(w: &mut World, id: ItemId) {
    flip_pair(w, id, 0x1F03, 0x1F04);
}

fn thigh_boots_flip(w: &mut World, id: ItemId) {
    flip_pair(w, id, 0x1711, 0x1712);
}

/// The generated reader of an `IArcaneEquip` class at `[SerializationGenerator(2, false)]`: version 2 is two
/// encoded ints, older records are a bool and (when set) two ints (`Deserialize(reader, version)`); neither runs
/// the setters, so a load does not call `Update()`.
fn arcane_load(w: &mut World, id: ItemId, r: &mut dyn GenericReader) -> Result<()> {
    let version = r.read_i32()?;
    let charges = if version < 2 {
        if r.read_bool()? {
            Some((r.read_i32()?, r.read_i32()?))
        } else {
            None
        }
    } else {
        Some((r.read_encoded_int()?, r.read_encoded_int()?))
    };
    if let (Some(c), Some(d)) = (charges, clothing::data_mut(w, id)) {
        d.arcane_charges = c;
    }
    Ok(())
}

fn arcane_save(w: &World, id: ItemId, out: &mut dyn GenericWriter) {
    out.write_i32(2);
    let (cur, max) = clothing::data(w, id).map_or((0, 0), |d| d.arcane_charges);
    out.write_encoded_int(cur);
    out.write_encoded_int(max);
}

// ---- Cloak ----

/// `Cloak(hue)`: `base(0x1515, hue)`.
pub fn cloak_new(w: &mut World, ty: &'static ItemType, hue: i32) -> ItemId {
    new_clothing(w, ty, 0x1515, hue)
}

/// `Cloak`' constants (`[Flippable]` with `Flip()`).
pub static CLOAK_CLASS: ClothingClass = ClothingClass {
    default_weight: Some(5.0),
    flip: Some(cloak_flip),
    ..BASE_CLOAK_CLASS
};

/// `Server.Items.Cloak`.
pub static CLOAK: ItemType = ItemType {
    full_name: "Server.Items.Cloak",
    parent: Some(&BASE_CLOAK),
    construct: Some(|w| cloak_new(w, &CLOAK, 0)),
    family: Some(&CLOAK_CLASS),
    get_properties: |w, id, l| arcane_get_properties(w, id, l, BASE_CLOAK.get_properties),
    on_single_click: |w, id, f| arcane_single_click(w, id, f, BASE_CLOAK.on_single_click),
    serialize: |w, id, out| {
        (BASE_CLOAK.serialize)(w, id, out);
        arcane_save(w, id, out);
    },
    deserialize: |w, id, r| {
        (BASE_CLOAK.deserialize)(w, id, r)?;
        arcane_load(w, id, r)
    },
    ..BASE_CLOAK
};

// ---- Robe, DeathRobe ----

/// `Robe(hue)`: `base(0x1F03, hue)`.
pub fn robe_new(w: &mut World, ty: &'static ItemType, hue: i32) -> ItemId {
    new_clothing(w, ty, 0x1F03, hue)
}

/// `Robe`' constants.
pub static ROBE_CLASS: ClothingClass = ClothingClass {
    default_weight: Some(3.0),
    flip: Some(robe_flip),
    ..BASE_OUTER_TORSO_CLASS
};

/// `Server.Items.Robe`.
pub static ROBE: ItemType = ItemType {
    full_name: "Server.Items.Robe",
    parent: Some(&BASE_OUTER_TORSO),
    construct: Some(|w| robe_new(w, &ROBE, 0)),
    family: Some(&ROBE_CLASS),
    get_properties: |w, id, l| arcane_get_properties(w, id, l, BASE_OUTER_TORSO.get_properties),
    on_single_click: |w, id, f| arcane_single_click(w, id, f, BASE_OUTER_TORSO.on_single_click),
    serialize: |w, id, out| {
        (BASE_OUTER_TORSO.serialize)(w, id, out);
        arcane_save(w, id, out);
    },
    deserialize: |w, id, r| {
        (BASE_OUTER_TORSO.deserialize)(w, id, r)?;
        arcane_load(w, id, r)
    },
    ..BASE_OUTER_TORSO
};

const DEATH_ROBE_DECAY: TimeSpan = TimeSpan::from_secs(60);

fn death_robe_stop_timer(w: &mut World, id: ItemId) {
    if let Some(d) = clothing::data_mut(w, id) {
        let mut h = d.decay_timer;
        d.decay_timer = TimerHandle::default();
        h.cancel(w);
    }
}

/// `BeginDecay(delay)`: the timer deletes the robe unless it is worn or locked down.
pub fn death_robe_begin_decay(w: &mut World, id: ItemId, delay: TimeSpan) {
    death_robe_stop_timer(w, id);
    let delay = if delay < TimeSpan::ZERO {
        TimeSpan::ZERO
    } else {
        delay
    };
    let h = timer::start(w, delay, TimeSpan::ZERO, 1, move |w| {
        let Some(i) = w.item(id) else { return };
        if i.parent.is_some() || ops::is_locked_down(w, id) {
            // `Stop(); _robe._decayTimer = null;`
            if let Some(d) = clothing::data_mut(w, id) {
                d.decay_timer = TimerHandle::default();
            }
        } else if let Some(ty) = w.item(id).map(|i| i.ty) {
            (ty.delete)(w, id);
        }
    });
    if let Some(d) = clothing::data_mut(w, id) {
        d.decay_timer = h;
    }
}

/// `DeathRobe()`: `LootType = Newbied; Hue = 2301; BeginDecay(m_DefaultDecayTime)`.
pub fn death_robe_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = robe_new(w, ty, 0);
    ops::set_loot_type(w, id, LootType::Newbied);
    hue(w, id, 2301);
    death_robe_begin_decay(w, id, DEATH_ROBE_DECAY);
    id
}

fn to_date(ticks: i64) -> DateTime {
    DateTime::try_from_ticks(ticks).unwrap_or(DateTime::MIN)
}

/// `Server.Items.DeathRobe` (`[SerializationGenerator(4, false)]`: version, then the timer's anchored time).
pub static DEATH_ROBE: ItemType = ItemType {
    full_name: "Server.Items.DeathRobe",
    parent: Some(&ROBE),
    construct: Some(|w| death_robe_new(w, &DEATH_ROBE)),
    display_loot_type: |_, _| false,
    on_dropped_to_world: |w, id, _from, _p| {
        death_robe_begin_decay(w, id, DEATH_ROBE_DECAY);
        true
    },
    on_dropped_into: |w, id, from, target, p| {
        if (ROBE.on_dropped_into)(w, id, from, target, p) {
            death_robe_stop_timer(w, id);
            return true;
        }
        false
    },
    on_dropped_to_mobile: |w, id, _from, _target| {
        death_robe_stop_timer(w, id);
        true
    },
    // `OnAfterDelete() { StopTimer(); }` does not call the base.
    on_after_delete: |w, id| death_robe_stop_timer(w, id),
    serialize: |w, id, out| {
        (ROBE.serialize)(w, id, out);
        out.write_i32(4);
        let h = clothing::data(w, id)
            .map(|d| d.decay_timer)
            .unwrap_or_default();
        let next = if h == TimerHandle::default() {
            DateTime::MIN
        } else {
            h.next(w)
        };
        out.write_anchored_time_ticks(next.ticks());
    },
    deserialize: |w, id, r| {
        (ROBE.deserialize)(w, id, r)?;
        let version = r.read_i32()?;
        let now = w.now;
        let delay = match version {
            // `Deserialize(reader, version)`: a bool, then a delta time.
            0..=2 => {
                if r.read_bool()? {
                    let t = to_date(r.read_delta_time_ticks(now.ticks())?);
                    Some(TimeSpan::from_ticks(t.ticks() - now.ticks()))
                } else {
                    None
                }
            }
            // V3Content: a delta time.
            3 => {
                let t = to_date(r.read_delta_time_ticks(now.ticks())?);
                (t != DateTime::MIN).then(|| TimeSpan::from_ticks(t.ticks() - now.ticks()))
            }
            _ => {
                let t = to_date(r.read_anchored_time_ticks()?);
                (t != DateTime::MIN).then(|| TimeSpan::from_ticks(t.ticks() - now.ticks()))
            }
        };
        if let Some(delay) = delay {
            death_robe_begin_decay(w, id, delay);
        }
        Ok(())
    },
    ..ROBE
};

// ---- ThighBoots ----

/// `ThighBoots(hue)`: `base(0x1711, hue)`.
pub fn thigh_boots_new(w: &mut World, ty: &'static ItemType, hue: i32) -> ItemId {
    new_clothing(w, ty, 0x1711, hue)
}

/// `ThighBoots`' constants.
pub static THIGH_BOOTS_CLASS: ClothingClass = ClothingClass {
    default_weight: Some(4.0),
    default_resource: muo_content::craft::CraftResource::RegularLeather,
    flip: Some(thigh_boots_flip),
    ..BASE_SHOES_CLASS
};

/// `Server.Items.ThighBoots`.
pub static THIGH_BOOTS: ItemType = ItemType {
    full_name: "Server.Items.ThighBoots",
    parent: Some(&BASE_SHOES),
    construct: Some(|w| thigh_boots_new(w, &THIGH_BOOTS, 0)),
    family: Some(&THIGH_BOOTS_CLASS),
    get_properties: |w, id, l| arcane_get_properties(w, id, l, BASE_SHOES.get_properties),
    on_single_click: |w, id, f| arcane_single_click(w, id, f, BASE_SHOES.on_single_click),
    serialize: |w, id, out| {
        (BASE_SHOES.serialize)(w, id, out);
        arcane_save(w, id, out);
    },
    deserialize: |w, id, r| {
        (BASE_SHOES.deserialize)(w, id, r)?;
        arcane_load(w, id, r)
    },
    ..BASE_SHOES
};

// ---- masks ----

/// `OrcishKinMask(hue = 0x8A4)`: `base(0x141B, hue)`.
pub fn orcish_kin_mask_new(w: &mut World, ty: &'static ItemType, hue: i32) -> ItemId {
    new_clothing(w, ty, 0x141B, hue)
}

/// `OrcishKinMask`' constants.
pub static ORCISH_KIN_MASK_CLASS: ClothingClass = ClothingClass {
    default_weight: Some(2.0),
    base_physical_resistance: 1,
    base_fire_resistance: 1,
    base_cold_resistance: 7,
    base_poison_resistance: 7,
    base_energy_resistance: 8,
    init_min_hits: muo_content::items::same(20),
    init_max_hits: muo_content::items::same(30),
    dye: dye_refused,
    ..BASE_HAT_CLASS
};

/// `Server.Items.OrcishKinMask`.
pub static ORCISH_KIN_MASK: ItemType = ItemType {
    full_name: "Server.Items.OrcishKinMask",
    parent: Some(&BASE_HAT),
    construct: Some(|w| orcish_kin_mask_new(w, &ORCISH_KIN_MASK, 0x8A4)),
    family: Some(&ORCISH_KIN_MASK_CLASS),
    default_name: |_, _| Some("a mask of orcish kin".to_string()),
    can_equip: |w, id, m| {
        if !(BASE_HAT.can_equip)(w, id, m) {
            return false;
        }
        let body_mod = w.mobile(m).map_or(0, |m| m.body_mod.body_id());
        if body_mod == 183 || body_mod == 184 {
            deps::send_localized_message(w, m, 1061629); // You can't do that while wearing savage kin paint.
            return false;
        }
        true
    },
    on_added: |w, id, parent| {
        (BASE_HAT.on_added)(w, id, parent);
        if let EntityId::Mobile(m) = parent {
            muo_content::misc::titles::award_karma(w, m, -20, true);
        }
    },
    serialize: |w, id, out| {
        (BASE_HAT.serialize)(w, id, out);
        out.write_i32(0);
    },
    deserialize: |w, id, r| {
        (BASE_HAT.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        Ok(())
    },
    ..BASE_HAT
};

/// `SavageMask.GetRandomHue()`.
pub fn savage_mask_random_hue(w: &mut World) -> i32 {
    let v = utility::random_bird_hue(&mut w.rng);
    if v == 2101 { 0 } else { v }
}

/// `SavageMask(hue)`: `base(0x154B, hue)`.
pub fn savage_mask_new(w: &mut World, ty: &'static ItemType, hue: i32) -> ItemId {
    new_clothing(w, ty, 0x154B, hue)
}

/// `SavageMask`' constants.
pub static SAVAGE_MASK_CLASS: ClothingClass = ClothingClass {
    default_weight: Some(2.0),
    base_physical_resistance: 3,
    base_fire_resistance: 0,
    base_cold_resistance: 6,
    base_poison_resistance: 10,
    base_energy_resistance: 5,
    init_min_hits: muo_content::items::same(20),
    init_max_hits: muo_content::items::same(30),
    dye: dye_refused,
    ..BASE_HAT_CLASS
};

/// `Server.Items.SavageMask` (two `[Constructible]` constructors: `SavageMask()` is `this(GetRandomHue())`).
pub static SAVAGE_MASK: ItemType = ItemType {
    full_name: "Server.Items.SavageMask",
    parent: Some(&BASE_HAT),
    construct: Some(|w| {
        let hue = savage_mask_random_hue(w);
        savage_mask_new(w, &SAVAGE_MASK, hue)
    }),
    family: Some(&SAVAGE_MASK_CLASS),
    serialize: |w, id, out| {
        (BASE_HAT.serialize)(w, id, out);
        out.write_i32(0);
    },
    deserialize: |w, id, r| {
        (BASE_HAT.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        Ok(())
    },
    ..BASE_HAT
};

/// Registers the classes of this file.
pub fn register(types: &mut TypeRegistry) {
    for t in [
        &CLOAK,
        &REWARD_CLOAK,
        &ROBE,
        &DEATH_ROBE,
        &REWARD_ROBE,
        &REWARD_DRESS,
        &THIGH_BOOTS,
        &ORCISH_KIN_MASK,
        &SAVAGE_MASK,
    ] {
        types.register_item(t, &[]);
    }
}
