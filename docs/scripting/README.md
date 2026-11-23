# muo-rs scripting

New here? Start with [Getting started](getting-started.md) and the [Examples](examples.md). This page is the
reference for class declarations and script files; the pages after it cover every function a script can call.

Content is [rhai](https://rhai.rs) in `Data/Scripts/**/*.rhai`. `[ReloadScripts` reloads it in game; a script error
stops startup (or refuses the reload, keeping what was loaded) with the file and line.

- [Classes, constructors and hooks](#classes) (below)
- [The world](world.md): lookups, players, spawning, broadcast, regions, time
- [Creatures and players](mobile_more.md): stats, flags, skills, combat, speech, inventory
- [Items](item_more.md): place, flags, containers, messages, magic properties
- [Sounds and effects](effects.md)
- [Targeting, prompts, dialogs and custom windows](interaction.md)

## Classes

```rhai
mobile("Orc", |c| { .. });      // a shipped class (declared once, in classes/mobiles/monsters.rhai: edit it there)
item("FlamingLongsword", |c| { c.extends = "Longsword"; .. });   // a new class
shop("SBBlacksmith", |s| { s.sells("Tongs", 13, 14, 0x0FBB); s.buys("Tongs", 7); });
```

On the class handle `c`, once at load:

- `c.<name> = value`: a class value (the C# class's constants): `name`, `weight`, `label`, `rarity`, `loot`,
  `corpse_name`, ..., or `c.prop("exactName", value)`.
- `c.extends`, `c.ai` (`"Melee"`, `coin("Melee", "Mage")`), `c.fight_mode`, `c.range_perception`, `c.range_fight`.
- `c.on_create(|e, args..| ..)`: the constructor; `e.<statement> = value` (`m.str = 96..=120`, `i.item_id = 0x1F19`,
  `m.resist("Fire", 20, 30)`, `m.skill("Swords", 50.1, 75.0)`, any statement as `e.step("name", value)`);
  `e.entity` is the live handle for everything in these pages.
- Hooks, each returning `true` to replace the default: `c.on_use(|i, from| ..)`, `c.on_death(|m, corpse| ..)`,
  `c.on_think(|m| ..)`, `c.on_damage(|m, amount, from| ..)`, `c.on_hear(|m, from, text| ..)`, and
  `c.on_outfit(|m| ..)` for an NPC's clothes. On an item, `c.on_move_over(|i, m| ..)` runs when a mobile steps onto
  it (a gate that opens a menu, a trap); `true` refuses the step.

## Script files

- A `///` header on a function makes it a command or an event handler, one tag per line:

  ```rhai
  /// @command Battle
  /// @access GameMaster
  fn battle(from, args) { .. }     // [Battle; `args` is the text after it

  /// @event login
  fn welcome(m) { .. }
  ```

  `@access` is Player, Counselor, GameMaster (the default), Seer, Administrator, Developer or Owner. Events:
  `login` (`fn x(m)`), `speech` (`fn x(m, text)`, return true to mark it handled), `server_started` (`fn x()`),
  `skill_gain` (`fn x(m, skill)`, asked before a player gains in a skill such as `"Swords"`; return true to refuse
  the gain, and the stat gain that comes with it), `quest_log` (`fn x(m)`, the paperdoll's Quests button; return
  true when you showed your own window, else the engine's quest log opens).
  An unknown tag, level or event, a wrong parameter count, or a command defined twice stops the load with the file
  and function, and so does a function still named the retired way (`command_<name>`, `on_login`, `on_speech`,
  `on_server_started`) without a header.
- Each file is on its own. To use another file's functions, import it as a rhai module, by its path under
  `Data/Scripts` without `.rhai`: `import "raids/places" as places;` then `places::dungeons()`. A module's top level
  runs on every import, so keep imported files to functions; class declarations, commands and events go in files
  nothing imports. Imports cannot go in a circle.
- `timer(ms, "function", arg)`: calls `function(arg)` later.
- Random draws: `random(a, b)`, `random_below(n)`, `random_bool()`, `random_double()`, `chance(p)`, `coin()`,
  `pick([..])`; `era("AOS")` is true from that expansion on.
- `log(text)` writes to the server log.
- Values that survive restarts (saved with the world): `store_set(key, value)`, `store_get(key)` (`()` when unset),
  `store_remove(key)`, `store_keys(prefix)`. A value is a number, text, bool, or an array or map of those; keep an
  entity's `serial`, not the handle.
