# Examples

Each example is a complete, working script. The healing stone is in `Data/Scripts/examples/` and the vendor in
`Data/Scripts/vendors/` (edit those files); the rune and the login counter are new and can be pasted into a new file
under `Data/Scripts` and tried after `[ReloadScripts`.

## An item with a hook: the healing stone

The class (in `classes/items/examples.rhai`, shown in [Getting started](getting-started.md#a-first-item)) binds
`examples/healing_stone.rhai`. That file's `on_use` runs when someone double-clicks the stone:

```rhai
fn on_use(stone, from) {
    if stone.hue == 0x3B2 {
        send_message(from, "The stone is still recharging.");
        return true;
    }
    from.hits = from.hits + random(10, 20);
    send_message(from, "The stone glows and your wounds close.");
    stone.hue = 0x3B2; // grey while it recharges
    timer(10000, "recharge", stone);
    true // handled: the item's own double-click does not run
}

// Called by the timer. If the stone was deleted meanwhile, setting its hue does nothing.
fn recharge(stone) {
    stone.hue = 1153;
}
```

- Returning `true` from a hook replaces the class's default behaviour; `false` runs it afterwards.
- `timer(ms, "function", arg)` calls a function of the same file later.
- The stone's hue doubles as its state; for state that must survive a restart, see `store_set` in the
  [Introduction](README.md#script-files).

## An inline hook: a rune you cannot step on

Small hooks can be written right in the class declaration instead of a separate file. `on_move_over` runs when a
mobile steps onto the item; `true` refuses the step:

```rhai
item("WardingRune", |c| {
    c.extends = "Item";
    c.on_create(|i| {
        i.item_id = 0x1F14;
        i.hue = 0x47E;
        i.name = "a warding rune";
        i.movable = false;
    });
    c.on_move_over(|rune, m| {
        send_message(m, "A force pushes you back.");
        true
    });
});
```

`[add WardingRune`, then try to walk over it. The same form works for `on_use`, `on_death`, `on_damage`,
`on_hear` and `on_think` (see [the hook list](README.md#classes)).

## A vendor

A `shop` declares what a vendor type sells and buys back. `sells(class, price, amount, item_id, ..)` lists a stock
line; `buys(class, price)` is what the vendor pays a player:

```rhai
// Data/Scripts/vendors/SBBaker.rhai (shortened)
shop("SBBaker", |s| {
    s.buys("BreadLoaf", 3);
    s.buys("Cake", 5);
    s.sells("BreadLoaf", 6, 20, 0x103B, #{ hue: 0 });
    s.sells("ApplePie", 7, 20, 0x1041, #{ hue: 0 });
    s.sells("Cake", 13, 20, 0x09E9, #{ hue: 0 });
});
```

Edit a price in `vendors/SBBaker.rhai` and `[ReloadScripts`: bakers made from then on use the new list. A vendor
builds its lists when it is made or loaded, so bakers already in the world keep the old prices until the next
restart.

## Commands and events

A command and an event handler in one file. The `login` event greets players and counts their logins in the store,
which is saved with the world:

```rhai
/// @event login
fn welcome(m) {
    let key = `logins/${m.serial}`;
    let n = store_get(key);
    let n = if type_of(n) == "()" { 1 } else { n + 1 };
    store_set(key, n);
    send_message(m, `Welcome, ${m.name}. This is visit number ${n}.`);
}

/// @command Logins
/// @access Player
fn logins(m, args) {
    let n = store_get(`logins/${m.serial}`);
    send_message(m, if type_of(n) == "()" { "No visits recorded." } else { `You have logged in ${n} times.` });
}
```

Other events are `speech`, `server_started`, `skill_gain` and `quest_log`; their parameters are listed in the
[Introduction](README.md#script-files).
