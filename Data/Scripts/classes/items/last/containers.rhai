item("Armoire", |c| {
    c.weight = 1.0;
    c.on_create(|i| {
        i.item_id = 0xA4F;
    });
});

item("FancyArmoire", |c| {
    c.weight = 1.0;
    c.on_create(|i| {
        i.item_id = 0xA4D;
    });
});
