//! The constructors of the humanoid hireables and townfolk, as table classes.

use super::humans_support::*;
use crate::common::*;
use muo_content::creature::{BASE_CREATURE, BASE_CREATURE_C};
use muo_content::hooks::rnd;
use muo_content::loot_pack;
use muo_primitives::utility;
use muo_server::ids::MobileId;
use muo_server::layer::Layer;
use muo_server::mobile::part2::items;
use muo_server::type_registry::TypeRegistry;
use muo_server::world::World;
fn hire_bard_new(w: &mut World) -> MobileId {
    let id = new_hire(w, &HIRE_BARD, AIType::AiMelee, false);
    let v = utility::random_dyed_hue(&mut w.rng);
    speech_hue(w, id, v);
    let skin = muo_content::vendor::human_random_skin_hue(w);
    hue(w, id, skin);
    title(w, id, "the bard");
    muo_content::vendor::roll_hair_item(w, id);
    muo_content::vendor::roll_hair_hue(w, id);
    muo_content::vendor::roll_facial_hair(w, id);
    set_str_range(w, id, 16, 16);
    set_dex_range(w, id, 26, 26);
    set_int_range(w, id, 26, 26);
    set_damage(w, id, 5, 10);
    set_skill_range(w, id, SkillName::Tactics, 35.0, 57.0);
    set_skill_range(w, id, SkillName::Magery, 22.0, 22.0);
    set_skill_range(w, id, SkillName::Swords, 45.0, 67.0);
    set_skill_range(w, id, SkillName::Archery, 36.0, 67.0);
    set_skill_range(w, id, SkillName::Parry, 45.0, 60.0);
    set_skill_range(w, id, SkillName::Musicianship, 66.0, 97.5);
    set_skill_range(w, id, SkillName::Peacemaking, 65.0, 87.5);
    fame(w, id, 100);
    karma(w, id, 100);
    let fem = rnd::bool(w);
    muo_server::mobile::part1::props_b::set_female(w, id, fem);
    if fem {
        body(w, id, 0x191);
        name_random(w, id, "female");
        match rnd::below(w, 2) {
            0 => {
                let h = utility::random_dyed_hue(&mut w.rng);
                equip_new(w, id, "Server.Items.Skirt", Some(h));
            }
            1 => {
                let h = utility::random_neutral_hue(&mut w.rng);
                equip_new(w, id, "Server.Items.Kilt", Some(h));
            }
            _ => {}
        }
    } else {
        body(w, id, 0x190);
        name_random(w, id, "male");
        let h = utility::random_neutral_hue(&mut w.rng);
        equip_new(w, id, "Server.Items.ShortPants", Some(h));
    }
    let h = utility::random_neutral_hue(&mut w.rng);
    equip_new(w, id, "Server.Items.Shoes", Some(h));
    match rnd::below(w, 2) {
        0 => {
            let h = utility::random_dyed_hue(&mut w.rng);
            equip_new(w, id, "Server.Items.Doublet", Some(h));
        }
        1 => {
            let h = utility::random_dyed_hue(&mut w.rng);
            equip_new(w, id, "Server.Items.Shirt", Some(h));
        }
        _ => {}
    }
    id
}
fn hire_bard_generate_loot(w: &mut World, id: MobileId) {
    add_loot(w, id, loot_pack::average(w));
    if rnd::bool(w) {
        let it = muo_content::loot::random_instrument(w);
        pack_item(w, id, it);
    }
    match rnd::below(w, 2) {
        0 => {
            pack_new(w, id, "Server.Items.Longsword");
        }
        1 => {
            pack_new(w, id, "Server.Items.Bow");
            pack_new_amount(w, id, "Server.Items.Arrow", 100);
        }
        _ => {}
    }
    pack_gold_range(w, id, 10, 50);
}
creature_leaf! {
    HIRE_BARD, HIRE_BARD_C = "Server.Mobiles.HireBard" : BASE_HIRE / BASE_HIRE_C, ctor Some(hire_bard_new), encoded true;
    c {
        generate_loot: hire_bard_generate_loot,
    }
    m {
        click_title: |_w, _| false,
    }
}

fn hire_bard_archer_new(w: &mut World) -> MobileId {
    let id = new_hire(w, &HIRE_BARD_ARCHER, AIType::AiArcher, true);
    let v = utility::random_dyed_hue(&mut w.rng);
    speech_hue(w, id, v);
    let skin = muo_content::vendor::human_random_skin_hue(w);
    hue(w, id, skin);
    title(w, id, "the bard");
    muo_content::vendor::roll_hair_item(w, id);
    muo_content::vendor::roll_hair_hue(w, id);
    muo_content::vendor::roll_facial_hair(w, id);
    set_str_range(w, id, 16, 16);
    set_dex_range(w, id, 26, 26);
    set_int_range(w, id, 26, 26);
    set_damage(w, id, 5, 10);
    set_skill_range(w, id, SkillName::Tactics, 35.0, 57.0);
    set_skill_range(w, id, SkillName::Magery, 22.0, 22.0);
    set_skill_range(w, id, SkillName::Swords, 45.0, 67.0);
    set_skill_range(w, id, SkillName::Archery, 36.0, 67.0);
    set_skill_range(w, id, SkillName::Parry, 45.0, 60.0);
    set_skill_range(w, id, SkillName::Musicianship, 66.0, 97.5);
    set_skill_range(w, id, SkillName::Peacemaking, 65.0, 87.5);
    fame(w, id, 100);
    karma(w, id, 100);
    let fem = rnd::bool(w);
    muo_server::mobile::part1::props_b::set_female(w, id, fem);
    if fem {
        body(w, id, 0x191);
        name_random(w, id, "female");
        match rnd::below(w, 2) {
            0 => {
                let h = utility::random_dyed_hue(&mut w.rng);
                equip_new(w, id, "Server.Items.Skirt", Some(h));
            }
            1 => {
                let h = utility::random_neutral_hue(&mut w.rng);
                equip_new(w, id, "Server.Items.Kilt", Some(h));
            }
            _ => {}
        }
    } else {
        body(w, id, 0x190);
        name_random(w, id, "male");
        let h = utility::random_neutral_hue(&mut w.rng);
        equip_new(w, id, "Server.Items.ShortPants", Some(h));
    }
    let h = utility::random_neutral_hue(&mut w.rng);
    equip_new(w, id, "Server.Items.Shoes", Some(h));
    match rnd::below(w, 2) {
        0 => {
            let h = utility::random_dyed_hue(&mut w.rng);
            equip_new(w, id, "Server.Items.Doublet", Some(h));
        }
        1 => {
            let h = utility::random_dyed_hue(&mut w.rng);
            equip_new(w, id, "Server.Items.Shirt", Some(h));
        }
        _ => {}
    }
    id
}
fn hire_bard_archer_generate_loot(w: &mut World, id: MobileId) {
    add_loot(w, id, loot_pack::average(w));
    if rnd::bool(w) {
        let it = muo_content::loot::random_instrument(w);
        pack_item(w, id, it);
    }
    match rnd::below(w, 2) {
        0 => {
            pack_new(w, id, "Server.Items.Longsword");
        }
        1 => {
            pack_new(w, id, "Server.Items.Bow");
            pack_new_amount(w, id, "Server.Items.Arrow", 100);
        }
        _ => {}
    }
    pack_gold_range(w, id, 10, 50);
}
creature_leaf! {
    HIRE_BARD_ARCHER, HIRE_BARD_ARCHER_C = "Server.Mobiles.HireBardArcher" : BASE_HIRE / BASE_HIRE_C, ctor Some(hire_bard_archer_new), encoded true;
    c {
        generate_loot: hire_bard_archer_generate_loot,
    }
    m {
        click_title: |_w, _| false,
    }
}

fn hire_beggar_new(w: &mut World) -> MobileId {
    let id = new_hire(w, &HIRE_BEGGAR, AIType::AiMelee, false);
    let v = utility::random_dyed_hue(&mut w.rng);
    speech_hue(w, id, v);
    let skin = muo_content::vendor::human_random_skin_hue(w);
    hue(w, id, skin);
    title(w, id, "the beggar");
    muo_content::vendor::roll_hair_item(w, id);
    muo_content::vendor::roll_hair_hue(w, id);
    muo_content::vendor::roll_facial_hair(w, id);
    set_str_range(w, id, 26, 26);
    set_dex_range(w, id, 21, 21);
    set_int_range(w, id, 36, 36);
    set_damage(w, id, 1, 1);
    set_skill_range(w, id, SkillName::Begging, 66.0, 97.0);
    set_skill_range(w, id, SkillName::Tactics, 5.0, 27.0);
    set_skill_range(w, id, SkillName::Wrestling, 5.0, 27.0);
    set_skill_range(w, id, SkillName::Magery, 2.0, 2.0);
    fame(w, id, 0);
    karma(w, id, 0);
    let fem = rnd::bool(w);
    muo_server::mobile::part1::props_b::set_female(w, id, fem);
    if fem {
        body(w, id, 0x191);
        name_random(w, id, "female");
        match rnd::below(w, 2) {
            0 => {
                let h = utility::random_neutral_hue(&mut w.rng);
                equip_new(w, id, "Server.Items.Skirt", Some(h));
            }
            1 => {
                let h = utility::random_neutral_hue(&mut w.rng);
                equip_new(w, id, "Server.Items.Kilt", Some(h));
            }
            _ => {}
        }
    } else {
        body(w, id, 0x190);
        name_random(w, id, "male");
        let h = utility::random_neutral_hue(&mut w.rng);
        equip_new(w, id, "Server.Items.ShortPants", Some(h));
    }
    let h = utility::random_neutral_hue(&mut w.rng);
    equip_new(w, id, "Server.Items.Sandals", Some(h));
    match rnd::below(w, 2) {
        0 => {
            let h = utility::random_neutral_hue(&mut w.rng);
            equip_new(w, id, "Server.Items.Doublet", Some(h));
        }
        1 => {
            let h = utility::random_neutral_hue(&mut w.rng);
            equip_new(w, id, "Server.Items.Shirt", Some(h));
        }
        _ => {}
    }
    pack_gold_range(w, id, 0, 25);
    id
}
creature_leaf! {
    HIRE_BEGGAR, HIRE_BEGGAR_C = "Server.Mobiles.HireBeggar" : BASE_HIRE / BASE_HIRE_C, ctor Some(hire_beggar_new), encoded true;
    c {
    }
    m {
        click_title: |_w, _| false,
    }
}

fn hire_fighter_new(w: &mut World) -> MobileId {
    let id = new_hire(w, &HIRE_FIGHTER, AIType::AiMelee, false);
    let v = utility::random_dyed_hue(&mut w.rng);
    speech_hue(w, id, v);
    let skin = muo_content::vendor::human_random_skin_hue(w);
    hue(w, id, skin);
    title(w, id, "the fighter");
    muo_content::vendor::roll_hair_item(w, id);
    muo_content::vendor::roll_hair_hue(w, id);
    muo_content::vendor::roll_facial_hair(w, id);
    set_str_range(w, id, 91, 91);
    set_dex_range(w, id, 91, 91);
    set_int_range(w, id, 50, 50);
    set_damage(w, id, 7, 14);
    set_skill_range(w, id, SkillName::Tactics, 36.0, 67.0);
    set_skill_range(w, id, SkillName::Magery, 22.0, 22.0);
    set_skill_range(w, id, SkillName::Swords, 64.0, 100.0);
    set_skill_range(w, id, SkillName::Parry, 60.0, 82.0);
    set_skill_range(w, id, SkillName::Macing, 36.0, 67.0);
    set_skill_range(w, id, SkillName::Focus, 36.0, 67.0);
    set_skill_range(w, id, SkillName::Wrestling, 25.0, 47.0);
    fame(w, id, 100);
    karma(w, id, 100);
    let fem = rnd::bool(w);
    muo_server::mobile::part1::props_b::set_female(w, id, fem);
    if fem {
        body(w, id, 0x191);
        name_random(w, id, "female");
    } else {
        body(w, id, 0x190);
        name_random(w, id, "male");
    }
    match rnd::below(w, 2) {
        0 => {
            let h = utility::random_neutral_hue(&mut w.rng);
            equip_new(w, id, "Server.Items.Shoes", Some(h));
        }
        1 => {
            let h = utility::random_neutral_hue(&mut w.rng);
            equip_new(w, id, "Server.Items.Boots", Some(h));
        }
        _ => {}
    }
    equip_new(w, id, "Server.Items.Shirt", None);
    let pick: Option<&str> = match rnd::below(w, 5) {
        1 => Some("Server.Items.Broadsword"),
        2 => Some("Server.Items.VikingSword"),
        3 => Some("Server.Items.BattleAxe"),
        4 => Some("Server.Items.TwoHandedAxe"),
        _ => Some("Server.Items.Longsword"),
    };
    let pick_weapon = pick;
    equip_pick(w, id, pick_weapon);
    if items::find_item_on_layer(w, id, Layer::TwoHanded).is_none() {
        let pick: Option<&str> = match rnd::below(w, 6) {
            1 => Some("Server.Items.HeaterShield"),
            2 => Some("Server.Items.MetalKiteShield"),
            3 => Some("Server.Items.MetalShield"),
            4 => Some("Server.Items.WoodenKiteShield"),
            5 => Some("Server.Items.WoodenShield"),
            _ => Some("Server.Items.BronzeShield"),
        };
        let pick_shield = pick;
        equip_pick(w, id, pick_shield);
    }
    let pick: Option<&str> = match rnd::below(w, 5) {
        1 => Some("Server.Items.Bascinet"),
        2 => Some("Server.Items.CloseHelm"),
        3 => Some("Server.Items.NorseHelm"),
        4 => Some("Server.Items.Helmet"),
        _ => None,
    };
    let pick_helm = pick;
    equip_pick(w, id, pick_helm);
    match rnd::below(w, 4) {
        0 => {
            equip_new(w, id, "Server.Items.LeatherChest", None);
            equip_new(w, id, "Server.Items.LeatherArms", None);
            equip_new(w, id, "Server.Items.LeatherGloves", None);
            equip_new(w, id, "Server.Items.LeatherGorget", None);
            equip_new(w, id, "Server.Items.LeatherLegs", None);
        }
        1 => {
            equip_new(w, id, "Server.Items.StuddedChest", None);
            equip_new(w, id, "Server.Items.StuddedArms", None);
            equip_new(w, id, "Server.Items.StuddedGloves", None);
            equip_new(w, id, "Server.Items.StuddedGorget", None);
            equip_new(w, id, "Server.Items.StuddedLegs", None);
        }
        2 => {
            equip_new(w, id, "Server.Items.RingmailChest", None);
            equip_new(w, id, "Server.Items.RingmailArms", None);
            equip_new(w, id, "Server.Items.RingmailGloves", None);
            equip_new(w, id, "Server.Items.RingmailLegs", None);
        }
        3 => {
            equip_new(w, id, "Server.Items.ChainChest", None);
            equip_new(w, id, "Server.Items.ChainLegs", None);
        }
        _ => {}
    }
    pack_gold_range(w, id, 25, 100);
    id
}
creature_leaf! {
    HIRE_FIGHTER, HIRE_FIGHTER_C = "Server.Mobiles.HireFighter" : BASE_HIRE / BASE_HIRE_C, ctor Some(hire_fighter_new), encoded true;
    c {
    }
    m {
        click_title: |_w, _| false,
    }
}

fn hire_mage_new(w: &mut World) -> MobileId {
    let id = new_hire(w, &HIRE_MAGE, AIType::AiMage, true);
    let v = utility::random_dyed_hue(&mut w.rng);
    speech_hue(w, id, v);
    let skin = muo_content::vendor::human_random_skin_hue(w);
    hue(w, id, skin);
    title(w, id, "the mage");
    muo_content::vendor::roll_hair_item(w, id);
    muo_content::vendor::roll_hair_hue(w, id);
    muo_content::vendor::roll_facial_hair(w, id);
    set_str_range(w, id, 61, 75);
    set_dex_range(w, id, 81, 95);
    set_int_range(w, id, 86, 100);
    set_damage(w, id, 10, 23);
    set_skill_range(w, id, SkillName::EvalInt, 100.0, 125.0);
    set_skill_range(w, id, SkillName::Magery, 100.0, 125.0);
    set_skill_range(w, id, SkillName::Meditation, 100.0, 125.0);
    set_skill_range(w, id, SkillName::MagicResist, 100.0, 125.0);
    set_skill_range(w, id, SkillName::Tactics, 100.0, 125.0);
    set_skill_range(w, id, SkillName::Macing, 100.0, 125.0);
    fame(w, id, 100);
    karma(w, id, 100);
    let fem = rnd::bool(w);
    muo_server::mobile::part1::props_b::set_female(w, id, fem);
    if fem {
        body(w, id, 0x191);
        name_random(w, id, "female");
    } else {
        body(w, id, 0x190);
        name_random(w, id, "male");
        let h = utility::random_neutral_hue(&mut w.rng);
        equip_new(w, id, "Server.Items.ShortPants", Some(h));
    }
    equip_new(w, id, "Server.Items.Shirt", None);
    let h = utility::random_neutral_hue(&mut w.rng);
    equip_new(w, id, "Server.Items.Robe", Some(h));
    if rnd::bool(w) {
        let h = utility::random_neutral_hue(&mut w.rng);
        equip_new(w, id, "Server.Items.Shoes", Some(h));
    } else {
        equip_new(w, id, "Server.Items.ThighBoots", None);
    }
    pack_gold_range(w, id, 20, 100);
    id
}
creature_leaf! {
    HIRE_MAGE, HIRE_MAGE_C = "Server.Mobiles.HireMage" : BASE_HIRE / BASE_HIRE_C, ctor Some(hire_mage_new), encoded true;
    c {
    }
    m {
        click_title: |_w, _| false,
    }
}

fn hire_paladin_new(w: &mut World) -> MobileId {
    let id = new_hire(w, &HIRE_PALADIN, AIType::AiMelee, false);
    let v = utility::random_dyed_hue(&mut w.rng);
    speech_hue(w, id, v);
    let skin = muo_content::vendor::human_random_skin_hue(w);
    hue(w, id, skin);
    title(w, id, "the paladin");
    muo_content::vendor::roll_hair_item(w, id);
    muo_content::vendor::roll_hair_hue(w, id);
    muo_content::vendor::roll_facial_hair(w, id);
    set_str_range(w, id, if w.core_aos() { 100 } else { 86 }, 100);
    set_dex_range(w, id, 81, 95);
    set_int_range(w, id, 61, 75);
    set_damage(w, id, 10, 23);
    set_skill_range(w, id, SkillName::Swords, 66.0, 97.5);
    set_skill_range(w, id, SkillName::Anatomy, 65.0, 87.5);
    set_skill_range(w, id, SkillName::MagicResist, 25.0, 47.5);
    set_skill_range(w, id, SkillName::Healing, 65.0, 87.5);
    set_skill_range(w, id, SkillName::Tactics, 65.0, 87.5);
    set_skill_range(w, id, SkillName::Wrestling, 15.0, 37.5);
    set_skill_range(w, id, SkillName::Parry, 45.0, 60.5);
    set_skill_range(w, id, SkillName::Chivalry, 85.0, 100.0);
    fame(w, id, 100);
    karma(w, id, 250);
    let fem = rnd::bool(w);
    muo_server::mobile::part1::props_b::set_female(w, id, fem);
    if fem {
        body(w, id, 0x191);
        name_random(w, id, "female");
    } else {
        body(w, id, 0x190);
        name_random(w, id, "male");
    }
    let pick: Option<&str> = match rnd::below(w, 5) {
        1 => Some("Server.Items.Bascinet"),
        2 => Some("Server.Items.CloseHelm"),
        3 => Some("Server.Items.NorseHelm"),
        4 => Some("Server.Items.Helmet"),
        _ => None,
    };
    let pick_helm = pick;
    equip_pick(w, id, pick_helm);
    let h = utility::random_neutral_hue(&mut w.rng);
    equip_new(w, id, "Server.Items.Shoes", Some(h));
    equip_new(w, id, "Server.Items.Shirt", None);
    equip_new(w, id, "Server.Items.VikingSword", None);
    equip_new(w, id, "Server.Items.MetalKiteShield", None);
    equip_new(w, id, "Server.Items.PlateChest", None);
    equip_new(w, id, "Server.Items.PlateLegs", None);
    equip_new(w, id, "Server.Items.PlateArms", None);
    equip_new(w, id, "Server.Items.LeatherGorget", None);
    pack_gold_range(w, id, 20, 100);
    id
}
creature_leaf! {
    HIRE_PALADIN, HIRE_PALADIN_C = "Server.Mobiles.HirePaladin" : BASE_HIRE / BASE_HIRE_C, ctor Some(hire_paladin_new), encoded true;
    c {
    }
    m {
        click_title: |_w, _| false,
    }
}

fn hire_peasant_new(w: &mut World) -> MobileId {
    let id = new_hire(w, &HIRE_PEASANT, AIType::AiMelee, false);
    let v = utility::random_dyed_hue(&mut w.rng);
    speech_hue(w, id, v);
    let skin = muo_content::vendor::human_random_skin_hue(w);
    hue(w, id, skin);
    title(w, id, "the peasant");
    muo_content::vendor::roll_hair_item(w, id);
    muo_content::vendor::roll_hair_hue(w, id);
    muo_content::vendor::roll_facial_hair(w, id);
    set_str_range(w, id, 26, 26);
    set_dex_range(w, id, 21, 21);
    set_int_range(w, id, 16, 16);
    set_damage(w, id, 10, 23);
    set_skill_range(w, id, SkillName::Tactics, 5.0, 27.0);
    set_skill_range(w, id, SkillName::Wrestling, 5.0, 5.0);
    set_skill_range(w, id, SkillName::Swords, 5.0, 27.0);
    fame(w, id, 0);
    karma(w, id, 0);
    let fem = rnd::bool(w);
    muo_server::mobile::part1::props_b::set_female(w, id, fem);
    if fem {
        body(w, id, 0x191);
        name_random(w, id, "female");
        match rnd::below(w, 2) {
            0 => {
                let h = utility::random_neutral_hue(&mut w.rng);
                equip_new(w, id, "Server.Items.Skirt", Some(h));
            }
            1 => {
                let h = utility::random_neutral_hue(&mut w.rng);
                equip_new(w, id, "Server.Items.Kilt", Some(h));
            }
            _ => {}
        }
    } else {
        body(w, id, 0x190);
        name_random(w, id, "male");
        let h = utility::random_neutral_hue(&mut w.rng);
        equip_new(w, id, "Server.Items.ShortPants", Some(h));
    }
    equip_new(w, id, "Server.Items.Katana", None);
    let h = utility::random_neutral_hue(&mut w.rng);
    equip_new(w, id, "Server.Items.Sandals", Some(h));
    match rnd::below(w, 2) {
        0 => {
            let h = utility::random_neutral_hue(&mut w.rng);
            equip_new(w, id, "Server.Items.Doublet", Some(h));
        }
        1 => {
            let h = utility::random_neutral_hue(&mut w.rng);
            equip_new(w, id, "Server.Items.Shirt", Some(h));
        }
        _ => {}
    }
    pack_gold_range(w, id, 0, 25);
    id
}
creature_leaf! {
    HIRE_PEASANT, HIRE_PEASANT_C = "Server.Mobiles.HirePeasant" : BASE_HIRE / BASE_HIRE_C, ctor Some(hire_peasant_new), encoded true;
    c {
    }
    m {
        click_title: |_w, _| false,
    }
}

fn hire_ranger_new(w: &mut World) -> MobileId {
    let id = new_hire(w, &HIRE_RANGER, AIType::AiMelee, false);
    let v = utility::random_dyed_hue(&mut w.rng);
    speech_hue(w, id, v);
    let skin = muo_content::vendor::human_random_skin_hue(w);
    hue(w, id, skin);
    title(w, id, "the ranger");
    muo_content::vendor::roll_hair_item(w, id);
    muo_content::vendor::roll_hair_hue(w, id);
    muo_content::vendor::roll_facial_hair(w, id);
    set_str_range(w, id, 91, 91);
    set_dex_range(w, id, 76, 76);
    set_int_range(w, id, 61, 61);
    set_damage(w, id, 13, 24);
    set_skill_range(w, id, SkillName::Wrestling, 15.0, 37.0);
    set_skill_range(w, id, SkillName::Parry, 45.0, 60.0);
    set_skill_range(w, id, SkillName::Archery, 66.0, 97.0);
    set_skill_range(w, id, SkillName::Magery, 62.0, 62.0);
    set_skill_range(w, id, SkillName::Swords, 35.0, 57.0);
    set_skill_range(w, id, SkillName::Fencing, 15.0, 37.0);
    set_skill_range(w, id, SkillName::Tactics, 65.0, 87.0);
    fame(w, id, 100);
    karma(w, id, 125);
    let fem = rnd::bool(w);
    muo_server::mobile::part1::props_b::set_female(w, id, fem);
    if fem {
        body(w, id, 0x191);
        name_random(w, id, "female");
    } else {
        body(w, id, 0x190);
        name_random(w, id, "male");
    }
    let h = utility::random_neutral_hue(&mut w.rng);
    equip_new(w, id, "Server.Items.Shoes", Some(h));
    equip_new(w, id, "Server.Items.Shirt", None);
    match rnd::below(w, 3) {
        0 => {
            equip_new(w, id, "Server.Items.Longsword", None);
        }
        1 => {
            equip_new(w, id, "Server.Items.VikingSword", None);
        }
        2 => {
            equip_new(w, id, "Server.Items.Broadsword", None);
        }
        _ => {}
    }
    equip_new(w, id, "Server.Items.StuddedChest", None);
    equip_new(w, id, "Server.Items.StuddedArms", None);
    equip_new(w, id, "Server.Items.StuddedGloves", None);
    equip_new(w, id, "Server.Items.StuddedLegs", None);
    equip_new(w, id, "Server.Items.StuddedGorget", None);
    id
}
fn hire_ranger_generate_loot(w: &mut World, id: MobileId) {
    add_loot(w, id, loot_pack::average(w));
    pack_new_amount(w, id, "Server.Items.Arrow", 20);
    pack_gold_range(w, id, 10, 75);
}
creature_leaf! {
    HIRE_RANGER, HIRE_RANGER_C = "Server.Mobiles.HireRanger" : BASE_HIRE / BASE_HIRE_C, ctor Some(hire_ranger_new), encoded true;
    c {
        generate_loot: hire_ranger_generate_loot,
    }
    m {
    }
}

fn hire_ranger_archer_new(w: &mut World) -> MobileId {
    let id = new_hire(w, &HIRE_RANGER_ARCHER, AIType::AiArcher, true);
    let v = utility::random_dyed_hue(&mut w.rng);
    speech_hue(w, id, v);
    let skin = muo_content::vendor::human_random_skin_hue(w);
    hue(w, id, skin);
    title(w, id, "the ranger");
    muo_content::vendor::roll_hair_item(w, id);
    muo_content::vendor::roll_hair_hue(w, id);
    muo_content::vendor::roll_facial_hair(w, id);
    set_str_range(w, id, 91, 91);
    set_dex_range(w, id, 76, 76);
    set_int_range(w, id, 61, 61);
    set_damage(w, id, 13, 24);
    set_skill_range(w, id, SkillName::Wrestling, 15.0, 37.0);
    set_skill_range(w, id, SkillName::Parry, 45.0, 60.0);
    set_skill_range(w, id, SkillName::Archery, 66.0, 97.0);
    set_skill_range(w, id, SkillName::Magery, 62.0, 62.0);
    set_skill_range(w, id, SkillName::Swords, 35.0, 57.0);
    set_skill_range(w, id, SkillName::Fencing, 15.0, 37.0);
    set_skill_range(w, id, SkillName::Tactics, 65.0, 87.0);
    fame(w, id, 100);
    karma(w, id, 125);
    let fem = rnd::bool(w);
    muo_server::mobile::part1::props_b::set_female(w, id, fem);
    if fem {
        body(w, id, 0x191);
        name_random(w, id, "female");
    } else {
        body(w, id, 0x190);
        name_random(w, id, "male");
    }
    let h = utility::random_neutral_hue(&mut w.rng);
    equip_new(w, id, "Server.Items.Shoes", Some(h));
    equip_new(w, id, "Server.Items.Shirt", None);
    match rnd::below(w, 2) {
        0 => {
            equip_new(w, id, "Server.Items.Bow", None);
        }
        1 => {
            equip_new(w, id, "Server.Items.CompositeBow", None);
        }
        _ => {}
    }
    equip_new(w, id, "Server.Items.RangerChest", None);
    equip_new(w, id, "Server.Items.RangerArms", None);
    equip_new(w, id, "Server.Items.RangerGloves", None);
    equip_new(w, id, "Server.Items.RangerGorget", None);
    equip_new(w, id, "Server.Items.RangerLegs", None);
    id
}
fn hire_ranger_archer_generate_loot(w: &mut World, id: MobileId) {
    pack_new_amount(w, id, "Server.Items.Arrow", 100);
}
creature_leaf! {
    HIRE_RANGER_ARCHER, HIRE_RANGER_ARCHER_C = "Server.Mobiles.HireRangerArcher" : BASE_HIRE / BASE_HIRE_C, ctor Some(hire_ranger_archer_new), encoded true;
    c {
        generate_loot: hire_ranger_archer_generate_loot,
    }
    m {
        click_title: |_w, _| false,
    }
}

fn hire_sailor_new(w: &mut World) -> MobileId {
    let id = new_hire(w, &HIRE_SAILOR, AIType::AiMelee, false);
    let v = utility::random_dyed_hue(&mut w.rng);
    speech_hue(w, id, v);
    let skin = muo_content::vendor::human_random_skin_hue(w);
    hue(w, id, skin);
    title(w, id, "the sailor");
    muo_content::vendor::roll_hair_item(w, id);
    muo_content::vendor::roll_hair_hue(w, id);
    muo_content::vendor::roll_facial_hair(w, id);
    set_str(w, id, 86);
    set_dex(w, id, 66);
    set_int(w, id, 41);
    set_damage(w, id, 10, 23);
    set_skill_range(w, id, SkillName::Stealing, 66.0, 97.5);
    set_skill_range(w, id, SkillName::Peacemaking, 65.0, 87.5);
    set_skill_range(w, id, SkillName::MagicResist, 25.0, 47.5);
    set_skill_range(w, id, SkillName::Healing, 65.0, 87.5);
    set_skill_range(w, id, SkillName::Tactics, 65.0, 87.5);
    set_skill_range(w, id, SkillName::Fencing, 65.0, 87.5);
    set_skill_range(w, id, SkillName::Parry, 45.0, 60.5);
    set_skill_range(w, id, SkillName::Lockpicking, 65.0, 87.0);
    set_skill_range(w, id, SkillName::Hiding, 65.0, 87.0);
    set_skill_range(w, id, SkillName::Snooping, 65.0, 87.0);
    fame(w, id, 100);
    karma(w, id, 0);
    let fem = rnd::bool(w);
    muo_server::mobile::part1::props_b::set_female(w, id, fem);
    if fem {
        body(w, id, 0x191);
        name_random(w, id, "female");
        let h = utility::random_neutral_hue(&mut w.rng);
        equip_new(w, id, "Server.Items.ShortPants", Some(h));
    } else {
        body(w, id, 0x190);
        name_random(w, id, "male");
        let h = utility::random_neutral_hue(&mut w.rng);
        equip_new(w, id, "Server.Items.ShortPants", Some(h));
    }
    let h = utility::random_neutral_hue(&mut w.rng);
    equip_new(w, id, "Server.Items.Shoes", Some(h));
    equip_new(w, id, "Server.Items.Cutlass", None);
    match rnd::below(w, 2) {
        0 => {
            let h = utility::random_dyed_hue(&mut w.rng);
            equip_new(w, id, "Server.Items.Doublet", Some(h));
        }
        1 => {
            let h = utility::random_dyed_hue(&mut w.rng);
            equip_new(w, id, "Server.Items.Shirt", Some(h));
        }
        _ => {}
    }
    id
}
creature_leaf! {
    HIRE_SAILOR, HIRE_SAILOR_C = "Server.Mobiles.HireSailor" : BASE_HIRE / BASE_HIRE_C, ctor Some(hire_sailor_new), encoded true;
    c {
    }
    m {
        click_title: |_w, _| false,
    }
}

fn hire_thief_new(w: &mut World) -> MobileId {
    let id = new_hire(w, &HIRE_THIEF, AIType::AiMelee, false);
    let v = utility::random_dyed_hue(&mut w.rng);
    speech_hue(w, id, v);
    let skin = muo_content::vendor::human_random_skin_hue(w);
    hue(w, id, skin);
    title(w, id, "the thief");
    muo_content::vendor::roll_hair_item(w, id);
    muo_content::vendor::roll_hair_hue(w, id);
    muo_content::vendor::roll_facial_hair(w, id);
    set_str_range(w, id, 81, 95);
    set_dex_range(w, id, 86, 100);
    set_int_range(w, id, 61, 75);
    set_damage(w, id, 10, 23);
    set_skill_range(w, id, SkillName::Stealing, 66.0, 97.5);
    set_skill_range(w, id, SkillName::Peacemaking, 65.0, 87.5);
    set_skill_range(w, id, SkillName::MagicResist, 25.0, 47.5);
    set_skill_range(w, id, SkillName::Healing, 65.0, 87.5);
    set_skill_range(w, id, SkillName::Tactics, 65.0, 87.5);
    set_skill_range(w, id, SkillName::Fencing, 65.0, 87.5);
    set_skill_range(w, id, SkillName::Parry, 45.0, 60.5);
    set_skill_range(w, id, SkillName::Lockpicking, 65.0, 87.0);
    set_skill_range(w, id, SkillName::Hiding, 65.0, 87.0);
    set_skill_range(w, id, SkillName::Snooping, 65.0, 87.0);
    fame(w, id, 100);
    karma(w, id, 0);
    let fem = rnd::bool(w);
    muo_server::mobile::part1::props_b::set_female(w, id, fem);
    if fem {
        body(w, id, 0x191);
        name_random(w, id, "female");
        match rnd::below(w, 2) {
            0 => {
                let h = utility::random_neutral_hue(&mut w.rng);
                equip_new(w, id, "Server.Items.Skirt", Some(h));
            }
            1 => {
                let h = utility::random_neutral_hue(&mut w.rng);
                equip_new(w, id, "Server.Items.Kilt", Some(h));
            }
            _ => {}
        }
    } else {
        body(w, id, 0x190);
        name_random(w, id, "male");
        let h = utility::random_neutral_hue(&mut w.rng);
        equip_new(w, id, "Server.Items.ShortPants", Some(h));
    }
    let h = utility::random_neutral_hue(&mut w.rng);
    equip_new(w, id, "Server.Items.Sandals", Some(h));
    equip_new(w, id, "Server.Items.Dagger", None);
    match rnd::below(w, 2) {
        0 => {
            let h = utility::random_neutral_hue(&mut w.rng);
            equip_new(w, id, "Server.Items.Doublet", Some(h));
        }
        1 => {
            let h = utility::random_neutral_hue(&mut w.rng);
            equip_new(w, id, "Server.Items.Shirt", Some(h));
        }
        _ => {}
    }
    pack_gold_range(w, id, 0, 25);
    id
}
creature_leaf! {
    HIRE_THIEF, HIRE_THIEF_C = "Server.Mobiles.HireThief" : BASE_HIRE / BASE_HIRE_C, ctor Some(hire_thief_new), encoded true;
    c {
    }
    m {
        click_title: |_w, _| false,
    }
}

fn actor_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &ACTOR,
        AIType::AiAnimal,
        FightMode::None,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    muo_server::mobile::part2::stats::init_stats(w, id, 31, 41, 51);
    muo_content::creature::speeds::set_speed(w, id, 0.2, 0.4, true);
    let v = utility::random_dyed_hue(&mut w.rng);
    speech_hue(w, id, v);
    let skin = muo_content::vendor::human_random_skin_hue(w);
    hue(w, id, skin);
    let fem = rnd::bool(w);
    muo_server::mobile::part1::props_b::set_female(w, id, fem);
    if fem {
        body(w, id, 0x191);
        name_random(w, id, "female");
        let h = utility::random_dyed_hue(&mut w.rng);
        add_new_hued(w, id, "Server.Items.FancyDress", Some(h));
        title(w, id, "the actress");
    } else {
        body(w, id, 0x190);
        name_random(w, id, "male");
        let h = utility::random_neutral_hue(&mut w.rng);
        add_new_hued(w, id, "Server.Items.LongPants", Some(h));
        let h = utility::random_dyed_hue(&mut w.rng);
        add_new_hued(w, id, "Server.Items.FancyShirt", Some(h));
        title(w, id, "the actor");
    }
    let h = utility::random_neutral_hue(&mut w.rng);
    add_new_hued(w, id, "Server.Items.Boots", Some(h));
    muo_content::vendor::assign_random_hair_random_hue(w, id);
    let pack = new_unmovable_pack(w);
    drop_gold(w, pack, 250, 300);
    add_pack(w, id, pack);
    id
}
creature_leaf! {
    ACTOR, ACTOR_C = "Server.Mobiles.Actor" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(actor_new), encoded false;
    c {
    }
    m {
        click_title: |_w, _| false,
    }
}

fn artist_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &ARTIST,
        AIType::AiAnimal,
        FightMode::None,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    muo_server::mobile::part2::stats::init_stats(w, id, 31, 41, 51);
    set_skill_range(w, id, SkillName::Healing, 36.0, 68.0);
    muo_content::creature::speeds::set_speed(w, id, 0.2, 0.4, true);
    let v = utility::random_dyed_hue(&mut w.rng);
    speech_hue(w, id, v);
    title(w, id, "the artist");
    let skin = muo_content::vendor::human_random_skin_hue(w);
    hue(w, id, skin);
    let fem = rnd::bool(w);
    muo_server::mobile::part1::props_b::set_female(w, id, fem);
    if fem {
        body(w, id, 0x191);
        name_random(w, id, "female");
    } else {
        body(w, id, 0x190);
        name_random(w, id, "male");
    }
    let h = utility::random_dyed_hue(&mut w.rng);
    add_new_hued(w, id, "Server.Items.Doublet", Some(h));
    let h = utility::random_neutral_hue(&mut w.rng);
    add_new_hued(w, id, "Server.Items.Sandals", Some(h));
    let h = utility::random_neutral_hue(&mut w.rng);
    add_new_hued(w, id, "Server.Items.ShortPants", Some(h));
    let h = utility::random_dyed_hue(&mut w.rng);
    add_new_hued(w, id, "Server.Items.HalfApron", Some(h));
    muo_content::vendor::assign_random_hair_random_hue(w, id);
    let pack = new_unmovable_pack(w);
    drop_gold(w, pack, 250, 300);
    add_pack(w, id, pack);
    id
}
creature_leaf! {
    ARTIST, ARTIST_C = "Server.Mobiles.Artist" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(artist_new), encoded false;
    c {
        can_teach: |_w, _| true,
    }
    m {
        click_title: |_w, _| false,
    }
}

fn gypsy_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &GYPSY,
        AIType::AiAnimal,
        FightMode::None,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    muo_server::mobile::part2::stats::init_stats(w, id, 31, 41, 51);
    set_skill_range(w, id, SkillName::Cooking, 65.0, 88.0);
    set_skill_range(w, id, SkillName::Snooping, 65.0, 88.0);
    set_skill_range(w, id, SkillName::Stealing, 65.0, 88.0);
    muo_content::creature::speeds::set_speed(w, id, 0.2, 0.4, true);
    let v = utility::random_dyed_hue(&mut w.rng);
    speech_hue(w, id, v);
    let skin = muo_content::vendor::human_random_skin_hue(w);
    hue(w, id, skin);
    let fem = rnd::bool(w);
    muo_server::mobile::part1::props_b::set_female(w, id, fem);
    if fem {
        body(w, id, 0x191);
        name_random(w, id, "female");
        let h = utility::random_dyed_hue(&mut w.rng);
        add_new_hued(w, id, "Server.Items.Kilt", Some(h));
        let h = utility::random_dyed_hue(&mut w.rng);
        add_new_hued(w, id, "Server.Items.Shirt", Some(h));
        add_new_hued(w, id, "Server.Items.ThighBoots", None);
        title(w, id, "the gypsy");
    } else {
        body(w, id, 0x190);
        name_random(w, id, "male");
        let h = utility::random_neutral_hue(&mut w.rng);
        add_new_hued(w, id, "Server.Items.ShortPants", Some(h));
        let h = utility::random_dyed_hue(&mut w.rng);
        add_new_hued(w, id, "Server.Items.Shirt", Some(h));
        add_new_hued(w, id, "Server.Items.Sandals", None);
        title(w, id, "the gypsy");
    }
    let h = utility::random_dyed_hue(&mut w.rng);
    add_new_hued(w, id, "Server.Items.Bandana", Some(h));
    add_new_hued(w, id, "Server.Items.Dagger", None);
    muo_content::vendor::assign_random_hair_random_hue(w, id);
    let pack = new_unmovable_pack(w);
    drop_gold(w, pack, 250, 300);
    add_pack(w, id, pack);
    id
}
creature_leaf! {
    GYPSY, GYPSY_C = "Server.Mobiles.Gypsy" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(gypsy_new), encoded false;
    c {
        can_teach: |_w, _| true,
    }
    m {
        click_title: |_w, _| false,
    }
}

fn harbor_master_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &HARBOR_MASTER,
        AIType::AiAnimal,
        FightMode::None,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    muo_server::mobile::part2::stats::init_stats(w, id, 31, 41, 51);
    set_skill_range(w, id, SkillName::Mining, 36.0, 68.0);
    muo_content::creature::speeds::set_speed(w, id, 0.2, 0.4, true);
    let v = utility::random_dyed_hue(&mut w.rng);
    speech_hue(w, id, v);
    let skin = muo_content::vendor::human_random_skin_hue(w);
    hue(w, id, skin);
    muo_server::mobile::part1::props_a::set_blessed(w, id, true);
    let fem = rnd::bool(w);
    muo_server::mobile::part1::props_b::set_female(w, id, fem);
    if fem {
        body(w, id, 0x191);
        name_random(w, id, "female");
        title(w, id, "the Harbor Mistress");
    } else {
        body(w, id, 0x190);
        name_random(w, id, "male");
        title(w, id, "the Harbor Master");
    }
    let h = utility::random_dyed_hue(&mut w.rng);
    add_new_hued(w, id, "Server.Items.Shirt", Some(h));
    add_new_hued(w, id, "Server.Items.Boots", None);
    let h = utility::random_neutral_hue(&mut w.rng);
    add_new_hued(w, id, "Server.Items.LongPants", Some(h));
    add_new_hued(w, id, "Server.Items.QuarterStaff", None);
    muo_content::vendor::assign_random_hair_random_hue(w, id);
    let pack = new_unmovable_pack(w);
    drop_gold(w, pack, 250, 300);
    add_pack(w, id, pack);
    id
}
creature_leaf! {
    HARBOR_MASTER, HARBOR_MASTER_C = "Server.Mobiles.HarborMaster" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(harbor_master_new), encoded false;
    c {
        can_teach: |_w, _| false,
    }
    m {
        click_title: |_w, _| false,
    }
}

fn ninja_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &NINJA,
        AIType::AiMelee,
        FightMode::Aggressor,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    title(w, id, "the ninja");
    muo_server::mobile::part2::stats::init_stats(w, id, 100, 100, 25);
    set_skill_range(w, id, SkillName::Fencing, 64.0, 80.0);
    set_skill_range(w, id, SkillName::Macing, 64.0, 80.0);
    set_skill_range(w, id, SkillName::Ninjitsu, 60.0, 80.0);
    set_skill_range(w, id, SkillName::Parry, 64.0, 80.0);
    set_skill_range(w, id, SkillName::Tactics, 64.0, 85.0);
    set_skill_range(w, id, SkillName::Swords, 64.0, 85.0);
    muo_content::creature::speeds::set_speed(w, id, 0.2, 0.4, true);
    let v = utility::random_dyed_hue(&mut w.rng);
    speech_hue(w, id, v);
    let skin = muo_content::vendor::human_random_skin_hue(w);
    hue(w, id, skin);
    let fem = rnd::bool(w);
    muo_server::mobile::part1::props_b::set_female(w, id, fem);
    if fem {
        body(w, id, 0x191);
        name_random(w, id, "female");
    } else {
        body(w, id, 0x190);
        name_random(w, id, "male");
    }
    if !npc_female(w, id) {
        add_new_hued(w, id, "Server.Items.LeatherNinjaHood", None);
    }
    add_new_hued(w, id, "Server.Items.LeatherNinjaPants", None);
    add_new_hued(w, id, "Server.Items.LeatherNinjaBelt", None);
    add_new_hued(w, id, "Server.Items.LeatherNinjaJacket", None);
    add_new_hued(w, id, "Server.Items.NinjaTabi", None);
    let hair_hue = utility::random_nondyed_hue(&mut w.rng);
    let hh = hair_hue;
    muo_content::vendor::assign_random_hair(w, id, hh);
    if rnd::below(w, 7) != 0 {
        let fh = hair_hue;
        muo_content::vendor::assign_random_facial_hair(w, id, fh);
    }
    pack_gold_range(w, id, 250, 300);
    id
}
creature_leaf! {
    NINJA, NINJA_C = "Server.Mobiles.Ninja" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(ninja_new), encoded true;
    c {
        can_teach: |_w, _| true,
    }
    m {
        click_title: |_w, _| false,
    }
}

fn samurai_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &SAMURAI,
        AIType::AiMelee,
        FightMode::Aggressor,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    title(w, id, "the samurai");
    muo_server::mobile::part2::stats::init_stats(w, id, 100, 100, 25);
    set_skill_range(w, id, SkillName::ArmsLore, 64.0, 80.0);
    set_skill_range(w, id, SkillName::Bushido, 64.0, 85.0);
    set_skill_range(w, id, SkillName::Parry, 64.0, 80.0);
    set_skill_range(w, id, SkillName::Swords, 64.0, 85.0);
    muo_content::creature::speeds::set_speed(w, id, 0.2, 0.4, true);
    let v = utility::random_dyed_hue(&mut w.rng);
    speech_hue(w, id, v);
    let skin = muo_content::vendor::human_random_skin_hue(w);
    hue(w, id, skin);
    let fem = rnd::bool(w);
    muo_server::mobile::part1::props_b::set_female(w, id, fem);
    if fem {
        body(w, id, 0x191);
        name_random(w, id, "female");
    } else {
        body(w, id, 0x190);
        name_random(w, id, "male");
    }
    match rnd::below(w, 3) {
        0 => {
            add_new_hued(w, id, "Server.Items.Lajatang", None);
        }
        1 => {
            add_new_hued(w, id, "Server.Items.Wakizashi", None);
        }
        2 => {
            add_new_hued(w, id, "Server.Items.NoDachi", None);
        }
        _ => {}
    }
    match rnd::below(w, 3) {
        0 => {
            add_new_hued(w, id, "Server.Items.LeatherSuneate", None);
        }
        1 => {
            add_new_hued(w, id, "Server.Items.PlateSuneate", None);
        }
        2 => {
            add_new_hued(w, id, "Server.Items.StuddedHaidate", None);
        }
        _ => {}
    }
    match rnd::below(w, 4) {
        0 => {
            add_new_hued(w, id, "Server.Items.LeatherJingasa", None);
        }
        1 => {
            add_new_hued(w, id, "Server.Items.ChainHatsuburi", None);
        }
        2 => {
            add_new_hued(w, id, "Server.Items.HeavyPlateJingasa", None);
        }
        3 => {
            add_new_hued(w, id, "Server.Items.DecorativePlateKabuto", None);
        }
        _ => {}
    }
    add_new_hued(w, id, "Server.Items.LeatherDo", None);
    add_new_hued(w, id, "Server.Items.LeatherHiroSode", None);
    let h = utility::random_nondyed_hue(&mut w.rng);
    add_new_hued(w, id, "Server.Items.SamuraiTabi", Some(h));
    let hair_hue = utility::random_nondyed_hue(&mut w.rng);
    let hh = hair_hue;
    muo_content::vendor::assign_random_hair(w, id, hh);
    if rnd::below(w, 7) != 0 {
        let fh = hair_hue;
        muo_content::vendor::assign_random_facial_hair(w, id, fh);
    }
    pack_gold_range(w, id, 250, 300);
    id
}
creature_leaf! {
    SAMURAI, SAMURAI_C = "Server.Mobiles.Samurai" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(samurai_new), encoded true;
    c {
        can_teach: |_w, _| true,
    }
    m {
        click_title: |_w, _| false,
    }
}

fn sculptor_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &SCULPTOR,
        AIType::AiAnimal,
        FightMode::None,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    muo_server::mobile::part2::stats::init_stats(w, id, 31, 41, 51);
    muo_content::creature::speeds::set_speed(w, id, 0.2, 0.4, true);
    let v = utility::random_dyed_hue(&mut w.rng);
    speech_hue(w, id, v);
    title(w, id, "the sculptor");
    let skin = muo_content::vendor::human_random_skin_hue(w);
    hue(w, id, skin);
    let fem = rnd::bool(w);
    muo_server::mobile::part1::props_b::set_female(w, id, fem);
    if fem {
        body(w, id, 0x191);
        name_random(w, id, "female");
        let h = utility::random_neutral_hue(&mut w.rng);
        add_new_hued(w, id, "Server.Items.Kilt", Some(h));
    } else {
        body(w, id, 0x190);
        name_random(w, id, "male");
        let h = utility::random_neutral_hue(&mut w.rng);
        add_new_hued(w, id, "Server.Items.LongPants", Some(h));
    }
    let h = utility::random_neutral_hue(&mut w.rng);
    add_new_hued(w, id, "Server.Items.Doublet", Some(h));
    add_new_hued(w, id, "Server.Items.HalfApron", None);
    muo_content::vendor::assign_random_hair_random_hue(w, id);
    let pack = new_unmovable_pack(w);
    drop_gold(w, pack, 250, 300);
    add_pack(w, id, pack);
    id
}
creature_leaf! {
    SCULPTOR, SCULPTOR_C = "Server.Mobiles.Sculptor" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(sculptor_new), encoded false;
    c {
    }
    m {
        click_title: |_w, _| false,
    }
}

pub(crate) fn register(types: &mut TypeRegistry) {
    // `BaseHire` is concrete (its serialization is in the table); it has no `[Constructible]`.
    types.register_mobile(&super::hires::BASE_HIRE, &[]);
    types.register_mobile(&HIRE_BARD, &[]);
    types.register_mobile(&HIRE_BARD_ARCHER, &[]);
    types.register_mobile(&HIRE_BEGGAR, &[]);
    types.register_mobile(&HIRE_FIGHTER, &[]);
    types.register_mobile(&HIRE_MAGE, &[]);
    types.register_mobile(&HIRE_PALADIN, &[]);
    types.register_mobile(&HIRE_PEASANT, &[]);
    types.register_mobile(&HIRE_RANGER, &[]);
    types.register_mobile(&HIRE_RANGER_ARCHER, &[]);
    types.register_mobile(&HIRE_SAILOR, &[]);
    types.register_mobile(&HIRE_THIEF, &[]);
    types.register_mobile(&ACTOR, &[]);
    types.register_mobile(&ARTIST, &[]);
    types.register_mobile(&GYPSY, &[]);
    types.register_mobile(&HARBOR_MASTER, &[]);
    types.register_mobile(&NINJA, &[]);
    types.register_mobile(&SAMURAI, &[]);
    types.register_mobile(&SCULPTOR, &[]);
}

#[cfg(test)]
pub(crate) fn all() -> Vec<&'static muo_server::mobile::MobileType> {
    vec![
        &HIRE_BARD,
        &HIRE_BARD_ARCHER,
        &HIRE_BEGGAR,
        &HIRE_FIGHTER,
        &HIRE_MAGE,
        &HIRE_PALADIN,
        &HIRE_PEASANT,
        &HIRE_RANGER,
        &HIRE_RANGER_ARCHER,
        &HIRE_SAILOR,
        &HIRE_THIEF,
        &ACTOR,
        &ARTIST,
        &GYPSY,
        &HARBOR_MASTER,
        &NINJA,
        &SAMURAI,
        &SCULPTOR,
    ]
}
