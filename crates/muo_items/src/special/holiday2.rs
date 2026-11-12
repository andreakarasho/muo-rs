//! `WreathAddon` and `WreathDeed`.
//!

use muo_content::gumps::base::{RelayInfo, StaticGumpLogic, StaticGumpObj};
use muo_content::gumps::builder::StaticGumpBuilder;
use muo_content::gumps::send_gump_to_mobile;
use muo_content::items::addon;
use muo_content::items::house_deps::{self, SimpleAddon};
use muo_content::items::interfaces::register_dyable;
use muo_primitives::{Point3D, utility};
use muo_server::World;
use muo_server::ids::{ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType, LootType, deps, ops};
use muo_server::net_state::NetStateId;
use muo_server::target::Targeted;
use muo_server::type_registry::TypeRegistry;

use super::util::{delete, msg, overhead};
use super::vet::{begin_target, target_point};
use crate::simple::leaf;

fn co_owner_at_item(w: &World, id: ItemId, m: MobileId) -> bool {
    muo_multis::house::access::find_house_at_item(w, id)
        .is_some_and(|h| muo_multis::house::access::is_co_owner(w, h, m))
}

/// `WreathAddon(hue)`.
pub fn wreath_addon_new_with(w: &mut World, hue: i32) -> ItemId {
    let id = ops::new_item_with_id(w, &WREATH_ADDON, 0x232C);
    super::hue(w, id, hue);
    ops::set_movable(w, id, false);
    id
}

fn wreath_addon_deed(w: &mut World, id: ItemId) -> ItemId {
    let hue = w.item(id).map_or(0, |i| (i.ty.hue)(w, id));
    wreath_deed_new_with(w, hue)
}

fn wreath_could_fit(w: &mut World, id: ItemId, p: Point3D, map: muo_server::map::MapId) -> bool {
    let height = i32::from(ops::item_data(w, id).height);
    if !muo_server::map::fit::can_fit_item(w, map, p.x, p.y, p.z, height) {
        return false;
    }
    if w.item(id).map_or(0, |i| i.item_id) == 0x232C {
        addon::is_wall(w, p.x, p.y - 1, p.z, Some(map)) // North wall
    } else {
        addon::is_wall(w, p.x - 1, p.y, p.z, Some(map)) // West wall
    }
}

/// `WreathAddon.WreathAddonGump`.
struct WreathAddonGump {
    addon: ItemId,
}

impl StaticGumpLogic for WreathAddonGump {
    const TYPE_NAME: &'static str = "Server.Items.WreathAddon+WreathAddonGump";

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
        if ops::is_deleted(w, self.addon) || info.button_id != 1 {
            return;
        }
        let Some(from) = w.net_states.get(&sender).and_then(|n| n.mobile) else {
            return;
        };
        if muo_content::mx::in_range_point(w, from, ops::get_world_location(w, self.addon), 3) {
            let deed = wreath_addon_deed(w, self.addon);
            deps::add_to_backpack(w, from, deed);
            delete(w, self.addon);
        } else {
            msg(w, from, 500295); // You are too far away to do that.
        }
    }
}

/// `Server.Items.WreathAddon` (`IDyable`, `IAddon`).
pub static WREATH_ADDON: ItemType = ItemType {
    full_name: "Server.Items.WreathAddon",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let hue = utility::random_dyed_hue(&mut w.rng);
        wreath_addon_new_with(w, hue)
    }),
    on_double_click: |w, id, from| {
        if !co_owner_at_item(w, id, from) {
            return;
        }
        if muo_content::mx::in_range_point(w, from, ops::get_world_location(w, id), 3) {
            if super::util::net_state(w, from).is_some() && !ops::is_deleted(w, id) {
                send_gump_to_mobile(
                    w,
                    from,
                    Box::new(StaticGumpObj::new(150, 50, WreathAddonGump { addon: id })),
                    false,
                );
            }
        } else {
            overhead(w, from, 0x3B2, 1019045); // I can't reach that.
        }
    },
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        out.write_encoded_int(0);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let _version = r.read_encoded_int()?;
        // [AfterDeserialization(false)] FixMovingCrate
        muo_server::timer::start(
            w,
            muo_primitives::TimeSpan::ZERO,
            muo_primitives::TimeSpan::ZERO,
            1,
            move |w| {
                fix_moving_crate(w, id);
            },
        );
        Ok(())
    },
    ..ITEM_BASE
};

/// `WreathAddon.FixMovingCrate()`: a movable or locked-down wreath becomes a deed again.
fn fix_moving_crate(w: &mut World, id: ItemId) {
    if ops::is_deleted(w, id)
        || !(w.item(id).is_some_and(|i| i.movable()) || ops::is_locked_down(w, id))
    {
        return;
    }
    let deed = wreath_addon_deed(w, id);
    let (loc, map, parent) = w.item(id).map_or((Point3D::default(), None, None), |i| {
        (i.location, i.map, i.parent)
    });
    if let Some(muo_server::ids::EntityId::Item(p)) = parent {
        muo_server::item::base::add_item(w, p, deed);
        ops::set_location(w, deed, loc);
    } else {
        ops::move_to_world_map(w, deed, loc, map);
    }
    delete(w, id);
}

/// `WreathDeed(hue)`.
pub fn wreath_deed_new_with(w: &mut World, hue: i32) -> ItemId {
    let id = ops::new_item_with_id(w, &WREATH_DEED, 0x14F0);
    super::hue(w, id, hue);
    ops::set_loot_type(w, id, LootType::Blessed);
    id
}

/// `WreathDeed.PlaceAddon(from, loc, northWall, westWall)`.
fn wreath_place(
    w: &mut World,
    deed: ItemId,
    from: MobileId,
    loc: Point3D,
    north: bool,
    west: bool,
) {
    if ops::is_deleted(w, deed) {
        return;
    }
    let map = w.mobile(from).and_then(|m| m.map);
    let house = muo_multis::house::access::find_house_at_point(w, loc, map, 16)
        .filter(|&h| muo_multis::house::access::is_co_owner(w, h, from));
    let Some(house) = house else {
        msg(w, from, 1042036); // That location is not in your house.
        return;
    };
    let mut item_id = 0;
    if north {
        item_id = 0x232C;
    } else if west {
        item_id = 0x232D;
    } else {
        msg(w, from, 1062840); // The decoration must be placed next to a wall.
    }
    if item_id > 0 {
        let hue = w.item(deed).map_or(0, |i| (i.ty.hue)(w, deed));
        let a = wreath_addon_new_with(w, hue);
        super::item_id_of(w, a, item_id);
        ops::move_to_world_map(w, a, loc, map);
        addon::house_addons_add(w, house, a);
        delete(w, deed);
    }
}

/// `WreathDeed.WreathDeedGump`.
struct WreathDeedGump {
    deed: ItemId,
    loc: Point3D,
}

impl StaticGumpLogic for WreathDeedGump {
    const TYPE_NAME: &'static str = "Server.Items.WreathDeed+WreathDeedGump";

    fn build_layout(&self, b: &mut StaticGumpBuilder) {
        b.add_background(0, 0, 300, 150, 0xA28);
        b.add_page(0);
        b.add_item(90, 30, 0x232D, 0);
        b.add_item(180, 30, 0x232C, 0);
        b.add_button(50, 35, 0x868, 0x869, 1);
        b.add_button(145, 35, 0x868, 0x869, 2);
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        if ops::is_deleted(w, self.deed) || info.button_id == 0 {
            return;
        }
        let Some(from) = w.net_states.get(&sender).and_then(|n| n.mobile) else {
            return;
        };
        if info.button_id == 1 {
            wreath_place(w, self.deed, from, self.loc, false, true);
        } else {
            wreath_place(w, self.deed, from, self.loc, true, false);
        }
    }
}

fn wreath_on_target(w: &mut World, deed: ItemId, from: MobileId, t: Targeted) {
    let Some(loc) = target_point(w, t) else {
        return;
    };
    let map = w.mobile(from).and_then(|m| m.map);
    let house = muo_multis::house::access::find_house_at_point(w, loc, map, 16)
        .filter(|&h| muo_multis::house::access::is_co_owner(w, h, from));
    if house.is_some() {
        let north = addon::is_wall(w, loc.x, loc.y - 1, loc.z, map);
        let west = addon::is_wall(w, loc.x - 1, loc.y, loc.z, map);
        if north && west {
            if super::util::net_state(w, from).is_some() {
                send_gump_to_mobile(
                    w,
                    from,
                    Box::new(StaticGumpObj::new(150, 50, WreathDeedGump { deed, loc })),
                    false,
                );
            }
        } else {
            wreath_place(w, deed, from, loc, north, west);
        }
    } else {
        msg(w, from, 1042036); // That location is not in your house.
    }
}

leaf! {
    /// `Server.Items.WreathDeed` (`[Flippable(0x14F0, 0x14EF)]`).
    WREATH_DEED = "Server.Items.WreathDeed" : ITEM_BASE, enc;
    construct: Some(|w| {
        let hue = utility::random_dyed_hue(&mut w.rng);
        wreath_deed_new_with(w, hue)
    }),
    default_weight: |_, _| 1.0,
    label_number: |_, _| 1062837, // holiday wreath deed
    on_double_click: |w, id, from| {
        if super::util::in_pack(w, id, from) {
            let house = muo_multis::house::access::find_house_at_mobile(w, from);
            if house.is_some_and(|h| muo_multis::house::access::is_co_owner(w, h, from)) {
                msg(w, from, 1062838); // Where would you like to place this decoration?
                begin_target(w, from, move |w, from, t| wreath_on_target(w, id, from, t));
            } else {
                msg(w, from, 502092); // You must be in your house to do this.
            }
        } else {
            msg(w, from, 1042001); // That must be in your pack for you to use it.
        }
    },
}

pub fn register(types: &mut TypeRegistry) {
    types.register_item(&WREATH_ADDON, &[]);
    types.register_item(&WREATH_DEED, &[]);
}

pub(super) fn configure(w: &mut World) {
    house_deps::deps_mut(w).simple_addons.push(SimpleAddon {
        is: |w, id| w.item(id).is_some_and(|i| i.ty.is(&WREATH_ADDON)),
        deed: wreath_addon_deed,
        could_fit: wreath_could_fit,
    });
    register_dyable(w, &WREATH_ADDON, |w, id, from, tub| {
        if ops::is_deleted(w, id) {
            return false;
        }
        if co_owner_at_item(w, id, from) {
            if muo_content::mx::in_range_point(w, from, ops::get_world_location(w, id), 1) {
                let hue = muo_content::items::clothing::tub_dyed_hue(w, tub);
                super::hue(w, id, hue);
                return true;
            }
            msg(w, from, 500295); // You are too far away to do that.
        }
        false
    });
}
