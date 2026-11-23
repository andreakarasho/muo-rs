item("BagOfSending", |c| {
    c.weight = 2.0;
    c.label = 1054104;
    c.on_create(|i| {
        i.item_id = 0xE76;
    });
});

item("BallOfSummoning", |c| {
    c.weight = 10.0;
    c.on_create(|i| {
        i.item_id = 0xE2E;
        i.light = "Circle150";
    });
});

item("BraceletOfBinding", |c| {
    c.weight = 1.0;
    c.graphic = 4230;
    c.on_create(|i| {
        i.hue = 0x489;
    });
});
