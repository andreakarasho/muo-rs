//! `BaseAddonContainer`, `AddonContainerComponent` (with `LocalizedContainerComponent`) and
//! `BaseAddonContainerDeed`: the addons that are a container with components around it.
//!
//! State is [`ContainerAddonData`] / [`ContainerComponentData`] in `Item::ext` (the container part itself is plain
//! `BaseContainer`); the per-class constants are [`ContainerAddonClass`] in `family` (`ContClass` of
//! `BaseContainer` is not overridden by these classes, so `cont::class` falls back to the base's).
//!
//! `ICraftable.OnCraft` of the deed is `deed_on_craft`; `house.AddSecure` goes through `AddonHooks`.

use muo_io::Result;
use muo_primitives::Point3D;
use muo_server::World;
use muo_server::context_menu_entry::ContextMenuEntry;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType, LootType, base, deps, ops};
use muo_server::map::MapId;
use muo_server::net_state::NetStateId;
use muo_server::property_list::PropertyList;
use muo_server::serialization_ext::{ReadEntityExt, WriteEntityExt};
use muo_server::target::{self, TargetFlags, Targeted};

use super::addon::{
    self, AddonFitResult, apply_light_to, check_house, find_house_at_item, house_addons_remove,
};
use super::cont::{BASE_CONTAINER, new_base_container};
use crate::core;
use crate::craft::{self, CraftResource};

/// What `BaseAddonContainer` adds to a container.
#[derive(Clone, Debug, Default)]
pub struct ContainerAddonData {
    pub components: Vec<ItemId>,
    pub resource: CraftResource,
}

/// What `AddonContainerComponent` adds to an `Item`.
#[derive(Clone, Debug, Default)]
pub struct ContainerComponentData {
    pub addon: Option<ItemId>,
    pub offset: Point3D,
}

/// `LocalizedContainerComponent._number`.
#[derive(Clone, Debug, Default)]
pub struct LocalizedContainerData {
    pub number: i32,
}

pub fn data(w: &World, id: ItemId) -> Option<&ContainerAddonData> {
    w.item(id)?.ext.get::<ContainerAddonData>()
}

pub fn data_mut(w: &mut World, id: ItemId) -> Option<&mut ContainerAddonData> {
    w.item_mut(id)?.ext.get_mut::<ContainerAddonData>()
}

fn cdata(w: &World, id: ItemId) -> Option<&ContainerComponentData> {
    w.item(id)?.ext.get::<ContainerComponentData>()
}

/// `BaseAddonContainer.Components`.
pub fn components(w: &World, id: ItemId) -> Vec<ItemId> {
    data(w, id).map_or_else(Vec::new, |d| d.components.clone())
}

pub fn component_addon(w: &World, c: ItemId) -> Option<ItemId> {
    cdata(w, c).and_then(|d| d.addon)
}

/// `item is AddonContainerComponent`.
pub fn is_container_component(w: &World, id: ItemId) -> bool {
    cdata(w, id).is_some()
}

fn component_offset(w: &World, c: ItemId) -> Point3D {
    cdata(w, c).map_or_else(Point3D::default, |d| d.offset)
}

/// `item is BaseAddonContainer`.
pub fn is_container_addon(w: &World, id: ItemId) -> bool {
    w.item(id).is_some_and(|i| i.ty.is(&BASE_ADDON_CONTAINER))
}

/// The virtuals of `BaseAddonContainer` a concrete class overrides.
#[derive(Copy, Clone)]
pub struct ContainerAddonClass {
    pub retain_deed_hue: bool,
    pub needs_wall: bool,
    pub share_hue: bool,
    pub wall_position: Point3D,
    /// `Deed => new XDeed()`.
    pub deed: Option<fn(&mut World) -> ItemId>,
    /// `Deed` computed from the addon itself (Aquarium: by its ItemID); wins over `deed`.
    pub deed_of: Option<fn(&mut World, ItemId) -> ItemId>,
    /// `DropItemsToGround()` (Aquarium also restarts the fishes' clocks).
    pub drop_items_to_ground: fn(&mut World, ItemId),
    /// `OnComponentLoaded(c)`.
    pub on_component_loaded: fn(&mut World, ItemId, ItemId),
}

pub static BASE_ADDON_CONTAINER_CLASS: ContainerAddonClass = ContainerAddonClass {
    retain_deed_hue: false,
    needs_wall: false,
    share_hue: true,
    wall_position: Point3D { x: 0, y: 0, z: 0 },
    deed: None,
    deed_of: None,
    drop_items_to_ground,
    on_component_loaded: |_, _, _| {},
};

/// `Deed`.
pub fn deed_for(w: &mut World, id: ItemId) -> Option<ItemId> {
    let c = class(w, id);
    c.deed_of.map(|f| f(w, id)).or_else(|| c.deed.map(|f| f(w)))
}

/// `RetainDeedHue`: the class's `addonRetainDeedHue` prop, else its table.
pub fn retain_deed_hue(w: &World, id: ItemId) -> bool {
    crate::templates::props::as_bool(crate::templates::props::item(w, id, "addonRetainDeedHue"))
        .unwrap_or_else(|| class(w, id).retain_deed_hue)
}

/// `ShareHue`: the class's `addonShareHue` prop, else its table.
pub fn share_hue(w: &World, id: ItemId) -> bool {
    crate::templates::props::as_bool(crate::templates::props::item(w, id, "addonShareHue"))
        .unwrap_or_else(|| class(w, id).share_hue)
}

pub fn class(w: &World, id: ItemId) -> &'static ContainerAddonClass {
    w.item(id)
        .and_then(|i| i.ty.family::<ContainerAddonClass>())
        .unwrap_or(&BASE_ADDON_CONTAINER_CLASS)
}

/// The abstract `Addon` of a `BaseAddonContainerDeed` class.
#[derive(Copy, Clone)]
pub struct ContainerDeedClass {
    pub addon: fn(&mut World, ItemId) -> ItemId,
    /// `IDirectionAddonDeed`: double click opens the direction gump (its `SelectionNumber`).
    pub select_direction: Option<i32>,
}

pub static BASE_ADDON_CONTAINER_DEED_CLASS: ContainerDeedClass = ContainerDeedClass {
    addon: |_, _| ItemId(muo_primitives::Serial::MINUS_ONE),
    select_direction: None,
};

/// `IDirectionAddonDeed.East` of a container deed.
pub fn deed_east(w: &World, id: ItemId) -> bool {
    w.item(id)
        .and_then(|i| i.ext.get::<ContainerDeedData>())
        .is_some_and(|d| d.east)
}

/// `ContainerDeedClass::select_direction` of the deed's class.
pub fn deed_select_direction(w: &World, id: ItemId) -> Option<i32> {
    deed_class(w, id).select_direction
}

fn deed_class(w: &World, id: ItemId) -> &'static ContainerDeedClass {
    w.item(id)
        .and_then(|i| i.ty.family::<ContainerDeedClass>())
        .unwrap_or(&BASE_ADDON_CONTAINER_DEED_CLASS)
}

// ---- AddonContainerComponent --------------------------------------------------------------------------------

/// `new AddonContainerComponent(itemID)` of class `ty` (`Movable = false; ApplyLightTo(this)`).
pub fn new_component(w: &mut World, ty: &'static ItemType, item_id: i32) -> ItemId {
    let id = ops::new_item_with_id(w, ty, item_id);
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(ContainerComponentData::default());
    }
    ops::set_movable(w, id, false);
    apply_light_to(w, id);
    id
}

/// `new LocalizedContainerComponent(itemID, label)`.
pub fn new_localized_component(w: &mut World, item_id: i32, label: i32) -> ItemId {
    let id = new_component(w, &LOCALIZED_CONTAINER_COMPONENT, item_id);
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(LocalizedContainerData { number: label });
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

/// `AddonContainerComponent.OnChop`.
pub fn component_on_chop(w: &mut World, id: ItemId, from: MobileId) {
    let loc = ops::get_world_location(w, id);
    match component_addon(w, id) {
        Some(a) if deps::mobile_in_range(w, from, loc, 3) => on_chop(w, a, from),
        _ => deps::send_localized_message(w, from, 500446), // That is too far away.
    }
}

fn component_after_deserialization(w: &mut World, id: ItemId) {
    if let Some(a) = component_addon(w, id) {
        (class(w, a).on_component_loaded)(w, a, id);
    }
    apply_light_to(w, id);
}

/// `Server.Items.AddonContainerComponent`.
pub static ADDON_CONTAINER_COMPONENT: ItemType = ItemType {
    full_name: "Server.Items.AddonContainerComponent",
    parent: Some(&ITEM_BASE),
    // [Constructible] AddonContainerComponent(int itemID): no parameterless constructor.
    construct: None,
    set_hue: component_set_hue,
    on_drag_drop: |w, id, from, dropped| match component_addon(w, id) {
        Some(a) => w
            .item(a)
            .map(|i| i.ty)
            .is_some_and(|ty| (ty.on_drag_drop)(w, a, from, dropped)),
        None => false,
    },
    // OnComponentUsed: `if (!Deleted) OnDoubleClick(from)` on the addon.
    on_double_click: |w, id, from| {
        if let Some(a) = component_addon(w, id)
            && !ops::is_deleted(w, a)
            && let Some(ty) = w.item(a).map(|i| i.ty)
        {
            (ty.on_double_click)(w, a, from);
        }
    },
    on_location_change: |w, id, _old| {
        if let Some(a) = component_addon(w, id) {
            let loc = w.item(id).map_or_else(Point3D::default, |i| i.location);
            let off = component_offset(w, id);
            ops::set_location(
                w,
                a,
                Point3D::new(loc.x - off.x, loc.y - off.y, loc.z - off.z),
            );
        }
    },
    on_map_change: |w, id| {
        if let Some(a) = component_addon(w, id) {
            let map = w.item(id).and_then(|i| i.map);
            ops::set_map(w, a, map);
        }
    },
    get_properties: |w, id, list: &mut PropertyList| {
        if let Some(a) = component_addon(w, id)
            && let Some(ty) = w.item(a).map(|i| i.ty)
        {
            (ty.get_properties)(w, a, list);
        }
    },
    get_context_menu_entries: |w, id, from, list: &mut Vec<ContextMenuEntry>| {
        if let Some(a) = component_addon(w, id)
            && let Some(ty) = w.item(a).map(|i| i.ty)
        {
            (ty.get_context_menu_entries)(w, a, from, list);
        }
    },
    on_after_delete: |w, id| {
        base::on_after_delete_base(w, id);
        if let Some(a) = component_addon(w, id)
            && let Some(ty) = w.item(a).map(|i| i.ty)
        {
            (ty.delete)(w, a);
        }
    },
    // The client is told the component is a container graphic first, then the plain world item follows.
    send_world_packet_to: |w, id, ns: NetStateId, world: &[u8]| {
        if let Some(mut buf) = muo_server::packets::item::create_world_item(w, id) {
            let graphic = component_addon(w, id)
                .and_then(|a| w.item(a))
                .map_or(0x9AB, |a| a.item_id) as u16;
            if buf.len() >= 9 {
                buf[7..9].copy_from_slice(&graphic.to_be_bytes());
            }
            w.send(ns, &buf);
        }
        base::send_world_packet_to_base(w, id, ns, world);
    },
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        out.write_i32(0);
        out.write_item(w, component_addon(w, id));
        out.write_point3d(component_offset(w, id));
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        let addon = r
            .read_item(w)?
            .filter(|a| w.item(*a).is_some_and(|x| x.ty.is(&BASE_ADDON_CONTAINER)));
        let offset = r.read_point3d()?;
        if let Some(i) = w.item_mut(id) {
            i.ext.insert(ContainerComponentData { addon, offset });
        }
        component_after_deserialization(w, id);
        Ok(())
    },
    ..ITEM_BASE
};

/// `Server.Items.LocalizedContainerComponent`: `LabelNumber => _number > 0 ? _number : base.LabelNumber`.
pub static LOCALIZED_CONTAINER_COMPONENT: ItemType = ItemType {
    full_name: "Server.Items.LocalizedContainerComponent",
    parent: Some(&ADDON_CONTAINER_COMPONENT),
    label_number: |w, id| {
        let n = w
            .item(id)
            .and_then(|i| i.ext.get::<LocalizedContainerData>())
            .map_or(0, |d| d.number);
        if n > 0 {
            n
        } else {
            (ITEM_BASE.label_number)(w, id)
        }
    },
    serialize: |w, id, out| {
        (ADDON_CONTAINER_COMPONENT.serialize)(w, id, out);
        out.write_i32(0);
        out.write_i32(
            w.item(id)
                .and_then(|i| i.ext.get::<LocalizedContainerData>())
                .map_or(0, |d| d.number),
        );
    },
    deserialize: |w, id, r| {
        (ADDON_CONTAINER_COMPONENT.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        let number = r.read_i32()?;
        if let Some(i) = w.item_mut(id) {
            i.ext.insert(LocalizedContainerData { number });
        }
        Ok(())
    },
    ..ADDON_CONTAINER_COMPONENT
};

// ---- BaseAddonContainer -----------------------------------------------------------------------------------

/// `BaseAddonContainer(itemID)` of class `ty`.
pub fn new_addon_container(w: &mut World, ty: &'static ItemType, item_id: i32) -> ItemId {
    let id = new_base_container(w, ty, item_id);
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(ContainerAddonData::default());
    }
    apply_light_to(w, id);
    id
}

fn set_hue(w: &mut World, id: ItemId, value: i32) {
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

/// `Resource` setter (`OnResourceChanged`).
pub fn set_resource(w: &mut World, id: ItemId, value: CraftResource) {
    match data_mut(w, id) {
        Some(d) if d.resource != value => d.resource = value,
        _ => return,
    }
    ops::invalidate_properties(w, id);
    let hue = craft::hue(w, value);
    if let Some(ty) = w.item(id).map(|i| i.ty) {
        (ty.set_hue)(w, id, hue);
    }
}

pub fn resource(w: &World, id: ItemId) -> CraftResource {
    data(w, id).map_or(CraftResource::None, |d| d.resource)
}

/// `AddComponent(c, x, y, z)`.
pub fn add_component(w: &mut World, addon: ItemId, c: ItemId, x: i32, y: i32, z: i32) {
    if ops::is_deleted(w, addon) {
        return;
    }
    if let Some(d) = data_mut(w, addon) {
        d.components.push(c);
    }
    if let Some(d) = w
        .item_mut(c)
        .and_then(|i| i.ext.get_mut::<ContainerComponentData>())
    {
        d.addon = Some(addon);
        d.offset = Point3D::new(x, y, z);
    }
    let (loc, map) = w
        .item(addon)
        .map_or((Point3D::default(), None), |i| (i.location, i.map));
    ops::move_to_world_map(w, c, Point3D::new(loc.x + x, loc.y + y, loc.z + z), map);
}

/// A component in a container addon's constructor.
#[derive(Copy, Clone)]
pub enum CComp {
    /// `new AddonContainerComponent(itemID)`.
    Plain(i32),
    /// `new LocalizedContainerComponent(itemID, label)`.
    Localized(i32, i32),
}

/// The constructor body: one `AddComponent` per row.
pub fn add_components(w: &mut World, addon: ItemId, rows: &[(CComp, i32, i32, i32)]) {
    for (c, x, y, z) in rows {
        let c = match *c {
            CComp::Plain(i) => new_component(w, &ADDON_CONTAINER_COMPONENT, i),
            CComp::Localized(i, l) => new_localized_component(w, i, l),
        };
        add_component(w, addon, c, *x, *y, *z);
    }
}

/// `DropItemsToGround`: every item to the addon's location, last first.
pub fn drop_items_to_ground(w: &mut World, id: ItemId) {
    let loc = w.item(id).map_or_else(Point3D::default, |i| i.location);
    let mut i = w.item(id).map_or(0, |c| c.items.len());
    while i > 0 {
        i -= 1;
        if let Some(it) = w.item(id).and_then(|c| c.items.get(i).copied())
            && let Some(ty) = w.item(it).map(|x| x.ty)
        {
            (ty.move_to_world)(w, it, loc);
        }
    }
}

/// `CouldFit(p, map, from, ref house)`.
pub fn could_fit(
    w: &mut World,
    id: ItemId,
    p: Point3D,
    map: MapId,
    from: Option<MobileId>,
    house: &mut Option<ItemId>,
) -> AddonFitResult {
    if ops::is_deleted(w, id) {
        return AddonFitResult::Blocked;
    }
    let comps = components(w, id);
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
        // AddonContainerComponent: NeedsWall is false and not overridden by any component.
    }

    let height = i32::from(ops::item_data(w, id).height);
    let z0 = w.item(id).is_some_and(|i| i.location.z == 0);
    if !muo_server::map::fit::can_fit(w, map, p.x, p.y, p.z, height, false, true, z0) {
        return AddonFitResult::Blocked;
    }
    if !check_house(w, from, p, map, height, house) {
        return AddonFitResult::NotInHouse;
    }
    let cls = class(w, id);
    if cls.needs_wall {
        let wall = cls.wall_position;
        if !addon::is_wall(w, p.x + wall.x, p.y + wall.y, p.z + wall.z, Some(map)) {
            return AddonFitResult::NoWall;
        }
    }

    if let Some(h) = *house {
        for door in addon::house_doors(w, h) {
            if addon::door_open(w, door) {
                return AddonFitResult::DoorsNotClosed;
            }
            for &c in &comps {
                let off = component_offset(w, c);
                let loc = Point3D::new(p.x + off.x, p.y + off.y, p.z + off.z);
                if addon::door_blocks(w, door, loc, ops::item_data(w, c).calc_height()) {
                    return AddonFitResult::DoorTooClose;
                }
            }
            if addon::door_blocks(w, door, p, ops::item_data(w, id).calc_height()) {
                return AddonFitResult::DoorTooClose;
            }
        }
    }
    AddonFitResult::Valid
}

/// `OnChop(from)`.
pub fn on_chop(w: &mut World, id: ItemId, from: MobileId) {
    let Some(house) = find_house_at_item(w, id) else {
        return;
    };
    if !addon::house_is_owner(w, house, from) {
        return;
    }
    if ops::is_secure(w, id) {
        deps::send_localized_message(w, from, 1074870); // This item must be unlocked/unsecured before re-deeding it.
        return;
    }
    let loc = ops::get_world_location(w, id);
    let map = w.item(id).and_then(|i| i.map);
    muo_server::effects::play_sound(w, loc, map, 0x3B3);
    deps::send_localized_message(w, from, 500461); // You destroy the item.

    let cls = class(w, id);
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

    (cls.drop_items_to_ground)(w, id);
    let res = resource(w, id);
    let deed = deed_for(w, id); // the item is gone after Delete
    if let Some(ty) = w.item(id).map(|i| i.ty) {
        (ty.delete)(w, id);
    }
    house_addons_remove(w, house, id);

    if let Some(deed) = deed {
        set_deed_resource(w, deed, res);
        if retain
            && let Some(ty) = w.item(deed).map(|i| i.ty)
        {
            (ty.set_hue)(w, deed, hue);
        }
        deps::add_to_backpack(w, from, deed);
    }
}

fn after_deserialization(w: &mut World, id: ItemId) {
    apply_light_to(w, id);
}

/// `Server.Items.BaseAddonContainer` (abstract).
pub static BASE_ADDON_CONTAINER: ItemType = ItemType {
    full_name: "Server.Items.BaseAddonContainer",
    parent: Some(&BASE_CONTAINER),
    construct: None,
    family: Some(&BASE_ADDON_CONTAINER_CLASS),
    display_weight: |_, _| false,
    set_hue,
    on_location_change: |w, id, old| {
        (BASE_CONTAINER.on_location_change)(w, id, old);
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
    },
    on_map_change: |w, id| {
        (BASE_CONTAINER.on_map_change)(w, id);
        if ops::is_deleted(w, id) {
            return;
        }
        let map = w.item(id).and_then(|i| i.map);
        for c in components(w, id) {
            ops::set_map(w, c, map);
        }
    },
    on_delete: |w, id| {
        if let Some(house) = find_house_at_item(w, id) {
            house_addons_remove(w, house, id);
        }
        (BASE_CONTAINER.on_delete)(w, id);
    },
    get_properties: |w, id, list: &mut PropertyList| {
        (BASE_CONTAINER.get_properties)(w, id, list);
        let res = resource(w, id);
        if !craft::is_standard(res) {
            list.add(craft::localization_number(res));
        }
    },
    on_after_delete: |w, id| {
        (BASE_CONTAINER.on_after_delete)(w, id);
        for c in components(w, id) {
            if let Some(ty) = w.item(c).map(|i| i.ty) {
                (ty.delete)(w, c);
            }
        }
    },
    invalidate_properties: |w, id| {
        (BASE_CONTAINER.invalidate_properties)(w, id);
        for c in components(w, id) {
            if let Some(ty) = w.item(c).map(|i| i.ty) {
                (ty.invalidate_properties)(w, c);
            }
        }
    },
    serialize: |w, id, out| {
        (BASE_CONTAINER.serialize)(w, id, out);
        out.write_i32(2);
        let comps = components(w, id);
        out.write_encoded_int(comps.len() as i32);
        for c in comps {
            out.write_item(w, Some(c));
        }
        out.write_enum(4, resource(w, id) as i64);
    },
    deserialize: |w, id, r| {
        (BASE_CONTAINER.deserialize)(w, id, r)?;
        let version = r.read_i32()?;
        let mut d = ContainerAddonData::default();
        let read_comp = |w: &World, r: &mut dyn muo_io::GenericReader| -> Result<Option<ItemId>> {
            Ok(r.read_item(w)?.filter(|c| {
                w.item(*c)
                    .is_some_and(|x| x.ty.is(&ADDON_CONTAINER_COMPONENT))
            }))
        };
        if version < 2 {
            // Deserialize(reader, version): an int count; the resource is an enum (v1) or a plain int (v0).
            let n = r.read_i32()?;
            for _ in 0..n.max(0) {
                d.components.extend(read_comp(w, r)?);
            }
            let v = if version == 1 {
                r.read_enum(4)? as i32
            } else {
                r.read_i32()?
            };
            d.resource = CraftResource::from_i32(v).ok_or_else(|| {
                muo_io::IoError::InvalidData(format!("CraftResource: no value {v}"))
            })?;
        } else {
            let n = r.read_encoded_int()?;
            for _ in 0..n.max(0) {
                d.components.extend(read_comp(w, r)?);
            }
            d.resource = super::read_enum(r, CraftResource::from_i32, "CraftResource")?;
        }
        if let Some(i) = w.item_mut(id) {
            i.ext.insert(d);
        }
        after_deserialization(w, id);
        Ok(())
    },
    ..BASE_CONTAINER
};

// ---- BaseAddonContainerDeed -------------------------------------------------------------------------------

/// `BaseAddonContainerDeed()`: `base(0x14F0)`, `LootType.Newbied` before AOS.
pub fn new_deed(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = ops::new_item_with_id(w, ty, 0x14F0);
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(ContainerDeedData::default());
    }
    if !core::aos(w) {
        ops::set_loot_type(w, id, LootType::Newbied);
    }
    id
}

/// `BaseAddonContainerDeed._resource`.
#[derive(Clone, Debug, Default)]
pub struct ContainerDeedData {
    pub resource: CraftResource,
    /// `IDirectionAddonDeed.East` (not serialized).
    pub east: bool,
}

/// `Resource` setter of the deed (`OnResourceChanged`).
pub fn set_deed_resource(w: &mut World, id: ItemId, value: CraftResource) {
    match w
        .item_mut(id)
        .and_then(|i| i.ext.get_mut::<ContainerDeedData>())
    {
        Some(d) if d.resource != value => d.resource = value,
        _ => return,
    }
    ops::invalidate_properties(w, id);
    let hue = craft::hue(w, value);
    if let Some(ty) = w.item(id).map(|i| i.ty) {
        (ty.set_hue)(w, id, hue);
    }
}

fn deed_resource(w: &World, id: ItemId) -> CraftResource {
    w.item(id)
        .and_then(|i| i.ext.get::<ContainerDeedData>())
        .map_or(CraftResource::None, |d| d.resource)
}

fn deed_on_target(w: &mut World, deed: ItemId, from: MobileId, targeted: Targeted) {
    let Some(map) = w.mobile(from).and_then(|m| m.map) else {
        return;
    };
    if ops::is_deleted(w, deed) {
        return;
    }
    let backpack = w.mobile(from).and_then(|m| m.backpack).map(EntityId::Item);
    if !ops::is_child_of(w, deed, backpack) {
        deps::send_localized_message(w, from, 1042001); // That must be in your pack for you to use it.
        return;
    }
    let a = (deed_class(w, deed).addon)(w, deed);
    if !is_container_addon(w, a) {
        return;
    }
    let res = deed_resource(w, deed);
    set_resource(w, a, res);

    let p = addon::surface_point(w, targeted);
    let mut house = None;
    let r = could_fit(w, a, p, map, Some(from), &mut house);
    if r == AddonFitResult::Valid {
        ops::move_to_world_map(w, a, p, Some(map));
    } else {
        addon::fit_failure(w, from, r);
    }
    if r == AddonFitResult::Valid {
        if let Some(ty) = w.item(deed).map(|i| i.ty) {
            (ty.delete)(w, deed);
        }
        if let Some(h) = house {
            addon::house_addons_add(w, h, a);
            addon::house_add_secure(w, h, from, a);
        }
    } else if let Some(ty) = w.item(a).map(|i| i.ty) {
        (ty.delete)(w, a);
    }
}

/// `BaseAddonContainerDeed.OnDoubleClick`.
pub fn deed_on_double_click(w: &mut World, id: ItemId, from: MobileId) {
    let backpack = w.mobile(from).and_then(|m| m.backpack).map(EntityId::Item);
    if ops::is_child_of(w, id, backpack) {
        let mut t = target::Target::new(w, -1, true, TargetFlags::NONE);
        t.check_los = false;
        t.handlers.on_target = Some(std::rc::Rc::new(
            move |w: &mut World, from, targeted: Targeted| {
                deed_on_target(w, id, from, targeted);
            },
        ));
        let tid = target::add_target(w, t);
        target::set_mobile_target(w, from, Some(tid));
    } else {
        deps::send_localized_message(w, from, 1062334); // This item must be in your backpack to be used.
    }
}

/// `Server.Items.BaseAddonContainerDeed` (abstract).
pub static BASE_ADDON_CONTAINER_DEED: ItemType = ItemType {
    full_name: "Server.Items.BaseAddonContainerDeed",
    parent: Some(&ITEM_BASE),
    construct: None,
    family: Some(&BASE_ADDON_CONTAINER_DEED_CLASS),
    default_weight: |_, _| 1.0,
    on_double_click: deed_on_double_click,
    get_properties: |w, id, list: &mut PropertyList| {
        (ITEM_BASE.get_properties)(w, id, list);
        let res = deed_resource(w, id);
        if !craft::is_standard(res) {
            list.add(craft::localization_number(res));
        }
    },
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        out.write_encoded_int(2);
        out.write_enum(4, deed_resource(w, id) as i64);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let version = r.read_encoded_int()?;
        let resource = if version < 2 {
            let v = r.read_i32()?;
            CraftResource::from_i32(v).ok_or_else(|| {
                muo_io::IoError::InvalidData(format!("CraftResource: no value {v}"))
            })?
        } else {
            super::read_enum(r, CraftResource::from_i32, "CraftResource")?
        };
        if let Some(i) = w.item_mut(id) {
            i.ext.insert(ContainerDeedData {
                resource,
                east: false,
            });
        }
        Ok(())
    },
    ..ITEM_BASE
};

/// What `BaseHouse` asks of an `IAddon` (a `BaseAddon` or a `BaseAddonContainer`), filled into `HouseDeps`.
pub fn configure_house_deps(w: &mut World) {
    use super::addon;
    use super::house_deps::deps_mut;
    let d = deps_mut(w);
    // `addon is IAddon`: the deed it gives; `Some(None)` for an addon with no deed
    d.addon_deed = Some(|w, item| {
        if addon::is_addon(w, item) {
            let c = addon::addon_class(w, item);
            Some(
                c.deed_of
                    .map(|f| f(w, item))
                    .or_else(|| c.deed.map(|f| f(w))),
            )
        } else if is_container_addon(w, item) {
            Some(deed_for(w, item))
        } else {
            simple_addon(w, item).map(|sa| Some((sa.deed)(w, item)))
        }
    });
    // only a `BaseAddon` has `RetainDeedHue`; the first hued component's hue, else 0
    d.addon_retain_deed_hue = Some(|w, item| {
        if !addon::is_addon(w, item) || !addon::retain_deed_hue(w, item) {
            return None;
        }
        let hue = addon::components(w, item)
            .into_iter()
            .map(|c| w.item(c).map_or(0, |i| i.hue))
            .find(|&h| h != 0);
        Some(hue.unwrap_or(0))
    });
    d.addon_could_fit = Some(|w, item, loc, map| {
        let Some(map) = map else { return false };
        if let Some(sa) = simple_addon(w, item) {
            return (sa.could_fit)(w, item, loc, map);
        }
        if is_container_addon(w, item) {
            let mut house = None;
            could_fit(w, item, loc, map, None, &mut house) == AddonFitResult::Valid
        } else {
            addon::could_fit_at(w, item, loc, map)
        }
    });
    d.addon_prepare_deed = Some(|w, item, deed| {
        let is_container_deed = w
            .item(deed)
            .is_some_and(|i| i.ty.is(&BASE_ADDON_CONTAINER_DEED));
        if is_container_deed && is_container_addon(w, item) {
            (class(w, item).drop_items_to_ground)(w, item);
            let r = resource(w, item);
            set_deed_resource(w, deed, r);
        } else if w
            .item(deed)
            .is_some_and(|i| i.ty.is(&addon::BASE_ADDON_DEED))
            && addon::is_addon(w, item)
        {
            let r = addon::addon_resource(w, item);
            addon::set_deed_resource(w, deed, r);
        }
    });
    d.container_component_addon = Some(|w, item| {
        w.item(item)
            .is_some_and(|i| i.ty.is(&ADDON_CONTAINER_COMPONENT))
            .then(|| {
                w.item(item)
                    .and_then(|i| i.ext.get::<ContainerComponentData>())
                    .and_then(|c| c.addon)
            })
    });
}

/// `BaseAddonContainerDeed.OnCraft` (`ICraftable`): the resource of the craft colors the deed, unless the player asked
/// for no color; the quality is returned as it came.
pub fn deed_on_craft(w: &mut World, id: ItemId, a: &super::equip_hooks::CraftArgs) -> Option<i32> {
    if !w.item(id)?.ty.is(&BASE_ADDON_CONTAINER_DEED) {
        return None;
    }
    set_deed_resource(w, id, a.resource);
    if a.do_not_color
        && let Some(ty) = w.item(id).map(|i| i.ty)
    {
        (ty.set_hue)(w, id, 0);
    }
    Some(a.quality)
}

fn configure(w: &mut World) {
    crate::craft::hooks_mut(w).craftables.push(deed_on_craft);
}

/// Registers the concrete classes of this file.
pub fn register(types: &mut muo_server::type_registry::TypeRegistry) {
    for t in [&ADDON_CONTAINER_COMPONENT, &LOCALIZED_CONTAINER_COMPONENT] {
        types.register_item(t, &[]);
    }
    types.register_phase_default(
        muo_server::type_registry::Phase::Configure,
        "Server.Items.BaseAddonContainer.HouseDeps",
        configure_house_deps,
    );
    types.register_phase_default(
        muo_server::type_registry::Phase::Configure,
        "Server.Items.BaseAddonContainerDeed.Configure",
        configure,
    );
}

/// The `IAddon` class of an item that is not a `BaseAddon` (see `HouseDeps::simple_addons`).
fn simple_addon(w: &World, item: ItemId) -> Option<super::house_deps::SimpleAddon> {
    super::house_deps::deps(w)?
        .simple_addons
        .iter()
        .copied()
        .find(|s| (s.is)(w, item))
}

/// `targeted is IChoppable` for the addon family (`AddonComponent`, `AddonContainerComponent`, `BaseAddon`,
/// `BaseAddonContainer`): their `OnChop`; false for anything else.
pub fn chop_addon_family(w: &mut World, id: ItemId, from: MobileId) -> bool {
    let Some(ty) = w.item(id).map(|i| i.ty) else {
        return false;
    };
    if ty.is(&addon::ADDON_COMPONENT) {
        addon::component_on_chop(w, id, from);
    } else if ty.is(&ADDON_CONTAINER_COMPONENT) {
        component_on_chop(w, id, from);
    } else if ty.is(&addon::BASE_ADDON) {
        addon::addon_on_chop(w, id, from);
    } else if ty.is(&BASE_ADDON_CONTAINER) {
        on_chop(w, id, from);
    } else {
        return false;
    }
    true
}
