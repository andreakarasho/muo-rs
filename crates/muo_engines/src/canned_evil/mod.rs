//! Champion spawns (the spawn item, platform, altar, idol, skulls, the Harrower's skull
//! platform), the daily rotation, the champion title system and `[GenChamps`, `[ClearChampByTarget`,
//! `[ClearChampByRegion`.
//!
//! `PropertiesGump` (the double click of a spawn) is a `fn` hook in [`CannedEvilHooks`] (set in `special_wiring`);
//! `HarrowerGate`, `StarRoomGate` (in `gates`), the Harrower, the champion artifact and the scrolls are wired in
//! `gates::register` and `special_wiring`.

use muo_primitives::Point3D;
use muo_server::World;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::map::MapId;
use muo_server::mobile::AccessLevel;
use muo_server::target::{TargetFlags, Targeted};
use muo_server::type_registry::{DEFAULT_CALL_PRIORITY, Phase, TypeRegistry};

pub mod gates;
pub mod info;
pub mod items;
pub mod spawn;
pub mod timer;
pub mod titles;

type CreateRandomSotFn = fn(&mut World, i32, i32) -> Option<ItemId>;
type CreateRandomPowerScrollFn = fn(&mut World, i32, i32) -> Option<ItemId>;
type HarrowerSpawnFn = fn(&mut World, Point3D, Option<MapId>) -> Option<MobileId>;
/// Calls the champion engine makes into classes another crate owns.
#[derive(Default)]
pub struct CannedEvilHooks {
    /// `new StarRoomGate(location, map, true)` (a `Moongate`).
    pub star_room_gate: Option<fn(&mut World, Point3D, Option<MapId>)>,
    /// `ScrollofTranscendence.CreateRandom(min, max)`.
    pub create_random_sot: Option<CreateRandomSotFn>,
    /// `PowerScroll.CreateRandomNoCraft(min, max)`.
    pub create_random_power_scroll: Option<CreateRandomPowerScrollFn>,
    /// `(champion as BaseChampion)?.GetArtifact()`.
    pub champion_artifact: Option<fn(&mut World, MobileId) -> Option<ItemId>>,
    /// `from.SendGump(new PropertiesGump(from, entity))`.
    pub properties_gump: Option<fn(&mut World, MobileId, EntityId)>,
    /// `Harrower.CanSpawn`. Unset: true.
    pub harrower_can_spawn: Option<fn(&World) -> bool>,
    /// `Harrower.Spawn(location, map)`.
    pub harrower_spawn: Option<HarrowerSpawnFn>,
}

fn spawn_of_target(w: &World, e: EntityId) -> Option<ItemId> {
    let EntityId::Item(i) = e else { return None };
    let ty = w.item(i)?.ty;
    if ty.is(&spawn::CHAMPION_SPAWN) {
        Some(i)
    } else if ty.is(&spawn::IDOL_OF_THE_CHAMPION) {
        spawn::idol_spawn(w, i)
    } else if ty.is(&items::CHAMPION_ALTAR) || ty.is(&items::CHAMPION_PLATFORM) {
        items::spawn_ref(w, i)
    } else {
        None
    }
}

/// Deletes the spawn's creatures and its champion.
fn clear_spawn(w: &mut World, s: ItemId) {
    spawn::delete_creatures(w, s);
    if let Some(c) = spawn::data(w, s).and_then(|d| d.champion)
        && let Some(t) = w.mobile(c).map(|m| m.ty)
    {
        (t.delete)(w, c);
    }
}

fn kill_by_target(w: &mut World, e: &mut muo_server::commands::CommandEventArgs) {
    let from = e.mobile;
    muo_server::target::begin_target(w, from, 15, false, TargetFlags::NONE, |w, from, t| {
        if muo_content::mx::access_level(w, from) < AccessLevel::Administrator {
            return;
        }
        let s = match t {
            Targeted::Entity(e) => spawn_of_target(w, e),
            _ => None,
        };
        if s.is_none() {
            muo_content::mx::send_message(
                w,
                from,
                "That is not a valid target. Please target the champion, altar, platform, or idol.",
            );
        }
        if let Some(s) = s {
            clear_spawn(w, s);
        }
    });
    muo_content::mx::send_message(w, from, "Which champion spawn would you like to clear?");
}

fn kill_by_region(w: &mut World, e: &mut muo_server::commands::CommandEventArgs) {
    let from = e.mobile;
    let region = w.mobile(from).map(|m| m.region);
    let s = region.and_then(|r| {
        let reg = w.region(r)?;
        if !reg.ty.is(&spawn::CHAMPION_SPAWN_REGION) {
            return None;
        }
        reg.ext.get::<spawn::RegionSpawn>().map(|s| s.spawn)
    });
    match s {
        Some(s) => clear_spawn(w, s),
        None => muo_content::mx::send_message(w, from, "You are not in a champion spawn region."),
    }
}

/// `ChampionEntry`.
struct Entry {
    class: &'static str,
    randomize: bool,
    ty: info::ChampionSpawnType,
    sign: Point3D,
    eject: Point3D,
}

const fn p(x: i32, y: i32, z: i32) -> Point3D {
    Point3D::new(x, y, z)
}

const LL: &str = "Server.Engines.CannedEvil.LLChampionSpawn";
const DUNGEON: &str = "Server.Engines.CannedEvil.DungeonChampionSpawn";

/// `LLLocations` (all Felucca; an entry without a type randomizes it, starting from Abyss).
fn ll_locations() -> Vec<Entry> {
    use info::ChampionSpawnType::*;
    let e = |sign, eject| Entry {
        class: LL,
        randomize: true,
        ty: Abyss,
        sign,
        eject,
    };
    vec![
        e(p(5511, 2360, 42), p(5439, 2323, 26)),
        e(p(6038, 2401, 47), p(5988, 2340, 24)),
        e(p(5549, 2640, 16), p(5645, 2696, -8)),
        e(p(5636, 2916, 37), p(5721, 2949, 28)),
        e(p(6035, 2943, 50), p(6098, 2997, 17)),
        e(p(5265, 3171, 105), p(5314, 3232, 2)),
        e(p(5282, 3368, 50), p(5215, 3318, 3)),
        e(p(5207, 3637, 20), p(5263, 3687, 0)),
        e(p(5954, 3475, 25), p(6013, 3529, 0)),
        e(p(5982, 3882, 20), p(5929, 3820, -1)),
        e(p(5724, 3991, 41), p(5774, 4041, 26)),
        Entry {
            class: LL,
            randomize: false,
            ty: ForestLord,
            sign: p(5559, 3757, 21),
            eject: p(5513, 3878, 3),
        },
    ]
}

fn dungeon_locations() -> Vec<Entry> {
    use info::ChampionSpawnType::*;
    let e = |ty, sign, eject| Entry {
        class: DUNGEON,
        randomize: false,
        ty,
        sign,
        eject,
    };
    vec![
        e(UnholyTerror, p(5179, 709, 20), p(4111, 432, 5)),
        e(VerminHorde, p(5557, 827, 65), p(5580, 632, 30)),
        e(ColdBlood, p(5259, 837, 64), p(1176, 2637, 0)),
        e(Abyss, p(5815, 1352, 5), p(2923, 3406, 8)),
        e(Arachnid, p(5190, 1607, 20), p(5482, 3161, -54)),
    ]
}

fn process(w: &mut World, entries: Vec<Entry>) {
    for e in entries {
        let Some(id) = muo_content::factory::make_item(w, e.class) else {
            tracing::error!(
                "Failed to generate champion \"{}\" at {:?} (Felucca).",
                e.class,
                e.sign
            );
            continue;
        };
        if let Some(d) = spawn::data_mut(w, id) {
            d.randomize_type = e.randomize;
            d.ty = e.ty as i32;
        }
        muo_server::item::ops::move_to_world_map(w, id, e.sign, Some(MapId::FELUCCA));
        if let Some(d) = spawn::data_mut(w, id) {
            d.eject_location = e.eject;
            d.eject_map = Some(MapId::FELUCCA);
        }
        if spawn::always_active(w, id)
            && let Some(d) = spawn::data_mut(w, id)
        {
            d.ready_to_activate = true;
        }
    }
}

/// `[GenChamps`: "Generates champions for Felucca Dungeons & Lost Lands."
fn gen_champs(w: &mut World, _e: &mut muo_server::commands::CommandEventArgs) {
    // We assume that all champion spawns are generated here.
    let spawns: Vec<ItemId> = w
        .items
        .iter()
        .filter(|(_, i)| i.ext.get::<spawn::SpawnData>().is_some())
        .map(|(&id, _)| id)
        .collect();
    for s in spawns {
        if let Some(t) = w.item(s).map(|i| i.ty) {
            (t.delete)(w, s);
        }
    }
    process(w, dungeon_locations());
    process(w, ll_locations());
}

/// The `Configure` phase: the commands and the hooks other systems call.
pub fn configure(w: &mut World) {
    use muo_server::commands::CommandSystem;
    CommandSystem::register(
        w,
        "ClearChampByTarget",
        AccessLevel::Administrator,
        &[],
        std::rc::Rc::new(kill_by_target),
    );
    CommandSystem::register(
        w,
        "ClearChampByRegion",
        AccessLevel::Administrator,
        &[],
        std::rc::Rc::new(kill_by_region),
    );
    CommandSystem::register(
        w,
        "GenChamps",
        AccessLevel::Developer,
        &[],
        std::rc::Rc::new(gen_champs),
    );

    // `ShepherdsCrook`'s `Region.Find(bc.Home, map) is ChampionSpawnRegion` branch.
    w.resources
        .get_mut::<muo_items::weapons::special::HerdingHooks>()
        .in_champion_spawn_region_of = Some(|w, bc| {
        let Some(d) = muo_content::creature::cd(w, bc) else {
            return false;
        };
        let r = muo_server::region::find(w, d.home, d.home_map);
        let Some(reg) = w.region(r) else { return false };
        if !reg.ty.is(&spawn::CHAMPION_SPAWN_REGION) {
            return false;
        }
        reg.ext
            .get::<spawn::RegionSpawn>()
            .and_then(|s| spawn::data(w, s.spawn))
            .is_some_and(|s| s.creatures.contains(&bc))
    });

    // The spells' travel validators.
    let h = muo_content::spells::hooks::hooks_mut(w);
    h.champion_spawn_of = Some(|w, m| {
        let mob = w.mobile(m)?;
        let r = muo_server::region::find(w, mob.location, mob.map);
        let s = spawn::spawn_of_region(w, r)?;
        let i = w.item(s)?;
        Some((i.location, i.map?))
    });
    h.is_champion_spawn = Some(|w, map, loc| {
        let r = muo_server::region::find(w, loc, Some(map));
        spawn::spawn_of_region(w, r).is_some()
    });

    // `BaseCreature.SpawnMinion`'s `ChampionSpawn.FindSpawnOf(this)?.AddMinion(minion)`.
    w.resources
        .get_mut::<muo_content::creature::control::MinionHooks>()
        .champion_add_minion = Some(|w, creature, minion| {
        if let Some(s) = spawn::find_spawn_of(w, creature) {
            spawn::add_minion(w, s, minion);
        }
    });
}

pub fn register(types: &mut TypeRegistry) {
    gates::register(types);
    spawn::register(types);
    items::register(types);
    types.register_phase(
        Phase::Configure,
        "ChampionGenerator.Configure",
        DEFAULT_CALL_PRIORITY,
        configure,
    );
    types.register_phase(
        Phase::Initialize,
        "CannedEvilTimer.Initialize",
        DEFAULT_CALL_PRIORITY,
        timer::initialize,
    );
    types.register_phase(
        Phase::Initialize,
        "ChampionTitleSystem.Initialize",
        DEFAULT_CALL_PRIORITY,
        titles::initialize,
    );
}
