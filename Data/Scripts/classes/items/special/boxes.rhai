item("RedVelvetGiftBox", |c| {
    c.gump_id = 63;
    c.label = 1077596;
    c.on_create(|i| {
        i.item_id = 0xE7A;
        i.hue = 0x20;
    });
});

item("ValentinesCard", |c| {
    c.name = "a Valentine's card";
});

item("CommodityDeedBox", |c| {
    c.weight = 4.0;
    c.label = 1080523;
    c.gump_id = 67;
    c.on_create(|i| {
        i.item_id = 0x9AA;
        i.hue = 0x47;
    });
});
