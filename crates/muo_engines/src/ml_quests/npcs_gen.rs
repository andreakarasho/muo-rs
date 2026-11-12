//! The ML quest givers on a `BaseCreature` base: one constructor per class.

use super::npc_support::*;
use super::quester::{self, Quester};
use muo_content::creature::{BASE_CREATURE, BASE_CREATURE_C};
use muo_content::hooks::rnd;
use muo_primitives::{Body, utility};
use muo_server::ids::MobileId;
use muo_server::type_registry::TypeRegistry;
use muo_server::world::World;

fn ben_new(w: &mut World) -> MobileId {
    let id = new_creature(w, &BEN, AIType::AiVendor, FightMode::None, 2, 1);
    set_title(w, id, Some("the Apprentice Necromancer".to_string()));
    set_body(w, id, Body(0x190));
    set_hue(w, id, 0x83FD);
    set_hair_item_id(w, id, 0x2048);
    set_hair_hue(w, id, 0x463);
    set_facial_hair_item_id(w, id, 0x204C);
    set_facial_hair_hue(w, id, 0x463);
    set_speed(w, id, 0.5, 2.0, true);
    init_stats(w, id, 100, 100, 25);
    add_item_hue(w, id, "Backpack", None);
    add_item_hue(w, id, "Shoes", Some(0x901));
    add_item_hue(w, id, "LongPants", Some(0x1BB));
    add_item_hue(w, id, "FancyShirt", Some(0x756));
    id
}
creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Ben`.
    BEN, BEN_C = "Server.Engines.MLQuests.Definitions.Ben" : BASE_CREATURE / BASE_CREATURE_C, ctor ben_new, encoded false;
    c {
        is_invulnerable: |_w, _id| true,
        default_name: |_w, _id| Some("Ben"),
    }
    m {
    }
}

fn frederic_new(w: &mut World) -> MobileId {
    let id = new_creature(w, &FREDERIC, AIType::AiVendor, FightMode::None, 2, 1);
    set_body(w, id, Body(0x1A));
    set_hue(w, id, 0x455);
    set_frozen(w, id, true);
    set_speed(w, id, 0.5, 2.0, true);
    init_stats(w, id, 100, 100, 25);
    id
}
creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Frederic`.
    FREDERIC, FREDERIC_C = "Server.Engines.MLQuests.Definitions.Frederic" : BASE_CREATURE / BASE_CREATURE_C, ctor frederic_new, encoded false;
    c {
        is_invulnerable: |_w, _id| true,
        default_name: |_w, _id| Some("The Ghost of Frederic Smithson"),
    }
    m {
    }
}

fn leon_new(w: &mut World) -> MobileId {
    let id = new_creature(w, &LEON, AIType::AiVendor, FightMode::None, 2, 1);
    set_title(w, id, Some("the Alchemist".to_string()));
    set_race_named(w, id, "Human");
    set_body(w, id, Body(0x190));
    set_female(w, id, false);
    let v1 = race_skin_hue(w, id);
    set_hue(w, id, v1);
    set_speed(w, id, 0.5, 2.0, true);
    init_stats(w, id, 100, 100, 25);
    assign_random_hair(w, id);
    add_item_hue(w, id, "Backpack", None);
    add_item_hue(w, id, "Shoes", Some(0x901));
    add_item_hue(w, id, "Robe", Some(0x657));
    id
}
creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Leon`.
    LEON, LEON_C = "Server.Engines.MLQuests.Definitions.Leon" : BASE_CREATURE / BASE_CREATURE_C, ctor leon_new, encoded false;
    c {
        is_invulnerable: |_w, _id| true,
        default_name: |_w, _id| Some("Leon"),
    }
    m {
    }
}

fn andros_new(w: &mut World) -> MobileId {
    let id = new_creature(w, &ANDROS, AIType::AiVendor, FightMode::None, 2, 1);
    set_title(w, id, Some("the Blacksmith".to_string()));
    set_body(w, id, Body(0x190));
    set_hue(w, id, 0x8409);
    set_facial_hair_item_id(w, id, 0x2041);
    set_facial_hair_hue(w, id, 0x45E);
    set_hair_item_id(w, id, 0x2049);
    set_hair_hue(w, id, 0x45E);
    set_speed(w, id, 0.5, 2.0, true);
    init_stats(w, id, 100, 100, 25);
    add_item_hue(w, id, "Backpack", None);
    add_item_hue(w, id, "Boots", Some(0x901));
    add_item_hue(w, id, "FancyShirt", Some(0x60B));
    add_item_hue(w, id, "LongPants", Some(0x1BB));
    add_item_hue(w, id, "FullApron", Some(0x901));
    add_item_hue(w, id, "SmithHammer", None);
    id
}
creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Andros`.
    ANDROS, ANDROS_C = "Server.Engines.MLQuests.Definitions.Andros" : BASE_CREATURE / BASE_CREATURE_C, ctor andros_new, encoded false;
    c {
        is_invulnerable: |_w, _id| true,
        default_name: |_w, _id| Some("Andros"),
    }
    m {
    }
}

fn kia_new(w: &mut World) -> MobileId {
    let id = new_creature(w, &KIA, AIType::AiVendor, FightMode::None, 2, 1);
    set_title(w, id, Some("the student".to_string()));
    set_race_named(w, id, "Human");
    set_body(w, id, Body(0x191));
    set_female(w, id, true);
    let v1 = race_skin_hue(w, id);
    set_hue(w, id, v1);
    set_speed(w, id, 0.5, 2.0, true);
    init_stats(w, id, 100, 100, 25);
    assign_random_hair(w, id);
    add_item_hue(w, id, "Backpack", None);
    add_item_hue(w, id, "Sandals", Some(0x709));
    add_item_hue(w, id, "Robe", Some(0x497));
    id
}
creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Kia`.
    KIA, KIA_C = "Server.Engines.MLQuests.Definitions.Kia" : BASE_CREATURE / BASE_CREATURE_C, ctor kia_new, encoded false;
    c {
        is_invulnerable: |_w, _id| true,
        default_name: |_w, _id| Some("Kia"),
    }
    m {
    }
}

fn emerillo_new(w: &mut World) -> MobileId {
    let id = new_creature(w, &EMERILLO, AIType::AiVendor, FightMode::None, 2, 1);
    set_title(w, id, Some("the cook".to_string()));
    set_race_named(w, id, "Human");
    set_body(w, id, Body(0x190));
    set_female(w, id, false);
    let v1 = race_skin_hue(w, id);
    set_hue(w, id, v1);
    set_speed(w, id, 0.5, 2.0, true);
    init_stats(w, id, 100, 100, 25);
    assign_random_hair(w, id);
    add_item_hue(w, id, "Backpack", None);
    let v2 = utility::random_neutral_hue(&mut w.rng);
    add_item_hue(w, id, "Sandals", Some(v2));
    let v3 = utility::random_pink_hue(&mut w.rng);
    add_item_hue(w, id, "ShortPants", Some(v3));
    add_item_hue(w, id, "Shirt", None);
    add_item_hue(w, id, "HalfApron", Some(0x8FD));
    id
}
creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Emerillo`.
    EMERILLO, EMERILLO_C = "Server.Engines.MLQuests.Definitions.Emerillo" : BASE_CREATURE / BASE_CREATURE_C, ctor emerillo_new, encoded false;
    c {
        is_invulnerable: |_w, _id| true,
        default_name: |_w, _id| Some("Emerillo"),
        can_shout: |_w, _id| true,
    }
    m {
    }
}
fn emerillo_shout(w: &mut World, id: MobileId, pm: MobileId) {
    quester::tell(w, Quester::Mobile(id), pm, 1074222);
}

fn nythalia_new(w: &mut World) -> MobileId {
    let id = new_creature(w, &NYTHALIA, AIType::AiVendor, FightMode::None, 2, 1);
    set_title(w, id, Some("the student".to_string()));
    set_race_named(w, id, "Human");
    set_body(w, id, Body(0x191));
    set_female(w, id, true);
    let v1 = race_skin_hue(w, id);
    set_hue(w, id, v1);
    set_speed(w, id, 0.5, 2.0, true);
    init_stats(w, id, 100, 100, 25);
    assign_random_hair(w, id);
    add_item_hue(w, id, "Backpack", None);
    let v2 = utility::random_neutral_hue(&mut w.rng);
    add_item_hue(w, id, "Shoes", Some(v2));
    let v3 = rnd::bool(w);
    let v4 = if v3 { 0x497 } else { 0x498 };
    add_item_hue(w, id, "Robe", Some(v4));
    id
}
creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Nythalia`.
    NYTHALIA, NYTHALIA_C = "Server.Engines.MLQuests.Definitions.Nythalia" : BASE_CREATURE / BASE_CREATURE_C, ctor nythalia_new, encoded false;
    c {
        is_invulnerable: |_w, _id| true,
        default_name: |_w, _id| Some("Mythalia"),
    }
    m {
    }
}

fn jamal_new(w: &mut World) -> MobileId {
    let id = new_creature(w, &JAMAL, AIType::AiVendor, FightMode::None, 2, 1);
    set_title(w, id, Some("the Fisherman".to_string()));
    set_body(w, id, Body(400));
    let v1 = race_skin_hue(w, id);
    set_hue(w, id, v1);
    init_stats(w, id, 100, 100, 25);
    assign_random_hair(w, id);
    let hh = hair_hue(w, id);
    assign_random_facial_hair_hue(w, id, hh);
    add_item_hue(w, id, "Shirt", Some(0x1BB));
    let v2 = utility::random_neutral_hue(&mut w.rng);
    add_item_hue(w, id, "ShortPants", Some(v2));
    let v3 = utility::random_animal_hue(&mut w.rng);
    add_item_hue(w, id, "ThighBoots", Some(v3));
    add_item_hue(w, id, "Backpack", None);
    id
}
creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Jamal`.
    JAMAL, JAMAL_C = "Server.Engines.MLQuests.Definitions.Jamal" : BASE_CREATURE / BASE_CREATURE_C, ctor jamal_new, encoded false;
    c {
        is_invulnerable: |_w, _id| true,
        default_name: |_w, _id| Some("Jamal"),
    }
    m {
    }
}

fn iosep_new(w: &mut World) -> MobileId {
    let id = new_creature(w, &IOSEP, AIType::AiVendor, FightMode::None, 2, 1);
    set_title(w, id, Some("the Exporter".to_string()));
    set_body(w, id, Body(400));
    let v1 = race_skin_hue(w, id);
    set_hue(w, id, v1);
    init_stats(w, id, 100, 100, 25);
    let v2 = utility::random_blue_hue(&mut w.rng);
    add_item_hue(w, id, "FancyShirt", Some(v2));
    add_item_hue(w, id, "LongPants", Some(0x1BB));
    let v3 = utility::random_neutral_hue(&mut w.rng);
    add_item_hue(w, id, "Shoes", Some(v3));
    add_item_hue(w, id, "Backpack", None);
    id
}
creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Iosep`.
    IOSEP, IOSEP_C = "Server.Engines.MLQuests.Definitions.Iosep" : BASE_CREATURE / BASE_CREATURE_C, ctor iosep_new, encoded false;
    c {
        is_invulnerable: |_w, _id| true,
        default_name: |_w, _id| Some("Iosep"),
        can_shout: |_w, _id| true,
    }
    m {
    }
}
fn iosep_shout(w: &mut World, id: MobileId, pm: MobileId) {
    let v1 = rnd::bool(w);
    let v2 = if v1 { 1074209 } else { 1074215 };
    quester::tell(w, Quester::Mobile(id), pm, v2);
}

fn aurelia_new(w: &mut World) -> MobileId {
    let id = new_creature(w, &AURELIA, AIType::AiVendor, FightMode::None, 2, 1);
    set_title(w, id, Some("the Architect's Daughter".to_string()));
    set_race_named(w, id, "Human");
    set_body(w, id, Body(0x191));
    set_female(w, id, true);
    let v1 = race_skin_hue(w, id);
    set_hue(w, id, v1);
    set_speed(w, id, 0.5, 2.0, true);
    init_stats(w, id, 100, 100, 25);
    assign_random_hair(w, id);
    add_item_hue(w, id, "Backpack", None);
    let v2 = utility::random_pink_hue(&mut w.rng);
    add_item_hue(w, id, "Sandals", Some(v2));
    let v3 = rnd::bool(w);
    if v3 {
        let v54 = utility::random_pink_hue(&mut w.rng);
        add_item_hue(w, id, "Kilt", Some(v54));
    } else {
        let v54 = utility::random_pink_hue(&mut w.rng);
        add_item_hue(w, id, "Skirt", Some(v54));
    }
    let v4 = utility::random_red_hue(&mut w.rng);
    add_item_hue(w, id, "FancyShirt", Some(v4));
    id
}
creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Aurelia`.
    AURELIA, AURELIA_C = "Server.Engines.MLQuests.Definitions.Aurelia" : BASE_CREATURE / BASE_CREATURE_C, ctor aurelia_new, encoded false;
    c {
        is_invulnerable: |_w, _id| true,
        default_name: |_w, _id| Some("Aurelia"),
    }
    m {
    }
}

fn skeleton_of_szandor_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &SKELETON_OF_SZANDOR,
        AIType::AiVendor,
        FightMode::None,
        2,
        1,
    );
    set_title(w, id, Some("the Late Architect".to_string()));
    set_hue(w, id, 0x83F2);
    set_body(w, id, Body(0x32));
    set_speed(w, id, 0.5, 2.0, true);
    init_stats(w, id, 100, 100, 25);
    id
}
creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.SkeletonOfSzandor`.
    SKELETON_OF_SZANDOR, SKELETON_OF_SZANDOR_C = "Server.Engines.MLQuests.Definitions.SkeletonOfSzandor" : BASE_CREATURE / BASE_CREATURE_C, ctor skeleton_of_szandor_new, encoded false;
    c {
        is_invulnerable: |_w, _id| true,
        default_name: |_w, _id| Some("Skeleton of Szandor"),
    }
    m {
    }
}

fn saril_new(w: &mut World) -> MobileId {
    let id = new_creature(w, &SARIL, AIType::AiVendor, FightMode::None, 2, 1);
    set_title(w, id, Some("the guard".to_string()));
    set_race_named(w, id, "Elf");
    set_body(w, id, Body(0x25E));
    set_female(w, id, true);
    let v1 = race_skin_hue(w, id);
    set_hue(w, id, v1);
    set_speed(w, id, 0.5, 2.0, true);
    init_stats(w, id, 100, 100, 25);
    assign_random_hair(w, id);
    set_skill_range(w, id, SkillName::Meditation, 60.0, 80.0);
    set_skill_range(w, id, SkillName::Focus, 60.0, 80.0);
    add_item_hue(w, id, "ElvenBoots", None);
    add_item_hue(w, id, "WoodlandLegs", None);
    add_item_hue(w, id, "WoodlandArms", None);
    add_item_hue(w, id, "WoodlandBelt", None);
    add_item_hue(w, id, "WingedHelm", None);
    add_item_hue(w, id, "FemaleElvenPlateChest", None);
    add_item_hue(w, id, "RadiantScimitar", None);
    id
}
creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Saril`.
    SARIL, SARIL_C = "Server.Engines.MLQuests.Definitions.Saril" : BASE_CREATURE / BASE_CREATURE_C, ctor saril_new, encoded false;
    c {
        is_invulnerable: |_w, _id| true,
        default_name: |_w, _id| Some("Saril"),
        can_teach: |_w, _id| true,
        can_shout: |_w, _id| true,
    }
    m {
    }
}
fn saril_shout(w: &mut World, id: MobileId, pm: MobileId) {
    let v1 = rnd::bool(w);
    let v2 = if v1 { 1074186 } else { 1074183 };
    quester::tell(w, Quester::Mobile(id), pm, v2);
}

fn cailla_new(w: &mut World) -> MobileId {
    let id = new_creature(w, &CAILLA, AIType::AiVendor, FightMode::None, 2, 1);
    set_title(w, id, Some("the guard".to_string()));
    set_race_named(w, id, "Elf");
    set_body(w, id, Body(0x25E));
    set_female(w, id, true);
    let v1 = race_skin_hue(w, id);
    set_hue(w, id, v1);
    set_speed(w, id, 0.5, 2.0, true);
    init_stats(w, id, 100, 100, 25);
    assign_random_hair(w, id);
    set_skill_range(w, id, SkillName::Meditation, 60.0, 80.0);
    set_skill_range(w, id, SkillName::Focus, 60.0, 80.0);
    add_item_hue(w, id, "ElvenBoots", None);
    add_item_hue(w, id, "HidePants", None);
    add_item_hue(w, id, "HidePauldrons", None);
    add_item_hue(w, id, "HideGloves", None);
    add_item_hue(w, id, "WoodlandBelt", None);
    add_item_hue(w, id, "RavenHelm", None);
    add_item_hue(w, id, "HideFemaleChest", None);
    add_item_hue(w, id, "MagicalShortbow", None);
    id
}
creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Cailla`.
    CAILLA, CAILLA_C = "Server.Engines.MLQuests.Definitions.Cailla" : BASE_CREATURE / BASE_CREATURE_C, ctor cailla_new, encoded false;
    c {
        is_invulnerable: |_w, _id| true,
        default_name: |_w, _id| Some("Cailla"),
        can_teach: |_w, _id| true,
        can_shout: |_w, _id| true,
    }
    m {
    }
}
fn cailla_shout(w: &mut World, id: MobileId, pm: MobileId) {
    let v1 = rnd::bool(w);
    let v2 = if v1 { 1074187 } else { 1074210 };
    quester::tell(w, Quester::Mobile(id), pm, v2);
}

fn tamm_new(w: &mut World) -> MobileId {
    let id = new_creature(w, &TAMM, AIType::AiVendor, FightMode::None, 2, 1);
    set_title(w, id, Some("the guard".to_string()));
    set_race_named(w, id, "Elf");
    set_body(w, id, Body(0x25D));
    set_female(w, id, false);
    let v1 = race_skin_hue(w, id);
    set_hue(w, id, v1);
    set_speed(w, id, 0.5, 2.0, true);
    init_stats(w, id, 100, 100, 25);
    assign_random_hair(w, id);
    set_skill_range(w, id, SkillName::Meditation, 60.0, 80.0);
    set_skill_range(w, id, SkillName::Focus, 60.0, 80.0);
    add_item_hue(w, id, "ElvenBoots", None);
    add_item_hue(w, id, "HidePants", None);
    add_item_hue(w, id, "HidePauldrons", None);
    add_item_hue(w, id, "WingedHelm", None);
    add_item_hue(w, id, "HideChest", None);
    add_item_hue(w, id, "ElvenCompositeLongbow", None);
    id
}
creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Tamm`.
    TAMM, TAMM_C = "Server.Engines.MLQuests.Definitions.Tamm" : BASE_CREATURE / BASE_CREATURE_C, ctor tamm_new, encoded false;
    c {
        is_invulnerable: |_w, _id| true,
        default_name: |_w, _id| Some("Tamm"),
        can_teach: |_w, _id| true,
        can_shout: |_w, _id| true,
    }
    m {
    }
}
fn tamm_shout(w: &mut World, id: MobileId, pm: MobileId) {
    let v1 = rnd::bool(w);
    let v2 = if v1 { 1074213 } else { 1074187 };
    quester::tell(w, Quester::Mobile(id), pm, v2);
}

fn landy_new(w: &mut World) -> MobileId {
    let id = new_creature(w, &LANDY, AIType::AiVendor, FightMode::None, 2, 1);
    set_title(w, id, Some("the soil nurturer".to_string()));
    set_race_named(w, id, "Elf");
    set_body(w, id, Body(0x25D));
    set_female(w, id, false);
    let v1 = race_skin_hue(w, id);
    set_hue(w, id, v1);
    set_speed(w, id, 0.5, 2.0, true);
    init_stats(w, id, 100, 100, 25);
    assign_random_hair(w, id);
    set_skill_range(w, id, SkillName::Meditation, 60.0, 80.0);
    set_skill_range(w, id, SkillName::Focus, 60.0, 80.0);
    let v2 = utility::random_yellow_hue(&mut w.rng);
    add_item_hue(w, id, "Sandals", Some(v2));
    let v3 = utility::random_yellow_hue(&mut w.rng);
    add_item_hue(w, id, "ShortPants", Some(v3));
    let v4 = utility::random_yellow_hue(&mut w.rng);
    add_item_hue(w, id, "Tunic", Some(v4));
    let v5 = utility::random_yellow_hue(&mut w.rng);
    add_item_props(w, id, "LeafGloves", None, Some(v5));
    id
}
creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Landy`.
    LANDY, LANDY_C = "Server.Engines.MLQuests.Definitions.Landy" : BASE_CREATURE / BASE_CREATURE_C, ctor landy_new, encoded false;
    c {
        is_invulnerable: |_w, _id| true,
        default_name: |_w, _id| Some("Landy"),
        can_teach: |_w, _id| true,
        can_shout: |_w, _id| true,
    }
    m {
    }
}
fn landy_shout(w: &mut World, id: MobileId, pm: MobileId) {
    let v1 = rnd::bool(w);
    let v2 = if v1 { 1074211 } else { 1074218 };
    quester::tell(w, Quester::Mobile(id), pm, v2);
}

fn alejaha_new(w: &mut World) -> MobileId {
    let id = new_creature(w, &ALEJAHA, AIType::AiVendor, FightMode::None, 2, 1);
    set_title(w, id, Some("the wise".to_string()));
    set_race_named(w, id, "Elf");
    set_body(w, id, Body(0x25E));
    set_female(w, id, true);
    let v1 = race_skin_hue(w, id);
    set_hue(w, id, v1);
    set_speed(w, id, 0.5, 2.0, true);
    init_stats(w, id, 100, 100, 25);
    assign_random_hair(w, id);
    set_skill_range(w, id, SkillName::Meditation, 60.0, 80.0);
    set_skill_range(w, id, SkillName::Focus, 60.0, 80.0);
    let v2 = utility::random_yellow_hue(&mut w.rng);
    add_item_hue(w, id, "Sandals", Some(v2));
    let v3 = utility::random_yellow_hue(&mut w.rng);
    add_item_hue(w, id, "ElvenShirt", Some(v3));
    add_item_hue(w, id, "GemmedCirclet", None);
    let v4 = utility::random_bright_hue(&mut w.rng);
    add_item_hue(w, id, "Cloak", Some(v4));
    let v5 = rnd::bool(w);
    if v5 {
        add_item_hue(w, id, "Kilt", Some(0x387));
    } else {
        add_item_hue(w, id, "Skirt", Some(0x387));
    }
    id
}
creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Alejaha`.
    ALEJAHA, ALEJAHA_C = "Server.Engines.MLQuests.Definitions.Alejaha" : BASE_CREATURE / BASE_CREATURE_C, ctor alejaha_new, encoded false;
    c {
        is_invulnerable: |_w, _id| true,
        default_name: |_w, _id| Some("Elder Alejaha"),
        can_teach: |_w, _id| true,
        can_shout: |_w, _id| true,
    }
    m {
    }
}
fn alejaha_shout(w: &mut World, id: MobileId, pm: MobileId) {
    quester::tell(w, Quester::Mobile(id), pm, 1074223);
}

fn mielan_new(w: &mut World) -> MobileId {
    let id = new_creature(w, &MIELAN, AIType::AiVendor, FightMode::None, 2, 1);
    set_title(w, id, Some("the arcanist".to_string()));
    set_race_named(w, id, "Elf");
    set_body(w, id, Body(0x25D));
    set_female(w, id, false);
    let v1 = race_skin_hue(w, id);
    set_hue(w, id, v1);
    set_speed(w, id, 0.5, 2.0, true);
    init_stats(w, id, 100, 100, 25);
    assign_random_hair(w, id);
    set_skill_range(w, id, SkillName::Meditation, 60.0, 80.0);
    set_skill_range(w, id, SkillName::Focus, 60.0, 80.0);
    add_item_hue(w, id, "ElvenBoots", Some(0x901));
    add_item_hue(w, id, "ElvenShirt", Some(0x56));
    add_item_hue(w, id, "GemmedCirclet", None);
    add_item_hue(w, id, "ElvenPants", Some(0x901));
    id
}
creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Mielan`.
    MIELAN, MIELAN_C = "Server.Engines.MLQuests.Definitions.Mielan" : BASE_CREATURE / BASE_CREATURE_C, ctor mielan_new, encoded false;
    c {
        is_invulnerable: |_w, _id| true,
        default_name: |_w, _id| Some("Mielan"),
        can_teach: |_w, _id| true,
        can_shout: |_w, _id| true,
    }
    m {
    }
}
fn mielan_shout(w: &mut World, id: MobileId, pm: MobileId) {
    let v1 = rnd::bool(w);
    let v2 = if v1 { 1074219 } else { 1074223 };
    quester::tell(w, Quester::Mobile(id), pm, v2);
}

fn ciala_new(w: &mut World) -> MobileId {
    let id = new_creature(w, &CIALA, AIType::AiVendor, FightMode::None, 2, 1);
    set_title(w, id, Some("the arborist".to_string()));
    set_race_named(w, id, "Elf");
    set_body(w, id, Body(0x25E));
    set_female(w, id, true);
    let v1 = race_skin_hue(w, id);
    set_hue(w, id, v1);
    set_speed(w, id, 0.5, 2.0, true);
    init_stats(w, id, 100, 100, 25);
    assign_random_hair(w, id);
    set_skill_range(w, id, SkillName::Meditation, 60.0, 80.0);
    set_skill_range(w, id, SkillName::Focus, 60.0, 80.0);
    let v2 = utility::random_blue_hue(&mut w.rng);
    add_item_hue(w, id, "Skirt", Some(v2));
    let v3 = utility::random_yellow_hue(&mut w.rng);
    add_item_hue(w, id, "ElvenShirt", Some(v3));
    add_item_hue(w, id, "RoyalCirclet", None);
    let v4 = rnd::bool(w);
    if v4 {
        let v55 = utility::random_yellow_hue(&mut w.rng);
        add_item_hue(w, id, "Boots", Some(v55));
    } else {
        let v55 = utility::random_yellow_hue(&mut w.rng);
        add_item_hue(w, id, "ThighBoots", Some(v55));
    }
    id
}
creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Ciala`.
    CIALA, CIALA_C = "Server.Engines.MLQuests.Definitions.Ciala" : BASE_CREATURE / BASE_CREATURE_C, ctor ciala_new, encoded false;
    c {
        is_invulnerable: |_w, _id| true,
        default_name: |_w, _id| Some("Ciala"),
        can_teach: |_w, _id| true,
        can_shout: |_w, _id| true,
    }
    m {
    }
}
fn ciala_shout(w: &mut World, id: MobileId, pm: MobileId) {
    let v1 = rnd::bool(w);
    let v2 = if v1 { 1074206 } else { 1074186 };
    quester::tell(w, Quester::Mobile(id), pm, v2);
}

fn aniel_new(w: &mut World) -> MobileId {
    let id = new_creature(w, &ANIEL, AIType::AiVendor, FightMode::None, 2, 1);
    set_title(w, id, Some("the arborist".to_string()));
    set_race_named(w, id, "Elf");
    set_body(w, id, Body(0x25D));
    set_female(w, id, false);
    let v1 = race_skin_hue(w, id);
    set_hue(w, id, v1);
    set_speed(w, id, 0.5, 2.0, true);
    init_stats(w, id, 100, 100, 25);
    assign_random_hair(w, id);
    set_skill_range(w, id, SkillName::Meditation, 60.0, 80.0);
    set_skill_range(w, id, SkillName::Focus, 60.0, 80.0);
    add_item_hue(w, id, "ElvenPants", Some(0x901));
    add_item_hue(w, id, "LeafChest", None);
    let v2 = utility::random_yellow_hue(&mut w.rng);
    add_item_hue(w, id, "HalfApron", Some(v2));
    add_item_hue(w, id, "ElvenBoots", Some(0x901));
    id
}
creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Aniel`.
    ANIEL, ANIEL_C = "Server.Engines.MLQuests.Definitions.Aniel" : BASE_CREATURE / BASE_CREATURE_C, ctor aniel_new, encoded false;
    c {
        is_invulnerable: |_w, _id| true,
        default_name: |_w, _id| Some("Aniel"),
        can_teach: |_w, _id| true,
        can_shout: |_w, _id| true,
    }
    m {
    }
}
fn aniel_shout(w: &mut World, id: MobileId, pm: MobileId) {
    let v1 = rnd::bool(w);
    let v2 = if v1 { 1074220 } else { 1074222 };
    quester::tell(w, Quester::Mobile(id), pm, v2);
}

fn aulan_new(w: &mut World) -> MobileId {
    let id = new_creature(w, &AULAN, AIType::AiVendor, FightMode::None, 2, 1);
    set_title(w, id, Some("the expeditionist".to_string()));
    set_race_named(w, id, "Elf");
    set_body(w, id, Body(0x25D));
    set_female(w, id, false);
    let v1 = race_skin_hue(w, id);
    set_hue(w, id, v1);
    set_speed(w, id, 0.5, 2.0, true);
    init_stats(w, id, 100, 100, 25);
    assign_random_hair(w, id);
    set_skill_range(w, id, SkillName::Meditation, 60.0, 80.0);
    set_skill_range(w, id, SkillName::Focus, 60.0, 80.0);
    let item = make_named(w, "ElvenBoots");
    let v2 = utility::random_yellow_hue(&mut w.rng);
    set_item_hue(w, item, v2);
    add_item_id(w, id, item);
    let v3 = utility::random_green_hue(&mut w.rng);
    add_item_hue(w, id, "ElvenPants", Some(v3));
    let v4 = utility::random_green_hue(&mut w.rng);
    add_item_hue(w, id, "Cloak", Some(v4));
    add_item_hue(w, id, "Circlet", None);
    let item = make_named(w, "HideChest");
    let v5 = utility::random_yellow_hue(&mut w.rng);
    set_item_hue(w, item, v5);
    add_item_id(w, id, item);
    let item = make_named(w, "HideGloves");
    let v6 = utility::random_yellow_hue(&mut w.rng);
    set_item_hue(w, item, v6);
    add_item_id(w, id, item);
    id
}
creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Aulan`.
    AULAN, AULAN_C = "Server.Engines.MLQuests.Definitions.Aulan" : BASE_CREATURE / BASE_CREATURE_C, ctor aulan_new, encoded false;
    c {
        is_invulnerable: |_w, _id| true,
        default_name: |_w, _id| Some("Aulan"),
        can_teach: |_w, _id| true,
        can_shout: |_w, _id| true,
    }
    m {
    }
}
fn aulan_shout(w: &mut World, id: MobileId, pm: MobileId) {
    let v2 = rnd::below(w, 3);
    let v1 = match v2 {
        0 => 1074188,
        1 => 1074191,
        _ => 1074195,
    };
    quester::tell(w, Quester::Mobile(id), pm, v1);
}

fn brinnae_new(w: &mut World) -> MobileId {
    let id = new_creature(w, &BRINNAE, AIType::AiVendor, FightMode::None, 2, 1);
    set_title(w, id, Some("the wise".to_string()));
    set_race_named(w, id, "Elf");
    set_body(w, id, Body(0x25E));
    set_female(w, id, true);
    let v1 = race_skin_hue(w, id);
    set_hue(w, id, v1);
    set_speed(w, id, 0.5, 2.0, true);
    init_stats(w, id, 100, 100, 25);
    assign_random_hair(w, id);
    add_item_hue(w, id, "ElvenBoots", None);
    add_item_hue(w, id, "FemaleLeafChest", None);
    add_item_hue(w, id, "LeafArms", None);
    add_item_hue(w, id, "HidePants", None);
    add_item_hue(w, id, "ElvenCompositeLongbow", None);
    id
}
creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Brinnae`.
    BRINNAE, BRINNAE_C = "Server.Engines.MLQuests.Definitions.Brinnae" : BASE_CREATURE / BASE_CREATURE_C, ctor brinnae_new, encoded false;
    c {
        is_invulnerable: |_w, _id| true,
        default_name: |_w, _id| Some("Brinnae"),
        can_shout: |_w, _id| true,
    }
    m {
    }
}
fn brinnae_shout(w: &mut World, id: MobileId, pm: MobileId) {
    let v1 = rnd::bool(w);
    let v2 = if v1 { 1074212 } else { 1074210 };
    quester::tell(w, Quester::Mobile(id), pm, v2);
}

fn caelas_new(w: &mut World) -> MobileId {
    let id = new_creature(w, &CAELAS, AIType::AiVendor, FightMode::None, 2, 1);
    set_title(w, id, Some("the wise".to_string()));
    set_race_named(w, id, "Elf");
    set_body(w, id, Body(0x25D));
    set_female(w, id, false);
    let v1 = race_skin_hue(w, id);
    set_hue(w, id, v1);
    set_speed(w, id, 0.5, 2.0, true);
    init_stats(w, id, 100, 100, 25);
    assign_random_hair(w, id);
    set_skill_range(w, id, SkillName::Meditation, 60.0, 80.0);
    set_skill_range(w, id, SkillName::Focus, 60.0, 80.0);
    add_item_hue(w, id, "ElvenBoots", Some(0x1BB));
    add_item_hue(w, id, "MaleElvenRobe", Some(0x489));
    add_item_hue(w, id, "Cloak", Some(0x718));
    add_item_hue(w, id, "RoyalCirclet", None);
    id
}
creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Caelas`.
    CAELAS, CAELAS_C = "Server.Engines.MLQuests.Definitions.Caelas" : BASE_CREATURE / BASE_CREATURE_C, ctor caelas_new, encoded false;
    c {
        is_invulnerable: |_w, _id| true,
        default_name: |_w, _id| Some("Elder Caelas"),
        can_teach: |_w, _id| true,
        can_shout: |_w, _id| true,
    }
    m {
    }
}
fn caelas_shout(w: &mut World, id: MobileId, pm: MobileId) {
    let v1 = rnd::bool(w);
    let v2 = if v1 { 1074204 } else { 1074201 };
    quester::tell(w, Quester::Mobile(id), pm, v2);
}

fn clehin_new(w: &mut World) -> MobileId {
    let id = new_creature(w, &CLEHIN, AIType::AiVendor, FightMode::None, 2, 1);
    set_title(w, id, Some("the soil nurturer".to_string()));
    set_race_named(w, id, "Elf");
    set_body(w, id, Body(0x25E));
    set_female(w, id, true);
    let v1 = race_skin_hue(w, id);
    set_hue(w, id, v1);
    set_speed(w, id, 0.5, 2.0, true);
    init_stats(w, id, 100, 100, 25);
    assign_random_hair(w, id);
    set_skill_range(w, id, SkillName::Meditation, 60.0, 80.0);
    set_skill_range(w, id, SkillName::Focus, 60.0, 80.0);
    add_item_hue(w, id, "ElvenBoots", None);
    add_item_hue(w, id, "ElvenShirt", None);
    add_item_hue(w, id, "LeafTonlet", None);
    id
}
creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Clehin`.
    CLEHIN, CLEHIN_C = "Server.Engines.MLQuests.Definitions.Clehin" : BASE_CREATURE / BASE_CREATURE_C, ctor clehin_new, encoded false;
    c {
        is_invulnerable: |_w, _id| true,
        default_name: |_w, _id| Some("Clehin"),
        can_teach: |_w, _id| true,
        can_shout: |_w, _id| true,
    }
    m {
    }
}
fn clehin_shout(w: &mut World, id: MobileId, pm: MobileId) {
    let v1 = rnd::bool(w);
    let v2 = if v1 { 1074211 } else { 1074186 };
    quester::tell(w, Quester::Mobile(id), pm, v2);
}

fn cloorne_new(w: &mut World) -> MobileId {
    let id = new_creature(w, &CLOORNE, AIType::AiVendor, FightMode::None, 2, 1);
    set_title(w, id, Some("the expeditionist".to_string()));
    set_race_named(w, id, "Elf");
    set_body(w, id, Body(0x25D));
    set_female(w, id, false);
    let v1 = race_skin_hue(w, id);
    set_hue(w, id, v1);
    set_speed(w, id, 0.5, 2.0, true);
    init_stats(w, id, 100, 100, 25);
    assign_random_hair(w, id);
    set_skill_range(w, id, SkillName::Meditation, 60.0, 80.0);
    set_skill_range(w, id, SkillName::Focus, 60.0, 80.0);
    add_item_hue(w, id, "ElvenBoots", Some(0x3B2));
    add_item_hue(w, id, "RadiantScimitar", None);
    add_item_hue(w, id, "WingedHelm", None);
    let item = make_named(w, "WoodlandLegs");
    set_item_hue(w, item, 0x74A);
    add_item_id(w, id, item);
    let item = make_named(w, "HideChest");
    set_item_hue(w, item, 0x726);
    add_item_id(w, id, item);
    let item = make_named(w, "LeafArms");
    set_item_hue(w, item, 0x73E);
    add_item_id(w, id, item);
    id
}
creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Cloorne`.
    CLOORNE, CLOORNE_C = "Server.Engines.MLQuests.Definitions.Cloorne" : BASE_CREATURE / BASE_CREATURE_C, ctor cloorne_new, encoded false;
    c {
        is_invulnerable: |_w, _id| true,
        default_name: |_w, _id| Some("Cloorne"),
        can_teach: |_w, _id| true,
        can_shout: |_w, _id| true,
    }
    m {
    }
}
fn cloorne_shout(w: &mut World, id: MobileId, pm: MobileId) {
    let v1 = rnd::bool(w);
    let v2 = if v1 { 1074185 } else { 1074186 };
    quester::tell(w, Quester::Mobile(id), pm, v2);
}

fn salaenih_new(w: &mut World) -> MobileId {
    let id = new_creature(w, &SALAENIH, AIType::AiVendor, FightMode::None, 2, 1);
    set_title(w, id, Some("the expeditionist".to_string()));
    set_race_named(w, id, "Elf");
    set_body(w, id, Body(0x25E));
    set_female(w, id, true);
    let v1 = race_skin_hue(w, id);
    set_hue(w, id, v1);
    set_speed(w, id, 0.5, 2.0, true);
    init_stats(w, id, 100, 100, 25);
    assign_random_hair(w, id);
    set_skill_range(w, id, SkillName::Meditation, 60.0, 80.0);
    set_skill_range(w, id, SkillName::Focus, 60.0, 80.0);
    add_item_hue(w, id, "ElvenBoots", None);
    add_item_hue(w, id, "WarCleaver", None);
    let item = make_named(w, "WoodlandBelt");
    set_item_hue(w, item, 0x597);
    add_item_id(w, id, item);
    let item = make_named(w, "VultureHelm");
    set_item_hue(w, item, 0x1BB);
    add_item_id(w, id, item);
    let item = make_named(w, "WoodlandLegs");
    set_item_hue(w, item, 0x1BB);
    add_item_id(w, id, item);
    let item = make_named(w, "WoodlandChest");
    set_item_hue(w, item, 0x1BB);
    add_item_id(w, id, item);
    let item = make_named(w, "WoodlandArms");
    set_item_hue(w, item, 0x1BB);
    add_item_id(w, id, item);
    id
}
creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Salaenih`.
    SALAENIH, SALAENIH_C = "Server.Engines.MLQuests.Definitions.Salaenih" : BASE_CREATURE / BASE_CREATURE_C, ctor salaenih_new, encoded false;
    c {
        is_invulnerable: |_w, _id| true,
        default_name: |_w, _id| Some("Salaenih"),
        can_teach: |_w, _id| true,
        can_shout: |_w, _id| true,
    }
    m {
    }
}
fn salaenih_shout(w: &mut World, id: MobileId, pm: MobileId) {
    let v1 = rnd::bool(w);
    let v2 = if v1 { 1074207 } else { 1074209 };
    quester::tell(w, Quester::Mobile(id), pm, v2);
}

fn vilo_new(w: &mut World) -> MobileId {
    let id = new_creature(w, &VILO, AIType::AiVendor, FightMode::None, 2, 1);
    set_title(w, id, Some("the guard".to_string()));
    set_race_named(w, id, "Elf");
    set_body(w, id, Body(0x25D));
    set_female(w, id, false);
    let v1 = race_skin_hue(w, id);
    set_hue(w, id, v1);
    set_speed(w, id, 0.5, 2.0, true);
    init_stats(w, id, 100, 100, 25);
    assign_random_hair(w, id);
    set_skill_range(w, id, SkillName::Meditation, 60.0, 80.0);
    set_skill_range(w, id, SkillName::Focus, 60.0, 80.0);
    add_item_hue(w, id, "ElvenBoots", Some(0x901));
    add_item_hue(w, id, "OrnateAxe", None);
    add_item_hue(w, id, "WoodlandBelt", Some(0x592));
    add_item_hue(w, id, "VultureHelm", None);
    add_item_hue(w, id, "WoodlandLegs", None);
    add_item_hue(w, id, "WoodlandChest", None);
    add_item_hue(w, id, "WoodlandArms", None);
    add_item_hue(w, id, "WoodlandGorget", None);
    id
}
creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Vilo`.
    VILO, VILO_C = "Server.Engines.MLQuests.Definitions.Vilo" : BASE_CREATURE / BASE_CREATURE_C, ctor vilo_new, encoded false;
    c {
        is_invulnerable: |_w, _id| true,
        default_name: |_w, _id| Some("Vilo"),
        can_teach: |_w, _id| true,
        can_shout: |_w, _id| true,
    }
    m {
    }
}
fn vilo_shout(w: &mut World, id: MobileId, pm: MobileId) {
    let v1 = rnd::bool(w);
    let v2 = if v1 { 1074210 } else { 1074220 };
    quester::tell(w, Quester::Mobile(id), pm, v2);
}

fn tholef_new(w: &mut World) -> MobileId {
    let id = new_creature(w, &THOLEF, AIType::AiVendor, FightMode::None, 2, 1);
    set_title(w, id, Some("the grape tender".to_string()));
    set_race_named(w, id, "Elf");
    set_body(w, id, Body(0x25D));
    set_female(w, id, false);
    let v1 = race_skin_hue(w, id);
    set_hue(w, id, v1);
    set_speed(w, id, 0.5, 2.0, true);
    init_stats(w, id, 100, 100, 25);
    assign_random_hair(w, id);
    set_skill_range(w, id, SkillName::Meditation, 60.0, 80.0);
    set_skill_range(w, id, SkillName::Focus, 60.0, 80.0);
    add_item_hue(w, id, "Sandals", Some(0x901));
    add_item_hue(w, id, "FullApron", Some(0x756));
    add_item_hue(w, id, "ShortPants", Some(0x28C));
    add_item_hue(w, id, "Shirt", Some(0x28C));
    add_item_props(w, id, "LeafArms", None, Some(0x28C));
    id
}
creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Tholef`.
    THOLEF, THOLEF_C = "Server.Engines.MLQuests.Definitions.Tholef" : BASE_CREATURE / BASE_CREATURE_C, ctor tholef_new, encoded false;
    c {
        is_invulnerable: |_w, _id| true,
        default_name: |_w, _id| Some("Tholef"),
        can_teach: |_w, _id| true,
        can_shout: |_w, _id| true,
    }
    m {
    }
}
fn tholef_shout(w: &mut World, id: MobileId, pm: MobileId) {
    let v1 = rnd::bool(w);
    let v2 = if v1 { 1074209 } else { 1074184 };
    quester::tell(w, Quester::Mobile(id), pm, v2);
}

fn tillanil_new(w: &mut World) -> MobileId {
    let id = new_creature(w, &TILLANIL, AIType::AiVendor, FightMode::None, 2, 1);
    set_title(w, id, Some("the grape tender".to_string()));
    set_race_named(w, id, "Elf");
    set_body(w, id, Body(0x25E));
    set_female(w, id, true);
    let v1 = race_skin_hue(w, id);
    set_hue(w, id, v1);
    set_speed(w, id, 0.5, 2.0, true);
    init_stats(w, id, 100, 100, 25);
    assign_random_hair(w, id);
    set_skill_range(w, id, SkillName::Meditation, 60.0, 80.0);
    set_skill_range(w, id, SkillName::Focus, 60.0, 80.0);
    add_item_hue(w, id, "Sandals", Some(0x1BB));
    add_item_hue(w, id, "Tunic", Some(0x759));
    add_item_hue(w, id, "ShortPants", Some(0x21));
    id
}
creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Tillanil`.
    TILLANIL, TILLANIL_C = "Server.Engines.MLQuests.Definitions.Tillanil" : BASE_CREATURE / BASE_CREATURE_C, ctor tillanil_new, encoded false;
    c {
        is_invulnerable: |_w, _id| true,
        default_name: |_w, _id| Some("Tillanil"),
        can_teach: |_w, _id| true,
        can_shout: |_w, _id| true,
    }
    m {
    }
}
fn tillanil_shout(w: &mut World, id: MobileId, pm: MobileId) {
    let v1 = rnd::bool(w);
    let v2 = if v1 { 1074187 } else { 1074222 };
    quester::tell(w, Quester::Mobile(id), pm, v2);
}

fn waelian_new(w: &mut World) -> MobileId {
    let id = new_creature(w, &WAELIAN, AIType::AiVendor, FightMode::None, 2, 1);
    set_title(w, id, Some("the trinket weaver".to_string()));
    set_race_named(w, id, "Elf");
    set_body(w, id, Body(0x25D));
    set_female(w, id, false);
    let v1 = race_skin_hue(w, id);
    set_hue(w, id, v1);
    set_speed(w, id, 0.5, 2.0, true);
    init_stats(w, id, 100, 100, 25);
    assign_random_hair(w, id);
    set_skill_range(w, id, SkillName::Meditation, 60.0, 80.0);
    set_skill_range(w, id, SkillName::Focus, 60.0, 80.0);
    add_item_hue(w, id, "Shoes", Some(0x901));
    add_item_hue(w, id, "SmithHammer", None);
    add_item_hue(w, id, "LongPants", Some(0x340));
    add_item_hue(w, id, "GemmedCirclet", None);
    add_item_props(w, id, "LeafChest", None, Some(0x344));
    id
}
creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Waelian`.
    WAELIAN, WAELIAN_C = "Server.Engines.MLQuests.Definitions.Waelian" : BASE_CREATURE / BASE_CREATURE_C, ctor waelian_new, encoded false;
    c {
        is_invulnerable: |_w, _id| true,
        default_name: |_w, _id| Some("Waelian"),
        can_teach: |_w, _id| true,
        can_shout: |_w, _id| true,
    }
    m {
    }
}
fn waelian_shout(w: &mut World, id: MobileId, pm: MobileId) {
    let v1 = rnd::bool(w);
    let v2 = if v1 { 1074221 } else { 1074201 };
    quester::tell(w, Quester::Mobile(id), pm, v2);
}

fn sleen_new(w: &mut World) -> MobileId {
    let id = new_creature(w, &SLEEN, AIType::AiVendor, FightMode::None, 2, 1);
    set_title(w, id, Some("the trinket weaver".to_string()));
    set_race_named(w, id, "Elf");
    set_body(w, id, Body(0x25E));
    set_female(w, id, true);
    let v1 = race_skin_hue(w, id);
    set_hue(w, id, v1);
    set_speed(w, id, 0.5, 2.0, true);
    init_stats(w, id, 100, 100, 25);
    assign_random_hair(w, id);
    set_skill_range(w, id, SkillName::Meditation, 60.0, 80.0);
    set_skill_range(w, id, SkillName::Focus, 60.0, 80.0);
    add_item_hue(w, id, "ElvenBoots", Some(0x901));
    add_item_hue(w, id, "SmithHammer", None);
    add_item_hue(w, id, "Cloak", Some(0x75A));
    add_item_hue(w, id, "ElvenShirt", None);
    id
}
creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Sleen`.
    SLEEN, SLEEN_C = "Server.Engines.MLQuests.Definitions.Sleen" : BASE_CREATURE / BASE_CREATURE_C, ctor sleen_new, encoded false;
    c {
        is_invulnerable: |_w, _id| true,
        default_name: |_w, _id| Some("Sleen"),
        can_teach: |_w, _id| true,
        can_shout: |_w, _id| true,
    }
    m {
    }
}
fn sleen_shout(w: &mut World, id: MobileId, pm: MobileId) {
    let v1 = rnd::bool(w);
    let v2 = if v1 { 1074200 } else { 1074206 };
    quester::tell(w, Quester::Mobile(id), pm, v2);
}

fn unoelil_new(w: &mut World) -> MobileId {
    let id = new_creature(w, &UNOELIL, AIType::AiVendor, FightMode::None, 2, 1);
    set_title(w, id, Some("the bark weaver".to_string()));
    set_race_named(w, id, "Elf");
    set_body(w, id, Body(0x25E));
    set_female(w, id, true);
    let v1 = race_skin_hue(w, id);
    set_hue(w, id, v1);
    set_speed(w, id, 0.5, 2.0, true);
    init_stats(w, id, 100, 100, 25);
    assign_random_hair(w, id);
    set_skill_range(w, id, SkillName::Meditation, 60.0, 80.0);
    set_skill_range(w, id, SkillName::Focus, 60.0, 80.0);
    add_item_hue(w, id, "ElvenBoots", Some(0x1BB));
    add_item_hue(w, id, "ShortPants", Some(0x1BB));
    add_item_hue(w, id, "Tunic", Some(0x64D));
    id
}
creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Unoelil`.
    UNOELIL, UNOELIL_C = "Server.Engines.MLQuests.Definitions.Unoelil" : BASE_CREATURE / BASE_CREATURE_C, ctor unoelil_new, encoded false;
    c {
        is_invulnerable: |_w, _id| true,
        default_name: |_w, _id| Some("Unoelil"),
        can_teach: |_w, _id| true,
        can_shout: |_w, _id| true,
    }
    m {
    }
}
fn unoelil_shout(w: &mut World, id: MobileId, pm: MobileId) {
    let v1 = rnd::bool(w);
    let v2 = if v1 { 1074209 } else { 1074186 };
    quester::tell(w, Quester::Mobile(id), pm, v2);
}

fn anolly_new(w: &mut World) -> MobileId {
    let id = new_creature(w, &ANOLLY, AIType::AiVendor, FightMode::None, 2, 1);
    set_title(w, id, Some("the bark weaver".to_string()));
    set_race_named(w, id, "Elf");
    set_body(w, id, Body(0x25D));
    set_female(w, id, false);
    let v1 = race_skin_hue(w, id);
    set_hue(w, id, v1);
    set_speed(w, id, 0.5, 2.0, true);
    init_stats(w, id, 100, 100, 25);
    assign_random_hair(w, id);
    set_skill_range(w, id, SkillName::Meditation, 60.0, 80.0);
    set_skill_range(w, id, SkillName::Focus, 60.0, 80.0);
    add_item_hue(w, id, "Sandals", Some(0x901));
    add_item_hue(w, id, "ShortPants", Some(0x3B3));
    add_item_hue(w, id, "FullApron", Some(0x1BB));
    add_item_hue(w, id, "SmithHammer", None);
    id
}
creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Anolly`.
    ANOLLY, ANOLLY_C = "Server.Engines.MLQuests.Definitions.Anolly" : BASE_CREATURE / BASE_CREATURE_C, ctor anolly_new, encoded false;
    c {
        is_invulnerable: |_w, _id| true,
        default_name: |_w, _id| Some("Anolly"),
        can_teach: |_w, _id| true,
    }
    m {
    }
}

fn jusae_new(w: &mut World) -> MobileId {
    let id = new_creature(w, &JUSAE, AIType::AiVendor, FightMode::None, 2, 1);
    set_title(w, id, Some("the bowcrafter".to_string()));
    set_race_named(w, id, "Elf");
    set_body(w, id, Body(0x25D));
    set_female(w, id, false);
    let v1 = race_skin_hue(w, id);
    set_hue(w, id, v1);
    set_speed(w, id, 0.5, 2.0, true);
    init_stats(w, id, 100, 100, 25);
    assign_random_hair(w, id);
    set_skill_range(w, id, SkillName::Meditation, 60.0, 80.0);
    set_skill_range(w, id, SkillName::Focus, 60.0, 80.0);
    add_item_hue(w, id, "Sandals", Some(0x901));
    add_item_hue(w, id, "ShortPants", Some(0x661));
    add_item_hue(w, id, "MagicalShortbow", None);
    let item = make_named(w, "HideChest");
    set_item_hue(w, item, 0x27B);
    add_item_id(w, id, item);
    let item = make_named(w, "HidePauldrons");
    set_item_hue(w, item, 0x27E);
    add_item_id(w, id, item);
    id
}
creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Jusae`.
    JUSAE, JUSAE_C = "Server.Engines.MLQuests.Definitions.Jusae" : BASE_CREATURE / BASE_CREATURE_C, ctor jusae_new, encoded false;
    c {
        is_invulnerable: |_w, _id| true,
        default_name: |_w, _id| Some("Jusae"),
        can_teach: |_w, _id| true,
        can_shout: |_w, _id| true,
    }
    m {
    }
}
fn jusae_shout(w: &mut World, id: MobileId, pm: MobileId) {
    let v1 = rnd::bool(w);
    let v2 = if v1 { 1074210 } else { 1074213 };
    quester::tell(w, Quester::Mobile(id), pm, v2);
}

fn cillitha_new(w: &mut World) -> MobileId {
    let id = new_creature(w, &CILLITHA, AIType::AiVendor, FightMode::None, 2, 1);
    set_title(w, id, Some("the bowcrafter".to_string()));
    set_race_named(w, id, "Elf");
    set_body(w, id, Body(0x25E));
    set_female(w, id, true);
    let v1 = race_skin_hue(w, id);
    set_hue(w, id, v1);
    set_speed(w, id, 0.5, 2.0, true);
    init_stats(w, id, 100, 100, 25);
    assign_random_hair(w, id);
    set_skill_range(w, id, SkillName::Meditation, 60.0, 80.0);
    set_skill_range(w, id, SkillName::Focus, 60.0, 80.0);
    add_item_hue(w, id, "ElvenBoots", Some(0x901));
    add_item_hue(w, id, "ElvenShirt", Some(0x731));
    add_item_hue(w, id, "LeafLegs", None);
    id
}
creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Cillitha`.
    CILLITHA, CILLITHA_C = "Server.Engines.MLQuests.Definitions.Cillitha" : BASE_CREATURE / BASE_CREATURE_C, ctor cillitha_new, encoded false;
    c {
        is_invulnerable: |_w, _id| true,
        default_name: |_w, _id| Some("Cillitha"),
        can_teach: |_w, _id| true,
        can_shout: |_w, _id| true,
    }
    m {
    }
}
fn cillitha_shout(w: &mut World, id: MobileId, pm: MobileId) {
    let v1 = rnd::bool(w);
    let v2 = if v1 { 1074223 } else { 1074213 };
    quester::tell(w, Quester::Mobile(id), pm, v2);
}

fn lohn_new(w: &mut World) -> MobileId {
    let id = new_creature(w, &LOHN, AIType::AiVendor, FightMode::None, 2, 1);
    set_title(w, id, Some("the metal weaver".to_string()));
    set_race_named(w, id, "Elf");
    set_body(w, id, Body(0x25D));
    set_female(w, id, false);
    let v1 = race_skin_hue(w, id);
    set_hue(w, id, v1);
    set_speed(w, id, 0.5, 2.0, true);
    init_stats(w, id, 100, 100, 25);
    assign_random_hair(w, id);
    set_skill_range(w, id, SkillName::Meditation, 60.0, 80.0);
    set_skill_range(w, id, SkillName::Focus, 60.0, 80.0);
    add_item_hue(w, id, "Shoes", Some(0x901));
    add_item_hue(w, id, "LongPants", Some(0x359));
    add_item_hue(w, id, "SmithHammer", None);
    add_item_hue(w, id, "GemmedCirclet", None);
    add_item_props(w, id, "LeafChest", None, Some(0x359));
    id
}
creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Lohn`.
    LOHN, LOHN_C = "Server.Engines.MLQuests.Definitions.Lohn" : BASE_CREATURE / BASE_CREATURE_C, ctor lohn_new, encoded false;
    c {
        is_invulnerable: |_w, _id| true,
        default_name: |_w, _id| Some("Lohn"),
        can_teach: |_w, _id| true,
        can_shout: |_w, _id| true,
    }
    m {
    }
}
fn lohn_shout(w: &mut World, id: MobileId, pm: MobileId) {
    let v1 = rnd::bool(w);
    let v2 = if v1 { 1074209 } else { 1074187 };
    quester::tell(w, Quester::Mobile(id), pm, v2);
}

fn olla_new(w: &mut World) -> MobileId {
    let id = new_creature(w, &OLLA, AIType::AiVendor, FightMode::None, 2, 1);
    set_title(w, id, Some("the metal weaver".to_string()));
    set_race_named(w, id, "Elf");
    set_body(w, id, Body(0x25E));
    set_female(w, id, true);
    let v1 = race_skin_hue(w, id);
    set_hue(w, id, v1);
    set_speed(w, id, 0.5, 2.0, true);
    init_stats(w, id, 100, 100, 25);
    assign_random_hair(w, id);
    set_skill_range(w, id, SkillName::Meditation, 60.0, 80.0);
    set_skill_range(w, id, SkillName::Focus, 60.0, 80.0);
    add_item_hue(w, id, "ElvenBoots", None);
    add_item_hue(w, id, "LongPants", Some(0x3B3));
    add_item_hue(w, id, "SmithHammer", None);
    add_item_hue(w, id, "FullApron", Some(0x1BB));
    add_item_hue(w, id, "ElvenShirt", None);
    id
}
creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Olla`.
    OLLA, OLLA_C = "Server.Engines.MLQuests.Definitions.Olla" : BASE_CREATURE / BASE_CREATURE_C, ctor olla_new, encoded false;
    c {
        is_invulnerable: |_w, _id| true,
        default_name: |_w, _id| Some("Olla"),
        can_teach: |_w, _id| true,
        can_shout: |_w, _id| true,
    }
    m {
    }
}
fn olla_shout(w: &mut World, id: MobileId, pm: MobileId) {
    let v1 = rnd::bool(w);
    let v2 = if v1 { 1074185 } else { 1074187 };
    quester::tell(w, Quester::Mobile(id), pm, v2);
}

fn thallary_new(w: &mut World) -> MobileId {
    let id = new_creature(w, &THALLARY, AIType::AiVendor, FightMode::None, 2, 1);
    set_title(w, id, Some("the cloth weaver".to_string()));
    set_race_named(w, id, "Elf");
    set_body(w, id, Body(0x25D));
    set_female(w, id, false);
    let v1 = race_skin_hue(w, id);
    set_hue(w, id, v1);
    set_speed(w, id, 0.5, 2.0, true);
    init_stats(w, id, 100, 100, 25);
    assign_random_hair(w, id);
    set_skill_range(w, id, SkillName::Meditation, 60.0, 80.0);
    set_skill_range(w, id, SkillName::Focus, 60.0, 80.0);
    add_item_hue(w, id, "Sandals", Some(0x901));
    add_item_hue(w, id, "LongPants", Some(0x72E));
    add_item_hue(w, id, "Cloak", Some(0x3B3));
    add_item_hue(w, id, "FancyShirt", Some(0x13));
    id
}
creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Thallary`.
    THALLARY, THALLARY_C = "Server.Engines.MLQuests.Definitions.Thallary" : BASE_CREATURE / BASE_CREATURE_C, ctor thallary_new, encoded false;
    c {
        is_invulnerable: |_w, _id| true,
        default_name: |_w, _id| Some("Thallary"),
        can_teach: |_w, _id| true,
        can_shout: |_w, _id| true,
    }
    m {
    }
}
fn thallary_shout(w: &mut World, id: MobileId, pm: MobileId) {
    let v1 = rnd::bool(w);
    let v2 = if v1 { 1074221 } else { 1074212 };
    quester::tell(w, Quester::Mobile(id), pm, v2);
}

fn ahie_new(w: &mut World) -> MobileId {
    let id = new_creature(w, &AHIE, AIType::AiVendor, FightMode::None, 2, 1);
    set_title(w, id, Some("the cloth weaver".to_string()));
    set_race_named(w, id, "Elf");
    set_body(w, id, Body(0x25E));
    set_female(w, id, true);
    let v1 = race_skin_hue(w, id);
    set_hue(w, id, v1);
    set_speed(w, id, 0.5, 2.0, true);
    init_stats(w, id, 100, 100, 25);
    assign_random_hair(w, id);
    set_skill_range(w, id, SkillName::Meditation, 60.0, 80.0);
    set_skill_range(w, id, SkillName::Focus, 60.0, 80.0);
    add_item_hue(w, id, "Boots", Some(0x901));
    add_item_hue(w, id, "Skirt", Some(0x1C));
    add_item_hue(w, id, "Cloak", Some(0x62));
    add_item_hue(w, id, "FancyShirt", Some(0x738));
    id
}
creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Ahie`.
    AHIE, AHIE_C = "Server.Engines.MLQuests.Definitions.Ahie" : BASE_CREATURE / BASE_CREATURE_C, ctor ahie_new, encoded false;
    c {
        is_invulnerable: |_w, _id| true,
        default_name: |_w, _id| Some("Ahie"),
        can_teach: |_w, _id| true,
        can_shout: |_w, _id| true,
    }
    m {
    }
}
fn ahie_shout(w: &mut World, id: MobileId, pm: MobileId) {
    let v1 = rnd::bool(w);
    let v2 = if v1 { 1074206 } else { 1074203 };
    quester::tell(w, Quester::Mobile(id), pm, v2);
}

fn tyeelor_new(w: &mut World) -> MobileId {
    let id = new_creature(w, &TYEELOR, AIType::AiVendor, FightMode::None, 2, 1);
    set_title(w, id, Some("the expeditionist".to_string()));
    set_race_named(w, id, "Elf");
    set_body(w, id, Body(0x25D));
    set_female(w, id, false);
    let v1 = race_skin_hue(w, id);
    set_hue(w, id, v1);
    set_speed(w, id, 0.5, 2.0, true);
    init_stats(w, id, 100, 100, 25);
    assign_random_hair(w, id);
    add_item_hue(w, id, "ElvenBoots", Some(0x1BB));
    let item = make_named(w, "WoodlandLegs");
    set_item_hue(w, item, 0x236);
    add_item_id(w, id, item);
    let item = make_named(w, "WoodlandChest");
    set_item_hue(w, item, 0x236);
    add_item_id(w, id, item);
    let item = make_named(w, "WoodlandArms");
    set_item_hue(w, item, 0x236);
    add_item_id(w, id, item);
    let item = make_named(w, "VultureHelm");
    set_item_hue(w, item, 0x236);
    add_item_id(w, id, item);
    let item = make_named(w, "WoodlandBelt");
    set_item_hue(w, item, 0x236);
    add_item_id(w, id, item);
    id
}
creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Tyeelor`.
    TYEELOR, TYEELOR_C = "Server.Engines.MLQuests.Definitions.Tyeelor" : BASE_CREATURE / BASE_CREATURE_C, ctor tyeelor_new, encoded false;
    c {
        is_invulnerable: |_w, _id| true,
        default_name: |_w, _id| Some("Tyeelor"),
    }
    m {
    }
}

fn athailon_new(w: &mut World) -> MobileId {
    let id = new_creature(w, &ATHAILON, AIType::AiVendor, FightMode::None, 2, 1);
    set_title(w, id, Some("the expeditionist".to_string()));
    set_race_named(w, id, "Elf");
    set_body(w, id, Body(0x25E));
    set_female(w, id, true);
    let v1 = race_skin_hue(w, id);
    set_hue(w, id, v1);
    set_speed(w, id, 0.5, 2.0, true);
    init_stats(w, id, 100, 100, 25);
    assign_random_hair(w, id);
    add_item_hue(w, id, "ElvenBoots", Some(0x901));
    add_item_hue(w, id, "WoodlandBelt", None);
    add_item_hue(w, id, "DiamondMace", None);
    let item = make_named(w, "WoodlandLegs");
    set_item_hue(w, item, 0x3B2);
    add_item_id(w, id, item);
    let item = make_named(w, "FemaleElvenPlateChest");
    set_item_hue(w, item, 0x3B2);
    add_item_id(w, id, item);
    let item = make_named(w, "WoodlandArms");
    set_item_hue(w, item, 0x3B2);
    add_item_id(w, id, item);
    let item = make_named(w, "WingedHelm");
    set_item_hue(w, item, 0x3B2);
    add_item_id(w, id, item);
    id
}
creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Athailon`.
    ATHAILON, ATHAILON_C = "Server.Engines.MLQuests.Definitions.Athailon" : BASE_CREATURE / BASE_CREATURE_C, ctor athailon_new, encoded false;
    c {
        is_invulnerable: |_w, _id| true,
        default_name: |_w, _id| Some("Athailon"),
    }
    m {
    }
}

fn elder_taellia_new(w: &mut World) -> MobileId {
    let id = new_creature(w, &ELDER_TAELLIA, AIType::AiVendor, FightMode::None, 2, 1);
    set_title(w, id, Some("the wise".to_string()));
    set_race_named(w, id, "Elf");
    set_body(w, id, Body(0x25E));
    set_female(w, id, true);
    let v1 = race_skin_hue(w, id);
    set_hue(w, id, v1);
    set_speed(w, id, 0.5, 2.0, true);
    init_stats(w, id, 100, 100, 25);
    assign_random_hair(w, id);
    add_item_hue(w, id, "ThighBoots", Some(0x127));
    let v2 = utility::random_bright_hue(&mut w.rng);
    add_item_hue(w, id, "FemaleElvenRobe", Some(v2));
    add_item_hue(w, id, "MagicWand", None);
    add_item_hue(w, id, "Circlet", None);
    id
}
creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.ElderTaellia`.
    ELDER_TAELLIA, ELDER_TAELLIA_C = "Server.Engines.MLQuests.Definitions.ElderTaellia" : BASE_CREATURE / BASE_CREATURE_C, ctor elder_taellia_new, encoded false;
    c {
        is_invulnerable: |_w, _id| true,
        default_name: |_w, _id| Some("Elder Taellia"),
    }
    m {
    }
}

fn elder_mallew_new(w: &mut World) -> MobileId {
    let id = new_creature(w, &ELDER_MALLEW, AIType::AiVendor, FightMode::None, 2, 1);
    set_title(w, id, Some("the wise".to_string()));
    set_race_named(w, id, "Elf");
    set_body(w, id, Body(0x25D));
    set_female(w, id, false);
    let v1 = race_skin_hue(w, id);
    set_hue(w, id, v1);
    set_speed(w, id, 0.5, 2.0, true);
    init_stats(w, id, 100, 100, 25);
    assign_random_hair(w, id);
    add_item_hue(w, id, "ElvenBoots", Some(0x1BB));
    add_item_hue(w, id, "Cloak", Some(0x3B2));
    add_item_hue(w, id, "Circlet", None);
    let item = make_named(w, "LeafTonlet");
    set_item_hue(w, item, 0x544);
    add_item_id(w, id, item);
    let item = make_named(w, "LeafChest");
    set_item_hue(w, item, 0x538);
    add_item_id(w, id, item);
    let item = make_named(w, "LeafArms");
    set_item_hue(w, item, 0x528);
    add_item_id(w, id, item);
    id
}
creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.ElderMallew`.
    ELDER_MALLEW, ELDER_MALLEW_C = "Server.Engines.MLQuests.Definitions.ElderMallew" : BASE_CREATURE / BASE_CREATURE_C, ctor elder_mallew_new, encoded false;
    c {
        is_invulnerable: |_w, _id| true,
        default_name: |_w, _id| Some("Elder Mallew"),
    }
    m {
    }
}

fn elder_abbein_new(w: &mut World) -> MobileId {
    let id = new_creature(w, &ELDER_ABBEIN, AIType::AiVendor, FightMode::None, 2, 1);
    set_title(w, id, Some("the wise".to_string()));
    set_race_named(w, id, "Elf");
    set_body(w, id, Body(0x25E));
    set_female(w, id, true);
    let v1 = race_skin_hue(w, id);
    set_hue(w, id, v1);
    set_speed(w, id, 0.5, 2.0, true);
    init_stats(w, id, 100, 100, 25);
    assign_random_hair(w, id);
    add_item_hue(w, id, "ElvenBoots", Some(0x72C));
    add_item_hue(w, id, "FemaleElvenRobe", Some(0x8B0));
    add_item_hue(w, id, "RoyalCirclet", None);
    id
}
creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.ElderAbbein`.
    ELDER_ABBEIN, ELDER_ABBEIN_C = "Server.Engines.MLQuests.Definitions.ElderAbbein" : BASE_CREATURE / BASE_CREATURE_C, ctor elder_abbein_new, encoded false;
    c {
        is_invulnerable: |_w, _id| true,
        default_name: |_w, _id| Some("Elder Abbein"),
    }
    m {
    }
}

fn elder_vicaie_new(w: &mut World) -> MobileId {
    let id = new_creature(w, &ELDER_VICAIE, AIType::AiVendor, FightMode::None, 2, 1);
    set_title(w, id, Some("the wise".to_string()));
    set_race_named(w, id, "Elf");
    set_body(w, id, Body(0x25E));
    set_female(w, id, true);
    let v1 = race_skin_hue(w, id);
    set_hue(w, id, v1);
    set_speed(w, id, 0.5, 2.0, true);
    init_stats(w, id, 100, 100, 25);
    assign_random_hair(w, id);
    add_item_hue(w, id, "ElvenBoots", None);
    add_item_hue(w, id, "Tunic", Some(0x732));
    add_item_props(w, id, "LeafLegs", None, Some(0x3B2));
    id
}
creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.ElderVicaie`.
    ELDER_VICAIE, ELDER_VICAIE_C = "Server.Engines.MLQuests.Definitions.ElderVicaie" : BASE_CREATURE / BASE_CREATURE_C, ctor elder_vicaie_new, encoded false;
    c {
        is_invulnerable: |_w, _id| true,
        default_name: |_w, _id| Some("Elder Vicaie"),
    }
    m {
    }
}

fn elder_jothan_new(w: &mut World) -> MobileId {
    let id = new_creature(w, &ELDER_JOTHAN, AIType::AiVendor, FightMode::None, 2, 1);
    set_title(w, id, Some("the wise".to_string()));
    set_race_named(w, id, "Elf");
    set_body(w, id, Body(0x25D));
    set_female(w, id, false);
    let v1 = race_skin_hue(w, id);
    set_hue(w, id, v1);
    set_speed(w, id, 0.5, 2.0, true);
    init_stats(w, id, 100, 100, 25);
    assign_random_hair(w, id);
    add_item_hue(w, id, "ThighBoots", None);
    add_item_hue(w, id, "ElvenPants", Some(0x58D));
    let v2 = utility::random_yellow_hue(&mut w.rng);
    add_item_hue(w, id, "ElvenShirt", Some(v2));
    let v3 = utility::random_bright_hue(&mut w.rng);
    add_item_hue(w, id, "Cloak", Some(v3));
    add_item_hue(w, id, "Circlet", None);
    id
}
creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.ElderJothan`.
    ELDER_JOTHAN, ELDER_JOTHAN_C = "Server.Engines.MLQuests.Definitions.ElderJothan" : BASE_CREATURE / BASE_CREATURE_C, ctor elder_jothan_new, encoded false;
    c {
        is_invulnerable: |_w, _id| true,
        default_name: |_w, _id| Some("Elder Jothan"),
    }
    m {
    }
}

fn elder_alethanian_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &ELDER_ALETHANIAN,
        AIType::AiVendor,
        FightMode::None,
        2,
        1,
    );
    set_title(w, id, Some("the wise".to_string()));
    set_race_named(w, id, "Elf");
    set_body(w, id, Body(0x25E));
    set_female(w, id, true);
    let v1 = race_skin_hue(w, id);
    set_hue(w, id, v1);
    set_speed(w, id, 0.5, 2.0, true);
    init_stats(w, id, 100, 100, 25);
    assign_random_hair(w, id);
    add_item_hue(w, id, "ElvenBoots", None);
    add_item_hue(w, id, "HidePants", None);
    add_item_hue(w, id, "HideFemaleChest", None);
    add_item_hue(w, id, "HidePauldrons", None);
    add_item_hue(w, id, "GemmedCirclet", None);
    id
}
creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.ElderAlethanian`.
    ELDER_ALETHANIAN, ELDER_ALETHANIAN_C = "Server.Engines.MLQuests.Definitions.ElderAlethanian" : BASE_CREATURE / BASE_CREATURE_C, ctor elder_alethanian_new, encoded false;
    c {
        is_invulnerable: |_w, _id| true,
        default_name: |_w, _id| Some("Elder Alethanian"),
    }
    m {
    }
}

fn rebinil_new(w: &mut World) -> MobileId {
    let id = new_creature(w, &REBINIL, AIType::AiVendor, FightMode::None, 2, 1);
    set_title(w, id, Some("the healer".to_string()));
    set_race_named(w, id, "Elf");
    set_body(w, id, Body(0x25E));
    set_female(w, id, true);
    let v1 = race_skin_hue(w, id);
    set_hue(w, id, v1);
    set_speed(w, id, 0.5, 2.0, true);
    init_stats(w, id, 100, 100, 25);
    assign_random_hair(w, id);
    add_item_hue(w, id, "Sandals", Some(0x715));
    add_item_hue(w, id, "FemaleElvenRobe", Some(0x742));
    add_item_hue(w, id, "RoyalCirclet", None);
    id
}
creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Rebinil`.
    REBINIL, REBINIL_C = "Server.Engines.MLQuests.Definitions.Rebinil" : BASE_CREATURE / BASE_CREATURE_C, ctor rebinil_new, encoded false;
    c {
        is_invulnerable: |_w, _id| true,
        default_name: |_w, _id| Some("Rebinil"),
    }
    m {
    }
}

fn aluniol_new(w: &mut World) -> MobileId {
    let id = new_creature(w, &ALUNIOL, AIType::AiVendor, FightMode::None, 2, 1);
    set_title(w, id, Some("the healer".to_string()));
    set_race_named(w, id, "Elf");
    set_body(w, id, Body(0x25D));
    set_female(w, id, false);
    let v1 = race_skin_hue(w, id);
    set_hue(w, id, v1);
    set_speed(w, id, 0.5, 2.0, true);
    init_stats(w, id, 100, 100, 25);
    assign_random_hair(w, id);
    add_item_hue(w, id, "ElvenBoots", Some(0x1BB));
    add_item_hue(w, id, "MaleElvenRobe", Some(0x47E));
    add_item_hue(w, id, "WildStaff", None);
    id
}
creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Aluniol`.
    ALUNIOL, ALUNIOL_C = "Server.Engines.MLQuests.Definitions.Aluniol" : BASE_CREATURE / BASE_CREATURE_C, ctor aluniol_new, encoded false;
    c {
        is_invulnerable: |_w, _id| true,
        default_name: |_w, _id| Some("Aluniol"),
    }
    m {
    }
}

fn olaeni_new(w: &mut World) -> MobileId {
    let id = new_creature(w, &OLAENI, AIType::AiVendor, FightMode::None, 2, 1);
    set_title(w, id, Some("the thaumaturgist".to_string()));
    set_race_named(w, id, "Elf");
    set_body(w, id, Body(0x25E));
    set_female(w, id, true);
    let v1 = race_skin_hue(w, id);
    set_hue(w, id, v1);
    set_speed(w, id, 0.5, 2.0, true);
    init_stats(w, id, 100, 100, 25);
    assign_random_hair(w, id);
    add_item_hue(w, id, "Shoes", Some(0x75A));
    add_item_hue(w, id, "FemaleElvenRobe", Some(0x13));
    add_item_hue(w, id, "MagicWand", None);
    add_item_hue(w, id, "GemmedCirclet", None);
    id
}
creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Olaeni`.
    OLAENI, OLAENI_C = "Server.Engines.MLQuests.Definitions.Olaeni" : BASE_CREATURE / BASE_CREATURE_C, ctor olaeni_new, encoded false;
    c {
        is_invulnerable: |_w, _id| true,
        default_name: |_w, _id| Some("Olaeni"),
    }
    m {
    }
}

fn bolaevin_new(w: &mut World) -> MobileId {
    let id = new_creature(w, &BOLAEVIN, AIType::AiVendor, FightMode::None, 2, 1);
    set_title(w, id, Some("the arcanist".to_string()));
    set_race_named(w, id, "Elf");
    set_body(w, id, Body(0x25D));
    set_female(w, id, false);
    let v1 = race_skin_hue(w, id);
    set_hue(w, id, v1);
    set_speed(w, id, 0.5, 2.0, true);
    init_stats(w, id, 100, 100, 25);
    assign_random_hair(w, id);
    add_item_hue(w, id, "ElvenBoots", Some(0x3B2));
    add_item_hue(w, id, "RoyalCirclet", None);
    add_item_hue(w, id, "LeafChest", None);
    add_item_hue(w, id, "LeafArms", None);
    add_item_props(w, id, "LeafLegs", None, Some(0x1BB));
    id
}
creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Bolaevin`.
    BOLAEVIN, BOLAEVIN_C = "Server.Engines.MLQuests.Definitions.Bolaevin" : BASE_CREATURE / BASE_CREATURE_C, ctor bolaevin_new, encoded false;
    c {
        is_invulnerable: |_w, _id| true,
        default_name: |_w, _id| Some("Bolaevin"),
    }
    m {
    }
}

fn lorekeeper_aneen_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &LOREKEEPER_ANEEN,
        AIType::AiVendor,
        FightMode::None,
        2,
        1,
    );
    set_title(w, id, Some("the keeper of tradition".to_string()));
    set_race_named(w, id, "Elf");
    set_body(w, id, Body(0x25D));
    set_female(w, id, false);
    let v1 = race_skin_hue(w, id);
    set_hue(w, id, v1);
    set_speed(w, id, 0.5, 2.0, true);
    init_stats(w, id, 100, 100, 25);
    assign_random_hair(w, id);
    add_item_hue(w, id, "Sandals", Some(0x1BB));
    add_item_hue(w, id, "MaleElvenRobe", Some(0x48F));
    add_item_hue(w, id, "MagicWand", None);
    id
}
creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.LorekeeperAneen`.
    LOREKEEPER_ANEEN, LOREKEEPER_ANEEN_C = "Server.Engines.MLQuests.Definitions.LorekeeperAneen" : BASE_CREATURE / BASE_CREATURE_C, ctor lorekeeper_aneen_new, encoded false;
    c {
        is_invulnerable: |_w, _id| true,
        default_name: |_w, _id| Some("Lorekeeper Aneen"),
    }
    m {
    }
}

fn daelas_new(w: &mut World) -> MobileId {
    let id = new_creature(w, &DAELAS, AIType::AiVendor, FightMode::None, 2, 1);
    set_title(w, id, Some("the arborist".to_string()));
    set_race_named(w, id, "Elf");
    set_body(w, id, Body(0x25D));
    set_female(w, id, false);
    let v1 = race_skin_hue(w, id);
    set_hue(w, id, v1);
    set_speed(w, id, 0.5, 2.0, true);
    init_stats(w, id, 100, 100, 25);
    assign_random_hair(w, id);
    add_item_hue(w, id, "ElvenBoots", Some(0x901));
    add_item_hue(w, id, "ElvenPants", Some(0x8AB));
    let item = make_named(w, "LeafChest");
    set_item_hue(w, item, 0x8B0);
    add_item_id(w, id, item);
    let item = make_named(w, "LeafGloves");
    set_item_hue(w, item, 0x1BB);
    add_item_id(w, id, item);
    id
}
creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Daelas`.
    DAELAS, DAELAS_C = "Server.Engines.MLQuests.Definitions.Daelas" : BASE_CREATURE / BASE_CREATURE_C, ctor daelas_new, encoded false;
    c {
        is_invulnerable: |_w, _id| true,
        default_name: |_w, _id| Some("Daelas"),
    }
    m {
    }
}

fn alelle_new(w: &mut World) -> MobileId {
    let id = new_creature(w, &ALELLE, AIType::AiVendor, FightMode::None, 2, 1);
    set_title(w, id, Some("the arborist".to_string()));
    set_race_named(w, id, "Elf");
    set_body(w, id, Body(0x25E));
    set_female(w, id, true);
    let v1 = race_skin_hue(w, id);
    set_hue(w, id, v1);
    set_speed(w, id, 0.5, 2.0, true);
    init_stats(w, id, 100, 100, 25);
    assign_random_hair(w, id);
    add_item_hue(w, id, "ElvenBoots", Some(0x1BB));
    let item = make_named(w, "FemaleLeafChest");
    set_item_hue(w, item, 0x3A);
    add_item_id(w, id, item);
    let item = make_named(w, "LeafLegs");
    set_item_hue(w, item, 0x74C);
    add_item_id(w, id, item);
    let item = make_named(w, "LeafGloves");
    set_item_hue(w, item, 0x1BB);
    add_item_id(w, id, item);
    id
}
creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Alelle`.
    ALELLE, ALELLE_C = "Server.Engines.MLQuests.Definitions.Alelle" : BASE_CREATURE / BASE_CREATURE_C, ctor alelle_new, encoded false;
    c {
        is_invulnerable: |_w, _id| true,
        default_name: |_w, _id| Some("Alelle"),
    }
    m {
    }
}

fn lorekeeper_nillaen_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &LOREKEEPER_NILLAEN,
        AIType::AiVendor,
        FightMode::None,
        2,
        1,
    );
    set_title(w, id, Some("the keeper of tradition".to_string()));
    set_race_named(w, id, "Elf");
    set_body(w, id, Body(0x25E));
    set_female(w, id, true);
    let v1 = race_skin_hue(w, id);
    set_hue(w, id, v1);
    set_speed(w, id, 0.5, 2.0, true);
    init_stats(w, id, 100, 100, 25);
    assign_random_hair(w, id);
    set_skill_range(w, id, SkillName::Meditation, 60.0, 80.0);
    set_skill_range(w, id, SkillName::Focus, 60.0, 80.0);
    add_item_hue(w, id, "Shoes", Some(0x1BB));
    add_item_hue(w, id, "LongPants", Some(0x1FB));
    add_item_hue(w, id, "ElvenShirt", None);
    add_item_hue(w, id, "GemmedCirclet", None);
    add_item_hue(w, id, "BodySash", Some(0x25));
    add_item_hue(w, id, "BlackStaff", None);
    id
}
creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.LorekeeperNillaen`.
    LOREKEEPER_NILLAEN, LOREKEEPER_NILLAEN_C = "Server.Engines.MLQuests.Definitions.LorekeeperNillaen" : BASE_CREATURE / BASE_CREATURE_C, ctor lorekeeper_nillaen_new, encoded false;
    c {
        is_invulnerable: |_w, _id| true,
        default_name: |_w, _id| Some("Lorekeeper Nillaen"),
        can_teach: |_w, _id| true,
    }
    m {
    }
}

fn lorekeeper_ryal_new(w: &mut World) -> MobileId {
    let id = new_creature(w, &LOREKEEPER_RYAL, AIType::AiVendor, FightMode::None, 2, 1);
    set_title(w, id, Some("the keeper of tradition".to_string()));
    set_race_named(w, id, "Elf");
    set_body(w, id, Body(0x25D));
    set_female(w, id, false);
    let v1 = race_skin_hue(w, id);
    set_hue(w, id, v1);
    set_speed(w, id, 0.5, 2.0, true);
    init_stats(w, id, 100, 100, 25);
    assign_random_hair(w, id);
    set_skill_range(w, id, SkillName::Meditation, 60.0, 80.0);
    set_skill_range(w, id, SkillName::Focus, 60.0, 80.0);
    add_item_hue(w, id, "ElvenBoots", Some(0x1BB));
    add_item_hue(w, id, "LeafTonlet", None);
    add_item_hue(w, id, "ElvenShirt", Some(0x2DD));
    add_item_hue(w, id, "Cloak", Some(0x219));
    add_item_hue(w, id, "GnarledStaff", None);
    id
}
creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.LorekeeperRyal`.
    LOREKEEPER_RYAL, LOREKEEPER_RYAL_C = "Server.Engines.MLQuests.Definitions.LorekeeperRyal" : BASE_CREATURE / BASE_CREATURE_C, ctor lorekeeper_ryal_new, encoded false;
    c {
        is_invulnerable: |_w, _id| true,
        default_name: |_w, _id| Some("Lorekeeper Ryal"),
        can_teach: |_w, _id| true,
        can_shout: |_w, _id| true,
    }
    m {
    }
}
fn lorekeeper_ryal_shout(w: &mut World, id: MobileId, pm: MobileId) {
    let v1 = rnd::bool(w);
    let v2 = if v1 { 1074204 } else { 1074200 };
    quester::tell(w, Quester::Mobile(id), pm, v2);
}

fn braen_new(w: &mut World) -> MobileId {
    let id = new_creature(w, &BRAEN, AIType::AiVendor, FightMode::None, 2, 1);
    set_title(w, id, Some("the thaumaturgist".to_string()));
    set_race_named(w, id, "Elf");
    set_body(w, id, Body(0x25D));
    set_female(w, id, false);
    let v1 = race_skin_hue(w, id);
    set_hue(w, id, v1);
    set_speed(w, id, 0.5, 2.0, true);
    init_stats(w, id, 100, 100, 25);
    assign_random_hair(w, id);
    set_skill_range(w, id, SkillName::Meditation, 60.0, 80.0);
    set_skill_range(w, id, SkillName::Focus, 60.0, 80.0);
    add_item_hue(w, id, "Sandals", Some(0x714));
    add_item_hue(w, id, "MaleElvenRobe", Some(0x64A));
    add_item_hue(w, id, "MagicWand", None);
    id
}
creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Braen`.
    BRAEN, BRAEN_C = "Server.Engines.MLQuests.Definitions.Braen" : BASE_CREATURE / BASE_CREATURE_C, ctor braen_new, encoded false;
    c {
        is_invulnerable: |_w, _id| true,
        default_name: |_w, _id| Some("Braen"),
        can_teach: |_w, _id| true,
        can_shout: |_w, _id| true,
    }
    m {
    }
}
fn braen_shout(w: &mut World, id: MobileId, pm: MobileId) {
    quester::tell(w, Quester::Mobile(id), pm, 1074187);
}

fn elder_acob_new(w: &mut World) -> MobileId {
    let id = new_creature(w, &ELDER_ACOB, AIType::AiVendor, FightMode::None, 2, 1);
    set_title(w, id, Some("the wise".to_string()));
    set_race_named(w, id, "Elf");
    set_body(w, id, Body(0x25D));
    set_female(w, id, false);
    let v1 = race_skin_hue(w, id);
    set_hue(w, id, v1);
    set_speed(w, id, 0.5, 2.0, true);
    init_stats(w, id, 100, 100, 25);
    assign_random_hair(w, id);
    set_skill_range(w, id, SkillName::Meditation, 60.0, 80.0);
    set_skill_range(w, id, SkillName::Focus, 60.0, 80.0);
    add_item_hue(w, id, "ElvenBoots", Some(0x714));
    let v2 = utility::random_bright_hue(&mut w.rng);
    add_item_hue(w, id, "ElvenShirt", Some(v2));
    add_item_hue(w, id, "HidePants", None);
    id
}
creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.ElderAcob`.
    ELDER_ACOB, ELDER_ACOB_C = "Server.Engines.MLQuests.Definitions.ElderAcob" : BASE_CREATURE / BASE_CREATURE_C, ctor elder_acob_new, encoded false;
    c {
        is_invulnerable: |_w, _id| true,
        default_name: |_w, _id| Some("Elder Acob"),
        can_teach: |_w, _id| true,
        can_shout: |_w, _id| true,
    }
    m {
    }
}
fn elder_acob_shout(w: &mut World, id: MobileId, pm: MobileId) {
    quester::tell(w, Quester::Mobile(id), pm, 1074197);
}

fn lorekeeper_calendor_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &LOREKEEPER_CALENDOR,
        AIType::AiVendor,
        FightMode::None,
        2,
        1,
    );
    set_title(w, id, Some("the keeper of tradition".to_string()));
    set_race_named(w, id, "Elf");
    set_body(w, id, Body(0x25E));
    set_female(w, id, true);
    let v1 = race_skin_hue(w, id);
    set_hue(w, id, v1);
    set_speed(w, id, 0.5, 2.0, true);
    init_stats(w, id, 100, 100, 25);
    assign_random_hair(w, id);
    set_skill_range(w, id, SkillName::Meditation, 60.0, 80.0);
    set_skill_range(w, id, SkillName::Focus, 60.0, 80.0);
    add_item_hue(w, id, "ElvenBoots", Some(0x714));
    let v2 = utility::random_orange_hue(&mut w.rng);
    add_item_hue(w, id, "ElvenShirt", Some(v2));
    let v3 = utility::random_orange_hue(&mut w.rng);
    add_item_hue(w, id, "Kilt", Some(v3));
    add_item_hue(w, id, "RoyalCirclet", None);
    id
}
creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.LorekeeperCalendor`.
    LOREKEEPER_CALENDOR, LOREKEEPER_CALENDOR_C = "Server.Engines.MLQuests.Definitions.LorekeeperCalendor" : BASE_CREATURE / BASE_CREATURE_C, ctor lorekeeper_calendor_new, encoded false;
    c {
        is_invulnerable: |_w, _id| true,
        default_name: |_w, _id| Some("Lorekeeper Calendor"),
        can_teach: |_w, _id| true,
        can_shout: |_w, _id| true,
    }
    m {
    }
}
fn lorekeeper_calendor_shout(w: &mut World, id: MobileId, pm: MobileId) {
    quester::tell(w, Quester::Mobile(id), pm, 1074204);
}

fn lorekeeper_siarra_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &LOREKEEPER_SIARRA,
        AIType::AiVendor,
        FightMode::None,
        2,
        1,
    );
    set_title(w, id, Some("the keeper of tradition".to_string()));
    set_race_named(w, id, "Elf");
    set_body(w, id, Body(0x25E));
    set_female(w, id, true);
    let v1 = race_skin_hue(w, id);
    set_hue(w, id, v1);
    set_speed(w, id, 0.5, 2.0, true);
    init_stats(w, id, 100, 100, 25);
    assign_random_hair(w, id);
    set_skill_range(w, id, SkillName::Meditation, 60.0, 80.0);
    set_skill_range(w, id, SkillName::Focus, 60.0, 80.0);
    add_item_hue(w, id, "Sandals", Some(0x1BB));
    add_item_hue(w, id, "ElvenShirt", None);
    add_item_hue(w, id, "LeafTonlet", None);
    add_item_hue(w, id, "GemmedCirclet", None);
    id
}
creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.LorekeeperSiarra`.
    LOREKEEPER_SIARRA, LOREKEEPER_SIARRA_C = "Server.Engines.MLQuests.Definitions.LorekeeperSiarra" : BASE_CREATURE / BASE_CREATURE_C, ctor lorekeeper_siarra_new, encoded false;
    c {
        is_invulnerable: |_w, _id| true,
        default_name: |_w, _id| Some("Lorekeeper Siarra"),
        can_teach: |_w, _id| true,
        can_shout: |_w, _id| true,
    }
    m {
    }
}
fn lorekeeper_siarra_shout(w: &mut World, id: MobileId, pm: MobileId) {
    quester::tell(w, Quester::Mobile(id), pm, 1074206);
}

fn enigma_new(w: &mut World) -> MobileId {
    let id = new_creature(w, &ENIGMA, AIType::AiVendor, FightMode::None, 2, 1);
    set_body(w, id, Body(788));
    base_sound_id(w, id, 0x3EE);
    set_speed(w, id, 0.5, 2.0, true);
    init_stats(w, id, 100, 100, 25);
    id
}
creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Enigma`.
    ENIGMA, ENIGMA_C = "Server.Engines.MLQuests.Definitions.Enigma" : BASE_CREATURE / BASE_CREATURE_C, ctor enigma_new, encoded false;
    c {
        is_invulnerable: |_w, _id| true,
        default_name: |_w, _id| Some("Enigma"),
    }
    m {
    }
}

fn sledge_new(w: &mut World) -> MobileId {
    let id = new_creature(w, &SLEDGE, AIType::AiVendor, FightMode::None, 2, 1);
    set_title(w, id, Some("the Versatile".to_string()));
    set_body(w, id, Body(400));
    let v1 = race_skin_hue(w, id);
    set_hue(w, id, v1);
    set_speed(w, id, 0.5, 2.0, true);
    init_stats(w, id, 100, 100, 25);
    let v2 = utility::random_neutral_hue(&mut w.rng);
    add_item_hue(w, id, "Tunic", Some(v2));
    let v3 = utility::random_blue_hue(&mut w.rng);
    add_item_hue(w, id, "LongPants", Some(v3));
    let v4 = utility::random_bright_hue(&mut w.rng);
    add_item_hue(w, id, "Cloak", Some(v4));
    let v5 = utility::random_neutral_hue(&mut w.rng);
    add_item_hue(w, id, "ElvenBoots", Some(v5));
    add_item_hue(w, id, "Backpack", None);
    id
}
creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Sledge`.
    SLEDGE, SLEDGE_C = "Server.Engines.MLQuests.Definitions.Sledge" : BASE_CREATURE / BASE_CREATURE_C, ctor sledge_new, encoded false;
    c {
        is_invulnerable: |_w, _id| true,
        default_name: |_w, _id| Some("Sledge"),
        can_shout: |_w, _id| true,
    }
    m {
    }
}
fn sledge_shout(w: &mut World, id: MobileId, pm: MobileId) {
    let v1 = [1074188, 1074195][rnd::below(w, 2) as usize];
    quester::tell(w, Quester::Mobile(id), pm, v1);
}

fn patricus_new(w: &mut World) -> MobileId {
    let id = new_creature(w, &PATRICUS, AIType::AiVendor, FightMode::None, 2, 1);
    set_title(w, id, Some("the Trader".to_string()));
    set_body(w, id, Body(400));
    let v1 = race_skin_hue(w, id);
    set_hue(w, id, v1);
    set_speed(w, id, 0.5, 2.0, true);
    init_stats(w, id, 100, 100, 25);
    let v2 = utility::random_neutral_hue(&mut w.rng);
    add_item_hue(w, id, "FancyShirt", Some(v2));
    let v3 = utility::random_bright_hue(&mut w.rng);
    add_item_hue(w, id, "LongPants", Some(v3));
    add_item_hue(w, id, "Cloak", Some(0x1BB));
    let v4 = utility::random_neutral_hue(&mut w.rng);
    add_item_hue(w, id, "Shoes", Some(v4));
    add_item_hue(w, id, "Backpack", None);
    id
}
creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Patricus`.
    PATRICUS, PATRICUS_C = "Server.Engines.MLQuests.Definitions.Patricus" : BASE_CREATURE / BASE_CREATURE_C, ctor patricus_new, encoded false;
    c {
        is_invulnerable: |_w, _id| true,
        default_name: |_w, _id| Some("Patricus"),
    }
    m {
    }
}

fn belulah_new(w: &mut World) -> MobileId {
    let id = new_creature(w, &BELULAH, AIType::AiVendor, FightMode::None, 2, 1);
    set_title(w, id, Some("the scorned".to_string()));
    set_female(w, id, true);
    set_body(w, id, Body(401));
    let v1 = race_skin_hue(w, id);
    set_hue(w, id, v1);
    set_speed(w, id, 0.5, 2.0, true);
    init_stats(w, id, 100, 100, 25);
    assign_random_hair(w, id);
    let v2 = utility::random_blue_hue(&mut w.rng);
    add_item_hue(w, id, "FancyShirt", Some(v2));
    let v3 = utility::random_nondyed_hue(&mut w.rng);
    add_item_hue(w, id, "LongPants", Some(v3));
    add_item_hue(w, id, "Boots", None);
    id
}
creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Belulah`.
    BELULAH, BELULAH_C = "Server.Engines.MLQuests.Definitions.Belulah" : BASE_CREATURE / BASE_CREATURE_C, ctor belulah_new, encoded false;
    c {
        is_invulnerable: |_w, _id| true,
        default_name: |_w, _id| Some("Belulah"),
        can_shout: |_w, _id| true,
    }
    m {
    }
}
fn belulah_shout(w: &mut World, id: MobileId, pm: MobileId) {
    let v1 = rnd::from(w, 1074205, 2);
    quester::tell(w, Quester::Mobile(id), pm, v1);
}

fn evan_new(w: &mut World) -> MobileId {
    let id = new_creature(w, &EVAN, AIType::AiVendor, FightMode::None, 2, 1);
    set_title(w, id, Some("the Beggar".to_string()));
    set_race_named(w, id, "Human");
    set_body(w, id, Body(0x190));
    set_female(w, id, false);
    let v1 = race_skin_hue(w, id);
    set_hue(w, id, v1);
    set_speed(w, id, 0.5, 2.0, true);
    init_stats(w, id, 100, 100, 25);
    assign_random_hair(w, id);
    add_item_hue(w, id, "Backpack", None);
    add_item_hue(w, id, "Doublet", None);
    add_item_hue(w, id, "ShortPants", Some(0x755));
    id
}
creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Evan`.
    EVAN, EVAN_C = "Server.Engines.MLQuests.Definitions.Evan" : BASE_CREATURE / BASE_CREATURE_C, ctor evan_new, encoded false;
    c {
        is_invulnerable: |_w, _id| true,
        default_name: |_w, _id| Some("Evan"),
    }
    m {
    }
}

fn regina_new(w: &mut World) -> MobileId {
    let id = new_creature(w, &REGINA, AIType::AiVendor, FightMode::None, 2, 1);
    set_title(w, id, Some("the Noble".to_string()));
    set_race_named(w, id, "Human");
    set_body(w, id, Body(0x191));
    set_female(w, id, true);
    let v1 = race_skin_hue(w, id);
    set_hue(w, id, v1);
    set_speed(w, id, 0.5, 2.0, true);
    init_stats(w, id, 100, 100, 25);
    assign_random_hair(w, id);
    add_item_hue(w, id, "Backpack", None);
    add_item_hue(w, id, "GildedDress", None);
    add_item_hue(w, id, "Boots", None);
    id
}
creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Regina`.
    REGINA, REGINA_C = "Server.Engines.MLQuests.Definitions.Regina" : BASE_CREATURE / BASE_CREATURE_C, ctor regina_new, encoded false;
    c {
        is_invulnerable: |_w, _id| true,
        default_name: |_w, _id| Some("Regina"),
    }
    m {
    }
}

fn grandpa_charley_new(w: &mut World) -> MobileId {
    let id = new_creature(w, &GRANDPA_CHARLEY, AIType::AiVendor, FightMode::None, 2, 1);
    set_title(w, id, Some("the farmer".to_string()));
    set_body(w, id, Body(400));
    let v1 = race_skin_hue(w, id);
    set_hue(w, id, v1);
    set_speed(w, id, 0.5, 2.0, true);
    init_stats(w, id, 100, 100, 25);
    let v2 = rnd::below(w, 2);
    let v3 = 0x3B2 + v2;
    assign_random_hair_hue(w, id, v3);
    set_facial_hair_item_id(w, id, 0x203E);
    set_facial_hair_hue(w, id, v3);
    set_skill_range(w, id, SkillName::ItemID, 80.0, 90.0);
    let v4 = utility::random_nondyed_hue(&mut w.rng);
    add_item_hue(w, id, "WideBrimHat", Some(v4));
    let v5 = utility::random_nondyed_hue(&mut w.rng);
    add_item_hue(w, id, "FancyShirt", Some(v5));
    let v6 = utility::random_nondyed_hue(&mut w.rng);
    add_item_hue(w, id, "LongPants", Some(v6));
    let v7 = utility::random_neutral_hue(&mut w.rng);
    add_item_hue(w, id, "Sandals", Some(v7));
    add_item_hue(w, id, "ShepherdsCrook", None);
    add_item_hue(w, id, "Backpack", None);
    id
}
creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.GrandpaCharley`.
    GRANDPA_CHARLEY, GRANDPA_CHARLEY_C = "Server.Engines.MLQuests.Definitions.GrandpaCharley" : BASE_CREATURE / BASE_CREATURE_C, ctor grandpa_charley_new, encoded false;
    c {
        is_invulnerable: |_w, _id| true,
        default_name: |_w, _id| Some("Grandpa Charley"),
        can_teach: |_w, _id| true,
    }
    m {
    }
}

fn jelrice_new(w: &mut World) -> MobileId {
    let id = new_creature(w, &JELRICE, AIType::AiVendor, FightMode::None, 2, 1);
    set_title(w, id, Some("the trader".to_string()));
    set_race_named(w, id, "Human");
    set_body(w, id, Body(0x191));
    set_female(w, id, true);
    let v1 = race_skin_hue(w, id);
    set_hue(w, id, v1);
    set_speed(w, id, 0.5, 2.0, true);
    init_stats(w, id, 100, 100, 25);
    assign_random_hair(w, id);
    let v2 = utility::random_neutral_hue(&mut w.rng);
    add_item_hue(w, id, "Shoes", Some(v2));
    let v3 = utility::random_blue_hue(&mut w.rng);
    add_item_hue(w, id, "Skirt", Some(v3));
    let v4 = utility::random_red_hue(&mut w.rng);
    add_item_hue(w, id, "FancyShirt", Some(v4));
    id
}
creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Jelrice`.
    JELRICE, JELRICE_C = "Server.Engines.MLQuests.Definitions.Jelrice" : BASE_CREATURE / BASE_CREATURE_C, ctor jelrice_new, encoded false;
    c {
        is_invulnerable: |_w, _id| true,
        default_name: |_w, _id| Some("Jelrice"),
        can_shout: |_w, _id| true,
    }
    m {
    }
}
fn jelrice_shout(w: &mut World, id: MobileId, pm: MobileId) {
    quester::tell(w, Quester::Mobile(id), pm, 1074221);
}

fn yorus_new(w: &mut World) -> MobileId {
    let id = new_creature(w, &YORUS, AIType::AiVendor, FightMode::None, 2, 1);
    set_title(w, id, Some("the tinker".to_string()));
    set_race_named(w, id, "Human");
    set_body(w, id, Body(0x190));
    set_female(w, id, false);
    let v1 = race_skin_hue(w, id);
    set_hue(w, id, v1);
    set_speed(w, id, 0.5, 2.0, true);
    init_stats(w, id, 100, 100, 25);
    assign_random_hair(w, id);
    let v2 = utility::random_neutral_hue(&mut w.rng);
    add_item_hue(w, id, "Shoes", Some(v2));
    let v3 = utility::random_blue_hue(&mut w.rng);
    add_item_hue(w, id, "LongPants", Some(v3));
    let v4 = utility::random_orange_hue(&mut w.rng);
    add_item_hue(w, id, "FancyShirt", Some(v4));
    let v5 = utility::random_bright_hue(&mut w.rng);
    add_item_hue(w, id, "Cloak", Some(v5));
    id
}
creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Yorus`.
    YORUS, YORUS_C = "Server.Engines.MLQuests.Definitions.Yorus" : BASE_CREATURE / BASE_CREATURE_C, ctor yorus_new, encoded false;
    c {
        is_invulnerable: |_w, _id| true,
        default_name: |_w, _id| Some("Yorus"),
        can_shout: |_w, _id| true,
    }
    m {
    }
}
fn yorus_shout(w: &mut World, id: MobileId, pm: MobileId) {
    quester::tell(w, Quester::Mobile(id), pm, 1074218);
}

fn drithen_new(w: &mut World) -> MobileId {
    let id = new_creature(w, &DRITHEN, AIType::AiVendor, FightMode::None, 2, 1);
    set_title(w, id, Some("the Fierce".to_string()));
    set_race_named(w, id, "Human");
    set_body(w, id, Body(0x190));
    set_female(w, id, false);
    let v1 = race_skin_hue(w, id);
    set_hue(w, id, v1);
    set_speed(w, id, 0.5, 2.0, true);
    init_stats(w, id, 100, 100, 25);
    add_item_hue(w, id, "Backpack", None);
    let v2 = utility::random_neutral_hue(&mut w.rng);
    add_item_hue(w, id, "ElvenBoots", Some(v2));
    let v3 = utility::random_blue_hue(&mut w.rng);
    add_item_hue(w, id, "LongPants", Some(v3));
    let v4 = utility::random_neutral_hue(&mut w.rng);
    add_item_hue(w, id, "Tunic", Some(v4));
    let v5 = utility::random_bright_hue(&mut w.rng);
    add_item_hue(w, id, "Cloak", Some(v5));
    set_skill_range(w, id, SkillName::Focus, 60.0, 80.0);
    id
}
creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Drithen`.
    DRITHEN, DRITHEN_C = "Server.Engines.MLQuests.Definitions.Drithen" : BASE_CREATURE / BASE_CREATURE_C, ctor drithen_new, encoded false;
    c {
        is_invulnerable: |_w, _id| true,
        can_teach: |_w, _id| true,
        default_name: |_w, _id| Some("Drithen"),
        can_shout: |_w, _id| true,
    }
    m {
    }
}
fn drithen_shout(w: &mut World, id: MobileId, pm: MobileId) {
    quester::tell(w, Quester::Mobile(id), pm, 1074188);
}

fn aernya_new(w: &mut World) -> MobileId {
    let id = new_creature(w, &AERNYA, AIType::AiVendor, FightMode::None, 2, 1);
    set_title(w, id, Some("the Mistress of Admissions".to_string()));
    set_race_named(w, id, "Human");
    set_body(w, id, Body(0x191));
    set_female(w, id, true);
    let v1 = race_skin_hue(w, id);
    set_hue(w, id, v1);
    set_speed(w, id, 0.5, 2.0, true);
    init_stats(w, id, 100, 100, 25);
    assign_random_hair(w, id);
    add_item_hue(w, id, "Backpack", None);
    let v2 = utility::random_neutral_hue(&mut w.rng);
    add_item_hue(w, id, "Sandals", Some(v2));
    let v3 = rnd::bool(w);
    let v4 = if v3 { 0x1 } else { 0x0 };
    add_item_hue(w, id, "Skirt", Some(v4));
    let v5 = utility::random_bright_hue(&mut w.rng);
    add_item_hue(w, id, "Cloak", Some(v5));
    let v6 = rnd::bool(w);
    let v7 = if v6 { 0x3B2 } else { 0x3B3 };
    add_item_hue(w, id, "FancyShirt", Some(v7));
    id
}
creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Aernya`.
    AERNYA, AERNYA_C = "Server.Engines.MLQuests.Definitions.Aernya" : BASE_CREATURE / BASE_CREATURE_C, ctor aernya_new, encoded false;
    c {
        is_invulnerable: |_w, _id| true,
        default_name: |_w, _id| Some("Aernya"),
    }
    m {
    }
}

fn gorrow_new(w: &mut World) -> MobileId {
    let id = new_creature(w, &GORROW, AIType::AiVendor, FightMode::None, 2, 1);
    set_title(w, id, Some("the Mayor".to_string()));
    set_race_named(w, id, "Human");
    set_body(w, id, Body(0x190));
    set_female(w, id, false);
    let v1 = race_skin_hue(w, id);
    set_hue(w, id, v1);
    set_speed(w, id, 0.5, 2.0, true);
    init_stats(w, id, 100, 100, 25);
    add_item_hue(w, id, "Backpack", None);
    add_item_hue(w, id, "Shoes", Some(0x1BB));
    let v2 = utility::random_neutral_hue(&mut w.rng);
    add_item_hue(w, id, "Tunic", Some(v2));
    add_item_hue(w, id, "LongPants", Some(0x901));
    let v3 = utility::random_red_hue(&mut w.rng);
    add_item_hue(w, id, "Cloak", Some(v3));
    id
}
creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Gorrow`.
    GORROW, GORROW_C = "Server.Engines.MLQuests.Definitions.Gorrow" : BASE_CREATURE / BASE_CREATURE_C, ctor gorrow_new, encoded false;
    c {
        is_invulnerable: |_w, _id| true,
        default_name: |_w, _id| Some("Gorrow"),
        can_shout: |_w, _id| true,
    }
    m {
    }
}
fn gorrow_shout(w: &mut World, id: MobileId, pm: MobileId) {
    let v1 = [1074200, 1074203][rnd::below(w, 2) as usize];
    quester::tell(w, Quester::Mobile(id), pm, v1);
}

fn master_gnosos_new(w: &mut World) -> MobileId {
    let id = new_creature(w, &MASTER_GNOSOS, AIType::AiVendor, FightMode::None, 2, 1);
    set_title(w, id, Some("the necromancer".to_string()));
    set_race_named(w, id, "Human");
    set_body(w, id, Body(0x190));
    set_female(w, id, false);
    set_hue(w, id, 0x83E8);
    set_speed(w, id, 0.5, 2.0, true);
    init_stats(w, id, 100, 100, 25);
    set_hair_item_id(w, id, 0x2049);
    set_facial_hair_item_id(w, id, 0x204B);
    add_item_hue(w, id, "Backpack", None);
    add_item_hue(w, id, "Shoes", Some(0x485));
    add_item_hue(w, id, "Robe", Some(0x497));
    set_skill_range(w, id, SkillName::EvalInt, 60.0, 80.0);
    set_skill_range(w, id, SkillName::Inscribe, 60.0, 80.0);
    set_skill_range(w, id, SkillName::MagicResist, 60.0, 80.0);
    set_skill_range(w, id, SkillName::SpiritSpeak, 60.0, 80.0);
    set_skill_range(w, id, SkillName::Meditation, 60.0, 80.0);
    set_skill_range(w, id, SkillName::Necromancy, 60.0, 80.0);
    id
}
creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.MasterGnosos`.
    MASTER_GNOSOS, MASTER_GNOSOS_C = "Server.Engines.MLQuests.Definitions.MasterGnosos" : BASE_CREATURE / BASE_CREATURE_C, ctor master_gnosos_new, encoded false;
    c {
        is_invulnerable: |_w, _id| true,
        can_teach: |_w, _id| true,
        default_name: |_w, _id| Some("Master Gnosos"),
        can_shout: |_w, _id| true,
    }
    m {
    }
}
fn master_gnosos_shout(w: &mut World, id: MobileId, pm: MobileId) {
    quester::tell(w, Quester::Mobile(id), pm, 1074186);
}

fn dimethro_new(w: &mut World) -> MobileId {
    let id = new_creature(w, &DIMETHRO, AIType::AiVendor, FightMode::None, 2, 1);
    set_title(w, id, Some("the Wrestling Instructor".to_string()));
    set_body(w, id, Body(0x190));
    set_hue(w, id, 0x83EA);
    set_hair_item_id(w, id, 0x203D);
    set_hair_hue(w, id, 0x455);
    set_facial_hair_item_id(w, id, 0x204D);
    set_facial_hair_hue(w, id, 0x455);
    set_speed(w, id, 0.5, 2.0, true);
    init_stats(w, id, 100, 100, 25);
    set_skill(w, id, SkillName::EvalInt, 120.0);
    set_skill(w, id, SkillName::Inscribe, 120.0);
    set_skill(w, id, SkillName::Magery, 120.0);
    set_skill(w, id, SkillName::MagicResist, 120.0);
    set_skill(w, id, SkillName::Wrestling, 120.0);
    set_skill(w, id, SkillName::Meditation, 120.0);
    add_item_hue(w, id, "Backpack", None);
    add_item_hue(w, id, "Sandals", Some(0x455));
    add_item_hue(w, id, "BodySash", Some(0x455));
    add_item_hue(w, id, "LongPants", Some(0x455));
    id
}
creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Dimethro`.
    DIMETHRO, DIMETHRO_C = "Server.Engines.MLQuests.Definitions.Dimethro" : BASE_CREATURE / BASE_CREATURE_C, ctor dimethro_new, encoded false;
    c {
        is_invulnerable: |_w, _id| true,
        can_teach: |_w, _id| true,
        default_name: |_w, _id| Some("Dimethro"),
        can_shout: |_w, _id| true,
    }
    m {
    }
}
fn dimethro_shout(w: &mut World, id: MobileId, pm: MobileId) {
    quester::tell(w, Quester::Mobile(id), pm, 1078128);
}

fn churchill_new(w: &mut World) -> MobileId {
    let id = new_creature(w, &CHURCHILL, AIType::AiVendor, FightMode::None, 2, 1);
    set_title(w, id, Some("the Mace Fighting Instructor".to_string()));
    set_body(w, id, Body(0x190));
    set_hue(w, id, 0x83EA);
    set_hair_item_id(w, id, 0x203C);
    set_hair_hue(w, id, 0x455);
    set_speed(w, id, 0.5, 2.0, true);
    init_stats(w, id, 100, 100, 25);
    set_skill(w, id, SkillName::Anatomy, 120.0);
    set_skill(w, id, SkillName::Parry, 120.0);
    set_skill(w, id, SkillName::Healing, 120.0);
    set_skill(w, id, SkillName::Tactics, 120.0);
    set_skill(w, id, SkillName::Macing, 120.0);
    set_skill(w, id, SkillName::Focus, 120.0);
    add_item_hue(w, id, "Backpack", None);
    add_item_hue(w, id, "OrderShield", None);
    add_item_hue(w, id, "WarMace", None);
    let item = make_named(w, "PlateLegs");
    set_item_hue(w, item, 0x966);
    add_item_id(w, id, item);
    let item = make_named(w, "PlateGloves");
    set_item_hue(w, item, 0x966);
    add_item_id(w, id, item);
    let item = make_named(w, "PlateGorget");
    set_item_hue(w, item, 0x966);
    add_item_id(w, id, item);
    let item = make_named(w, "PlateChest");
    set_item_hue(w, item, 0x966);
    add_item_id(w, id, item);
    let item = make_named(w, "PlateArms");
    set_item_hue(w, item, 0x966);
    add_item_id(w, id, item);
    id
}
creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Churchill`.
    CHURCHILL, CHURCHILL_C = "Server.Engines.MLQuests.Definitions.Churchill" : BASE_CREATURE / BASE_CREATURE_C, ctor churchill_new, encoded false;
    c {
        is_invulnerable: |_w, _id| true,
        can_teach: |_w, _id| true,
        default_name: |_w, _id| Some("Churchill"),
        can_shout: |_w, _id| true,
    }
    m {
    }
}
fn churchill_shout(w: &mut World, id: MobileId, pm: MobileId) {
    quester::tell(w, Quester::Mobile(id), pm, 1078141);
}

fn recaro_new(w: &mut World) -> MobileId {
    let id = new_creature(w, &RECARO, AIType::AiVendor, FightMode::None, 2, 1);
    set_title(w, id, Some("the Fencer Instructor".to_string()));
    set_body(w, id, Body(0x190));
    set_hue(w, id, 0x83EA);
    set_hair_item_id(w, id, 0x203C);
    set_hair_hue(w, id, 0x455);
    set_facial_hair_item_id(w, id, 0x204D);
    set_facial_hair_hue(w, id, 0x455);
    set_speed(w, id, 0.5, 2.0, true);
    init_stats(w, id, 100, 100, 25);
    set_skill(w, id, SkillName::Anatomy, 120.0);
    set_skill(w, id, SkillName::Parry, 120.0);
    set_skill(w, id, SkillName::Healing, 120.0);
    set_skill(w, id, SkillName::Tactics, 120.0);
    set_skill(w, id, SkillName::Fencing, 120.0);
    set_skill(w, id, SkillName::Focus, 120.0);
    add_item_hue(w, id, "Backpack", None);
    add_item_hue(w, id, "Shoes", Some(0x455));
    add_item_hue(w, id, "WarFork", None);
    let item = make_named(w, "StuddedLegs");
    set_item_hue(w, item, 0x455);
    add_item_id(w, id, item);
    let item = make_named(w, "StuddedGloves");
    set_item_hue(w, item, 0x455);
    add_item_id(w, id, item);
    let item = make_named(w, "StuddedGorget");
    set_item_hue(w, item, 0x455);
    add_item_id(w, id, item);
    let item = make_named(w, "StuddedChest");
    set_item_hue(w, item, 0x455);
    add_item_id(w, id, item);
    let item = make_named(w, "StuddedArms");
    set_item_hue(w, item, 0x455);
    add_item_id(w, id, item);
    id
}
creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Recaro`.
    RECARO, RECARO_C = "Server.Engines.MLQuests.Definitions.Recaro" : BASE_CREATURE / BASE_CREATURE_C, ctor recaro_new, encoded false;
    c {
        is_invulnerable: |_w, _id| true,
        can_teach: |_w, _id| true,
        default_name: |_w, _id| Some("Recaro"),
        can_shout: |_w, _id| true,
    }
    m {
    }
}
fn recaro_shout(w: &mut World, id: MobileId, pm: MobileId) {
    quester::tell(w, Quester::Mobile(id), pm, 1078187);
}

fn alden_armstrong_new(w: &mut World) -> MobileId {
    let id = new_creature(w, &ALDEN_ARMSTRONG, AIType::AiVendor, FightMode::None, 2, 1);
    set_title(w, id, Some("the Tactics Instructor".to_string()));
    set_body(w, id, Body(0x190));
    set_hue(w, id, 0x83EA);
    set_hair_item_id(w, id, 0x203B);
    set_hair_hue(w, id, 0x44E);
    set_speed(w, id, 0.5, 2.0, true);
    init_stats(w, id, 100, 100, 25);
    set_skill(w, id, SkillName::Anatomy, 120.0);
    set_skill(w, id, SkillName::Parry, 120.0);
    set_skill(w, id, SkillName::Healing, 120.0);
    set_skill(w, id, SkillName::Tactics, 120.0);
    set_skill(w, id, SkillName::Swords, 120.0);
    set_skill(w, id, SkillName::Focus, 120.0);
    add_item_hue(w, id, "Backpack", None);
    add_item_hue(w, id, "Shoes", None);
    add_item_hue(w, id, "StuddedLegs", None);
    add_item_hue(w, id, "StuddedGloves", None);
    add_item_hue(w, id, "StuddedGorget", None);
    add_item_hue(w, id, "StuddedChest", None);
    add_item_hue(w, id, "StuddedArms", None);
    add_item_hue(w, id, "Katana", None);
    id
}
creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.AldenArmstrong`.
    ALDEN_ARMSTRONG, ALDEN_ARMSTRONG_C = "Server.Engines.MLQuests.Definitions.AldenArmstrong" : BASE_CREATURE / BASE_CREATURE_C, ctor alden_armstrong_new, encoded false;
    c {
        is_invulnerable: |_w, _id| true,
        can_teach: |_w, _id| true,
        default_name: |_w, _id| Some("Alden Armstrong"),
        can_shout: |_w, _id| true,
    }
    m {
    }
}
fn alden_armstrong_shout(w: &mut World, id: MobileId, pm: MobileId) {
    quester::tell(w, Quester::Mobile(id), pm, 1078136);
}

fn jockles_new(w: &mut World) -> MobileId {
    let id = new_creature(w, &JOCKLES, AIType::AiVendor, FightMode::None, 2, 1);
    set_title(w, id, Some("the Swordsmanship Instructor".to_string()));
    set_body(w, id, Body(0x190));
    set_hue(w, id, 0x83FA);
    set_hair_item_id(w, id, 0x203C);
    set_hair_hue(w, id, 0x8A7);
    set_speed(w, id, 0.5, 2.0, true);
    init_stats(w, id, 100, 100, 25);
    set_skill(w, id, SkillName::Anatomy, 120.0);
    set_skill(w, id, SkillName::Parry, 120.0);
    set_skill(w, id, SkillName::Healing, 120.0);
    set_skill(w, id, SkillName::Tactics, 120.0);
    set_skill(w, id, SkillName::Swords, 120.0);
    set_skill(w, id, SkillName::Focus, 120.0);
    add_item_hue(w, id, "Backpack", None);
    add_item_hue(w, id, "Broadsword", None);
    add_item_hue(w, id, "PlateChest", None);
    add_item_hue(w, id, "PlateLegs", None);
    add_item_hue(w, id, "PlateGloves", None);
    add_item_hue(w, id, "PlateArms", None);
    add_item_hue(w, id, "PlateGorget", None);
    add_item_hue(w, id, "OrderShield", None);
    id
}
creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Jockles`.
    JOCKLES, JOCKLES_C = "Server.Engines.MLQuests.Definitions.Jockles" : BASE_CREATURE / BASE_CREATURE_C, ctor jockles_new, encoded false;
    c {
        is_invulnerable: |_w, _id| true,
        can_teach: |_w, _id| true,
        default_name: |_w, _id| Some("Jockles"),
        can_shout: |_w, _id| true,
    }
    m {
    }
}
fn jockles_shout(w: &mut World, id: MobileId, pm: MobileId) {
    quester::tell(w, Quester::Mobile(id), pm, 1078135);
}

fn tyl_ariadne_new(w: &mut World) -> MobileId {
    let id = new_creature(w, &TYL_ARIADNE, AIType::AiVendor, FightMode::None, 2, 1);
    set_title(w, id, Some("the Parrying Instructor".to_string()));
    set_body(w, id, Body(0x190));
    set_hue(w, id, 0x8374);
    set_hair_item_id(w, id, 0);
    set_speed(w, id, 0.5, 2.0, true);
    init_stats(w, id, 100, 100, 25);
    set_skill(w, id, SkillName::Anatomy, 120.0);
    set_skill(w, id, SkillName::Parry, 120.0);
    set_skill(w, id, SkillName::Healing, 120.0);
    set_skill(w, id, SkillName::Tactics, 120.0);
    set_skill(w, id, SkillName::Swords, 120.0);
    set_skill(w, id, SkillName::Meditation, 120.0);
    set_skill(w, id, SkillName::Focus, 120.0);
    add_item_hue(w, id, "Backpack", None);
    add_item_hue(w, id, "ElvenBoots", Some(0x96D));
    let item = make_named(w, "StuddedLegs");
    set_item_hue(w, item, 0x96D);
    add_item_id(w, id, item);
    let item = make_named(w, "StuddedGloves");
    set_item_hue(w, item, 0x96D);
    add_item_id(w, id, item);
    let item = make_named(w, "StuddedGorget");
    set_item_hue(w, item, 0x96D);
    add_item_id(w, id, item);
    let item = make_named(w, "StuddedChest");
    set_item_hue(w, item, 0x96D);
    add_item_id(w, id, item);
    let item = make_named(w, "StuddedArms");
    set_item_hue(w, item, 0x96D);
    add_item_id(w, id, item);
    let item = make_named(w, "DiamondMace");
    set_item_hue(w, item, 0x96D);
    add_item_id(w, id, item);
    id
}
creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.TylAriadne`.
    TYL_ARIADNE, TYL_ARIADNE_C = "Server.Engines.MLQuests.Definitions.TylAriadne" : BASE_CREATURE / BASE_CREATURE_C, ctor tyl_ariadne_new, encoded false;
    c {
        is_invulnerable: |_w, _id| true,
        can_teach: |_w, _id| true,
        default_name: |_w, _id| Some("Tyl Ariadne"),
        can_shout: |_w, _id| true,
    }
    m {
    }
}
fn tyl_ariadne_shout(w: &mut World, id: MobileId, pm: MobileId) {
    quester::tell(w, Quester::Mobile(id), pm, 1078140);
}

fn alefian_new(w: &mut World) -> MobileId {
    let id = new_creature(w, &ALEFIAN, AIType::AiVendor, FightMode::None, 2, 1);
    set_title(w, id, Some("the Resisting Spells Instructor".to_string()));
    set_body(w, id, Body(0x190));
    set_hue(w, id, 0x83EA);
    set_hair_item_id(w, id, 0x203D);
    set_hair_hue(w, id, 0x457);
    set_speed(w, id, 0.5, 2.0, true);
    init_stats(w, id, 100, 100, 25);
    set_skill(w, id, SkillName::EvalInt, 120.0);
    set_skill(w, id, SkillName::Inscribe, 120.0);
    set_skill(w, id, SkillName::Magery, 120.0);
    set_skill(w, id, SkillName::MagicResist, 120.0);
    set_skill(w, id, SkillName::Wrestling, 120.0);
    set_skill(w, id, SkillName::Meditation, 120.0);
    add_item_hue(w, id, "Backpack", None);
    add_item_hue(w, id, "Robe", None);
    add_item_hue(w, id, "Sandals", None);
    id
}
creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Alefian`.
    ALEFIAN, ALEFIAN_C = "Server.Engines.MLQuests.Definitions.Alefian" : BASE_CREATURE / BASE_CREATURE_C, ctor alefian_new, encoded false;
    c {
        is_invulnerable: |_w, _id| true,
        can_teach: |_w, _id| true,
        default_name: |_w, _id| Some("Alefian"),
        can_shout: |_w, _id| true,
    }
    m {
    }
}
fn alefian_shout(w: &mut World, id: MobileId, pm: MobileId) {
    quester::tell(w, Quester::Mobile(id), pm, 1078130);
}

fn gustar_new(w: &mut World) -> MobileId {
    let id = new_creature(w, &GUSTAR, AIType::AiVendor, FightMode::None, 2, 1);
    set_title(w, id, Some("the Meditation Instructor".to_string()));
    set_body(w, id, Body(0x190));
    set_hue(w, id, 0x83F5);
    set_hair_item_id(w, id, 0x203B);
    set_hair_hue(w, id, 0x455);
    set_speed(w, id, 0.5, 2.0, true);
    init_stats(w, id, 100, 100, 25);
    set_skill(w, id, SkillName::EvalInt, 120.0);
    set_skill(w, id, SkillName::Inscribe, 120.0);
    set_skill(w, id, SkillName::Magery, 120.0);
    set_skill(w, id, SkillName::MagicResist, 120.0);
    set_skill(w, id, SkillName::Wrestling, 120.0);
    set_skill(w, id, SkillName::Meditation, 120.0);
    add_item_hue(w, id, "Backpack", None);
    add_item_hue(w, id, "GustarShroud", None);
    add_item_hue(w, id, "Sandals", None);
    id
}
creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Gustar`.
    GUSTAR, GUSTAR_C = "Server.Engines.MLQuests.Definitions.Gustar" : BASE_CREATURE / BASE_CREATURE_C, ctor gustar_new, encoded false;
    c {
        is_invulnerable: |_w, _id| true,
        can_teach: |_w, _id| true,
        default_name: |_w, _id| Some("Gustar"),
        can_shout: |_w, _id| true,
    }
    m {
    }
}
fn gustar_shout(w: &mut World, id: MobileId, pm: MobileId) {
    quester::tell(w, Quester::Mobile(id), pm, 1078126);
}

fn jillian_new(w: &mut World) -> MobileId {
    let id = new_creature(w, &JILLIAN, AIType::AiVendor, FightMode::None, 2, 1);
    set_title(w, id, Some("the Inscription Instructor".to_string()));
    set_body(w, id, Body(0x191));
    set_female(w, id, true);
    set_hue(w, id, 0x83EA);
    set_hair_item_id(w, id, 0x203D);
    set_hair_hue(w, id, 0x455);
    set_speed(w, id, 0.5, 2.0, true);
    init_stats(w, id, 100, 100, 25);
    set_skill(w, id, SkillName::EvalInt, 120.0);
    set_skill(w, id, SkillName::Inscribe, 120.0);
    set_skill(w, id, SkillName::Magery, 120.0);
    set_skill(w, id, SkillName::MagicResist, 120.0);
    set_skill(w, id, SkillName::Wrestling, 120.0);
    set_skill(w, id, SkillName::Meditation, 120.0);
    add_item_hue(w, id, "Backpack", None);
    add_item_hue(w, id, "Robe", Some(0x479));
    add_item_hue(w, id, "Sandals", None);
    id
}
creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Jillian`.
    JILLIAN, JILLIAN_C = "Server.Engines.MLQuests.Definitions.Jillian" : BASE_CREATURE / BASE_CREATURE_C, ctor jillian_new, encoded false;
    c {
        is_invulnerable: |_w, _id| true,
        can_teach: |_w, _id| true,
        default_name: |_w, _id| Some("Jillian"),
        can_shout: |_w, _id| true,
    }
    m {
    }
}
fn jillian_shout(w: &mut World, id: MobileId, pm: MobileId) {
    quester::tell(w, Quester::Mobile(id), pm, 1078129);
}

fn kaelynna_new(w: &mut World) -> MobileId {
    let id = new_creature(w, &KAELYNNA, AIType::AiVendor, FightMode::None, 2, 1);
    set_title(w, id, Some("the Magery Instructor".to_string()));
    set_body(w, id, Body(0x191));
    set_female(w, id, true);
    set_hue(w, id, 0x83EA);
    set_hair_item_id(w, id, 0x203C);
    set_hair_hue(w, id, 0x47D);
    set_speed(w, id, 0.5, 2.0, true);
    init_stats(w, id, 100, 100, 25);
    set_skill(w, id, SkillName::EvalInt, 120.0);
    set_skill(w, id, SkillName::Inscribe, 120.0);
    set_skill(w, id, SkillName::Magery, 120.0);
    set_skill(w, id, SkillName::MagicResist, 120.0);
    set_skill(w, id, SkillName::Wrestling, 120.0);
    set_skill(w, id, SkillName::Meditation, 120.0);
    add_item_hue(w, id, "Backpack", None);
    add_item_hue(w, id, "Robe", Some(0x592));
    add_item_hue(w, id, "Sandals", None);
    id
}
creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Kaelynna`.
    KAELYNNA, KAELYNNA_C = "Server.Engines.MLQuests.Definitions.Kaelynna" : BASE_CREATURE / BASE_CREATURE_C, ctor kaelynna_new, encoded false;
    c {
        is_invulnerable: |_w, _id| true,
        can_teach: |_w, _id| true,
        default_name: |_w, _id| Some("Kaelynna"),
        can_shout: |_w, _id| true,
    }
    m {
    }
}
fn kaelynna_shout(w: &mut World, id: MobileId, pm: MobileId) {
    quester::tell(w, Quester::Mobile(id), pm, 1078125);
}

fn mithneral_new(w: &mut World) -> MobileId {
    let id = new_creature(w, &MITHNERAL, AIType::AiVendor, FightMode::None, 2, 1);
    set_title(
        w,
        id,
        Some("the Evaluating Intelligence Instructor".to_string()),
    );
    set_body(w, id, Body(0x190));
    set_hue(w, id, 0x83EA);
    set_hair_item_id(w, id, 0x203C);
    set_hair_hue(w, id, 0x455);
    set_speed(w, id, 0.5, 2.0, true);
    init_stats(w, id, 100, 100, 25);
    set_skill(w, id, SkillName::EvalInt, 120.0);
    set_skill(w, id, SkillName::Inscribe, 120.0);
    set_skill(w, id, SkillName::Magery, 120.0);
    set_skill(w, id, SkillName::MagicResist, 120.0);
    set_skill(w, id, SkillName::Wrestling, 120.0);
    set_skill(w, id, SkillName::Meditation, 120.0);
    add_item_hue(w, id, "Backpack", None);
    add_item_hue(w, id, "Sandals", None);
    add_item_props(w, id, "GustarShroud", None, Some(0x51C));
    id
}
creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Mithneral`.
    MITHNERAL, MITHNERAL_C = "Server.Engines.MLQuests.Definitions.Mithneral" : BASE_CREATURE / BASE_CREATURE_C, ctor mithneral_new, encoded false;
    c {
        is_invulnerable: |_w, _id| true,
        can_teach: |_w, _id| true,
        default_name: |_w, _id| Some("Mithneral"),
        can_shout: |_w, _id| true,
    }
    m {
    }
}
fn mithneral_shout(w: &mut World, id: MobileId, pm: MobileId) {
    quester::tell(w, Quester::Mobile(id), pm, 1078127);
}

fn andreas_vesalius_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &ANDREAS_VESALIUS,
        AIType::AiVendor,
        FightMode::None,
        2,
        1,
    );
    set_title(w, id, Some("the Anatomy Instructor".to_string()));
    set_body(w, id, Body(0x190));
    set_hue(w, id, 0x83EC);
    set_hair_item_id(w, id, 0x203C);
    set_hair_hue(w, id, 0x477);
    set_facial_hair_item_id(w, id, 0x203E);
    set_facial_hair_hue(w, id, 0x477);
    set_speed(w, id, 0.5, 2.0, true);
    init_stats(w, id, 100, 100, 25);
    set_skill(w, id, SkillName::Anatomy, 120.0);
    set_skill(w, id, SkillName::Parry, 120.0);
    set_skill(w, id, SkillName::Healing, 120.0);
    set_skill(w, id, SkillName::Tactics, 120.0);
    set_skill(w, id, SkillName::Swords, 120.0);
    set_skill(w, id, SkillName::Focus, 120.0);
    add_item_hue(w, id, "Backpack", None);
    add_item_hue(w, id, "Boots", None);
    add_item_hue(w, id, "BlackStaff", None);
    add_item_hue(w, id, "LongPants", None);
    add_item_hue(w, id, "Tunic", Some(0x66D));
    id
}
creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.AndreasVesalius`.
    ANDREAS_VESALIUS, ANDREAS_VESALIUS_C = "Server.Engines.MLQuests.Definitions.AndreasVesalius" : BASE_CREATURE / BASE_CREATURE_C, ctor andreas_vesalius_new, encoded false;
    c {
        is_invulnerable: |_w, _id| true,
        can_teach: |_w, _id| true,
        default_name: |_w, _id| Some("Andreas Vesalius"),
        can_shout: |_w, _id| true,
    }
    m {
    }
}
fn andreas_vesalius_shout(w: &mut World, id: MobileId, pm: MobileId) {
    quester::tell(w, Quester::Mobile(id), pm, 1078138);
}

fn avicenna_new(w: &mut World) -> MobileId {
    let id = new_creature(w, &AVICENNA, AIType::AiVendor, FightMode::None, 2, 1);
    set_title(w, id, Some("the Healing Instructor".to_string()));
    set_body(w, id, Body(0x190));
    set_hue(w, id, 0x83EA);
    set_hair_item_id(w, id, 0x203B);
    set_hair_hue(w, id, 0x477);
    set_speed(w, id, 0.5, 2.0, true);
    init_stats(w, id, 100, 100, 25);
    set_skill(w, id, SkillName::Anatomy, 120.0);
    set_skill(w, id, SkillName::Parry, 120.0);
    set_skill(w, id, SkillName::Healing, 120.0);
    set_skill(w, id, SkillName::Tactics, 120.0);
    set_skill(w, id, SkillName::Swords, 120.0);
    set_skill(w, id, SkillName::Focus, 120.0);
    add_item_hue(w, id, "Backpack", None);
    add_item_hue(w, id, "Robe", Some(0x66D));
    add_item_hue(w, id, "Boots", None);
    add_item_hue(w, id, "GnarledStaff", None);
    id
}
creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Avicenna`.
    AVICENNA, AVICENNA_C = "Server.Engines.MLQuests.Definitions.Avicenna" : BASE_CREATURE / BASE_CREATURE_C, ctor avicenna_new, encoded false;
    c {
        is_invulnerable: |_w, _id| true,
        can_teach: |_w, _id| true,
        default_name: |_w, _id| Some("Avicenna"),
        can_shout: |_w, _id| true,
    }
    m {
    }
}
fn avicenna_shout(w: &mut World, id: MobileId, pm: MobileId) {
    quester::tell(w, Quester::Mobile(id), pm, 1078137);
}

fn sarsmea_smythe_new(w: &mut World) -> MobileId {
    let id = new_creature(w, &SARSMEA_SMYTHE, AIType::AiVendor, FightMode::None, 2, 1);
    set_title(w, id, Some("the Focus Instructor".to_string()));
    set_body(w, id, Body(0x191));
    set_female(w, id, true);
    set_hue(w, id, 0x83EA);
    set_hair_item_id(w, id, 0x203C);
    set_hair_hue(w, id, 0x456);
    set_speed(w, id, 0.5, 2.0, true);
    init_stats(w, id, 100, 100, 25);
    set_skill(w, id, SkillName::Anatomy, 120.0);
    set_skill(w, id, SkillName::Parry, 120.0);
    set_skill(w, id, SkillName::Healing, 120.0);
    set_skill(w, id, SkillName::Tactics, 120.0);
    set_skill(w, id, SkillName::Swords, 120.0);
    set_skill(w, id, SkillName::Focus, 120.0);
    add_item_hue(w, id, "Backpack", None);
    add_item_hue(w, id, "ThighBoots", None);
    add_item_hue(w, id, "StuddedGorget", None);
    add_item_hue(w, id, "LeatherLegs", None);
    add_item_hue(w, id, "FemaleLeatherChest", None);
    add_item_hue(w, id, "StuddedGloves", None);
    add_item_hue(w, id, "LeatherNinjaBelt", None);
    add_item_hue(w, id, "LightPlateJingasa", None);
    id
}
creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.SarsmeaSmythe`.
    SARSMEA_SMYTHE, SARSMEA_SMYTHE_C = "Server.Engines.MLQuests.Definitions.SarsmeaSmythe" : BASE_CREATURE / BASE_CREATURE_C, ctor sarsmea_smythe_new, encoded false;
    c {
        is_invulnerable: |_w, _id| true,
        can_teach: |_w, _id| true,
        default_name: |_w, _id| Some("Sarsmea Smythe"),
        can_shout: |_w, _id| true,
    }
    m {
    }
}
fn sarsmea_smythe_shout(w: &mut World, id: MobileId, pm: MobileId) {
    quester::tell(w, Quester::Mobile(id), pm, 1078139);
}

fn chiyo_new(w: &mut World) -> MobileId {
    let id = new_creature(w, &CHIYO, AIType::AiVendor, FightMode::None, 2, 1);
    set_title(w, id, Some("the Hiding Instructor".to_string()));
    set_body(w, id, Body(0xF7));
    set_speed(w, id, 0.5, 2.0, true);
    init_stats(w, id, 100, 100, 25);
    set_skill(w, id, SkillName::Hiding, 120.0);
    set_skill(w, id, SkillName::Tactics, 120.0);
    set_skill(w, id, SkillName::Tracking, 120.0);
    set_skill(w, id, SkillName::Fencing, 120.0);
    set_skill(w, id, SkillName::Stealth, 120.0);
    set_skill(w, id, SkillName::Ninjitsu, 120.0);
    id
}
creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Chiyo`.
    CHIYO, CHIYO_C = "Server.Engines.MLQuests.Definitions.Chiyo" : BASE_CREATURE / BASE_CREATURE_C, ctor chiyo_new, encoded false;
    c {
        is_invulnerable: |_w, _id| true,
        can_teach: |_w, _id| true,
        default_name: |_w, _id| Some("Chiyo"),
        can_shout: |_w, _id| true,
    }
    m {
    }
}
fn chiyo_shout(w: &mut World, id: MobileId, pm: MobileId) {
    quester::tell(w, Quester::Mobile(id), pm, 1078165);
}

fn jun_new(w: &mut World) -> MobileId {
    let id = new_creature(w, &JUN, AIType::AiVendor, FightMode::None, 2, 1);
    set_title(w, id, Some("the Stealth Instructor".to_string()));
    set_body(w, id, Body(0x190));
    set_hue(w, id, 0x8403);
    set_hair_item_id(w, id, 0x203B);
    set_hair_hue(w, id, 0x455);
    set_speed(w, id, 0.5, 2.0, true);
    init_stats(w, id, 100, 100, 25);
    set_skill(w, id, SkillName::Hiding, 120.0);
    set_skill(w, id, SkillName::Tactics, 120.0);
    set_skill(w, id, SkillName::Tracking, 120.0);
    set_skill(w, id, SkillName::Fencing, 120.0);
    set_skill(w, id, SkillName::Stealth, 120.0);
    set_skill(w, id, SkillName::Ninjitsu, 120.0);
    add_item_hue(w, id, "Backpack", None);
    add_item_hue(w, id, "SamuraiTabi", None);
    add_item_hue(w, id, "LeatherNinjaPants", None);
    add_item_hue(w, id, "LeatherNinjaMitts", None);
    add_item_hue(w, id, "LeatherNinjaHood", None);
    add_item_hue(w, id, "LeatherNinjaJacket", None);
    add_item_hue(w, id, "LeatherNinjaBelt", None);
    id
}
creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Jun`.
    JUN, JUN_C = "Server.Engines.MLQuests.Definitions.Jun" : BASE_CREATURE / BASE_CREATURE_C, ctor jun_new, encoded false;
    c {
        is_invulnerable: |_w, _id| true,
        can_teach: |_w, _id| true,
        default_name: |_w, _id| Some("Jun"),
        can_shout: |_w, _id| true,
    }
    m {
    }
}
fn jun_shout(w: &mut World, id: MobileId, pm: MobileId) {
    quester::tell(w, Quester::Mobile(id), pm, 1078175);
}

fn walker_new(w: &mut World) -> MobileId {
    let id = new_creature(w, &WALKER, AIType::AiVendor, FightMode::None, 2, 1);
    set_title(w, id, Some("the Tracking Instructor".to_string()));
    set_body(w, id, Body(0x190));
    set_hue(w, id, 0x83EA);
    set_hair_item_id(w, id, 0x203B);
    set_hair_hue(w, id, 0x47D);
    set_facial_hair_item_id(w, id, 0x204B);
    set_facial_hair_hue(w, id, 0x47D);
    set_speed(w, id, 0.5, 2.0, true);
    init_stats(w, id, 100, 100, 25);
    set_skill(w, id, SkillName::Hiding, 120.0);
    set_skill(w, id, SkillName::Tactics, 120.0);
    set_skill(w, id, SkillName::Tracking, 120.0);
    set_skill(w, id, SkillName::Fencing, 120.0);
    set_skill(w, id, SkillName::Wrestling, 120.0);
    set_skill(w, id, SkillName::Stealth, 120.0);
    set_skill(w, id, SkillName::Ninjitsu, 120.0);
    add_item_hue(w, id, "Backpack", None);
    add_item_hue(w, id, "Boots", Some(0x455));
    add_item_hue(w, id, "LongPants", Some(0x455));
    add_item_hue(w, id, "FancyShirt", Some(0x47D));
    add_item_hue(w, id, "FloppyHat", Some(0x455));
    id
}
creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Walker`.
    WALKER, WALKER_C = "Server.Engines.MLQuests.Definitions.Walker" : BASE_CREATURE / BASE_CREATURE_C, ctor walker_new, encoded false;
    c {
        is_invulnerable: |_w, _id| true,
        can_teach: |_w, _id| true,
        default_name: |_w, _id| Some("Walker"),
        can_shout: |_w, _id| true,
    }
    m {
    }
}
fn walker_shout(w: &mut World, id: MobileId, pm: MobileId) {
    let v1 = [1078213, 1078212, 1078214][rnd::below(w, 3) as usize];
    quester::tell(w, Quester::Mobile(id), pm, v1);
}

fn jacob_waltz_new(w: &mut World) -> MobileId {
    let id = new_creature(w, &JACOB_WALTZ, AIType::AiVendor, FightMode::None, 2, 1);
    set_title(w, id, Some("the Miner Instructor".to_string()));
    set_body(w, id, Body(0x190));
    set_hue(w, id, 0x83EA);
    set_hair_item_id(w, id, 0x2048);
    set_hair_hue(w, id, 0x44E);
    set_facial_hair_item_id(w, id, 0x204D);
    set_facial_hair_hue(w, id, 0x44E);
    set_speed(w, id, 0.5, 2.0, true);
    init_stats(w, id, 100, 100, 25);
    set_skill(w, id, SkillName::ArmsLore, 120.0);
    set_skill(w, id, SkillName::Blacksmith, 120.0);
    set_skill(w, id, SkillName::Magery, 120.0);
    set_skill(w, id, SkillName::Tactics, 120.0);
    set_skill(w, id, SkillName::Tinkering, 120.0);
    set_skill(w, id, SkillName::Swords, 120.0);
    set_skill(w, id, SkillName::Mining, 120.0);
    add_item_hue(w, id, "Backpack", None);
    add_item_hue(w, id, "Pickaxe", None);
    add_item_hue(w, id, "Boots", None);
    add_item_hue(w, id, "WideBrimHat", Some(0x966));
    add_item_hue(w, id, "ShortPants", Some(0x370));
    add_item_hue(w, id, "Shirt", Some(0x966));
    add_item_hue(w, id, "HalfApron", Some(0x1BB));
    id
}
creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.JacobWaltz`.
    JACOB_WALTZ, JACOB_WALTZ_C = "Server.Engines.MLQuests.Definitions.JacobWaltz" : BASE_CREATURE / BASE_CREATURE_C, ctor jacob_waltz_new, encoded false;
    c {
        is_invulnerable: |_w, _id| true,
        can_teach: |_w, _id| true,
        default_name: |_w, _id| Some("Jacob Waltz"),
        can_shout: |_w, _id| true,
    }
    m {
    }
}
fn jacob_waltz_shout(w: &mut World, id: MobileId, pm: MobileId) {
    quester::tell(w, Quester::Mobile(id), pm, 1078124);
}

fn andric_new(w: &mut World) -> MobileId {
    let id = new_creature(w, &ANDRIC, AIType::AiVendor, FightMode::None, 2, 1);
    set_title(w, id, Some("the archer trainer".to_string()));
    set_race_named(w, id, "Human");
    set_body(w, id, Body(0x190));
    set_female(w, id, false);
    let v1 = race_skin_hue(w, id);
    set_hue(w, id, v1);
    set_speed(w, id, 0.5, 2.0, true);
    init_stats(w, id, 100, 100, 25);
    assign_random_hair(w, id);
    set_skill_range(w, id, SkillName::Archery, 60.0, 80.0);
    add_item_hue(w, id, "Backpack", None);
    let item = make_named(w, "LeatherChest");
    set_item_hue(w, item, 0x1BB);
    add_item_id(w, id, item);
    let item = make_named(w, "LeatherLegs");
    set_item_hue(w, item, 0x6AD);
    add_item_id(w, id, item);
    let item = make_named(w, "LeatherArms");
    set_item_hue(w, item, 0x6AD);
    add_item_id(w, id, item);
    let item = make_named(w, "LeatherGloves");
    set_item_hue(w, item, 0x1BB);
    add_item_id(w, id, item);
    add_item_hue(w, id, "Boots", Some(0x1BB));
    add_item_hue(w, id, "CompositeBow", None);
    id
}
creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Andric`.
    ANDRIC, ANDRIC_C = "Server.Engines.MLQuests.Definitions.Andric" : BASE_CREATURE / BASE_CREATURE_C, ctor andric_new, encoded false;
    c {
        is_invulnerable: |_w, _id| true,
        can_teach: |_w, _id| true,
        default_name: |_w, _id| Some("Andric"),
        can_shout: |_w, _id| true,
    }
    m {
    }
}
fn andric_shout(w: &mut World, id: MobileId, pm: MobileId) {
    let v1 = [1074205, 1074213][rnd::below(w, 2) as usize];
    quester::tell(w, Quester::Mobile(id), pm, v1);
}

fn kashiel_new(w: &mut World) -> MobileId {
    let id = new_creature(w, &KASHIEL, AIType::AiVendor, FightMode::None, 2, 1);
    set_title(w, id, Some("the archer".to_string()));
    set_race_named(w, id, "Human");
    set_body(w, id, Body(0x191));
    set_female(w, id, true);
    let v1 = race_skin_hue(w, id);
    set_hue(w, id, v1);
    set_speed(w, id, 0.5, 2.0, true);
    init_stats(w, id, 100, 100, 25);
    assign_random_hair(w, id);
    add_item_hue(w, id, "Backpack", None);
    let item = make_named(w, "LeatherChest");
    set_item_hue(w, item, 0x1BB);
    add_item_id(w, id, item);
    let item = make_named(w, "LeatherLegs");
    set_item_hue(w, item, 0x901);
    add_item_id(w, id, item);
    let item = make_named(w, "LeatherArms");
    set_item_hue(w, item, 0x901);
    add_item_id(w, id, item);
    let item = make_named(w, "LeatherGloves");
    set_item_hue(w, item, 0x1BB);
    add_item_id(w, id, item);
    add_item_hue(w, id, "Boots", Some(0x1BB));
    add_item_hue(w, id, "CompositeBow", None);
    id
}
creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Kashiel`.
    KASHIEL, KASHIEL_C = "Server.Engines.MLQuests.Definitions.Kashiel" : BASE_CREATURE / BASE_CREATURE_C, ctor kashiel_new, encoded false;
    c {
        default_name: |_w, _id| Some("Kashiel"),
        is_invulnerable: |_w, _id| true,
    }
    m {
    }
}

fn asandos_new(w: &mut World) -> MobileId {
    let id = new_creature(w, &ASANDOS, AIType::AiVendor, FightMode::None, 2, 1);
    set_title(w, id, Some("the chef".to_string()));
    set_race_named(w, id, "Human");
    set_body(w, id, Body(0x190));
    set_female(w, id, false);
    let v1 = race_skin_hue(w, id);
    set_hue(w, id, v1);
    set_speed(w, id, 0.5, 2.0, true);
    init_stats(w, id, 100, 100, 25);
    assign_random_hair(w, id);
    add_item_hue(w, id, "Backpack", None);
    add_item_hue(w, id, "Boots", Some(0x901));
    add_item_hue(w, id, "ShortPants", None);
    add_item_hue(w, id, "Shirt", None);
    add_item_hue(w, id, "Cap", None);
    add_item_hue(w, id, "HalfApron", Some(0x28));
    id
}
creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Asandos`.
    ASANDOS, ASANDOS_C = "Server.Engines.MLQuests.Definitions.Asandos" : BASE_CREATURE / BASE_CREATURE_C, ctor asandos_new, encoded false;
    c {
        is_invulnerable: |_w, _id| true,
        default_name: |_w, _id| Some("Asandos"),
        can_shout: |_w, _id| true,
    }
    m {
    }
}
fn asandos_shout(w: &mut World, id: MobileId, pm: MobileId) {
    let v1 = [1074205, 1074213][rnd::below(w, 2) as usize];
    quester::tell(w, Quester::Mobile(id), pm, v1);
}

fn clairesse_new(w: &mut World) -> MobileId {
    let id = new_creature(w, &CLAIRESSE, AIType::AiVendor, FightMode::None, 2, 1);
    set_title(w, id, Some("the servant".to_string()));
    set_race_named(w, id, "Human");
    set_body(w, id, Body(0x191));
    set_female(w, id, true);
    let v1 = race_skin_hue(w, id);
    set_hue(w, id, v1);
    set_speed(w, id, 0.5, 2.0, true);
    init_stats(w, id, 100, 100, 25);
    assign_random_hair(w, id);
    add_item_hue(w, id, "Backpack", None);
    let v2 = utility::random_neutral_hue(&mut w.rng);
    add_item_hue(w, id, "Shoes", Some(v2));
    add_item_hue(w, id, "PlainDress", Some(0x3C9));
    id
}
creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Clairesse`.
    CLAIRESSE, CLAIRESSE_C = "Server.Engines.MLQuests.Definitions.Clairesse" : BASE_CREATURE / BASE_CREATURE_C, ctor clairesse_new, encoded false;
    c {
        is_invulnerable: |_w, _id| true,
        default_name: |_w, _id| Some("Clarisse"),
        can_shout: |_w, _id| true,
    }
    m {
    }
}
fn clairesse_shout(w: &mut World, id: MobileId, pm: MobileId) {
    let v1 = [1074205, 1074213][rnd::below(w, 2) as usize];
    quester::tell(w, Quester::Mobile(id), pm, v1);
}

fn gervis_new(w: &mut World) -> MobileId {
    let id = new_creature(w, &GERVIS, AIType::AiVendor, FightMode::None, 2, 1);
    set_title(w, id, Some("the blacksmith trainer".to_string()));
    set_race_named(w, id, "Human");
    set_body(w, id, Body(0x190));
    set_female(w, id, false);
    let v1 = race_skin_hue(w, id);
    set_hue(w, id, v1);
    set_speed(w, id, 0.5, 2.0, true);
    init_stats(w, id, 100, 100, 25);
    assign_random_hair(w, id);
    set_skill_range(w, id, SkillName::Blacksmith, 60.0, 80.0);
    add_item_hue(w, id, "Backpack", None);
    add_item_hue(w, id, "Boots", Some(0x3B3));
    add_item_hue(w, id, "ShortPants", Some(0x1BB));
    add_item_hue(w, id, "Doublet", Some(0x652));
    add_item_hue(w, id, "SmithHammer", None);
    add_item_props(w, id, "LeatherGloves", None, Some(0x3B2));
    id
}
creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Gervis`.
    GERVIS, GERVIS_C = "Server.Engines.MLQuests.Definitions.Gervis" : BASE_CREATURE / BASE_CREATURE_C, ctor gervis_new, encoded false;
    c {
        is_invulnerable: |_w, _id| true,
        can_teach: |_w, _id| true,
        default_name: |_w, _id| Some("Gervis"),
        can_shout: |_w, _id| true,
    }
    m {
    }
}
fn gervis_shout(w: &mut World, id: MobileId, pm: MobileId) {
    let v1 = [1074205, 1074213, 1074211][rnd::below(w, 3) as usize];
    quester::tell(w, Quester::Mobile(id), pm, v1);
}

fn mugg_new(w: &mut World) -> MobileId {
    let id = new_creature(w, &MUGG, AIType::AiVendor, FightMode::None, 2, 1);
    set_title(w, id, Some("the miner".to_string()));
    set_race_named(w, id, "Human");
    set_body(w, id, Body(0x190));
    set_female(w, id, false);
    let v1 = race_skin_hue(w, id);
    set_hue(w, id, v1);
    set_speed(w, id, 0.5, 2.0, true);
    init_stats(w, id, 100, 100, 25);
    assign_random_hair(w, id);
    add_item_hue(w, id, "Backpack", None);
    add_item_hue(w, id, "Boots", Some(0x901));
    add_item_hue(w, id, "ShortPants", Some(0x3B3));
    add_item_hue(w, id, "Shirt", Some(0x22B));
    add_item_hue(w, id, "HalfApron", Some(0x5F1));
    add_item_hue(w, id, "SkullCap", Some(0x177));
    add_item_hue(w, id, "Pickaxe", None);
    id
}
creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Mugg`.
    MUGG, MUGG_C = "Server.Engines.MLQuests.Definitions.Mugg" : BASE_CREATURE / BASE_CREATURE_C, ctor mugg_new, encoded false;
    c {
        is_invulnerable: |_w, _id| true,
        default_name: |_w, _id| Some("Mugg"),
        can_shout: |_w, _id| true,
    }
    m {
    }
}
fn mugg_shout(w: &mut World, id: MobileId, pm: MobileId) {
    quester::tell(w, Quester::Mobile(id), pm, 1074211);
}

fn lowel_new(w: &mut World) -> MobileId {
    let id = new_creature(w, &LOWEL, AIType::AiVendor, FightMode::None, 2, 1);
    set_title(w, id, Some("the carpenter".to_string()));
    set_race_named(w, id, "Human");
    set_body(w, id, Body(0x190));
    set_female(w, id, false);
    let v1 = race_skin_hue(w, id);
    set_hue(w, id, v1);
    set_speed(w, id, 0.5, 2.0, true);
    init_stats(w, id, 100, 100, 25);
    assign_random_hair(w, id);
    add_item_hue(w, id, "Backpack", None);
    add_item_hue(w, id, "Boots", Some(0x543));
    add_item_hue(w, id, "ShortPants", Some(0x758));
    add_item_hue(w, id, "FancyShirt", Some(0x53A));
    add_item_hue(w, id, "HalfApron", Some(0x6D2));
    id
}
creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Lowel`.
    LOWEL, LOWEL_C = "Server.Engines.MLQuests.Definitions.Lowel" : BASE_CREATURE / BASE_CREATURE_C, ctor lowel_new, encoded false;
    c {
        is_invulnerable: |_w, _id| true,
        default_name: |_w, _id| Some("Lowel"),
        can_shout: |_w, _id| true,
    }
    m {
    }
}
fn lowel_shout(w: &mut World, id: MobileId, pm: MobileId) {
    let v1 = [1074205, 1074213][rnd::below(w, 2) as usize];
    quester::tell(w, Quester::Mobile(id), pm, v1);
}

fn lyle_new(w: &mut World) -> MobileId {
    let id = new_creature(w, &LYLE, AIType::AiVendor, FightMode::None, 2, 1);
    set_title(w, id, Some("the mage".to_string()));
    set_race_named(w, id, "Human");
    set_body(w, id, Body(0x190));
    set_female(w, id, false);
    let v1 = race_skin_hue(w, id);
    set_hue(w, id, v1);
    set_speed(w, id, 0.5, 2.0, true);
    init_stats(w, id, 100, 100, 25);
    assign_random_hair(w, id);
    add_item_hue(w, id, "Backpack", None);
    add_item_hue(w, id, "Robe", Some(0x2FD));
    add_item_hue(w, id, "ThighBoots", None);
    id
}
creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Lyle`.
    LYLE, LYLE_C = "Server.Engines.MLQuests.Definitions.Lyle" : BASE_CREATURE / BASE_CREATURE_C, ctor lyle_new, encoded false;
    c {
        is_invulnerable: |_w, _id| true,
        default_name: |_w, _id| Some("Lyle"),
        can_shout: |_w, _id| true,
    }
    m {
    }
}
fn lyle_shout(w: &mut World, id: MobileId, pm: MobileId) {
    let v1 = [1074205, 1074213][rnd::below(w, 2) as usize];
    quester::tell(w, Quester::Mobile(id), pm, v1);
}

fn nibbet_new(w: &mut World) -> MobileId {
    let id = new_creature(w, &NIBBET, AIType::AiVendor, FightMode::None, 2, 1);
    set_title(w, id, Some("the tinker".to_string()));
    set_race_named(w, id, "Human");
    set_body(w, id, Body(0x190));
    set_female(w, id, false);
    let v1 = race_skin_hue(w, id);
    set_hue(w, id, v1);
    set_speed(w, id, 0.5, 2.0, true);
    init_stats(w, id, 100, 100, 25);
    assign_random_hair(w, id);
    add_item_hue(w, id, "Backpack", None);
    add_item_hue(w, id, "Boots", Some(0x591));
    add_item_hue(w, id, "ShortPants", Some(0xF8));
    add_item_hue(w, id, "Shirt", Some(0x2D));
    add_item_hue(w, id, "FullApron", Some(0x288));
    id
}
creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Nibbet`.
    NIBBET, NIBBET_C = "Server.Engines.MLQuests.Definitions.Nibbet" : BASE_CREATURE / BASE_CREATURE_C, ctor nibbet_new, encoded false;
    c {
        default_name: |_w, _id| Some("Nibbet"),
        is_invulnerable: |_w, _id| true,
    }
    m {
    }
}

fn norton_new(w: &mut World) -> MobileId {
    let id = new_creature(w, &NORTON, AIType::AiVendor, FightMode::None, 2, 1);
    set_title(w, id, Some("the fisher".to_string()));
    set_race_named(w, id, "Human");
    set_body(w, id, Body(0x190));
    set_female(w, id, false);
    let v1 = race_skin_hue(w, id);
    set_hue(w, id, v1);
    set_speed(w, id, 0.5, 2.0, true);
    init_stats(w, id, 100, 100, 25);
    assign_random_hair(w, id);
    add_item_hue(w, id, "Backpack", None);
    add_item_hue(w, id, "ThighBoots", None);
    add_item_hue(w, id, "LongPants", Some(0x6C2));
    add_item_hue(w, id, "Shirt", Some(0x11D));
    id
}
creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Norton`.
    NORTON, NORTON_C = "Server.Engines.MLQuests.Definitions.Norton" : BASE_CREATURE / BASE_CREATURE_C, ctor norton_new, encoded false;
    c {
        is_invulnerable: |_w, _id| true,
        default_name: |_w, _id| Some("Norton"),
        can_shout: |_w, _id| true,
    }
    m {
    }
}
fn norton_shout(w: &mut World, id: MobileId, pm: MobileId) {
    let v1 = [1074205, 1074213, 1074211][rnd::below(w, 3) as usize];
    quester::tell(w, Quester::Mobile(id), pm, v1);
}

fn sadrah_new(w: &mut World) -> MobileId {
    let id = new_creature(w, &SADRAH, AIType::AiVendor, FightMode::None, 2, 1);
    set_title(w, id, Some("the courier".to_string()));
    set_race_named(w, id, "Human");
    set_body(w, id, Body(0x191));
    set_female(w, id, true);
    let v1 = race_skin_hue(w, id);
    set_hue(w, id, v1);
    set_speed(w, id, 0.5, 2.0, true);
    init_stats(w, id, 100, 100, 25);
    assign_random_hair(w, id);
    add_item_hue(w, id, "Backpack", None);
    add_item_hue(w, id, "Boots", Some(0x901));
    add_item_hue(w, id, "Skirt", Some(0x52));
    add_item_hue(w, id, "Shirt", Some(0x127));
    add_item_hue(w, id, "Cloak", Some(0x65));
    add_item_hue(w, id, "Longsword", None);
    id
}
creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Sadrah`.
    SADRAH, SADRAH_C = "Server.Engines.MLQuests.Definitions.Sadrah" : BASE_CREATURE / BASE_CREATURE_C, ctor sadrah_new, encoded false;
    c {
        is_invulnerable: |_w, _id| true,
        default_name: |_w, _id| Some("Sadrah"),
        can_shout: |_w, _id| true,
    }
    m {
    }
}
fn sadrah_shout(w: &mut World, id: MobileId, pm: MobileId) {
    let v1 = [1074205, 1074213, 1074211][rnd::below(w, 3) as usize];
    quester::tell(w, Quester::Mobile(id), pm, v1);
}

fn hargrove_new(w: &mut World) -> MobileId {
    let id = new_creature(w, &HARGROVE, AIType::AiVendor, FightMode::None, 2, 1);
    set_title(w, id, Some("the Lumberjack".to_string()));
    set_race_named(w, id, "Human");
    set_body(w, id, Body(0x190));
    set_female(w, id, false);
    let v1 = race_skin_hue(w, id);
    set_hue(w, id, v1);
    set_speed(w, id, 0.5, 2.0, true);
    init_stats(w, id, 100, 100, 25);
    assign_random_hair(w, id);
    add_item_hue(w, id, "Backpack", None);
    add_item_hue(w, id, "Boots", Some(0x901));
    add_item_hue(w, id, "StuddedLegs", None);
    add_item_hue(w, id, "Shirt", Some(0x288));
    add_item_hue(w, id, "Bandana", Some(0x20));
    add_item_hue(w, id, "BattleAxe", None);
    add_item_props(w, id, "PlateGloves", None, Some(0x21E));
    id
}
creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Hargrove`.
    HARGROVE, HARGROVE_C = "Server.Engines.MLQuests.Definitions.Hargrove" : BASE_CREATURE / BASE_CREATURE_C, ctor hargrove_new, encoded false;
    c {
        is_invulnerable: |_w, _id| true,
        default_name: |_w, _id| Some("Hargrove"),
        can_shout: |_w, _id| true,
    }
    m {
    }
}
fn hargrove_shout(w: &mut World, id: MobileId, pm: MobileId) {
    let v1 = [1074213, 1074211][rnd::below(w, 2) as usize];
    quester::tell(w, Quester::Mobile(id), pm, v1);
}

fn beotham_new(w: &mut World) -> MobileId {
    let id = new_creature(w, &BEOTHAM, AIType::AiVendor, FightMode::None, 2, 1);
    set_title(w, id, Some("the bowcrafter".to_string()));
    set_race_named(w, id, "Elf");
    set_body(w, id, Body(0x25D));
    set_female(w, id, false);
    let v1 = race_skin_hue(w, id);
    set_hue(w, id, v1);
    set_speed(w, id, 0.5, 2.0, true);
    init_stats(w, id, 100, 100, 25);
    assign_random_hair(w, id);
    set_skill_range(w, id, SkillName::Meditation, 60.0, 80.0);
    set_skill_range(w, id, SkillName::Focus, 60.0, 80.0);
    add_item_hue(w, id, "Sandals", Some(0x901));
    add_item_hue(w, id, "ShortPants", Some(0x522));
    add_item_hue(w, id, "FancyShirt", Some(0x515));
    add_item_props(w, id, "LeafGloves", None, Some(0x901));
    id
}
creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Beotham`.
    BEOTHAM, BEOTHAM_C = "Server.Engines.MLQuests.Definitions.Beotham" : BASE_CREATURE / BASE_CREATURE_C, ctor beotham_new, encoded false;
    c {
        is_invulnerable: |_w, _id| true,
        can_teach: |_w, _id| true,
        default_name: |_w, _id| Some("Beotham"),
        can_shout: |_w, _id| true,
    }
    m {
    }
}
fn beotham_shout(w: &mut World, id: MobileId, pm: MobileId) {
    let v1 = [1074187, 1074184][rnd::below(w, 2) as usize];
    quester::tell(w, Quester::Mobile(id), pm, v1);
}

fn danoel_new(w: &mut World) -> MobileId {
    let id = new_creature(w, &DANOEL, AIType::AiVendor, FightMode::None, 2, 1);
    set_title(w, id, Some("the metal weaver".to_string()));
    set_race_named(w, id, "Elf");
    set_body(w, id, Body(0x25D));
    set_female(w, id, false);
    let v1 = race_skin_hue(w, id);
    set_hue(w, id, v1);
    set_speed(w, id, 0.5, 2.0, true);
    init_stats(w, id, 100, 100, 25);
    assign_random_hair(w, id);
    set_skill_range(w, id, SkillName::Meditation, 60.0, 80.0);
    set_skill_range(w, id, SkillName::Focus, 60.0, 80.0);
    add_item_hue(w, id, "ElvenBoots", Some(0x901));
    add_item_hue(w, id, "ElvenPants", Some(0x386));
    add_item_hue(w, id, "ElvenShirt", Some(0x75F));
    add_item_hue(w, id, "FullApron", Some(0x1BB));
    add_item_hue(w, id, "RoyalCirclet", None);
    add_item_hue(w, id, "SmithHammer", None);
    id
}
creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Danoel`.
    DANOEL, DANOEL_C = "Server.Engines.MLQuests.Definitions.Danoel" : BASE_CREATURE / BASE_CREATURE_C, ctor danoel_new, encoded false;
    c {
        is_invulnerable: |_w, _id| true,
        can_teach: |_w, _id| true,
        default_name: |_w, _id| Some("Danoel"),
        can_shout: |_w, _id| true,
    }
    m {
    }
}
fn danoel_shout(w: &mut World, id: MobileId, pm: MobileId) {
    quester::tell(w, Quester::Mobile(id), pm, 1074197);
}

fn tallinin_new(w: &mut World) -> MobileId {
    let id = new_creature(w, &TALLININ, AIType::AiVendor, FightMode::None, 2, 1);
    set_title(w, id, Some("the cloth weaver".to_string()));
    set_race_named(w, id, "Elf");
    set_body(w, id, Body(0x25E));
    set_female(w, id, true);
    let v1 = race_skin_hue(w, id);
    set_hue(w, id, v1);
    set_speed(w, id, 0.5, 2.0, true);
    init_stats(w, id, 100, 100, 25);
    assign_random_hair(w, id);
    set_skill_range(w, id, SkillName::Meditation, 60.0, 80.0);
    set_skill_range(w, id, SkillName::Focus, 60.0, 80.0);
    add_item_hue(w, id, "ElvenBoots", Some(0x901));
    add_item_hue(w, id, "Tunic", Some(0x37));
    add_item_hue(w, id, "Cloak", Some(0x735));
    id
}
creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Tallinin`.
    TALLININ, TALLININ_C = "Server.Engines.MLQuests.Definitions.Tallinin" : BASE_CREATURE / BASE_CREATURE_C, ctor tallinin_new, encoded false;
    c {
        is_invulnerable: |_w, _id| true,
        can_teach: |_w, _id| true,
        default_name: |_w, _id| Some("Tallinin"),
        can_shout: |_w, _id| true,
    }
    m {
    }
}
fn tallinin_shout(w: &mut World, id: MobileId, pm: MobileId) {
    let v1 = [1074188, 1074211][rnd::below(w, 2) as usize];
    quester::tell(w, Quester::Mobile(id), pm, v1);
}

fn tiana_new(w: &mut World) -> MobileId {
    let id = new_creature(w, &TIANA, AIType::AiVendor, FightMode::None, 2, 1);
    set_title(w, id, Some("the guard".to_string()));
    set_race_named(w, id, "Elf");
    set_body(w, id, Body(0x25E));
    set_female(w, id, true);
    let v1 = race_skin_hue(w, id);
    set_hue(w, id, v1);
    set_speed(w, id, 0.5, 2.0, true);
    init_stats(w, id, 100, 100, 25);
    assign_random_hair(w, id);
    set_skill_range(w, id, SkillName::Meditation, 60.0, 80.0);
    set_skill_range(w, id, SkillName::Focus, 60.0, 80.0);
    add_item_hue(w, id, "ElvenBoots", None);
    add_item_hue(w, id, "HidePants", None);
    add_item_hue(w, id, "HideFemaleChest", None);
    add_item_hue(w, id, "HidePauldrons", None);
    let item = make_named(w, "WoodlandBelt");
    set_item_hue(w, item, 0x673);
    add_item_id(w, id, item);
    let item = make_named(w, "RavenHelm");
    set_item_hue(w, item, 0x443);
    add_item_id(w, id, item);
    id
}
creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Tiana`.
    TIANA, TIANA_C = "Server.Engines.MLQuests.Definitions.Tiana" : BASE_CREATURE / BASE_CREATURE_C, ctor tiana_new, encoded false;
    c {
        is_invulnerable: |_w, _id| true,
        can_teach: |_w, _id| true,
        default_name: |_w, _id| Some("Tiana"),
        can_shout: |_w, _id| true,
    }
    m {
    }
}
fn tiana_shout(w: &mut World, id: MobileId, pm: MobileId) {
    let v1 = rnd::below(w, 2);
    quester::tell(w, Quester::Mobile(id), pm, 1074214 + v1);
}

fn lorekeeper_oolua_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &LOREKEEPER_OOLUA,
        AIType::AiVendor,
        FightMode::None,
        2,
        1,
    );
    set_title(w, id, Some("the keeper of tradition".to_string()));
    set_race_named(w, id, "Elf");
    set_body(w, id, Body(0x25E));
    set_female(w, id, true);
    let v1 = race_skin_hue(w, id);
    set_hue(w, id, v1);
    set_speed(w, id, 0.5, 2.0, true);
    init_stats(w, id, 100, 100, 25);
    assign_random_hair(w, id);
    set_skill_range(w, id, SkillName::Meditation, 60.0, 80.0);
    set_skill_range(w, id, SkillName::Focus, 60.0, 80.0);
    add_item_hue(w, id, "ElvenBoots", Some(0x75A));
    let v2 = utility::random_bright_hue(&mut w.rng);
    add_item_hue(w, id, "Skirt", Some(v2));
    add_item_hue(w, id, "FancyShirt", Some(0x742));
    add_item_hue(w, id, "Cloak", Some(0x1BB));
    add_item_hue(w, id, "WildStaff", None);
    id
}
creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.LorekeeperOolua`.
    LOREKEEPER_OOLUA, LOREKEEPER_OOLUA_C = "Server.Engines.MLQuests.Definitions.LorekeeperOolua" : BASE_CREATURE / BASE_CREATURE_C, ctor lorekeeper_oolua_new, encoded false;
    c {
        is_invulnerable: |_w, _id| true,
        can_teach: |_w, _id| true,
        default_name: |_w, _id| Some("Lorekeeper Oolua"),
        can_shout: |_w, _id| true,
    }
    m {
    }
}
fn lorekeeper_oolua_shout(w: &mut World, id: MobileId, pm: MobileId) {
    quester::tell(w, Quester::Mobile(id), pm, 1074187);
}

fn lorekeeper_rollarn_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &LOREKEEPER_ROLLARN,
        AIType::AiVendor,
        FightMode::None,
        2,
        1,
    );
    set_title(w, id, Some("the keeper of tradition".to_string()));
    set_race_named(w, id, "Elf");
    set_body(w, id, Body(0x25D));
    set_female(w, id, false);
    let v1 = race_skin_hue(w, id);
    set_hue(w, id, v1);
    set_speed(w, id, 0.5, 2.0, true);
    init_stats(w, id, 100, 100, 25);
    assign_random_hair(w, id);
    set_skill_range(w, id, SkillName::Meditation, 60.0, 80.0);
    set_skill_range(w, id, SkillName::Focus, 60.0, 80.0);
    add_item_hue(w, id, "Sandals", Some(0x1BB));
    add_item_hue(w, id, "Cloak", Some(0x296));
    add_item_hue(w, id, "Circlet", None);
    add_item_hue(w, id, "LeafChest", None);
    add_item_props(w, id, "LeafLegs", None, Some(0x71A));
    id
}
creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.LorekeeperRollarn`.
    LOREKEEPER_ROLLARN, LOREKEEPER_ROLLARN_C = "Server.Engines.MLQuests.Definitions.LorekeeperRollarn" : BASE_CREATURE / BASE_CREATURE_C, ctor lorekeeper_rollarn_new, encoded false;
    c {
        is_invulnerable: |_w, _id| true,
        can_teach: |_w, _id| true,
        default_name: |_w, _id| Some("Lorekeeper Rollarn"),
        can_shout: |_w, _id| true,
    }
    m {
    }
}
fn lorekeeper_rollarn_shout(w: &mut World, id: MobileId, pm: MobileId) {
    let v1 = [1074196, 1074197][rnd::below(w, 2) as usize];
    quester::tell(w, Quester::Mobile(id), pm, v1);
}

fn dallid_new(w: &mut World) -> MobileId {
    let id = new_creature(w, &DALLID, AIType::AiVendor, FightMode::None, 2, 1);
    set_title(w, id, Some("the cook".to_string()));
    set_race_named(w, id, "Elf");
    set_body(w, id, Body(0x25D));
    set_female(w, id, false);
    let v1 = race_skin_hue(w, id);
    set_hue(w, id, v1);
    set_speed(w, id, 0.5, 2.0, true);
    init_stats(w, id, 100, 100, 25);
    assign_random_hair(w, id);
    set_skill_range(w, id, SkillName::Meditation, 60.0, 80.0);
    set_skill_range(w, id, SkillName::Focus, 60.0, 80.0);
    add_item_hue(w, id, "Boots", Some(0x901));
    add_item_hue(w, id, "ShortPants", Some(0x73C));
    add_item_hue(w, id, "Shirt", Some(0x744));
    add_item_hue(w, id, "FullApron", Some(0x1BE));
    add_item_hue(w, id, "Cleaver", None);
    id
}
creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Dallid`.
    DALLID, DALLID_C = "Server.Engines.MLQuests.Definitions.Dallid" : BASE_CREATURE / BASE_CREATURE_C, ctor dallid_new, encoded false;
    c {
        is_invulnerable: |_w, _id| true,
        can_teach: |_w, _id| true,
        default_name: |_w, _id| Some("Dallid"),
        can_shout: |_w, _id| true,
    }
    m {
    }
}
fn dallid_shout(w: &mut World, id: MobileId, pm: MobileId) {
    let v1 = [1074185, 1074195][rnd::below(w, 2) as usize];
    quester::tell(w, Quester::Mobile(id), pm, v1);
}

fn canir_new(w: &mut World) -> MobileId {
    let id = new_creature(w, &CANIR, AIType::AiVendor, FightMode::None, 2, 1);
    set_title(w, id, Some("the thaumaturgist".to_string()));
    set_race_named(w, id, "Elf");
    set_body(w, id, Body(0x25E));
    set_female(w, id, true);
    let v1 = race_skin_hue(w, id);
    set_hue(w, id, v1);
    set_speed(w, id, 0.5, 2.0, true);
    init_stats(w, id, 100, 100, 25);
    assign_random_hair(w, id);
    set_skill_range(w, id, SkillName::Meditation, 60.0, 80.0);
    set_skill_range(w, id, SkillName::Focus, 60.0, 80.0);
    add_item_hue(w, id, "Sandals", Some(0x1BB));
    add_item_hue(w, id, "FemaleElvenRobe", Some(0x5A7));
    add_item_hue(w, id, "GemmedCirclet", None);
    add_item_hue(w, id, "MagicWand", None);
    id
}
creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Canir`.
    CANIR, CANIR_C = "Server.Engines.MLQuests.Definitions.Canir" : BASE_CREATURE / BASE_CREATURE_C, ctor canir_new, encoded false;
    c {
        is_invulnerable: |_w, _id| true,
        can_teach: |_w, _id| true,
        default_name: |_w, _id| Some("Canir"),
        can_shout: |_w, _id| true,
    }
    m {
    }
}
fn canir_shout(w: &mut World, id: MobileId, pm: MobileId) {
    let v1 = [1074193, 1074186][rnd::below(w, 2) as usize];
    quester::tell(w, Quester::Mobile(id), pm, v1);
}

fn yellienir_new(w: &mut World) -> MobileId {
    let id = new_creature(w, &YELLIENIR, AIType::AiVendor, FightMode::None, 2, 1);
    set_title(w, id, Some("the bark weaver".to_string()));
    set_race_named(w, id, "Elf");
    set_body(w, id, Body(0x25E));
    set_female(w, id, true);
    let v1 = race_skin_hue(w, id);
    set_hue(w, id, v1);
    set_speed(w, id, 0.5, 2.0, true);
    init_stats(w, id, 100, 100, 25);
    assign_random_hair(w, id);
    add_item_hue(w, id, "ElvenBoots", None);
    add_item_hue(w, id, "LeafTonlet", None);
    add_item_hue(w, id, "FemaleLeafChest", None);
    add_item_hue(w, id, "LeafArms", None);
    add_item_hue(w, id, "Cloak", Some(0x3B2));
    id
}
creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Yellienir`.
    YELLIENIR, YELLIENIR_C = "Server.Engines.MLQuests.Definitions.Yellienir" : BASE_CREATURE / BASE_CREATURE_C, ctor yellienir_new, encoded false;
    c {
        default_name: |_w, _id| Some("Yellienir"),
        is_invulnerable: |_w, _id| true,
    }
    m {
    }
}

fn elder_onallan_new(w: &mut World) -> MobileId {
    let id = new_creature(w, &ELDER_ONALLAN, AIType::AiVendor, FightMode::None, 2, 1);
    set_title(w, id, Some("the wise".to_string()));
    set_race_named(w, id, "Elf");
    set_body(w, id, Body(0x25D));
    set_female(w, id, false);
    let v1 = race_skin_hue(w, id);
    set_hue(w, id, v1);
    set_speed(w, id, 0.5, 2.0, true);
    init_stats(w, id, 100, 100, 25);
    assign_random_hair(w, id);
    set_skill_range(w, id, SkillName::Meditation, 60.0, 80.0);
    set_skill_range(w, id, SkillName::Focus, 60.0, 80.0);
    add_item_hue(w, id, "Shoes", Some(0x729));
    add_item_hue(w, id, "WildStaff", None);
    add_item_hue(w, id, "Cloak", Some(0x64E));
    id
}
creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.ElderOnallan`.
    ELDER_ONALLAN, ELDER_ONALLAN_C = "Server.Engines.MLQuests.Definitions.ElderOnallan" : BASE_CREATURE / BASE_CREATURE_C, ctor elder_onallan_new, encoded false;
    c {
        is_invulnerable: |_w, _id| true,
        can_teach: |_w, _id| true,
        default_name: |_w, _id| Some("Elder Onallan"),
        can_shout: |_w, _id| true,
    }
    m {
    }
}
fn elder_onallan_shout(w: &mut World, id: MobileId, pm: MobileId) {
    let v1 = [1074217, 1074218][rnd::below(w, 2) as usize];
    quester::tell(w, Quester::Mobile(id), pm, v1);
}

fn aeluva_new(w: &mut World) -> MobileId {
    let id = new_creature(w, &AELUVA, AIType::AiVendor, FightMode::None, 2, 1);
    set_title(w, id, Some("the arcanist".to_string()));
    set_race_named(w, id, "Elf");
    set_female(w, id, true);
    set_body(w, id, Body(606));
    let v1 = race_skin_hue(w, id);
    set_hue(w, id, v1);
    set_speed(w, id, 0.5, 2.0, true);
    init_stats(w, id, 100, 100, 25);
    assign_random_hair(w, id);
    set_skill_range(w, id, SkillName::Meditation, 60.0, 80.0);
    set_skill_range(w, id, SkillName::Focus, 60.0, 80.0);
    add_item_hue(w, id, "ElvenShirt", None);
    let v2 = utility::random_nondyed_hue(&mut w.rng);
    add_item_hue(w, id, "Kilt", Some(v2));
    add_item_hue(w, id, "ElvenBoots", None);
    add_item_hue(w, id, "Circlet", None);
    id
}
creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Aeluva`.
    AELUVA, AELUVA_C = "Server.Engines.MLQuests.Definitions.Aeluva" : BASE_CREATURE / BASE_CREATURE_C, ctor aeluva_new, encoded false;
    c {
        is_invulnerable: |_w, _id| true,
        can_teach: |_w, _id| true,
        default_name: |_w, _id| Some("Aeluva"),
        can_shout: |_w, _id| true,
    }
    m {
    }
}
fn aeluva_shout(w: &mut World, id: MobileId, pm: MobileId) {
    let v1 = rnd::from(w, 1074206, 2);
    quester::tell(w, Quester::Mobile(id), pm, v1);
}

fn koole_new(w: &mut World) -> MobileId {
    let id = new_creature(w, &KOOLE, AIType::AiVendor, FightMode::None, 2, 1);
    set_title(w, id, Some("the arcanist".to_string()));
    set_race_named(w, id, "Elf");
    set_body(w, id, Body(605));
    let v1 = race_skin_hue(w, id);
    set_hue(w, id, v1);
    set_speed(w, id, 0.5, 2.0, true);
    init_stats(w, id, 100, 100, 25);
    assign_random_hair(w, id);
    set_skill_range(w, id, SkillName::Meditation, 60.0, 80.0);
    set_skill_range(w, id, SkillName::Focus, 60.0, 80.0);
    let item = make_named(w, "LeafChest");
    set_item_hue(w, item, 443);
    add_item_id(w, id, item);
    let item = make_named(w, "LeafArms");
    set_item_hue(w, item, 443);
    add_item_id(w, id, item);
    add_item_hue(w, id, "LeafTonlet", None);
    let v2 = utility::random_animal_hue(&mut w.rng);
    add_item_hue(w, id, "ThighBoots", Some(v2));
    add_item_hue(w, id, "RoyalCirclet", None);
    id
}
creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Koole`.
    KOOLE, KOOLE_C = "Server.Engines.MLQuests.Definitions.Koole" : BASE_CREATURE / BASE_CREATURE_C, ctor koole_new, encoded false;
    c {
        is_invulnerable: |_w, _id| true,
        can_teach: |_w, _id| true,
        default_name: |_w, _id| Some("Koole"),
        can_shout: |_w, _id| true,
    }
    m {
    }
}
fn koole_shout(w: &mut World, id: MobileId, pm: MobileId) {
    let v1 = [1074186, 1074218][rnd::below(w, 2) as usize];
    quester::tell(w, Quester::Mobile(id), pm, v1);
}

fn synaeva_new(w: &mut World) -> MobileId {
    let id = new_creature(w, &SYNAEVA, AIType::AiVendor, FightMode::None, 2, 1);
    set_title(w, id, Some("the arcanist".to_string()));
    set_race_named(w, id, "Elf");
    set_female(w, id, true);
    set_body(w, id, Body(606));
    let v1 = race_skin_hue(w, id);
    set_hue(w, id, v1);
    set_speed(w, id, 0.5, 2.0, true);
    init_stats(w, id, 100, 100, 25);
    assign_random_hair(w, id);
    set_skill_range(w, id, SkillName::Meditation, 60.0, 80.0);
    set_skill_range(w, id, SkillName::Focus, 60.0, 80.0);
    let v2 = utility::random_green_hue(&mut w.rng);
    add_item_props(w, id, "RavenHelm", None, Some(v2));
    add_item_hue(w, id, "FemaleLeafChest", None);
    add_item_hue(w, id, "LeafArms", None);
    add_item_hue(w, id, "LeafTonlet", None);
    add_item_hue(w, id, "ElvenBoots", None);
    add_item_hue(w, id, "WildStaff", None);
    id
}
creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Synaeva`.
    SYNAEVA, SYNAEVA_C = "Server.Engines.MLQuests.Definitions.Synaeva" : BASE_CREATURE / BASE_CREATURE_C, ctor synaeva_new, encoded false;
    c {
        is_invulnerable: |_w, _id| true,
        can_teach: |_w, _id| true,
        default_name: |_w, _id| Some("Synaeva"),
        can_shout: |_w, _id| true,
    }
    m {
    }
}
fn synaeva_shout(w: &mut World, id: MobileId, pm: MobileId) {
    quester::tell(w, Quester::Mobile(id), pm, 1074223);
}

fn elder_brae_new(w: &mut World) -> MobileId {
    let id = new_creature(w, &ELDER_BRAE, AIType::AiVendor, FightMode::None, 2, 1);
    set_title(w, id, Some("the wise".to_string()));
    set_race_named(w, id, "Elf");
    set_female(w, id, true);
    set_body(w, id, Body(606));
    let v1 = race_skin_hue(w, id);
    set_hue(w, id, v1);
    set_speed(w, id, 0.5, 2.0, true);
    init_stats(w, id, 100, 100, 25);
    assign_random_hair(w, id);
    set_skill_range(w, id, SkillName::Meditation, 60.0, 80.0);
    set_skill_range(w, id, SkillName::Focus, 60.0, 80.0);
    add_item_hue(w, id, "GemmedCirclet", None);
    let v2 = utility::random_bright_hue(&mut w.rng);
    add_item_hue(w, id, "FemaleElvenRobe", Some(v2));
    let v3 = utility::random_animal_hue(&mut w.rng);
    add_item_hue(w, id, "ElvenBoots", Some(v3));
    id
}
creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.ElderBrae`.
    ELDER_BRAE, ELDER_BRAE_C = "Server.Engines.MLQuests.Definitions.ElderBrae" : BASE_CREATURE / BASE_CREATURE_C, ctor elder_brae_new, encoded false;
    c {
        is_invulnerable: |_w, _id| true,
        can_teach: |_w, _id| true,
        default_name: |_w, _id| Some("Elder Brae"),
        can_shout: |_w, _id| true,
    }
    m {
    }
}
fn elder_brae_shout(w: &mut World, id: MobileId, pm: MobileId) {
    let v1 = [1074215, 1074218][rnd::below(w, 2) as usize];
    quester::tell(w, Quester::Mobile(id), pm, v1);
}

fn lorekeeper_broolol_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &LOREKEEPER_BROOLOL,
        AIType::AiVendor,
        FightMode::None,
        2,
        1,
    );
    set_title(w, id, Some("the keeper of tradition".to_string()));
    set_race_named(w, id, "Elf");
    set_body(w, id, Body(0x25E));
    set_female(w, id, true);
    let v1 = race_skin_hue(w, id);
    set_hue(w, id, v1);
    set_speed(w, id, 0.5, 2.0, true);
    init_stats(w, id, 100, 100, 25);
    assign_random_hair(w, id);
    set_skill_range(w, id, SkillName::Meditation, 60.0, 80.0);
    set_skill_range(w, id, SkillName::Focus, 60.0, 80.0);
    add_item_hue(w, id, "ElvenBoots", Some(0x70D));
    add_item_hue(w, id, "FemaleElvenRobe", Some(0x3A));
    add_item_hue(w, id, "WildStaff", None);
    id
}
creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.LorekeeperBroolol`.
    LOREKEEPER_BROOLOL, LOREKEEPER_BROOLOL_C = "Server.Engines.MLQuests.Definitions.LorekeeperBroolol" : BASE_CREATURE / BASE_CREATURE_C, ctor lorekeeper_broolol_new, encoded false;
    c {
        is_invulnerable: |_w, _id| true,
        can_teach: |_w, _id| true,
        default_name: |_w, _id| Some("Lorekeeper Broolol"),
        can_shout: |_w, _id| true,
    }
    m {
    }
}
fn lorekeeper_broolol_shout(w: &mut World, id: MobileId, pm: MobileId) {
    quester::tell(w, Quester::Mobile(id), pm, 1074200);
}

fn emilio_new(w: &mut World) -> MobileId {
    let id = new_creature(w, &EMILIO, AIType::AiVendor, FightMode::None, 2, 1);
    set_title(w, id, Some("the Tortured Artist".to_string()));
    set_race_named(w, id, "Human");
    set_body(w, id, Body(0x190));
    set_female(w, id, false);
    let v1 = race_skin_hue(w, id);
    set_hue(w, id, v1);
    set_speed(w, id, 0.5, 2.0, true);
    init_stats(w, id, 100, 100, 25);
    assign_random_hair(w, id);
    add_item_hue(w, id, "Backpack", None);
    add_item_hue(w, id, "Sandals", Some(0x72B));
    add_item_hue(w, id, "LongPants", Some(0x525));
    add_item_hue(w, id, "FancyShirt", Some(0x53F));
    add_item_hue(w, id, "FloppyHat", Some(0x58C));
    add_item_hue(w, id, "BodySash", Some(0x1C));
    id
}
creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Emilio`.
    EMILIO, EMILIO_C = "Server.Engines.MLQuests.Definitions.Emilio" : BASE_CREATURE / BASE_CREATURE_C, ctor emilio_new, encoded false;
    c {
        default_name: |_w, _id| Some("Emilio"),
        is_invulnerable: |_w, _id| true,
    }
    m {
    }
}

fn thalia_new(w: &mut World) -> MobileId {
    let id = new_creature(w, &THALIA, AIType::AiVendor, FightMode::None, 2, 1);
    set_title(w, id, Some("the Bride".to_string()));
    set_race_named(w, id, "Human");
    set_body(w, id, Body(0x191));
    set_female(w, id, true);
    let v1 = race_skin_hue(w, id);
    set_hue(w, id, v1);
    set_speed(w, id, 0.5, 2.0, true);
    init_stats(w, id, 100, 100, 25);
    assign_random_hair(w, id);
    add_item_hue(w, id, "Backpack", None);
    add_item_hue(w, id, "Sandals", Some(0x8FD));
    add_item_hue(w, id, "FancyDress", Some(0x8FD));
    id
}
creature_leaf! {
    /// `Server.Engines.MLQuests.Definitions.Thalia`.
    THALIA, THALIA_C = "Server.Engines.MLQuests.Definitions.Thalia" : BASE_CREATURE / BASE_CREATURE_C, ctor thalia_new, encoded false;
    c {
        default_name: |_w, _id| Some("Thalia"),
        is_invulnerable: |_w, _id| true,
    }
    m {
    }
}

/// Registers the generated mobile classes.
pub fn register(types: &mut TypeRegistry) {
    types.register_mobile(&BEN, &[]);
    types.register_mobile(&FREDERIC, &[]);
    types.register_mobile(&LEON, &[]);
    types.register_mobile(&ANDROS, &[]);
    types.register_mobile(&KIA, &[]);
    types.register_mobile(&EMERILLO, &[]);
    types.register_mobile(&NYTHALIA, &[]);
    types.register_mobile(&JAMAL, &[]);
    types.register_mobile(&IOSEP, &[]);
    types.register_mobile(&AURELIA, &[]);
    types.register_mobile(&SKELETON_OF_SZANDOR, &[]);
    types.register_mobile(&SARIL, &[]);
    types.register_mobile(&CAILLA, &[]);
    types.register_mobile(&TAMM, &[]);
    types.register_mobile(&LANDY, &[]);
    types.register_mobile(&ALEJAHA, &[]);
    types.register_mobile(&MIELAN, &[]);
    types.register_mobile(&CIALA, &[]);
    types.register_mobile(&ANIEL, &[]);
    types.register_mobile(&AULAN, &[]);
    types.register_mobile(&BRINNAE, &[]);
    types.register_mobile(&CAELAS, &[]);
    types.register_mobile(&CLEHIN, &[]);
    types.register_mobile(&CLOORNE, &[]);
    types.register_mobile(&SALAENIH, &[]);
    types.register_mobile(&VILO, &[]);
    types.register_mobile(&THOLEF, &[]);
    types.register_mobile(&TILLANIL, &[]);
    types.register_mobile(&WAELIAN, &[]);
    types.register_mobile(&SLEEN, &[]);
    types.register_mobile(&UNOELIL, &[]);
    types.register_mobile(&ANOLLY, &[]);
    types.register_mobile(&JUSAE, &[]);
    types.register_mobile(&CILLITHA, &[]);
    types.register_mobile(&LOHN, &[]);
    types.register_mobile(&OLLA, &[]);
    types.register_mobile(&THALLARY, &[]);
    types.register_mobile(&AHIE, &[]);
    types.register_mobile(&TYEELOR, &[]);
    types.register_mobile(&ATHAILON, &[]);
    types.register_mobile(&ELDER_TAELLIA, &[]);
    types.register_mobile(&ELDER_MALLEW, &[]);
    types.register_mobile(&ELDER_ABBEIN, &[]);
    types.register_mobile(&ELDER_VICAIE, &[]);
    types.register_mobile(&ELDER_JOTHAN, &[]);
    types.register_mobile(&ELDER_ALETHANIAN, &[]);
    types.register_mobile(&REBINIL, &[]);
    types.register_mobile(&ALUNIOL, &[]);
    types.register_mobile(&OLAENI, &[]);
    types.register_mobile(&BOLAEVIN, &[]);
    types.register_mobile(&LOREKEEPER_ANEEN, &[]);
    types.register_mobile(&DAELAS, &[]);
    types.register_mobile(&ALELLE, &[]);
    types.register_mobile(&LOREKEEPER_NILLAEN, &[]);
    types.register_mobile(&LOREKEEPER_RYAL, &[]);
    types.register_mobile(&BRAEN, &[]);
    types.register_mobile(&ELDER_ACOB, &[]);
    types.register_mobile(&LOREKEEPER_CALENDOR, &[]);
    types.register_mobile(&LOREKEEPER_SIARRA, &[]);
    types.register_mobile(&ENIGMA, &[]);
    types.register_mobile(&SLEDGE, &[]);
    types.register_mobile(&PATRICUS, &[]);
    types.register_mobile(&BELULAH, &[]);
    types.register_mobile(&EVAN, &[]);
    types.register_mobile(&REGINA, &[]);
    types.register_mobile(&GRANDPA_CHARLEY, &[]);
    types.register_mobile(&JELRICE, &[]);
    types.register_mobile(&YORUS, &[]);
    types.register_mobile(&DRITHEN, &[]);
    types.register_mobile(&AERNYA, &[]);
    types.register_mobile(&GORROW, &[]);
    types.register_mobile(&MASTER_GNOSOS, &[]);
    types.register_mobile(&DIMETHRO, &[]);
    types.register_mobile(&CHURCHILL, &[]);
    types.register_mobile(&RECARO, &[]);
    types.register_mobile(&ALDEN_ARMSTRONG, &[]);
    types.register_mobile(&JOCKLES, &[]);
    types.register_mobile(&TYL_ARIADNE, &[]);
    types.register_mobile(&ALEFIAN, &[]);
    types.register_mobile(&GUSTAR, &[]);
    types.register_mobile(&JILLIAN, &[]);
    types.register_mobile(&KAELYNNA, &[]);
    types.register_mobile(&MITHNERAL, &[]);
    types.register_mobile(&ANDREAS_VESALIUS, &[]);
    types.register_mobile(&AVICENNA, &[]);
    types.register_mobile(&SARSMEA_SMYTHE, &[]);
    types.register_mobile(&CHIYO, &[]);
    types.register_mobile(&JUN, &[]);
    types.register_mobile(&WALKER, &[]);
    types.register_mobile(&JACOB_WALTZ, &[]);
    types.register_mobile(&ANDRIC, &[]);
    types.register_mobile(&KASHIEL, &[]);
    types.register_mobile(&ASANDOS, &[]);
    types.register_mobile(&CLAIRESSE, &[]);
    types.register_mobile(&GERVIS, &[]);
    types.register_mobile(&MUGG, &[]);
    types.register_mobile(&LOWEL, &[]);
    types.register_mobile(&LYLE, &[]);
    types.register_mobile(&NIBBET, &[]);
    types.register_mobile(&NORTON, &[]);
    types.register_mobile(&SADRAH, &[]);
    types.register_mobile(&HARGROVE, &[]);
    types.register_mobile(&BEOTHAM, &[]);
    types.register_mobile(&DANOEL, &[]);
    types.register_mobile(&TALLININ, &[]);
    types.register_mobile(&TIANA, &[]);
    types.register_mobile(&LOREKEEPER_OOLUA, &[]);
    types.register_mobile(&LOREKEEPER_ROLLARN, &[]);
    types.register_mobile(&DALLID, &[]);
    types.register_mobile(&CANIR, &[]);
    types.register_mobile(&YELLIENIR, &[]);
    types.register_mobile(&ELDER_ONALLAN, &[]);
    types.register_mobile(&AELUVA, &[]);
    types.register_mobile(&KOOLE, &[]);
    types.register_mobile(&SYNAEVA, &[]);
    types.register_mobile(&ELDER_BRAE, &[]);
    types.register_mobile(&LOREKEEPER_BROOLOL, &[]);
    types.register_mobile(&EMILIO, &[]);
    types.register_mobile(&THALIA, &[]);
}

/// `Shout(pm)` overrides: class full name -> function.
#[allow(clippy::type_complexity)] // the callback type, spelled out
pub static SHOUTS: &[(&str, fn(&mut World, MobileId, MobileId))] = &[
    (
        "Server.Engines.MLQuests.Definitions.Emerillo",
        emerillo_shout,
    ),
    ("Server.Engines.MLQuests.Definitions.Iosep", iosep_shout),
    ("Server.Engines.MLQuests.Definitions.Saril", saril_shout),
    ("Server.Engines.MLQuests.Definitions.Cailla", cailla_shout),
    ("Server.Engines.MLQuests.Definitions.Tamm", tamm_shout),
    ("Server.Engines.MLQuests.Definitions.Landy", landy_shout),
    ("Server.Engines.MLQuests.Definitions.Alejaha", alejaha_shout),
    ("Server.Engines.MLQuests.Definitions.Mielan", mielan_shout),
    ("Server.Engines.MLQuests.Definitions.Ciala", ciala_shout),
    ("Server.Engines.MLQuests.Definitions.Aniel", aniel_shout),
    ("Server.Engines.MLQuests.Definitions.Aulan", aulan_shout),
    ("Server.Engines.MLQuests.Definitions.Brinnae", brinnae_shout),
    ("Server.Engines.MLQuests.Definitions.Caelas", caelas_shout),
    ("Server.Engines.MLQuests.Definitions.Clehin", clehin_shout),
    ("Server.Engines.MLQuests.Definitions.Cloorne", cloorne_shout),
    (
        "Server.Engines.MLQuests.Definitions.Salaenih",
        salaenih_shout,
    ),
    ("Server.Engines.MLQuests.Definitions.Vilo", vilo_shout),
    ("Server.Engines.MLQuests.Definitions.Tholef", tholef_shout),
    (
        "Server.Engines.MLQuests.Definitions.Tillanil",
        tillanil_shout,
    ),
    ("Server.Engines.MLQuests.Definitions.Waelian", waelian_shout),
    ("Server.Engines.MLQuests.Definitions.Sleen", sleen_shout),
    ("Server.Engines.MLQuests.Definitions.Unoelil", unoelil_shout),
    ("Server.Engines.MLQuests.Definitions.Jusae", jusae_shout),
    (
        "Server.Engines.MLQuests.Definitions.Cillitha",
        cillitha_shout,
    ),
    ("Server.Engines.MLQuests.Definitions.Lohn", lohn_shout),
    ("Server.Engines.MLQuests.Definitions.Olla", olla_shout),
    (
        "Server.Engines.MLQuests.Definitions.Thallary",
        thallary_shout,
    ),
    ("Server.Engines.MLQuests.Definitions.Ahie", ahie_shout),
    (
        "Server.Engines.MLQuests.Definitions.LorekeeperRyal",
        lorekeeper_ryal_shout,
    ),
    ("Server.Engines.MLQuests.Definitions.Braen", braen_shout),
    (
        "Server.Engines.MLQuests.Definitions.ElderAcob",
        elder_acob_shout,
    ),
    (
        "Server.Engines.MLQuests.Definitions.LorekeeperCalendor",
        lorekeeper_calendor_shout,
    ),
    (
        "Server.Engines.MLQuests.Definitions.LorekeeperSiarra",
        lorekeeper_siarra_shout,
    ),
    ("Server.Engines.MLQuests.Definitions.Sledge", sledge_shout),
    ("Server.Engines.MLQuests.Definitions.Belulah", belulah_shout),
    ("Server.Engines.MLQuests.Definitions.Jelrice", jelrice_shout),
    ("Server.Engines.MLQuests.Definitions.Yorus", yorus_shout),
    ("Server.Engines.MLQuests.Definitions.Drithen", drithen_shout),
    ("Server.Engines.MLQuests.Definitions.Gorrow", gorrow_shout),
    (
        "Server.Engines.MLQuests.Definitions.MasterGnosos",
        master_gnosos_shout,
    ),
    (
        "Server.Engines.MLQuests.Definitions.Dimethro",
        dimethro_shout,
    ),
    (
        "Server.Engines.MLQuests.Definitions.Churchill",
        churchill_shout,
    ),
    ("Server.Engines.MLQuests.Definitions.Recaro", recaro_shout),
    (
        "Server.Engines.MLQuests.Definitions.AldenArmstrong",
        alden_armstrong_shout,
    ),
    ("Server.Engines.MLQuests.Definitions.Jockles", jockles_shout),
    (
        "Server.Engines.MLQuests.Definitions.TylAriadne",
        tyl_ariadne_shout,
    ),
    ("Server.Engines.MLQuests.Definitions.Alefian", alefian_shout),
    ("Server.Engines.MLQuests.Definitions.Gustar", gustar_shout),
    ("Server.Engines.MLQuests.Definitions.Jillian", jillian_shout),
    (
        "Server.Engines.MLQuests.Definitions.Kaelynna",
        kaelynna_shout,
    ),
    (
        "Server.Engines.MLQuests.Definitions.Mithneral",
        mithneral_shout,
    ),
    (
        "Server.Engines.MLQuests.Definitions.AndreasVesalius",
        andreas_vesalius_shout,
    ),
    (
        "Server.Engines.MLQuests.Definitions.Avicenna",
        avicenna_shout,
    ),
    (
        "Server.Engines.MLQuests.Definitions.SarsmeaSmythe",
        sarsmea_smythe_shout,
    ),
    ("Server.Engines.MLQuests.Definitions.Chiyo", chiyo_shout),
    ("Server.Engines.MLQuests.Definitions.Jun", jun_shout),
    ("Server.Engines.MLQuests.Definitions.Walker", walker_shout),
    (
        "Server.Engines.MLQuests.Definitions.JacobWaltz",
        jacob_waltz_shout,
    ),
    ("Server.Engines.MLQuests.Definitions.Andric", andric_shout),
    ("Server.Engines.MLQuests.Definitions.Asandos", asandos_shout),
    (
        "Server.Engines.MLQuests.Definitions.Clairesse",
        clairesse_shout,
    ),
    ("Server.Engines.MLQuests.Definitions.Gervis", gervis_shout),
    ("Server.Engines.MLQuests.Definitions.Mugg", mugg_shout),
    ("Server.Engines.MLQuests.Definitions.Lowel", lowel_shout),
    ("Server.Engines.MLQuests.Definitions.Lyle", lyle_shout),
    ("Server.Engines.MLQuests.Definitions.Norton", norton_shout),
    ("Server.Engines.MLQuests.Definitions.Sadrah", sadrah_shout),
    (
        "Server.Engines.MLQuests.Definitions.Hargrove",
        hargrove_shout,
    ),
    ("Server.Engines.MLQuests.Definitions.Beotham", beotham_shout),
    ("Server.Engines.MLQuests.Definitions.Danoel", danoel_shout),
    (
        "Server.Engines.MLQuests.Definitions.Tallinin",
        tallinin_shout,
    ),
    ("Server.Engines.MLQuests.Definitions.Tiana", tiana_shout),
    (
        "Server.Engines.MLQuests.Definitions.LorekeeperOolua",
        lorekeeper_oolua_shout,
    ),
    (
        "Server.Engines.MLQuests.Definitions.LorekeeperRollarn",
        lorekeeper_rollarn_shout,
    ),
    ("Server.Engines.MLQuests.Definitions.Dallid", dallid_shout),
    ("Server.Engines.MLQuests.Definitions.Canir", canir_shout),
    (
        "Server.Engines.MLQuests.Definitions.ElderOnallan",
        elder_onallan_shout,
    ),
    ("Server.Engines.MLQuests.Definitions.Aeluva", aeluva_shout),
    ("Server.Engines.MLQuests.Definitions.Koole", koole_shout),
    ("Server.Engines.MLQuests.Definitions.Synaeva", synaeva_shout),
    (
        "Server.Engines.MLQuests.Definitions.ElderBrae",
        elder_brae_shout,
    ),
    (
        "Server.Engines.MLQuests.Definitions.LorekeeperBroolol",
        lorekeeper_broolol_shout,
    ),
];
