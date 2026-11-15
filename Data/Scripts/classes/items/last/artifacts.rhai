item("AegisOfGrace", |c| {
    c.base_physical_resistance = 10;
    c.base_fire_resistance = 9;
    c.base_cold_resistance = 7;
    c.base_poison_resistance = 7;
    c.base_energy_resistance = 15;
    c.init_min_hits = 255;
    c.init_max_hits = 255;
    c.label = 1075047;
    c.material_type = "Dragon";
    c.default_resource = "Iron";
    c.required_races = "AllowElvesOnly";
    c.on_create(|i| {
        i.skill_bonuses = [["MagicResist", 10.0]];
        i.attributes.defend_chance = 20;
        i.armor_attributes.self_repair = 2;
    });
});

item("FeyLeggings", |c| {
    c.base_physical_resistance = 12;
    c.base_fire_resistance = 8;
    c.base_cold_resistance = 7;
    c.base_poison_resistance = 4;
    c.base_energy_resistance = 19;
    c.init_min_hits = 255;
    c.init_max_hits = 255;
    c.label = 1075041;
    c.required_races = "AllowElvesOnly";
    c.on_create(|i| {
        i.attributes.bonus_hits = 6;
        i.attributes.defend_chance = 20;
        i.armor_attributes.mage_armor = 1;
    });
});

item("WildfireBow", |c| {
    c.init_min_hits = 255;
    c.init_max_hits = 255;
    c.label = 1075044;
    c.damage_types = #{ fire: 100 };
    c.on_create(|i| {
        i.hue = 0x489;
        i.skill_bonuses = [["Archery", 10.0]];
        i.weapon_attributes.resist_fire_bonus = 25;
        i.velocity = 15;
    });
});

item("Windsong", |c| {
    c.init_min_hits = 255;
    c.init_max_hits = 255;
    c.label = 1075031;
    c.on_create(|i| {
        i.hue = 0xF7;
        i.attributes.weapon_damage = 35;
        i.weapon_attributes.self_repair = 3;
        i.velocity = 25;
    });
});

item("ShieldOfInvulnerability", |c| {
    c.base_physical_resistance = 8;
    c.base_fire_resistance = 0;
    c.base_cold_resistance = 0;
    c.base_poison_resistance = 0;
    c.base_energy_resistance = 0;
    c.init_min_hits = 255;
    c.init_max_hits = 255;
    c.label = 1070693;
    c.on_create(|i| {
        i.hue = 0x4F2;
        i.attributes.spell_channeling = 1;
        i.attributes.reflect_physical = 10;
        i.attributes.defend_chance = 15;
        i.armor_attributes.lower_stat_req = 100;
    });
});

item("WrathOfTheDryad", |c| {
    c.init_min_hits = 255;
    c.init_max_hits = 255;
    c.label = 1070853;
    c.primary_ability = "ConcussionBlow";
    c.secondary_ability = "ParalyzingBlow";
    c.damage_types = #{ pois: 100 };
    c.on_create(|i| {
        i.hue = 0x29C;
        i.weapon_attributes.hit_leech_mana = 50;
        i.weapon_attributes.hit_lightning = 33;
        i.attributes.attack_chance = 15;
        i.attributes.weapon_damage = 40;
    });
});

item("OrcChieftainHelm", |c| {
    c.base_physical_resistance = 23;
    c.base_fire_resistance = 1;
    c.base_cold_resistance = 23;
    c.base_poison_resistance = 3;
    c.base_energy_resistance = 5;
    c.init_min_hits = 150;
    c.init_max_hits = 150;
    c.label = 1094924;
    c.can_fortify = false;
    c.on_create(|i| {
        i.hue = 0x2A3;
        i.attributes.luck = 100;
        i.attributes.regen_hits = 3;
        if coin() {
            i.attributes.bonus_hits = 30;
        } else {
            i.attributes.attack_chance = 30;
        }
    });
});

item("ShroudOfDeciet", |c| {
    c.base_physical_resistance = 11;
    c.base_fire_resistance = 6;
    c.base_cold_resistance = 18;
    c.base_poison_resistance = 15;
    c.base_energy_resistance = 13;
    c.init_min_hits = 150;
    c.init_max_hits = 150;
    c.label = 1094914;
    c.can_fortify = false;
    c.on_create(|i| {
        i.hue = 0x38F;
        i.attributes.regen_hits = 3;
        i.armor_attributes.mage_armor = 1;
        i.skill_bonuses = [["MagicResist", 10.0]];
    });
});

item("EscutcheonDeAriadne", |c| {
    c.base_physical_resistance = 5;
    c.base_energy_resistance = 1;
    c.aos_str_req = 14;
    c.label = 1077694;
    c.on_create(|i| {
        i.loot_type = "Blessed";
        i.hue = 0x8A5;
        i.armor_attributes.durability_bonus = 49;
        i.attributes.reflect_physical = 5;
        i.attributes.defend_chance = 5;
    });
});

item("TheRobeOfBritanniaAri", |c| {
    c.label = 1094931;
    c.base_physical_resistance = 10;
    c.init_min_hits = 150;
    c.init_max_hits = 150;
    c.can_fortify = false;
    c.graphic = 9860;
    c.on_create(|i| {
        i.hue = 0x48B;
        i.str_requirement = 0;
    });
});

item("TheMostKnowledgePerson", |c| {
    c.script = "content/equipment/the_most_knowledge_person.rhai";
    c.label = 1094893;
    c.init_min_hits = 150;
    c.init_max_hits = 150;
    c.can_fortify = false;
    c.graphic = 9860;
    c.on_create(|i| {
        i.hue = 0x117;
        i.str_requirement = 0;
    });
});

item("EmbroideredOakLeafCloak", |c| {
    c.label = 1094901;
    c.init_min_hits = 150;
    c.init_max_hits = 150;
    c.can_fortify = false;
    c.graphic = 9860;
    c.on_create(|i| {
        i.hue = 0x483;
        i.str_requirement = 0;
        i.skill_bonuses = [["Stealth", 5.0]];
    });
});

item("CaptainJohnsHat", |c| {
    c.label = 1094911;
    c.base_physical_resistance = 2;
    c.base_fire_resistance = 6;
    c.base_cold_resistance = 9;
    c.base_poison_resistance = 7;
    c.base_energy_resistance = 23;
    c.init_min_hits = 150;
    c.init_max_hits = 150;
    c.can_fortify = false;
    c.on_create(|i| {
        i.hue = 0x455;
        i.attributes.bonus_dex = 8;
        i.attributes.night_sight = 1;
        i.attributes.attack_chance = 15;
        i.skill_bonuses = [["Swords", 20.0]];
    });
});

item("RobeOfTheEquinox", |c| {
    c.label = 1075042;
    c.weight = 3.0;
    c.graphic = 7940;
    c.on_create(|i| {
        i.hue = 0xD6;
        i.attributes.luck = 95;
    });
});

item("RobeOfTheEclipse", |c| {
    c.label = 1075082;
    c.weight = 3.0;
    c.graphic = 7939;
    c.on_create(|i| {
        i.hue = 0x486;
        i.attributes.luck = 95;
    });
});

item("PadsOfTheCuSidhe", |c| {
    c.label = 1075048;
    c.on_create(|i| {
        i.hue = 0x47E;
    });
});

item("DreadPirateHat", |c| {
    c.label = 1063467;
    c.base_cold_resistance = 14;
    c.base_poison_resistance = 10;
    c.init_min_hits = 255;
    c.init_max_hits = 255;
    c.on_create(|i| {
        i.hue = 0x497;
        switch random_below(5) {
            0 => {
                i.skill_bonuses = [["Tactics", 10.0]];
            },
            1 => {
                i.skill_bonuses = [["Archery", 10.0]];
            },
            2 => {
                i.skill_bonuses = [["Swords", 10.0]];
            },
            3 => {
                i.skill_bonuses = [["Macing", 10.0]];
            },
            4 => {
                i.skill_bonuses = [["Fencing", 10.0]];
            }
        }
        i.attributes.bonus_dex = 8;
        i.attributes.attack_chance = 10;
        i.attributes.night_sight = 1;
    });
});

item("BurglarsBandana", |c| {
    c.label = 1063473;
    c.base_physical_resistance = 10;
    c.base_fire_resistance = 5;
    c.base_cold_resistance = 7;
    c.base_poison_resistance = 10;
    c.base_energy_resistance = 10;
    c.init_min_hits = 255;
    c.init_max_hits = 255;
    c.on_create(|i| {
        if coin() {
            i.hue = 0x58C;
        } else {
            i.hue = 0x10;
        }
        i.skill_bonuses = [["Stealing", 10.0], ["Stealth", 10.0], ["Snooping", 10.0]];
        i.attributes.bonus_dex = 5;
    });
});

item("JacobsPickaxe", |c| {
    c.label = 1077758;
    c.on_create(|i| {
        i.uses_remaining = 20;
        i.loot_type = "Blessed";
        i.skill_bonuses = [["Mining", 10.0]];
    });
});

item("DetectiveBoots", |c| {
    c.script = "content/equipment/detective_boots.rhai";
    c.init_min_hits = 150;
    c.init_max_hits = 150;
    c.can_fortify = false;
    c.on_create(|i| {
        i.hue = 0x455;
    });
});

item("BloodwoodSpirit", |c| {
    c.label = 1075034;
});

item("TotemOfVoid", |c| {
    c.label = 1075035;
});

item("AmeliasToolbox", |c| {
    c.label = 1077749;
    c.on_create(|i| {
        i.item_id = 0x1EB8;
        i.loot_type = "Blessed";
        i.hue = 0x767;
    });
});

item("HammerOfHephaestus", |c| {
    c.label = 1077740;
    c.on_create(|i| {
        i.item_id = 0x13E3;
        i.layer = "OneHanded";
        i.tool_uses = 20;
        i.loot_type = "Blessed";
    });
});

item("HallowedSpellbook", |c| {
    c.label = 1077620;
});
