//! Vendor buyback tests: a vendor over a local `Bandage` class.

use muo_server::ids::{ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType, ops};
use muo_server::layer::Layer;
use muo_server::mobile::{MOBILE_BASE, MobileType};
use muo_server::vendor::SellItemResponse;
use muo_server::world::World;

use super::*;
use crate::creature::CreatureType;
use crate::creature::speeds::{NpcSpeeds, SpeedClassEntry};
use crate::items::{cont, gold};

static BANDAGE: ItemType = ItemType {
    full_name: "Server.Items.Bandage",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let id = ops::new_item_with_id(w, &BANDAGE, 0xE21);
        ops::set_stackable(w, id, true);
        id
    }),
    ..ITEM_BASE
};

fn bandage_sb(_w: &mut World, _v: MobileId, sb: &mut SbInstance) {
    sb.sell.add("Server.Items.Bandage", 2);
}
static BANDAGE_SB: SbInfo = SbInfo {
    name: "BandageSB",
    build: bandage_sb,
};

/// Same sell price, but a buy list at the 250-entry limit so the buyback capacity is 0.
fn full_stock_sb(_w: &mut World, _v: MobileId, sb: &mut SbInstance) {
    sb.sell.add("Server.Items.Bandage", 2);
    for _ in 0..250 {
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Bandage", 5, 20, 0xE21, 0));
    }
}
static FULL_STOCK_SB: SbInfo = SbInfo {
    name: "FullStockSB",
    build: full_stock_sb,
};

static BANDAGE_VENDOR_V: VendorClass = VendorClass {
    init_sb_info: |w, id| vec![BANDAGE_SB.new_for(w, id)],
    ..BASE_VENDOR_CLASS
};
static FULL_STOCK_VENDOR_V: VendorClass = VendorClass {
    init_sb_info: |w, id| vec![FULL_STOCK_SB.new_for(w, id)],
    ..BASE_VENDOR_CLASS
};
static BANDAGE_VENDOR_C: CreatureType = CreatureType {
    subclass: Some(&BANDAGE_VENDOR_V),
    ..BASE_VENDOR_C
};
static FULL_STOCK_VENDOR_C: CreatureType = CreatureType {
    subclass: Some(&FULL_STOCK_VENDOR_V),
    ..BASE_VENDOR_C
};
static BANDAGE_VENDOR: MobileType = MobileType {
    full_name: "Test.BandageVendor",
    parent: Some(&BASE_VENDOR),
    construct: Some(|w| new_vendor(w, &BANDAGE_VENDOR, None)),
    family: Some(&BANDAGE_VENDOR_C),
    ..BASE_VENDOR
};
static FULL_STOCK_VENDOR: MobileType = MobileType {
    full_name: "Test.FullStockVendor",
    parent: Some(&BASE_VENDOR),
    construct: Some(|w| new_vendor(w, &FULL_STOCK_VENDOR, None)),
    family: Some(&FULL_STOCK_VENDOR_C),
    ..BASE_VENDOR
};

fn world() -> World {
    let mut w = World::new();
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
    w.types.register_item(&BANDAGE, &[]);
    w.types.register_item(&VENDOR_BUYBACK_PACK, &[]);
    cont::register(&mut w.types);
    gold::register(&mut w.types);
    w
}

/// A seller with a pack holding 10 bandages, who sells `amount` of them.
fn sell(w: &mut World, vendor: MobileId, amount: i32) -> (MobileId, ItemId) {
    let seller = w.new_mobile(&MOBILE_BASE);
    let pack = crate::factory::make_item(w, "Server.Items.Backpack").unwrap();
    muo_server::mobile::part2::items::add_item(w, seller, pack);
    if let Some(m) = w.mobile_mut(seller) {
        m.backpack = Some(pack);
    }
    let bandages = crate::factory::make_item(w, "Server.Items.Bandage").unwrap();
    ops::set_amount(w, bandages, 10);
    muo_server::item::container::drop_item_default(w, pack, bandages);
    assert!(trade::on_sell_items(
        w,
        vendor,
        seller,
        &[SellItemResponse::new(bandages, amount)]
    ));
    (seller, bandages)
}

fn gold_of(w: &World, m: MobileId) -> i32 {
    let pack = w.mobile(m).unwrap().backpack.unwrap();
    crate::factory::find_item_by_type(w, pack, "Server.Items.Gold")
        .map_or(0, |g| w.item(g).unwrap().amount)
}

#[test]
fn a_partial_stack_sale_splits_to_the_buyback_and_the_seller_keeps_the_rest() {
    let mut w = world();
    let v = (BANDAGE_VENDOR.construct.unwrap())(&mut w);
    let (seller, bandages) = sell(&mut w, v, 4);
    assert_eq!(w.item(bandages).unwrap().amount, 6);
    let seller_pack = w.mobile(seller).unwrap().backpack.unwrap();
    assert_eq!(
        w.item(bandages).unwrap().parent,
        Some(muo_server::ids::EntityId::Item(seller_pack))
    );
    let pack = buy_pack(&mut w, v).unwrap();
    let items = w.item(pack).unwrap().items.clone();
    assert_eq!(items.len(), 1);
    assert!(crate::factory::item_is(
        &w,
        items[0],
        "Server.Items.Bandage"
    ));
    assert_eq!(w.item(items[0]).unwrap().amount, 4);
    assert!(muo_server::item::base::skip_serialization(&w, items[0]));
    assert!(buyback_purge_scheduled(&w, v));
    assert_eq!(gold_of(&w, seller), 8, "4 bandages at 2 gold");
}

#[test]
fn restock_purges_the_buyback_and_a_later_sale_rearms_the_timer() {
    let mut w = world();
    let v = (BANDAGE_VENDOR.construct.unwrap())(&mut w);
    let (_, bandages) = sell(&mut w, v, 10);
    let pack = buy_pack(&mut w, v).unwrap();
    assert_eq!(w.item(pack).unwrap().items.len(), 1);
    assert!(buyback_purge_scheduled(&w, v));

    restock(&mut w, v);
    assert!(w.item(pack).unwrap().items.is_empty());
    assert!(w.item(bandages).is_none_or(|i| i.deleted()));
    assert!(!buyback_purge_scheduled(&w, v));

    sell(&mut w, v, 10);
    assert!(buyback_purge_scheduled(&w, v));
}

#[test]
fn deleting_the_vendor_cancels_the_purge_timer() {
    let mut w = world();
    let v = (BANDAGE_VENDOR.construct.unwrap())(&mut w);
    sell(&mut w, v, 10);
    assert!(buyback_purge_scheduled(&w, v));
    let t = w.mobile(v).unwrap().ty;
    (t.delete)(&mut w, v);
    // The vendor is gone from the world; its timer no longer runs.
    assert!(w.mobile(v).is_none_or(|m| m.deleted));
}

#[test]
fn a_legacy_buy_pack_is_replaced_and_its_contents_deleted() {
    let mut w = world();
    let v = (BANDAGE_VENDOR.construct.unwrap())(&mut w);
    let old = muo_server::mobile::part2::items::find_item_on_layer(&w, v, Layer::ShopBuy).unwrap();
    (w.item(old).unwrap().ty.delete)(&mut w, old);
    let legacy = crate::factory::make_item(&mut w, "Server.Items.Backpack").unwrap();
    (w.item(legacy).unwrap().ty.set_layer)(&mut w, legacy, Layer::ShopBuy);
    muo_server::mobile::part2::items::add_item(&mut w, v, legacy);
    let kids: Vec<_> = (0..3)
        .map(|_| {
            let k = crate::factory::make_item(&mut w, "Server.Items.Bandage").unwrap();
            muo_server::item::container::drop_item_default(&mut w, legacy, k);
            k
        })
        .collect();
    let pack = buy_pack(&mut w, v).unwrap();
    assert!(w.item(pack).unwrap().ty.is(&VENDOR_BUYBACK_PACK));
    assert_eq!(
        muo_server::mobile::part2::items::find_item_on_layer(&w, v, Layer::ShopBuy),
        Some(pack)
    );
    assert!(w.item(legacy).is_none_or(|i| i.deleted()));
    for k in kids {
        assert!(w.item(k).is_none_or(|i| i.deleted()));
    }
}

#[test]
fn a_sale_with_no_buyback_capacity_consumes_the_item_and_still_pays() {
    let mut w = world();
    let v = (FULL_STOCK_VENDOR.construct.unwrap())(&mut w);
    let (seller, bandages) = sell(&mut w, v, 10);
    assert!(w.item(bandages).is_none_or(|i| i.deleted()));
    let pack = buy_pack(&mut w, v).unwrap();
    assert!(w.item(pack).unwrap().items.is_empty());
    assert!(!buyback_purge_scheduled(&w, v));
    assert_eq!(gold_of(&w, seller), 20);
}

// ---- the remaining tests of VendorBuybackTests and VendorBuybackPackTests ----------------------------------------

fn is_deleted(w: &World, item: ItemId) -> bool {
    w.item(item).is_none_or(|i| i.deleted())
}

#[test]
fn restock_purges_buyback() {
    let mut w = world();
    let v = (BANDAGE_VENDOR.construct.unwrap())(&mut w);
    let (_, bandages) = sell(&mut w, v, 10);
    let pack = buy_pack(&mut w, v).unwrap();
    assert_eq!(w.item(pack).unwrap().items.len(), 1);

    restock(&mut w, v);

    assert!(w.item(pack).unwrap().items.is_empty());
    assert!(is_deleted(&w, bandages));
}

#[test]
fn restock_clears_purge_schedule_and_a_subsequent_sale_rearms_it() {
    let mut w = world();
    let v = (BANDAGE_VENDOR.construct.unwrap())(&mut w);
    sell(&mut w, v, 10);
    assert!(buyback_purge_scheduled(&w, v));

    restock(&mut w, v);

    assert!(!buyback_purge_scheduled(&w, v));

    sell(&mut w, v, 10);

    assert!(buyback_purge_scheduled(&w, v));
}

#[test]
fn delete_cancels_buyback_purge_timer() {
    let mut w = world();
    let v = (BANDAGE_VENDOR.construct.unwrap())(&mut w);
    sell(&mut w, v, 10);
    assert!(buyback_purge_scheduled(&w, v));

    let t = w.mobile(v).unwrap().ty;
    (t.delete)(&mut w, v);

    assert!(!buyback_purge_scheduled(&w, v));
}

fn new_pack(w: &mut World) -> ItemId {
    (VENDOR_BUYBACK_PACK.construct.expect("constructible"))(w)
}

fn new_item(w: &mut World) -> ItemId {
    ops::new_item_with_id(w, &ITEM_BASE, 0x1234)
}

#[test]
fn add_buyback_evicts_oldest_first_when_full() {
    let mut w = world();
    let pack = new_pack(&mut w);
    let items: Vec<ItemId> = (0..5).map(|_| new_item(&mut w)).collect();
    for i in &items {
        packs::add_buyback(&mut w, pack, *i, 3);
    }
    assert_eq!(w.item(pack).unwrap().items, items[2..].to_vec());
    assert!(is_deleted(&w, items[0]));
    assert!(is_deleted(&w, items[1]));
}

#[test]
fn add_buyback_with_no_capacity_consumes_the_item() {
    let mut w = world();
    let pack = new_pack(&mut w);
    let item = new_item(&mut w);
    packs::add_buyback(&mut w, pack, item, 0);
    assert!(is_deleted(&w, item));
    assert!(w.item(pack).unwrap().items.is_empty());
}

// DropItem rejects deleted items, so the stale entry is forced into Items directly.
#[test]
fn add_buyback_skips_already_deleted_entry_without_spinning() {
    let mut w = world();
    let pack = new_pack(&mut w);
    let (real1, stale, real2) = (new_item(&mut w), new_item(&mut w), new_item(&mut w));
    packs::add_buyback(&mut w, pack, real1, 100);

    (w.item(stale).unwrap().ty.delete)(&mut w, stale);
    w.item_mut(pack).unwrap().items.insert(0, stale);

    packs::add_buyback(&mut w, pack, real2, 1);

    assert!(is_deleted(&w, real1));
    assert!(is_deleted(&w, stale));
    assert_eq!(w.item(pack).unwrap().items, vec![real2]);
}

#[test]
fn purge_deletes_every_child() {
    let mut w = world();
    let pack = new_pack(&mut w);
    let items: Vec<ItemId> = (0..10).map(|_| new_item(&mut w)).collect();
    for i in &items {
        packs::add_buyback(&mut w, pack, *i, 250);
    }
    packs::purge(&mut w, pack);
    assert!(w.item(pack).unwrap().items.is_empty());
    for i in items {
        assert!(is_deleted(&w, i));
    }
}
