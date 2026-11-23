item("Aquarium", |c| {
    c.weight = 10.0;
});

item("AquariumEastDeed", |c| {
    c.label = 1074501;
});

item("AquariumNorthDeed", |c| {
    c.label = 1074497;
});

item("FishBowl", |c| {
    c.label = 1074499;
    c.weight = 2.0;
    c.on_create(|i| {
        i.item_id = 0x241C;
        i.hue = 0x47E;
    });
});
