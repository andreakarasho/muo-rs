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
