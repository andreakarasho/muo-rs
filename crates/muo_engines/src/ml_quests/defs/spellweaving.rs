//! The ML quest definitions of the Spellweaving quest group: quest classes, objectives and rewards.

use super::{player_of, special};
use crate::ml_quests::engine;
use crate::ml_quests::model::*;
use muo_primitives::TimeSpan;
use muo_server::mobile::part2::messages as mm;
use muo_server::world::World;

fn patience_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1072753);
    q.description = td(1072762);
    q.refusal_message = td(1072767);
    q.in_progress_message = td(1072774);
    q.completion_message = td(1074166);
    q.objectives.push(
        Objective::collect(20, "MiniatureMushroom", tds("miniature mushrooms"))
            .timed(TimeSpan::from_hours(1)),
    );
    q.rewards.push(Reward::dummy(td(1074872)));
    q
}
pub static PATIENCE: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.Patience",
    create: patience_create,
    next_quest: Some("NeedsOfTheManyHeartwood1"),
    ..QuestClass::BASE
};

fn needs_of_the_many_heartwood1_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1072797);
    q.description = td(1072763);
    q.refusal_message = td(1072768);
    q.in_progress_message = td(1072775);
    q.completion_message = td(1074110);
    q.objectives
        .push(Objective::collect(10, "Cotton", td(1023577)));
    q.rewards.push(Reward::dummy(td(1074872)));
    q
}
pub static NEEDS_OF_THE_MANY_HEARTWOOD1: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.NeedsOfTheManyHeartwood1",
    create: needs_of_the_many_heartwood1_create,
    next_quest: Some("NeedsOfTheManyHeartwood2"),
    chain_triggered: true,
    ..QuestClass::BASE
};

fn needs_of_the_many_heartwood2_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1072797);
    q.description = td(1072764);
    q.refusal_message = td(1072769);
    q.in_progress_message = td(1072776);
    q.completion_message = td(1074152);
    q.objectives
        .push(Objective::collect(250, "Board", td(1027127)));
    q.rewards.push(Reward::dummy(td(1074872)));
    q
}
pub static NEEDS_OF_THE_MANY_HEARTWOOD2: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.NeedsOfTheManyHeartwood2",
    create: needs_of_the_many_heartwood2_create,
    next_quest: Some("MakingAContributionHeartwood"),
    chain_triggered: true,
    ..QuestClass::BASE
};

fn making_a_contribution_heartwood_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1072798);
    q.description = td(1072765);
    q.refusal_message = td(1072770);
    q.in_progress_message = td(1072777);
    q.completion_message = td(1074158);
    q.objectives
        .push(Objective::collect(1, "SackFlour", td(1024153)));
    q.objectives
        .push(Objective::collect(10, "JarHoney", td(1022540)));
    q.objectives
        .push(Objective::collect(20, "FishSteak", td(1022427)));
    q.rewards.push(Reward::dummy(td(1074872)));
    q
}
pub static MAKING_A_CONTRIBUTION_HEARTWOOD: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.MakingAContributionHeartwood",
    create: making_a_contribution_heartwood_create,
    next_quest: Some("UnnaturalCreations"),
    chain_triggered: true,
    ..QuestClass::BASE
};

fn unnatural_creations_get_rewards(w: &mut World, inst: InstanceId) {
    let p = player_of(w, inst);
    special::spellweaving_award_to(w, p);
    engine::base_get_rewards(w, inst);
}
fn unnatural_creations_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1072758);
    q.description = td(1072780);
    q.refusal_message = td(1072771);
    q.in_progress_message = td(1072779);
    q.completion_message = td(1074167);
    q.objectives.push(Objective::kill(
        5,
        &["ExodusOverseer"],
        tds("Exodus Overseers"),
        None,
    ));
    q.objectives.push(Objective::kill(
        2,
        &["ExodusMinion"],
        tds("Exodus Minions"),
        None,
    ));
    q.rewards
        .push(Reward::item(td(1031601), "ArcaneCircleScroll", 1));
    q.rewards
        .push(Reward::item(td(1031600), "SpellweavingBook", 1));
    q.rewards
        .push(Reward::item(td(1031602), "GiftOfRenewalScroll", 1));
    q
}
pub static UNNATURAL_CREATIONS: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.UnnaturalCreations",
    create: unnatural_creations_create,
    chain_triggered: true,
    get_rewards: Some(unnatural_creations_get_rewards),
    ..QuestClass::BASE
};

fn discipline_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1072752);
    q.description = td(1072761);
    q.refusal_message = td(1072767);
    q.in_progress_message = td(1072773);
    q.objectives.push(
        Objective::kill(
            50,
            &["Rat"],
            tds("rats"),
            Some(QuestArea::new(td(1074807), "Sanctuary")),
        )
        .timed(TimeSpan::from_hours(1)),
    );
    q.rewards.push(Reward::dummy(td(1074872)));
    q
}
pub static DISCIPLINE: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.Discipline",
    create: discipline_create,
    next_quest: Some("NeedsOfTheManySanctuary"),
    ..QuestClass::BASE
};

fn needs_of_the_many_sanctuary_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1072754);
    q.description = td(1072763);
    q.refusal_message = td(1072768);
    q.in_progress_message = td(1072775);
    q.completion_message = td(1074110);
    q.objectives
        .push(Objective::collect(10, "Cotton", td(1023577)));
    q.rewards.push(Reward::dummy(td(1074872)));
    q
}
pub static NEEDS_OF_THE_MANY_SANCTUARY: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.NeedsOfTheManySanctuary",
    create: needs_of_the_many_sanctuary_create,
    next_quest: Some("MakingAContributionSanctuary"),
    chain_triggered: true,
    ..QuestClass::BASE
};

fn making_a_contribution_sanctuary_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1072755);
    q.description = td(1072764);
    q.refusal_message = td(1072769);
    q.in_progress_message = td(1072776);
    q.completion_message = td(1074152);
    q.objectives
        .push(Objective::collect(250, "Board", td(1027127)));
    q.rewards.push(Reward::dummy(td(1074872)));
    q
}
pub static MAKING_A_CONTRIBUTION_SANCTUARY: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.MakingAContributionSanctuary",
    create: making_a_contribution_sanctuary_create,
    next_quest: Some("SuppliesForSanctuary"),
    chain_triggered: true,
    ..QuestClass::BASE
};

fn supplies_for_sanctuary_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1072756);
    q.description = td(1072765);
    q.refusal_message = td(1072770);
    q.in_progress_message = td(1072777);
    q.completion_message = td(1074158);
    q.objectives
        .push(Objective::collect(1, "SackFlour", td(1024153)));
    q.objectives
        .push(Objective::collect(10, "JarHoney", td(1022540)));
    q.objectives
        .push(Objective::collect(20, "FishSteak", td(1022427)));
    q.rewards.push(Reward::dummy(td(1074872)));
    q
}
pub static SUPPLIES_FOR_SANCTUARY: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.SuppliesForSanctuary",
    create: supplies_for_sanctuary_create,
    next_quest: Some("TheHumanBlight"),
    chain_triggered: true,
    ..QuestClass::BASE
};

fn the_human_blight_get_rewards(w: &mut World, inst: InstanceId) {
    let p = player_of(w, inst);
    special::spellweaving_award_to(w, p);
    engine::base_get_rewards(w, inst);
}
fn the_human_blight_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1072757);
    q.description = td(1072766);
    q.refusal_message = td(1072771);
    q.in_progress_message = td(1072778);
    q.completion_message = td(1074160);
    q.objectives
        .push(Objective::collect(30, "SeveredHumanEars", td(1032591)));
    q.rewards
        .push(Reward::item(td(1031601), "ArcaneCircleScroll", 1));
    q.rewards
        .push(Reward::item(td(1031600), "SpellweavingBook", 1));
    q.rewards
        .push(Reward::item(td(1031602), "GiftOfRenewalScroll", 1));
    q
}
pub static THE_HUMAN_BLIGHT: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.TheHumanBlight",
    create: the_human_blight_create,
    chain_triggered: true,
    get_rewards: Some(the_human_blight_get_rewards),
    ..QuestClass::BASE
};

fn friend_of_the_fey_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1074284);
    q.description = td(1074286);
    q.refusal_message = td(1074288);
    q.in_progress_message = td(1074290);
    q.completion_message = td(1074292);
    q.objectives
        .push(Objective::collect(1, "Beads", td(1024235)));
    q.objectives
        .push(Objective::collect(1, "JarHoney", td(1022540)));
    q.rewards.push(Reward::dummy(td(1074874)));
    q
}
pub static FRIEND_OF_THE_FEY: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.FriendOfTheFey",
    create: friend_of_the_fey_create,
    next_quest: Some("TokenOfFriendship"),
    ..QuestClass::BASE
};

fn token_of_friendship_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1074293);
    q.description = td(1074297);
    q.refusal_message = td(1074310);
    q.in_progress_message = td(1074315);
    q.completion_message = td(1074319);
    q.objectives.push(Objective::deliver(
        "GiftForArielle",
        1,
        tds("gift for Arielle"),
        "Arielle",
    ));
    q.rewards.push(Reward::dummy(td(1074874)));
    q
}
pub static TOKEN_OF_FRIENDSHIP: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.TokenOfFriendship",
    create: token_of_friendship_create,
    next_quest: Some("Alliance"),
    chain_triggered: true,
    ..QuestClass::BASE
};

fn alliance_get_rewards(w: &mut World, inst: InstanceId) {
    let p = player_of(w, inst);
    engine::get_or_create_context(w, p).set_flag(flag::SUMMON_FEY, true);
    let p = player_of(w, inst);
    mm::send_localized_message(w, p, 1074320, "", 0x2A);
    engine::base_get_rewards(w, inst);
}
fn alliance_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1074294);
    q.description = td(1074298);
    q.refusal_message = td(1074311);
    q.in_progress_message = td(1074316);
    q.completion_notice = completion_notice_short_return();
    q.objectives
        .push(Objective::kill(20, &["Reaper"], tds("reapers"), None));
    q.rewards
        .push(Reward::item(td(1031607), "SummonFeyScroll", 1));
    q
}
pub static ALLIANCE: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.Alliance",
    create: alliance_create,
    chain_triggered: true,
    get_rewards: Some(alliance_get_rewards),
    ..QuestClass::BASE
};

fn fiendish_friends_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1074283);
    q.description = td(1074285);
    q.refusal_message = td(1074287);
    q.in_progress_message = td(1074289);
    q.completion_message = td(1074291);
    q.objectives
        .push(Objective::kill(50, &["Imp"], tds("imps"), None));
    q.rewards.push(Reward::dummy(td(107487)));
    q
}
pub static FIENDISH_FRIENDS: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.FiendishFriends",
    create: fiendish_friends_create,
    next_quest: Some("CrackingTheWhipI"),
    ..QuestClass::BASE
};

fn cracking_the_whip_i_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1074295);
    q.description = td(1074300);
    q.refusal_message = td(1074313);
    q.in_progress_message = td(1074317);
    q.completion_message = td(1074321);
    q.objectives
        .push(Objective::collect(1, "StoutWhip", tds("Stout Whip")));
    q.rewards.push(Reward::dummy(td(1074873)));
    q
}
pub static CRACKING_THE_WHIP_I: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.CrackingTheWhipI",
    create: cracking_the_whip_i_create,
    next_quest: Some("CrackingTheWhipII"),
    chain_triggered: true,
    ..QuestClass::BASE
};

fn cracking_the_whip_ii_get_rewards(w: &mut World, inst: InstanceId) {
    let p = player_of(w, inst);
    engine::get_or_create_context(w, p).set_flag(flag::SUMMON_FIEND, true);
    let p = player_of(w, inst);
    mm::send_localized_message(w, p, 1074322, "", 0x2A);
    engine::base_get_rewards(w, inst);
}
fn cracking_the_whip_ii_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1074295);
    q.description = td(1074302);
    q.refusal_message = td(1074314);
    q.in_progress_message = td(1074318);
    q.objectives
        .push(Objective::kill(1, &["ArcaneDaemon"], td(1029733), None));
    q.rewards
        .push(Reward::item(td(1031608), "SummonFiendScroll", 1));
    q
}
pub static CRACKING_THE_WHIP_II: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.CrackingTheWhipII",
    create: cracking_the_whip_ii_create,
    chain_triggered: true,
    get_rewards: Some(cracking_the_whip_ii_get_rewards),
    ..QuestClass::BASE
};
