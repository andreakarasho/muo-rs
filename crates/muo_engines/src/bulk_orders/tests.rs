//! Bulk orders: the deed classes construct and round-trip their save bytes in the existing save layout, the
//! reward tables hold their expected values, and a book entry survives its own serializer.

use muo_io::{SaveReader, SaveWriter};
use muo_primitives::{DateTime, Serial};
use muo_server::World;
use muo_server::ids::ItemId;
use muo_server::item::{Item, ItemType};

use super::*;

fn world() -> World {
    let mut w = World::new();
    muo_content::register(&mut w.types);
    muo_items::register(&mut w.types);
    w
}

fn save(w: &World, id: ItemId) -> Vec<u8> {
    let ty = w.item(id).unwrap().ty;
    let mut out = SaveWriter::new(true);
    (ty.serialize)(w, id, &mut out);
    out.into_vec()
}

fn reload(w: &mut World, t: &'static ItemType, bytes: &[u8]) -> ItemId {
    let id = w.add_item(Item::new_for_load(Serial::new(0x4000_0300), t));
    let mut r = SaveReader::new(bytes);
    (t.deserialize)(w, id, &mut r).unwrap_or_else(|e| panic!("{}: {e}", t.full_name));
    assert_eq!(r.remaining(), 0, "{}", t.full_name);
    id
}

#[test]
fn small_deed_round_trips_with_the_generated_layout() {
    let mut w = world();
    let id = new_small_with(
        &mut w,
        &SMALL_SMITH_BOD,
        0x44E,
        3,
        15,
        Some("Server.Items.Dagger"),
        1025128,
        0xF52,
        true,
        mat::BRONZE,
    );
    let bytes = save(&w, id);
    // ... Item part, then BaseBOD (encoded version 1, AmountMax, RequireExceptional, Material),
    // SmallBOD (int version 0, AmountCur, Type hash, Number, Graphic) and SmallSmithBOD (int version 0).
    let tail = &bytes[bytes.len() - (1 + 4 + 1 + 1 + 4 + 4 + 1 + 8 + 4 + 4 + 4)..];
    let mut want = vec![1u8];
    want.extend(15i32.to_le_bytes());
    want.push(1);
    want.push(mat::BRONZE as u8); // WriteEnum of an int enum is an encoded int
    want.extend(0i32.to_le_bytes());
    want.extend(3i32.to_le_bytes());
    want.push(2);
    want.extend(muo_primitives::hash::compute_hash64("Server.Items.Dagger").to_le_bytes());
    want.extend(1025128i32.to_le_bytes());
    want.extend(0xF52i32.to_le_bytes());
    want.extend(0i32.to_le_bytes());
    assert_eq!(tail, &want[..]);
    let again = reload(&mut w, &SMALL_SMITH_BOD, &bytes);
    assert_eq!(save(&w, again), bytes);
    let d = data(&w, again).unwrap();
    assert_eq!(
        (d.amount_cur, d.amount_max, d.material, d.type_name),
        (3, 15, mat::BRONZE, Some("Server.Items.Dagger"))
    );
}

#[test]
fn large_deed_round_trips() {
    let mut w = world();
    let entries = vec![
        LargeBulkEntry {
            amount: 5,
            details: SmallBulkEntry {
                type_name: Some("Server.Items.Dagger"),
                number: 1025128,
                graphic: 0xF52,
            },
        },
        LargeBulkEntry {
            amount: 0,
            details: SmallBulkEntry {
                type_name: None,
                number: 7,
                graphic: 8,
            },
        },
    ];
    let id = new_large_with(
        &mut w,
        &LARGE_TAILOR_BOD,
        0x483,
        10,
        false,
        mat::SPINED,
        entries,
    );
    let bytes = save(&w, id);
    let again = reload(&mut w, &LARGE_TAILOR_BOD, &bytes);
    assert_eq!(save(&w, again), bytes);
    assert_eq!(data(&w, again).unwrap().entries.len(), 2);
    assert!(!complete(&w, again));
}

#[test]
fn every_class_constructs() {
    for t in [
        &SMALL_SMITH_BOD,
        &SMALL_TAILOR_BOD,
        &LARGE_SMITH_BOD,
        &LARGE_TAILOR_BOD,
        &book::BULK_ORDER_BOOK,
    ] {
        let mut w = world();
        let id = (t.construct.unwrap())(&mut w);
        assert_eq!(w.item(id).unwrap().ty.full_name, t.full_name);
        let bytes = save(&w, id);
        let again = reload(&mut w, t, &bytes);
        assert_eq!(save(&w, again), bytes, "{}", t.full_name);
    }
}

#[test]
fn reward_points_and_groups_follow_the_tables() {
    let mut w = world();
    // 20 exceptional bronze small deed: 50 + 200 + (200 + 50 * 3)
    let id = new_small_with(
        &mut w,
        &SMALL_SMITH_BOD,
        0x44E,
        0,
        20,
        Some("Server.Items.Dagger"),
        1,
        1,
        true,
        mat::BRONZE,
    );
    assert_eq!(rewards::compute_points(&w, id), 50 + 200 + 350);
    assert_eq!(rewards::compute_fame(&w, id), 12 * 12);
    // tailor: 10 + 100 + spined 50
    let t = new_small_with(
        &mut w,
        &SMALL_TAILOR_BOD,
        0x483,
        0,
        10,
        Some("Server.Items.Shirt"),
        1,
        1,
        true,
        mat::SPINED,
    );
    assert_eq!(rewards::compute_points(&w, t), 160);
    let g = rewards::compute_rewards(&mut w, t, true);
    assert_eq!(g.len(), 2); // the 150 group: cloth 3, sandals
}

#[test]
fn book_entry_round_trips_and_reconstructs() {
    let mut w = world();
    let id = new_small_with(
        &mut w,
        &SMALL_SMITH_BOD,
        0x44E,
        2,
        10,
        Some("Server.Items.Dagger"),
        1025128,
        0xF52,
        false,
        mat::NONE,
    );
    let s = book::add_from_deed(&mut w, id);
    let e = book::entry(&w, s).unwrap().clone();
    let mut out = SaveWriter::new(true);
    book::write_entry_for_test(&e, &mut out);
    let mut back = book::BobEntry {
        created: DateTime::MIN,
        ..e.clone()
    };
    back.amount_max = 0;
    let mut r = SaveReader::new(out.as_slice());
    book::read_entry_for_test(&w, &mut r, &mut back).unwrap();
    assert_eq!(r.remaining(), 0);
    assert_eq!(back.amount_max, 10);
    let item = book::reconstruct(&mut w, &back).unwrap();
    assert_eq!(data(&w, item).unwrap().amount_cur, 2);
}

#[test]
fn every_reward_in_the_tables_constructs() {
    // AncientSmithyHammer has no parameterless constructor: the reward must pass its bonus.
    let mut w = world();
    muo_server::type_registry::invoke_phase(&mut w, muo_server::type_registry::Phase::Configure);
    for g in rewards::SMITH_GROUPS.iter().chain(rewards::TAILOR_GROUPS) {
        for r in g.items {
            let id = rewards::construct(&mut w, r);
            assert!(
                id.is_some(),
                "{:?} type {} did not construct",
                r.reward,
                r.ty
            );
        }
    }
    let r = rewards::RewardItem {
        weight: 1,
        reward: rewards::Reward::AncientHammer,
        ty: 30,
    };
    let id = rewards::construct(&mut w, &r).unwrap();
    assert_eq!(
        w.item(id).unwrap().ty.full_name,
        "Server.Items.AncientSmithyHammer"
    );
}
