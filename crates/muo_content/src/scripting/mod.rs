//! Rhai scripts: `Data/Scripts/**/*.rhai`, recompiled in game with `[reloadscripts`.
//!
//! - Per-class hooks: a template's `"script": "<path under Data/Scripts>"` binds the script to that class and every
//!   class below it (the class chain, then each template's `inherits`). The script may define `on_use(item, from)`,
//!   `on_death(m, corpse)`, `on_think(m)`, `on_damage(m, amount, from)`, `on_hear(m, from, text)` and
//!   `on_spawn(e)`; returning `true` skips the default behavior (see `muo_server::script_hooks`).
//! - Global hooks, in every script: `on_login(m)`, `on_speech(m, text)` (`true` marks the speech handled) and
//!   `on_server_started()`.
//! - Commands: `fn command_<name>(m, args)` registers `[<name>`; a `// access: <AccessLevel>` comment line right
//!   above it sets the level (default `GameMaster`).
//!
//! - Classes: `class(name, #{..}, |e| {..})` at a script's top level declares a class template (see `classes`).
//!
//! A script's top level runs once, when the templates load, to collect its `class` declarations; after that only
//! functions run. Functions resolve by name at call time, so a reload applies immediately. A script error (including a runaway loop hitting the operation limit) is
//! logged with the file and function, and the default behavior runs.

mod api;
mod api_engines;
mod api_equipment;
mod api_misc_items;
mod api_misc_items2;
mod api_mobiles;
mod api_npcs;
mod classes;
pub mod friendly;
#[cfg(test)]
mod tests;

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
use rhai::{AST, CallFnOptions, Dynamic, Engine, FnPtr, Scope};

/// One compiled file.
pub struct Script {
    /// The path under `Data/Scripts`, `/`-separated (what a template's `"script"` names).
    pub name: String,
    pub ast: AST,
    /// `(command, access)` of each `command_<name>` function.
    commands: Vec<(String, AccessLevel)>,
}

/// The global hooks: function name and parameter count.
const GLOBALS: [(&str, usize); 3] = [("on_login", 1), ("on_speech", 2), ("on_server_started", 0)];

/// Every compiled script, with the class bindings. Lives in `w.resources` as `Rc<ScriptHost>`: clone the `Rc`
/// before a call so the world is free during it.
pub struct ScriptHost {
    engine: Engine,
    pub scripts: Vec<Script>,
    /// The constructor closures of the `class` declarations: (script, closure); `Template::ctor` indexes this.
    ctors: Vec<(usize, FnPtr)>,
    /// The hook closures of the declarations: (class, hook name, script, closure).
    class_hooks: Vec<(String, String, usize, FnPtr)>,
    /// The scripts defining each of [`GLOBALS`].
    globals: [Vec<usize>; 3],
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
        let s = &self.scripts[script];
        let depth = &mut w.resources.get_mut::<Depth>().0;
        if *depth >= MAX_DEPTH {
            tracing::error!(
                "Script {}: {name}: nested deeper than {MAX_DEPTH} script calls",
                s.name
            );
            return None;
        }
        *depth += 1;
        // The pointer is the only way the world is reached during the call; `w` is not touched again until
        // `call_fn_with_options` returns (see `api::with_world` for the SAFETY argument).
        let ctx = api::Ctx {
            world: std::ptr::from_mut(w),
            script: s.name.as_str().into(),
        };
        let opts = CallFnOptions::new().eval_ast(false).with_tag(ctx);
        let r = self.engine.call_fn_with_options::<Dynamic>(
            opts,
            &mut Scope::new(),
            &s.ast,
            name,
            args,
        );
        w.resources.get_mut::<Depth>().0 -= 1;
        match r {
            Ok(v) => Some(v),
            Err(e) => {
                tracing::error!("Script {}: {name}: {e}", s.name);
                None
            }
        }
    }

    /// Compiles every `.rhai` under `dir` and runs each top level once, collecting its `class` declarations. A file
    /// that fails is left out and named in the errors.
    pub(crate) fn load(dir: &Path, types: &TypeRegistry) -> Loaded {
        let mut engine = api::engine();
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
                    let commands = parse_commands(&name, &text, &mut errors);
                    scripts.push(Script {
                        name,
                        ast,
                        commands,
                    });
                }
                Err(e) => errors.push(format!("{name}: {e}")),
            }
        }
        for (i, sc) in scripts.iter().enumerate() {
            collector.borrow_mut().current = i;
            if let Err(e) = engine.run_ast(&sc.ast) {
                errors.push(format!("{}: {e}", sc.name));
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
        for (g, (name, arity)) in GLOBALS.iter().enumerate() {
            host.globals[g] = (0..host.scripts.len())
                .filter(|&i| host.has_fn(i, name, *arity))
                .collect();
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

/// `fn command_<name>(` lines, with the `// access: <level>` of the comment block right above.
fn parse_commands(file: &str, text: &str, errors: &mut Vec<String>) -> Vec<(String, AccessLevel)> {
    let lines: Vec<&str> = text.lines().map(str::trim).collect();
    let mut out = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        let Some(rest) = line.strip_prefix("fn command_") else {
            continue;
        };
        let name: String = rest
            .chars()
            .take_while(|c| c.is_alphanumeric() || *c == '_')
            .collect();
        if name.is_empty() {
            continue;
        }
        let comment = lines[..i].iter().rev().take_while(|l| l.starts_with("//"));
        let access = comment
            .filter_map(|l| l.trim_start_matches('/').trim().strip_prefix("access:"))
            .map(str::trim)
            .next();
        let level = match access {
            None => AccessLevel::GameMaster,
            Some(a) => match (0..=6)
                .filter_map(AccessLevel::from_i32)
                .find(|l| format!("{l:?}") == a)
            {
                Some(l) => l,
                None => {
                    errors.push(format!("{file}: command_{name}: unknown access level {a}"));
                    continue;
                }
            },
        };
        out.push((name, level));
    }
    out
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
    let commands: Vec<(String, AccessLevel)> = host
        .scripts
        .iter()
        .flat_map(|s| s.commands.clone())
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

fn register_commands(w: &mut World, commands: Vec<(String, AccessLevel)>) {
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
    for (name, access) in commands {
        let fn_name = format!("command_{name}");
        let h: CommandEventHandler =
            Rc::new(move |w: &mut World, e: &mut CommandEventArgs| run_command(w, &fn_name, e));
        CommandSystem::register(w, &name, access, &[], h.clone());
        ours.push((name, h));
    }
    w.resources.get_mut::<ScriptCommands>().0 = ours;
}

fn run_command(w: &mut World, fn_name: &str, e: &mut CommandEventArgs) {
    let Some(host) = host(w) else { return };
    let Some(i) = (0..host.scripts.len()).find(|&i| host.has_fn(i, fn_name, 2)) else {
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

/// Runs global hook `GLOBALS[g]` in every script defining it; true when any returned `true`.
fn call_global(w: &mut World, g: usize, args: impl Fn() -> Vec<Dynamic>) -> bool {
    let Some(host) = host(w) else { return false };
    let mut handled = false;
    for &i in &host.globals[g] {
        let r = host.call(w, i, GLOBALS[g].0, args());
        handled |= r.is_some_and(|r| r.as_bool().unwrap_or(false));
    }
    handled
}

/// A `timer(ms, "fn", arg)` firing: `fn` in the script that started it, else in any script (resolved now, so a
/// reload in between applies).
fn fire_timer(w: &mut World, script: &str, name: &str, arg: Dynamic) {
    let Some(host) = host(w) else { return };
    let n = host.scripts.len();
    let found = (0..n)
        .find(|&i| host.scripts[i].name == script && host.has_fn(i, name, 1))
        .or_else(|| (0..n).find(|&i| host.has_fn(i, name, 1)));
    match found {
        Some(i) => {
            host.call(w, i, name, vec![arg]);
        }
        None => tracing::warn!("Script {script}: timer: no function {name}(arg)"),
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
    for e in install(w) {
        tracing::error!("Scripts: {e}");
    }
    w.events.connected.subscribe(|w, m| {
        let m = *m;
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
