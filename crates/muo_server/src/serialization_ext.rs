//! Serialization extensions and the entity-aware members of IGenericReader /
//! IGenericWriter (`ReadMap`, `ReadRace`, `ReadType`, `Write(Map)`, `Write(Race)`, `Write(Type)`), as extension
//! traits over `muo_io`'s [`GenericReader`] / [`GenericWriter`]. There is no `MarkDirty` and no
//! wrappers over `Add`/`Remove`/`Insert`: dirty tracking is off and the wrappers would add nothing
//! to `Vec`/`IndexMap` calls.
//!
//! `ReadEntity<T>` looked the entity up through `World.FindEntity` (items and mobiles) or per-type finders
//! registered with `RegisterFindEntity`. Here the entity kinds are the methods: [`ReadEntityExt::read_item`] and
//! friends; another serializable kind (guilds, accounts) reads its own serial with `read_serial` and looks itself up.
//! A reference to something that no longer exists reads as `None`; lists drop those entries.

use indexmap::IndexSet;
use muo_io::{GenericReader, GenericWriter, Result};
use muo_primitives::Serial;

use crate::ids::{EntityId, ItemId, MobileId};
use crate::item::ItemType;
use crate::map::MapId;
use crate::mobile::MobileType;
use crate::poison::PoisonId;
use crate::race::RaceId;
use crate::type_registry::EntityType;
use crate::world::World;
use muo_io::serialization::TypeRef;

pub trait ReadEntityExt: GenericReader {
    /// `ReadEntity<Item>()`: the serial, then `World.FindEntity`; a mobile serial, a deleted or unknown item is `None`.
    fn read_item(&mut self, w: &World) -> Result<Option<ItemId>> {
        let serial = self.read_serial()?;
        Ok(serial.is_item().then(|| w.find_item(serial)).flatten())
    }

    /// `ReadEntity<Mobile>()`.
    fn read_mobile(&mut self, w: &World) -> Result<Option<MobileId>> {
        let serial = self.read_serial()?;
        Ok((!serial.is_item()).then(|| w.find_mobile(serial)).flatten())
    }

    /// `ReadEntity<BaseGuild>()`: through the guild registry; an unknown or removed guild is `None`.
    fn read_guild(&mut self, w: &World) -> Result<Option<crate::guild::GuildId>> {
        let serial = self.read_serial()?;
        Ok(crate::guild::find_guild(w, serial))
    }

    /// `ReadEntityList<Guild>()`: missing guilds are dropped.
    fn read_guild_list(&mut self, w: &World) -> Result<Vec<crate::guild::GuildId>> {
        let count = self.read_i32()?;
        let mut list = Vec::with_capacity(count.clamp(0, 1 << 16) as usize);
        for _ in 0..count {
            list.extend(self.read_guild(w)?);
        }
        Ok(list)
    }

    /// A generated `List<Item>` reader that adds every `ReadEntity<Item>()` result, null or not (no `Tidy`, no null
    /// check): `count` entries come back, one that no longer resolves as an id nothing answers to (it writes back as
    /// -1 and `w.item` finds nothing), so the count and the positions survive. `encoded` is the count's encoding.
    fn read_item_list_kept(&mut self, encoded: bool) -> Result<Vec<ItemId>> {
        let count = if encoded {
            self.read_encoded_int()?
        } else {
            self.read_i32()?
        };
        let mut list = Vec::with_capacity(count.clamp(0, 1 << 16) as usize);
        for _ in 0..count {
            list.push(ItemId(self.read_serial()?));
        }
        Ok(list)
    }

    /// As [`Self::read_item_list_kept`] for `List<Mobile>`.
    fn read_mobile_list_kept(&mut self, encoded: bool) -> Result<Vec<MobileId>> {
        let count = if encoded {
            self.read_encoded_int()?
        } else {
            self.read_i32()?
        };
        let mut list = Vec::with_capacity(count.clamp(0, 1 << 16) as usize);
        for _ in 0..count {
            list.push(MobileId(self.read_serial()?));
        }
        Ok(list)
    }

    /// `ReadEntity<IEntity>()`.
    fn read_entity(&mut self, w: &World) -> Result<Option<EntityId>> {
        let serial = self.read_serial()?;
        Ok(w.find_entity(serial))
    }

    /// `ReadEntityList<Item>(nullIfEmpty: false)`: missing entities are dropped.
    fn read_item_list(&mut self, w: &World) -> Result<Vec<ItemId>> {
        let count = self.read_i32()?;
        read_items_counted(self, w, count)
    }

    /// `ReadEntityList<Item>(nullIfEmpty: true)`: `None` for a stored count of 0.
    fn read_item_list_opt(&mut self, w: &World) -> Result<Option<Vec<ItemId>>> {
        let count = self.read_i32()?;
        if count == 0 {
            return Ok(None);
        }
        read_items_counted(self, w, count).map(Some)
    }

    /// `ReadEntityList<Mobile>()`.
    fn read_mobile_list(&mut self, w: &World) -> Result<Vec<MobileId>> {
        let count = self.read_i32()?;
        let mut list = Vec::with_capacity(count.clamp(0, 1 << 16) as usize);
        for _ in 0..count {
            list.extend(self.read_mobile(w)?);
        }
        Ok(list)
    }

    /// `ReadEntitySet<Item>()`: insertion ordered, as the hash set enumerates until something is removed.
    fn read_item_set(&mut self, w: &World) -> Result<IndexSet<ItemId>> {
        Ok(self.read_item_list(w)?.into_iter().collect())
    }

    /// `ReadEntitySet<Mobile>()`.
    fn read_mobile_set(&mut self, w: &World) -> Result<IndexSet<MobileId>> {
        Ok(self.read_mobile_list(w)?.into_iter().collect())
    }

    /// `ReadMap()`: `Map.Maps[byte]`; 0xFF is no map.
    // shortcut: an index with no registered map should read as null; this has no world to ask, so every index below
    // 0xFF reads as a map and one that never resolves is written back as it was (not as 0xFF). Only a save
    // that names a map the server no longer has differs; fix by passing the world to every caller if that matters.
    fn read_map(&mut self) -> Result<Option<MapId>> {
        Ok(self.read_map_index()?.map(MapId))
    }

    /// `ReadRace()`.
    fn read_race(&mut self) -> Result<Option<RaceId>> {
        Ok(self.read_race_index()?.map(RaceId))
    }

    /// `ReadType()` for a class of the type registry (flag 1: a full name, ignoring case; flag 2: the hash of one).
    fn read_type(&mut self, w: &World) -> Result<Option<EntityType>> {
        Ok(match muo_io::GenericReader::read_type(self)? {
            None => None,
            Some(TypeRef::Name(n)) => w.types.find_type_by_full_name(&n, true),
            Some(TypeRef::Hash(h)) => w.types.find_type_by_hash(h),
        })
    }

    /// `ReadType()` where an item class is expected; another kind of class reads as `None`.
    fn read_item_type(&mut self, w: &World) -> Result<Option<&'static ItemType>> {
        Ok(match ReadEntityExt::read_type(self, w)? {
            Some(EntityType::Item(t)) => Some(t),
            _ => None,
        })
    }

    /// `ReadType()` where a mobile class is expected.
    fn read_mobile_type(&mut self, w: &World) -> Result<Option<&'static MobileType>> {
        Ok(match ReadEntityExt::read_type(self, w)? {
            Some(EntityType::Mobile(t)) => Some(t),
            _ => None,
        })
    }

    /// `ReadPoison()`: a presence flag then the poison's level (`Poison.GetPoisonByIndex`).
    // shortcut: a level that is not registered should read as null (`Poison.GetPoisonByIndex`); this has no world to
    // ask and keeps the level. Only a save naming a poison the server no longer has differs.
    fn read_poison(&mut self) -> Result<Option<PoisonId>> {
        Ok(if self.read_bool()? {
            Some(PoisonId(self.read_u8()?))
        } else {
            None
        })
    }
}

impl<R: GenericReader + ?Sized> ReadEntityExt for R {}

fn read_items_counted<R: GenericReader + ?Sized>(
    r: &mut R,
    w: &World,
    count: i32,
) -> Result<Vec<ItemId>> {
    let mut list = Vec::with_capacity(count.clamp(0, 1 << 16) as usize);
    for _ in 0..count {
        list.extend(r.read_item(w)?);
    }
    Ok(list)
}

pub trait WriteEntityExt: GenericWriter {
    /// `Write(ISerializable)` for an item: its serial, or -1 when it is none or deleted.
    fn write_item(&mut self, w: &World, id: Option<ItemId>) {
        let serial = id
            .and_then(|i| w.item(i))
            .filter(|i| !i.deleted())
            .map_or(Serial::MINUS_ONE, |i| i.serial);
        self.write_serial(serial);
    }

    /// `Write(ISerializable)` for a mobile.
    fn write_mobile(&mut self, w: &World, id: Option<MobileId>) {
        let serial = id
            .and_then(|m| w.mobile(m))
            .filter(|m| !m.deleted)
            .map_or(Serial::MINUS_ONE, |m| m.serial);
        self.write_serial(serial);
    }

    /// `Write(ISerializable)` for a guild: its serial, or -1 when it is none or not in the registry. `Deleted` is
    /// `Disbanded` (a subclass's rule), so a caller that can hold a disbanded guild filters it first.
    fn write_guild(&mut self, w: &World, id: Option<crate::guild::GuildId>) {
        let serial = id
            .and_then(|g| crate::guild::guild(w, g))
            .map_or(Serial::MINUS_ONE, |g| g.serial);
        self.write_serial(serial);
    }

    /// `Write<T>(ICollection<T>)` of guilds.
    fn write_guild_list(&mut self, w: &World, list: &[crate::guild::GuildId]) {
        self.write_i32(list.len() as i32);
        for id in list {
            self.write_guild(w, Some(*id));
        }
    }

    /// `Write(ISerializable)` for an `IEntity`.
    fn write_entity(&mut self, w: &World, id: Option<EntityId>) {
        match id {
            Some(EntityId::Item(i)) => self.write_item(w, Some(i)),
            Some(EntityId::Mobile(m)) => self.write_mobile(w, Some(m)),
            None => self.write_serial(Serial::MINUS_ONE),
        }
    }

    /// `Write<T>(ICollection<T>)` of items: count, then each reference (deleted ones as -1).
    fn write_item_list(&mut self, w: &World, list: &[ItemId]) {
        self.write_i32(list.len() as i32);
        for id in list {
            self.write_item(w, Some(*id));
        }
    }

    /// `Write<T>(ICollection<T>)` of mobiles.
    fn write_mobile_list(&mut self, w: &World, list: &[MobileId]) {
        self.write_i32(list.len() as i32);
        for id in list {
            self.write_mobile(w, Some(*id));
        }
    }

    /// `Write(Map)`: the map's id, or 0xFF for none.
    fn write_map(&mut self, map: Option<MapId>) {
        self.write_u8(map.map_or(0xFF, |m| m.0));
    }

    /// `Write(Race)`.
    fn write_race(&mut self, race: Option<RaceId>) {
        self.write_u8(race.map_or(0xFF, |r| r.0));
    }

    /// `Write(Type)`: 0 for null, else flag 2 and the XXH3 hash of the full name.
    fn write_type(&mut self, full_name: Option<&str>) {
        match full_name {
            None => self.write_u8(0),
            Some(n) => {
                self.write_u8(2);
                self.write_u64(muo_primitives::hash::compute_hash64(n));
            }
        }
    }

    /// `Write(Poison)`: a presence flag then the level.
    fn write_poison(&mut self, p: Option<PoisonId>) {
        match p {
            None => self.write_bool(false),
            Some(p) => {
                self.write_bool(true);
                self.write_u8(p.0);
            }
        }
    }
}

impl<W: GenericWriter + ?Sized> WriteEntityExt for W {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::item::ITEM_BASE;
    use crate::mobile::MOBILE_BASE;
    use muo_io::{SaveReader, SaveWriter};

    #[test]
    fn references_round_trip_and_deleted_write_minus_one() {
        let mut w = World::new();
        let item = w.new_item(&ITEM_BASE);
        let gone = w.new_item(&ITEM_BASE);
        let mobile = w.new_mobile(&MOBILE_BASE);
        w.item_mut(gone).unwrap().flags |= crate::item::ImplFlag::DELETED;

        let mut out = SaveWriter::new(true);
        out.write_item(&w, Some(item));
        out.write_item(&w, Some(gone));
        out.write_item(&w, None);
        out.write_mobile(&w, Some(mobile));
        out.write_item_list(&w, &[item, gone]);
        out.write_map(None);
        out.write_map(Some(MapId(1)));
        out.write_type(Some("Server.Item"));

        let bytes = out.into_vec();
        let mut r = SaveReader::new(&bytes);
        assert_eq!(r.read_item(&w).unwrap(), Some(item));
        assert_eq!(
            r.read_item(&w).unwrap(),
            None,
            "a deleted item is written as -1"
        );
        assert_eq!(r.read_item(&w).unwrap(), None);
        assert_eq!(r.read_mobile(&w).unwrap(), Some(mobile));
        assert_eq!(
            r.read_item_list(&w).unwrap(),
            vec![item],
            "a -1 entry is dropped"
        );
        assert_eq!(r.read_map().unwrap(), None);
        assert_eq!(r.read_map().unwrap(), Some(MapId(1)));
        assert!(matches!(
            ReadEntityExt::read_type(&mut r, &w).unwrap(),
            Some(EntityType::Item(_))
        ));
    }

    #[test]
    fn kept_lists_hold_a_null_entry() {
        let mut w = World::new();
        let item = w.new_item(&ITEM_BASE);
        let mut out = SaveWriter::new(true);
        out.write_encoded_int(3);
        out.write_item(&w, Some(item));
        out.write_item(&w, None);
        out.write_serial(Serial::new(0x4000_0123)); // an item that was never loaded
        let bytes = out.into_vec();
        let list = SaveReader::new(&bytes).read_item_list_kept(true).unwrap();
        assert_eq!(list.len(), 3);
        let mut again = SaveWriter::new(true);
        again.write_encoded_int(3);
        again.write_item_list(&w, &list);
        let again = again.into_vec();
        // count, then serial, -1, -1: the unresolved entry is written back as -1
        assert_eq!(&again[again.len() - 4..], &(-1i32).to_le_bytes());
    }
}
