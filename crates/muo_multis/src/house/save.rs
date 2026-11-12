//! `BaseHouse` `Serialize`/`Deserialize` (version 15, every older version read) and the constructor.
//!
//! The record is `BaseMulti`'s part (the `Item` record and an int 0), the house version int, then the fields below.
//! The subclasses add `[SerializationGenerator(0, false)]`'s int 0 on top (see `classes.rs`).

use muo_content::factory::item_is;
use muo_io::{GenericReader, GenericWriter, Result};
use muo_primitives::{DateTime, Point3D, Serial, TimeSpan, utility};
use muo_server::World;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::ItemType;
use muo_server::item::multi::{self, BASE_MULTI};
use muo_server::item::ops;
use muo_server::serialization_ext::{ReadEntityExt, WriteEntityExt};
use muo_server::timer;

use super::{
    DecayLevel, HouseData, RelocatedEntity, SecureInfo, SecureLevel, VendorInventory, class, d,
    data, data_mut, ensure_data, statics, storage,
};

/// The version `BaseHouse` writes.
pub const VERSION: i32 = 15;

/// `new BaseHouse(multiID, owner, maxLockDown, maxSecure)` for a class of the table `ty`.
pub fn new_base_house(
    w: &mut World,
    ty: &'static ItemType,
    multi_id: i32,
    owner: Option<MobileId>,
    max_lock_down: i32,
    max_secure: i32,
) -> ItemId {
    let id = multi::new_base_multi(w, ty, multi_id);
    ensure_data(w, id);
    statics(w).all_houses.push(id);
    let ban = (class(w, id).base_ban_location)(w, id);
    let now = w.now;
    if let Some(h) = data_mut(w, id) {
        h.initialized = true;
        h.last_refreshed = now;
        h.built_on = now;
        h.last_traded = DateTime::MIN;
        h.owner = owner;
        h.max_lock_downs = max_lock_down;
        h.max_secures = max_secure;
        h.relative_ban_location = ban;
    }
    if let Some(o) = owner {
        statics(w).table.entry(o).or_default().push(id);
    }
    ops::set_movable(w, id, false);
    id
}

fn tidy_items(w: &World, list: &[ItemId]) -> Vec<ItemId> {
    list.iter()
        .copied()
        .filter(|&i| w.item(i).is_some_and(|x| !x.deleted()))
        .collect()
}

fn tidy_mobiles(w: &World, list: &[MobileId]) -> Vec<MobileId> {
    list.iter()
        .copied()
        .filter(|&m| w.mobile(m).is_some_and(|x| !x.deleted))
        .collect()
}

fn serial_of(w: &World, e: EntityId) -> Serial {
    match e {
        EntityId::Item(i) => w
            .item(i)
            .filter(|x| !x.deleted())
            .map_or(Serial::MINUS_ONE, |x| x.serial),
        EntityId::Mobile(m) => w
            .mobile(m)
            .filter(|x| !x.deleted)
            .map_or(Serial::MINUS_ONE, |x| x.serial),
    }
}

/// `VendorInventory.Serialize(writer)`.
pub fn write_inventory(w: &World, out: &mut dyn GenericWriter, v: &VendorInventory) {
    out.write_encoded_int(1); // version
    out.write_mobile(w, v.owner);
    out.write_string(&v.vendor_name);
    out.write_string(&v.shop_name);
    out.write_item_list(w, &tidy_items(w, &v.items));
    out.write_i32(v.gold);
    out.write_anchored_time_ticks(v.expire_time.ticks());
}

pub fn serialize(w: &World, id: ItemId, out: &mut dyn GenericWriter) {
    (BASE_MULTI.serialize)(w, id, out);
    out.write_i32(VERSION);
    let h = d(w, id);

    if !super::decay::dynamic_enabled(w) {
        out.write_i32(-1);
    } else {
        out.write_i32(h.current_stage as i32);
        out.write_date_time_ticks(h.next_decay_stage.ticks());
    }

    out.write_point3d(h.relative_ban_location);

    out.write_item_list(w, &tidy_items(w, &h.vendor_rental_contracts));
    out.write_mobile_list(w, &tidy_mobiles(w, &h.internalized_vendors));

    out.write_encoded_int(h.relocated_entities.len() as i32);
    for r in &h.relocated_entities {
        out.write_point3d(r.relative_location);
        out.write_serial(serial_of(w, r.entity));
    }

    out.write_encoded_int(h.vendor_inventories.len() as i32);
    for v in &h.vendor_inventories {
        write_inventory(w, out, v);
    }

    out.write_date_time_ticks(h.last_refreshed.ticks());
    out.write_bool(h.restrict_decay);
    out.write_i32(h.visits);
    out.write_i32(h.price);
    out.write_mobile_list(w, &h.access);
    out.write_date_time_ticks(h.built_on.ticks());
    out.write_date_time_ticks(h.last_traded.ticks());
    out.write_item_list(w, &tidy_items(w, &h.addons));

    out.write_i32(h.secures.len() as i32);
    for s in &h.secures {
        out.write_item(w, Some(s.item));
        out.write_u8(s.level as u8);
    }

    out.write_bool(h.public);
    out.write_mobile(w, h.owner);
    out.write_mobile_list(w, &tidy_mobiles(w, &h.co_owners));
    out.write_mobile_list(w, &tidy_mobiles(w, &h.friends));
    out.write_mobile_list(w, &tidy_mobiles(w, &h.bans));
    out.write_item(w, h.sign);
    out.write_item(w, h.trash);
    out.write_item_list(w, &tidy_items(w, &h.doors));
    out.write_item_list(w, &tidy_items(w, &h.lock_downs));
    out.write_i32(h.max_lock_downs);
    out.write_i32(h.max_secures);
}

/// `new VendorInventory(house, reader)`.
fn read_inventory(
    w: &mut World,
    house: ItemId,
    r: &mut dyn GenericReader,
) -> Result<VendorInventory> {
    let version = r.read_encoded_int()?;
    let owner = r.read_mobile(w)?;
    let vendor_name = r.read_string()?.unwrap_or_default();
    let shop_name = r.read_string()?.unwrap_or_default();
    let items = r.read_item_list(w)?;
    let gold = r.read_i32()?;
    let expire_time = if version >= 1 {
        DateTime::from_ticks(r.read_anchored_time_ticks()?)
    } else {
        DateTime::from_ticks(r.read_delta_time_ticks(w.now.ticks())?)
    };
    Ok(crate::vendor_inventory::restore(
        w,
        house,
        owner,
        vendor_name,
        shop_name,
        items,
        gold,
        expire_time,
    ))
}

pub fn deserialize(w: &mut World, id: ItemId, r: &mut dyn GenericReader) -> Result<()> {
    (BASE_MULTI.deserialize)(w, id, r)?;
    ensure_data(w, id);
    statics(w).all_houses.push(id);
    let version = r.read_i32()?;
    let mut loaded_dynamic_decay = false;
    let mut h = HouseData::default();

    if version >= 15 {
        let stage = r.read_i32()?;
        if stage != -1 {
            h.current_stage = DecayLevel::from_i32(stage);
            h.next_decay_stage = DateTime::from_ticks(r.read_date_time_ticks()?);
            loaded_dynamic_decay = true;
        }
    }
    if version >= 14 {
        h.relative_ban_location = r.read_point3d()?;
    }
    if version >= 12 {
        h.vendor_rental_contracts = r
            .read_item_list(w)?
            .into_iter()
            .filter(|&i| item_is(w, i, "Server.Items.VendorRentalContract"))
            .collect();
        h.internalized_vendors = r.read_mobile_list(w)?;
        let n = r.read_encoded_int()?;
        for _ in 0..n {
            let rel = r.read_point3d()?;
            if let Some(e) = r.read_entity(w)? {
                h.relocated_entities.push(RelocatedEntity {
                    entity: e,
                    relative_location: rel,
                });
            }
        }
    }
    // The inventories need the house to exist with its data (they start timers that refer to it), so they are
    // read into a list first and attached below.
    let mut inventories_to_read = 0;
    if version >= 12 {
        inventories_to_read = r.read_encoded_int()?;
        // stash what was read so far, the inventories are read right after
        set_data(w, id, h.clone());
        for _ in 0..inventories_to_read {
            let inv = read_inventory(w, id, r)?;
            h.vendor_inventories.push(inv);
        }
    }
    let _ = inventories_to_read;
    if version >= 11 {
        h.last_refreshed = DateTime::from_ticks(r.read_date_time_ticks()?);
        h.restrict_decay = r.read_bool()?;
    }
    if version >= 9 {
        h.visits = r.read_i32()?;
    }
    if version >= 8 {
        h.price = r.read_i32()?;
    }
    if version >= 7 {
        h.access = r.read_mobile_list(w)?;
    }
    if version >= 6 {
        h.built_on = DateTime::from_ticks(r.read_date_time_ticks()?);
        h.last_traded = DateTime::from_ticks(r.read_date_time_ticks()?);
    }
    if version >= 4 {
        h.addons = r.read_item_list(w)?;
    }
    if version >= 3 {
        let count = r.read_i32()?;
        for _ in 0..count {
            let item = r.read_item(w)?;
            let level = SecureLevel::from_u8(r.read_u8()?);
            if let Some(item) = item.filter(|&i| ops::is_container(w, i)) {
                ops::set_secure(w, item, true);
                h.secures.push(SecureInfo { item, level });
            }
        }
    }
    if version >= 2 {
        h.public = r.read_bool()?;
    }
    if version == 1 || (1..13).contains(&version) {
        r.read_point3d()?; // house ban location
    }

    // case 0
    if version < 14 {
        h.relative_ban_location = (class(w, id).base_ban_location)(w, id);
    }
    if version < 4 {
        h.addons = Vec::new();
    }
    if version < 8 {
        h.price = (class(w, id).default_price)(w, id);
    }
    h.owner = r.read_mobile(w)?;
    if version < 5 {
        let count = r.read_i32()?;
        for _ in 0..count {
            r.read_rect2d()?;
        }
    }
    set_data(w, id, h.clone());
    storage::update_region(w, id);
    h.region = data(w, id).and_then(|x| x.region);

    h.co_owners = r.read_mobile_list(w)?;
    h.friends = r.read_mobile_list(w)?;
    h.bans = r.read_mobile_list(w)?;
    h.sign = r
        .read_item(w)?
        .filter(|&i| item_is(w, i, "Server.Multis.HouseSign"));
    h.trash = r
        .read_item(w)?
        .filter(|&i| item_is(w, i, "Server.Items.TrashBarrel"));
    h.doors = r.read_item_list(w)?;
    h.lock_downs = r.read_item_list(w)?;
    for &i in &h.lock_downs {
        ops::set_locked_down(w, i, true);
    }
    for &i in &h.vendor_rental_contracts {
        ops::set_locked_down(w, i, true);
    }
    if version < 3 {
        for i in r.read_item_list(w)? {
            if ops::is_container(w, i) {
                ops::set_secure(w, i, true);
                h.secures.push(SecureInfo {
                    item: i,
                    level: SecureLevel::CoOwners,
                });
            }
        }
    }
    h.max_lock_downs = r.read_i32()?;
    h.max_secures = r.read_i32()?;
    h.initialized = true;

    let (no_map, zero) = w
        .item(id)
        .map(|i| {
            (
                i.map.is_none() || i.map == Some(muo_server::map::MapId::INTERNAL),
                i.location == Point3D::ZERO,
            )
        })
        .unwrap_or((false, false));
    let owner = h.owner;
    let (relocated, friends, co_owners) = (
        h.relocated_entities.len(),
        h.friends.len(),
        h.co_owners.len(),
    );
    // `m_Table` and the rest of the record are in place before anything below reads them
    set_data(w, id, h);
    if no_map
        && zero
        && let Some(ty) = w.item(id).map(|i| i.ty)
    {
        (ty.delete)(w, id);
    }
    if let Some(o) = owner {
        statics(w).table.entry(o).or_default().push(id);
    }

    if version <= 1 {
        super::doors::change_sign_type(w, id, 0xBD2); // private house, plain brass sign
    }
    if version < 10 {
        timer::start(w, TimeSpan::ZERO, TimeSpan::ZERO, 1, move |w| {
            storage::fix_lockdowns(w, id)
        });
    }
    if version < 11 {
        let extra = TimeSpan::from_hours(24).ticks() as f64 * utility::random_double(&mut w.rng);
        let t = w.now + TimeSpan::from_ticks(extra as i64);
        if let Some(h) = data_mut(w, id) {
            h.last_refreshed = t;
        }
    }
    if super::decay::dynamic_enabled(w) && !loaded_dynamic_decay {
        let mut old = super::decay::old_decay_level(w, id);
        if old == DecayLevel::DemolitionPending {
            old = DecayLevel::Collapsed;
        }
        super::decay::set_dynamic_decay(w, id, old);
    }
    if !super::decay::check_decay(w, id) {
        if relocated > 0 {
            timer::start(w, TimeSpan::ZERO, TimeSpan::ZERO, 1, move |w| {
                storage::restore_relocated_entities(w, id)
            });
        }
        if owner.is_none() && friends == 0 && co_owners == 0 {
            timer::start(w, TimeSpan::from_secs(10), TimeSpan::ZERO, 1, move |w| {
                if let Some(ty) = w.item(id).map(|i| i.ty) {
                    (ty.delete)(w, id);
                }
            });
        }
    }
    Ok(())
}

fn set_data(w: &mut World, id: ItemId, h: HouseData) {
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(h);
    }
}

/// The record's tail every concrete house class writes: `[SerializationGenerator(0, false)]`'s int 0.
pub fn write_class_version(out: &mut dyn GenericWriter) {
    out.write_i32(0);
}

/// Reads what [`write_class_version`] wrote.
pub fn read_class_version(r: &mut dyn GenericReader) -> Result<()> {
    r.read_i32()?;
    Ok(())
}
