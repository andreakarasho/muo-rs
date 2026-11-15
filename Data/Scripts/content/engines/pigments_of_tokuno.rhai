// Server.Items.PigmentsOfTokuno (Data/Templates/Items/engines/misc.json): `PigmentsOfTokuno(type, uses)`.
// Without `uses` a pigment of no type or of a type from FadedCoal (10) on has 10 uses, the others 50.

fn on_spawn(item) {
    on_spawn(item, 0);
}

fn on_spawn(item, kind) {
    let uses = if kind == 0 || kind >= 10 { 10 } else { 50 };
    on_spawn(item, kind, uses);
}

fn on_spawn(item, kind, uses) {
    set_prop(item, "UsesRemaining", uses);
    set_prop(item, "Type", kind);
}
