item("Sextant", |c| {
    c.weight = 2.0;
    c.on_create(|i| {
        i.item_id = 0x1058;
    });
});

item("RecallRune", |c| {
    c.weight = 1.0;
    c.on_create(|i| {
        i.item_id = 0x1F14;
    });
});

item("Runebook", |c| {
    c.label = 1041267;
});
