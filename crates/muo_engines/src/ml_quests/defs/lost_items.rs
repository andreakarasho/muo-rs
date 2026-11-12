//! The ML quest definitions of the Lost Items quest group: quest classes, objectives and rewards.

use crate::ml_quests::model::*;
use muo_primitives::TimeSpan;

fn lost_and_found_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1072370);
    q.description = td(1072589);
    q.refusal_message = td(1072590);
    q.in_progress_message = td(1072591);
    q.completion_message = td(1074580);
    q.objectives.push(
        Objective::deliver_no_spawn("BatteredBucket", 1, tds("battered bucket"), "Dallid")
            .timed(TimeSpan::from_secs(600)),
    );
    q.rewards.push(rewards::bag_of_trinkets());
    q
}
pub static LOST_AND_FOUND: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.LostAndFound",
    create: lost_and_found_create,
    ..QuestClass::BASE
};
