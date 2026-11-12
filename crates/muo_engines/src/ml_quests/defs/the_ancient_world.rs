//! The ML quest definitions of The Ancient World quest group: quest classes, objectives and rewards.

use crate::ml_quests::model::*;

fn the_ancient_world_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1074534);
    q.description = td(1074535);
    q.refusal_message = td(1074538);
    q.in_progress_message = td(1074539);
    q.completion_message = td(1074542);
    q.objectives.push(Objective::collect(
        1,
        "FragmentOfAMap",
        tds("fragment of a map"),
    ));
    q.rewards.push(Reward::dummy(td(1074876)));
    q
}
pub static THE_ANCIENT_WORLD: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.TheAncientWorld",
    create: the_ancient_world_create,
    next_quest: Some("TheGoldenHorn"),
    ..QuestClass::BASE
};

fn the_golden_horn_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1074543);
    q.description = td(1074545);
    q.refusal_message = td(1074538);
    q.in_progress_message = td(1074547);
    q.completion_message = td(1074549);
    q.objectives.push(Objective::deliver(
        "FragmentOfAMapDelivery",
        1,
        tds("fragment of a map"),
        "Braen",
    ));
    q.rewards.push(Reward::dummy(td(1074876)));
    q
}
pub static THE_GOLDEN_HORN: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.TheGoldenHorn",
    create: the_golden_horn_create,
    next_quest: Some("Bullish"),
    chain_triggered: true,
    ..QuestClass::BASE
};

fn bullish_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1074550);
    q.description = td(1074552);
    q.refusal_message = td(1074554);
    q.in_progress_message = td(1074555);
    q.completion_message = td(1074556);
    q.objectives
        .push(Objective::collect(20, "GamanHorns", tds("gaman horns")));
    q.rewards.push(Reward::dummy(td(1074876)));
    q
}
pub static BULLISH: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.Bullish",
    create: bullish_create,
    next_quest: Some("LostCivilization"),
    chain_triggered: true,
    ..QuestClass::BASE
};

fn lost_civilization_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1074823);
    q.description = td(1074825);
    q.refusal_message = td(1074827);
    q.in_progress_message = td(1074828);
    q.completion_message = td(1074829);
    q.objectives.push(Objective::collect(
        3,
        "MinotaurArtifact",
        tds("minotaur artifacts"),
    ));
    q.rewards.push(rewards::strongbox());
    q
}
pub static LOST_CIVILIZATION: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.LostCivilization",
    create: lost_civilization_create,
    chain_triggered: true,
    ..QuestClass::BASE
};
