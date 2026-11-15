item("Kindling", |c| {
    c.weight = 5.0;
    c.on_create(|i| {
        i.item_id = 0xDE1;
        i.stackable = true;
    });
});

item("Bedroll", |c| {
    c.weight = 5.0;
    c.on_create(|i| {
        i.item_id = 0xA57;
    });
});

item("Campfire", |c| {
    c.on_create(|i| {
        i.item_id = 0xDE3;
        i.movable = false;
        i.light = "Circle300";
    });
});
