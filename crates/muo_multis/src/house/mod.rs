//! `BaseHouse`: the state, the class table, queries and the save layout of a player house.
//!
//! `BaseHouse` is a `BaseMulti`; its state is [`HouseData`] in `Item::ext`, what a concrete house class overrides
//! (`Area`, `BaseBanLocation`, `DefaultPrice`, `GetDeed`..) is its [`HouseClass`] (`ItemType::family`). The rest of
//! the file's members live in the sibling modules: `access` (owner, co-owners, friends, bans), `storage` (lockdowns,
//! secures, the moving crate, relocation), `decay`, `doors`, `save`, `targets`, `transfer`, `region_ops`.
//!
//! The classes this file reaches in crates above it (`PlayerVendor`, `PlayerBarkeeper`, `VendorRentalContract`,
//! doors, `StrongBox`, addons) go through `muo_content::items::house_deps::HouseDeps`.

pub mod access;
pub mod decay;
pub mod doors;
pub mod region_ops;
pub mod save;
pub mod storage;
pub mod targets;
pub mod transfer;
pub mod vendors;

use indexmap::IndexMap;
use muo_content::core;
use muo_data::client::multi_data::MultiComponentList;
use muo_primitives::{DateTime, Direction, Point3D, Rectangle2D, TimeSpan};
use muo_server::World;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::ItemType;
use muo_server::item::multi::{self, BASE_MULTI};
use muo_server::map::MapId;
use muo_server::region::RegionId;

use crate::placement_entry::{self, Table};

/// `DecayType`.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum DecayType {
    Ageless,
    AutoRefresh,
    ManualRefresh,
    Condemned,
}

/// `DecayLevel` (declaration order is the comparison order).
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(i32)]
pub enum DecayLevel {
    Ageless = 0,
    LikeNew,
    Slightly,
    Somewhat,
    Fairly,
    Greatly,
    IDOC,
    Collapsed,
    DemolitionPending,
}

impl DecayLevel {
    pub fn from_i32(n: i32) -> DecayLevel {
        match n {
            1 => Self::LikeNew,
            2 => Self::Slightly,
            3 => Self::Somewhat,
            4 => Self::Fairly,
            5 => Self::Greatly,
            6 => Self::IDOC,
            7 => Self::Collapsed,
            8 => Self::DemolitionPending,
            _ => Self::Ageless,
        }
    }
}

pub use muo_content::regions::house::SecureAccessResult;

/// `SecureLevel`.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
#[repr(u8)]
pub enum SecureLevel {
    Owner = 0,
    CoOwners,
    Friends,
    Anyone,
    Guild,
}

impl SecureLevel {
    /// `(SecureLevel)byte`: any byte is kept, an unnamed one grants nothing in `HasSecureAccess`.
    pub fn from_u8(n: u8) -> SecureLevel {
        match n {
            1 => Self::CoOwners,
            2 => Self::Friends,
            3 => Self::Anyone,
            4 => Self::Guild,
            _ => Self::Owner,
        }
    }
}

/// `SecureInfo`: a secured container and who may open it.
#[derive(Copy, Clone, Debug)]
pub struct SecureInfo {
    pub item: ItemId,
    pub level: SecureLevel,
}

/// `RelocatedEntity`.
#[derive(Copy, Clone, Debug)]
pub struct RelocatedEntity {
    pub entity: EntityId,
    pub relative_location: Point3D,
}

/// `VendorInventory` (Mobiles/Vendors): what a removed vendor left to be reclaimed.
#[derive(Clone, Debug)]
pub struct VendorInventory {
    pub owner: Option<MobileId>,
    pub vendor_name: String,
    pub shop_name: String,
    pub items: Vec<ItemId>,
    pub gold: i32,
    pub expire_time: DateTime,
    /// The expire timer.
    pub timer: muo_server::timer::TimerHandle,
    /// Identifies the inventory inside its house (the timer closure cannot hold a position).
    pub id: u64,
}

/// The state `BaseHouse` adds to an `Item`.
#[derive(Clone, Debug)]
pub struct HouseData {
    /// `LockDowns != null`: false while the record is still being read.
    pub initialized: bool,
    pub current_stage: DecayLevel,
    pub last_decay_level: DecayLevel,
    pub was_undecayable: bool,
    pub owner: Option<MobileId>,
    pub public: bool,
    pub region: Option<RegionId>,
    pub relative_ban_location: Point3D,
    pub trash: Option<ItemId>,
    pub last_refreshed: DateTime,
    pub restrict_decay: bool,
    pub visits: i32,
    pub max_secures: i32,
    pub max_lock_downs: i32,
    pub price: i32,
    pub next_decay_stage: DateTime,
    pub built_on: DateTime,
    pub last_traded: DateTime,
    pub co_owners: Vec<MobileId>,
    pub friends: Vec<MobileId>,
    pub access: Vec<MobileId>,
    pub bans: Vec<MobileId>,
    pub doors: Vec<ItemId>,
    pub lock_downs: Vec<ItemId>,
    pub secures: Vec<SecureInfo>,
    pub addons: Vec<ItemId>,
    pub sign: Option<ItemId>,
    pub player_vendors: Vec<MobileId>,
    pub player_barkeepers: Vec<MobileId>,
    pub vendor_rental_contracts: Vec<ItemId>,
    pub vendor_inventories: Vec<VendorInventory>,
    pub relocated_entities: Vec<RelocatedEntity>,
    pub moving_crate: Option<ItemId>,
    pub internalized_vendors: Vec<MobileId>,
}

impl Default for HouseData {
    fn default() -> Self {
        HouseData {
            initialized: false,
            current_stage: DecayLevel::Ageless,
            last_decay_level: DecayLevel::Ageless,
            was_undecayable: false,
            owner: None,
            public: false,
            region: None,
            relative_ban_location: Point3D::ZERO,
            trash: None,
            last_refreshed: DateTime::MIN,
            restrict_decay: false,
            visits: 0,
            max_secures: 0,
            max_lock_downs: 0,
            price: 0,
            next_decay_stage: DateTime::MIN,
            built_on: DateTime::MIN,
            last_traded: DateTime::MIN,
            co_owners: Vec::new(),
            friends: Vec::new(),
            access: Vec::new(),
            bans: Vec::new(),
            doors: Vec::new(),
            lock_downs: Vec::new(),
            secures: Vec::new(),
            addons: Vec::new(),
            sign: None,
            player_vendors: Vec::new(),
            player_barkeepers: Vec::new(),
            vendor_rental_contracts: Vec::new(),
            vendor_inventories: Vec::new(),
            relocated_entities: Vec::new(),
            moving_crate: None,
            internalized_vendors: Vec::new(),
        }
    }
}

/// The statics of `BaseHouse`: `AllHouses`, `m_Table` (insertion ordered: it is enumerated into lists), and
/// `DecayEnabled`.
#[derive(Default)]
pub struct HouseStatics {
    pub all_houses: Vec<ItemId>,
    pub table: IndexMap<MobileId, Vec<ItemId>>,
    pub decay_enabled: bool,
    pub next_inventory_id: u64,
}

pub fn statics(w: &mut World) -> &mut HouseStatics {
    w.resources.get_mut::<HouseStatics>()
}

pub fn statics_ref(w: &World) -> Option<&HouseStatics> {
    w.resources.get::<HouseStatics>()
}

/// `BaseHouse.MaxCoOwners`.
pub const MAX_CO_OWNERS: usize = 15;
/// `BaseHouse.MaximumBarkeepCount`.
pub const MAXIMUM_BARKEEP_COUNT: usize = 2;

/// What a concrete house class overrides.
pub struct HouseClass {
    /// `Area`: rectangles relative to the house's location.
    pub area: fn(&World, ItemId) -> Vec<Rectangle2D>,
    /// `BaseBanLocation`.
    pub base_ban_location: fn(&World, ItemId) -> Point3D,
    /// `DefaultPrice`.
    pub default_price: fn(&World, ItemId) -> i32,
    /// `ConvertEntry`: the table and index.
    pub convert_entry: fn(&World, ItemId) -> Option<(Table, usize)>,
    /// `ConvertOffsetX/Y/Z`.
    pub convert_offset: (i32, i32, i32),
    /// `GetDeed()`: a new deed, or None (HouseFoundation, `BaseHouse` itself).
    pub get_deed: fn(&mut World, ItemId) -> Option<ItemId>,
    /// `HouseDirection`.
    pub house_direction: Direction,
    /// `DecayPeriod`.
    pub decay_period: TimeSpan,
    /// `IsAosRules`: None is `Core.AOS`.
    pub is_aos_rules: Option<fn(&World, ItemId) -> bool>,
    /// `IsActive`.
    pub is_active: fn(&World, ItemId) -> bool,
    /// `Type.Name`, for `HousePlacementEntry.Find`.
    pub class_name: &'static str,
    /// What an `IsInside(p, height)` override adds to the base answer (BaseContestHouse' teleporters).
    pub is_inside_extra: Option<fn(&World, ItemId, Point3D) -> bool>,
}

/// `BaseHouse`'s own table (abstract).
pub static BASE_HOUSE_CLASS: HouseClass = HouseClass {
    area: |_, _| Vec::new(),
    base_ban_location: |_, _| Point3D::ZERO,
    default_price: |_, _| 0,
    convert_entry: |_, _| None,
    convert_offset: (0, 0, 0),
    get_deed: |_, _| None,
    house_direction: Direction::SOUTH,
    decay_period: TimeSpan::from_days(5),
    is_aos_rules: None,
    is_active: |_, _| true,
    class_name: "BaseHouse",
    is_inside_extra: None,
};

/// `Server.Multis.BaseHouse` (abstract).
pub static BASE_HOUSE: ItemType = ItemType {
    full_name: "Server.Multis.BaseHouse",
    parent: Some(&BASE_MULTI),
    construct: None,
    family: Some(&BASE_HOUSE_CLASS),
    decays: |_, _| false,
    in_range_point3d: |w, id, p, range| in_range(w, id, p.x, p.y, range),
    in_range_point2d: |w, id, p, range| in_range(w, id, p.x, p.y, range),
    serialize: save::serialize,
    deserialize: save::deserialize,
    on_map_change: storage::on_map_change,
    on_location_change: storage::on_location_change,
    on_delete: storage::on_delete,
    on_after_delete: storage::on_after_delete,
    ..BASE_MULTI
};

/// The `HouseClass` of a house.
pub fn class(w: &World, id: ItemId) -> &'static HouseClass {
    w.item(id)
        .and_then(|i| i.ty.family::<HouseClass>())
        .unwrap_or(&BASE_HOUSE_CLASS)
}

/// `item is BaseHouse`.
pub fn is_house(w: &World, id: ItemId) -> bool {
    w.item(id).is_some_and(|i| i.ty.is(&BASE_HOUSE))
}

pub fn data(w: &World, id: ItemId) -> Option<&HouseData> {
    w.item(id)?.ext.get::<HouseData>()
}

pub fn data_mut(w: &mut World, id: ItemId) -> Option<&mut HouseData> {
    w.item_mut(id)?.ext.get_mut::<HouseData>()
}

/// A copy of a house's data, or the empty default (a deleted house reads as one with nothing in it).
pub fn d(w: &World, id: ItemId) -> HouseData {
    data(w, id).cloned().unwrap_or_default()
}

/// `Deleted`.
pub fn deleted(w: &World, id: ItemId) -> bool {
    w.item(id).is_none_or(|i| i.deleted())
}

/// The map of an item.
pub fn map_of(w: &World, id: ItemId) -> Option<MapId> {
    w.item(id).and_then(|i| i.map)
}

pub fn location(w: &World, id: ItemId) -> Point3D {
    w.item(id).map_or(Point3D::ZERO, |i| i.location)
}

/// `NewVendorSystem` (`Core.AOS`).
pub fn new_vendor_system(w: &World) -> bool {
    core::aos(w)
}

/// `MaxFriends`.
pub fn max_friends(w: &World) -> usize {
    if !core::aos(w) { 50 } else { 140 }
}

/// `MaxBans`.
pub fn max_bans(w: &World) -> usize {
    if !core::aos(w) { 50 } else { 140 }
}

/// `IsAosRules`.
pub fn is_aos_rules(w: &World, id: ItemId) -> bool {
    match class(w, id).is_aos_rules {
        Some(f) => f(w, id),
        None => core::aos(w),
    }
}

/// `IsActive`.
pub fn is_active(w: &World, id: ItemId) -> bool {
    (class(w, id).is_active)(w, id)
}

/// `Area`.
pub fn area(w: &World, id: ItemId) -> Vec<Rectangle2D> {
    (class(w, id).area)(w, id)
}

/// `DecayPeriod`.
pub fn decay_period(w: &World, id: ItemId) -> TimeSpan {
    class(w, id).decay_period
}

/// `RestrictedPlacingTime`.
pub fn restricted_placing_time() -> TimeSpan {
    TimeSpan::from_hours(1)
}

/// `BonusStorageScalar`.
pub fn bonus_storage_scalar(w: &World) -> f64 {
    if core::ml(w) { 1.2 } else { 1.0 }
}

/// `Components` through the vtable.
pub fn components(w: &World, id: ItemId) -> &MultiComponentList {
    multi::components(w, id)
}

/// `Owner`.
pub fn owner(w: &World, id: ItemId) -> Option<MobileId> {
    data(w, id).and_then(|h| h.owner)
}

/// `Sign`.
pub fn sign(w: &World, id: ItemId) -> Option<ItemId> {
    data(w, id).and_then(|h| h.sign)
}

/// `Region`.
pub fn region(w: &World, id: ItemId) -> Option<RegionId> {
    data(w, id).and_then(|h| h.region)
}

/// `BanLocation` getter.
pub fn ban_location(w: &World, id: ItemId) -> Point3D {
    let Some(h) = data(w, id) else {
        return Point3D::ZERO;
    };
    if let Some(r) = h.region.and_then(|r| w.region(r)) {
        return r.go_location;
    }
    let rel = h.relative_ban_location;
    let l = location(w, id);
    Point3D::new(l.x + rel.x, l.y + rel.y, l.z + rel.z)
}

/// `BanLocation` setter.
pub fn set_ban_location(w: &mut World, id: ItemId, value: Point3D) {
    let l = location(w, id);
    set_relative_ban_location(
        w,
        id,
        Point3D::new(value.x - l.x, value.y - l.y, value.z - l.z),
    );
}

/// `RelativeBanLocation` setter.
pub fn set_relative_ban_location(w: &mut World, id: ItemId, value: Point3D) {
    let l = location(w, id);
    let region = data_mut(w, id).map(|h| {
        h.relative_ban_location = value;
        h.region
    });
    if let Some(Some(r)) = region
        && let Some(r) = w.region_mut(r)
    {
        r.go_location = Point3D::new(l.x + value.x, l.y + value.y, l.z + value.z);
    }
}

/// `InRange(Point2D, range)`: over the region's rectangles; a house without a region is never in range.
fn in_range(w: &World, id: ItemId, x: i32, y: i32, range: i32) -> bool {
    let Some(r) = region(w, id).and_then(|r| w.region(r)) else {
        return false;
    };
    r.area.iter().any(|rect| {
        x >= rect.start.x - range
            && y >= rect.start.y - range
            && x < rect.end.x + range
            && y < rect.end.y + range
    })
}

/// The `ConvertEntry` of a house.
pub fn convert_entry(w: &World, id: ItemId) -> Option<&'static placement_entry::Entry> {
    let (table, index) = (class(w, id).convert_entry)(w, id)?;
    match table {
        Table::Two => placement_entry::two_story().get(index),
        Table::Three => placement_entry::three_story().get(index),
    }
}

/// `GetAosEntry()`: `HousePlacementEntry.Find(this)`.
pub fn aos_entry(w: &World, id: ItemId) -> Option<&'static placement_entry::Entry> {
    let item_id = w.item(id)?.item_id;
    placement_entry::find(w, class(w, id).class_name, item_id)
}

/// `GetAosMaxSecures()`.
pub fn aos_max_secures(w: &World, id: ItemId) -> i32 {
    aos_entry(w, id).map_or(0, |e| {
        (e.storage(new_vendor_system(w)) as f64 * bonus_storage_scalar(w)) as i32
    })
}

/// `GetAosMaxLockdowns()`.
pub fn aos_max_lockdowns(w: &World, id: ItemId) -> i32 {
    aos_entry(w, id).map_or(0, |e| {
        (e.lockdowns(new_vendor_system(w)) as f64 * bonus_storage_scalar(w)) as i32
    })
}

/// `GetNewVendorSystemMaxVendors()`.
pub fn new_vendor_system_max_vendors(w: &World, id: ItemId) -> i32 {
    aos_entry(w, id).map_or(0, |e| (e.vendors as f64 * bonus_storage_scalar(w)) as i32)
}

/// Gives a house item its (empty) state.
pub fn ensure_data(w: &mut World, id: ItemId) {
    if let Some(i) = w.item_mut(id)
        && i.ext.get::<HouseData>().is_none()
    {
        i.ext.insert(HouseData::default());
    }
}
