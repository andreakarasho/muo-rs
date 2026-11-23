mobile("BardGuildmaster", |c| {
    c.npc_guild = "BardsGuild";
    c.on_create(|m| {
        m.npc_title_by_gender = #{ female: "the bard guildmistress", male: "the bard guildmaster" };
        m.skill("Archery", 80.0, 100.0);
        m.skill("Discordance", 80.0, 100.0);
        m.skill("Musicianship", 80.0, 100.0);
        m.skill("Peacemaking", 80.0, 100.0);
        m.skill("Provocation", 80.0, 100.0);
        m.skill("Swords", 80.0, 100.0);
    });
});

mobile("BlacksmithGuildmaster", |c| {
    c.npc_guild = "BlacksmithsGuild";
    c.npc_sb_infos = ["SBBlacksmith"];
    c.npc_shoe_type = "ThighBoots";
    c.on_create(|m| {
        m.npc_title_by_gender = #{ female: "the blacksmith guildmistress", male: "the blacksmith guildmaster" };
        m.skill("ArmsLore", 65.0, 88.0);
        m.skill("Blacksmith", 90.0, 100.0);
        m.skill("Macing", 36.0, 68.0);
        m.skill("Parry", 36.0, 68.0);
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

mobile("FisherGuildmaster", |c| {
    c.npc_guild = "FishermensGuild";
    c.on_create(|m| {
        m.npc_title_by_gender = #{ female: "the fisher guildmistress", male: "the fisher guildmaster" };
        m.skill("Fishing", 80.0, 100.0);
    });
});

mobile("HealerGuildmaster", |c| {
    c.npc_guild = "HealersGuild";
    c.npc_shoe_type = "Sandals";
    c.on_create(|m| {
        m.npc_title_by_gender = #{ female: "the healer guildmistress", male: "the healer guildmaster" };
        m.skill("Anatomy", 85.0, 100.0);
        m.skill("Healing", 90.0, 100.0);
        m.skill("Forensics", 75.0, 98.0);
        m.skill("MagicResist", 75.0, 98.0);
        m.skill("SpiritSpeak", 65.0, 88.0);
    });
    c.on_outfit(|m| {
        m.npc_vendor_outfit = true;
        m.npc_add_item = [#{ item: "Server.Items.Robe", hue: "randomYellow" }];
    });
});

mobile("MageGuildmaster", |c| {
    c.npc_guild = "MagesGuild";
    c.npc_shoe_type = ["Shoes", "Sandals"];
    c.on_create(|m| {
        m.npc_title_by_gender = #{ female: "the mage guildmistress", male: "the mage guildmaster" };
        m.skill("EvalInt", 85.0, 100.0);
        m.skill("Inscribe", 65.0, 88.0);
        m.skill("MagicResist", 64.0, 100.0);
        m.skill("Magery", 90.0, 100.0);
        m.skill("Wrestling", 60.0, 83.0);
        m.skill("Meditation", 85.0, 100.0);
        m.skill("Macing", 36.0, 68.0);
    });
    c.on_outfit(|m| {
        m.npc_vendor_outfit = true;
        m.npc_add_item = [#{ item: "Server.Items.Robe", hue: "randomBlue" }, "Server.Items.GnarledStaff"];
    });
});

mobile("MerchantGuildmaster", |c| {
    c.npc_guild = "MerchantsGuild";
    c.on_create(|m| {
        m.npc_title_by_gender = #{ female: "the merchant guildmistress", male: "the merchant guildmaster" };
        m.skill("ItemID", 85.0, 100.0);
        m.skill("ArmsLore", 85.0, 100.0);
    });
});

mobile("MinerGuildmaster", |c| {
    c.npc_guild = "MinersGuild";
    c.on_create(|m| {
        m.npc_title_by_gender = #{ female: "the miner guildmistress", male: "the miner guildmaster" };
        m.skill("ItemID", 60.0, 83.0);
        m.skill("Mining", 90.0, 100.0);
    });
});

mobile("RangerGuildmaster", |c| {
    c.npc_guild = "RangersGuild";
    c.on_create(|m| {
        m.npc_title_by_gender = #{ female: "the ranger guildmistress", male: "the ranger guildmaster" };
        m.skill("AnimalLore", 64.0, 100.0);
        m.skill("Camping", 75.0, 98.0);
        m.skill("Hiding", 75.0, 98.0);
        m.skill("MagicResist", 75.0, 98.0);
        m.skill("Tactics", 65.0, 88.0);
        m.skill("Archery", 90.0, 100.0);
        m.skill("Tracking", 90.0, 100.0);
        m.skill("Stealth", 60.0, 83.0);
        m.skill("Fencing", 36.0, 68.0);
        m.skill("Herding", 36.0, 68.0);
        m.skill("Swords", 45.0, 68.0);
    });
});

mobile("TailorGuildmaster", |c| {
    c.npc_guild = "TailorsGuild";
    c.on_create(|m| {
        m.npc_title_by_gender = #{ female: "the tailor guildmistress", male: "the tailor guildmaster" };
        m.skill("Tailoring", 90.0, 100.0);
    });
});

mobile("ThiefGuildmaster", |c| {
    c.npc_guild = "ThievesGuild";
    c.on_create(|m| {
        m.npc_title_by_gender = #{ female: "the thief guildmistress", male: "the thief guildmaster" };
        m.skill("DetectHidden", 75.0, 98.0);
        m.skill("Hiding", 65.0, 88.0);
        m.skill("Lockpicking", 85.0, 100.0);
        m.skill("Snooping", 90.0, 100.0);
        m.skill("Poisoning", 60.0, 83.0);
        m.skill("Stealing", 90.0, 100.0);
        m.skill("Fencing", 75.0, 98.0);
        m.skill("Stealth", 85.0, 100.0);
        m.skill("RemoveTrap", 85.0, 100.0);
    });
    c.on_outfit(|m| {
        m.npc_vendor_outfit = true;
        if coin() {
            m.npc_add_item = ["Server.Items.Kryss"];
        } else {
            m.npc_add_item = ["Server.Items.Dagger"];
        }
    });
});

mobile("TinkerGuildmaster", |c| {
    c.npc_guild = "TinkersGuild";
    c.on_create(|m| {
        m.npc_title_by_gender = #{ female: "the tinker guildmistress", male: "the tinker guildmaster" };
        m.skill("Lockpicking", 65.0, 88.0);
        m.skill("Tinkering", 90.0, 100.0);
        m.skill("RemoveTrap", 85.0, 100.0);
    });
});

mobile("WarriorGuildmaster", |c| {
    c.npc_guild = "WarriorsGuild";
    c.on_create(|m| {
        m.npc_title_by_gender = #{ female: "the warrior guildmistress", male: "the warrior guildmaster" };
        m.skill("ArmsLore", 75.0, 98.0);
        m.skill("Parry", 85.0, 100.0);
        m.skill("MagicResist", 60.0, 83.0);
        m.skill("Tactics", 85.0, 100.0);
        m.skill("Swords", 90.0, 100.0);
        m.skill("Macing", 60.0, 83.0);
        m.skill("Fencing", 60.0, 83.0);
    });
});
