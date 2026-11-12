//! `EthicsSystem : GenericEntityPersistence<EthicsEntity>("Ethics", 3, 0x1, 0x7FFFFFFF)` and the generated
//! serializers of `EthicsEntity`, `Ethic`, `HeroEthic`, `EvilEthic` and `Player`.
//!
//! Files are `Ethics/Ethics.{idx,bin}`, laid out as `AccountsPersistence` lays out its own: the idx names each
//! record's type (`Server.Ethics.Hero.HeroEthic`, `Server.Ethics.Evil.EvilEthic`, `Server.Ethics.Player`).
//! Versions read: `Player` 0 (obsolete layout, delta time), 1 (migrated), 2; `Ethic` 0 (with its embedded players)
//! and 1.

use std::panic::{AssertUnwindSafe, catch_unwind};

use muo_io::serialization::index::{EntityFileWriter, IdxTypeRef, TypeTable, anchored_time_shift};
use muo_io::serialization::persistence::{bin_path, load_indexes};
use muo_io::{GenericReader, GenericWriter, SaveReader, SaveWriter, SeekOrigin};
use muo_primitives::hash::compute_hash64;
use muo_primitives::{DateTime, Serial};
use muo_server::World;
use muo_server::persistence::{LoadContext, PersistError, Persistence, Snapshot};
use muo_server::serialization_ext::{ReadEntityExt, WriteEntityExt};
use muo_server::timer;

use super::{Entity, EthicData, EthicKind, PlayerData, check_attach, st, st_mut};

const PLAYER: &str = "Server.Ethics.Player";

fn kind_of_name(name: &str) -> Option<Option<EthicKind>> {
    match name {
        "Server.Ethics.Hero.HeroEthic" => Some(Some(EthicKind::Hero)),
        "Server.Ethics.Evil.EvilEthic" => Some(Some(EthicKind::Evil)),
        PLAYER => Some(None),
        _ => None,
    }
}

/// `ReadEntity<Ethic>()`: the serial, found in the Ethics table (none when it is not there).
fn entity_serial(w: &World, r: &mut dyn GenericReader) -> muo_io::Result<Option<Serial>> {
    let s = r.read_serial()?;
    Ok(st(w).and_then(|st| st.ethic(s)).map(|_| s))
}

/// `Player.Deserialize(reader)` after the record's `EthicsEntity` version: `version` is the Player's own.
fn read_player(w: &World, r: &mut dyn GenericReader, p: &mut PlayerData) -> muo_io::Result<()> {
    let version = r.read_encoded_int()?;
    match version {
        1 => {
            // V1Content, migrated
            p.mobile = r.read_mobile(w)?;
            p.power = r.read_i32()?;
            p.history = r.read_i32()?;
            p.steed = r.read_mobile(w)?;
            p.familiar = r.read_mobile(w)?;
            p.shield = DateTime::from_ticks(r.read_delta_time_ticks(DateTime::utc_now().ticks())?);
            p.ethic = entity_serial(w, r)?;
        }
        v if v < 2 => {
            // the obsolete layout: no ethic here (the owner sets it)
            p.mobile = r.read_mobile(w)?;
            p.power = r.read_encoded_int()?;
            p.history = r.read_encoded_int()?;
            p.steed = r.read_mobile(w)?;
            p.familiar = r.read_mobile(w)?;
            p.shield = DateTime::from_ticks(r.read_delta_time_ticks(DateTime::utc_now().ticks())?);
        }
        _ => {
            p.mobile = r.read_mobile(w)?;
            p.power = r.read_i32()?;
            p.history = r.read_i32()?;
            p.steed = r.read_mobile(w)?;
            p.familiar = r.read_mobile(w)?;
            p.shield = DateTime::from_ticks(r.read_anchored_time_ticks()?);
            p.ethic = entity_serial(w, r)?;
        }
    }
    Ok(())
}

fn write_player(w: &World, out: &mut dyn GenericWriter, p: &PlayerData) {
    out.write_encoded_int(0); // EthicsEntity
    out.write_encoded_int(2); // Player
    out.write_mobile(w, p.mobile);
    out.write_i32(p.power);
    out.write_i32(p.history);
    out.write_mobile(w, p.steed);
    out.write_mobile(w, p.familiar);
    out.write_anchored_time_ticks(p.shield.ticks());
    out.write_serial(p.ethic.unwrap_or(Serial::MINUS_ONE));
}

/// `Ethic.Deserialize` (+ `EthicsEntity`, `HeroEthic` / `EvilEthic`), returning the players a version 0 record
/// carries.
pub(super) fn read_ethic(
    w: &mut World,
    r: &mut dyn GenericReader,
    this: Serial,
    kind: EthicKind,
) -> muo_io::Result<()> {
    let _entity_version = r.read_encoded_int()?;
    let version = r.read_encoded_int()?;
    if version < 1 {
        let count = r.read_encoded_int()?;
        for _ in 0..count {
            // `new Player(this, null)` registers a new entity, then `pl.Deserialize(reader)`.
            let serial = st_mut(w).new_serial();
            let mut p = PlayerData::new(this, None, w.now);
            let _entity = r.read_encoded_int()?;
            read_player(w, r, &mut p)?;
            p.ethic = Some(this);
            let has_mobile = p.mobile.is_some();
            st_mut(w).entities.insert(serial, Entity::Player(p));
            if has_mobile {
                timer::start(
                    w,
                    muo_primitives::TimeSpan::ZERO,
                    muo_primitives::TimeSpan::ZERO,
                    1,
                    move |w| {
                        check_attach(w, serial);
                    },
                );
            }
        }
    }
    let _version = r.read_i32()?; // HeroEthic / EvilEthic
    let _ = kind;
    Ok(())
}

struct Span {
    serial: Serial,
    position: i64,
    length: i32,
}

#[derive(Default)]
pub struct EthicsPersistence {
    spans: Vec<Span>,
    anchored_shift: i64,
    estimated: u64,
}

pub fn register(w: &mut World) {
    w.persistence
        .register(3, Box::new(EthicsPersistence::default()));
}

impl Persistence for EthicsPersistence {
    fn name(&self) -> &str {
        "Ethics"
    }

    fn deserialize_indexes(
        &mut self,
        w: &mut World,
        ctx: &mut LoadContext<'_>,
    ) -> Result<(), PersistError> {
        self.spans.clear();
        for (_, idx) in load_indexes(ctx.save_path, "Ethics")? {
            if idx.version >= 5 {
                self.anchored_shift = anchored_time_shift(idx.anchor_ticks, w.now.ticks());
            }
            for r in &idx.records {
                let name = match r.ty {
                    IdxTypeRef::Table(i) => idx.type_table.get(usize::from(i)).cloned(),
                    IdxTypeRef::Hash(h) => [
                        "Server.Ethics.Hero.HeroEthic",
                        "Server.Ethics.Evil.EvilEthic",
                        PLAYER,
                    ]
                    .into_iter()
                    .find(|n| compute_hash64(n) == h)
                    .map(str::to_owned),
                    IdxTypeRef::Legacy(_) => None,
                };
                let Some(kind) = name.as_deref().and_then(kind_of_name) else {
                    continue;
                };
                let created = r.created_ticks.map_or(w.now, DateTime::from_ticks);
                let entity = match kind {
                    Some(kind) => Entity::Ethic(EthicData {
                        kind,
                        created,
                        players: Vec::new(),
                    }),
                    None => Entity::Player(PlayerData {
                        created,
                        mobile: None,
                        power: 0,
                        history: 0,
                        steed: None,
                        familiar: None,
                        shield: DateTime::MIN,
                        ethic: None,
                    }),
                };
                let s = st_mut(w);
                s.last_serial = s.last_serial.max(r.serial.value());
                s.entities.insert(r.serial, entity);
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
        let Ok(bytes) = std::fs::read(bin_path(ctx.save_path, "Ethics")) else {
            for s in spans {
                st_mut(w).entities.shift_remove(&s.serial);
            }
            return Ok(());
        };
        self.estimated = bytes.len() as u64;
        let mut reader = SaveReader::new(&bytes)
            .with_types_db(ctx.types_db)
            .with_anchored_time_shift_ticks(self.anchored_shift);
        let mut delete_all = false;
        let mut to_delete = Vec::new();
        for span in &spans {
            if span.length == 0 {
                to_delete.push(span.serial);
                continue;
            }
            let result = reader
                .seek(span.position, SeekOrigin::Begin)
                .map_err(|e| e.to_string())
                .and_then(|_| {
                    let outcome = catch_unwind(AssertUnwindSafe(|| {
                        read_entity(w, &mut reader, span.serial)
                    }));
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
                    Ok(())
                });
            if let Err(error) = result {
                ctx.prompt.write_line(&format!(
                    "***** Bad deserialize of EthicsEntity ({}) *****",
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
                to_delete.push(span.serial);
            }
        }
        for s in to_delete {
            st_mut(w).entities.shift_remove(&s);
        }
        Ok(())
    }

    fn serialize(&mut self, w: &World) -> Result<Snapshot, PersistError> {
        let mut types = TypeTable::new();
        let mut bytes = SaveWriter::new(true);
        let mut records = Vec::new();
        if let Some(s) = st(w) {
            for (serial, e) in &s.entities {
                let (name, created) = match e {
                    Entity::Ethic(e) => (e.kind.type_name(), e.created),
                    Entity::Player(p) => (PLAYER, p.created),
                };
                let index = types.register(name)?;
                let start = bytes.position() as usize;
                match e {
                    Entity::Ethic(_) => {
                        bytes.write_encoded_int(0); // EthicsEntity
                        bytes.write_encoded_int(1); // Ethic
                        bytes.write_i32(0); // HeroEthic / EvilEthic
                    }
                    Entity::Player(p) => write_player(w, &mut bytes, p),
                }
                records.push((
                    index,
                    *serial,
                    created.ticks(),
                    start,
                    bytes.position() as usize,
                ));
            }
        }
        let mut file = EntityFileWriter::new(w.save_start_time.ticks(), types.names(), &[]);
        let data = bytes.as_slice();
        for (index, serial, created, start, end) in records {
            file.add_record(index, serial, created, &data[start..end])?;
        }
        let (idx, bin) = file.finish();
        self.estimated = bin.len() as u64;
        Ok(Snapshot {
            name: "Ethics".to_owned(),
            idx: Some(idx),
            bin,
        })
    }

    fn estimated_size(&self) -> u64 {
        self.estimated
    }
}

/// One record: the entity is already in the table with its type (created from the idx).
fn read_entity(w: &mut World, r: &mut SaveReader<'_>, serial: Serial) -> muo_io::Result<()> {
    let kind = match st(w).and_then(|s| s.entities.get(&serial)) {
        Some(Entity::Ethic(e)) => Some(e.kind),
        _ => None,
    };
    match kind {
        Some(kind) => read_ethic(w, r, serial, kind),
        None => {
            let _entity_version = r.read_encoded_int()?;
            let mut p = PlayerData::new(serial, None, w.now);
            p.ethic = None;
            if let Some(Entity::Player(old)) = st(w).and_then(|s| s.entities.get(&serial)) {
                p.created = old.created;
            }
            read_player(w, r, &mut p)?;
            st_mut(w).entities.insert(serial, Entity::Player(p));
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use muo_io::{SaveReader, SaveWriter};

    #[test]
    fn pvp_ethics_player_record_golden_bytes() {
        let w = World::new();
        let mut p = PlayerData::new(Serial::new(1), None, DateTime::MIN);
        p.power = 7;
        let mut out = SaveWriter::new(true);
        write_player(&w, &mut out, &p);
        let b = out.as_slice();
        // EthicsEntity 0, Player 2, mobile -1, Power 7, History 5, steed -1, familiar -1, shield MinValue, ethic 1
        let mut want = vec![0u8, 2];
        want.extend_from_slice(&u32::MAX.to_le_bytes());
        want.extend_from_slice(&7i32.to_le_bytes());
        want.extend_from_slice(&5i32.to_le_bytes());
        want.extend_from_slice(&u32::MAX.to_le_bytes());
        want.extend_from_slice(&u32::MAX.to_le_bytes());
        want.extend_from_slice(&DateTime::MIN.ticks().to_le_bytes());
        want.extend_from_slice(&1u32.to_le_bytes());
        assert_eq!(b, &want[..]);
        let mut r = SaveReader::new(b);
        let _entity = r.read_encoded_int().unwrap();
        let mut w = w;
        st_mut(&mut w).entities.insert(
            Serial::new(1),
            Entity::Ethic(EthicData {
                kind: EthicKind::Hero,
                created: DateTime::MIN,
                players: Vec::new(),
            }),
        );
        let mut back = PlayerData::new(Serial::new(9), None, DateTime::MIN);
        read_player(&w, &mut r, &mut back).unwrap();
        assert_eq!(
            (back.power, back.history, back.ethic),
            (7, 5, Some(Serial::new(1)))
        );
    }

    #[test]
    fn pvp_ethics_json_free_names() {
        assert_eq!(kind_of_name("Server.Ethics.Player"), Some(None));
        assert_eq!(
            kind_of_name("Server.Ethics.Hero.HeroEthic"),
            Some(Some(EthicKind::Hero))
        );
        assert!(kind_of_name("Server.Ethics.Nope").is_none());
    }
}
