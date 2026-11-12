//! The 2004 mistletoe (the addon and its deed) and the 2010 fireflies (the addon and their deed).

use muo_content::gumps::base::{RelayInfo, StaticGumpLogic, StaticGumpObj};
use muo_content::gumps::builder::StaticGumpBuilder;
use muo_content::gumps::send_gump_to_mobile;
use muo_content::items::addon::{house_addons_add, is_wall};
use muo_content::items::house_deps::{self, SimpleAddon};
use muo_content::items::interfaces::register_dyable;
use muo_primitives::{Point3D, TimeSpan, utility};
use muo_server::World;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType, LootType, deps, ops};
use muo_server::map::MapId;
use muo_server::net_state::NetStateId;
use muo_server::property_list::PropertyList;
use muo_server::type_registry::TypeRegistry;

use super::util::{deleted, in_pack, msg, overhead};
use super::vet::{begin_target, demolition_gump, target_point};

const WINTER_2004: i32 = 1070880;

fn house_at_item(w: &World, id: ItemId) -> Option<ItemId> {
    muo_multis::house::access::find_house_at_item(w, id)
}

fn house_at_mobile(w: &World, m: MobileId) -> Option<ItemId> {
    muo_multis::house::access::find_house_at_mobile(w, m)
}

fn house_at_point(w: &World, p: Point3D, map: Option<MapId>) -> Option<ItemId> {
    muo_multis::house::access::find_house_at_point(w, p, map, 16)
}

fn co_owner(w: &World, house: Option<ItemId>, m: MobileId) -> bool {
    house.is_some_and(|h| muo_multis::house::access::is_co_owner(w, h, m))
}

fn owner(w: &World, house: Option<ItemId>, m: MobileId) -> bool {
    house.is_some_and(|h| muo_multis::house::access::is_owner(w, h, m))
}

fn mobile_map(w: &World, m: MobileId) -> Option<MapId> {
    w.mobile(m).and_then(|x| x.map)
}

fn winter_single_click(w: &mut World, id: ItemId, from: MobileId) {
    (ITEM_BASE.on_single_click)(w, id, from);
    ops::label_to_number(w, id, from, WINTER_2004, "");
}

fn winter_props(w: &World, id: ItemId, list: &mut PropertyList) {
    (ITEM_BASE.get_properties)(w, id, list);
    list.add(WINTER_2004);
}

// ---- MistletoeAddon ----

/// `MistletoeAddon(hue)`.
pub fn mistletoe_addon_new_with(w: &mut World, hue: i32) -> ItemId {
    let id = ops::new_item_with_id(w, &MISTLETOE_ADDON, 0x2375);
    crate::simple::hue(w, id, hue);
    ops::set_movable(w, id, false);
    id
}

/// `MistletoeAddon()`: `this(Utility.RandomDyedHue())`.
pub fn mistletoe_addon_new(w: &mut World) -> ItemId {
    let hue = utility::random_dyed_hue(&mut w.rng);
    mistletoe_addon_new_with(w, hue)
}

fn addon_hue(w: &World, id: ItemId) -> i32 {
    w.item(id).map_or(0, |i| i.hue)
}

fn mistletoe_deed_of(w: &mut World, id: ItemId) -> ItemId {
    let hue = addon_hue(w, id);
    mistletoe_deed_new(w, hue)
}

/// `MistletoeAddon.CouldFit(p, map)`.
fn mistletoe_could_fit(w: &mut World, id: ItemId, p: Point3D, map: MapId) -> bool {
    let height = i32::from(ops::item_data(w, id).height);
    if !muo_server::map::fit::can_fit_point(w, map, p, height) {
        return false;
    }
    if w.item(id).is_some_and(|i| i.item_id == 0x2375) {
        return is_wall(w, p.x, p.y - 1, p.z, Some(map)); // North wall
    }
    is_wall(w, p.x - 1, p.y, p.z, Some(map)) // West wall
}

/// `[AfterDeserialization(false)] FixMovingCrate()`: a movable or locked down addon is turned back into its deed.
fn fix_moving_crate(w: &mut World, id: ItemId) {
    if deleted(w, id) {
        return;
    }
    if w.item(id).is_some_and(|i| i.movable()) || ops::is_locked_down(w, id) {
        let deed = mistletoe_deed_of(w, id);
        let (loc, map, parent) = w.item(id).map_or((Point3D::default(), None, None), |i| {
            (i.location, i.map, i.parent)
        });
        if let Some(EntityId::Item(p)) = parent {
            muo_server::item::base::add_item(w, p, deed);
            muo_server::item::base::set_location(w, deed, loc);
        } else {
            ops::move_to_world_map(w, deed, loc, map);
        }
        super::util::delete(w, id);
    }
}

struct MistletoeAddonGump {
    addon: ItemId,
}

impl StaticGumpLogic for MistletoeAddonGump {
    const TYPE_NAME: &'static str = "Server.Items.MistletoeAddon+MistletoeAddonGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut StaticGumpBuilder) {
        b.add_page(0);
        b.add_background(0, 0, 220, 170, 0x13BE);
        b.add_background(10, 10, 200, 150, 0xBB8);
        b.add_html_localized(20, 30, 180, 60, 1062839, false, false); // Do you wish to re-deed this decoration?
        b.add_html_localized(55, 100, 160, 25, 1011011, false, false); // CONTINUE
        b.add_button(20, 100, 0xFA5, 0xFA7, 1);
        b.add_html_localized(55, 125, 160, 25, 1011012, false, false); // CANCEL
        b.add_button(20, 125, 0xFA5, 0xFA7, 0);
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        if deleted(w, self.addon) || info.button_id != 1 {
            return;
        }
        let Some(from) = w.net_states.get(&sender).and_then(|n| n.mobile) else {
            return;
        };
        if deps::mobile_in_range(w, from, ops::get_world_location(w, self.addon), 3) {
            let deed = mistletoe_deed_of(w, self.addon);
            muo_server::mobile::part2::items::add_to_backpack(w, from, deed);
            super::util::delete(w, self.addon);
        } else {
            msg(w, from, 500295); // You are too far away to do that.
        }
    }
}

/// `Server.Items.MistletoeAddon` (`IDyable`, `IAddon`).
pub static MISTLETOE_ADDON: ItemType = ItemType {
    full_name: "Server.Items.MistletoeAddon",
    parent: Some(&ITEM_BASE),
    construct: Some(mistletoe_addon_new),
    on_double_click: |w, id, from| {
        let house = house_at_item(w, id);
        if !co_owner(w, house, from) {
            return;
        }
        if deps::mobile_in_range(w, from, ops::get_world_location(w, id), 3) {
            if super::util::net_state(w, from).is_none() || deleted(w, id) {
                return;
            }
            send_gump_to_mobile(
                w,
                from,
                Box::new(StaticGumpObj::new(
                    150,
                    50,
                    MistletoeAddonGump { addon: id },
                )),
                false,
            );
        } else {
            overhead(w, from, 0x3B2, 1019045); // I can't reach that.
        }
    },
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        out.write_i32(0);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        muo_server::timer::start(w, TimeSpan::ZERO, TimeSpan::ZERO, 1, move |w| {
            fix_moving_crate(w, id)
        });
        Ok(())
    },
    ..ITEM_BASE
};

// ---- MistletoeDeed ----

/// `MistletoeDeed(hue)`.
pub fn mistletoe_deed_new(w: &mut World, hue: i32) -> ItemId {
    let id = ops::new_item_with_id(w, &MISTLETOE_DEED, 0x14F0);
    crate::simple::hue(w, id, hue);
    ops::set_loot_type(w, id, LootType::Blessed);
    id
}

/// `MistletoeDeed.PlaceAddon(from, loc, northWall, westWall)`.
fn place_addon(
    w: &mut World,
    deed: ItemId,
    from: MobileId,
    loc: Point3D,
    north_wall: bool,
    west_wall: bool,
) {
    if deleted(w, deed) {
        return;
    }
    let map = mobile_map(w, from);
    let house = house_at_point(w, loc, map);
    if !co_owner(w, house, from) {
        msg(w, from, 1042036); // That location is not in your house.
        return;
    }
    let item_id = if north_wall {
        0x2374
    } else if west_wall {
        0x2375
    } else {
        msg(w, from, 1070883); // The mistletoe must be placed next to a wall.
        0
    };
    if item_id > 0 {
        let hue = addon_hue(w, deed);
        let addon = mistletoe_addon_new_with(w, hue);
        crate::simple::item_id(w, addon, item_id);
        ops::move_to_world_map(w, addon, loc, map);
        if let Some(h) = house {
            house_addons_add(w, h, addon);
        }
        super::util::delete(w, deed);
    }
}

/// `MistletoeDeed.Placement_OnTarget`.
fn placement_on_target(
    w: &mut World,
    deed: ItemId,
    from: MobileId,
    t: muo_server::target::Targeted,
) {
    let Some(loc) = target_point(w, t) else {
        return;
    };
    let map = mobile_map(w, from);
    let house = house_at_point(w, loc, map);
    if !co_owner(w, house, from) {
        msg(w, from, 1042036); // That location is not in your house.
        return;
    }
    let north_wall = is_wall(w, loc.x, loc.y - 1, loc.z, map);
    let west_wall = is_wall(w, loc.x - 1, loc.y, loc.z, map);
    if north_wall && west_wall {
        if super::util::net_state(w, from).is_some() {
            send_gump_to_mobile(
                w,
                from,
                Box::new(StaticGumpObj::new(150, 50, MistletoeDeedGump { deed, loc })),
                false,
            );
        }
    } else {
        place_addon(w, deed, from, loc, north_wall, west_wall);
    }
}

struct MistletoeDeedGump {
    deed: ItemId,
    loc: Point3D,
}

impl StaticGumpLogic for MistletoeDeedGump {
    const TYPE_NAME: &'static str = "Server.Items.MistletoeDeed+MistletoeDeedGump";

    fn build_layout(&self, b: &mut StaticGumpBuilder) {
        b.add_background(0, 0, 300, 150, 0xA28);
        b.add_page(0);
        b.add_item(90, 30, 0x2375, 0);
        b.add_item(180, 30, 0x2374, 0);
        b.add_button(50, 35, 0x868, 0x869, 1);
        b.add_button(145, 35, 0x868, 0x869, 2);
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        if deleted(w, self.deed) || info.button_id == 0 {
            return;
        }
        let Some(from) = w.net_states.get(&sender).and_then(|n| n.mobile) else {
            return;
        };
        if info.button_id == 1 {
            place_addon(w, self.deed, from, self.loc, false, true);
        } else {
            place_addon(w, self.deed, from, self.loc, true, false);
        }
    }
}

/// `Server.Items.MistletoeDeed` (`[Flippable(0x14F0, 0x14EF)]`).
pub static MISTLETOE_DEED: ItemType = ItemType {
    full_name: "Server.Items.MistletoeDeed",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| mistletoe_deed_new(w, 0)),
    default_weight: |_, _| 1.0,
    label_number: |_, _| 1070882, // Mistletoe Deed
    on_single_click: winter_single_click,
    get_properties: winter_props,
    on_double_click: |w, id, from| {
        if !in_pack(w, id, from) {
            msg(w, from, 1042001); // That must be in your pack for you to use it.
            return;
        }
        if co_owner(w, house_at_mobile(w, from), from) {
            msg(w, from, 1062838); // Where would you like to place this decoration?
            begin_target(w, from, move |w, from, t| {
                placement_on_target(w, id, from, t)
            });
        } else {
            msg(w, from, 502092); // You must be in your house to do this.
        }
    },
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        out.write_i32(0);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        Ok(())
    },
    ..ITEM_BASE
};

// ---- Fireflies ----

/// `Fireflies(itemID)`.
pub fn fireflies_new(w: &mut World, item_id: i32) -> ItemId {
    let id = ops::new_item_with_id(w, &FIREFLIES, item_id);
    ops::set_loot_type(w, id, LootType::Blessed);
    ops::set_movable(w, id, false);
    id
}

fn facing_south(w: &World, id: ItemId) -> bool {
    w.item(id).is_some_and(|i| i.item_id == 0x2336)
}

fn fireflies_could_fit(w: &mut World, id: ItemId, p: Point3D, map: MapId) -> bool {
    let height = i32::from(ops::item_data(w, id).height);
    if !muo_server::map::fit::can_fit_point(w, map, p, height) {
        return false;
    }
    if facing_south(w, id) {
        is_wall(w, p.x, p.y - 1, p.z, Some(map))
    } else {
        is_wall(w, p.x - 1, p.y, p.z, Some(map))
    }
}

/// `Server.Items.Fireflies` (`[SerializationGenerator(0)]`, `IAddon`).
pub static FIREFLIES: ItemType = ItemType {
    full_name: "Server.Items.Fireflies",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| fireflies_new(w, 0x1596)),
    label_number: |_, _| 1150061,
    on_double_click: |w, id, from| {
        let loc = w.item(id).map_or(Point3D::default(), |i| i.location);
        if !deps::mobile_in_range(w, from, loc, 3) {
            overhead(w, from, 0x3B2, 1019045); // I can't reach that.
            return;
        }
        if owner(w, house_at_item(w, id), from) {
            demolition_gump(w, from, id, 1049783); // Do you wish to re-deed this decoration?
        } else {
            // You can only re-deed this decoration if you are the house owner or originally placed the decoration.
            msg(w, from, 1049784);
        }
    },
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        out.write_encoded_int(0);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let _version = r.read_encoded_int()?;
        Ok(())
    },
    ..ITEM_BASE
};

/// `FirefliesDeed()`.
pub fn fireflies_deed_new(w: &mut World) -> ItemId {
    let id = ops::new_item_with_id(w, &FIREFLIES_DEED, 0x14F0);
    ops::set_loot_type(w, id, LootType::Blessed);
    id
}

struct FacingGump {
    deed: ItemId,
}

impl StaticGumpLogic for FacingGump {
    const TYPE_NAME: &'static str = "Server.Items.FirefliesDeed+FacingGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut StaticGumpBuilder) {
        b.add_background(0, 0, 300, 150, 0xA28);
        b.add_page(0);
        b.add_item(90, 30, 0x2332, 0);
        b.add_item(180, 30, 0x2336, 0);
        b.add_button(50, 35, 0x868, 0x869, 2); // East
        b.add_button(145, 35, 0x868, 0x869, 1); // South
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        if info.button_id == 0 {
            return;
        }
        let item_id = match info.button_id {
            2 => 0x2332,
            1 => 0x2336,
            _ => 0,
        };
        if item_id == 0 {
            return;
        }
        let Some(from) = w.net_states.get(&sender).and_then(|n| n.mobile) else {
            return;
        };
        let deed = self.deed;
        begin_target(w, from, move |w, from, t| {
            fireflies_target(w, deed, item_id, from, t)
        });
    }
}

/// `FirefliesDeed.InternalTarget.OnTarget`.
fn fireflies_target(
    w: &mut World,
    deed: ItemId,
    item_id: i32,
    from: MobileId,
    t: muo_server::target::Targeted,
) {
    if deleted(w, deed) {
        return;
    }
    if !in_pack(w, deed, from) {
        msg(w, from, 1042038); // You must have the object in your backpack to use it.
        return;
    }
    if !owner(w, house_at_mobile(w, from), from) {
        msg(w, from, 502092); // You must be in your house to do this.
        return;
    }
    let Some(p) = target_point(w, t) else { return };
    let Some(map) = mobile_map(w, from) else {
        return;
    };
    if map == MapId::INTERNAL {
        return;
    }
    let height = i32::from(
        w.maps
            .tile_data
            .item((item_id as usize) & muo_data::TileData::MAX_ITEM_VALUE)
            .height,
    );
    if !muo_server::map::fit::can_fit_point(w, map, p, height) {
        msg(w, from, 500269); // You cannot build that there.
        return;
    }
    let house = muo_multis::house::access::find_house_at_point(w, p, Some(map), height);
    if !owner(w, house, from) {
        msg(w, from, 1042036); // That location is not in your house.
        return;
    }
    let north = is_wall(w, p.x, p.y - 1, p.z, Some(map));
    let west = is_wall(w, p.x - 1, p.y, p.z, Some(map));
    if (item_id != 0x2336 || !north) && (item_id != 0x2332 || !west) {
        msg(w, from, 1150065); // Holiday fireflies must be placed next to a wall.
        return;
    }
    for i in deps::items_at(w, MapId::MALAS, p) {
        if w.item(i)
            .is_some_and(|x| x.ty.is(&FIREFLIES) && x.location.z == p.z)
        {
            msg(w, from, 1150065);
            return;
        }
    }
    let flies = fireflies_new(w, item_id);
    if let Some(h) = house {
        house_addons_add(w, h, flies);
    }
    ops::move_to_world_map(w, flies, p, Some(map));
    super::util::delete(w, deed);
}

/// `Server.Items.FirefliesDeed`.
pub static FIREFLIES_DEED: ItemType = ItemType {
    full_name: "Server.Items.FirefliesDeed",
    parent: Some(&ITEM_BASE),
    construct: Some(fireflies_deed_new),
    default_weight: |_, _| 1.0,
    label_number: |_, _| 1150061,
    on_double_click: |w, id, from| {
        if !in_pack(w, id, from) {
            msg(w, from, 1042038); // You must have the object in your backpack to use it.
            return;
        }
        if !owner(w, house_at_mobile(w, from), from) {
            msg(w, from, 502092); // You must be in your house to do this.
            return;
        }
        if super::util::net_state(w, from).is_some() {
            send_gump_to_mobile(
                w,
                from,
                Box::new(StaticGumpObj::new(150, 50, FacingGump { deed: id })),
                false,
            );
        }
    },
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        out.write_encoded_int(0);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let _version = r.read_encoded_int()?;
        Ok(())
    },
    ..ITEM_BASE
};

// ---- registration ----

pub fn register(types: &mut TypeRegistry) {
    for t in [
        &MISTLETOE_ADDON,
        &MISTLETOE_DEED,
        &FIREFLIES,
        &FIREFLIES_DEED,
    ] {
        types.register_item(t, &[]);
    }
}

pub(super) fn configure(w: &mut World) {
    // `MistletoeAddon.Dye`: a co-owner within 1 tile re-dyes it.
    register_dyable(w, &MISTLETOE_ADDON, |w, id, from, tub| {
        if deleted(w, id) {
            return false;
        }
        if co_owner(w, house_at_item(w, id), from) {
            if deps::mobile_in_range(w, from, ops::get_world_location(w, id), 1) {
                let hue = muo_content::items::clothing::tub_dyed_hue(w, tub);
                crate::simple::hue(w, id, hue);
                return true;
            }
            msg(w, from, 500295); // You are too far away to do that.
        }
        false
    });
    let simple = |is: fn(&World, ItemId) -> bool,
                  deed: fn(&mut World, ItemId) -> ItemId,
                  could_fit| SimpleAddon {
        is,
        deed,
        could_fit,
    };
    let deps_ = house_deps::deps_mut(w);
    deps_.simple_addons.push(simple(
        |w, id| w.item(id).is_some_and(|i| i.ty.is(&MISTLETOE_ADDON)),
        mistletoe_deed_of,
        mistletoe_could_fit,
    ));
    deps_.simple_addons.push(simple(
        |w, id| w.item(id).is_some_and(|i| i.ty.is(&FIREFLIES)),
        |w, _| fireflies_deed_new(w),
        fireflies_could_fit,
    ));
}
