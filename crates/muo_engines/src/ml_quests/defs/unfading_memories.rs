//! The ML quest definitions of the Unfading Memories quest group: quest classes, objectives and rewards.

use crate::ml_quests::model::*;

fn unfading_memories_part_one_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1075355);
    q.description = td(1075356);
    q.refusal_message = td(1075358);
    q.in_progress_message = td(1075359);
    q.completion_message = td(1075360);
    q.completion_notice = completion_notice_short();
    q.objectives.push(Objective::collect(
        1,
        "PrismaticAmber",
        tds("Prismatic Amber"),
    ));
    q.rewards.push(Reward::dummy(td(1075357)));
    q
}
pub static UNFADING_MEMORIES_PART_ONE: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.UnfadingMemoriesPartOne",
    create: unfading_memories_part_one_create,
    next_quest: Some("UnfadingMemoriesPartTwo"),
    ..QuestClass::BASE
};

fn unfading_memories_part_two_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1075367);
    q.description = td(1075368);
    q.refusal_message = td(1075370);
    q.in_progress_message = td(1075371);
    q.completion_message = td(1075372);
    q.completion_notice = completion_notice_short();
    q.objectives.push(Objective::deliver(
        "PortraitOfTheBride",
        1,
        tds("Portrait of the Bride"),
        "Thalia",
    ));
    q.rewards.push(Reward::dummy(td(1075369)));
    q
}
pub static UNFADING_MEMORIES_PART_TWO: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.UnfadingMemoriesPartTwo",
    create: unfading_memories_part_two_create,
    next_quest: Some("UnfadingMemoriesPartThree"),
    chain_triggered: true,
    ..QuestClass::BASE
};

fn unfading_memories_part_three_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.one_time_only = true;
    q.title = td(1075373);
    q.description = td(1075374);
    q.refusal_message = td(1075376);
    q.in_progress_message = td(1075377);
    q.completion_message = td(1075378);
    q.completion_notice = completion_notice_short();
    q.objectives.push(Objective::deliver(
        "BridesLetter",
        1,
        tds("Bride's Letter"),
        "Emilio",
    ));
    q.rewards.push(Reward::item(td(1075375), "Bleach", 1));
    q
}
pub static UNFADING_MEMORIES_PART_THREE: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.UnfadingMemoriesPartThree",
    create: unfading_memories_part_three_create,
    chain_triggered: true,
    ..QuestClass::BASE
};
