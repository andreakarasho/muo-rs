//! Script functions for creatures and players: stats, flags, skills, combat, healing, speech, inventory.
//!
//! Every function takes a live handle (`m` in a hook, `m.entity` in an `on_create`); a deleted mobile reads as
//! defaults and actions on it do nothing. Names are as in `[props` (snake case here); skills, layers, poisons
//! and directions by name (`"Swords"`, `"OneHanded"`, `"Deadly"`, `"North"`).

use muo_primitives::{Direction, MessageType, Point3D, TimeSpan};
use muo_server::ids::{ItemId, MobileId};
use muo_server::layer::Layer;
use muo_server::map::MapId;
use muo_server::mobile::part1::{props_a, props_b};
use muo_server::mobile::part2::{items, messages};
use muo_server::type_registry::EntityType;
use muo_server::world::World;
use rhai::{Array, Dynamic, Engine, NativeCallContext};

use super::api::{Item, Mobile, R, i32_of, live_m, with_world};

/// A getter on live mobiles (`default` for a deleted one).
macro_rules! get {
    ($e:ident, $name:literal, $default:expr, |$w:ident, $id:ident, $ty:ident| $body:expr) => {
        $e.register_get($name, |ctx: NativeCallContext, m: &mut Mobile| {
            let m = *m;
            with_world(&ctx, |$w| match live_m($w, m) {
                Some($ty) => {
                    let $id = m.0;
                    $body
                }
                None => $default,
            })
        });
    };
}

/// A setter on live mobiles (nothing for a deleted one).
macro_rules! set {
    ($e:ident, $name:literal, $t:ty, |$w:ident, $id:ident, $ty:ident, $v:ident| $body:expr) => {
        $e.register_set($name, |ctx: NativeCallContext, m: &mut Mobile, $v: $t| {
            let m = *m;
            with_world(&ctx, |$w| {
                if let Some($ty) = live_m($w, m) {
                    let $id = m.0;
                    let _ = $ty;
                    $body;
                }
            })
        });
    };
}

/// An action on a live mobile.
fn act(ctx: &NativeCallContext, m: Mobile, f: impl FnOnce(&mut World, MobileId)) -> R<()> {
    with_world(ctx, |w| {
        if live_m(w, m).is_some() {
            f(w, m.0);
        }
    })
}

fn layer(name: &str) -> Option<Layer> {
    (0..=Layer::Bank.0)
        .map(Layer)
        .find(|l| l.name() == Some(name))
}

fn direction(name: &str) -> Option<Direction> {
    let all = [
        ("North", Direction::NORTH),
        ("Right", Direction::RIGHT),
        ("East", Direction::EAST),
        ("Down", Direction::DOWN),
        ("South", Direction::SOUTH),
        ("Left", Direction::LEFT),
        ("West", Direction::WEST),
        ("Up", Direction::UP),
    ];
    all.iter().find(|(n, _)| *n == name).map(|(_, d)| *d)
}

fn direction_name(d: Direction) -> &'static str {
    [
        "North", "Right", "East", "Down", "South", "Left", "West", "Up",
    ][usize::from(d.facing().bits() & 7)]
}

fn skill(name: &str) -> R<muo_server::skills::SkillName> {
    crate::templates::mobile_steps::skill(name).map_err(Into::into)
}

/// The registered item class of `name` (short or full).
fn item_class(w: &World, name: &str) -> Option<&'static muo_server::item::ItemType> {
    let full = super::api_world::class_name(w, name);
    match w.types.find_type_by_full_name(&full, false)? {
        EntityType::Item(t) => Some(t),
        EntityType::Mobile(_) => None,
    }
}

pub(crate) fn register(e: &mut Engine) {
    // ---- stats ----
    get!(e, "mana", 0_i64, |w, id, _ty| i64::from(
        w.mobile(id).map_or(0, |m| m.mana)
    ));
    set!(e, "mana", i64, |w, id, ty, v| props_b::set_mana(
        w,
        id,
        i32_of(v)
    ));
    get!(e, "stam", 0_i64, |w, id, _ty| i64::from(
        w.mobile(id).map_or(0, |m| m.stam)
    ));
    set!(e, "stam", i64, |w, id, ty, v| props_b::set_stam(
        w,
        id,
        i32_of(v)
    ));
    get!(e, "mana_max", 0_i64, |w, id, ty| i64::from((ty.mana_max)(
        w, id
    )));
    get!(e, "stam_max", 0_i64, |w, id, ty| i64::from((ty.stam_max)(
        w, id
    )));
    get!(e, "fame", 0_i64, |w, id, _ty| i64::from(
        w.mobile(id).map_or(0, |m| m.fame)
    ));
    set!(e, "fame", i64, |w, id, ty, v| props_a::set_fame(
        w,
        id,
        i32_of(v)
    ));
    get!(e, "karma", 0_i64, |w, id, _ty| i64::from(
        w.mobile(id).map_or(0, |m| m.karma)
    ));
    set!(e, "karma", i64, |w, id, ty, v| props_a::set_karma(
        w,
        id,
        i32_of(v)
    ));
    get!(e, "kills", 0_i64, |w, id, _ty| i64::from(
        w.mobile(id).map_or(0, |m| m.kills)
    ));
    set!(e, "kills", i64, |w, id, ty, v| props_b::set_kills(
        w,
        id,
        i32_of(v)
    ));
    get!(e, "title", String::new(), |w, id, _ty| w
        .mobile(id)
        .and_then(|m| m.title.clone())
        .unwrap_or_default());
    set!(e, "title", rhai::ImmutableString, |w, id, ty, v| {
        props_a::set_title(w, id, (!v.is_empty()).then(|| v.to_string()))
    });

    // ---- flags ----
    get!(e, "alive", false, |w, id, ty| (ty.alive)(w, id));
    get!(e, "female", false, |w, id, _ty| w
        .mobile(id)
        .is_some_and(|m| m.female));
    set!(e, "female", bool, |w, id, ty, v| props_b::set_female(
        w, id, v
    ));
    get!(e, "hidden", false, |w, id, _ty| w
        .mobile(id)
        .is_some_and(|m| m.hidden));
    set!(e, "hidden", bool, |w, id, ty, v| props_b::set_hidden(
        w, id, v
    ));
    get!(e, "blessed", false, |w, id, _ty| w
        .mobile(id)
        .is_some_and(|m| m.blessed));
    set!(e, "blessed", bool, |w, id, ty, v| props_a::set_blessed(
        w, id, v
    ));
    get!(e, "frozen", false, |w, id, _ty| w
        .mobile(id)
        .is_some_and(|m| m.frozen));
    set!(e, "frozen", bool, |w, id, ty, v| props_a::set_frozen(
        w, id, v
    ));
    get!(e, "paralyzed", false, |w, id, ty| (ty.paralyzed)(w, id));
    set!(e, "paralyzed", bool, |w, id, ty, v| (ty.set_paralyzed)(
        w, id, v
    ));
    get!(e, "criminal", false, |w, id, ty| (ty.criminal)(w, id));
    set!(e, "criminal", bool, |w, id, ty, v| (ty.set_criminal)(
        w, id, v
    ));
    get!(e, "murderer", false, |w, id, ty| (ty.murderer)(w, id));
    get!(e, "warmode", false, |w, id, _ty| w
        .mobile(id)
        .is_some_and(|m| m.warmode));
    set!(e, "warmode", bool, |w, id, ty, v| props_b::set_warmode(
        w, id, v
    ));
    get!(e, "access_level", String::new(), |w, id, _ty| w
        .mobile(id)
        .map(|m| format!("{:?}", m.access_level))
        .unwrap_or_default());
    get!(e, "direction", String::new(), |w, id, _ty| w
        .mobile(id)
        .map_or("", |m| direction_name(m.direction))
        .to_owned());
    set!(e, "direction", rhai::ImmutableString, |w, id, ty, v| {
        if let Some(d) = direction(&v) {
            props_b::set_direction(w, id, d);
        }
    });

    // ---- skills ----
    // `m.skill_value("Swords")` (with modifiers), `m.skill_base("Swords")`, `m.set_skill("Swords", 80.0)`
    e.register_fn(
        "skill_value",
        |ctx: NativeCallContext, m: Mobile, name: &str| -> R<f64> {
            let s = skill(name)?;
            with_world(&ctx, |w| {
                live_m(w, m).map_or(0.0, |_| muo_server::skills::value(w, m.0, s))
            })
        },
    );
    e.register_fn(
        "skill_base",
        |ctx: NativeCallContext, m: Mobile, name: &str| -> R<f64> {
            let s = skill(name)?;
            with_world(&ctx, |w| {
                live_m(w, m).map_or(0.0, |_| muo_server::skills::base(w, m.0, s))
            })
        },
    );
    e.register_fn(
        "set_skill",
        |ctx: NativeCallContext, m: Mobile, name: &str, v: f64| -> R<()> {
            let s = skill(name)?;
            act(&ctx, m, |w, id| muo_server::skills::set_base(w, id, s, v))
        },
    );

    // ---- combat and health ----
    e.register_fn("heal", |ctx: NativeCallContext, m: Mobile, amount: i64| {
        act(&ctx, m, |w, id| {
            muo_server::mobile::part2::damage::heal(w, id, i32_of(amount))
        })
    });
    e.register_fn("resurrect", |ctx: NativeCallContext, m: Mobile| {
        act(&ctx, m, muo_server::mobile::base::resurrect)
    });
    // `m.poison("Deadly")` / `m.poison("Lesser", from)`: true when it took
    e.register_fn(
        "poison",
        |ctx: NativeCallContext, m: Mobile, level: &str| -> R<bool> {
            poison(&ctx, m, level, None)
        },
    );
    e.register_fn(
        "poison",
        |ctx: NativeCallContext, m: Mobile, level: &str, from: Mobile| -> R<bool> {
            poison(&ctx, m, level, Some(from.0))
        },
    );
    e.register_fn("cure", |ctx: NativeCallContext, m: Mobile| -> R<bool> {
        with_world(&ctx, |w| match live_m(w, m) {
            Some(ty) => (ty.cure_poison)(w, m.0, None),
            None => false,
        })
    });
    e.register_get(
        "poisoned",
        |ctx: NativeCallContext, m: &mut Mobile| -> R<bool> {
            let m = *m;
            with_world(&ctx, |w| w.mobile(m.0).is_some_and(|x| x.poison.is_some()))
        },
    );
    // `m.paralyze(5.0)` / `m.freeze(5.0)`: for that many seconds
    e.register_fn(
        "paralyze",
        |ctx: NativeCallContext, m: Mobile, secs: f64| {
            act(&ctx, m, |w, id| {
                if let Some(ty) = w.mobile(id).map(|x| x.ty) {
                    (ty.paralyze)(w, id, TimeSpan::from_secs_f64(secs));
                }
            })
        },
    );
    e.register_fn("freeze", |ctx: NativeCallContext, m: Mobile, secs: f64| {
        act(&ctx, m, |w, id| {
            if let Some(ty) = w.mobile(id).map(|x| x.ty) {
                (ty.freeze)(w, id, TimeSpan::from_secs_f64(secs));
            }
        })
    });
    // `m.attack(target)`: start fighting it
    e.register_fn(
        "attack",
        |ctx: NativeCallContext, m: Mobile, target: Mobile| {
            act(&ctx, m, |w, id| {
                if let Some(ty) = w.mobile(id).map(|x| x.ty)
                    && w.mobile(target.0).is_some_and(|t| !t.deleted)
                {
                    (ty.attack)(w, id, target.0);
                }
            })
        },
    );
    e.register_get(
        "combatant",
        |ctx: NativeCallContext, m: &mut Mobile| -> R<Dynamic> {
            let m = *m;
            with_world(&ctx, |w| match live_m(w, m) {
                Some(ty) => {
                    (ty.combatant)(w, m.0).map_or(Dynamic::UNIT, |c| Dynamic::from(Mobile(c)))
                }
                None => Dynamic::UNIT,
            })
        },
    );
    e.register_set(
        "combatant",
        |ctx: NativeCallContext, m: &mut Mobile, target: Dynamic| {
            let m = *m;
            let t = target.try_cast::<Mobile>().map(|t| t.0);
            with_world(&ctx, |w| {
                if let Some(ty) = live_m(w, m) {
                    (ty.set_combatant)(w, m.0, t);
                }
            })
        },
    );

    // ---- speech and animation ----
    e.register_fn("emote", |ctx: NativeCallContext, m: Mobile, text: &str| {
        act(&ctx, m, |w, id| messages::emote(w, id, text))
    });
    e.register_fn("yell", |ctx: NativeCallContext, m: Mobile, text: &str| {
        act(&ctx, m, |w, id| messages::yell(w, id, text))
    });
    e.register_fn(
        "whisper",
        |ctx: NativeCallContext, m: Mobile, text: &str| {
            act(&ctx, m, |w, id| messages::whisper(w, id, text))
        },
    );
    // `m.say_to(player, text)`: words over `m`'s head that only `player` sees
    e.register_fn(
        "say_to",
        |ctx: NativeCallContext, m: Mobile, to: Mobile, text: &str| {
            act(&ctx, m, |w, id| {
                let state = w.mobile(to.0).and_then(|t| t.net_state);
                messages::private_overhead_message(
                    w,
                    id,
                    MessageType::REGULAR,
                    0x3B2,
                    false,
                    text,
                    state,
                );
            })
        },
    );
    // `send_message(m, text, hue)`: a system message in a colour
    e.register_fn(
        "send_message",
        |ctx: NativeCallContext, m: Mobile, text: &str, hue: i64| {
            act(&ctx, m, |w, id| {
                messages::send_message_hue(w, id, i32_of(hue), text)
            })
        },
    );
    // `m.animate(action, frames)`: play an animation once
    e.register_fn(
        "animate",
        |ctx: NativeCallContext, m: Mobile, action: i64, frames: i64| {
            act(&ctx, m, |w, id| {
                muo_server::mobile::base::animate(
                    w,
                    id,
                    i32_of(action),
                    i32_of(frames),
                    1,
                    true,
                    false,
                    0,
                );
            })
        },
    );

    // ---- place ----
    // `m.move_to(x, y, z, map)`: also to another map
    e.register_fn(
        "move_to",
        |ctx: NativeCallContext, m: Mobile, x: i64, y: i64, z: i64, map: i64| {
            act(&ctx, m, |w, id| {
                if let Ok(map) = u8::try_from(map) {
                    let p = Point3D::new(i32_of(x), i32_of(y), i32_of(z));
                    muo_server::mobile::base::move_to_world(w, id, p, MapId(map));
                }
            })
        },
    );

    // ---- inventory ----
    e.register_get(
        "backpack",
        |ctx: NativeCallContext, m: &mut Mobile| -> R<Dynamic> {
            let m = *m;
            with_world(&ctx, |w| match live_m(w, m) {
                Some(_) => {
                    props_b::backpack(w, m.0).map_or(Dynamic::UNIT, |i| Dynamic::from(Item(i)))
                }
                None => Dynamic::UNIT,
            })
        },
    );
    // `m.equipped("OneHanded")`: the item worn on that layer, or `()`
    e.register_fn(
        "equipped",
        |ctx: NativeCallContext, m: Mobile, layer_name: &str| -> R<Dynamic> {
            let l = layer(layer_name).ok_or_else(|| format!("unknown layer {layer_name}"))?;
            with_world(&ctx, |w| {
                live_m(w, m)
                    .and_then(|_| items::find_item_on_layer(w, m.0, l))
                    .map_or(Dynamic::UNIT, |i| Dynamic::from(Item(i)))
            })
        },
    );
    // every item worn (the backpack included)
    e.register_get(
        "worn",
        |ctx: NativeCallContext, m: &mut Mobile| -> R<Array> {
            let m = *m;
            with_world(&ctx, |w| {
                w.mobile(m.0)
                    .map(|x| x.items.iter().map(|&i| Dynamic::from(Item(i))).collect())
                    .unwrap_or_default()
            })
        },
    );
    // `m.equip(item)`: true when it was put on
    e.register_fn(
        "equip",
        |ctx: NativeCallContext, m: Mobile, item: Item| -> R<bool> {
            with_world(&ctx, |w| match live_m(w, m) {
                Some(ty) if w.item(item.0).is_some_and(|i| !i.deleted()) => {
                    (ty.equip_item)(w, m.0, item.0)
                }
                _ => false,
            })
        },
    );
    // `m.give(item)`: into the backpack (at the feet when there is none)
    e.register_fn("give", |ctx: NativeCallContext, m: Mobile, item: Item| {
        act(&ctx, m, |w, id| give(w, id, item.0))
    });
    // `m.count("Gold")`: how many of a class the backpack holds, containers included
    e.register_fn(
        "count",
        |ctx: NativeCallContext, m: Mobile, class: &str| -> R<i64> {
            with_world(&ctx, |w| {
                let (Some(_), Some(ty)) = (live_m(w, m), item_class(w, class)) else {
                    return 0;
                };
                props_b::backpack(w, m.0).map_or(0, |p| {
                    i64::from(muo_server::item::container::get_amount(w, p, ty, true))
                })
            })
        },
    );
    // `m.consume("Bandage", 2)`: takes them from the backpack; false (taking nothing) without enough
    e.register_fn(
        "consume",
        |ctx: NativeCallContext, m: Mobile, class: &str, amount: i64| -> R<bool> {
            with_world(&ctx, |w| {
                let (Some(_), Some(ty)) = (live_m(w, m), item_class(w, class)) else {
                    return false;
                };
                props_b::backpack(w, m.0).is_some_and(|p| {
                    muo_server::item::container::consume_total(w, p, ty, i32_of(amount), true, None)
                })
            })
        },
    );

    // ---- players ----
    get!(e, "online", false, |w, id, _ty| w
        .mobile(id)
        .is_some_and(|m| m.net_state.is_some()));
    get!(e, "account", String::new(), |w, id, _ty| w
        .mobile(id)
        .and_then(|m| m.account)
        .and_then(|a| crate::accounting::account(w, a))
        .map(|a| a.username.clone())
        .unwrap_or_default());
}

fn poison(ctx: &NativeCallContext, m: Mobile, level: &str, from: Option<MobileId>) -> R<bool> {
    with_world(ctx, |w| {
        let Some(ty) = live_m(w, m) else {
            return Ok(false);
        };
        let p = w
            .resources
            .get::<muo_server::poison::Poisons>()
            .and_then(|p| p.try_parse(level))
            .ok_or_else(|| format!("unknown poison {level}"))?;
        Ok(matches!(
            (ty.apply_poison)(w, m.0, from, p),
            muo_server::mobile::ApplyPoisonResult::Poisoned
        ))
    })?
}

/// Into the backpack, else at the feet.
fn give(w: &mut World, id: MobileId, item: ItemId) {
    if w.item(item).is_none_or(|i| i.deleted()) {
        return;
    }
    match props_b::backpack(w, id) {
        Some(p) => muo_server::item::container::drop_item_default(w, p, item),
        None => {
            if let Some(m) = w.mobile(id) {
                let (loc, map) = (m.location, m.map);
                muo_server::item::ops::move_to_world_map(w, item, loc, map);
            }
        }
    }
}
