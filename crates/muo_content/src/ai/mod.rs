//! `BaseAI` and its subclasses. An AI class is a `static AiClass` (virtuals as fn pointers,
//! inherited with `..BASE_AI`); the per-creature instance is the `AiState` in `CreatureData::ai`.
//!
//! The AI timer is one wheel timer per creature that thinks every `CurrentSpeed` and wakes earlier for the move
//! clock while a pursuit is live; `Prod` thinks now.

pub mod base;
pub mod classes;
pub mod context_menu;
pub mod familiar;
pub mod mage;
pub mod movement;
pub mod opposition_group;
pub mod orders;
pub mod speech;
pub mod timer;
pub mod transfer;

#[cfg(test)]
mod behavior_tests;

use muo_primitives::Point3D;
use muo_server::Ext;
use muo_server::ids::MobileId;
use muo_server::speech_event::SpeechEventArgs;
use muo_server::timer::TimerHandle;
use muo_server::world::World;

use crate::creature::data::{cd, cd_mut};
use crate::creature::enums::*;
use crate::creature::vtable::ctype;

pub use base::BASE_AI;

/// `ActionType`.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum ActionType {
    #[default]
    Wander,
    Combat,
    Guard,
    Flee,
    Backoff,
    Interact,
}

/// `ApproachOutcome`.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum ApproachOutcome {
    #[default]
    None,
    Arrived,
    Waiting,
    DirectProgress,
    Routing,
    Blocked,
    GaveUp,
    InvalidGoal,
}

/// `MoveResult` of `DoMoveImpl`.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum MoveResult {
    Success,
    SuccessAutoTurn,
    Blocked,
    BadState,
}

/// A movement goal: a mobile (chased) or a fixed point.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Goal {
    Mobile(MobileId),
    Point(Point3D),
}

/// `BaseAI`'s virtuals.
#[allow(clippy::type_complexity)]
pub struct AiClass {
    pub name: &'static str,
    /// FleeHealthThreshold / FleeChance / BackoffChance
    pub flee_health_threshold: f64,
    pub flee_chance: f64,
    pub backoff_chance: f64,
    pub think: fn(&mut World, MobileId) -> bool,
    pub obey: fn(&mut World, MobileId) -> bool,
    pub do_action_wander: fn(&mut World, MobileId) -> bool,
    pub do_action_combat: fn(&mut World, MobileId) -> bool,
    pub do_action_guard: fn(&mut World, MobileId) -> bool,
    pub do_action_flee: fn(&mut World, MobileId) -> bool,
    pub do_action_interact: fn(&mut World, MobileId) -> bool,
    pub do_action_backoff: fn(&mut World, MobileId) -> bool,
    pub on_aggressive_action: fn(&mut World, MobileId, MobileId) -> bool,
    pub handles_on_speech: fn(&World, MobileId, MobileId) -> bool,
    pub on_speech: fn(&mut World, MobileId, &mut SpeechEventArgs),
    pub can_detect_hidden: fn(&mut World, MobileId) -> bool,
    /// `DetectHidden()` (virtual).
    pub detect_hidden: fn(&mut World, MobileId),
    pub on_at_way_point: fn(&mut World, MobileId) -> bool,
    pub on_teleported: fn(&mut World, MobileId),
    /// `IssueOrder(order, previous, issuer, resuming, interruptedTarget)`; returns the order to rest in.
    pub issue_order: fn(
        &mut World,
        MobileId,
        OrderType,
        OrderType,
        Option<MobileId>,
        bool,
        Option<MobileId>,
    ) -> OrderType,
}

/// One creature's `BaseAI` instance.
pub struct AiState {
    pub class: &'static AiClass,
    action: ActionType,
    /// `AITimer`.
    pub timer: timer::AiTimer,
    /// `NextMove`.
    pub next_move: i64,
    pub next_debug_message: i64,
    pub next_detect_hidden: i64,
    pub last_order: muo_primitives::DateTime,
    // Last-known-position tracking.
    lkp_target: Option<MobileId>,
    lkp_location: Point3D,
    lkp_goal_set: bool,
    lkp_expire_tick: i64,
    guard_stop_tick: i64,
    investigate_stop_tick: i64,
    investigating: bool,
    herd_goal: Option<Point3D>,
    // Approach state.
    approach_goal: Option<MobileId>,
    approach_goal_loc: Point3D,
    approach_best_dist: f64,
    approach_stall_ticks: i32,
    approach_gave_up: bool,
    approach_gave_up_goal_loc: Point3D,
    pub last_approach: ApproachOutcome,
    // Move intent.
    move_intent: Option<Goal>,
    move_intent_range: i32,
    move_intent_expire: i64,
    /// `Path`.
    pub path: Option<crate::pathing::PathFollower>,
    // Pet orders.
    pub persistent_order: OrderType,
    persistent_target: Option<MobileId>,
    /// State of a class that adds fields (MageAI...).
    pub ext: Ext,
}

impl AiState {
    pub fn new(class: &'static AiClass, tick_count: i64) -> AiState {
        AiState {
            class,
            action: ActionType::Wander,
            timer: timer::AiTimer::new(tick_count),
            next_move: tick_count,
            next_debug_message: 0,
            next_detect_hidden: tick_count,
            last_order: muo_primitives::DateTime::MIN,
            lkp_target: None,
            lkp_location: Point3D::ZERO,
            lkp_goal_set: false,
            lkp_expire_tick: 0,
            guard_stop_tick: 0,
            investigate_stop_tick: 0,
            investigating: false,
            herd_goal: None,
            approach_goal: None,
            approach_goal_loc: Point3D::ZERO,
            approach_best_dist: 0.0,
            approach_stall_ticks: 0,
            approach_gave_up: false,
            approach_gave_up_goal_loc: Point3D::ZERO,
            last_approach: ApproachOutcome::None,
            move_intent: None,
            move_intent_range: 0,
            move_intent_expire: 0,
            path: None,
            persistent_order: OrderType::None,
            persistent_target: None,
            ext: Ext::NONE,
        }
    }
}

pub fn ai(w: &World, id: MobileId) -> Option<&AiState> {
    cd(w, id)?.ai.as_ref()
}

pub fn ai_mut(w: &mut World, id: MobileId) -> Option<&mut AiState> {
    cd_mut(w, id)?.ai.as_mut()
}

pub fn class_of(w: &World, id: MobileId) -> Option<&'static AiClass> {
    ai(w, id).map(|a| a.class)
}

/// Calls from other modules, `Option` for a creature without an AI (`AIObject?.X`).
pub fn action(w: &World, id: MobileId) -> Option<ActionType> {
    ai(w, id).map(|a| a.action)
}

/// `AIObject.Action = value`: `OnActionChanged` clears the move intent.
pub fn set_action(w: &mut World, id: MobileId, value: ActionType) {
    if let Some(a) = ai_mut(w, id)
        && a.action != value
    {
        a.action = value;
        base::on_action_changed(w, id);
    }
}

/// `ChangeAIType`: replaces the AI; `ForcedAI` wins over the type.
pub fn change_ai_type(w: &mut World, id: MobileId, new_ai: AIType) {
    stop_timer(w, id);
    // Read once: each read of ForcedAI constructs an AI whose ctor may start its timer.
    let forced = ctype(w, id).and_then(|c| (c.forced_ai)(w, id));
    let class = forced.or(match new_ai {
        AIType::AiMelee => Some(&classes::MELEE_AI),
        AIType::AiAnimal => Some(&classes::ANIMAL_AI),
        AIType::AiBerserk => Some(&classes::BERSERK_AI),
        AIType::AiArcher => Some(&classes::ARCHER_AI),
        AIType::AiHealer => Some(&classes::HEALER_AI),
        AIType::AiVendor => Some(&classes::VENDOR_AI),
        AIType::AiMage => Some(&mage::MAGE_AI),
        //TODO Implement PredatorAI (it is a MeleeAI for now)
        AIType::AiPredator => Some(&classes::MELEE_AI),
        AIType::AiThief => Some(&classes::THIEF_AI),
        AIType::AiUseDefault => None,
    });
    let tick = w.tick_count;
    let Some(class) = class else {
        if let Some(d) = cd_mut(w, id) {
            d.ai = None;
        }
        return;
    };
    if let Some(d) = cd_mut(w, id) {
        d.ai = Some(AiState::new(class, tick));
    }
    base::construct(w, id);
}

/// `new BaseAI(m)` tail: starts the timer when the sector is active, sets Wander.
pub fn stop_timer(w: &mut World, id: MobileId) {
    timer::stop(w, id);
}

/// `AIObject.Activate()`.
pub fn activate(w: &mut World, id: MobileId) {
    if ai(w, id).is_some() {
        timer::activate(w, id);
    }
}

/// `AIObject.Deactivate()`.
pub fn deactivate(w: &mut World, id: MobileId) {
    if ai(w, id).is_some() {
        base::deactivate(w, id);
    }
}

/// `AIObject?.AITimer?.Prod()`.
pub fn prod(w: &mut World, id: MobileId) {
    timer::prod(w, id);
}

/// `OnCurrentSpeedChange`: `AIObject?.OnCurrentSpeedChanged()`.
pub fn on_current_speed_changed(w: &mut World, id: MobileId) {
    timer::on_speed_changed(w, id);
}

/// `BaseCreature.CheckAIActive`.
pub fn check_ai_active(w: &mut World, id: MobileId) {
    let sensitive = ctype(w, id).is_some_and(|c| (c.player_range_sensitive)(w, id));
    if sensitive && ai(w, id).is_some() && crate::creature::overrides::sector_active(w, id) {
        activate(w, id);
    }
}

/// `AIObject.NextMove = value`.
pub fn set_next_move(w: &mut World, id: MobileId, value: i64) {
    if let Some(a) = ai_mut(w, id) {
        a.next_move = value;
    }
}

/// `AIObject?.IsPacingToMaster() == true`.
pub fn is_pacing_to_master(w: &World, id: MobileId) -> bool {
    ai(w, id).is_some() && movement::is_pacing_to_master(w, id)
}

pub fn on_teleported(w: &mut World, id: MobileId) {
    if let Some(c) = class_of(w, id) {
        (c.on_teleported)(w, id);
    }
}

pub fn on_aggressive_action(w: &mut World, id: MobileId, aggressor: MobileId) -> bool {
    match class_of(w, id) {
        Some(c) => (c.on_aggressive_action)(w, id, aggressor),
        None => false,
    }
}

pub fn handles_on_speech(w: &World, id: MobileId, from: MobileId) -> bool {
    class_of(w, id).is_some_and(|c| (c.handles_on_speech)(w, id, from))
}

pub fn on_speech(w: &mut World, id: MobileId, e: &mut SpeechEventArgs) {
    if let Some(c) = class_of(w, id) {
        (c.on_speech)(w, id, e);
    }
}

pub use orders::{is_guard_ally, persistent_order};

/// `IssueOrder` of the creature's AI class.
pub fn issue_order(
    w: &mut World,
    id: MobileId,
    order: OrderType,
    previous: OrderType,
    issuer: Option<MobileId>,
    resuming: bool,
    interrupted: Option<MobileId>,
) -> OrderType {
    match class_of(w, id) {
        Some(c) => (c.issue_order)(w, id, order, previous, issuer, resuming, interrupted),
        None => order,
    }
}

type FollowFn = fn(&mut World, MobileId, Goal, i32) -> bool;
/// Hooks into the pathing engine (Engines/Pathing: `PathFollower`). Unset: no routing, greedy steps only.
#[derive(Default)]
pub struct PathHooks {
    /// `PathFollower.Follow(range)` for the creature's current goal: true when it has arrived.
    pub follow: Option<FollowFn>,
    /// `Path.ForceRepath()`.
    pub force_repath: Option<fn(&mut World, MobileId)>,
}

/// `HandlesOnSpeech` of a class (default: `Mobile.Player`-agnostic false).
pub fn _unused(_: TimerHandle) {}

/// What the AI tests of this and downstream crates need on top of `map::test_support::world()`: the fixture's
/// era (Expansion.EJ), the Medium NPC speeds `PetTestStub.GetSpeeds` returns, the wiring, movement and pathing
/// hooks, and the player's follower list `BaseCreature` asks for.
#[doc(hidden)]
pub fn configure_for_tests(w: &mut World) {
    w.core.expansion = Some(muo_data::expansion::Expansion::EJ); // TestServerInitializer: Core.Expansion = EJ
    w.resources
        .get_mut::<crate::creature::speeds::NpcSpeeds>()
        .register(crate::creature::speeds::SpeedClassEntry {
            level: "Medium".into(),
            active: 0.2,
            passive: 0.4,
            active_move: 0.0,
            passive_move: 0.0,
            types: vec![],
        });
    crate::wiring::configure(w);
    crate::creature::misc::configure(w); // BaseCreature.Configure: the mount items
    muo_server::movement::configure(w);
    crate::pathing::configure(w);
    // PlayerMobile.Configure: what BaseCreature asks of a player's follower list
    let h = w.resources.get_mut::<crate::hooks::PetPlayerHooks>();
    h.add_follower = Some(crate::player_mobile::add_follower);
    h.remove_follower = Some(|w, pm, c| {
        crate::player_mobile::remove_follower(w, pm, c);
    });
    h.all_followers = Some(|w, pm| {
        crate::player_mobile::data(w, pm)
            .and_then(|d| d.all_followers.as_ref())
            .map_or_else(Vec::new, |f| f.iter().copied().collect())
    });
}
