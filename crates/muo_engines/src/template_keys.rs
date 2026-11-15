//! Template keys and class props of muo_engines content (`muo_content::templates::keys`).
//!
//! Mobile keys (`mlq*`) are the statements of the ML quest givers' constructors (`ml_quests/npc_support.rs`); item
//! keys (`questItem*`) those of the quest items. A hue value is an integer, a random hue name (`"randomPinkHue"`,
//! .. the `Utility.Random*Hue` draws) or `{"coin": [a, b]}` (`Utility.RandomBool() ? a : b`).

use serde_json::Value;

use muo_content::items::slayer::SlayerName;
use muo_content::templates::keys::{check_bool, check_int, int, item_key, mobile_key};
use muo_content::templates::props;
use muo_primitives::utility;
use muo_server::ids::{ItemId, MobileId};
use muo_server::type_registry::TypeRegistry;
use muo_server::world::World;

use crate::ml_quests::npc_support as npc;

type HueDraw = fn(&mut muo_primitives::BuiltInRng) -> i32;

const RANDOM_HUES: &[(&str, HueDraw)] = &[
    ("randomPinkHue", utility::random_pink_hue),
    ("randomBlueHue", utility::random_blue_hue),
    ("randomGreenHue", utility::random_green_hue),
    ("randomOrangeHue", utility::random_orange_hue),
    ("randomRedHue", utility::random_red_hue),
    ("randomYellowHue", utility::random_yellow_hue),
    ("randomNeutralHue", utility::random_neutral_hue),
    ("randomAnimalHue", utility::random_animal_hue),
    ("randomNondyedHue", utility::random_nondyed_hue),
    ("randomBrightHue", utility::random_bright_hue),
    ("randomDyedHue", utility::random_dyed_hue),
];

/// `vendor.GetRandomHue()`: only for keys that know the mobile.
const VENDOR_RANDOM_HUE: &str = "vendorRandomHue";

fn check_hue(v: &Value) -> Result<(), String> {
    match v {
        Value::String(s) if s == VENDOR_RANDOM_HUE || RANDOM_HUES.iter().any(|(n, _)| n == s) => {
            Ok(())
        }
        Value::Object(o) if o.len() == 1 => match o.get("coin").and_then(Value::as_array) {
            Some(a) if a.len() == 2 => a.iter().try_for_each(check_int),
            _ => Err(format!("expected {{\"coin\": [a, b]}}, got {v}")),
        },
        _ => check_int(v).map_err(|_| format!("expected a hue or a random hue name, got {v}")),
    }
}

/// [`hue`] for the mobile `owner`, which also knows `"vendorRandomHue"`.
fn hue_of(w: &mut World, owner: MobileId, v: &Value) -> i32 {
    match v.as_str() {
        Some(VENDOR_RANDOM_HUE) => muo_content::vendor::get_random_hue(w, owner),
        _ => hue(w, v),
    }
}

/// The hue of a checked hue value, drawing the RNG now.
fn hue(w: &mut World, v: &Value) -> i32 {
    match v {
        Value::String(s) => RANDOM_HUES
            .iter()
            .find(|(n, _)| n == s)
            .map_or(0, |(_, f)| f(&mut w.rng)),
        Value::Object(o) => {
            let a = o.get("coin").and_then(Value::as_array);
            let pick = usize::from(!muo_content::hooks::rnd::bool(w));
            a.and_then(|a| a.get(pick)).map_or(0, int)
        }
        _ => int(v),
    }
}

fn check_str(v: &Value) -> Result<(), String> {
    v.as_str()
        .map(|_| ())
        .ok_or_else(|| format!("expected a string, got {v}"))
}

fn check_true(v: &Value) -> Result<(), String> {
    match v {
        Value::Bool(true) => Ok(()),
        _ => Err(format!("only true is meaningful, got {v}")),
    }
}

fn check_nums(v: &Value, n: usize) -> Result<(), String> {
    match v.as_array() {
        Some(a) if a.len() == n && a.iter().all(Value::is_number) => Ok(()),
        _ => Err(format!("expected {n} numbers, got {v}")),
    }
}

fn nums(v: &Value) -> Vec<f64> {
    v.as_array()
        .map(|a| a.iter().filter_map(Value::as_f64).collect())
        .unwrap_or_default()
}

/// One `mlqEquip` entry: `{"item": "Robe", "hue": <hue>}` draws the hue then makes the item (`AddItem(new T(hue))`);
/// `"dye"` makes the item then draws (`new T { Hue = .. }`); `{"coin": [entry, entry]}` picks one.
fn check_equip_entry(v: &Value) -> Result<(), String> {
    let o = v
        .as_object()
        .ok_or_else(|| format!("expected an equip entry, got {v}"))?;
    if let Some(c) = o.get("coin") {
        return match c.as_array() {
            Some(a) if a.len() == 2 && o.len() == 1 => a.iter().try_for_each(check_equip_entry),
            _ => Err("expected {\"coin\": [entry, entry]}".into()),
        };
    }
    for (k, val) in o {
        match k.as_str() {
            "item" => check_str(val)?,
            "hue" | "dye" => check_hue(val)?,
            _ => return Err(format!("unknown equip key \"{k}\"")),
        }
    }
    if !o.contains_key("item") || (o.contains_key("hue") && o.contains_key("dye")) {
        return Err(format!(
            "an equip entry needs \"item\" and at most one of hue/dye: {v}"
        ));
    }
    Ok(())
}

fn equip_entry(w: &mut World, id: MobileId, v: &Value) {
    if let Some(a) = v.get("coin").and_then(Value::as_array) {
        let pick = usize::from(!muo_content::hooks::rnd::bool(w));
        return equip_entry(w, id, &a[pick]);
    }
    let name = v.get("item").and_then(Value::as_str).unwrap_or("");
    let before = v.get("hue").map(|h| hue(w, h));
    let item = npc::make_named(w, name);
    if let Some(h) = before {
        npc::set_item_hue(w, item, h);
    }
    if let Some(d) = v.get("dye") {
        let h = hue(w, d);
        npc::set_item_hue(w, item, h);
    }
    npc::add_item_id(w, id, item);
}

/// One `questEquip` entry: `{"item": "Server.Items.Robe", "hue": <hue>, "movable": false}`: the hue is drawn,
/// then `AddItem(new T { Hue = hue, Movable = movable })` (the class by its full name). `"oneOf": [names]` in place of
/// `"item"` picks the class (`Utility.Random(n)`) first.
fn check_quest_equip(v: &Value) -> Result<(), String> {
    let o = v
        .as_object()
        .ok_or_else(|| format!("expected an equip entry, got {v}"))?;
    for (k, val) in o {
        match k.as_str() {
            "item" => check_str(val)?,
            "oneOf" => check_list(val, check_str)?,
            "hue" => check_hue(val)?,
            "movable" => check_bool(val)?,
            _ => return Err(format!("unknown equip key \"{k}\"")),
        }
    }
    if o.contains_key("item") != o.contains_key("oneOf") {
        Ok(())
    } else {
        Err(format!("an equip entry needs \"item\" or \"oneOf\": {v}"))
    }
}

fn quest_equip(w: &mut World, id: MobileId, v: &Value) {
    let name = match v.get("oneOf").and_then(Value::as_array) {
        Some(a) if !a.is_empty() => {
            let i = muo_content::hooks::rnd::below(w, a.len() as i32) as usize;
            a[i].as_str().unwrap_or("").to_owned()
        }
        _ => v
            .get("item")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_owned(),
    };
    let h = v.get("hue").map(|h| hue_of(w, id, h));
    let Some(it) = muo_content::factory::make_item(w, &name) else {
        return;
    };
    if let Some(h) = h {
        crate::quests::quester::set_hue(w, it, h);
    }
    if let Some(m) = v.get("movable").and_then(Value::as_bool) {
        muo_server::item::ops::set_movable(w, it, m);
    }
    muo_server::mobile::part2::items::add_item(w, id, it);
}

fn check_list(v: &Value, f: fn(&Value) -> Result<(), String>) -> Result<(), String> {
    v.as_array()
        .ok_or_else(|| format!("expected an array, got {v}"))?
        .iter()
        .try_for_each(f)
}

/// `[["Feather", 10], ["FletcherTools"]]`: `DropItem(new T(amount))` in order.
fn check_drop(v: &Value) -> Result<(), String> {
    check_list(v, |e| match e.as_array().map(Vec::as_slice) {
        Some([n]) => check_str(n),
        Some([n, a]) => check_str(n).and(check_int(a)),
        _ => Err(format!(
            "expected [\"Class\"] or [\"Class\", amount], got {e}"
        )),
    })
}

fn drop_items(w: &mut World, id: ItemId, v: &Value) {
    for e in v.as_array().into_iter().flatten() {
        let name = e.get(0).and_then(Value::as_str).unwrap_or("");
        crate::ml_quests::item_support::drop_new(w, id, name, e.get(1).map(int));
    }
}

pub(crate) fn register(types: &mut TypeRegistry) {
    mobile_key(types, "mlqRace", check_str, |w, id, v| {
        npc::set_race_named(w, id, v.as_str().unwrap_or(""))
    });
    mobile_key(types, "mlqHue", check_hue, |w, id, v| {
        let h = hue(w, v);
        npc::set_hue(w, id, h);
    });
    mobile_key(types, "mlqSkinHue", check_true, |w, id, _| {
        let h = npc::race_skin_hue(w, id);
        npc::set_hue(w, id, h);
    });
    mobile_key(types, "mlqHairItemId", check_int, |w, id, v| {
        npc::set_hair_item_id(w, id, int(v))
    });
    mobile_key(types, "mlqHairHue", check_hue, |w, id, v| {
        let h = hue(w, v);
        npc::set_hair_hue(w, id, h);
    });
    mobile_key(types, "mlqFacialHairItemId", check_int, |w, id, v| {
        npc::set_facial_hair_item_id(w, id, int(v))
    });
    mobile_key(types, "mlqFacialHairHue", check_hue, |w, id, v| {
        let h = hue(w, v);
        npc::set_facial_hair_hue(w, id, h);
    });
    mobile_key(types, "mlqRandomHair", check_true, |w, id, _| {
        npc::assign_random_hair(w, id)
    });
    mobile_key(types, "mlqFrozen", check_bool, |w, id, v| {
        npc::set_frozen(w, id, v.as_bool().unwrap_or(false))
    });
    // `SetSpeed(active, passive, true)`.
    mobile_key(
        types,
        "mlqSpeed",
        |v| check_nums(v, 2),
        |w, id, v| {
            let n = nums(v);
            npc::set_speed(w, id, n[0], n[1], true);
        },
    );
    // `InitStats(str, dex, int)`.
    mobile_key(
        types,
        "mlqInitStats",
        |v| check_nums(v, 3),
        |w, id, v| {
            let n = nums(v);
            npc::init_stats(w, id, n[0] as i32, n[1] as i32, n[2] as i32);
        },
    );
    mobile_key(
        types,
        "mlqEquip",
        |v| check_list(v, check_equip_entry),
        |w, id, v| {
            for e in v.as_array().into_iter().flatten() {
                equip_entry(w, id, e);
            }
        },
    );
    // The quest NPCs and creatures of `quests/` (`quests/util.rs` and the files' constructor helpers).
    mobile_key(types, "questSkinHue", check_true, |w, id, _| {
        let h = muo_content::vendor::human_random_skin_hue(w);
        muo_server::mobile::base::set_hue(w, id, h);
    });
    mobile_key(types, "questHairItem", check_true, |w, id, _| {
        crate::quests::util::assign_hair_item(w, id)
    });
    mobile_key(types, "questFacialHairItem", check_true, |w, id, _| {
        crate::quests::util::assign_facial_item(w, id)
    });
    mobile_key(types, "questRandomFacialHair", check_true, |w, id, _| {
        crate::quests::util::assign_facial_random_hue(w, id)
    });
    // `Utility.AssignRandomFacialHair(m, m.HairHue)`.
    mobile_key(types, "questFacialHairOfHairHue", check_true, |w, id, _| {
        let h = w.mobile(id).map_or(0, |m| m.hair_hue);
        muo_content::vendor::assign_random_facial_hair(w, id, h);
    });
    mobile_key(
        types,
        "questEquip",
        |v| check_list(v, check_quest_equip),
        |w, id, v| {
            for e in v.as_array().into_iter().flatten() {
                quest_equip(w, id, e);
            }
        },
    );
    item_key(types, "questItemHue", check_hue, |w, id, v| {
        let h = hue(w, v);
        crate::ml_quests::item_support::hue(w, id, h);
    });
    item_key(types, "questItemDrop", check_drop, drop_items);
    // The Tokuno artifacts' `Slayer = ..` / `Slayer2 = ..`.
    item_key(types, "tokunoSlayer", check_slayer, |w, id, v| {
        muo_items::weapons::ctor::set_slayer(w, id, slayer(v))
    });
    item_key(types, "tokunoSlayer2", check_slayer, |w, id, v| {
        muo_items::weapons::ctor::set_slayer2(w, id, slayer(v) as i32)
    });
}

fn slayer(v: &Value) -> SlayerName {
    v.as_str()
        .and_then(|s| {
            (0..64)
                .filter_map(SlayerName::from_i32)
                .find(|n| format!("{n:?}") == s)
        })
        .unwrap_or(SlayerName::None)
}

fn check_slayer(v: &Value) -> Result<(), String> {
    match v.as_str() {
        Some("None") => Ok(()),
        Some(_) if slayer(v) != SlayerName::None => Ok(()),
        _ => Err(format!("expected a SlayerName, got {v}")),
    }
}

// Class props read by the converted classes' members; without the prop a member answers what its parent answers.

/// `LabelNumber`: the `labelNumber` prop.
pub fn item_label(w: &World, id: ItemId, parent: fn(&World, ItemId) -> i32) -> i32 {
    item_int(w, id, "labelNumber", parent)
}

pub use muo_content::template_keys::{item_bool, item_float, item_int, item_name};

/// A bool member of a creature class (`isInvulnerable`, `canShout`, `canTeach`).
pub fn mobile_bool(
    w: &World,
    id: MobileId,
    key: &str,
    parent: fn(&World, MobileId) -> bool,
) -> bool {
    props::as_bool(props::mobile(w, id, key)).unwrap_or_else(|| parent(w, id))
}
