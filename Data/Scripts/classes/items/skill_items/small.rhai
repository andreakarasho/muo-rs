item("Clock", |c| {
    c.weight = 3.0;
});

item("Spyglass", |c| {
    c.weight = 3.0;
    c.on_create(|i| {
        i.item_id = 0x14F5;
    });
});

item("Lockpick", |c| {
    c.on_create(|i| {
        i.item_id = 0x14FC;
        i.stackable = true;
    });
});

item("ClockRight", |c| {
    c.on_create(|i| {
        i.item_id = 0x104B;
    });
});

item("ClockLeft", |c| {
    c.on_create(|i| {
        i.item_id = 0x104C;
    });
});
