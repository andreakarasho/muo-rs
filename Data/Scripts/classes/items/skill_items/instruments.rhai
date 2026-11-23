item("Harp", |c| {
    c.weight = 35.0;
    c.on_create(|i| {
        i.item_id = 0xEB1;
    });
});

item("Drums", |c| {
    c.weight = 4.0;
    c.on_create(|i| {
        i.item_id = 0xE9C;
    });
});

item("BambooFlute", |c| {
    c.weight = 2.0;
    c.on_create(|i| {
        i.item_id = 0x2805;
    });
});

item("Lute", |c| {
    c.weight = 5.0;
    c.on_create(|i| {
        i.item_id = 0xEB3;
    });
});

item("LapHarp", |c| {
    c.weight = 10.0;
    c.on_create(|i| {
        i.item_id = 0xEB2;
    });
});

item("Tambourine", |c| {
    c.weight = 1.0;
    c.on_create(|i| {
        i.item_id = 0xE9D;
    });
});

item("TambourineTassel", |c| {
    c.weight = 1.0;
    c.on_create(|i| {
        i.item_id = 0xE9E;
    });
});
