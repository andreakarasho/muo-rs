//! The class syntax people write:
//!
//! ```rhai
//! mobile("Orc", |c| {
//!     c.corpse_name = "an orcish corpse";
//!     c.loot = "Meager";
//!     c.on_create(|m| {
//!         m.body = 17;
//!         m.str = 96..=120;
//!         m.resist("Fire", 20..=30);
//!     });
//!     c.on_death(|m, corpse| m.entity.say("Urg..."));
//! });
//! ```
//!
//! `mobile` / `item` run the setup closure once at load with a class handle `c`. `c.<name> = value` sets a class
//! value (a template prop: `corpse_name` is the prop `corpseName`), except the constructor settings (`ai`,
//! `fight_mode`, `range_perception`, `range_fight`, `extends`, `inherits`) and a few short names (`weight`,
//! `label`, `rarity`, `name`, `speech`, `opposition`, `loot`). `c.on_create(|e| ..)` is the constructor body;
//! `c.on_use`, `on_death`, `on_think`, `on_damage`, `on_hear` are the behavior hooks; `c.on_outfit(|m| ..)` an NPC's
//! outfit.
//!
//! In `on_create` the entity is a construction handle (`NewMobile` / `NewItem`): `m.<name> = value` runs the
//! template step of that name (`m.str = 96..=120` is `SetStr(96, 120)`, `m.hits` sets the hits maximum as the C#
//! constructor did), so a class written this way draws the RNG exactly as its template form. `m.entity` is the
//! live mobile for everything else. In hooks the entity is the live handle (`m.hits` is the current hits).

use std::cell::RefCell;
use std::rc::Rc;

use rhai::{Dynamic, Engine, FnPtr, Map, NativeCallContext};

use super::api::{Item, Mobile, R, live_i, live_m, to_json, with_world};
use super::classes::{Collector, Decl};
use crate::aos::{
    AosArmorAttribute, AosAttribute, AosElementAttribute, AosWeaponAttribute, AttrKind,
};
use crate::templates::keys::Keys;
use crate::templates::{Kind, item_steps, mobile_steps};
use muo_server::ids::{ItemId, MobileId};
use muo_server::world::World;

/// A class being declared: what `c.<name> = ..` and `c.on_..(..)` set.
#[derive(Default)]
pub(crate) struct Draft {
    pub meta: Map,
    pub props: Map,
    pub ctor: Option<FnPtr>,
    pub hooks: Vec<(String, FnPtr)>,
}

/// `c` in `mobile("Orc", |c| ..)`. Load-time only.
#[derive(Clone, Default)]
pub struct ClassHandle(pub(crate) Rc<RefCell<Draft>>);

/// The entity in an `on_create`: its properties are the template steps.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NewMobile(pub MobileId);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NewItem(pub ItemId);

/// `i.attributes`, `i.weapon_attributes`, ..: `i.attributes.luck = 200`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AttrBag {
    item: ItemId,
    kind: AttrKind,
}

/// `fight_mode` -> `fightMode`; a name already in camel case stays as it is.
pub fn camel(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut up = false;
    for ch in s.chars() {
        if ch == '_' {
            up = true;
        } else if up {
            out.extend(ch.to_uppercase());
            up = false;
        } else {
            out.push(ch);
        }
    }
    out
}

/// `defend_chance` -> `DefendChance`.
fn pascal(s: &str) -> String {
    let c = camel(s);
    let mut it = c.chars();
    it.next()
        .map(|f| f.to_uppercase().chain(it).collect())
        .unwrap_or_default()
}

/// `"Melee"` -> `"AiMelee"`, keeping a full name; applied inside a `coin` / `oneOf` choice too.
fn ai_name(v: Dynamic) -> Dynamic {
    fn one(v: Dynamic) -> Dynamic {
        match v.clone().into_string() {
            Ok(s) if !s.starts_with("Ai") => format!("Ai{s}").into(),
            _ => v,
        }
    }
    if let Some(m) = v.clone().try_cast::<Map>() {
        let m: Map = m
            .into_iter()
            .map(|(k, x)| {
                let x = match x.clone().try_cast::<rhai::Array>() {
                    Some(a) => Dynamic::from_array(a.into_iter().map(one).collect()),
                    None => x,
                };
                (k, x)
            })
            .collect();
        return Dynamic::from_map(m);
    }
    one(v)
}

/// `c.loot = "Meager"` / `["Meager", ["Rich", 2]]` -> `[["Meager", 1], ["Rich", 2]]`.
fn loot(v: Dynamic) -> Dynamic {
    let one = |x: Dynamic| -> Dynamic {
        if x.is_string() {
            Dynamic::from_array(vec![x, Dynamic::from_int(1)])
        } else {
            x
        }
    };
    match v.clone().try_cast::<rhai::Array>() {
        Some(a) => Dynamic::from_array(a.into_iter().map(one).collect()),
        None if v.is_string() => Dynamic::from_array(vec![one(v)]),
        None => v,
    }
}

/// `c.<name> = v`.
fn class_set(c: &mut ClassHandle, name: &str, v: Dynamic) {
    let mut d = c.0.borrow_mut();
    let meta = |k: &str| k.to_owned();
    match name {
        "ai" => {
            d.meta.insert(meta("ai").into(), ai_name(v));
        }
        "fight_mode" | "range_perception" | "range_fight" | "inherits" | "script" => {
            d.meta.insert(camel(name).into(), v);
        }
        "extends" => {
            d.meta.insert("parent".into(), v);
        }
        "weight" => {
            d.props.insert("defaultWeight".into(), v);
        }
        "label" => {
            d.props.insert("labelNumber".into(), v);
        }
        "rarity" => {
            d.props.insert("artifactRarity".into(), v);
        }
        "name" => {
            d.props.insert("defaultName".into(), v);
        }
        "speech" => {
            d.props.insert("speechType".into(), v);
        }
        "opposition" => {
            d.props.insert("oppositionGroup".into(), v);
        }
        "loot" => {
            d.props.insert("loot".into(), loot(v));
        }
        _ => {
            d.props.insert(camel(name).into(), v);
        }
    }
}

/// Runs the template step `key` (camel case) on a new entity.
fn mobile_step(ctx: &NativeCallContext, m: MobileId, key: &str, v: &Dynamic) -> R<()> {
    let v = to_json(v).map_err(|e| format!("{key}: {e}"))?;
    with_world(ctx, |w| {
        trace(w, key);
        if live_m(w, Mobile(m)).is_none() {
            return Ok(());
        }
        let step = mobile_steps::parse(key, &v, &Keys::of(&w.types))
            .map_err(|e| format!("m.{key}: {e}"))?;
        mobile_steps::apply(w, m, &step);
        Ok::<(), Box<rhai::EvalAltResult>>(())
    })?
}

fn item_step(ctx: &NativeCallContext, i: ItemId, key: &str, v: &Dynamic) -> R<()> {
    let v = to_json(v).map_err(|e| format!("{key}: {e}"))?;
    with_world(ctx, |w| {
        trace(w, key);
        if live_i(w, Item(i)).is_none() {
            return Ok(());
        }
        let step =
            item_steps::parse(key, &v, &Keys::of(&w.types)).map_err(|e| format!("i.{key}: {e}"))?;
        item_steps::apply(w, i, &step);
        Ok::<(), Box<rhai::EvalAltResult>>(())
    })?
}

fn trace(w: &mut World, key: &str) {
    use crate::templates::keys::{ItemKeys, MobileKeys};
    if w.resources
        .get::<crate::templates::StepTrace>()
        .is_none_or(|t| t.0.is_none())
    {
        return;
    }
    let registered = w
        .types
        .extensions
        .get::<ItemKeys>()
        .and_then(|k| k.0.get_key_value(key).map(|(k, _)| *k))
        .or_else(|| {
            w.types
                .extensions
                .get::<MobileKeys>()
                .and_then(|k| k.0.get_key_value(key).map(|(k, _)| *k))
        });
    let found = mobile_steps::KEYS
        .iter()
        .chain(item_steps::KEYS)
        .find(|k| **k == key)
        .copied()
        .or(registered);
    if let (Some(k), Some(t)) = (
        found,
        &mut w.resources.get_mut::<crate::templates::StepTrace>().0,
    ) {
        t.push(k);
    }
}

/// The step name of `m.<name>`: the short names, else camel case.
fn mobile_key(name: &str) -> String {
    match name {
        "armor" => "virtualArmor".into(),
        "sound" => "baseSoundId".into(),
        _ => camel(name),
    }
}

/// A one-entry map step (`resists`, `skills`, ..) from `(name, value)`.
fn one_entry(name: &str, v: Dynamic) -> Dynamic {
    let mut m = Map::new();
    m.insert(name.into(), v);
    Dynamic::from_map(m)
}

pub(crate) fn register(e: &mut Engine, c: &Rc<RefCell<Collector>>) {
    e.register_type_with_name::<ClassHandle>("Class")
        .register_type_with_name::<NewMobile>("NewMobile")
        .register_type_with_name::<NewItem>("NewItem")
        .register_type_with_name::<AttrBag>("Attributes");

    register_shops(e, c);

    // declarations
    for (fname, kind) in [("mobile", Kind::Mobile), ("item", Kind::Item)] {
        let c = c.clone();
        e.register_fn(
            fname,
            move |ctx: NativeCallContext, name: &str, setup: FnPtr| -> R<()> {
                let h = ClassHandle::default();
                let _ = setup.call_within_context::<Dynamic>(&ctx, (h.clone(),))?;
                let d = h.0.take();
                let mut meta = d.meta;
                if !d.props.is_empty() {
                    meta.insert("props".into(), Dynamic::from_map(d.props));
                }
                let mut col = c.borrow_mut();
                let script = col.current;
                col.decls.push(Decl {
                    name: name.to_owned(),
                    meta,
                    ctor: d.ctor,
                    script,
                    kind: Some(kind),
                    hooks: d.hooks,
                });
                Ok(())
            },
        );
    }

    // the class handle
    e.register_indexer_set(|c: &mut ClassHandle, name: &str, v: Dynamic| class_set(c, name, v));
    // a class value by its exact prop name, for one a short name would map elsewhere
    e.register_fn("prop", |c: &mut ClassHandle, name: &str, v: Dynamic| {
        c.0.borrow_mut().props.insert(name.into(), v);
    });
    e.register_fn("on_create", |c: &mut ClassHandle, f: FnPtr| {
        c.0.borrow_mut().ctor = Some(f);
    });
    for hook in [
        "on_use",
        "on_death",
        "on_think",
        "on_damage",
        "on_hear",
        "on_outfit",
        "on_move_over",
    ] {
        e.register_fn(hook, move |c: &mut ClassHandle, f: FnPtr| {
            c.0.borrow_mut().hooks.push((hook.to_owned(), f));
        });
    }
    // `c.ai = coin("Melee", "Mage")` / `one_of([..])`: drawn when a creature is made
    e.register_fn("coin", |a: Dynamic, b: Dynamic| {
        one_entry("coin", Dynamic::from_array(vec![a, b]))
    });
    e.register_fn("one_of", |a: rhai::Array| {
        one_entry("oneOf", Dynamic::from_array(a))
    });

    // the new mobile
    e.register_indexer_set(
        |ctx: NativeCallContext, m: &mut NewMobile, name: &str, v: Dynamic| {
            // `m.name = random_name("orc")` is `NameRandom`
            if name == "name"
                && let Some(k) = v
                    .clone()
                    .try_cast::<Map>()
                    .and_then(|m| m.get("$randomName").cloned())
            {
                return mobile_step(&ctx, m.0, "nameRandom", &k);
            }
            mobile_step(&ctx, m.0, &mobile_key(name), &v)
        },
    );
    e.register_get("entity", |m: &mut NewMobile| Mobile(m.0));
    for (f, key) in [
        ("resist", "resists"),
        ("damage_type", "damageTypes"),
        ("skill", "skills"),
        ("skill_base", "skillBase"),
    ] {
        e.register_fn(
            f,
            move |ctx: NativeCallContext, m: &mut NewMobile, n: &str, v: Dynamic| {
                mobile_step(&ctx, m.0, key, &one_entry(n, v))
            },
        );
        e.register_fn(
            f,
            move |ctx: NativeCallContext, m: &mut NewMobile, n: &str, a: Dynamic, b: Dynamic| {
                mobile_step(
                    &ctx,
                    m.0,
                    key,
                    &one_entry(n, Dynamic::from_array(vec![a, b])),
                )
            },
        );
    }
    // `m.step("creatureHue", v)`: any key by its exact name
    e.register_fn(
        "step",
        |ctx: NativeCallContext, m: &mut NewMobile, key: &str, v: Dynamic| {
            mobile_step(&ctx, m.0, key, &v)
        },
    );
    e.register_fn("random_name", |kind: &str| {
        one_entry("$randomName", kind.into())
    });

    // the new item
    e.register_indexer_set(
        |ctx: NativeCallContext, i: &mut NewItem, name: &str, v: Dynamic| {
            item_step(&ctx, i.0, &camel(name), &v)
        },
    );
    e.register_get("entity", |i: &mut NewItem| Item(i.0));
    e.register_fn(
        "step",
        |ctx: NativeCallContext, i: &mut NewItem, key: &str, v: Dynamic| {
            item_step(&ctx, i.0, key, &v)
        },
    );

    // magic properties, on new and live items
    for (prop, kind) in [
        ("attributes", AttrKind::Attributes),
        ("weapon_attributes", AttrKind::Weapon),
        ("armor_attributes", AttrKind::Armor),
        ("element_attributes", AttrKind::Element),
    ] {
        e.register_get(prop, move |i: &mut NewItem| AttrBag { item: i.0, kind });
        e.register_get(prop, move |i: &mut Item| AttrBag { item: i.0, kind });
        // `i.attributes.luck = 200` writes the bag back after setting the value: nothing to store
        e.register_set(prop, |_: &mut NewItem, _: AttrBag| {});
        e.register_set(prop, |_: &mut Item, _: AttrBag| {});
    }
    e.register_indexer_set(
        |ctx: NativeCallContext, b: &mut AttrBag, name: &str, v: i64| -> R<()> {
            let names: &[(&str, u32)] = match b.kind {
                AttrKind::Attributes => AosAttribute::NAMES,
                AttrKind::Weapon => AosWeaponAttribute::NAMES,
                AttrKind::Armor => AosArmorAttribute::NAMES,
                AttrKind::Element => AosElementAttribute::NAMES,
                AttrKind::SkillBonuses => &[],
            };
            let p = pascal(name);
            let bit = names
                .iter()
                .find(|(n, _)| *n == p)
                .map(|(_, b)| *b)
                .ok_or_else(|| format!("unknown attribute {name}"))?;
            let (item, kind) = (b.item, b.kind);
            with_world(&ctx, |w| {
                if live_i(w, Item(item)).is_some() {
                    crate::items::set_attr(w, item, kind, bit, super::api::i32_of(v));
                }
            })
        },
    );
    e.register_indexer_get(
        |ctx: NativeCallContext, b: &mut AttrBag, name: &str| -> R<i64> {
            let names: &[(&str, u32)] = match b.kind {
                AttrKind::Attributes => AosAttribute::NAMES,
                AttrKind::Weapon => AosWeaponAttribute::NAMES,
                AttrKind::Armor => AosArmorAttribute::NAMES,
                AttrKind::Element => AosElementAttribute::NAMES,
                AttrKind::SkillBonuses => &[],
            };
            let p = pascal(name);
            let Some(bit) = names.iter().find(|(n, _)| *n == p).map(|(_, b)| *b) else {
                return Err(format!("unknown attribute {name}").into());
            };
            let (item, kind) = (b.item, b.kind);
            with_world(&ctx, |w| i64::from(crate::items::attr(w, item, kind, bit)))
        },
    );

    // helpers for constructors, with the RNG draws of the C# statements
    e.register_fn("chance", |ctx: NativeCallContext, p: f64| {
        with_world(&ctx, |w| {
            muo_primitives::utility::random_double(&mut w.rng) < p
        })
    });
    e.register_fn("coin", |ctx: NativeCallContext| {
        with_world(&ctx, |w| muo_primitives::utility::random_bool(&mut w.rng))
    });
    e.register_fn("era", |ctx: NativeCallContext, era: &str| -> R<bool> {
        let Some(at) = (0..64)
            .filter_map(muo_data::expansion::Expansion::from_index)
            .find(|x| format!("{x:?}") == era)
        else {
            return Err(format!("unknown era {era}").into());
        };
        with_world(&ctx, |w| w.expansion() >= at)
    });
}

/// `s` in `shop("SBBlacksmith", |s| ..)`: the statements of a vendor's buy and sell lists, in order.
#[derive(Clone, Default)]
pub struct ShopHandle(Rc<RefCell<rhai::Array>>);

impl ShopHandle {
    fn push(&self, m: Map) {
        self.0.borrow_mut().push(Dynamic::from_map(m));
    }
}

fn map(entries: impl IntoIterator<Item = (&'static str, Dynamic)>) -> Map {
    entries.into_iter().map(|(k, v)| (k.into(), v)).collect()
}

/// Runs a nested `|s| ..` block of a shop and returns its statements.
fn shop_block(ctx: &NativeCallContext, f: &FnPtr) -> R<Dynamic> {
    let h = ShopHandle::default();
    let _ = f.call_within_context::<Dynamic>(ctx, (h.clone(),))?;
    let list = h.0.take();
    Ok(Dynamic::from_array(list))
}

/// A conditional statement: `{kind: value, "then": [..], "else": [..]}`.
fn shop_if(
    ctx: &NativeCallContext,
    s: &ShopHandle,
    kind: &'static str,
    v: Dynamic,
    then: &FnPtr,
    otherwise: Option<&FnPtr>,
) -> R<()> {
    let mut m = map([(kind, v), ("then", shop_block(ctx, then)?)]);
    if let Some(e) = otherwise {
        m.insert("else".into(), shop_block(ctx, e)?);
    }
    s.push(m);
    Ok(())
}

/// Class names a shop names by their short form become full names (`"Tongs"` -> `"Server.Items.Tongs"`). A strict
/// (server) load rejects a sold item that is no registered class.
pub(crate) fn resolve_shop_names(
    w: &World,
    v: &mut serde_json::Value,
    strict: bool,
) -> Result<(), String> {
    match v {
        serde_json::Value::Array(a) => {
            for x in a {
                resolve_shop_names(w, x, strict)?;
            }
        }
        serde_json::Value::Object(o) => {
            for (k, x) in o.iter_mut() {
                match (k.as_str(), x) {
                    ("sell" | "buy", serde_json::Value::String(n)) => {
                        let full = super::classes::full_name(w, n)?;
                        // what a vendor sells is built when bought, so it must be a class; what it buys only matches
                        // items by type (the port keeps C#'s `typeof(PresetMapEntry)`, which never matches)
                        if strict
                            && k == "buy"
                            && w.types.find_type_by_full_name(&full, false).is_none()
                        {
                            return Err(format!("{n} is not a class"));
                        }
                        *n = full;
                    }
                    (_, x) => resolve_shop_names(w, x, strict)?,
                }
            }
        }
        _ => {}
    }
    Ok(())
}

fn register_shops(e: &mut Engine, c: &Rc<RefCell<Collector>>) {
    e.register_type_with_name::<ShopHandle>("Shop");
    let c = c.clone();
    e.register_fn(
        "shop",
        move |ctx: NativeCallContext, name: &str, setup: FnPtr| -> R<()> {
            let build = shop_block(&ctx, &setup)?;
            let entry = map([("name", name.into()), ("build", build)]);
            c.borrow_mut()
                .data
                .push(("vendors".into(), name.to_owned(), Dynamic::from_map(entry)));
            Ok(())
        },
    );
    // what the vendor buys from players: `GenericSellInfo.Add(type, price)`
    e.register_fn("buys", |s: &mut ShopHandle, class: &str, price: i64| {
        s.push(map([("sell", class.into()), ("price", price.into())]));
    });
    // what players buy from the vendor: `GenericBuyInfo(type, price, amount, itemID, hue)`; `extra` holds
    // `name`, `hue`, `arg`, `control_slots`, `beverage` (an amount may be `#{ from: a, count: c }`)
    fn sells(
        s: &mut ShopHandle,
        class: &str,
        price: i64,
        amount: Dynamic,
        item_id: i64,
        extra: Map,
    ) {
        let mut m = map([
            ("buy", class.into()),
            ("price", price.into()),
            ("amount", amount),
            ("itemId", item_id.into()),
        ]);
        for (k, v) in extra {
            m.insert(camel(&k).into(), v);
        }
        s.push(m);
    }
    e.register_fn(
        "sells",
        |s: &mut ShopHandle, class: &str, price: i64, amount: Dynamic, item_id: i64| {
            sells(s, class, price, amount, item_id, Map::new())
        },
    );
    e.register_fn(
        "sells",
        |s: &mut ShopHandle, class: &str, price: i64, amount: Dynamic, item_id: i64, extra: Map| {
            sells(s, class, price, amount, item_id, extra)
        },
    );
    e.register_fn(
        "era",
        |ctx: NativeCallContext, s: &mut ShopHandle, era: &str, then: FnPtr| {
            shop_if(&ctx, s, "era", era.into(), &then, None)
        },
    );
    e.register_fn(
        "era",
        |ctx: NativeCallContext, s: &mut ShopHandle, era: &str, then: FnPtr, otherwise: FnPtr| {
            shop_if(&ctx, s, "era", era.into(), &then, Some(&otherwise))
        },
    );
    e.register_fn(
        "when",
        |ctx: NativeCallContext, s: &mut ShopHandle, cond: &str, then: FnPtr| {
            shop_if(&ctx, s, "if", cond.into(), &then, None)
        },
    );
    e.register_fn(
        "when",
        |ctx: NativeCallContext, s: &mut ShopHandle, cond: &str, then: FnPtr, otherwise: FnPtr| {
            shop_if(&ctx, s, "if", cond.into(), &then, Some(&otherwise))
        },
    );
    e.register_fn(
        "one_in",
        |ctx: NativeCallContext, s: &mut ShopHandle, n: i64, then: FnPtr| {
            shop_if(&ctx, s, "oneIn", n.into(), &then, None)
        },
    );
    e.register_fn(
        "one_in",
        |ctx: NativeCallContext, s: &mut ShopHandle, n: i64, then: FnPtr, otherwise: FnPtr| {
            shop_if(&ctx, s, "oneIn", n.into(), &then, Some(&otherwise))
        },
    );
    e.register_fn("scroll_buy", |s: &mut ShopHandle, limit: i64| {
        s.push(map([("scrollBuy", limit.into())]))
    });
    e.register_fn("scroll_sell", |s: &mut ShopHandle, mult: i64| {
        s.push(map([("scrollSell", mult.into())]))
    });
    e.register_fn("preset_maps", |s: &mut ShopHandle| {
        s.push(map([("presetMaps", true.into())]))
    });
    // a statement by its exact JSON form, for anything the methods above do not cover
    e.register_fn("statement", |s: &mut ShopHandle, m: Map| s.push(m));
}

/// A new mobile or item handle for a constructor closure.
pub(crate) fn new_entity(e: muo_server::ids::EntityId) -> Dynamic {
    match e {
        muo_server::ids::EntityId::Mobile(m) => Dynamic::from(NewMobile(m)),
        muo_server::ids::EntityId::Item(i) => Dynamic::from(NewItem(i)),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn names_convert() {
        assert_eq!(super::camel("corpse_name"), "corpseName");
        assert_eq!(super::camel("aosStrReq"), "aosStrReq");
        assert_eq!(super::pascal("defend_chance"), "DefendChance");
        assert_eq!(super::pascal("luck"), "Luck");
    }
}
