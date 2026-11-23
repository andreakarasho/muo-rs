item("Blocker", |c| {
    c.label = 503057;
    c.on_create(|i| {
        i.item_id = 0x21A4;
        i.movable = false;
    });
});

item("LOSBlocker", |c| {
    c.name = "no line of sight";
    c.on_create(|i| {
        i.item_id = 0x21A2;
        i.movable = false;
    });
});

item("PowerCrystal", |c| {
    c.weight = 1.0;
    c.name = "power crystal";
    c.on_create(|i| {
        i.item_id = 0x1F1C;
    });
});

item("Scales", |c| {
    c.weight = 4.0;
    c.on_create(|i| {
        i.item_id = 0x1852;
    });
});

item("WayPoint", |c| {
    c.name = "AI Way Point";
    c.on_create(|i| {
        i.item_id = 0x1F14;
    });
});

item("Guillotine", |c| {
    c.on_create(|i| {
        i.item_id = 0x1230;
        i.movable = false;
    });
});
