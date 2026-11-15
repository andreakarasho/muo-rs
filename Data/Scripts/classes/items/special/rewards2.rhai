item("LeatherGlovesOfMining", |c| {
    c.base_physical_resistance = 2;
    c.base_fire_resistance = 4;
    c.base_cold_resistance = 3;
    c.base_poison_resistance = 3;
    c.base_energy_resistance = 3;
    c.init_min_hits = 30;
    c.init_max_hits = 40;
    c.aos_str_req = 20;
    c.old_str_req = 10;
    c.armor_base = 13;
    c.label = 1045122;
    c.weight = 1.0;
    c.material_type = "Leather";
    c.default_resource = "RegularLeather";
    c.def_med_allowance = "All";
    c.graphic = 5062;
});

item("StuddedGlovesOfMining", |c| {
    c.base_physical_resistance = 2;
    c.base_fire_resistance = 4;
    c.base_cold_resistance = 3;
    c.base_poison_resistance = 3;
    c.base_energy_resistance = 4;
    c.init_min_hits = 35;
    c.init_max_hits = 45;
    c.aos_str_req = 25;
    c.old_str_req = 25;
    c.armor_base = 16;
    c.label = 1045123;
    c.weight = 2.0;
    c.material_type = "Studded";
    c.default_resource = "RegularLeather";
    c.graphic = 5077;
});

item("RingmailGlovesOfMining", |c| {
    c.base_physical_resistance = 3;
    c.base_fire_resistance = 3;
    c.base_cold_resistance = 1;
    c.base_poison_resistance = 5;
    c.base_energy_resistance = 3;
    c.init_min_hits = 40;
    c.init_max_hits = 50;
    c.aos_str_req = 40;
    c.old_str_req = 20;
    c.old_dex_bonus = -1;
    c.armor_base = 22;
    c.label = 1045124;
    c.weight = 1.0;
    c.material_type = "Ringmail";
    c.graphic = 5099;
});

item("OssianGrimoire", |c| {
    c.label = 1078148;
});

item("ColoredAnvil", |c| {
    c.weight = 20.0;
    c.on_create(|i| {
        i.item_id = 0xFAF;
    });
});

item("AncientSmithyHammer", |c| {
    c.weight = 8.0;
    c.label = 1045127;
});

item("PowderOfTemperament", |c| {
    c.weight = 1.0;
    c.label = 1049082;
    c.on_create(|i| {
        i.item_id = 0x1006;
        i.hue = 0x973;
    });
});
