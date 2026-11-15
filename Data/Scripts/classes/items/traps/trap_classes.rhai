item("FireColumnTrap", |c| {
    c.on_create(|i| {
        i.item_id = 0x1B71;
    });
});

item("FlameSpurtTrap", |c| {
    c.on_create(|i| {
        i.item_id = 0x1B71;
        i.visible = false;
    });
});

item("StoneFaceTrap", |c| {
    c.on_create(|i| {
        i.item_id = 0x10FC;
        i.light = "Circle225";
    });
});

item("StoneFaceTrapNoDamage", |c| {
    c.on_create(|i| {
        i.item_id = 0x10FC;
        i.light = "Circle225";
    });
});

item("GiantSpikeTrap", |c| {
    c.on_create(|i| {
        i.item_id = 1;
    });
});

item("MushroomTrap", |c| {
    c.on_create(|i| {
        i.item_id = 0x1125;
    });
});
