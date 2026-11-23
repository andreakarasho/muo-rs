item("Bokuto", |c| {
    c.aos_str_req = 20;
    c.aos_min_damage = 9;
    c.aos_max_damage = 11;
    c.aos_speed = 53;
    c.ml_speed = 2.0;
    c.old_str_req = 20;
    c.old_min_damage = 9;
    c.old_max_damage = 11;
    c.old_speed = 53;
    c.def_hit_sound = 1334;
    c.def_miss_sound = 570;
    c.init_min_hits = 25;
    c.init_max_hits = 50;
    c.weight = 7.0;
    c.primary_ability = "Feint";
    c.secondary_ability = "NerveStrike";
    c.graphic = 10152;
});

item("Daisho", |c| {
    c.aos_str_req = 40;
    c.aos_min_damage = 13;
    c.aos_max_damage = 15;
    c.aos_speed = 40;
    c.ml_speed = 2.75;
    c.old_str_req = 40;
    c.old_min_damage = 13;
    c.old_max_damage = 15;
    c.old_speed = 40;
    c.def_hit_sound = 571;
    c.def_miss_sound = 570;
    c.init_min_hits = 45;
    c.init_max_hits = 65;
    c.weight = 8.0;
    c.primary_ability = "Feint";
    c.secondary_ability = "DoubleStrike";
    c.graphic = 10153;
    c.on_create(|i| {
        i.layer = "TwoHanded";
    });
});

item("Kama", |c| {
    c.aos_str_req = 15;
    c.aos_min_damage = 9;
    c.aos_max_damage = 11;
    c.aos_speed = 55;
    c.ml_speed = 2.0;
    c.old_str_req = 15;
    c.old_min_damage = 9;
    c.old_max_damage = 11;
    c.old_speed = 55;
    c.def_hit_sound = 562;
    c.def_miss_sound = 568;
    c.init_min_hits = 35;
    c.init_max_hits = 60;
    c.weight = 7.0;
    c.primary_ability = "WhirlwindAttack";
    c.secondary_ability = "DefenseMastery";
    c.def_skill = "Fencing";
    c.def_type = "Piercing";
    c.def_animation = "Pierce1H";
    c.graphic = 10157;
    c.on_create(|i| {
        i.layer = "TwoHanded";
    });
});

item("Lajatang", |c| {
    c.aos_str_req = 65;
    c.aos_min_damage = 16;
    c.aos_max_damage = 18;
    c.aos_speed = 32;
    c.ml_speed = 3.5;
    c.old_str_req = 65;
    c.old_min_damage = 16;
    c.old_max_damage = 18;
    c.old_speed = 55;
    c.def_hit_sound = 562;
    c.def_miss_sound = 568;
    c.init_min_hits = 90;
    c.init_max_hits = 95;
    c.weight = 12.0;
    c.primary_ability = "DefenseMastery";
    c.secondary_ability = "FrenziedWhirlwind";
    c.def_skill = "Fencing";
    c.def_type = "Piercing";
    c.def_animation = "Pierce1H";
    c.graphic = 10151;
    c.on_create(|i| {
        i.layer = "TwoHanded";
    });
});

item("NoDachi", |c| {
    c.aos_str_req = 40;
    c.aos_min_damage = 16;
    c.aos_max_damage = 18;
    c.aos_speed = 35;
    c.ml_speed = 3.5;
    c.old_str_req = 40;
    c.old_min_damage = 16;
    c.old_max_damage = 18;
    c.old_speed = 35;
    c.def_hit_sound = 571;
    c.def_miss_sound = 570;
    c.init_min_hits = 31;
    c.init_max_hits = 90;
    c.weight = 10.0;
    c.primary_ability = "CrushingBlow";
    c.secondary_ability = "RidingSwipe";
    c.graphic = 10146;
    c.on_create(|i| {
        i.layer = "TwoHanded";
    });
});

item("Nunchaku", |c| {
    c.aos_str_req = 15;
    c.aos_min_damage = 11;
    c.aos_max_damage = 13;
    c.aos_speed = 47;
    c.ml_speed = 2.5;
    c.old_str_req = 15;
    c.old_min_damage = 11;
    c.old_max_damage = 13;
    c.old_speed = 47;
    c.def_hit_sound = 1333;
    c.def_miss_sound = 569;
    c.init_min_hits = 40;
    c.init_max_hits = 55;
    c.weight = 5.0;
    c.primary_ability = "Block";
    c.secondary_ability = "Feint";
    c.graphic = 10158;
});

item("Sai", |c| {
    c.aos_str_req = 15;
    c.aos_min_damage = 9;
    c.aos_max_damage = 11;
    c.aos_speed = 55;
    c.ml_speed = 2.0;
    c.old_str_req = 15;
    c.old_min_damage = 9;
    c.old_max_damage = 11;
    c.old_speed = 55;
    c.def_hit_sound = 572;
    c.def_miss_sound = 562;
    c.init_min_hits = 55;
    c.init_max_hits = 60;
    c.weight = 7.0;
    c.primary_ability = "Block";
    c.secondary_ability = "ArmorPierce";
    c.def_skill = "Fencing";
    c.def_type = "Piercing";
    c.def_animation = "Pierce1H";
    c.graphic = 10159;
    c.on_create(|i| {
        i.layer = "TwoHanded";
    });
});

item("Tekagi", |c| {
    c.aos_str_req = 10;
    c.aos_min_damage = 10;
    c.aos_max_damage = 12;
    c.aos_speed = 53;
    c.ml_speed = 2.0;
    c.old_str_req = 10;
    c.old_min_damage = 10;
    c.old_max_damage = 12;
    c.old_speed = 53;
    c.def_hit_sound = 568;
    c.def_miss_sound = 562;
    c.init_min_hits = 35;
    c.init_max_hits = 60;
    c.weight = 5.0;
    c.primary_ability = "DualWield";
    c.secondary_ability = "TalonStrike";
    c.def_skill = "Fencing";
    c.def_type = "Piercing";
    c.def_animation = "Pierce1H";
    c.graphic = 10155;
    c.on_create(|i| {
        i.layer = "TwoHanded";
    });
});

item("Tessen", |c| {
    c.aos_str_req = 10;
    c.aos_min_damage = 10;
    c.aos_max_damage = 12;
    c.aos_speed = 50;
    c.ml_speed = 2.0;
    c.old_str_req = 10;
    c.old_min_damage = 10;
    c.old_max_damage = 12;
    c.old_speed = 50;
    c.def_hit_sound = 562;
    c.def_miss_sound = 568;
    c.init_min_hits = 55;
    c.init_max_hits = 60;
    c.weight = 6.0;
    c.primary_ability = "Feint";
    c.secondary_ability = "Block";
    c.def_animation = "Bash2H";
    c.graphic = 10147;
    c.on_create(|i| {
        i.layer = "TwoHanded";
    });
});

item("Tetsubo", |c| {
    c.aos_str_req = 35;
    c.aos_min_damage = 12;
    c.aos_max_damage = 14;
    c.aos_speed = 45;
    c.ml_speed = 2.5;
    c.old_str_req = 35;
    c.old_min_damage = 12;
    c.old_max_damage = 14;
    c.old_speed = 45;
    c.def_hit_sound = 563;
    c.def_miss_sound = 568;
    c.init_min_hits = 60;
    c.init_max_hits = 65;
    c.weight = 8.0;
    c.primary_ability = "FrenziedWhirlwind";
    c.secondary_ability = "CrushingBlow";
    c.def_animation = "Bash2H";
    c.graphic = 10150;
    c.on_create(|i| {
        i.layer = "TwoHanded";
    });
});

item("Wakizashi", |c| {
    c.aos_str_req = 20;
    c.aos_min_damage = 11;
    c.aos_max_damage = 13;
    c.aos_speed = 44;
    c.ml_speed = 2.5;
    c.old_str_req = 20;
    c.old_min_damage = 11;
    c.old_max_damage = 13;
    c.old_speed = 44;
    c.def_hit_sound = 571;
    c.def_miss_sound = 570;
    c.init_min_hits = 45;
    c.init_max_hits = 50;
    c.weight = 5.0;
    c.primary_ability = "FrenziedWhirlwind";
    c.secondary_ability = "DoubleStrike";
    c.graphic = 10148;
    c.on_create(|i| {
        i.layer = "OneHanded";
    });
});

item("Yumi", |c| {
    c.aos_str_req = 35;
    c.aos_min_damage = [18, 16];
    c.aos_max_damage = 20;
    c.aos_speed = 25;
    c.ml_speed = 4.5;
    c.old_str_req = 35;
    c.old_min_damage = 18;
    c.old_max_damage = 20;
    c.old_speed = 25;
    c.def_max_range = 10;
    c.init_min_hits = 55;
    c.init_max_hits = 60;
    c.weight = 9.0;
    c.effect_id = 3906;
    c.ammo_type = "Server.Items.Arrow";
    c.primary_ability = "ArmorPierce";
    c.secondary_ability = "DoubleShot";
    c.def_animation = "ShootBow";
    c.graphic = 10149;
    c.on_create(|i| {
        i.layer = "TwoHanded";
        i.resource = "RegularWood";
    });
});
