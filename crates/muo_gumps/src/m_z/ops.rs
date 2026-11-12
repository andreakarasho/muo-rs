//! What the house, runebook and player vendor gumps ask of the systems that own those objects.
//!
//! Houses (`muo_multis`), runebooks, player vendors and rental contracts live in other crates that sit beside this
//! one. Each gump here reads a snapshot (`*View`) when it is made, which is right as it is sent, and calls the owner's trait object for the effects. A trait object is
//! installed in [`MzHooks`](super::hooks::MzHooks) by the owning crate's `register`; with none installed the gumps
//! do nothing.

use std::rc::Rc;

use muo_io::text::TextDefinition;
use muo_primitives::{DateTime, Point3D};
use muo_server::World;
use muo_server::ids::{ItemId, MobileId};
use muo_server::map::MapId;

// ---- houses ----

/// One `VendorInventory` of a house.
#[derive(Clone, Debug)]
pub struct VendorInventoryView {
    /// The inventory's id inside its house.
    pub id: u64,
    pub owner: Option<MobileId>,
    pub vendor_name: String,
    pub shop_name: String,
    pub expire_time: DateTime,
    pub items: Vec<ItemId>,
    pub gold: i32,
}

/// What `ViewHousesGump` shows of a `BaseHouse`.
#[derive(Clone, Debug)]
pub struct HouseView {
    pub house: ItemId,
    /// `FindHouseName(house)`: the placement entry's description, else the class name.
    pub name: TextDefinition,
    /// `house.Sign == null ? "An Unnamed House" : house.Sign.GetName()`.
    pub sign_name: String,
    pub owner_name: String,
    pub map: Option<MapId>,
    /// `map.Name`.
    pub map_name: Option<String>,
    pub location: Point3D,
    /// `Sextant.Format(..)`: `{lat}° {mins}'{N/S}, {long}° {mins}'{E/W}`.
    pub sextant: Option<String>,
    pub friends: usize,
    pub co_owners: usize,
    pub bans: usize,
    pub can_decay: bool,
    /// `DecayLevel.ToString()`.
    pub decay_level: String,
}

/// `BaseHouse` as the vendor and house-list gumps use it.
pub trait HouseOps {
    fn deleted(&self, w: &World, house: ItemId) -> bool;
    /// `house.IsActive`, `IsInside(m)`, `IsOwner(m)`.
    fn is_active(&self, w: &World, house: ItemId) -> bool;
    fn is_inside(&self, w: &World, house: ItemId, m: MobileId) -> bool;
    fn is_owner(&self, w: &World, house: ItemId, m: MobileId) -> bool;

    /// `house.InternalizedVendors`.
    fn internalized_vendors(&self, w: &World, house: ItemId) -> Vec<MobileId>;
    fn remove_internalized_vendor(&self, w: &mut World, house: ItemId, vendor: MobileId);
    /// `BaseHouse.IsThereVendor(location, map, out vendor, out contract)`.
    fn is_there_vendor(&self, w: &World, location: Point3D, map: Option<MapId>) -> (bool, bool);

    /// `house.Sign`.
    fn sign(&self, w: &World, house: ItemId) -> Option<ItemId>;
    /// `house.VendorInventories`.
    fn vendor_inventories(&self, w: &World, house: ItemId) -> Vec<VendorInventoryView>;
    /// One inventory read again (`None` once it is not in the house's list).
    fn vendor_inventory(&self, w: &World, house: ItemId, id: u64) -> Option<VendorInventoryView>;
    fn inventory_remove_item(&self, w: &mut World, house: ItemId, id: u64, item: ItemId);
    fn inventory_set_gold(&self, w: &mut World, house: ItemId, id: u64, gold: i32);
    /// `VendorInventory.Delete()`.
    fn inventory_delete(&self, w: &mut World, house: ItemId, id: u64);

    /// `BaseHouse.GetHouses(mob)` for each character of the owner's account (or the owner alone), sorted by `BuiltOn`.
    fn houses_of(&self, w: &World, owner: MobileId) -> Vec<ItemId>;
    /// `None` where the house is gone.
    fn view(&self, w: &World, house: ItemId) -> Option<HouseView>;
    /// `house.BanLocation`.
    fn ban_location(&self, w: &World, house: ItemId) -> Point3D;
    /// `sign.OnDoubleClick(from)`.
    fn open_sign_menu(&self, w: &mut World, house: ItemId, from: MobileId);
    /// `house.RefreshDecay()`.
    fn refresh_decay(&self, w: &mut World, house: ItemId);
    /// `from.SendGump(new ConfirmDemolishHouseGump(house))`.
    fn confirm_demolish(&self, w: &mut World, house: ItemId, from: MobileId);
}

// ---- runebooks ----

/// The `Sextant.Format` fields.
#[derive(Copy, Clone, Debug)]
pub struct SextantView {
    pub x_long: i32,
    pub y_lat: i32,
    pub x_mins: i32,
    pub y_mins: i32,
    pub x_east: bool,
    pub y_south: bool,
}

#[derive(Clone, Debug)]
pub struct RunebookEntryView {
    pub description: String,
    pub map: Option<MapId>,
    /// `RunebookGump.GetMapHue(entry.Map)`'s argument: the map's index.
    pub map_index: i32,
    pub location: Point3D,
    pub sextant: Option<SextantView>,
    /// `entry == book.Default`.
    pub is_default: bool,
}

/// What `RunebookGump` reads of a `Runebook`.
#[derive(Clone, Debug)]
pub struct RunebookView {
    pub book: ItemId,
    pub cur_charges: i32,
    pub max_charges: i32,
    pub entries: Vec<RunebookEntryView>,
}

/// The spells a rune can be used with.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum RuneSpell {
    /// `new RecallSpell(from, e, book, book)`: a charge is used.
    RecallCharge,
    /// `new RecallSpell(from, e)`.
    Recall,
    /// `new GateTravelSpell(from, e)`.
    Gate,
    /// `new SacredJourneySpell(from, e)`.
    SacredJourney,
}

pub trait RunebookOps {
    fn deleted(&self, w: &World, book: ItemId) -> bool;
    fn world_location(&self, w: &World, book: ItemId) -> Point3D;
    /// `book.Openers.Remove(from)`.
    fn remove_opener(&self, w: &mut World, book: ItemId, from: MobileId);
    /// `DesignContext.Check(from)`.
    fn design_context_check(&self, w: &mut World, from: MobileId) -> bool;
    /// `book.IsLockedDown`.
    fn is_locked_down(&self, w: &World, book: ItemId) -> bool;
    /// `book.CheckAccess(from)`.
    fn check_access(&self, w: &mut World, book: ItemId, from: MobileId) -> bool;
    fn cur_charges(&self, w: &World, book: ItemId) -> i32;
    fn entry_count(&self, w: &World, book: ItemId) -> usize;
    /// `book.OnTravel()`.
    fn on_travel(&self, w: &mut World, book: ItemId);
    /// `book.DropRune(from, e, index)`.
    fn drop_rune(&self, w: &mut World, book: ItemId, from: MobileId, index: usize);
    /// `book.Default = book.Entries[index]`.
    fn set_default(&self, w: &mut World, book: ItemId, index: usize);
    /// `book.SendGumpTo(from)`.
    fn send_gump_to(&self, w: &mut World, book: ItemId, from: MobileId);
    /// `book.Description = text`, already trimmed and `FixHtml`ed.
    fn set_description(&self, w: &mut World, book: ItemId, text: &str);
    /// The entry's location and the sextant text of `SendTargetCoordinates`.
    fn entry_location(
        &self,
        w: &World,
        book: ItemId,
        index: usize,
    ) -> Option<(Point3D, Option<MapId>, Option<SextantView>)>;
    /// `Spellbook.Find(from, spellID)?.HasSpell(spellID)`.
    fn has_spell(&self, w: &World, from: MobileId, spell_id: i32) -> bool;
    /// `new <Spell>(..).Cast()` for the entry.
    fn cast(&self, w: &mut World, book: ItemId, from: MobileId, index: usize, spell: RuneSpell);
}

// ---- player vendors ----

/// A `VendorItem` for sale.
#[derive(Clone, Debug)]
pub struct VendorItemView {
    pub item: ItemId,
    pub price: i32,
    pub description: String,
    pub valid: bool,
}

/// `PlayerVendor` as the buy and owner gumps use it.
#[derive(Clone, Debug)]
pub struct PlayerVendorView {
    pub vendor: MobileId,
    pub charge_per_day: i32,
    pub charge_per_real_world_day: i32,
    pub hold_gold: i32,
    pub bank_account: i32,
    pub shop_name: String,
    pub female: bool,
    /// `RentedVendor.ComputeRentalExpireDelay(out days, out hours)` when the vendor is rented.
    pub rental_expires: Option<(i32, i32)>,
}

pub trait PlayerVendorOps {
    fn view(&self, w: &World, vendor: MobileId) -> Option<PlayerVendorView>;
    /// `vendor.CanInteractWith(from, ownerOnly)`.
    fn can_interact_with(
        &self,
        w: &mut World,
        vendor: MobileId,
        from: MobileId,
        owner_only: bool,
    ) -> bool;
    fn is_owner(&self, w: &World, vendor: MobileId, from: MobileId) -> bool;
    /// `PlayerBarkeeper.IsOwner(from)`; `None` when the mobile is no barkeeper.
    fn barkeeper_is_owner(&self, w: &World, vendor: MobileId, from: MobileId) -> Option<bool>;
    /// Whether the mobile is a `PlayerVendor` (else it is a barkeeper).
    fn is_player_vendor(&self, w: &World, vendor: MobileId) -> bool;
    fn say(&self, w: &mut World, vendor: MobileId, text: &str);
    fn say_to(&self, w: &mut World, vendor: MobileId, to: MobileId, number: i32);
    /// The sale of `item` to `from` at `price` (the whole `OnResponse` body after the gold counts, where the
    /// vendor holds the gold).
    fn add_hold_gold(&self, w: &mut World, vendor: MobileId, amount: i32);
    fn vendor_item_valid_in_pack(&self, w: &World, vendor: MobileId, item: ItemId) -> bool;
    fn open_backpack(&self, w: &mut World, vendor: MobileId, from: MobileId);
    fn check_teleport(&self, w: &mut World, vendor: MobileId, from: MobileId);
    fn rename_shop(&self, w: &mut World, vendor: MobileId, from: MobileId);
    fn rename(&self, w: &mut World, vendor: MobileId, from: MobileId);
    fn display_paperdoll_to(&self, w: &mut World, vendor: MobileId, from: MobileId);
    fn collect_gold(&self, w: &mut World, vendor: MobileId, from: MobileId);
    fn dismiss(&self, w: &mut World, vendor: MobileId, from: MobileId);
}

// ---- rental contracts ----

/// A `VendorRentalDuration`.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct RentalDuration {
    /// The cliloc of the duration's name.
    pub name: i32,
}

/// `VendorRentalDuration.Instances`.
pub const RENTAL_DURATIONS: [RentalDuration; 4] = [
    RentalDuration { name: 1062361 }, // 1 Week
    RentalDuration { name: 1062362 }, // 2 Weeks
    RentalDuration { name: 1062363 }, // 3 Weeks
    RentalDuration { name: 1062364 }, // 1 Month
];

/// What the rental gumps read: the contract or the rented vendor.
#[derive(Clone, Debug)]
pub struct RentalView {
    pub duration: usize,
    pub price: i32,
    pub renewal_price: i32,
    pub landlord_name: Option<String>,
    pub renter_name: Option<String>,
    pub landlord_renew: bool,
    pub renter_renew: bool,
    pub renew: bool,
}

/// The effects of the rental gumps' buttons. `target` is the `VendorRentalContract` item or the `RentedVendor`.
#[allow(clippy::result_unit_err)] // `Err` is a failed call; it carries nothing
pub trait RentalOps {
    fn deleted_contract(&self, w: &World, contract: ItemId) -> bool;
    /// `contract.IsUsableBy(from, locked, accessible, inHouse, ownerOrCoOwner)`.
    fn contract_usable_by(
        &self,
        w: &mut World,
        contract: ItemId,
        from: MobileId,
        flags: [bool; 4],
    ) -> bool;
    fn contract_view(&self, w: &World, contract: ItemId, from: MobileId) -> Option<RentalView>;
    fn contract_is_locked_down(&self, w: &World, contract: ItemId) -> bool;
    fn contract_set_duration(&self, w: &mut World, contract: ItemId, duration: usize);
    fn contract_set_price(&self, w: &mut World, contract: ItemId, price: i32);
    fn contract_toggle_landlord_renew(&self, w: &mut World, contract: ItemId);
    fn contract_is_landlord(&self, w: &World, contract: ItemId, from: MobileId) -> bool;
    fn contract_set_offeree(&self, w: &mut World, contract: ItemId, offeree: Option<MobileId>);
    fn contract_offeree(&self, w: &World, contract: ItemId) -> Option<MobileId>;
    fn contract_location(&self, w: &World, contract: ItemId) -> Point3D;
    /// `new RentedVendor(from, house, duration, price, landlordRenew, goldToGive)` moved to the contract's place,
    /// then the contract deleted. `Err` carries nothing: no house at the contract.
    fn contract_accept(
        &self,
        w: &mut World,
        contract: ItemId,
        renter: MobileId,
        gold_to_give: i32,
    ) -> Result<(), ()>;
    fn contract_has_house(&self, w: &World, contract: ItemId) -> bool;
    fn contract_can_fit(&self, w: &mut World, contract: ItemId) -> bool;

    fn vendor_view(&self, w: &World, vendor: MobileId) -> Option<RentalView>;
    fn vendor_can_interact(
        &self,
        w: &mut World,
        vendor: MobileId,
        from: MobileId,
        owner_only: bool,
    ) -> bool;
    fn vendor_is_landlord(&self, w: &World, vendor: MobileId, from: MobileId) -> bool;
    fn vendor_toggle_renter_renew(&self, w: &mut World, vendor: MobileId);
    fn vendor_toggle_landlord_renew(&self, w: &mut World, vendor: MobileId);
    /// `vendor.RenewalPrice = price; vendor.RenterRenew = false`.
    fn vendor_set_renewal_price(&self, w: &mut World, vendor: MobileId, price: i32);
    fn vendor_name(&self, w: &World, vendor: MobileId) -> String;
    fn vendor_shop_name(&self, w: &World, vendor: MobileId) -> String;
    fn vendor_add_hold_gold(&self, w: &mut World, vendor: MobileId, amount: i32);
    /// `vendor.Destroy(false)`.
    fn vendor_destroy(&self, w: &mut World, vendor: MobileId);
}

/// The installed trait objects.
#[derive(Default, Clone)]
pub struct Ops {
    pub houses: Option<Rc<dyn HouseOps>>,
    pub runebooks: Option<Rc<dyn RunebookOps>>,
    pub player_vendors: Option<Rc<dyn PlayerVendorOps>>,
    pub rentals: Option<Rc<dyn RentalOps>>,
}
