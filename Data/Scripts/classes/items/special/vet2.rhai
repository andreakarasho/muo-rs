item("BannerDeed", |c| {
    c.weight = 1.0;
    c.label = 1041007;
    c.on_create(|i| {
        i.item_id = 0x14F0;
    });
});

item("FlamingHead", |c| {
    c.label = 1041266;
    c.on_create(|i| {
        i.item_id = 0x10FC;
    });
});

item("FlamingHeadDeed", |c| {
    c.weight = 1.0;
    c.label = 1041050;
    c.on_create(|i| {
        i.item_id = 0x14F0;
    });
});

item("MiniHouseDeed", |c| {
    c.weight = 1.0;
    c.label = 1062096;
});
