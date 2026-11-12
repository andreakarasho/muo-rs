//! `VendorInventory`: what a removed player vendor left in a house, until it is
//! reclaimed or expires (7 days). The record lives in the house (`HouseData::vendor_inventories`).

use muo_primitives::{DateTime, TimeSpan};
use muo_server::World;
use muo_server::ids::{ItemId, MobileId};
use muo_server::item::ops;
use muo_server::timer::{self, TimerHandle};

use crate::house::{self, VendorInventory, data, data_mut, statics};

/// `VendorInventory.GracePeriod`.
pub const GRACE_PERIOD: TimeSpan = TimeSpan::from_days(7);

fn next_id(w: &mut World) -> u64 {
    let s = statics(w);
    s.next_inventory_id += 1;
    s.next_inventory_id
}

/// `new VendorInventory(house, owner, vendorName, shopName)`; adds it to the house' list.
pub fn create(
    w: &mut World,
    house: ItemId,
    owner: Option<MobileId>,
    vendor_name: &str,
    shop_name: &str,
) -> u64 {
    let id = next_id(w);
    let expire_time = w.now + GRACE_PERIOD;
    let t = start_expire_timer(w, house, id, GRACE_PERIOD);
    if let Some(h) = data_mut(w, house) {
        h.vendor_inventories.push(VendorInventory {
            owner,
            vendor_name: vendor_name.to_string(),
            shop_name: shop_name.to_string(),
            items: Vec::new(),
            gold: 0,
            expire_time,
            timer: t,
            id,
        });
    }
    id
}

/// The read constructor `new VendorInventory(house, reader)`, after the fields were read. Not yet in the list.
#[allow(clippy::too_many_arguments)]
pub fn restore(
    w: &mut World,
    house: ItemId,
    owner: Option<MobileId>,
    vendor_name: String,
    shop_name: String,
    items: Vec<ItemId>,
    gold: i32,
    expire_time: DateTime,
) -> VendorInventory {
    let id = next_id(w);
    let timer_handle = if items.is_empty() && gold == 0 {
        timer::start(w, TimeSpan::ZERO, TimeSpan::ZERO, 1, move |w| {
            delete(w, house, id)
        });
        TimerHandle::default()
    } else {
        let delay = expire_time - w.now;
        start_expire_timer(
            w,
            house,
            id,
            if delay > TimeSpan::ZERO {
                delay
            } else {
                TimeSpan::ZERO
            },
        )
    };
    VendorInventory {
        owner,
        vendor_name,
        shop_name,
        items,
        gold,
        expire_time,
        timer: timer_handle,
        id,
    }
}

fn start_expire_timer(w: &mut World, house: ItemId, id: u64, delay: TimeSpan) -> TimerHandle {
    timer::start(w, delay, TimeSpan::ZERO, 1, move |w| expire(w, house, id))
}

/// `inventory.House = house` for every inventory of `house` (a converted house took them over): the expire timers
/// name their house, so each is restarted for the new one with the time that is left.
pub fn move_to_house(w: &mut World, house: ItemId) {
    let list: Vec<(u64, DateTime, TimerHandle)> = data(w, house)
        .map(|h| {
            h.vendor_inventories
                .iter()
                .map(|v| (v.id, v.expire_time, v.timer))
                .collect()
        })
        .unwrap_or_default();
    for (id, expire, mut old) in list {
        old.cancel(w);
        let left = expire - w.now;
        let t = start_expire_timer(
            w,
            house,
            id,
            if left > TimeSpan::ZERO {
                left
            } else {
                TimeSpan::ZERO
            },
        );
        if let Some(v) =
            data_mut(w, house).and_then(|h| h.vendor_inventories.iter_mut().find(|v| v.id == id))
        {
            v.timer = t;
        }
    }
}

/// `AddItem(item)`.
pub fn add_item(w: &mut World, house: ItemId, id: u64, item: ItemId) {
    ops::internalize(w, item);
    if let Some(v) =
        data_mut(w, house).and_then(|h| h.vendor_inventories.iter_mut().find(|v| v.id == id))
    {
        v.items.push(item);
    }
}

/// `Banker.Deposit(Container, amount)`.
pub use muo_content::vendor::banker::deposit_into as deposit;

/// The expire timer's tick.
fn expire(w: &mut World, house: ItemId, id: u64) {
    if data(w, house).is_some() {
        let inv = data(w, house)
            .and_then(|h| h.vendor_inventories.iter().find(|v| v.id == id))
            .cloned();
        if let Some(inv) = inv {
            if inv.gold > 0 {
                let c = match data(w, house).and_then(|h| h.moving_crate) {
                    Some(c) => c,
                    None => {
                        let c = crate::moving_crate::create(w, house);
                        if let Some(h) = data_mut(w, house) {
                            h.moving_crate = Some(c);
                        }
                        c
                    }
                };
                deposit(w, c, inv.gold);
            }
            for item in inv.items {
                if w.item(item).is_some_and(|i| !i.deleted()) {
                    house::storage::drop_to_moving_crate(w, house, item);
                }
            }
            if let Some(v) = data_mut(w, house)
                .and_then(|h| h.vendor_inventories.iter_mut().find(|v| v.id == id))
            {
                v.gold = 0;
                v.items.clear();
            }
        }
    }
    delete(w, house, id);
}

/// `Delete()`: the items go, the inventory leaves the house.
pub fn delete(w: &mut World, house: ItemId, id: u64) {
    let Some(inv) = data(w, house)
        .and_then(|h| h.vendor_inventories.iter().find(|v| v.id == id))
        .cloned()
    else {
        return;
    };
    for item in inv.items {
        if let Some(ty) = w.item(item).map(|i| i.ty) {
            (ty.delete)(w, item);
        }
    }
    let mut t = inv.timer;
    t.cancel(w);
    if let Some(h) = data_mut(w, house) {
        h.vendor_inventories.retain(|v| v.id != id);
    }
}
