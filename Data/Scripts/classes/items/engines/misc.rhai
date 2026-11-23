item("Silver", |c| {
    c.weight = 0.02;
    c.on_create(|i| {
        i.item_id = 0xEF0;
        i.stackable = true;
        i.amount = 1;
    });
});

item("BulkOrderBook", |c| {
    c.weight = 1.0;
});

item("Server.Engines.BulkOrders.BaseBOD", |c| {
    c.weight = 1.0;
});

item("Server.Engines.BulkOrders.SmallBOD", |c| {
    c.label = 1045151;
});

item("Server.Engines.BulkOrders.LargeBOD", |c| {
    c.label = 1045151;
});

item("HarrowerGate", |c| {
    c.label = 1049498;
});

item("StarRoomGate", |c| {
    c.label = 1049498;
});

item("ChampionSpawn", |c| {
    c.label = 1041030;
    c.on_create(|i| {
        i.item_id = 0xBD2;
        i.movable = false;
        i.visible = false;
    });
});

item("IdolOfTheChampion", |c| {
    c.name = "Idol of the Champion";
});

item("ArenasMoongate", |c| {
    c.name = "arena moongate";
    c.on_create(|i| {
        i.item_id = 0x1FD4;
        i.movable = false;
        i.light = "Circle300";
    });
});

item("BRBoard", |c| {
    c.name = "Scoreboard";
    c.on_create(|i| {
        i.item_id = 0x1E5E;
        i.movable = false;
    });
});

item("BRBomb", |c| {
    c.name = "da bomb";
});

item("BRGoal", |c| {
    c.name = "Bombing Run Goal";
});

item("BRController", |c| {
    c.name = "Bombing Run Controller";
    c.on_create(|i| {
        i.item_id = 0x1B7A;
        i.visible = false;
        i.movable = false;
    });
});

item("CTFBoard", |c| {
    c.name = "scoreboard";
    c.on_create(|i| {
        i.item_id = 0x1E5E;
        i.movable = false;
    });
});

item("CTFFlag", |c| {
    c.name = "old people cookies";
    c.on_create(|i| {
        i.item_id = 0x160B;
        i.movable = false;
    });
});

item("DDBoard", |c| {
    c.name = "scoreboard";
    c.on_create(|i| {
        i.item_id = 0x1E5E;
        i.movable = false;
    });
});

item("DDController", |c| {
    c.name = "DD Controller";
    c.on_create(|i| {
        i.item_id = 0x1B7A;
        i.visible = false;
        i.movable = false;
    });
});

item("LadderController", |c| {
    c.name = "ladder controller";
    c.on_create(|i| {
        i.item_id = 0x1B7A;
        i.visible = false;
        i.movable = false;
    });
});

item("PreferencesController", |c| {
    c.name = "preferences controller";
    c.on_create(|i| {
        i.item_id = 0x1B7A;
        i.visible = false;
        i.movable = false;
    });
});

item("LadderItem", |c| {
    c.name = "1v1 leaderboard";
    c.on_create(|i| {
        i.item_id = 0x117F;
        i.movable = false;
    });
});

item("ArenaMoongate", |c| {
    c.name = "spectator moongate";
});

item("TournamentBracketItem", |c| {
    c.name = "tournament bracket";
    c.on_create(|i| {
        i.item_id = 0xEBE;
        i.movable = false;
    });
});

item("TournamentSignupItem", |c| {
    c.name = "tournament signup book";
    c.on_create(|i| {
        i.item_id = 0xFBD;
        i.movable = false;
    });
});

item("SalvageBag", |c| {
    c.script = "content/engines/salvage_bag.rhai";
    c.label = 1079931;
    c.weight = 2.0;
    c.on_create(|i| {
        i.item_id = 0xE76;
    });
});

item("GauntletSpawner", |c| {
    c.name = "doom spawner";
    c.on_create(|i| {
        i.item_id = 0x36FE;
        i.visible = false;
        i.movable = false;
    });
});

item("FactionStone", |c| {
    c.name = "faction stone";
});

item("JoinStone", |c| {
    c.name = "faction signup stone";
});

item("TownStone", |c| {
    c.name = "faction town stone";
});

item("BloodRose", |c| {
    c.name = "blood rose";
});

item("FactionTrapRemovalKit", |c| {
    c.label = 1041508;
});

item("KhaldunPitTeleporter", |c| {
    c.label = 1016511;
});

item("MLQuestPersistence", |c| {
    c.name = "ML quests persistence - Internal";
});

item("RaceChangeDeed", |c| {
    c.name = "a race change deed";
    c.on_create(|i| {
        i.item_id = 0x14F0;
        i.loot_type = "Blessed";
    });
});

item("PlantItem", |c| {
    c.weight = 1.0;
});

item("Seed", |c| {
    c.label = 1060810;
    c.weight = 1.0;
});

item("PaintedImage", |c| {
    c.weight = 1.0;
});

item("EnchantedPaints", |c| {
    c.weight = 1.0;
    c.on_create(|i| {
        i.item_id = 0xFC1;
        i.loot_type = "Blessed";
    });
});

item("KronusScroll", |c| {
    c.label = 1060149;
    c.weight = 1.0;
    c.on_create(|i| {
        i.item_id = 0x227A;
        i.hue = 0x44E;
    });
});

item("ScrollOfAbraxus", |c| {
    c.label = 1028827;
    c.weight = 1.0;
    c.on_create(|i| {
        i.item_id = 0x227B;
    });
});

item("EminosKatana", |c| {
    c.label = 1063214;
    c.weight = 1.0;
    c.on_create(|i| {
        i.item_id = 0x13FF;
    });
});

item("HaochisKatana", |c| {
    c.label = 1063165;
    c.weight = 1.0;
    c.on_create(|i| {
        i.item_id = 0x13FF;
    });
});

item("HaochisKatanaGenerator", |c| {
    c.name = "Haochi's katana generator";
    c.on_create(|i| {
        i.item_id = 0x1B7B;
        i.visible = false;
        i.movable = false;
    });
});

item("DynamicTeleporter", |c| {
    c.label = 1049382;
});

item("HornOfRetreat", |c| {
    c.label = 1049117;
    c.weight = 1.0;
});

item("BellOfTheDead", |c| {
    c.label = 1050018;
    c.on_create(|i| {
        i.item_id = 0x91A;
        i.hue = 0x835;
        i.movable = false;
    });
});

item("HangoverCure", |c| {
    c.label = 1055060;
    c.weight = 1.0;
    c.on_create(|i| {
        i.item_id = 0xE2B;
        i.hue = 0x2D;
    });
});

item("CharacterStatueDeed", |c| {
    c.weight = 1.0;
});

item("CharacterStatueMaker", |c| {
    c.label = 1076173;
    c.weight = 5.0;
});

item("CharacterStatuePlinth", |c| {
    c.label = 1076201;
});

item("AncientUrn", |c| {
    c.script = "content/engines/tokuno_named.rhai";
    c.label = 1071014;
    c.weight = 1.0;
    c.on_create(|i| {
        i.item_id = 0x241D;
    });
});

item("HonorableSwords", |c| {
    c.script = "content/engines/tokuno_named.rhai";
    c.label = 1071015;
    c.weight = 5.0;
    c.on_create(|i| {
        i.item_id = 0x2853;
    });
});

item("ChestOfHeirlooms", |c| {
    c.label = 1070937;
    c.on_create(|i| {
        i.item_id = 0x2811;
        i.set = block([
            #{ Locked: true },
            #{ LockLevel: 95 },
            #{ MaxLockLevel: 140 },
            #{ RequiredSkill: 95 },
            #{ TrapType: "ExplosionTrap" },
            #{ TrapLevel: 10 },
            #{ TrapPower: 100 },
            #{ GumpID: 267 },
        ]);
    });
});

item("BasePigmentsOfTokuno", |c| {
    c.label = 1070933;
    c.weight = 1.0;
});

item("PigmentsOfTokuno", |c| {
    c.script = "content/engines/pigments_of_tokuno.rhai";
    c.label = 1070933;
});

item("KHBoard", |c| {
    c.on_create(|i| {
        i.item_id = 0x1E5E;
        i.name = "King of the Hill Scoreboard";
        i.movable = false;
    });
});

item("HillOfTheKing", |c| {
    c.on_create(|i| {
        i.item_id = 0x520;
        i.movable = false;
        i.name = "the hill";
    });
});

item("Obsidian", |c| {
    c.on_create(|i| {
        i.item_id = 0x1EA7;
        i.hue = 0x497;
    });
});

item("KronusScrollBox", |c| {
    c.on_create(|i| {
        i.item_id = 0xE80;
        i.movable = false;
    });
});

item("EminosKatanaChest", |c| {
    c.on_create(|i| {
        i.item_id = 0xE42;
        i.movable = false;
    });
});

item("HaochisTreasureChest", |c| {
    c.on_create(|i| {
        i.item_id = 0x2811;
        i.movable = false;
        i.set = #{ GumpID: 267 };
    });
});

item("FertileDirt", |c| {
    c.weight = 1.0;
    c.on_create(|i| {
        i.item_id = 0xF81;
        i.stackable = true;
        i.amount = 1;
    });
});

item("PlantBowl", |c| {
    c.label = 1060834;
    c.weight = 1.0;
    c.on_create(|i| {
        i.item_id = 0x15FD;
    });
});

item("OrangePetals", |c| {
    c.label = 1053122;
    c.weight = 0.1;
    c.on_create(|i| {
        i.item_id = 0x1021;
        i.stackable = true;
        i.hue = 0x2B;
        i.amount = 1;
    });
});

item("RedLeaves", |c| {
    c.label = 1053123;
    c.weight = 0.1;
    c.on_create(|i| {
        i.item_id = 0x1E85;
        i.stackable = true;
        i.hue = 0x21;
        i.amount = 1;
    });
});

item("GreenThorns", |c| {
    c.label = 1060837;
    c.weight = 1.0;
    c.on_create(|i| {
        i.item_id = 0xF42;
        i.stackable = true;
        i.hue = 0x42;
        i.amount = 1;
    });
});

item("GreenThornsSHTeleporter", |c| {
    c.name = "a hole";
});

item("LesserPigmentsOfTokuno", |c| {
    c.script = "content/engines/lesser_pigments_of_tokuno.rhai";
    c.on_create(|i| {
        i.set = #{ UsesRemaining: 1 };
    });
});

item("MetalPigmentsOfTokuno", |c| {
    c.script = "content/engines/metal_pigments_of_tokuno.rhai";
    c.on_create(|i| {
        i.set = #{ UsesRemaining: 1 };
        i.pigment_label = -1;
    });
});

item("RepairDeed", |c| {
    c.on_create(|i| {
        i.item_id = 0x14F0;
        i.hue = 0x1BC;
        i.loot_type = "Blessed";
    });
});

item("MaabusCoffin", |c| {
    c.on_create(|i| {
        i.coffin_components = [
            [7211, 7211, -1, -1],
            [7446, 7212, 0, -1],
            [7447, 7213, 1, -1],
            [7505, 7214, 2, -1],
            [7502, 7210, 0, 0],
            [7501, 7209, 1, 0],
            [7500, 7208, 2, 0],
        ];
    });
});

item("CTFController", |c| {
    c.on_create(|i| {
        i.item_id = 0x1B7A;
        i.visible = false;
        i.movable = false;
    });
});

item("KHController", |c| {
    c.on_create(|i| {
        i.item_id = 0x1B7A;
        i.visible = false;
        i.movable = false;
        i.name = "King of the Hill Controller";
    });
});

item("DDWayPoint", |c| {
    c.on_create(|i| {
        i.item_id = 0x519;
        i.visible = true;
        i.name = "SET MY NAME";
    });
});

item("DungeonChampionSpawn", |c| {
    c.inherits = "ChampionSpawn";
});

item("LLChampionSpawn", |c| {
    c.inherits = "ChampionSpawn";
});

item("ChampionSkullPlatform", |c| {
    c.on_create(|i| {
        i.engine_addon_components = [
            [1818, -1, -1, -1],
            [1801, 0, -1, -1],
            [1801, 1, -1, -1],
            [1801, -1, 0, -1],
            [1801, 0, 0, -1],
            [1801, 1, 0, -1],
            [1801, -1, 1, -1],
            [1801, 0, 1, -1],
            [1819, 1, 1, -1],
        ];
        i.skull_braziers = [
            ["Power", 0, -1],
            ["Enlightenment", 1, -1],
            ["Venom", -1, 0],
            ["Pain", 1, 0],
            ["Greed", -1, 1],
            ["Death", 0, 1],
        ];
        i.addon_localized_components = [[8402, 1049495, 1154, 0, 0, 5], [3023, 1049496, 1154, 0, 2, -7], [3024, 1049497, 1154, 2, 0, -7]];
    });
});
