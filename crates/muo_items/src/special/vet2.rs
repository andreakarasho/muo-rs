//! Veteran rewards (Banner, FlamingHead) and the mini houses with ContestMiniHouse. See `vet.rs` for the shared parts.

use muo_content::gumps::base::{
    DynamicGumpLogic, DynamicGumpObj, RelayInfo, StaticGumpLogic, StaticGumpObj,
};
use muo_content::gumps::builder::{DynamicGumpBuilder, StaticGumpBuilder};
use muo_content::gumps::{GumpButtonType, send_gump_to_mobile};
use muo_content::items::addon::{
    self, AddonClass, BASE_ADDON, BASE_ADDON_CLASS, BASE_ADDON_DEED, DeedClass,
};
use muo_content::items::house_deps::{self, SimpleAddon};
use muo_content::items::interfaces::register_dyable;
use muo_primitives::Point3D;
use muo_server::World;
use muo_server::ext::Ext;
use muo_server::ids::{ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType, LootType, ops};
use muo_server::map::MapId;
use muo_server::net_state::NetStateId;
use muo_server::target::Targeted;
use muo_server::type_registry::TypeRegistry;

use super::util::{in_pack, msg, overhead};
use super::vet::*;

// ---- Banner ----

/// `Banner(itemID)`.
pub fn banner_new(w: &mut World, item_id: i32) -> ItemId {
    let id = ops::new_item_with_id(w, &BANNER, item_id);
    ops::set_movable(w, id, false);
    blessed(w, id)
}

fn banner_could_fit(w: &mut World, id: ItemId, p: Point3D, map: MapId) -> bool {
    let (item_id, height) = {
        let Some(i) = w.item(id) else { return false };
        (i.item_id, item_height(w, i.item_id))
    };
    if !muo_server::map::fit::can_fit_item(w, map, p.x, p.y, p.z, height) {
        return false;
    }
    if item_id & 1 == 0 {
        addon::is_wall(w, p.x, p.y - 1, p.z, Some(map)) // north wall
    } else {
        addon::is_wall(w, p.x - 1, p.y, p.z, Some(map)) // west wall
    }
}

fn deed_for_banner(w: &mut World, id: ItemId) -> ItemId {
    let d = banner_deed_new(w);
    deed_with_reward(w, id, d)
}

rtype! {
    /// `Server.Items.Banner` (`IAddon`, `IDyable`; `Banner(int itemID)` only).
    BANNER = "Server.Items.Banner" : ITEM_BASE;
    construct: None,
    force_show_properties: |w, _| w.core.opl_enabled,
    get_properties: |w, id, list| {
        (ITEM_BASE.get_properties)(w, id, list);
        year_ml(w, id, list, 1076218); // 2nd Year Veteran Reward
    },
    on_double_click: |w, id, from| {
        if !near(w, id, from) {
            overhead(w, from, 0x3B2, 1019045); // I can't reach that.
            return;
        }
        let house = muo_multis::house::access::find_house_at_item(w, id);
        if house.is_some_and(|h| muo_multis::house::access::is_owner(w, h, from)) {
            demolition_gump(w, from, id, 1018318); // Do you wish to re-deed this banner?
        } else {
            msg(w, from, 1018330); // You can only re-deed a banner if you placed it or you are the owner of the house.
        }
    },
}

/// `BannerDeed()`.
pub fn banner_deed_new(w: &mut World) -> ItemId {
    let id = crate::simple::from_template(w, &BANNER_DEED, "Server.Items.BannerDeed");
    blessed(w, id)
}

const BANNER_START: i32 = 0x15AE;
const BANNER_END: i32 = 0x15F4;

/// `BannerDeed.BannerGump`.
struct BannerGump {
    deed: ItemId,
}

impl StaticGumpLogic for BannerGump {
    const TYPE_NAME: &'static str = "Server.Items.BannerDeed+BannerGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut StaticGumpBuilder) {
        b.add_page(0);
        b.add_background(25, 0, 520, 230, 0xA28);
        b.add_label(70, 12, 0x3E3, "Choose a Banner:");
        let mut item_id = BANNER_START;
        for i in 1..=4 {
            b.add_page(i);
            for j in 0..8 {
                b.add_item(50 + 60 * j, 70, item_id, 0);
                b.add_button(50 + 60 * j, 50, 0x845, 0x846, item_id);
                item_id += 2;
            }
            if i > 1 {
                b.add_button_type(75, 198, 0x8AF, 0x8AF, 0, GumpButtonType::Page, i - 1);
            }
            if i < 4 {
                b.add_button_type(475, 198, 0x8B0, 0x8B0, 0, GumpButtonType::Page, i + 1);
            }
        }
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        if ops::is_deleted(w, self.deed) {
            return;
        }
        let Some(m) = mobile_of(w, sender) else {
            return;
        };
        if !(BANNER_START..=BANNER_END).contains(&info.button_id) || info.button_id & 1 != 0 {
            return;
        }
        msg(w, m, 1042037); // Where would you like to place this banner?
        let (deed, item_id) = (self.deed, info.button_id);
        begin_target(w, m, move |w, from, t| {
            banner_on_target(w, deed, item_id, from, t)
        });
    }
}

fn banner_on_target(w: &mut World, deed: ItemId, item_id: i32, from: MobileId, targeted: Targeted) {
    if ops::is_deleted(w, deed) {
        return;
    }
    if !in_pack(w, deed, from) {
        msg(w, from, 1042038); // You must have the object in your backpack to use it.
        return;
    }
    if owned_house_at_mobile(w, from).is_none() {
        msg(w, from, 502092); // You must be in your house to do this.
        return;
    }
    let Some(p) = target_point(w, targeted) else {
        return;
    };
    let Some(map) = w.mobile(from).and_then(|m| m.map) else {
        return;
    };
    let height = item_height(w, item_id);
    if !muo_server::map::fit::can_fit_point(w, map, p, height) {
        msg(w, from, 500269); // You cannot build that there.
        return;
    }
    let Some(house) = owned_house_at_point(w, from, p, map, height) else {
        msg(w, from, 1042036); // That location is not in your house.
        return;
    };
    let north = addon::is_wall(w, p.x, p.y - 1, p.z, Some(map));
    let west = addon::is_wall(w, p.x - 1, p.y, p.z, Some(map));
    if north && west {
        let g = BannerFacingGump {
            deed,
            item_id,
            location: p,
            house,
        };
        send_gump_to_mobile(w, from, Box::new(DynamicGumpObj::new(150, 50, g)), false);
    } else if north || west {
        place_banner(w, deed, item_id + if west { 0 } else { 1 }, house, p, map);
    } else {
        msg(w, from, 1042039); // The banner must be placed next to a wall.
    }
}

fn place_banner(w: &mut World, deed: ItemId, item_id: i32, house: ItemId, p: Point3D, map: MapId) {
    let banner = banner_new(w, item_id);
    addon::house_addons_add(w, house, banner);
    set_reward(w, banner, is_reward(w, deed));
    ops::move_to_world_map(w, banner, p, Some(map));
    super::util::delete(w, deed);
}

/// `BannerDeed.InternalTarget.FacingGump`.
struct BannerFacingGump {
    deed: ItemId,
    item_id: i32,
    location: Point3D,
    house: ItemId,
}

impl DynamicGumpLogic for BannerFacingGump {
    const TYPE_NAME: &'static str = "Server.Items.BannerDeed+InternalTarget+FacingGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        b.set_no_resize();
        b.add_page(0);
        b.add_background(0, 0, 300, 150, 0xA28);
        b.add_item(90, 30, self.item_id + 1, 0);
        b.add_item(180, 30, self.item_id, 0);
        b.add_button(50, 35, 0x868, 0x869, 1); // East
        b.add_button(145, 35, 0x868, 0x869, 2); // South
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        if ops::is_deleted(w, self.deed) {
            return;
        }
        let Some(m) = mobile_of(w, sender) else {
            return;
        };
        let map = w.mobile(m).and_then(|x| x.map);
        let banner = banner_new(w, self.item_id + if info.button_id == 1 { 1 } else { 0 });
        addon::house_addons_add(w, self.house, banner);
        set_reward(w, banner, is_reward(w, self.deed));
        ops::move_to_world_map(w, banner, self.location, map);
        super::util::delete(w, self.deed);
    }
}

rtype! {
    /// `Server.Items.BannerDeed`.
    BANNER_DEED = "Server.Items.BannerDeed" : ITEM_BASE;
    construct: Some(banner_deed_new),
    default_weight: crate::class_props::default_weight,
    label_number: crate::class_props::label_number, // a banner deed
    get_properties: |w, id, list| {
        (ITEM_BASE.get_properties)(w, id, list);
        year(w, id, list, 1076218); // 2nd Year Veteran Reward
    },
    on_double_click: |w, id, from| {
        if reward_blocked(w, from, id) {
            return;
        }
        if in_pack(w, id, from) {
            if owned_house_at_mobile(w, from).is_some() {
                if super::util::net_state(w, from).is_some() {
                    send_gump_to_mobile(w, from, Box::new(StaticGumpObj::new(100, 200, BannerGump { deed: id })), false);
                }
            } else {
                msg(w, from, 502092); // You must be in your house to do this.
            }
        } else {
            msg(w, from, 1042038); // You must have the object in your backpack to use it.
        }
    },
}

// ---- FlamingHead ----

/// `StoneFaceTrapType` of the head's item id.
fn head_type(item_id: i32) -> i32 {
    match item_id {
        0x10F5..=0x10F7 => 0,
        0x10FC..=0x10FE => 1,
        _ => 2,
    }
}

use crate::traps::trap_classes::STONE_FACE_TRAP_NO_DAMAGE;

/// `FlamingHead(type)`.
pub fn flaming_head_new(w: &mut World, kind: i32) -> ItemId {
    let id = crate::simple::from_template(w, &FLAMING_HEAD, "Server.Items.FlamingHead");
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(crate::traps::trap_classes::TrapData::default());
    }
    ops::set_movable(w, id, false);
    ops::set_light(w, id, muo_server::light_type::LightType::Circle225);
    ops::set_loot_type(w, id, LootType::Blessed);
    super::item_id_of(w, id, crate::traps::trap_classes::stone_base_id(kind));
    id
}

fn head_could_fit(w: &mut World, id: ItemId, p: Point3D, map: MapId) -> bool {
    let kind = head_type(w.item(id).map_or(0, |i| i.item_id));
    let height = i32::from(ops::item_data(w, id).height);
    if !muo_server::map::fit::can_fit_item(w, map, p.x, p.y, p.z, height) {
        return false;
    }
    let (n, wst) = (p.y - 1, p.x - 1);
    match kind {
        0 => {
            addon::is_wall(w, p.x, n, p.z, Some(map)) && addon::is_wall(w, wst, p.y, p.z, Some(map))
        }
        1 => addon::is_wall(w, p.x, n, p.z, Some(map)),
        2 => addon::is_wall(w, wst, p.y, p.z, Some(map)),
        _ => false,
    }
}

fn deed_for_head(w: &mut World, id: ItemId) -> ItemId {
    let d = flaming_head_deed_new(w);
    deed_with_reward(w, id, d)
}

rtype! {
    /// `Server.Items.FlamingHead` (`StoneFaceTrapNoDamage`, `IAddon`; `FlamingHead(type = NorthWall)`).
    FLAMING_HEAD = "Server.Items.FlamingHead" : STONE_FACE_TRAP_NO_DAMAGE;
    construct: Some(|w| flaming_head_new(w, 1)),
    label_number: crate::class_props::label_number, // Flaming Head
    force_show_properties: |w, _| w.core.opl_enabled,
    get_properties: |w, id, list| {
        (STONE_FACE_TRAP_NO_DAMAGE.get_properties)(w, id, list);
        year_ml(w, id, list, 1076218); // 2nd Year Veteran Reward
    },
    on_double_click: |w, id, from| {
        if !near(w, id, from) {
            overhead(w, from, 0x3B2, 1019045); // I can't reach that.
            return;
        }
        let house = muo_multis::house::access::find_house_at_item(w, id);
        if house.is_some_and(|h| muo_multis::house::access::is_owner(w, h, from)) {
            demolition_gump(w, from, id, 1018329); // Do you wish to re-deed this skull?
        } else {
            msg(w, from, 1018328); // You can only re-deed a skull if you placed it or you are the owner of the house.
        }
    },
}

/// `FlamingHeadDeed()`.
pub fn flaming_head_deed_new(w: &mut World) -> ItemId {
    let id = crate::simple::from_template(w, &FLAMING_HEAD_DEED, "Server.Items.FlamingHeadDeed");
    blessed(w, id)
}

fn head_on_target(w: &mut World, deed: ItemId, from: MobileId, targeted: Targeted) {
    if ops::is_deleted(w, deed) {
        return;
    }
    if !in_pack(w, deed, from) {
        msg(w, from, 1042038); // You must have the object in your backpack to use it.
        return;
    }
    if owned_house_at_mobile(w, from).is_none() {
        msg(w, from, 502115); // You must be in your house to do this.
        return;
    }
    let Some(p) = target_point(w, targeted) else {
        return;
    };
    let Some(map) = w.mobile(from).and_then(|m| m.map) else {
        return;
    };
    let height = item_height(w, 0x10F5);
    let Some(house) = owned_house_at_point(w, from, p, map, height) else {
        msg(w, from, 1042036); // That location is not in your house.
        return;
    };
    if !muo_server::map::fit::can_fit_point(w, map, p, height) {
        msg(w, from, 1042266); // The head must be placed next to a wall.
        return;
    }
    let north = addon::is_wall(w, p.x, p.y - 1, p.z, Some(map));
    let west = addon::is_wall(w, p.x - 1, p.y, p.z, Some(map));
    if !(north || west) {
        msg(w, from, 1042266); // The head must be placed next to a wall.
        return;
    }
    let facing = if north && west {
        0
    } else if north {
        1
    } else {
        2
    };
    let head = flaming_head_new(w, facing);
    set_reward(w, head, is_reward(w, deed));
    super::util::delete(w, deed);
    addon::house_addons_add(w, house, head);
    ops::move_to_world_map(w, head, p, Some(map));
}

rtype! {
    /// `Server.Items.FlamingHeadDeed`.
    FLAMING_HEAD_DEED = "Server.Items.FlamingHeadDeed" : ITEM_BASE;
    construct: Some(flaming_head_deed_new),
    default_weight: crate::class_props::default_weight,
    label_number: crate::class_props::label_number, // a flaming head deed
    get_properties: |w, id, list| {
        (ITEM_BASE.get_properties)(w, id, list);
        year(w, id, list, 1076218); // 2nd Year Veteran Reward
    },
    on_double_click: |w, id, from| {
        if reward_blocked(w, from, id) {
            return;
        }
        if in_pack(w, id, from) {
            if owned_house_at_mobile(w, from).is_some() {
                msg(w, from, 1042264); // Where would you like to place this head?
                begin_target(w, from, move |w, from, t| head_on_target(w, id, from, t));
            } else {
                msg(w, from, 502115); // You must be in your house to do this.
            }
        } else {
            msg(w, from, 1042038); // You must have the object in your backpack to use it.
        }
    },
}

// ---- MiniHouses and ContestMiniHouse ----

/// `MiniHouseAddon._type` / `MiniHouseDeed._type`.
#[derive(Clone, Copy, Debug, Default)]
pub struct MiniType(pub i32);

pub fn mini_type(w: &World, id: ItemId) -> i32 {
    w.item(id)
        .and_then(|i| i.ext.get::<MiniType>())
        .map_or(0, |t| t.0)
}

pub fn set_mini_type_raw(w: &mut World, id: ItemId, t: i32) {
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(MiniType(t));
    }
}

/// `MiniHouseInfo.GetInfo(type)`: (graphics, label).
fn mini_info(kind: i32) -> (Vec<i32>, i32) {
    fn run(start: i32, count: i32, label: i32) -> (Vec<i32>, i32) {
        ((0..count).map(|i| start + i).collect(), label)
    }
    match kind {
        1 => run(0x22DE, 1, 1011304),
        2 => run(0x22DF, 1, 1011305),
        3 => run(0x22C9, 1, 1011306),
        4 => run(0x22E0, 1, 1011307),
        5 => run(0x22E1, 1, 1011308),
        6 => (vec![0x22CD, 0x22CB, 0x22CC, 0x22CA], 1011309),
        7 => (vec![0x2301, 0x2302, 0x2304, 0x2303], 1011310),
        8 => (vec![0x22FC, 0x22FD, 0x22FF, 0x22FE], 1011311),
        9 => (vec![0x22F7, 0x22F8, 0x22FA, 0x22F9], 1011312),
        10 => run(0x22E6, 9, 1011313),
        11 => (
            vec![
                0x22CE, 0x22D0, 0x22D2, 0x22D7, 0x22CF, 0x22D1, 0x22D4, 0x22D9, 0x22D3, 0x22D5,
                0x22D6, 0x22DB, 0x22D8, 0x22DA, 0x22DC, 0x22DD,
            ],
            1011314,
        ),
        12 => run(0x22E2, 4, 1011315),
        13 => run(0x22EF, 4, 1011316),
        14 => run(0x22F5, 1, 1011317),
        15 => run(0x22FB, 1, 1011318),
        16 => run(0x2300, 1, 1011319),
        17 => run(0x22F3, 1, 1011320),
        18 => run(0x22F6, 1, 1011321),
        19 => run(0x22F4, 1, 1011322),
        20 => (vec![0x2316, 0x2315, 0x2314, 0x2313], 1062692),
        21 => (vec![0x2318, 0x2317, 0x2319, 0x1], 1072215),
        _ => run(0x22C4, 1, 1011303),
    }
}

/// `MiniHouseAddon.Construct()`.
pub fn mini_construct(w: &mut World, id: ItemId) {
    for c in addon::components(w, id) {
        if let Some(d) = addon::component_data_mut(w, c) {
            d.addon = None;
        }
        super::util::delete(w, c);
    }
    if let Some(d) = addon::addon_data_mut(w, id) {
        d.components.clear();
    }
    let (graphics, _) = mini_info(mini_type(w, id));
    let size = (graphics.len() as f64).sqrt() as i32;
    let mut num = 0usize;
    for y in 0..size {
        for x in 0..size {
            if graphics[num] != 0x1 {
                // Veteran Rewards Mod
                let c = addon::new_component(w, &addon::ADDON_COMPONENT, graphics[num]);
                num += 1;
                addon::add_component(w, id, c, size - x - 1, size - y - 1, 0);
            }
        }
    }
}

/// `MiniHouseAddon(type)` of class `ty`.
fn mini_new(w: &mut World, ty: &'static ItemType, kind: i32) -> ItemId {
    let id = addon::new_addon(w, ty);
    set_mini_type_raw(w, id, kind);
    mini_construct(w, id);
    id
}

/// `MiniHouseAddon(type)`.
pub fn mini_house_addon_new(w: &mut World, kind: i32) -> ItemId {
    mini_new(w, &MINI_HOUSE_ADDON, kind)
}

fn mini_deed_of(w: &mut World, a: ItemId) -> ItemId {
    let kind = mini_type(w, a);
    mini_house_deed_new(w, kind)
}

static MINI_HOUSE_CLASS: AddonClass = AddonClass {
    deed_of: Some(mini_deed_of),
    ..BASE_ADDON_CLASS
};

/// `Server.Items.MiniHouseAddon` (`[SerializationGenerator(1, false)]`; version 0 had the type as an int).
pub static MINI_HOUSE_ADDON: ItemType = ItemType {
    full_name: "Server.Items.MiniHouseAddon",
    parent: Some(&BASE_ADDON),
    construct: Some(|w| mini_house_addon_new(w, 0)),
    family: Some(&MINI_HOUSE_CLASS),
    serialize: |w, id, out| {
        (BASE_ADDON.serialize)(w, id, out);
        out.write_i32(1);
        out.write_enum(4, i64::from(mini_type(w, id)));
    },
    deserialize: |w, id, r| {
        (BASE_ADDON.deserialize)(w, id, r)?;
        let version = r.read_i32()?;
        let t = if version < 1 {
            r.read_i32()?
        } else {
            r.read_enum(4)? as i32
        };
        set_mini_type_raw(w, id, t);
        Ok(())
    },
    ..BASE_ADDON
};

/// `MiniHouseDeed(type)`.
pub fn mini_house_deed_new(w: &mut World, kind: i32) -> ItemId {
    let id = addon::new_deed(w, &MINI_HOUSE_DEED);
    set_mini_type_raw(w, id, kind);
    blessed(w, id)
}

static MINI_HOUSE_DEED_CLASS: DeedClass = DeedClass {
    addon: |w, deed| {
        let kind = mini_type(w, deed);
        mini_house_addon_new(w, kind)
    },
    select_direction: None,
};

/// `Server.Items.MiniHouseDeed`.
pub static MINI_HOUSE_DEED: ItemType = ItemType {
    full_name: "Server.Items.MiniHouseDeed",
    parent: Some(&BASE_ADDON_DEED),
    construct: Some(|w| mini_house_deed_new(w, 0)),
    family: Some(&MINI_HOUSE_DEED_CLASS),
    default_weight: crate::class_props::default_weight,
    label_number: crate::class_props::label_number, // a mini house deed
    get_properties: |w, id, list| {
        (BASE_ADDON_DEED.get_properties)(w, id, list);
        list.add(mini_info(mini_type(w, id)).1);
    },
    serialize: |w, id, out| {
        (BASE_ADDON_DEED.serialize)(w, id, out);
        out.write_i32(1);
        out.write_enum(4, i64::from(mini_type(w, id)));
    },
    deserialize: |w, id, r| {
        (BASE_ADDON_DEED.deserialize)(w, id, r)?;
        let version = r.read_i32()?;
        let t = if version < 1 {
            r.read_i32()?
        } else {
            r.read_enum(4)? as i32
        };
        set_mini_type_raw(w, id, t);
        Ok(())
    },
    ..BASE_ADDON_DEED
};

/// `ContestMiniHouse(type)`.
pub fn contest_mini_house_new(w: &mut World, kind: i32) -> ItemId {
    mini_new(w, &CONTEST_MINI_HOUSE, kind)
}

/// `ContestMiniHouseDeed(type)`.
pub fn contest_mini_house_deed_new(w: &mut World, kind: i32) -> ItemId {
    let id = addon::new_deed(w, &CONTEST_MINI_HOUSE_DEED);
    set_mini_type_raw(w, id, kind);
    blessed(w, id)
}

fn contest_deed_of(w: &mut World, a: ItemId) -> ItemId {
    let kind = mini_type(w, a);
    let d = contest_mini_house_deed_new(w, kind);
    deed_with_reward(w, a, d)
}

static CONTEST_CLASS: AddonClass = AddonClass {
    deed_of: Some(contest_deed_of),
    ..BASE_ADDON_CLASS
};

rtype! {
    /// `Server.Items.ContestMiniHouse`.
    CONTEST_MINI_HOUSE = "Server.Items.ContestMiniHouse" : MINI_HOUSE_ADDON;
    construct: Some(|w| contest_mini_house_new(w, 20)),
    family: Some(&CONTEST_CLASS),
}

static CONTEST_DEED_CLASS: DeedClass = DeedClass {
    addon: |w, deed| {
        let kind = mini_type(w, deed);
        let a = contest_mini_house_new(w, kind);
        with_reward(w, deed, a)
    },
    select_direction: None,
};

rtype! {
    /// `Server.Items.ContestMiniHouseDeed`.
    CONTEST_MINI_HOUSE_DEED = "Server.Items.ContestMiniHouseDeed" : MINI_HOUSE_DEED;
    construct: Some(|w| contest_mini_house_deed_new(w, 20)),
    family: Some(&CONTEST_DEED_CLASS),
    on_double_click: addon_deed_double_click,
    get_properties: |w, id, list| {
        (MINI_HOUSE_DEED.get_properties)(w, id, list);
        year_ml(w, id, list, 1076217); // 1st Year Veteran Reward
    },
}

// ---- registration ----

pub fn register(types: &mut TypeRegistry) {
    for t in [
        &BANNER,
        &BANNER_DEED,
        &FLAMING_HEAD,
        &FLAMING_HEAD_DEED,
        &MINI_HOUSE_ADDON,
        &MINI_HOUSE_DEED,
        &CONTEST_MINI_HOUSE,
        &CONTEST_MINI_HOUSE_DEED,
    ] {
        types.register_item(t, &[]);
    }
}

/// Configure phase: the `IAddon` classes that are plain items, `IDyable` and the overloads.
pub(super) fn configure(w: &mut World) {
    let d = house_deps::deps_mut(w);
    d.simple_addons.push(SimpleAddon {
        is: |w, id| w.item(id).is_some_and(|i| i.ty.is(&BANNER)),
        deed: deed_for_banner,
        could_fit: banner_could_fit,
    });
    d.simple_addons.push(SimpleAddon {
        is: |w, id| w.item(id).is_some_and(|i| i.ty.is(&FLAMING_HEAD)),
        deed: deed_for_head,
        could_fit: head_could_fit,
    });
    register_dyable(w, &BANNER, |w, id, _from, tub| {
        if ops::is_deleted(w, id) {
            return false;
        }
        let hue = muo_content::items::clothing::tub_dyed_hue(w, tub);
        super::hue(w, id, hue);
        true
    });
}

/// `MiniHouseAddon.Type = value`: `_type = value; Construct();`.
pub fn set_mini_addon_type(w: &mut World, id: ItemId, kind: i32) {
    set_mini_type_raw(w, id, kind);
    mini_construct(w, id);
}
