//! The `Item` data, its enums, and (in submodules) the virtual surface.
//!
//! One `Item` holds what the class `Item` holds, with `CompactInfo` flattened into plain fields (the compact
//! form was a memory optimization). What a subclass adds goes in `ext`. The virtual members are fn pointers in
//! [`ItemType`] (`item_type.rs`), defaulting to the base behavior in [`base`].
//!
//! Statics of `Item` (`DefaultDecayTime`, `LockedDownFlag`, `ScissorCopyLootType`, the delta queue, open slots)
//! belong to the `World`, not here. The members that were not virtual (`Delta`, `MoveToWorld(loc, map)`, `Map`
//! setter, `AddItem`..) are `impl World` functions.
//!
//! The non-virtual members are free functions in [`ops`], the save format is [`serialize`], and the other files of
//! `Item` are the sibling modules. A `Container`'s extra state is [`container::ContainerState`] in
//! [`Item::cont`].

pub mod base;
pub mod bounds;
pub mod container;
pub mod containers;
pub mod decay;
pub mod deps;
pub mod effect_item;
pub mod enumerable;
pub mod item_type;
pub mod messages;
pub mod multi;
pub mod ops;
pub mod secure_trade_container;
pub mod serialize;
pub mod virtual_check;
pub mod virtual_hair;

#[cfg(test)]
mod client_tests;
#[cfg(test)]
mod container_tests;
#[cfg(test)]
mod decay_tests;
#[cfg(test)]
mod item_tests;
#[cfg(test)]
mod lockdown_tests;
#[cfg(test)]
mod skip_tests;

use bitflags::bitflags;
use muo_primitives::{DateTime, Direction, Point3D, Serial};

use crate::ext::Ext;
use crate::ids::{EntityId, ItemId, MobileId};
use crate::interfaces::SpawnerId;
use crate::layer::Layer;
use crate::light_type::LightType;
use crate::map::MapId;
use crate::property_list::PropertyList;

pub use item_type::{ITEM_BASE, ItemType};

bitflags! {
    /// How the item should be updated and resent to nearby clients.
    #[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash)]
    pub struct ItemDelta: i32 {
        const NONE = 0x0000_0000;
        /// Resend the item.
        const UPDATE = 0x0000_0001;
        /// Resend the item only if it is equipped.
        const EQUIP_ONLY = 0x0000_0002;
        /// Resend the item's properties.
        const PROPERTIES = 0x0000_0004;
    }
}

/// What happens to an item on its owner's death.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash)]
pub enum DeathMoveResult {
    /// The item should be placed onto the corpse.
    #[default]
    MoveToCorpse = 0,
    /// The item should remain equipped.
    RemainEquipped = 1,
    /// The item should be placed into the owner's backpack.
    MoveToBackpack = 2,
}

/// An item's loot and steal state: `LootType` is a byte enum, and a cast keeps any byte (a save can carry one
/// without a name), so this is the byte with the named values as constants.
#[derive(Copy, Clone, Default, Eq, PartialEq, Hash)]
pub struct LootType(pub u8);

#[allow(non_upper_case_globals)]
impl LootType {
    /// Stealable. Lootable.
    pub const Regular: LootType = LootType(0);
    /// Unstealable. Unlootable, unless owned by a murderer.
    pub const Newbied: LootType = LootType(1);
    /// Unstealable. Unlootable, always.
    pub const Blessed: LootType = LootType(2);
    /// Stealable. Lootable, always.
    pub const Cursed: LootType = LootType(3);

    /// The loot type of a named value; `None` for a byte without a name (`from_raw` keeps it).
    pub const fn from_u8(v: u8) -> Option<LootType> {
        if v <= 3 { Some(LootType(v)) } else { None }
    }

    /// `(LootType)v`: any byte, as a cast keeps it.
    pub const fn from_raw(v: u8) -> LootType {
        LootType(v)
    }
}

impl std::fmt::Debug for LootType {
    /// `LootType.ToString()`: the name, or the number for a byte without one.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            LootType::Regular => f.write_str("Regular"),
            LootType::Newbied => f.write_str("Newbied"),
            LootType::Blessed => f.write_str("Blessed"),
            LootType::Cursed => f.write_str("Cursed"),
            LootType(n) => write!(f, "{n}"),
        }
    }
}

/// `GetTotal`/`UpdateTotal`'s argument (used by both `Item` and `Mobile`).
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash)]
pub enum TotalType {
    #[default]
    Gold = 0,
    Items = 1,
    Weight = 2,
}

bitflags! {
    /// Which `CompactInfo` members an item had (the save's `ExpandFlag`).
    #[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash)]
    pub struct ExpandFlag: i32 {
        const NONE = 0x000;
        const NAME = 0x001;
        const ITEMS = 0x002;
        const BOUNCE = 0x004;
        const HOLDER = 0x008;
        const BLESSED = 0x010;
        const TEMP_FLAG = 0x020;
        const SAVE_FLAG = 0x040;
        const WEIGHT = 0x080;
        const SPAWNER = 0x100;
    }
}

bitflags! {
    /// `Item.ImplFlag` (private): the item's boolean state, one byte.
    #[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash)]
    pub struct ImplFlag: u8 {
        const NONE = 0x00;
        const VISIBLE = 0x01;
        const MOVABLE = 0x02;
        const DELETED = 0x04;
        const STACKABLE = 0x08;
        const IN_QUEUE = 0x10;
        const INSURED = 0x20;
        const PAID_INSURANCE = 0x40;
        const QUEST_ITEM = 0x80;
    }
}

bitflags! {
    /// `Item.SaveFlag` (private): which fields the item record carries, for the serializer.
    #[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash)]
    pub struct SaveFlag: u32 {
        const NONE = 0x0000_0000;
        const DIRECTION = 0x0000_0001;
        const BOUNCE = 0x0000_0002;
        const LOOT_TYPE = 0x0000_0004;
        const LOCATION_FULL = 0x0000_0008;
        const ITEM_ID = 0x0000_0010;
        const HUE = 0x0000_0020;
        const AMOUNT = 0x0000_0040;
        const LAYER = 0x0000_0080;
        const NAME = 0x0000_0100;
        const PARENT = 0x0000_0200;
        const ITEMS = 0x0000_0400;
        const WEIGHT_NOT_1_OR_0 = 0x0000_0800;
        const MAP = 0x0000_1000;
        const VISIBLE = 0x0000_2000;
        const MOVABLE = 0x0000_4000;
        const STACKABLE = 0x0000_8000;
        const WEIGHT_IS_0 = 0x0001_0000;
        const LOCATION_SBYTE_Z = 0x0002_0000;
        const LOCATION_SHORT_XY = 0x0004_0000;
        const LOCATION_BYTE_XY = 0x0008_0000;
        const IMPL_FLAGS = 0x0010_0000;
        const INSURED_FOR = 0x0020_0000;
        const BLESSED_FOR = 0x0040_0000;
        const HELD_BY = 0x0080_0000;
        const INT_WEIGHT = 0x0100_0000;
        const SAVED_FLAGS = 0x0200_0000;
        const NULL_WEIGHT = 0x0400_0000;
        const PLAYER_CONSTRUCTED = 0x0800_0000;
        const DECAY_RESET = 0x1000_0000;
    }
}

/// Where a lifted item came from (`BounceInfo`), so a refused drop can put it back.
#[derive(Clone, Debug)]
pub struct BounceInfo {
    pub location: Point3D,
    pub world_loc: Point3D,
    pub map: Option<MapId>,
    pub parent: Option<EntityId>,
}

/// `DecayScheduler.SlotNone`: the item is not tracked by the decay scheduler.
pub const DECAY_SLOT_NONE: i8 = -1;

/// An `Item` and everything its subclasses inherit; the subclass is `ty`, what it adds is `ext`.
pub struct Item {
    pub serial: Serial,
    /// The item's class.
    pub ty: &'static ItemType,
    /// Family state for classes that add fields; `Ext::NONE` otherwise.
    pub ext: Ext,

    /// `m_ItemID` (an int).
    pub item_id: i32,
    pub hue: i32,
    pub amount: i32,
    pub layer: Layer,
    pub location: Point3D,
    /// `m_Map`; `None` is no map (an item inside a container keeps the map it had).
    pub map: Option<MapId>,
    /// `m_Parent`: a `Mobile`, an `Item`, or `None` for the world.
    pub parent: Option<EntityId>,
    /// `m_Direction`; for lights it is a [`LightType`], see [`Item::light`].
    pub direction: Direction,
    pub loot_type: LootType,
    /// `m_Flags`: visible, movable, deleted, stackable, ..
    pub flags: ImplFlag,
    pub delta_flags: ItemDelta,
    /// `LastMoved`.
    pub last_moved: DateTime,
    /// `Created`.
    pub created: DateTime,
    /// `DecaySlot`: where `DecayScheduler` tracks this item. Not serialized.
    pub decay_slot: i8,
    /// `m_PropertyList`: the cached property list, `None` until built.
    pub property_list: Option<PropertyList>,
    /// `NoMoveHS`.
    pub no_move_hs: bool,
    /// `PlayerConstructed`.
    pub player_constructed: bool,
    /// Sector links (`Next`, `Previous`, `OnLinkList`).
    pub next: Option<ItemId>,
    pub previous: Option<ItemId>,
    pub on_link_list: bool,

    // CompactInfo, flattened.
    /// `CompactInfo.m_BlessedFor`.
    pub blessed_for: Option<MobileId>,
    /// `CompactInfo.m_Bounce`.
    pub bounce: Option<BounceInfo>,
    /// `CompactInfo.m_HeldBy`: the mobile holding this item on the cursor.
    pub held_by: Option<MobileId>,
    /// `CompactInfo.m_Items`, and `Container._items`: the contents of any item, containers included.
    pub items: Vec<ItemId>,
    /// `CompactInfo.m_Name`; `None` falls back to `default_name`.
    pub name: Option<String>,
    /// `CompactInfo.m_SavedFlags`.
    pub saved_flags: i32,
    /// `CompactInfo.m_Spawner`.
    pub spawner: Option<SpawnerId>,
    /// `CompactInfo.m_DecayReset`.
    pub decay_reset: DateTime,
    /// `CompactInfo.m_TempFlags`.
    pub temp_flags: i32,
    /// `CompactInfo.m_Weight`; a negative value means "use `default_weight`".
    pub weight: f64,
    /// `CompactInfo.Version`, and `Container._version`: bumped when the contents change.
    pub version: i32,
    /// What `Container` (and `BankBox`, `SecureTradeContainer`..) add; `None` for an item that is not one.
    pub cont: Option<Box<container::ContainerState>>,
    /// `BaseMulti.PathInteriorCacheState` (only a multi ever sets it).
    pub path_interior_cache_state: multi::MultiInteriorCacheState,
}

impl Item {
    /// `new Item(itemID)`: visible, movable, amount 1, on `Map.Internal`. `created` and `last_moved` are
    /// `Core.Now`; this leaves them at `DateTime::MIN`, use [`Item::new_at`] (or `World::new_item`) to stamp them.
    pub fn new(serial: Serial, ty: &'static ItemType) -> Item {
        Item::new_at(serial, ty, DateTime::MIN)
    }

    /// `new Item(itemID)` at `now` (`Core.Now`).
    pub fn new_at(serial: Serial, ty: &'static ItemType, now: DateTime) -> Item {
        Item {
            flags: ImplFlag::VISIBLE | ImplFlag::MOVABLE,
            amount: 1,
            map: Some(MapId::INTERNAL),
            last_moved: now,
            created: now,
            ..Item::new_for_load(serial, ty)
        }
    }

    /// `new Item(Serial)`, the constructor the loader uses before `Deserialize`: only the field initializers
    /// run, so no flags, amount 0 and no map. `Created` should be `Core.Now`; stamp it with `created = now`.
    pub fn new_for_load(serial: Serial, ty: &'static ItemType) -> Item {
        Item {
            serial,
            ty,
            ext: Ext::NONE,
            item_id: 0,
            hue: 0,
            amount: 0,
            layer: Layer::Invalid,
            location: Point3D::default(),
            map: None,
            parent: None,
            direction: Direction::NORTH,
            loot_type: LootType::Regular,
            flags: ImplFlag::NONE,
            delta_flags: ItemDelta::NONE,
            last_moved: DateTime::MIN,
            created: DateTime::MIN,
            decay_slot: DECAY_SLOT_NONE,
            property_list: None,
            no_move_hs: false,
            player_constructed: false,
            next: None,
            previous: None,
            on_link_list: false,
            blessed_for: None,
            bounce: None,
            held_by: None,
            items: Vec::new(),
            name: None,
            saved_flags: 0,
            spawner: None,
            decay_reset: DateTime::MIN,
            temp_flags: 0,
            weight: -1.0,
            version: 0,
            cont: None,
            path_interior_cache_state: multi::MultiInteriorCacheState::Unknown,
        }
    }

    pub fn id(&self) -> ItemId {
        ItemId(self.serial)
    }

    /// The container state, made on first use (a loaded container gets its state in `deserialize`).
    pub fn container_mut(&mut self) -> &mut container::ContainerState {
        self.cont.get_or_insert_with(Default::default)
    }

    pub fn get_flag(&self, flag: ImplFlag) -> bool {
        self.flags.intersects(flag)
    }

    pub fn set_flag(&mut self, flag: ImplFlag, value: bool) {
        self.flags.set(flag, value);
    }

    pub fn visible(&self) -> bool {
        self.get_flag(ImplFlag::VISIBLE)
    }

    pub fn movable(&self) -> bool {
        self.get_flag(ImplFlag::MOVABLE)
    }

    pub fn deleted(&self) -> bool {
        self.get_flag(ImplFlag::DELETED)
    }

    pub fn stackable(&self) -> bool {
        self.get_flag(ImplFlag::STACKABLE)
    }

    pub fn quest_item(&self) -> bool {
        self.get_flag(ImplFlag::QUEST_ITEM)
    }

    pub fn insured(&self) -> bool {
        self.get_flag(ImplFlag::INSURED)
    }

    pub fn paid_insurance(&self) -> bool {
        self.get_flag(ImplFlag::PAID_INSURANCE)
    }

    /// `Light`: the direction byte read as a [`LightType`].
    pub fn light(&self) -> Option<LightType> {
        LightType::from_i32(i32::from(self.direction.bits()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::World;

    static MID: ItemType = ItemType {
        full_name: "Server.Items.Mid",
        parent: Some(&ITEM_BASE),
        display_weight: |_, _| true,
        ..ITEM_BASE
    };

    static LEAF: ItemType = ItemType {
        full_name: "Server.Items.Leaf",
        parent: Some(&MID),
        force_show_properties: |_, _| true,
        ..MID
    };

    #[test]
    fn inherits_through_parent() {
        let mut w = World::new();
        let id = w.new_item(&LEAF);
        let leaf = w.item(id).map(|i| i.ty);
        assert_eq!(leaf.map(|t| t.full_name), Some("Server.Items.Leaf"));
        assert!((LEAF.display_weight)(&w, id), "from Mid");
        assert!((LEAF.force_show_properties)(&w, id), "its own");
        assert!(!(MID.force_show_properties)(&w, id), "Mid keeps the base");
        assert!((ITEM_BASE.display_weight)(&w, id), "Core.ML and movable");
        assert_eq!(
            (LEAF.get_lift_sound)(&w, id, MobileId(Serial::new(1))),
            0x57,
            "from Item"
        );
        assert!(LEAF.is(&MID) && LEAF.is(&ITEM_BASE) && LEAF.is(&LEAF));
        assert!(!MID.is(&LEAF));
    }

    #[test]
    fn constructor_defaults() {
        let it = Item::new(Serial::new(0x4000_0001), &ITEM_BASE);
        assert!(it.visible() && it.movable() && !it.deleted());
        assert_eq!(
            (it.amount, it.map, it.weight, it.decay_slot),
            (1, Some(MapId::INTERNAL), -1.0, -1)
        );
        let loaded = Item::new_for_load(Serial::new(0x4000_0002), &ITEM_BASE);
        assert_eq!(
            (loaded.amount, loaded.map, loaded.flags),
            (0, None, ImplFlag::NONE)
        );
    }

    #[test]
    fn label_number_and_decays() {
        let mut w = World::new();
        let id = w.new_item(&ITEM_BASE);
        w.item_mut(id).unwrap().item_id = 0x0EED;
        assert_eq!((ITEM_BASE.label_number)(&w, id), 1_020_000 + 0x0EED);
        assert!((ITEM_BASE.decays)(&w, id));
        w.item_mut(id).unwrap().spawner = Some(SpawnerId(1));
        assert!(!(ITEM_BASE.decays)(&w, id));
    }

    #[test]
    fn flag_values_are_fixed() {
        assert_eq!(ImplFlag::QUEST_ITEM.bits(), 0x80);
        assert_eq!(SaveFlag::DECAY_RESET.bits(), 0x1000_0000);
        assert_eq!(ExpandFlag::SPAWNER.bits(), 0x100);
        assert_eq!(LootType::Cursed.0, 3);
    }
}
