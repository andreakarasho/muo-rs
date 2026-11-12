//! `StaminaSystem`: stamina lost to steps and weight, and the idle gain.
//!
//! `IHasSteps` is implemented by `PlayerMobile` and by mounts (`BaseMount`, `EtherealMount`,
//! `VirtualMountItem`). Steps are kept per [`StepsKey`]; what a mount reports (`StepsMax`, the idle gain) comes
//! from [`StaminaHooks::step_traits`], which the mount classes set.
//!
//! `DeathStrike.AddStep(from)` (Spells/Ninjitsu) is the `death_strike_add_step` hook of `PlayerMobile`.

use indexmap::{IndexMap, IndexSet};
use muo_primitives::{DateTime, Direction, TimeSpan};
use muo_server::World;
use muo_server::events::MovementEventArgs;
use muo_server::ids::{EntityId, MobileId};
use muo_server::mobile::AccessLevel;
use muo_server::mobile::part1::props_b;
use muo_server::mobile::part2::messages;
use muo_server::timer;

/// `DFAlgorithm`.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum DfAlgorithm {
    #[default]
    Standard,
    PainSpike,
}

/// `IHasSteps`' three members.
#[derive(Copy, Clone, Debug)]
pub struct StepTraits {
    pub steps_max: i32,
    pub steps_gained_per_idle_time: i32,
    pub idle_time_per_steps_gain: TimeSpan,
}

/// `PlayerMobile.StepsMax` etc.
pub const PLAYER_STEPS: StepTraits = StepTraits {
    steps_max: 16,
    steps_gained_per_idle_time: 1,
    idle_time_per_steps_gain: TimeSpan::from_secs(1),
};

/// `EtherealMountStepCounter`: only the idle gain is used, not the other properties.
const ETHEREAL_COUNTER: StepTraits = StepTraits {
    steps_max: 3840,
    steps_gained_per_idle_time: 1,
    idle_time_per_steps_gain: TimeSpan::from_secs(1),
};

/// An `IHasSteps` instance.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum StepsKey {
    Entity(EntityId),
    /// `EtherealMountStepCounter` of a rider, one per player when `GlobalEtherealMountStamina`.
    EtherealCounter(MobileId),
}

#[derive(Default)]
pub struct StaminaHooks {
    /// `IHasSteps` of a mount or other entity. Players are answered here without it.
    pub step_traits: Option<fn(&World, EntityId) -> Option<StepTraits>>,
    /// `IsEthereal(m, out pm)`: the player riding an `EtherealMount` / `VirtualMountItem`.
    pub ethereal_rider: Option<fn(&World, EntityId) -> Option<MobileId>>,
    /// `Mobile.Mount` as an `IHasSteps`.
    pub mount_of: Option<fn(&World, MobileId) -> Option<EntityId>>,
}

#[derive(Copy, Clone, Debug)]
struct StepsTaken {
    steps: i32,
    idle_start_time: DateTime,
}

/// The settings (`Configure`) and the tables (`_stepsTaken`, `_etherealMountStepCounters`, `_resetHash`).
pub struct StaminaSystem {
    pub dfa: DfAlgorithm,
    pub stones_overweight_allowance: i32,
    pub cannot_run_when_fatigued: bool,
    pub cannot_walk_when_fatigued: bool,
    pub stones_per_overweight_loss: i32,
    pub base_overweight_loss: i32,
    pub additional_loss_when_below: f64,
    pub enable_mount_stamina: bool,
    pub use_mount_stamina_only_when_overloaded: bool,
    pub global_ethereal_mount_stamina: bool,
    ethereal_counters: IndexSet<MobileId>,
    steps_taken: IndexMap<StepsKey, StepsTaken>,
    reset_hash: IndexSet<StepsKey>,
}

impl Default for StaminaSystem {
    fn default() -> Self {
        StaminaSystem {
            dfa: DfAlgorithm::Standard,
            stones_overweight_allowance: 0,
            cannot_run_when_fatigued: false,
            cannot_walk_when_fatigued: false,
            stones_per_overweight_loss: 0,
            base_overweight_loss: 0,
            additional_loss_when_below: 0.0,
            enable_mount_stamina: false,
            use_mount_stamina_only_when_overloaded: false,
            global_ethereal_mount_stamina: false,
            ethereal_counters: IndexSet::new(),
            steps_taken: IndexMap::new(),
            reset_hash: IndexSet::new(),
        }
    }
}

const RESET_DURATION: TimeSpan = TimeSpan::from_hours(4);

fn sys(w: &World) -> Option<&StaminaSystem> {
    w.resources.get::<StaminaSystem>()
}

fn sys_mut(w: &mut World) -> &mut StaminaSystem {
    w.resources.get_mut::<StaminaSystem>()
}

/// `StaminaSystem.Configure`.
pub fn configure(w: &mut World) {
    let aos = crate::core::aos(w);
    let sa = crate::core::sa(w);
    let ml = crate::core::ml(w);
    let c = &mut w.config;
    let get_bool = |c: &mut muo_data::configuration::ServerConfiguration, k, d| {
        c.get_or_update_setting(k, d).unwrap_or(d)
    };
    let cannot_run = get_bool(c, "stamina.cannotRunWhenFatigued", !aos);
    let cannot_walk = get_bool(c, "stamina.cannotWalkWhenFatigued", false);
    let per_loss = c
        .get_or_update_setting("stamina.stonesPerOverweightLoss", 25)
        .unwrap_or(25);
    let allowance = c
        .get_or_update_setting("stamina.stonesOverweightAllowance", 4)
        .unwrap_or(4);
    let base_loss = c
        .get_or_update_setting("stamina.baseOverweightLoss", 5)
        .unwrap_or(5);
    let below = c
        .get_or_update_setting("stamina.additionalLossWhenBelow", 0.10)
        .unwrap_or(0.10);
    let mount = get_bool(c, "stamina.enableMountStamina", true);
    let only_overloaded = c.get_setting("stamina.useMountStaminaOnlyWhenOverloaded", sa);
    let global_ethereal = c.get_setting("stamina.globalEtherealMountStamina", ml);
    let s = sys_mut(w);
    s.cannot_run_when_fatigued = cannot_run;
    s.cannot_walk_when_fatigued = cannot_walk;
    s.stones_per_overweight_loss = per_loss;
    s.stones_overweight_allowance = allowance;
    s.base_overweight_loss = base_loss;
    s.additional_loss_when_below = below;
    s.enable_mount_stamina = mount;
    s.use_mount_stamina_only_when_overloaded = only_overloaded;
    s.global_ethereal_mount_stamina = global_ethereal;
}

fn traits(w: &World, key: StepsKey) -> Option<StepTraits> {
    match key {
        StepsKey::EtherealCounter(_) => Some(ETHEREAL_COUNTER),
        StepsKey::Entity(EntityId::Mobile(m)) if crate::player_mobile::data(w, m).is_some() => {
            Some(PLAYER_STEPS)
        }
        StepsKey::Entity(e) => w.resources.get::<StaminaHooks>()?.step_traits?(w, e),
    }
}

fn ethereal_rider(w: &World, e: StepsKey) -> Option<MobileId> {
    let StepsKey::Entity(e) = e else { return None };
    w.resources.get::<StaminaHooks>()?.ethereal_rider?(w, e)
}

fn mount_of(w: &World, m: MobileId) -> Option<EntityId> {
    match w.resources.get::<StaminaHooks>().and_then(|h| h.mount_of) {
        Some(f) => f(w, m),
        None => w.mobile(m)?.mount_item.map(EntityId::Item),
    }
}

/// `GetStepsTaken`'s key resolution: an ethereal mount's steps are the rider's global counter.
fn resolve_key(w: &World, key: StepsKey, create: bool, s_global: bool) -> Option<StepsKey> {
    if s_global && let Some(pm) = ethereal_rider(w, key) {
        let counter = StepsKey::EtherealCounter(pm);
        if create || sys(w).is_some_and(|s| s.ethereal_counters.contains(&pm)) {
            return Some(counter);
        }
        return None;
    }
    Some(key)
}

fn global(w: &World) -> bool {
    sys(w).is_some_and(|s| s.global_ethereal_mount_stamina)
}

/// `RemoveEntry(m)`.
pub fn remove_entry(w: &mut World, key: StepsKey) {
    if global(w)
        && let Some(pm) = ethereal_rider(w, key)
    {
        sys_mut(w).ethereal_counters.shift_remove(&pm);
    }
    sys_mut(w).steps_taken.shift_remove(&key);
}

/// `RegenSteps(ref stepsTaken, amount, removeOnInvalidation)`: whether the entry still exists.
fn regen_entry(w: &mut World, key: StepsKey, amount: i32, remove_on_invalidation: bool) -> bool {
    let (Some(t), Some(tr)) = (
        sys(w).and_then(|s| s.steps_taken.get(&key).copied()),
        traits(w, key),
    ) else {
        return false;
    };
    let idle = w.now - t.idle_start_time;
    let gained = (idle.ticks() / tr.idle_time_per_steps_gain.ticks().max(1)) as i32
        * tr.steps_gained_per_idle_time;
    let steps = t.steps - (gained + amount);
    if steps <= 0 {
        if remove_on_invalidation {
            remove_entry(w, key);
            return false;
        }
        if let Some(e) = sys_mut(w).steps_taken.get_mut(&key) {
            e.steps = 0;
        }
        return true;
    }
    if let Some(e) = sys_mut(w).steps_taken.get_mut(&key) {
        e.steps = steps;
    }
    true
}

/// `RegenSteps(m, amount, removeOnInvalidation)`: whether `m` still has steps taken.
pub fn regen_steps(w: &mut World, m: StepsKey, amount: i32, remove_on_invalidation: bool) -> bool {
    let g = global(w);
    match resolve_key(w, m, false, g) {
        Some(k) if sys(w).is_some_and(|s| s.steps_taken.contains_key(&k)) => {
            regen_entry(w, k, amount, remove_on_invalidation)
        }
        _ => false,
    }
}

/// `OnDismount(mount)`.
pub fn on_dismount(w: &mut World, mount: StepsKey) {
    if !sys(w).is_some_and(|s| s.enable_mount_stamina) {
        return;
    }
    if regen_steps(w, mount, 0, true) {
        sys_mut(w).reset_hash.insert(mount);
    } else {
        sys_mut(w).reset_hash.shift_remove(&mount);
    }
}

/// `FatigueOnDamage(m, damage)`.
pub fn fatigue_on_damage(w: &mut World, m: MobileId, damage: i32) {
    let Some(mm) = w.mobile(m) else { return };
    let (hits, stam) = (f64::from(mm.hits), f64::from(mm.stam));
    let d = f64::from(damage);
    let fatigue = match sys(w).map_or(DfAlgorithm::Standard, |s| s.dfa) {
        DfAlgorithm::Standard => d * (100.0 / hits) * (stam / 100.0) - 5.0,
        DfAlgorithm::PainSpike => d * (100.0 / hits + (50.0 + stam) / 100.0 - 1.0) - 5.0,
    };
    if fatigue > 0.0 {
        props_b::set_stam(w, m, mm.stam - fatigue as i32);
    }
}

/// `GetMaxWeight(m)`.
pub fn get_max_weight(w: &World, m: MobileId) -> i32 {
    w.mobile(m).map_or(0, |x| (x.ty.max_weight)(w, m))
}

/// `GetStamLoss(from, overWeight, running)`.
pub fn get_stam_loss(w: &mut World, from: MobileId, over_weight: i32, running: bool) -> i32 {
    let (base, per) = sys(w).map_or((0, 1), |s| {
        (s.base_overweight_loss, s.stones_per_overweight_loss)
    });
    let mut loss = base + over_weight / per; // divides by zero if the setting is 0
    if props_b::mounted(w, from) {
        loss /= 3;
    }
    if running {
        loss *= 2;
    }
    loss
}

/// `IsOverloaded(m)`.
pub fn is_overloaded(w: &World, m: MobileId) -> bool {
    let Some(mm) = w.mobile(m) else { return false };
    if !mm.player || !(mm.ty.alive)(w, m) || mm.access_level > AccessLevel::Player {
        return false;
    }
    let allowance = sys(w).map_or(0, |s| s.stones_overweight_allowance);
    props_b::body_weight(w) + mm.total_weight > get_max_weight(w, m) + allowance
}

fn stam_of(w: &World, m: MobileId) -> i32 {
    w.mobile(m).map_or(0, |x| x.stam)
}

/// `EventSink_Movement`.
pub fn on_movement(w: &mut World, e: &mut MovementEventArgs) {
    let from = e.mobile;
    let Some(m) = w.mobile(from) else { return };
    if !m.player || !(m.ty.alive)(w, from) || m.access_level > AccessLevel::Player {
        return;
    }
    let (allowance, enable_mount) = sys(w).map_or((0, false), |s| {
        (s.stones_overweight_allowance, s.enable_mount_stamina)
    });
    let max_weight = get_max_weight(w, from) + allowance;
    let overweight = props_b::body_weight(w) + m.total_weight - max_weight;
    match mount_of(w, from).filter(|_| enable_mount) {
        Some(mount) => process_mount_movement(w, StepsKey::Entity(mount), overweight, e),
        None => process_player_movement(w, overweight, e),
    }
    crate::player_mobile::hooks::death_strike_add_step(w, from);
}

fn process_player_movement(w: &mut World, overweight: i32, e: &mut MovementEventArgs) {
    let from = e.mobile;
    let running = e.direction.contains(Direction::RUNNING);
    let (cannot_walk, cannot_run, below) = sys(w).map_or((false, false, 0.0), |s| {
        (
            s.cannot_walk_when_fatigued,
            s.cannot_run_when_fatigued,
            s.additional_loss_when_below,
        )
    });
    if cannot_walk && stam_of(w, from) <= 0 {
        messages::send_localized_message(w, from, 500110, "", 0x3B2); // You are too fatigued to move.
        e.blocked = true;
        return;
    }
    if overweight > 0 {
        let loss = get_stam_loss(w, from, overweight, running);
        props_b::set_stam(w, from, stam_of(w, from) - loss);
        if stam_of(w, from) <= 0 {
            // You are too fatigued to move, because you are carrying too much weight!
            messages::send_localized_message(w, from, 500109, "", 0x3B2);
            e.blocked = true;
            return;
        }
    }
    if !running {
        return;
    }
    let stam_max = w.mobile(from).map_or(1, |m| (m.ty.stam_max)(w, from));
    if below > 0.0 && f64::from(stam_of(w, from)) / f64::from(stam_max.max(1)) < below {
        props_b::set_stam(w, from, stam_of(w, from) - 1);
    }
    if cannot_run && stam_of(w, from) <= 0 {
        messages::send_localized_message(w, from, 500110, "", 0x3B2);
        e.blocked = true;
        return;
    }
    if crate::player_mobile::data(w, from).is_some() {
        let key = StepsKey::Entity(EntityId::Mobile(from));
        let now = w.now;
        let created = !sys(w).is_some_and(|s| s.steps_taken.contains_key(&key));
        if created {
            sys_mut(w).steps_taken.insert(
                key,
                StepsTaken {
                    steps: 0,
                    idle_start_time: DateTime::MIN,
                },
            );
        } else {
            regen_entry(w, key, 0, false);
        }
        let mounted = props_b::mounted(w, from);
        let max_steps = PLAYER_STEPS.steps_max * if mounted { 3 } else { 1 };
        let steps = sys(w)
            .and_then(|s| s.steps_taken.get(&key))
            .map_or(0, |t| t.steps)
            + 1;
        let mut stored = steps;
        if steps > max_steps {
            props_b::set_stam(w, from, stam_of(w, from) - 1);
            stored = 0;
        }
        sys_mut(w).steps_taken.insert(
            key,
            StepsTaken {
                steps: stored,
                idle_start_time: now,
            },
        );
    }
}

fn process_mount_movement(
    w: &mut World,
    mount: StepsKey,
    overweight: i32,
    e: &mut MovementEventArgs,
) {
    let from = e.mobile;
    let Some(tr) = traits(w, mount) else { return };
    let running = e.direction.contains(Direction::RUNNING);
    let only_overloaded = sys(w).is_some_and(|s| s.use_mount_stamina_only_when_overloaded);
    let stam_loss = if overweight > 0 {
        get_stam_loss(w, from, overweight, running)
    } else {
        0
    };
    if stam_loss <= 0 && (!running || only_overloaded) {
        return;
    }
    let g = global(w);
    let Some(key) = resolve_key(w, mount, true, g) else {
        return;
    };
    if let StepsKey::EtherealCounter(pm) = key {
        sys_mut(w).ethereal_counters.insert(pm);
    }
    let now = w.now;
    if sys(w).is_some_and(|s| s.steps_taken.contains_key(&key)) {
        regen_entry(w, key, 0, false);
    } else {
        sys_mut(w).steps_taken.insert(
            key,
            StepsTaken {
                steps: 0,
                idle_start_time: DateTime::MIN,
            },
        );
    }
    let t = sys_mut(w).steps_taken.get_mut(&key);
    let Some(t) = t else { return };
    t.steps += stam_loss + 1;
    t.idle_start_time = now;
    if t.steps > tr.steps_max {
        t.steps = tr.steps_max;
        messages::send_localized_message(w, from, 500108, "", 0x3B2); // Your mount is too fatigued to move.
        e.blocked = true;
    }
}

/// `StaminaSystem.OnPlayerDeleted` (`PlayerDeletedEvent`).
pub fn on_player_deleted(w: &mut World, m: MobileId) {
    remove_entry(w, StepsKey::Entity(EntityId::Mobile(m)));
}

/// `StaminaSystem.OnLogin` (`PlayerLoginEvent`).
pub fn on_login(w: &mut World, pm: MobileId) {
    let now = w.now;
    if sys(w).is_some_and(|s| s.enable_mount_stamina)
        && let Some(mount) = mount_of(w, pm)
    {
        // Start idle for mount
        let key = StepsKey::Entity(mount);
        let g = global(w);
        if let Some(k) = resolve_key(w, key, false, g)
            && let Some(t) = sys(w).and_then(|s| s.steps_taken.get(&k).copied())
        {
            if t.steps <= 0 || now >= t.idle_start_time + TimeSpan::from_hours(4) {
                sys_mut(w).steps_taken.shift_remove(&k);
            } else if let Some(t) = sys_mut(w).steps_taken.get_mut(&k) {
                t.idle_start_time = now;
            }
        }
        sys_mut(w).reset_hash.shift_remove(&key);
    }
    let pk = StepsKey::Entity(EntityId::Mobile(pm));
    if regen_steps(w, pk, 0, true)
        && let Some(t) = sys_mut(w).steps_taken.get_mut(&pk)
    {
        t.idle_start_time = now;
    }
}

/// `StaminaSystem.Logout`.
pub fn on_logout(w: &mut World, m: MobileId) {
    if sys(w).is_none_or(|s| s.steps_taken.is_empty()) {
        return;
    }
    let now = w.now;
    if sys(w).is_some_and(|s| s.enable_mount_stamina)
        && let Some(mount) = mount_of(w, m)
    {
        let key = StepsKey::Entity(mount);
        if regen_steps(w, key, 0, true) {
            let g = global(w);
            if let Some(k) = resolve_key(w, key, false, g)
                && let Some(t) = sys_mut(w).steps_taken.get_mut(&k)
            {
                t.idle_start_time = now;
            }
            sys_mut(w).reset_hash.insert(key);
        }
    }
    let pk = StepsKey::Entity(EntityId::Mobile(m));
    if crate::player_mobile::data(w, m).is_some()
        && regen_steps(w, pk, 0, true)
        && let Some(t) = sys_mut(w).steps_taken.get_mut(&pk)
    {
        t.idle_start_time = now;
    }
}

/// `StaminaSystem.Initialize`: movement and logout events, the idle-credit pass over the loaded entries and the
/// five-minute `ResetTimer`.
pub fn initialize(w: &mut World) {
    w.events.movement.subscribe(on_movement);
    w.events.logout.subscribe(|w, m| on_logout(w, *m));
    crate::player_mobile::subscribe_deleted(w, on_player_deleted);
    crate::network::events::subscribe_ranked::<crate::network::events::PlayerLogin>(
        w,
        14,
        |w, e| on_login(w, e.0),
    );

    // Credit idle time
    let keys: Vec<StepsKey> =
        sys(w).map_or_else(Vec::new, |s| s.steps_taken.keys().copied().collect());
    for k in keys {
        if regen_steps(w, k, 0, false)
            && sys(w)
                .and_then(|s| s.steps_taken.get(&k))
                .is_some_and(|t| t.steps <= 0)
        {
            sys_mut(w).steps_taken.shift_remove(&k);
        }
    }

    // ResetTimer
    let mut next_check = w.now;
    timer::start(
        w,
        TimeSpan::from_mins(5),
        TimeSpan::from_mins(5),
        0,
        move |w| {
            if w.now < next_check {
                return;
            }
            let keys: Vec<StepsKey> =
                sys(w).map_or_else(Vec::new, |s| s.reset_hash.iter().copied().collect());
            for k in keys {
                let g = global(w);
                let t = resolve_key(w, k, false, g)
                    .and_then(|r| sys(w).and_then(|s| s.steps_taken.get(&r).copied()));
                if t.is_none_or(|t| w.now >= t.idle_start_time + RESET_DURATION) {
                    sys_mut(w).reset_hash.shift_remove(&k);
                }
            }
            next_check = w.now + TimeSpan::from_hours(1);
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> (World, MobileId) {
        let mut w = World::new();
        let s = sys_mut(&mut w);
        s.base_overweight_loss = 5;
        s.stones_per_overweight_loss = 25;
        s.stones_overweight_allowance = 4;
        let id = crate::player_mobile::new_player_mobile(&mut w);
        w.mobile_mut(id).unwrap().player = true;
        (w, id)
    }

    #[test]
    fn stam_loss_formula() {
        let (mut w, id) = setup();
        assert_eq!(get_stam_loss(&mut w, id, 50, false), 7);
        assert_eq!(get_stam_loss(&mut w, id, 50, true), 14);
    }

    #[test]
    fn running_counts_steps_and_costs_stamina_past_the_max() {
        let (mut w, id) = setup();
        {
            let m = w.mobile_mut(id).unwrap();
            m.stam = 50;
            m.dex = 50;
        }
        let mut blocked = 0;
        for _ in 0..17 {
            let mut e = MovementEventArgs::new(id, Direction::NORTH | Direction::RUNNING);
            on_movement(&mut w, &mut e);
            blocked += i32::from(e.blocked);
        }
        assert_eq!(blocked, 0);
        // the 17th step exceeds StepsMax (16): one stamina point, steps reset
        assert_eq!(w.mobile(id).unwrap().stam, 49);
    }

    #[test]
    fn fatigue_scales_with_damage() {
        let (mut w, id) = setup();
        {
            let m = w.mobile_mut(id).unwrap();
            m.hits = 100;
            m.dex = 100;
            m.stam = 100;
        }
        fatigue_on_damage(&mut w, id, 25); // 25 * 1.0 * 1.0 - 5 = 20
        assert_eq!(w.mobile(id).unwrap().stam, 80);
    }
}
