item("InteriorDecorator", |c| {
    c.weight = 1.0;
    c.label = 1041280;
    c.on_create(|i| {
        i.item_id = 0xFC1;
        i.loot_type = "Blessed";
    });
});
