//! `CannedEvilTimer`: once a day the dungeon and Lost Lands champion spawns rotate.

use indexmap::IndexSet;
use muo_content::hooks::rnd;
use muo_primitives::{DateTime, TimeSpan, timespan::TICKS_PER_DAY};
use muo_server::World;
use muo_server::ids::ItemId;
use muo_server::timer;

use super::spawn::{self, Kind};

/// `_dungeonSpawns`, `_lostLandsSpawns` and `_sliceTime`.
#[derive(Default)]
pub struct CannedEvilState {
    dungeon: IndexSet<ItemId>,
    lost_lands: IndexSet<ItemId>,
    slice_time: DateTime,
}

fn state(w: &mut World) -> &mut CannedEvilState {
    w.resources.get_mut::<CannedEvilState>()
}

fn set_for(w: &mut World, kind: Kind) -> &mut IndexSet<ItemId> {
    match kind {
        Kind::LostLands => &mut state(w).lost_lands,
        _ => &mut state(w).dungeon,
    }
}

fn kind_of(w: &World, spawn: ItemId) -> Kind {
    w.item(spawn).map_or(Kind::Plain, |i| spawn::kind(i.ty))
}

/// `AddSpawn(spawn)`.
pub fn add_spawn(w: &mut World, spawn: ItemId) {
    let kind = kind_of(w, spawn);
    set_for(w, kind).insert(spawn);
    on_slice(w, kind, false);
}

/// `RemoveSpawn(spawn)`.
pub fn remove_spawn(w: &mut World, spawn: ItemId) {
    let kind = kind_of(w, spawn);
    set_for(w, kind).shift_remove(&spawn);
    on_slice(w, kind, false);
}

/// `OnSlice(spawns, rotate)`.
pub fn on_slice(w: &mut World, kind: Kind, rotate: bool) {
    let spawns: Vec<ItemId> = set_for(w, kind).iter().copied().collect();
    if spawns.is_empty() {
        return;
    }
    let mut queue: Vec<ItemId> = Vec::new();
    for s in spawns {
        let Some(d) = spawn::data(w, s) else { continue };
        let (active, kills, level) = (d.active, d.kills, d.level);
        if spawn::always_active(w, s) && !active {
            if let Some(d) = spawn::data_mut(w, s) {
                d.ready_to_activate = true;
            }
        } else if rotate && (!active || kills == 0 && level == 0) {
            spawn::set_active(w, s, false);
            if let Some(d) = spawn::data_mut(w, s) {
                d.ready_to_activate = false;
            }
            queue.push(s);
        }
    }
    if rotate && !queue.is_empty() {
        let i = rnd::below(w, queue.len() as i32) as usize;
        if let Some(d) = spawn::data_mut(w, queue[i]) {
            d.ready_to_activate = true;
        }
    }
}

/// `OnTick()`.
fn on_tick(w: &mut World) {
    let restarting = w
        .resources
        .get::<muo_content::login::small::AutoRestartState>()
        .is_some_and(|a| a.restarting);
    let now = w.now;
    if !restarting && now >= state(w).slice_time {
        on_slice(w, Kind::Dungeon, true);
        on_slice(w, Kind::LostLands, true);
        let day_start = DateTime::from_ticks(now.ticks() - now.ticks() % TICKS_PER_DAY);
        state(w).slice_time = day_start + TimeSpan::from_days(1);
    }
}

/// `Initialize()`: `Instance = new CannedEvilTimer(); Instance.Start(); Instance.OnTick();`
pub fn initialize(w: &mut World) {
    let now = w.now;
    state(w).slice_time = now;
    timer::start(w, TimeSpan::ZERO, TimeSpan::from_mins(1), 0, on_tick);
    on_tick(w);
}
