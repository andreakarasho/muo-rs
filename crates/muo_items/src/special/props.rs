//! `[CommandProperty]` members of the aquarium family that are not plain fields: `Aquarium.Food` / `Water` (two
//! `AquariumState` objects, so two tables over the aquarium's `AquariumData`), the counts, `BaseFish.Dead`,
//! `FishBowl.Empty` / `Fish`.

use muo_content::props::{self, Kind, PropDef, Value};
use muo_content::utilities::types::{Numeric, PropType};
use muo_server::ids::EntityId;
use muo_server::item::ops as iops;
use muo_server::mobile::AccessLevel;
use muo_server::world::World;

use super::aquarium::{self, AquariumData, StateData};
use super::fish;

const GM: AccessLevel = AccessLevel::GameMaster;
const INT: Kind = Kind::Plain(PropType::Numeric(Numeric::Int));
const BOOL: Kind = Kind::Plain(PropType::Bool);

fn item_of(e: EntityId) -> Option<muo_server::ids::ItemId> {
    match e {
        EntityId::Item(i) => Some(i),
        EntityId::Mobile(_) => None,
    }
}

fn data(w: &World, e: EntityId) -> Option<&AquariumData> {
    item_of(e).and_then(|i| aquarium::data(w, i))
}

/// Writes `AquariumState`'s field `f` of the aquarium's `$state`; `State` clamps to 0..=4 (`AllowStateChange`).
fn edit(w: &mut World, e: EntityId, f: impl FnOnce(&mut AquariumData)) -> Result<(), String> {
    let i = item_of(e).ok_or_else(|| "That is not an item.".to_string())?;
    let d = w
        .item_mut(i)
        .and_then(|x| x.ext.get_mut::<AquariumData>())
        .ok_or_else(|| "That object has no such state.".to_string())?;
    f(d);
    // `AquariumState` is not an item: its own fields do not invalidate; the aquarium's `Food` / `Water` are
    // `[InvalidateProperties]` fields, whose setter is not run by a write to a member of the object
    let _ = iops::invalidate_properties;
    Ok(())
}

macro_rules! state_table {
    ($f:ident) => {
        &[
            PropDef {
                name: "State",
                kind: INT,
                read: GM,
                write: GM,
                get: |w, e| data(w, e).map_or(Value::Null, |d| Value::Int(i64::from(d.$f.state))),
                set: Some(|w, e, v| {
                    let n = v.i32().clamp(0, 4);
                    edit(w, e, |d| {
                        if d.$f.state != n {
                            d.$f.state = n
                        }
                    })
                }),
            },
            PropDef {
                name: "Maintain",
                kind: INT,
                read: GM,
                write: GM,
                get: |w, e| {
                    data(w, e).map_or(Value::Null, |d| Value::Int(i64::from(d.$f.maintain)))
                },
                set: Some(|w, e, v| edit(w, e, |d| d.$f.maintain = v.i32())),
            },
            PropDef {
                name: "Improve",
                kind: INT,
                read: GM,
                write: GM,
                get: |w, e| data(w, e).map_or(Value::Null, |d| Value::Int(i64::from(d.$f.improve))),
                set: Some(|w, e, v| edit(w, e, |d| d.$f.improve = v.i32())),
            },
            PropDef {
                name: "Added",
                kind: INT,
                read: GM,
                write: GM,
                get: |w, e| data(w, e).map_or(Value::Null, |d| Value::Int(i64::from(d.$f.added))),
                set: Some(|w, e, v| edit(w, e, |d| d.$f.added = v.i32())),
            },
        ]
    };
}

static FOOD: &[PropDef] = state_table!(food);
static WATER: &[PropDef] = state_table!(water);

// `AquariumState.ToString() => "..."`
static AQUARIUM: &[PropDef] = &[
    PropDef {
        name: "Food",
        kind: Kind::Object("AquariumState", FOOD),
        read: GM,
        write: GM,
        get: |w, e| data(w, e).map_or(Value::Null, |_| Value::Text("...".into())),
        set: None,
    },
    PropDef {
        name: "Water",
        kind: Kind::Object("AquariumState", WATER),
        read: GM,
        write: GM,
        get: |w, e| data(w, e).map_or(Value::Null, |_| Value::Text("...".into())),
        set: None,
    },
    PropDef {
        name: "DeadCreatures",
        kind: INT,
        read: GM,
        write: GM,
        get: |w, e| {
            item_of(e).map_or(Value::Null, |i| {
                Value::Int(i64::from(aquarium::dead_creatures(w, i)))
            })
        },
        set: None,
    },
    PropDef {
        name: "MaxLiveCreatures",
        kind: INT,
        read: GM,
        write: GM,
        get: |w, e| {
            item_of(e).map_or(Value::Null, |i| {
                Value::Int(i64::from(aquarium::max_live_creatures(w, i)))
            })
        },
        set: None,
    },
    PropDef {
        name: "IsFull",
        kind: BOOL,
        read: GM,
        write: GM,
        get: |w, e| item_of(e).map_or(Value::Null, |i| Value::Bool(aquarium::is_full(w, i))),
        set: None,
    },
    PropDef {
        name: "OptimalState",
        kind: BOOL,
        read: GM,
        write: GM,
        get: |w, e| item_of(e).map_or(Value::Null, |i| Value::Bool(aquarium::optimal_state(w, i))),
        set: None,
    },
];

static FISH: &[PropDef] = &[PropDef {
    name: "Dead",
    kind: BOOL,
    read: GM,
    write: GM,
    get: |w, e| item_of(e).map_or(Value::Null, |i| Value::Bool(fish::is_dead(w, i))),
    set: None,
}];

static FISH_BOWL: &[PropDef] = &[
    PropDef {
        name: "Empty",
        kind: BOOL,
        read: GM,
        write: GM,
        get: |w, e| {
            item_of(e)
                .and_then(|i| w.item(i))
                .map_or(Value::Null, |x| Value::Bool(x.items.is_empty()))
        },
        set: None,
    },
    // `Empty ? null : Items[0] as BaseFish`
    PropDef {
        name: "Fish",
        kind: Kind::Plain(PropType::Entity),
        read: GM,
        write: GM,
        get: |w, e| {
            let first = item_of(e)
                .and_then(|i| w.item(i))
                .and_then(|x| x.items.first().copied());
            match first {
                Some(f) if w.item(f).is_some_and(|x| x.ty.is(&fish::BASE_FISH)) => {
                    Value::Entity(Some(EntityId::Item(f)))
                }
                _ => Value::Entity(None),
            }
        },
        set: None,
    },
];

pub fn configure(w: &mut World) {
    props::add(w, "Server.Items.Aquarium", AQUARIUM);
    props::add(w, "Server.Items.BaseFish", FISH);
    props::add(w, "Server.Items.FishBowl", FISH_BOWL);
}

#[allow(dead_code)]
fn _unused(_: StateData) {}
