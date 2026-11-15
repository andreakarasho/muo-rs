//! Template keys of creatures, animals and mounts (`muo_content::templates::keys`).
//!
//! A number value is a fixed number; a random one is an object: `{"from": a, "count": n}` (`Utility.Random(a, n)`),
//! `{"min": a, "max": b}` (`Utility.RandomMinMax`), `{"raceSkinHue": "Human"}` / `{"raceHairHue": "Human"}`
//! (`Race.Human.RandomSkinHue()`), `{"nondyedHue": true}`, each optionally with `"or"`, `"and"` (bit masks) and
//! `"neg": true` applied to the drawn value.

use serde_json::{Map, Value};

use muo_content::creature::enums::{AIType, FightMode};
use muo_content::creature::{data::cd_mut, init_ops as ops, mount, packing, setters};
use muo_content::factory;
use muo_content::hooks::rnd;
use muo_content::templates::keys::Keys;
use muo_content::templates::keys::{check_bool, check_int, mobile_key};
use muo_content::templates::mobile_steps::{self, skill};
use muo_primitives::TimeSpan;
use muo_server::ids::{ItemId, MobileId};
use muo_server::item::{base as ibase, ops as iops};
use muo_server::mobile::part2::items;
use muo_server::type_registry::TypeRegistry;
use muo_server::world::World;

pub(crate) fn register(types: &mut TypeRegistry) {
    let ival: fn(&Value) -> Result<(), String> = |v| check_ival(v);
    mobile_key(types, "creatureHue", ival, |w, id, v| {
        if let Some(h) = roll_opt(w, v) {
            ops::hue(w, id, h);
        }
    });
    mobile_key(types, "creatureSpeechHue", ival, |w, id, v| {
        let h = roll(w, v);
        ops::speech_hue(w, id, h);
    });
    mobile_key(types, "creatureBody", ival, |w, id, v| {
        let b = roll(w, v);
        ops::body(w, id, b);
    });
    mobile_key(types, "creatureVirtualArmor", ival, |w, id, v| {
        let n = roll(w, v);
        ops::virtual_armor(w, id, n);
    });
    mobile_key(types, "creatureFame", ival, |w, id, v| {
        let n = roll(w, v);
        ops::fame(w, id, n);
    });
    mobile_key(types, "creatureKarma", ival, |w, id, v| {
        let n = roll(w, v);
        ops::karma(w, id, n);
    });
    mobile_key(types, "creatureGold", ival, |w, id, v| {
        let n = roll(w, v);
        setters::pack_gold(w, id, n);
    });
    mobile_key(types, "creatureNecroScroll", ival, |w, id, v| {
        let n = roll(w, v);
        packing::pack_necro_scroll(w, id, n);
    });
    mobile_key(
        types,
        "creatureInitStats",
        |v| match v.as_array() {
            Some(a) if a.len() == 3 => a.iter().try_for_each(check_ival),
            _ => Err(format!("expected [str, dex, int], got {v}")),
        },
        |w, id, v| {
            let a = v.as_array().map(Vec::as_slice).unwrap_or_default();
            let [s, d, i] = a else { return };
            let (s, d, i) = (roll(w, s), roll(w, d), roll(w, i));
            ops::init_stats(w, id, s, d, i);
        },
    );
    mobile_key(
        types,
        "creatureSkillBase",
        |v| {
            obj(v)?.iter().try_for_each(|(k, f)| {
                skill(k)?;
                check_fval(f)
            })
        },
        |w, id, v| {
            for (k, f) in v.as_object().into_iter().flatten() {
                if let Ok(s) = skill(k) {
                    let x = roll_f(w, f);
                    ops::skill_base(w, id, s, x);
                }
            }
        },
    );
    mobile_key(types, "creatureItem", check_items, |w, id, v| {
        let mut st = ListState::default();
        match v {
            Value::Array(a) => a
                .iter()
                .for_each(|e| item(w, id, e, Place::Mobile, &mut st)),
            e => item(w, id, e, Place::Mobile, &mut st),
        }
    });
    mobile_key(
        types,
        "creatureAi",
        |v| name_of(v, AIType::from_i32).map(drop),
        |w, id, v| {
            if let Ok(ai) = name_of(v, AIType::from_i32) {
                crate::hand::set_ai(w, id, ai);
            }
        },
    );
    mobile_key(
        types,
        "creatureFightMode",
        |v| name_of(v, FightMode::from_i32).map(drop),
        |w, id, v| {
            if let Ok(m) = name_of(v, FightMode::from_i32) {
                crate::hand::set_fight_mode(w, id, m);
            }
        },
    );
    mobile_key(types, "creatureSpeed", check_pair_f, |w, id, v| {
        let (a, p) = pair_f(v);
        muo_content::creature::speeds::set_speed(w, id, a, p, true);
    });
    mobile_key(types, "creatureMountItemId", check_int, |w, id, v| {
        mount::set_item_id(w, id, muo_content::templates::keys::int(v));
    });
    mobile_key(
        types,
        "creatureRider",
        |v| match v {
            Value::String(_) => Ok(()),
            Value::Object(o) if o.get("class").is_some_and(Value::is_string) => {
                match o.get("barding").and_then(Value::as_str) {
                    Some(b) if resource(b).is_none() => Err(format!("unknown resource {b}")),
                    _ => Ok(()),
                }
            }
            _ => Err(format!(
                "expected a class name or {{\"class\", \"barding\"}}, got {v}"
            )),
        },
        |w, id, v| {
            let class = v
                .as_str()
                .or_else(|| v.get("class").and_then(Value::as_str));
            let Some(m) = class.and_then(|c| factory::make_mobile(w, c)) else {
                return;
            };
            if let Some(r) = v.get("barding").and_then(Value::as_str).and_then(resource) {
                crate::classes::swamp_dragon::equip_barding(w, m, r);
            }
            mount::set_rider(w, m, Some(id));
        },
    );
    mobile_key(types, "creatureRace", check_str, |w, id, v| {
        let race = v.as_str().and_then(|r| crate::hand::race_named(w, r));
        muo_server::mobile::part1::props_a::set_race(w, id, race);
    });
    mobile_key(types, "creatureNextAbility", ival_pair, |w, id, v| {
        let a = v.as_array().map(Vec::as_slice).unwrap_or_default();
        let [lo, hi] = a else { return };
        let secs = rnd::min_max(w, int_of(lo), int_of(hi));
        let at = w.now + TimeSpan::from_secs(i64::from(secs));
        crate::hand::set_next_ability(w, id, at);
    });
    mobile_key(
        types,
        "creaturePackArmor",
        |v| match v.as_array().map(Vec::as_slice) {
            Some([a, b, c]) if a.is_i64() && b.is_i64() && c.is_number() => Ok(()),
            _ => Err(format!("expected [minLevel, maxLevel, chance], got {v}")),
        },
        |w, id, v| {
            let a = v.as_array().map(Vec::as_slice).unwrap_or_default();
            if let [lo, hi, c] = a {
                packing::pack_armor(w, id, int_of(lo), int_of(hi), c.as_f64().unwrap_or(0.0));
            }
        },
    );
    // `if (Utility.Random(n) == 0) { then } else { else }`: the blocks hold any template keys.
    mobile_key(
        types,
        "creatureOneIn",
        |v| {
            let o = obj(v)?;
            if let Some(k) = o
                .keys()
                .find(|k| !["n", "then", "else"].contains(&k.as_str()))
            {
                return Err(format!("unknown key \"{k}\" (expected n, then, else)"));
            }
            check_int(o.get("n").unwrap_or(&Value::Null))?;
            for b in ["then", "else"] {
                for (k, x) in o.get(b).map(obj).transpose()?.into_iter().flatten() {
                    // keys of other crates are checked when the block runs
                    match mobile_steps::parse(k, x, &Keys::default()) {
                        Err(e) if e != "unknown key" => {
                            return Err(format!("\"{b}\": \"{k}\": {e}"));
                        }
                        _ => {}
                    }
                }
            }
            Ok(())
        },
        |w, id, v| {
            let n = v.get("n").map_or(1, int_of);
            let block = if rnd::below(w, n) == 0 {
                "then"
            } else {
                "else"
            };
            for (k, x) in v
                .get(block)
                .and_then(Value::as_object)
                .into_iter()
                .flatten()
            {
                match mobile_steps::parse(k, x, &Keys::of(&w.types)) {
                    Ok(s) => mobile_steps::apply(w, id, &s),
                    Err(e) => tracing::warn!("Templates: creatureOneIn: \"{k}\": {e}"),
                }
            }
        },
    );
    mobile_key(
        types,
        "creatureSetup",
        |v| match v.as_str() {
            Some(s) if SETUPS.iter().any(|(n, _)| *n == s) => Ok(()),
            _ => Err(format!(
                "expected one of {}, got {v}",
                SETUPS
                    .iter()
                    .map(|(n, _)| *n)
                    .collect::<Vec<_>>()
                    .join(", ")
            )),
        },
        |w, id, v| {
            if let Some((_, f)) = SETUPS.iter().find(|(n, _)| Some(*n) == v.as_str()) {
                f(w, id);
            }
        },
    );
}

/// `creatureSetup`: the per-creature state a class constructor initializes (behavior data, timers).
type Setup = fn(&mut World, MobileId);
const SETUPS: &[(&str, Setup)] = &[
    ("Cow", crate::hand::cow_init),
    ("Sheep", crate::hand::sheep_init),
    ("Decanter", crate::hand::water_elemental_decanter),
    ("CanTalk", crate::hand::ferret_can_talk),
    ("BurstSac", crate::hand::burst_sac_init),
    ("Changeling", crate::hand::changeling_data),
    ("DefaultHue", |w, id| {
        let h = muo_content::creature::ctype(w, id).map_or(0, |c| (c.default_hue)(w, id));
        ops::hue(w, id, h);
    }),
    ("ShadowFiendTimer", crate::hand::shadow_fiend_start_timer),
    ("VorpalBunnyTunnel", crate::hand::vorpal_bunny_tunnel),
    ("NoKillAwards", |w, id| {
        if let Some(d) = cd_mut(w, id) {
            d.no_kill_awards = true;
        }
    }),
    ("DeleteBackpack", |w, id| {
        if let Some(p) = items::find_item_on_layer(w, id, muo_server::layer::Layer::Backpack) {
            ibase::delete(w, p);
        }
    }),
    ("StrongBackpack", crate::hand::pack_animal::replace_backpack),
    (
        "SwampDragon",
        crate::classes::swamp_dragon::swamp_dragon_data,
    ),
    ("HumanLook", crate::hand::human_look),
    ("HumanHair", crate::hand::protector_hair),
    ("HumanHairHue", crate::hand::protector_hair_hue),
    ("PlagueBeast", crate::hand::plague_beast_setup),
    (
        "NoFollowers",
        crate::classes::ilhenir::meraktus_no_followers,
    ),
];

fn obj(v: &Value) -> Result<&Map<String, Value>, String> {
    v.as_object()
        .ok_or_else(|| format!("expected an object, got {v}"))
}

fn check_str(v: &Value) -> Result<(), String> {
    v.as_str()
        .map(drop)
        .ok_or_else(|| format!("expected a string, got {v}"))
}

fn int_of(v: &Value) -> i32 {
    v.as_i64().and_then(|n| i32::try_from(n).ok()).unwrap_or(0)
}

fn ival_pair(v: &Value) -> Result<(), String> {
    match v.as_array().map(Vec::as_slice) {
        Some([a, b]) => check_int(a).and(check_int(b)),
        _ => Err(format!("expected [min, max], got {v}")),
    }
}

fn check_pair_f(v: &Value) -> Result<(), String> {
    match v.as_array().map(Vec::as_slice) {
        Some([a, b]) if a.is_number() && b.is_number() => Ok(()),
        _ => Err(format!("expected [a, b], got {v}")),
    }
}

fn pair_f(v: &Value) -> (f64, f64) {
    let a = v.as_array().map(Vec::as_slice).unwrap_or_default();
    match a {
        [x, y] => (x.as_f64().unwrap_or(0.0), y.as_f64().unwrap_or(0.0)),
        _ => (0.0, 0.0),
    }
}

fn name_of<T: std::fmt::Debug>(v: &Value, from: fn(i32) -> Option<T>) -> Result<T, String> {
    let s = v
        .as_str()
        .ok_or_else(|| format!("expected a name, got {v}"))?;
    (0..256)
        .filter_map(from)
        .find(|x| format!("{x:?}") == s)
        .ok_or_else(|| format!("unknown value {s}"))
}

const IVAL_KEYS: &[&str] = &[
    "from",
    "count",
    "min",
    "max",
    "raceSkinHue",
    "raceHairHue",
    "nondyedHue",
    "below",
    "table",
    "pick",
    "double",
    "cases",
    "or",
    "and",
    "neg",
];

fn check_ival(v: &Value) -> Result<(), String> {
    if v.is_i64() {
        return check_int(v);
    }
    let o = obj(v)?;
    if let Some(k) = o.keys().find(|k| !IVAL_KEYS.contains(&k.as_str())) {
        return Err(format!("unknown key \"{k}\" in a random number"));
    }
    let has = |k: &str| o.contains_key(k);
    let kinds = [
        has("from") && has("count"),
        has("min") && has("max"),
        has("raceSkinHue"),
        has("raceHairHue"),
        has("nondyedHue"),
        has("below") && has("table"),
        has("pick"),
        has("double") && has("cases"),
    ];
    if kinds.iter().filter(|b| **b).count() != 1 {
        return Err(format!(
            "expected one of from/count, min/max, raceSkinHue, raceHairHue, nondyedHue, got {v}"
        ));
    }
    for e in o.get("table").map_or(Ok(&Vec::new()), |x| {
        x.as_array()
            .ok_or_else(|| format!("expected [[at least, value], ..], got {x}"))
    })? {
        match e.as_array().map(Vec::as_slice) {
            Some([a, v]) if a.is_i64() => check_table_value(v)?,
            _ => return Err(format!("expected [at least, value], got {e}")),
        }
    }
    if let Some(p) = o.get("pick") {
        match p.as_array() {
            Some(a) if !a.is_empty() => a.iter().try_for_each(check_int)?,
            _ => return Err(format!("\"pick\": expected a list of numbers, got {p}")),
        }
    }
    if let Some(d) = o.get("double") {
        if !d.is_number() {
            return Err(format!("\"double\": expected a number, got {d}"));
        }
        for c in o
            .get("cases")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            let ok = c.get("below").is_some_and(Value::is_number)
                && c.as_object().is_some_and(|c| {
                    c.keys()
                        .all(|k| ["below", "orEqual", "value"].contains(&k.as_str()))
                });
            if !ok {
                return Err(format!("expected {{\"below\", \"value\"}}, got {c}"));
            }
            check_ival(c.get("value").unwrap_or(&Value::Null))?;
        }
    }
    for k in ["from", "count", "min", "max", "below", "or", "and"] {
        if let Some(x) = o.get(k) {
            check_int(x)?;
        }
    }
    Ok(())
}

/// The value of a (checked) number: fixed, or drawn now.
fn roll(w: &mut World, v: &Value) -> i32 {
    roll_opt(w, v).unwrap_or(0)
}

/// [`roll`]; `None` when a `double` draw falls in none of its cases (the statement sets nothing).
fn roll_opt(w: &mut World, v: &Value) -> Option<i32> {
    let Some(o) = v.as_object() else {
        return Some(int_of(v));
    };
    let g = |k: &str| o.get(k).map(int_of);
    let race = |k: &str| o.get(k).and_then(Value::as_str);
    let mut n = if let (Some(a), Some(c)) = (g("from"), g("count")) {
        rnd::from(w, a, c)
    } else if let (Some(a), Some(b)) = (g("min"), g("max")) {
        rnd::min_max(w, a, b)
    } else if let Some(r) = race("raceSkinHue") {
        ops::race_skin_hue(w, r)
    } else if let Some(r) = race("raceHairHue") {
        ops::race_hair_hue(w, r)
    } else if o.contains_key("nondyedHue") {
        muo_primitives::utility::random_nondyed_hue(&mut w.rng)
    } else if let Some(p) = o.get("pick").and_then(Value::as_array) {
        let i = rnd::below(w, p.len() as i32) as usize;
        p.get(i).map_or(0, int_of)
    } else if let Some(scale) = o.get("double").and_then(Value::as_f64) {
        // `var c = Utility.RandomDouble() * scale; if (c <= a) .. else if (c < b) ..`
        let c = rnd::double(w) * scale;
        let cases = o.get("cases").and_then(Value::as_array);
        let hit = cases.into_iter().flatten().find(|e| {
            let b = e.get("below").and_then(Value::as_f64).unwrap_or(0.0);
            c < b || (c == b && e.get("orEqual").and_then(Value::as_bool) == Some(true))
        })?;
        roll_opt(w, hit.get("value")?)?
    } else if let Some(n) = g("below") {
        // `switch (Utility.Random(n))` over ranges: the first entry whose bound the draw reaches
        let r = rnd::below(w, n);
        let table = o.get("table").and_then(Value::as_array);
        let hit = table
            .into_iter()
            .flatten()
            .filter_map(|e| e.as_array())
            .find(|e| e.first().is_some_and(|a| r >= int_of(a)));
        match hit.and_then(|e| e.get(1)) {
            Some(Value::Object(c)) => c
                .get("craftHue")
                .and_then(Value::as_str)
                .and_then(resource)
                .map_or(0, |r| muo_content::craft::hue(w, r)),
            Some(v) => int_of(v),
            None => 0,
        }
    } else {
        0
    };
    if o.get("neg").and_then(Value::as_bool) == Some(true) {
        n = -n;
    }
    if let Some(m) = g("and") {
        n &= m;
    }
    if let Some(m) = g("or") {
        n |= m;
    }
    Some(n)
}

fn check_table_value(v: &Value) -> Result<(), String> {
    match v.get("craftHue").and_then(Value::as_str) {
        Some(r) if resource(r).is_some() => Ok(()),
        Some(r) => Err(format!("unknown resource {r}")),
        None => check_int(v),
    }
}

fn resource(name: &str) -> Option<muo_content::craft::CraftResource> {
    (0..1024)
        .filter_map(muo_content::craft::CraftResource::from_i32)
        .find(|r| format!("{r:?}") == name)
}

/// A float: a number, or `{"base": b, "random": r}` (`b + Utility.RandomDouble() * r`), or an int draw.
fn check_fval(v: &Value) -> Result<(), String> {
    if v.is_number() {
        return Ok(());
    }
    match v.as_object() {
        Some(o) if o.len() == 2 && o.get("base").is_some_and(Value::is_number) => o
            .get("random")
            .filter(|r| r.is_number())
            .map(drop)
            .ok_or_else(|| format!("expected {{\"base\", \"random\"}}, got {v}")),
        _ => check_ival(v),
    }
}

fn roll_f(w: &mut World, v: &Value) -> f64 {
    if let Some(f) = v.as_f64() {
        return f;
    }
    let o = v.as_object();
    match (
        o.and_then(|o| o.get("base")).and_then(Value::as_f64),
        o.and_then(|o| o.get("random")).and_then(Value::as_f64),
    ) {
        (Some(b), Some(r)) => b + rnd::double(w) * r,
        _ => f64::from(roll(w, v)),
    }
}

// ---- creatureItem ----

const ITEM_KEYS: &[&str] = &[
    "item",
    "pick",
    "loot",
    "group",
    "plain",
    "held",
    "times",
    "args",
    "amount",
    "setAmount",
    "hue",
    "itemId",
    "layer",
    "movable",
    "lootType",
    "arcaneCharges",
    "quality",
    "crafterSelf",
    "contents",
    "resource",
    "keep",
    "to",
];

const LOOT: &[&str] = &[
    "Weapon",
    "ArmorOrShieldOrWeapon",
    "NecromancyReagent",
    "PossibleReagent",
    "Reagent",
    "Gem",
    "BonsaiSeed",
    "PeculiarSeed",
    "ArcanistScroll",
];

fn check_items(v: &Value) -> Result<(), String> {
    match v {
        Value::Array(a) => a.iter().try_for_each(|e| check_item(e, false)),
        _ => check_item(v, false),
    }
}

fn check_item(v: &Value, inside: bool) -> Result<(), String> {
    use muo_content::templates::keys::check_name;
    let o = obj(v)?;
    if let Some(k) = o.keys().find(|k| !ITEM_KEYS.contains(&k.as_str())) {
        return Err(format!("unknown item key \"{k}\""));
    }
    let what = ["item", "pick", "loot", "plain", "held"]
        .iter()
        .filter(|k| o.contains_key(**k))
        .count();
    if what != 1 {
        return Err(
            "an item needs one of \"item\", \"pick\", \"loot\", \"plain\" or \"held\"".into(),
        );
    }
    for k in ["item", "held", "keep"] {
        if let Some(x) = o.get(k) {
            check_str(x)?;
        }
    }
    if let Some(p) = o.get("pick") {
        let ok = p
            .as_array()
            .is_some_and(|a| !a.is_empty() && a.iter().all(|x| x.is_string() || x.is_null()));
        if !ok {
            return Err(format!(
                "\"pick\": expected class names (or null for none), got {p}"
            ));
        }
    }
    if let Some(l) = o.get("loot") {
        check_name(l, LOOT)?;
    }
    for k in ["times", "amount", "setAmount", "hue"] {
        match o.get(k) {
            Some(Value::String(s)) if k == "hue" && s == "same" => {}
            Some(x) => check_ival(x)?,
            None => {}
        }
    }
    for x in o
        .get("args")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        let map = x.get("map").and_then(Value::as_str);
        if !(map == Some("Trammel")
            || map == Some("Felucca")
            || x.get("enum").is_some_and(Value::is_i64))
        {
            check_ival(x)?;
        }
    }
    for k in ["itemId", "plain", "group", "arcaneCharges"] {
        if let Some(x) = o.get(k) {
            check_int(x)?;
        }
    }
    for k in ["movable", "crafterSelf"] {
        if let Some(x) = o.get(k) {
            check_bool(x)?;
        }
    }
    if let Some(l) = o.get("layer") {
        layer(l).ok_or_else(|| format!("unknown layer {l}"))?;
    }
    if let Some(r) = o.get("resource").and_then(Value::as_str) {
        resource(r).ok_or_else(|| format!("unknown resource {r}"))?;
    }
    if let Some(l) = o.get("lootType") {
        check_name(l, &["Regular", "Newbied", "Blessed", "Cursed"])?;
    }
    if let Some(q) = o.get("quality") {
        check_name(q, &["Low", "Regular", "Exceptional"])?;
    }
    if let Some(t) = o.get("to") {
        if inside {
            check_name(t, &["drop", "tryDrop"])?;
        } else {
            check_name(t, &["pack", "add", "equip", "replace"])?;
        }
    }
    if let Some(c) = o.get("contents") {
        c.as_array()
            .ok_or_else(|| format!("\"contents\": expected an array of items, got {c}"))?
            .iter()
            .try_for_each(|e| check_item(e, true))?;
    }
    Ok(())
}

fn layer(v: &Value) -> Option<muo_server::layer::Layer> {
    let s = v.as_str()?;
    (0..=255u8)
        .filter_map(muo_server::layer::Layer::from_u8)
        .find(|l| format!("{l:?}") == s)
}

fn loot(w: &mut World, name: &str, group: i32) -> Option<ItemId> {
    use muo_content::loot;
    match name {
        "Weapon" => loot::random_weapon(w, false, false, false),
        "ArmorOrShieldOrWeapon" => loot::random_armor_or_shield_or_weapon(w, false, false),
        "NecromancyReagent" => loot::random_necromancy_reagent(w),
        "PossibleReagent" => loot::random_possible_reagent(w),
        "Reagent" => loot::random_reagent(w),
        "Gem" => loot::random_gem(w),
        "BonsaiSeed" => crate::hand::random_bonsai_seed(w),
        "PeculiarSeed" => crate::hand::random_peculiar_seed(w, group),
        "ArcanistScroll" => loot::construct_any(w, loot::ARCANIST_SCROLL_TYPES),
        _ => None,
    }
}

/// What the items of one `creatureItem` list share: the `keep` items not placed yet, the last drawn hue.
#[derive(Default)]
struct ListState {
    held: Vec<(String, ItemId)>,
    hue: Option<i32>,
}

/// Where an item goes: on the creature, or into a container item.
#[derive(Copy, Clone)]
enum Place {
    Mobile,
    Into(ItemId),
}

/// One `creatureItem`, `times` times (drawn first). Each time the random amount, hue and constructor arguments are
/// drawn, the item is made (`item`, one class of `pick` (`null`: nothing), a `loot` table, a `plain` item id, or
/// the `held` item a `keep` stored earlier in the same list), set up (`setAmount` is drawn after the constructor), filled with its `contents`, and placed:
/// `"to": "pack"` (default, `PackItem`), `"add"` (`AddItem`), `"equip"` (`EquipItem`, deleted when it cannot be
/// worn) or `"replace"` (`ReplaceEquipment`); a content goes `"drop"` (default, `DropItem`) or `"tryDrop"`
/// (`TryDropItem`, deleted when it does not fit). `"keep": "<name>"` makes it and places it at a later `"held"`.
fn item(w: &mut World, id: MobileId, v: &Value, place: Place, st: &mut ListState) {
    let Some(o) = v.as_object() else { return };
    let times = o.get("times").map_or(1, |t| roll(w, t));
    for _ in 0..times {
        item_once(w, id, o, place, st);
    }
}

fn item_once(
    w: &mut World,
    id: MobileId,
    o: &Map<String, Value>,
    place: Place,
    st: &mut ListState,
) {
    use muo_content::props::Value as P;
    let class = match o.get("pick").and_then(Value::as_array) {
        Some(list) => {
            let i = rnd::below(w, list.len() as i32) as usize;
            match list.get(i).and_then(Value::as_str) {
                Some(c) => Some(c),
                None => return, // a `_ => null` arm
            }
        }
        None => o.get("item").and_then(Value::as_str),
    };
    let amount = o.get("amount").map(|a| roll(w, a));
    // `"hue": "same"`: the hue drawn for an item before it in the list
    let hue = match o.get("hue") {
        Some(Value::String(_)) => st.hue,
        Some(h) => Some(roll(w, h)),
        None => None,
    };
    if hue.is_some() {
        st.hue = hue;
    }
    let args: Vec<P> = o
        .get("args")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .map(
            |a| match (a.get("map").and_then(Value::as_str), a.get("enum")) {
                (Some(m), _) => P::Map(Some(if m == "Felucca" {
                    muo_server::map::MapId::FELUCCA
                } else {
                    muo_server::map::MapId::TRAMMEL
                })),
                (_, Some(e)) => P::Enum(e.as_i64().unwrap_or(0)),
                _ => P::Int(i64::from(roll(w, a))),
            },
        )
        .collect();
    let it = if let Some(c) = class {
        if args.is_empty() {
            factory::make_item(w, c)
        } else {
            // `new T(kind)` falls back to `new T()` when the class has no such overload
            factory::make_item_args(w, c, &args).or_else(|| match args.as_slice() {
                [P::Enum(_)] => factory::make_item(w, c),
                _ => None,
            })
        }
    } else if let Some(l) = o.get("loot").and_then(Value::as_str) {
        loot(w, l, o.get("group").map_or(0, int_of))
    } else if let Some(h) = o.get("held").and_then(Value::as_str) {
        st.held
            .iter()
            .position(|(n, _)| n == h)
            .map(|i| st.held.remove(i).1)
    } else {
        let gid = o.get("plain").map_or(0, int_of);
        Some(iops::new_item_with_id(w, &muo_server::item::ITEM_BASE, gid))
    };
    let Some(it) = it else { return };
    // `"setAmount"`: `item.Amount = ..` drawn after the constructor ran
    let amount = amount.or_else(|| o.get("setAmount").map(|a| roll(w, a)));
    if let Some(a) = amount {
        iops::set_amount(w, it, a);
    }
    if let Some(l) = o.get("layer").and_then(layer) {
        ibase::set_layer(w, it, l);
    }
    if let Some(n) = o.get("arcaneCharges") {
        crate::hand::set_arcane_charges(w, it, int_of(n));
    }
    if let Some(g) = o.get("itemId") {
        ibase::set_item_id(w, it, int_of(g));
    }
    if let Some(h) = hue {
        ibase::set_hue(w, it, h);
    }
    if let Some(m) = o.get("movable").and_then(Value::as_bool) {
        iops::set_movable(w, it, m);
    }
    if let Some(t) = o.get("lootType").and_then(Value::as_str) {
        use muo_server::item::LootType;
        let lt = match t {
            "Newbied" => LootType::Newbied,
            "Blessed" => LootType::Blessed,
            "Cursed" => LootType::Cursed,
            _ => LootType::Regular,
        };
        iops::set_loot_type(w, it, lt);
    }
    if o.get("crafterSelf").and_then(Value::as_bool) == Some(true) {
        let name = muo_server::mobile::part1::props_b::name(w, id);
        if let Some(d) = muo_content::items::weapon::data_mut(w, it) {
            d.crafter = name;
        }
    }
    if let Some(q) = o.get("quality").and_then(Value::as_str) {
        use muo_content::items::weapon::{WeaponQuality, set_quality};
        let q = match q {
            "Low" => WeaponQuality::Low,
            "Exceptional" => WeaponQuality::Exceptional,
            _ => WeaponQuality::Regular,
        };
        set_quality(w, it, q);
    }
    for c in o
        .get("contents")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        item(w, id, c, Place::Into(it), st);
    }
    if let Some(k) = o.get("keep").and_then(Value::as_str) {
        st.held.push((k.to_owned(), it));
        return;
    }
    let to = o.get("to").and_then(Value::as_str);
    match place {
        Place::Into(c) if to == Some("tryDrop") => {
            if !muo_server::item::container::try_drop_item(w, c, Some(id), it, false) {
                ibase::delete(w, it);
            }
        }
        Place::Into(c) => muo_server::item::container::drop_item_default(w, c, it),
        Place::Mobile => match to {
            Some("add") => items::add_item(w, id, it),
            Some("equip") => {
                if !muo_server::mobile::base::equip_item(w, id, it)
                    && let Some(t) = w.item(it).map(|i| i.ty)
                {
                    (t.delete)(w, it);
                }
            }
            Some("replace") => crate::npcs::vendors::replace_equipment_item(w, id, it),
            _ => setters::pack_item(w, id, Some(it)),
        },
    }
    // `armor.Resource = res` after `AddItem` (the dragoons' dragon armor)
    if let Some(r) = o.get("resource").and_then(Value::as_str).and_then(resource) {
        muo_content::items::armor::set_resource(w, it, r);
    }
}
