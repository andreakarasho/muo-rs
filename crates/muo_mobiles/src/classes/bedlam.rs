//! `LadyMarai`: a paragon skeletal knight of Bedlam.

use muo_server::World;
use muo_server::ids::MobileId;
use muo_server::type_registry::TypeRegistry;

use crate::leaf::creature_leaf;
use crate::monsters::leaves::{SKELETAL_KNIGHT, SKELETAL_KNIGHT_C};

fn lady_marai_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &LADY_MARAI)
}

creature_leaf! {
    /// `Server.Mobiles.LadyMarai`.
    LADY_MARAI, LADY_MARAI_C = "Server.Mobiles.LadyMarai" : SKELETAL_KNIGHT / SKELETAL_KNIGHT_C, ctor Some(lady_marai_new), encoded false;
    c {
        corpse_name: muo_content::templates::creature_props::corpse_name,
        default_name: muo_content::templates::creature_props::default_name,
        gives_ml_minor_artifact: muo_content::templates::creature_props::gives_ml_minor_artifact,
        generate_loot: crate::cprops::generate_loot,
        get_weapon_ability: crate::cprops::get_weapon_ability,
    }
    m {
    }
}

pub fn register(types: &mut TypeRegistry) {
    types.register_mobile(&LADY_MARAI, &[]);
}
