item("FireHorn", |c| {
    c.weight = 1.0;
    c.label = 1060456;
    c.on_create(|i| {
        i.item_id = 0xFC7;
        i.hue = 0x466;
    });
});

item("RecipeScroll", |c| {
    c.label = 1074560;
    c.on_create(|i| {
        i.item_id = 0x2831;
    });
});
