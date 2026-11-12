//! `RentedVendor` (`[SerializationGenerator(1)]`, a `PlayerVendor` that is rented), and the
//! owner-side traits the player vendor and rental gumps ask for (`muo_gumps::m_z::ops`) plus the `HouseDeps` and
//! `PlayerVendor` hooks of the house system.

use std::rc::Rc;

use muo_content::items::house_deps::deps_mut;
use muo_content::mx;
use muo_content::vendor::banker;
use muo_gumps::m_z::ops::{
    Ops, PlayerVendorOps, PlayerVendorView, RENTAL_DURATIONS, RentalOps, RentalView,
};
use muo_io::{GenericReader, GenericWriter};
use muo_multis::house;
use muo_primitives::{DateTime, Point3D, TimeSpan};
use muo_server::World;
use muo_server::context_menu_entry::ContextMenuEntry;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::mobile::part1::props_b;
use muo_server::mobile::part2::speech;
use muo_server::mobile::{AccessLevel, MobileType};
use muo_server::prompt::{Prompt, add_prompt};
use muo_server::timer::{self, TimerHandle};
use muo_server::type_registry::{Phase, TypeRegistry};

use super::player_vendor::{self as pv, PLAYER_VENDOR, PvClass, data, is_player_vendor};
use super::rental_contract as rc;

/// `VendorRentalDuration.Instances[i].Duration`.
const DURATIONS: [TimeSpan; 4] = [
    TimeSpan::from_days(7),
    TimeSpan::from_days(14),
    TimeSpan::from_days(21),
    TimeSpan::from_days(28),
];

/// What `RentedVendor` adds to a vendor.
#[derive(Default)]
pub struct RentedData {
    pub rental_duration_id: i32,
    pub rental_price: i32,
    pub landlord_renew: bool,
    pub renter_renew: bool,
    pub renewal_price: i32,
    pub rental_gold: i32,
    pub rental_expire_time: DateTime,
    expire_timer: TimerHandle,
}

pub fn rdata(w: &World, id: MobileId) -> Option<&RentedData> {
    w.mobile(id)?.ext.get::<RentedData>()
}

fn rdata_mut(w: &mut World, id: MobileId) -> Option<&mut RentedData> {
    w.mobile_mut(id)?.ext.get_mut::<RentedData>()
}

/// `m is RentedVendor`.
pub fn is_rented(w: &World, id: MobileId) -> bool {
    rdata(w, id).is_some()
}

fn duration(d: &RentedData) -> TimeSpan {
    DURATIONS[d.rental_duration_id.clamp(0, DURATIONS.len() as i32 - 1) as usize]
}

/// `IsOwner(m)`.
fn is_owner(w: &World, id: MobileId, m: MobileId) -> bool {
    let owner = pv::owner(w, id);
    if owner == Some(m) || mx::access_level(w, m) >= AccessLevel::GameMaster {
        return true;
    }
    // Core.ML && AccountHandler.CheckAccount(m, Owner)
    w.core_ml()
        && owner
            .and_then(|o| w.mobile(o))
            .and_then(|o| o.account)
            .and_then(|a| muo_content::accounting::account(w, a))
            .is_some_and(|a| a.mobiles.contains(&Some(m)))
}

/// `IsLandlord(m)`.
pub fn is_landlord(w: &World, id: MobileId, m: MobileId) -> bool {
    pv::house_of(w, id).is_some_and(|h| house::access::is_owner(w, h, m))
}

/// `Landlord`.
pub fn landlord(w: &World, id: MobileId) -> Option<MobileId> {
    house::owner(w, pv::house_of(w, id)?)
}

/// `Renew`.
pub fn renew(w: &World, id: MobileId) -> bool {
    let Some(d) = rdata(w, id) else { return false };
    d.landlord_renew
        && d.renter_renew
        && pv::house_of(w, id)
            .is_some_and(|h| house::decay::decay_type(w, h) != house::DecayType::Condemned)
}

/// `ComputeRentalExpireDelay(out days, out hours)`.
pub fn compute_rental_expire_delay(w: &World, id: MobileId) -> (i32, i32) {
    let delay = rdata(w, id).map_or(TimeSpan::ZERO, |d| d.rental_expire_time - w.now);
    if delay <= TimeSpan::ZERO {
        (0, 0)
    } else {
        (delay.days(), delay.hours())
    }
}

/// `SendRentalExpireMessage(to)`.
pub fn send_rental_expire_message(w: &mut World, id: MobileId, to: MobileId) {
    let (days, hours) = compute_rental_expire_delay(w, id);
    // The rental contract on this vendor will expire in ~1_DAY~ day(s) and ~2_HOUR~ hour(s).
    muo_server::mobile::part2::messages::send_localized_message(
        w,
        to,
        1062464,
        &format!("{days}\t{hours}"),
        0,
    );
}

fn start_expire_timer(w: &mut World, id: MobileId, delay: TimeSpan) {
    let interval = rdata(w, id).map_or(DURATIONS[0], duration);
    let h = timer::start(w, delay, interval, 0, move |w| expire_tick(w, id));
    if let Some(d) = rdata_mut(w, id) {
        d.expire_timer = h;
    }
}

/// `RentalExpireTimer.OnTick`.
fn expire_tick(w: &mut World, id: MobileId) {
    if mx::deleted(w, id) {
        return;
    }
    let renewal_price = rdata(w, id).map_or(0, |d| d.renewal_price);
    if renew(w, id) && pv::hold_gold(w, id) >= renewal_price {
        let hold = pv::hold_gold(w, id);
        pv::set_hold_gold(w, id, hold - renewal_price);
        let now = w.now;
        if let Some(d) = rdata_mut(w, id) {
            d.rental_gold += renewal_price;
            d.rental_price = renewal_price;
            d.rental_expire_time = now + duration(d);
        }
    } else {
        pv::destroy(w, id, false);
    }
}

/// `new RentedVendor(owner, house, duration, rentalPrice, landlordRenew, rentalGold)`.
pub fn make(
    w: &mut World,
    owner: Option<MobileId>,
    house_id: Option<ItemId>,
    duration_id: usize,
    rental_price: i32,
    landlord_renew: bool,
    rental_gold: i32,
) -> MobileId {
    let id = pv::make(w, &RENTED_VENDOR, owner, house_id);
    let expire = w.now + DURATIONS[duration_id];
    if let Some(m) = w.mobile_mut(id) {
        m.ext.insert(RentedData {
            rental_duration_id: duration_id as i32,
            rental_price,
            landlord_renew,
            renter_renew: false,
            renewal_price: rental_price,
            rental_gold,
            rental_expire_time: expire,
            expire_timer: TimerHandle::default(),
        });
    }
    start_expire_timer(w, id, DURATIONS[duration_id]);
    id
}

fn destroy(w: &mut World, id: MobileId, to_backpack: bool) {
    let gold = rdata(w, id).map_or(0, |d| d.rental_gold);
    if gold > 0
        && let Some(h) = pv::house_of(w, id).filter(|&h| house::is_aos_rules(w, h))
    {
        let existing = house::data(w, h).and_then(|d| d.moving_crate);
        let crate_id = existing.unwrap_or_else(|| {
            let c = muo_multis::moving_crate::create(w, h);
            if let Some(d) = house::data_mut(w, h) {
                d.moving_crate = Some(c);
            }
            c
        });
        banker::deposit_into(w, crate_id, gold);
        if let Some(d) = rdata_mut(w, id) {
            d.rental_gold = 0;
        }
    }
    // PlayerVendor.Destroy(toBackpack)
    pv::destroy_base(w, id, to_backpack);
}

fn on_after_delete(w: &mut World, id: MobileId) {
    (PLAYER_VENDOR.on_after_delete)(w, id);
    let mut t = rdata_mut(w, id)
        .map(|d| std::mem::take(&mut d.expire_timer))
        .unwrap_or_default();
    t.cancel(w);
}

fn get_context_menu_entries(
    w: &mut World,
    id: MobileId,
    from: MobileId,
    list: &mut Vec<ContextMenuEntry>,
) {
    if mx::alive(w, from) {
        let entry = |number: i32, f: fn(&mut World, MobileId, MobileId)| {
            let mut e = ContextMenuEntry::new(number, -1);
            e.on_click = Some(Rc::new(move |w, from, target| {
                if let EntityId::Mobile(v) = target
                    && muo_server::mobile::part2::stats::check_alive(w, from, true)
                    && !mx::deleted(w, v)
                    && is_rented(w, v)
                {
                    f(w, v, from);
                }
            }));
            e
        };
        if is_owner(w, id, from) {
            list.push(entry(6209, contract_options));
        } else if is_landlord(w, id, from) {
            if rdata(w, id).is_some_and(|d| d.rental_gold > 0) {
                list.push(entry(6212, collect_rent));
            }
            list.push(entry(6218, terminate_contract));
            list.push(entry(6209, contract_options));
        }
    }
    (PLAYER_VENDOR.get_context_menu_entries)(w, id, from, list);
}

fn contract_options(w: &mut World, vendor: MobileId, from: MobileId) {
    if is_owner(w, vendor, from) {
        muo_gumps::m_z::vendor_rental::renter_display_to(w, from, vendor);
        send_rental_expire_message(w, vendor, from);
    } else if is_landlord(w, vendor, from) {
        muo_gumps::m_z::vendor_rental::landlord_display_to(w, from, vendor);
        send_rental_expire_message(w, vendor, from);
    }
}

fn collect_rent(w: &mut World, vendor: MobileId, from: MobileId) {
    if !is_landlord(w, vendor, from) {
        return;
    }
    let gold = rdata(w, vendor).map_or(0, |d| d.rental_gold);
    if gold > 0 {
        let deposited = banker::deposit_up_to(w, from, gold, true, 0);
        if let Some(d) = rdata_mut(w, vendor) {
            d.rental_gold -= deposited;
        }
        if deposited > 0 {
            // ~1_AMOUNT~ gold has been deposited into your bank box.
            muo_server::mobile::part2::messages::send_localized_message(
                w,
                from,
                1060397,
                &deposited.to_string(),
                0,
            );
        }
        if rdata(w, vendor).is_some_and(|d| d.rental_gold > 0) {
            mx::msg(w, from, 500390); // Your bank box is full.
        }
    }
}

fn terminate_contract(w: &mut World, vendor: MobileId, from: MobileId) {
    if !is_landlord(w, vendor, from) {
        return;
    }
    // Enter the amount of gold you wish to offer the renter in exchange for immediate termination of this contract?
    mx::msg(w, from, 1062503);
    let p = Prompt {
        on_response: Some(Rc::new(move |w, from, text| {
            refund_offer(w, vendor, from, text)
        })),
        on_cancel: None,
    };
    let pid = add_prompt(w, p);
    props_b::set_prompt(w, from, Some(pid));
}

/// `RefundOfferPrompt.OnResponse`.
fn refund_offer(w: &mut World, vendor: MobileId, from: MobileId, text: &str) {
    if !pv::can_interact_with(w, vendor, from, false) || !is_landlord(w, vendor, from) {
        return;
    }
    let amount = text.trim().parse::<i32>().unwrap_or(-1);
    let Some(owner) = pv::owner(w, vendor) else {
        return;
    };
    if amount < 0 {
        mx::msg(w, from, 1062506); // You did not enter a valid amount.  Offer canceled.
    } else if banker::get_balance(w, from) < amount {
        mx::msg(w, from, 1062507); // You do not have that much money in your bank account.
    } else if mx::map(w, owner) != mx::map(w, vendor) || !mx::in_range(w, owner, vendor, 5) {
        mx::msg(w, from, 1062505); // The renter must be closer to the vendor in order for you to make this offer.
    } else {
        mx::msg(w, from, 1062504); // Please wait while the renter considers your offer.
        muo_gumps::m_z::vendor_rental::refund_display_to(w, owner, vendor, from, amount);
    }
}

// ---- serialization ----

fn serialize(w: &World, id: MobileId, out: &mut dyn GenericWriter) {
    (PLAYER_VENDOR.serialize)(w, id, out);
    out.write_encoded_int(1);
    let Some(d) = rdata(w, id) else { return };
    out.write_i32(d.rental_duration_id);
    out.write_i32(d.rental_price);
    out.write_bool(d.landlord_renew);
    out.write_bool(d.renter_renew);
    out.write_i32(d.renewal_price);
    out.write_i32(d.rental_gold);
    out.write_anchored_time_ticks(d.rental_expire_time.ticks());
}

fn deserialize(w: &mut World, id: MobileId, r: &mut dyn GenericReader) -> muo_io::Result<()> {
    (PLAYER_VENDOR.deserialize)(w, id, r)?;
    let version = r.read_encoded_int()?;
    let mut d = RentedData {
        rental_duration_id: r.read_i32()?,
        rental_price: r.read_i32()?,
        landlord_renew: r.read_bool()?,
        renter_renew: r.read_bool()?,
        renewal_price: r.read_i32()?,
        rental_gold: r.read_i32()?,
        ..RentedData::default()
    };
    d.rental_expire_time = if version == 0 {
        DateTime::from_ticks(r.read_delta_time_ticks(w.now.ticks())?)
    } else {
        DateTime::from_ticks(r.read_anchored_time_ticks()?)
    };
    let expire = d.rental_expire_time;
    d.rental_duration_id = d.rental_duration_id.clamp(0, DURATIONS.len() as i32 - 1);
    if let Some(m) = w.mobile_mut(id) {
        m.ext.insert(d);
    }
    // [AfterDeserialization]
    let delay = expire - w.now;
    start_expire_timer(
        w,
        id,
        if delay > TimeSpan::ZERO {
            delay
        } else {
            TimeSpan::ZERO
        },
    );
    Ok(())
}

static RENTED_PV_CLASS: PvClass = PvClass {
    is_owner,
    destroy,
    ..pv::BASE_PV_CLASS
};

pub static RENTED_VENDOR: MobileType = MobileType {
    full_name: "Server.Mobiles.RentedVendor",
    parent: Some(&PLAYER_VENDOR),
    construct: None,
    family: Some(&RENTED_PV_CLASS),
    serialize,
    deserialize,
    on_after_delete,
    get_context_menu_entries,
    ..PLAYER_VENDOR
};

// ---- PlayerVendorOps ----

struct PvOps;

impl PlayerVendorOps for PvOps {
    fn view(&self, w: &World, vendor: MobileId) -> Option<PlayerVendorView> {
        let d = data(w, vendor)?;
        Some(PlayerVendorView {
            vendor,
            charge_per_day: pv::charge_per_day(w, vendor),
            charge_per_real_world_day: pv::charge_per_real_world_day(w, vendor),
            hold_gold: d.hold_gold,
            bank_account: d.bank_account,
            shop_name: pv::shop_name(w, vendor),
            female: w.mobile(vendor).is_some_and(|m| m.female),
            rental_expires: is_rented(w, vendor).then(|| compute_rental_expire_delay(w, vendor)),
        })
    }
    fn can_interact_with(
        &self,
        w: &mut World,
        vendor: MobileId,
        from: MobileId,
        owner_only: bool,
    ) -> bool {
        pv::can_interact_with(w, vendor, from, owner_only)
    }
    fn is_owner(&self, w: &World, vendor: MobileId, from: MobileId) -> bool {
        pv::is_owner(w, vendor, from)
    }
    fn barkeeper_is_owner(&self, w: &World, vendor: MobileId, from: MobileId) -> Option<bool> {
        super::barkeeper::is_barkeeper(w, vendor)
            .then(|| super::barkeeper::is_owner(w, vendor, from))
    }
    fn is_player_vendor(&self, w: &World, vendor: MobileId) -> bool {
        is_player_vendor(w, vendor)
    }
    fn say(&self, w: &mut World, vendor: MobileId, text: &str) {
        muo_server::mobile::part2::messages::say_text(w, vendor, text);
    }
    fn say_to(&self, w: &mut World, vendor: MobileId, to: MobileId, number: i32) {
        speech::say_to_number(w, vendor, to, number, "");
    }
    fn add_hold_gold(&self, w: &mut World, vendor: MobileId, amount: i32) {
        let g = pv::hold_gold(w, vendor);
        pv::set_hold_gold(w, vendor, g.wrapping_add(amount));
    }
    fn vendor_item_valid_in_pack(&self, w: &World, vendor: MobileId, item: ItemId) -> bool {
        pv::get_vendor_item(w, vendor, item).is_some_and(|v| v.valid)
    }
    fn open_backpack(&self, w: &mut World, vendor: MobileId, from: MobileId) {
        pv::open_backpack(w, vendor, from);
    }
    fn check_teleport(&self, w: &mut World, vendor: MobileId, from: MobileId) {
        pv::check_teleport(w, vendor, from);
    }
    fn rename_shop(&self, w: &mut World, vendor: MobileId, from: MobileId) {
        pv::rename_shop(w, vendor, from);
    }
    fn rename(&self, w: &mut World, vendor: MobileId, from: MobileId) {
        pv::rename(w, vendor, from);
    }
    fn display_paperdoll_to(&self, w: &mut World, vendor: MobileId, from: MobileId) {
        if let Some(t) = w.mobile(vendor).map(|m| m.ty) {
            (t.display_paperdoll_to)(w, vendor, from);
        }
    }
    fn collect_gold(&self, w: &mut World, vendor: MobileId, from: MobileId) {
        pv::collect_gold(w, vendor, from);
    }
    fn dismiss(&self, w: &mut World, vendor: MobileId, from: MobileId) {
        pv::dismiss(w, vendor, from);
    }
}

// ---- RentalOps ----

struct RentOps;

impl RentalOps for RentOps {
    fn deleted_contract(&self, w: &World, contract: ItemId) -> bool {
        w.item(contract).is_none_or(|i| i.deleted())
    }
    fn contract_usable_by(
        &self,
        w: &mut World,
        contract: ItemId,
        from: MobileId,
        flags: [bool; 4],
    ) -> bool {
        rc::contract_usable_by(w, contract, from, flags)
    }
    fn contract_view(&self, w: &World, contract: ItemId, _from: MobileId) -> Option<RentalView> {
        rc::contract_view(w, contract)
    }
    fn contract_is_locked_down(&self, w: &World, contract: ItemId) -> bool {
        muo_server::item::ops::is_locked_down(w, contract)
    }
    fn contract_set_duration(&self, w: &mut World, contract: ItemId, duration: usize) {
        rc::contract_set_duration(w, contract, duration);
    }
    fn contract_set_price(&self, w: &mut World, contract: ItemId, price: i32) {
        rc::contract_set_price(w, contract, price);
    }
    fn contract_toggle_landlord_renew(&self, w: &mut World, contract: ItemId) {
        rc::contract_toggle_landlord_renew(w, contract);
    }
    fn contract_is_landlord(&self, w: &World, contract: ItemId, from: MobileId) -> bool {
        rc::is_landlord(w, contract, from)
    }
    fn contract_set_offeree(&self, w: &mut World, contract: ItemId, offeree: Option<MobileId>) {
        rc::set_offeree(w, contract, offeree);
    }
    fn contract_offeree(&self, w: &World, contract: ItemId) -> Option<MobileId> {
        rc::offeree(w, contract)
    }
    fn contract_location(&self, w: &World, contract: ItemId) -> Point3D {
        rc::contract_location(w, contract)
    }
    fn contract_accept(
        &self,
        w: &mut World,
        contract: ItemId,
        renter: MobileId,
        gold_to_give: i32,
    ) -> Result<(), ()> {
        rc::contract_accept(w, contract, renter, gold_to_give)
    }
    fn contract_has_house(&self, w: &World, contract: ItemId) -> bool {
        rc::contract_has_house(w, contract)
    }
    fn contract_can_fit(&self, w: &mut World, contract: ItemId) -> bool {
        rc::contract_can_fit(w, contract)
    }

    fn vendor_view(&self, w: &World, vendor: MobileId) -> Option<RentalView> {
        let d = rdata(w, vendor)?;
        let landlord_name =
            landlord(w, vendor).and_then(|l| w.mobile(l).and_then(|m| (m.ty.name)(w, l)));
        let renter_name =
            pv::owner(w, vendor).and_then(|l| w.mobile(l).and_then(|m| (m.ty.name)(w, l)));
        Some(RentalView {
            duration: d
                .rental_duration_id
                .clamp(0, RENTAL_DURATIONS.len() as i32 - 1) as usize,
            price: d.rental_price,
            renewal_price: d.renewal_price,
            landlord_name,
            renter_name,
            landlord_renew: d.landlord_renew,
            renter_renew: d.renter_renew,
            renew: renew(w, vendor),
        })
    }
    fn vendor_can_interact(
        &self,
        w: &mut World,
        vendor: MobileId,
        from: MobileId,
        owner_only: bool,
    ) -> bool {
        pv::can_interact_with(w, vendor, from, owner_only)
    }
    fn vendor_is_landlord(&self, w: &World, vendor: MobileId, from: MobileId) -> bool {
        is_landlord(w, vendor, from)
    }
    fn vendor_toggle_renter_renew(&self, w: &mut World, vendor: MobileId) {
        if let Some(d) = rdata_mut(w, vendor) {
            d.renter_renew = !d.renter_renew;
        }
    }
    fn vendor_toggle_landlord_renew(&self, w: &mut World, vendor: MobileId) {
        if let Some(d) = rdata_mut(w, vendor) {
            d.landlord_renew = !d.landlord_renew;
        }
    }
    fn vendor_set_renewal_price(&self, w: &mut World, vendor: MobileId, price: i32) {
        if let Some(d) = rdata_mut(w, vendor) {
            d.renewal_price = price;
            d.renter_renew = false;
        }
    }
    fn vendor_name(&self, w: &World, vendor: MobileId) -> String {
        w.mobile(vendor)
            .and_then(|m| (m.ty.name)(w, vendor))
            .unwrap_or_default()
    }
    fn vendor_shop_name(&self, w: &World, vendor: MobileId) -> String {
        pv::shop_name(w, vendor)
    }
    fn vendor_add_hold_gold(&self, w: &mut World, vendor: MobileId, amount: i32) {
        let g = pv::hold_gold(w, vendor);
        pv::set_hold_gold(w, vendor, g.wrapping_add(amount));
    }
    fn vendor_destroy(&self, w: &mut World, vendor: MobileId) {
        pv::destroy(w, vendor, false);
    }
}

/// Configure phase: installs the ops traits and sets the hooks the house system and the bulk order books wait for.
fn configure(w: &mut World) {
    let ops = &mut muo_gumps::m_z::hooks::hooks_mut(w).ops;
    *ops = Ops {
        player_vendors: Some(Rc::new(PvOps)),
        rentals: Some(Rc::new(RentOps)),
        ..ops.clone()
    };

    let d = deps_mut(w);
    d.vendor_return = Some(pv::return_vendor);
    d.vendor_destroy = Some(|w, v| pv::destroy(w, v, true));
    d.vendor_can_interact = Some(|w, v, m| {
        // `CanInteractWith(m, false)` reads the world mutably (messages); the pure part is answered here.
        is_player_vendor(w, v)
            && w.mobile(m)
                .map(|x| x.ty)
                .is_some_and(|x| (x.can_see_mobile)(w, m, v))
            && muo_server::mobile::part1::combat::in_update_range(
                mx::location(w, m),
                mx::location(w, v),
            )
            && mx::alive(w, m)
            && !(pv::house_of(w, v).is_some_and(|h| house::access::is_banned(w, h, m))
                && !pv::is_owner(w, v, m))
    });
    d.vendor_pack_items = Some(pv::pack_item_count);
    d.vendor_is_rented = Some(is_rented);
}

pub(crate) fn register(types: &mut TypeRegistry) {
    types.register_mobile(&RENTED_VENDOR, &[]);
    types.register_phase_default(Phase::Configure, "RentedVendor.Configure", configure);
}

#[allow(dead_code)]
fn _unused(w: &World, id: MobileId) {
    let _ = (props_b::set_prompt, DateTime::MIN, w.mobile(id));
}
