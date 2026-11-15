# Items

These work on a live handle: `i` in a hook, `i.entity` inside an `on_create`. A deleted item reads as defaults and
actions on it do nothing. Layers and loot types are names (`"OneHanded"`, `"Blessed"`).

Read and write: `name`, `hue`, `item_id`, `amount`, `movable`, `visible`, `weight`, `layer`, `loot_type`.
Read only: `x`, `y`, `z`, `map` (an item in a container or worn gives its holder's place; `-1` when nowhere),
`on_ground`, `is_container`, `contents`, `parent`, `root`, `serial`, `class_name`, `deleted`.

Anything `[props` shows: `i.get_prop("Quality")`, `set_prop(i, "Quality", "Exceptional")`.

| | |
|---|---|
| `i.move_to(x, y, z)` / `i.move_to(x, y, z, map)` | onto the ground |
| `container.drop_in(item)` | false when it is no container |
| `add_item(container, class, amount)` | a new item inside |
| `i.consume(n)` | from the stack; deleted at 0 |
| `i.say(text)` / `i.say(text, hue)` | words over the item |
| `delete(i)` | |

Magic properties: `i.attributes.luck = 200`, `i.weapon_attributes.hit_lightning = 30`, `i.armor_attributes.mage_armor
= 1`, `i.element_attributes.fire = 50` (read them the same way), `skill_bonus(i, 0, "Swords", 10.0)`.

```rhai
fn on_use(chest, from) {
    for i in chest.contents { from.give(i); }
    chest.say("Empty!");
    true
}
```
