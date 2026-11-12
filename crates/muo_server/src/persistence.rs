//! The entity-aware parts of persistence (`Persistence`, `GenericPersistence`); ad hoc persistence
//! lives in `muo_io::serialization::adhoc` (it needs no entities).
//!
//! A [`Persistence`] is one system's save (accounts, guilds, a content engine, and the two entity tables).
//! Systems register with `w.persistence.register(priority, Box::new(..))`; `Persistence.Load` and the save
//! flow drive them in registry order (priority, then registration). Differences from a threaded save, all for
//! single-threaded composing (see `world_save.rs`):
//!
//! - `Serialize()` + `WriteSnapshot()` become one step, [`Persistence::serialize`], that returns the bytes of the
//!   files ([`Snapshot`]); `SaveWorker` writes them. Nothing is lazy, so nothing needs a heap, worker or chunk.
//! - The console questions of the loaders ("Delete the object and continue? (y/n/a)") go through [`LoadPrompt`].

use std::io::{IsTerminal, Write};
use std::path::Path;

use muo_io::serialization::index::deserialize_exact;
use muo_io::serialization::persistence::{bin_path, write_entity_files, write_generic_file};
use muo_io::serialization::save_reader::{TypesDb, read_file_if_exists};
use muo_io::{GenericReader, GenericWriter, IoError, SaveWriter};

use crate::world::World;

#[derive(Debug, thiserror::Error)]
pub enum PersistError {
    #[error(transparent)]
    Io(#[from] IoError),
    /// A loader's failure: "Deserialization failed.", "Bad type 'X'".
    #[error("{0}")]
    Failed(String),
    /// `HeadlessConsoleInputException`: a question had to be asked and nobody can answer.
    #[error("{0}")]
    Headless(String),
}

/// `ConsoleInputHandler.ReadLine` and `Console.Write` as the loaders use them.
pub trait LoadPrompt {
    /// `Console.Write` (no newline added).
    fn write(&mut self, text: &str);
    /// `Console.WriteLine`.
    fn write_line(&mut self, text: &str) {
        self.write(text);
        self.write("\n");
    }
    /// `ConsoleInputHandler.ReadLine()`. `Err` is `HeadlessConsoleInputException`.
    fn read_line(&mut self) -> Result<String, PersistError>;
}

/// stdin/stdout. Reading when stdin is not a terminal fails like `Core.Headless`.
pub struct StdioPrompt {
    pub headless: bool,
}

impl StdioPrompt {
    pub fn new() -> Self {
        Self {
            headless: !std::io::stdin().is_terminal(),
        }
    }
}

impl Default for StdioPrompt {
    fn default() -> Self {
        Self::new()
    }
}

impl LoadPrompt for StdioPrompt {
    fn write(&mut self, text: &str) {
        print!("{text}");
        let _ = std::io::stdout().flush();
    }

    fn read_line(&mut self) -> Result<String, PersistError> {
        if self.headless {
            return Err(PersistError::Headless(
                "Console input is unavailable in headless mode: ConsoleInputHandler.ReadLine"
                    .into(),
            ));
        }
        let mut s = String::new();
        std::io::stdin().read_line(&mut s).map_err(IoError::from)?;
        Ok(s.trim_end_matches(['\r', '\n']).to_owned())
    }
}

/// Answers from a list (tests, scripted boots); everything written is kept in `output`. An empty list is headless.
#[derive(Default)]
pub struct ScriptedPrompt {
    pub answers: std::collections::VecDeque<String>,
    pub output: String,
}

impl ScriptedPrompt {
    pub fn answering(answers: &[&str]) -> Self {
        Self {
            answers: answers.iter().map(|s| (*s).to_owned()).collect(),
            output: String::new(),
        }
    }
}

impl LoadPrompt for ScriptedPrompt {
    fn write(&mut self, text: &str) {
        self.output.push_str(text);
    }

    fn read_line(&mut self) -> Result<String, PersistError> {
        self.answers.pop_front().ok_or_else(|| {
            PersistError::Headless(
                "Console input is unavailable in headless mode: ConsoleInputHandler.ReadLine"
                    .into(),
            )
        })
    }
}

/// What `Persistence.Load` hands every persistence.
pub struct LoadContext<'a> {
    pub save_path: &'a Path,
    /// `SerializedTypes.db`.
    pub types_db: &'a TypesDb,
    pub prompt: &'a mut dyn LoadPrompt,
}

/// The files one persistence writes for a save: `<save>/<name>/<name>.{idx,bin}`.
#[derive(Debug, Default)]
pub struct Snapshot {
    pub name: String,
    /// `Some` for an entity persistence.
    pub idx: Option<Vec<u8>>,
    pub bin: Vec<u8>,
}

impl Snapshot {
    /// `WriteSnapshot(savePath)`.
    pub fn write(&self, save_path: &Path) -> muo_io::Result<()> {
        match &self.idx {
            Some(idx) => write_entity_files(save_path, &self.name, idx, &self.bin),
            None => write_generic_file(save_path, &self.name, &self.bin),
        }
    }
}

/// `Persistence`. The registry owns each as a `Box<dyn Persistence>` (`World::persistence`).
pub trait Persistence {
    fn name(&self) -> &str;

    /// `IGenericEntityPersistence.DeserializeIndexes`: entity persistences read their idx and create the
    /// (still empty) entities. Called for every persistence, in order, before any `deserialize`.
    fn deserialize_indexes(
        &mut self,
        _w: &mut World,
        _ctx: &mut LoadContext<'_>,
    ) -> Result<(), PersistError> {
        Ok(())
    }

    /// `Deserialize(savePath, typesDb)`.
    fn deserialize(&mut self, w: &mut World, ctx: &mut LoadContext<'_>)
    -> Result<(), PersistError>;

    /// `Serialize()` + `WriteSnapshot`: the bytes of this system's files, composed now.
    fn serialize(&mut self, w: &World) -> Result<Snapshot, PersistError>;

    /// `PostWorldSave`.
    fn post_world_save(&mut self, _w: &mut World) {}

    /// `PostDeserialize`.
    fn post_deserialize(&mut self, _w: &mut World) {}

    /// `GenericPersistence.EstimatedSize`: orders `SerializeAll`, biggest first.
    fn estimated_size(&self) -> u64 {
        0
    }
}

/// `GenericPersistence.Serialize(IGenericWriter)` / `Deserialize(IGenericReader)`: a system with a single payload.
pub trait GenericPayload {
    fn name(&self) -> &str;
    fn serialize(&mut self, w: &World, writer: &mut dyn GenericWriter);
    fn deserialize(&mut self, w: &mut World, reader: &mut dyn GenericReader) -> muo_io::Result<()>;
    fn post_deserialize(&mut self, _w: &mut World) {}
    fn post_world_save(&mut self, _w: &mut World) {}
}

/// `GenericPersistence` around a payload: one `<name>/<name>.bin`, nothing written when the payload is empty.
pub struct GenericPersistence<T: GenericPayload> {
    pub payload: T,
    /// Previous save's payload size, or the loaded file's before the first save (scheduling only).
    estimated: u64,
}

impl<T: GenericPayload> GenericPersistence<T> {
    pub fn new(payload: T) -> Self {
        Self {
            payload,
            estimated: 0,
        }
    }
}

impl<T: GenericPayload> Persistence for GenericPersistence<T> {
    fn name(&self) -> &str {
        self.payload.name()
    }

    fn deserialize(
        &mut self,
        w: &mut World,
        ctx: &mut LoadContext<'_>,
    ) -> Result<(), PersistError> {
        let name = self.payload.name().to_owned();
        let file = bin_path(ctx.save_path, &name);
        let Some(bytes) = read_file_if_exists(&file)?.filter(|b| !b.is_empty()) else {
            return Ok(());
        };
        self.estimated = bytes.len() as u64;
        // These payloads carry no anchor of their own; they inherit the save-wide shift stamped while the entity
        // indexes were read (indexes always load before persistence payloads).
        let shift = w.load_time_shift_ticks;
        let result = deserialize_exact(&bytes, Some(ctx.types_db), shift, |r| {
            self.payload.deserialize(w, r)
        });
        if let Err(e) = result {
            ctx.prompt.write_line(&format!(
                "***** Bad deserialize of {} *****",
                file.display()
            ));
            ctx.prompt.write_line(&e.to_string());
            ctx.prompt.write("Skip this file and continue? (y/n): ");
            if !ctx.prompt.read_line()?.eq_ignore_ascii_case("y") {
                return Err(PersistError::Failed("Deserialization failed.".into()));
            }
        }
        Ok(())
    }

    fn serialize(&mut self, w: &World) -> Result<Snapshot, PersistError> {
        let mut writer = SaveWriter::new(true);
        self.payload.serialize(w, &mut writer);
        let bin = writer.into_vec();
        self.estimated = bin.len() as u64;
        Ok(Snapshot {
            name: self.payload.name().to_owned(),
            idx: None,
            bin,
        })
    }

    fn post_world_save(&mut self, w: &mut World) {
        self.payload.post_world_save(w);
    }

    fn post_deserialize(&mut self, w: &mut World) {
        self.payload.post_deserialize(w);
    }

    fn estimated_size(&self) -> u64 {
        self.estimated
    }
}

/// A system with no reader yet (guilds): keeps its files exactly as loaded and writes them back, so
/// saving with this server does not destroy data it cannot read yet.
pub struct RawPersistence {
    name: String,
    idx: Option<Vec<u8>>,
    bin: Vec<u8>,
}

impl RawPersistence {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            idx: None,
            bin: Vec::new(),
        }
    }
}

impl Persistence for RawPersistence {
    fn name(&self) -> &str {
        &self.name
    }

    fn deserialize(
        &mut self,
        _w: &mut World,
        ctx: &mut LoadContext<'_>,
    ) -> Result<(), PersistError> {
        use muo_io::serialization::persistence::entity_idx_path;
        self.idx = read_file_if_exists(&entity_idx_path(ctx.save_path, &self.name))?;
        self.bin = read_file_if_exists(&bin_path(ctx.save_path, &self.name))?.unwrap_or_default();
        Ok(())
    }

    fn serialize(&mut self, _w: &World) -> Result<Snapshot, PersistError> {
        Ok(Snapshot {
            name: self.name.clone(),
            idx: self.idx.clone(),
            bin: self.bin.clone(),
        })
    }

    fn estimated_size(&self) -> u64 {
        self.bin.len() as u64
    }
}
