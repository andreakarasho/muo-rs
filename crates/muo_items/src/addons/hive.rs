//! `SolenAntHole`, `SHTeleporter` and `BallotBox` (with their deeds).
//!
//! `BallotBoxGump.Singleton` is passed as the singleton flag of `send_gump_to_mobile`; the gump's type name is
//! its nested name.
//!
//! shortcut: a `BallotBox` list keeps deleted voters until the next save (`Tidy` runs in `Serialize` there; here the
//! write skips them and the in-memory list is not tidied, so a count read after a voter's deletion still includes
//! it for the rest of the run). Upgrade by giving `serialize` a `&mut World`.

use muo_content::gumps::base::{DynamicGumpLogic, DynamicGumpObj, RelayInfo};
use muo_content::gumps::builder::DynamicGumpBuilder;
use muo_content::items::addon::{
    self, ADDON_COMPONENT, AddonClass, BASE_ADDON, BASE_ADDON_CLASS, BASE_ADDON_DEED, Comp,
    DeedClass,
};
use muo_content::mx;
use muo_primitives::{Point3D, utility};
use muo_server::World;
use muo_server::ids::{ItemId, MobileId};
use muo_server::item::{ItemType, deps, messages, ops};
use muo_server::map::MapId;
use muo_server::mobile::AccessLevel;
use muo_server::net_state::NetStateId;
use muo_server::packets::message::MessageType;
use muo_server::prompt::{Prompt, add_prompt};
use muo_server::serialization_ext::{ReadEntityExt, WriteEntityExt};
use muo_server::type_registry::{Phase, TypeRegistry};

use crate::simple::leaf;

fn overhead(w: &mut World, from: MobileId, number: i32) {
    deps::local_overhead_message(w, from, MessageType::REGULAR, 0x3B2, number);
}

// ---- SolenAntHole -------------------------------------------------------------------------------------------

#[derive(Clone, Default)]
struct AntHoleData {
    spawned: Vec<MobileId>,
}

fn spawned(w: &World, id: ItemId) -> Vec<MobileId> {
    w.item(id)
        .and_then(|i| i.ext.get::<AntHoleData>())
        .map_or_else(Vec::new, |d| d.spawned.clone())
}

fn set_spawned(w: &mut World, id: ItemId, list: Vec<MobileId>) {
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(AntHoleData { spawned: list });
    }
}

fn ant_hole_component_on_double_click(w: &mut World, id: ItemId, from: MobileId) {
    let loc = w.item(id).map_or_else(Point3D::default, |i| i.location);
    if mx::in_range_point(w, from, loc, 2) {
        let map = w.item(id).and_then(|i| i.map);
        if map == Some(MapId::TRAMMEL) || map == Some(MapId::FELUCCA) {
            if let Some(map) = map {
                muo_server::mobile::base::move_to_world(w, from, Point3D::new(5922, 2024, 0), map);
            }
            let name = w
                .mobile(from)
                .and_then(|m| (m.ty.name)(w, from))
                .unwrap_or_default();
            // * ~1_NAME~ dives into the mysterious hole! *
            messages::public_overhead_message_number(
                w,
                id,
                MessageType::REGULAR,
                0x3B2,
                1114446,
                &name,
            );
        }
    } else {
        overhead(w, from, 1019045); // I can't reach that.
    }
}

/// `SolenAntHoleComponent(itemID)`.
pub fn solen_ant_hole_component_new(w: &mut World, item_id: i32) -> ItemId {
    addon::new_component(w, &SOLEN_ANT_HOLE_COMPONENT, item_id)
}

leaf! {
    /// `Server.Items.SolenAntHoleComponent`.
    SOLEN_ANT_HOLE_COMPONENT = "Server.Items.SolenAntHoleComponent" : ADDON_COMPONENT, enc;
    on_double_click: ant_hole_component_on_double_click,
}

#[allow(clippy::too_many_arguments)] // `AddComponent(component, x, y, z)` plus the component's name and hue
fn ant_hole_add(w: &mut World, a: ItemId, c: ItemId, name: &str, hue: i32, x: i32, y: i32, z: i32) {
    if let Some(ty) = w.item(c).map(|i| i.ty) {
        (ty.set_hue)(w, c, hue);
    }
    ops::set_name(w, c, Some(name.to_string()));
    addon::add_component(w, a, c, x, y, z);
}

/// `SolenAntHole()`.
pub fn solen_ant_hole_new(w: &mut World) -> ItemId {
    let id = addon::new_addon(w, &SOLEN_ANT_HOLE);
    set_spawned(w, id, Vec::new());
    let plain = |w: &mut World, i| addon::new_component(w, &ADDON_COMPONENT, i);
    let rows: [(i32, bool, &str, i32, i32, i32); 10] = [
        (0x914, false, "dirt", 0, 0, 0),
        (0x122A, true, "a hole", 0x1, 0, 0),
        (0x1B23, false, "dirt", 0x970, 1, 1),
        (0xEE0, false, "dirt", 0, 1, 0),
        (0x1B24, false, "dirt", 0x970, 1, -1),
        (0xEE1, false, "dirt", 0, 0, -1),
        (0x1B25, false, "dirt", 0x970, -1, -1),
        (0xEE2, false, "dirt", 0, -1, 0),
        (0x1B26, false, "dirt", 0x970, -1, 1),
        (0xED3, false, "dirt", 0, 0, 1),
    ];
    for (item, hole, name, hue, x, y) in rows {
        let c = if hole {
            solen_ant_hole_component_new(w, item)
        } else {
            plain(w, item)
        };
        ant_hole_add(w, id, c, name, hue, x, y, 0);
    }
    id
}

/// `SpawnKilled()`: forgets the dead and deleted, true when fewer than two remain.
fn ant_hole_spawn_killed(w: &mut World, id: ItemId) -> bool {
    let mut list = spawned(w, id);
    let mut i = list.len();
    while i > 0 {
        i -= 1;
        let m = list[i];
        if !mx::alive(w, m) || mx::deleted(w, m) {
            list.remove(i);
        }
    }
    let n = list.len();
    set_spawned(w, id, list);
    n < 2
}

/// `SpawnAnt(ant)`.
fn ant_hole_spawn_ant_of(w: &mut World, id: ItemId, ant: MobileId) {
    let mut list = spawned(w, id);
    list.push(ant);
    set_spawned(w, id, list);
    let map = w.item(id).and_then(|i| i.map);
    let loc = w.item(id).map_or_else(Point3D::default, |i| i.location);
    let mut p = loc;
    for _ in 0..5 {
        if muo_content::spells::helper::find_valid_spawn_location(w, map, &mut p, false) {
            break;
        }
    }
    if let Some(map) = map {
        muo_server::mobile::base::move_to_world(w, ant, p, map);
    }
    muo_content::creature::control::set_home(w, ant, loc);
    if let Some(d) = muo_content::creature::data::cd_mut(w, ant) {
        d.range_home = 10;
    }
}

fn ant_hole_spawn_ant(w: &mut World, id: ItemId) {
    let random = utility::random(&mut w.rng, 3);
    let map = w.item(id).and_then(|i| i.map);
    let name = if map == Some(MapId::TRAMMEL) {
        if random < 2 {
            "Server.Mobiles.RedSolenWorker"
        } else {
            "Server.Mobiles.RedSolenWarrior"
        }
    } else if map == Some(MapId::FELUCCA) {
        if random < 2 {
            "Server.Mobiles.BlackSolenWorker"
        } else {
            "Server.Mobiles.BlackSolenWarrior"
        }
    } else {
        return;
    };
    if let Some(ant) = muo_content::factory::make_mobile(w, name) {
        ant_hole_spawn_ant_of(w, id, ant);
    }
}

fn ant_hole_on_movement(w: &mut World, id: ItemId, m: MobileId, old: Point3D) {
    let hidden = w.mobile(m).is_some_and(|x| x.hidden);
    if !mx::player(w, m) || !mx::alive(w, m) || hidden || !ant_hole_spawn_killed(w, id) {
        return;
    }
    let loc = w.item(id).map_or_else(Point3D::default, |i| i.location);
    let mloc = mx::location(w, m);
    if utility::in_range(loc.x, loc.y, mloc.x, mloc.y, 3)
        && !utility::in_range(loc.x, loc.y, old.x, old.y, 3)
    {
        let count = 1 + utility::random(&mut w.rng, 4);
        for _ in 0..count {
            ant_hole_spawn_ant(w, id);
        }
        if utility::random_double(&mut w.rng) < 0.05
            && let Some(beetle) = muo_content::factory::make_mobile(w, "Server.Mobiles.Beetle")
        {
            ant_hole_spawn_ant_of(w, id, beetle);
        }
    }
}

static SOLEN_ANT_HOLE_CLASS: AddonClass = AddonClass {
    share_hue: false,
    ..BASE_ADDON_CLASS
};

/// `Server.Items.SolenAntHole`: `[SerializationGenerator(1)]`; version 0 read the list through `Deserialize(reader, 0)`.
pub static SOLEN_ANT_HOLE: ItemType = ItemType {
    full_name: "Server.Items.SolenAntHole",
    parent: Some(&BASE_ADDON),
    construct: Some(solen_ant_hole_new),
    family: Some(&SOLEN_ANT_HOLE_CLASS),
    handles_on_movement: |_, _| true,
    on_movement: ant_hole_on_movement,
    serialize: |w, id, out| {
        (BASE_ADDON.serialize)(w, id, out);
        out.write_encoded_int(1);
        let list = spawned(w, id);
        out.write_encoded_int(list.len() as i32);
        for m in list {
            out.write_mobile(w, Some(m));
        }
    },
    deserialize: |w, id, r| {
        (BASE_ADDON.deserialize)(w, id, r)?;
        // Version 0 (`Deserialize(reader, version)`): `ReadEntityList<Mobile>()`, an int count; version 1 an encoded one.
        let version = r.read_encoded_int()?;
        let list = if version < 1 {
            r.read_mobile_list(w)?
        } else {
            r.read_mobile_list_kept(true)?
        };
        set_spawned(w, id, list);
        Ok(())
    },
    ..BASE_ADDON
};

// ---- SHTeleporter -------------------------------------------------------------------------------------------

#[derive(Clone, Copy)]
pub struct TeleData {
    pub active: bool,
    pub dest: Option<ItemId>,
    pub offset: Point3D,
}

impl Default for TeleData {
    fn default() -> Self {
        TeleData {
            active: true,
            dest: None,
            offset: Point3D::default(),
        }
    }
}

#[derive(Clone, Copy, Default)]
pub struct TeleporterData {
    pub external: bool,
    pub up: Option<ItemId>,
    pub right: Option<ItemId>,
    pub down: Option<ItemId>,
    pub left: Option<ItemId>,
    pub changing: bool,
}

fn tele(w: &World, id: ItemId) -> TeleData {
    w.item(id)
        .and_then(|i| i.ext.get::<TeleData>())
        .copied()
        .unwrap_or_default()
}

fn teleporter(w: &World, id: ItemId) -> TeleporterData {
    w.item(id)
        .and_then(|i| i.ext.get::<TeleporterData>())
        .copied()
        .unwrap_or_default()
}

fn set_teleporter(w: &mut World, id: ItemId, d: TeleporterData) {
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(d);
    }
}

fn set_tele(w: &mut World, id: ItemId, d: TeleData) {
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(d);
    }
}

fn is_teleporter(w: &World, id: ItemId) -> bool {
    w.item(id).is_some_and(|i| i.ty.is(&SH_TELEPORTER))
}

fn is_tele_component(w: &World, id: ItemId) -> bool {
    w.item(id).is_some_and(|i| i.ty.is(&SH_TELE_COMPONENT))
}

/// `Addon is SHTeleporter`.
fn tele_owner(w: &World, c: ItemId) -> Option<ItemId> {
    addon::component_addon(w, c).filter(|a| is_teleporter(w, *a))
}

/// `SHTeleComponent.Active = value` (`OnActiveChanged` tells the addon).
pub fn sh_tele_set_active(w: &mut World, c: ItemId, value: bool) {
    let mut d = tele(w, c);
    if d.active == value {
        return;
    }
    d.active = value;
    set_tele(w, c, d);
    if let Some(a) = tele_owner(w, c) {
        sh_teleporter_change_active(w, a, value);
    }
}

/// `SHTeleComponent.TeleDest = value` (`OnTeleDestChanged` tells the addon).
pub fn sh_tele_set_dest(w: &mut World, c: ItemId, value: Option<ItemId>) {
    let mut d = tele(w, c);
    if d.dest == value {
        return;
    }
    d.dest = value;
    set_tele(w, c, d);
    if let Some(a) = tele_owner(w, c) {
        sh_teleporter_change_dest_component(w, a, value);
    }
}

fn tele_ends(w: &World, a: ItemId) -> [Option<ItemId>; 4] {
    let t = teleporter(w, a);
    [t.up, t.right, t.down, t.left]
}

/// `ChangeActive(active)`.
pub fn sh_teleporter_change_active(w: &mut World, a: ItemId, active: bool) {
    let mut t = teleporter(w, a);
    if t.changing {
        return;
    }
    t.changing = true;
    set_teleporter(w, a, t);
    for c in tele_ends(w, a).into_iter().flatten() {
        sh_tele_set_active(w, c, active);
    }
    let mut t = teleporter(w, a);
    t.changing = false;
    set_teleporter(w, a, t);
}

/// `ChangeDest(SHTeleComponent dest)`.
pub fn sh_teleporter_change_dest_component(w: &mut World, a: ItemId, dest: Option<ItemId>) {
    let mut t = teleporter(w, a);
    if t.changing {
        return;
    }
    t.changing = true;
    set_teleporter(w, a, t);
    let dest_addon = dest.and_then(|d| tele_owner(w, d));
    let ends = tele_ends(w, a);
    match dest_addon {
        None => {
            for c in ends.into_iter().flatten() {
                sh_tele_set_dest(w, c, dest);
            }
        }
        Some(da) => {
            let to = tele_ends(w, da);
            for (c, d) in ends.into_iter().zip(to) {
                if let Some(c) = c {
                    sh_tele_set_dest(w, c, d);
                }
            }
        }
    }
    let mut t = teleporter(w, a);
    t.changing = false;
    set_teleporter(w, a, t);
}

/// `ChangeDest(SHTeleporter destAddon)`.
pub fn sh_teleporter_change_dest(w: &mut World, a: ItemId, dest: Option<ItemId>) {
    let mut t = teleporter(w, a);
    if t.changing {
        return;
    }
    t.changing = true;
    set_teleporter(w, a, t);
    let ends = tele_ends(w, a);
    let to = dest.map_or([None; 4], |d| tele_ends(w, d));
    for (c, d) in ends.into_iter().zip(to) {
        if let Some(c) = c {
            sh_tele_set_dest(w, c, d);
        }
    }
    let mut t = teleporter(w, a);
    t.changing = false;
    set_teleporter(w, a, t);
}

/// `TelePoint`.
pub fn sh_tele_point(w: &World, c: ItemId) -> Point3D {
    let loc = w.item(c).map_or_else(Point3D::default, |i| i.location);
    let o = tele(w, c).offset;
    Point3D::new(loc.x + o.x, loc.y + o.y, loc.z + o.z)
}

fn sh_tele_on_double_click(w: &mut World, id: ItemId, m: MobileId) {
    let d = tele(w, id);
    let Some(dest) = d.dest.filter(|x| d.active && !ops::is_deleted(w, *x)) else {
        return;
    };
    let dest_map = w.item(dest).and_then(|i| i.map);
    let Some(map) = dest_map.filter(|m| *m != MapId::INTERNAL) else {
        return;
    };
    let loc = w.item(id).map_or_else(Point3D::default, |i| i.location);
    if mx::in_range_point(w, m, loc, 3) {
        let p = sh_tele_point(w, dest);
        muo_content::creature::pets::teleport_pets(w, m, p, map, false);
        muo_server::mobile::base::move_to_world(w, m, p, map);
    } else {
        mx::msg(w, m, 1019045); // I can't reach that.
    }
}

fn sh_tele_new(w: &mut World, item_id: i32, offset: Point3D) -> ItemId {
    let id = addon::new_component(w, &SH_TELE_COMPONENT, item_id);
    if let Some(ty) = w.item(id).map(|i| i.ty) {
        (ty.set_hue)(w, id, 1);
    }
    set_tele(
        w,
        id,
        TeleData {
            active: true,
            dest: None,
            offset,
        },
    );
    id
}

/// `Server.Items.SHTeleComponent`.
pub static SH_TELE_COMPONENT: ItemType = ItemType {
    full_name: "Server.Items.SHTeleComponent",
    parent: Some(&ADDON_COMPONENT),
    construct: Some(|w| sh_tele_new(w, 0x1775, Point3D::default())),
    default_name: |_, _| Some("a hole".to_string()),
    on_double_click: sh_tele_on_double_click,
    on_double_click_dead: sh_tele_on_double_click,
    serialize: |w, id, out| {
        (ADDON_COMPONENT.serialize)(w, id, out);
        out.write_i32(0);
        let d = tele(w, id);
        out.write_bool(d.active);
        out.write_item(w, d.dest);
        out.write_point3d(d.offset);
    },
    deserialize: |w, id, r| {
        (ADDON_COMPONENT.deserialize)(w, id, r)?;
        r.read_i32()?;
        let active = r.read_bool()?;
        let dest = r.read_item(w)?.filter(|x| is_tele_component(w, *x));
        let offset = r.read_point3d()?;
        set_tele(
            w,
            id,
            TeleData {
                active,
                dest,
                offset,
            },
        );
        Ok(())
    },
    ..ADDON_COMPONENT
};

static SH_TELEPORTER_CLASS: AddonClass = AddonClass {
    share_hue: false,
    ..BASE_ADDON_CLASS
};

/// `SHTeleporter(external)`.
pub fn sh_teleporter_new_with(w: &mut World, external: bool) -> ItemId {
    let id = addon::new_addon(w, &SH_TELEPORTER);
    let mut t = TeleporterData {
        external,
        ..Default::default()
    };
    set_teleporter(w, id, t);
    if external {
        const ROWS: [(i32, i32, i32); 16] = [
            (0x549, -1, -1),
            (0x54D, 0, -1),
            (0x54E, 1, -1),
            (0x548, 2, -1),
            (0x54B, -1, 0),
            (0x53B, 0, 0),
            (0x53B, 1, 0),
            (0x544, 2, 0),
            (0x54C, -1, 1),
            (0x53B, 0, 1),
            (0x53B, 1, 1),
            (0x545, 2, 1),
            (0x547, -1, 2),
            (0x541, 0, 2),
            (0x543, 1, 2),
            (0x540, 2, 2),
        ];
        for (item, x, y) in ROWS {
            addon::add_components(w, id, &[(Comp::Plain(item), x, y, 0)]);
        }
    }
    let p = |a, b| Point3D::new(a, b, 0);
    let (up_os, right_os, down_os, left_os) = if external {
        (p(-1, 0), p(-2, 0), p(-2, -1), p(-1, -1))
    } else {
        (p(-2, -1), p(2, -1), p(2, 2), p(-1, 2))
    };
    let (g_up, g_down) = if external {
        (0x1775, 0x1776)
    } else {
        (0x495, 0x495)
    };
    let up = sh_tele_new(w, g_up, up_os);
    addon::add_component(w, id, up, 0, 0, 0);
    let right = sh_tele_new(w, g_up, right_os);
    addon::add_component(w, id, right, 1, 0, 0);
    let down = sh_tele_new(w, g_down, down_os);
    addon::add_component(w, id, down, 1, 1, 0);
    let left = sh_tele_new(w, g_up, left_os);
    addon::add_component(w, id, left, 0, 1, 0);
    t.up = Some(up);
    t.right = Some(right);
    t.down = Some(down);
    t.left = Some(left);
    set_teleporter(w, id, t);
    id
}

/// `Server.Items.SHTeleporter`.
pub static SH_TELEPORTER: ItemType = ItemType {
    full_name: "Server.Items.SHTeleporter",
    parent: Some(&BASE_ADDON),
    construct: Some(|w| sh_teleporter_new_with(w, true)),
    family: Some(&SH_TELEPORTER_CLASS),
    serialize: |w, id, out| {
        (BASE_ADDON.serialize)(w, id, out);
        out.write_i32(0);
        let t = teleporter(w, id);
        out.write_bool(t.external);
        for c in [t.up, t.right, t.down, t.left] {
            out.write_item(w, c);
        }
    },
    deserialize: |w, id, r| {
        (BASE_ADDON.deserialize)(w, id, r)?;
        r.read_i32()?;
        let external = r.read_bool()?;
        let mut ends = [None; 4];
        for e in &mut ends {
            *e = r.read_item(w)?.filter(|x| is_tele_component(w, *x));
        }
        set_teleporter(
            w,
            id,
            TeleporterData {
                external,
                up: ends[0],
                right: ends[1],
                down: ends[2],
                left: ends[3],
                changing: false,
            },
        );
        Ok(())
    },
    ..BASE_ADDON
};

/// `FindSHTeleporter(map, p)`.
pub fn find_sh_teleporter(w: &World, map: MapId, p: Point3D) -> Option<ItemId> {
    deps::items_at(w, map, p)
        .into_iter()
        .find(|i| is_teleporter(w, *i) && w.item(*i).is_some_and(|x| x.location.z == p.z))
}

/// `AddSHT(map, ext, x, y, z)`.
pub fn add_sht(w: &mut World, map: MapId, ext: bool, x: i32, y: i32, z: i32) -> ItemId {
    let p = Point3D::new(x, y, z);
    if let Some(t) = find_sh_teleporter(w, map, p) {
        return t;
    }
    let t = sh_teleporter_new_with(w, ext);
    ops::move_to_world_map(w, t, p, Some(map));
    t
}

/// `Link(tele1, tele2)`.
pub fn link(w: &mut World, a: ItemId, b: ItemId) {
    sh_teleporter_change_dest(w, a, Some(b));
    sh_teleporter_change_dest(w, b, Some(a));
}

#[allow(clippy::too_many_arguments)]
fn add_sht_couple_on(
    w: &mut World,
    map: MapId,
    ext1: bool,
    x1: i32,
    y1: i32,
    z1: i32,
    ext2: bool,
    x2: i32,
    y2: i32,
    z2: i32,
) {
    let a = add_sht(w, map, ext1, x1, y1, z1);
    let b = add_sht(w, map, ext2, x2, y2, z2);
    link(w, a, b);
}

#[allow(clippy::too_many_arguments)]
fn add_sht_couple(
    w: &mut World,
    ext1: bool,
    x1: i32,
    y1: i32,
    z1: i32,
    ext2: bool,
    x2: i32,
    y2: i32,
    z2: i32,
) {
    add_sht_couple_on(w, MapId::TRAMMEL, ext1, x1, y1, z1, ext2, x2, y2, z2);
    add_sht_couple_on(w, MapId::FELUCCA, ext1, x1, y1, z1, ext2, x2, y2, z2);
}

fn left_tele_offset(w: &mut World, t: ItemId, o: Point3D) {
    if let Some(l) = teleporter(w, t).left {
        let mut d = tele(w, l);
        d.offset = o;
        set_tele(w, l, d);
    }
}

/// `CreateSHTeleporters()`.
pub fn create_sh_teleporters(w: &mut World) {
    let (t, f) = (MapId::TRAMMEL, MapId::FELUCCA);
    add_sht_couple(w, true, 2608, 763, 0, false, 5918, 1794, 0);
    add_sht_couple(w, false, 5897, 1877, 0, false, 5871, 1867, 0);
    add_sht_couple(w, false, 5852, 1848, 0, false, 5771, 1867, 0);

    // The same pairs on each facet, the first end with its left offset moved.
    for (map, rows) in [
        (t, &[(5747, 1895, false, 5658, 1898)][..]),
        (f, &[(5747, 1895, false, 5658, 1898)][..]),
    ] {
        for &(x1, y1, ext2, x2, y2) in rows {
            let t1 = add_sht(w, map, false, x1, y1, 0);
            left_tele_offset(w, t1, Point3D::new(-1, 3, 0));
            let t2 = add_sht(w, map, ext2, x2, y2, 0);
            link(w, t1, t2);
        }
    }

    add_sht_couple(w, false, 5727, 1894, 0, false, 5756, 1794, 0);
    add_sht_couple(w, false, 5784, 1929, 0, false, 5700, 1929, 0);

    for map in [t, f] {
        let t1 = add_sht(w, map, false, 5711, 1952, 0);
        left_tele_offset(w, t1, Point3D::new(-1, 3, 0));
        let t2 = add_sht(w, map, false, 5657, 1954, 0);
        link(w, t1, t2);
    }
    for map in [t, f] {
        let t1 = add_sht(w, map, false, 5655, 2018, 0);
        left_tele_offset(w, t1, Point3D::new(-1, 3, 0));
        let t2 = add_sht(w, map, true, 1690, 2789, 0);
        link(w, t1, t2);
    }

    add_sht_couple(w, false, 5809, 1905, 0, false, 5876, 1891, 0);

    for map in [t, f] {
        let t1 = add_sht(w, map, false, 5814, 2015, 0);
        left_tele_offset(w, t1, Point3D::new(-1, 3, 0));
        let t2 = add_sht(w, map, false, 5913, 1893, 0);
        link(w, t1, t2);
    }

    add_sht_couple(w, false, 5919, 2021, 0, true, 1724, 814, 0);

    for map in [t, f] {
        let t1 = add_sht(w, map, false, 5654, 1791, 0);
        let t2 = add_sht(w, map, true, 730, 1451, 0);
        link(w, t1, t2);
        let extra = add_sht(w, map, false, 5734, 1859, 0);
        sh_teleporter_change_dest(w, extra, Some(t2));
    }
}

fn sh_teleporter_configure(w: &mut World) {
    use muo_server::commands::CommandSystem;
    // Usage: SHTelGen; Description: Generates solen hives teleporters.
    CommandSystem::register(
        w,
        "SHTelGen",
        AccessLevel::Developer,
        &[],
        std::rc::Rc::new(
            |w: &mut World, e: &mut muo_server::commands::CommandEventArgs| {
                create_sh_teleporters(w);
                muo_server::mobile::part2::messages::send_message(
                    w,
                    e.mobile,
                    "Solen Hive teleporters have been created.",
                );
            },
        ),
    );
}

// ---- BallotBox ----------------------------------------------------------------------------------------------

const MAX_TOPIC_LINES: usize = 6;

#[derive(Clone, Default)]
struct BallotData {
    topic: Vec<String>,
    yes: Vec<MobileId>,
    no: Vec<MobileId>,
}

fn ballot(w: &World, id: ItemId) -> BallotData {
    w.item(id)
        .and_then(|i| i.ext.get::<BallotData>())
        .cloned()
        .unwrap_or_default()
}

fn set_ballot(w: &mut World, id: ItemId, d: BallotData) {
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(d);
    }
}

fn ballot_clear_votes(w: &mut World, id: ItemId) {
    let mut d = ballot(w, id);
    d.yes.clear();
    d.no.clear();
    set_ballot(w, id, d);
}

fn ballot_add_line(w: &mut World, id: ItemId, line: String) {
    let mut d = ballot(w, id);
    if d.topic.len() >= MAX_TOPIC_LINES {
        return;
    }
    d.topic.push(line);
    set_ballot(w, id, d);
    ballot_clear_votes(w, id);
}

fn ballot_is_owner(w: &mut World, id: ItemId, from: MobileId) -> bool {
    if mx::access_level(w, from) >= AccessLevel::GameMaster {
        return true;
    }
    match addon::find_house_at_item(w, id) {
        Some(h) => addon::house_is_owner(w, h, from),
        None => false,
    }
}

fn ballot_in_reach(w: &World, id: ItemId, from: MobileId, same_map: bool) -> bool {
    let loc = ops::get_world_location(w, id);
    let map_ok = !same_map || mx::map(w, from) == w.item(id).and_then(|i| i.map);
    map_ok && mx::in_range_point(w, from, loc, 2)
}

const BALLOT_GUMP: &str = "Server.Items.BallotBox+BallotBoxGump";

/// `BallotBoxGump.OnResponse`.
fn ballot_gump_response(w: &mut World, box_id: ItemId, sender: NetStateId, info: &RelayInfo) {
    if ops::is_deleted(w, box_id) || info.button_id == 0 {
        return;
    }
    let Some(from) = w.net_states.get(&sender).and_then(|n| n.mobile) else {
        return;
    };
    if !ballot_in_reach(w, box_id, from, true) {
        overhead(w, from, 1019045); // I can't reach that.
        return;
    }
    let is_owner = ballot_is_owner(w, box_id, from);
    match info.button_id {
        1 => {
            // change topic
            if is_owner {
                let mut d = ballot(w, box_id);
                d.topic.clear();
                set_ballot(w, box_id, d);
                ballot_clear_votes(w, box_id);
                muo_server::mobile::part2::messages::send_localized_message(
                    w, from, 500370, "", 0x35,
                );
                start_topic_prompt(w, from, box_id);
            }
            return;
        }
        2 => {
            if is_owner {
                ballot_clear_votes(w, box_id);
                mx::msg(w, from, 500371); // Votes zeroed out.
            }
        }
        3 | 4 if !is_owner => {
            let mut d = ballot(w, box_id);
            if d.yes.contains(&from) || d.no.contains(&from) {
                mx::msg(w, from, 500374); // You have already voted on this ballot.
            } else {
                if info.button_id == 3 {
                    d.yes.push(from);
                } else {
                    d.no.push(from);
                }
                set_ballot(w, box_id, d);
                mx::msg(w, from, 500373); // Your vote has been registered.
            }
        }
        _ => {}
    }
    ballot_show(w, from, box_id, is_owner);
}

/// The gump's layout needs the box's contents, so the gump is built from a snapshot taken when it is sent.
struct BallotLayout {
    topic: Vec<String>,
    yes: usize,
    no: usize,
    is_owner: bool,
    box_id: ItemId,
}

impl DynamicGumpLogic for BallotLayout {
    const TYPE_NAME: &'static str = BALLOT_GUMP;

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        b.add_page(0);
        b.add_background(0, 0, 400, 350, 0xA28);
        if self.is_owner {
            b.add_html_localized(0, 15, 400, 35, 1011000, false, false); // <center>Ballot Box Owner's Menu</center>
        } else {
            b.add_html_localized(0, 15, 400, 35, 1011001, false, false); // <center>Ballot Box -- Vote Here!</center>
        }
        b.add_html_localized(0, 50, 400, 35, 1011002, false, false); // <center>Topic</center>
        let line_count = self.topic.len() as i32;
        b.add_background(25, 90, 350, (20 * line_count).max(20), 0x1400);
        for (i, line) in self.topic.iter().enumerate() {
            if !line.is_empty() {
                b.add_label_cropped(30, 90 + i as i32 * 20, 340, 20, 0x3E3, line);
            }
        }
        let (yes, no) = (self.yes as i32, self.no as i32);
        let total = yes + no;
        b.add_html_localized(0, 215, 400, 35, 1011003, false, false); // <center>votes</center>
        if !self.is_owner {
            b.add_button(20, 240, 0xFA5, 0xFA7, 3);
        }
        b.add_html_localized(55, 242, 25, 35, 1011004, false, false); // aye:
        b.add_label(78, 242, 0x0, &format!("[{yes}]"));
        if !self.is_owner {
            b.add_button(20, 275, 0xFA5, 0xFA7, 4);
        }
        b.add_html_localized(55, 277, 25, 35, 1011005, false, false); // nay:
        b.add_label(78, 277, 0x0, &format!("[{no}]"));
        if total > 0 {
            b.add_image_tiled(130, 242, yes * 225 / total, 10, 0xD6);
            b.add_image_tiled(130, 277, no * 225 / total, 10, 0xD6);
        }
        b.add_button(45, 305, 0xFA5, 0xFA7, 0);
        b.add_html_localized(80, 308, 40, 35, 1011008, false, false); // done
        if self.is_owner {
            b.add_button(120, 305, 0xFA5, 0xFA7, 1);
            b.add_html_localized(155, 308, 100, 35, 1011006, false, false); // change topic
            b.add_button(240, 305, 0xFA5, 0xFA7, 2);
            b.add_html_localized(275, 308, 300, 100, 1011007, false, false); // reset votes
        }
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        ballot_gump_response(w, self.box_id, sender, info);
    }
}

/// `BallotBoxGump.DisplayTo(from, box, isOwner)`.
fn ballot_show(w: &mut World, from: MobileId, box_id: ItemId, is_owner: bool) {
    if w.mobile(from).is_none_or(|m| m.net_state.is_none()) || ops::is_deleted(w, box_id) {
        return;
    }
    let d = ballot(w, box_id);
    let gump = DynamicGumpObj::new(
        110,
        70,
        BallotLayout {
            topic: d.topic,
            yes: d.yes.len(),
            no: d.no.len(),
            is_owner,
            box_id,
        },
    );
    muo_content::gumps::send_gump_to_mobile(w, from, Box::new(gump), true);
}

fn start_topic_prompt(w: &mut World, from: MobileId, box_id: ItemId) {
    let prompt = Prompt {
        on_response: Some(std::rc::Rc::new(move |w, from, text| {
            topic_response(w, from, box_id, text)
        })),
        on_cancel: Some(std::rc::Rc::new(move |w, from| {
            topic_cancel(w, from, box_id)
        })),
    };
    let pid = add_prompt(w, prompt);
    muo_server::mobile::part1::props_b::set_prompt(w, from, Some(pid));
}

fn topic_response(w: &mut World, from: MobileId, box_id: ItemId, text: &str) {
    if ops::is_deleted(w, box_id) || !ballot_is_owner(w, box_id, from) {
        return;
    }
    if !ballot_in_reach(w, box_id, from, true) {
        overhead(w, from, 1019045); // I can't reach that.
        return;
    }
    ballot_add_line(w, box_id, text.trim_end().to_string());
    if ballot(w, box_id).topic.len() < MAX_TOPIC_LINES {
        muo_server::mobile::part2::messages::send_localized_message(w, from, 500377, "", 0x35); // Next line or ESC to finish:
        start_topic_prompt(w, from, box_id);
    } else {
        muo_server::mobile::part2::messages::send_localized_message(w, from, 500376, "", 0x35); // Ballot entry complete.
        ballot_show(w, from, box_id, true);
    }
}

fn topic_cancel(w: &mut World, from: MobileId, box_id: ItemId) {
    if ops::is_deleted(w, box_id) || !ballot_is_owner(w, box_id, from) {
        return;
    }
    if !ballot_in_reach(w, box_id, from, true) {
        overhead(w, from, 1019045); // I can't reach that.
        return;
    }
    muo_server::mobile::part2::messages::send_localized_message(w, from, 500376, "", 0x35); // Ballot entry complete.
    ballot_show(w, from, box_id, true);
}

fn ballot_on_double_click(w: &mut World, id: ItemId, from: MobileId) {
    if !ballot_in_reach(w, id, from, false) {
        overhead(w, from, 1019045); // I can't reach that.
    } else {
        let owner = ballot_is_owner(w, id, from);
        ballot_show(w, from, id, owner);
    }
}

/// `BallotBox()`.
pub fn ballot_box_new(w: &mut World) -> ItemId {
    let id = addon::new_component(w, &BALLOT_BOX, 0x9A8);
    set_ballot(w, id, BallotData::default());
    id
}

/// `Server.Items.BallotBox`.
pub static BALLOT_BOX: ItemType = ItemType {
    full_name: "Server.Items.BallotBox",
    parent: Some(&ADDON_COMPONENT),
    construct: Some(ballot_box_new),
    label_number: |_, _| 1041006, // a ballot box
    on_drag_drop: |w, id, from, _dropped| {
        messages::send_localized_message_to(w, id, from, 500369, ""); // I'm a ballot box, not a container!
        false
    },
    on_double_click: ballot_on_double_click,
    serialize: |w, id, out| {
        (ADDON_COMPONENT.serialize)(w, id, out);
        out.write_encoded_int(0);
        let d = ballot(w, id);
        out.write_encoded_int(d.topic.len() as i32);
        for line in &d.topic {
            out.write_string(line);
        }
        for list in [&d.yes, &d.no] {
            let live: Vec<MobileId> = list
                .iter()
                .copied()
                .filter(|m| w.mobile(*m).is_some_and(|x| !x.deleted))
                .collect();
            out.write_encoded_int(live.len() as i32);
            for m in live {
                out.write_mobile(w, Some(m));
            }
        }
    },
    deserialize: |w, id, r| {
        (ADDON_COMPONENT.deserialize)(w, id, r)?;
        r.read_encoded_int()?;
        let n = r.read_encoded_int()?;
        let mut d = BallotData::default();
        for _ in 0..n.max(0) {
            d.topic.push(r.read_string()?.unwrap_or_default());
        }
        for which in 0..2 {
            let n = r.read_encoded_int()?;
            let mut list = Vec::new();
            for _ in 0..n.max(0) {
                list.extend(r.read_mobile(w)?);
            }
            if which == 0 {
                d.yes = list
            } else {
                d.no = list
            }
        }
        set_ballot(w, id, d);
        Ok(())
    },
    ..ADDON_COMPONENT
};

/// `BallotBoxAddon()` (no `[Constructible]`).
pub fn ballot_box_addon_new(w: &mut World) -> ItemId {
    let id = addon::new_addon(w, &BALLOT_BOX_ADDON);
    let c = ballot_box_new(w);
    addon::add_component(w, id, c, 0, 0, 0);
    id
}

static BALLOT_BOX_ADDON_CLASS: AddonClass = AddonClass {
    deed: Some(|w| addon::new_deed(w, &BALLOT_BOX_DEED)),
    ..BASE_ADDON_CLASS
};
static BALLOT_BOX_DEED_CLASS: DeedClass = DeedClass {
    addon: |w, _| ballot_box_addon_new(w),
    select_direction: None,
};

leaf! {
    /// `Server.Items.BallotBoxAddon`.
    BALLOT_BOX_ADDON = "Server.Items.BallotBoxAddon" : BASE_ADDON, enc;
    family: Some(&BALLOT_BOX_ADDON_CLASS),
}
leaf! {
    /// `Server.Items.BallotBoxDeed`.
    BALLOT_BOX_DEED = "Server.Items.BallotBoxDeed" : BASE_ADDON_DEED, enc;
    construct: Some(|w| addon::new_deed(w, &BALLOT_BOX_DEED)),
    family: Some(&BALLOT_BOX_DEED_CLASS),
    label_number: |_, _| 1044327,
}

pub static ALL: &[(&ItemType, &[&str])] = &[
    (&SOLEN_ANT_HOLE_COMPONENT, &[]),
    (&SOLEN_ANT_HOLE, &[]),
    (&SH_TELE_COMPONENT, &[]),
    (&SH_TELEPORTER, &[]),
    (&BALLOT_BOX, &[]),
    (&BALLOT_BOX_ADDON, &[]),
    (&BALLOT_BOX_DEED, &[]),
];

pub fn register(types: &mut TypeRegistry) {
    for (t, aliases) in ALL {
        types.register_item(t, aliases);
    }
    types.register_phase_default(
        Phase::Configure,
        "Server.Items.SHTeleporter.Configure",
        sh_teleporter_configure,
    );
}
