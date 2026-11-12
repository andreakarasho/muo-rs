//! The ML quest definitions of the Town Escorts quest group: quest classes, objectives and rewards.

use crate::ml_quests::engine;
use crate::ml_quests::model::*;

fn escort_to_yew_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.completion_notice = completion_notice_short();
    q.activated = true;
    q.title = td(1072275);
    q.description = td(1072287);
    q.refusal_message = td(1072288);
    q.in_progress_message = td(1072289);
    q.objectives
        .push(Objective::escort(QuestArea::new(td(1072227), "Yew")));
    q.rewards.push(Reward::item(tds("Gold"), "Gold", 500));
    q
}
pub static ESCORT_TO_YEW: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.EscortToYew",
    create: escort_to_yew_create,
    award_human_in_need: Some(true),
    get_rewards: Some(engine::escort_get_rewards),
    ..QuestClass::BASE
};

fn escort_to_vesper_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.completion_notice = completion_notice_short();
    q.activated = true;
    q.title = td(1072276);
    q.description = td(1072287);
    q.refusal_message = td(1072288);
    q.in_progress_message = td(1072290);
    q.objectives
        .push(Objective::escort(QuestArea::new(td(1072229), "Vesper")));
    q.rewards.push(Reward::item(tds("Gold"), "Gold", 500));
    q
}
pub static ESCORT_TO_VESPER: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.EscortToVesper",
    create: escort_to_vesper_create,
    award_human_in_need: Some(true),
    get_rewards: Some(engine::escort_get_rewards),
    ..QuestClass::BASE
};

fn escort_to_trinsic_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.completion_notice = completion_notice_short();
    q.activated = true;
    q.title = td(1072277);
    q.description = td(1072287);
    q.refusal_message = td(1072288);
    q.in_progress_message = td(1072291);
    q.objectives
        .push(Objective::escort(QuestArea::new(td(1072236), "Trinsic")));
    q.rewards.push(Reward::item(tds("Gold"), "Gold", 500));
    q
}
pub static ESCORT_TO_TRINSIC: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.EscortToTrinsic",
    create: escort_to_trinsic_create,
    award_human_in_need: Some(true),
    get_rewards: Some(engine::escort_get_rewards),
    ..QuestClass::BASE
};

fn escort_to_skara_brae_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.completion_notice = completion_notice_short();
    q.activated = true;
    q.title = td(1072278);
    q.description = td(1072287);
    q.refusal_message = td(1072288);
    q.in_progress_message = td(1072292);
    q.objectives
        .push(Objective::escort(QuestArea::new(td(1072235), "Skara Brae")));
    q.rewards.push(Reward::item(tds("Gold"), "Gold", 500));
    q
}
pub static ESCORT_TO_SKARA_BRAE: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.EscortToSkaraBrae",
    create: escort_to_skara_brae_create,
    award_human_in_need: Some(true),
    get_rewards: Some(engine::escort_get_rewards),
    ..QuestClass::BASE
};

fn escort_to_serpents_hold_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.completion_notice = completion_notice_short();
    q.activated = true;
    q.title = td(1072279);
    q.description = td(1072287);
    q.refusal_message = td(1072288);
    q.in_progress_message = td(1072293);
    q.objectives.push(Objective::escort(QuestArea::new(
        td(1072238),
        "Serpent's Hold",
    )));
    q.rewards.push(Reward::item(tds("Gold"), "Gold", 500));
    q
}
pub static ESCORT_TO_SERPENTS_HOLD: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.EscortToSerpentsHold",
    create: escort_to_serpents_hold_create,
    award_human_in_need: Some(true),
    get_rewards: Some(engine::escort_get_rewards),
    ..QuestClass::BASE
};

fn escort_to_nujelm_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.completion_notice = completion_notice_short();
    q.activated = true;
    q.title = td(1072280);
    q.description = td(1072287);
    q.refusal_message = td(1072288);
    q.in_progress_message = td(1072294);
    q.objectives
        .push(Objective::escort(QuestArea::new(td(1072237), "Nujel'm")));
    q.rewards.push(Reward::item(tds("Gold"), "Gold", 500));
    q
}
pub static ESCORT_TO_NUJELM: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.EscortToNujelm",
    create: escort_to_nujelm_create,
    award_human_in_need: Some(true),
    get_rewards: Some(engine::escort_get_rewards),
    ..QuestClass::BASE
};

fn escort_to_moonglow_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.completion_notice = completion_notice_short();
    q.activated = true;
    q.title = td(1072281);
    q.description = td(1072287);
    q.refusal_message = td(1072288);
    q.in_progress_message = td(1072295);
    q.objectives
        .push(Objective::escort(QuestArea::new(td(1072232), "Moonglow")));
    q.rewards.push(Reward::item(tds("Gold"), "Gold", 500));
    q
}
pub static ESCORT_TO_MOONGLOW: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.EscortToMoonglow",
    create: escort_to_moonglow_create,
    award_human_in_need: Some(true),
    get_rewards: Some(engine::escort_get_rewards),
    ..QuestClass::BASE
};

fn escort_to_minoc_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.completion_notice = completion_notice_short();
    q.activated = true;
    q.title = td(1072282);
    q.description = td(1072287);
    q.refusal_message = td(1072288);
    q.in_progress_message = td(1072296);
    q.objectives
        .push(Objective::escort(QuestArea::new(td(1072228), "Minoc")));
    q.rewards.push(Reward::item(tds("Gold"), "Gold", 500));
    q
}
pub static ESCORT_TO_MINOC: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.EscortToMinoc",
    create: escort_to_minoc_create,
    award_human_in_need: Some(true),
    get_rewards: Some(engine::escort_get_rewards),
    ..QuestClass::BASE
};

fn escort_to_magincia_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.completion_notice = completion_notice_short();
    q.activated = true;
    q.title = td(1072283);
    q.description = td(1072287);
    q.refusal_message = td(1072288);
    q.in_progress_message = td(1072297);
    q.objectives
        .push(Objective::escort(QuestArea::new(td(1072233), "Magincia")));
    q.rewards.push(Reward::item(tds("Gold"), "Gold", 500));
    q
}
pub static ESCORT_TO_MAGINCIA: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.EscortToMagincia",
    create: escort_to_magincia_create,
    award_human_in_need: Some(true),
    get_rewards: Some(engine::escort_get_rewards),
    ..QuestClass::BASE
};

fn escort_to_jhelom_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.completion_notice = completion_notice_short();
    q.activated = true;
    q.title = td(1072284);
    q.description = td(1072287);
    q.refusal_message = td(1072288);
    q.in_progress_message = td(1072298);
    q.objectives
        .push(Objective::escort(QuestArea::new(td(1072239), "Jhelom")));
    q.rewards.push(Reward::item(tds("Gold"), "Gold", 500));
    q
}
pub static ESCORT_TO_JHELOM: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.EscortToJhelom",
    create: escort_to_jhelom_create,
    award_human_in_need: Some(true),
    get_rewards: Some(engine::escort_get_rewards),
    ..QuestClass::BASE
};

fn escort_to_cove_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.completion_notice = completion_notice_short();
    q.activated = true;
    q.title = td(1072285);
    q.description = td(1072287);
    q.refusal_message = td(1072288);
    q.in_progress_message = td(1072299);
    q.objectives
        .push(Objective::escort(QuestArea::new(td(1072230), "Cove")));
    q.rewards.push(Reward::item(tds("Gold"), "Gold", 500));
    q
}
pub static ESCORT_TO_COVE: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.EscortToCove",
    create: escort_to_cove_create,
    award_human_in_need: Some(true),
    get_rewards: Some(engine::escort_get_rewards),
    ..QuestClass::BASE
};

fn escort_to_britain_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.completion_notice = completion_notice_short();
    q.activated = true;
    q.title = td(1072286);
    q.description = td(1072287);
    q.refusal_message = td(1072288);
    q.in_progress_message = td(1072300);
    q.objectives
        .push(Objective::escort(QuestArea::new(td(1072231), "Britain")));
    q.rewards.push(Reward::item(tds("Gold"), "Gold", 500));
    q
}
pub static ESCORT_TO_BRITAIN: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.EscortToBritain",
    create: escort_to_britain_create,
    award_human_in_need: Some(true),
    get_rewards: Some(engine::escort_get_rewards),
    ..QuestClass::BASE
};

fn escort_to_ocllo_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.completion_notice = completion_notice_short();
    q.activated = true;
    q.title = td(1072312);
    q.description = td(1072287);
    q.refusal_message = td(1072288);
    q.in_progress_message = td(1072313);
    q.objectives
        .push(Objective::escort(QuestArea::new(td(1072234), "Ocllo")));
    q.rewards.push(Reward::item(tds("Gold"), "Gold", 500));
    q
}
pub static ESCORT_TO_OCLLO: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.EscortToOcllo",
    create: escort_to_ocllo_create,
    award_human_in_need: Some(true),
    get_rewards: Some(engine::escort_get_rewards),
    ..QuestClass::BASE
};
