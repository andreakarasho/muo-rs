//! `Mobile` statics of the second half, plus the ones only this half reads.
//! They are one value in the world's resources: `w.resources.get_mut::<Part2Statics>()`.
//!
//! // shortcut: most of these belong with the first half (`DragEffects`, `ActionDelay`, the regen handlers..);
//! the first half reads them from here (one copy).

use std::collections::VecDeque;

use indexmap::{IndexMap, IndexSet};
use muo_primitives::TimeSpan;

use crate::ids::{ItemId, MobileId};
use crate::skills::SkillName;
use crate::target::Targeted;
use crate::world::World;

pub type RegenRateHandler = fn(&World, MobileId) -> TimeSpan;

/// What the regen rate handlers do with `&mut` access before they compute the rate (`CheckBonusSkill`): run
/// by the regen timers right before `GetXRegenRate`. The `i32` is 0 for hits, 1 for stamina, 2 for mana.
pub type RegenPreRateHandler = fn(&mut World, MobileId, i32);

type SkillCheckLocationHandlerFn = fn(&mut World, MobileId, SkillName, f64, f64) -> bool;
type CreateCorpseHandlerFn = fn(&mut World, MobileId, &[ItemId], &[ItemId]) -> Option<ItemId>;
type SkillCheckDirectLocationHandlerFn = fn(&mut World, MobileId, SkillName, f64) -> bool;
type SkillCheckTargetHandlerFn = fn(&mut World, MobileId, SkillName, Targeted, f64, f64) -> bool;
type SkillCheckDirectTargetHandlerFn = fn(&mut World, MobileId, SkillName, Targeted, f64) -> bool;
pub struct Part2Statics {
    /// `m_DeltaQueue`.
    pub delta_queue: VecDeque<MobileId>,
    /// `GhostChars`; `None` reads as `DefaultGhostChars`.
    pub ghost_chars: Option<Vec<char>>,
    /// `DisableDismountInWarmode`.
    pub disable_dismount_in_warmode: bool,
    /// `DisableHiddenSelfClick`.
    pub disable_hidden_self_click: bool,
    /// `AsciiClickMessage`.
    pub ascii_click_message: bool,
    /// `GuildClickMessage`.
    pub guild_click_message: bool,
    /// `ExpireCriminalDelay`.
    pub expire_criminal_delay: TimeSpan,
    /// `_disableCastParalyze`.
    pub disable_cast_paralyze: bool,
    pub hits_regen_rate_handler: Option<RegenRateHandler>,
    pub default_hits_rate: TimeSpan,
    pub stam_regen_rate_handler: Option<RegenRateHandler>,
    pub default_stam_rate: TimeSpan,
    pub mana_regen_rate_handler: Option<RegenRateHandler>,
    pub default_mana_rate: TimeSpan,
    pub regen_pre_rate: Option<RegenPreRateHandler>,
    /// `CreateCorpseHandler(from, initialContent, equippedItems)`.
    pub create_corpse_handler: Option<CreateCorpseHandlerFn>,
    /// `SkillCheckLocationHandler(from, skill, minSkill, maxSkill)`.
    pub skill_check_location_handler: Option<SkillCheckLocationHandlerFn>,
    /// `SkillCheckDirectLocationHandler(from, skill, chance)`.
    pub skill_check_direct_location_handler: Option<SkillCheckDirectLocationHandlerFn>,
    /// `SkillCheckTargetHandler(from, skill, target, minSkill, maxSkill)`.
    pub skill_check_target_handler: Option<SkillCheckTargetHandlerFn>,
    /// `SkillCheckDirectTargetHandler(from, skill, target, chance)`.
    pub skill_check_direct_target_handler: Option<SkillCheckDirectTargetHandlerFn>,
    /// `StableMigrations`: mobile -> the mobiles it had stabled.
    pub stable_migrations: Option<IndexMap<MobileId, IndexSet<MobileId>>>,
    /// `MurderMigrations`.
    pub murder_migrations: Option<IndexMap<MobileId, i32>>,
    /// `VirtueMigrations`.
    pub virtue_migrations: Option<IndexMap<MobileId, [i32; 8]>>,
}

impl Default for Part2Statics {
    fn default() -> Self {
        Part2Statics {
            delta_queue: VecDeque::new(),
            ghost_chars: None,
            disable_dismount_in_warmode: false,
            disable_hidden_self_click: true,
            ascii_click_message: true,
            guild_click_message: true,
            expire_criminal_delay: TimeSpan::from_mins(2),
            disable_cast_paralyze: true,
            hits_regen_rate_handler: None,
            default_hits_rate: TimeSpan::ZERO,
            stam_regen_rate_handler: None,
            default_stam_rate: TimeSpan::ZERO,
            mana_regen_rate_handler: None,
            default_mana_rate: TimeSpan::ZERO,
            regen_pre_rate: None,
            create_corpse_handler: None,
            skill_check_location_handler: None,
            skill_check_direct_location_handler: None,
            skill_check_target_handler: None,
            skill_check_direct_target_handler: None,
            stable_migrations: None,
            murder_migrations: None,
            virtue_migrations: None,
        }
    }
}

/// `DefaultGhostChars`.
pub const DEFAULT_GHOST_CHARS: [char; 2] = ['o', 'O'];

/// `m_AccessLevelNames`.
pub const ACCESS_LEVEL_NAMES: [&str; 7] = [
    "a player",
    "a counselor",
    "a game master",
    "a seer",
    "an administrator",
    "a developer",
    "an owner",
];

/// `m_InvalidBodies`.
pub const INVALID_BODIES: [i32; 5] = [32, 95, 156, 197, 198];

/// `m_GuildTypes`.
pub const GUILD_TYPES: [&str; 3] = ["", " (Chaos)", " (Order)"];

/// Reads a static, with its default when nothing wrote one yet.
pub fn read<R>(w: &World, f: impl FnOnce(&Part2Statics) -> R) -> R {
    match w.resources.get::<Part2Statics>() {
        Some(s) => f(s),
        None => f(&Part2Statics::default()),
    }
}
