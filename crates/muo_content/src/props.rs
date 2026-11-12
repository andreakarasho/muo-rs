//! The property table behind `[props`, `[get`, `[set`, `[increase` and the generic commands' `where` / `order by`.
//!
//! A class lists its `[CommandProperty]` members: the members that matter as a
//! `static [PropDef]` and adds it with [`add`] (from a `Configure` phase function), keyed by the class's full name.
//! A class's properties are its own table plus those of every ancestor (`ItemType::parent` / `MobileType::parent`),
//! a more derived entry replacing an inherited one of the same name, as `Type.GetProperties` does for an `override`.
//!
//! A [`PropDef`] is a name, a [`Kind`] (what the value parses as, the existing `utilities::types::PropType`), the
//! two access levels of the attribute, a getter and an optional setter. No setter means "get only" or
//! `readOnly: true`. The getter and the setter go through the same functions the game code does
//! (`item::ops::set_amount`, `mobile::...::set_hits`), so their side effects (deltas, stat timers) happen.
//!
//! `Value` is the dynamically typed value a property returns. A property whose value is an item or a mobile returns
//! `Value::Entity`, which is what makes dot notation (`Backpack.Name`) walk on.
//!
//! An object property (`TextDefinition`, a `[PropertyObject]`) is a [`Kind::TextDef`] / [`Kind::Object`] whose
//! sub-properties are a table over the same owner; a value nobody wrote a parser for is a `Kind::Text` getter.

use indexmap::IndexMap;
use muo_primitives::dotnet::IgnoreCase;
use std::cmp::Ordering;

use muo_io::TextDefinition;
use muo_primitives::{DateTime, Point3D, Rectangle2D, Rectangle3D, Serial, TimeSpan};
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::map::MapId;
use muo_server::mobile::AccessLevel;
use muo_server::poison::PoisonId;
use muo_server::world::World;

use crate::systems::utilities::types::{ParseContext, Parsed, PropType, try_parse};

/// `SkillName`'s members.
pub const SKILL_NAMES: &[(&str, i64)] = &[
    ("Alchemy", 0),
    ("Anatomy", 1),
    ("AnimalLore", 2),
    ("ItemID", 3),
    ("ArmsLore", 4),
    ("Parry", 5),
    ("Begging", 6),
    ("Blacksmith", 7),
    ("Fletching", 8),
    ("Peacemaking", 9),
    ("Camping", 10),
    ("Carpentry", 11),
    ("Cartography", 12),
    ("Cooking", 13),
    ("DetectHidden", 14),
    ("Discordance", 15),
    ("EvalInt", 16),
    ("Healing", 17),
    ("Fishing", 18),
    ("Forensics", 19),
    ("Herding", 20),
    ("Hiding", 21),
    ("Provocation", 22),
    ("Inscribe", 23),
    ("Lockpicking", 24),
    ("Magery", 25),
    ("MagicResist", 26),
    ("Tactics", 27),
    ("Snooping", 28),
    ("Musicianship", 29),
    ("Poisoning", 30),
    ("Archery", 31),
    ("SpiritSpeak", 32),
    ("Stealing", 33),
    ("Tailoring", 34),
    ("AnimalTaming", 35),
    ("TasteID", 36),
    ("Tinkering", 37),
    ("Tracking", 38),
    ("Veterinary", 39),
    ("Swords", 40),
    ("Macing", 41),
    ("Fencing", 42),
    ("Wrestling", 43),
    ("Lumberjacking", 44),
    ("Mining", 45),
    ("Meditation", 46),
    ("Stealth", 47),
    ("RemoveTrap", 48),
    ("Necromancy", 49),
    ("Focus", 50),
    ("Chivalry", 51),
    ("Bushido", 52),
    ("Ninjitsu", 53),
    ("Spellweaving", 54),
    ("Mysticism", 55),
    ("Imbuing", 56),
    ("Throwing", 57),
];

/// Which object of its owner the sub-object whose getter or setter runs is (`usize::MAX` none): a props gump over the
/// n-th `PlayerBBMessage` of a board sets it before each call. Only tables reachable through such a gump read it.
pub struct SubKey(pub usize);

impl Default for SubKey {
    fn default() -> Self {
        SubKey(usize::MAX)
    }
}

pub fn sub_key(w: &World) -> usize {
    w.resources.get::<SubKey>().map_or(usize::MAX, |k| k.0)
}

pub fn set_sub_key(w: &mut World, k: usize) {
    w.resources.get_mut::<SubKey>().0 = k;
}

/// What a getter returns and a setter takes.
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Null,
    Int(i64),
    UInt(u64),
    Float(f64),
    Bool(bool),
    Str(String),
    /// An enum member's number (the name comes from the property's [`Kind`]).
    Enum(i64),
    Entity(Option<EntityId>),
    Point3D(Point3D),
    Map(Option<MapId>),
    /// A value that is only shown (an object without a parser).
    Text(String),
    TimeSpan(TimeSpan),
    DateTime(DateTime),
    Rect(Rectangle2D),
    Rect3(Rectangle3D),
    /// A `Poison` (`None` is null).
    Poison(Option<PoisonId>),
    /// A non-null `TextDefinition` (a null one is `Null`).
    TextDef(TextDefinition),
}

impl Value {
    pub fn i64(&self) -> i64 {
        match self {
            Value::Int(v) | Value::Enum(v) => *v,
            Value::UInt(v) => *v as i64,
            Value::Float(v) => *v as i64,
            Value::Bool(b) => i64::from(*b),
            _ => 0,
        }
    }
    pub fn i32(&self) -> i32 {
        self.i64() as i32
    }
    pub fn f64(&self) -> f64 {
        match self {
            Value::Float(v) => *v,
            other => other.i64() as f64,
        }
    }
    /// Null: `Null`, or an entity or map that is not there.
    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null | Value::Entity(None) | Value::Map(None))
    }
    pub fn bool(&self) -> bool {
        matches!(self, Value::Bool(true))
    }
    pub fn string(&self) -> Option<String> {
        match self {
            Value::Str(s) | Value::Text(s) => Some(s.clone()),
            _ => None,
        }
    }
    pub fn entity(&self) -> Option<EntityId> {
        match self {
            Value::Entity(e) => *e,
            _ => None,
        }
    }
    pub fn mobile(&self) -> Option<MobileId> {
        match self {
            Value::Entity(Some(EntityId::Mobile(m))) => Some(*m),
            _ => None,
        }
    }
    pub fn map(&self) -> Option<MapId> {
        match self {
            Value::Map(m) => *m,
            _ => None,
        }
    }
    pub fn point(&self) -> Point3D {
        match self {
            Value::Point3D(p) => *p,
            _ => Point3D::default(),
        }
    }

    /// `IComparable.CompareTo` where the type has one (`None` when it has not: only `==` and `!=` work).
    /// `PropertyExpressions.TryCompare` with sign +1: `null.CompareTo(null) = 0`, `real.CompareTo(null) = -1`,
    /// `null.CompareTo(real) = +1`: a null orders after everything else (a descending sort flips that).
    pub fn compare(&self, other: &Value) -> Option<Ordering> {
        use Value::*;
        // A null entity or map is null.
        match (self.is_null(), other.is_null()) {
            (true, true) => return Some(Ordering::Equal),
            (true, false) => return Some(Ordering::Greater),
            (false, true) => return Some(Ordering::Less),
            (false, false) => {}
        }
        Some(match (self, other) {
            (Int(a), Int(b)) | (Enum(a), Enum(b)) => a.cmp(b),
            (UInt(a), UInt(b)) => a.cmp(b),
            (Int(a), UInt(b)) => i128::from(*a).cmp(&i128::from(*b)),
            (UInt(a), Int(b)) => i128::from(*a).cmp(&i128::from(*b)),
            (Float(a), Float(b)) => a.partial_cmp(b)?,
            (Bool(a), Bool(b)) => a.cmp(b),
            (Str(a), Str(b)) | (Text(a), Text(b)) => muo_primitives::dotnet::string_compare(a, b), // string.CompareTo: culture
            (Map(a), Map(b)) => a.map(|m| m.0).cmp(&b.map(|m| m.0)),
            // Item and Mobile are IComparable by serial; Point3D by x, then y, then z.
            (Entity(a), Entity(b)) => a.map(|e| e.serial()).cmp(&b.map(|e| e.serial())),
            (Point3D(a), Point3D(b)) => (a.x, a.y, a.z).cmp(&(b.x, b.y, b.z)),
            (Value::TimeSpan(a), Value::TimeSpan(b)) => a.cmp(b),
            (Value::DateTime(a), Value::DateTime(b)) => a.cmp(b),
            // Poison, Rectangle2D and TextDefinition have no CompareTo: only `==` and `!=` (by value) work.
            _ => return None,
        })
    }
}

/// What kind of value a property holds (how `[set` parses text for it).
#[derive(Clone)]
pub enum Kind {
    Plain(PropType),
    /// A `Map`: parsed by the world's map names.
    Map,
    /// Shown but not parseable (an object nobody wrote a parser for).
    Text,
    TimeSpan,
    DateTime,
    /// `Rectangle2D`: equality by value, no `CompareTo`.
    Rect2D,
    /// `Rectangle3D`: equality by value, no `CompareTo`.
    Rect3D,
    /// `Poison`: parsed by index or name.
    Poison,
    /// `TextDefinition`; the sub-properties are `Number` and `String`, built per property by [`text_def_prop!`]
    /// because their getters read the parent value.
    TextDef(&'static [PropDef]),
    /// A `[PropertyObject]` or other object with sub-properties; the first field is the class name (`AosAttributes`).
    /// A sub-property's getter and setter take the same owner entity as the parent's, so `Aos*.WeaponDamage` is one table entry that reads the bag from the item.
    /// The parent's getter returns `Null` when the object is null, else `Text` (what `[props` shows).
    Object(&'static str, &'static [PropDef]),
    /// `Nullable<T>`: the underlying kind; an unset value is `Null`, relations on it are lifted.
    Nullable(&'static Kind),
    /// `[CustomEnum(names)]`: the names the gump offers, then the members the text parses against.
    CustomEnum(&'static [&'static str], &'static [(&'static str, i64)]),
}

pub type GetFn = fn(&mut World, EntityId) -> Value;
pub type SetFn = fn(&mut World, EntityId, Value) -> Result<(), String>;

/// One `[CommandProperty]` member.
pub struct PropDef {
    pub name: &'static str,
    pub kind: Kind,
    pub read: AccessLevel,
    pub write: AccessLevel,
    pub get: GetFn,
    pub set: Option<SetFn>,
}

/// A `TextDefinition` property: `Number` and `String` are its read-only sub-properties. `$get` is a closure
/// `|&World, EntityId| -> Option<TextDefinition>` (`None` is null), `$set` an `Option<SetFn>` taking
/// `Value::TextDef` or `Value::Null`.
#[macro_export]
macro_rules! text_def_prop {
    ($name:literal, $read:expr, $write:expr, $get:expr, $set:expr) => {
        $crate::props::PropDef {
            name: $name,
            kind: $crate::props::Kind::TextDef(&[
                $crate::props::PropDef {
                    name: "Number",
                    kind: $crate::props::Kind::Plain($crate::utilities::types::PropType::Numeric(
                        $crate::utilities::types::Numeric::Int,
                    )),
                    read: $read,
                    write: $write,
                    get: |w, e| match ($get)(w, e) {
                        Some(t) => $crate::props::Value::Int(i64::from(t.number)),
                        None => $crate::props::Value::Null,
                    },
                    set: None,
                },
                $crate::props::PropDef {
                    name: "String",
                    kind: $crate::props::Kind::Plain($crate::utilities::types::PropType::Str),
                    read: $read,
                    write: $write,
                    get: |w, e| match ($get)(w, e) {
                        Some(t) => t
                            .string
                            .map_or($crate::props::Value::Null, $crate::props::Value::Str),
                        None => $crate::props::Value::Null,
                    },
                    set: None,
                },
            ]),
            read: $read,
            write: $write,
            get: |w, e| match ($get)(w, e) {
                Some(t) => $crate::props::Value::TextDef(t),
                None => $crate::props::Value::Null,
            },
            set: $set,
        }
    };
}

/// A property over a data struct kept in `Item::ext`: `$data(w, item) -> Option<&T>` reads, `$data_mut` writes;
/// the setter calls `InvalidateProperties` when `$inval`. An entity that is not an item is `Null` / an error.
#[macro_export]
macro_rules! ext_field {
    ($data:path, $data_mut:path, $name:literal, $kind:expr, $read:expr, $write:expr,
     |$gd:ident| $get:expr, |$sd:ident, $sv:ident| $set:expr, $inval:expr) => {
        $crate::props::PropDef {
            name: $name,
            kind: $kind,
            read: $read,
            write: $write,
            get: |w: &mut muo_server::world::World, e: muo_server::ids::EntityId| match e {
                muo_server::ids::EntityId::Item(i) => {
                    $data(w, i).map_or($crate::props::Value::Null, |$gd| $get)
                }
                _ => $crate::props::Value::Null,
            },
            set: Some(
                |w: &mut muo_server::world::World,
                 e: muo_server::ids::EntityId,
                 v: $crate::props::Value| {
                    let muo_server::ids::EntityId::Item(i) = e else {
                        return Err("That is not an item.".to_string());
                    };
                    if let Some($sd) = $data_mut(w, i) {
                        let $sv = &v;
                        $set;
                    }
                    if $inval {
                        muo_server::item::ops::invalidate_properties(w, i);
                    }
                    Ok(())
                },
            ),
        }
    };
    // get only
    ($data:path, $name:literal, $kind:expr, $read:expr, $write:expr, |$gd:ident| $get:expr) => {
        $crate::props::PropDef {
            name: $name,
            kind: $kind,
            read: $read,
            write: $write,
            get: |w: &mut muo_server::world::World, e: muo_server::ids::EntityId| match e {
                muo_server::ids::EntityId::Item(i) => {
                    $data(w, i).map_or($crate::props::Value::Null, |$gd| $get)
                }
                _ => $crate::props::Value::Null,
            },
            set: None,
        }
    };
}

/// A `[CommandProperty]` of an item written by hand (what [`ext_field!`] cannot say: computed getters, setters that
/// call the class's `Allow*Change` / `On*Changed`). `|w, i| get` returns a [`Value`]; the setter body, `|w, i, v| ..`,
/// is a block of statements (it may `?` a `String` error) and the macro answers `Ok(())`.
#[macro_export]
macro_rules! item_prop {
    ($name:literal, $kind:expr, $rd:expr, $wr:expr, |$w:ident, $i:ident| $get:expr) => {
        $crate::props::PropDef {
            name: $name,
            kind: $kind,
            read: $rd,
            write: $wr,
            get: |$w, e| match e {
                muo_server::ids::EntityId::Item($i) => $get,
                _ => $crate::props::Value::Null,
            },
            set: None,
        }
    };
    ($name:literal, $kind:expr, $rd:expr, $wr:expr, |$w:ident, $i:ident| $get:expr, |$sw:ident, $si:ident, $v:ident| $set:expr) => {
        $crate::props::PropDef {
            name: $name,
            kind: $kind,
            read: $rd,
            write: $wr,
            get: |$w, e| match e {
                muo_server::ids::EntityId::Item($i) => $get,
                _ => $crate::props::Value::Null,
            },
            set: Some(|$sw, e, $v| {
                let muo_server::ids::EntityId::Item($si) = e else {
                    return Err("That is not an item.".to_string());
                };
                $set;
                Ok(())
            }),
        }
    };
}

/// [`item_prop!`] for a mobile.
#[macro_export]
macro_rules! mobile_prop {
    ($name:literal, $kind:expr, $rd:expr, $wr:expr, |$w:ident, $i:ident| $get:expr) => {
        $crate::props::PropDef {
            name: $name,
            kind: $kind,
            read: $rd,
            write: $wr,
            get: |$w, e| match e {
                muo_server::ids::EntityId::Mobile($i) => $get,
                _ => $crate::props::Value::Null,
            },
            set: None,
        }
    };
    ($name:literal, $kind:expr, $rd:expr, $wr:expr, |$w:ident, $i:ident| $get:expr, |$sw:ident, $si:ident, $v:ident| $set:expr) => {
        $crate::props::PropDef {
            name: $name,
            kind: $kind,
            read: $rd,
            write: $wr,
            get: |$w, e| match e {
                muo_server::ids::EntityId::Mobile($i) => $get,
                _ => $crate::props::Value::Null,
            },
            set: Some(|$sw, e, $v| {
                let muo_server::ids::EntityId::Mobile($si) = e else {
                    return Err("That is not a mobile.".to_string());
                };
                $set;
                Ok(())
            }),
        }
    };
}

/// The kinds hand-written tables share.
pub mod kinds {
    use super::Kind;
    use crate::utilities::types::{Numeric, PropType};
    pub const INT: Kind = Kind::Plain(PropType::Numeric(Numeric::Int));
    pub const LONG: Kind = Kind::Plain(PropType::Numeric(Numeric::Long));
    pub const BOOL: Kind = Kind::Plain(PropType::Bool);
    pub const STR: Kind = Kind::Plain(PropType::Str);
    pub const DOUBLE: Kind = Kind::Plain(PropType::Double);
    pub const ENTITY: Kind = Kind::Plain(PropType::Entity);
    pub const POINT3: Kind = Kind::Plain(PropType::IPoint3D);
    pub const POINT2: Kind = Kind::Plain(PropType::IPoint2D);
}

/// The tables classes added, by class full name.
#[derive(Default)]
pub struct PropRegistry {
    tables: IndexMap<&'static str, Vec<&'static [PropDef]>>,
}

/// Adds `defs` as the `[CommandProperty]` members `class` declares (`"Server.Item"`).
pub fn add(w: &mut World, class: &'static str, defs: &'static [PropDef]) {
    w.resources
        .get_mut::<PropRegistry>()
        .tables
        .entry(class)
        .or_default()
        .push(defs);
}

/// Every class that has a table, with the names of the properties it declares.
pub fn declared_names(w: &World) -> Vec<(&'static str, Vec<&'static str>)> {
    fn objects(defs: &'static [PropDef], out: &mut IndexMap<&'static str, Vec<&'static str>>) {
        for d in defs {
            let (name, subs) = match &d.kind {
                Kind::Object(n, s) => (*n, *s),
                Kind::TextDef(s) => ("TextDefinition", *s),
                Kind::Nullable(Kind::Object(n, s)) => (*n, *s),
                _ => continue,
            };
            let e = out.entry(name).or_default();
            for sd in subs {
                if !e.contains(&sd.name) {
                    e.push(sd.name);
                }
            }
            objects(subs, out);
        }
    }
    let Some(r) = w.resources.get::<PropRegistry>() else {
        return Vec::new();
    };
    let mut out: IndexMap<&'static str, Vec<&'static str>> = IndexMap::new();
    for (c, ts) in &r.tables {
        for t in ts {
            out.entry(*c).or_default().extend(t.iter().map(|d| d.name));
            objects(t, &mut out);
        }
    }
    out.into_iter().collect()
}

/// The class chain of an entity, most derived first.
pub fn class_chain(w: &World, e: EntityId) -> Vec<&'static str> {
    let mut out = Vec::new();
    match e {
        EntityId::Item(i) => {
            let mut t = w.item(i).map(|i| i.ty);
            while let Some(ty) = t {
                out.push(ty.full_name);
                // one class here stands for two: `Spawner : BaseSpawner`
                if ty.full_name == "Server.Engines.Spawners.Spawner" {
                    out.push("Server.Engines.Spawners.BaseSpawner");
                }
                t = ty.parent;
            }
        }
        EntityId::Mobile(m) => {
            let mut t = w.mobile(m).map(|m| m.ty);
            while let Some(ty) = t {
                out.push(ty.full_name);
                t = ty.parent;
            }
        }
    }
    out
}

/// `type.GetProperties()` that carry `[CommandProperty]`: ancestors first, a derived entry replacing an inherited
/// one of the same name (compared case-insensitively, as the commands do).
pub fn props_of(w: &World, e: EntityId) -> Vec<&'static PropDef> {
    let Some(reg) = w.resources.get::<PropRegistry>() else {
        return Vec::new();
    };
    let mut out: Vec<&'static PropDef> = Vec::new();
    let chain = class_chain(w, e);
    for class in chain.iter().rev() {
        for table in reg.tables.get(class).into_iter().flatten() {
            for d in table.iter() {
                match out.iter().position(|o| o.name.eq_ic(d.name)) {
                    Some(at) => out[at] = d,
                    None => out.push(d),
                }
            }
        }
    }
    hide_overridden(&chain, &mut out);
    out
}

/// An `override` without `[CommandProperty]` hides the base property from `GetProperties`' attribute lookup (attributes
/// of a property are not inherited): `BaseCreature.Combatant` overrides `Mobile.Combatant` bare.
fn hide_overridden(chain: &[&str], out: &mut Vec<&'static PropDef>) {
    if chain.contains(&"Server.Mobiles.BaseCreature") {
        out.retain(|d| !d.name.eq_ic("Combatant"));
    }
}

/// `GetProperty(name, IgnoreCase)`.
pub fn find_prop(w: &World, e: EntityId, name: &str) -> Option<&'static PropDef> {
    props_of(w, e).into_iter().find(|p| p.name.eq_ic(name))
}

/// The declared class of an entity-valued property (`Mobile.Backpack` is a `Container`), which is what a dotted
/// `where` / `order by` binding walks on at parse time. `None`: the declared type is an interface or a base
/// (`IEntity`, `Mobile`, `Item`), so any registered class may supply the next link.
pub fn link_class(prop: &str) -> Option<&'static str> {
    match prop.to_ascii_lowercase().as_str() {
        "backpack" => Some("Server.Items.Container"),
        "bankbox" => Some("Server.Items.BankBox"),
        "combatant" | "lastkiller" | "controlmaster" | "master" => Some("Server.Mobile"),
        "corpse" => Some("Server.Items.Container"),
        _ => None,
    }
}

/// `GetProperty(name)` over every registered table: the permissive stand-in for a link whose declared class is
/// not known ([`link_class`]). Tables of `Server.Item` and `Server.Mobile` come first.
pub fn find_prop_any(w: &World, name: &str) -> Option<&'static PropDef> {
    let reg = w.resources.get::<PropRegistry>()?;
    let hit = |tables: &Vec<&'static [PropDef]>| {
        tables
            .iter()
            .flat_map(|t| t.iter())
            .find(|d| d.name.eq_ic(name))
    };
    ["Server.Item", "Server.Mobile"]
        .iter()
        .filter_map(|c| reg.tables.get(c))
        .find_map(hit)
        .or_else(|| reg.tables.values().find_map(hit))
}

/// The property tables of a class for the type given by name, for binding a `where` clause before any object is
/// seen: ancestors of the class, then the class.
pub fn props_of_class(w: &World, class_chain_root_first: &[&'static str]) -> Vec<&'static PropDef> {
    let Some(reg) = w.resources.get::<PropRegistry>() else {
        return Vec::new();
    };
    let mut out: Vec<&'static PropDef> = Vec::new();
    for class in class_chain_root_first {
        for table in reg.tables.get(class).into_iter().flatten() {
            for d in table.iter() {
                match out.iter().position(|o| o.name.eq_ic(d.name)) {
                    Some(at) => out[at] = d,
                    None => out.push(d),
                }
            }
        }
    }
    hide_overridden(class_chain_root_first, &mut out);
    out
}

/// `PropertyAccess`.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Access {
    pub read: bool,
    pub write: bool,
}

impl Access {
    pub const READ: Access = Access {
        read: true,
        write: false,
    };
    pub const WRITE: Access = Access {
        read: false,
        write: true,
    };
    pub const READ_WRITE: Access = Access {
        read: true,
        write: true,
    };
}

/// `Mobile.GetAccessLevelName`.
pub fn access_level_name(l: AccessLevel) -> &'static str {
    match l {
        AccessLevel::Player => "Player",
        AccessLevel::Counselor => "Counselor",
        AccessLevel::GameMaster => "Game Master",
        AccessLevel::Seer => "Seer",
        AccessLevel::Administrator => "Administrator",
        AccessLevel::Developer => "Developer",
        AccessLevel::Owner => "Owner",
    }
}

/// `Properties.GetPropertyInfoByName`'s checks on one property; `Err` is the fail reason.
pub fn check_access(from: AccessLevel, def: &PropDef, access: Access) -> Result<(), String> {
    let name = def.name;
    if access.read && from < def.read {
        return Err(format!(
            "You must be at least {} to get the property '{name}'.",
            access_level_name(def.read)
        ));
    }
    if access.write && from < def.write {
        return Err(format!(
            "You must be at least {} to set the property '{name}'.",
            access_level_name(def.write)
        ));
    }
    if access.write && def.set.is_none() {
        return Err(format!("Property '{name}' is read only."));
    }
    Ok(())
}

/// A bound `a.b.c` chain over an entity's class.
pub struct Chain {
    pub names: Vec<String>,
    pub defs: Vec<&'static PropDef>,
}

/// The sub-properties of a property whose value is an object (`TextDefinition`, a `[PropertyObject]`), if any.
pub fn subs_of(kind: &Kind) -> Option<&'static [PropDef]> {
    match kind {
        Kind::TextDef(s) | Kind::Object(_, s) => Some(s),
        Kind::Rect2D => Some(&RECT2D_SUBS),
        Kind::Rect3D => Some(&RECT3D_SUBS),
        Kind::Nullable(inner) => subs_of(inner),
        _ => None,
    }
}

/// The struct a `Rectangle2D`/`Rectangle3D` sub-property reads and writes: its getter sees the parent's value here
/// (the struct is a boxed copy, so a setter changes this copy; the props gump writes it back with `OnValueChanged`).
#[derive(Default)]
pub struct ParentValue(pub Option<std::rc::Rc<std::cell::RefCell<Value>>>);

pub fn set_parent_value(w: &mut World, v: Option<std::rc::Rc<std::cell::RefCell<Value>>>) {
    w.resources.get_mut::<ParentValue>().0 = v;
}

/// Makes `v` (what a `Rectangle2D`/`Rectangle3D` getter returned) the parent of the sub-properties read next.
pub fn bind_parent(w: &mut World, v: &Value) {
    if matches!(v, Value::Rect(_) | Value::Rect3(_)) {
        set_parent_value(
            w,
            Some(std::rc::Rc::new(std::cell::RefCell::new(v.clone()))),
        );
    }
}

fn rect_get(w: &World, f2: fn(&Rectangle2D) -> Value, f3: fn(&Rectangle3D) -> Value) -> Value {
    match w
        .resources
        .get::<ParentValue>()
        .and_then(|p| p.0.as_ref())
        .map(|c| c.borrow().clone())
    {
        Some(Value::Rect(r)) => f2(&r),
        Some(Value::Rect3(r)) => f3(&r),
        _ => Value::Null,
    }
}

fn rect_set(
    w: &mut World,
    v: &Value,
    f2: fn(&mut Rectangle2D, &Value),
    f3: fn(&mut Rectangle3D, &Value),
) -> Result<(), String> {
    if let Some(c) = w.resources.get::<ParentValue>().and_then(|p| p.0.as_ref()) {
        match &mut *c.borrow_mut() {
            Value::Rect(r) => f2(r, v),
            Value::Rect3(r) => f3(r, v),
            _ => {}
        }
    }
    Ok(())
}

macro_rules! rect_sub {
    ($name:literal, $kind:expr, $get2:expr, $get3:expr, $set2:expr, $set3:expr) => {
        PropDef {
            name: $name,
            kind: $kind,
            read: AccessLevel::Counselor,
            write: AccessLevel::Counselor,
            get: |w, _| rect_get(w, $get2, $get3),
            set: Some(|w, _, v| rect_set(w, &v, $set2, $set3)),
        }
    };
}

macro_rules! rect_ro {
    ($name:literal, $f:expr) => {
        PropDef {
            name: $name,
            kind: kinds::INT,
            read: AccessLevel::Counselor,
            write: AccessLevel::Counselor,
            get: |w, _| rect_get(w, |_| Value::Null, |r| Value::Int(i64::from(($f)(r)))),
            set: None,
        }
    };
}

fn p2(x: i32, y: i32) -> Value {
    Value::Point3D(Point3D::new(x, y, 0))
}

/// `Rectangle2D`'s `[CommandProperty(AccessLevel.Counselor)]` members (`End` is exclusive; Width and Height derive).
static RECT2D_SUBS: [PropDef; 6] = [
    rect_sub!(
        "Start",
        kinds::POINT2,
        |r| p2(r.start.x, r.start.y),
        |_| Value::Null,
        |r, v| {
            let p = v.point();
            r.start = muo_primitives::Point2D::new(p.x, p.y)
        },
        |_, _| {}
    ),
    rect_sub!(
        "End",
        kinds::POINT2,
        |r| p2(r.end.x, r.end.y),
        |_| Value::Null,
        |r, v| {
            let p = v.point();
            r.end = muo_primitives::Point2D::new(p.x, p.y)
        },
        |_, _| {}
    ),
    rect_sub!(
        "X",
        kinds::INT,
        |r| Value::Int(r.start.x.into()),
        |_| Value::Null,
        |r, v| r.start.x = v.i32(),
        |_, _| {}
    ),
    rect_sub!(
        "Y",
        kinds::INT,
        |r| Value::Int(r.start.y.into()),
        |_| Value::Null,
        |r, v| r.start.y = v.i32(),
        |_, _| {}
    ),
    rect_sub!(
        "Width",
        kinds::INT,
        |r| Value::Int(r.end.x.wrapping_sub(r.start.x).into()),
        |_| Value::Null,
        |r, v| r.end.x = r.start.x.wrapping_add(v.i32()),
        |_, _| {}
    ),
    rect_sub!(
        "Height",
        kinds::INT,
        |r| Value::Int(r.end.y.wrapping_sub(r.start.y).into()),
        |_| Value::Null,
        |r, v| r.end.y = r.start.y.wrapping_add(v.i32()),
        |_, _| {}
    ),
];

/// `Rectangle3D`'s: Width, Height and Depth are get-only.
static RECT3D_SUBS: [PropDef; 8] = [
    rect_sub!(
        "Start",
        kinds::POINT3,
        |_| Value::Null,
        |r| Value::Point3D(r.start),
        |_, _| {},
        |r, v| r.start = v.point()
    ),
    rect_sub!(
        "End",
        kinds::POINT3,
        |_| Value::Null,
        |r| Value::Point3D(r.end),
        |_, _| {},
        |r, v| r.end = v.point()
    ),
    rect_sub!(
        "X",
        kinds::INT,
        |_| Value::Null,
        |r| Value::Int(r.start.x.into()),
        |_, _| {},
        |r, v| r.start.x = v.i32()
    ),
    rect_sub!(
        "Y",
        kinds::INT,
        |_| Value::Null,
        |r| Value::Int(r.start.y.into()),
        |_, _| {},
        |r, v| r.start.y = v.i32()
    ),
    rect_sub!(
        "Z",
        kinds::INT,
        |_| Value::Null,
        |r| Value::Int(r.start.z.into()),
        |_, _| {},
        |r, v| r.start.z = v.i32()
    ),
    rect_ro!("Width", |r: &Rectangle3D| r.end.x.wrapping_sub(r.start.x)),
    rect_ro!("Height", |r: &Rectangle3D| r.end.y.wrapping_sub(r.start.y)),
    rect_ro!("Depth", |r: &Rectangle3D| r.end.z.wrapping_sub(r.start.z)),
];

/// `Properties.GetPropertyInfoChain` for an object we have in hand: each link is looked up on the entity the
/// previous one returned, or in the sub-properties of the object it returned (which belong to the same owner). The
/// third value is the links' declared names (`InternalGetValue` prints those, not what was typed).
pub fn resolve_chain_named(
    w: &mut World,
    from: AccessLevel,
    e: EntityId,
    path: &str,
    access: Access,
) -> Result<(EntityId, &'static PropDef, Vec<&'static str>), String> {
    let mut names: Vec<&'static str> = Vec::new();
    let parts: Vec<&str> = path.split('.').collect();
    let mut cur = e;
    let mut subs: Option<&'static [PropDef]> = None;
    for (i, part) in parts.iter().enumerate() {
        let last = i + 1 == parts.len();
        let found = match subs {
            Some(s) => s.iter().find(|d| d.name.eq_ic(part)),
            None => find_prop(w, cur, part),
        };
        let Some(def) = found else {
            return Err(format!("Property '{part}' not found."));
        };
        check_access(from, def, if last { access } else { Access::READ })?;
        names.push(def.name);
        if last {
            return Ok((cur, def, names));
        }
        match (def.get)(w, cur) {
            Value::Entity(Some(next)) => {
                cur = next;
                subs = None;
            }
            Value::Null | Value::Entity(None) => {
                return Err(format!("Property '{}' is null.", def.name));
            }
            v => match subs_of(&def.kind) {
                Some(s) => {
                    bind_parent(w, &v);
                    subs = Some(s)
                }
                None => return Err(format!("Property '{}' is null.", def.name)),
            },
        }
    }
    Err("Property chain is empty.".into())
}

/// [`resolve_chain_named`] without the declared names.
pub fn resolve_chain(
    w: &mut World,
    from: AccessLevel,
    e: EntityId,
    path: &str,
    access: Access,
) -> Result<(EntityId, &'static PropDef), String> {
    resolve_chain_named(w, from, e, path, access).map(|(o, d, _)| (o, d))
}

/// The name of an enum member, or the number.
pub fn enum_name(members: &[(&'static str, i64)], v: i64) -> String {
    match members.iter().find(|(_, n)| *n == v) {
        Some((name, _)) => (*name).to_string(),
        None => v.to_string(),
    }
}

/// `object.ToString()` of a property value, per its kind.
pub fn display(w: &World, kind: &Kind, v: &Value) -> String {
    match v {
        Value::Null => "null".into(),
        Value::Int(n) => n.to_string(),
        Value::UInt(n) => n.to_string(),
        Value::Float(f) => format_double(*f),
        Value::Bool(b) => if *b { "True" } else { "False" }.into(),
        Value::Str(s) | Value::Text(s) => s.clone(),
        Value::Enum(n) => match kind {
            Kind::Plain(PropType::Enum(m)) | Kind::CustomEnum(_, m) => enum_name(m, *n),
            Kind::Nullable(Kind::Plain(PropType::Enum(m))) => enum_name(m, *n),
            _ => n.to_string(),
        },
        Value::Entity(None) => "null".into(),
        Value::Entity(Some(EntityId::Item(i))) => {
            let t = w.item(*i).map_or("", |i| short(i.ty.full_name));
            format!("0x{:08X} \"{t}\"", i.0.value())
        }
        Value::Entity(Some(EntityId::Mobile(m))) => {
            let name = w
                .mobile(*m)
                .and_then(|m| (m.ty.name)(w, MobileId(m.serial)))
                .unwrap_or_default();
            format!("0x{:08X} \"{name}\"", m.0.value())
        }
        Value::Point3D(p) if matches!(kind, Kind::Plain(PropType::IPoint2D)) => {
            format!("({}, {})", p.x, p.y)
        }
        Value::Point3D(p) => format!("({}, {}, {})", p.x, p.y, p.z),
        Value::TimeSpan(t) => t.to_string(),
        Value::DateTime(d) => d.to_string(),
        Value::Rect(r) => r.to_string(),
        Value::Rect3(r) => r.to_string(),
        Value::Poison(None) => "null".into(),
        Value::Poison(Some(p)) => w
            .resources
            .get::<muo_server::poison::Poisons>()
            .is_some()
            .then(|| muo_server::poison::find(w, *p))
            .flatten()
            .map_or_else(String::new, |p| p.name.to_string()),
        Value::TextDef(t) => t.to_string(),
        Value::Map(None) => "null".into(),
        Value::Map(Some(m)) => w
            .maps
            .get(*m)
            .map_or_else(String::new, |m| m.name().to_string()),
    }
}

/// `DateTime.Parse` for what `ToString` writes (`MM/dd/yyyy HH:mm:ss`) and the ISO forms.
fn parse_date_time(t: &str) -> Option<DateTime> {
    if let Some(d) = DateTime::parse_iso(t) {
        return Some(d);
    }
    let (date, time) = t.split_once(' ').unwrap_or((t, "00:00:00"));
    let mut d = date.splitn(3, '/');
    let (mo, day, y) = (
        d.next()?.parse().ok()?,
        d.next()?.parse().ok()?,
        d.next()?.parse().ok()?,
    );
    let mut tm = time.splitn(3, ':');
    let (h, mi, sec) = (
        tm.next()?.parse().ok()?,
        tm.next()?.parse().ok()?,
        tm.next().map_or(Some(0), |s| s.parse().ok())?,
    );
    DateTime::from_ymd_hms(y, mo, day, h, mi, sec)
}

fn short(full: &str) -> &str {
    full.rsplit(['.', '+']).next().unwrap_or(full)
}

/// `double.ToString()` for the values properties hold (shortest round trip, no exponent for ordinary ranges).
pub fn format_double(f: f64) -> String {
    if f.fract() == 0.0 && f.abs() < 1e15 {
        format!("{}", f as i64)
    } else {
        format!("{f}")
    }
}

/// `value.ToString("X")` of a numeric property of the given type.
fn hex_in_width(num: &crate::utilities::types::Numeric, n: i64) -> String {
    use crate::utilities::types::Numeric as N;
    match num {
        N::Byte | N::SByte => format!("{:X}", n as u8),
        N::Short | N::UShort => format!("{:X}", n as u16),
        N::Int | N::UInt => format!("{:X}", n as u32),
        N::Long | N::ULong => format!("{:X}", n as u64),
    }
}

/// `Properties.InternalGetValue`'s text (without the name).
pub fn describe(w: &World, def: &PropDef, v: &Value) -> String {
    match (&def.kind, v) {
        (_, Value::Null) => "null".into(),
        (_, Value::TextDef(t)) => t.format().unwrap_or_else(|| "empty".to_string()),
        (Kind::Nullable(inner), v) if !matches!(**inner, Kind::Plain(PropType::Enum(_))) => {
            // `Nullable<int>` is numeric through its underlying type.
            let d = PropDef {
                name: def.name,
                kind: (**inner).clone(),
                read: def.read,
                write: def.write,
                get: def.get,
                set: def.set,
            };
            describe(w, &d, v)
        }
        // `Mobile.Body`/`BodyMod` are `Body` structs, not numbers: `Body.ToString()` is `0x{BodyID:X}`.
        // shortcut: matched by name, the two properties declare an int kind; a typed Body kind would be cleaner.
        (_, Value::Int(n)) if matches!(def.name, "Body" | "BodyMod") => format!("0x{n:X}"),
        (Kind::Plain(PropType::Numeric(k)), Value::Int(n)) => {
            format!("{n} (0x{})", hex_in_width(k, *n))
        }
        (Kind::Plain(PropType::Numeric(_)), Value::UInt(n)) => format!("{n} (0x{n:X})"),
        (Kind::Plain(PropType::Float | PropType::Double | PropType::Decimal), Value::Float(f)) => {
            format_double(*f)
        }
        (Kind::Plain(PropType::Char), Value::Int(n)) => {
            let c = char::from_u32(*n as u32).unwrap_or('?');
            format!("'{c}' ({n} [0x{n:X}])")
        }
        (_, Value::Int(n)) => format!("{n} (0x{n:X})"),
        (_, Value::Str(s)) => {
            if s == "null" {
                "@\"null\"".to_string()
            } else {
                format!("\"{s}\"")
            }
        }
        _ => display(w, &def.kind, v),
    }
}

/// `Types.TryParse` for a property, then to a [`Value`]; `Err` is the message the command prints.
pub fn parse_for(w: &World, def: &PropDef, text: &str) -> Result<Value, String> {
    parse_kind(w, &def.kind, text)
}

/// `Types.TryParse(type, text)` for a kind of value.
pub fn parse_kind(w: &World, kind: &Kind, text: &str) -> Result<Value, String> {
    let ty = match kind {
        Kind::Plain(t) => t,
        Kind::Map => {
            let v = text.trim();
            if v == "(-null-)" {
                return Ok(Value::Map(None));
            }
            return w
                .maps
                .parse(v)
                .map(|m| Value::Map(Some(m)))
                .ok_or_else(|| "That is not a valid map.".to_string());
        }
        Kind::Text | Kind::Object(..) => return Err("That property cannot be parsed.".into()),
        Kind::TimeSpan => {
            return match text.trim() {
                "(-null-)" => Ok(Value::Null),
                t => TimeSpan::try_parse(t)
                    .map(Value::TimeSpan)
                    .ok_or_else(|| "That is not properly formatted.".to_string()),
            };
        }
        Kind::DateTime => {
            return match text.trim() {
                "(-null-)" => Ok(Value::Null),
                t => parse_date_time(t)
                    .map(Value::DateTime)
                    .ok_or_else(|| "That is not properly formatted.".to_string()),
            };
        }
        Kind::Rect2D => {
            return Rectangle2D::try_parse(text.trim())
                .map(Value::Rect)
                .ok_or_else(|| "That is not properly formatted.".to_string());
        }
        Kind::Rect3D => {
            return Rectangle3D::try_parse(text.trim())
                .map(Value::Rect3)
                .ok_or_else(|| "That is not properly formatted.".to_string());
        }
        Kind::Poison => {
            // A reference type: the null sentinel clears it. `Poison.Parse` takes an index or a name.
            let t = text.trim();
            if text == "(-null-)" {
                return Ok(Value::Poison(None));
            }
            return w
                .resources
                .get::<muo_server::poison::Poisons>()
                .and_then(|p| p.try_parse(t))
                .map(|p| Value::Poison(Some(p)))
                .ok_or_else(|| "That is not properly formatted.".to_string());
        }
        // `TextDefinition.Parse` always succeeds; the sentinel is null (a reference type).
        Kind::TextDef(_) => {
            return Ok(if text == "(-null-)" {
                Value::Null
            } else {
                Value::TextDef(TextDefinition::parse(text))
            });
        }
        // A lifted parse: `null` (and the sentinel) is the unset value, else the underlying type's.
        Kind::Nullable(inner) => {
            return if text == "null" || text == "(-null-)" {
                Ok(Value::Null)
            } else {
                parse_kind(w, inner, text)
            };
        }
        Kind::CustomEnum(_, members) => {
            return parse_kind(w, &Kind::Plain(PropType::Enum(members)), text);
        }
    };
    let find_entity = |s: Serial| w.find_entity(s).is_some();
    let find_type = |name: &str| class_by_name(w, name).map(str::to_string);
    let ctx = ParseContext {
        find_entity: &find_entity,
        find_type: &find_type,
    };
    let parsed = try_parse(ty, Some(text), &ctx)?;
    Ok(match parsed {
        Parsed::Null => Value::Null,
        Parsed::Int(n) => Value::Int(n),
        Parsed::UInt(n) => Value::UInt(n),
        Parsed::Float(f) => Value::Float(f),
        Parsed::Bool(b) => Value::Bool(b),
        Parsed::Char(c) => Value::Int(i64::from(u32::from(c))),
        Parsed::Str(s) => Value::Str(s),
        Parsed::Serial(s) => Value::UInt(u64::from(s.value())),
        Parsed::Entity(s) => Value::Entity(s.and_then(|s| w.find_entity(s))),
        Parsed::Enum(n) => Value::Enum(n),
        Parsed::Type(t) => Value::Str(t),
        Parsed::Point3D(p) => Value::Point3D(p),
        Parsed::Point2D(p) => Value::Point3D(Point3D::new(p.x, p.y, 0)),
    })
}

// ---- constructor overloads (`[add Gold 100`) ------------------------------------------------------------------

/// A `ParameterInfo`: its type and, when `HasDefaultValue`, the default (`Type.Missing` takes it).
pub struct Param {
    pub name: &'static str,
    pub kind: Kind,
    pub default: Option<Value>,
    /// The declared class (`ParameterType.Name`) when `kind` can't say it: an enum, entity or parsable parameter.
    /// Empty for the rest.
    pub ty: &'static str,
}

/// One public `[Constructible]` constructor.
pub struct Ctor {
    /// `[Constructible(AccessLevel)]`; `None` is the plain attribute.
    pub access: Option<AccessLevel>,
    pub params: &'static [Param],
    /// Makes the object from one value per parameter (defaults already filled in).
    pub make: fn(&mut World, &[Value]) -> EntityId,
}

/// The overloads of each class by full name, each list in declaration order (what `GetConstructors()`
/// returns and `[add` tries in).
#[derive(Default)]
pub struct CtorRegistry {
    tables: IndexMap<&'static str, &'static [Ctor]>,
}

/// Adds the constructors of `class`, in declaration order. A class without a table has only its
/// parameterless `ItemType::construct` / `MobileType::construct`.
/// A generated table: keeps a hand-written one for the class, whichever runs first (equal-priority Configure order
/// is an unstable sort).
pub fn add_generated_ctors(w: &mut World, class: &'static str, ctors: &'static [Ctor]) {
    w.resources
        .get_mut::<CtorRegistry>()
        .tables
        .entry(class)
        .or_insert(ctors);
}

pub fn add_ctors(w: &mut World, class: &'static str, ctors: &'static [Ctor]) {
    w.resources
        .get_mut::<CtorRegistry>()
        .tables
        .insert(class, ctors);
}

// ---- helpers of the generated tables ----

/// The registered item class `full`. The generated tables name only classes their own crate registers.
pub fn item_ty(w: &World, full: &str) -> &'static muo_server::item::ItemType {
    match w.types.find_type_by_full_name(full, false) {
        Some(muo_server::type_registry::EntityType::Item(t)) => t,
        _ => unreachable!("ctor table names {full}, which is not a registered item class"),
    }
}

/// The registered mobile class `full`.
pub fn mobile_ty(w: &World, full: &str) -> &'static muo_server::mobile::MobileType {
    match w.types.find_type_by_full_name(full, false) {
        Some(muo_server::type_registry::EntityType::Mobile(t)) => t,
        _ => unreachable!("ctor table names {full}, which is not a registered mobile class"),
    }
}

/// `new T()`: the class's parameterless constructor.
pub fn ctor_default(w: &mut World, full: &str) -> EntityId {
    match w.types.find_type_by_full_name(full, false) {
        Some(muo_server::type_registry::EntityType::Item(t)) => EntityId::Item((t
            .construct
            .unwrap_or_else(|| unreachable!("{full} has no constructor")))(
            w
        )),
        Some(muo_server::type_registry::EntityType::Mobile(t)) => EntityId::Mobile((t
            .construct
            .unwrap_or_else(|| unreachable!("{full} has no constructor")))(
            w
        )),
        None => unreachable!("ctor table names {full}, which is not registered"),
    }
}

// shortcut: `T(int amount)` whose only use of the parameter is `Amount = amount` (the generator checks the constructor chain) is
// the default object with the amount set; same random calls, since `T()` is `this(1)`. Upgrade to a real function if a
// class's parameterless constructor ever differs.
/// `new T(amount)`.
pub fn ctor_amount(w: &mut World, full: &str, amount: i32) -> EntityId {
    let e = ctor_default(w, full);
    if let EntityId::Item(id) = e {
        muo_server::item::ops::set_amount(w, id, amount);
    }
    e
}

// shortcut: as `ctor_amount`, for `T(int hue)` whose chain only does `Hue = hue`.
/// `new T(hue)`.
pub fn ctor_hue(w: &mut World, full: &str, hue: i32) -> EntityId {
    let e = ctor_default(w, full);
    if let EntityId::Item(id) = e {
        let ty = w.item(id).map(|i| i.ty);
        if let Some(ty) = ty {
            (ty.set_hue)(w, id, hue);
        }
    }
    e
}

/// Every class that has an overload table, with it (a test lists them against the `[Constructible]` constructors).
pub fn all_ctors(w: &World) -> Vec<(&'static str, &'static [Ctor])> {
    w.resources
        .get::<CtorRegistry>()
        .map_or_else(Vec::new, |r| {
            r.tables.iter().map(|(k, v)| (*k, *v)).collect()
        })
}

/// The overload table of exactly this class (constructors are not inherited).
pub fn ctors_of(w: &World, class: &str) -> Option<&'static [Ctor]> {
    w.resources
        .get::<CtorRegistry>()?
        .tables
        .get(class)
        .copied()
}

/// `Add.ParseValues`: one value per parameter from `args`, a parameter that does not parse taking its default;
/// `None` when a required one fails.
pub fn parse_ctor_args(w: &World, params: &[Param], args: &[String]) -> Option<Vec<Value>> {
    let mut out = Vec::with_capacity(params.len());
    let mut a = 0;
    for p in params {
        let arg = args.get(a);
        if arg.is_some() {
            a += 1;
        }
        let parsed = arg
            .and_then(|t| parse_kind(w, &p.kind, t).ok())
            .filter(|v| *v != Value::Null);
        match (parsed, &p.default) {
            (Some(v), _) => out.push(v),
            (None, Some(d)) => out.push(d.clone()),
            (None, None) => return None,
        }
    }
    Some(out)
}

#[derive(Clone)]
/// A class found by name: its ancestors (root first, itself last) and whether it derives from `Item`.
pub struct ClassInfo {
    pub full_name: &'static str,
    pub chain: Vec<&'static str>,
    pub is_item: bool,
}

/// `AssemblyHandler.FindTypeByName(name)` over the registered classes and their ancestors (abstract ones too, which
/// `where BaseCreature` needs).
pub fn find_class(w: &World, name: &str) -> Option<ClassInfo> {
    // A registered item class named by its exact full name: the scan below would stop at the first type whose
    // chain holds it and return this same ancestor chain.
    if let Some(muo_server::type_registry::EntityType::Item(t)) =
        w.types.find_type_by_full_name(name, false)
    {
        let mut chain = Vec::new();
        let mut cur = Some(t);
        while let Some(ty) = cur {
            chain.push(ty.full_name);
            cur = ty.parent;
        }
        chain.reverse();
        return Some(ClassInfo {
            full_name: t.full_name,
            chain,
            is_item: true,
        });
    }
    let matches = |full: &str| short(full).eq_ic(name) || full.eq_ic(name);
    for t in w.types.item_types() {
        let mut chain = Vec::new();
        let mut cur = Some(*t);
        while let Some(ty) = cur {
            chain.push(ty.full_name);
            cur = ty.parent;
        }
        if let Some(at) = chain.iter().position(|f| matches(f)) {
            let full_name = chain[at];
            chain.drain(..at);
            chain.reverse();
            return Some(ClassInfo {
                full_name,
                chain,
                is_item: true,
            });
        }
    }
    for t in w.types.mobile_types() {
        let mut chain = Vec::new();
        let mut cur = Some(*t);
        while let Some(ty) = cur {
            chain.push(ty.full_name);
            cur = ty.parent;
        }
        if let Some(at) = chain.iter().position(|f| matches(f)) {
            let full_name = chain[at];
            chain.drain(..at);
            chain.reverse();
            return Some(ClassInfo {
                full_name,
                chain,
                is_item: false,
            });
        }
    }
    None
}

/// The full name of the class called `name`.
pub fn class_by_name(w: &World, name: &str) -> Option<&'static str> {
    find_class(w, name).map(|c| c.full_name)
}

/// `entity is <class>`.
pub fn is_class(w: &World, e: EntityId, class: &str) -> bool {
    class_chain(w, e).contains(&class)
}

// ---- the tables for Item, Mobile, PlayerMobile and BaseCreature -------------------------------------------------

#[allow(unused_variables)]
mod tables {
    use super::*;
    use crate::creature::data::CreatureData;
    use crate::creature::enums::FightMode;
    use muo_primitives::{Body, Direction};
    use muo_server::item::{LootType, ops as iops};
    use muo_server::layer::Layer;
    use muo_server::light_type::LightType;
    use muo_server::mobile::{
        StatLockType, part1::lifecycle as ml, part1::props_a as ma, part1::props_b as mb,
    };

    use AccessLevel::{Administrator as ADM, Counselor as CNS, GameMaster as GM};

    const INT: Kind = Kind::Plain(PropType::Numeric(
        crate::systems::utilities::types::Numeric::Int,
    ));
    const LONG: Kind = Kind::Plain(PropType::Numeric(
        crate::systems::utilities::types::Numeric::Long,
    ));
    const BOOL: Kind = Kind::Plain(PropType::Bool);
    const STR: Kind = Kind::Plain(PropType::Str);
    const DOUBLE: Kind = Kind::Plain(PropType::Double);
    const ENTITY: Kind = Kind::Plain(PropType::Entity);
    const POINT: Kind = Kind::Plain(PropType::IPoint3D);
    const UINT: Kind = Kind::Plain(PropType::Serial);

    const LOOT_TYPES: &[(&str, i64)] = &[
        ("Regular", 0),
        ("Newbied", 1),
        ("Blessed", 2),
        ("Cursed", 3),
    ];
    const LAYERS: &[(&str, i64)] = &[
        ("Invalid", 0),
        ("OneHanded", 1),
        ("TwoHanded", 2),
        ("Shoes", 3),
        ("Pants", 4),
        ("Shirt", 5),
        ("Helm", 6),
        ("Gloves", 7),
        ("Ring", 8),
        ("Talisman", 9),
        ("Neck", 10),
        ("Hair", 11),
        ("Waist", 12),
        ("InnerTorso", 13),
        ("Bracelet", 14),
        ("UnusedXF", 15),
        ("FacialHair", 16),
        ("MiddleTorso", 17),
        ("Earrings", 18),
        ("Arms", 19),
        ("Cloak", 20),
        ("Backpack", 21),
        ("OuterTorso", 22),
        ("OuterLegs", 23),
        ("InnerLegs", 24),
        ("Mount", 25),
        ("ShopBuy", 26),
        ("ShopResale", 27),
        ("ShopSell", 28),
        ("Bank", 29),
    ];
    const DIRECTIONS: &[(&str, i64)] = &[
        ("North", 0),
        ("Right", 1),
        ("East", 2),
        ("Down", 3),
        ("South", 4),
        ("Left", 5),
        ("West", 6),
        ("Up", 7),
        ("Running", 0x80),
    ];
    const ACCESS_LEVELS: &[(&str, i64)] = &[
        ("Player", 0),
        ("Counselor", 1),
        ("GameMaster", 2),
        ("Seer", 3),
        ("Administrator", 4),
        ("Developer", 5),
        ("Owner", 6),
    ];
    const NPC_GUILDS: &[(&str, i64)] = &[
        ("None", 0),
        ("MagesGuild", 1),
        ("WarriorsGuild", 2),
        ("ThievesGuild", 3),
        ("RangersGuild", 4),
        ("HealersGuild", 5),
        ("MinersGuild", 6),
        ("MerchantsGuild", 7),
        ("TinkersGuild", 8),
        ("TailorsGuild", 9),
        ("FishermensGuild", 10),
        ("BardsGuild", 11),
        ("BlacksmithsGuild", 12),
    ];
    const SOLEN_FRIENDSHIPS: &[(&str, i64)] = &[("None", 0), ("Red", 1), ("Black", 2)];
    const STAT_LOCKS: &[(&str, i64)] = &[("Up", 0), ("Down", 1), ("Locked", 2)];
    const FIGHT_MODES: &[(&str, i64)] = &[
        ("None", 0),
        ("Aggressor", 1),
        ("Strongest", 2),
        ("Weakest", 3),
        ("Closest", 4),
        ("Evil", 5),
    ];
    const ORDERS: &[(&str, i64)] = &[
        ("None", 0),
        ("Come", 1),
        ("Drop", 2),
        ("Follow", 3),
        ("Friend", 4),
        ("Unfriend", 5),
        ("Guard", 6),
        ("Attack", 7),
        ("Patrol", 8),
        ("Release", 9),
        ("Stay", 10),
        ("Stop", 11),
        ("Transfer", 12),
        ("Rename", 13),
    ];
    const AI_TYPES: &[(&str, i64)] = &[
        ("AI_Use_Default", 0),
        ("AI_Melee", 1),
        ("AI_Animal", 2),
        ("AI_Archer", 3),
        ("AI_Healer", 4),
        ("AI_Vendor", 5),
        ("AI_Mage", 6),
        ("AI_Berserk", 7),
        ("AI_Predator", 8),
        ("AI_Thief", 9),
    ];
    const LIGHTS: &[(&str, i64)] = &[
        ("ArchedWindowEast", 0),
        ("Circle225", 1),
        ("Circle150", 2),
        ("DoorSouth", 3),
        ("DoorEast", 4),
        ("NorthBig", 5),
        ("NorthEastBig", 6),
        ("EastBig", 7),
        ("WestBig", 8),
        ("SouthWestBig", 9),
        ("SouthBig", 10),
        ("NorthSmall", 11),
        ("NorthEastSmall", 12),
        ("EastSmall", 13),
        ("WestSmall", 14),
        ("SouthSmall", 15),
        ("DecorationNorth", 16),
        ("DecorationNorthEast", 17),
        ("EastTiny", 18),
        ("DecorationWest", 19),
        ("DecorationSouthWest", 20),
        ("SouthTiny", 21),
        ("RectWindowSouthNoRay", 22),
        ("RectWindowEastNoRay", 23),
        ("RectWindowSouth", 24),
        ("RectWindowEast", 25),
        ("ArchedWindowSouthNoRay", 26),
        ("ArchedWindowEastNoRay", 27),
        ("ArchedWindowSouth", 28),
        ("Circle300", 29),
        ("NorthWestBig", 30),
        ("DarkSouthEast", 31),
        ("DarkSouth", 32),
        ("DarkNorthWest", 33),
        ("DarkSouthEast2", 34),
        ("DarkEast", 35),
        ("DarkCircle300", 36),
        ("DoorOpenSouth", 37),
        ("DoorOpenEast", 38),
        ("SquareWindowEast", 39),
        ("SquareWindowEastNoRay", 40),
        ("SquareWindowSouth", 41),
        ("SquareWindowSouthNoRay", 42),
        ("Empty", 43),
        ("SkinnyWindowSouthNoRay", 44),
        ("SkinnyWindowEast", 45),
        ("SkinnyWindowEastNoRay", 46),
        ("HoleSouth", 47),
        ("HoleEast", 48),
        ("Moongate", 49),
        ("Strips", 50),
        ("SmallHoleSouth", 51),
        ("SmallHoleEast", 52),
        ("NorthBig2", 53),
        ("WestBig2", 54),
        ("NorthWestBig2", 55),
    ];

    fn ent(e: Option<impl Into<EntityId>>) -> Value {
        Value::Entity(e.map(Into::into))
    }
    fn mobile_of(v: &Value) -> Option<MobileId> {
        match v.entity() {
            Some(EntityId::Mobile(k)) => Some(k),
            _ => None,
        }
    }
    fn opt_str(s: Option<String>) -> Value {
        s.map_or(Value::Null, Value::Str)
    }
    fn date(d: muo_primitives::DateTime) -> Value {
        Value::DateTime(d)
    }

    /// An item property: `get` always, `set` optionally. The setter body is an expression of type `()`.
    macro_rules! ip {
        ($name:literal, $kind:expr, $r:expr, $wl:expr, |$w:ident, $i:ident| $get:expr) => {
            PropDef {
                name: $name,
                kind: $kind,
                read: $r,
                write: $wl,
                get: |$w: &mut World, e: EntityId| match e {
                    EntityId::Item($i) => $get,
                    _ => Value::Null,
                },
                set: None,
            }
        };
        ($name:literal, $kind:expr, $r:expr, $wl:expr, |$w:ident, $i:ident| $get:expr,
         |$w2:ident, $i2:ident, $v:ident| $set:expr) => {
            PropDef {
                name: $name,
                kind: $kind,
                read: $r,
                write: $wl,
                get: |$w: &mut World, e: EntityId| match e {
                    EntityId::Item($i) => $get,
                    _ => Value::Null,
                },
                set: Some(|$w2: &mut World, e: EntityId, $v: Value| match e {
                    EntityId::Item($i2) => {
                        $set;
                        Ok(())
                    }
                    _ => Err("That is not an item.".to_string()),
                }),
            }
        };
    }
    /// A mobile property.
    macro_rules! mp {
        ($name:literal, $kind:expr, $r:expr, $wl:expr, |$w:ident, $m:ident| $get:expr) => {
            PropDef {
                name: $name,
                kind: $kind,
                read: $r,
                write: $wl,
                get: |$w: &mut World, e: EntityId| match e {
                    EntityId::Mobile($m) => $get,
                    _ => Value::Null,
                },
                set: None,
            }
        };
        ($name:literal, $kind:expr, $r:expr, $wl:expr, |$w:ident, $m:ident| $get:expr,
         |$w2:ident, $m2:ident, $v:ident| $set:expr) => {
            PropDef {
                name: $name,
                kind: $kind,
                read: $r,
                write: $wl,
                get: |$w: &mut World, e: EntityId| match e {
                    EntityId::Mobile($m) => $get,
                    _ => Value::Null,
                },
                set: Some(|$w2: &mut World, e: EntityId, $v: Value| match e {
                    EntityId::Mobile($m2) => {
                        $set;
                        Ok(())
                    }
                    _ => Err("That is not a mobile.".to_string()),
                }),
            }
        };
    }
    /// A plain `Mobile` field with no side effects (an auto-property).
    macro_rules! mfield {
        ($name:literal, int $f:ident) => {
            mp!(
                $name,
                INT,
                GM,
                GM,
                |w, m| Value::Int(i64::from(w.mobile(m).map_or(0, |x| x.$f))),
                |w, m, v| {
                    if let Some(x) = w.mobile_mut(m) {
                        x.$f = v.i32();
                    }
                }
            )
        };
        ($name:literal, bool $f:ident) => {
            mp!(
                $name,
                BOOL,
                GM,
                GM,
                |w, m| Value::Bool(w.mobile(m).is_some_and(|x| x.$f)),
                |w, m, v| {
                    if let Some(x) = w.mobile_mut(m) {
                        x.$f = v.bool();
                    }
                }
            )
        };
        ($name:literal, str $f:ident) => {
            mp!(
                $name,
                STR,
                GM,
                GM,
                |w, m| opt_str(w.mobile(m).and_then(|x| x.$f.clone())),
                |w, m, v| {
                    if let Some(x) = w.mobile_mut(m) {
                        x.$f = v.string();
                    }
                }
            )
        };
    }

    fn weight_total(w: &World, i: ItemId, t: muo_server::item::TotalType) -> Value {
        Value::Int(i64::from(iops::get_total(w, i, t)))
    }

    /// `Item`'s `[CommandProperty]` members.
    pub static ITEM: &[PropDef] = &[
        ip!(
            "LootType",
            Kind::Plain(PropType::Enum(LOOT_TYPES)),
            GM,
            GM,
            |w, i| Value::Enum(w.item(i).map_or(0, |x| x.loot_type.0 as i64)),
            |w, i, v| iops::set_loot_type(w, i, LootType::from_raw(v.i64() as u8))
        ),
        ip!("DecayTime", Kind::TimeSpan, GM, GM, |w, i| {
            Value::TimeSpan(
                (w.item(i).map(|x| x.ty).map(|t| (t.decay_time)(w, i))).unwrap_or_default(),
            )
        }),
        ip!("Decays", BOOL, GM, GM, |w, i| Value::Bool(
            muo_server::item::base::decays(w, i)
        )),
        ip!(
            "Stackable",
            BOOL,
            GM,
            GM,
            |w, i| Value::Bool(w.item(i).is_some_and(|x| x.stackable())),
            |w, i, v| iops::set_stackable(w, i, v.bool())
        ),
        ip!(
            "Visible",
            BOOL,
            GM,
            GM,
            |w, i| Value::Bool(w.item(i).is_some_and(|x| x.visible())),
            |w, i, v| iops::set_visible(w, i, v.bool())
        ),
        ip!(
            "Movable",
            BOOL,
            GM,
            GM,
            |w, i| Value::Bool(w.item(i).is_some_and(|x| x.movable())),
            |w, i, v| iops::set_movable(w, i, v.bool())
        ),
        ip!("TotalGold", INT, GM, GM, |w, i| weight_total(
            w,
            i,
            muo_server::item::TotalType::Gold
        )),
        ip!("TotalItems", INT, GM, GM, |w, i| weight_total(
            w,
            i,
            muo_server::item::TotalType::Items
        )),
        ip!("TotalWeight", INT, GM, GM, |w, i| weight_total(
            w,
            i,
            muo_server::item::TotalType::Weight
        )),
        ip!(
            "Weight",
            DOUBLE,
            CNS,
            GM,
            |w, i| Value::Float(iops::weight(w, i)),
            |w, i, v| iops::set_weight(w, i, v.f64())
        ),
        ip!("PileWeight", INT, CNS, GM, |w, i| Value::Int(i64::from(
            iops::pile_weight(w, i)
        ))),
        ip!(
            "Hue",
            INT,
            GM,
            GM,
            |w, i| Value::Int(i64::from(w.item(i).map_or(0, |x| (x.ty.hue)(w, i)))),
            |w, i, v| if let Some(t) = w.item(i).map(|x| x.ty) {
                (t.set_hue)(w, i, v.i32())
            }
        ),
        ip!(
            "Layer",
            Kind::Plain(PropType::Enum(LAYERS)),
            GM,
            GM,
            |w, i| Value::Enum(w.item(i).map_or(0, |x| (x.ty.layer)(w, i).0 as i64)),
            |w, i, v| muo_server::item::base::set_layer(w, i, Layer::from_raw(v.i64() as u8))
        ),
        ip!("RootParent", ENTITY, GM, GM, |w, i| ent(iops::root_parent(
            w, i
        ))),
        ip!(
            "ItemID",
            INT,
            GM,
            GM,
            |w, i| Value::Int(i64::from(w.item(i).map_or(0, |x| (x.ty.item_id)(w, i)))),
            |w, i, v| if let Some(t) = w.item(i).map(|x| x.ty) {
                (t.set_item_id)(w, i, v.i32())
            }
        ),
        ip!(
            "Name",
            STR,
            GM,
            GM,
            |w, i| opt_str(iops::name(w, i)),
            |w, i, v| iops::set_name(w, i, v.string())
        ),
        ip!("Parent", ENTITY, GM, GM, |w, i| ent(w
            .item(i)
            .and_then(|x| x.parent))),
        ip!(
            "Light",
            Kind::Plain(PropType::Enum(LIGHTS)),
            GM,
            GM,
            |w, i| Value::Enum(w.item(i).and_then(|x| x.light()).map_or(0, |l| l as i64)),
            |w, i, v| if let Some(l) = LightType::from_i32(v.i32()) {
                iops::set_light(w, i, l)
            }
        ),
        ip!(
            "Direction",
            Kind::Plain(PropType::Enum(DIRECTIONS)),
            GM,
            GM,
            |w, i| Value::Enum(w.item(i).map_or(0, |x| i64::from(x.direction.bits()))),
            |w, i, v| iops::set_direction(w, i, Direction::from_bits_retain(v.i64() as u8))
        ),
        ip!(
            "Amount",
            INT,
            GM,
            GM,
            |w, i| Value::Int(i64::from(w.item(i).map_or(0, |x| x.amount))),
            |w, i, v| iops::set_amount(w, i, v.i32())
        ),
        ip!(
            "PlayerConstructed",
            BOOL,
            GM,
            GM,
            |w, i| Value::Bool(w.item(i).is_some_and(|x| x.player_constructed)),
            |w, i, v| if let Some(x) = w.item_mut(i) {
                x.player_constructed = v.bool()
            }
        ),
        ip!(
            "QuestItem",
            BOOL,
            GM,
            GM,
            |w, i| Value::Bool(w.item(i).is_some_and(|x| x.quest_item())),
            |w, i, v| iops::set_quest_item(w, i, v.bool())
        ),
        ip!("Created", Kind::DateTime, GM, GM, |w, i| date(
            w.item(i)
                .map_or(muo_primitives::DateTime::MIN, |x| x.created)
        )),
        ip!("Serial", UINT, CNS, CNS, |w, i| Value::UInt(u64::from(
            i.0.value()
        ))),
        ip!(
            "Map",
            Kind::Map,
            CNS,
            GM,
            |w, i| Value::Map(w.item(i).and_then(|x| x.map)),
            |w, i, v| iops::set_map(
                w,
                i,
                match v {
                    Value::Map(m) => m,
                    _ => None,
                }
            )
        ),
        ip!(
            "Location",
            POINT,
            CNS,
            GM,
            |w, i| Value::Point3D(w.item(i).map_or_else(Point3D::default, |x| x.location)),
            |w, i, v| iops::set_location(w, i, v.point())
        ),
        ip!(
            "X",
            INT,
            CNS,
            GM,
            |w, i| Value::Int(i64::from(w.item(i).map_or(0, |x| x.location.x))),
            |w, i, v| {
                let mut p = w.item(i).map_or_else(Point3D::default, |x| x.location);
                p.x = v.i32();
                iops::set_location(w, i, p)
            }
        ),
        ip!(
            "Y",
            INT,
            CNS,
            GM,
            |w, i| Value::Int(i64::from(w.item(i).map_or(0, |x| x.location.y))),
            |w, i, v| {
                let mut p = w.item(i).map_or_else(Point3D::default, |x| x.location);
                p.y = v.i32();
                iops::set_location(w, i, p)
            }
        ),
        ip!(
            "Z",
            INT,
            CNS,
            GM,
            |w, i| Value::Int(i64::from(w.item(i).map_or(0, |x| x.location.z))),
            |w, i, v| {
                let mut p = w.item(i).map_or_else(Point3D::default, |x| x.location);
                p.z = v.i32();
                iops::set_location(w, i, p)
            }
        ),
    ];

    fn res(w: &World, m: MobileId, t: muo_server::mobile::ResistanceType) -> Value {
        Value::Int(i64::from(
            muo_server::mobile::part1::lifecycle::peek_resistance(w, m, t),
        ))
    }
    fn mobile_pt(w: &World, m: MobileId) -> Point3D {
        w.mobile(m).map_or_else(Point3D::default, |x| x.location)
    }
    fn lock(l: StatLockType) -> Value {
        Value::Enum(l as i64)
    }
    fn from_lock(v: &Value) -> StatLockType {
        match v.i64() {
            1 => StatLockType::Down,
            2 => StatLockType::Locked,
            _ => StatLockType::Up,
        }
    }

    /// `Mobile`'s `[CommandProperty]` members.
    pub static MOBILE: &[PropDef] = &[
        // `Skills Skills` (`canModify: true`): `[props` opens the skills gump, `Skills.Magery.Base` walks into it
        PropDef {
            name: "Skills",
            kind: Kind::Object("Skills", crate::skills_props::SKILLS),
            read: CNS,
            write: CNS,
            get: |w, e| match e {
                EntityId::Mobile(m) if w.mobile(m).is_some_and(|m| m.skills.is_some()) => {
                    Value::Text("...".into())
                }
                _ => Value::Null,
            },
            set: None,
        },
        mp!(
            "SkillsCap",
            INT,
            GM,
            GM,
            |w, m| Value::Int(i64::from(
                w.mobile(m)
                    .and_then(|m| m.skills.as_ref())
                    .map_or(0, |k| k.cap)
            )),
            |w, m, v| {
                if let Some(k) = w.mobile_mut(m).and_then(|m| m.skills.as_mut()) {
                    k.cap = v.i32();
                }
            }
        ),
        mp!("PhysicalResistance", INT, CNS, GM, |w, m| res(
            w,
            m,
            muo_server::mobile::ResistanceType::Physical
        )),
        mp!("FireResistance", INT, CNS, GM, |w, m| res(
            w,
            m,
            muo_server::mobile::ResistanceType::Fire
        )),
        mp!("ColdResistance", INT, CNS, GM, |w, m| res(
            w,
            m,
            muo_server::mobile::ResistanceType::Cold
        )),
        mp!("PoisonResistance", INT, CNS, GM, |w, m| res(
            w,
            m,
            muo_server::mobile::ResistanceType::Poison
        )),
        mp!("EnergyResistance", INT, CNS, GM, |w, m| res(
            w,
            m,
            muo_server::mobile::ResistanceType::Energy
        )),
        mp!(
            "IgnoreMobiles",
            BOOL,
            GM,
            GM,
            |w, m| Value::Bool(muo_server::mobile::base::ignore_mobiles(w, m)),
            |w, m, v| ma::set_ignore_mobiles_base(w, m, v.bool())
        ),
        mp!(
            "VirtualArmorMod",
            INT,
            GM,
            GM,
            |w, m| Value::Int(i64::from(w.mobile(m).map_or(0, |x| x.virtual_armor_mod))),
            |w, m, v| ma::set_virtual_armor_mod(w, m, v.i32())
        ),
        mfield!("MeleeDamageAbsorb", int melee_damage_absorb),
        mfield!("MagicDamageAbsorb", int magic_damage_absorb),
        mp!("SkillsTotal", INT, GM, GM, |w, m| Value::Int(i64::from(
            ma::skills_total(w, m)
        ))),
        mfield!("BaseSoundID", int base_sound_id),
        mfield!("NameHue", int name_hue),
        mp!(
            "Hunger",
            INT,
            GM,
            GM,
            |w, m| Value::Int(i64::from(w.mobile(m).map_or(0, |x| x.hunger))),
            |w, m, v| ma::set_hunger(w, m, v.i32())
        ),
        mp!(
            "Thirst",
            INT,
            GM,
            GM,
            |w, m| Value::Int(i64::from(w.mobile(m).map_or(0, |x| x.thirst))),
            |w, m, v| ma::set_thirst(w, m, v.i32())
        ),
        mfield!("BAC", int bac),
        mfield!("AllowedStealthSteps", int allowed_stealth_steps),
        mp!(
            "Paralyzed",
            BOOL,
            GM,
            GM,
            |w, m| Value::Bool(muo_server::mobile::base::paralyzed(w, m)),
            |w, m, v| ma::set_paralyzed_base(w, m, v.bool())
        ),
        mfield!("DisarmReady", bool disarm_ready),
        mfield!("StunReady", bool stun_ready),
        mp!(
            "Frozen",
            BOOL,
            GM,
            GM,
            |w, m| Value::Bool(w.mobile(m).is_some_and(|x| x.frozen)),
            |w, m, v| ma::set_frozen(w, m, v.bool())
        ),
        mp!(
            "StrLock",
            Kind::Plain(PropType::Enum(STAT_LOCKS)),
            GM,
            GM,
            |w, m| lock(w.mobile(m).map_or(StatLockType::Up, |x| x.str_lock)),
            |w, m, v| ma::set_str_lock(w, m, from_lock(&v))
        ),
        mp!(
            "DexLock",
            Kind::Plain(PropType::Enum(STAT_LOCKS)),
            GM,
            GM,
            |w, m| lock(w.mobile(m).map_or(StatLockType::Up, |x| x.dex_lock)),
            |w, m, v| ma::set_dex_lock(w, m, from_lock(&v))
        ),
        mp!(
            "IntLock",
            Kind::Plain(PropType::Enum(STAT_LOCKS)),
            GM,
            GM,
            |w, m| lock(w.mobile(m).map_or(StatLockType::Up, |x| x.int_lock)),
            |w, m, v| ma::set_int_lock(w, m, from_lock(&v))
        ),
        mp!(
            "NextActionTime",
            LONG,
            ADM,
            ADM,
            |w, m| Value::Int(w.mobile(m).map_or(0, |x| x.next_action_time)),
            |w, m, v| if let Some(x) = w.mobile_mut(m) {
                x.next_action_time = v.i64()
            }
        ),
        mp!(
            "NextSkillTime",
            LONG,
            ADM,
            ADM,
            |w, m| Value::Int(w.mobile(m).map_or(0, |x| x.next_skill_time)),
            |w, m, v| if let Some(x) = w.mobile_mut(m) {
                x.next_skill_time = v.i64()
            }
        ),
        mp!(
            "Combatant",
            ENTITY,
            GM,
            GM,
            |w, m| ent(muo_server::mobile::base::combatant(w, m)),
            |w, m, v| ma::set_combatant_base(
                w,
                m,
                match v.entity() {
                    Some(EntityId::Mobile(c)) => Some(c),
                    _ => None,
                }
            )
        ),
        mp!("TotalGold", INT, GM, GM, |w, m| Value::Int(i64::from(
            ma::total_gold(w, m)
        ))),
        mp!("TotalItems", INT, GM, GM, |w, m| Value::Int(i64::from(
            ma::total_items(w, m)
        ))),
        mp!("TotalWeight", INT, GM, GM, |w, m| Value::Int(i64::from(
            ma::total_weight(w, m)
        ))),
        mp!(
            "TithingPoints",
            INT,
            GM,
            GM,
            |w, m| Value::Int(i64::from(w.mobile(m).map_or(0, |x| x.tithing_points))),
            |w, m, v| ma::set_tithing_points(w, m, v.i32())
        ),
        mp!(
            "Followers",
            INT,
            GM,
            GM,
            |w, m| Value::Int(i64::from(w.mobile(m).map_or(0, |x| x.followers))),
            |w, m, v| ma::set_followers(w, m, v.i32())
        ),
        mp!(
            "FollowersMax",
            INT,
            GM,
            GM,
            |w, m| Value::Int(i64::from(w.mobile(m).map_or(0, |x| x.followers_max))),
            |w, m, v| ma::set_followers_max(w, m, v.i32())
        ),
        mfield!("AutoPageNotify", bool auto_page_notify),
        mp!(
            "VirtualArmor",
            INT,
            GM,
            GM,
            |w, m| Value::Int(i64::from(w.mobile(m).map_or(0, |x| x.virtual_armor))),
            |w, m, v| ma::set_virtual_armor(w, m, v.i32())
        ),
        mp!("ArmorRating", DOUBLE, GM, GM, |w, m| Value::Float(
            w.mobile(m).map_or(0.0, |x| (x.ty.armor_rating)(w, m))
        )),
        mp!("Alive", BOOL, GM, GM, |w, m| Value::Bool(
            w.mobile(m).is_some_and(|x| (x.ty.alive)(w, m))
        )),
        mp!(
            "Corpse",
            ENTITY,
            GM,
            GM,
            |w, m| ent(w.mobile(m).and_then(|x| x.corpse)),
            |w, m, v| if let Some(x) = w.mobile_mut(m) {
                x.corpse = match v.entity() {
                    Some(EntityId::Item(i)) => Some(i),
                    _ => None,
                }
            }
        ),
        mp!(
            "LastKiller",
            ENTITY,
            GM,
            GM,
            |w, m| ent(w.mobile(m).and_then(|x| x.last_killer)),
            |w, m, v| if let Some(x) = w.mobile_mut(m) {
                x.last_killer = match v.entity() {
                    Some(EntityId::Mobile(k)) => Some(k),
                    _ => None,
                }
            }
        ),
        mfield!("Squelched", bool squelched),
        mp!(
            "LightLevel",
            INT,
            GM,
            GM,
            |w, m| Value::Int(i64::from(w.mobile(m).map_or(0, |x| x.light_level))),
            |w, m, v| ma::set_light_level(w, m, v.i32())
        ),
        mfield!("Profile", str profile),
        mfield!("ProfileLocked", bool profile_locked),
        mp!(
            "Player",
            BOOL,
            GM,
            GM,
            |w, m| Value::Bool(w.mobile(m).is_some_and(|x| x.player)),
            |w, m, v| ma::set_player(w, m, v.bool())
        ),
        mp!(
            "Title",
            STR,
            GM,
            GM,
            |w, m| opt_str(w.mobile(m).and_then(|x| x.title.clone())),
            |w, m, v| ma::set_title(w, m, v.string())
        ),
        mp!(
            "AccessLevel",
            Kind::Plain(PropType::Enum(ACCESS_LEVELS)),
            GM,
            GM,
            |w, m| Value::Enum(w.mobile(m).map_or(0, |x| x.access_level as i64)),
            |w, m, v| if let Some(l) = AccessLevel::from_i32(v.i32()) {
                ma::set_access_level(w, m, l)
            }
        ),
        mp!(
            "Fame",
            INT,
            GM,
            GM,
            |w, m| Value::Int(i64::from(w.mobile(m).map_or(0, |x| x.fame))),
            |w, m, v| ma::set_fame(w, m, v.i32())
        ),
        mp!(
            "Karma",
            INT,
            GM,
            GM,
            |w, m| Value::Int(i64::from(w.mobile(m).map_or(0, |x| x.karma))),
            |w, m, v| ma::set_karma(w, m, v.i32())
        ),
        mp!(
            "Blessed",
            BOOL,
            GM,
            GM,
            |w, m| Value::Bool(w.mobile(m).is_some_and(|x| x.blessed)),
            |w, m, v| ma::set_blessed(w, m, v.bool())
        ),
        mp!(
            "HueMod",
            INT,
            GM,
            GM,
            |w, m| Value::Int(i64::from(w.mobile(m).map_or(0, |x| x.hue_mod))),
            |w, m, v| mb::set_hue_mod(w, m, v.i32())
        ),
        mp!(
            "Hue",
            INT,
            GM,
            GM,
            |w, m| Value::Int(i64::from(w.mobile(m).map_or(0, |x| (x.ty.hue)(w, m)))),
            |w, m, v| if let Some(t) = w.mobile(m).map(|x| x.ty) {
                (t.set_hue)(w, m, v.i32())
            }
        ),
        mp!(
            "Direction",
            Kind::Plain(PropType::Enum(DIRECTIONS)),
            GM,
            GM,
            |w, m| Value::Enum(w.mobile(m).map_or(0, |x| i64::from(x.direction.bits()))),
            |w, m, v| mb::set_direction(w, m, Direction::from_bits_retain(v.i64() as u8))
        ),
        mp!(
            "Female",
            BOOL,
            GM,
            GM,
            |w, m| Value::Bool(w.mobile(m).is_some_and(|x| x.female)),
            |w, m, v| mb::set_female(w, m, v.bool())
        ),
        mp!(
            "Flying",
            BOOL,
            GM,
            GM,
            |w, m| Value::Bool(w.mobile(m).is_some_and(|x| x.flying)),
            |w, m, v| mb::set_flying(w, m, v.bool())
        ),
        mp!(
            "Warmode",
            BOOL,
            GM,
            GM,
            |w, m| Value::Bool(w.mobile(m).is_some_and(|x| x.warmode)),
            |w, m, v| mb::set_warmode(w, m, v.bool())
        ),
        mp!(
            "Hidden",
            BOOL,
            GM,
            GM,
            |w, m| Value::Bool(w.mobile(m).is_some_and(|x| x.hidden)),
            |w, m, v| mb::set_hidden(w, m, v.bool())
        ),
        mfield!("Language", str language),
        mfield!("SpeechHue", int speech_hue),
        mfield!("EmoteHue", int emote_hue),
        mfield!("WhisperHue", int whisper_hue),
        mfield!("YellHue", int yell_hue),
        mp!(
            "GuildTitle",
            STR,
            GM,
            GM,
            |w, m| opt_str(w.mobile(m).and_then(|x| x.guild_title.clone())),
            |w, m, v| mb::set_guild_title(w, m, v.string())
        ),
        mp!(
            "DisplayGuildTitle",
            BOOL,
            GM,
            GM,
            |w, m| Value::Bool(w.mobile(m).is_some_and(|x| x.display_guild_title)),
            |w, m, v| mb::set_display_guild_title(w, m, v.bool())
        ),
        mp!(
            "GuildFealty",
            ENTITY,
            GM,
            GM,
            |w, m| ent(w.mobile(m).and_then(|x| x.guild_fealty)),
            |w, m, v| {
                let fealty = match v.entity() {
                    Some(EntityId::Mobile(f)) => Some(f),
                    _ => None,
                };
                if let Some(x) = w.mobile_mut(m) {
                    x.guild_fealty = fealty;
                }
            }
        ),
        mp!(
            "NameMod",
            STR,
            GM,
            GM,
            |w, m| opt_str(w.mobile(m).and_then(|x| x.name_mod.clone())),
            |w, m, v| mb::set_name_mod(w, m, v.string())
        ),
        mp!(
            "YellowHealthbar",
            BOOL,
            GM,
            GM,
            |w, m| Value::Bool(w.mobile(m).is_some_and(|x| x.yellow_healthbar)),
            |w, m, v| mb::set_yellow_healthbar(w, m, v.bool())
        ),
        mp!(
            "RawName",
            STR,
            GM,
            GM,
            |w, m| opt_str(mb::raw_name(w, m)),
            |w, m, v| mb::set_raw_name(w, m, v.string())
        ),
        mp!(
            "Name",
            STR,
            GM,
            GM,
            |w, m| opt_str(w.mobile(m).and_then(|x| (x.ty.name)(w, m))),
            |w, m, v| if let Some(t) = w.mobile(m).map(|x| x.ty) {
                (t.set_name)(w, m, v.string())
            }
        ),
        // `Poison Poison { get; set; }`: `[set Poison Lethal`, `[set Poison (-null-)`
        mp!(
            "Poison",
            Kind::Poison,
            GM,
            GM,
            |w, m| Value::Poison(w.mobile(m).and_then(|x| x.poison)),
            |w, m, v| {
                muo_server::mobile::part1::props_b::set_poison(
                    w,
                    m,
                    if let Value::Poison(p) = v { p } else { None },
                )
            }
        ),
        mp!("IsBodyMod", BOOL, GM, GM, |w, m| Value::Bool(
            muo_server::mobile::part1::props_b::is_body_mod(w, m)
        )),
        mp!("Poisoned", BOOL, GM, GM, |w, m| Value::Bool(
            w.mobile(m).is_some_and(|x| x.poison.is_some())
        )),
        mp!(
            "BodyMod",
            INT,
            GM,
            GM,
            |w, m| Value::Int(i64::from(w.mobile(m).map_or(0, |x| x.body_mod.0))),
            |w, m, v| mb::set_body_mod(w, m, Body(v.i32()))
        ),
        mp!(
            "Body",
            INT,
            GM,
            GM,
            |w, m| Value::Int(i64::from(w.mobile(m).map_or(0, |x| x.cur_body().0))),
            |w, m, v| mb::set_body(w, m, Body(v.i32()))
        ),
        mp!(
            "LogoutLocation",
            POINT,
            GM,
            GM,
            |w, m| Value::Point3D(
                w.mobile(m)
                    .map_or_else(Point3D::default, |x| x.logout_location)
            ),
            |w, m, v| if let Some(x) = w.mobile_mut(m) {
                x.logout_location = v.point()
            }
        ),
        mp!(
            "LogoutMap",
            Kind::Map,
            GM,
            GM,
            |w, m| Value::Map(w.mobile(m).and_then(|x| x.logout_map)),
            |w, m, v| if let Some(x) = w.mobile_mut(m) {
                x.logout_map = match v {
                    Value::Map(mm) => mm,
                    _ => None,
                }
            }
        ),
        mp!(
            "SolidHueOverride",
            INT,
            GM,
            GM,
            |w, m| Value::Int(i64::from(w.mobile(m).map_or(0, |x| x.solid_hue_override))),
            |w, m, v| mb::set_solid_hue_override(w, m, v.i32())
        ),
        // `BankBox.get` makes (and equips) the box when there is none: the gump reading it allocates the serial too.
        mp!("BankBox", ENTITY, GM, GM, |w, m| ent(
            muo_server::mobile::part1::props_b::bank_box(w, m)
        )),
        mp!("Backpack", ENTITY, GM, GM, |w, m| ent(w
            .mobile(m)
            .and_then(|x| x.backpack))),
        mp!(
            "Kills",
            INT,
            GM,
            GM,
            |w, m| Value::Int(i64::from(w.mobile(m).map_or(0, |x| x.kills))),
            |w, m, v| mb::set_kills(w, m, v.i32())
        ),
        mp!(
            "Criminal",
            BOOL,
            GM,
            GM,
            |w, m| Value::Bool(muo_server::mobile::base::criminal(w, m)),
            |w, m, v| muo_server::mobile::base::set_criminal(w, m, v.bool())
        ),
        mp!(
            "StatCap",
            INT,
            GM,
            GM,
            |w, m| Value::Int(i64::from(w.mobile(m).map_or(0, |x| x.stat_cap))),
            |w, m, v| mb::set_stat_cap(w, m, v.i32())
        ),
        mfield!("Meditating", bool meditating),
        mfield!("CanSwim", bool can_swim),
        mfield!("CantWalk", bool cant_walk),
        mfield!("CanHearGhosts", bool can_hear_ghosts),
        mp!("RawStatTotal", INT, GM, GM, |w, m| Value::Int(
            w.mobile(m).map_or(0, |x| i64::from(x.str + x.dex + x.int))
        )),
        mp!(
            "RawStr",
            INT,
            GM,
            GM,
            |w, m| Value::Int(i64::from(w.mobile(m).map_or(0, |x| x.str))),
            |w, m, v| mb::set_raw_str(w, m, v.i32())
        ),
        mp!(
            "Str",
            INT,
            GM,
            GM,
            |w, m| Value::Int(i64::from(w.mobile(m).map_or(0, |x| (x.ty.str)(w, m)))),
            |w, m, v| if let Some(t) = w.mobile(m).map(|x| x.ty) {
                (t.set_str)(w, m, v.i32())
            }
        ),
        mp!(
            "RawDex",
            INT,
            GM,
            GM,
            |w, m| Value::Int(i64::from(w.mobile(m).map_or(0, |x| x.dex))),
            |w, m, v| mb::set_raw_dex(w, m, v.i32())
        ),
        mp!(
            "Dex",
            INT,
            GM,
            GM,
            |w, m| Value::Int(i64::from(w.mobile(m).map_or(0, |x| (x.ty.dex)(w, m)))),
            |w, m, v| if let Some(t) = w.mobile(m).map(|x| x.ty) {
                (t.set_dex)(w, m, v.i32())
            }
        ),
        mp!(
            "RawInt",
            INT,
            GM,
            GM,
            |w, m| Value::Int(i64::from(w.mobile(m).map_or(0, |x| x.int))),
            |w, m, v| mb::set_raw_int(w, m, v.i32())
        ),
        mp!(
            "Int",
            INT,
            GM,
            GM,
            |w, m| Value::Int(i64::from(w.mobile(m).map_or(0, |x| (x.ty.int)(w, m)))),
            |w, m, v| if let Some(t) = w.mobile(m).map(|x| x.ty) {
                (t.set_int)(w, m, v.i32())
            }
        ),
        mp!(
            "Hits",
            INT,
            GM,
            GM,
            |w, m| Value::Int(i64::from(w.mobile(m).map_or(0, |x| x.hits))),
            |w, m, v| mb::set_hits(w, m, v.i32())
        ),
        mp!("HitsMax", INT, GM, GM, |w, m| Value::Int(i64::from(
            w.mobile(m).map_or(0, |x| (x.ty.hits_max)(w, m))
        ))),
        mp!(
            "Stam",
            INT,
            GM,
            GM,
            |w, m| Value::Int(i64::from(w.mobile(m).map_or(0, |x| x.stam))),
            |w, m, v| mb::set_stam(w, m, v.i32())
        ),
        mp!("StamMax", INT, GM, GM, |w, m| Value::Int(i64::from(
            w.mobile(m).map_or(0, |x| (x.ty.stam_max)(w, m))
        ))),
        mp!(
            "Mana",
            INT,
            GM,
            GM,
            |w, m| Value::Int(i64::from(w.mobile(m).map_or(0, |x| x.mana))),
            |w, m, v| mb::set_mana(w, m, v.i32())
        ),
        mp!("ManaMax", INT, GM, GM, |w, m| Value::Int(i64::from(
            w.mobile(m).map_or(0, |x| (x.ty.mana_max)(w, m))
        ))),
        mp!(
            "HairItemID",
            INT,
            GM,
            GM,
            |w, m| Value::Int(i64::from(w.mobile(m).map_or(0, |x| x.hair_item_id))),
            |w, m, v| mb::set_hair_item_id(w, m, v.i32())
        ),
        mp!(
            "FacialHairItemID",
            INT,
            GM,
            GM,
            |w, m| Value::Int(i64::from(w.mobile(m).map_or(0, |x| x.facial_hair_item_id))),
            |w, m, v| mb::set_facial_hair_item_id(w, m, v.i32())
        ),
        mp!(
            "HairHue",
            INT,
            GM,
            GM,
            |w, m| Value::Int(i64::from(w.mobile(m).map_or(0, |x| x.hair_hue))),
            |w, m, v| mb::set_hair_hue(w, m, v.i32())
        ),
        mp!(
            "FacialHairHue",
            INT,
            GM,
            GM,
            |w, m| Value::Int(i64::from(w.mobile(m).map_or(0, |x| x.facial_hair_hue))),
            |w, m, v| mb::set_facial_hair_hue(w, m, v.i32())
        ),
        mp!("Created", Kind::DateTime, GM, GM, |w, m| date(
            w.mobile(m)
                .map_or(muo_primitives::DateTime::MIN, |x| x.created)
        )),
        mp!("Serial", UINT, CNS, CNS, |w, m| Value::UInt(u64::from(
            m.0.value()
        ))),
        mp!(
            "Map",
            Kind::Map,
            CNS,
            GM,
            |w, m| Value::Map(w.mobile(m).and_then(|x| x.map)),
            |w, m, v| ml::set_map(
                w,
                m,
                match v {
                    Value::Map(mm) => mm,
                    _ => None,
                }
            )
        ),
        mp!(
            "Location",
            POINT,
            CNS,
            GM,
            |w, m| Value::Point3D(mobile_pt(w, m)),
            |w, m, v| ml::set_location(w, m, v.point())
        ),
        mp!(
            "X",
            INT,
            CNS,
            GM,
            |w, m| Value::Int(i64::from(mobile_pt(w, m).x)),
            |w, m, v| ml::set_x(w, m, v.i32())
        ),
        mp!(
            "Y",
            INT,
            CNS,
            GM,
            |w, m| Value::Int(i64::from(mobile_pt(w, m).y)),
            |w, m, v| ml::set_y(w, m, v.i32())
        ),
        mp!(
            "Z",
            INT,
            CNS,
            GM,
            |w, m| Value::Int(i64::from(mobile_pt(w, m).z)),
            |w, m, v| ml::set_z(w, m, v.i32())
        ),
    ];

    fn pd(w: &World, m: MobileId) -> Option<&crate::player_mobile::PlayerData> {
        crate::player_mobile::data(w, m)
    }
    macro_rules! pfield {
        ($name:literal, int $f:ident) => {
            mp!(
                $name,
                INT,
                GM,
                GM,
                |w, m| Value::Int(pd(w, m).map_or(0, |d| i64::from(d.$f))),
                |w, m, v| if let Some(d) = crate::player_mobile::data_mut(w, m) {
                    d.$f = v.i32()
                }
            )
        };
        ($name:literal, bool $f:ident) => {
            mp!(
                $name,
                BOOL,
                GM,
                GM,
                |w, m| Value::Bool(pd(w, m).is_some_and(|d| d.$f)),
                |w, m, v| if let Some(d) = crate::player_mobile::data_mut(w, m) {
                    d.$f = v.bool()
                }
            )
        };
        ($name:literal, date $f:ident) => {
            mp!(
                $name,
                Kind::DateTime,
                GM,
                GM,
                |w, m| date(pd(w, m).map_or(muo_primitives::DateTime::MIN, |d| d.$f)),
                |w, m, v| {
                    if let (Some(d), Value::DateTime(t)) = (crate::player_mobile::data_mut(w, m), v)
                    {
                        d.$f = t;
                    }
                }
            )
        };
    }

    /// A `PlayerFlag` property: `GetFlag(flag)` / `SetFlag(flag, value)`.
    macro_rules! pflag {
        ($name:literal, $flag:ident) => {
            mp!(
                $name,
                BOOL,
                GM,
                GM,
                |w, m| Value::Bool(
                    pd(w, m).is_some_and(|d| d.flag(crate::player_mobile::PlayerFlag::$flag))
                ),
                |w, m, v| if let Some(d) = crate::player_mobile::data_mut(w, m) {
                    d.set_flag(crate::player_mobile::PlayerFlag::$flag, v.bool())
                }
            )
        };
    }

    /// `PlayerMobile`'s `[CommandProperty]` members that are plain state of `PlayerData`. The ones of the
    /// ethics, factions, virtues, honor, champion-title and murder systems (`EthicPlayer`, `FactionPlayerState`,
    /// `Virtues`, `ReceivedHonorContext`, `PingPongs`, `ShortTermMurders`, the murder
    /// counts), `ToothAche`, `EnemyOfOneType` and `DesignContext` are objects or state of other crates and
    /// are not listed.
    macro_rules! pspan {
        ($name:literal, $get:expr) => {
            mp!($name, Kind::TimeSpan, GM, GM, |w, m| Value::TimeSpan($get(
                w, m
            )))
        };
    }
    /// `<Name>Value` of a `ChampionTitleContext` title (`slot` in the context's field order).
    macro_rules! champ_value {
        ($name:literal, $slot:expr) => {
            mp!(
                $name,
                INT,
                GM,
                GM,
                |w, m| Value::Int(i64::from(
                    crate::player_mobile::tables::champion_titles_mut(w, m).titles[$slot]
                        .map_or(0, |t| t.value)
                )),
                |w, m, v| crate::player_mobile::tables::set_title_value(w, m, $slot, v.i32())
            )
        };
    }
    /// `<Name>LastDecay`.
    macro_rules! champ_decay {
        ($name:literal, $slot:expr) => {
            mp!(
                $name,
                Kind::DateTime,
                GM,
                GM,
                |w, m| date(
                    crate::player_mobile::tables::champion_titles_mut(w, m).titles[$slot]
                        .map_or(muo_primitives::DateTime::MIN, |t| t.last_decay)
                ),
                |w, m, v| {
                    if let Value::DateTime(t) = v {
                        crate::player_mobile::tables::set_title_last_decay(w, m, $slot, t);
                    }
                }
            )
        };
    }
    /// The sub-properties of `PlayerMobile.ChampionTitles`.
    static CHAMPION_TITLES: &[PropDef] = &[
        mp!(
            "Harrower",
            INT,
            GM,
            GM,
            |w, m| Value::Int(i64::from(
                crate::player_mobile::tables::champion_titles_mut(w, m).harrower
            )),
            |w, m, v| crate::player_mobile::tables::champion_titles_mut(w, m).harrower = v.i32()
        ),
        champ_value!("AbyssValue", 0),
        champ_decay!("AbyssLastDecay", 0),
        champ_value!("ArachnidValue", 1),
        champ_decay!("ArachnidLastDecay", 1),
        champ_value!("ColdBloodValue", 2),
        champ_decay!("ColdBloodLastDecay", 2),
        champ_value!("ForestLordValue", 3),
        champ_decay!("ForestLordLastDecay", 3),
        champ_value!("VerminHordeValue", 4),
        champ_decay!("VerminHordeLastDecay", 4),
        champ_value!("UnholyTerrorValue", 5),
        champ_decay!("UnholyTerrorLastDecay", 5),
        champ_value!("SleepingDragonValue", 6),
        champ_decay!("SleepingDragonLastDecay", 6),
        champ_value!("CorruptValue", 7),
        champ_decay!("CorruptLastDecay", 7),
        champ_value!("GladeValue", 8),
        champ_decay!("GladeLastDecay", 8),
    ];
    pub static PLAYER_MOBILE: &[PropDef] = &[
        pflag!("PagingSquelched", PAGING_SQUELCHED),
        pflag!("Glassblowing", GLASSBLOWING),
        pflag!("Masonry", MASONRY),
        pflag!("SandMining", SAND_MINING),
        pflag!("StoneMining", STONE_MINING),
        pflag!("ToggleMiningStone", TOGGLE_MINING_STONE),
        pflag!("KarmaLocked", KARMA_LOCKED),
        pflag!("AutoRenewInsurance", AUTO_RENEW_INSURANCE),
        pflag!("UseOwnFilter", USE_OWN_FILTER),
        pflag!("AcceptGuildInvites", ACCEPT_GUILD_INVITES),
        pflag!("HasStatReward", HAS_STAT_REWARD),
        pflag!("RefuseTrades", REFUSE_TRADES),
        pflag!("DisplayChampionTitle", DISPLAY_CHAMPION_TITLE),
        // `ChampionTitleContext` (`canModify: true`): `GetOrCreateChampionTitleContext`, so never null.
        mp!(
            "ChampionTitles",
            Kind::Object("ChampionTitleContext", CHAMPION_TITLES),
            GM,
            GM,
            |_w, _m| Value::Text("...".into())
        ),
        mp!(
            "Young",
            BOOL,
            GM,
            GM,
            |w, m| Value::Bool(crate::player_mobile::young(w, m)),
            |w, m, v| crate::player_mobile::set_young(w, m, v.bool())
        ),
        pfield!("AnkhNextUse", date ankh_next_use),
        pfield!("PeacedUntil", date peaced_until),
        pfield!("LastEscortTime", date last_escort_time),
        pfield!("LastPetBallTime", date last_pet_ball_time),
        pfield!("SessionStart", date session_start),
        pfield!("GuildMessageHue", int guild_message_hue),
        pfield!("AllianceMessageHue", int alliance_message_hue),
        pfield!("Profession", int profession),
        pfield!("IsStealthing", bool is_stealthing),
        pfield!("NpcGuildJoinTime", date npc_guild_join_time),
        pfield!("NextBODTurnInTime", date next_bod_turn_in_time),
        pfield!("LastOnline", date last_online),
        pfield!("ToTItemsTurnedIn", int tot_items_turned_in),
        pfield!("ToTTotalMonsterFame", int tot_total_monster_fame),
        pfield!("AcceleratedStart", date accelerated_start),
        // `PlayerMurderSystem.GetMurderContext`; `DateTime.MinValue` where there is no context.
        mp!(
            "ShortTermMurderExpiration",
            Kind::DateTime,
            GM,
            GM,
            |w, m| date(
                w.resources
                    .get::<crate::player_mobile::tables::MurderTable>()
                    .and_then(|t| t.0.get(&m))
                    .filter(|c| c.short_term_murders > 0)
                    .map_or(muo_primitives::DateTime::MIN, |c| {
                        w.now + (c.short_term_elapse - crate::player_mobile::game_time(w, m))
                    })
            )
        ),
        mp!(
            "LongTermMurderExpiration",
            Kind::DateTime,
            GM,
            GM,
            |w, m| date(
                w.resources
                    .get::<crate::player_mobile::tables::MurderTable>()
                    .and_then(|t| t.0.get(&m))
                    .filter(|_| w.mobile(m).is_some_and(|x| x.kills > 0))
                    .map_or(muo_primitives::DateTime::MIN, |c| {
                        w.now + (c.long_term_elapse - crate::player_mobile::game_time(w, m))
                    })
            )
        ),
        pspan!(
            "DisguiseTimeLeft",
            crate::player_mobile::hooks::disguise_time_remaining
        ),
        pspan!(
            "SavagePaintExpiration",
            crate::player_mobile::savage_paint_expiration
        ),
        pspan!(
            "NextSmithBulkOrder",
            crate::player_mobile::next_smith_bulk_order
        ),
        pspan!(
            "NextTailorBulkOrder",
            crate::player_mobile::next_tailor_bulk_order
        ),
        pspan!("GameTime", crate::player_mobile::game_time),
        pspan!("NpcGuildGameTime", |w, m| pd(w, m)
            .map_or(muo_primitives::TimeSpan::ZERO, |d| d
                .npc_guild_game_time)),
        mp!(
            "NpcGuild",
            Kind::Plain(PropType::Enum(NPC_GUILDS)),
            GM,
            GM,
            |w, m| Value::Enum(pd(w, m).map_or(0, |d| d.npc_guild as i64)),
            |w, m, v| if let Some(d) = crate::player_mobile::data_mut(w, m) {
                d.npc_guild = match v.i64() {
                    1 => crate::player_mobile::NpcGuild::MagesGuild,
                    2 => crate::player_mobile::NpcGuild::WarriorsGuild,
                    3 => crate::player_mobile::NpcGuild::ThievesGuild,
                    4 => crate::player_mobile::NpcGuild::RangersGuild,
                    5 => crate::player_mobile::NpcGuild::HealersGuild,
                    6 => crate::player_mobile::NpcGuild::MinersGuild,
                    7 => crate::player_mobile::NpcGuild::MerchantsGuild,
                    8 => crate::player_mobile::NpcGuild::TinkersGuild,
                    9 => crate::player_mobile::NpcGuild::TailorsGuild,
                    10 => crate::player_mobile::NpcGuild::FishermensGuild,
                    11 => crate::player_mobile::NpcGuild::BardsGuild,
                    12 => crate::player_mobile::NpcGuild::BlacksmithsGuild,
                    _ => crate::player_mobile::NpcGuild::None,
                }
            }
        ),
        mp!(
            "SolenFriendship",
            Kind::Plain(PropType::Enum(SOLEN_FRIENDSHIPS)),
            GM,
            GM,
            |w, m| Value::Enum(pd(w, m).map_or(0, |d| d.solen_friendship as i64)),
            |w, m, v| if let Some(d) = crate::player_mobile::data_mut(w, m) {
                d.solen_friendship = match v.i64() {
                    1 => crate::player_mobile::SolenFriendship::Red,
                    2 => crate::player_mobile::SolenFriendship::Black,
                    _ => crate::player_mobile::SolenFriendship::None,
                }
            }
        ),
        mp!(
            "AcceleratedSkill",
            Kind::Plain(PropType::Enum(SKILL_NAMES)),
            GM,
            GM,
            |w, m| Value::Enum(pd(w, m).map_or(0, |d| d.accelerated_skill as i64)),
            |w, m, v| if let (Some(d), Some(s)) = (
                crate::player_mobile::data_mut(w, m),
                muo_server::skills::SkillName::from_i32(v.i64() as i32)
            ) {
                d.accelerated_skill = s
            }
        ),
        mp!("KnownRecipes", INT, CNS, CNS, |w, m| Value::Int(
            pd(w, m)
                .and_then(|d| d.acquired_recipes.as_ref())
                .map_or(0, |r| r.len() as i64)
        )),
    ];

    fn cd(w: &World, m: MobileId) -> Option<&CreatureData> {
        w.mobile(m).and_then(|x| x.ext.get::<CreatureData>())
    }
    macro_rules! cfield {
        ($name:literal, int $f:ident) => {
            mp!(
                $name,
                INT,
                GM,
                GM,
                |w, m| Value::Int(cd(w, m).map_or(0, |d| i64::from(d.$f))),
                |w, m, v| if let Some(d) = w
                    .mobile_mut(m)
                    .and_then(|x| x.ext.get_mut::<CreatureData>())
                {
                    d.$f = v.i32()
                }
            )
        };
        ($name:literal, resist $f:ident) => {
            // `fieldChanged: OnResistanceSeedChange` -> `UpdateResistances()`
            mp!(
                $name,
                INT,
                GM,
                GM,
                |w, m| Value::Int(cd(w, m).map_or(0, |d| i64::from(d.$f))),
                |w, m, v| {
                    if let Some(d) = w
                        .mobile_mut(m)
                        .and_then(|x| x.ext.get_mut::<CreatureData>())
                    {
                        d.$f = v.i32();
                    }
                    crate::creature::control::update_resistances(w, m);
                }
            )
        };
        ($name:literal, bool $f:ident) => {
            mp!(
                $name,
                BOOL,
                GM,
                GM,
                |w, m| Value::Bool(cd(w, m).is_some_and(|d| d.$f)),
                |w, m, v| if let Some(d) = w
                    .mobile_mut(m)
                    .and_then(|x| x.ext.get_mut::<CreatureData>())
                {
                    d.$f = v.bool()
                }
            )
        };
        ($name:literal, double $f:ident) => {
            mp!(
                $name,
                DOUBLE,
                GM,
                GM,
                |w, m| Value::Float(cd(w, m).map_or(0.0, |d| d.$f)),
                |w, m, v| if let Some(d) = w
                    .mobile_mut(m)
                    .and_then(|x| x.ext.get_mut::<CreatureData>())
                {
                    d.$f = v.f64()
                }
            )
        };
        ($name:literal, double_coerced $f:ident) => {
            // `CoerceMoveSpeed`: anything non-positive means "inherit" (0).
            mp!(
                $name,
                DOUBLE,
                GM,
                GM,
                |w, m| Value::Float(cd(w, m).map_or(0.0, |d| d.$f)),
                |w, m, v| if let Some(d) = w
                    .mobile_mut(m)
                    .and_then(|x| x.ext.get_mut::<CreatureData>())
                {
                    d.$f = v.f64().max(0.0)
                }
            )
        };
        ($name:literal, mobile $f:ident) => {
            mp!(
                $name,
                ENTITY,
                GM,
                GM,
                |w, m| ent(cd(w, m).and_then(|d| d.$f)),
                |w, m, v| if let Some(d) = w
                    .mobile_mut(m)
                    .and_then(|x| x.ext.get_mut::<CreatureData>())
                {
                    d.$f = match v.entity() {
                        Some(EntityId::Mobile(k)) => Some(k),
                        _ => None,
                    }
                }
            )
        };
    }

    /// `BaseCreature`'s `[CommandProperty]` members that are plain state of `CreatureData`.
    pub static BASE_CREATURE: &[PropDef] = &[
        cfield!("RangePerception", int range_perception),
        cfield!("RangeFight", int range_fight),
        cfield!("RangeHome", int range_home),
        cfield!("Team", int team),
        mp!(
            "FightMode",
            Kind::Plain(PropType::Enum(FIGHT_MODES)),
            GM,
            GM,
            |w, m| Value::Enum(cd(w, m).map_or(0, |d| d.fight_mode as i64)),
            |w, m, v| if let (Some(d), Some(f)) = (
                w.mobile_mut(m)
                    .and_then(|x| x.ext.get_mut::<CreatureData>()),
                FightMode::from_i32(v.i32())
            ) {
                d.fight_mode = f
            }
        ),
        cfield!("ActiveSpeed", double active_speed),
        cfield!("PassiveSpeed", double passive_speed),
        mp!(
            "CurrentSpeed",
            DOUBLE,
            GM,
            GM,
            |w, m| Value::Float(cd(w, m).map_or(0.0, |d| d.current_speed)),
            |w, m, v| crate::creature::speeds::set_current_speed(w, m, v.f64())
        ),
        mp!(
            "Home",
            POINT,
            GM,
            GM,
            |w, m| Value::Point3D(cd(w, m).map_or_else(Point3D::default, |d| d.home)),
            |w, m, v| if let Some(d) = w
                .mobile_mut(m)
                .and_then(|x| x.ext.get_mut::<CreatureData>())
            {
                d.home = v.point()
            }
        ),
        mp!(
            "Controlled",
            BOOL,
            GM,
            GM,
            |w, m| Value::Bool(cd(w, m).is_some_and(|d| d.controlled)),
            |w, m, v| crate::creature::control::set_controlled(w, m, v.bool())
        ),
        cfield!("ControlTarget", mobile control_target),
        cfield!("MinTameSkill", double min_tame_skill),
        cfield!("Loyalty", int loyalty),
        cfield!("DamageMin", int damage_min),
        cfield!("DamageMax", int damage_max),
        cfield!("PhysicalResistanceSeed", resist physical_resistance_seed),
        cfield!("FireResistSeed", resist fire_resist_seed),
        cfield!("ColdResistSeed", resist cold_resist_seed),
        cfield!("PoisonResistSeed", resist poison_resist_seed),
        cfield!("EnergyResistSeed", resist energy_resist_seed),
        cfield!("PhysicalDamage", int physical_damage),
        cfield!("FireDamage", int fire_damage),
        cfield!("ColdDamage", int cold_damage),
        cfield!("PoisonDamage", int poison_damage),
        cfield!("EnergyDamage", int energy_damage),
        cfield!("SeeksHome", bool seeks_home),
        cfield!("IsPrisoner", bool is_prisoner),
        cfield!("FocusMob", mobile focus_mob),
        cfield!("BardProvoked", bool bard_provoked),
        cfield!("BardPacified", bool bard_pacified),
        cfield!("BardMaster", mobile bard_master),
        cfield!("BardTarget", mobile bard_target),
        cfield!("ChaosDamage", int chaos_damage),
        cfield!("DirectDamage", int direct_damage),
        // ---- the members that run code, or whose read/write levels differ ---------------------------------------
        mp!(
            "Summoned",
            BOOL,
            ADM,
            ADM,
            |w, m| Value::Bool(cd(w, m).is_some_and(|d| d.summoned)),
            |w, m, v| if let Some(d) = w
                .mobile_mut(m)
                .and_then(|x| x.ext.get_mut::<CreatureData>())
            {
                d.summoned = v.bool()
            }
        ),
        mp!(
            "ControlSlots",
            INT,
            ADM,
            ADM,
            |w, m| Value::Int(cd(w, m).map_or(0, |d| i64::from(d.control_slots))),
            |w, m, v| if let Some(d) = w
                .mobile_mut(m)
                .and_then(|x| x.ext.get_mut::<CreatureData>())
            {
                d.control_slots = v.i32()
            }
        ),
        mp!(
            "IsBonded",
            BOOL,
            GM,
            GM,
            |w, m| Value::Bool(cd(w, m).is_some_and(|d| d.is_bonded)),
            |w, m, v| {
                // `fieldChanged: OnBondedChange` is InvalidateProperties().
                if let Some(d) = w
                    .mobile_mut(m)
                    .and_then(|x| x.ext.get_mut::<CreatureData>())
                {
                    d.is_bonded = v.bool()
                }
                muo_server::mobile::part2::properties::invalidate_properties(w, m);
            }
        ),
        mp!(
            "IsStabled",
            BOOL,
            GM,
            ADM,
            |w, m| Value::Bool(cd(w, m).is_some_and(|d| d.is_stabled)),
            |w, m, v| {
                let on = v.bool();
                if let Some(d) = w
                    .mobile_mut(m)
                    .and_then(|x| x.ext.get_mut::<CreatureData>())
                {
                    d.is_stabled = on
                }
                if on {
                    crate::creature::control::stop_delete_timer(w, m);
                }
            }
        ),
        mp!(
            "StabledBy",
            ENTITY,
            GM,
            ADM,
            |w, m| ent(cd(w, m).and_then(|d| d.stabled_by)),
            |w, m, v| if let Some(d) = w
                .mobile_mut(m)
                .and_then(|x| x.ext.get_mut::<CreatureData>())
            {
                d.stabled_by = match v.entity() {
                    Some(EntityId::Mobile(k)) => Some(k),
                    _ => None,
                }
            }
        ),
        mp!(
            "Debug",
            BOOL,
            ADM,
            ADM,
            |w, m| Value::Bool(cd(w, m).is_some_and(|d| d.debug)),
            |w, m, v| if let Some(d) = w
                .mobile_mut(m)
                .and_then(|x| x.ext.get_mut::<CreatureData>())
            {
                d.debug = v.bool()
            }
        ),
        mp!(
            "Name",
            STR,
            GM,
            GM,
            |w, m| opt_str(crate::creature::overrides::name(w, m)),
            |w, m, v| crate::creature::overrides::set_name(w, m, v.string())
        ),
        mp!(
            "IsParagon",
            BOOL,
            GM,
            GM,
            |w, m| Value::Bool(cd(w, m).is_some_and(|d| d.is_paragon)),
            |w, m, v| crate::creature::overrides::set_is_paragon(w, m, v.bool())
        ),
        mp!("HitsMax", INT, GM, GM, |w, m| Value::Int(i64::from(
            crate::creature::overrides::hits_max(w, m)
        ))),
        mp!("StamMax", INT, GM, GM, |w, m| Value::Int(i64::from(
            crate::creature::overrides::stam_max(w, m)
        ))),
        mp!("ManaMax", INT, GM, GM, |w, m| Value::Int(i64::from(
            crate::creature::overrides::mana_max(w, m)
        ))),
        mp!(
            "AI",
            Kind::Plain(PropType::Enum(AI_TYPES)),
            GM,
            GM,
            |w, m| Value::Enum(cd(w, m).map_or(0, |d| d.current_ai as i64)),
            |w, m, v| {
                // `AI = AI_Use_Default` takes the default AI; then `ChangeAIType`.
                use crate::creature::enums::AIType;
                if let Some(ai) = AIType::from_i32(v.i32()) {
                    let ai = match cd(w, m) {
                        Some(d) if ai == AIType::AiUseDefault => d.default_ai,
                        _ => ai,
                    };
                    if let Some(d) = w
                        .mobile_mut(m)
                        .and_then(|x| x.ext.get_mut::<CreatureData>())
                    {
                        d.current_ai = ai
                    }
                    crate::ai::change_ai_type(w, m, ai);
                }
            }
        ),
        mp!("ChaseLeashRange", INT, GM, GM, |w, m| Value::Int(
            crate::creature::vtable::ctype(w, m)
                .map_or(0, |c| i64::from((c.chase_leash_range)(w, m)))
        )),
        mp!("FollowMoveSpeed", DOUBLE, GM, GM, |w, m| Value::Float(
            crate::creature::vtable::ctype(w, m).map_or(0.0, |c| (c.follow_move_speed)(w, m))
        )),
        // `IPoint2D`: null when unset, else `Point2D.ToString()`; the gump's SetPoint2DGump sets a `Point2D`.
        mp!(
            "TargetLocation",
            Kind::Plain(PropType::IPoint2D),
            GM,
            GM,
            |w, m| match cd(w, m).and_then(|d| d.target_location) {
                Some(p) => Value::Point3D(Point3D::new(p.x, p.y, 0)),
                None => Value::Null,
            },
            |w, m, v| {
                if let Some(d) = w
                    .mobile_mut(m)
                    .and_then(|x| x.ext.get_mut::<CreatureData>())
                {
                    d.target_location = (!v.is_null())
                        .then(|| muo_primitives::Point2D::new(v.point().x, v.point().y));
                }
            }
        ),
        mp!("CurrentMoveSpeed", DOUBLE, GM, GM, |w, m| Value::Float(
            crate::creature::speeds::current_move_speed(w, m)
        )),
        mp!(
            "Master",
            ENTITY,
            GM,
            GM,
            |w, m| ent(cd(w, m).and_then(|d| d.master)),
            |w, m, v| crate::creature::control::set_master(w, m, mobile_of(&v))
        ),
        mp!(
            "ControlMaster",
            ENTITY,
            GM,
            GM,
            |w, m| ent(crate::creature::control::control_master(w, m)),
            |w, m, v| crate::creature::control::set_master(w, m, mobile_of(&v))
        ),
        mp!(
            "SummonMaster",
            ENTITY,
            GM,
            GM,
            |w, m| ent(crate::creature::control::summon_master(w, m)),
            |w, m, v| crate::creature::control::set_master(w, m, mobile_of(&v))
        ),
        mp!(
            "ControlOrder",
            Kind::Plain(PropType::Enum(ORDERS)),
            GM,
            GM,
            |w, m| Value::Enum(cd(w, m).map_or(0, |d| d.control_order as i64)),
            |w, m, v| if let Some(o) = crate::creature::enums::OrderType::from_i32(v.i32()) {
                crate::creature::control::set_control_order_raw(w, m, o)
            }
        ),
        mp!(
            "Tamable",
            BOOL,
            GM,
            GM,
            |w, m| Value::Bool(cd(w, m).is_some_and(|d| d.tamable())),
            |w, m, v| if let Some(d) = w
                .mobile_mut(m)
                .and_then(|x| x.ext.get_mut::<CreatureData>())
            {
                d.tamable_raw = v.bool()
            }
        ),
        mp!("MySpawner", ENTITY, GM, GM, |w, m| ent(w
            .mobile(m)
            .and_then(|x| x.spawner)
            .map(|s| ItemId(Serial::new(s.0))))),
        mp!("LastOwner", ENTITY, GM, GM, |w, m| ent(
            cd(w, m).and_then(|d| d.owners.last().copied())
        )),
        mp!("DeleteTimeLeft", Kind::TimeSpan, GM, GM, |w, m| {
            let h = cd(w, m).map(|d| d.pending_delete_timer);
            Value::TimeSpan(match h {
                Some(h) if h.running(w) => h.next(w) - w.now,
                _ => muo_primitives::TimeSpan::ZERO,
            })
        }),
        cfield!("ActiveMoveSpeed", double_coerced active_move_speed),
        cfield!("PassiveMoveSpeed", double_coerced passive_move_speed),
        mp!(
            "HomeMap",
            Kind::Map,
            GM,
            GM,
            |w, m| Value::Map(cd(w, m).and_then(|d| d.home_map)),
            |w, m, v| if let (Some(d), Value::Map(mm)) = (
                w.mobile_mut(m)
                    .and_then(|x| x.ext.get_mut::<CreatureData>()),
                v
            ) {
                d.home_map = mm
            }
        ),
        mp!(
            "ControlDest",
            POINT,
            GM,
            GM,
            |w, m| Value::Point3D(cd(w, m).map_or_else(Point3D::default, |d| d.control_dest)),
            |w, m, v| if let Some(d) = w
                .mobile_mut(m)
                .and_then(|x| x.ext.get_mut::<CreatureData>())
            {
                d.control_dest = v.point()
            }
        ),
        mp!(
            "CurrentWayPoint",
            ENTITY,
            GM,
            GM,
            |w, m| ent(cd(w, m).and_then(|d| d.current_way_point)),
            |w, m, v| if let Some(d) = w
                .mobile_mut(m)
                .and_then(|x| x.ext.get_mut::<CreatureData>())
            {
                d.current_way_point = match v.entity() {
                    Some(EntityId::Item(k)) => Some(k),
                    _ => None,
                }
            }
        ),
        cfield!("HitsMaxSeed", int hits_max_seed),
        cfield!("StamMaxSeed", int stam_max_seed),
        cfield!("ManaMaxSeed", int mana_max_seed),
        cfield!("RemoveIfUntamed", bool remove_if_untamed),
        cfield!("RemoveStep", int remove_step),
        mp!(
            "BondingBegin",
            Kind::DateTime,
            GM,
            GM,
            |w, m| date(cd(w, m).map_or(muo_primitives::DateTime::MIN, |d| d.bonding_begin)),
            |w, m, v| {
                if let (Some(d), Value::DateTime(t)) = (
                    w.mobile_mut(m)
                        .and_then(|x| x.ext.get_mut::<CreatureData>()),
                    v,
                ) {
                    d.bonding_begin = t;
                }
            }
        ),
        mp!(
            "OwnerAbandonTime",
            Kind::DateTime,
            GM,
            GM,
            |w, m| date(cd(w, m).map_or(muo_primitives::DateTime::MIN, |d| d.owner_abandon_time)),
            |w, m, v| {
                if let (Some(d), Value::DateTime(t)) = (
                    w.mobile_mut(m)
                        .and_then(|x| x.ext.get_mut::<CreatureData>()),
                    v,
                ) {
                    d.owner_abandon_time = t;
                }
            }
        ),
        mp!(
            "BardEndTime",
            Kind::DateTime,
            GM,
            GM,
            |w, m| date(cd(w, m).map_or(muo_primitives::DateTime::MIN, |d| d.bard_end_time)),
            |w, m, v| {
                if let (Some(d), Value::DateTime(t)) = (
                    w.mobile_mut(m)
                        .and_then(|x| x.ext.get_mut::<CreatureData>()),
                    v,
                ) {
                    d.bard_end_time = t;
                }
            }
        ),
        mp!(
            "CorpseNameOverride",
            STR,
            GM,
            GM,
            |w, m| opt_str(cd(w, m).and_then(|d| d.corpse_name_override.clone())),
            |w, m, v| if let Some(d) = w
                .mobile_mut(m)
                .and_then(|x| x.ext.get_mut::<CreatureData>())
            {
                d.corpse_name_override = v.string()
            }
        ),
    ];
}

/// Adds the tables for `Server.Item`, `Server.Mobile`, `Server.Mobiles.PlayerMobile` and
/// `Server.Mobiles.BaseCreature`. A leaf crate adds its own with [`add`] from its `Configure` phase.
pub fn register_core(w: &mut World) {
    add(w, "Server.Item", tables::ITEM);
    add(w, "Server.Mobile", tables::MOBILE);
    add(w, "Server.Mobiles.PlayerMobile", tables::PLAYER_MOBILE);
    add(w, "Server.Mobiles.BaseCreature", tables::BASE_CREATURE);
    crate::items::prop_tables::register(w);
}

#[cfg(test)]
mod compare_tests {
    use super::*;

    /// `PropertyExpressions.TryCompare`: `null.CompareTo(null) = 0`, `real.CompareTo(null) = -1`, `null.CompareTo(real) = +1`;
    /// `string.CompareTo` is culture-sensitive.
    #[test]
    fn null_orders_last_and_strings_by_culture() {
        let real = Value::Str("a".into());
        assert_eq!(Value::Null.compare(&Value::Null), Some(Ordering::Equal));
        assert_eq!(real.compare(&Value::Null), Some(Ordering::Less));
        assert_eq!(Value::Null.compare(&real), Some(Ordering::Greater));
        assert_eq!(
            Value::Str("apple".into()).compare(&Value::Str("Zed".into())),
            Some(Ordering::Less)
        );
    }

    /// `{value:X}` of a negative number is as wide as its type.
    #[test]
    fn hex_of_a_negative_property_uses_its_width() {
        use crate::utilities::types::Numeric;
        assert_eq!(hex_in_width(&Numeric::Int, -1), "FFFFFFFF");
        assert_eq!(hex_in_width(&Numeric::Short, -2), "FFFE");
        assert_eq!(hex_in_width(&Numeric::Long, -1), "FFFFFFFFFFFFFFFF");
        assert_eq!(hex_in_width(&Numeric::Byte, 255), "FF");
    }
}
