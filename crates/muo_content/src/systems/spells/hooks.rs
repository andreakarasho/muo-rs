//! Systems the spell base calls that other crates own (Necromancy/Ninjitsu/Spellweaving spells, dueling, feature
//! flags, wands, parties, guilds, houses, factions...), as `fn` pointers set by the owning crate's phase function.
//! An unset hook answers as if the system were absent or inert; each field says which.

use muo_primitives::Point3D;
use muo_server::World;
use muo_server::ids::{ItemId, MobileId};
use muo_server::map::MapId;

use muo_server::interfaces::SpellId;

type SpellBlockedReasonFn = fn(&World, &'static str) -> Option<Option<String>>;
type FeintDamageReductionFn = fn(&World, Option<MobileId>, MobileId) -> Option<i32>;
type ChampionSpawnOfFn = fn(&World, MobileId) -> Option<(Point3D, MapId)>;
type NewRevenantFn =
    fn(&mut World, MobileId, MobileId, muo_primitives::TimeSpan) -> Option<MobileId>;
type HouseBlocksNatureFuryFn = fn(&World, MobileId, Point3D, MapId) -> bool;
type DuelOpponentsFn = fn(&World, MobileId, MobileId) -> Option<bool>;
type HouseCheckFn = fn(&World, ItemId, Point3D, bool, i32) -> bool;
type SafeZoneBlocksTravelFn = fn(&World, MapId, Point3D, Option<MobileId>) -> bool;
type NewGateTravelMoongateFn =
    fn(&mut World, Point3D, Option<MapId>, Point3D, Option<MapId>) -> Option<ItemId>;
#[derive(Default)]
pub struct SpellHooks {
    /// `ProtectionSpell.Registry.TryGetValue(caster, out d)`: the protection value `d` the caster has. Unset: none.
    pub protection: Option<fn(&World, MobileId) -> Option<i32>>,
    /// `MindRotSpell.GetMindRotScalar(m, ref scalar)`: the scalar, if the mobile is under Mind Rot. Unset: none.
    pub mind_rot_scalar: Option<fn(&World, MobileId) -> Option<f64>>,
    /// `ThunderstormSpell.GetCastRecoveryMalus(m)`. Unset: 0.
    pub thunderstorm_cast_recovery_malus: Option<fn(&World, MobileId) -> i32>,
    /// `EssenceOfWindSpell.IsDebuffed(m)` then `GetFCMalus(m)`. Unset: 0.
    pub essence_of_wind_fc_malus: Option<fn(&World, MobileId) -> i32>,
    /// `AnimalForm.UnderTransformation(m)`. Unset: false.
    pub animal_form_under: Option<fn(&World, MobileId) -> bool>,
    /// `DuelContext.IsFreeConsume(caster)`. Unset: false.
    pub duel_is_free_consume: Option<fn(&World, MobileId) -> bool>,
    /// `((PlayerMobile)caster).DuelContext?.AllowSpellCast(caster, spell)`. Unset: true.
    pub duel_allow_spell_cast: Option<fn(&mut World, MobileId, SpellId) -> bool>,
    /// `FeatureFlagManager.IsSpellBlocked(type)` and the entry's reason. Unset: not blocked.
    pub spell_blocked_reason: Option<SpellBlockedReasonFn>,
    /// `ArcaneGem.ConsumeCharges(caster, count)`. Unset: false.
    pub arcane_gem_consume_charges: Option<fn(&mut World, MobileId, i32) -> bool>,
    /// `GetSlayerDamageScalar(defender)` of the caster's equipped spellbook and the defender's slayer.
    /// Unset: 1.0.
    pub slayer_damage_scalar: Option<fn(&mut World, MobileId, MobileId) -> f64>,
    /// `BaseWand.ConsumeCharge(from)`.
    pub wand_consume_charge: Option<fn(&mut World, ItemId, MobileId)>,
    /// `BaseWand.Charges`.
    pub wand_charges: Option<fn(&World, ItemId) -> i32>,
    /// `ReaperFormSpell.SpellDamageBonus` of the caster's active transformation. Unset: 0.
    pub reaper_form_damage_bonus: Option<fn(&World, MobileId) -> i32>,
    /// `WeaponAbility.ClearCurrentAbility(m)`. Unset: nothing.
    pub clear_current_weapon_ability: Option<fn(&mut World, MobileId)>,
    /// `Feint.GetDamageReduction(from, target, out reduction)`. Unset: none.
    pub feint_damage_reduction: Option<FeintDamageReductionFn>,
    /// `Mysticism.SpellPlagueSpell.OnMobileDamaged(target)`.
    pub spell_plague_on_damaged: Option<fn(&mut World, MobileId)>,
    /// `StaminaSystem.DFA = dfa` (the damage-from-attacker algorithm for the next AOS damage).
    pub set_dfa: Option<fn(&mut World, i32)>,
    /// `DuelContext.CheckSuddenDeath(m)`. Unset: false.
    pub duel_check_sudden_death: Option<fn(&World, MobileId) -> bool>,
    /// `TreasureMapChest.Level`. Unset: 0.
    pub treasure_map_chest_level: Option<fn(&World, ItemId) -> i32>,
    /// `item is ITelekinesisable t` then `t.OnTelekinesis(from)` (BaseDoor, Dice; TrappableContainer is handled by
    /// the container class): true if the item is one. Unset: false.
    pub item_on_telekinesis: Option<fn(&mut World, ItemId, MobileId) -> bool>,
    /// `item is ITelekinesisable` for the classes `item_on_telekinesis` runs. Unset: no item is.
    pub item_is_telekinesisable: Option<fn(&World, ItemId) -> bool>,
    /// `BaseHouse.CheckLockedDownOrSecured(item)` / `BaseHouse.CheckSecured(item)`. Unset: false.
    pub house_locked_down_or_secured: Option<fn(&World, ItemId) -> bool>,
    pub house_secured: Option<fn(&World, ItemId) -> bool>,
    /// `item is Moongate { Dispellable: false }`. Unset: false.
    pub moongate_not_dispellable: Option<fn(&World, ItemId) -> bool>,
    /// `PlagueBeastLord.OnParalyzed(from)` when the mobile is one: true if it was. Unset: false.
    pub plague_beast_lord_on_paralyzed: Option<fn(&mut World, MobileId, MobileId) -> bool>,
    /// `DisguisePersistence.StopTimer(m)`. Unset: nothing.
    pub disguise_stop_timer: Option<fn(&mut World, MobileId)>,
    /// `item is RecallRune` and `rune.Mark(from)`. Unset: not a rune.
    pub is_recall_rune: Option<fn(&World, ItemId) -> bool>,
    pub rune_mark: Option<fn(&mut World, ItemId, MobileId)>,
    /// `new GateTravelMoongate(target, targetMap)` moved to `(at, atMap)` (a Moongate
    /// subclass): the gate, and `a.LinkedGate = b; b.LinkedGate = a`. Unset: no gate is made.
    pub new_gate_travel_moongate: Option<NewGateTravelMoongateFn>,
    pub link_gate_travel_moongates: Option<fn(&mut World, ItemId, ItemId)>,
    /// `m.Region.GetRegion<ChampionSpawnRegion>()?.Spawn`: the spawn's location and map. Unset: none.
    pub champion_spawn_of: Option<ChampionSpawnOfFn>,
    /// `Faction.Find(m)` is the caster's faction and `m.Map == Faction.Facet` (Exorcism): true if so. Unset: false.
    pub faction_same_as_caster: Option<fn(&World, MobileId, MobileId) -> bool>,
    /// `new Revenant(caster, target, duration)` (Vengeful Spirit). Unset: none is made.
    pub new_revenant: Option<NewRevenantFn>,
    /// `item is MaabusCoffinComponent { Addon: MaabusCoffin }` and the Dark Tides quest step (Animate Dead): true if
    /// the item was one (the spell then ends). Unset: false.
    pub animate_maabus_coffin: Option<fn(&mut World, ItemId, MobileId) -> bool>,
    /// `BaseFormTalisman.EntryEnabled(m, type)` (a class name). Unset: enabled.
    pub form_talisman_entry_enabled: Option<fn(&World, MobileId, &str) -> bool>,
    /// A Cu Sidhe form heals with a bandage: `Backpack?.FindItemByType<Bandage>()` then `Consume()`: true if one was
    /// used. Unset: false.
    pub consume_bandage: Option<fn(&mut World, MobileId) -> bool>,
    /// `new Clone(caster).MoveToWorld(caster.Location, caster.Map)` (Mirror Image). Unset: no clone is made.
    pub new_mirror_image_clone: Option<fn(&mut World, MobileId)>,
    /// `new AnimatedWeapon(caster, level)`: gives the new creature its caster and level. Unset: nothing.
    pub init_animated_weapon: Option<fn(&mut World, MobileId, MobileId, i32)>,
    /// `state.SupportsExpansion(Expansion.ML)` of the mobile's client. Unset: supported.
    pub net_supports_ml: Option<fn(&World, MobileId) -> bool>,
    /// `MLQuestSystem.GetContext(pm)?.Spellweaving` / `SummonFey` / `SummonFiend` (the flag by name). Unset: true.
    pub ml_quest_flag: Option<fn(&World, MobileId, &str) -> bool>,
    /// `Region.Find(p, map).GetRegion<HouseRegion>()?.House?.IsFriend(caster) == false` (Nature's Fury). Unset: false.
    pub house_blocks_nature_fury: Option<HouseBlocksNatureFuryFn>,
    /// The Gift of Life death of a bonded pet: the master's `PetResurrectGump(master, pet, hitsScalar)`, else a
    /// friend's. Unset: nothing.
    pub pet_resurrect_gump: Option<fn(&mut World, MobileId, f64)>,
    /// `Sigil.ExistsOn(m)`. Unset: false.
    pub sigil_exists_on: Option<fn(&World, MobileId) -> bool>,
    /// `DisguisePersistence.IsDisguised(m)`. Unset: false.
    pub is_disguised: Option<fn(&World, MobileId) -> bool>,
    /// `Party.Get(from)?.Contains(to)`. Unset: false.
    pub party_contains: Option<fn(&World, MobileId, MobileId) -> bool>,
    /// `Guild` of a mobile (`m.Guild as Guild`), and `IsAlly`. Unset: no guilds.
    pub guild_of: Option<fn(&World, MobileId) -> Option<u32>>,
    pub guild_is_ally: Option<fn(&World, u32, u32) -> bool>,
    /// `DuelContext`: the two mobiles' participants differ while a duel runs (SpellHelper.ValidIndirectTarget's duel
    /// branch): `Some(true)` they are opponents, `Some(false)` same side, `None` no running duel shared.
    pub duel_opponents: Option<DuelOpponentsFn>,
    /// `pm.DuelContext?.Started != true || pm.DuelContext.Finished || pm.DuelContext.Ruleset.GetOption("Skills", name)`
    /// (Poison's total). Unset: true (no duel).
    pub duel_allows_skill: Option<fn(&World, MobileId, &str) -> bool>,
    /// `BaseHouse.IsInside(p, 16)` / `InRange(p, range)` over the multis at `p` (CheckMulti's house branch).
    pub house_check: Option<HouseCheckFn>,
    /// `multi.Contains(p)` for a multi that isn't a house.
    pub multi_contains: Option<fn(&World, ItemId, Point3D) -> bool>,
    /// `Region.IsPartOf<SafeZone>()`, `StrongholdRegion`, `ChampionSpawnRegion` and `MondainsLegacy.IsMLRegion`
    /// for the region at a location, and the duel-elimination state of the caster (travel validators 11-13, 18).
    pub safe_zone_blocks_travel: Option<SafeZoneBlocksTravelFn>,
    pub is_faction_stronghold: Option<fn(&World, MapId, Point3D) -> bool>,
    pub is_champion_spawn: Option<fn(&World, MapId, Point3D) -> bool>,
    pub is_ml_dungeon: Option<fn(&World, MapId, Point3D) -> bool>,
    /// `PlayerMobile.DuelContext?.Started` and not eliminated: the town rule of `SpellHelper.IsTown` for a safe zone.
    pub duel_in_progress_for: Option<fn(&World, MobileId) -> bool>,
    /// What `RecallSpellTarget.OnTarget` finds in an item: a rune, a runebook, a boat key (when `to_boat`) or a house
    /// raffle deed.
    pub resolve_recall_source:
        Option<fn(&mut World, muo_server::ids::EntityId, bool) -> super::targeting::RecallSource>,
}

pub fn hooks(w: &World) -> Option<&SpellHooks> {
    w.resources.get::<SpellHooks>()
}

pub fn hooks_mut(w: &mut World) -> &mut SpellHooks {
    w.resources.get_mut::<SpellHooks>()
}
