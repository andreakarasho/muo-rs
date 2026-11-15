item("FillableLargeCrate", |c| {
    c.on_create(|i| {
        i.item_id = 0xE3D;
    });
});

item("FillableSmallCrate", |c| {
    c.on_create(|i| {
        i.item_id = 0x9A9;
    });
});

item("FillableWoodenBox", |c| {
    c.on_create(|i| {
        i.item_id = 0x9AA;
    });
});

item("FillableMetalBox", |c| {
    c.on_create(|i| {
        i.item_id = 0x9A8;
    });
});

item("FillableBarrel", |c| {
    c.on_create(|i| {
        i.item_id = 0xE77;
    });
});

item("FillableMetalChest", |c| {
    c.on_create(|i| {
        i.item_id = 0x9AB;
    });
});

item("FillableMetalGoldenChest", |c| {
    c.on_create(|i| {
        i.item_id = 0xE41;
    });
});

item("FillableWoodenChest", |c| {
    c.on_create(|i| {
        i.item_id = 0xE43;
    });
});
