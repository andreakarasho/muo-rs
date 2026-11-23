item("Bandage", |c| {
    c.weight = 0.1;
    c.on_create(|i| {
        i.item_id = 0xE21;
        i.stackable = true;
    });
});
