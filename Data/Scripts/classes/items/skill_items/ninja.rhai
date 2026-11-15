item("Fukiya", |c| {
    c.weight = 1.0;
});

item("LeatherNinjaBelt", |c| {
    c.weight = 1.0;
});

item("SmokeBomb", |c| {
    c.weight = 1.0;
});

item("EggBomb", |c| {
    c.label = 1030249;
    c.on_create(|i| {
        i.item_id = 0x2808;
    });
});

item("FukiyaDarts", |c| {
    c.weight = 1.0;
});

item("Shuriken", |c| {
    c.weight = 1.0;
});
