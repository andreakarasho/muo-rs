# Sounds and effects

Seen and heard by the players near them. An effect's graphic is an item id (`0x3709` a fire column, `0x36BD` an
explosion, `0x376A` sparkles, `0x36E4` a fireball); `speed` and `duration` are in the client's units (10 and 30 are
common). Each works on a mobile or an item.

| | |
|---|---|
| `x.play_sound(id)` | a sound where it is |
| `play_sound_at(x, y, z, map, id)` | |
| `x.lightning()` | a lightning bolt with its thunder |
| `x.effect(graphic, speed, duration)` / `(.., hue)` | an animation following it |
| `x.particles(graphic, speed, duration, effect)` | the particle form |
| `effect_at(x, y, z, map, graphic, duration)` / `(.., hue)` | an animation on the ground |
| `moving_effect(from, to, graphic, speed)` / `(.., hue)` | a missile flying between them |
| `missile(from, to, graphic, speed)` | the same, exploding on arrival |
| `m.flash(kind)` | a screen effect for that player: FadeOut, FadeIn, LightFlash, FadeInOut, DarkFlash |

```rhai
fn smite(from, what) {
    if type_of(what) == "Mobile" {
        what.lightning();
        damage(what, 25, from);
    }
}
```
