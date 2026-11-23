mobile("MaulTheBear", |c| {
    c.name = "Maul";
    c.is_invulnerable = true;
    c.on_create(|m| {
        m.creature_ai = "AiVendor";
        m.creature_fight_mode = "None";
        m.tamable = false;
    });
});

mobile("Strongroot", |c| {
    c.name = "Strongroot";
    c.is_invulnerable = true;
    c.on_create(|m| {
        m.creature_ai = "AiVendor";
        m.creature_fight_mode = "None";
    });
});

mobile("BravehornsMate", |c| {
    c.name = "bravehorn's mate";
    c.is_invulnerable = false;
    c.on_create(|m| {
        m.tamable = false;
    });
});

mobile("Huntsman", |c| {
    c.name = "Huntsman";
    c.is_invulnerable = true;
    c.on_create(|m| {
        m.creature_ai = "AiVendor";
        m.creature_fight_mode = "None";
    });
});

mobile("Arielle", |c| {
    c.name = "Arielle";
    c.is_invulnerable = true;
    c.on_create(|m| {
        m.creature_ai = "AiVendor";
        m.creature_fight_mode = "None";
    });
});

mobile("Aelorn", |c| {
    c.inherits = "KeeperOfChivalry";
    c.name = "Aelorn";
    c.can_shout = true;
    c.on_create(|m| {
        m.title = "the Chivalry Instructor";
        m.body = 400;
        m.mlq_hue = 33770;
        m.mlq_hair_item_id = 8252;
        m.mlq_hair_hue = 1149;
        m.mlq_facial_hair_item_id = 8269;
        m.mlq_facial_hair_hue = 1149;
        m.mlq_speed = [0.5, 2.0];
        m.mlq_init_stats = [100, 100, 25];
        m.skill("Anatomy", 120.0);
        m.skill("MagicResist", 120.0);
        m.skill("Tactics", 120.0);
        m.skill("Swords", 120.0);
        m.skill("Meditation", 120.0);
        m.skill("Focus", 120.0);
        m.skill("Chivalry", 120.0);
    });
    c.on_outfit(|m| {
        m.mlq_equip = [
            #{ item: "Backpack" },
            #{ item: "VikingSword" },
            #{ item: "PlateChest" },
            #{ item: "PlateLegs" },
            #{ item: "PlateGloves" },
            #{ item: "PlateArms" },
            #{ item: "PlateGorget" },
            #{ item: "OrderShield" },
        ];
    });
});

mobile("Robyn", |c| {
    c.inherits = "Bowyer";
    c.name = "Robyn";
    c.can_shout = true;
    c.on_create(|m| {
        m.title = "the Archery Instructor";
        m.body = 401;
        m.female = true;
        m.mlq_hue = 33770;
        m.mlq_hair_item_id = 8252;
        m.mlq_hair_hue = 1149;
        m.mlq_speed = [0.5, 2.0];
        m.mlq_init_stats = [100, 100, 25];
        m.skill("Anatomy", 120.0);
        m.skill("Parry", 120.0);
        m.skill("Fletching", 120.0);
        m.skill("Healing", 120.0);
        m.skill("Tactics", 120.0);
        m.skill("Archery", 120.0);
        m.skill("Focus", 120.0);
    });
    c.on_outfit(|m| {
        m.mlq_equip = [
            #{ item: "Backpack" },
            #{ item: "Boots", hue: 1426 },
            #{ item: "Cloak", hue: 1426 },
            #{ item: "Bandana", hue: 1426 },
            #{ item: "CompositeBow" },
            #{ item: "StuddedLegs", dye: 1426 },
            #{ item: "StuddedGloves", dye: 1426 },
            #{ item: "StuddedGorget", dye: 1426 },
            #{ item: "StuddedChest", dye: 1426 },
            #{ item: "StuddedArms", dye: 1426 },
        ];
    });
});

mobile("AmeliaYoungstone", |c| {
    c.inherits = "Tinker";
    c.name = "Amelia Youngstone";
    c.can_shout = true;
    c.on_create(|m| {
        m.title = "the Tinkering Instructor";
        m.body = 401;
        m.female = true;
        m.mlq_hue = 33770;
        m.mlq_hair_item_id = 8253;
        m.mlq_hair_hue = 1132;
        m.mlq_speed = [0.5, 2.0];
        m.mlq_init_stats = [100, 100, 25];
        m.skill("ArmsLore", 120.0);
        m.skill("Blacksmith", 120.0);
        m.skill("Magery", 120.0);
        m.skill("Tactics", 120.0);
        m.skill("Swords", 120.0);
        m.skill("Tinkering", 120.0);
        m.skill("Mining", 120.0);
    });
    c.on_outfit(|m| {
        m.mlq_equip = [
            #{ item: "Backpack" },
            #{ item: "Sandals" },
            #{ item: "Doublet" },
            #{ item: "ShortPants" },
            #{ item: "HalfApron", hue: 2219 },
        ];
    });
});

mobile("Mulcivikh", |c| {
    c.inherits = "Mage";
    c.name = "Mulcivikh";
    c.can_shout = true;
    c.on_create(|m| {
        m.title = "the Necromancy Instructor";
        m.body = 400;
        m.mlq_hue = 33770;
        m.mlq_hair_item_id = 8253;
        m.mlq_hair_hue = 1111;
        m.mlq_speed = [0.5, 2.0];
        m.mlq_init_stats = [100, 100, 25];
        m.skill("Magery", 120.0);
        m.skill("MagicResist", 120.0);
        m.skill("SpiritSpeak", 120.0);
        m.skill("Swords", 120.0);
        m.skill("Meditation", 120.0);
        m.skill("Necromancy", 120.0);
    });
    c.on_outfit(|m| {
        m.mlq_equip = [
            #{ item: "Backpack" },
            #{ item: "Sandals", hue: 2301 },
            #{ item: "BoneHelm" },
            #{ item: "LeatherLegs", dye: 707 },
            #{ item: "LeatherGloves", dye: 707 },
            #{ item: "LeatherGorget", dye: 707 },
            #{ item: "LeatherChest", dye: 707 },
            #{ item: "LeatherArms", dye: 707 },
        ];
    });
});

mobile("Morganna", |c| {
    c.inherits = "Mage";
    c.name = "Morganna";
    c.can_shout = true;
    c.on_create(|m| {
        m.title = "the Spirit Speak Instructor";
        m.body = 401;
        m.female = true;
        m.mlq_hue = 33770;
        m.mlq_hair_item_id = 8252;
        m.mlq_hair_hue = 1109;
        m.mlq_speed = [0.5, 2.0];
        m.mlq_init_stats = [100, 100, 25];
        m.skill("Magery", 120.0);
        m.skill("MagicResist", 120.0);
        m.skill("SpiritSpeak", 120.0);
        m.skill("Swords", 120.0);
        m.skill("Meditation", 120.0);
        m.skill("Necromancy", 120.0);
    });
    c.on_outfit(|m| {
        m.mlq_equip = [
            #{ item: "Backpack" },
            #{ item: "Sandals" },
            #{ item: "Robe", hue: 1149 },
            #{ item: "SkullCap", hue: 1109 },
        ];
    });
});

mobile("GeorgeHephaestus", |c| {
    c.inherits = "Blacksmith";
    c.name = "George Hephaestus";
    c.can_shout = true;
    c.on_create(|m| {
        m.title = "the Blacksmith Instructor";
        m.body = 400;
        m.mlq_hue = 33770;
        m.mlq_hair_item_id = 8251;
        m.mlq_hair_hue = 1147;
        m.mlq_speed = [0.5, 2.0];
        m.mlq_init_stats = [100, 100, 25];
        m.skill("ArmsLore", 120.0);
        m.skill("Blacksmith", 120.0);
        m.skill("Magery", 120.0);
        m.skill("Tactics", 120.0);
        m.skill("Tinkering", 120.0);
        m.skill("Swords", 120.0);
        m.skill("Mining", 120.0);
    });
    c.on_outfit(|m| {
        m.mlq_equip = [
            #{ item: "Backpack" },
            #{ item: "Boots", hue: 2419 },
            #{ item: "LongPants" },
            #{ item: "Bascinet" },
            #{ item: "FullApron", hue: 2219 },
            #{ item: "SmithHammer", hue: 2219 },
        ];
    });
});

mobile("Ryuichi", |c| {
    c.name = "Ryuichi";
    c.can_shout = true;
    c.on_create(|m| {
        m.title = "the Ninjitsu Instructor";
        m.mlq_hue = 33795;
        m.skill("Hiding", 120.0);
        m.skill("Tactics", 120.0);
        m.skill("Tracking", 120.0);
        m.skill("Fencing", 120.0);
        m.skill("Stealth", 120.0);
        m.skill("Ninjitsu", 120.0);
    });
    c.on_outfit(|m| {
        m.mlq_hair_item_id = 8251;
        m.mlq_hair_hue = 1109;
        m.mlq_equip = [
            #{ item: "SamuraiTabi" },
            #{ item: "LeatherNinjaPants" },
            #{ item: "LeatherNinjaMitts" },
            #{ item: "LeatherNinjaHood" },
            #{ item: "LeatherNinjaJacket" },
            #{ item: "LeatherNinjaBelt" },
        ];
        m.pack = [#{ gold: [100, 200] }];
    });
});

mobile("Hamato", |c| {
    c.name = "Hamato";
    c.can_shout = true;
    c.on_create(|m| {
        m.title = "the Bushido Instructor";
        m.mlq_hue = 33795;
        m.skill("Anatomy", 120.0);
        m.skill("Parry", 120.0);
        m.skill("Healing", 120.0);
        m.skill("Tactics", 120.0);
        m.skill("Swords", 120.0);
        m.skill("Bushido", 120.0);
    });
    c.on_outfit(|m| {
        m.mlq_hair_item_id = 8253;
        m.mlq_hair_hue = 1175;
        m.mlq_equip = [
            #{ item: "Backpack" },
            #{ item: "NoDachi" },
            #{ item: "NinjaTabi" },
            #{ item: "PlateSuneate" },
            #{ item: "LightPlateJingasa" },
            #{ item: "LeatherDo" },
            #{ item: "LeatherHiroSode" },
        ];
        m.pack = [#{ gold: [100, 200] }];
    });
});

mobile("SirHelper", |c| {
    c.inherits = "Mage";
    c.name = "Sir Helper";
    c.npc_sb_infos = [];
    c.on_create(|m| {
        m.title = "the Profession Guide";
        m.mlq_hue = 33770;
        m.mlq_direction = "South";
        m.mlq_frozen = true;
    });
    c.on_outfit(|m| {
        m.mlq_hair_item_id = 8252;
        m.mlq_facial_hair_item_id = 8269;
        m.mlq_hair_hue = 2215;
        m.mlq_facial_hair_hue = 2215;
        m.mlq_equip = [
            #{ item: "Sandals" },
            #{ item: "Cloak", hue: 1109, itemId: 9901 },
            #{ item: "Robe", hue: 1195, itemId: 9902 },
            #{ item: "Backpack", movable: false },
        ];
    });
});

mobile("Darius", |c| {
    c.ai = "Vendor";
    c.fight_mode = "None";
    c.range_perception = 2;
    c.name = "Darius";
    c.is_invulnerable = true;
    c.on_create(|m| {
        m.mlq_speed = [0.5, 2.0];
        m.title = "the wise";
        m.mlq_race = "Elf";
        m.mlq_skin_hue = true;
        m.npc_speech_hue = "randomDyed";
        m.mlq_equip = [
            #{ item: "WildStaff" },
            #{ item: "Sandals", hue: 443 },
            #{ item: "GemmedCirclet" },
            #{ item: "Tunic", hue: "randomBrightHue" },
        ];
        m.mlq_random_hair = true;
        m.str = 40..=50;
        m.dex = 60..=70;
        m.int = 90..=100;
    });
});

mobile("Nedrick", |c| {
    c.ai = "Vendor";
    c.fight_mode = "None";
    c.range_perception = 2;
    c.name = "Nedrick";
    c.is_invulnerable = true;
    c.on_create(|m| {
        m.mlq_speed = [0.5, 2.0];
        m.title = "the iron worker";
        m.mlq_race = "Human";
        m.mlq_skin_hue = true;
        m.npc_speech_hue = "randomDyed";
        m.mlq_equip = [
            #{ item: "Boots" },
            #{ item: "LongPants", hue: "randomNondyedHue" },
            #{ item: "FancyShirt", hue: "randomNondyedHue" },
        ];
        m.mlq_random_hair = true;
        m.quest_facial_hair_of_hair_hue = true;
        m.str = 70..=80;
        m.dex = 50..=60;
        m.int = 60..=70;
    });
});
