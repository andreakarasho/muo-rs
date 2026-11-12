//! `[interface`, which lists the objects a scope found in a gump.
//!
//!
//! The gump itself is `muo_gumps::a_l::interface`, installed in `CommandHooks::interface_gump`; without it the
//! command prints the count.

use muo_primitives::dotnet::IgnoreCase;
use muo_server::commands::CommandEventArgs;
use muo_server::ids::EntityId;
use muo_server::mobile::AccessLevel;
use muo_server::world::World;

use super::{BaseCommand, CmdSpec, CommandSupport, Obj, ObjectTypes, Run};
use crate::hooks::CommandHooks;

pub struct InterfaceCommand {
    spec: CmdSpec,
}

impl InterfaceCommand {
    pub fn new() -> Self {
        Self {
            spec: CmdSpec::new(
                &["Interface"],
                AccessLevel::GameMaster,
                CommandSupport::COMPLEX | CommandSupport::SIMPLE,
                ObjectTypes::Both,
                "Interface [view <properties ...>]",
                "Opens an interface to interact with matched objects. Generally used with condition arguments.",
            )
            .list_optimized(),
        }
    }
}

impl Default for InterfaceCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl BaseCommand for InterfaceCommand {
    fn spec(&self) -> &CmdSpec {
        &self.spec
    }

    fn execute_list(&self, w: &mut World, run: &mut Run, e: &CommandEventArgs, list: &[Obj]) {
        if list.is_empty() {
            return run.add_response("No matching objects found.");
        }
        // The columns after "Object": the property names, `view` itself skipped.
        let mut columns = Vec::new();
        if e.length() > 0 {
            let mut offset = 0;
            if e.get_string(0).eq_ic("view") {
                offset = 1;
            }
            columns.extend(e.arguments[offset..].iter().cloned());
        }
        let ents: Vec<EntityId> = list
            .iter()
            .filter_map(|o| match o {
                muo_server::target::Targeted::Entity(e) => Some(*e),
                _ => None,
            })
            .collect();
        match w.resources.get_mut::<CommandHooks>().interface_gump {
            Some(g) => g(w, e.mobile, ents, columns),
            None => run.add_response(&format!(
                "{} objects matched; the interface gump is not available.",
                ents.len()
            )),
        }
    }
}
