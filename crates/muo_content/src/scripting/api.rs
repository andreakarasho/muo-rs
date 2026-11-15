//! The functions scripts call: thin wrappers over the engine's own functions.
//!
//! Handles (`Mobile`, `Item`) are ids, resolved on every use: a deleted entity reads as defaults (`deleted` is
//! true) and actions on it do nothing.

use muo_primitives::{Body, Point3D, TimeSpan};
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::map::MapId;
use muo_server::mobile::part1::props_b;
use muo_server::mobile::part2::messages;
use muo_server::script_hooks::ScriptArg;
use muo_server::world::World;
use rhai::{Dynamic, Engine, EvalAltResult, ImmutableString, NativeCallContext};

/// The evaluation tag of a script call: the world it runs against and the calling script's name.
#[derive(Clone)]
pub(crate) struct Ctx {
    pub world: *mut World,
    pub script: ImmutableString,
}

pub(crate) type R<T> = Result<T, Box<EvalAltResult>>;

/// Runs `f` with the world of the script call in progress.
///
/// SAFETY: `ScriptHost::call` makes `Ctx::world` from its `&mut World` right before `call_fn_with_options` and
/// does not touch that `&mut` until the call returns, so for the call's duration the pointer is the only path to
/// the world. Each native function turns it into one `&mut World` that lives only inside `f`; rhai runs native
/// functions one at a time on this thread, and none returns a reference into the world, so no two of these
/// borrows overlap. A nested script call (an action here raises a hook that runs a script) gets its own `Ctx`
/// made from the `&mut` reborrowed inside `f`, and the outer pointer is not used until `f` returns. The tag is
/// never handed to scripts and is dropped with the call, so the pointer cannot outlive the borrow it came from.
pub(crate) fn with_world<T>(ctx: &NativeCallContext, f: impl FnOnce(&mut World) -> T) -> R<T> {
    let ptr = ctx
        .tag()
        .and_then(|t| t.read_lock::<Ctx>())
        .map(|c| c.world);
    let Some(ptr) = ptr else {
        return Err("no world: only callable from a server hook".into());
    };
    // SAFETY: see above.
    Ok(f(unsafe { &mut *ptr }))
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Mobile(pub MobileId);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Item(pub ItemId);

pub(crate) fn mobile(m: MobileId) -> Dynamic {
    Dynamic::from(Mobile(m))
}

pub(crate) fn entity(e: Option<EntityId>) -> Dynamic {
    match e {
        Some(EntityId::Mobile(m)) => mobile(m),
        Some(EntityId::Item(i)) => Dynamic::from(Item(i)),
        None => Dynamic::UNIT,
    }
}

pub(crate) fn arg(a: ScriptArg) -> Dynamic {
    match a {
        ScriptArg::Mobile(m) => entity(m.map(EntityId::Mobile)),
        ScriptArg::Item(i) => entity(i.map(EntityId::Item)),
        ScriptArg::Int(n) => n.into(),
        ScriptArg::Str(s) => s.into(),
    }
}

pub(crate) fn i32_of(v: i64) -> i32 {
    v.clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32
}

/// A live mobile's class and id (`None` when deleted).
pub(crate) fn live_m(w: &World, m: Mobile) -> Option<&'static muo_server::mobile::MobileType> {
    w.mobile(m.0).filter(|x| !x.deleted).map(|x| x.ty)
}

pub(crate) fn live_i(w: &World, i: Item) -> Option<&'static muo_server::item::ItemType> {
    w.item(i.0).filter(|x| !x.deleted()).map(|x| x.ty)
}

/// `new T()` by class name, with `amount` set when above 1.
fn make_item(w: &mut World, class: &str, amount: i64) -> Option<ItemId> {
    let it = crate::factory::make_item(w, class)?;
    if amount > 1 {
        muo_server::item::ops::set_amount(w, it, i32_of(amount));
    }
    Some(it)
}

pub(crate) fn engine() -> Engine {
    let mut e = Engine::new();
    // A runaway script errors out instead of hanging the game loop.
    e.set_max_operations(1_000_000)
        .set_max_call_levels(32)
        .set_max_expr_depths(64, 32)
        .set_max_string_size(64 * 1024)
        .set_max_array_size(10_000)
        .set_max_map_size(10_000)
        .set_max_variables(1_000);
    e.on_print(|s| tracing::info!("[script] {s}"));
    e.on_debug(|s, src, pos| tracing::debug!("[script] {}:{pos}: {s}", src.unwrap_or("")));
    e.register_type_with_name::<Mobile>("Mobile")
        .register_type_with_name::<Item>("Item");
    mobile_api(&mut e);
    item_api(&mut e);
    world_api(&mut e);
    super::api_equipment::register(&mut e);
    super::api_misc_items::register(&mut e);
    super::api_misc_items2::register(&mut e);
    super::api_mobiles::register(&mut e);
    super::api_npcs::register(&mut e);
    super::api_engines::register(&mut e);
    e
}

macro_rules! mget {
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

macro_rules! mset {
    ($e:ident, $name:literal, $t:ty, |$w:ident, $id:ident, $ty:ident, $v:ident| $body:expr) => {
        $e.register_set($name, |ctx: NativeCallContext, m: &mut Mobile, $v: $t| {
            let m = *m;
            with_world(&ctx, |$w| {
                if let Some($ty) = live_m($w, m) {
                    let $id = m.0;
                    $body;
                }
            })
        });
    };
}

fn mobile_api(e: &mut Engine) {
    mget!(e, "name", String::new(), |w, id, ty| (ty.name)(w, id)
        .unwrap_or_default());
    mset!(e, "name", ImmutableString, |w, id, ty, v| (ty.set_name)(
        w,
        id,
        Some(v.to_string())
    ));
    mget!(e, "hits", 0_i64, |w, id, _ty| i64::from(
        w.mobile(id).map_or(0, |m| m.hits)
    ));
    mset!(e, "hits", i64, |w, id, _ty, v| props_b::set_hits(
        w,
        id,
        i32_of(v)
    ));
    mget!(e, "hits_max", 0_i64, |w, id, ty| i64::from((ty.hits_max)(
        w, id
    )));
    mget!(e, "str", 0_i64, |w, id, ty| i64::from((ty.str)(w, id)));
    mset!(e, "str", i64, |w, id, ty, v| (ty.set_str)(w, id, i32_of(v)));
    mget!(e, "dex", 0_i64, |w, id, ty| i64::from((ty.dex)(w, id)));
    mset!(e, "dex", i64, |w, id, ty, v| (ty.set_dex)(w, id, i32_of(v)));
    mget!(e, "int", 0_i64, |w, id, ty| i64::from((ty.int)(w, id)));
    mset!(e, "int", i64, |w, id, ty, v| (ty.set_int)(w, id, i32_of(v)));
    mget!(e, "x", 0_i64, |w, id, _ty| i64::from(
        w.mobile(id).map_or(0, |m| m.location.x)
    ));
    mget!(e, "y", 0_i64, |w, id, _ty| i64::from(
        w.mobile(id).map_or(0, |m| m.location.y)
    ));
    mget!(e, "z", 0_i64, |w, id, _ty| i64::from(
        w.mobile(id).map_or(0, |m| m.location.z)
    ));
    mget!(e, "map", -1_i64, |w, id, _ty| w
        .mobile(id)
        .and_then(|m| m.map)
        .map_or(-1, |m| i64::from(m.0)));
    mget!(e, "body", 0_i64, |w, id, _ty| i64::from(
        w.mobile(id).map_or(0, |m| m.body.0)
    ));
    mset!(e, "body", i64, |w, id, _ty, v| props_b::set_body(
        w,
        id,
        Body(i32_of(v))
    ));
    mget!(e, "hue", 0_i64, |w, id, ty| i64::from((ty.hue)(w, id)));
    mset!(e, "hue", i64, |w, id, ty, v| (ty.set_hue)(w, id, i32_of(v)));
    mget!(e, "is_player", false, |w, id, _ty| w
        .mobile(id)
        .is_some_and(|m| m.player));
    e.register_get("deleted", |ctx: NativeCallContext, m: &mut Mobile| {
        let m = *m;
        with_world(&ctx, |w| live_m(w, m).is_none())
    });

    e.register_fn("say", |ctx: NativeCallContext, m: Mobile, text: &str| {
        with_world(&ctx, |w| {
            if live_m(w, m).is_some() {
                messages::say(w, m.0, false, text);
            }
        })
    });
    e.register_fn(
        "send_message",
        |ctx: NativeCallContext, m: Mobile, text: &str| {
            with_world(&ctx, |w| {
                if live_m(w, m).is_some() {
                    messages::send_message(w, m.0, text);
                }
            })
        },
    );
    e.register_fn("kill", |ctx: NativeCallContext, m: Mobile| {
        with_world(&ctx, |w| {
            if let Some(ty) = live_m(w, m) {
                (ty.kill)(w, m.0);
            }
        })
    });
    e.register_fn(
        "damage",
        |ctx: NativeCallContext, m: Mobile, amount: i64, from: Dynamic| {
            let from = from.try_cast::<Mobile>().map(|f| f.0);
            with_world(&ctx, |w| {
                if let Some(ty) = live_m(w, m) {
                    (ty.damage)(w, m.0, i32_of(amount), from, true, false);
                }
            })
        },
    );
    e.register_fn(
        "move_to",
        |ctx: NativeCallContext, m: Mobile, x: i64, y: i64, z: i64| {
            with_world(&ctx, |w| {
                let map = live_m(w, m).and(w.mobile(m.0)).and_then(|o| o.map);
                if let Some(map) = map {
                    let p = Point3D::new(i32_of(x), i32_of(y), i32_of(z));
                    muo_server::mobile::base::move_to_world(w, m.0, p, map);
                }
            })
        },
    );
    e.register_fn(
        "pack_item",
        |ctx: NativeCallContext, m: Mobile, class: &str, amount: i64| {
            with_world(&ctx, |w| {
                live_m(w, m)?;
                let it = make_item(w, class, amount)?;
                crate::creature::setters::pack_item(w, m.0, Some(it));
                Some(it)
            })
            .map(|it| entity(it.map(EntityId::Item)))
        },
    );
    e.register_fn("delete", |ctx: NativeCallContext, m: Mobile| {
        with_world(&ctx, |w| {
            if live_m(w, m).is_some() {
                muo_server::mobile::base::delete(w, m.0);
            }
        })
    });
}

macro_rules! iget {
    ($e:ident, $name:literal, $default:expr, |$w:ident, $id:ident, $ty:ident| $body:expr) => {
        $e.register_get($name, |ctx: NativeCallContext, i: &mut Item| {
            let i = *i;
            with_world(&ctx, |$w| match live_i($w, i) {
                Some($ty) => {
                    let $id = i.0;
                    $body
                }
                None => $default,
            })
        });
    };
}

macro_rules! iset {
    ($e:ident, $name:literal, $t:ty, |$w:ident, $id:ident, $ty:ident, $v:ident| $body:expr) => {
        $e.register_set($name, |ctx: NativeCallContext, i: &mut Item, $v: $t| {
            let i = *i;
            with_world(&ctx, |$w| {
                if let Some($ty) = live_i($w, i) {
                    let $id = i.0;
                    $body;
                }
            })
        });
    };
}

fn item_api(e: &mut Engine) {
    use muo_server::item::{base, ops};
    iget!(e, "name", String::new(), |w, id, ty| w
        .item(id)
        .and_then(|i| i.name.clone())
        .or_else(|| (ty.default_name)(w, id))
        .unwrap_or_default());
    iset!(e, "name", ImmutableString, |w, id, _ty, v| ops::set_name(
        w,
        id,
        Some(v.to_string())
    ));
    iget!(e, "hue", 0_i64, |w, id, ty| i64::from((ty.hue)(w, id)));
    iset!(e, "hue", i64, |w, id, ty, v| (ty.set_hue)(w, id, i32_of(v)));
    iget!(e, "item_id", 0_i64, |w, id, _ty| i64::from(
        w.item(id).map_or(0, |i| i.item_id)
    ));
    iset!(e, "item_id", i64, |w, id, _ty, v| base::set_item_id(
        w,
        id,
        i32_of(v)
    ));
    iget!(e, "amount", 0_i64, |w, id, _ty| i64::from(
        w.item(id).map_or(0, |i| i.amount)
    ));
    iset!(e, "amount", i64, |w, id, _ty, v| ops::set_amount(
        w,
        id,
        i32_of(v)
    ));
    iget!(e, "parent", Dynamic::UNIT, |w, id, _ty| entity(
        w.item(id).and_then(|i| i.parent)
    ));
    iget!(e, "root", Dynamic::UNIT, |w, id, _ty| entity(
        ops::root_parent(w, id)
    ));
    e.register_get("deleted", |ctx: NativeCallContext, i: &mut Item| {
        let i = *i;
        with_world(&ctx, |w| live_i(w, i).is_none())
    });
    e.register_fn("delete", |ctx: NativeCallContext, i: Item| {
        with_world(&ctx, |w| {
            if live_i(w, i).is_some() {
                base::delete(w, i.0);
            }
        })
    });
    // Magic properties, named as in the JSON templates: `set_attr(item, "Attributes", "Luck", 200)`.
    e.register_fn(
        "set_attr",
        |ctx: NativeCallContext, i: Item, bag: &str, name: &str, v: i64| -> R<bool> {
            let Some((kind, bit)) = attr_bit(bag, name) else {
                return Ok(false);
            };
            with_world(&ctx, |w| {
                live_i(w, i).is_some_and(|_| {
                    crate::items::set_attr(w, i.0, kind, bit, i32_of(v));
                    true
                })
            })
        },
    );
    e.register_fn(
        "attr",
        |ctx: NativeCallContext, i: Item, bag: &str, name: &str| -> R<i64> {
            let Some((kind, bit)) = attr_bit(bag, name) else {
                return Ok(0);
            };
            with_world(&ctx, |w| {
                live_i(w, i).map_or(0, |_| i64::from(crate::items::attr(w, i.0, kind, bit)))
            })
        },
    );
    // `SkillBonuses.SetValues(index, skill, bonus)`, index 0..4.
    e.register_fn(
        "skill_bonus",
        |ctx: NativeCallContext, i: Item, index: i64, skill: &str, bonus: f64| -> R<bool> {
            let Ok(skill) = crate::templates::mobile_steps::skill(skill) else {
                return Ok(false);
            };
            if !(0..5).contains(&index) {
                return Ok(false);
            }
            with_world(&ctx, |w| {
                live_i(w, i).is_some_and(|_| {
                    let v = crate::aos::Attrs::skill_bonus_value(skill, bonus);
                    crate::items::set_attr(
                        w,
                        i.0,
                        crate::aos::AttrKind::SkillBonuses,
                        1 << index,
                        v,
                    );
                    true
                })
            })
        },
    );
    // Any `[set` property: `set_prop(item, "Quality", "Exceptional")`.
    e.register_fn(
        "set_prop",
        |ctx: NativeCallContext, i: Item, path: &str, value: Dynamic| -> R<()> {
            with_world(&ctx, |w| {
                if live_i(w, i).is_some() {
                    crate::templates::item_steps::set_property(w, i.0, path, &value.to_string());
                }
            })
        },
    );
    // Not a wrapper of one engine call: `new T()` then `container.DropItem(item)`, for a corpse in `on_death`.
    e.register_fn(
        "add_item",
        |ctx: NativeCallContext, c: Item, class: &str, amount: i64| {
            with_world(&ctx, |w| {
                live_i(w, c)?;
                let it = make_item(w, class, amount)?;
                muo_server::item::container::drop_item_default(w, c.0, it);
                Some(it)
            })
            .map(|it| entity(it.map(EntityId::Item)))
        },
    );
}

fn world_api(e: &mut Engine) {
    // The draws of `Utility.RandomMinMax`, `Utility.Random(n)`, `RandomBool`, `RandomDouble` and `RandomList`, so a
    // script reproduces a C#-ported constructor exactly.
    e.register_fn("random", |ctx: NativeCallContext, min: i64, max: i64| {
        with_world(&ctx, |w| match (i32::try_from(min), i32::try_from(max)) {
            (Ok(a), Ok(b)) => i64::from(muo_primitives::utility::random_min_max(&mut w.rng, a, b)),
            _ => muo_primitives::utility::random_min_max_i64(&mut w.rng, min, max),
        })
    });
    e.register_fn("random_below", |ctx: NativeCallContext, n: i64| {
        with_world(&ctx, |w| {
            i64::from(muo_primitives::utility::random(&mut w.rng, i32_of(n)))
        })
    });
    e.register_fn("random_bool", |ctx: NativeCallContext| {
        with_world(&ctx, |w| muo_primitives::utility::random_bool(&mut w.rng))
    });
    e.register_fn("random_double", |ctx: NativeCallContext| {
        with_world(&ctx, |w| muo_primitives::utility::random_double(&mut w.rng))
    });
    e.register_fn(
        "pick",
        |ctx: NativeCallContext, list: rhai::Array| -> R<Dynamic> {
            if list.is_empty() {
                return Ok(Dynamic::UNIT);
            }
            with_world(&ctx, |w| {
                let i = muo_primitives::utility::random(&mut w.rng, list.len() as i32) as usize;
                list[i].clone()
            })
        },
    );
    e.register_fn("log", |text: &str| tracing::info!("[script] {text}"));
    e.register_fn(
        "spawn_mobile",
        |ctx: NativeCallContext, class: &str, x: i64, y: i64, z: i64, map: i64| {
            with_world(&ctx, |w| {
                let map = MapId(u8::try_from(map).ok()?);
                let m = crate::factory::make_mobile(w, class)?;
                let p = Point3D::new(i32_of(x), i32_of(y), i32_of(z));
                muo_server::mobile::base::move_to_world(w, m, p, map);
                Some(m)
            })
            .map(|m| entity(m.map(EntityId::Mobile)))
        },
    );
    e.register_fn(
        "timer",
        |ctx: NativeCallContext, ms: i64, f: ImmutableString, arg: Dynamic| {
            let script = ctx
                .tag()
                .and_then(|t| t.read_lock::<Ctx>())
                .map(|c| c.script.clone());
            let script = script.unwrap_or_default();
            with_world(&ctx, |w| {
                let delay = TimeSpan::from_millis(ms.max(0));
                muo_server::timer::start(w, delay, TimeSpan::ZERO, 1, move |w| {
                    super::fire_timer(w, &script, &f, arg.clone());
                });
            })
        },
    );
}

/// The bag and bit of a magic property by the names the JSON templates use.
fn attr_bit(bag: &str, name: &str) -> Option<(crate::aos::AttrKind, u32)> {
    use crate::aos::*;
    let (kind, names) = match bag {
        "Attributes" => (AttrKind::Attributes, AosAttribute::NAMES),
        "WeaponAttributes" => (AttrKind::Weapon, AosWeaponAttribute::NAMES),
        "ArmorAttributes" => (AttrKind::Armor, AosArmorAttribute::NAMES),
        "ElementAttributes" => (AttrKind::Element, AosElementAttribute::NAMES),
        _ => return None,
    };
    names
        .iter()
        .find(|(n, _)| *n == name)
        .map(|(_, b)| (kind, *b))
}

/// One function per template key, built-in or registered by a content crate, named exactly like the key
/// (`str(m, [96, 120])`, `damageTypes(m, #{ Physical: 100 })`, `creatureHue(m, ..)`): the value is the key's JSON
/// value and the step runs the same setter the template did. A rhai object map iterates in key order, not written
/// order, so a statement whose map entries draw the RNG in sequence is written one entry per call, or with the
/// one-entry functions `resist`, `skill`, `damage_type`, `skill_base`.
pub(crate) fn register_steps(e: &mut Engine, types: &muo_server::type_registry::TypeRegistry) {
    use crate::templates::keys::{ItemKeys, Keys, MobileKeys};
    use crate::templates::{item_steps, mobile_steps};
    fn json(key: &str, v: &Dynamic) -> R<serde_json::Value> {
        to_json(v).map_err(|e| format!("{key}: {e}").into())
    }
    let mut mobile_keys: Vec<&'static str> = mobile_steps::KEYS.to_vec();
    if let Some(k) = types.extensions.get::<MobileKeys>() {
        mobile_keys.extend(k.0.keys().copied());
    }
    let mut item_keys: Vec<&'static str> = item_steps::KEYS.to_vec();
    if let Some(k) = types.extensions.get::<ItemKeys>() {
        item_keys.extend(k.0.keys().copied());
    }
    for key in mobile_keys {
        e.register_fn(
            key,
            move |ctx: NativeCallContext, m: Mobile, v: Dynamic| -> R<()> {
                let v = json(key, &v)?;
                with_world(&ctx, |w| {
                    if live_m(w, m).is_none() {
                        return Ok(());
                    }
                    let step = mobile_steps::parse(key, &v, &Keys::of(&w.types))
                        .map_err(|e| format!("{key}: {e}"))?;
                    mobile_steps::apply(w, m.0, &step);
                    Ok::<(), Box<EvalAltResult>>(())
                })?
            },
        );
    }
    for key in item_keys {
        e.register_fn(
            key,
            move |ctx: NativeCallContext, i: Item, v: Dynamic| -> R<()> {
                let v = json(key, &v)?;
                with_world(&ctx, |w| {
                    if live_i(w, i).is_none() {
                        return Ok(());
                    }
                    let step = item_steps::parse(key, &v, &Keys::of(&w.types))
                        .map_err(|e| format!("{key}: {e}"))?;
                    item_steps::apply(w, i.0, &step);
                    Ok::<(), Box<EvalAltResult>>(())
                })?
            },
        );
    }
    // one-entry forms of the map keys whose entries draw the RNG in order
    fn one(
        ctx: &NativeCallContext,
        m: Mobile,
        key: &'static str,
        name: &str,
        v: serde_json::Value,
    ) -> R<()> {
        let mut o = serde_json::Map::new();
        o.insert(name.to_owned(), v);
        with_world(ctx, |w| {
            if live_m(w, m).is_none() {
                return Ok(());
            }
            let step = crate::templates::mobile_steps::parse(key, &o.into(), &Keys::of(&w.types))
                .map_err(|e| format!("{key}: {e}"))?;
            crate::templates::mobile_steps::apply(w, m.0, &step);
            Ok::<(), Box<EvalAltResult>>(())
        })?
    }
    e.register_fn(
        "resist",
        |ctx: NativeCallContext, m: Mobile, n: &str, v: i64| one(&ctx, m, "resists", n, v.into()),
    );
    e.register_fn(
        "resist",
        |ctx: NativeCallContext, m: Mobile, n: &str, a: i64, b: i64| {
            one(&ctx, m, "resists", n, serde_json::json!([a, b]))
        },
    );
    e.register_fn(
        "damage_type",
        |ctx: NativeCallContext, m: Mobile, n: &str, v: i64| {
            one(&ctx, m, "damageTypes", n, v.into())
        },
    );
    e.register_fn(
        "damage_type",
        |ctx: NativeCallContext, m: Mobile, n: &str, a: i64, b: i64| {
            one(&ctx, m, "damageTypes", n, serde_json::json!([a, b]))
        },
    );
    e.register_fn(
        "skill",
        |ctx: NativeCallContext, m: Mobile, n: &str, v: f64| one(&ctx, m, "skills", n, v.into()),
    );
    e.register_fn(
        "skill",
        |ctx: NativeCallContext, m: Mobile, n: &str, a: f64, b: f64| {
            one(&ctx, m, "skills", n, serde_json::json!([a, b]))
        },
    );
    e.register_fn(
        "skill_base",
        |ctx: NativeCallContext, m: Mobile, n: &str, v: f64| one(&ctx, m, "skillBase", n, v.into()),
    );
    // an object whose entries keep their written order (a rhai map iterates sorted): `block([#{a: 1}, #{b: 2}])`
    e.register_type_with_name::<Block>("Block");
    e.register_fn("block", |entries: rhai::Array| -> R<Block> {
        entries
            .into_iter()
            .map(|d| {
                let m = d
                    .try_cast::<rhai::Map>()
                    .filter(|m| m.len() == 1)
                    .ok_or("block: every entry is a one-key map")?;
                let (k, v) = m.into_iter().next().ok_or("block: an empty entry")?;
                Ok((k.to_string(), v))
            })
            .collect::<R<Vec<_>>>()
            .map(Block)
    });
    // `Core.AOS`-style checks for constructors: `if era_at_least("AOS") { .. }`
    e.register_fn(
        "era_at_least",
        |ctx: NativeCallContext, era: &str| -> R<bool> {
            let Some(at) = (0..64)
                .filter_map(muo_data::expansion::Expansion::from_index)
                .find(|x| format!("{x:?}") == era)
            else {
                return Err(format!("unknown era {era}").into());
            };
            with_world(&ctx, |w| w.expansion() >= at)
        },
    );
}

/// [`register_steps`]' `block(..)`: an object in written order.
#[derive(Clone, Debug)]
pub(crate) struct Block(pub Vec<(String, Dynamic)>);

/// A script value as the JSON a template key takes: maps become objects (sorted), `block(..)` objects in written
/// order, arrays, numbers (int stays int), strings, bools, `()` null.
pub(crate) fn to_json(v: &Dynamic) -> Result<serde_json::Value, String> {
    use serde_json::Value;
    if v.is_unit() {
        return Ok(Value::Null);
    }
    if let Some(b) = v.read_lock::<Block>() {
        let mut o = serde_json::Map::new();
        for (k, v) in &b.0 {
            o.insert(k.clone(), to_json(v)?);
        }
        return Ok(Value::Object(o));
    }
    if let Some(m) = v.read_lock::<rhai::Map>() {
        let mut o = serde_json::Map::new();
        for (k, v) in m.iter() {
            o.insert(k.to_string(), to_json(v)?);
        }
        return Ok(Value::Object(o));
    }
    if let Some(a) = v.read_lock::<rhai::Array>() {
        return a
            .iter()
            .map(to_json)
            .collect::<Result<_, _>>()
            .map(Value::Array);
    }
    if let Ok(i) = v.as_int() {
        return Ok(i.into());
    }
    if let Ok(f) = v.as_float() {
        return serde_json::Number::from_f64(f)
            .map(Value::Number)
            .ok_or_else(|| format!("{f} is not a JSON number"));
    }
    if let Ok(b) = v.as_bool() {
        return Ok(b.into());
    }
    if let Some(s) = v.read_lock::<rhai::ImmutableString>() {
        return Ok(Value::String(s.to_string()));
    }
    Err(format!("{} is not a template value", v.type_name()))
}
