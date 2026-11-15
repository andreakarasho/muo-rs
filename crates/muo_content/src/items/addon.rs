//! `BaseAddon`, `AddonComponent` (with the Anvil, Forge and Localized components) and `BaseAddonDeed`.
//!
//! State: [`AddonData`] on a `BaseAddon` (its components and `Resource`), [`ComponentData`] on an `AddonComponent`,
//! [`DeedData`] on a `BaseAddonDeed`. A concrete addon, component or deed is an `ItemType` whose `family` is an
//! [`AddonClass`] / [`ComponentClass`] / [`DeedClass`] (the virtuals a leaf overrides); the leaves in `muo_items`
//! are `leaf!` calls of muo_items with these tables in `family`.
//!
//! Houses (`BaseHouse`) live elsewhere: the addon calls them through [`AddonHooks`] in `w.resources`; unset, no
//! house exists (`FindHouseAt` is null), so a deed answers "You can only place this in a house that you own!".
//!
//! The `[Anvil]`/`[Forge]` markers of the components are `craft::ANVIL_CLASSES`/`FORGE_CLASSES`.

use muo_io::{GenericReader, Result};
use muo_primitives::Point3D;
use muo_server::World;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType, LootType, base, deps, ops};
use muo_server::light_type::LightType;
use muo_server::map::MapId;
use muo_server::serialization_ext::{ReadEntityExt, WriteEntityExt};
use muo_server::target::{self, TargetFlags, Targeted};

use crate::core;
use crate::craft::{self, CraftResource};

/// `AddonFitResult`.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum AddonFitResult {
    Valid,
    Blocked,
    NotInHouse,
    DoorTooClose,
    NoWall,
    DoorsNotClosed,
}

type FindHouseAtPointFn = fn(&mut World, Point3D, MapId, i32) -> Option<ItemId>;
/// The calls an addon makes into `BaseHouse` and `BaseDoor` (Multis, owned by another crate). Unset: there is no house.
#[derive(Default)]
pub struct AddonHooks {
    /// `BaseHouse.FindHouseAt(p, map, height)`.
    pub find_house_at_point: Option<FindHouseAtPointFn>,
    /// `BaseHouse.FindHouseAt(item)`.
    pub find_house_at_item: Option<fn(&mut World, ItemId) -> Option<ItemId>>,
    /// `house.IsOwner(m)`.
    pub house_is_owner: Option<fn(&mut World, ItemId, MobileId) -> bool>,
    /// `BaseHouse.FindHouseAt(item)?.IsCoOwner(m) == true`.
    pub item_house_co_owner: Option<fn(&World, ItemId, MobileId) -> bool>,
    /// `house.Addons.Contains(addon)`.
    pub house_addons_contains: Option<fn(&World, ItemId, ItemId) -> bool>,
    /// `house.Addons.Add(addon)`.
    pub house_addons_add: Option<fn(&mut World, ItemId, ItemId)>,
    /// `house.Addons.Remove(addon)`.
    pub house_addons_remove: Option<fn(&mut World, ItemId, ItemId)>,
    /// `house.Doors` (a null entry is left out).
    pub house_doors: Option<fn(&World, ItemId) -> Vec<ItemId>>,
    /// `door.Open`.
    pub door_open: Option<fn(&World, ItemId) -> bool>,
    /// `house.AddSecure(from, addon)`.
    pub house_add_secure: Option<fn(&mut World, ItemId, MobileId, ItemId)>,
}

fn hooks(w: &World) -> Option<&AddonHooks> {
    w.resources.get::<AddonHooks>()
}

/// Mutable access to the hooks, for the owner of houses to set.
pub fn hooks_mut(w: &mut World) -> &mut AddonHooks {
    w.resources.get_mut::<AddonHooks>()
}

fn find_house_at_point(w: &mut World, p: Point3D, map: MapId, height: i32) -> Option<ItemId> {
    let f = hooks(w)?.find_house_at_point?;
    f(w, p, map, height)
}

/// `BaseHouse.FindHouseAt(item)`.
pub fn find_house_at_item(w: &mut World, item: ItemId) -> Option<ItemId> {
    let f = hooks(w)?.find_house_at_item?;
    f(w, item)
}

pub fn house_is_owner(w: &mut World, house: ItemId, m: MobileId) -> bool {
    match hooks(w).and_then(|h| h.house_is_owner) {
        Some(f) => f(w, house, m),
        None => false,
    }
}

/// `BaseHouse.FindHouseAt(item)?.IsCoOwner(m) == true`.
pub fn item_house_co_owner(w: &World, item: ItemId, m: MobileId) -> bool {
    hooks(w)
        .and_then(|h| h.item_house_co_owner)
        .is_some_and(|f| f(w, item, m))
}

pub fn house_doors(w: &World, house: ItemId) -> Vec<ItemId> {
    hooks(w)
        .and_then(|h| h.house_doors)
        .map_or_else(Vec::new, |f| f(w, house))
}

pub fn door_open(w: &World, door: ItemId) -> bool {
    hooks(w)
        .and_then(|h| h.door_open)
        .is_some_and(|f| f(w, door))
}

/// `house.Addons.Remove(addon)`.
pub fn house_addons_remove(w: &mut World, house: ItemId, addon: ItemId) {
    if let Some(f) = hooks(w).and_then(|h| h.house_addons_remove) {
        f(w, house, addon);
    }
}

pub fn house_addons_add(w: &mut World, house: ItemId, addon: ItemId) {
    if let Some(f) = hooks(w).and_then(|h| h.house_addons_add) {
        f(w, house, addon);
    }
}

fn house_addons_contains(w: &World, house: ItemId, addon: ItemId) -> bool {
    hooks(w)
        .and_then(|h| h.house_addons_contains)
        .is_some_and(|f| f(w, house, addon))
}

pub fn house_add_secure(w: &mut World, house: ItemId, from: MobileId, addon: ItemId) {
    if let Some(f) = hooks(w).and_then(|h| h.house_add_secure) {
        f(w, house, from, addon);
    }
}

// ---- state ------------------------------------------------------------------------------------------------

/// What `BaseAddon` adds to an `Item`.
#[derive(Clone, Debug, Default)]
pub struct AddonData {
    pub components: Vec<ItemId>,
    pub resource: CraftResource,
}

/// What `AddonComponent` adds to an `Item`.
#[derive(Clone, Debug, Default)]
pub struct ComponentData {
    pub addon: Option<ItemId>,
    pub offset: Point3D,
}

/// `LocalizedAddonComponent._number`.
#[derive(Clone, Debug, Default)]
pub struct LocalizedData {
    pub number: i32,
}

/// What `BaseAddonDeed` keeps (not serialized): `m_Resource`, and the `East` of the direction deeds.
#[derive(Clone, Debug, Default)]
pub struct DeedData {
    pub resource: CraftResource,
    pub east: bool,
}

pub fn addon_data(w: &World, id: ItemId) -> Option<&AddonData> {
    w.item(id)?.ext.get::<AddonData>()
}

pub fn addon_data_mut(w: &mut World, id: ItemId) -> Option<&mut AddonData> {
    w.item_mut(id)?.ext.get_mut::<AddonData>()
}

pub fn component_data(w: &World, id: ItemId) -> Option<&ComponentData> {
    w.item(id)?.ext.get::<ComponentData>()
}

pub fn component_data_mut(w: &mut World, id: ItemId) -> Option<&mut ComponentData> {
    w.item_mut(id)?.ext.get_mut::<ComponentData>()
}

pub fn deed_data(w: &World, id: ItemId) -> Option<&DeedData> {
    w.item(id)?.ext.get::<DeedData>()
}

/// `BaseAddon.Components`.
pub fn components(w: &World, addon: ItemId) -> Vec<ItemId> {
    addon_data(w, addon).map_or_else(Vec::new, |d| d.components.clone())
}

/// `AddonComponent.Addon`.
pub fn component_addon(w: &World, c: ItemId) -> Option<ItemId> {
    component_data(w, c).and_then(|d| d.addon)
}

/// `AddonComponent.Offset`.
pub fn component_offset(w: &World, c: ItemId) -> Point3D {
    component_data(w, c).map_or_else(Point3D::default, |d| d.offset)
}

/// `item is BaseAddon`.
pub fn is_addon(w: &World, id: ItemId) -> bool {
    w.item(id).is_some_and(|i| i.ty.is(&BASE_ADDON))
}

/// `item is AddonComponent`.
pub fn is_component(w: &World, id: ItemId) -> bool {
    w.item(id).is_some_and(|i| i.ty.is(&ADDON_COMPONENT))
}

// ---- the family tables --------------------------------------------------------------------------------------

/// The virtuals of `BaseAddon` a concrete addon overrides.
#[derive(Copy, Clone)]
pub struct AddonClass {
    /// `RetainDeedHue`.
    pub retain_deed_hue: bool,
    /// `ShareHue`.
    pub share_hue: bool,
    /// `Deed => new XDeed()`; `None` is the base's null.
    pub deed: Option<fn(&mut World) -> ItemId>,
    /// `Deed` of an addon whose deed copies its state (`IsRewardItem`..): wins over `deed`.
    pub deed_of: Option<fn(&mut World, ItemId) -> ItemId>,
    /// `OnComponentLoaded(c)`.
    pub on_component_loaded: fn(&mut World, ItemId, ItemId),
    /// `OnComponentUsed(c, from)`.
    pub on_component_used: fn(&mut World, ItemId, ItemId, MobileId),
    /// `OnChop(from)` override (StoneAnkh refuses); `None` is the base's destroy-and-redeed.
    pub on_chop: Option<fn(&mut World, ItemId, MobileId)>,
}

pub static BASE_ADDON_CLASS: AddonClass = AddonClass {
    retain_deed_hue: false,
    share_hue: true,
    deed: None,
    deed_of: None,
    on_component_loaded: |_, _, _| {},
    on_component_used: |_, _, _, _| {},
    on_chop: None,
};

/// `RetainDeedHue`: the class's `addonRetainDeedHue` prop, else its table.
pub fn retain_deed_hue(w: &World, id: ItemId) -> bool {
    crate::templates::props::as_bool(crate::templates::props::item(w, id, "addonRetainDeedHue"))
        .unwrap_or_else(|| addon_class(w, id).retain_deed_hue)
}

/// `ShareHue`: the class's `addonShareHue` prop, else its table.
pub fn share_hue(w: &World, id: ItemId) -> bool {
    crate::templates::props::as_bool(crate::templates::props::item(w, id, "addonShareHue"))
        .unwrap_or_else(|| addon_class(w, id).share_hue)
}

pub fn addon_class(w: &World, id: ItemId) -> &'static AddonClass {
    w.item(id)
        .and_then(|i| i.ty.family::<AddonClass>())
        .unwrap_or(&BASE_ADDON_CLASS)
}

/// The virtuals of `AddonComponent` a concrete component overrides.
#[derive(Copy, Clone)]
pub struct ComponentClass {
    /// `NeedsWall`.
    pub needs_wall: bool,
    /// `WallPosition`.
    pub wall_position: Point3D,
    /// `WallPosition` of a component that reads its own state (`DartBoard`: `East ? (-1,0,0) : (0,-1,0)`).
    pub wall_position_of: Option<fn(&World, ItemId) -> Point3D>,
}

pub static ADDON_COMPONENT_CLASS: ComponentClass = ComponentClass {
    needs_wall: false,
    wall_position: Point3D { x: 0, y: 0, z: 0 },
    wall_position_of: None,
};

pub fn component_class(w: &World, id: ItemId) -> &'static ComponentClass {
    w.item(id)
        .and_then(|i| i.ty.family::<ComponentClass>())
        .unwrap_or(&ADDON_COMPONENT_CLASS)
}

/// The abstract `Addon` of a `BaseAddonDeed` class: `Addon => new XAddon(..)`, built from the deed (the
/// direction deeds read their `East`).
#[derive(Copy, Clone)]
pub struct DeedClass {
    pub addon: fn(&mut World, ItemId) -> ItemId,
    /// `IDirectionAddonDeed`: double click opens the direction gump (its `SelectionNumber`).
    pub select_direction: Option<i32>,
}

pub static BASE_ADDON_DEED_CLASS: DeedClass = DeedClass {
    addon: |_, _| ItemId(muo_primitives::Serial::MINUS_ONE),
    select_direction: None,
};

pub fn deed_class(w: &World, id: ItemId) -> &'static DeedClass {
    w.item(id)
        .and_then(|i| i.ty.family::<DeedClass>())
        .unwrap_or(&BASE_ADDON_DEED_CLASS)
}

// ---- AddonComponent ---------------------------------------------------------------------------------------

struct LightEntry {
    light: LightType,
    ids: &'static [i32],
}

static LIGHT_ENTRIES: [LightEntry; 4] = [
    LightEntry {
        light: LightType::WestSmall,
        ids: &[
            1122, 1123, 1124, 1141, 1142, 1143, 1144, 1145, 1146, 2347, 2359, 2360, 2361, 2362,
            2363, 2364, 2387, 2388, 2389, 2390, 2391, 2392,
        ],
    },
    LightEntry {
        light: LightType::NorthSmall,
        ids: &[
            1131, 1133, 1134, 1147, 1148, 1149, 1150, 1151, 1152, 2352, 2373, 2374, 2375, 2376,
            2377, 2378, 2401, 2402, 2403, 2404, 2405, 2406,
        ],
    },
    LightEntry {
        light: LightType::Circle300,
        ids: &[6526, 6538, 6571],
    },
    LightEntry {
        light: LightType::Circle150,
        ids: &[5703, 6587],
    },
];

/// `AddonComponent.ApplyLightTo(item)`.
pub fn apply_light_to(w: &mut World, item: ItemId) {
    if !ops::item_data(w, item)
        .flags
        .contains(muo_data::tile_data::TileFlag::LIGHT_SOURCE)
    {
        return; // not a light source
    }
    let item_id = w.item(item).map_or(0, |i| i.item_id);
    for e in &LIGHT_ENTRIES {
        if e.ids.contains(&item_id) {
            ops::set_light(w, item, e.light);
            return;
        }
    }
}

/// `new AddonComponent(itemID)` of class `ty` (`Movable = false; ApplyLightTo(this)`).
pub fn new_component(w: &mut World, ty: &'static ItemType, item_id: i32) -> ItemId {
    let id = ops::new_item_with_id(w, ty, item_id);
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(ComponentData::default());
    }
    ops::set_movable(w, id, false);
    apply_light_to(w, id);
    id
}

/// `new LocalizedAddonComponent(itemID, labelNumber)` of class `ty`.
pub fn new_localized_component(
    w: &mut World,
    ty: &'static ItemType,
    item_id: i32,
    label: i32,
) -> ItemId {
    let id = new_component(w, ty, item_id);
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(LocalizedData { number: label });
    }
    id
}

fn component_set_hue(w: &mut World, id: ItemId, value: i32) {
    base::set_hue_base(w, id, value);
    if let Some(a) = component_addon(w, id)
        && share_hue(w, a)
        && let Some(ty) = w.item(a).map(|i| i.ty)
    {
        (ty.set_hue)(w, a, value);
    }
}

/// `AddonComponent.OnChop`.
pub fn component_on_chop(w: &mut World, id: ItemId, from: MobileId) {
    let loc = ops::get_world_location(w, id);
    match component_addon(w, id) {
        Some(a) if deps::mobile_in_range(w, from, loc, 3) => addon_on_chop(w, a, from),
        _ => deps::send_localized_message(w, from, 500446), // That is too far away.
    }
}

fn component_on_location_change(w: &mut World, id: ItemId, _old: Point3D) {
    if let Some(a) = component_addon(w, id) {
        let (loc, off) = (
            w.item(id).map_or_else(Point3D::default, |i| i.location),
            component_offset(w, id),
        );
        ops::set_location(
            w,
            a,
            Point3D::new(loc.x - off.x, loc.y - off.y, loc.z - off.z),
        );
    }
}

fn component_on_map_change(w: &mut World, id: ItemId) {
    if let Some(a) = component_addon(w, id) {
        let map = w.item(id).and_then(|i| i.map);
        ops::set_map(w, a, map);
    }
}

fn component_on_after_delete(w: &mut World, id: ItemId) {
    base::on_after_delete_base(w, id);
    if let Some(a) = component_addon(w, id)
        && let Some(ty) = w.item(a).map(|i| i.ty)
    {
        (ty.delete)(w, a);
    }
}

fn component_on_double_click(w: &mut World, id: ItemId, from: MobileId) {
    if let Some(a) = component_addon(w, id) {
        (addon_class(w, a).on_component_used)(w, a, id, from);
    }
}

fn component_after_deserialization(w: &mut World, id: ItemId) {
    if let Some(a) = component_addon(w, id) {
        (addon_class(w, a).on_component_loaded)(w, a, id);
    }
    apply_light_to(w, id);
}

/// `Server.Items.AddonComponent`.
pub static ADDON_COMPONENT: ItemType = ItemType {
    full_name: "Server.Items.AddonComponent",
    parent: Some(&ITEM_BASE),
    // [Constructible] AddonComponent(int itemID): no parameterless constructor.
    construct: None,
    family: Some(&ADDON_COMPONENT_CLASS),
    set_hue: component_set_hue,
    on_location_change: component_on_location_change,
    on_map_change: component_on_map_change,
    on_after_delete: component_on_after_delete,
    on_double_click: component_on_double_click,
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        out.write_i32(1);
        out.write_item(w, component_addon(w, id));
        out.write_point3d(component_offset(w, id));
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        deserialize_component_body(w, id, r)
    },
    ..ITEM_BASE
};

/// The part of `AddonComponent.Deserialize` after the base: the version, then version 1's fields. Version 0 reads
/// nothing (`Deserialize(reader, version)` only sets `Weight = -1` on a zero weight).
fn deserialize_component_body(w: &mut World, id: ItemId, r: &mut dyn GenericReader) -> Result<()> {
    let version = r.read_i32()?;
    let mut data = ComponentData::default();
    if version < 1 {
        if ops::weight(w, id) == 0.0 {
            ops::set_weight(w, id, -1.0);
        }
    } else {
        data.addon = r.read_item(w)?.filter(|a| is_addon(w, *a));
        data.offset = r.read_point3d()?;
    }
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(data);
    }
    component_after_deserialization(w, id);
    Ok(())
}

/// `Server.Items.AnvilComponent`.
pub static ANVIL_COMPONENT: ItemType = ItemType {
    full_name: "Server.Items.AnvilComponent",
    parent: Some(&ADDON_COMPONENT),
    serialize: |w, id, out| {
        (ADDON_COMPONENT.serialize)(w, id, out);
        out.write_i32(0);
    },
    deserialize: |w, id, r| {
        (ADDON_COMPONENT.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        Ok(())
    },
    ..ADDON_COMPONENT
};

/// `Server.Items.ForgeComponent`.
pub static FORGE_COMPONENT: ItemType = ItemType {
    full_name: "Server.Items.ForgeComponent",
    parent: Some(&ADDON_COMPONENT),
    serialize: |w, id, out| {
        (ADDON_COMPONENT.serialize)(w, id, out);
        out.write_i32(0);
    },
    deserialize: |w, id, r| {
        (ADDON_COMPONENT.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        Ok(())
    },
    ..ADDON_COMPONENT
};

/// `Server.Items.LocalizedAddonComponent`: `LabelNumber => _number`.
pub static LOCALIZED_ADDON_COMPONENT: ItemType = ItemType {
    full_name: "Server.Items.LocalizedAddonComponent",
    parent: Some(&ADDON_COMPONENT),
    label_number: |w, id| {
        w.item(id)
            .and_then(|i| i.ext.get::<LocalizedData>())
            .map_or(0, |d| d.number)
    },
    serialize: |w, id, out| {
        (ADDON_COMPONENT.serialize)(w, id, out);
        out.write_i32(0);
        out.write_i32(
            w.item(id)
                .and_then(|i| i.ext.get::<LocalizedData>())
                .map_or(0, |d| d.number),
        );
    },
    deserialize: |w, id, r| {
        (ADDON_COMPONENT.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        let number = r.read_i32()?;
        if let Some(i) = w.item_mut(id) {
            i.ext.insert(LocalizedData { number });
        }
        Ok(())
    },
    ..ADDON_COMPONENT
};

// ---- BaseAddon --------------------------------------------------------------------------------------------

/// `BaseAddon()`: `base(1)`, `Movable = false; Visible = false`.
pub fn new_addon(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = ops::new_item_with_id(w, ty, 1);
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(AddonData::default());
    }
    ops::set_movable(w, id, false);
    ops::set_visible(w, id, false);
    id
}

fn addon_set_hue(w: &mut World, id: ItemId, value: i32) {
    if w.item(id).is_none_or(|i| i.hue == value) {
        return;
    }
    base::set_hue_base(w, id, value);
    if !ops::is_deleted(w, id) && share_hue(w, id) {
        for c in components(w, id) {
            if let Some(ty) = w.item(c).map(|i| i.ty) {
                (ty.set_hue)(w, c, value);
            }
        }
    }
}

/// `Resource` setter (`OnResourceChanged`: `Hue = CraftResources.GetHue(_resource)`).
pub fn set_addon_resource(w: &mut World, id: ItemId, value: CraftResource) {
    match addon_data_mut(w, id) {
        Some(d) if d.resource != value => d.resource = value,
        _ => return,
    }
    ops::invalidate_properties(w, id);
    let hue = craft::hue(w, value);
    if let Some(ty) = w.item(id).map(|i| i.ty) {
        (ty.set_hue)(w, id, hue);
    }
}

/// `BaseAddon.Resource`.
pub fn addon_resource(w: &World, id: ItemId) -> CraftResource {
    addon_data(w, id).map_or(CraftResource::None, |d| d.resource)
}

/// `BaseAddon.AddComponent(c, x, y, z)`.
pub fn add_component(w: &mut World, addon: ItemId, c: ItemId, x: i32, y: i32, z: i32) {
    if ops::is_deleted(w, addon) {
        return;
    }
    if let Some(d) = addon_data_mut(w, addon) {
        d.components.push(c);
    }
    if let Some(d) = component_data_mut(w, c) {
        d.addon = Some(addon);
        d.offset = Point3D::new(x, y, z);
    }
    let (loc, map) = w
        .item(addon)
        .map_or((Point3D::default(), None), |i| (i.location, i.map));
    ops::move_to_world_map(w, c, Point3D::new(loc.x + x, loc.y + y, loc.z + z), map);
}

/// A component to build in an addon's constructor: `AddComponent(new X(args), x, y, z)`.
#[derive(Copy, Clone)]
pub enum Comp {
    /// `new AddonComponent(itemID)`.
    Plain(i32),
    /// `new LocalizedAddonComponent(itemID, label)`.
    Localized(i32, i32),
    /// `new AnvilComponent(itemID)`.
    Anvil(i32),
    /// `new ForgeComponent(itemID)`.
    Forge(i32),
    /// A component class with its own parameterless constructor.
    Custom(fn(&mut World) -> ItemId),
    /// A component class whose constructor takes the item id.
    Typed(fn(&mut World, i32) -> ItemId, i32),
}

impl Comp {
    pub fn build(self, w: &mut World) -> ItemId {
        match self {
            Comp::Plain(i) => new_component(w, &ADDON_COMPONENT, i),
            Comp::Localized(i, l) => new_localized_component(w, &LOCALIZED_ADDON_COMPONENT, i, l),
            Comp::Anvil(i) => new_component(w, &ANVIL_COMPONENT, i),
            Comp::Forge(i) => new_component(w, &FORGE_COMPONENT, i),
            Comp::Custom(f) => f(w),
            Comp::Typed(f, i) => f(w, i),
        }
    }
}

/// The constructor body of a plain addon: one `AddComponent` per row, in order.
pub fn add_components(w: &mut World, addon: ItemId, rows: &[(Comp, i32, i32, i32)]) {
    for (c, x, y, z) in rows {
        let c = c.build(w);
        add_component(w, addon, c, *x, *y, *z);
    }
}

/// `BaseAddon.CheckHouse(from, p, map, height, out house)`.
pub fn check_house(
    w: &mut World,
    from: Option<MobileId>,
    p: Point3D,
    map: MapId,
    height: i32,
    house: &mut Option<ItemId>,
) -> bool {
    *house = find_house_at_point(w, p, map, height);
    match *house {
        Some(h) => from.is_none_or(|f| house_is_owner(w, h, f)),
        None => false,
    }
}

/// `BaseAddon.IsWall(x, y, z, map)`.
pub fn is_wall(w: &mut World, x: i32, y: i32, z: i32, map: Option<MapId>) -> bool {
    let Some(map) = map else { return false };
    for t in muo_server::map::tiles::static_and_multi_tiles(w, map, x, y) {
        let id = w.maps.tile_data.item(usize::from(t.id));
        if id.flags.contains(muo_data::tile_data::TileFlag::WALL)
            && z + 16 > i32::from(t.z)
            && i32::from(t.z) + t.height(&w.maps.tile_data) > z
        {
            return true;
        }
    }
    false
}

pub fn door_blocks(w: &World, door: ItemId, loc: Point3D, height: i32) -> bool {
    let door_loc = ops::get_world_location(w, door);
    let door_height = ops::item_data(w, door).calc_height();
    muo_primitives::utility::in_range(door_loc.x, door_loc.y, loc.x, loc.y, 1)
        && (loc.z == door_loc.z
            || (loc.z + height > door_loc.z && door_loc.z + door_height > loc.z))
}

/// `BaseAddon.CouldFit(p, map, from, ref house)`.
pub fn could_fit(
    w: &mut World,
    addon: ItemId,
    p: Point3D,
    map: MapId,
    from: Option<MobileId>,
    house: &mut Option<ItemId>,
) -> AddonFitResult {
    if ops::is_deleted(w, addon) {
        return AddonFitResult::Blocked;
    }
    let comps = components(w, addon);
    for &c in &comps {
        let off = component_offset(w, c);
        let p3 = Point3D::new(p.x + off.x, p.y + off.y, p.z + off.z);
        let height = i32::from(ops::item_data(w, c).height);
        let c_z0 = w.item(c).is_some_and(|i| i.location.z == 0);
        if !muo_server::map::fit::can_fit(w, map, p3.x, p3.y, p3.z, height, false, true, c_z0) {
            return AddonFitResult::Blocked;
        }
        if !check_house(w, from, p3, map, height, house) {
            return AddonFitResult::NotInHouse;
        }
        let cc = component_class(w, c);
        if cc.needs_wall {
            let wall = cc.wall_position_of.map_or(cc.wall_position, |f| f(w, c));
            if !is_wall(w, p3.x + wall.x, p3.y + wall.y, p3.z + wall.z, Some(map)) {
                return AddonFitResult::NoWall;
            }
        }
    }

    // shortcut: an addon with no components has no house here, so no doors are checked.
    let Some(h) = *house else {
        return AddonFitResult::Valid;
    };
    for door in house_doors(w, h) {
        for &c in &comps {
            let off = component_offset(w, c);
            let loc = Point3D::new(p.x + off.x, p.y + off.y, p.z + off.z);
            let height = ops::item_data(w, c).calc_height();
            if door_blocks(w, door, loc, height) {
                return AddonFitResult::DoorTooClose;
            }
        }
    }
    AddonFitResult::Valid
}

/// `IAddon.CouldFit(p, map)`.
pub fn could_fit_at(w: &mut World, addon: ItemId, p: Point3D, map: MapId) -> bool {
    let mut house = None;
    could_fit(w, addon, p, map, None, &mut house) == AddonFitResult::Valid
}

/// `BaseAddon.OnChop(from)` (`IChoppable`).
pub fn addon_on_chop(w: &mut World, id: ItemId, from: MobileId) {
    if let Some(f) = addon_class(w, id).on_chop {
        return f(w, id, from);
    }
    let Some(house) = find_house_at_item(w, id) else {
        return;
    };
    if !(house_is_owner(w, house, from) && house_addons_contains(w, house, id)) {
        return;
    }
    let loc = ops::get_world_location(w, id);
    let map = w.item(id).and_then(|i| i.map);
    muo_server::effects::play_sound(w, loc, map, 0x3B3);
    deps::send_localized_message(w, from, 500461); // You destroy the item.

    let class = addon_class(w, id);
    let retain = retain_deed_hue(w, id);
    let mut hue = 0;
    if retain {
        for c in components(w, id) {
            if hue != 0 {
                break;
            }
            let ch = w.item(c).map_or(0, |i| i.hue);
            if ch != 0 {
                hue = ch;
            }
        }
    }

    // the deed before Delete: a `deed_of` may read the addon's class props, gone after Delete
    let deed = class
        .deed_of
        .map(|f| f(w, id))
        .or_else(|| class.deed.map(|f| f(w)));
    if let Some(ty) = w.item(id).map(|i| i.ty) {
        (ty.delete)(w, id);
    }
    house_addons_remove(w, house, id);

    if let Some(deed) = deed {
        if retain && let Some(ty) = w.item(deed).map(|i| i.ty) {
            (ty.set_hue)(w, deed, hue);
        }
        deps::add_to_backpack(w, from, deed);
    }
}

fn addon_on_location_change(w: &mut World, id: ItemId, _old: Point3D) {
    if ops::is_deleted(w, id) {
        return;
    }
    let loc = w.item(id).map_or_else(Point3D::default, |i| i.location);
    for c in components(w, id) {
        let off = component_offset(w, c);
        ops::set_location(
            w,
            c,
            Point3D::new(loc.x + off.x, loc.y + off.y, loc.z + off.z),
        );
    }
}

fn addon_on_map_change(w: &mut World, id: ItemId) {
    if ops::is_deleted(w, id) {
        return;
    }
    let map = w.item(id).and_then(|i| i.map);
    for c in components(w, id) {
        ops::set_map(w, c, map);
    }
}

fn addon_on_after_delete(w: &mut World, id: ItemId) {
    base::on_after_delete_base(w, id);
    for c in components(w, id) {
        if let Some(ty) = w.item(c).map(|i| i.ty) {
            (ty.delete)(w, c);
        }
    }
}

/// `[AfterDeserialization]`: `_components?.Tidy()` (nulls and deleted entries go).
fn addon_after_deserialization(w: &mut World, id: ItemId) {
    let keep: Vec<ItemId> = components(w, id)
        .into_iter()
        .filter(|c| !ops::is_deleted(w, *c))
        .collect();
    if let Some(d) = addon_data_mut(w, id) {
        d.components = keep;
    }
}

/// Reads `ReadEntity<T>()` filtered by class (a serial of another class reads as null).
fn read_typed(
    w: &World,
    r: &mut dyn GenericReader,
    class: &'static ItemType,
) -> Result<Option<ItemId>> {
    Ok(r.read_item(w)?
        .filter(|i| w.item(*i).is_some_and(|x| x.ty.is(class))))
}

/// `Server.Items.BaseAddon` (abstract).
pub static BASE_ADDON: ItemType = ItemType {
    full_name: "Server.Items.BaseAddon",
    parent: Some(&ITEM_BASE),
    construct: None,
    family: Some(&BASE_ADDON_CLASS),
    set_hue: addon_set_hue,
    on_location_change: addon_on_location_change,
    on_map_change: addon_on_map_change,
    on_after_delete: addon_on_after_delete,
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        out.write_i32(3);
        let comps = components(w, id);
        out.write_encoded_int(comps.len() as i32);
        for c in comps {
            out.write_item(w, Some(c));
        }
        out.write_enum(4, addon_resource(w, id) as i64);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let version = r.read_i32()?;
        let mut data = AddonData::default();
        if version < 3 {
            // Deserialize(reader, version): ReadEntityList<AddonComponent>() (an int count), then the resource.
            let count = r.read_i32()?;
            for _ in 0..count.max(0) {
                data.components.extend(read_typed(w, r, &ADDON_COMPONENT)?);
            }
            if version == 2 {
                let v = r.read_encoded_int()?;
                data.resource = read_resource(v)?;
            }
        } else {
            let count = r.read_encoded_int()?;
            for _ in 0..count.max(0) {
                data.components.extend(read_typed(w, r, &ADDON_COMPONENT)?);
            }
            data.resource = super::read_enum(r, CraftResource::from_i32, "CraftResource")?;
        }
        if let Some(i) = w.item_mut(id) {
            i.ext.insert(data);
        }
        addon_after_deserialization(w, id);
        Ok(())
    },
    ..ITEM_BASE
};

fn read_resource(v: i32) -> Result<CraftResource> {
    CraftResource::from_i32(v)
        .ok_or_else(|| muo_io::IoError::InvalidData(format!("CraftResource: no value {v}")))
}

// ---- BaseAddonDeed ----------------------------------------------------------------------------------------

/// `BaseAddonDeed()`: `base(0x14F0)`, `LootType.Newbied` before AOS.
pub fn new_deed(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = ops::new_item_with_id(w, ty, 0x14F0);
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(DeedData::default());
    }
    if !core::aos(w) {
        ops::set_loot_type(w, id, LootType::Newbied);
    }
    id
}

/// `BaseAddonDeed.Resource` setter.
pub fn set_deed_resource(w: &mut World, id: ItemId, value: CraftResource) {
    match w.item_mut(id).and_then(|i| i.ext.get_mut::<DeedData>()) {
        Some(d) if d.resource != value => d.resource = value,
        _ => return,
    }
    let hue = craft::hue(w, value);
    if let Some(ty) = w.item(id).map(|i| i.ty) {
        (ty.set_hue)(w, id, hue);
    }
    ops::invalidate_properties(w, id);
}

/// The message of an unfit result (`500269` and friends); false for `Valid`.
pub fn fit_failure(w: &mut World, from: MobileId, res: AddonFitResult) {
    match res {
        AddonFitResult::Valid => {}
        AddonFitResult::Blocked => deps::send_localized_message(w, from, 500269), // You cannot build that there.
        AddonFitResult::NotInHouse => deps::send_localized_message(w, from, 500274), // ...a house that you own!
        AddonFitResult::DoorsNotClosed => {
            deps::send_message(
                w,
                from,
                "You must close all house doors before placing this.",
            );
        }
        AddonFitResult::DoorTooClose => deps::send_localized_message(w, from, 500271), // ...near the door.
        AddonFitResult::NoWall => deps::send_localized_message(w, from, 500268), // ...mounted on something.
    }
}

/// `IPoint3D` of a target after `SpellHelper.GetSurfaceTop(ref p)`.
pub fn surface_point(w: &World, targeted: Targeted) -> Point3D {
    use crate::systems::spells::helper as sh;
    match targeted {
        Targeted::Static(s) => sh::static_target_surface_top(w, s),
        other => {
            let p = sh::get_surface_top(w, sh::Point3dRef::from(other));
            sh::point_of(w, p)
        }
    }
}

fn deed_on_target(w: &mut World, deed: ItemId, from: MobileId, targeted: Targeted) {
    let map = w.mobile(from).and_then(|m| m.map);
    let Some(map) = map else { return };
    if ops::is_deleted(w, deed) {
        return;
    }
    let backpack = w.mobile(from).and_then(|m| m.backpack).map(EntityId::Item);
    if !ops::is_child_of(w, deed, backpack) {
        deps::send_localized_message(w, from, 1042001); // That must be in your pack for you to use it.
        return;
    }
    let addon = (deed_class(w, deed).addon)(w, deed);
    if !is_addon(w, addon) {
        return;
    }
    let p = surface_point(w, targeted);
    let mut house = None;
    let res = could_fit(w, addon, p, map, Some(from), &mut house);
    if res == AddonFitResult::Valid {
        ops::move_to_world_map(w, addon, p, Some(map));
    } else {
        fit_failure(w, from, res);
    }
    if res == AddonFitResult::Valid {
        if let Some(ty) = w.item(deed).map(|i| i.ty) {
            (ty.delete)(w, deed);
        }
        if let Some(h) = house {
            house_addons_add(w, h, addon);
        }
    } else if let Some(ty) = w.item(addon).map(|i| i.ty) {
        (ty.delete)(w, addon);
    }
}

/// `target = new InternalTarget(deed)`: `base(-1, true, TargetFlags.None)`, `CheckLOS = false`.
pub fn begin_deed_target(w: &mut World, deed: ItemId, from: MobileId) {
    let mut t = target::Target::new(w, -1, true, TargetFlags::NONE);
    t.check_los = false;
    t.handlers.on_target = Some(std::rc::Rc::new(move |w: &mut World, from, targeted| {
        deed_on_target(w, deed, from, targeted);
    }));
    let tid = target::add_target(w, t);
    target::set_mobile_target(w, from, Some(tid));
}

fn in_pack(w: &World, deed: ItemId, from: MobileId) -> bool {
    let backpack = w.mobile(from).and_then(|m| m.backpack).map(EntityId::Item);
    ops::is_child_of(w, deed, backpack)
}

/// `BaseAddonDeed.OnDoubleClick`.
pub fn deed_on_double_click(w: &mut World, id: ItemId, from: MobileId) {
    if in_pack(w, id, from) {
        begin_deed_target(w, id, from);
    } else {
        deps::send_localized_message(w, from, 1042001); // That must be in your pack for you to use it.
    }
}

/// `Server.Items.BaseAddonDeed` (abstract).
pub static BASE_ADDON_DEED: ItemType = ItemType {
    full_name: "Server.Items.BaseAddonDeed",
    parent: Some(&ITEM_BASE),
    construct: None,
    family: Some(&BASE_ADDON_DEED_CLASS),
    default_weight: |_, _| 1.0,
    on_double_click: deed_on_double_click,
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        out.write_i32(0);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        if let Some(i) = w.item_mut(id) {
            i.ext.insert(DeedData::default());
        }
        Ok(())
    },
    ..ITEM_BASE
};

// ---- direction deeds (IDirectionAddonDeed, SelectAddonDirectionGump) --------------------------------------------

/// Whether the deed is a `BaseAddonContainerDeed` (its state is `ContainerDeedData`).
pub fn is_container_deed(w: &World, id: ItemId) -> bool {
    w.item(id).is_some_and(|i| {
        i.ext
            .get::<super::addon_container::ContainerDeedData>()
            .is_some()
    })
}

/// `IDirectionAddonDeed.East`.
pub fn deed_east(w: &World, id: ItemId) -> bool {
    deed_data(w, id).is_some_and(|d| d.east)
}

/// The deed's `Hue` (what `new XAddon(Hue)` passes).
pub fn deed_hue(w: &World, id: ItemId) -> i32 {
    w.item(id).map_or(0, |i| (i.ty.hue)(w, id))
}

/// `SelectAddonDirectionGump<T>`: South / East buttons; the response sets `East` and starts the deed's target.
pub struct SelectAddonDirectionGump<const K: usize> {
    deed: ItemId,
}

/// Each deed's private `InternalGump : SelectAddonDirectionGump<InternalGump>`: its full name (hashed into the
/// type id) and `SelectionNumber`.
const DIRECTION_GUMPS: [(&str, i32); 9] = [
    ("Server.Items.StoneStatueDeed+InternalGump", 1076579),
    ("Server.Items.UnmadeBedDeed+InternalGump", 1076580),
    ("Server.Items.CurtainsDeed+InternalGump", 1076581),
    ("Server.Items.VanityDeed+InternalGump", 1076744),
    ("Server.Items.HangingAxesDeed+InternalGump", 1076745),
    ("Server.Items.HangingSwordsDeed+InternalGump", 1076746),
    ("Server.Items.BrokenVanityDeed+InternalGump", 1076747),
    ("Server.Items.WoodenCoffinDeed+InternalGump", 1076748),
    ("Server.Items.BrokenBedDeed+InternalGump", 1076749),
];

impl<const K: usize> crate::gumps::base::StaticGumpLogic for SelectAddonDirectionGump<K> {
    const TYPE_NAME: &'static str = DIRECTION_GUMPS[K].0;

    fn singleton(&self) -> bool {
        false
    }

    fn build_layout(&self, b: &mut crate::gumps::builder::StaticGumpBuilder) {
        b.add_page(0);
        b.add_background(0, 0, 273, 324, 0x13BE);
        b.add_image_tiled(10, 10, 253, 20, 0xA40);
        b.add_image_tiled(10, 40, 253, 244, 0xA40);
        b.add_image_tiled(10, 294, 253, 20, 0xA40);
        b.add_alpha_region(10, 10, 253, 304);

        b.add_button(10, 294, 0xFB1, 0xFB2, 0);

        b.add_html_localized_color(45, 296, 450, 20, 1060051, 0x7FFF, false, false); // CANCEL
        b.add_html_localized_color(14, 12, 273, 20, DIRECTION_GUMPS[K].1, 0x7FFF, false, false);

        b.add_page(1);

        b.add_button(19, 49, 0x845, 0x846, 1);
        b.add_html_localized_color(44, 47, 213, 20, 1075386, 0x7FFF, false, false); // South
        b.add_button(19, 73, 0x845, 0x846, 2);
        b.add_html_localized_color(44, 71, 213, 20, 1075387, 0x7FFF, false, false); // East
    }

    fn on_response(
        &mut self,
        w: &mut World,
        sender: muo_server::net_state::NetStateId,
        info: &crate::gumps::base::RelayInfo,
    ) {
        if ops::is_deleted(w, self.deed) || info.button_id == 0 {
            return;
        }
        if let Some(m) = w.net_states.get(&sender).and_then(|n| n.mobile) {
            choose_direction_of(w, self.deed, info.button_id != 1, m);
        }
    }
}

/// `_deed.East = east; _deed.SendTarget(from)` (`SendTarget` is `base.OnDoubleClick`).
pub fn choose_direction_of(w: &mut World, deed: ItemId, east: bool, from: MobileId) {
    let container = is_container_deed(w, deed);
    if let Some(i) = w.item_mut(deed) {
        if let Some(d) = i.ext.get_mut::<DeedData>() {
            d.east = east;
        }
        if let Some(d) = i.ext.get_mut::<super::addon_container::ContainerDeedData>() {
            d.east = east;
        }
    }
    if container {
        super::addon_container::deed_on_double_click(w, deed, from);
    } else {
        deed_on_double_click(w, deed, from);
    }
}

/// `IDirectionAddonDeed`'s `SelectionNumber` (the direction gump's text): the class's `deedSelectionNumber` prop,
/// else its table; `None` for a deed without a direction.
pub fn deed_selection_number(w: &World, id: ItemId) -> Option<i32> {
    crate::templates::props::as_i32(crate::templates::props::item(w, id, "deedSelectionNumber"))
        .or_else(|| {
            deed_class(w, id)
                .select_direction
                .filter(|_| !is_container_deed(w, id))
                .or_else(|| super::addon_container::deed_select_direction(w, id))
        })
}

/// The `OnDoubleClick` of a direction deed: the gump in the pack, else message 1062334.
pub fn direction_deed_on_double_click(w: &mut World, id: ItemId, from: MobileId) {
    if !in_pack(w, id, from) {
        deps::send_localized_message(w, from, 1062334); // This item must be in your backpack to be used.
        return;
    }
    let Some(selection_number) = deed_selection_number(w, id) else {
        return;
    };
    let Some(k) = DIRECTION_GUMPS.iter().position(|g| g.1 == selection_number) else {
        return;
    };
    macro_rules! gump {
        ($($k:literal)*) => {
            match k {
                $($k => Box::new(crate::gumps::base::StaticGumpObj::new(60, 63, SelectAddonDirectionGump::<$k> { deed: id })) as Box<dyn crate::gumps::base::Gump>,)*
                _ => return,
            }
        };
    }
    let gump = gump!(0 1 2 3 4 5 6 7 8);
    crate::gumps::send_gump_to_mobile(w, from, gump, false);
}

// ---- shared bodies of the component overrides ---------------------------------------------------------------

/// `OnDoubleClick` of the decorative components: a sound when the mobile stands within 2, else "I can't reach that."
pub fn reach_sound(w: &mut World, id: ItemId, from: MobileId, min: i32, max: i32) {
    let Some(it) = w.item(id) else { return };
    let (loc, map) = (it.location, it.map);
    let near = w.mobile(from).is_some_and(|m| {
        muo_primitives::utility::in_range(loc.x, loc.y, m.location.x, m.location.y, 2)
    });
    if near {
        let sound = muo_primitives::utility::random_min_max(&mut w.rng, min, max);
        muo_server::effects::play_sound(w, loc, map, sound);
    } else {
        deps::local_overhead_message(
            w,
            from,
            muo_server::packets::message::MessageType::REGULAR,
            0x3B2,
            1019045, // I can't reach that.
        );
    }
}

/// `OnMoveOver` of the decorative components: the base answer, and a sound for a visible living player.
pub fn move_over_sound(w: &mut World, id: ItemId, m: MobileId, min: i32, max: i32) -> bool {
    let allow = base::on_move_over(w, id, m);
    if allow
        && let Some(mob) = w.mobile(m)
        && (mob.ty.alive)(w, m)
        && mob.player
        && (mob.access_level == muo_server::mobile::AccessLevel::Player || !mob.hidden)
    {
        let (loc, map) = w
            .item(id)
            .map_or((Point3D::default(), None), |i| (i.location, i.map));
        let sound = muo_primitives::utility::random_min_max(&mut w.rng, min, max);
        muo_server::effects::play_sound(w, loc, map, sound);
    }
    allow
}

/// Registers the concrete classes of this file (the abstract `BaseAddon` and `BaseAddonDeed` are not registered).
pub fn register(types: &mut muo_server::type_registry::TypeRegistry) {
    for t in [
        &ADDON_COMPONENT,
        &ANVIL_COMPONENT,
        &FORGE_COMPONENT,
        &LOCALIZED_ADDON_COMPONENT,
    ] {
        types.register_item(t, &[]);
    }
}
