item("OilCloth", |c| {
    c.label = 1041498;
    c.weight = 1.0;
    c.on_create(|i| {
        i.item_id = 0x175D;
        i.hue = 0x7D1;
    });
});

item("ClockworkAssembly", |c| {
    c.weight = 5.0;
    c.label = 1073426;
    c.on_create(|i| {
        i.item_id = 0x1EA8;
        i.hue = 0x44E;
    });
});

item("ArcaneGem", |c| {
    c.weight = 1.0;
    c.name = "arcane gem";
    c.on_create(|i| {
        i.item_id = 0x1EA7;
    });
});

item("TribalPaint", |c| {
    c.weight = 2.0;
    c.label = 1040000;
    c.on_create(|i| {
        i.item_id = 0x9EC;
        i.hue = 0x835;
    });
});

item("OilFlask", |c| {
    c.label = 1027199;
    c.on_create(|i| {
        i.item_id = 0x1C18;
        i.stackable = true;
    });
});

item("FishOilFlask", |c| {
    c.label = 1150863;
});

item("Bola", |c| {
    c.weight = 4.0;
    c.on_create(|i| {
        i.item_id = 0x26AC;
        i.stackable = true;
    });
});

item("Firebomb", |c| {
    c.weight = 2.0;
});

item("UnholyBone", |c| {
    c.name = "unholy bone";
    c.on_create(|i| {
        i.item_id = 0xF7E;
        i.movable = false;
        i.hue = 0x497;
    });
});

item("DeceitBrazier", |c| {
    c.label = 1023633;
    c.on_create(|i| {
        i.item_id = 0xE31;
        i.movable = false;
        i.light = "Circle225";
    });
});

item("FancyWindChimes", |c| {
    c.label = 1030291;
});

item("SerpentPillar", |c| {
    c.on_create(|i| {
        i.item_id = 0x233F;
        i.movable = false;
    });
});
