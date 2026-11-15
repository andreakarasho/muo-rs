item("BlackStaff", |c| {
    c.aos_str_req = 35;
    c.aos_min_damage = 13;
    c.aos_max_damage = 16;
    c.aos_speed = 39;
    c.ml_speed = 2.75;
    c.old_str_req = 35;
    c.old_min_damage = 8;
    c.old_max_damage = 33;
    c.old_speed = 35;
    c.init_min_hits = 31;
    c.init_max_hits = 70;
    c.weight = 6.0;
    c.primary_ability = "WhirlwindAttack";
    c.secondary_ability = "ParalyzingBlow";
    c.graphic = 3568;
});

item("GlassStaff", |c| {
    c.aos_str_req = 20;
    c.aos_min_damage = 11;
    c.aos_max_damage = 14;
    c.ml_speed = 2.25;
    c.init_min_hits = 31;
    c.init_max_hits = 70;
    c.weight = 4.0;
    c.primary_ability = "DoubleStrike";
    c.secondary_ability = "MortalStrike";
    c.required_races = "AllowGargoylesOnly";
    c.graphic = 2309;
    c.on_create(|i| {
        i.resource = "None";
    });
});

item("GnarledStaff", |c| {
    c.aos_str_req = 20;
    c.aos_min_damage = 15;
    c.aos_max_damage = 17;
    c.aos_speed = 33;
    c.ml_speed = 3.25;
    c.old_str_req = 20;
    c.old_min_damage = 10;
    c.old_max_damage = 30;
    c.old_speed = 33;
    c.init_min_hits = 31;
    c.init_max_hits = 50;
    c.weight = 3.0;
    c.primary_ability = "ConcussionBlow";
    c.secondary_ability = "ForceOfNature";
    c.graphic = 5112;
    c.on_create(|i| {
        i.resource = "RegularWood";
    });
});

item("QuarterStaff", |c| {
    c.aos_str_req = 30;
    c.aos_min_damage = 11;
    c.aos_max_damage = 14;
    c.aos_speed = 48;
    c.ml_speed = 2.25;
    c.old_str_req = 30;
    c.old_min_damage = 8;
    c.old_max_damage = 28;
    c.old_speed = 48;
    c.init_min_hits = 31;
    c.init_max_hits = 60;
    c.weight = 4.0;
    c.primary_ability = "DoubleStrike";
    c.secondary_ability = "ConcussionBlow";
    c.graphic = 3721;
    c.on_create(|i| {
        i.resource = "RegularWood";
    });
});

item("SerpentstoneStaff", |c| {
    c.aos_str_req = 35;
    c.aos_min_damage = 16;
    c.aos_max_damage = 19;
    c.ml_speed = 3.5;
    c.init_min_hits = 31;
    c.init_max_hits = 50;
    c.primary_ability = "CrushingBlow";
    c.secondary_ability = "Dismount";
    c.required_races = "AllowGargoylesOnly";
    c.graphic = 2310;
    c.on_create(|i| {
        i.resource = "RegularWood";
    });
});

item("ShepherdsCrook", |c| {
    c.aos_str_req = 20;
    c.aos_min_damage = 13;
    c.aos_max_damage = 15;
    c.aos_speed = 40;
    c.ml_speed = 2.75;
    c.old_str_req = 10;
    c.old_min_damage = 3;
    c.old_max_damage = 12;
    c.old_speed = 30;
    c.init_min_hits = 31;
    c.init_max_hits = 50;
    c.weight = 4.0;
    c.primary_ability = "CrushingBlow";
    c.secondary_ability = "Disarm";
    c.graphic = 3713;
    c.on_create(|i| {
        i.resource = "RegularWood";
    });
});

item("ChylothStaff", |c| {
    c.label = 1041111;
    c.on_create(|i| {
        i.hue = 0x482;
    });
});

item("GlacialStaff", |c| {
    c.script = "content/equipment/glacial_staff.rhai";
    c.label = 1017413;
    c.on_create(|i| {
        i.hue = 0x480;
    });
});
