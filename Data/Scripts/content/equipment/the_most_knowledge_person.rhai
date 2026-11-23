// TheMostKnowledgePerson: hit point increase 3..5.
fn on_spawn(robe) {
    set_attr(robe, "Attributes", "BonusHits", 3 + random(0, 2));
}
