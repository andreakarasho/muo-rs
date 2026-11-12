//! The rental contract gumps. The contract and the rented vendor are behind
//! [`RentalOps`](super::ops::RentalOps).
//!
//! `BaseVendorRentalGump` and its four subclasses are one [`RentalGump`] over a [`RentalKind`] that names the
//! class (the type id) and says which object it works on.

use std::marker::PhantomData;
use std::rc::Rc;

use muo_content::gumps::GumpButtonType;
use muo_content::gumps::base::{
    DynamicGumpLogic, DynamicGumpObj, RelayInfo, StaticGumpLogic, StaticGumpObj,
};
use muo_content::gumps::builder::{DynamicGumpBuilder, GumpStrings, StaticGumpBuilder};
use muo_server::World;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::net_state::NetStateId;
use muo_server::prompt::{Prompt, add_prompt};
use muo_server::target::{
    Target, TargetCancelType, TargetFlags, Targeted, add_target, set_mobile_target,
};

use super::common::{has_net_state, mobile_of, name_of, send, send_singleton};
use super::hooks::ops;
use super::ops::{RENTAL_DURATIONS, RentalOps, RentalView};

/// `BaseVendorRentalGump.GumpType`.
#[derive(Copy, Clone, Eq, PartialEq)]
pub enum GumpType {
    UnlockedContract,
    LockedContract,
    Offer,
    VendorLandlord,
    VendorRenter,
}

/// Names a subclass of `BaseVendorRentalGump`.
pub trait RentalKind: 'static {
    const TYPE_NAME: &'static str;
}

macro_rules! kind {
    ($name:ident, $full:literal) => {
        pub struct $name;
        impl RentalKind for $name {
            const TYPE_NAME: &'static str = $full;
        }
    };
}
kind!(ContractKind, "Server.Gumps.VendorRentalContractGump");
kind!(OfferKind, "Server.Gumps.VendorRentalOfferGump");
kind!(RenterKind, "Server.Gumps.RenterVendorRentalGump");
kind!(LandlordKind, "Server.Gumps.LandlordVendorRentalGump");

/// What the gump works on.
#[derive(Copy, Clone)]
enum Subject {
    Contract(ItemId),
    Vendor(MobileId),
}

pub struct RentalGump<K: RentalKind> {
    ty: GumpType,
    duration: usize,
    price: i32,
    renewal_price: i32,
    landlord: Option<String>,
    renter: Option<String>,
    landlord_renew: bool,
    renter_renew: bool,
    renew: bool,
    subject: Subject,
    /// The landlord of an offer.
    offer_landlord: Option<MobileId>,
    _kind: PhantomData<K>,
}

fn rental_ops(w: &World) -> Option<Rc<dyn RentalOps>> {
    ops(w).rentals
}

#[allow(clippy::too_many_arguments)]
fn make<K: RentalKind>(
    ty: GumpType,
    v: &RentalView,
    landlord: Option<String>,
    renter: Option<String>,
    subject: Subject,
    offer_landlord: Option<MobileId>,
    renew: bool,
) -> DynamicGumpObj<RentalGump<K>> {
    DynamicGumpObj::new(
        100,
        100,
        RentalGump {
            ty,
            duration: v.duration,
            price: v.price,
            renewal_price: v.renewal_price,
            landlord,
            renter,
            landlord_renew: v.landlord_renew,
            renter_renew: v.renter_renew,
            renew,
            subject,
            offer_landlord,
            _kind: PhantomData,
        },
    )
}

/// `VendorRentalContractGump.DisplayTo(from, contract)`.
pub fn contract_display_to(w: &mut World, from: MobileId, contract: ItemId) {
    let Some(o) = rental_ops(w) else { return };
    if !has_net_state(w, from) || o.deleted_contract(w, contract) {
        return;
    }
    let Some(v) = o.contract_view(w, contract, from) else {
        return;
    };
    let ty = if o.contract_is_locked_down(w, contract) {
        GumpType::LockedContract
    } else {
        GumpType::UnlockedContract
    };
    let landlord = Some(name_of(w, from));
    let renew_v = RentalView {
        renter_renew: false,
        renewal_price: v.price,
        ..v
    };
    let g = make::<ContractKind>(
        ty,
        &renew_v,
        landlord,
        None,
        Subject::Contract(contract),
        None,
        false,
    );
    send_singleton(w, from, g);
}

/// `VendorRentalOfferGump.DisplayTo(to, contract, landlord)`.
pub fn offer_display_to(w: &mut World, to: MobileId, contract: ItemId, landlord: MobileId) {
    let Some(o) = rental_ops(w) else { return };
    if !has_net_state(w, to) || o.deleted_contract(w, contract) {
        return;
    }
    let Some(v) = o.contract_view(w, contract, landlord) else {
        return;
    };
    let v = RentalView {
        renter_renew: false,
        renewal_price: v.price,
        ..v
    };
    let g = make::<OfferKind>(
        GumpType::Offer,
        &v,
        Some(name_of(w, landlord)),
        None,
        Subject::Contract(contract),
        Some(landlord),
        false,
    );
    send(w, to, g);
}

/// `RenterVendorRentalGump.DisplayTo(from, vendor)`.
pub fn renter_display_to(w: &mut World, from: MobileId, vendor: MobileId) {
    vendor_display_to::<RenterKind>(w, from, vendor, GumpType::VendorRenter);
}

/// `LandlordVendorRentalGump.DisplayTo(from, vendor)`.
pub fn landlord_display_to(w: &mut World, from: MobileId, vendor: MobileId) {
    vendor_display_to::<LandlordKind>(w, from, vendor, GumpType::VendorLandlord);
}

fn vendor_display_to<K: RentalKind>(w: &mut World, from: MobileId, vendor: MobileId, ty: GumpType) {
    let Some(o) = rental_ops(w) else { return };
    if !has_net_state(w, from) || muo_content::mx::deleted(w, vendor) {
        return;
    }
    let Some(v) = o.vendor_view(w, vendor) else {
        return;
    };
    let (landlord, renter, renew) = (v.landlord_name.clone(), v.renter_name.clone(), v.renew);
    let g = make::<K>(
        ty,
        &v,
        landlord,
        renter,
        Subject::Vendor(vendor),
        None,
        renew,
    );
    send_singleton(w, from, g);
}

impl<K: RentalKind> DynamicGumpLogic for RentalGump<K> {
    const TYPE_NAME: &'static str = K::TYPE_NAME;

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        let ty = self.ty;
        if ty == GumpType::Offer {
            b.set_no_close();
        }

        b.add_page(0);

        b.add_image(0, 0, 0x1F40, 0);
        b.add_image_tiled(20, 37, 300, 308, 0x1F42);
        b.add_image(20, 325, 0x1F43, 0);

        b.add_image(35, 8, 0x39, 0);
        b.add_image_tiled(65, 8, 257, 10, 0x3A);
        b.add_image(290, 8, 0x3B, 0);

        b.add_image_tiled(70, 55, 230, 2, 0x23C5);

        b.add_image(32, 33, 0x2635, 0);
        b.add_html_localized_color(70, 35, 270, 20, 1062353, 0x1, false, false); // Vendor Rental Contract

        b.add_page(1);

        if ty != GumpType::UnlockedContract {
            b.add_image(65, 60, 0x827, 0);
            b.add_html_localized_color(79, 58, 270, 20, 1062370, 0x1, false, false); // Landlord:
            b.add_label(150, 58, 0x64, self.landlord.as_deref().unwrap_or(""));

            b.add_image_tiled(70, 80, 230, 2, 0x23C5);
        }

        let contract = matches!(ty, GumpType::UnlockedContract | GumpType::LockedContract);
        if contract {
            b.add_button_type(30, 96, 0x15E1, 0x15E5, 0, GumpButtonType::Page, 2);
        }

        b.add_html_localized_color(50, 95, 150, 20, 1062354, 0x1, false, false); // Contract Length
        b.add_html_localized_color(
            230,
            95,
            270,
            20,
            RENTAL_DURATIONS[self.duration.min(3)].name,
            0x1,
            false,
            false,
        );

        if contract {
            b.add_button(30, 116, 0x15E1, 0x15E5, 1);
        }

        b.add_html_localized_color(50, 115, 150, 20, 1062356, 0x1, false, false); // Price Per Rental
        if self.price > 0 {
            b.add_label(230, 115, 0x64, &self.price.to_string());
        } else {
            b.add_label(230, 115, 0x64, "FREE");
        }

        b.add_image_tiled(50, 160, 250, 2, 0x23BF);

        if ty == GumpType::Offer {
            b.add_button(67, 180, 0x482, 0x483, 2);
            b.add_html_localized_color(100, 180, 270, 20, 1049011, 0x28, false, false); // I accept!

            b.add_button(67, 210, 0x47F, 0x480, 0);
            b.add_html_localized_color(100, 210, 270, 20, 1049012, 0x28, false, false); // No thanks, I decline.
        } else {
            b.add_image(49, 170, 0x61, 0);
            b.add_html_localized_color(60, 170, 250, 20, 1062355, 0x1, false, false); // Renew On Expiration?

            if matches!(
                ty,
                GumpType::LockedContract | GumpType::UnlockedContract | GumpType::VendorLandlord
            ) {
                b.add_button(30, 192, 0x15E1, 0x15E5, 3);
            }

            b.add_html_localized_color(85, 190, 250, 20, 1062359, 0x1, false, false); // Landlord:
            b.add_html_localized_color(
                230,
                190,
                270,
                20,
                if self.landlord_renew {
                    1049717
                } else {
                    1049718
                },
                0x1,
                false,
                false,
            ); // YES / NO

            if ty == GumpType::VendorRenter {
                b.add_button(30, 212, 0x15E1, 0x15E5, 4);
            }

            b.add_html_localized_color(85, 210, 250, 20, 1062360, 0x1, false, false); // Renter:
            b.add_html_localized_color(
                230,
                210,
                270,
                20,
                if self.renter_renew { 1049717 } else { 1049718 },
                0x1,
                false,
                false,
            ); // YES / NO

            if self.renew {
                b.add_image(49, 233, 0x939, 0);
                b.add_html_localized_color(70, 230, 250, 20, 1062482, 0x1, false, false); // Contract WILL renew
            } else {
                b.add_image(49, 233, 0x938, 0);
                b.add_html_localized_color(70, 230, 250, 20, 1062483, 0x1, false, false); // Contract WILL NOT renew
            }
        }

        b.add_image_tiled(30, 283, 257, 30, 0x5D);
        b.add_image(285, 283, 0x5E, 0);
        b.add_image(20, 288, 0x232C, 0);

        if ty == GumpType::LockedContract {
            b.add_button(67, 295, 0x15E1, 0x15E5, 5);
            b.add_html_localized_color(85, 294, 270, 20, 1062358, 0x28, false, false); // Offer Contract To Someone
        } else if matches!(ty, GumpType::VendorLandlord | GumpType::VendorRenter) {
            if ty == GumpType::VendorLandlord {
                b.add_button(30, 250, 0x15E1, 0x15E1, 6);
            }

            b.add_html_localized_color(85, 250, 250, 20, 1062499, 0x1, false, false); // Renewal Price
            b.add_label(230, 250, 0x64, &self.renewal_price.to_string());

            b.add_html_localized_color(60, 294, 270, 20, 1062369, 0x1, false, false); // Renter:
            b.add_label(120, 293, 0x64, self.renter.as_deref().unwrap_or(""));
        }

        if contract {
            b.add_page(2);

            for (i, d) in RENTAL_DURATIONS.iter().enumerate() {
                let i = i as i32;
                b.add_button_type(
                    30,
                    76 + i * 20,
                    0x15E1,
                    0x15E5,
                    0x10 | i,
                    GumpButtonType::Reply,
                    1,
                );
                b.add_html_localized_color(50, 75 + i * 20, 150, 20, d.name, 0x1, false, false);
            }
        }
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        let Some(from) = mobile_of(w, sender) else {
            return;
        };
        let Some(o) = rental_ops(w) else { return };

        if !self.is_valid_response(w, &*o, from) {
            return;
        }

        if (info.button_id & 0x10) != 0 {
            // Contract duration
            let index = (info.button_id & 0xF) as usize;

            if index < RENTAL_DURATIONS.len() {
                self.set_contract_duration(w, &*o, from, index);
            }

            return;
        }

        match info.button_id {
            1 => self.set_price_per_rental(w, from),
            2 => self.accept_offer(w, &*o, from),
            3 => self.landlord_renew_on_expiration(w, &*o, from),
            4 => self.renter_renew_on_expiration(w, &*o, from),
            5 => self.offer_contract(w, &*o, from),
            6 => self.set_renewal_price(w, from),
            _ => self.cancel(w, from),
        }
    }
}

impl<K: RentalKind> RentalGump<K> {
    fn is_valid_response(&self, w: &mut World, o: &dyn RentalOps, from: MobileId) -> bool {
        match (self.ty, self.subject) {
            (GumpType::UnlockedContract | GumpType::LockedContract, Subject::Contract(c)) => {
                o.contract_usable_by(w, c, from, [true, true, true, true])
            }
            (GumpType::Offer, Subject::Contract(c)) => {
                let Some(landlord) = self.offer_landlord else {
                    return false;
                };
                o.contract_usable_by(w, c, landlord, [true, false, false, false])
                    && muo_server::mobile::part2::stats::check_alive(w, from, true)
                    && o.contract_offeree(w, c) == Some(from)
            }
            (GumpType::VendorRenter, Subject::Vendor(v)) => o.vendor_can_interact(w, v, from, true),
            (GumpType::VendorLandlord, Subject::Vendor(v)) => {
                o.vendor_can_interact(w, v, from, false) && o.vendor_is_landlord(w, v, from)
            }
            _ => false,
        }
    }

    fn is_contract(&self) -> bool {
        matches!(
            self.ty,
            GumpType::UnlockedContract | GumpType::LockedContract
        )
    }

    fn set_contract_duration(
        &self,
        w: &mut World,
        o: &dyn RentalOps,
        from: MobileId,
        duration: usize,
    ) {
        if let (true, Subject::Contract(c)) = (self.is_contract(), self.subject) {
            o.contract_set_duration(w, c, duration);
            contract_display_to(w, from, c);
        }
    }

    fn set_price_per_rental(&self, w: &mut World, from: MobileId) {
        if let (true, Subject::Contract(c)) = (self.is_contract(), self.subject) {
            // Please enter the amount of gold that should be charged for this contract (ESC to cancel):
            muo_content::mx::msg(w, from, 1062365);
            begin_price_prompt(w, from, c);
        }
    }

    fn landlord_renew_on_expiration(&self, w: &mut World, o: &dyn RentalOps, from: MobileId) {
        match (self.ty, self.subject) {
            (GumpType::UnlockedContract | GumpType::LockedContract, Subject::Contract(c)) => {
                o.contract_toggle_landlord_renew(w, c);
                contract_display_to(w, from, c);
            }
            (GumpType::VendorLandlord, Subject::Vendor(v)) => {
                o.vendor_toggle_landlord_renew(w, v);
                landlord_display_to(w, from, v);
            }
            _ => {}
        }
    }

    fn renter_renew_on_expiration(&self, w: &mut World, o: &dyn RentalOps, from: MobileId) {
        if let (GumpType::VendorRenter, Subject::Vendor(v)) = (self.ty, self.subject) {
            o.vendor_toggle_renter_renew(w, v);
            renter_display_to(w, from, v);
        }
    }

    fn offer_contract(&self, w: &mut World, o: &dyn RentalOps, from: MobileId) {
        if let (true, Subject::Contract(c)) = (self.is_contract(), self.subject)
            && o.contract_is_landlord(w, c, from)
        {
            muo_content::mx::msg(w, from, 1062371); // Please target the person you wish to offer this contract to.
            begin_offer_target(w, from, c);
        }
    }

    fn set_renewal_price(&self, w: &mut World, from: MobileId) {
        if let (GumpType::VendorLandlord, Subject::Vendor(v)) = (self.ty, self.subject) {
            muo_content::mx::msg(w, from, 1062500); // Enter contract renewal price:
            begin_renewal_prompt(w, from, v);
        }
    }

    fn accept_offer(&self, w: &mut World, o: &dyn RentalOps, from: MobileId) {
        let (GumpType::Offer, Subject::Contract(contract), Some(landlord)) =
            (self.ty, self.subject, self.offer_landlord)
        else {
            return;
        };
        o.contract_set_offeree(w, contract, None);

        if !o.contract_can_fit(w, contract) {
            muo_content::mx::msg(w, landlord, 1062486); // A vendor cannot exist at that location.  Please try again.
            return;
        }

        if !o.contract_has_house(w, contract) {
            return;
        }

        // `_contract.Price` is read now, not the price the gump was drawn with (the landlord may have changed it).
        let price = o
            .contract_view(w, contract, landlord)
            .map_or(self.price, |v| v.price);
        let gold_to_give;

        if price > 0 {
            if muo_content::vendor::banker::withdraw(w, from, price) {
                // ~1_AMOUNT~ gold has been withdrawn from your bank box.
                muo_content::mx::msg_args(w, from, 1060398, &price.to_string());

                let deposited_gold =
                    muo_content::vendor::banker::deposit_up_to(w, landlord, price, true, 0);
                gold_to_give = price - deposited_gold;

                if deposited_gold > 0 {
                    // ~1_AMOUNT~ gold has been deposited into your bank box.
                    muo_content::mx::msg_args(w, landlord, 1060397, &price.to_string());
                }

                if gold_to_give > 0 {
                    muo_content::mx::msg(w, landlord, 500390); // Your bank box is full.
                }
            } else {
                // You do not have enough gold in your bank account to cover the cost of the contract.
                muo_content::mx::msg(w, from, 1062378);
                // ~1_NAME~ has declined your vendor rental offer.
                let name = name_of(w, from);
                muo_content::mx::msg_args(w, landlord, 1062374, &name);

                return;
            }
        } else {
            gold_to_give = 0;
        }

        if o.contract_accept(w, contract, from, gold_to_give).is_err() {
            return;
        }

        // You have accepted the offer and now own a vendor in this house.  Rental contract options and details may be
        // viewed on this vendor via the 'Contract Options' context menu.
        muo_content::mx::msg(w, from, 1062377);

        // ~1_NAME~ has accepted your vendor rental offer.  Rental contract details and options may be viewed on this
        // vendor via the 'Contract Options' context menu.
        let name = name_of(w, from);
        muo_content::mx::msg_args(w, landlord, 1062376, &name);
    }

    fn cancel(&self, w: &mut World, from: MobileId) {
        if let (GumpType::Offer, Subject::Contract(contract), Some(landlord)) =
            (self.ty, self.subject, self.offer_landlord)
        {
            if let Some(o) = rental_ops(w) {
                o.contract_set_offeree(w, contract, None);
            }

            muo_content::mx::msg(w, from, 1062375); // You decline the offer for a vendor space rental.
            let name = name_of(w, from);
            muo_content::mx::msg_args(w, landlord, 1062374, &name); // ~1_NAME~ has declined your vendor rental offer.
        }
    }
}

/// `int.TryParse(text.Trim())`, then the clamping both prompts do: `Err` is "Invalid entry".
fn parse_price(text: &str) -> (i32, bool) {
    // `int.TryParse(span.Trim())`: decimal only (no 0x prefix), an optional sign, white space around.
    let price = text
        .trim_matches(|c| matches!(c, '\u{9}'..='\u{D}' | ' '))
        .parse::<i32>()
        .unwrap_or(-1);
    if price < 0 {
        (0, true)
    } else if price > 5_000_000 {
        (5_000_000, false)
    } else {
        (price, false)
    }
}

/// `from.Prompt = new PricePerRentalPrompt(contract)`.
fn begin_price_prompt(w: &mut World, from: MobileId, contract: ItemId) {
    let prompt = Prompt {
        on_response: Some(Rc::new(move |w: &mut World, from: MobileId, text: &str| {
            let Some(o) = rental_ops(w) else { return };
            if !o.contract_usable_by(w, contract, from, [true, true, true, true]) {
                return;
            }

            let (price, invalid) = parse_price(text);
            if invalid {
                muo_content::mx::msg(w, from, 1062485); // Invalid entry.  Rental fee set to 0.
            }
            o.contract_set_price(w, contract, price);

            contract_display_to(w, from, contract);
        })),
        on_cancel: Some(Rc::new(move |w: &mut World, from: MobileId| {
            let Some(o) = rental_ops(w) else { return };
            if o.contract_usable_by(w, contract, from, [true, true, true, true]) {
                contract_display_to(w, from, contract);
            }
        })),
    };
    let id = add_prompt(w, prompt);
    muo_server::mobile::part1::props_b::set_prompt(w, from, Some(id));
}

/// `from.Prompt = new ContractRenewalPricePrompt(vendor)`.
fn begin_renewal_prompt(w: &mut World, from: MobileId, vendor: MobileId) {
    let prompt = Prompt {
        on_response: Some(Rc::new(move |w: &mut World, from: MobileId, text: &str| {
            let Some(o) = rental_ops(w) else { return };
            if !o.vendor_can_interact(w, vendor, from, false)
                || !o.vendor_is_landlord(w, vendor, from)
            {
                return;
            }

            let (price, invalid) = parse_price(text);
            if invalid {
                muo_content::mx::msg(w, from, 1062485); // Invalid entry.  Rental fee set to 0.
            }
            o.vendor_set_renewal_price(w, vendor, price);

            landlord_display_to(w, from, vendor);
        })),
        on_cancel: Some(Rc::new(move |w: &mut World, from: MobileId| {
            let Some(o) = rental_ops(w) else { return };
            if o.vendor_can_interact(w, vendor, from, false)
                && o.vendor_is_landlord(w, vendor, from)
            {
                landlord_display_to(w, from, vendor);
            }
        })),
    };
    let id = add_prompt(w, prompt);
    muo_server::mobile::part1::props_b::set_prompt(w, from, Some(id));
}

/// `from.Target = new OfferContractTarget(contract)`.
fn begin_offer_target(w: &mut World, from: MobileId, contract: ItemId) {
    let mut t = Target::new(w, -1, false, TargetFlags::NONE);
    t.handlers.on_target = Some(Rc::new(
        move |w: &mut World, from: MobileId, targeted: Targeted| {
            let Some(o) = rental_ops(w) else { return };
            if !o.contract_usable_by(w, contract, from, [true, false, true, true]) {
                return;
            }

            let valid = match targeted {
                Targeted::Entity(EntityId::Mobile(mob)) if mob != from => {
                    w.mobile(mob).is_some_and(|m| m.player) && muo_content::mx::alive(w, mob)
                }
                _ => false,
            };
            let Targeted::Entity(EntityId::Mobile(mob)) = targeted else {
                muo_content::mx::msg(w, from, 1071984); // That is not a valid target for a rental contract!
                return;
            };
            if !valid {
                muo_content::mx::msg(w, from, 1071984); // That is not a valid target for a rental contract!
            } else if !muo_content::mx::in_range_point(w, mob, o.contract_location(w, contract), 5)
            {
                muo_content::mx::msg(w, from, 501853); // Target is too far away.
            } else {
                muo_content::mx::msg(w, from, 1062372); // Please wait while that person considers your offer.

                // ~1_NAME~ is offering you a vendor rental.   If you choose to accept this offer, you have 30 seconds to do so.
                let name = name_of(w, from);
                muo_content::mx::msg_args(w, mob, 1062373, &name);
                offer_display_to(w, mob, contract, from);

                o.contract_set_offeree(w, contract, Some(mob));
            }
        },
    ));
    t.handlers.on_target_cancel = Some(Rc::new(
        move |w: &mut World, from: MobileId, _kind: TargetCancelType| {
            muo_content::mx::msg(w, from, 1062380); // You decide against offering the contract to anyone.
        },
    ));
    let id = add_target(w, t);
    set_mobile_target(w, from, Some(id));
}

// ---- VendorRentalRefundGump ----

pub struct VendorRentalRefundGump {
    vendor: MobileId,
    landlord: MobileId,
    refund_amount: i32,
    vendor_name: String,
    shop_name: String,
}

/// `VendorRentalRefundGump.DisplayTo(to, vendor, landlord, refundAmount)`.
pub fn refund_display_to(
    w: &mut World,
    to: MobileId,
    vendor: MobileId,
    landlord: MobileId,
    refund_amount: i32,
) {
    let Some(o) = rental_ops(w) else { return };
    if !has_net_state(w, to) || muo_content::mx::deleted(w, vendor) {
        return;
    }
    let g = StaticGumpObj::new(
        50,
        50,
        VendorRentalRefundGump {
            vendor,
            landlord,
            refund_amount,
            vendor_name: o.vendor_name(w, vendor),
            shop_name: o.vendor_shop_name(w, vendor),
        },
    );
    send_singleton(w, to, g);
}

impl StaticGumpLogic for VendorRentalRefundGump {
    const TYPE_NAME: &'static str = "Server.Gumps.VendorRentalRefundGump";
    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut StaticGumpBuilder) {
        b.add_page(0);

        b.add_background(0, 0, 420, 320, 0x13BE);

        b.add_image_tiled(10, 10, 400, 300, 0xA40);
        b.add_alpha_region(10, 10, 400, 300);

        /* The landlord for this vendor is offering you a partial refund of your rental fee
         * in exchange for immediate termination of your rental contract. [..] */
        b.add_html_localized_color(10, 10, 400, 150, 1062501, 0x7FFF, false, true);

        b.add_html_localized_color(10, 180, 150, 20, 1062508, 0x7FFF, false, false); // Vendor Name:
        b.add_label_placeholder(160, 180, 0x480, "vendorName");

        b.add_html_localized_color(10, 200, 150, 20, 1062509, 0x7FFF, false, false); // Shop Name:
        b.add_label_placeholder(160, 200, 0x480, "shopName");

        b.add_html_localized_color(10, 220, 150, 20, 1062510, 0x7FFF, false, false); // Refund Amount:
        b.add_label_placeholder(160, 220, 0x480, "refundAmount");

        b.add_button(10, 268, 0xFA5, 0xFA7, 1);
        b.add_html_localized_color(45, 268, 350, 20, 1062511, 0x7FFF, false, false); // Agree, and <strong>dismiss vendor</strong>

        b.add_button(10, 288, 0xFA5, 0xFA7, 0);
        b.add_html_localized_color(45, 288, 350, 20, 1062512, 0x7FFF, false, false); // No, I want to <strong>keep my vendor</strong>
    }

    fn build_strings(&self, strings: &mut GumpStrings) {
        strings.set_string_slot("vendorName", &self.vendor_name);
        strings.set_string_slot("shopName", &self.shop_name);
        strings.set_string_slot("refundAmount", &self.refund_amount.to_string());
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        let Some(from) = mobile_of(w, sender) else {
            return;
        };
        let Some(o) = rental_ops(w) else { return };
        let (vendor, landlord) = (self.vendor, self.landlord);

        if !o.vendor_can_interact(w, vendor, from, true)
            || !o.vendor_can_interact(w, vendor, landlord, false)
            || !o.vendor_is_landlord(w, vendor, landlord)
        {
            return;
        }

        if info.button_id != 1 {
            muo_content::mx::msg(w, landlord, 1062513); // The renter declined your offer.
            return;
        }

        if !muo_content::vendor::banker::withdraw(w, landlord, self.refund_amount) {
            muo_content::mx::msg(w, landlord, 1062507); // You do not have that much money in your bank account.
            return;
        }

        // ~1_AMOUNT~ gold has been withdrawn from your bank box.
        muo_content::mx::msg_args(w, landlord, 1060398, &self.refund_amount.to_string());

        let deposited_gold =
            muo_content::vendor::banker::deposit_up_to(w, from, self.refund_amount, true, 0);

        if deposited_gold > 0 {
            // ~1_AMOUNT~ gold has been deposited into your bank box.
            muo_content::mx::msg_args(w, from, 1060397, &deposited_gold.to_string());
        }

        o.vendor_add_hold_gold(w, vendor, self.refund_amount - deposited_gold);

        o.vendor_destroy(w, vendor);

        muo_content::mx::msg(w, from, 1071990); // Remember to claim your vendor's belongings from the house sign!
    }
}

#[cfg(test)]
mod tests {
    use super::parse_price;

    #[test]
    fn price_is_int_try_parse_then_clamped() {
        // `int.TryParse("0x10")` fails -> -1 -> invalid, price 0 (Utility.ToInt32 would have read 16).
        assert_eq!(parse_price("0x10"), (0, true));
        assert_eq!(parse_price(" 25 "), (25, false));
        assert_eq!(parse_price("+7"), (7, false));
        assert_eq!(parse_price("-1"), (0, true));
        assert_eq!(parse_price("5000001"), (5_000_000, false));
        assert_eq!(parse_price("99999999999"), (0, true));
        assert_eq!(parse_price(""), (0, true));
    }
}
