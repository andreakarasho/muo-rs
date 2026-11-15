item("EmptyBentoBox", |c| {
    c.weight = 5.0;
    c.on_create(|i| {
        i.item_id = 0x2834;
    });
});

item("GreenTeaBasket", |c| {
    c.weight = 10.0;
    c.on_create(|i| {
        i.item_id = 0x284B;
    });
});

item("Glass", |c| {
    c.weight = 0.1;
    c.on_create(|i| {
        i.item_id = 0x1F81;
    });
});

item("GlassBottle", |c| {
    c.weight = 0.3;
    c.on_create(|i| {
        i.item_id = 0xE2B;
    });
});

item("EmptyPewterBowl", |c| {
    c.weight = 1.0;
    c.on_create(|i| {
        i.item_id = 0x15FD;
    });
});

item("EmptyPewterTub", |c| {
    c.weight = 2.0;
    c.on_create(|i| {
        i.item_id = 0x1603;
    });
});

item("EmptyWoodenBowl", |c| {
    c.weight = 1.0;
    c.on_create(|i| {
        i.item_id = 0x15F8;
    });
});

item("EmptyWoodenTub", |c| {
    c.weight = 2.0;
    c.on_create(|i| {
        i.item_id = 0x1605;
    });
});

item("CocoaButter", |c| {
    c.label = 1080005;
    c.weight = 1.0;
    c.on_create(|i| {
        i.item_id = 0x1044;
        i.hue = 0x457;
    });
});

item("CocoaLiquor", |c| {
    c.label = 1080007;
    c.weight = 1.0;
    c.on_create(|i| {
        i.item_id = 0x103F;
        i.hue = 0x46A;
    });
});

item("Eggshells", |c| {
    c.weight = 0.5;
    c.on_create(|i| {
        i.item_id = 0x9B4;
    });
});

item("WoodenBowl", |c| {
    c.weight = 1.0;
    c.on_create(|i| {
        i.item_id = 0x15F8;
    });
});

item("SheafOfHay", |c| {
    c.weight = 10.0;
    c.on_create(|i| {
        i.item_id = 0xF36;
    });
});

item("CocoaPulp", |c| {
    c.label = 1080530;
    c.weight = 1.0;
    c.on_create(|i| {
        i.item_id = 0xF7C;
        i.hue = 0x219;
        i.stackable = true;
    });
});

item("SackOfSugar", |c| {
    c.label = 1080003;
    c.weight = 1.0;
    c.on_create(|i| {
        i.item_id = 0x1039;
        i.hue = 0x461;
        i.stackable = true;
    });
});

item("Vanilla", |c| {
    c.label = 1080009;
    c.weight = 1.0;
    c.on_create(|i| {
        i.item_id = 0xE2A;
        i.hue = 0x462;
        i.stackable = true;
    });
});
