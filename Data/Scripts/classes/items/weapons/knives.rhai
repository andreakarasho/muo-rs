item("ButcherKnife", |c| {
    c.aos_str_req = 5;
    c.aos_min_damage = 9;
    c.aos_max_damage = 11;
    c.aos_speed = 49;
    c.ml_speed = 2.25;
    c.old_str_req = 5;
    c.old_min_damage = 2;
    c.old_max_damage = 14;
    c.old_speed = 40;
    c.init_min_hits = 31;
    c.init_max_hits = 40;
    c.weight = 1.0;
    c.primary_ability = "InfectiousStrike";
    c.secondary_ability = "Disarm";
    c.graphic = 5110;
});

item("Cleaver", |c| {
    c.aos_str_req = 10;
    c.aos_min_damage = 11;
    c.aos_max_damage = 13;
    c.aos_speed = 46;
    c.ml_speed = 2.5;
    c.old_str_req = 10;
    c.old_min_damage = 2;
    c.old_max_damage = 13;
    c.old_speed = 40;
    c.init_min_hits = 31;
    c.init_max_hits = 50;
    c.weight = 2.0;
    c.primary_ability = "BleedAttack";
    c.secondary_ability = "InfectiousStrike";
    c.graphic = 3779;
});

item("Dagger", |c| {
    c.aos_str_req = 10;
    c.aos_min_damage = 10;
    c.aos_max_damage = 11;
    c.aos_speed = 56;
    c.ml_speed = 2.0;
    c.old_str_req = 1;
    c.old_min_damage = 3;
    c.old_max_damage = 15;
    c.old_speed = 55;
    c.init_min_hits = 31;
    c.init_max_hits = 40;
    c.weight = 1.0;
    c.primary_ability = "InfectiousStrike";
    c.secondary_ability = "ShadowStrike";
    c.def_skill = "Fencing";
    c.def_type = "Piercing";
    c.def_animation = "Pierce1H";
    c.graphic = 3922;
});

item("SkinningKnife", |c| {
    c.aos_str_req = 5;
    c.aos_min_damage = 9;
    c.aos_max_damage = 11;
    c.aos_speed = 49;
    c.ml_speed = 2.25;
    c.old_str_req = 5;
    c.old_min_damage = 1;
    c.old_max_damage = 10;
    c.old_speed = 40;
    c.init_min_hits = 31;
    c.init_max_hits = 40;
    c.weight = 1.0;
    c.primary_ability = "ShadowStrike";
    c.secondary_ability = "Disarm";
    c.graphic = 3780;
});
