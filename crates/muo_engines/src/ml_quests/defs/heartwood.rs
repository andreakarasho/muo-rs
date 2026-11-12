//! The ML quest definitions of the Heartwood quest group: quest classes, objectives and rewards.

use crate::ml_quests::model::*;

fn theyre_breeding_like_rabbits_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1072244);
    q.description = td(1072259);
    q.refusal_message = td(1072270);
    q.in_progress_message = td(1072271);
    q.objectives
        .push(Objective::kill(10, &["Rabbit"], tds("rabbits"), None));
    q.rewards.push(rewards::small_bag_of_trinkets());
    q
}
pub static THEYRE_BREEDING_LIKE_RABBITS: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.TheyreBreedingLikeRabbits",
    create: theyre_breeding_like_rabbits_create,
    ..QuestClass::BASE
};

fn theyll_eat_anything_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1072248);
    q.description = td(1072262);
    q.refusal_message = td(1072270);
    q.in_progress_message = td(1072271);
    q.objectives
        .push(Objective::kill(10, &["Pig"], tds("pigs"), None));
    q.rewards.push(rewards::small_bag_of_trinkets());
    q
}
pub static THEYLL_EAT_ANYTHING: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.TheyllEatAnything",
    create: theyll_eat_anything_create,
    ..QuestClass::BASE
};

fn no_good_fish_stealing_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1072251);
    q.description = td(1072265);
    q.refusal_message = td(1072270);
    q.in_progress_message = td(1072271);
    q.objectives
        .push(Objective::kill(10, &["Walrus"], tds("walruses"), None));
    q.rewards.push(rewards::small_bag_of_trinkets());
    q
}
pub static NO_GOOD_FISH_STEALING: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.NoGoodFishStealing",
    create: no_good_fish_stealing_create,
    ..QuestClass::BASE
};

fn a_hero_in_the_making_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1072246);
    q.description = td(1072257);
    q.refusal_message = td(1072270);
    q.in_progress_message = td(1072271);
    q.objectives
        .push(Objective::kill(10, &["Mongbat"], tds("mongbats"), None));
    q.rewards.push(rewards::small_bag_of_trinkets());
    q
}
pub static A_HERO_IN_THE_MAKING: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.AHeroInTheMaking",
    create: a_hero_in_the_making_create,
    ..QuestClass::BASE
};

fn bullfighting_sort_of_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1072247);
    q.description = td(1072254);
    q.refusal_message = td(1072270);
    q.in_progress_message = td(1072271);
    q.objectives
        .push(Objective::kill(10, &["Bull"], tds("bulls"), None));
    q.rewards.push(rewards::small_bag_of_trinkets());
    q
}
pub static BULLFIGHTING_SORT_OF: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.BullfightingSortOf",
    create: bullfighting_sort_of_create,
    ..QuestClass::BASE
};

fn a_fine_feast_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1072243);
    q.description = td(1072261);
    q.refusal_message = td(1072270);
    q.in_progress_message = td(1072271);
    q.objectives
        .push(Objective::kill(10, &["Sheep"], tds("sheep"), None));
    q.rewards.push(rewards::small_bag_of_trinkets());
    q
}
pub static A_FINE_FEAST: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.AFineFeast",
    create: a_fine_feast_create,
    ..QuestClass::BASE
};

fn forced_migration_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1072250);
    q.description = td(1072264);
    q.refusal_message = td(1072270);
    q.in_progress_message = td(1072271);
    q.objectives
        .push(Objective::kill(10, &["Bird"], tds("birds"), None));
    q.rewards.push(rewards::small_bag_of_trinkets());
    q
}
pub static FORCED_MIGRATION: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.ForcedMigration",
    create: forced_migration_create,
    ..QuestClass::BASE
};

fn filthy_pests_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1072242);
    q.description = td(1072253);
    q.refusal_message = td(1072270);
    q.in_progress_message = td(1072271);
    q.objectives
        .push(Objective::kill(10, &["SewerRat"], tds("sewer rats"), None));
    q.rewards.push(rewards::small_bag_of_trinkets());
    q
}
pub static FILTHY_PESTS: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.FilthyPests",
    create: filthy_pests_create,
    ..QuestClass::BASE
};

fn dead_man_walking_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1072983);
    q.description = td(1073009);
    q.refusal_message = td(1072270);
    q.in_progress_message = td(1072271);
    q.objectives
        .push(Objective::kill(5, &["Zombie"], tds("zombies"), None));
    q.objectives
        .push(Objective::kill(5, &["Skeleton"], tds("skeletons"), None));
    q.rewards.push(rewards::bag_of_trinkets());
    q
}
pub static DEAD_MAN_WALKING: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.DeadManWalking",
    create: dead_man_walking_create,
    ..QuestClass::BASE
};

fn king_of_bears_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1072996);
    q.description = td(1073030);
    q.refusal_message = td(1072270);
    q.in_progress_message = td(1072271);
    q.objectives.push(Objective::kill(
        10,
        &["GrizzlyBear"],
        tds("grizzly bears"),
        None,
    ));
    q.rewards.push(rewards::bag_of_trinkets());
    q
}
pub static KING_OF_BEARS: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.KingOfBears",
    create: king_of_bears_create,
    ..QuestClass::BASE
};

fn specimens_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1072999);
    q.description = td(1073032);
    q.refusal_message = td(1072270);
    q.in_progress_message = td(1072271);
    q.objective_type = ObjectiveType::Any;
    q.objectives.push(Objective::kill(
        12,
        &["RedSolenWorker"],
        tds("red solen workers"),
        None,
    ));
    q.objectives.push(Objective::kill(
        12,
        &["BlackSolenWorker"],
        tds("black solen workers"),
        None,
    ));
    q.rewards.push(rewards::bag_of_trinkets());
    q
}
pub static SPECIMENS: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.Specimens",
    create: specimens_create,
    ..QuestClass::BASE
};

fn spirits_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1073076);
    q.description = td(1073566);
    q.refusal_message = td(1073580);
    q.in_progress_message = td(1073586);
    q.objectives.push(Objective::kill(
        15,
        &["Spectre", "Shade", "Wraith"],
        tds("spectres or shades or wraiths"),
        None,
    ));
    q.rewards.push(rewards::bag_of_treasure());
    q
}
pub static SPIRITS: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.Spirits",
    create: spirits_create,
    ..QuestClass::BASE
};

fn roll_the_bones_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1073002);
    q.description = td(1073011);
    q.refusal_message = td(1072270);
    q.in_progress_message = td(1072271);
    q.objectives.push(Objective::kill(
        8,
        &["PatchworkSkeleton"],
        tds("patchwork skeletons"),
        None,
    ));
    q.rewards.push(rewards::bag_of_trinkets());
    q
}
pub static ROLL_THE_BONES: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.RollTheBones",
    create: roll_the_bones_create,
    ..QuestClass::BASE
};

fn its_a_ghastly_job_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1073008);
    q.description = td(1073012);
    q.refusal_message = td(1072270);
    q.in_progress_message = td(1072271);
    q.objectives
        .push(Objective::kill(12, &["Ghoul"], tds("ghouls"), None));
    q.rewards.push(rewards::bag_of_trinkets());
    q
}
pub static ITS_A_GHASTLY_JOB: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.ItsAGhastlyJob",
    create: its_a_ghastly_job_create,
    ..QuestClass::BASE
};

fn troglodytes_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1074688);
    q.description = td(1074689);
    q.refusal_message = td(1074690);
    q.in_progress_message = td(1074691);
    q.objectives.push(Objective::kill(
        12,
        &["Troglodyte"],
        tds("troglodytes"),
        None,
    ));
    q.rewards.push(rewards::bag_of_trinkets());
    q
}
pub static TROGLODYTES: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.Troglodytes",
    create: troglodytes_create,
    ..QuestClass::BASE
};

fn unholy_knights_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1073075);
    q.description = td(1073565);
    q.refusal_message = td(1073580);
    q.in_progress_message = td(1073585);
    q.objectives.push(Objective::kill(
        16,
        &["BoneKnight", "SkeletalKnight"],
        tds("bone knights or skeletal knights"),
        None,
    ));
    q.rewards.push(rewards::bag_of_treasure());
    q
}
pub static UNHOLY_KNIGHTS: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.UnholyKnights",
    create: unholy_knights_create,
    ..QuestClass::BASE
};

fn a_feather_in_yer_cap_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1074738);
    q.description = td(1074737);
    q.refusal_message = td(1074736);
    q.in_progress_message = td(1074735);
    q.completion_message = td(1074734);
    q.objectives.push(Objective::collect(
        1,
        "SalivasFeather",
        tds("Saliva's Feather"),
    ));
    q.rewards.push(rewards::bag_of_treasure());
    q
}
pub static A_FEATHER_IN_YER_CAP: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.AFeatherInYerCap",
    create: a_feather_in_yer_cap_create,
    ..QuestClass::BASE
};

fn a_tale_of_tail_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1074726);
    q.description = td(1074727);
    q.refusal_message = td(1074728);
    q.in_progress_message = td(1074729);
    q.completion_message = td(1074730);
    q.objectives
        .push(Objective::collect(1, "AbscessTail", tds("Abscess' Tail")));
    q.rewards.push(rewards::bag_of_treasure());
    q
}
pub static A_TALE_OF_TAIL: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.ATaleOfTail",
    create: a_tale_of_tail_create,
    ..QuestClass::BASE
};

fn a_trog_and_his_dog_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1074681);
    q.description = td(1074680);
    q.refusal_message = td(1074655);
    q.in_progress_message = td(1074682);
    q.completion_message = td(1074683);
    q.objectives
        .push(Objective::kill(1, &["Lurg"], tds("Lurg"), None));
    q.objectives
        .push(Objective::kill(1, &["Grobu"], tds("Grobu"), None));
    q.rewards.push(rewards::large_bag_of_treasure());
    q
}
pub static A_TROG_AND_HIS_DOG: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.ATrogAndHisDog",
    create: a_trog_and_his_dog_create,
    ..QuestClass::BASE
};

fn overpopulation_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1072252);
    q.description = td(1072267);
    q.refusal_message = td(1072270);
    q.in_progress_message = td(1072271);
    q.objectives
        .push(Objective::kill(10, &["Hind"], tds("hinds"), None));
    q.rewards.push(rewards::small_bag_of_trinkets());
    q
}
pub static OVERPOPULATION: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.Overpopulation",
    create: overpopulation_create,
    ..QuestClass::BASE
};

fn wild_boar_cull_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1072245);
    q.description = td(1072260);
    q.refusal_message = td(1072270);
    q.in_progress_message = td(1072271);
    q.objectives
        .push(Objective::kill(10, &["Boar"], tds("boars"), None));
    q.rewards.push(rewards::small_bag_of_trinkets());
    q
}
pub static WILD_BOAR_CULL: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.WildBoarCull",
    create: wild_boar_cull_create,
    ..QuestClass::BASE
};

fn its_elemental_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1073089);
    q.description = td(1073579);
    q.refusal_message = td(1073580);
    q.in_progress_message = td(1073599);
    q.objectives.push(Objective::kill(
        4,
        &["FireElemental"],
        tds("fire elementals"),
        None,
    ));
    q.objectives.push(Objective::kill(
        4,
        &["WaterElemental"],
        tds("water elementals"),
        None,
    ));
    q.objectives.push(Objective::kill(
        4,
        &["EarthElemental"],
        tds("earth elementals"),
        None,
    ));
    q.rewards.push(rewards::bag_of_treasure());
    q
}
pub static ITS_ELEMENTAL: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.ItsElemental",
    create: its_elemental_create,
    ..QuestClass::BASE
};

fn circle_of_life_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1073656);
    q.description = td(1073695);
    q.refusal_message = td(1073733);
    q.in_progress_message = td(1073736);
    q.objectives
        .push(Objective::kill(8, &["BogThing"], tds("bog things"), None));
    q.rewards.push(rewards::large_bag_of_treasure());
    q
}
pub static CIRCLE_OF_LIFE: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.CircleOfLife",
    create: circle_of_life_create,
    ..QuestClass::BASE
};

fn dust_to_dust_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1073074);
    q.description = td(1073564);
    q.refusal_message = td(1073580);
    q.in_progress_message = td(1073584);
    q.objectives.push(Objective::kill(
        12,
        &["EarthElemental"],
        tds("earth elementals"),
        None,
    ));
    q.rewards.push(rewards::bag_of_treasure());
    q
}
pub static DUST_TO_DUST: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.DustToDust",
    create: dust_to_dust_create,
    ..QuestClass::BASE
};

fn creepy_crawlies_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1072987);
    q.description = td(1073016);
    q.refusal_message = td(1072270);
    q.in_progress_message = td(1072271);
    q.objectives.push(Objective::kill(
        12,
        &["GiantSpider"],
        tds("giant spiders"),
        None,
    ));
    q.rewards.push(rewards::bag_of_trinkets());
    q
}
pub static CREEPY_CRAWLIES: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.CreepyCrawlies",
    create: creepy_crawlies_create,
    ..QuestClass::BASE
};

fn voracious_plants_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1073001);
    q.description = td(1073024);
    q.refusal_message = td(1073019);
    q.in_progress_message = td(1072271);
    q.objectives
        .push(Objective::kill(8, &["Corpser"], tds("corpsers"), None));
    q.objectives.push(Objective::kill(
        2,
        &["SwampTentacle"],
        tds("swamp tentacles"),
        None,
    ));
    q.rewards.push(rewards::bag_of_trinkets());
    q
}
pub static VORACIOUS_PLANTS: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.VoraciousPlants",
    create: voracious_plants_create,
    ..QuestClass::BASE
};

fn gibber_jabber_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1073004);
    q.description = td(1073024);
    q.refusal_message = td(1073019);
    q.in_progress_message = td(1072271);
    q.objectives.push(Objective::kill(
        10,
        &["Gibberling"],
        tds("gibberlings"),
        None,
    ));
    q.rewards.push(rewards::bag_of_trinkets());
    q
}
pub static GIBBER_JABBER: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.GibberJabber",
    create: gibber_jabber_create,
    ..QuestClass::BASE
};

fn animated_monstrosity_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1072990);
    q.description = td(1073020);
    q.refusal_message = td(1073019);
    q.in_progress_message = td(1072271);
    q.objectives.push(Objective::kill(
        12,
        &["FleshGolem"],
        tds("flesh golems"),
        None,
    ));
    q.rewards.push(rewards::bag_of_trinkets());
    q
}
pub static ANIMATED_MONSTROSITY: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.AnimatedMonstrosity",
    create: animated_monstrosity_create,
    ..QuestClass::BASE
};

fn birds_of_a_feather_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1073007);
    q.description = td(1073022);
    q.refusal_message = td(1073019);
    q.in_progress_message = td(1072271);
    q.objectives
        .push(Objective::kill(10, &["Harpy"], tds("harpies"), None));
    q.rewards.push(rewards::bag_of_trinkets());
    q
}
pub static BIRDS_OF_A_FEATHER: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.BirdsOfAFeather",
    create: birds_of_a_feather_create,
    ..QuestClass::BASE
};

fn frightmares_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1073000);
    q.description = td(1073036);
    q.refusal_message = td(1073019);
    q.in_progress_message = td(1072271);
    q.objectives.push(Objective::kill(
        10,
        &["PlagueSpawn"],
        tds("plague spawns"),
        None,
    ));
    q.rewards.push(rewards::bag_of_trinkets());
    q
}
pub static FRIGHTMARES: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.Frightmares",
    create: frightmares_create,
    ..QuestClass::BASE
};

fn molten_reptiles_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1072989);
    q.description = td(1073018);
    q.refusal_message = td(1073019);
    q.in_progress_message = td(1072271);
    q.objectives.push(Objective::kill(
        10,
        &["LavaLizard"],
        tds("lava lizards"),
        None,
    ));
    q.rewards.push(rewards::bag_of_trinkets());
    q
}
pub static MOLTEN_REPTILES: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.MoltenReptiles",
    create: molten_reptiles_create,
    ..QuestClass::BASE
};

fn bloody_nuisance_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1072992);
    q.description = td(1073021);
    q.refusal_message = td(1073019);
    q.in_progress_message = td(1072271);
    q.objectives.push(Objective::kill(
        10,
        &["GoreFiend"],
        tds("gore fiends"),
        None,
    ));
    q.rewards.push(rewards::bag_of_trinkets());
    q
}
pub static BLOODY_NUISANCE: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.BloodyNuisance",
    create: bloody_nuisance_create,
    ..QuestClass::BASE
};

fn blood_suckers_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1072997);
    q.description = td(1073025);
    q.refusal_message = td(1073019);
    q.in_progress_message = td(1072271);
    q.objectives.push(Objective::kill(
        10,
        &["VampireBat"],
        tds("vampire bats"),
        None,
    ));
    q.rewards.push(rewards::bag_of_trinkets());
    q
}
pub static BLOOD_SUCKERS: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.BloodSuckers",
    create: blood_suckers_create,
    ..QuestClass::BASE
};

fn the_afterlife_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1073073);
    q.description = td(1073563);
    q.refusal_message = td(1073580);
    q.in_progress_message = td(1073583);
    q.objectives
        .push(Objective::kill(15, &["Mummy"], tds("mummies"), None));
    q.rewards.push(rewards::bag_of_treasure());
    q
}
pub static THE_AFTERLIFE: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.TheAfterlife",
    create: the_afterlife_create,
    ..QuestClass::BASE
};

fn forked_tongue_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1073655);
    q.description = td(1073694);
    q.refusal_message = td(1073733);
    q.in_progress_message = td(1073735);
    q.objectives.push(Objective::kill(
        10,
        &["OphidianKnight"],
        tds("ophidian avengers or ophidian knight-errants"),
        None,
    ));
    q.rewards.push(rewards::large_bag_of_treasure());
    q
}
pub static FORKED_TONGUE: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.ForkedTongue",
    create: forked_tongue_create,
    ..QuestClass::BASE
};

fn impish_delights_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1073077);
    q.description = td(1073567);
    q.refusal_message = td(1073580);
    q.in_progress_message = td(1073587);
    q.objectives
        .push(Objective::kill(12, &["Imp"], tds("imps"), None));
    q.rewards.push(rewards::bag_of_treasure());
    q
}
pub static IMPISH_DELIGHTS: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.ImpishDelights",
    create: impish_delights_create,
    ..QuestClass::BASE
};

fn three_wishes_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1073660);
    q.description = td(1073699);
    q.refusal_message = td(1073733);
    q.in_progress_message = td(1073740);
    q.objectives
        .push(Objective::kill(8, &["Efreet"], tds("efreets"), None));
    q.rewards.push(rewards::large_bag_of_treasure());
    q
}
pub static THREE_WISHES: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.ThreeWishes",
    create: three_wishes_create,
    ..QuestClass::BASE
};

fn evil_eye_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1073084);
    q.description = td(1073574);
    q.refusal_message = td(1073580);
    q.in_progress_message = td(1073594);
    q.objectives
        .push(Objective::kill(12, &["Gazer"], tds("gazers"), None));
    q.rewards.push(rewards::bag_of_treasure());
    q
}
pub static EVIL_EYE: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.EvilEye",
    create: evil_eye_create,
    ..QuestClass::BASE
};

fn gargoyles_wrath_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1073658);
    q.description = td(1073697);
    q.refusal_message = td(1073733);
    q.in_progress_message = td(1073738);
    q.objectives.push(Objective::kill(
        6,
        &["GargoyleEnforcer"],
        tds("gargoyle enforcers"),
        None,
    ));
    q.rewards.push(rewards::large_bag_of_treasure());
    q
}
pub static GARGOYLES_WRATH: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.GargoylesWrath",
    create: gargoyles_wrath_create,
    ..QuestClass::BASE
};

fn undead_mages_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1073080);
    q.description = td(1073570);
    q.refusal_message = td(1073580);
    q.in_progress_message = td(1073590);
    q.objectives.push(Objective::kill(
        10,
        &["BoneMagi", "SkeletalMage"],
        tds("bone mages or skeletal mages"),
        None,
    ));
    q.rewards.push(rewards::bag_of_treasure());
    q
}
pub static UNDEAD_MAGES: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.UndeadMages",
    create: undead_mages_create,
    ..QuestClass::BASE
};

fn friendly_neighborhood_spiderkiller_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1073662);
    q.description = td(1073701);
    q.refusal_message = td(1073733);
    q.in_progress_message = td(1073742);
    q.objectives.push(Objective::kill(
        8,
        &["DreadSpider"],
        tds("dread spiders"),
        None,
    ));
    q.rewards.push(rewards::large_bag_of_treasure());
    q
}
pub static FRIENDLY_NEIGHBORHOOD_SPIDERKILLER: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.FriendlyNeighborhoodSpiderkiller",
    create: friendly_neighborhood_spiderkiller_create,
    ..QuestClass::BASE
};

fn mongbat_menace_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1073003);
    q.description = td(1073033);
    q.refusal_message = td(1072270);
    q.in_progress_message = td(1072271);
    q.objectives
        .push(Objective::kill(10, &["Mongbat"], tds("mongbats"), None));
    q.objectives.push(Objective::kill(
        4,
        &["GreaterMongbat"],
        tds("greater mongbats"),
        None,
    ));
    q.rewards.push(rewards::bag_of_trinkets());
    q
}
pub static MONGBAT_MENACE: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.MongbatMenace",
    create: mongbat_menace_create,
    ..QuestClass::BASE
};

fn stirring_the_nest_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1073087);
    q.description = td(1073577);
    q.refusal_message = td(1073580);
    q.in_progress_message = td(1073597);
    q.objective_type = ObjectiveType::Any;
    q.objectives.push(Objective::kill(
        3,
        &["RedSolenQueen"],
        tds("red solen queens"),
        None,
    ));
    q.objectives.push(Objective::kill(
        3,
        &["BlackSolenQueen"],
        tds("black solen queens"),
        None,
    ));
    q.rewards.push(rewards::bag_of_treasure());
    q
}
pub static STIRRING_THE_NEST: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.StirringTheNest",
    create: stirring_the_nest_create,
    ..QuestClass::BASE
};

fn warrior_caste_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1073078);
    q.description = td(1073568);
    q.refusal_message = td(1073580);
    q.in_progress_message = td(1073588);
    q.objectives.push(Objective::kill(
        10,
        &["TerathanWarrior"],
        tds("terathan warriors"),
        None,
    ));
    q.rewards.push(rewards::bag_of_treasure());
    q
}
pub static WARRIOR_CASTE: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.WarriorCaste",
    create: warrior_caste_create,
    ..QuestClass::BASE
};

fn big_worms_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1073088);
    q.description = td(1073578);
    q.refusal_message = td(1073580);
    q.in_progress_message = td(1073598);
    q.objectives.push(Objective::kill(
        10,
        &["IceSerpent"],
        tds("giant ice serpents"),
        None,
    ));
    q.rewards.push(rewards::bag_of_treasure());
    q
}
pub static BIG_WORMS: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.BigWorms",
    create: big_worms_create,
    ..QuestClass::BASE
};

fn orcish_elite_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1073081);
    q.description = td(1073571);
    q.refusal_message = td(1073580);
    q.in_progress_message = td(1073591);
    q.objectives
        .push(Objective::kill(6, &["OrcBomber"], tds("orc bombers"), None));
    q.objectives.push(Objective::kill(
        4,
        &["OrcCaptain"],
        tds("orc captain"),
        None,
    ));
    q.rewards.push(rewards::bag_of_treasure());
    q
}
pub static ORCISH_ELITE: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.OrcishElite",
    create: orcish_elite_create,
    ..QuestClass::BASE
};

fn thinning_the_herd_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1072249);
    q.description = td(1072263);
    q.refusal_message = td(1072270);
    q.in_progress_message = td(1072271);
    q.objectives
        .push(Objective::kill(10, &["Goat"], tds("goats"), None));
    q.rewards.push(rewards::small_bag_of_trinkets());
    q
}
pub static THINNING_THE_HERD: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.ThinningTheHerd",
    create: thinning_the_herd_create,
    ..QuestClass::BASE
};

fn squishy_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1072998);
    q.description = td(1073031);
    q.refusal_message = td(1072270);
    q.in_progress_message = td(1072271);
    q.objectives
        .push(Objective::kill(12, &["Slime"], tds("slimes"), None));
    q.rewards.push(rewards::bag_of_trinkets());
    q
}
pub static SQUISHY: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.Squishy",
    create: squishy_create,
    ..QuestClass::BASE
};

fn orc_slaying_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1072986);
    q.description = td(1073015);
    q.refusal_message = td(1072270);
    q.in_progress_message = td(1072271);
    q.objectives
        .push(Objective::kill(8, &["Orc"], tds("orcs"), None));
    q.objectives.push(Objective::kill(
        4,
        &["OrcishLord"],
        tds("orcish lords"),
        None,
    ));
    q.rewards.push(rewards::bag_of_trinkets());
    q
}
pub static ORC_SLAYING: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.OrcSlaying",
    create: orc_slaying_create,
    ..QuestClass::BASE
};

fn a_big_job_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1072988);
    q.description = td(1073017);
    q.refusal_message = td(1072270);
    q.in_progress_message = td(1072271);
    q.objectives
        .push(Objective::kill(5, &["Ogre"], tds("ogres"), None));
    q.objectives
        .push(Objective::kill(5, &["Ettin"], tds("ettins"), None));
    q.rewards.push(rewards::bag_of_trinkets());
    q
}
pub static A_BIG_JOB: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.ABigJob",
    create: a_big_job_create,
    ..QuestClass::BASE
};

fn trolling_for_trolls_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1072985);
    q.description = td(1073014);
    q.refusal_message = td(1072270);
    q.in_progress_message = td(1072271);
    q.objectives
        .push(Objective::kill(10, &["Troll"], tds("trolls"), None));
    q.rewards.push(rewards::bag_of_trinkets());
    q
}
pub static TROLLING_FOR_TROLLS: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.TrollingForTrolls",
    create: trolling_for_trolls_create,
    ..QuestClass::BASE
};

fn cold_hearted_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1072991);
    q.description = td(1073027);
    q.refusal_message = td(1072270);
    q.in_progress_message = td(1072271);
    q.objectives.push(Objective::kill(
        6,
        &["IceSerpent"],
        tds("giant ice serpents"),
        None,
    ));
    q.objectives.push(Objective::kill(
        6,
        &["FrostSpider"],
        tds("frost spiders"),
        None,
    ));
    q.rewards.push(rewards::bag_of_trinkets());
    q
}
pub static COLD_HEARTED: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.ColdHearted",
    create: cold_hearted_create,
    ..QuestClass::BASE
};

fn forked_tongues_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1072984);
    q.description = td(1073013);
    q.refusal_message = td(1072270);
    q.in_progress_message = td(1072271);
    q.objectives
        .push(Objective::kill(10, &["Lizardman"], tds("lizardmen"), None));
    q.rewards.push(rewards::bag_of_trinkets());
    q
}
pub static FORKED_TONGUES: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.ForkedTongues",
    create: forked_tongues_create,
    ..QuestClass::BASE
};

fn shaking_things_up_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1073083);
    q.description = td(1073573);
    q.refusal_message = td(1073580);
    q.in_progress_message = td(1073593);
    q.objective_type = ObjectiveType::Any;
    q.objectives.push(Objective::kill(
        10,
        &["RedSolenWarrior"],
        tds("red solen warriors"),
        None,
    ));
    q.objectives.push(Objective::kill(
        10,
        &["BlackSolenWarrior"],
        tds("black solen warriors"),
        None,
    ));
    q.rewards.push(rewards::bag_of_treasure());
    q
}
pub static SHAKING_THINGS_UP: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.ShakingThingsUp",
    create: shaking_things_up_create,
    ..QuestClass::BASE
};

fn arachnophobia_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1073079);
    q.description = td(1073569);
    q.refusal_message = td(1073580);
    q.in_progress_message = td(1073589);
    q.objectives.push(Objective::kill(
        12,
        &["GiantBlackWidow"],
        tds("giant black widows"),
        None,
    ));
    q.rewards.push(rewards::bag_of_treasure());
    q
}
pub static ARACHNOPHOBIA: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.Arachnophobia",
    create: arachnophobia_create,
    ..QuestClass::BASE
};

fn mini_swamp_thing_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1073072);
    q.description = td(1073562);
    q.refusal_message = td(1073580);
    q.in_progress_message = td(1073582);
    q.objectives
        .push(Objective::kill(12, &["Bogling"], tds("boglings"), None));
    q.rewards.push(rewards::bag_of_treasure());
    q
}
pub static MINI_SWAMP_THING: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.MiniSwampThing",
    create: mini_swamp_thing_create,
    ..QuestClass::BASE
};

fn the_perils_of_farming_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1073664);
    q.description = td(1073703);
    q.refusal_message = td(1073733);
    q.in_progress_message = td(1073744);
    q.objectives.push(Objective::kill(
        15,
        &["WhippingVine"],
        tds("whipping vines"),
        None,
    ));
    q.rewards.push(rewards::large_bag_of_treasure());
    q
}
pub static THE_PERILS_OF_FARMING: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.ThePerilsOfFarming",
    create: the_perils_of_farming_create,
    ..QuestClass::BASE
};

fn industrious_as_an_ant_lion_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1073665);
    q.description = td(1073704);
    q.refusal_message = td(1073733);
    q.in_progress_message = td(1073745);
    q.objectives
        .push(Objective::kill(12, &["AntLion"], tds("ant lions"), None));
    q.rewards.push(rewards::large_bag_of_treasure());
    q
}
pub static INDUSTRIOUS_AS_AN_ANT_LION: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.IndustriousAsAnAntLion",
    create: industrious_as_an_ant_lion_create,
    ..QuestClass::BASE
};

fn unholy_construct_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1073666);
    q.description = td(1073705);
    q.refusal_message = td(1073733);
    q.in_progress_message = td(1073746);
    q.completion_message = td(1073787);
    q.objectives
        .push(Objective::kill(10, &["Golem"], tds("golems"), None));
    q.rewards.push(rewards::large_bag_of_treasure());
    q
}
pub static UNHOLY_CONSTRUCT: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.UnholyConstruct",
    create: unholy_construct_create,
    ..QuestClass::BASE
};

fn a_chill_in_the_air_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1073663);
    q.description = td(1073702);
    q.refusal_message = td(1073733);
    q.in_progress_message = td(1073746);
    q.objectives.push(Objective::kill(
        15,
        &["IceElemental"],
        tds("ice elementals"),
        None,
    ));
    q.rewards.push(rewards::large_bag_of_treasure());
    q
}
pub static A_CHILL_IN_THE_AIR: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.AChillInTheAir",
    create: a_chill_in_the_air_create,
    ..QuestClass::BASE
};

fn the_king_of_clothing_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.has_restart_delay = true;
    q.title = td(1073902);
    q.description = td(1074092);
    q.refusal_message = td(1073921);
    q.in_progress_message = td(1073948);
    q.completion_message = td(1073974);
    q.objectives
        .push(Objective::collect(10, "Kilt", td(1025431)));
    q.rewards.push(rewards::tailor_satchel());
    q
}
pub static THE_KING_OF_CLOTHING: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.TheKingOfClothing",
    create: the_king_of_clothing_create,
    ..QuestClass::BASE
};

fn the_puffy_shirt_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.has_restart_delay = true;
    q.title = td(1073903);
    q.description = td(1074093);
    q.refusal_message = td(1073921);
    q.in_progress_message = td(1073949);
    q.completion_message = td(1073973);
    q.objectives
        .push(Objective::collect(10, "FancyShirt", td(1027933)));
    q.rewards.push(rewards::tailor_satchel());
    q
}
pub static THE_PUFFY_SHIRT: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.ThePuffyShirt",
    create: the_puffy_shirt_create,
    ..QuestClass::BASE
};

fn from_the_gaultier_collection_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.has_restart_delay = true;
    q.title = td(1073905);
    q.description = td(1074095);
    q.refusal_message = td(1073921);
    q.in_progress_message = td(1073951);
    q.completion_message = td(1073976);
    q.objectives
        .push(Objective::collect(10, "StuddedBustierArms", td(1027180)));
    q.rewards.push(rewards::tailor_satchel());
    q
}
pub static FROM_THE_GAULTIER_COLLECTION: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.FromTheGaultierCollection",
    create: from_the_gaultier_collection_create,
    ..QuestClass::BASE
};

fn haute_couture_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.has_restart_delay = true;
    q.title = td(1073901);
    q.description = td(1074091);
    q.refusal_message = td(1073921);
    q.in_progress_message = td(1073947);
    q.completion_message = td(1073973);
    q.objectives
        .push(Objective::collect(10, "FlowerGarland", td(1028965)));
    q.rewards.push(rewards::tailor_satchel());
    q
}
pub static HAUTE_COUTURE: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.HauteCouture",
    create: haute_couture_create,
    ..QuestClass::BASE
};

fn the_song_of_the_wind_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.has_restart_delay = true;
    q.title = td(1073910);
    q.description = td(1074100);
    q.refusal_message = td(1073921);
    q.in_progress_message = td(1073956);
    q.completion_message = td(1073980);
    q.objectives.push(Objective::collect(
        10,
        "FancyWindChimes",
        tds("fancy wind chimes"),
    ));
    q.rewards.push(rewards::tinker_satchel());
    q
}
pub static THE_SONG_OF_THE_WIND: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.TheSongOfTheWind",
    create: the_song_of_the_wind_create,
    ..QuestClass::BASE
};

fn beer_goggles_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.has_restart_delay = true;
    q.title = td(1073895);
    q.description = td(1074085);
    q.refusal_message = td(1073921);
    q.in_progress_message = td(1073941);
    q.completion_message = td(1073971);
    q.objectives
        .push(Objective::collect(25, "BarrelTap", td(1024100)));
    q.rewards.push(rewards::tinker_satchel());
    q
}
pub static BEER_GOGGLES: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.BeerGoggles",
    create: beer_goggles_create,
    ..QuestClass::BASE
};

fn message_in_a_bottle_quest_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.has_restart_delay = true;
    q.title = td(1073894);
    q.description = td(1074084);
    q.refusal_message = td(1073921);
    q.in_progress_message = td(1073940);
    q.completion_message = td(1073971);
    q.objectives
        .push(Objective::collect(50, "Bottle", td(1023854)));
    q.rewards.push(rewards::tinker_satchel());
    q
}
pub static MESSAGE_IN_A_BOTTLE_QUEST: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.MessageInABottleQuest",
    create: message_in_a_bottle_quest_create,
    ..QuestClass::BASE
};

fn necessitys_mother_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.has_restart_delay = true;
    q.title = td(1073906);
    q.description = td(1074096);
    q.refusal_message = td(1073921);
    q.in_progress_message = td(1073952);
    q.completion_message = td(1073977);
    q.objectives
        .push(Objective::collect(10, "TinkerTools", td(1027868)));
    q.rewards.push(rewards::tinker_satchel());
    q
}
pub static NECESSITYS_MOTHER: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.NecessitysMother",
    create: necessitys_mother_create,
    ..QuestClass::BASE
};

fn tick_tock_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.has_restart_delay = true;
    q.title = td(1073907);
    q.description = td(1074097);
    q.refusal_message = td(1073921);
    q.in_progress_message = td(1073953);
    q.completion_message = td(1073978);
    q.objectives
        .push(Objective::collect(10, "Clock", td(1024171)));
    q.rewards.push(rewards::tinker_satchel());
    q
}
pub static TICK_TOCK: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.TickTock",
    create: tick_tock_create,
    ..QuestClass::BASE
};

fn reptilian_dentist_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1074280);
    q.description = td(1074710);
    q.refusal_message = td(1074723);
    q.in_progress_message = td(1074722);
    q.completion_message = td(1074721);
    q.objectives
        .push(Objective::collect(1, "CoilsFang", tds("coil's fang")));
    q.rewards.push(rewards::tinker_satchel());
    q
}
pub static REPTILIAN_DENTIST: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.ReptilianDentist",
    create: reptilian_dentist_create,
    ..QuestClass::BASE
};

fn stop_harping_on_me_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.has_restart_delay = true;
    q.title = td(1073881);
    q.description = td(1074071);
    q.refusal_message = td(1073921);
    q.in_progress_message = td(1073927);
    q.completion_message = td(1073969);
    q.objectives
        .push(Objective::collect(20, "LapHarp", td(1023762)));
    q.rewards.push(rewards::carpentry_satchel());
    q
}
pub static STOP_HARPING_ON_ME: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.StopHarpingOnMe",
    create: stop_harping_on_me_create,
    ..QuestClass::BASE
};

fn the_far_eye_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.has_restart_delay = true;
    q.title = td(1073908);
    q.description = td(1074098);
    q.refusal_message = td(1073921);
    q.in_progress_message = td(1073954);
    q.completion_message = td(1073978);
    q.objectives
        .push(Objective::collect(20, "Spyglass", td(1025365)));
    q.rewards.push(rewards::tinker_satchel());
    q
}
pub static THE_FAR_EYE: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.TheFarEye",
    create: the_far_eye_create,
    ..QuestClass::BASE
};

fn lethal_darts_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.has_restart_delay = true;
    q.title = td(1073876);
    q.description = td(1074066);
    q.refusal_message = td(1073921);
    q.in_progress_message = td(1073922);
    q.completion_message = td(1073968);
    q.completion_notice = completion_notice_craft();
    q.objectives
        .push(Objective::collect(10, "Bolt", td(1027163)));
    q.rewards.push(rewards::fletching_satchel());
    q
}
pub static LETHAL_DARTS: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.LethalDarts",
    create: lethal_darts_create,
    ..QuestClass::BASE
};

fn a_simple_bow_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.has_restart_delay = true;
    q.title = td(1073877);
    q.description = td(1074067);
    q.refusal_message = td(1073921);
    q.in_progress_message = td(1073923);
    q.completion_message = td(1073968);
    q.completion_notice = completion_notice_craft();
    q.objectives
        .push(Objective::collect(10, "Bow", td(1025041)));
    q.rewards.push(rewards::fletching_satchel());
    q
}
pub static A_SIMPLE_BOW: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.ASimpleBow",
    create: a_simple_bow_create,
    ..QuestClass::BASE
};

fn ingenious_archery_part_one_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.has_restart_delay = true;
    q.title = td(1073878);
    q.description = td(1074068);
    q.refusal_message = td(1073921);
    q.in_progress_message = td(1073924);
    q.completion_message = td(1073968);
    q.completion_notice = completion_notice_craft();
    q.objectives
        .push(Objective::collect(10, "Crossbow", td(1023919)));
    q.rewards.push(rewards::fletching_satchel());
    q
}
pub static INGENIOUS_ARCHERY_PART_ONE: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.IngeniousArcheryPartOne",
    create: ingenious_archery_part_one_create,
    ..QuestClass::BASE
};

fn ingenious_archery_part_two_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.has_restart_delay = true;
    q.title = td(1073879);
    q.description = td(1074069);
    q.refusal_message = td(1073921);
    q.in_progress_message = td(1073925);
    q.completion_message = td(1073968);
    q.completion_notice = completion_notice_craft();
    q.objectives
        .push(Objective::collect(8, "HeavyCrossbow", td(1025116)));
    q.rewards.push(rewards::fletching_satchel());
    q
}
pub static INGENIOUS_ARCHERY_PART_TWO: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.IngeniousArcheryPartTwo",
    create: ingenious_archery_part_two_create,
    ..QuestClass::BASE
};

fn ingenious_archery_part_three_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.has_restart_delay = true;
    q.title = td(1073880);
    q.description = td(1074070);
    q.refusal_message = td(1073921);
    q.in_progress_message = td(1073926);
    q.completion_message = td(1073968);
    q.completion_notice = completion_notice_craft();
    q.objectives
        .push(Objective::collect(10, "RepeatingCrossbow", td(1029923)));
    q.rewards.push(rewards::fletching_satchel());
    q
}
pub static INGENIOUS_ARCHERY_PART_THREE: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.IngeniousArcheryPartThree",
    create: ingenious_archery_part_three_create,
    ..QuestClass::BASE
};

fn scale_armor_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1074711);
    q.description = td(1074712);
    q.refusal_message = td(1073580);
    q.in_progress_message = td(1074724);
    q.completion_message = td(1074725);
    q.objectives.push(Objective::collect(
        1,
        "ThrashersTail",
        tds("Thrasher's Tail"),
    ));
    q.objectives
        .push(Objective::collect(10, "HydraScale", tds("Hydra Scales")));
    q.rewards.push(rewards::bag_of_treasure());
    q
}
pub static SCALE_ARMOR: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.ScaleArmor",
    create: scale_armor_create,
    ..QuestClass::BASE
};

fn cuts_both_ways_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.has_restart_delay = true;
    q.title = td(1073913);
    q.description = td(1074103);
    q.refusal_message = td(1073921);
    q.in_progress_message = td(1073959);
    q.completion_message = td(1073978);
    q.objectives
        .push(Objective::collect(12, "Broadsword", td(1023934)));
    q.rewards.push(rewards::blacksmith_satchel());
    q
}
pub static CUTS_BOTH_WAYS: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.CutsBothWays",
    create: cuts_both_ways_create,
    ..QuestClass::BASE
};

fn dragon_protection_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.has_restart_delay = true;
    q.title = td(1073915);
    q.description = td(1074105);
    q.refusal_message = td(1073921);
    q.in_progress_message = td(1073961);
    q.completion_message = td(1073978);
    q.objectives
        .push(Objective::collect(10, "DragonHelm", td(1029797)));
    q.rewards.push(rewards::blacksmith_satchel());
    q
}
pub static DRAGON_PROTECTION: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.DragonProtection",
    create: dragon_protection_create,
    ..QuestClass::BASE
};

fn nothing_fancy_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.has_restart_delay = true;
    q.title = td(1073911);
    q.description = td(1074101);
    q.refusal_message = td(1073921);
    q.in_progress_message = td(1073957);
    q.completion_message = td(1073978);
    q.objectives
        .push(Objective::collect(15, "Bascinet", td(1025132)));
    q.rewards.push(rewards::blacksmith_satchel());
    q
}
pub static NOTHING_FANCY: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.NothingFancy",
    create: nothing_fancy_create,
    ..QuestClass::BASE
};

fn the_bulwark_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.has_restart_delay = true;
    q.title = td(1073912);
    q.description = td(1074102);
    q.refusal_message = td(1073921);
    q.in_progress_message = td(1073958);
    q.completion_message = td(1073978);
    q.objectives
        .push(Objective::collect(10, "HeaterShield", td(1027030)));
    q.rewards.push(rewards::blacksmith_satchel());
    q
}
pub static THE_BULWARK: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.TheBulwark",
    create: the_bulwark_create,
    ..QuestClass::BASE
};

fn arch_support_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.has_restart_delay = true;
    q.title = td(1073882);
    q.description = td(1074072);
    q.refusal_message = td(1073921);
    q.in_progress_message = td(1073928);
    q.completion_message = td(1073969);
    q.objectives
        .push(Objective::collect(10, "FootStool", td(1022910)));
    q.rewards.push(rewards::carpentry_satchel());
    q
}
pub static ARCH_SUPPORT: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.ArchSupport",
    create: arch_support_create,
    ..QuestClass::BASE
};

fn paroxysmus_succubi_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1073067);
    q.description = td(1074696);
    q.refusal_message = td(1072270);
    q.in_progress_message = td(1072271);
    q.objectives.push(Objective::kill(
        3,
        &["Succubus"],
        tds("succubi"),
        Some(QuestArea::new(td(1074806), "The Palace of Paroxysmus")),
    ));
    q.rewards.push(rewards::large_bag_of_treasure());
    q
}
pub static PAROXYSMUS_SUCCUBI: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.ParoxysmusSuccubi",
    create: paroxysmus_succubi_create,
    ..QuestClass::BASE
};

fn paroxysmus_moloch_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1073068);
    q.description = td(1074695);
    q.refusal_message = td(1072270);
    q.in_progress_message = td(1072271);
    q.objectives.push(Objective::kill(
        3,
        &["Moloch"],
        tds("molochs"),
        Some(QuestArea::new(td(1074806), "The Palace of Paroxysmus")),
    ));
    q.rewards.push(rewards::large_bag_of_treasure());
    q
}
pub static PAROXYSMUS_MOLOCH: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.ParoxysmusMoloch",
    create: paroxysmus_moloch_create,
    ..QuestClass::BASE
};

fn paroxysmus_daemons_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1073069);
    q.description = td(1074694);
    q.refusal_message = td(1072270);
    q.in_progress_message = td(1072271);
    q.objectives.push(Objective::kill(
        10,
        &["Daemon"],
        tds("daemons"),
        Some(QuestArea::new(td(1074806), "The Palace of Paroxysmus")),
    ));
    q.rewards.push(rewards::large_bag_of_treasure());
    q
}
pub static PAROXYSMUS_DAEMONS: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.ParoxysmusDaemons",
    create: paroxysmus_daemons_create,
    ..QuestClass::BASE
};

fn paroxysmus_arcane_daemons_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1073070);
    q.description = td(1074697);
    q.refusal_message = td(1072270);
    q.in_progress_message = td(1072271);
    q.objectives.push(Objective::kill(
        10,
        &["ArcaneDaemon"],
        tds("arcane daemons"),
        Some(QuestArea::new(td(1074806), "The Palace of Paroxysmus")),
    ));
    q.rewards.push(rewards::large_bag_of_treasure());
    q
}
pub static PAROXYSMUS_ARCANE_DAEMONS: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.ParoxysmusArcaneDaemons",
    create: paroxysmus_arcane_daemons_create,
    ..QuestClass::BASE
};

fn caustic_combo_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1073062);
    q.description = td(1074693);
    q.refusal_message = td(1072270);
    q.in_progress_message = td(1072271);
    q.objectives.push(Objective::kill(
        3,
        &["PoisonElemental"],
        tds("poison elementals"),
        Some(QuestArea::new(td(1074806), "The Palace of Paroxysmus")),
    ));
    q.objectives.push(Objective::kill(
        6,
        &["AcidElemental"],
        tds("acid elementals"),
        Some(QuestArea::new(td(1074806), "The Palace of Paroxysmus")),
    ));
    q.rewards.push(rewards::large_bag_of_treasure());
    q
}
pub static CAUSTIC_COMBO: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.CausticCombo",
    create: caustic_combo_create,
    ..QuestClass::BASE
};

fn plague_lord_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1073061);
    q.description = td(1074692);
    q.refusal_message = td(1072270);
    q.in_progress_message = td(1072271);
    q.objectives.push(Objective::kill(
        10,
        &["PlagueSpawn"],
        tds("plague spawns"),
        Some(QuestArea::new(td(1074806), "The Palace of Paroxysmus")),
    ));
    q.objectives.push(Objective::kill(
        3,
        &["PlagueBeast"],
        tds("plague beasts"),
        Some(QuestArea::new(td(1074806), "The Palace of Paroxysmus")),
    ));
    q.objectives.push(Objective::kill(
        1,
        &["PlagueBeastLord"],
        tds("plague beast lord"),
        Some(QuestArea::new(td(1074806), "The Palace of Paroxysmus")),
    ));
    q.rewards.push(rewards::large_bag_of_treasure());
    q
}
pub static PLAGUE_LORD: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.PlagueLord",
    create: plague_lord_create,
    ..QuestClass::BASE
};

fn glassy_foe_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1073055);
    q.description = td(1074669);
    q.refusal_message = td(1074671);
    q.in_progress_message = td(1074672);
    q.completion_message = td(1074673);
    q.objectives.push(Objective::kill(
        5,
        &["CrystalLatticeSeeker"],
        tds("crystal lattice seekers"),
        Some(QuestArea::new(td(1074805), "The Prism of Light")),
    ));
    q.rewards.push(rewards::large_bag_of_treasure());
    q
}
pub static GLASSY_FOE: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.GlassyFoe",
    create: glassy_foe_create,
    ..QuestClass::BASE
};

fn daemonic_prism_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1073053);
    q.description = td(1074668);
    q.refusal_message = td(1074671);
    q.in_progress_message = td(1074672);
    q.completion_message = td(1074673);
    q.objectives.push(Objective::kill(
        3,
        &["CrystalDaemon"],
        tds("crystal daemons"),
        Some(QuestArea::new(td(1074805), "The Prism of Light")),
    ));
    q.rewards.push(rewards::large_bag_of_treasure());
    q
}
pub static DAEMONIC_PRISM: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.DaemonicPrism",
    create: daemonic_prism_create,
    ..QuestClass::BASE
};

fn hailstorm_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1073057);
    q.description = td(1074670);
    q.refusal_message = td(1074671);
    q.in_progress_message = td(1074672);
    q.completion_message = td(1074673);
    q.objectives.push(Objective::kill(
        8,
        &["CrystalVortex"],
        tds("crystal vortices"),
        Some(QuestArea::new(td(1074805), "The Prism of Light")),
    ));
    q.rewards.push(rewards::large_bag_of_treasure());
    q
}
pub static HAILSTORM: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.Hailstorm",
    create: hailstorm_create,
    ..QuestClass::BASE
};

fn death_to_the_ninja_create(class: &'static QuestClass) -> MlQuest {
    let mut q = MlQuest::new(class);
    q.activated = true;
    q.title = td(1072913);
    q.description = td(1072966);
    q.refusal_message = td(1072979);
    q.in_progress_message = td(1072980);
    q.objectives.push(Objective::kill(
        10,
        &["EliteNinja"],
        tds("elite ninjas"),
        Some(QuestArea::new(td(1074804), "The Citadel")),
    ));
    q.rewards.push(rewards::bag_of_treasure());
    q
}
pub static DEATH_TO_THE_NINJA: QuestClass = QuestClass {
    full_name: "Server.Engines.MLQuests.Definitions.DeathToTheNinja",
    create: death_to_the_ninja_create,
    ..QuestClass::BASE
};
