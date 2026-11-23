// DetectiveBoots: intelligence +2..+4 (the level shows in the name).
fn on_spawn(boots) {
    set_attr(boots, "Attributes", "BonusInt", 2 + random(0, 2));
}
