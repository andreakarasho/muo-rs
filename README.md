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
   - `Data/`: copy the [`Data/`](Data) folder of this repository. It holds the game content: eras and maps
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
