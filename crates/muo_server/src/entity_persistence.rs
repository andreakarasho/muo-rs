//! Generic entity persistence (type tables, index and data files, add/delete queues), with `World.ItemPersistence`
//! and `World.MobilePersistence` (the `PostDeserialize` order) as the two [`EntityKind`]s.
//!
//! The entity tables themselves are `World::items` / `World::mobiles` (`EntitiesBySerial`); what the generic
//! class kept beside them (the type table, the pending add and delete queues) is [`EntityState`], on the `World`
//! because `AddEntity` is reached from everywhere. This struct keeps only the load/save bookkeeping.
//!
//! Load (`DeserializeIndexes`, `Deserialize`, `PostDeserialize`):
//! 1. the idx names each record's type; unknown types ask the [`LoadPrompt`] to delete them all, answering
//!    no throws "Bad type"; known ones create an empty entity with `Item::new_for_load`/`Mobile::new_for_load`
//!    (family state in `Ext` is the type's `deserialize`'s to create), stamped with the idx's creation time;
//! 2. the bin: every record is deserialized at its position and must consume exactly its length; a failure asks
//!    to delete the entity (y), all failures (a), or aborts; zero-length records are entities deleted since the
//!    save; failures and zero-length ones are deleted after the whole bin;
//! 3. the pending queues are applied and the kind's post-load pass runs.
//!
//! shortcut: save composes every record on the game thread (no worker heaps). `SkipSerialization` entities are not
//! serialized at all.

use std::collections::BTreeMap;
use std::panic::{AssertUnwindSafe, catch_unwind};

use indexmap::IndexSet;
use muo_io::GenericWriter;
use muo_io::serialization::index::{
    EntityFileWriter, Idx, IdxTypeRef, TypeTable, anchored_time_shift,
};
use muo_io::serialization::persistence::{bin_path, load_indexes, split_bin_path};
use muo_io::{GenericReader, IoError, SaveReader, SaveWriter, SeekOrigin};
use muo_primitives::hash::compute_hash64;
use muo_primitives::{DateTime, Serial};

use crate::ids::{ItemId, MobileId};
use crate::item::Item;
use crate::mobile::Mobile;
use crate::persistence::{LoadContext, PersistError, Persistence, Snapshot};
use crate::type_registry::EntityType;
use crate::world::{World, WorldState};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum EntityKind {
    Item,
    Mobile,
}

/// What `GenericEntityPersistence<T>` keeps next to the table.
#[derive(Default)]
pub struct EntityState {
    /// `_typeTable`: every entity type added since boot, insertion ordered; idx records name a type by slot.
    pub type_table: TypeTable,
    /// `_pendingAdd`: entities added while the world was loading or saving. They are in the table already (so
    /// `World::item` finds them, as `FindEntity` does) and become ordinary at `ProcessSafetyQueues`.
    pub pending_add: IndexSet<Serial>,
    /// `_pendingDelete`: entities removed meanwhile; still in the table until `ProcessSafetyQueues`.
    pub pending_delete: IndexSet<Serial>,
}

/// `EntitySpan<T>`.
struct EntitySpan {
    serial: Serial,
    position: i64,
    length: i32,
}

pub struct EntityPersistence {
    kind: EntityKind,
    name: &'static str,
    /// `_entities`: spans per idx file (0 for the single idx, `i` for legacy `<Name>_i`).
    spans: BTreeMap<usize, Vec<EntitySpan>>,
    to_delete: Vec<Serial>,
    /// `_anchoredTimeShift`
    anchored_shift: i64,
    estimated: u64,
}

impl EntityPersistence {
    /// `new ItemPersistence()`: `("Items", priority 2)`.
    pub fn items() -> Self {
        Self::new(EntityKind::Item, "Items")
    }

    /// `new MobilePersistence()`: `("Mobiles", priority 1)`.
    pub fn mobiles() -> Self {
        Self::new(EntityKind::Mobile, "Mobiles")
    }

    fn new(kind: EntityKind, name: &'static str) -> Self {
        Self {
            kind,
            name,
            spans: BTreeMap::new(),
            to_delete: Vec::new(),
            anchored_shift: 0,
            estimated: 0,
        }
    }

    fn type_for_name(w: &World, name: &str) -> Option<EntityType> {
        w.types.find_type_by_hash(compute_hash64(name))
    }

    /// `GetConstructorFor`, with the kind filter of `ctor.Invoke(..) is T`. `Ok(None)` means skip the records.
    fn resolve(
        &self,
        w: &mut World,
        ctx: &mut LoadContext<'_>,
        type_name: &str,
        found: Option<EntityType>,
    ) -> Result<Option<EntityType>, PersistError> {
        let Some(t) = found else {
            ctx.prompt.write_line("failed");
            ctx.prompt.write(&format!(
                "Error: Type '{type_name}' was not found. Delete all of those types? (y/n): "
            ));
            if ctx.prompt.read_line()?.eq_ignore_ascii_case("y") {
                ctx.prompt.write_line("Loading...");
                return Ok(None);
            }
            ctx.prompt
                .write_line("Types will not be deleted. An exception will be thrown.");
            return Err(PersistError::Failed(format!("Bad type '{type_name}'")));
        };
        match (self.kind, t) {
            (EntityKind::Item, EntityType::Item(ty)) => {
                // Keep the type table complete so the next save can index it.
                w.item_state.type_table.register(ty.full_name)?;
            }
            (EntityKind::Mobile, EntityType::Mobile(ty)) => {
                w.mobile_state.type_table.register(ty.full_name)?;
            }
            _ => return Ok(None),
        }
        Ok(Some(t))
    }

    fn create(&self, w: &mut World, t: EntityType, serial: Serial, created: DateTime) {
        match t {
            EntityType::Item(ty) => {
                let mut item = Item::new_for_load(serial, ty);
                item.created = created;
                w.items.insert(ItemId(serial), item);
            }
            EntityType::Mobile(ty) => {
                let mut m = Mobile::new_for_load(serial, ty, w.tick_count);
                m.created = created;
                w.mobiles.insert(MobileId(serial), m);
            }
        }
    }

    fn internal_deserialize_indexes(
        &mut self,
        w: &mut World,
        ctx: &mut LoadContext<'_>,
        idx: &Idx,
    ) -> Result<Vec<EntitySpan>, PersistError> {
        if idx.version >= 5 {
            // Re-base anchored timestamps by the elapsed time since the save started. The whole save shares one
            // anchor: publish it so payloads without their own (generic bins) shift too.
            self.anchored_shift = anchored_time_shift(idx.anchor_ticks, w.now.ticks());
            w.load_time_shift_ticks = self.anchored_shift;
        }

        // Types by idx type-table slot (v4+) or by hash / legacy index (v0..v3), resolved on first use.
        let mut table_types: Vec<Option<EntityType>> = Vec::new();
        if idx.version >= 4 {
            for name in &idx.type_table {
                let found = Self::type_for_name(w, name);
                table_types.push(self.resolve(w, ctx, name, found)?);
            }
        }
        // `ReadTypes` joins the folder with `Name/Name.tdb` twice, so the legacy .tdb of a v0/v1 index is
        // never found and its records are skipped. Same here.
        let mut by_hash: BTreeMap<u64, Option<EntityType>> = BTreeMap::new();

        let now = DateTime::utc_now();
        let mut spans = Vec::with_capacity(idx.records.len());
        for r in &idx.records {
            let t = match r.ty {
                IdxTypeRef::Table(i) => table_types.get(usize::from(i)).copied().flatten(),
                IdxTypeRef::Hash(hash) => match by_hash.get(&hash) {
                    Some(t) => *t,
                    None => {
                        let t = match ctx.types_db.get(&hash) {
                            Some(name) => {
                                let found = w.types.find_type_by_hash(hash);
                                self.resolve(w, ctx, name, found)?
                            }
                            None => None,
                        };
                        by_hash.insert(hash, t);
                        t
                    }
                },
                IdxTypeRef::Legacy(_) => None,
            };
            let Some(t) = t else { continue };
            let created = r.created_ticks.map_or(now, DateTime::from_ticks);
            self.create(w, t, r.serial, created);
            spans.push(EntitySpan {
                serial: r.serial,
                position: r.position,
                length: r.length,
            });
        }

        let last = match self.kind {
            EntityKind::Item => w.items.keys().map(|k| k.0).max(),
            EntityKind::Mobile => w.mobiles.keys().map(|k| k.0).max(),
        };
        if let Some(last) = last {
            match self.kind {
                EntityKind::Item => w.last_item_serial = last,
                EntityKind::Mobile => w.last_mobile_serial = last,
            }
        }
        Ok(spans)
    }

    fn exists(&self, w: &World, serial: Serial) -> bool {
        match self.kind {
            EntityKind::Item => w.items.contains_key(&ItemId(serial)),
            EntityKind::Mobile => w.mobiles.contains_key(&MobileId(serial)),
        }
    }

    fn type_name(&self, w: &World, serial: Serial) -> &'static str {
        match self.kind {
            EntityKind::Item => w.items.get(&ItemId(serial)).map_or("?", |i| i.ty.full_name),
            EntityKind::Mobile => w
                .mobiles
                .get(&MobileId(serial))
                .map_or("?", |m| m.ty.full_name),
        }
    }

    /// `entity.Delete()`
    fn delete(&self, w: &mut World, serial: Serial) {
        match self.kind {
            EntityKind::Item => {
                if let Some(ty) = w.items.get(&ItemId(serial)).map(|i| i.ty) {
                    (ty.delete)(w, ItemId(serial));
                }
            }
            EntityKind::Mobile => {
                if let Some(ty) = w.mobiles.get(&MobileId(serial)).map(|m| m.ty) {
                    (ty.delete)(w, MobileId(serial));
                }
            }
        }
    }

    /// `entity.Deserialize(reader)`; a panic in a content deserializer is a load failure.
    fn deserialize_entity(
        &self,
        w: &mut World,
        serial: Serial,
        r: &mut SaveReader<'_>,
    ) -> Result<(), String> {
        let outcome = catch_unwind(AssertUnwindSafe(|| match self.kind {
            EntityKind::Item => {
                let ty = w.items.get(&ItemId(serial)).map(|i| i.ty);
                ty.map_or(Ok(()), |ty| (ty.deserialize)(w, ItemId(serial), r))
            }
            EntityKind::Mobile => {
                let ty = w.mobiles.get(&MobileId(serial)).map(|m| m.ty);
                ty.map_or(Ok(()), |ty| (ty.deserialize)(w, MobileId(serial), r))
            }
        }));
        match outcome {
            Ok(Ok(())) => Ok(()),
            Ok(Err(e)) => Err(e.to_string()),
            Err(p) => Err(p
                .downcast_ref::<&str>()
                .map(|s| (*s).to_owned())
                .or_else(|| p.downcast_ref::<String>().cloned())
                .unwrap_or_else(|| "panic".into())),
        }
    }

    fn internal_deserialize(
        &mut self,
        w: &mut World,
        ctx: &mut LoadContext<'_>,
        file: &std::path::Path,
        key: usize,
    ) -> Result<(), PersistError> {
        let bytes = std::fs::read(file).map_err(IoError::from)?;
        self.estimated += bytes.len() as u64;
        let mut reader = SaveReader::new(&bytes)
            .with_types_db(ctx.types_db)
            .with_anchored_time_shift_ticks(self.anchored_shift);

        let mut delete_all_failures = false;
        let spans = self.spans.remove(&key).unwrap_or_default();
        for span in &spans {
            if span.length == 0 {
                self.delete(w, span.serial);
                continue;
            }
            if !self.exists(w, span.serial) {
                continue;
            }

            let result = reader
                .seek(span.position, SeekOrigin::Begin)
                .map_err(|e| e.to_string())
                .and_then(|_| {
                    self.deserialize_entity(w, span.serial, &mut reader)
                        .and_then(|()| {
                            let done = reader.position() - span.position as u64;
                            if done == span.length as u64 {
                                Ok(())
                            } else {
                                Err(format!(
                                    "Serialized object was {} bytes, but {done} bytes deserialized",
                                    span.length
                                ))
                            }
                        })
                });

            if let Err(error) = result {
                ctx.prompt.write_line(&format!(
                    "***** Bad deserialize of {} ({}) *****",
                    self.type_name(w, span.serial),
                    span.serial
                ));
                ctx.prompt.write_line(&error);

                if !delete_all_failures {
                    ctx.prompt
                        .write("Delete the object and continue? (y/n/a): ");
                    let pressed = ctx.prompt.read_line()?;
                    if pressed.eq_ignore_ascii_case("a") {
                        delete_all_failures = true;
                    } else if !pressed.eq_ignore_ascii_case("y") {
                        return Err(PersistError::Failed("Deserialization failed.".into()));
                    }
                }
                self.to_delete.push(span.serial);
            }
        }
        Ok(())
    }
}

impl Persistence for EntityPersistence {
    fn name(&self) -> &str {
        self.name
    }

    fn deserialize_indexes(
        &mut self,
        w: &mut World,
        ctx: &mut LoadContext<'_>,
    ) -> Result<(), PersistError> {
        self.spans.clear();
        for (i, idx) in load_indexes(ctx.save_path, self.name)? {
            let spans = self.internal_deserialize_indexes(w, ctx, &idx)?;
            self.spans.insert(i, spans);
        }
        Ok(())
    }

    fn deserialize(
        &mut self,
        w: &mut World,
        ctx: &mut LoadContext<'_>,
    ) -> Result<(), PersistError> {
        let data = bin_path(ctx.save_path, self.name);
        match std::fs::metadata(&data) {
            Err(_) => {
                // Legacy split files.
                let keys: Vec<usize> = self.spans.keys().copied().collect();
                for i in keys {
                    let path = split_bin_path(ctx.save_path, self.name, i);
                    self.internal_deserialize(w, ctx, &path, i)?;
                }
            }
            Ok(m) if m.len() > 0 => self.internal_deserialize(w, ctx, &data, 0)?,
            Ok(_) => {}
        }
        self.spans.clear();

        for serial in std::mem::take(&mut self.to_delete) {
            self.delete(w, serial);
        }
        Ok(())
    }

    fn serialize(&mut self, w: &World) -> Result<Snapshot, PersistError> {
        let (state, serials): (&EntityState, Vec<Serial>) = match self.kind {
            EntityKind::Item => (&w.item_state, w.items.keys().map(|k| k.0).collect()),
            EntityKind::Mobile => (&w.mobile_state, w.mobiles.keys().map(|k| k.0).collect()),
        };

        let mut bytes = SaveWriter::new(true);
        // (type index, serial, created ticks, start, end)
        let mut records = Vec::with_capacity(serials.len());
        for serial in serials {
            if state.pending_add.contains(&serial) {
                continue;
            }
            let (full_name, created, skip) = match self.kind {
                EntityKind::Item => {
                    let Some(i) = w.items.get(&ItemId(serial)) else {
                        continue;
                    };
                    (
                        i.ty.full_name,
                        i.created,
                        (i.ty.skip_serialization)(w, i.id()),
                    )
                }
                EntityKind::Mobile => {
                    let Some(m) = w.mobiles.get(&MobileId(serial)) else {
                        continue;
                    };
                    (
                        m.ty.full_name,
                        m.created,
                        (m.ty.skip_serialization)(w, m.id()),
                    )
                }
            };
            if skip {
                continue;
            }
            // Every path into the table registers the type first, so this cannot fire.
            let type_index = state.type_table.index_of(full_name).ok_or_else(|| {
                PersistError::Failed(format!(
                    "{full_name} was serialized but never registered; entities must enter {} through AddEntity.",
                    self.name
                ))
            })?;

            let start = bytes.position() as usize;
            let run = catch_unwind(AssertUnwindSafe(|| match self.kind {
                EntityKind::Item => {
                    let ty = w.items[&ItemId(serial)].ty;
                    (ty.serialize)(w, ItemId(serial), &mut bytes)
                }
                EntityKind::Mobile => {
                    let ty = w.mobiles[&MobileId(serial)].ty;
                    (ty.serialize)(w, MobileId(serial), &mut bytes)
                }
            }));
            if run.is_err() {
                return Err(PersistError::Failed(format!(
                    "Error serializing {full_name} ({serial})"
                )));
            }
            records.push((
                type_index,
                serial,
                created.ticks(),
                start,
                bytes.position() as usize,
            ));
        }

        let mut file =
            EntityFileWriter::new(w.save_start_time.ticks(), state.type_table.names(), &[]);
        let data = bytes.as_slice();
        for (type_index, serial, created, start, end) in records {
            file.add_record(type_index, serial, created, &data[start..end])?;
        }
        let (idx, bin) = file.finish();
        self.estimated = bin.len() as u64;
        Ok(Snapshot {
            name: self.name.to_owned(),
            idx: Some(idx),
            bin,
        })
    }

    fn post_world_save(&mut self, w: &mut World) {
        process_safety_queues(w, self.kind);
    }

    fn post_deserialize(&mut self, w: &mut World) {
        process_safety_queues(w, self.kind);

        // `ItemPersistence.PostDeserialize` / `MobilePersistence.PostDeserialize`, in table order.
        match self.kind {
            EntityKind::Item => {
                let ids: Vec<ItemId> = w.items.keys().copied().collect();
                for id in ids {
                    let Some(item) = w.items.get(&id) else {
                        continue;
                    };
                    let (ty, parentless) = (item.ty, item.parent.is_none());
                    if parentless {
                        (ty.update_totals)(w, id);
                    }
                    crate::item::base::clear_properties(w, id);
                    crate::item::ops::update_decay_registration(w, id);
                }
            }
            EntityKind::Mobile => {
                let ids: Vec<MobileId> = w.mobiles.keys().copied().collect();
                for id in ids {
                    let Some(ty) = w.mobiles.get(&id).map(|m| m.ty) else {
                        continue;
                    };
                    crate::mobile::part2::effects::update_region(w, id);
                    (ty.update_totals)(w, id);
                    crate::mobile::part2::properties::clear_properties(w, id);
                }
            }
        }
    }

    fn estimated_size(&self) -> u64 {
        self.estimated
    }
}

/// `ProcessSafetyQueues`: adds then deletes recorded while the world was loading or saving take effect.
pub fn process_safety_queues(w: &mut World, kind: EntityKind) {
    let (adds, deletes) = match kind {
        EntityKind::Item => (
            std::mem::take(&mut w.item_state.pending_add),
            std::mem::take(&mut w.item_state.pending_delete),
        ),
        EntityKind::Mobile => (
            std::mem::take(&mut w.mobile_state.pending_add),
            std::mem::take(&mut w.mobile_state.pending_delete),
        ),
    };
    // Adds only register their type here: the entity has been in the table since it was created.
    for serial in adds {
        match kind {
            EntityKind::Item => {
                if let Some(ty) = w.items.get(&ItemId(serial)).map(|i| i.ty) {
                    let _ = w.item_state.type_table.register(ty.full_name);
                }
            }
            EntityKind::Mobile => {
                if let Some(ty) = w.mobiles.get(&MobileId(serial)).map(|m| m.ty) {
                    let _ = w.mobile_state.type_table.register(ty.full_name);
                }
            }
        }
    }
    for serial in deletes {
        match kind {
            EntityKind::Item => {
                w.items.swap_remove(&ItemId(serial));
            }
            EntityKind::Mobile => {
                w.mobiles.swap_remove(&MobileId(serial));
            }
        }
    }
}

/// Whether adds and removes are deferred (`Loading`, `Saving`, `WritingSave`).
pub(crate) fn defers(state: WorldState) -> bool {
    matches!(
        state,
        WorldState::Loading | WorldState::Saving | WorldState::WritingSave
    )
}
