//! Golden save bytes (`Server.Mobiles.{BaseVendor, BaseChampion, BaseEscortable, PlayerVendor,
//! RentedVendor, PlayerBarkeeper, BarkeeperRumor}` and `Server.Items.VendorRentalContract`): each expected stream is
//! written out in field serialization order, with literal bytes for versions and serials.

use muo_data::expansion::Expansion;
use muo_io::{GenericWriter, SaveReader, SaveWriter};
use muo_server::World;
use muo_server::ids::MobileId;
use muo_server::mobile::MOBILE_BASE;

use super::vendors::barkeeper::{self, PLAYER_BARKEEPER, Rumor};
use super::vendors::player_vendor::{self as pv, PLAYER_VENDOR};
use super::vendors::rental_contract::{self, VENDOR_RENTAL_CONTRACT};
use super::vendors::rented::{self, RENTED_VENDOR};

fn world(era: Option<Expansion>) -> World {
    let mut w = World::new();
    w.core.expansion = era;
    w.resources
        .get_mut::<muo_content::creature::speeds::NpcSpeeds>()
        .register(muo_content::creature::speeds::SpeedClassEntry {
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

fn load(w: &mut World, ty: &'static muo_server::mobile::MobileType, bytes: &[u8]) -> MobileId {
    let id = w.new_mobile(ty);
    let mut r = SaveReader::new(bytes);
    (ty.deserialize)(w, id, &mut r).unwrap_or_else(|e| panic!("{}: {e}", ty.full_name));
    assert_eq!(r.remaining(), 0, "{} is read to its end", ty.full_name);
    id
}

fn creature_part(w: &World, id: MobileId) -> Vec<u8> {
    let mut base = SaveWriter::new(true);
    (muo_content::creature::BASE_CREATURE.serialize)(w, id, &mut base);
    base.into_vec()
}

fn string(s: &str) -> Vec<u8> {
    let mut o = SaveWriter::new(true);
    o.write_string(s);
    o.into_vec()
}

#[test]
fn golden_base_vendor_writes_version_two_and_reads_the_old_restock_pairs() {
    // BaseVendor: Write(2) then the subclass's Write(0).
    let mut w = world(None);
    let id = (super::vendors::npc::ALCHEMIST.construct.unwrap())(&mut w);
    let mut want = creature_part(&w, id);
    want.extend([2, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(save(&w, id), want);

    // Version 1: `while (ReadEncodedInt() > 0) ReadEncodedInt();` is skipped, then the subclass's version.
    let mut old = creature_part(&w, id);
    old.extend([1, 0, 0, 0]);
    old.extend([5, 9, 3, 1, 0]); // (5, 9), (3, 1), end
    old.extend([0, 0, 0, 0]);
    let mut w2 = world(None);
    let other = load(&mut w2, &super::vendors::npc::ALCHEMIST, &old);
    // Saved again, it is the version-2 layout.
    let all = save(&w2, other);
    assert_eq!(
        &all[creature_part(&w2, other).len()..],
        &[2, 0, 0, 0, 0, 0, 0, 0]
    );
}

#[test]
fn golden_base_champion_is_the_creature_then_two_versions() {
    let mut w = world(None);
    let id = (super::special::champions::BARRACOON.construct.unwrap())(&mut w);
    let mut want = creature_part(&w, id);
    want.extend([0, 0, 0, 0, 0, 0, 0, 0]); // BaseChampion's, then Barracoon's
    assert_eq!(save(&w, id), want);
    let mut w2 = world(None);
    let other = load(&mut w2, &super::special::champions::BARRACOON, &want);
    assert_eq!(save(&w2, other).len(), want.len());
}

#[test]
fn golden_base_escortable_version_three_layout() {
    let ty = &super::townfolk::classes::MERCHANT;
    let mut w = world(None);
    let id = (ty.construct.unwrap())(&mut w);

    // Fresh: null destination, MinValue anchored time, null type, two empty text definitions (encoded 3).
    let mut want = creature_part(&w, id);
    want.extend([3, 0, 0, 0]);
    let mut tail = SaveWriter::new(true);
    tail.write_opt_string(None);
    tail.write_anchored_time_ticks(muo_primitives::DateTime::MIN.ticks());
    want.extend(tail.into_vec());
    want.push(0); // Write(Type) null
    want.extend([3, 3]); // two null TextDefinitions
    want.extend([0, 0, 0, 0]); // Merchant's own version
    assert_eq!(save(&w, id), want);

    // A filled one reads and writes the same bytes.
    let mut filled = creature_part(&w, id);
    filled.extend([3, 0, 0, 0]);
    let mut tail = SaveWriter::new(true);
    tail.write_opt_string(Some("the Moongate"));
    tail.write_anchored_time_ticks(muo_primitives::DateTime::MIN.ticks());
    filled.extend(tail.into_vec());
    filled.push(2); // Write(Type): flag 2 and the name hash
    filled.extend(0x0123_4567_89AB_CDEFu64.to_le_bytes());
    let mut def = SaveWriter::new(true);
    def.write_encoded_int(1); // TextDefinition with a number
    def.write_encoded_int(1075265);
    filled.extend(def.into_vec());
    let mut def = SaveWriter::new(true);
    def.write_encoded_int(2); // TextDefinition with a string
    def.write_string("paid");
    filled.extend(def.into_vec());
    filled.extend([0, 0, 0, 0]);
    let mut w2 = world(None);
    let other = load(&mut w2, ty, &filled);
    assert_eq!(save(&w2, other), filled);
}

#[test]
fn golden_player_vendor_version_four_layout() {
    let mut w = world(Some(Expansion::AOS));
    let owner = w.new_mobile(&MOBILE_BASE);
    let v = pv::make(&mut w, &PLAYER_VENDOR, Some(owner), None);
    let next = pv::data(&w, v).unwrap().next_pay_time;

    let mut want = SaveWriter::new(true);
    (MOBILE_BASE.serialize)(&w, v, &mut want);
    want.write_i32(4);
    want.write_string("Shop Not Yet Named");
    want.write_anchored_time_ticks(next.ticks());
    want.write_i32(-1); // no house
    want.write_i32(w.mobile(owner).unwrap().serial.value() as i32);
    want.write_i32(0); // bank account of the new vendor system
    want.write_i32(4); // hold gold
    want.write_encoded_int(0); // no items for sale
    assert_eq!(save(&w, v), want.into_vec());
}

#[test]
fn golden_rented_vendor_version_one_layout_and_the_old_delta_time() {
    let mut w = world(Some(Expansion::AOS));
    let rv = rented::make(&mut w, None, None, 2, 7000, true, 250);
    let expire = rented::rdata(&w, rv).unwrap().rental_expire_time;

    let mut want = SaveWriter::new(true);
    (PLAYER_VENDOR.serialize)(&w, rv, &mut want);
    want.write_encoded_int(1);
    want.write_i32(2);
    want.write_i32(7000);
    want.write_bool(true); // landlord renew
    want.write_bool(false); // renter renew
    want.write_i32(7000); // renewal price
    want.write_i32(250);
    want.write_anchored_time_ticks(expire.ticks());
    let want = want.into_vec();
    assert_eq!(save(&w, rv), want);

    let mut w2 = world(Some(Expansion::AOS));
    let other = load(&mut w2, &RENTED_VENDOR, &want);
    // The Mobile part keeps fields the fresh mobile does not rebuild; the vendor and rental parts come back as saved.
    let again = save(&w2, other);
    assert_eq!(again[again.len() - 80..], want[want.len() - 80..]);
    let d = rented::rdata(&w2, other).unwrap();
    assert_eq!(
        (d.rental_duration_id, d.rental_price, d.rental_gold),
        (2, 7000, 250)
    );

    // Version 0 keeps the expiry as a delta time (the writer is obsolete but kept for this).
    let mut old = SaveWriter::new(true);
    (PLAYER_VENDOR.serialize)(&w, rv, &mut old);
    old.write_encoded_int(0);
    old.write_i32(1);
    old.write_i32(5000);
    old.write_bool(false);
    old.write_bool(true);
    old.write_i32(5000);
    old.write_i32(0);
    old.write_i64(muo_primitives::TimeSpan::from_days(3).ticks());
    let mut w3 = world(Some(Expansion::AOS));
    let o3 = load(&mut w3, &RENTED_VENDOR, &old.into_vec());
    let d = rented::rdata(&w3, o3).unwrap();
    assert_eq!(
        (d.rental_duration_id, d.rental_price, d.renter_renew),
        (1, 5000, true)
    );
}

#[test]
fn golden_player_barkeeper_version_two_and_the_old_rumor_flags() {
    let mut w = world(None);
    let owner = w.new_mobile(&MOBILE_BASE);
    let owner_serial = w.mobile(owner).unwrap().serial.value().to_le_bytes();
    let b = barkeeper::make(&mut w, Some(owner), None);

    // No rumors, no tip: house -1, owner, 0 rumors, null tip.
    let mut want = creature_part(&w, b);
    want.extend([2, 0, 0, 0]); // BaseVendor
    want.extend([2, 0, 0, 0]); // PlayerBarkeeper
    want.extend([0xFF; 4]);
    want.extend(owner_serial);
    want.push(0);
    let mut tip = SaveWriter::new(true);
    tip.write_opt_string(None);
    want.extend(tip.into_vec());
    assert_eq!(save(&w, b), want);

    // Rumors are BarkeeperRumor.Serialize: encoded version 0, message, keyword.
    barkeeper::end_change_rumor(&mut w, b, owner, 1, "news");
    barkeeper::end_change_keyword(&mut w, b, owner, 1, "key");
    barkeeper::end_change_tip(&mut w, b, owner, "tip");
    let mut want = creature_part(&w, b);
    want.extend([2, 0, 0, 0, 2, 0, 0, 0, 0xFF, 0xFF, 0xFF, 0xFF]);
    want.extend(owner_serial);
    want.push(3); // three slots: the null ones are saved as empty rumors
    let mut empty = SaveWriter::new(true);
    empty.write_encoded_int(0);
    empty.write_opt_string(None);
    empty.write_opt_string(None);
    let empty = empty.into_vec();
    want.extend(&empty);
    want.push(0);
    want.extend(string("news"));
    want.extend(string("key"));
    want.extend(&empty);
    want.extend(string("tip"));
    assert_eq!(save(&w, b), want);

    let mut w2 = world(None);
    let other = load(&mut w2, &PLAYER_BARKEEPER, &want);
    let d = barkeeper::bdata(&w2, other).unwrap();
    assert_eq!(d.tip_message.as_deref(), Some("tip"));
    assert_eq!(
        d.rumors.as_ref().unwrap()[1],
        Some(Rumor {
            message: Some("news".into()),
            keyword: Some("key".into())
        })
    );

    // Versions 0 and 1: a bool before each rumor, then message and keyword.
    let mut old = creature_part(&w, b);
    old.extend([
        2, 0, 0, 0, 1, 0, 0, 0, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    ]);
    old.push(2);
    let mut r = SaveWriter::new(true);
    r.write_bool(true);
    r.write_string("m");
    r.write_string("k");
    r.write_bool(false);
    r.write_opt_string(None);
    old.extend(r.into_vec());
    let mut w3 = world(None);
    let o3 = load(&mut w3, &PLAYER_BARKEEPER, &old);
    let d = barkeeper::bdata(&w3, o3).unwrap();
    assert_eq!(d.rumors.as_ref().unwrap().len(), 2);
    assert_eq!(d.rumors.as_ref().unwrap()[1], None);
}

#[test]
fn golden_vendor_rental_contract_encoded_version_and_three_fields() {
    let mut w = world(None);
    let id = (VENDOR_RENTAL_CONTRACT.construct.unwrap())(&mut w);
    assert_eq!(
        rental_contract::cdata(&w, id).map(|d| (d.duration_id, d.price)),
        Some((0, 1500))
    );
    rental_contract::contract_set_duration(&mut w, id, 3);
    rental_contract::contract_set_price(&mut w, id, 99);
    rental_contract::contract_toggle_landlord_renew(&mut w, id);

    let mut got = SaveWriter::new(true);
    (VENDOR_RENTAL_CONTRACT.serialize)(&w, id, &mut got);
    let mut want = SaveWriter::new(true);
    (muo_server::item::ITEM_BASE.serialize)(&w, id, &mut want);
    want.write_encoded_int(0);
    want.write_i32(3);
    want.write_i32(99);
    want.write_bool(true);
    let want = want.into_vec();
    assert_eq!(got.into_vec(), want);

    let other = w.new_item(&VENDOR_RENTAL_CONTRACT);
    let mut r = SaveReader::new(&want);
    (VENDOR_RENTAL_CONTRACT.deserialize)(&mut w, other, &mut r).unwrap();
    assert_eq!(r.remaining(), 0);
    assert_eq!(
        rental_contract::cdata(&w, other).map(|d| (d.duration_id, d.price, d.landlord_renew)),
        Some((3, 99, true))
    );
}
