//! The command implementors: the scopes (`[global`, `[area`, `[m`, `[self`..) a command runs under.

use muo_primitives::dotnet::IgnoreCase;
use std::rc::Rc;

use muo_primitives::{Point3D, Rectangle2D, Serial};
use muo_server::commands::{CommandEventArgs, CommandSystem};
use muo_server::ids::{EntityId, MobileId};
use muo_server::map::{MapId, enumerate};
use muo_server::mobile::AccessLevel;
use muo_server::mobile::part2::messages::send_message;
use muo_server::target::{TargetFlags, Targeted, begin_target};
use muo_server::world::World;

use super::conditional::Extensions;
use super::{
    BaseCommand, CommandSupport, GenericCommands, Obj, ObjectTypes, Run, as_item, as_mobile,
    is_accessible,
};
use crate::logging::CommandLogging;

/// Which class of `BaseCommandImplementor`.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Kind {
    Region,
    Global,
    Online,
    Single,
    Serial,
    Multi,
    Area,
    SelfOnly,
    Contained,
    IpAddress,
    Range,
    Screen,
    Facet,
}

pub struct Implementor {
    pub kind: Kind,
    pub accessors: &'static [&'static str],
    pub usage: &'static str,
    pub description: &'static str,
    pub access_level: AccessLevel,
    pub support: CommandSupport,
    pub supports_conditionals: bool,
    /// Lower-cased command name -> command (`StringComparer.OrdinalIgnoreCase`).
    pub commands: indexmap::IndexMap<String, Rc<dyn BaseCommand>>,
}

const NOT_SUPPORTED: &str =
    "That is either an invalid command name or one that does not support this modifier.";

fn imp(
    kind: Kind,
    accessors: &'static [&'static str],
    support: CommandSupport,
    conditionals: bool,
    access_level: AccessLevel,
    usage: &'static str,
    description: &'static str,
) -> Implementor {
    Implementor {
        kind,
        accessors,
        usage,
        description,
        access_level,
        support,
        supports_conditionals: conditionals,
        commands: Default::default(),
    }
}

/// `BaseCommandImplementor.RegisterImplementors`, and each implementor's accessors as commands.
pub fn register_implementors(w: &mut World) {
    use AccessLevel::{Administrator as ADM, Counselor as CNS, GameMaster as GM};
    let list = vec![
        imp(
            Kind::Region,
            &["Region"],
            CommandSupport::REGION,
            true,
            GM,
            "Region <command> [condition]",
            "Invokes the command on all appropriate mobiles in your current region. Optional condition arguments can further restrict the set of objects.",
        ),
        imp(
            Kind::Global,
            &["Global"],
            CommandSupport::GLOBAL,
            true,
            ADM,
            "Global <command> [condition]",
            "Invokes the command on all appropriate objects in the world. Optional condition arguments can further restrict the set of objects.",
        ),
        imp(
            Kind::Online,
            &["Online"],
            CommandSupport::ONLINE,
            true,
            GM,
            "Online <command> [condition]",
            "Invokes the command on all mobiles that are currently logged in. Optional condition arguments can further restrict the set of objects.",
        ),
        imp(
            Kind::Single,
            &["Single"],
            CommandSupport::SINGLE,
            false,
            CNS,
            "Single <command>",
            "Invokes the command on a single targeted object. This is the same as just invoking the command directly.",
        ),
        imp(
            Kind::Serial,
            &["Serial"],
            CommandSupport::SINGLE,
            false,
            CNS,
            "Serial <serial> <command>",
            "Invokes the command on a single object by serial.",
        ),
        imp(
            Kind::Multi,
            &["Multi", "m"],
            CommandSupport::MULTI,
            false,
            CNS,
            "Multi <command>",
            "Invokes the command on multiple targeted objects.",
        ),
        imp(
            Kind::Area,
            &["Area", "Group"],
            CommandSupport::AREA,
            true,
            GM,
            "Area <command> [condition]",
            "Invokes the command on all appropriate objects in a targeted area. Optional condition arguments can further restrict the set of objects.",
        ),
        imp(
            Kind::SelfOnly,
            &["Self"],
            CommandSupport::SELF,
            false,
            CNS,
            "Self <command>",
            "Invokes the command on the commanding player.",
        ),
        imp(
            Kind::Contained,
            &["Contained"],
            CommandSupport::CONTAINED,
            false,
            GM,
            "Contained <command> [condition]",
            "Invokes the command on all child items in a targeted container. Optional condition arguments can further restrict the set of objects.",
        ),
        imp(
            Kind::IpAddress,
            &["IPAddress"],
            CommandSupport::IP_ADDRESS,
            true,
            ADM,
            "IPAddress <command> [condition]",
            "Invokes the command on one mobile from each IP address that is logged in. Optional condition arguments can further restrict the set of objects.",
        ),
        imp(
            Kind::Range,
            &["Range"],
            CommandSupport::AREA,
            true,
            GM,
            "Range <range> <command> [condition]",
            "Invokes the command on all appropriate objects within a specified range of you. Optional condition arguments can further restrict the set of objects.",
        ),
        imp(
            Kind::Screen,
            &["Screen"],
            CommandSupport::AREA,
            true,
            GM,
            "Screen <command> [condition]",
            "Invokes the command on all appropriate objects in your screen. Optional condition arguments can further restrict the set of objects.",
        ),
        imp(
            Kind::Facet,
            &["Facet"],
            CommandSupport::AREA,
            true,
            GM,
            "Facet <command> [condition]",
            "Invokes the command on all appropriate objects within your facet's map bounds. Optional condition arguments can further restrict the set of objects.",
        ),
    ];
    let n = list.len();
    w.resources.get_mut::<GenericCommands>().implementors = list;
    for idx in 0..n {
        let (accessors, level) = {
            let i = &w.resources.get_mut::<GenericCommands>().implementors[idx];
            (i.accessors, i.access_level)
        };
        #[allow(clippy::type_complexity)] // the handler type, spelled out
        let handler: Rc<dyn Fn(&mut World, &mut CommandEventArgs)> =
            Rc::new(move |w, e| execute(w, idx, e));
        for name in accessors {
            CommandSystem::register(w, name, level, &[], handler.clone());
        }
    }
}

/// `TargetCommands.Register(command)`: into every implementor whose requirement it supports.
pub fn register_command(w: &mut World, command: Rc<dyn BaseCommand>) {
    let (names, supports, level) = {
        let s = command.spec();
        (s.commands.clone(), s.supports, s.access_level)
    };
    let gc = w.resources.get_mut::<GenericCommands>();
    gc.all.push(command.clone());
    let mut singles = false;
    for imp in &mut gc.implementors {
        if supports.intersects(imp.support) {
            for n in &names {
                imp.commands.insert(n.ic_key(), command.clone());
            }
            singles |= imp.kind == Kind::Single;
        }
    }
    if singles {
        // `SingleCommandImplementor.Register`: each command is also a command of its own.
        #[allow(clippy::type_complexity)] // the handler type, spelled out
        let handler: Rc<dyn Fn(&mut World, &mut CommandEventArgs)> = Rc::new(redirect);
        for n in &names {
            CommandSystem::register(w, n, level, &[], handler.clone());
        }
    }
}

/// `Scope.Commands[name]`.
pub fn find_command(w: &World, idx: usize, name: &str) -> Option<Rc<dyn BaseCommand>> {
    w.resources.get::<GenericCommands>()?.implementors[idx]
        .commands
        .get(&name.ic_key())
        .cloned()
}

fn level_of(w: &World, m: MobileId) -> AccessLevel {
    w.mobile(m).map_or(AccessLevel::Player, |x| x.access_level)
}

/// `SingleCommandImplementor.Redirect`.
fn redirect(w: &mut World, e: &mut CommandEventArgs) {
    let idx = single_index(w);
    let from = e.mobile;
    match find_command(w, idx, &e.command) {
        None => send_message(w, from, NOT_SUPPORTED),
        Some(c) if level_of(w, from) < c.spec().access_level => {
            send_message(w, from, "You do not have access to that command.")
        }
        Some(c) => process(w, idx, from, c, e.arguments.clone()),
    }
}

fn single_index(w: &World) -> usize {
    w.resources
        .get::<GenericCommands>()
        .and_then(|g| g.implementors.iter().position(|i| i.kind == Kind::Single))
        .unwrap_or(0)
}

fn kind_of(w: &World, idx: usize) -> Kind {
    w.resources
        .get::<GenericCommands>()
        .map_or(Kind::Single, |g| g.implementors[idx].kind)
}

/// `BaseCommandImplementor.Execute(e)` (the accessor command itself), with `Range` and `Serial` overriding.
fn execute(w: &mut World, idx: usize, e: &mut CommandEventArgs) {
    let from = e.mobile;
    match kind_of(w, idx) {
        Kind::Range => return range_execute(w, idx, e),
        Kind::Serial => return serial_execute(w, idx, e),
        _ => {}
    }
    if e.length() >= 1 {
        match find_command(w, idx, &e.get_string(0)) {
            None => send_message(w, from, NOT_SUPPORTED),
            Some(c) if level_of(w, from) < c.spec().access_level => {
                send_message(w, from, "You do not have access to that command.")
            }
            Some(c) => process(w, idx, from, c, e.arguments[1..].to_vec()),
        }
    } else {
        send_message(w, from, "You must supply a command name.");
    }
}

fn range_execute(w: &mut World, idx: usize, e: &mut CommandEventArgs) {
    let from = e.mobile;
    if e.length() >= 2 {
        let range = e.get_int32(0);
        if range < 0 {
            send_message(w, from, "The range must not be negative.");
        } else {
            match find_command(w, idx, &e.get_string(1)) {
                None => send_message(w, from, NOT_SUPPORTED),
                Some(c) if level_of(w, from) < c.spec().access_level => {
                    send_message(w, from, "You do not have access to that command.")
                }
                Some(c) => process_range(w, range, from, c, e.arguments[2..].to_vec()),
            }
        }
    } else {
        send_message(w, from, "You must supply a range and a command name.");
    }
}

fn serial_execute(w: &mut World, idx: usize, e: &mut CommandEventArgs) {
    let from = e.mobile;
    if e.length() < 2 {
        send_message(
            w,
            from,
            "You must supply an object serial and a command name.",
        );
        return;
    }
    let serial = Serial::new(e.get_uint32(0));
    let obj: Option<Obj> = if serial.is_item() {
        w.find_item(serial)
            .map(|i| Targeted::Entity(EntityId::Item(i)))
    } else if serial.is_mobile() {
        w.find_mobile(serial)
            .map(|m| Targeted::Entity(EntityId::Mobile(m)))
    } else {
        None
    };
    let Some(obj) = obj else {
        send_message(w, from, "That is not a valid serial.");
        return;
    };
    let Some(c) = find_command(w, idx, &e.get_string(1)) else {
        send_message(w, from, NOT_SUPPORTED);
        return;
    };
    if level_of(w, from) < c.spec().access_level {
        send_message(w, from, "You do not have access to that command.");
        return;
    }
    match c.spec().object_types {
        ObjectTypes::Items if as_item(obj).is_none() => {
            return send_message(w, from, "This command only works on items.");
        }
        ObjectTypes::Mobiles if as_mobile(obj).is_none() => {
            return send_message(w, from, "This command only works on mobiles.");
        }
        _ => {}
    }
    run_command_obj(w, from, Objs::One(obj), c, e.arguments[2..].to_vec());
}

/// The result of `Compile`.
pub enum Objs {
    None,
    One(Obj),
    List(Vec<Obj>),
}

/// `BaseCommandImplementor.GenerateArgString`.
pub fn generate_arg_string(args: &[String]) -> String {
    args.iter()
        .map(|a| {
            if a.contains(' ') {
                format!("\"{a}\"")
            } else {
                a.clone()
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// `CheckObjectTypes`: which of items and mobiles the command and the `where` type allow.
fn check_object_types(
    w: &mut World,
    from: MobileId,
    command: &dyn BaseCommand,
    ext: &Extensions,
) -> Option<(bool, bool)> {
    let (mut cond_item, mut cond_mobile) = (true, true);
    for e in &ext.0 {
        if let super::conditional::Extension::Where(c) = e {
            cond_item = c.is_item();
            cond_mobile = c.is_mobile();
            break;
        }
    }
    let (mut items, mut mobiles) = (false, false);
    match command.spec().object_types {
        ObjectTypes::All | ObjectTypes::Both => {
            items = cond_item;
            mobiles = cond_mobile;
        }
        ObjectTypes::Items => {
            if cond_item {
                items = true;
            } else if cond_mobile {
                send_message(
                    w,
                    from,
                    "You may not use a mobile type condition for this command.",
                );
                return None;
            }
        }
        ObjectTypes::Mobiles => {
            if cond_mobile {
                mobiles = true;
            } else if cond_item {
                send_message(
                    w,
                    from,
                    "You may not use an item type condition for this command.",
                );
                return None;
            }
        }
    }
    Some((items, mobiles))
}

fn ent_item(i: muo_server::ids::ItemId) -> Obj {
    Targeted::Entity(EntityId::Item(i))
}
fn ent_mobile(m: MobileId) -> Obj {
    Targeted::Entity(EntityId::Mobile(m))
}

/// `Compile` of Global, Online, Region, IPAddress and Self. `Err` is the exception message sent to the player.
fn compile(
    w: &mut World,
    idx: usize,
    from: MobileId,
    command: &Rc<dyn BaseCommand>,
    args: &mut Vec<String>,
    run: &mut Run,
) -> Result<Objs, String> {
    let level = level_of(w, from);
    match kind_of(w, idx) {
        Kind::SelfOnly => Ok(if command.spec().object_types == ObjectTypes::Items {
            Objs::None
        } else {
            Objs::One(ent_mobile(from))
        }),
        Kind::Global => {
            let ext = Extensions::parse(w, level, args)?;
            let Some((items, mobiles)) = check_object_types(w, from, command.as_ref(), &ext) else {
                return Ok(Objs::None);
            };
            let mut list = Vec::new();
            if items {
                let ids: Vec<_> = w.items.iter().map(|(id, _)| ent_item(*id)).collect();
                list.extend(ids.into_iter().filter(|o| ext.is_valid(w, *o)));
            }
            if mobiles {
                let ids: Vec<_> = w.mobiles.iter().map(|(id, _)| ent_mobile(*id)).collect();
                list.extend(ids.into_iter().filter(|o| ext.is_valid(w, *o)));
            }
            ext.filter(w, &mut list);
            Ok(Objs::List(list))
        }
        Kind::Online => {
            let ext = Extensions::parse(w, level, args)?;
            let Some((_, mobiles)) = check_object_types(w, from, command.as_ref(), &ext) else {
                return Ok(Objs::None);
            };
            if !mobiles {
                run.log_failure("This command does not support items.");
                return Ok(Objs::None);
            }
            let mut list: Vec<Obj> = Vec::new();
            let online: Vec<_> = w.net_states.values().filter_map(|d| d.mobile).collect();
            for m in online {
                {
                    let o = ent_mobile(m);
                    if is_accessible(w, from, Some(o)) && ext.is_valid(w, o) {
                        list.push(o);
                    }
                }
            }
            ext.filter(w, &mut list);
            Ok(Objs::List(list))
        }
        Kind::IpAddress => {
            let ext = Extensions::parse(w, level, args)?;
            let Some((_, mobiles)) = check_object_types(w, from, command.as_ref(), &ext) else {
                return Ok(Objs::None);
            };
            if !mobiles {
                run.log_failure("This command does not support items.");
                return Ok(Objs::None);
            }
            let mut list: Vec<Obj> = Vec::new();
            let mut addresses = Vec::new();
            let online: Vec<_> = w
                .net_states
                .values()
                .filter_map(|d| d.mobile.map(|m| (m, d.address)))
                .collect();
            for (m, address) in online {
                if !addresses.contains(&address) && ext.is_valid(w, ent_mobile(m)) {
                    list.push(ent_mobile(m));
                    addresses.push(address);
                }
            }
            ext.filter(w, &mut list);
            Ok(Objs::List(list))
        }
        Kind::Region => {
            let ext = Extensions::parse(w, level, args)?;
            let Some((items, mobiles)) = check_object_types(w, from, command.as_ref(), &ext) else {
                return Ok(Objs::None);
            };
            let region = w.mobile(from).map(|m| m.region);
            let mut list: Vec<Obj> = Vec::new();
            if let Some(r) = region {
                if mobiles {
                    for m in muo_server::region::get_mobiles(w, r) {
                        let o = ent_mobile(m);
                        if is_accessible(w, from, Some(o)) && ext.is_valid(w, o) {
                            list.push(o);
                        }
                    }
                }
                if items {
                    for i in muo_server::region::get_items(w, r) {
                        let o = ent_item(i);
                        if is_accessible(w, from, Some(o)) && ext.is_valid(w, o) {
                            list.push(o);
                        }
                    }
                }
            }
            ext.filter(w, &mut list);
            Ok(Objs::List(list))
        }
        _ => Ok(Objs::None),
    }
}

/// `BaseCommandImplementor.Process(from, command, args)`.
pub fn process(
    w: &mut World,
    idx: usize,
    from: MobileId,
    command: Rc<dyn BaseCommand>,
    args: Vec<String>,
) {
    match kind_of(w, idx) {
        Kind::Single | Kind::Multi | Kind::Contained => {
            let mut e = CommandEventArgs::new(
                from,
                command.spec().commands[0].clone(),
                generate_arg_string(&args),
                args.clone(),
            );
            if command.validate_args(w, &mut e) {
                begin(w, idx, from, command, args);
            }
        }
        Kind::Area => {
            send_message(w, from, "Target the first location of the bounding box.");
            pick_box(
                w,
                from,
                None,
                Rc::new(move |w, map, start, end| {
                    area_on_target(w, from, map, start, end, command.clone(), args.clone())
                }),
            );
        }
        Kind::Screen => process_range(w, 18, from, command, args),
        Kind::Facet => {
            let Some(map) = w.mobile(from).and_then(|m| m.map) else {
                return;
            };
            if map == MapId::INTERNAL {
                return;
            }
            let (width, height) = w.maps.get(map).map_or((0, 0), |m| (m.width, m.height));
            area_on_target(
                w,
                from,
                map,
                Point3D::new(0, 0, 0),
                Point3D::new(width - 1, height - 1, 0),
                command,
                args,
            );
        }
        _ => run_command(w, idx, from, command, args),
    }
}

/// `Range` / `Screen`: an area around the commanding mobile.
fn process_range(
    w: &mut World,
    range: i32,
    from: MobileId,
    command: Rc<dyn BaseCommand>,
    args: Vec<String>,
) {
    let Some((map, loc)) = w
        .mobile(from)
        .and_then(|m| m.map.map(|mm| (mm, m.location)))
    else {
        return;
    };
    if map == MapId::INTERNAL {
        return;
    }
    let start = Point3D::new(loc.x - range, loc.y - range, loc.z);
    let end = Point3D::new(loc.x + range, loc.y + range, loc.z);
    area_on_target(w, from, map, start, end, command, args);
}

fn begin(
    w: &mut World,
    idx: usize,
    from: MobileId,
    command: Rc<dyn BaseCommand>,
    args: Vec<String>,
) {
    let ground = command.spec().object_types == ObjectTypes::All;
    begin_target(
        w,
        from,
        -1,
        ground,
        TargetFlags::NONE,
        move |w, m, targeted| on_target(w, idx, m, targeted, command.clone(), args.clone()),
    );
}

/// `Single` / `Multi` / `Contained` `OnTarget`.
fn on_target(
    w: &mut World,
    idx: usize,
    from: MobileId,
    targeted: Targeted,
    command: Rc<dyn BaseCommand>,
    args: Vec<String>,
) {
    let kind = kind_of(w, idx);
    if !is_accessible(w, from, Some(targeted)) {
        muo_server::item::deps::send_localized_message(w, from, 500447); // That is not accessible.
        if kind == Kind::Multi {
            begin(w, idx, from, command, args);
        }
        return;
    }
    let ot = command.spec().object_types;
    let item = as_item(targeted).is_some();
    let mobile = as_mobile(targeted).is_some();
    match kind {
        Kind::Single | Kind::Multi => {
            let both_msg = "This command does not work on that.";
            let bad = match ot {
                ObjectTypes::Both if !(item || mobile) => Some(both_msg),
                ObjectTypes::Items if !item => Some("This command only works on items."),
                ObjectTypes::Mobiles if !mobile => Some("This command only works on mobiles."),
                _ => None,
            };
            if let Some(m) = bad {
                send_message(w, from, m);
                return;
            }
            run_command_obj(w, from, Objs::One(targeted), command.clone(), args.clone());
            if kind == Kind::Multi {
                begin(w, idx, from, command, args);
            }
        }
        Kind::Contained => {
            if ot == ObjectTypes::Mobiles {
                return;
            }
            let cont = as_item(targeted).filter(|i| w.item(*i).is_some_and(|x| x.cont.is_some()));
            let Some(cont) = cont else {
                send_message(w, from, "That is not a container.");
                return;
            };
            let mut args = args;
            let level = level_of(w, from);
            let result = (|| -> Result<Option<Vec<Obj>>, String> {
                let ext = Extensions::parse(w, level, &mut args)?;
                let Some((items, _)) = check_object_types(w, from, command.as_ref(), &ext) else {
                    return Ok(None);
                };
                if !items {
                    send_message(w, from, "This command only works on items.");
                    return Ok(None);
                }
                let mut list: Vec<Obj> = Vec::new();
                let mut queue = std::collections::VecDeque::from([cont]);
                while let Some(c) = queue.pop_front() {
                    let Some(it) = w.item(c) else { continue };
                    let kids = it.items.clone();
                    for child in kids {
                        let o = ent_item(child);
                        if ext.is_valid(w, o) {
                            list.push(o);
                        }
                        queue.push_back(child);
                    }
                }
                ext.filter(w, &mut list);
                Ok(Some(list))
            })();
            match result {
                Ok(Some(list)) => run_command_obj(w, from, Objs::List(list), command, args),
                Ok(None) => {}
                Err(m) => send_message(w, from, &m),
            }
        }
        _ => {}
    }
}

/// `AreaCommandImplementor.OnTarget`.
fn area_on_target(
    w: &mut World,
    from: MobileId,
    map: MapId,
    start: Point3D,
    end: Point3D,
    command: Rc<dyn BaseCommand>,
    mut args: Vec<String>,
) {
    let level = level_of(w, from);
    let rect = Rectangle2D::new(start.x, start.y, end.x - start.x + 1, end.y - start.y + 1);
    let ext = match Extensions::parse(w, level, &mut args) {
        Ok(e) => e,
        Err(m) => return send_message(w, from, &m),
    };
    let Some((items, mobiles)) = check_object_types(w, from, command.as_ref(), &ext) else {
        return;
    };
    if !(items || mobiles) {
        return;
    }
    let mut objs: Vec<Obj> = Vec::new();
    if mobiles {
        let found: Vec<MobileId> =
            enumerate::mobiles_in_bounds(w, Some(map), rect, false).collect();
        objs.extend(
            found
                .into_iter()
                .map(ent_mobile)
                .filter(|o| is_accessible(w, from, Some(*o)) && ext.is_valid(w, *o)),
        );
    }
    if items {
        let found: Vec<_> = enumerate::items_in_bounds(w, Some(map), rect, false).collect();
        objs.extend(
            found
                .into_iter()
                .map(ent_item)
                .filter(|o| is_accessible(w, from, Some(*o)) && ext.is_valid(w, *o)),
        );
    }
    ext.filter(w, &mut objs);
    run_command_obj(w, from, Objs::List(objs), command, args);
}

/// `RunCommand(from, command, args)`: compile, then run.
fn run_command(
    w: &mut World,
    idx: usize,
    from: MobileId,
    command: Rc<dyn BaseCommand>,
    mut args: Vec<String>,
) {
    let mut run = Run::default();
    match compile(w, idx, from, &command, &mut args, &mut run) {
        Ok(objs) => run_command_with(w, from, objs, command, args, run),
        Err(m) => send_message(w, from, &m),
    }
}

pub fn run_command_obj(
    w: &mut World,
    from: MobileId,
    objs: Objs,
    command: Rc<dyn BaseCommand>,
    args: Vec<String>,
) {
    run_command_with(w, from, objs, command, args, Run::default());
}

/// `RunCommand(from, obj, command, args)`.
fn run_command_with(
    w: &mut World,
    from: MobileId,
    objs: Objs,
    command: Rc<dyn BaseCommand>,
    args: Vec<String>,
    mut run: Run,
) {
    let mut e = CommandEventArgs::new(
        from,
        command.spec().commands[0].clone(),
        generate_arg_string(&args),
        args,
    );
    if !command.validate_args(w, &mut e) {
        return;
    }
    let mut flush_to_log = false;
    match objs {
        Objs::List(list) => {
            let big = list.len() > 20;
            if big {
                w.resources.get_mut::<CommandLogging>().enabled = false;
            } else if list.is_empty() {
                run.log_failure("Nothing was found to use this command on.");
            }
            command.execute_list(w, &mut run, &e, &list);
            if big {
                flush_to_log = true;
                w.resources.get_mut::<CommandLogging>().enabled = true;
            }
        }
        Objs::One(o) => {
            if command.spec().list_optimized {
                command.execute_list(w, &mut run, &e, &[o]);
            } else {
                command.execute(w, &mut run, &e, o);
            }
        }
        Objs::None => {}
    }
    run.flush(w, from, flush_to_log);
}

// ---- BoundingBoxPicker ------------------------------------------------------------------------------------------

type BoxCallback = Rc<dyn Fn(&mut World, MapId, Point3D, Point3D)>;

/// `BoundingBoxPicker.Begin` (the first prompt message is sent by the caller): two ground targets on one map.
pub fn pick_box(
    w: &mut World,
    from: MobileId,
    first: Option<(Point3D, Option<MapId>)>,
    callback: BoxCallback,
) {
    begin_target(
        w,
        from,
        -1,
        true,
        TargetFlags::NONE,
        move |w, m, targeted| {
            let p = match targeted {
                Targeted::Entity(EntityId::Item(i)) => muo_server::item::ops::get_world_top(w, i),
                Targeted::Entity(EntityId::Mobile(mb)) => {
                    w.mobile(mb).map(|x| x.location).unwrap_or_default()
                }
                Targeted::Land(l) => l.location,
                Targeted::Static(s) => s.location,
            };
            match first {
                None => {
                    send_message(
                        w,
                        m,
                        "Target another location to complete the bounding box.",
                    );
                    let map = w.mobile(m).and_then(|x| x.map);
                    pick_box(w, m, Some((p, map)), callback.clone());
                }
                Some((store, map)) => {
                    let cur = w.mobile(m).and_then(|x| x.map);
                    if cur != map {
                        send_message(w, m, "Both locations must reside on the same map.");
                    } else if let Some(map) = map.filter(|mm| *mm != MapId::INTERNAL) {
                        let (mut start, mut end) = (store, p);
                        muo_primitives::utility::fix_points(&mut start, &mut end);
                        callback(w, map, start, end);
                    }
                }
            }
        },
    );
}

/// `BoundingBoxPicker.Begin(from, callback)`.
pub fn begin_bounding_box(w: &mut World, from: MobileId, callback: BoxCallback) {
    send_message(w, from, "Target the first location of the bounding box.");
    pick_box(w, from, None, callback);
}
