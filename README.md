# muo

An Ultima Online server emulator written in Rust. It speaks the original UO client protocol, plays from login to
gameplay, and persists the world in a binary save format.

## Specification

**Protocol**
- Talks to the Classic Client across the client eras: login and game-server handoff, client encryption detection,
  Huffman-compressed game packets, packet throttles, firewall and IP bans.
- Default listener `0.0.0.0:2593`; any number of `ip:port` listeners can be configured.

**World**
- Single-threaded game loop over one owned `World`. Every item and mobile is reached by id (`ItemId`, `MobileId`).
  There are no shared references, so a deleted entity simply stops resolving.
- Seven maps (Felucca, Trammel, Ilshenar, Malas, Tokuno, Ter Mur and an internal map), driven by
  `Data/map-definitions.json`, with sectors, regions, teleporters, decoration and spawners.
- Expansion-aware rules (T2A through current eras) chosen at first start and stored in the configuration.
- Content: accounts, characters and professions, skills, magery, necromancy, chivalry, bushido, ninjitsu, spellweaving,
  mysticism and gargoyle spells, crafting, loot, creatures and AI, vendors, housing and multis, guilds, factions,
  quests, champion spawns, gumps, context menus, targeting, and the staff command set (`[add`, `[props`, `[set`,
  `[where`, `[decorate` ...).

**Saves**
- Binary, per-entity-type files under `Saves/` (`Items/`, `Mobiles/`, `Guilds/`, ...): an index file, a type table of
  serialized type names (`Server.Items.Longsword`), and versioned records. Older record versions are migrated on
  load.
- Saves run in the background from a snapshot, with automatic archiving of old saves.

**Engineering rules**
- Behavior is plain data plus function tables: one `Item` and one `Mobile` struct, and a `static ItemType` /
  `MobileType` per class whose fields are the overridable behaviors. No `Rc`/`RefCell`, no global mutable state.
- Malformed client input, save data or files never panic.
- Unfinished work is marked `// partial: <what is missing>`; deliberate simplifications `// shortcut: <limit>`.

## Workspace

Dependencies only point down the table.

| crate | holds |
|---|---|
| `muo` | the server binary |
| `muo_spells`, `muo_items`, `muo_mobiles`, `muo_multis`, `muo_engines`, `muo_gumps`, `muo_commands` | concrete game content |
| `muo_content` | shared content: accounts, player and creature logic, AI, skills, spell and item bases, regions |
| `muo_server` | `World`, items, mobiles, maps, regions, timers, persistence, entity packets, gumps, targeting |
| `muo_net` | sockets, encryption, compression, packet framing, throttles, bans, packet encoders |
| `muo_data` | client data files (tiledata, map/statics, UOP, multis, cliloc), data tables, JSON configuration |
| `muo_io` | byte cursors, save reader/writer, text encodings, compression |
| `muo_primitives` | serials, geometry, directions, flags, RNG, tick and date math |

Content crates never depend on each other. They make objects by type name and call into other systems through
hook tables that the owning crate fills in at registration.

## What you need to run it

1. **Rust 1.85 or newer** (edition 2024): <https://rustup.rs>.
2. **A Classic Ultima Online client install.** The server reads the map, statics, tiledata, multi and cliloc
   files from it (`.mul` or `.uop`). Use the client version your players will connect with.
3. **A server folder next to the executable** with:
   - `Data/`: copy the [`Data/`](Data) folder of this repository. It holds the game content: the classes and vendors
     (`Scripts/`), eras and maps
     (`expansions.json`, `map-definitions.json`), regions and teleporters, spawners (`Spawns/`), town decoration
     (`Decoration/`), skills, professions, bulk orders, NPC names and speeds, and the staff menus. The server needs
     at least `expansions.json`, `map-definitions.json` and `skills.json`; without the rest the world is empty.
   - `Configuration/`: created on first start (see below).
   - `Saves/`: created on the first save.

## Build and run

```sh
cargo build --release
```

Copy `target/release/muo` (`muo.exe` on Windows) into the server folder and start it there. The working directory is
always the executable's folder. On the first start the console asks for:

- the client data directories (where the UO client is installed),
- the listeners (`ip:port`, default `0.0.0.0:2593`),
- the server name,
- the expansion and the maps to enable.

The answers go to `Configuration/muo.json`; edit that file, or delete it to be asked again. When there are no
accounts, the console offers to create the owner account. After that, a new username at the login screen creates an
account (`accountHandler.enableAutoAccountCreation`, on by default).

Console commands while running: `help` (`?`), `save` (`s`), `shutdown` (`sh`), `restart` (`r`). Ctrl+C shuts down
cleanly.

## Customizing content without recompiling

Content is [rhai](https://rhai.rs) script in `Data/Scripts/**/*.rhai`: the classes (`classes/`), the vendors' buy
and sell lists (`vendors/`), and anything else you add. Run `[ReloadScripts` in game, or restart; a script error stops
startup (or refuses the reload and keeps what was loaded) with the file and line.

```rhai
mobile("Orc", |c| {
    c.corpse_name = "an orcish corpse";      // class values: set once, at load
    c.loot = "Meager";

    c.on_create(|m| {                        // every new orc
        m.name = random_name("orc");
        m.body = 17;
        m.str = 96..=120;                    // a random value in the range, as SetStr(96, 120)
        m.resist("Fire", 20, 30);
        m.skill("Swords", 50.1, 75.0);
        if chance(0.2) { m.pack = [#{ item: "Server.Items.BolaBall" }]; }
    });

    c.on_death(|m, corpse| {                 // behavior: true replaces the default
        m.say("Urg...");
    });
});

item("ArmorOfFortune", |c| {
    c.label = 1061098;
    c.rarity = 11;
    c.on_create(|i| {
        i.hue = 0x501;
        i.attributes.luck = 200;
        i.armor_attributes.mage_armor = 1;
    });
});

shop("SBBlacksmith", |s| {
    s.sells("Tongs", 13, 14, 0x0FBB);        // what players buy: class, price, amount, graphic
    s.buys("Tongs", 7);                      // what the vendor buys from players
    s.era("AOS", |s| { s.sells("Bascinet", 18, 20, 0x140C); });
});
```

- `mobile(name, |c| ..)` / `item(name, |c| ..)` declare a class, by short (`"Orc"`) or full name. `c.extends =
  "Longsword"` makes a new class deriving from a registered one, usable with `[add` and spawners.
- `c.<name> = value` sets a class value, read where the C# class had a constant (weapon damage, armor resistances,
  corpse name, loot packs, labels...): `weight`, `label`, `rarity`, `name`, `ai`, `fight_mode`, `loot`, or any other
  name in snake case (`corpse_name`). `c.on_create(|e| ..)` is the constructor; `c.on_use`, `on_death`, `on_think`,
  `on_damage`, `on_hear` are hooks; `c.on_outfit(|m| ..)` an NPC's clothes.
- In `on_create`, `e.<name> = value` runs that constructor statement (`m.hits = 58..=72` sets the hits maximum, as
  the C# constructor did); `e.entity` is the live mobile or item. Content crates add statements of their own (each
  crate's `template_keys*.rs`); any statement is also `e.step("exactName", value)`.
- Random draws match the C# ones, so a class keeps its behavior when edited elsewhere: `chance(p)`, `coin()`,
  `random(a, b)`, `random_below(n)`, `pick([..])`, `era("AOS")`.
- Edits apply to entities made afterwards; saved entities keep their saved values.
- The full function list is in [`docs/scripting`](docs/scripting/README.md): the world, creatures and players,
  items, sounds and effects, targeting, prompts and dialogs.
- Plain script files also work: per-file hooks (`fn on_use(item, from)` bound with `c.script = "file.rhai"`), global
  hooks (`on_login`, `on_speech`, `on_server_started`) and staff commands (`fn command_<name>(m, args)` adds
  `[<name>`). See `Data/Scripts/examples/`.

Behavior (double-click handlers, AI, combat, gumps) is Rust, and so are the few constructors that are real logic
(guildstones, treasure maps, teleporters, a handful of quest NPCs and engine pieces).

## Tests

```sh
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all
```

Tests use port 0, never a fixed port. A few tests read real data and skip when it is absent:

| variable | points at | used by |
|---|---|---|
| `MUO_DISTRIBUTION` | a server folder with `Data/` and `Configuration/` (the repository root works) | data-table, decoration, spawn, region and location tests |
| `MUO_GOLDEN_SAVE` | a real `Saves/` folder | load-and-resave tests (byte-for-byte per record) |
| `MUO_GOLDEN_WORK` | scratch folder for those tests (default `target/golden-saves`) | `golden_saves` |
| `MUO_SPAWNER_FIXTURES` | a folder of saved spawner records | spawner migration tests |

## License

GPL-3.0-or-later. See the copyright notice the server prints at startup.
