item("BansheesCall", |c| {
    c.init_min_hits = 255;
    c.init_max_hits = 255;
    c.label = 1113529;
    c.on_create(|i| {
        i.hue = 0x4F2;
        i.weapon_attributes.hit_harm = 40;
        i.weapon_attributes.hit_leech_hits = 45;
        i.attributes.bonus_str = 5;
        i.attributes.weapon_speed = 30;
        i.attributes.weapon_damage = 50;
        i.velocity = 35;
        i.element_attributes.cold = 100;
    });
});

item("Boomerang", |c| {
    c.aos_str_req = 25;
    c.aos_min_damage = 11;
    c.aos_max_damage = 15;
    c.ml_speed = 2.75;
    c.init_min_hits = 31;
    c.init_max_hits = 60;
    c.min_throw_range = 4;
    c.primary_ability = "MysticArc";
    c.secondary_ability = "ConcussionBlow";
    c.required_races = "AllowGargoylesOnly";
    c.weight = 4.0;
    c.graphic = 2303;
    c.on_create(|i| {
        i.layer = "OneHanded";
    });
});

item("Cyclone", |c| {
    c.aos_str_req = 40;
    c.aos_min_damage = 13;
    c.aos_max_damage = 17;
    c.ml_speed = 3.25;
    c.init_min_hits = 31;
    c.init_max_hits = 60;
    c.min_throw_range = 6;
    c.primary_ability = "MovingShot";
    c.secondary_ability = "InfusedThrow";
    c.required_races = "AllowGargoylesOnly";
    c.weight = 6.0;
    c.graphic = 2305;
    c.on_create(|i| {
        i.layer = "OneHanded";
    });
});

item("RaptorClaw", |c| {
    c.init_min_hits = 255;
    c.init_max_hits = 255;
    c.label = 1112394;
    c.on_create(|i| {
        i.hue = 0x35;
        i.attributes.attack_chance = 12;
        i.attributes.weapon_speed = 30;
        i.attributes.weapon_damage = 35;
        i.slayer = "Silver";
        i.weapon_attributes.hit_leech_stam = 40;
    });
});

item("SoulGlaive", |c| {
    c.aos_str_req = 60;
    c.aos_min_damage = 16;
    c.aos_max_damage = 20;
    c.ml_speed = 4.0;
    c.init_min_hits = 31;
    c.init_max_hits = 65;
    c.min_throw_range = 8;
    c.primary_ability = "ArmorIgnore";
    c.secondary_ability = "MortalStrike";
    c.required_races = "AllowGargoylesOnly";
    c.weight = 8.0;
    c.graphic = 2314;
    c.on_create(|i| {
        i.layer = "OneHanded";
    });
});

item("StoneSlithClaw", |c| {
    c.init_min_hits = 255;
    c.init_max_hits = 255;
    c.label = 1112393;
    c.on_create(|i| {
        i.hue = 0x47E;
        i.attributes.weapon_speed = 25;
        i.attributes.weapon_damage = 45;
        i.slayer = "DaemonDismissal";
        i.weapon_attributes.hit_harm = 40;
        i.weapon_attributes.hit_lower_defend = 40;
    });
});

item("StormCaller", |c| {
    c.init_min_hits = 255;
    c.init_max_hits = 255;
    c.label = 1113530;
    c.on_create(|i| {
        i.hue = 0x1C8;
        i.weapon_attributes.hit_lightning = 40;
        i.weapon_attributes.hit_lower_defend = 30;
        i.attributes.bonus_str = 5;
        i.attributes.weapon_speed = 30;
        i.attributes.weapon_damage = 40;
        i.element_attributes.physical = 20;
        i.element_attributes.fire = 20;
        i.element_attributes.cold = 20;
        i.element_attributes.poison = 20;
        i.element_attributes.energy = 20;
    });
});

item("ValkyriesGlaive", |c| {
    c.init_min_hits = 255;
    c.init_max_hits = 255;
    c.label = 1113531;
    c.rarity = 5;
    c.on_create(|i| {
        i.hue = 0x673;
        i.attributes.spell_channeling = 1;
        i.attributes.bonus_str = 5;
        i.attributes.weapon_speed = 20;
        i.attributes.weapon_damage = 20;
        i.slayer = "Silver";
        i.weapon_attributes.hit_fireball = 40;
    });
});

item("WindOfCorruption", |c| {
    c.init_min_hits = 255;
    c.init_max_hits = 255;
    c.label = 1150358;
    c.on_create(|i| {
        i.hue = 0x493;
        i.weapon_attributes.hit_leech_stam = 40;
        i.weapon_attributes.hit_lower_defend = 40;
        i.attributes.weapon_speed = 30;
        i.attributes.weapon_damage = 50;
        i.slayer = "Fey";
        i.element_attributes.chaos = 100;
    });
});

item("AbyssReaver", |c| {
    c.script = "content/equipment/abyss_reaver.rhai";
    c.label = 1112694;
});
