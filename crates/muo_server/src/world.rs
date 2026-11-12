//! All game state; passed as `&mut World` through the game loop.
//!
//! This file holds the tables and the entity add/remove/find rules (`AddEntity`, `RemoveEntity`, `FindEntity`
//! and their state machine); `world_save.rs` holds `Load`, `Save`, `Snapshot` and the staged-save publishing.

use indexmap::IndexMap;
use muo_data::configuration::{ServerConfiguration, ServerSettings};
use muo_io::SaveWorker;
use muo_io::serialization::persistence::PersistenceRegistry;
use muo_primitives::serial::{ITEM_OFFSET, MAX_ITEM_SERIAL, MAX_MOBILE_SERIAL, MAX_VIRTUAL_SERIAL};
use muo_primitives::{BuiltInRng, DateTime, Serial};

use crate::core_loop::CoreLoop;
use crate::entity_persistence::{EntityPersistence, EntityState, defers};
use crate::events::Events;
use crate::ids::{EntityId, ItemId, MobileId};
use crate::item::{Item, ItemType};
use crate::mobile::{Mobile, MobileType};
use crate::net_state::{NetOut, NetStates};
use crate::persistence::Persistence;
use crate::resources::Resources;
use crate::timer::TimerWheel;
use crate::type_registry::TypeRegistry;

/// `World.ResetVirtualSerial`: virtual serials count up from `MaxItemSerial`.
pub const RESET_VIRTUAL_SERIAL: u32 = MAX_ITEM_SERIAL;

/// `WorldState`.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum WorldState {
    Initial,
    Loading,
    Running,
    PendingSave,
    Saving,
    WritingSave,
}

pub struct World {
    pub items: IndexMap<ItemId, Item>,
    pub mobiles: IndexMap<MobileId, Mobile>,
    /// `Core.Now`, set once per tick.
    pub now: DateTime,
    /// `Core.TickCount`, set once per tick. Compare by wrapping subtraction only.
    pub tick_count: i64,
    pub rng: BuiltInRng,
    pub resources: Resources,
    /// The last serial `next_item_serial` handed out (`GenericEntityPersistence._lastEntitySerial` of items).
    /// After a load it is the highest serial in the save.
    pub last_item_serial: Serial,
    /// Same for mobiles.
    pub last_mobile_serial: Serial,
    /// `Timer`'s static wheel. See `timer.rs`.
    pub timers: TimerWheel,
    /// `EventSink`. See `events.rs`.
    pub events: Events,
    /// `Core`'s loop statics. See `core_loop.rs`.
    pub core: CoreLoop,
    /// `Map.Maps`, `TileData`, `MultiData`, `Region.Regions`. See `map/`.
    pub maps: crate::map::Maps,
    /// `World.WorldState`.
    pub state: WorldState,
    /// `state` as `WorldState as u8`, readable from other threads. Written
    /// only by [`World::set_state`].
    pub state_mirror: std::sync::Arc<std::sync::atomic::AtomicU8>,
    /// `World.SavePath`, absolute.
    pub save_path: std::path::PathBuf,
    /// `World._tempSavePath`, absolute.
    pub temp_save_path: std::path::PathBuf,
    /// `World.SaveStartTime`: when the current or latest save began; written into idx files.
    pub save_start_time: DateTime,
    /// `World.LoadTimeShift` in ticks.
    pub load_time_shift_ticks: i64,
    /// `World._nextVirtualSerial`.
    next_virtual_serial: u32,
    /// What replaces `AssemblyHandler`: classes and phase functions the content crate registers.
    pub types: TypeRegistry,
    /// `Persistence._registry`.
    pub persistence: PersistenceRegistry<Box<dyn Persistence>>,
    /// `GenericEntityPersistence<Item>`'s type table and pending queues.
    pub item_state: EntityState,
    /// Same for mobiles.
    pub mobile_state: EntityState,
    /// `ServerConfiguration`.
    pub config: ServerConfiguration,
    /// Writes snapshots (`ThreadPool.QueueUserWorkItem(WriteFiles)`); `wait_for_write_completion` joins it.
    pub save_worker: SaveWorker,
    /// Entity-aware mirror of each connection. See `net_state.rs`.
    pub net_states: NetStates,
    /// Packets and disconnects the game produced, in order, for `net_glue::pump`.
    pub net_out: Vec<NetOut>,
    /// Live `Target`s by TargetID. See `target.rs`.
    pub targets: IndexMap<crate::target::TargetId, crate::target::Target>,
    /// Live `Prompt`s by serial. See `prompt.rs`.
    pub prompts: IndexMap<crate::prompt::PromptId, crate::prompt::Prompt>,
    /// The serial counters of targets, menus, prompts and hue pickers.
    pub ui_serials: crate::target::UiSerials,
    /// Live `SecureTrade`s by id (`NetState.Trades` lists ids). See `secure_trade.rs`.
    pub trades: IndexMap<crate::net_state::TradeId, crate::secure_trade::SecureTrade>,
    pub next_trade_id: u32,
    /// `MovementThrottle`'s statics (config and the connections with queued movements).
    pub movement_throttle: muo_net::movement::MovementThrottle,
    /// Serials whose `OnAfterDelete` is running: `World.RemoveEntity` has already run, so serial lookups and
    /// enumerations don't see them, while the handler still reads the detached object through its id.
    pub detaching: Vec<Serial>,
}

impl World {
    /// `World.WorldState = value`, visible to other threads through [`World::state_mirror`].
    pub fn set_state(&mut self, state: WorldState) {
        self.state = state;
        self.state_mirror
            .store(state as u8, std::sync::atomic::Ordering::SeqCst);
    }

    /// An empty, running world with its clock at tick 0 (tests advance `tick_count` themselves). Items and mobiles
    /// persist as `Items` and `Mobiles`; the content crate adds `Accounts` and `Guilds`.
    pub fn new() -> World {
        let mut w = World::new_unloaded();
        w.set_state(WorldState::Running);
        w
    }

    /// As [`World::new`] but in `WorldState::Initial`: what `muo` starts with, before `World::load`.
    pub fn new_unloaded() -> World {
        let mut w = World::bare();
        w.persistence
            .register(1, Box::new(EntityPersistence::mobiles()));
        w.persistence
            .register(2, Box::new(EntityPersistence::items()));
        // `Guilds` is registered by the guild class's crate (muo_content::misc::guild), as `Accounts` is.
        w
    }

    fn bare() -> World {
        World {
            items: IndexMap::new(),
            mobiles: IndexMap::new(),
            now: DateTime::MIN,
            tick_count: 0,
            rng: BuiltInRng::new(),
            resources: Resources::default(),
            last_item_serial: Serial::new(ITEM_OFFSET - 1),
            last_mobile_serial: Serial::ZERO,
            timers: TimerWheel::default(),
            events: Events::default(),
            core: CoreLoop::default(),
            maps: crate::map::Maps::default(),
            state: WorldState::Initial,
            state_mirror: std::sync::Arc::new(std::sync::atomic::AtomicU8::new(
                WorldState::Initial as u8,
            )),
            save_path: std::path::PathBuf::from("Saves"),
            temp_save_path: std::path::PathBuf::from("temp"),
            save_start_time: DateTime::MIN,
            load_time_shift_ticks: 0,
            next_virtual_serial: RESET_VIRTUAL_SERIAL,
            types: TypeRegistry::new(),
            persistence: PersistenceRegistry::new(),
            item_state: EntityState::default(),
            mobile_state: EntityState::default(),
            config: ServerConfiguration::in_memory(ServerSettings::default()),
            save_worker: SaveWorker::new(),
            net_states: NetStates::default(),
            net_out: Vec::new(),
            targets: IndexMap::new(),
            prompts: IndexMap::new(),
            ui_serials: Default::default(),
            trades: IndexMap::new(),
            next_trade_id: 1,
            movement_throttle: Default::default(),
            detaching: Vec::new(),
        }
    }
}

impl World {
    /// The same as [`World::new`]: an empty world with every table at its initial value.
    pub fn new_with_defaults() -> World {
        World::new()
    }

    pub fn item(&self, id: ItemId) -> Option<&Item> {
        self.items.get(&id)
    }

    pub fn item_mut(&mut self, id: ItemId) -> Option<&mut Item> {
        self.items.get_mut(&id)
    }

    pub fn mobile(&self, id: MobileId) -> Option<&Mobile> {
        self.mobiles.get(&id)
    }

    pub fn mobile_mut(&mut self, id: MobileId) -> Option<&mut Mobile> {
        self.mobiles.get_mut(&id)
    }

    /// `World.NewItem`: the next free item serial after the last one handed out, wrapping at `MaxItemSerial`.
    /// `Serial::MINUS_ONE` when none is left (an out-of-memory error is logged).
    pub fn next_item_serial(&mut self) -> Serial {
        let items = &self.items;
        let next = next_free_serial(self.last_item_serial, ITEM_OFFSET, MAX_ITEM_SERIAL, |s| {
            items.contains_key(&ItemId(s))
        });
        if next != Serial::MINUS_ONE {
            self.last_item_serial = next;
        }
        next
    }

    /// `World.NewMobile`: as [`World::next_item_serial`] over `1..=MaxMobileSerial`.
    pub fn next_mobile_serial(&mut self) -> Serial {
        let mobiles = &self.mobiles;
        let next = next_free_serial(self.last_mobile_serial, 1, MAX_MOBILE_SERIAL, |s| {
            mobiles.contains_key(&MobileId(s))
        });
        if next != Serial::MINUS_ONE {
            self.last_mobile_serial = next;
        }
        next
    }

    /// `World.Running`.
    pub fn running(&self) -> bool {
        !matches!(self.state, WorldState::Loading | WorldState::Initial)
    }

    /// `World.Loading`.
    pub fn loading(&self) -> bool {
        self.state == WorldState::Loading
    }

    /// `World.Saving`.
    pub fn saving(&self) -> bool {
        self.state == WorldState::Saving
    }

    /// `World.NewVirtual`: virtual things don't persist across saves.
    pub fn new_virtual_serial(&mut self) -> Serial {
        let value = if self.next_virtual_serial > MAX_VIRTUAL_SERIAL {
            RESET_VIRTUAL_SERIAL
        } else {
            self.next_virtual_serial
        };
        self.next_virtual_serial = value + 1;
        Serial::new(value)
    }

    /// `World.FindItem(serial)`. The table keeps entities that are queued for deletion while the world loads or
    /// saves (they stay in the main table), so those are found too.
    pub fn find_item(&self, serial: Serial) -> Option<ItemId> {
        if self.state == WorldState::Initial {
            return None;
        }
        let id = ItemId(serial);
        (self.items.contains_key(&id) && !self.detaching.contains(&serial)).then_some(id)
    }

    /// `World.FindMobile(serial)`.
    pub fn find_mobile(&self, serial: Serial) -> Option<MobileId> {
        if self.state == WorldState::Initial {
            return None;
        }
        let id = MobileId(serial);
        (self.mobiles.contains_key(&id) && !self.detaching.contains(&serial)).then_some(id)
    }

    /// `World.FindEntity(serial)`: an item for an item serial, else a mobile.
    pub fn find_entity(&self, serial: Serial) -> Option<EntityId> {
        if serial.is_item() {
            self.find_item(serial).map(EntityId::Item)
        } else {
            self.find_mobile(serial).map(EntityId::Mobile)
        }
    }

    /// `World.AddEntity(item)`. While the world loads or saves the item is recorded in the pending-add queue
    /// (it can be found at once, but `ProcessSafetyQueues` completes the add); `Running` registers its class in
    /// the idx type table and refuses a serial that is taken.
    pub fn add_item(&mut self, item: Item) -> ItemId {
        let id = item.id();
        let serial = item.serial;
        match self.state {
            WorldState::Initial => panic!(
                "Added {} before world load.",
                short_type_name(item.ty.full_name)
            ),
            s if defers(s) => {
                if s == WorldState::Saving {
                    append_safety_log("add", item.ty.full_name, serial);
                }
                if self.item_state.pending_delete.shift_remove(&serial) {
                    tracing::warn!(
                        "Deleted then added {} during {:?} state.",
                        short_type_name(item.ty.full_name),
                        s
                    );
                }
                if self.items.contains_key(&id) && !self.item_state.pending_add.contains(&serial) {
                    tracing::error!(
                        "Attempted to add '{}' ({serial}) but the serial is taken.",
                        item.ty.full_name
                    );
                    return id;
                }
                self.item_state.pending_add.insert(serial);
                self.items.insert(id, item);
            }
            _ => {
                // Running, PendingSave
                if self
                    .item_state
                    .type_table
                    .register(item.ty.full_name)
                    .is_err()
                {
                    tracing::error!("Items exceeded 65536 distinct entity types.");
                }
                match self.items.get(&id) {
                    Some(existing) if std::ptr::eq(existing.ty, item.ty) => tracing::error!(
                        "Attempted to add '{}' ({serial}) but it already exists in the collection.",
                        item.ty.full_name
                    ),
                    Some(existing) => tracing::error!(
                        "Attempted to add '{}' ({serial}) but found '{}' ({}).",
                        item.ty.full_name,
                        existing.ty.full_name,
                        existing.serial
                    ),
                    None => {
                        self.items.insert(id, item);
                    }
                }
            }
        }
        id
    }

    /// `World.AddEntity(mobile)`. See [`World::add_item`].
    pub fn add_mobile(&mut self, mobile: Mobile) -> MobileId {
        let id = mobile.id();
        let serial = mobile.serial;
        match self.state {
            WorldState::Initial => panic!(
                "Added {} before world load.",
                short_type_name(mobile.ty.full_name)
            ),
            s if defers(s) => {
                if s == WorldState::Saving {
                    append_safety_log("add", mobile.ty.full_name, serial);
                }
                if self.mobile_state.pending_delete.shift_remove(&serial) {
                    tracing::warn!(
                        "Deleted then added {} during {:?} state.",
                        short_type_name(mobile.ty.full_name),
                        s
                    );
                }
                if self.mobiles.contains_key(&id)
                    && !self.mobile_state.pending_add.contains(&serial)
                {
                    tracing::error!(
                        "Attempted to add '{}' ({serial}) but the serial is taken.",
                        mobile.ty.full_name
                    );
                    return id;
                }
                self.mobile_state.pending_add.insert(serial);
                self.mobiles.insert(id, mobile);
            }
            _ => {
                if self
                    .mobile_state
                    .type_table
                    .register(mobile.ty.full_name)
                    .is_err()
                {
                    tracing::error!("Mobiles exceeded 65536 distinct entity types.");
                }
                match self.mobiles.get(&id) {
                    Some(existing) if std::ptr::eq(existing.ty, mobile.ty) => tracing::error!(
                        "Attempted to add '{}' ({serial}) but it already exists in the collection.",
                        mobile.ty.full_name
                    ),
                    Some(existing) => tracing::error!(
                        "Attempted to add '{}' ({serial}) but found '{}' ({}).",
                        mobile.ty.full_name,
                        existing.ty.full_name,
                        existing.serial
                    ),
                    None => {
                        self.mobiles.insert(id, mobile);
                    }
                }
            }
        }
        id
    }

    /// `World.RemoveEntity(item)`. While the world loads or saves the removal is queued and the item stays
    /// findable until `ProcessSafetyQueues`.
    pub fn remove_item(&mut self, id: ItemId) {
        let serial = id.0;
        match self.state {
            WorldState::Initial => panic!("Removed {} before world load.", self.item_name(id)),
            s if defers(s) => {
                if s == WorldState::Saving {
                    append_safety_log("delete", &self.item_name(id), serial);
                }
                if self.item_state.pending_add.shift_remove(&serial) {
                    self.items.swap_remove(&id);
                }
                self.item_state.pending_delete.insert(serial);
            }
            _ => {
                self.items.swap_remove(&id);
            }
        }
    }

    /// `World.RemoveEntity(mobile)`. See [`World::remove_item`].
    pub fn remove_mobile(&mut self, id: MobileId) {
        let serial = id.0;
        match self.state {
            WorldState::Initial => panic!("Removed {} before world load.", self.mobile_name(id)),
            s if defers(s) => {
                if s == WorldState::Saving {
                    append_safety_log("delete", &self.mobile_name(id), serial);
                }
                if self.mobile_state.pending_add.shift_remove(&serial) {
                    self.mobiles.swap_remove(&id);
                }
                self.mobile_state.pending_delete.insert(serial);
            }
            _ => {
                self.mobiles.swap_remove(&id);
            }
        }
    }

    fn item_name(&self, id: ItemId) -> String {
        self.items.get(&id).map_or_else(
            || "Item".to_owned(),
            |i| short_type_name(i.ty.full_name).to_owned(),
        )
    }

    fn mobile_name(&self, id: MobileId) -> String {
        self.mobiles.get(&id).map_or_else(
            || "Mobile".to_owned(),
            |m| short_type_name(m.ty.full_name).to_owned(),
        )
    }

    /// The constructor `new Item()` through `[Constructible]` of class `ty`: a fresh serial, the defaults
    /// stamped with `Core.Now`, added to the world.
    pub fn new_item(&mut self, ty: &'static ItemType) -> ItemId {
        let serial = self.next_item_serial();
        let item = Item::new_at(serial, ty, self.now);
        self.add_item(item)
    }

    /// `new Mobile()` of class `ty`, added to the world.
    pub fn new_mobile(&mut self, ty: &'static MobileType) -> MobileId {
        let serial = self.next_mobile_serial();
        let mut mobile = Mobile::new_at(serial, ty, self.now, self.tick_count);
        mobile.region = crate::region::default_region(self, Some(crate::map::MapId::INTERNAL));
        self.add_mobile(mobile)
    }
}

/// `Type.Name` of a full name.
fn short_type_name(full_name: &str) -> &str {
    let s = full_name.rsplit('.').next().unwrap_or(full_name);
    s.rsplit('+').next().unwrap_or(s)
}

/// `AppendSafetyLog`: adding or deleting during the freeze of a save is a script bug worth a log line.
fn append_safety_log(action: &str, entity: &str, serial: Serial) {
    use std::io::Write;
    let message = format!(
        "Warning: Attempted to {action} {entity} ({serial}) during world save.\nThis action could cause inconsistent state.\nIt is strongly advised that the offending scripts be corrected."
    );
    tracing::info!("{message}");
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("world-save-errors.log")
    {
        let _ = writeln!(f, "{}\t{message}\n", DateTime::utc_now().ticks());
    }
}

/// `GenericEntityPersistence.NewEntity`: scan up from `last`, wrapping from `max` to `min`, for a serial `taken`
/// doesn't know; at most `max` probes.
fn next_free_serial(last: Serial, min: u32, max: u32, taken: impl Fn(Serial) -> bool) -> Serial {
    let mut last = last.value();
    for _ in 0..max {
        last = if last >= max { min } else { last + 1 };
        if !taken(Serial::new(last)) {
            return Serial::new(last);
        }
    }
    Serial::MINUS_ONE
}

impl Default for World {
    fn default() -> Self {
        World::new()
    }
}
