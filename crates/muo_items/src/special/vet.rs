//! Veteran rewards (the first half) and the mini houses: the reward addons, deeds and
//! the decorations the reward gumps hand out.
//!
//! `IsRewardItem` is one [`Reward`] in `Item::ext` of every class here. The gumps of the reward system
//! (`RewardOptionGump`, `RewardDemolitionGump`) belong to the veteran rewards engine: the deeds that use them call
//! [`SpecialHooks::reward_option_gump`] / [`SpecialHooks::reward_demolition_gump`] (unset: nothing opens), and
//! `IRewardOption` is [`reward_options`] / [`reward_option_selected`].
//!
//! `IsRewardItem` is a `[CommandProperty]` of each class (tables registered in `vet5::configure`).

use std::rc::Rc;

use muo_content::gumps::base::{RelayInfo, StaticGumpLogic, StaticGumpObj};
use muo_content::gumps::builder::StaticGumpBuilder;
use muo_content::gumps::send_gump_to_mobile;
use muo_content::items::addon::{
    self, ADDON_COMPONENT, AddonClass, BASE_ADDON, BASE_ADDON_CLASS, BASE_ADDON_DEED, DeedClass,
};
use muo_content::items::house_deps::{self};

use muo_primitives::{DateTime, Point3D, utility};
use muo_server::World;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType, LootType, deps, ops};
use muo_server::map::MapId;
use muo_server::net_state::NetStateId;
use muo_server::packets::message::MessageType;
use muo_server::property_list::PropertyList;
use muo_server::serialization_ext::{ReadEntityExt, WriteEntityExt};
use muo_server::target::{self, TargetFlags, Targeted};
use muo_server::type_registry::TypeRegistry;

use super::hooks::hooks;
use super::util::{in_pack, msg, overhead, reward_usable_by};
use crate::simple::leaf;

// ---- IRewardItem ----

/// `IRewardItem.IsRewardItem`.
#[derive(Clone, Copy, Debug, Default)]
pub struct Reward(pub bool);

/// `IsRewardItem`.
pub fn is_reward(w: &World, id: ItemId) -> bool {
    w.item(id)
        .and_then(|i| i.ext.get::<Reward>())
        .is_some_and(|r| r.0)
}

/// `IsRewardItem = value` (`[InvalidateProperties]`).
pub fn set_reward(w: &mut World, id: ItemId, value: bool) {
    if is_reward(w, id) == value {
        return;
    }
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(Reward(value));
    }
    ops::invalidate_properties(w, id);
}

pub(super) fn set_reward_raw(w: &mut World, id: ItemId, value: bool) {
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(Reward(value));
    }
}

/// `_isRewardItem && !RewardSystem.CheckIsUsableBy(from, this)`: true when the item may not be used.
pub fn reward_blocked(w: &mut World, from: MobileId, id: ItemId) -> bool {
    is_reward(w, id) && !reward_usable_by(w, from, id)
}

pub(super) fn year(w: &World, id: ItemId, list: &mut PropertyList, label: i32) {
    if is_reward(w, id) {
        list.add(label);
    }
}

pub(super) fn year_ml(w: &World, id: ItemId, list: &mut PropertyList, label: i32) {
    if deps::core_ml(w) && is_reward(w, id) {
        list.add(label);
    }
}

/// An item class with `_isRewardItem` as its own first field: `[SerializationGenerator(0)]`.
macro_rules! rtype {
    (
        $(#[$m:meta])*
        $name:ident = $full:literal : $parent:ident;
        $($f:ident : $v:expr,)*
    ) => {
        $(#[$m])*
        pub static $name: ItemType = ItemType {
            full_name: $full,
            parent: Some(&$parent),
            $($f: $v,)*
            serialize: |w, id, out| {
                ($parent.serialize)(w, id, out);
                out.write_encoded_int(0);
                out.write_bool(is_reward(w, id));
            },
            deserialize: |w, id, r| {
                ($parent.deserialize)(w, id, r)?;
                let _version = r.read_encoded_int()?;
                let flag = r.read_bool()?;
                set_reward_raw(w, id, flag);
                Ok(())
            },
            ..$parent
        };
    };
}
pub(super) use rtype;

// ---- IRewardOption ----

/// `RewardOptionList` of an item: `(number, cliloc)` rows.
pub fn reward_options(w: &World, id: ItemId) -> Option<&'static [(i32, i32)]> {
    let ty = w.item(id)?.ty;
    OPTIONS.iter().find(|o| ty.is(o.0)).map(|o| o.1)
}

/// `OnOptionSelected(from, option)`.
pub fn reward_option_selected(w: &mut World, id: ItemId, from: MobileId, option: i32) {
    let Some(ty) = w.item(id).map(|i| i.ty) else {
        return;
    };
    if let Some(o) = OPTIONS.iter().find(|o| ty.is(o.0)) {
        (o.2)(w, id, from, option);
    }
}

type OptionRow = (
    &'static ItemType,
    &'static [(i32, i32)],
    fn(&mut World, ItemId, MobileId, i32),
);

static OPTIONS: [OptionRow; 5] = [
    (
        &super::vet5::CANNON_DEED,
        &[(2, 1075386), (1, 1075387), (0, 1075389), (3, 1075390)],
        |w, id, from, o| {
            deed_option(w, id, from, |d| d.statue = o);
        },
    ),
    (
        &MINOTAUR_STATUE_DEED,
        &[
            (100, 1080410),
            (101, 1080411),
            (102, 1080412),
            (103, 1080413),
        ],
        |w, id, from, o| {
            deed_option(w, id, from, |d| d.statue = o);
        },
    ),
    (
        &super::vet3::MINING_CART_DEED,
        &[
            (100, 1080391),
            (101, 1080390),
            (102, 1080500),
            (103, 1080499),
        ],
        |w, id, from, o| {
            deed_option(w, id, from, |d| d.statue = o);
        },
    ),
    (
        &super::vet3::ANKH_OF_SACRIFICE_DEED,
        &[(1, 1080398), (2, 1080399)],
        |w, id, from, o| {
            deed_option(w, id, from, |d| match o {
                1 => d.east = false,
                2 => d.east = true,
                _ => {}
            });
        },
    ),
    (
        &TREE_STUMP_DEED,
        &[(1, 1080403), (2, 1080404), (3, 1080401), (4, 1080402)],
        |w, id, from, o| {
            let item_id = match o {
                1 => 0xE56,
                2 => 0xE58,
                3 => 0xE57,
                4 => 0xE59,
                _ => deed_state(w, id).item_id,
            };
            deed_option(w, id, from, |d| d.item_id = item_id);
        },
    ),
];

/// What a reward deed keeps that is not serialized (`_east`, `_statueType`, `_itemID`), and `Logs` of the stump deed.
#[derive(Clone, Copy, Debug, Default)]
pub struct DeedState {
    pub east: bool,
    pub statue: i32,
    pub item_id: i32,
    pub logs: i32,
    /// `MiningCartDeed` `Gems` and `Ore`.
    pub gems: i32,
    pub ore: i32,
}

pub fn deed_state(w: &World, id: ItemId) -> DeedState {
    w.item(id)
        .and_then(|i| i.ext.get::<DeedState>())
        .copied()
        .unwrap_or_default()
}

pub(super) fn deed_option(
    w: &mut World,
    id: ItemId,
    from: MobileId,
    set: impl FnOnce(&mut DeedState),
) {
    let mut s = deed_state(w, id);
    set(&mut s);
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(s);
    }
    if !ops::is_deleted(w, id) {
        addon::deed_on_double_click(w, id, from); // base.OnDoubleClick(from)
    }
}

/// `RewardOptionGump.DisplayTo(from, deed)`.
pub(super) fn option_gump(w: &mut World, from: MobileId, deed: ItemId) {
    if let Some(f) = hooks(w).and_then(|h| h.reward_option_gump) {
        f(w, from, deed);
    }
}

/// `RewardDemolitionGump.DisplayTo(from, item, label)`.
pub(super) fn demolition_gump(w: &mut World, from: MobileId, item: ItemId, label: i32) {
    if let Some(f) = hooks(w).and_then(|h| h.reward_demolition_gump) {
        f(w, from, item, label);
    }
}

// ---- helpers ----

pub(super) fn mobile_of(w: &World, sender: NetStateId) -> Option<MobileId> {
    w.net_states.get(&sender).and_then(|n| n.mobile)
}

/// `new Point3D(targeted as IPoint3D)`.
pub fn target_point(w: &World, t: Targeted) -> Option<Point3D> {
    match t {
        Targeted::Land(l) => Some(l.location),
        Targeted::Static(s) => Some(s.location),
        Targeted::Entity(EntityId::Item(i)) => w.item(i).map(|x| x.location),
        Targeted::Entity(EntityId::Mobile(m)) => w.mobile(m).map(|x| x.location),
    }
}

/// `from.Target = new InternalTarget(..)` with `base(-1, true, TargetFlags.None)`.
pub fn begin_target(
    w: &mut World,
    from: MobileId,
    on_target: impl Fn(&mut World, MobileId, Targeted) + 'static,
) {
    let mut t = target::Target::new(w, -1, true, TargetFlags::NONE);
    t.handlers.on_target = Some(Rc::new(on_target));
    let tid = target::add_target(w, t);
    target::set_mobile_target(w, from, Some(tid));
}

/// `BaseHouse.FindHouseAt(m)` and `house.IsOwner(m)`.
pub(super) fn owned_house_at_mobile(w: &World, m: MobileId) -> Option<ItemId> {
    muo_multis::house::access::find_house_at_mobile(w, m)
        .filter(|&h| muo_multis::house::access::is_owner(w, h, m))
}

pub(super) fn owned_house_at_point(
    w: &World,
    m: MobileId,
    p: Point3D,
    map: MapId,
    height: i32,
) -> Option<ItemId> {
    muo_multis::house::access::find_house_at_point(w, p, Some(map), height)
        .filter(|&h| muo_multis::house::access::is_owner(w, h, m))
}

pub(super) fn item_height(w: &World, item_id: i32) -> i32 {
    i32::from(w.maps.tile_data.item((item_id & 0x3FFF) as usize).height)
}

/// `from.InRange(item.Location, 2)`.
pub(super) fn near(w: &World, id: ItemId, from: MobileId) -> bool {
    let p = w.item(id).map_or_else(Point3D::default, |i| i.location);
    muo_content::mx::in_range_point(w, from, p, 2)
}

pub(super) fn place_or_delete(w: &mut World, from: MobileId, item: ItemId) -> bool {
    if muo_server::mobile::part2::items::place_in_backpack(w, from, item) {
        true
    } else {
        super::util::delete(w, item);
        msg(w, from, 1078837); // Your backpack is full! Please make room and try again.
        false
    }
}

pub(super) fn blessed(w: &mut World, id: ItemId) -> ItemId {
    ops::set_loot_type(w, id, LootType::Blessed);
    id
}

/// A reward deed's `Addon`: builds it and copies `IsRewardItem`.
pub(super) fn with_reward(w: &mut World, deed: ItemId, a: ItemId) -> ItemId {
    let r = is_reward(w, deed);
    set_reward(w, a, r);
    a
}

pub(super) fn deed_with_reward(w: &mut World, addon_id: ItemId, d: ItemId) -> ItemId {
    let r = is_reward(w, addon_id);
    set_reward(w, d, r);
    d
}

/// The `OnDoubleClick` of a plain `BaseAddonDeed` reward: the reward check, then the base.
pub(super) fn addon_deed_double_click(w: &mut World, id: ItemId, from: MobileId) {
    if reward_blocked(w, from, id) {
        return;
    }
    addon::deed_on_double_click(w, id, from);
}

// ---- BloodyPentagram ----

pub(super) fn pentagram_component_new(w: &mut World, item_id: i32) -> ItemId {
    addon::new_component(w, &BLOODY_PENTAGRAM_COMPONENT, item_id)
}

leaf! {
    /// `Server.Items.BloodyPentagramComponent`.
    BLOODY_PENTAGRAM_COMPONENT = "Server.Items.BloodyPentagramComponent" : ADDON_COMPONENT, enc;
    display_weight: |_, _| false,
    label_number: crate::class_props::label_number, // Bloody Pentagram
}

const PENTAGRAM_ROWS: [(i32, i32, i32); 34] = [
    (0x1CF9, 0, 1),
    (0x1CF8, 0, 2),
    (0x1CF7, 0, 3),
    (0x1CF6, 0, 4),
    (0x1CF5, 0, 5),
    (0x1CFB, 1, 0),
    (0x1CFA, 1, 1),
    (0x1D09, 1, 2),
    (0x1D08, 1, 3),
    (0x1D07, 1, 4),
    (0x1CF4, 1, 5),
    (0x1CFC, 2, 0),
    (0x1D0A, 2, 1),
    (0x1D11, 2, 2),
    (0x1D10, 2, 3),
    (0x1D06, 2, 4),
    (0x1CF3, 2, 5),
    (0x1CFD, 3, 0),
    (0x1D0B, 3, 1),
    (0x1D12, 3, 2),
    (0x1D0F, 3, 3),
    (0x1D05, 3, 4),
    (0x1CF2, 3, 5),
    (0x1CFE, 4, 0),
    (0x1D0C, 4, 1),
    (0x1D0D, 4, 2),
    (0x1D0E, 4, 3),
    (0x1D04, 4, 4),
    (0x1CF1, 4, 5),
    (0x1CFF, 5, 0),
    (0x1D00, 5, 1),
    (0x1D01, 5, 2),
    (0x1D02, 5, 3),
    (0x1D03, 5, 4),
];

/// `BloodyPentagramAddon()`.
pub fn bloody_pentagram_addon_new(w: &mut World) -> ItemId {
    let id = addon::new_addon(w, &BLOODY_PENTAGRAM_ADDON);
    for (item_id, x, y) in PENTAGRAM_ROWS {
        let c = pentagram_component_new(w, item_id);
        addon::add_component(w, id, c, x, y, 0);
    }
    id
}

pub(super) fn pentagram_deed_of(w: &mut World, a: ItemId) -> ItemId {
    let d = bloody_pentagram_deed_new(w);
    deed_with_reward(w, a, d)
}

static PENTAGRAM_CLASS: AddonClass = AddonClass {
    deed_of: Some(pentagram_deed_of),
    ..BASE_ADDON_CLASS
};

rtype! {
    /// `Server.Items.BloodyPentagramAddon`.
    BLOODY_PENTAGRAM_ADDON = "Server.Items.BloodyPentagramAddon" : BASE_ADDON;
    construct: Some(bloody_pentagram_addon_new),
    family: Some(&PENTAGRAM_CLASS),
}

/// `BloodyPentagramDeed()`.
pub fn bloody_pentagram_deed_new(w: &mut World) -> ItemId {
    let id = addon::new_deed(w, &BLOODY_PENTAGRAM_DEED);
    blessed(w, id)
}

static PENTAGRAM_DEED_CLASS: DeedClass = DeedClass {
    addon: |w, deed| {
        let a = bloody_pentagram_addon_new(w);
        with_reward(w, deed, a)
    },
    select_direction: None,
};

rtype! {
    /// `Server.Items.BloodyPentagramDeed`.
    BLOODY_PENTAGRAM_DEED = "Server.Items.BloodyPentagramDeed" : BASE_ADDON_DEED;
    construct: Some(bloody_pentagram_deed_new),
    family: Some(&PENTAGRAM_DEED_CLASS),
    label_number: crate::class_props::label_number, // Bloody Pentagram
    on_double_click: addon_deed_double_click,
    get_properties: |w, id, list| {
        (BASE_ADDON_DEED.get_properties)(w, id, list);
        year(w, id, list, 1076221); // 5th Year Veteran Reward
    },
}

// ---- StoneAnkh ----

pub(super) fn stone_ankh_component_new(w: &mut World, item_id: i32) -> ItemId {
    addon::new_component(w, &STONE_ANKH_COMPONENT, item_id)
}

leaf! {
    /// `Server.Items.StoneAnkhComponent`.
    STONE_ANKH_COMPONENT = "Server.Items.StoneAnkhComponent" : ADDON_COMPONENT, enc;
    default_weight: crate::class_props::default_weight,
    force_show_properties: |w, _| w.core.opl_enabled,
    get_properties: |w, id, list| {
        (ADDON_COMPONENT.get_properties)(w, id, list);
        if addon::component_addon(w, id).is_some_and(|a| is_reward(w, a) && w.item(a).is_some_and(|i| i.ty.is(&STONE_ANKH))) {
            list.add(1076221); // 5th Year Veteran Reward
        }
    },
}

/// `StoneAnkh(east)`.
pub fn stone_ankh_new(w: &mut World, east: bool) -> ItemId {
    let id = addon::new_addon(w, &STONE_ANKH);
    let rows: [(i32, i32, i32); 2] = if east {
        [(0x2, 0, 0), (0x3, 0, -1)]
    } else {
        [(0x5, 0, 0), (0x4, -1, 0)]
    };
    for (item_id, x, y) in rows {
        let c = stone_ankh_component_new(w, item_id);
        addon::add_component(w, id, c, x, y, 0);
    }
    id
}

pub(super) fn stone_ankh_deed_of(w: &mut World, a: ItemId) -> ItemId {
    let d = stone_ankh_deed_new(w);
    deed_with_reward(w, a, d)
}

static STONE_ANKH_CLASS: AddonClass = AddonClass {
    deed_of: Some(stone_ankh_deed_of),
    on_chop: Some(|w, _, from| msg(w, from, 500489)), // You can't use an axe on that.
    on_component_used: |w, a, _c, from| {
        if !near(w, a, from) {
            overhead(w, from, 0x3B2, 1019045); // I can't reach that.
            return;
        }
        let house = muo_multis::house::access::find_house_at_item(w, a);
        if house.is_some_and(|h| muo_multis::house::access::is_owner(w, h, from)) {
            demolition_gump(w, from, a, 1049783); // Do you wish to re-deed this decoration?
        } else {
            // You can only re-deed this decoration if you are the house owner or originally placed the decoration.
            msg(w, from, 1049784);
        }
    },
    ..BASE_ADDON_CLASS
};

rtype! {
    /// `Server.Items.StoneAnkh`.
    STONE_ANKH = "Server.Items.StoneAnkh" : BASE_ADDON;
    construct: Some(|w| stone_ankh_new(w, true)),
    family: Some(&STONE_ANKH_CLASS),
    get_properties: |w, id, list| {
        (BASE_ADDON.get_properties)(w, id, list);
        year_ml(w, id, list, 1076221); // 5th Year Veteran Reward
    },
}

/// `StoneAnkhDeed()`.
pub fn stone_ankh_deed_new(w: &mut World) -> ItemId {
    let id = addon::new_deed(w, &STONE_ANKH_DEED);
    blessed(w, id)
}

static STONE_ANKH_DEED_CLASS: DeedClass = DeedClass {
    addon: |w, deed| {
        let east = deed_state(w, deed).east;
        let a = stone_ankh_new(w, east);
        with_reward(w, deed, a)
    },
    select_direction: None,
};

/// `StoneAnkhDeed.StoneAnkhGump`.
struct StoneAnkhGump {
    deed: ItemId,
}

impl StaticGumpLogic for StoneAnkhGump {
    const TYPE_NAME: &'static str = "Server.Items.StoneAnkhDeed+StoneAnkhGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut StaticGumpBuilder) {
        b.add_page(0);
        b.add_background(0, 0, 300, 150, 0xA28);
        b.add_item(90, 30, 0x4, 0);
        b.add_item(112, 30, 0x5, 0);
        b.add_button(50, 35, 0x867, 0x869, 1); // South
        b.add_item(170, 30, 0x2, 0);
        b.add_item(192, 30, 0x3, 0);
        b.add_button(145, 35, 0x867, 0x869, 2); // East
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        if ops::is_deleted(w, self.deed) || info.button_id == 0 {
            return;
        }
        if let Some(m) = mobile_of(w, sender) {
            deed_option(w, self.deed, m, |d| d.east = info.button_id == 2); // SendTarget
        }
    }
}

rtype! {
    /// `Server.Items.StoneAnkhDeed`.
    STONE_ANKH_DEED = "Server.Items.StoneAnkhDeed" : BASE_ADDON_DEED;
    construct: Some(stone_ankh_deed_new),
    family: Some(&STONE_ANKH_DEED_CLASS),
    label_number: crate::class_props::label_number, // deed for a stone ankh
    on_double_click: |w, id, from| {
        if reward_blocked(w, from, id) {
            return;
        }
        if in_pack(w, id, from) {
            send_gump_to_mobile(w, from, Box::new(StaticGumpObj::new(150, 50, StoneAnkhGump { deed: id })), false);
        } else {
            msg(w, from, 1042038); // You must have the object in your backpack to use it.
        }
    },
    get_properties: |w, id, list| {
        (BASE_ADDON_DEED.get_properties)(w, id, list);
        year(w, id, list, 1076221); // 5th Year Veteran Reward
    },
}

// ---- MinotaurStatue ----

/// `MinotaurStatue(type)`.
pub fn minotaur_statue_new(w: &mut World, kind: i32) -> ItemId {
    let id = addon::new_addon(w, &MINOTAUR_STATUE);
    let rows: &[(i32, i32, i32)] = match kind {
        100 => &[(0x306C, 0, 0), (0x306D, -1, 0), (0x306E, 0, -1)],
        101 => &[(0x3074, 0, 0), (0x3075, -1, 0), (0x3076, 0, -1)],
        102 => &[(0x3072, 0, 0), (0x3073, 0, -1)],
        103 => &[(0x306F, 0, 0), (0x3070, -1, 0), (0x3071, 0, -1)],
        _ => &[],
    };
    for &(item_id, x, y) in rows {
        let c = addon::new_component(w, &addon::ADDON_COMPONENT, item_id);
        addon::add_component(w, id, c, x, y, 0);
    }
    id
}

pub(super) fn minotaur_deed_of(w: &mut World, a: ItemId) -> ItemId {
    let d = minotaur_statue_deed_new(w);
    deed_with_reward(w, a, d)
}

static MINOTAUR_CLASS: AddonClass = AddonClass {
    deed_of: Some(minotaur_deed_of),
    ..BASE_ADDON_CLASS
};

rtype! {
    /// `Server.Items.MinotaurStatue` (`MinotaurStatue(MinotaurStatueType type)` only).
    MINOTAUR_STATUE = "Server.Items.MinotaurStatue" : BASE_ADDON;
    construct: None,
    family: Some(&MINOTAUR_CLASS),
}

/// `MinotaurStatueDeed()`.
pub fn minotaur_statue_deed_new(w: &mut World) -> ItemId {
    let id = addon::new_deed(w, &MINOTAUR_STATUE_DEED);
    blessed(w, id)
}

static MINOTAUR_DEED_CLASS: DeedClass = DeedClass {
    addon: |w, deed| {
        let kind = deed_state(w, deed).statue;
        let a = minotaur_statue_new(w, kind);
        with_reward(w, deed, a)
    },
    select_direction: None,
};

rtype! {
    /// `Server.Items.MinotaurStatueDeed`.
    MINOTAUR_STATUE_DEED = "Server.Items.MinotaurStatueDeed" : BASE_ADDON_DEED;
    construct: Some(minotaur_statue_deed_new),
    family: Some(&MINOTAUR_DEED_CLASS),
    label_number: crate::class_props::label_number, // Minotaur Statue Deed
    on_double_click: |w, id, from| {
        if reward_blocked(w, from, id) {
            return;
        }
        if in_pack(w, id, from) {
            option_gump(w, from, id);
        } else {
            msg(w, from, 1062334); // This item must be in your backpack to be used.
        }
    },
    get_properties: |w, id, list| {
        (BASE_ADDON_DEED.get_properties)(w, id, list);
        year(w, id, list, 1076218); // 2nd Year Veteran Reward
    },
}

// ---- TreeStump ----

/// `TreeStump._logs` and `_lastChecked`.
#[derive(Clone, Copy, Debug)]
pub struct StumpData {
    pub logs: i32,
    pub last_checked: DateTime,
}

pub(super) fn stump(w: &World, id: ItemId) -> StumpData {
    w.item(id)
        .and_then(|i| i.ext.get::<StumpData>())
        .copied()
        .unwrap_or(StumpData {
            logs: 0,
            last_checked: DateTime::MIN,
        })
}

pub(super) fn set_stump(w: &mut World, id: ItemId, d: StumpData) {
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(d);
    }
}

/// `TreeStump.Logs = value` (`[InvalidateProperties]`).
pub fn set_stump_logs(w: &mut World, id: ItemId, logs: i32) {
    let mut d = stump(w, id);
    if d.logs != logs {
        d.logs = logs;
        set_stump(w, id, d);
        ops::invalidate_properties(w, id);
    }
}

/// `TreeStump(itemID)`.
pub fn tree_stump_new(w: &mut World, item_id: i32) -> ItemId {
    let id = addon::new_addon(w, &TREE_STUMP);
    let c = addon::new_component(w, &addon::ADDON_COMPONENT, item_id);
    addon::add_component(w, id, c, 0, 0, 0);
    let now = w.now;
    set_stump(
        w,
        id,
        StumpData {
            logs: 0,
            last_checked: now,
        },
    );
    id
}

pub(super) fn stump_update_logs(w: &mut World, id: ItemId) {
    let now = w.now;
    let mut d = stump(w, id);
    let days = now.since(d.last_checked).days();
    d.logs = 100.min(d.logs + days * 10);
    d.last_checked = now;
    set_stump(w, id, d);
}

pub(super) fn stump_deed_of(w: &mut World, a: ItemId) -> ItemId {
    let d = tree_stump_deed_new(w);
    set_reward(w, d, is_reward(w, a));
    let mut s = deed_state(w, d);
    s.logs = stump(w, a).logs;
    if let Some(i) = w.item_mut(d) {
        i.ext.insert(s);
    }
    ops::invalidate_properties(w, d);
    d
}

pub(super) fn stump_component_used(w: &mut World, a: ItemId, _c: ItemId, from: MobileId) {
    let house = muo_multis::house::access::find_house_at_item(w, a);
    let loc = ops::get_world_location(w, a);
    let from_z = w.mobile(from).map_or(0, |m| m.location.z);
    let z = w.item(a).map_or(0, |i| i.location.z);
    if !muo_content::mx::in_range_point(w, from, loc, 2)
        || !deps::in_los_item(w, from, a)
        || !(from_z - z > -3 && from_z - z < 3)
    {
        overhead(w, from, 0x3B2, 1019045); // I can't reach that.
        return;
    }
    if !house.is_some_and(|h| {
        muo_multis::house::access::has_secure_access(
            w,
            h,
            from,
            muo_multis::house::SecureLevel::Friends,
        )
    }) {
        msg(w, from, 1061637); // You are not allowed to access this.
        return;
    }
    stump_update_logs(w, a);
    let logs = stump(w, a).logs;
    if logs <= 0 {
        msg(w, from, 1094720); // There are no more logs available.
        return;
    }
    let name = match utility::random(&mut w.rng, 7) {
        0 => "Server.Items.Log",
        1 => "Server.Items.AshLog",
        2 => "Server.Items.OakLog",
        3 => "Server.Items.YewLog",
        4 => "Server.Items.HeartwoodLog",
        5 => "Server.Items.BloodwoodLog",
        _ => "Server.Items.FrostwoodLog",
    };
    let Some(item) = muo_content::factory::make_item(w, name) else {
        return;
    };
    let amount = logs.min(10);
    ops::set_amount(w, item, amount);
    if !muo_server::mobile::part2::items::place_in_backpack(w, from, item) {
        super::util::delete(w, item);
        msg(w, from, 1078837); // Your backpack is full! Please make room and try again.
    } else {
        let mut d = stump(w, a);
        d.logs -= amount;
        set_stump(w, a, d);
        let text = d.logs.to_string();
        muo_server::item::messages::public_overhead_message_number(
            w,
            a,
            MessageType::REGULAR,
            0,
            1094719,
            &text,
        ); // Logs: ~1_COUNT~
    }
}

static TREE_STUMP_CLASS: AddonClass = AddonClass {
    deed_of: Some(stump_deed_of),
    on_component_used: stump_component_used,
    ..BASE_ADDON_CLASS
};

/// `Server.Items.TreeStump` (`[SerializationGenerator(1)]`: version 0 kept the next check time).
pub static TREE_STUMP: ItemType = ItemType {
    full_name: "Server.Items.TreeStump",
    parent: Some(&BASE_ADDON),
    construct: None,
    family: Some(&TREE_STUMP_CLASS),
    get_properties: |w, id, list| {
        (BASE_ADDON.get_properties)(w, id, list);
        year_ml(w, id, list, 1076223); // 7th Year Veteran Reward
    },
    serialize: |w, id, out| {
        (BASE_ADDON.serialize)(w, id, out);
        out.write_encoded_int(1);
        out.write_bool(is_reward(w, id));
        let d = stump(w, id);
        out.write_i32(d.logs);
        out.write_date_time_ticks(d.last_checked.ticks());
    },
    deserialize: |w, id, r| {
        (BASE_ADDON.deserialize)(w, id, r)?;
        let version = r.read_encoded_int()?;
        let reward = r.read_bool()?;
        let logs = r.read_i32()?;
        let mut last = DateTime::from_ticks(r.read_date_time_ticks()?);
        if version < 1 {
            // Deserialize(reader, 0): the next check time; `_lastChecked = next - 1 day`.
            let mut next = last;
            if next < w.now {
                next = w.now;
            }
            last = next - muo_primitives::TimeSpan::from_days(1);
        }
        set_reward_raw(w, id, reward);
        set_stump(
            w,
            id,
            StumpData {
                logs,
                last_checked: last,
            },
        );
        Ok(())
    },
    ..BASE_ADDON
};

/// `TreeStumpDeed()`.
pub fn tree_stump_deed_new(w: &mut World) -> ItemId {
    let id = addon::new_deed(w, &TREE_STUMP_DEED);
    blessed(w, id)
}

static STUMP_DEED_CLASS: DeedClass = DeedClass {
    addon: |w, deed| {
        let s = deed_state(w, deed);
        let a = tree_stump_new(w, s.item_id);
        set_reward(w, a, is_reward(w, deed));
        set_stump_logs(w, a, s.logs);
        a
    },
    select_direction: None,
};

/// `Server.Items.TreeStumpDeed` (`Logs` is `[SerializableField(1)]`).
pub static TREE_STUMP_DEED: ItemType = ItemType {
    full_name: "Server.Items.TreeStumpDeed",
    parent: Some(&BASE_ADDON_DEED),
    construct: Some(tree_stump_deed_new),
    family: Some(&STUMP_DEED_CLASS),
    label_number: crate::class_props::label_number, // a deed for a tree stump decoration
    on_double_click: |w, id, from| {
        if reward_blocked(w, from, id) {
            return;
        }
        if in_pack(w, id, from) {
            option_gump(w, from, id);
        } else {
            msg(w, from, 1062334); // This item must be in your backpack to be used.
        }
    },
    get_properties: |w, id, list| {
        (BASE_ADDON_DEED.get_properties)(w, id, list);
        year(w, id, list, 1076223); // 7th Year Veteran Reward
    },
    serialize: |w, id, out| {
        (BASE_ADDON_DEED.serialize)(w, id, out);
        out.write_encoded_int(0);
        out.write_bool(is_reward(w, id));
        out.write_i32(deed_state(w, id).logs);
    },
    deserialize: |w, id, r| {
        (BASE_ADDON_DEED.deserialize)(w, id, r)?;
        let _version = r.read_encoded_int()?;
        let reward = r.read_bool()?;
        let logs = r.read_i32()?;
        set_reward_raw(w, id, reward);
        let mut s = deed_state(w, id);
        s.logs = logs;
        if let Some(i) = w.item_mut(id) {
            i.ext.insert(s);
        }
        Ok(())
    },
    ..BASE_ADDON_DEED
};

// ---- RewardBrazier ----

/// `RewardBrazier._fire`.
#[derive(Clone, Copy, Debug, Default)]
struct Fire(Option<ItemId>);

pub(super) fn fire_of(w: &World, id: ItemId) -> Option<ItemId> {
    w.item(id)
        .and_then(|i| i.ext.get::<Fire>())
        .and_then(|f| f.0)
}

pub(super) fn set_fire(w: &mut World, id: ItemId, fire: Option<ItemId>) {
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(Fire(fire));
    }
}

/// `RewardBrazier.TurnOff()`.
pub fn brazier_turn_off(w: &mut World, id: ItemId) {
    if let Some(f) = fire_of(w, id) {
        super::util::delete(w, f);
        set_fire(w, id, None);
    }
}

pub(super) fn brazier_turn_on(w: &mut World, id: ItemId) {
    let fire = match fire_of(w, id) {
        Some(f) => f,
        None => {
            let f = ops::new_item_with_id(w, &ITEM_BASE, 0);
            set_fire(w, id, Some(f));
            f
        }
    };
    super::item_id_of(w, fire, 0x19AB);
    ops::set_movable(w, fire, false);
    let (loc, map, item_id) = w.item(id).map_or((Point3D::default(), None, 0), |i| {
        (i.location, i.map, i.item_id)
    });
    let h = i32::from(w.maps.tile_data.item((item_id & 0x3FFF) as usize).height);
    ops::move_to_world_map(w, fire, Point3D::new(loc.x, loc.y, loc.z + h + 2), map);
}

/// `RewardBrazier(itemID)`.
pub fn reward_brazier_new_with(w: &mut World, item_id: i32) -> ItemId {
    let id = ops::new_item_with_id(w, &REWARD_BRAZIER, item_id);
    blessed(w, id)
}

/// `Server.Items.RewardBrazier`: `_fire` is `[SerializableField(1)]` after `IsRewardItem`.
pub static REWARD_BRAZIER: ItemType = ItemType {
    full_name: "Server.Items.RewardBrazier",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let g = if utility::random(&mut w.rng, 2) == 0 {
            0x19AA
        } else {
            0x19BB
        };
        reward_brazier_new_with(w, g)
    }),
    default_weight: crate::class_props::default_weight,
    force_show_properties: |w, _| w.core.opl_enabled,
    on_delete: |w, id| {
        brazier_turn_off(w, id);
        (ITEM_BASE.on_delete)(w, id);
    },
    on_double_click: |w, id, from| {
        if !muo_content::mx::in_range_point(w, from, ops::get_world_location(w, id), 2) {
            overhead(w, from, 0x3B2, 1019045); // I can't reach that.
            return;
        }
        if !ops::is_locked_down(w, id) {
            msg(w, from, 502692); // This must be in a house and be locked down to work.
            return;
        }
        let house = muo_multis::house::access::find_house_at_mobile(w, from);
        if !house.is_some_and(|h| muo_multis::house::access::is_co_owner(w, h, from)) {
            msg(w, from, 502436); // That is not accessible.
            return;
        }
        if fire_of(w, id).is_some() {
            brazier_turn_off(w, id);
        } else {
            brazier_turn_on(w, id);
        }
    },
    on_location_change: |w, id, _old| {
        if let Some(f) = fire_of(w, id) {
            let (loc, map, item_id) = w.item(id).map_or((Point3D::default(), None, 0), |i| {
                (i.location, i.map, i.item_id)
            });
            let h = i32::from(w.maps.tile_data.item((item_id & 0x3FFF) as usize).height);
            ops::move_to_world_map(w, f, Point3D::new(loc.x, loc.y, loc.z + h), map);
        }
    },
    get_properties: |w, id, list| {
        (ITEM_BASE.get_properties)(w, id, list);
        year(w, id, list, 1076222); // 6th Year Veteran Reward
    },
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        out.write_encoded_int(0);
        out.write_bool(is_reward(w, id));
        out.write_item(w, fire_of(w, id));
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let _version = r.read_encoded_int()?;
        let flag = r.read_bool()?;
        let fire = r.read_item(w)?;
        set_reward_raw(w, id, flag);
        set_fire(w, id, fire);
        Ok(())
    },
    ..ITEM_BASE
};

// ---- "choose one" deeds: RewardBrazierDeed, PottedCactusDeed ----

/// A `StaticGump<T>` of a deed whose buttons are the item ids to choose from.
pub(super) trait Pick: 'static {
    const NAME: &'static str;
    const POS: (i32, i32);
    fn layout(b: &mut StaticGumpBuilder);
    fn valid(button: i32) -> bool;
    /// Whether the response needs the deed in the pack (1042038 otherwise).
    const CHECK_PACK: bool = false;
    /// The item the choice gives (its `IsRewardItem` copied from the deed).
    fn make(w: &mut World, button: i32, deed: ItemId) -> ItemId;
}

struct PickGump<T: Pick> {
    deed: ItemId,
    _t: std::marker::PhantomData<T>,
}

impl<T: Pick> StaticGumpLogic for PickGump<T> {
    const TYPE_NAME: &'static str = T::NAME;

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut StaticGumpBuilder) {
        T::layout(b);
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        if ops::is_deleted(w, self.deed) {
            return;
        }
        let Some(m) = mobile_of(w, sender) else {
            return;
        };
        if T::CHECK_PACK && !in_pack(w, self.deed, m) {
            msg(w, m, 1042038); // You must have the object in your backpack to use it.
            return;
        }
        if !T::valid(info.button_id) {
            return;
        }
        let item = T::make(w, info.button_id, self.deed);
        if place_or_delete(w, m, item) {
            super::util::delete(w, self.deed);
        }
    }
}

pub(super) fn show_pick<T: Pick>(w: &mut World, from: MobileId, deed: ItemId) {
    if super::util::net_state(w, from).is_none() || ops::is_deleted(w, deed) {
        return;
    }
    let g = PickGump::<T> {
        deed,
        _t: std::marker::PhantomData,
    };
    send_gump_to_mobile(
        w,
        from,
        Box::new(StaticGumpObj::new(T::POS.0, T::POS.1, g)),
        false,
    );
}

struct BrazierPick;

impl Pick for BrazierPick {
    const NAME: &'static str = "Server.Items.RewardBrazierDeed+BrazierGump";
    const POS: (i32, i32) = (100, 200);

    fn layout(b: &mut StaticGumpBuilder) {
        b.add_page(0);
        b.add_background(0, 0, 200, 200, 2600);
        b.add_page(1);
        b.add_label(45, 15, 0, "Choose a Brazier:");
        b.add_item(40, 75, 0x19AA, 0);
        b.add_button(55, 50, 0x845, 0x846, 0x19AA);
        b.add_item(100, 75, 0x19BB, 0);
        b.add_button(115, 50, 0x845, 0x846, 0x19BB);
    }

    fn valid(button: i32) -> bool {
        button == 0x19AA || button == 0x19BB
    }

    fn make(w: &mut World, button: i32, deed: ItemId) -> ItemId {
        let b = reward_brazier_new_with(w, button);
        set_reward(w, b, is_reward(w, deed));
        b
    }
}

/// `RewardBrazierDeed()`.
pub fn reward_brazier_deed_new(w: &mut World) -> ItemId {
    let id = crate::simple::from_template(w, &REWARD_BRAZIER_DEED, "Server.Items.RewardBrazierDeed");
    blessed(w, id)
}

rtype! {
    /// `Server.Items.RewardBrazierDeed`.
    REWARD_BRAZIER_DEED = "Server.Items.RewardBrazierDeed" : ITEM_BASE;
    construct: Some(reward_brazier_deed_new),
    default_weight: crate::class_props::default_weight,
    label_number: crate::class_props::label_number, // Brazier Deed
    on_double_click: |w, id, from| {
        if reward_blocked(w, from, id) {
            return;
        }
        if !in_pack(w, id, from) {
            msg(w, from, 1042038); // You must have the object in your backpack to use it.
            return;
        }
        show_pick::<BrazierPick>(w, from, id);
    },
    get_properties: |w, id, list| {
        (ITEM_BASE.get_properties)(w, id, list);
        year(w, id, list, 1076222); // 6th Year Veteran Reward
    },
}

/// `RewardPottedCactus(itemID)`.
pub fn potted_cactus_new_with(w: &mut World, item_id: i32) -> ItemId {
    ops::new_item_with_id(w, &REWARD_POTTED_CACTUS, item_id)
}

rtype! {
    /// `Server.Items.RewardPottedCactus`.
    REWARD_POTTED_CACTUS = "Server.Items.RewardPottedCactus" : ITEM_BASE;
    construct: Some(|w| {
        let g = utility::random_min_max(&mut w.rng, 0x1E0F, 0x1E14);
        potted_cactus_new_with(w, g)
    }),
    default_weight: crate::class_props::default_weight,
    force_show_properties: |w, _| w.core.opl_enabled,
}

struct CactusPick;

impl Pick for CactusPick {
    const NAME: &'static str = "Server.Items.PottedCactusDeed+PottedCactusGump";
    const POS: (i32, i32) = (100, 200);

    fn layout(b: &mut StaticGumpBuilder) {
        b.add_page(0);
        b.add_background(0, 0, 425, 250, 0xA28);
        b.add_page(1);
        b.add_label(45, 15, 0, "Choose a Potted Cactus:");
        for (x, bx, g) in [
            (45, 55, 0x1E0F),
            (105, 115, 0x1E10),
            (160, 175, 0x1E14),
            (220, 235, 0x1E11),
            (280, 295, 0x1E12),
            (340, 355, 0x1E13),
        ] {
            b.add_item(x, 75, g, 0);
            b.add_button(bx, 50, 0x845, 0x846, g);
        }
    }

    fn valid(button: i32) -> bool {
        (0x1E0F..=0x1E14).contains(&button)
    }

    fn make(w: &mut World, button: i32, deed: ItemId) -> ItemId {
        let c = potted_cactus_new_with(w, button);
        set_reward(w, c, is_reward(w, deed));
        c
    }
}

/// `PottedCactusDeed()`.
pub fn potted_cactus_deed_new(w: &mut World) -> ItemId {
    let id = crate::simple::from_template(w, &POTTED_CACTUS_DEED, "Server.Items.PottedCactusDeed");
    blessed(w, id)
}

rtype! {
    /// `Server.Items.PottedCactusDeed`.
    POTTED_CACTUS_DEED = "Server.Items.PottedCactusDeed" : ITEM_BASE;
    construct: Some(potted_cactus_deed_new),
    default_weight: crate::class_props::default_weight,
    label_number: crate::class_props::label_number, // Potted Cactus Deed
    on_double_click: |w, id, from| {
        if reward_blocked(w, from, id) {
            return;
        }
        if in_pack(w, id, from) {
            show_pick::<CactusPick>(w, from, id);
        } else {
            msg(w, from, 1042038); // You must have the object in your backpack to use it.
        }
    },
    get_properties: |w, id, list| {
        (ITEM_BASE.get_properties)(w, id, list);
        year(w, id, list, 1076219); // 3rd Year Veteran Reward
    },
}

pub fn register(types: &mut TypeRegistry) {
    for t in [
        &BLOODY_PENTAGRAM_COMPONENT,
        &BLOODY_PENTAGRAM_ADDON,
        &BLOODY_PENTAGRAM_DEED,
        &STONE_ANKH_COMPONENT,
        &STONE_ANKH,
        &STONE_ANKH_DEED,
        &MINOTAUR_STATUE,
        &MINOTAUR_STATUE_DEED,
        &TREE_STUMP,
        &TREE_STUMP_DEED,
        &REWARD_BRAZIER,
        &REWARD_BRAZIER_DEED,
        &REWARD_POTTED_CACTUS,
        &POTTED_CACTUS_DEED,
    ] {
        types.register_item(t, &[]);
    }
}

/// Configure phase: `RewardBrazier.TurnOff`.
pub(super) fn configure(w: &mut World) {
    house_deps::deps_mut(w).brazier_turn_off = Some(brazier_turn_off);
}
