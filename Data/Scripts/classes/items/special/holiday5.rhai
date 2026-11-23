item("MistletoeDeed", |c| {
    c.weight = 1.0;
    c.label = 1070882;
    c.on_create(|i| {
        i.item_id = 0x14F0;
    });
});

item("Fireflies", |c| {
    c.label = 1150061;
});

item("FirefliesDeed", |c| {
    c.weight = 1.0;
    c.label = 1150061;
    c.on_create(|i| {
        i.item_id = 0x14F0;
        i.loot_type = "Blessed";
    });
});

item("MistletoeAddon", |c| {
    c.on_create(|i| {
        i.item_id = 0x2375;
    });
});
