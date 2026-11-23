# The scripting engine

This page is for people working on the Rust side: how scripts are loaded, how a script function reaches the world,
and how a hook gets from a class declaration to the code that calls it. For writing scripts, start at
[Getting started](getting-started.md).

## The pieces

| Where | What |
|---|---|
| `crates/muo_content/src/scripting/mod.rs` | Loading, `///` headers, commands and events, binding scripts to classes, hook dispatch, `[ReloadScripts` |
| `crates/muo_content/src/scripting/api.rs` | `engine()`: builds the rhai `Engine`, registers the `Mobile`/`Item` types and every `api_*` module; `with_world` |
| `crates/muo_content/src/scripting/api_*.rs` | The functions scripts call, one module per area (world, mobiles, items, effects, interaction, store, ...) |
| `crates/muo_content/src/scripting/friendly.rs` | The class handle: `item(..)`/`mobile(..)`/`shop(..)`, `c.<value> = ..`, `c.on_create`, `c.on_<hook>` |
| `crates/muo_content/src/scripting/classes.rs` | The collector the declarations write into, turned into class templates |
| `crates/muo_server/src/script_hooks.rs` | The hook enum and the calls a server dispatch site makes (`script_hooks::item`, `script_hooks::mobile`) |

`muo_server` knows nothing about rhai. It only asks "does a script handle this?" through `script_hooks`, which
`muo_content` fills in.

## Loading

`ScriptHost::load` (in `mod.rs`) compiles every `.rhai` under `Data/Scripts`:

1. `api::engine()` makes the `Engine` and registers all API functions, and `api::register_steps` the constructor
   statements (`m.str = ..`); a module resolver rooted at `Data/Scripts` lets scripts `import "folder/file" as name;`.
2. `classes::register` and `friendly::register` add the declaration functions, writing into a shared `Collector`.
3. Each file is compiled, and `parse_headers` reads each function's `///` doc comments: `@command`, `@access`,
   `@event`. A bad tag, level, event or parameter count is a load error naming the file and function.
4. The file's top level runs once. That run is what collects `item(..)`, `mobile(..)` and `shop(..)` declarations;
   after it, only functions run.

The collected declarations become class templates. `install` then binds each class to its script (`c.script`, or
the first one up the class chain) and records, per class, which hooks the script defines as a bitmask in the
`ScriptHooks` resource. Inline hooks (`c.on_use(|..| ..)`) are kept as closures keyed by class and hook.

`[ReloadScripts` repeats all of this. Functions are looked up by name at call time, so a reload applies at once to
everything already in the world.

## Calling into the world

A native function gets at the world through `with_world`:

```rust
e.register_fn("clear_mods", |ctx: NativeCallContext, m: Mobile| {
    act(&ctx, m, clear_mods) // act = with_world + a check that the mobile still exists
});
```

`ScriptHost::call` puts a pointer to the `&mut World` in the call's context tag before running a script function;
`with_world` turns it back into a `&mut World` for the length of one closure. The safety argument is the comment on
`with_world` in `api.rs`: native functions run one at a time, and none returns a reference into the world. A function
called outside a server hook (no world) returns a script error.

Scripts hold `Mobile(MobileId)` and `Item(ItemId)` handles, never references. A handle can outlive its entity (a
timer firing after the item was deleted), so functions check it first with `live_m` / `live_i`; the `act` helpers do
this and do nothing for a deleted entity.

## How a hook runs

Take `on_move_over`, an item hook: a mobile steps onto an item, and a script can refuse the step.

1. **The declaration.** `c.on_move_over(|i, m| ..)` (inline) or `fn on_move_over(i, m)` in the class's script.
2. **The bitmask.** At install, a class whose script or declaration defines `on_move_over` gets
   `ScriptHook::MoveOver.bit()` set in `ScriptHooks.items`.
3. **The dispatch site.** The server's movement code asks before the class's own behaviour:

   ```rust
   // crates/muo_server/src/mobile/part1/combat.rs, on_move_over
   use crate::script_hooks::{ScriptArg, ScriptHook, item as script};
   if script(w, o, ScriptHook::MoveOver, &[ScriptArg::Mobile(Some(m))]) {
       return false; // a script's `on_move_over` refused the step
   }
   ```

   For a class with no such hook this costs one resource lookup and one map probe.
4. **The call.** `dispatch` (in `mod.rs`) finds the inline closure or the script function, builds the arguments
   (the entity first, then the `ScriptArg`s), and calls it. A `true` result means handled.

## Recipes

### Add a script function

1. Pick the `api_*.rs` module for its area and add `e.register_fn("name", |ctx: NativeCallContext, ..| ..)` in its
   `register`, using `with_world` to reach the world (`api_mobile_more.rs` and `api_item_more.rs` have an `act`
   helper that also skips deleted entities).
2. Add a test (in the matching `tests_*.rs`, or `tests.rs` when there is none): `script_world("name", r#"fn run() { .. }"#)`
   then `call(&mut w, "run", vec![])`.
3. Document it on the matching page in `docs/scripting/`.

### Add a hook

The `on_move_over` change (commit `07fd0bb`) is a worked example of steps 1 to 3 and 5:

1. `muo_server/src/script_hooks.rs`: a `ScriptHook` variant, its entry in `ALL`, and its `fn_name` (name and
   parameter count, the entity included). Each hook is one bit of a `u8` (`ScriptHook::bit`, the `ScriptHooks`
   maps), and seven are taken: an eighth fits, a ninth means widening those to `u16`.
2. The dispatch site in `muo_server`: call `script_hooks::item` / `script_hooks::mobile` before the default, and
   skip the default when it returns `true`.
3. `muo_content/src/scripting/friendly.rs`: the hook's name in the list of `c.on_*` registrations.
4. A test that declares a class with the hook and calls `script_hooks::item(..)` (see
   `friendly_items_set_attributes_and_hooks_run` in `scripting/tests.rs`).
5. The hook list in `docs/scripting/README.md`.

### Add an event

Events (`@event login`, ...) are the `GLOBALS` table in `mod.rs`: the retired function name (`""` for none), the
parameter count and the `@event` name. Adding one:

1. A row in `GLOBALS`, and grow both its length and the length of `ScriptHost::globals` (a fixed array indexed by
   the row; forgetting it is a panic at load, not a compile error).
2. Raise it: `login`, `speech` and `server_started` are subscribed to server events in `configure()`;
   `skill_gain` and `quest_log` are functions the server calls (`skill_gain_refused`, `quest_log_shown`). Follow
   whichever fits.
3. The event in the list in `docs/scripting/README.md`.
