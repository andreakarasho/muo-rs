//! Script functions for sounds and visual effects, seen by the players near them.
//!
//! An effect's graphic is an item id (`0x3709` a fire column, `0x36BD` an explosion, `0x376A` sparkles, ...);
//! `speed` and `duration` are in the client's units (10 and 30 are common).

use muo_primitives::Point3D;
use muo_server::effects::{self, EffectLayer, ScreenEffectType};
use muo_server::ids::EntityId;
use muo_server::map::MapId;
use rhai::{Engine, NativeCallContext};

use super::api::{Item, Mobile, R, i32_of, live_i, live_m, with_world};

/// Runs `f` on the entity of a live handle.
fn on(
    ctx: &NativeCallContext,
    e: EntityId,
    f: impl FnOnce(&mut muo_server::world::World, EntityId),
) -> R<()> {
    with_world(ctx, |w| {
        let live = match e {
            EntityId::Mobile(m) => live_m(w, Mobile(m)).is_some(),
            EntityId::Item(i) => live_i(w, Item(i)).is_some(),
        };
        if live {
            f(w, e);
        }
    })
}

fn point(x: i64, y: i64, z: i64) -> Point3D {
    Point3D::new(i32_of(x), i32_of(y), i32_of(z))
}

fn map_of(map: i64) -> Option<MapId> {
    u8::try_from(map).ok().map(MapId)
}

/// The same function for a mobile and an item handle: `$f(ctx, entity, args..)`.
macro_rules! both {
    ($e:ident, $name:literal, |$ctx:ident, $ent:ident $(, $a:ident : $t:ty)*| $body:expr) => {
        $e.register_fn($name, |$ctx: NativeCallContext, m: Mobile $(, $a: $t)*| {
            let $ent = EntityId::Mobile(m.0);
            $body
        });
        $e.register_fn($name, |$ctx: NativeCallContext, i: Item $(, $a: $t)*| {
            let $ent = EntityId::Item(i.0);
            $body
        });
    };
}

pub(crate) fn register(e: &mut Engine) {
    // `m.play_sound(0x1F5)` / `i.play_sound(..)`: a sound where it is
    both!(e, "play_sound", |ctx, ent, sound: i64| on(
        &ctx,
        ent,
        |w, x| { effects::play_sound_entity(w, x, i32_of(sound)) }
    ));
    // `play_sound_at(x, y, z, map, sound)`
    e.register_fn(
        "play_sound_at",
        |ctx: NativeCallContext, x: i64, y: i64, z: i64, map: i64, sound: i64| {
            with_world(&ctx, |w| {
                effects::play_sound(w, point(x, y, z), map_of(map), i32_of(sound))
            })
        },
    );
    // `m.lightning()`: a lightning bolt strikes it (with its thunder)
    both!(e, "lightning", |ctx, ent| on(&ctx, ent, |w, x| {
        effects::send_bolt_effect(w, x, true, 0)
    }));
    // `m.effect(graphic, speed, duration)` / `(.., hue)`: an animation on it, following it
    both!(e, "effect", |ctx,
                        ent,
                        item_id: i64,
                        speed: i64,
                        duration: i64| on(
        &ctx,
        ent,
        |w, x| {
            effects::send_target_effect(
                w,
                x,
                i32_of(item_id),
                i32_of(speed),
                i32_of(duration),
                0,
                0,
            )
        }
    ));
    both!(e, "effect", |ctx,
                        ent,
                        item_id: i64,
                        speed: i64,
                        duration: i64,
                        hue: i64| on(
        &ctx,
        ent,
        |w, x| {
            effects::send_target_effect(
                w,
                x,
                i32_of(item_id),
                i32_of(speed),
                i32_of(duration),
                i32_of(hue),
                0,
            )
        }
    ));
    // `m.particles(graphic, speed, duration, effect)`: the particle form (`effect` is the client's effect id)
    both!(e, "particles", |ctx,
                           ent,
                           item_id: i64,
                           speed: i64,
                           duration: i64,
                           effect: i64| on(
        &ctx,
        ent,
        |w, x| {
            effects::send_target_particles(
                w,
                x,
                i32_of(item_id),
                i32_of(speed),
                i32_of(duration),
                i32_of(effect),
                EffectLayer::Waist,
            )
        }
    ));
    // `effect_at(x, y, z, map, graphic, duration)` / `(.., hue)`: an animation on the ground
    e.register_fn(
        "effect_at",
        |ctx: NativeCallContext, x: i64, y: i64, z: i64, map: i64, item_id: i64, duration: i64| {
            with_world(&ctx, |w| {
                effects::send_location_effect(
                    w,
                    point(x, y, z),
                    map_of(map),
                    i32_of(item_id),
                    i32_of(duration),
                    10,
                    0,
                    0,
                )
            })
        },
    );
    e.register_fn(
        "effect_at",
        |ctx: NativeCallContext,
         x: i64,
         y: i64,
         z: i64,
         map: i64,
         item_id: i64,
         duration: i64,
         hue: i64| {
            with_world(&ctx, |w| {
                effects::send_location_effect(
                    w,
                    point(x, y, z),
                    map_of(map),
                    i32_of(item_id),
                    i32_of(duration),
                    10,
                    i32_of(hue),
                    0,
                )
            })
        },
    );
    // `moving_effect(from, to, graphic, speed)` / `(.., hue)`: a missile flying from one to the other
    for (fname, explodes) in [("moving_effect", false), ("missile", true)] {
        e.register_fn(
            fname,
            move |ctx: NativeCallContext, from: Mobile, to: Mobile, item_id: i64, speed: i64| {
                moving(
                    &ctx,
                    EntityId::Mobile(from.0),
                    EntityId::Mobile(to.0),
                    item_id,
                    speed,
                    0,
                    explodes,
                )
            },
        );
        e.register_fn(
            fname,
            move |ctx: NativeCallContext,
                  from: Mobile,
                  to: Mobile,
                  item_id: i64,
                  speed: i64,
                  hue: i64| {
                moving(
                    &ctx,
                    EntityId::Mobile(from.0),
                    EntityId::Mobile(to.0),
                    item_id,
                    speed,
                    hue,
                    explodes,
                )
            },
        );
        e.register_fn(
            fname,
            move |ctx: NativeCallContext, from: Mobile, to: Item, item_id: i64, speed: i64| {
                moving(
                    &ctx,
                    EntityId::Mobile(from.0),
                    EntityId::Item(to.0),
                    item_id,
                    speed,
                    0,
                    explodes,
                )
            },
        );
        e.register_fn(
            fname,
            move |ctx: NativeCallContext, from: Item, to: Mobile, item_id: i64, speed: i64| {
                moving(
                    &ctx,
                    EntityId::Item(from.0),
                    EntityId::Mobile(to.0),
                    item_id,
                    speed,
                    0,
                    explodes,
                )
            },
        );
    }
    // `m.flash("LightFlash")`: a screen effect for that player (FadeOut, FadeIn, LightFlash, FadeInOut, DarkFlash)
    e.register_fn(
        "flash",
        |ctx: NativeCallContext, m: Mobile, kind: &str| -> R<()> {
            let kind = match kind {
                "FadeOut" => ScreenEffectType::FadeOut,
                "FadeIn" => ScreenEffectType::FadeIn,
                "LightFlash" => ScreenEffectType::LightFlash,
                "FadeInOut" => ScreenEffectType::FadeInOut,
                "DarkFlash" => ScreenEffectType::DarkFlash,
                other => return Err(format!("unknown screen effect {other}").into()),
            };
            with_world(&ctx, |w| {
                if let Some(ns) = w
                    .mobile(m.0)
                    .filter(|x| !x.deleted)
                    .and_then(|x| x.net_state)
                {
                    muo_server::packets::effect::send_screen_effect(w, ns, kind);
                }
            })
        },
    );
}

fn moving(
    ctx: &NativeCallContext,
    from: EntityId,
    to: EntityId,
    item_id: i64,
    speed: i64,
    hue: i64,
    explodes: bool,
) -> R<()> {
    on(ctx, from, |w, f| {
        let alive = match to {
            EntityId::Mobile(m) => w.mobile(m).is_some_and(|x| !x.deleted),
            EntityId::Item(i) => w.item(i).is_some_and(|x| !x.deleted()),
        };
        if alive {
            effects::send_moving_effect(
                w,
                f,
                to,
                i32_of(item_id),
                i32_of(speed),
                0,
                false,
                explodes,
                i32_of(hue),
                0,
            );
        }
    })
}
