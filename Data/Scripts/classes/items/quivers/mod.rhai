item("BaseQuiver", |c| {
    c.gump_id = 264;
    c.max_items = 1;
    c.max_weight = 50;
    c.weight = 2.0;
});

item("ElvenQuiver", |c| {
    c.label = 1032657;
    c.graphic = 12215;
    c.on_create(|i| {
        i.quiver_weight_reduction = 30;
    });
});

item("QuiverOfFire", |c| {
    c.label = 1073109;
    c.bow_damage_split = [50, 50, 0, 0, 0, 0, 0];
    c.on_create(|i| {
        i.hue = 0x4E7;
    });
});

item("QuiverOfIce", |c| {
    c.label = 1073110;
    c.bow_damage_split = [50, 0, 50, 0, 0, 0, 0];
    c.on_create(|i| {
        i.hue = 0x4ED;
    });
});

item("QuiverOfBlight", |c| {
    c.label = 1073111;
    c.bow_damage_split = [0, 0, 50, 50, 0, 0, 0];
    c.on_create(|i| {
        i.hue = 0x4F3;
    });
});

item("QuiverOfLightning", |c| {
    c.label = 1073112;
    c.bow_damage_split = [50, 0, 0, 0, 50, 0, 0];
    c.on_create(|i| {
        i.hue = 0x4F9;
    });
});

item("QuiverOfElements", |c| {
    c.label = 1075040;
    c.bow_damage_split = [0, 0, 0, 0, 0, 100, 0];
    c.graphic = 12215;
    c.on_create(|i| {
        i.hue = 0xEB;
        i.quiver_weight_reduction = 50;
    });
});

item("QuiverOfRage", |c| {
    c.label = 1075038;
    c.bow_damage_split = [20, 20, 20, 20, 20, 0, 0];
    c.graphic = 12215;
    c.on_create(|i| {
        i.hue = 0x24C;
        i.quiver_weight_reduction = 25;
        i.quiver_damage_increase = 10;
    });
});
