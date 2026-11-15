mobile("Blacksmith", |c| {
    c.npc_sb_infos = ["SBBlacksmith", #{ "if": "tokunoVendor", then: ["SBSEArmor", "SBSEWeapons"] }];
    c.npc_guild = "BlacksmithsGuild";
    c.npc_shoe_type = "None";
    c.on_create(|m| {
        m.title = "the blacksmith";
        m.skill("ArmsLore", 36.0, 68.0);
        m.skill("Blacksmith", 65.0, 88.0);
        m.skill("Fencing", 60.0, 83.0);
        m.skill("Macing", 61.0, 93.0);
        m.skill("Swords", 60.0, 83.0);
        m.skill("Tactics", 60.0, 83.0);
        m.skill("Parry", 61.0, 93.0);
    });
    c.on_outfit(|m| {
        m.npc_vendor_outfit = true;
        if coin() {
            m.npc_equip_or_delete = #{ item: "Server.Items.FullApron" };
        } else {
            m.npc_equip_or_delete = #{ item: "Server.Items.RingmailChest" };
        }
        m.npc_add_item = ["Server.Items.Bascinet", "Server.Items.SmithHammer"];
    });
});

mobile("Weaponsmith", |c| {
    c.npc_sb_infos = ["SBWeaponSmith", #{ "if": "tokunoVendor", then: ["SBSEWeapons"] }];
    c.npc_shoe_type = ["Boots", "ThighBoots"];
    c.npc_shoe_hue = 0;
    c.on_create(|m| {
        m.title = "the weaponsmith";
        m.skill("ArmsLore", 64.0, 100.0);
        m.skill("Blacksmith", 65.0, 88.0);
        m.skill("Fencing", 45.0, 68.0);
        m.skill("Macing", 45.0, 68.0);
        m.skill("Swords", 45.0, 68.0);
        m.skill("Tactics", 36.0, 68.0);
    });
    c.on_outfit(|m| {
        m.npc_vendor_outfit = true;
        m.npc_add_item = ["Server.Items.HalfApron"];
    });
});

mobile("Tailor", |c| {
    c.npc_sb_infos = ["SBTailor"];
    c.npc_guild = "TailorsGuild";
    c.npc_shoe_type = ["Sandals", "Shoes"];
    c.on_create(|m| {
        m.title = "the tailor";
        m.skill("Tailoring", 64.0, 100.0);
    });
});

mobile("Weaver", |c| {
    c.npc_sb_infos = ["SBWeaver"];
    c.npc_guild = "TailorsGuild";
    c.npc_shoe_type = "Sandals";
    c.on_create(|m| {
        m.title = "the weaver";
        m.skill("Tailoring", 65.0, 88.0);
    });
});

mobile("IronWorker", |c| {
    c.npc_sb_infos = [
        "SBAxeWeapon",
        "SBKnifeWeapon",
        "SBMaceWeapon",
        "SBSmithTools",
        "SBPoleArmWeapon",
        "SBSpearForkWeapon",
        "SBSwordWeapon",
        "SBMetalShields",
        "SBHelmetArmor",
        "SBPlateArmor",
        "SBChainmailArmor",
        "SBRingmailArmor",
        "SBStuddedArmor",
        "SBLeatherArmor",
    ];
    c.npc_shoe_type = "None";
    c.on_create(|m| {
        m.title = "the iron worker";
        m.skill("ArmsLore", 36.0, 68.0);
        m.skill("Blacksmith", 65.0, 88.0);
        m.skill("Fencing", 60.0, 83.0);
        m.skill("Macing", 61.0, 93.0);
        m.skill("Swords", 60.0, 83.0);
        m.skill("Tactics", 60.0, 83.0);
        m.skill("Parry", 61.0, 93.0);
    });
    c.on_outfit(|m| {
        m.npc_vendor_outfit = true;
        if coin() {} else {
            m.npc_equip_or_delete = block([
                #{ item: "Server.Items.RingmailChest" },
                #{ "else": #{ npcAddItem: [#{ item: "Server.Items.FullApron", hue: "randomBright" }] } },
            ]);
        }
        m.npc_add_item = ["Server.Items.Bascinet", "Server.Items.SmithHammer"];
        m.npc_rehue_layers = block([
            #{ layers: ["Pants", "OuterLegs", "InnerLegs", "OuterTorso", "InnerTorso", "Shirt"] },
            #{ hue: "randomBright" },
        ]);
    });
});

mobile("Vagabond", |c| {
    c.npc_sb_infos = ["SBTinker", "SBVagabond"];
    c.on_create(|m| {
        m.title = "the vagabond";
        m.skill("ItemID", 60.0, 83.0);
    });
    c.on_outfit(|m| {
        m.npc_add_item = [
            #{ item: "Server.Items.FancyShirt", hue: "randomBright" },
            #{ item: "Server.Items.Shoes", hue: "vendorShoe" },
            #{ item: "Server.Items.LongPants", hue: "vendorRandom" },
        ];
        if coin() {
            m.npc_add_item = [#{ item: "Server.Items.Cloak", hue: "randomBright" }];
        } else {}
        m.npc_seq = [
            #{
                coin: #{
                    then: #{ npcAddItem: [#{ item: "Server.Items.SkullCap", hue: "randomNeutral" }] },
                    "else": #{ npcAddItem: [#{ item: "Server.Items.Bandana", hue: "randomNeutral" }] },
                },
            },
            block([#{ randomHair: true }, #{ npcFacialHair: "hairHue" }, #{ pack: [#{ gold: [100, 200] }] }]),
        ];
    });
});

mobile("KeeperOfChivalry", |c| {
    c.npc_sb_infos = ["SBKeeperOfChivalry"];
    c.on_create(|m| {
        m.title = "the Keeper of Chivalry";
        m.skill("Fencing", 75.0, 85.0);
        m.skill("Macing", 75.0, 85.0);
        m.skill("Swords", 75.0, 85.0);
        m.skill("Chivalry", 100.0);
    });
    c.on_outfit(|m| {
        m.npc_add_item = [
            "Server.Items.PlateArms",
            "Server.Items.PlateChest",
            "Server.Items.PlateGloves",
            "Server.Items.StuddedGorget",
            "Server.Items.PlateLegs",
        ];
        switch random_below(4) {
            0 => {
                m.npc_add_item = ["Server.Items.PlateHelm"];
            },
            1 => {
                m.npc_add_item = ["Server.Items.NorseHelm"];
            },
            2 => {
                m.npc_add_item = ["Server.Items.CloseHelm"];
            },
            3 => {
                m.npc_add_item = ["Server.Items.Helmet"];
            }
        }
        m.npc_seq = [
            #{
                oneOf: [
                    #{ npcAddItem: [#{ item: "Server.Items.BodySash", hue: 1154 }] },
                    #{ npcAddItem: [#{ item: "Server.Items.Doublet", hue: 1154 }] },
                    #{ npcAddItem: [#{ item: "Server.Items.Tunic", hue: 1154 }] },
                ],
            },
            block([
                #{ npcAddItem: ["Server.Items.Broadsword", #{ item: "Server.Items.MetalKiteShield", hueAfter: "randomNondyed" }] },
                #{ coin: #{
                    then: #{ npcAddItem: ["Server.Items.Boots"] },
                    "else": #{ npcAddItem: ["Server.Items.ThighBoots"] },
                } },
                #{ pack: [#{ gold: [100, 200] }] },
            ]),
        ];
    });
});

mobile("GypsyMaiden", |c| {
    c.npc_sb_infos = ["SBProvisioner"];
    c.on_create(|m| {
        m.title = "the gypsy maiden";
    });
    c.on_outfit(|m| {
        m.npc_vendor_outfit = true;
        switch random_below(4) {
            0 => {
                m.npc_add_item = [#{ item: "Server.Items.JesterHat", hue: "randomBright" }];
            },
            1 => {
                m.npc_add_item = [#{ item: "Server.Items.Bandana", hue: "randomBright" }];
            },
            2 => {
                m.npc_add_item = [#{ item: "Server.Items.SkullCap", hue: "randomBright" }];
            },
            3 => {}
        }
        if coin() {
            m.npc_add_item = [#{ item: "Server.Items.HalfApron", hue: "randomBright" }];
        } else {}
        m.npc_rehue_layers = block([#{ layers: ["Pants", "OuterLegs", "InnerLegs"] }, #{ hue: "randomBright" }]);
    });
});

mobile("HolyMage", |c| {
    c.npc_sb_infos = ["SBHolyMage"];
    c.on_create(|m| {
        m.title = "the Holy Mage";
        m.skill("EvalInt", 65.0, 88.0);
        m.skill("Inscribe", 60.0, 83.0);
        m.skill("Magery", 64.0, 100.0);
        m.skill("Meditation", 60.0, 83.0);
        m.skill("MagicResist", 65.0, 88.0);
        m.skill("Wrestling", 36.0, 68.0);
    });
    c.on_outfit(|m| {
        m.npc_add_item = [
            #{ item: "Server.Items.Robe", hue: 1150 },
            #{ item: "Server.Items.ThighBoots", hue: 1150 },
            #{ item: "Server.Items.BlackStaff", hue: 1150 },
        ];
        m.npc_by_gender = #{
            female: #{
                npcAddItem: [
                    #{ item: "Server.Items.LeatherGloves", hue: 1150 },
                    #{ item: "Server.Items.GoldNecklace", hue: 1150 },
                ],
                oneOf: [#{ npcHairItemId: 8252 }, #{ npcHairItemId: 8253 }],
            },
            male: #{
                npcAddItem: [#{ item: "Server.Items.PlateGloves", hue: 1150 }, #{ item: "Server.Items.PlateGorget", hue: 1150 }],
                oneOf: [#{ npcHairItemId: 8252 }],
            },
        };
        m.npc_hair_hue = 1150;
        m.pack = [#{ gold: [100, 200] }];
    });
});

mobile("Scribe", |c| {
    c.npc_sb_infos = [
        "SBScribe",
        #{
            era: "SE",
            then: [
                #{
                    "if": "tokunoVendor",
                    then: ["SBSamurai", "SBNinja"],
                    "else": [#{ era: "TOL", then: [#{ "if": "terMurVendor", then: ["SBSamurai", "SBNinja"] }] }],
                },
            ],
            "else": [#{ era: "TOL", then: [#{ "if": "terMurVendor", then: ["SBSamurai", "SBNinja"] }] }],
        },
    ];
    c.npc_guild = "MagesGuild";
    c.npc_shoe_type = ["Shoes", "Sandals"];
    c.on_create(|m| {
        m.title = "the scribe";
        m.skill("EvalInt", 60.0, 83.0);
        m.skill("Inscribe", 90.0, 100.0);
    });
    c.on_outfit(|m| {
        m.npc_vendor_outfit = true;
        m.npc_add_item = [#{ item: "Server.Items.Robe", hue: "randomNeutral" }];
    });
});

mobile("Banker", |c| {
    c.npc_sb_infos = ["SBBanker"];
    c.npc_guild = "MerchantsGuild";
    c.on_create(|m| {
        m.title = "the banker";
    });
});

mobile("Minter", |c| {
    c.on_create(|m| {
        m.title = "the minter";
    });
});

mobile("GypsyBanker", |c| {
    c.npc_guild = "None";
    c.on_create(|m| {
        m.title = "the gypsy banker";
    });
    c.on_outfit(|m| {
        m.npc_vendor_outfit = true;
        switch random_below(4) {
            0 => {
                m.npc_add_item = [#{ item: "Server.Items.JesterHat", hue: "randomBright" }];
            },
            1 => {
                m.npc_add_item = [#{ item: "Server.Items.Bandana", hue: "randomBright" }];
            },
            2 => {
                m.npc_add_item = [#{ item: "Server.Items.SkullCap", hue: "randomBright" }];
            },
            3 => {}
        }
        m.npc_rehue_layers = block([
            #{ layers: ["Pants", "Shoes", "OuterLegs", "InnerLegs", "OuterTorso", "InnerTorso", "Shirt"] },
            #{ hue: "randomBright" },
        ]);
    });
});

mobile("AnimalTrainer", |c| {
    c.npc_sb_infos = ["SBAnimalTrainer"];
    c.npc_shoe_type = #{ female: "ThighBoots", male: "Boots" };
    c.npc_shoe_hue = 0;
    c.on_create(|m| {
        m.title = "the animal trainer";
        m.skill("AnimalLore", 64.0, 100.0);
        m.skill("AnimalTaming", 90.0, 100.0);
        m.skill("Veterinary", 65.0, 88.0);
    });
    c.on_outfit(|m| {
        m.npc_vendor_outfit = true;
        if coin() {
            m.npc_add_item = ["Server.Items.QuarterStaff"];
        } else {
            m.npc_add_item = ["Server.Items.ShepherdsCrook"];
        }
    });
});

mobile("GypsyAnimalTrainer", |c| {
    c.inherits = "AnimalTrainer";
    c.on_create(|m| {
        if coin() {
            m.title = "the gypsy animal trainer";
        } else {
            m.title = "the gypsy animal herder";
        }
    });
    c.on_outfit(|m| {
        m.npc_vendor_outfit = true;
        if coin() {
            m.npc_add_item = ["Server.Items.QuarterStaff"];
        } else {
            m.npc_add_item = ["Server.Items.ShepherdsCrook"];
        }
        m.npc_rehue_layers = block([
            #{ layers: ["Pants", "OuterLegs", "InnerLegs", "OuterTorso", "InnerTorso", "Shirt"] },
            #{ hue: "randomBright" },
        ]);
    });
});

mobile("RealEstateBroker", |c| {
    c.npc_sb_infos = ["SBRealEstateBroker"];
    c.on_create(|m| {
        m.title = "the real estate broker";
    });
});
