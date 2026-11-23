item("AssassinSpike", |c| {
    c.aos_str_req = 15;
    c.aos_min_damage = 10;
    c.aos_max_damage = 12;
    c.aos_speed = 50;
    c.ml_speed = 2.0;
    c.old_str_req = 15;
    c.old_min_damage = 10;
    c.old_max_damage = 12;
    c.old_speed = 50;
    c.def_miss_sound = 569;
    c.init_min_hits = 30;
    c.init_max_hits = 60;
    c.weight = 4.0;
    c.primary_ability = "InfectiousStrike";
    c.secondary_ability = "ShadowStrike";
    c.def_skill = "Fencing";
    c.graphic = 11553;
});

item("DiamondMace", |c| {
    c.aos_str_req = 35;
    c.aos_min_damage = 14;
    c.aos_max_damage = 17;
    c.aos_speed = 37;
    c.ml_speed = 3.0;
    c.old_str_req = 35;
    c.old_min_damage = 14;
    c.old_max_damage = 17;
    c.old_speed = 37;
    c.init_min_hits = 30;
    c.init_max_hits = 60;
    c.weight = 10.0;
    c.primary_ability = "ConcussionBlow";
    c.secondary_ability = "CrushingBlow";
    c.graphic = 11556;
});

item("ElvenCompositeLongbow", |c| {
    c.aos_str_req = 45;
    c.aos_min_damage = 12;
    c.aos_max_damage = 16;
    c.aos_speed = 27;
    c.ml_speed = 4.0;
    c.old_str_req = 45;
    c.old_min_damage = 12;
    c.old_max_damage = 16;
    c.old_speed = 27;
    c.def_max_range = 10;
    c.init_min_hits = 41;
    c.init_max_hits = 90;
    c.weight = 8.0;
    c.effect_id = 3906;
    c.ammo_type = "Server.Items.Arrow";
    c.primary_ability = "ForceArrow";
    c.secondary_ability = "SerpentArrow";
    c.def_animation = "ShootBow";
    c.graphic = 11550;
    c.on_create(|i| {
        i.resource = "RegularWood";
    });
});

item("ElvenMachete", |c| {
    c.aos_str_req = 20;
    c.aos_min_damage = 13;
    c.aos_max_damage = 15;
    c.aos_speed = 41;
    c.ml_speed = 2.75;
    c.old_str_req = 20;
    c.old_min_damage = 13;
    c.old_max_damage = 15;
    c.old_speed = 41;
    c.def_hit_sound = 571;
    c.def_miss_sound = 569;
    c.init_min_hits = 30;
    c.init_max_hits = 60;
    c.weight = 6.0;
    c.primary_ability = "DefenseMastery";
    c.secondary_ability = "Bladeweave";
    c.graphic = 11573;
});

item("ElvenSpellblade", |c| {
    c.aos_str_req = 35;
    c.aos_min_damage = 12;
    c.aos_max_damage = 14;
    c.aos_speed = 44;
    c.ml_speed = 2.5;
    c.old_str_req = 35;
    c.old_min_damage = 12;
    c.old_max_damage = 14;
    c.old_speed = 44;
    c.def_miss_sound = 569;
    c.init_min_hits = 30;
    c.init_max_hits = 60;
    c.weight = 5.0;
    c.primary_ability = "PsychicAttack";
    c.secondary_ability = "BleedAttack";
    c.graphic = 11552;
    c.on_create(|i| {
        i.layer = "TwoHanded";
    });
});

item("Leafblade", |c| {
    c.aos_str_req = 20;
    c.aos_min_damage = 13;
    c.aos_max_damage = 15;
    c.aos_speed = 42;
    c.ml_speed = 2.75;
    c.old_str_req = 20;
    c.old_min_damage = 13;
    c.old_max_damage = 15;
    c.old_speed = 42;
    c.def_miss_sound = 569;
    c.init_min_hits = 30;
    c.init_max_hits = 60;
    c.weight = 8.0;
    c.primary_ability = "Feint";
    c.secondary_ability = "ArmorIgnore";
    c.def_skill = "Fencing";
    c.graphic = 11554;
});

item("MagicalShortbow", |c| {
    c.aos_str_req = 45;
    c.aos_min_damage = 9;
    c.aos_max_damage = 13;
    c.aos_speed = 38;
    c.ml_speed = 3.0;
    c.old_str_req = 45;
    c.old_min_damage = 9;
    c.old_max_damage = 13;
    c.old_speed = 38;
    c.def_max_range = 10;
    c.init_min_hits = 41;
    c.init_max_hits = 90;
    c.weight = 6.0;
    c.effect_id = 3906;
    c.ammo_type = "Server.Items.Arrow";
    c.primary_ability = "LightningArrow";
    c.secondary_ability = "PsychicAttack";
    c.graphic = 11563;
    c.on_create(|i| {
        i.resource = "RegularWood";
    });
});

item("OrnateAxe", |c| {
    c.aos_str_req = 45;
    c.aos_min_damage = 18;
    c.aos_max_damage = 20;
    c.aos_speed = 26;
    c.ml_speed = 3.5;
    c.old_str_req = 45;
    c.old_min_damage = 18;
    c.old_max_damage = 20;
    c.old_speed = 26;
    c.def_miss_sound = 569;
    c.init_min_hits = 30;
    c.init_max_hits = 60;
    c.weight = 12.0;
    c.primary_ability = "Disarm";
    c.secondary_ability = "CrushingBlow";
    c.graphic = 11560;
    c.on_create(|i| {
        i.layer = "TwoHanded";
    });
});

item("RadiantScimitar", |c| {
    c.aos_str_req = 20;
    c.aos_min_damage = 12;
    c.aos_max_damage = 14;
    c.aos_speed = 43;
    c.ml_speed = 2.5;
    c.old_str_req = 20;
    c.old_min_damage = 12;
    c.old_max_damage = 14;
    c.old_speed = 43;
    c.def_hit_sound = 571;
    c.def_miss_sound = 569;
    c.init_min_hits = 30;
    c.init_max_hits = 60;
    c.weight = 9.0;
    c.primary_ability = "WhirlwindAttack";
    c.secondary_ability = "Bladeweave";
    c.graphic = 11571;
});

item("RuneBlade", |c| {
    c.aos_str_req = 30;
    c.aos_min_damage = 15;
    c.aos_max_damage = 17;
    c.aos_speed = 35;
    c.ml_speed = 3.0;
    c.old_str_req = 30;
    c.old_min_damage = 15;
    c.old_max_damage = 17;
    c.old_speed = 35;
    c.def_hit_sound = 571;
    c.def_miss_sound = 569;
    c.init_min_hits = 30;
    c.init_max_hits = 60;
    c.weight = 7.0;
    c.primary_ability = "Disarm";
    c.secondary_ability = "Bladeweave";
    c.graphic = 11570;
    c.on_create(|i| {
        i.layer = "TwoHanded";
    });
});

item("WarCleaver", |c| {
    c.aos_str_req = 15;
    c.aos_min_damage = 9;
    c.aos_max_damage = 11;
    c.aos_speed = 48;
    c.ml_speed = 2.25;
    c.old_str_req = 15;
    c.old_min_damage = 9;
    c.old_max_damage = 11;
    c.old_speed = 48;
    c.def_hit_sound = 571;
    c.def_miss_sound = 569;
    c.init_min_hits = 30;
    c.init_max_hits = 60;
    c.weight = 10.0;
    c.primary_ability = "Disarm";
    c.secondary_ability = "Bladeweave";
    c.def_skill = "Fencing";
    c.def_type = "Piercing";
    c.def_animation = "Pierce1H";
    c.graphic = 11567;
});

item("WildStaff", |c| {
    c.aos_str_req = 15;
    c.aos_min_damage = 10;
    c.aos_max_damage = 12;
    c.aos_speed = 48;
    c.ml_speed = 2.25;
    c.old_str_req = 15;
    c.old_min_damage = 10;
    c.old_max_damage = 12;
    c.old_speed = 48;
    c.init_min_hits = 30;
    c.init_max_hits = 60;
    c.weight = 8.0;
    c.primary_ability = "Block";
    c.secondary_ability = "ForceOfNature";
    c.graphic = 11557;
    c.on_create(|i| {
        i.resource = "RegularWood";
    });
});

item("ButchersWarCleaver", |c| {
    c.label = 1073526;
});
