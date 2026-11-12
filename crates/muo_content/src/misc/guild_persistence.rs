//! Guild persistence: `GenericEntityPersistence<BaseGuild>("Guilds", 3, 1, 0x7FFFFFFF)`.
//!
//! Same layout as `accounting::persistence`: `Guilds/Guilds.{idx,bin}`, the idx naming `Server.Guilds.Guild`, the bin
//! holding one `Guild.Serialize` per record. Guilds are created (empty) from the idx before any mobile's body is read,
//! so `ReadEntity<BaseGuild>` finds them.
//!
//! shortcut: guilds created while a save is being written are simply part of the next one (the snapshot is composed in
//! one step on the game thread).

use std::panic::{AssertUnwindSafe, catch_unwind};

use muo_io::serialization::index::{EntityFileWriter, IdxTypeRef, TypeTable, anchored_time_shift};
use muo_io::serialization::persistence::{bin_path, load_indexes};
use muo_io::{GenericReader, GenericWriter, SaveReader, SaveWriter, SeekOrigin};
use muo_primitives::{DateTime, Serial};
use muo_server::World;
use muo_server::guild::{self, GuildId};
use muo_server::persistence::{LoadContext, PersistError, Persistence, Snapshot};

use super::guild_class::{TYPE_NAME, blank, deserialize_guild, schedule_verify, serialize};

struct Span {
    serial: Serial,
    position: i64,
    length: i32,
}

#[derive(Default)]
pub struct GuildsPersistence {
    spans: Vec<Span>,
    anchored_shift: i64,
    to_delete: Vec<Serial>,
    estimated: u64,
}

impl GuildsPersistence {
    /// Registers the persistence (priority 3, as `World`'s).
    pub fn register(w: &mut World) {
        w.persistence
            .register(3, Box::new(GuildsPersistence::default()));
    }
}

impl Persistence for GuildsPersistence {
    fn name(&self) -> &str {
        "Guilds"
    }

    fn deserialize_indexes(
        &mut self,
        w: &mut World,
        ctx: &mut LoadContext<'_>,
    ) -> Result<(), PersistError> {
        self.spans.clear();
        let now = DateTime::utc_now();
        let wanted_hash = muo_primitives::hash::compute_hash64(TYPE_NAME);
        for (_, idx) in load_indexes(ctx.save_path, "Guilds")? {
            if idx.version >= 5 {
                self.anchored_shift = anchored_time_shift(idx.anchor_ticks, w.now.ticks());
            }
            for r in &idx.records {
                let is_guild = match r.ty {
                    IdxTypeRef::Table(i) => idx
                        .type_table
                        .get(usize::from(i))
                        .is_some_and(|n| n == TYPE_NAME),
                    IdxTypeRef::Hash(h) => h == wanted_hash,
                    IdxTypeRef::Legacy(_) => false,
                };
                if !is_guild {
                    continue;
                }
                let created = r.created_ticks.map_or(now, DateTime::from_ticks);
                guild::add_guild(w, blank(r.serial, created));
                self.spans.push(Span {
                    serial: r.serial,
                    position: r.position,
                    length: r.length,
                });
            }
        }
        Ok(())
    }

    fn deserialize(
        &mut self,
        w: &mut World,
        ctx: &mut LoadContext<'_>,
    ) -> Result<(), PersistError> {
        let spans = std::mem::take(&mut self.spans);
        let bytes = match std::fs::read(bin_path(ctx.save_path, "Guilds")) {
            Ok(b) => b,
            Err(_) => {
                for s in spans {
                    guild::remove_guild(w, GuildId(s.serial.value()));
                }
                return Ok(());
            }
        };
        self.estimated = bytes.len() as u64;
        let mut reader = SaveReader::new(&bytes)
            .with_types_db(ctx.types_db)
            .with_anchored_time_shift_ticks(self.anchored_shift);
        let mut delete_all = false;
        for span in &spans {
            if span.length == 0 {
                self.to_delete.push(span.serial);
                continue;
            }
            let id = GuildId(span.serial.value());
            let result = reader
                .seek(span.position, SeekOrigin::Begin)
                .map_err(|e| e.to_string())
                .and_then(|_| {
                    match catch_unwind(AssertUnwindSafe(|| deserialize_guild(w, id, &mut reader))) {
                        Ok(Ok(())) => {}
                        Ok(Err(e)) => return Err(e.to_string()),
                        Err(_) => return Err("panic".into()),
                    }
                    let done = reader.position() - span.position as u64;
                    if done != span.length as u64 {
                        return Err(format!(
                            "Serialized object was {} bytes, but {done} bytes deserialized",
                            span.length
                        ));
                    }
                    Ok(())
                });
            if let Err(error) = result {
                ctx.prompt.write_line(&format!(
                    "***** Bad deserialize of {TYPE_NAME} ({}) *****",
                    span.serial
                ));
                ctx.prompt.write_line(&error);
                if !delete_all {
                    ctx.prompt
                        .write("Delete the object and continue? (y/n/a): ");
                    let pressed = ctx.prompt.read_line()?;
                    if pressed.eq_ignore_ascii_case("a") {
                        delete_all = true;
                    } else if !pressed.eq_ignore_ascii_case("y") {
                        return Err(PersistError::Failed("Deserialization failed.".into()));
                    }
                }
                self.to_delete.push(span.serial);
            }
        }
        for s in std::mem::take(&mut self.to_delete) {
            guild::remove_guild(w, GuildId(s.value()));
        }
        Ok(())
    }

    fn post_deserialize(&mut self, w: &mut World) {
        schedule_verify(w);
    }

    fn serialize(&mut self, w: &World) -> Result<Snapshot, PersistError> {
        let mut types = TypeTable::new();
        let ids = guild::guild_ids(w);
        if !ids.is_empty() {
            types.register(TYPE_NAME)?;
        }
        let mut bytes = SaveWriter::new(true);
        let mut records = Vec::new();
        for id in ids {
            let Some(b) = guild::guild(w, id) else {
                continue;
            };
            let start = bytes.position() as usize;
            if catch_unwind(AssertUnwindSafe(|| serialize(w, id, &mut bytes))).is_err() {
                return Err(PersistError::Failed(format!(
                    "Error serializing {TYPE_NAME} ({})",
                    b.serial
                )));
            }
            records.push((
                b.serial,
                b.created.ticks(),
                start,
                bytes.position() as usize,
            ));
        }
        let mut file = EntityFileWriter::new(w.save_start_time.ticks(), types.names(), &[]);
        let data = bytes.as_slice();
        for (serial, created, start, end) in records {
            file.add_record(0, serial, created, &data[start..end])?;
        }
        let (idx, bin) = file.finish();
        self.estimated = bin.len() as u64;
        Ok(Snapshot {
            name: "Guilds".to_owned(),
            idx: Some(idx),
            bin,
        })
    }

    fn estimated_size(&self) -> u64 {
        self.estimated
    }
}
