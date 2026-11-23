# Getting started

Server content (items, creatures, vendors, commands, events) is written in [rhai](https://rhai.rs), a small
scripting language with a Rust/JavaScript-like syntax. You do not need Rust or a compiler: edit a `.rhai` file, type
`[ReloadScripts` in game (Administrator access), and the change is live. Hooks, commands and events apply at once,
also to what already exists; a class's `on_create` and values only shape things made after the reload.

## Where scripts live

Every `.rhai` file under `Data/Scripts` is loaded at startup, in any folder. The shipped content is organised like
this:

| Folder | What is in it |
|---|---|
| `classes/items`, `classes/mobiles` | Class declarations: `item("..", \|c\| ..)`, `mobile("..", \|c\| ..)` |
| `content/` | Scripts bound to classes (`c.script = ".."`) for behaviour too big to write inline |
| `vendors/` | What each vendor type buys and sells: `shop("SBBaker", \|s\| ..)` |
| `examples/` | Small, commented examples (a command, a healing stone, a shouting rat) |

You can add your own folders; nothing depends on the layout.

## Reloading and errors

`[ReloadScripts` recompiles every file. If a file has an error, the reload is refused and the scripts already loaded
stay in place; you get the file and line. At startup, a script error stops the server with the same message, so a
broken file never runs half-loaded.

A script error while the server runs (in a hook or a command) is logged with the file and function, and the default
behaviour runs instead.

## A first item

Declare a new class by giving it a name and the class it `extends`. `on_create` runs for every new one and sets its
look:

```rhai
// Data/Scripts/classes/items/examples.rhai
item("Examples.HealingStone", |c| {
    c.extends = "Item";
    c.script = "examples/healing_stone.rhai";
    c.on_create(|i| {
        i.item_id = 0x1363;
        i.hue = 0x481;
        i.name = "a healing stone";
        i.movable = false;
    });
});
```

`[add Examples.HealingStone` makes one. `c.script` points at a file whose functions are the class's hooks; here,
`on_use` heals whoever double-clicks it (see [Examples](examples.md)).

## A first creature

`mobile("Name", ..)` with a new name and `extends` makes a new class. A class can be declared only once: the shipped
ones are already in `Data/Scripts/classes/` (`Rat` is in `classes/mobiles/animals.rhai`), so to change one, edit its
declaration there; declaring it again in another file is a load error.

```rhai
// Data/Scripts/classes/mobiles/examples.rhai
mobile("Examples.ShoutingRat", |c| {
    c.extends = "Rat";
    c.script = "examples/shouting_rat.rhai";
    c.on_create(|m| {
        m.name = "a shouting rat";
    });
});
```

Values can be ranges, rolled for each new creature: `m.str = 96..=120`, `m.skill("Swords", 50.1, 75.0)`. The
[Introduction](README.md#classes) lists everything a class declaration can set.

## A first command

A `///` header above a function makes it a command. `@access` sets who may use it:

```rhai
// Data/Scripts/examples/hello.rhai
/// @command Hello
/// @access GameMaster
fn hello(m, args) {
    if args == "" {
        say(m, "Hello!");
    } else {
        say(m, `Hello, ${args}!`);
    }
    log(`${m.name} said hello`);
}
```

`[Hello world` makes your character say "Hello, world!". `args` is the text after the command name.

## Where to go next

- [Examples](examples.md): an item with a hook, a vendor, an event handler.
- The reference pages, from [The world](world.md) onward: every function a script can call.
