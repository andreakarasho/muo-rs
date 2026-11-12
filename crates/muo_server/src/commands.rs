//! Text commands typed with the command prefix: registration, access levels and dispatch.
//!
//! `CommandSystem`'s statics live in a [`CommandSystem`] resource (`w.resources.get_mut::<CommandSystem>()`).
//! `Register`'s `[Aliases]` attribute is the explicit `aliases` argument. Two entries have the same handler
//! (`Handler.Method ==`) when they hold the same `Rc`.

use muo_primitives::dotnet::{IgnoreCase, InvariantCase};
use std::any::Any;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::rc::Rc;

use indexmap::IndexMap;
use muo_primitives::parse::{to_bool, to_f64, to_i32, to_u32};
use muo_primitives::{MessageType, TimeSpan};

use crate::events;
use crate::ids::MobileId;
use crate::mobile::AccessLevel;
use crate::world::World;

/// `CommandEventHandler`.
pub type CommandEventHandler = Rc<dyn Fn(&mut World, &mut CommandEventArgs)>;

/// `CommandEventArgs`.
pub struct CommandEventArgs {
    pub mobile: MobileId,
    pub command: String,
    pub arg_string: String,
    pub arguments: Vec<String>,
    context: HashMap<String, Box<dyn Any>>,
}

impl CommandEventArgs {
    pub fn new(
        mobile: MobileId,
        command: String,
        arg_string: String,
        arguments: Vec<String>,
    ) -> Self {
        Self {
            mobile,
            command,
            arg_string,
            arguments,
            context: HashMap::new(),
        }
    }

    pub fn length(&self) -> usize {
        self.arguments.len()
    }

    fn arg(&self, index: i32) -> Option<&str> {
        usize::try_from(index)
            .ok()
            .and_then(|i| self.arguments.get(i))
            .map(String::as_str)
    }

    pub fn get_string(&self, index: i32) -> String {
        self.arg(index).unwrap_or("").to_string()
    }

    pub fn get_int32(&self, index: i32) -> i32 {
        self.arg(index).map_or(0, to_i32)
    }

    pub fn get_uint32(&self, index: i32) -> u32 {
        self.arg(index).map_or(0, to_u32)
    }

    pub fn get_boolean(&self, index: i32) -> bool {
        self.arg(index).is_some_and(to_bool)
    }

    pub fn get_double(&self, index: i32) -> f64 {
        self.arg(index).map_or(0.0, to_f64)
    }

    /// `Utility.ToTimeSpan`: zero when it doesn't parse.
    pub fn get_time_span(&self, index: i32) -> TimeSpan {
        self.arg(index)
            .and_then(TimeSpan::try_parse)
            .unwrap_or(TimeSpan::ZERO)
    }

    /// `GetContext<T>`: the value when stored under `key` with the type `T`.
    pub fn get_context<T: Any>(&self, key: &str) -> Option<&T> {
        self.context.get(key)?.downcast_ref()
    }

    /// `SetContext<T>`: stores the value and returns a reference to it.
    pub fn set_context<T: Any>(&mut self, key: &str, value: T) -> &T {
        self.context.insert(key.to_string(), Box::new(value));
        self.get_context(key).expect("just stored")
    }
}

/// `CommandEntry`.
#[derive(Clone)]
pub struct CommandEntry {
    pub command: String,
    pub handler: CommandEventHandler,
    pub access_level: AccessLevel,
}

impl CommandEntry {
    /// `CommandEntry.GetList`: sorted descending by name (ordinal), later entries sharing a handler with an
    /// earlier one removed.
    pub fn get_list(w: &mut World) -> Vec<CommandEntry> {
        let mut commands: Vec<CommandEntry> = w
            .resources
            .get_mut::<CommandSystem>()
            .entries
            .values()
            .cloned()
            .collect();
        commands.sort_by(|a, b| a.command.cmp(&b.command));
        commands.reverse();

        let mut i = 0;
        while i < commands.len() {
            let h = commands[i].handler.clone();
            let mut j = i + 1;
            while j < commands.len() {
                if Rc::ptr_eq(&h, &commands[j].handler) {
                    commands.remove(j);
                } else {
                    j += 1;
                }
            }
            i += 1;
        }
        commands
    }
}

/// `CommandInfo`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommandInfo {
    pub access_level: AccessLevel,
    pub name: String,
    pub aliases: Vec<String>,
    pub usage: String,
    pub modifiers: Vec<String>,
    pub description: String,
}

/// `CommandInfoSorter.Compare`: highest access level first, then name ordinal.
pub fn compare_command_info(a: &CommandInfo, b: &CommandInfo) -> Ordering {
    b.access_level
        .cmp(&a.access_level)
        .then_with(|| a.name.cmp(&b.name))
}

/// `CommandSystem`.
pub struct CommandSystem {
    pub prefix: String,
    /// Keyed by the lowercase command (lookups are case-insensitive).
    pub entries: IndexMap<String, CommandEntry>,
    pub bad_command_ignore_level: AccessLevel,
}

impl Default for CommandSystem {
    fn default() -> Self {
        Self {
            prefix: "[".to_string(),
            entries: IndexMap::new(),
            bad_command_ignore_level: AccessLevel::Player,
        }
    }
}

/// `CommandSystem.Split`: words split on spaces; a `"` starts a quoted word that runs to the next `"` not
/// preceded by a backslash (the backslash stays); the character after the closing quote is skipped.
pub fn split(value: &str) -> Vec<String> {
    let array: Vec<char> = value.chars().collect();
    let mut list = Vec::new();
    let mut start = 0;

    while start < array.len() {
        let c = array[start];

        if c == '"' {
            start += 1;
            let mut end = start;
            while end < array.len() && (array[end] != '"' || array[end - 1] == '\\') {
                end += 1;
            }
            list.push(array[start..end].iter().collect());
            start = end + 2;
        } else if c != ' ' {
            let mut end = start;
            while end < array.len() && array[end] != ' ' {
                end += 1;
            }
            list.push(array[start..end].iter().collect());
            start = end + 1;
        } else {
            start += 1;
        }
    }
    list
}

impl CommandSystem {
    /// `CommandSystem.Register`: the command, then each alias, with the same handler and access level.
    pub fn register(
        w: &mut World,
        command: &str,
        access: AccessLevel,
        aliases: &[&str],
        handler: CommandEventHandler,
    ) {
        let cs = w.resources.get_mut::<CommandSystem>();
        cs.do_register(command, access, handler.clone());
        for alias in aliases {
            cs.do_register(alias, access, handler.clone());
        }
    }

    fn do_register(
        &mut self,
        command: &str,
        access_level: AccessLevel,
        handler: CommandEventHandler,
    ) {
        let key = command.ic_key();
        if let Some(e) = self.entries.get(&key) {
            if e.access_level == access_level && Rc::ptr_eq(&e.handler, &handler) {
                return;
            }
            tracing::warn!("Command {} already registered to another handler.", command);
            return;
        }
        self.entries.insert(
            key,
            CommandEntry {
                command: command.to_string(),
                handler,
                access_level,
            },
        );
    }

    /// `CommandSystem.Handle`: true when the text was taken as a command (or answered with an error).
    pub fn handle(w: &mut World, from: MobileId, text: &str, kind: MessageType) -> bool {
        let (prefix, ignore) = {
            let cs = w.resources.get_mut::<CommandSystem>();
            (cs.prefix.clone(), cs.bad_command_ignore_level)
        };

        if !text.starts_with(&prefix) && kind != MessageType::COMMAND {
            return false;
        }

        let text = if kind != MessageType::COMMAND {
            &text[prefix.len()..]
        } else {
            text
        };

        let (command, arg_string, args) = match text.find(' ') {
            Some(i) => {
                let arg_string = &text[i + 1..];
                (
                    text[..i].to_string(),
                    arg_string.to_string(),
                    split(arg_string),
                )
            }
            None => (text.lower_inv(), String::new(), Vec::new()),
        };

        let entry = w
            .resources
            .get_mut::<CommandSystem>()
            .entries
            .get(&command.ic_key())
            .cloned();
        let level = w
            .mobile(from)
            .map_or(AccessLevel::Player, |m| m.access_level);

        match entry {
            Some(entry) => {
                if level >= entry.access_level {
                    let mut e = CommandEventArgs::new(from, command, arg_string, args);
                    (entry.handler)(w, &mut e);
                    events::invoke_command(w, &mut e);
                } else {
                    if level <= ignore {
                        return false;
                    }
                    send_message(w, from, "You do not have access to that command.");
                }
            }
            None => {
                if level <= ignore {
                    return false;
                }
                send_message(w, from, "That is not a valid command.");
            }
        }
        true
    }
}

/// `Mobile.SendMessage(string)`.
pub fn send_message(w: &mut World, to: MobileId, text: &str) {
    crate::mobile::part2::messages::send_message(w, to, text);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(v: &[&str]) -> Vec<String> {
        v.iter().map(|x| x.to_string()).collect()
    }

    #[test]
    fn split_words_and_quotes() {
        assert_eq!(split(""), Vec::<String>::new());
        assert_eq!(split("a  b c"), s(&["a", "b", "c"]));
        assert_eq!(split("a \"b c\" d"), s(&["a", "b c", "d"]));
        assert_eq!(split("\"a \\\" b\" c"), s(&["a \\\" b", "c"]));
        assert_eq!(split("\"unterminated"), s(&["unterminated"]));
    }

    #[test]
    fn register_aliases_and_dedupe() {
        let mut w = World::new();
        let h: CommandEventHandler = Rc::new(|_, _| {});
        CommandSystem::register(&mut w, "Go", AccessLevel::Counselor, &["Goto"], h.clone());
        CommandSystem::register(&mut w, "go", AccessLevel::Counselor, &[], h);
        let cs = w.resources.get_mut::<CommandSystem>();
        assert_eq!(cs.entries.len(), 2);
        assert!(cs.entries.contains_key("GOTO"));
        // aliases share a handler: one entry
        assert_eq!(CommandEntry::get_list(&mut w).len(), 1);
    }

    #[test]
    fn handle_runs_handler_and_event() {
        use std::cell::Cell;
        let mut w = World::new();
        let hit = Rc::new(Cell::new(0));
        let h2 = hit.clone();
        CommandSystem::register(
            &mut w,
            "Ping",
            AccessLevel::Player,
            &[],
            Rc::new(move |_, e| h2.set(e.get_int32(0) + e.length() as i32)),
        );
        let ev = Rc::new(Cell::new(false));
        let ev2 = ev.clone();
        w.events.command.subscribe(move |_, _| ev2.set(true));
        let who = MobileId(muo_primitives::Serial::new(1));
        assert!(CommandSystem::handle(
            &mut w,
            who,
            "[PING 5",
            MessageType::REGULAR
        ));
        assert_eq!(hit.get(), 6);
        assert!(ev.get());
        assert!(!CommandSystem::handle(
            &mut w,
            who,
            "hello",
            MessageType::REGULAR
        ));
        // unknown command from a player is ignored
        assert!(!CommandSystem::handle(
            &mut w,
            who,
            "[nope",
            MessageType::REGULAR
        ));
    }
}
