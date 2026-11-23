//! Furniture containers (`Armoire`, `FancyArmoire`, `DynamicFurniture`, `FurnitureTimer`)
//! and the `IDyable` of `BaseBagBall` (`Bag` has the same `Dye`).

use std::collections::HashMap;

use muo_content::items::cont::{
    BACKPACK, BAG, BASE_CONTAINER, LARGE_BAG_BALL, SMALL_BAG_BALL, new_base_container,
};
use muo_content::items::interfaces::register_dyable;
use muo_primitives::TimeSpan;
use muo_server::World;
use muo_server::ids::ItemId;
use muo_server::ids::MobileId;
use muo_server::item::{ItemDelta, ItemType, ops};
use muo_server::timer::{self, TimerHandle};
use muo_server::type_registry::{Phase, TypeRegistry};

use crate::simple::item_id;

/// `DynamicFurniture._table`: the open containers and their `FurnitureTimer`.
#[derive(Default)]
struct Table(HashMap<ItemId, TimerHandle>);

fn swap(w: &mut World, c: ItemId, pairs: &[(i32, i32)]) {
    let cur = w.item(c).map_or(0, |i| i.item_id);
    if let Some(&(_, to)) = pairs.iter().find(|(from, _)| *from == cur) {
        item_id(w, c, to);
    }
}

/// `DynamicFurniture.Open(c, m)`: false when the container was open (it is closed and re-sent instead).
pub fn open(w: &mut World, c: ItemId, m: MobileId) -> bool {
    if w.resources
        .get::<Table>()
        .is_some_and(|t| t.0.contains_key(&c))
    {
        ops::send_remove_packet(w, c);
        close(w, c);
        ops::delta(w, c, ItemDelta::UPDATE);
        if let Some(ty) = w.item(c).map(|i| i.ty) {
            (ty.process_delta)(w, c);
        }
        return false;
    }
    // `c is Armoire or FancyArmoire`: the only classes that call this
    let h = timer::start(
        w,
        TimeSpan::from_millis(500),
        TimeSpan::from_millis(500),
        0,
        move |w| {
            // FurnitureTimer.OnTick
            let (item_map, loc) = match w.item(c) {
                Some(i) => (i.map, ops::get_world_location(w, c)),
                None => return close(w, c),
            };
            let mob_map = w.mobile(m).and_then(|x| x.map);
            if mob_map != item_map || !muo_server::mobile::base::in_range_point3d(w, m, loc, 3) {
                close(w, c);
            }
        },
    );
    w.resources.get_mut::<Table>().0.insert(c, h);
    swap(
        w,
        c,
        &[
            (0xA4D, 0xA4C),
            (0xA4F, 0xA4E),
            (0xA51, 0xA50),
            (0xA53, 0xA52),
        ],
    );
    true
}

/// `DynamicFurniture.Close(c)`.
pub fn close(w: &mut World, c: ItemId) {
    if let Some(mut h) = w.resources.get_mut::<Table>().0.remove(&c) {
        h.cancel(w);
    }
    if w.item(c).is_some() {
        swap(
            w,
            c,
            &[
                (0xA4C, 0xA4D),
                (0xA4E, 0xA4F),
                (0xA50, 0xA51),
                (0xA52, 0xA53),
            ],
        );
    }
}

macro_rules! armoire {
    ($(#[$m:meta])* $name:ident = $full:literal) => {
        $(#[$m])*
        pub static $name: ItemType = ItemType {
            full_name: $full,
            parent: Some(&BASE_CONTAINER),
            construct: Some(|w| {
                let id = new_base_container(w, &$name, 0);
                crate::simple::own_template(w, id, $full)
            }),
            default_weight: crate::class_props::default_weight,
            display_to: |w, id, m| {
                if open(w, id, m) {
                    (BASE_CONTAINER.display_to)(w, id, m);
                }
            },
            serialize: |w, id, out| {
                (BASE_CONTAINER.serialize)(w, id, out);
                out.write_i32(0);
            },
            deserialize: |w, id, r| {
                (BASE_CONTAINER.deserialize)(w, id, r)?;
                let _version = r.read_i32()?;
                close(w, id); // [AfterDeserialization]
                Ok(())
            },
            ..BASE_CONTAINER
        };
    };
}

armoire!(
    /// `Server.Items.Armoire`.
    ARMOIRE = "Server.Items.Armoire"
);
armoire!(
    /// `Server.Items.FancyArmoire`.
    FANCY_ARMOIRE = "Server.Items.FancyArmoire"
);

fn configure(w: &mut World) {
    // `BaseBagBall.Dye` / `Bag.Dye` / `Backpack.Dye`: `if (Deleted) return false; Hue = sender.DyedHue; return true;`
    for ty in [&SMALL_BAG_BALL, &LARGE_BAG_BALL, &BAG, &BACKPACK] {
        register_dyable(w, ty, |w, id, _from, tub| {
            if ops::is_deleted(w, id) {
                return false;
            }
            let hue = muo_content::items::clothing::tub_dyed_hue(w, tub);
            crate::simple::hue(w, id, hue);
            true
        });
    }
}

/// Registers the classes of this file.
pub fn register(types: &mut TypeRegistry) {
    types.register_item(&ARMOIRE, &[]);
    types.register_item(&FANCY_ARMOIRE, &[]);
    types.register_phase_default(
        Phase::Configure,
        "Server.Items.DynamicFurniture.Configure",
        configure,
    );
}
