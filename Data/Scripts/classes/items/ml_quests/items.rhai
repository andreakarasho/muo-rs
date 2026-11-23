item("ABauble", |c| {
    c.label = 1073137;
    c.on_create(|i| {
        i.item_id = 0x23B;
        i.loot_type = "Blessed";
    });
});

item("AlchemistsBandage", |c| {
    c.label = 1075452;
    c.nontransferable = true;
    c.on_create(|i| {
        i.item_id = 0xE21;
        i.loot_type = "Blessed";
        i.hue = 0x482;
    });
});

item("AndricSatchel", |c| {
    c.on_create(|i| {
        i.item_id = 0xE75;
        i.layer = "Backpack";
        i.quest_item_hue = "randomBrightHue";
        i.quest_item_drop = [["Feather", 10], ["FletcherTools"]];
    });
});

item("AsandosSatchel", |c| {
    c.on_create(|i| {
        i.item_id = 0xE75;
        i.layer = "Backpack";
        i.quest_item_hue = "randomBrightHue";
        i.quest_item_drop = [["SackFlour"], ["Skillet"]];
    });
});

item("BasinOfCrystalClearWater", |c| {
    c.label = 1075303;
    c.nontransferable = true;
    c.on_create(|i| {
        i.item_id = 0x1008;
        i.loot_type = "Blessed";
    });
});

item("BridesLetter", |c| {
    c.label = 1075301;
    c.nontransferable = true;
    c.on_create(|i| {
        i.item_id = 0x14ED;
        i.loot_type = "Blessed";
    });
});

item("CompletedTuitionReimbursementForm", |c| {
    c.label = 1074625;
    c.nontransferable = true;
    c.on_create(|i| {
        i.item_id = 0x14F0;
        i.loot_type = "Blessed";
    });
});

item("DreadSpiderSilk", |c| {
    c.label = 1075319;
    c.nontransferable = true;
    c.on_create(|i| {
        i.item_id = 0xDF8;
        i.loot_type = "Blessed";
        i.hue = 0x481;
    });
});

item("FragmentOfAMap", |c| {
    c.label = 1074533;
    c.on_create(|i| {
        i.item_id = 0x14ED;
        i.loot_type = "Blessed";
    });
});

item("FragmentOfAMapDelivery", |c| {
    c.label = 1074533;
    c.nontransferable = true;
    c.on_create(|i| {
        i.item_id = 0x14ED;
        i.loot_type = "Blessed";
    });
});

item("FriendOfTheLibraryToken", |c| {
    c.label = 1073136;
    c.on_create(|i| {
        i.item_id = 0x2F58;
        i.layer = "Talisman";
        i.hue = 0x28A;
    });
});

item("FriendsOfTheLibraryApplication", |c| {
    c.label = 1073131;
    c.nontransferable = true;
    c.on_create(|i| {
        i.item_id = 0xEC0;
        i.loot_type = "Blessed";
    });
});

item("GamanHorns", |c| {
    c.label = 1074557;
    c.on_create(|i| {
        i.item_id = 0x1084;
        i.loot_type = "Blessed";
        i.stackable = true;
        i.hue = 0x395;
    });
});

item("GervisSatchel", |c| {
    c.on_create(|i| {
        i.item_id = 0xE75;
        i.layer = "Backpack";
        i.quest_item_hue = "randomBrightHue";
        i.quest_item_drop = [["IronIngot", 10], ["SmithHammer"]];
    });
});

item("HargroveSatchel", |c| {
    c.on_create(|i| {
        i.item_id = 0xE75;
        i.layer = "Backpack";
        i.quest_item_hue = "randomBrightHue";
        i.quest_item_drop = [["Gold", 15], ["Hatchet"]];
    });
});

item("KirinBrains", |c| {
    c.label = 1074612;
    c.on_create(|i| {
        i.item_id = 0x1CF0;
        i.loot_type = "Blessed";
        i.hue = 0xD7;
    });
});

item("LowelSatchel", |c| {
    c.on_create(|i| {
        i.item_id = 0xE75;
        i.layer = "Backpack";
        i.quest_item_hue = "randomBrightHue";
        i.quest_item_drop = [["Board", 10], ["DovetailSaw"]];
    });
});

item("MirrorOfPurification", |c| {
    c.label = 1075304;
    c.on_create(|i| {
        i.item_id = 0x1008;
        i.loot_type = "Blessed";
        i.hue = 0x530;
    });
});

item("MuggSatchel", |c| {
    c.on_create(|i| {
        i.item_id = 0xE75;
        i.layer = "Backpack";
        i.quest_item_hue = "randomBrightHue";
        i.quest_item_drop = [["Pickaxe"], ["Pickaxe"]];
    });
});

item("NotarizedApplication", |c| {
    c.label = 1073135;
    c.nontransferable = true;
    c.on_create(|i| {
        i.item_id = 0x14EF;
        i.loot_type = "Blessed";
    });
});

item("OfficialSealingWax", |c| {
    c.label = 1072744;
    c.nontransferable = true;
    c.on_create(|i| {
        i.item_id = 0x1426;
        i.loot_type = "Blessed";
        i.hue = 0x84;
    });
});

item("PortraitOfTheBride", |c| {
    c.label = 1075300;
    c.nontransferable = true;
    c.on_create(|i| {
        i.item_id = 0xE9F;
        i.loot_type = "Blessed";
    });
});

item("ReginasLetter", |c| {
    c.label = 1075306;
    c.nontransferable = true;
    c.on_create(|i| {
        i.item_id = 0x14ED;
        i.loot_type = "Blessed";
    });
});

item("SadrahSatchel", |c| {
    c.on_create(|i| {
        i.item_id = 0xE75;
        i.layer = "Backpack";
        i.quest_item_hue = "randomBrightHue";
        i.quest_item_drop = [["Bloodmoss", 10], ["MortarPestle"]];
    });
});

item("SapOfSosaria", |c| {
    c.label = 1074178;
    c.on_create(|i| {
        i.item_id = 0x1848;
        i.loot_type = "Blessed";
        i.stackable = true;
    });
});

item("SealedNotesForJamal", |c| {
    c.label = 1074998;
    c.nontransferable = true;
    c.on_create(|i| {
        i.item_id = 0xEF9;
        i.loot_type = "Blessed";
    });
});

item("SealingWaxOrderAddressedToPetrus", |c| {
    c.label = 1073132;
    c.nontransferable = true;
    c.on_create(|i| {
        i.item_id = 0xEBF;
        i.loot_type = "Blessed";
    });
});

item("SeveredElfEars", |c| {
    c.on_create(|i| {
        switch random_below(2) {
            0 => {
                i.item_id = 0x312D;
            },
            1 => {
                i.item_id = 0x312E;
            }
        }
        i.stackable = true;
    });
});

item("SeveredHumanEars", |c| {
    c.on_create(|i| {
        switch random_below(2) {
            0 => {
                i.item_id = 0x312F;
            },
            1 => {
                i.item_id = 0x3130;
            }
        }
        i.stackable = true;
    });
});

item("SignedTuitionReimbursementForm", |c| {
    c.label = 1074614;
    c.nontransferable = true;
    c.on_create(|i| {
        i.item_id = 0x14F0;
        i.loot_type = "Blessed";
    });
});

item("SpecialTreatForDrithen", |c| {
    c.label = 1074517;
    c.on_create(|i| {
        i.item_id = 0x21B;
        i.loot_type = "Blessed";
        i.hue = 0x489;
    });
});

item("SpiritBottle", |c| {
    c.label = 1075283;
    c.nontransferable = true;
    c.on_create(|i| {
        i.item_id = 0xEFB;
        i.loot_type = "Blessed";
    });
});

item("StoutWhip", |c| {
    c.label = 1074812;
    c.on_create(|i| {
        i.item_id = 0x166F;
        i.loot_type = "Blessed";
    });
});

item("TaintedTreeSample", |c| {
    c.label = 1074997;
    c.nontransferable = true;
    c.on_create(|i| {
        i.item_id = 0xDE2;
        i.loot_type = "Blessed";
        i.hue = 0x9D;
    });
});

item("TuitionReimbursementForm", |c| {
    c.label = 1074610;
    c.nontransferable = true;
    c.on_create(|i| {
        i.item_id = 0xE3A;
        i.loot_type = "Blessed";
    });
});

item("UnicornRibs", |c| {
    c.label = 1074611;
    c.on_create(|i| {
        i.item_id = 0x9F1;
        i.loot_type = "Blessed";
        i.hue = 0x14B;
        i.stackable = true;
    });
});
