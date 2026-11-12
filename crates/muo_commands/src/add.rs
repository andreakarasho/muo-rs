//! The `Add` commands (`[add`, `[addtopack`, `[tile`, `[outline`..) and the
//! `AddCommand` / `AddToPackCommand` generic commands.
//!
//! Constructor overloads come from `muo_content::props::add_ctors` (the constructor overload tables); a
//! class with no table is built with its parameterless `construct`.
//! The menus `[add` opens with no arguments, or when nothing matches, are `add_menus.rs`.

use muo_primitives::dotnet::IgnoreCase;
use std::rc::Rc;
use std::time::Instant;

use muo_content::props::{self, Access, PropDef};
use muo_primitives::Point3D;
use muo_server::commands::{CommandEventArgs, CommandSystem};
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::mobile::AccessLevel;
use muo_server::mobile::part2::messages::send_message;
use muo_server::target::Targeted;
use muo_server::type_registry::EntityType;
use muo_server::world::World;

use crate::generic::implementors::begin_bounding_box;
use crate::generic::{BaseCommand, CmdSpec, CommandSupport as S, Obj, ObjectTypes as O, Run};
use crate::hooks::CommandHooks;
use crate::logging;
use crate::properties::internal_set_value;

pub fn configure(w: &mut World) {
    for (name, outline) in [("Tile", false), ("Outline", true)] {
        let cmd = |suffix: &str| format!("{name}{suffix}");
        #[allow(clippy::type_complexity)] // the handler type, spelled out
        let h = |f: fn(&mut World, &mut CommandEventArgs, bool)| -> Rc<dyn Fn(&mut World, &mut CommandEventArgs)> {
            Rc::new(move |w, e| f(w, e, outline))
        };
        CommandSystem::register(w, name, AccessLevel::GameMaster, &[], h(box_on_command));
        CommandSystem::register(
            w,
            &cmd("RXYZ"),
            AccessLevel::GameMaster,
            &[],
            h(rxyz_on_command),
        );
        CommandSystem::register(
            w,
            &cmd("XYZ"),
            AccessLevel::GameMaster,
            &[],
            h(xyz_on_command),
        );
        CommandSystem::register(w, &cmd("Z"), AccessLevel::GameMaster, &[], h(z_on_command));
        CommandSystem::register(
            w,
            &cmd("Avg"),
            AccessLevel::GameMaster,
            &[],
            h(avg_on_command),
        );
    }
}

/// `AddGump.ExactMatch(name)`: the registered class called `name`, ignoring case.
pub fn exact_match(w: &World, name: &str) -> Option<EntityType> {
    if name.trim().is_empty() {
        return None;
    }
    w.types.find_type_by_name(name.trim(), false, true)
}

fn type_name(t: EntityType) -> &'static str {
    let full = t.full_name();
    full.rsplit(['.', '+']).next().unwrap_or(full)
}

/// `Add.Invoke(from, start, end, string[] args, packs, outline, mapAvg)`.
#[allow(clippy::too_many_arguments)]
pub fn invoke_args(
    w: &mut World,
    from: MobileId,
    start: Point3D,
    end: Point3D,
    args: &[String],
    packs: Option<&[ItemId]>,
    outline: bool,
    map_avg: bool,
) {
    match args.first().and_then(|a| exact_match(w, a)) {
        None => send_message(w, from, "No type with that name was found."),
        Some(t) => invoke(w, from, start, end, t, &args[1..], packs, outline, map_avg),
    }
}

/// `Add.Invoke(from, start, end, Type type, args, packs, outline, mapAvg)`.
#[allow(clippy::too_many_arguments)]
pub fn invoke(
    w: &mut World,
    from: MobileId,
    start: Point3D,
    end: Point3D,
    t: EntityType,
    args: &[String],
    packs: Option<&[ItemId]>,
    outline: bool,
    map_avg: bool,
) {
    let map_name = w
        .mobile(from)
        .and_then(|m| m.map)
        .and_then(|m| w.maps.get(m))
        .map_or_else(String::new, |m| m.name().to_string());
    let level = w
        .mobile(from)
        .map_or(AccessLevel::Player, |m| m.access_level);
    let mut sb = format!(
        "{level:?} {} building ",
        logging::format_entity(w, EntityId::Mobile(from))
    );
    if start == end {
        sb += &format!("at ({}, {}, {}) in {map_name}", start.x, start.y, start.z);
    } else {
        sb += &format!(
            "from ({}, {}, {}) to ({}, {}, {}) in {map_name}",
            start.x, start.y, start.z, end.x, end.y, end.z
        );
    }
    sb += &format!(": {}", type_name(t));

    let mut props: Vec<(String, String)> = Vec::new();
    let mut set_args_index = None;
    for (i, a) in args.iter().enumerate() {
        sb += &format!(" \"{a}\"");
        if set_args_index.is_some() || !a.eq_ic("set") {
            continue;
        }
        set_args_index = Some(i);
        let remains = (args.len() - i - 1) / 2;
        for j in 0..remains {
            props.push((args[i + j * 2 + 1].clone(), args[i + j * 2 + 2].clone()));
        }
    }
    logging::write_line(w, from, &sb);

    let ctor_args = &args[..set_args_index.unwrap_or(args.len())];
    let watch = Instant::now();
    let built = build_objects(
        w, from, t, start, end, ctor_args, &props, packs, outline, map_avg,
    );
    if built <= 0 {
        send_usage(w, t, from);
        return;
    }
    let secs = watch.elapsed().as_secs_f64();
    let plural = if built == 1 { "object" } else { "objects" };
    send_message(
        w,
        from,
        &format!("{built} {plural} generated in {secs:.2} seconds."),
    );
}

/// `Add.Invoke(from, start, end, ConstructorInfo ctor)`: builds with every parameter at its default. `ctor` is the
/// index into the class's constructor table, `None` for a class with only the parameterless constructor.
pub fn invoke_ctor(
    w: &mut World,
    from: MobileId,
    start: Point3D,
    end: Point3D,
    t: EntityType,
    ctor: Option<usize>,
) {
    let map_name = w
        .mobile(from)
        .and_then(|m| m.map)
        .and_then(|m| w.maps.get(m))
        .map_or_else(String::new, |m| m.name().to_string());
    let level = w
        .mobile(from)
        .map_or(AccessLevel::Player, |m| m.access_level);
    let mut sb = format!(
        "{level:?} {} building ",
        logging::format_entity(w, EntityId::Mobile(from))
    );
    if start == end {
        sb += &format!("at ({}, {}, {}) in {map_name}", start.x, start.y, start.z);
    } else {
        sb += &format!(
            "from ({}, {}, {}) to ({}, {}, {}) in {map_name}",
            start.x, start.y, start.z, end.x, end.y, end.z
        );
    }
    sb += &format!(": {}", type_name(t));
    logging::write_line(w, from, &sb);

    let watch = Instant::now();
    let (mut start, mut end) = (start, end);
    muo_primitives::utility::fix_points(&mut start, &mut end);
    let table = props::ctors_of(w, t.full_name());
    let built = match (table, ctor) {
        (Some(ctors), Some(i)) if i < ctors.len() => {
            let c = &ctors[i];
            // `Type.Missing` for every parameter: its default value.
            let values: Vec<props::Value> = c
                .params
                .iter()
                .map(|p| p.default.clone().unwrap_or(props::Value::Null))
                .collect();
            let make = |w: &mut World| (c.make)(w, &values);
            build(w, from, start, end, &make, &[], &[], None, false, false)
        }
        (None, None) if t.construct_available() => {
            let make = |w: &mut World| match t {
                EntityType::Item(it) => EntityId::Item((it.construct.expect("checked"))(w)),
                EntityType::Mobile(mt) => EntityId::Mobile((mt.construct.expect("checked"))(w)),
            };
            build(w, from, start, end, &make, &[], &[], None, false, false)
        }
        _ => 0,
    };
    if built <= 0 {
        send_usage_ctor(w, t, ctor, from);
        return;
    }
    let secs = watch.elapsed().as_secs_f64();
    let plural = if built == 1 { "object" } else { "objects" };
    send_message(
        w,
        from,
        &format!("{built} {plural} generated in {secs:.2} seconds."),
    );
}

/// `Add.SendUsage(ctor, from)`: the one constructor only.
fn send_usage_ctor(w: &mut World, t: EntityType, ctor: Option<usize>, from: MobileId) {
    let level = w
        .mobile(from)
        .map_or(AccessLevel::Player, |m| m.access_level);
    match (props::ctors_of(w, t.full_name()), ctor) {
        (Some(ctors), Some(i)) if i < ctors.len() => {
            let c = &ctors[i];
            if !muo_content::utilities::attributes::is_constructible(
                Some(c.access.unwrap_or(AccessLevel::Player)),
                level,
            ) {
                send_message(w, from, "That type is not marked constructible.");
            } else {
                send_message(w, from, "Usage:");
                send_message(w, from, &ctor_text(type_name(t), c.params));
            }
        }
        (None, None) if t.construct_available() => {
            send_message(w, from, "Usage:");
            send_message(w, from, type_name(t));
        }
        _ => send_message(w, from, "That type is not marked constructible."),
    }
}

/// `Add.BuildObjects`.
#[allow(clippy::too_many_arguments)]
fn build_objects(
    w: &mut World,
    from: MobileId,
    t: EntityType,
    mut start: Point3D,
    mut end: Point3D,
    args: &[String],
    props: &[(String, String)],
    packs: Option<&[ItemId]>,
    outline: bool,
    map_avg: bool,
) -> i32 {
    muo_primitives::utility::fix_points(&mut start, &mut end);
    // The class's property table for the `set` list; a failing name is reported once and skipped.
    let chain = class_chain_of(t);
    let defs = props::props_of_class(w, &chain);
    let level = w
        .mobile(from)
        .map_or(AccessLevel::Player, |m| m.access_level);
    let mut real: Vec<Option<&'static PropDef>> = Vec::new();
    for (name, _) in props {
        let found = defs.iter().find(|d| d.name.eq_ic(name)).copied();
        match found.map(|d| props::check_access(level, d, Access::WRITE).map(|()| d)) {
            Some(Ok(d)) => real.push(Some(d)),
            Some(Err(m)) => {
                send_message(w, from, &m);
                real.push(None);
            }
            None => real.push(None), // not found: nothing is sent (failReason is null)
        }
    }
    // `type.GetConstructors()`: the class's table, else the parameterless constructor alone.
    if let Some(ctors) = props::ctors_of(w, t.full_name()) {
        for c in ctors {
            if !muo_content::utilities::attributes::is_constructible(
                Some(c.access.unwrap_or(AccessLevel::Player)),
                level,
            ) {
                continue;
            }
            let required = c.params.iter().filter(|p| p.default.is_none()).count();
            if args.len() < required || args.len() > c.params.len() {
                continue;
            }
            let Some(values) = props::parse_ctor_args(w, c.params, args) else {
                continue;
            };
            let make = |w: &mut World| (c.make)(w, &values);
            let built = build(
                w, from, start, end, &make, props, &real, packs, outline, map_avg,
            );
            if built > 0 {
                return built;
            }
        }
        return 0;
    }
    if !args.is_empty() || !t.construct_available() {
        return 0;
    }
    let make = |w: &mut World| match t {
        EntityType::Item(it) => EntityId::Item((it.construct.expect("checked"))(w)),
        EntityType::Mobile(mt) => EntityId::Mobile((mt.construct.expect("checked"))(w)),
    };
    build(
        w, from, start, end, &make, props, &real, packs, outline, map_avg,
    )
}

fn class_chain_of(t: EntityType) -> Vec<&'static str> {
    let mut chain = Vec::new();
    match t {
        EntityType::Item(i) => {
            let mut c = Some(i);
            while let Some(x) = c {
                chain.push(x.full_name);
                c = x.parent;
            }
        }
        EntityType::Mobile(m) => {
            let mut c = Some(m);
            while let Some(x) = c {
                chain.push(x.full_name);
                c = x.parent;
            }
        }
    }
    chain.reverse();
    chain
}

/// `Add.Build(from, ctor, values, props, realProps, ref sendError)`: one object.
fn build_one(
    w: &mut World,
    from: MobileId,
    make: &dyn Fn(&mut World) -> EntityId,
    props: &[(String, String)],
    real: &[Option<&'static PropDef>],
    send_error: &mut bool,
) -> Option<EntityId> {
    let built = make(w);
    let mut had_error = false;
    for (i, def) in real.iter().enumerate() {
        let Some(def) = def else { continue };
        let result =
            internal_set_value(w, from, built, built, def, &props[i].1, &props[i].1, false);
        if result != "Property has been set." {
            if *send_error {
                send_message(w, from, &result);
            }
            had_error = true;
        }
    }
    if had_error {
        *send_error = false;
    }
    Some(built)
}

/// `Add.Build(from, start, end, ctor, values, props, realProps, packs, outline, mapAvg)`.
#[allow(clippy::too_many_arguments)]
fn build(
    w: &mut World,
    from: MobileId,
    start: Point3D,
    end: Point3D,
    make: &dyn Fn(&mut World) -> EntityId,
    props: &[(String, String)],
    real: &[Option<&'static PropDef>],
    packs: Option<&[ItemId]>,
    mut outline: bool,
    map_avg: bool,
) -> i32 {
    let map = w.mobile(from).and_then(|m| m.map);
    let (width, height) = (end.x - start.x + 1, end.y - start.y + 1);
    if outline && (width < 3 || height < 3) {
        outline = false;
    }
    let object_count = match packs {
        Some(p) => p.len() as i32,
        None if outline => (width + height - 2) * 2,
        None => width * height,
    };
    if object_count >= 20 {
        send_message(
            w,
            from,
            &format!("Constructing {object_count} objects, please wait."),
        );
    }
    let mut send_error = true;
    let mut serials = String::from("Serials: ");
    let place = |w: &mut World, built: EntityId, p: Point3D| match built {
        EntityId::Item(i) => muo_server::item::ops::move_to_world_map(w, i, p, map),
        EntityId::Mobile(m) => {
            if let Some(map) = map {
                muo_server::mobile::base::move_to_world(w, m, p, map);
            }
        }
    };
    match packs {
        Some(packs) => {
            for pack in packs {
                let Some(built) = build_one(w, from, make, props, real, &mut send_error) else {
                    return 0;
                };
                serials += &format!("0x{:08X}; ", built_serial(built));
                match built {
                    EntityId::Item(i) => {
                        muo_server::item::container::drop_item_default(w, *pack, i)
                    }
                    EntityId::Mobile(_) => place(w, built, start),
                }
            }
        }
        None => {
            let mut z = start.z;
            for x in start.x..=end.x {
                for y in start.y..=end.y {
                    if outline && x != start.x && x != end.x && y != start.y && y != end.y {
                        continue;
                    }
                    if map_avg && let Some(m) = map {
                        z = w.maps.get_average_z(m, x, y);
                    }
                    let Some(built) = build_one(w, from, make, props, real, &mut send_error) else {
                        return 0;
                    };
                    serials += &format!("0x{:08X}; ", built_serial(built));
                    place(w, built, Point3D::new(x, y, z));
                }
            }
        }
    }
    logging::write_line(w, from, &serials);
    object_count
}

fn built_serial(e: EntityId) -> u32 {
    e.serial().value()
}

/// `Add.SendUsage(type, from)`: each constructible constructor, `Type Int32 amount, ..`. A class with no table has only
/// its parameterless `construct`.
fn send_usage(w: &mut World, t: EntityType, from: MobileId) {
    let level = w
        .mobile(from)
        .map_or(AccessLevel::Player, |m| m.access_level);
    let Some(ctors) = props::ctors_of(w, t.full_name()) else {
        if t.construct_available() {
            send_message(w, from, "Usage:");
            send_message(w, from, type_name(t));
        } else {
            send_message(w, from, "That type is not marked constructible.");
        }
        return;
    };
    let mut found = false;
    for c in ctors {
        if !muo_content::utilities::attributes::is_constructible(
            Some(c.access.unwrap_or(AccessLevel::Player)),
            level,
        ) {
            continue;
        }
        if !found {
            found = true;
            send_message(w, from, "Usage:");
        }
        send_message(w, from, &ctor_text(type_name(t), c.params));
    }
    if !found {
        send_message(w, from, "That type is not marked constructible.");
    }
}

/// `Add.SendCtor`'s line: the class name, then ` <ParameterType.Name> <name>` per parameter, comma separated.
fn ctor_text(class: &str, params: &[props::Param]) -> String {
    let mut sb = class.to_string();
    for (i, p) in params.iter().enumerate() {
        if i != 0 {
            sb.push(',');
        }
        sb += &format!(" {} {}", param_type(p), p.name);
    }
    sb
}

/// `ParameterType.Name`: the table's declared class, else the one the kind implies.
pub fn param_type(p: &props::Param) -> &'static str {
    if p.ty.is_empty() {
        param_type_name(&p.kind)
    } else {
        p.ty
    }
}

fn param_type_name(k: &props::Kind) -> &'static str {
    use muo_content::utilities::types::{Numeric as N, PropType as T};
    match k {
        props::Kind::Plain(T::Numeric(n)) => match n {
            N::Byte => "Byte",
            N::SByte => "SByte",
            N::Short => "Int16",
            N::UShort => "UInt16",
            N::Int => "Int32",
            N::UInt => "UInt32",
            N::Long => "Int64",
            N::ULong => "UInt64",
        },
        props::Kind::Plain(T::Float) => "Single",
        props::Kind::Plain(T::Double) => "Double",
        props::Kind::Plain(T::Decimal) => "Decimal",
        props::Kind::Plain(T::Bool) => "Boolean",
        props::Kind::Plain(T::Char) => "Char",
        props::Kind::Plain(T::Str) => "String",
        props::Kind::Plain(T::Serial) => "Serial",
        props::Kind::Plain(T::Type) => "Type",
        props::Kind::Plain(T::IPoint3D) => "Point3D",
        props::Kind::Plain(T::IPoint2D) => "IPoint2D",
        props::Kind::TimeSpan => "TimeSpan",
        props::Kind::Map => "Map",
        _ => "Object",
    }
}

// ---- [tile family -----------------------------------------------------------------------------------------------

#[derive(Copy, Clone)]
enum ZType {
    Start,
    Fixed(i32),
    MapAverage,
}

fn tile_box(
    w: &mut World,
    from: MobileId,
    mut start: Point3D,
    mut end: Point3D,
    z: ZType,
    args: Vec<String>,
    outline: bool,
) {
    let mut map_avg = false;
    match z {
        ZType::Fixed(f) => {
            start.z = f;
            end.z = f;
        }
        ZType::MapAverage => map_avg = true,
        ZType::Start => {}
    }
    invoke_args(w, from, start, end, &args, None, outline, map_avg);
}

fn begin_box(w: &mut World, from: MobileId, z: ZType, args: Vec<String>, outline: bool) {
    begin_bounding_box(
        w,
        from,
        Rc::new(move |w, _map, start, end| tile_box(w, from, start, end, z, args.clone(), outline)),
    );
}

fn format_help(outline: bool, rest: &str) -> String {
    format!("Format: {}{rest}", if outline { "Outline" } else { "Tile" })
}

fn box_on_command(w: &mut World, e: &mut CommandEventArgs, outline: bool) {
    if e.length() >= 1 {
        begin_box(w, e.mobile, ZType::Start, e.arguments.clone(), outline);
    } else {
        send_message(
            w,
            e.mobile,
            &format_help(
                outline,
                " <type> [params] [set {<propertyName> <value> ...}]",
            ),
        );
    }
}

fn rxyz_on_command(w: &mut World, e: &mut CommandEventArgs, outline: bool) {
    if e.length() >= 6 {
        let loc = w.mobile(e.mobile).map(|m| m.location).unwrap_or_default();
        let p = Point3D::new(
            loc.x + e.get_int32(0),
            loc.y + e.get_int32(1),
            loc.z + e.get_int32(4),
        );
        let p2 = Point3D::new(p.x + e.get_int32(2) - 1, p.y + e.get_int32(3) - 1, p.z);
        invoke_args(w, e.mobile, p, p2, &e.arguments[5..], None, outline, false);
    } else {
        send_message(
            w,
            e.mobile,
            &format_help(
                outline,
                "RXYZ <x> <y> <w> <h> <z> <type> [params] [set {<propertyName> <value> ...}]",
            ),
        );
    }
}

fn xyz_on_command(w: &mut World, e: &mut CommandEventArgs, outline: bool) {
    if e.length() >= 6 {
        let p = Point3D::new(e.get_int32(0), e.get_int32(1), e.get_int32(4));
        let p2 = Point3D::new(
            p.x + e.get_int32(2) - 1,
            p.y + e.get_int32(3) - 1,
            e.get_int32(4),
        );
        invoke_args(w, e.mobile, p, p2, &e.arguments[5..], None, outline, false);
    } else {
        send_message(
            w,
            e.mobile,
            &format_help(
                outline,
                "XYZ <x> <y> <w> <h> <z> <type> [params] [set {<propertyName> <value> ...}]",
            ),
        );
    }
}

fn z_on_command(w: &mut World, e: &mut CommandEventArgs, outline: bool) {
    if e.length() >= 2 {
        begin_box(
            w,
            e.mobile,
            ZType::Fixed(e.get_int32(0)),
            e.arguments[1..].to_vec(),
            outline,
        );
    } else {
        send_message(
            w,
            e.mobile,
            &format_help(
                outline,
                "Z <z> <type> [params] [set {<propertyName> <value> ...}]",
            ),
        );
    }
}

fn avg_on_command(w: &mut World, e: &mut CommandEventArgs, outline: bool) {
    if e.length() >= 1 {
        begin_box(w, e.mobile, ZType::MapAverage, e.arguments.clone(), outline);
    } else {
        send_message(
            w,
            e.mobile,
            &format_help(
                outline,
                "Avg <type> [params] [set {<propertyName> <value> ...}]",
            ),
        );
    }
}

// ---- AddCommand / AddToPackCommand ------------------------------------------------------------------------------

fn context_key() -> &'static str {
    "exactMatch"
}

pub struct AddCommand {
    spec: CmdSpec,
}

impl AddCommand {
    pub fn new() -> Self {
        Self {
            spec: CmdSpec::new(
                &["Add"],
                AccessLevel::GameMaster,
                S::SIMPLE | S::SELF,
                O::All,
                "Add [<name> [params] [set {<propertyName> <value> ...}]]",
                "Adds an item or npc by name to a targeted location. Optional constructor parameters. Optional set property list. If no arguments are specified, this brings up a categorized add menu.",
            ),
        }
    }
}

impl Default for AddCommand {
    fn default() -> Self {
        Self::new()
    }
}

fn open_add_gump(w: &mut World, from: MobileId, search: &str, categorized: bool) {
    // `new CategorizedAddGump(from)` with no search, else `new AddGump(search, 0, matches, ..)`; the hook lets a
    // host replace the menus.
    if let Some(g) = w.resources.get_mut::<CommandHooks>().add_gump {
        g(w, from, search, categorized);
    } else if categorized {
        crate::add_menus::open_categorized(w, from);
    } else {
        crate::add_menus::open_add_gump_search(w, from, search);
    }
}

impl BaseCommand for AddCommand {
    fn spec(&self) -> &CmdSpec {
        &self.spec
    }

    fn validate_args(&self, w: &mut World, e: &mut CommandEventArgs) -> bool {
        let from = e.mobile;
        if e.length() < 1 {
            open_add_gump(w, from, "", true);
            return false;
        }
        let matched = e.get_string(0).trim().to_string();
        if e.get_context::<EntityTypeBox>(context_key()).is_some() {
            return true;
        }
        if let Some(t) = exact_match(w, &matched) {
            e.set_context(context_key(), EntityTypeBox(t));
            return true;
        }
        if matched.chars().count() < 3 {
            send_message(w, from, "Invalid search string.");
            open_add_gump(w, from, &matched, false);
            return false;
        }
        send_message(w, from, "No type with that name was found.");
        open_add_gump(w, from, &matched, false);
        false
    }

    fn execute(&self, w: &mut World, _run: &mut Run, e: &CommandEventArgs, obj: Obj) {
        let p = match obj {
            Targeted::Entity(EntityId::Item(i)) => muo_server::item::ops::get_world_top(w, i),
            Targeted::Entity(EntityId::Mobile(m)) => {
                w.mobile(m).map(|x| x.location).unwrap_or_default()
            }
            Targeted::Land(l) => l.location,
            Targeted::Static(s) => s.location,
        };
        match e.get_context::<EntityTypeBox>(context_key()) {
            Some(EntityTypeBox(t)) => {
                invoke(w, e.mobile, p, p, *t, &e.arguments[1..], None, false, false)
            }
            None => send_message(w, e.mobile, "No type with that name was found."),
        }
    }
}

/// `EntityType` is `Copy`; a named wrapper for the event context's `Any`.
#[derive(Copy, Clone)]
struct EntityTypeBox(EntityType);

pub struct AddToPackCommand {
    spec: CmdSpec,
}

impl AddToPackCommand {
    pub fn new() -> Self {
        Self {
            spec: CmdSpec::new(
                &["AddToPack", "AddToCont"],
                AccessLevel::GameMaster,
                S::ALL,
                O::Both,
                "AddToPack <name> [params] [set {<propertyName> <value> ...}]",
                "Adds an item by name to the backpack of a targeted player or npc, or a targeted container. Optional constructor parameters. Optional set property list.",
            )
            .list_optimized(),
        }
    }
}

impl Default for AddToPackCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl BaseCommand for AddToPackCommand {
    fn spec(&self) -> &CmdSpec {
        &self.spec
    }

    fn execute_list(&self, w: &mut World, run: &mut Run, e: &CommandEventArgs, list: &[Obj]) {
        if e.arguments.is_empty() {
            return;
        }
        let mut packs = Vec::new();
        for o in list {
            let cont = match o {
                Targeted::Entity(EntityId::Mobile(m)) => w.mobile(*m).and_then(|x| x.backpack),
                Targeted::Entity(EntityId::Item(i))
                    if w.item(*i).is_some_and(|x| x.cont.is_some()) =>
                {
                    Some(*i)
                }
                _ => None,
            };
            match cont {
                Some(c) => packs.push(c),
                None => run.log_failure("That is not a container."),
            }
        }
        let loc = w.mobile(e.mobile).map(|m| m.location).unwrap_or_default();
        invoke_args(
            w,
            e.mobile,
            loc,
            loc,
            &e.arguments,
            Some(&packs),
            false,
            false,
        );
    }
}

#[cfg(test)]
mod usage_tests {
    use super::*;
    use muo_content::props::{Kind, Param, Value};
    use muo_content::utilities::types::{Numeric, PropType};

    #[test]
    fn ctor_line_is_class_then_type_and_name_per_parameter() {
        static P: [Param; 2] = [
            Param {
                name: "amount",
                kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
                default: Some(Value::Int(1)),
                ty: "",
            },
            Param {
                name: "name",
                kind: Kind::Plain(PropType::Str),
                default: None,
                ty: "",
            },
        ];
        assert_eq!(ctor_text("Gold", &P), "Gold Int32 amount, String name");
        assert_eq!(ctor_text("Gold", &[]), "Gold");
    }

    #[test]
    fn an_enum_or_entity_parameter_prints_its_declared_class() {
        static P: [Param; 2] = [
            Param {
                name: "type",
                kind: Kind::Plain(PropType::Enum(&[("A", 0)])),
                default: None,
                ty: "BeverageType",
            },
            Param {
                name: "owner",
                kind: Kind::Plain(PropType::Entity),
                default: None,
                ty: "Mobile",
            },
        ];
        assert_eq!(ctor_text("Jug", &P), "Jug BeverageType type, Mobile owner");
    }
}
