item("Longsword", |c| {
    c.aos_str_req = 35;
    c.aos_min_damage = 15;
    c.aos_max_damage = 16;
    c.aos_speed = 30;
    c.ml_speed = 3.5;
    c.old_str_req = 25;
    c.old_min_damage = 5;
    c.old_max_damage = 33;
    c.old_speed = 35;
    c.def_hit_sound = 567;
    c.def_miss_sound = 570;
    c.init_min_hits = 31;
    c.init_max_hits = 110;
    c.weight = 7.0;
    c.primary_ability = "ArmorIgnore";
    c.secondary_ability = "ConcussionBlow";
    c.graphic = 3937;
});

item("Katana", |c| {
    c.aos_str_req = 25;
    c.aos_min_damage = 11;
    c.aos_max_damage = 13;
    c.aos_speed = 46;
    c.ml_speed = 2.5;
    c.old_str_req = 10;
    c.old_min_damage = 5;
    c.old_max_damage = 26;
    c.old_speed = 58;
    c.def_hit_sound = 571;
    c.def_miss_sound = 570;
    c.init_min_hits = 31;
    c.init_max_hits = 90;
    c.weight = 6.0;
    c.primary_ability = "DoubleStrike";
    c.secondary_ability = "ArmorIgnore";
    c.graphic = 5119;
});

item("Bow", |c| {
    c.aos_str_req = 30;
    c.aos_min_damage = [16, 15];
    c.aos_max_damage = [18, 19];
    c.aos_speed = 25;
    c.ml_speed = 4.25;
    c.old_str_req = 20;
    c.old_min_damage = 9;
    c.old_max_damage = 41;
    c.old_speed = 20;
    c.def_max_range = 10;
    c.init_min_hits = 31;
    c.init_max_hits = 60;
    c.weight = 6.0;
    c.effect_id = 3906;
    c.ammo_type = "Server.Items.Arrow";
    c.primary_ability = "ParalyzingBlow";
    c.secondary_ability = "MortalStrike";
    c.def_animation = "ShootBow";
    c.graphic = 5042;
    c.on_create(|i| {
        i.layer = "TwoHanded";
        i.resource = "RegularWood";
    });
});

item("PlateChest", |c| {
    c.base_physical_resistance = 5;
    c.base_fire_resistance = 3;
    c.base_cold_resistance = 2;
    c.base_poison_resistance = 3;
    c.base_energy_resistance = 2;
    c.init_min_hits = 50;
    c.init_max_hits = 65;
    c.aos_str_req = 95;
    c.old_str_req = 60;
    c.old_dex_bonus = -8;
    c.armor_base = 40;
    c.weight = 10.0;
    c.material_type = "Plate";
    c.graphic = 5141;
});

item("LeatherChest", |c| {
    c.base_physical_resistance = 2;
    c.base_fire_resistance = 4;
    c.base_cold_resistance = 3;
    c.base_poison_resistance = 3;
    c.base_energy_resistance = 3;
    c.init_min_hits = 30;
    c.init_max_hits = 40;
    c.aos_str_req = 25;
    c.old_str_req = 15;
    c.armor_base = 13;
    c.weight = 6.0;
    c.material_type = "Leather";
    c.default_resource = "RegularLeather";
    c.def_med_allowance = "All";
    c.graphic = 5068;
});

item("ChainChest", |c| {
    c.base_physical_resistance = 4;
    c.base_fire_resistance = 4;
    c.base_cold_resistance = 4;
    c.base_poison_resistance = 1;
    c.base_energy_resistance = 2;
    c.init_min_hits = 45;
    c.init_max_hits = 60;
    c.aos_str_req = 60;
    c.old_str_req = 20;
    c.old_dex_bonus = -5;
    c.armor_base = 28;
    c.weight = 7.0;
    c.material_type = "Chainmail";
    c.graphic = 5055;
});

item("Cap", |c| {
    c.weight = 1.0;
    c.base_physical_resistance = 0;
    c.base_fire_resistance = 5;
    c.base_cold_resistance = 9;
    c.base_poison_resistance = 5;
    c.base_energy_resistance = 5;
    c.init_min_hits = 20;
    c.init_max_hits = 30;
    c.graphic = 5909;
});

item("FancyShirt", |c| {
    c.weight = 2.0;
    c.graphic = 7933;
});

item("GoldRing", |c| {
    c.weight = 0.1;
    c.graphic = 4234;
});

item("Necklace", |c| {
    c.weight = 0.1;
    c.graphic = 4229;
});
