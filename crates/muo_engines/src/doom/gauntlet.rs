//! `GauntletSpawner` and `GauntletRegion`.
//!
//! `GauntletSpawner.Spawn` finds the creature class with `AssemblyHandler.FindTypeByName(TypeName)`: the type
//! registry's by-name lookup (a short name is found by its simple name, a dotted one by its full name).

use muo_content::factory;
use muo_content::hooks::rnd;
use muo_io::{GenericReader, GenericWriter, Result};
use muo_primitives::{Point3D, Rectangle2D, TimeSpan};
use muo_server::World;
use muo_server::ext::Ext;
use muo_server::ids::{ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType, ops};
use muo_server::map::MapId;
use muo_server::region::{self, RegionId, RegionType};
use muo_server::serialization_ext::{ReadEntityExt, WriteEntityExt};
use muo_server::timer::{self, TimerHandle};
use muo_server::type_registry::TypeRegistry;

pub const PLAYERS_PER_SPAWN: i32 = 5;
pub const IN_SEQUENCE_ITEM_HUE: i32 = 0x000;
pub const IN_PROGRESS_ITEM_HUE: i32 = 0x676;
pub const COMPLETED_ITEM_HUE: i32 = 0x455;

/// `GauntletSpawnerState`.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
#[repr(i32)]
pub enum State {
    #[default]
    InSequence = 0,
    InProgress = 1,
    Completed = 2,
}

impl State {
    pub fn from_i32(v: i32) -> State {
        match v {
            1 => State::InProgress,
            2 => State::Completed,
            _ => State::InSequence,
        }
    }
}

/// What `GauntletSpawner` holds.
#[derive(Default)]
pub struct SpawnerData {
    pub region_bounds: Rectangle2D,
    pub traps: Vec<ItemId>,
    pub creatures: Vec<MobileId>,
    pub type_name: Option<String>,
    pub door: Option<ItemId>,
    pub addon: Option<ItemId>,
    pub sequence: Option<ItemId>,
    pub state: State,
    pub region: Option<RegionId>,
    pub timer: TimerHandle,
}

pub fn data(w: &World, id: ItemId) -> Option<&SpawnerData> {
    w.item(id)?.ext.get::<SpawnerData>()
}

pub fn data_mut(w: &mut World, id: ItemId) -> Option<&mut SpawnerData> {
    w.item_mut(id)?.ext.get_mut::<SpawnerData>()
}

/// `door.Link`.
pub fn door_link(w: &World, door: ItemId) -> Option<ItemId> {
    w.item(door)?
        .ext
        .get::<muo_items::construction::doors::DoorData>()?
        .link
}

fn deleted(w: &World, id: ItemId) -> bool {
    w.item(id).is_none_or(|i| i.deleted())
}

fn delete_item(w: &mut World, id: ItemId) {
    if let Some(t) = w.item(id).map(|i| i.ty) {
        (t.delete)(w, id);
    }
}

fn mobile_deleted(w: &World, m: MobileId) -> bool {
    w.mobile(m).is_none_or(|x| x.deleted)
}

/// `GauntletSpawner(typeName)`: `base(0x36FE)`, `Visible = false; Movable = false`.
pub fn new_spawner(w: &mut World, type_name: Option<&str>) -> ItemId {
    let id = ops::new_item_with_id(w, &GAUNTLET_SPAWNER, 0x36FE);
    ops::set_visible(w, id, false);
    ops::set_movable(w, id, false);
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(SpawnerData {
            type_name: type_name.map(str::to_owned),
            ..Default::default()
        });
    }
    id
}

/// `HasCompleted`.
pub fn has_completed(w: &World, id: ItemId) -> bool {
    let Some(d) = data(w, id) else { return false };
    !d.creatures.is_empty() && d.creatures.iter().all(|m| mobile_deleted(w, *m))
}

/// `State = value`.
pub fn set_state(w: &mut World, id: ItemId, value: State) {
    let changed = data_mut(w, id).is_some_and(|d| {
        let c = d.state != value;
        d.state = value;
        c
    });
    if changed {
        update(w, id, true);
    }
}

/// `Update(shouldSpawn)`.
fn update(w: &mut World, id: ItemId, should_spawn: bool) {
    let state = data(w, id).map_or(State::InSequence, |d| d.state);
    let lock_doors = state == State::InProgress;
    let hue = match state {
        State::InSequence => IN_SEQUENCE_ITEM_HUE,
        State::InProgress => IN_PROGRESS_ITEM_HUE,
        State::Completed => COMPLETED_ITEM_HUE,
    };
    let (door, addon_item) = data(w, id)
        .map(|d| (d.door, d.addon))
        .unwrap_or((None, None));
    if let Some(door) = door {
        crate::util::hue(w, door, hue);
        muo_items::construction::doors::set_locked(w, door, lock_doors);
        if lock_doors {
            let k = muo_content::items::gold::random_value(w);
            muo_items::construction::doors::set_key_value(w, door, k);
            muo_items::construction::doors::set_open(w, door, false);
        }
        let link = door_link(w, door).filter(|l| !deleted(w, *l));
        if let Some(l) = link {
            crate::util::hue(w, l, hue);
            muo_items::construction::doors::set_locked(w, l, lock_doors);
            if lock_doors {
                let k = muo_content::items::gold::random_value(w);
                muo_items::construction::doors::set_key_value(w, l, k);
                muo_items::construction::doors::set_open(w, door, false);
            }
        }
    }
    if let Some(a) = addon_item {
        crate::util::hue(w, a, hue);
    }

    if state == State::InProgress {
        create_region(w, id);
        if should_spawn {
            full_spawn(w, id);
        }
        let mut t = data(w, id).map(|d| d.timer).unwrap_or_default();
        t.cancel(w);
        let h = timer::start(
            w,
            TimeSpan::from_secs(1),
            TimeSpan::from_secs(1),
            0,
            move |w| slice(w, id),
        );
        if let Some(d) = data_mut(w, id) {
            d.timer = h;
        }
    } else {
        stop(w, id);
    }
}

/// `CreateRegion()`: `new GauntletRegion(this, map)`.
fn create_region(w: &mut World, id: ItemId) {
    if data(w, id).is_none_or(|d| d.region.is_some()) {
        return;
    }
    let map = w.item(id).and_then(|i| i.map);
    let Some(map) = map.filter(|m| *m != MapId::INTERNAL) else {
        return;
    };
    let (loc, bounds) = (
        w.item(id).map_or(Point3D::ZERO, |i| i.location),
        data(w, id).map_or(Rectangle2D::EMPTY, |d| d.region_bounds),
    );
    let parent = region::find(w, loc, Some(map));
    let r = region::new_region(
        w,
        &GAUNTLET_REGION,
        None,
        map,
        Some(parent),
        None,
        region::convert_all_to_3d(&[bounds]),
    );
    muo_content::regions::base_region::init_data(w, r);
    if let Some(reg) = w.region_mut(r) {
        reg.go_location = loc;
    }
    region::register(w, r);
    if let Some(d) = data_mut(w, id) {
        d.region = Some(r);
    }
}

/// `DestroyRegion()`.
fn destroy_region(w: &mut World, id: ItemId) {
    if let Some(r) = data_mut(w, id).and_then(|d| d.region.take()) {
        region::unregister(w, r);
    }
}

/// `GauntletRegion`: `AlterLightLevel` is 12, and entering/leaving does nothing (no `BaseRegion` behavior).
pub static GAUNTLET_REGION: RegionType = RegionType {
    name: "GauntletRegion",
    parent: Some(&muo_content::regions::base_region::BASE_REGION),
    alter_light_level: |_, _, _, global, _| *global = 12,
    on_enter: |_, _, _| {},
    on_exit: |_, _, _| {},
    ..muo_content::regions::base_region::BASE_REGION
};

/// `Stop()`.
pub fn stop(w: &mut World, id: ItemId) {
    let creatures = data_mut(w, id)
        .map(|d| std::mem::take(&mut d.creatures))
        .unwrap_or_default();
    let _ = creatures;
    let traps = data_mut(w, id)
        .map(|d| std::mem::take(&mut d.traps))
        .unwrap_or_default();
    let _ = traps;
    destroy_region(w, id);
    let mut t = data(w, id).map(|d| d.timer).unwrap_or_default();
    t.cancel(w);
    if let Some(d) = data_mut(w, id) {
        d.timer = t;
    }
}

/// `ComputeTrapCount()`.
fn compute_trap_count(w: &World, id: ItemId) -> i32 {
    let b = data(w, id).map_or(Rectangle2D::EMPTY, |d| d.region_bounds);
    b.width() * b.height() / 100
}

fn remove_all_traps(w: &mut World, id: ItemId) {
    let traps = data_mut(w, id)
        .map(|d| std::mem::take(&mut d.traps))
        .unwrap_or_default();
    for t in traps {
        delete_item(w, t);
    }
}

/// `SpawnTrap()`.
fn spawn_trap(w: &mut World, id: ItemId) {
    use muo_items::traps::trap_classes::{saw_base_id, spike_base_id};
    let Some(map) = w.item(id).and_then(|i| i.map) else {
        return;
    };
    let random = rnd::below(w, 100);
    let trap = if random < 22 {
        let kind = if rnd::bool(w) { 2 } else { 3 };
        factory::make_item(w, "Server.Items.SawTrap")
            .inspect(|t| set_item_id(w, *t, saw_base_id(kind)))
    } else if random < 44 {
        let kind = if rnd::bool(w) { 2 } else { 3 };
        factory::make_item(w, "Server.Items.SpikeTrap")
            .inspect(|t| set_item_id(w, *t, spike_base_id(kind)))
    } else if random < 66 {
        let kind = if rnd::bool(w) { 0 } else { 1 };
        Some(muo_items::traps::trap_classes::new_gas_trap(w, kind, None))
    } else if random < 88 {
        factory::make_item(w, "Server.Items.FireColumnTrap")
    } else {
        factory::make_item(w, "Server.Items.MushroomTrap")
    };
    let Some(trap) = trap else { return };
    if random >= 66 {
        crate::util::hue(w, trap, 0x451);
    }
    let bounds = data(w, id).map_or(Rectangle2D::EMPTY, |d| d.region_bounds);
    let zloc = w.item(id).map_or(0, |i| i.location.z);
    // try 10 times to find a valid location
    for _ in 0..10 {
        let x = rnd::from(w, bounds.x(), bounds.width());
        let y = rnd::from(w, bounds.y(), bounds.height());
        let mut z = zloc;
        if !muo_server::map::fit::can_fit(w, map, x, y, z, 16, false, false, true) {
            z = w.maps.get_average_z(map, x, y);
        }
        if !muo_server::map::fit::can_fit(w, map, x, y, z, 16, false, false, true) {
            continue;
        }
        ops::move_to_world_map(w, trap, Point3D::new(x, y, z), Some(map));
        if let Some(d) = data_mut(w, id) {
            d.traps.push(trap);
        }
        return;
    }
    delete_item(w, trap);
}

fn set_item_id(w: &mut World, id: ItemId, graphic: i32) {
    if let Some(t) = w.item(id).map(|i| i.ty) {
        (t.set_item_id)(w, id, graphic);
    }
}

/// `ComputeSpawnCount()`.
fn compute_spawn_count(w: &World, id: ItemId) -> i32 {
    let mut players = 0;
    if let Some(map) = w.item(id).and_then(|i| i.map) {
        let loc = ops::get_world_location(w, id);
        let r = region::find(w, loc, Some(map));
        if let Some(reg) = region::get_region_by_name(w, r, "Doom Gauntlet", true) {
            players = region::get_player_count(w, reg) as i32;
        }
    }
    if players == 0
        && let Some(r) = data(w, id).and_then(|d| d.region)
    {
        players = region::get_player_count(w, r) as i32;
    }
    ((players + PLAYERS_PER_SPAWN - 1) / PLAYERS_PER_SPAWN).max(1)
}

fn remove_all_creatures(w: &mut World, id: ItemId) {
    let creatures = data_mut(w, id)
        .map(|d| std::mem::take(&mut d.creatures))
        .unwrap_or_default();
    for m in creatures {
        if let Some(t) = w.mobile(m).map(|x| x.ty) {
            (t.delete)(w, m);
        }
    }
}

/// `FullSpawn()`.
fn full_spawn(w: &mut World, id: ItemId) {
    remove_all_creatures(w, id);
    let count = compute_spawn_count(w, id);
    for _ in 0..count {
        spawn(w, id);
    }
    remove_all_traps(w, id);
    for _ in 0..compute_trap_count(w, id) {
        spawn_trap(w, id);
    }
}

/// `Spawn()`: `AssemblyHandler.FindTypeByName(TypeName)`, then `MoveToWorld(GetWorldLocation(), Map)`.
fn spawn(w: &mut World, id: ItemId) {
    let Some(name) = data(w, id).and_then(|d| d.type_name.clone()) else {
        return;
    };
    // `AssemblyHandler.FindTypeByName(name)`: the simple name, ignoring case (fullName: false, ignoreCase: true)
    let found = w.types.find_type_by_name(&name, false, true);
    let Some(muo_server::type_registry::EntityType::Mobile(ty)) = found else {
        return;
    };
    let Some(construct) = ty.construct else {
        return;
    };
    let mob = construct(w);
    let loc = ops::get_world_location(w, id);
    if let Some(map) = w.item(id).and_then(|i| i.map) {
        muo_server::mobile::base::move_to_world(w, mob, loc, map);
    }
    if let Some(d) = data_mut(w, id) {
        d.creatures.push(mob);
    }
}

/// `RecurseReset()`.
fn recurse_reset(w: &mut World, id: ItemId) {
    if data(w, id).is_some_and(|d| d.state != State::InSequence) {
        set_state(w, id, State::InSequence);
        if let Some(s) = data(w, id)
            .and_then(|d| d.sequence)
            .filter(|s| !deleted(w, *s))
        {
            recurse_reset(w, s);
        }
    }
}

/// `Slice()`.
fn slice(w: &mut World, id: ItemId) {
    if data(w, id).is_none_or(|d| d.state != State::InProgress) {
        return;
    }
    let count = compute_spawn_count(w, id);
    let have = data(w, id).map_or(0, |d| d.creatures.len() as i32);
    for _ in have..count {
        spawn(w, id);
    }
    if has_completed(w, id) {
        set_state(w, id, State::Completed);
        if let Some(s) = data(w, id)
            .and_then(|d| d.sequence)
            .filter(|s| !deleted(w, *s))
        {
            if data(w, s).is_some_and(|d| d.state == State::Completed) {
                recurse_reset(w, id);
            }
            set_state(w, s, State::InProgress);
        }
    }
}

fn on_delete(w: &mut World, id: ItemId) {
    (ITEM_BASE.on_delete)(w, id);
    stop(w, id);
    let (door, addon_item) = data(w, id)
        .map(|d| (d.door, d.addon))
        .unwrap_or((None, None));
    if let Some(door) = door {
        if let Some(l) = door_link(w, door) {
            delete_item(w, l);
        }
        delete_item(w, door);
    }
    if let Some(a) = addon_item {
        delete_item(w, a);
    }
}

/// Both lists are written whole (no `[Tidy]`): a deleted entry is serial -1.
fn write_trap_list(w: &World, out: &mut dyn GenericWriter, list: &[ItemId]) {
    out.write_encoded_int(list.len() as i32);
    for &i in list {
        out.write_item(w, Some(i));
    }
}

/// `[SerializationGenerator(2, false)] GauntletSpawner`.
pub static GAUNTLET_SPAWNER: ItemType = ItemType {
    full_name: "Server.Engines.Doom.GauntletSpawner",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| new_spawner(w, None)),
    default_name: |w, id| crate::template_keys::item_name(w, id, ITEM_BASE.default_name),
    on_delete,
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        let Some(d) = data(w, id) else { return };
        out.write_i32(2);
        out.write_rect2d(d.region_bounds);
        write_trap_list(w, out, &d.traps);
        out.write_encoded_int(d.creatures.len() as i32);
        for &m in &d.creatures {
            out.write_mobile(w, Some(m));
        }
        out.write_opt_string(d.type_name.as_deref());
        out.write_item(w, d.door);
        out.write_item(w, d.addon);
        out.write_item(w, d.sequence);
        out.write_enum(4, d.state as i64);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let version = r.read_i32()?;
        let d = read_body(w, r, version)?;
        if let Some(i) = w.item_mut(id) {
            i.ext = Ext::new(d);
        }
        // `[AfterDeserialization(false)] Update(false)`
        timer::start(w, TimeSpan::ZERO, TimeSpan::ZERO, 1, move |w| {
            update(w, id, false)
        });
        Ok(())
    },
    ..ITEM_BASE
};

fn read_body(w: &World, r: &mut dyn GenericReader, version: i32) -> Result<SpawnerData> {
    let mut d = SpawnerData {
        region_bounds: r.read_rect2d()?,
        ..SpawnerData::default()
    };
    if version < 2 {
        d.traps = r.read_item_list(w)?;
        d.creatures = r.read_mobile_list(w)?;
    } else {
        d.traps = r.read_item_list_kept(true)?;
        d.creatures = r.read_mobile_list_kept(true)?;
    }
    d.type_name = r.read_string()?;
    d.door = r.read_item(w)?;
    d.addon = r.read_item(w)?;
    d.sequence = r.read_item(w)?;
    d.state = State::from_i32(if version < 2 {
        r.read_i32()?
    } else {
        r.read_enum(4)? as i32
    });
    Ok(d)
}

pub fn register(types: &mut TypeRegistry) {
    types.register_item(&GAUNTLET_SPAWNER, &[]);
}

#[cfg(test)]
mod tests {
    use super::*;
    use muo_io::{SaveReader, SaveWriter};
    use muo_server::mobile::MOBILE_BASE;

    fn save(w: &World, id: ItemId) -> Vec<u8> {
        let mut out = SaveWriter::new(true);
        (GAUNTLET_SPAWNER.serialize)(w, id, &mut out);
        out.into_vec()
    }

    /// `_traps` and `_creatures` have no `[Tidy]`: a deleted entry is written as serial -1 and read back as a null
    /// entry, so the saved counts stay as they were.
    #[test]
    fn deleted_trap_and_creature_stay_in_the_lists() {
        let mut w = World::new();
        let id = new_spawner(&mut w, None);
        let (t1, t2) = (w.new_item(&ITEM_BASE), w.new_item(&ITEM_BASE));
        let m = w.new_mobile(&MOBILE_BASE);
        {
            let d = data_mut(&mut w, id).unwrap();
            d.traps = vec![t1, t2];
            d.creatures = vec![m];
        }
        let before = save(&w, id);
        w.item_mut(t2).unwrap().flags |= muo_server::item::ImplFlag::DELETED;
        w.mobile_mut(m).unwrap().deleted = true;
        let after = save(&w, id);

        // By construction: the same bytes with the two serials replaced by -1.
        let mut expected = before.clone();
        for (old, new) in [
            (w.item(t2).unwrap().serial, -1i32),
            (w.mobile(m).unwrap().serial, -1),
        ] {
            let old = old.value().to_le_bytes();
            let at = expected.windows(4).rposition(|b| b == old).unwrap();
            expected[at..at + 4].copy_from_slice(&new.to_le_bytes());
        }
        assert_eq!(after, expected);

        // Round trip: the entries come back as nulls (an id that resolves to nothing), indexes unchanged.
        let id2 = new_spawner(&mut w, None);
        let mut r = SaveReader::new(&after);
        (GAUNTLET_SPAWNER.deserialize)(&mut w, id2, &mut r).unwrap();
        let d = data(&w, id2).unwrap();
        assert_eq!((d.traps.len(), d.creatures.len()), (2, 1));
        assert_eq!(d.traps[0], t1);
        assert!(w.item(d.traps[1]).is_none());
    }
}
