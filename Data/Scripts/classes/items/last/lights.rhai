item("Candelabra", |c| {
    c.weight = 3.0;
    c.lit_item_id = 2845;
    c.unlit_item_id = 2599;
    c.on_create(|i| {
        i.item_id = 0xA27;
    });
});

item("HeatingStand", |c| {
    c.weight = 1.0;
    c.lit_item_id = 6218;
    c.unlit_item_id = 6217;
    c.on_create(|i| {
        i.item_id = 0x1849;
        i.burnout_minutes = 25;
        i.burning = false;
        i.light = "Empty";
    });
});

item("RedHangingLantern", |c| {
    c.weight = 3.0;
    c.on_create(|i| {
        i.item_id = 0x24C2;
        i.movable = true;
        i.burnout_minutes = 0;
        i.burning = false;
        i.light = "Circle300";
    });
});

item("WhiteHangingLantern", |c| {
    c.weight = 3.0;
    c.on_create(|i| {
        i.item_id = 0x24C6;
        i.movable = true;
        i.burnout_minutes = 0;
        i.burning = false;
        i.light = "Circle300";
    });
});

item("WallSconce", |c| {
    c.weight = 3.0;
    c.on_create(|i| {
        i.item_id = 0x9FB;
        i.movable = false;
        i.burnout_minutes = 0;
        i.burning = false;
        i.light = "WestBig";
    });
});

item("WallTorch", |c| {
    c.weight = 3.0;
    c.on_create(|i| {
        i.item_id = 0xA05;
        i.movable = false;
        i.burnout_minutes = 0;
        i.burning = false;
        i.light = "WestBig";
    });
});
