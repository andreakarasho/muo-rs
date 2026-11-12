//! The `virtual` surface of BaseCreature. A member's doc names the member. Members that only exist so a
//! subclass can change a constant are `fn(&World, MobileId) -> T` with a closure default.

use muo_primitives::TimeSpan;
use muo_server::ids::{ItemId, MobileId};
use muo_server::mobile::part1::props_a;
use muo_server::mobile::{MOBILE_BASE, MobileType};
use muo_server::poison::PoisonId;
use muo_server::world::World;

use super::data::{cd, read};
use super::enums::*;
use super::{control, misc, overrides, speeds};
use crate::abilities::{self, MonsterAbility, MonsterAbilityTrigger};
use crate::ai::AiClass;

/// `ty.family::<CreatureType>()` of mobile `id`.
pub fn ctype(w: &World, id: MobileId) -> Option<&'static CreatureType> {
    w.mobile(id)?.ty.family::<CreatureType>()
}

fn aos(w: &World) -> bool {
    props_a::core_aos(w)
}

fn dead_pet(w: &World, id: MobileId) -> bool {
    read(w, id, false, |d| d.is_dead_pet)
}

/// BaseCreature's `virtual` members.
#[allow(clippy::type_complexity)]
pub struct CreatureType {
    /// DefaultName
    pub default_name: fn(&World, MobileId) -> Option<&'static str>,
    /// CorpseName
    pub corpse_name: fn(&World, MobileId) -> Option<&'static str>,
    /// FollowsAcquireRules
    pub follows_acquire_rules: fn(&World, MobileId) -> bool,
    /// FactionSilverWorth
    pub faction_silver_worth: fn(&World, MobileId) -> i32,
    /// GetWeaponAbility
    pub get_weapon_ability: fn(&mut World, MobileId) -> Option<crate::items::weapon::Ability>,
    /// WeaponAbilityChance
    pub weapon_ability_chance: fn(&World, MobileId) -> f64,
    /// HasManaOverride
    pub has_mana_override: fn(&World, MobileId) -> bool,
    /// FavoriteFood
    pub favorite_food: fn(&World, MobileId) -> FoodType,
    /// PackInstinct
    pub pack_instinct: fn(&World, MobileId) -> PackInstinct,
    /// AllowMaleTamer
    pub allow_male_tamer: fn(&World, MobileId) -> bool,
    /// AllowFemaleTamer
    pub allow_female_tamer: fn(&World, MobileId) -> bool,
    /// SubdueBeforeTame
    pub subdue_before_tame: fn(&World, MobileId) -> bool,
    /// StatLossAfterTame
    pub stat_loss_after_tame: fn(&World, MobileId) -> bool,
    /// ReduceSpeedWithDamage
    pub reduce_speed_with_damage: fn(&World, MobileId) -> bool,
    /// IsSubdued
    pub is_subdued: fn(&World, MobileId) -> bool,
    /// Commandable
    pub commandable: fn(&World, MobileId) -> bool,
    /// HitPoison
    pub hit_poison: fn(&mut World, MobileId) -> Option<PoisonId>,
    /// HitPoisonChance
    pub hit_poison_chance: fn(&World, MobileId) -> f64,
    /// PoisonImmune
    pub poison_immune: fn(&World, MobileId) -> Option<PoisonId>,
    /// BardImmune
    pub bard_immune: fn(&World, MobileId) -> bool,
    /// Unprovokable
    pub unprovokable: fn(&World, MobileId) -> bool,
    /// Uncalmable
    pub uncalmable: fn(&World, MobileId) -> bool,
    /// AreaPeaceImmune
    pub area_peace_immune: fn(&World, MobileId) -> bool,
    /// BleedImmune
    pub bleed_immune: fn(&World, MobileId) -> bool,
    /// BonusPetDamageScalar
    pub bonus_pet_damage_scalar: fn(&World, MobileId) -> f64,
    /// DeathAdderCharmable
    pub death_adder_charmable: fn(&World, MobileId) -> bool,
    /// DispelDifficulty
    pub dispel_difficulty: fn(&World, MobileId) -> f64,
    /// DispelFocus
    pub dispel_focus: fn(&World, MobileId) -> f64,
    /// DisplayWeight (`Backpack is StrongBackpack`)
    pub display_weight: fn(&World, MobileId) -> bool,
    /// CanFly
    pub can_fly: fn(&World, MobileId) -> bool,
    /// IsInvulnerable
    pub is_invulnerable: fn(&World, MobileId) -> bool,
    /// OppositionGroup
    pub opposition_group:
        fn(&World, MobileId) -> Option<&'static crate::ai::opposition_group::OppositionGroup>,
    /// IsAnimatedDead
    pub is_animated_dead: fn(&World, MobileId) -> bool,
    /// IsNecroFamiliar
    pub is_necro_familiar: fn(&World, MobileId) -> bool,
    /// DeleteCorpseOnDeath
    pub delete_corpse_on_death: fn(&World, MobileId) -> bool,
    /// ConstantFocus
    pub constant_focus: fn(&World, MobileId) -> Option<MobileId>,
    /// DisallowAllMoves
    pub disallow_all_moves: fn(&World, MobileId) -> bool,
    /// InitialInnocent
    pub initial_innocent: fn(&World, MobileId) -> bool,
    /// AlwaysMurderer
    pub always_murderer: fn(&World, MobileId) -> bool,
    /// AlwaysAttackable
    pub always_attackable: fn(&World, MobileId) -> bool,
    /// CanOpenDoors
    pub can_open_doors: fn(&World, MobileId) -> bool,
    /// CanMoveOverObstacles
    pub can_move_over_obstacles: fn(&World, MobileId) -> bool,
    /// CanDestroyObstacles
    pub can_destroy_obstacles: fn(&World, MobileId) -> bool,
    /// CanAngerOnTame
    pub can_anger_on_tame: fn(&World, MobileId) -> bool,
    /// ForcedAI: read once per `ChangeAIType`.
    pub forced_ai: fn(&World, MobileId) -> Option<&'static AiClass>,
    /// ChaseLeashRange
    pub chase_leash_range: fn(&World, MobileId) -> i32,
    /// FollowMoveSpeed
    pub follow_move_speed: fn(&World, MobileId) -> f64,
    /// NoHouseRestrictions
    pub no_house_restrictions: fn(&World, MobileId) -> bool,
    /// IsHouseSummonable
    pub is_house_summonable: fn(&World, MobileId) -> bool,
    /// AutoDispel
    pub auto_dispel: fn(&World, MobileId) -> bool,
    /// AutoDispelChance
    pub auto_dispel_chance: fn(&World, MobileId) -> f64,
    /// IsScaryToPets
    pub is_scary_to_pets: fn(&World, MobileId) -> bool,
    /// `FactionAllegiance`: the index of the faction (the factions engine's order: 0 CouncilOfMages, 1 Minax,
    /// Shadowlords, 3 TrueBritannians), or none.
    pub faction_allegiance: fn(&World, MobileId) -> Option<i32>,
    /// `EthicAllegiance`: 0 `Ethic.Hero`, 1 `Ethic.Evil`, or none.
    pub ethic_allegiance: fn(&World, MobileId) -> Option<i32>,
    /// `Changeling.DefaultHue`.
    pub default_hue: fn(&World, MobileId) -> i32,
    /// IsScaredOfScaryThings
    pub is_scared_of_scary_things: fn(&World, MobileId) -> bool,
    /// CanRummageCorpses
    pub can_rummage_corpses: fn(&World, MobileId) -> bool,
    /// DeleteOnRelease
    pub delete_on_release: fn(&World, MobileId) -> bool,
    /// CanDrop
    pub can_drop: fn(&World, MobileId) -> bool,
    /// TreasureMapLevel
    pub treasure_map_level: fn(&mut World, MobileId) -> i32,
    /// TreasureMapChance
    pub treasure_map_chance: fn(&World, MobileId) -> f64,
    /// IgnoreYoungProtection
    pub ignore_young_protection: fn(&World, MobileId) -> bool,
    /// BaseMount StepsMax
    pub steps_max: fn(&World, MobileId) -> i32,
    /// BaseMount StepsGainedPerIdleTime
    pub steps_gained_per_idle_time: fn(&World, MobileId) -> i32,
    /// BaseMount IdleTimePerStepsGain
    pub idle_time_per_steps_gain: fn(&World, MobileId) -> TimeSpan,
    /// BaseMount MountAbilityDelay
    pub mount_ability_delay: fn(&World, MobileId) -> TimeSpan,
    /// BaseMount AllowMaleRider
    pub allow_male_rider: fn(&World, MobileId) -> bool,
    /// BaseMount AllowFemaleRider
    pub allow_female_rider: fn(&World, MobileId) -> bool,
    /// BaseMount DoMountAbility(damage, attacker)
    pub do_mount_ability: fn(&mut World, MobileId, i32, Option<MobileId>) -> bool,
    /// BaseMount OnDisallowedRider(m)
    pub on_disallowed_rider: fn(&mut World, MobileId, MobileId),
    /// SpeechType
    pub speech_type:
        fn(&World, MobileId) -> Option<&'static crate::misc::inhuman_speech::InhumanSpeech>,
    /// GivesMLMinorArtifact
    pub gives_ml_minor_artifact: fn(&World, MobileId) -> bool,
    /// ReacquireDelay
    pub reacquire_delay: fn(&World, MobileId) -> TimeSpan,
    /// AcquireOnApproachDelay
    pub acquire_on_approach_delay: fn(&World, MobileId) -> TimeSpan,
    /// AcquireOnApproachRange
    pub acquire_on_approach_range: fn(&World, MobileId) -> i32,
    /// ReacquireOnMovement
    pub reacquire_on_movement: fn(&World, MobileId) -> bool,
    /// IsDispellable
    pub is_dispellable: fn(&World, MobileId) -> bool,
    /// PlayerRangeSensitive
    pub player_range_sensitive: fn(&World, MobileId) -> bool,
    /// ReturnsToHome
    pub returns_to_home: fn(&World, MobileId) -> bool,
    /// CanShout
    pub can_shout: fn(&World, MobileId) -> bool,
    /// StandsDownOnCommand
    pub stands_down_on_command: fn(&World, MobileId) -> bool,
    /// IsBondable
    pub is_bondable: fn(&World, MobileId) -> bool,
    /// BondingDelay
    pub bonding_delay: fn(&World, MobileId) -> TimeSpan,
    /// BondingAbandonDelay
    pub bonding_abandon_delay: fn(&World, MobileId) -> TimeSpan,
    /// BreathImmune
    pub breath_immune: fn(&World, MobileId) -> bool,
    /// CanFlee
    pub can_flee: fn(&World, MobileId) -> bool,
    /// AllowNewPetFriend
    pub allow_new_pet_friend: fn(&World, MobileId) -> bool,
    /// Feathers
    pub feathers: fn(&World, MobileId) -> i32,
    /// Wool
    pub wool: fn(&World, MobileId) -> i32,
    /// MeatType
    pub meat_type: fn(&World, MobileId) -> MeatType,
    /// Meat
    pub meat: fn(&World, MobileId) -> i32,
    /// Hides
    pub hides: fn(&World, MobileId) -> i32,
    /// HideType
    pub hide_type: fn(&World, MobileId) -> HideType,
    /// Scales
    pub scales: fn(&World, MobileId) -> i32,
    /// ScaleType
    pub scale_type: fn(&mut World, MobileId) -> ScaleType,
    /// CanTeach
    pub can_teach: fn(&World, MobileId) -> bool,
    /// CanHeal
    pub can_heal: fn(&World, MobileId) -> bool,
    /// CanHealOwner
    pub can_heal_owner: fn(&World, MobileId) -> bool,
    /// HealScalar
    pub heal_scalar: fn(&World, MobileId) -> f64,
    /// HealSound
    pub heal_sound: fn(&World, MobileId) -> i32,
    /// HealStartRange
    pub heal_start_range: fn(&World, MobileId) -> i32,
    /// HealEndRange
    pub heal_end_range: fn(&World, MobileId) -> i32,
    /// HealTrigger
    pub heal_trigger: fn(&World, MobileId) -> f64,
    /// HealDelay
    pub heal_delay: fn(&World, MobileId) -> f64,
    /// HealInterval
    pub heal_interval: fn(&World, MobileId) -> f64,
    /// HealFully
    pub heal_fully: fn(&World, MobileId) -> bool,
    /// HealOwnerTrigger
    pub heal_owner_trigger: fn(&World, MobileId) -> f64,
    /// HealOwnerDelay
    pub heal_owner_delay: fn(&World, MobileId) -> f64,
    /// HealOwnerInterval
    pub heal_owner_interval: fn(&World, MobileId) -> f64,
    /// HealOwnerFully
    pub heal_owner_fully: fn(&World, MobileId) -> bool,
    /// HasAura
    pub has_aura: fn(&World, MobileId) -> bool,
    /// AuraInterval
    pub aura_interval: fn(&World, MobileId) -> TimeSpan,
    /// AuraRange
    pub aura_range: fn(&World, MobileId) -> i32,
    /// AuraBaseDamage
    pub aura_base_damage: fn(&mut World, MobileId) -> i32,
    /// AuraPhysicalDamage
    pub aura_physical_damage: fn(&World, MobileId) -> i32,
    /// AuraFireDamage
    pub aura_fire_damage: fn(&World, MobileId) -> i32,
    /// AuraColdDamage
    pub aura_cold_damage: fn(&World, MobileId) -> i32,
    /// AuraPoisonDamage
    pub aura_poison_damage: fn(&World, MobileId) -> i32,
    /// AuraEnergyDamage
    pub aura_energy_damage: fn(&World, MobileId) -> i32,
    /// AuraChaosDamage
    pub aura_chaos_damage: fn(&World, MobileId) -> i32,
    /// GetMonsterAbilities: a pure, stable query.
    pub get_monster_abilities: fn(&World, MobileId) -> &'static [&'static MonsterAbility],
    /// CanUseAbility: a pure read.
    pub can_use_ability: fn(&World, MobileId, &'static MonsterAbility) -> bool,
    /// CheckControlChance
    pub check_control_chance: fn(&mut World, MobileId, MobileId) -> bool,
    /// CanBeControlledBy
    pub can_be_controlled_by: fn(&mut World, MobileId, MobileId) -> bool,
    /// GetControlChance(m, useBaseSkill)
    pub get_control_chance: fn(&mut World, MobileId, MobileId, bool) -> f64,
    /// AddCustomContextEntries
    pub add_custom_context_entries: fn(
        &mut World,
        MobileId,
        MobileId,
        &mut Vec<muo_server::context_menu_entry::ContextMenuEntry>,
    ),
    /// IsEnemy
    pub is_enemy: fn(&mut World, MobileId, MobileId) -> bool,
    /// OnDamagedBySpell
    pub on_damaged_by_spell: fn(&mut World, MobileId, MobileId, i32),
    /// OnDamageSpell
    pub on_damage_spell: fn(&mut World, MobileId, MobileId, i32),
    /// OnHarmfulSpell
    pub on_harmful_spell: fn(&mut World, MobileId, MobileId),
    /// CheckReflect: sets `reflect`.
    pub check_reflect: fn(&mut World, MobileId, MobileId, &mut bool),
    /// OnCarve
    pub on_carve: fn(&mut World, MobileId, MobileId, ItemId, Option<ItemId>),
    /// IsHumanInTown
    pub is_human_in_town: fn(&World, MobileId) -> bool,
    /// CheckGold
    pub check_gold: fn(&mut World, MobileId, MobileId, ItemId) -> bool,
    /// OnGoldGiven
    pub on_gold_given: fn(&mut World, MobileId, MobileId, ItemId) -> bool,
    /// OverrideBondingReqs
    pub override_bonding_reqs: fn(&World, MobileId) -> bool,
    /// OnTeamChange
    pub on_team_change: fn(&mut World, MobileId),
    /// OnGotMeleeAttack
    pub on_got_melee_attack: fn(&mut World, MobileId, MobileId, i32),
    /// Dispel
    pub dispel: fn(&mut World, MobileId, MobileId),
    /// OnGaveMeleeAttack
    pub on_gave_melee_attack: fn(&mut World, MobileId, MobileId, i32),
    /// GetFightModeRanking
    pub get_fight_mode_ranking: fn(&mut World, MobileId, MobileId, FightMode, bool) -> f64,
    /// Turn
    pub turn: fn(&mut World, MobileId, i32),
    /// TurnInternal
    pub turn_internal: fn(&mut World, MobileId, i32),
    /// GetTeamSize
    pub get_team_size: fn(&mut World, MobileId, i32) -> i32,
    /// CheckIdle
    pub check_idle: fn(&mut World, MobileId) -> bool,
    /// CheckedAnimate
    pub checked_animate: fn(&mut World, MobileId, i32, i32, i32, bool, bool, i32),
    /// ForceReacquire
    pub force_reacquire: fn(&mut World, MobileId),
    /// IsMonster
    pub is_monster: fn(&World, MobileId) -> bool,
    /// OnKilledBy
    pub on_killed_by: fn(&mut World, MobileId, MobileId),
    /// OnThink
    pub on_think: fn(&mut World, MobileId),
    /// Rummage
    pub rummage: fn(&mut World, MobileId) -> bool,
    /// ResurrectPet
    pub resurrect_pet: fn(&mut World, MobileId),
    /// Shout
    pub shout: fn(&mut World, MobileId, MobileId),
    /// NewHarmfulItem: the full name of the item class (`Acid`).
    pub new_harmful_item: fn(&mut World, MobileId) -> Option<ItemId>,
    /// StopFlee
    pub stop_flee: fn(&mut World, MobileId),
    /// CheckFlee
    pub check_flee: fn(&mut World, MobileId) -> bool,
    /// BeginFlee
    pub begin_flee: fn(&mut World, MobileId, TimeSpan),
    /// IsPetFriend
    pub is_pet_friend: fn(&World, MobileId, MobileId) -> bool,
    /// AddPetFriend
    pub add_pet_friend: fn(&mut World, MobileId, MobileId),
    /// RemovePetFriend
    pub remove_pet_friend: fn(&mut World, MobileId, MobileId),
    /// ClearPetFriends
    pub clear_pet_friends: fn(&mut World, MobileId),
    /// IsFriend
    pub is_friend: fn(&World, MobileId, MobileId) -> bool,
    /// GetFactionAllegiance
    pub get_faction_allegiance: fn(&World, MobileId, MobileId) -> Allegiance,
    /// GetEthicAllegiance
    pub get_ethic_allegiance: fn(&World, MobileId, MobileId) -> Allegiance,
    /// AlterDamageScalarFrom
    pub alter_damage_scalar_from: fn(&mut World, MobileId, MobileId, &mut f64),
    /// AlterDamageScalarTo
    pub alter_damage_scalar_to: fn(&mut World, MobileId, MobileId, &mut f64),
    /// AlterSpellDamageFrom
    pub alter_spell_damage_from: fn(&mut World, MobileId, Option<MobileId>, &mut i32),
    /// AlterSpellDamageTo
    pub alter_spell_damage_to: fn(&mut World, MobileId, MobileId, &mut i32),
    /// AlterMeleeDamageFrom
    pub alter_melee_damage_from: fn(&mut World, MobileId, MobileId, &mut i32),
    /// AlterMeleeDamageTo
    pub alter_melee_damage_to: fn(&mut World, MobileId, MobileId, &mut i32),
    /// CheckFoodPreference
    pub check_food_preference: fn(&World, MobileId, ItemId) -> bool,
    /// CheckFeed
    pub check_feed: fn(&mut World, MobileId, MobileId, ItemId) -> bool,
    /// OnActionWander
    pub on_action_wander: fn(&mut World, MobileId),
    /// OnActionCombat
    pub on_action_combat: fn(&mut World, MobileId),
    /// OnActionGuard
    pub on_action_guard: fn(&mut World, MobileId),
    /// OnActionFlee
    pub on_action_flee: fn(&mut World, MobileId),
    /// OnActionInteract
    pub on_action_interact: fn(&mut World, MobileId),
    /// OnActionBackoff
    pub on_action_backoff: fn(&mut World, MobileId),
    /// CheckTeach
    pub check_teach: fn(&World, MobileId, i32, MobileId) -> bool,
    /// CheckTeachingMatch
    pub check_teaching_match: fn(&World, MobileId, MobileId) -> bool,
    /// SpeedClass
    pub speed_class: fn(&World, MobileId) -> SpeedLevel,
    /// GetSpeeds: (active, passive)
    pub get_speeds: fn(&World, MobileId) -> (f64, f64),
    /// GetMoveSpeeds: (active, passive)
    pub get_move_speeds: fn(&World, MobileId) -> (f64, f64),
    /// DropBackpack
    pub drop_backpack: fn(&mut World, MobileId),
    /// GenerateLoot(bool spawning) is [`misc::generate_loot_spawning`]; 5404 GenerateLoot() is this.
    pub generate_loot: fn(&mut World, MobileId),
    /// GenerateLoot(bool spawning), virtual: the default is [`setters::base_generate_loot_spawning`].
    pub generate_loot_spawning: fn(&mut World, MobileId, bool),
    /// HealStart
    pub heal_start: fn(&mut World, MobileId, MobileId),
    /// Heal
    pub heal: fn(&mut World, MobileId, MobileId),
    /// StopHeal
    pub stop_heal: fn(&mut World, MobileId),
    /// HealEffect
    pub heal_effect: fn(&mut World, MobileId, MobileId),
    /// AuraDamage
    pub aura_damage: fn(&mut World, MobileId),
    /// AuraEffect
    pub aura_effect: fn(&mut World, MobileId, MobileId),
    /// The class table of the first subclass level below BaseCreature that has one (`vendor::VendorClass` for
    /// BaseVendor, `guard::GuardClass`, ...), inherited with `..PARENT_C`; read through `vendor::class` and such.
    pub subclass: Option<&'static (dyn std::any::Any + Send + Sync)>,
}

pub static BASE_CREATURE_C: CreatureType = CreatureType {
    default_name: |_, _| None,
    corpse_name: |_, _| None,
    follows_acquire_rules: |_, _| true,
    faction_silver_worth: |_, _| 30,
    get_weapon_ability: |_, _| None,
    weapon_ability_chance: |_, _| 0.4,
    has_mana_override: |_, _| false,
    favorite_food: |_, _| FoodType::MEAT,
    pack_instinct: |_, _| PackInstinct::empty(),
    allow_male_tamer: |_, _| true,
    allow_female_tamer: |_, _| true,
    subdue_before_tame: |_, _| false,
    stat_loss_after_tame: |w, id| match ctype(w, id) {
        Some(t) => (t.subdue_before_tame)(w, id),
        None => false,
    },
    reduce_speed_with_damage: |_, _| true,
    is_subdued: |w, id| {
        let Some(t) = ctype(w, id) else { return false };
        (t.subdue_before_tame)(w, id)
            && w.mobile(id)
                .is_some_and(|m| m.hits < (m.ty.hits_max)(w, id) / 10)
    },
    commandable: |_, _| true,
    hit_poison: |_, _| None,
    hit_poison_chance: |_, _| 0.5,
    poison_immune: |_, _| None,
    bard_immune: |_, _| false,
    unprovokable: |w, id| ctype(w, id).is_some_and(|t| (t.bard_immune)(w, id)) || dead_pet(w, id),
    uncalmable: |w, id| ctype(w, id).is_some_and(|t| (t.bard_immune)(w, id)) || dead_pet(w, id),
    area_peace_immune: |w, id| {
        ctype(w, id).is_some_and(|t| (t.bard_immune)(w, id)) || dead_pet(w, id)
    },
    bleed_immune: |_, _| false,
    bonus_pet_damage_scalar: |_, _| 1.0,
    death_adder_charmable: |_, _| false,
    dispel_difficulty: |_, _| 0.0,
    dispel_focus: |_, _| 20.0,
    display_weight: |w, id| {
        muo_server::mobile::part2::items::find_item_on_layer(
            w,
            id,
            muo_server::layer::Layer::Backpack,
        )
        .is_some_and(|p| crate::factory::item_is(w, p, "Server.Items.StrongBackpack"))
    },
    can_fly: |_, _| false,
    is_invulnerable: |_, _| false,
    opposition_group: |_, _| None,
    is_animated_dead: misc::is_animated_dead,
    is_necro_familiar: |w, id| {
        let master = super::control::control_master(w, id);
        super::data::read(w, id, false, |d| d.summoned)
            && master.is_some_and(|m| {
                crate::hooks::hooks(w)
                    .and_then(|h| h.necro_familiar_of)
                    .is_some_and(|f| f(w, m, id))
            })
    },
    delete_corpse_on_death: |w, id| !aos(w) && read(w, id, false, |d| d.summoned),
    constant_focus: |_, _| None,
    disallow_all_moves: |_, _| false,
    initial_innocent: |_, _| false,
    always_murderer: |_, _| false,
    always_attackable: |_, _| false,
    can_open_doors: misc::can_open_doors,
    can_move_over_obstacles: misc::can_move_over_obstacles,
    can_destroy_obstacles: |_, _| false,
    can_anger_on_tame: |_, _| false,
    forced_ai: |_, _| None,
    chase_leash_range: |w, id| read(w, id, 0, |d| d.range_perception * 2),
    follow_move_speed: |w, _| if aos(w) { 0.1 } else { 0.0 },
    no_house_restrictions: |_, _| false,
    is_house_summonable: |_, _| false,
    auto_dispel: |_, _| false,
    auto_dispel_chance: |w, _| if props_a::core_se(w) { 0.10 } else { 1.0 },
    is_scary_to_pets: |_, _| false,
    faction_allegiance: |_, _| None,
    ethic_allegiance: |_, _| None,
    default_hue: |_, _| 0,
    is_scared_of_scary_things: |_, _| true,
    can_rummage_corpses: |_, _| false,
    delete_on_release: |w, id| read(w, id, false, |d| d.summoned),
    can_drop: |w, id| read(w, id, false, |d| d.is_bonded),
    treasure_map_level: |_, _| -1,
    // TreasureMap.LootChance
    treasure_map_chance: |_, _| 0.01,
    ignore_young_protection: |_, _| false,
    steps_max: |_, _| 3840,
    steps_gained_per_idle_time: |_, _| 1,
    idle_time_per_steps_gain: |_, _| TimeSpan::from_secs(1),
    mount_ability_delay: |_, _| TimeSpan::ZERO,
    allow_male_rider: |_, _| true,
    allow_female_rider: |_, _| true,
    do_mount_ability: |_, _, _, _| false,
    on_disallowed_rider: |w, _, m| crate::mx::msg(w, m, 1042317), // You may not ride at this time
    speech_type: |_, _| None,
    gives_ml_minor_artifact: |_, _| false,
    reacquire_delay: |_, _| TimeSpan::from_secs(10),
    acquire_on_approach_delay: |w, id| {
        if read(w, id, false, |d| d.is_paragon) {
            TimeSpan::ZERO
        } else {
            TimeSpan::from_secs(2)
        }
    },
    acquire_on_approach_range: |_, _| 10,
    reacquire_on_movement: |_, _| false,
    is_dispellable: |w, id| {
        read(w, id, false, |d| d.summoned)
            && !ctype(w, id).is_some_and(|t| (t.is_animated_dead)(w, id))
    },
    player_range_sensitive: |w, id| read(w, id, true, |d| d.current_way_point.is_none()),
    returns_to_home: misc::returns_to_home,
    can_shout: |_, _| false,
    stands_down_on_command: |w, _| crate::creature::misc::pets_stand_down_on_command(w),
    is_bondable: |w, id| misc::bonding_enabled(w) && !read(w, id, false, |d| d.summoned),
    bonding_delay: |_, _| TimeSpan::from_days(7),
    bonding_abandon_delay: |_, _| TimeSpan::from_days(1),
    breath_immune: |_, _| false,
    can_flee: |w, id| !read(w, id, false, |d| d.is_paragon),
    allow_new_pet_friend: |w, id| {
        read(w, id, true, |d| {
            d.friends.as_ref().is_none_or(|f| f.len() < 5)
        })
    },
    feathers: |_, _| 0,
    wool: |_, _| 0,
    meat_type: |_, _| MeatType::Ribs,
    meat: |_, _| 0,
    hides: |_, _| 0,
    hide_type: |_, _| HideType::Regular,
    scales: |_, _| 0,
    scale_type: |_, _| ScaleType::Red,
    can_teach: |_, _| false,
    can_heal: |_, _| false,
    can_heal_owner: |_, _| false,
    heal_scalar: |_, _| 1.0,
    heal_sound: |_, _| 0x57,
    heal_start_range: |_, _| 2,
    heal_end_range: |w, id| read(w, id, 0, |d| d.range_perception),
    heal_trigger: |_, _| 0.78,
    heal_delay: |_, _| 6.5,
    heal_interval: |_, _| 0.0,
    heal_fully: |_, _| true,
    heal_owner_trigger: |_, _| 0.78,
    heal_owner_delay: |_, _| 6.5,
    heal_owner_interval: |_, _| 30.0,
    heal_owner_fully: |_, _| false,
    has_aura: |_, _| false,
    aura_interval: |_, _| TimeSpan::from_secs(5),
    aura_range: |_, _| 4,
    aura_base_damage: |_, _| 5,
    aura_physical_damage: |_, _| 0,
    aura_fire_damage: |_, _| 100,
    aura_cold_damage: |_, _| 0,
    aura_poison_damage: |_, _| 0,
    aura_energy_damage: |_, _| 0,
    aura_chaos_damage: |_, _| 0,
    get_monster_abilities: |_, _| abilities::NONE,
    can_use_ability: |_, _, _| true,
    check_control_chance: super::pets::check_control_chance,
    can_be_controlled_by: super::pets::can_be_controlled_by,
    get_control_chance: super::pets::get_control_chance,
    add_custom_context_entries: |_, _, _, _| {},
    is_enemy: misc::is_enemy,
    on_damaged_by_spell: |w, id, from, _| {
        abilities::trigger_ability(
            w,
            id,
            abilities::MonsterAbilityTrigger::TAKE_SPELL_DAMAGE,
            Some(from),
        );
    },
    on_damage_spell: |w, id, defender, _| {
        abilities::trigger_ability(
            w,
            id,
            abilities::MonsterAbilityTrigger::GIVE_SPELL_DAMAGE,
            Some(defender),
        );
    },
    on_harmful_spell: |_, _, _| {},
    check_reflect: |_, _, _, _| {},
    on_carve: misc::on_carve,
    is_human_in_town: misc::is_human_in_town,
    check_gold: misc::check_gold,
    on_gold_given: misc::on_gold_given,
    override_bonding_reqs: |_, _| false,
    on_team_change: |_, _| {},
    on_got_melee_attack: misc::on_got_melee_attack,
    dispel: misc::dispel,
    on_gave_melee_attack: misc::on_gave_melee_attack,
    get_fight_mode_ranking: misc::get_fight_mode_ranking,
    turn: misc::turn,
    turn_internal: misc::turn_internal,
    get_team_size: misc::get_team_size,
    check_idle: misc::check_idle,
    checked_animate: misc::checked_animate,
    force_reacquire: |w, id| {
        let now = w.tick_count;
        super::data::edit(w, id, |d| d.next_reacquire_time = now);
    },
    is_monster: misc::is_monster,
    on_killed_by: misc::on_killed_by,
    on_think: misc::on_think,
    rummage: misc::rummage,
    resurrect_pet: misc::resurrect_pet,
    shout: |_, _, _| {},
    new_harmful_item: misc::new_harmful_item,
    stop_flee: misc::stop_flee,
    check_flee: misc::check_flee,
    begin_flee: misc::begin_flee,
    is_pet_friend: control::is_pet_friend,
    add_pet_friend: control::add_pet_friend,
    remove_pet_friend: control::remove_pet_friend,
    clear_pet_friends: control::clear_pet_friends,
    is_friend: control::is_friend,
    get_faction_allegiance: misc::get_faction_allegiance,
    get_ethic_allegiance: misc::get_ethic_allegiance,
    // `BaseCreature.Alter*Damage*`: each runs the monster abilities' alterations for its trigger.
    alter_damage_scalar_from: |w, id, other, scalar| {
        abilities::trigger_ability_alter_damage_scalar(
            w,
            id,
            MonsterAbilityTrigger::TAKE_SPELL_DAMAGE,
            other,
            scalar,
        )
    },
    alter_damage_scalar_to: |w, id, other, scalar| {
        abilities::trigger_ability_alter_damage_scalar(
            w,
            id,
            MonsterAbilityTrigger::GIVE_SPELL_DAMAGE,
            other,
            scalar,
        )
    },
    // shortcut: a null `from` skips the abilities (only MagicalBarrier reads it)
    alter_spell_damage_from: |w, id, other, damage| {
        let Some(other) = other else { return };
        abilities::trigger_ability_alter_damage(
            w,
            id,
            MonsterAbilityTrigger::TAKE_SPELL_DAMAGE,
            other,
            damage,
        )
    },
    alter_spell_damage_to: |w, id, other, damage| {
        abilities::trigger_ability_alter_damage(
            w,
            id,
            MonsterAbilityTrigger::GIVE_SPELL_DAMAGE,
            other,
            damage,
        )
    },
    alter_melee_damage_from: |w, id, other, damage| {
        abilities::trigger_ability_alter_damage(
            w,
            id,
            MonsterAbilityTrigger::TAKE_MELEE_DAMAGE,
            other,
            damage,
        )
    },
    alter_melee_damage_to: |w, id, other, damage| {
        abilities::trigger_ability_alter_damage(
            w,
            id,
            MonsterAbilityTrigger::GIVE_MELEE_DAMAGE,
            other,
            damage,
        )
    },
    check_food_preference: misc::check_food_preference,
    check_feed: misc::check_feed,
    on_action_wander: |_, _| {},
    on_action_combat: |_, _| {},
    on_action_guard: |_, _| {},
    on_action_flee: |_, _| {},
    on_action_interact: |_, _| {},
    on_action_backoff: |_, _| {},
    check_teach: misc::check_teach,
    check_teaching_match: misc::check_teaching_match,
    speed_class: |_, _| SpeedLevel::None,
    get_speeds: speeds::get_speeds,
    get_move_speeds: speeds::get_move_speeds,
    drop_backpack: misc::drop_backpack,
    generate_loot: |_, _| {},
    generate_loot_spawning: super::setters::base_generate_loot_spawning,
    heal_start: misc::heal_start,
    heal: misc::heal,
    stop_heal: misc::stop_heal,
    heal_effect: |_, _, _| {},
    aura_damage: misc::aura_damage,
    aura_effect: |_, _, _| {},
    subclass: None,
};

/// The `MobileType` of `Server.Mobiles.BaseCreature` (abstract: not registered, no constructor).
/// The `MobileType` of `Server.Mobiles.BaseCreature` (abstract: not registered, no constructor).
pub static BASE_CREATURE: MobileType = MobileType {
    full_name: "Server.Mobiles.BaseCreature",
    parent: Some(&MOBILE_BASE),
    construct: None,
    family: Some(&BASE_CREATURE_C),
    serialize: super::serialize::serialize,
    deserialize: super::serialize::deserialize,
    name: overrides::name,
    set_name: overrides::set_name,
    murderer: overrides::murderer,
    hits_max: overrides::hits_max,
    stam_max: overrides::stam_max,
    mana_max: overrides::mana_max,
    should_check_stat_timers: overrides::should_check_stat_timers,
    can_regen_hits: overrides::can_regen_hits,
    can_regen_stam: overrides::can_regen_stam,
    can_regen_mana: overrides::can_regen_mana,
    is_dead_bonded_pet: overrides::is_dead_bonded_pet,
    base_physical_resistance: overrides::base_physical_resistance,
    base_fire_resistance: overrides::base_fire_resistance,
    base_cold_resistance: overrides::base_cold_resistance,
    base_poison_resistance: overrides::base_poison_resistance,
    base_energy_resistance: overrides::base_energy_resistance,
    damage: overrides::damage,
    set_location: overrides::set_location,
    on_before_spawn: overrides::on_before_spawn,
    apply_poison: overrides::apply_poison,
    check_poison_immunity: overrides::check_poison_immunity,
    on_damage: overrides::on_damage,
    apply_name_suffix: overrides::apply_name_suffix,
    move_dir: overrides::move_dir,
    aggressive_action_criminal: overrides::aggressive_action,
    on_move_over: overrides::on_move_over,
    handles_on_speech: overrides::handles_on_speech,
    on_speech: overrides::on_speech,
    is_harmful_criminal: overrides::is_harmful_criminal,
    criminal_action: overrides::criminal_action,
    do_harmful: overrides::do_harmful,
    on_combatant_change: overrides::on_combatant_change,
    on_map_change: overrides::on_map_change,
    on_location_change: overrides::on_location_change,
    on_movement: overrides::on_movement,
    on_double_click: overrides::on_double_click,
    add_name_properties: overrides::add_name_properties,
    on_single_click: overrides::on_single_click,
    on_before_death: super::death::on_before_death,
    on_death: super::death::on_death,
    on_delete: super::death::on_delete,
    set_combatant: overrides::set_combatant,
    can_be_harmful_full: overrides::can_be_harmful_full,
    can_be_renamed_by: overrides::can_be_renamed_by,
    on_region_change: overrides::on_region_change,
    get_damage_master: overrides::get_damage_master,
    can_be_damaged: overrides::can_be_damaged,
    on_sector_activate: overrides::on_sector_activate,
    on_sector_deactivate: overrides::on_sector_deactivate,
    on_after_delete: overrides::on_after_delete,
    on_drag_drop: overrides::on_drag_drop,
    revealing_action: overrides::revealing_action,
    get_context_menu_entries: overrides::get_context_menu_entries,
    ..MOBILE_BASE
};

#[allow(dead_code)]
fn _unused(w: &World, id: MobileId) -> bool {
    cd(w, id).is_some()
}
