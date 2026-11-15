//! Template keys of the NPCs (`muo_content::templates::keys`), and the vendor class members read from `"props"`.
//!
//! A hue value (`npcHue`, `npcSpeechHue`, an item's `"hue"`) is a number or a random hue: `"randomDyed"`,
//! `"randomNeutral"`, `"randomBlue"`, `"randomGreen"`, `"randomRed"`, `"randomYellow"`, `"randomPink"`,
//! `"randomBright"`, `"randomNondyed"`, `"humanSkin"` (`Race.Human.RandomSkinHue()`), `"humanHair"`.
//!
//! An item value (`npcEquip`, `npcAddItem`, `npcReplaceEquipment` take an array of them) is a class name or
//! `{"item": "<class>", "hue": <hue>, "hueAfter": <hue>}`: `hue` is drawn before the item is made (`new T(hue)`),
//! `hueAfter` after (`new T { Hue = .. }`).
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
//!
//! Nested blocks are parsed when they run, so a mistake in one is logged then (`templates_npcs` tests every
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

/// A hue: fixed, or drawn when applied.
#[derive(Copy, Clone, Debug)]
pub enum Hue {
    Fixed(i32),
    Random(fn(&mut World) -> i32),
}

const RANDOM_HUES: [(&str, fn(&mut World) -> i32); 11] = [
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
        let s = v
            .as_str()
            .ok_or_else(|| format!("expected a hue, got {v}"))?;
        RANDOM_HUES
            .iter()
            .find(|(n, _)| *n == s)
            .map(|(_, f)| Hue::Random(*f))
            .ok_or_else(|| format!("unknown hue {s}"))
    }

    pub fn roll(self, w: &mut World) -> i32 {
        match self {
            Hue::Fixed(n) => n,
            Hue::Random(f) => f(w),
        }
    }
}

fn hue_of(v: &Value) -> Hue {
    Hue::parse(v).unwrap_or(Hue::Fixed(0))
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
            if let Some(k) = o.keys().find(|k| !["item", "hue", "hueAfter"].contains(&k.as_str())) {
                return Err(format!("unknown item key \"{k}\""));
            }
            for k in ["hue", "hueAfter"] {
                if let Some(h) = o.get(k) {
                    check_hue(h)?;
                }
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
        let Some((name, hue, after)) = item_of(x) else {
            continue;
        };
        let hue = hue.map(|h| h.roll(w));
        let Some(it) = muo_content::factory::make_item(w, name) else {
            continue;
        };
        if let Some(h) = hue {
            muo_server::item::base::set_hue(w, it, h);
        }
        put(w, id, it);
        if let Some(h) = after {
            let h = h.roll(w);
            if w.item(it).is_some() {
                muo_server::item::base::set_hue(w, it, h);
            }
        }
    }
}

/// `EquipItem(item)`: a refused item stays unreferenced.
fn equip(w: &mut World, id: MobileId, it: ItemId) {
    muo_server::mobile::base::equip_item(w, id, it);
}

fn add_item(w: &mut World, id: MobileId, it: ItemId) {
    muo_server::mobile::part2::items::add_item(w, id, it);
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
        let h = hue_of(v).roll(w);
        muo_content::creature::init_ops::speech_hue(w, id, h);
    });
    mobile_key(types, "npcHue", check_hue, |w, id, v| {
        let h = hue_of(v).roll(w);
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
            let o = v.as_object().ok_or("expected {\"hue\", \"facialSkipOneIn\"}")?;
            check_hue(o.get("hue").ok_or("\"hue\" is missing")?)?;
            match o.get("facialSkipOneIn") {
                Some(n) if n.as_i64().is_none_or(|n| n < 1) => {
                    Err("\"facialSkipOneIn\": expected a count".into())
                }
                _ => Ok(()),
            }
        },
        |w, id, v| {
            let h = hue_of(&v["hue"]).roll(w);
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
            (Some(Value::String(_)), Some(Value::String(_))) if v.as_object().is_some_and(|o| o.len() == 2) => Ok(()),
            _ => Err(format!("expected {{\"female\": title, \"male\": title}}, got {v}")),
        },
        |w, id, v| {
            let female = w.mobile(id).is_some_and(|m| m.female);
            let t = v[if female { "female" } else { "male" }].as_str().map(str::to_owned);
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
