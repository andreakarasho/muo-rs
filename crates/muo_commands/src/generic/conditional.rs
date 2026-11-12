//! `ObjectConditional` and the `where`, `order by`, `distinct` and `limit` extensions.
//!
//! A condition is data evaluated per object:
//! [`Binding`] walks the property chain and [`Cond::test`] compares. The semantics are the compilers': a null link
//! in a chain is "no match" (also under `not`), relational operators need an ordering, equality is by value.
//!
//!
//! shortcut: a link's declared class is known only for the few entity properties `props::link_class` lists; for any
//! other entity link the next name is looked up in every registered table (`props::find_prop_any`), so a name
//! that exists nowhere is a parse error, but one that exists only on an unrelated class is accepted. Upgrade
//! by giving entity properties a declared class. `order by` and `distinct` sort with
//! `dotnet::net_sort`, which reproduces `List.Sort`'s order among ties.

use muo_primitives::dotnet::{IgnoreCase, InvariantCase};
use std::cmp::Ordering;

use muo_content::props::{self, Access, ClassInfo, Kind, PropDef, Value};
use muo_server::ids::EntityId;
use muo_server::mobile::AccessLevel;
use muo_server::world::World;

use super::Obj;

/// `Property`: a `a.b.c` binding with the viewer's access level.
#[derive(Clone)]
pub struct Binding {
    names: Vec<String>,
    /// One per link once bound; `[UNBOUND]` before.
    defs: Vec<&'static PropDef>,
    from: AccessLevel,
    access: Access,
}

impl Binding {
    /// `Property.BindTo` and `CheckAccess` over the class `class` (root first): every link is looked up at parse
    /// time, a link but the last needing read access.
    pub fn bind(
        w: &World,
        class: &ClassInfo,
        from: AccessLevel,
        path: &str,
        access: Access,
    ) -> Result<Binding, String> {
        Self::bind_in(w, class, from, path, access, false)
    }

    /// `bind` where a name no `[CommandProperty]` table has may also be a public property that `PUBLIC_ITEM` /
    /// `PUBLIC_MOBILE` lists (the Advanced Search resolver reads every public property, not only commands').
    pub fn bind_public(
        w: &World,
        class: &ClassInfo,
        from: AccessLevel,
        path: &str,
        access: Access,
    ) -> Result<Binding, String> {
        Self::bind_in(w, class, from, path, access, true)
    }

    fn bind_in(
        w: &World,
        class: &ClassInfo,
        from: AccessLevel,
        path: &str,
        access: Access,
        public: bool,
    ) -> Result<Binding, String> {
        let mut is_item = class.is_item;
        let names: Vec<String> = path.split('.').map(str::to_string).collect();
        let mut defs: Vec<&'static PropDef> = Vec::new();
        let mut checks: Vec<(Access, &'static PropDef)> = Vec::new();
        // Where the next link is looked up: the class chain, the sub-properties of an object, or anywhere.
        enum Next {
            Class(Vec<&'static str>),
            Subs(&'static [PropDef]),
            Any,
        }
        let mut next = Next::Class(class.chain.clone());
        for (i, name) in names.iter().enumerate() {
            let def = match &next {
                Next::Class(c) => props::props_of_class(w, c)
                    .into_iter()
                    .find(|d| d.name.eq_ic(name))
                    .or_else(|| {
                        let extra = if is_item { PUBLIC_ITEM } else { PUBLIC_MOBILE };
                        public
                            .then(|| extra.iter().find(|d| d.name.eq_ic(name)))
                            .flatten()
                    }),
                Next::Subs(s) => s.iter().find(|d| d.name.eq_ic(name)),
                Next::Any => props::find_prop_any(w, name),
            };
            let Some(def) = def else {
                return Err(format!("Property '{name}' not found."));
            };
            // `BindTo`: the whole chain is resolved first (a final write needs a setter); `CheckAccess` follows.
            let need = if i + 1 == names.len() {
                access
            } else {
                Access::READ
            };
            if need.write && def.set.is_none() {
                return Err("Property is read-only.".to_string());
            }
            checks.push((need, def));
            next = match props::subs_of(&def.kind) {
                Some(s) => Next::Subs(s),
                None => match props::link_class(def.name).and_then(|c| props::find_class(w, c)) {
                    Some(c) => {
                        is_item = c.is_item;
                        Next::Class(c.chain)
                    }
                    None => Next::Any,
                },
            };
            defs.push(def);
        }
        for (need, def) in checks {
            if need.read && from < def.read {
                return Err(format!(
                    "You must be at least {} to read this property.",
                    props::access_level_name(def.read)
                ));
            }
            // `WriteAccessException(this, from.AccessLevel, security.ReadLevel)` names the read level.
            if need.write && from < def.write {
                return Err(format!(
                    "You must be at least {} to write this property.",
                    props::access_level_name(def.read)
                ));
            }
        }
        Ok(Binding {
            names,
            defs,
            from,
            access,
        })
    }

    /// The type of the first link when the binding has one link (a constant is parsed against it).
    pub fn single_def(&self) -> Option<&'static PropDef> {
        (self.names.len() == 1).then(|| self.defs[0])
    }

    /// The last link: what the binding's value is.
    pub(crate) fn last_def(&self) -> &'static PropDef {
        self.defs[self.defs.len() - 1]
    }

    /// `Property is not comparable` when the value's type has no `CompareTo` (`System.Type`, `Point2D`).
    pub fn check_comparable(&self) -> Result<(), String> {
        use muo_content::utilities::types::PropType as T;
        match &self.last_def().kind {
            Kind::Plain(T::Type | T::IPoint2D)
            | Kind::Rect2D
            | Kind::Rect3D
            | Kind::Poison
            | Kind::TextDef(_)
            | Kind::Object(..) => Err("Property is not comparable.".into()),
            _ => Ok(()),
        }
    }

    /// The chain's value on `e`; `None` when a link is null, unreadable or missing.
    pub fn eval(&self, w: &mut World, e: EntityId) -> Option<(Value, &'static PropDef)> {
        let mut cur = e;
        let mut def = self.defs[0];
        // The previous link returned an object: this link is one of its sub-properties, bound at parse time.
        let mut in_object = false;
        for (i, name) in self.names.iter().enumerate() {
            if i > 0 {
                def = if in_object {
                    self.defs[i]
                } else {
                    props::find_prop(w, cur, name)?
                };
                if self.from < def.read {
                    return None;
                }
            }
            let v = (def.get)(w, cur);
            if i + 1 == self.names.len() {
                return Some((v, def));
            }
            match v {
                Value::Entity(Some(next)) => {
                    cur = next;
                    in_object = false;
                }
                Value::Null | Value::Entity(None) => return None,
                v => {
                    props::bind_parent(w, &v);
                    in_object = true
                }
            }
        }
        None
    }

    /// `ChainOrDefault`: an unreadable link reads as the type's default.
    pub fn eval_or_default(&self, w: &mut World, e: EntityId) -> Value {
        match self.eval(w, e) {
            Some((v, _)) => v,
            None => default_for(&self.last_def().kind),
        }
    }

    pub fn name(&self) -> String {
        self.names.join(".")
    }

    /// The links' declared names (`chain[j].Name`), as the properties are spelled, not as typed.
    pub fn canonical_name(&self) -> String {
        self.defs
            .iter()
            .map(|d| d.name)
            .collect::<Vec<_>>()
            .join(".")
    }

    pub fn access(&self) -> Access {
        self.access
    }
}

fn default_for(k: &Kind) -> Value {
    use muo_content::utilities::types::PropType as T;
    match k {
        Kind::Plain(T::Numeric(_)) => Value::Int(0),
        Kind::Plain(T::Float | T::Double | T::Decimal) => Value::Float(0.0),
        Kind::Plain(T::Bool) => Value::Bool(false),
        Kind::Plain(T::Enum(_)) => Value::Enum(0),
        Kind::TimeSpan => Value::TimeSpan(muo_primitives::TimeSpan::ZERO),
        _ => Value::Null,
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
enum StrOp {
    Equal,
    NotEqual,
    Contains,
    StartsWith,
    EndsWith,
}

#[derive(Copy, Clone, Eq, PartialEq)]
enum CmpOp {
    Equal,
    NotEqual,
    Greater,
    GreaterEqual,
    Lesser,
    LesserEqual,
}

enum Test {
    Str {
        op: StrOp,
        ignore_case: bool,
        value: String,
    },
    Cmp {
        op: CmpOp,
        value: Constant,
    },
    /// A floating point comparison within a tolerance (`AdvancedSearchConditions.EpsilonCondition`).
    Eps {
        op: CmpOp,
        value: f64,
        eps: f64,
    },
}

/// The right side: parsed once when the binding has one link, else per object against the last link's type.
enum Constant {
    Parsed(Value),
    Text(String),
}

pub struct Cond {
    binding: Binding,
    not: bool,
    test: Test,
}

fn parse_constant(w: &World, def: &PropDef, text: &str) -> Result<Value, String> {
    // `where` spells null as a bare `null`, not [set's (-null-), so it precedes the parser.
    let nullable = matches!(
        &def.kind,
        Kind::Text
            | Kind::Map
            | Kind::Poison
            | Kind::TextDef(_)
            | Kind::Object(..)
            | Kind::Nullable(_)
            | Kind::Plain(muo_content::utilities::types::PropType::Str)
            | Kind::Plain(
                muo_content::utilities::types::PropType::Entity
                    | muo_content::utilities::types::PropType::EntityOf(_)
            )
    );
    if text == "null" && nullable {
        return Ok(Value::Null);
    }
    props::parse_for(w, def, text)
}

impl Cond {
    /// `PropertyCondition.Guarded` around the test.
    fn test(&self, w: &mut World, e: EntityId) -> bool {
        let Some((v, def)) = self.binding.eval(w, e) else {
            return false;
        };
        let result = match &self.test {
            Test::Str {
                op,
                ignore_case,
                value,
            } => {
                let s = match &v {
                    Value::Str(s) | Value::Text(s) => Some(s.as_str()),
                    _ => None,
                };
                let (a, b) = match s {
                    Some(s) if *ignore_case => (Some(s.lower_inv()), value.lower_inv()),
                    Some(s) => (Some(s.to_string()), value.clone()),
                    None => (None, value.clone()),
                };
                match op {
                    StrOp::Equal => a.as_deref() == Some(b.as_str()),
                    StrOp::NotEqual => a.as_deref() != Some(b.as_str()),
                    StrOp::Contains => a.is_some_and(|a| a.contains(&b)),
                    StrOp::StartsWith => a.is_some_and(|a| a.starts_with(&b)),
                    StrOp::EndsWith => a.is_some_and(|a| a.ends_with(&b)),
                }
            }
            Test::Eps { op, value, eps } => {
                let v = v.f64();
                let d = (v - value).abs();
                match op {
                    CmpOp::Equal => d < *eps,
                    CmpOp::NotEqual => d >= *eps,
                    CmpOp::Greater => v > value + eps,
                    CmpOp::Lesser => v < value - eps,
                    CmpOp::GreaterEqual => v >= value - eps,
                    CmpOp::LesserEqual => v <= value + eps,
                }
            }
            Test::Cmp { op, value } => {
                let parsed;
                let c = match value {
                    Constant::Parsed(c) => c,
                    Constant::Text(t) => match parse_constant(w, def, t) {
                        Ok(c) => {
                            parsed = c;
                            &parsed
                        }
                        Err(_) => return false,
                    },
                };
                // Nullable<T> is lifted: an unset value is equal to null only, and no relation holds with it.
                if matches!(def.kind, Kind::Nullable(_)) && (v.is_null() || c.is_null()) {
                    let same = v.is_null() && c.is_null();
                    return match op {
                        CmpOp::Equal => same,
                        CmpOp::NotEqual => !same,
                        _ => false,
                    } != self.not;
                }
                match v.compare(c) {
                    Some(o) => match op {
                        CmpOp::Equal => o == Ordering::Equal,
                        CmpOp::NotEqual => o != Ordering::Equal,
                        CmpOp::Greater => o == Ordering::Greater,
                        CmpOp::GreaterEqual => o != Ordering::Less,
                        CmpOp::Lesser => o == Ordering::Less,
                        CmpOp::LesserEqual => o != Ordering::Greater,
                    },
                    None => match op {
                        CmpOp::Equal => v == *c,
                        CmpOp::NotEqual => v != *c,
                        _ => false,
                    },
                }
            }
        };
        result != self.not
    }
}

/// `ObjectConditional`.
pub struct ObjectConditional {
    /// `None` is `Empty`: no `where`, everything matches.
    pub class: Option<ClassInfo>,
    groups: Vec<Vec<Cond>>,
    /// The first error `Compile` would throw (a bad constant, an operator the property can't take): reported by
    /// `WhereExtension.Optimize`, after every extension was parsed. `parse_direct` reports it at once.
    deferred: Option<String>,
}

impl ObjectConditional {
    pub fn empty() -> ObjectConditional {
        ObjectConditional {
            class: None,
            groups: Vec::new(),
            deferred: None,
        }
    }

    /// `IsItem`: no type, or one that derives from `Item`.
    pub fn is_item(&self) -> bool {
        self.class.as_ref().is_none_or(|c| c.is_item)
    }

    pub fn is_mobile(&self) -> bool {
        self.class.as_ref().is_none_or(|c| !c.is_item)
    }

    /// `CheckCondition`.
    pub fn check(&self, w: &mut World, obj: Obj) -> bool {
        let Some(class) = &self.class else {
            return true;
        };
        let muo_server::target::Targeted::Entity(e) = obj else {
            return false;
        };
        if !props::is_class(w, e, class.full_name) {
            return false;
        }
        self.groups.iter().any(|g| g.iter().all(|c| c.test(w, e)))
    }

    /// `ObjectConditional.Parse`: cuts the arguments at `where` and parses what follows.
    pub fn parse(
        w: &World,
        from: AccessLevel,
        args: &mut Vec<String>,
    ) -> Result<ObjectConditional, String> {
        let mut cond_args: Option<Vec<String>> = None;
        if let Some(i) = args.iter().position(|a| a.eq_ic("where")) {
            cond_args = Some(args.split_off(i + 1));
            args.truncate(i);
        }
        match cond_args {
            Some(c) if !c.is_empty() => Self::parse_direct(w, from, &c),
            _ => Ok(Self::empty()),
        }
    }

    /// `ParseDirect`: `<Type> [not] <binding> <op> <value> [or ...]`.
    pub fn parse_direct(
        w: &World,
        from: AccessLevel,
        args: &[String],
    ) -> Result<ObjectConditional, String> {
        let mut c = Self::parse_deferred(w, from, args)?;
        match c.deferred.take() {
            Some(e) => Err(e),
            None => Ok(c),
        }
    }

    /// `ParseDirect` alone: what `Compile` rejects stays in `deferred`.
    fn parse_deferred(
        w: &World,
        from: AccessLevel,
        args: &[String],
    ) -> Result<ObjectConditional, String> {
        let mut deferred: Option<String> = None;
        if args.is_empty() {
            return Ok(Self::empty());
        }
        let Some(class) = props::find_class(w, &args[0]) else {
            return Err(format!("No type with that name ({}) was found.", args[0]));
        };
        let mut groups: Vec<Vec<Cond>> = Vec::new();
        let mut current: Vec<Cond> = Vec::new();
        let mut index = 1;
        let size = args.len();
        let improper = || "Improperly formatted object conditional.".to_string();

        while index < size {
            let cur = &args[index];
            let mut inverse = false;
            if cur.eq_ic("not") || cur == "!" {
                inverse = true;
                index += 1;
                if index >= size {
                    return Err(improper());
                }
            } else if cur.eq_ic("or") || cur == "||" {
                if !current.is_empty() {
                    groups.push(std::mem::take(&mut current));
                }
                index += 1;
                continue;
            }
            let binding = &args[index];
            index += 1;
            if index >= size {
                return Err(improper());
            }
            let oper = &args[index];
            index += 1;
            if index >= size {
                return Err(improper());
            }
            let val = &args[index];
            index += 1;

            let b = Binding::bind(w, &class, from, binding, Access::READ)?;
            let tested = match oper.as_str() {
                "=" | "==" | "is" => cmp(w, &b, CmpOp::Equal, val),
                "!=" => cmp(w, &b, CmpOp::NotEqual, val),
                ">" => cmp(w, &b, CmpOp::Greater, val),
                "<" => cmp(w, &b, CmpOp::Lesser, val),
                ">=" => cmp(w, &b, CmpOp::GreaterEqual, val),
                "<=" => cmp(w, &b, CmpOp::LesserEqual, val),
                "==~" | "~==" | "=~" | "~=" | "is~" | "~is" => strc(&b, StrOp::Equal, val, true),
                "!=~" | "~!=" => strc(&b, StrOp::NotEqual, val, true),
                "starts" => strc(&b, StrOp::StartsWith, val, false),
                "starts~" | "~starts" => strc(&b, StrOp::StartsWith, val, true),
                "ends" => strc(&b, StrOp::EndsWith, val, false),
                "ends~" | "~ends" => strc(&b, StrOp::EndsWith, val, true),
                "contains" => strc(&b, StrOp::Contains, val, false),
                "contains~" | "~contains" => strc(&b, StrOp::Contains, val, true),
                other => return Err(format!("Unrecognized operator (\"{other}\").")),
            };
            let test = match tested {
                Ok(t) => t,
                Err(e) => {
                    deferred.get_or_insert(e);
                    Test::Cmp {
                        op: CmpOp::Equal,
                        value: Constant::Text(String::new()),
                    }
                }
            };
            current.push(Cond {
                binding: b,
                not: inverse,
                test,
            });
        }
        groups.push(current);
        Ok(ObjectConditional {
            class: Some(class),
            groups,
            deferred,
        })
    }
}

fn cmp(w: &World, b: &Binding, op: CmpOp, val: &str) -> Result<Test, String> {
    // A struct with no `CompareTo` (`Rectangle2D`) answers `==` and `!=` only: the compiler rejects the rest.
    if !matches!(op, CmpOp::Equal | CmpOp::NotEqual)
        && matches!(b.last_def().kind, Kind::Rect2D | Kind::Rect3D)
    {
        return Err("Property is not comparable.".into());
    }
    let value = match b.single_def() {
        Some(def) => Constant::Parsed(parse_constant(w, def, val)?),
        None => Constant::Text(val.to_string()),
    };
    Ok(Test::Cmp { op, value })
}

fn strc(b: &Binding, op: StrOp, val: &str, ignore_case: bool) -> Result<Test, String> {
    // `StringCondition.Build` rejects non-string properties.
    if let Some(def) = b.single_def()
        && !matches!(
            &def.kind,
            Kind::Plain(muo_content::utilities::types::PropType::Str)
        )
    {
        return Err("String operators require a string property.".into());
    }
    Ok(Test::Str {
        op,
        ignore_case,
        value: string_literal(val),
    })
}

// ---- extensions -----------------------------------------------------------------------------------------------

/// `BaseExtension`: `where` (order 20), `distinct` (30), `order by` (40), `limit` (80).
pub enum Extension {
    Where(ObjectConditional),
    Distinct(Vec<Binding>),
    Sort(Vec<(Binding, bool)>),
    Limit(usize),
}

impl Extension {
    fn order(&self) -> i32 {
        match self {
            Extension::Where(_) => 20,
            Extension::Distinct(_) => 30,
            Extension::Sort(_) => 40,
            Extension::Limit(_) => 80,
        }
    }
}

/// `Extensions`.
#[derive(Default)]
pub struct Extensions(pub Vec<Extension>);

fn ext_info(name: &str) -> Option<(&'static str, i32)> {
    // (name, fixed argument count or -1)
    match name.lower_inv().as_str() {
        "where" => Some(("where", -1)),
        "distinct" => Some(("distinct", -1)),
        "order" => Some(("order", -1)),
        "limit" => Some(("limit", 1)),
        _ => None,
    }
}

impl Extensions {
    /// `Extensions.Parse`: scans from the end for modifier names, each taking the arguments after it.
    pub fn parse(
        w: &World,
        from: AccessLevel,
        args: &mut Vec<String>,
    ) -> Result<Extensions, String> {
        let mut parsed: Vec<Extension> = Vec::new();
        let mut size = args.len();
        let mut base: Option<ClassInfo> = None;
        let mut i = args.len();
        while i > 0 {
            i -= 1;
            let Some((name, fixed)) = ext_info(&args[i]) else {
                continue;
            };
            if fixed >= 0 && i as i64 != size as i64 - i64::from(fixed) - 1 {
                return Err("Invalid extended argument count.".into());
            }
            let rest = &args[i + 1..size];
            let ext = match name {
                "where" => {
                    if rest.is_empty() {
                        return Err("Invalid condition syntax.".into());
                    }
                    let c = ObjectConditional::parse_deferred(w, from, rest)?;
                    base = c.class.clone();
                    Extension::Where(c)
                }
                "limit" => {
                    let n = muo_primitives::parse::to_i32(&rest[0]);
                    if n < 0 {
                        return Err("Limit cannot be less than zero.".into());
                    }
                    Extension::Limit(n as usize)
                }
                "distinct" => {
                    if rest.is_empty() {
                        return Err("Invalid distinction syntax.".into());
                    }
                    Extension::Distinct(
                        rest.iter()
                            .map(|r| (r.clone(), true))
                            .map(|(r, _)| pending(r))
                            .collect(),
                    )
                }
                _ => parse_order(rest)?,
            };
            parsed.push(ext);
            size = i;
        }
        muo_primitives::dotnet::net_sort(&mut parsed, |a, b| a.order().cmp(&b.order())); // List.Sort
        // Optimize: bind distinct and order properties against the where class.
        for ext in &mut parsed {
            match ext {
                Extension::Where(c) => {
                    if let Some(e) = c.deferred.take() {
                        return Err(e); // WhereExtension.Optimize: Conditional.Compile()
                    }
                }
                Extension::Distinct(list) => {
                    let Some(class) = &base else {
                        return Err("Distinct extension may only be used in combination with an object conditional.".into());
                    };
                    let mut out = Vec::new();
                    for b in list.iter() {
                        let bound = Binding::bind(w, class, from, &b.name(), Access::READ)?;
                        // DistinctCompiler orders through SortCompiler.Build, which throws for a type with no CompareTo.
                        bound.check_comparable()?;
                        out.push(bound);
                    }
                    *list = out;
                }
                Extension::Sort(list) => {
                    let Some(class) = &base else {
                        return Err("The ordering extension may only be used in combination with an object conditional.".into());
                    };
                    let mut out = Vec::new();
                    for (b, asc) in list.iter() {
                        let bound = Binding::bind(w, class, from, &b.name(), Access::READ)?;
                        bound.check_comparable()?;
                        out.push((bound, *asc));
                    }
                    *list = out;
                }
                _ => {}
            }
        }
        args.truncate(size);
        Ok(Extensions(parsed))
    }

    /// `IsValid`.
    pub fn is_valid(&self, w: &mut World, obj: Obj) -> bool {
        self.0.iter().all(|e| match e {
            Extension::Where(c) => c.check(w, obj),
            _ => true,
        })
    }

    /// `Filter`: distinct, order by, limit, in that order.
    pub fn filter(&self, w: &mut World, list: &mut Vec<Obj>) {
        for ext in &self.0 {
            match ext {
                Extension::Where(_) => {}
                Extension::Limit(n) => list.truncate(*n),
                Extension::Sort(orders) => {
                    muo_primitives::dotnet::net_sort(list, |a, b| compare_by(w, orders, *a, *b));
                }
                Extension::Distinct(props) => {
                    let orders: Vec<(Binding, bool)> =
                        props.iter().map(|b| (b.clone(), true)).collect();
                    let mut copy = list.clone();
                    muo_primitives::dotnet::net_sort(&mut copy, |a, b| {
                        compare_by(w, &orders, *a, *b)
                    });
                    list.clear();
                    let mut last: Option<Obj> = None;
                    for o in copy {
                        if last.is_none_or(|l| compare_by(w, &orders, o, l) != Ordering::Equal) {
                            list.push(o);
                            last = Some(o);
                        }
                    }
                }
            }
        }
    }
}

/// A binding that has not been bound yet (the name only), until `Optimize`.
fn pending(name: String) -> Binding {
    Binding {
        names: vec![name],
        defs: vec![&UNBOUND],
        from: AccessLevel::Player,
        access: Access::READ,
    }
}

static UNBOUND: PropDef = PropDef {
    name: "",
    kind: Kind::Text,
    read: AccessLevel::Player,
    write: AccessLevel::Player,
    get: |_, _| Value::Null,
    set: None,
};

fn parse_order(rest: &[String]) -> Result<Extension, String> {
    let bad = || "Invalid ordering syntax.".to_string();
    let mut offset = 0;
    if rest.is_empty() {
        return Err(bad());
    }
    if rest[0].eq_ic("by") {
        offset = 1;
        if rest.len() < 2 {
            return Err(bad());
        }
    }
    let mut orders = Vec::new();
    while offset < rest.len() {
        let binding = rest[offset].clone();
        offset += 1;
        let mut asc = true;
        if let Some(next) = rest.get(offset) {
            match next.lower_inv().as_str() {
                "+" | "up" | "asc" | "ascending" => {
                    offset += 1;
                }
                "-" | "down" | "desc" | "descending" => {
                    asc = false;
                    offset += 1;
                }
                _ => {}
            }
        }
        orders.push((pending(binding), asc));
    }
    Ok(Extension::Sort(orders))
}

pub(crate) fn compare_by(w: &mut World, orders: &[(Binding, bool)], a: Obj, b: Obj) -> Ordering {
    let (muo_server::target::Targeted::Entity(ea), muo_server::target::Targeted::Entity(eb)) =
        (a, b)
    else {
        return Ordering::Equal;
    };
    for (binding, asc) in orders {
        let (va, vb) = (
            binding.eval_or_default(w, ea),
            binding.eval_or_default(w, eb),
        );
        let o = va.compare(&vb).unwrap_or(Ordering::Equal);
        if o != Ordering::Equal {
            return if *asc { o } else { o.reverse() };
        }
    }
    Ordering::Equal
}

// ---- AdvancedSearchConditions ---------------------------------------------------------------------------------

impl Cond {
    /// Whether `e` passes this leaf (`PropertyCondition.Guarded`: a null link is no match, also under `not`).
    pub fn matches(&self, w: &mut World, e: EntityId) -> bool {
        self.test(w, e)
    }
}

/// `Types.TryParse(typeof(string), ..)`: `@"text"` is the text inside, any other value is itself.
fn string_literal(value: &str) -> String {
    match value.strip_prefix("@\"").and_then(|v| v.strip_suffix('"')) {
        Some(inner) if value.len() >= 3 => inner.to_string(),
        _ => value.to_string(),
    }
}

macro_rules! public_prop {
    ($name:literal, $kind:expr, $variant:ident, |$w:ident, $x:ident| $get:expr) => {
        PropDef {
            name: $name,
            kind: $kind,
            read: AccessLevel::Player,
            write: AccessLevel::Owner,
            get: |$w, e| match e {
                EntityId::$variant($x) => $get,
                _ => Value::Null,
            },
            set: None,
        }
    };
}

const P_BOOL: Kind = Kind::Plain(muo_content::utilities::types::PropType::Bool);
const P_INT: Kind = Kind::Plain(muo_content::utilities::types::PropType::Numeric(
    muo_content::utilities::types::Numeric::Int,
));
const P_STR: Kind = Kind::Plain(muo_content::utilities::types::PropType::Str);

/// Public readable `Item` properties that are not `[CommandProperty]` and that a vtable entry or field gives: what
/// the Advanced Search resolver reaches by reflection and the command tables do not list.
pub static PUBLIC_ITEM: &[PropDef] = &[
    public_prop!("Deleted", P_BOOL, Item, |w, i| Value::Bool(
        w.item(i).is_none_or(|x| x.deleted())
    )),
    public_prop!("DefaultName", P_STR, Item, |w, i| w
        .item(i)
        .and_then(|x| (x.ty.default_name)(w, i))
        .map_or(Value::Null, Value::Str)),
    public_prop!("PhysicalResistance", P_INT, Item, |w, i| Value::Int(
        i64::from(w.item(i).map_or(0, |x| (x.ty.physical_resistance)(w, i)))
    )),
    public_prop!("FireResistance", P_INT, Item, |w, i| Value::Int(i64::from(
        w.item(i).map_or(0, |x| (x.ty.fire_resistance)(w, i))
    ))),
    public_prop!("ColdResistance", P_INT, Item, |w, i| Value::Int(i64::from(
        w.item(i).map_or(0, |x| (x.ty.cold_resistance)(w, i))
    ))),
    public_prop!("PoisonResistance", P_INT, Item, |w, i| Value::Int(
        i64::from(w.item(i).map_or(0, |x| (x.ty.poison_resistance)(w, i)))
    )),
    public_prop!("EnergyResistance", P_INT, Item, |w, i| Value::Int(
        i64::from(w.item(i).map_or(0, |x| (x.ty.energy_resistance)(w, i)))
    )),
    public_prop!("DisplayWeight", P_BOOL, Item, |w, i| Value::Bool(
        w.item(i).is_some_and(|x| (x.ty.display_weight)(w, i))
    )),
    public_prop!("Nontransferable", P_BOOL, Item, |w, i| Value::Bool(
        w.item(i).is_some_and(|x| (x.ty.nontransferable)(w, i))
    )),
    public_prop!("BlocksFit", P_BOOL, Item, |w, i| Value::Bool(
        w.item(i).is_some_and(|x| (x.ty.blocks_fit)(w, i))
    )),
    public_prop!("IsVirtualItem", P_BOOL, Item, |w, i| Value::Bool(
        w.item(i).is_some_and(|x| (x.ty.is_virtual_item)(w, i))
    )),
    public_prop!("HandlesOnSpeech", P_BOOL, Item, |w, i| Value::Bool(
        w.item(i).is_some_and(|x| (x.ty.handles_on_speech)(w, i))
    )),
    public_prop!("HandlesOnMovement", P_BOOL, Item, |w, i| Value::Bool(
        w.item(i).is_some_and(|x| (x.ty.handles_on_movement)(w, i))
    )),
    public_prop!("SkipSerialization", P_BOOL, Item, |w, i| Value::Bool(
        w.item(i).is_some_and(|x| (x.ty.skip_serialization)(w, i))
    )),
];

/// The same for `Mobile`.
pub static PUBLIC_MOBILE: &[PropDef] = &[
    public_prop!("Deleted", P_BOOL, Mobile, |w, m| Value::Bool(
        w.mobile(m).is_none_or(|x| x.deleted)
    )),
    public_prop!("Murderer", P_BOOL, Mobile, |w, m| Value::Bool(
        w.mobile(m).is_some_and(|x| (x.ty.murderer)(w, m))
    )),
    public_prop!("MaxWeight", P_INT, Mobile, |w, m| Value::Int(i64::from(
        w.mobile(m).map_or(0, |x| (x.ty.max_weight)(w, m))
    ))),
    public_prop!("Luck", P_INT, Mobile, |w, m| Value::Int(i64::from(
        w.mobile(m).map_or(0, |x| (x.ty.luck)(w, m))
    ))),
    public_prop!("HasTrade", P_BOOL, Mobile, |w, m| Value::Bool(
        muo_server::mobile::part1::props_b::has_trade(w, m)
    )),
    public_prop!("KeepsItemsOnDeath", P_BOOL, Mobile, |w, m| Value::Bool(
        w.mobile(m)
            .is_some_and(|x| (x.ty.keeps_items_on_death)(w, m))
    )),
    public_prop!("IsDeadBondedPet", P_BOOL, Mobile, |w, m| Value::Bool(
        w.mobile(m).is_some_and(|x| (x.ty.is_dead_bonded_pet)(w, m))
    )),
    public_prop!("CanRegenHits", P_BOOL, Mobile, |w, m| Value::Bool(
        w.mobile(m).is_some_and(|x| (x.ty.can_regen_hits)(w, m))
    )),
    public_prop!("RegenThroughPoison", P_BOOL, Mobile, |w, m| Value::Bool(
        w.mobile(m)
            .is_some_and(|x| (x.ty.regen_through_poison)(w, m))
    )),
    public_prop!("SkipSerialization", P_BOOL, Mobile, |w, m| Value::Bool(
        w.mobile(m).is_some_and(|x| (x.ty.skip_serialization)(w, m))
    )),
];

/// `AdvancedSearchUtilities.CalculateEpsilon`.
pub fn calculate_epsilon(value: &str) -> f64 {
    match value.find('.') {
        None => 1e-10,
        Some(dot) => match value.chars().count() - dot - 1 {
            0..=9 => 1e-10,
            10 => 1e-11,
            11 => 1e-12,
            12 => 1e-13,
            13 => 1e-14,
            14 => 1e-15,
            _ => 1e-16,
        },
    }
}

fn map_cmp_operator(op: &str) -> Option<CmpOp> {
    Some(match op {
        "=" | "==" => CmpOp::Equal,
        "!" | "!=" => CmpOp::NotEqual,
        ">" => CmpOp::Greater,
        "<" => CmpOp::Lesser,
        ">=" => CmpOp::GreaterEqual,
        "<=" => CmpOp::LesserEqual,
        _ => return None,
    })
}

/// One leaf of the Advanced Search property test (`AdvancedSearchConditions.Leaf`): `[~]name op value` over the
/// properties of `class`, resolved case-insensitively with no access check (the command is Administrator's). `None`
/// is "no match" for the leaf.
pub fn search_leaf(
    w: &World,
    class: &ClassInfo,
    negate: bool,
    op: &str,
    name: &str,
    value: &str,
) -> Option<Cond> {
    use muo_content::utilities::types::PropType as T;
    // `Resolve` trims every link of a dotted name.
    let name = name.split('.').map(str::trim).collect::<Vec<_>>().join(".");
    let binding = Binding::bind_public(w, class, AccessLevel::Owner, &name, Access::READ).ok()?;
    let def = binding.last_def();
    let single = binding.single_def();
    let test = match &def.kind {
        Kind::Plain(T::Str) => {
            let (str_op, ignore_case) = match op {
                "=" | "==" => (StrOp::Equal, false),
                "!" | "!=" => (StrOp::NotEqual, false),
                ">" => (StrOp::StartsWith, false),
                "<" => (StrOp::EndsWith, false),
                "~" => (StrOp::Contains, false),
                "~>" => (StrOp::StartsWith, true),
                "~<" => (StrOp::EndsWith, true),
                "~~" => (StrOp::Contains, true),
                "~=" => (StrOp::Equal, true),
                "~!" => (StrOp::NotEqual, true),
                _ => return None,
            };
            // `null` is the null string for equality; for the substring operators it is the four-letter word.
            if value == "null" && matches!(str_op, StrOp::Equal | StrOp::NotEqual) {
                let cmp = if str_op == StrOp::Equal {
                    CmpOp::Equal
                } else {
                    CmpOp::NotEqual
                };
                Test::Cmp {
                    op: cmp,
                    value: Constant::Parsed(Value::Null),
                }
            } else {
                Test::Str {
                    op: str_op,
                    ignore_case,
                    value: string_literal(value),
                }
            }
        }
        Kind::Plain(T::Float | T::Double) => {
            let cmp = map_cmp_operator(op)?;
            // A float property parses its value as a float first, so the widened constant carries the float's rounding.
            let parsed: f64 = if matches!(def.kind, Kind::Plain(T::Float)) {
                f64::from(value.trim().parse::<f32>().ok()?)
            } else {
                value.trim().parse().ok()?
            };
            Test::Eps {
                op: cmp,
                value: parsed,
                eps: calculate_epsilon(value),
            }
        }
        kind => {
            let cmp = map_cmp_operator(op)?;
            // A relational operator on a type with no CompareTo is "no match" for the leaf.
            if !matches!(cmp, CmpOp::Equal | CmpOp::NotEqual)
                && matches!(kind, Kind::Plain(T::Type | T::IPoint2D))
            {
                return None;
            }
            let constant = if matches!(kind, Kind::Plain(T::Bool)) {
                if !matches!(cmp, CmpOp::Equal | CmpOp::NotEqual) {
                    return None;
                }
                // The gump accepts the switch words as well as the literals.
                let b = match value.lower_inv().as_str() {
                    "true" | "1" | "enabled" | "on" => true,
                    "false" | "0" | "disabled" | "off" => false,
                    _ => return None,
                };
                Constant::Parsed(Value::Bool(b))
            } else {
                match single {
                    Some(d) => Constant::Parsed(parse_constant(w, d, value).ok()?),
                    None => Constant::Parsed(parse_constant(w, def, value).ok()?),
                }
            };
            Test::Cmp {
                op: cmp,
                value: constant,
            }
        }
    };
    Some(Cond {
        binding,
        not: negate,
        test,
    })
}
