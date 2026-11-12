//! Context menus.
//!
//! An entry is a [`ContextMenuEntry`] with its `on_click` closure (one per entry kind). What an entry calls into
//! another system (parties, spellbooks, food, houses, creature teaching) goes through [`ContextMenuHooks`], set by
//! the owning crate's `register`; an unset hook does nothing, as a click on an absent system would.
//!
//! Each hook is named after the call it stands for; Party (`muo_engines`), Spellbook/SpellScroll
//! (`muo_items`), BaseHouse (`muo_multis`) and Food/BaseCreature.Teach set theirs.

pub mod context_menu_system;
pub mod entries;

pub use context_menu_system::{
    ContextMenuSystem, context_menu_request, context_menu_response, create_context_menu,
};

use muo_server::World;
use muo_server::ids::{ItemId, MobileId};
use muo_server::interfaces::PartyId;
use muo_server::skills::SkillName;

/// `Party.Get(m)`: what `AddToPartyEntry` reads of a party.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct PartyInfo {
    pub id: PartyId,
    pub leader: MobileId,
    /// `Members.Count`.
    pub members: usize,
    /// `Candidates.Count`.
    pub candidates: usize,
}

type AddScrollToBookFn = fn(&mut World, MobileId, ItemId, ItemId) -> bool;
type CreatureTeachFn = fn(&mut World, MobileId, SkillName, MobileId) -> bool;
#[derive(Default)]
pub struct ContextMenuHooks {
    /// `Party.Get(m)`.
    pub party_get: Option<fn(&World, MobileId) -> Option<PartyInfo>>,
    /// `Party.Capacity` (10).
    pub party_capacity: Option<usize>,
    /// `Party.Invite(from, target)`.
    pub party_invite: Option<fn(&mut World, MobileId, MobileId)>,
    /// `target is SpellScroll`.
    pub is_spell_scroll: Option<fn(&World, ItemId) -> bool>,
    /// The body of `AddToSpellbookEntry.InternalTarget.OnTarget` for a `Spellbook` target `book` (type check,
    /// `HasSpell`, `Content |=`, `Consume`, the 0x249 sound). Returns false when `book` isn't a spellbook.
    pub add_scroll_to_book: Option<AddScrollToBookFn>,
    /// `target is Food`.
    pub is_food: Option<fn(&World, ItemId) -> bool>,
    /// `food.Eat(from)`.
    pub food_eat: Option<fn(&mut World, ItemId, MobileId)>,
    /// `EjectPlayerEntry.OnClick` after the target is a mobile: `BaseHouse.FindHouseAt(target)`, the
    /// `!from.Alive || house.Deleted || !house.IsFriend(from)` guard and `house.Kick(from, target)`.
    pub eject_from_house: Option<fn(&mut World, MobileId, MobileId)>,
    /// `target is BaseCreature` and `bc.Teach(skill, from, 0, false)`. Returns false when not a creature.
    pub creature_teach: Option<CreatureTeachFn>,
}
