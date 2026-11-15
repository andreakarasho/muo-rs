item("AxeOfTheHeavens", |c| {
    c.init_min_hits = 255;
    c.init_max_hits = 255;
    c.label = 1061106;
    c.rarity = 11;
    c.on_create(|i| {
        i.hue = 0x4D5;
        i.weapon_attributes.hit_lightning = 50;
        i.attributes.attack_chance = 15;
        i.attributes.defend_chance = 15;
        i.attributes.weapon_damage = 50;
    });
});

item("BladeOfInsanity", |c| {
    c.init_min_hits = 255;
    c.init_max_hits = 255;
    c.label = 1061088;
    c.rarity = 11;
    c.on_create(|i| {
        i.hue = 0x76D;
        i.weapon_attributes.hit_leech_stam = 100;
        i.attributes.regen_stam = 2;
        i.attributes.weapon_speed = 30;
        i.attributes.weapon_damage = 50;
    });
});

item("BladeOfTheRighteous", |c| {
    c.init_min_hits = 255;
    c.init_max_hits = 255;
    c.label = 1061107;
    c.rarity = 10;
    c.on_create(|i| {
        i.hue = 0x47E;
        i.slayer = "Exorcism";
        i.weapon_attributes.hit_leech_hits = 50;
        i.weapon_attributes.use_best_skill = 1;
        i.attributes.bonus_hits = 10;
        i.attributes.weapon_damage = 50;
    });
});

item("BoneCrusher", |c| {
    c.init_min_hits = 255;
    c.init_max_hits = 255;
    c.label = 1061596;
    c.rarity = 11;
    c.on_create(|i| {
        i.item_id = 0x1406;
        i.hue = 0x60C;
        i.weapon_attributes.hit_lower_defend = 50;
        i.attributes.bonus_str = 10;
        i.attributes.weapon_damage = 75;
    });
});

item("BreathOfTheDead", |c| {
    c.init_min_hits = 255;
    c.init_max_hits = 255;
    c.label = 1061109;
    c.rarity = 11;
    c.on_create(|i| {
        i.hue = 0x455;
        i.weapon_attributes.hit_leech_hits = 100;
        i.weapon_attributes.hit_harm = 25;
        i.attributes.spell_damage = 5;
        i.attributes.weapon_damage = 50;
    });
});

item("Frostbringer", |c| {
    c.init_min_hits = 255;
    c.init_max_hits = 255;
    c.label = 1061111;
    c.rarity = 11;
    c.damage_types = #{ cold: 100 };
    c.on_create(|i| {
        i.hue = 0x4F2;
        i.weapon_attributes.hit_dispel = 50;
        i.attributes.regen_stam = 10;
        i.attributes.weapon_damage = 50;
    });
});

item("LegacyOfTheDreadLord", |c| {
    c.init_min_hits = 255;
    c.init_max_hits = 255;
    c.label = 1060860;
    c.rarity = 10;
    c.on_create(|i| {
        i.hue = 0x676;
        i.attributes.spell_channeling = 1;
        i.attributes.cast_recovery = 3;
        i.attributes.weapon_speed = 30;
        i.attributes.weapon_damage = 50;
    });
});

item("SerpentsFang", |c| {
    c.init_min_hits = 255;
    c.init_max_hits = 255;
    c.label = 1061601;
    c.rarity = 11;
    c.damage_types = #{ phys: 25, pois: 75 };
    c.on_create(|i| {
        i.item_id = 0x1400;
        i.hue = 0x488;
        i.weapon_attributes.hit_poison_area = 100;
        i.weapon_attributes.resist_poison_bonus = 20;
        i.attributes.attack_chance = 15;
        i.attributes.weapon_damage = 50;
    });
});

item("StaffOfTheMagi", |c| {
    c.init_min_hits = 255;
    c.init_max_hits = 255;
    c.label = 1061600;
    c.rarity = 11;
    c.damage_types = #{ nrgy: 100 };
    c.on_create(|i| {
        i.hue = 0x481;
        i.weapon_attributes.mage_weapon = 30;
        i.attributes.spell_channeling = 1;
        i.attributes.cast_speed = 1;
        i.attributes.weapon_damage = 50;
    });
});

item("TheBeserkersMaul", |c| {
    c.init_min_hits = 255;
    c.init_max_hits = 255;
    c.label = 1061108;
    c.rarity = 11;
    c.on_create(|i| {
        i.hue = 0x21;
        i.attributes.weapon_speed = 75;
        i.attributes.weapon_damage = 50;
    });
});

item("TheDragonSlayer", |c| {
    c.init_min_hits = 255;
    c.init_max_hits = 255;
    c.label = 1061248;
    c.rarity = 11;
    c.damage_types = #{ nrgy: 100 };
    c.on_create(|i| {
        i.hue = 0x530;
        i.slayer = "DragonSlaying";
        i.attributes.luck = 110;
        i.attributes.weapon_damage = 50;
        i.weapon_attributes.resist_fire_bonus = 20;
        i.weapon_attributes.use_best_skill = 1;
    });
});

item("TheDryadBow", |c| {
    c.script = "content/equipment/the_dryad_bow.rhai";
    c.init_min_hits = 255;
    c.init_max_hits = 255;
    c.label = 1061090;
    c.rarity = 11;
    c.on_create(|i| {
        i.item_id = 0x13B1;
        i.hue = 0x48F;
        i.weapon_attributes.self_repair = 5;
        i.weapon_attributes.resist_poison_bonus = 15;
        i.attributes.weapon_speed = 50;
        i.attributes.weapon_damage = 35;
    });
});

item("TheTaskmaster", |c| {
    c.init_min_hits = 255;
    c.init_max_hits = 255;
    c.label = 1061110;
    c.rarity = 10;
    c.damage_types = #{ pois: 100 };
    c.on_create(|i| {
        i.hue = 0x4F8;
        i.weapon_attributes.hit_poison_area = 100;
        i.attributes.bonus_dex = 5;
        i.attributes.attack_chance = 15;
        i.attributes.weapon_damage = 50;
    });
});

item("TitansHammer", |c| {
    c.init_min_hits = 255;
    c.init_max_hits = 255;
    c.label = 1060024;
    c.rarity = 10;
    c.on_create(|i| {
        i.hue = 0x482;
        i.weapon_attributes.hit_energy_area = 100;
        i.attributes.bonus_str = 15;
        i.attributes.attack_chance = 15;
        i.attributes.weapon_damage = 50;
    });
});

item("ZyronicClaw", |c| {
    c.init_min_hits = 255;
    c.init_max_hits = 255;
    c.label = 1061593;
    c.rarity = 10;
    c.damage_types = #{ phys: 20, fire: 20, cold: 20, pois: 20, nrgy: 20 };
    c.on_create(|i| {
        i.hue = 0x485;
        i.slayer = "ElementalBan";
        i.weapon_attributes.hit_leech_mana = 50;
        i.attributes.attack_chance = 30;
        i.attributes.weapon_damage = 50;
    });
});

item("BlightGrippedLongbow", |c| {
    c.label = 1072907;
    c.on_create(|i| {
        i.hue = 0x8A4;
        i.weapon_attributes.hit_poison_area = 20;
        i.attributes.regen_stam = 3;
        i.attributes.night_sight = 1;
        i.attributes.weapon_speed = 20;
        i.attributes.weapon_damage = 35;
    });
});

item("ColdForgedBlade", |c| {
    c.label = 1072916;
    c.damage_types = #{ cold: 100 };
    c.on_create(|i| {
        i.weapon_attributes.hit_harm = 40;
        i.attributes.spell_channeling = 1;
        i.attributes.night_sight = 1;
        i.attributes.weapon_speed = 25;
        i.attributes.weapon_damage = 50;
        i.elemental_damage_hue = true;
    });
});

item("FaerieFire", |c| {
    c.label = 1072908;
    c.damage_types = #{ fire: 100 };
    c.on_create(|i| {
        i.hue = 0x489;
        i.weapon_attributes.hit_fireball = 25;
        i.balanced = true;
        i.attributes.bonus_dex = 3;
        i.attributes.weapon_speed = 20;
        i.attributes.weapon_damage = 60;
    });
});

item("LuminousRuneBlade", |c| {
    c.label = 1072922;
    c.damage_types = #{ nrgy: 100 };
    c.on_create(|i| {
        i.weapon_attributes.hit_lightning = 40;
        i.weapon_attributes.self_repair = 5;
        i.attributes.night_sight = 1;
        i.attributes.weapon_speed = 25;
        i.attributes.weapon_damage = 55;
        i.elemental_damage_hue = true;
    });
});

item("MischiefMaker", |c| {
    c.label = 1072910;
    c.damage_types = #{ cold: 100 };
    c.on_create(|i| {
        i.hue = 0x8AB;
        i.slayer = "Silver";
        i.balanced = true;
        i.attributes.weapon_speed = 35;
        i.attributes.weapon_damage = 45;
    });
});

item("OverseerSunderedBlade", |c| {
    c.label = 1072920;
    c.damage_types = #{ fire: 100 };
    c.on_create(|i| {
        i.item_id = 0x2D27;
        i.hue = 0x485;
        i.attributes.regen_stam = 2;
        i.attributes.attack_chance = 10;
        i.attributes.weapon_speed = 35;
        i.attributes.weapon_damage = 45;
        i.elemental_damage_hue = true;
    });
});

item("PhantomStaff", |c| {
    c.label = 1072919;
    c.damage_types = #{ cold: 50, pois: 50 };
    c.on_create(|i| {
        i.hue = 1;
        i.attributes.regen_hits = 2;
        i.attributes.night_sight = 1;
        i.attributes.weapon_speed = 20;
        i.attributes.weapon_damage = 60;
    });
});

item("RuneCarvingKnife", |c| {
    c.label = 1072915;
    c.on_create(|i| {
        i.hue = 0x48D;
        i.weapon_attributes.hit_leech_mana = 40;
        i.attributes.regen_stam = 2;
        i.attributes.lower_mana_cost = 10;
        i.attributes.weapon_speed = 35;
        i.attributes.weapon_damage = 30;
    });
});

item("ShardThrasher", |c| {
    c.label = 1072918;
    c.on_create(|i| {
        i.hue = 0x4F2;
        i.weapon_attributes.hit_physical_area = 30;
        i.attributes.bonus_stam = 8;
        i.attributes.attack_chance = 10;
        i.attributes.weapon_speed = 35;
        i.attributes.weapon_damage = 40;
    });
});

item("SilvanisFeywoodBow", |c| {
    c.label = 1072955;
    c.damage_types = #{ nrgy: 100 };
    c.on_create(|i| {
        i.hue = 0x1A;
        i.attributes.spell_channeling = 1;
        i.attributes.attack_chance = 12;
        i.attributes.weapon_speed = 30;
        i.attributes.weapon_damage = 35;
    });
});

item("TheNightReaper", |c| {
    c.label = 1072912;
    c.on_create(|i| {
        i.item_id = 0x26CD;
        i.hue = 0x41C;
        i.slayer = "Exorcism";
        i.attributes.night_sight = 1;
        i.attributes.weapon_speed = 25;
        i.attributes.weapon_damage = 55;
    });
});
