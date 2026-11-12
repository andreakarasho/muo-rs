//! What the multis (houses, boats, camps) call in classes that live in other crates: the doors of Items/Construction,
//! `StrongBox`, the addons, player vendors, guildstones. `muo_multis` cannot depend on those crates, so each is a
//! `fn` pointer here, set by the crate that implements the class. An unset hook behaves as
//! if the class were absent: no door is made, nothing is an addon, and so on.

use muo_primitives::Point3D;
use muo_server::World;
use muo_server::ids::{ItemId, MobileId};

/// `DoorFacing` (the door classes).
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(i32)]
pub enum DoorFacing {
    WestCW = 0,
    EastCCW = 1,
    WestCCW = 2,
    EastCW = 3,
    SouthCW = 4,
    NorthCCW = 5,
    SouthCCW = 6,
    NorthCW = 7,
    SouthCW2 = 8,
    SouthCCW2 = 9,
    NorthCW2 = 10,
    NorthCCW2 = 11,
}

impl DoorFacing {
    /// `(DoorFacing)n`; any int is accepted, the table here has the names of 0..=11.
    pub fn from_i32(n: i32) -> DoorFacing {
        match n {
            0 => Self::WestCW,
            1 => Self::EastCCW,
            2 => Self::WestCCW,
            3 => Self::EastCW,
            4 => Self::SouthCW,
            5 => Self::NorthCCW,
            6 => Self::SouthCCW,
            7 => Self::NorthCW,
            8 => Self::SouthCW2,
            9 => Self::SouthCCW2,
            10 => Self::NorthCW2,
            _ => Self::NorthCCW2,
        }
    }
}

/// Which house door class `BaseHouse.MakeDoor` / `AddDoor(itemID, ..)` builds.
#[derive(Copy, Clone, Debug)]
pub enum HouseDoorKind {
    /// `new DarkWoodHouseDoor(facing)`.
    DarkWood(DoorFacing),
    /// `new MetalHouseDoor(facing)`.
    Metal(DoorFacing),
    /// A door class by name that takes only a facing, e.g. `Server.Items.LightWoodGate`.
    Named(&'static str, DoorFacing),
    /// `new GenericHouseDoor(facing, baseItemID, openedSound, closedSound, autoAdjustOffset)`.
    Generic {
        facing: DoorFacing,
        base_item_id: i32,
        open_sound: i32,
        close_sound: i32,
        auto_adjust: bool,
    },
}

/// Which house gump page `HouseGumpAOS.DisplayTo` opens.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum HouseGumpPage {
    Information,
    Vendors,
}

/// The fixed parts of a `StaticWarningGump<T>` subclass.
#[derive(Copy, Clone, Debug)]
pub struct WarningGump {
    /// The subclass's `Type.FullName` (it is hashed into the gump's type id on the wire).
    pub name: &'static str,
    pub header: i32,
    pub header_color: i32,
    pub content: i32,
    pub width: i32,
    pub height: i32,
}

/// An `IAddon` that is not a `BaseAddon`: whether the item is one, its `Deed`, and `CouldFit(p, map)`.
#[derive(Copy, Clone)]
pub struct SimpleAddon {
    pub is: fn(&World, ItemId) -> bool,
    pub deed: fn(&mut World, ItemId) -> ItemId,
    pub could_fit: fn(&mut World, ItemId, Point3D, muo_server::map::MapId) -> bool,
}

type MakeStrongBoxFn = fn(&mut World, MobileId, ItemId) -> Option<ItemId>;
type StrongBoxOwnerFn = fn(&World, ItemId) -> Option<Option<MobileId>>;
type AddonDeedFn = fn(&mut World, ItemId) -> Option<Option<ItemId>>;
type AddonCouldFitFn = fn(&mut World, ItemId, Point3D, Option<muo_server::map::MapId>) -> bool;
type WarningGumpFn = fn(&mut World, MobileId, WarningGump, Box<dyn FnOnce(&mut World, bool)>);
type SextantFormatFn = fn(&World, Point3D, Option<muo_server::map::MapId>) -> Option<String>;
type ConstructFoundationFn = fn(&mut World, MobileId, i32, i32, i32) -> Option<ItemId>;
type ContainerComponentAddonFn = fn(&World, ItemId) -> Option<Option<ItemId>>;
type FindGuildstoneFn =
    fn(&World, muo_server::map::MapId, muo_primitives::Rectangle2D) -> Option<ItemId>;
/// The hooks of the classes the multis call.
#[derive(Default)]
pub struct HouseDeps {
    /// The door of `kind`, not placed.
    pub make_door: Option<fn(&mut World, HouseDoorKind) -> Option<ItemId>>,
    /// `door.Locked = v`.
    pub door_set_locked: Option<fn(&mut World, ItemId, bool)>,
    /// `door.KeyValue = v`.
    pub door_set_key_value: Option<fn(&mut World, ItemId, u32)>,
    /// `door.KeyValue`.
    pub door_key_value: Option<fn(&World, ItemId) -> u32>,
    /// `a.Link = b`.
    pub door_set_link: Option<fn(&mut World, ItemId, ItemId)>,
    /// `door.Open ? door.Offset : null` (the offset to undo when the door is open).
    pub door_open_offset: Option<fn(&World, ItemId) -> Option<Point3D>>,
    /// `((BaseHouseDoor)door).Facing` as a `DoorFacing` number; None when the item is not a house door.
    pub door_facing: Option<fn(&World, ItemId) -> Option<i32>>,
    /// `door.Link` of a door.
    pub door_link: Option<fn(&World, ItemId) -> Option<ItemId>>,
    /// `((BaseHouseDoor)door).Facing = facing`.
    pub door_set_facing: Option<fn(&mut World, ItemId, DoorFacing)>,

    /// `new StrongBox(owner, house)`.
    pub make_strong_box: Option<MakeStrongBoxFn>,
    /// `StrongBox.Owner`; None when the item is not a strong box.
    pub strong_box_owner: Option<StrongBoxOwnerFn>,
    /// `strongBox.ConvertToStandardContainer()`.
    pub strong_box_convert: Option<fn(&mut World, ItemId) -> ItemId>,
    /// `strongBox.Destroy()`.
    pub strong_box_destroy: Option<fn(&mut World, ItemId)>,

    /// `addon.Deed` for an `IAddon`; `None` when the item is not an addon, `Some(None)` for an addon with no deed.
    pub addon_deed: Option<AddonDeedFn>,
    /// `item is BaseAddon { RetainDeedHue: true }` then the first hued component's hue (0 if none), else None.
    pub addon_retain_deed_hue: Option<fn(&World, ItemId) -> Option<i32>>,
    /// `addon.CouldFit(location, map)`.
    pub addon_could_fit: Option<AddonCouldFitFn>,
    /// The `deed is BaseAddonContainerDeed && addon is BaseAddonContainer` / `BaseAddonDeed && BaseAddon` handling of
    /// a removed addon: `c.DropItemsToGround(); deed.Resource = c.Resource` (or the plain addon's resource).
    pub addon_prepare_deed: Option<fn(&mut World, ItemId, ItemId)>,

    /// `PlayerVendor.Return()` of a vendor.
    pub vendor_return: Option<fn(&mut World, MobileId)>,
    /// `PlayerVendor.Destroy(true)`.
    pub vendor_destroy: Option<fn(&mut World, MobileId)>,
    /// `PlayerVendor.CanInteractWith(m, false)`; None when the mobile is no vendor.
    pub vendor_can_interact: Option<fn(&mut World, MobileId, MobileId) -> bool>,
    /// `vendor.Backpack?.TotalItems`.
    pub vendor_pack_items: Option<fn(&World, MobileId) -> i32>,
    /// `item is RentedVendor` for a vendor.
    pub vendor_is_rented: Option<fn(&World, MobileId) -> bool>,

    /// `SectorSpawnCacheManager.InvalidateSectors(map, bounds)` (Engines/Spawners).
    pub invalidate_spawn_sectors:
        Option<fn(&mut World, muo_server::map::MapId, muo_primitives::Rectangle2D)>,
    /// `TreasureMapChest.Fill(chest, level)`.
    pub treasure_map_chest_fill: Option<fn(&mut World, ItemId, i32)>,
    /// The `IAddon` classes that are plain `Item`s (Banner, FlamingHead, ..): each class pushes one.
    pub simple_addons: Vec<SimpleAddon>,
    /// `RewardBrazier.TurnOff()`.
    pub brazier_turn_off: Option<fn(&mut World, ItemId)>,
    /// `item is ISecurable securable`: its `Level` (a `SecureLevel` byte), else None. Every crate that defines an
    /// `ISecurable` class pushes one (see [`securable_level`]).
    pub securables: Vec<fn(&World, ItemId) -> Option<u8>>,
    /// `securable.Level = level` of an `ISecurable` item (a `SecureLevel` byte): true when the item is one of the
    /// class that pushed it. Every crate that pushes a [`Self::securables`] getter pushes the setter too.
    pub securable_setters: Vec<fn(&mut World, ItemId, u8) -> bool>,
    /// `Guild.IsEnemy(other)` for two guilds.
    pub guild_is_enemy:
        Option<fn(&World, muo_server::guild::GuildId, muo_server::guild::GuildId) -> bool>,
    /// `HouseGumpAOS.DisplayTo(m, house, page)` / `HouseGump.DisplayTo(m, house)` (the gump picks by `IsAosRules`).
    pub display_house_gump: Option<fn(&mut World, MobileId, ItemId, HouseGumpPage)>,
    /// `VendorInventoryGump.DisplayTo(m, house)`.
    pub display_vendor_inventory_gump: Option<fn(&mut World, MobileId, ItemId)>,
    /// `m.SendGump(new SetSecureLevelGump(securable, house))` for the secure container `item` of `house`.
    pub set_secure_level_gump: Option<fn(&mut World, MobileId, ItemId, ItemId)>,
    /// `m.SendGump(new ConfirmResizeHouseGump(house))` (closing the `HouseGumpAOS` first).
    pub confirm_resize_house_gump: Option<fn(&mut World, MobileId, ItemId)>,
    /// A `StaticWarningGump<T>`: `(header, header_color, content cliloc, (width, height))`, `callback(okay)` when
    /// the player answers.
    pub warning_gump: Option<WarningGumpFn>,
    /// `Sextant.Format(location, map, ...)` as `{lat}° {mins}'{N/S}, {long}° {mins}'{E/W}`; None when not valid.
    pub sextant_format: Option<SextantFormatFn>,
    /// `!ContentFeatureFlags.HousePlacement && from.AccessLevel < FeatureFlagSettings.RequiredAccessLevel`.
    pub house_placement_blocked: Option<fn(&World, MobileId) -> bool>,
    /// `!ContentFeatureFlags.BoatPlacement && from.AccessLevel < FeatureFlagSettings.RequiredAccessLevel`.
    pub boat_placement_blocked: Option<fn(&World, MobileId) -> bool>,
    /// `new HouseFoundation(owner, multiID, storage, lockdowns)`.
    pub construct_foundation: Option<ConstructFoundationFn>,
    /// `HouseFoundation.Customizer`.
    pub foundation_customizer: Option<fn(&World, ItemId) -> Option<MobileId>>,
    /// `foundation.IsFixture(item)` for a `HouseFoundation` house.
    pub foundation_is_fixture: Option<fn(&World, ItemId, ItemId) -> bool>,
    /// `m.SendGump(new RecentlyCustomizedWarningGump())` unless one is open.
    pub recently_customized_warning: Option<fn(&mut World, MobileId)>,
    pub has_recently_customized_warning: Option<fn(&World, MobileId) -> bool>,
    /// `to.SendGump(new AcceptHouseTransferGump(from, house))`.
    pub accept_house_transfer_gump: Option<fn(&mut World, MobileId, MobileId, ItemId)>,
    /// `AddonContainerComponent.Addon`: `Some(None)` for a component without one, None when not a component.
    pub container_component_addon: Option<ContainerComponentAddonFn>,
    /// `map.GetItemsInBounds<Guildstone>(bounds)`'s first: a guildstone in the box.
    pub find_guildstone: Option<FindGuildstoneFn>,
}

pub fn deps(w: &World) -> Option<&HouseDeps> {
    w.resources.get::<HouseDeps>()
}

pub fn deps_mut(w: &mut World) -> &mut HouseDeps {
    w.resources.get_mut::<HouseDeps>()
}

/// `(item as ISecurable)?.Level`: the first registered class that knows the item answers.
pub fn securable_level(w: &World, item: ItemId) -> Option<u8> {
    deps(w)?.securables.iter().find_map(|f| f(w, item))
}

/// `securable.Level = level`; false when the item is no `ISecurable`.
pub fn set_securable_level(w: &mut World, item: ItemId, level: u8) -> bool {
    let setters = deps(w)
        .map(|d| d.securable_setters.clone())
        .unwrap_or_default();
    setters.iter().any(|f| f(w, item, level))
}
