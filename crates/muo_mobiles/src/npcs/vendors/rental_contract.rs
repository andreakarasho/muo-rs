//! `VendorRentalContract` (`[SerializationGenerator(0)]`) and the `RentalOps` contract half the
//! rental gumps (`muo_gumps::m_z::vendor_rental`) call. The vendor half is in `rented.rs`.

use std::rc::Rc;

use muo_content::mx;
use muo_content::vendor::banker;
use muo_gumps::m_z::ops::RentalView;
use muo_gumps::m_z::vendor_rental::{OfferKind, RentalGump, contract_display_to};
use muo_io::{GenericReader, GenericWriter};
use muo_multis::house;
use muo_primitives::{Point3D, TimeSpan};
use muo_server::World;
use muo_server::context_menu_entry::ContextMenuEntry;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType, deps, ops};
use muo_server::map::fit;
use muo_server::mobile::part2::{items, messages};
use muo_server::property_list::PropertyList;
use muo_server::target::{self, Target, TargetFlags, Targeted};
use muo_server::timer::{self, TimerHandle};
use muo_server::type_registry::TypeRegistry;

/// What the contract adds to an `Item`.
#[derive(Default)]
pub struct ContractData {
    /// `_rentalDurationId` (the `Duration` is `VendorRentalDuration.Instances[id]`).
    pub duration_id: i32,
    pub price: i32,
    pub landlord_renew: bool,
    offeree: Option<MobileId>,
    offer_timer: TimerHandle,
}

pub fn cdata(w: &World, id: ItemId) -> Option<&ContractData> {
    w.item(id)?.ext.get::<ContractData>()
}

fn cdata_mut(w: &mut World, id: ItemId) -> Option<&mut ContractData> {
    w.item_mut(id)?.ext.get_mut::<ContractData>()
}

fn msg(w: &mut World, to: MobileId, number: i32) {
    deps::send_localized_message(w, to, number);
}

/// `IsChildOf(from.Backpack)`.
fn in_pack(w: &World, id: ItemId, from: MobileId) -> bool {
    let pack = items::find_item_on_layer(w, from, muo_server::layer::Layer::Backpack);
    ops::is_child_of(w, id, pack.map(EntityId::Item))
}

fn delete(w: &mut World, id: ItemId) {
    if let Some(ty) = w.item(id).map(|i| i.ty) {
        (ty.delete)(w, id);
    }
}

/// `Offeree`.
pub fn offeree(w: &World, id: ItemId) -> Option<MobileId> {
    cdata(w, id)?.offeree
}

/// `Offeree = value`: restarts the 30 second expiry.
pub fn set_offeree(w: &mut World, id: ItemId, value: Option<MobileId>) {
    let mut t = cdata_mut(w, id)
        .map(|d| std::mem::take(&mut d.offer_timer))
        .unwrap_or_default();
    t.cancel(w);
    let h = value.map(|_| {
        timer::start(w, TimeSpan::from_secs(30), TimeSpan::ZERO, 1, move |w| {
            offer_expired(w, id)
        })
    });
    if let Some(d) = cdata_mut(w, id) {
        d.offeree = value;
        d.offer_timer = h.unwrap_or_default();
    }
    ops::invalidate_properties(w, id);
}

/// `OfferExpireTimer.OnTick`.
fn offer_expired(w: &mut World, id: ItemId) {
    if let Some(o) = offeree(w, id) {
        if let Some(ns) = w.mobile(o).and_then(|m| m.net_state) {
            muo_content::gumps::system::close_gump::<RentalGump<OfferKind>>(w, ns);
        }
        set_offeree(w, id, None);
    }
}

/// `IsLandlord(m)`.
pub fn is_landlord(w: &World, id: ItemId, m: MobileId) -> bool {
    if ops::is_locked_down(w, id)
        && let Some(h) = house::access::find_house_at_item(w, id)
        && house::decay::decay_type(w, h) != house::DecayType::Condemned
    {
        return house::access::is_owner(w, h, m);
    }
    false
}

/// Why `IsUsableBy` said no: a message to send (`0` none, `-1` the `CheckAlive` one).
enum No {
    Silent,
    Dead,
    Msg(i32),
}

fn usable_by_check(
    w: &World,
    id: ItemId,
    from: MobileId,
    by_landlord: bool,
    by_backpack: bool,
    no_offeree: bool,
) -> Result<(), No> {
    if w.item(id).is_none_or(|i| i.deleted()) {
        return Err(No::Silent);
    }
    if !mx::alive(w, from) {
        return Err(No::Dead);
    }
    if no_offeree && offeree(w, id).is_some() {
        return Err(No::Msg(1062343)); // That item is currently in use.
    }
    if by_backpack && in_pack(w, id, from) {
        return Ok(());
    }
    if by_landlord && is_landlord(w, id, from) {
        let item_map = w.item(id).and_then(|i| i.map);
        let loc = w.item(id).map_or(Point3D::ZERO, |i| i.location);
        if mx::map(w, from) != item_map || !mx::in_range_point(w, from, loc, 5) {
            return Err(No::Msg(501853)); // Target is too far away.
        }
        return Ok(());
    }
    Err(No::Silent)
}

/// `IsUsableBy(from, byLandlord, byBackpack, noOfferee, sendMessage)`.
pub fn is_usable_by(
    w: &mut World,
    id: ItemId,
    from: MobileId,
    by_landlord: bool,
    by_backpack: bool,
    no_offeree: bool,
    send_message: bool,
) -> bool {
    match usable_by_check(w, id, from, by_landlord, by_backpack, no_offeree) {
        Ok(()) => true,
        Err(No::Dead) => {
            if send_message {
                muo_server::mobile::part2::stats::check_alive(w, from, true);
            }
            false
        }
        Err(No::Msg(n)) => {
            if send_message {
                msg(w, from, n);
            }
            false
        }
        Err(No::Silent) => false,
    }
}

// ---- the item ----

fn make(w: &mut World) -> ItemId {
    let id = ops::new_item_with_id(w, &VENDOR_RENTAL_CONTRACT, 0x14F0);
    muo_server::item::base::set_hue(w, id, 0x672);
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(ContractData {
            duration_id: 0,
            price: 1500,
            ..ContractData::default()
        });
    }
    id
}

fn get_properties(w: &World, id: ItemId, list: &mut PropertyList) {
    (ITEM_BASE.get_properties)(w, id, list);
    if let Some(o) = offeree(w, id) {
        let name = w
            .mobile(o)
            .and_then(|m| (m.ty.name)(w, o))
            .unwrap_or_default();
        list.add_str(1062368, &name); // Being Offered To ~1_NAME~
    }
}

fn on_delete(w: &mut World, id: ItemId) {
    // Does not call the base method.
    if ops::is_locked_down(w, id)
        && let Some(h) = house::access::find_house_at_item(w, id)
        && let Some(d) = house::data_mut(w, h)
    {
        d.vendor_rental_contracts.retain(|&i| i != id);
    }
}

fn on_double_click(w: &mut World, id: ItemId, from: MobileId) {
    if offeree(w, id).is_some() {
        msg(w, from, 1062343); // That item is currently in use.
    } else if !ops::is_locked_down(w, id) {
        if !in_pack(w, id, from) {
            msg(w, from, 1062334); // This item must be in your backpack to be used.
            return;
        }
        let h = house::access::find_house_at_mobile(w, from);
        if !h.is_some_and(|h| house::access::is_owner(w, h, from)) {
            // You must be standing inside of a house that you own to make use of this contract.
            msg(w, from, 1062333);
        } else if let Some(h) = h {
            if !house::is_aos_rules(w, h) {
                messages::send_message(
                    w,
                    from,
                    "Rental contracts can only be placed in AOS-enabled houses.",
                );
            } else if !house::data(w, h).is_some_and(|d| d.public) {
                msg(w, from, 1062335); // Rental contracts can only be placed in public houses.
            } else if !house::storage::can_place_new_vendor(w, h) {
                msg(w, from, 1062352); // You do not have enough storage available to place this contract.
            } else {
                msg(w, from, 1062337); // Target the exact location you wish to rent out.
                begin_rent_target(w, from, id);
            }
        }
    } else if is_landlord(w, id, from) {
        if mx::in_range_point(w, from, w.item(id).map_or(Point3D::ZERO, |i| i.location), 5) {
            contract_display_to(w, from, id);
        } else {
            msg(w, from, 501853); // Target is too far away.
        }
    }
}

fn get_context_menu_entries(
    w: &mut World,
    id: ItemId,
    from: MobileId,
    list: &mut Vec<ContextMenuEntry>,
) {
    (ITEM_BASE.get_context_menu_entries)(w, id, from, list);
    if usable_by_check(w, id, from, true, true, true).is_ok() {
        let mut e = ContextMenuEntry::new(6209, -1);
        e.on_click = Some(Rc::new(|w, from, target| {
            if let EntityId::Item(c) = target
                && cdata(w, c).is_some()
                && is_usable_by(w, c, from, true, true, true, true)
            {
                contract_display_to(w, from, c);
            }
        }));
        list.push(e);
    }
}

/// `RentTarget`.
fn begin_rent_target(w: &mut World, from: MobileId, contract: ItemId) {
    let mut t = Target::new(w, -1, false, TargetFlags::NONE);
    t.handlers.on_target = Some(Rc::new(
        move |w: &mut World, from: MobileId, targeted: Targeted| {
            rent_on_target(w, from, contract, targeted)
        },
    ));
    t.handlers.on_target_cancel = Some(Rc::new(move |w: &mut World, from: MobileId, _| {
        msg(w, from, 1062336); // You decide not to place the contract at this time.
    }));
    let id = target::add_target(w, t);
    target::set_mobile_target(w, from, Some(id));
}

fn rent_on_target(w: &mut World, from: MobileId, contract: ItemId, targeted: Targeted) {
    if !is_usable_by(w, contract, from, false, true, true, true) {
        return;
    }
    let location = match targeted {
        Targeted::Entity(EntityId::Mobile(m)) => w.mobile(m).map(|m| m.location),
        Targeted::Entity(EntityId::Item(i)) => w.item(i).map(|i| i.location),
        Targeted::Land(l) => Some(l.location),
        Targeted::Static(s) => Some(s.location),
    };
    let Some(p_location) = location else { return };
    let map = mx::map(w, from);

    let h = house::access::find_house_at_point(w, p_location, map, 0);
    if !h.is_some_and(|h| house::access::is_owner(w, h, from)) {
        msg(w, from, 1062338); // The location being rented out must be inside of your house.
        return;
    }
    let Some(h) = h else { return };
    if house::access::find_house_at_mobile(w, from) != Some(h) {
        // You must be located inside of the house in which you are trying to place the contract.
        msg(w, from, 1062339);
    } else if !house::is_aos_rules(w, h) {
        messages::send_message(
            w,
            from,
            "Rental contracts can only be placed in AOS-enabled houses.",
        );
    } else if !house::data(w, h).is_some_and(|d| d.public) {
        msg(w, from, 1062335); // Rental contracts can only be placed in public houses.
    } else if house::decay::decay_type(w, h) == house::DecayType::Condemned {
        msg(w, from, 1062468); // You cannot place a contract in a condemned house.
    } else if !house::storage::can_place_new_vendor(w, h) {
        msg(w, from, 1062352); // You do not have enought storage available to place this contract.
    } else if !map.is_some_and(|m| {
        fit::can_fit(
            w,
            m,
            p_location.x,
            p_location.y,
            p_location.z,
            16,
            false,
            false,
            true,
        )
    }) {
        msg(w, from, 1062486); // A vendor cannot exist at that location.  Please try again.
    } else {
        let (vendor, cont) = house::vendors::is_there_vendor(w, p_location, map);
        if vendor {
            // You may not place a rental contract at this location while other beings occupy it.
            msg(w, from, 1062342);
        } else if cont {
            // That location is cluttered.  Please clear out any objects there and try again.
            msg(w, from, 1062341);
        } else {
            ops::move_to_world_map(w, contract, p_location, map);
            if !house::storage::lock_down(w, h, from, contract, true) {
                items::add_to_backpack(w, from, contract);
            }
        }
    }
}

// ---- serialization ----

fn serialize(w: &World, id: ItemId, out: &mut dyn GenericWriter) {
    (ITEM_BASE.serialize)(w, id, out);
    out.write_encoded_int(0);
    let Some(d) = cdata(w, id) else { return };
    out.write_i32(d.duration_id);
    out.write_i32(d.price);
    out.write_bool(d.landlord_renew);
}

fn deserialize(w: &mut World, id: ItemId, r: &mut dyn GenericReader) -> muo_io::Result<()> {
    (ITEM_BASE.deserialize)(w, id, r)?;
    let _version = r.read_encoded_int()?;
    let d = ContractData {
        // [AfterDeserialization]: clamped to the four valid durations, 0..=3.
        duration_id: r.read_i32()?.clamp(0, 3),
        price: r.read_i32()?,
        landlord_renew: r.read_bool()?,
        ..ContractData::default()
    };
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(d);
    }
    Ok(())
}

pub static VENDOR_RENTAL_CONTRACT: ItemType = ItemType {
    full_name: "Server.Items.VendorRentalContract",
    parent: Some(&ITEM_BASE),
    construct: Some(make),
    default_weight: |_, _| 1.0,
    label_number: |_, _| 1062332, // a vendor rental contract
    get_properties,
    get_context_menu_entries,
    on_delete,
    on_double_click,
    serialize,
    deserialize,
    ..ITEM_BASE
};

// ---- RentalOps, contract half ----

fn view(w: &World, c: ItemId) -> Option<RentalView> {
    let d = cdata(w, c)?;
    Some(RentalView {
        duration: d.duration_id.clamp(0, 3) as usize,
        price: d.price,
        renewal_price: d.price,
        landlord_name: None,
        renter_name: None,
        landlord_renew: d.landlord_renew,
        renter_renew: false,
        renew: false,
    })
}

pub(crate) fn contract_view(w: &World, c: ItemId) -> Option<RentalView> {
    view(w, c)
}

pub(crate) fn contract_usable_by(
    w: &mut World,
    c: ItemId,
    from: MobileId,
    flags: [bool; 4],
) -> bool {
    // `[byLandlord, byBackpack, noOfferee, sendMessage]`
    is_usable_by(w, c, from, flags[0], flags[1], flags[2], flags[3])
}

pub(crate) fn contract_set_duration(w: &mut World, c: ItemId, duration: usize) {
    // `Duration = value`: the id is the index.
    if let Some(d) = cdata_mut(w, c) {
        d.duration_id = duration.min(3) as i32;
    }
    ops::invalidate_properties(w, c);
}

pub(crate) fn contract_set_price(w: &mut World, c: ItemId, price: i32) {
    if let Some(d) = cdata_mut(w, c) {
        d.price = price;
    }
}

pub(crate) fn contract_toggle_landlord_renew(w: &mut World, c: ItemId) {
    if let Some(d) = cdata_mut(w, c) {
        d.landlord_renew = !d.landlord_renew;
    }
}

pub(crate) fn contract_location(w: &World, c: ItemId) -> Point3D {
    w.item(c).map_or(Point3D::ZERO, |i| i.location)
}

pub(crate) fn contract_has_house(w: &World, c: ItemId) -> bool {
    house::access::find_house_at_item(w, c).is_some()
}

pub(crate) fn contract_can_fit(w: &mut World, c: ItemId) -> bool {
    let Some((map, loc)) = w.item(c).map(|i| (i.map, i.location)) else {
        return false;
    };
    map.is_some_and(|m| fit::can_fit(w, m, loc.x, loc.y, loc.z, 16, false, false, true))
}

/// `new RentedVendor(from, house, _contract.Duration, price, _contract.LandlordRenew, goldToGive)` moved to the
/// contract's place, then `_contract.Delete()`.
pub(crate) fn contract_accept(
    w: &mut World,
    c: ItemId,
    renter: MobileId,
    gold_to_give: i32,
) -> Result<(), ()> {
    let h = house::access::find_house_at_item(w, c).ok_or(())?;
    let (duration, price, renew) = cdata(w, c)
        .map(|d| (d.duration_id, d.price, d.landlord_renew))
        .ok_or(())?;
    let (loc, map) = w.item(c).map(|i| (i.location, i.map)).ok_or(())?;
    let v = super::rented::make(
        w,
        Some(renter),
        Some(h),
        duration.clamp(0, 3) as usize,
        price,
        renew,
        gold_to_give,
    );
    if let Some(map) = map {
        muo_server::mobile::base::move_to_world(w, v, loc, map);
    }
    delete(w, c);
    Ok(())
}

#[allow(dead_code)]
fn _unused() {
    let _ = banker::get_balance;
}

pub(crate) fn register(types: &mut TypeRegistry) {
    types.register_item(&VENDOR_RENTAL_CONTRACT, &[]);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_offer_lasts_thirty_seconds_and_blocks_use() {
        let mut w = crate::test_world();
        let c = make(&mut w);
        let m = w.new_mobile(&muo_server::mobile::MOBILE_BASE);
        assert_eq!(offeree(&w, c), None);
        set_offeree(&mut w, c, Some(m));
        assert_eq!(offeree(&w, c), Some(m));
        // noOfferee: "That item is currently in use."
        assert!(!is_usable_by(&mut w, c, m, false, true, true, false));
        set_offeree(&mut w, c, None);
        assert_eq!(offeree(&w, c), None);
        // Not locked down: nobody is its landlord, and an accept finds no house.
        assert!(!is_landlord(&w, c, m));
        assert!(!contract_has_house(&w, c));
        assert!(contract_accept(&mut w, c, m, 0).is_err());
    }
}
