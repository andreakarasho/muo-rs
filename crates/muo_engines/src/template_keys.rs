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

/// `vendor.GetRandomHue()` / `vendor.GetShoeHue()`: only for keys that know the mobile.
const VENDOR_RANDOM_HUE: &str = "vendorRandomHue";
const VENDOR_SHOE_HUE: &str = "vendorShoeHue";

fn check_hue(v: &Value) -> Result<(), String> {
    match v {
        Value::String(s)
            if s == VENDOR_RANDOM_HUE
                || s == VENDOR_SHOE_HUE
                || RANDOM_HUES.iter().any(|(n, _)| n == s) =>
        {
            Ok(())
        }
        Value::Object(o) if o.len() == 1 => match o.get("coin").and_then(Value::as_array) {
            Some(a) if a.len() == 2 => a.iter().try_for_each(check_int),
            _ => Err(format!("expected {{\"coin\": [a, b]}}, got {v}")),
        },
        _ => check_int(v).map_err(|_| format!("expected a hue or a random hue name, got {v}")),
    }
}

/// [`hue`] for the mobile `owner`, which also knows `"vendorRandomHue"` and `"vendorShoeHue"`.
fn hue_of(w: &mut World, owner: MobileId, v: &Value) -> i32 {
    match v.as_str() {
        Some(VENDOR_RANDOM_HUE) => muo_content::vendor::get_random_hue(w, owner),
        Some(VENDOR_SHOE_HUE) => muo_content::vendor::get_shoe_hue(w, owner),
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
/// `"dye"` makes the item then draws (`new T { Hue = .. }`); `{"coin": [entry, entry]}` picks one. `"itemId": n`
/// sets `ItemID` (before the hue) and `"movable": false` `Movable`.
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
            "itemId" => check_int(val)?,
            "movable" => check_bool(val)?,
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
    if let (Some(i), Some(g)) = (item, v.get("itemId"))
        && let Some(ty) = w.item(i).map(|i| i.ty)
    {
        (ty.set_item_id)(w, i, int(g));
    }
    if let Some(h) = before {
        npc::set_item_hue(w, item, h);
    }
    if let Some(d) = v.get("dye") {
        let h = hue(w, d);
        npc::set_item_hue(w, item, h);
    }
    if let (Some(i), Some(m)) = (item, v.get("movable").and_then(Value::as_bool)) {
        muo_server::item::ops::set_movable(w, i, m);
    }
    npc::add_item_id(w, id, item);
}

/// One `questEquip` entry: `{"item": "Server.Items.Robe", "hue": <hue>, "movable": false}`: the hue is drawn,
/// then `AddItem(new T { Hue = hue, Movable = movable })` (the class by its full name). `"oneOf": [names]` in place of
/// `"item"` picks the class (`Utility.Random(n)`) first. `"equip": true` is `EquipItem` (a refused item is deleted)
/// in place of `AddItem`; `"ignite": true` lights the (light source) item once it is worn.
fn check_quest_equip(v: &Value) -> Result<(), String> {
    let o = v
        .as_object()
        .ok_or_else(|| format!("expected an equip entry, got {v}"))?;
    for (k, val) in o {
        match k.as_str() {
            "item" => check_str(val)?,
            "oneOf" => check_list(val, check_str)?,
            "hue" => check_hue(val)?,
            "movable" | "equip" | "ignite" => check_bool(val)?,
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
    if v.get("equip").and_then(Value::as_bool) == Some(true) {
        if !muo_server::mobile::base::equip_item(w, id, it) {
            muo_server::item::base::delete(w, it);
            return;
        }
    } else {
        muo_server::mobile::part2::items::add_item(w, id, it);
    }
    if v.get("ignite").and_then(Value::as_bool) == Some(true) {
        muo_items::lights::base::ignite(w, it);
    }
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
    // `HiddenFigure.Message = n` (saved).
    mobile_key(types, "hiddenFigureMessage", check_int, |w, id, v| {
        if let Some(m) = w.mobile_mut(id) {
            m.ext.insert(int(v));
        }
    });
    // `Direction = Direction.South`.
    mobile_key(
        types,
        "mlqDirection",
        |v| direction(v).map(drop),
        |w, id, v| {
            if let Ok(d) = direction(v) {
                muo_server::mobile::part2::effects::set_direction(w, id, d);
            }
        },
    );
    // `Home` range (`RangeHome = n`).
    mobile_key(types, "rangeHome", check_int, |w, id, v| {
        if let Some(d) = muo_content::creature::cd_mut(w, id) {
            d.range_home = int(v);
        }
    });
    // `BaseFactionGuard.GenerateBody`'s skin: `Hue = Utility.RandomSkinHue()` (`Random(1002, 57) | 0x8000`).
    mobile_key(types, "factionSkinHue", check_true, |w, id, _| {
        let h = utility::random_from(&mut w.rng, 1002, 57) | 0x8000;
        muo_server::mobile::base::set_hue(w, id, h);
    });
    mobile_key(
        types,
        "guardWear",
        |v| check_list(v, check_wear),
        |w, id, v| {
            for e in v.as_array().into_iter().flatten() {
                wear(w, id, e);
            }
        },
    );
    // `{"count": [min, max], "from": [classes]}`: `for (Utility.RandomMinMax(min, max)) PackItem(new <random of from>())`
    // (`PackStrongPotions` / `PackWeakPotions`).
    mobile_key(types, "packRandomItems", check_random_items, |w, id, v| {
        let c = nums(&v["count"]);
        let from = v["from"].as_array().map(Vec::as_slice).unwrap_or_default();
        let n = muo_content::hooks::rnd::min_max(w, c[0] as i32, c[1] as i32);
        for _ in 0..n {
            let i = muo_content::hooks::rnd::below(w, from.len() as i32) as usize;
            let item = muo_content::factory::make_item(w, from[i].as_str().unwrap_or(""));
            muo_content::creature::setters::pack_item(w, id, item);
        }
    });
    // `BasePigmentsOfTokuno.Label = n`.
    item_key(types, "pigmentLabel", check_int, |w, id, v| {
        crate::tokuno::pigments::set_label_key(w, id, int(v))
    });
    // `RewardBag.Fill(this, count, talismanChance)`.
    item_key(
        types,
        "rewardBagFill",
        |v| match (v.get("count").map(check_int), v.get("talismanChance")) {
            (Some(Ok(())), Some(p))
                if p.is_number() && v.as_object().is_some_and(|o| o.len() == 2) =>
            {
                Ok(())
            }
            _ => Err(format!(
                "expected {{\"count\": n, \"talismanChance\": p}}, got {v}"
            )),
        },
        |w, id, v| {
            let p = v["talismanChance"].as_f64().unwrap_or(0.0);
            crate::ml_quests::items2::fill(w, id, int(&v["count"]), p);
        },
    );
    // `AddBaseLoot(sets..)`: one random item of one of the named sets, enhanced.
    item_key(
        types,
        "satchelLoot",
        |v| {
            check_list(v, |s| match s.as_str() {
                Some(n) if loot_set(n).is_some() => Ok(()),
                _ => Err(format!("unknown loot set {s}")),
            })
        },
        |w, id, v| {
            let sets: Vec<&[&str]> = v
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|s| loot_set(s.as_str()?))
                .collect();
            crate::ml_quests::items2::add_base_loot(w, id, &sets);
        },
    );
    // `AddRecipe(craft system)`.
    item_key(types, "satchelRecipe", check_str, |w, id, v| {
        crate::ml_quests::items2::add_recipe(w, id, v.as_str().unwrap_or(""))
    });
    // `MaabusCoffin`'s components: `[[full, empty, x, y], ..]`.
    item_key(
        types,
        "coffinComponents",
        |v| check_list(v, |c| check_nums(c, 4)),
        |w, id, v| {
            let list: Vec<[i32; 4]> = v
                .as_array()
                .into_iter()
                .flatten()
                .map(|c| {
                    let n = nums(c);
                    [n[0] as i32, n[1] as i32, n[2] as i32, n[3] as i32]
                })
                .collect();
            crate::quests::dark_tides_mobs::add_coffin_components(w, id, &list);
        },
    );
    // An addon's plain components: `AddComponent(new AddonComponent(itemID), x, y, z)` for each `[itemID, x, y, z]`.
    item_key(
        types,
        "addonComponents",
        |v| check_list(v, |c| check_nums(c, 4)),
        |w, id, v| {
            use muo_content::items::addon;
            for c in v.as_array().into_iter().flatten() {
                let n = nums(c);
                let comp = addon::new_component(w, &addon::ADDON_COMPONENT, n[0] as i32);
                addon::add_component(w, id, comp, n[1] as i32, n[2] as i32, n[3] as i32);
            }
        },
    );
    // `[itemID, labelNumber, hue, x, y, z]`: a `LocalizedAddonComponent` of that hue.
    item_key(
        types,
        "addonLocalizedComponents",
        |v| check_list(v, |c| check_nums(c, 6)),
        |w, id, v| {
            use muo_content::items::addon;
            for c in v.as_array().into_iter().flatten() {
                let n: Vec<i32> = nums(c).into_iter().map(|x| x as i32).collect();
                let comp = addon::new_localized_component(
                    w,
                    &addon::LOCALIZED_ADDON_COMPONENT,
                    n[0],
                    n[1],
                );
                crate::util::hue(w, comp, n[2]);
                addon::add_component(w, id, comp, n[3], n[4], n[5]);
            }
        },
    );
    // `ChampionSkullPlatform`'s braziers: `[["Power", x, y], ..]`.
    item_key(
        types,
        "skullBraziers",
        |v| {
            check_list(v, |e| match e.as_array().map(Vec::as_slice) {
                Some([t, x, y]) if skull_type(t).is_some() && x.is_i64() && y.is_i64() => Ok(()),
                _ => Err(format!("expected [skull type, x, y], got {e}")),
            })
        },
        |w, id, v| {
            let list: Vec<_> = v
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|e| Some((skull_type(&e[0])?, int(&e[1]), int(&e[2]))))
                .collect();
            crate::canned_evil::items::add_skull_braziers(w, id, &list);
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

/// One `guardWear` entry, worn with `AddItem`: `{"item": class}`, `{"plainItem": itemID}` (`new Item(itemID)`) or
/// `{"virtualMount": true}` (the rider's `VirtualMountItem`), then `"layer"`, `"hue"`, `"movable": false` and
/// `"newbied": true` (`LootType.Newbied`) in that order.
fn check_wear(v: &Value) -> Result<(), String> {
    let o = v
        .as_object()
        .ok_or_else(|| format!("expected a wear entry, got {v}"))?;
    let mut what = 0;
    for (k, val) in o {
        match k.as_str() {
            "item" | "plainItem" | "virtualMount" => {
                match k.as_str() {
                    "item" => check_str(val)?,
                    "plainItem" => check_int(val)?,
                    _ => check_true(val)?,
                }
                what += 1;
            }
            "layer" => {
                layer(val).ok_or_else(|| format!("unknown layer {val}"))?;
            }
            "hue" => check_int(val)?,
            "movable" | "newbied" => check_bool(val)?,
            _ => return Err(format!("unknown wear key \"{k}\"")),
        }
    }
    if what == 1 {
        Ok(())
    } else {
        Err(format!(
            "a wear entry needs one of item, plainItem, virtualMount: {v}"
        ))
    }
}

fn layer(v: &Value) -> Option<muo_server::layer::Layer> {
    let s = v.as_str()?;
    (0..=0x1F)
        .map(muo_server::layer::Layer)
        .find(|l| format!("{l:?}") == s)
}

fn wear(w: &mut World, id: MobileId, v: &Value) {
    use muo_server::item::{ITEM_BASE, LootType, ops};
    let item = if let Some(g) = v.get("plainItem") {
        Some(ops::new_item_with_id(w, &ITEM_BASE, int(g)))
    } else if v.get("virtualMount").is_some() {
        Some(muo_content::creature::virtual_mount::new_virtual_mount_item(w, id))
    } else {
        muo_content::factory::make_item(w, v["item"].as_str().unwrap_or(""))
    };
    let Some(item) = item else { return };
    if let Some(l) = v.get("layer").and_then(layer)
        && let Some(ty) = w.item(item).map(|i| i.ty)
    {
        (ty.set_layer)(w, item, l);
    }
    if let Some(h) = v.get("hue") {
        muo_server::item::base::set_hue(w, item, int(h));
    }
    if let Some(m) = v.get("movable").and_then(Value::as_bool) {
        ops::set_movable(w, item, m);
    }
    if v.get("newbied").and_then(Value::as_bool) == Some(true) {
        ops::set_loot_type(w, item, LootType::Newbied);
    }
    muo_server::mobile::part2::items::add_item(w, id, item);
}

fn check_random_items(v: &Value) -> Result<(), String> {
    let ok = v.as_object().is_some_and(|o| o.len() == 2)
        && check_nums(&v["count"], 2).is_ok()
        && v["from"]
            .as_array()
            .is_some_and(|a| !a.is_empty() && a.iter().all(Value::is_string));
    if ok {
        Ok(())
    } else {
        Err(format!(
            "expected {{\"count\": [min, max], \"from\": [classes]}}, got {v}"
        ))
    }
}

fn loot_set(name: &str) -> Option<&'static [&'static str]> {
    crate::ml_quests::items2::LOOT_SETS
        .iter()
        .find(|(n, _)| *n == name)
        .map(|(_, s)| *s)
}

fn skull_type(v: &Value) -> Option<crate::canned_evil::info::ChampionSkullType> {
    let s = v.as_str()?;
    (0..6)
        .map(crate::canned_evil::info::ChampionSkullType::from_i32)
        .find(|t| format!("{t:?}") == s)
}

fn direction(v: &Value) -> Result<muo_primitives::Direction, String> {
    use muo_primitives::Direction as D;
    let all = [
        ("North", D::NORTH),
        ("Right", D::RIGHT),
        ("East", D::EAST),
        ("Down", D::DOWN),
        ("South", D::SOUTH),
        ("Left", D::LEFT),
        ("West", D::WEST),
        ("Up", D::UP),
    ];
    let s = v.as_str().unwrap_or("");
    all.iter()
        .find(|(n, _)| *n == s)
        .map(|(_, d)| *d)
        .ok_or_else(|| format!("expected a direction name, got {v}"))
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
