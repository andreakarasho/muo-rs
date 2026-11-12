//! The ML quest definitions of the New Haven Skill Training quest group: quest classes, objectives and rewards.

use super::special;
use crate::ml_quests::model::*;
use muo_server::skills::SkillName;

fn cleansing_old_haven_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.one_time_only = true;
    q.title = td(1077719);
    q.description = td(1077722);
    q.refusal_message = td(1077723);
    q.in_progress_message = td(1077724);
    q.completion_message = td(1077726);
    q.completion_notice = td(1077725);
    q.objectives
        .push(Objective::gain_skill(SkillName::Chivalry, 500, true, true));
    q.rewards
        .push(Reward::item(td(1077727), "BulwarkLeggings", 1));
    q
}
pub static CLEANSING_OLD_HAVEN: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.CleansingOldHaven",
    create: cleansing_old_haven_create,
    ..QuestClass::BASE
};

fn the_rudiments_of_self_defense_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.one_time_only = true;
    q.title = td(1077609);
    q.description = td(1077610);
    q.refusal_message = td(1077611);
    q.in_progress_message = td(1077630);
    q.completion_message = td(1077613);
    q.completion_notice = td(1077612);
    q.objectives
        .push(Objective::gain_skill(SkillName::Wrestling, 500, true, true));
    q.rewards
        .push(Reward::item(td(1077614), "GlovesOfSafeguarding", 1));
    q
}
pub static THE_RUDIMENTS_OF_SELF_DEFENSE: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.TheRudimentsOfSelfDefense",
    create: the_rudiments_of_self_defense_create,
    ..QuestClass::BASE
};

fn crushing_bones_and_taking_names_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.one_time_only = true;
    q.title = td(1078070);
    q.description = td(1078065);
    q.refusal_message = td(1078068);
    q.in_progress_message = td(1078067);
    q.completion_message = td(1078069);
    q.completion_notice = td(1078068);
    q.objectives
        .push(Objective::gain_skill(SkillName::Macing, 500, true, true));
    q.rewards
        .push(Reward::item(td(1078062), "ChurchillsWarMace", 1));
    q
}
pub static CRUSHING_BONES_AND_TAKING_NAMES: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.CrushingBonesAndTakingNames",
    create: crushing_bones_and_taking_names_create,
    ..QuestClass::BASE
};

fn swift_as_an_arrow_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.one_time_only = true;
    q.title = td(1078201);
    q.description = td(1078205);
    q.refusal_message = td(1078206);
    q.in_progress_message = td(1078207);
    q.completion_message = td(1078209);
    q.completion_notice = td(1078208);
    q.objectives
        .push(Objective::gain_skill(SkillName::Archery, 500, true, true));
    q.rewards.push(Reward::item(td(1078210), "Heartseeker", 1));
    q
}
pub static SWIFT_AS_AN_ARROW: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.SwiftAsAnArrow",
    create: swift_as_an_arrow_create,
    ..QuestClass::BASE
};

fn en_guarde_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.one_time_only = true;
    q.title = td(1078186);
    q.description = td(1078190);
    q.refusal_message = td(1078191);
    q.in_progress_message = td(1078192);
    q.completion_message = td(1078194);
    q.completion_notice = td(1078193);
    q.objectives
        .push(Objective::gain_skill(SkillName::Fencing, 500, true, true));
    q.rewards
        .push(Reward::item(td(1078195), "RecarosRiposte", 1));
    q
}
pub static EN_GUARDE: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.EnGuarde",
    create: en_guarde_create,
    ..QuestClass::BASE
};

fn the_art_of_war_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.one_time_only = true;
    q.title = td(1077667);
    q.description = td(1077670);
    q.refusal_message = td(1077671);
    q.in_progress_message = td(1077672);
    q.completion_message = td(1077674);
    q.completion_notice = td(1077673);
    q.objectives
        .push(Objective::gain_skill(SkillName::Tactics, 500, true, true));
    q.rewards
        .push(Reward::item(td(1077675), "ArmsOfArmstrong", 1));
    q
}
pub static THE_ART_OF_WAR: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.TheArtOfWar",
    create: the_art_of_war_create,
    ..QuestClass::BASE
};

fn the_way_of_the_blade_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.one_time_only = true;
    q.title = td(1077658);
    q.description = td(1077661);
    q.refusal_message = td(1077662);
    q.in_progress_message = td(1077663);
    q.completion_message = td(1077665);
    q.completion_notice = td(1077664);
    q.objectives
        .push(Objective::gain_skill(SkillName::Swords, 500, true, true));
    q.rewards
        .push(Reward::item(td(1077666), "JocklesQuicksword", 1));
    q
}
pub static THE_WAY_OF_THE_BLADE: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.TheWayOfTheBlade",
    create: the_way_of_the_blade_create,
    ..QuestClass::BASE
};

fn thou_and_thine_shield_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.one_time_only = true;
    q.title = td(1077704);
    q.description = td(1077707);
    q.refusal_message = td(1077708);
    q.in_progress_message = td(1077709);
    q.completion_message = td(1077711);
    q.completion_notice = td(1077710);
    q.objectives
        .push(Objective::gain_skill(SkillName::Parry, 500, true, true));
    q.rewards
        .push(Reward::item(td(1077694), "EscutcheonDeAriadne", 1));
    q
}
pub static THOU_AND_THINE_SHIELD: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.ThouAndThineShield",
    create: thou_and_thine_shield_create,
    ..QuestClass::BASE
};

fn defying_the_arcane_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.one_time_only = true;
    q.title = td(1077621);
    q.description = td(1077623);
    q.refusal_message = td(1077624);
    q.in_progress_message = td(1077632);
    q.completion_message = td(1077626);
    q.completion_notice = td(1077625);
    q.objectives.push(Objective::gain_skill(
        SkillName::MagicResist,
        500,
        true,
        true,
    ));
    q.rewards
        .push(Reward::item(td(1077627), "BraceletOfResilience", 1));
    q
}
pub static DEFYING_THE_ARCANE: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.DefyingTheArcane",
    create: defying_the_arcane_create,
    ..QuestClass::BASE
};

fn stopping_the_world_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.one_time_only = true;
    q.title = td(1077597);
    q.description = td(1077598);
    q.refusal_message = td(1077599);
    q.in_progress_message = td(1077628);
    q.completion_message = td(1077626);
    q.completion_notice = td(1077600);
    q.objectives.push(Objective::gain_skill(
        SkillName::Meditation,
        500,
        true,
        true,
    ));
    q.rewards
        .push(Reward::item(td(1077602), "PhilosophersHat", 1));
    q
}
pub static STOPPING_THE_WORLD: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.StoppingTheWorld",
    create: stopping_the_world_create,
    ..QuestClass::BASE
};

fn scribing_arcane_knowledge_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.one_time_only = true;
    q.title = td(1077615);
    q.description = td(1077616);
    q.refusal_message = td(1077617);
    q.in_progress_message = td(1077631);
    q.completion_message = td(1077619);
    q.completion_notice = td(1077618);
    q.objectives
        .push(Objective::gain_skill(SkillName::Inscribe, 500, true, true));
    q.rewards
        .push(Reward::item(td(1077620), "HallowedSpellbook", 1));
    q
}
pub static SCRIBING_ARCANE_KNOWLEDGE: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.ScribingArcaneKnowledge",
    create: scribing_arcane_knowledge_create,
    ..QuestClass::BASE
};

fn the_mages_apprentice_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.one_time_only = true;
    q.title = td(1077576);
    q.description = td(1077577);
    q.refusal_message = td(1077578);
    q.in_progress_message = td(1077579);
    q.completion_message = td(1077581);
    q.completion_notice = td(1077580);
    q.objectives
        .push(Objective::gain_skill(SkillName::Magery, 500, true, true));
    q.rewards.push(Reward::item(td(1077582), "EmberStaff", 1));
    q
}
pub static THE_MAGES_APPRENTICE: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.TheMagesApprentice",
    create: the_mages_apprentice_create,
    ..QuestClass::BASE
};

fn scholarly_task_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.one_time_only = true;
    q.title = td(1077603);
    q.description = td(1077604);
    q.refusal_message = td(1077605);
    q.in_progress_message = td(1077629);
    q.completion_message = td(1077607);
    q.completion_notice = td(1077606);
    q.objectives
        .push(Objective::gain_skill(SkillName::EvalInt, 500, true, true));
    q.rewards
        .push(Reward::item(td(1077608), "RingOfTheSavant", 1));
    q
}
pub static SCHOLARLY_TASK: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.ScholarlyTask",
    create: scholarly_task_create,
    ..QuestClass::BASE
};

fn the_right_tool_for_the_job_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.one_time_only = true;
    q.title = td(1077741);
    q.description = td(1077744);
    q.refusal_message = td(1077745);
    q.in_progress_message = td(1077746);
    q.completion_message = td(1077748);
    q.completion_notice = td(1077747);
    q.objectives
        .push(Objective::gain_skill(SkillName::Tinkering, 500, true, true));
    q.rewards
        .push(Reward::item(td(1077749), "AmeliasToolbox", 1));
    q
}
pub static THE_RIGHT_TOOL_FOR_THE_JOB: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.TheRightToolForTheJob",
    create: the_right_tool_for_the_job_create,
    ..QuestClass::BASE
};

fn know_thine_enemy_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.one_time_only = true;
    q.title = td(1077685);
    q.description = td(1077688);
    q.refusal_message = td(1077689);
    q.in_progress_message = td(1077690);
    q.completion_message = td(1077692);
    q.completion_notice = td(1077691);
    q.objectives
        .push(Objective::gain_skill(SkillName::Anatomy, 500, true, true));
    q.rewards
        .push(Reward::item(td(1077693), "TunicOfGuarding", 1));
    q
}
pub static KNOW_THINE_ENEMY: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.KnowThineEnemy",
    create: know_thine_enemy_create,
    ..QuestClass::BASE
};

fn bruises_bandages_and_blood_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.one_time_only = true;
    q.title = td(1077676);
    q.description = td(1077679);
    q.refusal_message = td(1077680);
    q.in_progress_message = td(1077681);
    q.completion_message = td(1077683);
    q.completion_notice = td(1077682);
    q.objectives
        .push(Objective::gain_skill(SkillName::Healing, 500, true, true));
    q.rewards.push(Reward::item(td(1077684), "HealersTouch", 1));
    q
}
pub static BRUISES_BANDAGES_AND_BLOOD: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.BruisesBandagesAndBlood",
    create: bruises_bandages_and_blood_create,
    ..QuestClass::BASE
};

fn the_inner_warrior_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.one_time_only = true;
    q.title = td(1077696);
    q.description = td(1077699);
    q.refusal_message = td(1077700);
    q.in_progress_message = td(1077701);
    q.completion_message = td(1077703);
    q.completion_notice = td(1077702);
    q.objectives
        .push(Objective::gain_skill(SkillName::Focus, 500, true, true));
    q.rewards
        .push(Reward::item(td(1077695), "ClaspOfConcentration", 1));
    q
}
pub static THE_INNER_WARRIOR: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.TheInnerWarrior",
    create: the_inner_warrior_create,
    ..QuestClass::BASE
};

fn the_art_of_stealth_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.one_time_only = true;
    q.title = td(1078154);
    q.description = td(1078158);
    q.refusal_message = td(1078159);
    q.in_progress_message = td(1078160);
    q.completion_message = td(1078162);
    q.completion_notice = td(1078161);
    q.objectives
        .push(Objective::gain_skill(SkillName::Ninjitsu, 500, true, true));
    q.rewards
        .push(Reward::item(td(1078163), "SilverSerpentBlade", 1));
    q
}
pub static THE_ART_OF_STEALTH: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.TheArtOfStealth",
    create: the_art_of_stealth_create,
    ..QuestClass::BASE
};

fn becoming_one_with_the_shadows_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.one_time_only = true;
    q.title = td(1078164);
    q.description = td(1078168);
    q.refusal_message = td(1078169);
    q.in_progress_message = td(1078170);
    q.completion_message = td(1078172);
    q.completion_notice = td(1078171);
    q.objectives
        .push(Objective::gain_skill(SkillName::Hiding, 500, true, true));
    q.rewards
        .push(Reward::item(td(1078173), "BagOfSmokeBombs", 1));
    q
}
pub static BECOMING_ONE_WITH_THE_SHADOWS: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.BecomingOneWithTheShadows",
    create: becoming_one_with_the_shadows_create,
    ..QuestClass::BASE
};

fn walking_silently_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.one_time_only = true;
    q.title = td(1078174);
    q.description = td(1078178);
    q.refusal_message = td(1078179);
    q.in_progress_message = td(1078180);
    q.completion_message = td(1078182);
    q.completion_notice = td(1078181);
    q.objectives
        .push(Objective::gain_skill(SkillName::Stealth, 500, true, true));
    q.rewards
        .push(Reward::item(td(1078183), "TwilightJacket", 1));
    q
}
pub static WALKING_SILENTLY: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.WalkingSilently",
    create: walking_silently_create,
    ..QuestClass::BASE
};

fn eyes_of_a_ranger_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.one_time_only = true;
    q.title = td(1078211);
    q.description = td(1078217);
    q.refusal_message = td(1078218);
    q.in_progress_message = td(1078219);
    q.completion_message = td(1078221);
    q.completion_notice = td(1078220);
    q.objectives
        .push(Objective::gain_skill(SkillName::Tracking, 500, true, true));
    q.rewards
        .push(Reward::item(td(1078222), "WalkersLeggings", 1));
    q
}
pub static EYES_OF_A_RANGER: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.EyesOfARanger",
    create: eyes_of_a_ranger_create,
    ..QuestClass::BASE
};

fn the_way_of_the_samurai_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.one_time_only = true;
    q.title = td(1078007);
    q.description = td(1078010);
    q.refusal_message = td(1078011);
    q.in_progress_message = td(1078012);
    q.completion_message = td(1078014);
    q.completion_notice = td(1078013);
    q.objectives
        .push(Objective::gain_skill(SkillName::Bushido, 500, true, true));
    q.rewards
        .push(Reward::item(td(1078015), "TheDragonsTail", 1));
    q
}
pub static THE_WAY_OF_THE_SAMURAI: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.TheWayOfTheSamurai",
    create: the_way_of_the_samurai_create,
    ..QuestClass::BASE
};

fn the_allure_of_dark_magic_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.one_time_only = true;
    q.title = td(1078036);
    q.description = td(1078039);
    q.refusal_message = td(1078040);
    q.in_progress_message = td(1078041);
    q.completion_message = td(1078043);
    q.completion_notice = td(1078042);
    q.objectives.push(Objective::gain_skill(
        SkillName::Necromancy,
        500,
        true,
        true,
    ));
    q.rewards.push(special::the_allure_of_dark_magic_reward());
    q
}
pub static THE_ALLURE_OF_DARK_MAGIC: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.TheAllureOfDarkMagic",
    create: the_allure_of_dark_magic_create,
    ..QuestClass::BASE
};

fn channeling_the_supernatural_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.one_time_only = true;
    q.title = td(1078044);
    q.description = td(1078047);
    q.refusal_message = td(1078048);
    q.in_progress_message = td(1078049);
    q.completion_message = td(1078051);
    q.completion_notice = td(1078050);
    q.objectives.push(Objective::gain_skill(
        SkillName::SpiritSpeak,
        500,
        true,
        true,
    ));
    q.rewards
        .push(Reward::item(td(1078053), "BagOfNecromancerReagents", 1));
    q
}
pub static CHANNELING_THE_SUPERNATURAL: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.ChannelingTheSupernatural",
    create: channeling_the_supernatural_create,
    ..QuestClass::BASE
};

fn the_delucians_lost_mine_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.one_time_only = true;
    q.title = td(1077750);
    q.description = td(1077753);
    q.refusal_message = td(1077754);
    q.in_progress_message = td(1077755);
    q.completion_message = td(1077757);
    q.completion_notice = td(1077756);
    q.objectives
        .push(Objective::gain_skill(SkillName::Mining, 500, true, true));
    q.rewards
        .push(Reward::item(td(1077758), "JacobsPickaxe", 1));
    q
}
pub static THE_DELUCIANS_LOST_MINE: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.TheDeluciansLostMine",
    create: the_delucians_lost_mine_create,
    ..QuestClass::BASE
};

fn its_hammer_time_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.one_time_only = true;
    q.title = td(1077732);
    q.description = td(1077735);
    q.refusal_message = td(1077736);
    q.in_progress_message = td(1077737);
    q.completion_message = td(1077739);
    q.completion_notice = td(1077738);
    q.objectives.push(Objective::gain_skill(
        SkillName::Blacksmith,
        500,
        true,
        true,
    ));
    q.rewards
        .push(Reward::item(td(1077740), "HammerOfHephaestus", 1));
    q
}
pub static ITS_HAMMER_TIME: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.ItsHammerTime",
    create: its_hammer_time_create,
    ..QuestClass::BASE
};
