//! The ML quest definitions of the Ilshenar quest group: quest classes, objectives and rewards.

use crate::ml_quests::engine;
use crate::ml_quests::model::*;

fn responsibility_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.completion_notice = completion_notice_short();
    q.activated = true;
    q.title = td(1074352);
    q.description = td(1074524);
    q.refusal_message = td(1074525);
    q.in_progress_message = td(1074526);
    q.objectives
        .push(Objective::escort(QuestArea::new(td(1074781), "Sheep Farm")));
    q.rewards.push(rewards::bag_of_trinkets());
    q
}
pub static RESPONSIBILITY: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.Responsibility",
    create: responsibility_create,
    award_human_in_need: Some(true),
    get_rewards: Some(engine::escort_get_rewards),
    ..QuestClass::BASE
};

fn something_to_wail_about_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1073071);
    q.description = td(1073561);
    q.refusal_message = td(1073580);
    q.in_progress_message = td(1073581);
    q.objectives.push(Objective::kill(
        12,
        &["WailingBanshee"],
        tds("wailing banshees"),
        None,
    ));
    q.rewards.push(rewards::bag_of_treasure());
    q
}
pub static SOMETHING_TO_WAIL_ABOUT: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.SomethingToWailAbout",
    create: something_to_wail_about_create,
    ..QuestClass::BASE
};

fn runaways_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1072993);
    q.description = td(1073026);
    q.refusal_message = td(1072270);
    q.in_progress_message = td(1072271);
    q.objectives.push(Objective::kill(
        12,
        &["FrenziedOstard"],
        tds("frenzied ostards"),
        None,
    ));
    q.rewards.push(rewards::bag_of_trinkets());
    q
}
pub static RUNAWAYS: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.Runaways",
    create: runaways_create,
    ..QuestClass::BASE
};

fn vicious_predator_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1072994);
    q.description = td(1073028);
    q.refusal_message = td(1072270);
    q.in_progress_message = td(1072271);
    q.objectives
        .push(Objective::kill(10, &["DireWolf"], tds("dire wolves"), None));
    q.rewards.push(rewards::bag_of_trinkets());
    q
}
pub static VICIOUS_PREDATOR: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.ViciousPredator",
    create: vicious_predator_create,
    ..QuestClass::BASE
};

fn guile_irk_and_spite_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1074739);
    q.description = td(1074740);
    q.refusal_message = td(1074745);
    q.in_progress_message = td(1074746);
    q.completion_message = td(1074747);
    q.objectives
        .push(Objective::kill(1, &["Guile"], tds("Guile"), None));
    q.objectives
        .push(Objective::kill(1, &["Irk"], tds("Irk"), None));
    q.objectives
        .push(Objective::kill(1, &["Spite"], tds("Spite"), None));
    q.rewards.push(rewards::strongbox());
    q
}
pub static GUILE_IRK_AND_SPITE: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.GuileIrkAndSpite",
    create: guile_irk_and_spite_create,
    ..QuestClass::BASE
};
