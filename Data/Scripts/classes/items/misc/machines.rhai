item("EffectController", |c| {
    c.name = "Effect Controller";
    c.on_create(|i| {
        i.item_id = 0x1B72;
        i.movable = false;
        i.visible = false;
    });
});

item("BroadcastCrystal", |c| {
    c.label = 1060740;
    c.on_create(|i| {
        i.item_id = 0x1ED0;
        i.light = "Circle150";
    });
});

item("ReceiverCrystal", |c| {
    c.label = 1060740;
    c.on_create(|i| {
        i.item_id = 0x1ED0;
        i.light = "Circle150";
    });
});

item("ControlPanel", |c| {
    c.name = "a control panel";
});

item("ProjectedItem", |c| {
    c.on_create(|i| {
        i.item_id = 1;
        i.movable = false;
        i.visible = false;
    });
});
