//! The data classes of the ML quests: `MLQuest` (the fields), the objectives and rewards (the definitions),
//! `QuestArea`, `MLQuestContext` and the fields of `MLQuestInstance`.
//!
//! A quest class is a `static QuestClass` (its constant overrides and hooks); the quest object a `MLQuest()`
//! constructor builds is an [`MlQuest`] in the registry. Types named by `typeof(X)` are short class names, matched
//! against an entity's `ty.parent` chain (`IsAssignableFrom`). The registry, the contexts and the instances live
//! in [`MlState`] in `w.resources`; references between them are ids.

use indexmap::IndexMap;
use muo_io::text::TextDefinition;
use muo_primitives::{DateTime, TimeSpan};
use muo_server::ids::{ItemId, MobileId};
use muo_server::skills::SkillName;
use muo_server::timer::TimerHandle;
use muo_server::world::World;
use std::collections::HashMap;

pub type Td = TextDefinition;

/// `TextDefinition` implicit from `int`.
pub fn td(number: i32) -> Td {
    Td::of_number(number)
}

/// `TextDefinition` implicit from `string`.
pub fn tds(text: &str) -> Td {
    Td::of_text(text)
}

/// Index of a quest in [`MlState::quests`]; stable for the run (registration order).
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct QuestId(pub usize);

/// An `MLQuestInstance`.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct InstanceId(pub u32);

/// `IQuestGiver`: a mobile (`BaseCreature`) or an item (`QuestGiverItem`).
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum Quester {
    Mobile(MobileId),
    Item(ItemId),
}

/// `QuestArea` (the `ForceMap` argument is never used by the quests).
#[derive(Clone, Debug)]
pub struct QuestArea {
    pub name: Td,
    pub region: &'static str,
}

impl QuestArea {
    pub fn new(name: Td, region: &'static str) -> Self {
        Self { name, region }
    }
}

/// `ObjectiveType`.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ObjectiveType {
    All,
    Any,
}

/// The objective classes.
#[derive(Clone, Debug)]
pub enum ObjectiveKind {
    /// `KillObjective(amount, types, name, area)`.
    Kill {
        amount: i32,
        types: Vec<&'static str>,
        name: Td,
        area: Option<QuestArea>,
    },
    /// `CollectObjective(amount, type, name)`; a subclass can hide the detail line and filter the items.
    Collect {
        amount: i32,
        ty: &'static str,
        name: Td,
        show_detailed: bool,
        check_item: Option<fn(&World, ItemId) -> bool>,
    },
    /// `DeliverObjective(delivery, amount, name, destination, spawnsDelivery)`.
    Deliver {
        delivery: &'static str,
        amount: i32,
        name: Td,
        destination: &'static str,
        spawns_delivery: bool,
    },
    /// `EscortObjective(destination)`.
    Escort { destination: Option<QuestArea> },
    /// `GainSkillObjective(skill, thresholdFixed, useReal, accelerate)`.
    GainSkill {
        skill: SkillName,
        threshold_fixed: i32,
        use_real: bool,
        accelerate: bool,
    },
}

/// A `BaseObjective`; `duration` is set by the `Timed*Objective` subclasses.
#[derive(Clone, Debug)]
pub struct Objective {
    pub kind: ObjectiveKind,
    pub duration: Option<TimeSpan>,
}

impl Objective {
    pub fn new(kind: ObjectiveKind) -> Self {
        Self {
            kind,
            duration: None,
        }
    }

    pub fn timed(mut self, duration: TimeSpan) -> Self {
        self.duration = Some(duration);
        self
    }

    pub fn kill(amount: i32, types: &[&'static str], name: Td, area: Option<QuestArea>) -> Self {
        Self::new(ObjectiveKind::Kill {
            amount,
            types: types.to_vec(),
            name,
            area,
        })
    }

    pub fn collect(amount: i32, ty: &'static str, name: Td) -> Self {
        Self::new(ObjectiveKind::Collect {
            amount,
            ty,
            name,
            show_detailed: true,
            check_item: None,
        })
    }

    pub fn deliver(
        delivery: &'static str,
        amount: i32,
        name: Td,
        destination: &'static str,
    ) -> Self {
        Self::new(ObjectiveKind::Deliver {
            delivery,
            amount,
            name,
            destination,
            spawns_delivery: true,
        })
    }

    pub fn deliver_no_spawn(
        delivery: &'static str,
        amount: i32,
        name: Td,
        destination: &'static str,
    ) -> Self {
        Self::new(ObjectiveKind::Deliver {
            delivery,
            amount,
            name,
            destination,
            spawns_delivery: false,
        })
    }

    /// A `CollectObjective` subclass with `ShowDetailed => false`.
    pub fn hide_detail(mut self) -> Self {
        if let ObjectiveKind::Collect { show_detailed, .. } = &mut self.kind {
            *show_detailed = false;
        }
        self
    }

    /// A `CollectObjective` subclass overriding `CheckItem`.
    pub fn check_item(mut self, f: fn(&World, ItemId) -> bool) -> Self {
        if let ObjectiveKind::Collect { check_item, .. } = &mut self.kind {
            *check_item = Some(f);
        }
        self
    }

    pub fn escort(destination: QuestArea) -> Self {
        Self::new(ObjectiveKind::Escort {
            destination: Some(destination),
        })
    }

    pub fn gain_skill(
        skill: SkillName,
        threshold_fixed: i32,
        use_real: bool,
        accelerate: bool,
    ) -> Self {
        Self::new(ObjectiveKind::GainSkill {
            skill,
            threshold_fixed,
            use_real,
            accelerate,
        })
    }

    /// `IsTimed`.
    pub fn is_timed(&self) -> bool {
        self.duration.is_some()
    }
}

/// The reward classes.
#[derive(Clone, Debug)]
pub enum RewardKind {
    /// `ItemReward(name, type, amount)`: `ty` is a short class name; `on_create` is an `ItemReward` subclass's
    /// `CreateItem` override that changes the new item.
    Item {
        ty: &'static str,
        amount: i32,
        on_create: Option<fn(&mut World, ItemId)>,
    },
    /// `DummyReward(name)`.
    Dummy,
}

/// A `BaseReward`.
#[derive(Clone, Debug)]
pub struct Reward {
    pub name: Td,
    pub kind: RewardKind,
}

impl Reward {
    pub fn item(name: Td, ty: &'static str, amount: i32) -> Self {
        Self {
            name,
            kind: RewardKind::Item {
                ty,
                amount,
                on_create: None,
            },
        }
    }

    /// An `ItemReward` subclass whose `CreateItem` changes the item.
    pub fn with_create(mut self, f: fn(&mut World, ItemId)) -> Self {
        if let RewardKind::Item { on_create, .. } = &mut self.kind {
            *on_create = Some(f);
        }
        self
    }

    pub fn dummy(name: Td) -> Self {
        Self {
            name,
            kind: RewardKind::Dummy,
        }
    }

    /// `BaseReward.LabelHeight`.
    pub fn label_height(&self) -> i32 {
        match self.kind {
            RewardKind::Dummy => 180,
            RewardKind::Item { .. } => 16,
        }
    }
}

/// `ItemReward`'s static rewards.
pub mod rewards {
    use super::*;

    pub fn small_bag_of_trinkets() -> Reward {
        Reward::item(td(1072268), "SmallBagOfTrinkets", 1)
    }
    pub fn bag_of_trinkets() -> Reward {
        Reward::item(td(1072341), "BagOfTrinkets", 1)
    }
    pub fn bag_of_treasure() -> Reward {
        Reward::item(td(1072583), "BagOfTreasure", 1)
    }
    pub fn large_bag_of_treasure() -> Reward {
        Reward::item(td(1072706), "LargeBagOfTreasure", 1)
    }
    pub fn strongbox() -> Reward {
        Reward::item(td(1072584), "RewardStrongbox", 1)
    }
    pub fn tailor_satchel() -> Reward {
        Reward::item(td(1074282), "TailorSatchel", 1)
    }
    pub fn blacksmith_satchel() -> Reward {
        Reward::item(td(1074282), "BlacksmithSatchel", 1)
    }
    pub fn fletching_satchel() -> Reward {
        Reward::item(td(1074282), "FletchingSatchel", 1)
    }
    pub fn carpentry_satchel() -> Reward {
        Reward::item(td(1074282), "CarpentrySatchel", 1)
    }
    pub fn tinker_satchel() -> Reward {
        Reward::item(td(1074282), "TinkerSatchel", 1)
    }
}

type CanOfferFn = fn(&mut World, QuestId, Quester, MobileId, bool) -> bool;
/// The constant overrides and virtuals of a quest class (`MLQuest` subclasses). `BASE` is `MLQuest` itself; a
/// definition is `QuestClass { full_name, create, ..BASE }`. A hook that replaces a virtual calls the base
/// behavior through the `base_*` functions of [`super::quest`].
#[derive(Copy, Clone)]
pub struct QuestClass {
    /// `GetType().FullName`.
    pub full_name: &'static str,
    /// The constructor.
    pub create: fn(&'static QuestClass) -> MlQuest,
    /// `NextQuest` (short class name).
    pub next_quest: Option<&'static str>,
    /// `IsChainTriggered`.
    pub chain_triggered: bool,
    /// `RecordCompletion => true`.
    pub record_completion: bool,
    /// `Version`.
    pub version: i32,
    /// `BaseEscort.AwardHumanInNeed`: `Some(value)` for a class under `BaseEscort`, else the class is not an escort.
    pub award_human_in_need: Option<bool>,
    pub generate: Option<fn(&mut World)>,
    pub can_offer: Option<CanOfferFn>,
    pub on_accepted: Option<fn(&mut World, InstanceId)>,
    pub get_rewards: Option<fn(&mut World, InstanceId)>,
    pub on_reward_claimed: Option<fn(&mut World, InstanceId)>,
    pub on_cancel: Option<fn(&mut World, InstanceId)>,
    pub on_quester_deleted: Option<fn(&mut World, InstanceId)>,
    pub on_player_death: Option<fn(&mut World, InstanceId)>,
    pub get_restart_delay: Option<fn(&mut World) -> TimeSpan>,
    pub refresh: Option<fn(&mut World, i32)>,
}

fn base_create(class: &'static QuestClass) -> MlQuest {
    MlQuest::new(class)
}

impl QuestClass {
    /// `MLQuest`.
    pub const BASE: QuestClass = QuestClass {
        full_name: "Server.Engines.MLQuests.MLQuest",
        create: base_create,
        next_quest: None,
        chain_triggered: false,
        record_completion: false,
        version: 0,
        award_human_in_need: None,
        generate: None,
        can_offer: None,
        on_accepted: None,
        get_rewards: None,
        on_reward_claimed: None,
        on_cancel: None,
        on_quester_deleted: None,
        on_player_death: None,
        get_restart_delay: None,
        refresh: None,
    };

    /// `Type.Name`.
    pub fn name(&self) -> &'static str {
        short_name(self.full_name)
    }
}

/// The text after the last `.`.
pub fn short_name(full_name: &str) -> &str {
    full_name.rsplit('.').next().unwrap_or(full_name)
}

/// `MLQuest`'s completion notices.
pub fn completion_notice_default() -> Td {
    td(1072273)
}
pub fn completion_notice_short() -> Td {
    td(1046258)
}
pub fn completion_notice_short_return() -> Td {
    td(1073775)
}
pub fn completion_notice_craft() -> Td {
    td(1073967)
}

/// An `MLQuest`.
pub struct MlQuest {
    pub class: &'static QuestClass,
    pub deserialized: bool,
    pub save_enabled: bool,
    pub activated: bool,
    pub objectives: Vec<Objective>,
    pub objective_type: ObjectiveType,
    pub rewards: Vec<Reward>,
    pub instances: Vec<InstanceId>,
    pub one_time_only: bool,
    pub has_restart_delay: bool,
    pub title: Td,
    pub description: Td,
    pub refusal_message: Td,
    pub in_progress_message: Td,
    pub completion_message: Td,
    pub completion_notice: Td,
}

impl MlQuest {
    /// `MLQuest()`.
    pub fn new(class: &'static QuestClass) -> Self {
        Self {
            class,
            deserialized: false,
            save_enabled: true,
            activated: false,
            objectives: Vec::new(),
            objective_type: ObjectiveType::All,
            rewards: Vec::new(),
            instances: Vec::new(),
            one_time_only: false,
            has_restart_delay: false,
            title: Td::EMPTY,
            description: Td::EMPTY,
            refusal_message: Td::EMPTY,
            in_progress_message: Td::EMPTY,
            completion_message: Td::EMPTY,
            completion_notice: completion_notice_default(),
        }
    }

    /// `IsEscort`.
    pub fn is_escort(&self) -> bool {
        self.objectives
            .iter()
            .any(|o| matches!(o.kind, ObjectiveKind::Escort { .. }))
    }

    /// `IsSkillTrainer`.
    pub fn is_skill_trainer(&self) -> bool {
        self.objectives
            .iter()
            .any(|o| matches!(o.kind, ObjectiveKind::GainSkill { .. }))
    }

    /// `RequiresCollection`.
    pub fn requires_collection(&self) -> bool {
        self.objectives.iter().any(|o| {
            matches!(
                o.kind,
                ObjectiveKind::Collect { .. } | ObjectiveKind::Deliver { .. }
            )
        })
    }

    /// `RecordCompletion`.
    pub fn record_completion(&self) -> bool {
        self.class.record_completion || self.one_time_only || self.has_restart_delay
    }

    /// `IsChainTriggered`.
    pub fn is_chain_triggered(&self) -> bool {
        self.class.chain_triggered
    }
}

/// `MLQuestFlag`.
pub mod flag {
    pub const SPELLWEAVING: i32 = 0x01;
    pub const SUMMON_FEY: i32 = 0x02;
    pub const SUMMON_FIEND: i32 = 0x04;
    pub const BEDLAM_ACCESS: i32 = 0x08;
}

/// `MLDoneQuestInfo`.
#[derive(Clone, Debug)]
pub struct DoneQuest {
    pub quest: QuestId,
    pub next_available: DateTime,
}

/// An `MLQuestContext`.
#[derive(Clone, Debug)]
pub struct Context {
    pub owner: MobileId,
    pub quest_instances: Vec<InstanceId>,
    pub done_quests: Vec<DoneQuest>,
    pub chain_offers: Vec<QuestId>,
    pub flags: i32,
}

impl Context {
    pub fn new(owner: MobileId) -> Self {
        Self {
            owner,
            quest_instances: Vec::new(),
            done_quests: Vec::new(),
            chain_offers: Vec::new(),
            flags: 0,
        }
    }

    /// `GetFlag`.
    pub fn get_flag(&self, flag: i32) -> bool {
        self.flags & flag != 0
    }

    /// `SetFlag`.
    pub fn set_flag(&mut self, flag: i32, value: bool) {
        if value {
            self.flags |= flag;
        } else {
            self.flags &= !flag;
        }
    }
}

/// `MLQuestInstanceFlags`.
pub mod iflag {
    pub const CLAIM_REWARD: u8 = 0x01;
    pub const REMOVED: u8 = 0x02;
    pub const FAILED: u8 = 0x04;
}

/// The per-instance state of an objective (`BaseObjectiveInstance` and its subclasses).
#[derive(Clone, Debug)]
pub enum ObjectiveState {
    Kill { slain: i32 },
    Collect,
    Deliver { has_completed: bool },
    Escort(EscortState),
    GainSkill,
}

#[derive(Clone, Debug)]
pub struct EscortState {
    /// `m_Escort`: the quester as a `BaseCreature` when the objective was made.
    pub escort: Option<MobileId>,
    pub has_completed: bool,
    pub last_seen_escorter: DateTime,
    pub timer: TimerHandle,
}

/// A `BaseObjectiveInstance`.
#[derive(Clone, Debug)]
pub struct ObjectiveInstance {
    /// `DateTime.MinValue` when the objective is not timed.
    pub end_time: DateTime,
    pub expired: bool,
    pub state: ObjectiveState,
}

impl ObjectiveInstance {
    /// `IsTimed`.
    pub fn is_timed(&self) -> bool {
        self.end_time != DateTime::MIN
    }
}

/// An `MLQuestInstance`.
pub struct Instance {
    pub quest: QuestId,
    pub quester: Option<Quester>,
    /// `QuesterType`: the full name of the quester's class.
    pub quester_type: Option<&'static str>,
    pub player: MobileId,
    pub accepted: DateTime,
    pub flags: u8,
    pub objectives: Vec<ObjectiveInstance>,
    pub timer: TimerHandle,
}

impl Instance {
    pub fn claim_reward(&self) -> bool {
        self.flags & iflag::CLAIM_REWARD != 0
    }
    pub fn removed(&self) -> bool {
        self.flags & iflag::REMOVED != 0
    }
    pub fn failed(&self) -> bool {
        self.flags & iflag::FAILED != 0
    }
    pub fn set_flag(&mut self, flag: u8, value: bool) {
        if value {
            self.flags |= flag;
        } else {
            self.flags &= !flag;
        }
    }
}

/// `MLQuestSystem`'s statics.
#[derive(Default)]
pub struct MlState {
    pub enabled: bool,
    /// `Quests` (`Dictionary<Type, MLQuest>`, in insertion order): full class name -> id.
    pub by_class: IndexMap<String, QuestId>,
    pub quests: Vec<MlQuest>,
    /// `QuestGivers`: lower-case short class name of the quester -> its quests.
    pub quest_givers: IndexMap<String, Vec<QuestId>>,
    /// `Contexts`.
    pub contexts: IndexMap<MobileId, Context>,
    pub instances: IndexMap<InstanceId, Instance>,
    pub next_instance: u32,
    /// `BaseCreature.m_MLNextShout` (not saved).
    pub next_shout: HashMap<MobileId, DateTime>,
    /// `MLQuestPersistence.m_Instance`.
    pub persistence: Option<ItemId>,
}

pub fn state(w: &mut World) -> &mut MlState {
    w.resources.get_mut::<MlState>()
}

pub fn state_ref(w: &World) -> Option<&MlState> {
    w.resources.get::<MlState>()
}
