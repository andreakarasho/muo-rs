# Scripting reference

Content is [rhai](https://rhai.rs) in `Data/Scripts/**/*.rhai`. `[ReloadScripts` reloads it in game; a script error
stops startup (or refuses the reload, keeping what was loaded) with the file and line.

- [Classes, constructors and hooks](#classes) (below)
- [The world](world.md): lookups, players, spawning, broadcast, regions, time
- [Creatures and players](mobile_more.md): stats, flags, skills, combat, speech, inventory
- [Items](item_more.md): place, flags, containers, messages, magic properties
- [Sounds and effects](effects.md)
- [Targeting, prompts and dialogs](interaction.md)

## Classes

```rhai
mobile("Orc", |c| { .. });      // an existing class, by short or full name
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
  `c.on_outfit(|m| ..)` for an NPC's clothes.

## Script files

- `fn command_<name>(from, args)` adds the staff command `[<name>`; a `// access: <level>` line right above sets
  who may use it (default `GameMaster`).
- `fn on_login(m)`, `fn on_speech(m, text)` (true marks it handled), `fn on_server_started()`: in any script.
- `timer(ms, "function", arg)`: calls `function(arg)` later.
- Random draws: `random(a, b)`, `random_below(n)`, `random_bool()`, `random_double()`, `chance(p)`, `coin()`,
  `pick([..])`; `era("AOS")` is true from that expansion on.
- `log(text)` writes to the server log.
