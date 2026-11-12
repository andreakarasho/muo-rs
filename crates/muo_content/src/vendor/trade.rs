//! BaseVendor's trading: `OnBuyItems`, `OnSellItems`, `VendorBuy`, `VendorSell`, the display-entity
//! lookup, the purchase helpers and the context menu entries.

use muo_primitives::{Direction, Serial};
use muo_server::context_menu_entry::ContextMenuEntry;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::{container, ops};
use muo_server::layer::Layer;
use muo_server::mobile::AccessLevel;
use muo_server::mobile::part2::{items, messages, speech, stats};
use muo_server::packets::vendor_buy::{self, BuyItemState};
use muo_server::packets::vendor_sell::{self, SellItemState};
use muo_server::vendor::{BuyItemResponse, SellItemResponse};
use muo_server::world::World;

use super::buy::{self, BuyInfo, SellInfo};
use super::{
    BUY_LIST_LIMIT, Feature, MAX_SELL, add_to_buyback, buy_pack, class, data, feature,
    is_active_buyer, is_active_seller, restock, update_buy_info,
};
use crate::hooks::rnd;
use crate::items::gold::GOLD;
use crate::mx;

/// `_COFFEE`: the serial of a buy entry with no display entity.
const COFFEE: Serial = Serial::new(0x7FC0FFEE);

fn cls(w: &World, id: MobileId) -> &'static super::VendorClass {
    class(w, id).unwrap_or(&super::BASE_VENDOR_CLASS)
}

fn say_number(w: &mut World, id: MobileId, number: i32) {
    messages::say_number(w, id, number, "");
}

fn say_to(w: &mut World, id: MobileId, to: MobileId, number: i32, args: &str) {
    speech::say_to_number(w, id, to, number, args);
}

fn say_to_text(w: &mut World, id: MobileId, to: MobileId, text: &str) {
    speech::say_to_ascii(w, id, to, true, text);
}

fn delete_item(w: &mut World, id: ItemId) {
    if let Some(t) = w.item(id).map(|i| i.ty) {
        (t.delete)(w, id);
    }
}

/// `LookupDisplayObject(obj)`: the buy entry whose display entity is `obj`.
fn lookup_display_object(w: &World, vendor: MobileId, obj: EntityId) -> Option<usize> {
    data(w, vendor)?
        .buy_info
        .iter()
        .position(|b| b.display == Some(obj))
}

/// `ProcessSinglePurchase`: false when the whole order is refused.
fn process_single_purchase(
    buy: &BuyItemResponse,
    bii: &BuyInfo,
    valid_buy: &mut Vec<BuyItemResponse>,
    control_slots: &mut i32,
    full_purchase: &mut bool,
    total_cost: &mut i32,
) -> bool {
    let mut amount = buy.amount;
    if amount > bii.amount {
        amount = bii.amount;
    }
    if amount <= 0 {
        return false;
    }
    let slots = bii.control_slots().wrapping_mul(amount);
    if *control_slots >= slots {
        *control_slots -= slots;
    } else {
        *full_purchase = false;
        return false;
    }
    let total = i64::from(*total_cost) + i64::from(bii.price().wrapping_mul(amount));
    if total > i64::from(i32::MAX) {
        tracing::error!(
            "BaseVendor int overflow for Name/cliloc: {} ItemId: {} totalCost: {} adding: {} * {}",
            bii.name,
            bii.item_id,
            total,
            bii.price(),
            bii.amount
        );
        return false;
    }
    valid_buy.push(buy.clone());
    *total_cost = total as i32;
    true
}

fn drop_or_move(w: &mut World, cont: Option<ItemId>, buyer: MobileId, item: ItemId) {
    let dropped = cont.is_some_and(|c| container::try_drop_item(w, c, Some(buyer), item, false));
    if !dropped {
        let (loc, map) = w
            .mobile(buyer)
            .map_or_else(Default::default, |m| (m.location, m.map));
        ops::move_to_world_map(w, item, loc, map);
    }
}

/// `sound`: the first mobile of an order plays its idle sound before it is given a master.
fn give_mobile(w: &mut World, buyer: MobileId, m: MobileId, sound: bool) {
    let dir = Direction::from_bits_retain(rnd::below(w, 8) as u8);
    muo_server::mobile::part1::props_b::set_direction(w, m, dir);
    let (loc, map) = w
        .mobile(buyer)
        .map_or_else(Default::default, |b| (b.location, b.map));
    if let Some(map) = map {
        muo_server::mobile::base::move_to_world(w, m, loc, map);
    }
    if sound {
        let snd = muo_server::mobile::base::get_idle_sound(w, m);
        mx::play_sound(w, m, snd);
    }
    if is_creature(w, m) {
        crate::creature::control::set_control_master(w, m, Some(buyer));
        crate::creature::edit(w, m, |d| {
            d.control_order = crate::creature::enums::OrderType::Stop
        });
    }
}

fn is_creature(w: &World, m: MobileId) -> bool {
    crate::creature::is_creature(w, m)
}

/// `ProcessValidPurchase`.
fn process_valid_purchase(
    w: &mut World,
    mut amount: i32,
    index: usize,
    vendor: MobileId,
    buyer: MobileId,
    cont: Option<ItemId>,
) {
    let Some(bii) = data(w, vendor).and_then(|d| d.buy_info.get(index)).cloned() else {
        return;
    };
    if amount > bii.amount {
        amount = bii.amount;
    }
    if amount < 1 {
        return;
    }
    if let Some(b) = super::data_mut(w, vendor).and_then(|d| d.buy_info.get_mut(index)) {
        b.set_amount(bii.amount - amount);
    }

    match buy::get_entity(w, &bii) {
        Some(EntityId::Item(item)) => {
            if w.item(item).is_some_and(|i| i.stackable()) {
                ops::set_amount(w, item, amount);
                drop_or_move(w, cont, buyer, item);
            } else {
                ops::set_amount(w, item, 1);
                drop_or_move(w, cont, buyer, item);
                for _ in 1..amount {
                    if let Some(EntityId::Item(n)) = buy::get_entity(w, &bii) {
                        ops::set_amount(w, n, 1);
                        drop_or_move(w, cont, buyer, n);
                    }
                }
            }
        }
        Some(EntityId::Mobile(m)) => {
            give_mobile(w, buyer, m, true);
            for _ in 1..amount {
                if let Some(EntityId::Mobile(n)) = buy::get_entity(w, &bii) {
                    give_mobile(w, buyer, n, false);
                }
            }
        }
        None => {}
    }
}

fn buyer_backpack(w: &World, m: MobileId) -> Option<ItemId> {
    w.mobile(m)?.backpack
}

/// `OnBuyItems(buyer, list)`.
pub fn on_buy_items(
    w: &mut World,
    vendor: MobileId,
    buyer: MobileId,
    list: &[BuyItemResponse],
) -> bool {
    if !is_active_seller(w, vendor) {
        return false;
    }
    if !stats::check_alive(w, buyer, true) {
        return false;
    }
    if !(cls(w, vendor).check_vendor_access)(w, vendor, buyer) {
        say_number(w, vendor, 501522); // I shall not treat with scum like thee!
        return false;
    }

    update_buy_info(w, vendor);

    let info: Vec<SellInfo> = data(w, vendor)
        .map(|d| d.sell_info.clone())
        .unwrap_or_default();
    let mut total_cost = 0i32;
    let mut valid_buy: Vec<BuyItemResponse> = Vec::with_capacity(list.len());
    let mut from_bank = false;
    let mut full_purchase = true;
    let mut control_slots = w.mobile(buyer).map_or(0, |m| m.followers_max - m.followers);
    let pack = buy_pack(w, vendor);

    for buy in list {
        let ser = buy.serial;
        let mut amount = buy.amount;

        if ser.is_item() {
            let Some(item) = w.find_item(ser) else {
                continue;
            };
            let gbi = lookup_display_object(w, vendor, EntityId::Item(item));
            if let Some(idx) = gbi {
                let bii = data(w, vendor).map(|d| d.buy_info[idx].clone());
                if let Some(bii) = bii
                    && !process_single_purchase(
                        buy,
                        &bii,
                        &mut valid_buy,
                        &mut control_slots,
                        &mut full_purchase,
                        &mut total_cost,
                    )
                {
                    return false;
                }
            } else if Some(item) != pack
                && pack.is_some_and(|p| ops::is_child_of(w, item, Some(EntityId::Item(p))))
            {
                let item_amount = w.item(item).map_or(0, |i| i.amount);
                if amount > item_amount {
                    amount = item_amount;
                }
                if amount <= 0 {
                    continue;
                }
                for ssi in &info {
                    if ssi.is_sellable(w, item) {
                        total_cost = total_cost
                            .wrapping_add(ssi.get_buy_price_for(w, item).wrapping_mul(amount));
                        valid_buy.push(buy.clone());
                        break;
                    }
                }
            }
        } else if ser.is_mobile() {
            let Some(mob) = w.find_mobile(ser) else {
                continue;
            };
            if let Some(idx) = lookup_display_object(w, vendor, EntityId::Mobile(mob)) {
                let bii = data(w, vendor).map(|d| d.buy_info[idx].clone());
                if let Some(bii) = bii
                    && !process_single_purchase(
                        buy,
                        &bii,
                        &mut valid_buy,
                        &mut control_slots,
                        &mut full_purchase,
                        &mut total_cost,
                    )
                {
                    return false;
                }
            }
        }
    }

    if full_purchase && valid_buy.is_empty() {
        say_to(w, vendor, buyer, 500190, ""); // Thou hast bought nothing!
    } else if valid_buy.is_empty() {
        say_to(w, vendor, buyer, 500187, ""); // Your order cannot be fulfilled, please try again.
    }
    if valid_buy.is_empty() {
        return false;
    }

    let gm = mx::access_level(w, buyer) >= AccessLevel::GameMaster;
    let mut bought = gm;

    let cont = buyer_backpack(w, buyer);
    if !bought && let Some(c) = cont {
        if container::consume_total(w, c, &GOLD, total_cost, true, None) {
            bought = true;
        } else if total_cost < 2000 {
            say_to(w, vendor, buyer, 500192, ""); // Begging thy pardon, but thou canst not afford that.
        }
    }

    if !bought && total_cost >= 2000 {
        if super::banker::withdraw(w, buyer, total_cost) {
            bought = true;
            from_bank = true;
        } else {
            say_to(w, vendor, buyer, 500191, ""); // Begging thy pardon, but thy bank account lacks these funds.
        }
    }

    if !bought {
        return false;
    }

    mx::play_sound(w, buyer, 0x32);

    let cont = match buyer_backpack(w, buyer) {
        Some(c) => Some(c),
        None => muo_server::mobile::part1::props_b::bank_box(w, buyer),
    };

    for buy in &valid_buy {
        let ser = buy.serial;
        let mut amount = buy.amount;
        if amount < 1 {
            continue;
        }

        if ser.is_item() {
            let Some(item) = w.find_item(ser) else {
                continue;
            };
            if let Some(idx) = lookup_display_object(w, vendor, EntityId::Item(item)) {
                process_valid_purchase(w, amount, idx, vendor, buyer, cont);
            } else {
                let item_amount = w.item(item).map_or(0, |i| i.amount);
                if amount > item_amount {
                    amount = item_amount;
                }
                for ssi in &info {
                    if !ssi.is_sellable(w, item) {
                        continue;
                    }
                    let buy_item = if amount >= item_amount {
                        item
                    } else {
                        items::lift_item_dupe(w, item, item_amount - amount).unwrap_or(item)
                    };
                    drop_or_move(w, cont, buyer, buy_item);
                    break;
                }
            }
        } else if ser.is_mobile() {
            let Some(mob) = w.find_mobile(ser) else {
                continue;
            };
            if let Some(idx) = lookup_display_object(w, vendor, EntityId::Mobile(mob)) {
                process_valid_purchase(w, amount, idx, vendor, buyer, cont);
            }
        }
    }

    if full_purchase {
        if gm {
            say_to_text(
                w,
                vendor,
                buyer,
                "I would not presume to charge thee anything.  Here are the goods you requested.",
            );
        } else if from_bank {
            // The total of your purchase is ~1_val~ gold, which has been drawn from your bank account.  My thanks for the patronage.
            say_to(w, vendor, buyer, 1151638, &total_cost.to_string());
        } else {
            // The total of your purchase is ~1_val~ gold.  My thanks for the patronage.
            say_to(w, vendor, buyer, 1151639, &total_cost.to_string());
        }
    } else if gm {
        say_to_text(
            w,
            vendor,
            buyer,
            "I would not presume to charge thee anything.  Unfortunately, I could not sell you all the goods you requested.",
        );
    } else if from_bank {
        say_to_text(
            w,
            vendor,
            buyer,
            &format!(
                "The total of thy purchase is {total_cost} gold, which has been withdrawn from your bank account.  My thanks for the patronage.  Unfortunately, I could not sell you all the goods you requested."
            ),
        );
    } else {
        say_to_text(
            w,
            vendor,
            buyer,
            &format!(
                "The total of thy purchase is {total_cost} gold.  My thanks for the patronage.  Unfortunately, I could not sell you all the goods you requested."
            ),
        );
    }
    true
}

fn is_container_with_items(w: &World, item: ItemId) -> bool {
    w.item(item)
        .is_some_and(|i| i.ty.is(&container::CONTAINER) && !i.items.is_empty())
}

/// `resp.Item.RootParent != seller || resp.Amount <= 0 || !IsStandardLoot() || !Movable || container with items`.
fn sell_skipped(w: &World, seller: MobileId, resp: &SellItemResponse) -> bool {
    ops::root_parent(w, resp.item) != Some(EntityId::Mobile(seller))
        || resp.amount <= 0
        || !muo_server::item::base::is_standard_loot(w, resp.item)
        || !w.item(resp.item).is_some_and(|i| i.movable())
        || is_container_with_items(w, resp.item)
}

/// `OnSellItems(seller, list)`.
pub fn on_sell_items(
    w: &mut World,
    vendor: MobileId,
    seller: MobileId,
    list: &[SellItemResponse],
) -> bool {
    if !is_active_buyer(w, vendor) {
        return false;
    }
    if !stats::check_alive(w, seller, true) {
        return false;
    }
    if !(cls(w, vendor).check_vendor_access)(w, vendor, seller) {
        say_number(w, vendor, 501522); // I shall not treat with scum like thee!
        return false;
    }

    mx::play_sound(w, seller, 0x32);

    let info: Vec<SellInfo> = data(w, vendor)
        .map(|d| d.sell_info.clone())
        .unwrap_or_default();
    let buy_count = data(w, vendor).map_or(0, |d| d.buy_info.len() as i32);
    let buyback_capacity = (BUY_LIST_LIMIT as i32 - buy_count).max(0);
    let mut give_gold = 0i32;
    let mut sold = 0;

    for resp in list {
        if sell_skipped(w, seller, resp) {
            continue;
        }
        if info.iter().any(|ssi| ssi.is_sellable(w, resp.item)) {
            sold += 1;
        }
    }

    if sold > MAX_SELL {
        say_to_text(
            w,
            vendor,
            seller,
            &format!("You may only sell {MAX_SELL} items at a time!"),
        );
        return false;
    }
    if sold == 0 {
        return true;
    }

    for resp in list {
        if sell_skipped(w, seller, resp) {
            continue;
        }
        for ssi in &info {
            if !ssi.is_sellable(w, resp.item) {
                continue;
            }
            let item_amount = w.item(resp.item).map_or(0, |i| i.amount);
            let amount = resp.amount.min(item_amount);

            // The price is read before the item can be deleted by the buyback (a deleted item has no class here).
            let unit_price = ssi.get_sell_price_for(w, resp.item);

            // IsResellable is the same test as IsSellable, and `bii.Restock(item, amount)` is false for every
            // GenericBuyInfo, so a sale always goes to the buyback pack.
            let mut sold_item = resp.item;
            if amount < item_amount {
                sold_item =
                    items::lift_item_dupe(w, resp.item, item_amount - amount).unwrap_or(resp.item);
            }
            add_to_buyback(w, vendor, sold_item, buyback_capacity);

            give_gold = give_gold.wrapping_add(unit_price.wrapping_mul(amount));
            break;
        }
    }

    if give_gold > 0 {
        while give_gold > 60000 {
            add_gold(w, seller, 60000);
            give_gold -= 60000;
        }
        add_gold(w, seller, give_gold);

        mx::play_sound(w, seller, 0x0037); // Gold dropping sound

        if feature(w, Feature::BulkOrders)
            && (cls(w, vendor).supports_bulk_orders)(w, vendor, seller)
            && let Some(bod) = (cls(w, vendor).create_bulk_order)(w, vendor, seller, false)
            && let Some(f) = super::hooks(w).and_then(|h| h.send_bod_accept_gump)
        {
            f(w, seller, bod);
        }
    }
    true
}

fn add_gold(w: &mut World, to: MobileId, amount: i32) {
    if let Some(g) = crate::factory::make_item(w, "Server.Items.Gold") {
        ops::set_amount(w, g, amount);
        items::add_to_backpack(w, to, g);
    }
}

/// `SendPacksTo(from)`.
pub fn send_packs_to(w: &mut World, vendor: MobileId, from: MobileId) {
    let Some(ns) = w.mobile(from).and_then(|m| m.net_state) else {
        return;
    };
    if let Some(p) = buy_pack(w, vendor) {
        muo_server::packets::equipment::send_equip_update(w, ns, p);
    }
    if let Some(p) = items::find_item_on_layer(w, vendor, Layer::ShopSell) {
        muo_server::packets::equipment::send_equip_update(w, ns, p);
    }
    let pack = match items::find_item_on_layer(w, vendor, Layer::ShopResale) {
        Some(p) => Some(p),
        None => super::add_resale_pack(w, vendor),
    };
    if let Some(p) = pack {
        muo_server::packets::equipment::send_equip_update(w, ns, p);
    }
}

/// The bytes of an entity's cached `PropertyList`, built when needed.
fn opl_bytes(w: &mut World, e: EntityId) -> Option<Vec<u8>> {
    match e {
        EntityId::Item(i) => ops::property_list(w, i).map(|l| l.buffer().to_vec()),
        EntityId::Mobile(m) => {
            if w.mobile(m)?.property_list.is_none() {
                let mut list = muo_server::property_list::PropertyList::new(w.mobile(m)?.serial);
                muo_server::mobile::part2::properties::initialize_property_list(w, m, &mut list);
                w.mobile_mut(m)?.property_list = Some(list);
            }
            w.mobile(m)?
                .property_list
                .as_ref()
                .map(|l| l.buffer().to_vec())
        }
    }
}

/// `VendorBuy(from)`.
pub fn vendor_buy(w: &mut World, vendor: MobileId, from: MobileId) {
    if !is_active_seller(w, vendor) || !stats::check_alive(w, from, true) {
        return;
    }
    if !feature(w, Feature::VendorPurchase) {
        messages::send_message_hue(w, from, 0x22, "Vendor purchases are temporarily disabled.");
        return;
    }
    if !(cls(w, vendor).check_vendor_access)(w, vendor, from) {
        say_number(w, vendor, 501522); // I shall not treat with scum like thee!
        return;
    }

    let last = data(w, vendor).map_or(w.now, |d| d.last_restock);
    if w.now.since(last) > (cls(w, vendor).restock_delay)(w, vendor) {
        restock(w, vendor);
    }

    update_buy_info(w, vendor);

    let buy_len = data(w, vendor).map_or(0, |d| d.buy_info.len());
    let sell_info: Vec<SellInfo> = data(w, vendor)
        .map(|d| d.sell_info.clone())
        .unwrap_or_default();
    let mut list: Vec<BuyItemState> = Vec::with_capacity(buy_len);
    let Some(cont) = buy_pack(w, vendor) else {
        return;
    };
    let cont_serial = w.item(cont).map(|i| i.serial).unwrap_or(Serial::MINUS_ONE);
    let want_opl = w
        .resources
        .get::<super::VendorStatics>()
        .is_some_and(|s| s.enable_vendor_buy_opl);
    let mut opls: Vec<Vec<u8>> = Vec::new();

    for idx in 0..buy_len {
        let Some(b) = data(w, vendor).map(|d| d.buy_info[idx].clone()) else {
            break;
        };
        if b.amount <= 0 || list.len() >= BUY_LIST_LIMIT {
            continue;
        }
        let disp = buy::get_display_entity(w, vendor, idx);
        let serial = match disp {
            Some(EntityId::Item(i)) => w.item(i).map(|i| i.serial),
            Some(EntityId::Mobile(m)) => w.mobile(m).map(|m| m.serial),
            None => None,
        }
        .unwrap_or(COFFEE);
        list.push(BuyItemState::new(
            Some(&b.name),
            cont_serial,
            serial,
            b.price(),
            b.amount,
            b.item_id,
            b.hue,
        ));
        if want_opl
            && let Some(e) = disp
            && let Some(buf) = opl_bytes(w, e)
        {
            opls.push(buf);
        }
    }

    let player_items: Vec<ItemId> = w.item(cont).map(|i| i.items.clone()).unwrap_or_default();
    for item in player_items {
        let mut price = 0;
        let mut name: Option<String> = None;
        for ssi in &sell_info {
            if ssi.is_sellable(w, item) {
                price = ssi.get_buy_price_for(w, item);
                name = Some(ssi.get_name_for(w, item));
                break;
            }
        }
        if let Some(name) = name
            && list.len() < BUY_LIST_LIMIT
            && let Some(it) = w.item(item)
        {
            list.push(BuyItemState::new(
                Some(&name),
                cont_serial,
                it.serial,
                price,
                it.amount,
                it.item_id,
                it.hue,
            ));
            if let Some(buf) = opl_bytes(w, EntityId::Item(item)) {
                opls.push(buf);
            }
        }
    }

    if list.is_empty() {
        return;
    }

    list.sort_by_key(|a| a.my_serial.value());

    send_packs_to(w, vendor, from);

    let Some(ns) = w.mobile(from).and_then(|m| m.net_state) else {
        return;
    };
    if w.cannot_send_packets(ns) {
        return;
    }
    let vserial = w
        .mobile(vendor)
        .map(|m| m.serial)
        .unwrap_or(Serial::MINUS_ONE);
    vendor_buy::send_vendor_buy_content(w, ns, &list);
    vendor_buy::send_vendor_buy_list(w, ns, vendor, &list);
    vendor_buy::send_display_buy_list(w, ns, vserial);
    muo_server::packets::mobile::send_mobile_status_self(w, ns, from); // make sure their gold amount is sent

    for buf in opls {
        if let Some(ns) = w.mobile(from).and_then(|m| m.net_state) {
            w.send(ns, &buf);
        }
    }

    say_to(w, vendor, from, 500186, ""); // Greetings.  Have a look around.
}

/// `VendorSell(from)`.
pub fn vendor_sell(w: &mut World, vendor: MobileId, from: MobileId) {
    if !is_active_buyer(w, vendor) || !stats::check_alive(w, from, true) {
        return;
    }
    if !feature(w, Feature::VendorSell) {
        messages::send_message_hue(w, from, 0x22, "Vendor sales are temporarily disabled.");
        return;
    }
    if !(cls(w, vendor).check_vendor_access)(w, vendor, from) {
        say_number(w, vendor, 501522); // I shall not treat with scum like thee!
        return;
    }

    let Some(pack) = buyer_backpack(w, from) else {
        return;
    };
    let info: Vec<SellInfo> = data(w, vendor)
        .map(|d| d.sell_info.clone())
        .unwrap_or_default();

    // HashSet<SellItemState>(SellItemStateComparer): one entry per item, in insertion order.
    let mut set: Vec<SellItemState> = Vec::new();
    for ssi in &info {
        let found = muo_server::item::enumerable::find_items(w, pack, true, |_, _| true);
        for item in found {
            if !ssi.in_type_list(w, item) {
                continue;
            }
            if is_container_with_items(w, item) {
                continue;
            }
            if muo_server::item::base::is_standard_loot(w, item)
                && w.item(item).is_some_and(|i| i.movable())
                && ssi.is_sellable(w, item)
                && !set.iter().any(|s| s.item == item)
            {
                set.push(SellItemState {
                    item,
                    price: ssi.get_sell_price_for(w, item),
                    name: Some(ssi.get_name_for(w, item)),
                });
            }
        }
    }

    if !set.is_empty() {
        send_packs_to(w, vendor, from);
        let vserial = w
            .mobile(vendor)
            .map(|m| m.serial)
            .unwrap_or(Serial::MINUS_ONE);
        if let Some(ns) = w.mobile(from).and_then(|m| m.net_state) {
            vendor_sell::send_vendor_sell_list(w, ns, vserial, &set);
        }
    } else {
        messages::say(
            w,
            vendor,
            true,
            "You have nothing I would be interested in.",
        );
    }
}

/// `AddCustomContextEntries(from, ref list)`.
pub fn add_custom_context_entries(
    w: &mut World,
    vendor: MobileId,
    from: MobileId,
    list: &mut Vec<ContextMenuEntry>,
) {
    if mx::alive(w, from) && super::is_active_vendor(w, vendor) {
        let c = cls(w, vendor);
        if feature(w, Feature::BulkOrders) && (c.supports_bulk_orders)(w, vendor, from) {
            list.push(bulk_order_info_entry());
        }
        let access = (c.check_vendor_access)(w, vendor, from);
        if feature(w, Feature::VendorPurchase) && is_active_seller(w, vendor) {
            list.push(vendor_entry(6103, access, true));
        }
        if feature(w, Feature::VendorSell) && is_active_buyer(w, vendor) {
            list.push(vendor_entry(6104, access, false));
        }
    }
    (crate::creature::BASE_CREATURE_C.add_custom_context_entries)(w, vendor, from, list);
}

/// `VendorBuyEntry` / `VendorSellEntry`: `base(6103/6104, 8)`.
fn vendor_entry(number: i32, enabled: bool, buy: bool) -> ContextMenuEntry {
    let mut e = ContextMenuEntry::new(number, 8);
    e.enabled = enabled;
    e.on_click = Some(std::rc::Rc::new(move |w: &mut World, from, target| {
        if let EntityId::Mobile(v) = target
            && let Some(c) = class(w, v)
        {
            if buy {
                (c.vendor_buy)(w, v, from);
            } else {
                (c.vendor_sell)(w, v, from);
            }
        }
    }));
    e
}

/// `BulkOrderInfoEntry`: `base(6152)`.
fn bulk_order_info_entry() -> ContextMenuEntry {
    let mut e = ContextMenuEntry::new(6152, -1);
    e.on_click = Some(std::rc::Rc::new(|w: &mut World, from, target| {
        let EntityId::Mobile(vendor) = target else {
            return;
        };
        let Some(c) = class(w, vendor) else { return };
        if !feature(w, Feature::BulkOrders) || !(c.supports_bulk_orders)(w, vendor, from) {
            return;
        }
        let ts = (c.get_next_bulk_order)(w, vendor, from);
        let total_seconds = ts.total_seconds();

        // Let them get a bulk order if they are within 1 second.
        if total_seconds < 1.0 {
            mx::msg(w, from, 1049038); // You can get an order now.
            if w.core_aos()
                && let Some(bod) = (c.create_bulk_order)(w, vendor, from, true)
                && let Some(f) = super::hooks(w).and_then(|h| h.send_bod_accept_gump)
            {
                f(w, from, bod);
            }
        } else {
            let old = w.mobile(vendor).map_or(0, |m| m.speech_hue);
            if let Some(m) = w.mobile_mut(vendor) {
                m.speech_hue = 0x3B2;
            }
            if w.core_se() {
                // An offer may be available in about ~1_minutes~ minutes.
                let n = format!("{:.0}", (total_seconds / 60.0).ceil());
                say_to(w, vendor, from, 1072058, &n);
            } else {
                // An offer may be available in about ~1_hours~ hours.
                let n = format!("{:.0}", (total_seconds / 3600.0).ceil());
                say_to(w, vendor, from, 1049039, &n);
            }
            if let Some(m) = w.mobile_mut(vendor) {
                m.speech_hue = old;
            }
        }
    }));
    e
}

#[allow(dead_code)]
fn _unused() {
    let _ = delete_item;
}
