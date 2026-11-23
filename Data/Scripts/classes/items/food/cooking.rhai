item("SackFlour", |c| {
    c.weight = 5.0;
    c.on_create(|i| {
        i.item_id = 0x1039;
    });
});

item("WheatSheaf", |c| {
    c.weight = 1.0;
    c.on_create(|i| {
        i.item_id = 0x1EBD;
        i.stackable = true;
    });
});

item("WoodenBowlOfCarrots", |c| {
    c.weight = 1.0;
    c.empty_bowl = "Server.Items.EmptyWoodenBowl";
    c.on_create(|i| {
        i.item_id = 0x15F9;
        i.stackable = false;
        i.food_fill_factor = 2;
    });
});

item("WoodenBowlOfCorn", |c| {
    c.weight = 1.0;
    c.empty_bowl = "Server.Items.EmptyWoodenBowl";
    c.on_create(|i| {
        i.item_id = 0x15FA;
        i.stackable = false;
        i.food_fill_factor = 2;
    });
});

item("WoodenBowlOfLettuce", |c| {
    c.weight = 1.0;
    c.empty_bowl = "Server.Items.EmptyWoodenBowl";
    c.on_create(|i| {
        i.item_id = 0x15FB;
        i.stackable = false;
        i.food_fill_factor = 2;
    });
});

item("WoodenBowlOfPeas", |c| {
    c.weight = 1.0;
    c.empty_bowl = "Server.Items.EmptyWoodenBowl";
    c.on_create(|i| {
        i.item_id = 0x15FC;
        i.stackable = false;
        i.food_fill_factor = 2;
    });
});

item("PewterBowlOfCarrots", |c| {
    c.weight = 1.0;
    c.empty_bowl = "Server.Items.EmptyPewterBowl";
    c.on_create(|i| {
        i.item_id = 0x15FE;
        i.stackable = false;
        i.food_fill_factor = 2;
    });
});

item("PewterBowlOfCorn", |c| {
    c.weight = 1.0;
    c.empty_bowl = "Server.Items.EmptyPewterBowl";
    c.on_create(|i| {
        i.item_id = 0x15FF;
        i.stackable = false;
        i.food_fill_factor = 2;
    });
});

item("PewterBowlOfLettuce", |c| {
    c.weight = 1.0;
    c.empty_bowl = "Server.Items.EmptyPewterBowl";
    c.on_create(|i| {
        i.item_id = 0x1600;
        i.stackable = false;
        i.food_fill_factor = 2;
    });
});

item("PewterBowlOfPeas", |c| {
    c.weight = 1.0;
    c.empty_bowl = "Server.Items.EmptyPewterBowl";
    c.on_create(|i| {
        i.item_id = 0x1601;
        i.stackable = false;
        i.food_fill_factor = 2;
    });
});

item("PewterBowlOfPotatos", |c| {
    c.weight = 1.0;
    c.empty_bowl = "Server.Items.EmptyPewterBowl";
    c.on_create(|i| {
        i.item_id = 0x1602;
        i.stackable = false;
        i.food_fill_factor = 2;
    });
});

item("WoodenBowlOfStew", |c| {
    c.weight = 2.0;
    c.empty_bowl = "Server.Items.EmptyWoodenTub";
    c.on_create(|i| {
        i.item_id = 0x1604;
        i.stackable = false;
        i.food_fill_factor = 2;
    });
});

item("WoodenBowlOfTomatoSoup", |c| {
    c.weight = 2.0;
    c.empty_bowl = "Server.Items.EmptyWoodenTub";
    c.on_create(|i| {
        i.item_id = 0x1606;
        i.stackable = false;
        i.food_fill_factor = 2;
    });
});

item("RawRibs", |c| {
    c.weight = 1.0;
    c.cooked = "Server.Items.Ribs";
    c.on_create(|i| {
        i.item_id = 0x9F1;
        i.cooking_level = 10;
        i.stackable = true;
    });
});

item("RawLambLeg", |c| {
    c.cooked = "Server.Items.LambLeg";
    c.on_create(|i| {
        i.item_id = 0x1609;
        i.cooking_level = 10;
        i.stackable = true;
    });
});

item("RawChickenLeg", |c| {
    c.weight = 1.0;
    c.cooked = "Server.Items.ChickenLeg";
    c.on_create(|i| {
        i.item_id = 0x1607;
        i.cooking_level = 10;
        i.stackable = true;
    });
});

item("RawBird", |c| {
    c.weight = 1.0;
    c.cooked = "Server.Items.CookedBird";
    c.on_create(|i| {
        i.item_id = 0x9B9;
        i.cooking_level = 10;
        i.stackable = true;
    });
});

item("UnbakedPeachCobbler", |c| {
    c.weight = 1.0;
    c.label = 1041335;
    c.cooked = "Server.Items.PeachCobbler";
    c.on_create(|i| {
        i.item_id = 0x1042;
        i.cooking_level = 25;
    });
});

item("UnbakedFruitPie", |c| {
    c.weight = 1.0;
    c.label = 1041334;
    c.cooked = "Server.Items.FruitPie";
    c.on_create(|i| {
        i.item_id = 0x1042;
        i.cooking_level = 25;
    });
});

item("UnbakedMeatPie", |c| {
    c.weight = 1.0;
    c.label = 1041338;
    c.cooked = "Server.Items.MeatPie";
    c.on_create(|i| {
        i.item_id = 0x1042;
        i.cooking_level = 25;
    });
});

item("UnbakedPumpkinPie", |c| {
    c.weight = 1.0;
    c.label = 1041342;
    c.cooked = "Server.Items.PumpkinPie";
    c.on_create(|i| {
        i.item_id = 0x1042;
        i.cooking_level = 25;
    });
});

item("UnbakedApplePie", |c| {
    c.weight = 1.0;
    c.label = 1041336;
    c.cooked = "Server.Items.ApplePie";
    c.on_create(|i| {
        i.item_id = 0x1042;
        i.cooking_level = 25;
    });
});

item("UncookedCheesePizza", |c| {
    c.weight = 1.0;
    c.label = 1041341;
    c.cooked = "Server.Items.CheesePizza";
    c.on_create(|i| {
        i.item_id = 0x1083;
        i.cooking_level = 20;
    });
});

item("UncookedSausagePizza", |c| {
    c.weight = 1.0;
    c.label = 1041337;
    c.cooked = "Server.Items.SausagePizza";
    c.on_create(|i| {
        i.item_id = 0x1083;
        i.cooking_level = 20;
    });
});

item("UnbakedQuiche", |c| {
    c.weight = 1.0;
    c.label = 1041339;
    c.cooked = "Server.Items.Quiche";
    c.on_create(|i| {
        i.item_id = 0x1042;
        i.cooking_level = 25;
    });
});

item("Eggs", |c| {
    c.weight = 1.0;
    c.cooked = "Server.Items.FriedEggs";
    c.on_create(|i| {
        i.item_id = 0x9B5;
        i.cooking_level = 15;
        i.stackable = true;
    });
});

item("BrightlyColoredEggs", |c| {
    c.weight = 0.5;
    c.name = "brightly colored eggs";
    c.cooked = "Server.Items.FriedEggs";
    c.on_create(|i| {
        i.item_id = 0x9B5;
        i.cooking_level = 15;
        switch random_below(20) {
            0 => {
                i.hue = 3;
            },
            1 => {
                i.hue = 8;
            },
            2 => {
                i.hue = 0xD;
            },
            3 => {
                i.hue = 0x12;
            },
            4 => {
                i.hue = 0x17;
            },
            5 => {
                i.hue = 0x1C;
            },
            6 => {
                i.hue = 0x21;
            },
            7 => {
                i.hue = 0x26;
            },
            8 => {
                i.hue = 0x2B;
            },
            9 => {
                i.hue = 0x30;
            },
            10 => {
                i.hue = 0x35;
            },
            11 => {
                i.hue = 0x3A;
            },
            12 => {
                i.hue = 0x3F;
            },
            13 => {
                i.hue = 0x44;
            },
            14 => {
                i.hue = 0x49;
            },
            15 => {
                i.hue = 0x4E;
            },
            16 => {
                i.hue = 0x53;
            },
            17 => {
                i.hue = 0x58;
            },
            18 => {
                i.hue = 0x5D;
            },
            19 => {
                i.hue = 0x62;
            }
        }
    });
});

item("EasterEggs", |c| {
    c.weight = 0.5;
    c.label = 1016105;
    c.cooked = "Server.Items.FriedEggs";
    c.on_create(|i| {
        i.item_id = 0x9B5;
        i.cooking_level = 15;
        switch random_below(20) {
            0 => {
                i.hue = 3;
            },
            1 => {
                i.hue = 8;
            },
            2 => {
                i.hue = 0xD;
            },
            3 => {
                i.hue = 0x12;
            },
            4 => {
                i.hue = 0x17;
            },
            5 => {
                i.hue = 0x1C;
            },
            6 => {
                i.hue = 0x21;
            },
            7 => {
                i.hue = 0x26;
            },
            8 => {
                i.hue = 0x2B;
            },
            9 => {
                i.hue = 0x30;
            },
            10 => {
                i.hue = 0x35;
            },
            11 => {
                i.hue = 0x3A;
            },
            12 => {
                i.hue = 0x3F;
            },
            13 => {
                i.hue = 0x44;
            },
            14 => {
                i.hue = 0x49;
            },
            15 => {
                i.hue = 0x4E;
            },
            16 => {
                i.hue = 0x53;
            },
            17 => {
                i.hue = 0x58;
            },
            18 => {
                i.hue = 0x5D;
            },
            19 => {
                i.hue = 0x62;
            }
        }
    });
});

item("CookieMix", |c| {
    c.weight = 1.0;
    c.cooked = "Server.Items.Cookies";
    c.on_create(|i| {
        i.item_id = 0x103F;
        i.cooking_level = 20;
    });
});

item("CakeMix", |c| {
    c.weight = 1.0;
    c.label = 1041002;
    c.cooked = "Server.Items.Cake";
    c.on_create(|i| {
        i.item_id = 0x103F;
        i.cooking_level = 40;
    });
});

item("RawFishSteak", |c| {
    c.weight = 0.1;
    c.cooked = "Server.Items.FishSteak";
    c.on_create(|i| {
        i.item_id = 0x97A;
        i.cooking_level = 10;
        i.stackable = true;
    });
});

item("Dough", |c| {
    c.weight = 1.0;
    c.on_create(|i| {
        i.item_id = 0x103D;
        if era("ML") {
            i.stackable = true;
        } else {
            i.stackable = false;
        }
    });
});

item("SweetDough", |c| {
    c.weight = 1.0;
    c.label = 1041340;
    c.on_create(|i| {
        i.item_id = 0x103D;
        if era("ML") {
            i.stackable = true;
        } else {
            i.stackable = false;
        }
        i.hue = 0x96;
    });
});

item("JarHoney", |c| {
    c.weight = 1.0;
    c.on_create(|i| {
        i.item_id = 0x9EC;
        i.stackable = true;
    });
});

item("BowlFlour", |c| {
    c.weight = 1.0;
    c.on_create(|i| {
        i.item_id = 0xA1E;
    });
});
