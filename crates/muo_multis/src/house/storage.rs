//! `BaseHouse`: what a house holds and where it stands (`IsInside`, the region, lockdowns, secures, the moving
//! crate, relocation of the contents, the vendors) and its life cycle (`OnMapChange`, `OnLocationChange`,
//! `OnDelete`, `OnAfterDelete`).

use muo_content::factory::item_is;
use muo_content::items::house_deps::deps;
use muo_content::regions::house as house_region;
use muo_primitives::{Point2D, Point3D, Rectangle2D, TimeSpan};
use muo_server::World;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::{TotalType, ops};
use muo_server::map::{MapId, enumerate, fit};
use muo_server::mobile::AccessLevel;
use muo_server::packets::message::MessageType;
use muo_server::region;
use muo_server::timer;

use super::access::{
    has_secure_access, is_co_owner, is_friend, is_inside_item, is_owner, msg, msg_args,
};
use super::{
    HouseData, RelocatedEntity, SecureAccessResult, SecureInfo, SecureLevel, aos_max_lockdowns,
    aos_max_secures, d, data, data_mut, deleted, is_active, is_aos_rules, location, map_of,
    new_vendor_system, statics,
};

fn is_foundation(w: &World, house: ItemId) -> bool {
    w.item(house)
        .is_some_and(|i| muo_content::items::is_named(i.ty, "Server.Multis.HouseFoundation"))
}

/// `IsInside(Point3D p, int height)` (virtual): the base answer, then what the class adds.
pub fn is_inside_point(w: &World, house: ItemId, p: Point3D, height: i32) -> bool {
    inside_base(w, house, p, height)
        || super::class(w, house)
            .is_inside_extra
            .is_some_and(|f| f(w, house, p))
}

/// `BaseHouse.IsInside(p, height)`.
fn inside_base(w: &World, house: ItemId, p: Point3D, height: i32) -> bool {
    let Some(it) = w.item(house) else {
        return false;
    };
    if it.deleted() {
        return false;
    }
    let mcl = super::components(w, house);
    let x = p.x - it.location.x - mcl.min().x;
    let y = p.y - it.location.y - mcl.min().y;
    if x < 0 || x >= mcl.width() || y < 0 || y >= mcl.height() {
        return false;
    }
    if is_foundation(w, house) && y < mcl.height() - 1 && p.z >= it.location.z {
        return true;
    }
    let Some(tiles) = mcl.tiles.get(x as usize).and_then(|c| c.get(y as usize)) else {
        return false;
    };
    for tile in tiles {
        let id = usize::from(tile.id) & muo_data::TileData::MAX_ITEM_VALUE;
        let data = w.maps.tile_data.item(id);
        // Slanted roofs do not count; they overhang blocking south and east sides of the multi
        if data.roof() {
            continue;
        }
        // Signs and signposts are not considered part of the multi
        if (0xB95..=0xC0E).contains(&id) || (0xC43..=0xC44).contains(&id) {
            continue;
        }
        let tile_z = i32::from(tile.z) + it.location.z;
        if p.z == tile_z || p.z + height > tile_z {
            return true;
        }
    }
    false
}

// ---- region ----

/// `UpdateRegion()`.
pub fn update_region(w: &mut World, house: ItemId) {
    if let Some(r) = data(w, house).and_then(|h| h.region) {
        region::unregister(w, r);
    }
    if map_of(w, house).is_some() {
        let r = house_region::new_house_region(w, house);
        if let Some(r) = r {
            region::register(w, r);
        }
        if let Some(h) = data_mut(w, house) {
            h.region = r;
        }
    } else if let Some(h) = data_mut(w, house) {
        h.region = None;
    }
}

fn sector_bounds(w: &World, house: ItemId) -> Rectangle2D {
    let mcl = super::components(w, house);
    let l = location(w, house);
    Rectangle2D::new(
        l.x + mcl.min().x,
        l.y + mcl.min().y,
        mcl.width(),
        mcl.height(),
    )
}

fn invalidate_spawn_sectors(w: &mut World, house: ItemId) {
    if let Some(map) = map_of(w, house).filter(|&m| m != MapId::INTERNAL) {
        let bounds = sector_bounds(w, house);
        if let Some(f) = deps(w).and_then(|x| x.invalidate_spawn_sectors) {
            f(w, map, bounds);
        }
    }
}

/// `OnMapChange()`.
pub fn on_map_change(w: &mut World, house: ItemId) {
    (muo_server::item::multi::BASE_MULTI.on_map_change)(w, house);
    if !data(w, house).is_some_and(|h| h.initialized) {
        return;
    }
    update_region(w, house);
    invalidate_spawn_sectors(w, house); // Invalidate spawn position cache for affected sectors

    let map = map_of(w, house);
    let h = d(w, house);
    if let Some(s) = h.sign.filter(|&s| !deleted(w, s)) {
        ops::set_map(w, s, map);
    }
    for door in h.doors {
        ops::set_map(w, door, map);
    }
    for e in house_entities(w, house) {
        match e {
            EntityId::Item(i) => ops::set_map(w, i, map),
            EntityId::Mobile(m) => muo_server::mobile::part2::deps::set_map(w, m, map),
        }
    }
}

/// `OnLocationChange(oldLocation)`.
pub fn on_location_change(w: &mut World, house: ItemId, old: Point3D) {
    (muo_server::item::multi::BASE_MULTI.on_location_change)(w, house, old);
    if !data(w, house).is_some_and(|h| h.initialized) {
        return;
    }
    let l = location(w, house);
    let (x, y, z) = (l.x - old.x, l.y - old.y, l.z - old.z);
    let h = d(w, house);

    if let Some(s) = h.sign.filter(|&s| !deleted(w, s)) {
        let p = location(w, s);
        ops::set_location(w, s, Point3D::new(p.x + x, p.y + y, p.z + z));
    }
    update_region(w, house);
    for door in h.doors {
        if !deleted(w, door) {
            let p = location(w, door);
            ops::set_location(w, door, Point3D::new(p.x + x, p.y + y, p.z + z));
        }
    }
    for e in house_entities(w, house) {
        match e {
            EntityId::Item(i) => {
                let p = location(w, i);
                ops::set_location(w, i, Point3D::new(p.x + x, p.y + y, p.z + z));
            }
            EntityId::Mobile(m) => {
                let p = w.mobile(m).map_or(Point3D::ZERO, |m| m.location);
                muo_server::mobile::part1::lifecycle::set_location(
                    w,
                    m,
                    Point3D::new(p.x + x, p.y + y, p.z + z),
                );
            }
        }
    }
}

// ---- contents ----

fn item_on_ground(w: &World, i: ItemId) -> bool {
    w.item(i)
        .is_some_and(|x| x.parent.is_none() && x.map != Some(MapId::INTERNAL))
}

/// `GetHouseEntities()`.
pub fn house_entities(w: &mut World, house: ItemId) -> Vec<EntityId> {
    let h = d(w, house);
    let mut list = Vec::new();
    if let Some(c) = h.moving_crate {
        crate::moving_crate::hide(w, c);
    }
    if let Some(t) = h
        .trash
        .filter(|&t| w.item(t).is_some_and(|x| x.map != Some(MapId::INTERNAL)))
    {
        list.push(EntityId::Item(t));
    }
    for &i in &h.lock_downs {
        if item_on_ground(w, i) {
            list.push(EntityId::Item(i));
        }
    }
    for &i in &h.vendor_rental_contracts {
        if item_on_ground(w, i) {
            list.push(EntityId::Item(i));
        }
    }
    for s in &h.secures {
        if item_on_ground(w, s.item) {
            list.push(EntityId::Item(s.item));
        }
    }
    for &i in &h.addons {
        if item_on_ground(w, i) {
            list.push(EntityId::Item(i));
        }
    }
    for &m in &h.player_vendors {
        if let Some(f) = deps(w).and_then(|x| x.vendor_return) {
            f(w, m);
        }
        if w.mobile(m).is_some_and(|x| x.map != Some(MapId::INTERNAL)) {
            list.push(EntityId::Mobile(m));
        }
    }
    for &m in &h.player_barkeepers {
        if w.mobile(m).is_some_and(|x| x.map != Some(MapId::INTERNAL)) {
            list.push(EntityId::Mobile(m));
        }
    }
    list
}

/// `RelocateEntities()`.
pub fn relocate_entities(w: &mut World, house: ItemId) {
    let l = location(w, house);
    for entity in house_entities(w, house) {
        let p = match entity {
            EntityId::Item(i) => location(w, i),
            EntityId::Mobile(m) => w.mobile(m).map_or(Point3D::ZERO, |m| m.location),
        };
        let rel = Point3D::new(p.x - l.x, p.y - l.y, p.z - l.z);
        if let Some(h) = data_mut(w, house) {
            h.relocated_entities.push(RelocatedEntity {
                entity,
                relative_location: rel,
            });
        }
        match entity {
            EntityId::Item(i) => ops::internalize(w, i),
            EntityId::Mobile(m) => muo_server::mobile::part2::items::internalize(w, m),
        }
    }
}

fn delete_item(w: &mut World, id: ItemId) {
    if let Some(ty) = w.item(id).map(|i| i.ty) {
        (ty.delete)(w, id);
    }
}

/// The `IAddon`/deed handling of an addon that is taken down: its deed (with the resource and the retained hue),
/// the addon deleted. `None` for an addon without a deed.
fn take_addon_deed(w: &mut World, addon: ItemId) -> Option<ItemId> {
    let deed = deps(w)
        .and_then(|x| x.addon_deed)
        .and_then(|f| f(w, addon))
        .flatten();
    let retain_hue = deps(w)
        .and_then(|x| x.addon_retain_deed_hue)
        .and_then(|f| f(w, addon));
    if let Some(deed) = deed {
        if let Some(f) = deps(w).and_then(|x| x.addon_prepare_deed) {
            f(w, addon, deed);
        }
        if let Some(hue) = retain_hue {
            muo_server::item::base::set_hue(w, deed, hue);
        }
    }
    deed
}

/// `RestoreRelocatedEntities()`.
pub fn restore_relocated_entities(w: &mut World, house: ItemId) {
    let list = d(w, house).relocated_entities;
    let l = location(w, house);
    let map = map_of(w, house);
    for r in list {
        let rel = r.relative_location;
        let loc = Point3D::new(rel.x + l.x, rel.y + l.y, rel.z + l.z);
        match r.entity {
            EntityId::Item(item) => restore_item(w, house, item, loc, map),
            EntityId::Mobile(m) => {
                if w.mobile(m).is_some_and(|x| !x.deleted) {
                    let fits = map.is_some_and(|map| {
                        fit::can_fit(w, map, loc.x, loc.y, loc.z, 16, false, false, true)
                    });
                    if fits {
                        if let Some(map) = map {
                            muo_server::mobile::base::move_to_world(w, m, loc, map);
                        }
                    } else if let Some(h) = data_mut(w, house) {
                        h.internalized_vendors.push(m);
                    }
                }
            }
        }
    }
    if let Some(h) = data_mut(w, house) {
        h.relocated_entities.clear();
    }
}

fn restore_item(w: &mut World, house: ItemId, item: ItemId, loc: Point3D, map: Option<MapId>) {
    if !deleted(w, item) {
        let is_addon = deps(w)
            .and_then(|x| x.addon_deed)
            .and_then(|f| f(w, item))
            .is_some();
        let mut placed = false;
        if is_addon {
            let fits = deps(w)
                .and_then(|x| x.addon_could_fit)
                .is_some_and(|f| f(w, item, loc, map));
            if fits {
                ops::move_to_world_map(w, item, loc, map);
                placed = true;
            }
        } else {
            let (height, require_surface) = if item_is(w, item, "Server.Items.VendorRentalContract")
            {
                (16, true)
            } else {
                (i32::from(ops::item_data(w, item).height), false)
            };
            let fits = map.is_some_and(|m| {
                fit::can_fit(
                    w,
                    m,
                    loc.x,
                    loc.y,
                    loc.z,
                    height,
                    false,
                    false,
                    require_surface,
                )
            });
            if fits {
                ops::move_to_world_map(w, item, loc, map);
                placed = true;
            }
        }
        if !placed {
            // The item can't fit
            if item_is(w, item, "Server.Items.TrashBarrel") {
                delete_item(w, item); // Trash barrels don't go to the moving crate
            } else {
                set_lockdown(w, house, item, false, false);
                ops::set_secure(w, item, false);
                ops::set_movable(w, item, true);
                let mut relocate = Some(item);
                if let Some(f) = deps(w).and_then(|x| x.strong_box_owner)
                    && f(w, item).is_some()
                    && let Some(c) = deps(w).and_then(|x| x.strong_box_convert)
                {
                    relocate = Some(c(w, item));
                }
                if is_addon {
                    relocate = take_addon_deed(w, item);
                    delete_item(w, item);
                }
                if let Some(r) = relocate {
                    drop_to_moving_crate(w, house, r);
                }
            }
        }
    }
    if let Some(h) = data_mut(w, house) {
        if h.trash == Some(item) {
            h.trash = None;
        }
        h.lock_downs.retain(|&i| i != item);
        h.vendor_rental_contracts.retain(|&i| i != item);
        h.addons.retain(|&i| i != item);
        h.secures.retain(|s| s.item != item);
    }
}

/// `DropToMovingCrate(item)`.
pub fn drop_to_moving_crate(w: &mut World, house: ItemId, item: ItemId) {
    let c = match data(w, house).and_then(|h| h.moving_crate) {
        Some(c) => c,
        None => {
            let c = crate::moving_crate::create(w, house);
            if let Some(h) = data_mut(w, house) {
                h.moving_crate = Some(c);
            }
            c
        }
    };
    crate::moving_crate::drop_item(w, c, item);
}

/// `MoveAllToCrate()`.
pub fn move_all_to_crate(w: &mut World, house: ItemId) {
    let h = d(w, house);
    if let Some(h) = data_mut(w, house) {
        h.relocated_entities.clear();
    }
    if let Some(c) = h.moving_crate {
        crate::moving_crate::hide(w, c);
    }
    if let Some(t) = h.trash {
        delete_item(w, t);
        if let Some(h) = data_mut(w, house) {
            h.trash = None;
        }
    }

    for &item in &h.lock_downs {
        if !deleted(w, item) {
            release_to_crate(w, house, item);
        }
    }
    if let Some(h) = data_mut(w, house) {
        h.lock_downs.clear();
    }
    for &item in &h.vendor_rental_contracts {
        if !deleted(w, item) {
            release_to_crate(w, house, item);
        }
    }
    if let Some(h) = data_mut(w, house) {
        h.vendor_rental_contracts.clear();
    }
    for info in &h.secures {
        let mut item = info.item;
        if !deleted(w, item) {
            if deps(w)
                .and_then(|x| x.strong_box_owner)
                .and_then(|f| f(w, item))
                .is_some()
                && let Some(c) = deps(w).and_then(|x| x.strong_box_convert)
            {
                item = c(w, item);
            }
            release_to_crate(w, house, item);
        }
    }
    if let Some(h) = data_mut(w, house) {
        h.secures.clear();
    }

    for &addon in &h.addons {
        if !deleted(w, addon) {
            match take_addon_deed(w, addon) {
                Some(deed) => {
                    delete_item(w, addon);
                    drop_to_moving_crate(w, house, deed);
                }
                None => drop_to_moving_crate(w, house, addon),
            }
        }
    }
    if let Some(h) = data_mut(w, house) {
        h.addons.clear();
    }

    for &m in &h.player_vendors {
        if let Some(f) = deps(w).and_then(|x| x.vendor_return) {
            f(w, m);
        }
        muo_server::mobile::part2::items::internalize(w, m);
        if let Some(h) = data_mut(w, house) {
            h.internalized_vendors.push(m);
        }
    }
    for &m in &h.player_barkeepers {
        muo_server::mobile::part2::items::internalize(w, m);
        if let Some(h) = data_mut(w, house) {
            h.internalized_vendors.push(m);
        }
    }
}

fn release_to_crate(w: &mut World, house: ItemId, item: ItemId) {
    ops::set_locked_down(w, item, false);
    ops::set_secure(w, item, false);
    ops::set_movable(w, item, true);
    if w.item(item).is_some_and(|i| i.parent.is_none()) {
        drop_to_moving_crate(w, house, item);
    }
}

/// `GetItems()`: the movable items inside the house.
pub fn get_items(w: &World, house: ItemId) -> Vec<ItemId> {
    let map = map_of(w, house);
    if map.is_none() || map == Some(MapId::INTERNAL) {
        return Vec::new();
    }
    let mcl = super::components(w, house);
    let l = location(w, house);
    let rect = Rectangle2D::from_points(
        Point2D::new(l.x + mcl.min().x, l.y + mcl.min().y),
        Point2D::new(l.x + mcl.max().x + 1, l.y + mcl.max().y + 1),
    );
    enumerate::items_in_bounds(w, map, rect, false)
        .filter(|&i| w.item(i).is_some_and(|x| x.movable()) && is_inside_item(w, house, i))
        .collect()
}

/// `GetMobilesPooled()`: the mobiles in the house region that are inside the footprint.
pub fn get_mobiles(w: &World, house: ItemId) -> Vec<MobileId> {
    let map = map_of(w, house);
    if map.is_none() || map == Some(MapId::INTERNAL) {
        return Vec::new();
    }
    let Some(r) = super::region(w, house) else {
        return Vec::new();
    };
    region::get_mobiles(w, r)
        .into_iter()
        .filter(|&m| super::access::is_inside_mobile(w, house, m))
        .collect()
}

// ---- limits ----

/// `GetLockdowns()`.
pub fn get_lockdowns(w: &World, house: ItemId) -> i32 {
    let mut count = 0;
    for &i in &d(w, house).lock_downs {
        if !ops::is_container(w, i) {
            count += ops::get_total(w, i, TotalType::Items);
        }
        count += 1;
    }
    count
}

/// `LockDownCount`.
pub fn lock_down_count(w: &World, house: ItemId) -> i32 {
    let mut count = get_lockdowns(w, house);
    for s in &d(w, house).secures {
        if deleted(w, s.item) {
            continue;
        }
        count += if item_is(w, s.item, "Server.Items.StrongBox") {
            1
        } else {
            125
        };
    }
    count
}

/// `SecureCount`.
pub fn secure_count(w: &World, house: ItemId) -> i32 {
    d(w, house)
        .secures
        .iter()
        .filter(|s| !deleted(w, s.item) && !item_is(w, s.item, "Server.Items.StrongBox"))
        .count() as i32
}

/// `GetAosCurLockdowns()`.
pub fn aos_cur_lockdowns(w: &World, house: ItemId) -> i32 {
    let h = d(w, house);
    let mut v = get_lockdowns(w, house);
    v += h.secures.len() as i32;
    if !new_vendor_system(w) {
        v += h.player_vendors.len() as i32 * 10;
    }
    v
}

/// `GetAosCurSecures(out fromSecures, out fromVendors, out fromLockdowns, out fromMovingCrate)`: the sum and the four parts.
pub fn aos_cur_secures_parts(w: &World, house: ItemId) -> (i32, i32, i32, i32, i32) {
    let h = d(w, house);
    let mut from_secures = 0;
    let mut from_vendors = 0;
    let mut from_crate = 0;
    for s in &h.secures {
        from_secures += ops::get_total(w, s.item, TotalType::Items);
    }
    let mut from_lockdowns = h.secures.len() as i32 + get_lockdowns(w, house);
    if !new_vendor_system(w) {
        for &v in &h.player_vendors {
            if let Some(f) = deps(w).and_then(|x| x.vendor_pack_items) {
                from_vendors += f(w, v);
            }
        }
    }
    if let Some(c) = h.moving_crate {
        from_crate += ops::get_total(w, c, TotalType::Items);
        for &i in &w.item(c).map(|x| x.items.clone()).unwrap_or_default() {
            if item_is(w, i, "Server.Multis.PackingBox") {
                from_crate -= 1;
            }
        }
    }
    from_lockdowns += 0;
    (
        from_secures + from_vendors + from_lockdowns + from_crate,
        from_secures,
        from_vendors,
        from_lockdowns,
        from_crate,
    )
}

/// `GetAosCurSecures(..)`'s sum.
pub fn aos_cur_secures(w: &World, house: ItemId) -> i32 {
    aos_cur_secures_parts(w, house).0
}

/// `CheckAosLockdowns(need)`.
pub fn check_aos_lockdowns(w: &World, house: ItemId, need: i32) -> bool {
    aos_cur_lockdowns(w, house) + need <= aos_max_lockdowns(w, house)
}

/// `CheckAosStorage(need)`.
pub fn check_aos_storage(w: &World, house: ItemId, need: i32) -> bool {
    aos_cur_secures(w, house) + need <= aos_max_secures(w, house)
}

/// `CanPlaceNewVendor()`.
pub fn can_place_new_vendor(w: &World, house: ItemId) -> bool {
    if !is_aos_rules(w, house) {
        return true;
    }
    if !new_vendor_system(w) {
        check_aos_lockdowns(w, house, 10)
    } else {
        let h = d(w, house);
        ((h.player_vendors.len() + h.vendor_rental_contracts.len()) as i32)
            < super::new_vendor_system_max_vendors(w, house)
    }
}

/// `CanPlaceNewBarkeep()`.
pub fn can_place_new_barkeep(w: &World, house: ItemId) -> bool {
    d(w, house).player_barkeepers.len() < super::MAXIMUM_BARKEEP_COUNT
}

// ---- lockdowns and secures ----

/// `HasLockedDownItem(check)`.
pub fn has_locked_down_item(w: &World, house: ItemId, check: ItemId) -> bool {
    let Some(h) = data(w, house) else {
        return false;
    };
    h.initialized && h.lock_downs.contains(&check) || h.vendor_rental_contracts.contains(&check)
}

/// `HasSecureItem(item)`.
pub fn has_secure_item(w: &World, house: ItemId, item: ItemId) -> bool {
    data(w, house).is_some_and(|h| h.secures.iter().any(|s| s.item == item))
}

fn is_addon_container(w: &World, i: ItemId) -> bool {
    item_is(w, i, "Server.Items.BaseAddonContainer")
}

/// `SetLockdown(i, locked, checkContains)`.
pub fn set_lockdown(w: &mut World, house: ItemId, i: ItemId, locked: bool, check_contains: bool) {
    if !data(w, house).is_some_and(|h| h.initialized) {
        return;
    }
    if is_addon_container(w, i) {
        ops::set_movable(w, i, false);
    } else {
        ops::set_movable(w, i, !locked);
    }
    ops::set_locked_down(w, i, locked);

    let contract = item_is(w, i, "Server.Items.VendorRentalContract");
    if let Some(h) = data_mut(w, house) {
        if locked {
            if contract {
                if !h.vendor_rental_contracts.contains(&i) {
                    h.vendor_rental_contracts.push(i);
                }
            } else if !check_contains || !h.lock_downs.contains(&i) {
                h.lock_downs.push(i);
            }
        } else {
            if contract && let Some(p) = h.vendor_rental_contracts.iter().position(|&x| x == i) {
                h.vendor_rental_contracts.remove(p);
            }
            if let Some(p) = h.lock_downs.iter().position(|&x| x == i) {
                h.lock_downs.remove(p);
            }
        }
    }
    if !locked {
        ops::set_last_moved(w, i);
    }
    if ops::is_container(w, i)
        && (!locked
            || !(item_is(w, i, "Server.Items.BaseBoard")
                || item_is(w, i, "Server.Items.Aquarium")
                || item_is(w, i, "Server.Items.FishBowl")))
    {
        let children = w.item(i).map(|x| x.items.clone()).unwrap_or_default();
        for c in children {
            set_lockdown(w, house, c, locked, check_contains);
        }
    }
}

fn root_item(w: &World, item: ItemId) -> Option<ItemId> {
    match ops::root_parent(w, item) {
        Some(EntityId::Item(i)) => Some(i),
        _ => None,
    }
}

/// `LockDown(m, item, checkIsInside)`.
#[allow(clippy::if_same_then_else)] // the chain of checks repeats a message
pub fn lock_down(
    w: &mut World,
    house: ItemId,
    m: MobileId,
    item: ItemId,
    check_is_inside: bool,
) -> bool {
    if !is_co_owner(w, house, m) || !is_active(w, house) {
        return false;
    }
    let movable = w.item(item).is_some_and(|i| i.movable());
    if is_addon_container(w, item) || (movable && !has_secure_item(w, house, item)) {
        let amt = 1 + ops::get_total(w, item, TotalType::Items);
        let root = ops::root_parent(w, item);
        let root_item = root_item(w, item);
        let parent_item = match w.item(item).and_then(|i| i.parent) {
            Some(EntityId::Item(p)) => Some(p),
            _ => None,
        };

        if check_is_inside && matches!(root, Some(EntityId::Mobile(_))) {
            msg(w, m, 1005525); // That is not in your house
        } else if check_is_inside && {
            let loc = ops::get_world_location(w, item);
            let h = i32::from(ops::item_data(w, item).height);
            !is_inside_point(w, house, loc, h)
        } {
            msg(w, m, 1005525); // That is not in your house
        } else if muo_content::player_mobile::hooks::ethic_of_item(w, item).is_some() {
            msg(w, m, 1005377); // You cannot lock that down
        } else if root_item.is_some_and(|r| has_secure_item(w, house, r)) {
            msg(w, m, 501737); // You need not lock down items in a secure container.
        } else if parent_item.is_some_and(|p| !has_locked_down_item(w, house, p)) {
            msg(w, m, 501736); // You must lockdown the container first!
        } else if !item_is(w, item, "Server.Items.VendorRentalContract")
            && (if is_aos_rules(w, house) {
                !check_aos_lockdowns(w, house, amt) || !check_aos_storage(w, house, amt)
            } else {
                lock_down_count(w, house) + amt > d(w, house).max_lock_downs
            })
        {
            msg(w, m, 1005379); // That would exceed the maximum lock down limit for this house
        } else {
            set_lockdown(w, house, item, true, false);
            return true;
        }
    } else if d(w, house).lock_downs.contains(&item) {
        mm_local_overhead(w, m, 0x3E9, 1005526); // That is already locked down
        return true;
    } else if item_is(w, item, "Server.Multis.HouseSign") || item_is(w, item, "Server.Items.Static")
    {
        mm_local_overhead(w, m, 0x3E9, 1005526); // This is already locked down.
    } else {
        msg(w, m, 1005377); // You cannot lock that down
    }
    false
}

/// `from.LocalOverheadMessage(MessageType.Regular, hue, number)`.
pub fn mm_local_overhead(w: &mut World, m: MobileId, hue: i32, number: i32) {
    muo_server::mobile::part2::messages::local_overhead_message_number(
        w,
        m,
        muo_primitives::MessageType::REGULAR,
        hue,
        number,
        "",
    );
}

fn label_message(w: &mut World, item: ItemId, hue: i32, number: i32) {
    muo_server::item::messages::public_overhead_message_number(
        w,
        item,
        MessageType::LABEL,
        hue,
        number,
        "",
    );
}

/// `Release(m, item)`.
pub fn release(w: &mut World, house: ItemId, m: MobileId, item: ItemId) {
    if !is_co_owner(w, house, m) || !is_active(w, house) {
        return;
    }
    if has_locked_down_item(w, house, item) {
        label_message(w, item, 0x3B2, 501657); // [no longer locked down]
        set_lockdown(w, house, item, false, false);
        if item_is(w, item, "Server.Items.RewardBrazier")
            && let Some(f) = deps(w).and_then(|x| x.brazier_turn_off)
        {
            f(w, item);
        }
    } else if has_secure_item(w, house, item) {
        release_secure(w, house, m, item);
    } else {
        mm_local_overhead(w, m, 0x3E9, 1010416); // This is not locked down or secured.
    }
}

/// `AddSecure(m, item)`.
pub fn add_secure(w: &mut World, house: ItemId, m: MobileId, item: ItemId) {
    if !is_owner(w, house, m) || !is_active(w, house) {
        return;
    }
    if !is_inside_item(w, house, item) {
        msg(w, m, 1005525); // That is not in your house
    } else if has_locked_down_item(w, house, item) {
        msg(w, m, 1010550); // This is already locked down and cannot be secured.
    } else if !ops::is_container(w, item) {
        lock_down(w, house, m, item, true);
    } else {
        let info = d(w, house).secures.iter().find(|s| s.item == item).copied();
        let movable = w.item(item).is_some_and(|i| i.movable());
        if let Some(info) = info {
            crate::secure_gump::send(w, m, house, info.item);
        } else if w.item(item).is_some_and(|i| i.parent.is_some()) {
            msg(w, m, 1010423); // You cannot secure this, place it on the ground first.
        } else if !is_addon_container(w, item) && !movable {
            msg(w, m, 1010424); // You cannot secure this.
        } else if !is_aos_rules(w, house) && secure_count(w, house) >= d(w, house).max_secures {
            // The maximum number of secure items has been reached :
            let n = d(w, house).max_secures.to_string();
            muo_server::mobile::part2::messages::send_localized_message_affix(
                w, m, 1008142, true, &n, "", 0x3B2,
            );
        } else if if is_aos_rules(w, house) {
            !check_aos_lockdowns(w, house, 1)
        } else {
            lock_down_count(w, house) + 125 >= d(w, house).max_lock_downs
        } {
            msg(w, m, 1005379); // That would exceed the maximum lock down limit for this house
        } else if is_aos_rules(w, house)
            && !check_aos_storage(w, house, ops::get_total(w, item, TotalType::Items))
        {
            msg(w, m, 1061839); // This action would exceed the secure storage limit of the house.
        } else {
            ops::set_locked_down(w, item, false);
            ops::set_secure(w, item, true);
            if let Some(h) = data_mut(w, house) {
                h.secures.push(SecureInfo {
                    item,
                    level: SecureLevel::Owner,
                });
                h.lock_downs.retain(|&i| i != item);
            }
            ops::set_movable(w, item, false);
            crate::secure_gump::send(w, m, house, item);
        }
    }
}

/// `ReleaseSecure(m, item)`.
pub fn release_secure(w: &mut World, house: ItemId, m: MobileId, item: ItemId) {
    if !is_owner(w, house, m) || item_is(w, item, "Server.Items.StrongBox") || !is_active(w, house)
    {
        return;
    }
    let secures = d(w, house).secures;
    for (i, info) in secures.iter().enumerate() {
        if info.item == item && has_secure_access(w, house, m, info.level) {
            ops::set_locked_down(w, item, false);
            ops::set_secure(w, item, false);
            ops::set_movable(w, item, !is_addon_container(w, item));
            ops::set_last_moved(w, item);
            label_message(w, item, 0x3B2, 501656); // [no longer secure]
            if let Some(h) = data_mut(w, house) {
                h.secures.remove(i);
            }
            return;
        }
    }
    msg(w, m, 501717); // This isn't secure...
}

/// `AddStrongBox(from)`.
pub fn add_strong_box(w: &mut World, house: ItemId, from: MobileId) {
    if !is_co_owner(w, house, from) || !is_active(w, house) {
        return;
    }
    if Some(from) == super::owner(w, house) {
        msg(w, from, 502109); // Owners don't get a strong box
        return;
    }
    let too_many = if is_aos_rules(w, house) {
        !check_aos_lockdowns(w, house, 1)
    } else {
        lock_down_count(w, house) + 1 > d(w, house).max_lock_downs
    };
    if too_many {
        msg(w, from, 1005379); // That would exceed the maximum lock down limit for this house
        return;
    }
    let owner_of = deps(w).and_then(|x| x.strong_box_owner);
    for s in d(w, house).secures {
        if !deleted(w, s.item)
            && owner_of
                .and_then(|f| f(w, s.item))
                .is_some_and(|o| o == Some(from))
        {
            msg(w, from, 502112); // You already have a strong box
            return;
        }
    }
    if super::doors::door_near(w, house, from) {
        msg(w, from, 502113); // You cannot place a strongbox near a door or near steps.
        return;
    }
    let Some(sb) = deps(w)
        .and_then(|x| x.make_strong_box)
        .and_then(|f| f(w, from, house))
    else {
        return;
    };
    ops::set_movable(w, sb, false);
    ops::set_locked_down(w, sb, false);
    ops::set_secure(w, sb, true);
    if let Some(h) = data_mut(w, house) {
        h.secures.push(SecureInfo {
            item: sb,
            level: SecureLevel::CoOwners,
        });
    }
    let (loc, map) = w
        .mobile(from)
        .map(|m| (m.location, m.map))
        .unwrap_or((Point3D::ZERO, None));
    ops::move_to_world_map(w, sb, loc, map);
}

/// `CheckSecureAccess(m, item)`.
pub fn check_secure_access(
    w: &World,
    house: ItemId,
    m: MobileId,
    item: ItemId,
) -> SecureAccessResult {
    if !ops::is_container(w, item) {
        return SecureAccessResult::Insecure;
    }
    for info in &d(w, house).secures {
        if info.item == item {
            return if has_secure_access(w, house, m, info.level) {
                SecureAccessResult::Accessible
            } else {
                SecureAccessResult::Inaccessible
            };
        }
    }
    SecureAccessResult::Insecure
}

/// `CheckAccessibility(item, from)`.
pub fn check_accessibility(w: &World, house: ItemId, item: ItemId, from: MobileId) -> bool {
    match check_secure_access(w, house, from, item) {
        SecureAccessResult::Accessible => return true,
        SecureAccessResult::Inaccessible => return false,
        SecureAccessResult::Insecure => {}
    }
    if !has_locked_down_item(w, house, item) {
        return true;
    }
    if w.mobile(from)
        .is_some_and(|m| m.access_level >= AccessLevel::GameMaster)
    {
        return true;
    }
    if item_is(w, item, "Server.Items.Runebook") {
        return true;
    }
    if let Some(level) = muo_content::items::house_deps::securable_level(w, item) {
        return has_secure_access(w, house, from, SecureLevel::from_u8(level));
    }
    if ops::is_container(w, item) {
        return is_co_owner(w, house, from);
    }
    if w.item(item).is_some_and(|i| i.stackable()) {
        return true;
    }
    if item_is(w, item, "Server.Items.BaseLight") || item_is(w, item, "Server.Items.PotionKeg") {
        return is_friend(w, house, from);
    }
    [
        "Dices",
        "RecallRune",
        "TreasureMap",
        "Clock",
        "BaseInstrument",
        "Dyes",
        "VendorRentalContract",
        "RewardBrazier",
    ]
    .iter()
    .any(|n| item_is(w, item, &format!("Server.Items.{n}")))
}

/// `BaseHouse.CheckAccessible(m, item)`.
pub fn check_accessible(w: &World, m: MobileId, item: ItemId) -> bool {
    if w.mobile(m)
        .is_some_and(|x| x.access_level >= AccessLevel::GameMaster)
    {
        return true; // Staff can access anything
    }
    let Some(house) = super::access::find_house_at_item(w, item) else {
        return true;
    };
    match check_secure_access(w, house, m, item) {
        SecureAccessResult::Accessible => return true,
        SecureAccessResult::Inaccessible => return false,
        SecureAccessResult::Insecure => {}
    }
    if has_locked_down_item(w, house, item) {
        return is_co_owner(w, house, m) && ops::is_container(w, item);
    }
    true
}

/// `BaseHouse.CheckHold(m, cont, item, message, checkItems, plusItems, plusWeight)`.
pub fn check_hold(
    w: &mut World,
    m: Option<MobileId>,
    cont: ItemId,
    item: ItemId,
    message: bool,
    plus_items: i32,
) -> bool {
    let Some(house) = super::access::find_house_at_item(w, cont) else {
        return true;
    };
    if !is_aos_rules(w, house) {
        return true;
    }
    if has_secure_item(w, house, cont)
        && !check_aos_storage(
            w,
            house,
            1 + ops::get_total(w, item, TotalType::Items) + plus_items,
        )
    {
        if message && let Some(m) = m {
            msg(w, m, 1061839); // This action would exceed the secure storage limit of the house.
        }
        return false;
    }
    true
}

/// `BaseHouse.CheckLockedDown(item)`.
pub fn check_locked_down(w: &World, item: ItemId) -> bool {
    super::access::find_house_at_item(w, item).is_some_and(|h| has_locked_down_item(w, h, item))
}

/// `BaseHouse.CheckSecured(item)`.
pub fn check_secured(w: &World, item: ItemId) -> bool {
    super::access::find_house_at_item(w, item).is_some_and(|h| has_secure_item(w, h, item))
}

/// `BaseHouse.CheckLockedDownOrSecured(item)`.
pub fn check_locked_down_or_secured(w: &World, item: ItemId) -> bool {
    super::access::find_house_at_item(w, item)
        .is_some_and(|h| has_secure_item(w, h, item) || has_locked_down_item(w, h, item))
}

/// `FixLockdowns_Sandbox()`.
pub fn fix_lockdowns(w: &mut World, house: ItemId) {
    let list = d(w, house).lock_downs;
    for item in list {
        if ops::is_container(w, item) {
            set_lockdown(w, house, item, true, true);
        }
    }
}

// ---- vendors ----

/// `KillVendors()`.
pub fn kill_vendors(w: &mut World, house: ItemId) {
    let h = d(w, house);
    for v in h.player_vendors {
        if let Some(f) = deps(w).and_then(|x| x.vendor_destroy) {
            f(w, v);
        }
    }
    for b in h.player_barkeepers {
        muo_server::mobile::base::delete(w, b);
    }
}

/// `FindGuildstone()`.
pub fn find_guildstone(w: &World, house: ItemId) -> Option<ItemId> {
    let map = map_of(w, house)?;
    let bounds = sector_bounds(w, house);
    deps(w)
        .and_then(|x| x.find_guildstone)
        .and_then(|f| f(w, map, bounds))
}

// ---- deletion ----

/// `OnDelete()`.
pub fn on_delete(w: &mut World, house: ItemId) {
    restore_relocated_entities(w, house);
    invalidate_spawn_sectors(w, house); // Invalidate spawn position cache for affected sectors before deletion

    // FixColumnTimer
    if let Some(map) = map_of(w, house) {
        let mcl = super::components(w, house);
        let l = location(w, house);
        let (sx, sy) = (l.x + mcl.min().x, l.y + mcl.min().y);
        let (ex, ey) = (l.x + mcl.max().x, l.y + mcl.max().y);
        timer::start(w, TimeSpan::ZERO, TimeSpan::ZERO, 1, move |w| {
            for x in sx..=ex {
                for y in sy..=ey {
                    muo_server::map::surface::fix_column(w, map, x, y);
                }
            }
        });
    }
    (muo_server::item::multi::BASE_MULTI.on_delete)(w, house);
}

fn free_item(w: &mut World, i: ItemId) {
    ops::set_locked_down(w, i, false);
    ops::set_secure(w, i, false);
    ops::set_movable(w, i, true);
    ops::set_last_moved(w, i);
}

/// `OnAfterDelete()`.
pub fn on_after_delete(w: &mut World, house: ItemId) {
    (muo_server::item::multi::BASE_MULTI.on_after_delete)(w, house);
    let Some(h) = data(w, house).cloned() else {
        return;
    };

    if let Some(o) = h.owner {
        let t = &mut statics(w).table;
        if let Some(list) = t.get_mut(&o)
            && let Some(p) = list.iter().position(|&x| x == house)
        {
            list.remove(p);
            if list.is_empty() {
                t.shift_remove(&o);
            }
        }
    }
    if let Some(r) = h.region {
        region::unregister(w, r);
        if let Some(h) = data_mut(w, house) {
            h.region = None;
        }
    }
    if let Some(s) = h.sign {
        delete_item(w, s);
    }
    if let Some(t) = h.trash {
        delete_item(w, t);
    }
    for &door in &h.doors {
        delete_item(w, door);
    }
    if let Some(x) = data_mut(w, house) {
        x.doors.clear();
    }
    for &i in &h.lock_downs {
        if w.item(i).is_some() {
            free_item(w, i);
        }
    }
    if let Some(x) = data_mut(w, house) {
        x.lock_downs.clear();
    }
    for &i in &h.vendor_rental_contracts {
        if w.item(i).is_some() {
            free_item(w, i);
        }
    }
    if let Some(x) = data_mut(w, house) {
        x.vendor_rental_contracts.clear();
    }
    for info in &h.secures {
        if item_is(w, info.item, "Server.Items.StrongBox") {
            if let Some(f) = deps(w).and_then(|x| x.strong_box_destroy) {
                f(w, info.item);
            }
        } else if w.item(info.item).is_some() {
            free_item(w, info.item);
        }
    }
    if let Some(x) = data_mut(w, house) {
        x.secures.clear();
    }
    for &item in &h.addons {
        if w.item(item).is_none() {
            continue;
        }
        if !deleted(w, item) {
            let is_addon = deps(w)
                .and_then(|x| x.addon_deed)
                .and_then(|f| f(w, item))
                .is_some();
            if is_addon {
                let deed = deps(w)
                    .and_then(|x| x.addon_deed)
                    .and_then(|f| f(w, item))
                    .flatten();
                let retain = deps(w)
                    .and_then(|x| x.addon_retain_deed_hue)
                    .and_then(|f| f(w, item));
                if let Some(deed) = deed {
                    if let Some(hue) = retain {
                        muo_server::item::base::set_hue(w, deed, hue);
                    }
                    let (loc, map) = w
                        .item(item)
                        .map(|i| (i.location, i.map))
                        .unwrap_or((Point3D::ZERO, None));
                    ops::move_to_world_map(w, deed, loc, map);
                }
            }
        }
        delete_item(w, item);
    }
    if let Some(x) = data_mut(w, house) {
        x.addons.clear();
    }
    let invs: Vec<u64> = h.vendor_inventories.iter().map(|v| v.id).collect();
    for id in invs {
        crate::vendor_inventory::delete(w, house, id);
    }
    if let Some(c) = h.moving_crate {
        delete_item(w, c);
    }
    kill_vendors(w, house);
    statics(w).all_houses.retain(|&x| x != house);
}

/// Helper for gumps: owner-only check with a message.
pub fn owner_or_msg(w: &mut World, house: ItemId, m: MobileId) -> bool {
    if is_owner(w, house, m) {
        true
    } else {
        msg_args(w, m, 1061637, "");
        false
    }
}

/// `HouseData` of a house whose record is being read.
pub fn blank(_: &HouseData) {}
