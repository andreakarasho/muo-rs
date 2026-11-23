//! Rhai scripts: `Data/Scripts/**/*.rhai`, recompiled in game with `[reloadscripts`.
//!
//! - Per-class hooks: a template's `"script": "<path under Data/Scripts>"` binds the script to that class and every
//!   class below it (the class chain, then each template's `inherits`). The script may define `on_use(item, from)`,
//!   `on_death(m, corpse)`, `on_think(m)`, `on_damage(m, amount, from)`, `on_hear(m, from, text)` and
//!   `on_spawn(e)`; returning `true` skips the default behavior (see `muo_server::script_hooks`).
//! - Headers: doc comments on a function (rhai's `metadata`), one tag per line:
//!   `/// @command Battle` + optional `/// @access GameMaster` makes `fn x(from, args)` the command `[Battle`;
//!   `/// @event login` (`login(m)`, `speech(m, text)`, `server_started()`, `skill_gain(m, skill)`, `quest_log(m)`) runs it on that
//!   event; `true` from `speech` marks the speech handled, from `skill_gain` refuses the gain. An unknown tag, an
//!   unknown level or event, a wrong parameter count or a command defined twice is a load error, and so is a function
//!   still named the retired way (`command_<name>`, `on_login`, `on_speech`, `on_server_started`) without a header.
//!
//! - Classes: `class(name, #{..}, |e| {..})` at a script's top level declares a class template (see `classes`).
//!
//! A script's top level runs once, when the templates load, to collect its `class` declarations; after that only
//! functions run. Functions resolve by name at call time, so a reload applies immediately. A script error (including a runaway loop hitting the operation limit) is
//! logged with the file and function, and the default behavior runs.

mod api;
mod api_effects;
mod api_engines;
mod api_equipment;
mod api_interaction;
mod api_item_more;
mod api_misc_items;
mod api_misc_items2;
mod api_mobile_more;
mod api_mobiles;
mod api_npcs;
pub mod api_store;
mod api_world;
mod classes;
pub mod friendly;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_effects;
#[cfg(test)]
mod tests_interaction;
#[cfg(test)]
mod tests_item_more;
#[cfg(test)]
mod tests_mobile_more;
#[cfg(test)]
mod tests_world;

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::rc::Rc;

use crate::templates::{self, Kind, TemplateTable};
use muo_primitives::dotnet::IgnoreCase;
use muo_server::commands::{CommandEventArgs, CommandEventHandler, CommandSystem};
use muo_server::ids::EntityId;
use muo_server::mobile::AccessLevel;
use muo_server::script_hooks::{self, ScriptArg, ScriptHook, ScriptHooks};
use muo_server::type_registry::{Phase, TypeRegistry};
use muo_server::world::World;
use rhai::{AST, Dynamic, Engine, FnPtr, Scope};

/// One compiled file.
pub struct Script {
    /// The path under `Data/Scripts`, `/`-separated (what a template's `"script"` names).
    pub name: String,
    pub ast: AST,
    /// The commands it defines.
    commands: Vec<ScriptCommand>,
    /// `(GLOBALS index, function)` of each `@event` header.
    events: Vec<(usize, String)>,
}

/// `[name` at `access` runs `function(from, args)`.
#[derive(Clone)]
struct ScriptCommand {
    name: String,
    access: AccessLevel,
    function: String,
}

/// The global events: the retired function name (now an error without a header), parameter count, and the
/// `@event` name.
const GLOBALS: [(&str, usize, &str); 5] = [
    ("on_login", 1, "login"),
    ("on_speech", 2, "speech"),
    ("on_server_started", 0, "server_started"),
    ("", 2, "skill_gain"), // header only: there never was an old-style name
    ("", 1, "quest_log"),
];

/// Every compiled script, with the class bindings. Lives in `w.resources` as `Rc<ScriptHost>`: clone the `Rc`
/// before a call so the world is free during it.
pub struct ScriptHost {
    engine: Engine,
    pub scripts: Vec<Script>,
    /// The constructor closures of the `class` declarations: (script, closure); `Template::ctor` indexes this.
    ctors: Vec<(usize, FnPtr)>,
    /// The hook closures of the declarations: (class, hook name, script, closure).
    class_hooks: Vec<(String, String, usize, FnPtr)>,
    /// `(script, function)` handling each of [`GLOBALS`].
    globals: [Vec<(usize, String)>; 5],
}

/// class key (`script_hooks::*_class_key`) -> script index, for classes whose script defines a hook. Rebuilt by
/// [`install`] after every template load.
#[derive(Default)]
struct ClassScripts {
    mobiles: HashMap<usize, usize>,
    items: HashMap<usize, usize>,
    /// (class key, hook bit) -> the declaration's hook closure (script, closure).
    closures: HashMap<(usize, u8), (usize, FnPtr)>,
}

/// What [`ScriptHost::load`] gives: the host, the class declarations, the other declared content
/// (kind, name, value) and the errors.
type Loaded = (
    ScriptHost,
    Vec<classes::Decl>,
    Vec<(String, String, Dynamic)>,
    Vec<String>,
);

/// Script calls in progress: a script action can raise a hook that runs a script again.
#[derive(Default)]
struct Depth(u32);
const MAX_DEPTH: u32 = 8;

/// The commands registered from scripts, unregistered on reload.
#[derive(Default)]
struct ScriptCommands(Vec<(String, CommandEventHandler)>);

impl ScriptHost {
    /// The parameter count of `name` in the script: the one taking `want` if it is overloaded, else any.
    fn arity(&self, script: usize, name: &str, want: usize) -> Option<usize> {
        let mut found = None;
        for f in self.scripts[script]
            .ast
            .iter_functions()
            .filter(|f| f.name == name)
        {
            if f.params.len() == want {
                return Some(want);
            }
            found = Some(f.params.len());
        }
        found
    }

    fn has_fn(&self, script: usize, name: &str, arity: usize) -> bool {
        self.scripts[script]
            .ast
            .iter_functions()
            .any(|f| f.name == name && f.params.len() == arity)
    }

    /// Calls `name` in script `script`; `None` (logged) on a script error or a too-deep nesting.
    fn call(
        &self,
        w: &mut World,
        script: usize,
        name: &str,
        args: Vec<Dynamic>,
    ) -> Option<Dynamic> {
        match self.call_result(w, script, name, args) {
            Ok(v) => Some(v),
            Err(e) => {
                tracing::error!("Script {}: {name}: {e}", self.scripts[script].name);
                None
            }
        }
    }

    /// [`ScriptHost::call`] returning the error instead of logging it.
    fn call_result(
        &self,
        w: &mut World,
        script: usize,
        name: &str,
        args: Vec<Dynamic>,
    ) -> Result<Dynamic, String> {
        let s = &self.scripts[script];
        let depth = &mut w.resources.get_mut::<Depth>().0;
        if *depth >= MAX_DEPTH {
            return Err(format!("nested deeper than {MAX_DEPTH} script calls"));
        }
        *depth += 1;
        // The pointer is the only way the world is reached during the call; `w` is not touched again until
        // `call_fn_with_options` returns (see `api::with_world` for the SAFETY argument).
        let ctx = api::Ctx {
            world: std::ptr::from_mut(w),
            script: s.name.as_str().into(),
        };
        // through a function pointer: unlike `call_fn`, it runs the function with its file's `import`s
        let mut global = self.engine.new_global_runtime_state();
        global.lib.push(s.ast.shared_lib().clone());
        global.tag = Dynamic::from(ctx);
        let call: rhai::NativeCallContext =
            (&self.engine, name, None, &global, rhai::Position::NONE).into();
        let mut args = args;
        let r = match s.ast.shared_lib().get_script_fn(name, args.len()) {
            Some(f) => FnPtr::from(f.clone()).call_raw(&call, None, args.as_mut_slice()),
            None => Err(format!("Function not found: {name} ({} arguments)", args.len()).into()),
        };
        w.resources.get_mut::<Depth>().0 -= 1;
        r.map_err(|e| match *e {
            rhai::EvalAltResult::ErrorInFunctionCall(f, _, inner, _) if f == name => {
                inner.to_string()
            }
            e => e.to_string(),
        })
    }

    /// Compiles every `.rhai` under `dir` and runs each top level once, collecting its `class` declarations. A file
    /// that fails is left out and named in the errors.
    pub(crate) fn load(dir: &Path, types: &TypeRegistry) -> Loaded {
        let mut engine = api::engine();
        // `import "raids/places" as places;` names a file under `Data/Scripts`
        engine.set_module_resolver(rhai::module_resolvers::FileModuleResolver::new_with_path(
            dir,
        ));
        api::register_steps(&mut engine, types);
        let collector = Rc::new(std::cell::RefCell::new(classes::Collector::default()));
        classes::register(&mut engine, &collector);
        friendly::register(&mut engine, &collector);
        let mut files = Vec::new();
        rhai_files(dir, &mut files);
        let mut scripts = Vec::new();
        let mut errors = Vec::new();
        for f in files {
            let name = f
                .strip_prefix(dir)
                .unwrap_or(&f)
                .to_string_lossy()
                .replace('\\', "/");
            let text = match std::fs::read_to_string(&f) {
                Ok(t) => t,
                Err(e) => {
                    errors.push(format!("{name}: {e}"));
                    continue;
                }
            };
            match engine.compile(&text) {
                Ok(ast) => {
                    let (commands, events) = parse_headers(&name, &ast, &mut errors);
                    scripts.push(Script {
                        name,
                        ast,
                        commands,
                        events,
                    });
                }
                Err(e) => errors.push(format!("{name}: {e}")),
            }
        }
        // a command is defined once: a second definition (in any style, any script) is an error and left out
        let mut seen: HashMap<String, String> = HashMap::new();
        for sc in &mut scripts {
            let file = sc.name.clone();
            sc.commands
                .retain(|c| match seen.get(&c.name.to_ascii_lowercase()) {
                    Some(first) => {
                        errors.push(format!(
                            "{file}: command {} is already defined in {first}",
                            c.name
                        ));
                        false
                    }
                    None => {
                        seen.insert(c.name.to_ascii_lowercase(), file.clone());
                        true
                    }
                });
        }
        for (i, sc) in scripts.iter_mut().enumerate() {
            collector.borrow_mut().current = i;
            // run as a module: its functions keep the file's `import`s for the calls the server makes later
            match rhai::Module::eval_ast_as_new(Scope::new(), &sc.ast, &engine) {
                Ok(m) => sc.ast = AST::new(Vec::<rhai::Stmt>::new(), m),
                Err(e) => errors.push(format!("{}: {e}", sc.name)),
            }
        }
        let decls = std::mem::take(&mut collector.borrow_mut().decls);
        let data = std::mem::take(&mut collector.borrow_mut().data);
        let mut host = ScriptHost {
            engine,
            scripts,
            ctors: Vec::new(),
            class_hooks: Vec::new(),
            globals: Default::default(),
        };
        for (i, sc) in host.scripts.iter().enumerate() {
            for (g, f) in &sc.events {
                host.globals[*g].push((i, f.clone()));
            }
        }
        (host, decls, data, errors)
    }
}

fn rhai_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return; // an absent folder is an empty one
    };
    let mut entries: Vec<_> = rd.filter_map(Result::ok).map(|e| e.path()).collect();
    entries.sort();
    for p in entries {
        if p.is_dir() {
            rhai_files(&p, out);
        } else if p
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("rhai"))
        {
            out.push(p);
        }
    }
}

fn access_level(name: &str) -> Option<AccessLevel> {
    (0..=6)
        .filter_map(AccessLevel::from_i32)
        .find(|l| format!("{l:?}").eq_ignore_ascii_case(name))
}

/// The `@command` / `@access` / `@event` doc-comment headers of the script's functions.
fn parse_headers(
    file: &str,
    ast: &AST,
    errors: &mut Vec<String>,
) -> (Vec<ScriptCommand>, Vec<(usize, String)>) {
    let (mut commands, mut events) = (Vec::new(), Vec::new());
    for f in ast.iter_functions() {
        let mut fail = |msg: String| errors.push(format!("{file}: fn {}: {msg}", f.name));
        let (mut command, mut access) = (None, None);
        // rhai joins a run of `///` lines into one entry
        for line in f.comments.iter().flat_map(|c| c.lines()) {
            let text = line
                .trim_start_matches(['/', '*'])
                .trim_end_matches(['*', '/'])
                .trim();
            let Some(tag) = text.strip_prefix('@') else {
                continue; // a plain doc comment
            };
            let (tag, value) = tag.split_once(char::is_whitespace).unwrap_or((tag, ""));
            let value = value.trim();
            match tag {
                "command" if !value.is_empty() => command = Some(value.to_owned()),
                "access" => match access_level(value) {
                    Some(l) => access = Some(l),
                    None => fail(format!("unknown access level '{value}'")),
                },
                "event" => match GLOBALS.iter().position(|g| g.2 == value) {
                    Some(g) if f.params.len() == GLOBALS[g].1 => {
                        events.push((g, f.name.to_owned()))
                    }
                    Some(g) => fail(format!("@event {value} takes {} parameters", GLOBALS[g].1)),
                    None => fail(format!(
                        "unknown event '{value}' (login, speech, server_started)"
                    )),
                },
                _ => fail(format!(
                    "unknown header '@{tag}' (@command Name, @access Level, @event name)"
                )),
            }
        }
        if command.is_none() && events.last().is_none_or(|(_, e)| e != f.name) {
            if let Some(rest) = f.name.strip_prefix("command_") {
                fail(format!(
                    "name the command in a header instead: /// @command {rest}"
                ));
            } else if let Some(g) = GLOBALS.iter().find(|g| !g.0.is_empty() && g.0 == f.name) {
                fail(format!(
                    "handle the event with a header instead: /// @event {}",
                    g.2
                ));
            }
        }
        match command {
            Some(name) if f.params.len() == 2 => commands.push(ScriptCommand {
                name,
                access: access.unwrap_or(AccessLevel::GameMaster),
                function: f.name.to_owned(),
            }),
            Some(_) => fail("a command takes 2 parameters (from, args)".into()),
            None if access.is_some() => fail("@access without @command".into()),
            None => {}
        }
    }
    (commands, events)
}

fn host(w: &World) -> Option<Rc<ScriptHost>> {
    w.resources.get::<Rc<ScriptHost>>().cloned()
}

/// The first `"script"` along the class chain and each class's template `inherits` chain.
fn script_of(
    t: &TemplateTable,
    kind: Kind,
    mut names: impl Iterator<Item = &'static str>,
) -> Option<&str> {
    names.find_map(|n| t.chain(kind, n).find_map(|tm| tm.script.as_deref()))
}

/// Binds the classes to scripts, installs the host and the hook table, and re-registers the script commands.
/// Returns the problems found (a template naming a script that is not loaded).
/// Binds the classes of the loaded templates to their scripts and registers the script commands.
fn install(w: &mut World) -> Vec<String> {
    let Some(host) = host(w) else {
        return Vec::new();
    };
    let by_name: HashMap<&str, usize> = host
        .scripts
        .iter()
        .enumerate()
        .map(|(i, s)| (s.name.as_str(), i))
        .collect();
    let mut hooks = ScriptHooks {
        call: Some(dispatch),
        ..ScriptHooks::default()
    };
    let mut missing = HashSet::new();
    let (mut mobiles, mut items) = (HashMap::new(), HashMap::new());
    if let Some(t) = templates::loaded(w) {
        let mut bind = |name: Option<&str>| -> Option<(usize, u8)> {
            let name = name?.replace('\\', "/");
            let Some(&i) = by_name.get(name.as_str()) else {
                missing.insert(name);
                return None;
            };
            let bits = ScriptHook::ALL
                .iter()
                .filter(|h| {
                    let (f, n) = h.fn_name();
                    // `on_spawn` takes the constructor's arguments after the entity, so any arity of 1 or more
                    match h {
                        ScriptHook::Spawn => host.arity(i, f, n).is_some_and(|a| a >= 1),
                        _ => host.has_fn(i, f, n),
                    }
                })
                .fold(0, |b, h| b | h.bit());
            (bits != 0).then_some((i, bits))
        };
        for &ty in w.types.mobile_types() {
            let names = std::iter::successors(Some(ty), |t| t.parent).map(|t| t.full_name);
            if let Some((i, bits)) = bind(script_of(t, Kind::Mobile, names)) {
                let k = script_hooks::mobile_class_key(ty);
                mobiles.insert(k, i);
                hooks.mobiles.insert(k, bits);
            }
        }
        for &ty in w.types.item_types() {
            let names = std::iter::successors(Some(ty), |t| t.parent).map(|t| t.full_name);
            if let Some((i, bits)) = bind(script_of(t, Kind::Item, names)) {
                let k = script_hooks::item_class_key(ty);
                items.insert(k, i);
                hooks.items.insert(k, bits);
            }
        }
    }
    // hook closures of `mobile(..)` / `item(..)` declarations: the class's own, else the nearest ancestor's
    let mut closures = HashMap::new();
    let by_class: HashMap<(&str, ScriptHook), (usize, &FnPtr)> = host
        .class_hooks
        .iter()
        .filter_map(|(class, hook, script, f)| {
            let h = ScriptHook::ALL
                .into_iter()
                .find(|h| h.fn_name().0 == hook)?;
            Some(((class.as_str(), h), (*script, f)))
        })
        .collect();
    if !by_class.is_empty() {
        let t = templates::loaded(w);
        let names_of = |first: &'static str, kind: Kind| -> Vec<String> {
            let mut out = vec![first.to_owned()];
            if let Some(t) = t {
                out.extend(t.chain(kind, first).skip(1).map(|tm| tm.type_name.clone()));
            }
            out
        };
        let mut bind = |key: usize, chain: Vec<String>, bits: &mut u8| {
            for h in ScriptHook::ALL {
                if let Some((s, f)) = chain.iter().find_map(|n| by_class.get(&(n.as_str(), h))) {
                    closures.insert((key, h.bit()), (*s, (*f).clone()));
                    *bits |= h.bit();
                }
            }
        };
        for &ty in w.types.mobile_types() {
            let chain: Vec<String> = std::iter::successors(Some(ty), |t| t.parent)
                .flat_map(|t| names_of(t.full_name, Kind::Mobile))
                .collect();
            let k = script_hooks::mobile_class_key(ty);
            let mut bits = hooks.mobiles.get(&k).copied().unwrap_or(0);
            bind(k, chain, &mut bits);
            if bits != 0 {
                hooks.mobiles.insert(k, bits);
            }
        }
        for &ty in w.types.item_types() {
            let chain: Vec<String> = std::iter::successors(Some(ty), |t| t.parent)
                .flat_map(|t| names_of(t.full_name, Kind::Item))
                .collect();
            let k = script_hooks::item_class_key(ty);
            let mut bits = hooks.items.get(&k).copied().unwrap_or(0);
            bind(k, chain, &mut bits);
            if bits != 0 {
                hooks.items.insert(k, bits);
            }
        }
    }
    w.resources.insert(ClassScripts {
        mobiles,
        items,
        closures,
    });
    let commands: Vec<(String, ScriptCommand)> = host
        .scripts
        .iter()
        .flat_map(|s| s.commands.iter().map(|c| (s.name.clone(), c.clone())))
        .collect();
    w.resources.insert(hooks);
    register_commands(w, commands);
    let mut missing: Vec<String> = missing
        .into_iter()
        .map(|m| format!("a template names script {m}, which is not loaded"))
        .collect();
    missing.sort();
    missing
}

fn register_commands(w: &mut World, commands: Vec<(String, ScriptCommand)>) {
    let old = std::mem::take(&mut w.resources.get_mut::<ScriptCommands>().0);
    let cs = w.resources.get_mut::<CommandSystem>();
    for (name, h) in old {
        let key = name.ic_key();
        if cs
            .entries
            .get(&key)
            .is_some_and(|e| Rc::ptr_eq(&e.handler, &h))
        {
            cs.entries.shift_remove(&key);
        }
    }
    let mut ours = Vec::new();
    for (script, c) in commands {
        let function = c.function.clone();
        let h: CommandEventHandler = Rc::new(move |w: &mut World, e: &mut CommandEventArgs| {
            run_command(w, &script, &function, e)
        });
        CommandSystem::register(w, &c.name, c.access, &[], h.clone());
        ours.push((c.name, h));
    }
    w.resources.get_mut::<ScriptCommands>().0 = ours;
}

/// `fn_name` in the script that defined the command, else in any script (resolved now: a reload applies).
fn run_command(w: &mut World, script: &str, fn_name: &str, e: &mut CommandEventArgs) {
    let Some(host) = host(w) else { return };
    let has = |i: usize| host.has_fn(i, fn_name, 2);
    let mine = (0..host.scripts.len()).find(|&i| host.scripts[i].name == script && has(i));
    let Some(i) = mine.or_else(|| (0..host.scripts.len()).find(|&i| has(i))) else {
        muo_server::commands::send_message(w, e.mobile, "That script command is no longer loaded.");
        return;
    };
    let args = vec![api::mobile(e.mobile), e.arg_string.clone().into()];
    host.call(w, i, fn_name, args);
}

/// The `ScriptHooks::call` of the hook table.
fn dispatch(w: &mut World, e: EntityId, hook: ScriptHook, args: &[ScriptArg]) -> bool {
    let Some(host) = host(w) else { return false };
    // a declaration's hook closure first
    let key = match e {
        EntityId::Mobile(m) => w.mobile(m).map(|m| script_hooks::mobile_class_key(m.ty)),
        EntityId::Item(i) => w.item(i).map(|i| script_hooks::item_class_key(i.ty)),
    };
    let closure = key.and_then(|k| {
        w.resources
            .get::<ClassScripts>()?
            .closures
            .get(&(k, hook.bit()))
            .cloned()
    });
    if let Some((script, f)) = closure {
        let mut v: Vec<Dynamic> = f.curry().to_vec();
        v.push(api::entity(Some(e)));
        v.extend(args.iter().map(|a| api::arg(*a)));
        if let Some(n) = host.arity(script, f.fn_name(), v.len()) {
            v.resize(n.max(f.curry().len() + 1), Dynamic::UNIT);
        }
        return host
            .call(w, script, f.fn_name(), v)
            .is_some_and(|r| r.as_bool().unwrap_or(false));
    }
    let script = match e {
        EntityId::Mobile(m) => w.mobile(m).and_then(|m| {
            let c = w.resources.get::<ClassScripts>()?;
            c.mobiles
                .get(&script_hooks::mobile_class_key(m.ty))
                .copied()
        }),
        EntityId::Item(i) => w.item(i).and_then(|i| {
            let c = w.resources.get::<ClassScripts>()?;
            c.items.get(&script_hooks::item_class_key(i.ty)).copied()
        }),
    };
    let Some(script) = script else { return false };
    let mut v = Vec::with_capacity(args.len() + 1);
    v.push(api::entity(Some(e)));
    v.extend(args.iter().map(|a| api::arg(*a)));
    if hook == ScriptHook::Spawn {
        // fit the constructor's arguments to the parameters the script declares: `on_spawn(e)` ignores them,
        // a parameter the constructor did not pass is `()`
        let n = host.arity(script, "on_spawn", v.len()).unwrap_or(1);
        v.resize(n.max(1), Dynamic::UNIT);
    }
    host.call(w, script, hook.fn_name().0, v)
        .is_some_and(|r| r.as_bool().unwrap_or(false))
}

/// Whether a script refuses `m` a gain in `skill` (a name, `"Swords"`): `@event skill_gain` returning `true`. Asked
/// before every skill gain of a player (`SkillCheck.AllowGain`); stat gains come from skill gains, so they stop too.
pub fn skill_gain_refused(w: &mut World, m: muo_server::ids::MobileId, skill: &str) -> bool {
    if host(w).is_none_or(|h| h.globals[3].is_empty()) {
        return false;
    }
    call_global(w, 3, || vec![api::mobile(m), skill.into()])
}

/// Whether a script showed `m` its quest log: `@event quest_log` returning `true`, asked when the paperdoll's
/// Quests button is pressed, before the engine's own log.
pub fn quest_log_shown(w: &mut World, m: muo_server::ids::MobileId) -> bool {
    host(w).is_some_and(|h| !h.globals[4].is_empty()) && call_global(w, 4, || vec![api::mobile(m)])
}

/// Runs global hook `GLOBALS[g]` in every script defining it; true when any returned `true`.
fn call_global(w: &mut World, g: usize, args: impl Fn() -> Vec<Dynamic>) -> bool {
    let Some(host) = host(w) else { return false };
    let mut handled = false;
    for (i, f) in &host.globals[g] {
        let r = host.call(w, *i, f, args());
        handled |= r.is_some_and(|r| r.as_bool().unwrap_or(false));
    }
    handled
}

/// A `timer(ms, "fn", arg)` firing: `fn` in the script that started it, else in any script (resolved now, so a
/// reload in between applies).
fn fire_timer(w: &mut World, script: &str, name: &str, arg: Dynamic) {
    if !call_named(w, script, name, vec![arg]) {
        tracing::warn!("Script {script}: timer: no function {name}(arg)");
    }
}

/// Calls script function `name` of any script with `args` and returns what it returned (`None` when no script
/// defines it or it failed). For tests.
#[cfg(test)]
pub(crate) fn call_value(w: &mut World, name: &str, args: Vec<Dynamic>) -> Result<Dynamic, String> {
    let host = host(w).ok_or("no script host")?;
    let i = (0..host.scripts.len())
        .find(|&i| host.has_fn(i, name, args.len()))
        .ok_or_else(|| format!("no function {name}"))?;
    host.call_result(w, i, name, args)
}

/// Calls script function `name` with `args`: the one in script `script` if it has it, else the first script
/// defining it with that many parameters (resolved now, so a reload in between applies). False when none does.
pub(crate) fn call_named(w: &mut World, script: &str, name: &str, args: Vec<Dynamic>) -> bool {
    let Some(host) = host(w) else { return false };
    let n = host.scripts.len();
    let arity = args.len();
    let found = (0..n)
        .find(|&i| host.scripts[i].name == script && host.has_fn(i, name, arity))
        .or_else(|| (0..n).find(|&i| host.has_fn(i, name, arity)));
    match found {
        Some(i) => {
            host.call(w, i, name, args);
            true
        }
        None => false,
    }
}

/// `Data/Scripts` under the server folder.
pub fn scripts_dir(w: &World) -> PathBuf {
    w.core.base_directory.join("Data").join("Scripts")
}

/// Reloads the templates (JSON and the scripts' `class` declarations, recompiling every script) and rebinds the
/// classes. A load error keeps everything that was loaded before and is returned.
pub fn reload_scripts(w: &mut World) -> Vec<String> {
    let mut errors: Vec<String> = templates::reload(w).err().into_iter().collect();
    errors.extend(install(w));
    errors
}

/// The template source: compiles `Data/Scripts`, turns its `class` declarations into templates, and makes the new
/// host current. A strict (server) load fails on any script error, so a broken file never leaves classes blank.
pub(crate) fn source(w: &mut World, lenient: bool, t: &mut TemplateTable) -> Result<(), String> {
    let Some(dir) = templates::data_root(w).map(|d| d.join("Scripts")) else {
        return Ok(());
    };
    let (mut host, decls, data, errors) = ScriptHost::load(&dir, &w.types);
    if !errors.is_empty() {
        if !lenient {
            return Err(format!("Scripts: {}", errors.join("; ")));
        }
        for e in &errors {
            tracing::error!("Scripts: {e}");
        }
    }
    let names: Vec<String> = host.scripts.iter().map(|s| s.name.clone()).collect();
    let (mut ctors, mut class_hooks) = (Vec::new(), Vec::new());
    classes::build(w, decls, &names, lenient, t, &mut ctors, &mut class_hooks)?;
    host.ctors = ctors;
    host.class_hooks = class_hooks;
    for (kind, name, v) in data {
        let mut v = api::to_json(&v).map_err(|e| format!("Scripts: {kind} {name}: {e}"))?;
        friendly::resolve_shop_names(w, &mut v, !lenient)
            .map_err(|e| format!("Scripts: {kind} {name}: {e}"))?;
        let list = t.data.entry(kind.clone()).or_default();
        if list.iter().any(|(n, _)| *n == name) {
            return Err(format!("Scripts: {kind} {name} is declared twice"));
        }
        list.push((name, v));
    }
    w.resources.insert(Rc::new(host));
    w.resources.insert(templates::CtorRunner(run_ctor));
    Ok(())
}

/// [`templates::CtorRunner`]: a class's constructor closure on a new entity, with the constructor's arguments
/// fitted to the closure's parameters as for `on_spawn`.
fn run_ctor(w: &mut World, e: EntityId, ctor: u32, args: &[ScriptArg<'_>]) {
    let Some(host) = host(w) else { return };
    let Some((script, f)) = host.ctors.get(ctor as usize).cloned() else {
        return;
    };
    let mut v: Vec<Dynamic> = f.curry().to_vec();
    v.push(friendly::new_entity(e));
    v.extend(args.iter().map(|a| api::arg(*a)));
    if let Some(n) = host.arity(script, f.fn_name(), v.len()) {
        v.resize(n.max(f.curry().len() + 1), Dynamic::UNIT);
    }
    host.call(w, script, f.fn_name(), v);
}

fn report(w: &mut World, to: muo_server::ids::MobileId, what: &str, errors: &[String]) {
    let send = muo_server::commands::send_message;
    if errors.is_empty() {
        send(w, to, &format!("{what} reloaded."));
    } else {
        send(
            w,
            to,
            &format!("{what} reloaded with {} error(s):", errors.len()),
        );
        for e in errors {
            send(w, to, e);
        }
    }
}

/// The Configure phase (after the templates): compile, bind, subscribe the global hooks, register the commands.
fn configure(w: &mut World) {
    api_store::configure(w);
    for e in install(w) {
        tracing::error!("Scripts: {e}");
    }
    // `@event login`: once the player's client is in the world (the end of DoLogin), so a window sent now shows
    crate::network::events::subscribe::<crate::network::events::PlayerLogin>(w, |w, e| {
        let m = e.0;
        call_global(w, 0, || vec![api::mobile(m)]);
    });
    w.events.speech.subscribe(|w, a| {
        let (m, text) = (a.mobile, &a.speech);
        if call_global(w, 1, || vec![api::mobile(m), text.clone().into()]) {
            a.handled = true;
        }
    });
    w.events.server_started.subscribe(|w, _| {
        call_global(w, 2, Vec::new);
    });
    CommandSystem::register(
        w,
        "ReloadScripts",
        AccessLevel::Administrator,
        &[],
        Rc::new(|w, e| {
            let errors = reload_scripts(w);
            report(w, e.mobile, "Scripts", &errors);
        }),
    );
    CommandSystem::register(
        w,
        "ReloadTemplates",
        AccessLevel::Administrator,
        &[],
        Rc::new(|w, e| {
            // one load: the classes declared in scripts are templates too
            let errors = reload_scripts(w);
            report(w, e.mobile, "Templates", &errors);
        }),
    );
}

/// Registers the scripting Configure phase; call last, after every content crate.
pub fn register(types: &mut TypeRegistry) {
    types.register_phase(
        Phase::Configure,
        "muo_content.Scripts.Configure",
        1000,
        configure,
    );
}

/// Gives `to` the class templates and script host `from` loaded (shared, they are read-only once loaded), so a world
/// built from the same registry need not compile the scripts again. Test helpers use it to load the classes once
/// per thread instead of once per test world.
pub fn share_loaded(from: &World, to: &mut World) {
    if let Some(t) = from.resources.get::<Rc<TemplateTable>>() {
        to.resources.insert(t.clone());
    }
    if let Some(h) = from.resources.get::<Rc<ScriptHost>>() {
        to.resources.insert(h.clone());
    }
    if let Some(r) = from.resources.get::<templates::CtorRunner>() {
        to.resources.insert(*r);
    }
}
