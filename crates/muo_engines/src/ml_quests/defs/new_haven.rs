//! The ML quest definitions of the New Haven quest group: quest classes, objectives and rewards.

use crate::ml_quests::engine;
use crate::ml_quests::model::*;

fn escort_to_nh_alchemist_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.completion_notice = completion_notice_short();
    q.activated = true;
    q.title = td(1072314);
    q.description = td(1042769);
    q.refusal_message = td(1072288);
    q.in_progress_message = td(1072326);
    q.objectives.push(Objective::escort(QuestArea::new(
        td(1073864),
        "the New Haven Alchemist",
    )));
    q.rewards.push(Reward::item(tds("Gold"), "Gold", 500));
    q
}
pub static ESCORT_TO_NH_ALCHEMIST: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.EscortToNHAlchemist",
    create: escort_to_nh_alchemist_create,
    award_human_in_need: Some(false),
    get_rewards: Some(engine::escort_get_rewards),
    ..QuestClass::BASE
};

fn escort_to_nh_bard_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.completion_notice = completion_notice_short();
    q.activated = true;
    q.title = td(1072315);
    q.description = td(1042772);
    q.refusal_message = td(1072288);
    q.in_progress_message = td(1072327);
    q.objectives.push(Objective::escort(QuestArea::new(
        td(1073865),
        "the New Haven Bard",
    )));
    q.rewards.push(Reward::item(tds("Gold"), "Gold", 500));
    q
}
pub static ESCORT_TO_NH_BARD: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.EscortToNHBard",
    create: escort_to_nh_bard_create,
    award_human_in_need: Some(false),
    get_rewards: Some(engine::escort_get_rewards),
    ..QuestClass::BASE
};

fn escort_to_nh_warrior_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.completion_notice = completion_notice_short();
    q.activated = true;
    q.title = td(1072316);
    q.description = td(1042787);
    q.refusal_message = td(1072288);
    q.in_progress_message = td(1072328);
    q.objectives.push(Objective::escort(QuestArea::new(
        td(1073866),
        "the New Haven Warrior",
    )));
    q.rewards.push(Reward::item(tds("Gold"), "Gold", 500));
    q
}
pub static ESCORT_TO_NH_WARRIOR: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.EscortToNHWarrior",
    create: escort_to_nh_warrior_create,
    award_human_in_need: Some(false),
    get_rewards: Some(engine::escort_get_rewards),
    ..QuestClass::BASE
};

fn escort_to_nh_tailor_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.completion_notice = completion_notice_short();
    q.activated = true;
    q.title = td(1072317);
    q.description = td(1042781);
    q.refusal_message = td(1072288);
    q.in_progress_message = td(1072329);
    q.objectives.push(Objective::escort(QuestArea::new(
        td(1073867),
        "the New Haven Tailor",
    )));
    q.rewards.push(Reward::item(tds("Gold"), "Gold", 500));
    q
}
pub static ESCORT_TO_NH_TAILOR: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.EscortToNHTailor",
    create: escort_to_nh_tailor_create,
    award_human_in_need: Some(false),
    get_rewards: Some(engine::escort_get_rewards),
    ..QuestClass::BASE
};

fn escort_to_nh_carpenter_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.completion_notice = completion_notice_short();
    q.activated = true;
    q.title = td(1072318);
    q.description = td(1042775);
    q.refusal_message = td(1072288);
    q.in_progress_message = td(1072330);
    q.objectives.push(Objective::escort(QuestArea::new(
        td(1073868),
        "the New Haven Carpenter",
    )));
    q.rewards.push(Reward::item(tds("Gold"), "Gold", 500));
    q
}
pub static ESCORT_TO_NH_CARPENTER: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.EscortToNHCarpenter",
    create: escort_to_nh_carpenter_create,
    award_human_in_need: Some(false),
    get_rewards: Some(engine::escort_get_rewards),
    ..QuestClass::BASE
};

fn escort_to_nh_mapmaker_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.completion_notice = completion_notice_short();
    q.activated = true;
    q.title = td(1072319);
    q.description = td(1042793);
    q.refusal_message = td(1072288);
    q.in_progress_message = td(1072331);
    q.objectives.push(Objective::escort(QuestArea::new(
        td(1073869),
        "the New Haven Mapmaker",
    )));
    q.rewards.push(Reward::item(tds("Gold"), "Gold", 500));
    q
}
pub static ESCORT_TO_NH_MAPMAKER: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.EscortToNHMapmaker",
    create: escort_to_nh_mapmaker_create,
    award_human_in_need: Some(false),
    get_rewards: Some(engine::escort_get_rewards),
    ..QuestClass::BASE
};

fn escort_to_nh_mage_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.completion_notice = completion_notice_short();
    q.activated = true;
    q.title = td(1072320);
    q.description = td(1042790);
    q.refusal_message = td(1072288);
    q.in_progress_message = td(1072332);
    q.objectives.push(Objective::escort(QuestArea::new(
        td(1073870),
        "the New Haven Mage",
    )));
    q.rewards.push(Reward::item(tds("Gold"), "Gold", 500));
    q
}
pub static ESCORT_TO_NH_MAGE: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.EscortToNHMage",
    create: escort_to_nh_mage_create,
    award_human_in_need: Some(false),
    get_rewards: Some(engine::escort_get_rewards),
    ..QuestClass::BASE
};

fn escort_to_nh_inn_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.completion_notice = completion_notice_short();
    q.activated = true;
    q.title = td(1072321);
    q.description = td(1042796);
    q.refusal_message = td(1072288);
    q.in_progress_message = td(1072333);
    q.objectives.push(Objective::escort(QuestArea::new(
        td(1073871),
        "the New Haven Inn",
    )));
    q.rewards.push(Reward::item(tds("Gold"), "Gold", 500));
    q
}
pub static ESCORT_TO_NH_INN: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.EscortToNHInn",
    create: escort_to_nh_inn_create,
    award_human_in_need: Some(false),
    get_rewards: Some(engine::escort_get_rewards),
    ..QuestClass::BASE
};

fn escort_to_nh_farm_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.completion_notice = completion_notice_short();
    q.activated = true;
    q.title = td(1072322);
    q.description = td(1042799);
    q.refusal_message = td(1072288);
    q.in_progress_message = td(1072334);
    q.objectives.push(Objective::escort(QuestArea::new(
        td(1073872),
        "the New Haven Farm",
    )));
    q.rewards.push(Reward::item(tds("Gold"), "Gold", 500));
    q
}
pub static ESCORT_TO_NH_FARM: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.EscortToNHFarm",
    create: escort_to_nh_farm_create,
    award_human_in_need: Some(false),
    get_rewards: Some(engine::escort_get_rewards),
    ..QuestClass::BASE
};

fn escort_to_nh_docks_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.completion_notice = completion_notice_short();
    q.activated = true;
    q.title = td(1072323);
    q.description = td(1042802);
    q.refusal_message = td(1072288);
    q.in_progress_message = td(1072335);
    q.objectives.push(Objective::escort(QuestArea::new(
        td(1073873),
        "the New Haven Docks",
    )));
    q.rewards.push(Reward::item(tds("Gold"), "Gold", 500));
    q
}
pub static ESCORT_TO_NH_DOCKS: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.EscortToNHDocks",
    create: escort_to_nh_docks_create,
    award_human_in_need: Some(false),
    get_rewards: Some(engine::escort_get_rewards),
    ..QuestClass::BASE
};

fn escort_to_nh_bowyer_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.completion_notice = completion_notice_short();
    q.activated = true;
    q.title = td(1072324);
    q.description = td(1042805);
    q.refusal_message = td(1072288);
    q.in_progress_message = td(1072336);
    q.objectives.push(Objective::escort(QuestArea::new(
        td(1073874),
        "the New Haven Bowyer",
    )));
    q.rewards.push(Reward::item(tds("Gold"), "Gold", 500));
    q
}
pub static ESCORT_TO_NH_BOWYER: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.EscortToNHBowyer",
    create: escort_to_nh_bowyer_create,
    award_human_in_need: Some(false),
    get_rewards: Some(engine::escort_get_rewards),
    ..QuestClass::BASE
};

fn escort_to_nh_bank_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.completion_notice = completion_notice_short();
    q.activated = true;
    q.title = td(1072325);
    q.description = td(1042784);
    q.refusal_message = td(1072288);
    q.in_progress_message = td(1072337);
    q.objectives.push(Objective::escort(QuestArea::new(
        td(1073875),
        "the New Haven Bank",
    )));
    q.rewards.push(Reward::item(tds("Gold"), "Gold", 500));
    q
}
pub static ESCORT_TO_NH_BANK: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.EscortToNHBank",
    create: escort_to_nh_bank_create,
    award_human_in_need: Some(false),
    get_rewards: Some(engine::escort_get_rewards),
    ..QuestClass::BASE
};
