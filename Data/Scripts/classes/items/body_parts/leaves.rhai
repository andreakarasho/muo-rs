item("LeftArm", |c| {
    c.on_create(|i| {
        i.item_id = 0x1DA1;
    });
});

item("LeftLeg", |c| {
    c.on_create(|i| {
        i.item_id = 0x1DA3;
    });
});

item("RightArm", |c| {
    c.on_create(|i| {
        i.item_id = 0x1DA2;
    });
});

item("RightLeg", |c| {
    c.on_create(|i| {
        i.item_id = 0x1DA4;
    });
});

item("Torso", |c| {
    c.weight = 2.0;
    c.on_create(|i| {
        i.item_id = 0x1D9F;
    });
});
