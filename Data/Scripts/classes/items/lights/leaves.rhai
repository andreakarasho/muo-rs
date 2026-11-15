item("DarkSource", |c| {
    c.on_create(|i| {
        i.item_id = 0x1646;
        i.layer = "TwoHanded";
        i.movable = false;
    });
});

item("LightSource", |c| {
    c.on_create(|i| {
        i.item_id = 0x1647;
        i.layer = "TwoHanded";
        i.movable = false;
    });
});

item("Brazier", |c| {
    c.weight = 20.0;
    c.lit_item_id = 3633;
    c.on_create(|i| {
        i.item_id = 0xE31;
        i.movable = false;
        i.burnout_minutes = 0;
        i.burning = true;
        i.light = "Circle225";
    });
});

item("BrazierTall", |c| {
    c.weight = 25.0;
    c.lit_item_id = 6570;
    c.on_create(|i| {
        i.item_id = 0x19AA;
        i.movable = false;
        i.burnout_minutes = 0;
        i.burning = true;
        i.light = "Circle300";
    });
});

item("CandelabraStand", |c| {
    c.weight = 20.0;
    c.lit_item_id = 2854;
    c.unlit_item_id = 2601;
    c.on_create(|i| {
        i.item_id = 0xA29;
        i.burnout_minutes = 0;
        i.burning = false;
        i.light = "Circle225";
    });
});

item("Candle", |c| {
    c.weight = 1.0;
    c.lit_item_id = 2575;
    c.unlit_item_id = 2600;
    c.on_create(|i| {
        i.item_id = 0xA28;
        i.burnout_minutes = 20;
        i.burning = false;
        i.stackable = true;
        i.light = "Circle150";
    });
});

item("CandleLarge", |c| {
    c.weight = 2.0;
    c.lit_item_id = 2842;
    c.unlit_item_id = 2598;
    c.on_create(|i| {
        i.item_id = 0xA26;
        i.burnout_minutes = 25;
        i.burning = false;
        i.light = "Circle150";
    });
});

item("CandleLong", |c| {
    c.weight = 1.0;
    c.lit_item_id = 5168;
    c.unlit_item_id = 5171;
    c.on_create(|i| {
        i.item_id = 0x1433;
        i.burnout_minutes = 30;
        i.burning = false;
        i.light = "Circle150";
    });
});

item("CandleShort", |c| {
    c.weight = 1.0;
    c.lit_item_id = 5164;
    c.unlit_item_id = 5167;
    c.on_create(|i| {
        i.item_id = 0x142F;
        i.burnout_minutes = 25;
        i.burning = false;
        i.light = "Circle150";
    });
});

item("CandleSkull", |c| {
    c.weight = 5.0;
    c.on_create(|i| {
        i.item_id = 0x1853;
        i.burnout_minutes = 25;
        i.burning = false;
        i.light = "Circle150";
    });
});

item("HangingLantern", |c| {
    c.weight = 40.0;
    c.lit_item_id = 2586;
    c.unlit_item_id = 2589;
    c.on_create(|i| {
        i.item_id = 0xA1D;
        i.movable = false;
        i.burnout_minutes = 0;
        i.burning = false;
        i.light = "Circle300";
    });
});

item("LampPost1", |c| {
    c.weight = 40.0;
    c.lit_item_id = 2848;
    c.unlit_item_id = 2849;
    c.on_create(|i| {
        i.item_id = 0xB21;
        i.movable = false;
        i.burnout_minutes = 0;
        i.burning = false;
        i.light = "Circle300";
    });
});

item("LampPost2", |c| {
    c.weight = 40.0;
    c.lit_item_id = 2850;
    c.unlit_item_id = 2851;
    c.on_create(|i| {
        i.item_id = 0xB23;
        i.movable = false;
        i.burnout_minutes = 0;
        i.burning = false;
        i.light = "Circle300";
    });
});

item("LampPost3", |c| {
    c.weight = 40.0;
    c.lit_item_id = 2852;
    c.unlit_item_id = 2853;
    c.on_create(|i| {
        i.item_id = 0xB25;
        i.movable = false;
        i.burnout_minutes = 0;
        i.burning = false;
        i.light = "Circle300";
    });
});

item("PaperLantern", |c| {
    c.weight = 3.0;
    c.lit_item_id = 9405;
    c.unlit_item_id = 9406;
    c.on_create(|i| {
        i.item_id = 0x24BE;
        i.movable = true;
        i.burnout_minutes = 0;
        i.burning = false;
        i.light = "Circle150";
    });
});

item("RoundPaperLantern", |c| {
    c.weight = 3.0;
    c.lit_item_id = 9417;
    c.unlit_item_id = 9418;
    c.on_create(|i| {
        i.item_id = 0x24CA;
        i.movable = true;
        i.burnout_minutes = 0;
        i.burning = false;
        i.light = "Circle150";
    });
});

item("ShojiLantern", |c| {
    c.weight = 3.0;
    c.lit_item_id = 9403;
    c.unlit_item_id = 9404;
    c.on_create(|i| {
        i.item_id = 0x24BC;
        i.movable = true;
        i.burnout_minutes = 0;
        i.burning = false;
        i.light = "Circle150";
    });
});
