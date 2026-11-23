mobile("PlayerVendor", |c| {
    c.on_create(|m| {
        if era("AOS") {} else {
            m.npc_name_hue = 53;
        }
        m.npc_init_stats = [100, 100, 25];
        m.npc_hue = "humanSkin";
        m.npc_speech_hue = 946;
        m.npc_seq = [
            #{ era: #{ atLeast: "AOS", "else": #{ npcNameHue: 53 } } },
            #{
                coin: #{
                    then: block([#{ female: true }, #{ body: 401 }, #{ nameRandom: "female" }]),
                    "else": block([#{ female: false }, #{ body: 400 }, #{ nameRandom: "male" }]),
                },
                npcAddItem: [
                    #{ item: "Server.Items.FancyShirt", hue: "randomNeutral", layer: "InnerTorso" },
                    #{ item: "Server.Items.LongPants", hue: "randomNeutral" },
                    #{ item: "Server.Items.BodySash", hue: "randomNeutral" },
                    #{ item: "Server.Items.Boots", hue: "randomNeutral" },
                    #{ item: "Server.Items.Cloak", hue: "randomNeutral" },
                ],
                randomHair: true,
            },
            #{ npcAddItem: [#{ item: "Server.Mobiles.VendorBackpack", movable: false }] },
        ];
    });
});

mobile("RentedVendor", |c| {
    c.inherits = "PlayerVendor";
});

mobile("PlayerBarkeeper", |c| {
    c.npc_shoe_type = ["ThighBoots", "Boots"];
    c.on_outfit(|m| {
        m.npc_vendor_outfit = true;
        m.npc_add_item = [#{ item: "Server.Items.HalfApron", hue: "randomBright" }];
        m.npc_delete_backpack = true;
    });
});

mobile("CustomHairstylist", |c| {
    c.npc_shoe_type = ["Shoes", "Sandals"];
    c.on_create(|m| {
        m.title = "the hairstylist";
    });
    c.on_outfit(|m| {
        m.npc_vendor_outfit = true;
        m.npc_add_item = [#{ item: "Server.Items.Robe", hue: "randomPink" }];
    });
});
