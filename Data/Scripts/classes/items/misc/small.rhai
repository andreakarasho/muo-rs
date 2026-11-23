item("TrashChest", |c| {
    c.max_weight = 0;
    c.on_create(|i| {
        i.item_id = 0xE41;
        i.movable = false;
    });
});

item("TrashBarrel", |c| {
    c.label = 1041064;
    c.max_weight = 0;
    c.on_create(|i| {
        i.item_id = 0xE77;
        i.hue = 0x3B2;
        i.movable = false;
    });
});

item("Acid", |c| {
    c.name = "a pool of acid";
    c.on_create(|i| {
        i.item_id = 0x122A;
        i.hue = 0x3F;
        i.movable = false;
    });
});

item("OrigamiPaper", |c| {
    c.label = 1030288;
    c.on_create(|i| {
        i.item_id = 0x2830;
    });
});

item("EmptyOilFlask", |c| {
    c.label = 1150866;
    c.on_create(|i| {
        i.item_id = 0x1C18;
        i.stackable = true;
    });
});
