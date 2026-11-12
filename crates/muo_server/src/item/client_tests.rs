//! Container child removal, lift reject resync and private child properties tests,
//! with `TestClientHarness` as `client` and the `received_*` probes: a probe
//! searches the bytes queued for a connection (`w.net_out`) for the packet.

use super::container::{self, CONTAINER};
use super::ops;
use super::*;
use crate::ids::{ItemId, MobileId};
use crate::layer::Layer;
use crate::lr_reason::LRReason;
use crate::map::MapId;
use crate::map::test_support::world;
use crate::mobile::MOBILE_BASE;
use crate::mobile::part1::lifecycle;
use crate::mobile::part2::items as mitems;
use crate::net_state::{NetOut, NetStateData, NetStateId};
use crate::packets;
use crate::world::World;
use muo_primitives::{Point3D, Serial};

const FEL: MapId = MapId::FELUCCA;
const OWNER_LOC: Point3D = Point3D {
    x: 1500,
    y: 1500,
    z: 0,
};

fn setup() -> World {
    let mut w = world();
    w.core.opl_enabled = true;
    w
}

fn mobile(w: &mut World) -> MobileId {
    let m = w.new_mobile(&MOBILE_BASE);
    crate::mobile::part2::stats::default_mobile_init(w, m);
    m
}

/// `CreateClient(location)`: a logged-in NetState and Mobile.
fn client(w: &mut World, x: i32, y: i32) -> (NetStateId, MobileId) {
    let ns = NetStateId(w.net_states.len() as u32 + 1);
    let m = mobile(w);
    w.net_states.insert(
        ns,
        NetStateData {
            mobile: Some(m),
            ..Default::default()
        },
    );
    w.mobile_mut(m).unwrap().net_state = Some(ns);
    lifecycle::move_to_world(w, m, Point3D::new(x, y, 0), FEL);
    (ns, m)
}

fn at(dx: i32) -> (i32, i32) {
    (OWNER_LOC.x + dx, OWNER_LOC.y)
}

fn sent_to(w: &World, ns: NetStateId) -> Vec<u8> {
    let mut out = Vec::new();
    for o in &w.net_out {
        if let NetOut::Send(n, b) = o
            && *n == ns
        {
            out.extend_from_slice(b);
        }
    }
    out
}

fn count(haystack: &[u8], needle: &[u8]) -> usize {
    if needle.is_empty() || haystack.len() < needle.len() {
        return 0;
    }
    let mut n = 0;
    let mut i = 0;
    while i + needle.len() <= haystack.len() {
        if &haystack[i..i + needle.len()] == needle {
            n += 1;
            i += needle.len();
        } else {
            i += 1;
        }
    }
    n
}

fn count_removes(w: &World, ns: NetStateId, serial: Serial) -> usize {
    count(
        &sent_to(w, ns),
        &packets::entity::create_remove_entity(serial),
    )
}

fn received_remove(w: &World, ns: NetStateId, item: ItemId) -> bool {
    count_removes(w, ns, item.0) > 0
}

fn received_with_id(w: &World, ns: NetStateId, id: u8, item: ItemId) -> bool {
    let mut expected = vec![id];
    expected.extend(item.0.value().to_be_bytes());
    count(&sent_to(w, ns), &expected) > 0
}

fn received_content_update(w: &World, ns: NetStateId, item: ItemId) -> bool {
    received_with_id(w, ns, 0x25, item)
}

fn received_equip_update(w: &World, ns: NetStateId, item: ItemId) -> bool {
    received_with_id(w, ns, 0x2E, item)
}

fn received_opl_info(w: &World, ns: NetStateId, item: ItemId) -> bool {
    received_with_id(w, ns, 0xDC, item)
}

fn received_world_item(w: &World, ns: NetStateId, item: ItemId) -> bool {
    let expected = packets::item::create_world_item(w, item).expect("world item packet");
    count(&sent_to(w, ns), &expected) > 0
}

fn new_item(w: &mut World) -> ItemId {
    let id = w.new_item(&ITEM_BASE);
    w.item_mut(id).unwrap().item_id = 0x1234;
    id
}

fn new_pack(w: &mut World) -> ItemId {
    container::new_container(w, &CONTAINER, 0xE75)
}

fn delete(w: &mut World, id: ItemId) {
    if let Some(ty) = w.item(id).map(|i| i.ty) {
        (ty.delete)(w, id);
    }
}

fn set_openers(w: &mut World, pack: ItemId, openers: Vec<MobileId>) {
    w.item_mut(pack).unwrap().container_mut().openers = Some(openers);
}

fn is_public_container(_: &World, _: ItemId) -> bool {
    true
}

fn is_child_public_never(_: &World, _: ItemId, _: ItemId) -> bool {
    false
}

static PUBLIC_CONTAINER: ItemType = ItemType {
    full_name: "Server.Tests.PublicContainer",
    parent: Some(&CONTAINER),
    is_public_container,
    ..CONTAINER
};

/// An override that narrows below IsPublicContainer instead of only widening it.
static PUBLIC_CONTAINER_NARROWING: ItemType = ItemType {
    full_name: "Server.Tests.PublicContainerWithNarrowingOverride",
    parent: Some(&CONTAINER),
    is_public_container,
    is_child_public: is_child_public_never,
    ..CONTAINER
};

/// `CreateOwnerWith(pack)`: a mobile wearing `pack` on the ShopBuy layer, invisible.
fn owner_with(w: &mut World, pack: ItemId) -> MobileId {
    let owner = mobile(w);
    lifecycle::move_to_world(w, owner, OWNER_LOC, FEL);
    w.item_mut(pack).unwrap().layer = Layer::ShopBuy;
    ops::set_visible(w, pack, false);
    mitems::add_item(w, owner, pack);
    owner
}

fn drop_into(w: &mut World, pack: ItemId, item: ItemId) {
    container::drop_item_default(w, pack, item);
}

#[test]
fn private_child_delete_is_not_broadcast_to_nearby_clients() {
    let mut w = setup();
    let (x, y) = at(1);
    let (ns, _client) = client(&mut w, x, y);
    let pack = new_pack(&mut w);
    let _owner = owner_with(&mut w, pack);
    let child = new_item(&mut w);
    drop_into(&mut w, pack, child);

    delete(&mut w, child);
    assert!(!received_remove(&w, ns, child));
}

#[test]
fn private_child_delete_reaches_openers() {
    let mut w = setup();
    let (x, y) = at(1);
    let (ns, client) = client(&mut w, x, y);
    let pack = new_pack(&mut w);
    let _owner = owner_with(&mut w, pack);
    let child = new_item(&mut w);
    drop_into(&mut w, pack, child);
    set_openers(&mut w, pack, vec![client]);

    delete(&mut w, child);
    assert!(received_remove(&w, ns, child));
}

#[test]
fn private_child_delete_reaches_the_root_owners_client() {
    let mut w = setup();
    let (owner_ns, owner) = client(&mut w, OWNER_LOC.x, OWNER_LOC.y);
    let (x, y) = at(1);
    let (bystander_ns, _bystander) = client(&mut w, x, y);

    let pack = new_pack(&mut w);
    w.item_mut(pack).unwrap().layer = Layer::Backpack;
    ops::set_visible(&mut w, pack, false);
    mitems::add_item(&mut w, owner, pack);

    let child = new_item(&mut w);
    drop_into(&mut w, pack, child);

    delete(&mut w, child);
    assert!(received_remove(&w, owner_ns, child));
    assert!(!received_remove(&w, bystander_ns, child));
}

#[test]
fn public_container_child_delete_is_broadcast() {
    let mut w = setup();
    let (x, y) = at(1);
    let (ns, _c) = client(&mut w, x, y);
    let pack = container::new_container(&mut w, &PUBLIC_CONTAINER, 0xE75);
    let _owner = owner_with(&mut w, pack);
    let child = new_item(&mut w);
    drop_into(&mut w, pack, child);

    delete(&mut w, child);
    assert!(received_remove(&w, ns, child));
}

#[test]
fn public_container_narrowing_child_override_still_broadcasts_removal() {
    let mut w = setup();
    let (x, y) = at(1);
    let (ns, _c) = client(&mut w, x, y);
    let pack = container::new_container(&mut w, &PUBLIC_CONTAINER_NARROWING, 0xE75);
    let _owner = owner_with(&mut w, pack);
    let child = new_item(&mut w);
    drop_into(&mut w, pack, child);

    delete(&mut w, child);
    assert!(received_remove(&w, ns, child));
}

#[test]
fn is_child_public_broadcasts_only_that_child() {
    // OneChildPublicContainer: IsChildPublic(child) == (child == PublicChild); the field is state, so the
    // type reads it from the container's `tag` list of one entry.
    let mut w = setup();
    let (x, y) = at(1);
    let (ns, _c) = client(&mut w, x, y);
    let pack = container::new_container(&mut w, &ONE_CHILD_PUBLIC, 0xE75);
    let _owner = owner_with(&mut w, pack);
    let public_child = new_item(&mut w);
    let private_child = new_item(&mut w);
    w.item_mut(private_child).unwrap().item_id = 0x1235;
    drop_into(&mut w, pack, public_child);
    drop_into(&mut w, pack, private_child);
    PUBLIC_CHILD.with(|c| c.set(Some(public_child)));

    delete(&mut w, private_child);
    assert!(!received_remove(&w, ns, private_child));

    delete(&mut w, public_child);
    assert!(received_remove(&w, ns, public_child));
    PUBLIC_CHILD.with(|c| c.set(None));
}

thread_local! {
    /// `OneChildPublicContainer.PublicChild` (test-local state; tests run one per thread).
    static PUBLIC_CHILD: std::cell::Cell<Option<ItemId>> = const { std::cell::Cell::new(None) };
}

fn is_child_public_one(_: &World, _: ItemId, child: ItemId) -> bool {
    PUBLIC_CHILD.with(|c| c.get()) == Some(child)
}

static ONE_CHILD_PUBLIC: ItemType = ItemType {
    full_name: "Server.Tests.OneChildPublicContainer",
    parent: Some(&CONTAINER),
    is_child_public: is_child_public_one,
    ..CONTAINER
};

#[test]
fn deleting_owner_sends_no_remove_for_private_children() {
    let mut w = setup();
    let (x, y) = at(1);
    let (ns, _c) = client(&mut w, x, y);
    let pack = new_pack(&mut w);
    let owner = owner_with(&mut w, pack);
    let mut children = Vec::new();
    for _ in 0..50 {
        let c = new_item(&mut w);
        drop_into(&mut w, pack, c);
        children.push(c);
    }

    crate::mobile::part1::lifecycle::delete(&mut w, owner);

    for c in children {
        assert!(w.item(c).is_none_or(|i| i.deleted()));
        assert!(!received_remove(&w, ns, c));
    }
}

#[test]
fn facet_change_does_not_remove_nested_container_children() {
    let mut w = setup();
    let (owner_ns, owner) = client(&mut w, OWNER_LOC.x, OWNER_LOC.y);
    let (x, y) = at(1);
    let (bystander_ns, _b) = client(&mut w, x, y);

    let backpack = new_pack(&mut w);
    w.item_mut(backpack).unwrap().layer = Layer::Backpack;
    mitems::add_item(&mut w, owner, backpack);
    let pouch = new_pack(&mut w);
    drop_into(&mut w, backpack, pouch);
    let reagent = new_item(&mut w);
    w.item_mut(reagent).unwrap().item_id = 0xF7A;
    drop_into(&mut w, pouch, reagent);

    lifecycle::move_to_world(&mut w, owner, OWNER_LOC, MapId::TRAMMEL);

    assert!(!received_remove(&w, owner_ns, pouch));
    assert!(!received_remove(&w, owner_ns, reagent));
    assert!(!received_remove(&w, bystander_ns, pouch));
    assert!(!received_remove(&w, bystander_ns, reagent));
}

#[test]
fn no_duplicate_remove_when_root_is_also_an_opener() {
    let mut w = setup();
    let (owner_ns, owner) = client(&mut w, OWNER_LOC.x, OWNER_LOC.y);

    let pack = new_pack(&mut w);
    w.item_mut(pack).unwrap().layer = Layer::Backpack;
    ops::set_visible(&mut w, pack, false);
    mitems::add_item(&mut w, owner, pack);
    set_openers(&mut w, pack, vec![owner]);

    let child = new_item(&mut w);
    drop_into(&mut w, pack, child);

    delete(&mut w, child);
    assert_eq!(count_removes(&w, owner_ns, child.0), 1);
}

#[test]
fn lifting_from_own_backpack_does_not_broadcast_remove() {
    let mut w = setup();
    let (owner_ns, owner) = client(&mut w, OWNER_LOC.x, OWNER_LOC.y);
    let (x, y) = at(1);
    let (bystander_ns, _b) = client(&mut w, x, y);

    let pack = new_pack(&mut w);
    w.item_mut(pack).unwrap().layer = Layer::Backpack;
    mitems::add_item(&mut w, owner, pack);
    let item = new_item(&mut w);
    drop_into(&mut w, pack, item);

    ops::internalize(&mut w, item);

    assert!(received_remove(&w, owner_ns, item));
    assert!(!received_remove(&w, bystander_ns, item));
}

#[test]
fn moving_private_child_to_ground_same_map_does_not_send_old_location_remove_to_bystanders() {
    let mut w = setup();
    let (_owner_ns, owner) = client(&mut w, OWNER_LOC.x, OWNER_LOC.y);
    let (x, y) = at(1);
    let (bystander_ns, _b) = client(&mut w, x, y);

    let pack = new_pack(&mut w);
    w.item_mut(pack).unwrap().layer = Layer::Backpack;
    mitems::add_item(&mut w, owner, pack);
    let item = new_item(&mut w);
    drop_into(&mut w, pack, item);

    let far = Point3D::new(OWNER_LOC.x + 100, OWNER_LOC.y, 0);
    ops::move_to_world_map(&mut w, item, far, Some(FEL));

    assert!(!received_remove(&w, bystander_ns, item));
}

#[test]
fn lifting_equipped_item_still_broadcasts_remove() {
    let mut w = setup();
    let (_owner_ns, owner) = client(&mut w, OWNER_LOC.x, OWNER_LOC.y);
    let (x, y) = at(1);
    let (bystander_ns, _b) = client(&mut w, x, y);

    let weapon = new_item(&mut w);
    w.item_mut(weapon).unwrap().layer = Layer::OneHanded;
    mitems::add_item(&mut w, owner, weapon);

    ops::internalize(&mut w, weapon);

    assert!(received_remove(&w, bystander_ns, weapon));
}

#[test]
fn ground_item_move_still_sends_old_location_remove() {
    let mut w = setup();
    let (bystander_ns, _b) = client(&mut w, OWNER_LOC.x, OWNER_LOC.y);

    let item = new_item(&mut w);
    ops::move_to_world_map(
        &mut w,
        item,
        Point3D::new(OWNER_LOC.x + 1, OWNER_LOC.y, 0),
        Some(FEL),
    );

    let far = Point3D::new(OWNER_LOC.x + 100, OWNER_LOC.y, 0);
    ops::move_to_world_map(&mut w, item, far, Some(FEL));

    assert!(received_remove(&w, bystander_ns, item));
}

// ---- PrivateChildPropertiesTests ----

const PROPS_LOC: Point3D = Point3D {
    x: 1600,
    y: 1600,
    z: 0,
};

/// `item.ProcessDelta()`.
fn drain_pending_delta(w: &mut World, item: ItemId) {
    let ty = w.item(item).unwrap().ty;
    (ty.process_delta)(w, item);
}

/// Queues ItemDelta.Properties directly, then processes it (the properties-only branch).
fn invalidate_and_flush(w: &mut World, item: ItemId) {
    ops::delta(w, item, ItemDelta::PROPERTIES);
    let ty = w.item(item).unwrap().ty;
    (ty.process_delta)(w, item);
}

#[test]
fn private_child_in_owner_backpack_properties_only_update_reaches_owner_not_bystander() {
    let mut w = setup();
    let (owner_ns, owner) = client(&mut w, PROPS_LOC.x, PROPS_LOC.y);
    let (bystander_ns, _b) = client(&mut w, PROPS_LOC.x + 1, PROPS_LOC.y);

    let pack = new_pack(&mut w);
    w.item_mut(pack).unwrap().layer = Layer::Backpack;
    mitems::add_item(&mut w, owner, pack);
    let child = new_item(&mut w);
    drop_into(&mut w, pack, child);

    drain_pending_delta(&mut w, child);
    invalidate_and_flush(&mut w, child);

    assert!(received_opl_info(&w, owner_ns, child));
    assert!(!received_opl_info(&w, bystander_ns, child));
}

#[test]
fn private_ground_container_properties_only_update_reaches_opener_not_non_opener() {
    let mut w = setup();
    let (opener_ns, opener) = client(&mut w, PROPS_LOC.x, PROPS_LOC.y);
    let (bystander_ns, _b) = client(&mut w, PROPS_LOC.x + 2, PROPS_LOC.y);

    let cont = new_pack(&mut w);
    ops::move_to_world_map(
        &mut w,
        cont,
        Point3D::new(PROPS_LOC.x + 1, PROPS_LOC.y, 0),
        Some(FEL),
    );
    set_openers(&mut w, cont, vec![opener]);
    let child = new_item(&mut w);
    drop_into(&mut w, cont, child);

    drain_pending_delta(&mut w, child);
    invalidate_and_flush(&mut w, child);

    assert!(received_opl_info(&w, opener_ns, child));
    assert!(!received_opl_info(&w, bystander_ns, child));
}

#[test]
fn public_container_child_properties_only_update_still_reaches_bystander() {
    let mut w = setup();
    let (bystander_ns, _b) = client(&mut w, PROPS_LOC.x, PROPS_LOC.y);

    let pack = container::new_container(&mut w, &PUBLIC_CONTAINER, 0xE75);
    ops::move_to_world_map(
        &mut w,
        pack,
        Point3D::new(PROPS_LOC.x + 1, PROPS_LOC.y, 0),
        Some(FEL),
    );
    let child = new_item(&mut w);
    drop_into(&mut w, pack, child);

    drain_pending_delta(&mut w, child);
    invalidate_and_flush(&mut w, child);

    assert!(received_opl_info(&w, bystander_ns, child));
}

#[test]
fn ground_item_properties_only_update_still_reaches_bystander() {
    let mut w = setup();
    let (bystander_ns, _b) = client(&mut w, PROPS_LOC.x, PROPS_LOC.y);

    let item = new_item(&mut w);
    ops::move_to_world_map(
        &mut w,
        item,
        Point3D::new(PROPS_LOC.x + 1, PROPS_LOC.y, 0),
        Some(FEL),
    );

    drain_pending_delta(&mut w, item);
    invalidate_and_flush(&mut w, item);

    assert!(received_opl_info(&w, bystander_ns, item));
}

#[test]
fn trade_partner_receives_remove_for_nested_item() {
    let mut w = setup();
    let (_a_ns, a) = client(&mut w, OWNER_LOC.x, OWNER_LOC.y);
    let (x, y) = at(1);
    let (b_ns, b) = client(&mut w, x, y);

    let trade = crate::secure_trade::new_trade(&mut w, a, b);
    let from_container = w.trades.get(&trade).unwrap().from.container.unwrap();
    let pouch = new_pack(&mut w);
    drop_into(&mut w, from_container, pouch);
    let item = new_item(&mut w);
    drop_into(&mut w, pouch, item);

    delete(&mut w, item);
    assert!(received_remove(&w, b_ns, item));
}

// ---- LiftRejectResyncTests ----

const BASE_LOC: Point3D = Point3D {
    x: 2200,
    y: 2200,
    z: 0,
};

fn hold(w: &mut World, m: MobileId) -> ItemId {
    let held = w.new_item(&ITEM_BASE);
    w.mobile_mut(m).unwrap().holding = Some(held);
    held
}

fn backpack_of(w: &mut World, owner: MobileId) -> ItemId {
    let backpack = new_pack(w);
    w.item_mut(backpack).unwrap().layer = Layer::Backpack;
    mitems::add_item(w, owner, backpack);
    backpack
}

#[test]
fn private_nested_item_rejected_lift_does_not_resync_bystander() {
    let mut w = setup();
    let (_owner_ns, owner) = client(&mut w, BASE_LOC.x, BASE_LOC.y);
    let (bystander_ns, bystander) = client(&mut w, BASE_LOC.x + 1, BASE_LOC.y);

    let backpack = backpack_of(&mut w, owner);
    let pouch = new_pack(&mut w);
    drop_into(&mut w, backpack, pouch);
    let item = new_item(&mut w);
    drop_into(&mut w, pouch, item);

    let (rejected, _) = crate::mobile::base::lift(&mut w, bystander, item, 1);

    // Whichever check trips first, the bystander was never sent this private child, so the rejection
    // must not resynchronize it to them.
    assert!(rejected);
    assert!(!received_content_update(&w, bystander_ns, item));
}

#[test]
fn owner_already_holding_rejected_lift_resyncs_private_child_to_owner() {
    let mut w = setup();
    let (owner_ns, owner) = client(&mut w, BASE_LOC.x, BASE_LOC.y);
    let backpack = backpack_of(&mut w, owner);
    let item = new_item(&mut w);
    drop_into(&mut w, backpack, item);
    hold(&mut w, owner);

    let (rejected, reject) = crate::mobile::base::lift(&mut w, owner, item, 1);

    assert!(rejected);
    assert_eq!(reject, LRReason::AreHolding);
    assert!(received_content_update(&w, owner_ns, item));
}

#[test]
fn ground_item_out_of_range_rejected_lift_does_not_resync_requester() {
    let mut w = setup();
    let (requester_ns, requester) = client(&mut w, BASE_LOC.x, BASE_LOC.y);
    let item = new_item(&mut w);
    ops::move_to_world_map(
        &mut w,
        item,
        Point3D::new(BASE_LOC.x, BASE_LOC.y + 100, 0),
        Some(FEL),
    );

    let (rejected, reject) = crate::mobile::base::lift(&mut w, requester, item, 1);

    assert!(rejected);
    assert_eq!(reject, LRReason::OutOfRange);
    assert!(!received_world_item(&w, requester_ns, item));
}

#[test]
fn ground_container_opener_forced_rejection_resyncs_private_child_to_opener() {
    let mut w = setup();
    let (requester_ns, requester) = client(&mut w, BASE_LOC.x, BASE_LOC.y);
    let cont = new_pack(&mut w);
    ops::move_to_world_map(
        &mut w,
        cont,
        Point3D::new(BASE_LOC.x + 1, BASE_LOC.y, 0),
        Some(FEL),
    );
    let item = new_item(&mut w);
    drop_into(&mut w, cont, item);
    set_openers(&mut w, cont, vec![requester]);
    hold(&mut w, requester);

    let (rejected, reject) = crate::mobile::base::lift(&mut w, requester, item, 1);

    assert!(rejected);
    assert_eq!(reject, LRReason::AreHolding);
    assert!(received_content_update(&w, requester_ns, item));
}

#[test]
fn trade_partner_rejected_lift_resyncs_nested_item_to_trade_partner() {
    let mut w = setup();
    let (_a_ns, a) = client(&mut w, BASE_LOC.x, BASE_LOC.y);
    let (b_ns, b) = client(&mut w, BASE_LOC.x + 1, BASE_LOC.y);

    let trade = crate::secure_trade::new_trade(&mut w, a, b);
    let from_container = w.trades.get(&trade).unwrap().from.container.unwrap();
    let pouch = new_pack(&mut w);
    drop_into(&mut w, from_container, pouch);
    let item = new_item(&mut w);
    drop_into(&mut w, pouch, item);
    hold(&mut w, b);

    // b is the trade partner of a: not the item root and never an Opener of the pouch, but a sanctioned
    // recipient of this private child through the trade relationship.
    let (rejected, reject) = crate::mobile::base::lift(&mut w, b, item, 1);

    assert!(rejected);
    assert_eq!(reject, LRReason::AreHolding);
    assert!(received_content_update(&w, b_ns, item));
}

#[test]
fn invisible_item_rejected_lift_does_not_resync_to_non_staff_owner() {
    let mut w = setup();
    let (owner_ns, owner) = client(&mut w, BASE_LOC.x, BASE_LOC.y);
    let backpack = backpack_of(&mut w, owner);
    let item = new_item(&mut w);
    ops::set_visible(&mut w, item, false);
    drop_into(&mut w, backpack, item);
    hold(&mut w, owner);

    let (rejected, reject) = crate::mobile::base::lift(&mut w, owner, item, 1);

    // ProcessDelta never sends an invisible item to a non-staff owner (CanSee gates every private
    // recipient); the rejection resync must honor the same gate.
    assert!(rejected);
    assert_eq!(reject, LRReason::AreHolding);
    assert!(!received_content_update(&w, owner_ns, item));
}

#[test]
fn invisible_item_rejected_lift_resyncs_to_staff_owner() {
    let mut w = setup();
    let (owner_ns, owner) = client(&mut w, BASE_LOC.x, BASE_LOC.y);
    w.mobile_mut(owner).unwrap().access_level = crate::mobile::AccessLevel::GameMaster;
    let backpack = backpack_of(&mut w, owner);
    let item = new_item(&mut w);
    ops::set_visible(&mut w, item, false);
    drop_into(&mut w, backpack, item);
    hold(&mut w, owner);

    let (rejected, reject) = crate::mobile::base::lift(&mut w, owner, item, 1);

    assert!(rejected);
    assert_eq!(reject, LRReason::AreHolding);
    assert!(received_content_update(&w, owner_ns, item));
}

#[test]
fn equipped_item_on_hidden_mobile_rejected_lift_no_equip_resync() {
    let mut w = setup();
    let (_owner_ns, owner) = client(&mut w, BASE_LOC.x, BASE_LOC.y);
    let (requester_ns, requester) = client(&mut w, BASE_LOC.x + 1, BASE_LOC.y);

    w.mobile_mut(owner).unwrap().hidden = true;
    let weapon = new_item(&mut w);
    w.item_mut(weapon).unwrap().layer = Layer::OneHanded;
    mitems::add_item(&mut w, owner, weapon);

    let (rejected, reject) = crate::mobile::base::lift(&mut w, requester, weapon, 1);

    // The requester cannot see a hidden, non-staff owner: the lift is rejected OutOfSight and
    // IsSentTo CanSee gate must suppress the equip resync.
    assert!(rejected);
    assert_eq!(reject, LRReason::OutOfSight);
    assert!(!received_equip_update(&w, requester_ns, weapon));
}

#[test]
fn invisible_ground_item_in_range_rejected_lift_no_world_resync() {
    let mut w = setup();
    let (requester_ns, requester) = client(&mut w, BASE_LOC.x, BASE_LOC.y);
    let item = new_item(&mut w);
    ops::set_visible(&mut w, item, false);
    ops::move_to_world_map(
        &mut w,
        item,
        Point3D::new(BASE_LOC.x + 1, BASE_LOC.y, 0),
        Some(FEL),
    );

    let (rejected, reject) = crate::mobile::base::lift(&mut w, requester, item, 1);

    // In range this time: only Visible gates it, so the rejection resync must not hand the requester a
    // world packet for an item they cannot see.
    assert!(rejected);
    assert_eq!(reject, LRReason::OutOfSight);
    assert!(!received_world_item(&w, requester_ns, item));
}
