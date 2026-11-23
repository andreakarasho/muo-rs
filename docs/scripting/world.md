# The world

Maps are numbers: `0` Felucca, `1` Trammel, `2` Ilshenar, `3` Malas, `4` Tokuno, `5` Ter Mur (`map_id("Malas")`
finds one by name). A class is a short (`"Torch"`) or full (`"Server.Items.Torch"`) name.

More maps come from `Data/map-definitions.json`. An entry with a `"count"` makes that many maps on the indexes from
its own, each named `<name><index>`: private instances of one place. Whoever is on one sees only what is on that
one, while `id` (the map the client draws) can be the same for all of them. With `"fileIndex": 127` such a map
reads no map files (flat, empty land), so it costs nothing until used. The raids' Hideouts are made this way; their
practice raids use copies of Felucca (`"fileIndex": 0`, opened when first used), which have its land and statics
but not its regions or spawners.

| Function | Gives | |
|---|---|---|
| `find_mobile(serial)` / `find_item(serial)` | handle or `()` | the entity with that serial |
| `m.serial` / `i.serial` | int | |
| `a == b`, `a != b` | bool | two handles are the same entity |
| `players()` | array of mobiles | everyone connected |
| `online_count()` | int | |
| `mobiles_in_range(x, y, map, range)` | array of mobiles | |
| `items_in_range(x, y, map, range)` | array of items | on the ground |
| `spawn_mobile(class, x, y, z, map)` | mobile or `()` | a new mobile, placed |
| `spawn_item(class, x, y, z, map)` | item or `()` | a new item on the ground |
| `spawn_house(owner, multi_id, x, y, z, map)` | item or `()` | a customizable house plot owned by `owner`, placed without placement checks (`0x147B` is 18x18) |
| `create_item(class)` / `create_mobile(class)` | handle or `()` | made but not placed yet |
| `broadcast(text)` / `broadcast(text, hue)` | | a message to every player |
| `broadcast_staff(text)` | | to game masters and above |
| `region_name(x, y, map)` | string | `""` outside any named region |
| `m.region` | string | the region a mobile stands in |
| `ground_z(x, y, map)` | int | the land height there |
| `map_id(name)` / `map_name(map)` | int / string | `-1` / `""` when unknown |
| `server_time()` | string | the server's date and time |
| `ticks()` | int | milliseconds since start, for measuring |

```rhai
let chest = spawn_item("WoodenChest", 1500, 1600, ground_z(1500, 1600, 0), 0);
broadcast("A treasure chest appeared near Britain!", 0x35);
```
