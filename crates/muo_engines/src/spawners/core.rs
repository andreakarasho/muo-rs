//! `BaseSpawner` (with its entries, hooks and DTO helpers), `Spawner`, `RegionSpawner` and `ProximitySpawner`: the
//! spawn timer, weighted entry choice, entity creation and spawn positions.

use muo_content::creature::data as cdata;
use muo_content::hooks::rnd;
use muo_content::mx;
use muo_content::regions::base_region;
use muo_primitives::{Point3D, Rectangle3D, TimeSpan};
use muo_server::ext::Ext;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::interfaces::SpawnerId;
use muo_server::item::{ITEM_BASE, ItemType, ops};
use muo_server::map::{MapId, fit};
use muo_server::mobile::AccessLevel;
use muo_server::mobile::part2::deps::SpawnerDeps;
use muo_server::property_list::PropertyList;
use muo_server::timer;
use muo_server::world::World;

use super::cache;
use super::spawn_args;
use super::{
    DEFAULT_MAX_DELAY, DEFAULT_MIN_DELAY, Entry, EntryFlags, Kind, PosState, SpawnPositionMode,
    SpawnerData, data, data_mut, item_of, spawner_id,
};

// ---------------------------------------------------------------------------------------------------------
// classes

/// `Server.Engines.Spawners.Spawner`.
pub static SPAWNER: ItemType = ItemType {
    full_name: "Server.Engines.Spawners.Spawner",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| construct(w, &SPAWNER, Kind::Spawner)),
    default_name: |_, _| Some("Spawner".to_owned()),
    can_see_staff_only: |w, _, from| mx::access_level(w, from) >= AccessLevel::Developer,
    is_accessible_to: |w, _, from| mx::access_level(w, from) >= AccessLevel::Developer,
    get_properties,
    on_single_click,
    on_double_click,
    on_delete,
    on_map_change,
    on_location_change,
    on_after_duped,
    serialize: super::save::serialize,
    deserialize: super::save::deserialize,
    ..ITEM_BASE
};

/// `Server.Engines.Spawners.RegionSpawner`.
pub static REGION_SPAWNER: ItemType = ItemType {
    full_name: "Server.Engines.Spawners.RegionSpawner",
    parent: Some(&SPAWNER),
    construct: Some(|w| construct(w, &REGION_SPAWNER, Kind::Region)),
    serialize: super::save::serialize_region,
    deserialize: super::save::deserialize_region,
    ..SPAWNER
};

/// `Server.Engines.Spawners.ProximitySpawner`.
pub static PROXIMITY_SPAWNER: ItemType = ItemType {
    full_name: "Server.Engines.Spawners.ProximitySpawner",
    parent: Some(&SPAWNER),
    construct: Some(|w| construct(w, &PROXIMITY_SPAWNER, Kind::Proximity)),
    default_name: |_, _| Some("Proximity Spawner".to_owned()),
    handles_on_movement: |w, id| data(w, id).is_some_and(|d| d.running),
    serialize: super::save::serialize_proximity,
    deserialize: super::save::deserialize_proximity,
    on_movement: proximity_on_movement,
    ..SPAWNER
};

/// The `Kind` a class loads as.
pub fn kind_of_type(ty: &'static ItemType) -> Kind {
    if std::ptr::eq(ty, &REGION_SPAWNER) {
        Kind::Region
    } else if std::ptr::eq(ty, &PROXIMITY_SPAWNER) {
        Kind::Proximity
    } else {
        Kind::Spawner
    }
}

/// `Configure` phase: the entity-side hooks.
pub fn configure(w: &mut World) {
    *w.resources.get_mut::<SpawnerDeps>() = SpawnerDeps {
        remove: Some(|w, s, e| remove(w, item_of(s), e)),
        notify_death: Some(notify_spawned_death),
        unlink_on_taming: Some(|_, _| true),
        return_on_deactivate: Some(|w, s| {
            data(w, item_of(s)).is_some_and(|d| d.return_on_deactivate)
        }),
        is_in_spawn_bounds: Some(|w, s, loc| is_in_spawn_bounds(w, item_of(s), loc)),
        get_spawn_position: Some(|w, s, m, map| {
            get_spawn_position(w, item_of(s), Some(EntityId::Mobile(m)), map)
        }),
        walking_range: Some(|w, s| walking_range(w, item_of(s))),
        region_spawner_region: Some(|w, s| {
            data(w, item_of(s))
                .filter(|d| d.kind == Kind::Region)
                .and_then(|d| d.spawn_region)
        }),
        region: Some(|w, s| region(w, item_of(s))),
    };
    // `SectorSpawnCacheManager.InvalidateSectors` for the houses (BaseHouse placed, moved or deleted)
    muo_content::items::house_deps::deps_mut(w).invalidate_spawn_sectors =
        Some(super::cache::invalidate_sectors);
    super::json::register_commands(w);
}

/// `Guid.NewGuid()`: random v4. Not the game generator (`Utility.Random` is not touched for it).
pub fn new_guid() -> muo_io::serialization::Guid {
    use std::hash::{BuildHasher, Hasher};
    let mut b = [0u8; 16];
    for half in 0..2 {
        let mut h = std::collections::hash_map::RandomState::new().build_hasher();
        h.write_u64(half);
        b[half as usize * 8..half as usize * 8 + 8].copy_from_slice(&h.finish().to_le_bytes());
    }
    b[6] = (b[6] & 0x0F) | 0x40;
    b[8] = (b[8] & 0x3F) | 0x80;
    muo_io::serialization::Guid(b)
}

// ---------------------------------------------------------------------------------------------------------
// construction

fn construct(w: &mut World, ty: &'static ItemType, kind: Kind) -> ItemId {
    create(
        w,
        ty,
        kind,
        1,
        DEFAULT_MIN_DELAY,
        DEFAULT_MAX_DELAY,
        0,
        Rectangle3D::default(),
        &[],
    )
}

/// `new Spawner(amount, minDelay, maxDelay, team, spawnBounds, spawnedNames)` (and the other classes').
#[allow(clippy::too_many_arguments)]
pub fn create(
    w: &mut World,
    ty: &'static ItemType,
    kind: Kind,
    amount: i32,
    min_delay: TimeSpan,
    max_delay: TimeSpan,
    team: i32,
    bounds: Rectangle3D,
    names: &[&str],
) -> ItemId {
    let id = ops::new_item_with_id(w, ty, 0x1f13);
    let mut d = SpawnerData::new(kind);
    d.guid = new_guid();
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(d);
    }
    init_spawn(w, id, amount, min_delay, max_delay, team, bounds);
    for n in names {
        add_entry(w, id, Some(n), 100, amount, false, None, None);
    }
    id
}

/// `InitSpawn(amount, minDelay, maxDelay, team, spawnBounds)`.
pub fn init_spawn(
    w: &mut World,
    id: ItemId,
    amount: i32,
    min: TimeSpan,
    max: TimeSpan,
    team: i32,
    bounds: Rectangle3D,
) {
    ops::set_visible(w, id, false);
    ops::set_movable(w, id, false);
    if let Some(d) = data_mut(w, id) {
        d.running = true;
        d.group = false;
        d.min_delay = min;
        d.max_delay = max;
        d.count = amount;
        d.team = team;
    }
    if bounds != Rectangle3D::default() {
        set_spawn_bounds(w, id, bounds);
    } else {
        set_home_range(w, id, 4);
    }
    if let Some(d) = data_mut(w, id) {
        d.entries.clear();
        d.spawned.clear();
    }
    do_timer_delay(w, id, TimeSpan::from_secs(1));
}

// ---------------------------------------------------------------------------------------------------------
// bounds

/// `SpawnBounds` setter.
pub fn set_spawn_bounds(w: &mut World, id: ItemId, bounds: Rectangle3D) {
    if let Some(d) = data_mut(w, id) {
        d.spawn_bounds = bounds;
    }
    ops::invalidate_properties(w, id);
}

/// `HomeRange` getter: the distance from the spawner to the nearest edge of the bounds.
pub fn home_range(w: &World, id: ItemId) -> i32 {
    let (Some(d), Some(i)) = (data(w, id), w.item(id)) else {
        return 0;
    };
    let b = d.spawn_bounds;
    if b == Rectangle3D::default() {
        return 0;
    }
    let loc = i.location;
    let dx_min = (loc.x - b.start.x).abs();
    let dx_max = (b.end.x - loc.x).abs();
    let dy_min = (loc.y - b.start.y).abs();
    let dy_max = (b.end.y - loc.y).abs();
    dx_min.min(dx_max).min(dy_min.min(dy_max))
}

/// `HomeRange` setter: square bounds centered on the spawner.
pub fn set_home_range(w: &mut World, id: ItemId, value: i32) {
    let (Some(d), Some(i)) = (data(w, id), w.item(id)) else {
        return;
    };
    let loc = i.location;
    let (z, depth) = if d.spawn_bounds != Rectangle3D::default() {
        (d.spawn_bounds.start.z, d.spawn_bounds.depth())
    } else if value == 0 {
        (loc.z, 0)
    } else {
        (-128, 256)
    };
    set_spawn_bounds(
        w,
        id,
        Rectangle3D::new(
            loc.x - value,
            loc.y - value,
            z,
            value * 2 + 1,
            value * 2 + 1,
            depth,
        ),
    );
    ops::invalidate_properties(w, id);
}

/// `IsHomeRangeStyleAt(location)`.
pub fn is_home_range_style_at(w: &World, id: ItemId, location: Point3D) -> bool {
    let Some(d) = data(w, id) else { return true };
    let b = d.spawn_bounds;
    if b == Rectangle3D::default() {
        return true;
    }
    if b.width() != b.height() {
        return false;
    }
    b.start.x + b.width() / 2 == location.x && b.start.y + b.height() / 2 == location.y
}

/// `WalkingRange` getter.
pub fn walking_range(w: &World, id: ItemId) -> i32 {
    match data(w, id) {
        Some(d) if d.walking_range > 0 => d.walking_range,
        _ => home_range(w, id),
    }
}

/// `WalkingRange` setter.
pub fn set_walking_range(w: &mut World, id: ItemId, value: i32) {
    if let Some(d) = data_mut(w, id) {
        d.walking_range = value;
    }
    ops::invalidate_properties(w, id);
}

/// `IsInSpawnBounds(location)`; a `RegionSpawner` has the base version too (its bounds stay empty).
pub fn is_in_spawn_bounds(w: &World, id: ItemId, location: Point3D) -> bool {
    data(w, id).is_some_and(|d| {
        d.spawn_bounds == Rectangle3D::default() || d.spawn_bounds.contains(location)
    })
}

/// `BoundsFromHomeRange(location, homeRange)`.
pub fn bounds_from_home_range(location: Point3D, home_range: i32) -> Rectangle3D {
    let (z, depth) = if home_range == 0 {
        (location.z, 0)
    } else {
        (-128, 256)
    };
    Rectangle3D::new(
        location.x - home_range,
        location.y - home_range,
        z,
        home_range * 2 + 1,
        home_range * 2 + 1,
        depth,
    )
}

// ---------------------------------------------------------------------------------------------------------
// entries

fn entry_name(e: &Entry) -> &str {
    e.name.as_deref().unwrap_or("")
}

/// `AddEntry(name, probability, amount, dotimer, properties, parameters)`: the index of the new entry.
#[allow(clippy::too_many_arguments)]
pub fn add_entry(
    w: &mut World,
    id: ItemId,
    name: Option<&str>,
    probability: i32,
    amount: i32,
    do_timer: bool,
    properties: Option<&str>,
    parameters: Option<&str>,
) -> usize {
    let mut idx = 0;
    if let Some(d) = data_mut(w, id) {
        d.entries.push(Entry::new(
            name,
            probability,
            amount,
            properties,
            parameters,
        ));
        idx = d.entries.len() - 1;
    }
    if do_timer {
        do_timer_delay(w, id, TimeSpan::from_secs(1));
    }
    idx
}

/// `RemoveEntry(entry)`: the entry's live spawns are deleted.
pub fn remove_entry(w: &mut World, id: ItemId, index: usize) {
    defrag(w, id);
    let removed = {
        let Some(d) = data_mut(w, id) else { return };
        if index >= d.entries.len() {
            return;
        }
        let e = d.entries.remove(index);
        // later entries shifted down
        for v in d.spawned.values_mut() {
            if *v > index {
                *v -= 1;
            }
        }
        e
    };
    for e in removed.spawned.iter().rev() {
        if let Some(d) = data_mut(w, id) {
            d.spawned.shift_remove(e);
        }
        delete_entity(w, *e);
    }
    restart_timer_if_needed(w, id);
    ops::invalidate_properties(w, id);
}

/// `RemoveAllEntries()`.
pub fn remove_all_entries(w: &mut World, id: ItemId) {
    remove_spawns(w, id);
    if let Some(d) = data_mut(w, id) {
        d.entries.clear();
    }
    ops::invalidate_properties(w, id);
}

/// `CopyEntriesTo(target)`: clones without the live spawns.
pub fn copy_entries_to(w: &mut World, id: ItemId, target: ItemId) {
    if id == target {
        return;
    }
    remove_all_entries(w, target);
    let clones: Vec<Entry> = data(w, id)
        .map(|d| {
            d.entries
                .iter()
                .map(|e| Entry {
                    name: e.name.clone(),
                    probability: e.probability,
                    max_count: e.max_count,
                    properties: e.properties.clone(),
                    parameters: e.parameters.clone(),
                    disabled: e.disabled,
                    ..Entry::default()
                })
                .collect()
        })
        .unwrap_or_default();
    if let Some(t) = data_mut(w, target) {
        t.entries.extend(clones);
    }
    ops::invalidate_properties(w, target);
}

/// `Spawned = new(); for each entry: TryAdd(spawned, entry)`, then `DoTimer(_end - Core.Now)`.
pub fn rebuild_spawned(w: &mut World, id: ItemId) {
    let now = w.now;
    let Some(d) = data_mut(w, id) else { return };
    d.spawned.clear();
    for (i, e) in d.entries.iter().enumerate() {
        for s in &e.spawned {
            d.spawned.entry(*s).or_insert(i);
        }
    }
    let delay = d.end - now;
    do_timer_delay(w, id, delay);
}

// ---------------------------------------------------------------------------------------------------------
// spawned bookkeeping

fn entity_deleted(w: &World, e: EntityId) -> bool {
    match e {
        EntityId::Item(i) => ops::is_deleted(w, i),
        EntityId::Mobile(m) => mx::deleted(w, m),
    }
}

fn delete_entity(w: &mut World, e: EntityId) {
    match e {
        EntityId::Item(i) => {
            if let Some(t) = w.item(i).map(|x| x.ty) {
                (t.delete)(w, i);
            }
        }
        EntityId::Mobile(m) => {
            if let Some(t) = w.mobile(m).map(|x| x.ty) {
                (t.delete)(w, m);
            }
        }
    }
}

fn spawner_of(w: &World, e: EntityId) -> Option<SpawnerId> {
    match e {
        EntityId::Item(i) => w.item(i)?.spawner,
        EntityId::Mobile(m) => w.mobile(m)?.spawner,
    }
}

/// The test of `OnDefragSpawn(spawned, remove: false)` before it unlinks: the spawn no longer belongs.
fn defrag_drops(w: &World, e: EntityId) -> bool {
    entity_deleted(w, e)
        || spawner_of(w, e).is_none()
        || match e {
            EntityId::Item(i) => {
                matches!(ops::root_parent(w, i), Some(EntityId::Mobile(_)))
                    || ops::is_locked_down(w, i)
                    || ops::is_secure(w, i)
            }
            EntityId::Mobile(m) => cdata::cd(w, m).is_some_and(|c| c.controlled || c.is_stabled),
        }
}

/// `OnDefragSpawn(spawned, remove: false)`: true when the spawn was dropped.
fn on_defrag_spawn(w: &mut World, id: ItemId, e: EntityId) -> bool {
    defrag_drops(w, e) && data_mut(w, id).is_some_and(|d| d.spawned.shift_remove(&e).is_some())
}

/// `Defrag()`: each entry drops the spawns that no longer belong to it.
pub fn defrag(w: &mut World, id: ItemId) {
    let n = data(w, id).map_or(0, |d| d.entries.len());
    for ei in 0..n {
        let mut i = 0;
        while let Some(e) = data(w, id)
            .and_then(|d| d.entries.get(ei))
            .and_then(|en| en.spawned.get(i))
            .copied()
        {
            if on_defrag_spawn(w, id, e) {
                if let Some(en) = data_mut(w, id).and_then(|d| d.entries.get_mut(ei)) {
                    en.spawned.remove(i);
                }
            } else {
                i += 1;
            }
        }
    }
}

/// `Remove(spawn)` (`ISpawner.Remove`).
pub fn remove(w: &mut World, id: ItemId, spawn: EntityId) {
    defrag(w, id);
    if let Some(d) = data_mut(w, id)
        && let Some(ei) = d.spawned.shift_remove(&spawn)
        && let Some(en) = d.entries.get_mut(ei)
    {
        en.spawned.retain(|s| *s != spawn);
    }
    restart_timer_if_needed(w, id);
}

fn restart_timer_if_needed(w: &mut World, id: ItemId) {
    let Some(d) = data(w, id) else { return };
    let timer_running = d.timer.is_some_and(|t| t.running(w));
    if d.running && !d.is_full() && !timer_running {
        do_timer(w, id);
    }
}

/// `NotifySpawnedDeath(spawned, killer)`: the hook a subclass overrides (`OnSpawnedDeath`); none in the shipped
/// content.
fn notify_spawned_death(w: &mut World, spawner: SpawnerId, m: MobileId, killer: Option<MobileId>) {
    let _ = (w, spawner, m, killer);
}

/// `CountSpawns(entry)`.
pub fn count_spawns(w: &mut World, id: ItemId, entry: usize) -> usize {
    defrag(w, id);
    data(w, id)
        .and_then(|d| d.entries.get(entry))
        .map_or(0, |e| e.spawned.len())
}

/// `RemoveSpawn(entry)`.
pub fn remove_spawn(w: &mut World, id: ItemId, entry: usize) {
    let mut i = data(w, id)
        .and_then(|d| d.entries.get(entry))
        .map_or(0, |e| e.spawned.len());
    while i > 0 {
        i -= 1;
        let e = {
            let Some(d) = data_mut(w, id) else { return };
            let Some(en) = d.entries.get_mut(entry) else {
                return;
            };
            if i >= en.spawned.len() {
                continue;
            }
            let e = en.spawned.remove(i);
            d.spawned.shift_remove(&e);
            e
        };
        delete_entity(w, e);
    }
}

/// `RemoveSpawns()`.
pub fn remove_spawns(w: &mut World, id: ItemId) {
    defrag(w, id);
    let n = data(w, id).map_or(0, |d| d.entries.len());
    for ei in 0..n {
        let mut j = data(w, id)
            .and_then(|d| d.entries.get(ei))
            .map_or(0, |e| e.spawned.len());
        while j > 0 {
            j -= 1;
            let e = {
                let Some(d) = data_mut(w, id) else { return };
                let Some(en) = d.entries.get_mut(ei) else {
                    break;
                };
                if j >= en.spawned.len() {
                    continue;
                }
                let e = en.spawned.remove(j);
                d.spawned.shift_remove(&e);
                e
            };
            delete_entity(w, e);
        }
    }
    restart_timer_if_needed(w, id);
    ops::invalidate_properties(w, id);
}

/// `BringToHome()`.
pub fn bring_to_home(w: &mut World, id: ItemId) {
    defrag(w, id);
    let (loc, map) = match w.item(id) {
        Some(i) => (i.location, i.map),
        None => return,
    };
    let spawned: Vec<EntityId> = data(w, id)
        .map(|d| d.spawned.keys().copied().collect())
        .unwrap_or_default();
    for e in spawned {
        match (e, map) {
            (EntityId::Item(i), m) => ops::move_to_world_map(w, i, loc, m),
            (EntityId::Mobile(m), Some(map)) => {
                if let Some(t) = w.mobile(m).map(|x| x.ty) {
                    (t.move_to_world)(w, m, loc, map);
                }
            }
            _ => {}
        }
    }
}

// ---------------------------------------------------------------------------------------------------------
// run state and the timer

/// `Running` setter.
pub fn set_running(w: &mut World, id: ItemId, value: bool) {
    if value {
        start(w, id);
    } else {
        stop(w, id);
    }
    ops::invalidate_properties(w, id);
}

/// `BeginStart()`.
fn begin_start(w: &mut World, id: ItemId) -> bool {
    match data_mut(w, id) {
        Some(d) if !d.running && !d.entries.is_empty() => {
            d.running = true;
            true
        }
        _ => false,
    }
}

/// `Start()`.
pub fn start(w: &mut World, id: ItemId) {
    if begin_start(w, id) {
        do_timer(w, id);
    }
}

/// `Stop()`.
pub fn stop(w: &mut World, id: ItemId) {
    let timer = data(w, id).and_then(|d| d.timer);
    if let Some(t) = timer {
        timer::stop(w, t);
    }
    if let Some(d) = data_mut(w, id) {
        d.running = false;
    }
}

/// `NextSpawn` setter.
pub fn set_next_spawn(w: &mut World, id: ItemId, value: TimeSpan) {
    if begin_start(w, id) {
        do_timer_delay(w, id, value);
    }
}

/// `Count` setter (`OnCountChanged`).
pub fn set_count(w: &mut World, id: ItemId, value: i32) {
    let Some(d) = data_mut(w, id) else { return };
    if d.count == value {
        return;
    }
    d.count = value;
    ops::invalidate_properties(w, id);
    let (full, timer) = data(w, id).map_or((false, None), |d| (d.is_full(), d.timer));
    if full {
        if let Some(t) = timer {
            timer::stop(w, t);
        }
    } else if !timer.is_some_and(|t| t.running(w)) {
        do_timer(w, id);
    }
}

/// `DoTimer()`: a random delay between the minimum and maximum.
pub fn do_timer(w: &mut World, id: ItemId) {
    let Some(d) = data(w, id) else { return };
    if !d.running {
        return;
    }
    let (min, max) = (d.min_delay.ticks() / 10_000, d.max_delay.ticks() / 10_000);
    let ms = muo_primitives::utility::random_min_max_i64(&mut w.rng, min, max);
    do_timer_delay(w, id, TimeSpan::from_millis(ms));
}

/// `DoTimer(delay)`. A `ProximitySpawner` only records `End`.
pub fn do_timer_delay(w: &mut World, id: ItemId, delay: TimeSpan) {
    let now = w.now;
    let Some(d) = data_mut(w, id) else { return };
    if !d.running {
        return;
    }
    if d.kind == Kind::Proximity {
        d.end = now + delay;
        return;
    }
    d.end = if delay <= TimeSpan::ZERO {
        now
    } else {
        now + delay
    };
    let delay = delay.max(TimeSpan::ZERO);
    let (existing, full) = (d.timer, d.is_full());
    let handle = match existing {
        Some(h) => {
            timer::stop(w, h);
            timer::set_delay(w, h, delay);
            h
        }
        None => {
            let h = timer::create(w, delay, TimeSpan::ZERO, 1, move |w| {
                if w.item(id).is_some_and(|i| !i.deleted()) {
                    on_tick(w, id);
                }
            });
            if let Some(d) = data_mut(w, id) {
                d.timer = Some(h);
            }
            h
        }
    };
    if !full {
        timer::begin(w, handle);
    }
}

/// `OnTick()`.
pub fn on_tick(w: &mut World, id: ItemId) {
    if data(w, id).is_some_and(|d| d.group) {
        defrag(w, id);
        if data(w, id).is_some_and(|d| !d.spawned.is_empty()) {
            return;
        }
        respawn(w, id);
    } else {
        spawn(w, id);
    }
    do_timer(w, id);
}

/// `Respawn()`.
pub fn respawn(w: &mut World, id: ItemId) {
    remove_spawns(w, id);
    if data(w, id).is_some_and(|d| d.kind == Kind::Proximity) {
        let now = w.now;
        if let Some(d) = data_mut(w, id) {
            d.end = now;
        }
        return;
    }
    for _ in 0..data(w, id).map_or(0, |d| d.count) {
        spawn(w, id);
    }
    do_timer(w, id);
}

/// `Reset()`.
pub fn reset(w: &mut World, id: ItemId) {
    stop(w, id);
    remove_spawns(w, id);
}

// ---------------------------------------------------------------------------------------------------------
// spawning

/// `Spawn()`: one weighted pick among the entries that have room.
pub fn spawn(w: &mut World, id: ItemId) {
    defrag(w, id);
    let Some(d) = data(w, id) else { return };
    if d.entries.is_empty() || d.is_full() {
        return;
    }
    let probsum: i32 = d
        .entries
        .iter()
        .filter(|e| !e.is_full() && !e.disabled)
        .map(|e| e.probability)
        .sum();
    if probsum <= 0 {
        return;
    }
    let mut rand = rnd::min_max(w, 1, probsum);
    let Some(d) = data(w, id) else { return };
    let mut chosen = None;
    for (i, e) in d.entries.iter().enumerate() {
        if e.is_full() || e.disabled {
            continue;
        }
        if rand <= e.probability {
            chosen = Some(i);
            break;
        }
        rand -= e.probability;
    }
    if let Some(i) = chosen {
        let (_, flags) = spawn_entry(w, id, i);
        if let Some(e) = data_mut(w, id).and_then(|d| d.entries.get_mut(i)) {
            e.valid = flags;
        }
    }
}

/// `Spawn(index, out flags)`.
pub fn spawn_index(w: &mut World, id: ItemId, index: i32) -> (bool, EntryFlags) {
    let n = data(w, id).map_or(0, |d| d.entries.len());
    if index >= 0 && (index as usize) < n {
        spawn_entry(w, id, index as usize)
    } else {
        (false, EntryFlags::INVALID_ENTRY)
    }
}

/// `Spawn(entry, out flags)`.
pub fn spawn_entry(w: &mut World, id: ItemId, entry: usize) -> (bool, EntryFlags) {
    let (map, loc, has_parent) = match w.item(id) {
        Some(i) => (i.map, i.location, i.parent.is_some()),
        None => return (false, EntryFlags::empty()),
    };
    let Some(map) = map
        .filter(|m| *m != MapId::INTERNAL)
        .filter(|_| !has_parent)
    else {
        return (false, EntryFlags::empty());
    };
    let Some(e) = data(w, id).and_then(|d| d.entries.get(entry)).cloned() else {
        return (false, EntryFlags::INVALID_ENTRY);
    };

    let Some(ty) = w.types.find_type_by_name(entry_name(&e), false, true) else {
        return (false, EntryFlags::INVALID_TYPE);
    };

    let Some(assignments) = spawn_args::resolve_props(w, ty, e.properties.as_deref()) else {
        return (false, EntryFlags::INVALID_PROPS);
    };
    let Some(entity) = spawn_args::construct(w, ty, e.parameters.as_deref()) else {
        return (false, EntryFlags::INVALID_TYPE | EntryFlags::INVALID_PARAMS);
    };
    if !spawn_args::apply(w, entity, &assignments) {
        delete_entity(w, entity);
        return (false, EntryFlags::INVALID_PROPS);
    }

    let sid = spawner_id(id);
    if let Some(d) = data_mut(w, id) {
        d.spawned.insert(entity, entry);
        if let Some(en) = d.entries.get_mut(entry) {
            en.spawned.push(entity);
        }
    }
    let pos = get_spawn_position(w, id, Some(entity), map);

    match entity {
        EntityId::Mobile(m) => {
            let Some(t) = w.mobile(m).map(|x| x.ty) else {
                return (true, EntryFlags::empty());
            };
            (t.on_before_spawn)(w, m, pos, map);
            (t.move_to_world)(w, m, pos, map);
            if cdata::is_creature(w, m) {
                let (wr, way, team, home_here) = {
                    let d = data(w, id);
                    (
                        walking_range(w, id),
                        d.and_then(|d| d.way_point),
                        d.map_or(0, |d| d.team),
                        d.is_some_and(|d| d.spawn_location_is_home),
                    )
                };
                cdata::edit(w, m, |c| {
                    c.range_home = wr;
                    c.current_way_point = way;
                    if team > 0 {
                        c.team = team;
                    }
                    if home_here {
                        c.home = pos;
                        c.home_map = Some(map);
                    } else {
                        c.home = loc;
                        c.home_map = Some(map);
                    }
                });
            }
            if let Some(x) = w.mobile_mut(m) {
                x.spawner = Some(sid);
            }
            if let Some(t) = w.mobile(m).map(|x| x.ty) {
                (t.on_after_spawn)(w, m);
            }
        }
        EntityId::Item(i) => {
            let Some(t) = w.item(i).map(|x| x.ty) else {
                return (true, EntryFlags::empty());
            };
            (t.on_before_spawn)(w, i, pos, map);
            ops::move_to_world_map(w, i, pos, Some(map));
            ops::set_spawner(w, i, Some(sid));
            if let Some(t) = w.item(i).map(|x| x.ty) {
                (t.on_after_spawn)(w, i);
            }
        }
    }
    muo_server::item::base::clear_properties(w, id);
    (true, EntryFlags::empty())
}

// ---------------------------------------------------------------------------------------------------------
// positions

fn get_bounds_for_attempt(w: &mut World, id: ItemId) -> Rectangle3D {
    let Some(d) = data(w, id) else {
        return Rectangle3D::default();
    };
    if d.kind != Kind::Region {
        return d.spawn_bounds;
    }
    let Some(rd) = d.spawn_region.and_then(|r| base_region::data(w, r)) else {
        return Rectangle3D::default();
    };
    if rd.total_weight <= 0 {
        return Rectangle3D::default();
    }
    let (total, weights, rects) = (
        rd.total_weight,
        rd.rectangle_weights.clone(),
        rd.rectangles.clone().unwrap_or_default(),
    );
    let mut rand = rnd::below(w, total);
    for (j, cur) in weights.iter().enumerate() {
        if rand < *cur {
            return rects.get(j).copied().unwrap_or_default();
        }
        rand -= cur;
    }
    Rectangle3D::default()
}

fn all_spawn_bounds(w: &World, id: ItemId) -> Vec<Rectangle3D> {
    let Some(d) = data(w, id) else {
        return Vec::new();
    };
    if d.kind != Kind::Region {
        return vec![d.spawn_bounds];
    }
    d.spawn_region
        .and_then(|r| base_region::data(w, r))
        .and_then(|rd| rd.rectangles.clone())
        .unwrap_or_default()
}

fn supports_spiral_scan(d: &SpawnerData) -> bool {
    d.kind != Kind::Region && d.use_spiral_scan
}

/// `GetSpawnPosition(spawned, map)` (with `RegionSpawner`'s region/map mismatch check).
pub fn get_spawn_position(
    w: &mut World,
    id: ItemId,
    spawned: Option<EntityId>,
    map: MapId,
) -> Point3D {
    let Some(loc) = w.item(id).map(|i| i.location) else {
        return Point3D::ZERO;
    };
    if let Some(d) = data(w, id).filter(|d| d.kind == Kind::Region) {
        let region_ok = d.spawn_region.is_some_and(|r| {
            w.region(r).is_some_and(|reg| reg.map == map)
                && base_region::data(w, r).is_some_and(|rd| rd.total_weight > 0)
        });
        if !region_ok || map == MapId::INTERNAL {
            return loc;
        }
    }
    let mode = data(w, id).map_or(SpawnPositionMode::Automatic, |d| d.spawn_position_mode);
    if map == MapId::INTERNAL || mode == SpawnPositionMode::Abandoned {
        return loc;
    }

    let mut state = data_mut(w, id)
        .and_then(|d| d.pos_state.take())
        .unwrap_or_default();
    let result = spawn_position_inner(w, id, spawned, map, loc, mode, &mut state);
    if let Some(d) = data_mut(w, id) {
        // abandoning dropped the state; otherwise keep it
        if d.spawn_position_mode != SpawnPositionMode::Abandoned {
            d.pos_state = Some(state);
        }
    }
    result
}

fn spawn_position_inner(
    w: &mut World,
    id: ItemId,
    spawned: Option<EntityId>,
    map: MapId,
    loc: Point3D,
    mode: SpawnPositionMode,
    state: &mut PosState,
) -> Point3D {
    let (is_mobile, can_swim, cant_walk) = match spawned {
        Some(EntityId::Mobile(m)) => w
            .mobile(m)
            .map_or((true, false, false), |x| (true, x.can_swim, x.cant_walk)),
        _ => (false, false, false),
    };
    let is_water_mob = can_swim && cant_walk;
    let mut has_non_transient = false;

    // Phase 1: random attempts
    let max_attempts = data(w, id).map_or(0, |d| d.max_spawn_attempts);
    let max_attempts = if max_attempts > 0 {
        max_attempts
    } else {
        super::DEFAULT_MAX_SPAWN_ATTEMPTS
    };
    for _ in 0..max_attempts {
        let raw = get_bounds_for_attempt(w, id);
        if raw == Rectangle3D::default() {
            return loc;
        }
        let b = raw.normalized();
        let x = rnd::min_max(w, b.start.x, b.end.x - 1);
        let y = rnd::min_max(w, b.start.y, b.end.y - 1);
        let (min_z, max_z) = (b.start.z, b.start.z.max(b.end.z - 1));

        let (ok, spawn_z, reason) = if is_mobile {
            match fit::can_spawn_mobile(w, map, x, y, min_z, max_z, can_swim, cant_walk) {
                Ok(z) => (true, z, muo_server::map::SpawnFailureReason::empty()),
                Err(r) => (false, 0, r),
            }
        } else {
            match fit::can_spawn_item(w, map, x, y, min_z, max_z) {
                Some(z) => (true, z, muo_server::map::SpawnFailureReason::empty()),
                None => (
                    false,
                    0,
                    muo_server::map::SpawnFailureReason::NON_TRANSIENT_BLOCKER,
                ),
            }
        };

        if ok {
            if is_mobile && cache::is_blocked_by_house(w, map, x, y, spawn_z) {
                has_non_transient = true;
            } else {
                if state.should_cache_positions(mode) {
                    cache::set_valid(w, map, Point3D::new(x, y, spawn_z), is_water_mob);
                }
                return Point3D::new(x, y, spawn_z);
            }
        } else if reason.contains(muo_server::map::SpawnFailureReason::NON_TRANSIENT_BLOCKER) {
            has_non_transient = true;
        }
    }

    // Phase 2
    if mode == SpawnPositionMode::Disabled {
        return loc;
    }
    let use_optimization = mode == SpawnPositionMode::Enabled
        || (mode == SpawnPositionMode::Automatic && has_non_transient);
    if !use_optimization {
        return loc;
    }
    state.non_transient_failures += 1;

    let all_bounds = all_spawn_bounds(w, id);

    // Phase 3
    let spiral = data(w, id).is_some_and(supports_spiral_scan);
    if !spiral {
        state.spiral_complete = true;
    } else if !state.spiral_complete {
        let raw = all_bounds.first().copied().unwrap_or_default();
        if raw != Rectangle3D::default() {
            let b = raw.normalized();
            let (min_z, max_z) = (b.start.z, b.start.z.max(b.end.z - 1));
            let rings = if state.spiral_ring == 0 { 3 } else { 1 };
            state.spiral_complete = cache::continue_spiral_scan(
                w,
                map,
                loc,
                b,
                min_z,
                max_z,
                can_swim,
                cant_walk,
                &mut state.spiral_ring,
                &mut state.spiral_ring_position,
                rings,
            );
        }
    }

    // Phase 4
    if let Some(pos) = verified_cached_position(
        w,
        map,
        &all_bounds,
        is_mobile,
        is_water_mob,
        can_swim,
        cant_walk,
    ) && pos != loc
    {
        state.consecutive_useless = 0;
        return pos;
    }
    state.consecutive_useless += 1;

    // Phase 5
    if mode == SpawnPositionMode::Automatic && state.should_abandon() {
        if let Some(d) = data_mut(w, id) {
            d.spawn_position_mode = SpawnPositionMode::Abandoned;
            d.pos_state = None;
        }
        tracing::warn!(
            "Spawner {} at {} ({}) marked abandoned - no valid spawn positions found after spiral scan.",
            id.0.value(),
            loc,
            w.maps.get(map).map_or("null", |m| m.name())
        );
    }
    loc
}

fn verified_cached_position(
    w: &mut World,
    map: MapId,
    all_bounds: &[Rectangle3D],
    is_mobile: bool,
    is_water_mob: bool,
    can_swim: bool,
    cant_walk: bool,
) -> Option<Point3D> {
    let (pos, containing) = cache::try_get_random_position(w, map, all_bounds, is_water_mob)?;
    let nb = containing.normalized();
    let (min_z, max_z) = (nb.start.z, nb.start.z.max(nb.end.z - 1));
    let z = if is_mobile {
        fit::can_spawn_mobile(w, map, pos.x, pos.y, min_z, max_z, can_swim, cant_walk).ok()?
    } else {
        fit::can_spawn_item(w, map, pos.x, pos.y, min_z, max_z)?
    };
    if is_mobile && cache::is_blocked_by_house(w, map, pos.x, pos.y, z) {
        return None;
    }
    Some(Point3D::new(pos.x, pos.y, z))
}

/// `ResetSpawnPositionState()`.
fn reset_spawn_position_state(w: &mut World, id: ItemId) {
    if let Some(d) = data_mut(w, id) {
        if let Some(s) = d.pos_state.as_mut() {
            s.reset();
        }
        if d.spawn_position_mode == SpawnPositionMode::Abandoned {
            d.spawn_position_mode = SpawnPositionMode::Automatic;
        }
    }
}

// ---------------------------------------------------------------------------------------------------------
// Item virtuals

fn on_map_change(w: &mut World, id: ItemId) {
    (ITEM_BASE.on_map_change)(w, id);
    if let Some(loc) = w.item(id).map(|i| i.location)
        && is_home_range_style_at(w, id, loc)
    {
        super::commands::remove_projection(w, id);
    }
}

fn on_location_change(w: &mut World, id: ItemId, old: Point3D) {
    (ITEM_BASE.on_location_change)(w, id, old);
    if is_home_range_style_at(w, id, old) {
        let width = data(w, id).map_or(0, |d| d.spawn_bounds.width());
        set_home_range(w, id, width / 2);
        super::commands::remove_projection(w, id);
    }
    reset_spawn_position_state(w, id);
}

fn on_after_duped(w: &mut World, id: ItemId, new_item: ItemId) {
    (ITEM_BASE.on_after_duped)(w, id, new_item);
    let Some(kind) = data(w, id).map(|d| d.kind) else {
        return;
    };
    if data(w, new_item).is_none() {
        let mut nd = SpawnerData::new(kind);
        nd.guid = new_guid();
        if let Some(i) = w.item_mut(new_item) {
            i.ext = Ext::new(nd);
        }
    } else if let Some(nd) = data_mut(w, new_item) {
        nd.guid = new_guid();
        nd.spawned.clear();
    }
    copy_entries_to(w, id, new_item);
}

fn on_delete(w: &mut World, id: ItemId) {
    (ITEM_BASE.on_delete)(w, id);
    // the borders check `Spawner.Deleted` on their next tick
    super::commands::remove_projection(w, id);
    stop(w, id);
    remove_spawns(w, id);
    if let Some(t) = data_mut(w, id).and_then(|d| d.timer.take()) {
        timer::release(w, t);
    }
}

fn on_double_click(w: &mut World, id: ItemId, from: MobileId) {
    super::gumps::send_spawner_gump(w, from, id, None, 0);
}

fn on_single_click(w: &mut World, id: ItemId, from: MobileId) {
    (ITEM_BASE.on_single_click)(w, id, from);
    let running = data(w, id).is_some_and(|d| d.running);
    ops::label_to(w, id, from, if running { "[Running]" } else { "[Off]" });
}

fn get_properties(w: &World, id: ItemId, list: &mut PropertyList) {
    (ITEM_BASE.get_properties)(w, id, list);
    let Some(d) = data(w, id) else { return };
    if !d.running {
        list.add(1060743); // inactive
        return;
    }
    list.add(1060742); // active
    list.add_int(1060656, d.count); // amount to make: ~1_val~
    if d.spawn_bounds != Rectangle3D::default() {
        list.add_str(1061169, &d.spawn_bounds.to_string()); // range ~1_val~
    }
    list.add_str(1050039, &format!("walking range:\t{}", d.walking_range)); // ~1_NUMBER~ ~2_ITEMNAME~
    list.add_str(
        1053099,
        &format!("group:\t{}", if d.group { "True" } else { "False" }),
    ); // ~1_oretype~: ~2_armortype~
    list.add_str(1060847, &format!("team:\t{}", d.team)); // ~1_val~ ~2_val~
    list.add_str(
        1063483,
        &format!("delay:\t{} to {}", d.min_delay, d.max_delay),
    ); // ~1_MATERIAL~: ~2_ITEMNAME~

    if d.kind == Kind::Region
        && let Some(name) = d
            .spawn_region
            .and_then(|r| w.region(r))
            .and_then(|r| r.name.as_deref())
    {
        list.add_str(1076228, &format!("region:\t{name}")); // ~1_DUMMY~ ~2_DUMMY~
    }

    // `CountSpawns` defrags first; the list is built from a read-only world, so count what the defrag would keep
    // (a spawn the spawner no longer tracks stays: `OnDefragSpawn` only drops what it can unlink)
    for (i, e) in d.entries.iter().take(6).enumerate() {
        let n = e
            .spawned
            .iter()
            .filter(|s| !(defrag_drops(w, **s) && d.spawned.contains_key(*s)))
            .count();
        list.add_str(1060658 + i as i32, &format!("\t{}\t{}", entry_name(e), n));
    }
}

// ---------------------------------------------------------------------------------------------------------
// ProximitySpawner

/// `ValidTrigger(m)`.
pub fn valid_trigger(w: &World, m: MobileId) -> bool {
    if let Some(c) = cdata::cd(w, m)
        && (c.is_dead_pet || !(c.controlled || c.summoned))
    {
        return false;
    }
    let Some(x) = w.mobile(m) else { return false };
    x.access_level == AccessLevel::Player
        && (x.player || (mx::alive(w, m) && !x.hidden && (x.ty.can_be_damaged)(w, m)))
}

/// `OnMovement(m, oldLocation)`.
fn proximity_on_movement(w: &mut World, id: ItemId, m: MobileId, old: Point3D) {
    let Some(d) = data(w, id) else { return };
    if !d.running {
        return;
    }
    let (empty, end, range, instant, msg) = (
        d.is_empty(),
        d.end,
        d.trigger_range,
        d.instant_flag,
        d.spawn_message.clone(),
    );
    let here = ops::get_world_location(w, id);
    if empty
        && end <= w.now
        && mx::in_range_point(w, m, here, range)
        && w.mobile(m).is_some_and(|x| x.location != old)
        && valid_trigger(w, m)
    {
        muo_content::text::send_message_to(w, msg.as_ref(), m);
        do_timer(w, id);
        spawn(w, id);
        if instant {
            let spawned: Vec<EntityId> = data(w, id)
                .map(|d| d.spawned.keys().copied().collect())
                .unwrap_or_default();
            for s in spawned {
                if let EntityId::Mobile(sm) = s {
                    mx::set_combatant(w, sm, Some(m));
                }
            }
        }
    }
}

/// The region a spawner is bound to (`ISpawner.Region`): a `RegionSpawner`'s, else the region at its location.
pub fn region(w: &World, id: ItemId) -> Option<muo_server::region::RegionId> {
    let d = data(w, id)?;
    if d.kind == Kind::Region {
        return d.spawn_region;
    }
    let i = w.item(id)?;
    Some(muo_server::region::find(w, i.location, i.map))
}

/// `RegionSpawner.SpawnRegion` setter.
pub fn set_spawn_region(w: &mut World, id: ItemId, region: Option<muo_server::region::RegionId>) {
    let name = region
        .and_then(|r| w.region(r))
        .and_then(|r| r.name.clone());
    if let Some(r) = region {
        base_region::init_rectangles(w, r);
    }
    if let Some(d) = data_mut(w, id) {
        d.spawn_region = region;
        d.spawn_region_name = name;
    }
    ops::invalidate_properties(w, id);
}
