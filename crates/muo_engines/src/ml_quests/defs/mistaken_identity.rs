//! The ML quest definitions of the Mistaken Identity quest group: quest classes, objectives and rewards.

use super::special;
use crate::ml_quests::model::*;

fn mistaken_identity_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1074573);
    q.description = td(1074574);
    q.refusal_message = td(1074606);
    q.in_progress_message = td(1074605);
    q.completion_message = td(1074607);
    q.completion_notice = completion_notice_short();
    q.objectives.push(Objective::deliver(
        "TuitionReimbursementForm",
        1,
        tds("Tuition Reimbursement Form"),
        "Gorrow",
    ));
    q.rewards.push(Reward::dummy(td(1074634)));
    q
}
pub static MISTAKEN_IDENTITY: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.MistakenIdentity",
    create: mistaken_identity_create,
    next_quest: Some("YouScratchMyBack"),
    ..QuestClass::BASE
};

fn you_scratch_my_back_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1074608);
    q.description = td(1074609);
    q.refusal_message = td(1074615);
    q.in_progress_message = td(1074616);
    q.completion_message = td(1074617);
    q.completion_notice = completion_notice_short_return();
    q.objectives
        .push(Objective::collect(1, "UnicornRibs", tds("Unicorn Ribs")));
    q.objectives
        .push(Objective::collect(2, "KirinBrains", tds("Ki-Rin Brains")));
    q.objectives
        .push(Objective::collect(5, "PixieLeg", tds("Pixie Leg")));
    q.rewards.push(Reward::dummy(td(1074634)));
    q
}
pub static YOU_SCRATCH_MY_BACK: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.YouScratchMyBack",
    create: you_scratch_my_back_create,
    next_quest: Some("FoolingAernya"),
    chain_triggered: true,
    ..QuestClass::BASE
};

fn fooling_aernya_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1074618);
    q.description = td(1074619);
    q.refusal_message = td(1074620);
    q.in_progress_message = td(1074621);
    q.completion_message = td(1074622);
    q.completion_notice = completion_notice_short();
    q.objectives.push(Objective::deliver(
        "SignedTuitionReimbursementForm",
        1,
        tds("Signed Tuition Reimbursement Form"),
        "Aernya",
    ));
    q.rewards.push(Reward::dummy(td(1074634)));
    q
}
pub static FOOLING_AERNYA: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.FoolingAernya",
    create: fooling_aernya_create,
    next_quest: Some("NotQuiteThatEasy"),
    chain_triggered: true,
    ..QuestClass::BASE
};

fn not_quite_that_easy_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1074623);
    q.description = td(1074624);
    q.refusal_message = td(1074626);
    q.in_progress_message = td(1074627);
    q.completion_message = td(1074628);
    q.completion_notice = completion_notice_short();
    q.objectives.push(Objective::deliver(
        "SignedTuitionReimbursementForm",
        1,
        tds("Signed Tuition Reimbursement Form"),
        "MasterGnosos",
    ));
    q.rewards.push(Reward::dummy(td(1074634)));
    q
}
pub static NOT_QUITE_THAT_EASY: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.NotQuiteThatEasy",
    create: not_quite_that_easy_create,
    next_quest: Some("ConvinceMe"),
    chain_triggered: true,
    generate: Some(special::not_quite_that_easy_generate),
    on_accepted: Some(special::not_quite_that_easy_on_accepted),
    ..QuestClass::BASE
};

fn convince_me_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1074629);
    q.description = td(1074630);
    q.refusal_message = td(1074631);
    q.in_progress_message = td(1074632);
    q.completion_message = td(1074633);
    q.completion_notice = completion_notice_short_return();
    q.objectives.push(Objective::kill(
        1,
        &["RedDeath"],
        tds("Red Death"),
        Some(QuestArea::new(td(1074835), "Bedlam")),
    ));
    q.objectives.push(Objective::kill(
        10,
        &["GoreFiend"],
        tds("gore fiends"),
        Some(QuestArea::new(td(1074835), "Bedlam")),
    ));
    q.objectives.push(Objective::kill(
        8,
        &["RottingCorpse"],
        tds("rotting corpses"),
        Some(QuestArea::new(td(1074835), "Bedlam")),
    ));
    q.rewards.push(Reward::dummy(td(1074634)));
    q
}
pub static CONVINCE_ME: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.ConvinceMe",
    create: convince_me_create,
    next_quest: Some("TuitionReimbursement"),
    chain_triggered: true,
    ..QuestClass::BASE
};

fn tuition_reimbursement_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1074634);
    q.description = td(1074635);
    q.refusal_message = td(1074636);
    q.in_progress_message = td(1074637);
    q.completion_message = td(1074638);
    q.completion_notice = completion_notice_short();
    q.objectives.push(Objective::deliver(
        "CompletedTuitionReimbursementForm",
        1,
        tds("Completed Tuition Reimbursement Form"),
        "Aernya",
    ));
    q.rewards.push(rewards::strongbox());
    q
}
pub static TUITION_REIMBURSEMENT: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.TuitionReimbursement",
    create: tuition_reimbursement_create,
    chain_triggered: true,
    ..QuestClass::BASE
};
