//! The ML quest definitions of the Bedlam quest group: quest classes, objectives and rewards.

use crate::ml_quests::model::*;

fn momento_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1074750);
    q.description = td(1074751);
    q.refusal_message = td(1074752);
    q.in_progress_message = td(1074753);
    q.completion_message = td(1074754);
    q.objectives.push(Objective::collect(
        1,
        "ResolvesBridle",
        tds("Resolve's Bridle"),
    ));
    q.rewards.push(rewards::large_bag_of_treasure());
    q
}
pub static MOMENTO: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.Momento",
    create: momento_create,
    ..QuestClass::BASE
};

fn culinary_crisis_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1074755);
    q.description = td(1074756);
    q.refusal_message = td(1074757);
    q.in_progress_message = td(1074758);
    q.completion_message = td(1074759);
    q.objectives
        .push(Objective::collect(20, "Dates", td(1025927)));
    q.objectives
        .push(Objective::collect(5, "CheeseWheel", td(1022430)));
    q.rewards.push(rewards::bag_of_treasure());
    q
}
pub static CULINARY_CRISIS: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.CulinaryCrisis",
    create: culinary_crisis_create,
    ..QuestClass::BASE
};

fn gone_native_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1074855);
    q.description = td(1074856);
    q.refusal_message = td(1074857);
    q.in_progress_message = td(1074858);
    q.completion_message = td(1074859);
    q.objectives.push(Objective::kill(
        1,
        &["MasterTheophilus"],
        tds("Master Theophilus"),
        None,
    ));
    q.rewards.push(rewards::large_bag_of_treasure());
    q
}
pub static GONE_NATIVE: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.GoneNative",
    create: gone_native_create,
    ..QuestClass::BASE
};
