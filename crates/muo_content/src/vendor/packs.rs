//! The vendor buyback pack and the `GenericBuyInfo.DisplayCache` container.

use muo_io::Result;
use muo_primitives::TimeSpan;
use muo_server::ext::Ext;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::container::{self, CONTAINER};
use muo_server::item::{ItemType, base, ops};
use muo_server::layer::Layer;
use muo_server::serialization_ext::{ReadEntityExt, WriteEntityExt};
use muo_server::timer;
use muo_server::world::World;

use crate::items::cont::{BACKPACK, new_base_container};

// ---- VendorBuybackPack ----

/// `Server.Items.VendorBuybackPack`: items players sold to an NPC vendor until the next restock. The contents
/// are never saved.
pub static VENDOR_BUYBACK_PACK: ItemType = ItemType {
    full_name: "Server.Items.VendorBuybackPack",
    parent: Some(&BACKPACK),
    construct: Some(new_buyback_pack),
    skips_child_serialization: |_, _| true,
    serialize: |w, id, out| {
        (BACKPACK.serialize)(w, id, out);
        out.write_encoded_int(0);
    },
    deserialize: |w, id, r| {
        (BACKPACK.deserialize)(w, id, r)?;
        let _version = r.read_encoded_int()?;
        Ok(())
    },
    ..BACKPACK
};

/// `new VendorBuybackPack()`: layer ShopBuy, not movable, not visible.
pub fn new_buyback_pack(w: &mut World) -> ItemId {
    let id = new_base_container(w, &VENDOR_BUYBACK_PACK, 0xE75);
    (VENDOR_BUYBACK_PACK.set_layer)(w, id, Layer::ShopBuy);
    ops::set_movable(w, id, false);
    ops::set_visible(w, id, false);
    id
}

fn delete(w: &mut World, id: ItemId) {
    if let Some(t) = w.item(id).map(|i| i.ty) {
        (t.delete)(w, id);
    }
}

/// `AddBuyback(item, capacity)`. `DropItem` appends, so the first item is always the oldest.
pub fn add_buyback(w: &mut World, pack: ItemId, item: ItemId, capacity: i32) {
    if capacity <= 0 {
        delete(w, item);
        return;
    }
    while w
        .item(pack)
        .is_some_and(|p| p.items.len() as i32 >= capacity)
    {
        let oldest = w.item(pack).map(|p| p.items[0]);
        let Some(oldest) = oldest else { break };
        delete(w, oldest);
        // Delete() is a no-op on an already-deleted item, which would spin this loop.
        if let Some(p) = w.item_mut(pack)
            && p.items.first() == Some(&oldest)
        {
            p.items.remove(0);
        }
    }
    container::drop_item_default(w, pack, item);
}

/// `Purge()`: deletes everything in the pack.
pub fn purge(w: &mut World, pack: ItemId) {
    let items: Vec<ItemId> = w.item(pack).map(|p| p.items.clone()).unwrap_or_default();
    for i in items.into_iter().rev() {
        delete(w, i);
    }
}

// ---- DisplayCache ----

/// What `DisplayCache` adds to a `Container`: `_mobiles`.
#[derive(Default)]
pub struct DisplayCacheData {
    pub mobiles: Vec<MobileId>,
}

/// `DisplayCache._cache`.
#[derive(Default)]
pub struct DisplayCacheStatic(pub Option<ItemId>);

/// `Server.Mobiles.GenericBuyInfo+DisplayCache`: holds the entities shown in vendor windows so they are not in
/// the world.
pub static DISPLAY_CACHE: ItemType = ItemType {
    full_name: "Server.Mobiles.GenericBuyInfo+DisplayCache",
    parent: Some(&CONTAINER),
    construct: None,
    serialize: |w, id, out| {
        (CONTAINER.serialize)(w, id, out);
        out.write_i32(0);
        let mobiles = w
            .item(id)
            .and_then(|i| i.ext.get::<DisplayCacheData>())
            .map(|d| d.mobiles.clone())
            .unwrap_or_default();
        out.write_encoded_int(mobiles.len() as i32);
        for m in mobiles {
            out.write_mobile(w, Some(m));
        }
    },
    deserialize: |w, id, r| {
        (CONTAINER.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        let mobiles = r.read_mobile_list_kept(true)?;
        if let Some(i) = w.item_mut(id) {
            i.ext = Ext::new(DisplayCacheData { mobiles });
        }
        after_deserialization(w, id);
        Result::Ok(())
    },
    on_after_delete: |w, id| {
        (CONTAINER.on_after_delete)(w, id);
        clear_contents(w, id);
        let st = w.resources.get_mut::<DisplayCacheStatic>();
        if st.0 == Some(id) {
            st.0 = None;
        }
    },
    ..CONTAINER
};

fn new_display_cache(w: &mut World) -> ItemId {
    let id = container::new_container(w, &DISPLAY_CACHE, 0);
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(DisplayCacheData::default());
    }
    id
}

/// `DisplayCache.Cache`: the shared container, made again when it was deleted.
pub fn cache(w: &mut World) -> ItemId {
    if let Some(c) = w.resources.get_mut::<DisplayCacheStatic>().0
        && w.item(c).is_some_and(|i| !i.deleted())
    {
        return c;
    }
    let c = new_display_cache(w);
    w.resources.get_mut::<DisplayCacheStatic>().0 = Some(c);
    c
}

/// `DisplayCache.Store(type, obj, cache: false)`.
pub fn display_cache_store(w: &mut World, e: EntityId) {
    let c = cache(w);
    match e {
        EntityId::Item(item) => base::add_item(w, c, item),
        EntityId::Mobile(m) => {
            if let Some(d) = w
                .item_mut(c)
                .and_then(|i| i.ext.get_mut::<DisplayCacheData>())
            {
                d.mobiles.push(m);
            }
        }
    }
}

/// `ClearContents()`.
fn clear_contents(w: &mut World, id: ItemId) {
    let mobiles = w
        .item_mut(id)
        .and_then(|i| i.ext.get_mut::<DisplayCacheData>())
        .map(|d| std::mem::take(&mut d.mobiles))
        .unwrap_or_default();
    for m in mobiles {
        if let Some(t) = w.mobile(m).map(|m| m.ty) {
            (t.delete)(w, m);
        }
    }
    let items: Vec<ItemId> = w.item(id).map(|i| i.items.clone()).unwrap_or_default();
    for i in items.into_iter().rev() {
        delete(w, i);
    }
}

/// `[AfterDeserialization]`.
fn after_deserialization(w: &mut World, id: ItemId) {
    let is_cache = w.resources.get::<DisplayCacheStatic>().and_then(|s| s.0) == Some(id);
    let empty = w.item(id).is_none_or(|i| {
        i.items.is_empty()
            && i.ext
                .get::<DisplayCacheData>()
                .is_none_or(|d| d.mobiles.is_empty())
    });
    if is_cache && empty {
        return;
    }
    timer::start(w, TimeSpan::ZERO, TimeSpan::ZERO, 1, move |w| {
        clear_contents(w, id);
        if w.resources.get_mut::<DisplayCacheStatic>().0.is_none() {
            w.resources.get_mut::<DisplayCacheStatic>().0 = Some(id);
        } else if let Some(t) = w.item(id).map(|i| i.ty) {
            (t.delete)(w, id);
        }
    });
}
