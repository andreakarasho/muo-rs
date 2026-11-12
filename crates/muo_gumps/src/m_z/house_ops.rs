//! `BaseHouse` as the vendor and house-list gumps use it ([`HouseOps`]), over `muo_multis`.

use muo_io::text::TextDefinition;
use muo_multis::house::{self, access, decay, vendors};
use muo_multis::{house_entries, sign};
use muo_primitives::Point3D;
use muo_server::World;
use muo_server::ids::{ItemId, MobileId};
use muo_server::map::MapId;

use super::ops::{HouseOps, HouseView, VendorInventoryView};

pub struct MultisHouseOps;

fn inventory_view(i: &muo_multis::house::VendorInventory) -> VendorInventoryView {
    VendorInventoryView {
        id: i.id,
        owner: i.owner,
        vendor_name: i.vendor_name.clone(),
        shop_name: i.shop_name.clone(),
        expire_time: i.expire_time,
        items: i.items.clone(),
        gold: i.gold,
    }
}

/// `FindHouseName(house)`: the description of the entry with the house's multi id, else the class name.
fn house_name(w: &World, house: ItemId) -> TextDefinition {
    let multi_id = w.item(house).map_or(0, |i| i.item_id);
    let classic = house_entries::CLASSIC_HOUSES.iter();
    let two = muo_multis::placement_entry::two_story().iter();
    let three = muo_multis::placement_entry::three_story().iter();
    for e in classic.chain(two).chain(three) {
        if e.multi_id == multi_id {
            return TextDefinition::of_number(e.description);
        }
    }
    TextDefinition::of_text(house::class(w, house).class_name)
}

impl HouseOps for MultisHouseOps {
    fn deleted(&self, w: &World, house: ItemId) -> bool {
        house::deleted(w, house)
    }
    fn is_active(&self, w: &World, house: ItemId) -> bool {
        house::is_active(w, house)
    }
    fn is_inside(&self, w: &World, house: ItemId, m: MobileId) -> bool {
        access::is_inside_mobile(w, house, m)
    }
    fn is_owner(&self, w: &World, house: ItemId, m: MobileId) -> bool {
        access::is_owner(w, house, m)
    }

    fn internalized_vendors(&self, w: &World, house: ItemId) -> Vec<MobileId> {
        house::d(w, house).internalized_vendors
    }
    fn remove_internalized_vendor(&self, w: &mut World, house: ItemId, vendor: MobileId) {
        if let Some(h) = house::data_mut(w, house) {
            h.internalized_vendors.retain(|&v| v != vendor);
        }
    }
    fn is_there_vendor(&self, w: &World, location: Point3D, map: Option<MapId>) -> (bool, bool) {
        vendors::is_there_vendor(w, location, map)
    }

    fn sign(&self, w: &World, house: ItemId) -> Option<ItemId> {
        house::sign(w, house)
    }
    fn vendor_inventories(&self, w: &World, house: ItemId) -> Vec<VendorInventoryView> {
        house::d(w, house)
            .vendor_inventories
            .iter()
            .map(inventory_view)
            .collect()
    }
    fn vendor_inventory(&self, w: &World, house: ItemId, id: u64) -> Option<VendorInventoryView> {
        house::data(w, house)?
            .vendor_inventories
            .iter()
            .find(|i| i.id == id)
            .map(inventory_view)
    }
    fn inventory_remove_item(&self, w: &mut World, house: ItemId, id: u64, item: ItemId) {
        if let Some(i) = house::data_mut(w, house)
            .and_then(|h| h.vendor_inventories.iter_mut().find(|i| i.id == id))
        {
            i.items.retain(|&x| x != item);
        }
    }
    fn inventory_set_gold(&self, w: &mut World, house: ItemId, id: u64, gold: i32) {
        if let Some(i) = house::data_mut(w, house)
            .and_then(|h| h.vendor_inventories.iter_mut().find(|i| i.id == id))
        {
            i.gold = gold;
        }
    }
    fn inventory_delete(&self, w: &mut World, house: ItemId, id: u64) {
        muo_multis::vendor_inventory::delete(w, house, id);
    }

    fn houses_of(&self, w: &World, owner: MobileId) -> Vec<ItemId> {
        // ViewHousesGump.GetHouses: every character of the account, or just the owner without one
        let chars: Vec<MobileId> = match w.mobile(owner).and_then(|m| m.account) {
            Some(a) => muo_content::accounting::account(w, a)
                .map(|x| x.mobiles.iter().flatten().copied().collect())
                .unwrap_or_default(),
            None => vec![owner],
        };
        let mut list: Vec<ItemId> = chars
            .into_iter()
            .flat_map(|m| access::get_houses(w, m))
            .collect();
        // HouseComparer: by BuiltOn (List.Sort)
        muo_primitives::dotnet::net_sort(&mut list, |&a, &b| {
            house::d(w, a).built_on.cmp(&house::d(w, b).built_on)
        });
        list
    }
    fn view(&self, w: &World, house: ItemId) -> Option<HouseView> {
        let item = w.item(house).filter(|i| !i.deleted())?;
        let map = item.map;
        let location = item.location;
        let h = house::d(w, house);
        let sign_name = h
            .sign
            .map_or_else(|| "An Unnamed House".to_string(), |s| sign::get_name(w, s));
        let owner_name = h.owner.map_or_else(
            || "nobody".to_string(),
            |o| muo_server::mobile::base::name(w, o).unwrap_or_default(),
        );
        Some(HouseView {
            house,
            name: house_name(w, house),
            sign_name,
            owner_name,
            map,
            map_name: map
                .and_then(|m| w.maps.get(m))
                .map(|m| m.name().to_string()),
            location,
            sextant: muo_content::items::house_deps::deps(w)
                .and_then(|d| d.sextant_format)
                .and_then(|f| f(w, location, map)),
            friends: h.friends.len(),
            co_owners: h.co_owners.len(),
            bans: h.bans.len(),
            can_decay: decay::can_decay(w, house),
            decay_level: format!("{:?}", decay::decay_level(w, house)),
        })
    }
    fn ban_location(&self, w: &World, house: ItemId) -> Point3D {
        house::ban_location(w, house)
    }
    fn open_sign_menu(&self, w: &mut World, house: ItemId, from: MobileId) {
        if let Some(s) = house::sign(w, house)
            && let Some(ty) = w.item(s).map(|i| i.ty)
        {
            (ty.on_double_click)(w, s, from);
        }
    }
    fn refresh_decay(&self, w: &mut World, house: ItemId) {
        decay::refresh_decay(w, house);
    }
    fn confirm_demolish(&self, w: &mut World, house: ItemId, from: MobileId) {
        crate::a_l::common::send(w, from, crate::a_l::houses::confirm_demolish(house));
    }
}
