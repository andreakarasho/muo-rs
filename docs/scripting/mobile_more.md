# Creatures and players

These work on a live handle: `m` in a hook or command, `m.entity` inside an `on_create`. A deleted mobile reads as
defaults and actions on it do nothing. Skills, layers, poisons and directions are names (`"Swords"`, `"OneHanded"`,
`"Deadly"`, `"North"`).

## Stats and flags

Read and write: `name`, `hits`, `str`, `dex`, `int`, `mana`, `stam`, `fame`, `karma`, `kills`, `title`, `body`,
`hue`, `female`, `hidden`, `blessed`, `frozen`, `paralyzed`, `criminal`, `warmode`, `direction`.
Read only: `hits_max`, `mana_max`, `stam_max`, `alive`, `murderer`, `poisoned`, `access_level`, `x`, `y`, `z`, `map`,
`region`, `is_player`, `online`, `account`, `serial`, `class_name`, `deleted`.

Anything `[props` shows: `m.get_prop("Team")`, `m.set_prop("FightMode", "Closest")`.

## Skills

| | |
|---|---|
| `m.skill_value("Swords")` | with modifiers |
| `m.skill_base("Swords")` | the trained value |
| `m.set_skill("Swords", 80.0)` | sets the trained value |

## Health and combat

| | |
|---|---|
| `m.heal(amount)` | |
| `damage(m, amount, from)` / `kill(m)` / `m.resurrect()` | |
| `m.poison("Deadly")` / `m.poison("Lesser", from)` | true when it took (Lesser, Regular, Greater, Deadly, Lethal) |
| `m.cure()` | true when it cured |
| `m.clear_mods()` | ends what spells, potions and songs put on it for a time (stat, skill and resistance modifiers); what worn items give stays |
| `m.paralyze(seconds)` / `m.freeze(seconds)` | |
| `m.attack(target)` | start fighting |
| `m.combatant` | who it fights, or `()`; settable |

## Speech and animation

`say(m, text)`, `m.emote(text)`, `m.yell(text)`, `m.whisper(text)`, `m.say_to(player, text)` (only that player sees
it), `send_message(m, text)` / `send_message(m, text, hue)` (a system message to a player), `m.animate(action,
frames)`.

## Place

`move_to(m, x, y, z)` (same map), `m.move_to(x, y, z, map)`.

## Inventory

| | |
|---|---|
| `m.backpack` | item or `()` |
| `m.equipped("OneHanded")` | the item on that layer, or `()` |
| `m.worn` | every worn item |
| `m.equip(item)` | true when put on |
| `m.give(item)` | into the backpack (at the feet without one) |
| `pack_item(m, class, amount)` | a new item into the backpack |
| `m.count("Gold")` | how many the backpack holds, containers included |
| `m.consume("Bandage", 2)` | takes them; false (taking nothing) without enough |

```rhai
/// @command Reward
/// @access Player
fn reward(from, args) {
    if from.consume("Gold", 1000) {
        from.give(create_item("Bandage"));
        from.say_to(from, "Here, take this.");
    }
}
```
