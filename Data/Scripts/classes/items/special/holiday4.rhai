item("LightOfTheWinterSolstice", |c| {
    c.weight = 1.0;
});

item("PileOfGlacialSnow", |c| {
    c.weight = 1.0;
    c.label = 1070874;
    c.on_create(|i| {
        i.item_id = 0x913;
        i.hue = 0x480;
    });
});

item("SnowPile", |c| {
    c.weight = 1.0;
    c.label = 1005578;
    c.on_create(|i| {
        i.item_id = 0x913;
        i.hue = 0x481;
    });
});

item("PumpkinScarecrow", |c| {
    c.label = 1096947;
});

item("RuinedTapestry", |c| {
    c.name = "Ruined Tapestry ";
});

item("ColoredSmallWebs", |c| {
    c.weight = 5.0;
});

item("DragonEasterEgg", |c| {
    c.label = 1097278;
    c.on_create(|i| {
        i.item_id = 0x47E6;
    });
});

item("CreepyCake", |c| {
    c.name = "Creepy Cake";
    c.on_create(|i| {
        i.item_id = 0x9E9;
        i.hue = 0x3E4;
    });
});

item("MrPlainsCookies", |c| {
    c.weight = 1.0;
    c.name = "Mr Plain's Cookies";
    c.on_create(|i| {
        i.item_id = 0x160C;
    });
});

item("PumpkinPizza", |c| {
    c.name = "Pumpkin Pizza";
    c.on_create(|i| {
        i.item_id = 0x1040;
        i.stackable = false;
    });
});

item("HarvestWine", |c| {
    c.name = "Harvest Wine";
    c.weight = 1.0;
});

item("MurkyMilk", |c| {
    c.name = "Murky Milk";
    c.weight = 1.0;
});

item("TwilightLantern", |c| {
    c.name = "Twilight Lantern";
});

item("CupidStatue", |c| {
    c.label = 1099220;
    c.on_create(|i| {
        i.item_id = 0x4F7D;
    });
});

item("CupidsArrow", |c| {
    c.label = 1152270;
    c.on_create(|i| {
        i.item_id = 0x4F7F;
    });
});

item("AnimatedHeartShapedBox", |c| {
    c.on_create(|i| {
        i.item_id = 0x49CA;
    });
});

item("HeartShapedBox", |c| {
    c.on_create(|i| {
        i.item_id = 0x49CA;
    });
});

item("DecorativeTopiary", |c| {
    c.weight = 1.0;
    c.on_create(|i| {
        i.item_id = 0x2378;
        i.loot_type = "Blessed";
    });
});

item("FestiveCactus", |c| {
    c.weight = 1.0;
    c.on_create(|i| {
        i.item_id = 0x2376;
        i.loot_type = "Blessed";
    });
});

item("SnowyTree", |c| {
    c.weight = 1.0;
    c.on_create(|i| {
        i.item_id = 0x2377;
        i.loot_type = "Blessed";
    });
});

item("GrimWarning", |c| {
    c.weight = 1.0;
    c.on_create(|i| {
        i.item_id = 0x42BD;
    });
});

item("SkullsOnPike", |c| {
    c.weight = 1.0;
    c.on_create(|i| {
        i.item_id = 0x42B5;
    });
});

item("ChairInAGhostCostume", |c| {
    c.weight = 5.0;
    c.on_create(|i| {
        i.item_id = 0x3F26;
    });
});

item("ExcellentIronMaiden", |c| {
    c.weight = 5.0;
    c.on_create(|i| {
        i.item_id = 0x3F15;
    });
});

item("HalloweenGuillotine", |c| {
    c.weight = 5.0;
    c.on_create(|i| {
        i.item_id = 0x3F27;
    });
});

item("PaintedDaemonMask", |c| {
    c.mask_name = "Daemon Mask";
    c.on_create(|i| {
        switch random_below(2) {
            0 => {
                i.item_id = 0x4A92;
            },
            1 => {
                i.item_id = 0x4A93;
            }
        }
    });
});

item("PaintedEvilClownMask", |c| {
    c.mask_name = "Evil Clown Mask";
    c.on_create(|i| {
        switch random_below(2) {
            0 => {
                i.item_id = 0x4A90;
            },
            1 => {
                i.item_id = 0x4A91;
            }
        }
    });
});

item("PaintedPlagueMask", |c| {
    c.mask_name = "Plague Mask";
    c.on_create(|i| {
        switch random_below(2) {
            0 => {
                i.item_id = 0x4A8E;
            },
            1 => {
                i.item_id = 0x4A8F;
            }
        }
    });
});

item("PaintedEvilJesterMask", |c| {
    c.mask_name = "Evil Jester Mask";
    c.on_create(|i| {
        switch random_below(2) {
            0 => {
                i.item_id = 0x4BA5;
            },
            1 => {
                i.item_id = 0x4BA6;
            }
        }
    });
});

item("PaintedPorcelainMask", |c| {
    c.mask_name = "Porcelain Mask";
    c.on_create(|i| {
        switch random_below(2) {
            0 => {
                i.item_id = 0x4BA7;
            },
            1 => {
                i.item_id = 0x4BA8;
            }
        }
    });
});

item("AngelDecoration", |c| {
    c.weight = 30.0;
    c.on_create(|i| {
        i.item_id = 0x46FA;
        i.loot_type = "Blessed";
    });
});

item("RockingHorse", |c| {
    c.weight = 30.0;
    c.on_create(|i| {
        i.item_id = 0x4214;
        i.loot_type = "Blessed";
    });
});

item("JellyBeans", |c| {
    c.label = 1096932;
    c.on_create(|i| {
        i.item_id = 0x468C;
        i.loot_type = "Blessed";
        i.stackable = true;
    });
});

item("Lollipops", |c| {
    c.on_create(|i| {
        switch random_below(3) {
            0 => {
                i.item_id = 0x468D;
            },
            1 => {
                i.item_id = 0x468E;
            },
            2 => {
                i.item_id = 0x468F;
            }
        }
        i.loot_type = "Blessed";
        i.stackable = true;
    });
});

item("NougatSwirl", |c| {
    c.label = 1096936;
    c.on_create(|i| {
        i.item_id = 0x4690;
        i.loot_type = "Blessed";
        i.stackable = true;
    });
});

item("Taffy", |c| {
    c.label = 1096949;
    c.on_create(|i| {
        i.item_id = 0x469D;
        i.loot_type = "Blessed";
        i.stackable = true;
    });
});

item("WrappedCandy", |c| {
    c.label = 1096950;
    c.on_create(|i| {
        i.item_id = 0x469E;
        i.loot_type = "Blessed";
        i.stackable = true;
    });
});
