//! `ReclaimVendorGump`, `VendorInventoryGump` and `ViewHousesGump`. The house itself is behind
//! [`HouseOps`](super::ops::HouseOps).

use std::rc::Rc;

use muo_content::gumps::GumpButtonType;
use muo_content::gumps::base::{
    DynamicGumpLogic, DynamicGumpObj, LegacyGumpLogic, LegacyGumpObj, RelayInfo,
};
use muo_content::gumps::builder::DynamicGumpBuilder;
use muo_primitives::html::{center_rgb, color_rgb, right_rgb};
use muo_server::World;
use muo_server::commands::{CommandEventArgs, CommandSystem};
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::mobile::AccessLevel;
use muo_server::net_state::NetStateId;
use muo_server::target::{TargetFlags, Targeted, begin_target};

use super::common::{has_net_state, mobile_of, name_of, send, send_singleton};
use super::hooks::ops;
use super::ops::{HouseOps, HouseView, VendorInventoryView};

fn house_ops(w: &World) -> Option<Rc<dyn HouseOps>> {
    ops(w).houses
}

// ---- ReclaimVendorGump ----

pub struct ReclaimVendorGump {
    house: ItemId,
    /// The vendors and their names.
    vendors: Vec<(MobileId, String)>,
}

/// `ReclaimVendorGump.DisplayTo(from, house)`.
pub fn reclaim_vendor_display_to(w: &mut World, from: MobileId, house: ItemId) {
    let Some(h) = house_ops(w) else { return };
    if !has_net_state(w, from) || h.deleted(w, house) {
        return;
    }
    let vendors = h.internalized_vendors(w, house);
    if vendors.is_empty() {
        return;
    }
    let vendors = vendors.into_iter().map(|v| (v, name_of(w, v))).collect();
    let g = DynamicGumpObj::new(50, 50, ReclaimVendorGump { house, vendors });
    send_singleton(w, from, g);
}

impl DynamicGumpLogic for ReclaimVendorGump {
    const TYPE_NAME: &'static str = "Server.Gumps.ReclaimVendorGump";
    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        let n = self.vendors.len() as i32;
        b.add_page(0);

        b.add_background(0, 0, 170, 50 + n * 20, 0x13BE);

        b.add_image_tiled(10, 10, 150, 20, 0xA40);
        b.add_html_localized_color(10, 10, 150, 20, 1061827, 0x7FFF, false, false); // <CENTER>Reclaim Vendor</CENTER>

        b.add_image_tiled(10, 40, 150, n * 20, 0xA40);

        for (i, (_, name)) in self.vendors.iter().enumerate() {
            let y = 40 + i as i32 * 20;

            b.add_button(10, y, 0xFA5, 0xFA7, i as i32 + 1);
            b.add_label(45, y, 0x481, name);
        }
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        let Some(from) = mobile_of(w, sender) else {
            return;
        };
        let Some(h) = house_ops(w) else { return };
        let house = self.house;

        if info.button_id == 0
            || !h.is_active(w, house)
            || !h.is_inside(w, house, from)
            || !h.is_owner(w, house, from)
            || !muo_server::mobile::part2::stats::check_alive(w, from, true)
        {
            return;
        }

        let index = info.button_id - 1;

        if index < 0 || index as usize >= self.vendors.len() {
            return;
        }

        let mob = self.vendors[index as usize].0;

        if !h.internalized_vendors(w, house).contains(&mob) {
            return;
        }

        if muo_content::mx::deleted(w, mob) {
            h.remove_internalized_vendor(w, house, mob);
        } else {
            let (loc, map) = w
                .mobile(from)
                .map_or((Default::default(), None), |m| (m.location, m.map));
            let (vendor, contract) = h.is_there_vendor(w, loc, map);

            if vendor {
                muo_content::mx::msg(w, from, 1062677); // You cannot place a vendor or barkeep at this location.
            } else if contract {
                muo_content::mx::msg(w, from, 1062678); // You cannot place a vendor or barkeep on top of a rental contract!
            } else {
                h.remove_internalized_vendor(w, house, mob);
                if let Some(map) = map {
                    muo_server::mobile::base::move_to_world(w, mob, loc, map);
                }
            }
        }
    }
}

// ---- VendorInventoryGump ----

pub struct VendorInventoryGump {
    house: ItemId,
    from: MobileId,
    inventories: Vec<VendorInventoryView>,
    /// `Core.Now` when the gump is sent; the layout is built in the same call.
    now: muo_primitives::DateTime,
}

/// `VendorInventoryGump.DisplayTo(from, house)`.
pub fn vendor_inventory_display_to(w: &mut World, from: MobileId, house: ItemId) {
    let Some(h) = house_ops(w) else { return };
    if !has_net_state(w, from) || h.deleted(w, house) {
        return;
    }
    let list = h.vendor_inventories(w, house);
    if list.is_empty() {
        return;
    }
    let inventories = list;
    let g = DynamicGumpObj::new(
        50,
        50,
        VendorInventoryGump {
            house,
            from,
            inventories,
            now: w.now,
        },
    );
    send_singleton(w, from, g);
}

impl DynamicGumpLogic for VendorInventoryGump {
    const TYPE_NAME: &'static str = "Server.Gumps.VendorInventoryGump";
    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        let n = self.inventories.len() as i32;
        b.add_page(0);

        b.add_background(0, 0, 420, 50 + 20 * n, 0x13BE);

        b.add_image_tiled(10, 10, 400, 20, 0xA40);
        b.add_html_localized_color(15, 10, 200, 20, 1062435, 0x7FFF, false, false); // Reclaim Vendor Inventory
        b.add_html_localized_color(330, 10, 50, 20, 1062465, 0x7FFF, false, false); // Expires

        b.add_image_tiled(10, 40, 400, 20 * n, 0xA40);

        for (i, inventory) in self.inventories.iter().enumerate() {
            let y = 40 + 20 * i as i32;

            if inventory.owner == Some(self.from) {
                b.add_button(10, y, 0xFA5, 0xFA7, i as i32 + 1);
            }

            b.add_label(
                45,
                y,
                0x481,
                &format!("{} ({})", inventory.shop_name, inventory.vendor_name),
            );

            let hours = (inventory.expire_time - self.now).total_hours() as i32;
            b.add_label(320, y, 0x481, &hours.to_string());
            b.add_html_localized_color(350, y, 50, 20, 1062466, 0x7FFF, false, false); // hour(s)
        }
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        if info.button_id == 0 {
            return;
        }

        let Some(from) = mobile_of(w, sender) else {
            return;
        };
        let Some(h) = house_ops(w) else { return };
        let house = self.house;
        let sign = h.sign(w, house);

        let sign = match sign {
            Some(s) if !h.deleted(w, house) && !muo_server::item::ops::is_deleted(w, s) => s,
            _ => return,
        };
        if !muo_server::mobile::part2::stats::check_alive(w, from, true) {
            return;
        }

        let (from_map, from_loc) = w
            .mobile(from)
            .map_or((None, Default::default()), |m| (m.map, m.location));
        let sign_map = w.item(sign).and_then(|i| i.map);
        let sign_loc = muo_server::item::ops::get_world_location(w, sign);
        if from_map != sign_map
            || !muo_primitives::utility::in_range(from_loc.x, from_loc.y, sign_loc.x, sign_loc.y, 5)
        {
            muo_content::mx::msg(w, from, 1062429); // You must be within five paces of the house sign to use this option.
            return;
        }

        let index = info.button_id - 1;
        if index < 0 || index as usize >= self.inventories.len() {
            return;
        }

        let id = self.inventories[index as usize].id;
        let Some(inventory) = h.vendor_inventory(w, house, id) else {
            return;
        };
        if inventory.owner != Some(from) {
            return;
        }

        use muo_server::item::TotalType;
        let item_total = |w: &World, item: ItemId| {
            1 + muo_server::item::ops::get_total(w, item, TotalType::Items)
        };

        let mut remaining_items = inventory.items.clone();
        let (mut total_items, mut given_to_backpack, mut given_to_bank_box) = (0, 0, 0);
        for i in (0..remaining_items.len()).rev() {
            let item = remaining_items[i];

            if muo_server::item::ops::is_deleted(w, item) {
                remaining_items.remove(i);
                h.inventory_remove_item(w, house, id, item);
                continue;
            }

            total_items += item_total(w, item);

            if muo_server::mobile::part2::items::place_in_backpack(w, from, item) {
                remaining_items.remove(i);
                h.inventory_remove_item(w, house, id, item);
                given_to_backpack += item_total(w, item);
            } else if muo_server::mobile::part1::props_b::bank_box(w, from).is_some_and(|bank| {
                muo_server::item::container::try_drop_item(w, bank, Some(from), item, false)
            }) {
                remaining_items.remove(i);
                h.inventory_remove_item(w, house, id, item);
                given_to_bank_box += item_total(w, item);
            }
        }

        // The vendor you selected had ~1_COUNT~ items in its inventory, and ~2_AMOUNT~ gold in its account.
        muo_content::mx::msg_args(
            w,
            from,
            1062436,
            &format!("{total_items}\t{}", inventory.gold),
        );

        let given_gold =
            muo_content::vendor::banker::deposit_up_to(w, from, inventory.gold, true, 0);
        let gold_left = inventory.gold - given_gold;
        h.inventory_set_gold(w, house, id, gold_left);

        // ~1_AMOUNT~ gold has been deposited into your bank box.
        muo_content::mx::msg_args(w, from, 1060397, &given_gold.to_string());

        // ~1_COUNT~ items have been removed from the shop inventory and placed in your backpack.
        // ~2_BANKCOUNT~ items were removed from the shop inventory and placed in your bank box.
        muo_content::mx::msg_args(
            w,
            from,
            1062437,
            &format!("{given_to_backpack}\t{given_to_bank_box}"),
        );

        if gold_left > 0 || !remaining_items.is_empty() {
            // Some of the shop inventory would not fit in your backpack or bank box.  Please free up some room and try again.
            muo_content::mx::msg(w, from, 1062440);
        } else {
            h.inventory_delete(w, house, id);
            muo_content::mx::msg(w, from, 1062438); // The shop is now empty of inventory and funds, so it has been deleted.
        }
    }
}

// ---- ViewHousesGump ----

const WHITE16: i32 = 0x7FFF;
const WHITE: i32 = 0xFFFFFF;

pub struct ViewHousesGump {
    from: MobileId,
    list: Rc<Vec<ItemId>>,
    selection: Option<ItemId>,
}

/// `new ViewHousesGump(from, list, sel)`.
pub fn new_view_houses(
    w: &World,
    from: MobileId,
    list: Rc<Vec<ItemId>>,
    sel: Option<ItemId>,
) -> LegacyGumpObj<ViewHousesGump> {
    let h = house_ops(w);
    let view: Option<HouseView> = match (&h, sel) {
        (Some(h), Some(s)) if !h.deleted(w, s) => h.view(w, s),
        _ => None,
    };
    let selection = view.as_ref().map(|v| v.house);
    let views: Vec<Option<HouseView>> = if view.is_none() {
        list.iter()
            .map(|&x| h.as_ref().and_then(|h| h.view(w, x)))
            .collect()
    } else {
        Vec::new()
    };

    let mut g = LegacyGumpObj::new(
        50,
        40,
        ViewHousesGump {
            from,
            list: list.clone(),
            selection,
        },
    );
    let l = &mut g.layout;

    l.add_page(0);

    l.add_background(0, 0, 240, 360, 5054);
    // AddBlackAlpha(10, 10, 220, 340)
    l.add_image_tiled(10, 10, 220, 340, 2624);
    l.add_alpha_region(10, 10, 220, 340);

    match view {
        None => {
            l.add_html(
                35,
                15,
                120,
                20,
                &color_rgb("House Type", WHITE),
                false,
                false,
            );

            if list.is_empty() {
                l.add_html(
                    35,
                    40,
                    160,
                    40,
                    &color_rgb("There were no houses found for that player.", WHITE),
                    false,
                    false,
                );
            }

            l.add_image(190, 17, 0x25EA, 0);
            l.add_image(207, 17, 0x25E6, 0);

            let mut page = 0;

            for (i, v) in views.iter().enumerate() {
                let i = i as i32;
                if i % 15 == 0 {
                    if page > 0 {
                        l.add_button_type(
                            207,
                            17,
                            0x15E1,
                            0x15E5,
                            0,
                            GumpButtonType::Page,
                            page + 1,
                        );
                    }

                    page += 1;
                    l.add_page(page);

                    if page > 1 {
                        l.add_button_type(
                            190,
                            17,
                            0x15E3,
                            0x15E7,
                            0,
                            GumpButtonType::Page,
                            page - 1,
                        );
                    }
                }

                let name = v.as_ref().map(|v| v.name.clone()).unwrap_or_default();

                l.add_html(
                    15,
                    40 + i % 15 * 20,
                    20,
                    20,
                    &color_rgb(&format!("{}.", i + 1), WHITE),
                    false,
                    false,
                );

                if name.number > 0 {
                    l.add_html_localized_color(
                        35,
                        40 + i % 15 * 20,
                        160,
                        20,
                        name.number,
                        WHITE16,
                        false,
                        false,
                    );
                } else {
                    l.add_html(
                        35,
                        40 + i % 15 * 20,
                        160,
                        20,
                        &color_rgb(name.string.as_deref().unwrap_or(""), WHITE),
                        false,
                        false,
                    );
                }

                l.add_button(198, 39 + i % 15 * 20, 4005, 4007, i + 1);
            }
        }
        Some(sel) => {
            let location = sel.sextant.clone().unwrap_or_else(|| "unknown".to_owned());
            let owner = &sel.owner_name;
            let map_name = sel.map_name.clone().unwrap_or_else(|| "(null)".to_owned());

            l.add_html(
                10,
                15,
                220,
                20,
                &center_rgb("House Properties", WHITE),
                false,
                false,
            );

            let mut row = |y: i32, label: &str, value: &str| {
                l.add_html(15, y, 210, 20, &color_rgb(label, WHITE), false, false);
                l.add_html(15, y, 210, 20, &right_rgb(value, WHITE), false, false);
            };
            row(40, "Facet:", &map_name);
            row(
                60,
                "Location:",
                &format!(
                    "({}, {}, {})",
                    sel.location.x, sel.location.y, sel.location.z
                ),
            );
            row(80, "Sextant:", &location);
            row(100, "Owner:", owner);
            row(120, "Name:", &sel.sign_name);
            row(140, "Friends:", &sel.friends.to_string());
            row(160, "Co-Owners:", &sel.co_owners.to_string());
            row(180, "Bans:", &sel.bans.to_string());
            row(200, "Decays:", if sel.can_decay { "Yes" } else { "No" });
            row(220, "Decay Level:", &sel.decay_level);

            l.add_button(15, 245, 4005, 4007, 1);
            l.add_html(
                50,
                245,
                120,
                20,
                &color_rgb("Go to house", WHITE),
                false,
                false,
            );

            l.add_button(15, 265, 4005, 4007, 2);
            l.add_html(
                50,
                265,
                120,
                20,
                &color_rgb("Open house menu", WHITE),
                false,
                false,
            );

            l.add_button(15, 285, 4005, 4007, 3);
            l.add_html(
                50,
                285,
                120,
                20,
                &color_rgb("Demolish house", WHITE),
                false,
                false,
            );

            l.add_button(15, 305, 4005, 4007, 4);
            l.add_html(
                50,
                305,
                120,
                20,
                &color_rgb("Refresh house", WHITE),
                false,
                false,
            );
        }
    }
    g
}

/// `ViewHousesGump.Configure`.
pub fn view_houses_configure(w: &mut World) {
    CommandSystem::register(
        w,
        "ViewHouses",
        AccessLevel::GameMaster,
        &[],
        Rc::new(view_houses_on_command),
    );
}

/// `ViewHouses_OnCommand`.
fn view_houses_on_command(w: &mut World, e: &mut CommandEventArgs) {
    begin_target(
        w,
        e.mobile,
        -1,
        false,
        TargetFlags::NONE,
        view_houses_on_target,
    );
}

/// `ViewHouses_OnTarget`.
fn view_houses_on_target(w: &mut World, from: MobileId, targeted: Targeted) {
    if let Targeted::Entity(EntityId::Mobile(mobile)) = targeted {
        let list = get_houses(w, mobile);
        let g = new_view_houses(w, from, Rc::new(list), None);
        send(w, from, g);
    }
}

/// `ViewHousesGump.GetHouses(owner)`.
pub fn get_houses(w: &World, owner: MobileId) -> Vec<ItemId> {
    house_ops(w).map_or_else(Vec::new, |h| h.houses_of(w, owner))
}

impl LegacyGumpLogic for ViewHousesGump {
    const TYPE_NAME: &'static str = "Server.Gumps.ViewHousesGump";
    fn singleton(&self) -> bool {
        true
    }

    fn on_response(&mut self, w: &mut World, _sender: NetStateId, info: &RelayInfo) {
        let from = self.from;
        let Some(h) = house_ops(w) else { return };

        match self.selection {
            None => {
                let v = info.button_id - 1;

                if v >= 0 && (v as usize) < self.list.len() {
                    let g =
                        new_view_houses(w, from, self.list.clone(), Some(self.list[v as usize]));
                    send(w, from, g);
                }
            }
            Some(sel) => {
                if h.deleted(w, sel) {
                    return;
                }
                match info.button_id {
                    0 => {
                        let g = new_view_houses(w, from, self.list.clone(), None);
                        send(w, from, g);
                    }
                    1 => {
                        let map = w.item(sel).and_then(|i| i.map);

                        if let Some(map) = map.filter(|&m| m != muo_server::map::MapId::INTERNAL) {
                            let p = h.ban_location(w, sel);
                            muo_server::mobile::base::move_to_world(w, from, p, map);
                        }

                        let g = new_view_houses(w, from, self.list.clone(), Some(sel));
                        send(w, from, g);
                    }
                    2 => {
                        let g = new_view_houses(w, from, self.list.clone(), Some(sel));
                        send(w, from, g);

                        h.open_sign_menu(w, sel, from);
                    }
                    3 => {
                        let g = new_view_houses(w, from, self.list.clone(), Some(sel));
                        send(w, from, g);
                        h.confirm_demolish(w, sel, from);
                    }
                    4 => {
                        h.refresh_decay(w, sel);
                        let g = new_view_houses(w, from, self.list.clone(), Some(sel));
                        send(w, from, g);
                    }
                    _ => {}
                }
            }
        }
    }
}

/// Sets the hook the houses open the vendor inventory gump through.
pub fn configure(w: &mut World) {
    muo_content::items::house_deps::deps_mut(w).display_vendor_inventory_gump =
        Some(vendor_inventory_display_to);
    super::hooks::hooks_mut(w).ops.houses = Some(Rc::new(super::house_ops::MultisHouseOps));
}
