//! The item constructor statements an item template can hold, each through the item's own setter.

use serde_json::Value;

use muo_server::ids::{EntityId, ItemId};
use muo_server::item::{LootType, ops};
use muo_server::layer::Layer;
use muo_server::light_type::LightType;
use muo_server::world::World;

use super::keys::{KeyDef, Keys};
use super::mobile_steps::skill;
use super::{Flow, choose, parse_enum, parse_flow};
use crate::aos::{
    AosArmorAttribute, AosAttribute, AosElementAttribute, AosWeaponAttribute, AttrKind, Attrs,
};
use crate::items::set_attr;
use muo_server::skills::SkillName;

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
    /// `Attributes.X = v`, `WeaponAttributes.X = v`, ...: `(bag, bit, value)` in file order.
    Attrs(Vec<(AttrKind, u32, i32)>),
    /// `SkillBonuses.SetValues(index, skill, bonus)` for index 0, 1, ... in list order.
    SkillBonuses(Vec<(SkillName, f64)>),
    /// Any `[set` property by its path (`"Attributes.Luck"`, `"Quality"`), as staff would type it.
    Set(Vec<(String, String)>),
    Flow(Box<Flow<Step>>),
    /// A key a content crate registered (`templates::keys`).
    Ext(&'static str, KeyDef<ItemId>, Value),
}

fn attrs(kind: AttrKind, names: &[(&str, u32)], v: &Value) -> Result<Step, String> {
    let o = v
        .as_object()
        .ok_or_else(|| format!("expected an object, got {v}"))?;
    o.iter()
        .map(|(k, val)| {
            let bit = names
                .iter()
                .find(|(n, _)| *n == k)
                .map(|(_, b)| *b)
                .ok_or_else(|| format!("unknown attribute {k}"))?;
            Ok((kind, bit, int(val)?))
        })
        .collect::<Result<_, String>>()
        .map(Step::Attrs)
}

fn skill_bonuses(v: &Value) -> Result<Step, String> {
    let a = v.as_array().ok_or("expected [[\"Skill\", bonus], ...]")?;
    if a.len() > 5 {
        return Err("at most 5 skill bonuses".into());
    }
    a.iter()
        .map(|e| match e.as_array().map(Vec::as_slice) {
            Some([s, b]) => Ok((
                skill(string(s)?)?,
                b.as_f64()
                    .ok_or_else(|| format!("expected a number, got {b}"))?,
            )),
            _ => Err(format!("expected [\"Skill\", bonus], got {e}")),
        })
        .collect::<Result<_, String>>()
        .map(Step::SkillBonuses)
}

fn sets(v: &Value) -> Result<Step, String> {
    let o = v
        .as_object()
        .ok_or_else(|| format!("expected an object, got {v}"))?;
    o.iter()
        .map(|(k, val)| {
            let text = match val {
                Value::String(s) => s.clone(),
                Value::Number(_) | Value::Bool(_) => val.to_string(),
                Value::Null => "null".to_owned(),
                _ => return Err(format!("{k}: expected a string, number or bool")),
            };
            Ok((k.clone(), text))
        })
        .collect::<Result<_, String>>()
        .map(Step::Set)
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
pub fn parse(key: &str, v: &Value, keys: &Keys<'_>) -> Result<Step, String> {
    if let Some(f) = parse_flow(key, v, &|k: &str, v: &Value| parse(k, v, keys)) {
        return f.map(|f| Step::Flow(Box::new(f)));
    }
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
        "attributes" => attrs(AttrKind::Attributes, AosAttribute::NAMES, v)?,
        "weaponAttributes" => attrs(AttrKind::Weapon, AosWeaponAttribute::NAMES, v)?,
        "armorAttributes" => attrs(AttrKind::Armor, AosArmorAttribute::NAMES, v)?,
        "elementAttributes" => attrs(AttrKind::Element, AosElementAttribute::NAMES, v)?,
        "skillBonuses" => skill_bonuses(v)?,
        "set" => sets(v)?,
        _ => match keys.items.and_then(|k| k.0.get_key_value(key)) {
            Some((name, def)) => {
                (def.check)(v)?;
                Step::Ext(name, *def, v.clone())
            }
            None => return Err("unknown key".into()),
        },
    })
}

/// `[set path value]` at Owner level, without the command log. A path or value the property system rejects is a
/// warning (the property tables are registered at Configure, after the templates load, so it cannot be checked
/// earlier).
pub fn set_property(w: &mut World, id: ItemId, path: &str, text: &str) {
    use crate::props::{self, Access};
    let r = props::resolve_chain(
        w,
        muo_server::mobile::AccessLevel::Owner,
        EntityId::Item(id),
        path,
        Access::WRITE,
    )
    .and_then(|(owner, def)| {
        let v = props::parse_for(w, def, text)?;
        let set = def.set.ok_or("read-only")?;
        set(w, owner, v)
    });
    if let Err(e) = r {
        let ty = w.item(id).map_or("", |i| i.ty.full_name);
        tracing::warn!("Templates: {ty}: \"set\": {path} = {text}: {e}");
    }
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
        Step::Attrs(l) => {
            for (kind, bit, v) in l {
                set_attr(w, id, *kind, *bit, *v);
            }
        }
        Step::SkillBonuses(l) => {
            for (i, (skill, bonus)) in l.iter().enumerate() {
                set_attr(
                    w,
                    id,
                    AttrKind::SkillBonuses,
                    1 << i,
                    Attrs::skill_bonus_value(*skill, *bonus),
                );
            }
        }
        Step::Set(l) => {
            for (path, text) in l {
                set_property(w, id, path, text);
            }
        }
        Step::Flow(f) => {
            for s in choose(w, f) {
                apply(w, id, s);
            }
        }
        Step::Ext(_, def, v) => (def.apply)(w, id, v),
    }
}
