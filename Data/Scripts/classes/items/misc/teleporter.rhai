item("Teleporter", |c| {
    c.label = 1026095;
    c.on_create(|i| {
        i.item_id = 0x1BC3;
        i.movable = false;
        i.visible = false;
    });
});

item("TimeoutGoal", |c| {
    c.name = "timeout teleporter goal";
    c.on_create(|i| {
        i.item_id = 0x1822;
        i.movable = false;
        i.visible = false;
        i.hue = 0x482;
    });
});
