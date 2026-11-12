//! The ML quest definitions of the Honest Beggar quest group: quest classes, objectives and rewards.

use crate::ml_quests::model::*;

fn honest_beggar_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1075392);
    q.description = td(1075393);
    q.refusal_message = td(1075395);
    q.in_progress_message = td(1075396);
    q.completion_message = td(1075397);
    q.completion_notice = completion_notice_short();
    q.objectives.push(Objective::deliver(
        "ReginasRing",
        1,
        tds("Regina's Ring"),
        "Regina",
    ));
    q.rewards.push(Reward::dummy(td(1075394)));
    q
}
pub static HONEST_BEGGAR: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.HonestBeggar",
    create: honest_beggar_create,
    next_quest: Some("ReginasThanks"),
    ..QuestClass::BASE
};

fn reginas_thanks_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.one_time_only = true;
    q.title = td(1075398);
    q.description = td(1075399);
    q.refusal_message = td(1075401);
    q.in_progress_message = td(1075402);
    q.completion_message = td(1075403);
    q.completion_notice = completion_notice_short();
    q.objectives.push(Objective::deliver(
        "ReginasLetter",
        1,
        tds("Regina's Letter"),
        "Evan",
    ));
    q.rewards
        .push(Reward::item(td(1075400), "TransparentHeart", 1));
    q
}
pub static REGINAS_THANKS: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.ReginasThanks",
    create: reginas_thanks_create,
    chain_triggered: true,
    ..QuestClass::BASE
};
