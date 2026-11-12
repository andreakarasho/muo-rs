//! `LadyMarai`: a paragon skeletal knight of Bedlam.

use muo_content::items::weapon::Ability;
use muo_content::loot_pack;
use muo_server::World;
use muo_server::ids::MobileId;
use muo_server::type_registry::TypeRegistry;

use crate::common::*;
use crate::leaf::creature_leaf;
use crate::monsters::leaves::{SKELETAL_KNIGHT, SKELETAL_KNIGHT_C, skeletal_knight_init};

fn lady_marai_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &LADY_MARAI,
        AIType::AiMelee,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    skeletal_knight_init(w, id);
    is_paragon(w, id, true);
    hue(w, id, 0x21);
    set_str_range(w, id, 221, 304);
    set_dex_range(w, id, 98, 138);
    set_int_range(w, id, 54, 99);
    set_hits_range(w, id, 694, 846);
    set_damage(w, id, 15, 25);
    set_damage_type(w, id, ResistanceType::Physical, 40);
    set_damage_type(w, id, ResistanceType::Cold, 60);
    set_resistance_range(w, id, ResistanceType::Physical, 55, 65);
    set_resistance_range(w, id, ResistanceType::Fire, 40, 50);
    set_resistance_range(w, id, ResistanceType::Cold, 70, 80);
    set_resistance_range(w, id, ResistanceType::Poison, 40, 50);
    set_resistance_range(w, id, ResistanceType::Energy, 50, 60);
    set_skill_range(w, id, SkillName::Wrestling, 126.6, 137.2);
    set_skill_range(w, id, SkillName::Tactics, 128.7, 134.5);
    set_skill_range(w, id, SkillName::MagicResist, 102.1, 119.1);
    set_skill_range(w, id, SkillName::Anatomy, 126.2, 136.5);
    fame(w, id, 18000);
    karma(w, id, -18000);
    id
}

creature_leaf! {
    /// `Server.Mobiles.LadyMarai`.
    LADY_MARAI, LADY_MARAI_C = "Server.Mobiles.LadyMarai" : SKELETAL_KNIGHT / SKELETAL_KNIGHT_C, ctor Some(lady_marai_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a Lady Marai corpse"),
        default_name: |_w, _id| Some("Lady Marai"),
        gives_ml_minor_artifact: |_w, _id| true,
        generate_loot: |w, id| add_loot_n(w, id, loot_pack::ultra_rich(w), 3),
        get_weapon_ability: |_w, _id| Some(Ability::CrushingBlow),
    }
    m {
    }
}

pub fn register(types: &mut TypeRegistry) {
    types.register_mobile(&LADY_MARAI, &[]);
}
