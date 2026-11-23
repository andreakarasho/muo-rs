item("Axe", |c| {
    c.aos_str_req = 35;
    c.aos_min_damage = 14;
    c.aos_max_damage = 16;
    c.aos_speed = 37;
    c.ml_speed = 3.0;
    c.old_str_req = 35;
    c.old_min_damage = 6;
    c.old_max_damage = 33;
    c.old_speed = 37;
    c.init_min_hits = 31;
    c.init_max_hits = 110;
    c.weight = 4.0;
    c.primary_ability = "CrushingBlow";
    c.secondary_ability = "Dismount";
    c.graphic = 3913;
});

item("BattleAxe", |c| {
    c.aos_str_req = 35;
    c.aos_min_damage = 15;
    c.aos_max_damage = 17;
    c.aos_speed = 31;
    c.ml_speed = 3.5;
    c.old_str_req = 40;
    c.old_min_damage = 6;
    c.old_max_damage = 38;
    c.old_speed = 30;
    c.init_min_hits = 31;
    c.init_max_hits = 70;
    c.weight = 4.0;
    c.primary_ability = "BleedAttack";
    c.secondary_ability = "ConcussionBlow";
    c.graphic = 3911;
    c.on_create(|i| {
        i.layer = "TwoHanded";
    });
});

item("DoubleAxe", |c| {
    c.aos_str_req = 45;
    c.aos_min_damage = 15;
    c.aos_max_damage = 17;
    c.aos_speed = 33;
    c.ml_speed = 3.25;
    c.old_str_req = 45;
    c.old_min_damage = 5;
    c.old_max_damage = 35;
    c.old_speed = 37;
    c.init_min_hits = 31;
    c.init_max_hits = 110;
    c.weight = 8.0;
    c.primary_ability = "DoubleStrike";
    c.secondary_ability = "WhirlwindAttack";
    c.graphic = 3915;
});

item("DualShortAxes", |c| {
    c.aos_str_req = 35;
    c.aos_min_damage = 14;
    c.aos_max_damage = 17;
    c.ml_speed = 3.0;
    c.init_min_hits = 31;
    c.init_max_hits = 110;
    c.primary_ability = "DoubleStrike";
    c.secondary_ability = "InfectiousStrike";
    c.required_races = "AllowGargoylesOnly";
    c.graphic = 2301;
});

item("ExecutionersAxe", |c| {
    c.aos_str_req = 40;
    c.aos_min_damage = 15;
    c.aos_max_damage = 17;
    c.aos_speed = 33;
    c.ml_speed = 3.25;
    c.old_str_req = 35;
    c.old_min_damage = 6;
    c.old_max_damage = 33;
    c.old_speed = 37;
    c.init_min_hits = 31;
    c.init_max_hits = 70;
    c.weight = 8.0;
    c.primary_ability = "BleedAttack";
    c.secondary_ability = "MortalStrike";
    c.graphic = 3909;
});

item("Hatchet", |c| {
    c.aos_str_req = 20;
    c.aos_min_damage = 13;
    c.aos_max_damage = 15;
    c.aos_speed = 41;
    c.ml_speed = 2.75;
    c.old_str_req = 15;
    c.old_min_damage = 2;
    c.old_max_damage = 17;
    c.old_speed = 40;
    c.init_min_hits = 31;
    c.init_max_hits = 80;
    c.weight = 4.0;
    c.primary_ability = "ArmorIgnore";
    c.secondary_ability = "Disarm";
    c.graphic = 3907;
});

item("LargeBattleAxe", |c| {
    c.aos_str_req = 80;
    c.aos_min_damage = 16;
    c.aos_max_damage = 17;
    c.aos_speed = 29;
    c.ml_speed = 3.75;
    c.old_str_req = 40;
    c.old_min_damage = 6;
    c.old_max_damage = 38;
    c.old_speed = 30;
    c.init_min_hits = 31;
    c.init_max_hits = 70;
    c.weight = 6.0;
    c.primary_ability = "WhirlwindAttack";
    c.secondary_ability = "BleedAttack";
    c.graphic = 5115;
});

item("Pickaxe", |c| {
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
    c.weight = 11.0;
    c.harvest_system = "Mining";
    c.primary_ability = "DoubleStrike";
    c.secondary_ability = "Disarm";
    c.def_animation = "Slash1H";
    c.graphic = 3718;
    c.on_create(|i| {
        i.uses_remaining = 50;
        i.weapon_show_uses_remaining = true;
    });
});

item("TwoHandedAxe", |c| {
    c.aos_str_req = 40;
    c.aos_min_damage = 16;
    c.aos_max_damage = 17;
    c.aos_speed = 31;
    c.ml_speed = 3.5;
    c.old_str_req = 35;
    c.old_min_damage = 5;
    c.old_max_damage = 39;
    c.old_speed = 30;
    c.init_min_hits = 31;
    c.init_max_hits = 90;
    c.weight = 8.0;
    c.primary_ability = "DoubleStrike";
    c.secondary_ability = "ShadowStrike";
    c.graphic = 5187;
});

item("WarAxe", |c| {
    c.aos_str_req = 35;
    c.aos_min_damage = 14;
    c.aos_max_damage = 15;
    c.aos_speed = 33;
    c.ml_speed = 3.25;
    c.old_str_req = 35;
    c.old_min_damage = 9;
    c.old_max_damage = 27;
    c.old_speed = 40;
    c.def_hit_sound = 563;
    c.def_miss_sound = 569;
    c.init_min_hits = 31;
    c.init_max_hits = 80;
    c.weight = 8.0;
    c.primary_ability = "ArmorIgnore";
    c.secondary_ability = "BleedAttack";
    c.def_skill = "Macing";
    c.def_type = "Bashing";
    c.def_animation = "Bash1H";
    c.harvest_system = ();
    c.graphic = 5040;
});

item("GuardianAxe", |c| {
    c.label = 1073545;
    c.on_create(|i| {
        i.attributes.bonus_hits = 4;
        i.attributes.regen_hits = 1;
    });
});

item("HeavyOrnateAxe", |c| {
    c.label = 1073548;
    c.on_create(|i| {
        i.attributes.weapon_damage = 8;
    });
});

item("SingingAxe", |c| {
    c.label = 1073546;
    c.on_create(|i| {
        i.skill_bonuses = [["Musicianship", 5.0]];
    });
});

item("ThunderingAxe", |c| {
    c.label = 1073547;
    c.on_create(|i| {
        i.weapon_attributes.hit_lightning = 10;
    });
});
