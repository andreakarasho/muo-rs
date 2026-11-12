//! A [`PropsHost`] over `muo_content::props`, the table of `[CommandProperty]` members that items and mobiles add.
//!
//! A `[PropertyObject]` property (`Kind::Object`) opens a sub-gump over a [`SubObj`]: the owner entity and the
//! object's sub-property table, whose getters and setters take that owner.
//!
//! shortcut: the declared type of an entity property is not in the table, so any item or mobile may be assigned
//! (rather than only the current value's class).

use muo_primitives::dotnet::IgnoreCase;
use std::any::Any;
use std::rc::Rc;

use muo_content::props::{self, Kind, PropDef, Value};
use muo_content::utilities::types::PropType as ParsePropType;
use muo_primitives::{Point2D, Point3D, Serial, TimeSpan};
use muo_server::World;
use muo_server::ids::{EntityId, MobileId};
use muo_server::target::Targeted;

use super::host::{ListEntry, PropInfo, PropKind, PropObj, PropType, PropValue, PropsHost};

/// The host `register` installs.
pub struct RegistryHost;

fn entity(obj: &PropObj) -> Option<EntityId> {
    match obj {
        PropObj::Entity(e) => Some(*e),
        PropObj::Other(_) => None,
    }
}

/// A `[PropertyObject]` a props gump shows: the entity that owns it and the object's sub-properties.
pub struct SubObj {
    owner: EntityId,
    subs: &'static [PropDef],
    name: &'static str,
    /// Which of several same-class objects the owner holds (`PlayerBBMessage` n of a board); see [`props::sub_key`].
    key: usize,
    /// A `Rectangle2D`/`Rectangle3D` the gump shows: the boxed struct its sub-properties read and write.
    value: Option<Rc<std::cell::RefCell<Value>>>,
}

impl SubObj {
    /// An object (`MLQuestContext`, ..) the gump shows with `subs` as its properties, each reading the `owner`.
    pub fn object(owner: EntityId, subs: &'static [PropDef], name: &'static str) -> PropObj {
        Self::keyed(owner, subs, name, usize::MAX)
    }

    /// [`SubObj::object`] for the `key`th object of the owner: the table's getters read [`props::sub_key`].
    pub fn keyed(
        owner: EntityId,
        subs: &'static [PropDef],
        name: &'static str,
        key: usize,
    ) -> PropObj {
        PropObj::Other(Rc::new(SubObj {
            owner,
            subs,
            name,
            key,
            value: None,
        }))
    }
}

fn sub(obj: &PropObj) -> Option<&SubObj> {
    match obj {
        PropObj::Other(o) => o.downcast_ref::<SubObj>(),
        PropObj::Entity(_) => None,
    }
}

/// A `LandTarget`/`StaticTarget` a props gump shows (`[props`-like gumps on a non-entity target).
fn target(obj: &PropObj) -> Option<&Targeted> {
    match obj {
        PropObj::Other(o) => o
            .downcast_ref::<Targeted>()
            .filter(|t| !matches!(t, Targeted::Entity(_))),
        PropObj::Entity(_) => None,
    }
}

/// The `[CommandProperty]` names of `LandTarget` / `StaticTarget`, sorted ordinally as `BuildList` sorts them.
fn target_props(t: &Targeted) -> &'static [&'static str] {
    match t {
        Targeted::Land(_) => &["Flags", "Location", "Name", "TileID", "X", "Y", "Z"],
        _ => &["Flags", "Hue", "ItemID", "Location", "Name", "X", "Y", "Z"],
    }
}

/// `TileFlag.ToString()`: the names of the set bits, ", " apart; "0" for none.
pub fn tile_flags_text(f: muo_data::tile_data::TileFlag) -> String {
    let names: Vec<String> = f
        .iter_names()
        .map(|(n, _)| {
            if n.starts_with("HS") {
                return n.to_owned();
            }
            n.split('_')
                .map(|p| format!("{}{}", &p[..1], p[1..].to_ascii_lowercase()))
                .collect()
        })
        .collect();
    if names.is_empty() {
        "0".into()
    } else {
        names.join(", ")
    }
}

fn target_value(w: &World, t: &Targeted, name: &str) -> String {
    let (loc, hue) = match t {
        Targeted::Land(l) => (l.location, 0),
        Targeted::Static(s) => (s.location, s.hue),
        Targeted::Entity(_) => return String::new(),
    };
    match (name, t) {
        ("X", _) => loc.x.to_string(),
        ("Y", _) => loc.y.to_string(),
        ("Z", _) => loc.z.to_string(),
        ("Location", _) => format!("({}, {}, {})", loc.x, loc.y, loc.z),
        ("Hue", _) => hue.to_string(),
        ("TileID", Targeted::Land(l)) => l.tile_id.to_string(),
        ("ItemID", Targeted::Static(s)) => s.item_id.to_string(),
        ("Name", Targeted::Land(l)) => w.maps.tile_data.land(l.tile_id as usize).name.clone(),
        ("Name", Targeted::Static(s)) => w.maps.tile_data.item(s.item_id as usize).name.clone(),
        ("Flags", Targeted::Land(l)) => {
            tile_flags_text(w.maps.tile_data.land(l.tile_id as usize).flags)
        }
        ("Flags", Targeted::Static(s)) => {
            tile_flags_text(w.maps.tile_data.item(s.item_id as usize).flags)
        }
        _ => String::new(),
    }
}

/// The entity whose state the object's properties read: the entity itself, or the owner of a sub-object.
fn owner_of(obj: &PropObj) -> Option<EntityId> {
    match obj {
        PropObj::Entity(e) => Some(*e),
        PropObj::Other(_) => sub(obj).map(|s| s.owner),
    }
}

/// [`owner_of`] for a caller about to run a getter or setter of the object's table: sets the sub-object key first.
fn owner(w: &mut World, obj: &PropObj) -> Option<EntityId> {
    if let Some(s) = sub(obj) {
        props::set_sub_key(w, s.key);
        props::set_parent_value(w, s.value.clone());
    }
    owner_of(obj)
}

fn def_of(prop: &PropInfo) -> Option<&'static PropDef> {
    prop.token.downcast_ref::<&'static PropDef>().copied()
}

fn short(full: &str) -> &str {
    full.rsplit(['.', '+']).next().unwrap_or(full)
}

fn access_of(w: &World, m: MobileId) -> muo_server::mobile::AccessLevel {
    muo_content::mx::access_level(w, m)
}

fn prop_info(def: &'static PropDef) -> PropInfo {
    let nullable = matches!(
        &def.kind,
        Kind::Map
            | Kind::Poison
            | Kind::TextDef(_)
            | Kind::Object(..)
            | Kind::Nullable(_)
            | Kind::Plain(
                ParsePropType::Entity
                    | ParsePropType::EntityOf(_)
                    | ParsePropType::Str
                    | ParsePropType::Type
            )
    );
    // shortcut: PropDef has no `canModify` flag; an object-valued property is one the gump can open (`canModify: true`
    // or a settable class-typed property), so its edit button shows; add the flag to PropDef to tell them apart.
    let object = matches!(&def.kind, Kind::Object(..));
    PropInfo {
        name: def.name.to_owned(),
        read_only: def.set.is_none() && !object,
        write_level: def.write,
        can_write: def.set.is_some(),
        cpa_can_modify: object,
        nullable,
        hued: def.name == "Hue" && matches!(&def.kind, Kind::Plain(ParsePropType::Numeric(_))),
        token: Rc::new(def) as Rc<dyn Any>,
    }
}

fn opaque(v: Value) -> PropValue {
    PropValue::Opaque(Rc::new(v))
}

/// `PropertiesGump.ValueToString(object)` over a property value.
pub(crate) fn value_text(w: &mut World, def: &PropDef, v: &Value) -> String {
    match v {
        Value::Null | Value::Entity(None) | Value::Map(None) => "-null-".into(),
        // `o is Type type` prints `type.Name`.
        Value::Str(s) | Value::Text(s) if matches!(&def.kind, Kind::Plain(ParsePropType::Type)) => {
            short(s).to_owned()
        }
        Value::Str(s) => format!("\"{s}\""),
        Value::Bool(b) => if *b { "True" } else { "False" }.into(),
        Value::Int(n) if matches!(&def.kind, Kind::Plain(ParsePropType::Char)) => {
            let c = char::from_u32(*n as u32).unwrap_or('?');
            format!("0x{n:X} '{c}'")
        }
        // `Mobile.Body`/`BodyMod` are `Body` structs: `Body.ToString()` is `0x{BodyID:X}` (see props::describe).
        Value::Int(n) if matches!(def.name, "Body" | "BodyMod") => format!("0x{n:X}"),
        Value::Int(n) => format!("{n} (0x{:X})", hex_bits(&def.kind, *n)),
        Value::UInt(n) if matches!(&def.kind, Kind::Plain(ParsePropType::Serial)) => {
            let s = Serial::new(*n as u32);
            if s.is_item() {
                format!("(I) {s}")
            } else if s.is_mobile() {
                format!("(M) {s}")
            } else {
                format!("(?) {s}")
            }
        }
        Value::UInt(n) => format!("{n} (0x{n:X})"),
        Value::Entity(Some(EntityId::Mobile(m))) => {
            let (serial, name) = w.mobile(*m).map_or((Serial::ZERO, String::new()), |mm| {
                (mm.serial, (mm.ty.name)(w, *m).unwrap_or_default())
            });
            format!("(M) {serial} \"{name}\"")
        }
        Value::Entity(Some(EntityId::Item(i))) => {
            format!("(I) {}", w.item(*i).map_or(Serial::ZERO, |ii| ii.serial))
        }
        Value::TextDef(t) => t.format().unwrap_or_else(|| "-empty-".into()),
        Value::Poison(None) => "-null-".into(),
        other => props::display(w, &def.kind, other),
    }
}

fn to_value(w: &World, def: &PropDef, v: &PropValue) -> Result<Value, ()> {
    match v {
        PropValue::Null => Ok(match &def.kind {
            Kind::Plain(ParsePropType::Entity | ParsePropType::EntityOf(_)) => Value::Entity(None),
            Kind::Map => Value::Map(None),
            // `PropertyInfo.SetValue(o, null)` on a value type sets its default (a failed `SetGump` parse is 0).
            Kind::Plain(ParsePropType::Numeric(_)) => Value::Int(0),
            Kind::Plain(ParsePropType::Float | ParsePropType::Double | ParsePropType::Decimal) => {
                Value::Float(0.0)
            }
            Kind::Plain(ParsePropType::Bool) => Value::Bool(false),
            Kind::Plain(ParsePropType::Enum(_)) => Value::Enum(0),
            Kind::TimeSpan => Value::TimeSpan(TimeSpan::ZERO),
            _ => Value::Null,
        }),
        PropValue::Point3D(p) => Ok(Value::Point3D(*p)),
        PropValue::Point2D(p) => Ok(Value::Point3D(Point3D::new(p.x, p.y, 0))),
        PropValue::Hue(h) => Ok(Value::Int(i64::from(*h))),
        PropValue::TimeSpan(t) => Ok(Value::TimeSpan(*t)),
        PropValue::Text(s) => props::parse_for(w, def, s).map_err(|_| ()),
        PropValue::Opaque(o) => match o.downcast_ref::<SubObj>().and_then(|s| s.value.as_ref()) {
            Some(c) => Ok(c.borrow().clone()),
            None => o.downcast_ref::<Value>().cloned().ok_or(()),
        },
    }
}

/// `IFoo`: an interface name, taken as satisfied by any entity.
fn is_interface_name(t: &str) -> bool {
    t.len() > 1 && t.starts_with('I') && t[1..].starts_with(|c: char| c.is_ascii_uppercase())
}

/// `type.IsInstanceOfType(entity)` by short class name.
fn instance_of(w: &World, target: EntityId, t: &str) -> bool {
    match target {
        _ if is_interface_name(t) => true,
        EntityId::Item(_) if t == "Item" => true,
        EntityId::Mobile(_) if t == "Mobile" => true,
        _ => props::class_chain(w, target).iter().any(|c| short(c) == t),
    }
}

/// Which gump a property of this kind opens (`PropertiesGump`'s chain of `IsType`).
fn route(w: &mut World, e: EntityId, def: &'static PropDef, kind: &Kind) -> PropKind {
    match kind {
        Kind::Plain(ParsePropType::EntityOf(t)) => {
            // `obj?.GetType() ?? prop.PropertyType`: the held value's class, else the declared one
            let name = match (def.get)(w, e) {
                Value::Entity(Some(v)) => props::class_chain(w, v)
                    .first()
                    .map_or(*t, |c| short(c))
                    .to_owned(),
                _ => (*t).to_owned(),
            };
            PropKind::Entity(PropType {
                name,
                is_system_type: false,
                token: Rc::new(()),
            })
        }
        Kind::Plain(ParsePropType::Entity) => {
            let name = match (def.get)(w, e) {
                Value::Entity(Some(v)) => props::class_chain(w, v)
                    .first()
                    .map_or("IEntity", |c| short(c))
                    .to_owned(),
                _ => "IEntity".to_owned(),
            };
            PropKind::Entity(PropType {
                name,
                is_system_type: false,
                token: Rc::new(()),
            })
        }
        Kind::Plain(ParsePropType::Type) => PropKind::Type(PropType {
            name: "Type".into(),
            is_system_type: true,
            token: Rc::new(()),
        }),
        Kind::Plain(ParsePropType::IPoint3D) => PropKind::Point3D,
        Kind::Plain(ParsePropType::IPoint2D) => PropKind::Point2D,
        Kind::Plain(ParsePropType::Enum(members)) => PropKind::List(
            members.iter().map(|(n, _)| (*n).to_owned()).collect(),
            members
                .iter()
                .map(|(_, v)| opaque(Value::Enum(*v)))
                .collect(),
        ),
        Kind::Plain(ParsePropType::Bool) => PropKind::List(
            vec!["True".into(), "False".into()],
            vec![opaque(Value::Bool(true)), opaque(Value::Bool(false))],
        ),
        Kind::Map => PropKind::List(
            w.maps.map_names(),
            w.maps
                .map_values()
                .into_iter()
                .map(|m| opaque(Value::Map(Some(m))))
                .collect(),
        ),
        Kind::Plain(_) => PropKind::Text,
        Kind::Text => PropKind::Other,
        Kind::TimeSpan => PropKind::TimeSpan,
        Kind::DateTime | Kind::TextDef(_) => PropKind::Text,
        // `Rectangle2D` carries `[PropertyObject]`, so it drills into X, Y, Width.
        Kind::Rect2D | Kind::Rect3D => PropKind::PropertyObject,
        Kind::Poison => {
            let names = ["None", "Lesser", "Regular", "Greater", "Deadly", "Lethal"];
            let poisons = w.resources.get::<muo_server::poison::Poisons>();
            PropKind::List(
                names.iter().map(|n| (*n).to_owned()).collect(),
                names
                    .iter()
                    .map(|n| {
                        opaque(Value::Poison(
                            poisons.and_then(|p| p.get_poison(n)).map(|p| p.id()),
                        ))
                    })
                    .collect(),
            )
        }
        // `Skills` of a mobile opens the skills gump; any other object drills in.
        Kind::Object("Skills", _) => PropKind::Skills,
        Kind::Object(..) => PropKind::PropertyObject,
        Kind::Nullable(inner) => route(w, e, def, inner),
        Kind::CustomEnum(names, _) => {
            PropKind::CustomEnum(names.iter().map(|n| (*n).to_owned()).collect())
        }
    }
}

impl PropsHost for RegistryHost {
    fn is_accessible(&self, w: &World, from: MobileId, obj: &PropObj) -> bool {
        muo_content::targets::is_accessible(w, from, owner_of(obj))
    }

    fn type_name(&self, w: &World, obj: &PropObj) -> String {
        if let Some(so) = sub(obj) {
            return so.name.to_owned();
        }
        if let Some(t) = target(obj) {
            return if matches!(t, Targeted::Land(_)) {
                "LandTarget"
            } else {
                "StaticTarget"
            }
            .into();
        }
        entity(obj)
            .and_then(|e| {
                props::class_chain(w, e)
                    .first()
                    .map(|c| short(c).to_owned())
            })
            .unwrap_or_default()
    }

    fn serial(&self, w: &World, obj: &PropObj) -> Option<Serial> {
        match entity(obj)? {
            EntityId::Item(i) => w.item(i).map(|i| i.serial),
            EntityId::Mobile(m) => w.mobile(m).map(|m| m.serial),
        }
    }

    fn build_list(&self, w: &mut World, viewer: MobileId, obj: &PropObj) -> Vec<ListEntry> {
        let access = access_of(w, viewer);
        if let Some(so) = sub(obj) {
            let mut defs: Vec<&'static PropDef> =
                so.subs.iter().filter(|d| access >= d.read).collect();
            // `[NoSort]` (Rectangle2D, Rectangle3D) keeps declaration order
            if !matches!(so.name, "Rectangle2D" | "Rectangle3D") {
                defs.sort_by(|a, b| a.name.cmp(b.name));
            }
            let mut list = vec![ListEntry::Type(short(so.name).to_owned())];
            list.extend(defs.into_iter().map(|d| ListEntry::Prop(prop_info(d))));
            return list;
        }
        if let Some(t) = target(obj) {
            let mut list = vec![ListEntry::Type(
                if matches!(t, Targeted::Land(_)) {
                    "LandTarget"
                } else {
                    "StaticTarget"
                }
                .into(),
            )];
            list.extend(target_props(t).iter().map(|n| {
                ListEntry::Prop(PropInfo {
                    name: (*n).to_owned(),
                    read_only: true,
                    write_level: muo_server::mobile::AccessLevel::Counselor,
                    can_write: false,
                    cpa_can_modify: false,
                    nullable: false,
                    hued: false,
                    token: Rc::new(*n) as Rc<dyn Any>,
                })
            }));
            return if access >= muo_server::mobile::AccessLevel::Counselor {
                list
            } else {
                Vec::new()
            };
        }
        let Some(e) = entity(obj) else {
            return Vec::new();
        };
        let chain = props::class_chain(w, e); // most derived first

        // Each property belongs to the root-most class that declares it (walking up while the base type has it).
        let mut groups: Vec<(usize, &'static str, Vec<&'static PropDef>)> = Vec::new();
        let mut seen: Vec<&'static str> = Vec::new();
        for (distance, class) in chain.iter().enumerate().rev() {
            let mut own = Vec::new();
            for d in props::props_of_class(w, &[class]) {
                if seen.iter().any(|n| n.eq_ic(d.name)) {
                    continue;
                }
                seen.push(d.name);
                // the most derived definition is the one that counts
                // (a property the entity's full table hides, `Combatant` under `BaseCreature`, is not listed)
                let Some(effective) = props::find_prop(w, e, d.name) else {
                    continue;
                };
                if access >= effective.read {
                    own.push(effective);
                }
            }
            if !own.is_empty() {
                groups.push((distance, class, own));
            }
        }
        // GroupComparer: nearest to the object's own type first
        groups.sort_by_key(|g| g.0);

        let mut list = Vec::new();
        for (i, (_, class, mut defs)) in groups.into_iter().enumerate() {
            defs.sort_by(|a, b| a.name.cmp(b.name)); // CompareOrdinal
            if i != 0 {
                list.push(ListEntry::Blank);
            }
            list.push(ListEntry::Type(short(class).to_owned()));
            list.extend(defs.into_iter().map(|d| ListEntry::Prop(prop_info(d))));
        }
        list
    }

    fn value_to_string(&self, w: &mut World, obj: &PropObj, prop: &PropInfo) -> String {
        if let Some(t) = target(obj) {
            return target_value(w, t, &prop.name);
        }
        let (Some(e), Some(def)) = (owner(w, obj), def_of(prop)) else {
            return String::new();
        };
        let v = (def.get)(w, e);
        value_text(w, def, &v)
    }

    fn kind(&self, w: &mut World, obj: &PropObj, prop: &PropInfo) -> PropKind {
        let (Some(e), Some(def)) = (owner(w, obj), def_of(prop)) else {
            return PropKind::Other;
        };
        route(w, e, def, &def.kind)
    }

    fn value_object(&self, w: &mut World, obj: &PropObj, prop: &PropInfo) -> Option<PropObj> {
        let (e, def) = (owner(w, obj)?, def_of(prop)?);
        match (def.get)(w, e) {
            Value::Entity(Some(v)) => Some(PropObj::Entity(v)),
            Value::Null | Value::Entity(None) => None,
            _ => match &def.kind {
                Kind::Object(name, subs) => Some(SubObj::object(e, subs, name)),
                Kind::Rect2D | Kind::Rect3D => {
                    let v = (def.get)(w, e);
                    let name = if matches!(def.kind, Kind::Rect2D) {
                        "Rectangle2D"
                    } else {
                        "Rectangle3D"
                    };
                    let subs = props::subs_of(&def.kind)?;
                    Some(PropObj::Other(Rc::new(SubObj {
                        owner: e,
                        subs,
                        name,
                        key: usize::MAX,
                        value: Some(Rc::new(std::cell::RefCell::new(v))),
                    })))
                }
                _ => None,
            },
        }
    }

    fn as_mobile(&self, obj: &PropObj) -> Option<MobileId> {
        match entity(obj)? {
            EntityId::Mobile(m) => Some(m),
            EntityId::Item(_) => None,
        }
    }

    fn text_value(&self, w: &mut World, obj: &PropObj, prop: &PropInfo) -> String {
        let (Some(e), Some(def)) = (owner(w, obj), def_of(prop)) else {
            return String::new();
        };
        match (def.get)(w, e) {
            Value::Null | Value::Entity(None) | Value::Map(None) | Value::Poison(None) => {
                String::new()
            }
            Value::TextDef(t) => t.get_value(),
            v => props::display(w, &def.kind, &v),
        }
    }

    fn point2d_value(&self, w: &mut World, obj: &PropObj, prop: &PropInfo) -> Point2D {
        let p = self.point3d_value(w, obj, prop);
        Point2D::new(p.x, p.y)
    }

    fn point3d_value(&self, w: &mut World, obj: &PropObj, prop: &PropInfo) -> Point3D {
        match (owner(w, obj), def_of(prop)) {
            (Some(e), Some(def)) => (def.get)(w, e).point(),
            _ => Point3D::ZERO,
        }
    }

    fn time_span_value(&self, w: &mut World, obj: &PropObj, prop: &PropInfo) -> TimeSpan {
        match (owner(w, obj), def_of(prop)) {
            (Some(e), Some(def)) => match (def.get)(w, e) {
                Value::TimeSpan(t) => t,
                _ => TimeSpan::ZERO,
            },
            _ => TimeSpan::ZERO,
        }
    }

    fn parse(&self, w: &World, prop: &PropInfo, text: &str) -> Result<PropValue, ()> {
        let def = def_of(prop).ok_or(())?;
        props::parse_for(w, def, text).map(opaque).map_err(|_| ())
    }

    fn set_value(
        &self,
        w: &mut World,
        obj: &PropObj,
        prop: &PropInfo,
        value: &PropValue,
    ) -> Result<(), ()> {
        let (e, def) = (owner(w, obj).ok_or(())?, def_of(prop).ok_or(())?);
        let v = to_value(w, def, value)?;
        (def.set.ok_or(())?)(w, e, v).map_err(|_| ())
    }

    fn set_direct(
        &self,
        w: &mut World,
        from: MobileId,
        obj: &PropObj,
        prop: &PropInfo,
        value: &PropValue,
    ) -> Result<String, ()> {
        let (e, def) = (owner(w, obj).ok_or(())?, def_of(prop).ok_or(())?);
        let v = to_value(w, def, value)?;
        Ok(muo_commands::properties::set_direct(
            w, from, e, e, def, def.name, v, true,
        ))
    }

    fn set_custom_enum(
        &self,
        w: &mut World,
        from: MobileId,
        obj: &PropObj,
        prop: &PropInfo,
        name: &str,
    ) -> Result<String, ()> {
        // `Parse` or `Enum.Parse` of the picked name
        let (e, def) = (owner(w, obj).ok_or(())?, def_of(prop).ok_or(())?);
        let v = props::parse_for(w, def, name).map_err(|_| ())?;
        Ok(muo_commands::properties::set_direct(
            w, from, e, e, def, def.name, v, true,
        ))
    }

    fn log_change(
        &self,
        w: &mut World,
        from: MobileId,
        obj: &PropObj,
        property_name: &str,
        value: &PropValue,
    ) {
        let Some(e) = owner(w, obj) else { return };
        let text = match value {
            PropValue::Null => "(null)".to_owned(),
            PropValue::Point3D(p) => format!("({}, {}, {})", p.x, p.y, p.z),
            PropValue::Point2D(p) => format!("({}, {})", p.x, p.y),
            PropValue::TimeSpan(t) => t.to_string(),
            PropValue::Hue(h) => h.to_string(),
            PropValue::Text(s) => s.clone(),
            PropValue::Opaque(o) => match (
                o.downcast_ref::<Value>(),
                props::find_prop(w, e, property_name),
            ) {
                (Some(v), Some(def)) => props::display(w, &def.kind, v),
                (Some(v), None) => format!("{v:?}"),
                _ => String::new(),
            },
        };
        muo_commands::logging::log_change_property(w, from, e, property_name, &text);
    }

    fn hued_item_id(&self, w: &World, obj: &PropObj) -> i32 {
        match entity(obj) {
            Some(EntityId::Item(i)) => w.item(i).map_or(0, |i| i.item_id),
            Some(EntityId::Mobile(m)) => {
                if w.mobile(m).is_some_and(|m| m.female) {
                    0x2107
                } else {
                    0x2106
                }
            }
            None => 0,
        }
    }

    fn assign_targeted(
        &self,
        w: &mut World,
        from: MobileId,
        obj: &PropObj,
        prop: &PropInfo,
        ty: &PropType,
        targeted: Targeted,
    ) -> Result<bool, ()> {
        let (e, def) = (owner(w, obj).ok_or(())?, def_of(prop).ok_or(())?);
        let Targeted::Entity(mut target) = targeted else {
            return Ok(false);
        };
        if !ty.is_system_type
            && matches!(
                def.kind,
                Kind::Plain(ParsePropType::EntityOf(_) | ParsePropType::Entity)
            )
        {
            // `m_Type` is the held value's class (`PropsGump`), else the declared one
            let t = ty.name.as_str();
            let is_interface = is_interface_name(t);
            // `m_Type == typeof(BaseAddon) || m_Type.IsAssignableFrom(typeof(BaseAddon))`; interfaces are taken as assignable.
            if (t == "BaseAddon" || t == "Item" || is_interface)
                && let EntityId::Item(c) = target
                && props::is_class(w, target, "Server.Items.AddonComponent")
            {
                let Some(a) = muo_content::items::addon::component_addon(w, c) else {
                    return Ok(false);
                };
                target = EntityId::Item(a);
            }
            if !instance_of(w, target, t) {
                return Ok(false);
            }
        }
        let value = if ty.is_system_type {
            // `targeted.GetType()`
            let class = props::class_chain(w, target)
                .first()
                .copied()
                .unwrap_or_default();
            Value::Str(class.to_owned())
        } else {
            Value::Entity(Some(target))
        };
        let text = props::display(w, &def.kind, &value);
        muo_commands::logging::log_change_property(w, from, e, &prop.name, &text);
        (def.set.ok_or(())?)(w, e, value).map_err(|_| ())?;
        Ok(true)
    }

    fn assign_serial(
        &self,
        w: &mut World,
        from: MobileId,
        obj: &PropObj,
        prop: &PropInfo,
        ty: &PropType,
        serial: Serial,
    ) -> Result<Option<bool>, ()> {
        let (e, def) = (owner(w, obj).ok_or(())?, def_of(prop).ok_or(())?);
        let Some(target) = w.find_entity(serial) else {
            return Ok(None);
        };
        // `m_Type.IsInstanceOfType(toSet)`
        if !ty.is_system_type && !instance_of(w, target, &ty.name) {
            return Ok(Some(false));
        }
        let value = Value::Entity(Some(target));
        let text = props::display(w, &def.kind, &value);
        muo_commands::logging::log_change_property(w, from, e, &prop.name, &text);
        (def.set.ok_or(())?)(w, e, value).map_err(|_| ())?;
        Ok(Some(true))
    }
}

/// Props gump `TextDefinition` tests: the props gump gives a `TextDefinition` row the plain text editor
/// (`SetGump`), never the drill-in into the read-only `Number` / `String`, and the editor round trips.
#[cfg(test)]
mod tests {
    use super::*;
    use muo_io::TextDefinition;
    use muo_server::item::{ITEM_BASE, ItemType};
    use muo_server::mobile::AccessLevel;

    const GM: AccessLevel = AccessLevel::GameMaster;

    #[derive(Clone, Default)]
    struct Msg(Option<TextDefinition>);

    fn msg_of(w: &World, e: EntityId) -> Option<TextDefinition> {
        match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|i| i.ext.get::<Msg>())
                .and_then(|m| m.0.clone()),
            EntityId::Mobile(_) => None,
        }
    }

    static PROPS: [PropDef; 1] = [muo_content::text_def_prop!(
        "Message",
        GM,
        GM,
        |w: &World, e: EntityId| msg_of(w, e),
        Some(|w: &mut World, e: EntityId, v: Value| {
            if let EntityId::Item(i) = e {
                let m = match v {
                    Value::TextDef(t) => Some(t),
                    _ => None,
                };
                w.item_mut(i).unwrap().ext.insert(Msg(m));
            }
            Ok(())
        })
    )];
    static SKILL_TELEPORTER: ItemType = ItemType {
        full_name: "Server.Items.SkillTeleporter",
        parent: Some(&ITEM_BASE),
        ..ITEM_BASE
    };

    /// A staff member and a `SkillTeleporter` with `message`, and the `Message` row of its props list.
    fn setup(message: Option<TextDefinition>) -> (World, MobileId, PropObj, PropInfo) {
        let mut w = muo_server::map::test_support::world();
        muo_content::props::register_core(&mut w);
        w.types.register_item(&SKILL_TELEPORTER, &[]);
        muo_content::props::add(&mut w, "Server.Items.SkillTeleporter", &PROPS);
        let from = w.new_mobile(&muo_content::player_mobile::PLAYER_MOBILE);
        w.mobile_mut(from).unwrap().access_level = GM;
        let tp = w.new_item(&SKILL_TELEPORTER);
        w.item_mut(tp).unwrap().ext.insert(Msg(message));
        let obj = PropObj::Entity(EntityId::Item(tp));
        let prop = RegistryHost
            .build_list(&mut w, from, &obj)
            .into_iter()
            .find_map(|e| match e {
                ListEntry::Prop(p) if p.name == "Message" => Some(p),
                _ => None,
            })
            .expect("the Message row is visible");
        (w, from, obj, prop)
    }

    fn message(w: &World, obj: &PropObj) -> Option<TextDefinition> {
        msg_of(w, entity(obj).unwrap())
    }

    #[test]
    fn pressing_set_on_a_populated_text_definition_opens_the_editor() {
        let (mut w, from, obj, prop) = setup(Some(TextDefinition::of_number(1060847)));
        assert!(prop.can_modify(access_of(&w, from)));
        assert!(matches!(
            RegistryHost.kind(&mut w, &obj, &prop),
            PropKind::Text
        ));
    }

    // With no object to drill into, the old path re-sent the same page and looked inert.
    #[test]
    fn pressing_set_on_a_null_text_definition_opens_the_editor() {
        let (mut w, _, obj, prop) = setup(None);
        assert!(matches!(
            RegistryHost.kind(&mut w, &obj, &prop),
            PropKind::Text
        ));
    }

    // Numeric input always wins, so "0" is cliloc 0 (empty), never the string "0".
    #[test]
    fn editor_round_trips_cliloc_and_string() {
        for (entered, number, string) in [
            ("1060847", 1060847, None),
            ("0", 0, None),
            ("Hail, traveller.", 0, Some("Hail, traveller.")),
        ] {
            let (mut w, from, obj, prop) = setup(None);
            let value = RegistryHost.parse(&w, &prop, entered).unwrap();
            RegistryHost
                .set_direct(&mut w, from, &obj, &prop, &value)
                .unwrap();
            let m = message(&w, &obj).expect("a definition was stored");
            assert_eq!(m.number, number, "{entered}");
            assert_eq!(m.string.as_deref(), string, "{entered}");
        }
    }

    // The edit box is seeded from GetValue(), so a cliloc-looking string must survive the trip.
    #[test]
    fn editor_preserves_a_string_that_looks_like_a_cliloc() {
        let (mut w, from, obj, prop) = setup(Some(TextDefinition::of_text("1060847")));
        let shown = RegistryHost.text_value(&mut w, &obj, &prop);
        let value = RegistryHost.parse(&w, &prop, &shown).unwrap();
        RegistryHost
            .set_direct(&mut w, from, &obj, &prop, &value)
            .unwrap();
        let m = message(&w, &obj).unwrap();
        assert_eq!((m.number, m.string.as_deref()), (0, Some("1060847")));
    }

    #[test]
    fn editor_null_button_clears_the_value() {
        let (mut w, from, obj, prop) = setup(Some(TextDefinition::of_text("Hail, traveller.")));
        RegistryHost
            .set_direct(&mut w, from, &obj, &prop, &PropValue::Null)
            .unwrap();
        assert!(message(&w, &obj).is_none());
    }

    #[derive(Clone, Default)]
    struct Count(i64);

    static COUNT_PROPS: [PropDef; 1] = [PropDef {
        name: "Count",
        kind: Kind::Plain(ParsePropType::Numeric(
            muo_content::utilities::types::Numeric::Int,
        )),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => Value::Int(
                w.item(i)
                    .and_then(|i| i.ext.get::<Count>())
                    .map_or(0, |c| c.0),
            ),
            EntityId::Mobile(_) => Value::Null,
        },
        set: Some(|w, e, v| {
            if let (EntityId::Item(i), Value::Int(n)) = (e, v) {
                w.item_mut(i).unwrap().ext.insert(Count(n));
                Ok(())
            } else {
                Err("not an int".into())
            }
        }),
    }];

    #[derive(Clone, Default)]
    struct Link(Option<EntityId>);

    #[derive(Clone, Default)]
    struct Held(Option<EntityId>);

    static LINK_PROPS: [PropDef; 3] = [
        PropDef {
            name: "Addon",
            kind: Kind::Plain(ParsePropType::EntityOf("BaseAddon")),
            read: GM,
            write: GM,
            get: |w, e| match e {
                EntityId::Item(i) => Value::Entity(
                    w.item(i)
                        .and_then(|i| i.ext.get::<Link>())
                        .and_then(|l| l.0),
                ),
                EntityId::Mobile(_) => Value::Null,
            },
            set: Some(|w, e, v| {
                if let EntityId::Item(i) = e {
                    w.item_mut(i).unwrap().ext.insert(Link(v.entity()));
                }
                Ok(())
            }),
        },
        PropDef {
            name: "Owner",
            kind: Kind::Plain(ParsePropType::EntityOf("Mobile")),
            read: GM,
            write: GM,
            get: |_, _| Value::Null,
            set: Some(|_, _, _| Ok(())),
        },
        // declared `Mobile`, holding a value
        PropDef {
            name: "Holder",
            kind: Kind::Plain(ParsePropType::EntityOf("Mobile")),
            read: GM,
            write: GM,
            get: |w, e| match e {
                EntityId::Item(i) => Value::Entity(
                    w.item(i)
                        .and_then(|i| i.ext.get::<Held>())
                        .and_then(|l| l.0),
                ),
                EntityId::Mobile(_) => Value::Null,
            },
            set: Some(|w, e, v| {
                if let EntityId::Item(i) = e {
                    w.item_mut(i).unwrap().ext.insert(Held(v.entity()));
                }
                Ok(())
            }),
        },
    ];
    static OTHER_MOBILE: muo_server::mobile::MobileType = muo_server::mobile::MobileType {
        full_name: "Server.Mobiles.BaseCreature",
        parent: Some(&muo_server::mobile::MOBILE_BASE),
        ..muo_server::mobile::MOBILE_BASE
    };
    static FAKE_ADDON: ItemType = ItemType {
        full_name: "Server.Items.BaseAddon",
        parent: Some(&ITEM_BASE),
        ..ITEM_BASE
    };

    // `SetObjectTarget`: `m_Type.IsInstanceOfType(targeted)` decides. `m_Type` is the held value's class (`PropsGump`:
    // `obj?.GetType() ?? prop.PropertyType`), so a null property checks the declared class and a held one its own.
    #[test]
    fn set_object_target_checks_the_held_class() {
        let (mut w, from, obj, _) = setup(None);
        w.types.register_item(&FAKE_ADDON, &[]);
        muo_content::props::add(&mut w, "Server.Items.SkillTeleporter", &LINK_PROPS);
        let rows = RegistryHost.build_list(&mut w, from, &obj);
        let row = |n: &str| {
            rows.iter()
                .find_map(|e| match e {
                    ListEntry::Prop(p) if p.name == n => Some(p.clone()),
                    _ => None,
                })
                .unwrap()
        };
        // the gump's `Type`, as `kind` routes it
        let ty = |w: &mut World, p: &PropInfo| match RegistryHost.kind(w, &obj, p) {
            PropKind::Entity(t) => t,
            _ => panic!("an entity row"),
        };
        let addon = w.new_item(&FAKE_ADDON);
        let other = w.new_item(&SKILL_TELEPORTER);
        let try_set = |w: &mut World, p: &PropInfo, t: EntityId| {
            let ty = ty(w, p);
            RegistryHost
                .assign_targeted(w, from, &obj, p, &ty, Targeted::Entity(t))
                .unwrap()
        };
        assert!(try_set(&mut w, &row("Addon"), EntityId::Item(addon)));
        assert!(!try_set(&mut w, &row("Addon"), EntityId::Item(other)));
        assert!(!try_set(&mut w, &row("Addon"), EntityId::Mobile(from)));
        assert!(try_set(&mut w, &row("Owner"), EntityId::Mobile(from)));
        assert!(!try_set(&mut w, &row("Owner"), EntityId::Item(addon)));
        let Some(EntityId::Item(t)) = entity(&obj) else {
            panic!()
        };
        assert_eq!(
            w.item(t).unwrap().ext.get::<Link>().and_then(|l| l.0),
            Some(EntityId::Item(addon))
        );

        // D3: a `Mobile` property holding a PlayerMobile refuses a creature of another class
        let creature = w.new_mobile(&OTHER_MOBILE);
        w.item_mut(t)
            .unwrap()
            .ext
            .insert(Held(Some(EntityId::Mobile(from))));
        assert_eq!(ty(&mut w, &row("Holder")).name, "PlayerMobile");
        assert!(!try_set(&mut w, &row("Holder"), EntityId::Mobile(creature)));
        assert!(try_set(&mut w, &row("Holder"), EntityId::Mobile(from)));

        // D2: the serial path checks the class too, and nothing is assigned on refusal
        let serial = |e: EntityId| match e {
            EntityId::Item(i) => w.item(i).unwrap().serial,
            EntityId::Mobile(m) => w.mobile(m).unwrap().serial,
        };
        let (cs, as_) = (
            serial(EntityId::Mobile(creature)),
            serial(EntityId::Item(addon)),
        );
        let holder = row("Holder");
        let hty = ty(&mut w, &holder);
        let by_serial = |w: &mut World, s| {
            RegistryHost
                .assign_serial(w, from, &obj, &holder, &hty, s)
                .unwrap()
        };
        assert_eq!(by_serial(&mut w, cs), Some(false));
        assert_eq!(by_serial(&mut w, as_), Some(false));
        assert_eq!(
            w.item(t).unwrap().ext.get::<Held>().and_then(|l| l.0),
            Some(EntityId::Mobile(from))
        );
        assert_eq!(by_serial(&mut w, Serial::new(0x7fff_0000)), None);
    }

    /// `Types.TryParse`'s error is ignored: a number that does not parse is set as null, which reflection makes 0.
    #[test]
    fn set_gump_sets_zero_when_the_number_does_not_parse() {
        use crate::m_z::hooks::hooks_mut;
        use muo_content::gumps::base::{Gump, RelayInfo};
        let (mut w, from, obj, _) = setup(None);
        muo_content::props::add(&mut w, "Server.Items.SkillTeleporter", &COUNT_PROPS);
        hooks_mut(&mut w).props = Some(Rc::new(RegistryHost));
        let prop = RegistryHost
            .build_list(&mut w, from, &obj)
            .into_iter()
            .find_map(|e| match e {
                ListEntry::Prop(p) if p.name == "Count" => Some(p),
                _ => None,
            })
            .expect("the Count row is visible");
        let ctx = crate::m_z::props::PropertiesContext {
            mobile: from,
            object: Some(obj.clone()),
            stack: None,
            list: Rc::new(vec![]),
            page: 0,
            variant: None,
        };
        let count = |w: &World| match entity(&obj) {
            Some(EntityId::Item(i)) => w.item(i).and_then(|i| i.ext.get::<Count>()).map(|c| c.0),
            _ => None,
        };
        let mut g = crate::m_z::props::setters::new_set(
            &mut w,
            prop.clone(),
            from,
            obj.clone(),
            ctx.clone(),
        );
        g.on_response(
            &mut w,
            muo_server::net_state::NetStateId(1),
            &RelayInfo::new(1, vec![], vec![(0, "42".into())]),
        );
        assert_eq!(count(&w), Some(42));
        let mut g = crate::m_z::props::setters::new_set(&mut w, prop, from, obj.clone(), ctx);
        g.on_response(
            &mut w,
            muo_server::net_state::NetStateId(1),
            &RelayInfo::new(1, vec![], vec![(0, "abc".into())]),
        );
        assert_eq!(count(&w), Some(0));
    }
    #[derive(Clone, Default)]
    struct Bounds(muo_primitives::Rectangle2D);

    static RECT_PROPS: [PropDef; 1] = [PropDef {
        name: "Bounds",
        kind: Kind::Rect2D,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => Value::Rect(
                w.item(i)
                    .and_then(|i| i.ext.get::<Bounds>())
                    .map_or_else(Default::default, |b| b.0),
            ),
            EntityId::Mobile(_) => Value::Null,
        },
        set: Some(|w, e, v| {
            if let (EntityId::Item(i), Value::Rect(r)) = (e, v) {
                w.item_mut(i).unwrap().ext.insert(Bounds(r));
            }
            Ok(())
        }),
    }];

    // Rectangle2D is a [PropertyObject]: the gump drills into X, Y, Width and Height of the value, and a change is
    // written back to the owner's property (OnValueChanged).
    #[test]
    fn rectangle_drills_into_its_sub_properties_and_writes_back() {
        let (mut w, from, obj, _) = setup(None);
        muo_content::props::add(&mut w, "Server.Items.SkillTeleporter", &RECT_PROPS);
        crate::m_z::hooks::hooks_mut(&mut w).props = Some(Rc::new(RegistryHost));
        let Some(EntityId::Item(it)) = entity(&obj) else {
            panic!()
        };
        w.item_mut(it)
            .unwrap()
            .ext
            .insert(Bounds(muo_primitives::Rectangle2D::new(10, 20, 5, 7)));
        let find = |w: &mut World, o: &PropObj, n: &str| {
            RegistryHost
                .build_list(w, from, o)
                .into_iter()
                .find_map(|e| match e {
                    ListEntry::Prop(p) if p.name == n => Some(p),
                    _ => None,
                })
                .unwrap()
        };
        let bounds = find(&mut w, &obj, "Bounds");
        assert!(matches!(
            RegistryHost.kind(&mut w, &obj, &bounds),
            PropKind::PropertyObject
        ));
        let rect = RegistryHost.value_object(&mut w, &obj, &bounds).unwrap();
        // D4: `[NoSort]` rows follow declaration order
        let names: Vec<_> = RegistryHost
            .build_list(&mut w, from, &rect)
            .into_iter()
            .filter_map(|e| {
                if let ListEntry::Prop(p) = e {
                    Some(p.name)
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(names, ["Start", "End", "X", "Y", "Width", "Height"]);
        let (x, width) = (find(&mut w, &rect, "X"), find(&mut w, &rect, "Width"));
        assert_eq!(RegistryHost.value_to_string(&mut w, &rect, &x), "10 (0xA)");
        assert_eq!(
            RegistryHost.value_to_string(&mut w, &rect, &width),
            "5 (0x5)"
        );
        let v = RegistryHost.parse(&w, &width, "9").unwrap();
        RegistryHost
            .set_direct(&mut w, from, &rect, &width, &v)
            .unwrap();
        let ctx = crate::m_z::props::PropertiesContext {
            mobile: from,
            object: Some(rect.clone()),
            stack: Some(Rc::new(std::cell::RefCell::new(vec![
                crate::m_z::props::properties::StackEntry {
                    object: obj.clone(),
                    prop: bounds,
                },
            ]))),
            list: Rc::new(vec![]),
            page: 0,
            variant: None,
        };
        assert_eq!(
            RegistryHost.value_to_string(&mut w, &rect, &width),
            "9 (0x9)"
        );
        ctx.on_value_changed(&mut w, &rect, &width);
        let r = w.item(it).unwrap().ext.get::<Bounds>().unwrap().0;
        assert_eq!((r.start.x, r.end.x), (10, 19));
    }
}

/// `{o:X}` of a boxed number prints the type's own width: `(sbyte)-1` is `FF`, `-1` is `FFFFFFFF`.
fn hex_bits(kind: &Kind, n: i64) -> u64 {
    use muo_content::utilities::types::Numeric as N;
    let kind = if let Kind::Nullable(inner) = kind {
        inner
    } else {
        kind
    };
    match kind {
        Kind::Plain(ParsePropType::Numeric(N::SByte)) => u64::from(n as i8 as u8),
        Kind::Plain(ParsePropType::Numeric(N::Short)) => u64::from(n as i16 as u16),
        Kind::Plain(ParsePropType::Numeric(N::Int)) => u64::from(n as i32 as u32),
        _ => n as u64,
    }
}

#[cfg(test)]
mod hex_tests {
    use super::*;
    use muo_content::utilities::types::Numeric as N;

    #[test]
    fn a_negative_number_prints_at_its_own_width() {
        let k = |n| Kind::Plain(ParsePropType::Numeric(n));
        assert_eq!(hex_bits(&k(N::SByte), -1), 0xFF);
        assert_eq!(hex_bits(&k(N::Int), -1), 0xFFFF_FFFF);
        assert_eq!(hex_bits(&k(N::Long), -1), u64::MAX);
    }
}
