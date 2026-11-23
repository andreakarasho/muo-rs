item("Amber", |c| {
    c.weight = 0.1;
    c.on_create(|i| {
        i.item_id = 0xF25;
        i.stackable = true;
    });
});

item("Amethyst", |c| {
    c.weight = 0.1;
    c.on_create(|i| {
        i.item_id = 0xF16;
        i.stackable = true;
    });
});

item("Citrine", |c| {
    c.weight = 0.1;
    c.on_create(|i| {
        i.item_id = 0xF15;
        i.stackable = true;
    });
});

item("Diamond", |c| {
    c.weight = 0.1;
    c.on_create(|i| {
        i.item_id = 0xF26;
        i.stackable = true;
    });
});

item("Emerald", |c| {
    c.weight = 0.1;
    c.on_create(|i| {
        i.item_id = 0xF10;
        i.stackable = true;
    });
});

item("Ruby", |c| {
    c.weight = 0.1;
    c.on_create(|i| {
        i.item_id = 0xF13;
        i.stackable = true;
    });
});

item("Sapphire", |c| {
    c.weight = 0.1;
    c.on_create(|i| {
        i.item_id = 0xF19;
        i.stackable = true;
    });
});

item("StarSapphire", |c| {
    c.weight = 0.1;
    c.on_create(|i| {
        i.item_id = 0xF21;
        i.stackable = true;
    });
});

item("Tourmaline", |c| {
    c.weight = 0.1;
    c.on_create(|i| {
        i.item_id = 0xF2D;
        i.stackable = true;
    });
});
