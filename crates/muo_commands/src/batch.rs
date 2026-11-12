//! `[Batch`, the `Batch` command object and its two grid gumps.
//!
//! The gump keeps the editable state ([`BatchState`])
//! and `Run` builds the command from a copy of it. `Scope.Commands[name]` throws `KeyNotFoundException` for a
//! command the scope does not have, and the outer `catch` sends its message: that is kept.
//!
//! shortcut: a batch command's `Object` property chain is read on item and mobile targets, and a property whose
//! value is not an item or a mobile is skipped (only commands over such values could use it).

use muo_primitives::dotnet::InvariantCase;
use std::rc::Rc;

use muo_content::gumps::base::{LegacyGumpLogic, LegacyGumpObj, RelayInfo};
use muo_content::props::{self, Access, Value};
use muo_primitives::html;
use muo_server::commands::{CommandEventArgs, CommandSystem, split};
use muo_server::ids::MobileId;
use muo_server::mobile::AccessLevel;
use muo_server::mobile::part2::messages::send_message;
use muo_server::net_state::NetStateId;
use muo_server::target::Targeted;
use muo_server::world::World;

use crate::generic::implementors::{self, find_command};
use crate::generic::{
    BaseCommand, CmdSpec, CommandSupport, GenericCommands, Obj, ObjectTypes, Run,
};
use crate::logging::CommandLogging;
use muo_content::gumps::base_grid::{
    ARROW_RIGHT_HEIGHT, ARROW_RIGHT_ID1, ARROW_RIGHT_ID2, ARROW_RIGHT_WIDTH, BaseGrid,
    get_button_id, send, split_button_id,
};

pub fn configure(w: &mut World) {
    CommandSystem::register(
        w,
        "Batch",
        AccessLevel::Counselor,
        &[],
        Rc::new(batch_on_command),
    );
}

/// `BatchCommand`.
#[derive(Clone, Default)]
pub struct BatchCommand {
    pub command: String,
    pub object: String,
}

impl BatchCommand {
    /// `GetDetails`: the command name, the argument text and the split arguments.
    fn details(&self) -> (String, String, Vec<String>) {
        match self.command.find(' ') {
            Some(i) => {
                let arg_string = self.command[i + 1..].to_string();
                let args = split(&arg_string);
                (self.command[..i].to_string(), arg_string, args)
            }
            None => (self.command.lower_inv(), String::new(), Vec::new()),
        }
    }
}

/// What `Batch` holds between gumps.
#[derive(Clone, Default)]
pub struct BatchState {
    /// Index into `GenericCommands::implementors`.
    pub scope: Option<usize>,
    pub condition: String,
    pub commands: Vec<BatchCommand>,
}

/// `[Usage("Batch")]`
fn batch_on_command(w: &mut World, e: &mut CommandEventArgs) {
    let g = new_batch_gump(e.mobile, BatchState::default(), None);
    send(w, e.mobile, g);
}

/// The `Batch` command as the scope runs it.
struct Batch {
    spec: CmdSpec,
    state: BatchState,
}

impl Batch {
    fn new(state: BatchState) -> Batch {
        let mut spec = CmdSpec::new(
            &["Batch"],
            AccessLevel::Player,
            CommandSupport::empty(),
            ObjectTypes::Both,
            "",
            "",
        );
        spec.list_optimized = true;
        Batch { spec, state }
    }
}

impl BaseCommand for Batch {
    fn spec(&self) -> &CmdSpec {
        &self.spec
    }

    fn execute_list(&self, w: &mut World, run: &mut Run, e: &CommandEventArgs, list: &[Obj]) {
        let from = e.mobile;
        if list.is_empty() {
            run.log_failure("Nothing was found to use this command on.");
            return;
        }
        if let Err(message) = self.run_all(w, from, list) {
            send_message(w, from, &message);
        }
    }
}

impl Batch {
    fn run_all(&self, w: &mut World, from: MobileId, list: &[Obj]) -> Result<(), String> {
        let Some(scope) = self.state.scope else {
            return Err("Object reference not set to an instance of an object.".into());
        };
        let level = w
            .mobile(from)
            .map_or(AccessLevel::Player, |m| m.access_level);
        let mut commands: Vec<Rc<dyn BaseCommand>> = Vec::new();
        let mut event_args: Vec<CommandEventArgs> = Vec::new();
        for bc in &self.state.commands {
            let (name, arg_string, args) = bc.details();
            let Some(command) = find_command(w, scope, &name) else {
                // `Scope.Commands[commandString]` is a Dictionary indexer.
                return Err(format!(
                    "The given key '{name}' was not present in the dictionary."
                ));
            };
            let mut ea = CommandEventArgs::new(from, name.clone(), arg_string, args);
            if level < command.spec().access_level {
                send_message(
                    w,
                    from,
                    &format!("You do not have access to that command: {name}."),
                );
                return Ok(());
            }
            if !command.validate_args(w, &mut ea) {
                return Ok(());
            }
            commands.push(command);
            event_args.push(ea);
        }
        for (i, command) in commands.iter().enumerate() {
            let bc = &self.state.commands[i];
            let big = list.len() > 20;
            if big {
                w.resources.get_mut::<CommandLogging>().enabled = false;
            }
            let mut used: Vec<Obj> = Vec::with_capacity(list.len());
            for obj in list {
                let Targeted::Entity(ent) = *obj else {
                    continue;
                };
                // Properties.GetPropertyInfoChain / GetPropertyInfo / GetValue
                let Ok((last, def)) = props::resolve_chain(w, level, ent, &bc.object, Access::READ)
                else {
                    continue;
                };
                if let Value::Entity(Some(next)) = (def.get)(w, last) {
                    used.push(Targeted::Entity(next));
                }
            }
            let mut run = Run::default();
            command.execute_list(w, &mut run, &event_args[i], &used);
            if big {
                w.resources.get_mut::<CommandLogging>().enabled = true;
            }
            run.flush(w, from, big);
        }
        Ok(())
    }
}

/// `Batch.Run(from)`.
fn run(w: &mut World, from: MobileId, state: &BatchState) -> bool {
    let Some(scope) = state.scope else {
        send_message(w, from, "You must select the batch command scope.");
        return false;
    };
    let supports = w
        .resources
        .get::<GenericCommands>()
        .is_some_and(|g| g.implementors[scope].supports_conditionals);
    if !state.condition.is_empty() && !supports {
        send_message(w, from, "This command scope does not support conditionals.");
        return false;
    }
    if !state.condition.is_empty() && !state.condition.lower_inv().starts_with("where") {
        send_message(w, from, "The condition field must start with \"where\".");
        return false;
    }
    let args = split(&state.condition);
    implementors::process(w, scope, from, Rc::new(Batch::new(state.clone())), args);
    true
}

// ---- BatchGump -------------------------------------------------------------------------------------------------

pub struct BatchGump {
    from: MobileId,
    state: BatchState,
}

impl LegacyGumpLogic for BatchGump {
    const TYPE_NAME: &'static str = "Server.Commands.BatchGump";

    fn on_response(&mut self, w: &mut World, _sender: NetStateId, info: &RelayInfo) {
        let Some((ty, index)) = split_button_id(info.button_id, 1) else {
            return;
        };
        let text = |i: i32| info.get_text_entry(i).unwrap_or("").to_string();
        self.state.condition = text(0);
        for i in (0..self.state.commands.len()).rev() {
            let sc = &mut self.state.commands[i];
            sc.command = text(1 + i as i32 * 2);
            sc.object = text(2 + i as i32 * 2);
            if sc.command.is_empty() && sc.object.is_empty() {
                self.state.commands.remove(i);
            }
        }
        if ty == 0 {
            match index {
                0 => {
                    run(w, self.from, &self.state);
                }
                1 => {
                    // set scope
                    let g = new_scope_gump(w, self.from, self.state.clone());
                    send(w, self.from, g);
                    return;
                }
                2 => self.state.commands.push(BatchCommand::default()),
                _ => {}
            }
        }
        let name = scope_name(w, &self.state);
        let g = new_batch_gump(self.from, self.state.clone(), name);
        send(w, self.from, g);
    }
}

pub fn new_batch_gump(
    from: MobileId,
    state: BatchState,
    scope_name: Option<String>,
) -> LegacyGumpObj<BatchGump> {
    let mut g = LegacyGumpObj::new(30, 30, BatchGump { from, state });
    let s = g.logic.state.clone();
    let l = &mut g.layout;
    let mut gr = BaseGrid::default();
    let arrow = |gr: &mut BaseGrid, l: &mut _, id: i32| {
        gr.add_entry_button(
            l,
            20,
            ARROW_RIGHT_ID1,
            ARROW_RIGHT_ID2,
            id,
            ARROW_RIGHT_WIDTH,
            ARROW_RIGHT_HEIGHT,
            1,
        );
    };
    gr.add_new_page(l);

    /* Header */
    gr.add_entry_header(l, 20, 1);
    gr.add_entry_html(l, 180, &html::center("Batch Commands"));
    gr.add_entry_header(l, 20, 1);
    gr.add_new_line();
    gr.add_entry_header(l, 9, 1);
    gr.add_entry_label(l, 191, "Run Batch");
    arrow(&mut gr, l, get_button_id(1, 0, 0));
    gr.add_new_line();
    gr.add_blank_line(l);

    /* Scope */
    gr.add_entry_header(l, 20, 1);
    gr.add_entry_html(l, 180, &html::center("Scope"));
    gr.add_entry_header(l, 20, 1);
    gr.add_new_line();
    gr.add_entry_header(l, 9, 1);
    let label = scope_name.unwrap_or_else(|| "Select Scope".to_string());
    gr.add_entry_label(l, 191, &label);
    arrow(&mut gr, l, get_button_id(1, 0, 1));
    gr.add_new_line();
    gr.add_blank_line(l);

    /* Condition */
    gr.add_entry_header(l, 20, 1);
    gr.add_entry_html(l, 180, &html::center("Condition"));
    gr.add_entry_header(l, 20, 1);
    gr.add_new_line();
    gr.add_entry_header(l, 9, 1);
    gr.add_entry_text(l, 202, 0, &s.condition);
    gr.add_entry_header(l, 9, 1);
    gr.add_new_line();
    gr.add_blank_line(l);

    /* Commands */
    gr.add_entry_header(l, 20, 1);
    gr.add_entry_html(l, 180, &html::center("Commands"));
    gr.add_entry_header(l, 20, 1);
    for (i, bc) in s.commands.iter().enumerate() {
        gr.add_new_line();
        let st = gr.style;
        let (x, y) = (gr.current_x, gr.current_y);
        l.add_image_tiled(x, y, 9, 2, 0x24A8);
        l.add_image_tiled(
            x,
            y + 2,
            2,
            st.entry_height + st.offset_size + st.entry_height - 4,
            0x24A8,
        );
        l.add_image_tiled(
            x,
            y + st.entry_height + st.offset_size + st.entry_height - 2,
            9,
            2,
            0x24A8,
        );
        l.add_image_tiled(
            x + 3,
            y + 3,
            6,
            st.entry_height + st.entry_height - 4 - st.offset_size,
            st.header_gump_id,
        );
        gr.increase_x(l, 9);
        gr.add_entry_text(l, 202, 1 + i as i32 * 2, &bc.command);
        gr.add_entry_header(l, 9, 2);
        gr.add_new_line();
        gr.increase_x(l, 9);
        gr.add_entry_text(l, 202, 2 + i as i32 * 2, &bc.object);
    }
    gr.add_new_line();
    gr.add_entry_header(l, 9, 1);
    gr.add_entry_label(l, 191, "Add New Command");
    arrow(&mut gr, l, get_button_id(1, 0, 2));
    gr.finish_page(l);
    g
}

/// The accessor of the scope a batch has chosen (`Scope.Accessors[0]`).
fn scope_name(w: &World, state: &BatchState) -> Option<String> {
    let i = state.scope?;
    w.resources
        .get::<GenericCommands>()
        .map(|g| g.implementors[i].accessors[0].to_string())
}

// ---- BatchScopeGump --------------------------------------------------------------------------------------------

pub struct BatchScopeGump {
    from: MobileId,
    state: BatchState,
}

impl LegacyGumpLogic for BatchScopeGump {
    const TYPE_NAME: &'static str = "Server.Commands.BatchScopeGump";

    fn on_response(&mut self, w: &mut World, _sender: NetStateId, info: &RelayInfo) {
        if let Some((0, index)) = split_button_id(info.button_id, 1) {
            let level = w
                .mobile(self.from)
                .map_or(AccessLevel::Player, |m| m.access_level);
            let ok = w.resources.get::<GenericCommands>().is_some_and(|g| {
                index >= 0
                    && (index as usize) < g.implementors.len()
                    && level >= g.implementors[index as usize].access_level
            });
            if ok {
                self.state.scope = Some(index as usize);
            }
        }
        let name = scope_name(w, &self.state);
        let g = new_batch_gump(self.from, self.state.clone(), name);
        send(w, self.from, g);
    }
}

pub fn new_scope_gump(
    w: &World,
    from: MobileId,
    state: BatchState,
) -> LegacyGumpObj<BatchScopeGump> {
    let mut g = LegacyGumpObj::new(30, 30, BatchScopeGump { from, state });
    let level = w
        .mobile(from)
        .map_or(AccessLevel::Player, |m| m.access_level);
    let l = &mut g.layout;
    let mut gr = BaseGrid::default();
    gr.add_new_page(l);

    /* Header */
    gr.add_entry_header(l, 20, 1);
    gr.add_entry_html(l, 140, &html::center("Change Scope"));
    gr.add_entry_header(l, 20, 1);

    /* Options */
    if let Some(gc) = w.resources.get::<GenericCommands>() {
        for (i, imp) in gc.implementors.iter().enumerate() {
            if level < imp.access_level {
                continue;
            }
            gr.add_new_line();
            let width = 20 + gr.style.offset_size + 140;
            gr.add_entry_label(l, width, imp.accessors[0]);
            gr.add_entry_button(
                l,
                20,
                ARROW_RIGHT_ID1,
                ARROW_RIGHT_ID2,
                get_button_id(1, 0, i as i32),
                ARROW_RIGHT_WIDTH,
                ARROW_RIGHT_HEIGHT,
                1,
            );
        }
    }
    gr.finish_page(l);
    g
}
