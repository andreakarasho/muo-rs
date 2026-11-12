//! A temporary, invisible item an effect is sent from or to.

use std::collections::VecDeque;

use muo_primitives::{Point3D, TimeSpan};

use crate::effects::Targeted;
use crate::ids::ItemId;
use crate::item::ItemType;
use crate::item::item_type::ITEM_BASE;
use crate::item::ops;
use crate::map::MapId;
use crate::timer;
use crate::world::World;

/// `Server.Items.EffectItem`.
pub static EFFECT_ITEM: ItemType = ItemType {
    full_name: "Server.Items.EffectItem",
    parent: Some(&ITEM_BASE),
    construct: None,
    decays: |_, _| true,
    skip_serialization: |_, _| true,
    serialize: |w, id, wr| {
        (ITEM_BASE.serialize)(w, id, wr);
        wr.write_i32(0); // version
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        r.read_i32()?;
        Ok(())
    },
    ..ITEM_BASE
};

/// `EffectItem._free`.
#[derive(Default)]
struct Free(VecDeque<ItemId>);

/// `EffectItem.DefaultDuration`.
pub const DEFAULT_DURATION: TimeSpan = TimeSpan::from_secs(5);

/// `EffectItem.Create(p, map, duration)`.
pub fn create(w: &mut World, p: Point3D, map: Option<MapId>, duration: TimeSpan) -> ItemId {
    let mut found = None;
    while let Some(free) = w.resources.get_mut::<Free>().0.pop_front() {
        if !ops::is_deleted(w, free) && w.item(free).is_some_and(|i| i.map == Some(MapId::INTERNAL))
        {
            found = Some(free);
            break;
        }
    }

    let id = match found {
        Some(id) => {
            if let Some(i) = w.item_mut(id) {
                i.item_id = 1;
            }
            id
        }
        None => {
            let id = ops::new_item_with_id(w, &EFFECT_ITEM, 1);
            ops::set_movable(w, id, false);
            id
        }
    };

    ops::move_to_world_map(w, id, p, map);
    begin_free(w, id, duration);
    id
}

/// `BeginFree(duration)`: the `FreeTimer`.
pub fn begin_free(w: &mut World, id: ItemId, duration: TimeSpan) {
    timer::start(w, duration, TimeSpan::ZERO, 1, move |w| {
        ops::internalize(w, id);
        w.resources.get_mut::<Free>().0.push_back(id);
    });
}

/// `EffectItem.Create(...)` as an effect endpoint.
pub fn create_targeted(
    w: &mut World,
    p: Point3D,
    map: Option<MapId>,
    duration: TimeSpan,
) -> Targeted {
    Targeted::Entity(create(w, p, map, duration).into())
}
