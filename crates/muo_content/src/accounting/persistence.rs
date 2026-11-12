//! Account persistence: `GenericEntityPersistence<IAccount>("Accounts", 3, 0x1, 0x7FFFFFFF)`.
//!
//! The files are `Accounts/Accounts.{idx,bin}`, laid out as every entity persistence's (see
//! `muo_server::entity_persistence`, which this follows record for record): the idx names the type
//! `Server.Accounting.Account`, the bin holds one `Account.Serialize` per record.
//!
//! shortcut: accounts created while a save is being written are simply part of the next one (the snapshot is
//! composed in one step on the game thread, so no pending queue is needed).

use std::panic::{AssertUnwindSafe, catch_unwind};

use muo_io::serialization::index::{EntityFileWriter, IdxTypeRef, TypeTable, anchored_time_shift};
use muo_io::serialization::persistence::{bin_path, load_indexes};
use muo_io::{GenericReader, GenericWriter, SaveReader, SaveWriter, SeekOrigin};
use muo_primitives::{DateTime, Serial};
use muo_server::World;
use muo_server::persistence::{LoadContext, PersistError, Persistence, Snapshot};

use super::{Account, Accounts, accounts, schedule_after_deserialization};
use muo_server::iaccount::AccountId;

pub const TYPE_NAME: &str = "Server.Accounting.Account";

struct Span {
    serial: Serial,
    position: i64,
    length: i32,
}

#[derive(Default)]
pub struct AccountsPersistence {
    spans: Vec<Span>,
    anchored_shift: i64,
    to_delete: Vec<Serial>,
    estimated: u64,
}

impl AccountsPersistence {
    /// `Accounts.Configure`: registers the persistence (priority 3).
    pub fn register(w: &mut World) {
        w.persistence
            .register(3, Box::new(AccountsPersistence::default()));
    }
}

impl Persistence for AccountsPersistence {
    fn name(&self) -> &str {
        "Accounts"
    }

    fn deserialize_indexes(
        &mut self,
        w: &mut World,
        ctx: &mut LoadContext<'_>,
    ) -> Result<(), PersistError> {
        self.spans.clear();
        // Backward compatibility: an old accounts.xml is loaded instead of the binary files.
        let xml = ctx.save_path.join("Accounts").join("accounts.xml");
        if xml.exists() {
            super::xml::deserialize_xml(w, &xml).map_err(PersistError::Failed)?;
            return Ok(());
        }
        let now = DateTime::utc_now();
        let wanted_hash = muo_primitives::hash::compute_hash64(TYPE_NAME);
        for (_, idx) in load_indexes(ctx.save_path, "Accounts")? {
            if idx.version >= 5 {
                self.anchored_shift = anchored_time_shift(idx.anchor_ticks, w.now.ticks());
            }
            for r in &idx.records {
                let is_account = match r.ty {
                    IdxTypeRef::Table(i) => idx
                        .type_table
                        .get(usize::from(i))
                        .is_some_and(|n| n == TYPE_NAME),
                    IdxTypeRef::Hash(h) => h == wanted_hash,
                    IdxTypeRef::Legacy(_) => false,
                };
                if !is_account {
                    continue;
                }
                let created = r.created_ticks.map_or(now, DateTime::from_ticks);
                let mut a = Account::blank(r.serial, created);
                a.mobiles = vec![None; 7];
                accounts(w).raw_insert(a);
                self.spans.push(Span {
                    serial: r.serial,
                    position: r.position,
                    length: r.length,
                });
            }
        }
        if let Some(last) = accounts(w).raw_serials().map(Serial::value).max() {
            accounts(w).set_last_serial(last);
        }
        Ok(())
    }

    fn deserialize(
        &mut self,
        w: &mut World,
        ctx: &mut LoadContext<'_>,
    ) -> Result<(), PersistError> {
        let file = bin_path(ctx.save_path, "Accounts");
        let spans = std::mem::take(&mut self.spans);
        let bytes = match std::fs::read(&file) {
            Ok(b) => b,
            Err(_) => {
                // No bin: every indexed account is gone.
                for s in spans {
                    accounts(w).remove(AccountId(s.serial.value()));
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
            let result = reader
                .seek(span.position, SeekOrigin::Begin)
                .map_err(|e| e.to_string())
                .and_then(|_| {
                    let mut a = Account::blank(span.serial, w.now);
                    if let Some(old) = accounts(w).get(AccountId(span.serial.value())) {
                        a.created = old.created;
                    }
                    let outcome = catch_unwind(AssertUnwindSafe(|| a.deserialize(w, &mut reader)));
                    match outcome {
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
                    accounts(w).raw_insert(a);
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
            accounts(w).remove(AccountId(s.value()));
        }
        accounts(w).rebuild_names();
        Ok(())
    }

    fn post_deserialize(&mut self, w: &mut World) {
        // Timer.DelayCall(AfterDeserialization) of every account loaded.
        for id in accounts(w).ids() {
            schedule_after_deserialization(w, id);
        }
    }

    fn serialize(&mut self, w: &World) -> Result<Snapshot, PersistError> {
        let empty = Accounts::default();
        let acc = w.resources.get::<Accounts>().unwrap_or(&empty);
        let mut types = TypeTable::new();
        if acc.type_registered {
            types.register(TYPE_NAME)?;
        }
        let mut bytes = SaveWriter::new(true);
        let mut records = Vec::new();
        for a in acc
            .raw_serials()
            .filter_map(|s| acc.get(AccountId(s.value())))
        {
            let start = bytes.position() as usize;
            let run = catch_unwind(AssertUnwindSafe(|| a.serialize(w, &mut bytes)));
            if run.is_err() {
                return Err(PersistError::Failed(format!(
                    "Error serializing {TYPE_NAME} ({})",
                    a.serial
                )));
            }
            records.push((
                a.serial,
                a.created.ticks(),
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
            name: "Accounts".to_owned(),
            idx: Some(idx),
            bin,
        })
    }

    fn estimated_size(&self) -> u64 {
        self.estimated
    }
}
