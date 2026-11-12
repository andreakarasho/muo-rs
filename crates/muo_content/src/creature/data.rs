//! The state `BaseCreature` adds to `Mobile` (the serialized and non-serialized fields).
//! One `CreatureData` lives in `Mobile::ext` of every `BaseCreature` subclass.

use muo_primitives::{DateTime, Point2D, Point3D};
use muo_server::ids::{ItemId, MobileId};
use muo_server::map::MapId;
use muo_server::timer::TimerHandle;
use muo_server::world::World;

use super::enums::*;
use crate::abilities::MonsterAbilityTrigger;
use crate::ai::AiState;

pub struct CreatureData {
    // --- Serialized state (field order = [SerializableField] index) ---
    pub default_ai: AIType,
    pub current_ai: AIType,
    pub range_perception: i32,
    pub range_fight: i32,
    pub range_home: i32,
    pub team: i32,
    pub fight_mode: FightMode,
    pub active_speed: f64,
    pub passive_speed: f64,
    pub current_speed: f64,
    pub active_move_speed: f64,
    pub passive_move_speed: f64,
    pub home: Point3D,
    pub home_map: Option<MapId>,
    pub controlled: bool,
    /// `_master`: the stored reference, flags ignored.
    pub master: Option<MobileId>,
    pub control_target: Option<MobileId>,
    pub control_dest: Point3D,
    pub control_order: OrderType,
    pub min_tame_skill: f64,
    /// `_tamable`: `Tamable` masks paragons, use [`CreatureData::tamable`].
    pub tamable_raw: bool,
    pub summoned: bool,
    pub summon_end: DateTime,
    pub control_slots: i32,
    pub loyalty: i32,
    pub current_way_point: Option<ItemId>,
    pub hits_max_seed: i32,
    pub stam_max_seed: i32,
    pub mana_max_seed: i32,
    pub damage_min: i32,
    pub damage_max: i32,
    pub physical_resistance_seed: i32,
    pub fire_resist_seed: i32,
    pub cold_resist_seed: i32,
    pub poison_resist_seed: i32,
    pub energy_resist_seed: i32,
    pub physical_damage: i32,
    pub fire_damage: i32,
    pub cold_damage: i32,
    pub poison_damage: i32,
    pub energy_damage: i32,
    pub owners: Vec<MobileId>,
    pub is_dead_pet: bool,
    pub is_bonded: bool,
    pub bonding_begin: DateTime,
    pub owner_abandon_time: DateTime,
    pub has_generated_loot: bool,
    pub is_paragon: bool,
    /// `_friends`: `None` is no list (distinct from an empty one).
    pub friends: Option<Vec<MobileId>>,
    pub remove_if_untamed: bool,
    pub remove_step: i32,
    /// `_pendingDeleteTimer` (the `DeleteTimer`).
    pub pending_delete_timer: TimerHandle,
    pub corpse_name_override: Option<String>,

    // --- Not serialized ---
    pub seeks_home: bool,
    pub is_stabled: bool,
    pub stabled_by: Option<MobileId>,
    pub is_prisoner: bool,
    pub debug: bool,
    pub focus_mob: Option<MobileId>,
    /// `AIObject`.
    pub ai: Option<AiState>,
    /// `_targetLocation` (herding).
    pub target_location: Option<Point2D>,
    pub failed_return_home: i32,
    pub heal_timer: TimerHandle,
    pub idle_release_time: DateTime,
    pub killers_luck: i32,
    pub ml_next_shout: DateTime,
    pub next_aura: i64,
    pub next_heal_owner_time: i64,
    pub next_heal_time: i64,
    pub next_rummage_time: i64,
    pub return_queued: bool,
    pub spawning: bool,
    /// `m_Teaching`; -1 is none.
    pub teaching: i32,
    pub bard_provoked: bool,
    pub bard_pacified: bool,
    pub bard_master: Option<MobileId>,
    pub bard_target: Option<MobileId>,
    pub bard_end_time: DateTime,
    pub no_kill_awards: bool,
    pub next_reacquire_time: i64,
    pub chaos_damage: i32,
    pub direct_damage: i32,
    pub end_flee_time: DateTime,
    pub active_triggers: MonsterAbilityTrigger,
    /// `_abilityCooldowns`: ability address -> `Core.TickCount` deadline, in insertion order.
    pub ability_cooldowns: Vec<(usize, i64)>,
    /// The `UnsummonTimer`.
    pub unsummon_timer: TimerHandle,
}

impl CreatureData {
    /// Field defaults; the constructor then sets the rest.
    pub fn new(tick_count: i64) -> CreatureData {
        CreatureData {
            default_ai: AIType::AiUseDefault,
            current_ai: AIType::AiUseDefault,
            range_perception: DEFAULT_RANGE_PERCEPTION,
            range_fight: 1,
            range_home: 10,
            team: 0,
            fight_mode: FightMode::None,
            active_speed: 0.0,
            passive_speed: 0.0,
            current_speed: 0.0,
            active_move_speed: 0.0,
            passive_move_speed: 0.0,
            home: Point3D::ZERO,
            home_map: None,
            controlled: false,
            master: None,
            control_target: None,
            control_dest: Point3D::ZERO,
            control_order: OrderType::None,
            min_tame_skill: 0.0,
            tamable_raw: false,
            summoned: false,
            summon_end: DateTime::MIN,
            control_slots: 1,
            loyalty: 0,
            current_way_point: None,
            hits_max_seed: -1,
            stam_max_seed: -1,
            mana_max_seed: -1,
            damage_min: -1,
            damage_max: -1,
            physical_resistance_seed: 0,
            fire_resist_seed: 0,
            cold_resist_seed: 0,
            poison_resist_seed: 0,
            energy_resist_seed: 0,
            physical_damage: 100,
            fire_damage: 0,
            cold_damage: 0,
            poison_damage: 0,
            energy_damage: 0,
            owners: Vec::new(),
            is_dead_pet: false,
            is_bonded: false,
            bonding_begin: DateTime::MIN,
            owner_abandon_time: DateTime::MIN,
            has_generated_loot: false,
            is_paragon: false,
            friends: None,
            remove_if_untamed: false,
            remove_step: 0,
            pending_delete_timer: TimerHandle::default(),
            corpse_name_override: None,
            seeks_home: false,
            is_stabled: false,
            stabled_by: None,
            is_prisoner: false,
            debug: false,
            focus_mob: None,
            ai: None,
            target_location: None,
            failed_return_home: 0,
            heal_timer: TimerHandle::default(),
            idle_release_time: DateTime::MIN,
            killers_luck: 0,
            ml_next_shout: DateTime::MIN,
            next_aura: 0,
            next_heal_owner_time: tick_count,
            next_heal_time: tick_count,
            next_rummage_time: 0,
            return_queued: false,
            spawning: false,
            teaching: -1,
            bard_provoked: false,
            bard_pacified: false,
            bard_master: None,
            bard_target: None,
            bard_end_time: DateTime::MIN,
            no_kill_awards: false,
            next_reacquire_time: 0,
            chaos_damage: 0,
            direct_damage: 0,
            end_flee_time: DateTime::MIN,
            active_triggers: MonsterAbilityTrigger::NONE,
            ability_cooldowns: Vec::new(),
            unsummon_timer: TimerHandle::default(),
        }
    }

    /// `Tamable`: the stored flag, false for a paragon.
    pub fn tamable(&self) -> bool {
        self.tamable_raw && !self.is_paragon
    }
}

/// The `CreatureData` of mobile `id`, if it is a creature.
pub fn cd(w: &World, id: MobileId) -> Option<&CreatureData> {
    w.mobile(id)?.ext.get::<CreatureData>()
}

pub fn cd_mut(w: &mut World, id: MobileId) -> Option<&mut CreatureData> {
    w.mobile_mut(id)?.ext.get_mut::<CreatureData>()
}

/// `m is BaseCreature`.
pub fn is_creature(w: &World, id: MobileId) -> bool {
    cd(w, id).is_some()
}

/// Reads a field of the creature, or `default` for a mobile that is not one.
pub fn read<R>(w: &World, id: MobileId, default: R, f: impl FnOnce(&CreatureData) -> R) -> R {
    cd(w, id).map_or(default, f)
}

/// Edits the creature's data; a mobile that is not a creature is left alone.
pub fn edit(w: &mut World, id: MobileId, f: impl FnOnce(&mut CreatureData)) {
    if let Some(d) = cd_mut(w, id) {
        f(d);
    }
}
