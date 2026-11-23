item("Club", |c| {
    c.aos_str_req = 40;
    c.aos_min_damage = 11;
    c.aos_max_damage = 13;
    c.aos_speed = 44;
    c.ml_speed = 2.5;
    c.old_str_req = 10;
    c.old_min_damage = 8;
    c.old_max_damage = 24;
    c.old_speed = 40;
    c.init_min_hits = 31;
    c.init_max_hits = 40;
    c.weight = 9.0;
    c.primary_ability = "ShadowStrike";
    c.secondary_ability = "Dismount";
    c.graphic = 5044;
});

item("DiscMace", |c| {
    c.aos_str_req = 45;
    c.aos_min_damage = 11;
    c.aos_max_damage = 15;
    c.ml_speed = 2.75;
    c.init_min_hits = 31;
    c.init_max_hits = 110;
    c.primary_ability = "ArmorIgnore";
    c.secondary_ability = "Disarm";
    c.required_races = "AllowGargoylesOnly";
    c.graphic = 2307;
});

item("HammerPick", |c| {
    c.aos_str_req = 45;
    c.aos_min_damage = 15;
    c.aos_max_damage = 17;
    c.aos_speed = 28;
    c.ml_speed = 3.75;
    c.old_str_req = 35;
    c.old_min_damage = 6;
    c.old_max_damage = 33;
    c.old_speed = 30;
    c.init_min_hits = 31;
    c.init_max_hits = 70;
    c.weight = 9.0;
    c.primary_ability = "ArmorIgnore";
    c.secondary_ability = "MortalStrike";
    c.graphic = 5181;
    c.on_create(|i| {
        i.layer = "OneHanded";
    });
});

item("Mace", |c| {
    c.aos_str_req = 45;
    c.aos_min_damage = 12;
    c.aos_max_damage = 14;
    c.aos_speed = 40;
    c.ml_speed = 2.75;
    c.old_str_req = 20;
    c.old_min_damage = 8;
    c.old_max_damage = 32;
    c.old_speed = 30;
    c.init_min_hits = 31;
    c.init_max_hits = 70;
    c.weight = 14.0;
    c.primary_ability = "ConcussionBlow";
    c.secondary_ability = "Disarm";
    c.graphic = 3932;
});

item("MagicWand", |c| {
    c.aos_str_req = 5;
    c.aos_min_damage = 9;
    c.aos_max_damage = 11;
    c.aos_speed = 40;
    c.ml_speed = 2.75;
    c.old_str_req = 0;
    c.old_min_damage = 2;
    c.old_max_damage = 6;
    c.old_speed = 35;
    c.init_min_hits = 31;
    c.init_max_hits = 110;
    c.weight = 1.0;
    c.primary_ability = "Dismount";
    c.secondary_ability = "Disarm";
    c.graphic = 3570;
});

item("Maul", |c| {
    c.aos_str_req = 45;
    c.aos_min_damage = 14;
    c.aos_max_damage = 16;
    c.aos_speed = 32;
    c.ml_speed = 3.5;
    c.old_str_req = 20;
    c.old_min_damage = 10;
    c.old_max_damage = 30;
    c.old_speed = 30;
    c.init_min_hits = 31;
    c.init_max_hits = 70;
    c.weight = 10.0;
    c.primary_ability = "CrushingBlow";
    c.secondary_ability = "ConcussionBlow";
    c.graphic = 5179;
});

item("Scepter", |c| {
    c.aos_str_req = 40;
    c.aos_min_damage = 14;
    c.aos_max_damage = 17;
    c.aos_speed = 30;
    c.ml_speed = 3.5;
    c.old_str_req = 40;
    c.old_min_damage = 14;
    c.old_max_damage = 17;
    c.old_speed = 30;
    c.init_min_hits = 31;
    c.init_max_hits = 110;
    c.weight = 8.0;
    c.primary_ability = "CrushingBlow";
    c.secondary_ability = "MortalStrike";
    c.graphic = 9916;
});

item("WarHammer", |c| {
    c.aos_str_req = 95;
    c.aos_min_damage = 17;
    c.aos_max_damage = 18;
    c.aos_speed = 28;
    c.ml_speed = 3.75;
    c.old_str_req = 40;
    c.old_min_damage = 8;
    c.old_max_damage = 36;
    c.old_speed = 31;
    c.init_min_hits = 31;
    c.init_max_hits = 110;
    c.weight = 10.0;
    c.primary_ability = "WhirlwindAttack";
    c.secondary_ability = "CrushingBlow";
    c.def_animation = "Bash2H";
    c.graphic = 5177;
    c.on_create(|i| {
        i.layer = "TwoHanded";
    });
});

item("WarMace", |c| {
    c.aos_str_req = 80;
    c.aos_min_damage = 16;
    c.aos_max_damage = 17;
    c.aos_speed = 26;
    c.ml_speed = 4.0;
    c.old_str_req = 30;
    c.old_min_damage = 10;
    c.old_max_damage = 30;
    c.old_speed = 32;
    c.init_min_hits = 31;
    c.init_max_hits = 110;
    c.weight = 17.0;
    c.primary_ability = "CrushingBlow";
    c.secondary_ability = "MortalStrike";
    c.graphic = 5127;
});

item("EmeraldMace", |c| {
    c.label = 1073530;
    c.on_create(|i| {
        i.weapon_attributes.resist_poison_bonus = 5;
    });
});

item("RubyMace", |c| {
    c.label = 1073529;
    c.on_create(|i| {
        i.attributes.weapon_damage = 5;
    });
});

item("SapphireMace", |c| {
    c.label = 1073531;
    c.on_create(|i| {
        i.weapon_attributes.resist_energy_bonus = 5;
    });
});

item("SilverEtchedMace", |c| {
    c.label = 1073532;
    c.on_create(|i| {
        i.slayer = "Exorcism";
    });
});
