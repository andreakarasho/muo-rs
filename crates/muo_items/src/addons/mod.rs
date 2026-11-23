//! The addon classes. `leaves.rs` holds the addons,
//! components and deeds that are constants and `AddComponent` rows); the classes with logic are hand-written next to it.
//!
//! A plain addon's components are its template's `addonComponents`, and an addon and its deed name each other with the
//! props `addonDeed` / `deedAddon` (read by the shared [`TMPL_ADDON_CLASS`] / [`TMPL_DEED_CLASS`]).

use muo_content::items::addon::{self, AddonClass, BASE_ADDON_CLASS, DeedClass};
use muo_content::items::addon_container::{
    self, BASE_ADDON_CONTAINER_CLASS, ContainerAddonClass, ContainerDeedClass,
};
use muo_content::templates::{self, props};
use muo_server::World;
use muo_server::ids::ItemId;
use muo_server::item::ItemType;
use muo_server::type_registry::EntityType;

pub mod devices;
pub mod hand;
pub mod hive;
pub mod leaves;

#[cfg(test)]
mod tests;

/// An item class by its full name.
pub fn item_type_named(w: &World, name: &str) -> Option<&'static ItemType> {
    match w.types.find_type_by_full_name(name, false) {
        Some(EntityType::Item(t)) => Some(t),
        _ => None,
    }
}

/// `new X()` of the class named by the item's class prop `key`; a missing or unknown class is an invalid id.
fn construct_by_prop(w: &mut World, id: ItemId, key: &str) -> ItemId {
    let name = props::as_static_str(w, props::item(w, id, key)).flatten();
    match name.and_then(|n| item_type_named(w, n)) {
        Some(t) if t.construct.is_some() => t.construct.map_or(id, |c| c(w)),
        // an addon only its deed makes (no `[Constructible]` constructor)
        Some(t) if t.is(&addon::BASE_ADDON) => tmpl_addon(w, t),
        Some(t) if t.is(&addon_container::BASE_ADDON_CONTAINER) => tmpl_container(w, t),
        _ => {
            let ty = w.item(id).map_or("", |i| i.ty.full_name);
            tracing::warn!("Templates: {ty}: prop {key}: no constructible item class {name:?}");
            ItemId(muo_primitives::Serial::MINUS_ONE)
        }
    }
}

/// `Deed => new <addonDeed>()`.
pub fn deed_by_prop(w: &mut World, id: ItemId) -> ItemId {
    construct_by_prop(w, id, "addonDeed")
}

/// A deed's `Addon => new <deedAddon>()`.
pub fn addon_by_prop(w: &mut World, deed: ItemId) -> ItemId {
    construct_by_prop(w, deed, "deedAddon")
}

/// A deed's `Addon => new <deedAddon>(Hue)`: the addon takes the deed's hue.
pub fn hued_addon_by_prop(w: &mut World, deed: ItemId) -> ItemId {
    let hue = addon::deed_hue(w, deed);
    let id = construct_by_prop(w, deed, "deedAddon");
    crate::simple::hue(w, id, hue);
    id
}

/// An addon whose deed is its `addonDeed` prop.
pub static TMPL_ADDON_CLASS: AddonClass = AddonClass {
    deed_of: Some(deed_by_prop),
    ..BASE_ADDON_CLASS
};

/// A container addon whose deed is its `addonDeed` prop.
pub static TMPL_CONTAINER_CLASS: ContainerAddonClass = ContainerAddonClass {
    deed_of: Some(deed_by_prop),
    ..BASE_ADDON_CONTAINER_CLASS
};

/// A deed whose addon is its `deedAddon` prop.
pub static TMPL_DEED_CLASS: DeedClass = DeedClass {
    addon: addon_by_prop,
    select_direction: None,
};

/// A deed whose addon is its `deedAddon` prop, made in the deed's hue.
pub static TMPL_HUED_DEED_CLASS: DeedClass = DeedClass {
    addon: hued_addon_by_prop,
    select_direction: None,
};

/// A container deed whose addon is its `deedAddon` prop.
pub static TMPL_CONTAINER_DEED_CLASS: ContainerDeedClass = ContainerDeedClass {
    addon: addon_by_prop,
    select_direction: None,
};

/// `new XAddon()`: the addon, then its template (the `addonComponents`).
pub fn tmpl_addon(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = addon::new_addon(w, ty);
    templates::apply_item_as(w, id, ty.full_name);
    id
}

/// `new XAddon()` of a container addon (`BaseAddonContainer(itemID)`; the graphic is the template's `itemId`).
pub fn tmpl_container(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = addon_container::new_addon_container(w, ty, 0);
    templates::apply_item_as(w, id, ty.full_name);
    addon::apply_light_to(w, id); // the light of the template's graphic
    id
}

/// `new XAddon(bool east)`: the addon, then its template, then the rows of its `addonComponentsEast` or
/// `addonComponentsSouth` prop (and a container's `addonItemIdEast` / `addonItemIdSouth` graphic).
pub fn tmpl_direction_addon(w: &mut World, ty: &'static ItemType, east: bool) -> ItemId {
    let container = ty.is(&addon_container::BASE_ADDON_CONTAINER);
    let id = if container {
        addon_container::new_addon_container(w, ty, 0)
    } else {
        addon::new_addon(w, ty)
    };
    templates::apply_item_as(w, id, ty.full_name);
    let side = if east { "East" } else { "South" };
    if let Some(gid) = props::as_i32(props::item(w, id, &format!("addonItemId{side}"))) {
        (ty.set_item_id)(w, id, gid);
    }
    if container {
        addon::apply_light_to(w, id);
    }
    if let Some(rows) = props::item(w, id, &format!("addonComponents{side}")).cloned() {
        crate::template_keys_misc2::add_component_rows(w, id, &rows);
    }
    id
}

/// A direction deed's `Addon => new <deedAddon>(East)`.
pub fn direction_addon_by_prop(w: &mut World, deed: ItemId) -> ItemId {
    let east = if addon::is_container_deed(w, deed) {
        addon_container::deed_east(w, deed)
    } else {
        addon::deed_east(w, deed)
    };
    let name = props::as_static_str(w, props::item(w, deed, "deedAddon")).flatten();
    match name.and_then(|n| item_type_named(w, n)) {
        Some(t) => tmpl_direction_addon(w, t, east),
        None => {
            tracing::warn!("Templates: direction deed: no addon class {name:?}");
            ItemId(muo_primitives::Serial::MINUS_ONE)
        }
    }
}

/// An `IDirectionAddonDeed` whose addon is its `deedAddon` prop (the gump's text is `deedSelectionNumber`).
pub static TMPL_DIRECTION_DEED_CLASS: DeedClass = DeedClass {
    addon: direction_addon_by_prop,
    select_direction: None,
};

/// A container `IDirectionAddonDeed` whose addon is its `deedAddon` prop.
pub static TMPL_DIRECTION_CONTAINER_DEED_CLASS: ContainerDeedClass = ContainerDeedClass {
    addon: direction_addon_by_prop,
    select_direction: None,
};

/// `new XDeed()`: the deed, then its template (a `lootType`..).
pub fn tmpl_deed(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = addon::new_deed(w, ty);
    templates::apply_item_as(w, id, ty.full_name);
    id
}

/// Registers every class of this folder.
pub fn register(types: &mut muo_server::type_registry::TypeRegistry) {
    devices::register(types);
    hive::register(types);
    hand::register(types);
    leaves::register(types);
}
