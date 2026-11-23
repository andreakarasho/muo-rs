//! Script functions for the world: lookups, players, spawning items, broadcast, regions, terrain, time.
//!
//! Maps are numbers (`0` Felucca, `1` Trammel, ...; `map_id("Ilshenar")`). A class is a short or full name.

use muo_primitives::{Point3D, Serial};
use muo_server::ids::{ItemId, MobileId};
use muo_server::map::MapId;
use muo_server::world::World;
use rhai::{Array, Dynamic, Engine, NativeCallContext};

use super::api::{Item, Mobile, R, i32_of, with_world};

/// The full name of a class a script names short or full (`"Torch"` -> `"Server.Items.Torch"`).
pub(crate) fn class_name(w: &World, name: &str) -> String {
    super::classes::full_name(w, name).unwrap_or_else(|_| name.to_owned())
}

fn map_of(map: i64) -> Option<MapId> {
    u8::try_from(map).ok().map(MapId)
}

pub(crate) fn register(e: &mut Engine) {
    // ---- lookups ----
    // `find_mobile(serial)` / `find_item(serial)`: the entity with that serial, or `()`
    e.register_fn(
        "find_mobile",
        |ctx: NativeCallContext, serial: i64| -> R<Dynamic> {
            with_world(&ctx, |w| {
                let id = MobileId(Serial::new(u32::try_from(serial).unwrap_or(0)));
                w.mobile(id)
                    .filter(|m| !m.deleted)
                    .map_or(Dynamic::UNIT, |_| Dynamic::from(Mobile(id)))
            })
        },
    );
    e.register_fn(
        "find_item",
        |ctx: NativeCallContext, serial: i64| -> R<Dynamic> {
            with_world(&ctx, |w| {
                let id = ItemId(Serial::new(u32::try_from(serial).unwrap_or(0)));
                w.item(id)
                    .filter(|i| !i.deleted())
                    .map_or(Dynamic::UNIT, |_| Dynamic::from(Item(id)))
            })
        },
    );
    // `a == b` / `a != b`: the same entity (a handle against anything else is never equal)
    e.register_fn("==", |a: Mobile, b: Mobile| a == b);
    e.register_fn("!=", |a: Mobile, b: Mobile| a != b);
    e.register_fn("==", |a: Item, b: Item| a == b);
    e.register_fn("!=", |a: Item, b: Item| a != b);
    e.register_get("serial", |m: &mut Mobile| i64::from(m.0.0.value()));
    e.register_get("serial", |i: &mut Item| i64::from(i.0.0.value()));
    // the mobiles of every connected player
    e.register_fn("players", |ctx: NativeCallContext| -> R<Array> {
        with_world(&ctx, |w| {
            w.net_states
                .values()
                .filter_map(|n| n.mobile)
                .map(|m| Dynamic::from(Mobile(m)))
                .collect()
        })
    });
    e.register_fn("online_count", |ctx: NativeCallContext| -> R<i64> {
        with_world(&ctx, |w| {
            w.net_states.values().filter(|n| n.mobile.is_some()).count() as i64
        })
    });
    // the items within `range` tiles of (x, y) on `map` (on the ground, not in containers)
    e.register_fn(
        "items_in_range",
        |ctx: NativeCallContext, x: i64, y: i64, map: i64, range: i64| -> R<Array> {
            with_world(&ctx, |w| {
                let Some(map) = map_of(map) else {
                    return Array::new();
                };
                muo_server::map::enumerate::items_in_range(
                    w,
                    Some(map),
                    i32_of(x),
                    i32_of(y),
                    i32_of(range),
                )
                .map(|i| Dynamic::from(Item(i)))
                .collect()
            })
        },
    );

    // ---- making things ----
    // `spawn_item(class, x, y, z, map)`: a new item on the ground; `()` for an unknown class
    e.register_fn(
        "spawn_item",
        |ctx: NativeCallContext, class: &str, x: i64, y: i64, z: i64, map: i64| -> R<Dynamic> {
            with_world(&ctx, |w| {
                let class = class_name(w, class);
                let (Some(map), Some(i)) = (map_of(map), crate::factory::make_item(w, &class))
                else {
                    return Dynamic::UNIT;
                };
                let p = Point3D::new(i32_of(x), i32_of(y), i32_of(z));
                muo_server::item::ops::move_to_world_map(w, i, p, Some(map));
                Dynamic::from(Item(i))
            })
        },
    );
    // `spawn_house(owner, multi_id, x, y, z, map)`: a customizable house foundation owned by `owner`, placed with no
    // placement checks (`0x147B` is the 18x18 plot); `()` when it cannot be made
    e.register_fn(
        "spawn_house",
        |ctx: NativeCallContext, owner: Mobile, multi_id: i64, x: i64, y: i64, z: i64, map: i64| {
            with_world(&ctx, |w| {
                let make = crate::items::house_deps::deps(w).and_then(|d| d.construct_foundation);
                let (Some(make), Some(map)) = (make, map_of(map)) else {
                    return Dynamic::UNIT;
                };
                // storage and lockdowns of the largest plot: the size of the multi does not tell them
                let Some(house) = make(w, owner.0, i32_of(multi_id), 2119, 1059) else {
                    return Dynamic::UNIT;
                };
                let p = Point3D::new(i32_of(x), i32_of(y), i32_of(z));
                muo_server::item::ops::move_to_world_map(w, house, p, Some(map));
                Dynamic::from(Item(house))
            })
        },
    );
    // `create_item(class)`: a new item nowhere yet (put it in a pack, a container or the world)
    e.register_fn(
        "create_item",
        |ctx: NativeCallContext, class: &str| -> R<Dynamic> {
            with_world(&ctx, |w| {
                let class = class_name(w, class);
                crate::factory::make_item(w, &class)
                    .map_or(Dynamic::UNIT, |i| Dynamic::from(Item(i)))
            })
        },
    );
    // `create_mobile(class)`: a new mobile nowhere yet (`m.move_to(x, y, z, map)` places it)
    e.register_fn(
        "create_mobile",
        |ctx: NativeCallContext, class: &str| -> R<Dynamic> {
            with_world(&ctx, |w| {
                let class = class_name(w, class);
                crate::factory::make_mobile(w, &class)
                    .map_or(Dynamic::UNIT, |m| Dynamic::from(Mobile(m)))
            })
        },
    );

    // ---- messages ----
    // `broadcast(text)` / `broadcast(text, hue)`: to every connected player, as the server's own messages
    e.register_fn("broadcast", |ctx: NativeCallContext, text: &str| {
        with_world(&ctx, |w| w.broadcast(0x482, false, text))
    });
    e.register_fn(
        "broadcast",
        |ctx: NativeCallContext, text: &str, hue: i64| {
            with_world(&ctx, |w| w.broadcast(i32_of(hue), false, text))
        },
    );
    e.register_fn("broadcast_staff", |ctx: NativeCallContext, text: &str| {
        with_world(&ctx, |w| w.broadcast_staff(0x482, false, text))
    });

    // ---- places ----
    // `region_name(x, y, map)`: the name of the region there, `""` for none
    e.register_fn(
        "region_name",
        |ctx: NativeCallContext, x: i64, y: i64, map: i64| -> R<String> {
            with_world(&ctx, |w| {
                let p = Point3D::new(i32_of(x), i32_of(y), 0);
                let r = muo_server::region::find(w, p, map_of(map));
                w.region(r).and_then(|r| r.name.clone()).unwrap_or_default()
            })
        },
    );
    e.register_get(
        "region",
        |ctx: NativeCallContext, m: &mut Mobile| -> R<String> {
            let m = *m;
            with_world(&ctx, |w| {
                w.mobile(m.0)
                    .and_then(|x| w.region(x.region))
                    .and_then(|r| r.name.clone())
                    .unwrap_or_default()
            })
        },
    );
    // `ground_z(x, y, map)`: the height of the land there, to place things on it
    e.register_fn(
        "ground_z",
        |ctx: NativeCallContext, x: i64, y: i64, map: i64| -> R<i64> {
            with_world(&ctx, |w| match map_of(map) {
                Some(m) if w.maps.get(m).is_some() => {
                    i64::from(w.maps.get_average_z(m, i32_of(x), i32_of(y)))
                }
                _ => 0,
            })
        },
    );
    // `map_id("Felucca")` -> 0 (-1 for an unknown name); `map_name(0)` -> "Felucca"
    e.register_fn("map_id", |ctx: NativeCallContext, name: &str| -> R<i64> {
        with_world(&ctx, |w| w.maps.parse(name).map_or(-1, |m| i64::from(m.0)))
    });
    e.register_fn(
        "map_name",
        |ctx: NativeCallContext, map: i64| -> R<String> {
            with_world(&ctx, |w| {
                map_of(map)
                    .and_then(|m| w.maps.get(m))
                    .map(|m| m.name().to_owned())
                    .unwrap_or_default()
            })
        },
    );

    // ---- time ----
    // `server_time()`: the server's date and time as text; `ticks()`: milliseconds since start
    e.register_fn("server_time", |ctx: NativeCallContext| -> R<String> {
        with_world(&ctx, |w| w.now.to_string())
    });
    e.register_fn("ticks", |ctx: NativeCallContext| -> R<i64> {
        with_world(&ctx, |w| w.tick_count)
    });
}
