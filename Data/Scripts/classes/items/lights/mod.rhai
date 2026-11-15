item("Torch", |c| {
    c.weight = 1.0;
    c.lit_item_id = 2578;
    c.unlit_item_id = 3947;
    c.on_create(|i| {
        i.item_id = 0xF6B;
        i.burnout_minutes = 30;
        i.burning = false;
        i.stackable = true;
        i.light = "Circle300";
    });
});

item("Lantern", |c| {
    c.weight = 2.0;
    c.on_create(|i| {
        i.item_id = 0xA25;
    });
});

item("LanternOfSouls", |c| {
    c.label = 1061618;
});
