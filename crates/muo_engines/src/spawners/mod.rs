//! Spawners: `BaseSpawner`, `Spawner`, `RegionSpawner`, `ProximitySpawner`,
//! `SpawnerEntry`, `SpawnPositionState`, `SectorSpawnCache` and the JSON import/export.
//!
//! The three item classes share one state struct, [`SpawnerData`], in `Item::ext` (the classes only differ
//! in a few virtuals, [`Kind`]). A `SpawnerId` is the serial of the spawner item. `Spawned` (the entity ->
//! entry dictionary) keeps insertion order in an `IndexMap`, the value being the entry's index.
//!
//! The gumps (`SpawnerGump`, `SpawnerControllerGump`, `SpawnPropsGump`) are in [`gumps`], the target commands in
//! [`commands`], `SpawnerEntry.Properties` / `.Parameters` in [`spawn_args`] (over the `muo_content::props` tables
//! instead of reflection), the exporter and the `.xml` importer in [`export`].

pub mod cache;
pub mod commands;
pub mod core;
pub mod export;
pub mod gumps;
pub mod json;
pub mod props;
pub mod save;
pub mod spawn_args;

#[cfg(test)]
mod tests;

use indexmap::IndexMap;
use muo_io::TextDefinition;
use muo_io::serialization::Guid;
use muo_primitives::{DateTime, Rectangle3D, TimeSpan};
use muo_server::ids::{EntityId, ItemId};
use muo_server::region::RegionId;
use muo_server::timer::TimerHandle;
use muo_server::world::World;

/// `SpawnPositionMode`.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
#[repr(u8)]
pub enum SpawnPositionMode {
    #[default]
    Automatic = 0,
    Enabled = 1,
    Disabled = 2,
    Abandoned = 3,
}

impl SpawnPositionMode {
    pub fn from_u8(v: u8) -> Self {
        match v {
            1 => Self::Enabled,
            2 => Self::Disabled,
            3 => Self::Abandoned,
            _ => Self::Automatic,
        }
    }
}

bitflags::bitflags! {
    /// `EntryFlags`.
    #[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
    pub struct EntryFlags: i32 {
        const INVALID_TYPE = 0x1;
        const INVALID_PARAMS = 0x2;
        const INVALID_PROPS = 0x4;
        const INVALID_ENTRY = 0x8;
    }
}

/// Which class: `Spawner`, `RegionSpawner` (a `Spawner`) or `ProximitySpawner` (a `Spawner`).
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum Kind {
    #[default]
    Spawner,
    Region,
    Proximity,
}

/// Identity of an entry object (stands in for a `SpawnerEntry` reference): a gump finds its entry again by it.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct EntryUid(u64);

impl Default for EntryUid {
    fn default() -> Self {
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
        Self(NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed))
    }
}

/// `SpawnerEntry`.
#[derive(Clone, Debug, Default)]
pub struct Entry {
    pub uid: EntryUid,
    pub name: Option<String>,
    pub probability: i32,
    pub max_count: i32,
    pub properties: Option<String>,
    pub parameters: Option<String>,
    pub spawned: Vec<EntityId>,
    pub disabled: bool,
    /// `Valid`: the flags of the last spawn attempt.
    pub valid: EntryFlags,
}

impl Entry {
    pub fn new(
        name: Option<&str>,
        probability: i32,
        max_count: i32,
        properties: Option<&str>,
        parameters: Option<&str>,
    ) -> Self {
        Self {
            name: name.map(str::to_owned),
            probability,
            max_count,
            properties: properties.map(str::to_owned),
            parameters: parameters.map(str::to_owned),
            ..Self::default()
        }
    }

    /// `IsFull`.
    pub fn is_full(&self) -> bool {
        self.spawned.len() as i32 >= self.max_count
    }
}

/// `SpawnPositionState`: runtime only.
#[derive(Clone, Debug, Default)]
pub struct PosState {
    pub non_transient_failures: i32,
    pub consecutive_useless: i32,
    pub spiral_ring: i32,
    pub spiral_ring_position: i32,
    pub spiral_complete: bool,
}

const FAILURE_THRESHOLD: i32 = 5;
const ABANDON_THRESHOLD: i32 = 25;

impl PosState {
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    pub fn should_cache_positions(&self, mode: SpawnPositionMode) -> bool {
        mode == SpawnPositionMode::Enabled
            || (mode == SpawnPositionMode::Automatic
                && self.non_transient_failures > FAILURE_THRESHOLD)
    }

    pub fn should_abandon(&self) -> bool {
        self.spiral_complete && self.consecutive_useless >= ABANDON_THRESHOLD
    }
}

pub const DEFAULT_MIN_DELAY: TimeSpan = TimeSpan::from_mins(5);
pub const DEFAULT_MAX_DELAY: TimeSpan = TimeSpan::from_mins(10);
pub const DEFAULT_MAX_SPAWN_ATTEMPTS: i32 = 10;

/// The fields of `BaseSpawner`, `Spawner`, `RegionSpawner` and `ProximitySpawner`.
#[derive(Debug)]
pub struct SpawnerData {
    pub kind: Kind,
    pub guid: Guid,
    pub return_on_deactivate: bool,
    pub walking_range: i32,
    pub way_point: Option<ItemId>,
    pub group: bool,
    pub min_delay: TimeSpan,
    pub max_delay: TimeSpan,
    pub team: i32,
    pub spawn_location_is_home: bool,
    pub end: DateTime,
    pub spawn_position_mode: SpawnPositionMode,
    pub max_spawn_attempts: i32,
    pub count: i32,
    pub running: bool,
    /// `Spawner.SpawnBounds`.
    pub spawn_bounds: Rectangle3D,
    pub use_spiral_scan: bool,
    pub entries: Vec<Entry>,
    /// `Spawned`: entity -> index into `entries`.
    pub spawned: IndexMap<EntityId, usize>,
    pub timer: Option<TimerHandle>,
    pub pos_state: Option<PosState>,
    /// A v10 save's `HomeRange`, turned into bounds once the location is known.
    pub pending_home_range: Option<i32>,
    // RegionSpawner
    pub spawn_region_name: Option<String>,
    pub spawn_region: Option<RegionId>,
    // ProximitySpawner
    pub trigger_range: i32,
    pub spawn_message: Option<TextDefinition>,
    pub instant_flag: bool,
}

impl SpawnerData {
    pub fn new(kind: Kind) -> Self {
        Self {
            kind,
            guid: Guid::default(),
            return_on_deactivate: false,
            walking_range: -1,
            way_point: None,
            group: false,
            min_delay: DEFAULT_MIN_DELAY,
            max_delay: DEFAULT_MAX_DELAY,
            team: 0,
            spawn_location_is_home: false,
            end: DateTime::MIN,
            spawn_position_mode: SpawnPositionMode::Automatic,
            max_spawn_attempts: 0, // the constructor never sets it
            count: 1,
            running: false,
            spawn_bounds: Rectangle3D::default(),
            use_spiral_scan: false,
            entries: Vec::new(),
            spawned: IndexMap::new(),
            timer: None,
            pos_state: None,
            pending_home_range: None,
            spawn_region_name: None,
            spawn_region: None,
            trigger_range: 0,
            spawn_message: None,
            instant_flag: false,
        }
    }

    /// `IsFull`.
    pub fn is_full(&self) -> bool {
        self.spawned.len() as i32 >= self.count
    }

    /// `IsEmpty`.
    pub fn is_empty(&self) -> bool {
        self.spawned.is_empty()
    }
}

pub fn data(w: &World, id: ItemId) -> Option<&SpawnerData> {
    w.item(id)?.ext.get()
}

pub fn data_mut(w: &mut World, id: ItemId) -> Option<&mut SpawnerData> {
    w.item_mut(id)?.ext.get_mut()
}

/// `Serial` of a spawner item as the `ISpawner` handle entities keep.
pub fn spawner_id(id: ItemId) -> muo_server::interfaces::SpawnerId {
    muo_server::interfaces::SpawnerId(id.0.value())
}

/// The item an `ISpawner` handle names.
pub fn item_of(s: muo_server::interfaces::SpawnerId) -> ItemId {
    ItemId(muo_primitives::Serial::new(s.0))
}

/// Registers the spawner classes and the hooks the engine owns.
pub fn register(types: &mut muo_server::type_registry::TypeRegistry) {
    types.register_item(&core::SPAWNER, &[]);
    types.register_item(&core::REGION_SPAWNER, &[]);
    types.register_item(&core::PROXIMITY_SPAWNER, &[]);
    types.register_item(&commands::SPAWNER_BORDER, &[]);
    types.register_phase_default(
        muo_server::type_registry::Phase::Configure,
        "Server.Engines.Spawners.Configure",
        core::configure,
    );
    types.register_phase_default(
        muo_server::type_registry::Phase::Configure,
        "Server.Engines.Spawners.Properties",
        props::configure,
    );
    // `TargetCommands.Register` needs the command implementors, which exist after every Configure
    types.register_phase_default(
        muo_server::type_registry::Phase::Initialize,
        "Server.Engines.Spawners.Commands",
        commands::configure,
    );
}
