//! The enraged creatures: the animals a meer summons. None has a parameterless
//! constructor (`new EnragedRabbit(summoner)`), so the registry cannot build them: use the `new_*` functions.

use muo_content::ai::movement;
use muo_content::creature::control;
use muo_content::creature::{BASE_CREATURE, BASE_CREATURE_C};
use muo_content::mx;
use muo_primitives::MessageType;
use muo_server::World;
use muo_server::ids::MobileId;
use muo_server::mobile::part2::messages;
use muo_server::type_registry::TypeRegistry;

use crate::leaf::creature_leaf;

fn master(w: &World, id: MobileId) -> Option<MobileId> {
    muo_content::creature::cd(w, id).and_then(|d| d.master)
}

/// `Combat(mobile)`: the combatant is there, alive and not a dead bonded pet.
fn combat(w: &World, m: MobileId) -> bool {
    mx::combatant(w, m).is_some_and(|c| {
        !mx::deleted(w, c)
            && !w.mobile(c).is_some_and(|x| (x.ty.is_dead_bonded_pet)(w, c))
            && mx::alive(w, c)
    })
}

/// `m != Master && base.IsEnemy(m)`: the meer's FightMode.Evil against our negative karma would make us enemies.
fn is_enemy(w: &mut World, id: MobileId, m: MobileId) -> bool {
    Some(m) != master(w, id) && (BASE_CREATURE_C.is_enemy)(w, id, m)
}

fn on_think(w: &mut World, id: MobileId) {
    let m = master(w, id);
    if m.is_none_or(|m| mx::deleted(w, m)) {
        muo_server::mobile::base::delete(w, id);
    } else if !combat(w, id) {
        // Like a "come" command, but they never actually follow.
        movement::move_to(w, id, m.unwrap(), 5);
    } else if !combat(w, m.unwrap()) {
        let m = m.unwrap();
        let c = mx::combatant(w, id).unwrap();
        let is_pet =
            muo_content::creature::cd(w, c).is_some() && control::get_master(w, c).is_some();
        if mx::player(w, c) || is_pet {
            mx::set_combatant(w, m, Some(c));
        }
    } else {
        (BASE_CREATURE_C.on_think)(w, id);
    }
}

fn on_single_click(w: &mut World, id: MobileId, from: MobileId) {
    (BASE_CREATURE.on_single_click)(w, id, from);
    let ns = w.mobile(from).and_then(|m| m.net_state);
    // enraged
    messages::private_overhead_message_number(w, id, MessageType::REGULAR, 0x3B2, 1060768, "", ns);
}

fn add_name_properties(
    w: &World,
    id: MobileId,
    list: &mut muo_server::property_list::PropertyList,
) {
    (BASE_CREATURE.add_name_properties)(w, id, list);
    list.add_localized(1060768); // enraged
}

creature_leaf! {
    /// `Server.Mobiles.BaseEnraged` (not constructible by the registry).
    BASE_ENRAGED, BASE_ENRAGED_C = "Server.Mobiles.BaseEnraged" : BASE_CREATURE / BASE_CREATURE_C, ctor None, encoded false;
    c {
        is_enemy: is_enemy,
        on_think: on_think,
    }
    m {
        on_single_click: on_single_click,
        add_name_properties: add_name_properties,
    }
}

macro_rules! enraged {
    ($new:ident, $static:ident, $cls:ident, $full:literal) => {
        /// `new <Class>(summoner)`: its template (`BaseEnraged`'s first), then `enraged.rhai` with the summoner.
        pub fn $new(w: &mut World, summoner: MobileId) -> MobileId {
            use muo_server::script_hooks::ScriptArg;
            crate::cprops::new_creature_with(w, &$static, &[ScriptArg::Mobile(Some(summoner))])
        }
        creature_leaf! {
            #[doc = $full]
            $static, $cls = $full : BASE_ENRAGED / BASE_ENRAGED_C, ctor None, encoded false;
            c {
                corpse_name: muo_content::templates::creature_props::corpse_name,
                default_name: muo_content::templates::creature_props::default_name,
            }
            m {
                get_attack_sound: crate::cprops::get_attack_sound,
                get_hurt_sound: crate::cprops::get_hurt_sound,
                get_death_sound: crate::cprops::get_death_sound,
            }
        }
    };
}

enraged!(
    new_enraged_rabbit,
    ENRAGED_RABBIT,
    ENRAGED_RABBIT_C,
    "Server.Mobiles.EnragedRabbit"
);
enraged!(
    new_enraged_hart,
    ENRAGED_HART,
    ENRAGED_HART_C,
    "Server.Mobiles.EnragedHart"
);
enraged!(
    new_enraged_hind,
    ENRAGED_HIND,
    ENRAGED_HIND_C,
    "Server.Mobiles.EnragedHind"
);
enraged!(
    new_enraged_black_bear,
    ENRAGED_BLACK_BEAR,
    ENRAGED_BLACK_BEAR_C,
    "Server.Mobiles.EnragedBlackBear"
);
enraged!(
    new_enraged_eagle,
    ENRAGED_EAGLE,
    ENRAGED_EAGLE_C,
    "Server.Mobiles.EnragedEagle"
);

pub fn register(types: &mut TypeRegistry) {
    for t in [
        &BASE_ENRAGED,
        &ENRAGED_RABBIT,
        &ENRAGED_HART,
        &ENRAGED_HIND,
        &ENRAGED_BLACK_BEAR,
        &ENRAGED_EAGLE,
    ] {
        types.register_mobile(t, &[]);
    }
}
