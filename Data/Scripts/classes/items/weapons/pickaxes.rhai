item("GargoylesPickaxe", |c| {
    c.aos_str_req = 50;
    c.aos_min_damage = 13;
    c.aos_max_damage = 15;
    c.aos_speed = 35;
    c.ml_speed = 3.0;
    c.old_str_req = 25;
    c.old_min_damage = 1;
    c.old_max_damage = 15;
    c.old_speed = 35;
    c.init_min_hits = 31;
    c.init_max_hits = 60;
    c.label = 1041281;
    c.weight = 11.0;
    c.harvest_system = "Mining";
    c.primary_ability = "DoubleStrike";
    c.secondary_ability = "Disarm";
    c.def_animation = "Slash1H";
});

item("SturdyPickaxe", |c| {
    c.aos_str_req = 50;
    c.aos_min_damage = 13;
    c.aos_max_damage = 15;
    c.aos_speed = 35;
    c.ml_speed = 3.0;
    c.old_str_req = 25;
    c.old_min_damage = 1;
    c.old_max_damage = 15;
    c.old_speed = 35;
    c.label = 1045126;
    c.weight = 11.0;
    c.harvest_system = "Mining";
    c.primary_ability = "DoubleStrike";
    c.secondary_ability = "Disarm";
    c.def_animation = "Slash1H";
    c.graphic = 3718;
    c.on_create(|i| {
        i.hue = 0x973;
        i.uses_remaining = 180;
        i.weapon_show_uses_remaining = true;
    });
});

item("ProspectorsTool", |c| {
    c.label = 1049065;
    c.weight = 9.0;
    c.primary_ability = "CrushingBlow";
    c.secondary_ability = "ShadowStrike";
    c.aos_str_req = 40;
    c.aos_min_damage = 13;
    c.aos_max_damage = 15;
    c.aos_speed = 33;
    c.old_str_req = 10;
    c.old_min_damage = 6;
    c.old_max_damage = 8;
    c.old_speed = 33;
    c.graphic = 4020;
    c.on_create(|i| {
        i.uses_remaining = 50;
        i.weapon_show_uses_remaining = true;
    });
});
