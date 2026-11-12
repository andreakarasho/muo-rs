//! Halloween engines: `HolidaySettings` (2006 trick or treat, 2009 pumpkin patch, 2012 player zombies). They only
//! run between `HolidaySettings.StartHalloween` and `FinishHalloween` (2021).

use indexmap::{IndexMap, IndexSet};
use muo_content::hooks::rnd;
use muo_content::mx;
use muo_content::props::Value;
use muo_primitives::{DateTime, Point3D, Rectangle2D, TimeSpan};
use muo_server::World;
use muo_server::events::SpeechEventArgs;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::layer::Layer;
use muo_server::map::MapId;
use muo_server::skills::SkillName;
use muo_server::target::{TargetFlags, Targeted};
use muo_server::timer::{self, TimerHandle};
use muo_server::type_registry::{Phase, TypeRegistry};

use super::holiday::{check_mobile, naughty_twin_new, zombie_skeleton_new};

/// `HolidaySettings.StartHalloween`.
pub fn start_halloween() -> DateTime {
    DateTime::from_ymd(2021, 10, 24).expect("a date")
}

/// `HolidaySettings.FinishHalloween`.
pub fn finish_halloween() -> DateTime {
    DateTime::from_ymd(2021, 11, 15).expect("a date")
}

const GM_BEGGAR_TREATS: [&str; 11] = [
    "Server.Items.CreepyCake",
    "Server.Items.PumpkinPizza",
    "Server.Items.GrimWarning",
    "Server.Items.HarvestWine",
    "Server.Items.MurkyMilk",
    "Server.Items.MrPlainsCookies",
    "Server.Items.SkullsOnPike",
    "Server.Items.ChairInAGhostCostume",
    "Server.Items.ExcellentIronMaiden",
    "Server.Items.HalloweenGuillotine",
    "Server.Items.ColoredSmallWebs",
];

const TREATS: [&str; 5] = [
    "Server.Items.Lollipops",
    "Server.Items.WrappedCandy",
    "Server.Items.JellyBeans",
    "Server.Items.Taffy",
    "Server.Items.NougatSwirl",
];

/// `HolidaySettings.RandomGMBeggerItem` / `RandomTreat`: `types.RandomElement().CreateInstance<Item>()`.
fn random_of(w: &mut World, list: &[&str]) -> Option<ItemId> {
    let i = rnd::below(w, list.len() as i32) as usize;
    muo_content::factory::make_item(w, list[i])
}

fn in_window(w: &World) -> bool {
    w.now >= start_halloween() && w.now <= finish_halloween()
}

// ---- TrickOrTreat ----

/// `TrickOrTreat.Initialize()`.
fn trick_or_treat_initialize(w: &mut World) {
    if in_window(w) {
        w.events.speech.subscribe(on_speech);
    }
}

fn on_speech(w: &mut World, e: &mut SpeechEventArgs) {
    if e.speech.to_lowercase().contains("trick or treat") {
        muo_server::target::begin_target(w, e.mobile, 15, false, TargetFlags::NONE, on_target);
        mx::msg(w, e.mobile, 1076764); // Pick someone to Trick or Treat.
    }
}

/// `TrickOrTreat.RandomPointOneAway(x, y, z, map)`.
fn random_point_one_away(w: &mut World, x: i32, y: i32, z: i32, map: MapId) -> Point3D {
    let (dx, dy) = (rnd::from(w, -1, 3), rnd::from(w, -1, 3));
    let (nx, ny) = (x + dx, y + dy);
    let z = if muo_server::map::fit::can_fit(w, map, nx, ny, 0, 0, false, true, true) {
        w.maps.get_average_z(map, nx, ny)
    } else {
        z
    };
    Point3D::new(nx, ny, z)
}

/// `TrickOrTreat.Bleeding(from)`.
fn bleeding(w: &mut World, from: MobileId) {
    if !check_mobile(w, from) {
        return;
    }
    let loc = mx::location(w, from);
    if loc == Point3D::ZERO {
        return;
    }
    let Some(map) = mx::map(w, from) else { return };
    let amount = rnd::min_max(w, 3, 7);
    for _ in 0..amount {
        let id = rnd::min_max(w, 0x122C, 0x122F);
        let blood = muo_content::factory::make_item_args(
            w,
            "Server.Items.Blood",
            &[Value::Int(i64::from(id))],
        );
        let p = random_point_one_away(w, loc.x, loc.y, loc.z, map);
        if let Some(b) = blood {
            muo_server::item::ops::move_to_world_map(w, b, p, Some(map));
        }
    }
}

/// `TrickOrTreat.SolidHueMobile(target)`.
fn solid_hue_mobile(w: &mut World, target: MobileId) {
    if check_mobile(w, target) {
        let h = rnd::min_max(w, 2501, 2644);
        muo_server::mobile::part1::props_b::set_solid_hue_override(w, target, h);
        timer::start(w, TimeSpan::from_secs(10), TimeSpan::ZERO, 1, move |w| {
            remove_hue_mod(w, target)
        });
    }
}

/// `TrickOrTreat.RemoveHueMod(target)`.
fn remove_hue_mod(w: &mut World, target: MobileId) {
    if !mx::deleted(w, target) {
        muo_server::mobile::part1::props_b::set_solid_hue_override(w, target, -1);
    }
}

/// `TrickOrTreat.MakeTwin(from)`.
fn make_twin(w: &mut World, from: MobileId) {
    if !check_mobile(w, from) {
        return;
    }
    let twin = naughty_twin_new(w, Some(from));
    if mx::deleted(w, twin) {
        return;
    }
    let skip = |l: Layer| matches!(l, Layer::Backpack | Layer::Mount | Layer::Bank);
    let items: Vec<ItemId> = w
        .mobile(from)
        .map(|m| m.items.clone())
        .unwrap_or_default()
        .into_iter()
        .filter(|&i| !skip(muo_server::item::base::layer(w, i)))
        .collect();
    for item in items {
        if let Some(dupe) = muo_server::mobile::part2::items::lift_item_dupe(w, item, 1) {
            muo_server::mobile::part2::items::add_item(w, twin, dupe);
        }
    }
    let worn: Vec<ItemId> = w.mobile(twin).map(|m| m.items.clone()).unwrap_or_default();
    for item in worn {
        if !skip(muo_server::item::base::layer(w, item)) {
            muo_server::item::ops::set_movable(w, item, false);
        }
    }
    let (hue, body, kills) = w.mobile(from).map_or((0, 0, 0), |m| {
        (
            (m.ty.hue)(w, from),
            muo_server::mobile::part1::props_b::body(w, from).0,
            m.kills,
        )
    });
    muo_server::mobile::base::set_hue(w, twin, hue);
    crate::common::body(w, twin, body);
    muo_server::mobile::part1::props_b::set_kills(w, twin, kills);

    let loc = mx::location(w, from);
    let Some(map) = mx::map(w, from) else { return };
    let point = random_point_one_away(w, loc.x, loc.y, loc.z, map);
    let to = if muo_server::map::fit::can_spawn_mobile_at(w, map, point.x, point.y, point.z) {
        point
    } else {
        loc
    };
    muo_server::mobile::base::move_to_world(w, twin, to, map);

    timer::start(w, TimeSpan::from_secs(5), TimeSpan::ZERO, 1, move |w| {
        if check_mobile(w, twin) {
            muo_server::mobile::base::delete(w, twin);
        }
    });
}

/// `TrickOrTreatTarget.OnTarget`.
fn on_target(w: &mut World, from: MobileId, targ: Targeted) {
    use muo_server::mobile::part2::messages::say_number;
    if !check_mobile(w, from) {
        return;
    }
    let Targeted::Entity(EntityId::Mobile(begged)) = targ else {
        mx::msg(w, from, 1076781); // There is little chance of getting candy from that!
        return;
    };
    if !muo_content::vendor::is_vendor(w, begged) || mx::deleted(w, begged) {
        mx::msg(w, from, 1076765); // That doesn't look friendly.
        return;
    }
    let now = w.now;
    if !check_mobile(w, begged) {
        return;
    }
    let next =
        muo_content::vendor::data(w, begged).map_or(DateTime::MIN, |d| d.next_trick_or_treat);
    if next > now {
        mx::msg(w, from, 1076767); // That doesn't appear to have any more candy.
        return;
    }
    let minutes = rnd::min_max(w, 5, 10);
    if let Some(d) = muo_content::vendor::data_mut(w, begged) {
        d.next_trick_or_treat = now + TimeSpan::from_mins(i64::from(minutes));
    }
    let pack = muo_server::mobile::part1::props_b::backpack(w, from);
    if pack.is_none_or(|p| muo_server::item::ops::is_deleted(w, p)) {
        return;
    }
    if rnd::double(w) < 0.90 {
        let n = match rnd::below(w, 3) {
            0 => 1076768, // Oooooh, aren't you cute!
            1 => 1076779, // All right...This better not spoil your dinner!
            _ => 1076778, // Here you go! Enjoy!
        };
        say_number(w, begged, n, "");
        if rnd::double(w) < 0.01 && muo_server::skills::value(w, from, SkillName::Begging) >= 100.0
        {
            if let Some(i) = random_of(w, &GM_BEGGAR_TREATS) {
                muo_server::mobile::part2::items::add_to_backpack(w, from, i);
            }
            mx::msg(w, from, 1076777); // You receive a special treat!
        } else {
            if let Some(i) = random_of(w, &TREATS) {
                muo_server::mobile::part2::items::add_to_backpack(w, from, i);
            }
            mx::msg(w, from, 1076769); // You receive some candy.
        }
    } else {
        say_number(w, begged, 1076770, ""); // TRICK!
        match rnd::below(w, 4) {
            0 => {
                timer::start(
                    w,
                    TimeSpan::from_secs(1),
                    TimeSpan::from_secs(1),
                    10,
                    move |w| bleeding(w, from),
                );
            }
            1 => {
                timer::start(w, TimeSpan::from_secs(2), TimeSpan::ZERO, 1, move |w| {
                    solid_hue_mobile(w, from)
                });
            }
            _ => {
                timer::start(w, TimeSpan::from_secs(2), TimeSpan::ZERO, 1, move |w| {
                    make_twin(w, from)
                });
            }
        }
    }
}

// ---- PumpkinPatchSpawner ----

const PUMPKIN_FIELDS: [(i32, i32, i32, i32); 8] = [
    (4557, 1471, 20, 10),
    (796, 2152, 36, 24),
    (816, 2251, 16, 8),
    (816, 2261, 16, 8),
    (816, 2271, 16, 8),
    (816, 2281, 16, 8),
    (835, 2344, 16, 16),
    (816, 2344, 16, 24),
];

/// `PumpkinPatchSpawner._timer`.
#[derive(Default)]
struct Patch(Option<TimerHandle>);

/// `Utility.RandomPointIn(rect, map)`.
fn random_point_in(w: &mut World, rect: Rectangle2D, map: MapId) -> Point3D {
    let (x, y) = muo_primitives::utility::random_point_xy_in(&mut w.rng, rect);
    let z = w.maps.get_average_z(map, x, y);
    Point3D::new(x, y, z)
}

/// `PumpkinPatchSpawner.AddPumpkin(map)`.
fn add_pumpkin(w: &mut World, map: MapId) {
    for (x, y, width, height) in PUMPKIN_FIELDS {
        let rect = Rectangle2D::new(x, y, width, height);
        let spawn_count = height * width / 20;
        let existing = muo_server::map::enumerate::items_in_bounds(w, Some(map), rect, false)
            .filter(|&i| {
                w.item(i).is_some_and(|it| {
                    muo_content::items::is_named(it.ty, "Server.Items.HalloweenPumpkin")
                })
            })
            .count() as i32;
        // `if (pumpkins++ >= spawncount) break;` counts one past the limit.
        let pumpkins = existing.min(spawn_count + 1);
        if spawn_count > pumpkins
            && let Some(p) = muo_content::factory::make_item(w, "Server.Items.HalloweenPumpkin")
        {
            let at = random_point_in(w, rect, map);
            muo_server::item::ops::move_to_world_map(w, p, at, Some(map));
        }
    }
}

/// `PumpkinPatchSpawner.Initialize()`.
fn pumpkin_patch_initialize(w: &mut World) {
    if in_window(w) {
        // `Timer.DelayCall(TimeSpan.FromSeconds(30), 0, callback)`: every 30 seconds, until stopped.
        let h = timer::start(w, TimeSpan::ZERO, TimeSpan::from_secs(30), 0, |w| {
            add_pumpkin(w, MapId::FELUCCA);
            add_pumpkin(w, MapId::TRAMMEL);
            if w.now > finish_halloween()
                && let Some(mut h) = w.resources.get_mut::<Patch>().0.take()
            {
                h.cancel(w);
            }
        });
        w.resources.get_mut::<Patch>().0 = Some(h);
    }
}

// ---- HalloweenHauntings ----

const TOTAL_ZOMBIE_LIMIT: usize = 200;
const DEATH_QUEUE_LIMIT: usize = 200;
const QUEUE_DELAY_SECONDS: i64 = 120;
const QUEUE_CLEAR_INTERVAL_SECONDS: i64 = 1800;

const CEMETARIES: [(i32, i32, i32, i32); 16] = [
    (1272, 3712, 30, 20), // Jhelom
    (1337, 1444, 48, 52), // Britain
    (2424, 1098, 20, 28), // Trinsic
    (2728, 840, 54, 54),  // Vesper
    (4528, 1314, 20, 28), // Moonglow
    (712, 1104, 30, 22),  // Yew
    (5824, 1464, 22, 6),  // Fire Dungeon
    (5224, 3655, 14, 5),  // T2A
    (1272, 3712, 20, 30), // Jhelom
    (1337, 1444, 52, 48), // Britain
    (2424, 1098, 28, 20), // Trinsic
    (2728, 840, 54, 54),  // Vesper
    (4528, 1314, 28, 20), // Moonglow
    (712, 1104, 22, 30),  // Yew
    (5824, 1464, 6, 22),  // Fire Dungeon
    (5224, 3655, 5, 14),  // T2A
];

/// The statics of `HalloweenHauntings`.
#[derive(Default)]
struct Hauntings {
    timer: Option<TimerHandle>,
    clear_timer: Option<TimerHandle>,
    death_queue: Option<IndexSet<MobileId>>,
    /// `_reAnimated`: the player to the zombie skeleton.
    re_animated: Option<IndexMap<MobileId, MobileId>>,
}

/// `ZombieSkeleton.OnDelete()`: `HalloweenHauntings._reAnimated?.Remove(_deadPlayer)`.
pub fn forget_zombie(w: &mut World, dead_player: MobileId) {
    if let Some(r) = w.resources.get_mut::<Hauntings>().re_animated.as_mut() {
        r.shift_remove(&dead_player);
    }
}

/// `HalloweenHauntings.OnPlayerDeathEvent(pm)`.
fn on_player_death(w: &mut World, pm: MobileId) {
    if !in_window(w) {
        return;
    }
    if w.resources.get_mut::<Hauntings>().timer.is_none() {
        let h = timer::start(
            w,
            TimeSpan::ZERO,
            TimeSpan::from_secs(QUEUE_DELAY_SECONDS),
            0,
            timer_callback,
        );
        w.resources.get_mut::<Hauntings>().timer = Some(h);
    }
    if w.resources.get_mut::<Hauntings>().clear_timer.is_none() {
        let h = timer::start(
            w,
            TimeSpan::ZERO,
            TimeSpan::from_secs(QUEUE_CLEAR_INTERVAL_SECONDS),
            0,
            clear_callback,
        );
        w.resources.get_mut::<Hauntings>().clear_timer = Some(h);
    }
    // `_timer.Running` is true: it was started here or is still going.
    let q = w
        .resources
        .get_mut::<Hauntings>()
        .death_queue
        .get_or_insert_with(IndexSet::new);
    if q.len() < DEATH_QUEUE_LIMIT {
        q.insert(pm);
    }
}

fn clear_callback(w: &mut World) {
    if w.now > finish_halloween() {
        if let Some(mut h) = w.resources.get_mut::<Hauntings>().clear_timer.take() {
            h.cancel(w);
        }
        let s = w.resources.get_mut::<Hauntings>();
        s.re_animated = None;
        s.death_queue = None;
        return;
    }
    let s = w.resources.get_mut::<Hauntings>();
    if let Some(r) = s.re_animated.as_mut() {
        r.clear();
    }
    if let Some(q) = s.death_queue.as_mut() {
        q.clear();
    }
}

fn timer_callback(w: &mut World) {
    if w.now > finish_halloween() {
        if let Some(mut h) = w.resources.get_mut::<Hauntings>().timer.take() {
            h.cancel(w);
        }
        return;
    }
    let s = w.resources.get_mut::<Hauntings>();
    let Some(queue) = s.death_queue.as_ref() else {
        return;
    };
    let player = queue
        .iter()
        .copied()
        .find(|e| !s.re_animated.as_ref().is_some_and(|r| r.contains_key(e)));
    let animated = s.re_animated.as_ref().map_or(0, IndexMap::len);
    let Some(player) = player else { return };
    if mx::deleted(w, player) || animated >= TOTAL_ZOMBIE_LIMIT {
        return;
    }

    let map = if rnd::bool(w) {
        MapId::TRAMMEL
    } else {
        MapId::FELUCCA
    };
    let (x, y, width, height) = CEMETARIES[rnd::below(w, CEMETARIES.len() as i32) as usize];
    let home = random_point_in(w, Rectangle2D::new(x, y, width, height), map);

    if muo_server::map::fit::can_spawn_mobile_at(w, map, home.x, home.y, home.z) {
        let zombie = zombie_skeleton_new(w, Some(player));
        w.resources
            .get_mut::<Hauntings>()
            .re_animated
            .get_or_insert_with(IndexMap::new)
            .insert(player, zombie);
        muo_content::creature::control::set_home(w, zombie, home);
        muo_content::creature::data::edit(w, zombie, |d| d.range_home = 10);
        muo_server::mobile::base::move_to_world(w, zombie, home, map);
        if let Some(q) = w.resources.get_mut::<Hauntings>().death_queue.as_mut() {
            q.shift_remove(&player);
        }
    }
}

fn configure(w: &mut World) {
    muo_content::player_mobile::subscribe_death(w, on_player_death);
}

pub fn register(types: &mut TypeRegistry) {
    types.register_phase_default(
        Phase::Configure,
        "Server.Engines.Events.HalloweenHauntings.Configure",
        configure,
    );
    types.register_phase_default(
        Phase::Initialize,
        "Server.Engines.Events.TrickOrTreat.Initialize",
        trick_or_treat_initialize,
    );
    types.register_phase_default(
        Phase::Initialize,
        "Server.Engines.Events.PumpkinPatchSpawner.Initialize",
        pumpkin_patch_initialize,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_event_window_is_the_2021_one() {
        let mut w = World::new();
        w.now = DateTime::from_ymd(2021, 10, 23).unwrap();
        assert!(!in_window(&w));
        w.now = DateTime::from_ymd(2021, 11, 1).unwrap();
        assert!(in_window(&w));
        w.now = DateTime::from_ymd(2021, 11, 16).unwrap();
        assert!(!in_window(&w));
    }

    #[test]
    fn a_zombie_skeleton_is_immune_to_regular_poison_not_lethal() {
        let mut w = World::new();
        let z = super::super::holiday::zombie_skeleton_new(&mut w, None);
        let id = (super::super::holiday::ZOMBIE_SKELETON_C.poison_immune)(&w, z);
        assert_eq!(id, Some(muo_server::poison::PoisonId(1)));
    }

    #[test]
    fn a_death_outside_the_window_queues_nothing() {
        let mut w = World::new();
        let p = muo_content::player_mobile::new_player_mobile(&mut w);
        w.now = DateTime::from_ymd(2026, 1, 1).unwrap();
        on_player_death(&mut w, p);
        assert!(w.resources.get_mut::<Hauntings>().death_queue.is_none());
        w.now = DateTime::from_ymd(2021, 11, 1).unwrap();
        on_player_death(&mut w, p);
        on_player_death(&mut w, p);
        assert_eq!(
            w.resources
                .get_mut::<Hauntings>()
                .death_queue
                .as_ref()
                .map(IndexSet::len),
            Some(1)
        );
    }
}
