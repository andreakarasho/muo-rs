//! The context menu system: building a menu for an entity, and the 0xBF subcommands
//! 0x13 (request) and 0x15 (response). `SendDisplayContextMenu` is `muo_server::packets::menu`.
//!
//! `network::extended_commands::configure` registers 0x13/0x15 with [`context_menu_request`] and
//! [`context_menu_response`] (the connection's body after the subcommand).

use indexmap::IndexMap;
use muo_net::net_state::NetState;
use muo_primitives::{Point3D, Serial};
use muo_server::World;
use muo_server::context_menu_entry::ContextMenu;
use muo_server::ids::{EntityId, MobileId};
use muo_server::item::ops as item_ops;
use muo_server::mobile::base as mb;
use muo_server::net_state::NetStateId;
use muo_server::packets::menu::send_display_context_menu;

use super::entries;

/// `_menus`: the menu each mobile was last sent.
#[derive(Default)]
pub struct ContextMenuSystem {
    menus: IndexMap<MobileId, ContextMenu>,
}

fn deleted(w: &World, e: EntityId) -> bool {
    match e {
        EntityId::Item(i) => w.item(i).is_none_or(|i| i.deleted()),
        EntityId::Mobile(m) => w.mobile(m).is_none_or(|m| m.deleted),
    }
}

fn location(w: &World, e: EntityId) -> Point3D {
    match e {
        EntityId::Mobile(m) => w.mobile(m).map(|m| m.location).unwrap_or_default(),
        EntityId::Item(i) => item_ops::get_world_location(w, i),
    }
}

fn map_of(w: &World, e: EntityId) -> Option<muo_server::map::MapId> {
    match e {
        EntityId::Mobile(m) => w.mobile(m).and_then(|m| m.map),
        EntityId::Item(i) => w.item(i).and_then(|i| i.map),
    }
}

/// `CreateContextMenu(from, target)`: the paperdoll entry first for a mobile that can open one, then the entity's own.
pub fn create_context_menu(w: &mut World, from: MobileId, target: EntityId) -> ContextMenu {
    if deleted(w, target) {
        return ContextMenu::new(from, target, Vec::new());
    }
    let mut list = Vec::new();
    match target {
        EntityId::Mobile(m) => {
            let ty = w.mobile(m).map(|x| x.ty);
            if let Some(ty) = ty {
                if (ty.can_paperdoll_be_opened_by)(w, m, from) {
                    list.push(entries::paperdoll());
                }
                (ty.get_context_menu_entries)(w, m, from, &mut list);
            }
        }
        EntityId::Item(i) => {
            if let Some(ty) = w.item(i).map(|x| x.ty) {
                (ty.get_context_menu_entries)(w, i, from, &mut list);
            }
        }
    }
    ContextMenu::new(from, target, list)
}

fn ns_of(state: &NetState) -> NetStateId {
    NetStateId(state.id() as u32)
}

/// `ContextMenuResponse(state, reader)`: 0x15, the serial then the entry index.
pub fn context_menu_response(w: &mut World, state: &mut NetState, body: &[u8]) {
    let Some(from) = w.net_mobile(ns_of(state)) else {
        return;
    };
    // _menus.Remove(from, out menu): the menu is used up whatever happens next.
    let Some(menu) = w
        .resources
        .get_mut::<ContextMenuSystem>()
        .menus
        .shift_remove(&from)
    else {
        return;
    };
    if from != menu.from {
        return;
    }
    let (Some(serial), Some(index)) = (be_u32(body), be_u16(body.get(4..))) else {
        return;
    };

    let Some(entity) = EntityId::from_serial(Serial::new(serial)).filter(|&e| !deleted(w, e))
    else {
        return;
    };
    if entity != menu.target {
        return;
    }
    let from_ty = match w.mobile(from) {
        Some(m) => m.ty,
        None => return,
    };
    let can_see = match entity {
        EntityId::Item(i) => (from_ty.can_see_item)(w, from, i),
        EntityId::Mobile(m) => (from_ty.can_see_mobile)(w, from, m),
    };
    if !can_see {
        return;
    }

    let p = location(w, entity);
    let Some(e) = menu.entries.get(usize::from(index)) else {
        return;
    };

    let range = if e.range == -1 { 18 } else { e.range };
    if e.enabled && mb::in_range_point3d(w, from, p, range) {
        e.click(w, from, entity);
    }
}

/// `ContextMenuRequest(state, reader)`: 0x13, the serial of the entity to menu.
pub fn context_menu_request(w: &mut World, state: &mut NetState, body: &[u8]) {
    let ns = ns_of(state);
    let Some(from) = w.net_mobile(ns) else { return };
    let Some(serial) = be_u32(body) else { return };
    let Some(target) = EntityId::from_serial(Serial::new(serial)).filter(|&e| !deleted(w, e))
    else {
        return;
    };
    let from_ty = match w.mobile(from) {
        Some(m) => m.ty,
        None => return,
    };
    let from_map = w.mobile(from).and_then(|m| m.map);
    let can_see = match target {
        EntityId::Item(i) => (from_ty.can_see_item)(w, from, i),
        EntityId::Mobile(m) => (from_ty.can_see_mobile)(w, from, m),
    };
    if from_map != map_of(w, target) || !can_see {
        return;
    }

    let check_location = location(w, target);
    let from_loc = w.mobile(from).map(|m| m.location).unwrap_or_default();
    if !(muo_server::mobile::part1::combat::in_update_range(from_loc, check_location)
        && (from_ty.check_context_menu_display)(w, from, target))
    {
        return;
    }

    let mut c = create_context_menu(w, from, target);
    if c.entries.is_empty() {
        return;
    }

    if let EntityId::Item(item) = target
        && let Some(EntityId::Mobile(holder)) = item_ops::root_parent(w, item)
    {
        let holder_level = w.mobile(holder).map(|m| m.access_level);
        let from_level = w.mobile(from).map(|m| m.access_level);
        if holder != from && holder_level >= from_level {
            for entry in &mut c.entries {
                if !entry.non_local_use {
                    entry.enabled = false;
                }
            }
        }
    }

    send_display_context_menu(w, ns, &c);
    w.resources
        .get_mut::<ContextMenuSystem>()
        .menus
        .insert(from, c);
}

fn be_u32(b: &[u8]) -> Option<u32> {
    Some(u32::from_be_bytes(b.get(..4)?.try_into().ok()?))
}

fn be_u16(b: Option<&[u8]>) -> Option<u16> {
    Some(u16::from_be_bytes(b?.get(..2)?.try_into().ok()?))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_deleted_or_missing_entity_gets_an_empty_menu() {
        let mut w = World::new();
        let from = MobileId(Serial::new(1));
        let c = create_context_menu(&mut w, from, EntityId::Mobile(MobileId(Serial::new(2))));
        assert!(c.entries.is_empty());
    }

    #[test]
    fn readers() {
        assert_eq!(be_u32(&[0, 0, 1, 2, 9]), Some(0x102));
        assert_eq!(be_u32(&[0, 0, 1]), None);
        assert_eq!(be_u16(Some(&[1, 2])), Some(0x102));
        assert_eq!(be_u16(None), None);
    }
}
