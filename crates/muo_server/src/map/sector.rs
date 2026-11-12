//! `Map.Sector` and the sector bookkeeping of the map (`OnEnter`, `OnLeave`, `OnMove`,
//! `ActivateSectors`, multi registration).

use muo_primitives::Point3D;

use super::{MapId, SECTOR_ACTIVE_RANGE, SECTOR_SHIFT};
use crate::World;
use crate::ids::{ItemId, MobileId};
use crate::net_state::NetStateId;
use crate::region::RegionId;

/// `Map.Sector`: the items, mobiles, clients, multis and regions of one 16x16 tile square.
pub struct Sector {
    pub x: i32,
    pub y: i32,
    owner: MapId,
    active: bool,
    /// `Items`, in the order they entered.
    pub items: Vec<ItemId>,
    /// `Mobiles`.
    pub mobiles: Vec<MobileId>,
    /// `Clients`: the connection and the mobile it controlled when it entered (`NetState.Mobile`).
    pub clients: Vec<(NetStateId, MobileId)>,
    /// `Multis`.
    pub multis: Vec<ItemId>,
    /// `MultisVersion`: counts multi enters and leaves.
    pub multis_version: i32,
    /// `Regions`, sorted by `Region.CompareTo`.
    pub regions: Vec<RegionId>,
}

impl Sector {
    pub const fn empty(owner: MapId) -> Sector {
        Sector {
            x: 0,
            y: 0,
            owner,
            active: false,
            items: Vec::new(),
            mobiles: Vec::new(),
            clients: Vec::new(),
            multis: Vec::new(),
            multis_version: 0,
            regions: Vec::new(),
        }
    }

    pub const fn new(x: i32, y: i32, owner: MapId) -> Sector {
        let mut s = Sector::empty(owner);
        s.x = x;
        s.y = y;
        s
    }

    pub fn owner(&self) -> MapId {
        self.owner
    }

    /// `Active`: awake, and not on the internal map.
    pub fn active(&self) -> bool {
        self.active && self.owner != MapId::INTERNAL
    }

    /// `HasMultis`.
    pub fn has_multis(&self) -> bool {
        !self.multis.is_empty()
    }
}

fn key_xy(key: Option<(i32, i32)>) -> (i32, i32) {
    // The invalid sector's X and Y are 0.
    key.unwrap_or((0, 0))
}

/// `Item.OnSectorActivate`-style wake of one sector: `Sector.Activate`.
fn activate(w: &mut World, map: MapId, sx: i32, sy: i32) {
    let Some(m) = w.maps.get(map) else { return };
    let s = m.sector(sx, sy);
    if s.active() {
        return;
    }
    let (items, mobiles) = (s.items.clone(), s.mobiles.clone());
    for id in items {
        if let Some(ty) = w.item(id).map(|i| i.ty) {
            (ty.on_sector_activate)(w, id);
        }
    }
    for id in mobiles {
        if let Some(ty) = w.mobile(id).map(|m| m.ty) {
            (ty.on_sector_activate)(w, id);
        }
    }
    if let Some(m) = w.maps.get_mut(map) {
        m.sector_mut(sx, sy).active = true;
    }
}

/// `Sector.Deactivate`.
fn deactivate(w: &mut World, map: MapId, sx: i32, sy: i32) {
    let Some(m) = w.maps.get(map) else { return };
    let s = m.sector(sx, sy);
    if !s.active() {
        return;
    }
    let (items, mobiles) = (s.items.clone(), s.mobiles.clone());
    for id in items {
        if let Some(ty) = w.item(id).map(|i| i.ty) {
            (ty.on_sector_deactivate)(w, id);
        }
    }
    for id in mobiles {
        if let Some(ty) = w.mobile(id).map(|m| m.ty) {
            (ty.on_sector_deactivate)(w, id);
        }
    }
    if let Some(m) = w.maps.get_mut(map) {
        m.sector_mut(sx, sy).active = false;
    }
}

/// `Map.ActivateSectors`: wakes the sectors within `SectorActiveRange` of sector `(cx, cy)`.
pub fn activate_sectors(w: &mut World, map: MapId, cx: i32, cy: i32) {
    for x in cx - SECTOR_ACTIVE_RANGE..=cx + SECTOR_ACTIVE_RANGE {
        for y in cy - SECTOR_ACTIVE_RANGE..=cy + SECTOR_ACTIVE_RANGE {
            let Some(m) = w.maps.get_mut(map) else { return };
            // GetRealSector creates the sector.
            m.sector_mut(x, y);
            if m.sector_key(x, y).is_some() {
                activate(w, map, x, y);
            }
        }
    }
}

/// `Map.DeactivateSectors`.
pub fn deactivate_sectors(w: &mut World, map: MapId, cx: i32, cy: i32) {
    for x in cx - SECTOR_ACTIVE_RANGE..=cx + SECTOR_ACTIVE_RANGE {
        for y in cy - SECTOR_ACTIVE_RANGE..=cy + SECTOR_ACTIVE_RANGE {
            let Some(m) = w.maps.get_mut(map) else { return };
            m.sector_mut(x, y);
            if m.sector_key(x, y).is_some() && !players_in_range(w, map, x, y, SECTOR_ACTIVE_RANGE)
            {
                deactivate(w, map, x, y);
            }
        }
    }
}

/// `Map.PlayersInRange(sector, range)`.
fn players_in_range(w: &mut World, map: MapId, sx: i32, sy: i32, range: i32) -> bool {
    let Some(m) = w.maps.get_mut(map) else {
        return false;
    };
    for x in sx - range..=sx + range {
        for y in sy - range..=sy + range {
            m.sector_mut(x, y);
            if m.sector_key(x, y).is_some() && !m.sector(x, y).clients.is_empty() {
                return true;
            }
        }
    }
    false
}

fn is_internal(map: MapId) -> bool {
    map == MapId::INTERNAL
}

fn sector_of(w: &World, map: MapId, p: Point3D) -> Option<(i32, i32)> {
    w.maps
        .get(map)
        .map(|_| (p.x >> SECTOR_SHIFT, p.y >> SECTOR_SHIFT))
}

/// `Map.OnClientChange(oldState, newState, m)`.
pub fn on_client_change(
    w: &mut World,
    map: MapId,
    old: Option<NetStateId>,
    new: Option<NetStateId>,
    m: MobileId,
) {
    if is_internal(map) {
        return;
    }
    let Some(loc) = w.mobile(m).map(|m| m.location) else {
        return;
    };
    let Some((sx, sy)) = sector_of(w, map, loc) else {
        return;
    };
    let Some(mp) = w.maps.get_mut(map) else {
        return;
    };
    let s = mp.sector_mut(sx, sy);
    let count = s.clients.len();
    if let Some(old) = old
        && let Some(i) = s.clients.iter().position(|c| c.0 == old)
    {
        s.clients.remove(i);
    }
    if let Some(new) = new {
        s.clients.push((new, m));
    }
    let now = s.clients.len();
    if now == 0 && count > 0 {
        deactivate_sectors(w, map, sx, sy);
    } else if count == 0 && now > 0 {
        activate_sectors(w, map, sx, sy);
    }
}

/// `Map.OnEnter(p, mobile)`.
pub fn on_enter_mobile(w: &mut World, map: MapId, p: Point3D, m: MobileId) {
    if is_internal(map) {
        return;
    }
    let ns = w.mobile(m).and_then(|m| m.net_state);
    let (sx, sy) = (p.x >> SECTOR_SHIFT, p.y >> SECTOR_SHIFT);
    let Some(mp) = w.maps.get_mut(map) else {
        return;
    };
    let s = mp.sector_mut(sx, sy);
    s.mobiles.push(m);
    if let Some(ns) = ns {
        s.clients.push((ns, m));
        activate_sectors(w, map, sx, sy);
    }
}

/// `Map.OnLeave(p, mobile)`.
pub fn on_leave_mobile(w: &mut World, map: MapId, p: Point3D, m: MobileId) {
    if is_internal(map) {
        return;
    }
    let ns = w.mobile(m).and_then(|m| m.net_state);
    let (sx, sy) = (p.x >> SECTOR_SHIFT, p.y >> SECTOR_SHIFT);
    let Some(mp) = w.maps.get_mut(map) else {
        return;
    };
    let s = mp.sector_mut(sx, sy);
    if let Some(i) = s.mobiles.iter().position(|&x| x == m) {
        s.mobiles.remove(i);
    }
    if let Some(ns) = ns {
        if let Some(i) = s.clients.iter().position(|c| c.0 == ns) {
            s.clients.remove(i);
        }
        deactivate_sectors(w, map, sx, sy);
    }
}

/// Whether the item is a `BaseMulti`.
pub fn is_multi(w: &World, id: ItemId) -> bool {
    match (w.maps.base_multi, w.item(id)) {
        (Some(base), Some(item)) => item.ty.is(base),
        _ => false,
    }
}

/// The corner sectors a multi at `loc` covers, `(start, end)` as sector coordinates.
fn multi_span(w: &World, map: MapId, id: ItemId, loc: Point3D) -> ((i32, i32), (i32, i32)) {
    let mcl = (w.maps.multi_components)(w, id);
    (
        key_xy(w.maps.multi_min_sector(map, loc, mcl)),
        key_xy(w.maps.multi_max_sector(map, loc, mcl)),
    )
}

fn add_multi(w: &mut World, map: MapId, id: ItemId, start: (i32, i32), end: (i32, i32)) {
    if is_internal(map) {
        return;
    }
    let Some(mp) = w.maps.get_mut(map) else {
        return;
    };
    for x in start.0..=end.0 {
        for y in start.1..=end.1 {
            let s = mp.sector_mut(x, y);
            s.multis.push(id);
            s.multis_version += 1;
        }
    }
}

fn remove_multi(w: &mut World, map: MapId, id: ItemId, start: (i32, i32), end: (i32, i32)) {
    if is_internal(map) {
        return;
    }
    let Some(mp) = w.maps.get_mut(map) else {
        return;
    };
    for x in start.0..=end.0 {
        for y in start.1..=end.1 {
            let s = mp.sector_mut(x, y);
            if let Some(i) = s.multis.iter().position(|&m| m == id) {
                s.multis.remove(i);
                s.multis_version += 1;
            }
        }
    }
}

/// `Map.OnEnter(p, item)`.
pub fn on_enter_item(w: &mut World, map: MapId, p: Point3D, id: ItemId) {
    let Some(item) = w.item(id) else { return };
    if is_internal(map) || item.parent.is_some() {
        return;
    }
    let loc = item.location;
    let (sx, sy) = (p.x >> SECTOR_SHIFT, p.y >> SECTOR_SHIFT);
    let Some(mp) = w.maps.get_mut(map) else {
        return;
    };
    mp.sector_mut(sx, sy).items.push(id);

    if is_multi(w, id) {
        let (start, end) = multi_span(w, map, id, loc);
        add_multi(w, map, id, start, end);
    }
}

/// `Map.OnLeave(p, item)`.
pub fn on_leave_item(w: &mut World, map: MapId, p: Point3D, id: ItemId) {
    let Some(item) = w.item(id) else { return };
    if is_internal(map) || item.parent.is_some() {
        return;
    }
    let loc = item.location;
    let (sx, sy) = (p.x >> SECTOR_SHIFT, p.y >> SECTOR_SHIFT);
    let Some(mp) = w.maps.get_mut(map) else {
        return;
    };
    let s = mp.sector_mut(sx, sy);
    if let Some(i) = s.items.iter().position(|&x| x == id) {
        s.items.remove(i);
    }

    if is_multi(w, id) {
        let (start, end) = multi_span(w, map, id, loc);
        remove_multi(w, map, id, start, end);
    }
}

/// `Map.OnMove(oldLocation, mobile)`.
pub fn on_move_mobile(w: &mut World, map: MapId, old: Point3D, m: MobileId) {
    if is_internal(map) {
        return;
    }
    let Some(new) = w.mobile(m).map(|m| m.location) else {
        return;
    };
    let (osx, osy) = (old.x >> SECTOR_SHIFT, old.y >> SECTOR_SHIFT);
    let (nsx, nsy) = (new.x >> SECTOR_SHIFT, new.y >> SECTOR_SHIFT);
    let Some(mp) = w.maps.get(map) else { return };
    if mp.sector_key(osx, osy) != mp.sector_key(nsx, nsy) {
        on_leave_mobile(w, map, old, m);
        on_enter_mobile(w, map, new, m);
    }
}

/// `Map.OnMove(oldLocation, item)`. (Unlike `OnLeave`/`OnEnter`, `Parent` is not looked at here.)
pub fn on_move_item(w: &mut World, map: MapId, old: Point3D, id: ItemId) {
    if is_internal(map) {
        return;
    }
    let Some(new) = w.item(id).map(|i| i.location) else {
        return;
    };
    let (osx, osy) = (old.x >> SECTOR_SHIFT, old.y >> SECTOR_SHIFT);
    let (nsx, nsy) = (new.x >> SECTOR_SHIFT, new.y >> SECTOR_SHIFT);
    let Some(mp) = w.maps.get_mut(map) else {
        return;
    };
    if mp.sector_key(osx, osy) != mp.sector_key(nsx, nsy) {
        let o = mp.sector_mut(osx, osy);
        if let Some(i) = o.items.iter().position(|&x| x == id) {
            o.items.remove(i);
        }
        mp.sector_mut(nsx, nsy).items.push(id);
    }

    if is_multi(w, id) {
        let (start, end) = multi_span(w, map, id, new);
        let (old_start, old_end) = multi_span(w, map, id, old);
        if old_start != start || old_end != end {
            remove_multi(w, map, id, old_start, old_end);
            add_multi(w, map, id, start, end);
        }
    }
}
