//! The ML quest definitions of the Heritage quest group: quest classes, objectives and rewards.

use super::{player_of, special};
use crate::ml_quests::engine;
use crate::ml_quests::model::*;
use muo_primitives::TimeSpan;
use muo_server::mobile::part2::messages as mm;
use muo_server::world::World;

fn seasons_get_rewards(w: &mut World, inst: InstanceId) {
    let p = player_of(w, inst);
    mm::send_localized_message(w, p, 1074940, "", 0x2A);
    engine::claim_rewards(w, inst);
}
fn seasons_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1072782);
    q.description = td(1072802);
    q.refusal_message = td(1072810);
    q.in_progress_message = td(1072811);
    q.completion_message = td(1074174);
    q.objectives
        .push(Objective::collect(20, "RawFishSteak", td(1022426)));
    q.rewards.push(Reward::dummy(td(1072803)));
    q
}
pub static SEASONS: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.Seasons",
    create: seasons_create,
    record_completion: true,
    get_rewards: Some(seasons_get_rewards),
    ..QuestClass::BASE
};

fn caretaker_of_the_land_get_rewards(w: &mut World, inst: InstanceId) {
    let p = player_of(w, inst);
    mm::send_localized_message(w, p, 1074941, "", 0x2A);
    engine::claim_rewards(w, inst);
}
fn caretaker_of_the_land_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1072783);
    q.description = td(1072812);
    q.refusal_message = td(1072813);
    q.in_progress_message = td(1072814);
    q.completion_message = td(1074175);
    q.objectives
        .push(Objective::collect(1, "SapOfSosaria", tds("sap of sosaria")));
    q.rewards.push(Reward::dummy(td(1072804)));
    q
}
pub static CARETAKER_OF_THE_LAND: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.CaretakerOfTheLand",
    create: caretaker_of_the_land_create,
    record_completion: true,
    get_rewards: Some(caretaker_of_the_land_get_rewards),
    ..QuestClass::BASE
};

fn wisdom_of_the_sphynx_get_rewards(w: &mut World, inst: InstanceId) {
    let p = player_of(w, inst);
    mm::send_localized_message(w, p, 1074945, "", 0x2A);
    engine::claim_rewards(w, inst);
}
fn wisdom_of_the_sphynx_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1072784);
    q.description = td(1072822);
    q.refusal_message = td(1072823);
    q.in_progress_message = td(1072824);
    q.completion_message = td(1074176);
    q.objectives.push(special::wisdom_of_the_sphynx_objective());
    q.rewards.push(Reward::dummy(td(1072805)));
    q
}
pub static WISDOM_OF_THE_SPHYNX: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.WisdomOfTheSphynx",
    create: wisdom_of_the_sphynx_create,
    record_completion: true,
    get_rewards: Some(wisdom_of_the_sphynx_get_rewards),
    ..QuestClass::BASE
};

fn defending_the_herd_get_rewards(w: &mut World, inst: InstanceId) {
    let p = player_of(w, inst);
    mm::send_localized_message(w, p, 1074942, "", 0x2A);
    engine::claim_rewards(w, inst);
}
fn defending_the_herd_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1072785);
    q.description = td(1072825);
    q.refusal_message = td(1072826);
    q.in_progress_message = td(1072827);
    q.completion_notice = completion_notice_short();
    q.objectives.push(Objective::escort(QuestArea::new(
        td(1074779),
        "Bravehorn's drinking pool",
    )));
    q.rewards.push(Reward::dummy(td(1072806)));
    q
}
pub static DEFENDING_THE_HERD: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.DefendingTheHerd",
    create: defending_the_herd_create,
    record_completion: true,
    get_rewards: Some(defending_the_herd_get_rewards),
    ..QuestClass::BASE
};

fn the_balance_of_nature_get_rewards(w: &mut World, inst: InstanceId) {
    let p = player_of(w, inst);
    mm::send_localized_message(w, p, 1074943, "", 0x2A);
    engine::claim_rewards(w, inst);
}
fn the_balance_of_nature_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1072786);
    q.description = td(1072829);
    q.refusal_message = td(1072830);
    q.in_progress_message = td(1072831);
    q.objectives.push(Objective::kill(
        15,
        &["TimberWolf"],
        tds("timber wolves"),
        Some(QuestArea::new(td(1074833), "Huntsman's Forest")),
    ));
    q.rewards.push(Reward::dummy(td(1072807)));
    q
}
pub static THE_BALANCE_OF_NATURE: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.TheBalanceOfNature",
    create: the_balance_of_nature_create,
    record_completion: true,
    get_rewards: Some(the_balance_of_nature_get_rewards),
    ..QuestClass::BASE
};

fn the_joys_of_life_get_rewards(w: &mut World, inst: InstanceId) {
    let p = player_of(w, inst);
    mm::send_localized_message(w, p, 1074944, "", 0x2A);
    engine::claim_rewards(w, inst);
}
fn the_joys_of_life_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1072787);
    q.description = td(1072832);
    q.refusal_message = td(1072833);
    q.in_progress_message = td(1072834);
    q.completion_message = td(1074177);
    q.objectives
        .push(Objective::collect(3, "ABauble", tds("arielle's baubles")));
    q.rewards.push(Reward::dummy(td(1072809)));
    q
}
pub static THE_JOYS_OF_LIFE: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.TheJoysOfLife",
    create: the_joys_of_life_create,
    record_completion: true,
    get_rewards: Some(the_joys_of_life_get_rewards),
    ..QuestClass::BASE
};

fn ingenuity_get_rewards(w: &mut World, inst: InstanceId) {
    let p = player_of(w, inst);
    mm::send_localized_message(w, p, 1074946, "", 0x2A);
    engine::claim_rewards(w, inst);
}
fn ingenuity_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1074350);
    q.description = td(1074462);
    q.refusal_message = td(1074508);
    q.in_progress_message = td(1074509);
    q.completion_message = td(1074510);
    q.completion_notice = completion_notice_short_return();
    q.objectives.push(Objective::collect(
        10,
        "PowerCrystal",
        tds("Power Crystals"),
    ));
    q.rewards.push(Reward::dummy(td(1074875)));
    q
}
pub static INGENUITY: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.Ingenuity",
    create: ingenuity_create,
    record_completion: true,
    get_rewards: Some(ingenuity_get_rewards),
    ..QuestClass::BASE
};

fn heave_ho_get_rewards(w: &mut World, inst: InstanceId) {
    let p = player_of(w, inst);
    mm::send_localized_message(w, p, 1074948, "", 0x2A);
    engine::claim_rewards(w, inst);
}
fn heave_ho_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1074351);
    q.description = td(1074519);
    q.refusal_message = td(1074521);
    q.in_progress_message = td(1074522);
    q.completion_message = td(1074523);
    q.completion_notice = completion_notice_short();
    q.objectives.push(
        Objective::deliver("CrateForSledge", 5, tds("Crates for Sledge"), "Sledge")
            .timed(TimeSpan::from_hours(1)),
    );
    q.rewards.push(Reward::dummy(td(1074875)));
    q
}
pub static HEAVE_HO: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.HeaveHo",
    create: heave_ho_create,
    record_completion: true,
    get_rewards: Some(heave_ho_get_rewards),
    ..QuestClass::BASE
};

fn human_in_need_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.title = td(1075011);
    q.description = td(0);
    q.refusal_message = td(0);
    q.in_progress_message = td(0);
    q
}
pub static HUMAN_IN_NEED: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.HumanInNeed",
    create: human_in_need_create,
    record_completion: true,
    ..QuestClass::BASE
};

fn all_season_adventurer_get_rewards(w: &mut World, inst: InstanceId) {
    let p = player_of(w, inst);
    mm::send_localized_message(w, p, 1074947, "", 0x2A);
    engine::claim_rewards(w, inst);
}
fn all_season_adventurer_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1074353);
    q.description = td(1074527);
    q.refusal_message = td(1074528);
    q.in_progress_message = td(1074529);
    q.completion_notice = completion_notice_short_return();
    q.objectives.push(Objective::kill(
        5,
        &["Efreet"],
        tds("efreets"),
        Some(QuestArea::new(td(1074808), "Fire")),
    ));
    q.objectives.push(Objective::kill(
        5,
        &["IceFiend"],
        tds("ice fiends"),
        Some(QuestArea::new(td(1074809), "Ice")),
    ));
    q.rewards.push(Reward::dummy(td(1074875)));
    q
}
pub static ALL_SEASON_ADVENTURER: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.AllSeasonAdventurer",
    create: all_season_adventurer_create,
    record_completion: true,
    get_rewards: Some(all_season_adventurer_get_rewards),
    ..QuestClass::BASE
};
