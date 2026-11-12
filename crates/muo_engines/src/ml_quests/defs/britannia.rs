//! The ML quest definitions of the Britannia quest group: quest classes, objectives and rewards.

use crate::ml_quests::model::*;

fn aemaeth1_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1075321);
    q.description = td(1075322);
    q.refusal_message = td(1075324);
    q.in_progress_message = td(1075325);
    q.completion_message = td(1075326);
    q.completion_notice = completion_notice_short();
    q.objectives.push(Objective::deliver(
        "BasinOfCrystalClearWater",
        1,
        tds("Basin of Crystal Clear Water"),
        "SkeletonOfSzandor",
    ));
    q.rewards.push(Reward::dummy(td(1075323)));
    q
}
pub static AEMAETH1: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.Aemaeth1",
    create: aemaeth1_create,
    next_quest: Some("Aemaeth2"),
    ..QuestClass::BASE
};

fn aemaeth2_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.one_time_only = true;
    q.title = td(1075327);
    q.description = td(1075328);
    q.refusal_message = td(1075330);
    q.in_progress_message = td(1075331);
    q.completion_message = td(1075332);
    q.completion_notice = completion_notice_short();
    q.objectives.push(Objective::deliver(
        "BasinOfCrystalClearWater",
        1,
        tds("Basin of Crystal Clear Water"),
        "Aurelia",
    ));
    q.rewards
        .push(Reward::item(td(1075304), "MirrorOfPurification", 1));
    q
}
pub static AEMAETH2: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.Aemaeth2",
    create: aemaeth2_create,
    chain_triggered: true,
    ..QuestClass::BASE
};

fn odds_and_ends_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1074354);
    q.description = td(1074677);
    q.refusal_message = td(1072270);
    q.in_progress_message = td(1074678);
    q.completion_message = td(1074679);
    q.objectives.push(Objective::collect(
        12,
        "PrimitiveFetish",
        tds("Primitive Fetishes"),
    ));
    q.rewards.push(rewards::bag_of_treasure());
    q
}
pub static ODDS_AND_ENDS: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.OddsAndEnds",
    create: odds_and_ends_create,
    ..QuestClass::BASE
};

fn embracing_humanity_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.one_time_only = true;
    q.title = td(1074349);
    q.description = td(1074357);
    q.refusal_message = td(1074459);
    q.in_progress_message = td(1074460);
    q.completion_message = td(1074461);
    q.objectives.push(Objective::deliver(
        "SpecialTreatForDrithen",
        1,
        tds("treat for Drithen"),
        "Drithen",
    ));
    q.rewards.push(rewards::bag_of_treasure());
    q
}
pub static EMBRACING_HUMANITY: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.EmbracingHumanity",
    create: embracing_humanity_create,
    ..QuestClass::BASE
};
