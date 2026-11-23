//! The veteran reward `HangingSkeleton`. See `vet.rs` for the shared parts.

use muo_content::gumps::base::{
    DynamicGumpLogic, DynamicGumpObj, RelayInfo, StaticGumpLogic, StaticGumpObj,
};
use muo_content::gumps::builder::{DynamicGumpBuilder, StaticGumpBuilder};
use muo_content::gumps::send_gump_to_mobile;
use muo_content::items::addon;
use muo_content::items::house_deps::{self, SimpleAddon};
use muo_primitives::Point3D;
use muo_server::World;
use muo_server::ids::{ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType, LootType, deps, ops};
use muo_server::map::MapId;
use muo_server::net_state::NetStateId;
use muo_server::target::Targeted;
use muo_server::type_registry::TypeRegistry;

use super::hooks::hooks;
use super::util::{in_pack, msg, overhead};
use super::vet::*;

/// `HangingSkeleton(itemID = 0x1596)`.
pub fn hanging_skeleton_new(w: &mut World, item_id: i32) -> ItemId {
    let id = ops::new_item_with_id(w, &HANGING_SKELETON, item_id);
    ops::set_loot_type(w, id, LootType::Blessed);
    ops::set_movable(w, id, false);
    id
}

fn skeleton_could_fit(w: &mut World, id: ItemId, p: Point3D, map: MapId) -> bool {
    let (item_id, height) = {
        let Some(i) = w.item(id) else { return false };
        (i.item_id, item_height(w, i.item_id))
    };
    if !muo_server::map::fit::can_fit_item(w, map, p.x, p.y, p.z, height) {
        return false;
    }
    if matches!(item_id, 0x1A03 | 0x1A05 | 0x1A09 | 0x1B1E | 0x1B7F) {
        addon::is_wall(w, p.x, p.y - 1, p.z, Some(map)) // north wall
    } else {
        addon::is_wall(w, p.x - 1, p.y, p.z, Some(map)) // west wall
    }
}

fn skeleton_deed(w: &mut World, id: ItemId) -> ItemId {
    let d = hanging_skeleton_deed_new(w);
    let r = is_reward(w, id);
    set_reward(w, d, r);
    d
}

rtype! {
    /// `Server.Items.HangingSkeleton` (`IAddon`).
    HANGING_SKELETON = "Server.Items.HangingSkeleton" : ITEM_BASE;
    construct: Some(|w| hanging_skeleton_new(w, 0x1596)),
    force_show_properties: |w, _| w.core.opl_enabled,
    get_properties: |w, id, list| {
        (ITEM_BASE.get_properties)(w, id, list);
        if deps::core_ml(w) && is_reward(w, id) {
            list.add(1076220); // 4th Year Veteran Reward
        }
    },
    on_double_click: |w, id, from| {
        let loc = w.item(id).map_or_else(Point3D::default, |i| i.location);
        if !muo_content::mx::in_range_point(w, from, loc, 3) {
            overhead(w, from, 0x3B2, 1019045); // I can't reach that.
            return;
        }
        let house = muo_multis::house::access::find_house_at_item(w, id);
        if house.is_some_and(|h| muo_multis::house::access::is_owner(w, h, from)) {
            if let Some(f) = hooks(w).and_then(|h| h.reward_demolition_gump) {
                f(w, from, id, 1049783); // Do you wish to re-deed this decoration?
            }
        } else {
            // You can only re-deed this decoration if you are the house owner or originally placed the decoration.
            msg(w, from, 1049784);
        }
    },
}

/// `HangingSkeletonDeed.GetWestItemID(south)`.
pub fn skeleton_west_item_id(south: i32) -> i32 {
    match south {
        0x1B1E => 0x1B1D,
        0x1B7F => 0x1B7C,
        _ => south + 1,
    }
}

/// `HangingSkeletonDeed()`.
pub fn hanging_skeleton_deed_new(w: &mut World) -> ItemId {
    crate::simple::from_template(
        w,
        &HANGING_SKELETON_DEED,
        "Server.Items.HangingSkeletonDeed",
    )
}

const SKELETONS: [i32; 5] = [0x1A03, 0x1A05, 0x1A09, 0x1B1E, 0x1B7F];

/// `HangingSkeletonDeed.HangingSkeletonGump`.
struct SkeletonGump {
    deed: ItemId,
}

impl StaticGumpLogic for SkeletonGump {
    const TYPE_NAME: &'static str = "Server.Items.HangingSkeletonDeed+HangingSkeletonGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut StaticGumpBuilder) {
        b.add_page(0);
        b.add_background(25, 0, 500, 230, 0xA28);
        b.add_page(1);
        for (x, bx, g) in [
            (130, 150, 0x1A03),
            (190, 210, 0x1A05),
            (250, 270, 0x1A09),
            (310, 330, 0x1B1E),
            (370, 390, 0x1B7F),
        ] {
            b.add_item(x, 70, g, 0);
            b.add_button(bx, 50, 0x845, 0x846, g);
        }
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        if ops::is_deleted(w, self.deed) || !SKELETONS.contains(&info.button_id) {
            return;
        }
        let Some(m) = w.net_states.get(&sender).and_then(|n| n.mobile) else {
            return;
        };
        msg(w, m, 1049780); // Where would you like to place this decoration?
        let (deed, item_id) = (self.deed, info.button_id);
        begin_target(w, m, move |w, from, t| {
            skeleton_on_target(w, deed, item_id, from, t)
        });
    }
}

fn skeleton_on_target(
    w: &mut World,
    deed: ItemId,
    item_id: i32,
    from: MobileId,
    targeted: Targeted,
) {
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
        if super::util::net_state(w, from).is_some() {
            let g = SkeletonFacingGump {
                deed,
                item_id,
                location: p,
                house,
            };
            send_gump_to_mobile(w, from, Box::new(DynamicGumpObj::new(150, 50, g)), false);
        }
    } else if north || west {
        let s = hanging_skeleton_new(
            w,
            if west {
                skeleton_west_item_id(item_id)
            } else {
                item_id
            },
        );
        addon::house_addons_add(w, house, s);
        set_reward(w, s, is_reward(w, deed));
        ops::move_to_world_map(w, s, p, Some(map));
        super::util::delete(w, deed);
    } else {
        msg(w, from, 1042039); // The banner must be placed next to a wall.
    }
}

/// `HangingSkeletonDeed.InternalTarget.FacingGump`.
struct SkeletonFacingGump {
    deed: ItemId,
    item_id: i32,
    location: Point3D,
    house: ItemId,
}

impl DynamicGumpLogic for SkeletonFacingGump {
    const TYPE_NAME: &'static str = "Server.Items.HangingSkeletonDeed+InternalTarget+FacingGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        b.set_no_resize();
        b.add_page(0);
        b.add_background(0, 0, 300, 150, 0xA28);
        b.add_item(90, 30, skeleton_west_item_id(self.item_id), 0);
        b.add_item(180, 30, self.item_id, 0);
        b.add_button(50, 35, 0x868, 0x869, 2); // East
        b.add_button(145, 35, 0x868, 0x869, 1); // South
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        if ops::is_deleted(w, self.deed) || info.button_id == 0 {
            return;
        }
        let Some(m) = w.net_states.get(&sender).and_then(|n| n.mobile) else {
            return;
        };
        let map = w.mobile(m).and_then(|x| x.map);
        let s = match info.button_id {
            2 => hanging_skeleton_new(w, skeleton_west_item_id(self.item_id)),
            1 => hanging_skeleton_new(w, self.item_id),
            _ => return,
        };
        addon::house_addons_add(w, self.house, s);
        set_reward(w, s, is_reward(w, self.deed));
        ops::move_to_world_map(w, s, self.location, map);
        super::util::delete(w, self.deed);
    }
}

rtype! {
    /// `Server.Items.HangingSkeletonDeed`.
    HANGING_SKELETON_DEED = "Server.Items.HangingSkeletonDeed" : ITEM_BASE;
    construct: Some(hanging_skeleton_deed_new),
    default_weight: crate::class_props::default_weight,
    label_number: crate::class_props::label_number, // deed for a hanging skeleton decoration
    get_properties: |w, id, list| {
        (ITEM_BASE.get_properties)(w, id, list);
        if is_reward(w, id) {
            list.add(1076220); // 4th Year Veteran Reward
        }
    },
    on_double_click: |w, id, from| {
        if reward_blocked(w, from, id) {
            return;
        }
        if !in_pack(w, id, from) {
            msg(w, from, 1042038); // You must have the object in your backpack to use it.
            return;
        }
        if owned_house_at_mobile(w, from).is_none() {
            msg(w, from, 502092); // You must be in your house to do this.
            return;
        }
        if super::util::net_state(w, from).is_some() {
            send_gump_to_mobile(w, from, Box::new(StaticGumpObj::new(100, 200, SkeletonGump { deed: id })), false);
        }
    },
}

pub fn register(types: &mut TypeRegistry) {
    types.register_item(&HANGING_SKELETON, &[]);
    types.register_item(&HANGING_SKELETON_DEED, &[]);
}

pub(super) fn configure(w: &mut World) {
    house_deps::deps_mut(w).simple_addons.push(SimpleAddon {
        is: |w, id| w.item(id).is_some_and(|i| i.ty.is(&HANGING_SKELETON)),
        deed: skeleton_deed,
        could_fit: skeleton_could_fit,
    });
}
