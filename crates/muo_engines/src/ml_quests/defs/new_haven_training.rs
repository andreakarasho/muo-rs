//! The ML quest definitions of the New Haven Training quest group: quest classes, objectives and rewards.

use super::special;
use crate::ml_quests::model::*;

fn split_ends_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.has_restart_delay = true;
    q.title = td(1075506);
    q.description = td(1075507);
    q.refusal_message = td(1075508);
    q.in_progress_message = td(1072271);
    q.completion_message = td(1072272);
    q.objectives
        .push(Objective::collect(20, "Arrow", td(1023902)));
    q.rewards
        .push(Reward::item(td(1074282), "AndricSatchel", 1));
    q
}
pub static SPLIT_ENDS: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.SplitEnds",
    create: split_ends_create,
    ..QuestClass::BASE
};

fn i_shot_an_arrow_into_the_air_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1075486);
    q.description = td(1075482);
    q.refusal_message = td(1075483);
    q.in_progress_message = td(1075484);
    q.objectives
        .push(Objective::kill(10, &["Sheep"], td(1018270), None));
    q.rewards.push(rewards::bag_of_trinkets());
    q
}
pub static I_SHOT_AN_ARROW_INTO_THE_AIR: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.IShotAnArrowIntoTheAir",
    create: i_shot_an_arrow_into_the_air_create,
    ..QuestClass::BASE
};

fn bakers_dozen_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.has_restart_delay = true;
    q.title = td(1075478);
    q.description = td(1075479);
    q.refusal_message = td(1075480);
    q.in_progress_message = td(1072271);
    q.completion_message = td(1075481);
    q.objectives
        .push(Objective::collect(5, "CookieMix", td(1024159)));
    q.rewards
        .push(Reward::item(td(1074282), "AsandosSatchel", 1));
    q
}
pub static BAKERS_DOZEN: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.BakersDozen",
    create: bakers_dozen_create,
    ..QuestClass::BASE
};

fn a_stitch_in_time_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.has_restart_delay = true;
    q.title = td(1075523);
    q.description = td(1075522);
    q.refusal_message = td(1075526);
    q.in_progress_message = td(1075527);
    q.completion_message = td(1075528);
    q.objectives
        .push(Objective::collect(1, "FancyDress", td(1027935)));
    q.rewards.push(Reward::item(td(1075524), "AnOldRing", 1));
    q.rewards
        .push(Reward::item(td(1075525), "AnOldNecklace", 1));
    q
}
pub static A_STITCH_IN_TIME: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.AStitchInTime",
    create: a_stitch_in_time_create,
    ..QuestClass::BASE
};

fn battered_bucklers_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.has_restart_delay = true;
    q.title = td(1075511);
    q.description = td(1075512);
    q.refusal_message = td(1075514);
    q.in_progress_message = td(1075515);
    q.completion_message = td(1075516);
    q.objectives
        .push(Objective::collect(10, "Buckler", td(1027027)));
    q.rewards
        .push(Reward::item(td(1074282), "GervisSatchel", 1));
    q
}
pub static BATTERED_BUCKLERS: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.BatteredBucklers",
    create: battered_bucklers_create,
    ..QuestClass::BASE
};

fn more_ore_please_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.has_restart_delay = true;
    q.title = td(1075530);
    q.description = td(1075529);
    q.refusal_message = td(1075531);
    q.in_progress_message = td(1075532);
    q.completion_message = td(1075533);
    q.objectives.push(special::more_ore_please_objective());
    q.rewards.push(Reward::item(td(1074282), "MuggSatchel", 1));
    q
}
pub static MORE_ORE_PLEASE: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.MoreOrePlease",
    create: more_ore_please_create,
    ..QuestClass::BASE
};

fn comfortable_seating_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.has_restart_delay = true;
    q.title = td(1075517);
    q.description = td(1075518);
    q.refusal_message = td(1072687);
    q.in_progress_message = td(1075509);
    q.completion_message = td(1074720);
    q.objectives
        .push(Objective::collect(1, "BambooChair", tds("straw chair")));
    q.rewards.push(Reward::item(td(1074282), "LowelSatchel", 1));
    q
}
pub static COMFORTABLE_SEATING: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.ComfortableSeating",
    create: comfortable_seating_create,
    ..QuestClass::BASE
};

fn the_pen_is_mightier_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.has_restart_delay = true;
    q.title = td(1075542);
    q.description = td(1075543);
    q.refusal_message = td(1075546);
    q.in_progress_message = td(1075547);
    q.completion_message = td(1075548);
    q.objectives
        .push(Objective::collect(5, "RecallScroll", tds("recall scroll")));
    q.rewards
        .push(Reward::item(td(1075545), "RedLeatherBook", 1));
    q
}
pub static THE_PEN_IS_MIGHTIER: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.ThePenIsMightier",
    create: the_pen_is_mightier_create,
    ..QuestClass::BASE
};

fn a_clockwork_puzzle_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.has_restart_delay = true;
    q.title = td(1075535);
    q.description = td(1075534);
    q.refusal_message = td(1072981);
    q.in_progress_message = td(1072271);
    q.completion_message = td(1075536);
    q.objectives
        .push(Objective::collect(5, "ClockParts", td(1024175)));
    q.rewards
        .push(Reward::item(td(1074282), "NibbetSatchel", 1));
    q
}
pub static A_CLOCKWORK_PUZZLE: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.AClockworkPuzzle",
    create: a_clockwork_puzzle_create,
    ..QuestClass::BASE
};

fn delicious_fishes_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.has_restart_delay = true;
    q.title = td(1075555);
    q.description = td(1075556);
    q.refusal_message = td(1075558);
    q.in_progress_message = td(1075559);
    q.completion_message = td(1075560);
    q.objectives
        .push(Objective::collect(5, "Fish", td(1022508)));
    q.rewards
        .push(Reward::item(td(1075557), "PeppercornFishsteak", 3));
    q
}
pub static DELICIOUS_FISHES: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.DeliciousFishes",
    create: delicious_fishes_create,
    ..QuestClass::BASE
};

fn flee_and_fatigue_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.has_restart_delay = true;
    q.title = td(1075487);
    q.description = td(1075488);
    q.refusal_message = td(1075489);
    q.in_progress_message = td(1075490);
    q.completion_message = td(1075491);
    q.objectives.push(Objective::collect(
        10,
        "RefreshPotion",
        tds("refresh potions"),
    ));
    q.rewards
        .push(Reward::item(td(1074282), "SadrahSatchel", 1));
    q
}
pub static FLEE_AND_FATIGUE: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.FleeAndFatigue",
    create: flee_and_fatigue_create,
    ..QuestClass::BASE
};

fn chop_chop_on_the_double_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.has_restart_delay = true;
    q.title = td(1075537);
    q.description = td(1075538);
    q.refusal_message = td(1072981);
    q.in_progress_message = td(1072271);
    q.completion_message = td(1075539);
    q.objectives
        .push(Objective::collect(60, "Log", td(1027133)));
    q.rewards
        .push(Reward::item(td(1074282), "HargroveSatchel", 1));
    q
}
pub static CHOP_CHOP_ON_THE_DOUBLE: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.ChopChopOnTheDouble",
    create: chop_chop_on_the_double_create,
    ..QuestClass::BASE
};
