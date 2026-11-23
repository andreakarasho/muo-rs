//! `PlayerVendor`, `VendorItem`, `VendorBackpack` and `PlayerVendorPlaceholder`. A `PlayerVendor`
//! is a plain `Mobile`; `[SerializationGenerator(4, false)]`. Its house is the `muo_multis` house (the house keeps
//! the vendor in `player_vendors`), the owner and buy gumps are `muo_gumps::m_z::player_vendor`.

use std::rc::Rc;

use indexmap::IndexMap;
use muo_content::creature::BASE_CREATURE_C;
use muo_content::items::cont::BACKPACK;
use muo_content::vendor::banker;
use muo_content::{factory, mx};
use muo_io::{GenericReader, GenericWriter};
use muo_multis::house;
use muo_primitives::{DateTime, MessageType, Point3D, TimeSpan};
use muo_server::World;
use muo_server::context_menu_entry::ContextMenuEntry;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType, container, ops};
use muo_server::layer::Layer;
use muo_server::map::MapId;
use muo_server::mobile::part1::{props_a, props_b};
use muo_server::mobile::part2::{items, speech};
use muo_server::mobile::{AccessLevel, MOBILE_BASE, MobileType};
use muo_server::prompt::{Prompt, add_prompt};
use muo_server::property_list::PropertyList;
use muo_server::serialization_ext::{ReadEntityExt, WriteEntityExt};
use muo_server::target::{self, TargetFlags, Targeted};
use muo_server::timer::{self, TimerHandle};
use muo_server::type_registry::TypeRegistry;

/// What the vendors ask of systems above this crate.
#[derive(Default)]
pub struct PlayerVendorHooks {
    /// `ContentFeatureFlags.PlayerVendors`. Unset: enabled.
    pub feature_enabled: Option<fn(&World) -> bool>,
    /// `Ethic.IsImbued(item, true)`. Unset: false.
    pub is_imbued: Option<fn(&World, ItemId) -> bool>,
}

fn pv_hooks(w: &World) -> Option<&PlayerVendorHooks> {
    w.resources.get::<PlayerVendorHooks>()
}

fn new_vendor_system(w: &World) -> bool {
    house::new_vendor_system(w)
}

// ---- VendorItem ----

/// `VendorItem`: the price and description of an item for sale.
#[derive(Clone, Debug)]
pub struct VendorItem {
    pub item: Option<ItemId>,
    pub price: i32,
    pub description: String,
    pub created: DateTime,
    /// `Valid`.
    pub valid: bool,
}

impl VendorItem {
    /// `IsForSale`.
    pub fn is_for_sale(&self) -> bool {
        self.price >= 0
    }

    /// `IsForFree`.
    pub fn is_for_free(&self) -> bool {
        self.price == 0
    }

    /// `FormattedPrice`: with thousands separators from the ML on.
    pub fn formatted_price(&self, ml: bool) -> String {
        if !ml {
            return self.price.to_string();
        }
        let digits = self.price.unsigned_abs().to_string();
        let mut out = String::new();
        for (i, c) in digits.chars().enumerate() {
            if i > 0 && (digits.len() - i).is_multiple_of(3) {
                out.push(',');
            }
            out.push(c);
        }
        if self.price < 0 {
            format!("-{out}")
        } else {
            out
        }
    }
}

// ---- state ----

/// What `PlayerVendor` adds to a `Mobile`.
#[derive(Default)]
pub struct PlayerVendorData {
    pub shop_name: Option<String>,
    pub next_pay_time: DateTime,
    pub house: Option<ItemId>,
    pub owner: Option<MobileId>,
    pub bank_account: i32,
    pub hold_gold: i32,
    pub sell_items: IndexMap<ItemId, VendorItem>,
    pub pay_timer: TimerHandle,
    /// `Placeholder`.
    pub placeholder: Option<ItemId>,
}

pub fn data(w: &World, id: MobileId) -> Option<&PlayerVendorData> {
    w.mobile(id)?.ext.get::<PlayerVendorData>()
}

pub fn data_mut(w: &mut World, id: MobileId) -> Option<&mut PlayerVendorData> {
    w.mobile_mut(id)?.ext.get_mut::<PlayerVendorData>()
}

/// `m is PlayerVendor`.
pub fn is_player_vendor(w: &World, m: MobileId) -> bool {
    data(w, m).is_some()
}

/// The virtuals the `PlayerVendor` subclasses (`RentedVendor`, `PlayerBarkeeper`) override.
pub struct PvClass {
    /// `IsOwner(m)`.
    pub is_owner: fn(&World, MobileId, MobileId) -> bool,
    /// `Destroy(toBackpack)`.
    pub destroy: fn(&mut World, MobileId, bool),
}

pub static BASE_PV_CLASS: PvClass = PvClass {
    is_owner: base_is_owner,
    destroy: destroy_base,
};

fn pclass(w: &World, id: MobileId) -> &'static PvClass {
    w.mobile(id)
        .and_then(|m| m.ty.family)
        .and_then(|f| f.downcast_ref::<PvClass>())
        .unwrap_or(&BASE_PV_CLASS)
}

/// `ShopName`.
pub fn shop_name(w: &World, id: MobileId) -> String {
    data(w, id)
        .and_then(|d| d.shop_name.clone())
        .unwrap_or_default()
}

/// `ShopName = value`.
pub fn set_shop_name(w: &mut World, id: MobileId, value: Option<String>) {
    if let Some(d) = data_mut(w, id) {
        if d.shop_name == value {
            return;
        }
        d.shop_name = value;
    }
    muo_server::mobile::part2::properties::invalidate_properties(w, id);
}

pub fn owner(w: &World, id: MobileId) -> Option<MobileId> {
    data(w, id)?.owner
}

pub fn hold_gold(w: &World, id: MobileId) -> i32 {
    data(w, id).map_or(0, |d| d.hold_gold)
}

pub fn set_hold_gold(w: &mut World, id: MobileId, value: i32) {
    if let Some(d) = data_mut(w, id) {
        d.hold_gold = value;
    }
}

pub fn bank_account(w: &World, id: MobileId) -> i32 {
    data(w, id).map_or(0, |d| d.bank_account)
}

pub fn set_bank_account(w: &mut World, id: MobileId, value: i32) {
    if let Some(d) = data_mut(w, id) {
        d.bank_account = value;
    }
}

pub fn house_of(w: &World, id: MobileId) -> Option<ItemId> {
    data(w, id)?.house
}

/// `House = value` (`OnHouseChanged`: the old house forgets the vendor, the new one lists it).
pub fn set_house(w: &mut World, id: MobileId, value: Option<ItemId>) {
    let old = house_of(w, id);
    if old == value {
        return;
    }
    if let Some(d) = data_mut(w, id) {
        d.house = value;
    }
    if let Some(h) = old.and_then(|h| house::data_mut(w, h)) {
        h.player_vendors.retain(|&v| v != id);
    }
    if let Some(h) = value.and_then(|h| house::data_mut(w, h)) {
        h.player_vendors.push(id);
    }
}

/// `GetVendorItem(item)`.
pub fn get_vendor_item(w: &World, id: MobileId, item: ItemId) -> Option<&VendorItem> {
    data(w, id)?.sell_items.get(&item)
}

fn total_price(w: &World, id: MobileId) -> i64 {
    data(w, id).map_or(0, |d| {
        d.sell_items.values().map(|v| i64::from(v.price)).sum()
    })
}

/// `ChargePerDay`.
pub fn charge_per_day(w: &World, id: MobileId) -> i32 {
    if new_vendor_system(w) {
        return charge_per_real_world_day(w, id) / 12;
    }
    (20 + (total_price(w, id) - 500).max(0) / 500) as i32
}

/// `ChargePerRealWorldDay`.
pub fn charge_per_real_world_day(w: &World, id: MobileId) -> i32 {
    if new_vendor_system(w) {
        return (60 + total_price(w, id) / 500 * 3) as i32;
    }
    charge_per_day(w, id) * 12
}

// ---- construction ----

/// `PayTimer.GetInterval()`.
fn pay_interval(w: &World) -> TimeSpan {
    if new_vendor_system(w) {
        TimeSpan::from_days(1)
    } else {
        TimeSpan::from_mins(120) // Clock.MinutesPerUODay
    }
}

fn start_pay_timer(w: &mut World, id: MobileId, delay: TimeSpan) {
    let interval = pay_interval(w);
    let h = timer::start(w, delay, interval, 0, move |w| pay_tick(w, id));
    if let Some(d) = data_mut(w, id) {
        d.pay_timer = h;
    }
}

/// `new PlayerVendor(owner, house)` as the class `ty`.
pub fn make(
    w: &mut World,
    ty: &'static MobileType,
    owner: Option<MobileId>,
    house_id: Option<ItemId>,
) -> MobileId {
    let id = w.new_mobile(ty);
    if let Some(m) = w.mobile_mut(id) {
        m.ext.insert(PlayerVendorData::default());
    }
    if let Some(d) = data_mut(w, id) {
        d.owner = owner;
    }
    set_house(w, id, house_id);
    if new_vendor_system(w) {
        set_bank_account(w, id, 0);
        set_hold_gold(w, id, 4);
    } else {
        set_bank_account(w, id, 1000);
        set_hold_gold(w, id, 0);
    }
    set_shop_name(w, id, Some("Shop Not Yet Named".to_owned()));
    if let Some(m) = w.mobile_mut(id) {
        m.cant_walk = true;
    }
    // the name hue, stats, InitBody() and InitOutfit()
    muo_content::templates::apply_mobile(w, id);
    let delay = pay_interval(w);
    start_pay_timer(w, id, delay);
    let next = w.now + delay;
    if let Some(d) = data_mut(w, id) {
        d.next_pay_time = next;
    }
    id
}

fn set_name_hue(w: &mut World, id: MobileId, v: i32) {
    if let Some(m) = w.mobile_mut(id) {
        m.name_hue = v;
    }
}

/// `Backpack` without creating one.
fn pack_of(w: &World, id: MobileId) -> Option<ItemId> {
    items::find_item_on_layer(w, id, Layer::Backpack)
}

/// `1 + item.TotalItems`.
fn count_items(w: &World, item: ItemId) -> i32 {
    1 + w.item(item).map_or(0, |i| {
        i.items.iter().map(|&c| count_items(w, c)).sum::<i32>()
    })
}

/// `house.InRange(location, range)`.
fn house_in_range(w: &World, h: ItemId, p: Point3D, range: i32) -> bool {
    let Some(r) = house::region(w, h).and_then(|r| w.region(r)) else {
        return false;
    };
    r.area.iter().any(|rect| {
        p.x >= rect.start.x - range
            && p.y >= rect.start.y - range
            && p.x < rect.end.x + range
            && p.y < rect.end.y + range
    })
}

// ---- ownership and items ----

fn base_is_owner(w: &World, id: MobileId, m: MobileId) -> bool {
    if mx::access_level(w, m) >= AccessLevel::GameMaster {
        return true;
    }
    if new_vendor_system(w)
        && let Some(h) = house_of(w, id)
    {
        return house::access::is_owner(w, h, m);
    }
    owner(w, id) == Some(m)
}

/// `IsOwner(m)`.
pub fn is_owner(w: &World, id: MobileId, m: MobileId) -> bool {
    (pclass(w, id).is_owner)(w, id, m)
}

/// `GetItems()`: the movable worn items (but the pack and the hair) and the pack's contents.
fn get_items(w: &World, id: MobileId) -> Vec<ItemId> {
    let pack = pack_of(w, id);
    let mut list = Vec::new();
    if let Some(m) = w.mobile(id) {
        for &item in &m.items {
            let Some(it) = w.item(item) else { continue };
            if it.movable()
                && Some(item) != pack
                && it.layer != Layer::Hair
                && it.layer != Layer::FacialHair
            {
                list.push(item);
            }
        }
    }
    if let Some(p) = pack.and_then(|p| w.item(p)) {
        list.extend(p.items.iter().copied());
    }
    list
}

/// `Destroy(toBackpack)`.
pub fn destroy(w: &mut World, id: MobileId, to_backpack: bool) {
    (pclass(w, id).destroy)(w, id, to_backpack);
}

/// `PlayerVendor.Destroy(toBackpack)` itself, for the subclasses that call `base.Destroy`.
pub fn destroy_base(w: &mut World, id: MobileId, to_backpack: bool) {
    return_vendor(w, id);
    if !new_vendor_system(w) {
        fix_dresswear(w, id);
    }
    /* Possible cases regarding item return:
     *
     * 1. No item must be returned
     *       -> do nothing.
     * 2. ( toBackpack is false OR the vendor is in the internal map ) AND the vendor is associated with a AOS house
     *       -> put the items into the moving crate or a vendor inventory,
     *          depending on whether the vendor owner is also the house owner.
     * 3. ( toBackpack is true OR the vendor isn't associated with any AOS house ) AND the vendor isn't in the internal map
     *       -> put the items into a backpack.
     * 4. The vendor isn't associated with any house AND it's in the internal map
     *       -> do nothing (we can't do anything).
     */
    let list = get_items(w, id);
    let gold = hold_gold(w, id);
    if !list.is_empty() || gold > 0 {
        let map = mx::map(w, id);
        let house_id = house_of(w, id);
        let aos_house = house_id.filter(|&h| house::is_aos_rules(w, h));
        let internal = map == Some(MapId::INTERNAL);
        if (!to_backpack || internal)
            && let Some(h) = aos_house
        {
            let owner = owner(w, id);
            if owner.is_some_and(|o| house::access::is_owner(w, h, o)) {
                // Move to moving crate
                let existing = house::data(w, h).and_then(|d| d.moving_crate);
                let crate_id = match existing {
                    Some(c) => c,
                    None => {
                        let c = muo_multis::moving_crate::create(w, h);
                        if let Some(d) = house::data_mut(w, h) {
                            d.moving_crate = Some(c);
                        }
                        c
                    }
                };
                if gold > 0 {
                    banker::deposit_into(w, crate_id, gold);
                }
                for item in list {
                    muo_multis::moving_crate::drop_item(w, crate_id, item);
                }
            } else {
                // Move to vendor inventory
                let name = w
                    .mobile(id)
                    .and_then(|m| (m.ty.name)(w, id))
                    .unwrap_or_default();
                let inv =
                    muo_multis::vendor_inventory::create(w, h, owner, &name, &shop_name(w, id));
                if let Some(i) = house::data_mut(w, h)
                    .and_then(|d| d.vendor_inventories.iter_mut().find(|v| v.id == inv))
                {
                    i.gold = gold;
                }
                for item in list {
                    muo_multis::vendor_inventory::add_item(w, h, inv, item);
                }
            }
        } else if (to_backpack || aos_house.is_none()) && !internal {
            // Case 3 - Move to backpack
            if let Some(backpack) = factory::make_item(w, "Server.Items.Backpack") {
                if gold > 0 {
                    banker::deposit_into(w, backpack, gold);
                }
                for item in list {
                    container::drop_item_default(w, backpack, item);
                }
                ops::move_to_world_map(w, backpack, mx::location(w, id), map);
            }
        }
    }
    muo_server::mobile::base::delete(w, id);
}

/// `FixDresswear()`.
fn fix_dresswear(w: &mut World, id: MobileId) {
    let worn: Vec<ItemId> = w.mobile(id).map_or_else(Vec::new, |m| m.items.clone());
    for item in worn {
        let layer = [
            ("BaseHat", Layer::Helm),
            ("BaseMiddleTorso", Layer::MiddleTorso),
            ("BaseOuterLegs", Layer::OuterLegs),
            ("BaseOuterTorso", Layer::OuterTorso),
            ("BasePants", Layer::Pants),
            ("BaseShirt", Layer::Shirt),
            ("BaseWaist", Layer::Waist),
            ("BaseShoes", Layer::Shoes),
        ]
        .iter()
        .find(|(n, _)| factory::item_is(w, item, &format!("Server.Items.{n}")))
        .map(|(_, l)| *l);
        if let Some(l) = layer {
            muo_server::item::base::set_layer(w, item, l);
        }
        if factory::item_is(w, item, "Server.Items.Sandals") {
            muo_server::item::base::set_hue(w, item, 0);
        }
    }
}

/// `vendor.Backpack?.TotalItems`.
pub fn pack_item_count(w: &World, id: MobileId) -> i32 {
    pack_of(w, id).map_or(0, |p| count_items(w, p) - 1)
}

/// `Return()`.
pub fn return_vendor(w: &mut World, id: MobileId) {
    if let Some(p) = data(w, id).and_then(|d| d.placeholder) {
        muo_server::item::base::delete(w, p);
    }
}

fn on_after_delete(w: &mut World, id: MobileId) {
    (MOBILE_BASE.on_after_delete)(w, id);
    let mut t = data_mut(w, id)
        .map(|d| std::mem::take(&mut d.pay_timer))
        .unwrap_or_default();
    t.cancel(w);
    set_house(w, id, None);
    if let Some(p) = data(w, id).and_then(|d| d.placeholder) {
        muo_server::item::base::delete(w, p);
    }
}

/// `PayTimer.OnTick`.
fn pay_tick(w: &mut World, id: MobileId) {
    if mx::deleted(w, id) {
        return;
    }
    let next = w.now + pay_interval(w);
    if let Some(d) = data_mut(w, id) {
        d.next_pay_time = next;
    }
    let nvs = new_vendor_system(w);
    let (mut pay, total_gold) = if nvs {
        (charge_per_real_world_day(w, id), hold_gold(w, id))
    } else {
        (
            charge_per_day(w, id),
            bank_account(w, id) + hold_gold(w, id),
        )
    };
    if pay > total_gold {
        destroy(w, id, !nvs);
    } else {
        if !nvs {
            let bank = bank_account(w, id);
            if bank >= pay {
                set_bank_account(w, id, bank - pay);
                pay = 0;
            } else {
                pay -= bank;
                set_bank_account(w, id, 0);
            }
        }
        let hold = hold_gold(w, id);
        set_hold_gold(w, id, hold - pay);
    }
}

// ---- vendor items ----

fn set_vendor_item(w: &mut World, id: MobileId, item: ItemId, price: i32, description: &str) {
    let created = w.now;
    set_vendor_item_created(w, id, item, price, description, created);
}

fn set_vendor_item_created(
    w: &mut World,
    id: MobileId,
    item: ItemId,
    price: i32,
    description: &str,
    created: DateTime,
) {
    remove_vendor_item(w, id, item);
    let vi = VendorItem {
        item: Some(item),
        price,
        description: description.to_owned(),
        created,
        valid: true,
    };
    if let Some(d) = data_mut(w, id) {
        d.sell_items.insert(item, vi);
    }
    ops::invalidate_properties(w, item);
}

fn remove_vendor_item(w: &mut World, id: MobileId, item: ItemId) {
    let had = data_mut(w, id).and_then(|d| d.sell_items.shift_remove(&item));
    if had.is_some() {
        let subs: Vec<ItemId> = w.item(item).map_or_else(Vec::new, |i| i.items.clone());
        for s in subs {
            remove_vendor_item(w, id, s);
        }
        ops::invalidate_properties(w, item);
    }
}

fn can_be_vendor_item(w: &World, id: MobileId, item: ItemId) -> bool {
    let parent = w.item(item).and_then(|i| i.parent);
    let pack = pack_of(w, id);
    if let Some(EntityId::Item(p)) = parent {
        if Some(p) == pack {
            return true;
        }
        if ops::is_container(w, p)
            && let Some(vi) = get_vendor_item(w, id, p)
        {
            return !vi.is_for_sale();
        }
    }
    false
}

fn on_sub_item_added(w: &mut World, id: MobileId, item: ItemId) {
    (MOBILE_BASE.on_sub_item_added)(w, id, item);
    if get_vendor_item(w, id, item).is_none() && can_be_vendor_item(w, id, item) {
        set_vendor_item(w, id, item, 999, "");
    }
}

fn on_sub_item_removed(w: &mut World, id: MobileId, item: ItemId) {
    (MOBILE_BASE.on_sub_item_removed)(w, id, item);
    if w.item(item).is_some_and(|i| i.bounce.is_none()) {
        remove_vendor_item(w, id, item);
    }
}

fn on_sub_item_bounce_cleared(w: &mut World, id: MobileId, item: ItemId) {
    (MOBILE_BASE.on_sub_item_bounce_cleared)(w, id, item);
    if !can_be_vendor_item(w, id, item) {
        remove_vendor_item(w, id, item);
    }
}

fn on_item_removed(w: &mut World, id: MobileId, item: ItemId) {
    (MOBILE_BASE.on_item_removed)(w, id, item);
    if pack_of(w, id) == Some(item) {
        let subs: Vec<ItemId> = w.item(item).map_or_else(Vec::new, |i| i.items.clone());
        for s in subs {
            remove_vendor_item(w, id, s);
        }
    }
}

fn say_to(w: &mut World, id: MobileId, to: MobileId, number: i32) {
    speech::say_to_number(w, id, to, number, "");
}

fn on_drag_drop(w: &mut World, id: MobileId, from: MobileId, item: ItemId) -> bool {
    if !is_owner(w, id, from) {
        say_to(w, id, from, 503209); // I can only take item from the shop owner.
        return false;
    }
    if factory::item_is(w, item, "Server.Items.Gold") {
        let amount = w.item(item).map_or(0, |i| i.amount);
        if new_vendor_system(w) {
            if hold_gold(w, id) < 1_000_000 {
                say_to(w, id, from, 503210); // I'll take that to fund my services.
                let g = hold_gold(w, id);
                set_hold_gold(w, id, g.wrapping_add(amount));
                muo_server::item::base::delete(w, item);
                return true;
            }
            mx::msg(w, from, 1062493); // Your vendor has sufficient funds for operation and cannot accept this gold.
            return false;
        }
        if bank_account(w, id) < 1_000_000 {
            say_to(w, id, from, 503210);
            let g = bank_account(w, id);
            set_bank_account(w, id, g.wrapping_add(amount));
            muo_server::item::base::delete(w, item);
            return true;
        }
        mx::msg(w, from, 1062493);
        return false;
    }
    let new_item = get_vendor_item(w, id, item).is_none();
    let dropped =
        pack_of(w, id).is_some_and(|p| container::try_drop_item(w, p, Some(from), item, false));
    if dropped {
        if new_item {
            on_item_given(w, id, from, item);
        }
        return true;
    }
    say_to(w, id, from, 503211); // I can't carry any more.
    false
}

fn check_nonlocal_drop(
    w: &mut World,
    id: MobileId,
    from: MobileId,
    item: ItemId,
    _target: ItemId,
) -> bool {
    if is_owner(w, id, from) {
        if get_vendor_item(w, id, item).is_none() {
            timer::start(w, TimeSpan::ZERO, TimeSpan::ZERO, 1, move |w| {
                on_item_given(w, id, from, item)
            });
        }
        return true;
    }
    say_to(w, id, from, 503209); // I can only take item from the shop owner.
    false
}

/// `OnItemGiven(from, item)`.
fn on_item_given(w: &mut World, id: MobileId, from: MobileId, item: ItemId) {
    if get_vendor_item(w, id, item).is_none() {
        return;
    }
    let name = w
        .item(item)
        .and_then(|i| i.name.clone())
        .filter(|n| !n.is_empty())
        .unwrap_or_else(|| format!("#{}", muo_server::item::base::label_number(w, item)));
    muo_server::mobile::part2::messages::send_localized_message(w, from, 1043303, &name, 0x3B2); // Type in a price and description for ~1_ITEM~ (ESC=not for sale)
    let on_response = Rc::new(move |w: &mut World, from: MobileId, text: &str| {
        price_response(w, id, item, from, Some(text))
    });
    let on_cancel =
        Rc::new(move |w: &mut World, from: MobileId| price_response(w, id, item, from, None));
    let p = add_prompt(
        w,
        Prompt {
            on_response: Some(on_response),
            on_cancel: Some(on_cancel),
        },
    );
    props_b::set_prompt(w, from, Some(p));
}

/// `VendorPricePrompt.OnResponse` / `OnCancel`.
fn price_response(w: &mut World, id: MobileId, item: ItemId, from: MobileId, text: Option<&str>) {
    let valid = get_vendor_item(w, id, item).is_some_and(|v| v.valid);
    if !valid || !can_interact_with(w, id, from, true) {
        return;
    }
    let (price, description) = match text {
        None => (-1, String::new()),
        Some(text) => {
            let sep = text.find([' ', ',']);
            let first = sep.map_or(text, |i| &text[..i]);
            match first.parse::<i32>() {
                Ok(price) => {
                    let d = sep.map_or("", |i| text[i + 1..].trim());
                    (price, muo_primitives::utility::fix_html(d))
                }
                Err(_) => (-1, muo_primitives::utility::fix_html(text.trim())),
            }
        }
    };
    set_info(w, id, item, from, price, &description);
}

/// `SetInfo(from, price, description)`.
fn set_info(
    w: &mut World,
    id: MobileId,
    item: ItemId,
    from: MobileId,
    price: i32,
    description: &str,
) {
    let mut price = price;
    let mut set_price = false;
    if price < 0 {
        // Not for sale
        price = -1;
        if ops::is_container(w, item) {
            let locked = factory::item_is(w, item, "Server.Items.LockableContainer")
                && muo_content::items::cont::locked(w, item);
            if locked {
                say_to(w, id, from, 1043298); // Locked items may not be made not-for-sale.
            } else if w.item(item).is_some_and(|i| !i.items.is_empty()) {
                say_to(w, id, from, 1043299); // To be not for sale, all items in a container must be for sale.
            } else {
                set_price = true;
            }
        } else if factory::item_is(w, item, "Server.Items.BaseBook")
            || factory::item_is(w, item, "Server.Engines.BulkOrders.BulkOrderBook")
        {
            set_price = true;
        } else {
            // Only the following may be made not-for-sale: books, containers, keyrings, and items in for-sale containers.
            say_to(w, id, from, 1043301);
        }
    } else {
        if price > 100_000_000 {
            price = 100_000_000;
            mx::send_message(
                w,
                from,
                "You cannot price items above 100,000,000 gold.  The price has been adjusted.",
            );
        }
        set_price = true;
    }
    if set_price {
        set_vendor_item(w, id, item, price, description);
    } else if let Some(v) = data_mut(w, id).and_then(|d| d.sell_items.get_mut(&item)) {
        v.description = description.to_owned();
        let valid = v.valid;
        if valid {
            ops::invalidate_properties(w, item);
        }
    }
}

fn allow_equip_from(w: &World, id: MobileId, from: MobileId) -> bool {
    (new_vendor_system(w) && is_owner(w, id, from)) || (MOBILE_BASE.allow_equip_from)(w, id, from)
}

fn check_nonlocal_lift(w: &mut World, id: MobileId, from: MobileId, item: ItemId) -> bool {
    let pack = pack_of(w, id);
    if pack.is_some_and(|p| ops::is_child_of(w, item, Some(EntityId::Item(p)))) {
        if is_owner(w, id, from) {
            return true;
        }
        say_to(w, id, from, 503223); // If you'd like to purchase an item, just ask.
        return false;
    }
    if new_vendor_system(w) && is_owner(w, id, from) {
        return true;
    }
    (MOBILE_BASE.check_nonlocal_lift)(w, id, from, item)
}

/// `CanInteractWith(from, ownerOnly)`.
pub fn can_interact_with(w: &mut World, id: MobileId, from: MobileId, owner_only: bool) -> bool {
    let ty = w.mobile(from).map(|m| m.ty);
    let sees = ty.is_some_and(|t| (t.can_see_mobile)(w, from, id));
    if !sees
        || !muo_server::mobile::part1::combat::in_update_range(
            mx::location(w, from),
            mx::location(w, id),
        )
        || !muo_server::mobile::part2::stats::check_alive(w, from, true)
    {
        return false;
    }
    if owner_only {
        return is_owner(w, id, from);
    }
    if let Some(h) = house_of(w, id)
        && house::access::is_banned(w, h, from)
        && !is_owner(w, id, from)
    {
        mx::msg(w, from, 1062674); // You can't shop from this home as you have been banned from this establishment.
        return false;
    }
    true
}

fn on_double_click(w: &mut World, id: MobileId, from: MobileId) {
    if is_owner(w, id, from) {
        send_owner_gump(w, id, from);
    } else if can_interact_with(w, id, from, false) {
        open_backpack(w, id, from);
    }
}

fn display_paperdoll_to(w: &mut World, id: MobileId, m: MobileId) {
    if new_vendor_system(w) {
        (MOBILE_BASE.display_paperdoll_to)(w, id, m);
    } else if can_interact_with(w, id, m, false) {
        open_backpack(w, id, m);
    }
}

/// `SendOwnerGump(to)`.
pub fn send_owner_gump(w: &mut World, id: MobileId, to: MobileId) {
    use muo_gumps::m_z::player_vendor as g;
    if let Some(ns) = w.mobile(to).and_then(|m| m.net_state) {
        if new_vendor_system(w) {
            muo_content::gumps::system::close_gump::<g::NewPlayerVendorCustomizeGump>(w, ns);
            muo_content::gumps::system::close_gump::<g::NewPlayerVendorOwnerGump>(w, ns);
            g::new_owner_display_to(w, to, id);
        } else {
            muo_content::gumps::system::close_gump::<g::PlayerVendorCustomizeGump>(w, ns);
            muo_content::gumps::system::close_gump::<g::PlayerVendorOwnerGump>(w, ns);
            g::owner_display_to(w, to, id);
        }
    }
}

/// `OpenBackpack(from)`.
pub fn open_backpack(w: &mut World, id: MobileId, from: MobileId) {
    if let Some(pack) = pack_of(w, id) {
        let n = if is_owner(w, id, from) {
            1010642
        } else {
            503208
        }; // Take a look at my/your goods.
        say_to(w, id, from, n);
        if let Some(t) = w.item(pack).map(|i| i.ty) {
            (t.display_to)(w, pack, from);
        }
    }
}

/// `PlayerVendor.TryToBuy(item, from)`.
pub fn try_to_buy(w: &mut World, item: ItemId, from: MobileId) {
    let Some(EntityId::Mobile(vendor)) = ops::root_parent(w, item) else {
        return;
    };
    if !is_player_vendor(w, vendor) || !can_interact_with(w, vendor, from, false) {
        return;
    }
    let enabled = pv_hooks(w)
        .and_then(|h| h.feature_enabled)
        .is_none_or(|f| f(w));
    if !enabled && mx::access_level(w, from) < AccessLevel::Administrator {
        muo_server::mobile::part2::messages::send_message_hue(
            w,
            from,
            0x22,
            "Player vendor transactions are temporarily disabled.",
        );
        return;
    }
    if is_owner(w, vendor, from) {
        say_to(w, vendor, from, 503212); // You own this shop, just take what you want.
        return;
    }
    let vi = get_vendor_item(w, vendor, item).cloned();
    match vi {
        None => say_to(w, vendor, from, 503216), // You can't buy that.
        Some(vi) if !vi.is_for_sale() => say_to(w, vendor, from, 503202), // This item is not for sale.
        Some(vi) if vi.created + TimeSpan::from_mins(1) > w.now => {
            mx::send_message(
                w,
                from,
                "You cannot buy this item right now.  Please wait one minute and try again.",
            );
        }
        Some(vi) => {
            let view = muo_gumps::m_z::ops::VendorItemView {
                item,
                price: vi.price,
                description: vi.description,
                valid: vi.valid,
            };
            muo_gumps::m_z::player_vendor::buy_display_to(w, from, vendor, view);
        }
    }
}

/// `CollectGold(to)`.
pub fn collect_gold(w: &mut World, id: MobileId, to: MobileId) {
    let gold = hold_gold(w, id);
    if gold > 0 {
        speech::say_to(
            w,
            id,
            to,
            &format!("How much of the {gold} that I'm holding would you like?"),
        );
        mx::send_message(
            w,
            to,
            "Enter the amount of gold you wish to withdraw (ESC = CANCEL):",
        );
        let p = Prompt {
            on_response: Some(Rc::new(move |w, from, text| {
                if !can_interact_with(w, id, from, true) {
                    return;
                }
                let amount = text.trim().parse::<i32>().unwrap_or(0);
                collect_gold_amount(w, id, from, amount);
            })),
            on_cancel: Some(Rc::new(move |w, from| {
                if can_interact_with(w, id, from, true) {
                    collect_gold_amount(w, id, from, 0);
                }
            })),
        };
        let pid = add_prompt(w, p);
        props_b::set_prompt(w, to, Some(pid));
    } else {
        say_to(w, id, to, 503215); // I am holding no gold for you.
    }
}

fn collect_gold_amount(w: &mut World, id: MobileId, to: MobileId, amount: i32) {
    if amount <= 0 {
        speech::say_to(
            w,
            id,
            to,
            "Very well. I will hold on to the money for now then.",
        );
    } else {
        give_gold(w, id, to, amount);
    }
}

/// `GiveGold(to, amount)`.
pub fn give_gold(w: &mut World, id: MobileId, to: MobileId, amount: i32) -> i32 {
    if amount <= 0 {
        return 0;
    }
    let hold = hold_gold(w, id);
    if amount > hold {
        speech::say_to(
            w,
            id,
            to,
            &format!("I'm sorry, but I'm only holding {hold} gold for you."),
        );
        return 0;
    }
    let given = banker::deposit_up_to(w, to, amount, true, 0);
    set_hold_gold(w, id, hold - given);
    if given > 0 {
        muo_server::mobile::part2::messages::send_localized_message(
            w,
            to,
            1060397,
            &given.to_string(),
            0,
        ); // ~1_AMOUNT~ gold has been deposited into your bank box.
    }
    if given == 0 {
        say_to(w, id, to, 1070755); // Your bank box cannot hold the gold you are requesting.  I will keep the gold until you can take it.
    } else if amount > given {
        say_to(w, id, to, 1070756); // I can only give you part of the gold now, as your bank box is too full to hold the full amount.
    } else if hold_gold(w, id) > 0 {
        say_to(w, id, to, 1042639); // Your gold has been transferred.
    } else {
        say_to(w, id, to, 503234); // All the gold I have been carrying for you has been deposited into your bank account.
    }
    given
}

/// `Dismiss(from)`.
pub fn dismiss(w: &mut World, id: MobileId, from: MobileId) {
    if pack_of(w, id).is_some_and(|p| w.item(p).is_some_and(|i| !i.items.is_empty())) {
        say_to(w, id, from, 1038325); // You cannot dismiss me while I am holding your goods.
        return;
    }
    let hold = hold_gold(w, id);
    if hold > 0 {
        give_gold(w, id, from, hold);
        if hold_gold(w, id) > 0 {
            return;
        }
    }
    destroy(w, id, true);
}

/// `NameVerification.ValidateVendorName(name)`.
fn validate_vendor_name(_w: &World, name: &str) -> bool {
    muo_content::misc::name_verification::validate_vendor_name(name)
}

/// `Rename(from)`.
pub fn rename(w: &mut World, id: MobileId, from: MobileId) {
    mx::msg(w, from, 1062494); // Enter a new name for your vendor (20 characters max):
    let p = Prompt {
        on_response: Some(Rc::new(move |w, from, text| {
            if !can_interact_with(w, id, from, true) {
                return;
            }
            let name = text.trim();
            if !validate_vendor_name(w, name) {
                speech::say_to(w, id, from, "That name is unacceptable.");
                return;
            }
            let n = muo_primitives::utility::fix_html(name);
            (w.mobile(id).map_or(MOBILE_BASE.set_name, |m| m.ty.set_name))(w, id, Some(n));
            mx::msg(w, from, 1062496); // Your vendor has been renamed.
            muo_gumps::m_z::player_vendor::new_owner_display_to(w, from, id);
        })),
        on_cancel: None,
    };
    let pid = add_prompt(w, p);
    props_b::set_prompt(w, from, Some(pid));
}

/// `RenameShop(from)`.
pub fn rename_shop(w: &mut World, id: MobileId, from: MobileId) {
    mx::msg(w, from, 1062433); // Enter a new name for your shop (20 chars max):
    let p = Prompt {
        on_response: Some(Rc::new(move |w, from, text| {
            if !can_interact_with(w, id, from, true) {
                return;
            }
            let name = text.trim();
            if !validate_vendor_name(w, name) {
                speech::say_to(w, id, from, "That name is unacceptable.");
                return;
            }
            set_shop_name(w, id, Some(muo_primitives::utility::fix_html(name)));
            muo_gumps::m_z::player_vendor::new_owner_display_to(w, from, id);
        })),
        on_cancel: None,
    };
    let pid = add_prompt(w, p);
    props_b::set_prompt(w, from, Some(pid));
}

/// `CheckTeleport(to)`.
pub fn check_teleport(w: &mut World, id: MobileId, to: MobileId) -> bool {
    let Some(h) = house_of(w, id) else {
        return false;
    };
    if mx::deleted(w, id) || !is_owner(w, id, to) || mx::map(w, id) == Some(MapId::INTERNAL) {
        return false;
    }
    if house::access::is_inside_mobile(w, h, to)
        || mx::map(w, to) != house::map_of(w, h)
        || !house_in_range(w, h, mx::location(w, to), 5)
    {
        return false;
    }
    let placeholder = data(w, id).and_then(|d| d.placeholder);
    match placeholder {
        None => {
            let p = placeholder_new(w, id);
            if let Some(d) = data_mut(w, id) {
                d.placeholder = Some(p);
            }
            ops::move_to_world_map(w, p, mx::location(w, id), mx::map(w, id));
            if let Some(map) = mx::map(w, to) {
                muo_server::mobile::base::move_to_world(w, id, mx::location(w, to), map);
            }
            // This vendor has been moved out of the house to your current location temporarily.  The vendor will return home automatically after two minutes have passed once you are done managing its inventory or customizing it.
            mx::msg(w, to, 1062431);
        }
        Some(p) => {
            restart_placeholder_timer(w, p);
            // This vendor is currently temporarily in a location outside its house.  The vendor will return home automatically after two minutes have passed once you are done managing its inventory or customizing it.
            mx::msg(w, to, 1062430);
        }
    }
    true
}

fn get_properties(w: &World, id: MobileId, list: &mut PropertyList) {
    (MOBILE_BASE.get_properties)(w, id, list);
    if new_vendor_system(w) {
        list.add_str(1062449, &shop_name(w, id)); // Shop Name: ~1_NAME~
    }
}

fn get_context_menu_entries(
    w: &mut World,
    id: MobileId,
    from: MobileId,
    list: &mut Vec<ContextMenuEntry>,
) {
    if mx::alive(w, from)
        && data(w, id).is_some_and(|d| d.placeholder.is_some())
        && is_owner(w, id, from)
    {
        let mut e = ContextMenuEntry::new(6214, -1);
        e.on_click = Some(Rc::new(|w, from, target| {
            if let EntityId::Mobile(v) = target
                && muo_server::mobile::part2::stats::check_alive(w, from, true)
                && !mx::deleted(w, v)
                && is_player_vendor(w, v)
                && is_owner(w, v, from)
            {
                return_vendor(w, v);
            }
        }));
        list.push(e);
    }
    (MOBILE_BASE.get_context_menu_entries)(w, id, from, list);
}

fn handles_on_speech(w: &World, id: MobileId, from: MobileId) -> bool {
    let (a, b) = (mx::location(w, from), mx::location(w, id));
    let d = f64::from((a.x - b.x) * (a.x - b.x) + (a.y - b.y) * (a.y - b.y)).sqrt();
    mx::alive(w, from) && d <= 3.0
}

fn was_named(w: &World, id: MobileId, speech: &str) -> bool {
    w.mobile(id)
        .and_then(|m| (m.ty.name)(w, id))
        .as_ref()
        .is_some_and(|n| speech.to_lowercase().starts_with(&n.to_lowercase()))
}

fn on_speech(w: &mut World, id: MobileId, e: &mut muo_server::events::SpeechEventArgs) {
    let from = e.mobile;
    let (a, b) = (mx::location(w, from), mx::location(w, id));
    let dist = f64::from((a.x - b.x) * (a.x - b.x) + (a.y - b.y) * (a.y - b.y)).sqrt();
    if e.handled || !mx::alive(w, from) || dist > 3.0 {
        return;
    }
    let named = was_named(w, id, &e.speech);
    let kw = |plain: i32, named_kw: i32| e.has_keyword(plain) || (e.has_keyword(named_kw) && named);
    let banned = house_of(w, id).is_some_and(|h| house::access::is_banned(w, h, from));
    if kw(0x3C, 0x171) {
        // vendor buy, *buy*
        if is_owner(w, id, from) {
            say_to(w, id, from, 503212); // You own this shop, just take what you want.
        } else if !banned {
            mx::msg(w, from, 503213); // Select the item you wish to buy.
            let mut t =
                target::Target::simple(w, 3, TargetFlags::NONE, false, |w, from, targeted| {
                    if let Targeted::Entity(EntityId::Item(item)) = targeted {
                        try_to_buy(w, item, from);
                    }
                });
            t.allow_nonlocal = true;
            t.player_vendor_target = true;
            let tid = target::add_target(w, t);
            target::set_mobile_target(w, from, Some(tid));
            e.handled = true;
        }
    } else if kw(0x3D, 0x172) {
        // vendor browse, *browse
        if banned && !is_owner(w, id, from) {
            say_to(w, id, from, 1062674); // You can't shop from this home as you have been banned from this establishment.
        } else {
            if named {
                open_backpack(w, id, from);
            } else {
                for m in muo_content::mx::mobiles_in_range(w, from, 2) {
                    if is_player_vendor(w, m) {
                        let sees = w
                            .mobile(m)
                            .map(|x| x.ty)
                            .is_some_and(|t| (t.can_see_mobile)(w, m, from));
                        if sees && mx::in_los(w, m, from) {
                            open_backpack(w, m, from);
                        }
                    }
                }
            }
            e.handled = true;
        }
    } else if kw(0x3E, 0x173) {
        // vendor collect, *collect
        if is_owner(w, id, from) {
            collect_gold(w, id, from);
            e.handled = true;
        }
    } else if kw(0x3F, 0x174) {
        // vendor status, *status
        if is_owner(w, id, from) {
            send_owner_gump(w, id, from);
            e.handled = true;
        } else {
            say_to(w, id, from, 503226); // What do you care? You don't run this shop.
        }
    } else if kw(0x40, 0x175) {
        // vendor dismiss, *dismiss
        if is_owner(w, id, from) {
            dismiss(w, id, from);
            e.handled = true;
        }
    } else if kw(0x41, 0x176) {
        // vendor cycle, *cycle
        if is_owner(w, id, from) {
            let d = mx::get_direction_to(w, id, from);
            mx::set_direction(w, id, d);
            e.handled = true;
        }
    }
}

// ---- serialization ----

fn serialize(w: &World, id: MobileId, out: &mut dyn GenericWriter) {
    (MOBILE_BASE.serialize)(w, id, out);
    out.write_i32(4);
    let Some(d) = data(w, id) else { return };
    out.write_opt_string(d.shop_name.as_deref());
    out.write_anchored_time_ticks(d.next_pay_time.ticks());
    out.write_item(w, d.house);
    out.write_mobile(w, d.owner);
    out.write_i32(d.bank_account);
    out.write_i32(d.hold_gold);
    out.write_encoded_int(d.sell_items.len() as i32);
    for (&item, vi) in &d.sell_items {
        out.write_item(w, Some(item));
        out.write_i32(0); // VendorItem's version
        out.write_item(w, vi.item);
        out.write_i32(vi.price);
        out.write_opt_string(Some(&vi.description));
        out.write_date_time_ticks(vi.created.ticks());
    }
}

/// `VendorItem.Deserialize(reader)`.
fn read_vendor_item(w: &World, r: &mut dyn GenericReader) -> muo_io::Result<VendorItem> {
    let _version = r.read_i32()?;
    let item = r.read_item(w)?;
    let mut price = r.read_i32()?;
    let description = r.read_string()?.unwrap_or_default();
    let created = DateTime::from_ticks(r.read_date_time_ticks()?);
    // [AfterDeserialization]
    if price > 100_000_000 {
        price = 100_000_000;
    }
    Ok(VendorItem {
        item,
        price,
        description,
        created,
        valid: true,
    })
}

fn deserialize(w: &mut World, id: MobileId, r: &mut dyn GenericReader) -> muo_io::Result<()> {
    (MOBILE_BASE.deserialize)(w, id, r)?;
    let version = r.read_i32()?;
    let mut d = PlayerVendorData::default();
    let house_ref;
    if version >= 4 {
        d.shop_name = r.read_string()?;
        d.next_pay_time = DateTime::from_ticks(r.read_anchored_time_ticks()?);
        house_ref = r.read_item(w)?;
        d.owner = r.read_mobile(w)?;
        d.bank_account = r.read_i32()?;
        d.hold_gold = r.read_i32()?;
        let count = r.read_encoded_int()?;
        for _ in 0..count {
            let key = r.read_item(w)?;
            let vi = read_vendor_item(w, r)?;
            if let Some(k) = key {
                d.sell_items.insert(k, vi);
            }
        }
    } else if version == 3 {
        d.shop_name = r.read_string()?;
        d.next_pay_time = DateTime::from_ticks(r.read_delta_time_ticks(w.now.ticks())?);
        house_ref = r.read_item(w)?;
        d.owner = r.read_mobile(w)?;
        d.bank_account = r.read_i32()?;
        d.hold_gold = r.read_i32()?;
        let count = r.read_encoded_int()?;
        for _ in 0..count {
            let key = r.read_item(w)?;
            let vi = read_vendor_item(w, r)?;
            if let Some(k) = key {
                d.sell_items.insert(k, vi);
            }
        }
    } else {
        r.read_bool()?; // New vendor system?
        d.shop_name = r.read_string()?;
        d.next_pay_time = DateTime::from_ticks(r.read_delta_time_ticks(w.now.ticks())?);
        house_ref = r.read_item(w)?;
        d.owner = r.read_mobile(w)?;
        d.bank_account = r.read_i32()?;
        d.hold_gold = r.read_i32()?;
        let count = r.read_i32()?;
        for _ in 0..count {
            let key = r.read_item(w)?;
            let vi = read_vendor_item(w, r)?;
            if let Some(k) = key {
                d.sell_items.insert(k, vi);
            }
        }
    }
    let next_pay = d.next_pay_time;
    if let Some(m) = w.mobile_mut(id) {
        m.ext.insert(d);
    }
    set_house(w, id, house_ref);
    after_deserialization(w, id, next_pay);
    Ok(())
}

/// `[AfterDeserialization]`.
fn after_deserialization(w: &mut World, id: MobileId, next_pay: DateTime) {
    let delay = next_pay - w.now;
    start_pay_timer(
        w,
        id,
        if delay > TimeSpan::ZERO {
            delay
        } else {
            TimeSpan::ZERO
        },
    );
    props_a::set_blessed(w, id, false);
    if w.core_aos() && w.mobile(id).is_some_and(|m| m.name_hue == 0x35) {
        set_name_hue(w, id, -1);
    }
    // Do we have a vendor that may have been orphaned? Let's try to recover them and attach them to their house.
    if house_of(w, id).is_none() {
        let owner = owner(w, id);
        if owner.is_none() {
            timer::start(w, TimeSpan::ZERO, TimeSpan::ZERO, 1, move |w| {
                muo_server::mobile::base::delete(w, id);
            }); // Don't try to dismiss, no owner.
        }
        timer::start(w, TimeSpan::ZERO, TimeSpan::ZERO, 1, move |w| {
            if mx::deleted(w, id) {
                return;
            }
            if let Some(h) = house::access::find_house_at_mobile(w, id) {
                set_house(w, id, Some(h));
            } else if let Some(o) = owner
                && mx::access_level(w, o) == AccessLevel::Player
            {
                // If we can't find a house, dismiss the vendor.
                dismiss(w, id, o);
            }
        });
    }
}

pub static PLAYER_VENDOR: MobileType = MobileType {
    full_name: "Server.Mobiles.PlayerVendor",
    parent: Some(&MOBILE_BASE),
    construct: None,
    family: Some(&BASE_PV_CLASS),
    serialize,
    deserialize,
    on_after_delete,
    on_sub_item_added,
    on_sub_item_removed,
    on_sub_item_bounce_cleared,
    on_item_removed,
    on_drag_drop,
    check_nonlocal_drop,
    allow_equip_from,
    check_nonlocal_lift,
    on_double_click,
    display_paperdoll_to,
    get_properties,
    get_context_menu_entries,
    is_snoop: |_, _, _| false,
    handles_on_speech,
    on_speech,
    can_be_damaged: |_, _| false,
    ..MOBILE_BASE
};

// ---- the constructor table ----

fn ctor_player_vendor(w: &mut World, v: &[muo_content::props::Value]) -> EntityId {
    let owner = match v.first() {
        Some(muo_content::props::Value::Entity(Some(EntityId::Mobile(m)))) => Some(*m),
        _ => None,
    };
    let house = match v.get(1) {
        Some(muo_content::props::Value::Entity(Some(EntityId::Item(i)))) => Some(*i),
        _ => None,
    };
    EntityId::Mobile(make(w, &PLAYER_VENDOR, owner, house))
}

fn configure(w: &mut World) {
    use muo_content::props::{Ctor, Kind, Param};
    use muo_content::utilities::types::PropType;
    static CTORS: &[Ctor] = &[Ctor {
        access: None,
        params: &[
            Param {
                name: "owner",
                kind: Kind::Plain(PropType::Entity),
                default: None,
                ty: "Mobile",
            },
            Param {
                name: "house",
                kind: Kind::Plain(PropType::Entity),
                default: None,
                ty: "BaseHouse",
            },
        ],
        make: ctor_player_vendor,
    }];
    muo_content::props::add_ctors(w, "Server.Mobiles.PlayerVendor", CTORS);
}

// ---- VendorBackpack ----

fn vendor_backpack_new(w: &mut World) -> ItemId {
    let id = muo_content::items::cont::new_base_container(w, &VENDOR_BACKPACK, 0xE75);
    muo_server::item::base::set_layer(w, id, Layer::Backpack);
    id
}

fn pack_vendor(w: &World, pack: ItemId) -> Option<MobileId> {
    match ops::root_parent(w, pack) {
        Some(EntityId::Mobile(m)) if is_player_vendor(w, m) => Some(m),
        _ => None,
    }
}

#[allow(clippy::too_many_arguments)]
fn pack_check_hold(
    w: &mut World,
    id: ItemId,
    m: Option<MobileId>,
    item: ItemId,
    message: bool,
    check_items: bool,
    plus_items: i32,
    plus_weight: i32,
) -> bool {
    if !(BACKPACK.check_hold)(
        w,
        id,
        m,
        item,
        message,
        check_items,
        plus_items,
        plus_weight,
    ) {
        return false;
    }
    if pv_hooks(w)
        .and_then(|h| h.is_imbued)
        .is_some_and(|f| f(w, item))
    {
        if message && let Some(m) = m {
            mx::send_message(w, m, "Imbued items may not be sold here.");
        }
        return false;
    }
    if !new_vendor_system(w)
        && let Some(EntityId::Mobile(vendor)) = w.item(id).and_then(|i| i.parent)
        && let Some(h) = house_of(w, vendor)
        && house::is_aos_rules(w, h)
    {
        let total = count_items(w, item) + plus_items;
        if !house::storage::check_aos_storage(w, h, total) {
            if message && let Some(m) = m {
                mx::msg(w, m, 1061839); // This action would exceed the secure storage limit of the house.
            }
            return false;
        }
    }
    true
}

fn pack_check_item_use(w: &mut World, id: ItemId, from: MobileId, item: ItemId) -> bool {
    if !(BACKPACK.check_item_use)(w, id, from, item) {
        return false;
    }
    if ops::is_container(w, item)
        || factory::item_is(w, item, "Server.Engines.BulkOrders.BulkOrderBook")
    {
        return true;
    }
    mx::msg(w, from, 500447); // That is not accessible.
    false
}

fn pack_check_target(
    w: &mut World,
    id: ItemId,
    from: MobileId,
    target: target::TargetId,
    targeted: Targeted,
) -> bool {
    (BACKPACK.check_target)(w, id, from, target, targeted)
        && (mx::access_level(w, from) >= AccessLevel::GameMaster
            || w.targets
                .get(&target)
                .is_some_and(|t| t.player_vendor_target))
}

fn pack_child_context_menu(
    w: &mut World,
    id: ItemId,
    from: MobileId,
    list: &mut Vec<ContextMenuEntry>,
    item: ItemId,
) {
    (BACKPACK.get_child_context_menu_entries)(w, id, from, list, item);
    let Some(pv) = pack_vendor(w, id) else { return };
    if is_owner(w, pv, from) {
        return;
    }
    if get_vendor_item(w, pv, item).is_some() {
        let mut e = ContextMenuEntry::new(6103, -1);
        e.non_local_use = true;
        e.on_click = Some(Rc::new(|w, from, target| {
            if let EntityId::Item(i) = target
                && w.item(i).is_some_and(|x| !x.deleted())
            {
                try_to_buy(w, i, from);
            }
        }));
        list.push(e);
    }
}

fn price_line(w: &World, vi: &VendorItem, list: &mut PropertyList) {
    if !vi.is_for_sale() {
        list.add(1043307); // Price: Not for sale.
    } else if vi.is_for_free() {
        list.add(1043306); // Price: FREE!
    } else {
        list.add_str(1043304, &vi.formatted_price(w.core_ml())); // Price: ~1_COST~
    }
}

fn pack_child_name_properties(w: &World, id: ItemId, list: &mut PropertyList, item: ItemId) {
    (BACKPACK.get_child_name_properties)(w, id, list, item);
    if let Some(pv) = pack_vendor(w, id)
        && let Some(vi) = get_vendor_item(w, pv, item)
    {
        price_line(w, vi, list);
    }
}

fn pack_child_properties(w: &World, id: ItemId, list: &mut PropertyList, item: ItemId) {
    (BACKPACK.get_child_properties)(w, id, list, item);
    if let Some(pv) = pack_vendor(w, id)
        && let Some(vi) = get_vendor_item(w, pv, item)
        && !vi.description.is_empty()
    {
        list.add_str(1043305, &vi.description); // <br>Seller's Description:<br>"~1_DESC~"
    }
}

fn pack_single_click_contained(w: &mut World, id: ItemId, from: MobileId, item: ItemId) {
    if let Some(pv) = pack_vendor(w, id)
        && let Some(vi) = get_vendor_item(w, pv, item).cloned()
    {
        if !vi.is_for_sale() {
            ops::label_to_number(w, item, from, 1043307, ""); // Price: Not for sale.
        } else if vi.is_for_free() {
            ops::label_to_number(w, item, from, 1043306, ""); // Price: FREE!
        } else {
            ops::label_to_number(w, item, from, 1043304, &vi.formatted_price(w.core_ml())); // Price: ~1_COST~
        }
        if !vi.description.is_empty() {
            ops::label_to(w, item, from, &format!("Description: {}", vi.description));
        }
    }
    (BACKPACK.on_single_click_contained)(w, id, from, item);
}

pub static VENDOR_BACKPACK: ItemType = ItemType {
    full_name: "Server.Mobiles.VendorBackpack",
    parent: Some(&BACKPACK),
    construct: None,
    make: Some(vendor_backpack_new),
    serialize: |w, id, out| {
        (BACKPACK.serialize)(w, id, out);
        out.write_i32(0);
    },
    deserialize: |w, id, r| {
        (BACKPACK.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        Ok(())
    },
    default_weight: |_, _| 1.0,
    default_max_weight: |_, _| 0,
    check_hold: pack_check_hold,
    is_accessible_to: |_, _, _| true,
    check_item_use: pack_check_item_use,
    check_target: pack_check_target,
    get_child_context_menu_entries: pack_child_context_menu,
    get_child_name_properties: pack_child_name_properties,
    get_child_properties: pack_child_properties,
    on_single_click_contained: pack_single_click_contained,
    ..BACKPACK
};

// ---- PlayerVendorPlaceholder ----

/// What the placeholder adds to an item.
#[derive(Default)]
pub struct PlaceholderData {
    pub vendor: Option<MobileId>,
    timer: TimerHandle,
}

fn placeholder_data(w: &World, id: ItemId) -> Option<&PlaceholderData> {
    w.item(id)?.ext.get::<PlaceholderData>()
}

/// `new PlayerVendorPlaceholder(vendor)`.
pub fn placeholder_new(w: &mut World, vendor: MobileId) -> ItemId {
    let id = ops::new_item_with_id(w, &PLAYER_VENDOR_PLACEHOLDER, 0x1F28);
    muo_server::item::base::set_hue(w, id, 0x672);
    ops::set_movable(w, id, false);
    let t = timer::start(w, TimeSpan::from_mins(2), TimeSpan::ZERO, 1, move |w| {
        muo_server::item::base::delete(w, id)
    });
    if let Some(i) = w.item_mut(id) {
        i.ext = muo_server::Ext::new(PlaceholderData {
            vendor: Some(vendor),
            timer: t,
        });
    }
    id
}

/// `RestartTimer()`.
pub fn restart_placeholder_timer(w: &mut World, id: ItemId) {
    let mut t = placeholder_data(w, id).map(|d| d.timer).unwrap_or_default();
    t.cancel(w);
    let t = timer::start(w, TimeSpan::from_mins(2), TimeSpan::ZERO, 1, move |w| {
        muo_server::item::base::delete(w, id)
    });
    if let Some(d) = w
        .item_mut(id)
        .and_then(|i| i.ext.get_mut::<PlaceholderData>())
    {
        d.timer = t;
    }
}

fn placeholder_get_properties(w: &World, id: ItemId, list: &mut PropertyList) {
    (ITEM_BASE.get_properties)(w, id, list);
    if let Some(v) = placeholder_data(w, id).and_then(|d| d.vendor) {
        let name = w
            .mobile(v)
            .and_then(|m| (m.ty.name)(w, v))
            .unwrap_or_default();
        list.add_str(1062498, &name); // reserved for vendor ~1_NAME~
    }
}

fn placeholder_on_delete(w: &mut World, id: ItemId) {
    if let Some(v) = placeholder_data(w, id).and_then(|d| d.vendor)
        && !mx::deleted(w, v)
    {
        let (loc, map) = (
            ops::get_world_location(w, id),
            w.item(id).and_then(|i| i.map),
        );
        if let Some(map) = map {
            muo_server::mobile::base::move_to_world(w, v, loc, map);
        }
        if let Some(d) = data_mut(w, v) {
            d.placeholder = None;
        }
    }
    (ITEM_BASE.on_delete)(w, id);
}

pub static PLAYER_VENDOR_PLACEHOLDER: ItemType = ItemType {
    full_name: "Server.Mobiles.PlayerVendorPlaceholder",
    parent: Some(&ITEM_BASE),
    construct: None,
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        out.write_encoded_int(0);
        out.write_mobile(w, placeholder_data(w, id).and_then(|d| d.vendor));
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let _version = r.read_encoded_int()?;
        let vendor = r.read_mobile(w)?;
        if let Some(i) = w.item_mut(id) {
            i.ext = muo_server::Ext::new(PlaceholderData {
                vendor,
                timer: TimerHandle::default(),
            });
        }
        // [AfterDeserialization(false)]
        muo_server::item::base::delete(w, id);
        Ok(())
    },
    get_properties: placeholder_get_properties,
    on_delete: placeholder_on_delete,
    ..ITEM_BASE
};

pub(crate) fn register(types: &mut TypeRegistry) {
    types.register_mobile(&PLAYER_VENDOR, &[]);
    types.register_item(&VENDOR_BACKPACK, &[]);
    types.register_item(&PLAYER_VENDOR_PLACEHOLDER, &[]);
    types.register_phase_default(
        muo_server::type_registry::Phase::Configure,
        "PlayerVendor.Configure",
        configure,
    );
}

#[allow(dead_code)]
fn _unused(w: &mut World, id: MobileId) {
    let _ = (&BASE_CREATURE_C, Point3D::ZERO, MessageType::REGULAR);
    let _ = (w.mobile(id), props_a::set_title);
}
