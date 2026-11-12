//! The statics of the craft engine (`DefBlacksmithy.CraftSystem` and the others, every system's context table,
//! the `Recipe.Recipes` dictionary, `CraftItem._itemIds`), kept in `w.resources`.

use std::collections::HashMap;
use std::rc::Rc;

use indexmap::IndexMap;
use muo_server::World;
use muo_server::ids::{ItemId, MobileId};
use muo_server::item::ItemType;

use super::model::{BeverageType, CraftContext, CraftSystem, SkillName};

/// How the engine reads `IHasQuantity` items (beverages, kegs): set by the owner of those classes.
#[derive(Copy, Clone)]
pub struct QuantityApi {
    /// `type is IHasQuantity`.
    pub type_is_quantity: fn(&'static ItemType) -> bool,
    /// `(Quantity, Content when it is a BaseBeverage)`; `None` if the item is not an `IHasQuantity`.
    pub quantity: QuantityFn,
    /// `Quantity = value`.
    pub set_quantity: fn(&mut World, ItemId, i32),
}

/// `Recipe`'s place: `(system name, item index)`.
pub type RecipeRef = (&'static str, usize);

type QuantityFn = fn(&World, ItemId) -> Option<(i32, Option<BeverageType>)>;
type TalismanFn = fn(&World, MobileId) -> Option<(SkillName, i32, i32)>;
#[derive(Default)]
pub struct CraftState {
    pub systems: IndexMap<&'static str, Rc<CraftSystem>>,
    pub contexts: HashMap<(&'static str, MobileId), CraftContext>,
    pub item_ids: HashMap<&'static str, i32>,
    /// `Recipe.Recipes`.
    pub recipes: IndexMap<i32, RecipeRef>,
    /// `T2ACraftSystem.Enabled`: the packet-based menus.
    // `GetSetting("t2aCraftMenus", !Core.UOTD)`: true by default before UOTD. Only the craft items that read it follow it; the
    // packet menus are `t2a`.
    pub t2a: bool,
    pub quantity: Option<QuantityApi>,
    /// `Spellbook.Find(from, scroll.SpellID)?.HasSpell(id)` for a scroll class (full name). Unset: nobody knows any.
    pub spell_known: Option<fn(&mut World, MobileId, &str) -> bool>,
    /// `scroll.SpellID` of a scroll class (full name), by making and deleting a scroll. Unset: unknown.
    pub spell_id_of: Option<fn(&mut World, &str) -> Option<i32>>,
    /// `Spellbook.Find(from, id)?.HasSpell(id) == true`.
    pub has_spell: Option<fn(&mut World, MobileId, i32) -> bool>,
    /// The T2A inscription menu's `_spellIds`: one throwaway scroll per class, not one per menu open.
    pub spell_ids: HashMap<String, i32>,
    /// `Faction.Find(from) != null`. Unset: factions are inert, nobody is in one.
    pub in_faction: Option<fn(&World, MobileId) -> bool>,
    /// `from.Talisman is BaseTalisman t`: `(t.Skill, t.SuccessBonus, t.ExceptionalBonus)`. Unset: no talisman.
    pub talisman: Option<TalismanFn>,
    /// `DragonBardingDeed.Resource`. Unset: none.
    pub deed_resource: Option<fn(&World, ItemId) -> muo_content::craft::CraftResource>,
    pub built: bool,
}

pub fn state(w: &World) -> Option<&CraftState> {
    w.resources.get::<CraftState>()
}

pub fn state_mut(w: &mut World) -> &mut CraftState {
    w.resources.get_mut::<CraftState>()
}

/// `T2ACraftSystem.Enabled`.
pub fn t2a_enabled(w: &World) -> bool {
    state(w).is_some_and(|s| s.t2a)
}

/// `Faction.Find(from) != null`.
pub fn faction_member(w: &World, m: MobileId) -> bool {
    state(w).and_then(|s| s.in_faction).is_some_and(|f| f(w, m))
}

/// `from.Talisman` as `(Skill, SuccessBonus, ExceptionalBonus)`.
pub fn talisman(w: &World, m: MobileId) -> Option<(SkillName, i32, i32)> {
    state(w).and_then(|s| s.talisman).and_then(|f| f(w, m))
}

/// `RecallRune.Marked` (the runebook craft needs a blank rune). Set by the owner of RecallRune.
#[derive(Copy, Clone)]
pub struct RuneApi {
    pub marked: fn(&World, ItemId) -> bool,
}
