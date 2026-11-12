//! Calls the common item classes (containers, corpses, gold, keys) make into systems that live in other crates
//! (houses, feature flags, quests, parties, ...), as `fn` pointers in `w.resources`. An unset hook behaves as if
//! the system were absent or inert (noted per field).

use muo_server::World;
use muo_server::ids::{ItemId, MobileId};

type HouseDropCheckFn = fn(&mut World, ItemId, MobileId, ItemId) -> bool;
type SecureLevelEntryFn =
    fn(&World, MobileId, ItemId) -> Option<muo_server::context_menu_entry::ContextMenuEntry>;
type CreatureOnCarveFn = fn(&mut World, MobileId, MobileId, ItemId, Option<ItemId>);
type BankerDepositUpToFn = fn(&mut World, MobileId, i32, bool, i32) -> i32;
type HouseCheckHoldFn =
    fn(&mut World, Option<MobileId>, ItemId, ItemId, bool, bool, i32, i32) -> bool;
#[derive(Default)]
pub struct CommonItemHooks {
    /// `BaseHouse.CheckAccessible(m, item)`. Unset: true.
    pub house_check_accessible: Option<fn(&World, MobileId, ItemId) -> bool>,
    /// `BaseHouse.CheckHold(m, cont, item, message, checkItems, plusItems, plusWeight)`. Unset: true.
    pub house_check_hold: Option<HouseCheckHoldFn>,
    /// The drop path of `BaseContainer.TryDropItem`/`OnDragDropInto` for a container in a house:
    /// `house = BaseHouse.FindHouseAt(cont); if (house?.HasLockedDownItem(cont) == true) { reject a vendor rental
    /// contract (message 1062492); if (!house.LockDown(from, dropped, false)) return false; }`.
    /// Returns false when the drop is refused. Unset: true.
    pub house_drop_check: Option<HouseDropCheckFn>,
    /// `FeatureFlagManager.IsItemUseBlocked(type, out reason)` (when `AccessLevel < RequiredAccessLevel`).
    /// Unset: not blocked.
    pub item_use_blocked: Option<fn(&World, &'static str) -> Option<String>>,
    /// `FeatureFlagManager.IsContainerAccessBlocked(type, out reason)` (when `AccessLevel < RequiredAccessLevel`).
    /// Unset: not blocked.
    pub container_access_blocked: Option<fn(&World, &'static str) -> Option<String>>,
    /// `RootParent is PlayerVendor`. Unset: false.
    pub root_is_player_vendor: Option<fn(&World, ItemId) -> bool>,
    /// `SetSecureLevelEntry.AddTo(from, item, ref list)`: the context-menu entry of the house system, if `item` is
    /// something `from` can set the security of. Unset: no entry.
    pub secure_level_entry: Option<SecureLevelEntryFn>,
    /// `NotorietyHandlers.CorpseNotoriety(from, corpse)`: the `Notoriety` number (1 Innocent .. 7 Invulnerable).
    /// Unset: 1 (Innocent).
    pub corpse_notoriety: Option<fn(&World, MobileId, ItemId) -> i32>,
    /// `Party.Get(owner)?.Contains(from)` and `p[owner].CanLoot`: true when the owner's party lets `from` loot.
    /// Unset: false.
    pub party_can_loot: Option<fn(&World, MobileId, MobileId) -> bool>,
    /// `Party.Get(a) == Party.Get(b)` and not null. Unset: false.
    pub same_party: Option<fn(&World, MobileId, MobileId) -> bool>,
    /// `Mobile.CriminalAction(true)`. Unset: nothing.
    pub criminal_action: Option<fn(&mut World, MobileId, bool)>,
    /// `Titles.AwardKarma(from, amount, true)`. Unset: nothing.
    pub award_karma: Option<fn(&mut World, MobileId, i32, bool)>,
    /// `BaseCreature.OnCarve(from, corpse, with)` of the corpse's owner. Unset: nothing.
    pub creature_on_carve: Option<CreatureOnCarveFn>,
    /// The quests that react to opening a corpse (`UzeraanTurmoilQuest`, `TheSummoningQuest`); returns true when
    /// the open ends there (`return`). Unset: false.
    pub corpse_quest_open: Option<fn(&mut World, MobileId, ItemId) -> bool>,
    /// `new Head(dead.Name)` with `BountyTarget = dead` when `PlayerMurderSystem.BountiesEnabled` and `dead` is a
    /// player (the carve of a corpse). Unset: no head.
    pub corpse_new_head: Option<fn(&mut World, MobileId) -> Option<ItemId>>,
    /// `IDevourer.Devour(corpse)` of the killer. Unset: false.
    pub devour: Option<fn(&mut World, MobileId, ItemId) -> bool>,
    /// The quest hook of `BankCheck.OnDoubleClick` (`CashBankCheckObjective` of DarkTides/Uzeraan). Unset: nothing.
    pub bank_check_cashed: Option<fn(&mut World, MobileId)>,
    /// `ILockable` outside LockableContainer (doors, boat planks, ..): one entry per owner, see [`add_lockable`].
    pub lockables: Vec<Lockable>,
    /// `Banker.DepositUpTo(from, amount, false, reserveSlots)`: returns the amount deposited. Unset: 0.
    pub banker_deposit_up_to: Option<BankerDepositUpToFn>,
    /// `targeted is IChoppable` then `OnChop(from)`: each class that implements it pushes a function that answers
    /// whether the item is one (running its `OnChop`). The harvest system tries them in order.
    pub choppables: Vec<fn(&mut World, ItemId, MobileId) -> bool>,
    /// `new FireFieldItem(itemID, loc, caster, map, duration, val)` (its constructor is
    /// not `[Constructible]`, `Firebomb` makes them): the new item. Unset: no field.
    pub fire_field_item: Option<FireFieldMaker>,
}

/// The signature of [`CommonItemHooks::fire_field_item`].
pub type FireFieldMaker = fn(
    &mut World,
    i32,
    muo_primitives::Point3D,
    Option<MobileId>,
    Option<muo_server::map::MapId>,
    muo_primitives::TimeSpan,
    i32,
) -> Option<ItemId>;

pub fn hooks(w: &World) -> Option<&CommonItemHooks> {
    w.resources.get::<CommonItemHooks>()
}

/// One `ILockable` implementer: its `KeyValue` (None when the item is not one of its) and the toggle
/// (`if (o is BaseDoor door && !door.UseLocks()) refuse; o.Locked = !o.Locked`): the new `Locked`, or None when the
/// item is not its or the toggle is refused.
#[derive(Copy, Clone)]
pub struct Lockable {
    pub key_value: fn(&World, ItemId) -> Option<u32>,
    pub toggle: fn(&mut World, ItemId) -> Option<bool>,
}

/// Registers an `ILockable` owner; every owner is asked in turn (a later owner never replaces an earlier one).
pub fn add_lockable(w: &mut World, l: Lockable) {
    hooks_mut(w).lockables.push(l);
}

/// `(o as ILockable)?.KeyValue` over the registered owners.
pub fn lockable_key_value(w: &World, item: ItemId) -> Option<u32> {
    hooks(w)?
        .lockables
        .iter()
        .find_map(|l| (l.key_value)(w, item))
}

/// Toggles the first owner that accepts `item`; the new `Locked`.
pub fn lockable_toggle(w: &mut World, item: ItemId) -> Option<bool> {
    let owners = hooks(w)?.lockables.clone();
    owners.iter().find_map(|l| (l.toggle)(w, item))
}

pub fn hooks_mut(w: &mut World) -> &mut CommonItemHooks {
    w.resources.get_mut::<CommonItemHooks>()
}

/// Registers the classes of the common item modules (containers, corpses, gold, keys, food, resources).
pub fn register(types: &mut muo_server::type_registry::TypeRegistry) {
    super::cont::register(types);
    super::gold::register(types);
    super::corpse::register(types);
    super::food::register(types);
    super::resource::register(types);
    super::flip::register(types);
    super::addon::register(types);
    super::addon_container::register(types);
    super::ammo_recovery::register(types);
    // `BaseClothing : IDyable, IScissorable` and `BaseArmor : IScissorable`: the dye tub and the scissors look the
    // item up in the interface registry, so the families list themselves like the single classes do.
    types.register_phase_default(
        muo_server::type_registry::Phase::Configure,
        "BaseClothing.IDyable",
        |w| {
            use super::interfaces::{register_dyable, register_scissorable};
            register_dyable(w, &super::clothing::BASE_CLOTHING, super::clothing::dye);
            register_scissorable(w, &super::clothing::BASE_CLOTHING, |w, id, from, _| {
                super::clothing::scissor(w, id, from)
            });
            register_scissorable(w, &super::armor::BASE_ARMOR, |w, id, from, _| {
                super::armor::scissor(w, id, from)
            });
        },
    );
    // `shield.OnHit(weapon, damage)` and `armor.OnHit(weapon, damage)` of an `IWearableDurability` item, which the
    // weapon's `AbsorbDamage` calls through these hooks (BaseArmor, BaseShield and BaseClothing own the code).
    types.register_phase_default(
        muo_server::type_registry::Phase::Configure,
        "BaseWeapon.AbsorbDamageHooks",
        |w| {
            use super::{armor, clothing, weapon_hooks::WeaponHooks};
            let h = w.resources.get_mut::<WeaponHooks>();
            h.shield_on_hit = Some(armor::on_hit);
            // `PlayerMobile.SetMountBlock(type, delay, dismount)`; `kind` is 0 = Dazed, 1 = DismountRecovery
            h.set_mount_block = Some(|w, m, kind, delay, message| {
                use crate::player_mobile::BlockMountType;
                let ty = if kind == 1 {
                    BlockMountType::DismountRecovery
                } else {
                    BlockMountType::Dazed
                };
                crate::player_mobile::members::set_mount_block(w, m, ty, Some(delay), message);
            });
            h.base_mount_dismount = Some(crate::creature::mount::dismount);
            // `state.SupportsExpansion(Expansion.SE)` of the mobile's client (`WeaponAbility.Validate`)
            h.net_supports_se = Some(|w, m| {
                let Some(ns) = w.mobile(m).and_then(|m| m.net_state) else {
                    return false;
                };
                let (Some(n), Some(table)) = (w.net_state(ns), w.expansion_table()) else {
                    return false;
                };
                n.caps.supports_expansion(
                    table,
                    table.get_info(muo_data::expansion::Expansion::SE as i32),
                    w.expansion() as i32,
                    true,
                )
            });
            h.wearable_on_hit = Some(|w, item, weapon, damage| {
                if armor::is_armor(w, item) {
                    Some(armor::on_hit(w, item, weapon, damage))
                } else if w
                    .item(item)
                    .is_some_and(|i| i.ty.is(&clothing::BASE_CLOTHING))
                {
                    Some(clothing::on_hit(w, item, weapon, damage))
                } else {
                    None
                }
            });
        },
    );
}

#[cfg(test)]
#[path = "common_tests.rs"]
mod common_tests;
