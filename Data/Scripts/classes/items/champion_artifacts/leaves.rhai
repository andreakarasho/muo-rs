item("BraveKnightOfTheBritannia", |c| {
    c.init_min_hits = 150;
    c.init_max_hits = 150;
    c.label = 1094909;
    c.can_fortify = false;
    c.damage_types = #{ fire: 40, cold: 30, pois: 10, nrgy: 20 };
    c.on_create(|i| {
        i.hue = 0x47E;
        i.attributes.weapon_speed = 30;
        i.attributes.weapon_damage = 35;
        i.weapon_attributes.hit_leech_stam = 48;
        i.weapon_attributes.hit_harm = 26;
        i.weapon_attributes.hit_leech_hits = 22;
    });
});

item("GuantletsOfAnger", |c| {
    c.base_physical_resistance = 4;
    c.base_fire_resistance = 4;
    c.base_cold_resistance = 5;
    c.base_poison_resistance = 6;
    c.base_energy_resistance = 5;
    c.init_min_hits = 150;
    c.init_max_hits = 150;
    c.label = 1094902;
    c.can_fortify = false;
    c.on_create(|i| {
        i.hue = 0x29B;
        i.attributes.bonus_hits = 8;
        i.attributes.regen_hits = 2;
        i.attributes.defend_chance = 10;
    });
});

item("OblivionsNeedle", |c| {
    c.init_min_hits = 150;
    c.init_max_hits = 150;
    c.label = 1094916;
    c.can_fortify = false;
    c.on_create(|i| {
        i.attributes.bonus_stam = 20;
        i.attributes.attack_chance = 20;
        i.attributes.defend_chance = -20;
        i.attributes.weapon_damage = 40;
        i.weapon_attributes.hit_leech_stam = 50;
    });
});

item("RoyalGuardSurvivalKnife", |c| {
    c.init_min_hits = 150;
    c.init_max_hits = 150;
    c.label = 1094918;
    c.can_fortify = false;
    c.on_create(|i| {
        i.attributes.spell_channeling = 1;
        i.attributes.luck = 140;
        i.attributes.enhance_potions = 25;
        i.weapon_attributes.use_best_skill = 1;
        i.weapon_attributes.lower_stat_req = 50;
    });
});

item("Calm", |c| {
    c.init_min_hits = 150;
    c.init_max_hits = 150;
    c.label = 1094915;
    c.can_fortify = false;
    c.on_create(|i| {
        i.hue = 0x2CB;
        i.attributes.spell_channeling = 1;
        i.attributes.weapon_speed = 20;
        i.attributes.weapon_damage = 50;
        i.weapon_attributes.hit_leech_mana = 100;
        i.weapon_attributes.use_best_skill = 1;
    });
});

item("FangOfRactus", |c| {
    c.init_min_hits = 150;
    c.init_max_hits = 150;
    c.label = 1094892;
    c.can_fortify = false;
    c.on_create(|i| {
        i.hue = 0x117;
        i.attributes.spell_channeling = 1;
        i.attributes.attack_chance = 5;
        i.attributes.defend_chance = 5;
        i.attributes.weapon_damage = 35;
        i.weapon_attributes.hit_poison_area = 20;
        i.weapon_attributes.resist_poison_bonus = 15;
    });
});

item("GladiatorsCollar", |c| {
    c.base_physical_resistance = 18;
    c.base_fire_resistance = 18;
    c.base_cold_resistance = 17;
    c.base_poison_resistance = 18;
    c.base_energy_resistance = 16;
    c.init_min_hits = 150;
    c.init_max_hits = 150;
    c.label = 1094917;
    c.can_fortify = false;
    c.on_create(|i| {
        i.hue = 0x26D;
        i.attributes.bonus_hits = 10;
        i.attributes.attack_chance = 10;
        i.armor_attributes.mage_armor = 1;
    });
});

item("Pacify", |c| {
    c.init_min_hits = 150;
    c.init_max_hits = 150;
    c.label = 1094929;
    c.can_fortify = false;
    c.on_create(|i| {
        i.hue = 0x835;
        i.attributes.spell_channeling = 1;
        i.attributes.attack_chance = 10;
        i.attributes.weapon_speed = 20;
        i.attributes.weapon_damage = 50;
        i.weapon_attributes.hit_leech_mana = 100;
        i.weapon_attributes.use_best_skill = 1;
    });
});

item("Quell", |c| {
    c.init_min_hits = 150;
    c.init_max_hits = 150;
    c.label = 1094928;
    c.can_fortify = false;
    c.on_create(|i| {
        i.hue = 0x225;
        i.attributes.spell_channeling = 1;
        i.attributes.weapon_speed = 20;
        i.attributes.weapon_damage = 50;
        i.attributes.attack_chance = 10;
        i.weapon_attributes.hit_leech_mana = 100;
        i.weapon_attributes.use_best_skill = 1;
    });
});

item("Subdue", |c| {
    c.init_min_hits = 150;
    c.init_max_hits = 150;
    c.label = 1094930;
    c.can_fortify = false;
    c.on_create(|i| {
        i.hue = 0x2CB;
        i.attributes.spell_channeling = 1;
        i.attributes.weapon_speed = 20;
        i.attributes.weapon_damage = 50;
        i.attributes.attack_chance = 10;
        i.weapon_attributes.hit_leech_mana = 100;
        i.weapon_attributes.use_best_skill = 1;
    });
});

item("ArtifactLargeVase", |c| {
    c.on_create(|i| {
        i.item_id = 0xB47;
    });
});

item("ArtifactVase", |c| {
    c.on_create(|i| {
        i.item_id = 0xB48;
    });
});

item("DirtPatch", |c| {
    c.on_create(|i| {
        i.item_id = 0x913;
    });
});

item("EvilIdolSkull", |c| {
    c.on_create(|i| {
        i.item_id = 0x1F18;
    });
});

item("LavaTile", |c| {
    c.on_create(|i| {
        i.item_id = 0x12EE;
    });
});

item("SkullPole", |c| {
    c.on_create(|i| {
        i.item_id = 0x2204;
    });
});

item("SwampTile", |c| {
    c.on_create(|i| {
        i.item_id = 0x320D;
    });
});

item("TatteredAncientMummyWrapping", |c| {
    c.on_create(|i| {
        i.item_id = 0xE21;
        i.hue = 0x909;
    });
});

item("WaterTile", |c| {
    c.on_create(|i| {
        i.item_id = 0x346E;
    });
});

item("WindSpirit", |c| {
    c.on_create(|i| {
        i.item_id = 0x1F1F;
    });
});

item("CrownOfTalKeesh", |c| {
    c.label = 1094903;
    c.base_physical_resistance = 0;
    c.base_fire_resistance = 5;
    c.base_cold_resistance = 9;
    c.base_poison_resistance = 20;
    c.base_energy_resistance = 20;
    c.init_min_hits = 150;
    c.init_max_hits = 150;
    c.can_fortify = false;
    c.on_create(|i| {
        i.hue = 0x4F2;
        i.attributes.bonus_int = 8;
        i.attributes.regen_mana = 4;
        i.attributes.spell_damage = 10;
    });
});

item("AcidProofRobe", |c| {
    c.label = 1095236;
    c.base_fire_resistance = 4;
    c.init_min_hits = 150;
    c.init_max_hits = 150;
    c.can_fortify = false;
    c.on_create(|i| {
        i.hue = 0x455;
        i.loot_type = "Blessed";
    });
});

item("SamaritanRobe", |c| {
    c.label = 1094926;
    c.base_physical_resistance = 5;
    c.init_min_hits = 150;
    c.init_max_hits = 150;
    c.can_fortify = false;
    c.on_create(|i| {
        i.hue = 0x2A3;
    });
});

item("LieutenantOfTheBritannianRoyalGuard", |c| {
    c.label = 1094910;
    c.init_min_hits = 150;
    c.init_max_hits = 150;
    c.can_fortify = false;
    c.on_create(|i| {
        i.hue = 0xE8;
        i.attributes.bonus_int = 5;
        i.attributes.regen_mana = 2;
        i.attributes.lower_reg_cost = 10;
    });
});

item("DjinnisRing", |c| {
    c.label = 1094927;
    c.init_min_hits = 150;
    c.init_max_hits = 150;
    c.on_create(|i| {
        i.attributes.bonus_int = 5;
        i.attributes.spell_damage = 10;
        i.attributes.cast_speed = 2;
    });
});

item("ANecromancerShroud", |c| {
    c.label = 1094913;
    c.base_cold_resistance = 5;
    c.init_min_hits = 150;
    c.init_max_hits = 150;
    c.can_fortify = false;
    c.on_create(|i| {
        i.hue = 0x455;
    });
});
