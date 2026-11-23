item("ThrowingDagger", |c| {
    c.weight = 1.0;
    c.name = "a throwing dagger";
    c.on_create(|i| {
        i.item_id = 0xF52;
        i.layer = "OneHanded";
    });
});

item("Fists", |c| {
    c.primary_ability = "Disarm";
    c.secondary_ability = "ParalyzingBlow";
    c.aos_str_req = 0;
    c.aos_min_damage = 1;
    c.aos_max_damage = 4;
    c.aos_speed = 50;
    c.ml_speed = 2.5;
    c.old_str_req = 0;
    c.old_min_damage = 1;
    c.old_max_damage = 8;
    c.old_speed = 30;
    c.def_hit_sound = -1;
    c.def_miss_sound = -1;
    c.def_skill = "Wrestling";
    c.def_type = "Fists";
    c.def_animation = "Wrestle";
    c.graphic = 0;
    c.on_create(|i| {
        i.visible = false;
        i.movable = false;
    });
});

item("FireworksWand", |c| {
    c.on_create(|i| {
        i.weapon_charges = 100;
        i.loot_type = "Blessed";
    });
});
