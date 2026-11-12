//! Generic commands: `BaseCommand`, the scope implementors (`[global`, `[area`, `[m`..), the
//! `where` / `distinct` / `order by` / `limit` modifiers, and the target commands (`[set`, `[get`, `[delete`..).
//!
//! A command is a stateless `Rc<dyn BaseCommand>`; what a run keeps between `ValidateArgs` and
//! `Execute` lives in the [`CommandEventArgs`] context (as `Add` already did there) and the per-run response and
//! failure lists in [`Run`]. An implementor is a row of [`GenericCommands`] in `w.resources`, run by index.
//!
//! The object a command runs on is [`Obj`] is [`Targeted`] (an item or mobile, or for `[add` and
//! `[tele` the land or static tile that was targeted).

pub mod commands;
pub mod conditional;
pub mod implementors;
pub mod interface;

use std::rc::Rc;

use bitflags::bitflags;
use muo_server::commands::CommandEventArgs;
use muo_server::ids::{EntityId, MobileId};
use muo_server::mobile::AccessLevel;
use muo_server::mobile::part2::messages::send_message;
use muo_server::target::Targeted;
use muo_server::world::World;

use crate::logging;

/// What a command runs on.
pub type Obj = Targeted;

/// `ObjectTypes`.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ObjectTypes {
    Both,
    Items,
    Mobiles,
    All,
}

bitflags! {
    /// `CommandSupport`.
    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    pub struct CommandSupport: u32 {
        const SINGLE = 0x0001;
        const GLOBAL = 0x0002;
        const ONLINE = 0x0004;
        const MULTI = 0x0008;
        const AREA = 0x0010;
        const SELF = 0x0020;
        const REGION = 0x0040;
        const CONTAINED = 0x0080;
        const IP_ADDRESS = 0x0100;

        const ALL = 0x01FF;
        const ALL_MOBILES = 0x01FF & !0x0080;
        const ALL_NPCS = 0x01FF & !(0x0100 | 0x0004 | 0x0020 | 0x0080);
        const ALL_ITEMS = 0x01FF & !(0x0100 | 0x0004 | 0x0020);

        const SIMPLE = 0x0001 | 0x0008;
        const COMPLEX = 0x0002 | 0x0004 | 0x0010 | 0x0040 | 0x0080 | 0x0100;
    }
}

/// The fields every `BaseCommand` sets in its constructor.
pub struct CmdSpec {
    pub commands: Vec<String>,
    pub usage: String,
    pub description: String,
    pub access_level: AccessLevel,
    pub object_types: ObjectTypes,
    pub supports: CommandSupport,
    /// `ListOptimized`: a single object is passed to `execute_list` as a one element list.
    pub list_optimized: bool,
}

impl CmdSpec {
    pub fn new(
        commands: &[&str],
        access_level: AccessLevel,
        supports: CommandSupport,
        object_types: ObjectTypes,
        usage: &str,
        description: &str,
    ) -> CmdSpec {
        CmdSpec {
            commands: commands.iter().map(|s| (*s).to_string()).collect(),
            usage: usage.into(),
            description: description.into(),
            access_level,
            object_types,
            supports,
            list_optimized: false,
        }
    }

    pub fn list_optimized(mut self) -> CmdSpec {
        self.list_optimized = true;
        self
    }
}

/// The responses and failures `AddResponse` / `LogFailure` collect for one run (each message once with a count, ten
/// distinct messages at most), printed by `Flush`.
#[derive(Default)]
pub struct Run {
    responses: Vec<(String, u32)>,
    failures: Vec<(String, u32)>,
}

fn push_message(list: &mut Vec<(String, u32)>, message: &str) {
    if let Some(e) = list.iter_mut().find(|e| e.0 == message) {
        e.1 += 1;
    } else if list.len() < 10 {
        list.push((message.to_string(), 1));
    }
}

fn entry_text(e: &(String, u32)) -> String {
    if e.1 > 1 {
        format!("{} ({})", e.0, e.1)
    } else {
        e.0.clone()
    }
}

impl Run {
    pub fn add_response(&mut self, message: &str) {
        push_message(&mut self.responses, message);
    }

    pub fn log_failure(&mut self, message: &str) {
        push_message(&mut self.failures, message);
    }

    /// `Flush`: the responses if there are any, else the failures.
    pub fn flush(&mut self, w: &mut World, from: MobileId, flush_to_log: bool) {
        if !self.responses.is_empty() {
            for e in &self.responses {
                let text = entry_text(e);
                send_message(w, from, &text);
                if flush_to_log {
                    logging::write_line(w, from, &text);
                }
            }
        } else {
            for e in &self.failures {
                send_message(w, from, &entry_text(e));
            }
        }
        self.responses.clear();
        self.failures.clear();
    }
}

/// `BaseCommand`.
pub trait BaseCommand {
    fn spec(&self) -> &CmdSpec;

    /// `ExecuteList`: the default runs `execute` on each.
    fn execute_list(&self, w: &mut World, run: &mut Run, e: &CommandEventArgs, list: &[Obj]) {
        for o in list {
            self.execute(w, run, e, *o);
        }
    }

    fn execute(&self, _w: &mut World, _run: &mut Run, _e: &CommandEventArgs, _obj: Obj) {}

    /// `ValidateArgs`.
    fn validate_args(&self, _w: &mut World, _e: &mut CommandEventArgs) -> bool {
        true
    }
}

/// `BaseCommand.IsAccessible`: staff below Administrator can only touch their own mobile, or a lesser one (and
/// that mobile's items).
pub fn is_accessible(w: &World, from: MobileId, obj: Option<Obj>) -> bool {
    let Some(from_level) = w.mobile(from).map(|m| m.access_level) else {
        return true;
    };
    if from_level >= AccessLevel::Administrator {
        return true;
    }
    let mob = match obj {
        Some(Targeted::Entity(EntityId::Mobile(m))) => Some(m),
        Some(Targeted::Entity(EntityId::Item(i))) => match muo_server::item::ops::root_parent(w, i)
        {
            Some(EntityId::Mobile(m)) => Some(m),
            _ => None,
        },
        _ => None,
    };
    match mob {
        None => true,
        Some(m) => m == from || w.mobile(m).is_none_or(|x| from_level > x.access_level),
    }
}

/// `Obj` as an item.
pub fn as_item(o: Obj) -> Option<muo_server::ids::ItemId> {
    match o {
        Targeted::Entity(EntityId::Item(i)) => Some(i),
        _ => None,
    }
}

/// `Obj` as a mobile.
pub fn as_mobile(o: Obj) -> Option<MobileId> {
    match o {
        Targeted::Entity(EntityId::Mobile(m)) => Some(m),
        _ => None,
    }
}

/// All the registered target commands (`TargetCommands.AllCommands`) and the implementors that run them.
#[derive(Default)]
pub struct GenericCommands {
    pub all: Vec<Rc<dyn BaseCommand>>,
    pub implementors: Vec<implementors::Implementor>,
}

/// `TargetCommands.Configure`.
pub fn configure(w: &mut World) {
    implementors::register_implementors(w);
    commands::register_all(w);
}
