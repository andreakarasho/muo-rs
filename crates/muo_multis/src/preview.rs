//! `PreviewHouse`: the multi shown while a player decides on a house placement.

use muo_primitives::{Point3D, TimeSpan};
use muo_server::World;
use muo_server::ids::ItemId;
use muo_server::item::multi::{self, BASE_MULTI};
use muo_server::item::{ItemType, ops};
use muo_server::serialization_ext::{ReadEntityExt, WriteEntityExt};
use muo_server::timer::{self, TimerHandle};
use muo_server::type_registry::TypeRegistry;

/// What `PreviewHouse` adds.
#[derive(Clone, Debug, Default)]
pub struct PreviewData {
    pub components: Option<Vec<ItemId>>,
    pub timer: Option<TimerHandle>,
}

fn data(w: &World, id: ItemId) -> PreviewData {
    w.item(id)
        .and_then(|i| i.ext.get::<PreviewData>())
        .cloned()
        .unwrap_or_default()
}

fn set_data(w: &mut World, id: ItemId, d: PreviewData) {
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(d);
    }
}

fn delete(w: &mut World, id: ItemId) {
    if let Some(ty) = w.item(id).map(|i| i.ty) {
        (ty.delete)(w, id);
    }
}

/// `new PreviewHouse(multiID)`.
pub fn new_preview_house(w: &mut World, multi_id: i32) -> ItemId {
    let id = multi::new_base_multi(w, &PREVIEW_HOUSE, multi_id);
    set_data(w, id, PreviewData::default());
    let entries: Vec<_> = multi::components(w, id).list.clone();
    let (loc, map) = w
        .item(id)
        .map(|i| (i.location, i.map))
        .unwrap_or((Point3D::ZERO, None));

    let mut components: Option<Vec<ItemId>> = None;
    for entry in entries.iter().skip(1) {
        if entry.flags.is_empty() {
            let Some(item) = muo_content::factory::make_item(w, "Server.Items.Static") else {
                continue;
            };
            if let Some(ty) = w.item(item).map(|i| i.ty) {
                (ty.set_item_id)(w, item, i32::from(entry.item_id));
            }
            let p = Point3D::new(
                loc.x + i32::from(entry.offset_x),
                loc.y + i32::from(entry.offset_y),
                loc.z + i32::from(entry.offset_z),
            );
            ops::move_to_world_map(w, item, p, map);
            components.get_or_insert_with(Vec::new).push(item);
        }
    }
    let t = timer::start(w, TimeSpan::from_secs(20), TimeSpan::ZERO, 1, move |w| {
        delete(w, id)
    });
    set_data(
        w,
        id,
        PreviewData {
            components,
            timer: Some(t),
        },
    );
    id
}

/// `Server.Multis.PreviewHouse`.
pub static PREVIEW_HOUSE: ItemType = ItemType {
    full_name: "Server.Multis.PreviewHouse",
    parent: Some(&BASE_MULTI),
    construct: None,
    skip_serialization: |_, _| true,
    on_location_change: |w, id, old| {
        (BASE_MULTI.on_location_change)(w, id, old);
        let Some(list) = data(w, id).components else {
            return;
        };
        let l = w.item(id).map_or(Point3D::ZERO, |i| i.location);
        let map = w.item(id).and_then(|i| i.map);
        let (x, y, z) = (l.x - old.x, l.y - old.y, l.z - old.z);
        for item in list {
            let p = w.item(item).map_or(Point3D::ZERO, |i| i.location);
            ops::move_to_world_map(w, item, Point3D::new(p.x + x, p.y + y, p.z + z), map);
        }
    },
    on_map_change: |w, id| {
        (BASE_MULTI.on_map_change)(w, id);
        let Some(list) = data(w, id).components else {
            return;
        };
        let map = w.item(id).and_then(|i| i.map);
        for item in list {
            ops::set_map(w, item, map);
        }
    },
    on_delete: |w, id| {
        (BASE_MULTI.on_delete)(w, id);
        if let Some(list) = data(w, id).components {
            for item in list {
                delete(w, item);
            }
        }
    },
    on_after_delete: |w, id| {
        if let Some(mut t) = data(w, id).timer {
            t.cancel(w);
        }
        (BASE_MULTI.on_after_delete)(w, id);
    },
    serialize: |w, id, out| {
        (BASE_MULTI.serialize)(w, id, out);
        out.write_i32(0); // version
        let list = data(w, id).components.unwrap_or_default();
        out.write_item_list(w, &list);
    },
    deserialize: |w, id, r| {
        (BASE_MULTI.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        let list = r.read_item_list(w)?;
        set_data(
            w,
            id,
            PreviewData {
                components: Some(list),
                timer: None,
            },
        );
        Ok(())
    },
    ..BASE_MULTI
};

pub fn register(types: &mut TypeRegistry) {
    types.register_item(&PREVIEW_HOUSE, &[]);
}
