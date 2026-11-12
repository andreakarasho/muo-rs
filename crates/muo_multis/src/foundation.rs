//! `HouseFoundation`: the customizable house. The state is [`FoundationData`] in `Item::ext`
//! next to `HouseData`; the three [`DesignState`]s (visible, being designed, backup) hold a footprint each. The
//! client's design packets are in `foundation_packets.rs`, the 0xD8 encoder in `house_packets.rs`.

use muo_content::items::house_deps::{DoorFacing, deps};
use muo_data::client::multi_data::{MultiComponentList, MultiTileEntry};
use muo_data::tile_data::TileFlag;
use muo_io::{GenericReader, GenericWriter, Result};
use muo_primitives::{Point3D, Rectangle2D};
use muo_server::World;
use muo_server::ids::{ItemId, MobileId};
use muo_server::item::multi::components_default;
use muo_server::item::{ItemDelta, ItemType, ops};
use muo_server::net_state::NetStateId;
use muo_server::serialization_ext::{ReadEntityExt, WriteEntityExt};

use crate::house::access::msg;
use crate::house::doors::{add_door, create_keys, door_for_item_id, remove_keys, set_sign};
use crate::house::save::new_base_house;
use crate::house::{self, BASE_HOUSE, HouseClass, storage};
use crate::house_packets;

pub const ALLOW_STAIR_SECTIONING: bool = true;

/// `FoundationType` (the ints the save and the house gump use).
pub const STONE: i32 = 0;

/// Stair block IDs (sorted ascending).
const BLOCK_IDS: [i32; 16] = [
    0x3EE, 0x709, 0x71E, 0x721, 0x738, 0x750, 0x76C, 0x788, 0x7A3, 0x7BA, 0x35D2, 0x3609, 0x4317,
    0x4318, 0x4B07, 0x7807,
];
/// Stair sequence IDs (sorted ascending): stairs in the proper N,W,S,E sequence.
const STAIR_SEQS: [i32; 8] = [0x3EF, 0x70A, 0x722, 0x739, 0x751, 0x76D, 0x789, 0x7A4];
/// Other stair IDs, in order north, west, south, east.
const STAIR_IDS: [i32; 34] = [
    0x71F, 0x736, 0x737, 0x749, 0x35D4, 0x35D3, 0x35D6, 0x35D5, 0x360B, 0x360A, 0x360D, 0x360C,
    0x4360, 0x435E, 0x435F, 0x4361, 0x435C, 0x435A, 0x435B, 0x435D, 0x4364, 0x4362, 0x4363, 0x4365,
    0x4B05, 0x4B04, 0x4B34, 0x4B33, 0x7809, 0x7808, 0x780A, 0x780B, 0x7BB, 0x7BC,
];

/// `DesignState`.
#[derive(Clone, Debug)]
pub struct DesignState {
    pub components: MultiComponentList,
    pub fixtures: Vec<MultiTileEntry>,
    pub revision: i32,
    /// `PacketCache`.
    pub packet_cache: Option<Vec<u8>>,
}

impl Default for DesignState {
    fn default() -> Self {
        DesignState::new(MultiComponentList::empty().clone())
    }
}

impl DesignState {
    /// `new DesignState(foundation, components)`.
    pub fn new(components: MultiComponentList) -> Self {
        DesignState {
            components,
            fixtures: Vec::new(),
            revision: 0,
            packet_cache: None,
        }
    }

    /// `new DesignState(toCopy)`: the packet cache is not copied.
    pub fn copy_of(o: &DesignState) -> Self {
        DesignState {
            components: o.components.clone(),
            fixtures: o.fixtures.clone(),
            revision: o.revision,
            packet_cache: None,
        }
    }

    /// `new DesignState(foundation, reader)`.
    pub fn read(r: &mut dyn GenericReader) -> Result<Self> {
        let version = r.read_i32()?;
        let mut s = DesignState::default();
        if version == 0 {
            s.components = MultiComponentList::deserialize(r)
                .map_err(|e| muo_io::IoError::InvalidData(e.to_string()))?;
            let length = r.read_i32()?;
            if !(0..=(1 << 20)).contains(&length) {
                return Err(muo_io::IoError::InvalidData(
                    "design state fixture count is out of range".into(),
                ));
            }
            for _ in 0..length {
                let item_id = r.read_u16()?;
                let (x, y, z) = (r.read_i16()?, r.read_i16()?, r.read_i16()?);
                let flags = TileFlag::from_bits_retain(u64::from(r.read_i32()? as u32));
                s.fixtures
                    .push(MultiTileEntry::new(item_id, x, y, z, flags));
            }
            s.revision = r.read_i32()?;
        }
        Ok(s)
    }

    /// `Serialize(writer)`.
    pub fn serialize(&self, out: &mut dyn GenericWriter) {
        out.write_i32(0);
        self.components.serialize(out);
        out.write_i32(self.fixtures.len() as i32);
        for e in &self.fixtures {
            out.write_u16(e.item_id);
            out.write_i16(e.offset_x);
            out.write_i16(e.offset_y);
            out.write_i16(e.offset_z);
            out.write_i32(e.flags.bits() as i32);
        }
        out.write_i32(self.revision);
    }
}

/// What `HouseFoundation` adds to a house.
#[derive(Clone, Debug, Default)]
pub struct FoundationData {
    pub ty: i32,
    pub last_revision: i32,
    pub fixtures: Vec<ItemId>,
    pub sign_hanger: Option<ItemId>,
    pub signpost: Option<ItemId>,
    pub signpost_graphic: i32,
    pub customizer: Option<MobileId>,
    /// Only read from saves older than 5.
    pub default_price: i32,
    pub current: Option<DesignState>,
    pub design: Option<DesignState>,
    pub backup: Option<DesignState>,
}

pub fn fd(w: &World, id: ItemId) -> Option<&FoundationData> {
    w.item(id)?.ext.get::<FoundationData>()
}

pub fn fd_mut(w: &mut World, id: ItemId) -> Option<&mut FoundationData> {
    w.item_mut(id)?.ext.get_mut::<FoundationData>()
}

/// `item is HouseFoundation`.
pub fn is_foundation(w: &World, id: ItemId) -> bool {
    w.item(id).is_some_and(|i| i.ty.is(&HOUSE_FOUNDATION))
}

/// `Type`.
pub fn foundation_type(w: &World, id: ItemId) -> i32 {
    fd(w, id).map_or(0, |f| f.ty)
}

/// `CustomizationCost`.
pub fn customization_cost(w: &World) -> i32 {
    if muo_content::core::aos(w) { 0 } else { 10000 }
}

/// `Customizer`.
pub fn customizer(w: &World, id: ItemId) -> Option<MobileId> {
    fd(w, id).and_then(|f| f.customizer)
}

/// `Signpost != null`.
pub fn has_signpost(w: &World, id: ItemId) -> bool {
    fd(w, id).is_some_and(|f| f.signpost.is_some())
}

/// `IsFixture(item)`.
pub fn is_fixture(w: &World, id: ItemId, item: ItemId) -> bool {
    fd(w, id).is_some_and(|f| f.fixtures.contains(&item))
}

/// `IsHiddenToCustomizer(item)`.
pub fn is_hidden_to_customizer(w: &World, id: ItemId, item: ItemId) -> bool {
    let f = fd(w, id);
    f.is_some_and(|f| f.signpost == Some(item) || f.sign_hanger == Some(item))
        || house::sign(w, id) == Some(item)
        || is_fixture(w, id, item)
}

// ---- the three states ----

/// `SetInitialState()`.
pub fn set_initial_state(w: &mut World, id: ItemId) {
    let empty = get_empty_foundation(w, id);
    if let Some(f) = fd_mut(w, id) {
        f.current = Some(DesignState::new(empty.clone()));
        f.design = Some(DesignState::copy_of(
            f.current.as_ref().unwrap_or(&DesignState::default()),
        ));
        f.backup = f.design.clone();
    }
}

/// The lazy getters: `CurrentState`, `DesignState`, `BackupState` start from the empty foundation.
fn ensure_states(w: &mut World, id: ItemId) {
    if fd(w, id).is_some_and(|f| f.current.is_none() || f.design.is_none() || f.backup.is_none()) {
        // all three are always created together at load or construction
        let (c, d, b) = fd(w, id)
            .map(|f| (f.current.is_some(), f.design.is_some(), f.backup.is_some()))
            .unwrap_or_default();
        if !c && !d && !b {
            set_initial_state(w, id);
        } else {
            let empty = get_empty_foundation(w, id);
            if let Some(f) = fd_mut(w, id) {
                f.current
                    .get_or_insert_with(|| DesignState::new(empty.clone()));
                let cur = f.current.clone().unwrap_or_default();
                f.design.get_or_insert_with(|| DesignState::copy_of(&cur));
                f.backup.get_or_insert_with(|| DesignState::copy_of(&cur));
            }
        }
    }
}

pub fn current_state(w: &mut World, id: ItemId) -> DesignState {
    ensure_states(w, id);
    fd(w, id)
        .and_then(|f| f.current.clone())
        .unwrap_or_default()
}

/// Runs `f` on the foundation's data and the tile data (taken out of the item meanwhile, put back after).
pub fn edit<R>(
    w: &mut World,
    id: ItemId,
    f: impl FnOnce(&mut FoundationData, &muo_data::TileData) -> R,
) -> Option<R> {
    ensure_states(w, id);
    let mut data = std::mem::take(fd_mut(w, id)?);
    let r = f(&mut data, &w.maps.tile_data);
    *fd_mut(w, id)? = data;
    Some(r)
}

impl FoundationData {
    /// `Revision = ++Foundation.LastRevision` for a state.
    pub fn revise(&mut self, s: &mut DesignState) {
        self.last_revision += 1;
        s.revision = self.last_revision;
        s.packet_cache = None;
    }
}

/// `DesignState` (the one being designed), initialized.
pub fn design_state(w: &mut World, id: ItemId) -> DesignState {
    ensure_states(w, id);
    fd(w, id).and_then(|f| f.design.clone()).unwrap_or_default()
}

/// `MultiComponentList Components`: the visible state's.
fn components(w: &World, id: ItemId) -> &MultiComponentList {
    fd(w, id)
        .and_then(|f| f.current.as_ref())
        .map_or_else(|| components_default(w, id), |s| &s.components)
}

/// `GetFoundationGraphics(type)`: (east, south, post, corner).
pub fn foundation_graphics(ty: i32) -> (i32, i32, i32, i32) {
    match ty {
        2 => (0x00BE, 0x00BF, 0x00C0, 0x00BD), // LightWood
        3 => (0x02FF, 0x02FE, 0x0300, 0x02FD), // Dungeon
        4 => (0x0043, 0x0042, 0x0044, 0x0041), // Brick
        0 => (0x0064, 0x0063, 0x0066, 0x0065), // Stone
        5 => (0x2DF9, 0x2DFA, 0x2DF8, 0x2DF7), // ElvenGrey
        6 => (0x2DFD, 0x2DFE, 0x2DFC, 0x2DFB), // ElvenNatural
        7 => (0x3671, 0x3670, 0x3673, 0x3672), // Crystal
        8 => (0x3675, 0x3674, 0x3677, 0x3676), // Shadow
        _ => (0x0015, 0x0016, 0x0017, 0x0014), // DarkWood
    }
}

/// `ApplyFoundation(type, mcl)`.
pub fn apply_foundation(ty: i32, mcl: &mut MultiComponentList, td: &muo_data::TileData) {
    let (east, south, post, corner) = foundation_graphics(ty);
    let (xc, yc) = (mcl.center().x, mcl.center().y);
    mcl.add(post, -xc, -yc, 0, td);
    mcl.add(corner, mcl.width() - 1 - xc, mcl.height() - 2 - yc, 0, td);
    for x in 1..mcl.width() {
        mcl.add(south, x - xc, -yc, 0, td);
        if x < mcl.width() - 1 {
            mcl.add(south, x - xc, mcl.height() - 2 - yc, 0, td);
        }
    }
    for y in 1..mcl.height() - 1 {
        mcl.add(east, -xc, y - yc, 0, td);
        if y < mcl.height() - 2 {
            mcl.add(east, mcl.width() - 1 - xc, y - yc, 0, td);
        }
    }
}

/// `GetEmptyFoundation()`.
pub fn get_empty_foundation(w: &World, id: ItemId) -> MultiComponentList {
    let item_id = w.item(id).map_or(0, |i| i.item_id);
    let mut mcl = w.maps.multi_data.get_components(item_id).clone();
    let td = &w.maps.tile_data;
    mcl.resize(mcl.width(), mcl.height() + 1);
    let (xc, yc) = (mcl.center().x, mcl.center().y);
    let y = mcl.height() - 1;
    apply_foundation(foundation_type(w, id), &mut mcl, td);
    for x in 1..mcl.width() {
        mcl.add(0x751, x - xc, y - yc, 0, td);
    }
    mcl
}

// ---- the class ----

/// `MaxLevels`.
pub fn max_levels(w: &World, id: ItemId) -> i32 {
    let mcl = components(w, id);
    if mcl.width() >= 14 || mcl.height() >= 14 {
        4
    } else {
        3
    }
}

/// `GetLevelZ(level, house)`.
pub fn get_level_z(w: &World, level: i32, house: ItemId) -> i32 {
    let level = if level < 1 || level > max_levels(w, house) {
        1
    } else {
        level
    };
    (level - 1) * 20 + 7
}

/// `GetZLevel(z, house)`.
pub fn get_z_level(w: &World, z: i32, house: ItemId) -> i32 {
    let level = (z - 7) / 20 + 1;
    if level < 1 || level > max_levels(w, house) {
        1
    } else {
        level
    }
}

pub static HOUSE_FOUNDATION_CLASS: HouseClass = HouseClass {
    area: |w, id| {
        let mcl = components(w, id);
        vec![Rectangle2D::new(
            mcl.min().x,
            mcl.min().y,
            mcl.width(),
            mcl.height(),
        )]
    },
    base_ban_location: |w, id| {
        let mcl = components(w, id);
        Point3D::new(mcl.min().x, mcl.height() - 1 - mcl.center().y, 0)
    },
    default_price: |w, id| fd(w, id).map_or(0, |f| f.default_price),
    is_aos_rules: Some(|_, _| true),
    is_active: |w, id| customizer(w, id).is_none(),
    class_name: "HouseFoundation",
    ..house::BASE_HOUSE_CLASS
};

/// `Server.Multis.HouseFoundation`.
pub static HOUSE_FOUNDATION: ItemType = ItemType {
    full_name: "Server.Multis.HouseFoundation",
    parent: Some(&BASE_HOUSE),
    construct: None,
    family: Some(&HOUSE_FOUNDATION_CLASS),
    components,
    get_max_update_range: |_, _| 24,
    get_update_range: |w, id, _| {
        let mcl = components(w, id);
        let (cw, ch) = (mcl.width(), mcl.height() - 1);
        (18 + (if cw > ch { cw } else { ch }) / 2).clamp(18, 24)
    },
    on_after_delete,
    on_location_change,
    on_map_change,
    send_info_to,
    serialize,
    deserialize,
    ..BASE_HOUSE
};

fn delete_item(w: &mut World, id: ItemId) {
    if let Some(ty) = w.item(id).map(|i| i.ty) {
        (ty.delete)(w, id);
    }
}

fn new_static(w: &mut World, graphic: i32) -> Option<ItemId> {
    let id = muo_content::factory::make_item(w, "Server.Items.Static")?;
    if let Some(ty) = w.item(id).map(|i| i.ty) {
        (ty.set_item_id)(w, id, graphic);
    }
    Some(id)
}

/// `new HouseFoundation(owner, multiID, maxLockdowns, maxSecures)`.
pub fn new_foundation(
    w: &mut World,
    owner: MobileId,
    multi_id: i32,
    max_lockdowns: i32,
    max_secures: i32,
) -> ItemId {
    let id = new_base_house(
        w,
        &HOUSE_FOUNDATION,
        multi_id,
        Some(owner),
        max_lockdowns,
        max_secures,
    );
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(FoundationData {
            signpost_graphic: 9,
            ..Default::default()
        });
    }
    // the first read of `Components` builds the initial states (`SetInitialState`)
    set_initial_state(w, id);
    let (x, y) = {
        let mcl = components(w, id);
        (mcl.min().x, mcl.height() - 1 - mcl.center().y)
    };
    let l = house::location(w, id);
    let hanger = new_static(w, 0xB98);
    if let Some(f) = fd_mut(w, id) {
        f.sign_hanger = hanger;
    }
    if let Some(h) = hanger {
        ops::move_to_world_map(
            w,
            h,
            Point3D::new(l.x + x, l.y + y, l.z + 7),
            house::map_of(w, id),
        );
    }
    check_signpost(w, id);
    set_sign(w, id, x, y, 7);
    id
}

/// `CheckWall(mcl, x, y)`.
pub fn check_wall(mcl: &MultiComponentList, x: i32, y: i32, td: &muo_data::TileData) -> bool {
    let (x, y) = (x + mcl.center().x, y + mcl.center().y);
    if x >= 0 && x < mcl.width() && y >= 0 && y < mcl.height() {
        return mcl.tiles[x as usize][y as usize]
            .iter()
            .any(|t| t.z == 7 && t.height(td) == 20);
    }
    false
}

/// `CheckSignpost()`.
pub fn check_signpost(w: &mut World, id: ItemId) {
    let (x, y, wall) = {
        let mcl = components(w, id);
        let (x, y) = (mcl.min().x, mcl.height() - 2 - mcl.center().y);
        (x, y, check_wall(mcl, x, y, &w.maps.tile_data))
    };
    let Some((post, graphic)) = fd(w, id).map(|f| (f.signpost, f.signpost_graphic)) else {
        return;
    };
    let l = house::location(w, id);
    let map = house::map_of(w, id);
    let at = Point3D::new(l.x + x, l.y + y, l.z + 7);
    if wall {
        if let Some(p) = post {
            delete_item(w, p);
        }
        if let Some(f) = fd_mut(w, id) {
            f.signpost = None;
        }
    } else if let Some(p) = post {
        if let Some(ty) = w.item(p).map(|i| i.ty) {
            (ty.set_item_id)(w, p, graphic);
        }
        ops::move_to_world_map(w, p, at, map);
    } else if let Some(p) = new_static(w, graphic) {
        if let Some(f) = fd_mut(w, id) {
            f.signpost = Some(p);
        }
        ops::move_to_world_map(w, p, at, map);
    }
}

fn on_after_delete(w: &mut World, id: ItemId) {
    storage::on_after_delete(w, id);
    let Some(f) = fd(w, id) else { return };
    let (hanger, post, fixtures) = (f.sign_hanger, f.signpost, f.fixtures.clone());
    for i in hanger.into_iter().chain(post).chain(fixtures) {
        delete_item(w, i);
    }
    if let Some(f) = fd_mut(w, id) {
        f.fixtures.clear();
    }
}

fn on_location_change(w: &mut World, id: ItemId, old: Point3D) {
    storage::on_location_change(w, id, old);
    let l = house::location(w, id);
    let (x, y, z) = (l.x - old.x, l.y - old.y, l.z - old.z);
    let map = house::map_of(w, id);
    let Some(f) = fd(w, id) else { return };
    let (hanger, post, fixtures) = (f.sign_hanger, f.signpost, f.fixtures.clone());
    for i in hanger.into_iter().chain(post) {
        let p = house::location(w, i);
        ops::move_to_world_map(w, i, Point3D::new(p.x + x, p.y + y, p.z + z), map);
    }
    let doors = house::d(w, id).doors;
    for i in fixtures {
        if doors.contains(&i) && crate::foundation::is_door(w, i) {
            continue;
        }
        let p = house::location(w, i);
        ops::move_to_world_map(w, i, Point3D::new(p.x + x, p.y + y, p.z + z), map);
    }
}

fn is_door(w: &World, item: ItemId) -> bool {
    deps(w)
        .and_then(|d| d.door_facing)
        .is_some_and(|f| f(w, item).is_some())
}

fn on_map_change(w: &mut World, id: ItemId) {
    storage::on_map_change(w, id);
    let map = house::map_of(w, id);
    let Some(f) = fd(w, id) else { return };
    let all: Vec<ItemId> = f
        .sign_hanger
        .into_iter()
        .chain(f.signpost)
        .chain(f.fixtures.iter().copied())
        .collect();
    for i in all {
        ops::set_map(w, i, map);
    }
}

fn send_info_to(w: &mut World, id: ItemId, ns: NetStateId, world: &[u8]) {
    muo_server::item::base::send_info_to_base(w, id, ns, world);
    let designing = w
        .net_state(ns)
        .and_then(|d| d.mobile)
        .and_then(|m| find_context(w, m))
        .is_some_and(|c| c.foundation == id);
    let st = if designing {
        design_state(w, id)
    } else {
        current_state(w, id)
    };
    house_packets::send_design_state_general(w, ns, serial_of(w, id), st.revision);
}

pub fn serial_of(w: &World, id: ItemId) -> muo_primitives::Serial {
    w.item(id)
        .map_or(muo_primitives::Serial::MINUS_ONE, |i| i.serial)
}

// ---- save ----

fn serialize(w: &World, id: ItemId, out: &mut dyn GenericWriter) {
    let Some(f) = fd(w, id) else {
        (BASE_HOUSE.serialize)(w, id, out);
        return;
    };
    out.write_i32(5);
    out.write_item(w, f.signpost);
    out.write_i32(f.signpost_graphic);
    out.write_i32(f.ty);
    out.write_item(w, f.sign_hanger);
    out.write_i32(f.last_revision);
    let live: Vec<ItemId> = f
        .fixtures
        .iter()
        .copied()
        .filter(|&i| w.item(i).is_some_and(|x| !x.deleted()))
        .collect();
    out.write_item_list(w, &live);
    let empty = || DesignState::new(get_empty_foundation(w, id));
    f.current.clone().unwrap_or_else(empty).serialize(out);
    f.design.clone().unwrap_or_else(empty).serialize(out);
    f.backup.clone().unwrap_or_else(empty).serialize(out);
    (BASE_HOUSE.serialize)(w, id, out);
}

fn deserialize(w: &mut World, id: ItemId, r: &mut dyn GenericReader) -> Result<()> {
    let mut f = FoundationData {
        signpost_graphic: 9,
        ..Default::default()
    };
    let version = r.read_i32()?;
    if version >= 4 {
        f.signpost = r.read_item(w)?;
        f.signpost_graphic = r.read_i32()?;
    }
    if version >= 3 {
        f.ty = r.read_i32()?;
    }
    if version >= 2 {
        f.sign_hanger = r.read_item(w)?;
    }
    if (1..5).contains(&version) {
        f.default_price = r.read_i32()?;
    }
    if version >= 0 {
        f.last_revision = r.read_i32()?;
        f.fixtures = r.read_item_list(w)?;
        f.current = Some(DesignState::read(r)?);
        f.design = Some(DesignState::read(r)?);
        f.backup = Some(DesignState::read(r)?);
    }
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(f);
    }
    (BASE_HOUSE.deserialize)(w, id, r)
}

// ---- fixtures ----

/// `ClearFixtures(from)`.
pub fn clear_fixtures(w: &mut World, id: ItemId, from: MobileId) {
    if fd(w, id).is_none() {
        return;
    }
    remove_keys(w, id, from);
    let fixtures = fd(w, id).map(|f| f.fixtures.clone()).unwrap_or_default();
    for i in fixtures {
        delete_item(w, i);
        if let Some(h) = house::data_mut(w, id) {
            h.doors.retain(|&d| d != i);
        }
    }
    if let Some(f) = fd_mut(w, id) {
        f.fixtures.clear();
    }
}

/// `AddFixture(item, mte)`.
pub fn add_fixture(w: &mut World, id: ItemId, item: ItemId, mte: &MultiTileEntry) {
    if let Some(f) = fd_mut(w, id) {
        f.fixtures.push(item);
    }
    let l = house::location(w, id);
    let p = Point3D::new(
        l.x + i32::from(mte.offset_x),
        l.y + i32::from(mte.offset_y),
        l.z + i32::from(mte.offset_z),
    );
    ops::move_to_world_map(w, item, p, house::map_of(w, id));
}

/// The doors `AddFixtures` knows: `door_for_item_id` less the Time of Legends ranges only `AddDoor` has.
pub(crate) fn fixture_door_for_item_id(
    item_id: i32,
) -> Option<muo_content::items::house_deps::HouseDoorKind> {
    if (0x9AD7..=0x9AE6).contains(&item_id) || (0x9B3C..=0x9B4B).contains(&item_id) {
        return None;
    }
    door_for_item_id(item_id)
}

/// `AddFixtures(from, list)`.
pub fn add_fixtures(w: &mut World, id: ItemId, from: MobileId, list: &[MultiTileEntry]) {
    let mut key_value = 0u32;
    for mte in list {
        let item_id = i32::from(mte.item_id);
        if (0x181D..0x1829).contains(&item_id) {
            let tp = crate::teleporter::new_house_teleporter(w, item_id, None);
            add_fixture(w, id, tp, mte);
        } else if let Some(kind) = fixture_door_for_item_id(item_id) {
            let Some(door) = deps(w).and_then(|d| d.make_door).and_then(|f| f(w, kind)) else {
                continue;
            };
            if key_value == 0 {
                key_value = create_keys(w, id, from);
            }
            if let Some(f) = deps(w).and_then(|d| d.door_set_locked) {
                f(w, door, true);
            }
            if let Some(f) = deps(w).and_then(|d| d.door_set_key_value) {
                f(w, door, key_value);
            }
            add_door(
                w,
                id,
                door,
                i32::from(mte.offset_x),
                i32::from(mte.offset_y),
                i32::from(mte.offset_z),
            );
            if let Some(f) = fd_mut(w, id) {
                f.fixtures.push(door);
            }
        }
    }
    link_fixtures(w, id);
}

/// The second loop of `AddFixtures`: teleporters target the next of their graphic, doors pair up.
fn link_fixtures(w: &mut World, id: ItemId) {
    let fixtures = fd(w, id).map(|f| f.fixtures.clone()).unwrap_or_default();
    let n = fixtures.len();
    let live = |w: &World, i: ItemId| w.item(i).is_some_and(|x| !x.deleted());
    let door_facing =
        |w: &World, i: ItemId| deps(w).and_then(|d| d.door_facing).and_then(|f| f(w, i));
    let door_link = |w: &World, i: ItemId| deps(w).and_then(|d| d.door_link).and_then(|f| f(w, i));
    for i in 0..n {
        let fixture = fixtures[i];
        if w.item(fixture)
            .is_some_and(|x| x.ty.is(&crate::teleporter::HOUSE_TELEPORTER))
        {
            let gid = w.item(fixture).map_or(0, |x| x.item_id);
            for j in 1..=n {
                let check = fixtures[(i + j) % n];
                if w.item(check).is_some_and(|x| {
                    x.ty.is(&crate::teleporter::HOUSE_TELEPORTER) && x.item_id == gid
                }) {
                    crate::teleporter::set_target(w, fixture, Some(check));
                    break;
                }
            }
        } else if let Some(facing) = door_facing(w, fixture) {
            if door_link(w, fixture).is_some_and(|l| live(w, l)) {
                continue;
            }
            use DoorFacing::*;
            let (link_facing, xo, yo) = match DoorFacing::from_i32(facing) {
                EastCCW => (WestCW, -1, 0),
                WestCCW => (EastCW, 1, 0),
                EastCW => (WestCCW, -1, 0),
                SouthCW => (NorthCCW, 0, -1),
                NorthCCW => (SouthCW, 0, 1),
                SouthCCW => (NorthCW, 0, -1),
                NorthCW => (SouthCCW, 0, 1),
                SouthCW2 => (SouthCCW2, 1, 0),
                SouthCCW2 => (SouthCW2, -1, 0),
                NorthCCW2 => (NorthCW2, 0, 1),
                NorthCW2 => (NorthCCW2, 0, -1),
                WestCW => (EastCCW, 1, 0),
            };
            let dl = house::location(w, fixture);
            for &check in &fixtures[i + 1..] {
                let Some(cf) = door_facing(w, check) else {
                    continue;
                };
                let cl = house::location(w, check);
                // `check.Link?.Deleted == false`: only a door that is already linked qualifies (an old slip that
                // keeps two fresh doors from pairing here; kept)
                if door_link(w, check).is_some_and(|l| live(w, l))
                    && cf == link_facing as i32
                    && cl.x - dl.x == xo
                    && cl.y - dl.y == yo
                    && cl.z == dl.z
                {
                    if let Some(f) = deps(w).and_then(|d| d.door_set_link) {
                        f(w, check, fixture);
                        f(w, fixture, check);
                    }
                    break;
                }
            }
        }
    }
}

// ---- customization ----

/// `BeginCustomize(m)`.
pub fn begin_customize(w: &mut World, id: ItemId, m: MobileId) {
    if !muo_server::mobile::part2::stats::check_alive(w, m, true) {
        return;
    }
    if muo_content::spells::helper::check_combat(w, m) {
        muo_server::mobile::part2::messages::send_localized_message(w, m, 1005564, "", 0x22); // Wouldst thou flee during the heat of battle??
        return;
    }
    storage::relocate_entities(w, id);
    let ban = house::ban_location(w, id);
    for item in storage::get_items(w, id) {
        ops::set_location(w, item, ban);
    }
    for mob in storage::get_mobiles(w, id) {
        if mob != m {
            muo_server::mobile::part1::lifecycle::set_location(w, mob, ban);
        }
    }
    add_context(w, m, id);
    let ns = muo_server::item::deps::net_state_of(w, m);
    if let Some(ns) = ns {
        house_packets::send_begin_house_customization(w, ns, serial_of(w, id));
        let ty = w.item(id).map(|i| i.ty);
        if let Some(ty) = ty {
            (ty.send_info_to)(w, id, ns, &[]);
        }
        send_detailed(w, id, ns, true);
    }
}

/// `DesignState.SendDetailedInfoTo(ns)` for the design (`design`) or the visible state.
pub fn send_detailed(w: &mut World, id: ItemId, ns: NetStateId, design: bool) {
    let mut st = if design {
        design_state(w, id)
    } else {
        current_state(w, id)
    };
    if st.packet_cache.is_none() {
        let packet = house_packets::create_house_design_state_detailed(
            w,
            serial_of(w, id),
            st.revision,
            &st.components,
        );
        st.packet_cache = Some(packet);
        if let Some(f) = fd_mut(w, id) {
            let slot = if design {
                &mut f.design
            } else {
                &mut f.current
            };
            if let Some(s) = slot {
                s.packet_cache = st.packet_cache.clone();
            }
        }
    }
    if let Some(p) = &st.packet_cache {
        w.send(ns, p);
    }
}

/// `EndConfirmCommit(from)`.
pub fn end_confirm_commit(w: &mut World, id: ItemId, from: MobileId) {
    let old_price = house::d(w, id).price;
    let dsn = design_state(w, id);
    let cur = current_state(w, id);
    let new_price = old_price
        + customization_cost(w)
        + (dsn.components.list.len() as i32
            - (cur.components.list.len() as i32 + cur.fixtures.len() as i32))
            * 500;
    let cost = new_price - old_price;

    if !house::deleted(w, id) {
        // Temporary Fix. We should be booting a client out of customization mode in the delete handler.
        let gm = w
            .mobile(from)
            .is_some_and(|m| m.access_level >= muo_server::mobile::AccessLevel::GameMaster);
        if gm && cost != 0 {
            let text = if cost > 0 {
                format!(
                    "{cost} gold would have been withdrawn from your bank if you were not a GM."
                )
            } else {
                format!(
                    "{cost} gold would have been deposited into your bank if you were not a GM."
                )
            };
            muo_server::mobile::part2::messages::send_message(w, from, &text);
        } else if cost > 0 {
            if muo_content::vendor::banker::withdraw(w, from, cost) {
                crate::house::access::msg_args(w, from, 1060398, &cost.to_string()); // ~1_AMOUNT~ gold has been withdrawn from your bank box.
            } else {
                msg(w, from, 1061903); // You cannot commit this house design, because you do not have the necessary funds in your bank box to pay for the upgrade.  Please back up your design, obtain the required funds, and commit your design again.
                return;
            }
        } else if cost < 0 {
            if muo_content::vendor::banker::deposit(w, from, -cost, true) {
                crate::house::access::msg_args(w, from, 1060397, &(-cost).to_string()); // ~1_AMOUNT~ gold has been deposited into your bank box.
            } else {
                return;
            }
        }
    }

    let mut copy = DesignState::copy_of(&dsn);
    clear_fixtures(w, id, from);
    edit(w, id, |f, _| melt_fixtures(&mut copy, &mut f.last_revision));
    add_fixtures(w, id, from, &copy.fixtures);
    if let Some(f) = fd_mut(w, id) {
        f.current = Some(copy);
    }
    let price = new_price - customization_cost(w);
    if let Some(h) = house::data_mut(w, id) {
        h.price = price;
    }
    remove_context(w, from);
    if let Some(ns) = muo_server::item::deps::net_state_of(w, from) {
        house_packets::send_end_house_customization(w, ns, serial_of(w, id));
    }
    ops::delta(w, id, ItemDelta::UPDATE);
    muo_server::item::base::process_delta(w, id);
    if let Some(ns) = muo_server::item::deps::net_state_of(w, from) {
        send_detailed(w, id, ns, false);
    }
    check_signpost(w, id);
    eject_all(w, id, from);
}

/// "Eject all from house": the customizer is revealed, items and mobiles inside go to the ban location, relocated
/// entities come back.
pub fn eject_all(w: &mut World, id: ItemId, from: MobileId) {
    muo_server::mobile::base::revealing_action(w, from);
    let ban = house::ban_location(w, id);
    for item in storage::get_items(w, id) {
        ops::set_location(w, item, ban);
    }
    for mob in storage::get_mobiles(w, id) {
        muo_server::mobile::part1::lifecycle::set_location(w, mob, ban);
    }
    storage::restore_relocated_entities(w, id);
}

/// `DesignState.MeltFixtures()`: `OnRevised()`, then the fixture graphics leave the components.
pub fn melt_fixtures(s: &mut DesignState, last_revision: &mut i32) {
    *last_revision += 1;
    s.revision = *last_revision;
    s.packet_cache = None;
    let list = s.components.list.clone();
    let mut melted: Vec<MultiTileEntry> = Vec::new();
    for mte in list.iter().rev() {
        if design_is_fixture(i32::from(mte.item_id)) {
            melted.push(*mte);
            s.components.remove(
                i32::from(mte.item_id),
                i32::from(mte.offset_x),
                i32::from(mte.offset_y),
                i32::from(mte.offset_z),
            );
        }
    }
    melted.reverse();
    s.fixtures = melted;
}

/// `DesignState.FreezeFixtures()`: `OnRevised()`, then the fixtures become components.
pub fn freeze_fixtures(s: &mut DesignState, last_revision: &mut i32, td: &muo_data::TileData) {
    *last_revision += 1;
    s.revision = *last_revision;
    s.packet_cache = None;
    for mte in std::mem::take(&mut s.fixtures) {
        s.components.add(
            i32::from(mte.item_id),
            i32::from(mte.offset_x),
            i32::from(mte.offset_y),
            i32::from(mte.offset_z),
            td,
        );
    }
}

/// `DesignState.IsFixture(itemID)`.
pub fn design_is_fixture(item_id: i32) -> bool {
    matches!(item_id,
        0x675..=0x6F4 | 0x314..=0x363 | 0x824..=0x833 | 0x839..=0x848 | 0x84C..=0x85B | 0x866..=0x875
        | 0x0E8..=0x0F7 | 0x1FED..=0x1FFC | 0x181D..=0x1828 | 0x241F..=0x2420 | 0x2423..=0x2424
        | 0x2A05..=0x2A1C | 0x319C..=0x31AF
        | 0x2D46 | 0x2D48 | 0x2FE2 | 0x2FE4 | 0x2D63..=0x2D6F | 0x367B..=0x369A
        | 0x409B..=0x40A2 | 0x410C..=0x4113 | 0x41C2..=0x41C9 | 0x41CF..=0x41D6 | 0x436E..=0x437D
        | 0x46DD..=0x46E4 | 0x4D22..=0x4D29 | 0x50C8..=0x50D7 | 0x5142..=0x5149
        | 0x9AD7..=0x9AE6 | 0x9B3C..=0x9B4B)
}

// ---- stairs ----

/// `IsStairBlock(id)`.
pub fn is_stair_block(id: i32) -> bool {
    let mut delta = -1;
    let mut i = 0;
    while delta < 0 && i < BLOCK_IDS.len() {
        delta = BLOCK_IDS[i] - id;
        i += 1;
    }
    delta == 0
}

/// `IsStair(id, ref dir)`: n=0 w=1 s=2 e=3.
pub fn is_stair(id: i32, dir: &mut i32) -> bool {
    let mut delta = -4;
    let mut i = 0;
    while delta < -3 && i < STAIR_SEQS.len() {
        delta = STAIR_SEQS[i] - id;
        i += 1;
    }
    if (-3..=0).contains(&delta) {
        *dir = -delta;
        return true;
    }
    if let Some(i) = STAIR_IDS.iter().position(|&s| s == id) {
        *dir = (i % 4) as i32;
        return true;
    }
    false
}

/// `DeleteStairs(mcl, id, x, y, z)`.
pub fn delete_stairs(
    mcl: &mut MultiComponentList,
    mut id: i32,
    mut x: i32,
    mut y: i32,
    mut z: i32,
    td: &muo_data::TileData,
) -> bool {
    let (ax, ay) = (x + mcl.center().x, y + mcl.center().y);
    if ax < 0 || ay < 0 || ax >= mcl.width() || ay >= mcl.height() - 1 || z < 7 || (z - 7) % 5 != 0
    {
        return false;
    }
    if is_stair_block(id) {
        let tiles = mcl.tiles[ax as usize][ay as usize].clone();
        for tile in tiles {
            if i32::from(tile.z) == z + 5 {
                id = i32::from(tile.id);
                z = i32::from(tile.z);
                if !is_stair_block(id) {
                    break;
                }
            }
        }
    }
    let mut dir = 0;
    if !is_stair(id, &mut dir) {
        return false;
    }
    if ALLOW_STAIR_SECTIONING {
        return true; // skip deletion
    }
    let height = (z - 7) % 20 / 5;
    let (x_start, y_start, x_inc, y_inc) = match dir {
        1 => (x + height, y, -1, 0),
        2 => (x, y - height, 0, 1),
        3 => (x - height, y, 1, 0),
        _ => (x, y + height, 0, -1),
    };
    let z_start = z - height * 5;
    for i in 0..4 {
        x = x_start + i * x_inc;
        y = y_start + i * y_inc;
        for j in 0..=i {
            mcl.remove_xyzh(x, y, z_start + j * 5, 5, td);
        }
        let (ax, ay) = (x + mcl.center().x, y + mcl.center().y);
        if ax >= 1 && ax < mcl.width() && ay >= 1 && ay < mcl.height() - 1 {
            let has_base_floor = mcl.tiles[ax as usize][ay as usize]
                .iter()
                .any(|t| t.z == 7 && t.id != 1);
            if !has_base_floor {
                mcl.add(0x31F4, x, y, 7, td);
            }
        }
    }
    true
}

// ---- DesignContext ----

/// `DesignContext`.
#[derive(Copy, Clone, Debug)]
pub struct DesignContext {
    pub foundation: ItemId,
    pub level: i32,
}

/// `DesignContext.Table` (insertion ordered).
#[derive(Default)]
pub struct DesignContexts(pub indexmap::IndexMap<MobileId, DesignContext>);

/// `DesignContext.Find(from)`.
pub fn find_context(w: &World, m: MobileId) -> Option<DesignContext> {
    w.resources.get::<DesignContexts>()?.0.get(&m).copied()
}

/// `DesignContext.Check(m)`.
pub fn check_context(w: &mut World, m: MobileId) -> bool {
    if find_context(w, m).is_none() {
        return true;
    }
    msg(w, m, 1062206); // You cannot do that while customizing a house.
    false
}

/// `DesignContext.Add(from, foundation)`.
pub fn add_context(w: &mut World, from: MobileId, foundation: ItemId) {
    w.resources.get_mut::<DesignContexts>().0.insert(
        from,
        DesignContext {
            foundation,
            level: 1,
        },
    );
    if let Some(f) = fd_mut(w, foundation) {
        f.customizer = Some(from);
    }
    muo_server::mobile::part1::props_b::set_hidden(w, from, true);
    let l = house::location(w, foundation);
    muo_server::mobile::part1::lifecycle::set_location(w, from, Point3D::new(l.x, l.y, l.z + 7));
    let Some(ns) = muo_server::item::deps::net_state_of(w, from) else {
        return;
    };
    let Some(f) = fd(w, foundation) else { return };
    let mut gone: Vec<ItemId> = f.fixtures.clone();
    gone.extend(f.signpost);
    gone.extend(f.sign_hanger);
    gone.extend(house::sign(w, foundation));
    for i in gone {
        if let Some(s) = w.item(i).map(|x| x.serial) {
            muo_server::packets::entity::send_remove_entity(w, ns, s);
        }
    }
}

/// `DesignContext.Remove(from)`.
pub fn remove_context(w: &mut World, from: MobileId) {
    let Some(ctx) = find_context(w, from) else {
        return;
    };
    w.resources
        .get_mut::<DesignContexts>()
        .0
        .shift_remove(&from);
    if let Some(f) = fd_mut(w, ctx.foundation) {
        f.customizer = None;
    }
    let Some(ns) = muo_server::item::deps::net_state_of(w, from) else {
        return;
    };
    let Some(f) = fd(w, ctx.foundation) else {
        return;
    };
    let mut shown: Vec<ItemId> = f.fixtures.clone();
    shown.extend(f.signpost);
    shown.extend(f.sign_hanger);
    shown.extend(house::sign(w, ctx.foundation));
    for i in shown {
        if let Some(ty) = w.item(i).map(|x| x.ty) {
            (ty.send_info_to)(w, i, ns, &[]);
        }
    }
}

/// The disconnect branch of `PlayerMobile.EventSink_Disconnected`.
pub fn on_disconnect(w: &mut World, m: MobileId) {
    let Some(ctx) = find_context(w, m) else {
        return;
    };
    remove_context(w, m);
    eject_all(w, ctx.foundation, m);
}

/// The walkable box `PlayerMobile.OnLocationChange` and `CheckMovement` read.
pub fn design_bounds(
    w: &World,
    m: MobileId,
) -> Option<muo_content::player_mobile::hooks::DesignBounds> {
    let ctx = find_context(w, m)?;
    let f = ctx.foundation;
    let mcl = components(w, f);
    let l = house::location(w, f);
    let start_x = l.x + mcl.min().x + 1;
    let start_y = l.y + mcl.min().y + 1;
    Some(muo_content::player_mobile::hooks::DesignBounds {
        start_x,
        start_y,
        end_x: start_x + mcl.width() - 1,
        end_y: start_y + mcl.height() - 2,
        z: l.z + get_level_z(w, ctx.level, f),
        foundation_x: l.x,
        foundation_y: l.y,
        map: house::map_of(w, f),
    })
}

/// `HouseFoundation.Initialize`'s speech block and the hooks the other crates left for the foundation.
pub fn install(w: &mut World) {
    use muo_content::player_mobile::hooks::SystemHooks;
    let s = w.resources.get_mut::<SystemHooks>();
    s.design_bounds = Some(design_bounds);
    s.design_check = Some(check_context);
    s.design_item_hidden = Some(|w, m, item| {
        find_context(w, m).is_some_and(|c| is_hidden_to_customizer(w, c.foundation, item))
    });
    s.design_on_disconnect = Some(on_disconnect);

    let d = muo_content::items::house_deps::deps_mut(w);
    d.construct_foundation = Some(|w, owner, multi_id, storage, lockdowns| {
        Some(new_foundation(w, owner, multi_id, storage, lockdowns))
    });
    d.foundation_customizer = Some(customizer);
    d.foundation_is_fixture = Some(is_fixture);

    w.events.speech.subscribe(|w, e| {
        if find_context(w, e.mobile).is_some() {
            msg(w, e.mobile, 1061925); // You cannot speak while customizing your house.
            e.blocked = true;
        }
    });
}

// ---- what the house gump changes ----

/// `foundation.SignHanger.ItemID = id`.
pub fn set_sign_hanger_graphic(w: &mut World, id: ItemId, graphic: i32) {
    if let Some(h) = fd(w, id).and_then(|f| f.sign_hanger)
        && let Some(ty) = w.item(h).map(|i| i.ty)
    {
        (ty.set_item_id)(w, h, graphic);
    }
}

/// `foundation.SignpostGraphic = id; foundation.CheckSignpost()`.
pub fn set_signpost_graphic(w: &mut World, id: ItemId, graphic: i32) {
    if let Some(f) = fd_mut(w, id) {
        f.signpost_graphic = graphic;
    }
    check_signpost(w, id);
}

/// `foundation.Type = t`, then `ApplyFoundation` and `OnRevised()` on the backup, design and current states (in
/// that order), then `Delta(ItemDelta.Update)` (HouseGumpAOS' foundation type buttons).
pub fn set_foundation_type(w: &mut World, id: ItemId, ty: i32) {
    edit(w, id, |f, td| {
        f.ty = ty;
        let mut backup = f.backup.take().unwrap_or_default();
        let mut design = f.design.take().unwrap_or_default();
        let mut current = f.current.take().unwrap_or_default();
        for st in [&mut backup, &mut design, &mut current] {
            apply_foundation(ty, &mut st.components, td);
            f.revise(st);
        }
        f.backup = Some(backup);
        f.design = Some(design);
        f.current = Some(current);
    });
    ops::delta(w, id, ItemDelta::UPDATE);
}
