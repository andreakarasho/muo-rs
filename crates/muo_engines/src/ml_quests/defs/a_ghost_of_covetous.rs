//! The ML quest definitions of the A Ghost of Covetous quest group: quest classes, objectives and rewards.

use crate::ml_quests::model::*;
use muo_primitives::TimeSpan;

fn a_ghost_of_covetous_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1075287);
    q.description = td(1075286);
    q.refusal_message = td(1075288);
    q.in_progress_message = td(1075290);
    q.completion_message = td(1075291);
    q.completion_notice = completion_notice_short();
    q.objectives.push(Objective::deliver(
        "SpiritBottle",
        1,
        tds("Spirit Bottle"),
        "Frederic",
    ));
    q.rewards.push(Reward::dummy(td(1075284)));
    q
}
pub static A_GHOST_OF_COVETOUS: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.AGhostOfCovetous",
    create: a_ghost_of_covetous_create,
    next_quest: Some("SaveHisDad"),
    ..QuestClass::BASE
};

fn save_his_dad_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1075337);
    q.description = td(1075338);
    q.refusal_message = td(1075340);
    q.in_progress_message = td(1075341);
    q.completion_message = td(1075342);
    q.completion_notice = completion_notice_short();
    q.objectives.push(
        Objective::deliver("DreadSpiderSilk", 1, tds("Dread Spider Silk"), "Leon")
            .timed(TimeSpan::from_secs(600)),
    );
    q.rewards.push(Reward::dummy(td(1075339)));
    q
}
pub static SAVE_HIS_DAD: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.SaveHisDad",
    create: save_his_dad_create,
    next_quest: Some("AFathersGratitude"),
    chain_triggered: true,
    ..QuestClass::BASE
};

fn a_fathers_gratitude_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.one_time_only = true;
    q.title = td(1075343);
    q.description = td(1075344);
    q.refusal_message = td(1075346);
    q.in_progress_message = td(1075347);
    q.completion_message = td(1075348);
    q.completion_notice = completion_notice_short();
    q.objectives.push(Objective::deliver(
        "AlchemistsBandage",
        1,
        tds("Alchemist's Bandage"),
        "Andros",
    ));
    q.rewards
        .push(Reward::item(td(1075345), "AndrosGratitude", 1));
    q
}
pub static A_FATHERS_GRATITUDE: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.AFathersGratitude",
    create: a_fathers_gratitude_create,
    chain_triggered: true,
    ..QuestClass::BASE
};
