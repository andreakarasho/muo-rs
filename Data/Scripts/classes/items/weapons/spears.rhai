item("BladedStaff", |c| {
    c.aos_str_req = 40;
    c.aos_min_damage = 14;
    c.aos_max_damage = 16;
    c.aos_speed = 37;
    c.ml_speed = 3.0;
    c.old_str_req = 40;
    c.old_min_damage = 14;
    c.old_max_damage = 16;
    c.old_speed = 37;
    c.init_min_hits = 21;
    c.init_max_hits = 110;
    c.weight = 4.0;
    c.primary_ability = "ArmorIgnore";
    c.secondary_ability = "Dismount";
    c.def_skill = "Swords";
    c.graphic = 9917;
});

item("DoubleBladedStaff", |c| {
    c.aos_str_req = 50;
    c.aos_min_damage = 12;
    c.aos_max_damage = 13;
    c.aos_speed = 49;
    c.ml_speed = 2.25;
    c.old_str_req = 50;
    c.old_min_damage = 12;
    c.old_max_damage = 13;
    c.old_speed = 49;
    c.init_min_hits = 31;
    c.init_max_hits = 80;
    c.weight = 2.0;
    c.primary_ability = "DoubleStrike";
    c.secondary_ability = "InfectiousStrike";
    c.graphic = 9919;
});

item("DualPointedSpear", |c| {
    c.aos_str_req = 50;
    c.aos_min_damage = 11;
    c.aos_max_damage = 14;
    c.ml_speed = 2.25;
    c.init_min_hits = 31;
    c.init_max_hits = 80;
    c.primary_ability = "DoubleStrike";
    c.secondary_ability = "Disarm";
    c.required_races = "AllowGargoylesOnly";
    c.graphic = 2308;
});

item("Pike", |c| {
    c.aos_str_req = 50;
    c.aos_min_damage = 14;
    c.aos_max_damage = 16;
    c.aos_speed = 37;
    c.ml_speed = 3.0;
    c.old_str_req = 50;
    c.old_min_damage = 14;
    c.old_max_damage = 16;
    c.old_speed = 37;
    c.init_min_hits = 31;
    c.init_max_hits = 110;
    c.weight = 8.0;
    c.primary_ability = "ParalyzingBlow";
    c.secondary_ability = "InfectiousStrike";
    c.graphic = 9918;
});

item("Pitchfork", |c| {
    c.aos_str_req = 55;
    c.aos_min_damage = 13;
    c.aos_max_damage = 14;
    c.aos_speed = 43;
    c.ml_speed = 2.5;
    c.old_str_req = 15;
    c.old_min_damage = 4;
    c.old_max_damage = 16;
    c.old_speed = 45;
    c.init_min_hits = 31;
    c.init_max_hits = 60;
    c.weight = 11.0;
    c.primary_ability = "BleedAttack";
    c.secondary_ability = "Dismount";
    c.graphic = 3719;
});

item("ShortSpear", |c| {
    c.aos_str_req = 40;
    c.aos_min_damage = 10;
    c.aos_max_damage = 13;
    c.aos_speed = 55;
    c.ml_speed = 2.0;
    c.old_str_req = 15;
    c.old_min_damage = 4;
    c.old_max_damage = 32;
    c.old_speed = 50;
    c.init_min_hits = 31;
    c.init_max_hits = 70;
    c.weight = 4.0;
    c.primary_ability = "ShadowStrike";
    c.secondary_ability = "MortalStrike";
    c.def_animation = "Pierce1H";
    c.graphic = 5123;
});

item("Spear", |c| {
    c.aos_str_req = 50;
    c.aos_min_damage = 13;
    c.aos_max_damage = 15;
    c.aos_speed = 42;
    c.ml_speed = 2.75;
    c.old_str_req = 30;
    c.old_min_damage = 2;
    c.old_max_damage = 36;
    c.old_speed = 46;
    c.init_min_hits = 31;
    c.init_max_hits = 80;
    c.weight = 7.0;
    c.primary_ability = "ArmorIgnore";
    c.secondary_ability = "ParalyzingBlow";
    c.graphic = 3938;
});

item("TribalSpear", |c| {
    c.aos_str_req = 50;
    c.aos_min_damage = 13;
    c.aos_max_damage = 15;
    c.aos_speed = 42;
    c.ml_speed = 2.75;
    c.old_str_req = 30;
    c.old_min_damage = 2;
    c.old_max_damage = 36;
    c.old_speed = 46;
    c.init_min_hits = 31;
    c.init_max_hits = 80;
    c.weight = 7.0;
    c.primary_ability = "ArmorIgnore";
    c.secondary_ability = "ParalyzingBlow";
    c.virtual_damage_bonus = 25;
    c.graphic = 3938;
    c.name = "a tribal spear";
    c.on_create(|i| {
        i.hue = 0x345;
    });
});

item("WarFork", |c| {
    c.aos_str_req = 45;
    c.aos_min_damage = 12;
    c.aos_max_damage = 13;
    c.aos_speed = 43;
    c.ml_speed = 2.5;
    c.old_str_req = 35;
    c.old_min_damage = 4;
    c.old_max_damage = 32;
    c.old_speed = 45;
    c.def_hit_sound = 566;
    c.def_miss_sound = 568;
    c.init_min_hits = 31;
    c.init_max_hits = 110;
    c.weight = 9.0;
    c.primary_ability = "BleedAttack";
    c.secondary_ability = "Disarm";
    c.def_animation = "Pierce1H";
    c.graphic = 5125;
});
