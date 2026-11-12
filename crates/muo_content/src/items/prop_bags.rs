//! `[CommandProperty]` members of the classes that keep `AosAttributes` / `AosSkillBonuses` bags and
//! `TalismanAttribute` objects in `ext_bags` data: `BaseQuiver`, `BaseTalisman`, `Spellbook` (their scalar members are
//! generated into `props_gen.rs`), and `ElvenGlasses.WeaponAttributes`.
//!
//! A bag is a `Kind::Object` over the owner's `Attrs` (`items::attrs`); a `TalismanAttribute` is one over the owner's
//! `protection` / `killer` / `summoner` field, so each of the three has a table of its own.

use muo_server::ids::{EntityId, ItemId};
use muo_server::mobile::AccessLevel;
use muo_server::type_registry::EntityType;
use muo_server::world::World;

use super::ext_bags::{TalismanAttr, TalismanData};
use crate::aos::AttrKind;
use crate::props::{self, Kind, PropDef, Value};
use crate::text_def_prop;
use crate::utilities::types::{Numeric, PropType};

const GM: AccessLevel = AccessLevel::GameMaster;
const INT: Kind = Kind::Plain(PropType::Numeric(Numeric::Int));

fn item_of(e: EntityId) -> Option<ItemId> {
    match e {
        EntityId::Item(i) => Some(i),
        EntityId::Mobile(_) => None,
    }
}

fn bag(
    name: &'static str,
    cname: &'static str,
    kind: AttrKind,
    subs: &'static [PropDef],
) -> PropDef {
    // the getter is a plain `fn`: one per bag kind
    macro_rules! get_for {
        ($k:expr) => {
            |w: &mut World, e: EntityId| match item_of(e) {
                Some(i) if w.item(i).and_then(|x| crate::items::attrs(x, $k)).is_some() => {
                    Value::Text("...".into())
                }
                _ => Value::Null,
            }
        };
    }
    let get: props::GetFn = match kind {
        AttrKind::Attributes => get_for!(AttrKind::Attributes),
        AttrKind::SkillBonuses => get_for!(AttrKind::SkillBonuses),
        AttrKind::Weapon => get_for!(AttrKind::Weapon),
        AttrKind::Armor => get_for!(AttrKind::Armor),
        AttrKind::Element => get_for!(AttrKind::Element),
    };
    PropDef {
        name,
        kind: Kind::Object(cname, subs),
        read: GM,
        write: GM,
        get,
        set: None,
    }
}

fn talisman(w: &World, e: EntityId) -> Option<&TalismanData> {
    w.item(item_of(e)?)?.ext.get::<TalismanData>()
}

fn talisman_mut(w: &mut World, e: EntityId) -> Option<&mut TalismanData> {
    w.item_mut(item_of(e)?)?.ext.get_mut::<TalismanData>()
}

/// `TalismanAttribute`'s members over the `TalismanAttr` that `$f` of the owner's `TalismanData` is.
macro_rules! attr_table {
    ($f:ident) => {
        &[
            // `Type`: a class by its full name
            PropDef {
                name: "Type",
                kind: Kind::Plain(PropType::Type),
                read: GM,
                write: GM,
                get: |w, e| {
                    talisman(w, e).map_or(Value::Null, |d| {
                        d.$f.type_name.clone().map_or(Value::Null, Value::Str)
                    })
                },
                set: Some(|w, e, v| {
                    set_attr(w, e, |d| {
                        d.$f.type_name = v.string().filter(|s| !s.is_empty())
                    })
                }),
            },
            text_def_prop!(
                "Name",
                GM,
                GM,
                |w: &World, e: EntityId| talisman(w, e).and_then(|d| d.$f.name.clone()),
                Some(|w: &mut World, e: EntityId, v: Value| {
                    set_attr(w, e, |d| {
                        d.$f.name = match &v {
                            Value::TextDef(t) => Some(t.clone()),
                            _ => None,
                        }
                    })
                })
            ),
            PropDef {
                name: "Amount",
                kind: INT,
                read: GM,
                write: GM,
                get: |w, e| {
                    talisman(w, e).map_or(Value::Null, |d| Value::Int(i64::from(d.$f.amount)))
                },
                set: Some(|w, e, v| set_attr(w, e, |d| d.$f.amount = v.i32())),
            },
            // `IsEmpty => _type == null`
            PropDef {
                name: "IsEmpty",
                kind: Kind::Plain(PropType::Bool),
                read: GM,
                write: GM,
                get: |w, e| talisman(w, e).map_or(Value::Null, |d| Value::Bool(d.$f.is_empty())),
                set: None,
            },
            // `_type.IsAssignableTo(typeof(Item))` (a null type: null)
            PropDef {
                name: "IsItem",
                kind: Kind::Plain(PropType::Bool),
                read: GM,
                write: GM,
                get: |w, e| match talisman(w, e).and_then(|d| d.$f.type_name.clone()) {
                    Some(n) => match w.types.find_type_by_full_name(&n, false) {
                        Some(EntityType::Item(_)) => Value::Bool(true),
                        _ => Value::Bool(false),
                    },
                    None => Value::Null,
                },
                set: None,
            },
        ]
    };
}

/// A `TalismanAttribute` setter marks its owner dirty (`[DirtyTrackingEntity] _owner`): there is no dirty set, a save
/// writes every item, so writing the field is all there is. The setters do not invalidate properties: only assigning the
/// owner's whole `Protection` / `Killer` / `Summoner` does (`[InvalidateProperties]` on those fields).
fn set_attr(w: &mut World, e: EntityId, f: impl FnOnce(&mut TalismanData)) -> Result<(), String> {
    item_of(e).ok_or_else(|| "That is not an item.".to_string())?;
    let d = talisman_mut(w, e).ok_or_else(|| "That object has no such state.".to_string())?;
    f(d);
    Ok(())
}

static PROTECTION: &[PropDef] = attr_table!(protection);
static KILLER: &[PropDef] = attr_table!(killer);
static SUMMONER: &[PropDef] = attr_table!(summoner);

fn attr_object(name: &'static str, subs: &'static [PropDef], get: props::GetFn) -> PropDef {
    PropDef {
        name,
        kind: Kind::Object("TalismanAttribute", subs),
        read: GM,
        write: GM,
        get,
        set: None,
    }
}

fn some_attr(a: Option<&TalismanAttr>) -> Value {
    a.map_or(Value::Null, |a| {
        Value::Text(
            a.type_name
                .as_deref()
                .map_or("None", |t| t.rsplit('.').next().unwrap_or(t))
                .to_string(),
        )
    })
}

pub fn register(w: &mut World) {
    use super::prop_tables::{AOS_ATTRIBUTES, AOS_SKILL_BONUSES};
    let quiver: &'static [PropDef] = Box::leak(Box::new([bag(
        "Attributes",
        "AosAttributes",
        AttrKind::Attributes,
        AOS_ATTRIBUTES,
    )]));
    props::add(w, "Server.Items.BaseQuiver", quiver);
    let talisman_bags: &'static [PropDef] = Box::leak(Box::new([
        bag(
            "Attributes",
            "AosAttributes",
            AttrKind::Attributes,
            AOS_ATTRIBUTES,
        ),
        bag(
            "SkillBonuses",
            "AosSkillBonuses",
            AttrKind::SkillBonuses,
            AOS_SKILL_BONUSES,
        ),
        attr_object("Protection", PROTECTION, |w, e| {
            some_attr(talisman(w, e).map(|d| &d.protection))
        }),
        attr_object("Killer", KILLER, |w, e| {
            some_attr(talisman(w, e).map(|d| &d.killer))
        }),
        attr_object("Summoner", SUMMONER, |w, e| {
            some_attr(talisman(w, e).map(|d| &d.summoner))
        }),
    ]));
    props::add(w, "Server.Items.BaseTalisman", talisman_bags);
    let spellbook: &'static [PropDef] = Box::leak(Box::new([
        bag(
            "Attributes",
            "AosAttributes",
            AttrKind::Attributes,
            AOS_ATTRIBUTES,
        ),
        bag(
            "SkillBonuses",
            "AosSkillBonuses",
            AttrKind::SkillBonuses,
            AOS_SKILL_BONUSES,
        ),
    ]));
    props::add(w, "Server.Items.Spellbook", spellbook);
    // `ElvenGlasses._weaponAttributes` (`AosWeaponAttributes`, `canModify: true`)
    let glasses: &'static [PropDef] = Box::leak(Box::new([bag(
        "WeaponAttributes",
        "AosWeaponAttributes",
        AttrKind::Weapon,
        super::prop_tables::AOS_WEAPON_ATTRIBUTES,
    )]));
    props::add(w, "Server.Items.ElvenGlasses", glasses);
}
