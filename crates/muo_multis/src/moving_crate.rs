//! `MovingCrate` and `PackingBox`.

use muo_primitives::{Point3D, Rectangle2D, TimeSpan};
use muo_server::World;
use muo_server::ids::ItemId;
use muo_server::item::container::{self, CONTAINER};
use muo_server::item::{ItemType, TotalType, ops};
use muo_server::lr_reason::LRReason;
use muo_server::mobile::AccessLevel;
use muo_server::serialization_ext::{ReadEntityExt, WriteEntityExt};
use muo_server::timer::{self, TimerHandle};
use muo_server::type_registry::TypeRegistry;

use muo_content::items::cont::BASE_CONTAINER;

use crate::house::{self, access::msg};

const MAX_ITEMS_PER_SUBCONTAINER: usize = 20;
const ROWS: usize = 3;
const COLUMNS: usize = 5;
const HORIZONTAL_SPACING: i32 = 25;
const VERTICAL_SPACING: i32 = 25;

/// What `MovingCrate` adds: the `[SerializableField]` house and the internalize timer.
#[derive(Clone, Debug, Default)]
pub struct MovingCrateData {
    pub house: Option<ItemId>,
    pub internalize_timer: Option<TimerHandle>,
}

fn data(w: &World, id: ItemId) -> MovingCrateData {
    w.item(id)
        .and_then(|i| i.ext.get::<MovingCrateData>())
        .cloned()
        .unwrap_or_default()
}

fn data_mut(w: &mut World, id: ItemId) -> Option<&mut MovingCrateData> {
    w.item_mut(id)?.ext.get_mut::<MovingCrateData>()
}

/// `new MovingCrate(house)`.
pub fn create(w: &mut World, house: ItemId) -> ItemId {
    let id = container::new_container(w, &MOVING_CRATE, 0xE3D);
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(MovingCrateData {
            house: Some(house),
            internalize_timer: None,
        });
    }
    muo_server::item::base::set_hue(w, id, 0x8A5);
    ops::set_movable(w, id, false);
    id
}

fn is_packing_box(w: &World, i: ItemId) -> bool {
    w.item(i).is_some_and(|x| x.ty.is(&PACKING_BOX))
}

fn children(w: &World, id: ItemId) -> Vec<ItemId> {
    w.item(id).map(|i| i.items.clone()).unwrap_or_default()
}

/// `DropItem(dropped)` of the crate (virtual).
pub fn drop_item(w: &mut World, id: ItemId, dropped: ItemId) {
    let Some(ty) = w.item(id).map(|i| i.ty) else {
        return;
    };
    (ty.drop_item)(w, id, dropped);
}

fn crate_drop_item(w: &mut World, id: ItemId, dropped: ItemId) {
    // 1. Try to stack the item
    for item in children(w, id) {
        if is_packing_box(w, item) {
            for sub in children(w, item) {
                let Some(ty) = w.item(sub).map(|i| i.ty) else {
                    continue;
                };
                if !ty.is(&CONTAINER) && (ty.stack_with)(w, sub, None, dropped, false) {
                    return;
                }
            }
        }
    }
    // 2. Try to drop the item into an existing container
    for item in children(w, id) {
        if is_packing_box(w, item) && children(w, item).len() < MAX_ITEMS_PER_SUBCONTAINER {
            (PACKING_BOX.drop_item)(w, item, dropped);
            return;
        }
    }
    // 3. Drop the item into a new container
    let sub = container::new_container(w, &PACKING_BOX, 0x9A8);
    ops::set_movable(w, sub, false);
    (PACKING_BOX.drop_item)(w, sub, dropped);

    let location = free_location(w, id);
    if location != Point3D::ZERO {
        if let Some(ty) = w.item(id).map(|i| i.ty) {
            (ty.add_item)(w, id, sub);
        }
        ops::set_location(w, sub, location);
    } else {
        (CONTAINER.drop_item)(w, id, sub);
    }
}

fn free_location(w: &World, id: ItemId) -> Point3D {
    let Some(ty) = w.item(id).map(|i| i.ty) else {
        return Point3D::ZERO;
    };
    let bounds = (ty.bounds)(w, id);
    let mut positions = [[false; COLUMNS]; ROWS];
    for item in children(w, id) {
        if is_packing_box(w, item) {
            let p = w.item(item).map_or(Point3D::ZERO, |i| i.location);
            let i = ((p.y - bounds.y()) / VERTICAL_SPACING).clamp(0, ROWS as i32 - 1) as usize;
            let j = ((p.x - bounds.x()) / HORIZONTAL_SPACING).clamp(0, COLUMNS as i32 - 1) as usize;
            positions[i][j] = true;
        }
    }
    for (i, row) in positions.iter().enumerate() {
        for (j, taken) in row.iter().enumerate() {
            if !taken {
                return Point3D::new(
                    bounds.x() + j as i32 * HORIZONTAL_SPACING,
                    bounds.y() + i as i32 * VERTICAL_SPACING,
                    0,
                );
            }
        }
    }
    Point3D::ZERO
}

fn crate_house(w: &World, id: ItemId) -> Option<ItemId> {
    data(w, id).house.filter(|&h| !house::deleted(w, h))
}

/// `RestartTimer()`.
pub fn restart_timer(w: &mut World, id: ItemId) {
    if let Some(mut t) = data_mut(w, id).and_then(|d| d.internalize_timer.take()) {
        t.cancel(w);
    }
    let h = timer::start(w, TimeSpan::from_mins(5), TimeSpan::ZERO, 1, move |w| {
        hide(w, id)
    });
    if let Some(d) = data_mut(w, id) {
        d.internalize_timer = Some(h);
    }
}

/// `Hide()`.
pub fn hide(w: &mut World, id: ItemId) {
    if w.item(id).is_none() {
        return;
    }
    if let Some(mut t) = data_mut(w, id).and_then(|d| d.internalize_timer.take()) {
        t.cancel(w);
    }
    for item in children(w, id) {
        if is_packing_box(w, item) && children(w, item).is_empty() {
            delete(w, item);
        }
    }
    if ops::get_total(w, id, TotalType::Items) == 0 {
        delete(w, id);
    } else {
        ops::internalize(w, id);
    }
}

fn delete(w: &mut World, id: ItemId) {
    if let Some(ty) = w.item(id).map(|i| i.ty) {
        (ty.delete)(w, id);
    }
}

/// `Server.Multis.MovingCrate`.
pub static MOVING_CRATE: ItemType = ItemType {
    full_name: "Server.Multis.MovingCrate",
    parent: Some(&CONTAINER),
    construct: None,
    label_number: |w, id| {
        crate::template_props::item_int(w, id, "labelNumber", CONTAINER.label_number)
    },
    default_max_items: |_, _| 0,
    default_max_weight: |_, _| 0,
    is_deco_container: |_, _| false,
    drop_item: crate_drop_item,
    check_hold: |w, id, m, item, message, check_items, plus_items, plus_weight| {
        if let Some(m) = m
            && w.mobile(m)
                .is_some_and(|x| x.access_level < AccessLevel::GameMaster)
        {
            msg(w, m, 1061145); // You cannot place items into a house moving crate.
            return false;
        }
        (CONTAINER.check_hold)(
            w,
            id,
            m,
            item,
            message,
            check_items,
            plus_items,
            plus_weight,
        )
    },
    check_lift: |w, id, from, item, reject: &mut LRReason| {
        crate_house(w, id).is_some()
            && (CONTAINER.check_lift)(w, id, from, item, reject)
            && crate_house(w, id).is_some_and(|h| house::access::is_owner(w, h, from))
    },
    check_item_use: |w, id, from, item| {
        crate_house(w, id).is_some()
            && (CONTAINER.check_item_use)(w, id, from, item)
            && crate_house(w, id).is_some_and(|h| house::access::is_owner(w, h, from))
    },
    on_item_removed: |w, id, item| {
        (CONTAINER.on_item_removed)(w, id, item);
        if ops::get_total(w, id, TotalType::Items) == 0 {
            delete(w, id);
        }
    },
    on_after_delete: |w, id| {
        (CONTAINER.on_after_delete)(w, id);
        let d = data(w, id);
        if let Some(h) = d.house
            && let Some(hd) = house::data_mut(w, h)
            && hd.moving_crate == Some(id)
        {
            hd.moving_crate = None;
        }
        if let Some(mut t) = d.internalize_timer {
            t.cancel(w);
        }
    },
    serialize: |w, id, out| {
        (CONTAINER.serialize)(w, id, out);
        out.write_encoded_int(0); // version
        out.write_item(w, data(w, id).house);
    },
    deserialize: |w, id, r| {
        (CONTAINER.deserialize)(w, id, r)?;
        let _version = r.read_encoded_int()?;
        let h = r.read_item(w)?.filter(|&h| house::is_house(w, h));
        if let Some(i) = w.item_mut(id) {
            i.ext.insert(MovingCrateData {
                house: h,
                internalize_timer: None,
            });
        }
        // [AfterDeserialization]: the house record may not be read yet, so link on the first tick
        timer::start(w, TimeSpan::ZERO, TimeSpan::ZERO, 1, move |w| {
            match data(w, id).house {
                Some(h) if !house::deleted(w, h) => {
                    if let Some(hd) = house::data_mut(w, h) {
                        hd.moving_crate = Some(id);
                    }
                    hide(w, id);
                }
                _ => delete(w, id),
            }
        });
        Ok(())
    },
    ..CONTAINER
};

/// `PackingBox`' bounds.
const PACKING_BOX_BOUNDS: Rectangle2D = Rectangle2D::new(16, 51, 168, 73);

/// `Server.Multis.PackingBox`.
pub static PACKING_BOX: ItemType = ItemType {
    full_name: "Server.Multis.PackingBox",
    parent: Some(&BASE_CONTAINER),
    construct: None,
    label_number: |w, id| {
        crate::template_props::item_int(w, id, "labelNumber", BASE_CONTAINER.label_number)
    },
    default_gump_id: |_, _| 0x4B,
    default_drop_sound: |_, _| 0x42,
    bounds: |_, _| PACKING_BOX_BOUNDS,
    default_max_items: |_, _| 0,
    default_max_weight: |_, _| 0,
    send_cant_store_message: |w, _id, to, _item| {
        msg(w, to, 1061145); // You cannot place items into a house moving crate.
    },
    on_item_removed: |w, id, item| {
        (BASE_CONTAINER.on_item_removed)(w, id, item);
        let bounced = w.item(item).is_some_and(|i| i.bounce.is_some());
        if !bounced && ops::get_total(w, id, TotalType::Items) == 0 {
            delete(w, id);
        }
    },
    on_item_bounce_cleared: |w, id, item| {
        (BASE_CONTAINER.on_item_bounce_cleared)(w, id, item);
        if ops::get_total(w, id, TotalType::Items) == 0 {
            delete(w, id);
        }
    },
    serialize: |w, id, out| {
        (BASE_CONTAINER.serialize)(w, id, out);
        out.write_encoded_int(0);
    },
    deserialize: |w, id, r| {
        (BASE_CONTAINER.deserialize)(w, id, r)?;
        let _version = r.read_encoded_int()?;
        Ok(())
    },
    ..BASE_CONTAINER
};

pub fn register(types: &mut TypeRegistry) {
    types.register_item(&MOVING_CRATE, &[]);
    types.register_item(&PACKING_BOX, &[]);
}
