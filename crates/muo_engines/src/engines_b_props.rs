//! The `[CommandProperty]` tables of the Plants, Khaldun, Doom, CannedEvil and Treasures of Tokuno classes for
//! `[props`, `[get` and `[set`.
//!
//! The `Creatures`, `Minions`, `RedSkulls`, `WhiteSkulls` of a `ChampionSpawn` and `Traps`/`Creatures` of a
//! `GauntletSpawner` are `List<T>`: `[CommandProperty]` shows what `ToString()` of the list says (the type name), as
//! `[props` does, and nothing opens. The `PigmentType` enums are plain numbers.

use muo_content::props::{Kind, PropDef, Value};
use muo_content::utilities::types::{Numeric, PropType};
use muo_primitives::{Rectangle2D, TimeSpan};
use muo_server::World;
use muo_server::ids::{EntityId, ItemId};
use muo_server::mobile::AccessLevel;

use crate::canned_evil::{info::ChampionSkullType, items as ce_items, spawn as ce_spawn};
use crate::doom::{gauntlet, lever};
use crate::khaldun::items as kh;
use crate::plants::{item as plant, seed};
use crate::tokuno::{artifacts as tok, pigments};

const GM: AccessLevel = AccessLevel::GameMaster;
const ADMIN: AccessLevel = AccessLevel::Administrator;

const INT: Kind = Kind::Plain(PropType::Numeric(Numeric::Int));
const USHORT: Kind = Kind::Plain(PropType::Numeric(Numeric::UShort));
const BOOL: Kind = Kind::Plain(PropType::Bool);
const STR: Kind = Kind::Plain(PropType::Str);
const ENTITY: Kind = Kind::Plain(PropType::Entity);
const POINT: Kind = Kind::Plain(PropType::IPoint3D);
const TIME_SPAN: Kind = Kind::TimeSpan;
const RECT: Kind = Kind::Rect2D;

const SECURE_LEVELS: &[(&str, i64)] = &[
    ("Owner", 0),
    ("CoOwners", 1),
    ("Friends", 2),
    ("Anyone", 3),
    ("Guild", 4),
];
const PLANT_STATUSES: &[(&str, i64)] = &[
    ("BowlOfDirt", 0),
    ("Seed", 1),
    ("Sapling", 2),
    ("Plant", 4),
    ("FullGrownPlant", 7),
    ("DecorativePlant", 10),
    ("DeadTwigs", 11),
    ("Stage3", 3),
    ("Stage5", 5),
    ("Stage6", 6),
    ("Stage8", 8),
    ("Stage9", 9),
];
const SKULL_TYPES: &[(&str, i64)] = &[
    ("Power", 0),
    ("Enlightenment", 1),
    ("Venom", 2),
    ("Pain", 3),
    ("Greed", 4),
    ("Death", 5),
    ("None", 6),
];
const SPAWN_TYPES: &[(&str, i64)] = &[
    ("Abyss", 0),
    ("Arachnid", 1),
    ("ColdBlood", 2),
    ("ForestLord", 3),
    ("VerminHorde", 4),
    ("UnholyTerror", 5),
    ("SleepingDragon", 6),
    ("Glade", 7),
    ("Corrupt", 8),
];
const GAUNTLET_STATES: &[(&str, i64)] = &[("InSequence", 0), ("InProgress", 1), ("Completed", 2)];

fn span(v: &Value) -> Result<TimeSpan, String> {
    match v {
        Value::TimeSpan(t) => Ok(*t),
        _ => Err("not a time span".to_owned()),
    }
}

fn rect(v: &Value) -> Result<Rectangle2D, String> {
    match v {
        Value::Rect(r) => Ok(*r),
        _ => Err("not a rectangle".to_owned()),
    }
}

fn item(e: EntityId) -> Option<ItemId> {
    match e {
        EntityId::Item(i) => Some(i),
        EntityId::Mobile(_) => None,
    }
}

fn ent_i(i: Option<ItemId>) -> Value {
    Value::Entity(i.map(EntityId::Item))
}

fn ent_m(m: Option<muo_server::ids::MobileId>) -> Value {
    Value::Entity(m.map(EntityId::Mobile))
}

fn want_item(v: &Value) -> Option<ItemId> {
    match v {
        Value::Entity(Some(EntityId::Item(i))) => Some(*i),
        _ => None,
    }
}

fn want_mobile(v: &Value) -> Option<muo_server::ids::MobileId> {
    match v {
        Value::Entity(Some(EntityId::Mobile(m))) => Some(*m),
        _ => None,
    }
}

fn time_text(t: TimeSpan) -> Value {
    Value::TimeSpan(t)
}

/// `get`/`set` of one field: the closure bodies are given item ids.
macro_rules! p {
    ($name:literal, $kind:expr, $get:expr, $set:expr) => {
        PropDef {
            name: $name,
            kind: $kind,
            read: GM,
            write: GM,
            get: |w, e| match item(e) {
                Some(i) => ($get)(w, i),
                None => Value::Null,
            },
            set: Some(|w, e, v| match item(e) {
                Some(i) => ($set)(w, i, v),
                None => Ok(()),
            }),
        }
    };
    ($name:literal, $kind:expr, $get:expr) => {
        PropDef {
            name: $name,
            kind: $kind,
            read: GM,
            write: GM,
            get: |w, e| match item(e) {
                Some(i) => ($get)(w, i),
                None => Value::Null,
            },
            set: None,
        }
    };
}

// ---- Plants ----

static SEED_PROPS: &[PropDef] = &[
    p!(
        "PlantType",
        Kind::Plain(PropType::Enum(seed::TYPE_NAMES)),
        |w: &World, i| Value::Enum(i64::from(seed::data(w, i).map_or(0, |d| d.plant_type))),
        |w: &mut World, i, v: Value| {
            if let Some(d) = w
                .item_mut(i)
                .and_then(|x| x.ext.get_mut::<seed::SeedData>())
            {
                d.plant_type = v.i32();
            }
            Ok(())
        }
    ),
    p!(
        "PlantHue",
        Kind::Plain(PropType::Enum(seed::HUE_NAMES)),
        |w: &World, i| Value::Enum(i64::from(seed::data(w, i).map_or(0, |d| d.plant_hue))),
        |w: &mut World, i, v: Value| {
            seed::set_plant_hue(w, i, v.i32());
            Ok(())
        }
    ),
    p!(
        "ShowType",
        BOOL,
        |w: &World, i| Value::Bool(seed::data(w, i).is_some_and(|d| d.show_type)),
        |w: &mut World, i, v: Value| {
            if let Some(d) = w
                .item_mut(i)
                .and_then(|x| x.ext.get_mut::<seed::SeedData>())
            {
                d.show_type = v.bool();
            }
            muo_server::item::ops::invalidate_properties(w, i);
            Ok(())
        }
    ),
];

/// `ValidGrowthLocation` without `FindBankNoCreate`'s `&mut`: the bank is the mobile's item on the bank layer.
fn valid_growth_location(w: &World, id: ItemId) -> bool {
    use muo_server::item::ops;
    let root = ops::root_parent(w, id);
    if ops::is_locked_down(w, id) && root.is_none() {
        return true;
    }
    let Some(EntityId::Mobile(owner)) = root else {
        return false;
    };
    let Some(m) = w.mobile(owner) else {
        return false;
    };
    if ops::is_child_of(w, id, m.backpack.map(EntityId::Item)) {
        return true;
    }
    let bank = m.items.iter().copied().find(|&i| {
        w.item(i)
            .is_some_and(|x| x.layer == muo_server::layer::Layer::Bank)
    });
    ops::is_child_of(w, id, bank.map(EntityId::Item))
}

static PLANT_ITEM_PROPS: &[PropDef] = &[
    p!(
        "Level",
        Kind::Plain(PropType::Enum(SECURE_LEVELS)),
        |w: &World, i| Value::Enum(i64::from(plant::data(w, i).map_or(0, |d| d.level))),
        |w: &mut World, i, v: Value| {
            plant::set_level(w, i, v.i32());
            Ok(())
        }
    ),
    p!(
        "PlantStatus",
        Kind::Plain(PropType::Enum(PLANT_STATUSES)),
        |w: &World, i| Value::Enum(i64::from(plant::plant_status(w, i))),
        |w: &mut World, i, v: Value| {
            plant::set_status(w, i, v.i32());
            Ok(())
        }
    ),
    p!(
        "PlantType",
        Kind::Plain(PropType::Enum(seed::TYPE_NAMES)),
        |w: &World, i| Value::Enum(i64::from(plant::plant_type(w, i))),
        |w: &mut World, i, v: Value| {
            plant::set_plant_type(w, i, v.i32());
            Ok(())
        }
    ),
    p!(
        "PlantHue",
        Kind::Plain(PropType::Enum(seed::HUE_NAMES)),
        |w: &World, i| Value::Enum(i64::from(plant::plant_hue(w, i))),
        |w: &mut World, i, v: Value| {
            plant::set_plant_hue(w, i, v.i32());
            Ok(())
        }
    ),
    p!(
        "ShowType",
        BOOL,
        |w: &World, i| Value::Bool(plant::data(w, i).is_some_and(|d| d.show_type)),
        |w: &mut World, i, v: Value| {
            plant::set_show_type(w, i, v.bool());
            Ok(())
        }
    ),
    p!("ValidGrowthLocation", BOOL, |w: &World, i| Value::Bool(
        valid_growth_location(w, i)
    )),
    p!("IsGrowable", BOOL, |w: &World, i| Value::Bool(
        plant::is_growable(w, i)
    )),
    p!("IsCrossable", BOOL, |w: &World, i| Value::Bool(
        plant::is_crossable(w, i)
    )),
    p!("Reproduces", BOOL, |w: &World, i| Value::Bool(
        plant::reproduces(w, i)
    )),
];

// ---- Khaldun ----

static PIT_PROPS: &[PropDef] = &[
    p!(
        "Active",
        BOOL,
        |w: &World, i| Value::Bool(kh::pit_data(w, i).is_some_and(|d| d.active)),
        |w: &mut World, i, v: Value| {
            kh::pit_edit(w, i, |d| d.active = v.bool());
            Ok(())
        }
    ),
    p!(
        "PointDest",
        POINT,
        |w: &World, i| Value::Point3D(
            kh::pit_data(w, i).map_or(Default::default(), |d| d.point_dest)
        ),
        |w: &mut World, i, v: Value| {
            kh::pit_edit(w, i, |d| d.point_dest = v.point());
            Ok(())
        }
    ),
    p!(
        "MapDest",
        Kind::Map,
        |w: &World, i| Value::Map(kh::pit_data(w, i).and_then(|d| d.map_dest)),
        |w: &mut World, i, v: Value| {
            kh::pit_edit(w, i, |d| d.map_dest = v.map());
            Ok(())
        }
    ),
];

static RAISABLE_PROPS: &[PropDef] = &[
    p!(
        "MaxElevation",
        INT,
        |w: &World, i| Value::Int(i64::from(
            kh::raisable_data(w, i).map_or(0, |d| d.max_elevation)
        )),
        |w: &mut World, i, v: Value| {
            kh::set_max_elevation(w, i, v.i32());
            Ok(())
        }
    ),
    p!(
        "MoveSound",
        INT,
        |w: &World, i| Value::Int(i64::from(
            kh::raisable_data(w, i).map_or(0, |d| d.move_sound)
        )),
        |w: &mut World, i, v: Value| {
            kh::raisable_edit(w, i, |d| d.move_sound = v.i32());
            Ok(())
        }
    ),
    p!(
        "StopSound",
        INT,
        |w: &World, i| Value::Int(i64::from(
            kh::raisable_data(w, i).map_or(0, |d| d.stop_sound)
        )),
        |w: &mut World, i, v: Value| {
            kh::raisable_edit(w, i, |d| d.stop_sound = v.i32());
            Ok(())
        }
    ),
    p!(
        "CloseDelay",
        TIME_SPAN,
        |w: &World, i| time_text(kh::raisable_data(w, i).map_or(TimeSpan::ZERO, |d| d.close_delay)),
        |w: &mut World, i, v: Value| {
            let t = span(&v)?;
            kh::raisable_edit(w, i, |d| d.close_delay = t);
            Ok(())
        }
    ),
    p!("Elevation", INT, |w: &World, i| Value::Int(i64::from(
        kh::raisable_data(w, i).map_or(0, |d| d.elevation)
    ))),
];

static RAISE_SWITCH_PROPS: &[PropDef] = &[p!(
    "RaisableItem",
    ENTITY,
    |w: &World, i| ent_i(kh::switch_raisable(w, i)),
    |w: &mut World, i, v: Value| {
        kh::set_raisable_item(w, i, want_item(&v).filter(|r| kh::is_raisable(w, *r)));
        Ok(())
    }
)];

// ---- Doom ----

static GAUNTLET_PROPS: &[PropDef] = &[
    p!(
        "RegionBounds",
        RECT,
        |w: &World, i| Value::Rect(
            gauntlet::data(w, i).map_or(Rectangle2D::EMPTY, |d| d.region_bounds)
        ),
        |w: &mut World, i, v: Value| {
            let r = rect(&v)?;
            if let Some(d) = gauntlet::data_mut(w, i) {
                d.region_bounds = r;
            }
            Ok(())
        }
    ),
    p!("Traps", Kind::Text, |w: &World, i| gauntlet::data(w, i)
        .map_or(Value::Null, |_| Value::Text(
            "System.Collections.Generic.List`1[Server.Items.BaseTrap]".into()
        ))),
    p!("Creatures", Kind::Text, |w: &World, i| gauntlet::data(w, i)
        .map_or(Value::Null, |_| Value::Text(
            "System.Collections.Generic.List`1[Server.Mobile]".into()
        ))),
    p!(
        "TypeName",
        STR,
        |w: &World, i| gauntlet::data(w, i)
            .and_then(|d| d.type_name.clone())
            .map_or(Value::Null, Value::Str),
        |w: &mut World, i, v: Value| {
            if let Some(d) = gauntlet::data_mut(w, i) {
                d.type_name = v.string();
            }
            Ok(())
        }
    ),
    p!(
        "Door",
        ENTITY,
        |w: &World, i| ent_i(gauntlet::data(w, i).and_then(|d| d.door)),
        |w: &mut World, i, v: Value| {
            let door = want_item(&v).filter(|d| muo_items::construction::doors::is_door(w, *d));
            if let Some(d) = gauntlet::data_mut(w, i) {
                d.door = door;
            }
            Ok(())
        }
    ),
    p!(
        "Addon",
        ENTITY,
        |w: &World, i| ent_i(gauntlet::data(w, i).and_then(|d| d.addon)),
        |w: &mut World, i, v: Value| {
            if let Some(d) = gauntlet::data_mut(w, i) {
                d.addon = want_item(&v);
            }
            Ok(())
        }
    ),
    p!(
        "Sequence",
        ENTITY,
        |w: &World, i| ent_i(gauntlet::data(w, i).and_then(|d| d.sequence)),
        |w: &mut World, i, v: Value| {
            let s = want_item(&v).filter(|s| {
                w.item(*s)
                    .is_some_and(|x| x.ty.is(&gauntlet::GAUNTLET_SPAWNER))
            });
            if let Some(d) = gauntlet::data_mut(w, i) {
                d.sequence = s;
            }
            Ok(())
        }
    ),
    p!("HasCompleted", BOOL, |w: &World, i| Value::Bool(
        gauntlet::has_completed(w, i)
    )),
    p!(
        "State",
        Kind::Plain(PropType::Enum(GAUNTLET_STATES)),
        |w: &World, i| Value::Enum(gauntlet::data(w, i).map_or(0, |d| d.state as i64)),
        |w: &mut World, i, v: Value| {
            gauntlet::set_state(w, i, gauntlet::State::from_i32(v.i32()));
            Ok(())
        }
    ),
];

static LEVER_PUZZLE_PROPS: &[PropDef] = &[
    p!(
        "MyKey",
        USHORT,
        |w: &World, i| Value::Int(lever::data(w, i).map_or(0, |d| i64::from(d.my_key))),
        |w: &mut World, i, v: Value| {
            if let Some(d) = lever::data_mut(w, i) {
                d.my_key = v.i64() as u16;
            }
            Ok(())
        }
    ),
    p!(
        "TheirKey",
        USHORT,
        |w: &World, i| Value::Int(lever::data(w, i).map_or(0, |d| i64::from(d.their_key))),
        |w: &mut World, i, v: Value| {
            if let Some(d) = lever::data_mut(w, i) {
                d.their_key = v.i64() as u16;
            }
            Ok(())
        }
    ),
    p!(
        "Enabled",
        BOOL,
        |w: &World, i| Value::Bool(lever::enabled(w, i)),
        |w: &mut World, i, v: Value| {
            if let Some(d) = lever::data_mut(w, i) {
                d.enabled = v.bool();
            }
            Ok(())
        }
    ),
];

// ---- CannedEvil ----

static SKULL_PROPS: &[PropDef] = &[p!(
    "Type",
    Kind::Plain(PropType::Enum(SKULL_TYPES)),
    |w: &World, i| Value::Enum(ce_items::skull_type(w, i) as i64),
    |w: &mut World, i, v: Value| {
        if let Some(d) = w
            .item_mut(i)
            .and_then(|x| x.ext.get_mut::<ce_items::SkullData>())
        {
            d.ty = ChampionSkullType::from_i32(v.i32());
        }
        Ok(())
    }
)];

static BRAZIER_PROPS: &[PropDef] = &[
    p!(
        "Type",
        Kind::Plain(PropType::Enum(SKULL_TYPES)),
        |w: &World, i| Value::Enum(ce_items::brazier_data(w, i).map_or(6, |d| d.ty as i64)),
        |w: &mut World, i, v: Value| {
            if let Some(d) = w
                .item_mut(i)
                .and_then(|x| x.ext.get_mut::<ce_items::BrazierData>())
            {
                d.ty = ChampionSkullType::from_i32(v.i32());
            }
            Ok(())
        }
    ),
    p!(
        "Platform",
        ENTITY,
        |w: &World, i| ent_i(ce_items::brazier_data(w, i).and_then(|d| d.platform)),
        |w: &mut World, i, v: Value| {
            if let Some(d) = w
                .item_mut(i)
                .and_then(|x| x.ext.get_mut::<ce_items::BrazierData>())
            {
                d.platform = want_item(&v);
            }
            muo_server::item::ops::invalidate_properties(w, i);
            Ok(())
        }
    ),
    p!(
        "Skull",
        ENTITY,
        |w: &World, i| ent_i(ce_items::brazier_data(w, i).and_then(|d| d.skull)),
        |w: &mut World, i, v: Value| {
            ce_items::set_skull(w, i, want_item(&v));
            Ok(())
        }
    ),
];

macro_rules! spawn_bool {
    ($name:literal, $f:ident) => {
        p!(
            $name,
            BOOL,
            |w: &World, i| Value::Bool(ce_spawn::data(w, i).is_some_and(|d| d.$f)),
            |w: &mut World, i, v: Value| {
                if let Some(d) = ce_spawn::data_mut(w, i) {
                    d.$f = v.bool();
                }
                Ok(())
            }
        )
    };
}

macro_rules! spawn_span {
    ($name:literal, $f:ident) => {
        p!(
            $name,
            TIME_SPAN,
            |w: &World, i| time_text(ce_spawn::data(w, i).map_or(TimeSpan::ZERO, |d| d.$f)),
            |w: &mut World, i, v: Value| {
                let t = span(&v)?;
                if let Some(d) = ce_spawn::data_mut(w, i) {
                    d.$f = t;
                }
                Ok(())
            }
        )
    };
}

macro_rules! spawn_date {
    ($name:literal, $f:ident) => {
        p!($name, Kind::DateTime, |w: &World, i| Value::DateTime(
            ce_spawn::data(w, i).map_or(Default::default(), |d| d.$f)
        ))
    };
}

/// A `List<T>` member: `ToString()` of the list is its type name; the list itself is never null.
macro_rules! spawn_list {
    ($name:literal, $f:ident, $t:literal) => {
        p!($name, Kind::Text, |w: &World, i| ce_spawn::data(w, i)
            .map_or(Value::Null, |_| Value::Text(
                concat!("System.Collections.Generic.List`1[", $t, "]").into()
            )))
    };
}

static CHAMPION_SPAWN_PROPS: &[PropDef] = &[
    spawn_bool!("ActivatedByProximity", activated_by_proximity),
    spawn_date!("NextProximityTime", next_proximity_time),
    spawn_bool!("ActivatedByValor", activated_by_valor),
    spawn_bool!("ConfinedRoaming", confined_roaming),
    p!(
        "Idol",
        ENTITY,
        |w: &World, i| ent_i(ce_spawn::data(w, i).and_then(|d| d.idol)),
        |w: &mut World, i, v: Value| {
            let idol = want_item(&v).filter(|x| {
                w.item(*x)
                    .is_some_and(|it| it.ty.is(&ce_spawn::IDOL_OF_THE_CHAMPION))
            });
            if let Some(d) = ce_spawn::data_mut(w, i) {
                d.idol = idol;
            }
            Ok(())
        }
    ),
    spawn_bool!("HasBeenAdvanced", has_been_advanced),
    spawn_bool!("RandomizeType", randomize_type),
    p!(
        "Type",
        Kind::Plain(PropType::Enum(SPAWN_TYPES)),
        |w: &World, i| Value::Enum(i64::from(ce_spawn::data(w, i).map_or(0, |d| d.ty))),
        |w: &mut World, i, v: Value| {
            ce_spawn::set_type(w, i, v.i32());
            Ok(())
        }
    ),
    spawn_list!("Creatures", creatures, "Server.Mobile"),
    spawn_list!("RedSkulls", red_skulls, "Server.Item"),
    spawn_list!("WhiteSkulls", white_skulls, "Server.Item"),
    p!(
        "Platform",
        ENTITY,
        |w: &World, i| ent_i(ce_spawn::data(w, i).and_then(|d| d.platform)),
        |w: &mut World, i, v: Value| {
            let p = want_item(&v).filter(|x| {
                w.item(*x)
                    .is_some_and(|it| it.ty.is(&ce_items::CHAMPION_PLATFORM))
            });
            if let Some(d) = ce_spawn::data_mut(w, i) {
                d.platform = p;
            }
            Ok(())
        }
    ),
    p!(
        "Altar",
        ENTITY,
        |w: &World, i| ent_i(ce_spawn::data(w, i).and_then(|d| d.altar)),
        |w: &mut World, i, v: Value| {
            let a = want_item(&v).filter(|x| {
                w.item(*x)
                    .is_some_and(|it| it.ty.is(&ce_items::CHAMPION_ALTAR))
            });
            if let Some(d) = ce_spawn::data_mut(w, i) {
                d.altar = a;
            }
            Ok(())
        }
    ),
    spawn_span!("ExpireDelay", expire_delay),
    spawn_date!("ExpireTime", expire_time),
    p!(
        "Champion",
        ENTITY,
        |w: &World, i| ent_m(ce_spawn::data(w, i).and_then(|d| d.champion)),
        |w: &mut World, i, v: Value| {
            if let Some(d) = ce_spawn::data_mut(w, i) {
                d.champion = want_mobile(&v);
            }
            Ok(())
        }
    ),
    spawn_span!("RestartDelay", restart_delay),
    spawn_date!("RestartTime", restart_time),
    spawn_list!("Minions", minions, "Server.Mobile"),
    PropDef {
        name: "MaxLevel",
        kind: INT,
        read: GM,
        write: ADMIN,
        get: |w, e| {
            Value::Int(
                item(e)
                    .and_then(|i| ce_spawn::data(w, i))
                    .map_or(0, |d| i64::from(d.max_level)),
            )
        },
        set: Some(|w, e, v| {
            if let Some(d) = item(e).and_then(|i| ce_spawn::data_mut(w, i)) {
                d.max_level = v.i32().clamp(0, 18);
            }
            Ok(())
        }),
    },
    p!(
        "SpawnArea",
        RECT,
        |w: &World, i| Value::Rect(
            ce_spawn::data(w, i).map_or(Rectangle2D::EMPTY, |d| d.spawn_area)
        ),
        |w: &mut World, i, v: Value| {
            let r = rect(&v)?;
            ce_spawn::set_spawn_area(w, i, r);
            Ok(())
        }
    ),
    p!(
        "Kills",
        INT,
        |w: &World, i| Value::Int(ce_spawn::data(w, i).map_or(0, |d| i64::from(d.kills))),
        |w: &mut World, i, v: Value| {
            ce_spawn::set_kills(w, i, v.i32());
            Ok(())
        }
    ),
    p!(
        "Level",
        INT,
        |w: &World, i| Value::Int(i64::from(ce_spawn::level(w, i))),
        |w: &mut World, i, v: Value| {
            ce_spawn::set_level(w, i, v.i32());
            Ok(())
        }
    ),
    p!(
        "Active",
        BOOL,
        |w: &World, i| Value::Bool(ce_spawn::data(w, i).is_some_and(|d| d.active)),
        |w: &mut World, i, v: Value| {
            ce_spawn::set_active(w, i, v.bool());
            Ok(())
        }
    ),
    spawn_bool!("ReadyToActivate", ready_to_activate),
    p!(
        "EjectLocation",
        POINT,
        |w: &World, i| Value::Point3D(
            ce_spawn::data(w, i).map_or(Default::default(), |d| d.eject_location)
        ),
        |w: &mut World, i, v: Value| {
            if let Some(d) = ce_spawn::data_mut(w, i) {
                d.eject_location = v.point();
            }
            Ok(())
        }
    ),
    p!(
        "EjectMap",
        Kind::Map,
        |w: &World, i| Value::Map(ce_spawn::data(w, i).and_then(|d| d.eject_map)),
        |w: &mut World, i, v: Value| {
            if let Some(d) = ce_spawn::data_mut(w, i) {
                d.eject_map = v.map();
            }
            Ok(())
        }
    ),
    p!("MaxKills", INT, |w: &World, i| Value::Int(i64::from(
        ce_spawn::max_kills(w, i)
    ))),
];

// ---- Treasures of Tokuno ----

static PIGMENT_PROPS: &[PropDef] = &[p!(
    "UsesRemaining",
    INT,
    |w: &World, i| Value::Int(i64::from(pigments::uses_remaining(w, i))),
    |w: &mut World, i, v: Value| {
        pigments::set_uses_remaining(w, i, v.i32());
        Ok(())
    }
)];

static PIGMENTS_OF_TOKUNO_PROPS: &[PropDef] = &[p!(
    "Type",
    INT,
    |w: &World, i| Value::Int(i64::from(pigments::pigment_type(w, i))),
    |w: &mut World, i, v: Value| {
        pigments::set_type(w, i, v.i32());
        Ok(())
    }
)];

static LESSER_PIGMENTS_PROPS: &[PropDef] = &[p!(
    "Type",
    INT,
    |w: &World, i| Value::Int(i64::from(pigments::pigment_type(w, i))),
    |w: &mut World, i, v: Value| {
        pigments::set_lesser_type(w, i, v.i32());
        Ok(())
    }
)];

static URN_PROPS: &[PropDef] = &[p!(
    "UrnName",
    STR,
    |w: &World, i| tok::name(w, i).map_or(Value::Null, Value::Str),
    |w: &mut World, i, v: Value| {
        tok::set_name(w, i, v.string());
        Ok(())
    }
)];

static SWORDS_PROPS: &[PropDef] = &[p!(
    "SwordsName",
    STR,
    |w: &World, i| tok::name(w, i).map_or(Value::Null, Value::Str),
    |w: &mut World, i, v: Value| {
        tok::set_name(w, i, v.string());
        Ok(())
    }
)];

pub fn configure(w: &mut World) {
    use muo_content::props::add;
    add(w, "Server.Engines.Plants.Seed", SEED_PROPS);
    add(w, "Server.Engines.Plants.PlantItem", PLANT_ITEM_PROPS);
    add(w, "Server.Items.KhaldunPitTeleporter", PIT_PROPS);
    add(w, "Server.Items.RaisableItem", RAISABLE_PROPS);
    add(w, "Server.Items.RaiseSwitch", RAISE_SWITCH_PROPS);
    add(w, "Server.Engines.Doom.GauntletSpawner", GAUNTLET_PROPS);
    add(
        w,
        "Server.Engines.Doom.LeverPuzzleController",
        LEVER_PUZZLE_PROPS,
    );
    add(w, "Server.Items.ChampionSkull", SKULL_PROPS);
    add(
        w,
        "Server.Engines.CannedEvil.ChampionSkullBrazier",
        BRAZIER_PROPS,
    );
    add(
        w,
        "Server.Engines.CannedEvil.ChampionSpawn",
        CHAMPION_SPAWN_PROPS,
    );
    add(w, "Server.Items.BasePigmentsOfTokuno", PIGMENT_PROPS);
    add(w, "Server.Items.PigmentsOfTokuno", PIGMENTS_OF_TOKUNO_PROPS);
    add(
        w,
        "Server.Items.LesserPigmentsOfTokuno",
        LESSER_PIGMENTS_PROPS,
    );
    add(w, "Server.Items.AncientUrn", URN_PROPS);
    add(w, "Server.Items.HonorableSwords", SWORDS_PROPS);
    configure_statues(w);
}

// ---- Character Statue Maker ----

use crate::rewards::statue as st;

const STATUE_TYPES: &[(&str, i64)] = &[("Marble", 0), ("Jade", 1), ("Bronze", 2)];
const STATUE_POSES: &[(&str, i64)] = &[
    ("Ready", 0),
    ("Casting", 1),
    ("Salute", 2),
    ("AllPraiseMe", 3),
    ("Fighting", 4),
    ("HandsOnHips", 5),
];
const STATUE_MATERIALS: &[(&str, i64)] =
    &[("Antique", 0), ("Dark", 1), ("Medium", 2), ("Light", 3)];

fn mobile(e: EntityId) -> Option<muo_server::ids::MobileId> {
    match e {
        EntityId::Mobile(m) => Some(m),
        EntityId::Item(_) => None,
    }
}

macro_rules! sp {
    ($name:literal, $kind:expr, $get:expr, $set:expr) => {
        PropDef {
            name: $name,
            kind: $kind,
            read: GM,
            write: GM,
            get: |w, e| match mobile(e) {
                Some(m) => ($get)(w, m),
                None => Value::Null,
            },
            set: Some(|w, e, v| match mobile(e) {
                Some(m) => ($set)(w, m, v),
                None => Ok(()),
            }),
        }
    };
    ($name:literal, $kind:expr, $get:expr) => {
        PropDef {
            name: $name,
            kind: $kind,
            read: GM,
            write: GM,
            get: |w, e| match mobile(e) {
                Some(m) => ($get)(w, m),
                None => Value::Null,
            },
            set: None,
        }
    };
}

static CHARACTER_STATUE_PROPS: &[PropDef] = &[
    sp!(
        "StatueType",
        Kind::Plain(PropType::Enum(STATUE_TYPES)),
        |w: &World, m| Value::Enum(st::statue(w, m).map_or(0, |d| i64::from(d.statue_type))),
        |w: &mut World, m, v: Value| {
            st::set_statue_type(w, m, v.i32());
            Ok(())
        }
    ),
    sp!(
        "Pose",
        Kind::Plain(PropType::Enum(STATUE_POSES)),
        |w: &World, m| Value::Enum(st::statue(w, m).map_or(0, |d| i64::from(d.pose))),
        |w: &mut World, m, v: Value| {
            st::set_pose(w, m, v.i32());
            Ok(())
        }
    ),
    sp!(
        "Material",
        Kind::Plain(PropType::Enum(STATUE_MATERIALS)),
        |w: &World, m| Value::Enum(st::statue(w, m).map_or(0, |d| i64::from(d.material))),
        |w: &mut World, m, v: Value| {
            st::set_material(w, m, v.i32());
            Ok(())
        }
    ),
    sp!(
        "SculptedBy",
        STR,
        |w: &World, m| st::statue(w, m)
            .and_then(|d| d.sculpted_by.clone())
            .map_or(Value::Null, Value::Str),
        |w: &mut World, m, v: Value| {
            if let Some(d) = st::statue_mut(w, m) {
                d.sculpted_by = v.string();
            }
            muo_server::mobile::part2::properties::invalidate_properties(w, m);
            Ok(())
        }
    ),
    sp!(
        "SculptedOn",
        Kind::DateTime,
        |w: &World, m| Value::DateTime(
            st::statue(w, m).map_or(Default::default(), |d| d.sculpted_on)
        )
    ),
    sp!(
        "Plinth",
        ENTITY,
        |w: &World, m| ent_i(st::statue(w, m).and_then(|d| d.plinth)),
        |w: &mut World, m, v: Value| {
            let p = want_item(&v).filter(|x| {
                w.item(*x)
                    .is_some_and(|i| i.ty.is(&st::CHARACTER_STATUE_PLINTH))
            });
            if let Some(d) = st::statue_mut(w, m) {
                d.plinth = p;
            }
            Ok(())
        }
    ),
    sp!(
        "IsRewardItem",
        BOOL,
        |w: &World, m| Value::Bool(st::statue(w, m).is_some_and(|d| d.is_reward_item)),
        |w: &mut World, m, v: Value| {
            if let Some(d) = st::statue_mut(w, m) {
                d.is_reward_item = v.bool();
            }
            Ok(())
        }
    ),
];

static STATUE_MAKER_PROPS: &[PropDef] = &[
    p!(
        "IsRewardItem",
        BOOL,
        |w: &World, i| Value::Bool(st::maker_reward(w, i).unwrap_or(false)),
        |w: &mut World, i, v: Value| {
            st::set_maker_reward(w, i, v.bool());
            Ok(())
        }
    ),
    p!(
        "StatueType",
        Kind::Plain(PropType::Enum(STATUE_TYPES)),
        |w: &World, i| Value::Enum(st::maker_type(w, i)),
        |w: &mut World, i, v: Value| {
            st::set_maker_type(w, i, v.i32());
            Ok(())
        }
    ),
];

static STATUE_DEED_PROPS: &[PropDef] = &[p!(
    "StatueType",
    Kind::Plain(PropType::Enum(STATUE_TYPES)),
    |w: &World, i| Value::Enum(i64::from(st::deed_type(w, i))),
    |w: &mut World, i, v: Value| {
        st::set_deed_type_pub(w, i, v.i32());
        Ok(())
    }
)];

pub fn configure_statues(w: &mut World) {
    use muo_content::props::add;
    add(w, "Server.Mobiles.CharacterStatue", CHARACTER_STATUE_PROPS);
    add(w, "Server.Items.CharacterStatueMaker", STATUE_MAKER_PROPS);
    add(w, "Server.Mobiles.CharacterStatueDeed", STATUE_DEED_PROPS);
}
