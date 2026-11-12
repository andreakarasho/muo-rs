//! Calls the equipment bases (`BaseArmor`, `BaseClothing`, `BaseJewel`, shields) make into systems that live
//! in other modules (ethics, factions, crafting, spells, localization), as `fn` pointers in `w.resources` set by the
//! owning module's `register`. Each field says what the call is and what an unset hook does (the answer of a
//! server where that system is inert).

use muo_primitives::dotnet::InvariantCase;
use muo_server::World;
use muo_server::ids::{ItemId, MobileId};
use muo_server::item::ItemType;

use super::jewel::GemType;
use crate::craft::CraftResource;

/// What the crafting system hands `ICraftable.OnCraft(quality, makersMark, from, craftSystem, typeRes, tool,
/// craftItem, resHue)`, already resolved to plain values.
#[derive(Clone, Debug)]
pub struct CraftArgs {
    pub quality: i32,
    pub makers_mark: bool,
    pub from: MobileId,
    /// `CraftResources.GetFromType(typeRes ?? craftItem.Resources[0].ItemType)`.
    pub resource: CraftResource,
    pub res_hue: i32,
    /// `craftSystem.GetContext(from)?.DoNotColor == true`.
    pub do_not_color: bool,
    /// The `tool` when it is a `BaseRunicTool`.
    pub runic_tool: Option<ItemId>,
    /// `craftItem.Resources.Count > 1`, and the `GemType` of `craftItem.Resources[1].ItemType` (jewelry).
    pub second_resource: Option<GemType>,
    /// `context.PendingGemType`/`PendingGemCount` when set (T2A jewelry). The craft system clears them afterwards.
    pub pending_gem: Option<(GemType, i32)>,
    /// The name of the `craftSystem` (`Alchemy`, `Blacksmithy`..): `craftSystem is DefAlchemy` is `== "Alchemy"`.
    pub craft_system: &'static str,
}

impl CraftArgs {
    /// Arguments for a craft without a runic tool, gems or a special resource hue.
    pub fn new(
        quality: i32,
        makers_mark: bool,
        from: MobileId,
        resource: CraftResource,
    ) -> CraftArgs {
        CraftArgs {
            quality,
            makers_mark,
            from,
            resource,
            res_hue: 0,
            do_not_color: false,
            runic_tool: None,
            second_resource: None,
            pending_gem: None,
            craft_system: "",
        }
    }
}

/// `Ethic.CheckTrade(from, to, newOwner, item)`.
pub type CheckTradeFn = fn(&mut World, MobileId, MobileId, MobileId, ItemId) -> bool;
/// The tailoring half of `Scissor`: `(world, class, resource, recipe_fallback)`, the new resource item and the
/// recipe's first resource amount.
pub type TailoringScissorFn =
    fn(&mut World, &'static ItemType, CraftResource, bool) -> Option<(ItemId, i32)>;
/// `Localization.GetText(number)`.
pub type LocalizedTextFn = fn(&World, i32) -> Option<String>;

type DyeTubFn = fn(&World, ItemId) -> (i32, i32);
type ConsumeGemsFn = fn(&mut World, MobileId, GemType, i32) -> bool;

#[derive(Default)]
pub struct EquipHooks {
    /// `Ethic.CheckEquip(from, item)`. Unset: true.
    pub ethic_check_equip: Option<fn(&mut World, MobileId, ItemId) -> bool>,
    /// `Ethic.CheckTrade(from, to, newOwner, item)`. Unset: true.
    pub ethic_check_trade: Option<CheckTradeFn>,
    /// `(DyeTub.DyedHue, DyeTub.FailMessage)` of a tub (set by the owner of DyeTub). Unset: the tub's `Hue` and 1042083.
    pub dye_tub: Option<DyeTubFn>,
    /// `Ethic.IsImbued(item)`. Unset: false.
    pub ethic_is_imbued: Option<fn(&World, ItemId) -> bool>,
    /// `m_FactionState != null` (the item is a faction item). Unset: false.
    pub faction_item: Option<fn(&World, ItemId) -> bool>,
    /// `CraftItem.RetainsColor(GetType())`. Unset: true (every metal and leather armor).
    pub retains_color: Option<fn(&'static ItemType) -> bool>,
    /// The tailoring half of `Scissor`: `DefTailoring.CraftSystem.CraftItems.SearchFor(type)` with exactly one
    /// resource of at least 2, and the new resource item (`CraftResources.GetInfo(resource).ResourceTypes[0]`, or
    /// for clothing, when the flag is set, the recipe's resource type as a fallback). Returns the item made and the
    /// recipe's first resource amount. Unset: None ("Scissors can not be used on that").
    pub tailoring_scissor: Option<TailoringScissorFn>,
    /// `BaseRunicTool.ApplyAttributesTo(item)`: `(tool, item)`. Unset: nothing.
    pub runic_apply_attributes: Option<fn(&mut World, ItemId, ItemId)>,
    /// `AosSkillBonuses.CheckCancelMorph(m)` (transformation spells, animal form, polymorph, incognito). Unset:
    /// nothing.
    pub check_cancel_morph: Option<fn(&mut World, MobileId)>,
    /// `from.Backpack.ConsumeTotal(gemItemType, count)`. Unset: false (the gems are "no longer available").
    pub consume_gems: Option<ConsumeGemsFn>,
    /// `Localization.GetText(number)`. Unset: "".
    pub localized_text: Option<LocalizedTextFn>,
}

pub fn hooks(w: &World) -> Option<&EquipHooks> {
    w.resources.get::<EquipHooks>()
}

pub fn hooks_mut(w: &mut World) -> &mut EquipHooks {
    w.resources.get_mut::<EquipHooks>()
}

/// `Localization.GetText(number).ToLowerInvariant()`.
pub fn lower_text(w: &World, number: i32) -> String {
    hooks(w)
        .and_then(|h| h.localized_text)
        .and_then(|f| f(w, number))
        .unwrap_or_default()
        .lower_inv()
}

/// The `FactionItemState` setter of `BaseArmor`, `BaseClothing` and `BaseWeapon`. The state itself belongs to the
/// faction engine (`FactionItem`); this is what the setter does to the item: without a state the hue goes back to
/// the resource's (clothing: 0), and the loot type is `Blessed` with a state, `Regular` without.
pub fn set_faction_item_state(w: &mut World, id: ItemId, present: bool) {
    let Some(ty) = w.item(id).map(|i| i.ty) else {
        return;
    };
    if !present {
        let resource = if ty.is(&super::armor::BASE_ARMOR) {
            super::armor::data(w, id).map(|d| d.resource)
        } else if ty.is(&super::weapon::BASE_WEAPON) {
            super::weapon::data(w, id).map(|d| d.resource)
        } else {
            None
        };
        let hue = resource.map_or(0, |r| crate::craft::hue(w, r));
        (ty.set_hue)(w, id, hue);
    }
    muo_server::item::ops::set_loot_type(
        w,
        id,
        if present {
            muo_server::item::LootType::Blessed
        } else {
            muo_server::item::LootType::Regular
        },
    );
}
