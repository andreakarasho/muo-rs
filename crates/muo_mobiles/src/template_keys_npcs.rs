//! Template keys of the NPCs (`muo_content::templates::keys`), and the vendor class members read from `"props"`.
//!
//! A hue value (`npcHue`, `npcSpeechHue`, an item's `"hue"`) is a number or a random hue: `"randomDyed"`,
//! `"randomNeutral"`, `"randomBlue"`, `"randomGreen"`, `"randomRed"`, `"randomYellow"`, `"randomPink"`,
//! `"randomBright"`, `"randomNondyed"`, `"humanSkin"` (`Race.Human.RandomSkinHue()`), `"humanHair"`; one read from
//! the mobile being built: `"hairHue"` (`HairHue`), `"vendorRandom"` / `"vendorShoe"` (its vendor class's
//! `GetRandomHue()` / `GetShoeHue()`); `{"oneOf": [hues]}` (`Utility.Random(n) switch`), or `"$name"`, the hue an
//! earlier `npcHueVars` drew.
//!
//! An item value (`npcEquip`, `npcAddItem`, `npcReplaceEquipment` take an array of them) is a class name or
//! `{"item": "<class>", "hue": <hue>, "hueAfter": <hue>, "layer": "InnerTorso", "movable": false}`: `hue` is drawn
//! before the item is made (`new T(hue)`), `hueAfter` after (`new T { Hue = .. }`).
//!
//! Keys:
//! - `npcSpeechHue`, `npcHue`: `SpeechHue = ..`, `Hue = ..`.
//! - `npcRollHair: true`: `HairItemID = Race.RandomHair(Female); HairHue = Race.RandomHairHue();
//!   Race.RandomFacialHair(this);` (the facial roll is discarded, as the hireling constructors do).
//! - `npcHair: {"hue": <hue>, "facialSkipOneIn": n}`: `AssignRandomHair(this, hue)`, then
//!   `if (Utility.Random(n) != 0) AssignRandomFacialHair(this, hue)` with the same hue.
//! - `npcInitStats: [str, dex, int]`, `npcSpeed: [active, passive]`, `npcBlessed: true`, `npcNameHue: n`.
//! - `npcEquip` (`EquipItem`), `npcAddItem` (`AddItem`), `npcReplaceEquipment` (`ReplaceEquipment`).
//! - `npcGoldPack: [min, max]`: an unmovable backpack holding `new Gold(min, max)`.
//! - `npcHoldGold: n`: the `BaseHire(AIType)` constructor's `HoldGold = 8`.
//! - `npcVendorOutfit: true`: `BaseVendor.InitOutfit()` (in an `npcOutfit` prop).
//! - `npcTitleByGender: {"female": "..", "male": ".."}`: `Title = Female ? .. : ..`.
//! - `npcSeq: [{..}, {..}]`: blocks of template keys run in order (a block can repeat a key of an earlier one).
//! - `npcIfLayerEmpty: {"layer": "TwoHanded", "then": {..}}`: the block when nothing is on that layer.
//! - `npcHueVars: {"low": <hue>}`: draws hues now, read later as `"$low"` (`int lowHue = GetRandomHue();`).
//! - `npcByGender: {"female": {..}, "male": {..}}`: the block of the mobile's `Female`.
//! - `npcEquipOrDelete: {"item": <item>, "else": {..}}`: `if (!EquipItem(item)) { item.Delete(); else-block }`.
//! - `npcRehueLayers: {"layers": [..], "hue": <hue>}`: each worn item on those layers gets a hue drawn for it.
//! - `npcFacialHair: <hue>` (`AssignRandomFacialHair(this, hue)`), `npcHairItemId: n`, `npcHairHue: <hue>`.
//! - `npcDeleteBackpack: true`, `npcReplaceBackpack: {"weight": n}` (a new unmovable backpack of that weight).
//! - `npcUnmovablePack: [{"gold": [min, max]}, {"item": "<class>", "amount": n, "newbied": true}]`: the contents
//!   made in order, then an unmovable backpack holding them (the guards).
//! - `npcGuardWeapon: <item>`: unmovable, crafted by the mobile, exceptional.
//! - `npcMount: "<class>"` (`new T().Rider = this`), `npcCombatDelay: ms` (`NextCombatTime = Core.TickCount + ms`).
//! - `npcEscortOutfit: true` (`BaseEscortable.InitOutfit()`: the class's `npcOutfit` prop), `npcPrisoner: true`.
//!
//! Nested blocks are parsed when they run, so a mistake in one is logged then (`npcs::template_tests` parses every
//! shipped class).

use serde_json::Value;

use muo_content::hooks::rnd;
use muo_content::templates::keys::{Keys, check_bool, mobile_key};
use muo_content::templates::mobile_steps;
use muo_content::templates::props;
use muo_primitives::utility;
use muo_server::ids::{ItemId, MobileId};
use muo_server::layer::Layer;
use muo_server::type_registry::TypeRegistry;
use muo_server::world::World;

/// A hue: fixed, drawn when applied, read from the mobile being built, one of a list (`Utility.Random(n)` picks),
/// or a hue an earlier `npcHueVars` drew.
#[derive(Clone, Debug)]
pub enum Hue {
    Fixed(i32),
    Random(HueFn),
    Of(MobileHueFn),
    OneOf(Vec<Hue>),
    Var(String),
}

type HueFn = fn(&mut World) -> i32;
type MobileHueFn = fn(&mut World, MobileId) -> i32;

fn vendor_class(w: &World, id: MobileId) -> &'static muo_content::vendor::VendorClass {
    muo_content::vendor::class(w, id).unwrap_or(&muo_content::vendor::BASE_VENDOR_CLASS)
}

/// `HairHue`, and the vendor class's `GetRandomHue()` / `GetShoeHue()`.
const MOBILE_HUES: [(&str, MobileHueFn); 3] = [
    ("hairHue", |w, id| w.mobile(id).map_or(0, |m| m.hair_hue)),
    ("vendorRandom", |w, id| {
        (vendor_class(w, id).get_random_hue)(w, id)
    }),
    ("vendorShoe", |w, id| {
        (vendor_class(w, id).get_shoe_hue)(w, id)
    }),
];

/// The hues `npcHueVars` drew, by name (read while the same mobile is being built).
#[derive(Default)]
struct HueVars(std::collections::HashMap<String, i32>);

const RANDOM_HUES: [(&str, HueFn); 11] = [
    ("randomDyed", |w| utility::random_dyed_hue(&mut w.rng)),
    ("randomNeutral", |w| utility::random_neutral_hue(&mut w.rng)),
    ("randomBlue", |w| utility::random_blue_hue(&mut w.rng)),
    ("randomGreen", |w| utility::random_green_hue(&mut w.rng)),
    ("randomRed", |w| utility::random_red_hue(&mut w.rng)),
    ("randomYellow", |w| utility::random_yellow_hue(&mut w.rng)),
    ("randomPink", |w| utility::random_pink_hue(&mut w.rng)),
    ("randomBright", |w| utility::random_bright_hue(&mut w.rng)),
    ("randomNondyed", |w| utility::random_nondyed_hue(&mut w.rng)),
    ("humanSkin", muo_content::vendor::human_random_skin_hue),
    ("humanHair", |w| {
        muo_content::creature::init_ops::race_hair_hue(w, "Human")
    }),
];

impl Hue {
    pub fn parse(v: &Value) -> Result<Hue, String> {
        if let Some(n) = v.as_i64().and_then(|n| i32::try_from(n).ok()) {
            return Ok(Hue::Fixed(n));
        }
        if let Some(a) = v.get("oneOf").and_then(Value::as_array) {
            if a.is_empty() || v.as_object().is_some_and(|o| o.len() != 1) {
                return Err(format!("expected {{\"oneOf\": [hues]}}, got {v}"));
            }
            return a
                .iter()
                .map(Hue::parse)
                .collect::<Result<_, _>>()
                .map(Hue::OneOf);
        }
        let s = v
            .as_str()
            .ok_or_else(|| format!("expected a hue, got {v}"))?;
        if let Some(n) = s.strip_prefix('$') {
            return Ok(Hue::Var(n.to_owned()));
        }
        if let Some((_, f)) = MOBILE_HUES.iter().find(|(n, _)| *n == s) {
            return Ok(Hue::Of(*f));
        }
        RANDOM_HUES
            .iter()
            .find(|(n, _)| *n == s)
            .map(|(_, f)| Hue::Random(*f))
            .ok_or_else(|| format!("unknown hue {s}"))
    }

    /// The hue now; `id` is the mobile being built (a hue read from it is 0 without one).
    pub fn roll(&self, w: &mut World, id: Option<MobileId>) -> i32 {
        match self {
            Hue::Fixed(n) => *n,
            Hue::Random(f) => f(w),
            Hue::Of(f) => id.map_or(0, |id| f(w, id)),
            Hue::OneOf(l) => {
                let i = rnd::below(w, l.len() as i32) as usize;
                l[i].roll(w, id)
            }
            Hue::Var(n) => w
                .resources
                .get::<HueVars>()
                .and_then(|v| v.0.get(n).copied())
                .unwrap_or(0),
        }
    }
}

fn hue_of(w: &mut World, id: MobileId, v: &Value) -> i32 {
    Hue::parse(v).unwrap_or(Hue::Fixed(0)).roll(w, Some(id))
}

fn check_hue(v: &Value) -> Result<(), String> {
    Hue::parse(v).map(|_| ())
}

fn ints<const N: usize>(v: &Value) -> Option<[i32; N]> {
    let a = v.as_array()?;
    if a.len() != N {
        return None;
    }
    let mut out = [0; N];
    for (o, x) in out.iter_mut().zip(a) {
        *o = i32::try_from(x.as_i64()?).ok()?;
    }
    Some(out)
}

fn check_ints<const N: usize>(v: &Value) -> Result<(), String> {
    ints::<N>(v)
        .map(|_| ())
        .ok_or_else(|| format!("expected {N} integers, got {v}"))
}

/// An item value: the class, the hue drawn before it is made (`new T(hue)`) and the one set after
/// (`new T { Hue = .. }`).
fn item_of(v: &Value) -> Option<(&str, Option<Hue>, Option<Hue>)> {
    let hue = |k| match v.get(k) {
        None => Some(None),
        Some(h) => Hue::parse(h).ok().map(Some),
    };
    match v {
        Value::String(s) => Some((s, None, None)),
        Value::Object(o) => Some((o.get("item")?.as_str()?, hue("hue")?, hue("hueAfter")?)),
        _ => None,
    }
}

fn check_items(v: &Value) -> Result<(), String> {
    let a = v.as_array().ok_or("expected an array of items")?;
    for x in a {
        if let Some(o) = x.as_object() {
            if let Some(k) = o
                .keys()
                .find(|k| !["item", "hue", "hueAfter", "movable", "layer"].contains(&k.as_str()))
            {
                return Err(format!("unknown item key \"{k}\""));
            }
            for k in ["hue", "hueAfter"] {
                if let Some(h) = o.get(k) {
                    check_hue(h)?;
                }
            }
            if let Some(m) = o.get("movable") {
                check_bool(m)?;
            }
            if let Some(l) = o.get("layer") {
                let l = l.as_str().unwrap_or_default();
                layer(l).ok_or_else(|| format!("unknown layer {l}"))?;
            }
        }
        item_of(x).ok_or_else(|| format!("expected an item, got {x}"))?;
    }
    Ok(())
}

type Put = fn(&mut World, MobileId, ItemId);

/// Makes each item of an item array (its `hue` drawn first) and hands it to `put`, then sets its `hueAfter`.
fn items(w: &mut World, id: MobileId, v: &Value, put: Put) {
    for x in v.as_array().map(Vec::as_slice).unwrap_or_default() {
        item(w, id, x, put);
    }
}

/// One item value: made (its `hue` drawn first, then `layer`, `movable`), handed to `put`, then its `hueAfter`.
fn item(w: &mut World, id: MobileId, x: &Value, put: Put) -> Option<ItemId> {
    let (name, hue, after) = item_of(x)?;
    let hue = hue.map(|h| h.roll(w, Some(id)));
    let it = muo_content::factory::make_item(w, name)?;
    if let Some(h) = hue {
        muo_server::item::base::set_hue(w, it, h);
    }
    if let Some(l) = x.get("layer").and_then(Value::as_str).and_then(layer) {
        muo_server::item::base::set_layer(w, it, l);
    }
    if let Some(m) = x.get("movable").and_then(Value::as_bool) {
        muo_server::item::ops::set_movable(w, it, m);
    }
    put(w, id, it);
    if let Some(h) = after {
        let h = h.roll(w, Some(id));
        if w.item(it).is_some() {
            muo_server::item::base::set_hue(w, it, h);
        }
    }
    Some(it)
}

/// `EquipItem(item)`: a refused item stays unreferenced.
fn equip(w: &mut World, id: MobileId, it: ItemId) {
    muo_server::mobile::base::equip_item(w, id, it);
}

fn add_item(w: &mut World, id: MobileId, it: ItemId) {
    muo_server::mobile::part2::items::add_item(w, id, it);
}

/// `if (!EquipItem(item)) item.Delete();` (the refusal is read back by `npcEquipOrDelete`'s `else`).
fn equip_or_delete(w: &mut World, id: MobileId, it: ItemId) {
    if !muo_server::mobile::base::equip_item(w, id, it)
        && let Some(t) = w.item(it).map(|i| i.ty)
    {
        (t.delete)(w, it);
    }
}

fn check_block(v: &Value) -> Result<(), String> {
    v.as_object()
        .map(|_| ())
        .ok_or_else(|| format!("expected an object of template keys, got {v}"))
}

/// Parses then applies a block of template keys (a JSON object) on `id`.
pub fn run_block(w: &mut World, id: MobileId, v: &Value) {
    let Some(o) = v.as_object() else { return };
    let steps: Result<Vec<_>, String> = {
        let keys = Keys::of(&w.types);
        o.iter()
            .map(|(k, v)| mobile_steps::parse(k, v, &keys).map_err(|e| format!("\"{k}\": {e}")))
            .collect()
    };
    match steps {
        Ok(steps) => steps.iter().for_each(|s| mobile_steps::apply(w, id, s)),
        Err(e) => tracing::error!("Templates: a nested NPC block: {e}"),
    }
}

fn layer(s: &str) -> Option<Layer> {
    (0..=0x1F).map(Layer).find(|l| format!("{l:?}") == s)
}

pub(crate) fn register(types: &mut TypeRegistry) {
    mobile_key(types, "npcSpeechHue", check_hue, |w, id, v| {
        let h = hue_of(w, id, v);
        muo_content::creature::init_ops::speech_hue(w, id, h);
    });
    mobile_key(types, "npcHue", check_hue, |w, id, v| {
        let h = hue_of(w, id, v);
        muo_content::creature::init_ops::hue(w, id, h);
    });
    mobile_key(types, "npcRollHair", check_bool, |w, id, _| {
        muo_content::vendor::roll_hair_item(w, id);
        muo_content::vendor::roll_hair_hue(w, id);
        muo_content::vendor::roll_facial_hair(w, id);
    });
    mobile_key(
        types,
        "npcHair",
        |v| {
            let o = v
                .as_object()
                .ok_or("expected {\"hue\", \"facialSkipOneIn\"}")?;
            check_hue(o.get("hue").ok_or("\"hue\" is missing")?)?;
            match o.get("facialSkipOneIn") {
                Some(n) if n.as_i64().is_none_or(|n| n < 1) => {
                    Err("\"facialSkipOneIn\": expected a count".into())
                }
                _ => Ok(()),
            }
        },
        |w, id, v| {
            let h = hue_of(w, id, &v["hue"]);
            muo_content::vendor::assign_random_hair(w, id, h);
            let skip = v
                .get("facialSkipOneIn")
                .and_then(Value::as_i64)
                .map(|n| rnd::below(w, n as i32) == 0);
            if skip != Some(true) {
                muo_content::vendor::assign_random_facial_hair(w, id, h);
            }
        },
    );
    mobile_key(types, "npcInitStats", check_ints::<3>, |w, id, v| {
        if let Some([s, d, i]) = ints::<3>(v) {
            muo_content::creature::init_ops::init_stats(w, id, s, d, i);
        }
    });
    mobile_key(
        types,
        "npcSpeed",
        |v| match v.as_array().map(Vec::as_slice) {
            Some([a, p]) if a.is_number() && p.is_number() => Ok(()),
            _ => Err(format!("expected [active, passive], got {v}")),
        },
        |w, id, v| {
            let (a, p) = (v[0].as_f64().unwrap_or(0.2), v[1].as_f64().unwrap_or(0.4));
            muo_content::creature::speeds::set_speed(w, id, a, p, true);
        },
    );
    mobile_key(
        types,
        "npcNameHue",
        muo_content::templates::keys::check_int,
        |w, id, v| {
            if let Some(m) = w.mobile_mut(id) {
                m.name_hue = muo_content::templates::keys::int(v);
            }
        },
    );
    mobile_key(types, "npcBlessed", check_bool, |w, id, v| {
        muo_server::mobile::part1::props_a::set_blessed(w, id, v.as_bool() == Some(true));
    });
    mobile_key(types, "npcEquip", check_items, |w, id, v| {
        items(w, id, v, equip);
    });
    mobile_key(types, "npcAddItem", check_items, |w, id, v| {
        items(w, id, v, add_item);
    });
    mobile_key(types, "npcReplaceEquipment", check_items, |w, id, v| {
        items(w, id, v, crate::npcs::vendors::replace_equipment_item);
    });
    mobile_key(types, "npcGoldPack", check_ints::<2>, |w, id, v| {
        if let Some([a, b]) = ints::<2>(v) {
            crate::npcs::humans_support::add_gold_pack(w, id, a, b);
        }
    });
    mobile_key(
        types,
        "npcHoldGold",
        muo_content::templates::keys::check_int,
        |w, id, v| crate::npcs::hires::set_hold_gold(w, id, muo_content::templates::keys::int(v)),
    );
    mobile_key(
        types,
        "npcTitleByGender",
        |v| match (v.get("female"), v.get("male")) {
            (Some(Value::String(_)), Some(Value::String(_)))
                if v.as_object().is_some_and(|o| o.len() == 2) =>
            {
                Ok(())
            }
            _ => Err(format!(
                "expected {{\"female\": title, \"male\": title}}, got {v}"
            )),
        },
        |w, id, v| {
            let female = w.mobile(id).is_some_and(|m| m.female);
            let t = v[if female { "female" } else { "male" }]
                .as_str()
                .map(str::to_owned);
            muo_server::mobile::part1::props_a::set_title(w, id, t);
        },
    );
    mobile_key(types, "npcVendorOutfit", check_bool, |w, id, _| {
        muo_content::vendor::init_outfit(w, id);
    });
    mobile_key(
        types,
        "npcSeq",
        |v| {
            v.as_array()
                .ok_or("expected an array of blocks")?
                .iter()
                .try_for_each(check_block)
        },
        |w, id, v| {
            for b in v.as_array().map(Vec::as_slice).unwrap_or_default() {
                run_block(w, id, b);
            }
        },
    );
    mobile_key(
        types,
        "npcIfLayerEmpty",
        |v| {
            let o = v.as_object().ok_or("expected {\"layer\", \"then\"}")?;
            let l = o.get("layer").and_then(Value::as_str).unwrap_or_default();
            layer(l).ok_or_else(|| format!("unknown layer {l}"))?;
            check_block(o.get("then").ok_or("\"then\" is missing")?)
        },
        |w, id, v| {
            let l = v.get("layer").and_then(Value::as_str).and_then(layer);
            if let Some(l) = l
                && muo_server::mobile::part2::items::find_item_on_layer(w, id, l).is_none()
            {
                run_block(w, id, &v["then"]);
            }
        },
    );
    mobile_key(
        types,
        "npcHueVars",
        |v| {
            v.as_object()
                .ok_or("expected {\"name\": hue, ..}")?
                .values()
                .try_for_each(check_hue)
        },
        |w, id, v| {
            for (n, h) in v.as_object().into_iter().flatten() {
                let h = hue_of(w, id, h);
                w.resources.get_mut::<HueVars>().0.insert(n.clone(), h);
            }
        },
    );
    mobile_key(
        types,
        "npcByGender",
        |v| {
            let o = v.as_object().ok_or("expected {\"female\", \"male\"}")?;
            if let Some(k) = o.keys().find(|k| !["female", "male"].contains(&k.as_str())) {
                return Err(format!("unknown key \"{k}\""));
            }
            o.values().try_for_each(check_block)
        },
        |w, id, v| {
            let female = w.mobile(id).is_some_and(|m| m.female);
            if let Some(b) = v.get(if female { "female" } else { "male" }) {
                run_block(w, id, b);
            }
        },
    );
    mobile_key(
        types,
        "npcEquipOrDelete",
        |v| {
            let o = v.as_object().ok_or("expected {\"item\", \"else\"}")?;
            if let Some(k) = o.keys().find(|k| !["item", "else"].contains(&k.as_str())) {
                return Err(format!("unknown key \"{k}\""));
            }
            check_items(&Value::Array(vec![
                o.get("item").ok_or("\"item\" is missing")?.clone(),
            ]))?;
            o.get("else").map_or(Ok(()), check_block)
        },
        |w, id, v| {
            let Some(it) = item(w, id, &v["item"], equip_or_delete) else {
                return;
            };
            if w.item(it).is_none_or(|i| i.deleted())
                && let Some(b) = v.get("else")
            {
                run_block(w, id, b);
            }
        },
    );
    mobile_key(
        types,
        "npcRehueLayers",
        |v| {
            let o = v.as_object().ok_or("expected {\"layers\", \"hue\"}")?;
            for l in o
                .get("layers")
                .and_then(Value::as_array)
                .ok_or("\"layers\": expected an array")?
            {
                let l = l.as_str().unwrap_or_default();
                layer(l).ok_or_else(|| format!("unknown layer {l}"))?;
            }
            check_hue(o.get("hue").ok_or("\"hue\" is missing")?)
        },
        |w, id, v| {
            let layers = v["layers"]
                .as_array()
                .map(Vec::as_slice)
                .unwrap_or_default();
            for l in layers.iter().filter_map(|l| l.as_str().and_then(layer)) {
                if let Some(it) = muo_server::mobile::part2::items::find_item_on_layer(w, id, l) {
                    let h = hue_of(w, id, &v["hue"]);
                    muo_server::item::base::set_hue(w, it, h);
                }
            }
        },
    );
    mobile_key(types, "npcFacialHair", check_hue, |w, id, v| {
        let h = hue_of(w, id, v);
        muo_content::vendor::assign_random_facial_hair(w, id, h);
    });
    mobile_key(
        types,
        "npcHairItemId",
        muo_content::templates::keys::check_int,
        |w, id, v| {
            let n = muo_content::templates::keys::int(v);
            muo_server::mobile::part1::props_b::set_hair_item_id(w, id, n);
        },
    );
    mobile_key(types, "npcHairHue", check_hue, |w, id, v| {
        let h = hue_of(w, id, v);
        muo_server::mobile::part1::props_b::set_hair_hue(w, id, h);
    });
    mobile_key(
        types,
        "npcReplaceBackpack",
        |v| {
            v.get("weight")
                .and_then(Value::as_f64)
                .map(|_| ())
                .ok_or_else(|| format!("expected {{\"weight\": n}}, got {v}"))
        },
        |w, id, v| {
            // Backpack?.Delete(); AddItem(new Backpack { Movable = false, Weight = n });
            if let Some(old) = w.mobile(id).and_then(|m| m.backpack)
                && let Some(t) = w.item(old).map(|i| i.ty)
            {
                (t.delete)(w, old);
            }
            if let Some(pack) = muo_content::factory::make_item(w, "Server.Items.Backpack") {
                muo_server::item::ops::set_movable(w, pack, false);
                muo_server::item::ops::set_weight(w, pack, v["weight"].as_f64().unwrap_or(0.0));
                add_item(w, id, pack);
            }
        },
    );
    mobile_key(types, "npcDeleteBackpack", check_bool, |w, id, _| {
        if let Some(p) = muo_server::mobile::part1::props_b::backpack(w, id)
            && let Some(t) = w.item(p).map(|i| i.ty)
        {
            (t.delete)(w, p);
        }
    });
    mobile_key(types, "npcEscortOutfit", check_bool, |w, id, _| {
        crate::npcs::townfolk::init_outfit(w, id);
    });
    mobile_key(types, "npcPrisoner", check_bool, |w, id, v| {
        if let Some(d) = muo_content::creature::data::cd_mut(w, id) {
            d.is_prisoner = v.as_bool() == Some(true);
        }
    });
    mobile_key(
        types,
        "npcGuardWeapon",
        |v| check_items(&Value::Array(vec![v.clone()])),
        |w, id, v| {
            item(w, id, v, add_guard_weapon);
        },
    );
    mobile_key(types, "npcUnmovablePack", check_pack, |w, id, v| {
        let mut contents = Vec::new();
        for c in v.as_array().map(Vec::as_slice).unwrap_or_default() {
            contents.extend(pack_content(w, c));
        }
        let Some(pack) = muo_content::factory::make_item(w, "Server.Items.Backpack") else {
            return;
        };
        muo_server::item::ops::set_movable(w, pack, false);
        for c in contents {
            muo_server::item::container::drop_item_default(w, pack, c);
        }
        add_item(w, id, pack);
    });
    mobile_key(
        types,
        "npcMount",
        |v| match v.as_str() {
            Some(_) => Ok(()),
            None => Err(format!("expected a mount class, got {v}")),
        },
        |w, id, v| {
            let m = v
                .as_str()
                .and_then(|n| muo_content::factory::make_mobile(w, n));
            if let Some(m) = m {
                muo_content::creature::mount::set_rider(w, m, Some(id));
            }
        },
    );
    mobile_key(
        types,
        "npcCombatDelay",
        muo_content::templates::keys::check_int,
        |w, id, v| {
            let t = w.tick_count + i64::from(muo_content::templates::keys::int(v));
            if let Some(m) = w.mobile_mut(id) {
                m.next_combat_time = t;
            }
        },
    );
}

/// `weapon.Movable = false; weapon.Crafter = Name; weapon.Quality = Exceptional; AddItem(weapon)`.
fn add_guard_weapon(w: &mut World, id: MobileId, wp: ItemId) {
    use muo_content::items::weapon::{self, WeaponQuality};
    muo_server::item::ops::set_movable(w, wp, false);
    let name = w.mobile(id).and_then(|m| (m.ty.name)(w, id));
    if let Some(d) = weapon::data_mut(w, wp) {
        d.crafter = name;
    }
    weapon::set_quality(w, wp, WeaponQuality::Exceptional);
    add_item(w, id, wp);
}

fn check_pack(v: &Value) -> Result<(), String> {
    for c in v.as_array().ok_or("expected an array of pack contents")? {
        let o = c.as_object().ok_or("expected a pack content object")?;
        if let Some(g) = o.get("gold") {
            check_ints::<2>(g)?;
        } else {
            o.get("item")
                .and_then(Value::as_str)
                .ok_or("\"item\": expected a class name")?;
            if let Some(a) = o.get("amount") {
                muo_content::templates::keys::check_int(a)?;
            }
        }
        if let Some(k) = o
            .keys()
            .find(|k| !["gold", "item", "amount", "newbied"].contains(&k.as_str()))
        {
            return Err(format!("unknown pack content key \"{k}\""));
        }
    }
    Ok(())
}

/// `new Gold(Utility.RandomMinMax(min, max))`, or `new T { Amount = n, LootType = Newbied }`.
fn pack_content(w: &mut World, c: &Value) -> Option<ItemId> {
    use muo_server::item::ops;
    if let Some([a, b]) = c.get("gold").and_then(ints::<2>) {
        let amount = rnd::min_max(w, a, b);
        let g = muo_content::factory::make_item(w, "Server.Items.Gold")?;
        ops::set_amount(w, g, amount);
        return Some(g);
    }
    let it = muo_content::factory::make_item(w, c.get("item")?.as_str()?)?;
    if let Some(a) = c.get("amount") {
        ops::set_amount(w, it, muo_content::templates::keys::int(a));
    }
    if c.get("newbied").and_then(Value::as_bool) == Some(true) {
        ops::set_loot_type(w, it, muo_server::item::LootType::Newbied);
    }
    Some(it)
}

/// `InitOutfit()` of a class whose outfit is its `"npcOutfit"` prop (none: nothing).
pub fn outfit_prop(w: &mut World, id: MobileId) {
    if let Some(v) = props::mobile(w, id, "npcOutfit").cloned() {
        run_block(w, id, &v);
    }
}

// ---- class members read from props ----

/// `"npcClickTitle": bool` (`ClickTitle`), else `BaseCreature`'s.
pub fn click_title(w: &World, id: MobileId) -> bool {
    props::as_bool(props::mobile(w, id, "npcClickTitle"))
        .unwrap_or_else(|| (muo_content::creature::BASE_CREATURE.click_title)(w, id))
}

/// `"npcCanTeach": bool` (`CanTeach`), else `BaseCreature`'s.
pub fn can_teach(w: &World, id: MobileId) -> bool {
    props::as_bool(props::mobile(w, id, "npcCanTeach"))
        .unwrap_or_else(|| (muo_content::creature::BASE_CREATURE_C.can_teach)(w, id))
}

/// `"npcGuild": "MagesGuild"`.
pub fn npc_guild(w: &World, id: MobileId) -> muo_content::player_mobile::NpcGuild {
    use muo_content::player_mobile::NpcGuild::*;
    let s = props::mobile(w, id, "npcGuild").and_then(Value::as_str);
    [
        MagesGuild,
        WarriorsGuild,
        ThievesGuild,
        RangersGuild,
        HealersGuild,
        MinersGuild,
        MerchantsGuild,
        TinkersGuild,
        TailorsGuild,
        FishermensGuild,
        BardsGuild,
        BlacksmithsGuild,
    ]
    .into_iter()
    .find(|g| s.is_some_and(|s| format!("{g:?}") == s))
    .unwrap_or(None)
}

fn shoe(s: &str) -> Option<muo_content::vendor::ShoeType> {
    use muo_content::vendor::ShoeType::*;
    [None, Shoes, Boots, Sandals, ThighBoots]
        .into_iter()
        .find(|t| format!("{t:?}") == s)
}

/// `"npcShoeType"`: `"Boots"`, `["Shoes", "Sandals"]` (`Utility.RandomBool() ? first : second`), or
/// `{"female": "ThighBoots", "male": "Boots"}`.
pub fn shoe_type(w: &mut World, id: MobileId) -> muo_content::vendor::ShoeType {
    let v = props::mobile(w, id, "npcShoeType").cloned();
    let pick = match &v {
        Some(Value::String(s)) => Some(s.as_str()),
        Some(Value::Array(a)) if a.len() == 2 => {
            let i = if rnd::bool(w) { 0 } else { 1 };
            a[i].as_str()
        }
        Some(Value::Object(o)) => {
            let female = w.mobile(id).is_some_and(|m| m.female);
            o.get(if female { "female" } else { "male" })
                .and_then(Value::as_str)
        }
        _ => Option::None,
    };
    pick.and_then(shoe)
        .unwrap_or(muo_content::vendor::ShoeType::Shoes)
}

/// `"npcShoeHue": n`, else `BaseVendor.GetShoeHue()`.
pub fn shoe_hue(w: &mut World, id: MobileId) -> i32 {
    match props::as_i32(props::mobile(w, id, "npcShoeHue")) {
        Some(h) => h,
        None => muo_content::vendor::get_shoe_hue(w, id),
    }
}

/// `"npcOutfit": {template keys}` replaces `InitOutfit()`; without it, `BaseVendor.InitOutfit()`.
pub fn init_outfit(w: &mut World, id: MobileId) {
    match props::mobile(w, id, "npcOutfit").cloned() {
        Some(v) => run_block(w, id, &v),
        None => muo_content::vendor::init_outfit(w, id),
    }
}
