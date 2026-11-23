item("HornOfRetreatMoongate", |c| {
    c.label = 1049114;
});

item("Moonstone", |c| {
    c.weight = 1.0;
    c.on_create(|i| {
        i.item_id = 0xF8B;
    });
});

item("Moongate", |c| {
    c.on_create(|i| {
        i.item_id = 0xF6C;
        i.movable = false;
        i.light = "Circle300";
    });
});

item("PublicMoongate", |c| {
    c.on_create(|i| {
        i.item_id = 0xF6C;
        i.movable = false;
        i.light = "Circle300";
    });
});
