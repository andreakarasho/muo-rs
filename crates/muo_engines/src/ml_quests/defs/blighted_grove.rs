//! The ML quest definitions of the Blighted Grove quest group: quest classes, objectives and rewards.

use super::special;
use crate::ml_quests::model::*;

fn vile_poison_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1074950);
    q.description = td(1074956);
    q.refusal_message = td(1074964);
    q.in_progress_message = td(1074968);
    q.completion_message = td(1074991);
    q.objectives.push(Objective::deliver(
        "TaintedTreeSample",
        1,
        tds("tainted tree sample"),
        "Iosep",
    ));
    q.rewards.push(Reward::dummy(td(1074962)));
    q
}
pub static VILE_POISON: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.VilePoison",
    create: vile_poison_create,
    next_quest: Some("ARockAndAHardPlace"),
    ..QuestClass::BASE
};

fn a_rock_and_a_hard_place_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1074951);
    q.description = td(1074957);
    q.refusal_message = td(1074965);
    q.in_progress_message = td(1074969);
    q.completion_message = td(1074992);
    q.objectives
        .push(Objective::collect(4, "BaseGranite", td(1026009)));
    q.objectives
        .push(Objective::collect(2, "BlueDiamond", td(1032696)));
    q.rewards.push(Reward::dummy(td(1074962)));
    q
}
pub static A_ROCK_AND_A_HARD_PLACE: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.ARockAndAHardPlace",
    create: a_rock_and_a_hard_place_create,
    next_quest: Some("SympatheticMagic"),
    chain_triggered: true,
    ..QuestClass::BASE
};

fn sympathetic_magic_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1074952);
    q.description = td(1074958);
    q.refusal_message = td(1074965);
    q.in_progress_message = td(1074970);
    q.completion_message = td(1074993);
    q.objectives
        .push(Objective::collect(10, "BarkFragment", td(1032687)));
    q.rewards.push(Reward::dummy(td(1074962)));
    q
}
pub static SYMPATHETIC_MAGIC: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.SympatheticMagic",
    create: sympathetic_magic_create,
    next_quest: Some("AlreadyDead"),
    chain_triggered: true,
    ..QuestClass::BASE
};

fn already_dead_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1074953);
    q.description = td(1074959);
    q.refusal_message = td(1074965);
    q.in_progress_message = td(1074971);
    q.completion_message = td(1074994);
    q.objectives.push(special::already_dead_objective());
    q.rewards.push(Reward::dummy(td(1074962)));
    q
}
pub static ALREADY_DEAD: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.AlreadyDead",
    create: already_dead_create,
    next_quest: Some("Eureka"),
    chain_triggered: true,
    ..QuestClass::BASE
};

fn eureka_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1074954);
    q.description = td(1074960);
    q.refusal_message = td(1074966);
    q.in_progress_message = td(1074972);
    q.completion_message = td(1074995);
    q.objectives.push(Objective::deliver(
        "SealedNotesForJamal",
        1,
        tds("sealed note for Jamal"),
        "Jamal",
    ));
    q.rewards.push(Reward::dummy(td(1074962)));
    q
}
pub static EUREKA: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.Eureka",
    create: eureka_create,
    next_quest: Some("SubContracting"),
    chain_triggered: true,
    get_rewards: Some(special::eureka_get_rewards),
    ..QuestClass::BASE
};

fn sub_contracting_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1074955);
    q.description = td(1074961);
    q.refusal_message = td(1074967);
    q.in_progress_message = td(1074973);
    q.completion_message = td(1074996);
    q.objectives.push(Objective::collect(
        3,
        "SamplesOfCorruptedWater",
        tds("samples of corrupted water"),
    ));
    q.rewards.push(rewards::large_bag_of_treasure());
    q
}
pub static SUB_CONTRACTING: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.SubContracting",
    create: sub_contracting_create,
    chain_triggered: true,
    ..QuestClass::BASE
};
