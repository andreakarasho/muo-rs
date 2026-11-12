//! Vendor buyback tests (the pure part) and the buy-info arithmetic.

use muo_primitives::{DateTime, TimeSpan};

use super::buy::BuyInfo;
use super::get_next_buyback_purge;

fn base() -> DateTime {
    DateTime::from_ymd_hms(2026, 1, 1, 0, 0, 0).expect("valid date")
}

#[test]
fn next_buyback_purge_is_the_next_restock_boundary_after_now() {
    for (since, expected) in [(0.25, 1.0), (5.0, 6.0), (5.5, 6.0)] {
        let last = base();
        let now = last + TimeSpan::from_hours_f64(since);
        let next = get_next_buyback_purge(last, TimeSpan::from_hours(1), now);
        assert_eq!(next, last + TimeSpan::from_hours_f64(expected), "{since}");
    }
}

#[test]
fn next_buyback_purge_future_anchor_is_clamped_to_now() {
    let now = base();
    let last = now + TimeSpan::from_hours(4);
    assert_eq!(
        get_next_buyback_purge(last, TimeSpan::from_hours(1), now),
        now + TimeSpan::from_hours(1)
    );
}

#[test]
fn buy_info_price_scales_and_rounds() {
    let mut b = BuyInfo::new(None, "Server.Items.Garlic", 3, 20, 0xF84, 0);
    assert_eq!(b.price(), 3);
    b.price_scalar = 105;
    assert_eq!(b.price(), (3 * 105 + 50) / 100);
    assert_eq!(b.name, (1020000 + 0xF84).to_string());
}

#[test]
fn restock_grows_a_sold_out_entry_and_halves_an_untouched_one() {
    let mut b = BuyInfo::new(None, "Server.Items.Garlic", 3, 20, 0xF84, 0);
    b.set_amount(0);
    b.on_restock();
    assert_eq!((b.max_amount, b.amount), (40, 40));
    // Less than half of the stock was bought: the maximum is halved.
    b.set_amount(30);
    b.on_restock();
    assert_eq!((b.max_amount, b.amount), (20, 20));
}

#[test]
fn banker_deposits_withdraws_and_counts_gold_and_checks() {
    use super::banker::{deposit, get_balance, withdraw};
    use muo_server::mobile::MOBILE_BASE;
    use muo_server::world::World;

    let mut w = World::new();
    w.resources
        .get_mut::<muo_server::mobile::part1::props_b::PropsBStatics>()
        .create_bank_box =
        Some(|w, owner| Some(muo_server::item::containers::new_bank_box(w, Some(owner))));
    let m = w.new_mobile(&MOBILE_BASE);
    assert!(muo_server::mobile::part1::props_b::bank_box(&mut w, m).is_some());

    assert!(deposit(&mut w, m, 1234, true));
    assert_eq!(get_balance(&mut w, m), 1234);
    assert!(withdraw(&mut w, m, 200));
    assert_eq!(get_balance(&mut w, m), 1034);
    // 7000 is a check; the piles of gold stay as they are.
    assert!(deposit(&mut w, m, 7000, true));
    assert_eq!(get_balance(&mut w, m), 8034);
    assert!(withdraw(&mut w, m, 7500));
    assert_eq!(get_balance(&mut w, m), 534);
    assert!(
        !withdraw(&mut w, m, 535),
        "more than the balance is refused and nothing is taken"
    );
    assert_eq!(get_balance(&mut w, m), 534);
}

// ---- BaseCreatureSerializationTests.GeneratedVendorChain_RoundTrips ----

static TEST_VENDOR: muo_server::mobile::MobileType = muo_server::mobile::MobileType {
    full_name: "Server.Tests.TestVendor",
    parent: Some(&super::BASE_VENDOR),
    ..super::BASE_VENDOR
};

// BaseVendor is generated on top of the generated BaseCreature; the chain must write and read both sections in
// order with exact consumption.
#[test]
fn generated_vendor_chain_round_trips() {
    use crate::creature::enums::{AIType, FightMode};
    use muo_io::{SaveReader, SaveWriter};

    let mut w = muo_server::map::test_support::world();
    crate::ai::configure_for_tests(&mut w);
    let vendor = super::new_vendor(&mut w, &TEST_VENDOR, Some("the stub"));
    crate::creature::control::set_home(&mut w, vendor, muo_primitives::Point3D::new(1500, 1600, 0));
    crate::creature::data::cd_mut(&mut w, vendor)
        .unwrap()
        .range_home = 2;

    let mut out = SaveWriter::new(true);
    (w.mobile(vendor).unwrap().ty.serialize)(&w, vendor, &mut out);
    let buffer = out.into_vec();

    let copy = w.new_mobile(&TEST_VENDOR);
    let mut reader = SaveReader::new(&buffer);
    let ty = w.mobile(copy).unwrap().ty;
    (ty.deserialize)(&mut w, copy, &mut reader).unwrap();

    assert_eq!(reader.remaining(), 0);
    let d = crate::creature::data::cd(&w, copy).unwrap();
    assert_eq!(d.current_ai, AIType::AiVendor);
    assert_eq!(d.fight_mode, FightMode::None);
    assert_eq!(d.home, muo_primitives::Point3D::new(1500, 1600, 0));
    assert_eq!(d.range_home, 2);
}

#[test]
fn a_disabled_vendor_feature_tells_the_player_in_red() {
    // Expected: from.SendMessage(0x22, "Vendor purchases are temporarily disabled.")
    use muo_server::net_state::{NetOut, NetStateData, NetStateId};
    let mut w = muo_server::map::test_support::world();
    crate::ai::configure_for_tests(&mut w);
    w.resources.get_mut::<super::VendorHooks>().feature_enabled = Some(|_, _| false);
    let vendor = super::new_vendor(&mut w, &super::BASE_VENDOR, Some("the stub"));
    let from = w.new_mobile(&muo_server::mobile::MOBILE_BASE);
    let ns = NetStateId(1);
    w.net_states.insert(ns, NetStateData::default());
    w.mobile_mut(from).unwrap().net_state = Some(ns);
    super::trade::vendor_buy(&mut w, vendor, from);
    let Some(NetOut::Send(_, b)) = w.net_out.pop() else {
        panic!("nothing sent")
    };
    assert_eq!(&b[0..1], &[0xAE]);
    assert!(b.windows(2).any(|p| p == [0x00, 0x22]), "hue 0x22 in {b:?}");
}
