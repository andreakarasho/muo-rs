//! The Inscription craft list (`InitCraftList`), generated: do not edit by hand.

use super::hand::*;
use crate::craft::model::*;
use muo_content::core;
use muo_data::expansion::Expansion;
use muo_io::text::TextDefinition as Td;
use muo_server::World;

/// `InitCraftList()`.
pub fn init(w: &World, s: &mut CraftSystem) {
    let mut st = HelperState {
        circle: 0,
        mana: 4,
        ..HelperState::default()
    };
    add_spell(
        s,
        &mut st,
        "Server.Items.ReactiveArmorScroll",
        &[Reg::Garlic, Reg::SpidersSilk, Reg::SulfurousAsh],
    );
    add_spell(
        s,
        &mut st,
        "Server.Items.ClumsyScroll",
        &[Reg::Bloodmoss, Reg::Nightshade],
    );
    add_spell(
        s,
        &mut st,
        "Server.Items.CreateFoodScroll",
        &[Reg::Garlic, Reg::Ginseng, Reg::MandrakeRoot],
    );
    add_spell(
        s,
        &mut st,
        "Server.Items.FeeblemindScroll",
        &[Reg::Nightshade, Reg::Ginseng],
    );
    add_spell(
        s,
        &mut st,
        "Server.Items.HealScroll",
        &[Reg::Garlic, Reg::Ginseng, Reg::SpidersSilk],
    );
    add_spell(
        s,
        &mut st,
        "Server.Items.MagicArrowScroll",
        &[Reg::SulfurousAsh],
    );
    add_spell(
        s,
        &mut st,
        "Server.Items.NightSightScroll",
        &[Reg::SpidersSilk, Reg::SulfurousAsh],
    );
    add_spell(
        s,
        &mut st,
        "Server.Items.WeakenScroll",
        &[Reg::Garlic, Reg::Nightshade],
    );
    st.circle = 1;
    st.mana = 6;
    add_spell(
        s,
        &mut st,
        "Server.Items.AgilityScroll",
        &[Reg::Bloodmoss, Reg::MandrakeRoot],
    );
    add_spell(
        s,
        &mut st,
        "Server.Items.CunningScroll",
        &[Reg::Nightshade, Reg::MandrakeRoot],
    );
    add_spell(
        s,
        &mut st,
        "Server.Items.CureScroll",
        &[Reg::Garlic, Reg::Ginseng],
    );
    add_spell(
        s,
        &mut st,
        "Server.Items.HarmScroll",
        &[Reg::Nightshade, Reg::SpidersSilk],
    );
    add_spell(
        s,
        &mut st,
        "Server.Items.MagicTrapScroll",
        &[Reg::Garlic, Reg::SpidersSilk, Reg::SulfurousAsh],
    );
    add_spell(
        s,
        &mut st,
        "Server.Items.MagicUnTrapScroll",
        &[Reg::Bloodmoss, Reg::SulfurousAsh],
    );
    add_spell(
        s,
        &mut st,
        "Server.Items.ProtectionScroll",
        &[Reg::Garlic, Reg::Ginseng, Reg::SulfurousAsh],
    );
    add_spell(
        s,
        &mut st,
        "Server.Items.StrengthScroll",
        &[Reg::Nightshade, Reg::MandrakeRoot],
    );
    st.circle = 2;
    st.mana = 9;
    add_spell(
        s,
        &mut st,
        "Server.Items.BlessScroll",
        &[Reg::Garlic, Reg::MandrakeRoot],
    );
    add_spell(
        s,
        &mut st,
        "Server.Items.FireballScroll",
        &[Reg::BlackPearl],
    );
    add_spell(
        s,
        &mut st,
        "Server.Items.MagicLockScroll",
        &[Reg::Bloodmoss, Reg::Garlic, Reg::SulfurousAsh],
    );
    add_spell(s, &mut st, "Server.Items.PoisonScroll", &[Reg::Nightshade]);
    add_spell(
        s,
        &mut st,
        "Server.Items.TelekinesisScroll",
        &[Reg::Bloodmoss, Reg::MandrakeRoot],
    );
    add_spell(
        s,
        &mut st,
        "Server.Items.TeleportScroll",
        &[Reg::Bloodmoss, Reg::MandrakeRoot],
    );
    add_spell(
        s,
        &mut st,
        "Server.Items.UnlockScroll",
        &[Reg::Bloodmoss, Reg::SulfurousAsh],
    );
    add_spell(
        s,
        &mut st,
        "Server.Items.WallOfStoneScroll",
        &[Reg::Bloodmoss, Reg::Garlic],
    );
    st.circle = 3;
    st.mana = 11;
    add_spell(
        s,
        &mut st,
        "Server.Items.ArchCureScroll",
        &[Reg::Garlic, Reg::Ginseng, Reg::MandrakeRoot],
    );
    add_spell(
        s,
        &mut st,
        "Server.Items.ArchProtectionScroll",
        &[
            Reg::Garlic,
            Reg::Ginseng,
            Reg::MandrakeRoot,
            Reg::SulfurousAsh,
        ],
    );
    add_spell(
        s,
        &mut st,
        "Server.Items.CurseScroll",
        &[Reg::Garlic, Reg::Nightshade, Reg::SulfurousAsh],
    );
    add_spell(
        s,
        &mut st,
        "Server.Items.FireFieldScroll",
        &[Reg::BlackPearl, Reg::SpidersSilk, Reg::SulfurousAsh],
    );
    add_spell(
        s,
        &mut st,
        "Server.Items.GreaterHealScroll",
        &[
            Reg::Garlic,
            Reg::SpidersSilk,
            Reg::MandrakeRoot,
            Reg::Ginseng,
        ],
    );
    add_spell(
        s,
        &mut st,
        "Server.Items.LightningScroll",
        &[Reg::MandrakeRoot, Reg::SulfurousAsh],
    );
    add_spell(
        s,
        &mut st,
        "Server.Items.ManaDrainScroll",
        &[Reg::BlackPearl, Reg::SpidersSilk, Reg::MandrakeRoot],
    );
    add_spell(
        s,
        &mut st,
        "Server.Items.RecallScroll",
        &[Reg::BlackPearl, Reg::Bloodmoss, Reg::MandrakeRoot],
    );
    st.circle = 4;
    st.mana = 14;
    add_spell(
        s,
        &mut st,
        "Server.Items.BladeSpiritsScroll",
        &[Reg::BlackPearl, Reg::Nightshade, Reg::MandrakeRoot],
    );
    add_spell(
        s,
        &mut st,
        "Server.Items.DispelFieldScroll",
        &[
            Reg::BlackPearl,
            Reg::Garlic,
            Reg::SpidersSilk,
            Reg::SulfurousAsh,
        ],
    );
    add_spell(
        s,
        &mut st,
        "Server.Items.IncognitoScroll",
        &[Reg::Bloodmoss, Reg::Garlic, Reg::Nightshade],
    );
    add_spell(
        s,
        &mut st,
        "Server.Items.MagicReflectScroll",
        &[Reg::Garlic, Reg::MandrakeRoot, Reg::SpidersSilk],
    );
    add_spell(
        s,
        &mut st,
        "Server.Items.MindBlastScroll",
        &[
            Reg::BlackPearl,
            Reg::MandrakeRoot,
            Reg::Nightshade,
            Reg::SulfurousAsh,
        ],
    );
    add_spell(
        s,
        &mut st,
        "Server.Items.ParalyzeScroll",
        &[Reg::Garlic, Reg::MandrakeRoot, Reg::SpidersSilk],
    );
    add_spell(
        s,
        &mut st,
        "Server.Items.PoisonFieldScroll",
        &[Reg::BlackPearl, Reg::Nightshade, Reg::SpidersSilk],
    );
    add_spell(
        s,
        &mut st,
        "Server.Items.SummonCreatureScroll",
        &[Reg::Bloodmoss, Reg::MandrakeRoot, Reg::SpidersSilk],
    );
    st.circle = 5;
    st.mana = 20;
    add_spell(
        s,
        &mut st,
        "Server.Items.DispelScroll",
        &[Reg::Garlic, Reg::MandrakeRoot, Reg::SulfurousAsh],
    );
    add_spell(
        s,
        &mut st,
        "Server.Items.EnergyBoltScroll",
        &[Reg::BlackPearl, Reg::Nightshade],
    );
    add_spell(
        s,
        &mut st,
        "Server.Items.ExplosionScroll",
        &[Reg::Bloodmoss, Reg::MandrakeRoot],
    );
    add_spell(
        s,
        &mut st,
        "Server.Items.InvisibilityScroll",
        &[Reg::Bloodmoss, Reg::Nightshade],
    );
    add_spell(
        s,
        &mut st,
        "Server.Items.MarkScroll",
        &[Reg::Bloodmoss, Reg::BlackPearl, Reg::MandrakeRoot],
    );
    add_spell(
        s,
        &mut st,
        "Server.Items.MassCurseScroll",
        &[
            Reg::Garlic,
            Reg::MandrakeRoot,
            Reg::Nightshade,
            Reg::SulfurousAsh,
        ],
    );
    add_spell(
        s,
        &mut st,
        "Server.Items.ParalyzeFieldScroll",
        &[Reg::BlackPearl, Reg::Ginseng, Reg::SpidersSilk],
    );
    add_spell(
        s,
        &mut st,
        "Server.Items.RevealScroll",
        &[Reg::Bloodmoss, Reg::SulfurousAsh],
    );
    st.circle = 6;
    st.mana = 40;
    add_spell(
        s,
        &mut st,
        "Server.Items.ChainLightningScroll",
        &[
            Reg::BlackPearl,
            Reg::Bloodmoss,
            Reg::MandrakeRoot,
            Reg::SulfurousAsh,
        ],
    );
    add_spell(
        s,
        &mut st,
        "Server.Items.EnergyFieldScroll",
        &[
            Reg::BlackPearl,
            Reg::MandrakeRoot,
            Reg::SpidersSilk,
            Reg::SulfurousAsh,
        ],
    );
    add_spell(
        s,
        &mut st,
        "Server.Items.FlamestrikeScroll",
        &[Reg::SpidersSilk, Reg::SulfurousAsh],
    );
    add_spell(
        s,
        &mut st,
        "Server.Items.GateTravelScroll",
        &[Reg::BlackPearl, Reg::MandrakeRoot, Reg::SulfurousAsh],
    );
    add_spell(
        s,
        &mut st,
        "Server.Items.ManaVampireScroll",
        &[
            Reg::BlackPearl,
            Reg::Bloodmoss,
            Reg::MandrakeRoot,
            Reg::SpidersSilk,
        ],
    );
    add_spell(
        s,
        &mut st,
        "Server.Items.MassDispelScroll",
        &[
            Reg::BlackPearl,
            Reg::Garlic,
            Reg::MandrakeRoot,
            Reg::SulfurousAsh,
        ],
    );
    add_spell(
        s,
        &mut st,
        "Server.Items.MeteorSwarmScroll",
        &[
            Reg::Bloodmoss,
            Reg::MandrakeRoot,
            Reg::SulfurousAsh,
            Reg::SpidersSilk,
        ],
    );
    add_spell(
        s,
        &mut st,
        "Server.Items.PolymorphScroll",
        &[Reg::Bloodmoss, Reg::MandrakeRoot, Reg::SpidersSilk],
    );
    st.circle = 7;
    st.mana = 50;
    add_spell(
        s,
        &mut st,
        "Server.Items.EarthquakeScroll",
        &[
            Reg::Bloodmoss,
            Reg::MandrakeRoot,
            Reg::Ginseng,
            Reg::SulfurousAsh,
        ],
    );
    add_spell(
        s,
        &mut st,
        "Server.Items.EnergyVortexScroll",
        &[
            Reg::BlackPearl,
            Reg::Bloodmoss,
            Reg::MandrakeRoot,
            Reg::Nightshade,
        ],
    );
    add_spell(
        s,
        &mut st,
        "Server.Items.ResurrectionScroll",
        &[Reg::Bloodmoss, Reg::Garlic, Reg::Ginseng],
    );
    add_spell(
        s,
        &mut st,
        "Server.Items.SummonAirElementalScroll",
        &[Reg::Bloodmoss, Reg::MandrakeRoot, Reg::SpidersSilk],
    );
    add_spell(
        s,
        &mut st,
        "Server.Items.SummonDaemonScroll",
        &[
            Reg::Bloodmoss,
            Reg::MandrakeRoot,
            Reg::SpidersSilk,
            Reg::SulfurousAsh,
        ],
    );
    add_spell(
        s,
        &mut st,
        "Server.Items.SummonEarthElementalScroll",
        &[Reg::Bloodmoss, Reg::MandrakeRoot, Reg::SpidersSilk],
    );
    add_spell(
        s,
        &mut st,
        "Server.Items.SummonFireElementalScroll",
        &[
            Reg::Bloodmoss,
            Reg::MandrakeRoot,
            Reg::SpidersSilk,
            Reg::SulfurousAsh,
        ],
    );
    add_spell(
        s,
        &mut st,
        "Server.Items.SummonWaterElementalScroll",
        &[Reg::Bloodmoss, Reg::MandrakeRoot, Reg::SpidersSilk],
    );
    if core::se(w) {
        add_necro_spell(
            s,
            &mut st,
            0,
            23,
            39.6,
            "Server.Items.AnimateDeadScroll",
            &["Server.Items.GraveDust", "Server.Items.DaemonBlood"],
        );
        add_necro_spell(
            s,
            &mut st,
            1,
            13,
            19.6,
            "Server.Items.BloodOathScroll",
            &["Server.Items.DaemonBlood"],
        );
        add_necro_spell(
            s,
            &mut st,
            2,
            11,
            19.6,
            "Server.Items.CorpseSkinScroll",
            &["Server.Items.BatWing", "Server.Items.GraveDust"],
        );
        add_necro_spell(
            s,
            &mut st,
            3,
            7,
            19.6,
            "Server.Items.CurseWeaponScroll",
            &["Server.Items.PigIron"],
        );
        add_necro_spell(
            s,
            &mut st,
            4,
            11,
            19.6,
            "Server.Items.EvilOmenScroll",
            &["Server.Items.BatWing", "Server.Items.NoxCrystal"],
        );
        add_necro_spell(
            s,
            &mut st,
            5,
            11,
            39.6,
            "Server.Items.HorrificBeastScroll",
            &["Server.Items.BatWing", "Server.Items.DaemonBlood"],
        );
        add_necro_spell(
            s,
            &mut st,
            6,
            23,
            69.6,
            "Server.Items.LichFormScroll",
            &[
                "Server.Items.GraveDust",
                "Server.Items.DaemonBlood",
                "Server.Items.NoxCrystal",
            ],
        );
        add_necro_spell(
            s,
            &mut st,
            7,
            17,
            29.6,
            "Server.Items.MindRotScroll",
            &[
                "Server.Items.BatWing",
                "Server.Items.DaemonBlood",
                "Server.Items.PigIron",
            ],
        );
        add_necro_spell(
            s,
            &mut st,
            8,
            5,
            19.6,
            "Server.Items.PainSpikeScroll",
            &["Server.Items.GraveDust", "Server.Items.PigIron"],
        );
        add_necro_spell(
            s,
            &mut st,
            9,
            17,
            49.6,
            "Server.Items.PoisonStrikeScroll",
            &["Server.Items.NoxCrystal"],
        );
        add_necro_spell(
            s,
            &mut st,
            10,
            29,
            64.6,
            "Server.Items.StrangleScroll",
            &["Server.Items.DaemonBlood", "Server.Items.NoxCrystal"],
        );
        add_necro_spell(
            s,
            &mut st,
            11,
            17,
            29.6,
            "Server.Items.SummonFamiliarScroll",
            &[
                "Server.Items.BatWing",
                "Server.Items.GraveDust",
                "Server.Items.DaemonBlood",
            ],
        );
        add_necro_spell(
            s,
            &mut st,
            12,
            23,
            98.6,
            "Server.Items.VampiricEmbraceScroll",
            &[
                "Server.Items.BatWing",
                "Server.Items.NoxCrystal",
                "Server.Items.PigIron",
            ],
        );
        add_necro_spell(
            s,
            &mut st,
            13,
            41,
            79.6,
            "Server.Items.VengefulSpiritScroll",
            &[
                "Server.Items.BatWing",
                "Server.Items.GraveDust",
                "Server.Items.PigIron",
            ],
        );
        add_necro_spell(
            s,
            &mut st,
            14,
            23,
            59.6,
            "Server.Items.WitherScroll",
            &[
                "Server.Items.GraveDust",
                "Server.Items.NoxCrystal",
                "Server.Items.PigIron",
            ],
        );
        add_necro_spell(
            s,
            &mut st,
            15,
            17,
            79.6,
            "Server.Items.WraithFormScroll",
            &["Server.Items.NoxCrystal", "Server.Items.PigIron"],
        );
        add_necro_spell(
            s,
            &mut st,
            16,
            40,
            79.6,
            "Server.Items.ExorcismScroll",
            &["Server.Items.NoxCrystal", "Server.Items.GraveDust"],
        );
    }
    let mut index;
    if core::ml(w) {
        index = s.add_craft(
            "Server.Items.EnchantedSwitch",
            Td::of_number(1044294),
            Td::of_number(1072893),
            None,
            None,
            45.0,
            95.0,
            "Server.Items.BlankScroll",
            Td::of_number(1044377),
            1,
            Td::of_number(1044378),
        );
        s.add_res(
            index,
            "Server.Items.SpidersSilk",
            Td::of_number(1044360),
            1,
            Td::of_number(1044253),
        );
        s.add_res(
            index,
            "Server.Items.BlackPearl",
            Td::of_number(1044353),
            1,
            Td::of_number(1044253),
        );
        s.add_res(
            index,
            "Server.Items.SwitchItem",
            Td::of_number(1073464),
            1,
            Td::of_number(1044253),
        );
        s.force_non_exceptional(index);
        s.set_needed_expansion(index, Expansion::ML);
        index = s.add_craft(
            "Server.Items.RunedPrism",
            Td::of_number(1044294),
            Td::of_number(1073465),
            None,
            None,
            45.0,
            95.0,
            "Server.Items.BlankScroll",
            Td::of_number(1044377),
            1,
            Td::of_number(1044378),
        );
        s.add_res(
            index,
            "Server.Items.SpidersSilk",
            Td::of_number(1044360),
            1,
            Td::of_number(1044253),
        );
        s.add_res(
            index,
            "Server.Items.BlackPearl",
            Td::of_number(1044353),
            1,
            Td::of_number(1044253),
        );
        s.add_res(
            index,
            "Server.Items.HollowPrism",
            Td::of_number(1072895),
            1,
            Td::of_number(1044253),
        );
        s.force_non_exceptional(index);
        s.set_needed_expansion(index, Expansion::ML);
    }
    index = s.add_craft(
        "Server.Items.Runebook",
        Td::of_number(1044294),
        Td::of_number(1041267),
        None,
        None,
        45.0,
        95.0,
        "Server.Items.BlankScroll",
        Td::of_number(1044377),
        8,
        Td::of_number(1044378),
    );
    s.add_res(
        index,
        "Server.Items.RecallScroll",
        Td::of_number(1044445),
        1,
        Td::of_number(1044253),
    );
    s.add_res(
        index,
        "Server.Items.GateTravelScroll",
        Td::of_number(1044446),
        1,
        Td::of_number(1044253),
    );
    if core::aos(w) {
        s.add_craft(
            "Server.Engines.BulkOrders.BulkOrderBook",
            Td::of_number(1044294),
            Td::of_number(1028793),
            None,
            None,
            65.0,
            115.0,
            "Server.Items.BlankScroll",
            Td::of_number(1044377),
            10,
            Td::of_number(1044378),
        );
    }
    if core::se(w) {
        s.add_craft(
            "Server.Items.Spellbook",
            Td::of_number(1044294),
            Td::of_number(1023834),
            None,
            None,
            50.0,
            126.0,
            "Server.Items.BlankScroll",
            Td::of_number(1044377),
            10,
            Td::of_number(1044378),
        );
    }
    if core::sa(w) {
        s.add_craft(
            "Server.Items.MysticSpellbook",
            Td::of_number(1044294),
            Td::of_number(1031677),
            None,
            None,
            50.0,
            150.0,
            "Server.Items.BlankScroll",
            Td::of_number(1044377),
            10,
            Td::of_number(1044378),
        );
        add_mysticism_spell(
            s,
            &mut st,
            0,
            4,
            -25.0,
            25.0,
            "Server.Items.NetherBoltScroll",
            &["Server.Items.BlackPearl", "Server.Items.SulfurousAsh"],
        );
        add_mysticism_spell(
            s,
            &mut st,
            1,
            4,
            -25.0,
            25.0,
            "Server.Items.HealingStoneScroll",
            &[
                "Server.Items.Bone",
                "Server.Items.Garlic",
                "Server.Items.Ginseng",
                "Server.Items.SpidersSilk",
            ],
        );
        add_mysticism_spell(
            s,
            &mut st,
            2,
            6,
            -10.8,
            39.2,
            "Server.Items.PurgeMagicScroll",
            &[
                "Server.Items.FertileDirt",
                "Server.Items.Garlic",
                "Server.Items.MandrakeRoot",
                "Server.Items.SulfurousAsh",
            ],
        );
        add_mysticism_spell(
            s,
            &mut st,
            3,
            6,
            -10.8,
            39.2,
            "Server.Items.EnchantScroll",
            &[
                "Server.Items.SpidersSilk",
                "Server.Items.MandrakeRoot",
                "Server.Items.SulfurousAsh",
            ],
        );
        add_mysticism_spell(
            s,
            &mut st,
            4,
            9,
            3.5,
            53.5,
            "Server.Items.SleepScroll",
            &[
                "Server.Items.Nightshade",
                "Server.Items.SpidersSilk",
                "Server.Items.BlackPearl",
            ],
        );
        add_mysticism_spell(
            s,
            &mut st,
            5,
            9,
            3.5,
            53.5,
            "Server.Items.EagleStrikeScroll",
            &[
                "Server.Items.Bloodmoss",
                "Server.Items.Bone",
                "Server.Items.SpidersSilk",
                "Server.Items.MandrakeRoot",
            ],
        );
        add_mysticism_spell(
            s,
            &mut st,
            6,
            11,
            17.8,
            67.8,
            "Server.Items.AnimatedWeaponScroll",
            &[
                "Server.Items.Bone",
                "Server.Items.BlackPearl",
                "Server.Items.MandrakeRoot",
                "Server.Items.Nightshade",
            ],
        );
        add_mysticism_spell(
            s,
            &mut st,
            7,
            11,
            17.8,
            67.8,
            "Server.Items.StoneFormScroll",
            &[
                "Server.Items.Bloodmoss",
                "Server.Items.FertileDirt",
                "Server.Items.Garlic",
            ],
        );
        add_mysticism_spell(
            s,
            &mut st,
            8,
            14,
            32.1,
            82.1,
            "Server.Items.SpellTriggerScroll",
            &[
                "Server.Items.DragonsBlood",
                "Server.Items.Garlic",
                "Server.Items.MandrakeRoot",
                "Server.Items.SpidersSilk",
            ],
        );
        add_mysticism_spell(
            s,
            &mut st,
            9,
            14,
            32.1,
            82.1,
            "Server.Items.MassSleepScroll",
            &[
                "Server.Items.Ginseng",
                "Server.Items.Nightshade",
                "Server.Items.SpidersSilk",
            ],
        );
        add_mysticism_spell(
            s,
            &mut st,
            10,
            20,
            46.4,
            96.4,
            "Server.Items.CleansingWindsScroll",
            &[
                "Server.Items.DragonsBlood",
                "Server.Items.Garlic",
                "Server.Items.Ginseng",
                "Server.Items.MandrakeRoot",
            ],
        );
        add_mysticism_spell(
            s,
            &mut st,
            11,
            20,
            46.4,
            96.4,
            "Server.Items.BombardScroll",
            &[
                "Server.Items.Bloodmoss",
                "Server.Items.DragonsBlood",
                "Server.Items.Garlic",
                "Server.Items.SulfurousAsh",
            ],
        );
        add_mysticism_spell(
            s,
            &mut st,
            12,
            40,
            60.7,
            110.7,
            "Server.Items.SpellPlagueScroll",
            &[
                "Server.Items.DaemonBone",
                "Server.Items.DragonsBlood",
                "Server.Items.Nightshade",
                "Server.Items.SulfurousAsh",
            ],
        );
        add_mysticism_spell(
            s,
            &mut st,
            13,
            40,
            60.7,
            110.7,
            "Server.Items.HailStormScroll",
            &[
                "Server.Items.DragonsBlood",
                "Server.Items.Bloodmoss",
                "Server.Items.BlackPearl",
                "Server.Items.MandrakeRoot",
            ],
        );
        add_mysticism_spell(
            s,
            &mut st,
            14,
            50,
            75.0,
            125.0,
            "Server.Items.NetherCycloneScroll",
            &[
                "Server.Items.MandrakeRoot",
                "Server.Items.Nightshade",
                "Server.Items.SulfurousAsh",
                "Server.Items.Bloodmoss",
            ],
        );
        add_mysticism_spell(
            s,
            &mut st,
            15,
            50,
            75.0,
            125.0,
            "Server.Items.RisingColossusScroll",
            &[
                "Server.Items.DaemonBone",
                "Server.Items.DragonsBlood",
                "Server.Items.FertileDirt",
                "Server.Items.Nightshade",
            ],
        );
    }
    s.mark_option = true;
}
