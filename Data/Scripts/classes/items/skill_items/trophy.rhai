item("TaxidermyKit", |c| {
    c.weight = 1.0;
    c.label = 1041279;
    c.on_create(|i| {
        i.item_id = 0x1EBA;
    });
});

item("TrophyDeed", |c| {
    c.on_create(|i| {
        i.item_id = 0x14F0;
    });
});
