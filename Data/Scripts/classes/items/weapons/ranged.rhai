item("CompositeBow", |c| {
    c.aos_str_req = 45;
    c.aos_min_damage = [15, 13];
    c.aos_max_damage = 17;
    c.aos_speed = 25;
    c.ml_speed = 4.0;
    c.old_str_req = 45;
    c.old_min_damage = 15;
    c.old_max_damage = 17;
    c.old_speed = 25;
    c.def_max_range = 10;
    c.init_min_hits = 31;
    c.init_max_hits = 70;
    c.weight = 5.0;
    c.effect_id = 3906;
    c.ammo_type = "Server.Items.Arrow";
    c.primary_ability = "ArmorIgnore";
    c.secondary_ability = "MovingShot";
    c.def_animation = "ShootBow";
    c.graphic = 9922;
    c.on_create(|i| {
        i.resource = "RegularWood";
    });
});

item("Crossbow", |c| {
    c.aos_str_req = 35;
    c.aos_min_damage = 18;
    c.aos_max_damage = [20, 22];
    c.aos_speed = 24;
    c.ml_speed = 4.5;
    c.old_str_req = 30;
    c.old_min_damage = 8;
    c.old_max_damage = 43;
    c.old_speed = 18;
    c.def_max_range = 8;
    c.init_min_hits = 31;
    c.init_max_hits = 80;
    c.weight = 7.0;
    c.effect_id = 7166;
    c.ammo_type = "Server.Items.Bolt";
    c.primary_ability = "ConcussionBlow";
    c.secondary_ability = "MortalStrike";
    c.graphic = 3920;
    c.on_create(|i| {
        i.layer = "TwoHanded";
        i.resource = "RegularWood";
    });
});

item("HeavyCrossbow", |c| {
    c.aos_str_req = 80;
    c.aos_min_damage = [19, 20];
    c.aos_max_damage = [20, 24];
    c.aos_speed = 22;
    c.ml_speed = 5.0;
    c.old_str_req = 40;
    c.old_min_damage = 11;
    c.old_max_damage = 56;
    c.old_speed = 10;
    c.def_max_range = 8;
    c.init_min_hits = 31;
    c.init_max_hits = 100;
    c.weight = 9.0;
    c.effect_id = 7166;
    c.ammo_type = "Server.Items.Bolt";
    c.primary_ability = "MovingShot";
    c.secondary_ability = "Dismount";
    c.graphic = 5117;
    c.on_create(|i| {
        i.layer = "TwoHanded";
        i.resource = "RegularWood";
    });
});

item("JukaBow", |c| {
    c.aos_str_req = 80;
    c.aos_dex_req = 80;
    c.old_str_req = 80;
    c.old_dex_req = 80;
    c.on_create(|i| {
        i.resource = "RegularWood";
    });
});

item("RepeatingCrossbow", |c| {
    c.aos_str_req = 30;
    c.aos_min_damage = [10, 8];
    c.aos_max_damage = 12;
    c.aos_speed = 41;
    c.ml_speed = 2.75;
    c.old_str_req = 30;
    c.old_min_damage = 10;
    c.old_max_damage = 12;
    c.old_speed = 41;
    c.def_max_range = 7;
    c.init_min_hits = 31;
    c.init_max_hits = 80;
    c.weight = 6.0;
    c.effect_id = 7166;
    c.ammo_type = "Server.Items.Bolt";
    c.primary_ability = "DoubleStrike";
    c.secondary_ability = "MovingShot";
    c.graphic = 9923;
    c.on_create(|i| {
        i.resource = "RegularWood";
    });
});

item("AssassinsShortbow", |c| {
    c.label = 1073512;
    c.on_create(|i| {
        i.attributes.attack_chance = 3;
        i.attributes.weapon_damage = 4;
    });
});

item("BarbedLongbow", |c| {
    c.label = 1073505;
    c.on_create(|i| {
        i.attributes.reflect_physical = 12;
    });
});

item("FrozenLongbow", |c| {
    c.label = 1073507;
    c.on_create(|i| {
        i.attributes.weapon_speed = -5;
        i.attributes.defend_chance = 10;
    });
});

item("LightweightShortbow", |c| {
    c.label = 1073510;
    c.on_create(|i| {
        i.balanced = true;
    });
});

item("LongbowOfMight", |c| {
    c.label = 1073508;
    c.on_create(|i| {
        i.attributes.weapon_damage = 5;
    });
});

item("MysticalShortbow", |c| {
    c.label = 1073511;
    c.on_create(|i| {
        i.attributes.spell_channeling = 1;
        i.attributes.cast_speed = -1;
    });
});

item("RangersShortbow", |c| {
    c.label = 1073509;
    c.on_create(|i| {
        i.attributes.weapon_speed = 5;
    });
});

item("SlayerLongbow", |c| {
    c.script = "content/equipment/slayer_longbow.rhai";
    c.label = 1073506;
});
