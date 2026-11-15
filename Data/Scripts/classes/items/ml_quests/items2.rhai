item("AnOldNecklace", |c| {
    c.label = 1075525;
    c.on_create(|i| {
        i.hue = 0x222;
    });
});

item("AnOldRing", |c| {
    c.label = 1075524;
    c.on_create(|i| {
        i.hue = 0x222;
    });
});

item("ReginasRing", |c| {
    c.label = 1075305;
    c.nontransferable = true;
    c.on_create(|i| {
        i.loot_type = "Blessed";
    });
});

item("TransparentHeart", |c| {
    c.label = 1075400;
    c.on_create(|i| {
        i.loot_type = "Blessed";
        i.hue = 0x4AB;
    });
});

item("AndrosGratitude", |c| {
    c.label = 1075345;
    c.on_create(|i| {
        i.loot_type = "Blessed";
    });
});

item("Bleach", |c| {
    c.label = 1075375;
    c.on_create(|i| {
        i.loot_type = "Blessed";
    });
});

item("MiniatureMushroom", |c| {
    c.label = 1073138;
    c.on_create(|i| {
        i.item_id = 0xD16;
        i.loot_type = "Blessed";
    });
});

item("PeppercornFishsteak", |c| {
    c.label = 1075557;
    c.on_create(|i| {
        i.hue = 0x222;
    });
});

item("PixieLeg", |c| {
    c.label = 1074613;
    c.on_create(|i| {
        i.loot_type = "Blessed";
        i.hue = 0x1C2;
    });
});

item("RedLeatherBook", |c| {
    c.on_create(|i| {
        i.hue = 0x485;
    });
});

item("PrismaticAmber", |c| {
    c.label = 1075299;
});

item("GustarShroud", |c| {
    c.name = " ";
    c.on_create(|i| {
        i.item_id = 0x2684;
        i.hue = 0x479;
    });
});

item("GiftForArielle", |c| {
    c.label = 1074356;
    c.default_gump_id = 65;
    c.nontransferable = true;
    c.on_create(|i| {
        i.item_id = 0x1882;
        i.hue = 0x2C4;
    });
});

item("PrismaticCrystal", |c| {
    c.label = 1074269;
    c.on_create(|i| {
        i.item_id = 0x2DA;
        i.movable = false;
        i.hue = 0x32;
    });
});

item("SmallBagOfTrinkets", |c| {
    c.on_create(|i| {
        i.item_id = 0xE76;
        i.reward_bag_fill = #{ count: 1, talismanChance: 0.0 };
    });
});

item("BagOfTrinkets", |c| {
    c.on_create(|i| {
        i.item_id = 0xE76;
        i.reward_bag_fill = #{ count: 2, talismanChance: 0.05 };
    });
});

item("BagOfTreasure", |c| {
    c.on_create(|i| {
        i.item_id = 0xE76;
        i.reward_bag_fill = #{ count: 3, talismanChance: 0.2 };
    });
});

item("LargeBagOfTreasure", |c| {
    c.on_create(|i| {
        i.item_id = 0xE76;
        i.reward_bag_fill = #{ count: 4, talismanChance: 0.5 };
    });
});

item("RewardStrongbox", |c| {
    c.on_create(|i| {
        i.item_id = 0x9AA;
        i.reward_bag_fill = #{ count: 5, talismanChance: 1.0 };
    });
});

item("BaseCraftmansSatchel", |c| {
    c.on_create(|i| {
        i.item_id = 0xE75;
        i.layer = "Backpack";
        i.quest_item_hue = "randomBrightHue";
    });
});

item("TailorSatchel", |c| {
    c.inherits = "BaseCraftmansSatchel";
    c.on_create(|i| {
        i.satchel_loot = ["MLArmor", "Jewelry", "Talisman"];
        if coin() {
            i.satchel_recipe = "Tailoring";
        } else {}
    });
});

item("BlacksmithSatchel", |c| {
    c.inherits = "BaseCraftmansSatchel";
    c.on_create(|i| {
        i.satchel_loot = ["MLWeapons", "Jewelry", "Talisman"];
        if coin() {
            i.satchel_recipe = "Blacksmithy";
        } else {}
    });
});

item("TinkerSatchel", |c| {
    c.inherits = "BaseCraftmansSatchel";
    c.on_create(|i| {
        i.satchel_loot = ["MLArmor", "MLWeapons", "MLRangedWeapons", "Jewelry", "Talisman"];
        if coin() {
            switch random_below(6) {
                0 => {
                    i.satchel_recipe = "Inscription";
                },
                1 => {
                    i.satchel_recipe = "Alchemy";
                },
                2 => {},
                3 => {},
                4 => {},
                5 => {}
            }
        } else {}
    });
});

item("FletchingSatchel", |c| {
    c.inherits = "BaseCraftmansSatchel";
    c.on_create(|i| {
        i.satchel_loot = ["MLRangedWeapons", "Jewelry", "Talisman"];
        if coin() {
            i.satchel_recipe = "BowFletching";
        } else {}
    });
});

item("CarpentrySatchel", |c| {
    c.inherits = "BaseCraftmansSatchel";
    c.on_create(|i| {
        i.satchel_loot = ["MLArmor", "MLWeapons", "MLRangedWeapons", "Jewelry", "Talisman"];
        if coin() {
            i.satchel_recipe = "Carpentry";
        } else {}
    });
});

item("BedlamTeleporter", |c| {
    c.label = 1074161;
    c.on_create(|i| {
        i.item_id = 0x124D;
        i.movable = false;
    });
});

item("NibbetSatchel", |c| {
    c.on_create(|i| {
        i.item_id = 0xE75;
        i.layer = "Backpack";
        i.quest_item_hue = "randomBrightHue";
        i.quest_item_drop = [["TinkerTools"]];
        switch random_below(10) {
            0 => {
                i.quest_item_drop = [["Springs", 3]];
            },
            1 => {
                i.quest_item_drop = [["Axle", 3]];
            },
            2 => {
                i.quest_item_drop = [["Hinge", 3]];
            },
            3 => {
                i.quest_item_drop = [["Key"]];
            },
            4 => {
                i.quest_item_drop = [["Scissors"]];
            },
            5 => {
                i.quest_item_drop = [["BarrelTap"]];
            },
            6 => {
                i.quest_item_drop = [["BarrelHoops"]];
            },
            7 => {
                i.quest_item_drop = [["Gears", 3]];
            },
            8 => {
                i.quest_item_drop = [["Lockpick", 3]];
            },
            9 => {
                i.quest_item_drop = [["ClockFrame", 3]];
            }
        }
    });
});
