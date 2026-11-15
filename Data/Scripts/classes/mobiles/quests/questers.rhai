mobile("AlbertaGiacco", |c| {
    c.name = "Alberta Giacco";
    c.on_create(|m| {
        m.mlq_init_stats = [100, 100, 25];
        m.hue = 0x83F2;
        m.female = true;
        m.body = 401;
        m.quest_equip = [
            #{ item: "Server.Items.FancyShirt" },
            #{ item: "Server.Items.Skirt", hue: 1435 },
            #{ item: "Server.Items.Boots" },
            #{ item: "Server.Items.FeatheredHat", hue: 1435 },
            #{ item: "Server.Items.FullApron", hue: 1435 },
        ];
        m.mlq_hair_item_id = 8253;
        m.mlq_hair_hue = 1111;
    });
});

mobile("ElwoodMcCarrin", |c| {
    c.name = "Elwood McCarrin";
    c.on_create(|m| {
        m.mlq_init_stats = [100, 100, 25];
        m.hue = 0x83ED;
        m.female = false;
        m.body = 400;
        m.quest_equip = [
            #{ item: "Server.Items.FancyShirt" },
            #{ item: "Server.Items.LongPants", hue: 1348 },
            #{ item: "Server.Items.Shoes", hue: 1108 },
            #{ item: "Server.Items.JesterHat", hue: 1234 },
            #{ item: "Server.Items.FullApron", hue: 1234 },
        ];
        m.mlq_hair_item_id = 8253;
        m.mlq_hair_hue = 1149;
        m.mlq_facial_hair_item_id = 8256;
        m.mlq_facial_hair_hue = 1149;
    });
});

mobile("GabrielPiete", |c| {
    c.name = "Gabriel Piete";
    c.on_create(|m| {
        m.mlq_init_stats = [100, 100, 25];
        m.hue = 0x83EF;
        m.female = false;
        m.body = 400;
        m.quest_equip = [
            #{ item: "Server.Items.FancyShirt" },
            #{ item: "Server.Items.LongPants", hue: 1527 },
            #{ item: "Server.Items.Shoes", hue: 1527 },
        ];
        m.mlq_hair_item_id = 8265;
        m.mlq_hair_hue = 1120;
        m.mlq_facial_hair_item_id = 8257;
        m.mlq_facial_hair_hue = 1120;
    });
});

mobile("Impresario", |c| {
    c.on_create(|m| {
        m.mlq_init_stats = [100, 100, 25];
        m.quest_skin_hue = true;
        m.female = false;
        m.body = 400;
        m.name = random_name("male");
        m.quest_equip = [
            #{ item: "Server.Items.FancyShirt", hue: "randomDyedHue" },
            #{ item: "Server.Items.LongPants", hue: "randomNondyedHue" },
            #{ item: "Server.Items.Shoes", hue: "randomNeutralHue" },
        ];
        m.random_hair = true;
        m.quest_random_facial_hair = true;
    });
});

mobile("TomasONeerlan", |c| {
    c.name = "Tomas O'Neerlan";
    c.on_create(|m| {
        m.mlq_init_stats = [100, 100, 25];
        m.hue = 0x83F8;
        m.female = false;
        m.body = 400;
        m.quest_equip = [
            #{ item: "Server.Items.FancyShirt" },
            #{ item: "Server.Items.LongPants", hue: 1350 },
            #{ item: "Server.Items.Boots", hue: 1106 },
            #{ item: "Server.Items.FullApron", hue: 1109 },
        ];
        m.mlq_hair_item_id = 8251;
        m.mlq_hair_hue = 1109;
    });
});

mobile("Horus", |c| {
    c.name = "Horus";
    c.on_create(|m| {
        m.mlq_init_stats = [100, 100, 25];
        m.hue = 0x83F3;
        m.body = 400;
        m.quest_equip = [
            #{ item: "Server.Items.PlateLegs", hue: 2121 },
            #{ item: "Server.Items.PlateChest", hue: 2121 },
            #{ item: "Server.Items.PlateArms", hue: 2121 },
            #{ item: "Server.Items.PlateGloves", hue: 2121 },
            #{ item: "Server.Items.PlateGorget", hue: 2121 },
            #{ item: "Server.Items.Bardiche", hue: 1154 },
            #{ item: "Server.Items.Boots", hue: 1 },
            #{ item: "Server.Items.Cloak", hue: 1154 },
        ];
        m.quest_hair_item = true;
        m.quest_facial_hair_item = true;
    });
});

mobile("Maabus", |c| {
    c.name = "Maabus";
    c.on_create(|m| {
        m.body = 148;
    });
});

mobile("Mardoth", |c| {
    c.name = "Mardoth";
    c.on_create(|m| {
        m.mlq_init_stats = [100, 100, 25];
        m.hue = 0x8849;
        m.body = 400;
        m.quest_equip = [
            #{ item: "Server.Items.Sandals", hue: 1 },
            #{ item: "Server.Items.Robe", hue: 1645 },
            #{ item: "Server.Items.BlackStaff" },
            #{ item: "Server.Items.WizardsHat", hue: 1 },
            #{ item: "Server.Items.BoneGloves", hue: 1645 },
            #{ item: "Server.Items.PlateGorget", hue: 1 },
        ];
        m.mlq_facial_hair_item_id = 8257;
        m.mlq_facial_hair_hue = 1154;
        m.mlq_hair_item_id = 8252;
        m.mlq_hair_hue = 1154;
    });
});

mobile("Emino", |c| {
    c.name = "Daimyo Emino";
    c.on_create(|m| {
        m.mlq_init_stats = [100, 100, 25];
        m.hue = 0x83FE;
        m.female = false;
        m.body = 400;
        m.mlq_hair_item_id = 8251;
        m.mlq_hair_hue = 2305;
        m.quest_equip = [
            #{ item: "Server.Items.MaleKimono" },
            #{ item: "Server.Items.SamuraiTabi" },
            #{ item: "Server.Items.Bandana" },
            #{ item: "Server.Items.PlateHaidate" },
            #{ item: "Server.Items.PlateDo" },
            #{ item: "Server.Items.PlateHiroSode" },
            #{ item: "Server.Items.Nunchaku", movable: false },
        ];
    });
});

mobile("EnshroudedFigure", |c| {
    c.name = "an enshrouded figure";
    c.on_create(|m| {
        m.mlq_init_stats = [100, 100, 25];
        m.hue = 0x8401;
        m.female = false;
        m.body = 400;
        m.quest_equip = [#{ item: "Server.Items.DeathShroud" }, #{ item: "Server.Items.ThighBoots" }];
    });
});

mobile("JedahEntille", |c| {
    c.name = "Jedah Entille";
    c.on_create(|m| {
        m.mlq_init_stats = [100, 100, 25];
        m.hue = 0x83FE;
        m.female = true;
        m.body = 401;
        m.mlq_hair_item_id = 8252;
        m.mlq_hair_hue = 1726;
        m.quest_equip = [
            #{ item: "Server.Items.PlainDress", hue: 1320 },
            #{ item: "Server.Items.ThighBoots" },
            #{ item: "Server.Items.FloppyHat" },
        ];
    });
});

mobile("Zoel", |c| {
    c.name = "Elite Ninja Zoel";
    c.on_create(|m| {
        m.mlq_init_stats = [100, 100, 25];
        m.hue = 0x83FE;
        m.female = false;
        m.body = 400;
        m.mlq_hair_item_id = 8251;
        m.mlq_hair_hue = 2305;
        m.quest_equip = [
            #{ item: "Server.Items.HakamaShita", hue: 1 },
            #{ item: "Server.Items.NinjaTabi" },
            #{ item: "Server.Items.TattsukeHakama" },
            #{ item: "Server.Items.Bandana" },
            #{ item: "Server.Items.LeatherNinjaBelt" },
            #{ item: "Server.Items.Tekagi", movable: false },
        ];
    });
});

mobile("Haochi", |c| {
    c.name = "Daimyo Haochi";
    c.on_create(|m| {
        m.mlq_init_stats = [100, 100, 25];
        m.hue = 0x8403;
        m.female = false;
        m.body = 400;
        m.mlq_hair_item_id = 8266;
        m.mlq_hair_hue = 2305;
        m.quest_equip = [
            #{ item: "Server.Items.SamuraiTabi" },
            #{ item: "Server.Items.JinBaori" },
            #{ item: "Server.Items.PlateHaidate" },
            #{ item: "Server.Items.StandardPlateKabuto" },
            #{ item: "Server.Items.PlateMempo" },
            #{ item: "Server.Items.PlateDo" },
            #{ item: "Server.Items.PlateHiroSode" },
        ];
    });
});

mobile("Relnia", |c| {
    c.name = "Disheveled Relnia";
    c.on_create(|m| {
        m.mlq_init_stats = [100, 100, 25];
        m.hue = 0x83FF;
        m.female = true;
        m.body = 401;
        m.mlq_hair_item_id = 8252;
        m.mlq_hair_hue = 1620;
        m.quest_equip = [
            #{ item: "Server.Items.ThighBoots", hue: 2305 },
            #{ item: "Server.Items.FancyShirt", hue: 1523 },
            #{ item: "Server.Items.SkullCap", hue: 1703 },
            #{ item: "Server.Items.Skirt", hue: 1348 },
        ];
    });
});

mobile("AnsellaGryen", |c| {
    c.name = "Ansella Gryen";
    c.on_create(|m| {
        m.mlq_init_stats = [100, 100, 25];
        m.hue = 0x83EA;
        m.female = true;
        m.body = 401;
        m.mlq_hair_item_id = 8251;
        m.mlq_hair_hue = 443;
        m.quest_equip = [
            #{ item: "Server.Items.SamuraiTabi", hue: 2301 },
            #{ item: "Server.Items.FemaleKimono", hue: 1206 },
            #{ item: "Server.Items.Obi", hue: 1318 },
            #{ item: "Server.Items.GoldBracelet" },
        ];
    });
});

mobile("Uzeraan", |c| {
    c.name = "Uzeraan";
    c.on_create(|m| {
        m.mlq_init_stats = [100, 100, 25];
        m.hue = 0x83F3;
        m.female = false;
        m.body = 400;
        m.quest_equip = [
            #{ item: "Server.Items.Robe", hue: 1245 },
            #{ item: "Server.Items.WizardsHat", hue: 2213 },
            #{ item: "Server.Items.Shoes", hue: 2213 },
            #{ item: "Server.Items.BlackStaff", movable: false },
        ];
        m.mlq_hair_item_id = 8252;
        m.mlq_hair_hue = 1109;
        m.mlq_facial_hair_item_id = 8254;
        m.mlq_facial_hair_hue = 1109;
    });
});

mobile("Schmendrick", |c| {
    c.name = "Schmendrick";
    c.on_create(|m| {
        m.mlq_init_stats = [100, 100, 25];
        m.hue = 0x83F3;
        m.female = false;
        m.body = 400;
        m.quest_equip = [
            #{ item: "Server.Items.Robe", hue: 1245 },
            #{ item: "Server.Items.WizardsHat", hue: 1154 },
            #{ item: "Server.Items.Shoes", hue: 1154 },
            #{ item: "Server.Items.GlacialStaff", movable: false },
            #{ item: "Server.Items.Backpack", movable: false },
        ];
        m.mlq_hair_item_id = 8252;
        m.mlq_hair_hue = 1109;
        m.mlq_facial_hair_item_id = 8254;
        m.mlq_facial_hair_hue = 1109;
    });
});

mobile("Blackheart", |c| {
    c.name = "Captain Blackheart";
    c.on_create(|m| {
        m.mlq_init_stats = [100, 100, 25];
        m.hue = 0x83EF;
        m.female = false;
        m.body = 400;
        m.quest_equip = [
            #{ item: "Server.Items.FancyShirt" },
            #{ item: "Server.Items.LongPants", hue: 1645 },
            #{ item: "Server.Items.ThighBoots" },
            #{ item: "Server.Items.TricorneHat", hue: 1 },
            #{ item: "Server.Items.BodySash", hue: 1645 },
            #{ item: "Server.Items.LeatherGloves", hue: 1645 },
            #{ item: "Server.Items.Cutlass", movable: false },
        ];
        m.mlq_facial_hair_item_id = 8254;
        m.mlq_facial_hair_hue = 1109;
    });
});

mobile("Grizelda", |c| {
    c.name = "Grizelda";
    c.on_create(|m| {
        m.mlq_init_stats = [100, 100, 25];
        m.hue = 0x83EA;
        m.female = true;
        m.body = 401;
        m.quest_equip = [
            #{ item: "Server.Items.Robe", hue: 1 },
            #{ item: "Server.Items.Sandals" },
            #{ item: "Server.Items.WizardsHat", hue: 1 },
            #{ item: "Server.Items.GoldBracelet" },
            #{ item: "Server.Items.GnarledStaff", movable: false },
        ];
        m.mlq_hair_item_id = 8252;
    });
});

mobile("Zeefzorpul", |c| {
    c.name = "Zeefzorpul";
    c.on_create(|m| {
        m.body = 74;
    });
});

mobile("HiddenFigure", |c| {
    c.on_create(|m| {
        m.mlq_init_stats = [100, 100, 25];
        m.quest_skin_hue = true;
        if coin() {
            m.female = true;
            m.body = 401;
            m.name = random_name("female");
        } else {
            m.female = false;
            m.body = 400;
            m.name = random_name("male");
        }
        m.random_hair = true;
        m.quest_equip = [
            #{ item: "Server.Items.TattsukeHakama", hue: "vendorRandomHue" },
            #{ item: "Server.Items.Kasa" },
            #{ item: "Server.Items.HakamaShita", hue: "vendorRandomHue" },
        ];
        m.npc_seq = [
            #{
                coin: #{
                    then: #{ questEquip: [#{ item: "Server.Items.Shoes", hue: "vendorShoeHue" }] },
                    "else": #{ questEquip: [#{ item: "Server.Items.Sandals", hue: "vendorShoeHue" }] },
                },
            },
            #{ oneOf: [#{ hiddenFigureMessage: 1063191 }, #{ hiddenFigureMessage: 1063192 }] },
        ];
    });
});

mobile("HaochisGuardsman", |c| {
    c.on_create(|m| {
        m.mlq_init_stats = [100, 100, 25];
        m.quest_skin_hue = true;
        m.female = false;
        m.body = 400;
        m.name = random_name("male");
        m.random_hair = true;
        m.quest_equip = [
            #{ item: "Server.Items.LeatherDo" },
            #{ item: "Server.Items.LeatherHiroSode" },
            #{ item: "Server.Items.SamuraiTabi", hue: "randomNondyedHue" },
            #{
                oneOf: ["Server.Items.StuddedHaidate", "Server.Items.PlateSuneate", "Server.Items.LeatherSuneate"],
            },
            #{
                oneOf: [
                    "Server.Items.DecorativePlateKabuto",
                    "Server.Items.ChainHatsuburi",
                    "Server.Items.LightPlateJingasa",
                    "Server.Items.LeatherJingasa",
                ],
            },
            block([
                #{ oneOf: ["Server.Items.NoDachi", "Server.Items.Lajatang", "Server.Items.Wakizashi"] },
                #{ movable: false },
            ]),
        ];
    });
});

mobile("Naturalist", |c| {
    c.on_create(|m| {
        m.mlq_init_stats = [100, 100, 25];
        m.quest_skin_hue = true;
        m.female = false;
        m.body = 400;
        m.name = random_name("male");
        m.quest_equip = [
            #{ item: "Server.Items.Tunic", hue: 1432 },
            #{ item: "Server.Items.LongPants", hue: 1435 },
            #{ item: "Server.Items.Boots" },
        ];
        m.random_hair = true;
        m.quest_facial_hair_of_hair_hue = true;
    });
});

mobile("Chyloth", |c| {
    c.name = "Chyloth";
    c.on_create(|m| {
        m.mlq_init_stats = [100, 100, 25];
        m.hue = 0x8455;
        m.body = 400;
        m.quest_equip = [
            #{ item: "Server.Engines.Quests.Doom.ChylothShroud", equip: true },
            #{ item: "Server.Engines.Quests.Doom.ChylothStaff", equip: true },
        ];
    });
});

mobile("Victoria", |c| {
    c.name = "Victoria";
    c.on_create(|m| {
        m.mlq_init_stats = [100, 100, 25];
        m.female = true;
        m.hue = 0x8835;
        m.body = 401;
        m.quest_equip = [
            #{ item: "Server.Engines.Quests.Doom.GrandGrimoire", equip: true },
            #{ item: "Server.Items.Sandals", hue: 1109, equip: true },
            #{ item: "Server.Items.SkullCap", hue: 1109, equip: true },
            #{ item: "Server.Items.PlainDress", hue: 1109, equip: true },
        ];
        m.mlq_hair_item_id = 8252;
        m.mlq_hair_hue = 1154;
    });
});

mobile("MansionGuard", |c| {
    c.on_create(|m| {
        m.mlq_init_stats = [100, 25, 25];
        m.quest_skin_hue = true;
        m.female = false;
        m.body = 400;
        m.name = random_name("male");
        m.quest_equip = [
            #{ item: "Server.Items.PlateChest" },
            #{ item: "Server.Items.PlateArms" },
            #{ item: "Server.Items.PlateGloves" },
            #{ item: "Server.Items.PlateLegs" },
        ];
        m.random_hair = true;
        m.quest_facial_hair_of_hair_hue = true;
        m.npc_seq = [#{ questEquip: [#{ item: "Server.Items.Bardiche", movable: false }] }];
    });
});

mobile("MilitiaCanoneer", |c| {
    c.on_create(|m| {
        m.mlq_init_stats = [100, 125, 25];
        m.quest_skin_hue = true;
        m.female = false;
        m.body = 400;
        m.name = random_name("male");
        m.random_hair = true;
        m.quest_facial_hair_of_hair_hue = true;
        m.quest_equip = [
            #{ item: "Server.Items.PlateChest" },
            #{ item: "Server.Items.PlateArms" },
            #{ item: "Server.Items.PlateGloves" },
            #{ item: "Server.Items.PlateLegs" },
            #{ item: "Server.Items.Torch", movable: false, ignite: true },
        ];
    });
});

mobile("MilitiaFighter", |c| {
    c.on_create(|m| {
        m.mlq_init_stats = [40, 30, 5];
        m.title = "the Militia Fighter";
        m.npc_speech_hue = "randomDyed";
        m.quest_skin_hue = true;
        m.female = false;
        m.body = 400;
        m.name = random_name("male");
        m.random_hair = true;
        m.quest_facial_hair_of_hair_hue = true;
        m.quest_equip = [
            #{ item: "Server.Items.ThighBoots", hue: 443 },
            #{ item: "Server.Items.LeatherChest" },
            #{ item: "Server.Items.LeatherArms" },
            #{ item: "Server.Items.LeatherLegs" },
            #{ item: "Server.Items.LeatherCap" },
            #{ item: "Server.Items.LeatherGloves" },
            #{ item: "Server.Items.LeatherGorget" },
            block([
                #{ oneOf: [
                    "Server.Items.Broadsword",
                    "Server.Items.Cutlass",
                    "Server.Items.Katana",
                    "Server.Items.Longsword",
                    "Server.Items.Scimitar",
                    "Server.Items.VikingSword",
                ] },
                #{ movable: false },
            ]),
            #{ item: "Server.Items.BronzeShield", movable: false },
        ];
        m.skill("Swords", 20.0);
    });
});
