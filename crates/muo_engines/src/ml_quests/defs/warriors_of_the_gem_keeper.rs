//! The ML quest definitions of the Warriors of the Gem Keeper quest group: quest classes, objectives and rewards.

use crate::ml_quests::model::*;

fn warriors_of_the_gemkeeper_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1074536);
    q.description = td(1074537);
    q.refusal_message = td(1074063);
    q.in_progress_message = td(1074540);
    q.completion_message = td(1074542);
    q.objectives.push(Objective::collect(
        1,
        "FragmentOfAMap",
        tds("fragment of a map"),
    ));
    q.rewards.push(Reward::dummy(td(1074876)));
    q
}
pub static WARRIORS_OF_THE_GEMKEEPER: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.WarriorsOfTheGemkeeper",
    create: warriors_of_the_gemkeeper_create,
    next_quest: Some("CloseEnough"),
    ..QuestClass::BASE
};

fn close_enough_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1074544);
    q.description = td(1074546);
    q.refusal_message = td(1074063);
    q.in_progress_message = td(1074548);
    q.completion_message = td(1074549);
    q.objectives.push(Objective::deliver(
        "FragmentOfAMapDelivery",
        1,
        tds("fragment of a map"),
        "Canir",
    ));
    q.rewards.push(Reward::dummy(td(1074876)));
    q
}
pub static CLOSE_ENOUGH: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.CloseEnough",
    create: close_enough_create,
    next_quest: Some("TakingTheBullByTheHorns"),
    chain_triggered: true,
    ..QuestClass::BASE
};

fn taking_the_bull_by_the_horns_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1074551);
    q.description = td(1074553);
    q.refusal_message = td(1074554);
    q.in_progress_message = td(1074555);
    q.completion_message = td(1074556);
    q.objectives
        .push(Objective::collect(20, "GamanHorns", tds("gaman horns")));
    q.rewards.push(Reward::dummy(td(1074876)));
    q
}
pub static TAKING_THE_BULL_BY_THE_HORNS: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.TakingTheBullByTheHorns",
    create: taking_the_bull_by_the_horns_create,
    next_quest: Some("EmissaryToTheMinotaur"),
    chain_triggered: true,
    ..QuestClass::BASE
};

fn emissary_to_the_minotaur_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1074824);
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
pub static EMISSARY_TO_THE_MINOTAUR: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.EmissaryToTheMinotaur",
    create: emissary_to_the_minotaur_create,
    chain_triggered: true,
    ..QuestClass::BASE
};
