//! The ML quest definitions of the Malas quest group: quest classes, objectives and rewards.

use crate::ml_quests::model::*;

fn pointy_ears_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1074640);
    q.description = td(1074641);
    q.refusal_message = td(1074642);
    q.in_progress_message = td(1074643);
    q.completion_message = td(1074644);
    q.objectives
        .push(Objective::collect(20, "SeveredElfEars", td(1032590)));
    q.rewards.push(rewards::bag_of_trinkets());
    q
}
pub static POINTY_EARS: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.PointyEars",
    create: pointy_ears_create,
    ..QuestClass::BASE
};
