item("WreathDeed", |c| {
    c.weight = 1.0;
    c.label = 1062837;
    c.on_create(|i| {
        i.item_id = 0x14F0;
    });
});

item("WreathAddon", |c| {
    c.on_create(|i| {
        i.item_id = 0x232C;
    });
});
