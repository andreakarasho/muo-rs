//! `Item.Serialize` / `Item.Deserialize` / `BounceInfo`. The bytes must stay identical with existing save files;
//! versions 0 to 11 load.

use muo_io::{GenericReader, GenericWriter, Result};
use muo_primitives::{DateTime, Direction, Point3D, Serial};

use crate::ids::{EntityId, ItemId, MobileId};
use crate::item::container::CONTAINER;
use crate::item::deps;
use crate::item::{BounceInfo, ImplFlag, LootType, SaveFlag};
use crate::layer::Layer;
use crate::map::MapId;
use crate::world::World;

/// The version `Serialize` writes.
pub const VERSION: i32 = 11;

/// `writer.Write(ISerializable)`: the serial, or -1 for null or deleted.
pub fn write_entity(w: &World, writer: &mut dyn GenericWriter, e: Option<EntityId>) {
    let serial = match e {
        Some(EntityId::Item(i)) if !crate::item::ops::is_deleted(w, i) => i.0,
        Some(EntityId::Mobile(m)) if !deps::mobile_deleted(w, m) => m.0,
        _ => Serial::MINUS_ONE,
    };
    writer.write_serial(serial);
}

/// `writer.Write(Mobile)`.
pub fn write_mobile(w: &World, writer: &mut dyn GenericWriter, m: Option<MobileId>) {
    write_entity(w, writer, m.map(EntityId::Mobile));
}

/// `writer.Write(List<Item>)`: the count, then each serial (-1 for a deleted one).
pub fn write_item_list(w: &World, writer: &mut dyn GenericWriter, items: &[ItemId]) {
    writer.write_i32(items.len() as i32);
    for &i in items {
        write_entity(w, writer, Some(EntityId::Item(i)));
    }
}

/// `reader.ReadEntity<Mobile>()`: `None` when the serial names no mobile.
pub fn read_mobile(w: &World, r: &mut dyn GenericReader) -> Result<Option<MobileId>> {
    let s = r.read_serial()?;
    Ok(Some(MobileId(s)).filter(|m| w.mobile(*m).is_some()))
}

/// `reader.ReadEntity<Item>()`.
pub fn read_item(w: &World, r: &mut dyn GenericReader) -> Result<Option<ItemId>> {
    let s = r.read_serial()?;
    Ok(Some(ItemId(s)).filter(|i| w.item(*i).is_some()))
}

/// `reader.ReadEntity<IEntity>()`.
pub fn read_entity(w: &World, r: &mut dyn GenericReader) -> Result<Option<EntityId>> {
    let s = r.read_serial()?;
    Ok(EntityId::from_serial(s).filter(|e| match e {
        EntityId::Item(i) => w.item(*i).is_some(),
        EntityId::Mobile(m) => w.mobile(*m).is_some(),
    }))
}

/// `reader.ReadEntityList<Item>()`: unresolved serials are left out.
pub fn read_item_list(w: &World, r: &mut dyn GenericReader) -> Result<Vec<ItemId>> {
    let count = r.read_i32()?;
    let mut list = Vec::with_capacity(count.clamp(0, 4096) as usize);
    for _ in 0..count.max(0) {
        if let Some(i) = read_item(w, r)? {
            list.push(i);
        }
    }
    Ok(list)
}

fn map_from_index(i: Option<u8>) -> Option<MapId> {
    i.map(MapId)
}

/// `BounceInfo.Serialize`.
pub fn serialize_bounce(w: &World, info: Option<&BounceInfo>, writer: &mut dyn GenericWriter) {
    match info {
        None => writer.write_bool(false),
        Some(info) => {
            writer.write_bool(true);
            writer.write_map_index(info.map.map(|m| m.0));
            writer.write_point3d(info.location);
            writer.write_point3d(info.world_loc);
            match info.parent {
                Some(p) => write_entity(w, writer, Some(p)),
                None => writer.write_serial(Serial::ZERO),
            }
        }
    }
}

/// `BounceInfo.Deserialize`.
pub fn deserialize_bounce(w: &World, reader: &mut dyn GenericReader) -> Result<Option<BounceInfo>> {
    if reader.read_bool()? {
        let map = map_from_index(reader.read_map_index()?);
        let location = reader.read_point3d()?;
        let world_loc = reader.read_point3d()?;
        let parent = read_entity(w, reader)?;
        Ok(Some(BounceInfo {
            location,
            world_loc,
            map,
            parent,
        }))
    } else {
        Ok(None)
    }
}

pub fn serialize(w: &World, id: ItemId, writer: &mut dyn GenericWriter) {
    let Some(i) = w.item(id) else { return };
    writer.write_i32(VERSION);

    let mut flags = SaveFlag::NONE;

    let (x, y, z) = (i.location.x, i.location.y, i.location.z);

    if x != 0 || y != 0 || z != 0 {
        if (i32::from(i16::MIN)..=i32::from(i16::MAX)).contains(&x)
            && (i32::from(i16::MIN)..=i32::from(i16::MAX)).contains(&y)
            && (i32::from(i8::MIN)..=i32::from(i8::MAX)).contains(&z)
        {
            if x != 0 || y != 0 {
                if (0..=255).contains(&x) && (0..=255).contains(&y) {
                    flags |= SaveFlag::LOCATION_BYTE_XY;
                } else {
                    flags |= SaveFlag::LOCATION_SHORT_XY;
                }
            }
            if z != 0 {
                flags |= SaveFlag::LOCATION_SBYTE_Z;
            }
        } else {
            flags |= SaveFlag::LOCATION_FULL;
        }
    }

    // Skipping containers never write their child serials: those children are not saved as top-level records
    // (see SkipSerialization), so a written serial can be handed to an unrelated new item during the next
    // load's synchronous deserialize loop.
    let skips = i.ty.is(&CONTAINER) && (i.ty.skips_child_serialization)(w, id);
    let items: &[ItemId] = if skips { &[] } else { &i.items };

    if i.direction != Direction::NORTH {
        flags |= SaveFlag::DIRECTION;
    }
    if i.bounce.is_some() {
        flags |= SaveFlag::BOUNCE;
    }
    if i.loot_type != LootType::Regular {
        flags |= SaveFlag::LOOT_TYPE;
    }
    if i.item_id != 0 {
        flags |= SaveFlag::ITEM_ID;
    }
    if i.hue != 0 {
        flags |= SaveFlag::HUE;
    }
    if i.amount != 1 {
        flags |= SaveFlag::AMOUNT;
    }
    if i.layer != Layer::Invalid {
        flags |= SaveFlag::LAYER;
    }
    if i.name.is_some() {
        flags |= SaveFlag::NAME;
    }
    if i.parent.is_some() {
        flags |= SaveFlag::PARENT;
    }
    if !items.is_empty() {
        flags |= SaveFlag::ITEMS;
    }
    if i.map != Some(MapId::INTERNAL) {
        flags |= SaveFlag::MAP;
    }

    if i.blessed_for.is_some_and(|m| !deps::mobile_deleted(w, m)) {
        flags |= SaveFlag::BLESSED_FOR;
    }
    if i.held_by.is_some_and(|m| !deps::mobile_deleted(w, m)) {
        flags |= SaveFlag::HELD_BY;
    }
    if i.saved_flags != 0 {
        flags |= SaveFlag::SAVED_FLAGS;
    }
    if i.decay_reset > i.last_moved {
        flags |= SaveFlag::DECAY_RESET;
    }

    if i.weight < 0.0 {
        flags |= SaveFlag::NULL_WEIGHT;
    } else if i.weight == 0.0 {
        flags |= SaveFlag::WEIGHT_IS_0;
    } else if i.weight == f64::from(i.weight as i32) {
        flags |= SaveFlag::INT_WEIGHT;
    } else {
        flags |= SaveFlag::WEIGHT_NOT_1_OR_0;
    }

    let impl_flags = i.flags
        & (ImplFlag::VISIBLE
            | ImplFlag::MOVABLE
            | ImplFlag::STACKABLE
            | ImplFlag::INSURED
            | ImplFlag::PAID_INSURANCE
            | ImplFlag::QUEST_ITEM);

    if impl_flags != (ImplFlag::VISIBLE | ImplFlag::MOVABLE) {
        flags |= SaveFlag::IMPL_FLAGS;
    }
    if i.player_constructed {
        flags |= SaveFlag::PLAYER_CONSTRUCTED;
    }

    writer.write_i32(flags.bits() as i32);

    // Anchored: shifted by downtime at load, so time-since-moved is preserved and the bytes are stable across
    // saves while the item does not move.
    writer.write_anchored_time_ticks(i.last_moved.ticks());

    if flags.contains(SaveFlag::DECAY_RESET) {
        writer.write_anchored_time_ticks(i.decay_reset.ticks());
    }
    if flags.contains(SaveFlag::DIRECTION) {
        writer.write_u8(i.direction.bits());
    }
    if flags.contains(SaveFlag::BOUNCE) {
        serialize_bounce(w, i.bounce.as_ref(), writer);
    }
    if flags.contains(SaveFlag::LOOT_TYPE) {
        writer.write_u8(i.loot_type.0);
    }

    if flags.contains(SaveFlag::LOCATION_FULL) {
        writer.write_encoded_int(x);
        writer.write_encoded_int(y);
        writer.write_encoded_int(z);
    } else {
        if flags.contains(SaveFlag::LOCATION_BYTE_XY) {
            writer.write_u8(x as u8);
            writer.write_u8(y as u8);
        } else if flags.contains(SaveFlag::LOCATION_SHORT_XY) {
            writer.write_i16(x as i16);
            writer.write_i16(y as i16);
        }
        if flags.contains(SaveFlag::LOCATION_SBYTE_Z) {
            writer.write_i8(z as i8);
        }
    }

    if flags.contains(SaveFlag::ITEM_ID) {
        writer.write_encoded_int(i.item_id);
    }
    if flags.contains(SaveFlag::HUE) {
        writer.write_encoded_int(i.hue);
    }
    if flags.contains(SaveFlag::AMOUNT) {
        writer.write_encoded_int(i.amount);
    }
    if flags.contains(SaveFlag::LAYER) {
        writer.write_u8(i.layer.0);
    }
    if flags.contains(SaveFlag::NAME) {
        writer.write_opt_string(i.name.as_deref());
    }
    if flags.contains(SaveFlag::PARENT) {
        write_entity(w, writer, i.parent);
    }
    if flags.contains(SaveFlag::ITEMS) {
        write_item_list(w, writer, items);
    }

    if flags.contains(SaveFlag::INT_WEIGHT) {
        writer.write_encoded_int(i.weight as i32);
    } else if flags.contains(SaveFlag::WEIGHT_NOT_1_OR_0) {
        writer.write_f64(i.weight);
    }

    if flags.contains(SaveFlag::MAP) {
        writer.write_map_index(i.map.map(|m| m.0));
    }
    if flags.contains(SaveFlag::IMPL_FLAGS) {
        writer.write_encoded_int(i32::from(impl_flags.bits()));
    }
    if flags.contains(SaveFlag::INSURED_FOR) {
        writer.write_serial(Serial::MINUS_ONE);
    }
    if flags.contains(SaveFlag::BLESSED_FOR) {
        write_mobile(w, writer, i.blessed_for);
    }
    if flags.contains(SaveFlag::HELD_BY) {
        write_mobile(w, writer, i.held_by);
    }
    if flags.contains(SaveFlag::SAVED_FLAGS) {
        writer.write_encoded_int(i.saved_flags);
    }
}

/// `m_Parent = World.FindMobile/FindItem(serial)`, and the delayed delete of an item whose parent is gone.
fn read_parent(
    w: &mut World,
    id: ItemId,
    reader: &mut dyn GenericReader,
    skip_check: bool,
) -> Result<()> {
    let parent = reader.read_serial()?;
    let mut found = if parent.is_mobile() {
        Some(MobileId(parent))
            .filter(|m| w.mobile(*m).is_some())
            .map(EntityId::Mobile)
    } else if parent.is_item() {
        Some(ItemId(parent))
            .filter(|i| w.item(*i).is_some())
            .map(EntityId::Item)
    } else {
        None
    };

    // A skipping container never wrote its children's serials (see Serialize), so this item's frozen parent
    // reference is stale/unreachable: treat it as a missing parent.
    if skip_check
        && let Some(EntityId::Item(p)) = found
        && w.item(p)
            .is_some_and(|c| c.ty.is(&CONTAINER) && (c.ty.skips_child_serialization)(w, p))
    {
        found = None;
    }

    if let Some(i) = w.item_mut(id) {
        i.parent = found;
    }

    if found.is_none() && (parent.is_mobile() || parent.is_item()) {
        deps::delay_delete(w, id);
    }
    Ok(())
}

/// Weight of a v5+ record: `IntWeight`, `WeightNot1or0`, `WeightIs0`, else 1.
fn read_weight(flags: SaveFlag, reader: &mut dyn GenericReader) -> Result<f64> {
    Ok(if flags.contains(SaveFlag::INT_WEIGHT) {
        f64::from(reader.read_encoded_int()?)
    } else if flags.contains(SaveFlag::WEIGHT_NOT_1_OR_0) {
        reader.read_f64()?
    } else if flags.contains(SaveFlag::WEIGHT_IS_0) {
        0.0
    } else {
        1.0
    })
}

fn set_name_if_not_default(w: &mut World, id: ItemId, name: Option<String>) {
    let Some(ty) = w.item(id).map(|i| i.ty) else {
        return;
    };
    if name != (ty.default_name)(w, id)
        && let Some(i) = w.item_mut(id)
    {
        i.name = name;
    }
}

fn set_weight_if_not_default(w: &mut World, id: ItemId, weight: f64) {
    let Some(ty) = w.item(id).map(|i| i.ty) else {
        return;
    };
    if weight != (ty.default_weight)(w, id)
        && let Some(i) = w.item_mut(id)
    {
        i.weight = weight;
    }
}

pub fn deserialize(w: &mut World, id: ItemId, reader: &mut dyn GenericReader) -> Result<()> {
    let version = reader.read_i32()?;
    let now = w.now;

    // Default for versions without a saved value. Stamp only: map and parent are still unread, and
    // PostDeserialize registers every item once its state is final.
    mutate(w, id, |i| i.last_moved = now);

    match version {
        6..=11 => {
            let flags = SaveFlag::from_bits_retain(reader.read_i32()? as u32);

            let last_moved = if version >= 11 {
                DateTime::from_ticks(reader.read_anchored_time_ticks()?)
            } else if version < 7 {
                DateTime::from_ticks(reader.read_delta_time_ticks(now.ticks())?)
            } else {
                let minutes = reader.read_encoded_int()?;
                // `Core.Now - TimeSpan.FromMinutes(minutes)`; an out of range result is `Core.Now`.
                now.checked_add_ticks(-i64::from(minutes) * 600_000_000)
                    .unwrap_or(now)
            };
            crate::item::ops::set_last_moved_value(w, id, last_moved);

            if version >= 10 && flags.contains(SaveFlag::DECAY_RESET) {
                let reset = if version >= 11 {
                    DateTime::from_ticks(reader.read_anchored_time_ticks()?)
                } else {
                    DateTime::from_ticks(reader.read_delta_time_ticks(now.ticks())?)
                };
                // Pre-v11 LastMoved was stored at whole-minute precision; keep the stamp only while it still
                // extends the deadline.
                if reset > last_moved {
                    mutate(w, id, |i| i.decay_reset = reset);
                }
            }

            if flags.contains(SaveFlag::DIRECTION) {
                let d = reader.read_u8()?;
                mutate(w, id, |i| i.direction = Direction::from_bits_retain(d));
            }
            if flags.contains(SaveFlag::BOUNCE) {
                let b = deserialize_bounce(w, reader)?;
                mutate(w, id, |i| i.bounce = b);
            }
            if flags.contains(SaveFlag::LOOT_TYPE) {
                let l = reader.read_u8()?;
                mutate(w, id, |i| i.loot_type = LootType::from_raw(l));
            }

            let (mut x, mut y, mut z) = (0, 0, 0);
            if flags.contains(SaveFlag::LOCATION_FULL) {
                x = reader.read_encoded_int()?;
                y = reader.read_encoded_int()?;
                z = reader.read_encoded_int()?;
            } else {
                if flags.contains(SaveFlag::LOCATION_BYTE_XY) {
                    x = i32::from(reader.read_u8()?);
                    y = i32::from(reader.read_u8()?);
                } else if flags.contains(SaveFlag::LOCATION_SHORT_XY) {
                    x = i32::from(reader.read_i16()?);
                    y = i32::from(reader.read_i16()?);
                }
                if flags.contains(SaveFlag::LOCATION_SBYTE_Z) {
                    z = i32::from(reader.read_i8()?);
                }
            }
            mutate(w, id, |i| i.location = Point3D::new(x, y, z));

            if flags.contains(SaveFlag::ITEM_ID) {
                let v = reader.read_encoded_int()?;
                mutate(w, id, |i| i.item_id = v);
            }
            if flags.contains(SaveFlag::HUE) {
                let v = reader.read_encoded_int()?;
                mutate(w, id, |i| i.hue = v);
            }
            let amount = if flags.contains(SaveFlag::AMOUNT) {
                reader.read_encoded_int()?
            } else {
                1
            };
            mutate(w, id, |i| i.amount = amount);

            if flags.contains(SaveFlag::LAYER) {
                let l = reader.read_u8()?;
                mutate(w, id, |i| i.layer = layer_from_u8(l));
            }
            if flags.contains(SaveFlag::NAME) {
                let name = reader.read_string()?;
                set_name_if_not_default(w, id, name);
            }
            if flags.contains(SaveFlag::PARENT) {
                read_parent(w, id, reader, true)?;
            }
            if flags.contains(SaveFlag::ITEMS) {
                let items = read_item_list(w, reader)?;
                mutate(w, id, |i| i.items = items);
            }

            if version < 8 || !flags.contains(SaveFlag::NULL_WEIGHT) {
                let weight = read_weight(flags, reader)?;
                set_weight_if_not_default(w, id, weight);
            }

            let map = if flags.contains(SaveFlag::MAP) {
                map_from_index(reader.read_map_index()?)
            } else {
                Some(MapId::INTERNAL)
            };
            mutate(w, id, |i| i.map = map);

            let visible = !flags.contains(SaveFlag::VISIBLE) || reader.read_bool()?;
            mutate(w, id, |i| i.set_flag(ImplFlag::VISIBLE, visible));
            let movable = !flags.contains(SaveFlag::MOVABLE) || reader.read_bool()?;
            mutate(w, id, |i| i.set_flag(ImplFlag::MOVABLE, movable));

            if flags.contains(SaveFlag::STACKABLE) {
                let s = reader.read_bool()?;
                mutate(w, id, |i| i.set_flag(ImplFlag::STACKABLE, s));
            }
            if flags.contains(SaveFlag::IMPL_FLAGS) {
                let f = reader.read_encoded_int()?;
                mutate(w, id, |i| i.flags = ImplFlag::from_bits_retain(f as u8));
            }
            if flags.contains(SaveFlag::INSURED_FOR) {
                read_mobile(w, reader)?; // m_InsuredFor
            }
            if flags.contains(SaveFlag::BLESSED_FOR) {
                let m = read_mobile(w, reader)?;
                mutate(w, id, |i| i.blessed_for = m);
            }
            if flags.contains(SaveFlag::HELD_BY) {
                let m = read_mobile(w, reader)?;
                mutate(w, id, |i| i.held_by = m);
            }
            if flags.contains(SaveFlag::SAVED_FLAGS) {
                let v = reader.read_encoded_int()?;
                mutate(w, id, |i| i.saved_flags = v);
            }
            mutate(w, id, |i| {
                i.player_constructed = flags.contains(SaveFlag::PLAYER_CONSTRUCTED)
            });

            enter_map(w, id);
        }
        5 => {
            let flags = SaveFlag::from_bits_retain(reader.read_i32()? as u32);
            let last_moved = DateTime::from_ticks(reader.read_delta_time_ticks(now.ticks())?);
            crate::item::ops::set_last_moved_value(w, id, last_moved);

            if flags.contains(SaveFlag::DIRECTION) {
                let d = reader.read_u8()?;
                mutate(w, id, |i| i.direction = Direction::from_bits_retain(d));
            }
            if flags.contains(SaveFlag::BOUNCE) {
                let b = deserialize_bounce(w, reader)?;
                mutate(w, id, |i| i.bounce = b);
            }
            if flags.contains(SaveFlag::LOOT_TYPE) {
                let l = reader.read_u8()?;
                mutate(w, id, |i| i.loot_type = LootType::from_raw(l));
            }
            if flags.contains(SaveFlag::LOCATION_FULL) {
                let p = reader.read_point3d()?;
                mutate(w, id, |i| i.location = p);
            }
            if flags.contains(SaveFlag::ITEM_ID) {
                let v = reader.read_i32()?;
                mutate(w, id, |i| i.item_id = v);
            }
            if flags.contains(SaveFlag::HUE) {
                let v = reader.read_i32()?;
                mutate(w, id, |i| i.hue = v);
            }
            let amount = if flags.contains(SaveFlag::AMOUNT) {
                reader.read_i32()?
            } else {
                1
            };
            mutate(w, id, |i| i.amount = amount);
            if flags.contains(SaveFlag::LAYER) {
                let l = reader.read_u8()?;
                mutate(w, id, |i| i.layer = layer_from_u8(l));
            }
            if flags.contains(SaveFlag::NAME) {
                let name = reader.read_string()?;
                set_name_if_not_default(w, id, name);
            }
            if flags.contains(SaveFlag::PARENT) {
                read_parent(w, id, reader, true)?;
            }
            if flags.contains(SaveFlag::ITEMS) {
                let items = read_item_list(w, reader)?;
                mutate(w, id, |i| i.items = items);
            }
            let weight = read_weight(flags, reader)?;
            set_weight_if_not_default(w, id, weight);

            let map = if flags.contains(SaveFlag::MAP) {
                map_from_index(reader.read_map_index()?)
            } else {
                Some(MapId::INTERNAL)
            };
            mutate(w, id, |i| i.map = map);

            let visible = !flags.contains(SaveFlag::VISIBLE) || reader.read_bool()?;
            mutate(w, id, |i| i.set_flag(ImplFlag::VISIBLE, visible));
            let movable = !flags.contains(SaveFlag::MOVABLE) || reader.read_bool()?;
            mutate(w, id, |i| i.set_flag(ImplFlag::MOVABLE, movable));
            if flags.contains(SaveFlag::STACKABLE) {
                let s = reader.read_bool()?;
                mutate(w, id, |i| i.set_flag(ImplFlag::STACKABLE, s));
            }

            enter_map(w, id);
        }
        0..=4 => {
            if version >= 3 {
                let d = reader.read_i32()?;
                mutate(w, id, |i| {
                    i.direction = Direction::from_bits_retain(d as u8)
                });
            }
            if version >= 2 {
                let b = deserialize_bounce(w, reader)?;
                mutate(w, id, |i| i.bounce = b);
                let lm = DateTime::from_ticks(reader.read_delta_time_ticks(now.ticks())?);
                crate::item::ops::set_last_moved_value(w, id, lm);
            }
            if version >= 1 {
                let l = reader.read_u8()?;
                mutate(w, id, |i| i.loot_type = LootType::from_raw(l));
            }

            let p = reader.read_point3d()?;
            let item_id = reader.read_i32()?;
            let hue = reader.read_i32()?;
            let amount = reader.read_i32()?;
            let layer = reader.read_u8()?;
            mutate(w, id, |i| {
                i.location = p;
                i.item_id = item_id;
                i.hue = hue;
                i.amount = amount;
                i.layer = layer_from_u8(layer);
            });

            let name = reader.read_string()?;
            set_name_if_not_default(w, id, name);

            read_parent(w, id, reader, false)?;

            let count = reader.read_i32()?;
            if count > 0 {
                let mut items = Vec::with_capacity(count.min(4096) as usize);
                for _ in 0..count {
                    if let Some(item) = read_item(w, reader)? {
                        items.push(item);
                    }
                }
                mutate(w, id, |i| i.items = items);
            }

            let weight = reader.read_f64()?;
            set_weight_if_not_default(w, id, weight);

            if version <= 3 {
                reader.read_i32()?;
                reader.read_i32()?;
                reader.read_i32()?;
            }

            let map = map_from_index(reader.read_map_index()?);
            mutate(w, id, |i| i.map = map);
            let visible = reader.read_bool()?;
            mutate(w, id, |i| i.set_flag(ImplFlag::VISIBLE, visible));
            let movable = reader.read_bool()?;
            mutate(w, id, |i| i.set_flag(ImplFlag::MOVABLE, movable));

            if version <= 3 {
                reader.read_bool()?; // m_Deleted
            }

            let stackable = reader.read_bool()?;
            mutate(w, id, |i| i.set_flag(ImplFlag::STACKABLE, stackable));

            enter_map(w, id);
        }
        _ => {}
    }

    if let Some(held_by) = w.item(id).and_then(|i| i.held_by) {
        let _ = held_by;
        crate::timer::start(
            w,
            muo_primitives::TimeSpan::ZERO,
            muo_primitives::TimeSpan::ZERO,
            1,
            move |w| fix_holding(w, id),
        );
    }

    Ok(())
}

fn mutate(w: &mut World, id: ItemId, f: impl FnOnce(&mut crate::item::Item)) {
    if let Some(i) = w.item_mut(id) {
        f(i);
    }
}

/// `if (m_Map != null && m_Parent == null) m_Map.OnEnter(this)`.
fn enter_map(w: &mut World, id: ItemId) {
    if let Some(i) = w.item(id)
        && let (Some(map), None) = (i.map, i.parent)
    {
        deps::map_on_enter(w, map, id);
    }
}

/// `FixHolding_Sandbox`: an item saved on a cursor goes back where it came from, or into the backpack.
fn fix_holding(w: &mut World, id: ItemId) {
    let Some(held_by) = w.item(id).and_then(|i| i.held_by) else {
        return;
    };
    if w.item(id).is_some_and(|i| i.bounce.is_some()) {
        crate::item::ops::bounce(w, id, held_by);
    } else {
        crate::mobile::part1::props_a::set_holding(w, held_by, None);
        deps::add_to_backpack(w, held_by, id);
        crate::item::ops::clear_bounce(w, id);
    }
}

fn layer_from_u8(v: u8) -> Layer {
    // `(Layer)b`: a cast keeps a byte without a name.
    Layer::from_raw(v)
}
