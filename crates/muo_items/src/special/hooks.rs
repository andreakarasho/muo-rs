//! Calls the Items/Special classes make into systems that live in other crates (gumps, factions, the veteran reward
//! system, the boat and house code), as `fn` pointers in `w.resources`. An unset hook behaves as if the system were absent
//! (noted per field). The owning crate (muo_gumps, muo_engines) sets them; see `muo_engines::special_wiring`.

use muo_server::World;
use muo_server::ids::{EntityId, ItemId, MobileId};

/// `TranslocationItem` of one class.
#[derive(Clone, Copy)]
pub struct TranslocationApi {
    pub is_item: fn(&World, ItemId) -> bool,
    /// `(Charges, Recharges, MaxCharges, MaxRecharges)`.
    pub get: fn(&World, ItemId) -> (i32, i32, i32, i32),
    /// `Charges = c; Recharges = r`.
    pub set: fn(&mut World, ItemId, i32, i32),
    /// `TranslocationItemName`: (number, string).
    pub name: fn(&World, ItemId) -> (i32, Option<String>),
}

/// What the Mutation Core items ask of `PlagueBeastLord` (owned by the mobiles crate): who is one,
/// `OpenedBy`, `Unfreeze()`. Unset: nothing is a plague beast lord (every innard is deleted when it loads).
#[derive(Clone, Copy)]
pub struct PlagueLordApi {
    pub is_lord: fn(&World, MobileId) -> bool,
    pub opened_by: fn(&World, MobileId) -> Option<MobileId>,
    pub unfreeze: fn(&mut World, MobileId),
}

#[derive(Default)]
pub struct SpecialHooks {
    /// The `PlagueBeastLord` side of the Mutation Core items.
    pub plague_lord: Option<PlagueLordApi>,
    /// `Sigil.ExistsOn(m)`. Unset: false.
    pub sigil_exists_on: Option<fn(&World, MobileId) -> bool>,
    /// `HeritageTokenGump.DisplayTo(from, token)`. Unset: nothing happens.
    pub heritage_token_gump: Option<fn(&mut World, MobileId, ItemId)>,
    /// `from.SendGump(new ConfirmBreakCrystalGump(item))`. Unset: nothing happens.
    pub break_crystal_gump: Option<fn(&mut World, MobileId, ItemId)>,
    /// `MLQuestSystem.GetContext(pm)` has an unexpired `GainSkillObjectiveInstance` that handles the skill.
    /// Unset: none.
    pub gain_skill_quest: Option<fn(&World, MobileId, muo_server::skills::SkillName) -> bool>,
    /// The classes that implement `TranslocationItem` (BagOfSending, BallOfSummoning, BraceletOfBinding), each
    /// registered by its owner. Unset or empty: the powder has no effect on anything.
    pub translocation: Vec<TranslocationApi>,
    /// `RewardOptionGump.DisplayTo(from, deed)`: the gump lists `vet::reward_options` and answers with
    /// `vet::reward_option_selected`. Unset: nothing opens.
    pub reward_option_gump: Option<fn(&mut World, MobileId, ItemId)>,
    /// `RewardDemolitionGump.DisplayTo(from, item, label)` (re-deed an `IAddon`). Unset: nothing opens.
    pub reward_demolition_gump: Option<fn(&mut World, MobileId, ItemId, i32)>,
    /// `m.SendGump(new AnkhResurrectGump(m, ResurrectMessage.VirtueShrine))` of the Ankh of Sacrifice; its response
    /// starts with `vet3::ankh_response_prelude`. Unset: nothing opens.
    pub ankh_resurrect_gump: Option<fn(&mut World, MobileId)>,
    /// `AquariumGump.DisplayTo(from, aquarium)` (a snapshot of the tank; its buttons call `aquarium::remove_item`).
    /// Unset: nothing opens.
    pub aquarium_gump: Option<fn(&mut World, MobileId, ItemId)>,
    /// `from.CloseGump<AquariumGump>()`. Unset: nothing.
    pub aquarium_close_gump: Option<fn(&mut World, MobileId)>,
    /// `HouseRaffleManagementGump.DisplayTo(from, stone)`. Unset: nothing opens.
    pub raffle_management_gump: Option<fn(&mut World, MobileId, ItemId)>,
    /// `WritOfLeaseGump.DisplayTo(from, deed)`. Unset: nothing opens.
    pub writ_of_lease_gump: Option<fn(&mut World, MobileId, ItemId)>,
    /// `from.SendGump(new ConfirmTicketPurchaseGump(.., okay => stone.Purchase_Callback(from, okay)))`; the gump
    /// answers with `raffle::purchase_callback`. Unset: nothing opens.
    pub raffle_ticket_gump: Option<fn(&mut World, MobileId, ItemId)>,
    /// `from.SendGump(new PropertiesGump(from, entity))`. Unset: nothing opens.
    pub properties_gump: Option<fn(&mut World, MobileId, EntityId)>,
}

pub fn hooks(w: &World) -> Option<&SpecialHooks> {
    w.resources.get::<SpecialHooks>()
}

pub fn hooks_mut(w: &mut World) -> &mut SpecialHooks {
    w.resources.get_mut::<SpecialHooks>()
}
