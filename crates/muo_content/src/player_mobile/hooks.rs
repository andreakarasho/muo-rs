//! Calls `PlayerMobile` makes into other systems (duels, house design, ethics, factions, insurance,
//! virtues, honor, quests, guilds, spells, creatures, regions). Each is a `fn` pointer in [`SystemHooks`] (in
//! `w.resources`) set by the owning module's `register`; the wrapper of the same name does the call. An unset
//! hook answers as if that system were absent or inert (the default after `=`).
//! "Content crates".

use muo_primitives::{Point3D, TimeSpan};
use muo_server::World;
use muo_server::ids::{ItemId, MobileId};
use muo_server::map::MapId;
use muo_server::property_list::PropertyList;
use muo_server::skills::SkillName;

/// `DuelContext` / `DuelPlayer` as `PlayerMobile` reads them (`None` from [`duel_state`] is a null context).
#[derive(Copy, Clone, Debug, Default)]
pub struct DuelState {
    pub registered: bool,
    pub started: bool,
    pub finished: bool,
    /// `m_Tournament != null`.
    pub tournament: bool,
    /// `DuelPlayer?.Eliminated` (true when there is no `DuelPlayer`, as `?.Eliminated != false` reads it).
    pub eliminated: bool,
    /// The player has a `DuelPlayer`.
    pub has_player: bool,
}

/// What `DesignContext` gives `PlayerMobile`: the walkable box inside the foundation.
#[derive(Copy, Clone, Debug)]
pub struct DesignBounds {
    pub start_x: i32,
    pub start_y: i32,
    pub end_x: i32,
    pub end_y: i32,
    /// `foundation.Z + HouseFoundation.GetLevelZ(context.Level, foundation)`.
    pub z: i32,
    /// `foundation.X`, `foundation.Y`.
    pub foundation_x: i32,
    pub foundation_y: i32,
    pub map: Option<MapId>,
}

/// The house at a player's location, as the context menu needs it.
#[derive(Copy, Clone, Debug, Default)]
pub struct HouseInfo {
    pub internalized_vendors: i32,
    pub is_owner: bool,
    pub is_aos_rules: bool,
    pub is_friend: bool,
    pub ban_location: Point3D,
}

/// The state of a `BaseCreature` pet `AutoStablePets`/`ClaimAutoStabledPets` read.
#[derive(Copy, Clone, Debug, Default)]
pub struct PetInfo {
    pub controlled: bool,
    pub has_control_master: bool,
    pub summoned: bool,
    pub deleted: bool,
    pub map: Option<MapId>,
    /// `(pet as IMount)?.Rider != null`.
    pub has_rider: bool,
    /// `pet is PackLlama or PackHorse or Beetle && pet.Backpack?.Items.Count > 0`.
    pub pack_animal_with_items: bool,
    /// `pet is BaseEscortable`.
    pub escortable: bool,
    pub control_slots: i32,
}

macro_rules! hooks {
    ($( $(#[$m:meta])* $name:ident ( $wty:ty ; $($arg:ident : $aty:ty),* ) -> $ret:ty = $default:expr; )*) => {
        /// One optional `fn` pointer per call into a system outside the player module.
        #[derive(Default)]
        pub struct SystemHooks {
            $( $(#[$m])* pub $name: Option<fn($wty $(, $aty)*) -> $ret>, )*
        }
        $(
            $(#[$m])*
            #[allow(unused_variables)]
            pub fn $name(w: $wty $(, $arg: $aty)*) -> $ret {
                let f = w.resources.get::<SystemHooks>().and_then(|h| h.$name);
                match f {
                    Some(f) => f(w $(, $arg)*),
                    None => $default,
                }
            }
        )*
    };
}

hooks! {
    // ---- duels (Engines/ConPVP) ----
    /// `DuelContext` / `DuelPlayer` of the player, or null.
    duel_state(&World; m: MobileId) -> Option<DuelState> = None;
    /// `a.DuelContext == b.DuelContext` (both null counts as equal).
    duel_same_context(&World; a: MobileId, b: MobileId) -> bool = true;
    /// `DuelContext?.AllowItemUse(m, item) != false`.
    duel_allow_item_use(&mut World; m: MobileId, item: ItemId) -> bool = true;
    /// `DuelContext?.AllowItemEquip(m, item) != false`.
    duel_allow_item_equip(&mut World; m: MobileId, item: ItemId) -> bool = true;
    /// `DuelContext?.AllowSkillUse(m, skill) != false`.
    duel_allow_skill_use(&mut World; m: MobileId, skill: SkillName) -> bool = true;
    /// `DuelContext?.OnLocationChanged(m)`.
    duel_on_location_changed(&mut World; m: MobileId) -> () = ();
    /// `DuelContext?.OnMapChanged(m)`.
    duel_on_map_changed(&mut World; m: MobileId) -> () = ();
    /// `m.SendEverything()` after the tournament status changed (the `DuelPlayer` setter).
    send_everything(&mut World; m: MobileId) -> () = ();

    // ---- house design (Multis/Houses/HouseFoundation) ----
    /// `DesignContext.Find(m)` as the walkable box, or null.
    design_bounds(&World; m: MobileId) -> Option<DesignBounds> = None;
    /// `DesignContext.Check(m)`: false (with the message) while customizing.
    design_check(&mut World; m: MobileId) -> bool = true;
    /// `context.Foundation.IsHiddenToCustomizer(item)`.
    design_item_hidden(&World; m: MobileId, item: ItemId) -> bool = false;
    /// The disconnect branch of `EventSink_Disconnected`: remove the context, eject all, restore entities.
    design_on_disconnect(&mut World; m: MobileId) -> () = ();
    /// `BaseHouse.FindHouseAt(m)`.
    house_at(&World; m: MobileId, from: MobileId) -> Option<HouseInfo> = None;
    /// `ReclaimVendorGump.DisplayTo(m, house)`.
    house_reclaim_vendor_gump(&mut World; m: MobileId) -> () = ();
    /// `Region.IsPartOf<SafeZone>()`.
    in_safe_zone(&World; m: MobileId) -> bool = false;

    // ---- ethics and factions ----
    /// `Ethic.Find(m)` as an opaque id.
    ethic_of_mobile(&World; m: MobileId) -> Option<i32> = None;
    /// `Ethic.Find(item)` as an opaque id.
    ethic_of_item(&World; item: ItemId) -> Option<i32> = None;
    /// `EthicPlayer.Ethic.Definition.Adjunct.String`.
    ethic_adjunct(&World; m: MobileId) -> Option<String> = None;
    /// `Faction.Find(m)` as an opaque id.
    faction_of(&World; m: MobileId) -> Option<i32> = None;
    /// `FactionItem.Find(item)?.Faction`.
    faction_item_faction(&mut World; item: ItemId) -> Option<i32> = None;
    /// `FactionItem.GetMaxWearables(m)`.
    faction_max_wearables(&World; m: MobileId) -> i32 = 0;
    /// `Faction.Facet`.
    faction_facet(&World; ) -> Option<MapId> = None;
    /// `[` + `faction.Definition.Abbreviation` + `]` of `Faction.Find(m)`.
    faction_abbreviation(&World; m: MobileId) -> Option<String> = None;
    /// `Faction.HandleDeath(m, killer)`.
    faction_handle_death(&mut World; m: MobileId, killer: Option<MobileId>) -> () = ();
    /// `Faction.Find(m)?.RemoveMember(m)`.
    faction_remove_member(&mut World; m: MobileId) -> () = ();
    /// The faction lines of `GetProperties` (`PlayerState.Find(this)`), added when `Map == Faction.Facet`.
    faction_add_properties(&World; m: MobileId, list: &mut PropertyList) -> () = ();
    /// The faction label of `OnSingleClick`: `(text, ascii, hue)` for `from`.
    faction_single_click_label(&World; m: MobileId, from: MobileId) -> Option<(String, bool, i32)> = None;

    // ---- insurance (Engines/Insurance) ----
    /// `Insurance.Enabled`.
    insurance_enabled(&World; ) -> bool = false;
    /// `Insurance.CheckInsuranceBeforeDeath(m)`.
    insurance_before_death(&mut World; m: MobileId) -> () = ();
    /// `Insurance.CheckItemInsuranceOnDeath(m, item)`.
    insurance_item_on_death(&mut World; m: MobileId, item: ItemId) -> bool = false;
    /// `Insurance.CheckInsuranceOnDeath(m)`.
    insurance_on_death(&mut World; m: MobileId) -> () = ();
    /// `Insurance.ToggleItemInsurance(m)`.
    insurance_toggle(&mut World; m: MobileId) -> () = ();
    /// `Insurance.OpenItemInsuranceMenu(m)`.
    insurance_open_menu(&mut World; m: MobileId) -> () = ();
    /// `Insurance.CancelRenewInventoryInsurance(m)`.
    insurance_cancel_renew(&mut World; m: MobileId) -> () = ();
    /// `Insurance.AutoRenewInventoryInsurance(m)`.
    insurance_auto_renew(&mut World; m: MobileId) -> () = ();
    /// `ServerFeatureFlags.InsuranceEnabled`.
    insurance_feature_enabled(&World; ) -> bool = true;

    // ---- virtues and honor ----
    /// `VirtueSystem.CheckAtrophies(m)`.
    virtue_check_atrophies(&mut World; m: MobileId) -> () = ();
    /// `VirtueSystem.Award(m, VirtueName.Justice, points, ref gainedPath)`: `(awarded, gainedPath)`.
    virtue_award_justice(&mut World; m: MobileId, points: i32) -> (bool, bool) = (false, false);
    /// `JusticeVirtue.IsProtected(m)`.
    justice_is_protected(&World; m: MobileId) -> bool = false;
    /// `JusticeVirtue.CancelProtection(m, out prot)`.
    justice_cancel_protection(&mut World; m: MobileId) -> Option<MobileId> = None;
    /// `ReceivedHonorContext?.OnTargetDamaged(from, amount)`; `SentHonorContext?.OnSourceDamaged(from, amount)`.
    honor_on_damaged(&mut World; m: MobileId, from: Option<MobileId>, amount: i32) -> () = ();
    /// `ReceivedHonorContext?.OnTargetKilled()`; `SentHonorContext?.OnSourceKilled()`.
    honor_on_killed(&mut World; m: MobileId) -> () = ();
    /// `SentHonorContext?.OnSourceBeneficialAction(target)`.
    honor_on_source_beneficial_action(&mut World; m: MobileId, target: MobileId) -> () = ();
    /// `ReceivedHonorContext?.Cancel(); SentHonorContext?.Cancel()`.
    honor_cancel(&mut World; m: MobileId) -> () = ();
    /// `PlayerMurderSystem` tables are in [`super::tables`]; nothing to hook.
    /// `AmmoRecovery.Forget(m)`.
    ammo_recovery_forget(&mut World; m: MobileId) -> () = ();

    // ---- spells, specials and skills ----
    /// `BandageContext.GetContext(m)?.Slip()`.
    bandage_slip(&mut World; m: MobileId) -> () = ();
    /// `Confidence.StopRegenerating(m)`.
    confidence_stop_regenerating(&mut World; m: MobileId) -> () = ();
    /// `SpecialMove.ClearAllMoves(m)`.
    special_move_clear_all(&mut World; m: MobileId) -> () = ();
    /// `SpecialMove.ClearCurrentMove(m)`.
    special_move_clear_current(&mut World; m: MobileId) -> () = ();
    /// `AnimalForm.UnderTransformation(m)`.
    animal_form_under(&World; m: MobileId) -> bool = false;
    /// `AnimalForm.UnderTransformation(m, typeof(type name))`.
    animal_form_under_type(&World; m: MobileId, type_name: &str) -> bool = false;
    /// `AnimalForm.GetContext(m)?.SpeedBoost == true`.
    animal_form_speed_boost(&World; m: MobileId) -> bool = false;
    /// `AnimalForm.RemoveContext(m)`.
    animal_form_remove_context(&mut World; m: MobileId) -> () = ();
    /// `AnimalForm.GetContext(m)?.Timer.Start()`.
    animal_form_start_timer(&mut World; m: MobileId) -> () = ();
    /// `TransformationSpellHelper.UnderTransformation(m, typeof(spell type name))` (`ReaperFormSpell`, `WraithFormSpell`).
    transformation_under(&World; m: MobileId, spell: &str) -> bool = false;
    /// `InvisibilitySpell.StopTimer(m)`.
    invisibility_stop_timer(&mut World; m: MobileId) -> () = ();
    /// `new FlySpell(m).Cast()` after stopping a running `FlySpell`.
    fly_cast(&mut World; m: MobileId) -> () = ();
    /// `MovementImpl.IgnoreMovableImpassables = value`.
    ignore_movable_impassables(&mut World; value: bool) -> () = ();
    /// `BaseMount.Dismount(m)`.
    dismount(&mut World; m: MobileId) -> () = ();
    /// `DeathStrike.AddStep(m)` (Spells/Ninjitsu), after each step.
    death_strike_add_step(&mut World; m: MobileId) -> () = ();
    /// `Stealth.OnUse(m)`.
    stealth_on_use(&mut World; m: MobileId) -> () = ();
    /// `DetectHidden.TryDetectStealther(detector, target)`.
    detect_hidden_try(&mut World; detector: MobileId, target: MobileId) -> () = ();
    /// `FeatureFlagManager.IsItemEquipBlocked(item.GetType(), out reason)` when `AccessLevel < RequiredAccessLevel`.
    item_equip_blocked(&World; m: MobileId, item: ItemId) -> Option<String> = None;
    /// `FeatureFlagManager.IsSkillBlocked(skill, out reason)` when `AccessLevel < RequiredAccessLevel`.
    skill_blocked(&World; m: MobileId, skill: SkillName) -> Option<String> = None;
    /// `FeatureFlagManager.IsItemUseBlocked(item.GetType(), out reason)` when `AccessLevel < RequiredAccessLevel`.
    item_use_blocked(&World; m: MobileId, item: ItemId) -> Option<String> = None;
    /// `Stealing.ClassicMode`.
    stealing_classic_mode(&World; ) -> bool = false;
    /// `StolenItem.IsStolen(item)`.
    item_is_stolen(&World; item: ItemId) -> bool = false;
    /// `PlayerMurderSystem.ManuallySetPingPong(pm, n)` (the `PingPongs` property setter).
    manually_set_ping_pong(&mut World; m: MobileId, n: i32) -> () = ();
    /// `PlayerMurderSystem.ManuallySetShortTermMurders(pm, n)` (the `ShortTermMurders` property setter).
    manually_set_short_term_murders(&mut World; m: MobileId, n: i32) -> () = ();
    /// `item is TrappableContainer { TrapType: not TrapType.None }`.
    item_is_trapped(&World; item: ItemId) -> bool = false;
    /// `CharacterStatue.OnRequestedAnimation(viewer)` of a statue `m`.
    statue_requested_animation(&mut World; m: MobileId, viewer: MobileId) -> () = ();
    /// `TestCenter.Enabled`.
    test_center_enabled(&World; ) -> bool = false;

    // ---- quests ----
    /// `pm.Quest?.StartTimer()` / `StopTimer()`.
    quest_start_timer(&mut World; m: MobileId) -> () = ();
    quest_stop_timer(&mut World; m: MobileId) -> () = ();
    /// `Quest?.GetContextMenuEntries(ref list)`.
    quest_context_menu_entries(&World; m: MobileId, list: &mut Vec<muo_server::context_menu_entry::ContextMenuEntry>) -> () = ();
    /// `Quest?.IgnoreYoungProtection(from) == true`.
    quest_ignore_young_protection(&World; m: MobileId, from: MobileId) -> bool = false;
    /// `m.Quest is UzeraanTurmoilQuest qs && qs.IsObjectiveInProgress(typeof(FindSchmendrickObjective))` (RestlessSoul.IsEnemy).
    quest_haven_schmendrick_in_progress(&World; m: MobileId) -> bool = false;
    /// `MLQuestSystem.Enabled`.
    ml_quest_enabled(&World; ) -> bool = false;
    /// `MLQuestSystem.HandleSkillGain(m, skill)`.
    ml_quest_skill_gain(&mut World; m: MobileId, skill: SkillName) -> () = ();
    /// `MLQuestSystem.MarkQuestItem(m, item)`.
    ml_quest_mark_item(&mut World; m: MobileId, item: ItemId) -> bool = false;
    /// `BaseMLQuestGump.CloseOtherGumps(m)` and closing the quest log/offer gumps.
    ml_quest_close_gumps(&mut World; m: MobileId) -> () = ();
    /// `DisguisePersistence.StartTimer(m)` / `StopTimer(m)`.
    disguise_start_timer(&mut World; m: MobileId) -> () = ();
    disguise_stop_timer(&mut World; m: MobileId) -> () = ();
    /// `DisguisePersistence.TimeRemaining(m)` (the `DisguiseTimeLeft` property).
    disguise_time_remaining(&World; m: MobileId) -> TimeSpan = TimeSpan::ZERO;

    // ---- virtues (Engines/Virtues), what honor.rs needs beyond the hooks above ----
    /// `ReceivedHonorContext?.OnTargetPoisoned()` of an `IHonorTarget` (on poisoning).
    honor_on_target_poisoned(&mut World; m: MobileId) -> () = ();
    /// `SentHonorContext.PerfectionLuckBonus` when its target is `victim` (loot luck).
    honor_perfection_luck(&World; killer: MobileId, victim: MobileId) -> i32 = 0;

    // ---- guilds ----
    /// `Guilds.Guild.HandleDeath(m, killer)`.
    guild_handle_death(&mut World; m: MobileId, killer: Option<MobileId>) -> () = ();
    /// `g.Alliance?.IsMember(g) == true` for the player's guild.
    guild_in_alliance(&World; m: MobileId) -> bool = false;
    /// `g.Alliance.AllianceChat(m, text)`.
    guild_alliance_chat(&mut World; m: MobileId, text: &str) -> () = ();
    /// `g.GuildChat(m, text)`.
    guild_chat(&mut World; m: MobileId, text: &str) -> () = ();

    /// `Party.Get(from)?.Remove(target)` (the `RemoveFromPartyEntry` click).
    party_remove_member(&mut World; from: MobileId, target: MobileId) -> () = ();

    // ---- accounts ----
    /// `SendGump(new ServerLockdownNoticeGump(notice))`.
    lockdown_notice_gump(&mut World; m: MobileId, notice: &str) -> () = ();
    /// `SendGump(new YoungDeathNoticeGump())`.
    young_death_notice_gump(&mut World; m: MobileId) -> () = ();
    /// Closes the `ResurrectGump` of the player; `Some(true)` if one was open.
    resurrect_gump_close(&mut World; m: MobileId, alive: bool) -> bool = false;
    /// `CurseSpell.UnderEffect(m)`.
    curse_under_effect(&World; m: MobileId) -> bool = false;
    /// `ContentFeatureFlags.YoungPlayerSystem`.
    young_player_system(&World; ) -> bool = true;
    /// `ProfessionInfo.VerifyProfession(profession)`.
    verify_profession(&mut World; profession: i32) -> bool = true;
    /// `ProfessionInfo.Professions` is only reached through this.

    // ---- regions ----
    /// `Region.GetRegion<GuardedRegion>()?.IsDisabled() == false`.
    guarded_region_protects(&World; m: MobileId) -> bool = false;
    /// `Region is BaseRegion { YoungProtected: false }`: true when the region does NOT protect the young.
    region_not_young_protected(&World; m: MobileId) -> bool = false;
    /// `Region.GetRegion<DungeonRegion>()`'s `(Entrance, Map)` when the entrance is not zero.
    dungeon_entrance(&World; m: MobileId) -> Option<(Point3D, Option<MapId>)> = None;
    /// `Region.IsPartOf<JailRegion>()`.
    in_jail_region(&World; m: MobileId) -> bool = false;

    // ---- creatures and pets ----
    /// A pet's state, or None when `pet` is not a `BaseCreature`.
    pet_info(&World; pet: MobileId) -> Option<PetInfo> = None;
    /// `AutoStablePets` on one pet: `ControlTarget = null; ControlOrder = Stay; Internalize(); SetControlMaster(null);
    /// IsStabled = true; StabledBy = owner; Loyalty = MaxLoyalty`.
    pet_auto_stable(&mut World; pet: MobileId, owner: MobileId) -> () = ();
    /// The claim of `ClaimAutoStabledPets` for one pet: `SetControlMaster(owner); ControlTarget = owner;
    /// ControlOrder = Follow; MoveToWorld(owner.Location, owner.Map); IsStabled = false; StabledBy = null;
    /// Loyalty = MaxLoyalty`.
    pet_claim(&mut World; pet: MobileId, owner: MobileId) -> () = ();
    /// `pet.IsStabled = false; pet.StabledBy = null` (a deleted stabled pet).
    pet_clear_stabled(&mut World; pet: MobileId) -> () = ();
    /// `bc.IsStabled = true; bc.StabledBy = owner`.
    pet_mark_stabled(&mut World; pet: MobileId, owner: MobileId) -> () = ();
    /// `follower is BaseCreature { ControlOrder: OrderType.Guard }`.
    pet_is_guarding(&World; pet: MobileId) -> bool = false;
    /// `bc.Summoned || bc.IsBonded || bc.IsDeadPet`; None when `follower` is not a `BaseCreature` or is a
    /// `BaseEscortable`/`BaseHire`.
    pet_goes_with_owner(&World; pet: MobileId) -> Option<bool> = None;
    /// `bc.ControlOrder = OrderType.Release`.
    pet_release(&mut World; pet: MobileId) -> () = ();
    /// `target is BaseCreature { IsInvulnerable: true } || PlayerVendor || TownCrier`.
    cannot_be_harmed(&World; target: MobileId) -> bool = false;
    /// `target is BaseCreature { Controlled: false }`, or None when not a creature.
    creature_uncontrolled(&World; target: MobileId) -> Option<bool> = None;
    /// `bc.InitialInnocent && !bc.Controlled`.
    creature_initial_innocent_uncontrolled(&World; target: MobileId) -> bool = false;
    /// `bc.Controlled && bc.ControlMaster == m`.
    creature_controlled_by(&World; target: MobileId, m: MobileId) -> bool = false;
    /// `bc.GetMaster()` of a `BaseCreature`.
    creature_master(&World; m: MobileId) -> Option<MobileId> = None;
    /// `bc.IgnoreYoungProtection`.
    creature_ignores_young_protection(&World; m: MobileId) -> bool = false;
    /// `bc.PlaySound(bc.GetAngerSound()); Timer.StartTimer(bc.Delete)` for a summoned pet on another map.
    pet_dismiss_summoned(&mut World; pet: MobileId) -> () = ();

    /// `Furniture`/`DynamicFlipping` item with a `[Flippable(..)]`: its `ItemIDs`.
    furniture_flip_ids(&World; item: ItemId) -> Option<Vec<i32>> = None;
    /// `AnimalTaming.DisableMessage = value`.
    animal_taming_disable_message(&mut World; value: bool) -> () = ();

    // ---- talisman and combat ----
    /// `Talisman is BaseTalisman t && t.Protection?.Type?.IsInstanceOfType(from)`: the protection percent, or 0.
    talisman_protection(&World; m: MobileId, from: MobileId) -> i32 = 0;
    /// `Mobile.PermaFlags` of a corpse: `corpse.Criminal = true`.
    corpse_set_criminal(&mut World; corpse: muo_server::ids::ItemId) -> () = ();
}
