item("CherryArmoire", |c| {
    c.weight = 1.0;
    c.on_create(|i| {
        i.item_id = 0x285D;
    });
});

item("Drawer", |c| {
    c.weight = 1.0;
    c.on_create(|i| {
        i.item_id = 0xA2C;
    });
});

item("ElegantArmoire", |c| {
    c.weight = 1.0;
    c.on_create(|i| {
        i.item_id = 0x2859;
    });
});

item("FancyDrawer", |c| {
    c.weight = 1.0;
    c.on_create(|i| {
        i.item_id = 0xA30;
    });
});

item("FancyElvenArmoire", |c| {
    c.weight = 1.0;
    c.gump_id = 78;
    c.drop_sound = 66;
    c.on_create(|i| {
        i.item_id = 0x2D07;
    });
});

item("FullBookcase", |c| {
    c.weight = 1.0;
    c.on_create(|i| {
        i.item_id = 0xA97;
    });
});

item("MapleArmoire", |c| {
    c.weight = 1.0;
    c.on_create(|i| {
        i.item_id = 0x285B;
    });
});

item("RedArmoire", |c| {
    c.weight = 1.0;
    c.on_create(|i| {
        i.item_id = 0x2857;
    });
});

item("ShortCabinet", |c| {
    c.weight = 1.0;
    c.on_create(|i| {
        i.item_id = 0x2817;
    });
});

item("SimpleElvenArmoire", |c| {
    c.weight = 1.0;
    c.gump_id = 79;
    c.drop_sound = 66;
    c.on_create(|i| {
        i.item_id = 0x2D05;
    });
});

item("TallCabinet", |c| {
    c.weight = 1.0;
    c.on_create(|i| {
        i.item_id = 0x2815;
    });
});

item("EmptyBookcase", |c| {
    c.on_create(|i| {
        i.item_id = 0xA9D;
    });
});
