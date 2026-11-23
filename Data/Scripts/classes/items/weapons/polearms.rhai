item("Bardiche", |c| {
    c.aos_str_req = 45;
    c.aos_min_damage = 17;
    c.aos_max_damage = 18;
    c.aos_speed = 28;
    c.ml_speed = 3.75;
    c.old_str_req = 40;
    c.old_min_damage = 5;
    c.old_max_damage = 43;
    c.old_speed = 26;
    c.init_min_hits = 31;
    c.init_max_hits = 100;
    c.weight = 7.0;
    c.primary_ability = "ParalyzingBlow";
    c.secondary_ability = "Dismount";
    c.graphic = 3917;
});

item("Halberd", |c| {
    c.aos_str_req = 95;
    c.aos_min_damage = 18;
    c.aos_max_damage = 19;
    c.aos_speed = 25;
    c.ml_speed = 4.25;
    c.old_str_req = 45;
    c.old_min_damage = 5;
    c.old_max_damage = 49;
    c.old_speed = 25;
    c.init_min_hits = 31;
    c.init_max_hits = 80;
    c.weight = 16.0;
    c.primary_ability = "WhirlwindAttack";
    c.secondary_ability = "ConcussionBlow";
    c.graphic = 5182;
});

item("Scythe", |c| {
    c.aos_str_req = 45;
    c.aos_min_damage = 15;
    c.aos_max_damage = 18;
    c.aos_speed = 32;
    c.ml_speed = 3.5;
    c.old_str_req = 45;
    c.old_min_damage = 15;
    c.old_max_damage = 18;
    c.old_speed = 32;
    c.init_min_hits = 31;
    c.init_max_hits = 100;
    c.weight = 5.0;
    c.primary_ability = "BleedAttack";
    c.secondary_ability = "ParalyzingBlow";
    c.harvest_system = ();
    c.graphic = 9914;
});
