mobile("Alchemist", |c| {
    c.npc_sb_infos = ["SBAlchemist"];
    c.npc_guild = "MagesGuild";
    c.npc_shoe_type = ["Shoes", "Sandals"];
    c.on_create(|m| {
        m.title = "the alchemist";
        m.skill("Alchemy", 85.0, 100.0);
        m.skill("TasteID", 65.0, 88.0);
    });
    c.on_outfit(|m| {
        m.npc_vendor_outfit = true;
        m.npc_add_item = [#{ item: "Server.Items.Robe", hue: "randomPink" }];
    });
});

mobile("Architect", |c| {
    c.npc_sb_infos = [#{ era: "AOS", "else": ["SBHouseDeed"] }, "SBArchitect"];
    c.npc_guild = "TinkersGuild";
    c.on_create(|m| {
        m.title = "the architect";
    });
});

mobile("Armorer", |c| {
    c.npc_sb_infos = [
        #{
            oneOf: [
                [
                    "SBLeatherArmor",
                    "SBStuddedArmor",
                    "SBMetalShields",
                    "SBPlateArmor",
                    "SBHelmetArmor",
                    "SBChainmailArmor",
                    "SBRingmailArmor",
                ],
                ["SBStuddedArmor", "SBLeatherArmor", "SBMetalShields", "SBHelmetArmor"],
                ["SBMetalShields", "SBPlateArmor", "SBHelmetArmor", "SBChainmailArmor", "SBRingmailArmor"],
                ["SBMetalShields", "SBHelmetArmor"],
            ],
        },
        #{ "if": "tokunoVendor", then: ["SBSELeatherArmor", "SBSEArmor"] },
    ];
    c.npc_shoe_type = "Boots";
    c.on_create(|m| {
        m.title = "the armorer";
        m.skill("ArmsLore", 64.0, 100.0);
        m.skill("Blacksmith", 60.0, 83.0);
    });
    c.on_outfit(|m| {
        m.npc_vendor_outfit = true;
        m.npc_add_item = [#{ item: "Server.Items.HalfApron", hue: "randomYellow" }, "Server.Items.Bascinet"];
    });
});

mobile("Baker", |c| {
    c.npc_sb_infos = ["SBBaker"];
    c.on_create(|m| {
        m.title = "the baker";
        m.skill("Cooking", 75.0, 98.0);
        m.skill("TasteID", 36.0, 68.0);
    });
});

mobile("Bard", |c| {
    c.npc_sb_infos = ["SBBard"];
    c.npc_guild = "BardsGuild";
    c.on_create(|m| {
        m.title = "the bard";
        m.skill("Discordance", 64.0, 100.0);
        m.skill("Musicianship", 64.0, 100.0);
        m.skill("Peacemaking", 65.0, 88.0);
        m.skill("Provocation", 60.0, 83.0);
        m.skill("Archery", 36.0, 68.0);
        m.skill("Swords", 36.0, 68.0);
    });
});

mobile("Barkeeper", |c| {
    c.npc_sb_infos = ["SBBarkeeper"];
    c.npc_shoe_type = ["ThighBoots", "Boots"];
    c.on_create(|m| {
        m.title = "the barkeeper";
    });
    c.on_outfit(|m| {
        m.npc_vendor_outfit = true;
        m.npc_add_item = [#{ item: "Server.Items.HalfApron", hue: "randomBright" }];
    });
});

mobile("Beekeeper", |c| {
    c.npc_sb_infos = ["SBBeekeeper"];
    c.npc_shoe_type = "Boots";
    c.on_create(|m| {
        m.title = "the beekeeper";
    });
});

mobile("Bowyer", |c| {
    c.npc_sb_infos = ["SBBowyer", "SBRangedWeapon", #{ "if": "tokunoVendor", then: ["SBSEBowyer"] }];
    c.npc_shoe_type = #{ female: "ThighBoots", male: "Boots" };
    c.npc_shoe_hue = 0;
    c.on_create(|m| {
        m.title = "the bowyer";
        m.skill("Fletching", 80.0, 100.0);
        m.skill("Archery", 80.0, 100.0);
    });
    c.on_outfit(|m| {
        m.npc_vendor_outfit = true;
        m.npc_add_item = ["Server.Items.Bow", "Server.Items.LeatherGorget"];
    });
});

mobile("Butcher", |c| {
    c.npc_sb_infos = ["SBButcher"];
    c.on_create(|m| {
        m.title = "the butcher";
        m.skill("Anatomy", 45.0, 68.0);
    });
    c.on_outfit(|m| {
        m.npc_vendor_outfit = true;
        m.npc_add_item = ["Server.Items.HalfApron", "Server.Items.Cleaver"];
    });
});

mobile("Carpenter", |c| {
    c.npc_sb_infos = [
        "SBStavesWeapon",
        "SBCarpenter",
        "SBWoodenShields",
        #{ "if": "tokunoVendor", then: ["SBSECarpenter"] },
    ];
    c.npc_guild = "TinkersGuild";
    c.on_create(|m| {
        m.title = "the carpenter";
        m.skill("Carpentry", 85.0, 100.0);
        m.skill("Lumberjacking", 60.0, 83.0);
    });
    c.on_outfit(|m| {
        m.npc_vendor_outfit = true;
        m.npc_add_item = ["Server.Items.HalfApron"];
    });
});

mobile("Cobbler", |c| {
    c.npc_sb_infos = ["SBCobbler"];
    c.npc_shoe_type = ["Sandals", "Shoes"];
    c.on_create(|m| {
        m.title = "the cobbler";
        m.skill("Tailoring", 60.0, 83.0);
    });
});

mobile("Cook", |c| {
    c.npc_sb_infos = ["SBCook", #{ "if": "tokunoVendor", then: ["SBSECook"] }];
    c.npc_shoe_type = ["Sandals", "Shoes"];
    c.on_create(|m| {
        m.title = "the cook";
        m.skill("Cooking", 90.0, 100.0);
        m.skill("TasteID", 75.0, 98.0);
    });
    c.on_outfit(|m| {
        m.npc_vendor_outfit = true;
        m.npc_add_item = ["Server.Items.HalfApron"];
    });
});

mobile("Farmer", |c| {
    c.npc_sb_infos = ["SBFarmer"];
    c.npc_shoe_type = "ThighBoots";
    c.npc_shoe_hue = 0;
    c.on_create(|m| {
        m.title = "the farmer";
        m.skill("Lumberjacking", 36.0, 68.0);
        m.skill("TasteID", 36.0, 68.0);
        m.skill("Cooking", 36.0, 68.0);
    });
    c.on_outfit(|m| {
        m.npc_vendor_outfit = true;
        m.npc_add_item = [#{ item: "Server.Items.WideBrimHat", hue: "randomNeutral" }];
    });
});

mobile("Fisherman", |c| {
    c.npc_sb_infos = ["SBFisherman"];
    c.npc_guild = "FishermensGuild";
    c.on_create(|m| {
        m.title = "the fisher";
        m.skill("Fishing", 75.0, 98.0);
    });
    c.on_outfit(|m| {
        m.npc_vendor_outfit = true;
        m.npc_add_item = ["Server.Items.FishingPole"];
    });
});

mobile("Furtrader", |c| {
    c.npc_sb_infos = ["SBFurtrader"];
    c.on_create(|m| {
        m.title = "the furtrader";
        m.skill("Camping", 55.0, 78.0);
        m.skill("AnimalLore", 85.0, 100.0);
        m.skill("Cooking", 45.0, 68.0);
        m.skill("Tracking", 36.0, 68.0);
    });
});

mobile("Glassblower", |c| {
    c.npc_sb_infos = ["SBGlassblower", "SBAlchemist"];
    c.npc_guild = "MagesGuild";
    c.on_create(|m| {
        m.title = "the alchemist";
        m.skill("Alchemy", 85.0, 100.0);
        m.skill("TasteID", 85.0, 100.0);
    });
});

mobile("GolemCrafter", |c| {
    c.npc_sb_infos = ["SBTinker", "SBVagabond"];
    c.on_create(|m| {
        m.title = "the golem crafter";
        m.skill("Lockpicking", 60.0, 83.0);
        m.skill("RemoveTrap", 75.0, 98.0);
        m.skill("Tinkering", 64.0, 100.0);
    });
});

mobile("HairStylist", |c| {
    c.npc_sb_infos = ["SBHairStylist"];
    c.on_create(|m| {
        m.title = "the hair stylist";
        m.skill("Alchemy", 80.0, 100.0);
        m.skill("Magery", 90.0, 110.0);
        m.skill("TasteID", 85.0, 100.0);
    });
});

mobile("Herbalist", |c| {
    c.npc_sb_infos = ["SBHerbalist"];
    c.npc_guild = "MagesGuild";
    c.npc_shoe_type = ["Shoes", "Sandals"];
    c.on_create(|m| {
        m.title = "the herbalist";
        m.skill("Alchemy", 80.0, 100.0);
        m.skill("Cooking", 80.0, 100.0);
        m.skill("TasteID", 80.0, 100.0);
    });
});

mobile("InnKeeper", |c| {
    c.npc_sb_infos = ["SBInnKeeper", #{ "if": "tokunoVendor", then: ["SBSEFood"] }];
    c.npc_shoe_type = ["Sandals", "Shoes"];
    c.on_create(|m| {
        m.title = "the innkeeper";
    });
});

mobile("Jeweler", |c| {
    c.npc_sb_infos = ["SBJewel"];
    c.on_create(|m| {
        m.title = "the jeweler";
        m.skill("ItemID", 64.0, 100.0);
    });
});

mobile("LeatherWorker", |c| {
    c.npc_sb_infos = ["SBLeatherArmor", "SBStuddedArmor", "SBLeatherWorker"];
    c.on_create(|m| {
        m.title = "the leather worker";
    });
});

mobile("Mage", |c| {
    c.npc_sb_infos = ["SBMage"];
    c.npc_guild = "MagesGuild";
    c.npc_shoe_type = ["Shoes", "Sandals"];
    c.on_create(|m| {
        m.title = "the mage";
        m.skill("EvalInt", 65.0, 88.0);
        m.skill("Inscribe", 60.0, 83.0);
        m.skill("Magery", 64.0, 100.0);
        m.skill("Meditation", 60.0, 83.0);
        m.skill("MagicResist", 65.0, 88.0);
        m.skill("Wrestling", 36.0, 68.0);
    });
    c.on_outfit(|m| {
        m.npc_vendor_outfit = true;
        m.npc_add_item = [#{ item: "Server.Items.Robe", hue: "randomBlue" }];
    });
});

mobile("Mapmaker", |c| {
    c.npc_sb_infos = ["SBMapmaker"];
    c.on_create(|m| {
        m.title = "the mapmaker";
        m.skill("Cartography", 90.0, 100.0);
    });
});

mobile("Miller", |c| {
    c.npc_sb_infos = ["SBMiller"];
    c.on_create(|m| {
        m.title = "the miller";
    });
});

mobile("Miner", |c| {
    c.npc_sb_infos = ["SBMiner"];
    c.on_create(|m| {
        m.title = "the miner";
        m.skill("Mining", 65.0, 88.0);
    });
    c.on_outfit(|m| {
        m.npc_vendor_outfit = true;
        m.npc_replace_equipment = [#{ item: "Server.Items.FancyShirt", hue: 996 }, #{ item: "Server.Items.LongPants", hue: 402 }];
        m.npc_add_item = ["Server.Items.Pickaxe"];
        m.npc_seq = [#{ npcReplaceEquipment: [#{ item: "Server.Items.ThighBoots", hue: 643 }] }];
    });
});

mobile("Monk", |c| {
    c.npc_sb_infos = ["SBMonk"];
    c.on_create(|m| {
        m.title = "the Monk";
        m.skill("EvalInt", 100.0);
        m.skill("Tactics", 70.0, 90.0);
        m.skill("Wrestling", 70.0, 90.0);
        m.skill("MagicResist", 70.0, 90.0);
        m.skill("Macing", 70.0, 90.0);
    });
    c.on_outfit(|m| {
        m.npc_add_item = ["Server.Items.Sandals", "Server.Items.MonkRobe"];
    });
});

mobile("Provisioner", |c| {
    c.npc_sb_infos = ["SBProvisioner", #{ "if": "tokunoVendor", then: ["SBSEHats"] }];
    c.on_create(|m| {
        m.title = "the provisioner";
        m.skill("Camping", 45.0, 68.0);
        m.skill("Tactics", 45.0, 68.0);
    });
});

mobile("Rancher", |c| {
    c.npc_sb_infos = ["SBRancher"];
    c.on_create(|m| {
        m.title = "the rancher";
        m.skill("AnimalLore", 55.0, 78.0);
        m.skill("AnimalTaming", 55.0, 78.0);
        m.skill("Herding", 64.0, 100.0);
        m.skill("Veterinary", 60.0, 83.0);
    });
});

mobile("Ranger", |c| {
    c.npc_sb_infos = ["SBRanger"];
    c.on_create(|m| {
        m.title = "the ranger";
        m.skill("Camping", 55.0, 78.0);
        m.skill("DetectHidden", 65.0, 88.0);
        m.skill("Hiding", 45.0, 68.0);
        m.skill("Archery", 65.0, 88.0);
        m.skill("Tracking", 65.0, 88.0);
        m.skill("Veterinary", 60.0, 83.0);
    });
    c.on_outfit(|m| {
        m.npc_vendor_outfit = true;
        m.npc_replace_equipment = [
            #{ item: "Server.Items.Shirt", hue: "randomNeutral" },
            #{ item: "Server.Items.LongPants", hue: "randomNeutral" },
        ];
        m.npc_add_item = ["Server.Items.Bow"];
        m.npc_seq = [#{ npcReplaceEquipment: [#{ item: "Server.Items.ThighBoots", hue: "randomNeutral" }] }];
    });
});

mobile("Shipwright", |c| {
    c.npc_sb_infos = ["SBShipwright"];
    c.on_create(|m| {
        m.title = "the shipwright";
        m.skill("Carpentry", 60.0, 83.0);
        m.skill("Macing", 36.0, 68.0);
    });
    c.on_outfit(|m| {
        m.npc_vendor_outfit = true;
        m.npc_add_item = ["Server.Items.SmithHammer"];
    });
});

mobile("StoneCrafter", |c| {
    c.npc_sb_infos = ["SBStoneCrafter", "SBStavesWeapon", "SBCarpenter", "SBWoodenShields"];
    c.npc_guild = "TinkersGuild";
    c.on_create(|m| {
        m.title = "the stone crafter";
        m.skill("Carpentry", 85.0, 100.0);
    });
});

mobile("Tanner", |c| {
    c.npc_sb_infos = ["SBTanner"];
    c.on_create(|m| {
        m.title = "the tanner";
        m.skill("Tailoring", 36.0, 68.0);
    });
});

mobile("TavernKeeper", |c| {
    c.npc_sb_infos = ["SBTavernKeeper"];
    c.on_create(|m| {
        m.title = "the tavern keeper";
    });
    c.on_outfit(|m| {
        m.npc_vendor_outfit = true;
        m.npc_add_item = ["Server.Items.HalfApron"];
    });
});

mobile("Thief", |c| {
    c.npc_sb_infos = ["SBThief"];
    c.on_create(|m| {
        m.title = "the thief";
        m.skill("Camping", 55.0, 78.0);
        m.skill("DetectHidden", 65.0, 88.0);
        m.skill("Hiding", 45.0, 68.0);
        m.skill("Archery", 65.0, 88.0);
        m.skill("Tracking", 65.0, 88.0);
        m.skill("Veterinary", 60.0, 83.0);
    });
    c.on_outfit(|m| {
        m.npc_vendor_outfit = true;
        m.npc_replace_equipment = [
            #{ item: "Server.Items.Shirt", hue: "randomNeutral" },
            #{ item: "Server.Items.LongPants", hue: "randomNeutral" },
        ];
        m.npc_add_item = ["Server.Items.Dagger"];
        m.npc_seq = [#{ npcReplaceEquipment: [#{ item: "Server.Items.ThighBoots", hue: "randomNeutral" }] }];
    });
});

mobile("Tinker", |c| {
    c.npc_sb_infos = ["SBTinker"];
    c.npc_guild = "TinkersGuild";
    c.on_create(|m| {
        m.title = "the tinker";
        m.skill("Lockpicking", 60.0, 83.0);
        m.skill("RemoveTrap", 75.0, 98.0);
        m.skill("Tinkering", 64.0, 100.0);
    });
});

mobile("VarietyDealer", |c| {
    c.npc_sb_infos = ["SBVarietyDealer"];
    c.on_create(|m| {
        m.title = "the variety dealer";
    });
});

mobile("Veterinarian", |c| {
    c.npc_sb_infos = ["SBVeterinarian"];
    c.on_create(|m| {
        m.title = "the vet";
        m.skill("AnimalLore", 85.0, 100.0);
        m.skill("Veterinary", 90.0, 100.0);
    });
});

mobile("Waiter", |c| {
    c.npc_sb_infos = ["SBWaiter"];
    c.on_create(|m| {
        m.title = "the waiter";
        m.skill("Discordance", 36.0, 68.0);
    });
    c.on_outfit(|m| {
        m.npc_vendor_outfit = true;
        m.npc_add_item = ["Server.Items.HalfApron"];
    });
});
