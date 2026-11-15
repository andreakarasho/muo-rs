//! The constructors of the humanoid hireables and townfolk, as table classes.

use super::humans_support::*;
use crate::common::*;
use muo_content::creature::{BASE_CREATURE, BASE_CREATURE_C};
use muo_content::hooks::rnd;
use crate::template_keys_npcs as npc_keys;
use muo_content::loot_pack;
use muo_content::templates;
use muo_server::ids::MobileId;
use muo_server::type_registry::TypeRegistry;
use muo_server::world::World;
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
    HIRE_BARD, HIRE_BARD_C = "Server.Mobiles.HireBard" : BASE_HIRE / BASE_HIRE_C, ctor Some(|w| new_template_hire(w, &HIRE_BARD)), encoded true;
    c {
        generate_loot: hire_bard_generate_loot,
    }
    m {
        click_title: npc_keys::click_title,
    }
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
    HIRE_BARD_ARCHER, HIRE_BARD_ARCHER_C = "Server.Mobiles.HireBardArcher" : BASE_HIRE / BASE_HIRE_C, ctor Some(|w| new_template_hire(w, &HIRE_BARD_ARCHER)), encoded true;
    c {
        generate_loot: hire_bard_archer_generate_loot,
    }
    m {
        click_title: npc_keys::click_title,
    }
}

creature_leaf! {
    HIRE_BEGGAR, HIRE_BEGGAR_C = "Server.Mobiles.HireBeggar" : BASE_HIRE / BASE_HIRE_C, ctor Some(|w| new_template_hire(w, &HIRE_BEGGAR)), encoded true;
    c {
    }
    m {
        click_title: npc_keys::click_title,
    }
}

creature_leaf! {
    HIRE_FIGHTER, HIRE_FIGHTER_C = "Server.Mobiles.HireFighter" : BASE_HIRE / BASE_HIRE_C, ctor Some(|w| new_template_hire(w, &HIRE_FIGHTER)), encoded true;
    c {
    }
    m {
        click_title: npc_keys::click_title,
    }
}

creature_leaf! {
    HIRE_MAGE, HIRE_MAGE_C = "Server.Mobiles.HireMage" : BASE_HIRE / BASE_HIRE_C, ctor Some(|w| new_template_hire(w, &HIRE_MAGE)), encoded true;
    c {
    }
    m {
        click_title: npc_keys::click_title,
    }
}

creature_leaf! {
    HIRE_PALADIN, HIRE_PALADIN_C = "Server.Mobiles.HirePaladin" : BASE_HIRE / BASE_HIRE_C, ctor Some(|w| new_template_hire(w, &HIRE_PALADIN)), encoded true;
    c {
    }
    m {
        click_title: npc_keys::click_title,
    }
}

creature_leaf! {
    HIRE_PEASANT, HIRE_PEASANT_C = "Server.Mobiles.HirePeasant" : BASE_HIRE / BASE_HIRE_C, ctor Some(|w| new_template_hire(w, &HIRE_PEASANT)), encoded true;
    c {
    }
    m {
        click_title: npc_keys::click_title,
    }
}

fn hire_ranger_generate_loot(w: &mut World, id: MobileId) {
    add_loot(w, id, loot_pack::average(w));
    pack_new_amount(w, id, "Server.Items.Arrow", 20);
    pack_gold_range(w, id, 10, 75);
}
creature_leaf! {
    HIRE_RANGER, HIRE_RANGER_C = "Server.Mobiles.HireRanger" : BASE_HIRE / BASE_HIRE_C, ctor Some(|w| new_template_hire(w, &HIRE_RANGER)), encoded true;
    c {
        generate_loot: hire_ranger_generate_loot,
    }
    m {
    }
}

fn hire_ranger_archer_generate_loot(w: &mut World, id: MobileId) {
    pack_new_amount(w, id, "Server.Items.Arrow", 100);
}
creature_leaf! {
    HIRE_RANGER_ARCHER, HIRE_RANGER_ARCHER_C = "Server.Mobiles.HireRangerArcher" : BASE_HIRE / BASE_HIRE_C, ctor Some(|w| new_template_hire(w, &HIRE_RANGER_ARCHER)), encoded true;
    c {
        generate_loot: hire_ranger_archer_generate_loot,
    }
    m {
        click_title: npc_keys::click_title,
    }
}

creature_leaf! {
    HIRE_SAILOR, HIRE_SAILOR_C = "Server.Mobiles.HireSailor" : BASE_HIRE / BASE_HIRE_C, ctor Some(|w| new_template_hire(w, &HIRE_SAILOR)), encoded true;
    c {
    }
    m {
        click_title: npc_keys::click_title,
    }
}

creature_leaf! {
    HIRE_THIEF, HIRE_THIEF_C = "Server.Mobiles.HireThief" : BASE_HIRE / BASE_HIRE_C, ctor Some(|w| new_template_hire(w, &HIRE_THIEF)), encoded true;
    c {
    }
    m {
        click_title: npc_keys::click_title,
    }
}

creature_leaf! {
    ACTOR, ACTOR_C = "Server.Mobiles.Actor" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(|w| templates::new_creature(w, &ACTOR)), encoded false;
    c {
    }
    m {
        click_title: npc_keys::click_title,
    }
}

creature_leaf! {
    ARTIST, ARTIST_C = "Server.Mobiles.Artist" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(|w| templates::new_creature(w, &ARTIST)), encoded false;
    c {
        can_teach: npc_keys::can_teach,
    }
    m {
        click_title: npc_keys::click_title,
    }
}

creature_leaf! {
    GYPSY, GYPSY_C = "Server.Mobiles.Gypsy" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(|w| templates::new_creature(w, &GYPSY)), encoded false;
    c {
        can_teach: npc_keys::can_teach,
    }
    m {
        click_title: npc_keys::click_title,
    }
}

creature_leaf! {
    HARBOR_MASTER, HARBOR_MASTER_C = "Server.Mobiles.HarborMaster" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(|w| templates::new_creature(w, &HARBOR_MASTER)), encoded false;
    c {
        can_teach: npc_keys::can_teach,
    }
    m {
        click_title: npc_keys::click_title,
    }
}

creature_leaf! {
    NINJA, NINJA_C = "Server.Mobiles.Ninja" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(|w| templates::new_creature(w, &NINJA)), encoded true;
    c {
        can_teach: npc_keys::can_teach,
    }
    m {
        click_title: npc_keys::click_title,
    }
}

creature_leaf! {
    SAMURAI, SAMURAI_C = "Server.Mobiles.Samurai" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(|w| templates::new_creature(w, &SAMURAI)), encoded true;
    c {
        can_teach: npc_keys::can_teach,
    }
    m {
        click_title: npc_keys::click_title,
    }
}

creature_leaf! {
    SCULPTOR, SCULPTOR_C = "Server.Mobiles.Sculptor" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(|w| templates::new_creature(w, &SCULPTOR)), encoded false;
    c {
    }
    m {
        click_title: npc_keys::click_title,
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
