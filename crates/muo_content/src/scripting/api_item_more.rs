//! Script functions for items: location, flags, containers, overhead messages.
//!
//! Every function takes a live item handle (`i.entity` in an `on_create`); a deleted item reads as defaults and
//! actions on it do nothing. Layers and loot types by name (`"OneHanded"`, `"Blessed"`).

use muo_primitives::Point3D;
use muo_server::ids::{EntityId, ItemId};
use muo_server::item::{LootType, base, ops};
use muo_server::layer::Layer;
use muo_server::map::MapId;
use muo_server::packets::message::MessageType;
use muo_server::world::World;
use rhai::{Array, Dynamic, Engine, NativeCallContext};

use super::api::{Item, R, i32_of, live_i, with_world};

const LOOT_TYPES: [&str; 4] = ["Regular", "Newbied", "Blessed", "Cursed"];

macro_rules! get {
    ($e:ident, $name:literal, $default:expr, |$w:ident, $id:ident| $body:expr) => {
        $e.register_get($name, |ctx: NativeCallContext, i: &mut Item| {
            let i = *i;
            with_world(&ctx, |$w| match live_i($w, i) {
                Some(_) => {
                    let $id = i.0;
                    $body
                }
                None => $default,
            })
        });
    };
}

macro_rules! set {
    ($e:ident, $name:literal, $t:ty, |$w:ident, $id:ident, $v:ident| $body:expr) => {
        $e.register_set($name, |ctx: NativeCallContext, i: &mut Item, $v: $t| {
            let i = *i;
            with_world(&ctx, |$w| {
                if live_i($w, i).is_some() {
                    let $id = i.0;
                    $body;
                }
            })
        });
    };
}

fn act(ctx: &NativeCallContext, i: Item, f: impl FnOnce(&mut World, ItemId)) -> R<()> {
    with_world(ctx, |w| {
        if live_i(w, i).is_some() {
            f(w, i.0);
        }
    })
}

/// Where an item is in the world: its own place, or its outermost container's / wearer's. The internal map (where a
/// new item waits until it is placed) is no map.
fn world_place(w: &World, id: ItemId) -> (Point3D, Option<MapId>) {
    let (p, map) = world_place_raw(w, id);
    (p, map.filter(|m| *m != MapId::INTERNAL))
}

fn world_place_raw(w: &World, id: ItemId) -> (Point3D, Option<MapId>) {
    match ops::root_parent(w, id) {
        Some(EntityId::Mobile(m)) => w
            .mobile(m)
            .map_or((Point3D::default(), None), |m| (m.location, m.map)),
        Some(EntityId::Item(r)) if r != id => w
            .item(r)
            .map_or((Point3D::default(), None), |r| (r.location, r.map)),
        _ => w.item(id).map_or((Point3D::default(), None), |i| {
            (base::location(w, id), i.map)
        }),
    }
}

pub(crate) fn register(e: &mut Engine) {
    // ---- place: in the world (in a container or worn: the outermost holder's place) ----
    get!(e, "x", 0_i64, |w, id| i64::from(world_place(w, id).0.x));
    get!(e, "y", 0_i64, |w, id| i64::from(world_place(w, id).0.y));
    get!(e, "z", 0_i64, |w, id| i64::from(world_place(w, id).0.z));
    get!(e, "map", -1_i64, |w, id| world_place(w, id)
        .1
        .map_or(-1, |m| i64::from(m.0)));
    // `i.on_ground`: lying in the world, not held by anything
    get!(e, "on_ground", false, |w, id| w
        .item(id)
        .is_some_and(|i| i.parent.is_none() && i.map.is_some()));
    // `i.move_to(x, y, z)` (same map) / `i.move_to(x, y, z, map)`: onto the ground there
    e.register_fn(
        "move_to",
        |ctx: NativeCallContext, i: Item, x: i64, y: i64, z: i64| {
            act(&ctx, i, |w, id| {
                let map = world_place(w, id).1;
                ops::move_to_world_map(w, id, Point3D::new(i32_of(x), i32_of(y), i32_of(z)), map);
            })
        },
    );
    e.register_fn(
        "move_to",
        |ctx: NativeCallContext, i: Item, x: i64, y: i64, z: i64, map: i64| {
            act(&ctx, i, |w, id| {
                if let Ok(map) = u8::try_from(map) {
                    let p = Point3D::new(i32_of(x), i32_of(y), i32_of(z));
                    ops::move_to_world_map(w, id, p, Some(MapId(map)));
                }
            })
        },
    );

    // ---- flags ----
    get!(e, "movable", false, |w, id| w
        .item(id)
        .is_some_and(|i| i.movable()));
    set!(e, "movable", bool, |w, id, v| ops::set_movable(w, id, v));
    get!(e, "visible", false, |w, id| w
        .item(id)
        .is_some_and(|i| i.visible()));
    set!(e, "visible", bool, |w, id, v| ops::set_visible(w, id, v));
    get!(e, "weight", 0.0_f64, |w, id| ops::weight(w, id));
    set!(e, "weight", f64, |w, id, v| ops::set_weight(w, id, v));
    get!(e, "layer", String::new(), |w, id| base::layer(w, id)
        .name()
        .unwrap_or("")
        .to_owned());
    set!(e, "layer", rhai::ImmutableString, |w, id, v| {
        if let Some(l) = (0..=Layer::Bank.0)
            .map(Layer)
            .find(|l| l.name() == Some(v.as_str()))
        {
            base::set_layer(w, id, l);
        }
    });
    get!(e, "loot_type", String::new(), |w, id| w
        .item(id)
        .and_then(|i| LOOT_TYPES.get(usize::from(i.loot_type.0)))
        .copied()
        .unwrap_or("")
        .to_owned());
    set!(e, "loot_type", rhai::ImmutableString, |w, id, v| {
        if let Some(n) = LOOT_TYPES.iter().position(|n| *n == v.as_str()) {
            ops::set_loot_type(w, id, LootType(n as u8));
        }
    });

    // ---- containers ----
    get!(e, "is_container", false, |w, id| ops::is_container(w, id));
    // `i.contents`: the items directly inside (or `[]`)
    e.register_get(
        "contents",
        |ctx: NativeCallContext, i: &mut Item| -> R<Array> {
            let i = *i;
            with_world(&ctx, |w| {
                w.item(i.0)
                    .map(|c| c.items.iter().map(|&x| Dynamic::from(Item(x))).collect())
                    .unwrap_or_default()
            })
        },
    );
    // `container.drop_in(item)`: puts `item` inside (false when `container` is no container)
    e.register_fn(
        "drop_in",
        |ctx: NativeCallContext, c: Item, item: Item| -> R<bool> {
            with_world(&ctx, |w| {
                if live_i(w, c).is_none()
                    || live_i(w, item).is_none()
                    || !ops::is_container(w, c.0)
                    || c == item
                {
                    return false;
                }
                muo_server::item::container::drop_item_default(w, c.0, item.0);
                true
            })
        },
    );
    // `i.consume(n)`: takes `n` from the stack (the item is deleted at 0)
    e.register_fn("consume", |ctx: NativeCallContext, i: Item, n: i64| {
        act(&ctx, i, |w, id| base::consume(w, id, i32_of(n)))
    });

    // ---- messages ----
    // `i.say(text)` / `i.say(text, hue)`: words over the item, for everyone near
    e.register_fn("say", |ctx: NativeCallContext, i: Item, text: &str| {
        act(&ctx, i, |w, id| {
            muo_server::item::messages::public_overhead_message(
                w,
                id,
                MessageType::REGULAR,
                0x3B2,
                false,
                text,
            );
        })
    });
    e.register_fn(
        "say",
        |ctx: NativeCallContext, i: Item, text: &str, hue: i64| {
            act(&ctx, i, |w, id| {
                muo_server::item::messages::public_overhead_message(
                    w,
                    id,
                    MessageType::REGULAR,
                    i32_of(hue),
                    false,
                    text,
                );
            })
        },
    );
}
