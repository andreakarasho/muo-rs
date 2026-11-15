item("LockableBarrel", |c| {
    c.weight = 1.0;
    c.on_create(|i| {
        i.item_id = 0xE77;
    });
});

item("MovingCrate", |c| {
    c.label = 1061690;
});

item("PackingBox", |c| {
    c.label = 1061690;
});

item("TransferItem", |c| {
    c.name = "a house transfer contract";
});

item("BrigandCamp", |c| {
    c.on_create(|i| {
        i.item_id = 0x10EE;
    });
});

item("LizardmenCamp", |c| {
    c.on_create(|i| {
        i.item_id = 0x10EE;
    });
});

item("RatCamp", |c| {
    c.on_create(|i| {
        i.item_id = 0x10EE;
    });
});

item("OrcCamp", |c| {
    c.on_create(|i| {
        i.item_id = 0x10EE;
    });
});

item("HealerCamp", |c| {
    c.on_create(|i| {
        i.item_id = 0x1F4;
    });
});

item("BankerCamp", |c| {
    c.on_create(|i| {
        i.item_id = 0x1F6;
    });
});

item("MageCamp", |c| {
    c.on_create(|i| {
        i.item_id = 0x1F5;
    });
});
