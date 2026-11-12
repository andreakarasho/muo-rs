//! `ChampionSpawn` (the item, `ChampionSpawnRegion`, `IdolOfTheChampion`), `DungeonChampionSpawn` and
//! `LLChampionSpawn`.
//!
//! The state is [`SpawnData`] in `Item::ext`; the three classes share it. `ProximitySpawn`, `AlwaysActive` and
//! `HasStarRoomGate` are decided by [`kind`].
//!
//! Everything a champion drops or a spawn asks of other crates goes through [`super::CannedEvilHooks`] (set in
//! `gates` and `special_wiring`).

use indexmap::IndexMap;
use muo_content::creature::{cd, cd_mut};
use muo_content::factory;
use muo_content::hooks::rnd;
use muo_content::player_mobile as pmob;
use muo_io::{GenericReader, GenericWriter, Result};
use muo_primitives::{DateTime, Point2D, Point3D, Rectangle2D, TimeSpan};
use muo_server::World;
use muo_server::ext::Ext;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType, ops};
use muo_server::light_type::LightType;
use muo_server::map::MapId;
use muo_server::region::{self, RegionId, RegionType};
use muo_server::serialization_ext::{ReadEntityExt, WriteEntityExt};
use muo_server::timer::{self, TimerHandle};
use muo_server::type_registry::TypeRegistry;

use super::info::{self, ChampionSpawnType};

/// The three classes of the item.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Kind {
    Plain,
    Dungeon,
    LostLands,
}

/// `ProximitySpawn`, `AlwaysActive`, `HasStarRoomGate` by class.
pub fn kind(ty: &'static ItemType) -> Kind {
    if ty.is(&DUNGEON_CHAMPION_SPAWN) {
        Kind::Dungeon
    } else if ty.is(&LL_CHAMPION_SPAWN) {
        Kind::LostLands
    } else {
        Kind::Plain
    }
}

/// What `ChampionSpawn` holds.
pub struct SpawnData {
    pub level: i32,
    pub activated_by_proximity: bool,
    pub next_proximity_time: DateTime,
    pub max_level: i32,
    pub activated_by_valor: bool,
    /// `_damageEntries` (a `Dictionary`: insertion order).
    pub damage_entries: IndexMap<MobileId, i32>,
    pub confined_roaming: bool,
    pub idol: Option<ItemId>,
    pub has_been_advanced: bool,
    pub spawn_area: Rectangle2D,
    pub randomize_type: bool,
    pub kills: i32,
    pub active: bool,
    pub ty: i32,
    pub creatures: Vec<MobileId>,
    pub red_skulls: Vec<ItemId>,
    pub white_skulls: Vec<ItemId>,
    pub platform: Option<ItemId>,
    pub altar: Option<ItemId>,
    pub expire_delay: TimeSpan,
    pub expire_time: DateTime,
    pub champion: Option<MobileId>,
    pub restart_delay: TimeSpan,
    pub restart_time: DateTime,
    pub minions: Vec<MobileId>,
    // not serialized
    pub eject_location: Point3D,
    pub eject_map: Option<MapId>,
    pub ready_to_activate: bool,
    pub region: Option<RegionId>,
    pub timer: TimerHandle,
    pub restart_timer: TimerHandle,
}

impl Default for SpawnData {
    fn default() -> Self {
        SpawnData {
            level: 0,
            activated_by_proximity: false,
            next_proximity_time: DateTime::MIN,
            max_level: 0,
            activated_by_valor: false,
            damage_entries: IndexMap::new(),
            confined_roaming: true,
            idol: None,
            has_been_advanced: false,
            spawn_area: Rectangle2D::EMPTY,
            randomize_type: false,
            kills: 0,
            active: false,
            ty: 0,
            creatures: Vec::new(),
            red_skulls: Vec::new(),
            white_skulls: Vec::new(),
            platform: None,
            altar: None,
            expire_delay: TimeSpan::ZERO,
            expire_time: DateTime::MIN,
            champion: None,
            restart_delay: TimeSpan::ZERO,
            restart_time: DateTime::MIN,
            minions: Vec::new(),
            eject_location: Point3D::ZERO,
            eject_map: None,
            ready_to_activate: false,
            region: None,
            timer: TimerHandle::default(),
            restart_timer: TimerHandle::default(),
        }
    }
}

pub fn data(w: &World, id: ItemId) -> Option<&SpawnData> {
    w.item(id)?.ext.get::<SpawnData>()
}

pub fn data_mut(w: &mut World, id: ItemId) -> Option<&mut SpawnData> {
    w.item_mut(id)?.ext.get_mut::<SpawnData>()
}

fn ty_of(w: &World, id: ItemId) -> Option<&'static ItemType> {
    w.item(id).map(|i| i.ty)
}

fn loc(w: &World, id: ItemId) -> Point3D {
    w.item(id).map_or(Point3D::ZERO, |i| i.location)
}

fn map_of(w: &World, id: ItemId) -> Option<MapId> {
    w.item(id).and_then(|i| i.map)
}

fn is_deleted(w: &World, id: ItemId) -> bool {
    w.item(id).is_none_or(|i| i.deleted())
}

fn mobile_deleted(w: &World, m: MobileId) -> bool {
    w.mobile(m).is_none_or(|x| x.deleted)
}

fn delete_item(w: &mut World, id: ItemId) {
    if let Some(t) = ty_of(w, id) {
        (t.delete)(w, id);
    }
}

fn delete_mobile(w: &mut World, m: MobileId) {
    if let Some(t) = w.mobile(m).map(|x| x.ty) {
        (t.delete)(w, m);
    }
}

// ---- virtuals ----

/// `BroadcastMessage`.
pub const BROADCAST_MESSAGE: &str = "The Champion has sensed your presence!  Beware its wrath!";

pub fn proximity_spawn(w: &World, id: ItemId) -> bool {
    ty_of(w, id).is_some_and(|t| kind(t) == Kind::Dungeon)
}

pub fn always_active(_w: &World, _id: ItemId) -> bool {
    false
}

pub fn has_star_room_gate(w: &World, id: ItemId) -> bool {
    ty_of(w, id).is_none_or(|t| kind(t) != Kind::LostLands)
}

/// `MaxKills`.
pub fn max_kills(w: &World, id: ItemId) -> i32 {
    match data(w, id).map_or(0, |d| d.level) {
        16.. => 16,
        12.. => 32,
        8.. => 64,
        4.. => 128,
        _ => 256,
    }
}

/// `MaxSpawn`.
pub fn max_spawn(w: &World, id: ItemId) -> i32 {
    250 - get_sub_level(w, id) * 40
}

/// `GetSubLevel()`.
pub fn get_sub_level(w: &World, id: ItemId) -> i32 {
    let level = data(w, id).map_or(0, |d| d.level);
    if level <= 4 {
        0
    } else if level <= 8 {
        1
    } else if level <= 12 {
        2
    } else {
        3
    }
}

// ---- construction ----

/// `ChampionSpawn()`: `base(0xBD2)`, `Movable = false; Visible = false`, the platform, altar and idol, and
/// `Timer.StartTimer(TimeSpan.Zero, SetInitialSpawnArea)`.
pub fn new_spawn(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = ops::new_item_with_id(w, ty, 0xBD2);
    ops::set_movable(w, id, false);
    ops::set_visible(w, id, false);
    let d = SpawnData {
        expire_delay: TimeSpan::from_mins(30),
        restart_delay: TimeSpan::from_mins(30),
        ..SpawnData::default()
    };
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(d);
    }
    let platform = super::items::new_platform(w, id);
    let altar = super::items::new_altar(w, id);
    let idol = new_idol(w, id);
    if let Some(d) = data_mut(w, id) {
        d.platform = Some(platform);
        d.altar = Some(altar);
        d.idol = Some(idol);
    }
    timer::start(w, TimeSpan::ZERO, TimeSpan::ZERO, 1, move |w| {
        set_initial_spawn_area(w, id)
    });
    id
}

/// `new DungeonChampionSpawn()`: `CannedEvilTimer.AddSpawn(this)`.
pub fn new_dungeon_spawn(w: &mut World) -> ItemId {
    let id = new_spawn(w, &DUNGEON_CHAMPION_SPAWN);
    super::timer::add_spawn(w, id);
    id
}

/// `new LLChampionSpawn()`.
pub fn new_ll_spawn(w: &mut World) -> ItemId {
    let id = new_spawn(w, &LL_CHAMPION_SPAWN);
    super::timer::add_spawn(w, id);
    id
}

/// `SetInitialSpawnArea()`: the previous default used to be 24.
pub fn set_initial_spawn_area(w: &mut World, id: ItemId) {
    let l = loc(w, id);
    set_spawn_area(
        w,
        id,
        Rectangle2D::from_points(
            Point2D::new(l.x - 24, l.y - 24),
            Point2D::new(l.x + 24, l.y + 24),
        ),
    );
}

/// `SpawnArea = value` (`OnSpawnAreaChanged`: `UpdateRegion()`).
pub fn set_spawn_area(w: &mut World, id: ItemId, value: Rectangle2D) {
    let changed = data_mut(w, id).is_some_and(|d| {
        let c = d.spawn_area != value;
        d.spawn_area = value;
        c
    });
    if changed {
        ops::invalidate_properties(w, id);
        update_region(w, id);
    }
}

// ---- the region ----

/// `UpdateRegion()`.
pub fn update_region(w: &mut World, id: ItemId) {
    if let Some(r) = data_mut(w, id).and_then(|d| d.region.take()) {
        region::unregister(w, r);
        region::delete_region(w, r);
    }
    let map = map_of(w, id);
    if !is_deleted(w, id) && map.is_some_and(|m| m != MapId::INTERNAL) {
        let r = new_region(w, id);
        if let Some(d) = data_mut(w, id) {
            d.region = r;
        }
        if let Some(r) = r {
            region::register(w, r);
        }
    }
}

/// `GetRegion()`: `new ChampionSpawnRegion(this)`, `base(null, spawn.Map, Find(spawn.Location, spawn.Map), SpawnArea)`.
fn new_region(w: &mut World, spawn: ItemId) -> Option<RegionId> {
    let map = map_of(w, spawn)?;
    let parent = region::find(w, loc(w, spawn), Some(map));
    let area = data(w, spawn)?.spawn_area;
    let r = region::new_region(
        w,
        &CHAMPION_SPAWN_REGION,
        None,
        map,
        Some(parent),
        None,
        region::convert_all_to_3d(&[area]),
    );
    muo_content::regions::base_region::init_data(w, r);
    if let Some(reg) = w.region_mut(r) {
        reg.ext.insert(RegionSpawn { spawn });
    }
    Some(r)
}

/// `ChampionSpawnRegion.Spawn`.
#[derive(Copy, Clone)]
pub struct RegionSpawn {
    pub spawn: ItemId,
}

/// `Region.GetRegion<ChampionSpawnRegion>()?.Spawn` for the region `r`.
pub fn spawn_of_region(w: &World, r: RegionId) -> Option<ItemId> {
    let reg = region::get_region_of_type(w, r, &CHAMPION_SPAWN_REGION)?;
    w.region(reg)?.ext.get::<RegionSpawn>().map(|s| s.spawn)
}

/// `ChampionSpawn.FindSpawnOf(creature)`: the spawn the creature fights for.
pub fn find_spawn_of(w: &World, creature: MobileId) -> Option<ItemId> {
    let m = w.mobile(creature)?;
    let r = region::find(w, m.location, m.map);
    let spawn = spawn_of_region(w, r)?;
    let d = data(w, spawn)?;
    (!is_deleted(w, spawn)
        && (d.champion == Some(creature)
            || d.minions.contains(&creature)
            || d.creatures.contains(&creature)))
    .then_some(spawn)
}

/// `AddMinion(minion)`.
pub fn add_minion(w: &mut World, spawn: ItemId, minion: MobileId) {
    if let Some(d) = data_mut(w, spawn) {
        d.minions.push(minion);
    }
}

fn eject_ok(w: &World, spawn: ItemId) -> bool {
    data(w, spawn).is_some_and(|d| d.eject_location != Point3D::ZERO && d.eject_map.is_some())
}

fn region_on_enter(w: &mut World, r: RegionId, m: MobileId) {
    let Some(spawn) = w
        .region(r)
        .and_then(|x| x.ext.get::<RegionSpawn>())
        .map(|s| s.spawn)
    else {
        return;
    };
    if !muo_content::mx::player(w, m)
        || muo_content::mx::access_level(w, m) != muo_server::mobile::AccessLevel::Player
        || data(w, spawn).is_some_and(|d| d.active)
    {
        return;
    }
    if data(w, spawn).is_some_and(|d| d.ready_to_activate) {
        start(w, spawn);
        return;
    }
    let Some(d) = data(w, spawn) else { return };
    if !proximity_spawn(w, spawn) || d.activated_by_proximity || w.now < d.next_proximity_time {
        return;
    }
    let scope = w.region(r).and_then(|x| x.parent).unwrap_or(r);
    let players = region::get_players(w, scope);
    let mut addresses: Vec<std::net::IpAddr> = Vec::new();
    for &p in &players {
        let young = muo_content::hooks::pet_player_hooks(w)
            .and_then(|h| h.young)
            .is_some_and(|f| f(w, p));
        let ns = w.mobile(p).and_then(|x| x.net_state);
        if muo_content::mx::access_level(w, p) == muo_server::mobile::AccessLevel::Player
            && !young
            && let Some(a) = ns
                .and_then(|n| w.net_states.get(&n))
                .and_then(|n| n.address)
            && !addresses.contains(&a)
        {
            addresses.push(a);
        }
    }
    if addresses.len() >= 15 {
        for &p in &players {
            muo_server::mobile::part2::messages::send_message_hue(w, p, 0x20, BROADCAST_MESSAGE);
        }
        if let Some(d) = data_mut(w, spawn) {
            d.activated_by_proximity = true;
        }
        begin_restart(w, spawn, TimeSpan::from_mins(5));
    }
}

fn region_on_move_into(
    w: &mut World,
    r: RegionId,
    m: MobileId,
    d: muo_primitives::Direction,
    new_location: Point3D,
    old_location: Point3D,
) -> bool {
    use muo_content::regions::base_region::BASE_REGION;
    if !(BASE_REGION.on_move_into)(w, r, m, d, new_location, old_location) {
        return false;
    }
    if !muo_content::mx::player(w, m) {
        return true;
    }
    let young = muo_content::hooks::pet_player_hooks(w)
        .and_then(|h| h.young)
        .is_some_and(|f| f(w, m));
    if young {
        muo_content::mx::send_message(w, m, "You decide against going here because of the danger.");
        return false;
    }
    if !muo_content::mx::alive(w, m) {
        muo_content::mx::send_message(
            w,
            m,
            "A magical force prevents ghosts from entering this region.",
        );
        return false;
    }
    true
}

fn region_on_before_death(w: &mut World, r: RegionId, m: MobileId) -> bool {
    if let Some(p) = w.region(r).and_then(|x| x.parent) {
        let ty = w.region(p).map(|x| x.ty);
        if let Some(ty) = ty
            && !(ty.on_before_death)(w, p, m)
        {
            return false;
        }
    }
    if muo_content::mx::player(w, m) {
        // Give them 5 minutes to resurrect, then they are booted.
        muo_content::mx::send_message(
            w,
            m,
            "A magical force encompasses you, attempting to force you out of the area.",
        );
        timer::start(w, TimeSpan::from_mins(5), TimeSpan::ZERO, 1, move |w| {
            eject_tick(w, r, m)
        });
    }
    true
}

fn eject_tick(w: &mut World, r: RegionId, from: MobileId) {
    let Some(spawn) = w
        .region(r)
        .and_then(|x| x.ext.get::<RegionSpawn>())
        .map(|s| s.spawn)
    else {
        return;
    };
    // See if they are dead, or logged out!
    if is_deleted(w, spawn) || !eject_ok(w, spawn) || muo_content::mx::alive(w, from) {
        return;
    }
    let (eloc, emap) = match data(w, spawn) {
        Some(d) => (d.eject_location, d.eject_map),
        None => return,
    };
    let Some(mob) = w.mobile(from) else { return };
    if mob.net_state.is_some() {
        if region::is_part_of(w, mob.region, r)
            && let Some(map) = emap
        {
            muo_server::mobile::base::move_to_world(w, from, eloc, map);
            muo_content::mx::send_message(w, from, "A magical force forces you out of the area.");
        }
    } else {
        let lr = region::find(w, mob.logout_location, mob.logout_map);
        if region::is_part_of(w, lr, r)
            && let Some(m) = w.mobile_mut(from)
        {
            m.logout_location = eloc;
            m.logout_map = emap;
        }
    }
}

/// `ChampionSpawnRegion`.
pub static CHAMPION_SPAWN_REGION: RegionType = RegionType {
    name: "ChampionSpawnRegion",
    parent: Some(&muo_content::regions::base_region::BASE_REGION),
    allow_housing: |_, _, _, _| false,
    // This is a guesstimate. TODO: verify & get exact values (OSI testing: at 2 red skulls, light = 0x3 ...).
    alter_light_level: |w, r, m, global, personal| {
        (muo_content::regions::base_region::BASE_REGION.alter_light_level)(
            w, r, m, global, personal,
        );
        let level = w
            .region(r)
            .and_then(|x| x.ext.get::<RegionSpawn>())
            .and_then(|s| data(w, s.spawn))
            .map_or(0, |d| d.level);
        *global = (*global).max(1 + level);
    },
    on_enter: region_on_enter,
    on_move_into: region_on_move_into,
    on_before_death: region_on_before_death,
    ..muo_content::regions::base_region::BASE_REGION
};

// ---- level, skulls ----

/// `Level` getter.
pub fn level(w: &World, id: ItemId) -> i32 {
    data(w, id).map_or(0, |d| d.level)
}

/// `Level = value`.
pub fn set_level(w: &mut World, id: ItemId, value: i32) {
    let reds = data(w, id)
        .map(|d| d.red_skulls.clone())
        .unwrap_or_default();
    let mut reds = reds;
    let mut i = reds.len() as i32 - 1;
    while i >= value {
        delete_item(w, reds[i as usize]);
        reds.remove(i as usize);
        i -= 1;
    }
    let map = map_of(w, id);
    let mut i = reds.len() as i32;
    while i < value.min(16) {
        let skull = make_plain_item(w, 0x1854);
        if let Some(skull) = skull {
            plain_skull(w, skull, 0x26);
            let p = red_skull_location(w, id, i);
            ops::move_to_world_map(w, skull, p, map);
            reds.push(skull);
        }
        i += 1;
    }
    if let Some(d) = data_mut(w, id) {
        d.red_skulls = reds;
        d.level = value;
    }
    ops::invalidate_properties(w, id);
}

/// `new Item(0x1854)`: a plain `Item` is not constructible by name (it has no parameterless `[Constructible]`).
fn make_plain_item(w: &mut World, graphic: i32) -> Option<ItemId> {
    Some(ops::new_item_with_id(w, &ITEM_BASE, graphic))
}

/// `{ Hue = hue, Movable = false, Light = LightType.Circle150 }`.
fn plain_skull(w: &mut World, skull: ItemId, hue: i32) {
    if hue != 0 {
        crate::util::hue(w, skull, hue);
    }
    ops::set_movable(w, skull, false);
    ops::set_light(w, skull, LightType::Circle150);
}

/// `GetRedSkullLocation(index)`.
pub fn red_skull_location(w: &World, id: ItemId, index: i32) -> Point3D {
    let (x, y) = if index < 5 {
        (index - 2, -2)
    } else if index < 9 {
        (2, index - 6)
    } else if index < 13 {
        (10 - index, 2)
    } else {
        (-2, 14 - index)
    };
    let l = loc(w, id);
    Point3D::new(l.x + x, l.y + y, l.z - 15)
}

/// `GetWhiteSkullLocation(index)`.
pub fn white_skull_location(w: &World, id: ItemId, index: i32) -> Point3D {
    let (x, y) = match index {
        1 => (1, -1),
        2 => (1, 1),
        3 => (-1, 1),
        _ => (-1, -1),
    };
    let l = loc(w, id);
    Point3D::new(l.x + x, l.y + y, l.z - 15)
}

/// `SetWhiteSkullCount(val)`.
pub fn set_white_skull_count(w: &mut World, id: ItemId, val: i32) {
    let mut whites = data(w, id)
        .map(|d| d.white_skulls.clone())
        .unwrap_or_default();
    let mut i = whites.len() as i32 - 1;
    while i >= val {
        delete_item(w, whites[i as usize]);
        whites.remove(i as usize);
        i -= 1;
    }
    let map = map_of(w, id);
    let mut i = whites.len() as i32;
    while i < val {
        let skull = ops::new_item_with_id(w, &ITEM_BASE, 0x1854);
        plain_skull(w, skull, 0);
        let p = white_skull_location(w, id, i);
        ops::move_to_world_map(w, skull, p, map);
        whites.push(skull);
        muo_server::effects::play_sound(w, p, map, 0x29);
        muo_server::effects::send_location_effect(
            w,
            Point3D::new(p.x + 1, p.y + 1, p.z),
            map,
            0x3728,
            10,
            10,
            0,
            0,
        );
        i += 1;
    }
    if let Some(d) = data_mut(w, id) {
        d.white_skulls = whites;
    }
}

/// `Kills = value` (`OnKillsChanged`).
pub fn set_kills(w: &mut World, id: ItemId, value: i32) {
    let changed = data_mut(w, id).is_some_and(|d| {
        let c = d.kills != value;
        d.kills = value;
        c
    });
    if changed {
        ops::invalidate_properties(w, id);
        let n = f64::from(value) / f64::from(max_kills(w, id));
        let p = (n * 100.0) as i32;
        if p < 90 {
            set_white_skull_count(w, id, p / 20);
        }
    }
}

// ---- start, stop, restart ----

/// `Active` setter.
pub fn set_active(w: &mut World, id: ItemId, value: bool) {
    if value {
        start(w, id);
    } else {
        stop(w, id);
    }
    ops::invalidate_properties(w, id);
}

fn set_hue_of(w: &mut World, id: Option<ItemId>, hue: i32) {
    if let Some(id) = id {
        crate::util::hue(w, id, hue);
    }
}

/// `Start()`.
pub fn start(w: &mut World, id: ItemId) {
    if data(w, id).is_none_or(|d| d.active) || is_deleted(w, id) {
        return;
    }
    if data(w, id).is_some_and(|d| d.randomize_type) {
        let t = rnd::below(w, 5);
        let new = match t {
            0 => Some(ChampionSpawnType::VerminHorde),
            1 => Some(ChampionSpawnType::UnholyTerror),
            2 => Some(ChampionSpawnType::ColdBlood),
            3 => Some(ChampionSpawnType::Abyss),
            4 => Some(ChampionSpawnType::Arachnid),
            _ => None,
        };
        if let Some(t) = new {
            set_type(w, id, t as i32);
        }
    }
    let max_level = 16 + rnd::below(w, 3);
    let (altar, platform, expire_delay, champion) = {
        let Some(d) = data_mut(w, id) else { return };
        d.active = true;
        d.ready_to_activate = false;
        d.has_been_advanced = false;
        d.max_level = max_level.clamp(0, 18);
        (d.altar, d.platform, d.expire_delay, d.champion)
    };
    let mut t = data(w, id).map(|d| d.timer).unwrap_or_default();
    t.cancel(w);
    let h = timer::start(
        w,
        TimeSpan::from_secs(1),
        TimeSpan::from_secs(1),
        0,
        move |w| on_slice(w, id),
    );
    let mut rt = data(w, id).map(|d| d.restart_timer).unwrap_or_default();
    rt.cancel(w);
    if let Some(d) = data_mut(w, id) {
        d.timer = h;
        d.restart_timer = rt;
    }
    if altar.is_some() {
        set_hue_of(w, altar, if champion.is_some() { 0x26 } else { 0 });
    }
    set_hue_of(w, platform, 0x452);
    let now = w.now;
    if let Some(d) = data_mut(w, id) {
        d.expire_time = now + expire_delay;
    }
}

/// `Type = value` (`InvalidateProperties`).
pub fn set_type(w: &mut World, id: ItemId, value: i32) {
    if let Some(d) = data_mut(w, id) {
        d.ty = value;
    }
    ops::invalidate_properties(w, id);
}

/// `Stop()`.
pub fn stop(w: &mut World, id: ItemId) {
    if data(w, id).is_none_or(|d| !d.active) || is_deleted(w, id) {
        return;
    }
    let (altar, platform) = {
        let Some(d) = data_mut(w, id) else { return };
        d.active = false;
        d.activated_by_valor = false;
        d.has_been_advanced = false;
        d.max_level = 0;
        (d.altar, d.platform)
    };
    let mut t = data(w, id).map(|d| d.timer).unwrap_or_default();
    t.cancel(w);
    let mut rt = data(w, id).map(|d| d.restart_timer).unwrap_or_default();
    rt.cancel(w);
    if let Some(d) = data_mut(w, id) {
        d.timer = t;
        d.restart_timer = rt;
    }
    set_hue_of(w, altar, 0);
    set_hue_of(w, platform, 0x497);

    if always_active(w, id) {
        let delay = data(w, id).map_or(TimeSpan::ZERO, |d| d.restart_delay);
        begin_restart(w, id, delay);
    } else if data(w, id).is_some_and(|d| d.activated_by_proximity) {
        let now = w.now;
        if let Some(d) = data_mut(w, id) {
            d.activated_by_proximity = false;
            d.next_proximity_time = now + TimeSpan::from_hours(6);
        }
    }
    timer::start(w, TimeSpan::from_mins(10), TimeSpan::ZERO, 1, move |w| {
        expire_creatures(w, id)
    });
}

/// `BeginRestart(ts)`.
pub fn begin_restart(w: &mut World, id: ItemId, ts: TimeSpan) {
    let now = w.now;
    let mut rt = data(w, id).map(|d| d.restart_timer).unwrap_or_default();
    rt.cancel(w);
    let h = timer::start(w, ts, TimeSpan::ZERO, 1, move |w| end_restart(w, id));
    if let Some(d) = data_mut(w, id) {
        d.restart_time = now + ts;
        d.restart_timer = h;
    }
}

/// `EndRestart()`.
pub fn end_restart(w: &mut World, id: ItemId) {
    if let Some(d) = data_mut(w, id) {
        d.has_been_advanced = false;
        d.ready_to_activate = true;
    }
}

// ---- the slice ----

/// `CreateRandomSoT(isFel)` and `CreateRandomPS()` and `GiveScrollTo(killer, scroll, isFelucca)`.
fn give_scroll_to(w: &mut World, killer: MobileId, scroll: ItemId, is_felucca: bool, is_sot: bool) {
    if is_sot {
        muo_content::mx::msg(w, killer, 1094936); // You have received a Scroll of Transcendence!
    } else {
        muo_content::mx::msg(w, killer, 1049524); // You have received a scroll of power!
    }
    if !is_felucca {
        muo_server::mobile::part2::items::add_to_backpack(w, killer, scroll);
        return;
    }
    let corpse = w
        .mobile(killer)
        .and_then(|m| m.corpse)
        .filter(|c| !is_deleted(w, *c));
    if muo_content::mx::alive(w, killer) || corpse.is_none() {
        muo_server::mobile::part2::items::add_to_backpack(w, killer, scroll);
    } else if let Some(c) = corpse {
        muo_server::item::container::drop_item_default(w, c, scroll);
    }
    // Justice reward
    let hooks = muo_mobiles::npcs::special::hooks::get(w);
    let (protector, check_region, level) = (
        hooks.and_then(|h| h.justice_protector),
        hooks.and_then(|h| h.justice_check_map_region),
        hooks.and_then(|h| h.justice_level),
    );
    let Some(prot) = protector.and_then(|f| f(w, killer)) else {
        return;
    };
    if w.mobile(prot).map(|m| m.map) != w.mobile(killer).map(|m| m.map)
        || muo_server::mobile::base::murderer(w, prot)
        || muo_server::mobile::base::criminal(w, prot)
        || !check_region.is_some_and(|f| f(w, killer, prot))
    {
        return;
    }
    let chance = match level.map_or(0, |f| f(w, prot)) {
        1 => 60,  // Seeker
        2 => 80,  // Follower
        3 => 100, // Knight
        _ => 0,
    };
    if chance > 0
        && chance > rnd::below(w, 100)
        && let Some(dupe) = muo_items::special::scrolls::duplicate(w, scroll)
    {
        muo_content::mx::msg(w, prot, 1049368); // You have been rewarded for your dedication to Justice!
        muo_server::mobile::part2::items::add_to_backpack(w, prot, dupe);
    }
}

/// `OnSlice()`.
pub fn on_slice(w: &mut World, id: ItemId) {
    if data(w, id).is_none_or(|d| !d.active) || is_deleted(w, id) {
        return;
    }
    if let Some(champion) = data(w, id).and_then(|d| d.champion) {
        if mobile_deleted(w, champion) {
            register_damage_to(w, id, champion);
            if w.core_ml() {
                let art = champion_artifact(w, id, champion);
                award_artifact(w, id, art);
            }
            if let Some(d) = data_mut(w, id) {
                d.damage_entries.clear();
            }
            let (platform, altar) = data(w, id)
                .map(|d| (d.platform, d.altar))
                .unwrap_or((None, None));
            set_hue_of(w, platform, 0x497);
            if let Some(altar) = altar {
                crate::util::hue(w, altar, 0);
                if has_star_room_gate(w, id)
                    && (!w.core_ml() || map_of(w, id) == Some(MapId::FELUCCA))
                {
                    // `new StarRoomGate(_altar.Location, _altar.Map, true)`
                    if let Some(f) = w
                        .resources
                        .get::<super::CannedEvilHooks>()
                        .and_then(|h| h.star_room_gate)
                    {
                        let (p, m) = (loc(w, altar), map_of(w, altar));
                        f(w, p, m);
                    }
                }
            }
            if let Some(d) = data_mut(w, id) {
                d.champion = None;
            }
            stop(w, id);
        }
        return;
    }

    let kills_before = data(w, id).map_or(0, |d| d.kills);
    let mut kills = kills_before;
    let mut i = 0;
    while let Some(m) = data(w, id).and_then(|d| d.creatures.get(i).copied()) {
        if !mobile_deleted(w, m) {
            i += 1;
            continue;
        }
        if let Some(c) = w
            .mobile(m)
            .and_then(|x| x.corpse)
            .filter(|c| !is_deleted(w, *c))
        {
            muo_content::items::corpse::begin_decay(w, c, TimeSpan::from_mins(1));
        }
        if let Some(d) = data_mut(w, id) {
            d.creatures.remove(i);
        }
        kills += 1;
        set_kills_raw(w, id, kills);

        let mut killer = muo_server::mobile::part2::damage::find_most_recent_damager(w, m, false);
        register_damage_to(w, id, m);
        if let Some(k) = killer
            && cd(w, k).is_some()
        {
            killer = muo_content::creature::control::get_master(w, k);
        }
        let Some(pm) = killer.filter(|k| pmob::data(w, *k).is_some()) else {
            continue;
        };

        let mob_sub_level = get_sub_level_for(w, id, m) + 1;
        if mob_sub_level >= 0 {
            let points = mob_sub_level * 40;
            // VirtueSystem.Award(pm, VirtueName.Valor, points, ref gainedPath)
            if let Some(gained_path) =
                crate::virtues::award(w, pm, crate::virtues::VirtueName::Valor, points)
            {
                if gained_path {
                    muo_content::mx::msg(w, pm, 1054032); // You have gained a path in Valor!
                } else {
                    muo_content::mx::msg(w, pm, 1054030); // You have gained in Valor!
                }
            }
            let ty = data(w, id).map_or(0, |d| d.ty);
            if let Some(t) = ChampionSpawnType::from_i32(ty) {
                super::titles::award(w, pm, t, mob_sub_level);
            }
        }

        if !w.core_ml() {
            continue;
        }
        let map = map_of(w, id);
        let is_fel = map == Some(MapId::FELUCCA);
        let map_chance = if is_fel {
            0.001
        } else if map == Some(MapId::ILSHENAR) || map == Some(MapId::TOKUNO) {
            0.0015
        } else {
            // Nothing drops in Trammel, Malas or TerMur
            continue;
        };
        if rnd::double(w) >= map_chance {
            continue;
        }
        if !is_fel || rnd::bool(w) {
            // CreateRandomSoT(isFel): `ScrollofTranscendence.CreateRandom(min, max)`
            let (min, max) = if is_fel { (1, 5) } else { (6, 10) };
            if let Some(f) = w
                .resources
                .get::<super::CannedEvilHooks>()
                .and_then(|h| h.create_random_sot)
                && let Some(s) = f(w, min, max)
            {
                give_scroll_to(w, pm, s, is_fel, true);
            }
        } else if let Some(f) = w
            .resources
            .get::<super::CannedEvilHooks>()
            .and_then(|h| h.create_random_power_scroll)
        {
            // CreateRandomPS(): `PowerScroll.CreateRandomNoCraft(5, 5)`
            if let Some(s) = f(w, 5, 5) {
                give_scroll_to(w, pm, s, true, false);
            }
        }
    }

    // Only really needed once.
    let _ = kills_before;
    if data(w, id).map_or(0, |d| d.kills) > kills_before {
        ops::invalidate_properties(w, id);
    }

    let n = f64::from(data(w, id).map_or(0, |d| d.kills)) / f64::from(max_kills(w, id));
    let p = (n * 100.0) as i32;
    if p >= 99 {
        advance_level(w, id);
    } else if p > 0 {
        set_white_skull_count(w, id, p / 20);
    }

    if data(w, id).is_some_and(|d| w.now >= d.expire_time) {
        expire(w, id);
    }
    respawn(w, id);
}

/// `++_kills` on the field (the loop uses the field, so `OnKillsChanged` does not run).
fn set_kills_raw(w: &mut World, id: ItemId, value: i32) {
    if let Some(d) = data_mut(w, id) {
        d.kills = value;
    }
}

/// `(Champion as BaseChampion)?.GetArtifact()`.
fn champion_artifact(w: &mut World, _spawn: ItemId, champion: MobileId) -> Option<ItemId> {
    let f = w
        .resources
        .get::<super::CannedEvilHooks>()
        .and_then(|h| h.champion_artifact)?;
    f(w, champion)
}

/// `AdvanceLevel()`.
pub fn advance_level(w: &mut World, id: ItemId) {
    let now = w.now;
    let (delay, level, max_level, altar) = match data(w, id) {
        Some(d) => (d.expire_delay, d.level, d.max_level, d.altar),
        None => return,
    };
    if let Some(d) = data_mut(w, id) {
        d.expire_time = now + delay;
    }
    if level < max_level {
        set_kills_raw(w, id, 0);
        set_level(w, id, level + 1);
        ops::invalidate_properties(w, id);
        set_white_skull_count(w, id, 0);
        if let Some(a) = altar {
            let (p, m) = (loc(w, a), map_of(w, a));
            muo_server::effects::play_sound(w, p, m, 0x29);
            muo_server::effects::send_location_effect(
                w,
                Point3D::new(p.x + 1, p.y + 1, p.z),
                m,
                0x3728,
                10,
                10,
                0,
                0,
            );
        }
    } else {
        spawn_champion(w, id);
    }
}

/// The `Home`/`RangeHome` block shared by `SpawnChampion` and `Respawn`.
fn set_home_for(w: &mut World, id: ItemId, bc: MobileId) {
    let Some(d) = data(w, id) else { return };
    let (confined, area) = (d.confined_roaming, d.spawn_area);
    let (sloc, smap) = (loc(w, id), map_of(w, id));
    let Some(m) = w.mobile(bc) else { return };
    let (bl, bmap) = (m.location, m.map);
    if confined {
        if let Some(c) = cd_mut(w, bc) {
            c.home = sloc;
            c.home_map = smap;
            c.range_home = (area.width() / 2).min(area.height() / 2);
        }
    } else {
        let dist = |p: Point2D| f64::from((bl.x - p.x).pow(2) + (bl.y - p.y).pow(2)).sqrt();
        let x_wall1 = Point2D::new(area.x(), bl.y);
        let x_wall2 = Point2D::new(area.x() + area.width(), bl.y);
        let y_wall1 = Point2D::new(bl.x, area.y());
        let y_wall2 = Point2D::new(bl.x, area.y() + area.height());
        let min_x = dist(x_wall1).min(dist(x_wall2));
        let min_y = dist(y_wall1).min(dist(y_wall2));
        if let Some(c) = cd_mut(w, bc) {
            c.home = bl;
            c.home_map = bmap;
            c.range_home = min_x.min(min_y) as i32;
        }
    }
}

/// `SpawnChampion()`.
pub fn spawn_champion(w: &mut World, id: ItemId) {
    let (altar, platform) = data(w, id)
        .map(|d| (d.altar, d.platform))
        .unwrap_or((None, None));
    set_hue_of(w, altar, 0x26);
    set_hue_of(w, platform, 0x452);
    set_kills_raw(w, id, 0);
    set_level(w, id, 0);
    ops::invalidate_properties(w, id);
    set_white_skull_count(w, id, 0);

    let ty = data(w, id).map_or(0, |d| d.ty);
    let champion = factory::make_mobile(w, info::get_info(ty).champion);
    if champion.is_none() {
        tracing::error!(
            "Failed to spawn champion \"{}\" at {:?}.",
            info::get_info(ty).champion,
            loc(w, id)
        );
    }
    if let Some(d) = data_mut(w, id) {
        d.champion = champion;
    }
    let Some(champion) = champion else { return };
    let l = loc(w, id);
    if let Some(map) = map_of(w, id) {
        muo_server::mobile::base::move_to_world(w, champion, Point3D::new(l.x, l.y, l.z - 15), map);
    }
    if cd(w, champion).is_some() {
        set_home_for(w, id, champion);
    } else {
        tracing::error!("Champion Spawn is not inherited from BaseCreature");
    }
}

/// `Respawn()`.
pub fn respawn(w: &mut World, id: ItemId) {
    if data(w, id).is_none_or(|d| !d.active || d.champion.is_some()) || is_deleted(w, id) {
        return;
    }
    loop {
        let count = data(w, id).map_or(0, |d| d.creatures.len() as i32);
        if count >= max_spawn(w, id) {
            return;
        }
        let Some(m) = spawn_one(w, id) else { return };
        let spawn_loc = get_spawn_location(w, id);
        let map = map_of(w, id);
        // Allow creatures to turn into Paragons at Ilshenar champions.
        if let Some(map) = map
            && let Some(t) = w.mobile(m).map(|x| x.ty)
        {
            (t.on_before_spawn)(w, m, spawn_loc, map);
        }
        if let Some(d) = data_mut(w, id) {
            d.creatures.push(m);
        }
        if let Some(map) = map {
            muo_server::mobile::base::move_to_world(w, m, spawn_loc, map);
        }
        if let Some(c) = cd_mut(w, m) {
            c.tamable_raw = false;
            set_home_for(w, id, m);
        }
    }
}

/// `Spawn()`.
pub fn spawn_one(w: &mut World, id: ItemId) -> Option<MobileId> {
    let ty = data(w, id)?.ty;
    let v = get_sub_level(w, id);
    if !(0..4).contains(&v) {
        return None;
    }
    let types = info::spawn_types(w, ty, v as usize);
    // `Spawn(params Type[] types)`
    let name = types[rnd::below(w, types.len() as i32) as usize];
    let m = factory::make_mobile(w, name);
    if m.is_none() {
        tracing::error!(
            "Failed to spawn minion \"{name}\" for champion at {:?}.",
            loc(w, id)
        );
    }
    m
}

/// `GetSubLevelfor(m)`.
pub fn get_sub_level_for(w: &World, id: ItemId, m: MobileId) -> i32 {
    let ty = data(w, id).map_or(0, |d| d.ty);
    let Some(name) = w.mobile(m).map(|x| x.ty.full_name) else {
        return -1;
    };
    let t = info::get_info(ty);
    for (i, types) in t.spawn_types.iter().enumerate() {
        let types = if i == 0 {
            info::spawn_types(w, ty, 0)
        } else {
            types
        };
        if types.contains(&name) {
            return i as i32;
        }
    }
    -1
}

/// `GetSpawnLocation()`.
pub fn get_spawn_location(w: &mut World, id: ItemId) -> Point3D {
    let Some(map) = map_of(w, id) else {
        return loc(w, id);
    };
    let area = data(w, id).map_or(Rectangle2D::EMPTY, |d| d.spawn_area);
    // Try 20 times to find a spawnable location.
    for _ in 0..20 {
        let x = rnd::from(w, area.x(), area.width());
        let y = rnd::from(w, area.y(), area.height());
        let z = w.maps.get_average_z(map, x, y);
        if muo_server::map::fit::can_spawn_mobile_at(w, map, x, y, z) {
            return Point3D::new(x, y, z);
        }
    }
    loc(w, id)
}

/// `Expire()`.
pub fn expire(w: &mut World, id: ItemId) {
    set_kills_raw(w, id, 0);
    let (whites, level, delay) = match data(w, id) {
        Some(d) => (d.white_skulls.len(), d.level, d.expire_delay),
        None => return,
    };
    if whites == 0 {
        // They didn't even get 20%, go back a level
        let mut lv = level;
        if level > 0 {
            lv -= 1;
            set_level(w, id, lv);
        }
        if !always_active(w, id) && lv == 0 {
            stop(w, id);
        }
        ops::invalidate_properties(w, id);
    } else {
        set_white_skull_count(w, id, 0);
    }
    let now = w.now;
    if let Some(d) = data_mut(w, id) {
        d.expire_time = now + delay;
    }
}

/// `ExpireCreatures()`.
pub fn expire_creatures(w: &mut World, id: ItemId) {
    if data(w, id).is_some_and(|d| !d.active && !d.ready_to_activate) && !always_active(w, id) {
        delete_creatures(w, id);
    }
}

/// `DeleteCreatures()`.
pub fn delete_creatures(w: &mut World, id: ItemId) {
    let creatures = data_mut(w, id)
        .map(|d| std::mem::take(&mut d.creatures))
        .unwrap_or_default();
    for m in creatures {
        if !muo_content::mx::player(w, m) {
            delete_mobile(w, m);
        }
    }
    let minions = data_mut(w, id)
        .map(|d| std::mem::take(&mut d.minions))
        .unwrap_or_default();
    for m in minions {
        delete_mobile(w, m);
    }
}

// ---- damage and artifacts ----

/// `RegisterDamageTo(m)`.
pub fn register_damage_to(w: &mut World, id: ItemId, m: MobileId) {
    let entries: Vec<(MobileId, i32)> = w
        .mobile(m)
        .map(|x| {
            x.damage_entries
                .iter()
                .map(|e| (e.damager, e.damage_given))
                .collect()
        })
        .unwrap_or_default();
    for (damager, given) in entries {
        let master = muo_server::mobile::base::get_damage_master(w, damager, m);
        register_damage(w, id, master.unwrap_or(damager), given);
    }
}

/// `RegisterDamage(from, amount)`.
pub fn register_damage(w: &mut World, id: ItemId, from: MobileId, amount: i32) {
    if !muo_content::mx::player(w, from) {
        return;
    }
    if let Some(d) = data_mut(w, id) {
        *d.damage_entries.entry(from).or_insert(0) += amount;
    }
}

/// `IsEligible(m, artifact)`.
fn is_eligible(w: &mut World, id: ItemId, m: MobileId, artifact: ItemId) -> bool {
    let region = data(w, id).and_then(|d| d.region);
    let Some(mob) = w.mobile(m) else { return false };
    let mreg = mob.region;
    if !(muo_content::mx::player(w, m) && muo_content::mx::alive(w, m) && Some(mreg) == region) {
        return false;
    }
    let Some(pack) = w.mobile(m).and_then(|x| x.backpack) else {
        return false;
    };
    muo_server::item::container::check_hold(w, pack, Some(m), artifact, false)
}

/// `GiveArtifact(to, artifact)`.
pub fn give_artifact(w: &mut World, to: MobileId, artifact: ItemId) -> bool {
    let Some(pack) = w.mobile(to).and_then(|x| x.backpack) else {
        return false;
    };
    if !muo_server::item::container::try_drop_item(w, pack, Some(to), artifact, false) {
        return false;
    }
    muo_content::mx::msg(w, to, 1062317); // For your valor in combating the fallen beast, a special artifact has been bestowed on you.
    true
}

/// `AwardArtifact(artifact)`.
pub fn award_artifact(w: &mut World, id: ItemId, artifact: Option<ItemId>) {
    let Some(artifact) = artifact else { return };
    let entries: Vec<(MobileId, i32)> = data(w, id)
        .map(|d| d.damage_entries.iter().map(|(k, v)| (*k, *v)).collect())
        .unwrap_or_default();
    if entries.is_empty() {
        delete_item(w, artifact);
        return;
    }
    let mut total = 0;
    let mut valid: Vec<(MobileId, i32)> = Vec::new();
    for (k, v) in entries {
        if is_eligible(w, id, k, artifact) {
            valid.push((k, v));
            total += v;
        }
    }
    loop {
        let random_damage = rnd::min_max(w, 1, total);
        let mut check = 0;
        let mut given = false;
        for idx in 0..valid.len() {
            check += valid[idx].1;
            if check > random_damage {
                if give_artifact(w, valid[idx].0, artifact) {
                    given = true;
                } else {
                    valid.remove(idx);
                }
                break;
            }
        }
        if valid.is_empty() {
            // EVERYONE has a full backpack?!@
            delete_item(w, artifact);
            break;
        }
        if given {
            break;
        }
    }
}

// ---- properties and clicks ----

fn add_name_property(_w: &World, _id: ItemId, list: &mut muo_server::property_list::PropertyList) {
    list.add_text("champion spawn");
}

fn get_properties(w: &World, id: ItemId, list: &mut muo_server::property_list::PropertyList) {
    (ITEM_BASE.get_properties)(w, id, list);
    let Some(d) = data(w, id) else { return };
    if d.active {
        list.add(1060742); // active
        let name = ChampionSpawnType::from_i32(d.ty)
            .map_or_else(|| d.ty.to_string(), |t| t.name().to_owned());
        list.add_str(1060658, &format!("Type\t{name}")); // ~1_val~: ~2_val~
        list.add_str(1060659, &format!("Level\t{}", d.level)); // ~1_val~: ~2_val~
        let mk = max_kills(w, id);
        let kill_ratio = 100.0 * (f64::from(d.kills) / f64::from(mk));
        list.add_str(
            1060660,
            &format!("Kills\t{} of {} ({:.1}%)", d.kills, mk, kill_ratio),
        ); // ~1_val~: ~2_val~
    } else {
        list.add(1060743); // inactive
    }
}

fn type_name(w: &World, id: ItemId) -> String {
    let t = data(w, id).map_or(0, |d| d.ty);
    ChampionSpawnType::from_i32(t).map_or_else(|| t.to_string(), |t| t.name().to_owned())
}

fn on_single_click(w: &mut World, id: ItemId, from: MobileId) {
    let Some(d) = data(w, id) else { return };
    let text = if d.active {
        format!(
            "{} (Active; Level: {}; Kills: {}/{})",
            type_name(w, id),
            d.level,
            d.kills,
            max_kills(w, id)
        )
    } else {
        format!("{} (Inactive)", type_name(w, id))
    };
    ops::label_to(w, id, from, &text);
}

/// `OnLocationChange(oldLoc)`.
fn on_location_change(w: &mut World, id: ItemId, old: Point3D) {
    if is_deleted(w, id) {
        return;
    }
    let l = loc(w, id);
    let Some(d) = data(w, id) else { return };
    let (platform, altar, idol) = (d.platform, d.altar, d.idol);
    let (reds, whites) = (d.red_skulls.clone(), d.white_skulls.clone());
    if let Some(p) = platform {
        ops::set_location(w, p, Point3D::new(l.x, l.y, l.z - 20));
    }
    if let Some(a) = altar {
        ops::set_location(w, a, Point3D::new(l.x, l.y, l.z - 15));
    }
    if let Some(i) = idol {
        ops::set_location(w, i, Point3D::new(l.x, l.y, l.z - 15));
    }
    for (i, s) in reds.iter().take(16).enumerate() {
        let p = red_skull_location(w, id, i as i32);
        ops::set_location(w, *s, p);
    }
    for (i, s) in whites.iter().enumerate() {
        let p = white_skull_location(w, id, i as i32);
        ops::set_location(w, *s, p);
    }
    if let Some(d) = data_mut(w, id) {
        let (dx, dy) = (l.x - old.x, l.y - old.y);
        d.spawn_area.set_x(d.spawn_area.x() + dx);
        d.spawn_area.set_y(d.spawn_area.y() + dy);
    }
    update_region(w, id);
}

/// `OnMapChange()`.
fn on_map_change(w: &mut World, id: ItemId) {
    if is_deleted(w, id) {
        return;
    }
    let map = map_of(w, id);
    let Some(d) = data(w, id) else { return };
    let mut all: Vec<ItemId> = d
        .red_skulls
        .iter()
        .chain(d.white_skulls.iter())
        .copied()
        .collect();
    all.extend(d.platform);
    all.extend(d.altar);
    all.extend(d.idol);
    for i in all {
        ops::set_map(w, i, map);
    }
    update_region(w, id);
}

/// `OnAfterDelete()`.
fn on_after_delete(w: &mut World, id: ItemId) {
    (ITEM_BASE.on_after_delete)(w, id);
    let Some(d) = data(w, id) else { return };
    let (platform, altar, idol) = (d.platform, d.altar, d.idol);
    let (reds, whites) = (d.red_skulls.clone(), d.white_skulls.clone());
    for i in [platform, altar, idol].into_iter().flatten() {
        delete_item(w, i);
    }
    for s in reds.into_iter().chain(whites) {
        delete_item(w, s);
    }
    if let Some(d) = data_mut(w, id) {
        d.red_skulls.clear();
        d.white_skulls.clear();
    }
    delete_creatures(w, id);
    if let Some(c) = data(w, id).and_then(|d| d.champion)
        && !muo_content::mx::player(w, c)
    {
        delete_mobile(w, c);
    }
    stop(w, id);
    update_region(w, id);
    if let Some(t) = ty_of(w, id)
        && kind(t) != Kind::Plain
    {
        super::timer::remove_spawn(w, id);
    }
}

// ---- save ----

const SERIALIZATION_VERSION: i32 = 12;

fn write_item_list(w: &World, out: &mut dyn GenericWriter, list: &[ItemId]) {
    let live: Vec<ItemId> = list
        .iter()
        .copied()
        .filter(|i| !is_deleted(w, *i))
        .collect();
    out.write_encoded_int(live.len() as i32);
    for i in live {
        out.write_item(w, Some(i));
    }
}

fn write_mobile_list(w: &World, out: &mut dyn GenericWriter, list: &[MobileId]) {
    let live: Vec<MobileId> = list
        .iter()
        .copied()
        .filter(|m| !mobile_deleted(w, *m))
        .collect();
    out.write_encoded_int(live.len() as i32);
    for m in live {
        out.write_mobile(w, Some(m));
    }
}

fn read_item_list(w: &World, r: &mut dyn GenericReader) -> Result<Vec<ItemId>> {
    let n = r.read_encoded_int()?;
    let mut v = Vec::new();
    for _ in 0..n.max(0) {
        v.extend(r.read_item(w)?);
    }
    Ok(v)
}

fn read_mobile_list(w: &World, r: &mut dyn GenericReader) -> Result<Vec<MobileId>> {
    let n = r.read_encoded_int()?;
    let mut v = Vec::new();
    for _ in 0..n.max(0) {
        v.extend(r.read_mobile(w)?);
    }
    Ok(v)
}

fn date(ticks: i64) -> DateTime {
    DateTime::try_from_ticks(ticks).unwrap_or(DateTime::MIN)
}

/// The generated `Serialize` (version 12).
pub fn serialize_body(w: &World, id: ItemId, out: &mut dyn GenericWriter) {
    let Some(d) = data(w, id) else { return };
    out.write_i32(SERIALIZATION_VERSION);
    out.write_i32(d.level);
    out.write_bool(d.activated_by_proximity);
    out.write_anchored_time_ticks(d.next_proximity_time.ticks());
    out.write_i32(d.max_level);
    out.write_bool(d.activated_by_valor);
    out.write_encoded_int(d.damage_entries.len() as i32);
    for (m, v) in &d.damage_entries {
        out.write_mobile(w, Some(*m));
        out.write_i32(*v);
    }
    out.write_bool(d.confined_roaming);
    out.write_item(w, d.idol);
    out.write_bool(d.has_been_advanced);
    out.write_rect2d(d.spawn_area);
    out.write_bool(d.randomize_type);
    out.write_i32(d.kills);
    out.write_bool(d.active);
    out.write_enum(4, i64::from(d.ty));
    write_mobile_list(w, out, &d.creatures);
    write_item_list(w, out, &d.red_skulls);
    write_item_list(w, out, &d.white_skulls);
    out.write_item(w, d.platform);
    out.write_item(w, d.altar);
    out.write_time_span_ticks(d.expire_delay.ticks());
    out.write_anchored_time_ticks(d.expire_time.ticks());
    out.write_mobile(w, d.champion);
    out.write_time_span_ticks(d.restart_delay.ticks());
    out.write_anchored_time_ticks(d.restart_time.ticks());
    write_mobile_list(w, out, &d.minions);
}

/// The generated `Deserialize`: versions 12, 11 and 10 (`MigrateFrom`) and the older hand-written layout.
pub fn deserialize_body(w: &mut World, id: ItemId, r: &mut dyn GenericReader) -> Result<()> {
    let version = r.read_i32()?;
    let mut d = SpawnData::default();
    match version {
        10 | 11 => {
            // V10Content / V11Content: the field order of version 12 up to `restartTime`, then (11 only) `minions`.
            read_current(w, r, &mut d, version == 10, false)?;
        }
        v if v < SERIALIZATION_VERSION => {
            // The pre-generator `Deserialize(reader, version)`.
            d.level = r.read_i32()?;
            d.activated_by_proximity = r.read_bool()?;
            d.next_proximity_time = date(r.read_delta_time_ticks(w.now.ticks())?);
            d.max_level = r.read_i32()?;
            d.activated_by_valor = r.read_bool()?;
            let entries = r.read_i32()?;
            for _ in 0..entries.max(0) {
                let m = r.read_mobile(w)?;
                let damage = r.read_i32()?;
                if let Some(m) = m {
                    d.damage_entries.insert(m, damage);
                }
            }
            d.confined_roaming = r.read_bool()?;
            d.idol = r.read_item(w)?;
            d.has_been_advanced = r.read_bool()?;
            d.spawn_area = r.read_rect2d()?;
            d.randomize_type = r.read_bool()?;
            d.kills = r.read_i32()?;
            d.active = r.read_bool()?;
            d.ty = r.read_i32()?;
            d.creatures = r.read_mobile_list(w)?;
            d.minions = Vec::new();
            d.red_skulls = r.read_item_list(w)?;
            d.white_skulls = r.read_item_list(w)?;
            d.platform = r.read_item(w)?;
            d.altar = r.read_item(w)?;
            d.expire_delay = TimeSpan::from_ticks(r.read_time_span_ticks()?);
            d.expire_time = date(r.read_delta_time_ticks(w.now.ticks())?);
            d.champion = r.read_mobile(w)?;
            d.restart_delay = TimeSpan::from_ticks(r.read_time_span_ticks()?);
            if r.read_bool()? {
                d.restart_time = date(r.read_delta_time_ticks(w.now.ticks())?);
            }
        }
        _ => read_current(w, r, &mut d, false, true)?,
    }
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(d);
    }
    // `Timer.DelayCall(AfterDeserialization)`
    timer::start(w, TimeSpan::ZERO, TimeSpan::ZERO, 1, move |w| {
        after_deserialization(w, id)
    });
    Ok(())
}

/// The version 12 field order (versions 10 and 11 share it; 10 has no `minions`).
fn read_current(
    w: &World,
    r: &mut dyn GenericReader,
    d: &mut SpawnData,
    delta_times: bool,
    minions: bool,
) -> Result<()> {
    let now = w.now.ticks();
    let time = |r: &mut dyn GenericReader| -> Result<DateTime> {
        Ok(date(if delta_times {
            r.read_delta_time_ticks(now)?
        } else {
            r.read_anchored_time_ticks()?
        }))
    };
    d.level = r.read_i32()?;
    d.activated_by_proximity = r.read_bool()?;
    d.next_proximity_time = time(r)?;
    d.max_level = r.read_i32()?;
    d.activated_by_valor = r.read_bool()?;
    let n = r.read_encoded_int()?;
    for _ in 0..n.max(0) {
        let m = r.read_mobile(w)?;
        let v = r.read_i32()?;
        if let Some(m) = m {
            d.damage_entries.insert(m, v);
        }
    }
    d.confined_roaming = r.read_bool()?;
    d.idol = r.read_item(w)?;
    d.has_been_advanced = r.read_bool()?;
    d.spawn_area = r.read_rect2d()?;
    d.randomize_type = r.read_bool()?;
    d.kills = r.read_i32()?;
    d.active = r.read_bool()?;
    d.ty = r.read_enum(4)? as i32;
    d.creatures = read_mobile_list(w, r)?;
    d.red_skulls = read_item_list(w, r)?;
    d.white_skulls = read_item_list(w, r)?;
    d.platform = r.read_item(w)?;
    d.altar = r.read_item(w)?;
    d.expire_delay = TimeSpan::from_ticks(r.read_time_span_ticks()?);
    d.expire_time = time(r)?;
    d.champion = r.read_mobile(w)?;
    d.restart_delay = TimeSpan::from_ticks(r.read_time_span_ticks()?);
    d.restart_time = time(r)?;
    if minions {
        d.minions = read_mobile_list(w, r)?;
    }
    Ok(())
}

/// `[AfterDeserialization(false)] AfterDeserialization()`, and the subclasses' (`CannedEvilTimer.AddSpawn`).
fn after_deserialization(w: &mut World, id: ItemId) {
    if is_deleted(w, id) {
        return;
    }
    let (restart_time, now) = (data(w, id).map_or(DateTime::MIN, |d| d.restart_time), w.now);
    if restart_time < now {
        begin_restart(w, id, restart_time - now);
    }
    let (complete, active) = data(w, id).map_or((false, false), |d| {
        (
            d.platform.is_some() && d.altar.is_some() && d.idol.is_some(),
            d.active,
        )
    });
    if !complete {
        delete_item(w, id);
    } else if active {
        if let Some(d) = data_mut(w, id) {
            d.active = false; // Set back to false so we can properly start
        }
        start(w, id);
    } else if always_active(w, id)
        && let Some(d) = data_mut(w, id)
    {
        d.ready_to_activate = true;
    }
    timer::start(w, TimeSpan::ZERO, TimeSpan::ZERO, 1, move |w| {
        update_region(w, id)
    });
    if ty_of(w, id).is_some_and(|t| kind(t) != Kind::Plain) {
        super::timer::add_spawn(w, id);
    }
}

fn spawn_serialize(w: &World, id: ItemId, out: &mut dyn GenericWriter) {
    (ITEM_BASE.serialize)(w, id, out);
    serialize_body(w, id, out);
}

fn spawn_deserialize(w: &mut World, id: ItemId, r: &mut dyn GenericReader) -> Result<()> {
    (ITEM_BASE.deserialize)(w, id, r)?;
    deserialize_body(w, id, r)
}

/// `[SerializationGenerator(12, false)] ChampionSpawn`.
pub static CHAMPION_SPAWN: ItemType = ItemType {
    full_name: "Server.Engines.CannedEvil.ChampionSpawn",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| new_spawn(w, &CHAMPION_SPAWN)),
    decay_time: |_, _| TimeSpan::from_secs(180),
    label_number: |_, _| 1041030, // Evil in a Can:  Don't delete me!
    add_name_property,
    get_properties,
    on_single_click,
    on_double_click: |w, id, from| {
        // `from.SendGump(new PropertiesGump(from, this))`
        if let Some(f) = w
            .resources
            .get::<super::CannedEvilHooks>()
            .and_then(|h| h.properties_gump)
        {
            f(w, from, EntityId::Item(id));
        }
    },
    on_location_change,
    on_map_change,
    on_after_delete,
    serialize: spawn_serialize,
    deserialize: spawn_deserialize,
    ..ITEM_BASE
};

/// `[SerializationGenerator(0)] DungeonChampionSpawn`.
pub static DUNGEON_CHAMPION_SPAWN: ItemType = ItemType {
    full_name: "Server.Engines.CannedEvil.DungeonChampionSpawn",
    parent: Some(&CHAMPION_SPAWN),
    construct: Some(new_dungeon_spawn),
    serialize: |w, id, out| {
        spawn_serialize(w, id, out);
        out.write_encoded_int(0);
    },
    deserialize: |w, id, r| {
        spawn_deserialize(w, id, r)?;
        r.read_encoded_int()?;
        Ok(())
    },
    ..CHAMPION_SPAWN
};

/// `[SerializationGenerator(0)] LLChampionSpawn`.
pub static LL_CHAMPION_SPAWN: ItemType = ItemType {
    full_name: "Server.Engines.CannedEvil.LLChampionSpawn",
    parent: Some(&CHAMPION_SPAWN),
    construct: Some(new_ll_spawn),
    serialize: |w, id, out| {
        spawn_serialize(w, id, out);
        out.write_encoded_int(0);
    },
    deserialize: |w, id, r| {
        spawn_deserialize(w, id, r)?;
        r.read_encoded_int()?;
        Ok(())
    },
    ..CHAMPION_SPAWN
};

// ---- IdolOfTheChampion ----

/// `IdolOfTheChampion._spawn`.
#[derive(Clone, Copy, Default)]
pub struct IdolData {
    pub spawn: Option<ItemId>,
}

/// `IdolOfTheChampion(spawn)`: `base(0x1F18)`, `Movable = false`.
pub fn new_idol(w: &mut World, spawn: ItemId) -> ItemId {
    let id = ops::new_item_with_id(w, &IDOL_OF_THE_CHAMPION, 0x1F18);
    ops::set_movable(w, id, false);
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(IdolData { spawn: Some(spawn) });
    }
    id
}

pub fn idol_spawn(w: &World, id: ItemId) -> Option<ItemId> {
    w.item(id)?.ext.get::<IdolData>()?.spawn
}

/// `IdolOfTheChampion`.
pub static IDOL_OF_THE_CHAMPION: ItemType = ItemType {
    full_name: "Server.Engines.CannedEvil.IdolOfTheChampion",
    parent: Some(&ITEM_BASE),
    // No parameterless constructor: `IdolOfTheChampion(ChampionSpawn spawn)`.
    construct: None,
    default_name: |_, _| Some("Idol of the Champion".to_owned()),
    on_after_delete: |w, id| {
        (ITEM_BASE.on_after_delete)(w, id);
        if let Some(s) = idol_spawn(w, id) {
            delete_item(w, s);
        }
    },
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        out.write_i32(0);
        out.write_item(w, idol_spawn(w, id));
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        let spawn = r.read_item(w)?;
        if let Some(i) = w.item_mut(id) {
            i.ext = Ext::new(IdolData { spawn });
        }
        // `[AfterDeserialization(false)]`: no spawn, no idol.
        timer::start(w, TimeSpan::ZERO, TimeSpan::ZERO, 1, move |w| {
            if idol_spawn(w, id).is_none() {
                delete_item(w, id);
            }
        });
        Ok(())
    },
    ..ITEM_BASE
};

pub fn register(types: &mut TypeRegistry) {
    types.register_item(&CHAMPION_SPAWN, &[]);
    types.register_item(&DUNGEON_CHAMPION_SPAWN, &[]);
    types.register_item(&LL_CHAMPION_SPAWN, &[]);
    types.register_item(&IDOL_OF_THE_CHAMPION, &[]);
}
