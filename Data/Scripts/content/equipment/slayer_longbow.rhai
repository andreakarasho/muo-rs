// SlayerLongbow: a random second slayer (1..27). The class replays no template of its own: its parent
// ElvenCompositeLongbow's replay runs this hook.
fn on_spawn(bow) {
    set_slayer2(bow, random(1, 27));
}
