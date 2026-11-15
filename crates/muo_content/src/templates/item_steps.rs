//! The item constructor statements an item template can hold, each through the item's own setter.

use serde_json::Value;

use muo_server::ids::ItemId;
use muo_server::item::{LootType, ops};
use muo_server::layer::Layer;
use muo_server::light_type::LightType;
use muo_server::world::World;

use super::parse_enum;

#[derive(Clone, Debug, PartialEq)]
pub enum Step {
    ItemId(i32),
    Hue(i32),
    Weight(f64),
    Amount(i32),
    Name(String),
    Movable(bool),
    Stackable(bool),
    Visible(bool),
    LootType(LootType),
    Layer(Layer),
    Light(LightType),
}

fn loot_type(s: &str) -> Option<LootType> {
    let names = ["Regular", "Newbied", "Blessed", "Cursed"];
    names
        .iter()
        .position(|n| *n == s)
        .map(|i| LootType(i as u8))
}

fn layer(s: &str) -> Option<Layer> {
    (0..=Layer::Bank.0).map(Layer).find(|l| l.name() == Some(s))
}

fn int(v: &Value) -> Result<i32, String> {
    v.as_i64()
        .and_then(|n| i32::try_from(n).ok())
        .ok_or_else(|| format!("expected an integer, got {v}"))
}

fn boolean(v: &Value) -> Result<bool, String> {
    v.as_bool()
        .ok_or_else(|| format!("expected true or false, got {v}"))
}

fn string(v: &Value) -> Result<&str, String> {
    v.as_str()
        .ok_or_else(|| format!("expected a string, got {v}"))
}

/// One `"key": value` of an item template.
pub fn parse(key: &str, v: &Value) -> Result<Step, String> {
    Ok(match key {
        "itemId" => Step::ItemId(int(v)?),
        "hue" => Step::Hue(int(v)?),
        "weight" => Step::Weight(
            v.as_f64()
                .ok_or_else(|| format!("expected a number, got {v}"))?,
        ),
        "amount" => Step::Amount(int(v)?),
        "name" => Step::Name(string(v)?.to_owned()),
        "movable" => Step::Movable(boolean(v)?),
        "stackable" => Step::Stackable(boolean(v)?),
        "visible" => Step::Visible(boolean(v)?),
        "lootType" => Step::LootType(loot_type(string(v)?).ok_or("unknown loot type")?),
        "layer" => Step::Layer(layer(string(v)?).ok_or("unknown layer")?),
        "light" => {
            Step::Light(parse_enum(string(v)?, LightType::from_i32).ok_or("unknown light type")?)
        }
        _ => return Err("unknown key".into()),
    })
}

pub fn apply(w: &mut World, id: ItemId, s: &Step) {
    let Some(ty) = w.item(id).map(|i| i.ty) else {
        return;
    };
    match s {
        Step::ItemId(v) => (ty.set_item_id)(w, id, *v),
        Step::Hue(v) => (ty.set_hue)(w, id, *v),
        Step::Weight(v) => ops::set_weight(w, id, *v),
        Step::Amount(v) => ops::set_amount(w, id, *v),
        Step::Name(v) => ops::set_name(w, id, Some(v.clone())),
        Step::Movable(v) => ops::set_movable(w, id, *v),
        Step::Stackable(v) => ops::set_stackable(w, id, *v),
        Step::Visible(v) => ops::set_visible(w, id, *v),
        Step::LootType(v) => ops::set_loot_type(w, id, *v),
        Step::Layer(v) => (ty.set_layer)(w, id, *v),
        Step::Light(v) => ops::set_light(w, id, *v),
    }
}
