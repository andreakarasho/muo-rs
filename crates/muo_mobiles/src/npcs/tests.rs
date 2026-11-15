//! Every class of Mobiles/{Vendors,Healers,..}: names are unique, it constructs and its save round-trips; golden
//! bytes per family.

use std::collections::HashSet;

use muo_content::creature::speeds::{NpcSpeeds, SpeedClassEntry};
use muo_data::expansion::Expansion;
use muo_io::{SaveReader, SaveWriter};
use muo_server::World;
use muo_server::ids::MobileId;

fn world(era: Option<Expansion>) -> World {
    let mut w = crate::test_world();
    w.core.expansion = era;
    w.resources
        .get_mut::<NpcSpeeds>()
        .register(SpeedClassEntry {
            level: "Medium".into(),
            active: 0.2,
            passive: 0.4,
            active_move: 0.0,
            passive_move: 0.0,
            types: vec![],
        });
    w
}

fn save(w: &World, id: MobileId) -> Vec<u8> {
    let ty = w.mobile(id).unwrap().ty;
    let mut out = SaveWriter::new(true);
    (ty.serialize)(w, id, &mut out);
    out.into_vec()
}

#[test]
fn npc_names_are_unique_and_registered() {
    let mut names = HashSet::new();
    for t in super::all() {
        assert!(names.insert(t.full_name), "{} twice", t.full_name);
        assert!(t.full_name.starts_with("Server.Mobiles."));
    }
    let mut types = muo_server::type_registry::TypeRegistry::default();
    crate::register(&mut types);
    for t in super::all() {
        assert!(
            types.find_type_by_full_name(t.full_name, false).is_some(),
            "{} is registered",
            t.full_name
        );
    }
}

#[test]
fn every_npc_constructs_and_round_trips() {
    for era in [None, Some(Expansion::AOS), Some(Expansion::ML)] {
        for ty in super::all() {
            let mut w = world(era);
            let id = (ty.construct.or(ty.make).unwrap())(&mut w);
            assert_eq!(w.mobile(id).unwrap().ty.full_name, ty.full_name);
            assert!(
                muo_content::creature::is_creature(&w, id) || super::guards::is_guard(&w, id),
                "{}",
                ty.full_name
            );
            let bytes = save(&w, id);

            let mut w2 = world(era);
            let other = w2.new_mobile(ty);
            let mut r = SaveReader::new(&bytes);
            (ty.deserialize)(&mut w2, other, &mut r)
                .unwrap_or_else(|e| panic!("{}: {e}", ty.full_name));
            assert_eq!(r.remaining(), 0, "{} is read to its end", ty.full_name);
            // Neira wears a VirtualMountItem that the fresh world does not have, so its reference does not come back.
            if ty.full_name != "Server.Mobiles.Neira" {
                assert_eq!(bytes.len(), save(&w2, other).len(), "{}", ty.full_name);
            }
        }
    }
}

#[test]
fn a_vendor_saves_its_versions_after_the_creature_part() {
    // Alchemist: [SerializationGenerator(0, false)] over BaseVendor (version 2): two ints after BaseCreature.
    let mut w = world(None);
    let id = (super::vendors::npc::ALCHEMIST.construct.unwrap())(&mut w);
    let bytes = save(&w, id);
    let mut base = SaveWriter::new(true);
    (muo_content::creature::BASE_CREATURE.serialize)(&w, id, &mut base);
    let mut want = base.into_vec();
    want.extend([2, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(bytes, want);
}

#[test]
fn a_vendor_loads_its_sb_infos_and_prices() {
    let mut w = world(Some(Expansion::AOS));
    let id = (super::vendors::npc::ALCHEMIST.construct.unwrap())(&mut w);
    let d = muo_content::vendor::data(&w, id).unwrap();
    assert!(!d.buy_info.is_empty());
    assert_eq!(d.sell_info.len(), 1);
    let garlic = d
        .buy_info
        .iter()
        .find(|b| b.type_name.rsplit('.').next() == Some("Garlic")) // a test world keeps short names
        .unwrap();
    assert_eq!((garlic.price(), garlic.amount), (3, 20));
}

#[test]
fn a_healer_saves_its_price_and_reads_older_layouts() {
    let mut w = world(None);
    let id = (super::healers::PRICED_HEALER.construct.unwrap())(&mut w);
    assert_eq!(super::healers::data(&w, id).unwrap().price, 5000);
    let bytes = save(&w, id);
    // BaseHealer: version 3 then the price, then PricedHealer's version 1.
    let tail = &bytes[bytes.len() - 12..];
    assert_eq!(tail, &[3, 0, 0, 0, 0x88, 0x13, 0, 0, 1, 0, 0, 0]);
}

#[test]
fn a_town_crier_constructs_and_round_trips() {
    let mut w = world(None);
    let ty = &super::townfolk::crier::TOWN_CRIER;
    let id = (ty.construct.or(ty.make).unwrap())(&mut w);
    let bytes = save(&w, id);
    let mut w2 = world(None);
    let other = w2.new_mobile(ty);
    let mut r = SaveReader::new(&bytes);
    (ty.deserialize)(&mut w2, other, &mut r).unwrap();
    assert_eq!(r.remaining(), 0);
    assert_eq!(bytes.len(), save(&w2, other).len());
}

#[test]
fn a_champion_saves_its_creature_part_and_version_zero() {
    let mut w = world(None);
    let id = (super::special::champions::BARRACOON.construct.unwrap())(&mut w);
    let bytes = save(&w, id);
    let mut base = SaveWriter::new(true);
    (muo_content::creature::BASE_CREATURE.serialize)(&w, id, &mut base);
    let mut want = base.into_vec();
    // BaseChampion's version, then Barracoon's own.
    want.extend([0, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(bytes, want);
}

#[test]
fn an_escortable_saves_version_three_and_its_destination_fields() {
    let mut w = world(None);
    let id = (super::townfolk::classes::MERCHANT.construct.unwrap())(&mut w);
    let bytes = save(&w, id);
    // BaseEscortable: version 3, null destination, no delete timer (MinValue ticks), no quest type, two empty texts.
    let mut base = SaveWriter::new(true);
    (muo_content::creature::BASE_CREATURE.serialize)(&w, id, &mut base);
    let mut want = base.into_vec();
    want.extend([3, 0, 0, 0]);
    let tail = &bytes[want.len()..];
    // string null (1 byte), 8 bytes of time, type flag 0, then two text definitions, then the subclass's version int.
    assert_eq!(tail[0], 0, "a null destination string");
    assert_eq!(&bytes[..want.len()], &want[..]);
    assert_eq!(&bytes[bytes.len() - 4..], &[0, 0, 0, 0]);
}

#[test]
fn a_player_vendor_and_a_rented_vendor_round_trip() {
    use super::vendors::player_vendor::{self as pv, PLAYER_VENDOR};
    use super::vendors::rented::{self, RENTED_VENDOR};
    let mut w = world(Some(Expansion::AOS));
    let v = pv::make(&mut w, &PLAYER_VENDOR, None, None);
    // The new vendor system starts with 4 gold held and no bank account.
    assert_eq!((pv::hold_gold(&w, v), pv::bank_account(&w, v)), (4, 0));
    assert_eq!(pv::shop_name(&w, v), "Shop Not Yet Named");
    let bytes = save(&w, v);
    let mut w2 = world(Some(Expansion::AOS));
    let other = w2.new_mobile(&PLAYER_VENDOR);
    let mut r = SaveReader::new(&bytes);
    (PLAYER_VENDOR.deserialize)(&mut w2, other, &mut r).unwrap();
    assert_eq!(r.remaining(), 0);
    assert_eq!(
        (pv::hold_gold(&w2, other), pv::shop_name(&w2, other)),
        (4, "Shop Not Yet Named".to_owned())
    );

    let rv = rented::make(&mut w, None, None, 1, 5000, true, 100);
    let bytes = save(&w, rv);
    let other = w2.new_mobile(&RENTED_VENDOR);
    let mut r = SaveReader::new(&bytes);
    (RENTED_VENDOR.deserialize)(&mut w2, other, &mut r).unwrap();
    assert_eq!(r.remaining(), 0);
    let d = rented::rdata(&w2, other).unwrap();
    assert_eq!(
        (
            d.rental_duration_id,
            d.rental_price,
            d.renewal_price,
            d.rental_gold,
            d.landlord_renew
        ),
        (1, 5000, 5000, 100, true)
    );
}

#[test]
fn a_vendor_item_formats_its_price() {
    use super::vendors::player_vendor::VendorItem;
    let vi = VendorItem {
        item: None,
        price: 1234567,
        description: String::new(),
        created: muo_primitives::DateTime::MIN,
        valid: true,
    };
    assert_eq!(vi.formatted_price(true), "1,234,567");
    assert_eq!(vi.formatted_price(false), "1234567");
    assert!(vi.is_for_sale() && !vi.is_for_free());
}

/// `GetAttackSound() => Utility.Random(0x2C7, 5)` draws from the generator.
#[test]
fn rikktor_attack_sound_draws_from_the_rng() {
    use super::special::champions::RIKKTOR;
    let mut w = world(None);
    let id = (RIKKTOR.construct.unwrap())(&mut w);
    let mut twin = w.rng.clone();
    for _ in 0..20 {
        let got = (RIKKTOR.get_attack_sound)(&mut w, id);
        assert_eq!(
            got,
            muo_primitives::utility::random_from(&mut twin, 0x2C7, 5)
        );
    }
}

/// `AddCustomContextEntries` asks `GetEscorter()` with its side effects: a lost escorter makes the escortable stay,
/// and once the abandon delay is over it is dropped (`SetControlMaster(null)`).
#[test]
fn the_escort_menu_runs_get_escorter_with_its_side_effects() {
    use muo_content::creature::control;
    use muo_content::creature::data::{cd, cd_mut};
    use muo_content::creature::enums::OrderType;
    let mut w = world(None);
    let id = (super::townfolk::classes::MERCHANT.construct.unwrap())(&mut w);
    let master = (super::townfolk::classes::MERCHANT.construct.unwrap())(&mut w);
    let from = muo_content::player_mobile::new_player_mobile(&mut w);
    cd_mut(&mut w, id).unwrap().controlled = true;
    control::set_control_master(&mut w, id, Some(master));
    w.mobile_mut(master).unwrap().deleted = true; // lost
    let mut list = Vec::new();
    (super::townfolk::BASE_ESCORTABLE_C.add_custom_context_entries)(&mut w, id, from, &mut list);
    assert_eq!(cd(&w, id).unwrap().control_order, OrderType::Stay);
    assert_eq!(control::control_master(&w, id), Some(master));
    w.now += muo_primitives::TimeSpan::from_mins(3);
    (super::townfolk::BASE_ESCORTABLE_C.add_custom_context_entries)(&mut w, id, from, &mut list);
    assert_eq!(
        control::control_master(&w, id),
        None,
        "the lost escorter was dropped"
    );
}

#[test]
fn a_guard_whose_idle_timer_ran_out_keeps_it_not_null() {
    // `Stop()` leaves the IdleTimer field set, so Focus = null on a spawned guard doesn't start a new one.
    let mut w = world(None);
    let id = super::guards::warrior_guard_with(&mut w, None);
    w.mobile_mut(id).unwrap().spawner = Some(muo_server::interfaces::SpawnerId(1));
    let h = muo_server::timer::start(&mut w, Default::default(), Default::default(), 1, |_| {});
    super::guards::data_mut_for_test(&mut w, id).idle_timer = h;
    super::guards::set_focus(&mut w, id, None);
    assert_eq!(super::guards::data(&w, id).unwrap().idle_timer, h);
}
