//! Calls the weapon code (BaseWeapon, the weapon abilities) makes into systems that live in other crates: spells
//! (Divine Fury, Animal Form, Evasion..), Bushido/Ninjitsu special moves, virtues, ethics, talismans, duels.
//! Each is a `fn` pointer in [`WeaponHooks`] (in `w.resources`) set by the owning module's `register`; the wrapper
//! with the same name does the call. An unset hook answers as if that system were absent or inert
//! (the default after `=`).

use muo_primitives::TimeSpan;
use muo_server::World;
use muo_server::ids::{ItemId, MobileId};

use super::weapon::Ability;

macro_rules! hooks {
    ($( $(#[$m:meta])* $name:ident ( $wty:ty ; $($arg:ident : $aty:ty),* ) -> $ret:ty = $default:expr; )*) => {
        /// One optional `fn` pointer per call into a system that lives outside this crate.
        #[derive(Default)]
        pub struct WeaponHooks {
            $( $(#[$m])* pub $name: Option<fn($wty $(, $aty)*) -> $ret>, )*
        }
        $(
            $(#[$m])*
            #[allow(unused_variables)]
            pub fn $name(w: $wty $(, $arg: $aty)*) -> $ret {
                let f = w.resources.get::<WeaponHooks>().and_then(|h| h.$name);
                match f {
                    Some(f) => f(w $(, $arg)*),
                    None => $default,
                }
            }
        )*
    };
}

hooks! {
    /// `DivineFurySpell.UnderEffect(m)`.
    divine_fury_under_effect(&World; m: MobileId) -> bool = false;
    /// `DivineFurySpell.GetAttackBonus(m)` (the ninja weapons' own `CombatCheck`).
    divine_fury_attack_bonus(&World; m: MobileId) -> i32 = 0;
    /// `DivineFurySpell.GetDefendMalus(m)`.
    divine_fury_defend_malus(&World; m: MobileId) -> i32 = 0;
    /// `DivineFurySpell.GetWeaponSpeed(m)`.
    divine_fury_weapon_speed(&World; m: MobileId) -> i32 = 0;
    /// `DivineFurySpell.GetDamageBonus(m)`.
    divine_fury_damage_bonus(&World; m: MobileId) -> i32 = 0;
    /// `AnimalForm.UnderTransformation(m)` (`None`) or `UnderTransformation(m, typeof(X))` (the form's class name).
    animal_form_under(&World; m: MobileId, form: Option<&'static str>) -> bool = false;
    /// `HonorableExecution.GetSwingBonus(m)`.
    honorable_execution_swing_bonus(&World; m: MobileId) -> i32 = 0;
    /// `HonorableExecution.RemovePenalty(m)`.
    honorable_execution_remove_penalty(&mut World; m: MobileId) -> () = ();
    /// `HonorableExecution.IsUnderPenalty(m)`.
    honorable_execution_is_under_penalty(&World; m: MobileId) -> bool = false;
    /// `ReaperFormSpell.SwingSpeedBonus` when `TransformationSpellHelper.GetContext(m).Spell` is a ReaperFormSpell.
    reaper_form_swing_bonus(&World; m: MobileId) -> i32 = 0;
    /// `ReaperFormSpell.SpellDamageBonus` under the same condition.
    reaper_form_spell_damage_bonus(&World; m: MobileId) -> i32 = 0;
    /// `TransformationSpellHelper.UnderTransformation(m, typeof(HorrificBeastSpell))`.
    under_horrific_beast(&World; m: MobileId) -> bool = false;
    /// `context?.Spell is NecromancerSpell && context.Type != typeof(HorrificBeastSpell)` for `m`.
    in_necromancer_form_not_horrific(&World; m: MobileId) -> bool = false;
    /// `EssenceOfWindSpell.IsDebuffed(m) ? GetSSIMalus(m) : 0`.
    essence_of_wind_ssi_malus(&World; m: MobileId) -> i32 = 0;
    /// `Discordance.GetEffect(m, ref effect)`.
    discordance_effect(&World; m: MobileId) -> Option<i32> = None;
    /// `Evasion.IsEvading(m)`.
    evasion_is_evading(&World; m: MobileId) -> bool = false;
    /// `Evasion.GetParryScalar(m)`.
    evasion_parry_scalar(&World; m: MobileId) -> f64 = 1.0;
    /// `CounterAttack.IsCountering(m)`.
    counter_attack_is_countering(&World; m: MobileId) -> bool = false;
    /// `CounterAttack.StopCountering(m)`.
    counter_attack_stop(&mut World; m: MobileId) -> () = ();
    /// `Confidence.IsConfident(m)`.
    confidence_is_confident(&World; m: MobileId) -> bool = false;
    /// `SurpriseAttack.GetMalus(m, ref malus)`.
    surprise_attack_malus(&World; m: MobileId) -> Option<i32> = None;
    /// `MirrorImage.HasClone(m)`.
    mirror_image_has_clone(&World; m: MobileId) -> bool = false;
    /// The first `Clone` of `m` within 4 tiles whose `SummonMaster == m`.
    mirror_image_clone_in_range(&World; m: MobileId) -> Option<MobileId> = None;
    /// `ImmolatingWeaponSpell.IsImmolating(weapon)`.
    immolating_is(&World; weapon: ItemId) -> bool = false;
    /// `ImmolatingWeaponSpell.DoEffect(weapon, defender)`.
    immolating_do_effect(&mut World; weapon: ItemId, defender: MobileId) -> () = ();
    /// `ImmolatingWeaponSpell.StopImmolating(weapon)`.
    immolating_stop(&mut World; weapon: ItemId) -> () = ();
    /// `PlayerMobile.DuelContext?.CheckItemEquip(m, item)` (Engines/ConPVP): false forbids the swing.
    duel_check_item_equip(&mut World; m: MobileId, item: ItemId) -> bool = true;
    /// `DuelContext.AllowSpecialAbility(m, option, message)`.
    duel_allow_special_ability(&mut World; m: MobileId, option: &'static str, message: bool) -> bool = true;
    /// `VirtueSystem.GetVirtues(pm)?.HonorActive == true` (Engines/Virtues).
    virtue_honor_active(&World; m: MobileId) -> bool = false;
    /// `pm.SentHonorContext.PerfectionDamageBonus` when `SentHonorContext.Target == defender`, else 0.
    honor_perfection_bonus(&World; attacker: MobileId, defender: MobileId) -> i32 = 0;
    /// `(defender as IHonorTarget)?.ReceivedHonorContext?.OnTargetHit(attacker)`.
    honor_on_target_hit(&mut World; defender: MobileId, attacker: MobileId) -> () = ();
    /// `(defender as IHonorTarget)?.ReceivedHonorContext?.OnTargetMissed(attacker)`.
    honor_on_target_missed(&mut World; defender: MobileId, attacker: MobileId) -> () = ();
    /// `attacker.Talisman is BaseTalisman { Killer: not null } t` -> `t.Killer.DamageBonus(defender)`.
    talisman_killer_bonus(&World; attacker: MobileId, defender: MobileId) -> i32 = 0;
    /// `attacker.Talisman is BaseTalisman t && TalismanSlayer.Slays(t.Slayer, defender)`.
    talisman_slays(&World; attacker: MobileId, defender: MobileId) -> bool = false;
    /// `TalismanSlayer.Slays(TalismanSlayerName.Bovine, defender)` (ButchersWarCleaver).
    talisman_slayer_bovine_slays(&World; defender: MobileId) -> bool = false;
    /// `quiver.AlterBowDamage(ref phys, ref fire, ref cold, ref pois, ref nrgy, ref chaos, ref direct)` for the
    /// quiver on `Layer.Cloak` (Items/Equipment/Quivers).
    quiver_alter_bow_damage(&World; attacker: MobileId, split: [i32; 7]) -> [i32; 7] = split;
    /// `Spellbook.FindEquippedSpellbook(m)`'s `(Slayer, Slayer2)` (`None` without a spellbook).
    equipped_spellbook_slayers(&World; m: MobileId) -> Option<(i32, i32)> = None;
    /// `shield.OnHit(weapon, damage)` of a `BaseShield`: the damage left.
    shield_on_hit(&mut World; shield: ItemId, weapon: ItemId, damage: i32) -> i32 = damage;
    /// `armor.OnHit(weapon, damage)` of an `IWearableDurability` item; `None` if the item is not one.
    wearable_on_hit(&mut World; item: ItemId, weapon: ItemId, damage: i32) -> Option<i32> = None;
    /// `AttuneWeaponSpell.TryAbsorb(defender, ref damage)` (Spells/Necromancy... Spells/Chivalry/AttuneWeapon): the damage left.
    attune_weapon_try_absorb(&mut World; defender: MobileId, damage: i32) -> i32 = damage;
    /// `ReactiveArmorSpell.HandleMeleeHit(attacker, defender, ref damage)` (Spells/First): the damage left.
    reactive_armor_handle_melee_hit(&mut World; attacker: MobileId, defender: MobileId, damage: i32) -> i32 = damage;
    /// `BaseCreature.GetWeaponAbility()`.
    creature_weapon_ability(&mut World; bc: MobileId) -> Option<Ability> = None;
    /// `SpellHelper.Damage(delay, target, from, damage, phys, fire, cold, pois, nrgy)`.
    spell_helper_damage(&mut World; delay: TimeSpan, target: MobileId, from: MobileId, damage: f64, split: [i32; 5]) -> () = ();
    /// `SpellHelper.ValidIndirectTarget(from, to)`.
    spell_helper_valid_indirect_target(&mut World; from: MobileId, to: MobileId) -> bool = true;
    /// `new DispelSpell(caster).CheckResisted(target)`.
    dispel_check_resisted(&mut World; caster: MobileId, target: MobileId) -> bool = false;
    /// `attacker.Spell is Spell { IsCasting: true, BlocksMovement: true }`.
    spell_blocks_movement(&World; m: MobileId) -> bool = false;
    /// `MindRotSpell.GetMindRotScalar(m, ref scalar)`.
    mind_rot_scalar(&World; m: MobileId) -> Option<f64> = None;
    /// `SpecialMove.GetCurrentMove(m) != null` (Bushido and Ninjitsu).
    special_move_active(&mut World; m: MobileId) -> bool = false;
    /// `SpecialMove.ClearCurrentMove(m)`.
    special_move_clear(&mut World; m: MobileId) -> () = ();
    /// `move.OnBeforeSwing(attacker, defender)`.
    special_move_on_before_swing(&mut World; attacker: MobileId, defender: MobileId) -> bool = true;
    /// `move.GetAccuracyBonus(attacker)`.
    special_move_accuracy_bonus(&mut World; attacker: MobileId) -> i32 = 0;
    /// `move.GetDamageScalar(attacker, defender)`.
    special_move_damage_scalar(&mut World; attacker: MobileId, defender: MobileId) -> f64 = 1.0;
    /// `move.OnBeforeDamage(attacker, defender)`.
    special_move_on_before_damage(&mut World; attacker: MobileId, defender: MobileId) -> bool = true;
    /// `move.IgnoreArmor(attacker)`.
    special_move_ignore_armor(&mut World; attacker: MobileId) -> bool = false;
    /// `move.GetPropertyBonus(attacker)`.
    special_move_property_bonus(&mut World; attacker: MobileId) -> f64 = 1.0;
    /// `move.OnHit(attacker, defender, damage)`.
    special_move_on_hit(&mut World; attacker: MobileId, defender: MobileId, damage: i32) -> () = ();
    /// `move.OnMiss(attacker, defender)`.
    special_move_on_miss(&mut World; attacker: MobileId, defender: MobileId) -> () = ();
    /// The `Fists` singleton, `Mobile.DefaultWeapon`: the weapon item of a mobile with
    /// nothing wielded.
    fists_item(&World; ) -> Option<ItemId> = None;
    /// `PlayerMobile.SetMountBlock(type, delay, message)` (`kind` is `BlockMountType`).
    set_mount_block(&mut World; m: MobileId, kind: i32, delay: TimeSpan, message: bool) -> () = ();
    /// `pm.AddBuff(new BuffInfo(icon, cliloc, duration))` (Engines/BuffIcons).
    add_buff(&mut World; m: MobileId, icon: i32, cliloc: i32, duration: TimeSpan) -> () = ();
    /// `pm.RemoveBuff(icon)`.
    remove_buff(&mut World; m: MobileId, icon: i32) -> () = ();
    /// `BaseRunicTool.Resource` of a runic tool.
    runic_tool_resource(&World; tool: ItemId) -> crate::craft::CraftResource = crate::craft::CraftResource::None;
    /// `HarvestSystem is Mining` for an axe, pole arm or pickaxe (`HarvestSystem => Lumberjacking.System` otherwise).
    harvest_is_mining(&World; tool: ItemId) -> bool = false;
    /// `HarvestSystem.BeginHarvesting(from, tool)` (Engines/Harvest).
    harvest_begin(&mut World; from: MobileId, tool: ItemId) -> () = ();
    /// `BaseHarvestTool.AddContextMenuEntries(from, tool, ref list, HarvestSystem)`.
    harvest_context_entries(&mut World; from: MobileId, tool: ItemId, list: &mut Vec<muo_server::context_menu_entry::ContextMenuEntry>) -> () = ();
    /// `attacker.FindItemOnLayer<BaseQuiver>(Layer.Cloak)`.
    quiver_of(&World; m: MobileId) -> Option<ItemId> = None;
    /// `quiver.LowerAmmoCost`.
    quiver_lower_ammo_cost(&World; quiver: ItemId) -> i32 = 0;
    /// `quiver.InvalidateWeight()`.
    quiver_invalidate_weight(&mut World; quiver: ItemId) -> () = ();
    /// `AmmoRecovery.Bank(pm, ammoType)`.
    ammo_recovery_bank(&mut World; pm: MobileId, ammo_type: &'static str) -> () = ();
    /// `state.SupportsExpansion(Expansion.SE)` of the mobile's client (`WeaponAbility.Validate`).
    net_supports_se(&World; m: MobileId) -> bool = true;
    /// `TransformationSpellHelper.GetContext(m)?.Type` is `LichFormSpell` or `WraithFormSpell` (bleed immunity).
    bleed_immune_form(&World; m: MobileId) -> bool = false;
    /// `defender.Mount.Rider = null` / `BaseMount.Dismount(m)`.
    base_mount_dismount(&mut World; m: MobileId) -> () = ();
}

/// The mobile's wielded weapon item (`Mobile.Weapon as BaseWeapon`): its `Item`, or the `Fists` singleton.
pub fn weapon_item(w: &mut World, m: MobileId) -> Option<ItemId> {
    match muo_server::mobile::base::weapon(w, m)? {
        muo_server::interfaces::WeaponRef::Item(i) => Some(i),
        muo_server::interfaces::WeaponRef::Default => fists_item(w),
    }
}
