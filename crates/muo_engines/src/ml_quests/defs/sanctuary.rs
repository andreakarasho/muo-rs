//! The ML quest definitions of the Sanctuary quest group: quest classes, objectives and rewards.

use crate::ml_quests::model::*;

fn broken_shaft_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.has_restart_delay = true;
    q.title = td(1074018);
    q.description = td(1074112);
    q.refusal_message = td(1074063);
    q.in_progress_message = td(1074064);
    q.completion_message = td(1074065);
    q.objectives
        .push(Objective::collect(10, "Arrow", td(1023902)));
    q.rewards.push(rewards::fletching_satchel());
    q
}
pub static BROKEN_SHAFT: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.BrokenShaft",
    create: broken_shaft_create,
    ..QuestClass::BASE
};

fn bending_the_bow_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.has_restart_delay = true;
    q.title = td(1074019);
    q.description = td(1074113);
    q.refusal_message = td(1074063);
    q.in_progress_message = td(1074064);
    q.completion_message = td(1074065);
    q.objectives
        .push(Objective::collect(10, "Bow", td(1025041)));
    q.rewards.push(rewards::fletching_satchel());
    q
}
pub static BENDING_THE_BOW: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.BendingTheBow",
    create: bending_the_bow_create,
    ..QuestClass::BASE
};

fn arms_race_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.has_restart_delay = true;
    q.title = td(1074020);
    q.description = td(1074114);
    q.refusal_message = td(1074063);
    q.in_progress_message = td(1074064);
    q.completion_message = td(1074065);
    q.objectives
        .push(Objective::collect(10, "Crossbow", td(1023919)));
    q.rewards.push(rewards::fletching_satchel());
    q
}
pub static ARMS_RACE: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.ArmsRace",
    create: arms_race_create,
    ..QuestClass::BASE
};

fn improved_crossbows_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.has_restart_delay = true;
    q.title = td(1074021);
    q.description = td(1074115);
    q.refusal_message = td(1074063);
    q.in_progress_message = td(1074064);
    q.completion_message = td(1074065);
    q.objectives
        .push(Objective::collect(10, "HeavyCrossbow", td(1025116)));
    q.rewards.push(rewards::fletching_satchel());
    q
}
pub static IMPROVED_CROSSBOWS: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.ImprovedCrossbows",
    create: improved_crossbows_create,
    ..QuestClass::BASE
};

fn building_the_better_crossbow_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.has_restart_delay = true;
    q.title = td(1074022);
    q.description = td(1074116);
    q.refusal_message = td(1074063);
    q.in_progress_message = td(1074064);
    q.completion_message = td(1074065);
    q.objectives
        .push(Objective::collect(10, "RepeatingCrossbow", td(1029923)));
    q.rewards.push(rewards::fletching_satchel());
    q
}
pub static BUILDING_THE_BETTER_CROSSBOW: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.BuildingTheBetterCrossbow",
    create: building_the_better_crossbow_create,
    ..QuestClass::BASE
};

fn instrument_of_war_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.has_restart_delay = true;
    q.title = td(1074055);
    q.description = td(1074149);
    q.refusal_message = td(1074063);
    q.in_progress_message = td(1074064);
    q.completion_message = td(1074065);
    q.objectives
        .push(Objective::collect(12, "Broadsword", td(1023934)));
    q.rewards.push(rewards::blacksmith_satchel());
    q
}
pub static INSTRUMENT_OF_WAR: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.InstrumentOfWar",
    create: instrument_of_war_create,
    ..QuestClass::BASE
};

fn the_shield_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.has_restart_delay = true;
    q.title = td(1074054);
    q.description = td(1074148);
    q.refusal_message = td(1074063);
    q.in_progress_message = td(1074064);
    q.completion_message = td(1074065);
    q.objectives
        .push(Objective::collect(10, "HeaterShield", td(1027030)));
    q.rewards.push(rewards::blacksmith_satchel());
    q
}
pub static THE_SHIELD: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.TheShield",
    create: the_shield_create,
    ..QuestClass::BASE
};

fn music_to_my_ears_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.has_restart_delay = true;
    q.title = td(1074023);
    q.description = td(1074117);
    q.refusal_message = td(1074063);
    q.in_progress_message = td(1074064);
    q.completion_message = td(1074065);
    q.objectives
        .push(Objective::collect(10, "LapHarp", td(1023762)));
    q.rewards.push(rewards::carpentry_satchel());
    q
}
pub static MUSIC_TO_MY_EARS: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.MusicToMyEars",
    create: music_to_my_ears_create,
    ..QuestClass::BASE
};

fn the_glass_eye_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.has_restart_delay = true;
    q.title = td(1074050);
    q.description = td(1074144);
    q.refusal_message = td(1074063);
    q.in_progress_message = td(1074064);
    q.completion_message = td(1074065);
    q.objectives
        .push(Objective::collect(10, "Spyglass", td(1025365)));
    q.rewards.push(rewards::tinker_satchel());
    q
}
pub static THE_GLASS_EYE: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.TheGlassEye",
    create: the_glass_eye_create,
    ..QuestClass::BASE
};

fn lazy_humans_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.has_restart_delay = true;
    q.title = td(1074024);
    q.description = td(1074118);
    q.refusal_message = td(1074063);
    q.in_progress_message = td(1074064);
    q.completion_message = td(1074065);
    q.objectives
        .push(Objective::collect(10, "FootStool", td(1022910)));
    q.rewards.push(rewards::carpentry_satchel());
    q
}
pub static LAZY_HUMANS: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.LazyHumans",
    create: lazy_humans_create,
    ..QuestClass::BASE
};

fn inventive_tools_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.has_restart_delay = true;
    q.title = td(1074048);
    q.description = td(1074142);
    q.refusal_message = td(1074063);
    q.in_progress_message = td(1074064);
    q.completion_message = td(1074065);
    q.objectives
        .push(Objective::collect(10, "TinkerTools", td(1027868)));
    q.rewards.push(rewards::tinker_satchel());
    q
}
pub static INVENTIVE_TOOLS: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.InventiveTools",
    create: inventive_tools_create,
    ..QuestClass::BASE
};

fn pixie_dust_to_dust_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1073661);
    q.description = td(1073700);
    q.refusal_message = td(1073733);
    q.in_progress_message = td(1073741);
    q.objectives
        .push(Objective::kill(10, &["Pixie"], tds("pixies"), None));
    q.rewards.push(rewards::large_bag_of_treasure());
    q
}
pub static PIXIE_DUST_TO_DUST: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.PixieDustToDust",
    create: pixie_dust_to_dust_create,
    ..QuestClass::BASE
};

fn an_impressive_plaid_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.has_restart_delay = true;
    q.title = td(1074044);
    q.description = td(1074138);
    q.refusal_message = td(1074063);
    q.in_progress_message = td(1074064);
    q.completion_message = td(1074065);
    q.objectives
        .push(Objective::collect(10, "Kilt", td(1025431)));
    q.rewards.push(rewards::tailor_satchel());
    q
}
pub static AN_IMPRESSIVE_PLAID: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.AnImpressivePlaid",
    create: an_impressive_plaid_create,
    ..QuestClass::BASE
};

fn a_nice_shirt_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.has_restart_delay = true;
    q.title = td(1074045);
    q.description = td(1074139);
    q.refusal_message = td(1074063);
    q.in_progress_message = td(1074064);
    q.completion_message = td(1074065);
    q.objectives
        .push(Objective::collect(10, "FancyShirt", td(1027933)));
    q.rewards.push(rewards::tailor_satchel());
    q
}
pub static A_NICE_SHIRT: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.ANiceShirt",
    create: a_nice_shirt_create,
    ..QuestClass::BASE
};

fn leather_and_lace_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.has_restart_delay = true;
    q.title = td(1074047);
    q.description = td(1074141);
    q.refusal_message = td(1074063);
    q.in_progress_message = td(1074064);
    q.completion_message = td(1074065);
    q.objectives
        .push(Objective::collect(10, "StuddedBustierArms", td(1027180)));
    q.rewards.push(rewards::tailor_satchel());
    q
}
pub static LEATHER_AND_LACE: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.LeatherAndLace",
    create: leather_and_lace_create,
    ..QuestClass::BASE
};

fn fey_headgear_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.has_restart_delay = true;
    q.title = td(1074043);
    q.description = td(1074137);
    q.refusal_message = td(1074063);
    q.in_progress_message = td(1074064);
    q.completion_message = td(1074065);
    q.objectives
        .push(Objective::collect(10, "FlowerGarland", td(1028965)));
    q.rewards.push(rewards::tailor_satchel());
    q
}
pub static FEY_HEADGEAR: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.FeyHeadgear",
    create: fey_headgear_create,
    ..QuestClass::BASE
};

fn new_cloak_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1074684);
    q.description = td(1074685);
    q.refusal_message = td(1074655);
    q.in_progress_message = td(1074686);
    q.completion_message = td(1074687);
    q.objectives
        .push(Objective::collect(1, "GrobusFur", tds("Grobu's Fur")));
    q.rewards.push(rewards::tailor_satchel());
    q
}
pub static NEW_CLOAK: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.NewCloak",
    create: new_cloak_create,
    ..QuestClass::BASE
};

fn a_dish_best_served_cold_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1072372);
    q.description = td(1072657);
    q.refusal_message = td(1072667);
    q.in_progress_message = td(1072668);
    q.objectives.push(Objective::kill(
        10,
        &["Orc"],
        tds("orcs"),
        Some(QuestArea::new(td(1074807), "Sanctuary")),
    ));
    q.objectives.push(Objective::kill(
        5,
        &["OrcBomber"],
        tds("orc bombers"),
        Some(QuestArea::new(td(1074807), "Sanctuary")),
    ));
    q.objectives.push(Objective::kill(
        3,
        &["OrcBrute"],
        tds("orc brutes"),
        Some(QuestArea::new(td(1074807), "Sanctuary")),
    ));
    q.rewards.push(rewards::bag_of_treasure());
    q
}
pub static A_DISH_BEST_SERVED_COLD: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.ADishBestServedCold",
    create: a_dish_best_served_cold_create,
    ..QuestClass::BASE
};

fn arch_enemies_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1073085);
    q.description = td(1073575);
    q.refusal_message = td(1073580);
    q.in_progress_message = td(1073595);
    q.objectives.push(Objective::kill(
        10,
        &["RatmanArcher"],
        tds("ratman archers"),
        None,
    ));
    q.rewards.push(rewards::bag_of_treasure());
    q
}
pub static ARCH_ENEMIES: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.ArchEnemies",
    create: arch_enemies_create,
    ..QuestClass::BASE
};

fn vermin_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1072995);
    q.description = td(1073029);
    q.refusal_message = td(1072270);
    q.in_progress_message = td(1072271);
    q.objectives
        .push(Objective::kill(12, &["Ratman"], tds("ratmen"), None));
    q.rewards.push(rewards::bag_of_trinkets());
    q
}
pub static VERMIN: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.Vermin",
    create: vermin_create,
    ..QuestClass::BASE
};

fn moug_guur_must_die_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1072368);
    q.description = td(1072561);
    q.refusal_message = td(1072571);
    q.in_progress_message = td(1072572);
    q.completion_message = td(1072573);
    q.objectives.push(Objective::kill(
        1,
        &["MougGuur"],
        tds("Moug-Guur"),
        Some(QuestArea::new(td(1074807), "Sanctuary")),
    ));
    q.rewards.push(rewards::bag_of_treasure());
    q
}
pub static MOUG_GUUR_MUST_DIE: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.MougGuurMustDie",
    create: moug_guur_must_die_create,
    next_quest: Some("LeaderOfThePack"),
    ..QuestClass::BASE
};

fn leader_of_the_pack_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1072560);
    q.description = td(1072574);
    q.refusal_message = td(1072575);
    q.in_progress_message = td(1072576);
    q.completion_message = td(1072577);
    q.objectives.push(Objective::kill(
        1,
        &["Chiikkaha"],
        tds("Chiikkaha"),
        Some(QuestArea::new(td(1074807), "Sanctuary")),
    ));
    q.rewards.push(rewards::bag_of_treasure());
    q
}
pub static LEADER_OF_THE_PACK: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.LeaderOfThePack",
    create: leader_of_the_pack_create,
    next_quest: Some("SayonaraSzavetra"),
    chain_triggered: true,
    ..QuestClass::BASE
};

fn sayonara_szavetra_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1072375);
    q.description = td(1072578);
    q.refusal_message = td(1072579);
    q.in_progress_message = td(1072581);
    q.completion_message = td(1072582);
    q.objectives.push(Objective::kill(
        1,
        &["Szavetra"],
        tds("Szavetra"),
        Some(QuestArea::new(td(1074807), "Sanctuary")),
    ));
    q.rewards.push(rewards::strongbox());
    q
}
pub static SAYONARA_SZAVETRA: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.SayonaraSzavetra",
    create: sayonara_szavetra_create,
    chain_triggered: true,
    ..QuestClass::BASE
};

fn tapping_the_keg_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.has_restart_delay = true;
    q.title = td(1074037);
    q.description = td(1074131);
    q.refusal_message = td(1074063);
    q.in_progress_message = td(1074064);
    q.completion_message = td(1074065);
    q.objectives
        .push(Objective::collect(10, "BarrelTap", td(1024100)));
    q.rewards.push(rewards::tinker_satchel());
    q
}
pub static TAPPING_THE_KEG: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.TappingTheKeg",
    create: tapping_the_keg_create,
    ..QuestClass::BASE
};

fn waiting_to_be_filled_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.has_restart_delay = true;
    q.title = td(1074036);
    q.description = td(1074130);
    q.refusal_message = td(1074063);
    q.in_progress_message = td(1074064);
    q.completion_message = td(1074065);
    q.objectives
        .push(Objective::collect(20, "Bottle", td(1023854)));
    q.rewards.push(rewards::tinker_satchel());
    q
}
pub static WAITING_TO_BE_FILLED: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.WaitingToBeFilled",
    create: waiting_to_be_filled_create,
    ..QuestClass::BASE
};

fn breezes_song_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.has_restart_delay = true;
    q.title = td(1074052);
    q.description = td(1074146);
    q.refusal_message = td(1074063);
    q.in_progress_message = td(1074064);
    q.completion_message = td(1074065);
    q.objectives
        .push(Objective::collect(10, "FancyWindChimes", td(1030291)));
    q.rewards.push(rewards::tinker_satchel());
    q
}
pub static BREEZES_SONG: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.BreezesSong",
    create: breezes_song_create,
    ..QuestClass::BASE
};

fn proof_of_the_deed_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1072339);
    q.description = td(1072340);
    q.refusal_message = td(1072342);
    q.in_progress_message = td(1072343);
    q.completion_message = td(1072344);
    q.objectives
        .push(Objective::collect(20, "SeveredHumanEars", td(1032591)));
    q.rewards.push(rewards::bag_of_trinkets());
    q
}
pub static PROOF_OF_THE_DEED: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.ProofOfTheDeed",
    create: proof_of_the_deed_create,
    ..QuestClass::BASE
};

fn marauders_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1072374);
    q.description = td(1072686);
    q.refusal_message = td(1072687);
    q.in_progress_message = td(1072688);
    q.objectives.push(Objective::kill(
        10,
        &["Ogre"],
        tds("ogres"),
        Some(QuestArea::new(td(1074807), "Sanctuary")),
    ));
    q.rewards.push(rewards::bag_of_treasure());
    q
}
pub static MARAUDERS: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.Marauders",
    create: marauders_create,
    next_quest: Some("TheBrainsOfTheOperation"),
    ..QuestClass::BASE
};

fn the_brains_of_the_operation_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1072692);
    q.description = td(1072707);
    q.refusal_message = td(1072708);
    q.in_progress_message = td(1072709);
    q.objectives.push(Objective::kill(
        10,
        &["OgreLord"],
        tds("ogre lords"),
        Some(QuestArea::new(td(1074807), "Sanctuary")),
    ));
    q.rewards.push(rewards::large_bag_of_treasure());
    q
}
pub static THE_BRAINS_OF_THE_OPERATION: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.TheBrainsOfTheOperation",
    create: the_brains_of_the_operation_create,
    next_quest: Some("TheBrawn"),
    chain_triggered: true,
    ..QuestClass::BASE
};

fn the_brawn_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1072693);
    q.description = td(1072710);
    q.refusal_message = td(1072711);
    q.in_progress_message = td(1072712);
    q.objectives.push(Objective::kill(
        6,
        &["Cyclops"],
        tds("cyclops"),
        Some(QuestArea::new(td(1074807), "Sanctuary")),
    ));
    q.rewards.push(rewards::large_bag_of_treasure());
    q
}
pub static THE_BRAWN: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.TheBrawn",
    create: the_brawn_create,
    next_quest: Some("TheBiggerTheyAre"),
    chain_triggered: true,
    ..QuestClass::BASE
};

fn the_bigger_they_are_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1072694);
    q.description = td(1072713);
    q.refusal_message = td(1072714);
    q.in_progress_message = td(1072715);
    q.objectives.push(Objective::kill(
        3,
        &["Titan"],
        tds("titans"),
        Some(QuestArea::new(td(1074807), "Sanctuary")),
    ));
    q.rewards.push(rewards::large_bag_of_treasure());
    q
}
pub static THE_BIGGER_THEY_ARE: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.TheBiggerTheyAre",
    create: the_bigger_they_are_create,
    chain_triggered: true,
    ..QuestClass::BASE
};

fn trouble_on_the_wing_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1072371);
    q.description = td(1072593);
    q.refusal_message = td(1072594);
    q.in_progress_message = td(1072595);
    q.objectives.push(Objective::kill(
        12,
        &["Gargoyle"],
        tds("gargoyles"),
        Some(QuestArea::new(td(1074807), "Sanctuary")),
    ));
    q.rewards.push(rewards::bag_of_trinkets());
    q
}
pub static TROUBLE_ON_THE_WING: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.TroubleOnTheWing",
    create: trouble_on_the_wing_create,
    ..QuestClass::BASE
};

fn brotherly_love_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.one_time_only = true;
    q.title = td(1072369);
    q.description = td(1072585);
    q.refusal_message = td(1072587);
    q.in_progress_message = td(1072588);
    q.completion_message = td(1074579);
    q.completion_notice = completion_notice_short();
    q.objectives.push(Objective::deliver(
        "APersonalLetterAddressedToAhie",
        1,
        tds("letter"),
        "Ahie",
    ));
    q.rewards.push(rewards::bag_of_trinkets());
    q
}
pub static BROTHERLY_LOVE: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.BrotherlyLove",
    create: brotherly_love_create,
    ..QuestClass::BASE
};

fn common_brigands_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1073082);
    q.description = td(1073572);
    q.refusal_message = td(1073580);
    q.in_progress_message = td(1073592);
    q.objectives
        .push(Objective::kill(20, &["Brigand"], td(1074894), None));
    q.rewards.push(rewards::bag_of_treasure());
    q
}
pub static COMMON_BRIGANDS: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.CommonBrigands",
    create: common_brigands_create,
    ..QuestClass::BASE
};
