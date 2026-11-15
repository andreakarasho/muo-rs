item("DawnsMusicBox", |c| {
    c.weight = 1.0;
    c.label = 1075198;
    c.on_create(|i| {
        i.item_id = 0x2AF9;
    });
});

item("DawnsMusicGear", |c| {
    c.weight = 1.0;
    c.on_create(|i| {
        i.item_id = 0x1053;
    });
});

item("EnhancedBandage", |c| {
    c.weight = 0.1;
    c.label = 1152441;
    c.on_create(|i| {
        i.item_id = 0xE21;
        i.stackable = true;
    });
});

item("FountainOfLife", |c| {
    c.label = 1075197;
    c.gump_id = 1156;
    c.drop_sound = 66;
    c.max_items = 125;
});

item("FountainOfLifeDeed", |c| {
    c.label = 1075197;
});
