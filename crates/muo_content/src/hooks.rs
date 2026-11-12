//! Calls this crate makes into systems that live in other crates (spells, items, virtues, factions...), as `fn`
//! pointers in `w.resources` set by the owning module's `register`. An unset hook behaves as if the
//! system were absent or inert (noted per field).

use muo_primitives::utility;
use muo_server::ids::{ItemId, MobileId};
use muo_server::world::World;

/// `Utility.Random*` over the world's generator.
pub mod rnd {
    use super::*;

    pub fn double(w: &mut World) -> f64 {
        utility::random_double(&mut w.rng)
    }
    /// `Utility.Random(count)`: `[0, count)`.
    pub fn below(w: &mut World, count: i32) -> i32 {
        utility::random(&mut w.rng, count)
    }
    /// `Utility.Random(from, count)`.
    pub fn from(w: &mut World, from: i32, count: i32) -> i32 {
        utility::random_from(&mut w.rng, from, count)
    }
    pub fn min_max(w: &mut World, min: i32, max: i32) -> i32 {
        utility::random_min_max(&mut w.rng, min, max)
    }
    pub fn bool(w: &mut World) -> bool {
        utility::random_bool(&mut w.rng)
    }
}

/// `AOS.Damage`'s parameters beyond the two mobiles.
#[derive(Copy, Clone, Debug, Default)]
pub struct AosDamage {
    pub amount: i32,
    pub ignore_armor: bool,
    pub phys: i32,
    pub fire: i32,
    pub cold: i32,
    pub pois: i32,
    pub nrgy: i32,
    pub chaos: i32,
    pub direct: i32,
}

type AosDamageFn = fn(&mut World, MobileId, Option<MobileId>, AosDamage) -> i32;
type NewAcidFn = fn(
    &mut World,
    muo_primitives::TimeSpan,
    muo_primitives::TimeSpan,
    i32,
    i32,
) -> muo_server::ids::ItemId;
type CreatureUnregisterSummonFn = fn(&mut World, MobileId, Option<MobileId>, bool);
type ParagonCheckConvertFn =
    fn(&mut World, MobileId, muo_primitives::Point3D, muo_server::map::MapId) -> bool;
type EthicShieldedFn = fn(&World, MobileId, Option<i32>) -> bool;

#[derive(Default)]
pub struct ContentHooks {
    /// `Loot.RandomTalisman()`. Unset: none.
    pub random_talisman: Option<fn(&mut World) -> ItemId>,
    /// `new StaffRobe(level)` (Items/Suits). Unset: no robe.
    pub new_staff_robe: Option<fn(&mut World, muo_server::mobile::AccessLevel) -> ItemId>,
    /// `from.SendGump(new RenounceYoungGump())` unless `from.HasGump<RenounceYoungGump>()`. Unset: nothing.
    pub renounce_young_gump: Option<fn(&mut World, MobileId)>,
    /// `/Password` from another IP: `PageQueue.GetEntry` (already queued), `PageQueue.CheckAllowedToPage`, then the two messages and `PageQueue.Enqueue(new
    /// PageEntry(from, text, PageType.Account))`. Unset (no help system): the mismatch message only.
    pub page_account_password: Option<fn(&mut World, MobileId, &str)>,
    /// `if (mobile.Target is Bola.BolaTarget) Target.Cancel(mobile)` (BaseMount.Rider). Unset: nothing.
    pub cancel_bola_target: Option<fn(&mut World, MobileId)>,
    /// `AOS.Damage(m, from, ...)`: returns the damage dealt. Unset: plain `Mobile.Damage` of the amount (0 dealt).
    pub aos_damage: Option<AosDamageFn>,
    /// `Paragon.CheckConvert(bc, location, map)`. Unset: false.
    pub paragon_check_convert: Option<ParagonCheckConvertFn>,
    /// `Paragon.Convert(bc)` / `Paragon.UnConvert(bc)`.
    pub paragon_convert: Option<fn(&mut World, MobileId, bool)>,
    /// `Paragon.HitsBuff`. Unset: 5.0.
    pub paragon_hits_buff: Option<f64>,
    /// `Paragon.ChestChance` and `Paragon.ChocolateIngredientChance`. Unset: 0.
    pub paragon_chest_chance: Option<f64>,
    pub paragon_chocolate_chance: Option<f64>,
    /// `Evasion.CheckSpellEvasion(target)`. Unset: false.
    pub check_spell_evasion: Option<fn(&mut World, MobileId) -> bool>,
    /// The barding of a player's `SwampDragon` mount (AOS.Damage): `(mount owner, damage)` returns the damage left
    /// after the barding took its share, and reports a destroyed barding itself. Unset: all of it.
    pub swamp_dragon_absorb: Option<fn(&mut World, MobileId, i32) -> i32>,
    /// `MeerMage.StopEffect(m, message)`. Unset: nothing.
    pub meer_mage_stop: Option<fn(&mut World, MobileId, bool)>,
    /// `EtherealMount.Rider` of a mount item: the rider is its `Mobile.Mount` (as for a `VirtualMountItem`). Unset: none.
    pub ethereal_mount_rider: Option<fn(&World, muo_server::ids::ItemId) -> Option<MobileId>>,
    /// `Mount.Rider = null` of a rider on an `EtherealMount`. Unset: nothing.
    pub ethereal_dismount: Option<fn(&mut World, MobileId)>,
    /// `EtherealMount.StopMounting(mob)`. Unset: nothing.
    pub ethereal_stop_mounting: Option<fn(&mut World, MobileId)>,
    /// `new Acid(duration, tickRate, minDamage, maxDamage)` (Items/Misc). Unset: a default `Acid`.
    pub new_acid: Option<NewAcidFn>,
    /// `EvilOmenSpell.EndEffect(m)`. Unset: false.
    pub evil_omen_end_effect: Option<fn(&mut World, MobileId) -> bool>,
    /// `BloodOathSpell.GetBloodOath(from)`: the creature `from` has a blood oath with. Unset: none.
    pub blood_oath_of: Option<fn(&World, MobileId) -> Option<MobileId>>,
    /// `BloodOathSpell.ComputeReflectedDamage(damage, magicResist, sa)`.
    pub blood_oath_reflected: Option<fn(i32, f64, bool) -> i32>,
    /// `BaseCreature.OnDelete`: `AnimateDeadSpell.Unregister(SummonMaster, bc)` when `IsAnimatedDead`, and
    /// `SummonFamiliarSpell.Unregister(SummonMaster, bc)` when there is a summon master. Unset: nothing.
    pub creature_unregister_summon: Option<CreatureUnregisterSummonFn>,
    /// `AnimateDeadSpell.Register(SummonMaster, bc)` after loading an animated dead creature. Unset: nothing.
    pub animate_dead_register: Option<fn(&mut World, Option<MobileId>, MobileId)>,
    /// `InvisibilitySpell.StopTimer(bc)` in `BaseCreature.RevealingAction`. Unset: nothing.
    pub invisibility_stop_timer: Option<fn(&mut World, MobileId)>,
    /// `SummonFamiliarSpell.Table.TryGetValue(controlMaster, out var bc) && bc == this` (`IsNecroFamiliar`). Unset: false.
    pub necro_familiar_of: Option<fn(&World, MobileId, MobileId) -> bool>,
    /// `SummonFamiliarSpell.Table.TryGetValue(master, out var bc)` as a live `DeathAdder` (`bc is DeathAdder
    /// { Deleted: false }`): that familiar. Unset: none.
    pub death_adder_familiar_of: Option<fn(&World, MobileId) -> Option<MobileId>>,
    /// `PoisonImpl.IncreaseLevel(poison)` is `Poisons::increase_level` in muo_server.
    /// `Faction.Find` allegiance of `m` toward the creature (`GetFactionAllegiance`). Unset: None.
    pub faction_allegiance:
        Option<fn(&World, MobileId, MobileId) -> crate::creature::enums::Allegiance>,
    /// `Ethic` allegiance (`GetEthicAllegiance`). Unset: None.
    pub ethic_allegiance:
        Option<fn(&World, MobileId, MobileId) -> crate::creature::enums::Allegiance>,
    /// `pl = Ethics.Player.Find(m); pl.IsShielded && (ourEthic == null || ourEthic == pl.Ethic)` for `IsEnemy`; the 2nd argument is the creature's `EthicAllegiance` (Hero 0 / Evil 1). Unset: false.
    pub ethic_shielded: Option<EthicShieldedFn>,
    /// `Ethics.Player.Find(aggressor, true)`, and `FinishShield()` when it `IsShielded` (`AggressiveAction`). Unset: nothing.
    pub ethic_finish_shield: Option<fn(&mut World, MobileId)>,
    /// `VirtueSystem.GetVirtues(pm)?.HonorActive || under Ethereal Voyage` for `IsEnemy`. Unset: false.
    pub enemy_protection: Option<fn(&World, MobileId) -> bool>,
    /// `BaseWeapon.OnHit(attacker, defender)` for `StunAttack`.
    pub weapon_on_hit: Option<fn(&mut World, MobileId, MobileId)>,
    /// `defender.Weapon is BaseRanged`.
    pub uses_ranged_weapon: Option<fn(&mut World, MobileId) -> bool>,
    /// `Insurance.Enabled && item.Insured`, `IAosItem.SpellChanneling > 0` and `IDurability.MaxHitPoints > 0` of an
    /// item for `DestroyEquipment`: true when the item may be destroyed.
    pub destroy_equipment_candidate: Option<fn(&World, ItemId) -> bool>,
    /// `Mobile.Weapon.MaxRange` (ArcherAI).
    pub weapon_max_range: Option<fn(&mut World, MobileId) -> i32>,
    /// `SpellHelper.ValidIndirectTarget(from, to)`. Unset: valid.
    pub valid_indirect_target: Option<fn(&mut World, MobileId, MobileId) -> bool>,
    /// `AnimalTaming.CheckMastery(from, creature)`. Unset: false.
    pub animal_taming_mastery: Option<fn(&World, MobileId, MobileId) -> bool>,
    /// `BaseInstrument.GetBaseDifficulty(creature)`. Unset: 0.
    pub instrument_base_difficulty: Option<fn(&mut World, MobileId) -> f64>,
    /// Opens `ConfirmReleaseGump(from, pet)`.
    pub confirm_release_gump: Option<fn(&mut World, MobileId, MobileId)>,
    /// The taming context-menu entry: `UseSkill(AnimalTaming)` then the target on the creature.
    pub tame_from_context_menu: Option<fn(&mut World, MobileId, MobileId)>,
}

pub fn hooks(w: &World) -> Option<&ContentHooks> {
    w.resources.get::<ContentHooks>()
}

pub fn hooks_mut(w: &mut World) -> &mut ContentHooks {
    w.resources.get_mut::<ContentHooks>()
}

/// PlayerMobile-side calls (the player task sets these).
#[derive(Default)]
pub struct PetPlayerHooks {
    /// `pm.AddFollower(creature)`.
    pub add_follower: Option<fn(&mut World, MobileId, MobileId)>,
    /// `pm.RemoveFollower(creature)` and `pm.AutoStabled?.Remove(creature)`.
    pub remove_follower: Option<fn(&mut World, MobileId, MobileId)>,
    /// `pm.Learning` (-1 when none).
    pub learning: Option<fn(&World, MobileId) -> i32>,
    pub set_learning: Option<fn(&mut World, MobileId, i32)>,
    /// `pm.Young`.
    pub young: Option<fn(&World, MobileId) -> bool>,
    /// `pm.AllFollowers` (a player's pets), for `TeleportPets`.
    pub all_followers: Option<fn(&World, MobileId) -> Vec<MobileId>>,
}

pub fn pet_player_hooks(w: &World) -> Option<&PetPlayerHooks> {
    w.resources.get::<PetPlayerHooks>()
}

type HouseBanLocationFn =
    fn(&World, MobileId) -> Option<(muo_primitives::Point3D, muo_server::map::MapId)>;
type ComputeKillAwardsFn = fn(&mut World, MobileId) -> (i32, i32);
/// Calls into the corpse/item classes (the items task) and regions for `BaseCreature`.
#[derive(Default)]
pub struct WorldHooks {
    /// `BaseMount.Rider` of a creature that is a mount. Unset: none.
    pub mount_rider: Option<fn(&World, MobileId) -> Option<MobileId>>,
    /// `Region.IsPartOf<GuardedRegion>()` at the mobile's location. Unset: false.
    pub in_guarded_region: Option<fn(&World, MobileId) -> bool>,
    /// `Region.IsPartOf<HouseRegion>()`. Unset: false.
    pub in_house_region: Option<fn(&World, MobileId) -> bool>,
    /// `BaseHouse.FindHouseAt(m)`: the house's `(BanLocation, Map)`. Unset: no house.
    pub house_ban_location: Option<HouseBanLocationFn>,
    /// `Corpse.Animated`, `Corpse.Carved`.
    pub corpse_animated: Option<fn(&World, ItemId) -> bool>,
    /// `WayPoint.NextPoint` (skipping a deleted next point).
    pub waypoint_next: Option<fn(&World, ItemId) -> Option<ItemId>>,
    pub corpse_set_carved: Option<fn(&mut World, ItemId)>,
    /// `corpse.AddCarvedItem(item, from)`.
    pub corpse_add_carved_item: Option<fn(&mut World, ItemId, ItemId, MobileId)>,
    /// `corpse.IsCriminalAction(from)`.
    pub corpse_is_criminal_action: Option<fn(&World, ItemId, MobileId) -> bool>,
    /// `Titles.AwardFame/AwardKarma` and `Titles.ComputeKillAwards`: `(fame, karma)` of a kill.
    pub compute_kill_awards: Option<ComputeKillAwardsFn>,
    pub award_fame_karma: Option<fn(&mut World, MobileId, i32, i32)>,
    /// `Party.Get(m)`'s members, or empty.
    pub party_members: Option<fn(&World, MobileId) -> Vec<MobileId>>,
    /// Everything `OnDeath` hands a looting-rights holder (faction kill, ToT, quests, artifacts).
    pub on_killed_by_rights_holder: Option<fn(&mut World, MobileId, MobileId)>,
}

pub fn world_hooks_mut(w: &mut World) -> &mut WorldHooks {
    w.resources.get_mut::<WorldHooks>()
}

pub fn world_hooks(w: &World) -> Option<&WorldHooks> {
    w.resources.get::<WorldHooks>()
}

type OnDeathRightsHolderFn = fn(&mut World, MobileId, MobileId, Option<ItemId>);
type QuestOnKillFn = fn(&mut World, MobileId, MobileId, Option<ItemId>) -> bool;
/// More calls out of `BaseCreature`'s death and spawn paths (items, quests, factions, virtues).
#[derive(Default)]
pub struct DeathHooks {
    /// `TreasureMap.IsInHavenIsland(creature)`.
    pub in_haven_island: Option<fn(&World, MobileId) -> bool>,
    /// `PackItem(new ParagonChest(Name, level))`.
    pub pack_paragon_chest: Option<fn(&mut World, MobileId, i32)>,
    /// `PackItem(new TreasureMap(level, Map))`.
    pub pack_treasure_map: Option<fn(&mut World, MobileId, i32)>,
    /// `TreasuresOfTokuno.HandleKill(creature, holder)`: once per death, for the first looting-rights holder when the
    /// creature is on Tokuno or the holder is in the Yomotsu Mines or Fan Dancer's Dojo (`givenToTKill`).
    pub on_death_rights_holder: Option<OnDeathRightsHolderFn>,
    /// `pm.Quest?.OnKill(creature, corpse)` for a rights holder `pm` (first argument): true when the player has a
    /// quest (`givenQuestKill`, the loop then skips the other holders).
    pub quest_on_kill: Option<QuestOnKillFn>,
    /// `TheSummoningQuest.GetDaemonBonesFor(creature)`: the amount of `DaemonBone` to pack.
    pub daemon_bones_for: Option<fn(&mut World, MobileId) -> i32>,
}

pub fn death_hooks(w: &World) -> Option<&DeathHooks> {
    w.resources.get::<DeathHooks>()
}
