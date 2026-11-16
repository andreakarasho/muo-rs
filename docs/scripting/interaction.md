# Targeting, prompts and dialogs

Each takes the name of a script function to call with the player's answer. It is looked up when the answer comes,
in the calling script first, else in any script, so a `[ReloadScripts` in between applies.

| | Calls |
|---|---|
| `target(from, "callback")` / `target(from, "callback", range)` | `callback(from, picked)`: a mobile, an item, or `#{ x, y, z, item_id }` for land and statics |
| `cancel_target(from)` | closes an open cursor |
| `prompt(from, text, "callback")` | `callback(from, answer)`; `""` when cancelled |
| `dialog(to, title, text, ["Yes", "No"], "callback")` | `callback(to, button)`: 1 for the first button, 0 when closed |

Dialog text may hold simple HTML (`<br>`, `<b>`). A dialog has at most 10 buttons.

```rhai
/// @command Smite
fn smite_command(from, args) {
    target(from, "smite");
}

fn smite(from, what) {
    if type_of(what) == "Mobile" { what.lightning(); }
}

/// @command Quest
/// @access Player
fn quest(from, args) {
    dialog(from, "The Lost Ring", "Will you find my ring?", ["Yes", "No"], "quest_answer");
}

fn quest_answer(from, button) {
    if button == 1 { from.say_to(from, "Thank you, traveller!"); }
}
```

## Custom windows

`gump(to, elements, "callback")` draws a window element by element; answering calls `callback(to, button,
switches)`: the button's id (0 when closed) and the ids of the ticked checkboxes. Each element is a map with a
`kind`:

| kind | keys |
|---|---|
| `background` | x, y, w, h, id (9270) |
| `alpha` | x, y, w, h (see-through) |
| `label` | x, y, text, hue |
| `html` | x, y, w, h, text, background, scroll |
| `button` | x, y, id, normal (4005), pressed (4007) |
| `page` | page: what follows is on that page (0: every page) |
| `page_button` | x, y, page: switches pages without answering |
| `check` | x, y, id, checked |
| `item` | x, y, item_id, hue (an item's picture) |
| `image` | x, y, id, hue |
| `no_close` | the player cannot close it (right-click does nothing) |

```rhai
fn shop(from) {
    gump(from, [
        #{ kind: "background", w: 300, h: 120 },
        #{ kind: "label", x: 20, y: 20, text: "A torch: 20 gold" },
        #{ kind: "button", x: 20, y: 60, id: 1 },
    ], "shop_answer");
}

fn shop_answer(from, button, switches) {
    if button == 1 && from.consume("Gold", 20) { pack_item(from, "Torch", 1); }
}
```

A class's `c.on_use(|npc, from| ..)` also works on creatures: double-clicking the creature runs it (return `true` to
skip the default).
